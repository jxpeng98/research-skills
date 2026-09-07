#![allow(clippy::disallowed_methods)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use qiongli_content::{
    CompatibleProduct, ResourcePackBuildMetadata, build_resource_pack, collect_canonical_sources,
    load_resource_pack,
};
use qiongli_platform::{
    ClaudeRegistrationDisposition, ClientActivationTarget, CodexAdapterError,
    CodexRegistrationDisposition, GrantMode, GrantSignatureV1, InstallDisposition, InstallerKind,
    IntegrationScope, LaunchGrantV1, NativeCandidateLocalInstallError,
    NativeCandidatePluginSourceDisposition, NativeCandidatePluginSourceError,
    NativeCandidateRegistrationCommit, NativeClientPluginGrantV1, NativeReleaseAuthority,
    NativeReleaseCandidateError, NativeReleaseCandidateVerificationContext,
    NativeReleaseSignatureV1, ReleaseChannel, SignatureAlgorithm, SignedLaunchGrantV1,
    SignedNativeReleaseCandidateV1, SignedNativeReleaseEnvelopeV1, TransactionError,
    apply_native_release_candidate_local, approve_native_artifact_target,
    approve_native_portable_archive_target, build_native_release_candidate,
    build_native_release_envelope, compose_native_artifact, compose_native_portable_archive,
    current_target_native_artifact_identity, launch_grant_signing_bytes,
    materialize_native_candidate_plugin_source, native_artifact_id,
    native_portable_archive_file_name, native_release_candidate_signing_bytes,
    native_release_envelope_signing_bytes, prepare_native_candidate_plugin_source_target,
    remove_native_candidate_plugin_source, remove_native_release_candidate_local,
    verify_installed_native_candidate_product, verify_native_candidate_plugin_source,
    verify_native_release_candidate_local,
};
use serde_json::json;

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
const NOW: u64 = 1_750_000_000;
const SOURCE_COMMIT: &str = "89abcdef0123456789abcdef0123456789abcdef";
const NOTES: &[u8] = b"# Qiongli 2.0.0-alpha.1\n\nLite local release candidate.\n";
const CONTENT_ROOTS: [&str; 12] = [
    ".claude-plugin",
    ".codex-plugin",
    "distribution",
    "mcp-contracts",
    "roles",
    "schemas",
    "skills",
    "standards",
    "subjects",
    "templates",
    "venue-profiles",
    "workflow",
];

struct Fixture {
    root: PathBuf,
    source_binary: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-native-release-candidate-tests");
        fs::create_dir_all(&test_base).expect("release candidate test base must exist");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let root = test_base.join(format!(
            "{name}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        create_private_directory(&root);
        let source_binary = root.join(format!("qiongli-source{}", std::env::consts::EXE_SUFFIX));
        fs::write(
            &source_binary,
            b"qiongli-native-release-candidate-fixture-v1",
        )
        .expect("bounded candidate fixture binary must write");
        set_executable_mode(&source_binary);
        Self {
            root,
            source_binary,
        }
    }

    fn target(&self, parent_name: &str, leaf: &str) -> PathBuf {
        let parent = self.root.join(parent_name);
        create_private_directory(&parent);
        parent.join(leaf)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn create_private_directory(path: &Path) {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .expect("private fixture directory must be created");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture directory must remain private");
}

#[cfg(windows)]
fn create_private_directory(path: &Path) {
    qiongli_windows_security::create_owner_only_directory(path)
        .expect("owner-only Windows fixture directory must be created");
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(path: &Path) {
    fs::create_dir(path).expect("private fixture directory must be created");
}

#[cfg(unix)]
fn set_executable_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture binary must be executable");
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) {}

fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(N * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn sign_grant(grant: LaunchGrantV1, key: &SigningKey, key_id: &str) -> SignedLaunchGrantV1 {
    let signature = key.sign(&launch_grant_signing_bytes(&grant).unwrap());
    SignedLaunchGrantV1 {
        grant,
        signature: GrantSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: key_id.to_string(),
            value_hex: encode_hex(&signature.to_bytes()),
        },
    }
}

fn plugin_grant(
    artifact: &qiongli_platform::ArtifactIdentityV1,
    target: ClientActivationTarget,
    binary_sha256: &str,
    pack_sha256: &str,
    key: &SigningKey,
) -> NativeClientPluginGrantV1 {
    let mut plugin_artifact = artifact.clone();
    plugin_artifact.installer_kind = InstallerKind::PluginBundle;
    NativeClientPluginGrantV1 {
        target,
        signed_launch_grant: sign_grant(
            LaunchGrantV1 {
                schema_version: 1,
                generation: 23,
                artifact: plugin_artifact,
                binary_sha256: binary_sha256.to_string(),
                resource_pack_sha256: pack_sha256.to_string(),
                allowed_modes: target.allowed_grant_modes().to_vec(),
                integration_scopes: vec![target.integration_scope()],
                not_before_unix: NOW - 60,
                expires_at_unix: NOW + 3_600,
            },
            key,
            "candidate-launch-test-key",
        ),
    }
}

fn authority_with_policy(
    release_key: &SigningKey,
    launch_key: &SigningKey,
    channel: &str,
    minimum_release_generation: u64,
    release_key_minimum_generation: u64,
    release_key_maximum_generation_exclusive: u64,
) -> NativeReleaseAuthority {
    let document = json!({
        "schema_version": 1,
        "channel": channel,
        "minimum_release_generation": minimum_release_generation,
        "minimum_launch_grant_generation": 23,
        "release_keys": [{
            "key_id": "candidate-release-test-key",
            "public_key_hex": encode_hex(&release_key.verifying_key().to_bytes()),
            "minimum_generation": release_key_minimum_generation,
            "maximum_generation_exclusive": release_key_maximum_generation_exclusive
        }],
        "launch_grant_keys": [{
            "key_id": "candidate-launch-test-key",
            "public_key_hex": encode_hex(&launch_key.verifying_key().to_bytes())
        }]
    });
    let bytes = serde_json_canonicalizer::to_vec(&document).unwrap();
    NativeReleaseAuthority::from_json(&bytes).expect("test authority must be canonical")
}

fn authority(release_key: &SigningKey, launch_key: &SigningKey) -> NativeReleaseAuthority {
    authority_with_policy(release_key, launch_key, "alpha", 29, 29, 30)
}

fn resign_candidate(
    mut candidate: SignedNativeReleaseCandidateV1,
    release_key: &SigningKey,
) -> SignedNativeReleaseCandidateV1 {
    let signature = release_key.sign(
        &native_release_candidate_signing_bytes(&candidate.candidate)
            .expect("candidate signing bytes must rebuild"),
    );
    candidate.signature.value_hex = encode_hex(&signature.to_bytes());
    candidate
}

fn corrupt_hex(value: &mut String) {
    let replacement = if value.starts_with('0') { "1" } else { "0" };
    value.replace_range(0..1, replacement);
}

fn minimal_pack(root: &Path) -> qiongli_content::BuiltResourcePack {
    let content_root = root.join("minimal-content");
    fs::create_dir(&content_root).expect("minimal content root must be created");
    for directory in CONTENT_ROOTS {
        fs::create_dir_all(content_root.join(directory))
            .expect("canonical content directory must be created");
    }
    fs::write(content_root.join("skills-core.md"), b"minimal core")
        .expect("core fixture must write");
    fs::write(content_root.join("skills-summary.md"), b"minimal summary")
        .expect("summary fixture must write");
    fs::write(
        content_root.join(".codex-plugin/plugin.json"),
        br#"{"name":"qiongli","version":"0.0.0"}"#,
    )
    .expect("Codex manifest fixture must write");
    fs::write(
        content_root.join(".claude-plugin/plugin.json"),
        br#"{"name":"qiongli","version":"0.0.0"}"#,
    )
    .expect("Claude manifest fixture must write");
    fs::write(
        content_root.join("workflow/SKILL.md"),
        b"---\nname: qiongli\ndescription: minimal candidate fixture\n---\n",
    )
    .expect("workflow fixture must write");
    let resources = collect_canonical_sources(&content_root).expect("minimal content must collect");
    build_resource_pack(
        &ResourcePackBuildMetadata {
            pack_id: "qiongli-core".to_string(),
            content_version: "1.19.0-beta.1".to_string(),
            source_commit: SOURCE_COMMIT.to_string(),
            compatible_product: CompatibleProduct {
                minimum: "2.0.0-alpha.1".to_string(),
                maximum_exclusive: "3.0.0".to_string(),
            },
        },
        &resources,
    )
    .expect("minimal resource pack must build")
}

#[test]
fn signed_candidate_verifies_both_target_capabilities_and_rejects_tampering() {
    let fixture = Fixture::new("complete-candidate");
    let built_pack = minimal_pack(&fixture.root);
    let content = load_resource_pack(built_pack.core_bytes(), built_pack.pack_sha256())
        .expect("minimal content must verify");
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), ReleaseChannel::Alpha)
            .expect("current target artifact must resolve");
    let artifact_id = native_artifact_id(&artifact).expect("artifact ID must render");
    let artifact_path = fixture.target("artifact", &artifact_id);
    let artifact_target = approve_native_artifact_target(&artifact_path, &artifact)
        .expect("artifact target must approve");
    let assembled = compose_native_artifact(
        &content,
        &artifact,
        &fixture.source_binary,
        &artifact_target,
    )
    .expect("native artifact must compose");
    let archive_name =
        native_portable_archive_file_name(&artifact).expect("archive name must render");
    let archive_path = fixture.target("archive", &archive_name);
    let archive_target = approve_native_portable_archive_target(&archive_path, &artifact)
        .expect("archive target must approve");
    let archive = compose_native_portable_archive(&content, &artifact_target, &archive_target)
        .expect("portable archive must compose");

    let launch_key = SigningKey::from_bytes(&[101_u8; 32]);
    let release_key = SigningKey::from_bytes(&[102_u8; 32]);
    let portable_grant = sign_grant(
        LaunchGrantV1 {
            schema_version: 1,
            generation: 23,
            artifact: artifact.clone(),
            binary_sha256: assembled.manifest().binary_sha256.clone(),
            resource_pack_sha256: content.pack_sha256().to_string(),
            allowed_modes: vec![GrantMode::LiteMcp],
            integration_scopes: vec![
                IntegrationScope::CodexLocal,
                IntegrationScope::ClaudeCodeLocal,
            ],
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        },
        &launch_key,
        "candidate-launch-test-key",
    );
    let envelope =
        build_native_release_envelope(29, &archive, &portable_grant, NOW - 30, NOW + 1_800)
            .expect("portable release envelope must build");
    let release_signature =
        release_key.sign(&native_release_envelope_signing_bytes(&envelope).unwrap());
    let signed_release = SignedNativeReleaseEnvelopeV1 {
        envelope,
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "candidate-release-test-key".to_string(),
            value_hex: encode_hex(&release_signature.to_bytes()),
        },
    };
    let candidate = build_native_release_candidate(
        29,
        SOURCE_COMMIT,
        &signed_release,
        [
            plugin_grant(
                &artifact,
                ClientActivationTarget::Codex,
                &assembled.manifest().binary_sha256,
                content.pack_sha256(),
                &launch_key,
            ),
            plugin_grant(
                &artifact,
                ClientActivationTarget::ClaudeCode,
                &assembled.manifest().binary_sha256,
                content.pack_sha256(),
                &launch_key,
            ),
        ],
        NOTES,
        NOW,
        NOW + 1_200,
    )
    .expect("release candidate must build");
    let candidate_signature =
        release_key.sign(&native_release_candidate_signing_bytes(&candidate).unwrap());
    let signed_candidate = SignedNativeReleaseCandidateV1 {
        candidate,
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "candidate-release-test-key".to_string(),
            value_hex: encode_hex(&candidate_signature.to_bytes()),
        },
    };
    let authority = authority(&release_key, &launch_key);

    let codex_context = NativeReleaseCandidateVerificationContext {
        now_unix: NOW + 1,
        expected_source_commit: SOURCE_COMMIT,
        expected_artifact: &artifact,
        requested_target: ClientActivationTarget::Codex,
    };
    let codex = signed_candidate
        .verify(&authority, &codex_context, &content, &archive_target, NOTES)
        .expect("Codex candidate must verify");
    assert_eq!(codex.target(), ClientActivationTarget::Codex);
    assert_eq!(
        codex.plugin_grant().authorized_scope(),
        IntegrationScope::CodexLocal
    );
    assert_eq!(
        codex.portable_release().archive().archive_sha256(),
        archive.archive_sha256()
    );
    let claude_context = NativeReleaseCandidateVerificationContext {
        requested_target: ClientActivationTarget::ClaudeCode,
        ..codex_context
    };
    let claude = signed_candidate
        .verify(
            &authority,
            &claude_context,
            &content,
            &archive_target,
            NOTES,
        )
        .expect("Claude candidate must verify");
    assert_eq!(claude.target(), ClientActivationTarget::ClaudeCode);
    assert_eq!(
        claude.plugin_grant().authorized_scope(),
        IntegrationScope::ClaudeCodeLocal
    );

    let home = fixture.root.join("home");
    create_private_directory(&home);
    let codex_source =
        prepare_native_candidate_plugin_source_target(&home, ClientActivationTarget::Codex)
            .expect("fixed Codex source target must prepare");
    let codex_materialized = materialize_native_candidate_plugin_source(
        &content,
        &codex,
        &fixture.source_binary,
        &codex_source,
    )
    .expect("Codex candidate source must materialize");
    assert_eq!(
        codex_materialized.disposition,
        NativeCandidatePluginSourceDisposition::Materialized
    );
    let codex_replayed = materialize_native_candidate_plugin_source(
        &content,
        &codex,
        &fixture.source_binary,
        &codex_source,
    )
    .expect("healthy Codex source must replay");
    assert_eq!(
        codex_replayed.disposition,
        NativeCandidatePluginSourceDisposition::AlreadyHealthy
    );
    assert_eq!(
        codex_replayed.verification,
        verify_native_candidate_plugin_source(&codex_source).unwrap()
    );

    let claude_source =
        prepare_native_candidate_plugin_source_target(&home, ClientActivationTarget::ClaudeCode)
            .expect("fixed Claude source target must prepare");
    assert_eq!(
        materialize_native_candidate_plugin_source(
            &content,
            &codex,
            &fixture.source_binary,
            &claude_source,
        )
        .unwrap_err(),
        NativeCandidatePluginSourceError::TargetMismatch
    );
    let claude_materialized = materialize_native_candidate_plugin_source(
        &content,
        &claude,
        &fixture.source_binary,
        &claude_source,
    )
    .expect("Claude candidate source must materialize");
    assert_eq!(
        claude_materialized.disposition,
        NativeCandidatePluginSourceDisposition::Materialized
    );
    let canary = home.join("user-canary");
    fs::write(&canary, b"preserve").unwrap();
    assert_eq!(
        remove_native_candidate_plugin_source(&claude_source).unwrap(),
        claude_materialized.verification
    );
    assert_eq!(
        remove_native_candidate_plugin_source(&codex_source).unwrap(),
        codex_materialized.verification
    );
    assert_eq!(fs::read(canary).unwrap(), b"preserve");

    let codex_install = apply_native_release_candidate_local(&content, &codex, &home, NOW + 2)
        .expect("Codex candidate journey must apply");
    assert_eq!(
        codex_install.payload.disposition,
        InstallDisposition::Applied
    );
    assert_eq!(
        codex_install.source.disposition,
        NativeCandidatePluginSourceDisposition::Materialized
    );
    assert!(matches!(
        codex_install.registration,
        NativeCandidateRegistrationCommit::Codex(ref commit)
            if commit.disposition == CodexRegistrationDisposition::Registered
    ));
    let codex_replay = apply_native_release_candidate_local(&content, &codex, &home, NOW + 3)
        .expect("Codex candidate journey must replay");
    assert_eq!(
        codex_replay.payload.disposition,
        InstallDisposition::AlreadyApplied
    );
    assert_eq!(
        codex_replay.source.disposition,
        NativeCandidatePluginSourceDisposition::AlreadyHealthy
    );
    assert!(matches!(
        codex_replay.registration,
        NativeCandidateRegistrationCommit::Codex(ref commit)
            if commit.disposition == CodexRegistrationDisposition::AlreadyRegistered
    ));
    let codex_verified = verify_native_release_candidate_local(
        &content,
        &home,
        ClientActivationTarget::Codex,
        &codex_install.payload.receipt.install_id,
    )
    .unwrap();
    assert_eq!(codex_verified.source, codex_install.source.verification);
    let installed_path = home.join(".qiongli/native/payloads").join(&artifact_id);
    let installed_target = approve_native_artifact_target(&installed_path, &artifact).unwrap();
    let installed_binary = installed_path.join(&assembled.manifest().binary_path);
    let installed = signed_candidate
        .verify_installed(
            &authority,
            &codex_context,
            &content,
            &installed_target,
            &installed_binary,
        )
        .expect("installed candidate must bind the installed executable");
    assert_eq!(
        installed.current_executable(),
        fs::canonicalize(&installed_binary).unwrap()
    );
    assert_eq!(installed.candidate().source_commit, SOURCE_COMMIT);
    let record_path = home
        .join(".qiongli/native/payloads")
        .join(qiongli_platform::native_release_candidate_file_name(&artifact).unwrap());
    let record_bytes = fs::read(&record_path).unwrap();
    assert_eq!(record_bytes, signed_candidate.to_canonical_json().unwrap());
    let verify_product = || {
        verify_installed_native_candidate_product(
            &content,
            &authority,
            &codex_context,
            &home,
            &installed_binary,
        )
    };
    verify_product().expect("persisted candidate and active receipt must revalidate");
    fs::remove_file(&record_path).unwrap();
    assert!(
        verify_product().is_err(),
        "legacy receipts alone cannot authorize a product"
    );
    let upgraded = apply_native_release_candidate_local(&content, &codex, &home, NOW + 3).unwrap();
    assert_eq!(
        upgraded.payload.disposition,
        InstallDisposition::AlreadyApplied
    );
    assert_eq!(fs::read(&record_path).unwrap(), record_bytes);
    let record_link = fixture.root.join("candidate-record-hard-link");
    fs::hard_link(&record_path, &record_link).unwrap();
    assert!(
        verify_product().is_err(),
        "linked authority records must be refused"
    );
    fs::remove_file(record_link).unwrap();

    fs::write(&record_path, b"untrusted candidate canary").unwrap();
    assert!(verify_product().is_err());
    assert!(apply_native_release_candidate_local(&content, &codex, &home, NOW + 3).is_err());
    assert_eq!(
        fs::read(&record_path).unwrap(),
        b"untrusted candidate canary"
    );
    fs::write(&record_path, &record_bytes).unwrap();
    verify_product().expect("restored signed candidate must revalidate freshly");

    assert_eq!(
        installed.plugin_grant().authorized_scope(),
        IntegrationScope::CodexLocal
    );
    assert!(!format!("{installed:?}").contains(fixture.root.to_string_lossy().as_ref()));

    assert!(
        verify_native_release_candidate_local(
            &content,
            &home,
            ClientActivationTarget::ClaudeCode,
            &codex_install.payload.receipt.install_id,
        )
        .is_err()
    );
    let recovery_marker = home
        .join(".qiongli/native/payloads")
        .join(".qiongli-native-payload-transaction.json");
    fs::write(&recovery_marker, b"recovery-canary").unwrap();
    assert!(verify_product().is_err());
    assert_eq!(
        verify_native_release_candidate_local(
            &content,
            &home,
            ClientActivationTarget::Codex,
            &codex_install.payload.receipt.install_id,
        )
        .unwrap_err(),
        NativeCandidateLocalInstallError::Transaction(TransactionError::RecoveryRequired)
    );
    fs::remove_file(recovery_marker).unwrap();
    remove_native_release_candidate_local(
        &content,
        &home,
        ClientActivationTarget::Codex,
        &codex_install.payload.receipt.install_id,
        NOW + 4,
    )
    .unwrap();
    assert!(
        record_path.exists(),
        "signed diagnostic metadata is retained like receipts"
    );
    assert!(
        verify_product().is_err(),
        "removed payload cannot regain authority from retained metadata"
    );

    assert!(
        !home
            .join(".qiongli/native/payloads")
            .join(&artifact_id)
            .exists()
    );

    let claude_home = fixture.root.join("claude-home");
    create_private_directory(&claude_home);
    let claude_install =
        apply_native_release_candidate_local(&content, &claude, &claude_home, NOW + 2)
            .expect("Claude candidate journey must apply");
    assert_eq!(
        claude_install.payload.disposition,
        InstallDisposition::Applied
    );
    assert_eq!(
        claude_install.source.disposition,
        NativeCandidatePluginSourceDisposition::Materialized
    );
    assert!(matches!(
        claude_install.registration,
        NativeCandidateRegistrationCommit::ClaudeCode(ref commit)
            if commit.disposition == ClaudeRegistrationDisposition::Registered
    ));
    verify_native_release_candidate_local(
        &content,
        &claude_home,
        ClientActivationTarget::ClaudeCode,
        &claude_install.payload.receipt.install_id,
    )
    .unwrap();
    let claude_binary = claude_home
        .join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next/bin")
        .join(format!("qiongli{}", std::env::consts::EXE_SUFFIX));
    let healthy_binary = fs::read(&claude_binary).unwrap();
    fs::write(&claude_binary, b"drift").unwrap();
    assert!(
        verify_native_release_candidate_local(
            &content,
            &claude_home,
            ClientActivationTarget::ClaudeCode,
            &claude_install.payload.receipt.install_id,
        )
        .is_err()
    );
    fs::write(&claude_binary, healthy_binary).unwrap();
    verify_native_release_candidate_local(
        &content,
        &claude_home,
        ClientActivationTarget::ClaudeCode,
        &claude_install.payload.receipt.install_id,
    )
    .unwrap();
    remove_native_release_candidate_local(
        &content,
        &claude_home,
        ClientActivationTarget::ClaudeCode,
        &claude_install.payload.receipt.install_id,
        NOW + 4,
    )
    .unwrap();

    let conflict_home = fixture.root.join("conflict-home");
    create_private_directory(&conflict_home);
    let agents = conflict_home.join(".agents");
    create_private_directory(&agents);
    let agent_plugins = agents.join("plugins");
    create_private_directory(&agent_plugins);
    let conflict_marketplace = serde_json::to_vec(&json!({
        "plugins": [{
            "name": "qiongli-next",
            "source": {"source": "local", "path": "./someone-else"},
            "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
            "category": "Education"
        }]
    }))
    .unwrap();
    let marketplace_path = agent_plugins.join("marketplace.json");
    fs::write(&marketplace_path, &conflict_marketplace).unwrap();
    assert_eq!(
        apply_native_release_candidate_local(&content, &codex, &conflict_home, NOW + 6,)
            .unwrap_err(),
        NativeCandidateLocalInstallError::Codex(CodexAdapterError::RegistrationConflict)
    );
    assert!(
        !conflict_home
            .join(".qiongli/native/payloads")
            .join(&artifact_id)
            .exists()
    );
    assert!(
        !conflict_home
            .join(".qiongli/plugins/codex/qiongli-next")
            .exists()
    );
    assert_eq!(fs::read(marketplace_path).unwrap(), conflict_marketplace);

    assert_eq!(
        signed_candidate
            .verify(
                &authority,
                &codex_context,
                &content,
                &archive_target,
                b"tampered notes",
            )
            .unwrap_err(),
        NativeReleaseCandidateError::ReleaseNotesInvalid
    );
    let wrong_source = NativeReleaseCandidateVerificationContext {
        expected_source_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ..codex_context
    };
    assert_eq!(
        signed_candidate
            .verify(&authority, &wrong_source, &content, &archive_target, NOTES,)
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateSourceMismatch
    );

    let mut bad_signature = signed_candidate.clone();
    corrupt_hex(&mut bad_signature.signature.value_hex);
    assert_eq!(
        bad_signature
            .verify(&authority, &codex_context, &content, &archive_target, NOTES,)
            .unwrap_err(),
        NativeReleaseCandidateError::SignatureInvalid
    );

    let not_yet_valid = NativeReleaseCandidateVerificationContext {
        now_unix: NOW - 1,
        ..codex_context
    };
    assert_eq!(
        signed_candidate
            .verify(&authority, &not_yet_valid, &content, &archive_target, NOTES,)
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateNotYetValid
    );
    let expired = NativeReleaseCandidateVerificationContext {
        now_unix: NOW + 1_200,
        ..codex_context
    };
    assert_eq!(
        signed_candidate
            .verify(&authority, &expired, &content, &archive_target, NOTES,)
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateExpired
    );

    let stale_authority = authority_with_policy(&release_key, &launch_key, "alpha", 30, 29, 31);
    assert_eq!(
        signed_candidate
            .verify(
                &stale_authority,
                &codex_context,
                &content,
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateReplayed
    );
    let beta_authority = authority_with_policy(&release_key, &launch_key, "beta", 29, 29, 30);
    assert_eq!(
        signed_candidate
            .verify(
                &beta_authority,
                &codex_context,
                &content,
                &archive_target,
                NOTES,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::CandidateChannelMismatch
    );

    let mut untrusted = signed_candidate.clone();
    untrusted.signature.key_id = "other-release-key".to_string();
    assert_eq!(
        untrusted
            .verify(&authority, &codex_context, &content, &archive_target, NOTES,)
            .unwrap_err(),
        NativeReleaseCandidateError::ReleaseKeyUntrusted
    );

    let mut bad_portable = signed_candidate.clone();
    corrupt_hex(
        &mut bad_portable
            .candidate
            .signed_portable_release
            .signature
            .value_hex,
    );
    let bad_portable = resign_candidate(bad_portable, &release_key);
    assert_eq!(
        bad_portable
            .verify(&authority, &codex_context, &content, &archive_target, NOTES,)
            .unwrap_err(),
        NativeReleaseCandidateError::PortableReleaseInvalid
    );

    let mut bad_plugin = signed_candidate.clone();
    corrupt_hex(
        &mut bad_plugin.candidate.client_plugins[0]
            .signed_launch_grant
            .signature
            .value_hex,
    );
    let bad_plugin = resign_candidate(bad_plugin, &release_key);
    assert_eq!(
        bad_plugin
            .verify(&authority, &codex_context, &content, &archive_target, NOTES,)
            .unwrap_err(),
        NativeReleaseCandidateError::PluginGrantInvalid
    );

    // Runtime identity does not require the download or release-note bytes.
    fs::remove_file(&archive_path).unwrap();
    let executable = artifact_path.join(&assembled.manifest().binary_path);
    for context in [codex_context, claude_context] {
        let installed = signed_candidate
            .verify_installed(
                &authority,
                &context,
                &content,
                &artifact_target,
                &executable,
            )
            .unwrap();
        assert_eq!(
            installed.plugin_grant().authorized_scope(),
            context.requested_target.integration_scope()
        );
    }
    for (candidate, policy, context, expected) in [
        (
            &signed_candidate,
            &authority,
            &wrong_source,
            NativeReleaseCandidateError::CandidateSourceMismatch,
        ),
        (
            &bad_signature,
            &authority,
            &codex_context,
            NativeReleaseCandidateError::SignatureInvalid,
        ),
        (
            &signed_candidate,
            &authority,
            &not_yet_valid,
            NativeReleaseCandidateError::CandidateNotYetValid,
        ),
        (
            &signed_candidate,
            &authority,
            &expired,
            NativeReleaseCandidateError::CandidateExpired,
        ),
        (
            &signed_candidate,
            &stale_authority,
            &codex_context,
            NativeReleaseCandidateError::CandidateReplayed,
        ),
        (
            &signed_candidate,
            &beta_authority,
            &codex_context,
            NativeReleaseCandidateError::CandidateChannelMismatch,
        ),
        (
            &untrusted,
            &authority,
            &codex_context,
            NativeReleaseCandidateError::ReleaseKeyUntrusted,
        ),
        (
            &bad_portable,
            &authority,
            &codex_context,
            NativeReleaseCandidateError::PortableReleaseInvalid,
        ),
        (
            &bad_plugin,
            &authority,
            &codex_context,
            NativeReleaseCandidateError::PluginGrantInvalid,
        ),
    ] {
        assert_eq!(
            candidate
                .verify_installed(policy, context, &content, &artifact_target, &executable)
                .unwrap_err(),
            expected
        );
    }
    assert_eq!(
        signed_candidate
            .verify_installed(
                &authority,
                &codex_context,
                &content,
                &artifact_target,
                &fixture.source_binary,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::ExecutableInvalid
    );
    let original = fs::read(&executable).unwrap();
    fs::write(&executable, b"tampered installed executable").unwrap();
    assert_eq!(
        signed_candidate
            .verify_installed(
                &authority,
                &codex_context,
                &content,
                &artifact_target,
                &executable,
            )
            .unwrap_err(),
        NativeReleaseCandidateError::PortableReleaseInvalid
    );
    fs::write(&executable, original).unwrap();

    let debug = format!("{codex:?}");
    assert!(!debug.contains(fixture.root.to_string_lossy().as_ref()));
    assert!(!debug.contains(&signed_candidate.signature.value_hex));
}
