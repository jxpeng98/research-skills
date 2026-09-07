use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::EmbeddedContent;
use qiongli_platform::{
    ApprovalRequirement, CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH, CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH,
    ClientActivationTarget, HostAction, InstallDisposition, LifecycleDisposition,
    MAX_NATIVE_RELEASE_CANDIDATE_BYTES, MAX_NATIVE_RELEASE_NOTES_BYTES,
    NATIVE_CANDIDATE_MANAGED_ROOT_SYMBOLIC_PATH, NativeCandidateLocalInstallCommit,
    NativeCandidateLocalRemoveCommit, NativeCandidateLocalVerification,
    NativeCandidatePluginSourceDisposition, NativeCandidateRegistrationCommit,
    NativeCandidateRegistrationLifecycleCommit, NativeCandidateRegistrationVerification,
    NativeReleaseAuthority, NativeReleaseCandidateVerificationContext,
    SignedNativeReleaseCandidateV1, VerifiedNativeReleaseCandidate,
    apply_native_release_candidate_local, approve_native_portable_archive_target,
    current_target_native_artifact_identity, native_payload_install_id,
    native_portable_archive_file_name, native_release_candidate_file_name,
    native_release_notes_file_name, remove_native_release_candidate_local,
    verify_native_release_candidate_local,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const OUTPUT_SCHEMA_VERSION: u32 = 1;
const APPROVAL_DOMAIN: &[u8] = b"QIONGLI-NATIVE-CANDIDATE-APPROVAL-V1\0";
const APPLY_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];
const REMOVE_APPROVALS: [ApprovalRequirement; 2] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CandidateReleaseOptions {
    pub candidate: PathBuf,
    pub archive: PathBuf,
    pub release_notes: PathBuf,
    pub target: ClientActivationTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CandidateReceiptOptions {
    pub target: ClientActivationTarget,
    pub install_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CandidateCliCommand {
    ActivatePreview {
        options: CandidateReleaseOptions,
        previous_install_id: String,
    },
    StagePreview(CandidateReleaseOptions),
    Stage {
        options: CandidateReleaseOptions,
        expected_approval_digest: String,
    },
    Preview(CandidateReleaseOptions),
    Apply {
        options: CandidateReleaseOptions,
        expected_approval_digest: String,
    },
    Verify(CandidateReceiptOptions),
    Remove(CandidateReceiptOptions),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum CandidateCliOutput {
    ActivationPreview(CandidateActivationPreviewOutput),
    Stage(CandidateStageOutput),
    Preview(CandidatePreviewOutput),
    Apply(CandidateApplyOutput),
    Verify(CandidateVerifyOutput),
    Remove(CandidateRemoveOutput),
}

/// Runs the installed command with an empty PATH and checks its verified native
/// product plan against the already verified candidate. This does not approve writes.
pub fn check_native_cli_health(
    home: &Path,
    config_root: &Path,
    candidate: &VerifiedNativeReleaseCandidate,
) -> Result<(), &'static str> {
    if !home.is_absolute() || !config_root.is_absolute() {
        return Err("native-activation-health-root-invalid");
    }
    let executable = crate::cli_install::cli_target(home);
    let release = &candidate.candidate().signed_portable_release.envelope;
    let verify_binary = || {
        if crate::cli_install::regular_file_sha256(&executable)? != release.binary_sha256 {
            return Err("native-activation-health-binary-mismatch");
        }
        Ok(())
    };
    verify_binary()?;
    let output = crate::desktop::native_cli_health_output(home, config_root, &executable)?;
    crate::managed_operation::verify_native_cli_health_plan(
        &output,
        &candidate.candidate().artifact.version,
        &release.resource_pack_sha256,
        candidate.signed_payload_sha256(),
    )?;
    verify_binary()
}

pub(crate) fn execute(
    command: CandidateCliCommand,
    authority: Option<&NativeReleaseAuthority>,
    expected_source_commit: Option<&str>,
    home: Option<&Path>,
    content: &EmbeddedContent,
) -> Result<CandidateCliOutput, &'static str> {
    match command {
        CandidateCliCommand::ActivatePreview {
            options,
            previous_install_id,
        } => {
            let now = now_unix()?;
            let authority = require_authority(authority)?;
            let source = require_source_commit(expected_source_commit)?;
            let prepared = prepare_candidate(&options, authority, source, content, now)?;
            let home = home.ok_or("native-candidate-home-unavailable")?;
            let executable =
                std::env::current_exe().map_err(|_| "native-activation-process-unavailable")?;
            let product = qiongli_platform::verify_native_packaged_product(
                content.pack(),
                authority,
                home,
                &executable,
                &prepared.verified.candidate().artifact.version,
                source,
                now,
            )
            .map_err(|error| error.reason_code())?;
            Ok(CandidateCliOutput::ActivationPreview(
                preview_native_candidate_activation(
                    &product,
                    &prepared.verified,
                    &previous_install_id,
                )?,
            ))
        }
        CandidateCliCommand::StagePreview(options) => {
            let prepared = prepare_candidate(
                &options,
                require_authority(authority)?,
                require_source_commit(expected_source_commit)?,
                content,
                now_unix()?,
            )?;
            Ok(CandidateCliOutput::Stage(stage_output(
                &prepared,
                StageCommand::Preview,
                StageState::Ready,
            )))
        }
        CandidateCliCommand::Stage {
            options,
            expected_approval_digest,
        } => {
            let now = now_unix()?;
            let prepared = prepare_candidate(
                &options,
                require_authority(authority)?,
                require_source_commit(expected_source_commit)?,
                content,
                now,
            )?;
            if stage_approval_digest(&prepared.approval_digest_sha256) != expected_approval_digest {
                return Err("native-candidate-stage-approval-digest-mismatch");
            }
            let commit = qiongli_platform::stage_native_release_candidate_local(
                content.pack(),
                &prepared.verified,
                home.ok_or("native-candidate-home-unavailable")?,
                now,
            )
            .map_err(|error| error.reason_code())?;
            let state = if commit.disposition == InstallDisposition::Applied {
                StageState::Staged
            } else {
                StageState::AlreadyStaged
            };
            Ok(CandidateCliOutput::Stage(stage_output(
                &prepared,
                StageCommand::Stage,
                state,
            )))
        }
        CandidateCliCommand::Preview(options) => {
            let prepared = prepare_candidate(
                &options,
                require_authority(authority)?,
                require_source_commit(expected_source_commit)?,
                content,
                now_unix()?,
            )?;
            Ok(CandidateCliOutput::Preview(prepared.preview_output()))
        }
        CandidateCliCommand::Apply {
            options,
            expected_approval_digest,
        } => {
            let now_unix = now_unix()?;
            let prepared = prepare_candidate(
                &options,
                require_authority(authority)?,
                require_source_commit(expected_source_commit)?,
                content,
                now_unix,
            )?;
            if prepared.approval_digest_sha256 != expected_approval_digest {
                return Err("native-candidate-approval-digest-mismatch");
            }
            let home = home.ok_or("native-candidate-home-unavailable")?;
            let commit = apply_native_release_candidate_local(
                content.pack(),
                &prepared.verified,
                home,
                now_unix,
            )
            .map_err(|error| error.reason_code())?;
            Ok(CandidateCliOutput::Apply(apply_output(&prepared, commit)))
        }
        CandidateCliCommand::Verify(options) => {
            let home = home.ok_or("native-candidate-home-unavailable")?;
            let verification = verify_native_release_candidate_local(
                content.pack(),
                home,
                options.target,
                &options.install_id,
            )
            .map_err(|error| error.reason_code())?;
            Ok(CandidateCliOutput::Verify(verify_output(verification)))
        }
        CandidateCliCommand::Remove(options) => {
            let home = home.ok_or("native-candidate-home-unavailable")?;
            let commit = remove_native_release_candidate_local(
                content.pack(),
                home,
                options.target,
                &options.install_id,
                now_unix()?,
            )
            .map_err(|error| error.reason_code())?;
            Ok(CandidateCliOutput::Remove(remove_output(commit)))
        }
    }
}

#[derive(Debug, Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct CandidateStageOutput {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    command: StageCommand,
    state: StageState,
    #[schemars(regex(pattern = "^(codex|claude-code)$"))]
    target: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    candidate_digest_sha256: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    approval_digest_sha256: String,
    #[schemars(regex(pattern = "^native-payload-[0-9a-f]{64}$"))]
    install_id: String,
    approvals_required: [StageApproval; 1],
}

#[derive(Debug, Serialize, serde::Deserialize, schemars::JsonSchema)]
enum StageCommand {
    #[serde(rename = "install-candidate-stage-preview")]
    Preview,
    #[serde(rename = "install-candidate-stage")]
    Stage,
}
#[derive(Debug, Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum StageState {
    Ready,
    Staged,
    AlreadyStaged,
}
#[derive(Debug, Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum StageApproval {
    FilesystemWrite,
}

fn stage_approval_digest(install_digest: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(b"QIONGLI-CANDIDATE-STAGE-V1\0");
    hash.update(install_digest.as_bytes());
    encode_hex(&hash.finalize())
}

fn stage_output(
    prepared: &PreparedCandidate,
    command: StageCommand,
    state: StageState,
) -> CandidateStageOutput {
    CandidateStageOutput {
        schema_version: 1,
        command,
        state,
        target: match prepared.verified.target() {
            ClientActivationTarget::Codex => "codex",
            ClientActivationTarget::ClaudeCode => "claude-code",
        }
        .to_string(),
        candidate_digest_sha256: prepared.verified.signed_payload_sha256().to_string(),
        approval_digest_sha256: stage_approval_digest(&prepared.approval_digest_sha256),
        install_id: native_payload_install_id(prepared.verified.portable_release().archive()),
        approvals_required: [StageApproval::FilesystemWrite],
    }
}

pub fn candidate_stage_contract_json() -> Result<String, serde_json::Error> {
    let schema = schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<CandidateStageOutput>();
    let fixtures = [
        (StageCommand::Preview, StageState::Ready),
        (StageCommand::Stage, StageState::Staged),
        (StageCommand::Stage, StageState::AlreadyStaged),
    ]
    .into_iter()
    .map(|(command, state)| CandidateStageOutput {
        schema_version: 1,
        command,
        state,
        target: "codex".to_string(),
        candidate_digest_sha256: "1".repeat(64),
        approval_digest_sha256: stage_approval_digest(&"2".repeat(64)),
        install_id: format!("native-payload-{}", "3".repeat(64)),
        approvals_required: [StageApproval::FilesystemWrite],
    })
    .collect::<Vec<_>>();
    serde_json::to_string_pretty(&serde_json::json!({"schema": schema, "fixtures": fixtures}))
}

/// A read-only identity snapshot. This digest is not an activation approval.
#[derive(Debug, Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CandidateActivationPreviewOutput {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    command: ActivationPreviewCommand,
    #[schemars(regex(pattern = "^(codex|claude-code)$"))]
    target: String,
    previous_version: String,
    candidate_version: String,
    #[schemars(regex(pattern = "^native-payload-[0-9a-f]{64}$"))]
    previous_install_id: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    previous_pack_sha256: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    candidate_digest_sha256: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    cli_plan_sha256: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    preflight_digest_sha256: String,
    mutation: PreviewMutation,
}

#[derive(Debug, Serialize, serde::Deserialize, schemars::JsonSchema)]
enum ActivationPreviewCommand {
    #[serde(rename = "install-candidate-activate-preview")]
    Preview,
}
#[derive(Debug, Serialize, serde::Deserialize, schemars::JsonSchema)]
enum PreviewMutation {
    #[serde(rename = "none")]
    None,
}

/// The caller supplies freshly verified running-product and candidate authority.
/// Checks do not reserve a transaction or authorize activation; apply must revalidate.
pub fn preview_native_candidate_activation(
    product: &qiongli_platform::VerifiedPackagedProduct,
    candidate: &VerifiedNativeReleaseCandidate,
    previous_install_id: &str,
) -> Result<CandidateActivationPreviewOutput, &'static str> {
    if product.artifact() != &candidate.candidate().artifact
        || product.control_sha256() != candidate.signed_payload_sha256()
        || product.product_source_commit() != candidate.candidate().source_commit
        || product.resource_pack_sha256()
            != candidate
                .candidate()
                .signed_portable_release
                .envelope
                .resource_pack_sha256
    {
        return Err("native-activation-candidate-process-mismatch");
    }
    let previous = qiongli_platform::verify_receipt_owned_native_candidate_local(
        product.home(),
        candidate.target(),
        previous_install_id,
    )
    .map_err(|error| error.reason_code())?;
    let mut expected = previous.payload.receipt.artifact.clone();
    expected
        .version
        .clone_from(&candidate.candidate().artifact.version);
    if expected != candidate.candidate().artifact
        || semver::Version::parse(&candidate.candidate().artifact.version)
            .map_err(|_| "native-activation-version-invalid")?
            <= semver::Version::parse(&previous.payload.receipt.artifact.version)
                .map_err(|_| "native-activation-version-invalid")?
    {
        return Err("native-activation-predecessor-incompatible");
    }
    let home = product.home();
    let target = crate::cli_install::cli_target(home);
    let plan = crate::cli_install::preview_cli_install(
        home,
        product.current_executable(),
        &candidate.candidate().artifact.version,
    )?;
    crate::cli_install::installed_native_cli_source(
        home,
        &target,
        &previous.payload.receipt.artifact,
    )?
    .ok_or("native-activation-predecessor-cli-unavailable")?;
    crate::cli_install::verify_cli_install_plan(&plan)?;
    if plan.source_sha256
        != candidate
            .candidate()
            .signed_portable_release
            .envelope
            .binary_sha256
    {
        return Err("native-activation-candidate-process-mismatch");
    }
    let registration = match &previous.registration {
        NativeCandidateRegistrationVerification::Codex(value) => {
            serde_json::to_value(&value.receipt)
        }
        NativeCandidateRegistrationVerification::ClaudeCode(value) => {
            serde_json::to_value(&value.receipt)
        }
    }
    .map_err(|_| "native-activation-preview-serialization-failed")?;
    let bytes = serde_json::to_vec(&(
        "QIONGLI-NATIVE-ACTIVATION-PREFLIGHT-V1",
        candidate.signed_payload_sha256(),
        candidate.target(),
        plan.plan_sha256(),
        &previous.payload.receipt,
        &previous.source.receipt_sha256,
        registration,
    ))
    .map_err(|_| "native-activation-preview-serialization-failed")?;
    Ok(CandidateActivationPreviewOutput {
        schema_version: 1,
        command: ActivationPreviewCommand::Preview,
        target: match candidate.target() {
            ClientActivationTarget::Codex => "codex",
            ClientActivationTarget::ClaudeCode => "claude-code",
        }
        .to_string(),
        previous_version: previous.payload.receipt.artifact.version,
        candidate_version: candidate.candidate().artifact.version.clone(),
        previous_install_id: previous_install_id.to_string(),
        previous_pack_sha256: previous.payload.receipt.operation.pack_sha256,
        candidate_digest_sha256: candidate.signed_payload_sha256().to_string(),
        cli_plan_sha256: plan.plan_sha256().to_string(),
        preflight_digest_sha256: encode_hex(&Sha256::digest(bytes)),
        mutation: PreviewMutation::None,
    })
}

pub fn candidate_activation_preview_contract_json() -> Result<String, serde_json::Error> {
    let schema = schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<CandidateActivationPreviewOutput>();
    let fixture = CandidateActivationPreviewOutput {
        schema_version: 1,
        command: ActivationPreviewCommand::Preview,
        target: "codex".to_string(),
        previous_version: "2.0.0-alpha.5".to_string(),
        candidate_version: "2.0.0-alpha.6".to_string(),
        previous_install_id: format!("native-payload-{}", "1".repeat(64)),
        previous_pack_sha256: "2".repeat(64),
        candidate_digest_sha256: "3".repeat(64),
        cli_plan_sha256: "4".repeat(64),
        preflight_digest_sha256: "5".repeat(64),
        mutation: PreviewMutation::None,
    };
    serde_json::to_string_pretty(&serde_json::json!({"schema": schema, "fixture": fixture}))
}

pub(crate) struct PreparedCandidate {
    verified: VerifiedNativeReleaseCandidate,
    approval_digest_sha256: String,
}

impl PreparedCandidate {
    pub(crate) fn into_verified(self) -> VerifiedNativeReleaseCandidate {
        self.verified
    }

    fn preview_output(&self) -> CandidatePreviewOutput {
        CandidatePreviewOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-candidate-preview",
            target: self.verified.target(),
            artifact: self.verified.candidate().artifact.clone(),
            candidate_digest_sha256: self.verified.signed_payload_sha256().to_string(),
            approval_digest_sha256: self.approval_digest_sha256.clone(),
            release_key_id: self.verified.release_key_id().to_string(),
            install_id: native_payload_install_id(self.verified.portable_release().archive()),
            archive_sha256: self
                .verified
                .portable_release()
                .archive()
                .archive_sha256()
                .to_string(),
            release_notes_sha256: self.verified.candidate().release_notes.sha256.clone(),
            managed_root: NATIVE_CANDIDATE_MANAGED_ROOT_SYMBOLIC_PATH,
            plugin_source: plugin_source_symbolic_path(self.verified.target()),
            approvals_required: APPLY_APPROVALS,
            outstanding_host_action: HostAction::InstallOrEnablePlugin,
            mutation: "none",
        }
    }
}

pub(crate) fn prepare_candidate(
    options: &CandidateReleaseOptions,
    authority: &NativeReleaseAuthority,
    expected_source_commit: &str,
    content: &EmbeddedContent,
    now_unix: u64,
) -> Result<PreparedCandidate, &'static str> {
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), authority.channel())
            .map_err(|error| error.reason_code())?;
    let candidate_bytes = read_bounded(
        &options.candidate,
        MAX_NATIVE_RELEASE_CANDIDATE_BYTES,
        "native-release-candidate-read-failed",
        "native-release-candidate-too-large",
    )?;
    let signed = SignedNativeReleaseCandidateV1::from_json(&candidate_bytes)
        .map_err(|error| error.reason_code())?;
    validate_candidate_file_set(options, &signed)?;
    let archive_target = approve_native_portable_archive_target(&options.archive, &artifact)
        .map_err(|error| error.reason_code())?;
    let release_notes = read_bounded(
        &options.release_notes,
        MAX_NATIVE_RELEASE_NOTES_BYTES,
        "native-release-notes-read-failed",
        "native-release-notes-too-large",
    )?;
    let context = NativeReleaseCandidateVerificationContext {
        now_unix,
        expected_source_commit,
        expected_artifact: &artifact,
        requested_target: options.target,
    };
    let verified = signed
        .verify(
            authority,
            &context,
            content.pack(),
            &archive_target,
            &release_notes,
        )
        .map_err(|error| error.reason_code())?;
    let approval_digest_sha256 =
        candidate_approval_digest(verified.signed_payload_sha256(), options.target);
    Ok(PreparedCandidate {
        verified,
        approval_digest_sha256,
    })
}

fn validate_candidate_file_set(
    options: &CandidateReleaseOptions,
    signed: &SignedNativeReleaseCandidateV1,
) -> Result<(), &'static str> {
    let artifact = &signed.candidate.artifact;
    let candidate_name =
        native_release_candidate_file_name(artifact).map_err(|error| error.reason_code())?;
    let archive_name =
        native_portable_archive_file_name(artifact).map_err(|error| error.reason_code())?;
    let notes_name =
        native_release_notes_file_name(artifact).map_err(|error| error.reason_code())?;
    let parent = options
        .candidate
        .parent()
        .ok_or("native-candidate-file-set-invalid")?;
    if options
        .candidate
        .file_name()
        .and_then(|value| value.to_str())
        != Some(candidate_name.as_str())
        || options.archive.file_name().and_then(|value| value.to_str())
            != Some(archive_name.as_str())
        || options
            .release_notes
            .file_name()
            .and_then(|value| value.to_str())
            != Some(notes_name.as_str())
        || options.archive.parent() != Some(parent)
        || options.release_notes.parent() != Some(parent)
    {
        return Err("native-candidate-file-set-invalid");
    }
    Ok(())
}

fn read_bounded(
    path: &Path,
    max_bytes: usize,
    read_error: &'static str,
    too_large_error: &'static str,
) -> Result<Vec<u8>, &'static str> {
    let file = fs::File::open(path).map_err(|_| read_error)?;
    let limit = u64::try_from(max_bytes).map_err(|_| too_large_error)?;
    let mut bounded = file.take(limit.saturating_add(1));
    let mut bytes = Vec::new();
    bounded.read_to_end(&mut bytes).map_err(|_| read_error)?;
    if bytes.len() > max_bytes {
        return Err(too_large_error);
    }
    Ok(bytes)
}

pub(crate) fn candidate_approval_digest(
    candidate_digest_sha256: &str,
    target: ClientActivationTarget,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(APPROVAL_DOMAIN);
    hasher.update(candidate_digest_sha256.as_bytes());
    hasher.update([0]);
    hasher.update(match target {
        ClientActivationTarget::Codex => b"codex".as_slice(),
        ClientActivationTarget::ClaudeCode => b"claude-code".as_slice(),
    });
    encode_hex(&hasher.finalize())
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn apply_output(
    prepared: &PreparedCandidate,
    commit: NativeCandidateLocalInstallCommit,
) -> CandidateApplyOutput {
    let (registration_disposition, registration_transaction_id, registration_cleanup_required) =
        registration_commit_details(&commit.registration);
    CandidateApplyOutput {
        schema_version: OUTPUT_SCHEMA_VERSION,
        command: "install-candidate-apply",
        target: commit.target,
        artifact: commit.payload.receipt.artifact,
        candidate_digest_sha256: prepared.verified.signed_payload_sha256().to_string(),
        approval_digest_sha256: prepared.approval_digest_sha256.clone(),
        install_id: commit.payload.receipt.install_id,
        payload_disposition: install_disposition(commit.payload.disposition),
        payload_transaction_id: commit.payload.receipt.transaction_id,
        source_disposition: source_disposition(commit.source.disposition),
        source_receipt_sha256: commit.source.verification.receipt_sha256,
        registration_disposition,
        registration_transaction_id,
        managed_root: NATIVE_CANDIDATE_MANAGED_ROOT_SYMBOLIC_PATH,
        plugin_source: plugin_source_symbolic_path(commit.target),
        outstanding_host_action: commit.outstanding_host_action,
        cleanup_required: commit.payload.cleanup_required || registration_cleanup_required,
    }
}

fn verify_output(verification: NativeCandidateLocalVerification) -> CandidateVerifyOutput {
    let registration_transaction_id = match &verification.registration {
        NativeCandidateRegistrationVerification::Codex(verification) => {
            verification.receipt.transaction_id.clone()
        }
        NativeCandidateRegistrationVerification::ClaudeCode(verification) => {
            verification.receipt.transaction_id.clone()
        }
    };
    CandidateVerifyOutput {
        schema_version: OUTPUT_SCHEMA_VERSION,
        command: "install-candidate-verify",
        target: verification.target,
        state: "healthy",
        artifact: verification.payload.receipt.artifact,
        install_id: verification.payload.receipt.install_id,
        payload_transaction_id: verification.payload.receipt.transaction_id,
        source_receipt_sha256: verification.source.receipt_sha256,
        registration_transaction_id,
        managed_root: NATIVE_CANDIDATE_MANAGED_ROOT_SYMBOLIC_PATH,
        plugin_source: plugin_source_symbolic_path(verification.target),
        outstanding_host_action: verification.outstanding_host_action,
    }
}

fn remove_output(commit: NativeCandidateLocalRemoveCommit) -> CandidateRemoveOutput {
    let (registration_disposition, registration_transaction_id, registration_cleanup_required) =
        registration_lifecycle_details(&commit.registration);
    CandidateRemoveOutput {
        schema_version: OUTPUT_SCHEMA_VERSION,
        command: "install-candidate-remove",
        target: commit.target,
        install_id: commit.payload.receipt.install_id,
        registration_disposition,
        registration_transaction_id,
        source_receipt_sha256: commit.source.receipt_sha256,
        payload_disposition: lifecycle_disposition(commit.payload.disposition),
        payload_transaction_id: commit.payload.receipt.transaction_id,
        approvals_applied: REMOVE_APPROVALS,
        cleanup_required: commit.payload.cleanup_required || registration_cleanup_required,
    }
}

fn registration_commit_details(
    registration: &NativeCandidateRegistrationCommit,
) -> (&'static str, String, bool) {
    match registration {
        NativeCandidateRegistrationCommit::Codex(commit) => (
            codex_registration_disposition(commit.disposition),
            commit.receipt.transaction_id.clone(),
            commit.cleanup_required,
        ),
        NativeCandidateRegistrationCommit::ClaudeCode(commit) => (
            claude_registration_disposition(commit.disposition),
            commit.receipt.transaction_id.clone(),
            commit.cleanup_required,
        ),
    }
}

fn registration_lifecycle_details(
    registration: &NativeCandidateRegistrationLifecycleCommit,
) -> (&'static str, String, bool) {
    match registration {
        NativeCandidateRegistrationLifecycleCommit::Codex(commit) => (
            codex_registration_lifecycle_disposition(commit.disposition),
            commit.receipt.transaction_id.clone(),
            commit.cleanup_required,
        ),
        NativeCandidateRegistrationLifecycleCommit::ClaudeCode(commit) => (
            claude_registration_lifecycle_disposition(commit.disposition),
            commit.receipt.transaction_id.clone(),
            commit.cleanup_required,
        ),
    }
}

const fn plugin_source_symbolic_path(target: ClientActivationTarget) -> &'static str {
    match target {
        ClientActivationTarget::Codex => CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH,
        ClientActivationTarget::ClaudeCode => CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH,
    }
}

const fn install_disposition(disposition: InstallDisposition) -> &'static str {
    match disposition {
        InstallDisposition::Applied => "applied",
        InstallDisposition::AlreadyApplied => "already-applied",
        InstallDisposition::Repaired => "repaired",
        InstallDisposition::AlreadyHealthy => "already-healthy",
    }
}

const fn source_disposition(disposition: NativeCandidatePluginSourceDisposition) -> &'static str {
    match disposition {
        NativeCandidatePluginSourceDisposition::Materialized => "materialized",
        NativeCandidatePluginSourceDisposition::AlreadyHealthy => "already-healthy",
    }
}

const fn lifecycle_disposition(disposition: LifecycleDisposition) -> &'static str {
    match disposition {
        LifecycleDisposition::Removed => "removed",
        LifecycleDisposition::AlreadyRemoved => "already-removed",
        LifecycleDisposition::RolledBack => "rolled-back",
        LifecycleDisposition::AlreadyRolledBack => "already-rolled-back",
    }
}

const fn codex_registration_disposition(
    disposition: qiongli_platform::CodexRegistrationDisposition,
) -> &'static str {
    use qiongli_platform::CodexRegistrationDisposition;
    match disposition {
        CodexRegistrationDisposition::Registered => "registered",
        CodexRegistrationDisposition::AlreadyRegistered => "already-registered",
        CodexRegistrationDisposition::Repaired => "repaired",
        CodexRegistrationDisposition::AlreadyHealthy => "already-healthy",
    }
}

const fn claude_registration_disposition(
    disposition: qiongli_platform::ClaudeRegistrationDisposition,
) -> &'static str {
    use qiongli_platform::ClaudeRegistrationDisposition;
    match disposition {
        ClaudeRegistrationDisposition::Registered => "registered",
        ClaudeRegistrationDisposition::AlreadyRegistered => "already-registered",
        ClaudeRegistrationDisposition::Repaired => "repaired",
        ClaudeRegistrationDisposition::AlreadyHealthy => "already-healthy",
    }
}

const fn codex_registration_lifecycle_disposition(
    disposition: qiongli_platform::CodexRegistrationLifecycleDisposition,
) -> &'static str {
    use qiongli_platform::CodexRegistrationLifecycleDisposition;
    match disposition {
        CodexRegistrationLifecycleDisposition::Removed => "removed",
        CodexRegistrationLifecycleDisposition::AlreadyRemoved => "already-removed",
        CodexRegistrationLifecycleDisposition::RolledBack => "rolled-back",
        CodexRegistrationLifecycleDisposition::AlreadyRolledBack => "already-rolled-back",
    }
}

const fn claude_registration_lifecycle_disposition(
    disposition: qiongli_platform::ClaudeRegistrationLifecycleDisposition,
) -> &'static str {
    use qiongli_platform::ClaudeRegistrationLifecycleDisposition;
    match disposition {
        ClaudeRegistrationLifecycleDisposition::Removed => "removed",
        ClaudeRegistrationLifecycleDisposition::AlreadyRemoved => "already-removed",
        ClaudeRegistrationLifecycleDisposition::RolledBack => "rolled-back",
        ClaudeRegistrationLifecycleDisposition::AlreadyRolledBack => "already-rolled-back",
    }
}

fn require_authority(
    authority: Option<&NativeReleaseAuthority>,
) -> Result<&NativeReleaseAuthority, &'static str> {
    authority.ok_or("native-release-authority-unavailable")
}

fn require_source_commit(source_commit: Option<&str>) -> Result<&str, &'static str> {
    source_commit.ok_or("native-source-commit-unavailable")
}

pub(crate) fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "system-clock-unavailable")
}

#[derive(Debug, Serialize)]
pub(crate) struct CandidatePreviewOutput {
    schema_version: u32,
    command: &'static str,
    target: ClientActivationTarget,
    artifact: qiongli_platform::ArtifactIdentityV1,
    candidate_digest_sha256: String,
    approval_digest_sha256: String,
    release_key_id: String,
    install_id: String,
    archive_sha256: String,
    release_notes_sha256: String,
    managed_root: &'static str,
    plugin_source: &'static str,
    approvals_required: [ApprovalRequirement; 3],
    outstanding_host_action: HostAction,
    mutation: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct CandidateApplyOutput {
    schema_version: u32,
    command: &'static str,
    target: ClientActivationTarget,
    artifact: qiongli_platform::ArtifactIdentityV1,
    candidate_digest_sha256: String,
    approval_digest_sha256: String,
    install_id: String,
    payload_disposition: &'static str,
    payload_transaction_id: String,
    source_disposition: &'static str,
    source_receipt_sha256: String,
    registration_disposition: &'static str,
    registration_transaction_id: String,
    managed_root: &'static str,
    plugin_source: &'static str,
    outstanding_host_action: HostAction,
    cleanup_required: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct CandidateVerifyOutput {
    schema_version: u32,
    command: &'static str,
    target: ClientActivationTarget,
    state: &'static str,
    artifact: qiongli_platform::ArtifactIdentityV1,
    install_id: String,
    payload_transaction_id: String,
    source_receipt_sha256: String,
    registration_transaction_id: String,
    managed_root: &'static str,
    plugin_source: &'static str,
    outstanding_host_action: HostAction,
}

#[derive(Debug, Serialize)]
pub(crate) struct CandidateRemoveOutput {
    schema_version: u32,
    command: &'static str,
    target: ClientActivationTarget,
    install_id: String,
    registration_disposition: &'static str,
    registration_transaction_id: String,
    source_receipt_sha256: String,
    payload_disposition: &'static str,
    payload_transaction_id: String,
    approvals_applied: [ApprovalRequirement; 2],
    cleanup_required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_preview_contract_matches_generated_and_consumer() {
        let generated: serde_json::Value =
            serde_json::from_str(&candidate_activation_preview_contract_json().unwrap()).unwrap();
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../schemas/candidate-activation-preview-v1.schema.json"
        ))
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/candidate-activation-preview-v1.json"
        ))
        .unwrap();
        assert_eq!(generated["schema"], schema);
        assert_eq!(generated["fixture"], fixture);
        let decoded: CandidateActivationPreviewOutput =
            serde_json::from_value(fixture.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), fixture);
        let mut invalid = fixture;
        invalid["approval_digest_sha256"] = serde_json::json!("a".repeat(64));
        assert!(serde_json::from_value::<CandidateActivationPreviewOutput>(invalid).is_err());
    }

    #[test]
    fn candidate_stage_contract_matches_generated_schema_and_fixtures() {
        let generated: serde_json::Value =
            serde_json::from_str(&candidate_stage_contract_json().unwrap()).unwrap();
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../schemas/candidate-stage-v1.schema.json"))
                .unwrap();
        assert_eq!(generated["schema"], schema);
        for (index, fixture) in [
            include_str!("../tests/fixtures/candidate-stage-v1.ready.json"),
            include_str!("../tests/fixtures/candidate-stage-v1.staged.json"),
            include_str!("../tests/fixtures/candidate-stage-v1.already-staged.json"),
        ]
        .into_iter()
        .enumerate()
        {
            let value: serde_json::Value = serde_json::from_str(fixture).unwrap();
            assert_eq!(generated["fixtures"][index], value);
            let decoded: CandidateStageOutput = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), value);
            let mut unknown = value;
            unknown["unexpected"] = true.into();
            assert!(serde_json::from_value::<CandidateStageOutput>(unknown).is_err());
        }
    }

    #[test]
    fn approval_digest_is_deterministic_lower_hex_and_target_bound() {
        let candidate = "a".repeat(64);
        let codex = candidate_approval_digest(&candidate, ClientActivationTarget::Codex);
        let claude = candidate_approval_digest(&candidate, ClientActivationTarget::ClaudeCode);
        assert_eq!(codex.len(), 64);
        assert!(
            codex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        );
        assert_eq!(
            codex,
            candidate_approval_digest(&candidate, ClientActivationTarget::Codex)
        );
        assert_ne!(codex, claude);
        assert_ne!(stage_approval_digest(&codex), codex);
        assert_ne!(
            stage_approval_digest(&codex),
            stage_approval_digest(&claude)
        );
        assert_ne!(
            stage_approval_digest(&codex),
            stage_approval_digest(&candidate_approval_digest(
                &"b".repeat(64),
                ClientActivationTarget::Codex
            ))
        );
    }
}
