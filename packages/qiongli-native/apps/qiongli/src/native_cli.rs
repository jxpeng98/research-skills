use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::EmbeddedContent;
use qiongli_platform::{
    AllowedRootV1, ApprovalRequirement, CapabilityProfile, GrantMode, GrantVerificationContext,
    InstallDisposition, InstallPlanMetadataV1, InstallScope, IntegrationScope,
    LifecycleDisposition, LocalSurface, LocalTargetFamily, MAX_NATIVE_RELEASE_ENVELOPE_BYTES,
    ManagedNativePayloadExecutor, NativeReleaseAuthority, NativeReleaseVerificationContext,
    SignedNativeReleaseEnvelopeV1, SymbolicRoot, TargetDescriptorV1, VerifiedInstallPlan,
    VerifiedNativeReleaseEnvelope, approve_install_plan, approve_managed_root,
    approve_native_portable_archive_target, current_target_native_artifact_identity,
    native_payload_install_id, preview_native_payload_install,
};
use serde::Serialize;

const OUTPUT_SCHEMA_VERSION: u32 = 1;
const PLAN_TTL_SECONDS: u64 = 600;
const MANAGED_ROOT_ID: &str = "qiongli-data";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeClientTarget {
    Codex,
    Claude,
}

impl NativeClientTarget {
    const fn family(self) -> LocalTargetFamily {
        match self {
            Self::Codex => LocalTargetFamily::CodexLocal,
            Self::Claude => LocalTargetFamily::ClaudeCodeLocal,
        }
    }

    const fn integration_scope(self) -> IntegrationScope {
        match self {
            Self::Codex => IntegrationScope::CodexLocal,
            Self::Claude => IntegrationScope::ClaudeCodeLocal,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeReleaseOptions {
    pub release: PathBuf,
    pub archive: PathBuf,
    pub managed_root: PathBuf,
    pub target: NativeClientTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeReceiptOptions {
    pub managed_root: PathBuf,
    pub install_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NativeCliCommand {
    Preview(NativeReleaseOptions),
    Apply {
        options: NativeReleaseOptions,
        expected_plan_digest: String,
    },
    Verify(NativeReceiptOptions),
    Remove(NativeReceiptOptions),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum NativeCliOutput {
    Preview(NativePreviewOutput),
    Apply(NativeApplyOutput),
    Verify(NativeVerifyOutput),
    Remove(NativeRemoveOutput),
}

pub(crate) fn execute(
    command: NativeCliCommand,
    authority: Option<&NativeReleaseAuthority>,
    content: &EmbeddedContent,
    environment: &crate::command::CommandEnvironment,
) -> Result<NativeCliOutput, &'static str> {
    match command {
        NativeCliCommand::Preview(options) => {
            let authority = authority.ok_or("native-release-authority-unavailable")?;
            let now_unix = now_unix()?;
            let prepared = prepare_install(&options, authority, content, now_unix)?;
            Ok(NativeCliOutput::Preview(prepared.preview_output()))
        }
        NativeCliCommand::Apply {
            options,
            expected_plan_digest,
        } => {
            let authority = authority.ok_or("native-release-authority-unavailable")?;
            let now_unix = now_unix()?;
            let prepared = prepare_install(&options, authority, content, now_unix)?;
            if prepared.plan.plan().semantic_digest_sha256 != expected_plan_digest {
                return Err("native-install-plan-digest-mismatch");
            }
            let approval = approve_install_plan(
                &prepared.plan,
                &[ApprovalRequirement::FilesystemWrite],
                now_unix,
            )
            .map_err(|error| error.reason_code())?;
            let _write_guard = crate::update_reconcile::acquire_managed_write_guard(
                &managed_payload_home(&options.managed_root, environment)?,
                crate::command::config_root(environment).map_err(|error| error.reason_code())?,
            )?;
            let executor = ManagedNativePayloadExecutor::new(prepared.managed_root);
            let commit = executor
                .apply(
                    &prepared.plan,
                    &approval,
                    content.pack(),
                    &prepared.release,
                    now_unix,
                )
                .map_err(|error| error.reason_code())?;
            Ok(NativeCliOutput::Apply(NativeApplyOutput {
                schema_version: OUTPUT_SCHEMA_VERSION,
                command: "install-native-apply",
                disposition: install_disposition(commit.disposition),
                transaction_id: commit.receipt.transaction_id,
                install_id: commit.receipt.install_id,
                plan_digest_sha256: commit.receipt.semantic_digest_sha256,
                release_envelope_sha256: commit.receipt.operation.release_envelope_sha256,
                archive_sha256: commit.receipt.operation.archive_sha256,
                cleanup_required: commit.cleanup_required,
            }))
        }
        NativeCliCommand::Verify(options) => {
            let managed_root = approve_managed_root(&allowed_root(), &options.managed_root)
                .map_err(|error| error.reason_code())?;
            let verification = ManagedNativePayloadExecutor::new(managed_root)
                .verify(&options.install_id, content.pack())
                .map_err(|error| error.reason_code())?;
            Ok(NativeCliOutput::Verify(NativeVerifyOutput {
                schema_version: OUTPUT_SCHEMA_VERSION,
                command: "install-native-verify",
                state: "healthy",
                transaction_id: verification.receipt.transaction_id,
                install_id: verification.receipt.install_id,
                plan_digest_sha256: verification.receipt.semantic_digest_sha256,
                release_envelope_sha256: verification.receipt.operation.release_envelope_sha256,
                archive_sha256: verification.receipt.operation.archive_sha256,
            }))
        }
        NativeCliCommand::Remove(options) => {
            let managed_root = approve_managed_root(&allowed_root(), &options.managed_root)
                .map_err(|error| error.reason_code())?;
            let _write_guard = crate::update_reconcile::acquire_managed_write_guard(
                &managed_payload_home(&options.managed_root, environment)?,
                crate::command::config_root(environment).map_err(|error| error.reason_code())?,
            )?;
            let commit = ManagedNativePayloadExecutor::new(managed_root)
                .remove(&options.install_id, content.pack(), now_unix()?)
                .map_err(|error| error.reason_code())?;
            Ok(NativeCliOutput::Remove(NativeRemoveOutput {
                schema_version: OUTPUT_SCHEMA_VERSION,
                command: "install-native-remove",
                disposition: lifecycle_disposition(commit.disposition),
                transaction_id: commit.receipt.transaction_id,
                install_id: commit.receipt.install_id,
                prior_transaction_id: commit.receipt.prior_transaction_id,
                cleanup_required: commit.cleanup_required,
            }))
        }
    }
}

// Resolve the actual fixed candidate root before choosing its installation lock.
// General engineering roots keep the invoking Home's coordination scope.
fn managed_payload_home(
    managed_root: &Path,
    environment: &crate::command::CommandEnvironment,
) -> Result<PathBuf, &'static str> {
    let canonical =
        fs::canonicalize(managed_root).map_err(|_| "native-managed-root-unavailable")?;
    if let Some(home) = canonical.ancestors().nth(3)
        && home.join(".qiongli/native/payloads") == canonical
    {
        return Ok(home.to_path_buf());
    }
    environment
        .platform_home()
        .map(Path::to_path_buf)
        .ok_or("native-candidate-home-unavailable")
}

struct PreparedInstall {
    release: VerifiedNativeReleaseEnvelope,
    plan: VerifiedInstallPlan,
    managed_root: qiongli_platform::ApprovedManagedRoot,
}

impl PreparedInstall {
    fn preview_output(&self) -> NativePreviewOutput {
        NativePreviewOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-native-preview",
            artifact: self.plan.plan().artifact.clone(),
            target: self.plan.plan().target.clone(),
            install_id: native_payload_install_id(self.release.archive()),
            plan_digest_sha256: self.plan.plan().semantic_digest_sha256.clone(),
            release_envelope_sha256: self.release.signed_payload_sha256().to_string(),
            archive_sha256: self.release.archive().archive_sha256().to_string(),
            approvals_required: [ApprovalRequirement::FilesystemWrite],
            mutation: "none",
        }
    }
}

fn prepare_install(
    options: &NativeReleaseOptions,
    authority: &NativeReleaseAuthority,
    content: &EmbeddedContent,
    now_unix: u64,
) -> Result<PreparedInstall, &'static str> {
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), authority.channel())
            .map_err(|error| error.reason_code())?;
    let managed_root = approve_managed_root(&allowed_root(), &options.managed_root)
        .map_err(|error| error.reason_code())?;
    let archive_target = approve_native_portable_archive_target(&options.archive, &artifact)
        .map_err(|error| error.reason_code())?;
    let release_bytes = read_release(&options.release)?;
    let signed_release = SignedNativeReleaseEnvelopeV1::from_json(&release_bytes)
        .map_err(|error| error.reason_code())?;
    let release_context = NativeReleaseVerificationContext {
        now_unix,
        minimum_release_generation: authority.minimum_release_generation(),
        minimum_launch_grant_generation: authority.minimum_launch_grant_generation(),
        expected_artifact: &artifact,
        expected_channel: authority.channel(),
        requested_mode: GrantMode::LiteMcp,
        requested_scope: options.target.integration_scope(),
    };
    let release = signed_release
        .verify(
            authority.release_keys(),
            authority.launch_grant_keys(),
            &release_context,
            content.pack(),
            &archive_target,
        )
        .map_err(|error| error.reason_code())?;
    let expires_at_unix = now_unix
        .saturating_add(PLAN_TTL_SECONDS)
        .min(release.envelope().expires_at_unix);
    let digest_prefix = release
        .signed_payload_sha256()
        .get(..32)
        .ok_or("native-release-envelope-invalid")?;
    let plan = preview_native_payload_install(
        InstallPlanMetadataV1 {
            plan_id: format!("native-cli-{digest_prefix}"),
            created_at_unix: now_unix,
            expires_at_unix,
        },
        &release,
        TargetDescriptorV1 {
            family: options.target.family(),
            surface: LocalSurface::CliLocal,
            scope: InstallScope::User,
            profile: CapabilityProfile::Lite,
            os: artifact.os,
            arch: artifact.arch,
            adapter_version: 1,
        },
        allowed_root(),
    )
    .map_err(|error| error.reason_code())?;
    let grant_context = GrantVerificationContext {
        now_unix,
        minimum_generation: authority.minimum_launch_grant_generation(),
        expected_artifact: &artifact,
        binary_sha256: &release.envelope().binary_sha256,
        resource_pack_sha256: &release.envelope().resource_pack_sha256,
        requested_mode: GrantMode::LiteMcp,
        requested_scope: options.target.integration_scope(),
    };
    let plan = plan
        .verify(authority.launch_grant_keys(), &grant_context)
        .map_err(|error| error.reason_code())?;
    Ok(PreparedInstall {
        release,
        plan,
        managed_root,
    })
}

fn read_release(path: &PathBuf) -> Result<Vec<u8>, &'static str> {
    let file = fs::File::open(path).map_err(|_| "native-release-envelope-read-failed")?;
    let limit = u64::try_from(MAX_NATIVE_RELEASE_ENVELOPE_BYTES)
        .map_err(|_| "native-release-envelope-too-large")?;
    let mut bounded = file.take(limit.saturating_add(1));
    let mut bytes = Vec::new();
    bounded
        .read_to_end(&mut bytes)
        .map_err(|_| "native-release-envelope-read-failed")?;
    if bytes.len() > MAX_NATIVE_RELEASE_ENVELOPE_BYTES {
        return Err("native-release-envelope-too-large");
    }
    Ok(bytes)
}

fn allowed_root() -> AllowedRootV1 {
    AllowedRootV1 {
        id: MANAGED_ROOT_ID.to_string(),
        root: SymbolicRoot::QiongliManagedData,
    }
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "system-clock-unavailable")
}

const fn install_disposition(disposition: InstallDisposition) -> &'static str {
    match disposition {
        InstallDisposition::Applied => "applied",
        InstallDisposition::AlreadyApplied => "already-applied",
        InstallDisposition::Repaired => "repaired",
        InstallDisposition::AlreadyHealthy => "already-healthy",
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

#[derive(Debug, Serialize)]
pub(crate) struct NativePreviewOutput {
    schema_version: u32,
    command: &'static str,
    artifact: qiongli_platform::ArtifactIdentityV1,
    target: TargetDescriptorV1,
    install_id: String,
    plan_digest_sha256: String,
    release_envelope_sha256: String,
    archive_sha256: String,
    approvals_required: [ApprovalRequirement; 1],
    mutation: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeApplyOutput {
    schema_version: u32,
    command: &'static str,
    disposition: &'static str,
    transaction_id: String,
    install_id: String,
    plan_digest_sha256: String,
    release_envelope_sha256: String,
    archive_sha256: String,
    cleanup_required: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeVerifyOutput {
    schema_version: u32,
    command: &'static str,
    state: &'static str,
    transaction_id: String,
    install_id: String,
    plan_digest_sha256: String,
    release_envelope_sha256: String,
    archive_sha256: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeRemoveOutput {
    schema_version: u32,
    command: &'static str,
    disposition: &'static str,
    transaction_id: String,
    install_id: String,
    prior_transaction_id: String,
    cleanup_required: bool,
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use ed25519_dalek::{Signer, SigningKey};
    use qiongli_platform::{
        GrantSignatureV1, LaunchGrantV1, NativeReleaseSignatureV1, ReleaseChannel,
        SignatureAlgorithm, SignedLaunchGrantV1, SignedNativeReleaseEnvelopeV1,
        approve_native_artifact_target, approve_native_portable_archive_target,
        build_native_release_envelope, compose_native_artifact, compose_native_portable_archive,
        current_target_native_artifact_identity, launch_grant_signing_bytes, native_artifact_id,
        native_portable_archive_file_name, native_release_envelope_signing_bytes,
    };
    use serde_json::json;

    use super::*;

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        managed_root: PathBuf,
        archive_path: PathBuf,
        release_path: PathBuf,
        artifact_id: String,
        authority: NativeReleaseAuthority,
    }

    impl Fixture {
        fn new(content: &EmbeddedContent) -> Self {
            let test_base = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .expect("app crate must live below the native workspace")
                .join("target/qiongli-native-cli-tests");
            fs::create_dir_all(&test_base).expect("native CLI test base must exist");
            let root = test_base.join(format!(
                "fixture-{}-{}-{}",
                std::process::id(),
                now_unix().unwrap(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            create_private_directory(&root);

            let artifact = [
                ReleaseChannel::Alpha,
                ReleaseChannel::Beta,
                ReleaseChannel::Stable,
            ]
            .into_iter()
            .find_map(|channel| {
                current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), channel).ok()
            })
            .expect("current native CLI test identity must resolve");
            let artifact_id = native_artifact_id(&artifact).unwrap();
            let archive_name = native_portable_archive_file_name(&artifact).unwrap();
            let source_binary =
                root.join(format!("bounded-qiongli{}", std::env::consts::EXE_SUFFIX));
            fs::write(&source_binary, b"bounded-native-cli-test-payload")
                .expect("bounded native CLI payload must be written");
            set_executable_mode(&source_binary);

            let source_parent = root.join("source-parent");
            create_private_directory(&source_parent);
            let source_path = source_parent.join(&artifact_id);
            let source_target = approve_native_artifact_target(&source_path, &artifact)
                .expect("native CLI source target must approve");
            compose_native_artifact(content.pack(), &artifact, &source_binary, &source_target)
                .expect("native CLI source artifact must compose");

            let archive_parent = root.join("archive-private-path-canary");
            create_private_directory(&archive_parent);
            let archive_path = archive_parent.join(archive_name);
            let archive_target = approve_native_portable_archive_target(&archive_path, &artifact)
                .expect("native CLI archive target must approve");
            let archive =
                compose_native_portable_archive(content.pack(), &source_target, &archive_target)
                    .expect("native CLI archive must compose");

            let now = now_unix().unwrap();
            let launch_key = SigningKey::from_bytes(&[91_u8; 32]);
            let grant = LaunchGrantV1 {
                schema_version: 1,
                generation: 13,
                artifact: artifact.clone(),
                binary_sha256: archive.payload().manifest().binary_sha256.clone(),
                resource_pack_sha256: archive.payload().manifest().content.pack_sha256.clone(),
                allowed_modes: vec![GrantMode::LiteMcp],
                integration_scopes: vec![IntegrationScope::CodexLocal],
                not_before_unix: now.saturating_sub(60),
                expires_at_unix: now.saturating_add(3_600),
            };
            let grant_signature = launch_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
            let signed_grant = SignedLaunchGrantV1 {
                grant,
                signature: GrantSignatureV1 {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: "cli-launch-test-key".to_string(),
                    value_hex: encode_hex(&grant_signature.to_bytes()),
                },
            };
            let envelope = build_native_release_envelope(
                19,
                &archive,
                &signed_grant,
                now.saturating_sub(30),
                now.saturating_add(1_800),
            )
            .expect("native CLI release envelope must build");
            let release_key = SigningKey::from_bytes(&[92_u8; 32]);
            let release_signature =
                release_key.sign(&native_release_envelope_signing_bytes(&envelope).unwrap());
            let signed_release = SignedNativeReleaseEnvelopeV1 {
                envelope,
                signature: NativeReleaseSignatureV1 {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: "cli-release-test-key".to_string(),
                    value_hex: encode_hex(&release_signature.to_bytes()),
                },
            };
            let release_path = root.join("release-private-path-canary.json");
            fs::write(&release_path, signed_release.to_canonical_json().unwrap())
                .expect("native CLI signed release must be written");
            let authority_bytes = serde_json_canonicalizer::to_vec(&json!({
                "schema_version": 1,
                "channel": artifact.channel,
                "minimum_release_generation": 19,
                "minimum_launch_grant_generation": 13,
                "release_keys": [{
                    "key_id": "cli-release-test-key",
                    "public_key_hex": encode_hex(&release_key.verifying_key().to_bytes()),
                    "minimum_generation": 19,
                    "maximum_generation_exclusive": 20
                }],
                "launch_grant_keys": [{
                    "key_id": "cli-launch-test-key",
                    "public_key_hex": encode_hex(&launch_key.verifying_key().to_bytes())
                }]
            }))
            .expect("native CLI authority must canonicalize");
            let authority = NativeReleaseAuthority::from_json(&authority_bytes)
                .expect("native CLI authority must validate");
            let managed_root = root.join("managed-private-path-canary");
            create_private_directory(&managed_root);
            Self {
                root,
                managed_root,
                archive_path,
                release_path,
                artifact_id,
                authority,
            }
        }

        fn release_options(&self) -> NativeReleaseOptions {
            NativeReleaseOptions {
                release: self.release_path.clone(),
                archive: self.archive_path.clone(),
                managed_root: self.managed_root.clone(),
                target: NativeClientTarget::Codex,
            }
        }

        fn receipt_options(&self, install_id: &str) -> NativeReceiptOptions {
            NativeReceiptOptions {
                managed_root: self.managed_root.clone(),
                install_id: install_id.to_string(),
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn authority_backed_cli_previews_applies_verifies_and_removes() {
        let content = crate::embedded_content().expect("embedded content must verify");
        for use_other_home in [false, true] {
            let mut fixture = Fixture::new(&content);
            let invoking_home = if use_other_home {
                qiongli_platform::prepare_native_candidate_managed_root(&fixture.root).unwrap();
                fixture.managed_root = fixture.root.join(".qiongli/native/payloads");
                let other = fixture.root.join("other-home");
                create_private_directory(&other);
                other
            } else {
                fixture.root.clone()
            };
            let environment = crate::command::CommandEnvironment::with_paths(
                None::<std::ffi::OsString>,
                Some(invoking_home),
                None,
            );
            let execute = |command, authority, content| {
                super::execute(command, authority, content, &environment)
            };

            let preview = execute(
                NativeCliCommand::Preview(fixture.release_options()),
                Some(&fixture.authority),
                &content,
            )
            .expect("native CLI preview must succeed");
            let preview_rendered = serde_json::to_string(&preview).unwrap();
            let NativeCliOutput::Preview(preview) = preview else {
                panic!("native CLI preview returned the wrong output");
            };
            assert_eq!(preview.mutation, "none");
            assert_eq!(preview.target.family, LocalTargetFamily::CodexLocal);
            assert!(!fixture.managed_root.join(&fixture.artifact_id).exists());

            assert_eq!(
                execute(
                    NativeCliCommand::Apply {
                        options: fixture.release_options(),
                        expected_plan_digest: "0".repeat(64),
                    },
                    Some(&fixture.authority),
                    &content,
                )
                .unwrap_err(),
                "native-install-plan-digest-mismatch"
            );
            assert!(!fixture.managed_root.join(&fixture.artifact_id).exists());

            let apply_command = NativeCliCommand::Apply {
                options: fixture.release_options(),
                expected_plan_digest: preview.plan_digest_sha256.clone(),
            };
            #[cfg(unix)]
            {
                let held =
                    crate::update_reconcile::acquire_native_home_write_lock(&fixture.root).unwrap();
                assert_eq!(
                    execute(apply_command.clone(), Some(&fixture.authority), &content).unwrap_err(),
                    "native-update-replacement-active"
                );
                assert!(!fixture.managed_root.join(&fixture.artifact_id).exists());
                drop(held);
            }
            let applied = execute(apply_command.clone(), Some(&fixture.authority), &content)
                .expect("native CLI apply must succeed");
            let applied_rendered = serde_json::to_string(&applied).unwrap();
            let NativeCliOutput::Apply(applied) = applied else {
                panic!("native CLI apply returned the wrong output");
            };
            assert_eq!(applied.disposition, "applied");
            assert_eq!(applied.install_id, preview.install_id);
            assert!(fixture.managed_root.join(&fixture.artifact_id).is_dir());

            let replay = execute(apply_command, Some(&fixture.authority), &content)
                .expect("native CLI apply replay must succeed");
            let NativeCliOutput::Apply(replay) = replay else {
                panic!("native CLI replay returned the wrong output");
            };
            assert_eq!(replay.disposition, "already-applied");

            let receipt_options = fixture.receipt_options(&preview.install_id);
            let verified = execute(
                NativeCliCommand::Verify(receipt_options.clone()),
                None,
                &content,
            )
            .expect("receipt-backed native CLI verify must succeed without authority");
            let verified_rendered = serde_json::to_string(&verified).unwrap();
            let NativeCliOutput::Verify(verified) = verified else {
                panic!("native CLI verify returned the wrong output");
            };
            assert_eq!(verified.state, "healthy");
            assert_eq!(verified.install_id, preview.install_id);

            #[cfg(unix)]
            {
                let marker = fixture
                    .root
                    .join(".qiongli/native/active-installation.json");
                fs::write(&marker, b"pending-recovery").unwrap();
                assert_eq!(
                    execute(
                        NativeCliCommand::Remove(receipt_options.clone()),
                        None,
                        &content
                    )
                    .unwrap_err(),
                    "native-activation-recovery-required"
                );
                assert!(fixture.managed_root.join(&fixture.artifact_id).is_dir());
                fs::remove_file(marker).unwrap();
            }
            let removed = execute(
                NativeCliCommand::Remove(receipt_options.clone()),
                None,
                &content,
            )
            .expect("receipt-backed native CLI remove must succeed without authority");
            let removed_rendered = serde_json::to_string(&removed).unwrap();
            let NativeCliOutput::Remove(removed) = removed else {
                panic!("native CLI remove returned the wrong output");
            };
            assert_eq!(removed.disposition, "removed");
            assert!(!fixture.managed_root.join(&fixture.artifact_id).exists());

            let removed_replay = execute(NativeCliCommand::Remove(receipt_options), None, &content)
                .expect("native CLI remove replay must succeed");
            let NativeCliOutput::Remove(removed_replay) = removed_replay else {
                panic!("native CLI remove replay returned the wrong output");
            };
            assert_eq!(removed_replay.disposition, "already-removed");

            for output in [
                preview_rendered,
                applied_rendered,
                verified_rendered,
                removed_rendered,
            ] {
                assert!(!output.contains(fixture.release_path.to_string_lossy().as_ref()));
                assert!(!output.contains(fixture.archive_path.to_string_lossy().as_ref()));
                assert!(!output.contains(fixture.managed_root.to_string_lossy().as_ref()));
                assert!(!output.contains("private-path-canary"));
            }
        }
    }

    fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(N * 2);
        for byte in bytes {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    #[cfg(unix)]
    fn create_private_directory(path: &Path) {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder
            .create(path)
            .expect("private directory must be created");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .expect("private directory mode must be retained");
    }

    #[cfg(windows)]
    fn create_private_directory(path: &Path) {
        qiongli_windows_security::create_owner_only_directory(path)
            .expect("owner-only Windows directory must be created");
    }

    #[cfg(not(any(unix, windows)))]
    fn create_private_directory(path: &Path) {
        fs::create_dir(path).expect("private directory must be created");
    }

    #[cfg(unix)]
    fn set_executable_mode(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .expect("test payload must be executable");
    }

    #[cfg(not(unix))]
    fn set_executable_mode(_path: &Path) {}
}
