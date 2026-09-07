#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_config::{LoadedWorkflowVariant, WorkflowVariantStore};
use qiongli_content::{
    EmbeddedContent, MaterializationReceiptV1, MaterializationTarget, ProfileId,
    approve_materialization_target, verify_materialization,
};
use qiongli_platform::{
    ClientActivationTarget, PackagedProductInstallEffect, PackagedProductInstallPreview,
    PackagedProductInstallVerification, VerifiedPackagedProduct,
    apply_packaged_product_batch_install_with_overrides,
    preview_packaged_product_batch_install_with_variant, remove_packaged_product_install,
    verify_receipt_owned_packaged_product_install,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cli_install::{
    CliRemovalEffect, apply_cli_install, apply_cli_path_configure, apply_cli_remove,
    preview_cli_install, preview_cli_path_configure, preview_cli_remove,
};
use crate::command::{CommandEnvironment, config_root};
use crate::desktop::{
    HostPluginPlan, execute_host_plugin_plan_set, host_plugin_plans_digest,
    prepare_host_plugin_plans, update_store,
};
use crate::managed_content::{
    ManagedContentEntryV1, ManagedSkillsEntryState, apply_managed_materialization_with_overrides,
    detach_managed_materialization, load_managed_content_registry, managed_skills_target_id,
    materialization_receipt_sha256, observe_managed_skills_entry_with_variant,
    remove_managed_materialization_with_overrides,
};

const PLAN_DOCUMENT_KIND: &str = "qiongli-managed-operation-plan";
const PLAN_SCHEMA_VERSION: u32 = 1;
const PLAN_TTL_SECONDS: u64 = 600;
const PLAN_CLOCK_SKEW_SECONDS: u64 = 60;
const MAX_PLAN_BYTES: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ManagedSkillsPresetV1 {
    QiongliManaged,
    CurrentProject,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ManagedSkillsStateV1 {
    Missing,
    Current,
    UpdateAvailable,
    Drifted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ManagedOperationApprovalV1 {
    FilesystemWrite,
    ClientConfigChange,
    HostTrust,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ManagedIntegrationTargetV1 {
    Codex,
    ClaudeCode,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ManagedIntegrationEffectV1 {
    Install,
    Repair,
    ContentUpdate,
    AlreadyCurrent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ManagedIntegrationModeV1 {
    Install,
    Repair,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagedIntegrationInstallPreviewV1 {
    target: ManagedIntegrationTargetV1,
    effect: ManagedIntegrationEffectV1,
    native_plan_digest_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagedIntegrationVerificationV1 {
    target: ManagedIntegrationTargetV1,
    evidence_digest_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ManagedOperationV1 {
    SkillsReconcilePreset {
        preset: ManagedSkillsPresetV1,
        target_id: String,
        profile: ProfileId,
        expected_state: ManagedSkillsStateV1,
        expected_receipt_sha256: Option<String>,
    },
    SkillsUpdateTarget {
        target_id: String,
        profile: ProfileId,
        expected_state: ManagedSkillsStateV1,
        expected_receipt_sha256: String,
    },
    SkillsRemoveTarget {
        target_id: String,
        profile: ProfileId,
        expected_state: ManagedSkillsStateV1,
        expected_receipt_sha256: String,
    },
    SkillsDetachTarget {
        target_id: String,
        profile: ProfileId,
        expected_state: ManagedSkillsStateV1,
        expected_receipt_sha256: String,
    },
    IntegrationsReconcile {
        mode: ManagedIntegrationModeV1,
        control_sha256: String,
        native_batch_plan_digest_sha256: String,
        installs: Vec<ManagedIntegrationInstallPreviewV1>,
    },
    IntegrationsRemove {
        control_sha256: String,
        verifications: Vec<ManagedIntegrationVerificationV1>,
    },
    CliInstall {
        control_sha256: String,
        native_plan_digest_sha256: String,
    },
    CliRemove {
        control_sha256: String,
        native_plan_digest_sha256: String,
    },
    CliPathConfigure {
        control_sha256: String,
        native_plan_digest_sha256: String,
        profile_name: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagedOperationPlanV1 {
    document_kind: String,
    schema_version: u32,
    product_version: String,
    content_pack_sha256: String,
    content_root_sha256: String,
    created_at_unix: u64,
    expires_at_unix: u64,
    operation: ManagedOperationV1,
    approvals_required: Vec<ManagedOperationApprovalV1>,
    semantic_digest_sha256: String,
    plan_digest_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ManagedOperationCliCommand {
    PlanSkillsReconcile {
        preset: ManagedSkillsPresetV1,
        profile: ProfileId,
    },
    PlanSkillsUpdate {
        target_id: String,
    },
    PlanSkillsRemove {
        target_id: String,
    },
    PlanSkillsDetach {
        target_id: String,
    },
    PlanIntegrationsInstall {
        targets: Vec<ManagedIntegrationTargetV1>,
    },
    PlanIntegrationsReconcile {
        targets: Vec<ManagedIntegrationTargetV1>,
    },
    PlanIntegrationsRemove {
        targets: Vec<ManagedIntegrationTargetV1>,
    },
    PlanCliInstall,
    PlanCliRemove,
    PlanCliPathConfigure,
    Apply {
        plan_path: PathBuf,
        expected_plan_digest: String,
        approve_filesystem_write: bool,
        approve_client_config_change: bool,
        approve_host_trust: bool,
    },
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct ManagedOperationPlanBodyV1<'a> {
    document_kind: &'a str,
    schema_version: u32,
    product_version: &'a str,
    content_pack_sha256: &'a str,
    content_root_sha256: &'a str,
    created_at_unix: u64,
    expires_at_unix: u64,
    operation: &'a ManagedOperationV1,
    approvals_required: &'a [ManagedOperationApprovalV1],
    semantic_digest_sha256: &'a str,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct ManagedOperationResultV1 {
    schema_version: u32,
    command: &'static str,
    operation: &'static str,
    targets: Vec<String>,
    result: &'static str,
    receipt_sha256: Option<String>,
}

struct ManagedSkillsObservation {
    target: MaterializationTarget,
    profile: Option<ProfileId>,
    state: ManagedSkillsStateV1,
    receipt: Option<MaterializationReceiptV1>,
    receipt_sha256: Option<String>,
    workflow_variant_sha256: Option<String>,
}

pub(crate) fn execute(
    command: &ManagedOperationCliCommand,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<String, &'static str> {
    match command {
        ManagedOperationCliCommand::PlanSkillsReconcile { preset, profile } => {
            let plan = prepare_skills_reconcile_plan(environment, content, *preset, *profile)?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanSkillsUpdate { target_id } => {
            let plan = prepare_skills_update_plan(environment, content, target_id)?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanSkillsRemove { target_id } => {
            let plan = prepare_skills_remove_plan(environment, content, target_id)?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanSkillsDetach { target_id } => {
            let plan = prepare_skills_detach_plan(environment, content, target_id)?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanIntegrationsInstall { targets } => {
            let plan = prepare_integrations_reconcile_plan(
                environment,
                content,
                targets,
                ManagedIntegrationModeV1::Install,
            )?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanIntegrationsReconcile { targets } => {
            let plan = prepare_integrations_reconcile_plan(
                environment,
                content,
                targets,
                ManagedIntegrationModeV1::Repair,
            )?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanIntegrationsRemove { targets } => {
            let plan = prepare_integrations_remove_plan(environment, content, targets)?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanCliInstall => {
            let plan = prepare_cli_install_plan(environment, content)?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanCliRemove => {
            let plan = prepare_cli_remove_plan(environment, content)?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::PlanCliPathConfigure => {
            let plan = prepare_cli_path_configure_plan(environment, content)?;
            plan.to_canonical_json()
        }
        ManagedOperationCliCommand::Apply {
            plan_path,
            expected_plan_digest,
            approve_filesystem_write,
            approve_client_config_change,
            approve_host_trust,
        } => apply_plan(
            environment,
            content,
            plan_path,
            expected_plan_digest,
            &[
                (
                    ManagedOperationApprovalV1::FilesystemWrite,
                    *approve_filesystem_write,
                ),
                (
                    ManagedOperationApprovalV1::ClientConfigChange,
                    *approve_client_config_change,
                ),
                (ManagedOperationApprovalV1::HostTrust, *approve_host_trust),
            ],
        ),
    }
}

impl ManagedOperationPlanV1 {
    fn new(
        content: &EmbeddedContent,
        now_unix: u64,
        operation: ManagedOperationV1,
        approvals_required: Vec<ManagedOperationApprovalV1>,
        semantic_digest_sha256: String,
    ) -> Result<Self, &'static str> {
        let mut plan = Self {
            document_kind: PLAN_DOCUMENT_KIND.to_string(),
            schema_version: PLAN_SCHEMA_VERSION,
            product_version: env!("CARGO_PKG_VERSION").to_string(),
            content_pack_sha256: content.pack().pack_sha256().to_string(),
            content_root_sha256: content.pack().manifest().content_root_sha256.clone(),
            created_at_unix: now_unix,
            expires_at_unix: now_unix.saturating_add(PLAN_TTL_SECONDS),
            operation,
            approvals_required,
            semantic_digest_sha256,
            plan_digest_sha256: String::new(),
        };
        plan.plan_digest_sha256 = plan.compute_digest()?;
        plan.validate(now_unix)?;
        Ok(plan)
    }

    fn body(&self) -> ManagedOperationPlanBodyV1<'_> {
        ManagedOperationPlanBodyV1 {
            document_kind: &self.document_kind,
            schema_version: self.schema_version,
            product_version: &self.product_version,
            content_pack_sha256: &self.content_pack_sha256,
            content_root_sha256: &self.content_root_sha256,
            created_at_unix: self.created_at_unix,
            expires_at_unix: self.expires_at_unix,
            operation: &self.operation,
            approvals_required: &self.approvals_required,
            semantic_digest_sha256: &self.semantic_digest_sha256,
        }
    }

    fn compute_digest(&self) -> Result<String, &'static str> {
        let bytes = serde_json_canonicalizer::to_vec(&self.body())
            .map_err(|_| "managed-operation-plan-invalid")?;
        Ok(sha256_hex(&bytes))
    }

    fn validate(&self, now_unix: u64) -> Result<(), &'static str> {
        self.validate_for_version(now_unix, env!("CARGO_PKG_VERSION"))
    }

    fn validate_for_version(&self, now_unix: u64, version: &str) -> Result<(), &'static str> {
        if self.document_kind != PLAN_DOCUMENT_KIND
            || self.schema_version != PLAN_SCHEMA_VERSION
            || self.product_version != version
            || !valid_sha256(&self.content_pack_sha256)
            || !valid_sha256(&self.content_root_sha256)
            || !valid_sha256(&self.semantic_digest_sha256)
            || !valid_sha256(&self.plan_digest_sha256)
            || self.expires_at_unix < self.created_at_unix
            || self.expires_at_unix.saturating_sub(self.created_at_unix) > PLAN_TTL_SECONDS
            || self.created_at_unix > now_unix.saturating_add(PLAN_CLOCK_SKEW_SECONDS)
        {
            return Err("managed-operation-plan-invalid");
        }
        if now_unix > self.expires_at_unix {
            return Err("managed-operation-plan-expired");
        }
        validate_operation(&self.operation)?;
        if self.approvals_required != expected_approvals(&self.operation) {
            return Err("managed-operation-plan-invalid");
        }
        if self.compute_digest()? != self.plan_digest_sha256 {
            return Err("managed-operation-plan-digest-invalid");
        }
        Ok(())
    }

    fn to_canonical_json(&self) -> Result<String, &'static str> {
        let bytes =
            serde_json_canonicalizer::to_vec(self).map_err(|_| "managed-operation-plan-invalid")?;
        String::from_utf8(bytes).map_err(|_| "managed-operation-plan-invalid")
    }
}

pub(crate) fn verify_native_cli_health_plan(
    output: &str,
    version: &str,
    pack: &str,
    candidate_digest: &str,
) -> Result<(), &'static str> {
    let plan: ManagedOperationPlanV1 =
        serde_json::from_str(output).map_err(|_| "native-activation-health-plan-invalid")?;
    // Health observes the verified candidate, which may be the restored predecessor.
    // Managed writes still validate against this process's own version.
    plan.validate_for_version(now_unix()?, version)?;
    if plan.product_version != version
        || plan.content_pack_sha256 != pack
        || !matches!(&plan.operation, ManagedOperationV1::CliInstall { control_sha256, .. }
            if control_sha256 == candidate_digest)
    {
        return Err("native-activation-health-identity-mismatch");
    }
    Ok(())
}

fn prepare_skills_reconcile_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    preset: ManagedSkillsPresetV1,
    profile: ProfileId,
) -> Result<ManagedOperationPlanV1, &'static str> {
    let observation = observe_preset(environment, content, preset)?;
    if observation.state == ManagedSkillsStateV1::Drifted {
        return Err("managed-skills-target-drifted");
    }
    if observation
        .profile
        .is_some_and(|observed| observed != profile)
    {
        return Err("managed-skills-profile-change-not-supported");
    }
    let target_id = target_id(&observation.target)?;
    let operation = ManagedOperationV1::SkillsReconcilePreset {
        preset,
        target_id: target_id.clone(),
        profile,
        expected_state: observation.state,
        expected_receipt_sha256: observation.receipt_sha256.clone(),
    };
    let semantic_digest_sha256 = skills_semantic_digest(
        "reconcile-preset",
        &target_id,
        profile,
        observation.state,
        observation.receipt_sha256.as_deref(),
        content.pack().pack_sha256(),
        observation.workflow_variant_sha256.as_deref(),
    );
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        operation,
        vec![ManagedOperationApprovalV1::FilesystemWrite],
        semantic_digest_sha256,
    )
}

fn prepare_skills_update_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    target_id: &str,
) -> Result<ManagedOperationPlanV1, &'static str> {
    let observation = observe_registered_target(environment, content, target_id)?;
    if observation.state == ManagedSkillsStateV1::Current {
        return Err("managed-skills-target-already-current");
    }
    if observation.state != ManagedSkillsStateV1::UpdateAvailable {
        return Err("managed-skills-target-not-installed");
    }
    let profile = observation
        .profile
        .ok_or("managed-skills-target-not-installed")?;
    let receipt_sha256 = observation
        .receipt_sha256
        .ok_or("managed-skills-target-not-installed")?;
    let operation = ManagedOperationV1::SkillsUpdateTarget {
        target_id: target_id.to_string(),
        profile,
        expected_state: observation.state,
        expected_receipt_sha256: receipt_sha256.clone(),
    };
    let semantic_digest_sha256 = skills_semantic_digest(
        "update-target",
        target_id,
        profile,
        observation.state,
        Some(&receipt_sha256),
        content.pack().pack_sha256(),
        observation.workflow_variant_sha256.as_deref(),
    );
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        operation,
        vec![ManagedOperationApprovalV1::FilesystemWrite],
        semantic_digest_sha256,
    )
}

fn prepare_skills_remove_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    target_id: &str,
) -> Result<ManagedOperationPlanV1, &'static str> {
    let observation = observe_registered_target(environment, content, target_id)?;
    if !matches!(
        observation.state,
        ManagedSkillsStateV1::Current | ManagedSkillsStateV1::UpdateAvailable
    ) {
        return Err("managed-skills-target-not-removable");
    }
    let profile = observation
        .profile
        .ok_or("managed-skills-target-not-installed")?;
    let receipt_sha256 = observation
        .receipt_sha256
        .ok_or("managed-skills-target-not-installed")?;
    let operation = ManagedOperationV1::SkillsRemoveTarget {
        target_id: target_id.to_string(),
        profile,
        expected_state: observation.state,
        expected_receipt_sha256: receipt_sha256.clone(),
    };
    let semantic_digest_sha256 = skills_semantic_digest(
        "remove-target",
        target_id,
        profile,
        observation.state,
        Some(&receipt_sha256),
        content.pack().pack_sha256(),
        observation.workflow_variant_sha256.as_deref(),
    );
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        operation,
        vec![ManagedOperationApprovalV1::FilesystemWrite],
        semantic_digest_sha256,
    )
}

fn prepare_skills_detach_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    target_id: &str,
) -> Result<ManagedOperationPlanV1, &'static str> {
    let observation = observe_registered_target(environment, content, target_id)?;
    if observation.state != ManagedSkillsStateV1::Drifted {
        return Err("managed-skills-target-not-drifted");
    }
    let profile = observation
        .profile
        .ok_or("managed-skills-target-not-installed")?;
    let receipt_sha256 = observation
        .receipt_sha256
        .ok_or("managed-skills-target-not-installed")?;
    let operation = ManagedOperationV1::SkillsDetachTarget {
        target_id: target_id.to_string(),
        profile,
        expected_state: observation.state,
        expected_receipt_sha256: receipt_sha256.clone(),
    };
    let semantic_digest_sha256 = skills_semantic_digest(
        "detach-target",
        target_id,
        profile,
        observation.state,
        Some(&receipt_sha256),
        content.pack().pack_sha256(),
        observation.workflow_variant_sha256.as_deref(),
    );
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        operation,
        vec![ManagedOperationApprovalV1::FilesystemWrite],
        semantic_digest_sha256,
    )
}

fn prepare_integrations_reconcile_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    targets: &[ManagedIntegrationTargetV1],
    mode: ManagedIntegrationModeV1,
) -> Result<ManagedOperationPlanV1, &'static str> {
    let product = crate::desktop::verify_running_packaged_product(environment, content)?;
    let root = config_root(environment).map_err(|error| error.reason_code())?;
    let workflow_variant = WorkflowVariantStore::new(root)
        .load(content.pack())
        .map_err(|error| error.reason_code())?;
    let native_targets = native_targets(targets)?;
    reject_unsupported_integration_versions(environment, &native_targets)?;
    let preview = preview_packaged_product_batch_install_with_variant(
        &product,
        &native_targets,
        workflow_variant.variant_sha256(),
    )
    .map_err(|error| error.reason_code())?;
    if preview
        .installs
        .iter()
        .any(|install| install.effect == PackagedProductInstallEffect::RecoveryRequired)
    {
        return Err("packaged-product-recovery-required");
    }
    let installs = preview
        .installs
        .iter()
        .map(|install| managed_install_preview(&product, install))
        .collect::<Result<Vec<_>, &'static str>>()?;
    validate_integration_mode(mode, &installs)?;
    let host_plans = prepare_host_plugin_plans(
        environment,
        &preview
            .installs
            .iter()
            .map(|install| (install.target, host_install_effect(install.effect)))
            .collect::<Vec<_>>(),
    )?;
    let host_plan_digest_sha256 = host_plugin_plans_digest(&host_plans);
    let semantic_digest_sha256 = integration_reconcile_digest(
        mode,
        product.control_sha256(),
        &preview.plan_digest_sha256,
        &installs,
        &host_plan_digest_sha256,
    );
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        ManagedOperationV1::IntegrationsReconcile {
            mode,
            control_sha256: product.control_sha256().to_string(),
            native_batch_plan_digest_sha256: preview.plan_digest_sha256,
            installs,
        },
        integration_approvals(),
        semantic_digest_sha256,
    )
}

fn prepare_integrations_remove_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    targets: &[ManagedIntegrationTargetV1],
) -> Result<ManagedOperationPlanV1, &'static str> {
    let product = crate::desktop::verify_running_packaged_product(environment, content)?;
    let verifications = native_targets(targets)?
        .into_iter()
        .map(|target| {
            let verification = verify_receipt_owned_packaged_product_install(&product, target)
                .map_err(|error| error.reason_code())?;
            Ok(ManagedIntegrationVerificationV1 {
                target: managed_target(target),
                evidence_digest_sha256: integration_verification_digest(&verification)?,
            })
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    let semantic_digest_sha256 =
        integration_remove_digest(product.control_sha256(), &verifications);
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        ManagedOperationV1::IntegrationsRemove {
            control_sha256: product.control_sha256().to_string(),
            verifications,
        },
        integration_approvals(),
        semantic_digest_sha256,
    )
}

fn prepare_cli_install_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<ManagedOperationPlanV1, &'static str> {
    let product = crate::desktop::verify_running_packaged_product(environment, content)?;
    let home = environment
        .platform_home()
        .ok_or("qiongli-cli-home-unavailable")?;
    let native = preview_cli_install(
        home,
        product.current_executable(),
        env!("CARGO_PKG_VERSION"),
    )?;
    let semantic_digest_sha256 = cli_install_digest(product.control_sha256(), native.plan_sha256());
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        ManagedOperationV1::CliInstall {
            control_sha256: product.control_sha256().to_string(),
            native_plan_digest_sha256: native.plan_sha256().to_string(),
        },
        vec![ManagedOperationApprovalV1::FilesystemWrite],
        semantic_digest_sha256,
    )
}

fn prepare_cli_remove_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<ManagedOperationPlanV1, &'static str> {
    let product = crate::desktop::verify_running_packaged_product(environment, content)?;
    let home = environment
        .platform_home()
        .ok_or("qiongli-cli-home-unavailable")?;
    let path = std::env::var_os("PATH");
    let shell = std::env::var_os("SHELL");
    let native = preview_cli_remove(home, path.as_deref(), shell.as_deref())?;
    let semantic_digest_sha256 = cli_lifecycle_digest(
        b"REMOVE",
        product.control_sha256(),
        native.plan_sha256(),
        None,
    );
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        ManagedOperationV1::CliRemove {
            control_sha256: product.control_sha256().to_string(),
            native_plan_digest_sha256: native.plan_sha256().to_string(),
        },
        vec![ManagedOperationApprovalV1::FilesystemWrite],
        semantic_digest_sha256,
    )
}

fn prepare_cli_path_configure_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<ManagedOperationPlanV1, &'static str> {
    let product = crate::desktop::verify_running_packaged_product(environment, content)?;
    let home = environment
        .platform_home()
        .ok_or("qiongli-cli-home-unavailable")?;
    let shell = std::env::var_os("SHELL");
    let native = preview_cli_path_configure(home, shell.as_deref())?;
    let semantic_digest_sha256 = cli_lifecycle_digest(
        b"PATH-CONFIGURE",
        product.control_sha256(),
        native.plan_sha256(),
        Some(native.profile_name()),
    );
    ManagedOperationPlanV1::new(
        content,
        now_unix()?,
        ManagedOperationV1::CliPathConfigure {
            control_sha256: product.control_sha256().to_string(),
            native_plan_digest_sha256: native.plan_sha256().to_string(),
            profile_name: native.profile_name().to_string(),
        },
        vec![ManagedOperationApprovalV1::FilesystemWrite],
        semantic_digest_sha256,
    )
}

fn apply_integration_content_update(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    product: &VerifiedPackagedProduct,
    workflow_variant: &LoadedWorkflowVariant,
    targets: (&[ClientActivationTarget], &[ClientActivationTarget]),
    host_plans: &[HostPluginPlan],
    plan_digest_sha256: &str,
) -> Result<(), &'static str> {
    let (selected_targets, content_update_targets) = targets;
    if !valid_sha256(plan_digest_sha256) {
        return Err("managed-operation-plan-digest-invalid");
    }
    let claude_config_root = environment
        .claude_config_root()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| product.home().join(".claude"));
    let codex_grant = product
        .capability(ClientActivationTarget::Codex)
        .map(|capability| capability.grant())
        .ok_or("packaged-product-authority-unavailable")?;
    let claude_grant = product
        .capability(ClientActivationTarget::ClaudeCode)
        .map(|capability| capability.grant())
        .ok_or("packaged-product-authority-unavailable")?;
    let reconciliation_now_unix = now_unix()?;
    let store = update_store(environment)?;
    let transaction_id = format!("update-{}", &plan_digest_sha256[..32]);
    crate::update_reconcile::prepare_reconciliation_transaction_root(&store, &transaction_id)?;
    let discard = || {
        let _ = crate::update_reconcile::discard_prepared_reconciliation(&store, &transaction_id);
        let _ = crate::update_reconcile::remove_reconciliation_transaction_root(
            &store,
            &transaction_id,
        );
    };
    let prepared = match crate::update_reconcile::prepare_update_reconciliation(
        &crate::update_reconcile::ReconciliationPreparation {
            cli_update: None,
            native_release: None,
            store: &store,
            transaction_id: &transaction_id,
            target_version: &product.artifact().version,
            content,
            platform_home: product.home(),
            claude_config_root: &claude_config_root,
            source_binary: product.current_executable(),
            codex_grant,
            claude_grant,
            workflow_overrides: workflow_variant.overrides(),
            targets: content_update_targets,
            now_unix: reconciliation_now_unix,
        },
    ) {
        Ok(prepared) => prepared,
        Err(code) => {
            discard();
            return Err(code);
        }
    };
    let journal =
        match crate::update_reconcile::load_reconciliation_journal(&store, &transaction_id) {
            Ok(journal) => journal,
            Err(code) => {
                discard();
                return Err(code);
            }
        };
    let journal_sha256 = match crate::update_reconcile::reconciliation_journal_sha256(&journal) {
        Ok(digest) => digest,
        Err(code) => {
            discard();
            return Err(code);
        }
    };
    if journal_sha256 != prepared.journal_sha256
        || crate::update_reconcile::verify_prepared_reconciliation(&journal).is_err()
    {
        discard();
        return Err("native-update-reconciliation-invalid");
    }
    if let Err(code) = crate::update_reconcile::activate_prepared_reconciliation(&journal) {
        discard();
        return Err(code);
    }
    let readiness = execute_host_plugin_plan_set(environment, host_plans).and_then(|()| {
        for target in selected_targets {
            qiongli_platform::verify_packaged_product_install_with_variant(
                product,
                *target,
                workflow_variant.variant_sha256(),
            )
            .map_err(|error| error.reason_code())?;
        }
        Ok(())
    });
    if let Err(code) = readiness {
        let recovered = crate::update_reconcile::rollback_active_reconciliation(&journal)
            .and_then(|()| crate::update_reconcile::cleanup_rolled_back_reconciliation(&journal))
            .and_then(|()| {
                crate::update_reconcile::remove_committed_reconciliation_journal(
                    &store,
                    &transaction_id,
                )
            })
            .and_then(|()| {
                crate::update_reconcile::remove_reconciliation_transaction_root(
                    &store,
                    &transaction_id,
                )
            });
        return Err(if recovered.is_ok() {
            code
        } else {
            "native-update-reconciliation-recovery-required"
        });
    }
    crate::update_reconcile::cleanup_committed_reconciliation(&journal)
        .and_then(|()| {
            crate::update_reconcile::remove_committed_reconciliation_journal(
                &store,
                &transaction_id,
            )
        })
        .and_then(|()| {
            crate::update_reconcile::remove_reconciliation_transaction_root(&store, &transaction_id)
        })
}

fn apply_plan(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    plan_path: &Path,
    expected_plan_digest: &str,
    approvals: &[(ManagedOperationApprovalV1, bool)],
) -> Result<String, &'static str> {
    if !valid_sha256(expected_plan_digest) {
        return Err("managed-operation-plan-digest-invalid");
    }
    let validation_now_unix = now_unix()?;
    let plan = read_plan(plan_path)?;
    plan.validate(validation_now_unix)?;
    if plan.plan_digest_sha256 != expected_plan_digest {
        return Err("managed-operation-plan-digest-mismatch");
    }
    if plan.content_pack_sha256 != content.pack().pack_sha256()
        || plan.content_root_sha256 != content.pack().manifest().content_root_sha256
    {
        return Err("managed-operation-product-changed");
    }
    validate_approvals(&plan.approvals_required, approvals)?;
    let root = config_root(environment).map_err(|error| error.reason_code())?;
    let _write_guard = crate::update_reconcile::acquire_managed_write_guard(
        environment
            .platform_home()
            .ok_or("native-candidate-home-unavailable")?,
        root.clone(),
    )?;
    let result = match &plan.operation {
        ManagedOperationV1::SkillsReconcilePreset {
            preset,
            target_id,
            profile,
            expected_state,
            expected_receipt_sha256,
        } => {
            let workflow_variant = WorkflowVariantStore::new(root.clone())
                .load(content.pack())
                .map_err(|error| error.reason_code())?;
            let observation = observe_preset(environment, content, *preset)?;
            validate_observation(
                &observation,
                target_id,
                *expected_state,
                expected_receipt_sha256.as_deref(),
            )?;
            validate_workflow_variant_observation(workflow_variant.variant_sha256(), &observation)?;
            let semantic = skills_semantic_digest(
                "reconcile-preset",
                target_id,
                *profile,
                *expected_state,
                expected_receipt_sha256.as_deref(),
                content.pack().pack_sha256(),
                observation.workflow_variant_sha256.as_deref(),
            );
            if semantic != plan.semantic_digest_sha256 {
                return Err("managed-operation-precondition-changed");
            }
            let receipt = apply_managed_materialization_with_overrides(
                &root,
                content,
                &observation.target,
                *profile,
                workflow_variant.overrides(),
            )?;
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: "skills-reconcile-preset",
                targets: vec![target_id.clone()],
                result: if *expected_state == ManagedSkillsStateV1::Missing {
                    "installed"
                } else if *expected_state == ManagedSkillsStateV1::Current {
                    "already-current"
                } else {
                    "updated"
                },
                receipt_sha256: Some(materialization_receipt_sha256(&receipt)?),
            }
        }
        ManagedOperationV1::SkillsUpdateTarget {
            target_id,
            profile,
            expected_state,
            expected_receipt_sha256,
        } => {
            let workflow_variant = WorkflowVariantStore::new(root.clone())
                .load(content.pack())
                .map_err(|error| error.reason_code())?;
            let observation = observe_registered_target(environment, content, target_id)?;
            if observation.profile != Some(*profile) {
                return Err("managed-operation-precondition-changed");
            }
            validate_observation(
                &observation,
                target_id,
                *expected_state,
                Some(expected_receipt_sha256),
            )?;
            validate_workflow_variant_observation(workflow_variant.variant_sha256(), &observation)?;
            let semantic = skills_semantic_digest(
                "update-target",
                target_id,
                *profile,
                *expected_state,
                Some(expected_receipt_sha256),
                content.pack().pack_sha256(),
                observation.workflow_variant_sha256.as_deref(),
            );
            if semantic != plan.semantic_digest_sha256 {
                return Err("managed-operation-precondition-changed");
            }
            let receipt = apply_managed_materialization_with_overrides(
                &root,
                content,
                &observation.target,
                *profile,
                workflow_variant.overrides(),
            )?;
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: "skills-update-target",
                targets: vec![target_id.clone()],
                result: "updated",
                receipt_sha256: Some(materialization_receipt_sha256(&receipt)?),
            }
        }
        ManagedOperationV1::SkillsRemoveTarget {
            target_id,
            profile,
            expected_state,
            expected_receipt_sha256,
        } => {
            let workflow_variant = WorkflowVariantStore::new(root.clone())
                .load(content.pack())
                .map_err(|error| error.reason_code())?;
            let observation = observe_registered_target(environment, content, target_id)?;
            if observation.profile != Some(*profile) {
                return Err("managed-operation-precondition-changed");
            }
            validate_observation(
                &observation,
                target_id,
                *expected_state,
                Some(expected_receipt_sha256),
            )?;
            validate_workflow_variant_observation(workflow_variant.variant_sha256(), &observation)?;
            let semantic = skills_semantic_digest(
                "remove-target",
                target_id,
                *profile,
                *expected_state,
                Some(expected_receipt_sha256),
                content.pack().pack_sha256(),
                observation.workflow_variant_sha256.as_deref(),
            );
            if semantic != plan.semantic_digest_sha256 {
                return Err("managed-operation-precondition-changed");
            }
            let expected_receipt = observation
                .receipt
                .ok_or("managed-skills-target-not-installed")?;
            let removed = remove_managed_materialization_with_overrides(
                root.state_root(),
                content,
                &observation.target,
                &expected_receipt,
                workflow_variant.overrides(),
            )?;
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: "skills-remove-target",
                targets: vec![target_id.clone()],
                result: "removed",
                receipt_sha256: Some(materialization_receipt_sha256(&removed)?),
            }
        }
        ManagedOperationV1::SkillsDetachTarget {
            target_id,
            profile,
            expected_state,
            expected_receipt_sha256,
        } => {
            let workflow_variant = WorkflowVariantStore::new(root.clone())
                .load(content.pack())
                .map_err(|error| error.reason_code())?;
            let observation = observe_registered_target(environment, content, target_id)?;
            if observation.profile != Some(*profile) {
                return Err("managed-operation-precondition-changed");
            }
            validate_observation(
                &observation,
                target_id,
                *expected_state,
                Some(expected_receipt_sha256),
            )?;
            validate_workflow_variant_observation(workflow_variant.variant_sha256(), &observation)?;
            let semantic = skills_semantic_digest(
                "detach-target",
                target_id,
                *profile,
                *expected_state,
                Some(expected_receipt_sha256),
                content.pack().pack_sha256(),
                observation.workflow_variant_sha256.as_deref(),
            );
            if semantic != plan.semantic_digest_sha256 {
                return Err("managed-operation-precondition-changed");
            }
            detach_managed_materialization(
                root.state_root(),
                &observation.target,
                expected_receipt_sha256,
            )?;
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: "skills-detach-target",
                targets: vec![target_id.clone()],
                result: "detached-preserved",
                receipt_sha256: Some(expected_receipt_sha256.clone()),
            }
        }
        ManagedOperationV1::IntegrationsReconcile {
            mode,
            control_sha256,
            native_batch_plan_digest_sha256,
            installs,
        } => {
            let product = crate::desktop::verify_running_packaged_product(environment, content)?;
            let workflow_variant = WorkflowVariantStore::new(root.clone())
                .load(content.pack())
                .map_err(|error| error.reason_code())?;
            if product.control_sha256() != control_sha256 {
                return Err("managed-operation-product-changed");
            }
            let targets = installs
                .iter()
                .map(|install| native_target(install.target))
                .collect::<Vec<_>>();
            reject_unsupported_integration_versions(environment, &targets)
                .map_err(|_| "managed-operation-precondition-changed")?;
            let preview = preview_packaged_product_batch_install_with_variant(
                &product,
                &targets,
                workflow_variant.variant_sha256(),
            )
            .map_err(|error| error.reason_code())?;
            let current_installs = preview
                .installs
                .iter()
                .map(|install| managed_install_preview(&product, install))
                .collect::<Result<Vec<_>, &'static str>>()?;
            validate_integration_mode(*mode, &current_installs)
                .map_err(|_| "managed-operation-precondition-changed")?;
            let host_plans = prepare_host_plugin_plans(
                environment,
                &preview
                    .installs
                    .iter()
                    .map(|install| (install.target, host_install_effect(install.effect)))
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| "managed-operation-precondition-changed")?;
            let host_plan_digest_sha256 = host_plugin_plans_digest(&host_plans);
            let content_updates = preview
                .installs
                .iter()
                .filter(|install| install.effect == PackagedProductInstallEffect::ReplaceRequired)
                .map(|install| install.target)
                .collect::<Vec<_>>();
            if (!preview.can_apply && content_updates.is_empty())
                || preview.plan_digest_sha256 != *native_batch_plan_digest_sha256
                || current_installs != *installs
                || integration_reconcile_digest(
                    *mode,
                    control_sha256,
                    native_batch_plan_digest_sha256,
                    installs,
                    &host_plan_digest_sha256,
                ) != plan.semantic_digest_sha256
            {
                return Err("managed-operation-precondition-changed");
            }
            let all_current = if content_updates.is_empty() {
                // Product verification binds every launch grant to its own verification
                // timestamp. Re-sample after that boundary so a wall-clock second rollover
                // cannot make the native activation plan appear older than its verified grant.
                let activation_now_unix = now_unix()?;
                let commit = apply_packaged_product_batch_install_with_overrides(
                    content.pack(),
                    &product,
                    &preview,
                    activation_now_unix,
                    workflow_variant.overrides(),
                )
                .map_err(|error| error.reason_code())?;
                execute_host_plugin_plan_set(environment, &host_plans)?;
                commit.installs.iter().all(|install| {
                    install.disposition
                        == qiongli_platform::PackagedProductInstallDisposition::AlreadyCurrent
                })
            } else {
                apply_integration_content_update(
                    environment,
                    content,
                    &product,
                    &workflow_variant,
                    (&targets, &content_updates),
                    &host_plans,
                    &plan.plan_digest_sha256,
                )?;
                false
            };
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: match mode {
                    ManagedIntegrationModeV1::Install => "integrations-install",
                    ManagedIntegrationModeV1::Repair => "integrations-reconcile",
                },
                targets: installs
                    .iter()
                    .map(|install| integration_target_name(install.target).to_string())
                    .collect(),
                result: if all_current {
                    "already-current"
                } else {
                    "reconciled"
                },
                receipt_sha256: None,
            }
        }
        ManagedOperationV1::IntegrationsRemove {
            control_sha256,
            verifications,
        } => {
            let product = crate::desktop::verify_running_packaged_product(environment, content)?;
            if product.control_sha256() != control_sha256 {
                return Err("managed-operation-product-changed");
            }
            let current = verifications
                .iter()
                .map(|expected| {
                    let target = native_target(expected.target);
                    let verification =
                        verify_receipt_owned_packaged_product_install(&product, target)
                            .map_err(|error| error.reason_code())?;
                    Ok(ManagedIntegrationVerificationV1 {
                        target: expected.target,
                        evidence_digest_sha256: integration_verification_digest(&verification)?,
                    })
                })
                .collect::<Result<Vec<_>, &'static str>>()?;
            if current != *verifications
                || integration_remove_digest(control_sha256, verifications)
                    != plan.semantic_digest_sha256
            {
                return Err("managed-operation-precondition-changed");
            }
            let removal_now_unix = now_unix()?;
            for verification in verifications {
                remove_packaged_product_install(
                    &product,
                    native_target(verification.target),
                    removal_now_unix,
                )
                .map_err(|error| error.reason_code())?;
            }
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: "integrations-remove",
                targets: verifications
                    .iter()
                    .map(|verification| integration_target_name(verification.target).to_string())
                    .collect(),
                result: "removed",
                receipt_sha256: None,
            }
        }
        ManagedOperationV1::CliInstall {
            control_sha256,
            native_plan_digest_sha256,
        } => {
            let product = crate::desktop::verify_running_packaged_product(environment, content)?;
            if product.control_sha256() != control_sha256 {
                return Err("managed-operation-product-changed");
            }
            let home = environment
                .platform_home()
                .ok_or("qiongli-cli-home-unavailable")?;
            let native = preview_cli_install(
                home,
                product.current_executable(),
                env!("CARGO_PKG_VERSION"),
            )?;
            if native.plan_sha256() != native_plan_digest_sha256
                || cli_install_digest(control_sha256, native_plan_digest_sha256)
                    != plan.semantic_digest_sha256
            {
                return Err("managed-operation-precondition-changed");
            }
            let result = apply_cli_install(&native)?;
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: "cli-install",
                targets: vec!["qiongli-cli".to_string()],
                result: match result {
                    "qiongli-cli-installed" => "installed",
                    "qiongli-cli-updated" => "updated",
                    _ => return Err("qiongli-cli-install-result-invalid"),
                },
                receipt_sha256: None,
            }
        }
        ManagedOperationV1::CliRemove {
            control_sha256,
            native_plan_digest_sha256,
        } => {
            let product = crate::desktop::verify_running_packaged_product(environment, content)?;
            if product.control_sha256() != control_sha256 {
                return Err("managed-operation-product-changed");
            }
            let home = environment
                .platform_home()
                .ok_or("qiongli-cli-home-unavailable")?;
            let path = std::env::var_os("PATH");
            let shell = std::env::var_os("SHELL");
            let native = preview_cli_remove(home, path.as_deref(), shell.as_deref())?;
            if native.plan_sha256() != native_plan_digest_sha256
                || cli_lifecycle_digest(b"REMOVE", control_sha256, native_plan_digest_sha256, None)
                    != plan.semantic_digest_sha256
            {
                return Err("managed-operation-precondition-changed");
            }
            let effect = apply_cli_remove(&native)?;
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: "cli-remove",
                targets: vec!["qiongli-cli".to_string()],
                result: match effect {
                    CliRemovalEffect::Removed => "removed",
                    CliRemovalEffect::RestoredPredecessor => "restored-predecessor",
                },
                receipt_sha256: None,
            }
        }
        ManagedOperationV1::CliPathConfigure {
            control_sha256,
            native_plan_digest_sha256,
            profile_name,
        } => {
            let product = crate::desktop::verify_running_packaged_product(environment, content)?;
            if product.control_sha256() != control_sha256 {
                return Err("managed-operation-product-changed");
            }
            let home = environment
                .platform_home()
                .ok_or("qiongli-cli-home-unavailable")?;
            let shell = std::env::var_os("SHELL");
            let native = preview_cli_path_configure(home, shell.as_deref())?;
            if native.plan_sha256() != native_plan_digest_sha256
                || native.profile_name() != profile_name
                || cli_lifecycle_digest(
                    b"PATH-CONFIGURE",
                    control_sha256,
                    native_plan_digest_sha256,
                    Some(profile_name),
                ) != plan.semantic_digest_sha256
            {
                return Err("managed-operation-precondition-changed");
            }
            if apply_cli_path_configure(&native)? != "qiongli-cli-path-configured" {
                return Err("qiongli-cli-path-result-invalid");
            }
            ManagedOperationResultV1 {
                schema_version: 1,
                command: "app-apply",
                operation: "cli-path-configure",
                targets: vec![format!("<user-home>/{profile_name}")],
                result: "configured",
                receipt_sha256: None,
            }
        }
    };
    canonical_json(&result)
}

fn observe_preset(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    preset: ManagedSkillsPresetV1,
) -> Result<ManagedSkillsObservation, &'static str> {
    let path = match preset {
        ManagedSkillsPresetV1::QiongliManaged => environment
            .platform_home()
            .ok_or("managed-skills-home-unavailable")?
            .join(".qiongli-skills"),
        ManagedSkillsPresetV1::CurrentProject => environment
            .project_root()
            .ok_or("managed-skills-project-unavailable")?
            .join(".qiongli-skills"),
    };
    observe_path(environment, content, &path)
}

fn observe_registered_target(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    target_id: &str,
) -> Result<ManagedSkillsObservation, &'static str> {
    validate_target_id(target_id)?;
    let root = config_root(environment).map_err(|error| error.reason_code())?;
    let registry = load_managed_content_registry(root.state_root())?;
    let variant = WorkflowVariantStore::new(root)
        .load(content.pack())
        .map_err(|error| error.reason_code())?;
    let mut matches = registry
        .entries
        .iter()
        .filter(|entry| managed_skills_target_id(&entry.target) == target_id);
    let entry = matches
        .next()
        .ok_or("managed-skills-target-not-registered")?;
    if matches.next().is_some() {
        return Err("managed-skills-target-ambiguous");
    }
    observe_entry(content, entry, variant.variant_sha256())
}

fn observe_path(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    path: &Path,
) -> Result<ManagedSkillsObservation, &'static str> {
    let target = approve_materialization_target(path).map_err(|error| error.reason_code())?;
    let root = config_root(environment).map_err(|error| error.reason_code())?;
    let registry = load_managed_content_registry(root.state_root())?;
    let variant = WorkflowVariantStore::new(root)
        .load(content.pack())
        .map_err(|error| error.reason_code())?;
    let path_text = target
        .path()
        .to_str()
        .ok_or("managed-content-target-invalid")?;
    match registry
        .entries
        .binary_search_by(|entry| entry.target.as_str().cmp(path_text))
    {
        Ok(index) => observe_entry(content, &registry.entries[index], variant.variant_sha256()),
        Err(_) => {
            if target.path().exists() && verify_materialization(&target).is_ok() {
                return Err("managed-skills-target-not-registered");
            }
            if target.path().exists()
                && fs::read_dir(target.path())
                    .map_err(|_| "managed-skills-target-unavailable")?
                    .next()
                    .is_some()
            {
                return Err("unmanaged-materialization-target");
            }
            Ok(ManagedSkillsObservation {
                target,
                profile: None,
                state: ManagedSkillsStateV1::Missing,
                receipt: None,
                receipt_sha256: None,
                workflow_variant_sha256: variant.variant_sha256().map(str::to_owned),
            })
        }
    }
}

fn observe_entry(
    content: &EmbeddedContent,
    entry: &ManagedContentEntryV1,
    expected_variant_sha256: Option<&str>,
) -> Result<ManagedSkillsObservation, &'static str> {
    match observe_managed_skills_entry_with_variant(content, entry, expected_variant_sha256) {
        Ok(observed) => Ok(ManagedSkillsObservation {
            target: observed.target,
            profile: Some(entry.profile),
            state: match observed.state {
                ManagedSkillsEntryState::Current => ManagedSkillsStateV1::Current,
                ManagedSkillsEntryState::UpdateAvailable => ManagedSkillsStateV1::UpdateAvailable,
            },
            receipt: Some(observed.receipt),
            receipt_sha256: Some(observed.receipt_sha256),
            workflow_variant_sha256: expected_variant_sha256.map(str::to_owned),
        }),
        Err("managed-skills-target-drifted") => Ok(ManagedSkillsObservation {
            target: approve_materialization_target(Path::new(&entry.target))
                .map_err(|error| error.reason_code())?,
            profile: Some(entry.profile),
            state: ManagedSkillsStateV1::Drifted,
            receipt: None,
            receipt_sha256: Some(entry.receipt_sha256.clone()),
            workflow_variant_sha256: expected_variant_sha256.map(str::to_owned),
        }),
        Err(code) => Err(code),
    }
}

fn validate_observation(
    observation: &ManagedSkillsObservation,
    expected_target_id: &str,
    expected_state: ManagedSkillsStateV1,
    expected_receipt_sha256: Option<&str>,
) -> Result<(), &'static str> {
    if target_id(&observation.target)? != expected_target_id
        || observation.state != expected_state
        || observation.receipt_sha256.as_deref() != expected_receipt_sha256
    {
        return Err("managed-operation-precondition-changed");
    }
    Ok(())
}

fn validate_workflow_variant_observation(
    current_variant_sha256: Option<&str>,
    observation: &ManagedSkillsObservation,
) -> Result<(), &'static str> {
    (current_variant_sha256 == observation.workflow_variant_sha256.as_deref())
        .then_some(())
        .ok_or("managed-operation-precondition-changed")
}

fn validate_operation(operation: &ManagedOperationV1) -> Result<(), &'static str> {
    match operation {
        ManagedOperationV1::SkillsReconcilePreset {
            target_id,
            expected_state,
            expected_receipt_sha256,
            ..
        } => {
            validate_target_id(target_id)?;
            if *expected_state == ManagedSkillsStateV1::Drifted
                || ((*expected_state == ManagedSkillsStateV1::Missing)
                    != expected_receipt_sha256.is_none())
            {
                return Err("managed-operation-plan-invalid");
            }
            if expected_receipt_sha256
                .as_deref()
                .is_some_and(|digest| !valid_sha256(digest))
            {
                return Err("managed-operation-plan-invalid");
            }
        }
        ManagedOperationV1::CliInstall {
            control_sha256,
            native_plan_digest_sha256,
        } => {
            if !valid_sha256(control_sha256) || !valid_sha256(native_plan_digest_sha256) {
                return Err("managed-operation-plan-invalid");
            }
        }
        ManagedOperationV1::CliRemove {
            control_sha256,
            native_plan_digest_sha256,
        } => {
            if !valid_sha256(control_sha256) || !valid_sha256(native_plan_digest_sha256) {
                return Err("managed-operation-plan-invalid");
            }
        }
        ManagedOperationV1::CliPathConfigure {
            control_sha256,
            native_plan_digest_sha256,
            profile_name,
        } => {
            if !valid_sha256(control_sha256)
                || !valid_sha256(native_plan_digest_sha256)
                || !matches!(
                    profile_name.as_str(),
                    ".zprofile" | ".bash_profile" | ".profile"
                )
            {
                return Err("managed-operation-plan-invalid");
            }
        }
        ManagedOperationV1::SkillsUpdateTarget {
            target_id,
            expected_state,
            expected_receipt_sha256,
            ..
        } => {
            validate_target_id(target_id)?;
            if *expected_state != ManagedSkillsStateV1::UpdateAvailable
                || !valid_sha256(expected_receipt_sha256)
            {
                return Err("managed-operation-plan-invalid");
            }
        }
        ManagedOperationV1::SkillsRemoveTarget {
            target_id,
            expected_state,
            expected_receipt_sha256,
            ..
        } => {
            validate_target_id(target_id)?;
            if !matches!(
                expected_state,
                ManagedSkillsStateV1::Current | ManagedSkillsStateV1::UpdateAvailable
            ) || !valid_sha256(expected_receipt_sha256)
            {
                return Err("managed-operation-plan-invalid");
            }
        }
        ManagedOperationV1::SkillsDetachTarget {
            target_id,
            expected_state,
            expected_receipt_sha256,
            ..
        } => {
            validate_target_id(target_id)?;
            if *expected_state != ManagedSkillsStateV1::Drifted
                || !valid_sha256(expected_receipt_sha256)
            {
                return Err("managed-operation-plan-invalid");
            }
        }
        ManagedOperationV1::IntegrationsReconcile {
            mode,
            control_sha256,
            native_batch_plan_digest_sha256,
            installs,
        } => {
            if !valid_sha256(control_sha256)
                || !valid_sha256(native_batch_plan_digest_sha256)
                || installs.is_empty()
                || installs.len() > 2
                || !ordered_unique_targets(installs.iter().map(|install| install.target))
                || installs
                    .iter()
                    .any(|install| !valid_sha256(&install.native_plan_digest_sha256))
                || validate_integration_mode(*mode, installs).is_err()
            {
                return Err("managed-operation-plan-invalid");
            }
        }
        ManagedOperationV1::IntegrationsRemove {
            control_sha256,
            verifications,
        } => {
            if !valid_sha256(control_sha256)
                || verifications.is_empty()
                || verifications.len() > 2
                || !ordered_unique_targets(
                    verifications.iter().map(|verification| verification.target),
                )
                || verifications
                    .iter()
                    .any(|verification| !valid_sha256(&verification.evidence_digest_sha256))
            {
                return Err("managed-operation-plan-invalid");
            }
        }
    }
    Ok(())
}

fn expected_approvals(operation: &ManagedOperationV1) -> Vec<ManagedOperationApprovalV1> {
    match operation {
        ManagedOperationV1::SkillsReconcilePreset { .. }
        | ManagedOperationV1::SkillsUpdateTarget { .. }
        | ManagedOperationV1::SkillsRemoveTarget { .. }
        | ManagedOperationV1::SkillsDetachTarget { .. }
        | ManagedOperationV1::CliInstall { .. }
        | ManagedOperationV1::CliRemove { .. }
        | ManagedOperationV1::CliPathConfigure { .. } => {
            vec![ManagedOperationApprovalV1::FilesystemWrite]
        }
        ManagedOperationV1::IntegrationsReconcile { .. }
        | ManagedOperationV1::IntegrationsRemove { .. } => integration_approvals(),
    }
}

fn validate_approvals(
    required: &[ManagedOperationApprovalV1],
    supplied: &[(ManagedOperationApprovalV1, bool)],
) -> Result<(), &'static str> {
    for required_approval in required {
        if !supplied
            .iter()
            .any(|(approval, supplied)| approval == required_approval && *supplied)
        {
            return Err("managed-operation-approval-required");
        }
    }
    if supplied
        .iter()
        .any(|(approval, supplied)| *supplied && !required.contains(approval))
    {
        return Err("managed-operation-approval-unexpected");
    }
    Ok(())
}

fn read_plan(path: &Path) -> Result<ManagedOperationPlanV1, &'static str> {
    validate_plan_path(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| "managed-operation-plan-unavailable")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_PLAN_BYTES {
        return Err("managed-operation-plan-invalid");
    }
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
    open_plan(path)?
        .take(MAX_PLAN_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "managed-operation-plan-unavailable")?;
    if bytes.len() as u64 > MAX_PLAN_BYTES {
        return Err("managed-operation-plan-invalid");
    }
    let plan: ManagedOperationPlanV1 =
        serde_json::from_slice(&bytes).map_err(|_| "managed-operation-plan-invalid")?;
    let canonical =
        serde_json_canonicalizer::to_vec(&plan).map_err(|_| "managed-operation-plan-invalid")?;
    if bytes != canonical && bytes != [canonical.as_slice(), b"\n"].concat() {
        return Err("managed-operation-plan-noncanonical");
    }
    Ok(plan)
}

fn open_plan(path: &Path) -> Result<File, &'static str> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        OpenOptions::new()
            .read(true)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(path)
            .map_err(|_| "managed-operation-plan-unavailable")
    }
    #[cfg(not(unix))]
    {
        File::open(path).map_err(|_| "managed-operation-plan-unavailable")
    }
}

fn validate_plan_path(path: &Path) -> Result<(), &'static str> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err("managed-operation-plan-path-invalid");
    }
    Ok(())
}

fn target_id(target: &MaterializationTarget) -> Result<String, &'static str> {
    let path = target
        .path()
        .to_str()
        .ok_or("managed-content-target-invalid")?;
    Ok(managed_skills_target_id(path))
}

fn validate_target_id(target_id: &str) -> Result<(), &'static str> {
    target_id
        .strip_prefix("skills-target-")
        .filter(|digest| valid_sha256(digest))
        .map(|_| ())
        .ok_or("managed-skills-target-id-invalid")
}

fn native_targets(
    targets: &[ManagedIntegrationTargetV1],
) -> Result<Vec<ClientActivationTarget>, &'static str> {
    if !ordered_unique_targets(targets.iter().copied()) {
        return Err("managed-integration-targets-invalid");
    }
    Ok(targets.iter().copied().map(native_target).collect())
}

const fn native_target(target: ManagedIntegrationTargetV1) -> ClientActivationTarget {
    match target {
        ManagedIntegrationTargetV1::Codex => ClientActivationTarget::Codex,
        ManagedIntegrationTargetV1::ClaudeCode => ClientActivationTarget::ClaudeCode,
    }
}

const fn managed_target(target: ClientActivationTarget) -> ManagedIntegrationTargetV1 {
    match target {
        ClientActivationTarget::Codex => ManagedIntegrationTargetV1::Codex,
        ClientActivationTarget::ClaudeCode => ManagedIntegrationTargetV1::ClaudeCode,
    }
}

fn reject_unsupported_integration_versions(
    environment: &CommandEnvironment,
    targets: &[ClientActivationTarget],
) -> Result<(), &'static str> {
    if targets.iter().copied().any(|target| {
        crate::desktop::managed_integration_version_is_unsupported(environment, target)
    }) {
        Err("integration-client-version-unsupported")
    } else {
        Ok(())
    }
}

fn managed_install_preview(
    product: &VerifiedPackagedProduct,
    install: &PackagedProductInstallPreview,
) -> Result<ManagedIntegrationInstallPreviewV1, &'static str> {
    let (effect, native_plan_digest_sha256) = match install.effect {
        PackagedProductInstallEffect::Install => (
            ManagedIntegrationEffectV1::Install,
            install.plan_digest_sha256.clone(),
        ),
        PackagedProductInstallEffect::Repair => (
            ManagedIntegrationEffectV1::Repair,
            install.plan_digest_sha256.clone(),
        ),
        PackagedProductInstallEffect::AlreadyCurrent => (
            ManagedIntegrationEffectV1::AlreadyCurrent,
            install.plan_digest_sha256.clone(),
        ),
        PackagedProductInstallEffect::ReplaceRequired => {
            let verification =
                verify_receipt_owned_packaged_product_install(product, install.target)
                    .map_err(|error| error.reason_code())?;
            (
                ManagedIntegrationEffectV1::ContentUpdate,
                integration_content_update_digest(
                    &install.plan_digest_sha256,
                    &integration_verification_digest(&verification)?,
                ),
            )
        }
        PackagedProductInstallEffect::RecoveryRequired => {
            return Err("packaged-product-recovery-required");
        }
    };
    Ok(ManagedIntegrationInstallPreviewV1 {
        target: managed_target(install.target),
        effect,
        native_plan_digest_sha256,
    })
}

const fn host_install_effect(effect: PackagedProductInstallEffect) -> PackagedProductInstallEffect {
    match effect {
        PackagedProductInstallEffect::ReplaceRequired => PackagedProductInstallEffect::Repair,
        _ => effect,
    }
}

fn validate_integration_mode(
    mode: ManagedIntegrationModeV1,
    installs: &[ManagedIntegrationInstallPreviewV1],
) -> Result<(), &'static str> {
    match mode {
        ManagedIntegrationModeV1::Install => {
            if installs
                .iter()
                .any(|install| install.effect == ManagedIntegrationEffectV1::ContentUpdate)
            {
                Err("integration-reconcile-selection-invalid")
            } else if installs
                .iter()
                .any(|install| install.effect == ManagedIntegrationEffectV1::Install)
            {
                Ok(())
            } else {
                Err("integration-install-not-required")
            }
        }
        ManagedIntegrationModeV1::Repair => {
            if installs
                .iter()
                .any(|install| install.effect == ManagedIntegrationEffectV1::Install)
            {
                return Err("integration-reconcile-selection-invalid");
            }
            if installs.iter().any(|install| {
                matches!(
                    install.effect,
                    ManagedIntegrationEffectV1::Repair | ManagedIntegrationEffectV1::ContentUpdate
                )
            }) {
                Ok(())
            } else {
                Err("integration-reconcile-not-required")
            }
        }
    }
}

fn ordered_unique_targets(targets: impl IntoIterator<Item = ManagedIntegrationTargetV1>) -> bool {
    matches!(
        targets.into_iter().collect::<Vec<_>>().as_slice(),
        [ManagedIntegrationTargetV1::Codex]
            | [ManagedIntegrationTargetV1::ClaudeCode]
            | [
                ManagedIntegrationTargetV1::Codex,
                ManagedIntegrationTargetV1::ClaudeCode
            ]
    )
}

fn integration_approvals() -> Vec<ManagedOperationApprovalV1> {
    vec![
        ManagedOperationApprovalV1::FilesystemWrite,
        ManagedOperationApprovalV1::ClientConfigChange,
        ManagedOperationApprovalV1::HostTrust,
    ]
}

fn cli_install_digest(control_sha256: &str, native_plan_digest_sha256: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-MANAGED-OPERATION-CLI-INSTALL-V1\0");
    hash_component(&mut hasher, control_sha256.as_bytes());
    hash_component(&mut hasher, native_plan_digest_sha256.as_bytes());
    encode_lower_hex(&hasher.finalize())
}

fn cli_lifecycle_digest(
    operation: &[u8],
    control_sha256: &str,
    native_plan_digest_sha256: &str,
    profile_name: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-MANAGED-OPERATION-CLI-LIFECYCLE-V1\0");
    hash_component(&mut hasher, operation);
    hash_component(&mut hasher, control_sha256.as_bytes());
    hash_component(&mut hasher, native_plan_digest_sha256.as_bytes());
    if let Some(profile_name) = profile_name {
        hash_component(&mut hasher, profile_name.as_bytes());
    }
    encode_lower_hex(&hasher.finalize())
}

fn integration_reconcile_digest(
    mode: ManagedIntegrationModeV1,
    control_sha256: &str,
    native_batch_plan_digest_sha256: &str,
    installs: &[ManagedIntegrationInstallPreviewV1],
    host_plan_digest_sha256: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-MANAGED-OPERATION-INTEGRATION-RECONCILE-V2\0");
    hash_component(&mut hasher, integration_mode_name(mode).as_bytes());
    hash_component(&mut hasher, control_sha256.as_bytes());
    hash_component(&mut hasher, native_batch_plan_digest_sha256.as_bytes());
    for install in installs {
        hash_component(
            &mut hasher,
            integration_target_name(install.target).as_bytes(),
        );
        hash_component(
            &mut hasher,
            integration_effect_name(install.effect).as_bytes(),
        );
        hash_component(&mut hasher, install.native_plan_digest_sha256.as_bytes());
    }
    hash_component(&mut hasher, host_plan_digest_sha256.as_bytes());
    encode_lower_hex(&hasher.finalize())
}

fn integration_content_update_digest(native_plan_digest: &str, evidence_digest: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-MANAGED-OPERATION-INTEGRATION-CONTENT-UPDATE-V1\0");
    hash_component(&mut hasher, native_plan_digest.as_bytes());
    hash_component(&mut hasher, evidence_digest.as_bytes());
    encode_lower_hex(&hasher.finalize())
}

fn integration_remove_digest(
    control_sha256: &str,
    verifications: &[ManagedIntegrationVerificationV1],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-MANAGED-OPERATION-INTEGRATION-REMOVE-V1\0");
    hash_component(&mut hasher, control_sha256.as_bytes());
    for verification in verifications {
        hash_component(
            &mut hasher,
            integration_target_name(verification.target).as_bytes(),
        );
        hash_component(&mut hasher, verification.evidence_digest_sha256.as_bytes());
    }
    encode_lower_hex(&hasher.finalize())
}

fn integration_verification_digest(
    verification: &PackagedProductInstallVerification,
) -> Result<String, &'static str> {
    let artifact = serde_json_canonicalizer::to_vec(&verification.source.artifact)
        .map_err(|_| "packaged-product-evidence-invalid")?;
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-MANAGED-OPERATION-INTEGRATION-VERIFICATION-V1\0");
    hash_component(
        &mut hasher,
        integration_target_name(managed_target(verification.target)).as_bytes(),
    );
    hash_component(&mut hasher, &artifact);
    hash_component(
        &mut hasher,
        verification.source.signed_grant_payload_sha256.as_bytes(),
    );
    hash_component(&mut hasher, verification.source.receipt_sha256.as_bytes());
    hash_component(
        &mut hasher,
        verification.source.package_content_root_sha256.as_bytes(),
    );
    hash_component(&mut hasher, verification.source.binary_sha256.as_bytes());
    hash_component(
        &mut hasher,
        verification.source.resource_pack_sha256.as_bytes(),
    );
    hash_component(
        &mut hasher,
        verification.activation_transaction_id.as_bytes(),
    );
    Ok(encode_lower_hex(&hasher.finalize()))
}

const fn integration_target_name(target: ManagedIntegrationTargetV1) -> &'static str {
    match target {
        ManagedIntegrationTargetV1::Codex => "codex",
        ManagedIntegrationTargetV1::ClaudeCode => "claude-code",
    }
}

const fn integration_effect_name(effect: ManagedIntegrationEffectV1) -> &'static str {
    match effect {
        ManagedIntegrationEffectV1::Install => "install",
        ManagedIntegrationEffectV1::Repair => "repair",
        ManagedIntegrationEffectV1::ContentUpdate => "content-update",
        ManagedIntegrationEffectV1::AlreadyCurrent => "already-current",
    }
}

const fn integration_mode_name(mode: ManagedIntegrationModeV1) -> &'static str {
    match mode {
        ManagedIntegrationModeV1::Install => "install",
        ManagedIntegrationModeV1::Repair => "repair",
    }
}

fn skills_semantic_digest(
    action: &str,
    target_id: &str,
    profile: ProfileId,
    state: ManagedSkillsStateV1,
    receipt_sha256: Option<&str>,
    pack_sha256: &str,
    workflow_variant_sha256: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-MANAGED-OPERATION-SKILLS-V2\0");
    hash_component(&mut hasher, action.as_bytes());
    hash_component(&mut hasher, target_id.as_bytes());
    hash_component(&mut hasher, profile_name(profile).as_bytes());
    hash_component(&mut hasher, state_name(state).as_bytes());
    hash_component(&mut hasher, receipt_sha256.unwrap_or("").as_bytes());
    hash_component(&mut hasher, pack_sha256.as_bytes());
    hash_component(
        &mut hasher,
        workflow_variant_sha256.unwrap_or("canonical").as_bytes(),
    );
    encode_lower_hex(&hasher.finalize())
}

fn canonical_json(value: &impl Serialize) -> Result<String, &'static str> {
    let bytes =
        serde_json_canonicalizer::to_vec(value).map_err(|_| "managed-operation-output-failed")?;
    String::from_utf8(bytes).map_err(|_| "managed-operation-output-failed")
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "managed-operation-clock-invalid")
}

const fn profile_name(profile: ProfileId) -> &'static str {
    match profile {
        ProfileId::SkillOnly => "skill-only",
        ProfileId::MarketplaceLite => "marketplace-lite",
        ProfileId::Full => "full",
    }
}

const fn state_name(state: ManagedSkillsStateV1) -> &'static str {
    match state {
        ManagedSkillsStateV1::Missing => "missing",
        ManagedSkillsStateV1::Current => "current",
        ManagedSkillsStateV1::UpdateAvailable => "update-available",
        ManagedSkillsStateV1::Drifted => "drifted",
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_lower_hex(&Sha256::digest(bytes))
}

fn hash_component(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(bytes);
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_cli_health_requires_the_exact_candidate_plan() {
        let content = crate::embedded_content().unwrap();
        let candidate = "1".repeat(64);
        let mut plan = super::ManagedOperationPlanV1::new(
            &content,
            super::now_unix().unwrap(),
            super::ManagedOperationV1::CliInstall {
                control_sha256: candidate.clone(),
                native_plan_digest_sha256: "2".repeat(64),
            },
            vec![super::ManagedOperationApprovalV1::FilesystemWrite],
            "3".repeat(64),
        )
        .unwrap();
        let output = plan.to_canonical_json().unwrap();
        let version = env!("CARGO_PKG_VERSION");
        let pack = content.pack().pack_sha256();
        assert!(super::verify_native_cli_health_plan(&output, version, pack, &candidate).is_ok());
        for (v, p, c) in [
            ("2.0.0-alpha.0", pack, candidate.as_str()),
            (version, "wrong-pack", candidate.as_str()),
            (version, pack, "wrong-candidate"),
        ] {
            assert!(super::verify_native_cli_health_plan(&output, v, p, c).is_err());
        }
        plan.product_version = "2.0.0-alpha.4".to_string();
        plan.plan_digest_sha256 = plan.compute_digest().unwrap();
        let previous = plan.to_canonical_json().unwrap();
        assert!(
            super::verify_native_cli_health_plan(
                &previous,
                &plan.product_version,
                pack,
                &candidate
            )
            .is_ok()
        );
        assert_eq!(
            plan.validate(super::now_unix().unwrap()),
            Err("managed-operation-plan-invalid")
        );
        assert!(
            super::verify_native_cli_health_plan(&previous, version, pack, &candidate).is_err()
        );
        plan.plan_digest_sha256 = "0".repeat(64);
        assert_eq!(
            super::verify_native_cli_health_plan(
                &plan.to_canonical_json().unwrap(),
                &plan.product_version,
                pack,
                &candidate
            ),
            Err("managed-operation-plan-digest-invalid")
        );
        plan.product_version = version.to_string();
        plan.operation = super::ManagedOperationV1::CliRemove {
            control_sha256: candidate.clone(),
            native_plan_digest_sha256: "2".repeat(64),
        };
        plan.plan_digest_sha256 = plan.compute_digest().unwrap();
        assert!(
            super::verify_native_cli_health_plan(
                &plan.to_canonical_json().unwrap(),
                version,
                pack,
                &candidate
            )
            .is_err()
        );
        assert!(
            super::verify_native_cli_health_plan("not-json", version, pack, &candidate).is_err()
        );
    }

    use super::*;

    #[cfg(unix)]
    fn test_root(label: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("target/qiongli-managed-operation-tests")
            .join(format!("{}-{label}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        root
    }

    #[cfg(unix)]
    fn environment(root: &Path, project: Option<&Path>) -> CommandEnvironment {
        CommandEnvironment::with_paths(None, Some(root.to_path_buf()), None).with_inventory_context(
            None,
            project.map(Path::to_path_buf),
            false,
            false,
        )
    }

    #[cfg(unix)]
    fn write_plan(path: &Path, plan: &ManagedOperationPlanV1) {
        fs::write(path, plan.to_canonical_json().unwrap()).unwrap();
    }

    #[cfg(unix)]
    fn filesystem_approval() -> [(ManagedOperationApprovalV1, bool); 3] {
        [
            (ManagedOperationApprovalV1::FilesystemWrite, true),
            (ManagedOperationApprovalV1::ClientConfigChange, false),
            (ManagedOperationApprovalV1::HostTrust, false),
        ]
    }

    #[test]
    fn integration_plans_reject_known_unsupported_clients_but_allow_unknown_versions() {
        let unsupported_codex = CommandEnvironment::default().with_client_versions(
            Some(crate::command::DetectedClientVersion {
                major: 0,
                minor: 144,
                patch: 0,
            }),
            Some(crate::command::DetectedClientVersion {
                major: 2,
                minor: 1,
                patch: 206,
            }),
        );
        assert_eq!(
            reject_unsupported_integration_versions(
                &unsupported_codex,
                &[ClientActivationTarget::Codex],
            ),
            Err("integration-client-version-unsupported")
        );
        assert_eq!(
            reject_unsupported_integration_versions(
                &unsupported_codex,
                &[ClientActivationTarget::ClaudeCode],
            ),
            Ok(())
        );
        assert_eq!(
            reject_unsupported_integration_versions(
                &CommandEnvironment::default(),
                &[ClientActivationTarget::Codex],
            ),
            Ok(())
        );
    }

    #[cfg(unix)]
    #[test]
    fn skills_plan_is_path_free_and_survives_a_fresh_process_context() {
        let root = test_root("skills-lifecycle");
        let first_environment = environment(&root, None);
        let first_content = crate::embedded_content().unwrap();
        let plan = prepare_skills_reconcile_plan(
            &first_environment,
            &first_content,
            ManagedSkillsPresetV1::QiongliManaged,
            ProfileId::SkillOnly,
        )
        .unwrap();
        let json = plan.to_canonical_json().unwrap();
        assert!(!json.contains(root.to_string_lossy().as_ref()));
        assert!(!json.contains(".qiongli-skills"));
        let target_id = match &plan.operation {
            ManagedOperationV1::SkillsReconcilePreset { target_id, .. } => target_id.clone(),
            _ => unreachable!(),
        };
        let plan_path = root.join("skills-install.plan.json");
        write_plan(&plan_path, &plan);

        let second_environment = environment(&root, None);
        let second_content = crate::embedded_content().unwrap();
        let result = apply_plan(
            &second_environment,
            &second_content,
            &plan_path,
            &plan.plan_digest_sha256,
            &filesystem_approval(),
        )
        .unwrap();
        assert!(result.contains("\"result\":\"installed\""));
        let observed =
            observe_registered_target(&second_environment, &second_content, &target_id).unwrap();
        assert_eq!(observed.state, ManagedSkillsStateV1::Current);
        let app_snapshot =
            crate::desktop::app_snapshot_json(&second_environment, &second_content).unwrap();
        assert!(!app_snapshot.contains(root.to_string_lossy().as_ref()));
        let app_snapshot: serde_json::Value = serde_json::from_str(&app_snapshot).unwrap();
        let app_destination = app_snapshot["content"]["managedSkills"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|destination| destination["targetId"] == target_id)
            .expect("CLI-installed Skills must be visible through the GUI snapshot");
        assert_eq!(app_destination["state"], "current");
        assert_eq!(app_destination["profile"], "skill-only");

        let removal =
            prepare_skills_remove_plan(&second_environment, &second_content, &target_id).unwrap();
        let removal_path = root.join("skills-remove.plan.json");
        write_plan(&removal_path, &removal);
        let result = apply_plan(
            &environment(&root, None),
            &crate::embedded_content().unwrap(),
            &removal_path,
            &removal.plan_digest_sha256,
            &filesystem_approval(),
        )
        .unwrap();
        assert!(result.contains("\"result\":\"removed\""));
        assert_eq!(
            observe_registered_target(
                &environment(&root, None),
                &crate::embedded_content().unwrap(),
                &target_id
            )
            .err()
            .unwrap(),
            "managed-skills-target-not-registered"
        );
        let app_snapshot = crate::desktop::app_snapshot_json(
            &environment(&root, None),
            &crate::embedded_content().unwrap(),
        )
        .unwrap();
        let app_snapshot: serde_json::Value = serde_json::from_str(&app_snapshot).unwrap();
        let app_destination = app_snapshot["content"]["managedSkills"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|destination| destination["targetId"] == target_id)
            .expect("the preset remains discoverable after CLI removal");
        assert_eq!(app_destination["state"], "missing");
        assert!(app_destination["profile"].is_null());
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn drifted_skills_detach_plan_preserves_target_bytes_across_processes() {
        let root = test_root("skills-detach");
        let first_environment = environment(&root, None);
        let content = crate::embedded_content().unwrap();
        let install = prepare_skills_reconcile_plan(
            &first_environment,
            &content,
            ManagedSkillsPresetV1::QiongliManaged,
            ProfileId::SkillOnly,
        )
        .unwrap();
        let target_id = match &install.operation {
            ManagedOperationV1::SkillsReconcilePreset { target_id, .. } => target_id.clone(),
            _ => unreachable!(),
        };
        let install_path = root.join("skills-install.plan.json");
        write_plan(&install_path, &install);
        apply_plan(
            &environment(&root, None),
            &crate::embedded_content().unwrap(),
            &install_path,
            &install.plan_digest_sha256,
            &filesystem_approval(),
        )
        .unwrap();

        let destination = root.join(".qiongli-skills");
        fs::write(destination.join(".qiongli-managed.json"), b"{}").unwrap();
        fs::write(
            destination.join("retained-user-change.txt"),
            b"retain-this-user-change",
        )
        .unwrap();
        let detach = prepare_skills_detach_plan(
            &environment(&root, None),
            &crate::embedded_content().unwrap(),
            &target_id,
        )
        .unwrap();
        let detach_json = detach.to_canonical_json().unwrap();
        assert!(!detach_json.contains(root.to_string_lossy().as_ref()));
        assert!(!detach_json.contains(".qiongli-skills"));
        assert!(detach_json.contains("\"expected_state\":\"drifted\""));
        let detach_path = root.join("skills-detach.plan.json");
        write_plan(&detach_path, &detach);

        let result = apply_plan(
            &environment(&root, None),
            &crate::embedded_content().unwrap(),
            &detach_path,
            &detach.plan_digest_sha256,
            &filesystem_approval(),
        )
        .unwrap();
        assert!(result.contains("\"result\":\"detached-preserved\""));
        assert_eq!(
            fs::read(destination.join(".qiongli-managed.json")).unwrap(),
            b"{}"
        );
        assert_eq!(
            fs::read(destination.join("retained-user-change.txt")).unwrap(),
            b"retain-this-user-change"
        );
        assert_eq!(
            observe_registered_target(
                &environment(&root, None),
                &crate::embedded_content().unwrap(),
                &target_id,
            )
            .err()
            .unwrap(),
            "managed-skills-target-not-registered"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn current_project_plan_is_bound_to_the_reviewed_project_target() {
        let root = test_root("project-binding");
        let first_project = root.join("first-project");
        let second_project = root.join("second-project");
        fs::create_dir_all(&first_project).unwrap();
        fs::create_dir_all(&second_project).unwrap();
        let content = crate::embedded_content().unwrap();
        let plan = prepare_skills_reconcile_plan(
            &environment(&root, Some(&first_project)),
            &content,
            ManagedSkillsPresetV1::CurrentProject,
            ProfileId::SkillOnly,
        )
        .unwrap();
        let plan_path = root.join("project.plan.json");
        write_plan(&plan_path, &plan);
        assert_eq!(
            apply_plan(
                &environment(&root, Some(&second_project)),
                &content,
                &plan_path,
                &plan.plan_digest_sha256,
                &filesystem_approval(),
            )
            .unwrap_err(),
            "managed-operation-precondition-changed"
        );
        assert!(!first_project.join(".qiongli-skills").exists());
        assert!(!second_project.join(".qiongli-skills").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn apply_rejects_digest_approval_expiry_and_noncanonical_input() {
        let root = test_root("plan-validation");
        let environment = environment(&root, None);
        let content = crate::embedded_content().unwrap();
        let plan = prepare_skills_reconcile_plan(
            &environment,
            &content,
            ManagedSkillsPresetV1::QiongliManaged,
            ProfileId::SkillOnly,
        )
        .unwrap();
        let path = root.join("validation.plan.json");
        write_plan(&path, &plan);
        assert_eq!(
            apply_plan(
                &environment,
                &content,
                &path,
                &"0".repeat(64),
                &filesystem_approval(),
            )
            .unwrap_err(),
            "managed-operation-plan-digest-mismatch"
        );
        assert_eq!(
            apply_plan(
                &environment,
                &content,
                &path,
                &plan.plan_digest_sha256,
                &[
                    (ManagedOperationApprovalV1::FilesystemWrite, false),
                    (ManagedOperationApprovalV1::ClientConfigChange, false),
                    (ManagedOperationApprovalV1::HostTrust, false),
                ],
            )
            .unwrap_err(),
            "managed-operation-approval-required"
        );

        let mut expired = plan.clone();
        expired.created_at_unix = 1;
        expired.expires_at_unix = 2;
        expired.plan_digest_sha256 = expired.compute_digest().unwrap();
        write_plan(&path, &expired);
        assert_eq!(
            apply_plan(
                &environment,
                &content,
                &path,
                &expired.plan_digest_sha256,
                &filesystem_approval(),
            )
            .unwrap_err(),
            "managed-operation-plan-expired"
        );

        fs::write(
            &path,
            serde_json::to_string_pretty(&plan).unwrap().as_bytes(),
        )
        .unwrap();
        assert_eq!(
            read_plan(&path).unwrap_err(),
            "managed-operation-plan-noncanonical"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plan_contract_rejects_unknown_or_path_bearing_fields() {
        let content = crate::embedded_content().unwrap();
        let operation = ManagedOperationV1::SkillsReconcilePreset {
            preset: ManagedSkillsPresetV1::QiongliManaged,
            target_id: format!("skills-target-{}", "1".repeat(64)),
            profile: ProfileId::SkillOnly,
            expected_state: ManagedSkillsStateV1::Missing,
            expected_receipt_sha256: None,
        };
        let plan = ManagedOperationPlanV1::new(
            &content,
            100,
            operation,
            vec![ManagedOperationApprovalV1::FilesystemWrite],
            "2".repeat(64),
        )
        .unwrap();
        let mut value = serde_json::to_value(&plan).unwrap();
        value["operation"]["target_path"] = serde_json::json!("/private/secret");
        assert!(
            serde_json::from_value::<ManagedOperationPlanV1>(value)
                .unwrap_err()
                .to_string()
                .contains("unknown field")
        );
    }

    #[cfg(unix)]
    #[test]
    fn source_process_cannot_mint_packaged_integration_plans() {
        let root = test_root("source-integration-authority");
        let environment = environment(&root, None);
        let content = crate::embedded_content().unwrap();
        assert_eq!(
            prepare_integrations_reconcile_plan(
                &environment,
                &content,
                &[ManagedIntegrationTargetV1::Codex],
                ManagedIntegrationModeV1::Install,
            )
            .unwrap_err(),
            "source-build-read-only"
        );
        assert_eq!(
            prepare_integrations_remove_plan(
                &environment,
                &content,
                &[ManagedIntegrationTargetV1::Codex],
            )
            .unwrap_err(),
            "source-build-read-only"
        );
        assert_eq!(
            prepare_cli_install_plan(&environment, &content).unwrap_err(),
            "source-build-read-only"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn integration_plan_contract_requires_canonical_targets_and_all_approvals() {
        let content = crate::embedded_content().unwrap();
        let installs = vec![
            ManagedIntegrationInstallPreviewV1 {
                target: ManagedIntegrationTargetV1::Codex,
                effect: ManagedIntegrationEffectV1::Install,
                native_plan_digest_sha256: "6".repeat(64),
            },
            ManagedIntegrationInstallPreviewV1 {
                target: ManagedIntegrationTargetV1::ClaudeCode,
                effect: ManagedIntegrationEffectV1::Repair,
                native_plan_digest_sha256: "7".repeat(64),
            },
        ];
        ManagedOperationPlanV1::new(
            &content,
            100,
            ManagedOperationV1::IntegrationsReconcile {
                mode: ManagedIntegrationModeV1::Install,
                control_sha256: "8".repeat(64),
                native_batch_plan_digest_sha256: "9".repeat(64),
                installs: installs.clone(),
            },
            integration_approvals(),
            "a".repeat(64),
        )
        .unwrap();
        ManagedOperationPlanV1::new(
            &content,
            100,
            ManagedOperationV1::IntegrationsReconcile {
                mode: ManagedIntegrationModeV1::Repair,
                control_sha256: "8".repeat(64),
                native_batch_plan_digest_sha256: "9".repeat(64),
                installs: vec![ManagedIntegrationInstallPreviewV1 {
                    target: ManagedIntegrationTargetV1::Codex,
                    effect: ManagedIntegrationEffectV1::ContentUpdate,
                    native_plan_digest_sha256: "6".repeat(64),
                }],
            },
            integration_approvals(),
            "a".repeat(64),
        )
        .unwrap();
        ManagedOperationPlanV1::new(
            &content,
            100,
            ManagedOperationV1::IntegrationsReconcile {
                mode: ManagedIntegrationModeV1::Repair,
                control_sha256: "8".repeat(64),
                native_batch_plan_digest_sha256: "9".repeat(64),
                installs: vec![
                    ManagedIntegrationInstallPreviewV1 {
                        target: ManagedIntegrationTargetV1::Codex,
                        effect: ManagedIntegrationEffectV1::Repair,
                        native_plan_digest_sha256: "6".repeat(64),
                    },
                    ManagedIntegrationInstallPreviewV1 {
                        target: ManagedIntegrationTargetV1::ClaudeCode,
                        effect: ManagedIntegrationEffectV1::AlreadyCurrent,
                        native_plan_digest_sha256: "7".repeat(64),
                    },
                ],
            },
            integration_approvals(),
            "a".repeat(64),
        )
        .unwrap();
        assert_eq!(
            ManagedOperationPlanV1::new(
                &content,
                100,
                ManagedOperationV1::IntegrationsReconcile {
                    mode: ManagedIntegrationModeV1::Install,
                    control_sha256: "8".repeat(64),
                    native_batch_plan_digest_sha256: "9".repeat(64),
                    installs: vec![installs[0].clone(), installs[0].clone()],
                },
                integration_approvals(),
                "a".repeat(64),
            )
            .unwrap_err(),
            "managed-operation-plan-invalid"
        );
        assert_eq!(
            ManagedOperationPlanV1::new(
                &content,
                100,
                ManagedOperationV1::IntegrationsReconcile {
                    mode: ManagedIntegrationModeV1::Install,
                    control_sha256: "8".repeat(64),
                    native_batch_plan_digest_sha256: "9".repeat(64),
                    installs: vec![ManagedIntegrationInstallPreviewV1 {
                        target: ManagedIntegrationTargetV1::Codex,
                        effect: ManagedIntegrationEffectV1::ContentUpdate,
                        native_plan_digest_sha256: "6".repeat(64),
                    }],
                },
                integration_approvals(),
                "a".repeat(64),
            )
            .unwrap_err(),
            "managed-operation-plan-invalid"
        );
        assert_eq!(
            ManagedOperationPlanV1::new(
                &content,
                100,
                ManagedOperationV1::IntegrationsReconcile {
                    mode: ManagedIntegrationModeV1::Install,
                    control_sha256: "8".repeat(64),
                    native_batch_plan_digest_sha256: "9".repeat(64),
                    installs: vec![ManagedIntegrationInstallPreviewV1 {
                        target: ManagedIntegrationTargetV1::Codex,
                        effect: ManagedIntegrationEffectV1::AlreadyCurrent,
                        native_plan_digest_sha256: "6".repeat(64),
                    }],
                },
                integration_approvals(),
                "a".repeat(64),
            )
            .unwrap_err(),
            "managed-operation-plan-invalid"
        );
        assert_eq!(
            ManagedOperationPlanV1::new(
                &content,
                100,
                ManagedOperationV1::IntegrationsReconcile {
                    mode: ManagedIntegrationModeV1::Install,
                    control_sha256: "8".repeat(64),
                    native_batch_plan_digest_sha256: "9".repeat(64),
                    installs,
                },
                vec![ManagedOperationApprovalV1::FilesystemWrite],
                "a".repeat(64),
            )
            .unwrap_err(),
            "managed-operation-plan-invalid"
        );
        assert_eq!(
            ManagedOperationPlanV1::new(
                &content,
                100,
                ManagedOperationV1::IntegrationsReconcile {
                    mode: ManagedIntegrationModeV1::Repair,
                    control_sha256: "8".repeat(64),
                    native_batch_plan_digest_sha256: "9".repeat(64),
                    installs: vec![ManagedIntegrationInstallPreviewV1 {
                        target: ManagedIntegrationTargetV1::Codex,
                        effect: ManagedIntegrationEffectV1::Install,
                        native_plan_digest_sha256: "6".repeat(64),
                    }],
                },
                integration_approvals(),
                "a".repeat(64),
            )
            .unwrap_err(),
            "managed-operation-plan-invalid"
        );
    }
}
