#![cfg_attr(not(target_os = "macos"), allow(dead_code, unused_imports))]

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use qiongli_config::UpdateStateStore;
use qiongli_content::{
    EmbeddedContent, ProfileId, WorkflowOverrides, approve_materialization_target,
    verify_materialization,
};
use qiongli_platform::{
    CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION, CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
    CapabilityProfile, ClaudePluginBundleReceiptV1, ClaudeRegistrationReceiptV1,
    ClaudeRegistrationState, ClaudeRegistrationStateV1, ClaudeSkillsPluginState, ClaudeSourceState,
    ClientActivationTarget, CodexPluginBundleReceiptV1, CodexRegistrationReceiptV1,
    CodexRegistrationState, CodexRegistrationStateV1, CodexSourceState, HostAction, InstallScope,
    LocalSurface, LocalTargetFamily, OwnershipMarkerV1, ProductId, TargetDescriptorV1,
    VerifiedLaunchGrant, approve_claude_plugin_bundle_target, approve_codex_plugin_bundle_target,
    compose_claude_plugin_bundle_with_overrides, compose_codex_plugin_bundle_with_overrides,
    discover_claude_user_with_config, discover_codex_user, verify_claude_plugin_bundle,
    verify_codex_plugin_bundle,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::managed_content::{
    load_managed_content_registry, managed_content_registry_bytes, managed_content_registry_path,
    materialization_receipt_sha256, parse_managed_content_registry,
};

pub(crate) const RECONCILIATION_JOURNAL_FILE: &str = "reconciliation-journal.json";
const JOURNAL_DOCUMENT_KIND: &str = "qiongli-update-reconciliation";
const JOURNAL_SCHEMA_VERSION: u32 = 1;
const CLI_JOURNAL_SCHEMA_VERSION: u32 = 2;
const RELEASE_JOURNAL_SCHEMA_VERSION: u32 = 3;
const MAX_JOURNAL_BYTES: u64 = 1024 * 1024;
const MAX_OPERATIONS: usize = 136;
const MAX_STATE_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReconciliationSurface {
    Skills,
    ManagedContentRegistry,
    CodexPluginBundle,
    CodexRegistration,
    ClaudePluginBundle,
    ClaudeSkillsPluginBundle,
    ClaudeRegistration,
    CliBinary,
    CliReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReconciliationOperationV1 {
    pub(crate) operation_id: String,
    pub(crate) surface: ReconciliationSurface,
    pub(crate) destination: PathBuf,
    pub(crate) staged: PathBuf,
    pub(crate) backup: PathBuf,
    pub(crate) staging_container: PathBuf,
    pub(crate) old_product_version: String,
    pub(crate) new_product_version: String,
    pub(crate) old_pack_sha256: String,
    pub(crate) new_pack_sha256: String,
    pub(crate) old_receipt_sha256: String,
    pub(crate) new_receipt_sha256: String,
    pub(crate) old_content_sha256: String,
    pub(crate) new_content_sha256: String,
    pub(crate) plan_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReconciliationJournalV1 {
    document_kind: String,
    schema_version: u32,
    pub(crate) transaction_id: String,
    pub(crate) target_version: String,
    pub(crate) target_pack_sha256: String,
    pub(crate) operations: Vec<ReconciliationOperationV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_release: Option<NativeActivationReleaseV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeActivationReleaseV1 {
    candidate_digest_sha256: String,
    previous_update_revision: u64,
    previous_last_accepted_generation: u64,
    previous_last_known_good: Option<qiongli_config::UpdateLastKnownGood>,
    next: qiongli_config::UpdateLastKnownGood,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedReconciliation {
    pub(crate) operation_count: usize,
    pub(crate) journal_sha256: String,
}

pub(crate) struct ReconciliationPreparation<'a> {
    pub(crate) store: &'a UpdateStateStore,
    pub(crate) transaction_id: &'a str,
    pub(crate) target_version: &'a str,
    pub(crate) content: &'a EmbeddedContent,
    pub(crate) platform_home: &'a Path,
    pub(crate) claude_config_root: &'a Path,
    pub(crate) source_binary: &'a Path,
    pub(crate) codex_grant: &'a VerifiedLaunchGrant,
    pub(crate) claude_grant: &'a VerifiedLaunchGrant,
    pub(crate) workflow_overrides: Option<&'a WorkflowOverrides>,
    pub(crate) targets: &'a [ClientActivationTarget],
    pub(crate) now_unix: u64,
    // The old pack digest must come from the verified predecessor product.
    pub(crate) cli_update: Option<(&'a crate::cli_install::CliInstallPlan, &'a str)>,
    pub(crate) native_release: Option<(
        &'a qiongli_platform::VerifiedNativeReleaseCandidate,
        &'a qiongli_config::LoadedUpdateState,
    )>,
}

#[cfg(test)]
pub(crate) fn empty_reconciliation_journal(
    transaction_id: &str,
    target_version: &str,
    target_pack_sha256: &str,
) -> ReconciliationJournalV1 {
    ReconciliationJournalV1 {
        native_release: None,
        document_kind: JOURNAL_DOCUMENT_KIND.to_string(),
        schema_version: JOURNAL_SCHEMA_VERSION,
        transaction_id: transaction_id.to_string(),
        target_version: target_version.to_string(),
        target_pack_sha256: target_pack_sha256.to_string(),
        operations: Vec::new(),
    }
}

pub(crate) fn prepare_update_reconciliation(
    preparation: &ReconciliationPreparation<'_>,
) -> Result<PreparedReconciliation, &'static str> {
    validate_transaction_id(preparation.transaction_id)?;
    validate_v2_version(preparation.target_version)?;
    if preparation.content.pack().pack_sha256()
        != preparation.codex_grant.grant().resource_pack_sha256
        || preparation.content.pack().pack_sha256()
            != preparation.claude_grant.grant().resource_pack_sha256
        || preparation.codex_grant.grant().artifact.version != preparation.target_version
        || preparation.claude_grant.grant().artifact.version != preparation.target_version
        || preparation.codex_grant.authorized_scope()
            != ClientActivationTarget::Codex.integration_scope()
        || preparation.claude_grant.authorized_scope()
            != ClientActivationTarget::ClaudeCode.integration_scope()
        || (preparation.targets.is_empty() && preparation.cli_update.is_none())
        || preparation.targets.len() > 2
        || preparation
            .targets
            .iter()
            .enumerate()
            .any(|(index, target)| preparation.targets[..index].contains(target))
    {
        return Err("native-update-reconciliation-identity-mismatch");
    }
    let native_release = preparation
        .native_release
        .map(|(candidate, loaded)| {
            let (cli, _) = preparation
                .cli_update
                .ok_or("native-activation-release-invalid")?;
            let release = &candidate.candidate().signed_portable_release.envelope;
            if candidate.candidate().artifact.version != preparation.target_version
                || release.resource_pack_sha256 != preparation.content.pack().pack_sha256()
                || release.binary_sha256 != cli.source_sha256
                || loaded.state.active_transaction.is_some()
                || candidate.candidate().generation <= loaded.state.last_accepted_generation
            {
                return Err("native-activation-release-invalid");
            }
            Ok(NativeActivationReleaseV1 {
                candidate_digest_sha256: candidate.signed_payload_sha256().to_string(),
                previous_update_revision: loaded.revision,
                previous_last_accepted_generation: loaded.state.last_accepted_generation,
                previous_last_known_good: loaded.state.last_known_good.clone(),
                next: qiongli_config::UpdateLastKnownGood {
                    version: candidate.candidate().artifact.version.clone(),
                    channel: match candidate.candidate().artifact.channel {
                        qiongli_platform::ReleaseChannel::Alpha => {
                            qiongli_config::UpdateReleaseChannel::Alpha
                        }
                        qiongli_platform::ReleaseChannel::Beta => {
                            qiongli_config::UpdateReleaseChannel::Beta
                        }
                        qiongli_platform::ReleaseChannel::Stable => {
                            qiongli_config::UpdateReleaseChannel::Stable
                        }
                    },
                    generation: candidate.candidate().generation,
                    archive_sha256: release.archive_sha256.clone(),
                    resource_pack_sha256: release.resource_pack_sha256.clone(),
                },
            })
        })
        .transpose()?;
    let transaction_root = preparation
        .store
        .staging_root()
        .join(preparation.transaction_id);
    let journal_path = transaction_root.join(RECONCILIATION_JOURNAL_FILE);
    if journal_path.exists() {
        let journal = load_reconciliation_journal(preparation.store, preparation.transaction_id)?;
        if journal.target_version != preparation.target_version
            || journal.target_pack_sha256 != preparation.content.pack().pack_sha256()
            || (journal.schema_version >= CLI_JOURNAL_SCHEMA_VERSION)
                != preparation.cli_update.is_some()
            || journal.native_release != native_release
        {
            return Err("native-update-reconciliation-identity-mismatch");
        }
        if let Some((plan, old_pack)) = preparation.cli_update {
            let binary = journal
                .operations
                .iter()
                .find(|op| op.surface == ReconciliationSurface::CliBinary)
                .ok_or("native-update-reconciliation-identity-mismatch")?;
            if binary.destination != plan.target
                || binary.old_pack_sha256 != old_pack
                || binary.new_content_sha256 != plan.source_sha256
            {
                return Err("native-update-reconciliation-identity-mismatch");
            }
        }
        verify_prepared_reconciliation(&journal)?;
        return Ok(PreparedReconciliation {
            operation_count: journal.operations.len(),
            journal_sha256: reconciliation_journal_sha256(&journal)?,
        });
    }

    let mut operations = Vec::new();
    let result = (|| {
        prepare_registered_skills(preparation, &mut operations)?;
        if preparation.targets.contains(&ClientActivationTarget::Codex) {
            prepare_codex(preparation, &mut operations)?;
        }
        if preparation
            .targets
            .contains(&ClientActivationTarget::ClaudeCode)
        {
            prepare_claude(preparation, &mut operations)?;
        }
        if let Some((plan, old_pack)) = preparation.cli_update {
            prepare_cli_update_operations(
                plan,
                preparation.transaction_id,
                old_pack,
                preparation.content.pack().pack_sha256(),
                &mut operations,
            )?;
        }
        let journal = ReconciliationJournalV1 {
            native_release,
            document_kind: JOURNAL_DOCUMENT_KIND.to_string(),
            schema_version: if preparation.native_release.is_some() {
                RELEASE_JOURNAL_SCHEMA_VERSION
            } else if preparation.cli_update.is_some() {
                CLI_JOURNAL_SCHEMA_VERSION
            } else {
                JOURNAL_SCHEMA_VERSION
            },
            transaction_id: preparation.transaction_id.to_string(),
            target_version: preparation.target_version.to_string(),
            target_pack_sha256: preparation.content.pack().pack_sha256().to_string(),
            operations: operations.clone(),
        };
        validate_journal(&journal)?;
        verify_prepared_reconciliation(&journal)?;
        let bytes = canonical_json(&journal)?;
        write_new_private_file(&journal_path, &bytes)?;
        sync_directory(&transaction_root)?;
        Ok(PreparedReconciliation {
            operation_count: journal.operations.len(),
            journal_sha256: sha256_hex(&bytes),
        })
    })();
    if result.is_err() {
        cleanup_staging_operations(&operations);
    }
    result
}

#[cfg(unix)]
pub(crate) fn acquire_replacement_lock(store: &UpdateStateStore) -> Result<File, &'static str> {
    let updates_root = store
        .staging_root()
        .parent()
        .ok_or("native-update-staging-unavailable")?
        .to_path_buf();
    acquire_private_write_lock(&updates_root.join(".replacement.lock"))
}

#[cfg(unix)]
fn acquire_private_write_lock(lock_path: &Path) -> Result<File, &'static str> {
    use std::fs::TryLockError;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    if let Ok(metadata) = fs::symlink_metadata(lock_path)
        && (metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o077 != 0)
    {
        return Err("native-update-replacement-lock-unsafe");
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(lock_path)
        .map_err(|_| "native-update-replacement-lock-unavailable")?;
    let opened = lock
        .metadata()
        .map_err(|_| "native-update-replacement-lock-unavailable")?;
    let linked = fs::symlink_metadata(lock_path)
        .map_err(|_| "native-update-replacement-lock-unavailable")?;
    if opened.uid() != rustix::process::geteuid().as_raw()
        || opened.mode() & 0o077 != 0
        || opened.dev() != linked.dev()
        || opened.ino() != linked.ino()
    {
        return Err("native-update-replacement-lock-unsafe");
    }
    match lock.try_lock() {
        Ok(()) => Ok(lock),
        Err(TryLockError::WouldBlock) => Err("native-update-replacement-active"),
        Err(TryLockError::Error(_)) => Err("native-update-replacement-lock-unavailable"),
    }
}

#[cfg(not(unix))]
pub(crate) fn acquire_replacement_lock(_store: &UpdateStateStore) -> Result<File, &'static str> {
    Err("native-update-target-unsupported")
}

const HOME_ACTIVATION_MARKER: &str = "active-installation.json";

fn native_home_state_root(home: &Path) -> Result<PathBuf, &'static str> {
    qiongli_platform::prepare_native_candidate_managed_root(home)
        .map_err(|error| error.reason_code())?;
    Ok(home.join(".qiongli/native"))
}

#[cfg(unix)]
pub(crate) fn acquire_native_home_write_lock(home: &Path) -> Result<File, &'static str> {
    acquire_private_write_lock(&native_home_state_root(home)?.join(".installation.lock"))
}
#[cfg(not(unix))]
pub(crate) fn acquire_native_home_write_lock(_home: &Path) -> Result<File, &'static str> {
    Err("native-update-target-unsupported")
}

pub(crate) fn refuse_native_home_activation(home: &Path) -> Result<(), &'static str> {
    ensure_absent(&home.join(".qiongli/native").join(HOME_ACTIVATION_MARKER))
        .map_err(|_| "native-activation-recovery-required")
}

// ponytail: serialize installation writes per Home; split only for proven disjoint targets.
/// Unix native activation is excluded across all config roots sharing one Home.
/// Other platforms retain existing behavior until native activation is supported.
pub(crate) fn acquire_managed_write_guard(
    home: &Path,
    config: qiongli_config::ConfigRoot,
) -> Result<Option<(File, File)>, &'static str> {
    #[cfg(unix)]
    {
        let store =
            UpdateStateStore::new(config.clone(), qiongli_config::UpdateStreamPreference::Beta);
        if store
            .load()
            .map_err(|error| error.reason_code())?
            .state
            .active_transaction
            .is_some()
        {
            return Err("native-update-transaction-active");
        }
        let guard = acquire_update_write_guard(home, &store)?;
        if store
            .load()
            .map_err(|error| error.reason_code())?
            .state
            .active_transaction
            .is_some()
        {
            return Err("native-update-transaction-active");
        }
        Ok(guard)
    }
    #[cfg(not(unix))]
    {
        let _ = (home, config);
        Ok(None)
    }
}

/// Update stages may continue their own active transaction under these locks.
pub(crate) fn acquire_update_write_guard(
    home: &Path,
    store: &UpdateStateStore,
) -> Result<Option<(File, File)>, &'static str> {
    #[cfg(unix)]
    {
        store.load().map_err(|error| error.reason_code())?;
        let home_lock = acquire_native_home_write_lock(home)?;
        refuse_native_home_activation(home)?;
        let root = store
            .state_root()
            .parent()
            .ok_or("native-update-staging-unavailable")?;
        let config = qiongli_config::resolve_config_root(Some(root.as_os_str()), home)
            .map_err(|error| error.reason_code())?;
        qiongli_config::GlobalSettingsStore::new(config)
            .prepare_store()
            .map_err(|error| error.reason_code())?;
        ensure_private_directory(&store.state_root().join("updates"))?;
        let update_lock = acquire_replacement_lock(store)?;
        Ok(Some((home_lock, update_lock)))
    }
    #[cfg(not(unix))]
    {
        let _ = (home, store);
        Ok(None)
    }
}

fn native_journal_home(journal: &ReconciliationJournalV1) -> Result<PathBuf, &'static str> {
    let binary = journal
        .operations
        .iter()
        .find(|operation| operation.surface == ReconciliationSurface::CliBinary)
        .ok_or("native-activation-journal-mismatch")?;
    let home = binary
        .destination
        .ancestors()
        .nth(3)
        .ok_or("native-activation-journal-mismatch")?;
    if crate::cli_install::cli_target(home) != binary.destination {
        return Err("native-activation-journal-mismatch");
    }
    Ok(home.to_path_buf())
}

fn bind_home_activation(
    store: &UpdateStateStore,
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    let home = native_journal_home(journal)?;
    let start = read_private_file(
        &store
            .staging_root()
            .join(&journal.transaction_id)
            .join(NATIVE_ACTIVATION_RECORD),
        MAX_STATE_BYTES,
    )?;
    bind_installation_marker(&home, &start)
}

/// Call only while holding the Home installation lock.
pub(crate) fn bind_installation_marker(home: &Path, start: &[u8]) -> Result<(), &'static str> {
    let marker = native_home_state_root(home)?.join(HOME_ACTIVATION_MARKER);
    if marker
        .try_exists()
        .map_err(|_| "native-activation-record-invalid")?
    {
        if read_private_file(&marker, MAX_STATE_BYTES)? != start {
            return Err("native-activation-recovery-required");
        }
    } else {
        ensure_absent(&marker)?;
        write_new_private_file(&marker, start)?;
        sync_directory(marker.parent().ok_or("native-activation-record-invalid")?)?;
    }
    Ok(())
}

fn clear_home_activation(
    store: &UpdateStateStore,
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    bind_home_activation(store, journal)?;
    let home = native_journal_home(journal)?;
    let start = read_private_file(
        &store
            .staging_root()
            .join(&journal.transaction_id)
            .join(NATIVE_ACTIVATION_RECORD),
        MAX_STATE_BYTES,
    )?;
    clear_installation_marker(&home, &start)
}

/// Read the bounded private marker without accepting an arbitrary source path.
pub(crate) fn read_installation_marker(home: &Path) -> Result<Vec<u8>, &'static str> {
    qiongli_platform::discover_native_candidate_managed_root(home)
        .map_err(|error| error.reason_code())?;
    let native_root = home.join(".qiongli/native");
    verify_existing_private_directory(&native_root)?;
    read_private_file(&native_root.join(HOME_ACTIVATION_MARKER), MAX_STATE_BYTES)
}

/// Remove only the exact marker whose operation completed under the Home lock.
pub(crate) fn clear_installation_marker(home: &Path, expected: &[u8]) -> Result<(), &'static str> {
    let marker = native_home_state_root(home)?.join(HOME_ACTIVATION_MARKER);
    if read_private_file(&marker, MAX_STATE_BYTES)? != expected {
        return Err("native-activation-recovery-required");
    }
    fs::remove_file(&marker).map_err(|_| "native-activation-record-invalid")?;
    sync_directory(marker.parent().ok_or("native-activation-record-invalid")?)
}

const NATIVE_ACTIVATION_RECORD: &str = "native-activation.json";
const NATIVE_ACTIVATION_OUTCOME: &str = "native-activation-outcome.json";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeActivationOutcome {
    Committed,
    RolledBack,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeActivationRecord {
    schema_version: u32,
    transaction_id: String,
    journal_sha256: String,
    outcome: Option<NativeActivationOutcome>,
}

// Native process inspection currently has a macOS implementation only.
fn refuse_running_native_cli(journal: &ReconciliationJournalV1) -> Result<(), &'static str> {
    #[cfg(target_os = "macos")]
    {
        let paths: Vec<&Path> = journal
            .operations
            .iter()
            .filter(|operation| operation.surface == ReconciliationSurface::CliBinary)
            .flat_map(|operation| {
                [
                    operation.destination.as_path(),
                    operation.staged.as_path(),
                    operation.backup.as_path(),
                ]
            })
            .collect();
        crate::native_update_replace::refuse_running_installation_paths(&paths)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = journal;
        Ok(())
    }
}

/// Executes an already approved v2/v3 reconciliation journal. The caller owns
/// candidate trust, human approval, process pinning and the health check.
pub fn activate_native_reconciliation(
    store: &UpdateStateStore,
    transaction_id: &str,
    expected_journal_sha256: &str,
    health: impl FnOnce() -> Result<(), &'static str>,
) -> Result<NativeActivationOutcome, &'static str> {
    let initial_journal = checked_native_journal(store, transaction_id, expected_journal_sha256)?;
    store.load().map_err(|error| error.reason_code())?;
    let home = native_journal_home(&initial_journal)?;
    let _home_lock = acquire_native_home_write_lock(&home)?;
    let _lock = acquire_replacement_lock(store)?;
    let journal = checked_native_journal(store, transaction_id, expected_journal_sha256)?;
    if refuse_other_activation(store, &journal)? {
        return Err("native-activation-recovery-required");
    }
    refuse_native_home_activation(&home)?;
    if native_record_exists(store, transaction_id, NATIVE_ACTIVATION_RECORD)? {
        return Err("native-activation-recovery-required");
    }
    let initial = store.load().map_err(|error| error.reason_code())?;
    if let Some(release) = &journal.native_release
        && (initial.revision != release.previous_update_revision
            || !matches_pre_activation_release(&initial.state, release))
    {
        return Err("native-activation-state-changed");
    }
    verify_prepared_reconciliation(&journal)?;
    refuse_running_native_cli(&journal)?;
    write_native_activation_record(store, &journal, expected_journal_sha256, None)?;
    bind_home_activation(store, &journal)?;
    set_native_activation_phase(
        store,
        &journal,
        Some(qiongli_config::UpdateTransactionPhase::Activating),
        Some(initial.revision),
    )?;
    let attempt = activate_prepared_reconciliation(&journal).and_then(|()| health());
    let outcome = if attempt.is_ok() {
        verify_active_reconciliation(&journal)?;
        NativeActivationOutcome::Committed
    } else {
        rollback_active_reconciliation(&journal)?;
        NativeActivationOutcome::RolledBack
    };
    write_native_activation_record(store, &journal, expected_journal_sha256, Some(outcome))?;
    finish_native_activation(store, &journal, outcome)?;
    clear_home_activation(store, &journal)?;
    attempt.map(|()| outcome)
}

/// Recovers the same approved journal without re-running health or activation.
/// A durable committed outcome authorizes cleanup only; otherwise restore old state.
pub fn recover_native_reconciliation(
    store: &UpdateStateStore,
    transaction_id: &str,
    expected_journal_sha256: &str,
) -> Result<NativeActivationOutcome, &'static str> {
    let initial_journal = checked_native_journal(store, transaction_id, expected_journal_sha256)?;
    store.load().map_err(|error| error.reason_code())?;
    let home = native_journal_home(&initial_journal)?;
    let _home_lock = acquire_native_home_write_lock(&home)?;
    let _lock = acquire_replacement_lock(store)?;
    let journal = checked_native_journal(store, transaction_id, expected_journal_sha256)?;
    refuse_other_activation(store, &journal)?;
    read_native_activation_record(
        store,
        &journal,
        expected_journal_sha256,
        NATIVE_ACTIVATION_RECORD,
    )?;
    refuse_running_native_cli(&journal)?;
    bind_home_activation(store, &journal)?;
    let outcome = if native_record_exists(store, transaction_id, NATIVE_ACTIVATION_OUTCOME)? {
        read_native_activation_record(
            store,
            &journal,
            expected_journal_sha256,
            NATIVE_ACTIVATION_OUTCOME,
        )?
        .ok_or("native-activation-record-invalid")?
    } else {
        rollback_active_reconciliation(&journal)?;
        write_native_activation_record(
            store,
            &journal,
            expected_journal_sha256,
            Some(NativeActivationOutcome::RolledBack),
        )?;
        NativeActivationOutcome::RolledBack
    };
    finish_native_activation(store, &journal, outcome)?;
    clear_home_activation(store, &journal)?;
    Ok(outcome)
}

/// Discards only an unactivated native preparation after exact-journal filesystem
/// approval. Started activations must use recovery instead. Missing journals refuse.
pub fn discard_native_reconciliation(
    store: &UpdateStateStore,
    transaction_id: &str,
    expected_journal_sha256: &str,
) -> Result<(), &'static str> {
    let initial_journal = checked_native_journal(store, transaction_id, expected_journal_sha256)?;
    store.load().map_err(|error| error.reason_code())?;
    let home = native_journal_home(&initial_journal)?;
    let _home_lock = acquire_native_home_write_lock(&home)?;
    let _lock = acquire_replacement_lock(store)?;
    refuse_native_home_activation(&home)?;
    let journal = checked_native_journal(store, transaction_id, expected_journal_sha256)?;
    if store
        .load()
        .map_err(|error| error.reason_code())?
        .state
        .active_transaction
        .is_some()
        || native_record_exists(store, transaction_id, NATIVE_ACTIVATION_RECORD)?
        || native_record_exists(store, transaction_id, NATIVE_ACTIVATION_OUTCOME)?
    {
        return Err("native-activation-recovery-required");
    }
    let root = store.staging_root().join(transaction_id);
    for entry in fs::read_dir(&root).map_err(|_| "native-update-reconciliation-cleanup-required")? {
        if entry
            .map_err(|_| "native-update-reconciliation-cleanup-required")?
            .file_name()
            != RECONCILIATION_JOURNAL_FILE
        {
            return Err("native-update-reconciliation-cleanup-required");
        }
    }
    for operation in &journal.operations {
        ensure_absent(&operation.backup)?;
    }
    // The shared owner verifies every old destination and every remaining staged
    // file before deletion, including retries after interrupted staged cleanup.
    cleanup_rolled_back_reconciliation(&journal)?;
    checked_native_journal(store, transaction_id, expected_journal_sha256)?;
    remove_committed_reconciliation_journal(store, transaction_id)?;
    remove_reconciliation_transaction_root(store, transaction_id)
}

fn checked_native_journal(
    store: &UpdateStateStore,
    transaction_id: &str,
    expected: &str,
) -> Result<ReconciliationJournalV1, &'static str> {
    let journal = load_reconciliation_journal(store, transaction_id)?;
    if !matches!(
        journal.schema_version,
        CLI_JOURNAL_SCHEMA_VERSION | RELEASE_JOURNAL_SCHEMA_VERSION
    ) || !valid_sha256(expected)
        || reconciliation_journal_sha256(&journal)? != expected
    {
        return Err("native-activation-journal-mismatch");
    }
    Ok(journal)
}

fn refuse_other_activation(
    store: &UpdateStateStore,
    journal: &ReconciliationJournalV1,
) -> Result<bool, &'static str> {
    let loaded = store.load().map_err(|error| error.reason_code())?;
    if loaded
        .state
        .active_transaction
        .as_ref()
        .is_some_and(|active| {
            active.transaction_id != journal.transaction_id
                || active.target_version != journal.target_version
                || active.phase != qiongli_config::UpdateTransactionPhase::Activating
        })
    {
        return Err("native-update-transaction-active");
    }
    Ok(loaded.state.active_transaction.is_some())
}

fn set_native_activation_phase(
    store: &UpdateStateStore,
    journal: &ReconciliationJournalV1,
    phase: Option<qiongli_config::UpdateTransactionPhase>,
    expected_revision: Option<u64>,
) -> Result<(), &'static str> {
    let loaded = store.load().map_err(|error| error.reason_code())?;
    if expected_revision.is_some_and(|revision| revision != loaded.revision) {
        return Err("native-activation-state-changed");
    }
    if loaded
        .state
        .active_transaction
        .as_ref()
        .is_some_and(|active| {
            active.transaction_id != journal.transaction_id
                || active.target_version != journal.target_version
                || active.phase != qiongli_config::UpdateTransactionPhase::Activating
        })
    {
        return Err("native-update-transaction-active");
    }
    if phase.is_none() && loaded.state.active_transaction.is_none() {
        return Ok(());
    }
    let mut state = loaded.state;
    state.active_transaction = phase.map(|phase| qiongli_config::UpdateActiveTransaction {
        transaction_id: journal.transaction_id.clone(),
        target_version: journal.target_version.clone(),
        phase,
    });
    let result = store
        .replace(loaded.revision, state)
        .map_err(|error| error.reason_code())?;
    if result.cleanup_required {
        return Err("native-update-state-cleanup-required");
    }
    Ok(())
}

fn finish_native_activation(
    store: &UpdateStateStore,
    journal: &ReconciliationJournalV1,
    outcome: NativeActivationOutcome,
) -> Result<(), &'static str> {
    let loaded = store.load().map_err(|error| error.reason_code())?;
    refuse_other_activation(store, journal)?;
    let mut next_state = loaded.state.clone();
    if let Some(release) = &journal.native_release {
        let already_committed = outcome == NativeActivationOutcome::Committed
            && loaded.state.active_transaction.is_none()
            && loaded.state.last_accepted_generation == release.next.generation
            && loaded.state.last_known_good.as_ref() == Some(&release.next);
        if !already_committed
            && (!matches_pre_activation_release(&loaded.state, release)
                || (outcome == NativeActivationOutcome::Committed
                    && loaded.state.active_transaction.is_none()))
        {
            return Err("native-activation-state-changed");
        }
        if outcome == NativeActivationOutcome::Committed {
            next_state.last_accepted_generation = release.next.generation;
            next_state.last_known_good = Some(release.next.clone());
        }
        next_state.active_transaction = None;
    }
    match outcome {
        NativeActivationOutcome::Committed => cleanup_committed_reconciliation(journal)?,
        NativeActivationOutcome::RolledBack => cleanup_rolled_back_reconciliation(journal)?,
    }
    // Keep the immutable journal/outcome for replay. The release metadata and
    // reservation are committed together only after the durable outcome and cleanup.
    if journal.native_release.is_none() {
        return set_native_activation_phase(store, journal, None, None);
    }
    if next_state == loaded.state {
        return Ok(());
    }
    let commit = store
        .replace(loaded.revision, next_state)
        .map_err(|error| error.reason_code())?;
    if commit.cleanup_required {
        return Err("native-update-state-cleanup-required");
    }
    Ok(())
}

fn native_record_exists(
    store: &UpdateStateStore,
    transaction_id: &str,
    file: &str,
) -> Result<bool, &'static str> {
    let path = store.staging_root().join(transaction_id).join(file);
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        _ => Err("native-activation-record-invalid"),
    }
}

fn write_native_activation_record(
    store: &UpdateStateStore,
    journal: &ReconciliationJournalV1,
    digest: &str,
    outcome: Option<NativeActivationOutcome>,
) -> Result<(), &'static str> {
    let root = store.staging_root().join(&journal.transaction_id);
    let record = NativeActivationRecord {
        schema_version: 1,
        transaction_id: journal.transaction_id.clone(),
        journal_sha256: digest.into(),
        outcome,
    };
    write_new_private_file(
        &root.join(if outcome.is_some() {
            NATIVE_ACTIVATION_OUTCOME
        } else {
            NATIVE_ACTIVATION_RECORD
        }),
        &canonical_json(&record)?,
    )?;
    sync_directory(&root)
}

fn read_native_activation_record(
    store: &UpdateStateStore,
    journal: &ReconciliationJournalV1,
    digest: &str,
    file: &str,
) -> Result<Option<NativeActivationOutcome>, &'static str> {
    let path = store
        .staging_root()
        .join(&journal.transaction_id)
        .join(file);
    let bytes = read_private_file(&path, MAX_STATE_BYTES)?;
    let record: NativeActivationRecord =
        serde_json::from_slice(&bytes).map_err(|_| "native-activation-record-invalid")?;
    if record.schema_version != 1
        || record.transaction_id != journal.transaction_id
        || record.journal_sha256 != digest
        || canonical_json(&record)? != bytes
        || (file == NATIVE_ACTIVATION_RECORD) != record.outcome.is_none()
    {
        return Err("native-activation-record-invalid");
    }
    Ok(record.outcome)
}

pub(crate) fn prepare_reconciliation_transaction_root(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<(), &'static str> {
    validate_transaction_id(transaction_id)?;
    ensure_private_directory(store.state_root())?;
    ensure_private_directory(
        store
            .staging_root()
            .parent()
            .ok_or("native-update-reconciliation-invalid")?,
    )?;
    ensure_private_directory(&store.staging_root())?;
    let transaction_root = store.staging_root().join(transaction_id);
    if transaction_root.exists() {
        return Err("native-update-reconciliation-invalid");
    }
    create_private_directory(&transaction_root)
}

pub(crate) fn remove_reconciliation_transaction_root(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<(), &'static str> {
    validate_transaction_id(transaction_id)?;
    let transaction_root = store.staging_root().join(transaction_id);
    if transaction_root.exists() {
        let mut entries = fs::read_dir(&transaction_root)
            .map_err(|_| "native-update-reconciliation-cleanup-required")?;
        if entries.next().is_some() {
            return Err("native-update-reconciliation-cleanup-required");
        }
        fs::remove_dir(&transaction_root)
            .map_err(|_| "native-update-reconciliation-cleanup-required")?;
        sync_directory(&store.staging_root())?;
    }
    Ok(())
}

pub(crate) fn remove_committed_reconciliation_journal(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<(), &'static str> {
    validate_transaction_id(transaction_id)?;
    let path = store
        .staging_root()
        .join(transaction_id)
        .join(RECONCILIATION_JOURNAL_FILE);
    fs::remove_file(&path).map_err(|_| "native-update-reconciliation-cleanup-required")?;
    sync_directory(
        path.parent()
            .ok_or("native-update-reconciliation-invalid")?,
    )
}

pub(crate) fn load_reconciliation_journal(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<ReconciliationJournalV1, &'static str> {
    validate_transaction_id(transaction_id)?;
    let path = store
        .staging_root()
        .join(transaction_id)
        .join(RECONCILIATION_JOURNAL_FILE);
    for directory in [
        store.state_root().to_path_buf(),
        store
            .staging_root()
            .parent()
            .ok_or("native-update-reconciliation-invalid")?
            .to_path_buf(),
        store.staging_root(),
        store.staging_root().join(transaction_id),
    ] {
        verify_existing_private_directory(&directory)?;
    }
    let bytes = read_private_file(&path, MAX_JOURNAL_BYTES)?;
    let journal: ReconciliationJournalV1 =
        serde_json::from_slice(&bytes).map_err(|_| "native-update-reconciliation-invalid")?;
    validate_journal(&journal)?;
    if journal.transaction_id != transaction_id || canonical_json(&journal)? != bytes {
        return Err("native-update-reconciliation-invalid");
    }
    Ok(journal)
}

pub(crate) fn reconciliation_journal_sha256(
    journal: &ReconciliationJournalV1,
) -> Result<String, &'static str> {
    Ok(sha256_hex(&canonical_json(journal)?))
}

pub(crate) fn verify_prepared_reconciliation(
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    validate_journal(journal)?;
    for operation in &journal.operations {
        ensure_absent(&operation.backup)?;
        verify_operation_identity(operation, &operation.destination, false)?;
        verify_operation_identity(operation, &operation.staged, true)?;
    }
    Ok(())
}

pub(crate) fn verify_active_reconciliation(
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    validate_journal(journal)?;
    for operation in &journal.operations {
        ensure_absent(&operation.staged)?;
        verify_operation_identity(operation, &operation.destination, true)?;
        verify_operation_identity(operation, &operation.backup, false)?;
    }
    Ok(())
}

pub(crate) fn activate_prepared_reconciliation(
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    verify_prepared_reconciliation(journal)?;
    let mut applied = 0_usize;
    for operation in &journal.operations {
        if let Err(error) = rename_without_replacement(&operation.destination, &operation.backup) {
            rollback_applied_operations(&journal.operations[..applied])?;
            return Err(error);
        }
        if let Err(error) = rename_without_replacement(&operation.staged, &operation.destination) {
            let _ = rename_without_replacement(&operation.backup, &operation.destination);
            rollback_applied_operations(&journal.operations[..applied])?;
            return Err(error);
        }
        sync_operation_parent(operation)?;
        applied = applied.saturating_add(1);
    }
    verify_active_reconciliation(journal)
        .map_err(|_| "native-update-reconciliation-recovery-required")
}

pub(crate) fn rollback_active_reconciliation(
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    validate_journal(journal)?;
    rollback_applied_operations(&journal.operations)?;
    for operation in &journal.operations {
        verify_operation_identity(operation, &operation.destination, false)?;
        ensure_absent(&operation.backup)?;
    }
    Ok(())
}

pub(crate) fn cleanup_committed_reconciliation(
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    cleanup_reconciliation(journal, true)
}

pub(crate) fn cleanup_rolled_back_reconciliation(
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    cleanup_reconciliation(journal, false)
}

fn cleanup_reconciliation(
    journal: &ReconciliationJournalV1,
    committed: bool,
) -> Result<(), &'static str> {
    validate_journal(journal)?;
    // Check the complete retained state and every remaining cleanup target before
    // deletion. Missing targets are valid after an interrupted cleanup.
    for operation in &journal.operations {
        verify_operation_identity(operation, &operation.destination, committed)?;
        let (remaining, absent) = if committed {
            (&operation.backup, &operation.staged)
        } else {
            (&operation.staged, &operation.backup)
        };
        ensure_absent(absent)?;
        if remaining.exists() {
            verify_operation_identity(operation, remaining, !committed)?;
        } else {
            ensure_absent(remaining)?;
        }
        match fs::symlink_metadata(&operation.staging_container) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                for entry in fs::read_dir(&operation.staging_container)
                    .map_err(|_| "native-update-reconciliation-cleanup-required")?
                {
                    let entry =
                        entry.map_err(|_| "native-update-reconciliation-cleanup-required")?;
                    if committed || entry.path() != operation.staged {
                        return Err("native-update-reconciliation-cleanup-required");
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err("native-update-reconciliation-cleanup-required"),
        }
    }
    for operation in &journal.operations {
        let remaining = if committed {
            &operation.backup
        } else {
            &operation.staged
        };
        if remaining.exists() {
            remove_path(remaining)?;
        } else {
            ensure_absent(remaining)?;
        }
        match fs::remove_dir(&operation.staging_container) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("native-update-reconciliation-cleanup-required"),
        }
        sync_operation_parent(operation)?;
    }
    Ok(())
}

pub(crate) fn discard_prepared_reconciliation(
    store: &UpdateStateStore,
    transaction_id: &str,
) -> Result<(), &'static str> {
    let path = store
        .staging_root()
        .join(transaction_id)
        .join(RECONCILIATION_JOURNAL_FILE);
    if !path.exists() {
        return ensure_absent(&path);
    }
    let journal = load_reconciliation_journal(store, transaction_id)?;
    cleanup_rolled_back_reconciliation(&journal)?;
    fs::remove_file(&path).map_err(|_| "native-update-reconciliation-cleanup-required")?;
    sync_directory(
        path.parent()
            .ok_or("native-update-reconciliation-invalid")?,
    )
}

fn prepare_cli_update_operations(
    plan: &crate::cli_install::CliInstallPlan,
    transaction_id: &str,
    old_pack: &str,
    new_pack: &str,
    operations: &mut Vec<ReconciliationOperationV1>,
) -> Result<(), &'static str> {
    if !valid_sha256(old_pack) || !valid_sha256(new_pack) {
        return Err("native-update-reconciliation-identity-mismatch");
    }
    let old_bytes = read_private_file(&plan.receipt_path, MAX_STATE_BYTES)?;
    let old =
        crate::cli_install::read_receipt(&plan.receipt_path)?.ok_or("qiongli-cli-not-managed")?;
    let binary = prepare_directory_paths(transaction_id, "cli-binary", &plan.target, "qiongli")?;
    let mut containers = vec![binary.staging_container.clone()];
    let result = (|| {
        let receipt = prepare_file_paths(transaction_id, "cli-receipt", &plan.receipt_path)?;
        containers.push(receipt.staging_container.clone());
        crate::cli_install::stage_cli_update(plan, &binary.staged, &receipt.staged)?;
        let new_bytes = read_private_file(&receipt.staged, MAX_STATE_BYTES)?;
        let new =
            crate::cli_install::read_receipt(&receipt.staged)?.ok_or("qiongli-cli-not-managed")?;
        let old_digest = sha256_hex(&old_bytes);
        let new_digest = sha256_hex(&new_bytes);
        let mut pair = Vec::new();
        for (id, surface, paths) in [
            ("cli-binary", ReconciliationSurface::CliBinary, binary),
            ("cli-receipt", ReconciliationSurface::CliReceipt, receipt),
        ] {
            pair.push(directory_operation(
                id.to_string(),
                surface,
                paths,
                &old.product_version,
                &plan.product_version,
                old_pack,
                new_pack,
                &old_digest,
                &new_digest,
                &old.installed_sha256,
                &new.installed_sha256,
            )?);
        }
        for operation in &pair {
            verify_operation_identity(operation, &operation.destination, false)?;
            verify_operation_identity(operation, &operation.staged, true)?;
        }
        operations.extend(pair);
        Ok(())
    })();
    if result.is_err() {
        for container in containers {
            let _ = remove_empty_or_staged_container(&container);
        }
    }
    result
}

fn prepare_registered_skills(
    preparation: &ReconciliationPreparation<'_>,
    operations: &mut Vec<ReconciliationOperationV1>,
) -> Result<(), &'static str> {
    let mut registry = load_managed_content_registry(preparation.store.state_root())?;
    if registry.entries.is_empty() {
        return Ok(());
    }
    let old_version = registry.entries[0].product_version.clone();
    let old_pack = registry.entries[0].pack_sha256.clone();
    if registry
        .entries
        .iter()
        .any(|entry| entry.product_version != old_version || entry.pack_sha256 != old_pack)
    {
        return Err("native-update-reconciliation-ambiguous-inventory");
    }
    let old_registry_bytes = managed_content_registry_bytes(&registry)?;
    for index in 0..registry.entries.len() {
        let entry = registry.entries[index].clone();
        let destination = PathBuf::from(&entry.target);
        let target = approve_materialization_target(&destination)
            .map_err(|_| "native-update-reconciliation-target-invalid")?;
        let old = verify_materialization(&target)
            .map_err(|_| "native-update-reconciliation-receipt-drift")?;
        if old.profile != entry.profile
            || old.pack_sha256 != entry.pack_sha256
            || old.content_root_sha256 != entry.content_root_sha256
            || materialization_receipt_sha256(&old)? != entry.receipt_sha256
        {
            return Err("native-update-reconciliation-receipt-drift");
        }
        validate_v2_version(&entry.product_version)?;
        let operation_id = format!("skills-{index:03}");
        let paths = prepare_directory_paths(
            preparation.transaction_id,
            &operation_id,
            &destination,
            "content",
        )?;
        let staged_target = approve_materialization_target(&paths.staged)
            .map_err(|_| "native-update-reconciliation-target-invalid")?;
        let new = preparation
            .content
            .materialize_profile_with_overrides(
                profile_name(entry.profile),
                &staged_target,
                preparation.workflow_overrides,
            )
            .map_err(|_| "native-update-reconciliation-prepare-failed")?;
        operations.push(directory_operation(
            operation_id,
            ReconciliationSurface::Skills,
            paths,
            &entry.product_version,
            preparation.target_version,
            &old.pack_sha256,
            &new.pack_sha256,
            &materialization_receipt_sha256(&old)?,
            &materialization_receipt_sha256(&new)?,
            &old.content_root_sha256,
            &new.content_root_sha256,
        )?);
        registry.entries[index].product_version = preparation.target_version.to_string();
        registry.entries[index].receipt_sha256 = materialization_receipt_sha256(&new)?;
        registry.entries[index].pack_sha256 = new.pack_sha256;
        registry.entries[index].content_root_sha256 = new.content_root_sha256;
    }
    registry.generation = registry
        .generation
        .checked_add(1)
        .ok_or("native-update-reconciliation-invalid")?;
    registry.validate()?;
    let new_registry_bytes = managed_content_registry_bytes(&registry)?;
    let old_registry_sha256 = sha256_hex(&old_registry_bytes);
    let new_registry_sha256 = sha256_hex(&new_registry_bytes);
    operations.push(prepare_state_operation(
        preparation.transaction_id,
        "skills-registry",
        ReconciliationSurface::ManagedContentRegistry,
        managed_content_registry_path(preparation.store.state_root()),
        &old_registry_bytes,
        &new_registry_bytes,
        &old_version,
        preparation.target_version,
        &old_pack,
        preparation.content.pack().pack_sha256(),
        &old_registry_sha256,
        &new_registry_sha256,
    )?);
    Ok(())
}

fn prepare_codex(
    preparation: &ReconciliationPreparation<'_>,
    operations: &mut Vec<ReconciliationOperationV1>,
) -> Result<(), &'static str> {
    let discovered = discover_codex_user(preparation.platform_home)
        .map_err(|_| "native-update-codex-inventory-invalid")?;
    if matches!(
        discovered.summary().registration,
        CodexRegistrationState::Conflict
            | CodexRegistrationState::Drifted
            | CodexRegistrationState::RecoveryRequired
    ) {
        return Err("native-update-codex-registration-blocked");
    }
    if discovered.summary().source == CodexSourceState::Missing {
        return Ok(());
    }
    let destination = preparation
        .platform_home
        .join(".qiongli/plugins/codex/qiongli-next");
    let old_target = approve_codex_plugin_bundle_target(&destination)
        .map_err(|_| "native-update-codex-inventory-invalid")?;
    let old = verify_codex_plugin_bundle(&old_target)
        .map_err(|_| "native-update-reconciliation-receipt-drift")?;
    validate_v2_version(&old.receipt().artifact.version)?;
    let paths = prepare_directory_paths(
        preparation.transaction_id,
        "codex-01-bundle",
        &destination,
        "qiongli-next",
    )?;
    let staged_target = approve_codex_plugin_bundle_target(&paths.staged)
        .map_err(|_| "native-update-codex-inventory-invalid")?;
    let new = compose_codex_plugin_bundle_with_overrides(
        preparation.content.pack(),
        preparation.codex_grant,
        preparation.source_binary,
        &staged_target,
        preparation.workflow_overrides,
    )
    .map_err(|_| "native-update-reconciliation-prepare-failed")?;
    operations.push(plugin_operation(
        "codex-01-bundle",
        ReconciliationSurface::CodexPluginBundle,
        paths,
        old.receipt(),
        new.receipt(),
        old.receipt_sha256(),
        new.receipt_sha256(),
    )?);
    if discovered.summary().registration == CodexRegistrationState::Registered {
        operations.push(prepare_codex_registration(
            preparation,
            &discovered,
            old.receipt(),
            new.receipt(),
            new.receipt_sha256(),
        )?);
    }
    Ok(())
}

fn prepare_claude(
    preparation: &ReconciliationPreparation<'_>,
    operations: &mut Vec<ReconciliationOperationV1>,
) -> Result<(), &'static str> {
    let discovered =
        discover_claude_user_with_config(preparation.platform_home, preparation.claude_config_root)
            .map_err(|_| "native-update-claude-inventory-invalid")?;
    if matches!(
        discovered.summary().registration,
        ClaudeRegistrationState::Conflict
            | ClaudeRegistrationState::Drifted
            | ClaudeRegistrationState::RecoveryRequired
    ) {
        return Err("native-update-claude-registration-blocked");
    }
    let skills_destination = preparation.claude_config_root.join("skills/qiongli-next");
    let registered_skills = load_managed_content_registry(preparation.store.state_root())?
        .entries
        .iter()
        .any(|entry| Path::new(&entry.target) == skills_destination);
    if discovered.summary().skills_plugin == ClaudeSkillsPluginState::Conflict && !registered_skills
    {
        return Err("native-update-claude-skills-blocked");
    }
    if discovered.summary().source == ClaudeSourceState::Ready {
        let destination = preparation
            .platform_home
            .join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next");
        let old_target = approve_claude_plugin_bundle_target(&destination)
            .map_err(|_| "native-update-claude-inventory-invalid")?;
        let old = verify_claude_plugin_bundle(&old_target)
            .map_err(|_| "native-update-reconciliation-receipt-drift")?;
        validate_v2_version(&old.receipt().artifact.version)?;
        let paths = prepare_directory_paths(
            preparation.transaction_id,
            "claude-01-bundle",
            &destination,
            "qiongli-next",
        )?;
        let staged_target = approve_claude_plugin_bundle_target(&paths.staged)
            .map_err(|_| "native-update-claude-inventory-invalid")?;
        let new = compose_claude_plugin_bundle_with_overrides(
            preparation.content.pack(),
            preparation.claude_grant,
            preparation.source_binary,
            &staged_target,
            preparation.workflow_overrides,
        )
        .map_err(|_| "native-update-reconciliation-prepare-failed")?;
        operations.push(plugin_operation(
            "claude-01-bundle",
            ReconciliationSurface::ClaudePluginBundle,
            paths,
            old.receipt(),
            new.receipt(),
            old.receipt_sha256(),
            new.receipt_sha256(),
        )?);
        if discovered.summary().registration == ClaudeRegistrationState::Registered {
            operations.push(prepare_claude_registration(
                preparation,
                &discovered,
                old.receipt(),
                new.receipt(),
                new.receipt_sha256(),
            )?);
        }
    }
    if discovered.summary().skills_plugin == ClaudeSkillsPluginState::Ready {
        let destination = skills_destination;
        let old_target = approve_claude_plugin_bundle_target(&destination)
            .map_err(|_| "native-update-claude-inventory-invalid")?;
        let old = verify_claude_plugin_bundle(&old_target)
            .map_err(|_| "native-update-reconciliation-receipt-drift")?;
        validate_v2_version(&old.receipt().artifact.version)?;
        let paths = prepare_directory_paths(
            preparation.transaction_id,
            "claude-03-skills-bundle",
            &destination,
            "qiongli-next",
        )?;
        let staged_target = approve_claude_plugin_bundle_target(&paths.staged)
            .map_err(|_| "native-update-claude-inventory-invalid")?;
        let new = compose_claude_plugin_bundle_with_overrides(
            preparation.content.pack(),
            preparation.claude_grant,
            preparation.source_binary,
            &staged_target,
            preparation.workflow_overrides,
        )
        .map_err(|_| "native-update-reconciliation-prepare-failed")?;
        operations.push(plugin_operation(
            "claude-03-skills-bundle",
            ReconciliationSurface::ClaudeSkillsPluginBundle,
            paths,
            old.receipt(),
            new.receipt(),
            old.receipt_sha256(),
            new.receipt_sha256(),
        )?);
    }
    Ok(())
}

fn prepare_codex_registration(
    preparation: &ReconciliationPreparation<'_>,
    discovered: &qiongli_platform::CodexUserTarget,
    old_bundle: &CodexPluginBundleReceiptV1,
    new_bundle: &CodexPluginBundleReceiptV1,
    new_source_receipt_sha256: &str,
) -> Result<ReconciliationOperationV1, &'static str> {
    let old_state = discovered
        .registration_state()
        .cloned()
        .ok_or("native-update-codex-registration-blocked")?;
    let old_active = old_state
        .active
        .as_ref()
        .ok_or("native-update-codex-registration-blocked")?;
    if old_active.source_receipt_sha256 != sha256_hex(&canonical_json(old_bundle)?) {
        return Err("native-update-codex-registration-blocked");
    }
    let destination = discovered.registration_state_path();
    let new_receipt = reconciled_codex_receipt(
        preparation,
        old_active,
        new_bundle,
        new_source_receipt_sha256,
        &destination,
    )?;
    let new_state = CodexRegistrationStateV1 {
        schema_version: CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
        generation: old_state
            .generation
            .checked_add(1)
            .ok_or("native-update-reconciliation-invalid")?,
        install_id: old_state.install_id.clone(),
        active: Some(new_receipt),
        last_lifecycle: old_state.last_lifecycle.clone(),
    };
    prepare_state_operation(
        preparation.transaction_id,
        "codex-02-registration",
        ReconciliationSurface::CodexRegistration,
        destination,
        &old_state
            .to_canonical_json()
            .map_err(|_| "native-update-codex-registration-blocked")?,
        &new_state
            .to_canonical_json()
            .map_err(|_| "native-update-codex-registration-blocked")?,
        &old_active.artifact.version,
        &new_bundle.artifact.version,
        &old_bundle.resource_pack_sha256,
        &new_bundle.resource_pack_sha256,
        &old_active.source_receipt_sha256,
        new_source_receipt_sha256,
    )
}

fn prepare_claude_registration(
    preparation: &ReconciliationPreparation<'_>,
    discovered: &qiongli_platform::ClaudeUserTarget,
    old_bundle: &ClaudePluginBundleReceiptV1,
    new_bundle: &ClaudePluginBundleReceiptV1,
    new_source_receipt_sha256: &str,
) -> Result<ReconciliationOperationV1, &'static str> {
    let old_state = discovered
        .registration_state()
        .cloned()
        .ok_or("native-update-claude-registration-blocked")?;
    let old_active = old_state
        .active
        .as_ref()
        .ok_or("native-update-claude-registration-blocked")?;
    if old_active.source_receipt_sha256 != sha256_hex(&canonical_json(old_bundle)?) {
        return Err("native-update-claude-registration-blocked");
    }
    let destination = discovered.registration_state_path();
    let new_receipt = reconciled_claude_receipt(
        preparation,
        old_active,
        new_bundle,
        new_source_receipt_sha256,
        &destination,
    )?;
    let new_state = ClaudeRegistrationStateV1 {
        schema_version: CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
        generation: old_state
            .generation
            .checked_add(1)
            .ok_or("native-update-reconciliation-invalid")?,
        install_id: old_state.install_id.clone(),
        active: Some(new_receipt),
        last_lifecycle: old_state.last_lifecycle.clone(),
    };
    prepare_state_operation(
        preparation.transaction_id,
        "claude-02-registration",
        ReconciliationSurface::ClaudeRegistration,
        destination,
        &old_state
            .to_canonical_json()
            .map_err(|_| "native-update-claude-registration-blocked")?,
        &new_state
            .to_canonical_json()
            .map_err(|_| "native-update-claude-registration-blocked")?,
        &old_active.artifact.version,
        &new_bundle.artifact.version,
        &old_bundle.resource_pack_sha256,
        &new_bundle.resource_pack_sha256,
        &old_active.source_receipt_sha256,
        new_source_receipt_sha256,
    )
}

fn reconciled_codex_receipt(
    preparation: &ReconciliationPreparation<'_>,
    old: &CodexRegistrationReceiptV1,
    bundle: &CodexPluginBundleReceiptV1,
    source_receipt_sha256: &str,
    destination: &Path,
) -> Result<CodexRegistrationReceiptV1, &'static str> {
    let plan_sha256 = registration_plan_sha256(
        preparation.transaction_id,
        ReconciliationSurface::CodexRegistration,
        destination,
        &old.artifact.version,
        &bundle.artifact.version,
        &old.source_receipt_sha256,
        source_receipt_sha256,
        preparation.codex_grant.signed_payload_sha256(),
    )?;
    Ok(CodexRegistrationReceiptV1 {
        schema_version: old.schema_version,
        transaction_id: format!("codex-{}", preparation.transaction_id),
        plan_id: "update-reconcile-codex".to_string(),
        semantic_digest_sha256: plan_sha256,
        install_id: old.install_id.clone(),
        artifact: bundle.artifact.clone(),
        target: TargetDescriptorV1 {
            family: LocalTargetFamily::CodexLocal,
            surface: LocalSurface::DesktopLocal,
            scope: InstallScope::User,
            profile: CapabilityProfile::Lite,
            os: bundle.artifact.os,
            arch: bundle.artifact.arch,
            adapter_version: 1,
        },
        ownership: OwnershipMarkerV1 {
            schema_version: 1,
            product: ProductId::Qiongli,
            install_id: old.install_id.clone(),
            artifact_digest_sha256: preparation.codex_grant.signed_payload_sha256().to_string(),
        },
        source_receipt_sha256: source_receipt_sha256.to_string(),
        source_content_root_sha256: bundle.package_content_root_sha256.clone(),
        marketplace_entry_sha256: old.marketplace_entry_sha256.clone(),
        marketplace_document_sha256: old.marketplace_document_sha256.clone(),
        registered_at_unix: preparation.now_unix,
        outstanding_host_action: HostAction::InstallOrEnablePlugin,
    })
}

fn reconciled_claude_receipt(
    preparation: &ReconciliationPreparation<'_>,
    old: &ClaudeRegistrationReceiptV1,
    bundle: &ClaudePluginBundleReceiptV1,
    source_receipt_sha256: &str,
    destination: &Path,
) -> Result<ClaudeRegistrationReceiptV1, &'static str> {
    let plan_sha256 = registration_plan_sha256(
        preparation.transaction_id,
        ReconciliationSurface::ClaudeRegistration,
        destination,
        &old.artifact.version,
        &bundle.artifact.version,
        &old.source_receipt_sha256,
        source_receipt_sha256,
        preparation.claude_grant.signed_payload_sha256(),
    )?;
    Ok(ClaudeRegistrationReceiptV1 {
        schema_version: old.schema_version,
        transaction_id: format!("claude-{}", preparation.transaction_id),
        plan_id: "update-reconcile-claude".to_string(),
        semantic_digest_sha256: plan_sha256,
        install_id: old.install_id.clone(),
        artifact: bundle.artifact.clone(),
        target: TargetDescriptorV1 {
            family: LocalTargetFamily::ClaudeCodeLocal,
            surface: LocalSurface::CliLocal,
            scope: InstallScope::User,
            profile: CapabilityProfile::Lite,
            os: bundle.artifact.os,
            arch: bundle.artifact.arch,
            adapter_version: 1,
        },
        ownership: OwnershipMarkerV1 {
            schema_version: 1,
            product: ProductId::Qiongli,
            install_id: old.install_id.clone(),
            artifact_digest_sha256: preparation.claude_grant.signed_payload_sha256().to_string(),
        },
        source_receipt_sha256: source_receipt_sha256.to_string(),
        source_content_root_sha256: bundle.package_content_root_sha256.clone(),
        marketplace_entry_sha256: old.marketplace_entry_sha256.clone(),
        marketplace_document_sha256: old.marketplace_document_sha256.clone(),
        registered_at_unix: preparation.now_unix,
        outstanding_host_action: HostAction::InstallOrEnablePlugin,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "the operation constructor binds every old and new identity field explicitly"
)]
fn prepare_state_operation(
    transaction_id: &str,
    operation_id: &str,
    surface: ReconciliationSurface,
    destination: PathBuf,
    old_bytes: &[u8],
    new_bytes: &[u8],
    old_version: &str,
    new_version: &str,
    old_pack: &str,
    new_pack: &str,
    old_content: &str,
    new_content: &str,
) -> Result<ReconciliationOperationV1, &'static str> {
    let paths = prepare_file_paths(transaction_id, operation_id, &destination)?;
    write_new_private_file(&paths.staged, new_bytes)?;
    let old_receipt = sha256_hex(old_bytes);
    let new_receipt = sha256_hex(new_bytes);
    let mut operation = ReconciliationOperationV1 {
        operation_id: operation_id.to_string(),
        surface,
        destination,
        staged: paths.staged,
        backup: paths.backup,
        staging_container: paths.staging_container,
        old_product_version: old_version.to_string(),
        new_product_version: new_version.to_string(),
        old_pack_sha256: old_pack.to_string(),
        new_pack_sha256: new_pack.to_string(),
        old_receipt_sha256: old_receipt,
        new_receipt_sha256: new_receipt,
        old_content_sha256: old_content.to_string(),
        new_content_sha256: new_content.to_string(),
        plan_sha256: String::new(),
    };
    operation.plan_sha256 = operation_plan_sha256(&operation)?;
    Ok(operation)
}

fn plugin_operation<T>(
    operation_id: &str,
    surface: ReconciliationSurface,
    paths: PreparedPaths,
    old: &T,
    new: &T,
    old_receipt_sha256: &str,
    new_receipt_sha256: &str,
) -> Result<ReconciliationOperationV1, &'static str>
where
    T: PluginReceipt,
{
    directory_operation(
        operation_id.to_string(),
        surface,
        paths,
        old.product_version(),
        new.product_version(),
        old.pack_sha256(),
        new.pack_sha256(),
        old_receipt_sha256,
        new_receipt_sha256,
        old.content_sha256(),
        new.content_sha256(),
    )
}

#[allow(clippy::too_many_arguments)]
fn directory_operation(
    operation_id: String,
    surface: ReconciliationSurface,
    paths: PreparedPaths,
    old_version: &str,
    new_version: &str,
    old_pack: &str,
    new_pack: &str,
    old_receipt: &str,
    new_receipt: &str,
    old_content: &str,
    new_content: &str,
) -> Result<ReconciliationOperationV1, &'static str> {
    let mut operation = ReconciliationOperationV1 {
        operation_id,
        surface,
        destination: paths.destination,
        staged: paths.staged,
        backup: paths.backup,
        staging_container: paths.staging_container,
        old_product_version: old_version.to_string(),
        new_product_version: new_version.to_string(),
        old_pack_sha256: old_pack.to_string(),
        new_pack_sha256: new_pack.to_string(),
        old_receipt_sha256: old_receipt.to_string(),
        new_receipt_sha256: new_receipt.to_string(),
        old_content_sha256: old_content.to_string(),
        new_content_sha256: new_content.to_string(),
        plan_sha256: String::new(),
    };
    operation.plan_sha256 = operation_plan_sha256(&operation)?;
    Ok(operation)
}

trait PluginReceipt {
    fn product_version(&self) -> &str;
    fn pack_sha256(&self) -> &str;
    fn content_sha256(&self) -> &str;
}

impl PluginReceipt for CodexPluginBundleReceiptV1 {
    fn product_version(&self) -> &str {
        &self.artifact.version
    }

    fn pack_sha256(&self) -> &str {
        &self.resource_pack_sha256
    }

    fn content_sha256(&self) -> &str {
        &self.package_content_root_sha256
    }
}

impl PluginReceipt for ClaudePluginBundleReceiptV1 {
    fn product_version(&self) -> &str {
        &self.artifact.version
    }

    fn pack_sha256(&self) -> &str {
        &self.resource_pack_sha256
    }

    fn content_sha256(&self) -> &str {
        &self.package_content_root_sha256
    }
}

#[derive(Clone, Debug)]
struct PreparedPaths {
    destination: PathBuf,
    staged: PathBuf,
    backup: PathBuf,
    staging_container: PathBuf,
}

fn prepare_directory_paths(
    transaction_id: &str,
    operation_id: &str,
    destination: &Path,
    staged_leaf: &str,
) -> Result<PreparedPaths, &'static str> {
    validate_destination(destination)?;
    let parent = destination
        .parent()
        .ok_or("native-update-reconciliation-target-invalid")?;
    let staging_container = parent.join(format!(
        ".qiongli-reconcile-stage-{transaction_id}-{operation_id}"
    ));
    let staged = staging_container.join(staged_leaf);
    let backup = backup_path(transaction_id, operation_id, destination)?;
    ensure_absent(&staging_container)?;
    ensure_absent(&backup)?;
    create_private_directory(&staging_container)?;
    ensure_same_filesystem(parent, &staging_container)?;
    Ok(PreparedPaths {
        destination: destination.to_path_buf(),
        staged,
        backup,
        staging_container,
    })
}

fn prepare_file_paths(
    transaction_id: &str,
    operation_id: &str,
    destination: &Path,
) -> Result<PreparedPaths, &'static str> {
    let paths = prepare_directory_paths(transaction_id, operation_id, destination, "state.json")?;
    Ok(paths)
}

fn backup_path(
    transaction_id: &str,
    operation_id: &str,
    destination: &Path,
) -> Result<PathBuf, &'static str> {
    let parent = destination
        .parent()
        .ok_or("native-update-reconciliation-target-invalid")?;
    let leaf = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("native-update-reconciliation-target-invalid")?;
    Ok(parent.join(format!(
        ".{leaf}.qiongli-reconcile-backup-{transaction_id}-{operation_id}"
    )))
}

#[derive(Serialize)]
struct OperationPlanV1<'a> {
    schema_version: u32,
    operation_id: &'a str,
    surface: ReconciliationSurface,
    destination: &'a Path,
    old_product_version: &'a str,
    new_product_version: &'a str,
    old_pack_sha256: &'a str,
    new_pack_sha256: &'a str,
    old_receipt_sha256: &'a str,
    new_receipt_sha256: &'a str,
    old_content_sha256: &'a str,
    new_content_sha256: &'a str,
}

fn operation_plan_sha256(operation: &ReconciliationOperationV1) -> Result<String, &'static str> {
    let plan = OperationPlanV1 {
        schema_version: JOURNAL_SCHEMA_VERSION,
        operation_id: &operation.operation_id,
        surface: operation.surface,
        destination: &operation.destination,
        old_product_version: &operation.old_product_version,
        new_product_version: &operation.new_product_version,
        old_pack_sha256: &operation.old_pack_sha256,
        new_pack_sha256: &operation.new_pack_sha256,
        old_receipt_sha256: &operation.old_receipt_sha256,
        new_receipt_sha256: &operation.new_receipt_sha256,
        old_content_sha256: &operation.old_content_sha256,
        new_content_sha256: &operation.new_content_sha256,
    };
    Ok(sha256_hex(&canonical_json(&plan)?))
}

#[derive(Serialize)]
struct RegistrationPlanV1<'a> {
    schema_version: u32,
    transaction_id: &'a str,
    surface: ReconciliationSurface,
    destination: &'a Path,
    old_product_version: &'a str,
    new_product_version: &'a str,
    old_source_receipt_sha256: &'a str,
    new_source_receipt_sha256: &'a str,
    signed_grant_payload_sha256: &'a str,
}

#[allow(clippy::too_many_arguments)]
fn registration_plan_sha256(
    transaction_id: &str,
    surface: ReconciliationSurface,
    destination: &Path,
    old_version: &str,
    new_version: &str,
    old_source_receipt: &str,
    new_source_receipt: &str,
    grant_digest: &str,
) -> Result<String, &'static str> {
    Ok(sha256_hex(&canonical_json(&RegistrationPlanV1 {
        schema_version: JOURNAL_SCHEMA_VERSION,
        transaction_id,
        surface,
        destination,
        old_product_version: old_version,
        new_product_version: new_version,
        old_source_receipt_sha256: old_source_receipt,
        new_source_receipt_sha256: new_source_receipt,
        signed_grant_payload_sha256: grant_digest,
    })?))
}

fn validate_activation_release(
    release: &NativeActivationReleaseV1,
    journal: &ReconciliationJournalV1,
) -> Result<(), &'static str> {
    const MAX_REVISION: u64 = 9_007_199_254_740_991;
    if !valid_sha256(&release.candidate_digest_sha256)
        || release.previous_update_revision > MAX_REVISION
        || release.next.generation <= release.previous_last_accepted_generation
        || release.next.version != journal.target_version
        || release.next.resource_pack_sha256 != journal.target_pack_sha256
    {
        return Err("native-activation-release-invalid");
    }
    for known_good in std::iter::once(&release.next).chain(release.previous_last_known_good.iter())
    {
        let channel = match known_good.channel {
            qiongli_config::UpdateReleaseChannel::Alpha => qiongli_platform::ReleaseChannel::Alpha,
            qiongli_config::UpdateReleaseChannel::Beta => qiongli_platform::ReleaseChannel::Beta,
            qiongli_config::UpdateReleaseChannel::Stable => {
                qiongli_platform::ReleaseChannel::Stable
            }
        };
        if known_good.generation == 0
            || known_good.generation > MAX_REVISION
            || !valid_sha256(&known_good.archive_sha256)
            || !valid_sha256(&known_good.resource_pack_sha256)
            || qiongli_platform::current_target_native_artifact_identity(
                &known_good.version,
                channel,
            )
            .is_err()
        {
            return Err("native-activation-release-invalid");
        }
    }
    if release
        .previous_last_known_good
        .as_ref()
        .is_some_and(|known_good| known_good.generation > release.previous_last_accepted_generation)
    {
        return Err("native-activation-release-invalid");
    }
    Ok(())
}

fn matches_pre_activation_release(
    state: &qiongli_config::UpdateState,
    release: &NativeActivationReleaseV1,
) -> bool {
    state.last_accepted_generation == release.previous_last_accepted_generation
        && state.last_known_good == release.previous_last_known_good
}

fn validate_journal(journal: &ReconciliationJournalV1) -> Result<(), &'static str> {
    validate_transaction_id(&journal.transaction_id)?;
    validate_v2_version(&journal.target_version)?;
    if journal.document_kind != JOURNAL_DOCUMENT_KIND
        || !matches!(
            journal.schema_version,
            JOURNAL_SCHEMA_VERSION | CLI_JOURNAL_SCHEMA_VERSION | RELEASE_JOURNAL_SCHEMA_VERSION
        )
        || !valid_sha256(&journal.target_pack_sha256)
        || journal.operations.len()
            > MAX_OPERATIONS
                + if journal.schema_version >= CLI_JOURNAL_SCHEMA_VERSION {
                    2
                } else {
                    0
                }
    {
        return Err("native-update-reconciliation-invalid");
    }
    if (journal.schema_version == RELEASE_JOURNAL_SCHEMA_VERSION)
        != journal.native_release.is_some()
    {
        return Err("native-activation-release-invalid");
    }
    if let Some(release) = &journal.native_release {
        validate_activation_release(release, journal)?;
    }
    let cli = journal
        .operations
        .iter()
        .filter(|operation| {
            matches!(
                operation.surface,
                ReconciliationSurface::CliBinary | ReconciliationSurface::CliReceipt
            )
        })
        .collect::<Vec<_>>();
    match (journal.schema_version, cli.as_slice()) {
        (JOURNAL_SCHEMA_VERSION, []) => {}
        (CLI_JOURNAL_SCHEMA_VERSION | RELEASE_JOURNAL_SCHEMA_VERSION, [binary, receipt])
            if binary.surface == ReconciliationSurface::CliBinary
                && receipt.surface == ReconciliationSurface::CliReceipt
                && binary.old_product_version == receipt.old_product_version
                && binary.old_pack_sha256 == receipt.old_pack_sha256
                && binary.old_receipt_sha256 == receipt.old_receipt_sha256
                && binary.new_receipt_sha256 == receipt.new_receipt_sha256
                && binary.old_content_sha256 == receipt.old_content_sha256
                && binary.new_content_sha256 == receipt.new_content_sha256 => {}
        _ => return Err("native-update-reconciliation-invalid"),
    }
    let mut ids = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    for operation in &journal.operations {
        if !valid_identifier(&operation.operation_id)
            || !ids.insert(operation.operation_id.clone())
            || !destinations.insert(operation.destination.clone())
            || operation.new_product_version != journal.target_version
            || operation.new_pack_sha256 != journal.target_pack_sha256
            || !valid_v2_identity(operation)
            || operation.plan_sha256 != operation_plan_sha256(operation)?
        {
            return Err("native-update-reconciliation-invalid");
        }
        validate_operation_paths(operation)?;
    }
    Ok(())
}

fn valid_v2_identity(operation: &ReconciliationOperationV1) -> bool {
    validate_v2_version(&operation.old_product_version).is_ok()
        && validate_v2_version(&operation.new_product_version).is_ok()
        && [
            &operation.old_pack_sha256,
            &operation.new_pack_sha256,
            &operation.old_receipt_sha256,
            &operation.new_receipt_sha256,
            &operation.old_content_sha256,
            &operation.new_content_sha256,
            &operation.plan_sha256,
        ]
        .into_iter()
        .all(|digest| valid_sha256(digest))
}

fn validate_operation_paths(operation: &ReconciliationOperationV1) -> Result<(), &'static str> {
    for path in [
        &operation.destination,
        &operation.staged,
        &operation.backup,
        &operation.staging_container,
    ] {
        validate_destination(path)?;
    }
    if operation.staged.parent() != Some(operation.staging_container.as_path())
        || operation.destination == operation.staged
        || operation.destination == operation.backup
        || operation.staged == operation.backup
        || operation.staging_container.parent() != operation.destination.parent()
        || operation.backup.parent() != operation.destination.parent()
    {
        return Err("native-update-reconciliation-invalid");
    }
    Ok(())
}

fn verify_operation_identity(
    operation: &ReconciliationOperationV1,
    path: &Path,
    new: bool,
) -> Result<(), &'static str> {
    let expected_version = if new {
        &operation.new_product_version
    } else {
        &operation.old_product_version
    };
    let expected_pack = if new {
        &operation.new_pack_sha256
    } else {
        &operation.old_pack_sha256
    };
    let expected_receipt = if new {
        &operation.new_receipt_sha256
    } else {
        &operation.old_receipt_sha256
    };
    let expected_content = if new {
        &operation.new_content_sha256
    } else {
        &operation.old_content_sha256
    };
    match operation.surface {
        ReconciliationSurface::CliBinary => {
            // Stream large executables through the existing CLI owner.
            if crate::cli_install::regular_file_sha256(path)? != *expected_content {
                return Err("native-update-reconciliation-verification-failed");
            }
        }
        ReconciliationSurface::CliReceipt => {
            let bytes = read_private_file(path, MAX_STATE_BYTES)?;
            let receipt = crate::cli_install::read_receipt(path)?
                .ok_or("native-update-reconciliation-verification-failed")?;
            if receipt.product_version != *expected_version
                || receipt.installed_sha256 != *expected_content
                || sha256_hex(&bytes) != *expected_receipt
            {
                return Err("native-update-reconciliation-verification-failed");
            }
        }
        ReconciliationSurface::Skills => {
            let target = approve_materialization_target(path)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            let receipt = verify_materialization(&target)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            if &receipt.pack_sha256 != expected_pack
                || &materialization_receipt_sha256(&receipt)? != expected_receipt
                || &receipt.content_root_sha256 != expected_content
            {
                return Err("native-update-reconciliation-verification-failed");
            }
        }
        ReconciliationSurface::ManagedContentRegistry => {
            let bytes = read_private_file(path, MAX_STATE_BYTES)?;
            let registry = parse_managed_content_registry(&bytes)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            if registry.entries.is_empty()
                || registry.entries.iter().any(|entry| {
                    &entry.product_version != expected_version
                        || &entry.pack_sha256 != expected_pack
                })
                || &sha256_hex(&bytes) != expected_receipt
                || &sha256_hex(&bytes) != expected_content
            {
                return Err("native-update-reconciliation-verification-failed");
            }
        }
        ReconciliationSurface::CodexPluginBundle => {
            let target = approve_codex_plugin_bundle_target(path)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            let bundle = verify_codex_plugin_bundle(&target)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            if &bundle.receipt().artifact.version != expected_version
                || &bundle.receipt().resource_pack_sha256 != expected_pack
                || bundle.receipt_sha256() != expected_receipt
                || &bundle.receipt().package_content_root_sha256 != expected_content
            {
                return Err("native-update-reconciliation-verification-failed");
            }
        }
        ReconciliationSurface::ClaudePluginBundle
        | ReconciliationSurface::ClaudeSkillsPluginBundle => {
            let target = approve_claude_plugin_bundle_target(path)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            let bundle = verify_claude_plugin_bundle(&target)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            if &bundle.receipt().artifact.version != expected_version
                || &bundle.receipt().resource_pack_sha256 != expected_pack
                || bundle.receipt_sha256() != expected_receipt
                || &bundle.receipt().package_content_root_sha256 != expected_content
            {
                return Err("native-update-reconciliation-verification-failed");
            }
        }
        ReconciliationSurface::CodexRegistration => {
            let bytes = read_private_file(path, MAX_STATE_BYTES)?;
            let state = CodexRegistrationStateV1::from_json(&bytes)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            let active = state
                .active
                .ok_or("native-update-reconciliation-verification-failed")?;
            if &active.artifact.version != expected_version
                || &sha256_hex(&bytes) != expected_receipt
                || &active.source_receipt_sha256 != expected_content
            {
                return Err("native-update-reconciliation-verification-failed");
            }
        }
        ReconciliationSurface::ClaudeRegistration => {
            let bytes = read_private_file(path, MAX_STATE_BYTES)?;
            let state = ClaudeRegistrationStateV1::from_json(&bytes)
                .map_err(|_| "native-update-reconciliation-verification-failed")?;
            let active = state
                .active
                .ok_or("native-update-reconciliation-verification-failed")?;
            if &active.artifact.version != expected_version
                || &sha256_hex(&bytes) != expected_receipt
                || &active.source_receipt_sha256 != expected_content
            {
                return Err("native-update-reconciliation-verification-failed");
            }
        }
    }
    Ok(())
}

fn rollback_applied_operations(
    operations: &[ReconciliationOperationV1],
) -> Result<(), &'static str> {
    // Validate the entire rollback set before the first rename. Both the active
    // and interrupted-rename states retain exactly one copy of each identity.
    for operation in operations {
        if operation.backup.exists() {
            verify_operation_identity(operation, &operation.backup, false)?;
            if operation.destination.exists() {
                ensure_absent(&operation.staged)?;
                verify_operation_identity(operation, &operation.destination, true)?;
            } else {
                ensure_absent(&operation.destination)?;
                verify_operation_identity(operation, &operation.staged, true)?;
            }
        } else {
            ensure_absent(&operation.backup)?;
            verify_operation_identity(operation, &operation.destination, false)?;
            if operation.staged.exists() {
                verify_operation_identity(operation, &operation.staged, true)?;
            } else {
                ensure_absent(&operation.staged)?;
            }
        }
    }
    for operation in operations.iter().rev() {
        if !operation.backup.exists() {
            continue;
        }
        if operation.destination.exists() {
            ensure_absent(&operation.staged)?;
            rename_without_replacement(&operation.destination, &operation.staged)?;
        }
        rename_without_replacement(&operation.backup, &operation.destination)?;
        sync_operation_parent(operation)?;
    }
    Ok(())
}

fn cleanup_staging_operations(operations: &[ReconciliationOperationV1]) {
    for operation in operations {
        let _ = remove_empty_or_staged_container(&operation.staging_container);
    }
}

fn remove_empty_or_staged_container(path: &Path) -> Result<(), &'static str> {
    if !path.exists() {
        return Ok(());
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "native-update-reconciliation-cleanup-required")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("native-update-reconciliation-cleanup-required");
    }
    fs::remove_dir_all(path).map_err(|_| "native-update-reconciliation-cleanup-required")
}

fn remove_path(path: &Path) -> Result<(), &'static str> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "native-update-reconciliation-cleanup-required")?;
    if metadata.file_type().is_symlink() {
        return Err("native-update-reconciliation-cleanup-required");
    }
    if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else if metadata.is_file() {
        fs::remove_file(path)
    } else {
        return Err("native-update-reconciliation-cleanup-required");
    }
    .map_err(|_| "native-update-reconciliation-cleanup-required")
}

fn sync_operation_parent(operation: &ReconciliationOperationV1) -> Result<(), &'static str> {
    sync_directory(
        operation
            .destination
            .parent()
            .ok_or("native-update-reconciliation-invalid")?,
    )
}

fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder
            .create(path)
            .map_err(|_| "native-update-reconciliation-prepare-failed")
    }
    #[cfg(not(unix))]
    {
        fs::create_dir(path).map_err(|_| "native-update-reconciliation-prepare-failed")
    }
}

fn verify_existing_private_directory(path: &Path) -> Result<(), &'static str> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "native-update-reconciliation-target-invalid")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("native-update-reconciliation-target-invalid");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & 0o077 != 0
        {
            return Err("native-update-reconciliation-target-invalid");
        }
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), &'static str> {
    match fs::symlink_metadata(path) {
        Ok(_) => verify_existing_private_directory(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_private_directory(path)
        }
        Err(_) => Err("native-update-reconciliation-target-invalid"),
    }
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(path)
        .map_err(|_| "native-update-reconciliation-prepare-failed")?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "native-update-reconciliation-prepare-failed")
}

fn read_private_file(path: &Path, maximum_size: u64) -> Result<Vec<u8>, &'static str> {
    #[cfg(unix)]
    let file = {
        let descriptor = rustix::fs::open(
            path,
            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW,
            rustix::fs::Mode::empty(),
        )
        .map_err(|_| "native-update-reconciliation-invalid")?;
        File::from(descriptor)
    };
    #[cfg(not(unix))]
    let file = File::open(path).map_err(|_| "native-update-reconciliation-invalid")?;
    let metadata = file
        .metadata()
        .map_err(|_| "native-update-reconciliation-invalid")?;
    if !metadata.is_file() || metadata.len() > maximum_size {
        return Err("native-update-reconciliation-invalid");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o077 != 0 {
            return Err("native-update-reconciliation-invalid");
        }
    }
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
    file.take(maximum_size.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "native-update-reconciliation-invalid")?;
    if bytes.len() as u64 > maximum_size {
        return Err("native-update-reconciliation-invalid");
    }
    Ok(bytes)
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
    .map_err(|_| "native-update-reconciliation-activation-failed")
}

#[cfg(not(target_os = "macos"))]
fn rename_without_replacement(source: &Path, destination: &Path) -> Result<(), &'static str> {
    ensure_absent(destination)?;
    fs::rename(source, destination).map_err(|_| "native-update-reconciliation-activation-failed")
}

fn ensure_absent(path: &Path) -> Result<(), &'static str> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err("native-update-reconciliation-collision"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("native-update-reconciliation-invalid"),
    }
}

fn ensure_same_filesystem(first: &Path, second: &Path) -> Result<(), &'static str> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let first =
            fs::metadata(first).map_err(|_| "native-update-reconciliation-target-invalid")?;
        let second =
            fs::metadata(second).map_err(|_| "native-update-reconciliation-target-invalid")?;
        if first.dev() != second.dev() {
            return Err("native-update-reconciliation-cross-device");
        }
    }
    #[cfg(not(unix))]
    let _ = (first, second);
    Ok(())
}

fn validate_destination(path: &Path) -> Result<(), &'static str> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err("native-update-reconciliation-target-invalid");
    }
    Ok(())
}

pub(crate) fn validate_transaction_id(value: &str) -> Result<(), &'static str> {
    if value.strip_prefix("update-").is_some_and(|suffix| {
        suffix.len() == 32 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
    }) {
        Ok(())
    } else {
        Err("native-update-reconciliation-invalid")
    }
}

fn validate_v2_version(value: &str) -> Result<(), &'static str> {
    Version::parse(value)
        .ok()
        .filter(|version| version.major >= 2 && version.build.is_empty())
        .map(|_| ())
        .ok_or("native-update-reconciliation-legacy-content")
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn profile_name(profile: ProfileId) -> &'static str {
    match profile {
        ProfileId::SkillOnly => "skill-only",
        ProfileId::MarketplaceLite => "marketplace-lite",
        ProfileId::Full => "full",
    }
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| "native-update-reconciliation-invalid")
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn sync_directory(path: &Path) -> Result<(), &'static str> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| "native-update-reconciliation-persistence-failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_legacy_content_identity() {
        assert_eq!(
            validate_v2_version("1.19.0"),
            Err("native-update-reconciliation-legacy-content")
        );
        assert!(validate_v2_version("2.0.0-alpha.1").is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn cli_pair_activation_recovers_both_files_and_rejects_incomplete_journals() {
        use crate::cli_install::{apply_cli_install, preview_cli_install};
        use std::os::unix::fs::PermissionsExt;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("target/qiongli-cli-reconciliation-tests")
            .join(std::process::id().to_string());
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let source = root.join("source-cli");
        fs::write(&source, b"old-cli").unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o700)).unwrap();
        let initial = preview_cli_install(&root, &source, "2.0.0-alpha.4").unwrap();
        apply_cli_install(&initial).unwrap();
        let old_receipt = fs::read(&initial.receipt_path).unwrap();
        fs::write(&source, b"new-cli").unwrap();
        let plan = preview_cli_install(&root, &source, "2.0.0-alpha.5").unwrap();
        let transaction = "update-0123456789abcdef0123456789abcdef";
        let mut operations = Vec::new();
        prepare_cli_update_operations(
            &plan,
            transaction,
            &"1".repeat(64),
            &"2".repeat(64),
            &mut operations,
        )
        .unwrap();
        assert_eq!(fs::read(&initial.target).unwrap(), b"old-cli");
        assert_eq!(fs::read(&initial.receipt_path).unwrap(), old_receipt);
        let mut journal = ReconciliationJournalV1 {
            native_release: None,
            document_kind: JOURNAL_DOCUMENT_KIND.into(),
            schema_version: CLI_JOURNAL_SCHEMA_VERSION,
            transaction_id: transaction.into(),
            target_version: "2.0.0-alpha.5".into(),
            target_pack_sha256: "2".repeat(64),
            operations,
        };
        verify_prepared_reconciliation(&journal).unwrap();
        for version in [JOURNAL_SCHEMA_VERSION, RELEASE_JOURNAL_SCHEMA_VERSION, 4] {
            journal.schema_version = version;
            assert!(activate_prepared_reconciliation(&journal).is_err());
            assert_eq!(fs::read(&initial.target).unwrap(), b"old-cli");
        }
        journal.schema_version = CLI_JOURNAL_SCHEMA_VERSION;
        let mut incomplete = journal.clone();
        incomplete.operations.pop();
        assert!(activate_prepared_reconciliation(&incomplete).is_err());
        let mut mismatched = journal.clone();
        mismatched.operations[1].old_content_sha256 = "3".repeat(64);
        mismatched.operations[1].plan_sha256 =
            operation_plan_sha256(&mismatched.operations[1]).unwrap();
        assert!(validate_journal(&mismatched).is_err());
        // A freshly decoded journal recovers after the binary moves but the
        // receipt still describes the predecessor.
        let bytes = canonical_json(&journal).unwrap();
        let journal: ReconciliationJournalV1 = serde_json::from_slice(&bytes).unwrap();
        let binary = &journal.operations[0];
        fs::rename(&binary.destination, &binary.backup).unwrap();
        fs::rename(&binary.staged, &binary.destination).unwrap();
        rollback_active_reconciliation(&journal).unwrap();
        assert_eq!(fs::read(&initial.target).unwrap(), b"old-cli");
        assert_eq!(fs::read(&initial.receipt_path).unwrap(), old_receipt);
        activate_prepared_reconciliation(&journal).unwrap();
        verify_active_reconciliation(&journal).unwrap();
        assert_eq!(fs::read(&initial.target).unwrap(), b"new-cli");
        let active_receipt = fs::read(&initial.receipt_path).unwrap();
        fs::write(&initial.receipt_path, b"receipt-drift-canary").unwrap();
        assert!(rollback_active_reconciliation(&journal).is_err());
        assert_eq!(fs::read(&initial.target).unwrap(), b"new-cli");
        assert_eq!(
            fs::read(&initial.receipt_path).unwrap(),
            b"receipt-drift-canary"
        );
        fs::write(&initial.receipt_path, active_receipt).unwrap();
        rollback_active_reconciliation(&journal).unwrap();
        rollback_active_reconciliation(&journal).unwrap();
        assert_eq!(fs::read(&initial.target).unwrap(), b"old-cli");
        assert_eq!(fs::read(&initial.receipt_path).unwrap(), old_receipt);
        let config = qiongli_config::resolve_config_root(Some(root.as_os_str()), &root).unwrap();
        let store = UpdateStateStore::new(config, qiongli_config::UpdateStreamPreference::Beta);
        prepare_reconciliation_transaction_root(&store, transaction).unwrap();
        let journal_path = store
            .staging_root()
            .join(transaction)
            .join(RECONCILIATION_JOURNAL_FILE);
        write_new_private_file(&journal_path, &canonical_json(&journal).unwrap()).unwrap();
        let journal = load_reconciliation_journal(&store, transaction).unwrap();
        let canary = journal.operations[1]
            .staging_container
            .join("foreign-canary");
        fs::write(&canary, b"keep-this-file").unwrap();
        assert!(discard_prepared_reconciliation(&store, transaction).is_err());
        assert!(journal_path.exists());
        assert!(journal.operations[0].staged.exists());
        assert_eq!(fs::read(&canary).unwrap(), b"keep-this-file");
        fs::remove_file(canary).unwrap();
        // Resume after the first owned staged file was already deleted.
        remove_path(&journal.operations[0].staged).unwrap();
        discard_prepared_reconciliation(&store, transaction).unwrap();
        discard_prepared_reconciliation(&store, transaction).unwrap();
        assert!(!journal_path.exists());
        cleanup_rolled_back_reconciliation(&journal).unwrap();
        let mut next = journal.clone();
        next.operations.clear();
        prepare_cli_update_operations(
            &plan,
            transaction,
            &"1".repeat(64),
            &"2".repeat(64),
            &mut next.operations,
        )
        .unwrap();
        activate_prepared_reconciliation(&next).unwrap();
        // Resume committed cleanup after deleting only one old backup.
        remove_path(&next.operations[0].backup).unwrap();
        cleanup_committed_reconciliation(&next).unwrap();
        cleanup_committed_reconciliation(&next).unwrap();
        assert_eq!(fs::read(&initial.target).unwrap(), b"new-cli");
        assert_eq!(
            crate::cli_install::read_receipt(&initial.receipt_path)
                .unwrap()
                .unwrap()
                .product_version,
            "2.0.0-alpha.5"
        );
        let other_config =
            qiongli_config::resolve_config_root(Some(root.join("other-config").as_os_str()), &root)
                .unwrap();
        let held = acquire_managed_write_guard(&root, other_config.clone()).unwrap();
        assert_eq!(
            acquire_native_home_write_lock(&root).unwrap_err(),
            "native-update-replacement-active"
        );
        drop(held);
        for (index, mode) in [
            "failed-health",
            "interrupted-health",
            "committed-cleanup-interrupted",
            "committed",
        ]
        .into_iter()
        .enumerate()
        {
            fs::write(&source, b"coordinated-cli").unwrap();
            let plan = preview_cli_install(&root, &source, "2.0.0-alpha.6").unwrap();
            let id = format!("update-{:032x}", index + 1);
            prepare_reconciliation_transaction_root(&store, &id).unwrap();
            let mut operations = Vec::new();
            prepare_cli_update_operations(
                &plan,
                &id,
                &"2".repeat(64),
                &"3".repeat(64),
                &mut operations,
            )
            .unwrap();
            let journal = ReconciliationJournalV1 {
                native_release: None,
                document_kind: JOURNAL_DOCUMENT_KIND.into(),
                schema_version: CLI_JOURNAL_SCHEMA_VERSION,
                transaction_id: id.clone(),
                target_version: plan.product_version.clone(),
                target_pack_sha256: "3".repeat(64),
                operations,
            };
            write_new_private_file(
                &store
                    .staging_root()
                    .join(&id)
                    .join(RECONCILIATION_JOURNAL_FILE),
                &canonical_json(&journal).unwrap(),
            )
            .unwrap();
            let digest = reconciliation_journal_sha256(&journal).unwrap();
            assert_eq!(
                activate_native_reconciliation(&store, &id, &"0".repeat(64), || Ok(())),
                Err("native-activation-journal-mismatch")
            );
            assert!(!native_record_exists(&store, &id, NATIVE_ACTIVATION_RECORD).unwrap());
            let held = acquire_replacement_lock(&store).unwrap();
            assert_eq!(
                activate_native_reconciliation(&store, &id, &digest, || Ok(())),
                Err("native-update-replacement-active")
            );
            drop(held);
            if index == 0 {
                let before = store.load().unwrap();
                let mut other = before.state.clone();
                other.active_transaction = Some(qiongli_config::UpdateActiveTransaction {
                    transaction_id: "update-ffffffffffffffffffffffffffffffff".into(),
                    target_version: "2.0.0-alpha.6".into(),
                    phase: qiongli_config::UpdateTransactionPhase::Activating,
                });
                let changed = store.replace(before.revision, other.clone()).unwrap();
                assert_eq!(
                    activate_native_reconciliation(&store, &id, &digest, || Ok(())),
                    Err("native-update-transaction-active")
                );
                assert_eq!(store.load().unwrap().state, other);
                assert!(!native_record_exists(&store, &id, NATIVE_ACTIVATION_RECORD).unwrap());
                store.replace(changed.revision, before.state).unwrap();
            }
            match mode {
                "failed-health" => assert_eq!(
                    activate_native_reconciliation(&store, &id, &digest, || Err(
                        "test-health-failed"
                    )),
                    Err("test-health-failed")
                ),
                "interrupted-health" => {
                    let mut competing_write_error = None;
                    let interrupted =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let _ = activate_native_reconciliation(&store, &id, &digest, || {
                                competing_write_error =
                                    acquire_managed_write_guard(&root, other_config.clone()).err();
                                panic!("simulated-process-interruption")
                            });
                        }));
                    assert!(interrupted.is_err());
                    assert_eq!(
                        competing_write_error,
                        Some("native-update-replacement-active")
                    );
                    assert!(store.load().unwrap().state.active_transaction.is_some());
                    assert_eq!(fs::read(&plan.target).unwrap(), b"coordinated-cli");
                }
                "committed-cleanup-interrupted" => {
                    let canary = journal.operations[1].staging_container.join("late-canary");
                    assert_eq!(
                        activate_native_reconciliation(&store, &id, &digest, || {
                            fs::write(&canary, b"late-file").unwrap();
                            Ok(())
                        }),
                        Err("native-update-reconciliation-cleanup-required")
                    );
                    assert!(native_record_exists(&store, &id, NATIVE_ACTIVATION_OUTCOME).unwrap());
                    assert_eq!(fs::read(&plan.target).unwrap(), b"coordinated-cli");
                    fs::remove_file(canary).unwrap();
                }
                _ => assert_eq!(
                    activate_native_reconciliation(&store, &id, &digest, || Ok(())),
                    Ok(NativeActivationOutcome::Committed)
                ),
            }
            if mode == "interrupted-health" || mode == "committed-cleanup-interrupted" {
                assert_eq!(
                    acquire_managed_write_guard(&root, other_config.clone()).unwrap_err(),
                    "native-activation-recovery-required"
                );
            }
            if mode == "interrupted-health" {
                let marker = root.join(".qiongli/native").join(HOME_ACTIVATION_MARKER);
                let original_marker = fs::read(&marker).unwrap();
                fs::write(&marker, b"another-transaction").unwrap();
                assert_eq!(
                    recover_native_reconciliation(&store, &id, &digest),
                    Err("native-activation-recovery-required")
                );
                assert_eq!(fs::read(&plan.target).unwrap(), b"coordinated-cli");
                fs::write(&marker, original_marker).unwrap();
                let record_path = store
                    .staging_root()
                    .join(&id)
                    .join(NATIVE_ACTIVATION_RECORD);
                let original = fs::read(&record_path).unwrap();
                fs::write(&record_path, b"invalid-private-record").unwrap();
                assert_eq!(
                    recover_native_reconciliation(&store, &id, &digest),
                    Err("native-activation-record-invalid")
                );
                assert_eq!(fs::read(&plan.target).unwrap(), b"coordinated-cli");
                assert!(store.load().unwrap().state.active_transaction.is_some());
                fs::write(&record_path, original).unwrap();
            }
            let expected = if mode.starts_with("committed") {
                NativeActivationOutcome::Committed
            } else {
                NativeActivationOutcome::RolledBack
            };
            assert_eq!(
                recover_native_reconciliation(&store, &id, &digest),
                Ok(expected)
            );
            let recovered = store.load().unwrap();
            assert!(recovered.state.active_transaction.is_none());
            assert!(
                !root
                    .join(".qiongli/native")
                    .join(HOME_ACTIVATION_MARKER)
                    .exists()
            );
            drop(acquire_managed_write_guard(&root, other_config.clone()).unwrap());
            assert_eq!(
                recover_native_reconciliation(&store, &id, &digest),
                Ok(expected)
            );
            assert_eq!(store.load().unwrap().revision, recovered.revision);
            assert_eq!(
                fs::read(&plan.target).unwrap(),
                if mode.starts_with("committed") {
                    b"coordinated-cli".as_slice()
                } else {
                    b"new-cli".as_slice()
                }
            );
            assert_eq!(
                activate_native_reconciliation(&store, &id, &digest, || panic!(
                    "must not rerun health"
                )),
                Err("native-activation-recovery-required")
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    #[allow(clippy::disallowed_methods)]
    fn native_activation_and_recovery_refuse_live_cli_files() {
        use crate::cli_install::{apply_cli_install, preview_cli_install};
        use std::os::unix::fs::PermissionsExt;
        use std::process::{Command, Stdio};
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/native-live-cli-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let source = root.join("source-cli");
        fs::copy("/bin/sleep", &source).unwrap();
        let initial = preview_cli_install(&root, &source, "2.0.0-alpha.4").unwrap();
        apply_cli_install(&initial).unwrap();
        let receipt = fs::read(&initial.receipt_path).unwrap();
        let plan = preview_cli_install(&root, &source, "2.0.0-alpha.5").unwrap();
        let config = qiongli_config::resolve_config_root(None, &root).unwrap();
        qiongli_config::GlobalSettingsStore::new(config.clone())
            .prepare_store()
            .unwrap();
        let store = UpdateStateStore::new(config, qiongli_config::UpdateStreamPreference::Beta);
        let transaction = "update-0123456789abcdef0123456789abcdef";
        prepare_reconciliation_transaction_root(&store, transaction).unwrap();
        let mut operations = Vec::new();
        prepare_cli_update_operations(
            &plan,
            transaction,
            &"1".repeat(64),
            &"2".repeat(64),
            &mut operations,
        )
        .unwrap();
        let journal = ReconciliationJournalV1 {
            native_release: None,
            document_kind: JOURNAL_DOCUMENT_KIND.into(),
            schema_version: CLI_JOURNAL_SCHEMA_VERSION,
            transaction_id: transaction.into(),
            target_version: plan.product_version.clone(),
            target_pack_sha256: "2".repeat(64),
            operations,
        };
        write_new_private_file(
            &store
                .staging_root()
                .join(transaction)
                .join(RECONCILIATION_JOURNAL_FILE),
            &canonical_json(&journal).unwrap(),
        )
        .unwrap();
        let digest = reconciliation_journal_sha256(&journal).unwrap();
        let cli = journal
            .operations
            .iter()
            .find(|op| op.surface == ReconciliationSurface::CliBinary)
            .unwrap();
        let exercise = |path: &Path, recovery: bool| {
            let before = store.load().unwrap();
            let mut child = Command::new(path)
                .arg("30")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let result = if recovery {
                recover_native_reconciliation(&store, transaction, &digest)
            } else {
                activate_native_reconciliation(&store, transaction, &digest, || {
                    panic!("live CLI must prevent health")
                })
            };
            let _ = child.kill();
            child.wait().unwrap();
            assert_eq!(result, Err("native-update-application-running"));
            assert_eq!(store.load().unwrap(), before);
        };
        for path in [&cli.destination, &cli.staged] {
            exercise(path, false);
            assert!(!native_record_exists(&store, transaction, NATIVE_ACTIVATION_RECORD).unwrap());
            assert_eq!(fs::read(&initial.receipt_path).unwrap(), receipt);
        }
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = activate_native_reconciliation(&store, transaction, &digest, || {
                panic!("interrupted health")
            });
        }));
        assert!(interrupted.is_err());
        let marker = root.join(".qiongli/native").join(HOME_ACTIVATION_MARKER);
        let marker_bytes = fs::read(&marker).unwrap();
        for path in [&cli.destination, &cli.backup] {
            exercise(path, true);
            assert_eq!(fs::read(&marker).unwrap(), marker_bytes);
            assert!(cli.backup.exists());
        }
        assert_eq!(
            recover_native_reconciliation(&store, transaction, &digest),
            Ok(NativeActivationOutcome::RolledBack)
        );
        assert_eq!(fs::read(&initial.receipt_path).unwrap(), receipt);
        assert!(!marker.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rel_913_activation_and_rollback_preserve_non_product_canaries() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("target/qiongli-reconciliation-tests")
            .join(format!("{}-{}", std::process::id(), 1));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();

        let canaries = [
            ("config.json", b"config-byte-canary".as_slice()),
            ("secret.ref", b"secret-reference-canary".as_slice()),
            ("research.db", b"research-data-canary".as_slice()),
            ("unmanaged.txt", b"unmanaged-host-canary".as_slice()),
            ("legacy-1x.bin", b"legacy-one-x-canary".as_slice()),
        ];
        for (name, bytes) in canaries {
            fs::write(root.join(name), bytes).unwrap();
        }

        let content = crate::embedded_content().unwrap();
        let transaction_id = "update-0123456789abcdef0123456789abcdef";
        let destination = root.join("managed-skills");
        let old_target = approve_materialization_target(&destination).unwrap();
        let old = content
            .materialize_profile("skill-only", &old_target)
            .unwrap();
        let old_inode = fs::metadata(&destination).unwrap().ino();

        let paths =
            prepare_directory_paths(transaction_id, "skills-000", &destination, "content").unwrap();
        let staged_target = approve_materialization_target(&paths.staged).unwrap();
        let new = content
            .materialize_profile("skill-only", &staged_target)
            .unwrap();
        let staged_inode = fs::metadata(&paths.staged).unwrap().ino();
        let skills_operation = directory_operation(
            "skills-000".to_string(),
            ReconciliationSurface::Skills,
            paths,
            env!("CARGO_PKG_VERSION"),
            "2.0.0-alpha.2",
            &old.pack_sha256,
            &new.pack_sha256,
            &materialization_receipt_sha256(&old).unwrap(),
            &materialization_receipt_sha256(&new).unwrap(),
            &old.content_root_sha256,
            &new.content_root_sha256,
        )
        .unwrap();
        let state_root = root.join("state/v2");
        crate::managed_content::register_managed_materialization(&state_root, &old_target, &old)
            .unwrap();
        let registry_destination = managed_content_registry_path(&state_root);
        let old_registry_inode = fs::metadata(&registry_destination).unwrap().ino();
        let old_registry_bytes = fs::read(&registry_destination).unwrap();
        let mut new_registry = load_managed_content_registry(&state_root).unwrap();
        new_registry.generation += 1;
        new_registry.entries[0].product_version = "2.0.0-alpha.2".to_string();
        new_registry.entries[0].receipt_sha256 = materialization_receipt_sha256(&new).unwrap();
        new_registry.entries[0].pack_sha256 = new.pack_sha256.clone();
        new_registry.entries[0].content_root_sha256 = new.content_root_sha256.clone();
        let new_registry_bytes = managed_content_registry_bytes(&new_registry).unwrap();
        let old_registry_sha256 = sha256_hex(&old_registry_bytes);
        let new_registry_sha256 = sha256_hex(&new_registry_bytes);
        let registry_operation = prepare_state_operation(
            transaction_id,
            "skills-registry",
            ReconciliationSurface::ManagedContentRegistry,
            registry_destination.clone(),
            &old_registry_bytes,
            &new_registry_bytes,
            env!("CARGO_PKG_VERSION"),
            "2.0.0-alpha.2",
            &old.pack_sha256,
            &new.pack_sha256,
            &old_registry_sha256,
            &new_registry_sha256,
        )
        .unwrap();
        let staged_registry_inode = fs::metadata(&registry_operation.staged).unwrap().ino();
        let mut journal = ReconciliationJournalV1 {
            native_release: None,
            document_kind: JOURNAL_DOCUMENT_KIND.to_string(),
            schema_version: JOURNAL_SCHEMA_VERSION,
            transaction_id: transaction_id.to_string(),
            target_version: "2.0.0-alpha.2".to_string(),
            target_pack_sha256: content.pack().pack_sha256().to_string(),
            operations: vec![skills_operation, registry_operation],
        };

        // Non-empty v1 journals keep their canonical representation and remain
        // readable before adding the explicitly versioned CLI pair.
        let legacy_bytes = canonical_json(&journal).unwrap();
        let legacy: ReconciliationJournalV1 = serde_json::from_slice(&legacy_bytes).unwrap();
        verify_prepared_reconciliation(&legacy).unwrap();
        assert_eq!(canonical_json(&legacy).unwrap(), legacy_bytes);
        let cli_source = root.join("cli-source");
        fs::write(&cli_source, b"old-shared-cli").unwrap();
        fs::set_permissions(&cli_source, fs::Permissions::from_mode(0o700)).unwrap();
        let old_cli =
            crate::cli_install::preview_cli_install(&root, &cli_source, env!("CARGO_PKG_VERSION"))
                .unwrap();
        crate::cli_install::apply_cli_install(&old_cli).unwrap();
        let old_cli_receipt = fs::read(&old_cli.receipt_path).unwrap();
        fs::write(&cli_source, b"new-shared-cli").unwrap();
        let new_cli =
            crate::cli_install::preview_cli_install(&root, &cli_source, "2.0.0-alpha.2").unwrap();
        prepare_cli_update_operations(
            &new_cli,
            transaction_id,
            &old.pack_sha256,
            &new.pack_sha256,
            &mut journal.operations,
        )
        .unwrap();
        journal.schema_version = CLI_JOURNAL_SCHEMA_VERSION;
        activate_prepared_reconciliation(&journal).unwrap();
        assert_eq!(fs::read(&new_cli.target).unwrap(), b"new-shared-cli");
        assert_eq!(fs::metadata(&destination).unwrap().ino(), staged_inode);
        assert_eq!(
            fs::metadata(&registry_destination).unwrap().ino(),
            staged_registry_inode
        );
        assert_eq!(
            parse_managed_content_registry(&fs::read(&registry_destination).unwrap())
                .unwrap()
                .entries[0]
                .product_version,
            "2.0.0-alpha.2"
        );
        assert_eq!(
            fs::metadata(&journal.operations[0].backup).unwrap().ino(),
            old_inode
        );
        // Validate every operation before moving any surface, including one earlier
        // in the reverse rollback order whose backup or active bytes have drifted.
        for path in [&journal.operations[1].backup, &registry_destination] {
            let bytes = fs::read(path).unwrap();
            fs::write(path, b"modified-private-canary").unwrap();
            assert!(rollback_active_reconciliation(&journal).is_err());
            assert_eq!(fs::read(&new_cli.target).unwrap(), b"new-shared-cli");
            assert_eq!(fs::metadata(&destination).unwrap().ino(), staged_inode);
            assert_eq!(
                fs::metadata(&registry_destination).unwrap().ino(),
                staged_registry_inode
            );
            assert_eq!(fs::read(path).unwrap(), b"modified-private-canary");
            for operation in &journal.operations {
                assert!(operation.backup.exists());
                assert!(!operation.staged.exists());
            }
            fs::write(path, bytes).unwrap();
        }
        // A dangling backup link in the last rollback operation must not cause
        // the earlier registry operation to move before the failure is detected.
        let backup = &journal.operations[0].backup;
        let held_backup = root.join("held-skills-backup");
        fs::rename(backup, &held_backup).unwrap();
        std::os::unix::fs::symlink(root.join("missing-backup-target"), backup).unwrap();
        assert!(rollback_active_reconciliation(&journal).is_err());
        assert_eq!(
            fs::metadata(&registry_destination).unwrap().ino(),
            staged_registry_inode
        );
        assert_eq!(fs::metadata(&destination).unwrap().ino(), staged_inode);
        assert!(
            fs::symlink_metadata(backup)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        fs::remove_file(backup).unwrap();
        fs::rename(&held_backup, backup).unwrap();

        // Resume a rollback interrupted between moving the active file aside and
        // restoring its backup, without moving unrelated state.
        fs::rename(&registry_destination, &journal.operations[1].staged).unwrap();
        rollback_active_reconciliation(&journal).unwrap();
        rollback_active_reconciliation(&journal).unwrap();
        assert_eq!(fs::metadata(&destination).unwrap().ino(), old_inode);
        assert_eq!(
            fs::metadata(&registry_destination).unwrap().ino(),
            old_registry_inode
        );
        assert_eq!(
            parse_managed_content_registry(&fs::read(&registry_destination).unwrap())
                .unwrap()
                .entries[0]
                .product_version,
            env!("CARGO_PKG_VERSION")
        );
        // The other interrupted rename state occurs during activation, with the
        // old destination backed up and the new bytes still staged.
        fs::rename(&destination, &journal.operations[0].backup).unwrap();
        rollback_active_reconciliation(&journal).unwrap();
        assert_eq!(fs::metadata(&destination).unwrap().ino(), old_inode);
        assert_eq!(fs::read(&old_cli.target).unwrap(), b"old-shared-cli");
        assert_eq!(fs::read(&old_cli.receipt_path).unwrap(), old_cli_receipt);
        cleanup_rolled_back_reconciliation(&journal).unwrap();

        for (name, bytes) in canaries {
            assert_eq!(fs::read(root.join(name)).unwrap(), bytes);
        }
        let _ = fs::remove_dir_all(root);
    }
}
