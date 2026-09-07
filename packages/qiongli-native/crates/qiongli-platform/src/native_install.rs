use std::fs;
use std::path::{Path, PathBuf};

use qiongli_content::LoadedResourcePack;
use same_file::Handle;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::transaction::{
    create_private_new_file, ensure_destination_absent, path_exists, read_private_file,
    rename_path, sync_directory, transaction_id, write_sync_file,
};
use crate::{
    AllowedRootV1, ApprovalRequirement, ApprovedInstallPlan, ApprovedManagedRoot,
    ArtifactIdentityV1, CapabilityProfile, InstallActionV1, InstallDisposition,
    InstallLifecycleKind, InstallOperationV1, InstallPlanDraftV1, InstallPlanMetadataV1,
    InstallPlanV1, InstallScope, LifecycleDisposition, NativeArtifactTarget,
    NativePortableArchiveError, NativePortableArchiveTarget, OwnershipMarkerV1, PlanStateV1,
    PlatformError, ProductId, SymbolicRoot, TargetDescriptorV1, TransactionError,
    VerifiedInstallPlan, VerifiedNativeArtifact, VerifiedNativePortableArchive,
    VerifiedNativeReleaseEnvelope, approve_native_artifact_target, extract_native_portable_archive,
    native_artifact_id, observed_plan_state_sha256, verify_native_artifact,
    verify_native_portable_archive,
};

pub const NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_PAYLOAD_INSTALL_STATE_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_PAYLOAD_INSTALL_JOURNAL_SCHEMA_VERSION: u32 = 1;

const MAX_STATE_BYTES: u64 = 1024 * 1024;
const MAX_IDENTIFIER_BYTES: usize = 128;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const ENTRY_KEY: &str = "native-payload";
const STATE_FILE_PREFIX: &str = ".qiongli-";
const STATE_FILE_SUFFIX: &str = ".json";
const JOURNAL_FILE_NAME: &str = ".qiongli-native-payload-transaction.json";
const QUARANTINE_PREFIX: &str = ".qiongli-native-payload-quarantine-";

#[must_use]
pub fn native_payload_install_id(archive: &VerifiedNativePortableArchive) -> String {
    format!("native-payload-{}", archive.archive_sha256())
}

pub fn preview_native_payload_install(
    metadata: InstallPlanMetadataV1,
    release: &VerifiedNativeReleaseEnvelope,
    target: TargetDescriptorV1,
    root: AllowedRootV1,
) -> Result<InstallPlanV1, PlatformError> {
    let verified_grant = release.launch_grant();
    let archive = release.archive();
    let manifest = archive.payload().manifest();
    if archive.artifact() != &verified_grant.grant().artifact
        || manifest.artifact != verified_grant.grant().artifact
        || manifest.binary_sha256 != verified_grant.grant().binary_sha256
        || manifest.content.pack_sha256 != verified_grant.grant().resource_pack_sha256
        || root.root != SymbolicRoot::QiongliManagedData
        || metadata.created_at_unix < release.envelope().not_before_unix
        || metadata.created_at_unix < release.verified_at_unix()
        || metadata.expires_at_unix > release.envelope().expires_at_unix
    {
        return Err(PlatformError::InstallPlanTargetMismatch);
    }
    let artifact_id =
        native_artifact_id(archive.artifact()).map_err(|_| PlatformError::InvalidInstallPlan)?;
    let install_id = native_payload_install_id(archive);
    let ownership = OwnershipMarkerV1 {
        schema_version: 1,
        product: ProductId::Qiongli,
        install_id: install_id.clone(),
        artifact_digest_sha256: verified_grant.signed_payload_sha256().to_string(),
    };
    let content_sha256 = manifest.artifact_content_root_sha256.clone();
    InstallPlanV1::build(
        metadata,
        verified_grant,
        InstallPlanDraftV1 {
            target,
            allowed_roots: vec![root.clone()],
            operations: vec![InstallOperationV1 {
                operation_id: "install-native-payload".to_string(),
                action: InstallActionV1::InstallNativePayload {
                    root_id: root.id.clone(),
                    entry_key: ENTRY_KEY.to_string(),
                    relative_path: artifact_id,
                    release_envelope_sha256: release.signed_payload_sha256().to_string(),
                    archive_sha256: archive.archive_sha256().to_string(),
                    manifest_sha256: archive.manifest_sha256().to_string(),
                    pack_sha256: manifest.content.pack_sha256.clone(),
                    artifact_content_root_sha256: content_sha256.clone(),
                    binary_sha256: manifest.binary_sha256.clone(),
                    ownership: ownership.clone(),
                },
                precondition: PlanStateV1::Missing,
                observed_state_sha256: observed_plan_state_sha256(&PlanStateV1::Missing)?,
                postcondition: PlanStateV1::Managed {
                    ownership: ownership.clone(),
                    content_sha256: content_sha256.clone(),
                },
                inverse: InstallActionV1::RemoveManagedEntry {
                    root_id: root.id,
                    entry_key: ENTRY_KEY.to_string(),
                    expected_ownership: ownership,
                    expected_sha256: content_sha256,
                },
            }],
            approvals_required: vec![ApprovalRequirement::FilesystemWrite],
            outstanding_host_action: None,
        },
    )
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativePayloadOperationReceiptV1 {
    pub operation_id: String,
    pub root_id: String,
    pub entry_key: String,
    pub relative_path: String,
    pub ownership: OwnershipMarkerV1,
    pub release_envelope_sha256: String,
    pub archive_sha256: String,
    pub manifest_sha256: String,
    pub pack_sha256: String,
    pub artifact_content_root_sha256: String,
    pub binary_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativePayloadInstallReceiptV1 {
    pub schema_version: u32,
    pub transaction_id: String,
    pub plan_id: String,
    pub semantic_digest_sha256: String,
    pub install_id: String,
    pub artifact: ArtifactIdentityV1,
    pub target: TargetDescriptorV1,
    pub operation: NativePayloadOperationReceiptV1,
    pub applied_at_unix: u64,
    pub replaces_transaction_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativePayloadLifecycleReceiptV1 {
    pub schema_version: u32,
    pub transaction_id: String,
    pub install_id: String,
    pub prior_transaction_id: String,
    pub kind: InstallLifecycleKind,
    pub completed_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativePayloadInstallStateV1 {
    pub schema_version: u32,
    pub generation: u64,
    pub install_id: String,
    pub active: Option<NativePayloadInstallReceiptV1>,
    pub last_lifecycle: Option<NativePayloadLifecycleReceiptV1>,
}

impl NativePayloadInstallStateV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, TransactionError> {
        if input.len() as u64 > MAX_STATE_BYTES {
            return Err(TransactionError::InvalidReceipt);
        }
        let state: Self =
            serde_json::from_slice(input).map_err(|_| TransactionError::InvalidReceipt)?;
        state.validate()?;
        if canonical_json(&state)? != input {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(state)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, TransactionError> {
        self.validate()?;
        let bytes = canonical_json(self)?;
        if bytes.len() as u64 > MAX_STATE_BYTES {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(bytes)
    }

    fn validate(&self) -> Result<(), TransactionError> {
        if self.schema_version != NATIVE_PAYLOAD_INSTALL_STATE_SCHEMA_VERSION
            || self.generation == 0
            || self.generation > JCS_MAX_SAFE_INTEGER
            || !valid_identifier(&self.install_id)
            || (self.active.is_none() && self.last_lifecycle.is_none())
        {
            return Err(TransactionError::InvalidReceipt);
        }
        if let Some(active) = &self.active {
            active.validate()?;
            if active.install_id != self.install_id {
                return Err(TransactionError::InvalidReceipt);
            }
        }
        if let Some(lifecycle) = &self.last_lifecycle {
            lifecycle.validate()?;
            if lifecycle.install_id != self.install_id {
                return Err(TransactionError::InvalidReceipt);
            }
        }
        Ok(())
    }
}

impl NativePayloadInstallReceiptV1 {
    fn validate(&self) -> Result<(), TransactionError> {
        self.artifact
            .validate()
            .map_err(|_| TransactionError::InvalidReceipt)?;
        let artifact_id =
            native_artifact_id(&self.artifact).map_err(|_| TransactionError::InvalidReceipt)?;
        let expected_install_id = format!("native-payload-{}", self.operation.archive_sha256);
        if self.schema_version != NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION
            || !valid_identifier(&self.transaction_id)
            || !valid_identifier(&self.plan_id)
            || !valid_digest(&self.semantic_digest_sha256)
            || self.install_id != expected_install_id
            || self.applied_at_unix > JCS_MAX_SAFE_INTEGER
            || self.target.profile != CapabilityProfile::Lite
            || self.target.scope != InstallScope::User
            || self.target.os != self.artifact.os
            || self.target.arch != self.artifact.arch
            || self.target.adapter_version != 1
            || self.operation.relative_path != artifact_id
            || self.operation.ownership.install_id != self.install_id
            || self.operation.ownership.product != ProductId::Qiongli
            || self.operation.ownership.schema_version != 1
            || self
                .replaces_transaction_id
                .as_ref()
                .is_some_and(|value| !valid_identifier(value))
        {
            return Err(TransactionError::InvalidReceipt);
        }
        self.operation.validate()
    }
}

impl NativePayloadOperationReceiptV1 {
    fn validate(&self) -> Result<(), TransactionError> {
        if !valid_identifier(&self.operation_id)
            || !valid_identifier(&self.root_id)
            || self.entry_key != ENTRY_KEY
            || !valid_leaf(&self.relative_path)
            || !valid_digest(&self.ownership.artifact_digest_sha256)
            || !valid_digest(&self.release_envelope_sha256)
            || !valid_digest(&self.archive_sha256)
            || !valid_digest(&self.manifest_sha256)
            || !valid_digest(&self.pack_sha256)
            || !valid_digest(&self.artifact_content_root_sha256)
            || !valid_digest(&self.binary_sha256)
        {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(())
    }
}

impl NativePayloadLifecycleReceiptV1 {
    fn validate(&self) -> Result<(), TransactionError> {
        if self.schema_version != NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION
            || !valid_identifier(&self.transaction_id)
            || !valid_identifier(&self.install_id)
            || !valid_identifier(&self.prior_transaction_id)
            || self.completed_at_unix > JCS_MAX_SAFE_INTEGER
        {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePayloadInstallCommit {
    pub disposition: InstallDisposition,
    pub receipt: NativePayloadInstallReceiptV1,
    pub cleanup_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePayloadInstallVerification {
    pub receipt: NativePayloadInstallReceiptV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePayloadLifecycleCommit {
    pub disposition: LifecycleDisposition,
    pub receipt: NativePayloadLifecycleReceiptV1,
    pub cleanup_required: bool,
}

#[derive(Clone, Debug)]
pub struct ManagedNativePayloadExecutor {
    root: ApprovedManagedRoot,
}

impl ManagedNativePayloadExecutor {
    #[must_use]
    pub const fn new(root: ApprovedManagedRoot) -> Self {
        Self { root }
    }

    pub fn apply(
        &self,
        plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        pack: &LoadedResourcePack<'_>,
        release: &VerifiedNativeReleaseEnvelope,
        now_unix: u64,
    ) -> Result<NativePayloadInstallCommit, TransactionError> {
        self.apply_with(
            plan,
            approval,
            pack,
            release,
            now_unix,
            ApplyKind::Fresh,
            &NoFaults,
        )
    }

    pub fn repair(
        &self,
        plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        pack: &LoadedResourcePack<'_>,
        release: &VerifiedNativeReleaseEnvelope,
        now_unix: u64,
    ) -> Result<NativePayloadInstallCommit, TransactionError> {
        self.apply_with(
            plan,
            approval,
            pack,
            release,
            now_unix,
            ApplyKind::Repair,
            &NoFaults,
        )
    }

    pub fn verify(
        &self,
        install_id: &str,
        pack: &LoadedResourcePack<'_>,
    ) -> Result<NativePayloadInstallVerification, TransactionError> {
        self.root.validate()?;
        ensure_no_journal(self.root.path(), install_id)?;
        let state =
            load_state(self.root.path(), install_id)?.ok_or(TransactionError::ReceiptMissing)?;
        let receipt = state.active.ok_or(TransactionError::ReceiptMissing)?;
        verify_active_receipt(self.root.path(), self.root.root_id(), pack, &receipt)?;
        Ok(NativePayloadInstallVerification { receipt })
    }

    /// Verifies the complete active payload against its persisted receipt, including
    /// older resource packs. This is ownership evidence, not signed launch authority.
    pub fn verify_receipt_owned(
        &self,
        install_id: &str,
    ) -> Result<NativePayloadInstallVerification, TransactionError> {
        self.root.validate()?;
        ensure_no_journal(self.root.path(), install_id)?;
        let state =
            load_state(self.root.path(), install_id)?.ok_or(TransactionError::ReceiptMissing)?;
        let receipt = state.active.ok_or(TransactionError::ReceiptMissing)?;
        receipt.validate()?;
        if receipt.operation.root_id != self.root.root_id() {
            return Err(TransactionError::ManagedStateDrift);
        }
        let target = approve_native_artifact_target(
            self.root.path().join(&receipt.operation.relative_path),
            &receipt.artifact,
        )
        .map_err(|_| TransactionError::ManagedStateDrift)?;
        let verified = crate::native_artifact::verify_receipt_owned_native_artifact(
            &target,
            &receipt.operation.manifest_sha256,
        )
        .map_err(|_| TransactionError::ManagedStateDrift)?;
        verify_artifact_against_receipt(&target, &receipt, verified)?;
        Ok(NativePayloadInstallVerification { receipt })
    }

    pub fn remove(
        &self,
        install_id: &str,
        pack: &LoadedResourcePack<'_>,
        now_unix: u64,
    ) -> Result<NativePayloadLifecycleCommit, TransactionError> {
        self.lifecycle_with(
            install_id,
            pack,
            now_unix,
            InstallLifecycleKind::Removed,
            &NoFaults,
        )
    }

    pub fn rollback(
        &self,
        install_id: &str,
        pack: &LoadedResourcePack<'_>,
        now_unix: u64,
    ) -> Result<NativePayloadLifecycleCommit, TransactionError> {
        self.lifecycle_with(
            install_id,
            pack,
            now_unix,
            InstallLifecycleKind::RolledBack,
            &NoFaults,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_with<F: NativePayloadFaults>(
        &self,
        verified_plan: &VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        pack: &LoadedResourcePack<'_>,
        release: &VerifiedNativeReleaseEnvelope,
        now_unix: u64,
        kind: ApplyKind,
        faults: &F,
    ) -> Result<NativePayloadInstallCommit, TransactionError> {
        self.root.validate()?;
        let executable = ExecutableNativePayload::from_plan(
            verified_plan,
            approval,
            pack,
            release,
            &self.root,
            now_unix,
        )?;
        ensure_no_journal(self.root.path(), executable.install_id())?;
        let prior_state = load_state(self.root.path(), executable.install_id())?;

        match kind {
            ApplyKind::Fresh => {
                if let Some(active) = prior_state.as_ref().and_then(|state| state.active.as_ref()) {
                    if executable.matches_active(active) {
                        verify_active_receipt(self.root.path(), self.root.root_id(), pack, active)?;
                        return Ok(NativePayloadInstallCommit {
                            disposition: InstallDisposition::AlreadyApplied,
                            receipt: active.clone(),
                            cleanup_required: false,
                        });
                    }
                    return Err(TransactionError::DestinationConflict);
                }
                ensure_destination_absent(&executable.destination)?;
            }
            ApplyKind::Repair => {
                let active = prior_state
                    .as_ref()
                    .and_then(|state| state.active.as_ref())
                    .ok_or(TransactionError::ReceiptMissing)?;
                if !executable.matches_active(active) {
                    return Err(TransactionError::UnsupportedPlan);
                }
                if path_exists(&executable.destination)? {
                    verify_active_receipt(self.root.path(), self.root.root_id(), pack, active)?;
                    return Ok(NativePayloadInstallCommit {
                        disposition: InstallDisposition::AlreadyHealthy,
                        receipt: active.clone(),
                        cleanup_required: false,
                    });
                }
            }
        }

        let transaction_id = transaction_id();
        let prior_state_sha256 = prior_state
            .as_ref()
            .map(NativePayloadInstallStateV1::to_canonical_json)
            .transpose()?
            .as_deref()
            .map(sha256_hex);
        let journal_value = NativePayloadJournalV1 {
            schema_version: NATIVE_PAYLOAD_INSTALL_JOURNAL_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            kind: match kind {
                ApplyKind::Fresh => JournalKind::Apply,
                ApplyKind::Repair => JournalKind::Repair,
            },
            install_id: executable.install_id().to_string(),
            plan_digest_sha256: Some(executable.plan_digest.to_string()),
            prior_state_sha256,
            target_leaf: executable.relative_path.to_string(),
            started_at_unix: now_unix,
        };
        let mut journal = NativePayloadJournalGuard::create(self.root.path(), journal_value)?;
        if let Err(error) = faults.check(FaultPoint::AfterJournal) {
            journal.finish()?;
            return Err(error);
        }

        if self.root.validate().is_err() {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        executable.verify_archive(pack)?;
        ensure_destination_absent(&executable.destination)?;
        if state_digest(self.root.path(), executable.install_id())?
            != prior_state
                .as_ref()
                .map(NativePayloadInstallStateV1::to_canonical_json)
                .transpose()?
                .as_deref()
                .map(sha256_hex)
        {
            journal.finish()?;
            return Err(TransactionError::ObservedStateMismatch);
        }

        let replaces_transaction_id = prior_state
            .as_ref()
            .and_then(|state| state.active.as_ref())
            .map(|receipt| receipt.transaction_id.clone());
        let receipt =
            executable.build_receipt(transaction_id.clone(), now_unix, replaces_transaction_id);
        let destination =
            approve_native_artifact_target(&executable.destination, &verified_plan.plan().artifact)
                .map_err(|_| TransactionError::UnsafeManagedRoot)?;
        if let Err(error) =
            extract_native_portable_archive(pack, executable.archive_target(), &destination)
        {
            match error {
                NativePortableArchiveError::ExtractionFailed => {
                    return self.fail_after_possible_extract(
                        &mut journal,
                        pack,
                        &receipt,
                        ExtractedTargetOwnership::Uncertain,
                        faults,
                        TransactionError::NativePayloadInstallFailed,
                    );
                }
                NativePortableArchiveError::DestinationExists
                | NativePortableArchiveError::DestinationBusy => {
                    journal.finish()?;
                    return Err(TransactionError::DestinationConflict);
                }
                NativePortableArchiveError::DestinationUnsafe => {
                    journal.finish()?;
                    return Err(TransactionError::UnsafeManagedRoot);
                }
                _ => {
                    journal.finish()?;
                    return Err(TransactionError::NativePayloadInstallFailed);
                }
            }
        }
        if verify_target_against_receipt(pack, &destination, &receipt).is_err() {
            return self.fail_after_possible_extract(
                &mut journal,
                pack,
                &receipt,
                ExtractedTargetOwnership::CreatedByTransaction,
                faults,
                TransactionError::ManagedStateDrift,
            );
        }
        if let Err(error) = faults.check(FaultPoint::AfterExtract) {
            return self.fail_after_possible_extract(
                &mut journal,
                pack,
                &receipt,
                ExtractedTargetOwnership::CreatedByTransaction,
                faults,
                error,
            );
        }

        let next_state = NativePayloadInstallStateV1 {
            schema_version: NATIVE_PAYLOAD_INSTALL_STATE_SCHEMA_VERSION,
            generation: next_generation(prior_state.as_ref())?,
            install_id: executable.install_id().to_string(),
            active: Some(receipt.clone()),
            last_lifecycle: prior_state.and_then(|state| state.last_lifecycle),
        };
        let commit_result = faults
            .check(FaultPoint::BeforeStateCommit)
            .and_then(|()| persist_state(self.root.path(), &next_state, &transaction_id, faults));
        if let Err(error) = commit_result {
            if error == TransactionError::RecoveryRequired {
                journal.retain();
                return Err(error);
            }
            if faults.check(FaultPoint::DuringRollback).is_err()
                || rollback_created_payload(self.root.path(), pack, &receipt, &transaction_id)
                    .is_err()
            {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
            journal.finish()?;
            return Err(error);
        }

        let cleanup_required = journal.finish().is_err();
        Ok(NativePayloadInstallCommit {
            disposition: match kind {
                ApplyKind::Fresh => InstallDisposition::Applied,
                ApplyKind::Repair => InstallDisposition::Repaired,
            },
            receipt,
            cleanup_required,
        })
    }

    fn fail_after_possible_extract<F: NativePayloadFaults>(
        &self,
        journal: &mut NativePayloadJournalGuard,
        pack: &LoadedResourcePack<'_>,
        receipt: &NativePayloadInstallReceiptV1,
        ownership: ExtractedTargetOwnership,
        faults: &F,
        original_error: TransactionError,
    ) -> Result<NativePayloadInstallCommit, TransactionError> {
        let target = self.root.path().join(&receipt.operation.relative_path);
        let target_exists = match path_exists(&target) {
            Ok(exists) => exists,
            Err(_) => {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
        };
        if !target_exists {
            journal.finish()?;
            return Err(original_error);
        }
        if ownership == ExtractedTargetOwnership::Uncertain {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        let approved = match approve_native_artifact_target(&target, &receipt.artifact) {
            Ok(target) => target,
            Err(_) => {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
        };
        if verify_target_against_receipt(pack, &approved, receipt).is_err()
            || faults.check(FaultPoint::DuringRollback).is_err()
            || rollback_created_payload(self.root.path(), pack, receipt, &receipt.transaction_id)
                .is_err()
        {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        journal.finish()?;
        Err(original_error)
    }

    fn lifecycle_with<F: NativePayloadFaults>(
        &self,
        install_id: &str,
        pack: &LoadedResourcePack<'_>,
        now_unix: u64,
        kind: InstallLifecycleKind,
        faults: &F,
    ) -> Result<NativePayloadLifecycleCommit, TransactionError> {
        if now_unix > JCS_MAX_SAFE_INTEGER || !valid_identifier(install_id) {
            return Err(TransactionError::InvalidReceipt);
        }
        self.root.validate()?;
        ensure_no_journal(self.root.path(), install_id)?;
        let prior_state =
            load_state(self.root.path(), install_id)?.ok_or(TransactionError::ReceiptMissing)?;
        if let Some(lifecycle) = prior_state.last_lifecycle.as_ref()
            && prior_state.active.is_none()
            && lifecycle.kind == kind
        {
            return Ok(NativePayloadLifecycleCommit {
                disposition: idempotent_lifecycle_disposition(kind),
                receipt: lifecycle.clone(),
                cleanup_required: false,
            });
        }
        let active = prior_state
            .active
            .as_ref()
            .ok_or(TransactionError::ReceiptMissing)?;
        if now_unix < active.applied_at_unix {
            return Err(TransactionError::ObservedStateMismatch);
        }
        verify_active_receipt(self.root.path(), self.root.root_id(), pack, active)?;

        let transaction_id = transaction_id();
        let journal_value = NativePayloadJournalV1 {
            schema_version: NATIVE_PAYLOAD_INSTALL_JOURNAL_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            kind: match kind {
                InstallLifecycleKind::Removed => JournalKind::Remove,
                InstallLifecycleKind::RolledBack => JournalKind::Rollback,
            },
            install_id: install_id.to_string(),
            plan_digest_sha256: Some(active.semantic_digest_sha256.clone()),
            prior_state_sha256: Some(sha256_hex(&prior_state.to_canonical_json()?)),
            target_leaf: active.operation.relative_path.clone(),
            started_at_unix: now_unix,
        };
        let mut journal = NativePayloadJournalGuard::create(self.root.path(), journal_value)?;
        if let Err(error) = faults.check(FaultPoint::AfterJournal) {
            journal.finish()?;
            return Err(error);
        }
        if self.root.validate().is_err() {
            journal.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        let reloaded =
            load_state(self.root.path(), install_id)?.ok_or(TransactionError::ReceiptMissing)?;
        if reloaded != prior_state {
            journal.finish()?;
            return Err(TransactionError::ObservedStateMismatch);
        }
        let active = reloaded
            .active
            .as_ref()
            .ok_or(TransactionError::ReceiptMissing)?;
        verify_active_receipt(self.root.path(), self.root.root_id(), pack, active)?;
        let quarantine =
            match move_verified_to_quarantine(self.root.path(), pack, active, &transaction_id) {
                Ok(quarantine) => quarantine,
                Err(error) => {
                    if error == TransactionError::RecoveryRequired {
                        journal.retain();
                    }
                    return Err(error);
                }
            };

        let lifecycle = NativePayloadLifecycleReceiptV1 {
            schema_version: NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION,
            transaction_id: transaction_id.clone(),
            install_id: install_id.to_string(),
            prior_transaction_id: active.transaction_id.clone(),
            kind,
            completed_at_unix: now_unix,
        };
        let next_state = NativePayloadInstallStateV1 {
            schema_version: NATIVE_PAYLOAD_INSTALL_STATE_SCHEMA_VERSION,
            generation: next_generation(Some(&prior_state))?,
            install_id: install_id.to_string(),
            active: None,
            last_lifecycle: Some(lifecycle.clone()),
        };
        let commit_result = faults
            .check(FaultPoint::BeforeStateCommit)
            .and_then(|()| persist_state(self.root.path(), &next_state, &transaction_id, faults));
        if let Err(error) = commit_result {
            if error == TransactionError::RecoveryRequired {
                journal.retain();
                return Err(error);
            }
            if faults.check(FaultPoint::DuringRollback).is_err()
                || restore_quarantine(self.root.path(), pack, active, &quarantine).is_err()
            {
                journal.retain();
                return Err(TransactionError::RecoveryRequired);
            }
            journal.finish()?;
            return Err(error);
        }

        let cleanup_failed = faults.check(FaultPoint::DuringCleanup).is_err()
            || remove_verified_quarantine(pack, active, &quarantine).is_err();
        if cleanup_failed {
            journal.retain();
            return Ok(NativePayloadLifecycleCommit {
                disposition: lifecycle_disposition(kind),
                receipt: lifecycle,
                cleanup_required: true,
            });
        }
        let cleanup_required = journal.finish().is_err();
        Ok(NativePayloadLifecycleCommit {
            disposition: lifecycle_disposition(kind),
            receipt: lifecycle,
            cleanup_required,
        })
    }
}

#[derive(Clone, Copy)]
enum ApplyKind {
    Fresh,
    Repair,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ExtractedTargetOwnership {
    CreatedByTransaction,
    Uncertain,
}

struct ExecutableNativePayload<'a> {
    plan: &'a VerifiedInstallPlan,
    operation: &'a InstallOperationV1,
    root_id: &'a str,
    relative_path: &'a str,
    release_envelope_sha256: &'a str,
    archive_sha256: &'a str,
    manifest_sha256: &'a str,
    pack_sha256: &'a str,
    artifact_content_root_sha256: &'a str,
    binary_sha256: &'a str,
    ownership: &'a OwnershipMarkerV1,
    release: &'a VerifiedNativeReleaseEnvelope,
    destination: PathBuf,
    plan_digest: &'a str,
}

impl<'a> ExecutableNativePayload<'a> {
    fn from_plan(
        verified_plan: &'a VerifiedInstallPlan,
        approval: &ApprovedInstallPlan,
        pack: &LoadedResourcePack<'_>,
        release: &'a VerifiedNativeReleaseEnvelope,
        root: &ApprovedManagedRoot,
        now_unix: u64,
    ) -> Result<Self, TransactionError> {
        let plan = verified_plan.plan();
        approval.validate_for(verified_plan, now_unix)?;
        if now_unix < release.verified_at_unix()
            || now_unix < release.envelope().not_before_unix
            || now_unix >= release.envelope().expires_at_unix
        {
            return Err(TransactionError::PlanExpired);
        }
        if plan.approvals_required != [ApprovalRequirement::FilesystemWrite]
            || plan.target.profile != CapabilityProfile::Lite
            || plan.target.scope != InstallScope::User
            || plan.allowed_roots.len() != 1
            || plan.operations.len() != 1
            || plan.outstanding_host_action.is_some()
        {
            return Err(TransactionError::UnsupportedPlan);
        }
        let allowed_root = &plan.allowed_roots[0];
        if allowed_root.root != SymbolicRoot::QiongliManagedData
            || allowed_root.id != root.root_id()
        {
            return Err(TransactionError::UnsafeManagedRoot);
        }
        let operation = &plan.operations[0];
        let InstallActionV1::InstallNativePayload {
            root_id,
            entry_key,
            relative_path,
            release_envelope_sha256,
            archive_sha256,
            manifest_sha256,
            pack_sha256,
            artifact_content_root_sha256,
            binary_sha256,
            ownership,
        } = &operation.action
        else {
            return Err(TransactionError::UnsupportedPlan);
        };
        let artifact_id =
            native_artifact_id(&plan.artifact).map_err(|_| TransactionError::UnsupportedPlan)?;
        let expected_install_id = format!("native-payload-{archive_sha256}");
        if root_id != root.root_id()
            || entry_key != ENTRY_KEY
            || relative_path != &artifact_id
            || ownership.install_id != expected_install_id
            || operation.precondition != PlanStateV1::Missing
            || operation.observed_state_sha256
                != observed_plan_state_sha256(&PlanStateV1::Missing)
                    .map_err(|_| TransactionError::UnsupportedPlan)?
        {
            return Err(TransactionError::ObservedStateMismatch);
        }
        if release_envelope_sha256 != release.signed_payload_sha256() {
            return Err(TransactionError::PayloadMismatch);
        }
        let PlanStateV1::Managed {
            ownership: post_ownership,
            content_sha256,
        } = &operation.postcondition
        else {
            return Err(TransactionError::UnsupportedPlan);
        };
        let InstallActionV1::RemoveManagedEntry {
            root_id: inverse_root,
            entry_key: inverse_key,
            expected_ownership,
            expected_sha256,
        } = &operation.inverse
        else {
            return Err(TransactionError::UnsupportedPlan);
        };
        if post_ownership != ownership
            || content_sha256 != artifact_content_root_sha256
            || inverse_root != root_id
            || inverse_key != entry_key
            || expected_ownership != ownership
            || expected_sha256 != artifact_content_root_sha256
        {
            return Err(TransactionError::UnsupportedPlan);
        }
        if verified_plan.grant().signed_grant() != release.launch_grant().signed_grant()
            || verified_plan.grant().signed_payload_sha256()
                != release.launch_grant().signed_payload_sha256()
            || verified_plan.grant().authorized_mode() != release.launch_grant().authorized_mode()
            || verified_plan.grant().authorized_scope() != release.launch_grant().authorized_scope()
        {
            return Err(TransactionError::PayloadMismatch);
        }
        let archive = release.archive_target();
        let verified = verify_native_portable_archive(pack, archive)
            .map_err(|_| TransactionError::PayloadMismatch)?;
        let manifest = verified.payload().manifest();
        if verified.artifact() != &plan.artifact
            || archive.artifact() != &plan.artifact
            || &verified != release.archive()
            || verified.archive_sha256() != archive_sha256
            || verified.manifest_sha256() != manifest_sha256
            || manifest.content.pack_sha256 != *pack_sha256
            || manifest.artifact_content_root_sha256 != *artifact_content_root_sha256
            || manifest.binary_sha256 != *binary_sha256
            || pack.pack_sha256() != pack_sha256
            || verified_plan.grant().grant().resource_pack_sha256 != *pack_sha256
            || verified_plan.grant().grant().binary_sha256 != *binary_sha256
        {
            return Err(TransactionError::PayloadMismatch);
        }
        Ok(Self {
            plan: verified_plan,
            operation,
            root_id,
            relative_path,
            release_envelope_sha256,
            archive_sha256,
            manifest_sha256,
            pack_sha256,
            artifact_content_root_sha256,
            binary_sha256,
            ownership,
            release,
            destination: root.path().join(relative_path),
            plan_digest: &plan.semantic_digest_sha256,
        })
    }

    fn install_id(&self) -> &str {
        &self.ownership.install_id
    }

    fn verify_archive(&self, pack: &LoadedResourcePack<'_>) -> Result<(), TransactionError> {
        let verified = verify_native_portable_archive(pack, self.release.archive_target())
            .map_err(|_| TransactionError::PayloadMismatch)?;
        let manifest = verified.payload().manifest();
        if &verified != self.release.archive()
            || verified.archive_sha256() != self.archive_sha256
            || verified.manifest_sha256() != self.manifest_sha256
            || manifest.content.pack_sha256 != self.pack_sha256
            || manifest.artifact_content_root_sha256 != self.artifact_content_root_sha256
            || manifest.binary_sha256 != self.binary_sha256
        {
            return Err(TransactionError::PayloadMismatch);
        }
        Ok(())
    }

    fn archive_target(&self) -> &NativePortableArchiveTarget {
        self.release.archive_target()
    }

    fn build_receipt(
        &self,
        transaction_id: String,
        applied_at_unix: u64,
        replaces_transaction_id: Option<String>,
    ) -> NativePayloadInstallReceiptV1 {
        NativePayloadInstallReceiptV1 {
            schema_version: NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION,
            transaction_id,
            plan_id: self.plan.plan().plan_id.clone(),
            semantic_digest_sha256: self.plan_digest.to_string(),
            install_id: self.install_id().to_string(),
            artifact: self.plan.plan().artifact.clone(),
            target: self.plan.plan().target.clone(),
            operation: NativePayloadOperationReceiptV1 {
                operation_id: self.operation.operation_id.clone(),
                root_id: self.root_id.to_string(),
                entry_key: ENTRY_KEY.to_string(),
                relative_path: self.relative_path.to_string(),
                ownership: self.ownership.clone(),
                release_envelope_sha256: self.release_envelope_sha256.to_string(),
                archive_sha256: self.archive_sha256.to_string(),
                manifest_sha256: self.manifest_sha256.to_string(),
                pack_sha256: self.pack_sha256.to_string(),
                artifact_content_root_sha256: self.artifact_content_root_sha256.to_string(),
                binary_sha256: self.binary_sha256.to_string(),
            },
            applied_at_unix,
            replaces_transaction_id,
        }
    }

    fn matches_active(&self, active: &NativePayloadInstallReceiptV1) -> bool {
        active.install_id == self.install_id()
            && active.semantic_digest_sha256 == self.plan_digest
            && active.operation.root_id == self.root_id
            && active.operation.relative_path == self.relative_path
            && active.operation.ownership == *self.ownership
            && active.operation.release_envelope_sha256 == self.release_envelope_sha256
            && active.operation.archive_sha256 == self.archive_sha256
            && active.operation.manifest_sha256 == self.manifest_sha256
            && active.operation.pack_sha256 == self.pack_sha256
            && active.operation.artifact_content_root_sha256 == self.artifact_content_root_sha256
            && active.operation.binary_sha256 == self.binary_sha256
    }
}

fn verify_active_receipt(
    root: &Path,
    expected_root_id: &str,
    pack: &LoadedResourcePack<'_>,
    receipt: &NativePayloadInstallReceiptV1,
) -> Result<VerifiedNativeArtifact, TransactionError> {
    receipt.validate()?;
    if receipt.operation.root_id != expected_root_id
        || receipt.operation.pack_sha256 != pack.pack_sha256()
    {
        return Err(TransactionError::ManagedStateDrift);
    }
    let path = root.join(&receipt.operation.relative_path);
    let target = approve_native_artifact_target(path, &receipt.artifact)
        .map_err(|_| TransactionError::ManagedStateDrift)?;
    verify_target_against_receipt(pack, &target, receipt)
}

fn verify_target_against_receipt(
    pack: &LoadedResourcePack<'_>,
    target: &NativeArtifactTarget,
    receipt: &NativePayloadInstallReceiptV1,
) -> Result<VerifiedNativeArtifact, TransactionError> {
    let verified =
        verify_native_artifact(pack, target).map_err(|_| TransactionError::ManagedStateDrift)?;
    verify_artifact_against_receipt(target, receipt, verified)
}

fn verify_artifact_against_receipt(
    target: &NativeArtifactTarget,
    receipt: &NativePayloadInstallReceiptV1,
    verified: VerifiedNativeArtifact,
) -> Result<VerifiedNativeArtifact, TransactionError> {
    let manifest = verified.manifest();
    if target.artifact() != &receipt.artifact
        || target.artifact_id() != receipt.operation.relative_path
        || verified.manifest_sha256() != receipt.operation.manifest_sha256
        || manifest.content.pack_sha256 != receipt.operation.pack_sha256
        || manifest.artifact_content_root_sha256 != receipt.operation.artifact_content_root_sha256
        || manifest.binary_sha256 != receipt.operation.binary_sha256
    {
        return Err(TransactionError::ManagedStateDrift);
    }
    Ok(verified)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum JournalKind {
    Apply,
    Repair,
    Remove,
    Rollback,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NativePayloadJournalV1 {
    schema_version: u32,
    transaction_id: String,
    kind: JournalKind,
    install_id: String,
    plan_digest_sha256: Option<String>,
    prior_state_sha256: Option<String>,
    target_leaf: String,
    started_at_unix: u64,
}

impl NativePayloadJournalV1 {
    fn validate(&self) -> Result<(), TransactionError> {
        if self.schema_version != NATIVE_PAYLOAD_INSTALL_JOURNAL_SCHEMA_VERSION
            || !valid_identifier(&self.transaction_id)
            || !valid_identifier(&self.install_id)
            || !valid_leaf(&self.target_leaf)
            || self.started_at_unix > JCS_MAX_SAFE_INTEGER
            || self
                .plan_digest_sha256
                .as_ref()
                .is_some_and(|digest| !valid_digest(digest))
            || self
                .prior_state_sha256
                .as_ref()
                .is_some_and(|digest| !valid_digest(digest))
        {
            return Err(TransactionError::InvalidReceipt);
        }
        Ok(())
    }
}

struct NativePayloadJournalGuard {
    root: PathBuf,
    path: PathBuf,
    identity: Option<Handle>,
    armed: bool,
}

impl NativePayloadJournalGuard {
    fn create(root: &Path, journal: NativePayloadJournalV1) -> Result<Self, TransactionError> {
        journal.validate()?;
        let path = journal_path(root);
        ensure_destination_absent(&path)?;
        let bytes = canonical_json(&journal)?;
        let mut file = create_private_new_file(&path)?;
        if let Err(error) = write_sync_file(&mut file, &bytes) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        drop(file);
        sync_directory(root)?;
        let identity = Handle::from_path(&path).map_err(|_| TransactionError::RecoveryRequired)?;
        Ok(Self {
            root: root.to_path_buf(),
            path,
            identity: Some(identity),
            armed: true,
        })
    }

    fn finish(&mut self) -> Result<(), TransactionError> {
        if !self.armed {
            return Ok(());
        }
        let expected = self
            .identity
            .as_ref()
            .ok_or(TransactionError::RecoveryRequired)?;
        let current =
            Handle::from_path(&self.path).map_err(|_| TransactionError::RecoveryRequired)?;
        if &current != expected {
            self.retain();
            return Err(TransactionError::RecoveryRequired);
        }
        fs::remove_file(&self.path)
            .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
        sync_directory(&self.root)?;
        self.identity.take();
        self.armed = false;
        Ok(())
    }

    fn retain(&mut self) {
        self.identity.take();
        self.armed = false;
    }
}

impl Drop for NativePayloadJournalGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let still_owned = self.identity.as_ref().is_some_and(|expected| {
            Handle::from_path(&self.path).is_ok_and(|current| &current == expected)
        });
        self.identity.take();
        if still_owned {
            let _ = fs::remove_file(&self.path);
            let _ = sync_directory(&self.root);
        }
    }
}

struct Quarantine {
    container: PathBuf,
    target: NativeArtifactTarget,
    identity: Handle,
}

fn move_verified_to_quarantine(
    root: &Path,
    pack: &LoadedResourcePack<'_>,
    receipt: &NativePayloadInstallReceiptV1,
    transaction_id: &str,
) -> Result<Quarantine, TransactionError> {
    let source_path = root.join(&receipt.operation.relative_path);
    let source = approve_native_artifact_target(&source_path, &receipt.artifact)
        .map_err(|_| TransactionError::ManagedStateDrift)?;
    verify_target_against_receipt(pack, &source, receipt)?;
    let before =
        Handle::from_path(&source_path).map_err(|_| TransactionError::ManagedStateDrift)?;
    let container = root.join(format!("{QUARANTINE_PREFIX}{transaction_id}"));
    ensure_destination_absent(&container)?;
    create_private_directory(&container)?;
    let identity = Handle::from_path(&container).map_err(|_| TransactionError::RecoveryRequired)?;
    let quarantine_path = container.join(&receipt.operation.relative_path);
    if let Err(error) = rename_path(&source_path, &quarantine_path, false) {
        let _ = fs::remove_dir(&container);
        return Err(error);
    }
    sync_directory(root).map_err(|_| TransactionError::RecoveryRequired)?;
    sync_directory(&container).map_err(|_| TransactionError::RecoveryRequired)?;
    let target = approve_native_artifact_target(&quarantine_path, &receipt.artifact)
        .map_err(|_| TransactionError::RecoveryRequired)?;
    let after =
        Handle::from_path(&quarantine_path).map_err(|_| TransactionError::RecoveryRequired)?;
    if before != after || verify_target_against_receipt(pack, &target, receipt).is_err() {
        return Err(TransactionError::RecoveryRequired);
    }
    Ok(Quarantine {
        container,
        target,
        identity,
    })
}

fn restore_quarantine(
    root: &Path,
    pack: &LoadedResourcePack<'_>,
    receipt: &NativePayloadInstallReceiptV1,
    quarantine: &Quarantine,
) -> Result<(), TransactionError> {
    verify_quarantine_container(quarantine)?;
    verify_target_against_receipt(pack, &quarantine.target, receipt)
        .map_err(|_| TransactionError::RollbackConflict)?;
    let before = Handle::from_path(quarantine.target.path())
        .map_err(|_| TransactionError::RollbackConflict)?;
    let destination = root.join(&receipt.operation.relative_path);
    ensure_destination_absent(&destination)?;
    let after = Handle::from_path(quarantine.target.path())
        .map_err(|_| TransactionError::RollbackConflict)?;
    if before != after {
        return Err(TransactionError::RollbackConflict);
    }
    rename_path(quarantine.target.path(), &destination, false)?;
    sync_directory(root)?;
    fs::remove_dir(&quarantine.container)
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    sync_directory(root)?;
    let restored = approve_native_artifact_target(destination, &receipt.artifact)
        .map_err(|_| TransactionError::RollbackConflict)?;
    verify_target_against_receipt(pack, &restored, receipt)
        .map_err(|_| TransactionError::RollbackConflict)?;
    Ok(())
}

fn remove_verified_quarantine(
    pack: &LoadedResourcePack<'_>,
    receipt: &NativePayloadInstallReceiptV1,
    quarantine: &Quarantine,
) -> Result<(), TransactionError> {
    verify_quarantine_container(quarantine)?;
    verify_target_against_receipt(pack, &quarantine.target, receipt)
        .map_err(|_| TransactionError::RollbackConflict)?;
    let before = Handle::from_path(quarantine.target.path())
        .map_err(|_| TransactionError::RollbackConflict)?;
    let after = Handle::from_path(quarantine.target.path())
        .map_err(|_| TransactionError::RollbackConflict)?;
    if before != after {
        return Err(TransactionError::RollbackConflict);
    }
    fs::remove_dir_all(quarantine.target.path())
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    fs::remove_dir(&quarantine.container)
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))?;
    sync_directory(
        quarantine
            .container
            .parent()
            .ok_or(TransactionError::UnsafeManagedRoot)?,
    )
}

fn verify_quarantine_container(quarantine: &Quarantine) -> Result<(), TransactionError> {
    let current =
        Handle::from_path(&quarantine.container).map_err(|_| TransactionError::RollbackConflict)?;
    if current == quarantine.identity {
        Ok(())
    } else {
        Err(TransactionError::RollbackConflict)
    }
}

fn rollback_created_payload(
    root: &Path,
    pack: &LoadedResourcePack<'_>,
    receipt: &NativePayloadInstallReceiptV1,
    transaction_id: &str,
) -> Result<(), TransactionError> {
    let quarantine = move_verified_to_quarantine(root, pack, receipt, transaction_id)?;
    remove_verified_quarantine(pack, receipt, &quarantine)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum FaultPoint {
    AfterJournal,
    AfterExtract,
    BeforeStateCommit,
    DuringStateCommit,
    DuringRollback,
    DuringCleanup,
}

trait NativePayloadFaults {
    fn check(&self, point: FaultPoint) -> Result<(), TransactionError>;
}

struct NoFaults;

impl NativePayloadFaults for NoFaults {
    fn check(&self, _point: FaultPoint) -> Result<(), TransactionError> {
        Ok(())
    }
}

fn persist_state<F: NativePayloadFaults>(
    root: &Path,
    state: &NativePayloadInstallStateV1,
    transaction_id: &str,
    faults: &F,
) -> Result<(), TransactionError> {
    let bytes = state.to_canonical_json()?;
    let destination = state_path(root, &state.install_id);
    if path_exists(&destination)? {
        let _ = read_private_file(&destination)?;
    }
    let staging = root.join(format!(
        "{STATE_FILE_PREFIX}{}.stage-{transaction_id}",
        state.install_id
    ));
    ensure_destination_absent(&staging)?;
    let mut file = create_private_new_file(&staging)?;
    if let Err(error) = write_sync_file(&mut file, &bytes) {
        drop(file);
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    drop(file);
    if let Err(error) = faults.check(FaultPoint::DuringStateCommit) {
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    if rename_path(&staging, &destination, path_exists(&destination)?).is_err() {
        let _ = fs::remove_file(&staging);
        return Err(TransactionError::RecoveryRequired);
    }
    sync_directory(root).map_err(|_| TransactionError::RecoveryRequired)?;
    let committed =
        read_private_file(&destination).map_err(|_| TransactionError::RecoveryRequired)?;
    if committed != bytes {
        return Err(TransactionError::RecoveryRequired);
    }
    Ok(())
}

fn load_state(
    root: &Path,
    install_id: &str,
) -> Result<Option<NativePayloadInstallStateV1>, TransactionError> {
    if !valid_identifier(install_id) {
        return Err(TransactionError::InvalidReceipt);
    }
    let path = state_path(root, install_id);
    if !path_exists(&path)? {
        return Ok(None);
    }
    NativePayloadInstallStateV1::from_json(&read_private_file(&path)?).map(Some)
}

fn state_digest(root: &Path, install_id: &str) -> Result<Option<String>, TransactionError> {
    let path = state_path(root, install_id);
    if !path_exists(&path)? {
        return Ok(None);
    }
    read_private_file(&path).map(|bytes| Some(sha256_hex(&bytes)))
}

fn next_generation(state: Option<&NativePayloadInstallStateV1>) -> Result<u64, TransactionError> {
    state
        .map_or(Some(1), |state| state.generation.checked_add(1))
        .filter(|generation| *generation <= JCS_MAX_SAFE_INTEGER)
        .ok_or(TransactionError::InvalidReceipt)
}

fn ensure_no_journal(root: &Path, install_id: &str) -> Result<(), TransactionError> {
    if !valid_identifier(install_id) {
        return Err(TransactionError::InvalidReceipt);
    }
    if path_exists(&journal_path(root))? {
        Err(TransactionError::RecoveryRequired)
    } else {
        Ok(())
    }
}

fn state_path(root: &Path, install_id: &str) -> PathBuf {
    root.join(format!(
        "{STATE_FILE_PREFIX}{install_id}{STATE_FILE_SUFFIX}"
    ))
}

fn journal_path(root: &Path) -> PathBuf {
    root.join(JOURNAL_FILE_NAME)
}

fn lifecycle_disposition(kind: InstallLifecycleKind) -> LifecycleDisposition {
    match kind {
        InstallLifecycleKind::Removed => LifecycleDisposition::Removed,
        InstallLifecycleKind::RolledBack => LifecycleDisposition::RolledBack,
    }
}

fn idempotent_lifecycle_disposition(kind: InstallLifecycleKind) -> LifecycleDisposition {
    match kind {
        InstallLifecycleKind::Removed => LifecycleDisposition::AlreadyRemoved,
        InstallLifecycleKind::RolledBack => LifecycleDisposition::AlreadyRolledBack,
    }
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, TransactionError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| TransactionError::InvalidReceipt)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_leaf(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && !matches!(value, "." | "..")
        && !value.ends_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), TransactionError> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|error| TransactionError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn create_private_directory(path: &Path) -> Result<(), TransactionError> {
    qiongli_windows_security::create_owner_only_directory(path)
        .map(|_| ())
        .map_err(|error| {
            TransactionError::PersistenceFailed(
                error
                    .io_kind()
                    .unwrap_or(std::io::ErrorKind::PermissionDenied),
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(_path: &Path) -> Result<(), TransactionError> {
    Err(TransactionError::UnsupportedPlatform)
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write as _};
    use std::sync::atomic::{AtomicU64, Ordering};

    use ed25519_dalek::{Signer, SigningKey};
    use qiongli_content::{
        BuiltResourcePack, CompatibleProduct, ResourcePackBuildMetadata, build_resource_pack,
        collect_canonical_sources, load_resource_pack,
    };

    use super::*;
    use crate::{
        GrantMode, GrantSignatureV1, GrantVerificationContext, IntegrationScope, LaunchGrantV1,
        LocalSurface, LocalTargetFamily, NativeReleaseSignatureV1,
        NativeReleaseVerificationContext, ReleaseChannel, SignatureAlgorithm, SignedLaunchGrantV1,
        SignedNativeReleaseEnvelopeV1, TrustedPublicKey, TrustedReleasePublicKey,
        approve_install_plan, approve_managed_root, approve_native_portable_archive_target,
        build_native_release_envelope, compose_native_artifact, compose_native_portable_archive,
        current_target_native_artifact_identity, launch_grant_signing_bytes,
        native_artifact_binary_path, native_portable_archive_file_name,
        native_release_envelope_signing_bytes,
    };

    const NOW: u64 = 1_750_000_000;
    const CANONICAL_DIRECTORIES: [&str; 12] = [
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
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        container: PathBuf,
        source: PathBuf,
        binary: PathBuf,
        managed: PathBuf,
        source_parent: PathBuf,
        archive_parent: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/qiongli-native-payload-install-tests");
            fs::create_dir_all(&base).expect("native install test base must exist");
            let requested = base.join(format!(
                "{name}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&requested).expect("native install container must exist");
            let container =
                fs::canonicalize(requested).expect("native install container must canonicalize");
            let source = container.join("pack-source");
            fs::create_dir(&source).expect("pack source must exist");
            for directory in CANONICAL_DIRECTORIES {
                let directory = source.join(directory);
                fs::create_dir(&directory).expect("canonical source directory must exist");
                fs::write(
                    directory.join("entry.md"),
                    format!("{name}:{}\n", directory.display()),
                )
                .expect("canonical source file must write");
            }
            fs::write(source.join("skills-core.md"), b"core\n").expect("skills core must write");
            fs::write(source.join("skills-summary.md"), b"summary\n")
                .expect("skills summary must write");

            let binary = container.join(format!("source-qiongli{}", std::env::consts::EXE_SUFFIX));
            fs::write(&binary, b"qiongli native payload test binary\n")
                .expect("test binary must write");
            set_test_executable(&binary);
            let managed = container.join("managed");
            let source_parent = container.join("artifact-source");
            let archive_parent = container.join("archive-source");
            create_private_directory(&managed).expect("managed root must be private");
            create_private_directory(&source_parent).expect("artifact parent must be private");
            create_private_directory(&archive_parent).expect("archive parent must be private");
            Self {
                container,
                source,
                binary,
                managed,
                source_parent,
                archive_parent,
            }
        }

        fn pack(&self) -> BuiltResourcePack {
            let resources = collect_canonical_sources(&self.source)
                .expect("native install sources must collect");
            build_resource_pack(
                &ResourcePackBuildMetadata {
                    pack_id: "qiongli-core".to_string(),
                    content_version: "1.19.0-beta.1".to_string(),
                    source_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                    compatible_product: CompatibleProduct {
                        minimum: "2.0.0-alpha.1".to_string(),
                        maximum_exclusive: "3.0.0".to_string(),
                    },
                },
                &resources,
            )
            .expect("native install pack must build")
        }

        fn approved_root(&self) -> ApprovedManagedRoot {
            approve_managed_root(
                &AllowedRootV1 {
                    id: "qiongli-data".to_string(),
                    root: SymbolicRoot::QiongliManagedData,
                },
                &self.managed,
            )
            .expect("managed root must approve")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.container);
        }
    }

    struct Prepared {
        built: BuiltResourcePack,
        archive_target: NativePortableArchiveTarget,
        archive: VerifiedNativePortableArchive,
        artifact_id: String,
        install_id: String,
    }

    fn prepare(fixture: &Fixture) -> Prepared {
        let built = fixture.pack();
        let loaded = load_resource_pack(built.core_bytes(), built.pack_sha256())
            .expect("native install pack must load");
        let artifact =
            current_target_native_artifact_identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .expect("current target artifact must resolve");
        let artifact_id = native_artifact_id(&artifact).unwrap();
        let source_target =
            approve_native_artifact_target(fixture.source_parent.join(&artifact_id), &artifact)
                .expect("source target must approve");
        compose_native_artifact(&loaded, &artifact, &fixture.binary, &source_target)
            .expect("source artifact must compose");
        let archive_target = approve_native_portable_archive_target(
            fixture
                .archive_parent
                .join(native_portable_archive_file_name(&artifact).unwrap()),
            &artifact,
        )
        .expect("archive target must approve");
        let archive = compose_native_portable_archive(&loaded, &source_target, &archive_target)
            .expect("archive must compose");
        let install_id = native_payload_install_id(&archive);
        Prepared {
            built,
            archive_target,
            archive,
            artifact_id,
            install_id,
        }
    }

    fn verified_plan(
        prepared: &Prepared,
    ) -> (
        VerifiedNativeReleaseEnvelope,
        VerifiedInstallPlan,
        ApprovedInstallPlan,
    ) {
        let manifest = prepared.archive.payload().manifest();
        let grant = LaunchGrantV1 {
            schema_version: 1,
            generation: 11,
            artifact: prepared.archive.artifact().clone(),
            binary_sha256: manifest.binary_sha256.clone(),
            resource_pack_sha256: manifest.content.pack_sha256.clone(),
            allowed_modes: vec![GrantMode::LiteMcp],
            integration_scopes: vec![IntegrationScope::CodexLocal],
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        };
        let signing_key = SigningKey::from_bytes(&[42_u8; 32]);
        let signature = signing_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
        let signed = SignedLaunchGrantV1 {
            grant,
            signature: GrantSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "native-payload-test-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        };
        let trusted = TrustedPublicKey::new(
            "native-payload-test-key",
            signing_key.verifying_key().to_bytes(),
        )
        .unwrap();
        let context = GrantVerificationContext {
            now_unix: NOW,
            minimum_generation: 11,
            expected_artifact: prepared.archive.artifact(),
            binary_sha256: &manifest.binary_sha256,
            resource_pack_sha256: &manifest.content.pack_sha256,
            requested_mode: GrantMode::LiteMcp,
            requested_scope: IntegrationScope::CodexLocal,
        };
        let loaded = load_resource_pack(prepared.built.core_bytes(), prepared.built.pack_sha256())
            .expect("native install pack must reload");
        let envelope =
            build_native_release_envelope(17, &prepared.archive, &signed, NOW - 30, NOW + 1_800)
                .expect("native payload release envelope must build");
        let release_signing_key = SigningKey::from_bytes(&[43_u8; 32]);
        let release_signature = release_signing_key.sign(
            &native_release_envelope_signing_bytes(&envelope)
                .expect("native payload release preimage must build"),
        );
        let signed_release = SignedNativeReleaseEnvelopeV1 {
            envelope,
            signature: NativeReleaseSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "native-release-test-key".to_string(),
                value_hex: encode_hex(&release_signature.to_bytes()),
            },
        };
        let trusted_release = TrustedReleasePublicKey::new(
            "native-release-test-key",
            release_signing_key.verifying_key().to_bytes(),
            17,
            Some(18),
        )
        .expect("native release key must be trusted");
        let release_context = NativeReleaseVerificationContext {
            now_unix: NOW,
            minimum_release_generation: 17,
            minimum_launch_grant_generation: 11,
            expected_artifact: prepared.archive.artifact(),
            expected_channel: ReleaseChannel::Alpha,
            requested_mode: GrantMode::LiteMcp,
            requested_scope: IntegrationScope::CodexLocal,
        };
        let release = signed_release
            .verify(
                std::slice::from_ref(&trusted_release),
                std::slice::from_ref(&trusted),
                &release_context,
                &loaded,
                &prepared.archive_target,
            )
            .expect("native payload release must verify");
        let plan = preview_native_payload_install(
            InstallPlanMetadataV1 {
                plan_id: "r3i-native-payload-plan".to_string(),
                created_at_unix: NOW,
                expires_at_unix: NOW + 600,
            },
            &release,
            TargetDescriptorV1 {
                family: LocalTargetFamily::CodexLocal,
                surface: LocalSurface::CliLocal,
                scope: InstallScope::User,
                profile: CapabilityProfile::Lite,
                os: prepared.archive.artifact().os,
                arch: prepared.archive.artifact().arch,
                adapter_version: 1,
            },
            AllowedRootV1 {
                id: "qiongli-data".to_string(),
                root: SymbolicRoot::QiongliManagedData,
            },
        )
        .expect("native payload plan must preview");
        let canonical = plan.to_canonical_json().unwrap();
        let parsed = InstallPlanV1::from_json(&canonical).unwrap();
        let verified = parsed
            .verify(std::slice::from_ref(&trusted), &context)
            .expect("native payload plan must verify");
        let approval =
            approve_install_plan(&verified, &[ApprovalRequirement::FilesystemWrite], NOW)
                .expect("native payload plan must approve");
        (release, verified, approval)
    }

    #[test]
    fn rel_913_apply_verify_replay_and_remove_are_canonical_and_idempotent() {
        let fixture = Fixture::new("apply-remove");
        let prepared = prepare(&fixture);
        let loaded =
            load_resource_pack(prepared.built.core_bytes(), prepared.built.pack_sha256()).unwrap();
        let (release, plan, approval) = verified_plan(&prepared);
        let executor = ManagedNativePayloadExecutor::new(fixture.approved_root());

        let applied = executor
            .apply(&plan, &approval, &loaded, &release, NOW + 1)
            .expect("native payload must apply");
        assert_eq!(applied.disposition, InstallDisposition::Applied);
        assert!(!applied.cleanup_required);
        assert!(fixture.managed.join(&prepared.artifact_id).is_dir());
        assert_eq!(
            executor
                .verify(&prepared.install_id, &loaded)
                .unwrap()
                .receipt,
            applied.receipt
        );
        let state_file = state_path(&fixture.managed, &prepared.install_id);
        let state_bytes = fs::read(&state_file).unwrap();
        assert_eq!(
            NativePayloadInstallStateV1::from_json(&state_bytes)
                .unwrap()
                .to_canonical_json()
                .unwrap(),
            state_bytes
        );
        assert!(
            !String::from_utf8_lossy(&state_bytes)
                .contains(fixture.managed.to_string_lossy().as_ref())
        );

        let replay = executor
            .apply(&plan, &approval, &loaded, &release, NOW + 2)
            .expect("identical apply must replay");
        assert_eq!(replay.disposition, InstallDisposition::AlreadyApplied);
        assert_eq!(replay.receipt, applied.receipt);

        let linked_state = fixture.container.join("linked-state.json");
        fs::hard_link(&state_file, &linked_state).unwrap();
        assert_eq!(
            executor.verify(&prepared.install_id, &loaded),
            Err(TransactionError::InvalidReceipt)
        );
        fs::remove_file(linked_state).unwrap();
        assert!(executor.verify(&prepared.install_id, &loaded).is_ok());

        let removed = executor
            .remove(&prepared.install_id, &loaded, NOW + 3)
            .expect("native payload must remove");
        assert_eq!(removed.disposition, LifecycleDisposition::Removed);
        assert!(!fixture.managed.join(&prepared.artifact_id).exists());
        assert_eq!(
            executor
                .remove(&prepared.install_id, &loaded, NOW + 4)
                .unwrap()
                .disposition,
            LifecycleDisposition::AlreadyRemoved
        );
    }

    #[test]
    fn receipt_owned_predecessor_verification_preserves_pack_bound_authority() {
        let fixture = Fixture::new("predecessor");
        let prepared = prepare(&fixture);
        let loaded =
            load_resource_pack(prepared.built.core_bytes(), prepared.built.pack_sha256()).unwrap();
        let (release, plan, approval) = verified_plan(&prepared);
        let executor = ManagedNativePayloadExecutor::new(fixture.approved_root());
        let applied = executor
            .apply(&plan, &approval, &loaded, &release, NOW + 1)
            .unwrap();

        let successor = Fixture::new("successor");
        let successor = prepare(&successor);
        let next_pack =
            load_resource_pack(successor.built.core_bytes(), successor.built.pack_sha256())
                .unwrap();
        assert_ne!(loaded.pack_sha256(), next_pack.pack_sha256());
        assert_eq!(
            executor.verify(&prepared.install_id, &next_pack),
            Err(TransactionError::ManagedStateDrift)
        );
        assert_eq!(
            executor
                .verify_receipt_owned(&prepared.install_id)
                .unwrap()
                .receipt,
            applied.receipt
        );

        let binary = fixture
            .managed
            .join(&prepared.artifact_id)
            .join(native_artifact_binary_path(&applied.receipt.artifact).unwrap());
        let original = fs::read(&binary).unwrap();
        fs::write(&binary, b"changed predecessor").unwrap();
        assert_eq!(
            executor.verify_receipt_owned(&prepared.install_id),
            Err(TransactionError::ManagedStateDrift)
        );
        assert_eq!(fs::read(&binary).unwrap(), b"changed predecessor");
        fs::write(&binary, original).unwrap();

        let foreign = fixture
            .managed
            .join(&prepared.artifact_id)
            .join("foreign.txt");
        fs::write(&foreign, b"preserve me").unwrap();
        assert!(executor.verify_receipt_owned(&prepared.install_id).is_err());
        assert_eq!(fs::read(&foreign).unwrap(), b"preserve me");
        fs::remove_file(foreign).unwrap();

        let journal = journal_path(&fixture.managed);
        fs::write(&journal, b"unfinished transaction").unwrap();
        assert_eq!(
            executor.verify_receipt_owned(&prepared.install_id),
            Err(TransactionError::RecoveryRequired)
        );
        assert_eq!(fs::read(&journal).unwrap(), b"unfinished transaction");
        fs::remove_file(journal).unwrap();
        #[cfg(unix)]
        {
            let saved = fixture.container.join("saved-binary");
            fs::rename(&binary, &saved).unwrap();
            std::os::unix::fs::symlink(&saved, &binary).unwrap();
            assert!(executor.verify_receipt_owned(&prepared.install_id).is_err());
            fs::remove_file(&binary).unwrap();
            fs::rename(saved, &binary).unwrap();
        }

        let state_file = state_path(&fixture.managed, &prepared.install_id);
        let original = fs::read(&state_file).unwrap();
        let mut state = NativePayloadInstallStateV1::from_json(&original).unwrap();
        state.active.as_mut().unwrap().operation.manifest_sha256 = "a".repeat(64);
        fs::write(&state_file, state.to_canonical_json().unwrap()).unwrap();
        assert!(executor.verify_receipt_owned(&prepared.install_id).is_err());
        fs::write(&state_file, original).unwrap();
        let link = fixture.container.join("linked-receipt");
        fs::hard_link(&state_file, &link).unwrap();
        assert_eq!(
            executor.verify_receipt_owned(&prepared.install_id),
            Err(TransactionError::InvalidReceipt)
        );
        fs::remove_file(link).unwrap();
        assert!(executor.verify_receipt_owned(&prepared.install_id).is_ok());
        executor
            .remove(&prepared.install_id, &loaded, NOW + 2)
            .unwrap();
        assert_eq!(
            executor.verify_receipt_owned(&prepared.install_id),
            Err(TransactionError::ReceiptMissing)
        );
    }

    #[test]
    fn rel_913_repair_requires_absence_and_refuses_present_drift() {
        let fixture = Fixture::new("repair-drift");
        let prepared = prepare(&fixture);
        let loaded =
            load_resource_pack(prepared.built.core_bytes(), prepared.built.pack_sha256()).unwrap();
        let (release, plan, approval) = verified_plan(&prepared);
        let executor = ManagedNativePayloadExecutor::new(fixture.approved_root());
        executor
            .apply(&plan, &approval, &loaded, &release, NOW + 1)
            .unwrap();
        fs::remove_dir_all(fixture.managed.join(&prepared.artifact_id)).unwrap();
        let repaired = executor
            .repair(&plan, &approval, &loaded, &release, NOW + 2)
            .expect("absent managed payload must repair");
        assert_eq!(repaired.disposition, InstallDisposition::Repaired);
        assert!(repaired.receipt.replaces_transaction_id.is_some());

        let binary = fixture
            .managed
            .join(&prepared.artifact_id)
            .join(native_artifact_binary_path(&repaired.receipt.artifact).unwrap());
        fs::OpenOptions::new()
            .append(true)
            .open(&binary)
            .unwrap()
            .write_all(b"drift")
            .unwrap();
        assert_eq!(
            executor.verify(&prepared.install_id, &loaded),
            Err(TransactionError::ManagedStateDrift)
        );
        assert_eq!(
            executor.repair(&plan, &approval, &loaded, &release, NOW + 3),
            Err(TransactionError::ManagedStateDrift)
        );
        assert!(fs::read(binary).unwrap().ends_with(b"drift"));
    }

    #[test]
    fn state_commit_faults_restore_apply_and_lifecycle() {
        let fixture = Fixture::new("rollback-faults");
        let prepared = prepare(&fixture);
        let loaded =
            load_resource_pack(prepared.built.core_bytes(), prepared.built.pack_sha256()).unwrap();
        let (release, plan, approval) = verified_plan(&prepared);
        let executor = ManagedNativePayloadExecutor::new(fixture.approved_root());
        let fault = FaultAt(FaultPoint::BeforeStateCommit);

        assert_eq!(
            executor.apply_with(
                &plan,
                &approval,
                &loaded,
                &release,
                NOW + 1,
                ApplyKind::Fresh,
                &fault,
            ),
            Err(TransactionError::PersistenceFailed(io::ErrorKind::Other))
        );
        assert!(!fixture.managed.join(&prepared.artifact_id).exists());
        assert!(!state_path(&fixture.managed, &prepared.install_id).exists());
        assert!(!journal_path(&fixture.managed).exists());

        executor
            .apply(&plan, &approval, &loaded, &release, NOW + 2)
            .unwrap();
        assert_eq!(
            executor.lifecycle_with(
                &prepared.install_id,
                &loaded,
                NOW + 3,
                InstallLifecycleKind::RolledBack,
                &fault,
            ),
            Err(TransactionError::PersistenceFailed(io::ErrorKind::Other))
        );
        assert!(fixture.managed.join(&prepared.artifact_id).is_dir());
        assert!(executor.verify(&prepared.install_id, &loaded).is_ok());
        assert!(!journal_path(&fixture.managed).exists());

        let rolled_back = executor
            .rollback(&prepared.install_id, &loaded, NOW + 4)
            .unwrap();
        assert_eq!(rolled_back.disposition, LifecycleDisposition::RolledBack);
        assert_eq!(
            executor
                .rollback(&prepared.install_id, &loaded, NOW + 5)
                .unwrap()
                .disposition,
            LifecycleDisposition::AlreadyRolledBack
        );
    }

    #[test]
    fn archive_drift_and_foreign_destination_fail_before_adoption() {
        let fixture = Fixture::new("archive-drift");
        let prepared = prepare(&fixture);
        let loaded =
            load_resource_pack(prepared.built.core_bytes(), prepared.built.pack_sha256()).unwrap();
        let (release, plan, approval) = verified_plan(&prepared);
        let executor = ManagedNativePayloadExecutor::new(fixture.approved_root());
        let original_archive = fs::read(prepared.archive_target.path()).unwrap();
        fs::OpenOptions::new()
            .append(true)
            .open(prepared.archive_target.path())
            .unwrap()
            .write_all(b"drift")
            .unwrap();
        assert_eq!(
            executor.apply(&plan, &approval, &loaded, &release, NOW + 1),
            Err(TransactionError::PayloadMismatch)
        );
        assert!(!fixture.managed.join(&prepared.artifact_id).exists());

        fs::write(prepared.archive_target.path(), original_archive).unwrap();

        fs::create_dir(fixture.managed.join(&prepared.artifact_id)).unwrap();
        fs::write(
            fixture
                .managed
                .join(&prepared.artifact_id)
                .join("foreign.txt"),
            b"caller data\n",
        )
        .unwrap();
        assert_eq!(
            executor.apply(&plan, &approval, &loaded, &release, NOW + 2),
            Err(TransactionError::DestinationConflict)
        );
        assert_eq!(
            fs::read(
                fixture
                    .managed
                    .join(&prepared.artifact_id)
                    .join("foreign.txt")
            )
            .unwrap(),
            b"caller data\n"
        );
    }

    struct FaultAt(FaultPoint);

    impl NativePayloadFaults for FaultAt {
        fn check(&self, point: FaultPoint) -> Result<(), TransactionError> {
            if self.0 == point {
                Err(TransactionError::PersistenceFailed(io::ErrorKind::Other))
            } else {
                Ok(())
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
    fn set_test_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    #[cfg(not(unix))]
    fn set_test_executable(_path: &Path) {}
}
