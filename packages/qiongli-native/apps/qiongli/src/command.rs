use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use qiongli_config::{
    ConfigError, ConfigRoot, GlobalSettingsStore, LegacyProviderId, LegacyProviderResolution,
    LegacyProviderResolutionStrategy, RedactedConfigStatus, SecretStoreStatus, UpdateStateStore,
    UpdateStreamPreference, resolve_config_root,
};
use qiongli_content::{EmbeddedContent, ProfileId, ProfileProjection};
use qiongli_execution::BackendControlService;
use qiongli_platform::{
    ARTIFACT_IDENTITY_SCHEMA_VERSION, Architecture, CLAUDE_ADAPTER_SCHEMA_VERSION,
    CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION, CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
    CODEX_ADAPTER_SCHEMA_VERSION, CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION,
    CODEX_REGISTRATION_STATE_SCHEMA_VERSION, ClaudeDiscoverySummaryV1, ClientActivationTarget,
    ClientInventory, ClientInventoryInput, ClientInventorySummaryV1, CodexDiscoverySummaryV1,
    INSTALL_PLAN_SCHEMA_VERSION, INSTALL_RECEIPT_SCHEMA_VERSION, LAUNCH_GRANT_SCHEMA_VERSION,
    LocalTargetFamily, NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION,
    NATIVE_RELEASE_AUTHORITY_SCHEMA_VERSION, NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION,
    NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION, NativeReleaseAuthority, OperatingSystem,
    discover_claude_user_with_config, discover_client_inventory, discover_codex_user,
};
use qiongli_project::{AcademicGraphEntityKind, ProjectId};
use serde::Serialize;

use crate::candidate_cli::{CandidateCliCommand, CandidateReceiptOptions, CandidateReleaseOptions};
use crate::legacy_migration_cli::{LegacyMigrationCliCommand, LegacyMigrationContinueAction};
use crate::managed_operation::{
    ManagedIntegrationTargetV1, ManagedOperationCliCommand, ManagedSkillsPresetV1,
};
use crate::native_cli::{
    NativeCliCommand, NativeClientTarget, NativeReceiptOptions, NativeReleaseOptions,
};
use crate::update_cli::UpdateCliCommand;

const OUTPUT_SCHEMA_VERSION: u32 = 1;
const MAX_CLIENT_METADATA_BYTES: u64 = 256 * 1_024;

const USAGE: &str = "Qiongli native platform\n\nUsage:\n  qiongli\n  qiongli --version\n  qiongli --help\n  qiongli ui [--startup-check]\n  qiongli app <snapshot|verify-integrations|verify-skills|plan|apply>\n  qiongli project <list|show|doctor|create|register|migrate|import|export|archive|restore|refresh|unregister>\n  qiongli content list\n  qiongli config show\n  qiongli config set --expected-revision <revision> --default-profile <profile>\n  qiongli config backend status\n  qiongli update status\n  qiongli update channel --expected-revision <revision> --stream <stable|beta>\n  qiongli update check\n  qiongli update download --expected-revision <revision>\n  qiongli update verify --expected-revision <revision>\n  qiongli update stage --expected-revision <revision>\n  qiongli update install --expected-revision <revision>\n  qiongli update cancel --expected-revision <revision>\n  qiongli install status\n  qiongli install inventory\n  qiongli install codex status\n  qiongli install claude status\n  qiongli migrate-1x <inspect|preview|apply|continue|status|recover> [options]\n  qiongli mcp serve --profile <lite|marketplace-lite|full> --transport stdio\n  qiongli status\n  qiongli doctor\n\nProfiles:\n  skill-only | marketplace-lite | lite | full\n\nOptions:\n  -h, --help  Print help\n  --version   Print the native product version\n";

const INSPECTION_USAGE: &str = "\nInspection:\n  qiongli paths             Show exact resolved paths\n  qiongli paths --json      Show the versioned exact-path JSON snapshot\n  qiongli doctor            Run redacted native Product Doctor checks\n  qiongli doctor --paths exact\n                            Include the exact-path snapshot explicitly\n";

const APP_USAGE: &str = "Qiongli App control contract\n\nUsage:\n  qiongli app snapshot\n  qiongli app read-project-artifact --project-id <prj_id> --expected-project-revision <revision> --expected-projection-id <grp_id> <--node-id <nod_id>|--edge-id <edg_id>>\n  qiongli app verify-integrations --target <codex|claude|all>\n  qiongli app verify-skills --preset <qiongli-managed|current-project>\n  qiongli app verify-skills --target-id <skills-target-sha256>\n  qiongli app plan cli-install\n  qiongli app plan cli-remove\n  qiongli app plan cli-path-configure\n  qiongli app plan skills-reconcile --preset <qiongli-managed|current-project> --profile <profile>\n  qiongli app plan skills-update --target-id <skills-target-sha256>\n  qiongli app plan skills-remove --target-id <skills-target-sha256>\n  qiongli app plan skills-detach --target-id <skills-target-sha256>\n  qiongli app plan integrations-install --target <codex|claude|all>\n  qiongli app plan integrations-reconcile --target <codex|claude|all>\n  qiongli app plan integrations-remove --target <codex|claude|all>\n  qiongli app apply --plan <absolute-plan.json> --expected-plan-digest <sha256> --approve-filesystem-write [--approve-client-config-change --approve-host-trust]\n  qiongli app --help\n\nRead-only commands use the same native DesktopService and versioned App event contract as the GUI. Project artifact reads are revision-, projection-, and entity-bound and return only a bounded, path-redacted App event. CLI install, PATH configuration, remove or predecessor restoration, and integration repair are separate state-bound plans. Drifted Skills can be detached without changing their retained files. All mutations use a canonical, expiring, digest-bound plan and the same receipt-bound native transaction authority as the App.\n";

const CONTENT_USAGE: &str = "Qiongli embedded content (read only)\n\nUsage:\n  qiongli content list\n  qiongli content --help\n\nManaged Skills mutations use `qiongli app plan skills-reconcile|skills-update|skills-remove|skills-detach` followed by `qiongli app apply`. Choose a new custom destination in the Desktop App so its absolute path remains inside the native service. The retired `content materialize` syntax returns `managed-skills-plan-required` without writing.\n";

const CONFIG_USAGE: &str = "Qiongli global config\n\nUsage:\n  qiongli config show\n  qiongli config set --expected-revision <revision> --default-profile <profile>\n  qiongli config backend status\n  qiongli config --help\n\nModel execution is owned by Codex, Claude Code, or another supported host. Direct backend configuration and connection tests are not available in the default product.\n";

const UPDATE_USAGE: &str = "Qiongli native update\n\nUsage:\n  qiongli update status\n  qiongli update channel --expected-revision <revision> --stream <stable|beta>\n  qiongli update check\n  qiongli update download --expected-revision <revision>\n  qiongli update verify --expected-revision <revision>\n  qiongli update stage --expected-revision <revision>\n  qiongli update install --expected-revision <revision>\n  qiongli update cancel --expected-revision <revision>\n  qiongli update --help\n";

const MCP_USAGE: &str = "Qiongli native MCP\n\nUsage:\n  qiongli mcp serve --profile <lite|marketplace-lite|full> --transport stdio\n  qiongli mcp --help\n\nFull profile adds redacted Research Library, capture, academic graph, and local checkpoint controls. The connected host owns model execution and returns revision-bound candidates through the host handoff contract.\n";

const INSTALL_USAGE: &str = "Qiongli native payload inspection and release engineering\n\nUsage:\n\nRead-only observation:\n  qiongli install status\n  qiongli install inventory\n  qiongli install codex status\n  qiongli install claude status\n\nRelease-engineering payload commands:\n  qiongli install candidate activate-discard --transaction-id <update-id> --expected-journal-digest <sha256> --approve-filesystem-write\n  qiongli install candidate activate-prepare --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude> --previous-install-id <native-payload-id> --expected-preflight-digest <preflight-sha256> --approve-filesystem-write\n  qiongli install candidate activate-preview --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude> --previous-install-id <native-payload-id>\n  qiongli install candidate stage-preview --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude>\n  qiongli install candidate stage --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude> --expected-approval-digest <sha256> --approve-filesystem-write\n  qiongli install candidate preview --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude>\n  qiongli install candidate apply --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude> --expected-approval-digest <sha256> --approve-filesystem-write --approve-client-config-change --approve-host-trust\n  qiongli install candidate verify --target <codex|claude> --install-id <native-payload-id>\n  qiongli install candidate remove --target <codex|claude> --install-id <native-payload-id> --approve-filesystem-write --approve-client-config-change\n  qiongli install native preview --release <release.json> --archive <archive> --managed-root <absolute-path> --target <codex|claude>\n  qiongli install native apply --release <release.json> --archive <archive> --managed-root <absolute-path> --target <codex|claude> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli install native verify --managed-root <absolute-path> --install-id <native-payload-id>\n  qiongli install native remove --managed-root <absolute-path> --install-id <native-payload-id> --approve-filesystem-write\n  qiongli install --help\n\nCandidate activate-preview only checks installed identities; its preflight digest does not authorize activation.\n\nNormal Qiongli CLI, Plugin, and standalone Skills lifecycle uses `qiongli app plan` followed by `qiongli app apply`. The candidate/native commands above are retained for signed payload release engineering and are not a second end-user integration installer.\n";

const MIGRATION_USAGE: &str = "Qiongli 1.x replacement migration\n\nUsage:\n  qiongli migrate-1x inspect\n  qiongli migrate-1x preview [--provider-resolution <provider>=<keep-v2|use-legacy|merge-compatible>]...\n  qiongli migrate-1x apply --migration-id <id> --expected-plan-digest <sha256> --approve-filesystem-write [--approve-client-config-change] [--approve-secret-store-write]\n  qiongli migrate-1x continue --migration-id <id> --confirm-host-activation\n  qiongli migrate-1x continue --migration-id <id> --approve-cleanup\n  qiongli migrate-1x continue --migration-id <id> --finalize\n  qiongli migrate-1x status --migration-id <id>\n  qiongli migrate-1x recover --migration-id <id>\n  qiongli migrate-1x --help\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DetectedClientVersion {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
}

#[derive(Clone, Default)]
pub struct CommandEnvironment {
    configured_root: Option<OsString>,
    platform_home: Option<PathBuf>,
    codex_config_root: Option<PathBuf>,
    claude_config_root: Option<PathBuf>,
    project_root: Option<PathBuf>,
    zotero_connector_url: Option<String>,
    codex_host_present: bool,
    claude_host_present: bool,
    codex_host_version: Option<DetectedClientVersion>,
    claude_host_version: Option<DetectedClientVersion>,
    #[cfg(test)]
    client_discovery_disabled: bool,
}

impl CommandEnvironment {
    #[must_use]
    pub fn from_process() -> Self {
        let platform_home = process_platform_home();
        let (codex_host_present, codex_host_version) =
            discover_client_host("codex", platform_home.as_deref(), true);
        let (claude_host_present, claude_host_version) =
            discover_client_host("claude", platform_home.as_deref(), false);
        Self {
            configured_root: env::var_os("QIONGLI_CONFIG_HOME"),
            codex_host_present,
            claude_host_present,
            codex_host_version,
            claude_host_version,
            #[cfg(test)]
            client_discovery_disabled: false,
            platform_home,
            codex_config_root: nonempty_environment_path("CODEX_HOME"),
            claude_config_root: nonempty_environment_path("CLAUDE_CONFIG_DIR"),
            project_root: env::current_dir().ok(),
            zotero_connector_url: env::var("QIONGLI_ZOTERO_CONNECTOR_URL")
                .ok()
                .filter(|value| !value.is_empty()),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_paths(
        configured_root: Option<OsString>,
        platform_home: Option<PathBuf>,
        claude_config_root: Option<PathBuf>,
    ) -> Self {
        Self {
            configured_root,
            platform_home,
            codex_config_root: None,
            claude_config_root,
            project_root: None,
            zotero_connector_url: None,
            codex_host_present: false,
            claude_host_present: false,
            codex_host_version: None,
            claude_host_version: None,
            client_discovery_disabled: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_inventory_context(
        mut self,
        codex_config_root: Option<PathBuf>,
        project_root: Option<PathBuf>,
        codex_host_present: bool,
        claude_host_present: bool,
    ) -> Self {
        self.codex_config_root = codex_config_root;
        self.project_root = project_root;
        self.codex_host_present = codex_host_present;
        self.claude_host_present = claude_host_present;
        self
    }

    #[cfg(test)]
    pub(crate) fn with_client_versions(
        mut self,
        codex: Option<DetectedClientVersion>,
        claude: Option<DetectedClientVersion>,
    ) -> Self {
        self.codex_host_version = codex;
        self.claude_host_version = claude;
        self
    }

    #[cfg(test)]
    pub(crate) fn without_client_discovery(mut self) -> Self {
        self.client_discovery_disabled = true;
        self
    }

    pub(crate) fn platform_home(&self) -> Option<&Path> {
        self.platform_home.as_deref()
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn configured_root(&self) -> Option<&OsStr> {
        self.configured_root.as_deref()
    }

    pub(crate) fn claude_config_root(&self) -> Option<&Path> {
        self.claude_config_root.as_deref()
    }

    pub(crate) fn codex_config_root(&self) -> Option<&Path> {
        self.codex_config_root.as_deref()
    }

    pub(crate) fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    pub(crate) fn zotero_connector_url(&self) -> Option<&str> {
        self.zotero_connector_url.as_deref()
    }

    pub(crate) fn without_project_context(mut self) -> Self {
        self.project_root = None;
        self
    }

    pub(crate) const fn codex_host_version(&self) -> Option<DetectedClientVersion> {
        self.codex_host_version
    }

    pub(crate) const fn claude_host_version(&self) -> Option<DetectedClientVersion> {
        self.claude_host_version
    }

    pub(crate) fn detect_client_versions(&mut self) {
        #[cfg(test)]
        if self.client_discovery_disabled {
            self.codex_host_present = false;
            self.claude_host_present = false;
            self.codex_host_version = None;
            self.claude_host_version = None;
            return;
        }
        let (codex_present, codex_version) =
            discover_client_host("codex", self.platform_home.as_deref(), true);
        let (claude_present, claude_version) =
            discover_client_host("claude", self.platform_home.as_deref(), false);
        self.codex_host_present = codex_present;
        self.claude_host_present = claude_present;
        self.codex_host_version = codex_version;
        self.claude_host_version = claude_version;
    }

    pub(crate) fn client_executable(&self, name: &str) -> Option<PathBuf> {
        find_client_executable(name, self.platform_home.as_deref())
    }

    pub(crate) fn client_inventory(&self) -> Option<ClientInventory> {
        let home = self.platform_home()?;
        Some(discover_client_inventory(
            ClientInventoryInput::new(home)
                .with_codex_config_root(self.codex_config_root.as_deref())
                .with_claude_config_root(self.claude_config_root.as_deref())
                .with_project_root(self.project_root.as_deref())
                .with_host_presence(self.codex_host_present, self.claude_host_present),
        ))
    }
}

pub struct CliOutput {
    exit_code: u8,
    stdout: String,
    stderr: String,
}

pub enum ProductAction {
    Output(CliOutput),
    ServeLiteMcpStdio,
    ServeFullMcpStdio,
    LaunchDesktop,
    LaunchDesktopWithCandidate(Box<crate::DesktopCandidateSession>),
}

impl CliOutput {
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        self.exit_code
    }

    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    pub(crate) fn success_text(stdout: impl Into<String>) -> Self {
        Self {
            exit_code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    pub(crate) fn operation_failure(reason_code: &'static str) -> Self {
        Self {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("error: {reason_code}\n"),
        }
    }

    fn usage_failure(error: UsageError) -> Self {
        Self {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("error: {}\n\n{}", error.message, error.usage),
        }
    }
}

#[must_use]
pub fn failed_embedded_content_output() -> CliOutput {
    CliOutput::operation_failure("embedded-content-integrity-failed")
}

pub fn run_cli(
    args: impl IntoIterator<Item = OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> CliOutput {
    match prepare_action(args, environment, content) {
        ProductAction::Output(output) => output,
        ProductAction::ServeLiteMcpStdio => {
            CliOutput::operation_failure("streaming-command-requires-product-entrypoint")
        }
        ProductAction::ServeFullMcpStdio => {
            CliOutput::operation_failure("streaming-command-requires-product-entrypoint")
        }
        ProductAction::LaunchDesktop => {
            CliOutput::operation_failure("desktop-command-requires-product-entrypoint")
        }
        ProductAction::LaunchDesktopWithCandidate(_) => {
            CliOutput::operation_failure("desktop-command-requires-product-entrypoint")
        }
    }
}

pub fn prepare_action(
    args: impl IntoIterator<Item = OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> ProductAction {
    let authority = match crate::embedded_release_authority() {
        Ok(authority) => authority,
        Err(_) => {
            return ProductAction::Output(CliOutput::operation_failure(
                "native-release-authority-invalid",
            ));
        }
    };
    prepare_action_with_release_authority(args, environment, content, authority.as_ref())
}

pub(crate) fn prepare_action_with_release_authority(
    args: impl IntoIterator<Item = OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    authority: Option<&NativeReleaseAuthority>,
) -> ProductAction {
    let command = match parse_args(args) {
        Ok(command) => command,
        Err(error) => return ProductAction::Output(CliOutput::usage_failure(error)),
    };

    let output = match command {
        Command::Help => CliOutput::success_text(format!("{USAGE}{INSPECTION_USAGE}")),
        Command::Version => {
            CliOutput::success_text(format!("qiongli {}\n", env!("CARGO_PKG_VERSION")))
        }
        Command::Ui => return ProductAction::LaunchDesktop,
        Command::UiCandidate(options) => {
            let authority = match authority {
                Some(authority) => authority,
                None => {
                    return ProductAction::Output(CliOutput::operation_failure(
                        "native-release-authority-unavailable",
                    ));
                }
            };
            let source_commit = match crate::embedded_source_commit() {
                Some(source_commit) => source_commit,
                None => {
                    return ProductAction::Output(CliOutput::operation_failure(
                        "native-source-commit-unavailable",
                    ));
                }
            };
            let now_unix = match crate::candidate_cli::now_unix() {
                Ok(now_unix) => now_unix,
                Err(reason_code) => {
                    return ProductAction::Output(CliOutput::operation_failure(reason_code));
                }
            };
            let prepared = match crate::candidate_cli::prepare_candidate(
                &options,
                authority,
                source_commit,
                content,
                now_unix,
            ) {
                Ok(prepared) => prepared,
                Err(reason_code) => {
                    return ProductAction::Output(CliOutput::operation_failure(reason_code));
                }
            };
            return ProductAction::LaunchDesktopWithCandidate(Box::new(
                crate::DesktopCandidateSession::new(prepared.into_verified()),
            ));
        }
        Command::UiStartupCheck => ui_startup_check(environment, content),
        Command::AppHelp => CliOutput::success_text(APP_USAGE),
        Command::AppSnapshot => match crate::desktop::app_snapshot_json(environment, content) {
            Ok(snapshot) => CliOutput::success_text(snapshot),
            Err(reason_code) => CliOutput::operation_failure(reason_code),
        },
        Command::AppReadProjectArtifact {
            project_id,
            expected_project_revision,
            expected_projection_id,
            entity_kind,
            entity_id,
        } => match crate::desktop::app_read_project_artifact_json(
            environment,
            content,
            &project_id,
            expected_project_revision,
            &expected_projection_id,
            entity_kind,
            &entity_id,
        ) {
            Ok(event) => CliOutput::success_text(event),
            Err(reason_code) => CliOutput::operation_failure(reason_code),
        },
        Command::AppVerifyIntegrations { target } => {
            let (codex, claude_code) = match target {
                AppVerificationTarget::Codex => (true, false),
                AppVerificationTarget::Claude => (false, true),
                AppVerificationTarget::All => (true, true),
            };
            match crate::desktop::app_verify_integrations_json(
                environment,
                content,
                codex,
                claude_code,
            ) {
                Ok(event) => CliOutput::success_text(event),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::AppVerifySkills { preset } => {
            match crate::desktop::app_verify_skills_json(
                environment,
                content,
                matches!(preset, AppSkillsVerificationPreset::QiongliManaged),
            ) {
                Ok(event) => CliOutput::success_text(event),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::AppVerifyManagedSkillsTarget { target_id } => {
            match crate::desktop::app_verify_managed_skills_target_json(
                environment,
                content,
                target_id,
            ) {
                Ok(event) => CliOutput::success_text(event),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::AppManaged(command) => {
            match crate::managed_operation::execute(&command, environment, content) {
                Ok(output) => CliOutput::success_text(output),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::Project(command) => crate::project_cli::execute(command, environment),
        Command::ContentHelp => CliOutput::success_text(CONTENT_USAGE),
        Command::ContentList => content_list(content),
        Command::ContentMaterialize { .. } => {
            CliOutput::operation_failure("managed-skills-plan-required")
        }
        Command::ConfigHelp => CliOutput::success_text(CONFIG_USAGE),
        Command::ConfigShow => config_show(environment),
        Command::ConfigSet {
            expected_revision,
            default_profile,
        } => config_set(environment, expected_revision, default_profile),
        Command::ConfigBackendStatus => config_backend_status(environment),
        Command::UpdateHelp => CliOutput::success_text(UPDATE_USAGE),
        Command::Update(command) => {
            let store = match update_store(environment) {
                Ok(store) => store,
                Err(error) => {
                    return ProductAction::Output(CliOutput::operation_failure(
                        error.reason_code(),
                    ));
                }
            };
            match crate::update_cli::execute(
                command,
                &store,
                authority,
                crate::embedded_macos_team_id(),
                environment,
                content,
            ) {
                Ok(output) => json_output(&output, 0),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::InstallHelp => CliOutput::success_text(INSTALL_USAGE),
        Command::InstallStatus => install_status(authority),
        Command::InstallInventory => install_inventory(environment),
        Command::InstallCodexStatus => install_codex_status(environment),
        Command::InstallClaudeStatus => install_claude_status(environment),
        Command::InstallCandidate(command) => {
            match crate::candidate_cli::execute(
                command,
                authority,
                crate::embedded_source_commit(),
                environment,
                content,
            ) {
                Ok(output) => json_output(&output, 0),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::InstallNative(command) => {
            match crate::native_cli::execute(command, authority, content, environment) {
                Ok(output) => json_output(&output, 0),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::MigrationHelp => CliOutput::success_text(MIGRATION_USAGE),
        Command::Migrate1x(command) => {
            match crate::legacy_migration_cli::execute(command, environment, content) {
                Ok(output) => json_output(&output, 0),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::McpHelp => CliOutput::success_text(MCP_USAGE),
        Command::McpServeLiteStdio => return ProductAction::ServeLiteMcpStdio,
        Command::McpServeFullStdio => return ProductAction::ServeFullMcpStdio,
        Command::Status => status(environment, content),
        Command::Paths { json } => paths(environment, content, json),
        Command::Doctor { exact_paths } => doctor(environment, content, exact_paths),
    };
    ProductAction::Output(output)
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Help,
    Version,
    Ui,
    UiCandidate(CandidateReleaseOptions),
    UiStartupCheck,
    AppHelp,
    AppSnapshot,
    AppReadProjectArtifact {
        project_id: ProjectId,
        expected_project_revision: u64,
        expected_projection_id: String,
        entity_kind: AcademicGraphEntityKind,
        entity_id: String,
    },
    AppVerifyIntegrations {
        target: AppVerificationTarget,
    },
    AppVerifySkills {
        preset: AppSkillsVerificationPreset,
    },
    AppVerifyManagedSkillsTarget {
        target_id: String,
    },
    AppManaged(ManagedOperationCliCommand),
    Project(crate::project_cli::ProjectCliCommand),
    ContentHelp,
    ContentList,
    ContentMaterialize {
        profile: ProfileId,
        target: PathBuf,
    },
    ConfigHelp,
    ConfigShow,
    ConfigSet {
        expected_revision: u64,
        default_profile: ProfileId,
    },
    ConfigBackendStatus,
    UpdateHelp,
    Update(UpdateCliCommand),
    InstallHelp,
    InstallStatus,
    InstallInventory,
    InstallCodexStatus,
    InstallClaudeStatus,
    InstallCandidate(CandidateCliCommand),
    InstallNative(NativeCliCommand),
    MigrationHelp,
    Migrate1x(LegacyMigrationCliCommand),
    McpHelp,
    McpServeLiteStdio,
    McpServeFullStdio,
    Status,
    Paths {
        json: bool,
    },
    Doctor {
        exact_paths: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppVerificationTarget {
    Codex,
    Claude,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppSkillsVerificationPreset {
    QiongliManaged,
    CurrentProject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UsageError {
    message: &'static str,
    usage: &'static str,
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Command, UsageError> {
    let args = args.into_iter().collect::<Vec<_>>();
    if args.is_empty() {
        return Ok(if cfg!(feature = "desktop") {
            Command::Ui
        } else {
            Command::Help
        });
    }
    let Some(command) = args.first().and_then(|value| value.to_str()) else {
        return Err(global_usage_error("command or option is not valid text"));
    };

    match command {
        "-h" | "--help" if args.len() == 1 => Ok(Command::Help),
        "--version" if args.len() == 1 => Ok(Command::Version),
        "content" => parse_content_args(&args[1..]),
        "config" => parse_config_args(&args[1..]),
        "update" => parse_update_args(&args[1..]),
        "install" => parse_install_args(&args[1..]),
        "migrate-1x" => parse_migration_args(&args[1..]),
        "mcp" => parse_mcp_args(&args[1..]),
        "project" => crate::project_cli::parse(&args[1..])
            .map(Command::Project)
            .map_err(project_usage_error),
        "app" => parse_app_args(&args[1..]),
        "ui" if args.len() == 1 => Ok(Command::Ui),
        "ui" if args.get(1).and_then(|value| value.to_str()) == Some("--startup-check")
            && args.len() == 2 =>
        {
            Ok(Command::UiStartupCheck)
        }
        "ui" if args.len() > 1 => parse_candidate_release_options(&args[1..], false)
            .map(|parsed| Command::UiCandidate(parsed.options)),
        "status" if args.len() == 1 => Ok(Command::Status),
        "paths" if args.len() == 1 => Ok(Command::Paths { json: false }),
        "paths" if args.len() == 2 && args[1] == OsStr::new("--json") => {
            Ok(Command::Paths { json: true })
        }
        "doctor" if args.len() == 1 => Ok(Command::Doctor { exact_paths: false }),
        "doctor"
            if args.len() == 3
                && args[1] == OsStr::new("--paths")
                && args[2] == OsStr::new("exact") =>
        {
            Ok(Command::Doctor { exact_paths: true })
        }
        "-h" | "--help" | "--version" | "ui" | "status" | "paths" | "doctor" => {
            Err(global_usage_error("unexpected extra argument"))
        }
        _ => Err(global_usage_error("unknown command or option")),
    }
}

fn parse_app_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(app_usage_error("an App subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::AppHelp),
        "snapshot" if args.len() == 1 => Ok(Command::AppSnapshot),
        "read-project-artifact" => parse_app_project_artifact_args(&args[1..]),
        "verify-integrations" if args.len() == 3 && args[1] == OsStr::new("--target") => {
            let target = match args[2].to_str() {
                Some("codex") => AppVerificationTarget::Codex,
                Some("claude") => AppVerificationTarget::Claude,
                Some("all") => AppVerificationTarget::All,
                _ => return Err(app_usage_error("App integration target is invalid")),
            };
            Ok(Command::AppVerifyIntegrations { target })
        }
        "verify-skills" if args.len() == 3 && args[1] == OsStr::new("--preset") => {
            let preset = match args[2].to_str() {
                Some("qiongli-managed") => AppSkillsVerificationPreset::QiongliManaged,
                Some("current-project") => AppSkillsVerificationPreset::CurrentProject,
                _ => return Err(app_usage_error("App Skills preset is invalid")),
            };
            Ok(Command::AppVerifySkills { preset })
        }
        "verify-skills" if args.len() == 3 && args[1] == OsStr::new("--target-id") => {
            let target_id = args[2]
                .to_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| app_usage_error("App managed Skills target is invalid"))?
                .to_string();
            Ok(Command::AppVerifyManagedSkillsTarget { target_id })
        }
        "plan" => parse_app_plan_args(&args[1..]).map(Command::AppManaged),
        "apply" => parse_app_apply_args(&args[1..]).map(Command::AppManaged),
        "--help" | "snapshot" | "verify-integrations" | "verify-skills" => {
            Err(app_usage_error("unexpected App argument"))
        }
        _ => Err(app_usage_error("unknown App subcommand")),
    }
}

fn parse_app_project_artifact_args(args: &[OsString]) -> Result<Command, UsageError> {
    if args.len() != 8 {
        return Err(app_usage_error(
            "project artifact inspection requires four exact option-value pairs",
        ));
    }
    let mut project_id = None;
    let mut expected_project_revision = None;
    let mut expected_projection_id = None;
    let mut entity = None;
    for pair in args.chunks_exact(2) {
        let option = pair[0]
            .to_str()
            .ok_or_else(|| app_usage_error("project artifact option is not valid UTF-8"))?;
        let value = pair[1]
            .to_str()
            .ok_or_else(|| app_usage_error("project artifact value is not valid UTF-8"))?;
        match option {
            "--project-id" if project_id.is_none() => {
                project_id = Some(
                    ProjectId::parse(value.to_owned())
                        .map_err(|_| app_usage_error("project artifact project ID is invalid"))?,
                );
            }
            "--expected-project-revision" if expected_project_revision.is_none() => {
                expected_project_revision = Some(
                    parse_revision(&pair[1])
                        .filter(|revision| *revision > 0)
                        .ok_or_else(|| {
                            app_usage_error("project artifact project revision is invalid")
                        })?,
                );
            }
            "--expected-projection-id" if expected_projection_id.is_none() => {
                if !valid_prefixed_digest(value, "grp_") {
                    return Err(app_usage_error("project artifact projection ID is invalid"));
                }
                expected_projection_id = Some(value.to_owned());
            }
            "--node-id" if entity.is_none() => {
                if !valid_prefixed_digest(value, "nod_") {
                    return Err(app_usage_error("project artifact node ID is invalid"));
                }
                entity = Some((AcademicGraphEntityKind::Node, value.to_owned()));
            }
            "--edge-id" if entity.is_none() => {
                if !valid_prefixed_digest(value, "edg_") {
                    return Err(app_usage_error("project artifact edge ID is invalid"));
                }
                entity = Some((AcademicGraphEntityKind::Edge, value.to_owned()));
            }
            "--project-id"
            | "--expected-project-revision"
            | "--expected-projection-id"
            | "--node-id"
            | "--edge-id" => {
                return Err(app_usage_error(
                    "project artifact option is duplicate or conflicts",
                ));
            }
            _ => return Err(app_usage_error("unknown project artifact option")),
        }
    }
    let (entity_kind, entity_id) =
        entity.ok_or_else(|| app_usage_error("one project artifact entity is required"))?;
    Ok(Command::AppReadProjectArtifact {
        project_id: project_id
            .ok_or_else(|| app_usage_error("project artifact project ID is required"))?,
        expected_project_revision: expected_project_revision
            .ok_or_else(|| app_usage_error("project artifact project revision is required"))?,
        expected_projection_id: expected_projection_id
            .ok_or_else(|| app_usage_error("project artifact projection ID is required"))?,
        entity_kind,
        entity_id,
    })
}

fn valid_prefixed_digest(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}

fn parse_app_plan_args(args: &[OsString]) -> Result<ManagedOperationCliCommand, UsageError> {
    let Some(operation) = args.first().and_then(|value| value.to_str()) else {
        return Err(app_usage_error("an App plan operation is required"));
    };
    match operation {
        "cli-install" if args.len() == 1 => Ok(ManagedOperationCliCommand::PlanCliInstall),
        "cli-install" => Err(app_usage_error("unexpected App CLI install argument")),
        "cli-remove" if args.len() == 1 => Ok(ManagedOperationCliCommand::PlanCliRemove),
        "cli-remove" => Err(app_usage_error("unexpected App CLI remove argument")),
        "cli-path-configure" if args.len() == 1 => {
            Ok(ManagedOperationCliCommand::PlanCliPathConfigure)
        }
        "cli-path-configure" => Err(app_usage_error("unexpected App CLI PATH argument")),
        "skills-reconcile" => {
            let mut preset = None;
            let mut profile = None;
            let mut index = 1;
            while index < args.len() {
                let option = args[index]
                    .to_str()
                    .ok_or_else(|| app_usage_error("App plan option is invalid"))?;
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| app_usage_error("App plan option value is missing"))?;
                match option {
                    "--preset" if preset.is_none() => {
                        preset = Some(parse_managed_skills_preset(value)?);
                    }
                    "--profile" if profile.is_none() => {
                        profile = parse_profile(value);
                        if profile.is_none() {
                            return Err(app_usage_error("App Skills profile is invalid"));
                        }
                    }
                    "--preset" | "--profile" => {
                        return Err(app_usage_error("App plan option is duplicated"));
                    }
                    _ => return Err(app_usage_error("App plan option is unexpected")),
                }
                index += 2;
            }
            Ok(ManagedOperationCliCommand::PlanSkillsReconcile {
                preset: preset.ok_or_else(|| app_usage_error("App Skills preset is required"))?,
                profile: profile
                    .ok_or_else(|| app_usage_error("App Skills profile is required"))?,
            })
        }
        "skills-update" | "skills-remove" | "skills-detach" => {
            if args.len() != 3 || args[1] != OsStr::new("--target-id") {
                return Err(app_usage_error("App managed Skills target is required"));
            }
            let target_id = args[2]
                .to_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| app_usage_error("App managed Skills target is invalid"))?
                .to_string();
            match operation {
                "skills-update" => Ok(ManagedOperationCliCommand::PlanSkillsUpdate { target_id }),
                "skills-remove" => Ok(ManagedOperationCliCommand::PlanSkillsRemove { target_id }),
                _ => Ok(ManagedOperationCliCommand::PlanSkillsDetach { target_id }),
            }
        }
        "integrations-install" | "integrations-reconcile" | "integrations-remove" => {
            if args.len() != 3 || args[1] != OsStr::new("--target") {
                return Err(app_usage_error("App integration target is required"));
            }
            let targets = match args[2].to_str() {
                Some("codex") => vec![ManagedIntegrationTargetV1::Codex],
                Some("claude") => vec![ManagedIntegrationTargetV1::ClaudeCode],
                Some("all") => vec![
                    ManagedIntegrationTargetV1::Codex,
                    ManagedIntegrationTargetV1::ClaudeCode,
                ],
                _ => return Err(app_usage_error("App integration target is invalid")),
            };
            match operation {
                "integrations-install" => {
                    Ok(ManagedOperationCliCommand::PlanIntegrationsInstall { targets })
                }
                "integrations-reconcile" => {
                    Ok(ManagedOperationCliCommand::PlanIntegrationsReconcile { targets })
                }
                _ => Ok(ManagedOperationCliCommand::PlanIntegrationsRemove { targets }),
            }
        }
        _ => Err(app_usage_error("unknown App plan operation")),
    }
}

fn parse_app_apply_args(args: &[OsString]) -> Result<ManagedOperationCliCommand, UsageError> {
    let mut plan_path = None;
    let mut expected_plan_digest = None;
    let mut approve_filesystem_write = false;
    let mut approve_client_config_change = false;
    let mut approve_host_trust = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| app_usage_error("App apply option is invalid"))?;
        match option {
            "--plan" if plan_path.is_none() => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| app_usage_error("App plan path is missing"))?;
                plan_path = Some(PathBuf::from(value));
                index += 2;
            }
            "--expected-plan-digest" if expected_plan_digest.is_none() => {
                let value = args
                    .get(index + 1)
                    .and_then(|value| value.to_str())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| app_usage_error("App plan digest is invalid"))?;
                expected_plan_digest = Some(value.to_string());
                index += 2;
            }
            "--approve-filesystem-write" if !approve_filesystem_write => {
                approve_filesystem_write = true;
                index += 1;
            }
            "--approve-client-config-change" if !approve_client_config_change => {
                approve_client_config_change = true;
                index += 1;
            }
            "--approve-host-trust" if !approve_host_trust => {
                approve_host_trust = true;
                index += 1;
            }
            "--plan"
            | "--expected-plan-digest"
            | "--approve-filesystem-write"
            | "--approve-client-config-change"
            | "--approve-host-trust" => {
                return Err(app_usage_error("App apply option is duplicated"));
            }
            _ => return Err(app_usage_error("App apply option is unexpected")),
        }
    }
    Ok(ManagedOperationCliCommand::Apply {
        plan_path: plan_path.ok_or_else(|| app_usage_error("App plan path is required"))?,
        expected_plan_digest: expected_plan_digest
            .ok_or_else(|| app_usage_error("App plan digest is required"))?,
        approve_filesystem_write,
        approve_client_config_change,
        approve_host_trust,
    })
}

fn parse_managed_skills_preset(value: &OsStr) -> Result<ManagedSkillsPresetV1, UsageError> {
    match value.to_str() {
        Some("qiongli-managed") => Ok(ManagedSkillsPresetV1::QiongliManaged),
        Some("current-project") => Ok(ManagedSkillsPresetV1::CurrentProject),
        _ => Err(app_usage_error("App Skills preset is invalid")),
    }
}

fn parse_install_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(install_usage_error("an install subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::InstallHelp),
        "status" if args.len() == 1 => Ok(Command::InstallStatus),
        "inventory" if args.len() == 1 => Ok(Command::InstallInventory),
        "codex"
            if args.get(1).and_then(|value| value.to_str()) == Some("status")
                && args.len() == 2 =>
        {
            Ok(Command::InstallCodexStatus)
        }
        "claude"
            if args.get(1).and_then(|value| value.to_str()) == Some("status")
                && args.len() == 2 =>
        {
            Ok(Command::InstallClaudeStatus)
        }
        "candidate"
            if args.get(1).and_then(|value| value.to_str()) == Some("--help")
                && args.len() == 2 =>
        {
            Ok(Command::InstallHelp)
        }
        "candidate" => parse_candidate_install_args(&args[1..]).map(Command::InstallCandidate),
        "native"
            if args.get(1).and_then(|value| value.to_str()) == Some("--help")
                && args.len() == 2 =>
        {
            Ok(Command::InstallHelp)
        }
        "native" => parse_native_install_args(&args[1..]).map(Command::InstallNative),
        "--help" | "status" | "inventory" | "codex" | "claude" => {
            Err(install_usage_error("unexpected extra argument"))
        }
        _ => Err(install_usage_error("unknown install subcommand")),
    }
}

fn parse_migration_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(migration_usage_error("a migration subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::MigrationHelp),
        "inspect" if args.len() == 1 => Ok(Command::Migrate1x(LegacyMigrationCliCommand::Inspect)),
        "preview" => parse_migration_preview_options(&args[1..]).map(Command::Migrate1x),
        "apply" => parse_migration_apply_options(&args[1..]).map(Command::Migrate1x),
        "continue" => parse_migration_continue_options(&args[1..]).map(Command::Migrate1x),
        "status" => parse_migration_id_option(&args[1..]).map(|migration_id| {
            Command::Migrate1x(LegacyMigrationCliCommand::Status { migration_id })
        }),
        "recover" => parse_migration_id_option(&args[1..]).map(|migration_id| {
            Command::Migrate1x(LegacyMigrationCliCommand::Recover { migration_id })
        }),
        "--help" | "inspect" => Err(migration_usage_error("unexpected migration argument")),
        _ => Err(migration_usage_error("unknown migration subcommand")),
    }
}

fn parse_migration_preview_options(
    args: &[OsString],
) -> Result<LegacyMigrationCliCommand, UsageError> {
    let mut resolutions = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] != OsStr::new("--provider-resolution") {
            return Err(migration_usage_error(
                "migration preview option is unexpected",
            ));
        }
        let value = args
            .get(index + 1)
            .and_then(|value| value.to_str())
            .ok_or_else(|| migration_usage_error("provider resolution is invalid"))?;
        let (provider, strategy) = value
            .split_once('=')
            .ok_or_else(|| migration_usage_error("provider resolution is invalid"))?;
        if strategy.contains('=') {
            return Err(migration_usage_error("provider resolution is invalid"));
        }
        let provider = match provider {
            "openalex" => LegacyProviderId::OpenAlex,
            "semantic-scholar" => LegacyProviderId::SemanticScholar,
            "crossref" => LegacyProviderId::Crossref,
            "pubmed" => LegacyProviderId::Pubmed,
            "arxiv" => LegacyProviderId::Arxiv,
            _ => return Err(migration_usage_error("provider resolution is invalid")),
        };
        let strategy = match strategy {
            "keep-v2" => LegacyProviderResolutionStrategy::KeepV2,
            "use-legacy" => LegacyProviderResolutionStrategy::UseLegacy,
            "merge-compatible" => LegacyProviderResolutionStrategy::MergeCompatible,
            _ => return Err(migration_usage_error("provider resolution is invalid")),
        };
        if resolutions
            .iter()
            .any(|resolution: &LegacyProviderResolution| resolution.provider == provider)
        {
            return Err(migration_usage_error("provider resolution is duplicate"));
        }
        resolutions.push(LegacyProviderResolution { provider, strategy });
        if resolutions.len() > 5 {
            return Err(migration_usage_error("too many provider resolutions"));
        }
        index += 2;
    }
    resolutions.sort_unstable_by_key(|resolution| resolution.provider);
    Ok(LegacyMigrationCliCommand::Preview {
        provider_resolutions: resolutions,
    })
}

fn parse_migration_apply_options(
    args: &[OsString],
) -> Result<LegacyMigrationCliCommand, UsageError> {
    let mut migration_id = None;
    let mut expected_plan_digest = None;
    let mut filesystem_write = false;
    let mut client_config_change = false;
    let mut secret_store_write = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| migration_usage_error("migration option is not valid UTF-8"))?;
        match option {
            "--approve-filesystem-write" if !filesystem_write => {
                filesystem_write = true;
                index += 1;
            }
            "--approve-client-config-change" if !client_config_change => {
                client_config_change = true;
                index += 1;
            }
            "--approve-secret-store-write" if !secret_store_write => {
                secret_store_write = true;
                index += 1;
            }
            "--migration-id" if migration_id.is_none() => {
                let value = args
                    .get(index + 1)
                    .and_then(|value| parse_migration_id(value))
                    .ok_or_else(|| migration_usage_error("migration ID is invalid"))?;
                migration_id = Some(value);
                index += 2;
            }
            "--expected-plan-digest" if expected_plan_digest.is_none() => {
                let value = args
                    .get(index + 1)
                    .and_then(|value| parse_sha256(value))
                    .ok_or_else(|| migration_usage_error("migration plan digest is invalid"))?;
                expected_plan_digest = Some(value);
                index += 2;
            }
            _ => {
                return Err(migration_usage_error(
                    "migration apply option is unexpected or duplicate",
                ));
            }
        }
    }
    Ok(LegacyMigrationCliCommand::Apply {
        migration_id: migration_id
            .ok_or_else(|| migration_usage_error("migration ID is required"))?,
        expected_plan_digest: expected_plan_digest
            .ok_or_else(|| migration_usage_error("migration plan digest is required"))?,
        approve_filesystem_write: filesystem_write,
        approve_client_config_change: client_config_change,
        approve_secret_store_write: secret_store_write,
    })
}

fn parse_migration_continue_options(
    args: &[OsString],
) -> Result<LegacyMigrationCliCommand, UsageError> {
    let mut migration_id = None;
    let mut action = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| migration_usage_error("migration option is not valid UTF-8"))?;
        match option {
            "--migration-id" if migration_id.is_none() => {
                migration_id = args
                    .get(index + 1)
                    .and_then(|value| parse_migration_id(value));
                if migration_id.is_none() {
                    return Err(migration_usage_error("migration ID is invalid"));
                }
                index += 2;
            }
            "--confirm-host-activation" if action.is_none() => {
                action = Some(LegacyMigrationContinueAction::ConfirmHostActivation);
                index += 1;
            }
            "--approve-cleanup" if action.is_none() => {
                action = Some(LegacyMigrationContinueAction::Cleanup);
                index += 1;
            }
            "--finalize" if action.is_none() => {
                action = Some(LegacyMigrationContinueAction::Finalize);
                index += 1;
            }
            _ => {
                return Err(migration_usage_error(
                    "migration continue option is unexpected or duplicate",
                ));
            }
        }
    }
    Ok(LegacyMigrationCliCommand::Continue {
        migration_id: migration_id
            .ok_or_else(|| migration_usage_error("migration ID is required"))?,
        action: action
            .ok_or_else(|| migration_usage_error("migration continue action is required"))?,
    })
}

fn parse_migration_id_option(args: &[OsString]) -> Result<String, UsageError> {
    if args.len() != 2 || args[0] != OsStr::new("--migration-id") {
        return Err(migration_usage_error(
            "exactly one --migration-id option is required",
        ));
    }
    parse_migration_id(&args[1]).ok_or_else(|| migration_usage_error("migration ID is invalid"))
}

fn parse_candidate_install_args(args: &[OsString]) -> Result<CandidateCliCommand, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(install_usage_error(
            "a candidate install subcommand is required",
        ));
    };
    match subcommand {
        "activate-discard" => {
            let mut transaction_id = None;
            let mut digest = None;
            let mut approved = false;
            let mut args = args[1..].iter();
            while let Some(option) = args.next() {
                match option.to_str() {
                    Some("--approve-filesystem-write") if !approved => approved = true,
                    Some("--transaction-id") if transaction_id.is_none() => {
                        transaction_id = Some(
                            args.next()
                                .and_then(|value| value.to_str())
                                .filter(|value| {
                                    crate::update_reconcile::validate_transaction_id(value).is_ok()
                                })
                                .ok_or_else(|| {
                                    install_usage_error("activation transaction id is invalid")
                                })?
                                .to_string(),
                        );
                    }
                    Some("--expected-journal-digest") if digest.is_none() => {
                        digest = Some(
                            args.next()
                                .and_then(|value| parse_sha256(value))
                                .ok_or_else(|| {
                                    install_usage_error("activation journal digest is invalid")
                                })?,
                        );
                    }
                    _ => {
                        return Err(install_usage_error(
                            "unexpected or duplicate activation discard option",
                        ));
                    }
                }
            }
            if !approved {
                return Err(install_usage_error("filesystem-write approval is required"));
            }
            Ok(CandidateCliCommand::ActivateDiscard {
                transaction_id: transaction_id
                    .ok_or_else(|| install_usage_error("activation transaction id is required"))?,
                expected_journal_sha256: digest
                    .ok_or_else(|| install_usage_error("activation journal digest is required"))?,
            })
        }
        "activate-preview" | "activate-prepare" => {
            let prepare = subcommand == "activate-prepare";
            let mut release_args = Vec::new();
            let mut predecessor = None;
            let mut remaining = &args[1..];
            while !remaining.is_empty() {
                let width = if remaining[0] == "--approve-filesystem-write" {
                    1
                } else {
                    2.min(remaining.len())
                };
                let (pair, rest) = remaining.split_at(width);
                remaining = rest;
                if pair[0] == "--previous-install-id" {
                    if predecessor.is_some() || pair.len() != 2 {
                        return Err(install_usage_error(
                            "previous install id is missing or duplicate",
                        ));
                    }
                    predecessor = pair[1]
                        .to_str()
                        .filter(|value| {
                            value.strip_prefix("native-payload-").is_some_and(|digest| {
                                digest.len() == 64
                                    && digest.bytes().all(|byte| {
                                        byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
                                    })
                            })
                        })
                        .map(str::to_string);
                    if predecessor.is_none() {
                        return Err(install_usage_error("previous install id is invalid"));
                    }
                } else if prepare && pair[0] == "--expected-preflight-digest" {
                    release_args.push("--expected-approval-digest".into());
                    release_args.extend_from_slice(&pair[1..]);
                } else if pair[0] == "--expected-approval-digest" {
                    return Err(install_usage_error(
                        "activation preparation requires a preflight digest",
                    ));
                } else {
                    release_args.extend_from_slice(pair);
                }
            }
            let parsed = parse_candidate_release_options_inner(&release_args, prepare, true)?;
            let previous_install_id = predecessor
                .ok_or_else(|| install_usage_error("previous install id is required"))?;
            if prepare {
                Ok(CandidateCliCommand::ActivatePrepare {
                    options: parsed.options,
                    previous_install_id,
                    expected_preflight_digest: parsed.expected_approval_digest.ok_or_else(
                        || install_usage_error("expected preflight digest is required"),
                    )?,
                })
            } else {
                Ok(CandidateCliCommand::ActivatePreview {
                    options: parsed.options,
                    previous_install_id,
                })
            }
        }
        "stage-preview" => parse_candidate_release_options_inner(&args[1..], false, true)
            .map(|parsed| CandidateCliCommand::StagePreview(parsed.options)),
        "stage" => {
            parse_candidate_release_options_inner(&args[1..], true, true).and_then(|parsed| {
                Ok(CandidateCliCommand::Stage {
                    options: parsed.options,
                    expected_approval_digest: parsed.expected_approval_digest.ok_or_else(|| {
                        install_usage_error("expected candidate staging digest is required")
                    })?,
                })
            })
        }
        "preview" => parse_candidate_release_options(&args[1..], false)
            .map(|parsed| CandidateCliCommand::Preview(parsed.options)),
        "apply" => parse_candidate_release_options(&args[1..], true).and_then(|parsed| {
            Ok(CandidateCliCommand::Apply {
                options: parsed.options,
                expected_approval_digest: parsed.expected_approval_digest.ok_or_else(|| {
                    install_usage_error("expected candidate approval digest is required")
                })?,
            })
        }),
        "verify" => {
            parse_candidate_receipt_options(&args[1..], false).map(CandidateCliCommand::Verify)
        }
        "remove" => {
            parse_candidate_receipt_options(&args[1..], true).map(CandidateCliCommand::Remove)
        }
        "--help" if args.len() == 1 => Err(install_usage_error(
            "use qiongli install --help for candidate install options",
        )),
        "--help" => Err(install_usage_error("unexpected candidate install argument")),
        _ => Err(install_usage_error("unknown candidate install subcommand")),
    }
}

struct ParsedCandidateReleaseOptions {
    options: CandidateReleaseOptions,
    expected_approval_digest: Option<String>,
}

fn parse_candidate_release_options(
    args: &[OsString],
    apply: bool,
) -> Result<ParsedCandidateReleaseOptions, UsageError> {
    parse_candidate_release_options_inner(args, apply, false)
}

fn parse_candidate_release_options_inner(
    args: &[OsString],
    apply: bool,
    stage: bool,
) -> Result<ParsedCandidateReleaseOptions, UsageError> {
    let mut candidate = None;
    let mut archive = None;
    let mut release_notes = None;
    let mut target = None;
    let mut expected_approval_digest = None;
    let mut filesystem_approved = false;
    let mut config_approved = false;
    let mut host_trust_approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| install_usage_error("candidate install option is not valid UTF-8"))?;
        let approval = match option {
            "--approve-filesystem-write" => Some(&mut filesystem_approved),
            "--approve-client-config-change" => Some(&mut config_approved),
            "--approve-host-trust" => Some(&mut host_trust_approved),
            _ => None,
        };
        if let Some(approved) = approval {
            if !apply || *approved {
                return Err(install_usage_error(
                    "candidate install approval is unexpected or duplicate",
                ));
            }
            *approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| install_usage_error("candidate install option value is required"))?;
        match option {
            "--candidate" if candidate.is_none() => candidate = nonempty_path(value),
            "--archive" if archive.is_none() => archive = nonempty_path(value),
            "--release-notes" if release_notes.is_none() => release_notes = nonempty_path(value),
            "--target" if target.is_none() => {
                target =
                    Some(parse_candidate_target(value).ok_or_else(|| {
                        install_usage_error("candidate install target is invalid")
                    })?);
            }
            "--expected-approval-digest" if apply && expected_approval_digest.is_none() => {
                expected_approval_digest =
                    Some(parse_sha256(value).ok_or_else(|| {
                        install_usage_error("candidate approval digest is invalid")
                    })?);
            }
            "--candidate"
            | "--archive"
            | "--release-notes"
            | "--target"
            | "--expected-approval-digest" => {
                return Err(install_usage_error(
                    "candidate install option is unexpected or duplicate",
                ));
            }
            _ => return Err(install_usage_error("unknown candidate install option")),
        }
        if matches!(option, "--candidate" | "--archive" | "--release-notes") && value.is_empty() {
            return Err(install_usage_error("candidate install path is empty"));
        }
        index += 2;
    }
    if stage && (config_approved || host_trust_approved) {
        return Err(install_usage_error(
            "candidate staging only accepts filesystem-write approval",
        ));
    }
    if apply && !(filesystem_approved && (stage || (config_approved && host_trust_approved))) {
        return Err(install_usage_error(
            "all candidate install approvals are required",
        ));
    }
    Ok(ParsedCandidateReleaseOptions {
        options: CandidateReleaseOptions {
            candidate: candidate
                .ok_or_else(|| install_usage_error("release candidate path is required"))?,
            archive: archive
                .ok_or_else(|| install_usage_error("candidate archive path is required"))?,
            release_notes: release_notes
                .ok_or_else(|| install_usage_error("release notes path is required"))?,
            target: target
                .ok_or_else(|| install_usage_error("candidate install target is required"))?,
        },
        expected_approval_digest,
    })
}

fn parse_candidate_receipt_options(
    args: &[OsString],
    remove: bool,
) -> Result<CandidateReceiptOptions, UsageError> {
    let mut target = None;
    let mut install_id = None;
    let mut filesystem_approved = false;
    let mut config_approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| install_usage_error("candidate install option is not valid UTF-8"))?;
        let approval = match option {
            "--approve-filesystem-write" => Some(&mut filesystem_approved),
            "--approve-client-config-change" => Some(&mut config_approved),
            _ => None,
        };
        if let Some(approved) = approval {
            if !remove || *approved {
                return Err(install_usage_error(
                    "candidate remove approval is unexpected or duplicate",
                ));
            }
            *approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| install_usage_error("candidate install option value is required"))?;
        match option {
            "--target" if target.is_none() => {
                target =
                    Some(parse_candidate_target(value).ok_or_else(|| {
                        install_usage_error("candidate install target is invalid")
                    })?);
            }
            "--install-id" if install_id.is_none() => {
                install_id = Some(
                    parse_native_install_id(value)
                        .ok_or_else(|| install_usage_error("candidate install ID is invalid"))?,
                );
            }
            "--target" | "--install-id" => {
                return Err(install_usage_error(
                    "candidate install option is unexpected or duplicate",
                ));
            }
            _ => return Err(install_usage_error("unknown candidate install option")),
        }
        index += 2;
    }
    if remove && !(filesystem_approved && config_approved) {
        return Err(install_usage_error(
            "all candidate remove approvals are required",
        ));
    }
    Ok(CandidateReceiptOptions {
        target: target
            .ok_or_else(|| install_usage_error("candidate install target is required"))?,
        install_id: install_id
            .ok_or_else(|| install_usage_error("candidate install ID is required"))?,
    })
}

fn parse_native_install_args(args: &[OsString]) -> Result<NativeCliCommand, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(install_usage_error(
            "a native install subcommand is required",
        ));
    };
    match subcommand {
        "preview" => parse_native_release_options(&args[1..], false)
            .map(|parsed| NativeCliCommand::Preview(parsed.options)),
        "apply" => parse_native_release_options(&args[1..], true).and_then(|parsed| {
            Ok(NativeCliCommand::Apply {
                options: parsed.options,
                expected_plan_digest: parsed.expected_plan_digest.ok_or_else(|| {
                    install_usage_error("expected native install plan digest is required")
                })?,
            })
        }),
        "verify" => parse_native_receipt_options(&args[1..], false)
            .map(|parsed| NativeCliCommand::Verify(parsed.options)),
        "remove" => parse_native_receipt_options(&args[1..], true)
            .map(|parsed| NativeCliCommand::Remove(parsed.options)),
        "--help" if args.len() == 1 => Err(install_usage_error(
            "use qiongli install --help for native install options",
        )),
        "--help" => Err(install_usage_error("unexpected native install argument")),
        _ => Err(install_usage_error("unknown native install subcommand")),
    }
}

struct ParsedNativeReleaseOptions {
    options: NativeReleaseOptions,
    expected_plan_digest: Option<String>,
}

fn parse_native_release_options(
    args: &[OsString],
    apply: bool,
) -> Result<ParsedNativeReleaseOptions, UsageError> {
    let mut release = None;
    let mut archive = None;
    let mut managed_root = None;
    let mut target = None;
    let mut expected_plan_digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| install_usage_error("native install option is not valid UTF-8"))?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err(install_usage_error(
                    "native install approval option is unexpected or duplicate",
                ));
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| install_usage_error("native install option value is required"))?;
        match option {
            "--release" if release.is_none() => release = nonempty_path(value),
            "--archive" if archive.is_none() => archive = nonempty_path(value),
            "--managed-root" if managed_root.is_none() => managed_root = nonempty_path(value),
            "--target" if target.is_none() => {
                target = Some(
                    parse_native_client_target(value)
                        .ok_or_else(|| install_usage_error("native install target is invalid"))?,
                );
            }
            "--expected-plan-digest" if apply && expected_plan_digest.is_none() => {
                expected_plan_digest =
                    Some(parse_sha256(value).ok_or_else(|| {
                        install_usage_error("native install plan digest is invalid")
                    })?);
            }
            "--release"
            | "--archive"
            | "--managed-root"
            | "--target"
            | "--expected-plan-digest" => {
                return Err(install_usage_error(
                    "native install option is unexpected or duplicate",
                ));
            }
            _ => return Err(install_usage_error("unknown native install option")),
        }
        if matches!(option, "--release" | "--archive" | "--managed-root") && value.is_empty() {
            return Err(install_usage_error("native install path is empty"));
        }
        index += 2;
    }
    if apply && !approved {
        return Err(install_usage_error(
            "explicit filesystem-write approval is required",
        ));
    }
    Ok(ParsedNativeReleaseOptions {
        options: NativeReleaseOptions {
            release: release
                .ok_or_else(|| install_usage_error("native release path is required"))?,
            archive: archive
                .ok_or_else(|| install_usage_error("native archive path is required"))?,
            managed_root: managed_root
                .ok_or_else(|| install_usage_error("native managed root is required"))?,
            target: target
                .ok_or_else(|| install_usage_error("native install target is required"))?,
        },
        expected_plan_digest,
    })
}

struct ParsedNativeReceiptOptions {
    options: NativeReceiptOptions,
}

fn parse_native_receipt_options(
    args: &[OsString],
    remove: bool,
) -> Result<ParsedNativeReceiptOptions, UsageError> {
    let mut managed_root = None;
    let mut install_id = None;
    let mut approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| install_usage_error("native install option is not valid UTF-8"))?;
        if option == "--approve-filesystem-write" {
            if !remove || approved {
                return Err(install_usage_error(
                    "native install approval option is unexpected or duplicate",
                ));
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| install_usage_error("native install option value is required"))?;
        match option {
            "--managed-root" if managed_root.is_none() => managed_root = nonempty_path(value),
            "--install-id" if install_id.is_none() => {
                install_id = Some(
                    parse_native_install_id(value)
                        .ok_or_else(|| install_usage_error("native install ID is invalid"))?,
                );
            }
            "--managed-root" | "--install-id" => {
                return Err(install_usage_error(
                    "native install option is unexpected or duplicate",
                ));
            }
            _ => return Err(install_usage_error("unknown native install option")),
        }
        if option == "--managed-root" && value.is_empty() {
            return Err(install_usage_error("native install path is empty"));
        }
        index += 2;
    }
    if remove && !approved {
        return Err(install_usage_error(
            "explicit filesystem-write approval is required",
        ));
    }
    Ok(ParsedNativeReceiptOptions {
        options: NativeReceiptOptions {
            managed_root: managed_root
                .ok_or_else(|| install_usage_error("native managed root is required"))?,
            install_id: install_id
                .ok_or_else(|| install_usage_error("native install ID is required"))?,
        },
    })
}

fn nonempty_path(value: &OsStr) -> Option<PathBuf> {
    (!value.is_empty()).then(|| PathBuf::from(value))
}

fn parse_native_client_target(value: &OsStr) -> Option<NativeClientTarget> {
    match value.to_str()? {
        "codex" => Some(NativeClientTarget::Codex),
        "claude" => Some(NativeClientTarget::Claude),
        _ => None,
    }
}

fn parse_candidate_target(value: &OsStr) -> Option<ClientActivationTarget> {
    match value.to_str()? {
        "codex" => Some(ClientActivationTarget::Codex),
        "claude" => Some(ClientActivationTarget::ClaudeCode),
        _ => None,
    }
}

fn parse_sha256(value: &OsStr) -> Option<String> {
    let value = value.to_str()?;
    (value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then(|| value.to_string())
}

fn parse_native_install_id(value: &OsStr) -> Option<String> {
    let value = value.to_str()?;
    let digest = value.strip_prefix("native-payload-")?;
    (digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then(|| value.to_string())
}

fn parse_migration_id(value: &OsStr) -> Option<String> {
    let value = value.to_str()?;
    ((1..=64).contains(&value.len())
        && value.starts_with("migration-")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'))
    .then(|| value.to_owned())
}

fn parse_mcp_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(mcp_usage_error("an MCP subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::McpHelp),
        "serve"
            if args.get(1).and_then(|value| value.to_str()) == Some("--help")
                && args.len() == 2 =>
        {
            Ok(Command::McpHelp)
        }
        "serve" => parse_mcp_serve_options(&args[1..]),
        "--help" => Err(mcp_usage_error("unexpected extra argument")),
        _ => Err(mcp_usage_error("unknown MCP subcommand")),
    }
}

fn parse_mcp_serve_options(args: &[OsString]) -> Result<Command, UsageError> {
    let mut profile = None;
    let mut transport = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| mcp_usage_error("MCP option is not valid UTF-8"))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| mcp_usage_error("MCP option value is required"))?
            .to_str()
            .ok_or_else(|| mcp_usage_error("MCP option value is not valid UTF-8"))?;
        match option {
            "--profile" if profile.is_none() => {
                if !matches!(value, "lite" | "marketplace-lite" | "full") {
                    return Err(mcp_usage_error("MCP profile is unavailable"));
                }
                profile = Some(value);
            }
            "--transport" if transport.is_none() => {
                if value != "stdio" {
                    return Err(mcp_usage_error("MCP transport is unavailable"));
                }
                transport = Some(value);
            }
            "--profile" | "--transport" => {
                return Err(mcp_usage_error("duplicate MCP option"));
            }
            _ => return Err(mcp_usage_error("unknown MCP option")),
        }
        index += 2;
    }
    if profile.is_none() {
        return Err(mcp_usage_error("MCP profile is required"));
    }
    if transport.is_none() {
        return Err(mcp_usage_error("MCP transport is required"));
    }
    Ok(if profile == Some("full") {
        Command::McpServeFullStdio
    } else {
        Command::McpServeLiteStdio
    })
}

fn parse_content_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(content_usage_error("a content subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::ContentHelp),
        "list" if args.len() == 1 => Ok(Command::ContentList),
        "materialize" => parse_materialize_options(&args[1..]),
        "--help" | "list" => Err(content_usage_error("unexpected extra argument")),
        _ => Err(content_usage_error("unknown content subcommand")),
    }
}

fn parse_materialize_options(args: &[OsString]) -> Result<Command, UsageError> {
    let mut profile = None;
    let mut target = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| content_usage_error("content option is not valid UTF-8"))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| content_usage_error("content option value is required"))?;
        match option {
            "--profile" if profile.is_none() => {
                profile = Some(
                    parse_profile(value)
                        .ok_or_else(|| content_usage_error("content profile is invalid"))?,
                );
            }
            "--target" if target.is_none() => target = Some(PathBuf::from(value)),
            "--profile" | "--target" => {
                return Err(content_usage_error("duplicate content option"));
            }
            _ => return Err(content_usage_error("unknown content option")),
        }
        index += 2;
    }
    Ok(Command::ContentMaterialize {
        profile: profile.ok_or_else(|| content_usage_error("content profile is required"))?,
        target: target.ok_or_else(|| content_usage_error("content target is required"))?,
    })
}

fn parse_config_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(config_usage_error("a config subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::ConfigHelp),
        "show" if args.len() == 1 => Ok(Command::ConfigShow),
        "set" => parse_config_set_options(&args[1..]),
        "backend" => parse_config_backend_args(&args[1..]),
        "--help" | "show" => Err(config_usage_error("unexpected extra argument")),
        _ => Err(config_usage_error("unknown config subcommand")),
    }
}

fn parse_config_backend_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(config_usage_error("a backend subcommand is required"));
    };
    match subcommand {
        "status" if args.len() == 1 => Ok(Command::ConfigBackendStatus),
        "set" | "test" => Err(config_usage_error("host-driven execution required")),
        "status" => Err(config_usage_error("unexpected backend argument")),
        _ => Err(config_usage_error("unknown backend subcommand")),
    }
}

fn parse_config_set_options(args: &[OsString]) -> Result<Command, UsageError> {
    let mut expected_revision = None;
    let mut default_profile = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| config_usage_error("config option is not valid UTF-8"))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| config_usage_error("config option value is required"))?;
        match option {
            "--expected-revision" if expected_revision.is_none() => {
                expected_revision = Some(
                    parse_revision(value)
                        .ok_or_else(|| config_usage_error("expected revision is invalid"))?,
                );
            }
            "--default-profile" if default_profile.is_none() => {
                default_profile = Some(
                    parse_profile(value)
                        .ok_or_else(|| config_usage_error("default profile is invalid"))?,
                );
            }
            "--expected-revision" | "--default-profile" => {
                return Err(config_usage_error("duplicate config option"));
            }
            _ => return Err(config_usage_error("unknown config option")),
        }
        index += 2;
    }
    Ok(Command::ConfigSet {
        expected_revision: expected_revision
            .ok_or_else(|| config_usage_error("expected revision is required"))?,
        default_profile: default_profile
            .ok_or_else(|| config_usage_error("default profile is required"))?,
    })
}

fn parse_update_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(update_usage_error("an update subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::UpdateHelp),
        "status" if args.len() == 1 => Ok(Command::Update(UpdateCliCommand::Status)),
        "check" if args.len() == 1 => Ok(Command::Update(UpdateCliCommand::Check)),
        "channel" => parse_update_channel_options(&args[1..]),
        "download" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Download { expected_revision })
        }),
        "verify" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Verify { expected_revision })
        }),
        "stage" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Stage { expected_revision })
        }),
        "install" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Install { expected_revision })
        }),
        "reconcile" => parse_update_health_options(&args[1..]).map(|command| match command {
            Command::Update(UpdateCliCommand::Health { transaction_id }) => {
                Command::Update(UpdateCliCommand::Reconcile { transaction_id })
            }
            _ => unreachable!("health parser always returns an update health command"),
        }),
        "health" => parse_update_health_options(&args[1..]),
        "cancel" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Cancel { expected_revision })
        }),
        "--help" | "status" | "check" => Err(update_usage_error("unexpected extra argument")),
        _ => Err(update_usage_error("unknown update subcommand")),
    }
}

fn parse_update_health_options(args: &[OsString]) -> Result<Command, UsageError> {
    if args.len() != 2 || args.first().and_then(|value| value.to_str()) != Some("--transaction-id")
    {
        return Err(update_usage_error(
            "exactly one transaction id option is required",
        ));
    }
    let transaction_id = args[1]
        .to_str()
        .ok_or_else(|| update_usage_error("transaction id is invalid"))?;
    Ok(Command::Update(UpdateCliCommand::Health {
        transaction_id: transaction_id.to_string(),
    }))
}

fn parse_update_expected_revision(args: &[OsString]) -> Result<u64, UsageError> {
    if args.len() != 2
        || args.first().and_then(|value| value.to_str()) != Some("--expected-revision")
    {
        return Err(update_usage_error(
            "exactly one expected revision option is required",
        ));
    }
    parse_revision(&args[1]).ok_or_else(|| update_usage_error("expected revision is invalid"))
}

fn parse_update_channel_options(args: &[OsString]) -> Result<Command, UsageError> {
    let mut expected_revision = None;
    let mut stream = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| update_usage_error("update option is not valid UTF-8"))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| update_usage_error("update option value is required"))?;
        match option {
            "--expected-revision" if expected_revision.is_none() => {
                expected_revision = Some(
                    parse_revision(value)
                        .ok_or_else(|| update_usage_error("expected revision is invalid"))?,
                );
            }
            "--stream" if stream.is_none() => {
                stream = Some(match value.to_str() {
                    Some("stable") => UpdateStreamPreference::Stable,
                    Some("beta") => UpdateStreamPreference::Beta,
                    _ => return Err(update_usage_error("update stream is invalid")),
                });
            }
            "--expected-revision" | "--stream" => {
                return Err(update_usage_error("duplicate update option"));
            }
            _ => return Err(update_usage_error("unknown update option")),
        }
        index += 2;
    }
    Ok(Command::Update(UpdateCliCommand::Channel {
        expected_revision: expected_revision
            .ok_or_else(|| update_usage_error("expected revision is required"))?,
        stream: stream.ok_or_else(|| update_usage_error("update stream is required"))?,
    }))
}

fn parse_profile(value: &OsStr) -> Option<ProfileId> {
    match value.to_str()? {
        "skill-only" => Some(ProfileId::SkillOnly),
        "marketplace-lite" | "lite" => Some(ProfileId::MarketplaceLite),
        "full" => Some(ProfileId::Full),
        _ => None,
    }
}

fn parse_revision(value: &OsStr) -> Option<u64> {
    value
        .to_str()?
        .parse::<u64>()
        .ok()
        .filter(|revision| *revision <= qiongli_config::MAX_GLOBAL_SETTINGS_REVISION)
}

const fn global_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: USAGE,
    }
}

const fn app_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: APP_USAGE,
    }
}

const fn content_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: CONTENT_USAGE,
    }
}

const fn config_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: CONFIG_USAGE,
    }
}

const fn update_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: UPDATE_USAGE,
    }
}

const fn mcp_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: MCP_USAGE,
    }
}

const fn install_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: INSTALL_USAGE,
    }
}

const fn migration_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: MIGRATION_USAGE,
    }
}

const fn project_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: crate::project_cli::PROJECT_USAGE,
    }
}

fn content_list(content: &EmbeddedContent) -> CliOutput {
    let manifest = content.pack().manifest();
    json_output(
        &ContentListOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "content-list",
            pack_id: &manifest.pack_id,
            content_version: &manifest.content_version,
            source_commit: &manifest.source_commit,
            pack_sha256: content.pack().pack_sha256(),
            content_root_sha256: &manifest.content_root_sha256,
            profiles: content.profiles(),
        },
        0,
    )
}

fn ui_startup_check(environment: &CommandEnvironment, content: &EmbeddedContent) -> CliOutput {
    if crate::desktop::validate_desktop_startup(environment, content).is_err() {
        return CliOutput::operation_failure("desktop-startup-check-failed");
    }
    let (Some(os), Some(arch)) = (OperatingSystem::current(), Architecture::current()) else {
        return CliOutput::operation_failure("unsupported-build-target");
    };
    json_output(
        &UiStartupCheckOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "ui-startup-check",
            product_version: env!("CARGO_PKG_VERSION"),
            current_target: InstallBuildTarget { os, arch },
            service: "ready",
            snapshot: "ready",
            app_state: "ready",
            update_surface: "ready",
            window_entrypoint: "available",
            window: "not-opened",
        },
        0,
    )
}

fn config_show(environment: &CommandEnvironment) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &ConfigShowOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "config-show",
            config: native_config_status(&store),
        },
        0,
    )
}

fn config_set(
    environment: &CommandEnvironment,
    expected_revision: u64,
    default_profile: ProfileId,
) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let mut loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    loaded.settings.default_profile = default_profile;
    let outcome = match store.replace(expected_revision, loaded.settings) {
        Ok(outcome) => outcome,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &ConfigSetOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "config-set",
            revision: outcome.revision,
            default_profile,
            cleanup_required: outcome.cleanup_required,
        },
        0,
    )
}

fn config_backend_status(environment: &CommandEnvironment) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let secret_store = crate::credential_store::native_secret_store();
    let service = BackendControlService::from_global_settings(&loaded.settings, secret_store);
    json_output(
        &ConfigBackendStatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "config-backend-status",
            revision: loaded.revision,
            backend: service.openai_status(),
        },
        0,
    )
}

fn install_status(authority: Option<&NativeReleaseAuthority>) -> CliOutput {
    let (Some(os), Some(arch)) = (OperatingSystem::current(), Architecture::current()) else {
        return CliOutput::operation_failure("unsupported-build-target");
    };
    let candidate_ready = authority.is_some() && crate::embedded_source_commit().is_some();
    json_output(
        &InstallStatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-status",
            contracts: InstallContractVersions {
                artifact_identity: ARTIFACT_IDENTITY_SCHEMA_VERSION,
                launch_grant: LAUNCH_GRANT_SCHEMA_VERSION,
                release_authority: NATIVE_RELEASE_AUTHORITY_SCHEMA_VERSION,
                release_envelope: NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION,
                release_candidate: NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION,
                install_plan: INSTALL_PLAN_SCHEMA_VERSION,
                install_receipt: INSTALL_RECEIPT_SCHEMA_VERSION,
                native_payload_install_receipt: NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION,
                codex_adapter: CODEX_ADAPTER_SCHEMA_VERSION,
                codex_registration_receipt: CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION,
                codex_registration_state: CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
                claude_adapter: CLAUDE_ADAPTER_SCHEMA_VERSION,
                claude_registration_receipt: CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION,
                claude_registration_state: CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
            },
            current_target: InstallBuildTarget { os, arch },
            transaction_engine: "grant-and-approval-gated",
            release_authority: if authority.is_some() {
                "embedded"
            } else {
                "unavailable"
            },
            launch_grant: if authority.is_some() {
                "release-bound"
            } else {
                "unavailable"
            },
            source_commit: if crate::embedded_source_commit().is_some() {
                "embedded"
            } else {
                "unavailable"
            },
            candidate: if candidate_ready {
                "signed-current-target-required"
            } else {
                "unavailable"
            },
            preview: if candidate_ready {
                "signed-candidate-required"
            } else {
                "unavailable"
            },
            apply: if candidate_ready {
                "signed-candidate-and-approval-required"
            } else {
                "unavailable"
            },
            verify: "receipt-backed",
            remove: "receipt-backed-explicit-approval",
            targets: [
                InstallTargetStatus {
                    family: LocalTargetFamily::CodexLocal,
                    state: "adapter-engine-ready",
                },
                InstallTargetStatus {
                    family: LocalTargetFamily::ClaudeCodeLocal,
                    state: "adapter-engine-ready",
                },
            ],
        },
        0,
    )
}

fn install_codex_status(environment: &CommandEnvironment) -> CliOutput {
    let Some(home) = environment.platform_home.as_deref() else {
        return CliOutput::operation_failure("codex-home-unavailable");
    };
    let target = match discover_codex_user(home) {
        Ok(target) => target,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &InstallCodexStatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-codex-status",
            target: target.summary(),
            launch_grant: "unavailable",
            preview: "unavailable",
            apply: "unavailable",
            activation: "client-action-required",
        },
        0,
    )
}

fn install_inventory(environment: &CommandEnvironment) -> CliOutput {
    let Some(inventory) = environment.client_inventory() else {
        return CliOutput::operation_failure("client-inventory-home-unavailable");
    };
    json_output(
        &InstallInventoryOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-inventory",
            inventory: inventory.summary(),
        },
        0,
    )
}

fn install_claude_status(environment: &CommandEnvironment) -> CliOutput {
    let Some(home) = environment.platform_home.as_deref() else {
        return CliOutput::operation_failure("claude-home-unavailable");
    };
    let claude_config_root = environment
        .claude_config_root
        .clone()
        .unwrap_or_else(|| home.join(".claude"));
    let target = match discover_claude_user_with_config(home, &claude_config_root) {
        Ok(target) => target,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &InstallClaudeStatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-claude-status",
            target: target.summary(),
            launch_grant: "unavailable",
            preview: "unavailable",
            apply: "unavailable",
            activation: "reload-or-client-action-required",
        },
        0,
    )
}

fn status(environment: &CommandEnvironment, content: &EmbeddedContent) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &StatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "status",
            product_version: env!("CARGO_PKG_VERSION"),
            content: content_summary(content),
            config: native_config_status(&store),
        },
        0,
    )
}

fn paths(environment: &CommandEnvironment, content: &EmbeddedContent, json: bool) -> CliOutput {
    let secret_store = crate::credential_store::native_secret_store();
    let inspection =
        crate::product_diagnostics::inspect_product(environment, content, secret_store.status());
    if json {
        return json_output(
            &PathsOutput {
                schema_version: inspection.schema_version,
                command: "paths",
                product_version: inspection.product_version,
                paths: &inspection.paths,
            },
            0,
        );
    }
    let mut output = format!(
        "Qiongli {} resolved paths (explicit exact-path view)\n",
        inspection.product_version
    );
    for path in &inspection.paths {
        output.push_str(&format!(
            "\n{} [{}]\n  path: {}\n  symbolic: {}\n  scope: {:?} · source: {:?} · selected: {}\n  state: {:?} · expected: {} · matches: {:?} · owner: {:?} · writability: {:?} · safety: {:?}\n",
            path.label,
            path.id,
            path.exact_path,
            path.symbolic_path,
            path.scope,
            path.source,
            path.selected,
            path.file_type,
            path.expected_type,
            path.type_matches_expected,
            path.owner,
            path.writability,
            path.safety,
        ));
        if let Some(target) = path.resolved_target.as_deref() {
            output.push_str(&format!("  resolved-target: {target}\n"));
        }
    }
    CliOutput::success_text(output)
}

fn doctor(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    exact_paths: bool,
) -> CliOutput {
    let secret_store = crate::credential_store::native_secret_store();
    let inspection =
        crate::product_diagnostics::inspect_product(environment, content, secret_store.status());
    let blocking = inspection.blocking();
    let attention = inspection.requires_attention();
    json_output(
        &DoctorOutput {
            schema_version: inspection.schema_version,
            command: "doctor",
            overall: if attention { "attention" } else { "ready" },
            checks: &inspection.checks,
            paths: exact_paths.then_some(inspection.paths.as_slice()),
        },
        u8::from(blocking),
    )
}

fn native_config_status(store: &GlobalSettingsStore) -> RedactedConfigStatus {
    let mut status = store.status();
    let secret_store = crate::credential_store::native_secret_store();
    if secret_store.status() == SecretStoreStatus::Available {
        status.secret_store = "ready";
        if status.remediation_code == "secure-store-not-implemented" {
            status.remediation_code = "none";
        }
    }
    status
}

pub(crate) fn config_store(
    environment: &CommandEnvironment,
) -> Result<GlobalSettingsStore, ConfigError> {
    Ok(GlobalSettingsStore::new(config_root(environment)?))
}

pub(crate) fn config_root(environment: &CommandEnvironment) -> Result<ConfigRoot, ConfigError> {
    let home = environment
        .platform_home
        .as_deref()
        .ok_or(ConfigError::HomeUnavailable)?;
    resolve_config_root(environment.configured_root.as_deref(), home)
}

fn update_store(environment: &CommandEnvironment) -> Result<UpdateStateStore, ConfigError> {
    Ok(UpdateStateStore::new(
        config_root(environment)?,
        default_update_stream(),
    ))
}

fn default_update_stream() -> UpdateStreamPreference {
    let version = env!("CARGO_PKG_VERSION");
    if version.contains("-alpha.") || version.contains("-beta.") {
        UpdateStreamPreference::Beta
    } else {
        UpdateStreamPreference::Stable
    }
}

fn content_summary(content: &EmbeddedContent) -> ContentSummary<'_> {
    let manifest = content.pack().manifest();
    ContentSummary {
        state: "ready",
        pack_id: &manifest.pack_id,
        content_version: &manifest.content_version,
        pack_sha256: content.pack().pack_sha256(),
        content_root_sha256: &manifest.content_root_sha256,
        profiles: content.profiles(),
    }
}

fn json_output(value: &impl Serialize, exit_code: u8) -> CliOutput {
    match serde_json::to_string_pretty(value) {
        Ok(mut stdout) => {
            stdout.push('\n');
            CliOutput {
                exit_code,
                stdout,
                stderr: String::new(),
            }
        }
        Err(_) => CliOutput::operation_failure("output-serialization-failed"),
    }
}

#[derive(Serialize)]
struct ContentListOutput<'a> {
    schema_version: u32,
    command: &'static str,
    pack_id: &'a str,
    content_version: &'a str,
    source_commit: &'a str,
    pack_sha256: &'a str,
    content_root_sha256: &'a str,
    profiles: &'a [ProfileProjection],
}

#[derive(Serialize)]
struct UiStartupCheckOutput {
    schema_version: u32,
    command: &'static str,
    product_version: &'static str,
    current_target: InstallBuildTarget,
    service: &'static str,
    snapshot: &'static str,
    app_state: &'static str,
    update_surface: &'static str,
    window_entrypoint: &'static str,
    window: &'static str,
}

#[derive(Serialize)]
struct ConfigShowOutput {
    schema_version: u32,
    command: &'static str,
    config: RedactedConfigStatus,
}

#[derive(Serialize)]
struct ConfigSetOutput {
    schema_version: u32,
    command: &'static str,
    revision: u64,
    default_profile: ProfileId,
    cleanup_required: bool,
}

#[derive(Serialize)]
struct ConfigBackendStatusOutput {
    schema_version: u32,
    command: &'static str,
    revision: u64,
    backend: qiongli_execution::BackendStatusV1,
}

#[derive(Serialize)]
struct InstallStatusOutput {
    schema_version: u32,
    command: &'static str,
    contracts: InstallContractVersions,
    current_target: InstallBuildTarget,
    transaction_engine: &'static str,
    release_authority: &'static str,
    source_commit: &'static str,
    candidate: &'static str,
    launch_grant: &'static str,
    preview: &'static str,
    apply: &'static str,
    verify: &'static str,
    remove: &'static str,
    targets: [InstallTargetStatus; 2],
}

#[derive(Serialize)]
struct InstallCodexStatusOutput<'a> {
    schema_version: u32,
    command: &'static str,
    target: &'a CodexDiscoverySummaryV1,
    launch_grant: &'static str,
    preview: &'static str,
    apply: &'static str,
    activation: &'static str,
}

#[derive(Serialize)]
struct InstallInventoryOutput<'a> {
    schema_version: u32,
    command: &'static str,
    inventory: &'a ClientInventorySummaryV1,
}

#[derive(Serialize)]
struct InstallClaudeStatusOutput<'a> {
    schema_version: u32,
    command: &'static str,
    target: &'a ClaudeDiscoverySummaryV1,
    launch_grant: &'static str,
    preview: &'static str,
    apply: &'static str,
    activation: &'static str,
}

#[derive(Serialize)]
struct InstallContractVersions {
    artifact_identity: u32,
    launch_grant: u32,
    release_authority: u32,
    release_envelope: u32,
    release_candidate: u32,
    install_plan: u32,
    install_receipt: u32,
    native_payload_install_receipt: u32,
    codex_adapter: u32,
    codex_registration_receipt: u32,
    codex_registration_state: u32,
    claude_adapter: u32,
    claude_registration_receipt: u32,
    claude_registration_state: u32,
}

#[derive(Serialize)]
struct InstallBuildTarget {
    os: OperatingSystem,
    arch: Architecture,
}

#[derive(Serialize)]
struct InstallTargetStatus {
    family: LocalTargetFamily,
    state: &'static str,
}

#[derive(Serialize)]
struct ContentSummary<'a> {
    state: &'static str,
    pack_id: &'a str,
    content_version: &'a str,
    pack_sha256: &'a str,
    content_root_sha256: &'a str,
    profiles: &'a [ProfileProjection],
}

#[derive(Serialize)]
struct StatusOutput<'a> {
    schema_version: u32,
    command: &'static str,
    product_version: &'static str,
    content: ContentSummary<'a>,
    config: RedactedConfigStatus,
}

#[derive(Serialize)]
struct DoctorOutput<'a> {
    schema_version: u32,
    command: &'static str,
    overall: &'static str,
    checks: &'a [crate::product_diagnostics::ProductDoctorCheckV1],
    #[serde(skip_serializing_if = "Option::is_none")]
    paths: Option<&'a [crate::product_diagnostics::ProductPathInspectionV1]>,
}

#[derive(Serialize)]
struct PathsOutput<'a> {
    schema_version: u32,
    command: &'static str,
    product_version: &'static str,
    paths: &'a [crate::product_diagnostics::ProductPathInspectionV1],
}

#[cfg(unix)]
fn process_platform_home() -> Option<PathBuf> {
    nonempty_environment_path("HOME")
}

#[cfg(windows)]
fn process_platform_home() -> Option<PathBuf> {
    nonempty_environment_path("USERPROFILE")
        .or_else(windows_drive_home)
        .or_else(|| nonempty_environment_path("HOME"))
}

#[cfg(not(any(unix, windows)))]
fn process_platform_home() -> Option<PathBuf> {
    nonempty_environment_path("HOME")
}

fn nonempty_environment_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn discover_client_host(
    name: &str,
    home: Option<&Path>,
    include_codex_desktop: bool,
) -> (bool, Option<DetectedClientVersion>) {
    let executable = find_client_executable(name, home);
    let cli_version = executable
        .as_deref()
        .and_then(|path| package_version_for_executable(name, path))
        .or_else(|| home.and_then(|home| installed_tool_version(name, home)));
    let desktop_present = include_codex_desktop && codex_desktop_app_present(home);
    let desktop_version = include_codex_desktop
        .then(|| codex_desktop_version(home))
        .flatten();
    (
        executable.is_some() || desktop_present || cli_version.is_some(),
        cli_version.or(desktop_version),
    )
}

fn find_client_executable(name: &str, home: Option<&Path>) -> Option<PathBuf> {
    let mut directories = env::var_os("PATH")
        .map(|search_path| {
            env::split_paths(&search_path)
                .filter(|directory| directory.is_absolute())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(home) = home {
        directories.extend([
            home.join(".local/bin"),
            home.join(".local/share/mise/shims"),
            home.join(".local/share/pnpm"),
            home.join(".cargo/bin"),
            home.join(".npm-global/bin"),
            home.join(".bun/bin"),
            home.join("Library/pnpm"),
        ]);
    }
    #[cfg(target_os = "macos")]
    directories.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ]);

    directories.into_iter().find_map(|directory| {
        executable_names(name)
            .into_iter()
            .map(|candidate| directory.join(candidate))
            .find(|candidate| observed_file(candidate))
    })
}

fn detected_version(version: semver::Version) -> DetectedClientVersion {
    DetectedClientVersion {
        major: version.major,
        minor: version.minor,
        patch: version.patch,
    }
}

fn package_version_for_executable(name: &str, executable: &Path) -> Option<DetectedClientVersion> {
    let canonical = fs::canonicalize(executable).ok()?;
    let expected_package = match name {
        "codex" => "@openai/codex",
        "claude" => "@anthropic-ai/claude-code",
        _ => return None,
    };
    for directory in canonical.parent()?.ancestors().take(8) {
        let package = directory.join("package.json");
        let Some(bytes) = read_bounded_metadata(&package) else {
            continue;
        };
        let Ok(document) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        let Some(package_name) = document.get("name").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let accepted = package_name == expected_package
            || (name == "codex" && package_name.starts_with("@openai/codex-"));
        if accepted {
            return document
                .get("version")?
                .as_str()
                .and_then(|version| semver::Version::parse(version).ok())
                .map(detected_version);
        }
    }
    version_from_managed_path(name, &canonical)
}

fn read_bounded_metadata(path: &Path) -> Option<Vec<u8>> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_CLIENT_METADATA_BYTES {
        return None;
    }
    fs::read(path).ok()
}

fn version_from_managed_path(name: &str, path: &Path) -> Option<DetectedClientVersion> {
    let accepted_tools: &[&str] = match name {
        "codex" => &["codex"],
        "claude" => &["claude-code", "claude"],
        _ => return None,
    };
    let components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    components.windows(2).find_map(|pair| {
        accepted_tools
            .contains(&pair[0])
            .then(|| semver::Version::parse(pair[1]).ok())
            .flatten()
            .map(detected_version)
    })
}

fn installed_tool_version(name: &str, home: &Path) -> Option<DetectedClientVersion> {
    let roots = match name {
        "codex" => vec![home.join(".local/share/mise/installs/codex")],
        "claude" => vec![
            home.join(".local/share/mise/installs/claude-code"),
            home.join(".local/share/claude/versions"),
        ],
        _ => return None,
    };
    roots
        .iter()
        .filter_map(|root| newest_installed_version(root, name))
        .max_by_key(|version| (version.major, version.minor, version.patch))
}

fn newest_installed_version(root: &Path, name: &str) -> Option<DetectedClientVersion> {
    fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let version = entry
                .file_name()
                .to_str()
                .and_then(|value| semver::Version::parse(value).ok())?;
            installed_version_payload_present(name, &entry.path())
                .then(|| detected_version(version))
        })
        .max_by_key(|version| (version.major, version.minor, version.patch))
}

fn installed_version_payload_present(name: &str, root: &Path) -> bool {
    let candidates: &[&str] = match name {
        "codex" => &["codex", "bin/codex", "codex.exe", "bin/codex.exe"],
        "claude" => &["claude", "bin/claude", "claude.exe", "bin/claude.exe"],
        _ => return false,
    };
    candidates.iter().any(|candidate| {
        fs::metadata(root.join(candidate)).is_ok_and(|metadata| metadata.is_file())
    }) || fs::metadata(root).is_ok_and(|metadata| metadata.is_file())
}

#[cfg(target_os = "macos")]
fn codex_desktop_version(home: Option<&Path>) -> Option<DetectedClientVersion> {
    let mut applications = vec![PathBuf::from("/Applications/Codex.app")];
    if let Some(home) = home {
        applications.push(home.join("Applications/Codex.app"));
    }
    applications.into_iter().find_map(|application| {
        let plist = application.join("Contents/Info.plist");
        read_macos_bundle_version(&plist)
    })
}

#[cfg(target_os = "macos")]
fn read_macos_bundle_version(plist: &Path) -> Option<DetectedClientVersion> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let bytes = read_bounded_metadata(plist)?;
    let mut reader = Reader::from_reader(bytes.as_slice());
    reader.config_mut().trim_text(true);
    let mut saw_version_key = false;
    loop {
        match reader.read_event().ok()? {
            Event::Start(tag) if tag.name().as_ref() == b"key" => {
                saw_version_key =
                    reader.read_text(tag.name()).ok()?.as_ref() == "CFBundleShortVersionString";
            }
            Event::Start(tag) if saw_version_key && tag.name().as_ref() == b"string" => {
                return semver::Version::parse(reader.read_text(tag.name()).ok()?.as_ref())
                    .ok()
                    .map(detected_version);
            }
            Event::Eof => return None,
            _ => saw_version_key = false,
        }
    }
}

#[cfg(not(target_os = "macos"))]
const fn codex_desktop_version(_home: Option<&Path>) -> Option<DetectedClientVersion> {
    None
}

#[cfg(windows)]
fn executable_names(name: &str) -> [String; 4] {
    [
        name.to_owned(),
        format!("{name}.exe"),
        format!("{name}.cmd"),
        format!("{name}.bat"),
    ]
}

#[cfg(not(windows))]
fn executable_names(name: &str) -> [String; 1] {
    [name.to_owned()]
}

#[cfg(unix)]
fn observed_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn observed_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() || metadata.file_type().is_symlink())
}

#[cfg(target_os = "macos")]
fn codex_desktop_app_present(home: Option<&Path>) -> bool {
    observed_directory(Path::new("/Applications/Codex.app"))
        || home.is_some_and(|path| observed_directory(&path.join("Applications/Codex.app")))
}

#[cfg(not(target_os = "macos"))]
const fn codex_desktop_app_present(_home: Option<&Path>) -> bool {
    false
}

#[cfg(target_os = "macos")]
fn observed_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_dir() || metadata.file_type().is_symlink())
}

#[cfg(windows)]
fn windows_drive_home() -> Option<PathBuf> {
    let mut drive = env::var_os("HOMEDRIVE").filter(|value| !value.is_empty())?;
    let path = env::var_os("HOMEPATH").filter(|value| !value.is_empty())?;
    drive.push(path);
    Some(PathBuf::from(drive))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_discard_requires_exact_transaction_digest_and_approval() {
        let mut args: Vec<OsString> = vec![
            "activate-discard".into(),
            "--transaction-id".into(),
            format!("update-{}", "1".repeat(32)).into(),
            "--expected-journal-digest".into(),
            "2".repeat(64).into(),
        ];
        assert!(super::parse_candidate_install_args(&args).is_err());
        args.push("--approve-filesystem-write".into());
        assert!(matches!(
            super::parse_candidate_install_args(&args),
            Ok(super::CandidateCliCommand::ActivateDiscard { .. })
        ));
        for flag in [
            "--approve-filesystem-write",
            "--approve-host-trust",
            "--approve-client-config-change",
        ] {
            let mut invalid = args.clone();
            invalid.push(flag.into());
            assert!(super::parse_candidate_install_args(&invalid).is_err());
        }
        let mut invalid = args.clone();
        invalid[2] = "../foreign".into();
        assert!(super::parse_candidate_install_args(&invalid).is_err());
        invalid = args.clone();
        invalid[4] = "invalid".into();
        assert!(super::parse_candidate_install_args(&invalid).is_err());
        args.extend([
            "--transaction-id".into(),
            format!("update-{}", "3".repeat(32)).into(),
        ]);
        assert!(super::parse_candidate_install_args(&args).is_err());
    }

    #[test]
    fn activation_preview_requires_one_predecessor_and_no_approval() {
        let mut args: Vec<OsString> = [
            "activate-preview",
            "--candidate",
            "candidate.json",
            "--archive",
            "archive.zip",
            "--release-notes",
            "notes.md",
            "--target",
            "codex",
            "--previous-install-id",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        args.push(format!("native-payload-{}", "a".repeat(64)).into());
        assert!(matches!(
            super::parse_candidate_install_args(&args),
            Ok(super::CandidateCliCommand::ActivatePreview { .. })
        ));
        assert!(super::parse_candidate_install_args(&args[..args.len() - 2]).is_err());
        let mut invalid = args.clone();
        invalid.push("--approve-filesystem-write".into());
        assert!(super::parse_candidate_install_args(&invalid).is_err());
        invalid = args.clone();
        invalid.extend_from_slice(&args[args.len() - 2..]);
        assert!(super::parse_candidate_install_args(&invalid).is_err());
        let mut prepared = args.clone();
        prepared[0] = "activate-prepare".into();
        prepared.extend(["--expected-preflight-digest".into(), "a".repeat(64).into()]);
        assert!(super::parse_candidate_install_args(&prepared).is_err());
        prepared.insert(1, "--approve-filesystem-write".into());
        assert!(matches!(
            super::parse_candidate_install_args(&prepared),
            Ok(super::CandidateCliCommand::ActivatePrepare { .. })
        ));
        for flag in [
            "--approve-filesystem-write",
            "--approve-host-trust",
            "--approve-client-config-change",
        ] {
            let mut invalid = prepared.clone();
            invalid.push(flag.into());
            assert!(super::parse_candidate_install_args(&invalid).is_err());
        }
        *args.last_mut().unwrap() = "../foreign".into();
        assert!(super::parse_candidate_install_args(&args).is_err());
    }

    #[test]
    fn candidate_stage_requires_exact_filesystem_approval() {
        use std::ffi::OsString;
        let common: Vec<OsString> = [
            "stage-preview",
            "--candidate",
            "candidate.json",
            "--archive",
            "archive",
            "--release-notes",
            "notes.md",
            "--target",
            "codex",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        assert!(matches!(
            super::parse_candidate_install_args(&common),
            Ok(super::CandidateCliCommand::StagePreview(_))
        ));
        let mut stage = common.clone();
        stage[0] = "stage".into();
        stage.extend([
            OsString::from("--expected-approval-digest"),
            "a".repeat(64).into(),
        ]);
        assert!(super::parse_candidate_install_args(&stage).is_err());
        stage.push("--approve-filesystem-write".into());
        assert!(matches!(
            super::parse_candidate_install_args(&stage),
            Ok(super::CandidateCliCommand::Stage { .. })
        ));
        for flag in [
            "--approve-filesystem-write",
            "--approve-client-config-change",
            "--approve-host-trust",
        ] {
            let mut invalid = stage.clone();
            invalid.push(flag.into());
            assert!(super::parse_candidate_install_args(&invalid).is_err());
            let mut preview = common.clone();
            preview.push(flag.into());
            assert!(super::parse_candidate_install_args(&preview).is_err());
        }
        let mut missing_digest = common;
        missing_digest[0] = "stage".into();
        missing_digest.push("--approve-filesystem-write".into());
        assert!(super::parse_candidate_install_args(&missing_digest).is_err());
        stage[0] = "apply".into();
        assert!(super::parse_candidate_install_args(&stage).is_err());
    }

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn desktop_context_drops_the_process_working_directory_without_losing_home() {
        let home = PathBuf::from("/bounded-home");
        let project = PathBuf::from("/implicit-process-cwd");
        let environment = CommandEnvironment::with_paths(None, Some(home.clone()), None)
            .with_inventory_context(None, Some(project), false, false)
            .without_project_context();

        assert_eq!(environment.platform_home(), Some(home.as_path()));
        assert_eq!(environment.project_root(), None);
    }

    #[test]
    fn client_versions_are_read_from_bounded_package_and_tool_metadata() {
        let requested_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!(
                "qiongli-command-client-version-{}",
                std::process::id()
            ));
        fs::create_dir(&requested_root).expect("version fixture root must be unique");
        let root = fs::canonicalize(&requested_root).expect("fixture root must canonicalize");
        let package_root = root.join("node_modules/@openai/codex");
        let executable = package_root.join("bin/codex.js");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"bounded fixture").unwrap();
        fs::write(
            package_root.join("package.json"),
            br#"{"name":"@openai/codex","version":"0.144.4"}"#,
        )
        .unwrap();
        assert_eq!(
            package_version_for_executable("codex", &executable),
            Some(DetectedClientVersion {
                major: 0,
                minor: 144,
                patch: 4,
            })
        );

        let mise_root = root.join(".local/share/mise/installs/claude-code");
        for version in ["2.1.100", "2.1.209"] {
            let payload = mise_root.join(version).join("claude");
            fs::create_dir_all(payload.parent().unwrap()).unwrap();
            fs::write(payload, b"bounded fixture").unwrap();
        }
        assert_eq!(
            installed_tool_version("claude", &root),
            Some(DetectedClientVersion {
                major: 2,
                minor: 1,
                patch: 209,
            })
        );
        fs::remove_dir_all(requested_root).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn codex_bundle_version_is_read_without_launching_an_external_runtime() {
        let requested_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!(
                "qiongli-command-bundle-version-{}",
                std::process::id()
            ));
        fs::create_dir(&requested_root).expect("bundle fixture root must be unique");
        let plist = requested_root.join("Info.plist");
        fs::write(
            &plist,
            br#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.openai.codex</string>
<key>CFBundleShortVersionString</key><string>1.19.3</string>
</dict></plist>"#,
        )
        .unwrap();

        assert_eq!(
            read_macos_bundle_version(&plist),
            Some(DetectedClientVersion {
                major: 1,
                minor: 19,
                patch: 3,
            })
        );
        fs::remove_dir_all(requested_root).unwrap();
    }

    #[test]
    fn parser_accepts_the_frozen_command_families() {
        assert_eq!(
            parse_args(args(&[])),
            Ok(if cfg!(feature = "desktop") {
                Command::Ui
            } else {
                Command::Help
            })
        );
        assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
        assert_eq!(parse_args(args(&["ui"])), Ok(Command::Ui));
        assert_eq!(
            parse_args(args(&["ui", "--startup-check"])),
            Ok(Command::UiStartupCheck)
        );
        assert_eq!(
            parse_args(args(&["app", "snapshot"])),
            Ok(Command::AppSnapshot)
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "read-project-artifact",
                "--node-id",
                "nod_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--expected-projection-id",
                "grp_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "--project-id",
                "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "--expected-project-revision",
                "12",
            ])),
            Ok(Command::AppReadProjectArtifact {
                project_id: ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051".to_owned())
                    .unwrap(),
                expected_project_revision: 12,
                expected_projection_id:
                    "grp_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                        .to_owned(),
                entity_kind: AcademicGraphEntityKind::Node,
                entity_id: "nod_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_owned(),
            })
        );
        assert_eq!(
            parse_args(args(&["app", "verify-integrations", "--target", "all",])),
            Ok(Command::AppVerifyIntegrations {
                target: AppVerificationTarget::All,
            })
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "verify-skills",
                "--preset",
                "current-project",
            ])),
            Ok(Command::AppVerifySkills {
                preset: AppSkillsVerificationPreset::CurrentProject,
            })
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "verify-skills",
                "--target-id",
                "skills-target-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ])),
            Ok(Command::AppVerifyManagedSkillsTarget {
                target_id:
                    "skills-target-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        .to_string(),
            })
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "plan",
                "integrations-install",
                "--target",
                "codex",
            ])),
            Ok(Command::AppManaged(
                ManagedOperationCliCommand::PlanIntegrationsInstall {
                    targets: vec![ManagedIntegrationTargetV1::Codex],
                }
            ))
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "plan",
                "skills-reconcile",
                "--profile",
                "skill-only",
                "--preset",
                "qiongli-managed",
            ])),
            Ok(Command::AppManaged(
                ManagedOperationCliCommand::PlanSkillsReconcile {
                    preset: ManagedSkillsPresetV1::QiongliManaged,
                    profile: ProfileId::SkillOnly,
                }
            ))
        );
        assert_eq!(
            parse_args(args(&["app", "plan", "cli-install"])),
            Ok(Command::AppManaged(
                ManagedOperationCliCommand::PlanCliInstall
            ))
        );
        assert_eq!(
            parse_args(args(&["app", "plan", "cli-remove"])),
            Ok(Command::AppManaged(
                ManagedOperationCliCommand::PlanCliRemove
            ))
        );
        assert_eq!(
            parse_args(args(&["app", "plan", "cli-path-configure"])),
            Ok(Command::AppManaged(
                ManagedOperationCliCommand::PlanCliPathConfigure
            ))
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "plan",
                "integrations-reconcile",
                "--target",
                "all",
            ])),
            Ok(Command::AppManaged(
                ManagedOperationCliCommand::PlanIntegrationsReconcile {
                    targets: vec![
                        ManagedIntegrationTargetV1::Codex,
                        ManagedIntegrationTargetV1::ClaudeCode,
                    ],
                }
            ))
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "plan",
                "skills-remove",
                "--target-id",
                "skills-target-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ])),
            Ok(Command::AppManaged(
                ManagedOperationCliCommand::PlanSkillsRemove {
                    target_id:
                        "skills-target-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                            .to_string(),
                }
            ))
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "plan",
                "skills-detach",
                "--target-id",
                "skills-target-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ])),
            Ok(Command::AppManaged(
                ManagedOperationCliCommand::PlanSkillsDetach {
                    target_id:
                        "skills-target-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                            .to_string(),
                }
            ))
        );
        assert_eq!(
            parse_args(args(&[
                "app",
                "apply",
                "--plan",
                "/approved/managed-operation.json",
                "--expected-plan-digest",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "--approve-filesystem-write",
            ])),
            Ok(Command::AppManaged(ManagedOperationCliCommand::Apply {
                plan_path: PathBuf::from("/approved/managed-operation.json"),
                expected_plan_digest:
                    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                approve_filesystem_write: true,
                approve_client_config_change: false,
                approve_host_trust: false,
            }))
        );
        assert_eq!(
            parse_args(args(&[
                "ui",
                "--candidate",
                "/approved/qiongli.candidate.json",
                "--archive",
                "/approved/qiongli.zip",
                "--release-notes",
                "/approved/qiongli.release-notes.md",
                "--target",
                "codex",
            ])),
            Ok(Command::UiCandidate(CandidateReleaseOptions {
                candidate: PathBuf::from("/approved/qiongli.candidate.json"),
                archive: PathBuf::from("/approved/qiongli.zip"),
                release_notes: PathBuf::from("/approved/qiongli.release-notes.md"),
                target: ClientActivationTarget::Codex,
            }))
        );
        assert_eq!(
            parse_args(args(&["content", "list"])),
            Ok(Command::ContentList)
        );
        assert_eq!(
            parse_args(args(&["config", "show"])),
            Ok(Command::ConfigShow)
        );
        assert_eq!(
            parse_args(args(&["config", "backend", "status"])),
            Ok(Command::ConfigBackendStatus)
        );
        for values in [
            vec![
                "config",
                "backend",
                "set",
                "--enabled",
                "true",
                "--expected-revision",
                "3",
            ],
            vec!["config", "backend", "test", "--confirm-network-request"],
        ] {
            assert_eq!(
                parse_args(args(&values)),
                Err(config_usage_error("host-driven execution required"))
            );
        }
        assert_eq!(
            parse_args(args(&["update", "status"])),
            Ok(Command::Update(UpdateCliCommand::Status))
        );
        assert_eq!(
            parse_args(args(&["update", "check"])),
            Ok(Command::Update(UpdateCliCommand::Check))
        );
        assert_eq!(
            parse_args(args(&["update", "download", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Download {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&["update", "verify", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Verify {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&["update", "stage", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Stage {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&["update", "install", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Install {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&[
                "update",
                "health",
                "--transaction-id",
                "update-0123456789abcdef0123456789abcdef",
            ])),
            Ok(Command::Update(UpdateCliCommand::Health {
                transaction_id: "update-0123456789abcdef0123456789abcdef".to_string(),
            }))
        );
        assert_eq!(
            parse_args(args(&["update", "cancel", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Cancel {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&[
                "update",
                "channel",
                "--expected-revision",
                "3",
                "--stream",
                "stable",
            ])),
            Ok(Command::Update(UpdateCliCommand::Channel {
                expected_revision: 3,
                stream: UpdateStreamPreference::Stable,
            }))
        );
        assert_eq!(parse_args(args(&["status"])), Ok(Command::Status));
        assert_eq!(
            parse_args(args(&["paths"])),
            Ok(Command::Paths { json: false })
        );
        assert_eq!(
            parse_args(args(&["paths", "--json"])),
            Ok(Command::Paths { json: true })
        );
        assert_eq!(
            parse_args(args(&["doctor"])),
            Ok(Command::Doctor { exact_paths: false })
        );
        assert_eq!(
            parse_args(args(&["doctor", "--paths", "exact"])),
            Ok(Command::Doctor { exact_paths: true })
        );
        assert_eq!(
            parse_args(args(&["install", "codex", "status"])),
            Ok(Command::InstallCodexStatus)
        );
        assert_eq!(
            parse_args(args(&["install", "inventory"])),
            Ok(Command::InstallInventory)
        );
        assert_eq!(
            parse_args(args(&["install", "claude", "status"])),
            Ok(Command::InstallClaudeStatus)
        );
        assert_eq!(
            parse_args(args(&[
                "install",
                "native",
                "preview",
                "--release",
                "/approved/release.json",
                "--archive",
                "/approved/archive.zip",
                "--managed-root",
                "/approved/managed",
                "--target",
                "codex",
            ])),
            Ok(Command::InstallNative(NativeCliCommand::Preview(
                NativeReleaseOptions {
                    release: PathBuf::from("/approved/release.json"),
                    archive: PathBuf::from("/approved/archive.zip"),
                    managed_root: PathBuf::from("/approved/managed"),
                    target: NativeClientTarget::Codex,
                }
            )))
        );
        assert_eq!(
            parse_args(args(&[
                "install",
                "candidate",
                "preview",
                "--candidate",
                "/approved/qiongli.candidate.json",
                "--archive",
                "/approved/qiongli.zip",
                "--release-notes",
                "/approved/qiongli.release-notes.md",
                "--target",
                "claude",
            ])),
            Ok(Command::InstallCandidate(CandidateCliCommand::Preview(
                CandidateReleaseOptions {
                    candidate: PathBuf::from("/approved/qiongli.candidate.json"),
                    archive: PathBuf::from("/approved/qiongli.zip"),
                    release_notes: PathBuf::from("/approved/qiongli.release-notes.md"),
                    target: ClientActivationTarget::ClaudeCode,
                }
            )))
        );
        assert_eq!(
            parse_args(args(&[
                "mcp",
                "serve",
                "--profile",
                "lite",
                "--transport",
                "stdio",
            ])),
            Ok(Command::McpServeLiteStdio)
        );
        assert_eq!(
            parse_args(args(&[
                "mcp",
                "serve",
                "--profile",
                "full",
                "--transport",
                "stdio",
            ])),
            Ok(Command::McpServeFullStdio)
        );
    }

    #[test]
    fn parser_accepts_bounded_legacy_migration_stages() {
        let migration_id = "migration-1800000000-42";
        assert_eq!(
            parse_args(args(&["migrate-1x", "inspect"])),
            Ok(Command::Migrate1x(LegacyMigrationCliCommand::Inspect))
        );
        assert_eq!(
            parse_args(args(&["migrate-1x", "preview"])),
            Ok(Command::Migrate1x(LegacyMigrationCliCommand::Preview {
                provider_resolutions: Vec::new(),
            }))
        );
        assert_eq!(
            parse_args(args(&[
                "migrate-1x",
                "preview",
                "--provider-resolution",
                "crossref=use-legacy",
                "--provider-resolution",
                "openalex=keep-v2",
            ])),
            Ok(Command::Migrate1x(LegacyMigrationCliCommand::Preview {
                provider_resolutions: vec![
                    LegacyProviderResolution {
                        provider: LegacyProviderId::OpenAlex,
                        strategy: LegacyProviderResolutionStrategy::KeepV2,
                    },
                    LegacyProviderResolution {
                        provider: LegacyProviderId::Crossref,
                        strategy: LegacyProviderResolutionStrategy::UseLegacy,
                    },
                ],
            }))
        );
        assert_eq!(
            parse_args(args(&[
                "migrate-1x",
                "apply",
                "--migration-id",
                migration_id,
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write",
                "--approve-client-config-change",
                "--approve-secret-store-write",
            ])),
            Ok(Command::Migrate1x(LegacyMigrationCliCommand::Apply {
                migration_id: migration_id.to_owned(),
                expected_plan_digest:
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
                approve_filesystem_write: true,
                approve_client_config_change: true,
                approve_secret_store_write: true,
            }))
        );
        for (flag, action) in [
            (
                "--confirm-host-activation",
                LegacyMigrationContinueAction::ConfirmHostActivation,
            ),
            ("--approve-cleanup", LegacyMigrationContinueAction::Cleanup),
            ("--finalize", LegacyMigrationContinueAction::Finalize),
        ] {
            assert_eq!(
                parse_args(args(&[
                    "migrate-1x",
                    "continue",
                    "--migration-id",
                    migration_id,
                    flag,
                ])),
                Ok(Command::Migrate1x(LegacyMigrationCliCommand::Continue {
                    migration_id: migration_id.to_owned(),
                    action,
                }))
            );
        }
        assert!(
            parse_args(args(&[
                "migrate-1x",
                "continue",
                "--migration-id",
                migration_id,
                "--approve-cleanup",
                "--finalize",
            ]))
            .is_err()
        );
    }

    #[test]
    fn parser_accepts_both_option_orders_and_the_lite_alias() {
        let first = parse_args(args(&[
            "content",
            "materialize",
            "--profile",
            "lite",
            "--target",
            "/approved/target",
        ]));
        let second = parse_args(args(&[
            "content",
            "materialize",
            "--target",
            "/approved/target",
            "--profile",
            "marketplace-lite",
        ]));
        assert_eq!(first, second);

        let digest = "a".repeat(64);
        let first = parse_args(args(&[
            "install",
            "candidate",
            "apply",
            "--candidate",
            "/approved/qiongli.candidate.json",
            "--archive",
            "/approved/qiongli.zip",
            "--release-notes",
            "/approved/qiongli.release-notes.md",
            "--target",
            "codex",
            "--expected-approval-digest",
            &digest,
            "--approve-filesystem-write",
            "--approve-client-config-change",
            "--approve-host-trust",
        ]));
        let second = parse_args(args(&[
            "install",
            "candidate",
            "apply",
            "--approve-host-trust",
            "--target",
            "codex",
            "--release-notes",
            "/approved/qiongli.release-notes.md",
            "--approve-filesystem-write",
            "--expected-approval-digest",
            &digest,
            "--archive",
            "/approved/qiongli.zip",
            "--approve-client-config-change",
            "--candidate",
            "/approved/qiongli.candidate.json",
        ]));
        assert_eq!(first, second);

        let first = parse_args(args(&[
            "install",
            "native",
            "apply",
            "--release",
            "/approved/release.json",
            "--archive",
            "/approved/archive.zip",
            "--managed-root",
            "/approved/managed",
            "--target",
            "claude",
            "--expected-plan-digest",
            &digest,
            "--approve-filesystem-write",
        ]));
        let second = parse_args(args(&[
            "install",
            "native",
            "apply",
            "--approve-filesystem-write",
            "--target",
            "claude",
            "--managed-root",
            "/approved/managed",
            "--expected-plan-digest",
            &digest,
            "--archive",
            "/approved/archive.zip",
            "--release",
            "/approved/release.json",
        ]));
        assert_eq!(first, second);

        let first = parse_args(args(&[
            "mcp",
            "serve",
            "--profile",
            "lite",
            "--transport",
            "stdio",
        ]));
        let second = parse_args(args(&[
            "mcp",
            "serve",
            "--transport",
            "stdio",
            "--profile",
            "marketplace-lite",
        ]));
        assert_eq!(first, second);

        let first = parse_args(args(&[
            "config",
            "set",
            "--expected-revision",
            "7",
            "--default-profile",
            "full",
        ]));
        let second = parse_args(args(&[
            "config",
            "set",
            "--default-profile",
            "full",
            "--expected-revision",
            "7",
        ]));
        assert_eq!(first, second);
    }

    #[test]
    fn library_cli_boundary_does_not_launch_product_entrypoint_modes() {
        let content = crate::embedded_content().unwrap();
        let environment = CommandEnvironment::default();
        let no_args = run_cli(args(&[]), &environment, &content);
        if cfg!(feature = "desktop") {
            assert_eq!(no_args.exit_code(), 1);
            assert_eq!(
                no_args.stderr(),
                "error: desktop-command-requires-product-entrypoint\n"
            );
        } else {
            assert_eq!(no_args.exit_code(), 0);
            assert!(no_args.stderr().is_empty());
            assert_eq!(
                no_args.stdout(),
                run_cli(args(&["--help"]), &environment, &content).stdout()
            );
        }

        let ui = run_cli(args(&["ui"]), &environment, &content);
        assert_eq!(ui.exit_code(), 1);
        assert_eq!(
            ui.stderr(),
            "error: desktop-command-requires-product-entrypoint\n"
        );

        let startup_check = run_cli(args(&["ui", "--startup-check"]), &environment, &content);
        assert_eq!(startup_check.exit_code(), 0);
        assert!(startup_check.stderr().is_empty());
        let startup_check: serde_json::Value =
            serde_json::from_str(startup_check.stdout()).unwrap();
        assert_eq!(startup_check["command"], "ui-startup-check");
        assert_eq!(startup_check["service"], "ready");
        assert_eq!(startup_check["snapshot"], "ready");
        assert_eq!(startup_check["app_state"], "ready");
        assert_eq!(startup_check["update_surface"], "ready");
        assert_eq!(startup_check["window_entrypoint"], "available");
        assert_eq!(startup_check["window"], "not-opened");

        let mcp = run_cli(
            args(&["mcp", "serve", "--profile", "lite", "--transport", "stdio"]),
            &environment,
            &content,
        );
        assert_eq!(mcp.exit_code(), 1);
        assert_eq!(
            mcp.stderr(),
            "error: streaming-command-requires-product-entrypoint\n"
        );
    }

    #[test]
    fn install_inventory_is_read_only_and_redacts_real_paths() {
        let requested_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!("qiongli-command-inventory-{}", std::process::id()));
        fs::create_dir(&requested_root).expect("inventory fixture root must be unique");
        let root = fs::canonicalize(&requested_root).expect("fixture root must canonicalize");
        let home = root.join("home");
        let project = root.join("project");
        fs::create_dir(&home).expect("fixture home must exist");
        fs::create_dir(&project).expect("fixture project must exist");
        let environment = CommandEnvironment::with_paths(None, Some(home.clone()), None)
            .with_inventory_context(Some(root.join("codex-config")), Some(project), true, true);
        let content = crate::embedded_content().expect("embedded content must load");

        let output = run_cli(args(&["install", "inventory"]), &environment, &content);

        assert_eq!(output.exit_code(), 0);
        assert!(output.stderr().is_empty());
        let document: serde_json::Value =
            serde_json::from_str(output.stdout()).expect("inventory output must be JSON");
        assert_eq!(document["command"], "install-inventory");
        assert_eq!(
            document["inventory"]["schema_version"],
            qiongli_platform::CLIENT_INVENTORY_SCHEMA_VERSION
        );
        assert!(output.stdout().contains("codex-config-override"));
        assert!(output.stdout().contains("project"));
        assert!(!output.stdout().contains(root.to_string_lossy().as_ref()));
        assert!(!home.join(".qiongli").exists());
        assert!(!home.join(".agents").exists());
        assert!(!home.join(".claude").exists());
        fs::remove_dir_all(requested_root).expect("fixture cleanup must succeed");
    }

    #[test]
    fn app_verification_commands_share_the_gui_event_contract_without_writing_state() {
        let requested_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!("qiongli-command-app-verify-{}", std::process::id()));
        fs::create_dir(&requested_root).expect("App verification fixture root must be unique");
        let root = fs::canonicalize(&requested_root).expect("fixture root must canonicalize");
        let home = root.join("home");
        let project = root.join("project");
        fs::create_dir(&home).expect("fixture home must exist");
        fs::create_dir(&project).expect("fixture project must exist");
        let environment = CommandEnvironment::with_paths(None, Some(home.clone()), None)
            .with_inventory_context(None, Some(project), false, false);
        let content = crate::embedded_content().expect("embedded content must load");

        for values in [
            vec!["app", "verify-integrations", "--target", "all"],
            vec!["app", "verify-skills", "--preset", "qiongli-managed"],
            vec![
                "app",
                "verify-skills",
                "--target-id",
                "skills-target-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ],
        ] {
            let output = run_cli(args(&values), &environment, &content);
            assert_eq!(output.exit_code(), 0, "{}", output.stderr());
            assert!(output.stderr().is_empty());
            let event: serde_json::Value =
                serde_json::from_str(output.stdout()).expect("App event output must be JSON");
            assert!(
                matches!(
                    event["type"].as_str(),
                    Some("completed" | "failed" | "validation-failed")
                ),
                "verification must return a versioned GUI-compatible App event"
            );
        }

        assert!(!home.join(".qiongli").exists());
        assert!(!home.join(".qiongli-skills").exists());
        assert!(!home.join(".agents").exists());
        fs::remove_dir_all(requested_root).expect("fixture cleanup must succeed");
    }

    #[test]
    fn project_artifact_parser_rejects_ambiguous_or_unbounded_identity() {
        for values in [
            vec![
                "app",
                "read-project-artifact",
                "--project-id",
                "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "--expected-project-revision",
                "12",
                "--expected-projection-id",
                "grp_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ],
            vec![
                "app",
                "read-project-artifact",
                "--project-id",
                "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "--expected-project-revision",
                "12",
                "--expected-projection-id",
                "grp_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "--node-id",
                "nod_not-a-digest",
            ],
            vec![
                "app",
                "read-project-artifact",
                "--project-id",
                "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "--expected-project-revision",
                "0",
                "--expected-projection-id",
                "grp_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "--edge-id",
                "edg_cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            ],
        ] {
            assert!(parse_args(args(&values)).is_err(), "{values:?}");
        }
    }

    #[test]
    fn parser_rejects_missing_duplicate_unknown_and_private_values() {
        for values in [
            vec!["app", "plan"],
            vec!["app", "plan", "skills-reconcile", "--profile", "skill-only"],
            vec![
                "app",
                "plan",
                "skills-reconcile",
                "--preset",
                "qiongli-managed",
                "--preset",
                "current-project",
                "--profile",
                "skill-only",
            ],
            vec![
                "app",
                "plan",
                "skills-update",
                "--target-id",
                "skills-target-private-canary",
                "--path",
                "/private/canary",
            ],
            vec![
                "app",
                "apply",
                "--plan",
                "/approved/plan.json",
                "--expected-plan-digest",
                "a",
                "--approve-filesystem-write",
                "--approve-filesystem-write",
            ],
            vec!["content"],
            vec!["content", "list", "extra"],
            vec!["content", "materialize", "--profile", "full"],
            vec![
                "content",
                "materialize",
                "--profile",
                "full",
                "--profile",
                "full",
                "--target",
                "/approved/target",
            ],
            vec!["config", "set", "--expected-revision", "not-a-number"],
            vec!["config", "set", "--api-key", "private-canary"],
            vec!["config", "backend"],
            vec!["config", "backend", "test"],
            vec!["config", "backend", "test", "--api-key", "private-canary"],
            vec![
                "config",
                "backend",
                "set",
                "--expected-revision",
                "0",
                "--enabled",
                "maybe",
            ],
            vec!["update"],
            vec!["update", "status", "extra"],
            vec!["update", "channel", "--expected-revision", "0"],
            vec!["update", "download"],
            vec!["update", "verify"],
            vec!["update", "cancel", "--expected-revision", "not-a-number"],
            vec![
                "update",
                "channel",
                "--expected-revision",
                "0",
                "--stream",
                "alpha",
            ],
            vec!["update", "check", "--url", "https://private-canary"],
            vec!["status", "extra"],
            vec!["mcp"],
            vec!["mcp", "serve", "--profile", "lite"],
            vec!["mcp", "serve", "--transport", "stdio"],
            vec![
                "install",
                "native",
                "apply",
                "--release",
                "/approved/release.json",
                "--archive",
                "/approved/archive.zip",
                "--managed-root",
                "/approved/managed",
                "--target",
                "codex",
                "--expected-plan-digest",
                "a",
                "--approve-filesystem-write",
            ],
            vec![
                "install",
                "candidate",
                "apply",
                "--candidate",
                "/approved/qiongli.candidate.json",
                "--archive",
                "/approved/qiongli.zip",
                "--release-notes",
                "/approved/qiongli.release-notes.md",
                "--target",
                "codex",
                "--expected-approval-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write",
                "--approve-client-config-change",
            ],
            vec![
                "install",
                "candidate",
                "remove",
                "--target",
                "codex",
                "--install-id",
                "native-payload-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write",
            ],
            vec![
                "install",
                "native",
                "remove",
                "--managed-root",
                "/approved/managed",
                "--install-id",
                "native-payload-invalid",
                "--approve-filesystem-write",
            ],
            vec![
                "install",
                "native",
                "remove",
                "--managed-root",
                "/approved/managed",
                "--install-id",
                "native-payload-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ],
            vec!["mcp", "serve", "--profile", "lite", "--transport", "http"],
            vec![
                "mcp",
                "serve",
                "--profile",
                "lite",
                "--profile",
                "lite",
                "--transport",
                "stdio",
            ],
        ] {
            assert!(parse_args(args(&values)).is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn parser_rejects_non_utf8_unix_control_tokens() {
        use std::os::unix::ffi::OsStringExt;

        let invalid = OsString::from_vec(vec![0xff, 0xfe]);
        assert!(parse_args([invalid]).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn parser_rejects_unpaired_windows_surrogates_in_control_tokens() {
        use std::os::windows::ffi::OsStringExt;

        let invalid = OsString::from_wide(&[0xd800]);
        assert!(parse_args([invalid]).is_err());
    }
}
