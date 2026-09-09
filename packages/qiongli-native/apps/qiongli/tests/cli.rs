#![allow(clippy::disallowed_methods)]

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use qiongli_config::UPDATE_STATE_FILE;
use qiongli_config::{
    EmailAddress, GLOBAL_SETTINGS_FILE, GlobalSettings, GlobalSettingsStore, resolve_config_root,
};
use qiongli_content::{MATERIALIZATION_RECEIPT_FILE, ProfileId};
use qiongli_project::{
    ApprovedCaptureIntake, ApprovedProjectMutation, CaptureArea, CaptureDelivery,
    CaptureDeliveryDestinationV1, CaptureDeliveryEnvelopeV1, CaptureDeliveryState, CapturePolicy,
    CaptureSource, ContradictionV1, DecisionCandidateV1, DecisionRelation, EvidenceLocatorKind,
    EvidenceReferenceV1, PortfolioQueryFiltersV1, PortfolioQueryV1, ProjectBindingV1, ProjectId,
    ProjectKind, ProjectRegistrationOptions, ProjectStage, ProjectStateService,
    ResearchCaptureDraftV1, ResearchCaptureV1, SemanticChangeV1, SemanticTimelineQueryV1,
};
use serde_json::Value;

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    config_root: PathBuf,
    home: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-cli-tests");
        fs::create_dir_all(&test_base).expect("CLI test base must be created");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let root = test_base.join(format!(
            "{name}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("isolated CLI root must be created");
        set_private_directory_mode(&root);
        let home = root.join("home");
        fs::create_dir(&home).expect("isolated CLI home must be created");
        set_private_directory_mode(&home);
        let config_root = root.join("private-config-path-canary");
        Self {
            root,
            config_root,
            home,
        }
    }

    fn store(&self) -> GlobalSettingsStore {
        let root = resolve_config_root(Some(self.config_root.as_os_str()), &self.home)
            .expect("fixture config root must resolve");
        GlobalSettingsStore::new(root)
    }

    fn state_root(&self) -> PathBuf {
        self.config_root.join("v2")
    }

    fn settings_path(&self) -> PathBuf {
        self.state_root().join(GLOBAL_SETTINGS_FILE)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn set_private_directory_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture directory mode must be private");
}

#[cfg(not(unix))]
fn set_private_directory_mode(_path: &Path) {}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .args(args)
        .output()
        .expect("native qiongli binary should start")
}

fn run_without_path(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .args(args)
        .env("PATH", "")
        .output()
        .expect("native qiongli binary should start without PATH")
}

fn run_without_home_or_path(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .args(args)
        .env("PATH", "")
        .env_remove("QIONGLI_CONFIG_HOME")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .env_remove("HOMEDRIVE")
        .env_remove("HOMEPATH")
        .output()
        .expect("native qiongli binary should start without home or PATH")
}

fn fixture_command(executable: &Path, fixture: &Fixture) -> Command {
    let mut command = Command::new(executable);
    command
        .current_dir(&fixture.root)
        .env("QIONGLI_CONFIG_HOME", &fixture.config_root)
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home);
    command
}

fn output_with_executable_busy_retry(command: &mut Command) -> io::Result<Output> {
    const MAX_ATTEMPTS: usize = 5;
    for attempt in 0..MAX_ATTEMPTS {
        match command.output() {
            Err(error)
                if error.kind() == io::ErrorKind::ExecutableFileBusy
                    && attempt + 1 < MAX_ATTEMPTS =>
            {
                thread::sleep(Duration::from_millis(20));
            }
            result => return result,
        }
    }
    unreachable!("the final executable launch attempt always returns")
}

fn run_configured(fixture: &Fixture, args: &[&str]) -> Output {
    fixture_command(Path::new(env!("CARGO_BIN_EXE_qiongli")), fixture)
        .args(args)
        .output()
        .expect("configured native qiongli binary should start")
}

fn run_configured_os(
    executable: &Path,
    fixture: &Fixture,
    args: &[OsString],
    without_path: bool,
) -> Output {
    let mut command = fixture_command(executable, fixture);
    command.args(args);
    if without_path {
        command.env("PATH", "");
    }
    output_with_executable_busy_retry(&mut command)
        .expect("configured native qiongli binary should start")
}

fn parse_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("command stdout must be one JSON object")
}

fn public_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn text_contains_path(text: &str, path: &Path) -> bool {
    let display = path.to_string_lossy();
    let encoded = serde_json::to_string(display.as_ref())
        .expect("a filesystem path must have a JSON string representation");
    let escaped = encoded
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .expect("serialized JSON strings must be quoted");
    text.contains(display.as_ref()) || text.contains(escaped)
}

fn output_contains_path(output: &Output, path: &Path) -> bool {
    text_contains_path(&public_output(output), path)
}

fn run_project_os(fixture: &Fixture, args: Vec<OsString>) -> Output {
    run_configured_os(
        Path::new(env!("CARGO_BIN_EXE_qiongli")),
        fixture,
        &args,
        true,
    )
}

#[test]
fn version_uses_the_workspace_package_version() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("qiongli {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn copied_binary_recovers_delivery_mutations_across_process_restarts() {
    let fixture = Fixture::new("capture-delivery-restart");
    let config = resolve_config_root(Some(fixture.config_root.as_os_str()), &fixture.home).unwrap();
    let service = ProjectStateService::new(config.clone());
    let project_root = fixture.root.join("delivery-paper");
    let create = service
        .preview_create(
            &project_root,
            ProjectRegistrationOptions::new("Delivery Paper", ProjectKind::Article)
                .with_stage(ProjectStage::Writing),
            1_800_000_000,
        )
        .unwrap();
    let project_id = create.preview().project_id.clone();
    service
        .apply(
            &create,
            &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
            1_800_000_000,
        )
        .unwrap();

    let capture = ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            project_id.clone(),
            1,
            ProjectStage::Writing,
            "Retain the restart-safe delivery identity",
            CapturePolicy::ReviewRequired,
        )
        .unwrap(),
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Connected,
        captured_at_unix: 1_800_000_001,
        summary: "This private summary must not cross the delivery CLI boundary.".to_string(),
        changes: Vec::new(),
        decisions: Vec::new(),
        evidence: Vec::new(),
        contradictions: Vec::new(),
        next_actions: vec!["Resume delivery after the client restarts.".to_string()],
    }
    .into_capture()
    .unwrap();
    let envelope = CaptureDeliveryEnvelopeV1::new(
        capture,
        Some(CaptureDeliveryDestinationV1::new(project_id.clone(), 1).unwrap()),
        1_800_000_010,
    )
    .unwrap();
    let queued = service.enqueue_capture_delivery(envelope.clone()).unwrap();
    let delivering = service
        .begin_capture_delivery(
            &envelope.envelope_id,
            queued.generation,
            &queued.record_sha256,
            1_800_000_011,
        )
        .unwrap();

    let source_executable = PathBuf::from(env!("CARGO_BIN_EXE_qiongli"));
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-capture-delivery-runtime-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&runtime_root).expect("outside-checkout runtime root must be created");
    set_private_directory_mode(&runtime_root);
    let copied = runtime_root.join(
        source_executable
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source_executable, &copied)
        .expect("native executable must copy outside the checkout");

    let inspect = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "delivery".into(),
            "inspect".into(),
            "--envelope-id".into(),
            envelope.envelope_id.as_str().into(),
        ],
        true,
    );
    assert!(inspect.status.success(), "{}", public_output(&inspect));
    let inspect_json = parse_json(&inspect);
    assert_eq!(inspect_json["command"], "project-capture-delivery-inspect");
    assert_eq!(inspect_json["delivery"]["state"], "delivering");
    assert_eq!(
        inspect_json["delivery"]["destination"]["projectId"],
        project_id.as_str()
    );
    assert_eq!(
        inspect_json["delivery"]["destination"]["expectedProjectRevision"],
        1
    );
    for forbidden in [&project_root, &fixture.config_root, &runtime_root] {
        assert!(!output_contains_path(&inspect, forbidden));
    }
    assert!(!public_output(&inspect).contains("private summary"));

    let retry = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "delivery".into(),
            "retry".into(),
            "--envelope-id".into(),
            envelope.envelope_id.as_str().into(),
            "--expected-generation".into(),
            delivering.generation.to_string().into(),
            "--expected-record-sha256".into(),
            delivering.record_sha256.clone().into(),
            "--retried-at-unix".into(),
            "1800000012".into(),
            "--cause".into(),
            "transport-unavailable".into(),
        ],
        true,
    );
    assert!(retry.status.success(), "{}", public_output(&retry));
    let retry_json = parse_json(&retry);
    assert_eq!(retry_json["command"], "project-capture-delivery-retry");
    assert_eq!(retry_json["delivery"]["state"], "retry-required");
    assert_eq!(retry_json["delivery"]["retryCount"], 0);

    let list = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "delivery".into(),
            "list".into(),
        ],
        true,
    );
    assert!(list.status.success(), "{}", public_output(&list));
    let list_json = parse_json(&list);
    assert_eq!(list_json["command"], "project-capture-delivery-list");
    assert_eq!(list_json["deliveries"].as_array().unwrap().len(), 1);
    assert_eq!(list_json["deliveries"][0], retry_json["delivery"]);

    let retry_status = service
        .inspect_capture_delivery(&envelope.envelope_id)
        .unwrap()
        .unwrap();
    let restarted_service = ProjectStateService::new(config);
    let delivering_again = restarted_service
        .begin_capture_delivery(
            &envelope.envelope_id,
            retry_status.generation,
            &retry_status.record_sha256,
            1_800_000_013,
        )
        .unwrap();
    assert_eq!(delivering_again.retry_count, 1);
    let cancel_args = [
        "project".into(),
        "capture".into(),
        "delivery".into(),
        "cancel".into(),
        "--envelope-id".into(),
        envelope.envelope_id.as_str().into(),
        "--expected-generation".into(),
        delivering_again.generation.to_string().into(),
        "--expected-record-sha256".into(),
        delivering_again.record_sha256.clone().into(),
        "--cancelled-at-unix".into(),
        "1800000014".into(),
    ];
    let cancelled = run_configured_os(&copied, &fixture, &cancel_args, true);
    assert!(cancelled.status.success(), "{}", public_output(&cancelled));
    let cancelled_json = parse_json(&cancelled);
    assert_eq!(cancelled_json["command"], "project-capture-delivery-cancel");
    assert_eq!(cancelled_json["delivery"]["state"], "cancelled");

    let replay = run_configured_os(&copied, &fixture, &cancel_args, true);
    assert!(replay.status.success(), "{}", public_output(&replay));
    assert_eq!(parse_json(&replay)["delivery"], cancelled_json["delivery"]);
    assert_eq!(
        ProjectStateService::new(
            resolve_config_root(Some(fixture.config_root.as_os_str()), &fixture.home).unwrap(),
        )
        .inspect_capture_delivery(&envelope.envelope_id)
        .unwrap()
        .unwrap()
        .state,
        CaptureDeliveryState::Cancelled
    );
    for output in [&retry, &list, &cancelled, &replay] {
        assert!(!output_contains_path(output, &project_root));
        assert!(!output_contains_path(output, &fixture.config_root));
        assert!(!public_output(output).contains("private summary"));
    }

    fs::remove_dir_all(runtime_root).expect("outside-checkout runtime root must be removed");
}

#[test]
fn copied_binary_assigns_and_resolves_capture_lineage_across_restarts() {
    let fixture = Fixture::new("capture-assignment-resolution-restart");
    let config = resolve_config_root(Some(fixture.config_root.as_os_str()), &fixture.home).unwrap();
    let service = ProjectStateService::new(config);
    let project_root = fixture.root.join("resolution-paper");
    let create = service
        .preview_create(
            &project_root,
            ProjectRegistrationOptions::new("Resolution Paper", ProjectKind::Article)
                .with_stage(ProjectStage::Writing),
            1_800_010_000,
        )
        .unwrap();
    let project_id = create.preview().project_id.clone();
    service
        .apply(
            &create,
            &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
            1_800_010_000,
        )
        .unwrap();

    let source_executable = PathBuf::from(env!("CARGO_BIN_EXE_qiongli"));
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-capture-resolution-runtime-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&runtime_root).expect("outside-checkout runtime root must be created");
    set_private_directory_mode(&runtime_root);
    let copied = runtime_root.join(
        source_executable
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source_executable, &copied)
        .expect("native executable must copy outside the checkout");

    let first_capture = resolution_capture(&project_id, 1, 1_800_010_001, false);
    let first_envelope = CaptureDeliveryEnvelopeV1::new(
        first_capture,
        Some(CaptureDeliveryDestinationV1::new(project_id.clone(), 1).unwrap()),
        1_800_010_010,
    )
    .unwrap();
    service
        .enqueue_capture_delivery(first_envelope.clone())
        .unwrap();
    let first_assignment_preview = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "preview",
            &first_envelope.envelope_id,
            &project_id,
            "assign",
            1_800_010_020,
            None,
        ),
        true,
    );
    assert!(
        first_assignment_preview.status.success(),
        "{}",
        public_output(&first_assignment_preview)
    );
    let first_assignment_preview_json = parse_json(&first_assignment_preview);
    assert_eq!(
        first_assignment_preview_json["command"],
        "project-capture-assignment-preview"
    );
    assert_eq!(
        first_assignment_preview_json["preview"]["bindingEffect"],
        "direct"
    );
    assert_eq!(
        first_assignment_preview_json["preview"]["outcome"],
        "resolution-required"
    );
    let first_assignment_digest = first_assignment_preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let first_assignment = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "apply",
            &first_envelope.envelope_id,
            &project_id,
            "assign",
            1_800_010_020,
            Some(&first_assignment_digest),
        ),
        true,
    );
    assert!(
        first_assignment.status.success(),
        "{}",
        public_output(&first_assignment)
    );
    let first_assignment_json = parse_json(&first_assignment);
    let first_assignment_intent = first_assignment_json["commit"]["intentId"]
        .as_str()
        .unwrap()
        .to_string();
    let first_assignment_receipt = first_assignment_json["commit"]["receiptId"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(first_assignment_json["commit"]["outcome"], "assigned");

    let first_resolution_preview = run_configured_os(
        &copied,
        &fixture,
        &resolution_preview_args(&first_assignment_receipt, 1_800_010_030, &[]),
        true,
    );
    assert!(
        first_resolution_preview.status.success(),
        "{}",
        public_output(&first_resolution_preview)
    );
    let first_resolution_preview_json = parse_json(&first_resolution_preview);
    assert_eq!(
        first_resolution_preview_json["preview"]["items"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    let first_selections = resolution_selections(
        &first_resolution_preview_json,
        &[
            ("semantic-change", "accept-capture"),
            ("decision", "accept-capture"),
            ("evidence", "accept-capture"),
            ("contradiction", "accept-capture"),
            ("next-action", "accept-capture"),
        ],
    );
    let first_selection_preview = run_configured_os(
        &copied,
        &fixture,
        &resolution_preview_args(&first_assignment_receipt, 1_800_010_030, &first_selections),
        true,
    );
    assert!(
        first_selection_preview.status.success(),
        "{}",
        public_output(&first_selection_preview)
    );
    let first_selection_json = parse_json(&first_selection_preview);
    let first_plan_digest = first_selection_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let first_selection_digest = first_selection_json["selectionSet"]["selectionDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let first_resolution = run_configured_os(
        &copied,
        &fixture,
        &resolution_apply_args(
            &first_assignment_receipt,
            1_800_010_030,
            1_800_010_031,
            &first_selections,
            &first_plan_digest,
            &first_selection_digest,
        ),
        true,
    );
    assert!(
        first_resolution.status.success(),
        "{}",
        public_output(&first_resolution)
    );
    let first_resolution_json = parse_json(&first_resolution);
    let first_resolution_receipt = first_resolution_json["commit"]["receiptId"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(first_resolution_json["commit"]["toProjectRevision"], 2);
    assert_eq!(
        first_resolution_json["commit"]["childState"],
        "acknowledged"
    );

    let second_capture = resolution_capture(&project_id, 1, 1_800_010_040, true);
    let second_envelope =
        CaptureDeliveryEnvelopeV1::new(second_capture, None, 1_800_010_050).unwrap();
    service
        .enqueue_capture_delivery(second_envelope.clone())
        .unwrap();
    let second_assignment_preview = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "preview",
            &second_envelope.envelope_id,
            &project_id,
            "assign",
            1_800_010_060,
            None,
        ),
        true,
    );
    assert!(
        second_assignment_preview.status.success(),
        "{}",
        public_output(&second_assignment_preview)
    );
    let second_assignment_preview_json = parse_json(&second_assignment_preview);
    assert_eq!(
        second_assignment_preview_json["preview"]["bindingEffect"],
        "rebound"
    );
    assert_eq!(
        second_assignment_preview_json["preview"]["outcome"],
        "resolution-required"
    );
    let second_assignment_digest = second_assignment_preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let second_assignment = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "apply",
            &second_envelope.envelope_id,
            &project_id,
            "assign",
            1_800_010_060,
            Some(&second_assignment_digest),
        ),
        true,
    );
    assert!(
        second_assignment.status.success(),
        "{}",
        public_output(&second_assignment)
    );
    let second_assignment_receipt = parse_json(&second_assignment)["commit"]["receiptId"]
        .as_str()
        .unwrap()
        .to_string();

    let second_resolution_preview = run_configured_os(
        &copied,
        &fixture,
        &resolution_preview_args(&second_assignment_receipt, 1_800_010_070, &[]),
        true,
    );
    assert!(
        second_resolution_preview.status.success(),
        "{}",
        public_output(&second_resolution_preview)
    );
    let second_resolution_preview_json = parse_json(&second_resolution_preview);
    let second_selections = resolution_selections(
        &second_resolution_preview_json,
        &[
            ("semantic-change", "accept-current"),
            ("decision", "retain-both"),
            ("evidence", "accept-capture"),
            ("contradiction", "reject-capture"),
            ("next-action", "accept-current"),
        ],
    );
    let second_selection_preview = run_configured_os(
        &copied,
        &fixture,
        &resolution_preview_args(
            &second_assignment_receipt,
            1_800_010_070,
            &second_selections,
        ),
        true,
    );
    assert!(
        second_selection_preview.status.success(),
        "{}",
        public_output(&second_selection_preview)
    );
    let second_selection_json = parse_json(&second_selection_preview);
    let second_plan_digest = second_selection_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let second_selection_digest = second_selection_json["selectionSet"]["selectionDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let second_resolution_args = resolution_apply_args(
        &second_assignment_receipt,
        1_800_010_070,
        1_800_010_071,
        &second_selections,
        &second_plan_digest,
        &second_selection_digest,
    );
    let second_resolution = run_configured_os(&copied, &fixture, &second_resolution_args, true);
    assert!(
        second_resolution.status.success(),
        "{}",
        public_output(&second_resolution)
    );
    assert_eq!(
        parse_json(&second_resolution)["commit"]["toProjectRevision"],
        3
    );

    let exact_replay = run_configured_os(&copied, &fixture, &second_resolution_args, true);
    assert!(
        exact_replay.status.success(),
        "{}",
        public_output(&exact_replay)
    );
    assert_eq!(parse_json(&exact_replay)["commit"]["exactReplay"], true);

    let duplicate_capture = resolution_capture(&project_id, 3, 1_800_010_072, false);
    let duplicate_intake = service.preview_capture(duplicate_capture.clone()).unwrap();
    service
        .apply_capture(
            &duplicate_intake,
            &ApprovedCaptureIntake::new(duplicate_intake.preview().plan_digest.clone(), true),
            1_800_010_073,
        )
        .unwrap();
    let duplicate_envelope =
        CaptureDeliveryEnvelopeV1::new(duplicate_capture, None, 1_800_010_074).unwrap();
    service
        .enqueue_capture_delivery(duplicate_envelope.clone())
        .unwrap();
    let duplicate_preview = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "preview",
            &duplicate_envelope.envelope_id,
            &project_id,
            "assign",
            1_800_010_075,
            None,
        ),
        true,
    );
    assert!(
        duplicate_preview.status.success(),
        "{}",
        public_output(&duplicate_preview)
    );
    assert_eq!(
        parse_json(&duplicate_preview)["preview"]["outcome"],
        "duplicate"
    );

    let assignment_inspect = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "assignment".into(),
            "inspect".into(),
            "--intent-id".into(),
            first_assignment_intent.into(),
        ],
        true,
    );
    assert!(
        assignment_inspect.status.success(),
        "{}",
        public_output(&assignment_inspect)
    );
    assert_eq!(
        parse_json(&assignment_inspect)["assignment"]["receiptId"],
        first_assignment_receipt
    );
    let resolution_inspect = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "resolution".into(),
            "inspect".into(),
            "--project-id".into(),
            project_id.as_str().into(),
            "--receipt-id".into(),
            first_resolution_receipt.into(),
        ],
        true,
    );
    assert!(
        resolution_inspect.status.success(),
        "{}",
        public_output(&resolution_inspect)
    );
    assert_eq!(
        parse_json(&resolution_inspect)["resolution"]["receipt"]["targetProjectId"],
        project_id.as_str()
    );
    let resolution_list = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "resolution".into(),
            "list".into(),
            "--project-id".into(),
            project_id.as_str().into(),
        ],
        true,
    );
    assert!(
        resolution_list.status.success(),
        "{}",
        public_output(&resolution_list)
    );
    assert_eq!(
        parse_json(&resolution_list)["resolutions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let rejected_capture = resolution_capture(&project_id, 3, 1_800_010_080, false);
    let rejected_envelope =
        CaptureDeliveryEnvelopeV1::new(rejected_capture, None, 1_800_010_081).unwrap();
    service
        .enqueue_capture_delivery(rejected_envelope.clone())
        .unwrap();
    let reject_preview = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "preview",
            &rejected_envelope.envelope_id,
            &project_id,
            "reject",
            1_800_010_082,
            None,
        ),
        true,
    );
    assert!(
        reject_preview.status.success(),
        "{}",
        public_output(&reject_preview)
    );
    let reject_digest = parse_json(&reject_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let rejected = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "apply",
            &rejected_envelope.envelope_id,
            &project_id,
            "reject",
            1_800_010_082,
            Some(&reject_digest),
        ),
        true,
    );
    assert!(rejected.status.success(), "{}", public_output(&rejected));
    assert_eq!(parse_json(&rejected)["commit"]["outcome"], "rejected");

    let stale_capture = resolution_capture(&project_id, 3, 1_800_010_090, false);
    let stale_envelope =
        CaptureDeliveryEnvelopeV1::new(stale_capture, None, 1_800_010_091).unwrap();
    service
        .enqueue_capture_delivery(stale_envelope.clone())
        .unwrap();
    let stale_preview = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "preview",
            &stale_envelope.envelope_id,
            &project_id,
            "assign",
            1_800_010_092,
            None,
        ),
        true,
    );
    assert!(
        stale_preview.status.success(),
        "{}",
        public_output(&stale_preview)
    );
    let stale_digest = parse_json(&stale_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    fs::write(
        project_root.join("context/stage_handoff.md"),
        "The target changed after assignment preview.\n",
    )
    .unwrap();
    let refresh = service.preview_refresh(&project_id, 1_800_010_093).unwrap();
    service
        .apply(
            &refresh,
            &ApprovedProjectMutation::new(refresh.preview().plan_digest.clone(), true),
            1_800_010_093,
        )
        .unwrap();
    let stale_apply = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "apply",
            &stale_envelope.envelope_id,
            &project_id,
            "assign",
            1_800_010_092,
            Some(&stale_digest),
        ),
        true,
    );
    assert_eq!(stale_apply.status.code(), Some(1));
    assert_eq!(stale_apply.stderr, b"error: project-plan-mismatch\n");

    let archive = service.preview_archive(&project_id).unwrap();
    service
        .apply(
            &archive,
            &ApprovedProjectMutation::new(archive.preview().plan_digest.clone(), true),
            1_800_010_094,
        )
        .unwrap();
    let archived_target = run_configured_os(
        &copied,
        &fixture,
        &assignment_args(
            "preview",
            &stale_envelope.envelope_id,
            &project_id,
            "assign",
            1_800_010_095,
            None,
        ),
        true,
    );
    assert_eq!(archived_target.status.code(), Some(1));
    assert_eq!(
        archived_target.stderr,
        b"error: project-revision-conflict\n"
    );

    let resolution_lock = fixture
        .state_root()
        .join("capture-resolution/v1/.ledger.lock");
    let resolution_lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&resolution_lock)
        .unwrap();
    resolution_lock_file.lock().unwrap();
    let lock_busy = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "assignment".into(),
            "list".into(),
        ],
        true,
    );
    assert_eq!(lock_busy.status.code(), Some(1));
    assert_eq!(lock_busy.stderr, b"error: project-library-lock-busy\n");
    resolution_lock_file.unlock().unwrap();
    drop(resolution_lock_file);

    let corrupt_record = fixture
        .state_root()
        .join("capture-delivery/v1/records")
        .join(format!("{}.json", stale_envelope.envelope_id.as_str()));
    fs::write(&corrupt_record, b"{\"schema_version\":1}").unwrap();
    let corrupt = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "delivery".into(),
            "inspect".into(),
            "--envelope-id".into(),
            stale_envelope.envelope_id.as_str().into(),
        ],
        true,
    );
    assert_eq!(corrupt.status.code(), Some(1));
    assert_eq!(
        corrupt.stderr,
        b"error: capture-delivery-document-invalid\n"
    );

    let assignment_list = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "assignment".into(),
            "list".into(),
        ],
        true,
    );
    assert_eq!(assignment_list.status.code(), Some(1));
    assert_eq!(
        assignment_list.stderr,
        b"error: capture-delivery-document-invalid\n"
    );
    for output in [
        &first_assignment_preview,
        &first_assignment,
        &first_resolution_preview,
        &first_resolution,
        &second_assignment_preview,
        &second_assignment,
        &second_resolution_preview,
        &second_resolution,
        &exact_replay,
        &duplicate_preview,
        &assignment_inspect,
        &resolution_inspect,
        &resolution_list,
        &rejected,
        &stale_apply,
        &archived_target,
        &lock_busy,
        &corrupt,
    ] {
        assert!(!output_contains_path(output, &project_root));
        assert!(!output_contains_path(output, &fixture.config_root));
        assert!(!output_contains_path(output, &runtime_root));
    }

    fs::remove_dir_all(runtime_root).expect("outside-checkout runtime root must be removed");
}

fn resolution_capture(
    project_id: &ProjectId,
    base_revision: u64,
    captured_at_unix: u64,
    divergent: bool,
) -> ResearchCaptureV1 {
    ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            project_id.clone(),
            base_revision,
            ProjectStage::Writing,
            "Reconcile one bounded academic capture",
            CapturePolicy::ReviewRequired,
        )
        .unwrap(),
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Connected,
        captured_at_unix,
        summary: if divergent {
            "Review divergent academic content after a stale client resumes."
        } else {
            "Review the initial academic content from a connected client."
        }
        .to_string(),
        changes: vec![SemanticChangeV1 {
            area: CaptureArea::Thesis,
            summary: if divergent {
                "The revised thesis preserves exact lineage across restarts."
            } else {
                "The thesis preserves exact lineage across restarts."
            }
            .to_string(),
        }],
        decisions: vec![DecisionCandidateV1 {
            relation: DecisionRelation::Refinement,
            statement: if divergent {
                "Use a revised content-addressed resolution protocol."
            } else {
                "Use a content-addressed resolution protocol."
            }
            .to_string(),
            rationale: if divergent {
                "The stale client requires an explicit coexistence decision."
            } else {
                "The protocol survives process restarts."
            }
            .to_string(),
            target: Some("decision:resolution-protocol".to_string()),
        }],
        evidence: vec![EvidenceReferenceV1 {
            locator_kind: EvidenceLocatorKind::Doi,
            locator: "10.1000/qiongli-resolution".to_string(),
            relevance: if divergent {
                "Supports the revised restart qualification."
            } else {
                "Supports the initial restart qualification."
            }
            .to_string(),
            limitation: divergent.then(|| "Requires explicit review.".to_string()),
        }],
        contradictions: vec![ContradictionV1 {
            statement: "Automatic overwrite is unsafe.".to_string(),
            conflicts_with: "Unreviewed stale client state.".to_string(),
            consequence: if divergent {
                "Reject the revised capture item explicitly."
            } else {
                "Record the initial contradiction."
            }
            .to_string(),
        }],
        next_actions: vec!["Inspect the durable resolution receipt.".to_string()],
    }
    .into_capture()
    .unwrap()
}

fn assignment_args(
    command: &str,
    envelope_id: &qiongli_project::DeliveryEnvelopeId,
    project_id: &ProjectId,
    decision: &str,
    decided_at_unix: u64,
    expected_plan_digest: Option<&str>,
) -> Vec<OsString> {
    let mut args = vec![
        "project".into(),
        "capture".into(),
        "assignment".into(),
        command.into(),
        "--source-envelope-id".into(),
        envelope_id.as_str().into(),
        "--target-project-id".into(),
        project_id.as_str().into(),
        "--decision".into(),
        decision.into(),
        "--decided-at-unix".into(),
        decided_at_unix.to_string().into(),
    ];
    if let Some(digest) = expected_plan_digest {
        args.extend([
            "--expected-plan-digest".into(),
            digest.into(),
            "--approve-assignment-write".into(),
        ]);
    }
    args
}

fn resolution_preview_args(
    assignment_receipt_id: &str,
    reviewed_at_unix: u64,
    selections: &[String],
) -> Vec<OsString> {
    let mut args = vec![
        "project".into(),
        "capture".into(),
        "resolution".into(),
        "preview".into(),
        "--assignment-receipt-id".into(),
        assignment_receipt_id.into(),
        "--reviewed-at-unix".into(),
        reviewed_at_unix.to_string().into(),
    ];
    for selection in selections {
        args.extend(["--select".into(), selection.into()]);
    }
    args
}

fn resolution_apply_args(
    assignment_receipt_id: &str,
    reviewed_at_unix: u64,
    resolved_at_unix: u64,
    selections: &[String],
    expected_plan_digest: &str,
    expected_selection_digest: &str,
) -> Vec<OsString> {
    let mut args = vec![
        "project".into(),
        "capture".into(),
        "resolution".into(),
        "apply".into(),
        "--assignment-receipt-id".into(),
        assignment_receipt_id.into(),
        "--reviewed-at-unix".into(),
        reviewed_at_unix.to_string().into(),
        "--resolved-at-unix".into(),
        resolved_at_unix.to_string().into(),
    ];
    for selection in selections {
        args.extend(["--select".into(), selection.into()]);
    }
    args.extend([
        "--expected-plan-digest".into(),
        expected_plan_digest.into(),
        "--expected-selection-digest".into(),
        expected_selection_digest.into(),
        "--approve-academic-review".into(),
        "--approve-filesystem-write".into(),
    ]);
    args
}

fn resolution_selections(preview: &Value, dispositions: &[(&str, &str)]) -> Vec<String> {
    let items = preview["preview"]["items"].as_array().unwrap();
    assert_eq!(items.len(), dispositions.len());
    items
        .iter()
        .zip(dispositions)
        .map(|(item, (expected_kind, disposition))| {
            assert_eq!(item["item"]["kind"], *expected_kind);
            format!("{}={disposition}", item["item"]["itemId"].as_str().unwrap())
        })
        .collect()
}

#[test]
fn project_graph_cli_rebuilds_and_queries_without_writing_index_state() {
    let fixture = Fixture::new("project-graph-query");
    let project_root = fixture.root.join("graph-paper");
    let config = resolve_config_root(Some(fixture.config_root.as_os_str()), &fixture.home).unwrap();
    let projects = ProjectStateService::new(config);
    let plan = projects
        .preview_create(
            &project_root,
            ProjectRegistrationOptions::new("Graph Paper", ProjectKind::Article),
            1,
        )
        .unwrap();
    let project_id = plan.preview().project_id.clone();
    projects
        .apply(
            &plan,
            &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
            1,
        )
        .unwrap();
    fs::write(
        project_root.join("context/research_state.md"),
        "- main_question_or_thesis: Which exposure changes returns?\n",
    )
    .unwrap();
    let refresh = projects.preview_refresh(&project_id, 2).unwrap();
    projects
        .apply(
            &refresh,
            &ApprovedProjectMutation::new(refresh.preview().plan_digest.clone(), true),
            2,
        )
        .unwrap();

    let snapshot = run_configured(
        &fixture,
        &[
            "project",
            "graph",
            "snapshot",
            "--project-id",
            project_id.as_str(),
        ],
    );
    assert!(snapshot.status.success(), "{}", public_output(&snapshot));
    assert!(!output_contains_path(&snapshot, &project_root));
    let snapshot_json = parse_json(&snapshot);
    let projection_id = snapshot_json["snapshot"]["projectionId"].as_str().unwrap();
    let project_revision = snapshot_json["snapshot"]["projectRevision"]
        .as_u64()
        .unwrap();
    assert_eq!(
        snapshot_json["readiness"]["projectRevision"],
        project_revision
    );
    assert_eq!(snapshot_json["readiness"]["staleSourceCount"], 0);
    let project_revision_text = project_revision.to_string();
    let node_id = snapshot_json["snapshot"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| {
            node["artifactPath"] == "context/research_state.md"
                && node["nodeType"] == "research-question"
        })
        .and_then(|node| node["nodeId"].as_str())
        .unwrap();
    assert_eq!(snapshot_json["command"], "project-graph-snapshot");

    let artifact = run_configured(
        &fixture,
        &[
            "app",
            "read-project-artifact",
            "--project-id",
            project_id.as_str(),
            "--expected-project-revision",
            &project_revision_text,
            "--expected-projection-id",
            projection_id,
            "--node-id",
            node_id,
        ],
    );
    assert!(artifact.status.success(), "{}", public_output(&artifact));
    assert!(!output_contains_path(&artifact, &project_root));
    assert!(!output_contains_path(&artifact, &fixture.config_root));
    let artifact_json = parse_json(&artifact);
    assert_eq!(artifact_json["type"], "project-artifact-read");
    assert_eq!(artifact_json["artifact"]["projectId"], project_id.as_str());
    assert_eq!(
        artifact_json["artifact"]["projectRevision"],
        project_revision
    );
    assert_eq!(artifact_json["artifact"]["projectionId"], projection_id);
    assert_eq!(artifact_json["artifact"]["entityKind"], "node");
    assert_eq!(artifact_json["artifact"]["entityId"], node_id);
    assert_eq!(
        artifact_json["artifact"]["artifactPath"],
        "context/research_state.md"
    );
    assert_eq!(artifact_json["artifact"]["format"], "markdown");
    assert_eq!(
        artifact_json["artifact"]["sourceAnchor"],
        "field:main_question_or_thesis"
    );
    assert_eq!(artifact_json["artifact"]["anchorLine"], 1);
    assert_eq!(artifact_json["artifact"]["anchorMatched"], true);
    assert!(
        artifact_json["artifact"]["content"]
            .as_str()
            .unwrap()
            .contains("Which exposure changes returns?")
    );

    let stale_revision_text = (project_revision - 1).to_string();
    let stale_artifact = run_configured(
        &fixture,
        &[
            "app",
            "read-project-artifact",
            "--project-id",
            project_id.as_str(),
            "--expected-project-revision",
            &stale_revision_text,
            "--expected-projection-id",
            projection_id,
            "--node-id",
            node_id,
        ],
    );
    assert!(!stale_artifact.status.success());
    assert_eq!(
        String::from_utf8_lossy(&stale_artifact.stderr),
        "error: project-revision-conflict\n"
    );

    let portfolio = run_configured(&fixture, &["project", "graph", "portfolio"]);
    assert!(portfolio.status.success(), "{}", public_output(&portfolio));
    assert!(!output_contains_path(&portfolio, &project_root));
    let portfolio_json = parse_json(&portfolio);
    assert_eq!(portfolio_json["command"], "project-graph-portfolio");
    assert_eq!(portfolio_json["portfolio"]["projectCount"], 1);
    assert_eq!(portfolio_json["portfolio"]["includedProjectCount"], 1);
    assert_eq!(
        portfolio_json["portfolio"]["projects"][0]["projectId"],
        project_id.as_str()
    );
    assert!(
        portfolio_json["portfolio"]["portfolioId"]
            .as_str()
            .is_some_and(|value| value.starts_with("gpf_"))
    );

    let doctor = run_configured(
        &fixture,
        &[
            "project",
            "graph",
            "doctor",
            "--project-id",
            project_id.as_str(),
        ],
    );
    assert!(doctor.status.success(), "{}", public_output(&doctor));
    assert!(!output_contains_path(&doctor, &project_root));
    let doctor_json = parse_json(&doctor);
    assert_eq!(doctor_json["command"], "project-graph-doctor");
    assert_eq!(doctor_json["projectId"], project_id.as_str());
    assert_eq!(doctor_json["projectionId"], projection_id);
    assert_eq!(doctor_json["deterministicRebuild"], true);
    assert_eq!(doctor_json["persistentIndexState"], "none");
    assert_eq!(doctor_json["portableAuthority"], false);
    assert_eq!(doctor_json["readiness"]["staleSourceCount"], 0);

    let query = run_configured(
        &fixture,
        &[
            "project",
            "graph",
            "query",
            "--project-id",
            project_id.as_str(),
            "--expected-projection-id",
            projection_id,
            "--node-type",
            "research-question",
            "--canonical-id",
            "research-question:current",
        ],
    );
    assert!(query.status.success(), "{}", public_output(&query));
    assert!(!output_contains_path(&query, &project_root));
    let query_json = parse_json(&query);
    assert_eq!(query_json["command"], "project-graph-query");
    assert_eq!(query_json["readiness"]["projectRevision"], project_revision);
    assert_eq!(query_json["result"]["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(
        query_json["result"]["nodes"][0]["canonicalId"],
        "research-question:current"
    );
    assert!(!project_root.join(".qiongli/graph-index").exists());

    let stale = run_configured(
        &fixture,
        &[
            "project",
            "graph",
            "query",
            "--project-id",
            project_id.as_str(),
            "--expected-projection-id",
            "grp_ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ],
    );
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("project-revision-conflict"));
}

#[test]
fn copied_binary_reconciles_queries_deletes_and_recovers_portfolio_without_path() {
    let fixture = Fixture::new("portfolio-restart");
    let config = resolve_config_root(Some(fixture.config_root.as_os_str()), &fixture.home).unwrap();
    let projects = ProjectStateService::new(config);
    let mut project_ids = Vec::new();
    let mut project_roots = Vec::new();
    for (name, timestamp) in [("Portfolio Restart A", 10), ("Portfolio Restart B", 20)] {
        let project_root = fixture.root.join(name.to_lowercase().replace(' ', "-"));
        let plan = projects
            .preview_create(
                &project_root,
                ProjectRegistrationOptions::new(name, ProjectKind::Article),
                timestamp,
            )
            .unwrap();
        project_ids.push(plan.preview().project_id.clone());
        project_roots.push(project_root);
        projects
            .apply(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                timestamp,
            )
            .unwrap();
    }

    let source_executable = PathBuf::from(env!("CARGO_BIN_EXE_qiongli"));
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-portfolio-runtime-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&runtime_root).expect("outside-checkout runtime root must be created");
    set_private_directory_mode(&runtime_root);
    let copied = runtime_root.join(
        source_executable
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source_executable, &copied)
        .expect("native executable must copy outside the checkout");
    let run_copied = |args: Vec<OsString>| run_configured_os(&copied, &fixture, &args, true);

    let empty_status = run_copied(vec!["project".into(), "portfolio".into(), "status".into()]);
    assert!(
        empty_status.status.success(),
        "{}",
        public_output(&empty_status)
    );
    assert!(parse_json(&empty_status)["catalog"].is_null());

    let preview = run_copied(vec![
        "project".into(),
        "portfolio".into(),
        "reconcile".into(),
        "preview".into(),
    ]);
    assert!(preview.status.success(), "{}", public_output(&preview));
    let preview_json = parse_json(&preview);
    let digest = preview_json["preview"]["planDigest"].as_str().unwrap();
    let applied = run_copied(vec![
        "project".into(),
        "portfolio".into(),
        "reconcile".into(),
        "apply".into(),
        "--expected-plan-digest".into(),
        digest.into(),
        "--approve-derived-state-write".into(),
    ]);
    assert!(applied.status.success(), "{}", public_output(&applied));
    let applied_json = parse_json(&applied);
    assert_eq!(applied_json["command"], "project-portfolio-reconcile-apply");
    assert_eq!(
        applied_json["reconciliation"]["rebuiltProjectCount"],
        project_ids.len()
    );
    let catalog_id = applied_json["reconciliation"]["snapshot"]["catalog"]["catalogId"]
        .as_str()
        .unwrap()
        .to_string();
    for forbidden in project_roots
        .iter()
        .chain([&fixture.config_root, &runtime_root])
    {
        assert!(!output_contains_path(&applied, forbidden));
    }

    let portfolio_query = PortfolioQueryV1::new(catalog_id.clone())
        .unwrap()
        .with_filters(PortfolioQueryFiltersV1 {
            project_id: Some(project_ids[0].clone()),
            ..PortfolioQueryFiltersV1::default()
        })
        .unwrap();
    let portfolio_query_json =
        String::from_utf8(portfolio_query.to_canonical_json().unwrap()).unwrap();
    let query = run_copied(vec![
        "project".into(),
        "portfolio".into(),
        "query".into(),
        "--request-json".into(),
        portfolio_query_json.into(),
    ]);
    assert!(query.status.success(), "{}", public_output(&query));
    let query_json = parse_json(&query);
    assert_eq!(query_json["command"], "project-portfolio-query");
    assert_eq!(
        query_json["result"]["projects"][0]["projectId"],
        project_ids[0].as_str()
    );

    let timeline_query = SemanticTimelineQueryV1::new(catalog_id)
        .unwrap()
        .for_project(project_ids[1].clone())
        .unwrap();
    let timeline_query_json =
        String::from_utf8(timeline_query.to_canonical_json().unwrap()).unwrap();
    let timeline = run_copied(vec![
        "project".into(),
        "portfolio".into(),
        "timeline".into(),
        "--request-json".into(),
        timeline_query_json.into(),
    ]);
    assert!(timeline.status.success(), "{}", public_output(&timeline));
    let timeline_json = parse_json(&timeline);
    assert_eq!(timeline_json["command"], "project-portfolio-timeline");
    assert!(
        timeline_json["result"]["events"]
            .as_array()
            .is_some_and(|events| !events.is_empty())
    );

    let doctor = run_copied(vec!["project".into(), "portfolio".into(), "doctor".into()]);
    assert!(doctor.status.success(), "{}", public_output(&doctor));
    assert_eq!(parse_json(&doctor)["doctor"]["status"], "equivalent");

    let delete_preview = run_copied(vec![
        "project".into(),
        "portfolio".into(),
        "delete-derived-state".into(),
        "preview".into(),
    ]);
    assert!(
        delete_preview.status.success(),
        "{}",
        public_output(&delete_preview)
    );
    let delete_digest = parse_json(&delete_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let deletion = run_copied(vec![
        "project".into(),
        "portfolio".into(),
        "delete-derived-state".into(),
        "apply".into(),
        "--expected-plan-digest".into(),
        delete_digest.into(),
        "--approve-derived-state-write".into(),
    ]);
    assert!(deletion.status.success(), "{}", public_output(&deletion));
    assert_eq!(
        parse_json(&deletion)["deletion"]["removedContributionCount"],
        project_ids.len()
    );

    let rebuild_preview = run_copied(vec![
        "project".into(),
        "portfolio".into(),
        "rebuild".into(),
        "preview".into(),
    ]);
    assert!(
        rebuild_preview.status.success(),
        "{}",
        public_output(&rebuild_preview)
    );
    let rebuild_digest = parse_json(&rebuild_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let rebuilt = run_copied(vec![
        "project".into(),
        "portfolio".into(),
        "rebuild".into(),
        "apply".into(),
        "--expected-plan-digest".into(),
        rebuild_digest.into(),
        "--approve-derived-state-write".into(),
    ]);
    assert!(rebuilt.status.success(), "{}", public_output(&rebuilt));
    assert_eq!(
        parse_json(&rebuilt)["reconciliation"]["rebuiltProjectCount"],
        project_ids.len()
    );

    fs::write(
        fixture
            .state_root()
            .join("portfolio-catalog/v1/catalog.json"),
        b"{}",
    )
    .expect("catalog can be corrupted for the fail-closed fixture");
    let corrupted = run_copied(vec!["project".into(), "portfolio".into(), "status".into()]);
    assert!(!corrupted.status.success());
    assert!(
        String::from_utf8_lossy(&corrupted.stderr).contains("portfolio-catalog-document-invalid")
    );
    let _ = fs::remove_dir_all(runtime_root);
}

#[test]
fn project_cli_creates_refreshes_and_unregisters_without_leaking_roots() {
    let fixture = Fixture::new("project-library");
    let project_root = fixture.root.join("paper-one");

    let empty = run_configured(&fixture, &["project", "list"]);
    assert!(empty.status.success(), "{}", public_output(&empty));
    assert_eq!(parse_json(&empty)["library"]["revision"], 0);
    assert!(!fixture.config_root.exists());

    let preview = run_project_os(
        &fixture,
        vec![
            "project".into(),
            "create".into(),
            "preview".into(),
            "--root".into(),
            project_root.as_os_str().to_owned(),
            "--name".into(),
            "First Article".into(),
        ],
    );
    assert!(preview.status.success(), "{}", public_output(&preview));
    assert!(!output_contains_path(&preview, &project_root));
    let preview_json = parse_json(&preview);
    let project_id = preview_json["preview"]["projectId"]
        .as_str()
        .unwrap()
        .to_string();
    let plan_digest = preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(preview_json["preview"]["rootLabel"], "paper-one");
    assert!(!project_root.exists());

    let applied = run_project_os(
        &fixture,
        vec![
            "project".into(),
            "create".into(),
            "apply".into(),
            "--root".into(),
            project_root.as_os_str().to_owned(),
            "--name".into(),
            "First Article".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--expected-plan-digest".into(),
            plan_digest.into(),
            "--approve-filesystem-write".into(),
        ],
    );
    assert!(applied.status.success(), "{}", public_output(&applied));
    assert_eq!(parse_json(&applied)["command"], "project-create-apply");
    assert!(project_root.join("context/project_manifest.json").is_file());

    let listed = run_configured(&fixture, &["project", "list"]);
    assert!(listed.status.success(), "{}", public_output(&listed));
    assert!(!output_contains_path(&listed, &project_root));
    let listed_json = parse_json(&listed);
    assert_eq!(
        listed_json["library"]["projects"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        listed_json["library"]["projects"][0]["displayName"],
        "First Article"
    );

    fs::write(
        project_root.join("context/research_state.md"),
        "RQ: How does durable project memory affect research continuity?\nThesis: Portable state outlives sessions.\nNext: Validate the library contract.\n",
    )
    .unwrap();
    let refresh_preview = run_configured(
        &fixture,
        &["project", "refresh", "preview", "--project-id", &project_id],
    );
    assert!(
        refresh_preview.status.success(),
        "{}",
        public_output(&refresh_preview)
    );
    let refresh_json = parse_json(&refresh_preview);
    assert_eq!(
        refresh_json["preview"]["effect"],
        "update-semantic-revision"
    );
    let refresh_digest = refresh_json["preview"]["planDigest"].as_str().unwrap();
    let refresh_apply = run_configured(
        &fixture,
        &[
            "project",
            "refresh",
            "apply",
            "--project-id",
            &project_id,
            "--expected-plan-digest",
            refresh_digest,
            "--approve-filesystem-write",
        ],
    );
    assert!(
        refresh_apply.status.success(),
        "{}",
        public_output(&refresh_apply)
    );
    let shown = run_configured(&fixture, &["project", "show", "--project-id", &project_id]);
    let shown_json = parse_json(&shown);
    assert_eq!(shown_json["project"]["semanticRevision"], 2);
    assert_eq!(
        shown_json["project"]["overview"]["thesis"],
        "Portable state outlives sessions."
    );

    let unregister_preview = run_configured(
        &fixture,
        &[
            "project",
            "unregister",
            "preview",
            "--project-id",
            &project_id,
        ],
    );
    let unregister_digest = parse_json(&unregister_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let unregister_apply = run_configured(
        &fixture,
        &[
            "project",
            "unregister",
            "apply",
            "--project-id",
            &project_id,
            "--expected-plan-digest",
            &unregister_digest,
            "--approve-filesystem-write",
        ],
    );
    assert!(
        unregister_apply.status.success(),
        "{}",
        public_output(&unregister_apply)
    );
    assert!(project_root.join("context/project_manifest.json").is_file());
    assert!(
        parse_json(&run_configured(&fixture, &["project", "list"]))["library"]["projects"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn project_cli_migrates_a_legacy_project_without_mutating_the_source() {
    const MANIFEST_CREATED_AT_UNIX: u64 = 1_721_337_601;
    let fixture = Fixture::new("project-migration");
    let source = fixture.root.join("legacy-paper");
    let destination = fixture.root.join("migrated-paper");
    fs::create_dir(&source).unwrap();
    fs::create_dir(source.join("context")).unwrap();
    let research_state = b"RQ: Can the CLI preserve legacy work?\n";
    fs::write(source.join("context/research_state.md"), research_state).unwrap();
    fs::create_dir(source.join(".qiongli")).unwrap();
    fs::write(
        source.join(".qiongli/guidance_manifest.yaml"),
        b"active_subject: management\n",
    )
    .unwrap();

    let preview = run_project_os(
        &fixture,
        vec![
            "project".into(),
            "migrate".into(),
            "preview".into(),
            "--source".into(),
            source.as_os_str().to_owned(),
            "--root".into(),
            destination.as_os_str().to_owned(),
            "--name".into(),
            "Legacy Article".into(),
            "--kind".into(),
            "review".into(),
            "--stage".into(),
            "writing".into(),
            "--manifest-created-at-unix".into(),
            MANIFEST_CREATED_AT_UNIX.to_string().into(),
        ],
    );
    assert!(preview.status.success(), "{}", public_output(&preview));
    assert!(!output_contains_path(&preview, &source));
    assert!(!output_contains_path(&preview, &destination));
    let preview_json = parse_json(&preview);
    assert_eq!(preview_json["command"], "project-migrate-preview");
    assert_eq!(preview_json["preview"]["sourceRetained"], true);
    assert_eq!(preview_json["preview"]["excludedEntryCount"], 1);
    assert_eq!(
        preview_json["preview"]["manifestCreatedAtUnix"],
        MANIFEST_CREATED_AT_UNIX
    );
    let project_id = preview_json["preview"]["projectId"]
        .as_str()
        .unwrap()
        .to_string();
    let plan_digest = preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();

    let mismatched_timestamp = run_project_os(
        &fixture,
        vec![
            "project".into(),
            "migrate".into(),
            "apply".into(),
            "--source".into(),
            source.as_os_str().to_owned(),
            "--root".into(),
            destination.as_os_str().to_owned(),
            "--name".into(),
            "Legacy Article".into(),
            "--kind".into(),
            "review".into(),
            "--stage".into(),
            "writing".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--manifest-created-at-unix".into(),
            (MANIFEST_CREATED_AT_UNIX + 1).to_string().into(),
            "--expected-plan-digest".into(),
            plan_digest.clone().into(),
            "--approve-filesystem-write".into(),
        ],
    );
    assert_eq!(
        mismatched_timestamp.stderr,
        b"error: project-plan-mismatch\n"
    );
    assert!(!destination.exists());

    let applied = run_project_os(
        &fixture,
        vec![
            "project".into(),
            "migrate".into(),
            "apply".into(),
            "--source".into(),
            source.as_os_str().to_owned(),
            "--root".into(),
            destination.as_os_str().to_owned(),
            "--name".into(),
            "Legacy Article".into(),
            "--kind".into(),
            "review".into(),
            "--stage".into(),
            "writing".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--manifest-created-at-unix".into(),
            MANIFEST_CREATED_AT_UNIX.to_string().into(),
            "--expected-plan-digest".into(),
            plan_digest.into(),
            "--approve-filesystem-write".into(),
        ],
    );
    assert!(applied.status.success(), "{}", public_output(&applied));
    assert!(!output_contains_path(&applied, &source));
    assert!(!output_contains_path(&applied, &destination));
    assert_eq!(parse_json(&applied)["command"], "project-migrate-apply");

    assert_eq!(
        fs::read(source.join("context/research_state.md")).unwrap(),
        research_state
    );
    assert!(!source.join("context/project_manifest.json").exists());
    assert!(source.join(".qiongli/guidance_manifest.yaml").is_file());
    assert_eq!(
        fs::read(destination.join("context/research_state.md")).unwrap(),
        research_state
    );
    assert!(destination.join("context/project_manifest.json").is_file());
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(destination.join("context/project_manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["created_at_unix"], MANIFEST_CREATED_AT_UNIX);
    assert!(
        destination
            .join(".qiongli/v2/project-migration.json")
            .is_file()
    );
    assert!(!destination.join(".qiongli/guidance_manifest.yaml").exists());
    let listed = parse_json(&run_configured(&fixture, &["project", "list"]));
    assert_eq!(listed["library"]["projects"][0]["projectId"], project_id);
    assert!(
        listed["library"]["projects"][0]["registeredAtUnix"]
            .as_u64()
            .unwrap()
            > MANIFEST_CREATED_AT_UNIX
    );
}

#[test]
fn copied_binary_accepts_repository_capture_without_runtime() {
    let fixture = Fixture::new("tier1-repository-capture");
    let source_executable = PathBuf::from(env!("CARGO_BIN_EXE_qiongli"));
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-tier1-repository-capture-runtime-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&runtime_root).expect("outside-checkout runtime root must be created");
    set_private_directory_mode(&runtime_root);
    let copied = runtime_root.join(
        source_executable
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source_executable, &copied)
        .expect("native executable must copy outside the checkout");

    let project_root = fixture.root.join("repository-capture-paper");
    let create_preview = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "create".into(),
            "preview".into(),
            "--root".into(),
            project_root.as_os_str().to_owned(),
            "--name".into(),
            "Repository Capture Paper".into(),
            "--kind".into(),
            "article".into(),
            "--stage".into(),
            "writing".into(),
        ],
        true,
    );
    assert!(
        create_preview.status.success(),
        "{}",
        public_output(&create_preview)
    );
    let create_preview_json = parse_json(&create_preview);
    let project_id = create_preview_json["preview"]["projectId"]
        .as_str()
        .unwrap()
        .to_string();
    let create_digest = create_preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let create_apply = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "create".into(),
            "apply".into(),
            "--root".into(),
            project_root.as_os_str().to_owned(),
            "--name".into(),
            "Repository Capture Paper".into(),
            "--kind".into(),
            "article".into(),
            "--stage".into(),
            "writing".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--expected-plan-digest".into(),
            create_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        create_apply.status.success(),
        "{}",
        public_output(&create_apply)
    );

    let capture = ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            ProjectId::parse(project_id.clone()).unwrap(),
            1,
            ProjectStage::Writing,
            "Retain the repository-backed article argument",
            CapturePolicy::ReviewRequired,
        )
        .unwrap(),
        source: CaptureSource::Repository,
        delivery: CaptureDelivery::RepositoryBacked,
        captured_at_unix: 1_721_337_601,
        summary: "The article argument should enter Qiongli without exposing a repository path."
            .to_string(),
        changes: Vec::new(),
        decisions: Vec::new(),
        evidence: Vec::new(),
        contradictions: Vec::new(),
        next_actions: vec!["Review the repository capture before consolidation.".to_string()],
    }
    .into_capture()
    .unwrap();
    let capture_id = capture.capture_id.as_str().to_string();
    let repository_inbox = project_root.join("context/capture-inbox");
    fs::create_dir_all(&repository_inbox).unwrap();
    let repository_packet = repository_inbox.join(format!("{capture_id}.json"));
    fs::write(&repository_packet, capture.to_canonical_json().unwrap()).unwrap();

    let list = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "list".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    assert!(list.status.success(), "{}", public_output(&list));
    assert!(!output_contains_path(&list, &project_root));
    assert!(!output_contains_path(&list, &repository_packet));
    let list_json = parse_json(&list);
    assert_eq!(list_json["command"], "project-capture-repository-list");
    assert_eq!(list_json["inbox"]["pendingCount"], 1);
    assert_eq!(list_json["inbox"]["acceptedCount"], 0);
    assert_eq!(list_json["inbox"]["entries"][0]["state"], "pending");
    assert_eq!(
        list_json["inbox"]["entries"][0]["repositoryEntry"],
        format!("context/capture-inbox/{capture_id}.json")
    );

    let read = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "read".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
        ],
        true,
    );
    assert!(read.status.success(), "{}", public_output(&read));
    assert_eq!(parse_json(&read)["capture"]["capture_id"], capture_id);
    assert!(!output_contains_path(&read, &project_root));

    let delivery_preview = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "delivery".into(),
            "preview".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
            "--queued-at-unix".into(),
            "1721337610".into(),
        ],
        true,
    );
    assert!(
        delivery_preview.status.success(),
        "{}",
        public_output(&delivery_preview)
    );
    assert!(!output_contains_path(&delivery_preview, &project_root));
    assert!(!output_contains_path(&delivery_preview, &repository_packet));
    let delivery_preview_json = parse_json(&delivery_preview);
    assert_eq!(
        delivery_preview_json["command"],
        "project-capture-repository-delivery-preview"
    );
    assert_eq!(
        delivery_preview_json["preview"]["destinationProjectId"],
        project_id
    );
    assert_eq!(
        delivery_preview_json["preview"]["expectedDestinationRevision"],
        1
    );
    assert_eq!(
        delivery_preview_json["preview"]["approvalsRequired"],
        serde_json::json!(["delivery-write"])
    );
    let delivery_digest = delivery_preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let delivery_apply = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "delivery".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
            "--queued-at-unix".into(),
            "1721337610".into(),
            "--expected-plan-digest".into(),
            delivery_digest.into(),
            "--approve-delivery-write".into(),
        ],
        true,
    );
    assert!(
        delivery_apply.status.success(),
        "{}",
        public_output(&delivery_apply)
    );
    assert!(!output_contains_path(&delivery_apply, &project_root));
    assert_eq!(
        parse_json(&delivery_apply)["commit"]["delivery"]["state"],
        "queued"
    );

    let rejected_path = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "preview".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
            "--repository-path".into(),
            fixture.root.as_os_str().to_owned(),
        ],
        true,
    );
    assert_eq!(rejected_path.status.code(), Some(2));
    assert!(
        rejected_path
            .stderr
            .starts_with(b"error: unknown repository capture option\n")
    );
    assert!(!output_contains_path(&rejected_path, &fixture.root));

    let preview = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "preview".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
        ],
        true,
    );
    assert!(preview.status.success(), "{}", public_output(&preview));
    assert!(!output_contains_path(&preview, &project_root));
    let preview_json = parse_json(&preview);
    assert_eq!(
        preview_json["command"],
        "project-capture-repository-preview"
    );
    assert_eq!(
        preview_json["preview"]["intake"]["delivery"],
        "repository-backed"
    );
    assert_eq!(
        preview_json["preview"]["intake"]["approvalsRequired"],
        serde_json::json!(["filesystem-write"])
    );
    let plan_digest = preview_json["preview"]["intake"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();

    let missing_approval = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
            "--expected-plan-digest".into(),
            plan_digest.clone().into(),
        ],
        true,
    );
    assert_eq!(missing_approval.status.code(), Some(2));
    assert!(missing_approval.stderr.starts_with(
        b"error: repository capture apply requires plan digest and filesystem approval\n"
    ));

    let mismatched = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
            "--expected-plan-digest".into(),
            "0".repeat(64).into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert_eq!(mismatched.status.code(), Some(1));
    assert_eq!(mismatched.stderr, b"error: project-plan-mismatch\n");

    let apply = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
            "--expected-plan-digest".into(),
            plan_digest.clone().into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(apply.status.success(), "{}", public_output(&apply));
    assert!(!output_contains_path(&apply, &project_root));
    assert!(!output_contains_path(&apply, &repository_packet));
    let apply_json = parse_json(&apply);
    assert_eq!(apply_json["command"], "project-capture-repository-apply");
    assert_eq!(apply_json["commit"]["captureId"], capture_id);
    assert!(
        apply_json["commit"]["acknowledgement"]
            .as_str()
            .unwrap()
            .starts_with("ack_")
    );

    let accepted = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "list".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    assert!(accepted.status.success(), "{}", public_output(&accepted));
    let accepted_json = parse_json(&accepted);
    assert_eq!(accepted_json["inbox"]["pendingCount"], 0);
    assert_eq!(accepted_json["inbox"]["acceptedCount"], 1);
    assert_eq!(accepted_json["inbox"]["entries"][0]["state"], "accepted");
    assert_eq!(
        accepted_json["inbox"]["entries"][0]["historyEntry"],
        format!("context/captures/{capture_id}.json")
    );

    let inbox = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "list".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    assert!(inbox.status.success(), "{}", public_output(&inbox));
    assert_eq!(parse_json(&inbox)["inbox"]["pendingReviewCount"], 1);

    let coverage = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "coverage".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    assert!(coverage.status.success(), "{}", public_output(&coverage));
    assert!(!output_contains_path(&coverage, &project_root));
    assert!(!output_contains_path(&coverage, &repository_packet));
    let coverage_json = parse_json(&coverage);
    assert_eq!(coverage_json["command"], "project-capture-coverage");
    assert_eq!(coverage_json["coverage"]["captureCount"], 1);
    assert_eq!(coverage_json["coverage"]["repositoryBackedCount"], 1);
    assert_eq!(coverage_json["coverage"]["pendingReviewCount"], 1);
    assert_eq!(coverage_json["coverage"]["unknownSourceCount"], 6);
    assert_eq!(
        coverage_json["coverage"]["sources"]
            .as_array()
            .unwrap()
            .len(),
        7
    );

    fs::write(
        project_root.join("context/research_state.md"),
        b"RQ: How should article memory survive across research clients?\n",
    )
    .unwrap();
    let artifact_changes = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "changes".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    assert!(
        artifact_changes.status.success(),
        "{}",
        public_output(&artifact_changes)
    );
    assert!(!output_contains_path(&artifact_changes, &project_root));
    let artifact_changes_json = parse_json(&artifact_changes);
    assert_eq!(
        artifact_changes_json["command"],
        "project-capture-artifact-changes"
    );
    let changes = &artifact_changes_json["changes"];
    assert_eq!(changes["state"], "unattributed");
    assert_eq!(changes["changeCount"], 1);
    assert_eq!(changes["unattributedCount"], 1);
    assert_eq!(changes["changes"][0]["detection"], "exact");
    assert_eq!(changes["changes"][0]["effect"], "created");
    assert_eq!(
        changes["changes"][0]["relativePaths"],
        serde_json::json!(["context/research_state.md"])
    );
    assert!(changes["changes"][0].get("source").is_none());
    assert!(changes["changes"][0].get("client").is_none());
    assert!(changes["changes"][0].get("session").is_none());

    let replay = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "repository".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.into(),
            "--capture-id".into(),
            capture_id.into(),
            "--expected-plan-digest".into(),
            plan_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert_eq!(replay.status.code(), Some(1));
    assert_eq!(replay.stderr, b"error: research-capture-already-applied\n");
    assert!(!output_contains_path(&replay, &project_root));

    fs::remove_dir_all(runtime_root).expect("outside-checkout runtime root must be removed");
}

#[test]
fn copied_binary_consolidates_a_reviewed_capture_without_runtime() {
    fn snapshot(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
        let mut files = std::collections::BTreeMap::new();
        if root.exists() {
            for entry in fs::read_dir(root).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    files.extend(snapshot(&path));
                } else {
                    files.insert(path.clone(), fs::read(path).unwrap());
                }
            }
        }
        files
    }

    let fixture = Fixture::new("tier1-capture-consolidation");
    // Qualify this same journey against a named package without replacing build output.
    let source_executable = std::env::var_os("QIONGLI_TEST_CONSOLIDATION_BINARY").map_or_else(
        || PathBuf::from(env!("CARGO_BIN_EXE_qiongli")),
        PathBuf::from,
    );
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-tier1-consolidation-runtime-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&runtime_root).expect("outside-checkout runtime root must be created");
    set_private_directory_mode(&runtime_root);
    let copied = runtime_root.join(
        source_executable
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source_executable, &copied)
        .expect("native executable must copy outside the checkout");

    let project_root = fixture.root.join("consolidated-paper");
    let create_preview = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "create".into(),
            "preview".into(),
            "--root".into(),
            project_root.as_os_str().to_owned(),
            "--name".into(),
            "Copied Binary Consolidation".into(),
            "--kind".into(),
            "article".into(),
            "--stage".into(),
            "writing".into(),
        ],
        true,
    );
    assert!(
        create_preview.status.success(),
        "{}",
        public_output(&create_preview)
    );
    let create_preview_json = parse_json(&create_preview);
    let project_id = create_preview_json["preview"]["projectId"]
        .as_str()
        .unwrap()
        .to_string();
    let create_digest = create_preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let create_apply = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "create".into(),
            "apply".into(),
            "--root".into(),
            project_root.as_os_str().to_owned(),
            "--name".into(),
            "Copied Binary Consolidation".into(),
            "--kind".into(),
            "article".into(),
            "--stage".into(),
            "writing".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--expected-plan-digest".into(),
            create_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        create_apply.status.success(),
        "{}",
        public_output(&create_apply)
    );

    let captured_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock must follow the Unix epoch")
        .as_secs();
    let draft = ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            ProjectId::parse(project_id.clone()).unwrap(),
            1,
            ProjectStage::Writing,
            "Preserve the reviewed article argument",
            CapturePolicy::ReviewRequired,
        )
        .unwrap(),
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Portable,
        captured_at_unix,
        summary: "The reviewed argument must become portable academic state.".to_string(),
        changes: Vec::new(),
        decisions: Vec::new(),
        evidence: Vec::new(),
        contradictions: Vec::new(),
        next_actions: vec!["Inspect the consolidated research state.".to_string()],
    };
    let capture = draft.clone().into_capture().unwrap();
    let capture_id = capture.capture_id.as_str().to_string();
    let capture_file = fixture.root.join("reviewed-capture.json");
    fs::write(&capture_file, capture.to_canonical_json().unwrap()).unwrap();
    let intake_preview = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "preview".into(),
            "--file".into(),
            capture_file.as_os_str().to_owned(),
        ],
        true,
    );
    assert!(
        intake_preview.status.success(),
        "{}",
        public_output(&intake_preview)
    );
    let intake_digest = parse_json(&intake_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let before_intake = (snapshot(&project_root), snapshot(&fixture.config_root));
    let edited_capture = ResearchCaptureDraftV1 {
        summary: "Edited synthetic candidate; the original review does not approve this text."
            .to_string(),
        ..draft
    }
    .into_capture()
    .unwrap();
    assert_ne!(edited_capture.capture_id, capture.capture_id);
    fs::write(&capture_file, edited_capture.to_canonical_json().unwrap()).unwrap();
    let edited_preview = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "preview".into(),
            "--file".into(),
            capture_file.as_os_str().to_owned(),
        ],
        true,
    );
    assert!(
        edited_preview.status.success(),
        "{}",
        public_output(&edited_preview)
    );
    assert_ne!(
        parse_json(&edited_preview)["preview"]["planDigest"],
        intake_digest
    );
    let stale_intake = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "apply".into(),
            "--file".into(),
            capture_file.as_os_str().to_owned(),
            "--expected-plan-digest".into(),
            intake_digest.clone().into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert_eq!(stale_intake.status.code(), Some(1));
    assert_eq!(stale_intake.stderr, b"error: project-plan-mismatch\n");
    assert_eq!(
        before_intake,
        (snapshot(&project_root), snapshot(&fixture.config_root))
    );
    fs::write(&capture_file, capture.to_canonical_json().unwrap()).unwrap();
    let intake_apply = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "apply".into(),
            "--file".into(),
            capture_file.as_os_str().to_owned(),
            "--expected-plan-digest".into(),
            intake_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        intake_apply.status.success(),
        "{}",
        public_output(&intake_apply)
    );

    let consolidation_preview = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "consolidate".into(),
            "preview".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
        ],
        true,
    );
    assert!(
        consolidation_preview.status.success(),
        "{}",
        public_output(&consolidation_preview)
    );
    assert!(!output_contains_path(&consolidation_preview, &project_root));
    assert!(!output_contains_path(&consolidation_preview, &capture_file));
    let consolidation_preview_json = parse_json(&consolidation_preview);
    assert_eq!(
        consolidation_preview_json["command"],
        "project-capture-consolidate-preview"
    );
    assert_eq!(consolidation_preview_json["preview"]["outcome"], "ready");
    assert_eq!(
        consolidation_preview_json["preview"]["approvalsRequired"],
        serde_json::json!(["academic-consolidation", "filesystem-write"])
    );
    let reviewed_at_unix = consolidation_preview_json["preview"]["reviewedAtUnix"]
        .as_u64()
        .unwrap();
    let consolidation_digest = consolidation_preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();

    // Each invocation has exited: persisted previews/captures grant no next-process approval.
    let before_consolidation = (snapshot(&project_root), snapshot(&fixture.config_root));
    let apply_args: Vec<OsString> = vec![
        "project".into(),
        "capture".into(),
        "consolidate".into(),
        "apply".into(),
        "--project-id".into(),
        project_id.clone().into(),
        "--capture-id".into(),
        capture_id.clone().into(),
        "--reviewed-at-unix".into(),
        reviewed_at_unix.to_string().into(),
        "--expected-plan-digest".into(),
        consolidation_digest.clone().into(),
    ];
    let mut approved_args = apply_args.clone();
    approved_args.extend([
        "--approve-academic-review".into(),
        "--approve-filesystem-write".into(),
    ]);
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        // Hold the actual write lock: no approved child can commit before it is killed.
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(fixture.state_root().join("research-library/.library.lock"))
            .unwrap();
        lock.lock().unwrap();
        let blocked = run_configured_os(&copied, &fixture, &approved_args, true);
        assert_eq!(blocked.status.code(), Some(1));
        assert_eq!(blocked.stderr, b"error: project-library-lock-busy\n");
        let mut child = fixture_command(&copied, &fixture)
            .args(&approved_args)
            .env("PATH", "")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        let before_kill = child.try_wait();
        let killed = child.kill();
        let output = child.wait_with_output().unwrap();
        assert!(before_kill.unwrap().is_none());
        assert!(killed.is_ok());
        assert_eq!(output.status.signal(), Some(9));
        assert!(output.stdout.is_empty());
        drop(lock);
        assert_eq!(
            before_consolidation,
            (snapshot(&project_root), snapshot(&fixture.config_root))
        );
    }
    for partial_approval in [
        None,
        Some("--approve-academic-review"),
        Some("--approve-filesystem-write"),
    ] {
        let mut args = apply_args.clone();
        if let Some(flag) = partial_approval {
            args.push(flag.into());
        }
        let refused = run_configured_os(&copied, &fixture, &args, true);
        assert_eq!(refused.status.code(), Some(2));
        assert!(refused.stderr.starts_with(b"error: capture consolidation apply requires review timestamp, plan digest, academic approval, and filesystem approval\n"));
        assert_eq!(
            before_consolidation,
            (snapshot(&project_root), snapshot(&fixture.config_root))
        );
    }
    let reopened = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "read".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
        ],
        true,
    );
    assert!(reopened.status.success(), "{}", public_output(&reopened));
    assert_eq!(
        parse_json(&reopened)["capture"],
        serde_json::to_value(&capture).unwrap()
    );

    let changed_review_time = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "consolidate".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
            "--reviewed-at-unix".into(),
            reviewed_at_unix.saturating_add(1).to_string().into(),
            "--expected-plan-digest".into(),
            consolidation_digest.clone().into(),
            "--approve-academic-review".into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert_eq!(changed_review_time.status.code(), Some(1));
    assert_eq!(
        changed_review_time.stderr,
        b"error: project-plan-mismatch\n"
    );
    assert_eq!(
        before_consolidation,
        (snapshot(&project_root), snapshot(&fixture.config_root))
    );

    let consolidation_apply = run_configured_os(&copied, &fixture, &approved_args, true);
    assert!(
        consolidation_apply.status.success(),
        "{}",
        public_output(&consolidation_apply)
    );
    assert!(!output_contains_path(&consolidation_apply, &project_root));
    let consolidation_apply_json = parse_json(&consolidation_apply);
    assert_eq!(
        consolidation_apply_json["command"],
        "project-capture-consolidate-apply"
    );
    assert_eq!(consolidation_apply_json["commit"]["semanticRevision"], 2);
    assert_eq!(
        consolidation_apply_json["commit"]["artifactsUpdated"],
        serde_json::json!(["research-state"])
    );
    let receipt_entry = consolidation_apply_json["commit"]["receiptEntry"]
        .as_str()
        .unwrap();
    assert!(project_root.join(receipt_entry).is_file());
    let research_state =
        fs::read_to_string(project_root.join("context/research_state.md")).unwrap();
    assert!(research_state.contains(&capture_id));
    assert!(research_state.contains(&capture.summary));

    let inbox = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "list".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    assert!(inbox.status.success(), "{}", public_output(&inbox));
    let inbox_json = parse_json(&inbox);
    assert_eq!(inbox_json["inbox"]["pendingReviewCount"], 0);
    assert_eq!(inbox_json["inbox"]["appliedCount"], 1);
    assert_eq!(inbox_json["inbox"]["entries"][0]["state"], "applied");

    let after_commit = (snapshot(&project_root), snapshot(&fixture.config_root));
    let replay_preview = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "consolidate".into(),
            "preview".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
            "--reviewed-at-unix".into(),
            reviewed_at_unix.saturating_add(1).to_string().into(),
        ],
        true,
    );
    assert!(
        replay_preview.status.success(),
        "{}",
        public_output(&replay_preview)
    );
    let replay_preview_json = parse_json(&replay_preview);
    assert_eq!(
        replay_preview_json["preview"]["outcome"],
        "already-consolidated"
    );
    let replay_digest = replay_preview_json["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let replay = run_configured_os(
        &copied,
        &fixture,
        &[
            "project".into(),
            "capture".into(),
            "consolidate".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.into(),
            "--capture-id".into(),
            capture_id.into(),
            "--reviewed-at-unix".into(),
            reviewed_at_unix.saturating_add(1).to_string().into(),
            "--expected-plan-digest".into(),
            replay_digest.into(),
            "--approve-academic-review".into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert_eq!(replay.status.code(), Some(1));
    assert_eq!(
        replay.stderr,
        b"error: capture-consolidation-already-applied\n"
    );
    assert_eq!(
        after_commit,
        (snapshot(&project_root), snapshot(&fixture.config_root))
    );

    fs::remove_dir_all(runtime_root).expect("outside-checkout runtime root must be removed");
}

#[test]
fn copied_binary_round_trips_portable_and_legacy_projects_without_runtime() {
    let source_fixture = Fixture::new("tier1-portable-source");
    let destination_fixture = Fixture::new("tier1-portable-destination");
    let migration_fixture = Fixture::new("tier1-migration");
    let source_executable = PathBuf::from(env!("CARGO_BIN_EXE_qiongli"));
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-tier1-project-runtime-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&runtime_root).expect("outside-checkout runtime root must be created");
    set_private_directory_mode(&runtime_root);
    let copied = runtime_root.join(
        source_executable
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source_executable, &copied)
        .expect("native executable must copy outside the checkout");

    let project_root = source_fixture.root.join("portable-source-paper");
    let create_preview = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "create".into(),
            "preview".into(),
            "--root".into(),
            project_root.as_os_str().to_owned(),
            "--name".into(),
            "Tier 1 Portable Paper".into(),
            "--kind".into(),
            "article".into(),
            "--stage".into(),
            "writing".into(),
        ],
        true,
    );
    assert!(
        create_preview.status.success(),
        "{}",
        public_output(&create_preview)
    );
    assert!(!output_contains_path(&create_preview, &project_root));
    let create_preview = parse_json(&create_preview);
    let project_id = create_preview["preview"]["projectId"]
        .as_str()
        .unwrap()
        .to_string();
    let create_digest = create_preview["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let create_apply = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "create".into(),
            "apply".into(),
            "--root".into(),
            project_root.as_os_str().to_owned(),
            "--name".into(),
            "Tier 1 Portable Paper".into(),
            "--kind".into(),
            "article".into(),
            "--stage".into(),
            "writing".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--expected-plan-digest".into(),
            create_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        create_apply.status.success(),
        "{}",
        public_output(&create_apply)
    );
    assert!(!output_contains_path(&create_apply, &project_root));

    let capture = ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            ProjectId::parse(project_id.clone()).unwrap(),
            1,
            ProjectStage::Writing,
            "Retain the cross-client article argument",
            CapturePolicy::ReviewRequired,
        )
        .unwrap(),
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Portable,
        captured_at_unix: 1_721_337_600,
        summary: "The article argument should persist independently of a chat session.".to_string(),
        changes: Vec::new(),
        decisions: Vec::new(),
        evidence: Vec::new(),
        contradictions: Vec::new(),
        next_actions: vec!["Review the captured argument before consolidation.".to_string()],
    }
    .into_capture()
    .unwrap();
    let capture_id = capture.capture_id.as_str().to_string();
    let capture_file = source_fixture.root.join("portable-capture.json");
    fs::write(&capture_file, capture.to_canonical_json().unwrap()).unwrap();
    let capture_preview = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "capture".into(),
            "preview".into(),
            "--file".into(),
            capture_file.as_os_str().to_owned(),
        ],
        true,
    );
    assert!(
        capture_preview.status.success(),
        "{}",
        public_output(&capture_preview)
    );
    assert!(!output_contains_path(&capture_preview, &capture_file));
    let capture_digest = parse_json(&capture_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let capture_apply = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "capture".into(),
            "apply".into(),
            "--file".into(),
            capture_file.as_os_str().to_owned(),
            "--expected-plan-digest".into(),
            capture_digest.clone().into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        capture_apply.status.success(),
        "{}",
        public_output(&capture_apply)
    );
    assert!(!output_contains_path(&capture_apply, &capture_file));
    assert_eq!(
        parse_json(&capture_apply)["command"],
        "project-capture-apply"
    );

    let capture_list = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "capture".into(),
            "list".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    assert!(
        capture_list.status.success(),
        "{}",
        public_output(&capture_list)
    );
    let capture_list_json = parse_json(&capture_list);
    assert_eq!(capture_list_json["inbox"]["pendingReviewCount"], 1);
    assert_eq!(
        capture_list_json["inbox"]["entries"][0]["captureId"],
        capture_id
    );
    let capture_read = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "capture".into(),
            "read".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--capture-id".into(),
            capture_id.clone().into(),
        ],
        true,
    );
    assert!(
        capture_read.status.success(),
        "{}",
        public_output(&capture_read)
    );
    assert_eq!(
        parse_json(&capture_read)["capture"]["capture_id"],
        capture_id
    );
    let replay = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "capture".into(),
            "apply".into(),
            "--file".into(),
            capture_file.as_os_str().to_owned(),
            "--expected-plan-digest".into(),
            capture_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert_eq!(replay.status.code(), Some(1));
    assert_eq!(replay.stderr, b"error: research-capture-already-applied\n");
    assert!(!output_contains_path(&replay, &capture_file));

    let research_state = b"RQ: Does a portable project survive every Tier 1 runtime?\nThesis: Canonical artifacts remain portable.\n";
    fs::write(
        project_root.join("context/research_state.md"),
        research_state,
    )
    .unwrap();
    fs::write(
        project_root.join("secret-token.txt"),
        b"portable-secret-canary",
    )
    .unwrap();
    fs::create_dir(project_root.join("sessions")).unwrap();
    fs::write(
        project_root.join("sessions/raw.json"),
        b"raw-session-canary",
    )
    .unwrap();
    let refresh_preview = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "refresh".into(),
            "preview".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    let refresh_digest = parse_json(&refresh_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let refresh_apply = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "refresh".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--expected-plan-digest".into(),
            refresh_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        refresh_apply.status.success(),
        "{}",
        public_output(&refresh_apply)
    );
    let stale_capture_list = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "capture".into(),
            "list".into(),
            "--project-id".into(),
            project_id.clone().into(),
        ],
        true,
    );
    assert!(
        stale_capture_list.status.success(),
        "{}",
        public_output(&stale_capture_list)
    );
    assert_eq!(parse_json(&stale_capture_list)["inbox"]["staleCount"], 1);

    let portable_package = source_fixture.root.join("portable-package");
    let export_preview = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "export".into(),
            "preview".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--destination".into(),
            portable_package.as_os_str().to_owned(),
        ],
        true,
    );
    assert!(
        export_preview.status.success(),
        "{}",
        public_output(&export_preview)
    );
    assert!(!output_contains_path(&export_preview, &portable_package));
    assert_eq!(
        parse_json(&export_preview)["preview"]["excludedEntryCount"],
        3
    );
    let export_digest = parse_json(&export_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let export_apply = run_configured_os(
        &copied,
        &source_fixture,
        &[
            "project".into(),
            "export".into(),
            "apply".into(),
            "--project-id".into(),
            project_id.clone().into(),
            "--destination".into(),
            portable_package.as_os_str().to_owned(),
            "--expected-plan-digest".into(),
            export_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        export_apply.status.success(),
        "{}",
        public_output(&export_apply)
    );
    assert!(!portable_package.join("project/secret-token.txt").exists());
    assert!(!portable_package.join("project/sessions").exists());

    let imported_root = destination_fixture.root.join("portable-imported-paper");
    let import_preview = run_configured_os(
        &copied,
        &destination_fixture,
        &[
            "project".into(),
            "import".into(),
            "preview".into(),
            "--source".into(),
            portable_package.as_os_str().to_owned(),
            "--root".into(),
            imported_root.as_os_str().to_owned(),
        ],
        true,
    );
    assert!(
        import_preview.status.success(),
        "{}",
        public_output(&import_preview)
    );
    assert!(!output_contains_path(&import_preview, &portable_package));
    assert!(!output_contains_path(&import_preview, &imported_root));
    let import_digest = parse_json(&import_preview)["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let import_apply = run_configured_os(
        &copied,
        &destination_fixture,
        &[
            "project".into(),
            "import".into(),
            "apply".into(),
            "--source".into(),
            portable_package.as_os_str().to_owned(),
            "--root".into(),
            imported_root.as_os_str().to_owned(),
            "--expected-plan-digest".into(),
            import_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        import_apply.status.success(),
        "{}",
        public_output(&import_apply)
    );
    let imported = parse_json(&run_configured_os(
        &copied,
        &destination_fixture,
        &["project".into(), "list".into()],
        true,
    ));
    assert_eq!(imported["library"]["projects"][0]["projectId"], project_id);
    assert_eq!(imported["library"]["projects"][0]["semanticRevision"], 2);
    assert_eq!(
        fs::read(imported_root.join("context/research_state.md")).unwrap(),
        research_state
    );

    let legacy_root = migration_fixture.root.join("legacy-paper");
    let migrated_root = migration_fixture.root.join("migrated-paper");
    fs::create_dir(&legacy_root).unwrap();
    fs::create_dir(legacy_root.join("context")).unwrap();
    let legacy_state = b"RQ: Can legacy artifacts move without their private runtime?\n";
    fs::write(legacy_root.join("context/research_state.md"), legacy_state).unwrap();
    fs::create_dir(legacy_root.join(".qiongli")).unwrap();
    fs::write(
        legacy_root.join(".qiongli/session.json"),
        b"raw-session-canary",
    )
    .unwrap();
    let migration_preview = run_configured_os(
        &copied,
        &migration_fixture,
        &[
            "project".into(),
            "migrate".into(),
            "preview".into(),
            "--source".into(),
            legacy_root.as_os_str().to_owned(),
            "--root".into(),
            migrated_root.as_os_str().to_owned(),
            "--name".into(),
            "Tier 1 Migrated Paper".into(),
        ],
        true,
    );
    assert!(
        migration_preview.status.success(),
        "{}",
        public_output(&migration_preview)
    );
    assert!(!output_contains_path(&migration_preview, &legacy_root));
    assert!(!output_contains_path(&migration_preview, &migrated_root));
    let migration_preview = parse_json(&migration_preview);
    let migration_project_id = migration_preview["preview"]["projectId"]
        .as_str()
        .unwrap()
        .to_string();
    let migration_digest = migration_preview["preview"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let migration_manifest_created_at_unix = migration_preview["preview"]["manifestCreatedAtUnix"]
        .as_u64()
        .unwrap()
        .to_string();
    let migration_apply = run_configured_os(
        &copied,
        &migration_fixture,
        &[
            "project".into(),
            "migrate".into(),
            "apply".into(),
            "--source".into(),
            legacy_root.as_os_str().to_owned(),
            "--root".into(),
            migrated_root.as_os_str().to_owned(),
            "--name".into(),
            "Tier 1 Migrated Paper".into(),
            "--project-id".into(),
            migration_project_id.into(),
            "--manifest-created-at-unix".into(),
            migration_manifest_created_at_unix.into(),
            "--expected-plan-digest".into(),
            migration_digest.into(),
            "--approve-filesystem-write".into(),
        ],
        true,
    );
    assert!(
        migration_apply.status.success(),
        "{}",
        public_output(&migration_apply)
    );
    assert_eq!(
        fs::read(legacy_root.join("context/research_state.md")).unwrap(),
        legacy_state
    );
    assert!(legacy_root.join(".qiongli/session.json").is_file());
    assert_eq!(
        fs::read(migrated_root.join("context/research_state.md")).unwrap(),
        legacy_state
    );
    assert!(!migrated_root.join(".qiongli/session.json").exists());

    fs::remove_dir_all(runtime_root).expect("outside-checkout runtime root must be removed");
}

#[cfg(unix)]
#[test]
fn update_status_and_channel_use_independent_revision_safe_state_without_path() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::new("update-state");
    let executable = Path::new(env!("CARGO_BIN_EXE_qiongli"));
    let run_update = |args: &[&str]| {
        let mut command = fixture_command(executable, &fixture);
        command.env("PATH", "").args(args);
        command
            .output()
            .expect("configured update command should start without PATH")
    };

    let status = run_update(&["update", "status"]);
    assert!(status.status.success(), "{}", public_output(&status));
    let status_json = parse_json(&status);
    assert_eq!(status_json["command"], "update-status");
    assert_eq!(status_json["revision"], 0);
    assert_eq!(status_json["selected_stream"], "beta");
    assert!(!fixture.config_root.exists());

    let changed = run_update(&[
        "update",
        "channel",
        "--expected-revision",
        "0",
        "--stream",
        "stable",
    ]);
    assert!(changed.status.success(), "{}", public_output(&changed));
    let changed_json = parse_json(&changed);
    assert_eq!(changed_json["command"], "update-channel");
    assert_eq!(changed_json["revision"], 1);
    assert_eq!(changed_json["selected_stream"], "stable");

    let update_state = fixture.state_root().join(UPDATE_STATE_FILE);
    assert!(update_state.is_file());
    assert_eq!(
        fs::metadata(&update_state).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(!fixture.settings_path().exists());

    let stale = run_update(&[
        "update",
        "channel",
        "--expected-revision",
        "0",
        "--stream",
        "beta",
    ]);
    assert_eq!(stale.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&stale.stderr),
        "error: revision-conflict\n"
    );
}

#[test]
fn root_and_nested_help_use_stdout_and_return_success() {
    for args in [
        ["--help"].as_slice(),
        ["-h"].as_slice(),
        ["content", "--help"].as_slice(),
        ["config", "--help"].as_slice(),
        ["app", "--help"].as_slice(),
        ["install", "--help"].as_slice(),
        ["install", "native", "--help"].as_slice(),
        ["project", "capture", "--help"].as_slice(),
        ["project", "capture", "consolidate", "--help"].as_slice(),
    ] {
        let output = run(args);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
        assert!(output.stderr.is_empty());
    }

    let root = run(&["--help"]);
    let root_help = String::from_utf8_lossy(&root.stdout);
    assert!(root_help.contains("qiongli ui"));
    assert!(!root_help.contains("content materialize"));
    assert!(!root_help.contains("ui --candidate"));
    assert!(!root_help.contains("install candidate"));
    assert!(!root_help.contains("install native"));

    let content = run(&["content", "--help"]);
    let content_help = String::from_utf8_lossy(&content.stdout);
    assert!(content_help.contains("qiongli app plan"));
    assert!(content_help.contains("managed-skills-plan-required"));

    let install = run(&["install", "--help"]);
    let install_help = String::from_utf8_lossy(&install.stdout);
    assert!(install_help.contains("release engineering"));
    assert!(install_help.contains("qiongli app plan"));
    assert!(install_help.contains("not a second end-user integration installer"));

    let app = run(&["app", "--help"]);
    let app_help = String::from_utf8_lossy(&app.stdout);
    assert!(app_help.contains("qiongli app plan integrations-install"));
    assert!(app_help.contains("qiongli app plan integrations-reconcile"));
    assert!(app_help.contains("qiongli app plan cli-remove"));
    assert!(app_help.contains("qiongli app plan cli-path-configure"));
    assert!(app_help.contains(
        "CLI install, PATH configuration, remove or predecessor restoration, and integration repair"
    ));
    assert!(app_help.contains("are separate state-bound plans"));
}

#[test]
fn content_list_is_versioned_deterministic_and_runtime_independent() {
    let first = run_without_path(&["content", "list"]);
    let second = run_without_path(&["content", "list"]);
    assert!(first.status.success());
    assert!(second.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);

    let value = parse_json(&first);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "content-list");
    assert_eq!(value["pack_id"], "qiongli-core");
    assert_eq!(value["pack_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(value["content_root_sha256"].as_str().unwrap().len(), 64);
    let profiles = value["profiles"]
        .as_array()
        .expect("content profiles must be an array");
    assert_eq!(profiles.len(), 3);
    assert_eq!(profiles[0]["id"], "skill-only");
    assert_eq!(profiles[1]["id"], "marketplace-lite");
    assert_eq!(profiles[2]["id"], "full");
}

#[test]
fn retired_content_materialization_requires_the_managed_plan_without_writing() {
    let fixture = Fixture::new("materialize-private-canary");
    let target = fixture.root.join("materialized-skill-only");
    let args = [
        OsString::from("content"),
        OsString::from("materialize"),
        OsString::from("--target"),
        target.clone().into_os_string(),
        OsString::from("--profile"),
        OsString::from("skill-only"),
    ];
    let output = run_configured_os(
        Path::new(env!("CARGO_BIN_EXE_qiongli")),
        &fixture,
        &args,
        true,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"error: managed-skills-plan-required\n");
    assert!(!output_contains_path(&output, &target));
    assert!(!target.exists());
}

#[test]
fn retired_content_materialization_preserves_unmanaged_and_relative_targets() {
    let fixture = Fixture::new("materialize-failure-private-canary");
    let unmanaged = fixture.root.join("unmanaged-target-private-canary");
    fs::create_dir(&unmanaged).expect("unmanaged target must be created");
    let existing = unmanaged.join("existing-private-canary.txt");
    fs::write(&existing, b"preserve-me").expect("existing target file must be written");
    let before = fs::read(&existing).unwrap();
    let args = [
        OsString::from("content"),
        OsString::from("materialize"),
        OsString::from("--profile"),
        OsString::from("full"),
        OsString::from("--target"),
        unmanaged.clone().into_os_string(),
    ];
    let output = run_configured_os(
        Path::new(env!("CARGO_BIN_EXE_qiongli")),
        &fixture,
        &args,
        false,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"error: managed-skills-plan-required\n");
    assert!(!output_contains_path(&output, &unmanaged));
    assert_eq!(fs::read(&existing).unwrap(), before);
    assert!(!unmanaged.join(MATERIALIZATION_RECEIPT_FILE).exists());

    let relative_canary = "relative-target-private-canary";
    let output = run_configured(
        &fixture,
        &[
            "content",
            "materialize",
            "--profile",
            "full",
            "--target",
            relative_canary,
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"error: managed-skills-plan-required\n");
    assert!(!public_output(&output).contains(relative_canary));
    assert!(!fixture.root.join(relative_canary).exists());
}

#[test]
fn config_show_and_set_are_redacted_revision_safe_and_owner_only() {
    let fixture = Fixture::new("config-lifecycle-private-canary");
    let missing = run_configured(&fixture, &["config", "show"]);
    assert!(missing.status.success(), "{}", public_output(&missing));
    assert!(missing.stderr.is_empty());
    let missing_json = parse_json(&missing);
    assert_eq!(missing_json["schema_version"], 1);
    assert_eq!(missing_json["command"], "config-show");
    assert_eq!(missing_json["config"]["root_source"], "override");
    assert_eq!(
        missing_json["config"]["symbolic_state_root"],
        "<configured-root>/v2"
    );
    assert_eq!(missing_json["config"]["state"], "missing");
    assert_eq!(missing_json["config"]["revision"], 0);
    assert_eq!(
        missing_json["config"]["default_profile"],
        "marketplace-lite"
    );
    assert!(!output_contains_path(&missing, &fixture.root));

    let set = run_configured(
        &fixture,
        &[
            "config",
            "set",
            "--default-profile",
            "full",
            "--expected-revision",
            "0",
        ],
    );
    assert!(set.status.success(), "{}", public_output(&set));
    let set_json = parse_json(&set);
    assert_eq!(set_json["schema_version"], 1);
    assert_eq!(set_json["command"], "config-set");
    assert_eq!(set_json["revision"], 1);
    assert_eq!(set_json["default_profile"], "full");
    assert_eq!(set_json["cleanup_required"], false);
    assert_private_config_permissions(&fixture);

    let ready = run_configured(&fixture, &["config", "show"]);
    assert!(ready.status.success());
    let ready_json = parse_json(&ready);
    assert_eq!(ready_json["config"]["state"], "ready");
    assert_eq!(ready_json["config"]["revision"], 1);
    assert_eq!(ready_json["config"]["default_profile"], "full");

    let before = fs::read(fixture.settings_path()).unwrap();
    let stale = run_configured(
        &fixture,
        &[
            "config",
            "set",
            "--expected-revision",
            "0",
            "--default-profile",
            "skill-only",
        ],
    );
    assert_eq!(stale.status.code(), Some(1));
    assert!(stale.stdout.is_empty());
    assert_eq!(stale.stderr, b"error: revision-conflict\n");
    assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
}

#[test]
fn backend_config_cli_is_read_only_and_direct_execution_is_unavailable() {
    let fixture = Fixture::new("backend-config-private-canary");

    let missing = run_configured(&fixture, &["config", "backend", "status"]);
    assert!(missing.status.success(), "{}", public_output(&missing));
    assert!(missing.stderr.is_empty());
    let missing_json = parse_json(&missing);
    assert_eq!(missing_json["schema_version"], 1);
    assert_eq!(missing_json["command"], "config-backend-status");
    assert_eq!(missing_json["revision"], 0);
    assert_eq!(missing_json["backend"]["backendId"], "openai-responses");
    assert_eq!(missing_json["backend"]["model"], "gpt-5.6-sol");
    assert_eq!(missing_json["backend"]["enabled"], false);
    assert_eq!(missing_json["backend"]["readiness"], "disabled");
    assert_eq!(missing_json["backend"]["testAvailable"], false);
    assert!(!fixture.config_root.exists());

    let missing_confirmation = run_configured(&fixture, &["config", "backend", "test"]);
    assert_eq!(missing_confirmation.status.code(), Some(2));
    assert!(missing_confirmation.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&missing_confirmation.stderr)
            .contains("host-driven execution required")
    );
    assert!(!fixture.config_root.exists());

    let enable_attempt = run_configured(
        &fixture,
        &[
            "config",
            "backend",
            "set",
            "--expected-revision",
            "0",
            "--enabled",
            "true",
        ],
    );
    assert_eq!(enable_attempt.status.code(), Some(2));
    assert!(enable_attempt.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&enable_attempt.stderr).contains("host-driven execution required")
    );
    assert!(!fixture.config_root.exists());

    let confirmed = run_configured(
        &fixture,
        &["config", "backend", "test", "--confirm-network-request"],
    );
    assert_eq!(confirmed.status.code(), Some(2));
    assert!(confirmed.stdout.is_empty());
    assert!(String::from_utf8_lossy(&confirmed.stderr).contains("host-driven execution required"));
    assert!(!fixture.config_root.exists());
}

#[test]
fn config_help_advertises_only_the_read_only_backend_migration_view() {
    let fixture = Fixture::new("backend-help-host-driven");
    let output = run_configured(&fixture, &["config", "--help"]);

    assert!(output.status.success(), "{}", public_output(&output));
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("qiongli config backend status"));
    assert!(help.contains("Model execution is owned by Codex, Claude Code"));
    assert!(!help.contains("config backend set"));
    assert!(!help.contains("config backend test"));
    assert!(!fixture.config_root.exists());
}

#[cfg(unix)]
fn assert_private_config_permissions(fixture: &Fixture) {
    use std::os::unix::fs::PermissionsExt;

    let state_mode = fs::metadata(fixture.state_root())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    let settings_mode = fs::metadata(fixture.settings_path())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(state_mode, 0o700);
    assert_eq!(settings_mode, 0o600);
}

#[cfg(not(unix))]
fn assert_private_config_permissions(_fixture: &Fixture) {}

#[test]
fn config_set_preserves_provider_fields_and_show_hides_public_identifiers() {
    let fixture = Fixture::new("provider-preservation-private-canary");
    let mut settings = GlobalSettings::default();
    settings.providers.crossref.enabled = true;
    settings.providers.crossref.email = Some(
        EmailAddress::parse("provider-email-private-canary@example.org")
            .expect("provider email fixture must be valid"),
    );
    let expected_providers = settings.providers.clone();
    fixture
        .store()
        .replace(0, settings)
        .expect("initial provider settings must persist");

    let output = run_configured(
        &fixture,
        &[
            "config",
            "set",
            "--expected-revision",
            "1",
            "--default-profile",
            "skill-only",
        ],
    );
    assert!(output.status.success(), "{}", public_output(&output));
    let loaded = fixture.store().load().unwrap();
    assert_eq!(loaded.revision, 2);
    assert_eq!(loaded.settings.default_profile, ProfileId::SkillOnly);
    assert_eq!(loaded.settings.providers, expected_providers);

    let shown = run_configured(&fixture, &["config", "show"]);
    assert!(shown.status.success());
    let shown_text = public_output(&shown);
    assert!(!shown_text.contains("provider-email-private-canary@example.org"));
    assert!(!text_contains_path(&shown_text, &fixture.root));
    let shown_json = parse_json(&shown);
    assert_eq!(
        shown_json["config"]["providers"]["crossref"]["readiness"],
        "ready"
    );
}

#[test]
fn status_and_doctor_are_read_only_redacted_and_explicit_about_limitations() {
    let fixture = Fixture::new("diagnostics-private-canary");
    let status = run_configured(&fixture, &["status"]);
    assert!(status.status.success(), "{}", public_output(&status));
    assert!(status.stderr.is_empty());
    let status_json = parse_json(&status);
    assert_eq!(status_json["schema_version"], 1);
    assert_eq!(status_json["command"], "status");
    assert_eq!(status_json["content"]["state"], "ready");
    assert_eq!(status_json["config"]["state"], "missing");
    assert!(!fixture.config_root.exists());

    let doctor = run_configured(&fixture, &["doctor"]);
    assert!(doctor.status.success(), "{}", public_output(&doctor));
    assert!(doctor.stderr.is_empty());
    let doctor_json = parse_json(&doctor);
    assert_eq!(doctor_json["schema_version"], 1);
    assert_eq!(doctor_json["command"], "doctor");
    assert!(doctor_json.get("paths").is_none());
    let checks = doctor_json["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 10);
    let expected_overall = if checks.iter().any(|check| {
        !matches!(
            check["state"].as_str(),
            Some("ready" | "missing" | "deferred")
        )
    }) {
        "attention"
    } else {
        "ready"
    };
    assert_eq!(doctor_json["overall"], expected_overall);
    let config = checks
        .iter()
        .find(|check| check["id"] == "global-config")
        .unwrap();
    assert_eq!(config["state"], "missing");
    assert_eq!(config["blocking"], false);
    let secure_store = checks
        .iter()
        .find(|check| check["id"] == "secure-store")
        .unwrap();
    assert_eq!(
        secure_store["state"],
        if cfg!(target_os = "macos") {
            "ready"
        } else {
            "unavailable"
        }
    );
    assert_eq!(secure_store["blocking"], false);
    let full_runtime = checks
        .iter()
        .find(|check| check["id"] == "full-runtime")
        .unwrap();
    assert_eq!(full_runtime["state"], "deferred");
    assert_eq!(full_runtime["blocking"], false);
    assert!(!fixture.config_root.exists());
    assert!(!output_contains_path(&doctor, &fixture.root));
}

#[test]
fn paths_and_exact_doctor_are_explicit_source_attributed_views() {
    let fixture = Fixture::new("exact-path-inspection");
    let codex_skills = fixture.home.join(".agents/skills");
    fs::create_dir_all(&codex_skills).unwrap();

    let paths = run_configured(&fixture, &["paths", "--json"]);
    assert!(paths.status.success(), "{}", public_output(&paths));
    assert!(paths.stderr.is_empty());
    let paths_json = parse_json(&paths);
    assert_eq!(paths_json["schema_version"], 1);
    assert_eq!(paths_json["command"], "paths");
    let codex = paths_json["paths"]
        .as_array()
        .unwrap()
        .iter()
        .find(|path| path["id"] == "codex-user-skills")
        .unwrap();
    assert_eq!(codex["exact_path"], codex_skills.to_string_lossy().as_ref());
    assert_eq!(codex["source"], "official-default");
    assert_eq!(codex["file_type"], "directory");
    assert_eq!(codex["selected"], true);

    let human = run_configured(&fixture, &["paths"]);
    assert!(human.status.success(), "{}", public_output(&human));
    assert!(output_contains_path(&human, &codex_skills));
    assert!(public_output(&human).contains("explicit exact-path view"));

    let exact_doctor = run_configured(&fixture, &["doctor", "--paths", "exact"]);
    assert!(
        exact_doctor.status.success(),
        "{}",
        public_output(&exact_doctor)
    );
    let exact_json = parse_json(&exact_doctor);
    assert!(
        exact_json["paths"]
            .as_array()
            .is_some_and(|paths| !paths.is_empty())
    );
    assert!(
        exact_json["paths"]
            .as_array()
            .is_some_and(|paths| paths.iter().any(|path| {
                path["exact_path"]
                    .as_str()
                    .is_some_and(|path| Path::new(path).starts_with(&fixture.root))
            }))
    );

    let redacted_doctor = run_configured(&fixture, &["doctor"]);
    assert!(redacted_doctor.status.success());
    assert!(!output_contains_path(&redacted_doctor, &fixture.root));
}

#[test]
fn doctor_returns_blocking_json_for_invalid_config_without_exposing_document_bytes() {
    let fixture = Fixture::new("doctor-invalid-private-canary");
    let initialized = run_configured(
        &fixture,
        &[
            "config",
            "set",
            "--expected-revision",
            "0",
            "--default-profile",
            "full",
        ],
    );
    assert!(initialized.status.success());
    fs::write(fixture.settings_path(), b"invalid-document-private-canary")
        .expect("managed config must be made invalid");

    let doctor = run_configured(&fixture, &["doctor"]);
    assert_eq!(doctor.status.code(), Some(1));
    assert!(doctor.stderr.is_empty());
    let doctor_json = parse_json(&doctor);
    assert_eq!(doctor_json["overall"], "attention");
    let config = doctor_json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == "global-config")
        .unwrap();
    assert_eq!(config["state"], "invalid");
    assert_eq!(config["blocking"], true);
    let output = public_output(&doctor);
    assert!(!output.contains("invalid-document-private-canary"));
    assert!(!text_contains_path(&output, &fixture.root));
}

#[test]
fn supported_commands_do_not_require_an_external_runtime_path() {
    for args in [
        ["--version"].as_slice(),
        ["--help"].as_slice(),
        ["content", "list"].as_slice(),
        ["install", "status"].as_slice(),
    ] {
        let output = run_without_home_or_path(args);
        assert!(output.status.success(), "{}", public_output(&output));
        assert!(output.stderr.is_empty());
    }

    let fixture = Fixture::new("empty-path-private-canary");
    for args in [["status"].as_slice(), ["doctor"].as_slice()] {
        let mut command = fixture_command(Path::new(env!("CARGO_BIN_EXE_qiongli")), &fixture);
        let output = command
            .args(args)
            .env("PATH", "")
            .output()
            .expect("native data command should start without PATH");
        assert!(output.status.success(), "{}", public_output(&output));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn install_status_is_read_only_and_truthful_for_source_builds() {
    let output = run_without_home_or_path(&["install", "status"]);
    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty());
    let value = parse_json(&output);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "install-status");
    assert_eq!(value["contracts"]["artifact_identity"], 1);
    assert_eq!(value["contracts"]["launch_grant"], 1);
    assert_eq!(value["contracts"]["release_authority"], 1);
    assert_eq!(value["contracts"]["release_envelope"], 1);
    assert_eq!(value["contracts"]["release_candidate"], 1);
    assert_eq!(value["contracts"]["install_plan"], 1);
    assert_eq!(value["contracts"]["install_receipt"], 1);
    assert_eq!(value["contracts"]["native_payload_install_receipt"], 1);
    assert_eq!(value["contracts"]["codex_adapter"], 1);
    assert_eq!(value["contracts"]["codex_registration_receipt"], 1);
    assert_eq!(value["contracts"]["codex_registration_state"], 1);
    assert_eq!(value["contracts"]["claude_adapter"], 1);
    assert_eq!(value["contracts"]["claude_registration_receipt"], 1);
    assert_eq!(value["contracts"]["claude_registration_state"], 1);
    assert!(value["current_target"]["os"].is_string());
    assert!(value["current_target"]["arch"].is_string());
    assert_eq!(value["transaction_engine"], "grant-and-approval-gated");
    assert_eq!(value["release_authority"], "unavailable");
    assert_eq!(value["source_commit"], "unavailable");
    assert_eq!(value["candidate"], "unavailable");
    assert_eq!(value["launch_grant"], "unavailable");
    assert_eq!(value["preview"], "unavailable");
    assert_eq!(value["apply"], "unavailable");
    assert_eq!(value["verify"], "receipt-backed");
    assert_eq!(value["remove"], "receipt-backed-explicit-approval");
    assert_eq!(value["targets"][0]["family"], "codex-local");
    assert_eq!(value["targets"][1]["family"], "claude-code-local");
    assert_eq!(value["targets"][0]["state"], "adapter-engine-ready");
    assert_eq!(value["targets"][1]["state"], "adapter-engine-ready");
}

#[test]
fn legacy_migration_inspection_is_path_free_and_preview_requires_packaged_authority() {
    let fixture = Fixture::new("legacy-migration-inspect");
    let plugin = fixture.home.join(".agents/plugins/qiongli");
    fs::create_dir_all(plugin.join("skills/qiongli-workflow")).unwrap();
    fs::write(
        plugin.join(".qiongli-managed.json"),
        serde_json::to_vec(&serde_json::json!({
            "managed_by": "qiongli-cli",
            "plugin": "qiongli",
            "surface": "plugin",
            "platform": "codex",
            "version": "1.19.0-beta.1"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(plugin.join("skills/qiongli-workflow/data"), b"legacy").unwrap();

    let inspect = run_configured(&fixture, &["migrate-1x", "inspect"]);
    assert!(inspect.status.success(), "{}", public_output(&inspect));
    assert!(inspect.stderr.is_empty());
    let value = parse_json(&inspect);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "inspect");
    assert_eq!(value["inventory"]["detected_item_count"], 1);
    assert_eq!(value["inventory"]["eligible_item_count"], 1);
    assert_eq!(value["inventory"]["review_item_count"], 0);
    assert!(!output_contains_path(&inspect, &fixture.root));

    let preview = run_configured(&fixture, &["migrate-1x", "preview"]);
    assert_eq!(preview.status.code(), Some(1));
    assert!(preview.stdout.is_empty());
    assert_eq!(preview.stderr, b"error: source-build-read-only\n");
    assert!(
        !fixture
            .home
            .join(".qiongli/v2/migrations/1x-to-2x")
            .exists()
    );
}

#[test]
fn source_build_has_no_release_authority_and_cannot_preview_native_install() {
    assert!(qiongli::embedded_release_authority().unwrap().is_none());
    assert!(qiongli::embedded_source_commit().is_none());
    let fixture = Fixture::new("native-preview-source-build-private-canary");
    let release = fixture.root.join("release-private-canary.json");
    let archive = fixture.root.join("archive-private-canary.zip");
    let managed = fixture.root.join("managed-private-canary");
    let output = run_configured_os(
        Path::new(env!("CARGO_BIN_EXE_qiongli")),
        &fixture,
        &[
            OsString::from("install"),
            OsString::from("native"),
            OsString::from("preview"),
            OsString::from("--release"),
            release.clone().into_os_string(),
            OsString::from("--archive"),
            archive.clone().into_os_string(),
            OsString::from("--managed-root"),
            managed.clone().into_os_string(),
            OsString::from("--target"),
            OsString::from("codex"),
        ],
        true,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"error: native-release-authority-unavailable\n"
    );
    let public = public_output(&output);
    assert!(!public.contains(release.to_string_lossy().as_ref()));
    assert!(!public.contains(archive.to_string_lossy().as_ref()));
    assert!(!public.contains(managed.to_string_lossy().as_ref()));
    assert!(!managed.exists());

    let candidate = fixture.root.join("candidate-private-canary.json");
    let notes = fixture.root.join("notes-private-canary.md");
    for prefix in [
        vec![
            OsString::from("install"),
            OsString::from("candidate"),
            OsString::from("preview"),
        ],
        vec!["install".into(), "candidate".into(), "stage-preview".into()],
        vec![
            "install".into(),
            "candidate".into(),
            "activate-preview".into(),
            "--previous-install-id".into(),
            format!("native-payload-{}", "a".repeat(64)).into(),
        ],
        vec![
            "install".into(),
            "candidate".into(),
            "stage".into(),
            "--expected-approval-digest".into(),
            "a".repeat(64).into(),
            "--approve-filesystem-write".into(),
        ],
        vec![
            "install".into(),
            "candidate".into(),
            "activate-prepare".into(),
            "--previous-install-id".into(),
            format!("native-payload-{}", "a".repeat(64)).into(),
            "--expected-preflight-digest".into(),
            "b".repeat(64).into(),
            "--approve-filesystem-write".into(),
        ],
        vec![OsString::from("ui")],
    ] {
        let mut args = prefix;
        args.extend([
            OsString::from("--candidate"),
            candidate.clone().into_os_string(),
            OsString::from("--archive"),
            archive.clone().into_os_string(),
            OsString::from("--release-notes"),
            notes.clone().into_os_string(),
            OsString::from("--target"),
            OsString::from("codex"),
        ]);
        let output = run_configured_os(
            Path::new(env!("CARGO_BIN_EXE_qiongli")),
            &fixture,
            &args,
            true,
        );
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            b"error: native-release-authority-unavailable\n"
        );
        let public = public_output(&output);
        assert!(!public.contains(candidate.to_string_lossy().as_ref()));
        assert!(!public.contains(archive.to_string_lossy().as_ref()));
        assert!(!public.contains(notes.to_string_lossy().as_ref()));
    }
}

#[test]
fn codex_install_status_discovers_without_writing_or_leaking_home() {
    let fixture = Fixture::new("codex-discovery-private-canary");
    let output = run_configured(&fixture, &["install", "codex", "status"]);
    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty());
    let value = parse_json(&output);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "install-codex-status");
    assert_eq!(value["target"]["source"], "missing");
    assert_eq!(value["target"]["marketplace"], "missing");
    assert_eq!(value["target"]["registration"], "absent");
    assert_eq!(
        value["target"]["marketplace_path"],
        "<user-home>/.agents/plugins/marketplace.json"
    );
    assert_eq!(value["launch_grant"], "unavailable");
    assert_eq!(value["preview"], "unavailable");
    assert_eq!(value["apply"], "unavailable");
    assert_eq!(value["activation"], "client-action-required");
    assert!(!fixture.home.join(".agents").exists());
    assert!(!fixture.home.join(".qiongli").exists());
    assert!(!public_output(&output).contains(fixture.home.to_string_lossy().as_ref()));
}

#[test]
fn claude_install_status_discovers_without_writing_or_leaking_home() {
    let fixture = Fixture::new("claude-discovery-private-canary");
    let claude_config_root = fixture.root.join("claude-config-private-canary");
    let output = fixture_command(Path::new(env!("CARGO_BIN_EXE_qiongli")), &fixture)
        .args(["install", "claude", "status"])
        .env("CLAUDE_CONFIG_DIR", &claude_config_root)
        .output()
        .expect("configured native qiongli binary should discover Claude state");
    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty());
    let value = parse_json(&output);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "install-claude-status");
    assert_eq!(value["target"]["skills_plugin"], "missing");
    assert_eq!(value["target"]["source"], "missing");
    assert_eq!(value["target"]["marketplace"], "missing");
    assert_eq!(value["target"]["registration"], "absent");
    assert_eq!(
        value["target"]["skills_plugin_path"],
        "<claude-config>/skills/qiongli-next"
    );
    assert_eq!(
        value["target"]["marketplace_path"],
        "<user-home>/.qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json"
    );
    assert_eq!(
        value["target"]["plugin_source_path"],
        "<user-home>/.qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next"
    );
    assert_eq!(
        value["target"]["marketplace_source"],
        "./plugins/qiongli-next"
    );
    assert_eq!(value["launch_grant"], "unavailable");
    assert_eq!(value["preview"], "unavailable");
    assert_eq!(value["apply"], "unavailable");
    assert_eq!(value["activation"], "reload-or-client-action-required");
    assert!(!claude_config_root.exists());
    assert!(!fixture.home.join(".qiongli").exists());
    assert!(!public_output(&output).contains(fixture.home.to_string_lossy().as_ref()));
    assert!(!public_output(&output).contains(claude_config_root.to_string_lossy().as_ref()));
}

#[test]
fn shared_client_inventory_reports_host_paths_and_project_without_writing() {
    let fixture = Fixture::new("shared-client-inventory-private-canary");
    let bin = fixture.root.join("observed-bin");
    fs::create_dir(&bin).expect("host evidence directory must exist");
    fs::write(bin.join("codex"), b"").expect("Codex host evidence must exist");
    fs::write(bin.join("claude"), b"").expect("Claude host evidence must exist");
    set_executable_file_mode(&bin.join("codex"));
    set_executable_file_mode(&bin.join("claude"));
    let codex_config = fixture.root.join("codex-config-private-canary");
    let claude_config = fixture.root.join("claude-config-private-canary");
    let output = fixture_command(Path::new(env!("CARGO_BIN_EXE_qiongli")), &fixture)
        .args(["install", "inventory"])
        .env("PATH", &bin)
        .env("CODEX_HOME", &codex_config)
        .env("CLAUDE_CONFIG_DIR", &claude_config)
        .output()
        .expect("configured native qiongli binary should report shared inventory");

    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty());
    let value = parse_json(&output);
    assert_eq!(value["command"], "install-inventory");
    assert_eq!(
        value["inventory"]["schema_version"],
        qiongli_platform::CLIENT_INVENTORY_SCHEMA_VERSION
    );
    let clients = value["inventory"]["clients"]
        .as_array()
        .expect("inventory clients must be an array");
    assert_eq!(clients.len(), 2);
    for client in clients {
        assert_eq!(client["discovery"], "detected");
        assert_eq!(client["host_presence"], "observed");
        assert_eq!(client["readiness"], "install-ready");
        assert!(
            client["paths"]
                .as_array()
                .expect("inventory paths must be an array")
                .iter()
                .any(|path| path["scope"] == "project")
        );
    }
    let public = public_output(&output);
    assert!(public.contains("codex-config-override"));
    assert!(public.contains("claude-config-override"));
    assert!(!public.contains(fixture.root.to_string_lossy().as_ref()));
    assert!(!codex_config.exists());
    assert!(!claude_config.exists());
    assert!(!fixture.home.join(".qiongli").exists());
    assert!(!fixture.home.join(".agents").exists());
}

#[cfg(unix)]
fn set_executable_file_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("host evidence must be executable");
}

#[cfg(not(unix))]
fn set_executable_file_mode(_path: &Path) {}

#[test]
fn invalid_explicit_invocations_and_environment_fail_without_echoing_private_values() {
    let cases: &[(&[&str], Option<&str>)] = &[
        (
            &["ui", "extra-private-canary"],
            Some("extra-private-canary"),
        ),
        (&["content"], None),
        (&["content", "help"], None),
        (
            &["content", "list", "extra-private-canary"],
            Some("extra-private-canary"),
        ),
        (&["content", "materialize", "--profile", "full"], None),
        (&["config"], None),
        (&["config", "-h"], None),
        (&["install"], None),
        (
            &["install", "status", "extra-private-canary"],
            Some("extra-private-canary"),
        ),
        (
            &["install", "claude", "extra-private-canary"],
            Some("extra-private-canary"),
        ),
        (
            &[
                "install",
                "native",
                "preview",
                "--private-key",
                "native-private-key-canary",
            ],
            Some("native-private-key-canary"),
        ),
        (
            &["config", "show", "extra-private-canary"],
            Some("extra-private-canary"),
        ),
        (
            &[
                "config",
                "set",
                "--expected-revision",
                "revision-private-canary",
            ],
            Some("revision-private-canary"),
        ),
        (
            &["--version", "trailing-private-canary"],
            Some("trailing-private-canary"),
        ),
        (&["command-private-canary"], Some("command-private-canary")),
    ];

    for &(args, canary) in cases {
        let output = run(args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("error:"));
        assert!(stderr.contains("Usage:"));
        if let Some(canary) = canary {
            assert!(!stderr.contains(canary));
        }
    }

    let fixture = Fixture::new("environment-error-private-canary");
    let output = fixture_command(Path::new(env!("CARGO_BIN_EXE_qiongli")), &fixture)
        .arg("status")
        .env("QIONGLI_CONFIG_HOME", "relative-environment-private-canary")
        .output()
        .expect("native status command should start");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"error: invalid-config-home\n");
    assert!(!public_output(&output).contains("relative-environment-private-canary"));

    let output = Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .arg("status")
        .env_remove("QIONGLI_CONFIG_HOME")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .env_remove("HOMEDRIVE")
        .env_remove("HOMEPATH")
        .output()
        .expect("native status command should start without a home");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"error: home-unavailable\n");
}

#[test]
fn copied_binary_lists_content_and_retires_direct_materialization_without_source_lookup() {
    let fixture = Fixture::new("copied-runtime-private-canary");
    let source = PathBuf::from(env!("CARGO_BIN_EXE_qiongli"));
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-copied-runtime-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&runtime_root).expect("outside-checkout runtime root must be created");
    set_private_directory_mode(&runtime_root);
    let copied = runtime_root.join(
        source
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source, &copied).expect("native executable must copy outside the checkout");

    let mut list_command = fixture_command(&copied, &fixture);
    list_command
        .current_dir(&runtime_root)
        .args(["content", "list"])
        .env("PATH", "");
    let list = output_with_executable_busy_retry(&mut list_command)
        .expect("copied executable must list embedded content outside the checkout");
    assert!(list.status.success(), "{}", public_output(&list));
    assert_eq!(parse_json(&list)["command"], "content-list");

    let mut install_status_command = fixture_command(&copied, &fixture);
    install_status_command
        .current_dir(&runtime_root)
        .args(["install", "status"])
        .env("PATH", "")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .env_remove("HOMEDRIVE")
        .env_remove("HOMEPATH");
    let install_status = output_with_executable_busy_retry(&mut install_status_command)
        .expect("copied executable must report install status without a runtime");
    assert!(
        install_status.status.success(),
        "{}",
        public_output(&install_status)
    );
    let install_value = parse_json(&install_status);
    assert_eq!(install_value["command"], "install-status");
    assert_eq!(
        install_value["transaction_engine"],
        "grant-and-approval-gated"
    );
    assert_eq!(install_value["launch_grant"], "unavailable");
    assert_eq!(install_value["apply"], "unavailable");

    let target = fixture.root.join("copied-binary-materialized");
    let mut materialize_command = fixture_command(&copied, &fixture);
    materialize_command
        .current_dir(&runtime_root)
        .args([
            OsString::from("content"),
            OsString::from("materialize"),
            OsString::from("--profile"),
            OsString::from("lite"),
            OsString::from("--target"),
            target.clone().into_os_string(),
        ])
        .env("PATH", "");
    let materialize = output_with_executable_busy_retry(&mut materialize_command)
        .expect("copied executable must reject retired materialization outside the checkout");
    assert_eq!(materialize.status.code(), Some(1));
    assert!(materialize.stdout.is_empty());
    assert_eq!(materialize.stderr, b"error: managed-skills-plan-required\n");
    assert!(!target.exists());
    assert!(!output_contains_path(&materialize, &target));
    fs::remove_dir_all(runtime_root).expect("outside-checkout runtime root must be removed");
}

#[cfg(not(feature = "desktop"))]
#[test]
fn cli_only_empty_args_print_help_and_ui_fails_without_a_window() {
    let help = run(&["--help"]);
    let empty = run(&[]);
    assert!(empty.status.success());
    assert_eq!(empty.stdout, help.stdout);
    assert!(empty.stderr.is_empty());
    let ui = run(&["ui"]);
    assert!(!ui.status.success());
    assert!(ui.stdout.is_empty());
    assert_eq!(
        String::from_utf8(ui.stderr).unwrap(),
        format!("error: {}\n", qiongli::DESKTOP_STARTUP_ERROR_CODE)
    );
}

#[test]
fn update_recovery_requires_explicit_digest_and_approval_without_creating_state() {
    let fixture = Fixture::new("update-recovery-contract");
    let digest = "a".repeat(64);
    let help = run_configured(&fixture, &["update", "--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("recovery-preview"));
    for args in [
        vec!["update", "recover"],
        vec!["update", "recover", "--approve-filesystem-write"],
        vec!["update", "recover", "--expected-marker-digest", &digest],
        vec![
            "update",
            "recover",
            "--expected-marker-digest",
            "bad",
            "--approve-filesystem-write",
        ],
        vec![
            "update",
            "recover",
            "--expected-marker-digest",
            &digest,
            "--approve-filesystem-write",
            "--approve-filesystem-write",
        ],
        vec![
            "update",
            "recover",
            "--expected-marker-digest",
            &digest,
            "--expected-marker-digest",
            &digest,
            "--approve-filesystem-write",
        ],
        vec![
            "update",
            "recover",
            "--expected-marker-digest",
            &digest,
            "--approve-filesystem-write",
            "--unknown",
        ],
        vec!["update", "recovery-preview", "--approve-filesystem-write"],
    ] {
        assert_eq!(
            run_configured(&fixture, &args).status.code(),
            Some(2),
            "{args:?}"
        );
    }
    for args in [
        vec!["update", "recovery-preview"],
        vec![
            "update",
            "recover",
            "--expected-marker-digest",
            &digest,
            "--approve-filesystem-write",
        ],
        vec![
            "update",
            "recover",
            "--approve-filesystem-write",
            "--expected-marker-digest",
            &digest,
        ],
    ] {
        let output = run_configured(&fixture, &args);
        assert!(!output.status.success());
        assert_ne!(output.status.code(), Some(2), "valid syntax: {args:?}");
        assert!(!fixture.config_root.exists());
        assert!(!fixture.home.join(".qiongli").exists());
    }
}

#[test]
fn native_activation_public_entry_refuses_source_authority_without_writes() {
    let fixture = Fixture::new("native-activation-source");
    let help = run_configured(&fixture, &["install", "--help"]);
    assert!(String::from_utf8_lossy(&help.stdout).contains("candidate activate --candidate"));
    let output = run_configured(
        &fixture,
        &[
            "install",
            "candidate",
            "activate",
            "--candidate",
            "missing.json",
            "--archive",
            "missing.zip",
            "--release-notes",
            "missing.md",
            "--target",
            "codex",
            "--previous-install-id",
            &format!("native-payload-{}", "1".repeat(64)),
            "--transaction-id",
            &format!("update-{}", "2".repeat(32)),
            "--expected-journal-digest",
            &"3".repeat(64),
            "--expected-approval-digest",
            &"4".repeat(64),
            "--approve-filesystem-write",
            "--approve-client-config-change",
            "--approve-host-trust",
        ],
    );
    assert!(!output.status.success());
    assert_ne!(output.status.code(), Some(2));
    let reason = String::from_utf8_lossy(&output.stderr);
    assert!(
        reason.contains(if cfg!(any(target_os = "macos", target_os = "linux")) {
            "native-release-authority-unavailable"
        } else {
            "native-update-target-unsupported"
        }),
        "{reason}"
    );
    assert!(!fixture.config_root.exists());
    assert!(!fixture.home.join(".qiongli").exists());
}
