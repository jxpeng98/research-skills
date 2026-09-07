use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use qiongli_config::ProviderSettings;
use qiongli_config::{
    GlobalSettings, LegacyProviderResolution, LegacyProviderSecret, LoadedGlobalSettings,
    SecretRef, SecretStore, SecretStoreError, SecretStoreStatus, SecretValue,
};
use qiongli_content::EmbeddedContent;
use qiongli_platform::{
    ClientActivationTarget, ClientKind, LegacyMigrationAction, LegacyMigrationApproval,
    LegacyMigrationInventory, LegacyMigrationInventoryV1, LegacyMigrationItemId,
    LegacyMigrationPlanInput, LegacyMigrationPlanV1, LegacyMigrationReceiptItemState,
    LegacyMigrationReceiptItemV1, LegacyMigrationReceiptV1, LegacyMigrationState,
    LegacyMigrationStore, advance_legacy_migration_receipt, apply_legacy_migration_cleanup,
    apply_packaged_product_batch_install, approve_legacy_migration_plan,
    discover_legacy_migration_with_config, finalize_legacy_migration_cleanup,
    initial_legacy_migration_receipt_from_plan, prepare_legacy_migration_cleanup,
    preview_legacy_migration, preview_packaged_product_batch_install,
    recover_legacy_migration_cleanup, resume_legacy_migration_plan,
    verify_legacy_migration_cutover, verify_packaged_product_install,
};
use serde::Serialize;
use sha2::{Digest as _, Sha256};

use crate::command::{CommandEnvironment, config_root, config_store};
use crate::credential_store::native_secret_store;
use crate::desktop::verify_running_packaged_product;

const OUTPUT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LegacyMigrationCliCommand {
    Inspect,
    Preview {
        provider_resolutions: Vec<LegacyProviderResolution>,
    },
    Apply {
        migration_id: String,
        expected_plan_digest: String,
        approve_filesystem_write: bool,
        approve_client_config_change: bool,
        approve_secret_store_write: bool,
    },
    Continue {
        migration_id: String,
        action: LegacyMigrationContinueAction,
    },
    Status {
        migration_id: String,
    },
    Recover {
        migration_id: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LegacyMigrationContinueAction {
    ConfirmHostActivation,
    Cleanup,
    Finalize,
}

#[derive(Debug, Serialize)]
#[serde(tag = "command", rename_all = "kebab-case")]
pub(crate) enum LegacyMigrationCliOutput {
    Inspect {
        schema_version: u32,
        inventory: LegacyMigrationInventoryV1,
    },
    Preview {
        schema_version: u32,
        plan: LegacyMigrationPlanV1,
        receipt: LegacyMigrationReceiptV1,
    },
    Apply {
        schema_version: u32,
        migration_id: String,
        state: LegacyMigrationState,
        plan_sha256: String,
    },
    Continue {
        schema_version: u32,
        migration_id: String,
        state: LegacyMigrationState,
        receipt_sha256: String,
    },
    Status {
        schema_version: u32,
        plan: LegacyMigrationPlanV1,
        receipt: LegacyMigrationReceiptV1,
        inventory: LegacyMigrationInventoryV1,
    },
    Recover {
        schema_version: u32,
        migration_id: String,
        state: LegacyMigrationState,
        restored_item_count: usize,
    },
}

pub(crate) fn execute(
    command: LegacyMigrationCliCommand,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<LegacyMigrationCliOutput, &'static str> {
    let secret_store = native_secret_store();
    execute_with_secret_store(command, environment, content, secret_store.as_ref())
}

pub(crate) fn execute_with_secret_store(
    command: LegacyMigrationCliCommand,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    secret_store: &dyn SecretStore,
) -> Result<LegacyMigrationCliOutput, &'static str> {
    let inventory = inventory(environment)?;
    match command {
        LegacyMigrationCliCommand::Inspect => Ok(LegacyMigrationCliOutput::Inspect {
            schema_version: OUTPUT_SCHEMA_VERSION,
            inventory: inventory.summary().clone(),
        }),
        LegacyMigrationCliCommand::Preview {
            provider_resolutions,
        } => preview(&inventory, environment, content, &provider_resolutions),
        LegacyMigrationCliCommand::Apply {
            migration_id,
            expected_plan_digest,
            approve_filesystem_write,
            approve_client_config_change,
            approve_secret_store_write,
        } => apply(
            &inventory,
            environment,
            content,
            secret_store,
            &migration_id,
            &expected_plan_digest,
            approve_filesystem_write,
            approve_client_config_change,
            approve_secret_store_write,
        ),
        LegacyMigrationCliCommand::Continue {
            migration_id,
            action,
        } => continue_migration(
            &inventory,
            environment,
            content,
            secret_store,
            &migration_id,
            action,
        ),
        LegacyMigrationCliCommand::Status { migration_id } => {
            let store = LegacyMigrationStore::for_inventory(&inventory)
                .map_err(|error| error.reason_code())?;
            Ok(LegacyMigrationCliOutput::Status {
                schema_version: OUTPUT_SCHEMA_VERSION,
                plan: store
                    .load_plan(&migration_id)
                    .map_err(|error| error.reason_code())?,
                receipt: store
                    .load_receipt(&migration_id)
                    .map_err(|error| error.reason_code())?,
                inventory: inventory.summary().clone(),
            })
        }
        LegacyMigrationCliCommand::Recover { migration_id } => recover(&inventory, &migration_id),
    }
}

fn preview(
    inventory: &LegacyMigrationInventory,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    provider_resolutions: &[LegacyProviderResolution],
) -> Result<LegacyMigrationCliOutput, &'static str> {
    let product = verify_running_packaged_product(environment, content)?;
    let now_unix = now_unix()?;
    let mut provider_resolutions = provider_resolutions.to_vec();
    provider_resolutions.sort_unstable_by_key(|resolution| resolution.provider);
    let plan = preview_legacy_migration(
        inventory,
        LegacyMigrationPlanInput {
            plan_id: &format!("migration-{now_unix}-{}", std::process::id()),
            product_version: &product.artifact().version,
            source_commit: product.product_source_commit(),
            resource_pack_sha256: product.resource_pack_sha256(),
            created_at_unix: now_unix,
            provider_resolutions: &provider_resolutions,
        },
    )
    .map_err(|error| error.reason_code())?;
    validate_legacy_provider_destination(&plan, inventory, environment)?;
    let receipt =
        initial_legacy_migration_receipt_from_plan(&plan).map_err(|error| error.reason_code())?;
    LegacyMigrationStore::for_inventory(inventory)
        .map_err(|error| error.reason_code())?
        .persist_preview(&plan, &receipt)
        .map_err(|error| error.reason_code())?;
    Ok(LegacyMigrationCliOutput::Preview {
        schema_version: OUTPUT_SCHEMA_VERSION,
        plan,
        receipt,
    })
}

#[allow(clippy::too_many_arguments)]
fn apply(
    inventory: &LegacyMigrationInventory,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    secret_store: &dyn SecretStore,
    migration_id: &str,
    expected_plan_digest: &str,
    filesystem_write: bool,
    client_config_change: bool,
    secret_store_write: bool,
) -> Result<LegacyMigrationCliOutput, &'static str> {
    let store =
        LegacyMigrationStore::for_inventory(inventory).map_err(|error| error.reason_code())?;
    let plan = store
        .load_plan(migration_id)
        .map_err(|error| error.reason_code())?;
    if plan.plan_sha256 != expected_plan_digest {
        return Err("legacy-migration-plan-digest-mismatch");
    }
    let receipt = store
        .load_receipt(migration_id)
        .map_err(|error| error.reason_code())?;
    if receipt.state != LegacyMigrationState::PreviewReady {
        return Err("legacy-migration-preview-state-invalid");
    }
    let approvals = staging_approvals(
        &plan,
        filesystem_write,
        client_config_change,
        secret_store_write,
    )?;
    let _approved = approve_legacy_migration_plan(plan.clone(), inventory, now_unix()?, &approvals)
        .map_err(|error| error.reason_code())?;
    let targets = migration_targets(&plan);
    let product = verify_running_packaged_product(environment, content)?;
    if product.artifact().version != plan.product_version
        || product.product_source_commit() != plan.source_commit
        || product.resource_pack_sha256() != plan.resource_pack_sha256
    {
        return Err("legacy-migration-product-identity-mismatch");
    }
    let staged_provider =
        stage_legacy_provider_config(&plan, inventory, environment, secret_store)?;
    if targets.is_empty() && staged_provider.is_none() {
        return Err("legacy-migration-no-eligible-items");
    }
    let install_result = if targets.is_empty() {
        Ok(())
    } else {
        let preview = preview_packaged_product_batch_install(&product, &targets)
            .map_err(|error| error.reason_code())?;
        if !preview.can_apply {
            Err("legacy-migration-product-install-blocked")
        } else {
            apply_packaged_product_batch_install(content.pack(), &product, &preview, now_unix()?)
                .map(|_| ())
                .map_err(|error| error.reason_code())
        }
    };
    if let Err(error) = install_result {
        if let Some(staged_provider) = staged_provider.as_ref() {
            rollback_legacy_provider_config(staged_provider, environment, secret_store)?;
        }
        return Err(error);
    }

    let staged = transition_items(
        &receipt,
        LegacyMigrationState::Staged,
        LegacyMigrationReceiptItemState::Pending,
        LegacyMigrationReceiptItemState::Staged,
        "legacy-migration-item-staged",
    )?;
    store
        .replace_receipt(&receipt.receipt_sha256, &staged)
        .map_err(|error| error.reason_code())?;
    let awaiting = transition_items(
        &staged,
        LegacyMigrationState::AwaitingClientActivation,
        LegacyMigrationReceiptItemState::Staged,
        LegacyMigrationReceiptItemState::AwaitingActivation,
        "legacy-migration-awaiting-host-activation",
    )?;
    store
        .replace_receipt(&staged.receipt_sha256, &awaiting)
        .map_err(|error| error.reason_code())?;
    Ok(LegacyMigrationCliOutput::Apply {
        schema_version: OUTPUT_SCHEMA_VERSION,
        migration_id: migration_id.to_owned(),
        state: awaiting.state,
        plan_sha256: plan.plan_sha256,
    })
}

fn continue_migration(
    inventory: &LegacyMigrationInventory,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    secret_store: &dyn SecretStore,
    migration_id: &str,
    action: LegacyMigrationContinueAction,
) -> Result<LegacyMigrationCliOutput, &'static str> {
    let store =
        LegacyMigrationStore::for_inventory(inventory).map_err(|error| error.reason_code())?;
    let plan = store
        .load_plan(migration_id)
        .map_err(|error| error.reason_code())?;
    let mut receipt = store
        .load_receipt(migration_id)
        .map_err(|error| error.reason_code())?;
    let product = verify_running_packaged_product(environment, content)?;
    let next = match action {
        LegacyMigrationContinueAction::ConfirmHostActivation => {
            if receipt.state == LegacyMigrationState::Staged {
                let awaiting = transition_items(
                    &receipt,
                    LegacyMigrationState::AwaitingClientActivation,
                    LegacyMigrationReceiptItemState::Staged,
                    LegacyMigrationReceiptItemState::AwaitingActivation,
                    "legacy-migration-awaiting-host-activation",
                )?;
                store
                    .replace_receipt(&receipt.receipt_sha256, &awaiting)
                    .map_err(|error| error.reason_code())?;
                receipt = awaiting;
            }
            if !matches!(
                receipt.state,
                LegacyMigrationState::AwaitingClientActivation
                    | LegacyMigrationState::VerificationRequired
            ) {
                return Err("legacy-migration-host-confirmation-state-invalid");
            }
            verify_legacy_provider_config(&plan, inventory, environment, secret_store)?;
            let approvals = approvals_for_resume(&plan, false);
            let approved = resume_legacy_migration_plan(plan, inventory, &receipt, &approvals)
                .map_err(|error| error.reason_code())?;
            verify_legacy_migration_cutover(approved, &product)
                .map_err(|error| error.reason_code())?;
            let verification = if receipt.state == LegacyMigrationState::VerificationRequired {
                receipt.clone()
            } else {
                let verification = transition_items(
                    &receipt,
                    LegacyMigrationState::VerificationRequired,
                    LegacyMigrationReceiptItemState::AwaitingActivation,
                    LegacyMigrationReceiptItemState::Verified,
                    "legacy-migration-item-verified",
                )?;
                store
                    .replace_receipt(&receipt.receipt_sha256, &verification)
                    .map_err(|error| error.reason_code())?;
                verification
            };
            let cleanup_ready = advance_legacy_migration_receipt(
                &verification,
                LegacyMigrationState::CleanupReady,
                verification.items.clone(),
            )
            .map_err(|error| error.reason_code())?;
            store
                .replace_receipt(&verification.receipt_sha256, &cleanup_ready)
                .map_err(|error| error.reason_code())?;
            cleanup_ready
        }
        LegacyMigrationContinueAction::Cleanup => {
            if receipt.state != LegacyMigrationState::CleanupReady {
                return Err("legacy-migration-cleanup-state-invalid");
            }
            verify_legacy_provider_config(&plan, inventory, environment, secret_store)?;
            let approvals = approvals_for_resume(&plan, true);
            let approved = resume_legacy_migration_plan(plan, inventory, &receipt, &approvals)
                .map_err(|error| error.reason_code())?;
            let cutover = verify_legacy_migration_cutover(approved, &product)
                .map_err(|error| error.reason_code())?;
            let prepared = prepare_legacy_migration_cleanup(&cutover, inventory)
                .map_err(|error| error.reason_code())?;
            apply_legacy_migration_cleanup(&prepared).map_err(|error| error.reason_code())?;
            let complete = transition_items(
                &receipt,
                LegacyMigrationState::Complete,
                LegacyMigrationReceiptItemState::Verified,
                LegacyMigrationReceiptItemState::Cleaned,
                "legacy-migration-item-cleaned",
            )?;
            store
                .replace_receipt(&receipt.receipt_sha256, &complete)
                .map_err(|_| "legacy-migration-cleanup-recovery-required")?;
            complete
        }
        LegacyMigrationContinueAction::Finalize => {
            if receipt.state != LegacyMigrationState::Complete {
                return Err("legacy-migration-finalize-state-invalid");
            }
            for target in migration_targets(&plan) {
                verify_packaged_product_install(&product, target)
                    .map_err(|error| error.reason_code())?;
            }
            finalize_legacy_migration_cleanup(inventory, &receipt)
                .map_err(|error| error.reason_code())?;
            receipt
        }
    };
    Ok(LegacyMigrationCliOutput::Continue {
        schema_version: OUTPUT_SCHEMA_VERSION,
        migration_id: migration_id.to_owned(),
        state: next.state,
        receipt_sha256: next.receipt_sha256,
    })
}

fn recover(
    inventory: &LegacyMigrationInventory,
    migration_id: &str,
) -> Result<LegacyMigrationCliOutput, &'static str> {
    let store =
        LegacyMigrationStore::for_inventory(inventory).map_err(|error| error.reason_code())?;
    let current = store
        .load_receipt(migration_id)
        .map_err(|error| error.reason_code())?;
    let recovery = recover_legacy_migration_cleanup(inventory, migration_id)
        .map_err(|error| error.reason_code())?;
    let items = current
        .items
        .iter()
        .map(|item| LegacyMigrationReceiptItemV1 {
            item_id: item.item_id,
            state: if matches!(
                item.state,
                LegacyMigrationReceiptItemState::Cleaned
                    | LegacyMigrationReceiptItemState::Verified
                    | LegacyMigrationReceiptItemState::RecoveryRequired
            ) {
                LegacyMigrationReceiptItemState::RecoveryRequired
            } else {
                item.state
            },
            result_code: "legacy-migration-item-recovery-required".to_owned(),
        })
        .collect();
    let next =
        advance_legacy_migration_receipt(&current, LegacyMigrationState::RecoveryRequired, items)
            .map_err(|error| error.reason_code())?;
    store
        .replace_receipt(&current.receipt_sha256, &next)
        .map_err(|error| error.reason_code())?;
    Ok(LegacyMigrationCliOutput::Recover {
        schema_version: OUTPUT_SCHEMA_VERSION,
        migration_id: migration_id.to_owned(),
        state: next.state,
        restored_item_count: recovery.restored_items.len(),
    })
}

struct StagedLegacyProviderConfig {
    before: Option<LoadedGlobalSettings>,
    after: GlobalSettings,
    after_revision: u64,
    secret_refs: Vec<(LegacyProviderSecret, SecretRef)>,
    created_secret_refs: Vec<SecretRef>,
}

fn validate_legacy_provider_destination(
    plan: &LegacyMigrationPlanV1,
    inventory: &LegacyMigrationInventory,
    environment: &CommandEnvironment,
) -> Result<(), &'static str> {
    if !plan.items.iter().any(|item| {
        item.item_id == LegacyMigrationItemId::LegacyProviderConfig
            && item.action == LegacyMigrationAction::Convert
    }) {
        return Ok(());
    }
    let legacy = inventory
        .legacy_provider_config()
        .ok_or("legacy-provider-config-inventory-mismatch")?;
    let secret_refs = legacy
        .secret_values()
        .map_err(|error| error.reason_code())?
        .into_iter()
        .map(|(provider, _)| {
            deterministic_legacy_secret_ref(plan, provider).map(|reference| (provider, reference))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let loaded = config_store(environment)
        .and_then(|store| store.load())
        .map_err(|error| error.reason_code())?;
    legacy
        .project_settings_with_resolutions(&loaded, &secret_refs, &plan.provider_resolutions)
        .map(|_| ())
        .map_err(|error| error.reason_code())
}

fn stage_legacy_provider_config(
    plan: &LegacyMigrationPlanV1,
    inventory: &LegacyMigrationInventory,
    environment: &CommandEnvironment,
    secret_store: &dyn SecretStore,
) -> Result<Option<StagedLegacyProviderConfig>, &'static str> {
    if !plan.items.iter().any(|item| {
        item.item_id == LegacyMigrationItemId::LegacyProviderConfig
            && item.action == LegacyMigrationAction::Convert
    }) {
        return Ok(None);
    }
    let legacy = inventory
        .legacy_provider_config()
        .ok_or("legacy-provider-config-inventory-mismatch")?;
    let all_secrets = legacy
        .secret_values()
        .map_err(|error| error.reason_code())?;
    let all_secret_refs = all_secrets
        .iter()
        .map(|(provider, _)| {
            deterministic_legacy_secret_ref(plan, *provider).map(|reference| (*provider, reference))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let store = config_store(environment).map_err(|error| error.reason_code())?;
    let loaded = store.load().map_err(|error| error.reason_code())?;
    let projected = legacy
        .project_settings_with_resolutions(&loaded, &all_secret_refs, &plan.provider_resolutions)
        .map_err(|error| error.reason_code())?;
    let selected = all_secrets
        .into_iter()
        .zip(all_secret_refs)
        .filter(|((provider, _), (_, reference))| {
            projected_uses_legacy_secret(&projected, *provider, reference)
        })
        .collect::<Vec<_>>();
    let (secrets, secret_refs): (Vec<_>, Vec<_>) = selected.into_iter().unzip();
    if !secrets.is_empty() && secret_store.status() != SecretStoreStatus::Available {
        return Err("legacy-provider-secret-store-unavailable");
    }

    let created_secret_refs = stage_legacy_secret_values(&secrets, &secret_refs, secret_store)?;
    if loaded.settings == projected {
        if let Err(error) = verify_legacy_secret_values(&secrets, &secret_refs, secret_store) {
            remove_secret_refs(&created_secret_refs, secret_store);
            return Err(error);
        }
        return Ok(Some(StagedLegacyProviderConfig {
            before: None,
            after: projected,
            after_revision: loaded.revision,
            secret_refs,
            created_secret_refs,
        }));
    }
    let outcome = match store.replace(loaded.revision, projected.clone()) {
        Ok(outcome) => outcome,
        Err(error) => {
            remove_secret_refs(&created_secret_refs, secret_store);
            return Err(error.reason_code());
        }
    };
    let verified = store
        .load()
        .map_err(|_| "legacy-provider-v2-recovery-required")?;
    if verified.revision != outcome.revision || verified.settings != projected {
        return Err("legacy-provider-v2-recovery-required");
    }
    let staged = StagedLegacyProviderConfig {
        before: Some(loaded),
        after: projected,
        after_revision: outcome.revision,
        secret_refs,
        created_secret_refs,
    };
    if let Err(error) = verify_legacy_secret_values(&secrets, &staged.secret_refs, secret_store) {
        rollback_legacy_provider_config(&staged, environment, secret_store)?;
        return Err(error);
    }
    Ok(Some(staged))
}

fn stage_legacy_secret_values(
    secrets: &[(LegacyProviderSecret, SecretValue)],
    secret_refs: &[(LegacyProviderSecret, SecretRef)],
    secret_store: &dyn SecretStore,
) -> Result<Vec<SecretRef>, &'static str> {
    let mut created_secret_refs = Vec::new();
    for ((provider, value), (reference_provider, reference)) in secrets.iter().zip(secret_refs) {
        if provider != reference_provider {
            remove_secret_refs(&created_secret_refs, secret_store);
            return Err("legacy-provider-secret-reference-mismatch");
        }
        match secret_store.resolve(reference) {
            Ok(observed) if observed.as_bytes() == value.as_bytes() => {}
            Ok(_) => {
                remove_secret_refs(&created_secret_refs, secret_store);
                return Err("legacy-provider-secret-reference-conflict");
            }
            Err(SecretStoreError::NotFound) => {
                if secret_store.store(reference, value).is_err() {
                    remove_secret_refs(&created_secret_refs, secret_store);
                    return Err("legacy-provider-secret-write-failed");
                }
                created_secret_refs.push(reference.clone());
            }
            Err(_) => {
                remove_secret_refs(&created_secret_refs, secret_store);
                return Err("legacy-provider-secret-store-unavailable");
            }
        }
    }
    Ok(created_secret_refs)
}

fn rollback_legacy_provider_config(
    staged: &StagedLegacyProviderConfig,
    environment: &CommandEnvironment,
    secret_store: &dyn SecretStore,
) -> Result<(), &'static str> {
    if let Some(before) = staged.before.as_ref() {
        let store = config_store(environment).map_err(|error| error.reason_code())?;
        let current = store.load().map_err(|error| error.reason_code())?;
        if current.revision != staged.after_revision || current.settings != staged.after {
            return Err("legacy-provider-rollback-conflict");
        }
        store
            .replace(current.revision, before.settings.clone())
            .map_err(|_| "legacy-provider-rollback-failed")?;
    }
    remove_secret_refs(&staged.created_secret_refs, secret_store);
    Ok(())
}

fn verify_legacy_provider_config(
    plan: &LegacyMigrationPlanV1,
    inventory: &LegacyMigrationInventory,
    environment: &CommandEnvironment,
    secret_store: &dyn SecretStore,
) -> Result<(), &'static str> {
    if !plan.items.iter().any(|item| {
        item.item_id == LegacyMigrationItemId::LegacyProviderConfig
            && item.action == LegacyMigrationAction::Convert
    }) {
        return Ok(());
    }
    let legacy = inventory
        .legacy_provider_config()
        .ok_or("legacy-provider-config-inventory-mismatch")?;
    let all_secrets = legacy
        .secret_values()
        .map_err(|error| error.reason_code())?;
    let all_secret_refs = all_secrets
        .iter()
        .map(|(provider, _)| {
            deterministic_legacy_secret_ref(plan, *provider).map(|reference| (*provider, reference))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let loaded = config_store(environment)
        .and_then(|store| store.load())
        .map_err(|error| error.reason_code())?;
    let projected = legacy
        .project_settings_with_resolutions(&loaded, &all_secret_refs, &plan.provider_resolutions)
        .map_err(|error| error.reason_code())?;
    if loaded.settings != projected {
        return Err("legacy-provider-v2-verification-failed");
    }
    let selected = all_secrets
        .into_iter()
        .zip(all_secret_refs)
        .filter(|((provider, _), (_, reference))| {
            projected_uses_legacy_secret(&projected, *provider, reference)
        })
        .collect::<Vec<_>>();
    let (secrets, secret_refs): (Vec<_>, Vec<_>) = selected.into_iter().unzip();
    verify_legacy_secret_values(&secrets, &secret_refs, secret_store)
}

fn projected_uses_legacy_secret(
    projected: &GlobalSettings,
    provider: LegacyProviderSecret,
    reference: &SecretRef,
) -> bool {
    match provider {
        LegacyProviderSecret::OpenAlex => {
            projected.providers.openalex.api_key_ref.as_ref() == Some(reference)
        }
        LegacyProviderSecret::SemanticScholar => {
            projected.providers.semantic_scholar.api_key_ref.as_ref() == Some(reference)
        }
        LegacyProviderSecret::Pubmed => {
            projected.providers.pubmed.api_key_ref.as_ref() == Some(reference)
        }
    }
}

fn verify_legacy_secret_values(
    secrets: &[(LegacyProviderSecret, qiongli_config::SecretValue)],
    secret_refs: &[(LegacyProviderSecret, SecretRef)],
    secret_store: &dyn SecretStore,
) -> Result<(), &'static str> {
    if secrets.len() != secret_refs.len() {
        return Err("legacy-provider-secret-reference-mismatch");
    }
    for ((provider, expected), (reference_provider, reference)) in secrets.iter().zip(secret_refs) {
        let matches = secret_store
            .resolve(reference)
            .is_ok_and(|observed| observed.as_bytes() == expected.as_bytes());
        if provider != reference_provider || !matches {
            return Err("legacy-provider-secret-verification-failed");
        }
    }
    Ok(())
}

fn deterministic_legacy_secret_ref(
    plan: &LegacyMigrationPlanV1,
    provider: LegacyProviderSecret,
) -> Result<SecretRef, &'static str> {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-LEGACY-PROVIDER-SECRET-REF-V1\0");
    hasher.update(plan.plan_id.as_bytes());
    hasher.update([0]);
    hasher.update(plan.plan_sha256.as_bytes());
    hasher.update([0]);
    hasher.update(match provider {
        LegacyProviderSecret::OpenAlex => b"openalex".as_slice(),
        LegacyProviderSecret::SemanticScholar => b"semantic-scholar".as_slice(),
        LegacyProviderSecret::Pubmed => b"pubmed".as_slice(),
    });
    let digest = hasher.finalize();
    let mut identifier = String::with_capacity(32);
    for byte in &digest[..16] {
        use std::fmt::Write as _;
        let _ = write!(identifier, "{byte:02x}");
    }
    SecretRef::parse(&format!("qsr1_{identifier}"))
        .map_err(|_| "legacy-provider-secret-reference-unavailable")
}

fn remove_secret_refs(refs: &[SecretRef], secret_store: &dyn SecretStore) {
    for reference in refs {
        let _ = secret_store.remove(reference);
    }
}

fn inventory(environment: &CommandEnvironment) -> Result<LegacyMigrationInventory, &'static str> {
    let clients = environment
        .client_inventory()
        .ok_or("legacy-migration-home-unavailable")?;
    let root = config_root(environment).map_err(|error| error.reason_code())?;
    Ok(discover_legacy_migration_with_config(&clients, Some(&root)))
}

fn staging_approvals(
    plan: &LegacyMigrationPlanV1,
    filesystem_write: bool,
    client_config_change: bool,
    secret_store_write: bool,
) -> Result<Vec<LegacyMigrationApproval>, &'static str> {
    let mut approvals = Vec::new();
    if filesystem_write {
        approvals.push(LegacyMigrationApproval::FilesystemWrite);
    }
    if client_config_change {
        approvals.push(LegacyMigrationApproval::ClientConfigChange);
    }
    if secret_store_write {
        approvals.push(LegacyMigrationApproval::SecretStoreWrite);
    }
    approvals.sort_unstable();
    if plan.required_approvals.iter().any(|required| {
        !approvals.contains(required)
            && matches!(
                required,
                LegacyMigrationApproval::FilesystemWrite
                    | LegacyMigrationApproval::ClientConfigChange
                    | LegacyMigrationApproval::SecretStoreWrite
            )
    }) {
        return Err("legacy-migration-staging-approval-missing");
    }
    Ok(approvals)
}

fn approvals_for_resume(
    plan: &LegacyMigrationPlanV1,
    cleanup: bool,
) -> Vec<LegacyMigrationApproval> {
    plan.required_approvals
        .iter()
        .copied()
        .filter(|approval| cleanup || *approval != LegacyMigrationApproval::LegacyCleanup)
        .collect()
}

pub(crate) fn migration_targets(plan: &LegacyMigrationPlanV1) -> Vec<ClientActivationTarget> {
    [ClientKind::Codex, ClientKind::ClaudeCode]
        .into_iter()
        .filter(|client| {
            plan.items.iter().any(|item| {
                item.item_id.client() == Some(*client)
                    && matches!(
                        item.action,
                        LegacyMigrationAction::Convert
                            | LegacyMigrationAction::Regenerate
                            | LegacyMigrationAction::RemoveAfterVerify
                    )
            })
        })
        .map(|client| match client {
            ClientKind::Codex => ClientActivationTarget::Codex,
            ClientKind::ClaudeCode => ClientActivationTarget::ClaudeCode,
        })
        .collect()
}

fn transition_items(
    receipt: &LegacyMigrationReceiptV1,
    state: LegacyMigrationState,
    from: LegacyMigrationReceiptItemState,
    to: LegacyMigrationReceiptItemState,
    result_code: &str,
) -> Result<LegacyMigrationReceiptV1, &'static str> {
    let items = receipt
        .items
        .iter()
        .map(|item| LegacyMigrationReceiptItemV1 {
            item_id: item.item_id,
            state: if item.state == from { to } else { item.state },
            result_code: result_code.to_owned(),
        })
        .collect();
    advance_legacy_migration_receipt(receipt, state, items).map_err(|error| error.reason_code())
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "system-clock-unavailable")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use qiongli_config::ConfigError;
    use qiongli_project::{
        ApprovedProjectMutation, ProjectError, ProjectHealth, ProjectKind,
        ProjectRegistrationOptions, ProjectStateService,
    };
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct PersistedStateFixtures {
        predecessors: Vec<PersistedStateFixture>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct PersistedStateFixture {
        window_position: String,
        release_tag: String,
        peeled_commit: String,
        project_files: BTreeMap<String, String>,
        provider_config: serde_json::Value,
    }

    #[derive(Default)]
    struct TestSecretStore {
        values: Mutex<BTreeMap<String, Vec<u8>>>,
    }

    impl SecretStore for TestSecretStore {
        fn status(&self) -> SecretStoreStatus {
            SecretStoreStatus::Available
        }

        fn resolve(
            &self,
            secret_ref: &SecretRef,
        ) -> Result<qiongli_config::SecretValue, SecretStoreError> {
            let values = self
                .values
                .lock()
                .map_err(|_| SecretStoreError::Unavailable)?;
            let value = values
                .get(secret_ref.storage_key())
                .cloned()
                .ok_or(SecretStoreError::NotFound)?;
            qiongli_config::SecretValue::new(value).map_err(|_| SecretStoreError::PersistenceFailed)
        }

        fn store(
            &self,
            secret_ref: &SecretRef,
            value: &qiongli_config::SecretValue,
        ) -> Result<(), SecretStoreError> {
            self.values
                .lock()
                .map_err(|_| SecretStoreError::Unavailable)?
                .insert(
                    secret_ref.storage_key().to_owned(),
                    value.as_bytes().to_vec(),
                );
            Ok(())
        }

        fn remove(&self, secret_ref: &SecretRef) -> Result<(), SecretStoreError> {
            self.values
                .lock()
                .map_err(|_| SecretStoreError::Unavailable)?
                .remove(secret_ref.storage_key())
                .map(|_| ())
                .ok_or(SecretStoreError::NotFound)
        }
    }

    fn fixture(label: &str) -> (PathBuf, CommandEnvironment, EmbeddedContent) {
        let requested = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!(
                "qiongli-legacy-migration-cli-{label}-{}",
                std::process::id()
            ));
        fs::create_dir(&requested).unwrap();
        let root = fs::canonicalize(&requested).unwrap();
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        (
            requested,
            CommandEnvironment::with_paths(None::<OsString>, Some(home), None),
            crate::embedded_content().unwrap(),
        )
    }

    #[test]
    fn rel_902_migrates_and_rolls_back_both_supported_predecessors() {
        let fixtures: PersistedStateFixtures = serde_json::from_str(include_str!(
            "../tests/fixtures/rel-902-persisted-state.json"
        ))
        .unwrap();
        assert_eq!(fixtures.predecessors.len(), 2);
        assert_eq!(
            fixtures
                .predecessors
                .iter()
                .map(|row| (
                    row.window_position.as_str(),
                    row.release_tag.as_str(),
                    row.peeled_commit.as_str(),
                ))
                .collect::<Vec<_>>(),
            [
                (
                    "N-1",
                    "v1.19.0-beta.1",
                    "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f",
                ),
                (
                    "N-2",
                    "v1.18.0-beta.3",
                    "12aea420bff9a3fbfa5e421c482ae8da2588c9ed",
                ),
            ]
        );

        for (index, row) in fixtures.predecessors.iter().enumerate() {
            let (root, environment, _content) = fixture(&format!("rel-902-{index}"));
            let root = fs::canonicalize(root).unwrap();
            let source = root.join("legacy-project");
            for (relative, contents) in &row.project_files {
                let path = source.join(relative);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, contents).unwrap();
            }
            let project_before = row
                .project_files
                .keys()
                .map(|relative| (relative.clone(), fs::read(source.join(relative)).unwrap()))
                .collect::<BTreeMap<_, _>>();

            let provider_path = root.join("home/.config/qiongli/providers.json");
            fs::create_dir_all(provider_path.parent().unwrap()).unwrap();
            fs::write(
                &provider_path,
                serde_json::to_vec_pretty(&row.provider_config).unwrap(),
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;

                fs::set_permissions(&provider_path, fs::Permissions::from_mode(0o600)).unwrap();
            }
            let provider_before = fs::read(&provider_path).unwrap();
            let settings_before = config_store(&environment).unwrap().load().unwrap().settings;
            let secret_store = TestSecretStore::default();
            let secrets_before = secret_store.values.lock().unwrap().clone();

            let projects = ProjectStateService::new(config_root(&environment).unwrap());
            let destination = root.join("migrated-project");
            let migration = projects
                .preview_migrate(
                    &source,
                    &destination,
                    ProjectRegistrationOptions::new(
                        format!("REL-902 {}", row.window_position),
                        ProjectKind::Article,
                    ),
                    1_800_000_000 + index as u64,
                )
                .unwrap();
            assert!(migration.preview().source_retained);
            projects
                .apply_migration(
                    &migration,
                    &ApprovedProjectMutation::new(migration.preview().plan_digest.clone(), true),
                    1_800_000_010 + index as u64,
                )
                .unwrap();
            let library = projects.snapshot().unwrap();
            assert_eq!(library.projects.len(), 1);
            assert_eq!(
                library.projects[0].display_name,
                format!("REL-902 {}", row.window_position)
            );
            assert_eq!(
                fs::read(destination.join("context/research_state.md")).unwrap(),
                project_before["context/research_state.md"]
            );

            let inventory = inventory(&environment).unwrap();
            let plan_id = format!("rel-902-{index}");
            let provider_plan = preview_legacy_migration(
                &inventory,
                LegacyMigrationPlanInput {
                    plan_id: &plan_id,
                    product_version: env!("CARGO_PKG_VERSION"),
                    source_commit: "1111111111111111111111111111111111111111",
                    resource_pack_sha256:
                        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    created_at_unix: 1_800_000_020 + index as u64,
                    provider_resolutions: &[],
                },
            )
            .unwrap();
            validate_legacy_provider_destination(&provider_plan, &inventory, &environment).unwrap();
            let staged = stage_legacy_provider_config(
                &provider_plan,
                &inventory,
                &environment,
                &secret_store,
            )
            .unwrap()
            .unwrap();
            verify_legacy_provider_config(&provider_plan, &inventory, &environment, &secret_store)
                .unwrap();
            let migrated_settings = config_store(&environment).unwrap().load().unwrap().settings;
            assert!(migrated_settings.providers.openalex.api_key_ref.is_some());
            let stored_secrets = secret_store.values.lock().unwrap();
            assert_eq!(stored_secrets.len(), 1);
            assert_eq!(
                stored_secrets.values().next().unwrap(),
                row.provider_config["providers"]["openalex"]["api_key"]
                    .as_str()
                    .unwrap()
                    .as_bytes()
            );
            drop(stored_secrets);

            let rollback = projects
                .preview_migration_rollback(&source, &destination)
                .unwrap();
            projects
                .apply_migration_rollback(
                    &rollback,
                    &ApprovedProjectMutation::new(rollback.preview().plan_digest.clone(), true),
                )
                .unwrap();
            rollback_legacy_provider_config(&staged, &environment, &secret_store).unwrap();

            assert!(!destination.exists());
            assert!(projects.snapshot().unwrap().projects.is_empty());
            assert_eq!(fs::read_dir(&source).unwrap().count(), 2);
            assert_eq!(fs::read_dir(source.join("context")).unwrap().count(), 1);
            assert_eq!(fs::read_dir(source.join(".qiongli")).unwrap().count(), 1);
            assert_eq!(
                row.project_files
                    .keys()
                    .map(|relative| (relative.clone(), fs::read(source.join(relative)).unwrap(),))
                    .collect::<BTreeMap<_, _>>(),
                project_before
            );
            assert_eq!(fs::read(&provider_path).unwrap(), provider_before);
            assert_eq!(
                config_store(&environment).unwrap().load().unwrap().settings,
                settings_before
            );
            assert_eq!(*secret_store.values.lock().unwrap(), secrets_before);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn rel_903_future_project_and_global_state_fail_closed_without_writes() {
        let (root, environment, _content) = fixture("rel-903");
        let root = fs::canonicalize(root).unwrap();
        let config_root = config_root(&environment).unwrap();
        let projects = ProjectStateService::new(config_root.clone());
        let project_root = root.join("RESEARCH/future-project");
        let create = projects
            .preview_create(
                &project_root,
                ProjectRegistrationOptions::new("Future project", ProjectKind::Article),
                1,
            )
            .unwrap();
        let project_id = create.preview().project_id.clone();
        projects
            .apply(
                &create,
                &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();

        let settings = config_store(&environment).unwrap();
        settings.replace(0, GlobalSettings::default()).unwrap();
        let settings_path = config_root.state_root().join("settings.json");
        let library_path = config_root
            .state_root()
            .join("research-library/library.json");
        let manifest_path = project_root.join("context/project_manifest.json");
        let future_document = |bytes: &[u8]| {
            let current = std::str::from_utf8(bytes).unwrap();
            let future = current
                .replace("\"schema_version\": 1", "\"schema_version\": 2")
                .replace("\"schema_version\":1", "\"schema_version\":2");
            assert_ne!(future, current);
            future.into_bytes()
        };

        let future_settings = future_document(&fs::read(&settings_path).unwrap());
        fs::write(&settings_path, &future_settings).unwrap();
        assert_eq!(
            settings.load(),
            Err(ConfigError::UnsupportedSchema { observed: Some(2) })
        );
        assert_eq!(fs::read(&settings_path).unwrap(), future_settings);

        let current_library = fs::read(&library_path).unwrap();
        let future_library = future_document(&current_library);
        fs::write(&library_path, &future_library).unwrap();
        assert!(matches!(
            projects.snapshot(),
            Err(ProjectError::InvalidLibraryDocument)
        ));
        assert_eq!(fs::read(&library_path).unwrap(), future_library);
        fs::write(&library_path, current_library).unwrap();

        let future_manifest = future_document(&fs::read(&manifest_path).unwrap());
        fs::write(&manifest_path, &future_manifest).unwrap();
        assert_eq!(
            projects.snapshot().unwrap().projects[0].health,
            ProjectHealth::InspectionBlocked
        );
        assert!(matches!(
            projects.preview_refresh(&project_id, 2),
            Err(ProjectError::InvalidProjectDocument)
        ));
        assert_eq!(fs::read(&manifest_path).unwrap(), future_manifest);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inspect_is_available_without_packaged_product_authority() {
        let (root, environment, content) = fixture("inspect");
        let output = execute(LegacyMigrationCliCommand::Inspect, &environment, &content).unwrap();
        let LegacyMigrationCliOutput::Inspect { inventory, .. } = output else {
            panic!("inspect output expected");
        };
        assert_eq!(inventory.detected_item_count, 0);
        assert_eq!(inventory.items.len(), 9);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preview_does_not_bypass_source_build_product_control() {
        let (root, environment, content) = fixture("source-preview");
        assert_eq!(
            execute(
                LegacyMigrationCliCommand::Preview {
                    provider_resolutions: Vec::new(),
                },
                &environment,
                &content,
            )
            .unwrap_err(),
            "source-build-read-only"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provider_stage_moves_plaintext_secrets_into_store_and_rolls_back_exactly() {
        let (root, environment, _content) = fixture("provider-stage");
        let provider_path = root.join("home/.config/qiongli/providers.json");
        fs::create_dir_all(provider_path.parent().unwrap()).unwrap();
        fs::write(
            &provider_path,
            br#"{
  "version": 1,
  "providers": {
    "openalex": {
      "enabled": true,
      "email": "person@example.org",
      "api_key": "legacy-openalex-secret"
    },
    "crossref": {"email": "crossref@example.org"},
    "arxiv": {"enabled": false}
  }
}"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;

            fs::set_permissions(&provider_path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let inventory = inventory(&environment).unwrap();
        let plan = preview_legacy_migration(
            &inventory,
            LegacyMigrationPlanInput {
                plan_id: "migration-provider-stage",
                product_version: "2.0.0-alpha.2",
                source_commit: "0123456789abcdef0123456789abcdef01234567",
                resource_pack_sha256:
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                created_at_unix: 1_800_000_000,
                provider_resolutions: &[],
            },
        )
        .unwrap();
        validate_legacy_provider_destination(&plan, &inventory, &environment).unwrap();
        let secret_store = TestSecretStore::default();
        let staged = stage_legacy_provider_config(&plan, &inventory, &environment, &secret_store)
            .unwrap()
            .unwrap();

        let loaded = config_store(&environment).unwrap().load().unwrap();
        assert!(loaded.settings.providers.openalex.enabled);
        assert!(loaded.settings.providers.openalex.api_key_ref.is_some());
        assert!(loaded.settings.providers.crossref.enabled);
        assert!(!loaded.settings.providers.arxiv.enabled);
        assert_eq!(secret_store.values.lock().unwrap().len(), 1);
        verify_legacy_provider_config(&plan, &inventory, &environment, &secret_store).unwrap();

        rollback_legacy_provider_config(&staged, &environment, &secret_store).unwrap();
        assert_eq!(
            config_store(&environment)
                .unwrap()
                .load()
                .unwrap()
                .settings
                .providers,
            ProviderSettings::default()
        );
        assert!(secret_store.values.lock().unwrap().is_empty());

        let secret_refs = inventory
            .legacy_provider_config()
            .unwrap()
            .secret_values()
            .unwrap()
            .into_iter()
            .map(|(provider, _)| {
                (
                    provider,
                    deterministic_legacy_secret_ref(&plan, provider).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let store = config_store(&environment).unwrap();
        let current = store.load().unwrap();
        let projected = inventory
            .legacy_provider_config()
            .unwrap()
            .project_settings(&current, &secret_refs)
            .unwrap();
        store.replace(current.revision, projected.clone()).unwrap();

        let resumed = stage_legacy_provider_config(&plan, &inventory, &environment, &secret_store)
            .unwrap()
            .unwrap();
        assert!(resumed.before.is_none());
        assert_eq!(secret_store.values.lock().unwrap().len(), 1);
        rollback_legacy_provider_config(&resumed, &environment, &secret_store).unwrap();
        assert_eq!(
            config_store(&environment).unwrap().load().unwrap().settings,
            projected
        );
        assert!(secret_store.values.lock().unwrap().is_empty());

        let store = config_store(&environment).unwrap();
        let current = store.load().unwrap();
        let mut conflicting = current.settings.clone();
        conflicting.providers.openalex.enabled = false;
        store.replace(current.revision, conflicting).unwrap();
        assert_eq!(
            validate_legacy_provider_destination(&plan, &inventory, &environment).unwrap_err(),
            "legacy-provider-v2-conflict"
        );
        let resolved_plan = preview_legacy_migration(
            &inventory,
            LegacyMigrationPlanInput {
                plan_id: "migration-provider-resolved",
                product_version: "2.0.0-alpha.2",
                source_commit: "0123456789abcdef0123456789abcdef01234567",
                resource_pack_sha256:
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                created_at_unix: 1_800_000_001,
                provider_resolutions: &[LegacyProviderResolution {
                    provider: qiongli_config::LegacyProviderId::OpenAlex,
                    strategy: qiongli_config::LegacyProviderResolutionStrategy::UseLegacy,
                }],
            },
        )
        .unwrap();
        validate_legacy_provider_destination(&resolved_plan, &inventory, &environment).unwrap();
        let resolved =
            stage_legacy_provider_config(&resolved_plan, &inventory, &environment, &secret_store)
                .unwrap()
                .unwrap();
        assert!(
            config_store(&environment)
                .unwrap()
                .load()
                .unwrap()
                .settings
                .providers
                .openalex
                .enabled
        );
        assert_eq!(secret_store.values.lock().unwrap().len(), 1);
        rollback_legacy_provider_config(&resolved, &environment, &secret_store).unwrap();
        assert!(secret_store.values.lock().unwrap().is_empty());
        fs::remove_dir_all(root).unwrap();
    }
}
