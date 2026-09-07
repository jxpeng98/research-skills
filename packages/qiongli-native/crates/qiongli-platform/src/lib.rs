mod activation;
mod candidate_install;
mod candidate_source;
mod claude;
mod claude_bundle;
mod client_inventory;
mod codex;
mod codex_bundle;
mod community_alpha;
mod community_alpha_integrity;
mod desktop_package;
mod distribution;
mod error;
mod grant;
mod identity;
mod legacy_migration;
mod native_archive;
mod native_artifact;
mod native_install;
mod native_release;
mod native_update;
mod plan;
mod product_control;
mod release_authority;
mod release_candidate;
mod transaction;
mod zotero_companion;
mod zotero_companion_stage;

pub use activation::{
    CLIENT_ACTIVATION_SCHEMA_VERSION, ClientActivationCommit, ClientActivationCoordinator,
    ClientActivationDiscoveryV1, ClientActivationDisposition, ClientActivationEffect,
    ClientActivationError, ClientActivationHandle, ClientActivationLifecycleCommit,
    ClientActivationLifecycleDisposition, ClientActivationPreview, ClientActivationState,
    ClientActivationTarget, ClientActivationVerification, discover_client_activation,
    preview_client_activation,
};
pub use candidate_install::{
    NATIVE_CANDIDATE_MANAGED_ROOT_SYMBOLIC_PATH, NativeCandidateLocalInstallCommit,
    NativeCandidateLocalInstallError, NativeCandidateLocalRemoveCommit,
    NativeCandidateLocalVerification, NativeCandidateRegistrationCommit,
    NativeCandidateRegistrationLifecycleCommit, NativeCandidateRegistrationVerification,
    apply_native_release_candidate_local, discover_native_candidate_managed_root,
    prepare_native_candidate_managed_root, remove_native_release_candidate_local,
    verify_installed_native_candidate_product, verify_native_release_candidate_local,
};
pub use candidate_source::{
    NativeCandidatePluginSourceCommit, NativeCandidatePluginSourceDisposition,
    NativeCandidatePluginSourceError, NativeCandidatePluginSourceTarget,
    NativeCandidatePluginSourceVerification, discover_native_candidate_plugin_source_target,
    materialize_native_candidate_plugin_source, materialize_packaged_product_plugin_source,
    materialize_packaged_product_plugin_source_with_overrides,
    prepare_native_candidate_plugin_source_target, remove_native_candidate_plugin_source,
    verify_native_candidate_plugin_source,
};
pub use claude::{
    CLAUDE_ADAPTER_SCHEMA_VERSION, CLAUDE_MARKETPLACE_SYMBOLIC_PATH,
    CLAUDE_PLUGIN_SOURCE_MARKETPLACE_PATH, CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH,
    CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION, CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
    CLAUDE_SKILLS_PLUGIN_SYMBOLIC_PATH, ClaudeAdapterError, ClaudeDiscoverySummaryV1,
    ClaudeMarketplaceState, ClaudeRegistrationCommit, ClaudeRegistrationDisposition,
    ClaudeRegistrationEffect, ClaudeRegistrationExecutor, ClaudeRegistrationLifecycleCommit,
    ClaudeRegistrationLifecycleDisposition, ClaudeRegistrationLifecycleKind,
    ClaudeRegistrationLifecycleReceiptV1, ClaudeRegistrationPreview, ClaudeRegistrationReceiptV1,
    ClaudeRegistrationState, ClaudeRegistrationStateV1, ClaudeRegistrationVerification,
    ClaudeSkillsPluginState, ClaudeSourceState, ClaudeUserTarget, discover_claude_user,
    discover_claude_user_with_config, preview_claude_registration,
};
pub use claude_bundle::{
    CLAUDE_PLUGIN_BUNDLE_RECEIPT_FILE, CLAUDE_PLUGIN_BUNDLE_RECEIPT_SCHEMA_VERSION,
    ClaudePluginBundleEntryV1, ClaudePluginBundleError, ClaudePluginBundleKind,
    ClaudePluginBundleReceiptV1, ClaudePluginBundleTarget, VerifiedClaudePluginBundle,
    approve_claude_plugin_bundle_target, compose_claude_plugin_bundle,
    compose_claude_plugin_bundle_with_overrides, remove_claude_plugin_bundle,
    replace_claude_plugin_bundle_with_overrides, verify_claude_plugin_bundle,
};
pub use client_inventory::{
    CLIENT_INVENTORY_SCHEMA_VERSION, ClientActionReadiness, ClientComponentInventoryV1,
    ClientComponentState, ClientDiscoveryState, ClientHostPresence, ClientInventory,
    ClientInventoryEntryV1, ClientInventoryInput, ClientInventorySummaryV1, ClientKind,
    ClientOwnershipState, ClientPathCandidateV1, ClientPathId, ClientPathManagement,
    ClientPathScope, ClientPathSource, ClientPathState, ClientPathSurface, ClientSymbolicPath,
    discover_client_inventory,
};
pub use codex::{
    CODEX_ADAPTER_SCHEMA_VERSION, CODEX_MARKETPLACE_SYMBOLIC_PATH,
    CODEX_PLUGIN_SOURCE_MARKETPLACE_PATH, CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH,
    CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION, CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
    CodexAdapterError, CodexDiscoverySummaryV1, CodexMarketplaceState, CodexRegistrationCommit,
    CodexRegistrationDisposition, CodexRegistrationEffect, CodexRegistrationExecutor,
    CodexRegistrationLifecycleCommit, CodexRegistrationLifecycleDisposition,
    CodexRegistrationLifecycleKind, CodexRegistrationLifecycleReceiptV1, CodexRegistrationPreview,
    CodexRegistrationReceiptV1, CodexRegistrationState, CodexRegistrationStateV1,
    CodexRegistrationVerification, CodexSourceState, CodexUserTarget, discover_codex_user,
    preview_codex_registration,
};
pub use codex_bundle::{
    CODEX_PLUGIN_BUNDLE_RECEIPT_FILE, CODEX_PLUGIN_BUNDLE_RECEIPT_SCHEMA_VERSION,
    CodexPluginBundleEntryV1, CodexPluginBundleError, CodexPluginBundleKind,
    CodexPluginBundleReceiptV1, CodexPluginBundleTarget, VerifiedCodexPluginBundle,
    approve_codex_plugin_bundle_target, compose_codex_plugin_bundle,
    compose_codex_plugin_bundle_with_overrides, remove_codex_plugin_bundle,
    replace_codex_plugin_bundle_with_overrides, verify_codex_plugin_bundle,
};
pub use community_alpha::{
    MAX_NATIVE_COMMUNITY_ALPHA_CANDIDATE_SET_BYTES, MAX_NATIVE_COMMUNITY_ALPHA_PROMOTION_BYTES,
    NATIVE_COMMUNITY_ALPHA_CANDIDATE_SET_SCHEMA_VERSION,
    NATIVE_COMMUNITY_ALPHA_PROMOTION_SCHEMA_VERSION, NativeCommunityAlphaAssetRole,
    NativeCommunityAlphaAssetV1, NativeCommunityAlphaBuildProvenance,
    NativeCommunityAlphaCandidateSetContentV1, NativeCommunityAlphaCandidateSetV1,
    NativeCommunityAlphaCandidateStatus, NativeCommunityAlphaEvidenceRole,
    NativeCommunityAlphaEvidenceV1, NativeCommunityAlphaPromotionError,
    NativeCommunityAlphaTargetPromotionV1,
};
pub use community_alpha_integrity::{
    MAX_NATIVE_COMMUNITY_ALPHA_INTEGRITY_BYTES,
    MAX_NATIVE_COMMUNITY_ALPHA_PUBLICATION_RECEIPT_BYTES,
    NATIVE_COMMUNITY_ALPHA_INTEGRITY_SCHEMA_VERSION,
    NATIVE_COMMUNITY_ALPHA_PUBLICATION_RECEIPT_SCHEMA_VERSION, NativeCommunityAlphaIntegrityError,
    NativeCommunityAlphaIntegrityManifestV1, NativeCommunityAlphaPublicationReceiptV1,
    SignedNativeCommunityAlphaIntegrityV1, VerifiedNativeCommunityAlphaIntegrity,
    native_community_alpha_integrity_signing_bytes,
};
pub use desktop_package::{
    DESKTOP_PACKAGE_MANIFEST_FILE, DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION,
    DesktopApplicationMetadataV1, DesktopPackageBinaries, DesktopPackageEntryV1,
    DesktopPackageError, DesktopPackageInput, DesktopPackageKind, DesktopPackageManifestV1,
    DesktopPackageRecordType, DesktopPackageStatus, DesktopZoteroCompanionBindingV1,
    VerifiedDesktopPackage, attach_product_control_to_desktop_manifest, compose_desktop_package,
    desktop_package_file_name, parse_desktop_package_manifest, verify_desktop_package,
};
pub use distribution::{
    MAX_NATIVE_DISTRIBUTION_POLICY_BYTES, MAX_NATIVE_DISTRIBUTION_RELEASE_SET_BYTES,
    MAX_NATIVE_PUBLICATION_AUTHORIZATION_BYTES, NATIVE_DISTRIBUTION_POLICY_SCHEMA_VERSION,
    NATIVE_DISTRIBUTION_RELEASE_SET_SCHEMA_VERSION,
    NATIVE_PUBLICATION_AUTHORIZATION_SCHEMA_VERSION, NativeDistributionClass,
    NativeDistributionError, NativeDistributionPolicyV1, NativeDistributionReleaseLabel,
    NativeDistributionReleaseSetV1, NativeDistributionWarning, NativePlatformTrust,
    NativePublicationAuthorizationAuthority, NativePublicationAuthorizationContext,
    NativePublicationAuthorizationDecision, NativePublicationAuthorizationRequirement,
    NativePublicationAuthorizationV1, VerifiedNativeDistributionReleaseSet,
    VerifiedNativePublicationAuthorization, verify_native_distribution_release_set_authorization,
    verify_native_publication_authorization,
};
pub use error::PlatformError;
pub use grant::{
    GrantMode, GrantSignatureV1, GrantVerificationContext, IntegrationScope, LaunchGrantV1,
    SignatureAlgorithm, SignedLaunchGrantV1, TrustedPublicKey, VerifiedLaunchGrant,
    launch_grant_signing_bytes,
};
pub use identity::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind, OperatingSystem, ProductId,
    ReleaseChannel,
};
pub use legacy_migration::{
    ApprovedLegacyMigrationPlan, LEGACY_MIGRATION_INVENTORY_SCHEMA_VERSION,
    LEGACY_MIGRATION_PLAN_SCHEMA_VERSION, LEGACY_MIGRATION_RECEIPT_SCHEMA_VERSION,
    LegacyMigrationAction, LegacyMigrationApproval, LegacyMigrationClassification,
    LegacyMigrationCleanupCommit, LegacyMigrationCleanupError, LegacyMigrationCleanupFinalization,
    LegacyMigrationCleanupPreview, LegacyMigrationCleanupRecovery, LegacyMigrationContractError,
    LegacyMigrationCutoverError, LegacyMigrationError, LegacyMigrationInventory,
    LegacyMigrationInventoryV1, LegacyMigrationItemId, LegacyMigrationItemState,
    LegacyMigrationItemV1, LegacyMigrationOwnershipEvidence, LegacyMigrationPersistenceError,
    LegacyMigrationPlanInput, LegacyMigrationPlanItemV1, LegacyMigrationPlanV1,
    LegacyMigrationReadiness, LegacyMigrationReceiptItemState, LegacyMigrationReceiptItemV1,
    LegacyMigrationReceiptV1, LegacyMigrationState, LegacyMigrationStore,
    PreparedLegacyMigrationCleanup, VerifiedLegacyMigrationCutover,
    advance_legacy_migration_receipt, apply_legacy_migration_cleanup,
    approve_legacy_migration_plan, discover_legacy_migration,
    discover_legacy_migration_with_config, finalize_legacy_migration_cleanup,
    grant_legacy_migration_approval, initial_legacy_migration_receipt,
    initial_legacy_migration_receipt_from_plan, prepare_legacy_migration_cleanup,
    preview_legacy_migration, recover_legacy_migration_cleanup, resume_legacy_migration_plan,
    verify_legacy_migration_cutover,
};
pub use native_archive::{
    NATIVE_PORTABLE_ARCHIVE_EXTENSION, NativePortableArchiveError, NativePortableArchiveTarget,
    VerifiedNativePortableArchive, approve_native_portable_archive_target,
    compose_native_portable_archive, extract_native_portable_archive,
    native_portable_archive_file_name, verify_native_portable_archive,
};
pub use native_artifact::{
    NATIVE_ARTIFACT_MANIFEST_FILE, NATIVE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
    NativeArtifactContentV1, NativeArtifactEntryV1, NativeArtifactError, NativeArtifactManifestV1,
    NativeArtifactRecordType, NativeArtifactStatus, NativeArtifactTarget, VerifiedNativeArtifact,
    approve_native_artifact_target, compose_native_artifact,
    current_target_native_artifact_identity, native_artifact_binary_path, native_artifact_id,
    verify_native_artifact,
};
pub use native_install::{
    ManagedNativePayloadExecutor, NATIVE_PAYLOAD_INSTALL_JOURNAL_SCHEMA_VERSION,
    NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION, NATIVE_PAYLOAD_INSTALL_STATE_SCHEMA_VERSION,
    NativePayloadInstallCommit, NativePayloadInstallReceiptV1, NativePayloadInstallStateV1,
    NativePayloadInstallVerification, NativePayloadLifecycleCommit,
    NativePayloadLifecycleReceiptV1, NativePayloadOperationReceiptV1, native_payload_install_id,
    preview_native_payload_install,
};
pub use native_release::{
    MAX_NATIVE_RELEASE_ENVELOPE_BYTES, NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION,
    NativeReleaseEnvelopeV1, NativeReleaseError, NativeReleaseSignatureV1,
    NativeReleaseVerificationContext, SignedNativeReleaseEnvelopeV1, TrustedReleasePublicKey,
    VerifiedNativeReleaseEnvelope, build_native_release_envelope,
    native_release_envelope_signing_bytes,
};
pub use native_update::{
    MAX_NATIVE_UPDATE_MANIFEST_BYTES, NATIVE_UPDATE_MANIFEST_SCHEMA_VERSION,
    NativeUpdateDisposition, NativeUpdateError, NativeUpdateEvidenceError, NativeUpdateManifestV1,
    NativeUpdateStream, NativeUpdateVerificationContext, SignedNativeUpdateManifestV1,
    VerifiedNativeUpdateEvidence, VerifiedNativeUpdateManifest,
    native_update_manifest_signing_bytes,
};
pub use plan::{
    AllowedRootV1, ApprovalRequirement, HostAction, InstallActionV1, InstallOperationV1,
    InstallPlanDraftV1, InstallPlanMetadataV1, InstallPlanV1, InstallScope, LocalSurface,
    LocalTargetFamily, OwnershipMarkerV1, PlanStateV1, SymbolicRoot, TargetDescriptorV1,
    VerifiedInstallPlan, observed_plan_state_sha256,
};
pub use product_control::{
    PACKAGED_PRODUCT_CONTROL_FILE, PACKAGED_PRODUCT_CONTROL_SCHEMA_VERSION,
    PackagedProductActivationExpectation, PackagedProductBatchInstallCommit,
    PackagedProductBatchInstallPreview, PackagedProductControlError, PackagedProductControlV1,
    PackagedProductDesiredStateV1, PackagedProductInstallCapability, PackagedProductInstallCommit,
    PackagedProductInstallDisposition, PackagedProductInstallEffect, PackagedProductInstallPreview,
    PackagedProductInstallVerification, PackagedProductPluginIdentity, PackagedProductRecordType,
    PackagedProductSkillsScope, PackagedProductVerificationInput, VerifiedPackagedProduct,
    apply_packaged_product_batch_install, apply_packaged_product_batch_install_with_overrides,
    apply_packaged_product_install, apply_packaged_product_install_with_overrides,
    packaged_product_control_path, preview_packaged_product_batch_install,
    preview_packaged_product_batch_install_with_variant, preview_packaged_product_install,
    preview_packaged_product_install_with_variant, remove_packaged_product_install,
    verify_native_packaged_product, verify_packaged_product, verify_packaged_product_install,
    verify_packaged_product_install_with_variant, verify_receipt_owned_packaged_product_install,
};
pub use release_authority::{
    MAX_NATIVE_RELEASE_AUTHORITY_BYTES, NATIVE_RELEASE_AUTHORITY_SCHEMA_VERSION,
    NativeReleaseAuthority, NativeReleaseAuthorityError,
};
pub use release_candidate::{
    MAX_NATIVE_RELEASE_CANDIDATE_BYTES, MAX_NATIVE_RELEASE_NOTES_BYTES,
    NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION, NativeClientPluginGrantV1,
    NativeReleaseCandidateError, NativeReleaseCandidateStatus, NativeReleaseCandidateV1,
    NativeReleaseCandidateVerificationContext, NativeReleaseNotesV1,
    SignedNativeReleaseCandidateV1, VerifiedInstalledNativeCandidate,
    VerifiedNativeReleaseCandidate, build_native_release_candidate,
    native_release_candidate_file_name, native_release_candidate_signing_bytes,
    native_release_notes_file_name,
};
pub use transaction::{
    ApprovedInstallPlan, ApprovedManagedRoot, INSTALL_JOURNAL_SCHEMA_VERSION,
    INSTALL_RECEIPT_SCHEMA_VERSION, InstallCommit, InstallDisposition, InstallLifecycleKind,
    InstallLifecycleReceiptV1, InstallReceiptV1, InstallVerification, LifecycleCommit,
    LifecycleDisposition, MANAGED_INSTALL_STATE_SCHEMA_VERSION, ManagedInstallStateV1,
    ManagedOperationReceiptV1, ManagedResourceExecutor, TransactionError, approve_install_plan,
    approve_managed_root,
};
pub use zotero_companion::{
    VerifiedZoteroCompanionArtifact, ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE,
    ZOTERO_COMPANION_ARTIFACT_MANIFEST_SCHEMA_VERSION, ZOTERO_COMPANION_DISPLAY_NAME,
    ZOTERO_COMPANION_ENDPOINT_VERSION, ZOTERO_COMPANION_ID, ZOTERO_COMPANION_PACKAGED_XPI_FILE,
    ZOTERO_COMPANION_SOURCE_PATHS, ZOTERO_COMPANION_UPDATE_URL,
    ZOTERO_COMPANION_ZOTERO_MAX_VERSION, ZOTERO_COMPANION_ZOTERO_MIN_VERSION,
    ZoteroCompanionArtifactEntryV1, ZoteroCompanionArtifactError,
    ZoteroCompanionArtifactManifestV1, ZoteroCompanionArtifactRecordType,
    ZoteroCompanionArtifactStatus, ZoteroCompanionSourceEntry, compose_zotero_companion_artifact,
    verify_zotero_companion_artifact,
};
pub use zotero_companion_stage::{
    VerifiedZoteroCompanionStage, ZOTERO_COMPANION_STAGE_RECEIPT_FILE,
    ZOTERO_COMPANION_STAGE_RECEIPT_SCHEMA_VERSION, ZoteroCompanionStageEffect,
    ZoteroCompanionStageError, ZoteroCompanionStagePlan, ZoteroCompanionStageReceiptV1,
    ZoteroCompanionStageRecordType, ZoteroCompanionStageStatus, apply_zotero_companion_stage,
    preview_zotero_companion_stage, verify_zotero_companion_stage,
};

pub const ARTIFACT_IDENTITY_SCHEMA_VERSION: u32 = 1;
pub const LAUNCH_GRANT_SCHEMA_VERSION: u32 = 1;
pub const INSTALL_PLAN_SCHEMA_VERSION: u32 = 1;
