mod agent_run;
mod all_chat_api;
mod all_chat_control;
mod all_chat_history;
mod all_chat_research;
#[doc(hidden)]
pub use all_chat_control::all_chat_control_schema_json;
#[doc(hidden)]
pub use all_chat_history::all_chat_history_schema_json;
#[doc(hidden)]
pub use all_chat_research::{all_chat_research_control_schema_json, all_chat_research_schema_json};
mod application;
mod candidate_cli;
mod capture_assignment_cli;
mod capture_cli;
mod capture_consolidation_cli;
mod capture_delivery_cli;
mod capture_resolution_cli;
mod cli_install;
mod command;
mod credential_store;
mod desktop;
mod desktop_api;
mod desktop_contract;
mod legacy_migration_cli;
mod macos_update_stage;
mod managed_content;
mod managed_operation;
mod mcp;
mod native_cli;
mod native_update_replace;
mod orchestration_control;
#[cfg(test)]
mod platform_capacity;
mod portfolio_cli;
mod product_diagnostics;
mod project_cli;
mod repository_capture_cli;
mod update_cli;
mod update_reconcile;

pub use all_chat_api::{
    ALL_CHAT_APP_CONTRACT_SCHEMA_ID, ALL_CHAT_APP_CONTRACT_SCHEMA_VERSION, AllChatAppSnapshotV1,
    all_chat_app_contract_cancelled_fixture_json, all_chat_app_contract_completed_fixture_json,
    all_chat_app_contract_schema_json, project_all_chat_app_snapshot,
};
pub use application::{
    DesktopApplicationAssetError, DesktopApplicationError, DesktopApplicationMetadata,
    desktop_application_icon_png, desktop_application_metadata, run_desktop_application,
};
pub use command::{
    CliOutput, CommandEnvironment, ProductAction, failed_embedded_content_output, prepare_action,
    run_cli,
};
#[doc(hidden)]
pub use credential_store::native_secret_store;
pub use desktop::{
    DesktopActivationSession, DesktopCandidateSession, DesktopLaunchError,
    app_api_contract_fixture_json, run_desktop, run_desktop_with_activation_sessions,
    run_desktop_with_candidate_sessions,
};
pub use desktop_contract::{
    DESKTOP_APPLICATION_IDENTIFIER, DESKTOP_CONTENT_ERROR_CODE, DESKTOP_PRODUCT_LICENSE,
    DESKTOP_PRODUCT_NAME, DESKTOP_PRODUCT_VERSION, DESKTOP_RUNTIME_ERROR_CODE,
    DESKTOP_STARTUP_ERROR_CODE, DESKTOP_WINDOW_TITLE,
};
#[doc(hidden)]
pub use mcp::FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES;
pub use mcp::{serve_full_mcp, serve_lite_mcp};
pub use native_update_replace::run_native_update_helper;
use qiongli_content::{EmbeddedContent, ResourcePackLoaderError};
use qiongli_platform::{
    NativeReleaseAuthority, NativeReleaseAuthorityError, VerifiedZoteroCompanionArtifact,
    ZoteroCompanionArtifactError, verify_zotero_companion_artifact,
};

pub const EMBEDDED_PACK_SHA256: &str =
    include_str!(concat!(env!("OUT_DIR"), "/qiongli-core.qlpack.sha256"));

static EMBEDDED_PACK_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/qiongli-core.qlpack"));

static EMBEDDED_RELEASE_AUTHORITY_BYTES: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/qiongli-native-release-authority.json"
));

static EMBEDDED_ZOTERO_COMPANION_XPI_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/qiongli-zotero-companion.xpi"));

static EMBEDDED_ZOTERO_COMPANION_MANIFEST_BYTES: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/qiongli-zotero-companion.manifest.json"
));

const EMBEDDED_SOURCE_COMMIT: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/qiongli-native-source-commit.txt"
));

const EMBEDDED_MACOS_TEAM_ID: &str =
    include_str!(concat!(env!("OUT_DIR"), "/qiongli-macos-team-id.txt"));

pub fn embedded_content() -> Result<EmbeddedContent, ResourcePackLoaderError> {
    EmbeddedContent::load(EMBEDDED_PACK_BYTES, EMBEDDED_PACK_SHA256)
}

pub fn embedded_release_authority()
-> Result<Option<NativeReleaseAuthority>, NativeReleaseAuthorityError> {
    if EMBEDDED_RELEASE_AUTHORITY_BYTES.is_empty() {
        Ok(None)
    } else {
        let authority = NativeReleaseAuthority::from_json(EMBEDDED_RELEASE_AUTHORITY_BYTES)?;
        authority.validate_product_version(env!("CARGO_PKG_VERSION"))?;
        Ok(Some(authority))
    }
}

pub fn embedded_zotero_companion()
-> Result<VerifiedZoteroCompanionArtifact, ZoteroCompanionArtifactError> {
    verify_zotero_companion_artifact(
        EMBEDDED_ZOTERO_COMPANION_MANIFEST_BYTES,
        EMBEDDED_ZOTERO_COMPANION_XPI_BYTES,
    )
}

#[must_use]
pub const fn embedded_source_commit() -> Option<&'static str> {
    if EMBEDDED_SOURCE_COMMIT.is_empty() {
        None
    } else {
        Some(EMBEDDED_SOURCE_COMMIT)
    }
}

#[must_use]
pub const fn embedded_macos_team_id() -> Option<&'static str> {
    if EMBEDDED_MACOS_TEAM_ID.is_empty() {
        None
    } else {
        Some(EMBEDDED_MACOS_TEAM_ID)
    }
}

pub use candidate_cli::{
    CandidateActivationPreviewOutput, candidate_activation_preview_contract_json,
    candidate_stage_contract_json, preview_native_candidate_activation,
};

pub use update_reconcile::{
    NativeActivationOutcome, activate_native_reconciliation, recover_native_reconciliation,
};

pub use candidate_cli::check_native_cli_health;
