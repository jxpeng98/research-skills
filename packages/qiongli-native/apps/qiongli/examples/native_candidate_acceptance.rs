#![allow(clippy::disallowed_methods)]

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, Metadata};
use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use qiongli::FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES;
use qiongli_platform::{
    Architecture, ArtifactIdentityV1, CLAUDE_PLUGIN_BUNDLE_RECEIPT_FILE,
    CODEX_PLUGIN_BUNDLE_RECEIPT_FILE, ClientActivationTarget, GrantMode, GrantSignatureV1,
    InstallerKind, IntegrationScope, LaunchGrantV1, MAX_NATIVE_RELEASE_NOTES_BYTES,
    NativeClientPluginGrantV1, NativeReleaseAuthority, NativeReleaseSignatureV1, OperatingSystem,
    ReleaseChannel, SignatureAlgorithm, SignedLaunchGrantV1, SignedNativeReleaseCandidateV1,
    SignedNativeReleaseEnvelopeV1, approve_claude_plugin_bundle_target,
    approve_codex_plugin_bundle_target, approve_native_artifact_target,
    approve_native_portable_archive_target, build_native_release_candidate,
    build_native_release_envelope, compose_native_artifact, compose_native_portable_archive,
    current_target_native_artifact_identity, discover_native_candidate_managed_root,
    extract_native_portable_archive, launch_grant_signing_bytes, native_artifact_binary_path,
    native_artifact_id, native_portable_archive_file_name, native_release_candidate_file_name,
    native_release_candidate_signing_bytes, native_release_envelope_signing_bytes,
    native_release_notes_file_name, verify_claude_plugin_bundle, verify_codex_plugin_bundle,
};
use qiongli_runtime::{FULL_PROJECT_PUBLIC_TOOL_NAMES, LITE_PUBLIC_TOOL_NAMES};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const RELEASE_KEY_ID: &str = "community-alpha-acceptance-release-key";
const LAUNCH_KEY_ID: &str = "community-alpha-acceptance-launch-key";
const GENERATION: u64 = 2;
const RELEASE_VALIDITY_SECONDS: u64 = 3_600;
const CANDIDATE_VALIDITY_SECONDS: u64 = 1_800;
const MAX_STAGED_PRODUCT_BYTES: u64 = 128 * 1024 * 1024;
const RELEASE_NOTES_TEMPLATE: &str = include_str!("native_community_alpha_release_notes.md.tmpl");

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let arguments = Arguments::parse(env::args_os().skip(1))?;
    create_private_directory(&arguments.output)?;
    let authority_root = create_child_directory(&arguments.output, "authority")?;
    let candidate_root = create_child_directory(&arguments.output, "candidate")?;
    let staging_root = create_child_directory(&arguments.output, "staging")?;
    let runtime_root = create_child_directory(&arguments.output, "runtime")?;
    let build_root = shared_build_root()?;

    let mut release_seed = random_seed()?;
    let mut launch_seed = random_seed()?;
    while launch_seed == release_seed {
        launch_seed = random_seed()?;
    }
    let release_key = SigningKey::from_bytes(&release_seed);
    let launch_key = SigningKey::from_bytes(&launch_seed);
    release_seed.fill(0);
    launch_seed.fill(0);

    let authority_bytes = authority_bytes(&release_key, &launch_key)?;
    let authority_path = authority_root.join("qiongli-native-release-authority.json");
    fs::write(&authority_path, &authority_bytes)
        .map_err(|_| "candidate-acceptance-authority-write-failed")?;
    let authority = NativeReleaseAuthority::from_json(&authority_bytes)
        .map_err(|_| "candidate-acceptance-authority-invalid")?;

    let built_product = build_product(&build_root, &authority_path, &arguments.source_commit)?;
    let content =
        qiongli::embedded_content().map_err(|_| "candidate-acceptance-embedded-content-invalid")?;
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), ReleaseChannel::Alpha)
            .map_err(|_| "candidate-acceptance-target-unsupported")?;
    let artifact_id =
        native_artifact_id(&artifact).map_err(|_| "candidate-acceptance-artifact-invalid")?;
    let artifact_target =
        approve_native_artifact_target(staging_root.join(&artifact_id), &artifact)
            .map_err(|_| "candidate-acceptance-artifact-target-invalid")?;
    let product_binary = staging_root.join(format!(
        ".qiongli-acceptance-source{}",
        env::consts::EXE_SUFFIX
    ));
    stage_product_binary(&built_product, &product_binary)?;
    let assembled =
        compose_native_artifact(content.pack(), &artifact, &product_binary, &artifact_target);
    let source_cleanup = fs::remove_file(&product_binary);
    let assembled = assembled.map_err(|error| error.reason_code())?;
    source_cleanup.map_err(|_| "candidate-acceptance-source-cleanup-failed")?;

    let archive_name = native_portable_archive_file_name(&artifact)
        .map_err(|_| "candidate-acceptance-archive-name-invalid")?;
    let archive_path = candidate_root.join(&archive_name);
    let archive_target = approve_native_portable_archive_target(&archive_path, &artifact)
        .map_err(|_| "candidate-acceptance-archive-target-invalid")?;
    let archive =
        compose_native_portable_archive(content.pack(), &artifact_target, &archive_target)
            .map_err(|_| "candidate-acceptance-archive-compose-failed")?;
    let candidate_name = native_release_candidate_file_name(&artifact)
        .map_err(|_| "candidate-acceptance-candidate-name-invalid")?;
    let notes_name = native_release_notes_file_name(&artifact)
        .map_err(|_| "candidate-acceptance-notes-name-invalid")?;
    let (signed_candidate, notes) = sign_candidate_bundle(
        &artifact,
        &archive,
        &assembled.manifest().binary_sha256,
        content.pack().pack_sha256(),
        &arguments.source_commit,
        (&release_key, &launch_key),
        GENERATION,
    )?;
    let notes_size_bytes =
        u64::try_from(notes.len()).map_err(|_| "candidate-acceptance-notes-size-invalid")?;
    let notes_sha256 = sha256_hex(&notes);
    let candidate_path = candidate_root.join(&candidate_name);
    let candidate_bytes = signed_candidate
        .to_canonical_json()
        .map_err(|_| "candidate-acceptance-candidate-serialization-failed")?;
    let candidate_size_bytes = u64::try_from(candidate_bytes.len())
        .map_err(|_| "candidate-acceptance-candidate-size-invalid")?;
    let candidate_sha256 = sha256_hex(&candidate_bytes);
    fs::write(&candidate_path, &candidate_bytes)
        .map_err(|_| "candidate-acceptance-candidate-write-failed")?;
    let notes_path = candidate_root.join(&notes_name);
    fs::write(&notes_path, &notes).map_err(|_| "candidate-acceptance-notes-write-failed")?;
    assert_exact_candidate_files(
        &candidate_root,
        [&archive_name, &candidate_name, &notes_name],
    )?;

    let runtime_target = approve_native_artifact_target(runtime_root.join(&artifact_id), &artifact)
        .map_err(|_| "candidate-acceptance-runtime-target-invalid")?;
    extract_native_portable_archive(content.pack(), &archive_target, &runtime_target)
        .map_err(|_| "candidate-acceptance-runtime-extract-failed")?;
    let runtime_binary = runtime_target.path().join(
        native_artifact_binary_path(&artifact)
            .map_err(|_| "candidate-acceptance-runtime-binary-invalid")?,
    );

    let verified_candidate = signed_candidate
        .verify(
            &authority,
            &qiongli_platform::NativeReleaseCandidateVerificationContext {
                now_unix: crate::now_unix()?,
                expected_source_commit: &arguments.source_commit,
                expected_artifact: &artifact,
                requested_target: ClientActivationTarget::Codex,
            },
            content.pack(),
            &archive_target,
            &notes,
        )
        .map_err(|error| error.reason_code())?;

    let acceptance = run_acceptance(
        &arguments.output,
        &runtime_binary,
        &candidate_path,
        &archive_path,
        &notes_path,
        &arguments.external_clients,
        &verified_candidate,
    )?;
    let activation_journey = if let Some(manifest) = &arguments.predecessor_manifest {
        run_native_activation_journey(
            &arguments.output,
            manifest,
            &build_root,
            &authority_path,
            &arguments.source_commit,
            &authority,
            (&release_key, &launch_key),
            &verified_candidate,
            &candidate_path,
            &archive_path,
            &notes_path,
        )?
    } else {
        json!({"status": "not-run", "reason": "predecessor-manifest-not-provided"})
    };
    drop(release_key);
    drop(launch_key);
    let evidence = json!({
        "schema_version": 1,
        "record_type": "qiongli-native-candidate-acceptance",
        "status": "passed",
        "publication_allowed": false,
        "signing": "ephemeral-test-keys-memory-only",
        "source_commit": arguments.source_commit,
        "artifact": artifact,
        "candidate_files": [archive_name, candidate_name, notes_name],
        "candidate_set": {
            "archive": {
                "file": archive.file_name(),
                "size_bytes": archive.size_bytes(),
                "sha256": archive.archive_sha256()
            },
            "candidate": {
                "file": candidate_name,
                "size_bytes": candidate_size_bytes,
                "sha256": candidate_sha256
            },
            "release_notes": {
                "file": notes_name,
                "size_bytes": notes_size_bytes,
                "sha256": notes_sha256
            }
        },
        "checks": acceptance.checks,
        "native_activation_journey": activation_journey,
        "external_gates": {
            "real_client": acceptance.real_client,
            "displayed_window": {
                "status": "not-run",
                "reason": "interactive-display-not-provided"
            },
            "production_signing": {
                "status": "not-run",
                "reason": "maintainer-signing-boundary"
            }
        }
    });
    let evidence_bytes = serde_json::to_vec_pretty(&evidence)
        .map_err(|_| "candidate-acceptance-evidence-serialization-failed")?;
    fs::write(
        arguments.output.join("acceptance-evidence.json"),
        evidence_bytes,
    )
    .map_err(|_| "candidate-acceptance-evidence-write-failed")?;
    println!("candidate-acceptance-passed");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn sign_candidate_bundle(
    artifact: &ArtifactIdentityV1,
    archive: &qiongli_platform::VerifiedNativePortableArchive,
    binary_sha256: &str,
    pack_sha256: &str,
    source_commit: &str,
    keys: (&SigningKey, &SigningKey),
    generation: u64,
) -> Result<(SignedNativeReleaseCandidateV1, Vec<u8>), &'static str> {
    let (release_key, launch_key) = keys;
    let artifact_id =
        native_artifact_id(artifact).map_err(|_| "candidate-acceptance-artifact-invalid")?;
    let archive_name = native_portable_archive_file_name(artifact)
        .map_err(|_| "candidate-acceptance-archive-invalid")?;
    let candidate_name = native_release_candidate_file_name(artifact)
        .map_err(|_| "candidate-acceptance-candidate-name-invalid")?;
    let notes_name = native_release_notes_file_name(artifact)
        .map_err(|_| "candidate-acceptance-notes-name-invalid")?;
    let notes = render_release_notes(
        artifact,
        &artifact_id,
        &archive_name,
        &candidate_name,
        &notes_name,
    )?;

    let now_unix = now_unix()?;
    let portable_grant = sign_grant(
        LaunchGrantV1 {
            schema_version: 1,
            generation,
            artifact: artifact.clone(),
            binary_sha256: binary_sha256.to_string(),
            resource_pack_sha256: pack_sha256.to_string(),
            allowed_modes: vec![GrantMode::LiteMcp],
            integration_scopes: vec![
                IntegrationScope::CodexLocal,
                IntegrationScope::ClaudeCodeLocal,
            ],
            not_before_unix: now_unix.saturating_sub(60),
            expires_at_unix: now_unix.saturating_add(RELEASE_VALIDITY_SECONDS),
        },
        launch_key,
    )?;
    let envelope = build_native_release_envelope(
        generation,
        archive,
        &portable_grant,
        now_unix.saturating_sub(30),
        now_unix.saturating_add(RELEASE_VALIDITY_SECONDS),
    )
    .map_err(|_| "candidate-acceptance-release-envelope-invalid")?;
    let release_signature = release_key.sign(
        &native_release_envelope_signing_bytes(&envelope)
            .map_err(|_| "candidate-acceptance-release-signing-input-invalid")?,
    );
    let signed_release = SignedNativeReleaseEnvelopeV1 {
        envelope,
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: RELEASE_KEY_ID.to_string(),
            value_hex: encode_hex(&release_signature.to_bytes()),
        },
    };
    let candidate = build_native_release_candidate(
        generation,
        source_commit,
        &signed_release,
        [
            plugin_grant(
                artifact,
                ClientActivationTarget::Codex,
                binary_sha256,
                pack_sha256,
                launch_key,
                now_unix,
                generation,
            )?,
            plugin_grant(
                artifact,
                ClientActivationTarget::ClaudeCode,
                binary_sha256,
                pack_sha256,
                launch_key,
                now_unix,
                generation,
            )?,
        ],
        &notes,
        now_unix,
        now_unix.saturating_add(CANDIDATE_VALIDITY_SECONDS),
    )
    .map_err(|_| "candidate-acceptance-candidate-invalid")?;
    let candidate_signature = release_key.sign(
        &native_release_candidate_signing_bytes(&candidate)
            .map_err(|_| "candidate-acceptance-candidate-signing-input-invalid")?,
    );
    let signed_candidate = SignedNativeReleaseCandidateV1 {
        candidate,
        signature: NativeReleaseSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: RELEASE_KEY_ID.to_string(),
            value_hex: encode_hex(&candidate_signature.to_bytes()),
        },
    };
    Ok((signed_candidate, notes))
}

#[allow(clippy::too_many_arguments)]
fn run_native_activation_journey(
    root: &Path,
    predecessor_manifest: &Path,
    build_root: &Path,
    authority_path: &Path,
    source_commit: &str,
    authority: &NativeReleaseAuthority,
    keys: (&SigningKey, &SigningKey),
    successor: &qiongli_platform::VerifiedNativeReleaseCandidate,
    candidate_path: &Path,
    archive_path: &Path,
    notes_path: &Path,
) -> Result<Value, &'static str> {
    if !cfg!(target_os = "macos") {
        return Err("candidate-activation-journey-target-unsupported");
    }
    let home = create_child_directory(root, "activation-probe-home")?;
    let built = build_product_manifest(
        build_root,
        authority_path,
        source_commit,
        predecessor_manifest,
    )?;
    let version_output = run_product(&built, root, &home, ["--version"])?;
    let version = std::str::from_utf8(&version_output.stdout)
        .map_err(|_| "candidate-predecessor-version-invalid")?
        .trim()
        .strip_prefix("qiongli ")
        .ok_or("candidate-predecessor-version-invalid")?;
    let previous_version =
        semver::Version::parse(version).map_err(|_| "candidate-predecessor-version-invalid")?;
    if previous_version
        >= semver::Version::parse(&successor.candidate().artifact.version)
            .map_err(|_| "candidate-predecessor-version-invalid")?
    {
        return Err("candidate-predecessor-version-invalid");
    }
    let content =
        qiongli::embedded_content().map_err(|_| "candidate-acceptance-embedded-content-invalid")?;
    let artifact = current_target_native_artifact_identity(version, ReleaseChannel::Alpha)
        .map_err(|_| "candidate-predecessor-version-invalid")?;
    let previous_root = create_child_directory(root, "predecessor")?;
    let staged_binary = previous_root.join("source-cli");
    stage_product_binary(&built, &staged_binary)?;
    let artifact_target = approve_native_artifact_target(
        previous_root.join(native_artifact_id(&artifact).map_err(|error| error.reason_code())?),
        &artifact,
    )
    .map_err(|error| error.reason_code())?;
    let assembled =
        compose_native_artifact(content.pack(), &artifact, &staged_binary, &artifact_target)
            .map_err(|error| error.reason_code())?;
    let previous_archive = approve_native_portable_archive_target(
        previous_root.join(
            native_portable_archive_file_name(&artifact).map_err(|error| error.reason_code())?,
        ),
        &artifact,
    )
    .map_err(|error| error.reason_code())?;
    let archive =
        compose_native_portable_archive(content.pack(), &artifact_target, &previous_archive)
            .map_err(|error| error.reason_code())?;
    let (signed, notes) = sign_candidate_bundle(
        &artifact,
        &archive,
        &assembled.manifest().binary_sha256,
        content.pack().pack_sha256(),
        source_commit,
        keys,
        1,
    )?;
    let previous_candidate = signed
        .verify(
            authority,
            &qiongli_platform::NativeReleaseCandidateVerificationContext {
                now_unix: now_unix()?,
                expected_source_commit: source_commit,
                expected_artifact: &artifact,
                requested_target: ClientActivationTarget::Codex,
            },
            content.pack(),
            &previous_archive,
            &notes,
        )
        .map_err(|error| error.reason_code())?;
    let mut success = run_native_activation_case(
        root,
        &previous_candidate,
        successor,
        candidate_path,
        archive_path,
        notes_path,
        false,
    )?;
    success["interrupted_recovery"] = run_native_activation_case(
        root,
        &previous_candidate,
        successor,
        candidate_path,
        archive_path,
        notes_path,
        true,
    )?;
    Ok(success)
}

fn run_native_activation_case(
    root: &Path,
    previous_candidate: &qiongli_platform::VerifiedNativeReleaseCandidate,
    successor: &qiongli_platform::VerifiedNativeReleaseCandidate,
    candidate_path: &Path,
    archive_path: &Path,
    notes_path: &Path,
    interrupt: bool,
) -> Result<Value, &'static str> {
    let home = create_child_directory(
        root,
        if interrupt {
            "activation-interrupted-home"
        } else {
            "activation-home"
        },
    )?;
    let content =
        qiongli::embedded_content().map_err(|_| "candidate-acceptance-embedded-content-invalid")?;
    let artifact = &previous_candidate.candidate().artifact;
    let version = artifact.version.as_str();
    let installed = qiongli_platform::apply_native_release_candidate_local(
        content.pack(),
        previous_candidate,
        &home,
        now_unix()?,
    )
    .map_err(|error| error.reason_code())?;
    let previous_id = &installed.payload.receipt.install_id;
    let previous_binary = installed_candidate_binary(&home, artifact)?;
    run_managed_fixture_operation(&previous_binary, root, &home, "cli-install", None)?;
    let command = home.join(".local/bin/qiongli");
    if run_product(&command, root, &home, ["--version"])?.stdout
        != format!("qiongli {version}\n").as_bytes()
    {
        return Err("candidate-predecessor-installed-version-invalid");
    }
    let previous_hash =
        sha256_hex(&fs::read(&command).map_err(|_| "candidate-predecessor-read-failed")?);
    let config =
        qiongli_config::resolve_config_root(Some(home.join(".qiongli/config").as_os_str()), &home)
            .map_err(|error| error.reason_code())?;
    let store =
        qiongli_config::UpdateStateStore::new(config, qiongli_config::UpdateStreamPreference::Beta);
    let initial = store.load().map_err(|error| error.reason_code())?;
    let mut state = initial.state;
    state.last_accepted_generation = 1;
    state.last_known_good = Some(qiongli_config::UpdateLastKnownGood {
        version: version.to_string(),
        channel: qiongli_config::UpdateReleaseChannel::Alpha,
        generation: 1,
        archive_sha256: previous_candidate
            .candidate()
            .signed_portable_release
            .envelope
            .archive_sha256
            .clone(),
        resource_pack_sha256: content.pack().pack_sha256().to_string(),
    });
    store
        .replace(initial.revision, state)
        .map_err(|error| error.reason_code())?;
    qiongli_platform::stage_native_release_candidate_local(
        content.pack(),
        successor,
        &home,
        now_unix()?,
    )
    .map_err(|error| error.reason_code())?;
    let next_binary = installed_candidate_binary(&home, &successor.candidate().artifact)?;
    let common: Vec<OsString> = vec![
        "--candidate".into(),
        candidate_path.into(),
        "--archive".into(),
        archive_path.into(),
        "--release-notes".into(),
        notes_path.into(),
        "--target".into(),
        "codex".into(),
        "--previous-install-id".into(),
        previous_id.into(),
    ];
    let invoke = |subcommand: &str, extra: &[OsString]| -> Result<Value, &'static str> {
        let mut args: Vec<OsString> = vec!["install".into(), "candidate".into(), subcommand.into()];
        args.extend(common.clone());
        args.extend_from_slice(extra);
        parse_output_json(&run_product(&next_binary, root, &home, args)?)
    };
    let preview = invoke("activate-preview", &[])?;
    let preflight = preview["preflight_digest_sha256"]
        .as_str()
        .ok_or("candidate-activation-preview-invalid")?;
    let prepared = invoke(
        "activate-prepare",
        &[
            "--expected-preflight-digest".into(),
            preflight.into(),
            "--approve-filesystem-write".into(),
        ],
    )?;
    let transaction = prepared["transaction_id"]
        .as_str()
        .ok_or("candidate-activation-preparation-invalid")?;
    let journal = prepared["journal_sha256"]
        .as_str()
        .ok_or("candidate-activation-preparation-invalid")?;
    let approval = prepared["approval_digest_sha256"]
        .as_str()
        .ok_or("candidate-activation-preparation-invalid")?;
    let activation_args = [
        "--transaction-id".into(),
        transaction.into(),
        "--expected-journal-digest".into(),
        journal.into(),
        "--expected-approval-digest".into(),
        approval.into(),
        "--approve-filesystem-write".into(),
        "--approve-client-config-change".into(),
        "--approve-host-trust".into(),
    ];
    if interrupt {
        let mut args: Vec<OsString> = vec!["install".into(), "candidate".into(), "activate".into()];
        args.extend(common);
        args.extend_from_slice(&activation_args);
        kill_native_activation_after_cli_switch(
            &next_binary,
            root,
            &home,
            &command,
            &store,
            transaction,
            &args,
        )?;
        let pending = store.load().map_err(|error| error.reason_code())?;
        if pending.state.last_accepted_generation != 1
            || pending
                .state
                .active_transaction
                .as_ref()
                .is_none_or(|active| active.transaction_id != transaction)
            || sha256_hex(
                &fs::read(&command).map_err(|_| "candidate-interrupted-command-unavailable")?,
            ) != successor
                .candidate()
                .signed_portable_release
                .envelope
                .binary_sha256
        {
            return Err("candidate-interrupted-state-invalid");
        }
        let recovered = parse_output_json(&run_product(
            &next_binary,
            root,
            &home,
            [
                "install",
                "candidate",
                "activate-recover",
                "--transaction-id",
                transaction,
                "--expected-journal-digest",
                journal,
                "--approve-filesystem-write",
            ],
        )?)?;
        let restored = store.load().map_err(|error| error.reason_code())?;
        if recovered["outcome"] != "rolled-back"
            || restored.state.active_transaction.is_some()
            || restored.state.last_accepted_generation != 1
            || restored.state.last_known_good != pending.state.last_known_good
            || sha256_hex(
                &fs::read(&command).map_err(|_| "candidate-restored-command-unavailable")?,
            ) != previous_hash
            || run_product(&command, root, &home, ["--version"])?.stdout
                != format!("qiongli {version}\n").as_bytes()
            || home
                .join(".qiongli/native/active-installation.json")
                .exists()
        {
            return Err("candidate-interrupted-recovery-invalid");
        }
        qiongli::check_native_cli_health(&home, &home.join(".qiongli/config"), previous_candidate)?;
        run_mcp(&command, root, &home)?;
        return Ok(
            json!({"status": "passed", "signal": "SIGKILL", "observed_boundary": "new-cli-inode-before-durable-outcome",
            "outcome": "rolled-back", "prior_release_preserved": true, "old_binary_and_version_restored": true,
            "restored_health_and_mcp": "passed", "process_scope": "test-owned-process-group"}),
        );
    }
    let result = invoke("activate", &activation_args)?;
    if result["outcome"] != "committed"
        || result["transaction_id"] != transaction
        || result["journal_sha256"] != journal
    {
        return Err("candidate-activation-outcome-invalid");
    }
    let next_version = run_product(&command, root, &home, ["--version"])?;
    if next_version.stdout
        != format!("qiongli {}\n", successor.candidate().artifact.version).as_bytes()
        || sha256_hex(&fs::read(&command).map_err(|_| "candidate-activation-command-read-failed")?)
            == previous_hash
    {
        return Err("candidate-activation-version-switch-invalid");
    }
    qiongli::check_native_cli_health(&home, &home.join(".qiongli/config"), successor)?;
    run_mcp(&command, root, &home)?;
    let committed = store.load().map_err(|error| error.reason_code())?;
    if committed.state.last_accepted_generation != successor.candidate().generation
        || committed.state.active_transaction.is_some()
        || committed
            .state
            .last_known_good
            .as_ref()
            .is_none_or(|release| {
                release.version != successor.candidate().artifact.version
                    || release.archive_sha256
                        != successor
                            .candidate()
                            .signed_portable_release
                            .envelope
                            .archive_sha256
            })
    {
        return Err("candidate-activation-release-state-invalid");
    }
    let recovered = parse_output_json(&run_product(
        &next_binary,
        root,
        &home,
        [
            "install",
            "candidate",
            "activate-recover",
            "--transaction-id",
            transaction,
            "--expected-journal-digest",
            journal,
            "--approve-filesystem-write",
        ],
    )?)?;
    if recovered["outcome"] != "committed"
        || store.load().map_err(|error| error.reason_code())? != committed
    {
        return Err("candidate-activation-replay-invalid");
    }
    Ok(
        json!({"status": "passed", "predecessor_version": version, "successor_version": successor.candidate().artifact.version,
        "predecessor_binary_sha256": previous_hash, "successor_binary_sha256": successor.candidate().signed_portable_release.envelope.binary_sha256,
        "target": "codex", "public_activation": "committed", "installed_health_and_mcp": "passed", "recovery_replay": "unchanged",
        "predecessor_source": "caller-provided-manifest-built-with-ephemeral-authority", "publication_allowed": false}),
    )
}

#[cfg(target_os = "macos")]
#[allow(clippy::too_many_arguments)]
fn kill_native_activation_after_cli_switch(
    binary: &Path,
    root: &Path,
    home: &Path,
    installed: &Path,
    store: &qiongli_config::UpdateStateStore,
    transaction: &str,
    args: &[OsString],
) -> Result<(), &'static str> {
    use std::os::unix::{
        fs::MetadataExt,
        process::{CommandExt, ExitStatusExt},
    };
    use std::time::{Duration, Instant};
    let old_inode = fs::metadata(installed)
        .map_err(|_| "candidate-interruption-source-unavailable")?
        .ino();
    let outcome = store
        .staging_root()
        .join(transaction)
        .join("native-activation-outcome.json");
    let marker = home.join(".qiongli/native/active-installation.json");
    let mut command = product_command(binary, root, home);
    command
        .args(args)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|_| "candidate-interruption-start-failed")?;
    let group = rustix::process::Pid::from_raw(
        i32::try_from(child.id()).map_err(|_| "candidate-interruption-pid-invalid")?,
    )
    .ok_or("candidate-interruption-pid-invalid")?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let observed = loop {
        match child.try_wait() {
            // A reaped child no longer reserves its PID: never signal that group afterward.
            Ok(Some(_)) => return Err("candidate-interruption-boundary-missed"),
            Ok(None) => {}
            Err(_) => break false,
        }
        if marker.is_file()
            && !outcome.exists()
            && fs::metadata(installed).is_ok_and(|metadata| metadata.ino() != old_inode)
        {
            break true;
        }
        if Instant::now() >= deadline {
            break false;
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    // This group was created above solely for the fixture; includes its health child.
    let killed = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
    let status = child
        .wait()
        .map_err(|_| "candidate-interruption-wait-failed")?;
    if !observed
        || killed.is_err()
        || status.signal() != Some(9)
        || outcome.exists()
        || !marker.is_file()
    {
        return Err("candidate-interruption-boundary-missed");
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn kill_native_activation_after_cli_switch(
    _binary: &Path,
    _root: &Path,
    _home: &Path,
    _installed: &Path,
    _store: &qiongli_config::UpdateStateStore,
    _transaction: &str,
    _args: &[OsString],
) -> Result<(), &'static str> {
    Err("candidate-activation-journey-target-unsupported")
}

struct Arguments {
    output: PathBuf,
    source_commit: String,
    external_clients: ExternalClients,
    predecessor_manifest: Option<PathBuf>,
}

struct ExternalClients {
    codex: Option<CodexClient>,
    claude: Option<PathBuf>,
}

struct CodexClient {
    binary: PathBuf,
    validator: PathBuf,
    validator_python: PathBuf,
}

impl Arguments {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let values = values.into_iter().collect::<Vec<_>>();
        let mut output = None;
        let mut source_commit = None;
        let mut codex_binary = None;
        let mut plugin_validator = None;
        let mut plugin_validator_python = None;
        let mut claude_binary = None;
        let mut predecessor_manifest = None;
        let mut index = 0;
        while index < values.len() {
            let option = values[index]
                .to_str()
                .ok_or("candidate-acceptance-usage-invalid")?;
            let value = values
                .get(index + 1)
                .ok_or("candidate-acceptance-usage-invalid")?;
            match option {
                "--output" if output.is_none() => output = Some(PathBuf::from(value)),
                "--source-commit" if source_commit.is_none() => {
                    source_commit = value.to_str().map(ToOwned::to_owned)
                }
                "--codex-bin" if codex_binary.is_none() => {
                    codex_binary = Some(PathBuf::from(value))
                }
                "--plugin-validator" if plugin_validator.is_none() => {
                    plugin_validator = Some(PathBuf::from(value))
                }
                "--plugin-validator-python" if plugin_validator_python.is_none() => {
                    plugin_validator_python = Some(PathBuf::from(value))
                }
                "--predecessor-manifest" if predecessor_manifest.is_none() => {
                    predecessor_manifest = Some(valid_external_file(PathBuf::from(value))?);
                }
                "--claude-bin" if claude_binary.is_none() => {
                    claude_binary = Some(PathBuf::from(value))
                }
                _ => return Err("candidate-acceptance-usage-invalid"),
            }
            index += 2;
        }
        let output = output.ok_or("candidate-acceptance-usage-invalid")?;
        let source_commit = source_commit.ok_or("candidate-acceptance-usage-invalid")?;
        let codex = match (codex_binary, plugin_validator, plugin_validator_python) {
            (None, None, None) => None,
            (Some(binary), Some(validator), Some(validator_python)) => Some(CodexClient {
                binary: valid_external_command(binary)?,
                validator: valid_external_file(validator)?,
                validator_python: valid_external_command(validator_python)?,
            }),
            _ => return Err("candidate-acceptance-usage-invalid"),
        };
        let claude = claude_binary.map(valid_external_command).transpose()?;
        if !output.is_absolute()
            || output.exists()
            || !valid_source_commit(&source_commit)
            || output.parent().is_none()
            || !outside_checkout(&output)
        {
            return Err("candidate-acceptance-usage-invalid");
        }
        Ok(Self {
            output,
            source_commit,
            external_clients: ExternalClients { codex, claude },
            predecessor_manifest,
        })
    }
}

fn valid_external_command(path: PathBuf) -> Result<PathBuf, &'static str> {
    valid_external_file(path.clone())?;
    // Preserve virtual-environment and argv[0] behavior of the requested command.
    Ok(path)
}

fn valid_external_file(path: PathBuf) -> Result<PathBuf, &'static str> {
    if !path.is_absolute() {
        return Err("candidate-acceptance-usage-invalid");
    }
    let canonical = fs::canonicalize(path).map_err(|_| "candidate-acceptance-usage-invalid")?;
    let metadata =
        fs::symlink_metadata(&canonical).map_err(|_| "candidate-acceptance-usage-invalid")?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err("candidate-acceptance-usage-invalid");
    }
    Ok(canonical)
}

fn authority_bytes(
    release_key: &SigningKey,
    launch_key: &SigningKey,
) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(&json!({
        "schema_version": 1,
        "channel": "alpha",
        "minimum_release_generation": 1,
        "minimum_launch_grant_generation": 1,
        "release_keys": [{
            "key_id": RELEASE_KEY_ID,
            "public_key_hex": encode_hex(&release_key.verifying_key().to_bytes()),
            "minimum_generation": 1,
            "maximum_generation_exclusive": GENERATION + 1
        }],
        "launch_grant_keys": [{
            "key_id": LAUNCH_KEY_ID,
            "public_key_hex": encode_hex(&launch_key.verifying_key().to_bytes())
        }]
    }))
    .map_err(|_| "candidate-acceptance-authority-serialization-failed")
}

fn build_product(
    build_root: &Path,
    authority_path: &Path,
    source_commit: &str,
) -> Result<PathBuf, &'static str> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(|root| root.join("Cargo.toml"))
        .ok_or("candidate-acceptance-manifest-unavailable")?;
    build_product_manifest(build_root, authority_path, source_commit, &manifest)
}

fn build_product_manifest(
    build_root: &Path,
    authority_path: &Path,
    source_commit: &str,
    manifest: &Path,
) -> Result<PathBuf, &'static str> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let status = Command::new(cargo)
        .args([
            OsStr::new("build"),
            OsStr::new("--manifest-path"),
            manifest.as_os_str(),
            OsStr::new("--package"),
            OsStr::new("qiongli"),
            OsStr::new("--bin"),
            OsStr::new("qiongli"),
            OsStr::new("--no-default-features"),
            OsStr::new("--release"),
            OsStr::new("--locked"),
            OsStr::new("--target-dir"),
            build_root.as_os_str(),
        ])
        .env("QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE", authority_path)
        .env("QIONGLI_NATIVE_SOURCE_COMMIT", source_commit)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .status()
        .map_err(|_| "candidate-acceptance-product-build-failed")?;
    if !status.success() {
        return Err("candidate-acceptance-product-build-failed");
    }
    let binary = build_root
        .join("release")
        .join(format!("qiongli{}", env::consts::EXE_SUFFIX));
    if !binary.is_file() {
        return Err("candidate-acceptance-product-binary-missing");
    }
    Ok(binary)
}

fn shared_build_root() -> Result<PathBuf, &'static str> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(|root| root.join("target/qiongli-native-candidate-acceptance-build"))
        .ok_or("candidate-acceptance-build-root-unavailable")
}

fn render_release_notes(
    artifact: &ArtifactIdentityV1,
    artifact_id: &str,
    archive_name: &str,
    candidate_name: &str,
    notes_name: &str,
) -> Result<Vec<u8>, &'static str> {
    let replacements = [
        ("{{version}}", artifact.version.as_str()),
        ("{{os}}", operating_system(artifact.os)),
        ("{{arch}}", architecture(artifact.arch)),
        ("{{artifact_id}}", artifact_id),
        ("{{archive_name}}", archive_name),
        ("{{candidate_name}}", candidate_name),
        ("{{notes_name}}", notes_name),
    ];
    let mut rendered = RELEASE_NOTES_TEMPLATE.to_string();
    for (token, value) in replacements {
        if !rendered.contains(token) {
            return Err("candidate-acceptance-notes-template-token-missing");
        }
        rendered = rendered.replace(token, value);
    }
    let required_claims = [
        artifact_id,
        archive_name,
        candidate_name,
        notes_name,
        "empty runtime PATH",
        "Codex local",
        "Claude Code local",
        "Full MCP",
        "Alpha.2",
        "recovery-required",
        "community-alpha — not platform-trusted",
        "per-app Open Anyway flow",
        "Smart App Control or enterprise policy",
        "AppImage facilities",
        "Raw CI artifacts must not be uploaded directly",
        "exact-set maintainer authorization",
        "disable global security controls",
        "self-signed Windows root certificate",
        "does not authorize publication",
    ];
    if rendered.contains("{{") {
        return Err("candidate-acceptance-notes-template-token-unresolved");
    }
    if required_claims
        .into_iter()
        .any(|claim| !rendered.contains(claim))
    {
        return Err("candidate-acceptance-notes-required-claim-missing");
    }
    if rendered.is_empty() || rendered.len() > MAX_NATIVE_RELEASE_NOTES_BYTES {
        return Err("candidate-acceptance-notes-size-invalid");
    }
    Ok(rendered.into_bytes())
}

const fn operating_system(value: OperatingSystem) -> &'static str {
    match value {
        OperatingSystem::Macos => "macos",
        OperatingSystem::Windows => "windows",
        OperatingSystem::Linux => "linux",
    }
}

const fn architecture(value: Architecture) -> &'static str {
    match value {
        Architecture::Aarch64 => "aarch64",
        Architecture::X86_64 => "x86-64",
    }
}

fn stage_product_binary(source: &Path, destination: &Path) -> Result<(), &'static str> {
    let metadata =
        fs::symlink_metadata(source).map_err(|_| "candidate-acceptance-product-source-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_STAGED_PRODUCT_BYTES
    {
        return Err("candidate-acceptance-product-source-invalid");
    }
    let result = (|| {
        let mut source = File::open(source)
            .map_err(|_| "candidate-acceptance-product-source-invalid")?
            .take(MAX_STAGED_PRODUCT_BYTES.saturating_add(1));
        let mut destination_file = create_private_product_file(destination)?;
        let copied = io::copy(&mut source, &mut destination_file)
            .map_err(|_| "candidate-acceptance-product-stage-failed")?;
        destination_file
            .sync_all()
            .map_err(|_| "candidate-acceptance-product-stage-failed")?;
        drop(destination_file);
        if copied != metadata.len() || copied > MAX_STAGED_PRODUCT_BYTES {
            return Err("candidate-acceptance-product-stage-failed");
        }
        set_product_executable(destination)?;
        verify_staged_product(destination)
    })();
    if result.is_err() {
        let _ = fs::remove_file(destination);
    }
    result
}

#[cfg(unix)]
fn create_private_product_file(path: &Path) -> Result<File, &'static str> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o700)
        .open(path)
        .map_err(|_| "candidate-acceptance-product-stage-failed")
}

#[cfg(windows)]
fn create_private_product_file(path: &Path) -> Result<File, &'static str> {
    qiongli_windows_security::create_owner_only_new_file(path)
        .map_err(|_| "candidate-acceptance-product-stage-failed")
}

#[cfg(not(any(unix, windows)))]
fn create_private_product_file(_path: &Path) -> Result<File, &'static str> {
    Err("candidate-acceptance-platform-unsupported")
}

#[cfg(unix)]
fn set_product_executable(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| "candidate-acceptance-product-stage-failed")
}

#[cfg(not(unix))]
fn set_product_executable(_path: &Path) -> Result<(), &'static str> {
    Ok(())
}

fn verify_staged_product(path: &Path) -> Result<(), &'static str> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "candidate-acceptance-staged-product-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_STAGED_PRODUCT_BYTES
    {
        return Err("candidate-acceptance-staged-product-invalid");
    }
    verify_staged_product_security(path, &metadata)
}

#[cfg(unix)]
fn verify_staged_product_security(path: &Path, metadata: &Metadata) -> Result<(), &'static str> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let parent = path
        .parent()
        .ok_or("candidate-acceptance-staged-product-invalid")?;
    let parent_metadata =
        fs::symlink_metadata(parent).map_err(|_| "candidate-acceptance-staged-product-invalid")?;
    if !parent_metadata.is_dir()
        || parent_metadata.uid() != metadata.uid()
        || parent_metadata.permissions().mode() & 0o077 != 0
        || metadata.nlink() != 1
        || metadata.permissions().mode() & 0o111 == 0
    {
        return Err("candidate-acceptance-staged-product-invalid");
    }
    Ok(())
}

#[cfg(windows)]
fn verify_staged_product_security(path: &Path, _metadata: &Metadata) -> Result<(), &'static str> {
    let file = qiongli_windows_security::open_owner_only_file(path)
        .map_err(|_| "candidate-acceptance-staged-product-invalid")?;
    let facts = qiongli_windows_security::handle_facts(&file)
        .map_err(|_| "candidate-acceptance-staged-product-invalid")?;
    if facts.number_of_links != 1 {
        return Err("candidate-acceptance-staged-product-invalid");
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn verify_staged_product_security(_path: &Path, _metadata: &Metadata) -> Result<(), &'static str> {
    Err("candidate-acceptance-platform-unsupported")
}

fn outside_checkout(output: &Path) -> bool {
    let Some(output_parent) = output.parent() else {
        return false;
    };
    let Some(checkout_root) = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4) else {
        return false;
    };
    let Ok(output_parent) = fs::canonicalize(output_parent) else {
        return false;
    };
    let Ok(checkout_root) = fs::canonicalize(checkout_root) else {
        return false;
    };
    !output_parent.starts_with(checkout_root)
}

fn assert_exact_candidate_files(root: &Path, expected: [&str; 3]) -> Result<(), &'static str> {
    let mut actual = fs::read_dir(root)
        .map_err(|_| "candidate-acceptance-candidate-files-invalid")?
        .map(|entry| {
            let entry = entry.map_err(|_| "candidate-acceptance-candidate-files-invalid")?;
            if !entry
                .file_type()
                .map_err(|_| "candidate-acceptance-candidate-files-invalid")?
                .is_file()
            {
                return Err("candidate-acceptance-candidate-files-invalid");
            }
            entry
                .file_name()
                .into_string()
                .map_err(|_| "candidate-acceptance-candidate-files-invalid")
        })
        .collect::<Result<Vec<_>, _>>()?;
    actual.sort_unstable();
    let mut expected = expected.map(ToOwned::to_owned);
    expected.sort_unstable();
    if actual != expected {
        return Err("candidate-acceptance-candidate-files-invalid");
    }
    Ok(())
}

fn sign_grant(grant: LaunchGrantV1, key: &SigningKey) -> Result<SignedLaunchGrantV1, &'static str> {
    let signature = key.sign(
        &launch_grant_signing_bytes(&grant)
            .map_err(|_| "candidate-acceptance-grant-signing-input-invalid")?,
    );
    Ok(SignedLaunchGrantV1 {
        grant,
        signature: GrantSignatureV1 {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: LAUNCH_KEY_ID.to_string(),
            value_hex: encode_hex(&signature.to_bytes()),
        },
    })
}

fn plugin_grant(
    portable_artifact: &qiongli_platform::ArtifactIdentityV1,
    target: ClientActivationTarget,
    binary_sha256: &str,
    pack_sha256: &str,
    key: &SigningKey,
    now_unix: u64,
    generation: u64,
) -> Result<NativeClientPluginGrantV1, &'static str> {
    let mut artifact = portable_artifact.clone();
    artifact.installer_kind = InstallerKind::PluginBundle;
    Ok(NativeClientPluginGrantV1 {
        target,
        signed_launch_grant: sign_grant(
            LaunchGrantV1 {
                schema_version: 1,
                generation,
                artifact,
                binary_sha256: binary_sha256.to_string(),
                resource_pack_sha256: pack_sha256.to_string(),
                allowed_modes: target.allowed_grant_modes().to_vec(),
                integration_scopes: vec![target.integration_scope()],
                not_before_unix: now_unix.saturating_sub(60),
                expires_at_unix: now_unix.saturating_add(RELEASE_VALIDITY_SECONDS),
            },
            key,
        )?,
    })
}

struct AcceptanceOutcome {
    checks: Value,
    real_client: Value,
}

fn run_acceptance(
    root: &Path,
    binary: &Path,
    candidate: &Path,
    archive: &Path,
    notes: &Path,
    external_clients: &ExternalClients,
    verified_candidate: &qiongli_platform::VerifiedNativeReleaseCandidate,
) -> Result<AcceptanceOutcome, &'static str> {
    let artifact = &verified_candidate.candidate().artifact;
    let product_home = create_child_directory(root, "product-home")?;
    check_cli_entry(binary, root, &product_home)?;
    let version = run_product(binary, root, &product_home, [OsStr::new("--version")])?;
    let version_text = String::from_utf8(version.stdout)
        .map_err(|_| "candidate-acceptance-version-output-invalid")?;
    if version_text.trim() != format!("qiongli {}", env!("CARGO_PKG_VERSION")) {
        return Err("candidate-acceptance-version-mismatch");
    }

    let content_list = run_product(
        binary,
        root,
        &product_home,
        [OsStr::new("content"), OsStr::new("list")],
    )?;
    let content_json = parse_output_json(&content_list)?;
    if content_json["command"] != "content-list" {
        return Err("candidate-acceptance-content-list-invalid");
    }
    let retired_materialized = root.join("materialized-lite");
    run_product_expected_failure(
        binary,
        root,
        &product_home,
        [
            OsStr::new("content"),
            OsStr::new("materialize"),
            OsStr::new("--profile"),
            OsStr::new("lite"),
            OsStr::new("--target"),
            retired_materialized.as_os_str(),
        ],
        "managed-skills-plan-required",
    )?;
    if retired_materialized.exists() {
        return Err("candidate-acceptance-retired-content-writer-mutated");
    }

    let skills_plan = run_product(
        binary,
        root,
        &product_home,
        [
            OsStr::new("app"),
            OsStr::new("plan"),
            OsStr::new("skills-reconcile"),
            OsStr::new("--preset"),
            OsStr::new("qiongli-managed"),
            OsStr::new("--profile"),
            OsStr::new("skill-only"),
        ],
    )?;
    let skills_plan_json = parse_output_json(&skills_plan)?;
    let skills_plan_digest = skills_plan_json["plan_digest_sha256"]
        .as_str()
        .filter(|value| valid_sha256(value))
        .ok_or("candidate-acceptance-managed-skills-plan-invalid")?;
    if skills_plan_json["document_kind"] != "qiongli-managed-operation-plan"
        || skills_plan_json["schema_version"] != 1
        || skills_plan_json["approvals_required"] != json!(["filesystem-write"])
    {
        return Err("candidate-acceptance-managed-skills-plan-invalid");
    }
    let skills_plan_path = product_home.join("skills-install.plan.json");
    write_private_new(&skills_plan_path, &skills_plan.stdout)?;
    let skills_apply = run_product(
        binary,
        root,
        &product_home,
        [
            OsStr::new("app"),
            OsStr::new("apply"),
            OsStr::new("--plan"),
            skills_plan_path.as_os_str(),
            OsStr::new("--expected-plan-digest"),
            OsStr::new(skills_plan_digest),
            OsStr::new("--approve-filesystem-write"),
        ],
    );
    let _ = fs::remove_file(&skills_plan_path);
    let skills_apply = skills_apply?;
    let skills_apply_json = parse_output_json(&skills_apply)?;
    if skills_apply_json["operation"] != "skills-reconcile-preset"
        || skills_apply_json["result"] != "installed"
        || !product_home.join(".qiongli-skills").is_dir()
    {
        return Err("candidate-acceptance-managed-skills-install-invalid");
    }
    let skills_verify = run_product(
        binary,
        root,
        &product_home,
        [
            OsStr::new("app"),
            OsStr::new("verify-skills"),
            OsStr::new("--preset"),
            OsStr::new("qiongli-managed"),
        ],
    )?;
    let skills_verify_json = parse_output_json(&skills_verify)?;
    if skills_verify_json["type"] != "completed"
        || skills_verify_json["code"] != "skills-materialization-verified"
    {
        return Err("candidate-acceptance-managed-skills-verify-invalid");
    }

    let ui = run_product(
        binary,
        root,
        &product_home,
        [OsStr::new("ui"), OsStr::new("--startup-check")],
    )?;
    let ui_json = parse_output_json(&ui)?;
    if ui_json["command"] != "ui-startup-check"
        || ui_json["service"] != "ready"
        || ui_json["update_surface"] != "ready"
    {
        return Err("candidate-acceptance-ui-preflight-invalid");
    }
    run_mcp(binary, root, &product_home)?;

    check_candidate_staging(binary, root, candidate, archive, notes)?;

    let mut codex_client_evidence = json!({
        "status": "not-run",
        "reason": "external-client-not-provided"
    });
    let mut claude_client_evidence = json!({
        "status": "not-run",
        "reason": "external-client-not-provided"
    });
    for (target, directory) in [("codex", "codex-home"), ("claude", "claude-home")] {
        let home = create_child_directory(root, directory)?;
        let user_project = create_child_directory(&home, "user-project")?;
        let qiongli_home = create_child_directory(&home, ".qiongli")?;
        let config_home = create_child_directory(&qiongli_home, "config")?;
        let global_v2 = create_child_directory(&config_home, "v2")?;
        let host_home = create_child_directory(
            &home,
            if target == "codex" {
                ".agents"
            } else {
                ".claude"
            },
        )?;
        let canaries = [
            (
                user_project.join("research.md"),
                b"user-project-byte-canary".as_slice(),
            ),
            (
                global_v2.join("user-state.canary"),
                b"global-v2-state-byte-canary".as_slice(),
            ),
            (
                host_home.join("unmanaged-state.canary"),
                b"unmanaged-host-byte-canary".as_slice(),
            ),
        ];
        for (path, bytes) in &canaries {
            write_private_new(path, bytes)?;
        }
        let common = [
            OsStr::new("--candidate"),
            candidate.as_os_str(),
            OsStr::new("--archive"),
            archive.as_os_str(),
            OsStr::new("--release-notes"),
            notes.as_os_str(),
            OsStr::new("--target"),
            OsStr::new(target),
        ];
        let mut preview_args = vec![OsString::from("install"), OsString::from("candidate")];
        preview_args.push(OsString::from("preview"));
        preview_args.extend(common.iter().map(|value| (*value).to_os_string()));
        let preview = run_product(binary, root, &home, preview_args)?;
        let preview_json = parse_output_json(&preview)?;
        if preview_json["mutation"] != "none" || candidate_owned_state_present(&home, target) {
            return Err("candidate-acceptance-preview-mutated-state");
        }
        assert_canaries_unchanged(&canaries)?;
        let approval_digest = preview_json["approval_digest_sha256"]
            .as_str()
            .filter(|value| valid_sha256(value))
            .ok_or("candidate-acceptance-preview-digest-invalid")?;
        let install_id = preview_json["install_id"]
            .as_str()
            .ok_or("candidate-acceptance-preview-install-id-invalid")?
            .to_string();

        let mut apply_args = vec![
            OsString::from("install"),
            OsString::from("candidate"),
            OsString::from("apply"),
        ];
        apply_args.extend(common.iter().map(|value| (*value).to_os_string()));
        apply_args.extend([
            OsString::from("--expected-approval-digest"),
            OsString::from(approval_digest),
            OsString::from("--approve-filesystem-write"),
            OsString::from("--approve-client-config-change"),
            OsString::from("--approve-host-trust"),
        ]);
        if target == "codex" {
            let mut wrong_digest_args = apply_args.clone();
            let digest_index = wrong_digest_args
                .iter()
                .position(|value| value == "--expected-approval-digest")
                .and_then(|index| index.checked_add(1))
                .ok_or("candidate-acceptance-apply-arguments-invalid")?;
            wrong_digest_args[digest_index] = OsString::from(
                if approval_digest
                    == "0000000000000000000000000000000000000000000000000000000000000000"
                {
                    "1111111111111111111111111111111111111111111111111111111111111111"
                } else {
                    "0000000000000000000000000000000000000000000000000000000000000000"
                },
            );
            run_product_failure(binary, root, &home, wrong_digest_args, 1)?;
            let mut partial_approval_args = apply_args.clone();
            partial_approval_args.retain(|value| value != "--approve-host-trust");
            run_product_failure(binary, root, &home, partial_approval_args, 2)?;
            if candidate_owned_state_present(&home, target) {
                return Err("candidate-acceptance-failed-approval-mutated-state");
            }
            assert_canaries_unchanged(&canaries)?;

            let plugins = create_child_directory(&host_home, "plugins")?;
            let marketplace = plugins.join("marketplace.json");
            let conflict = serde_json::to_vec(&json!({
                "plugins": [{
                    "name": "qiongli-next",
                    "source": {"source": "local", "path": "./foreign-source"},
                    "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
                    "category": "Education"
                }]
            }))
            .map_err(|_| "candidate-acceptance-conflict-serialization-failed")?;
            fs::write(&marketplace, &conflict)
                .map_err(|_| "candidate-acceptance-conflict-write-failed")?;
            run_product_failure(binary, root, &home, apply_args.clone(), 1)?;
            if home.join(".qiongli/plugins/codex/qiongli-next").exists()
                || payload_directory_present(&home.join(".qiongli/native/payloads"))?
                || fs::read(&marketplace)
                    .map_err(|_| "candidate-acceptance-conflict-read-failed")?
                    != conflict
            {
                return Err("candidate-acceptance-compensation-invalid");
            }
            assert_canaries_unchanged(&canaries)?;
            fs::remove_file(marketplace)
                .map_err(|_| "candidate-acceptance-conflict-cleanup-failed")?;
        }
        let apply = run_product(binary, root, &home, apply_args.clone())?;
        let apply_json = parse_output_json(&apply)?;
        if apply_json["install_id"] != install_id
            || apply_json["outstanding_host_action"] != "install-or-enable-plugin"
        {
            return Err("candidate-acceptance-apply-output-invalid");
        }
        assert_canaries_unchanged(&canaries)?;
        let verify = run_product(
            binary,
            root,
            &home,
            [
                OsStr::new("install"),
                OsStr::new("candidate"),
                OsStr::new("verify"),
                OsStr::new("--target"),
                OsStr::new(target),
                OsStr::new("--install-id"),
                OsStr::new(&install_id),
            ],
        )?;
        if parse_output_json(&verify)?["state"] != "healthy" {
            return Err("candidate-acceptance-verify-output-invalid");
        }
        assert_canaries_unchanged(&canaries)?;
        discover_native_candidate_managed_root(&home).map_err(|error| error.reason_code())?;
        let installed_binary = installed_candidate_binary(&home, artifact)?;
        check_cli_entry(&installed_binary, root, &home)?;
        run_mcp(&installed_binary, root, &home)?;
        // A fresh process must read the same installed resources after shutdown.
        run_mcp(&installed_binary, root, &home)?;
        assert_canaries_unchanged(&canaries)?;
        match target {
            "codex" => {
                if let Some(client) = &external_clients.codex {
                    codex_client_evidence = run_real_codex_client(root, &home, client)?;
                }
            }
            "claude" => {
                if let Some(client) = &external_clients.claude {
                    claude_client_evidence = run_real_claude_client(root, &home, client)?;
                }
            }
            _ => return Err("candidate-acceptance-client-target-invalid"),
        }
        run_managed_fixture_operation(&installed_binary, root, &home, "cli-install", None)?;
        let command_binary = if cfg!(windows) {
            home.join("AppData/Local/Qiongli/bin/qiongli.exe")
        } else {
            home.join(".local/bin/qiongli")
        };
        check_cli_entry(&command_binary, root, &home)?;
        run_mcp(&command_binary, root, &home)?;
        qiongli::check_native_cli_health(&home, &home.join(".qiongli/config"), verified_candidate)?;
        fs::write(&command_binary, b"untrusted-command-canary")
            .map_err(|_| "candidate-acceptance-health-canary-write-failed")?;
        if qiongli::check_native_cli_health(
            &home,
            &home.join(".qiongli/config"),
            verified_candidate,
        ) != Err("native-activation-health-binary-mismatch")
            || fs::read(&command_binary)
                .map_err(|_| "candidate-acceptance-health-canary-read-failed")?
                != b"untrusted-command-canary"
        {
            return Err("candidate-acceptance-health-canary-invalid");
        }
        fs::copy(&installed_binary, &command_binary)
            .map_err(|_| "candidate-acceptance-health-command-restore-failed")?;

        #[cfg(unix)]
        {
            const PROFILE_CANARY: &[u8] = b"# candidate shell profile canary\n";
            let profile = home.join(".bash_profile");
            write_private_new(&profile, PROFILE_CANARY)?;
            run_managed_fixture_operation(
                &command_binary,
                root,
                &home,
                "cli-path-configure",
                None,
            )?;
            if !fs::read(&profile)
                .map_err(|_| "candidate-acceptance-profile-read-failed")?
                .starts_with(PROFILE_CANARY)
            {
                return Err("candidate-acceptance-profile-canary-changed");
            }
            let login = product_command(Path::new("/bin/bash"), root, &home)
                .args(["--login", "-c", "qiongli --version"])
                .output()
                .map_err(|_| "candidate-acceptance-login-start-failed")?;
            if !login.status.success()
                || login.stdout != format!("qiongli {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
            {
                return Err("candidate-acceptance-login-path-invalid");
            }
        }
        let removed = run_managed_fixture_operation(
            &command_binary,
            root,
            &home,
            "integrations-remove",
            Some(target),
        )?;
        if removed["result"] != "removed" {
            return Err("candidate-acceptance-managed-remove-invalid");
        }
        run_managed_fixture_operation(&installed_binary, root, &home, "cli-remove", None)?;
        if command_binary.exists() {
            return Err("candidate-acceptance-command-remove-invalid");
        }

        // Restore through the already-approved candidate so the original complete
        // candidate removal check still covers payload, source and registration.
        run_product(binary, root, &home, apply_args)?;
        assert_canaries_unchanged(&canaries)?;
        let remove = run_product(
            binary,
            root,
            &home,
            [
                OsStr::new("install"),
                OsStr::new("candidate"),
                OsStr::new("remove"),
                OsStr::new("--target"),
                OsStr::new(target),
                OsStr::new("--install-id"),
                OsStr::new(&install_id),
                OsStr::new("--approve-filesystem-write"),
                OsStr::new("--approve-client-config-change"),
            ],
        )?;
        if parse_output_json(&remove)?["payload_disposition"] != "removed" {
            return Err("candidate-acceptance-remove-invalid");
        }
        assert_canaries_unchanged(&canaries)?;
    }

    let real_client_status = match (
        external_clients.codex.is_some(),
        external_clients.claude.is_some(),
    ) {
        (true, true) => ("passed", Value::Null),
        (false, false) => (
            "not-run",
            Value::String("external-client-not-provided".to_string()),
        ),
        _ => (
            "partial",
            Value::String("both-external-clients-required".to_string()),
        ),
    };
    Ok(AcceptanceOutcome {
        checks: json!({
            "runtime_path": "empty",
            "checkout_boundary": "outside-checkout",
            "version": "passed",
            "cli_empty_arguments": "passed",
            "cli_window_refusal": "passed",
            "installed_payload_runtime": "passed",
            "installed_payload_restart": "passed",
            "installed_product_managed_operation": "passed",
            "installed_command_runtime_and_authority": "passed",
            "installed_command_remove": "passed",
            "unix_login_path": if cfg!(unix) { "passed" } else { "not-run" },
            "embedded_skills": "passed",
            "ui_startup_preflight": "passed",
            "lite_mcp": "passed",
            "codex_local_lifecycle": "passed",
            "claude_code_local_lifecycle": "passed",
            "digest_and_partial_approval_rejection": "passed",
            "candidate_stage_preview_approval_replay_and_host_isolation": "passed",
            "installed_cli_child_health_and_tamper_refusal": "passed",
            "fresh_failure_compensation": "passed",
            "clean_install": "passed",
            "uninstall": "passed",
            "user_project_preservation": "passed",
            "global_v2_state_preservation": "passed",
            "unmanaged_host_state_preservation": "passed",
            "path_redaction": "passed",
            "unrelated_state_preservation": "passed"
        }),
        real_client: json!({
            "status": real_client_status.0,
            "reason": real_client_status.1,
            "isolation": "fresh-home-and-client-config",
            "candidate_backed": external_clients.codex.is_some() || external_clients.claude.is_some(),
            "codex": codex_client_evidence,
            "claude_code": claude_client_evidence
        }),
    })
}

fn check_candidate_staging(
    binary: &Path,
    root: &Path,
    candidate: &Path,
    archive: &Path,
    notes: &Path,
) -> Result<(), &'static str> {
    for target in ["codex", "claude"] {
        let home = create_child_directory(root, &format!("stage-{target}-home"))?;
        let mut args = vec![
            OsString::from("install"),
            "candidate".into(),
            "stage-preview".into(),
            "--candidate".into(),
            candidate.into(),
            "--archive".into(),
            archive.into(),
            "--release-notes".into(),
            notes.into(),
            "--target".into(),
            target.into(),
        ];
        let preview = parse_output_json(&run_product(binary, root, &home, &args)?)?;
        let digest = preview["approval_digest_sha256"]
            .as_str()
            .filter(|value| valid_sha256(value))
            .ok_or("candidate-stage-preview-invalid")?;
        if preview["command"] != "install-candidate-stage-preview"
            || preview["state"] != "ready"
            || preview["approvals_required"] != json!(["filesystem-write"])
            || candidate_owned_state_present(&home, target)
        {
            return Err("candidate-stage-preview-invalid");
        }
        args[2] = "preview".into();
        let install = parse_output_json(&run_product(binary, root, &home, &args)?)?;
        let install_digest = install["approval_digest_sha256"]
            .as_str()
            .ok_or("candidate-stage-install-digest-invalid")?;
        args[2] = "stage".into();
        args.extend([OsString::from("--expected-approval-digest"), digest.into()]);
        run_product_failure(binary, root, &home, &args, 2)?;
        args.push("--approve-filesystem-write".into());
        let mut wrong = args.clone();
        let digest_index = wrong.len() - 2;
        wrong[digest_index] = install_digest.into();
        run_product_expected_failure(
            binary,
            root,
            &home,
            &wrong,
            "native-candidate-stage-approval-digest-mismatch",
        )?;
        let mut reverse = args.clone();
        reverse[2] = "apply".into();
        reverse.extend([
            OsString::from("--approve-client-config-change"),
            "--approve-host-trust".into(),
        ]);
        run_product_expected_failure(
            binary,
            root,
            &home,
            &reverse,
            "native-candidate-approval-digest-mismatch",
        )?;
        if candidate_owned_state_present(&home, target) {
            return Err("candidate-stage-rejection-mutated-state");
        }
        for state in ["staged", "already-staged"] {
            let actual = parse_output_json(&run_product(binary, root, &home, &args)?)?;
            let mut expected = preview.clone();
            expected["command"] = "install-candidate-stage".into();
            expected["state"] = state.into();
            if actual != expected || !home.join(".qiongli/native/payloads").is_dir() {
                return Err("candidate-stage-output-invalid");
            }
            for path in [
                ".agents",
                ".codex",
                ".claude",
                ".qiongli/plugins",
                ".local",
                "AppData",
                ".qiongli/v2/cli",
            ] {
                if home.join(path).exists() {
                    return Err("candidate-stage-mutated-host-or-command");
                }
            }
        }
    }
    Ok(())
}

fn run_real_codex_client(
    root: &Path,
    home: &Path,
    client: &CodexClient,
) -> Result<Value, &'static str> {
    let source_root = home.join(".qiongli/plugins/codex/qiongli-next");
    let source_target = approve_codex_plugin_bundle_target(&source_root)
        .map_err(|_| "candidate-acceptance-codex-source-invalid")?;
    let source = verify_codex_plugin_bundle(&source_target)
        .map_err(|_| "candidate-acceptance-codex-source-invalid")?;
    let source_receipt_sha256 = source.receipt_sha256().to_string();

    let mut validator = Command::new(&client.validator_python);
    validator
        .arg(&client.validator)
        .arg(&source_root)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("XDG_CACHE_HOME", home.join(".cache"))
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local/share"))
        .env("PYTHONNOUSERSITE", "1");
    run_external_command(
        validator,
        "candidate-acceptance-codex-validator-start-failed",
        "candidate-acceptance-codex-validator-failed",
    )?;

    let codex_home = ensure_private_child_directory(home, ".codex")?;
    let version = run_external_command(
        isolated_codex_command(&client.binary, home, &codex_home, [OsStr::new("--version")]),
        "candidate-acceptance-codex-start-failed",
        "candidate-acceptance-codex-version-failed",
    )?;
    let version = public_client_version(&version.stdout, root)?;
    run_external_command(
        isolated_codex_command(
            &client.binary,
            home,
            &codex_home,
            [
                OsStr::new("plugin"),
                OsStr::new("add"),
                OsStr::new("--json"),
                OsStr::new("qiongli-next@personal"),
            ],
        ),
        "candidate-acceptance-codex-start-failed",
        "candidate-acceptance-codex-install-failed",
    )?;
    let listed = run_external_command(
        isolated_codex_command(
            &client.binary,
            home,
            &codex_home,
            [
                OsStr::new("plugin"),
                OsStr::new("list"),
                OsStr::new("--json"),
            ],
        ),
        "candidate-acceptance-codex-start-failed",
        "candidate-acceptance-codex-list-failed",
    )?;
    if !String::from_utf8_lossy(&listed.stdout).contains("qiongli-next") {
        return Err("candidate-acceptance-codex-list-invalid");
    }
    let cached_root = find_cached_bundle(
        &codex_home.join("plugins/cache"),
        CODEX_PLUGIN_BUNDLE_RECEIPT_FILE,
        0,
    )?
    .ok_or("candidate-acceptance-codex-cache-missing")?;
    let cached_target = approve_codex_plugin_bundle_target(&cached_root)
        .map_err(|_| "candidate-acceptance-codex-cache-invalid")?;
    let cached = verify_codex_plugin_bundle(&cached_target)
        .map_err(|_| "candidate-acceptance-codex-cache-invalid")?;
    if cached.receipt_sha256() != source_receipt_sha256 {
        return Err("candidate-acceptance-codex-cache-drift");
    }
    run_cached_mcp(&cached_root.join(&cached.receipt().binary_path), root, home)?;
    run_external_command(
        isolated_codex_command(
            &client.binary,
            home,
            &codex_home,
            [
                OsStr::new("plugin"),
                OsStr::new("remove"),
                OsStr::new("--json"),
                OsStr::new("qiongli-next@personal"),
            ],
        ),
        "candidate-acceptance-codex-start-failed",
        "candidate-acceptance-codex-remove-failed",
    )?;
    let after = run_external_command(
        isolated_codex_command(
            &client.binary,
            home,
            &codex_home,
            [
                OsStr::new("plugin"),
                OsStr::new("list"),
                OsStr::new("--json"),
            ],
        ),
        "candidate-acceptance-codex-start-failed",
        "candidate-acceptance-codex-list-failed",
    )?;
    if String::from_utf8_lossy(&after.stdout).contains("qiongli-next") {
        return Err("candidate-acceptance-codex-remove-invalid");
    }

    Ok(json!({
        "status": "passed",
        "client_version": version,
        "plugin_creator_valid": true,
        "candidate_source_receipt_sha256": source_receipt_sha256,
        "client_install_succeeded": true,
        "client_listed_plugin": true,
        "client_cache_verified": true,
        "cached_mcp_empty_path_succeeded": true,
        "client_remove_succeeded": true,
        "client_absence_verified": true,
        "lite_tool_count": LITE_PUBLIC_TOOL_NAMES.len(),
        "full_tool_count": LITE_PUBLIC_TOOL_NAMES.len() + FULL_PROJECT_PUBLIC_TOOL_NAMES.len()
            + FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES.len(),
        "full_route_profile_verified": true
    }))
}

fn run_real_claude_client(root: &Path, home: &Path, client: &Path) -> Result<Value, &'static str> {
    let marketplace_root = home.join(".qiongli/plugins/claude-code/qiongli-local");
    let source_root = marketplace_root.join("plugins/qiongli-next");
    let source_target = approve_claude_plugin_bundle_target(&source_root)
        .map_err(|_| "candidate-acceptance-claude-source-invalid")?;
    let source = verify_claude_plugin_bundle(&source_target)
        .map_err(|_| "candidate-acceptance-claude-source-invalid")?;
    let source_receipt_sha256 = source.receipt_sha256().to_string();
    let claude_config = ensure_private_child_directory(home, ".claude")?;

    let version = run_external_command(
        isolated_claude_command(client, home, &claude_config, [OsStr::new("--version")]),
        "candidate-acceptance-claude-start-failed",
        "candidate-acceptance-claude-version-failed",
    )?;
    let version = public_client_version(&version.stdout, root)?;
    run_external_command(
        isolated_claude_command(
            client,
            home,
            &claude_config,
            [
                OsStr::new("plugin"),
                OsStr::new("validate"),
                OsStr::new("--strict"),
                marketplace_root.as_os_str(),
            ],
        ),
        "candidate-acceptance-claude-start-failed",
        "candidate-acceptance-claude-validator-failed",
    )?;
    run_external_command(
        isolated_claude_command(
            client,
            home,
            &claude_config,
            [
                OsStr::new("plugin"),
                OsStr::new("marketplace"),
                OsStr::new("add"),
                marketplace_root.as_os_str(),
                OsStr::new("--scope"),
                OsStr::new("user"),
            ],
        ),
        "candidate-acceptance-claude-start-failed",
        "candidate-acceptance-claude-marketplace-add-failed",
    )?;
    run_external_command(
        isolated_claude_command(
            client,
            home,
            &claude_config,
            [
                OsStr::new("plugin"),
                OsStr::new("install"),
                OsStr::new("qiongli-next@qiongli-local"),
                OsStr::new("--scope"),
                OsStr::new("user"),
            ],
        ),
        "candidate-acceptance-claude-start-failed",
        "candidate-acceptance-claude-install-failed",
    )?;
    let listed = run_external_command(
        isolated_claude_command(
            client,
            home,
            &claude_config,
            [
                OsStr::new("plugin"),
                OsStr::new("list"),
                OsStr::new("--json"),
            ],
        ),
        "candidate-acceptance-claude-start-failed",
        "candidate-acceptance-claude-list-failed",
    )?;
    if !String::from_utf8_lossy(&listed.stdout).contains("qiongli-next@qiongli-local") {
        return Err("candidate-acceptance-claude-list-invalid");
    }
    let cached_root = find_cached_bundle(
        &claude_config.join("plugins/cache"),
        CLAUDE_PLUGIN_BUNDLE_RECEIPT_FILE,
        0,
    )?
    .ok_or("candidate-acceptance-claude-cache-missing")?;
    let cached_target = approve_claude_plugin_bundle_target(&cached_root)
        .map_err(|_| "candidate-acceptance-claude-cache-invalid")?;
    let cached = verify_claude_plugin_bundle(&cached_target)
        .map_err(|_| "candidate-acceptance-claude-cache-invalid")?;
    if cached.receipt_sha256() != source_receipt_sha256 {
        return Err("candidate-acceptance-claude-cache-drift");
    }
    run_cached_mcp(&cached_root.join(&cached.receipt().binary_path), root, home)?;
    run_external_command(
        isolated_claude_command(
            client,
            home,
            &claude_config,
            [
                OsStr::new("plugin"),
                OsStr::new("uninstall"),
                OsStr::new("qiongli-next@qiongli-local"),
                OsStr::new("--scope"),
                OsStr::new("user"),
            ],
        ),
        "candidate-acceptance-claude-start-failed",
        "candidate-acceptance-claude-remove-failed",
    )?;
    run_external_command(
        isolated_claude_command(
            client,
            home,
            &claude_config,
            [
                OsStr::new("plugin"),
                OsStr::new("marketplace"),
                OsStr::new("remove"),
                OsStr::new("qiongli-local"),
            ],
        ),
        "candidate-acceptance-claude-start-failed",
        "candidate-acceptance-claude-marketplace-remove-failed",
    )?;
    let after = run_external_command(
        isolated_claude_command(
            client,
            home,
            &claude_config,
            [
                OsStr::new("plugin"),
                OsStr::new("list"),
                OsStr::new("--json"),
            ],
        ),
        "candidate-acceptance-claude-start-failed",
        "candidate-acceptance-claude-list-failed",
    )?;
    if String::from_utf8_lossy(&after.stdout).contains("qiongli-next@qiongli-local") {
        return Err("candidate-acceptance-claude-remove-invalid");
    }

    Ok(json!({
        "status": "passed",
        "client_version": version,
        "strict_plugin_validation": true,
        "candidate_source_receipt_sha256": source_receipt_sha256,
        "local_marketplace_added": true,
        "client_install_succeeded": true,
        "client_listed_plugin": true,
        "client_cache_verified": true,
        "cached_mcp_empty_path_succeeded": true,
        "client_remove_succeeded": true,
        "client_absence_verified": true,
        "lite_tool_count": LITE_PUBLIC_TOOL_NAMES.len(),
        "full_tool_count": LITE_PUBLIC_TOOL_NAMES.len() + FULL_PROJECT_PUBLIC_TOOL_NAMES.len()
            + FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES.len(),
        "full_route_profile_verified": true
    }))
}

fn isolated_codex_command<I, S>(binary: &Path, home: &Path, codex_home: &Path, args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(binary);
    command
        .args(args)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("CODEX_HOME", codex_home)
        .env("XDG_CACHE_HOME", home.join(".cache"))
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local/share"))
        .env("NO_COLOR", "1");
    command
}

fn isolated_claude_command<I, S>(
    binary: &Path,
    home: &Path,
    claude_config: &Path,
    args: I,
) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(binary);
    command
        .args(args)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("CLAUDE_CONFIG_DIR", claude_config)
        .env("XDG_CACHE_HOME", home.join(".cache"))
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local/share"))
        .env("NO_COLOR", "1");
    command
}

fn run_external_command(
    mut command: Command,
    start_error: &'static str,
    failure_error: &'static str,
) -> Result<Output, &'static str> {
    let output = command.output().map_err(|_| start_error)?;
    if output.stdout.len().saturating_add(output.stderr.len()) > 4 * 1024 * 1024 {
        return Err(failure_error);
    }
    if !output.status.success() {
        return Err(failure_error);
    }
    Ok(output)
}

fn public_client_version(bytes: &[u8], private_root: &Path) -> Result<String, &'static str> {
    let version = std::str::from_utf8(bytes)
        .map_err(|_| "candidate-acceptance-client-version-invalid")?
        .trim();
    if version.is_empty()
        || version.len() > 256
        || version.chars().any(char::is_control)
        || version.contains(private_root.to_string_lossy().as_ref())
    {
        return Err("candidate-acceptance-client-version-invalid");
    }
    Ok(version.to_string())
}

fn find_cached_bundle(
    root: &Path,
    receipt_file: &str,
    depth: usize,
) -> Result<Option<PathBuf>, &'static str> {
    if depth > 8 || !root.exists() {
        return Ok(None);
    }
    let root_metadata =
        fs::symlink_metadata(root).map_err(|_| "candidate-acceptance-client-cache-invalid")?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err("candidate-acceptance-client-cache-invalid");
    }
    let mut entry_count = 0_usize;
    for entry in fs::read_dir(root).map_err(|_| "candidate-acceptance-client-cache-invalid")? {
        entry_count = entry_count.saturating_add(1);
        if entry_count > 4_096 {
            return Err("candidate-acceptance-client-cache-invalid");
        }
        let entry = entry.map_err(|_| "candidate-acceptance-client-cache-invalid")?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|_| "candidate-acceptance-client-cache-invalid")?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        let path = entry.path();
        if path.join(receipt_file).is_file() {
            return Ok(Some(path));
        }
        if let Some(found) = find_cached_bundle(&path, receipt_file, depth.saturating_add(1))? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

fn ensure_private_child_directory(root: &Path, leaf: &str) -> Result<PathBuf, &'static str> {
    let path = root.join(leaf);
    if path.exists() {
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| "candidate-acceptance-directory-create-failed")?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("candidate-acceptance-directory-create-failed");
        }
        Ok(path)
    } else {
        create_private_directory(&path)?;
        Ok(path)
    }
}

fn run_managed_fixture_operation(
    binary: &Path,
    root: &Path,
    home: &Path,
    operation: &str,
    target: Option<&str>,
) -> Result<Value, &'static str> {
    let mut plan_args = vec!["app", "plan", operation];
    if let Some(target) = target {
        plan_args.extend(["--target", target]);
    }
    let plan = run_product(binary, root, home, plan_args)?;
    let plan_json = parse_output_json(&plan)?;
    let digest = plan_json["plan_digest_sha256"]
        .as_str()
        .filter(|value| valid_sha256(value))
        .ok_or("candidate-acceptance-managed-plan-invalid")?;
    let path = home.join(format!("managed-{operation}-plan.json"));
    write_private_new(&path, &plan.stdout)?;
    let args = vec![
        OsString::from("app"),
        OsString::from("apply"),
        OsString::from("--plan"),
        path.into_os_string(),
        OsString::from("--expected-plan-digest"),
        OsString::from(digest),
    ];
    run_product_expected_failure(
        binary,
        root,
        home,
        args.clone(),
        "managed-operation-approval-required",
    )?;
    let mut approved = args;
    approved.push(OsString::from("--approve-filesystem-write"));
    if operation == "integrations-remove" {
        approved
            .extend(["--approve-client-config-change", "--approve-host-trust"].map(OsString::from));
    }
    let result = run_product(binary, root, home, approved)?;
    let result = parse_output_json(&result)?;
    if result["operation"] != operation {
        return Err("candidate-acceptance-managed-operation-invalid");
    }
    Ok(result)
}

fn installed_candidate_binary(
    home: &Path,
    artifact: &ArtifactIdentityV1,
) -> Result<PathBuf, &'static str> {
    Ok(home
        .join(".qiongli/native/payloads")
        .join(native_artifact_id(artifact).map_err(|error| error.reason_code())?)
        .join(native_artifact_binary_path(artifact).map_err(|error| error.reason_code())?))
}

fn check_cli_entry(binary: &Path, root: &Path, home: &Path) -> Result<(), &'static str> {
    let help = run_product(binary, root, home, ["--help"])?;
    let empty = run_product(binary, root, home, std::iter::empty::<&str>())?;
    if help.stdout.is_empty() || help.stdout != empty.stdout {
        return Err("candidate-acceptance-cli-help-invalid");
    }
    run_product_expected_failure(
        binary,
        root,
        home,
        ["ui"],
        qiongli::DESKTOP_STARTUP_ERROR_CODE,
    )
}

fn run_mcp(binary: &Path, root: &Path, home: &Path) -> Result<(), &'static str> {
    let mut command = product_command(binary, root, home);
    command
        .args([
            "mcp",
            "serve",
            "--transport",
            "stdio",
            "--profile",
            "marketplace-lite",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    run_mcp_command(command, root, false)
}

fn run_cached_mcp(executable: &Path, root: &Path, home: &Path) -> Result<(), &'static str> {
    for profile in ["marketplace-lite", "full"] {
        let mut command = Command::new(executable);
        command
            .env_clear()
            .env("PATH", "")
            .env("HOME", home)
            .env("USERPROFILE", home)
            .env("QIONGLI_CONFIG_HOME", home.join(".qiongli/config"))
            .current_dir(root)
            .args(["mcp", "serve", "--transport", "stdio", "--profile", profile])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in ["SYSTEMROOT", "WINDIR", "TEMP", "TMP", "TMPDIR"] {
            if let Some(value) = env::var_os(name) {
                command.env(name, value);
            }
        }
        run_mcp_command(command, root, profile == "full")?;
    }
    Ok(())
}

fn run_mcp_command(mut command: Command, root: &Path, full: bool) -> Result<(), &'static str> {
    let mut child = command
        .spawn()
        .map_err(|_| "candidate-acceptance-mcp-start-failed")?;
    let mut requests = vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2025-11-25", "capabilities": {}}
        }),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "qiongli_config_status", "arguments": {}}
        }),
    ];
    if full {
        requests.push(json!({
            "jsonrpc": "2.0", "id": 4, "method": "tools/call",
            "params": {"name": "qiongli_orchestrator_route", "arguments": {
                "request": "Compare two literature sources with independent review and auditable handoff."
            }}
        }));
    }
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("candidate-acceptance-mcp-stdin-unavailable")?;
        for request in requests {
            serde_json::to_writer(&mut *stdin, &request)
                .map_err(|_| "candidate-acceptance-mcp-request-invalid")?;
            stdin
                .write_all(b"\n")
                .map_err(|_| "candidate-acceptance-mcp-request-write-failed")?;
        }
    }
    drop(child.stdin.take());
    let output = child
        .wait_with_output()
        .map_err(|_| "candidate-acceptance-mcp-wait-failed")?;
    validate_public_output(&output, root)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err("candidate-acceptance-mcp-failed");
    }
    let stdout =
        String::from_utf8(output.stdout).map_err(|_| "candidate-acceptance-mcp-output-invalid")?;
    let responses = stdout
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "candidate-acceptance-mcp-output-invalid")?;
    if responses.len() != if full { 4 } else { 3 } {
        return Err("candidate-acceptance-mcp-output-invalid");
    }
    let tools = responses
        .iter()
        .find(|value| value["id"] == 2)
        .and_then(|value| value["result"]["tools"].as_array())
        .ok_or("candidate-acceptance-mcp-tools-invalid")?;
    let names = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    let mut expected = LITE_PUBLIC_TOOL_NAMES.to_vec();
    if full {
        expected.extend(FULL_PROJECT_PUBLIC_TOOL_NAMES);
        expected.extend(FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES);
    }
    if names != expected {
        return Err("candidate-acceptance-mcp-tools-invalid");
    }
    let call = responses
        .iter()
        .find(|value| value["id"] == 3)
        .ok_or("candidate-acceptance-mcp-call-invalid")?;
    if call["result"]["structuredContent"]["config_path"] != "<managed-native-config>" {
        return Err("candidate-acceptance-mcp-call-invalid");
    }
    if full {
        let route = responses
            .iter()
            .find(|value| value["id"] == 4)
            .ok_or("candidate-acceptance-full-route-invalid")?;
        validate_full_route(&route["result"]["structuredContent"])?;
    }
    Ok(())
}

fn validate_full_route(route: &Value) -> Result<(), &'static str> {
    if route["route"] != "orchestrator_mcp"
        || route["requires_full_runtime"] != true
        || [
            "preview_only",
            "runtime_profile",
            "recommended_runtime",
            "upgrade",
        ]
        .iter()
        .any(|field| route.get(field).is_some())
    {
        return Err("candidate-acceptance-full-route-invalid");
    }
    Ok(())
}

fn run_product<I, S>(
    binary: &Path,
    root: &Path,
    home: &Path,
    args: I,
) -> Result<Output, &'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = product_command(binary, root, home)
        .args(args)
        .output()
        .map_err(|_| "candidate-acceptance-product-start-failed")?;
    validate_public_output(&output, root)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err("candidate-acceptance-product-command-failed");
    }
    Ok(output)
}

fn run_product_failure<I, S>(
    binary: &Path,
    root: &Path,
    home: &Path,
    args: I,
    expected_exit_code: i32,
) -> Result<(), &'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = product_command(binary, root, home)
        .args(args)
        .output()
        .map_err(|_| "candidate-acceptance-product-start-failed")?;
    validate_public_output(&output, root)?;
    if output.status.code() != Some(expected_exit_code)
        || !output.stdout.is_empty()
        || output.stderr.is_empty()
    {
        return Err("candidate-acceptance-product-failure-contract-invalid");
    }
    Ok(())
}

fn run_product_expected_failure<I, S>(
    binary: &Path,
    root: &Path,
    home: &Path,
    args: I,
    expected_reason: &str,
) -> Result<(), &'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = product_command(binary, root, home)
        .args(args)
        .output()
        .map_err(|_| "candidate-acceptance-product-start-failed")?;
    validate_public_output(&output, root)?;
    let expected_stderr = format!("error: {expected_reason}\n");
    if output.status.code() != Some(1)
        || !output.stdout.is_empty()
        || output.stderr != expected_stderr.as_bytes()
    {
        return Err("candidate-acceptance-product-failure-reason-invalid");
    }
    Ok(())
}

fn product_command(binary: &Path, root: &Path, home: &Path) -> Command {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .env("PATH", "")
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("QIONGLI_CONFIG_HOME", home.join(".qiongli/config"))
        .current_dir(root);
    #[cfg(unix)]
    command.env("SHELL", "/bin/bash");
    for name in ["SYSTEMROOT", "WINDIR", "TEMP", "TMP", "TMPDIR"] {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}

#[cfg(unix)]
fn write_private_new(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "candidate-acceptance-private-write-failed")?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "candidate-acceptance-private-write-failed")
}

#[cfg(windows)]
fn write_private_new(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    let mut file = qiongli_windows_security::create_owner_only_new_file(path)
        .map_err(|_| "candidate-acceptance-private-write-failed")?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "candidate-acceptance-private-write-failed")
}

#[cfg(not(any(unix, windows)))]
fn write_private_new(_path: &Path, _bytes: &[u8]) -> Result<(), &'static str> {
    Err("candidate-acceptance-platform-unsupported")
}

fn parse_output_json(output: &Output) -> Result<Value, &'static str> {
    serde_json::from_slice(&output.stdout)
        .map_err(|_| "candidate-acceptance-product-output-invalid")
}

fn validate_public_output(output: &Output, root: &Path) -> Result<(), &'static str> {
    let root = root.to_string_lossy();
    if String::from_utf8_lossy(&output.stdout).contains(root.as_ref())
        || String::from_utf8_lossy(&output.stderr).contains(root.as_ref())
    {
        return Err("candidate-acceptance-private-path-leaked");
    }
    Ok(())
}

fn create_child_directory(root: &Path, leaf: &str) -> Result<PathBuf, &'static str> {
    let path = root.join(leaf);
    create_private_directory(&path)?;
    Ok(path)
}

fn payload_directory_present(root: &Path) -> Result<bool, &'static str> {
    if !root.exists() {
        return Ok(false);
    }
    let entries =
        fs::read_dir(root).map_err(|_| "candidate-acceptance-payload-directory-read-failed")?;
    for entry in entries {
        let entry = entry.map_err(|_| "candidate-acceptance-payload-directory-read-failed")?;
        if entry
            .file_type()
            .map_err(|_| "candidate-acceptance-payload-directory-read-failed")?
            .is_dir()
            && !entry.file_name().to_string_lossy().starts_with('.')
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn candidate_owned_state_present(home: &Path, target: &str) -> bool {
    let plugin = match target {
        "codex" => home.join(".qiongli/plugins/codex/qiongli-next"),
        "claude" => home.join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next"),
        _ => return true,
    };
    home.join(".qiongli/native").exists() || plugin.exists()
}

fn assert_canaries_unchanged(canaries: &[(PathBuf, &[u8])]) -> Result<(), &'static str> {
    for (path, expected) in canaries {
        let actual = fs::read(path).map_err(|_| "candidate-acceptance-canary-read-failed")?;
        if actual != *expected {
            return Err("candidate-acceptance-canary-drift");
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|_| "candidate-acceptance-directory-create-failed")
}

#[cfg(windows)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    qiongli_windows_security::create_owner_only_directory(path)
        .map(|_| ())
        .map_err(|_| "candidate-acceptance-directory-create-failed")
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(_path: &Path) -> Result<(), &'static str> {
    Err("candidate-acceptance-platform-unsupported")
}

fn random_seed() -> Result<[u8; 32], &'static str> {
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|_| "candidate-acceptance-random-unavailable")?;
    Ok(seed)
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

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    encode_hex(&digest)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && valid_lower_hex(value)
}

fn valid_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "candidate-acceptance-clock-unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn external_commands_preserve_symlink_invocation_paths() {
        let root =
            env::temp_dir().join(format!("qiongli-command-path-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let target = root.join("interpreter");
        let command = root.join("venv-python");
        fs::write(&target, b"test interpreter").unwrap();
        std::os::unix::fs::symlink(&target, &command).unwrap();
        assert_eq!(valid_external_command(command.clone()).unwrap(), command);
        assert_eq!(
            valid_external_file(command.clone()).unwrap(),
            fs::canonicalize(&target).unwrap()
        );
        fs::remove_file(target).unwrap();
        assert!(valid_external_command(command).is_err());
        assert!(valid_external_command(root.clone()).is_err());
        assert!(valid_external_command(PathBuf::from("relative-python")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn full_route_rejects_lite_and_incomplete_responses() {
        let full = json!({"route": "orchestrator_mcp", "requires_full_runtime": true});
        assert!(validate_full_route(&full).is_ok());
        assert!(validate_full_route(&Value::Null).is_err());
        let mut incomplete = full.clone();
        incomplete["requires_full_runtime"] = json!(false);
        assert!(validate_full_route(&incomplete).is_err());
        for field in [
            "preview_only",
            "runtime_profile",
            "recommended_runtime",
            "upgrade",
        ] {
            let mut mixed = full.clone();
            mixed[field] = Value::Null;
            assert!(validate_full_route(&mixed).is_err());
        }
    }

    #[test]
    fn predecessor_manifest_is_explicit_and_optional() {
        let mut args = vec![
            OsString::from("--output"),
            env::temp_dir()
                .join("qiongli-predecessor-arguments")
                .into_os_string(),
            OsString::from("--source-commit"),
            OsString::from("0000000000000000000000000000000000000000"),
        ];
        assert!(
            Arguments::parse(args.clone())
                .unwrap()
                .predecessor_manifest
                .is_none()
        );
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        args.extend([
            "--predecessor-manifest".into(),
            manifest.clone().into_os_string(),
        ]);
        assert_eq!(
            Arguments::parse(args.clone()).unwrap().predecessor_manifest,
            Some(fs::canonicalize(manifest).unwrap())
        );
        args.extend(["--predecessor-manifest".into(), "relative.toml".into()]);
        assert!(Arguments::parse(args).is_err());
    }

    #[test]
    fn codex_external_client_arguments_are_all_or_nothing() {
        let output = env::temp_dir().join("qiongli-candidate-argument-output");
        let result = Arguments::parse([
            OsString::from("--output"),
            output.into_os_string(),
            OsString::from("--source-commit"),
            OsString::from("0000000000000000000000000000000000000000"),
            OsString::from("--codex-bin"),
            OsString::from("/missing/codex"),
        ]);
        assert!(matches!(result, Err("candidate-acceptance-usage-invalid")));
    }

    #[test]
    fn public_client_versions_reject_private_paths_and_control_bytes() {
        let root = Path::new("/private/acceptance-root");
        assert_eq!(
            public_client_version(b"codex-cli 1.2.3\n", root).unwrap(),
            "codex-cli 1.2.3"
        );
        assert!(public_client_version(b"line-one\nline-two", root).is_err());
        assert!(public_client_version(b"client /private/acceptance-root", root).is_err());
    }

    #[test]
    fn candidate_owned_state_detection_covers_both_host_sources() {
        let root = env::temp_dir().join(format!(
            "qiongli-candidate-owned-state-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".qiongli/plugins/codex/qiongli-next")).unwrap();
        assert!(candidate_owned_state_present(&root, "codex"));

        fs::remove_dir_all(root.join(".qiongli")).unwrap();
        fs::create_dir_all(
            root.join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next"),
        )
        .unwrap();
        assert!(candidate_owned_state_present(&root, "claude"));
        assert!(candidate_owned_state_present(&root, "unsupported"));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn staged_product_normalizes_a_hard_linked_cargo_style_source() {
        let mut nonce = [0_u8; 16];
        getrandom::fill(&mut nonce).expect("test nonce must be available");
        let root = env::temp_dir().join(format!(
            "qiongli-native-candidate-source-test-{}",
            encode_hex(&nonce)
        ));
        create_private_directory(&root).expect("test root must be private");
        let source = root.join(format!("source{}", env::consts::EXE_SUFFIX));
        fs::write(&source, b"candidate-source-bytes").expect("source must write");
        set_product_executable(&source).expect("source must be executable");
        let linked = root.join(format!("source-linked{}", env::consts::EXE_SUFFIX));
        fs::hard_link(&source, &linked).expect("source hard link must be available");
        assert_eq!(link_count(&source), 2);

        let staged = root.join(format!("staged{}", env::consts::EXE_SUFFIX));
        stage_product_binary(&source, &staged).expect("staging must succeed");

        assert_eq!(link_count(&staged), 1);
        assert_eq!(
            fs::read(&staged).expect("staged bytes must read"),
            b"candidate-source-bytes"
        );
        fs::remove_dir_all(root).expect("test root must clean up");
    }

    #[test]
    fn release_notes_bind_the_exact_current_target_and_limitations() {
        let artifact = current_target_native_artifact_identity(
            env!("CARGO_PKG_VERSION"),
            ReleaseChannel::Alpha,
        )
        .expect("current target must be supported");
        let artifact_id = native_artifact_id(&artifact).expect("artifact ID must be valid");
        let archive_name =
            native_portable_archive_file_name(&artifact).expect("archive name must be valid");
        let candidate_name =
            native_release_candidate_file_name(&artifact).expect("candidate name must be valid");
        let notes_name =
            native_release_notes_file_name(&artifact).expect("notes name must be valid");

        let notes = render_release_notes(
            &artifact,
            &artifact_id,
            &archive_name,
            &candidate_name,
            &notes_name,
        )
        .expect("release notes must render");
        let notes = String::from_utf8(notes).expect("release notes must be UTF-8");
        let normalized_notes = notes.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(normalized_notes.contains(&format!(
            "`{} / {}`",
            operating_system(artifact.os),
            architecture(artifact.arch)
        )));
        assert!(normalized_notes.contains(&artifact_id));
        assert!(
            normalized_notes
                .contains("displayed window and accessibility evidence remain external")
        );
        assert!(normalized_notes.contains("community-alpha — not platform-trusted"));
        assert!(normalized_notes.contains("Raw CI artifacts must not be uploaded directly"));
        assert!(!notes.contains("{{"));
    }

    #[cfg(unix)]
    fn link_count(path: &Path) -> u64 {
        use std::os::unix::fs::MetadataExt;

        fs::symlink_metadata(path)
            .expect("link metadata must exist")
            .nlink()
    }

    #[cfg(windows)]
    fn link_count(path: &Path) -> u64 {
        let file = File::open(path).expect("linked file must open");
        u64::from(
            qiongli_windows_security::handle_facts(&file)
                .expect("link facts must be available")
                .number_of_links,
        )
    }
}
