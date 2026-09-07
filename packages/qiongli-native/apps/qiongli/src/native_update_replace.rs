#![cfg_attr(not(target_os = "macos"), allow(dead_code, unused_imports, unused_mut))]

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use qiongli_config::{
    UpdateLastKnownGood, UpdateReleaseChannel, UpdateState, UpdateStateStore,
    UpdateStreamPreference, UpdateTransactionPhase, resolve_config_root,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::update_reconcile::{
    acquire_replacement_lock, activate_prepared_reconciliation, cleanup_committed_reconciliation,
    cleanup_rolled_back_reconciliation, load_reconciliation_journal, reconciliation_journal_sha256,
    rollback_active_reconciliation, verify_active_reconciliation, verify_prepared_reconciliation,
};

const JOURNAL_FILE: &str = "replacement-journal.json";
const HEALTH_TOKEN_FILE: &str = "replacement-health-token";
const FAILED_APPLICATION_DIRECTORY: &str = "failed-application";
const APPLICATION_NAME: &str = "Qiongli.app";
const CANONICAL_BINARY_NAME: &str = "qiongli-cli";
const UPDATE_HELPER_NAME: &str = "qiongli-update-helper";
const JOURNAL_DOCUMENT_KIND: &str = "qiongli-native-replacement";
const JOURNAL_SCHEMA_VERSION: u32 = 1;
const MAX_JOURNAL_BYTES: u64 = 64 * 1024;
const MAX_TOKEN_BYTES: u64 = 128;
const MINIMUM_AVAILABLE_BYTES: u64 = 64 * 1024 * 1024;
const CHILD_TIMEOUT: Duration = Duration::from_secs(60);
const EXIT_HANDOFF_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(20);
#[cfg(test)]
const TEST_INTERRUPTION: &str = "native-update-test-interruption";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReplacementCheckpoint {
    BeforeAwaitingExit,
    AfterAwaitingExit,
    BeforeActivating,
    AfterActivating,
    AfterOldApplicationBackup,
    AfterNewApplicationActivation,
    BeforeHealthWindow,
    AfterHealthWindow,
    BeforeHealthCommit,
    AfterHealthCommit,
}

#[cfg(test)]
thread_local! {
    static INJECTED_REPLACEMENT_CHECKPOINT:
        std::cell::Cell<Option<ReplacementCheckpoint>> = const { std::cell::Cell::new(None) };
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedReplacement {
    pub(crate) revision: u64,
    pub(crate) transaction_id: String,
    pub(crate) target_version: String,
    pub(crate) generation: u64,
    pub(crate) cleanup_required: bool,
}

pub(crate) struct ReplacementPreparation<'a> {
    pub(crate) transaction_id: &'a str,
    pub(crate) target_version: &'a str,
    pub(crate) target_channel: UpdateReleaseChannel,
    pub(crate) generation: u64,
    pub(crate) archive_sha256: &'a str,
    pub(crate) resource_pack_sha256: &'a str,
    pub(crate) launcher_sha256: &'a str,
    pub(crate) canonical_binary_sha256: &'a str,
    pub(crate) update_helper_sha256: &'a str,
    pub(crate) reconciliation_journal_sha256: &'a str,
}

#[cfg(target_os = "macos")]
fn advance_to_awaiting_exit(
    store: &UpdateStateStore,
    mut state: UpdateState,
    expected_revision: u64,
    transaction_root: &Path,
) -> Result<(u64, bool, UpdateState), &'static str> {
    if let Err(error) = replacement_checkpoint(ReplacementCheckpoint::BeforeAwaitingExit) {
        remove_replacement_contract(transaction_root);
        return Err(error);
    }
    state
        .active_transaction
        .as_mut()
        .ok_or("native-update-transaction-missing")?
        .phase = UpdateTransactionPhase::AwaitingExit;
    let outcome = match store.replace(expected_revision, state.clone()) {
        Ok(outcome) => outcome,
        Err(error) => {
            remove_replacement_contract(transaction_root);
            return Err(error.reason_code());
        }
    };
    if outcome.cleanup_required {
        return Err("native-update-state-cleanup-required");
    }
    if let Err(error) = replacement_checkpoint(ReplacementCheckpoint::AfterAwaitingExit) {
        state
            .active_transaction
            .as_mut()
            .ok_or("native-update-transaction-missing")?
            .phase = UpdateTransactionPhase::ReconciliationPrepared;
        if store.replace(outcome.revision, state).is_err() {
            return Err("native-update-recovery-required");
        }
        remove_replacement_contract(transaction_root);
        return Err(error);
    }
    Ok((outcome.revision, outcome.cleanup_required, state))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReplacementJournalV1 {
    document_kind: String,
    schema_version: u32,
    transaction_id: String,
    parent_process_id: u32,
    destination_application: PathBuf,
    staged_application: PathBuf,
    backup_application: PathBuf,
    target_version: String,
    target_channel: UpdateReleaseChannel,
    generation: u64,
    archive_sha256: String,
    resource_pack_sha256: String,
    launcher_sha256: String,
    canonical_binary_sha256: String,
    update_helper_sha256: String,
    reconciliation_journal_sha256: String,
    health_token_sha256: String,
    created_at_unix: u64,
}

pub(crate) fn prepare_replacement(
    store: &UpdateStateStore,
    state: UpdateState,
    expected_revision: u64,
    preparation: &ReplacementPreparation<'_>,
) -> Result<PreparedReplacement, &'static str> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (store, state, expected_revision, preparation);
        Err("native-update-target-unsupported")
    }
    #[cfg(target_os = "macos")]
    {
        validate_preparation(&state, preparation)?;
        let current_executable =
            env::current_exe().map_err(|_| "native-update-installation-layout-invalid")?;
        let destination_application = application_from_canonical_executable(&current_executable)?;
        validate_current_application(&destination_application, &current_executable)?;

        let transaction_root = store.staging_root().join(preparation.transaction_id);
        let staged_application = transaction_root.join("application").join(APPLICATION_NAME);
        let staged_canonical = staged_application
            .join("Contents")
            .join("MacOS")
            .join(CANONICAL_BINARY_NAME);
        let staged_helper = staged_application
            .join("Contents")
            .join("MacOS")
            .join(UPDATE_HELPER_NAME);
        validate_staged_executable(&staged_canonical, preparation.canonical_binary_sha256)?;
        validate_staged_executable(&staged_helper, preparation.update_helper_sha256)?;
        validate_replacement_filesystem(&transaction_root, &destination_application)?;
        let reconciliation = load_reconciliation_journal(store, preparation.transaction_id)?;
        verify_prepared_reconciliation(&reconciliation)?;
        if reconciliation_journal_sha256(&reconciliation)?
            != preparation.reconciliation_journal_sha256
        {
            return Err("native-update-reconciliation-invalid");
        }
        run_startup_check(&staged_canonical)?;

        let backup_application = destination_application
            .parent()
            .ok_or("native-update-installation-layout-invalid")?
            .join(format!(
                ".Qiongli.app.qiongli-backup-{}",
                preparation.transaction_id
            ));
        ensure_absent(&backup_application)?;
        ensure_absent(&transaction_root.join(FAILED_APPLICATION_DIRECTORY))?;

        let mut token_bytes = [0_u8; 32];
        getrandom::fill(&mut token_bytes).map_err(|_| "native-update-health-token-unavailable")?;
        let token = encode_lower_hex(&token_bytes);
        let journal = ReplacementJournalV1 {
            document_kind: JOURNAL_DOCUMENT_KIND.to_string(),
            schema_version: JOURNAL_SCHEMA_VERSION,
            transaction_id: preparation.transaction_id.to_string(),
            parent_process_id: std::process::id(),
            destination_application,
            staged_application,
            backup_application,
            target_version: preparation.target_version.to_string(),
            target_channel: preparation.target_channel,
            generation: preparation.generation,
            archive_sha256: preparation.archive_sha256.to_string(),
            resource_pack_sha256: preparation.resource_pack_sha256.to_string(),
            launcher_sha256: preparation.launcher_sha256.to_string(),
            canonical_binary_sha256: preparation.canonical_binary_sha256.to_string(),
            update_helper_sha256: preparation.update_helper_sha256.to_string(),
            reconciliation_journal_sha256: preparation.reconciliation_journal_sha256.to_string(),
            health_token_sha256: sha256_hex(token.as_bytes()),
            created_at_unix: now_unix()?,
        };
        validate_journal(&journal, store, preparation.transaction_id)?;
        let journal_path = transaction_root.join(JOURNAL_FILE);
        let token_path = transaction_root.join(HEALTH_TOKEN_FILE);
        write_new_private_file(
            &journal_path,
            &serde_json::to_vec_pretty(&journal).map_err(|_| "native-update-journal-invalid")?,
        )?;
        if let Err(error) = write_new_private_file(&token_path, token.as_bytes()) {
            let _ = fs::remove_file(&journal_path);
            return Err(error);
        }
        sync_directory(&transaction_root)?;

        let (revision, cleanup_required, mut awaiting_exit_state) =
            advance_to_awaiting_exit(store, state, expected_revision, &transaction_root)?;
        if let Err(error) = spawn_helper(&staged_helper, preparation.transaction_id) {
            awaiting_exit_state
                .active_transaction
                .as_mut()
                .ok_or("native-update-transaction-missing")?
                .phase = UpdateTransactionPhase::ReconciliationPrepared;
            if store.replace(revision, awaiting_exit_state).is_err() {
                return Err("native-update-recovery-required");
            }
            remove_replacement_contract(&transaction_root);
            return Err(error);
        }
        Ok(PreparedReplacement {
            revision,
            transaction_id: preparation.transaction_id.to_string(),
            target_version: preparation.target_version.to_string(),
            generation: preparation.generation,
            cleanup_required,
        })
    }
}

pub(crate) fn confirm_replacement_health(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<u64, &'static str> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (store, transaction_id);
        Err("native-update-target-unsupported")
    }
    #[cfg(target_os = "macos")]
    {
        validate_transaction_id(transaction_id)?;
        let journal = load_journal(store, transaction_id)?;
        let token = load_health_token(store, transaction_id)?;
        let supplied = env::var("QIONGLI_UPDATE_HEALTH_TOKEN")
            .map_err(|_| "native-update-health-token-missing")?;
        if !constant_time_equal(token.as_bytes(), supplied.as_bytes())
            || sha256_hex(supplied.as_bytes()) != journal.health_token_sha256
        {
            return Err("native-update-health-token-invalid");
        }
        let current_executable =
            env::current_exe().map_err(|_| "native-update-health-check-failed")?;
        if current_executable
            != journal
                .destination_application
                .join("Contents")
                .join("MacOS")
                .join(CANONICAL_BINARY_NAME)
            || sha256_file(&current_executable, 512 * 1024 * 1024)?
                != journal.canonical_binary_sha256
            || env!("CARGO_PKG_VERSION") != journal.target_version
            || crate::EMBEDDED_PACK_SHA256.trim() != journal.resource_pack_sha256
        {
            return Err("native-update-health-check-failed");
        }
        let reconciliation = load_reconciliation_journal(store, transaction_id)?;
        if reconciliation_journal_sha256(&reconciliation)? != journal.reconciliation_journal_sha256
        {
            return Err("native-update-health-check-failed");
        }
        verify_active_reconciliation(&reconciliation)?;
        commit_replacement_health(store, &journal)
    }
}

pub fn run_native_update_helper(
    arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), &'static str> {
    let arguments = arguments
        .into_iter()
        .map(|value| value.as_ref().to_owned())
        .collect::<Vec<OsString>>();
    let [transaction_id] = arguments.as_slice() else {
        return Err("native-update-helper-usage-invalid");
    };
    let transaction_id = transaction_id
        .to_str()
        .ok_or("native-update-helper-usage-invalid")?;
    validate_transaction_id(transaction_id)?;
    #[cfg(not(target_os = "macos"))]
    {
        let _ = transaction_id;
        Err("native-update-target-unsupported")
    }
    #[cfg(target_os = "macos")]
    {
        run_macos_helper(transaction_id)
    }
}

#[cfg(target_os = "macos")]
fn run_macos_helper(transaction_id: &str) -> Result<(), &'static str> {
    let store = update_store_from_process()?;
    let journal = load_journal(&store, transaction_id)?;
    let reconciliation = load_reconciliation_journal(&store, transaction_id)?;
    if reconciliation_journal_sha256(&reconciliation)? != journal.reconciliation_journal_sha256 {
        return Err("native-update-reconciliation-invalid");
    }
    validate_running_helper(&journal)?;
    let _lock = acquire_replacement_lock(&store)?;
    let handoff = (|| {
        let transaction_root = journal
            .staged_application
            .parent()
            .and_then(Path::parent)
            .ok_or("native-update-journal-invalid")?;
        validate_replacement_filesystem(transaction_root, &journal.destination_application)?;
        wait_for_parent_exit(journal.parent_process_id)?;
        validate_replacement_filesystem(transaction_root, &journal.destination_application)?;
        Ok(())
    })();
    if let Err(error) = handoff {
        restore_pre_activation_state(&store, &journal);
        return Err(error);
    }
    continue_replacement_after_handoff(&store, &journal, &reconciliation, || {
        run_health_process(&journal)
    })
}

#[cfg(target_os = "macos")]
fn continue_replacement_after_handoff(
    store: &UpdateStateStore,
    journal: &ReplacementJournalV1,
    reconciliation: &crate::update_reconcile::ReconciliationJournalV1,
    run_health: impl FnOnce() -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    if let Err(error) = transition_phase(
        store,
        &journal.transaction_id,
        UpdateTransactionPhase::AwaitingExit,
        UpdateTransactionPhase::Activating,
    ) {
        restore_pre_activation_state(store, journal);
        return Err(error);
    }
    if let Err(error) = activate_application(journal) {
        if error == "native-update-recovery-required" {
            mark_recovery_required(store, &journal.transaction_id);
            return Err(error);
        }
        restore_pre_activation_state(store, journal);
        return Err(error);
    }
    if let Err(error) = activate_prepared_reconciliation(reconciliation) {
        rollback_activated_application(store, journal)?;
        cleanup_rolled_back_reconciliation(reconciliation)
            .map_err(|_| "native-update-reconciliation-cleanup-required")?;
        return Err(error);
    }
    if transition_phase(
        store,
        &journal.transaction_id,
        UpdateTransactionPhase::Activating,
        UpdateTransactionPhase::HealthWindow,
    )
    .is_err()
    {
        rollback_active_reconciliation(reconciliation)
            .map_err(|_| "native-update-recovery-required")?;
        rollback_activated_application(store, journal)?;
        cleanup_rolled_back_reconciliation(reconciliation)
            .map_err(|_| "native-update-reconciliation-cleanup-required")?;
        return Err("native-update-recovery-required");
    }
    if let Err(error) = run_health() {
        if confirm_committed_state(store, journal).is_ok() {
            return cleanup_committed_replacement(store, journal, reconciliation);
        }
        rollback_active_reconciliation(reconciliation)
            .map_err(|_| "native-update-recovery-required")?;
        rollback_activated_application(store, journal)?;
        cleanup_rolled_back_reconciliation(reconciliation)
            .map_err(|_| "native-update-reconciliation-cleanup-required")?;
        return Err(health_failure_reason(error));
    }
    confirm_committed_state(store, journal)?;
    cleanup_committed_replacement(store, journal, reconciliation)
}

#[cfg(target_os = "macos")]
fn cleanup_committed_replacement(
    store: &UpdateStateStore,
    journal: &ReplacementJournalV1,
    reconciliation: &crate::update_reconcile::ReconciliationJournalV1,
) -> Result<(), &'static str> {
    cleanup_committed_reconciliation(reconciliation)?;
    fs::remove_dir_all(&journal.backup_application)
        .map_err(|_| "native-update-backup-cleanup-required")?;
    sync_directory(
        journal
            .backup_application
            .parent()
            .ok_or("native-update-installation-layout-invalid")?,
    )?;
    let transaction_root = store.staging_root().join(&journal.transaction_id);
    fs::remove_dir_all(&transaction_root).map_err(|_| "native-update-staging-cleanup-required")?;
    sync_directory(&store.staging_root())
}

#[cfg(target_os = "macos")]
fn activate_application(journal: &ReplacementJournalV1) -> Result<(), &'static str> {
    rename_without_replacement(
        &journal.destination_application,
        &journal.backup_application,
    )?;
    if let Err(error) = replacement_checkpoint(ReplacementCheckpoint::AfterOldApplicationBackup) {
        if rename_without_replacement(
            &journal.backup_application,
            &journal.destination_application,
        )
        .is_err()
        {
            return Err("native-update-recovery-required");
        }
        return Err(error);
    }
    if let Err(error) = rename_without_replacement(
        &journal.staged_application,
        &journal.destination_application,
    ) {
        let _ = rename_without_replacement(
            &journal.backup_application,
            &journal.destination_application,
        );
        return Err(error);
    }
    let activation_result = replacement_checkpoint(
        ReplacementCheckpoint::AfterNewApplicationActivation,
    )
    .and_then(|()| {
        sync_directory(
            journal
                .destination_application
                .parent()
                .ok_or("native-update-installation-layout-invalid")?,
        )
    });
    if let Err(error) = activation_result {
        if restore_pre_health_application_layout(journal).is_err() {
            return Err("native-update-recovery-required");
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn restore_pre_health_application_layout(
    journal: &ReplacementJournalV1,
) -> Result<(), &'static str> {
    rename_without_replacement(
        &journal.destination_application,
        &journal.staged_application,
    )?;
    rename_without_replacement(
        &journal.backup_application,
        &journal.destination_application,
    )?;
    sync_directory(
        journal
            .destination_application
            .parent()
            .ok_or("native-update-installation-layout-invalid")?,
    )
}

#[cfg(target_os = "macos")]
fn rollback_activated_application(
    store: &UpdateStateStore,
    journal: &ReplacementJournalV1,
) -> Result<(), &'static str> {
    let failed = store
        .staging_root()
        .join(&journal.transaction_id)
        .join(FAILED_APPLICATION_DIRECTORY);
    ensure_absent(&failed)?;
    rename_without_replacement(&journal.destination_application, &failed)?;
    if rename_without_replacement(
        &journal.backup_application,
        &journal.destination_application,
    )
    .is_err()
    {
        mark_recovery_required(store, &journal.transaction_id);
        return Err("native-update-recovery-required");
    }
    sync_directory(
        journal
            .destination_application
            .parent()
            .ok_or("native-update-installation-layout-invalid")?,
    )?;
    let _ = fs::remove_dir_all(&failed);
    clear_failed_transaction(store, &journal.transaction_id)?;
    let transaction_root = store.staging_root().join(&journal.transaction_id);
    let _ = fs::remove_dir_all(&transaction_root);
    let _ = sync_directory(&store.staging_root());
    Ok(())
}

#[cfg(target_os = "macos")]
fn restore_pre_activation_state(store: &UpdateStateStore, journal: &ReplacementJournalV1) {
    if journal.backup_application.exists() && !journal.destination_application.exists() {
        let _ = rename_without_replacement(
            &journal.backup_application,
            &journal.destination_application,
        );
    }
    let Ok(loaded) = store.load() else {
        return;
    };
    let mut state = loaded.state;
    if let Some(transaction) = state.active_transaction.as_mut()
        && transaction.transaction_id == journal.transaction_id
    {
        transaction.phase = UpdateTransactionPhase::ReconciliationPrepared;
        if store.replace(loaded.revision, state).is_ok()
            && let Some(transaction_root) =
                journal.staged_application.parent().and_then(Path::parent)
        {
            remove_replacement_contract(transaction_root);
        }
    }
}

#[cfg(target_os = "macos")]
#[allow(
    clippy::disallowed_methods,
    reason = "the update helper launches only the newly activated sibling Qiongli binary"
)]
fn run_health_process(journal: &ReplacementJournalV1) -> Result<(), &'static str> {
    let transaction_root = journal
        .staged_application
        .parent()
        .and_then(Path::parent)
        .ok_or("native-update-journal-invalid")?;
    let token = read_private_file(&transaction_root.join(HEALTH_TOKEN_FILE), MAX_TOKEN_BYTES)?;
    let token = String::from_utf8(token).map_err(|_| "native-update-health-token-invalid")?;
    let executable = journal
        .destination_application
        .join("Contents")
        .join("MacOS")
        .join(CANONICAL_BINARY_NAME);
    let mut command = Command::new(executable);
    command
        .arg("update")
        .arg("health")
        .arg("--transaction-id")
        .arg(&journal.transaction_id)
        .env_clear()
        .env("QIONGLI_UPDATE_HEALTH_TOKEN", token)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    copy_config_environment(&mut command)?;
    run_bounded_child(&mut command)
}

#[cfg(target_os = "macos")]
fn load_health_token(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<String, &'static str> {
    let bytes = read_private_file(
        &store
            .staging_root()
            .join(transaction_id)
            .join(HEALTH_TOKEN_FILE),
        MAX_TOKEN_BYTES,
    )?;
    let token = String::from_utf8(bytes).map_err(|_| "native-update-health-token-invalid")?;
    if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("native-update-health-token-invalid");
    }
    Ok(token)
}

#[cfg(target_os = "macos")]
fn transition_phase(
    store: &UpdateStateStore,
    transaction_id: &str,
    expected: UpdateTransactionPhase,
    replacement: UpdateTransactionPhase,
) -> Result<u64, &'static str> {
    let (before, after) = transition_checkpoints(expected, replacement)?;
    replacement_checkpoint(before)?;
    let loaded = store.load().map_err(|error| error.reason_code())?;
    let mut state = loaded.state;
    let transaction = state
        .active_transaction
        .as_mut()
        .ok_or("native-update-transaction-missing")?;
    if transaction.transaction_id != transaction_id || transaction.phase != expected {
        return Err("native-update-transaction-state-invalid");
    }
    transaction.phase = replacement;
    let outcome = store
        .replace(loaded.revision, state)
        .map_err(|error| error.reason_code())?;
    if outcome.cleanup_required {
        return Err("native-update-state-cleanup-required");
    }
    replacement_checkpoint(after)?;
    Ok(outcome.revision)
}

#[cfg(target_os = "macos")]
fn transition_checkpoints(
    expected: UpdateTransactionPhase,
    replacement: UpdateTransactionPhase,
) -> Result<(ReplacementCheckpoint, ReplacementCheckpoint), &'static str> {
    match (expected, replacement) {
        (UpdateTransactionPhase::AwaitingExit, UpdateTransactionPhase::Activating) => Ok((
            ReplacementCheckpoint::BeforeActivating,
            ReplacementCheckpoint::AfterActivating,
        )),
        (UpdateTransactionPhase::Activating, UpdateTransactionPhase::HealthWindow) => Ok((
            ReplacementCheckpoint::BeforeHealthWindow,
            ReplacementCheckpoint::AfterHealthWindow,
        )),
        _ => Err("native-update-transaction-state-invalid"),
    }
}

#[cfg(target_os = "macos")]
fn clear_failed_transaction(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<(), &'static str> {
    let loaded = store.load().map_err(|error| error.reason_code())?;
    let mut state = loaded.state;
    if state
        .active_transaction
        .as_ref()
        .is_some_and(|transaction| transaction.transaction_id == transaction_id)
    {
        state.active_transaction = None;
        let outcome = store
            .replace(loaded.revision, state)
            .map_err(|error| error.reason_code())?;
        if outcome.cleanup_required {
            return Err("native-update-state-cleanup-required");
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn mark_recovery_required(store: &UpdateStateStore, transaction_id: &str) {
    let Ok(loaded) = store.load() else {
        return;
    };
    let mut state = loaded.state;
    if let Some(transaction) = state.active_transaction.as_mut()
        && transaction.transaction_id == transaction_id
    {
        transaction.phase = UpdateTransactionPhase::RecoveryRequired;
        let _ = store.replace(loaded.revision, state);
    }
}

#[cfg(target_os = "macos")]
fn confirm_committed_state(
    store: &UpdateStateStore,
    journal: &ReplacementJournalV1,
) -> Result<(), &'static str> {
    let loaded = store.load().map_err(|error| error.reason_code())?;
    let last_known_good = loaded
        .state
        .last_known_good
        .ok_or("native-update-health-state-invalid")?;
    if loaded.state.active_transaction.is_some()
        || loaded.state.last_accepted_generation != journal.generation
        || last_known_good.version != journal.target_version
        || last_known_good.generation != journal.generation
        || last_known_good.archive_sha256 != journal.archive_sha256
        || last_known_good.resource_pack_sha256 != journal.resource_pack_sha256
    {
        return Err("native-update-health-state-invalid");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn commit_replacement_health(
    store: &UpdateStateStore,
    journal: &ReplacementJournalV1,
) -> Result<u64, &'static str> {
    replacement_checkpoint(ReplacementCheckpoint::BeforeHealthCommit)?;
    let loaded = store.load().map_err(|error| error.reason_code())?;
    let transaction = loaded
        .state
        .active_transaction
        .as_ref()
        .ok_or("native-update-transaction-missing")?;
    if transaction.transaction_id != journal.transaction_id
        || transaction.target_version != journal.target_version
        || transaction.phase != UpdateTransactionPhase::HealthWindow
    {
        return Err("native-update-health-state-invalid");
    }
    let mut state = loaded.state;
    state.last_accepted_generation = journal.generation;
    state.last_known_good = Some(UpdateLastKnownGood {
        version: journal.target_version.clone(),
        channel: journal.target_channel,
        generation: journal.generation,
        archive_sha256: journal.archive_sha256.clone(),
        resource_pack_sha256: journal.resource_pack_sha256.clone(),
    });
    state.active_transaction = None;
    let outcome = store
        .replace(loaded.revision, state)
        .map_err(|error| error.reason_code())?;
    if outcome.cleanup_required {
        return Err("native-update-state-cleanup-required");
    }
    replacement_checkpoint(ReplacementCheckpoint::AfterHealthCommit)?;
    Ok(outcome.revision)
}

#[cfg(target_os = "macos")]
fn validate_running_helper(journal: &ReplacementJournalV1) -> Result<(), &'static str> {
    let executable = env::current_exe().map_err(|_| "native-update-helper-invalid")?;
    if executable.file_name().and_then(OsStr::to_str) != Some(UPDATE_HELPER_NAME)
        || sha256_file(&executable, 128 * 1024 * 1024)? != journal.update_helper_sha256
    {
        return Err("native-update-helper-invalid");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn wait_for_parent_exit(parent_process_id: u32) -> Result<(), &'static str> {
    let raw = i32::try_from(parent_process_id).map_err(|_| "native-update-parent-invalid")?;
    let pid = rustix::process::Pid::from_raw(raw).ok_or("native-update-parent-invalid")?;
    let deadline = Instant::now() + EXIT_HANDOFF_TIMEOUT;
    loop {
        match rustix::process::test_kill_process(pid) {
            Ok(()) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(()) => return Err("native-update-parent-exit-timeout"),
            Err(rustix::io::Errno::SRCH) => return Ok(()),
            Err(_) => return Err("native-update-parent-inspection-failed"),
        }
    }
}

#[cfg(target_os = "macos")]
fn update_store_from_process() -> Result<UpdateStateStore, &'static str> {
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or("native-update-home-unavailable")?;
    let configured = env::var_os("QIONGLI_CONFIG_HOME").filter(|value| !value.is_empty());
    let root =
        resolve_config_root(configured.as_deref(), &home).map_err(|error| error.reason_code())?;
    Ok(UpdateStateStore::new(
        root,
        if env!("CARGO_PKG_VERSION").contains("-alpha.")
            || env!("CARGO_PKG_VERSION").contains("-beta.")
        {
            UpdateStreamPreference::Beta
        } else {
            UpdateStreamPreference::Stable
        },
    ))
}

fn validate_preparation(
    state: &UpdateState,
    preparation: &ReplacementPreparation<'_>,
) -> Result<(), &'static str> {
    validate_transaction_id(preparation.transaction_id)?;
    let transaction = state
        .active_transaction
        .as_ref()
        .ok_or("native-update-transaction-missing")?;
    if transaction.transaction_id != preparation.transaction_id
        || transaction.target_version != preparation.target_version
        || transaction.phase != UpdateTransactionPhase::ReconciliationPrepared
        || preparation.generation == 0
        || preparation.generation <= state.last_accepted_generation
        || !valid_version(preparation.target_version)
        || !valid_sha256(preparation.archive_sha256)
        || !valid_sha256(preparation.resource_pack_sha256)
        || !valid_sha256(preparation.launcher_sha256)
        || !valid_sha256(preparation.canonical_binary_sha256)
        || !valid_sha256(preparation.update_helper_sha256)
        || !valid_sha256(preparation.reconciliation_journal_sha256)
    {
        return Err("native-update-install-contract-invalid");
    }
    Ok(())
}

fn load_journal(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<ReplacementJournalV1, &'static str> {
    validate_transaction_id(transaction_id)?;
    validate_owned_private_directory(&store.staging_root().join(transaction_id))?;
    let bytes = read_private_file(
        &store.staging_root().join(transaction_id).join(JOURNAL_FILE),
        MAX_JOURNAL_BYTES,
    )?;
    let journal = serde_json::from_slice::<ReplacementJournalV1>(&bytes)
        .map_err(|_| "native-update-journal-invalid")?;
    validate_journal(&journal, store, transaction_id)?;
    Ok(journal)
}

fn validate_journal(
    journal: &ReplacementJournalV1,
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<(), &'static str> {
    let transaction_root = store.staging_root().join(transaction_id);
    let expected_staged = transaction_root.join("application").join(APPLICATION_NAME);
    let expected_backup = journal
        .destination_application
        .parent()
        .ok_or("native-update-journal-invalid")?
        .join(format!(".Qiongli.app.qiongli-backup-{transaction_id}"));
    if journal.document_kind != JOURNAL_DOCUMENT_KIND
        || journal.schema_version != JOURNAL_SCHEMA_VERSION
        || journal.transaction_id != transaction_id
        || journal.parent_process_id == 0
        || journal.staged_application != expected_staged
        || journal.backup_application != expected_backup
        || journal
            .destination_application
            .file_name()
            .and_then(OsStr::to_str)
            != Some(APPLICATION_NAME)
        || !journal.destination_application.is_absolute()
        || !journal.staged_application.is_absolute()
        || !journal.backup_application.is_absolute()
        || has_unsafe_components(&journal.destination_application)
        || has_unsafe_components(&journal.staged_application)
        || has_unsafe_components(&journal.backup_application)
        || !valid_version(&journal.target_version)
        || journal.generation == 0
        || !valid_sha256(&journal.archive_sha256)
        || !valid_sha256(&journal.resource_pack_sha256)
        || !valid_sha256(&journal.launcher_sha256)
        || !valid_sha256(&journal.canonical_binary_sha256)
        || !valid_sha256(&journal.update_helper_sha256)
        || !valid_sha256(&journal.reconciliation_journal_sha256)
        || !valid_sha256(&journal.health_token_sha256)
        || journal.created_at_unix == 0
    {
        return Err("native-update-journal-invalid");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn application_from_canonical_executable(executable: &Path) -> Result<PathBuf, &'static str> {
    if !executable.is_absolute()
        || executable.file_name().and_then(OsStr::to_str) != Some(CANONICAL_BINARY_NAME)
    {
        return Err("native-update-installation-layout-invalid");
    }
    let macos = executable
        .parent()
        .ok_or("native-update-installation-layout-invalid")?;
    let contents = macos
        .parent()
        .ok_or("native-update-installation-layout-invalid")?;
    let application = contents
        .parent()
        .ok_or("native-update-installation-layout-invalid")?;
    if macos.file_name().and_then(OsStr::to_str) != Some("MacOS")
        || contents.file_name().and_then(OsStr::to_str) != Some("Contents")
        || application.file_name().and_then(OsStr::to_str) != Some(APPLICATION_NAME)
    {
        return Err("native-update-installation-layout-invalid");
    }
    Ok(application.to_path_buf())
}

#[cfg(target_os = "macos")]
fn validate_current_application(application: &Path, executable: &Path) -> Result<(), &'static str> {
    validate_owned_directory(application)?;
    validate_owned_directory(&application.join("Contents"))?;
    validate_owned_directory(&application.join("Contents").join("MacOS"))?;
    let metadata = fs::symlink_metadata(executable)
        .map_err(|_| "native-update-installation-layout-invalid")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("native-update-installation-layout-invalid");
    }
    let parent = application
        .parent()
        .ok_or("native-update-installation-layout-invalid")?;
    validate_owned_directory(parent)?;
    validate_writable_parent(parent)
}

#[cfg(target_os = "macos")]
fn validate_staged_executable(path: &Path, expected_sha256: &str) -> Result<(), &'static str> {
    if sha256_file(path, 512 * 1024 * 1024)? != expected_sha256 {
        return Err("native-update-staged-application-drift");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn validate_replacement_filesystem(
    transaction_root: &Path,
    destination_application: &Path,
) -> Result<(), &'static str> {
    use std::os::unix::fs::MetadataExt;

    validate_owned_private_directory(transaction_root)?;
    let staged = transaction_root.join("application").join(APPLICATION_NAME);
    validate_owned_private_directory(
        staged
            .parent()
            .ok_or("native-update-staged-application-missing")?,
    )?;
    validate_owned_directory(&staged)?;
    let destination_parent = destination_application
        .parent()
        .ok_or("native-update-installation-layout-invalid")?;
    validate_owned_directory(destination_parent)?;
    validate_writable_parent(destination_parent)?;
    let staged_metadata =
        fs::symlink_metadata(&staged).map_err(|_| "native-update-staged-application-missing")?;
    let destination_metadata = fs::symlink_metadata(destination_parent)
        .map_err(|_| "native-update-installation-layout-invalid")?;
    if staged_metadata.dev() != destination_metadata.dev() {
        return Err("native-update-cross-device-install-unsupported");
    }
    let filesystem = rustix::fs::statvfs(destination_parent)
        .map_err(|_| "native-update-filesystem-inspection-failed")?;
    let available = filesystem.f_frsize.saturating_mul(filesystem.f_bavail);
    if available < MINIMUM_AVAILABLE_BYTES {
        return Err("native-update-insufficient-disk-space");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn validate_owned_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::MetadataExt;

    let metadata =
        fs::symlink_metadata(path).map_err(|_| "native-update-installation-layout-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
    {
        return Err("native-update-installation-location-unsafe");
    }
    Ok(())
}

fn validate_owned_private_directory(path: &Path) -> Result<(), &'static str> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let metadata =
            fs::symlink_metadata(path).map_err(|_| "native-update-staging-unavailable")?;
        if metadata.file_type().is_symlink()
            || !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o077 != 0
        {
            return Err("native-update-staging-unsafe");
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err("native-update-target-unsupported")
    }
}

#[cfg(target_os = "macos")]
fn validate_writable_parent(parent: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::OpenOptionsExt;

    let probe = parent.join(format!(
        ".qiongli-update-write-probe-{}",
        std::process::id()
    ));
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&probe)
        .map_err(|_| "native-update-installation-location-not-writable")?;
    file.sync_all()
        .map_err(|_| "native-update-installation-location-not-writable")?;
    fs::remove_file(&probe).map_err(|_| "native-update-installation-location-not-writable")?;
    sync_directory(parent)
}

#[cfg(target_os = "macos")]
#[allow(
    clippy::disallowed_methods,
    reason = "the updater preflights only the verified staged Qiongli binary"
)]
fn run_startup_check(executable: &Path) -> Result<(), &'static str> {
    let mut command = Command::new(executable);
    command
        .arg("ui")
        .arg("--startup-check")
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    copy_config_environment(&mut command)?;
    run_bounded_child(&mut command).map_err(|_| "native-update-startup-preflight-failed")
}

#[cfg(target_os = "macos")]
#[allow(
    clippy::disallowed_methods,
    reason = "the updater launches only the verified native helper bundled in the staged application"
)]
fn spawn_helper(helper: &Path, transaction_id: &str) -> Result<(), &'static str> {
    let mut command = Command::new(helper);
    command
        .arg(transaction_id)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    copy_config_environment(&mut command)?;
    command
        .spawn()
        .map(|_| ())
        .map_err(|_| "native-update-helper-launch-failed")
}

#[cfg(target_os = "macos")]
fn copy_config_environment(command: &mut Command) -> Result<(), &'static str> {
    let home = env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .ok_or("native-update-home-unavailable")?;
    command.env("HOME", home);
    if let Some(configured) = env::var_os("QIONGLI_CONFIG_HOME").filter(|value| !value.is_empty()) {
        command.env("QIONGLI_CONFIG_HOME", configured);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_bounded_child(command: &mut Command) -> Result<(), &'static str> {
    let mut child = command
        .spawn()
        .map_err(|_| "native-update-child-launch-failed")?;
    let deadline = Instant::now() + CHILD_TIMEOUT;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| "native-update-child-wait-failed")?
        {
            return status
                .success()
                .then_some(())
                .ok_or("native-update-child-failed");
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("native-update-child-timeout");
        }
        thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(target_os = "macos")]
fn rename_without_replacement(source: &Path, destination: &Path) -> Result<(), &'static str> {
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        source,
        rustix::fs::CWD,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|_| "native-update-atomic-replacement-failed")
}

fn read_private_file(path: &Path, maximum_size: u64) -> Result<Vec<u8>, &'static str> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let descriptor = rustix::fs::open(
            path,
            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW,
            rustix::fs::Mode::empty(),
        )
        .map_err(|_| "native-update-private-file-unavailable")?;
        let mut file = File::from(descriptor);
        let metadata = file
            .metadata()
            .map_err(|_| "native-update-private-file-unavailable")?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o077 != 0
            || metadata.len() == 0
            || metadata.len() > maximum_size
        {
            return Err("native-update-private-file-unsafe");
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut bytes)
            .map_err(|_| "native-update-private-file-unavailable")?;
        if bytes.len() as u64 != metadata.len() {
            return Err("native-update-private-file-drift");
        }
        Ok(bytes)
    }
    #[cfg(not(unix))]
    {
        let _ = (path, maximum_size);
        Err("native-update-target-unsupported")
    }
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .map_err(|_| "native-update-journal-write-failed")?;
        file.write_all(bytes)
            .and_then(|()| file.flush())
            .and_then(|()| file.sync_all())
            .map_err(|_| "native-update-journal-write-failed")
    }
    #[cfg(not(unix))]
    {
        let _ = (path, bytes);
        Err("native-update-target-unsupported")
    }
}

fn sync_directory(path: &Path) -> Result<(), &'static str> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| "native-update-directory-sync-failed")
}

fn sha256_file(path: &Path, maximum_size: u64) -> Result<String, &'static str> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "native-update-executable-unavailable")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum_size
    {
        return Err("native-update-executable-invalid");
    }
    let mut file = File::open(path).map_err(|_| "native-update-executable-unavailable")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut observed = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| "native-update-executable-unavailable")?;
        if read == 0 {
            break;
        }
        observed = observed
            .checked_add(read as u64)
            .ok_or("native-update-executable-invalid")?;
        if observed > maximum_size {
            return Err("native-update-executable-invalid");
        }
        hasher.update(&buffer[..read]);
    }
    if observed != metadata.len() {
        return Err("native-update-executable-drift");
    }
    Ok(encode_lower_hex(&hasher.finalize()))
}

fn ensure_absent(path: &Path) -> Result<(), &'static str> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err("native-update-replacement-collision"),
    }
}

fn remove_replacement_contract(transaction_root: &Path) {
    let _ = fs::remove_file(transaction_root.join(JOURNAL_FILE));
    let _ = fs::remove_file(transaction_root.join(HEALTH_TOKEN_FILE));
    let _ = sync_directory(transaction_root);
}

fn validate_transaction_id(value: &str) -> Result<(), &'static str> {
    if value.len() != 39
        || !value.starts_with("update-")
        || !value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("native-update-transaction-id-invalid");
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_version(value: &str) -> bool {
    semver::Version::parse(value).is_ok()
}

fn has_unsafe_components(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            Component::CurDir | Component::ParentDir | Component::Prefix(_)
        )
    })
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "native-update-clock-invalid")
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_lower_hex(&Sha256::digest(bytes))
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(HEX[usize::from(byte >> 4)] as char);
        output.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    output
}

fn replacement_checkpoint(checkpoint: ReplacementCheckpoint) -> Result<(), &'static str> {
    #[cfg(test)]
    {
        let interrupted = INJECTED_REPLACEMENT_CHECKPOINT.with(|injected| {
            if injected.get() == Some(checkpoint) {
                injected.set(None);
                true
            } else {
                false
            }
        });
        if interrupted {
            return Err(TEST_INTERRUPTION);
        }
    }
    #[cfg(not(test))]
    let _ = checkpoint;
    Ok(())
}

fn health_failure_reason(error: &'static str) -> &'static str {
    #[cfg(test)]
    if error == TEST_INTERRUPTION {
        return error;
    }
    let _ = error;
    "native-update-health-check-failed"
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    use crate::update_reconcile::empty_reconciliation_journal;
    #[cfg(target_os = "macos")]
    use qiongli_config::UpdateActiveTransaction;

    #[test]
    fn helper_accepts_only_one_well_formed_transaction_id() {
        assert_eq!(
            run_native_update_helper(std::iter::empty::<&str>()),
            Err("native-update-helper-usage-invalid")
        );
        assert_eq!(
            run_native_update_helper(["update-not-hex"]),
            Err("native-update-transaction-id-invalid")
        );
        assert_eq!(
            run_native_update_helper([
                "update-0123456789abcdef0123456789abcdef",
                "/tmp/untrusted-path",
            ]),
            Err("native-update-helper-usage-invalid")
        );
    }

    #[test]
    fn health_tokens_compare_without_early_exit() {
        assert!(constant_time_equal(b"0123", b"0123"));
        assert!(!constant_time_equal(b"0123", b"0124"));
        assert!(!constant_time_equal(b"0123", b"012"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn awaiting_exit_transition_interruptions_restore_a_retryable_stage() {
        for checkpoint in [
            ReplacementCheckpoint::BeforeAwaitingExit,
            ReplacementCheckpoint::AfterAwaitingExit,
        ] {
            let fixture = replacement_fixture(&format!("awaiting-exit-{checkpoint:?}"), true);
            let loaded = fixture.store.load().unwrap();
            let result = with_replacement_interruption(checkpoint, || {
                advance_to_awaiting_exit(
                    &fixture.store,
                    loaded.state,
                    loaded.revision,
                    &fixture.transaction_root,
                )
            });

            assert_eq!(result, Err(TEST_INTERRUPTION));
            assert_retryable_staged(&fixture);
            let _ = fs::remove_dir_all(&fixture.root);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn pre_health_interruptions_restore_the_complete_last_known_good_application() {
        for checkpoint in [
            ReplacementCheckpoint::BeforeActivating,
            ReplacementCheckpoint::AfterActivating,
            ReplacementCheckpoint::AfterOldApplicationBackup,
            ReplacementCheckpoint::AfterNewApplicationActivation,
        ] {
            let fixture = replacement_fixture(&format!("pre-health-{checkpoint:?}"), false);
            let result = with_replacement_interruption(checkpoint, || {
                continue_replacement_after_handoff(
                    &fixture.store,
                    &fixture.journal,
                    &fixture.reconciliation,
                    || commit_replacement_health(&fixture.store, &fixture.journal).map(|_| ()),
                )
            });

            assert_eq!(result, Err(TEST_INTERRUPTION));
            assert_retryable_staged(&fixture);
            let _ = fs::remove_dir_all(&fixture.root);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn rel_913_health_window_interruptions_roll_back_without_advancing_last_known_good() {
        for checkpoint in [
            ReplacementCheckpoint::BeforeHealthWindow,
            ReplacementCheckpoint::AfterHealthWindow,
            ReplacementCheckpoint::BeforeHealthCommit,
        ] {
            let fixture = replacement_fixture(&format!("health-window-{checkpoint:?}"), false);
            let result = with_replacement_interruption(checkpoint, || {
                continue_replacement_after_handoff(
                    &fixture.store,
                    &fixture.journal,
                    &fixture.reconciliation,
                    || commit_replacement_health(&fixture.store, &fixture.journal).map(|_| ()),
                )
            });

            assert_eq!(
                result,
                Err(if checkpoint == ReplacementCheckpoint::BeforeHealthCommit {
                    TEST_INTERRUPTION
                } else {
                    "native-update-recovery-required"
                })
            );
            assert_rolled_back(&fixture);
            let _ = fs::remove_dir_all(&fixture.root);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn rel_913_interruption_after_health_commit_keeps_the_new_known_good_application() {
        let fixture = replacement_fixture("after-health-commit", false);
        let result =
            with_replacement_interruption(ReplacementCheckpoint::AfterHealthCommit, || {
                continue_replacement_after_handoff(
                    &fixture.store,
                    &fixture.journal,
                    &fixture.reconciliation,
                    || commit_replacement_health(&fixture.store, &fixture.journal).map(|_| ()),
                )
            });

        assert_eq!(result, Ok(()));
        assert_committed(&fixture);
        let _ = fs::remove_dir_all(&fixture.root);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn journal_rejects_path_substitution() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("target/qiongli-native-replacement-journal-tests")
            .join(format!("{}-{}", std::process::id(), now_unix().unwrap()));
        let config = resolve_config_root(Some(root.as_os_str()), &root).unwrap();
        let store = UpdateStateStore::new(config, UpdateStreamPreference::Beta);
        let transaction_id = "update-0123456789abcdef0123456789abcdef";
        let destination = root.join("Applications").join(APPLICATION_NAME);
        let mut journal = ReplacementJournalV1 {
            document_kind: JOURNAL_DOCUMENT_KIND.to_string(),
            schema_version: JOURNAL_SCHEMA_VERSION,
            transaction_id: transaction_id.to_string(),
            parent_process_id: 1,
            destination_application: destination.clone(),
            staged_application: store
                .staging_root()
                .join(transaction_id)
                .join("application")
                .join(APPLICATION_NAME),
            backup_application: destination
                .parent()
                .unwrap()
                .join(format!(".Qiongli.app.qiongli-backup-{transaction_id}")),
            target_version: "2.0.0-alpha.2".to_string(),
            target_channel: UpdateReleaseChannel::Alpha,
            generation: 2,
            archive_sha256: "1".repeat(64),
            resource_pack_sha256: "2".repeat(64),
            launcher_sha256: "3".repeat(64),
            canonical_binary_sha256: "4".repeat(64),
            update_helper_sha256: "5".repeat(64),
            reconciliation_journal_sha256: "7".repeat(64),
            health_token_sha256: "6".repeat(64),
            created_at_unix: 1,
        };
        assert!(validate_journal(&journal, &store, transaction_id).is_ok());
        journal.staged_application = root.join("attacker-controlled.app");
        assert_eq!(
            validate_journal(&journal, &store, transaction_id),
            Err("native-update-journal-invalid")
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn activation_restores_the_old_application_when_the_staged_rename_fails() {
        let root = test_root("activation-rollback");
        let destination = root.join(APPLICATION_NAME);
        let staged = root.join("missing-staged").join(APPLICATION_NAME);
        let backup = root.join(".Qiongli.app.qiongli-backup-fixture");
        create_directory_with_file(&destination, b"old");
        let journal = journal_fixture(destination.clone(), staged, backup.clone());

        assert_eq!(
            activate_application(&journal),
            Err("native-update-atomic-replacement-failed")
        );
        assert_eq!(fs::read(destination.join("payload")).unwrap(), b"old");
        assert!(!backup.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn failed_health_restores_last_known_good_and_clears_the_transaction() {
        let root = test_root("health-rollback");
        let config_root = root.join("config");
        let config = resolve_config_root(Some(config_root.as_os_str()), Path::new("/tmp")).unwrap();
        let store = UpdateStateStore::new(config, UpdateStreamPreference::Beta);
        let transaction_id = "update-0123456789abcdef0123456789abcdef";
        let mut state = UpdateState::initial(UpdateStreamPreference::Beta);
        state.active_transaction = Some(UpdateActiveTransaction {
            transaction_id: transaction_id.to_string(),
            target_version: "2.0.0-alpha.2".to_string(),
            phase: UpdateTransactionPhase::HealthWindow,
        });
        store.replace(0, state).unwrap();
        let transaction_root = store.staging_root().join(transaction_id);
        create_private_tree(&transaction_root);
        let applications = root.join("Applications");
        create_private_tree(&applications);
        let destination = applications.join(APPLICATION_NAME);
        let backup = applications.join(format!(".Qiongli.app.qiongli-backup-{transaction_id}"));
        create_directory_with_file(&destination, b"failed-new");
        create_directory_with_file(&backup, b"known-good");
        let journal = journal_fixture(
            destination.clone(),
            transaction_root.join("application").join(APPLICATION_NAME),
            backup,
        );

        rollback_activated_application(&store, &journal).unwrap();

        assert_eq!(
            fs::read(destination.join("payload")).unwrap(),
            b"known-good"
        );
        assert!(store.load().unwrap().state.active_transaction.is_none());
        assert!(!transaction_root.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn pre_activation_failure_returns_to_staged_and_removes_contract_files() {
        let root = test_root("pre-activation-recovery");
        let config_root = root.join("config");
        let config = resolve_config_root(Some(config_root.as_os_str()), Path::new("/tmp")).unwrap();
        let store = UpdateStateStore::new(config, UpdateStreamPreference::Beta);
        let transaction_id = "update-0123456789abcdef0123456789abcdef";
        let mut state = UpdateState::initial(UpdateStreamPreference::Beta);
        state.active_transaction = Some(UpdateActiveTransaction {
            transaction_id: transaction_id.to_string(),
            target_version: "2.0.0-alpha.2".to_string(),
            phase: UpdateTransactionPhase::AwaitingExit,
        });
        store.replace(0, state).unwrap();
        let transaction_root = store.staging_root().join(transaction_id);
        create_private_tree(&transaction_root.join("application"));
        write_new_private_file(&transaction_root.join(JOURNAL_FILE), b"journal").unwrap();
        write_new_private_file(&transaction_root.join(HEALTH_TOKEN_FILE), b"token").unwrap();
        let applications = root.join("Applications");
        create_private_tree(&applications);
        let destination = applications.join(APPLICATION_NAME);
        create_directory_with_file(&destination, b"known-good");
        let journal = journal_fixture(
            destination,
            transaction_root.join("application").join(APPLICATION_NAME),
            applications.join(format!(".Qiongli.app.qiongli-backup-{transaction_id}")),
        );

        restore_pre_activation_state(&store, &journal);

        assert_eq!(
            store
                .load()
                .unwrap()
                .state
                .active_transaction
                .unwrap()
                .phase,
            UpdateTransactionPhase::ReconciliationPrepared
        );
        assert!(!transaction_root.join(JOURNAL_FILE).exists());
        assert!(!transaction_root.join(HEALTH_TOKEN_FILE).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(target_os = "macos")]
    fn journal_fixture(
        destination_application: PathBuf,
        staged_application: PathBuf,
        backup_application: PathBuf,
    ) -> ReplacementJournalV1 {
        ReplacementJournalV1 {
            document_kind: JOURNAL_DOCUMENT_KIND.to_string(),
            schema_version: JOURNAL_SCHEMA_VERSION,
            transaction_id: "update-0123456789abcdef0123456789abcdef".to_string(),
            parent_process_id: 1,
            destination_application,
            staged_application,
            backup_application,
            target_version: "2.0.0-alpha.2".to_string(),
            target_channel: UpdateReleaseChannel::Alpha,
            generation: 2,
            archive_sha256: "1".repeat(64),
            resource_pack_sha256: "2".repeat(64),
            launcher_sha256: "3".repeat(64),
            canonical_binary_sha256: "4".repeat(64),
            update_helper_sha256: "5".repeat(64),
            reconciliation_journal_sha256: reconciliation_journal_sha256(
                &empty_reconciliation_journal(
                    "update-0123456789abcdef0123456789abcdef",
                    "2.0.0-alpha.2",
                    &"2".repeat(64),
                ),
            )
            .unwrap(),
            health_token_sha256: "6".repeat(64),
            created_at_unix: 1,
        }
    }

    #[cfg(target_os = "macos")]
    struct ReplacementFixture {
        root: PathBuf,
        store: UpdateStateStore,
        transaction_root: PathBuf,
        journal: ReplacementJournalV1,
        reconciliation: crate::update_reconcile::ReconciliationJournalV1,
    }

    #[cfg(target_os = "macos")]
    fn replacement_fixture(name: &str, staged_phase: bool) -> ReplacementFixture {
        let root = test_root(name);
        let config_root = root.join("config");
        let config = resolve_config_root(Some(config_root.as_os_str()), &root).unwrap();
        let store = UpdateStateStore::new(config, UpdateStreamPreference::Beta);
        let transaction_id = "update-0123456789abcdef0123456789abcdef";
        let mut state = UpdateState::initial(UpdateStreamPreference::Beta);
        state.last_accepted_generation = 1;
        state.last_known_good = Some(UpdateLastKnownGood {
            version: "2.0.0-alpha.1".to_string(),
            channel: UpdateReleaseChannel::Alpha,
            generation: 1,
            archive_sha256: "a".repeat(64),
            resource_pack_sha256: "b".repeat(64),
        });
        state.active_transaction = Some(UpdateActiveTransaction {
            transaction_id: transaction_id.to_string(),
            target_version: "2.0.0-alpha.2".to_string(),
            phase: if staged_phase {
                UpdateTransactionPhase::ReconciliationPrepared
            } else {
                UpdateTransactionPhase::AwaitingExit
            },
        });
        store.replace(0, state).unwrap();

        let transaction_root = store.staging_root().join(transaction_id);
        let staged = transaction_root.join("application").join(APPLICATION_NAME);
        create_directory_with_file(&staged, b"new-known-good");
        write_new_private_file(&transaction_root.join(JOURNAL_FILE), b"journal").unwrap();
        write_new_private_file(&transaction_root.join(HEALTH_TOKEN_FILE), b"token").unwrap();

        let applications = root.join("Applications");
        create_private_tree(&applications);
        let destination = applications.join(APPLICATION_NAME);
        create_directory_with_file(&destination, b"old-known-good");
        let backup = applications.join(format!(".Qiongli.app.qiongli-backup-{transaction_id}"));
        let journal = journal_fixture(destination, staged, backup);
        let reconciliation =
            empty_reconciliation_journal(transaction_id, "2.0.0-alpha.2", &"2".repeat(64));
        ReplacementFixture {
            root,
            store,
            transaction_root,
            journal,
            reconciliation,
        }
    }

    #[cfg(target_os = "macos")]
    fn with_replacement_interruption<T>(
        checkpoint: ReplacementCheckpoint,
        operation: impl FnOnce() -> T,
    ) -> T {
        INJECTED_REPLACEMENT_CHECKPOINT.with(|injected| {
            assert!(injected.replace(Some(checkpoint)).is_none());
        });
        let result = operation();
        INJECTED_REPLACEMENT_CHECKPOINT.with(|injected| {
            assert!(injected.replace(None).is_none());
        });
        result
    }

    #[cfg(target_os = "macos")]
    fn assert_retryable_staged(fixture: &ReplacementFixture) {
        let loaded = fixture.store.load().unwrap();
        assert_eq!(
            loaded.state.active_transaction.unwrap().phase,
            UpdateTransactionPhase::ReconciliationPrepared
        );
        assert_eq!(
            fs::read(fixture.journal.destination_application.join("payload")).unwrap(),
            b"old-known-good"
        );
        assert_eq!(
            fs::read(fixture.journal.staged_application.join("payload")).unwrap(),
            b"new-known-good"
        );
        assert!(!fixture.journal.backup_application.exists());
        assert!(!fixture.transaction_root.join(JOURNAL_FILE).exists());
        assert!(!fixture.transaction_root.join(HEALTH_TOKEN_FILE).exists());
    }

    #[cfg(target_os = "macos")]
    fn assert_rolled_back(fixture: &ReplacementFixture) {
        let loaded = fixture.store.load().unwrap();
        assert!(loaded.state.active_transaction.is_none());
        assert_eq!(loaded.state.last_accepted_generation, 1);
        assert_eq!(loaded.state.last_known_good.unwrap().generation, 1);
        assert_eq!(
            fs::read(fixture.journal.destination_application.join("payload")).unwrap(),
            b"old-known-good"
        );
        assert!(!fixture.journal.backup_application.exists());
        assert!(!fixture.transaction_root.exists());
    }

    #[cfg(target_os = "macos")]
    fn assert_committed(fixture: &ReplacementFixture) {
        let loaded = fixture.store.load().unwrap();
        assert!(loaded.state.active_transaction.is_none());
        assert_eq!(loaded.state.last_accepted_generation, 2);
        let last_known_good = loaded.state.last_known_good.unwrap();
        assert_eq!(last_known_good.version, "2.0.0-alpha.2");
        assert_eq!(last_known_good.generation, 2);
        assert_eq!(
            fs::read(fixture.journal.destination_application.join("payload")).unwrap(),
            b"new-known-good"
        );
        assert!(!fixture.journal.backup_application.exists());
        assert!(!fixture.transaction_root.exists());
    }

    #[cfg(target_os = "macos")]
    fn test_root(name: &str) -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("target/qiongli-native-replacement-tests")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        create_private_tree(&root);
        root
    }

    #[cfg(target_os = "macos")]
    fn create_private_tree(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        fs::create_dir_all(path).unwrap();
        for ancestor in path.ancestors() {
            if ancestor.ends_with("target") {
                break;
            }
            if ancestor.exists() {
                fs::set_permissions(ancestor, fs::Permissions::from_mode(0o700)).unwrap();
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn create_directory_with_file(path: &Path, bytes: &[u8]) {
        create_private_tree(path);
        fs::write(path.join("payload"), bytes).unwrap();
    }
}
