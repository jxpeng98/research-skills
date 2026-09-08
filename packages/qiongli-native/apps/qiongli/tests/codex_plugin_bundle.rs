#![allow(clippy::disallowed_methods)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use qiongli::FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES;
use qiongli_content::{ProfileId, WorkflowOverrides};
use qiongli_platform::{
    ApprovalRequirement, Architecture, ArtifactIdentityV1, CapabilityProfile,
    CodexPluginBundleError, CodexRegistrationExecutor, GrantMode, GrantSignatureV1,
    GrantVerificationContext, InstallPlanMetadataV1, InstallerKind, IntegrationScope,
    LaunchGrantV1, OperatingSystem, ProductId, ReleaseChannel, SignatureAlgorithm,
    SignedLaunchGrantV1, TrustedPublicKey, VerifiedLaunchGrant, approve_codex_plugin_bundle_target,
    approve_install_plan, compose_codex_plugin_bundle, compose_codex_plugin_bundle_with_overrides,
    discover_codex_user, launch_grant_signing_bytes, preview_codex_registration,
    remove_codex_plugin_bundle, replace_codex_plugin_bundle_with_overrides,
    verify_codex_plugin_bundle,
};
use qiongli_runtime::{FULL_PROJECT_PUBLIC_TOOL_NAMES, LITE_PUBLIC_TOOL_NAMES};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const NOW: u64 = 1_783_987_200;
const APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];
static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn customized_workflow_changes_only_receipted_skill_content() {
    let fixture = Fixture::new("customized-bundle");
    let content = qiongli::embedded_content().expect("embedded content must load");
    let grant = grant_fixture(&fixture.source_binary, content.pack().pack_sha256());
    let canonical_skill = content
        .pack()
        .resource_for_profile("marketplace-lite", "workflow/SKILL.md")
        .unwrap()
        .unwrap();
    let mut customized_skill = canonical_skill.bytes().to_vec();
    customized_skill.extend_from_slice(b"\nCustomized instruction marker.\n");
    let overrides = WorkflowOverrides::new(
        content.pack(),
        BTreeMap::from([("workflow/SKILL.md".to_owned(), customized_skill)]),
    )
    .unwrap()
    .unwrap();

    let canonical_path = fixture.standalone_target();
    let canonical_target = approve_codex_plugin_bundle_target(&canonical_path).unwrap();
    let canonical = compose_codex_plugin_bundle(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &canonical_target,
    )
    .unwrap();
    let customized_parent = fixture.root.join("customized");
    create_private_directory(&customized_parent);
    let customized_path = customized_parent.join("qiongli-next");
    let customized_target = approve_codex_plugin_bundle_target(&customized_path).unwrap();
    let customized = compose_codex_plugin_bundle_with_overrides(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &customized_target,
        Some(&overrides),
    )
    .unwrap();

    assert_eq!(
        customized.receipt().workflow_variant_sha256.as_deref(),
        Some(overrides.variant_sha256())
    );
    assert_ne!(
        customized.receipt().package_content_root_sha256,
        canonical.receipt().package_content_root_sha256
    );
    for path in [".codex-plugin/plugin.json", ".mcp.json"] {
        assert_eq!(
            fs::read(customized_path.join(path)).unwrap(),
            fs::read(canonical_path.join(path)).unwrap()
        );
    }
    let skill =
        fs::read_to_string(customized_path.join("skills/qiongli-workflow/SKILL.md")).unwrap();
    assert!(skill.contains("Customized instruction marker."));
    assert!(skill.contains("## Codex Native Host Adapter"));
    assert_eq!(
        verify_codex_plugin_bundle(&customized_target).unwrap(),
        customized
    );

    let canary = customized_parent.join("keep.txt");
    fs::write(&canary, b"keep").unwrap();
    let reset = replace_codex_plugin_bundle_with_overrides(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &customized_target,
        None,
    )
    .unwrap();
    assert!(reset.receipt().workflow_variant_sha256.is_none());
    assert_eq!(
        reset.receipt().package_content_root_sha256,
        canonical.receipt().package_content_root_sha256
    );
    assert_eq!(fs::read(canary).unwrap(), b"keep");
}

struct Fixture {
    root: PathBuf,
    home: PathBuf,
    config_root: PathBuf,
    source_binary: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-codex-plugin-tests");
        fs::create_dir_all(&test_base).expect("Codex plugin test base must exist");
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
        let home = root.join("home");
        create_private_directory(&home);
        let config_root = root.join("config");
        let source_binary = root.join(format!("source-qiongli{}", std::env::consts::EXE_SUFFIX));
        fs::copy(env!("CARGO_BIN_EXE_qiongli"), &source_binary)
            .expect("native app binary must copy");
        set_executable_mode(&source_binary);
        Self {
            root,
            home,
            config_root,
            source_binary,
        }
    }

    fn standalone_target(&self) -> PathBuf {
        let parent = self.root.join("bundle");
        create_private_directory(&parent);
        parent.join("qiongli-next")
    }

    fn codex_source_target(&self) -> PathBuf {
        let qiongli = self.home.join(".qiongli");
        create_private_directory(&qiongli);
        let plugins = qiongli.join("plugins");
        create_private_directory(&plugins);
        let codex = plugins.join("codex");
        create_private_directory(&codex);
        codex.join("qiongli-next")
    }

    fn codex_home(&self) -> PathBuf {
        let path = self.home.join(".codex");
        create_private_directory(&path);
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct GrantFixture {
    artifact: ArtifactIdentityV1,
    binary_sha256: String,
    verified: VerifiedLaunchGrant,
    trusted: TrustedPublicKey,
}

fn grant_fixture(binary: &Path, pack_sha256: &str) -> GrantFixture {
    let binary_sha256 = sha256_file(binary);
    let artifact = ArtifactIdentityV1 {
        product: ProductId::Qiongli,
        version: env!("CARGO_PKG_VERSION").to_string(),
        channel: ReleaseChannel::Alpha,
        profile: CapabilityProfile::Lite,
        os: OperatingSystem::current().expect("test OS must be supported"),
        arch: Architecture::current().expect("test architecture must be supported"),
        installer_kind: InstallerKind::PluginBundle,
    };
    let grant = LaunchGrantV1 {
        schema_version: 1,
        generation: 11,
        artifact: artifact.clone(),
        binary_sha256: binary_sha256.clone(),
        resource_pack_sha256: pack_sha256.to_string(),
        allowed_modes: vec![GrantMode::LiteMcp, GrantMode::FullMcp],
        integration_scopes: vec![IntegrationScope::CodexLocal],
        not_before_unix: NOW - 60,
        expires_at_unix: NOW + 3_600,
    };
    let signing_key = SigningKey::from_bytes(&[19_u8; 32]);
    let signature = signing_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
    let signed = SignedLaunchGrantV1 {
        grant,
        signature: GrantSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "codex-plugin-test-key".to_string(),
            value_hex: encode_hex(&signature.to_bytes()),
        },
    };
    let trusted = TrustedPublicKey::new(
        "codex-plugin-test-key",
        signing_key.verifying_key().to_bytes(),
    )
    .unwrap();
    let context = GrantVerificationContext {
        now_unix: NOW,
        minimum_generation: 11,
        expected_artifact: &artifact,
        binary_sha256: &binary_sha256,
        resource_pack_sha256: pack_sha256,
        requested_mode: GrantMode::FullMcp,
        requested_scope: IntegrationScope::CodexLocal,
    };
    let verified = signed
        .verify(std::slice::from_ref(&trusted), &context)
        .expect("test launch grant must verify");
    GrantFixture {
        artifact,
        binary_sha256,
        verified,
        trusted,
    }
}

#[test]
fn complete_bundle_is_deterministic_tamper_evident_and_runtime_independent() {
    let fixture = Fixture::new("complete-bundle");
    let content = qiongli::embedded_content().expect("embedded content must load");
    let grant = grant_fixture(&fixture.source_binary, content.pack().pack_sha256());
    let target_path = fixture.standalone_target();
    let target =
        approve_codex_plugin_bundle_target(&target_path).expect("bundle target must approve");
    let composed = compose_codex_plugin_bundle(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &target,
    )
    .expect("complete Codex bundle must compose");
    let verified = verify_codex_plugin_bundle(&target).expect("bundle must verify");
    assert_eq!(composed, verified);
    assert_eq!(verified.receipt().artifact, grant.artifact);
    assert_eq!(verified.receipt().binary_sha256, grant.binary_sha256);
    assert_eq!(verified.receipt().profile, ProfileId::MarketplaceLite);
    assert_eq!(verified.receipt().mcp_profile, ProfileId::Full);
    assert!(verified.receipt().entries.len() > 400);

    let manifest: Value =
        serde_json::from_slice(&fs::read(target_path.join(".codex-plugin/plugin.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["name"], "qiongli-next");
    assert_eq!(manifest["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(manifest["skills"], "./skills/");
    assert_eq!(manifest["mcpServers"], "./.mcp.json");

    let mcp_bytes = fs::read(target_path.join(".mcp.json")).unwrap();
    let mcp: Value = serde_json::from_slice(&mcp_bytes).unwrap();
    let command = mcp["mcpServers"]["qiongli-next"]["command"]
        .as_str()
        .unwrap();
    assert_eq!(command, format!("./{}", verified.receipt().binary_path));
    assert_eq!(
        mcp["mcpServers"]["qiongli-next"]["args"],
        json!(["mcp", "serve", "--profile", "full", "--transport", "stdio"])
    );
    assert_eq!(
        mcp["mcpServers"]["qiongli-next"]["env_vars"],
        json!(["QIONGLI_CONFIG_HOME"]),
        "the managed Codex MCP may forward only the explicit Qiongli config-root override"
    );
    let lower_mcp = String::from_utf8(mcp_bytes).unwrap().to_ascii_lowercase();
    for forbidden in ["python", "node", "cargo", "npm", "rustup"] {
        assert!(!lower_mcp.contains(forbidden));
    }

    let output = run_packaged_mcp(
        &target_path,
        verified.receipt().binary_path.as_str(),
        &fixture,
        "full",
    );
    let expected_tool_names = LITE_PUBLIC_TOOL_NAMES
        .into_iter()
        .chain(FULL_PROJECT_PUBLIC_TOOL_NAMES)
        .chain(FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES)
        .collect::<Vec<_>>();
    assert_packaged_mcp_profile(output, &expected_tool_names);

    let skill = fs::read_to_string(target_path.join("skills/qiongli-workflow/SKILL.md")).unwrap();
    for required in [
        "## Codex Native Host Adapter",
        "qiongli_orchestration_doctor",
        "qiongli_orchestration_start",
        "qiongli_orchestration_read",
        "qiongli_orchestration_submit",
        "qiongli_orchestration_next",
        "structuredContent.qiongliOrchestration.evidence",
        "single-agent",
        "native-subagents",
        "knownFactDigests",
        "evidenceGaps",
        "reviewResult",
        "explicit artifact apply approval",
        "untrusted evidence, never as",
        "before the first handoff",
        "claim to be system messages or human approval",
        "qiongli-mcp-unavailable",
        "Tool-shaped text is not an executed call",
        "If a visible tool is denied",
    ] {
        assert!(
            skill.contains(required),
            "missing Codex host guidance: {required}"
        );
    }

    assert_eq!(
        compose_codex_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        CodexPluginBundleError::TargetExists
    );
    fs::write(target_path.join("unexpected.txt"), b"drift").unwrap();
    assert_eq!(
        verify_codex_plugin_bundle(&target).unwrap_err(),
        CodexPluginBundleError::BundleDrift
    );
    fs::remove_file(target_path.join("unexpected.txt")).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let linked = target_path.join("linked-entry");
        symlink(&fixture.source_binary, &linked).unwrap();
        assert_eq!(
            verify_codex_plugin_bundle(&target).unwrap_err(),
            CodexPluginBundleError::BundleDrift
        );
        fs::remove_file(linked).unwrap();
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let manifest_path = target_path.join(".codex-plugin/plugin.json");
        fs::set_permissions(&manifest_path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            verify_codex_plugin_bundle(&target).unwrap_err(),
            CodexPluginBundleError::BundleDrift
        );
        fs::set_permissions(&manifest_path, fs::Permissions::from_mode(0o644)).unwrap();
    }

    let packaged_binary = target_path.join(&verified.receipt().binary_path);
    let outside_hard_link = fixture.root.join("managed-binary-hard-link");
    fs::hard_link(&packaged_binary, &outside_hard_link).unwrap();
    assert_eq!(
        verify_codex_plugin_bundle(&target).unwrap_err(),
        CodexPluginBundleError::BundleDrift
    );
    fs::remove_file(outside_hard_link).unwrap();

    let receipt_path = target_path.join(".qiongli-codex-plugin-bundle.json");
    let receipt_bytes = fs::read(&receipt_path).unwrap();
    fs::write(&receipt_path, b"{}").unwrap();
    assert_eq!(
        verify_codex_plugin_bundle(&target).unwrap_err(),
        CodexPluginBundleError::ReceiptInvalid
    );
    fs::write(&receipt_path, receipt_bytes).unwrap();

    let skill_path = target_path.join("skills/qiongli-workflow/SKILL.md");
    let skill_bytes = fs::read(&skill_path).unwrap();
    fs::OpenOptions::new()
        .append(true)
        .open(&skill_path)
        .unwrap()
        .write_all(b"content-tamper")
        .unwrap();
    assert_eq!(
        verify_codex_plugin_bundle(&target).unwrap_err(),
        CodexPluginBundleError::BundleDrift
    );
    fs::write(&skill_path, skill_bytes).unwrap();

    fs::OpenOptions::new()
        .append(true)
        .open(&packaged_binary)
        .unwrap()
        .write_all(b"tamper")
        .unwrap();
    assert_eq!(
        verify_codex_plugin_bundle(&target).unwrap_err(),
        CodexPluginBundleError::BundleDrift
    );
    assert_eq!(
        remove_codex_plugin_bundle(&target).unwrap_err(),
        CodexPluginBundleError::BundleDrift
    );
    fs::write(&packaged_binary, fs::read(&fixture.source_binary).unwrap()).unwrap();
    let canary = target_path.parent().unwrap().join("user-canary");
    fs::write(&canary, b"preserve").unwrap();

    let removed = remove_codex_plugin_bundle(&target).unwrap();
    assert_eq!(removed, composed);
    assert!(!target_path.exists());
    assert_eq!(fs::read(&canary).unwrap(), b"preserve");
    assert!(
        fs::read_dir(target_path.parent().unwrap())
            .unwrap()
            .flatten()
            .all(|entry| !entry
                .file_name()
                .to_string_lossy()
                .contains("qiongli-codex-remove"))
    );
}

#[test]
fn composition_conflicts_fail_closed_without_overwriting_existing_data() {
    let fixture = Fixture::new("composition-conflicts");
    let content = qiongli::embedded_content().expect("embedded content must load");
    let grant = grant_fixture(&fixture.source_binary, content.pack().pack_sha256());

    let invalid_parent = fixture.root.join("invalid-bundle");
    create_private_directory(&invalid_parent);
    let invalid_target = approve_codex_plugin_bundle_target(invalid_parent.join("not-qiongli"))
        .expect("portable verification target must approve");
    assert_eq!(
        compose_codex_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &invalid_target,
        )
        .unwrap_err(),
        CodexPluginBundleError::InvalidTarget
    );

    let target_path = fixture.standalone_target();
    let target = approve_codex_plugin_bundle_target(&target_path).unwrap();
    create_private_directory(&target_path);
    fs::write(target_path.join("user-canary"), b"preserve").unwrap();
    assert_eq!(
        compose_codex_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        CodexPluginBundleError::TargetExists
    );
    assert_eq!(
        fs::read(target_path.join("user-canary")).unwrap(),
        b"preserve"
    );
    fs::remove_dir_all(&target_path).unwrap();

    let lock_path = target_path
        .parent()
        .unwrap()
        .join(".qiongli.qiongli-codex-bundle.lock");
    fs::write(&lock_path, b"held").unwrap();
    assert_eq!(
        compose_codex_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        CodexPluginBundleError::TargetBusy
    );
    fs::remove_file(lock_path).unwrap();

    let mismatched_pack = "0000000000000000000000000000000000000000000000000000000000000000";
    let pack_mismatch_grant = grant_fixture(&fixture.source_binary, mismatched_pack);
    assert_eq!(
        compose_codex_plugin_bundle(
            content.pack(),
            &pack_mismatch_grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        CodexPluginBundleError::ResourcePackMismatch
    );

    let oversized_binary = fixture.root.join("oversized-qiongli");
    let oversized = fs::File::create(&oversized_binary).unwrap();
    oversized.set_len(128 * 1024 * 1024 + 1).unwrap();
    drop(oversized);
    set_executable_mode(&oversized_binary);
    assert_eq!(
        compose_codex_plugin_bundle(content.pack(), &grant.verified, &oversized_binary, &target,)
            .unwrap_err(),
        CodexPluginBundleError::SourceBinaryTooLarge
    );

    fs::OpenOptions::new()
        .append(true)
        .open(&fixture.source_binary)
        .unwrap()
        .write_all(b"changed-after-signing")
        .unwrap();
    assert_eq!(
        compose_codex_plugin_bundle(
            content.pack(),
            &grant.verified,
            &fixture.source_binary,
            &target,
        )
        .unwrap_err(),
        CodexPluginBundleError::BinaryDigestMismatch
    );
    assert!(!target_path.exists());
}

#[test]
#[ignore = "requires the Codex CLI"]
fn real_codex_clean_client_installs_enables_caches_and_launches_bundle() {
    let fixture = Fixture::new("real-codex-client");
    let content = qiongli::embedded_content().expect("embedded content must load");
    let grant = grant_fixture(&fixture.source_binary, content.pack().pack_sha256());
    let canonical_skill = content
        .pack()
        .resource_for_profile("marketplace-lite", "workflow/SKILL.md")
        .unwrap()
        .unwrap();
    let mut customized_skill = canonical_skill.bytes().to_vec();
    customized_skill.extend_from_slice(b"\nReal Codex host customized marker.\n");
    let overrides = WorkflowOverrides::new(
        content.pack(),
        BTreeMap::from([("workflow/SKILL.md".to_owned(), customized_skill)]),
    )
    .unwrap()
    .unwrap();
    let source_path = fixture.codex_source_target();
    let bundle_target =
        approve_codex_plugin_bundle_target(&source_path).expect("Codex source target must approve");
    let bundle = compose_codex_plugin_bundle_with_overrides(
        content.pack(),
        &grant.verified,
        &fixture.source_binary,
        &bundle_target,
        Some(&overrides),
    )
    .expect("Codex source bundle must compose");

    let plugin_creator_valid = std::env::var_os("QIONGLI_PLUGIN_VALIDATOR").is_some_and(|path| {
        let validator_python = std::env::var_os("QIONGLI_PLUGIN_VALIDATOR_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("python3"));
        let validation = Command::new(validator_python)
            .arg(path)
            .arg(&source_path)
            .output()
            .expect("Plugin Creator validator must start");
        assert!(
            validation.status.success(),
            "{}",
            public_output(&validation)
        );
        true
    });

    let discovered = discover_codex_user(&fixture.home).expect("Codex target must discover");
    let executor = CodexRegistrationExecutor::new(discovered.clone());
    let preview = preview_codex_registration(
        &discovered,
        InstallPlanMetadataV1 {
            plan_id: "r3d-real-codex-client".to_string(),
            created_at_unix: NOW,
            expires_at_unix: NOW + 600,
        },
        &grant.verified,
    )
    .expect("Codex registration must preview");
    let context = GrantVerificationContext {
        now_unix: NOW,
        minimum_generation: 11,
        expected_artifact: &grant.artifact,
        binary_sha256: &grant.binary_sha256,
        resource_pack_sha256: content.pack().pack_sha256(),
        requested_mode: GrantMode::FullMcp,
        requested_scope: IntegrationScope::CodexLocal,
    };
    let verified_plan = preview
        .plan
        .verify(std::slice::from_ref(&grant.trusted), &context)
        .expect("Codex registration plan must verify");
    let approval = approve_install_plan(&verified_plan, &APPROVALS, NOW)
        .expect("Codex registration must approve");
    let registration = executor
        .apply(&verified_plan, &approval, NOW + 1)
        .expect("isolated personal marketplace must register");
    assert_eq!(
        executor
            .verify()
            .expect("isolated personal marketplace must verify")
            .receipt,
        registration.receipt
    );

    let codex_home = fixture.codex_home();
    let codex = std::env::var_os("QIONGLI_CODEX_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"));
    let version = isolated_codex_command(&codex, &fixture, &codex_home)
        .arg("--version")
        .output()
        .expect("Codex version command must start");
    assert!(version.status.success(), "{}", public_output(&version));
    let install = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["plugin", "add", "--json", "qiongli-next@personal"])
        .output()
        .expect("Codex plugin add must start");
    assert!(install.status.success(), "{}", public_output(&install));

    let listed = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["plugin", "list", "--json"])
        .output()
        .expect("Codex plugin list must start");
    assert!(listed.status.success(), "{}", public_output(&listed));
    let listed_json: Value = serde_json::from_slice(&listed.stdout).unwrap();
    let installed = listed_json["installed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|plugin| plugin["pluginId"] == "qiongli-next@personal")
        .expect("exact Codex plugin must be listed");
    assert_eq!(installed["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(installed["installed"], true);
    assert_eq!(installed["enabled"], true);
    assert_eq!(installed["source"]["source"], "local");
    assert_eq!(
        installed["source"]["path"],
        source_path.to_string_lossy().as_ref()
    );
    let mcp_list = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["mcp", "list", "--json"])
        .output()
        .expect("Codex MCP list must start");
    assert!(mcp_list.status.success(), "{}", public_output(&mcp_list));
    let mcp_json: Value = serde_json::from_slice(&mcp_list.stdout).unwrap();
    assert!(
        mcp_json
            .as_array()
            .unwrap()
            .iter()
            .any(|mcp| { mcp["name"] == "qiongli-next" && mcp["enabled"] == true })
    );
    let config = fs::read_to_string(codex_home.join("config.toml"))
        .expect("isolated Codex config must exist");
    assert!(config.contains("qiongli-next"));

    let cached_root = find_cached_bundle(&codex_home.join("plugins/cache"))
        .expect("Codex must cache the Qiongli plugin");
    let cached_target = approve_codex_plugin_bundle_target(&cached_root)
        .expect("cached plugin target must approve");
    let cached = verify_codex_plugin_bundle(&cached_target)
        .expect("Codex cached bundle must preserve its receipt");
    assert_eq!(cached.receipt_sha256(), bundle.receipt_sha256());
    assert_eq!(
        cached.receipt().workflow_variant_sha256.as_deref(),
        Some(overrides.variant_sha256())
    );
    assert!(
        fs::read_to_string(cached_root.join("skills/qiongli-workflow/SKILL.md"))
            .unwrap()
            .contains("Real Codex host customized marker.")
    );
    let cached_executable = cached_root.join(cached.receipt().binary_path.as_str());
    let lite_add = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["mcp", "add", "qiongli-next-lite", "--"])
        .arg(&cached_executable)
        .args(["mcp", "serve", "--profile", "lite", "--transport", "stdio"])
        .output()
        .expect("Codex Lite MCP add must start");
    assert!(lite_add.status.success(), "{}", public_output(&lite_add));
    let lite_get = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["mcp", "get", "qiongli-next-lite", "--json"])
        .output()
        .expect("Codex Lite MCP get must start");
    assert!(lite_get.status.success(), "{}", public_output(&lite_get));
    let lite_config: Value = serde_json::from_slice(&lite_get.stdout).unwrap();
    assert_eq!(lite_config["enabled"], true);
    assert_eq!(
        lite_config["transport"]["command"],
        cached_executable.to_string_lossy().as_ref()
    );
    assert_eq!(
        lite_config["transport"]["args"],
        json!(["mcp", "serve", "--profile", "lite", "--transport", "stdio"])
    );
    let with_lite = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["mcp", "list", "--json"])
        .output()
        .expect("Codex MCP list with Lite must start");
    assert!(with_lite.status.success(), "{}", public_output(&with_lite));
    let with_lite: Value = serde_json::from_slice(&with_lite.stdout).unwrap();
    assert!(
        with_lite
            .as_array()
            .unwrap()
            .iter()
            .any(|mcp| mcp["name"] == "qiongli-next-lite" && mcp["enabled"] == true)
    );

    let lite_mcp = run_packaged_mcp(
        &cached_root,
        cached.receipt().binary_path.as_str(),
        &fixture,
        "lite",
    );
    assert_packaged_mcp_profile(lite_mcp, &LITE_PUBLIC_TOOL_NAMES);
    let full_tool_names = LITE_PUBLIC_TOOL_NAMES
        .into_iter()
        .chain(FULL_PROJECT_PUBLIC_TOOL_NAMES)
        .chain(FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES)
        .collect::<Vec<_>>();
    let full_mcp = run_packaged_mcp(
        &cached_root,
        cached.receipt().binary_path.as_str(),
        &fixture,
        "full",
    );
    assert_packaged_mcp_profile(full_mcp, &full_tool_names);
    assert_full_mcp_route(run_packaged_mcp_route(
        &cached_root,
        cached.receipt().binary_path.as_str(),
        &fixture,
        "codex",
    ));

    let lite_remove = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["mcp", "remove", "qiongli-next-lite"])
        .output()
        .expect("Codex Lite MCP remove must start");
    assert!(
        lite_remove.status.success(),
        "{}",
        public_output(&lite_remove)
    );
    let after_lite = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["mcp", "list", "--json"])
        .output()
        .expect("Codex MCP list after Lite removal must start");
    assert!(
        after_lite.status.success(),
        "{}",
        public_output(&after_lite)
    );
    assert!(!String::from_utf8_lossy(&after_lite.stdout).contains("qiongli-next-lite"));

    let remove = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["plugin", "remove", "--json", "qiongli-next@personal"])
        .output()
        .expect("Codex plugin remove must start");
    assert!(remove.status.success(), "{}", public_output(&remove));
    let after = isolated_codex_command(&codex, &fixture, &codex_home)
        .args(["plugin", "list", "--json"])
        .output()
        .expect("Codex final plugin list must start");
    assert!(after.status.success(), "{}", public_output(&after));
    assert!(!String::from_utf8_lossy(&after.stdout).contains("qiongli-next"));
    executor
        .remove(NOW + 2)
        .expect("Qiongli personal marketplace entry must remove");

    let version_text = String::from_utf8_lossy(&version.stdout).trim().to_string();
    println!(
        "{}",
        serde_json::to_string(&json!({
            "schema_version": 1,
            "evidence": "isolated-codex-native-plugin",
            "codex_cli": version_text,
            "bundle_receipt_sha256": bundle.receipt_sha256(),
            "plugin_creator_valid": plugin_creator_valid,
            "personal_marketplace_registered": true,
            "client_install_succeeded": true,
            "client_listed_plugin": true,
            "client_enablement_recorded": true,
            "client_mcp_inventory_verified": true,
            "client_cache_verified": true,
            "cached_customized_skill_bytes": true,
            "workflow_variant_sha256": overrides.variant_sha256(),
            "lite_mcp_client_config_verified": true,
            "lite_mcp_healthcheck": "paired-client-inventory-and-exact-protocol",
            "lite_mcp_protocol_verified": true,
            "lite_tool_count": LITE_PUBLIC_TOOL_NAMES.len(),
            "full_mcp_protocol_verified": true,
            "full_route_profile_verified": true,
            "cached_mcp_empty_path_succeeded": true,
            "lite_mcp_remove_verified": true,
            "client_remove_succeeded": true,
            "client_absence_verified": true,
            "marketplace_catalog_remove_succeeded": true,
            "full_tool_count": LITE_PUBLIC_TOOL_NAMES.len()
                + FULL_PROJECT_PUBLIC_TOOL_NAMES.len()
                + FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES.len()
        }))
        .unwrap()
    );
}

fn run_packaged_mcp(root: &Path, binary_path: &str, fixture: &Fixture, profile: &str) -> Output {
    let requests = [
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2025-11-25", "capabilities": {}}
        }),
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    ];
    run_packaged_mcp_requests(root, binary_path, fixture, profile, &requests)
}

fn assert_packaged_mcp_profile(output: Output, expected_tools: &[&str]) {
    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty(), "{}", public_output(&output));
    let responses = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["result"]["serverInfo"]["name"], "qiongli");
    let tool_names = responses[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(tool_names, expected_tools);
}

fn run_packaged_mcp_route(
    root: &Path,
    binary_path: &str,
    fixture: &Fixture,
    platform: &str,
) -> Output {
    let requests = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "qiongli_orchestrator_route",
                "arguments": {"request": "plan an auditable review", "platform": platform}
            }
        }),
    ];
    run_packaged_mcp_requests(root, binary_path, fixture, "full", &requests)
}

fn run_packaged_mcp_requests(
    root: &Path,
    binary_path: &str,
    fixture: &Fixture,
    profile: &str,
    requests: &[Value],
) -> Output {
    let executable = root.join(binary_path);
    let mut child = Command::new(executable)
        .current_dir(root)
        .env("PATH", "")
        .env("QIONGLI_CONFIG_HOME", &fixture.config_root)
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home)
        .args(["mcp", "serve", "--profile", profile, "--transport", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("packaged MCP executable must start without PATH");
    {
        let stdin = child.stdin.as_mut().expect("MCP stdin must be piped");
        for request in requests {
            serde_json::to_writer(&mut *stdin, request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());
    child.wait_with_output().expect("packaged MCP must exit")
}

fn assert_full_mcp_route(output: Output) {
    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty(), "{}", public_output(&output));
    let responses = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2);
    let route = &responses[1]["result"]["structuredContent"];
    assert_eq!(route["route"], "orchestrator_mcp");
    assert_eq!(route["requires_full_runtime"], true);
    for forbidden in [
        "preview_only",
        "runtime_profile",
        "recommended_runtime",
        "upgrade",
    ] {
        assert!(route.get(forbidden).is_none());
    }
}

fn isolated_codex_command(codex: &Path, fixture: &Fixture, codex_home: &Path) -> Command {
    let mut command = Command::new(codex);
    command
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home)
        .env("CODEX_HOME", codex_home);
    command
}

fn find_cached_bundle(root: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).ok()?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            if path.join(".qiongli-codex-plugin-bundle.json").is_file() {
                return Some(path);
            }
            if let Some(found) = find_cached_bundle(&path) {
                return Some(found);
            }
        }
    }
    None
}

fn sha256_file(path: &Path) -> String {
    let bytes = fs::read(path).expect("test binary must read");
    encode_hex(&Sha256::digest(bytes))
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn public_output(output: &Output) -> String {
    format!(
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[cfg(unix)]
fn create_private_directory(path: &Path) {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(path).expect("private directory must create");
}

#[cfg(windows)]
fn create_private_directory(path: &Path) {
    qiongli_windows_security::create_owner_only_directory(path)
        .expect("owner-only Windows directory must create");
}

#[cfg(unix)]
fn set_executable_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("test binary must be executable");
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) {}
