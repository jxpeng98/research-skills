//! User-approved Plugin source exports. Signed product and Host authority stay separate.
use std::fs;
use std::path::{Path, PathBuf};

use qiongli_config::WorkflowVariantStore;
use qiongli_content::EmbeddedContent;
use qiongli_platform::{
    approve_claude_plugin_bundle_target, approve_codex_plugin_bundle_target,
    compose_local_claude_plugin_source, compose_local_codex_plugin_source,
    remove_local_claude_plugin_source, remove_local_codex_plugin_source,
    verify_local_claude_plugin_source, verify_local_codex_plugin_source,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::command::{CommandEnvironment, config_root};
use crate::managed_operation::ManagedIntegrationTargetV1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PluginSourceAction {
    Install,
    Update,
    Remove,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct PluginSourcePlan {
    pub action: PluginSourceAction,
    pub target: ManagedIntegrationTargetV1,
    pub destination: PathBuf,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub binary_sha256: String,
    pub expected_receipt_sha256: Option<String>,
    pub workflow_variant_sha256: Option<String>,
}

impl PluginSourcePlan {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if !self.destination.is_absolute()
            || self.destination.file_name().and_then(|s| s.to_str()) != Some("qiongli-next")
            || !digest_valid(&self.binary_sha256)
            || (self.action == PluginSourceAction::Install)
                != self.expected_receipt_sha256.is_none()
            || self
                .expected_receipt_sha256
                .as_deref()
                .is_some_and(|d| !digest_valid(d))
            || self
                .workflow_variant_sha256
                .as_deref()
                .is_some_and(|d| !digest_valid(d))
        {
            return Err("plugin-source-plan-invalid");
        }
        Ok(())
    }

    pub(crate) fn digest(&self) -> Result<String, &'static str> {
        let bytes =
            serde_json_canonicalizer::to_vec(self).map_err(|_| "plugin-source-plan-invalid")?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }
}

fn digest_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

#[derive(Serialize, schemars::JsonSchema)]
struct SourceObservation {
    receipt_sha256: String,
    binary_sha256: String,
    content_pack_sha256: String,
    workflow_variant_sha256: Option<String>,
    version: String,
}

fn validate_destination(
    environment: &CommandEnvironment,
    destination: &Path,
) -> Result<(), &'static str> {
    let home = environment
        .platform_home()
        .ok_or("plugin-source-home-unavailable")?;
    let codex = environment
        .codex_config_root()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".codex"));
    let claude = environment
        .claude_config_root()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".claude"));
    // These owners contain managed sources or private Host caches, not export destinations.
    let reserved = [codex, claude, home.join(".agents"), home.join(".qiongli")];
    if reserved.iter().any(|root| destination.starts_with(root)) {
        return Err("plugin-source-destination-reserved");
    }
    if destination.file_name().and_then(|s| s.to_str()) != Some("qiongli-next") {
        return Err("plugin-source-destination-invalid");
    }
    // The existing target owner rejects traversal, links, non-directories and unsafe parents.
    qiongli_content::approve_materialization_target(destination).map_err(|e| e.reason_code())?;
    let parent = fs::canonicalize(
        destination
            .parent()
            .ok_or("plugin-source-destination-invalid")?,
    )
    .map_err(|_| "plugin-source-destination-invalid")?;
    // Canonical comparisons also cover case aliases on Windows/macOS filesystems.
    if reserved
        .iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .any(|root| parent.starts_with(root))
    {
        return Err("plugin-source-destination-reserved");
    }
    Ok(())
}

fn observe(
    environment: &CommandEnvironment,
    target: ManagedIntegrationTargetV1,
    destination: &Path,
) -> Result<Option<SourceObservation>, &'static str> {
    validate_destination(environment, destination)?;
    match target {
        ManagedIntegrationTargetV1::Codex => {
            approve_codex_plugin_bundle_target(destination).map_err(|e| e.reason_code())?;
        }
        ManagedIntegrationTargetV1::ClaudeCode => {
            approve_claude_plugin_bundle_target(destination).map_err(|e| e.reason_code())?;
        }
    }
    match fs::symlink_metadata(destination) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("plugin-source-unavailable"),
        Ok(_) => {}
    }
    Ok(Some(match target {
        ManagedIntegrationTargetV1::Codex => {
            let target =
                approve_codex_plugin_bundle_target(destination).map_err(|e| e.reason_code())?;
            let verified =
                verify_local_codex_plugin_source(&target).map_err(|e| e.reason_code())?;
            let r = verified.receipt();
            SourceObservation {
                receipt_sha256: verified.receipt_sha256().to_owned(),
                binary_sha256: r.binary_sha256.clone(),
                content_pack_sha256: r.resource_pack_sha256.clone(),
                workflow_variant_sha256: r.workflow_variant_sha256.clone(),
                version: r.artifact.version.clone(),
            }
        }
        ManagedIntegrationTargetV1::ClaudeCode => {
            let target =
                approve_claude_plugin_bundle_target(destination).map_err(|e| e.reason_code())?;
            let verified =
                verify_local_claude_plugin_source(&target).map_err(|e| e.reason_code())?;
            let r = verified.receipt();
            SourceObservation {
                receipt_sha256: verified.receipt_sha256().to_owned(),
                binary_sha256: r.binary_sha256.clone(),
                content_pack_sha256: r.resource_pack_sha256.clone(),
                workflow_variant_sha256: r.workflow_variant_sha256.clone(),
                version: r.artifact.version.clone(),
            }
        }
    }))
}

pub(crate) fn plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    action: PluginSourceAction,
    target: ManagedIntegrationTargetV1,
    destination: &Path,
) -> Result<PluginSourcePlan, &'static str> {
    let observed = observe(environment, target, destination)?;
    if (action == PluginSourceAction::Install) != observed.is_none() {
        return Err(if observed.is_some() {
            "plugin-source-already-exists"
        } else {
            "plugin-source-not-installed"
        });
    }
    let root = config_root(environment).map_err(|e| e.reason_code())?;
    let variant = WorkflowVariantStore::new(root)
        .load(content.pack())
        .map_err(|e| e.reason_code())?;
    let executable = std::env::current_exe().map_err(|_| "plugin-source-executable-unavailable")?;
    let result = PluginSourcePlan {
        action,
        target,
        destination: destination.to_owned(),
        binary_sha256: crate::cli_install::regular_file_sha256(&executable)?,
        expected_receipt_sha256: observed.map(|o| o.receipt_sha256),
        workflow_variant_sha256: variant.variant_sha256().map(str::to_owned),
    };
    result.validate()?;
    Ok(result)
}

/// Called only after the shared managed plan, approvals and Home/config write guard.
pub(crate) fn apply(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    expected: &PluginSourcePlan,
) -> Result<String, &'static str> {
    let current = plan(
        environment,
        content,
        expected.action,
        expected.target,
        &expected.destination,
    )?;
    if &current != expected {
        return Err("managed-operation-precondition-changed");
    }
    let root = config_root(environment).map_err(|e| e.reason_code())?;
    let variant = WorkflowVariantStore::new(root)
        .load(content.pack())
        .map_err(|e| e.reason_code())?;
    if variant.variant_sha256() != expected.workflow_variant_sha256.as_deref() {
        return Err("managed-operation-precondition-changed");
    }
    let binary = std::env::current_exe().map_err(|_| "plugin-source-executable-unavailable")?;
    let prior = expected.expected_receipt_sha256.as_deref();
    match expected.target {
        ManagedIntegrationTargetV1::Codex => {
            let target = approve_codex_plugin_bundle_target(&expected.destination)
                .map_err(|e| e.reason_code())?;
            let verified = if expected.action == PluginSourceAction::Remove {
                remove_local_codex_plugin_source(
                    &target,
                    prior.ok_or("plugin-source-plan-invalid")?,
                )
            } else {
                compose_local_codex_plugin_source(
                    content.pack(),
                    &binary,
                    &expected.binary_sha256,
                    &target,
                    variant.overrides(),
                    prior,
                )
            }
            .map_err(|e| e.reason_code())?;
            Ok(verified.receipt_sha256().to_owned())
        }
        ManagedIntegrationTargetV1::ClaudeCode => {
            let target = approve_claude_plugin_bundle_target(&expected.destination)
                .map_err(|e| e.reason_code())?;
            let verified = if expected.action == PluginSourceAction::Remove {
                remove_local_claude_plugin_source(
                    &target,
                    prior.ok_or("plugin-source-plan-invalid")?,
                )
            } else {
                compose_local_claude_plugin_source(
                    content.pack(),
                    &binary,
                    &expected.binary_sha256,
                    &target,
                    variant.overrides(),
                    prior,
                )
            }
            .map_err(|e| e.reason_code())?;
            Ok(verified.receipt_sha256().to_owned())
        }
    }
}

pub(crate) fn status(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    target: ManagedIntegrationTargetV1,
    destination: &Path,
) -> Result<String, &'static str> {
    let observed = observe(environment, target, destination)?;
    let root = config_root(environment).map_err(|e| e.reason_code())?;
    let variant = WorkflowVariantStore::new(root)
        .load(content.pack())
        .map_err(|e| e.reason_code())?;
    let binary = std::env::current_exe().map_err(|_| "plugin-source-executable-unavailable")?;
    let hash = crate::cli_install::regular_file_sha256(&binary)?;
    let state = match &observed {
        None => "missing",
        Some(o)
            if o.binary_sha256 == hash
                && o.content_pack_sha256 == content.pack().pack_sha256()
                && o.version == env!("CARGO_PKG_VERSION")
                && o.workflow_variant_sha256.as_deref() == variant.variant_sha256() =>
        {
            "source-current"
        }
        Some(_) => "source-update-available",
    };
    serde_json_canonicalizer::to_string(&SourceStatus {
        schema_version: 1,
        command: "plugin-source-status".into(),
        target,
        destination: destination.to_owned(),
        state: state.to_owned(),
        source: observed,
        authority: "user-local-source".into(),
        host_state: "not-verified".into(),
        plugin_id: "qiongli-next@qiongli-cli-local".into(),
    })
    .map_err(|_| "plugin-source-status-invalid")
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SourceStatus {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    #[schemars(regex(pattern = "^plugin-source-status$"))]
    command: String,
    target: ManagedIntegrationTargetV1,
    destination: PathBuf,
    #[schemars(regex(pattern = "^(missing|source-current|source-update-available)$"))]
    state: String,
    source: Option<SourceObservation>,
    #[schemars(regex(pattern = "^user-local-source$"))]
    authority: String,
    #[schemars(regex(pattern = "^not-verified$"))]
    host_state: String,
    #[schemars(regex(pattern = "^qiongli-next@qiongli-cli-local$"))]
    plugin_id: String,
}

pub(crate) fn contract_source() -> PluginSourcePlan {
    PluginSourcePlan {
        action: PluginSourceAction::Install,
        target: ManagedIntegrationTargetV1::Codex,
        destination: PathBuf::from("/example/qiongli-next"),
        binary_sha256: "3".repeat(64),
        expected_receipt_sha256: None,
        workflow_variant_sha256: None,
    }
}

pub fn plugin_source_contract_json() -> Result<String, serde_json::Error> {
    let status = SourceStatus {
        schema_version: 1,
        command: "plugin-source-status".into(),
        target: ManagedIntegrationTargetV1::Codex,
        destination: PathBuf::from("/example/qiongli-next"),
        state: "missing".into(),
        source: None,
        authority: "user-local-source".into(),
        host_state: "not-verified".into(),
        plugin_id: "qiongli-next@qiongli-cli-local".into(),
    };
    serde_json::to_string_pretty(&serde_json::json!({
        "managed": crate::managed_operation::generated_plan_contract(),
        "status": {"schema": schemars::generate::SchemaSettings::draft2020_12().into_generator().into_root_schema_for::<SourceStatus>(), "fixture": status}
    }))
}
