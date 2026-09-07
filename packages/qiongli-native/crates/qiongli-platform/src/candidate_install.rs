use std::fmt::{self, Display, Formatter};
use std::fs::{self, Metadata};
use std::io;
#[cfg(not(any(unix, windows)))]
use std::path::Component;
use std::path::Path;

use qiongli_content::LoadedResourcePack;

use crate::transaction::{
    create_private_new_file, path_exists, read_private_file, rename_path, sync_directory,
    transaction_id, write_sync_file,
};

use crate::{
    AllowedRootV1, ApprovalRequirement, ApprovedManagedRoot, CapabilityProfile, ClaudeAdapterError,
    ClaudeRegistrationCommit, ClaudeRegistrationDisposition, ClaudeRegistrationExecutor,
    ClaudeRegistrationLifecycleCommit, ClaudeRegistrationVerification, ClientActivationTarget,
    CodexAdapterError, CodexRegistrationCommit, CodexRegistrationDisposition,
    CodexRegistrationExecutor, CodexRegistrationLifecycleCommit, CodexRegistrationVerification,
    HostAction, InstallDisposition, InstallPlanMetadataV1, InstallScope, InstallerKind,
    LocalSurface, LocalTargetFamily, ManagedNativePayloadExecutor,
    NativeCandidatePluginSourceCommit, NativeCandidatePluginSourceDisposition,
    NativeCandidatePluginSourceError, NativeCandidatePluginSourceTarget,
    NativeCandidatePluginSourceVerification, NativePayloadInstallCommit,
    NativePayloadInstallVerification, NativePayloadLifecycleCommit, NativeReleaseAuthority,
    NativeReleaseCandidateError, NativeReleaseCandidateVerificationContext, PlatformError,
    SignedNativeReleaseCandidateV1, SymbolicRoot, TargetDescriptorV1, TransactionError,
    VerifiedInstalledNativeCandidate, VerifiedNativeReleaseCandidate, approve_install_plan,
    approve_managed_root, approve_native_artifact_target, discover_claude_user,
    discover_codex_user, discover_native_candidate_plugin_source_target,
    materialize_native_candidate_plugin_source, native_artifact_binary_path, native_artifact_id,
    native_release_candidate_file_name, prepare_native_candidate_plugin_source_target,
    preview_claude_registration, preview_codex_registration, preview_native_payload_install,
    remove_native_candidate_plugin_source, verify_native_candidate_plugin_source,
};

const PLAN_TTL_SECONDS: u64 = 600;
const MANAGED_ROOT_ID: &str = "qiongli-native-payloads";
pub const NATIVE_CANDIDATE_MANAGED_ROOT_SYMBOLIC_PATH: &str =
    "<user-home>/.qiongli/native/payloads";
const PAYLOAD_APPROVALS: [ApprovalRequirement; 1] = [ApprovalRequirement::FilesystemWrite];
const REGISTRATION_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeCandidateRegistrationCommit {
    Codex(CodexRegistrationCommit),
    ClaudeCode(ClaudeRegistrationCommit),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeCandidateRegistrationVerification {
    Codex(CodexRegistrationVerification),
    ClaudeCode(ClaudeRegistrationVerification),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeCandidateRegistrationLifecycleCommit {
    Codex(CodexRegistrationLifecycleCommit),
    ClaudeCode(ClaudeRegistrationLifecycleCommit),
}

impl NativeCandidateRegistrationCommit {
    fn was_fresh(&self) -> bool {
        match self {
            Self::Codex(commit) => commit.disposition == CodexRegistrationDisposition::Registered,
            Self::ClaudeCode(commit) => {
                commit.disposition == ClaudeRegistrationDisposition::Registered
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCandidateLocalInstallCommit {
    pub target: ClientActivationTarget,
    pub payload: NativePayloadInstallCommit,
    pub source: NativeCandidatePluginSourceCommit,
    pub registration: NativeCandidateRegistrationCommit,
    pub outstanding_host_action: HostAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCandidateLocalVerification {
    pub target: ClientActivationTarget,
    pub payload: NativePayloadInstallVerification,
    pub source: NativeCandidatePluginSourceVerification,
    pub registration: NativeCandidateRegistrationVerification,
    pub outstanding_host_action: HostAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCandidateLocalRemoveCommit {
    pub target: ClientActivationTarget,
    pub registration: NativeCandidateRegistrationLifecycleCommit,
    pub source: NativeCandidatePluginSourceVerification,
    pub payload: NativePayloadLifecycleCommit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeCandidateLocalInstallError {
    Platform(PlatformError),
    Transaction(TransactionError),
    Source(NativeCandidatePluginSourceError),
    Codex(CodexAdapterError),
    ClaudeCode(ClaudeAdapterError),
    InstalledBinaryInvalid,
    ReceiptClosureInvalid,
    Candidate(NativeReleaseCandidateError),
    RecoveryRequired,
    CompensationFailed,
}

impl NativeCandidateLocalInstallError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::Platform(error) => error.reason_code(),
            Self::Transaction(error) => error.reason_code(),
            Self::Source(error) => error.reason_code(),
            Self::Codex(error) => error.reason_code(),
            Self::ClaudeCode(error) => error.reason_code(),
            Self::InstalledBinaryInvalid => "native-candidate-installed-binary-invalid",
            Self::Candidate(error) => error.reason_code(),
            Self::ReceiptClosureInvalid => "native-candidate-local-receipt-closure-invalid",
            Self::RecoveryRequired => "native-candidate-install-recovery-required",
            Self::CompensationFailed => "native-candidate-install-compensation-failed",
        }
    }
}

impl Display for NativeCandidateLocalInstallError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeCandidateLocalInstallError {}

/// Applies one verified candidate to its fixed local-client integration.
///
/// The home is a trusted product boundary. The managed payload root, source
/// path, installed binary path, plans, and inverse operations are all fixed and
/// derived internally.
pub fn apply_native_release_candidate_local(
    pack: &LoadedResourcePack<'_>,
    candidate: &VerifiedNativeReleaseCandidate,
    home: impl AsRef<Path>,
    now_unix: u64,
) -> Result<NativeCandidateLocalInstallCommit, NativeCandidateLocalInstallError> {
    let home = home.as_ref();
    let (managed_root, payload) = apply_candidate_payload(pack, candidate, home, now_unix)?;
    let payload_fresh = payload.disposition == InstallDisposition::Applied;
    let payload_executor = ManagedNativePayloadExecutor::new(managed_root.clone());

    let installed_binary = managed_root
        .path()
        .join(
            native_artifact_id(&candidate.candidate().artifact)
                .map_err(|_| NativeCandidateLocalInstallError::InstalledBinaryInvalid)?,
        )
        .join(
            native_artifact_binary_path(&candidate.candidate().artifact)
                .map_err(|_| NativeCandidateLocalInstallError::InstalledBinaryInvalid)?,
        );
    let source_target =
        match prepare_native_candidate_plugin_source_target(home, candidate.target()) {
            Ok(target) => target,
            Err(error) => {
                compensate_payload(
                    &payload_executor,
                    &payload.receipt.install_id,
                    pack,
                    payload_fresh,
                    now_unix,
                )?;
                return Err(NativeCandidateLocalInstallError::Source(error));
            }
        };
    let source = match materialize_native_candidate_plugin_source(
        pack,
        candidate,
        &installed_binary,
        &source_target,
    ) {
        Ok(commit) => commit,
        Err(error) if source_commit_may_be_ambiguous(error) => {
            return Err(NativeCandidateLocalInstallError::RecoveryRequired);
        }
        Err(error) => {
            compensate_payload(
                &payload_executor,
                &payload.receipt.install_id,
                pack,
                payload_fresh,
                now_unix,
            )?;
            return Err(NativeCandidateLocalInstallError::Source(error));
        }
    };
    let source_fresh = source.disposition == NativeCandidatePluginSourceDisposition::Materialized;
    let compensation = CandidateCompensation {
        source_target: &source_target,
        source_fresh,
        payload_executor: &payload_executor,
        install_id: &payload.receipt.install_id,
        pack,
        payload_fresh,
        now_unix,
    };

    let registration = match apply_registration(candidate, home, now_unix) {
        Ok(commit) => commit,
        Err(error) if registration_error_requires_recovery(error) => {
            return Err(NativeCandidateLocalInstallError::RecoveryRequired);
        }
        Err(error) => {
            compensate_source_and_payload(&compensation)?;
            return Err(error);
        }
    };

    if verify_native_release_candidate_local(
        pack,
        home,
        candidate.target(),
        &payload.receipt.install_id,
    )
    .is_err()
    {
        compensate_registration_source_and_payload(&registration, home, &compensation)?;
        return Err(NativeCandidateLocalInstallError::RecoveryRequired);
    }

    if let Err(error) = persist_verified_candidate(&managed_root, candidate) {
        compensate_registration_source_and_payload(&registration, home, &compensation)?;
        return Err(error);
    }

    Ok(NativeCandidateLocalInstallCommit {
        target: candidate.target(),
        payload,
        source,
        registration,
        outstanding_host_action: HostAction::InstallOrEnablePlugin,
    })
}

/// Stages a verified version without changing Host integrations or the installed
/// command. Existing payload transactions own recovery and can roll back this
/// stage while a prior version and its integrations remain intact.
/// The trusted caller must obtain filesystem-write approval for the exact
/// candidate before entering this boundary; a release signature is not approval.
pub fn stage_native_release_candidate_local(
    pack: &LoadedResourcePack<'_>,
    candidate: &VerifiedNativeReleaseCandidate,
    home: impl AsRef<Path>,
    now_unix: u64,
) -> Result<NativePayloadInstallCommit, NativeCandidateLocalInstallError> {
    let (root, payload) = apply_candidate_payload(pack, candidate, home.as_ref(), now_unix)?;
    if let Err(error) = persist_verified_candidate(&root, candidate) {
        compensate_payload(
            &ManagedNativePayloadExecutor::new(root),
            &payload.receipt.install_id,
            pack,
            payload.disposition == InstallDisposition::Applied,
            now_unix,
        )?;
        return Err(error);
    }
    Ok(payload)
}

fn apply_candidate_payload(
    pack: &LoadedResourcePack<'_>,
    candidate: &VerifiedNativeReleaseCandidate,
    home: &Path,
    now_unix: u64,
) -> Result<(ApprovedManagedRoot, NativePayloadInstallCommit), NativeCandidateLocalInstallError> {
    let managed_root = prepare_native_candidate_managed_root(home)
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let candidate_path = managed_root.path().join(
        native_release_candidate_file_name(&candidate.candidate().artifact)
            .map_err(NativeCandidateLocalInstallError::Candidate)?,
    );
    let candidate_bytes = candidate
        .signed_candidate()
        .to_canonical_json()
        .map_err(NativeCandidateLocalInstallError::Candidate)?;
    check_candidate_record(&candidate_path, &candidate_bytes)
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let root = AllowedRootV1 {
        id: MANAGED_ROOT_ID.to_string(),
        root: SymbolicRoot::QiongliManagedData,
    };
    let payload_plan = preview_native_payload_install(
        plan_metadata(candidate, "payload", now_unix)?,
        candidate.portable_release(),
        target_descriptor(candidate),
        root,
    )
    .map_err(NativeCandidateLocalInstallError::Platform)?
    .verify_with_grant_capability(candidate.portable_release().launch_grant(), now_unix)
    .map_err(NativeCandidateLocalInstallError::Platform)?;
    let payload_approval = approve_install_plan(&payload_plan, &PAYLOAD_APPROVALS, now_unix)
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let payload_executor = ManagedNativePayloadExecutor::new(managed_root.clone());
    let payload = payload_executor
        .apply(
            &payload_plan,
            &payload_approval,
            pack,
            candidate.portable_release(),
            now_unix,
        )
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let payload_fresh = payload.disposition == InstallDisposition::Applied;
    if payload_executor
        .verify(&payload.receipt.install_id, pack)
        .is_err()
    {
        compensate_payload(
            &payload_executor,
            &payload.receipt.install_id,
            pack,
            payload_fresh,
            now_unix,
        )?;
        return Err(NativeCandidateLocalInstallError::RecoveryRequired);
    }

    Ok((managed_root, payload))
}

fn persist_verified_candidate(
    root: &ApprovedManagedRoot,
    candidate: &VerifiedNativeReleaseCandidate,
) -> Result<(), NativeCandidateLocalInstallError> {
    let path = root.path().join(
        native_release_candidate_file_name(&candidate.candidate().artifact)
            .map_err(NativeCandidateLocalInstallError::Candidate)?,
    );
    let bytes = candidate
        .signed_candidate()
        .to_canonical_json()
        .map_err(NativeCandidateLocalInstallError::Candidate)?;
    persist_candidate_record(root, &path, &bytes)
        .map_err(NativeCandidateLocalInstallError::Transaction)
}

// Immutable signed metadata is retained with lifecycle receipts after uninstall.
// It grants nothing without fresh signature, active payload and executable checks.
fn check_candidate_record(path: &Path, bytes: &[u8]) -> Result<bool, TransactionError> {
    if !path_exists(path)? {
        return Ok(false);
    }
    if read_private_file(path)? != bytes {
        return Err(TransactionError::InvalidReceipt);
    }
    Ok(true)
}

fn persist_candidate_record(
    root: &ApprovedManagedRoot,
    path: &Path,
    bytes: &[u8],
) -> Result<(), TransactionError> {
    root.validate()?;
    if check_candidate_record(path, bytes)? {
        return Ok(());
    }
    let temporary = root
        .path()
        .join(format!(".qiongli-candidate-{}.tmp", transaction_id()));
    let mut file = create_private_new_file(&temporary)?;
    let result = write_sync_file(&mut file, bytes);
    drop(file);
    let result = result.and_then(|()| rename_path(&temporary, path, false));
    // A concurrent writer may have committed the same immutable candidate.
    let result = match result {
        Err(_) if check_candidate_record(path, bytes).unwrap_or(false) => Ok(()),
        other => other,
    };
    let _ = fs::remove_file(&temporary);
    result?;
    sync_directory(root.path())
}

/// Revalidates the installed product using fixed home-relative paths and the
/// trusted caller's current_exe(), embedded identity, authority and current time.
/// Receipt integrity alone cannot create this capability. This does not approve writes.
pub fn verify_installed_native_candidate_product(
    pack: &LoadedResourcePack<'_>,
    authority: &NativeReleaseAuthority,
    context: &NativeReleaseCandidateVerificationContext<'_>,
    home: impl AsRef<Path>,
    current_executable: &Path,
) -> Result<VerifiedInstalledNativeCandidate, NativeCandidateLocalInstallError> {
    let root = discover_native_candidate_managed_root(home)
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let candidate_path = root.path().join(
        native_release_candidate_file_name(context.expected_artifact)
            .map_err(NativeCandidateLocalInstallError::Candidate)?,
    );
    let bytes = read_private_file(&candidate_path)
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let signed = SignedNativeReleaseCandidateV1::from_json(&bytes)
        .map_err(NativeCandidateLocalInstallError::Candidate)?;
    let install_id = format!(
        "native-payload-{}",
        signed
            .candidate
            .signed_portable_release
            .envelope
            .archive_sha256
    );
    let receipt = ManagedNativePayloadExecutor::new(root.clone())
        .verify(&install_id, pack)
        .map_err(NativeCandidateLocalInstallError::Transaction)?
        .receipt;
    let target = approve_native_artifact_target(
        root.path().join(
            native_artifact_id(context.expected_artifact)
                .map_err(|_| NativeCandidateLocalInstallError::InstalledBinaryInvalid)?,
        ),
        context.expected_artifact,
    )
    .map_err(|_| NativeCandidateLocalInstallError::InstalledBinaryInvalid)?;
    let installed = signed
        .verify_installed(authority, context, pack, &target, current_executable)
        .map_err(NativeCandidateLocalInstallError::Candidate)?;
    if receipt.artifact != installed.candidate().artifact
        || receipt.operation.release_envelope_sha256
            != crate::grant::sha256_hex(
                &crate::native_release_envelope_signing_bytes(
                    &signed.candidate.signed_portable_release.envelope,
                )
                .map_err(|_| NativeCandidateLocalInstallError::ReceiptClosureInvalid)?,
            )
    {
        return Err(NativeCandidateLocalInstallError::ReceiptClosureInvalid);
    }
    Ok(installed)
}

/// Creates and approves the one product-owned native payload root below the
/// current-user home. No caller-selected root is accepted.
pub fn prepare_native_candidate_managed_root(
    home: impl AsRef<Path>,
) -> Result<ApprovedManagedRoot, TransactionError> {
    let home = home.as_ref();
    validate_candidate_home(home)?;
    let qiongli_root = home.join(".qiongli");
    ensure_candidate_directory(&qiongli_root, false)?;
    let native_root = qiongli_root.join("native");
    ensure_candidate_directory(&native_root, true)?;
    let managed_root = native_root.join("payloads");
    ensure_candidate_directory(&managed_root, true)?;
    approve_managed_root(&candidate_allowed_root(), managed_root)
}

/// Re-opens the fixed product-owned payload root without creating any path.
pub fn discover_native_candidate_managed_root(
    home: impl AsRef<Path>,
) -> Result<ApprovedManagedRoot, TransactionError> {
    let home = home.as_ref();
    validate_candidate_home(home)?;
    approve_managed_root(&candidate_allowed_root(), candidate_managed_root(home))
}

/// Verifies the complete receipt closure for one installed local integration.
/// This operation requires neither candidate bytes nor an unexpired release.
pub fn verify_native_release_candidate_local(
    pack: &LoadedResourcePack<'_>,
    home: impl AsRef<Path>,
    target: ClientActivationTarget,
    install_id: &str,
) -> Result<NativeCandidateLocalVerification, NativeCandidateLocalInstallError> {
    let home = home.as_ref();
    let managed_root = discover_native_candidate_managed_root(home)
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let payload = ManagedNativePayloadExecutor::new(managed_root)
        .verify(install_id, pack)
        .map_err(NativeCandidateLocalInstallError::Transaction)?;
    let source_target = discover_native_candidate_plugin_source_target(home, target)
        .map_err(NativeCandidateLocalInstallError::Source)?;
    let source = verify_native_candidate_plugin_source(&source_target)
        .map_err(NativeCandidateLocalInstallError::Source)?;
    let registration = verify_registration(target, home)?;
    validate_receipt_closure(target, &payload, &source, &registration)?;
    Ok(NativeCandidateLocalVerification {
        target,
        payload,
        source,
        registration,
        outstanding_host_action: HostAction::InstallOrEnablePlugin,
    })
}

/// Removes a complete receipt-verified local integration in reverse order.
/// Candidate validity and release inputs are deliberately not consulted.
pub fn remove_native_release_candidate_local(
    pack: &LoadedResourcePack<'_>,
    home: impl AsRef<Path>,
    target: ClientActivationTarget,
    install_id: &str,
    now_unix: u64,
) -> Result<NativeCandidateLocalRemoveCommit, NativeCandidateLocalInstallError> {
    let home = home.as_ref();
    verify_native_release_candidate_local(pack, home, target, install_id)?;
    let registration = remove_registration(target, home, now_unix)?;
    let source_target = discover_native_candidate_plugin_source_target(home, target)
        .map_err(|_| NativeCandidateLocalInstallError::RecoveryRequired)?;
    let source = remove_native_candidate_plugin_source(&source_target)
        .map_err(|_| NativeCandidateLocalInstallError::RecoveryRequired)?;
    let managed_root = discover_native_candidate_managed_root(home)
        .map_err(|_| NativeCandidateLocalInstallError::RecoveryRequired)?;
    let payload = ManagedNativePayloadExecutor::new(managed_root)
        .remove(install_id, pack, now_unix)
        .map_err(|_| NativeCandidateLocalInstallError::RecoveryRequired)?;
    Ok(NativeCandidateLocalRemoveCommit {
        target,
        registration,
        source,
        payload,
    })
}

fn apply_registration(
    candidate: &VerifiedNativeReleaseCandidate,
    home: &Path,
    now_unix: u64,
) -> Result<NativeCandidateRegistrationCommit, NativeCandidateLocalInstallError> {
    match candidate.target() {
        ClientActivationTarget::Codex => {
            let discovered =
                discover_codex_user(home).map_err(NativeCandidateLocalInstallError::Codex)?;
            let plan = preview_codex_registration(
                &discovered,
                plan_metadata(candidate, "codex", now_unix)?,
                candidate.plugin_grant(),
            )
            .map_err(NativeCandidateLocalInstallError::Codex)?
            .plan
            .verify_with_grant_capability(candidate.plugin_grant(), now_unix)
            .map_err(NativeCandidateLocalInstallError::Platform)?;
            let approval = approve_install_plan(&plan, &REGISTRATION_APPROVALS, now_unix)
                .map_err(NativeCandidateLocalInstallError::Transaction)?;
            CodexRegistrationExecutor::new(discovered)
                .apply(&plan, &approval, now_unix)
                .map(NativeCandidateRegistrationCommit::Codex)
                .map_err(NativeCandidateLocalInstallError::Codex)
        }
        ClientActivationTarget::ClaudeCode => {
            let discovered =
                discover_claude_user(home).map_err(NativeCandidateLocalInstallError::ClaudeCode)?;
            let plan = preview_claude_registration(
                &discovered,
                plan_metadata(candidate, "claude-code", now_unix)?,
                candidate.plugin_grant(),
            )
            .map_err(NativeCandidateLocalInstallError::ClaudeCode)?
            .plan
            .verify_with_grant_capability(candidate.plugin_grant(), now_unix)
            .map_err(NativeCandidateLocalInstallError::Platform)?;
            let approval = approve_install_plan(&plan, &REGISTRATION_APPROVALS, now_unix)
                .map_err(NativeCandidateLocalInstallError::Transaction)?;
            ClaudeRegistrationExecutor::new(discovered)
                .apply(&plan, &approval, now_unix)
                .map(NativeCandidateRegistrationCommit::ClaudeCode)
                .map_err(NativeCandidateLocalInstallError::ClaudeCode)
        }
    }
}

fn verify_registration(
    target: ClientActivationTarget,
    home: &Path,
) -> Result<NativeCandidateRegistrationVerification, NativeCandidateLocalInstallError> {
    match target {
        ClientActivationTarget::Codex => {
            let target =
                discover_codex_user(home).map_err(NativeCandidateLocalInstallError::Codex)?;
            CodexRegistrationExecutor::new(target)
                .verify()
                .map(NativeCandidateRegistrationVerification::Codex)
                .map_err(NativeCandidateLocalInstallError::Codex)
        }
        ClientActivationTarget::ClaudeCode => {
            let target =
                discover_claude_user(home).map_err(NativeCandidateLocalInstallError::ClaudeCode)?;
            ClaudeRegistrationExecutor::new(target)
                .verify()
                .map(NativeCandidateRegistrationVerification::ClaudeCode)
                .map_err(NativeCandidateLocalInstallError::ClaudeCode)
        }
    }
}

fn remove_registration(
    target: ClientActivationTarget,
    home: &Path,
    now_unix: u64,
) -> Result<NativeCandidateRegistrationLifecycleCommit, NativeCandidateLocalInstallError> {
    match target {
        ClientActivationTarget::Codex => {
            let target =
                discover_codex_user(home).map_err(NativeCandidateLocalInstallError::Codex)?;
            CodexRegistrationExecutor::new(target)
                .remove(now_unix)
                .map(NativeCandidateRegistrationLifecycleCommit::Codex)
                .map_err(NativeCandidateLocalInstallError::Codex)
        }
        ClientActivationTarget::ClaudeCode => {
            let target =
                discover_claude_user(home).map_err(NativeCandidateLocalInstallError::ClaudeCode)?;
            ClaudeRegistrationExecutor::new(target)
                .remove(now_unix)
                .map(NativeCandidateRegistrationLifecycleCommit::ClaudeCode)
                .map_err(NativeCandidateLocalInstallError::ClaudeCode)
        }
    }
}

fn validate_receipt_closure(
    target: ClientActivationTarget,
    payload: &NativePayloadInstallVerification,
    source: &NativeCandidatePluginSourceVerification,
    registration: &NativeCandidateRegistrationVerification,
) -> Result<(), NativeCandidateLocalInstallError> {
    let (expected_family, expected_surface) = match target {
        ClientActivationTarget::Codex => {
            (LocalTargetFamily::CodexLocal, LocalSurface::DesktopLocal)
        }
        ClientActivationTarget::ClaudeCode => {
            (LocalTargetFamily::ClaudeCodeLocal, LocalSurface::CliLocal)
        }
    };
    let mut expected_portable_artifact = source.artifact.clone();
    expected_portable_artifact.installer_kind = InstallerKind::PortableArchive;
    let payload_receipt = &payload.receipt;
    if source.target != target
        || source.artifact.installer_kind != InstallerKind::PluginBundle
        || expected_portable_artifact != payload_receipt.artifact
        || source.binary_sha256 != payload_receipt.operation.binary_sha256
        || source.resource_pack_sha256 != payload_receipt.operation.pack_sha256
        || payload_receipt.target.family != expected_family
        || payload_receipt.target.surface != expected_surface
        || payload_receipt.target.scope != InstallScope::User
        || payload_receipt.target.profile != CapabilityProfile::Lite
        || payload_receipt.target.adapter_version != 1
    {
        return Err(NativeCandidateLocalInstallError::ReceiptClosureInvalid);
    }
    let registration_matches = match registration {
        NativeCandidateRegistrationVerification::Codex(verification) => {
            target == ClientActivationTarget::Codex
                && registration_receipt_matches(
                    &verification.receipt.artifact,
                    &verification.receipt.target,
                    &verification.receipt.ownership.artifact_digest_sha256,
                    &verification.receipt.source_receipt_sha256,
                    &verification.receipt.source_content_root_sha256,
                    verification.receipt.outstanding_host_action,
                    source,
                    expected_family,
                    expected_surface,
                )
        }
        NativeCandidateRegistrationVerification::ClaudeCode(verification) => {
            target == ClientActivationTarget::ClaudeCode
                && registration_receipt_matches(
                    &verification.receipt.artifact,
                    &verification.receipt.target,
                    &verification.receipt.ownership.artifact_digest_sha256,
                    &verification.receipt.source_receipt_sha256,
                    &verification.receipt.source_content_root_sha256,
                    verification.receipt.outstanding_host_action,
                    source,
                    expected_family,
                    expected_surface,
                )
        }
    };
    if registration_matches {
        Ok(())
    } else {
        Err(NativeCandidateLocalInstallError::ReceiptClosureInvalid)
    }
}

#[allow(clippy::too_many_arguments)]
fn registration_receipt_matches(
    artifact: &crate::ArtifactIdentityV1,
    descriptor: &TargetDescriptorV1,
    grant_digest: &str,
    source_receipt_sha256: &str,
    source_content_root_sha256: &str,
    host_action: HostAction,
    source: &NativeCandidatePluginSourceVerification,
    expected_family: LocalTargetFamily,
    expected_surface: LocalSurface,
) -> bool {
    artifact == &source.artifact
        && descriptor.family == expected_family
        && descriptor.surface == expected_surface
        && descriptor.scope == InstallScope::User
        && descriptor.profile == CapabilityProfile::Lite
        && descriptor.os == source.artifact.os
        && descriptor.arch == source.artifact.arch
        && descriptor.adapter_version == 1
        && grant_digest == source.signed_grant_payload_sha256
        && source_receipt_sha256 == source.receipt_sha256
        && source_content_root_sha256 == source.package_content_root_sha256
        && host_action == HostAction::InstallOrEnablePlugin
}

struct CandidateCompensation<'a, 'pack> {
    source_target: &'a NativeCandidatePluginSourceTarget,
    source_fresh: bool,
    payload_executor: &'a ManagedNativePayloadExecutor,
    install_id: &'a str,
    pack: &'a LoadedResourcePack<'pack>,
    payload_fresh: bool,
    now_unix: u64,
}

fn compensate_registration_source_and_payload(
    registration: &NativeCandidateRegistrationCommit,
    home: &Path,
    compensation: &CandidateCompensation<'_, '_>,
) -> Result<(), NativeCandidateLocalInstallError> {
    if registration.was_fresh() {
        let rollback_failed = match registration {
            NativeCandidateRegistrationCommit::Codex(_) => discover_codex_user(home)
                .and_then(|target| {
                    CodexRegistrationExecutor::new(target).rollback(compensation.now_unix)
                })
                .is_err(),
            NativeCandidateRegistrationCommit::ClaudeCode(_) => discover_claude_user(home)
                .and_then(|target| {
                    ClaudeRegistrationExecutor::new(target).rollback(compensation.now_unix)
                })
                .is_err(),
        };
        if rollback_failed {
            return Err(NativeCandidateLocalInstallError::CompensationFailed);
        }
    }
    compensate_source_and_payload(compensation)
}

fn compensate_source_and_payload(
    compensation: &CandidateCompensation<'_, '_>,
) -> Result<(), NativeCandidateLocalInstallError> {
    if compensation.source_fresh
        && remove_native_candidate_plugin_source(compensation.source_target).is_err()
    {
        return Err(NativeCandidateLocalInstallError::CompensationFailed);
    }
    compensate_payload(
        compensation.payload_executor,
        compensation.install_id,
        compensation.pack,
        compensation.payload_fresh,
        compensation.now_unix,
    )
}

fn compensate_payload(
    executor: &ManagedNativePayloadExecutor,
    install_id: &str,
    pack: &LoadedResourcePack<'_>,
    fresh: bool,
    now_unix: u64,
) -> Result<(), NativeCandidateLocalInstallError> {
    if fresh && executor.rollback(install_id, pack, now_unix).is_err() {
        return Err(NativeCandidateLocalInstallError::CompensationFailed);
    }
    Ok(())
}

fn target_descriptor(candidate: &VerifiedNativeReleaseCandidate) -> TargetDescriptorV1 {
    let artifact = &candidate.candidate().artifact;
    let (family, surface) = match candidate.target() {
        ClientActivationTarget::Codex => {
            (LocalTargetFamily::CodexLocal, LocalSurface::DesktopLocal)
        }
        ClientActivationTarget::ClaudeCode => {
            (LocalTargetFamily::ClaudeCodeLocal, LocalSurface::CliLocal)
        }
    };
    TargetDescriptorV1 {
        family,
        surface,
        scope: InstallScope::User,
        profile: CapabilityProfile::Lite,
        os: artifact.os,
        arch: artifact.arch,
        adapter_version: 1,
    }
}

fn plan_metadata(
    candidate: &VerifiedNativeReleaseCandidate,
    phase: &str,
    now_unix: u64,
) -> Result<InstallPlanMetadataV1, NativeCandidateLocalInstallError> {
    let prefix = candidate
        .signed_payload_sha256()
        .get(..24)
        .ok_or(NativeCandidateLocalInstallError::InstalledBinaryInvalid)?;
    let target = match candidate.target() {
        ClientActivationTarget::Codex => "codex",
        ClientActivationTarget::ClaudeCode => "claude-code",
    };
    let expires_at_unix = now_unix
        .saturating_add(PLAN_TTL_SECONDS)
        .min(candidate.candidate().expires_at_unix);
    if expires_at_unix <= now_unix {
        return Err(NativeCandidateLocalInstallError::Platform(
            PlatformError::InstallPlanExpired,
        ));
    }
    Ok(InstallPlanMetadataV1 {
        plan_id: format!("candidate-{target}-{phase}-{prefix}"),
        created_at_unix: now_unix,
        expires_at_unix,
    })
}

fn candidate_allowed_root() -> AllowedRootV1 {
    AllowedRootV1 {
        id: MANAGED_ROOT_ID.to_string(),
        root: SymbolicRoot::QiongliManagedData,
    }
}

fn candidate_managed_root(home: &Path) -> std::path::PathBuf {
    home.join(".qiongli/native/payloads")
}

fn validate_candidate_home(path: &Path) -> Result<(), TransactionError> {
    if !path.is_absolute() || has_lexical_traversal(path) {
        return Err(TransactionError::UnsafeManagedRoot);
    }
    validate_candidate_directory(path, false)
}

fn ensure_candidate_directory(path: &Path, private: bool) -> Result<(), TransactionError> {
    match fs::symlink_metadata(path) {
        Ok(_) => validate_candidate_directory(path, private),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            create_candidate_private_directory(path)?;
            validate_candidate_directory(path, true)
        }
        Err(error) => Err(TransactionError::PersistenceFailed(error.kind())),
    }
}

fn validate_candidate_directory(path: &Path, private: bool) -> Result<(), TransactionError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            TransactionError::UnsafeManagedRoot
        } else {
            TransactionError::PersistenceFailed(error.kind())
        }
    })?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_dir() {
        return Err(TransactionError::UnsafeManagedRoot);
    }
    validate_candidate_directory_security(path, &metadata, private)
}

#[cfg(unix)]
fn validate_candidate_directory_security(
    _path: &Path,
    metadata: &Metadata,
    private: bool,
) -> Result<(), TransactionError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let mode = metadata.permissions().mode();
    if metadata.uid() != rustix::process::geteuid().as_raw()
        || mode & 0o022 != 0
        || (private && mode & 0o077 != 0)
    {
        return Err(TransactionError::UnsafeManagedRoot);
    }
    Ok(())
}

#[cfg(windows)]
fn validate_candidate_directory_security(
    path: &Path,
    _metadata: &Metadata,
    private: bool,
) -> Result<(), TransactionError> {
    let result = if private {
        qiongli_windows_security::open_owner_only_directory(path)
    } else {
        qiongli_windows_security::open_directory_no_reparse(path)
    };
    result
        .map(|_| ())
        .map_err(|_| TransactionError::UnsafeManagedRoot)
}

#[cfg(not(any(unix, windows)))]
fn validate_candidate_directory_security(
    _path: &Path,
    _metadata: &Metadata,
    _private: bool,
) -> Result<(), TransactionError> {
    Err(TransactionError::UnsupportedPlatform)
}

#[cfg(unix)]
fn create_candidate_private_directory(path: &Path) -> Result<(), TransactionError> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn create_candidate_private_directory(path: &Path) -> Result<(), TransactionError> {
    qiongli_windows_security::create_owner_only_directory(path)
        .map(|_| ())
        .map_err(|error| {
            TransactionError::PersistenceFailed(
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn create_candidate_private_directory(_path: &Path) -> Result<(), TransactionError> {
    Err(TransactionError::UnsupportedPlatform)
}

#[cfg(unix)]
fn has_lexical_traversal(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;

    path.as_os_str()
        .as_bytes()
        .split(|byte| *byte == b'/')
        .any(|component| component == b"." || component == b"..")
}

#[cfg(windows)]
fn has_lexical_traversal(path: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;

    path.as_os_str()
        .encode_wide()
        .collect::<Vec<_>>()
        .split(|unit| matches!(*unit, 47 | 92))
        .any(|component| component == [46] || component == [46, 46])
}

#[cfg(not(any(unix, windows)))]
fn has_lexical_traversal(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

const fn source_commit_may_be_ambiguous(error: NativeCandidatePluginSourceError) -> bool {
    matches!(
        error,
        NativeCandidatePluginSourceError::CodexBundle(
            crate::CodexPluginBundleError::CommitFailed(_)
                | crate::CodexPluginBundleError::CommittedPersistenceFailed(_)
                | crate::CodexPluginBundleError::CommittedVerificationFailed
        ) | NativeCandidatePluginSourceError::ClaudeBundle(
            crate::ClaudePluginBundleError::CommitFailed(_)
                | crate::ClaudePluginBundleError::CommittedPersistenceFailed(_)
                | crate::ClaudePluginBundleError::CommittedVerificationFailed
        )
    )
}

const fn registration_error_requires_recovery(error: NativeCandidateLocalInstallError) -> bool {
    matches!(
        error,
        NativeCandidateLocalInstallError::Codex(
            CodexAdapterError::RecoveryRequired | CodexAdapterError::RollbackFailed
        ) | NativeCandidateLocalInstallError::ClaudeCode(
            ClaudeAdapterError::RecoveryRequired | ClaudeAdapterError::RollbackFailed
        )
    )
}
