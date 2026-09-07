#![cfg_attr(
    any(test, not(feature = "desktop")),
    allow(
        dead_code,
        reason = "the Tauri IPC adapter is excluded from CLI-only builds and core library unit tests"
    )
)]

use std::collections::BTreeMap;

use qiongli_platform::{
    CLAUDE_MARKETPLACE_SYMBOLIC_PATH, CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH,
    CODEX_MARKETPLACE_SYMBOLIC_PATH, CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH,
};
use qiongli_project::{
    ACADEMIC_CONSOLIDATION_SCHEMA_VERSION, ARTIFACT_CHANGE_SCHEMA_VERSION, AcademicGraphConfidence,
    AcademicGraphDiagnosticV1, AcademicGraphEdgeStatus, AcademicGraphEdgeV1,
    AcademicGraphEntityKind, AcademicGraphIdentityScope, AcademicGraphLayer, AcademicGraphNodeType,
    AcademicGraphNodeV1, AcademicGraphPathQueryV1, AcademicGraphPathResultV1,
    AcademicGraphPathStatus, AcademicGraphPathStepV1, AcademicGraphPathTraversal,
    AcademicGraphPortfolioSnapshotV1, AcademicGraphQueryResultV1, AcademicGraphQueryV1,
    AcademicGraphReadinessV1, AcademicGraphRelation, AcademicGraphRevisionComparisonV1,
    AcademicGraphSnapshotV1, AcademicGraphSourceKind, AcademicGraphSourceRefV1,
    AcademicInferenceStrength, ArtifactChangeSnapshotV1, ArtifactChangeState,
    CAPTURE_COVERAGE_SCHEMA_VERSION, CAPTURE_INBOX_SCHEMA_VERSION, CAPTURE_INTAKE_SCHEMA_VERSION,
    CaptureArea, CaptureAssignmentBindingEffect, CaptureAssignmentDecision,
    CaptureAssignmentIntentId, CaptureAssignmentOutcome, CaptureAssignmentPreviewOutcome,
    CaptureAssignmentPreviewV1, CaptureAssignmentReceiptId, CaptureAssignmentStatusState,
    CaptureAssignmentStatusV1, CaptureConsolidationOutcome, CaptureConsolidationPreviewV1,
    CaptureCoverageDelivery, CaptureCoverageSnapshotV1, CaptureCoverageState, CaptureDelivery,
    CaptureDeliveryAcknowledgementPreviewV1, CaptureDeliveryReason, CaptureDeliveryRetryCause,
    CaptureDeliveryState, CaptureDeliveryStatusV1, CaptureDisposition, CaptureId,
    CaptureInboxSnapshotV1, CaptureIntakeEffect, CaptureIntakePreviewV1, CapturePolicy,
    CaptureResolutionCounterpartState, CaptureResolutionDisposition,
    CaptureResolutionItemContentV1, CaptureResolutionItemId, CaptureResolutionItemKind,
    CaptureResolutionPreviewV1, CaptureResolutionReceiptId, CaptureResolutionReceiptV1,
    CaptureResolutionSelectionV1, CaptureSource, CaptureSourceCoverageV1, ContradictionV1,
    DecisionCandidateV1, DecisionRelation, DeliveryEnvelopeId, EvidenceLocatorKind,
    EvidenceReferenceV1, IncrementalPortfolioSnapshotV1, PortableProjectOperation,
    PortableProjectPreviewV1, PortfolioDerivedStateDeletionV1, PortfolioDoctorStatus,
    PortfolioDoctorV1, PortfolioEvidenceSignal, PortfolioLineageKind,
    PortfolioMaintenanceOperation, PortfolioMaintenancePreviewV1, PortfolioQueryCursorV1,
    PortfolioQueryFiltersV1, PortfolioQueryLimitsV1, PortfolioQueryResultV1, PortfolioQueryV1,
    PortfolioReconciliationV1, PortfolioSharedIdentityFilterV1, ProjectArtifactFormat,
    ProjectArtifactViewV1, ProjectBindingV1, ProjectHealth, ProjectId, ProjectKind,
    ProjectLifecycle, ProjectMigrationPreviewV1, ProjectMigrationReconciliationV1,
    ProjectMigrationRecoveryPreviewV1, ProjectMigrationRollbackPreviewV1, ProjectMutationEffect,
    ProjectMutationKind, ProjectMutationPreviewV1, ProjectStage, RegisteredArtifact,
    RegisteredArtifactObservationV1, ResearchCaptureDraftV1, ResearchCaptureV1,
    ResearchLibrarySnapshotV1, SemanticActivityKind, SemanticActivityTimestampSource,
    SemanticChangeV1, SemanticTimelineCursorV1, SemanticTimelineQueryV1, SemanticTimelineResultV1,
    SemanticTimelineView,
};
use qiongli_ui::{
    AgentBackendSecretChange, DesktopEvent, DesktopIntent, DesktopSnapshotV1, IntegrationPathView,
    IntegrationSelection, IntegrationTarget, IntegrationView, ManagedSkillsStateView,
    ManagedSkillsView, McpSelfTestCheckId, McpSelfTestState, McpSelfTestView, OperationApproval,
    OperationKind, OperationPreview, OperationToken, PrivateText, ProductTrustView, ProfileKind,
    ProviderConfigurationField, ProviderKind, ProviderReadinessView, ProviderSecretChange,
    ProviderSettingsPatch, PublicSettingChange, SkillsDestinationPreset, StatusCode,
    UpdatePhaseView, UpdateStreamView, UpdateView,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::desktop::{
    HostPluginMode, full_public_tool_count, host_plugin_command_templates,
    host_plugin_executable_name, host_plugin_scope,
};
use crate::orchestration_control::{OrchestrationRunListViewV1, OrchestrationRunSummaryV1};

pub(crate) const APP_API_SCHEMA_VERSION: u32 = 19;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSnapshotV1 {
    schema_version: u32,
    product: AppProductView,
    content: AppContentView,
    mcp: AppMcpView,
    cli: AppCliView,
    zotero: AppZoteroIntegrationView,
    configuration: AppConfigurationView,
    update: AppUpdateView,
    research_library: ResearchLibrarySnapshotV1,
    legacy_migration: AppLegacyMigrationStatusView,
    integrations: Vec<AppIntegrationView>,
    capabilities: AppCapabilityView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppContentPreviewResourceV1 {
    pub(crate) path: String,
    pub(crate) format: &'static str,
    pub(crate) editable: bool,
    pub(crate) canonical_sha256: String,
    pub(crate) current_sha256: String,
    pub(crate) overridden: bool,
    pub(crate) content: String,
}

impl AppContentPreviewResourceV1 {
    pub(crate) fn from_bytes(
        path: &str,
        format: &'static str,
        editable: bool,
        canonical_sha256: &str,
        current_sha256: &str,
        bytes: &[u8],
    ) -> Result<Self, &'static str> {
        if bytes.len() > 128 * 1024 {
            return Err("content-preview-too-large");
        }
        Ok(Self {
            path: path.to_owned(),
            format,
            editable,
            canonical_sha256: canonical_sha256.to_owned(),
            current_sha256: current_sha256.to_owned(),
            overridden: canonical_sha256 != current_sha256,
            content: std::str::from_utf8(bytes)
                .map_err(|_| "content-preview-not-utf8")?
                .to_owned(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppProjectGuidanceV1 {
    pub(crate) project_id: ProjectId,
    pub(crate) symbolic_path: &'static str,
    pub(crate) content_sha256: Option<String>,
    pub(crate) content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppContentCustomizationV1 {
    pub(crate) profile: &'static str,
    pub(crate) state: &'static str,
    pub(crate) revision: u64,
    pub(crate) variant_sha256: Option<String>,
    pub(crate) cleanup_required: bool,
    pub(crate) resources: Vec<AppContentPreviewResourceV1>,
    pub(crate) guidance: Option<AppProjectGuidanceV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppProductView {
    version: String,
    build: String,
    operating_system: String,
    architecture: String,
    trust: AppTrustView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppTrustView {
    mode: &'static str,
    label: &'static str,
    can_apply: bool,
    reason_code: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppContentView {
    status: &'static str,
    pack_id: String,
    content_version: String,
    entry_count: usize,
    profiles: Vec<AppProfileView>,
    managed_skills: AppManagedSkillsInventoryView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppManagedSkillsInventoryView {
    status: &'static str,
    destinations: Vec<AppManagedSkillsDestinationView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppManagedSkillsDestinationView {
    target_id: String,
    preset: &'static str,
    symbolic_path: &'static str,
    state: &'static str,
    status: &'static str,
    profile: Option<&'static str>,
    product_version: Option<String>,
    project_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppProjectSkillsTargetView {
    pub(crate) project_id: String,
    pub(crate) destination: ManagedSkillsView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppProfileView {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    included_resource_kinds: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppMcpView {
    status: &'static str,
    profile: &'static str,
    public_tool_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppMcpSelfTestView {
    profile: &'static str,
    product_version: &'static str,
    state: &'static str,
    checks: [AppMcpSelfTestCheckView; 6],
    public_tool_count: usize,
    enabled_provider_count: usize,
    ready_provider_count: usize,
    discovered_client_count: usize,
    registered_client_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppMcpSelfTestCheckView {
    check: &'static str,
    status: &'static str,
    code: &'static str,
    remediation: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppCliView {
    status: &'static str,
    state: &'static str,
    installed_version: Option<String>,
    available_version: String,
    symbolic_target: &'static str,
    path_status: &'static str,
    path_state: &'static str,
    reason_code: &'static str,
    can_install: bool,
    can_test: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppZoteroIntegrationView {
    status: &'static str,
    state: &'static str,
    observation: &'static str,
    zotero_version: Option<String>,
    connector_available: bool,
    companion_available: bool,
    companion_version: Option<String>,
    available_companion_version: Option<String>,
    available_companion_sha256: Option<String>,
    available_companion_size_bytes: Option<u64>,
    endpoint_version: Option<String>,
    supported_endpoint_version: &'static str,
    supported_zotero_min_version: &'static str,
    supported_zotero_max_version: &'static str,
    installation_prepared: bool,
    fallback_import_available: bool,
    fallback_formats: [&'static str; 4],
    reason_code: &'static str,
    can_prepare_install: bool,
    can_reveal: bool,
    can_open_zotero: bool,
    can_verify: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppConfigurationView {
    status: &'static str,
    revision: Option<u64>,
    secret_store: &'static str,
    providers: Vec<AppProviderView>,
    legacy_credential: AppLegacyCredentialView,
    cleanup_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppProviderView {
    provider: &'static str,
    enabled: bool,
    readiness: &'static str,
    configuration_fields: Vec<AppProviderConfigurationFieldView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppProviderConfigurationFieldView {
    field: &'static str,
    configured: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppLegacyCredentialView {
    reference_present: bool,
    cleanup_available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppUpdateView {
    status: &'static str,
    selected_stream: &'static str,
    phase: &'static str,
    available_version: Option<String>,
    archive_size_bytes: Option<u64>,
    progress: Option<AppUpdateProgressView>,
    reason_code: &'static str,
    remediation: &'static str,
    can_select_stream: bool,
    can_check: bool,
    can_prepare: bool,
    can_install: bool,
    can_cancel: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppUpdateProgressView {
    completed_steps: u8,
    total_steps: u8,
    label: &'static str,
    indeterminate: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppIntegrationView {
    target: &'static str,
    label: &'static str,
    connection: AppConnectionView,
    client: AppClientView,
    plugin: AppPluginVersionView,
    discovery: &'static str,
    candidate_required: bool,
    legacy_detected: bool,
    migration: AppLegacyMigrationView,
    overall: &'static str,
    managed_content: AppManagedContentView,
    symbolic_location: &'static str,
    activation_policy: &'static str,
    host_action: Option<AppHostActionView>,
    ownership: &'static str,
    ownership_state: &'static str,
    next_action: &'static str,
    evidence_code: &'static str,
    paths: Vec<AppIntegrationPathView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppHostActionView {
    scope: &'static str,
    restart_required: bool,
    commands: Vec<AppHostCommandView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppHostCommandView {
    executable: &'static str,
    arguments: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppLegacyMigrationView {
    state: &'static str,
    detected_items: usize,
    eligible_items: usize,
    review_items: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppLegacyMigrationStatusView {
    state: &'static str,
    next_action: &'static str,
    migration_id: Option<String>,
    detected_items: usize,
    eligible_items: usize,
    review_items: usize,
    reason_code: &'static str,
    provider_conflicts: Vec<AppLegacyProviderConflictView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppLegacyProviderConflictView {
    provider: &'static str,
    differing_fields: Vec<String>,
    legacy_secret_present: bool,
    current_secret_reference_present: bool,
    default_strategy: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppLegacyProvider {
    Openalex,
    SemanticScholar,
    Crossref,
    Pubmed,
    Arxiv,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppLegacyProviderResolutionStrategy {
    KeepV2,
    UseLegacy,
    MergeCompatible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppLegacyProviderResolution {
    provider: AppLegacyProvider,
    strategy: AppLegacyProviderResolutionStrategy,
}

impl AppLegacyProviderResolution {
    const fn into_desktop(self) -> qiongli_ui::LegacyProviderResolutionView {
        qiongli_ui::LegacyProviderResolutionView {
            provider: match self.provider {
                AppLegacyProvider::Openalex => qiongli_ui::LegacyProviderView::OpenAlex,
                AppLegacyProvider::SemanticScholar => {
                    qiongli_ui::LegacyProviderView::SemanticScholar
                }
                AppLegacyProvider::Crossref => qiongli_ui::LegacyProviderView::Crossref,
                AppLegacyProvider::Pubmed => qiongli_ui::LegacyProviderView::Pubmed,
                AppLegacyProvider::Arxiv => qiongli_ui::LegacyProviderView::Arxiv,
            },
            strategy: match self.strategy {
                AppLegacyProviderResolutionStrategy::KeepV2 => {
                    qiongli_ui::LegacyProviderResolutionStrategyView::KeepV2
                }
                AppLegacyProviderResolutionStrategy::UseLegacy => {
                    qiongli_ui::LegacyProviderResolutionStrategyView::UseLegacy
                }
                AppLegacyProviderResolutionStrategy::MergeCompatible => {
                    qiongli_ui::LegacyProviderResolutionStrategyView::MergeCompatible
                }
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppLiteratureProvider {
    Openalex,
    SemanticScholar,
    Crossref,
    Pubmed,
    Arxiv,
}

impl AppLiteratureProvider {
    const fn into_desktop(self) -> ProviderKind {
        match self {
            Self::Openalex => ProviderKind::OpenAlex,
            Self::SemanticScholar => ProviderKind::SemanticScholar,
            Self::Crossref => ProviderKind::Crossref,
            Self::Pubmed => ProviderKind::PubMed,
            Self::Arxiv => ProviderKind::Arxiv,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppProviderEnablement {
    openalex: bool,
    semantic_scholar: bool,
    crossref: bool,
    pubmed: bool,
    arxiv: bool,
}

impl AppProviderEnablement {
    const fn into_desktop(self) -> [bool; 5] {
        [
            self.openalex,
            self.semantic_scholar,
            self.crossref,
            self.pubmed,
            self.arxiv,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppProviderSecretChange {
    Replace,
    Remove,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(tag = "change", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum AppProviderPublicSettingChange {
    Replace {
        provider: AppLiteratureProvider,
        value: String,
    },
    Remove {
        provider: AppLiteratureProvider,
    },
}

impl AppProviderPublicSettingChange {
    fn into_desktop(self) -> Result<(ProviderKind, PublicSettingChange), &'static str> {
        match self {
            Self::Replace { provider, value } if !value.is_empty() && value.len() <= 320 => Ok((
                provider.into_desktop(),
                PublicSettingChange::Replace(PrivateText::new(value)),
            )),
            Self::Remove { provider } => Ok((provider.into_desktop(), PublicSettingChange::Clear)),
            Self::Replace { .. } => Err("provider-public-setting-change-invalid"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppClientView {
    detected: bool,
    status: &'static str,
    version: Option<String>,
    compatibility: &'static str,
    minimum_supported_version: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppConnectionView {
    state: &'static str,
    label: &'static str,
    reason_code: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppPluginVersionView {
    installed_version: Option<String>,
    available_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppManagedContentView {
    source: &'static str,
    skills: &'static str,
    marketplace: &'static str,
    direct_package: Option<&'static str>,
    registration: &'static str,
    activation: &'static str,
    activation_observation: &'static str,
    mcp_attachment: &'static str,
    mcp_attachment_observation: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppIntegrationPathView {
    surface: &'static str,
    scope: &'static str,
    source: &'static str,
    state: &'static str,
    management: &'static str,
    selected: bool,
    symbolic_path: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppCapabilityView {
    refresh: bool,
    skills_materialize: bool,
    integration_discovery: bool,
    integration_preview: bool,
    project_library: bool,
    project_mutation: bool,
    capture_inbox: bool,
    capture_mutation: bool,
    capture_delivery: bool,
    capture_resolution: bool,
    academic_graph: bool,
    portfolio: bool,
    timeline: bool,
    orchestration_inspect: bool,
    orchestration_control: bool,
    legacy_credential_cleanup: bool,
    apply: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppIntegrationSelection {
    codex: bool,
    claude_code: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum AppAcademicGraphEntity {
    Node { id: String },
    Edge { id: String },
}

impl AppAcademicGraphEntity {
    pub(crate) fn into_parts(self) -> (AcademicGraphEntityKind, String) {
        match self {
            Self::Node { id } => (AcademicGraphEntityKind::Node, id),
            Self::Edge { id } => (AcademicGraphEntityKind::Edge, id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum AppProjectArtifactReference {
    AcademicGraphEntity {
        expected_projection_id: String,
        entity: AppAcademicGraphEntity,
    },
    RegisteredArtifact {
        artifact_path: String,
        source_anchor: Option<String>,
    },
}

impl AppProjectArtifactReference {
    fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::AcademicGraphEntity {
                expected_projection_id,
                entity,
            } => {
                if !valid_prefixed_app_digest(expected_projection_id, "grp_") {
                    return Err("app-project-artifact-projection-invalid");
                }
                let valid_entity = match entity {
                    AppAcademicGraphEntity::Node { id } => valid_prefixed_app_digest(id, "nod_"),
                    AppAcademicGraphEntity::Edge { id } => valid_prefixed_app_digest(id, "edg_"),
                };
                valid_entity
                    .then_some(())
                    .ok_or("app-project-artifact-entity-invalid")
            }
            Self::RegisteredArtifact {
                artifact_path,
                source_anchor,
            } => {
                const PATHS: [&str; 10] = [
                    "context/project_manifest.json",
                    "context/research_state.md",
                    "context/decision_log.md",
                    "context/stage_handoff.md",
                    "context/boundary_review.md",
                    "context/idea_funnel.md",
                    "literature/literature_map.md",
                    "evidence/claim-evidence-ledger.csv",
                    "manuscript/claims_evidence_map.md",
                    "graph/semantic_links.jsonl",
                ];
                if !PATHS.contains(&artifact_path.as_str())
                    || source_anchor.as_ref().is_some_and(|anchor| {
                        anchor.is_empty()
                            || anchor.len() > 512
                            || anchor.trim() != anchor
                            || anchor.chars().any(char::is_control)
                    })
                {
                    return Err("app-project-artifact-reference-invalid");
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppCaptureAssignmentDecision {
    Assign,
    Reject,
}

impl AppCaptureAssignmentDecision {
    pub(crate) const fn into_project(self) -> CaptureAssignmentDecision {
        match self {
            Self::Assign => CaptureAssignmentDecision::Assign,
            Self::Reject => CaptureAssignmentDecision::Reject,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppCaptureAssignmentStatusState {
    Pending,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppPortfolioMaintenanceOperation {
    Reconcile,
    FullRebuild,
    DeleteDerivedState,
}

impl AppPortfolioMaintenanceOperation {
    pub(crate) const fn into_project(self) -> PortfolioMaintenanceOperation {
        match self {
            Self::Reconcile => PortfolioMaintenanceOperation::Reconcile,
            Self::FullRebuild => PortfolioMaintenanceOperation::FullRebuild,
            Self::DeleteDerivedState => PortfolioMaintenanceOperation::DeleteDerivedState,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppContinuityCursorKind {
    Deliveries,
    Assignments,
    Resolutions,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppContinuityCursorV1 {
    schema_version: u32,
    cursor_id: String,
    kind: AppContinuityCursorKind,
    snapshot_id: String,
    after_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppCaptureDeliveryListRequestV1 {
    #[serde(default)]
    project_id: Option<ProjectId>,
    #[serde(default)]
    states: Vec<CaptureDeliveryState>,
    limit: u16,
    #[serde(default)]
    cursor: Option<AppContinuityCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppCaptureAssignmentListRequestV1 {
    #[serde(default)]
    project_id: Option<ProjectId>,
    #[serde(default)]
    states: Vec<AppCaptureAssignmentStatusState>,
    limit: u16,
    #[serde(default)]
    cursor: Option<AppContinuityCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppCaptureResolutionListRequestV1 {
    project_id: ProjectId,
    limit: u16,
    #[serde(default)]
    cursor: Option<AppContinuityCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppCaptureResolutionSelectionV1 {
    item_id: CaptureResolutionItemId,
    disposition: CaptureResolutionDisposition,
}

impl AppCaptureResolutionSelectionV1 {
    pub(crate) fn into_project(self) -> CaptureResolutionSelectionV1 {
        CaptureResolutionSelectionV1 {
            item_id: self.item_id,
            disposition: self.disposition,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppPortfolioSharedIdentityFilterV1 {
    node_type: AcademicGraphNodeType,
    canonical_id: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppPortfolioQueryFiltersV1 {
    #[serde(default)]
    project_id: Option<ProjectId>,
    #[serde(default)]
    stage: Option<ProjectStage>,
    #[serde(default)]
    evidence_signal: Option<PortfolioEvidenceSignal>,
    #[serde(default)]
    manuscript_section: Option<String>,
    #[serde(default)]
    shared_identity: Option<AppPortfolioSharedIdentityFilterV1>,
    #[serde(default)]
    capture_source: Option<CaptureSource>,
    #[serde(default)]
    capture_delivery: Option<CaptureDelivery>,
    #[serde(default)]
    delivery_state: Option<CaptureDeliveryState>,
    #[serde(default)]
    assignment_outcome: Option<CaptureAssignmentOutcome>,
    #[serde(default)]
    lineage_id: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppPortfolioQueryLimitsV1 {
    projects: u16,
    nodes: u16,
    edges: u16,
    lineage: u16,
    max_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppPortfolioQueryRequestV1 {
    catalog_id: String,
    #[serde(default)]
    filters: AppPortfolioQueryFiltersV1,
    limits: AppPortfolioQueryLimitsV1,
    #[serde(default)]
    cursor: Option<PortfolioQueryCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AppSemanticTimelineRequestV1 {
    catalog_id: String,
    #[serde(default)]
    project_id: Option<ProjectId>,
    view: SemanticTimelineView,
    limit: u16,
    max_bytes: usize,
    #[serde(default)]
    cursor: Option<SemanticTimelineCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureDeliveryDestinationV1 {
    project_id: ProjectId,
    expected_project_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureDeliveryAcknowledgementV1 {
    acknowledgement_id: String,
    destination_project_id: ProjectId,
    accepted_capture_id: String,
    expected_project_revision: u64,
    resulting_project_revision: u64,
    acknowledged_at_unix: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureDeliveryCapabilitiesV1 {
    can_retry: bool,
    can_cancel: bool,
    can_acknowledge: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureDeliveryViewV1 {
    schema_version: u32,
    envelope_id: String,
    capture_id: String,
    source: CaptureSource,
    delivery: CaptureDelivery,
    destination: Option<AppCaptureDeliveryDestinationV1>,
    state: CaptureDeliveryState,
    generation: u64,
    attempt_count: u32,
    retry_count: u32,
    created_at_unix: u64,
    updated_at_unix: u64,
    last_reason: CaptureDeliveryReason,
    envelope_sha256: String,
    record_sha256: String,
    acknowledgement: Option<AppCaptureDeliveryAcknowledgementV1>,
    capabilities: AppCaptureDeliveryCapabilitiesV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureDeliveryPageV1 {
    schema_version: u32,
    snapshot_id: String,
    project_id: Option<ProjectId>,
    entries: Vec<AppCaptureDeliveryViewV1>,
    truncated: bool,
    next_cursor: Option<AppContinuityCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureDeliveryAcknowledgementPreviewV1 {
    schema_version: u32,
    plan_digest: String,
    envelope_id: String,
    destination_project_id: ProjectId,
    accepted_capture_id: String,
    expected_project_revision: u64,
    resulting_project_revision: u64,
    acknowledged_at_unix: u64,
    expected_generation: u64,
    expected_record_sha256: String,
    approvals_required: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureAssignmentViewV1 {
    schema_version: u32,
    state: CaptureAssignmentStatusState,
    intent_id: String,
    source_envelope_id: String,
    source_capture_id: String,
    target_project_id: ProjectId,
    target_project_revision: u64,
    outcome: Option<CaptureAssignmentOutcome>,
    receipt_id: Option<String>,
    derived_capture_id: Option<String>,
    child_envelope_id: Option<String>,
    created_at_unix: u64,
    decided_at_unix: Option<u64>,
    can_resolve: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureAssignmentPageV1 {
    schema_version: u32,
    snapshot_id: String,
    project_id: Option<ProjectId>,
    entries: Vec<AppCaptureAssignmentViewV1>,
    truncated: bool,
    next_cursor: Option<AppContinuityCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureAssignmentPreviewV1 {
    schema_version: u32,
    plan_digest: String,
    intent_id: String,
    decision: CaptureAssignmentDecision,
    outcome: CaptureAssignmentPreviewOutcome,
    binding_effect: CaptureAssignmentBindingEffect,
    source_disposition: CaptureDisposition,
    source_envelope_id: String,
    source_capture_id: String,
    source_record_state: CaptureDeliveryState,
    expected_source_generation: u64,
    target_project_id: ProjectId,
    expected_library_revision: u64,
    expected_project_revision: u64,
    target_stage: ProjectStage,
    derived_capture_id: Option<String>,
    child_envelope_id: Option<String>,
    resolution_required: bool,
    decided_at_unix: u64,
    explanation: String,
    approvals_required: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureResolutionDecisionV1 {
    item_id: String,
    kind: CaptureResolutionItemKind,
    disposition: CaptureResolutionDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureResolutionViewV1 {
    schema_version: u32,
    receipt_id: String,
    assignment_receipt_id: String,
    source_envelope_id: String,
    source_capture_id: String,
    derived_capture_id: String,
    child_envelope_id: String,
    target_project_id: ProjectId,
    from_project_revision: u64,
    to_project_revision: u64,
    reviewed_at_unix: u64,
    resolved_at_unix: u64,
    decisions: Vec<AppCaptureResolutionDecisionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureResolutionPageV1 {
    schema_version: u32,
    snapshot_id: String,
    project_id: ProjectId,
    entries: Vec<AppCaptureResolutionViewV1>,
    truncated: bool,
    next_cursor: Option<AppContinuityCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureResolutionItemPreviewV1 {
    item_id: String,
    kind: CaptureResolutionItemKind,
    counterpart_state: CaptureResolutionCounterpartState,
    allowed_dispositions: Vec<CaptureResolutionDisposition>,
    unavailable_dispositions: Vec<CaptureResolutionDisposition>,
    source_summary: String,
    current_summary: Option<String>,
    explanation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppCaptureResolutionPreviewV1 {
    schema_version: u32,
    plan_digest: String,
    assignment_receipt_id: String,
    source_envelope_id: String,
    source_capture_id: String,
    derived_capture_id: String,
    child_envelope_id: String,
    target_project_id: ProjectId,
    expected_library_revision: u64,
    expected_project_revision: u64,
    next_project_revision: u64,
    reviewed_at_unix: u64,
    items: Vec<AppCaptureResolutionItemPreviewV1>,
    approvals_required: Vec<String>,
    exact_replay: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(
    dead_code,
    reason = "the strict App API contract reserves every truthful catalog state for C4"
)]
pub(crate) enum AppPortfolioCatalogState {
    Current,
    Missing,
    Stale,
    RecoveryRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioCapabilitiesV1 {
    can_query: bool,
    can_reconcile: bool,
    can_rebuild: bool,
    can_delete_derived_state: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioStatusV1 {
    schema_version: u32,
    state: AppPortfolioCatalogState,
    library_revision: u64,
    catalog_id: Option<String>,
    catalog_generation: Option<u64>,
    portfolio_id: Option<String>,
    contribution_count: usize,
    project_count: usize,
    node_count: usize,
    edge_count: usize,
    reason_code: &'static str,
    capabilities: AppPortfolioCapabilitiesV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioQueryProjectV1 {
    result_id: String,
    project_id: ProjectId,
    display_name: String,
    stage: ProjectStage,
    lifecycle: ProjectLifecycle,
    health: ProjectHealth,
    semantic_revision: u64,
    projection_id: String,
    node_count: usize,
    edge_count: usize,
    lineage_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioQueryNodeV1 {
    result_id: String,
    project_id: ProjectId,
    projection_id: String,
    node: AcademicGraphNodeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioQueryEdgeV1 {
    result_id: String,
    project_id: ProjectId,
    projection_id: String,
    edge: AcademicGraphEdgeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioLineageV1 {
    lineage_id: String,
    kind: PortfolioLineageKind,
    project_ids: Vec<ProjectId>,
    related_ids: Vec<String>,
    occurred_at_unix: u64,
    source: Option<CaptureSource>,
    delivery: Option<CaptureDelivery>,
    delivery_state: Option<CaptureDeliveryState>,
    assignment_outcome: Option<CaptureAssignmentOutcome>,
    from_project_revision: Option<u64>,
    to_project_revision: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioQueryResultV1 {
    schema_version: u32,
    request_id: String,
    query_id: String,
    catalog_id: String,
    portfolio_id: String,
    lineage_digest: String,
    matched_project_count: usize,
    matched_node_count: usize,
    matched_edge_count: usize,
    matched_lineage_count: usize,
    projects_truncated: bool,
    nodes_truncated: bool,
    edges_truncated: bool,
    lineage_truncated: bool,
    projects: Vec<AppPortfolioQueryProjectV1>,
    nodes: Vec<AppPortfolioQueryNodeV1>,
    edges: Vec<AppPortfolioQueryEdgeV1>,
    lineage: Vec<AppPortfolioLineageV1>,
    next_cursor: Option<PortfolioQueryCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSemanticActivityV1 {
    event_id: String,
    kind: SemanticActivityKind,
    occurred_at_unix: u64,
    timestamp_source: SemanticActivityTimestampSource,
    project_ids: Vec<ProjectId>,
    related_ids: Vec<String>,
    from_project_revision: Option<u64>,
    to_project_revision: Option<u64>,
    lifecycle: Option<ProjectLifecycle>,
    source: Option<CaptureSource>,
    delivery: Option<CaptureDelivery>,
    delivery_state: Option<CaptureDeliveryState>,
    delivery_reason: Option<CaptureDeliveryReason>,
    delivery_generation: Option<u64>,
    assignment_outcome: Option<CaptureAssignmentOutcome>,
    resolution_item_id: Option<String>,
    resolution_item_kind: Option<CaptureResolutionItemKind>,
    resolution_disposition: Option<CaptureResolutionDisposition>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSemanticTimelineResultV1 {
    schema_version: u32,
    request_id: String,
    query_id: String,
    catalog_id: String,
    portfolio_id: String,
    timeline_digest: String,
    project_id: Option<ProjectId>,
    view: SemanticTimelineView,
    matched_event_count: usize,
    truncated: bool,
    events: Vec<AppSemanticActivityV1>,
    next_cursor: Option<SemanticTimelineCursorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioDoctorV1 {
    schema_version: u32,
    status: PortfolioDoctorStatus,
    library_revision: u64,
    catalog_id: Option<String>,
    incremental_portfolio_id: Option<String>,
    clean_portfolio_id: String,
    byte_equivalent: bool,
    contribution_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioMaintenancePreviewV1 {
    schema_version: u32,
    plan_digest: String,
    operation: PortfolioMaintenanceOperation,
    expected_library_revision: u64,
    expected_catalog_id: Option<String>,
    expected_catalog_generation: Option<u64>,
    current_contribution_count: usize,
    derived_state_only: bool,
    explanation: String,
    approvals_required: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(
    dead_code,
    reason = "the strict App API contract reserves every native operation phase for C4"
)]
pub(crate) enum AppContinuityOperationPhase {
    Queued,
    Running,
    Completed,
    Cancelled,
    RecoveryRequired,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppContinuityOperationProgressV1 {
    schema_version: u32,
    operation_id: String,
    operation: PortfolioMaintenanceOperation,
    phase: AppContinuityOperationPhase,
    completed_units: usize,
    total_units: usize,
    catalog_id: Option<String>,
    cancellable: bool,
    reason_code: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppPortfolioMaintenanceResultV1 {
    schema_version: u32,
    operation_id: String,
    operation: PortfolioMaintenanceOperation,
    library_revision: u64,
    catalog_id: Option<String>,
    portfolio_id: Option<String>,
    catalog_changed: bool,
    rebuilt_project_count: usize,
    reused_project_count: usize,
    removed_project_count: usize,
    removed_contribution_count: usize,
    derived_state_only: bool,
}

#[allow(
    dead_code,
    reason = "host-handoff preview fields remain decode-only until their native UI is scheduled"
)]
#[derive(Deserialize)]
#[serde(
    tag = "action",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum AppIntent {
    Refresh,
    RefreshResearchLibrary,
    SelectProjectDirectory,
    SelectProjectCreateDestination {
        suggested_name: String,
    },
    PreviewProjectCreate {
        directory_token: String,
        display_name: String,
        project_kind: ProjectKind,
        stage: ProjectStage,
    },
    PreviewProjectRegister {
        directory_token: String,
    },
    OpenProject {
        project_id: String,
    },
    SelectProjectExportDestination {
        project_id: String,
    },
    PreviewProjectExport {
        directory_token: String,
    },
    SelectProjectImportLocations {
        suggested_name: String,
    },
    PreviewProjectImport {
        directory_token: String,
    },
    SelectProjectMigrationLocations {
        suggested_name: String,
    },
    PreviewProjectMigration {
        directory_token: String,
        display_name: String,
        project_kind: ProjectKind,
        stage: ProjectStage,
    },
    SelectProjectMigrationRecoveryLocations,
    PreviewProjectMigrationRecovery {
        directory_token: String,
    },
    SelectProjectMigrationRollbackLocations,
    PreviewProjectMigrationRollback {
        directory_token: String,
    },
    PreviewProjectRepairManifest {
        project_id: String,
    },
    PreviewProjectArchive {
        project_id: String,
    },
    PreviewProjectRestore {
        project_id: String,
    },
    PreviewProjectRefresh {
        project_id: String,
    },
    PreviewProjectUnregister {
        project_id: String,
    },
    LoadCaptureInbox {
        project_id: String,
    },
    LoadCaptureCoverage {
        project_id: String,
    },
    LoadArtifactChanges {
        project_id: String,
    },
    LoadAcademicGraph {
        project_id: String,
    },
    LoadAcademicGraphPortfolio,
    QueryAcademicGraph {
        project_id: String,
        query: AcademicGraphQueryV1,
    },
    QueryAcademicGraphPath {
        project_id: String,
        query: AcademicGraphPathQueryV1,
    },
    OpenAcademicGraphArtifact {
        project_id: String,
        expected_project_revision: u64,
        expected_projection_id: String,
        entity: AppAcademicGraphEntity,
    },
    ReadProjectArtifact {
        project_id: String,
        expected_project_revision: u64,
        reference: AppProjectArtifactReference,
        max_bytes: usize,
    },
    ReadCapture {
        project_id: String,
        capture_id: String,
    },
    SelectCaptureFile {
        project_id: String,
    },
    PreviewCaptureIntake {
        file_token: String,
    },
    PreviewCaptureConsolidation {
        project_id: String,
        capture_id: String,
    },
    LoadCaptureDeliveries {
        request: AppCaptureDeliveryListRequestV1,
    },
    InspectCaptureDelivery {
        envelope_id: DeliveryEnvelopeId,
    },
    RetryCaptureDelivery {
        envelope_id: DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: String,
        retried_at_unix: u64,
        cause: CaptureDeliveryRetryCause,
    },
    CancelCaptureDelivery {
        envelope_id: DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: String,
        cancelled_at_unix: u64,
    },
    PreviewCaptureDeliveryAcknowledgement {
        envelope_id: DeliveryEnvelopeId,
        destination_project_id: ProjectId,
        accepted_capture_id: CaptureId,
        expected_project_revision: u64,
        resulting_project_revision: u64,
        acknowledged_at_unix: u64,
        expected_generation: u64,
        expected_record_sha256: String,
    },
    LoadCaptureAssignments {
        request: AppCaptureAssignmentListRequestV1,
    },
    InspectCaptureAssignment {
        intent_id: CaptureAssignmentIntentId,
    },
    PreviewCaptureAssignment {
        source_envelope_id: DeliveryEnvelopeId,
        target_project_id: ProjectId,
        decision: AppCaptureAssignmentDecision,
        decided_at_unix: u64,
    },
    LoadCaptureResolutions {
        request: AppCaptureResolutionListRequestV1,
    },
    InspectCaptureResolution {
        project_id: ProjectId,
        receipt_id: CaptureResolutionReceiptId,
    },
    PreviewCaptureResolution {
        assignment_receipt_id: CaptureAssignmentReceiptId,
        reviewed_at_unix: u64,
        #[serde(default)]
        selections: Option<Vec<AppCaptureResolutionSelectionV1>>,
    },
    LoadPortfolioStatus,
    QueryPortfolio {
        request: AppPortfolioQueryRequestV1,
    },
    LoadSemanticTimeline {
        request: AppSemanticTimelineRequestV1,
    },
    LoadPortfolioDoctor,
    PreviewPortfolioMaintenance {
        operation: AppPortfolioMaintenanceOperation,
    },
    PollContinuityOperation {
        operation_id: String,
    },
    CancelContinuityOperation {
        operation_id: String,
    },
    RunFullMcpSelfTest,
    PollFullMcpSelfTest,
    CancelFullMcpSelfTest,
    RefreshIntegrationDiscovery,
    RefreshZoteroIntegration,
    PreviewZoteroCompanionStage,
    RevealZoteroCompanion,
    OpenZotero,
    VerifyZoteroIntegration,
    PrepareLegacyMigration {
        provider_resolutions: Vec<AppLegacyProviderResolution>,
    },
    PreviewLegacyMigrationNext,
    SelectUpdateStream {
        stream: AppUpdateStream,
    },
    CheckForUpdates,
    PrepareUpdate,
    PollUpdate,
    CancelUpdate,
    PreviewUpdateInstall,
    PreviewCliInstall,
    PreviewCliRemove,
    PreviewCliPathConfigure,
    TestCliCommand,
    LoadContentCustomization {
        profile: AppProfileId,
        project_id: Option<String>,
    },
    PreviewWorkflowResourceReplace {
        expected_revision: u64,
        expected_variant_sha256: Option<String>,
        path: String,
        expected_current_sha256: String,
        content: String,
    },
    PreviewWorkflowResourceReset {
        expected_revision: u64,
        expected_variant_sha256: Option<String>,
        path: String,
        expected_current_sha256: String,
    },
    PreviewProjectGuidance {
        project_id: String,
        expected_sha256: Option<String>,
        content: String,
    },
    PreviewProviderSettings {
        expected_revision: u64,
        providers_enabled: AppProviderEnablement,
        public_setting_changes: Vec<AppProviderPublicSettingChange>,
    },
    PreviewProviderSecretChange {
        provider: AppLiteratureProvider,
        change: AppProviderSecretChange,
        value: Option<String>,
    },
    TestLiteratureProvider {
        provider: AppLiteratureProvider,
    },
    PreviewRemoveAgentBackendCredential,
    LoadOrchestration {
        project_id: String,
        expected_project_revision: u64,
    },
    ControlOrchestration {
        project_id: String,
        expected_project_revision: u64,
        run_id: String,
        expected_generation: u64,
        expected_document_sha256: String,
        action_name: AppOrchestrationControlAction,
    },
    PreviewInstallRecommended,
    PreviewInstallSelected {
        selection: AppIntegrationSelection,
    },
    VerifyIntegrations {
        selection: AppIntegrationSelection,
    },
    PreviewReconcileIntegrations {
        selection: AppIntegrationSelection,
    },
    PreviewRemoveIntegrations {
        selection: AppIntegrationSelection,
    },
    SelectSkillsDestination,
    PreviewSkillsPresetMaterialization {
        profile: AppProfileId,
        preset: AppSkillsPreset,
    },
    PreviewProjectSkillsMaterialization {
        profile: AppProfileId,
        project_id: String,
    },
    VerifySkillsPreset {
        preset: AppSkillsPreset,
    },
    PreviewSkillsPresetRemoval {
        preset: AppSkillsPreset,
    },
    VerifyManagedSkillsTarget {
        target_id: String,
    },
    PreviewUpdateManagedSkillsTarget {
        target_id: String,
    },
    PreviewRemoveManagedSkillsTarget {
        target_id: String,
    },
    PreviewDetachManagedSkillsTarget {
        target_id: String,
    },
    ConfirmOperation {
        token: String,
    },
    CancelOperation {
        token: String,
    },
}

impl AppIntent {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        match self {
            Self::ReadProjectArtifact {
                expected_project_revision,
                reference,
                max_bytes,
                ..
            } => {
                if !valid_app_revision(*expected_project_revision)
                    || !(1_024..=256 * 1_024).contains(max_bytes)
                {
                    return Err("app-project-artifact-query-invalid");
                }
                reference.validate()
            }
            Self::LoadCaptureDeliveries { request } => request.validate(),
            Self::InspectCaptureDelivery { envelope_id } => validate_prefixed_domain_id(
                envelope_id.as_str(),
                "env_",
                "app-capture-delivery-id-invalid",
            ),
            Self::RetryCaptureDelivery {
                envelope_id,
                expected_generation,
                expected_record_sha256,
                retried_at_unix,
                ..
            } => {
                validate_prefixed_domain_id(
                    envelope_id.as_str(),
                    "env_",
                    "app-capture-delivery-id-invalid",
                )?;
                validate_record_reference(
                    *expected_generation,
                    expected_record_sha256,
                    *retried_at_unix,
                )
            }
            Self::CancelCaptureDelivery {
                envelope_id,
                expected_generation,
                expected_record_sha256,
                cancelled_at_unix,
                ..
            } => {
                validate_prefixed_domain_id(
                    envelope_id.as_str(),
                    "env_",
                    "app-capture-delivery-id-invalid",
                )?;
                validate_record_reference(
                    *expected_generation,
                    expected_record_sha256,
                    *cancelled_at_unix,
                )
            }
            Self::PreviewCaptureDeliveryAcknowledgement {
                envelope_id,
                destination_project_id,
                accepted_capture_id,
                expected_project_revision,
                resulting_project_revision,
                acknowledged_at_unix,
                expected_generation,
                expected_record_sha256,
                ..
            } => {
                validate_prefixed_domain_id(
                    envelope_id.as_str(),
                    "env_",
                    "app-capture-delivery-id-invalid",
                )?;
                validate_project_id(destination_project_id)?;
                validate_prefixed_domain_id(
                    accepted_capture_id.as_str(),
                    "cap_",
                    "app-capture-id-invalid",
                )?;
                validate_record_reference(
                    *expected_generation,
                    expected_record_sha256,
                    *acknowledged_at_unix,
                )?;
                if !valid_app_revision(*expected_project_revision)
                    || !valid_app_revision(*resulting_project_revision)
                    || resulting_project_revision < expected_project_revision
                {
                    return Err("app-capture-acknowledgement-revision-invalid");
                }
                Ok(())
            }
            Self::LoadCaptureAssignments { request } => request.validate(),
            Self::InspectCaptureAssignment { intent_id } => validate_prefixed_domain_id(
                intent_id.as_str(),
                "cai_",
                "app-capture-assignment-id-invalid",
            ),
            Self::PreviewCaptureAssignment {
                source_envelope_id,
                target_project_id,
                decided_at_unix,
                ..
            } => {
                validate_prefixed_domain_id(
                    source_envelope_id.as_str(),
                    "env_",
                    "app-capture-delivery-id-invalid",
                )?;
                validate_project_id(target_project_id)?;
                validate_app_timestamp(*decided_at_unix)
            }
            Self::LoadCaptureResolutions { request } => request.validate(),
            Self::InspectCaptureResolution {
                project_id,
                receipt_id,
            } => {
                validate_project_id(project_id)?;
                validate_prefixed_domain_id(
                    receipt_id.as_str(),
                    "crr_",
                    "app-capture-resolution-id-invalid",
                )
            }
            Self::PreviewCaptureResolution {
                assignment_receipt_id,
                reviewed_at_unix,
                selections,
                ..
            } => {
                validate_prefixed_domain_id(
                    assignment_receipt_id.as_str(),
                    "car_",
                    "app-capture-assignment-receipt-id-invalid",
                )?;
                validate_app_timestamp(*reviewed_at_unix)?;
                let selections = selections.as_deref().unwrap_or_default();
                if selections.len() > 80
                    || selections.iter().any(|selection| {
                        !valid_prefixed_app_digest(selection.item_id.as_str(), "cri_")
                    })
                    || selections.iter().enumerate().any(|(index, selection)| {
                        selections[index + 1..]
                            .iter()
                            .any(|candidate| candidate.item_id == selection.item_id)
                    })
                {
                    return Err("app-capture-resolution-selections-invalid");
                }
                Ok(())
            }
            Self::QueryPortfolio { request } => request.validate(),
            Self::LoadSemanticTimeline { request } => request.validate(),
            Self::PollContinuityOperation { operation_id }
            | Self::CancelContinuityOperation { operation_id } => {
                valid_prefixed_app_digest(operation_id, "cop_")
                    .then_some(())
                    .ok_or("app-continuity-operation-id-invalid")
            }
            Self::VerifyManagedSkillsTarget { target_id }
            | Self::PreviewUpdateManagedSkillsTarget { target_id }
            | Self::PreviewRemoveManagedSkillsTarget { target_id }
            | Self::PreviewDetachManagedSkillsTarget { target_id } => {
                valid_prefixed_app_digest(target_id, "skills-target-")
                    .then_some(())
                    .ok_or("managed-skills-target-id-invalid")
            }
            Self::LoadContentCustomization { project_id, .. } => {
                project_id.as_deref().map_or(Ok(()), |project_id| {
                    ProjectId::parse(project_id.to_owned())
                        .map(|_| ())
                        .map_err(|_| "app-project-id-invalid")
                })
            }
            Self::PreviewWorkflowResourceReplace {
                expected_revision,
                expected_variant_sha256,
                path,
                expected_current_sha256,
                content,
            } => validate_workflow_variant_reference(
                *expected_revision,
                expected_variant_sha256.as_deref(),
                path,
                expected_current_sha256,
            )
            .and_then(|()| {
                valid_app_text(content, 128 * 1_024)
                    .then_some(())
                    .ok_or("workflow-variant-content-invalid")
            }),
            Self::PreviewWorkflowResourceReset {
                expected_revision,
                expected_variant_sha256,
                path,
                expected_current_sha256,
            } => validate_workflow_variant_reference(
                *expected_revision,
                expected_variant_sha256.as_deref(),
                path,
                expected_current_sha256,
            ),
            Self::PreviewProjectGuidance {
                project_id,
                expected_sha256,
                content,
            } => {
                ProjectId::parse(project_id.to_owned()).map_err(|_| "app-project-id-invalid")?;
                if expected_sha256
                    .as_deref()
                    .is_some_and(|value| !valid_app_sha256(value))
                    || !valid_app_text(content, 32 * 1_024)
                {
                    return Err("project-guidance-invalid");
                }
                Ok(())
            }
            Self::LoadPortfolioStatus
            | Self::LoadPortfolioDoctor
            | Self::PreviewPortfolioMaintenance { .. } => Ok(()),
            _ => Ok(()),
        }
    }
}

impl AppCaptureDeliveryListRequestV1 {
    pub(crate) const fn project_id(&self) -> Option<&ProjectId> {
        self.project_id.as_ref()
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self
            .project_id
            .as_ref()
            .is_some_and(|project_id| validate_project_id(project_id).is_err())
            || self.limit == 0
            || self.limit > 256
            || self.states.len() > 7
            || has_duplicates(&self.states)
            || self
                .cursor
                .as_ref()
                .is_some_and(|cursor| !cursor.valid_for(AppContinuityCursorKind::Deliveries))
        {
            return Err("app-capture-delivery-page-invalid");
        }
        Ok(())
    }
}

impl AppCaptureAssignmentListRequestV1 {
    pub(crate) const fn project_id(&self) -> Option<&ProjectId> {
        self.project_id.as_ref()
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self
            .project_id
            .as_ref()
            .is_some_and(|project_id| validate_project_id(project_id).is_err())
            || self.limit == 0
            || self.limit > 256
            || self.states.len() > 2
            || has_duplicates(&self.states)
            || self
                .cursor
                .as_ref()
                .is_some_and(|cursor| !cursor.valid_for(AppContinuityCursorKind::Assignments))
        {
            return Err("app-capture-assignment-page-invalid");
        }
        Ok(())
    }
}

impl AppCaptureResolutionListRequestV1 {
    pub(crate) const fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    fn validate(&self) -> Result<(), &'static str> {
        if validate_project_id(&self.project_id).is_err()
            || self.limit == 0
            || self.limit > 128
            || self
                .cursor
                .as_ref()
                .is_some_and(|cursor| !cursor.valid_for(AppContinuityCursorKind::Resolutions))
        {
            return Err("app-capture-resolution-page-invalid");
        }
        Ok(())
    }
}

impl AppContinuityCursorV1 {
    fn valid_for(&self, kind: AppContinuityCursorKind) -> bool {
        let (snapshot_prefix, after_valid) = match kind {
            AppContinuityCursorKind::Deliveries => (
                "dls_",
                DeliveryEnvelopeId::parse(self.after_id.clone()).is_ok(),
            ),
            AppContinuityCursorKind::Assignments => (
                "als_",
                CaptureAssignmentIntentId::parse(self.after_id.clone()).is_ok(),
            ),
            AppContinuityCursorKind::Resolutions => (
                "rls_",
                CaptureResolutionReceiptId::parse(self.after_id.clone()).is_ok(),
            ),
        };
        self.schema_version == 1
            && self.kind == kind
            && valid_prefixed_app_digest(&self.cursor_id, "apc_")
            && valid_prefixed_app_digest(&self.snapshot_id, snapshot_prefix)
            && after_valid
            && app_continuity_cursor_id(self.kind, &self.snapshot_id, &self.after_id)
                .is_ok_and(|expected| expected == self.cursor_id)
    }
}

impl AppPortfolioQueryRequestV1 {
    fn validate(&self) -> Result<(), &'static str> {
        if !valid_prefixed_app_digest(&self.catalog_id, "pca_")
            || self
                .filters
                .project_id
                .as_ref()
                .is_some_and(|project_id| validate_project_id(project_id).is_err())
            || !self.filters.validate()
            || self.limits.projects == 0
            || self.limits.projects > 128
            || self.limits.nodes == 0
            || self.limits.nodes > 256
            || self.limits.edges == 0
            || self.limits.edges > 256
            || self.limits.lineage == 0
            || self.limits.lineage > 256
            || !(65_536..=4 * 1_024 * 1_024).contains(&self.limits.max_bytes)
            || self.cursor.as_ref().is_some_and(|cursor| {
                !valid_prefixed_app_digest(&cursor.cursor_id, "pqc_")
                    || !valid_prefixed_app_digest(&cursor.query_id, "pqy_")
            })
        {
            return Err("app-portfolio-query-invalid");
        }
        Ok(())
    }
}

impl AppPortfolioQueryFiltersV1 {
    fn validate(&self) -> bool {
        self.manuscript_section
            .as_deref()
            .is_none_or(|value| valid_app_text(value, 512))
            && self.shared_identity.as_ref().is_none_or(|identity| {
                matches!(
                    identity.node_type,
                    AcademicGraphNodeType::Paper
                        | AcademicGraphNodeType::Concept
                        | AcademicGraphNodeType::Method
                ) && valid_app_text(&identity.canonical_id, 512)
            })
            && self
                .lineage_id
                .as_deref()
                .is_none_or(|value| valid_app_text(value, 160))
            && self
                .text
                .as_deref()
                .is_none_or(|value| valid_app_text(value, 256))
    }
}

impl AppSemanticTimelineRequestV1 {
    fn validate(&self) -> Result<(), &'static str> {
        if !valid_prefixed_app_digest(&self.catalog_id, "pca_")
            || self
                .project_id
                .as_ref()
                .is_some_and(|project_id| validate_project_id(project_id).is_err())
            || self.limit == 0
            || self.limit > 512
            || !(65_536..=4 * 1_024 * 1_024).contains(&self.max_bytes)
            || self.cursor.as_ref().is_some_and(|cursor| {
                !valid_prefixed_app_digest(&cursor.cursor_id, "ptc_")
                    || !valid_prefixed_app_digest(&cursor.query_id, "pty_")
                    || !valid_prefixed_app_digest(&cursor.after_event_id, "pte_")
                    || !valid_app_timestamp_value(cursor.after_occurred_at_unix)
            })
        {
            return Err("app-semantic-timeline-query-invalid");
        }
        Ok(())
    }
}

fn validate_record_reference(
    generation: u64,
    record_sha256: &str,
    occurred_at_unix: u64,
) -> Result<(), &'static str> {
    if !valid_app_revision(generation) || !valid_app_sha256(record_sha256) {
        return Err("app-capture-delivery-reference-invalid");
    }
    validate_app_timestamp(occurred_at_unix)
}

fn validate_project_id(project_id: &ProjectId) -> Result<(), &'static str> {
    ProjectId::parse(project_id.as_str().to_owned())
        .map(|_| ())
        .map_err(|_| "app-project-id-invalid")
}

fn validate_prefixed_domain_id(
    value: &str,
    prefix: &str,
    error: &'static str,
) -> Result<(), &'static str> {
    valid_prefixed_app_digest(value, prefix)
        .then_some(())
        .ok_or(error)
}

fn validate_app_timestamp(value: u64) -> Result<(), &'static str> {
    valid_app_timestamp_value(value)
        .then_some(())
        .ok_or("app-timestamp-invalid")
}

const fn valid_app_timestamp_value(value: u64) -> bool {
    value <= 9_007_199_254_740_991
}

const fn valid_app_revision(value: u64) -> bool {
    value > 0 && valid_app_timestamp_value(value)
}

fn valid_app_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_prefixed_app_digest(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(valid_app_sha256)
}

fn valid_app_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value
            .chars()
            .all(|character| !character.is_control() || character == '\n' || character == '\t')
}

fn validate_workflow_variant_reference(
    expected_revision: u64,
    expected_variant_sha256: Option<&str>,
    path: &str,
    expected_current_sha256: &str,
) -> Result<(), &'static str> {
    let editable_path = path == "workflow/SKILL.md"
        || path.strip_prefix("skills/").is_some_and(|suffix| {
            !suffix.is_empty()
                && suffix.ends_with(".md")
                && !suffix
                    .split('/')
                    .any(|part| part.is_empty() || matches!(part, "." | ".."))
        });
    if !valid_app_timestamp_value(expected_revision)
        || expected_variant_sha256.is_some_and(|value| !valid_app_sha256(value))
        || !valid_app_sha256(expected_current_sha256)
        || path.len() > 512
        || !editable_path
    {
        return Err("workflow-variant-reference-invalid");
    }
    Ok(())
}

fn has_duplicates<T: Eq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[index + 1..].contains(value))
}

pub(crate) fn app_capture_delivery_view(
    status: CaptureDeliveryStatusV1,
) -> AppCaptureDeliveryViewV1 {
    let can_retry = matches!(
        status.state,
        CaptureDeliveryState::Delivering
            | CaptureDeliveryState::Delivered
            | CaptureDeliveryState::Conflicted
    );
    let can_cancel = matches!(
        status.state,
        CaptureDeliveryState::Queued
            | CaptureDeliveryState::Delivering
            | CaptureDeliveryState::Delivered
            | CaptureDeliveryState::RetryRequired
            | CaptureDeliveryState::Conflicted
    );
    let can_acknowledge = status.state == CaptureDeliveryState::Delivered
        && status.destination.is_some()
        && status.acknowledgement.is_none();
    AppCaptureDeliveryViewV1 {
        schema_version: status.schema_version,
        envelope_id: status.envelope_id.as_str().to_owned(),
        capture_id: status.capture_id.as_str().to_owned(),
        source: status.source,
        delivery: status.delivery,
        destination: status
            .destination
            .map(|destination| AppCaptureDeliveryDestinationV1 {
                project_id: destination.project_id,
                expected_project_revision: destination.expected_project_revision,
            }),
        state: status.state,
        generation: status.generation,
        attempt_count: status.attempt_count,
        retry_count: status.retry_count,
        created_at_unix: status.created_at_unix,
        updated_at_unix: status.updated_at_unix,
        last_reason: status.last_reason,
        envelope_sha256: status.envelope_sha256,
        record_sha256: status.record_sha256,
        acknowledgement: status.acknowledgement.map(|acknowledgement| {
            AppCaptureDeliveryAcknowledgementV1 {
                acknowledgement_id: acknowledgement.acknowledgement_id.as_str().to_owned(),
                destination_project_id: acknowledgement.destination_project_id,
                accepted_capture_id: acknowledgement.accepted_capture_id.as_str().to_owned(),
                expected_project_revision: acknowledgement.expected_project_revision,
                resulting_project_revision: acknowledgement.resulting_project_revision,
                acknowledged_at_unix: acknowledgement.acknowledged_at_unix,
            }
        }),
        capabilities: AppCaptureDeliveryCapabilitiesV1 {
            can_retry,
            can_cancel,
            can_acknowledge,
        },
    }
}

pub(crate) fn app_capture_delivery_acknowledgement_preview(
    preview: &CaptureDeliveryAcknowledgementPreviewV1,
) -> AppCaptureDeliveryAcknowledgementPreviewV1 {
    AppCaptureDeliveryAcknowledgementPreviewV1 {
        schema_version: preview.schema_version,
        plan_digest: preview.plan_digest.clone(),
        envelope_id: preview.envelope_id.as_str().to_owned(),
        destination_project_id: preview.destination_project_id.clone(),
        accepted_capture_id: preview.accepted_capture_id.as_str().to_owned(),
        expected_project_revision: preview.expected_project_revision,
        resulting_project_revision: preview.resulting_project_revision,
        acknowledged_at_unix: preview.acknowledged_at_unix,
        expected_generation: preview.expected_generation,
        expected_record_sha256: preview.expected_record_sha256.clone(),
        approvals_required: preview.approvals_required.clone(),
    }
}

pub(crate) fn app_capture_delivery_page(
    request: AppCaptureDeliveryListRequestV1,
    statuses: Vec<CaptureDeliveryStatusV1>,
) -> Result<AppCaptureDeliveryPageV1, &'static str> {
    request.validate()?;
    let AppCaptureDeliveryListRequestV1 {
        project_id,
        states,
        limit,
        cursor,
    } = request;
    let mut entries = statuses
        .into_iter()
        .filter(|status| states.is_empty() || states.contains(&status.state))
        .map(app_capture_delivery_view)
        .collect::<Vec<_>>();
    let scope = (&project_id, &states);
    let page = paginate_app_continuity(
        AppContinuityPageRequest {
            kind: AppContinuityCursorKind::Deliveries,
            snapshot_prefix: "dls_",
            domain: b"qiongli-app-delivery-list-v1\0",
            scope: &scope,
            limit: usize::from(limit),
            cursor,
            identity: |entry: &AppCaptureDeliveryViewV1| entry.envelope_id.as_str(),
        },
        &mut entries,
    )?;
    Ok(AppCaptureDeliveryPageV1 {
        schema_version: 1,
        snapshot_id: page.snapshot_id,
        project_id,
        entries: page.entries,
        truncated: page.truncated,
        next_cursor: page.next_cursor,
    })
}

pub(crate) fn app_capture_assignment_view(
    status: CaptureAssignmentStatusV1,
    can_resolve: bool,
) -> AppCaptureAssignmentViewV1 {
    AppCaptureAssignmentViewV1 {
        schema_version: status.schema_version,
        state: status.state,
        intent_id: status.intent_id.as_str().to_owned(),
        source_envelope_id: status.source_envelope_id.as_str().to_owned(),
        source_capture_id: status.source_capture_id.as_str().to_owned(),
        target_project_id: status.target_project_id,
        target_project_revision: status.target_project_revision,
        outcome: status.outcome,
        receipt_id: status.receipt_id.map(|value| value.as_str().to_owned()),
        derived_capture_id: status
            .derived_capture_id
            .map(|value| value.as_str().to_owned()),
        child_envelope_id: status
            .child_envelope_id
            .map(|value| value.as_str().to_owned()),
        created_at_unix: status.created_at_unix,
        decided_at_unix: status.decided_at_unix,
        can_resolve,
    }
}

pub(crate) fn app_capture_assignment_preview(
    preview: &CaptureAssignmentPreviewV1,
) -> AppCaptureAssignmentPreviewV1 {
    let explanation = match preview.outcome {
        CaptureAssignmentPreviewOutcome::Ready => {
            "Assign the capture to the selected project and retain exact source-to-child lineage."
        }
        CaptureAssignmentPreviewOutcome::Duplicate => {
            "Record the explicit assignment while reusing the exact matching capture identity."
        }
        CaptureAssignmentPreviewOutcome::ResolutionRequired => {
            "Assign the capture and require item-scoped academic review before changing canonical project artifacts."
        }
        CaptureAssignmentPreviewOutcome::Rejected => {
            "Record the explicit rejection and retain the immutable source delivery lineage."
        }
    };
    AppCaptureAssignmentPreviewV1 {
        schema_version: preview.schema_version,
        plan_digest: preview.plan_digest.clone(),
        intent_id: preview.intent_id.as_str().to_owned(),
        decision: preview.decision,
        outcome: preview.outcome,
        binding_effect: preview.binding_effect,
        source_disposition: preview.source_disposition,
        source_envelope_id: preview.source_envelope_id.as_str().to_owned(),
        source_capture_id: preview.source_capture_id.as_str().to_owned(),
        source_record_state: preview.source_record_state,
        expected_source_generation: preview.expected_source_generation,
        target_project_id: preview.target_project_id.clone(),
        expected_library_revision: preview.expected_library_revision,
        expected_project_revision: preview.expected_project_revision,
        target_stage: preview.target_stage,
        derived_capture_id: preview
            .derived_capture_id
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        child_envelope_id: preview
            .child_envelope_id
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        resolution_required: preview.resolution_required,
        decided_at_unix: preview.decided_at_unix,
        explanation: explanation.to_owned(),
        approvals_required: preview.approvals_required.clone(),
    }
}

pub(crate) fn app_capture_assignment_page(
    request: AppCaptureAssignmentListRequestV1,
    statuses: Vec<CaptureAssignmentStatusV1>,
    resolvable_receipt_ids: &std::collections::BTreeSet<String>,
) -> Result<AppCaptureAssignmentPageV1, &'static str> {
    request.validate()?;
    let AppCaptureAssignmentListRequestV1 {
        project_id,
        states,
        limit,
        cursor,
    } = request;
    let mut entries = statuses
        .into_iter()
        .filter(|status| {
            project_id
                .as_ref()
                .is_none_or(|candidate| status.target_project_id == *candidate)
        })
        .map(|status| {
            let can_resolve = status
                .receipt_id
                .as_ref()
                .is_some_and(|receipt_id| resolvable_receipt_ids.contains(receipt_id.as_str()));
            app_capture_assignment_view(status, can_resolve)
        })
        .filter(|assignment| {
            states.is_empty()
                || states.iter().any(|state| {
                    matches!(
                        (state, assignment.state),
                        (
                            AppCaptureAssignmentStatusState::Pending,
                            CaptureAssignmentStatusState::Pending
                        ) | (
                            AppCaptureAssignmentStatusState::Completed,
                            CaptureAssignmentStatusState::Completed
                        )
                    )
                })
        })
        .collect::<Vec<_>>();
    let scope = (&project_id, &states);
    let page = paginate_app_continuity(
        AppContinuityPageRequest {
            kind: AppContinuityCursorKind::Assignments,
            snapshot_prefix: "als_",
            domain: b"qiongli-app-assignment-list-v1\0",
            scope: &scope,
            limit: usize::from(limit),
            cursor,
            identity: |entry: &AppCaptureAssignmentViewV1| entry.intent_id.as_str(),
        },
        &mut entries,
    )?;
    Ok(AppCaptureAssignmentPageV1 {
        schema_version: 1,
        snapshot_id: page.snapshot_id,
        project_id,
        entries: page.entries,
        truncated: page.truncated,
        next_cursor: page.next_cursor,
    })
}

pub(crate) fn app_capture_resolution_view(
    receipt: CaptureResolutionReceiptV1,
) -> AppCaptureResolutionViewV1 {
    AppCaptureResolutionViewV1 {
        schema_version: receipt.schema_version,
        receipt_id: receipt.receipt_id.as_str().to_owned(),
        assignment_receipt_id: receipt.receipt.assignment_receipt_id.as_str().to_owned(),
        source_envelope_id: receipt.receipt.source_envelope_id.as_str().to_owned(),
        source_capture_id: receipt.receipt.source_capture_id.as_str().to_owned(),
        derived_capture_id: receipt.receipt.derived_capture_id.as_str().to_owned(),
        child_envelope_id: receipt.receipt.child_envelope_id.as_str().to_owned(),
        target_project_id: receipt.receipt.target_project_id,
        from_project_revision: receipt.receipt.from_project_revision,
        to_project_revision: receipt.receipt.to_project_revision,
        reviewed_at_unix: receipt.receipt.reviewed_at_unix,
        resolved_at_unix: receipt.receipt.resolved_at_unix,
        decisions: receipt
            .receipt
            .decisions
            .into_iter()
            .map(|decision| AppCaptureResolutionDecisionV1 {
                item_id: decision.item.item_id.as_str().to_owned(),
                kind: decision.item.kind,
                disposition: decision.disposition,
            })
            .collect(),
    }
}

pub(crate) fn app_capture_resolution_preview(
    preview: &CaptureResolutionPreviewV1,
) -> AppCaptureResolutionPreviewV1 {
    AppCaptureResolutionPreviewV1 {
        schema_version: preview.schema_version,
        plan_digest: preview.plan_digest.clone(),
        assignment_receipt_id: preview.assignment_receipt_id.as_str().to_owned(),
        source_envelope_id: preview.source_envelope_id.as_str().to_owned(),
        source_capture_id: preview.source_capture_id.as_str().to_owned(),
        derived_capture_id: preview.derived_capture_id.as_str().to_owned(),
        child_envelope_id: preview.child_envelope_id.as_str().to_owned(),
        target_project_id: preview.target_project_id.clone(),
        expected_library_revision: preview.expected_library_revision,
        expected_project_revision: preview.expected_project_revision,
        next_project_revision: preview.next_project_revision,
        reviewed_at_unix: preview.reviewed_at_unix,
        items: preview
            .items
            .iter()
            .map(|item| AppCaptureResolutionItemPreviewV1 {
                item_id: item.item.item_id.as_str().to_owned(),
                kind: item.item.kind,
                counterpart_state: item.item.counterpart_state,
                allowed_dispositions: item.item.allowed_dispositions.clone(),
                unavailable_dispositions: item.unavailable_dispositions.clone(),
                source_summary: app_capture_resolution_content_summary(&item.source),
                current_summary: item
                    .current
                    .as_ref()
                    .map(app_capture_resolution_content_summary),
                explanation: bounded_app_summary(&item.explanation),
            })
            .collect(),
        approvals_required: preview.approvals_required.clone(),
        exact_replay: preview.exact_replay,
    }
}

fn app_capture_resolution_content_summary(content: &CaptureResolutionItemContentV1) -> String {
    let summary = match content {
        CaptureResolutionItemContentV1::SemanticChange(change) => change.summary.clone(),
        CaptureResolutionItemContentV1::Decision(decision) => {
            format!("{} — {}", decision.statement, decision.rationale)
        }
        CaptureResolutionItemContentV1::Evidence(evidence) => {
            evidence.limitation.as_ref().map_or_else(
                || evidence.relevance.clone(),
                |limitation| format!("{} — {}", evidence.relevance, limitation),
            )
        }
        CaptureResolutionItemContentV1::Contradiction(contradiction) => format!(
            "{} — {}",
            contradiction.statement, contradiction.consequence
        ),
        CaptureResolutionItemContentV1::NextAction(action) => action.clone(),
    };
    bounded_app_summary(&summary)
}

fn bounded_app_summary(value: &str) -> String {
    const MAX_APP_SUMMARY_BYTES: usize = 4_096;
    if value.len() <= MAX_APP_SUMMARY_BYTES {
        return value.to_owned();
    }
    let mut boundary = MAX_APP_SUMMARY_BYTES;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value[..boundary].to_owned()
}

pub(crate) fn app_capture_resolution_page(
    request: AppCaptureResolutionListRequestV1,
    receipts: Vec<CaptureResolutionReceiptV1>,
) -> Result<AppCaptureResolutionPageV1, &'static str> {
    request.validate()?;
    let AppCaptureResolutionListRequestV1 {
        project_id,
        limit,
        cursor,
    } = request;
    let mut entries = receipts
        .into_iter()
        .map(app_capture_resolution_view)
        .collect::<Vec<_>>();
    let page = paginate_app_continuity(
        AppContinuityPageRequest {
            kind: AppContinuityCursorKind::Resolutions,
            snapshot_prefix: "rls_",
            domain: b"qiongli-app-resolution-list-v1\0",
            scope: &project_id,
            limit: usize::from(limit),
            cursor,
            identity: |entry: &AppCaptureResolutionViewV1| entry.receipt_id.as_str(),
        },
        &mut entries,
    )?;
    Ok(AppCaptureResolutionPageV1 {
        schema_version: 1,
        snapshot_id: page.snapshot_id,
        project_id,
        entries: page.entries,
        truncated: page.truncated,
        next_cursor: page.next_cursor,
    })
}

struct AppContinuityPageRequest<'a, S, T> {
    kind: AppContinuityCursorKind,
    snapshot_prefix: &'static str,
    domain: &'static [u8],
    scope: &'a S,
    limit: usize,
    cursor: Option<AppContinuityCursorV1>,
    identity: fn(&T) -> &str,
}

struct AppContinuityPage<T> {
    snapshot_id: String,
    entries: Vec<T>,
    truncated: bool,
    next_cursor: Option<AppContinuityCursorV1>,
}

fn paginate_app_continuity<T, S>(
    request: AppContinuityPageRequest<'_, S, T>,
    entries: &mut Vec<T>,
) -> Result<AppContinuityPage<T>, &'static str>
where
    T: Serialize,
    S: Serialize,
{
    let AppContinuityPageRequest {
        kind,
        snapshot_prefix,
        domain,
        scope,
        limit,
        cursor,
        identity,
    } = request;
    entries.sort_by(|left, right| identity(left).cmp(identity(right)));
    let snapshot_id = app_prefixed_digest(snapshot_prefix, domain, &(scope, &*entries))?;
    let start = if let Some(cursor) = cursor {
        if !cursor.valid_for(kind) || cursor.snapshot_id != snapshot_id {
            return Err("app-continuity-cursor-stale");
        }
        entries
            .iter()
            .position(|entry| identity(entry) == cursor.after_id)
            .map(|index| index + 1)
            .ok_or("app-continuity-cursor-stale")?
    } else {
        0
    };
    let mut page = entries
        .drain(start..)
        .take(limit.saturating_add(1))
        .collect::<Vec<_>>();
    let truncated = page.len() > limit;
    if truncated {
        page.truncate(limit);
    }
    let next_cursor = if truncated {
        let after_id = page
            .last()
            .map(|entry| identity(entry).to_owned())
            .ok_or("app-continuity-page-invalid")?;
        Some(AppContinuityCursorV1::new(
            kind,
            snapshot_id.clone(),
            after_id,
        )?)
    } else {
        None
    };
    Ok(AppContinuityPage {
        snapshot_id,
        entries: page,
        truncated,
        next_cursor,
    })
}

impl AppContinuityCursorV1 {
    fn new(
        kind: AppContinuityCursorKind,
        snapshot_id: String,
        after_id: String,
    ) -> Result<Self, &'static str> {
        let cursor_id = app_continuity_cursor_id(kind, &snapshot_id, &after_id)?;
        Ok(Self {
            schema_version: 1,
            cursor_id,
            kind,
            snapshot_id,
            after_id,
        })
    }
}

fn app_continuity_cursor_id(
    kind: AppContinuityCursorKind,
    snapshot_id: &str,
    after_id: &str,
) -> Result<String, &'static str> {
    app_prefixed_digest(
        "apc_",
        b"qiongli-app-continuity-cursor-v1\0",
        &(kind, snapshot_id, after_id),
    )
}

fn app_prefixed_digest<T: Serialize>(
    prefix: &str,
    domain: &[u8],
    value: &T,
) -> Result<String, &'static str> {
    let bytes =
        serde_json_canonicalizer::to_vec(value).map_err(|_| "app-continuity-identity-invalid")?;
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    Ok(format!("{prefix}{:x}", hasher.finalize()))
}

pub(crate) fn app_portfolio_current_status(
    current: &IncrementalPortfolioSnapshotV1,
) -> AppPortfolioStatusV1 {
    AppPortfolioStatusV1 {
        schema_version: 1,
        state: AppPortfolioCatalogState::Current,
        library_revision: current.catalog.library_revision,
        catalog_id: Some(current.catalog.catalog_id.clone()),
        catalog_generation: Some(current.catalog.generation),
        portfolio_id: Some(current.portfolio.portfolio_id.clone()),
        contribution_count: current.catalog.contribution_count,
        project_count: current.portfolio.project_count,
        node_count: current.portfolio.node_count,
        edge_count: current.portfolio.edge_count,
        reason_code: "portfolio-current",
        capabilities: AppPortfolioCapabilitiesV1 {
            can_query: true,
            can_reconcile: true,
            can_rebuild: true,
            can_delete_derived_state: true,
        },
    }
}

pub(crate) fn app_portfolio_unavailable_status(
    library: &ResearchLibrarySnapshotV1,
    state: AppPortfolioCatalogState,
) -> AppPortfolioStatusV1 {
    let (reason_code, can_reconcile, can_rebuild, can_delete_derived_state) = match state {
        AppPortfolioCatalogState::Current => ("portfolio-current", true, true, true),
        AppPortfolioCatalogState::Missing => ("portfolio-missing", true, true, false),
        AppPortfolioCatalogState::Stale => ("portfolio-stale", true, true, true),
        AppPortfolioCatalogState::RecoveryRequired => {
            ("portfolio-recovery-required", false, false, false)
        }
    };
    AppPortfolioStatusV1 {
        schema_version: 1,
        state,
        library_revision: library.revision,
        catalog_id: None,
        catalog_generation: None,
        portfolio_id: None,
        contribution_count: 0,
        project_count: library.projects.len(),
        node_count: 0,
        edge_count: 0,
        reason_code,
        capabilities: AppPortfolioCapabilitiesV1 {
            can_query: false,
            can_reconcile,
            can_rebuild,
            can_delete_derived_state,
        },
    }
}

pub(crate) fn app_portfolio_query(
    request: AppPortfolioQueryRequestV1,
) -> Result<PortfolioQueryV1, &'static str> {
    request.validate()?;
    let filters = PortfolioQueryFiltersV1 {
        project_id: request.filters.project_id,
        stage: request.filters.stage,
        evidence_signal: request.filters.evidence_signal,
        manuscript_section: request.filters.manuscript_section,
        shared_identity: request.filters.shared_identity.map(|identity| {
            PortfolioSharedIdentityFilterV1 {
                node_type: identity.node_type,
                canonical_id: identity.canonical_id,
            }
        }),
        capture_source: request.filters.capture_source,
        capture_delivery: request.filters.capture_delivery,
        delivery_state: request.filters.delivery_state,
        assignment_outcome: request.filters.assignment_outcome,
        lineage_id: request.filters.lineage_id,
        text: request.filters.text,
    };
    let limits = PortfolioQueryLimitsV1 {
        projects: usize::from(request.limits.projects),
        nodes: usize::from(request.limits.nodes),
        edges: usize::from(request.limits.edges),
        lineage: usize::from(request.limits.lineage),
        max_bytes: request.limits.max_bytes,
    };
    let mut query = PortfolioQueryV1::new(request.catalog_id)
        .and_then(|query| query.with_filters(filters))
        .and_then(|query| query.with_limits(limits))
        .map_err(|error| error.reason_code())?;
    if let Some(cursor) = request.cursor {
        query = query
            .with_cursor(cursor)
            .map_err(|error| error.reason_code())?;
    }
    Ok(query)
}

pub(crate) fn app_portfolio_query_result(
    result: PortfolioQueryResultV1,
) -> AppPortfolioQueryResultV1 {
    AppPortfolioQueryResultV1 {
        schema_version: result.schema_version,
        request_id: result.request_id,
        query_id: result.query_id,
        catalog_id: result.catalog_id,
        portfolio_id: result.portfolio_id,
        lineage_digest: result.lineage_digest,
        matched_project_count: result.matched_project_count,
        matched_node_count: result.matched_node_count,
        matched_edge_count: result.matched_edge_count,
        matched_lineage_count: result.matched_lineage_count,
        projects_truncated: result.projects_truncated,
        nodes_truncated: result.nodes_truncated,
        edges_truncated: result.edges_truncated,
        lineage_truncated: result.lineage_truncated,
        projects: result
            .projects
            .into_iter()
            .map(|project| AppPortfolioQueryProjectV1 {
                result_id: project.result_id,
                project_id: project.project_id,
                display_name: project.display_name,
                stage: project.stage,
                lifecycle: project.lifecycle,
                health: project.health,
                semantic_revision: project.semantic_revision,
                projection_id: project.projection_id,
                node_count: project.node_count,
                edge_count: project.edge_count,
                lineage_count: project.lineage_count,
            })
            .collect(),
        nodes: result
            .nodes
            .into_iter()
            .map(|node| AppPortfolioQueryNodeV1 {
                result_id: node.result_id,
                project_id: node.project_id,
                projection_id: node.projection_id,
                node: node.node,
            })
            .collect(),
        edges: result
            .edges
            .into_iter()
            .map(|edge| AppPortfolioQueryEdgeV1 {
                result_id: edge.result_id,
                project_id: edge.project_id,
                projection_id: edge.projection_id,
                edge: edge.edge,
            })
            .collect(),
        lineage: result
            .lineage
            .into_iter()
            .map(|lineage| AppPortfolioLineageV1 {
                lineage_id: lineage.lineage_id,
                kind: lineage.kind,
                project_ids: lineage.project_ids,
                related_ids: lineage.related_ids,
                occurred_at_unix: lineage.occurred_at_unix,
                source: lineage.source,
                delivery: lineage.delivery,
                delivery_state: lineage.delivery_state,
                assignment_outcome: lineage.assignment_outcome,
                from_project_revision: lineage.from_project_revision,
                to_project_revision: lineage.to_project_revision,
            })
            .collect(),
        next_cursor: result.next_cursor,
    }
}

pub(crate) fn app_semantic_timeline_query(
    request: AppSemanticTimelineRequestV1,
) -> Result<SemanticTimelineQueryV1, &'static str> {
    request.validate()?;
    let mut query = SemanticTimelineQueryV1::new(request.catalog_id)
        .and_then(|query| query.with_view(request.view))
        .and_then(|query| query.with_limits(usize::from(request.limit), request.max_bytes))
        .map_err(|error| error.reason_code())?;
    if let Some(project_id) = request.project_id {
        query = query
            .for_project(project_id)
            .map_err(|error| error.reason_code())?;
    }
    if let Some(cursor) = request.cursor {
        query = query
            .with_cursor(cursor)
            .map_err(|error| error.reason_code())?;
    }
    Ok(query)
}

pub(crate) fn app_semantic_timeline_result(
    result: SemanticTimelineResultV1,
) -> AppSemanticTimelineResultV1 {
    AppSemanticTimelineResultV1 {
        schema_version: result.schema_version,
        request_id: result.request_id,
        query_id: result.query_id,
        catalog_id: result.catalog_id,
        portfolio_id: result.portfolio_id,
        timeline_digest: result.timeline_digest,
        project_id: result.project_id,
        view: result.view,
        matched_event_count: result.matched_event_count,
        truncated: result.truncated,
        events: result
            .events
            .into_iter()
            .map(|event| AppSemanticActivityV1 {
                event_id: event.event_id,
                kind: event.kind,
                occurred_at_unix: event.occurred_at_unix,
                timestamp_source: event.timestamp_source,
                project_ids: event.project_ids,
                related_ids: event.related_ids,
                from_project_revision: event.from_project_revision,
                to_project_revision: event.to_project_revision,
                lifecycle: event.lifecycle,
                source: event.source,
                delivery: event.delivery,
                delivery_state: event.delivery_state,
                delivery_reason: event.delivery_reason,
                delivery_generation: event.delivery_generation,
                assignment_outcome: event.assignment_outcome,
                resolution_item_id: event.resolution_item_id,
                resolution_item_kind: event.resolution_item_kind,
                resolution_disposition: event.resolution_disposition,
            })
            .collect(),
        next_cursor: result.next_cursor,
    }
}

pub(crate) fn app_portfolio_doctor(doctor: PortfolioDoctorV1) -> AppPortfolioDoctorV1 {
    AppPortfolioDoctorV1 {
        schema_version: doctor.schema_version,
        status: doctor.status,
        library_revision: doctor.library_revision,
        catalog_id: doctor.catalog_id,
        incremental_portfolio_id: doctor.incremental_portfolio_id,
        clean_portfolio_id: doctor.clean_portfolio_id,
        byte_equivalent: doctor.byte_equivalent,
        contribution_count: doctor.contribution_count,
    }
}

pub(crate) fn app_portfolio_maintenance_preview(
    preview: &PortfolioMaintenancePreviewV1,
) -> AppPortfolioMaintenancePreviewV1 {
    let explanation = match preview.operation {
        PortfolioMaintenanceOperation::Reconcile => {
            "Reconcile only changed or missing derived project contributions against the current Research Library. Canonical academic artifacts are retained."
        }
        PortfolioMaintenanceOperation::FullRebuild => {
            "Rebuild every derived project contribution from the current registered canonical artifacts. Canonical academic artifacts are retained."
        }
        PortfolioMaintenanceOperation::DeleteDerivedState => {
            "Delete only the private rebuildable portfolio catalog and contributions. Registered projects and canonical academic artifacts are retained."
        }
    };
    AppPortfolioMaintenancePreviewV1 {
        schema_version: preview.schema_version,
        plan_digest: preview.plan_digest.clone(),
        operation: preview.operation,
        expected_library_revision: preview.expected_library_revision,
        expected_catalog_id: preview.expected_catalog_id.clone(),
        expected_catalog_generation: preview.expected_catalog_generation,
        current_contribution_count: preview.current_contribution_count,
        derived_state_only: preview.derived_state_only,
        explanation: explanation.to_owned(),
        approvals_required: preview.approvals_required.clone(),
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "the strict App progress contract requires every bounded native field"
)]
pub(crate) fn app_continuity_operation_progress(
    operation_id: String,
    operation: PortfolioMaintenanceOperation,
    phase: AppContinuityOperationPhase,
    completed_units: usize,
    total_units: usize,
    catalog_id: Option<String>,
    cancellable: bool,
    reason_code: &'static str,
) -> AppContinuityOperationProgressV1 {
    AppContinuityOperationProgressV1 {
        schema_version: 1,
        operation_id,
        operation,
        phase,
        completed_units,
        total_units,
        catalog_id,
        cancellable,
        reason_code,
    }
}

pub(crate) fn app_portfolio_reconciliation_result(
    operation_id: String,
    operation: PortfolioMaintenanceOperation,
    reconciliation: PortfolioReconciliationV1,
) -> AppPortfolioMaintenanceResultV1 {
    AppPortfolioMaintenanceResultV1 {
        schema_version: reconciliation.schema_version,
        operation_id,
        operation,
        library_revision: reconciliation.snapshot.catalog.library_revision,
        catalog_id: Some(reconciliation.snapshot.catalog.catalog_id),
        portfolio_id: Some(reconciliation.snapshot.portfolio.portfolio_id),
        catalog_changed: reconciliation.catalog_changed,
        rebuilt_project_count: reconciliation.rebuilt_project_count,
        reused_project_count: reconciliation.reused_project_count,
        removed_project_count: reconciliation.removed_project_count,
        removed_contribution_count: 0,
        derived_state_only: true,
    }
}

pub(crate) fn app_portfolio_deletion_result(
    operation_id: String,
    deletion: PortfolioDerivedStateDeletionV1,
) -> AppPortfolioMaintenanceResultV1 {
    AppPortfolioMaintenanceResultV1 {
        schema_version: deletion.schema_version,
        operation_id,
        operation: PortfolioMaintenanceOperation::DeleteDerivedState,
        library_revision: deletion.library_revision,
        catalog_id: None,
        portfolio_id: None,
        catalog_changed: deletion.removed_catalog_id.is_some(),
        rebuilt_project_count: 0,
        reused_project_count: 0,
        removed_project_count: 0,
        removed_contribution_count: deletion.removed_contribution_count,
        derived_state_only: deletion.derived_state_only,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppProfileId {
    SkillOnly,
    MarketplaceLite,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppUpdateStream {
    Stable,
    Beta,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppSkillsPreset {
    QiongliManaged,
    CurrentProject,
    CustomFolder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AppOrchestrationControlAction {
    Pause,
    Recover,
    Resume,
    Cancel,
}

macro_rules! define_app_events {
    ($(
        $variant:ident { $($field:ident: $field_type:ty),* $(,)? } => $event_type:literal
    ),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
        #[serde(
            tag = "type",
            rename_all = "kebab-case",
            rename_all_fields = "camelCase"
        )]
        pub(crate) enum AppEvent {
            $($variant { $($field: $field_type),* },)+
        }

        impl AppEvent {
            const fn contract_type(&self) -> &'static str {
                match self {
                    $(Self::$variant { .. } => $event_type,)+
                }
            }
        }

        const APP_EVENT_VARIANT_COUNT: usize = [$($event_type),+].len();
    };
}

define_app_events! {
    Snapshot { snapshot: AppSnapshotV1 } => "snapshot",
    Preview { preview: AppOperationPreview } => "preview",
    ContentCustomization {
        customization: AppContentCustomizationV1,
    } => "content-customization",
    SkillsDestinationSelected {
        target_id: String,
        symbolic_path: &'static str,
    } => "skills-destination-selected",
    CaptureInbox { inbox: CaptureInboxSnapshotV1 } => "capture-inbox",
    CaptureCoverage { coverage: CaptureCoverageSnapshotV1 } => "capture-coverage",
    ArtifactChanges { changes: ArtifactChangeSnapshotV1 } => "artifact-changes",
    AcademicGraph {
        graph: AcademicGraphSnapshotV1,
        readiness: AcademicGraphReadinessV1,
        comparison: Option<AcademicGraphRevisionComparisonV1>,
    } => "academic-graph",
    AcademicGraphPortfolio {
        portfolio: AcademicGraphPortfolioSnapshotV1,
    } => "academic-graph-portfolio",
    AcademicGraphQuery { result: AcademicGraphQueryResultV1 } => "academic-graph-query",
    AcademicGraphPath { result: AcademicGraphPathResultV1 } => "academic-graph-path",
    AcademicGraphArtifactOpened {
        project_id: ProjectId,
        project_revision: u64,
        projection_id: String,
        entity: AppAcademicGraphEntity,
    } => "academic-graph-artifact-opened",
    ProjectArtifactRead {
        artifact: ProjectArtifactViewV1,
    } => "project-artifact-read",
    CaptureRead { capture: AppResearchCaptureV1 } => "capture-read",
    CaptureFileSelected { token: String, file_label: String } => "capture-file-selected",
    CaptureIntakePreview {
        intake: CaptureIntakePreviewV1,
        preview: AppOperationPreview,
    } => "capture-intake-preview",
    CaptureConsolidationPreview {
        consolidation: CaptureConsolidationPreviewV1,
        preview: AppOperationPreview,
    } => "capture-consolidation-preview",
    CaptureDeliveries { page: AppCaptureDeliveryPageV1 } => "capture-deliveries",
    CaptureDeliveryInspected {
        delivery: AppCaptureDeliveryViewV1,
    } => "capture-delivery-inspected",
    CaptureDeliveryUpdated {
        delivery: AppCaptureDeliveryViewV1,
    } => "capture-delivery-updated",
    CaptureDeliveryAcknowledgementPreview {
        acknowledgement: AppCaptureDeliveryAcknowledgementPreviewV1,
        preview: AppOperationPreview,
    } => "capture-delivery-acknowledgement-preview",
    CaptureAssignments { page: AppCaptureAssignmentPageV1 } => "capture-assignments",
    CaptureAssignmentInspected {
        assignment: AppCaptureAssignmentViewV1,
    } => "capture-assignment-inspected",
    CaptureAssignmentPreview {
        assignment: AppCaptureAssignmentPreviewV1,
        preview: AppOperationPreview,
    } => "capture-assignment-preview",
    CaptureResolutions { page: AppCaptureResolutionPageV1 } => "capture-resolutions",
    CaptureResolutionInspected {
        resolution: AppCaptureResolutionViewV1,
    } => "capture-resolution-inspected",
    CaptureResolutionPlan {
        resolution: AppCaptureResolutionPreviewV1,
    } => "capture-resolution-plan",
    CaptureResolutionPreview {
        resolution: AppCaptureResolutionPreviewV1,
        selections: Vec<AppCaptureResolutionSelectionV1>,
        preview: AppOperationPreview,
    } => "capture-resolution-preview",
    PortfolioStatus { portfolio: AppPortfolioStatusV1 } => "portfolio-status",
    PortfolioQuery { result: AppPortfolioQueryResultV1 } => "portfolio-query",
    SemanticTimeline { result: AppSemanticTimelineResultV1 } => "semantic-timeline",
    PortfolioDoctor { doctor: AppPortfolioDoctorV1 } => "portfolio-doctor",
    PortfolioMaintenancePreview {
        maintenance: AppPortfolioMaintenancePreviewV1,
        preview: AppOperationPreview,
    } => "portfolio-maintenance-preview",
    ContinuityOperationProgress {
        progress: AppContinuityOperationProgressV1,
    } => "continuity-operation-progress",
    PortfolioMaintenanceCompleted {
        result: AppPortfolioMaintenanceResultV1,
    } => "portfolio-maintenance-completed",
    ProjectDirectorySelected { token: String, root_label: String } => "project-directory-selected",
    ProjectMigrationCompleted {
        code: &'static str,
        snapshot: AppSnapshotV1,
        qualification: AppProjectMigrationQualification,
    } => "project-migration-completed",
    UpdateChanged { update: AppUpdateView, close_requested: bool } => "update-changed",
    McpSelfTestUpdated { self_test: AppMcpSelfTestView } => "mcp-self-test-updated",
    OrchestrationLoaded { runs: OrchestrationRunListViewV1 } => "orchestration-loaded",
    OrchestrationRunUpdated {
        run: OrchestrationRunSummaryV1,
        runs: OrchestrationRunListViewV1,
    } => "orchestration-run-updated",
    Completed { code: &'static str, snapshot: AppSnapshotV1 } => "completed",
    CaptureOperationCompleted {
        code: &'static str,
        snapshot: Box<AppSnapshotV1>,
        inbox: CaptureInboxSnapshotV1,
        coverage: CaptureCoverageSnapshotV1,
        changes: ArtifactChangeSnapshotV1,
        delivery: Box<Option<AppCaptureDeliveryViewV1>>,
        assignment: Box<Option<AppCaptureAssignmentViewV1>>,
        resolution: Box<Option<AppCaptureResolutionViewV1>>,
    } => "capture-operation-completed",
    Cancelled { code: &'static str } => "cancelled",
    ValidationFailed { code: &'static str } => "validation-failed",
    Failed { code: &'static str } => "failed",
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppApiContractFixtureV1 {
    schema_version: u32,
    snapshot: AppSnapshotV1,
    events: Vec<AppEvent>,
}

pub(crate) fn serialize_app_api_contract_fixture(
    snapshot: AppSnapshotV1,
) -> Result<String, &'static str> {
    let project_id = ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051")
        .map_err(|_| "app-api-contract-project-id-invalid")?;
    let capture = canonical_contract_capture(project_id.clone())?;
    let capture_id = capture.capture_id.clone();
    let inbox = canonical_contract_inbox(project_id.clone());
    let coverage = canonical_contract_coverage(project_id.clone());
    let changes = canonical_contract_artifact_changes(project_id.clone());
    let (graph, graph_query, graph_path) = canonical_contract_graph(project_id.clone())?;
    let continuity_events = canonical_contract_continuity_events(project_id.clone(), &graph)?;
    let graph_artifact_opened = AppEvent::AcademicGraphArtifactOpened {
        project_id: graph.project_id.clone(),
        project_revision: graph.project_revision,
        projection_id: graph.projection_id.clone(),
        entity: AppAcademicGraphEntity::Node {
            id: graph.nodes[0].node_id.clone(),
        },
    };
    let artifact_content = "{\"displayName\":\"Canonical article project\"}\n".to_owned();
    let project_artifact_read = AppEvent::ProjectArtifactRead {
        artifact: ProjectArtifactViewV1 {
            schema_version: 1,
            document_kind: "qiongli-project-artifact-view".to_owned(),
            project_id: graph.project_id.clone(),
            project_revision: graph.project_revision,
            projection_id: Some(graph.projection_id.clone()),
            entity_kind: Some(AcademicGraphEntityKind::Node),
            entity_id: Some(graph.nodes[0].node_id.clone()),
            artifact_path: graph.nodes[0].artifact_path.clone(),
            source_anchor: Some(graph.nodes[0].source_anchor.clone()),
            format: ProjectArtifactFormat::Json,
            content_digest: format!("{:x}", Sha256::digest(artifact_content.as_bytes())),
            source_size_bytes: artifact_content.len() as u64,
            content_size_bytes: artifact_content.len() as u64,
            content: artifact_content,
            start_line: 1,
            end_line: 2,
            anchor_line: Some(1),
            anchor_matched: true,
            truncated_before: false,
            truncated_after: false,
        },
    };
    let project_preview = ProjectMutationPreviewV1 {
        schema_version: 1,
        plan_digest: "c".repeat(64),
        operation: ProjectMutationKind::Refresh,
        effect: ProjectMutationEffect::UpdateSemanticRevision,
        project_id: project_id.clone(),
        display_name: "Canonical article project".to_owned(),
        project_kind: ProjectKind::Article,
        stage: ProjectStage::Writing,
        expected_library_revision: 0,
        expected_project_revision: Some(1),
        root_label: "canonical-project".to_owned(),
        manifest_action: "advance-semantic-revision".to_owned(),
        missing_continuity_artifacts: Vec::new(),
        approvals_required: vec!["filesystem-write".to_owned()],
    };
    let intake = CaptureIntakePreviewV1 {
        schema_version: CAPTURE_INTAKE_SCHEMA_VERSION,
        plan_digest: "a".repeat(64),
        capture_id: capture_id.clone(),
        project_id: project_id.clone(),
        disposition: CaptureDisposition::Refinement,
        effect: CaptureIntakeEffect::AppendPendingHistory,
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Connected,
        expected_library_revision: 0,
        expected_project_revision: 1,
        change_count: 0,
        decision_count: 0,
        evidence_count: 0,
        contradiction_count: 0,
        next_action_count: 0,
        history_entry: format!("captures/history/{}.json", capture_id.as_str()),
        approvals_required: vec!["filesystem-write".to_owned()],
    };
    let consolidation = CaptureConsolidationPreviewV1 {
        schema_version: ACADEMIC_CONSOLIDATION_SCHEMA_VERSION,
        plan_digest: "b".repeat(64),
        capture_id,
        project_id,
        disposition: CaptureDisposition::Refinement,
        outcome: CaptureConsolidationOutcome::Ready,
        expected_library_revision: 0,
        expected_project_revision: 1,
        next_project_revision: Some(2),
        project_stage: ProjectStage::Writing,
        reviewed_at_unix: 1,
        conflicts: Vec::new(),
        artifact_deltas: Vec::new(),
        receipt_entry: "captures/consolidated/canonical.json".to_owned(),
        approvals_required: vec![
            "academic-consolidation".to_owned(),
            "filesystem-write".to_owned(),
        ],
    };
    let intake_operation = app_capture_intake_operation_preview(
        "0000000000000000000000000000002c".to_owned(),
        "canonical-capture.json".to_owned(),
        &intake,
    );
    let consolidation_operation = app_capture_consolidation_operation_preview(
        "0000000000000000000000000000002d".to_owned(),
        &consolidation,
    );
    let project_operation = app_project_operation_preview(
        "0000000000000000000000000000002e".to_owned(),
        &project_preview,
    );
    let orchestration_project_id = ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051")
        .map_err(|_| "app-api-contract-project-id-invalid")?;
    let orchestration_run = OrchestrationRunSummaryV1 {
        run_id: qiongli_execution::RunId::parse(format!("run_{}", "2".repeat(32)))
            .map_err(|_| "app-api-contract-run-id-invalid")?,
        profile_id: format!("host-solo-{}", "a".repeat(24)),
        execution_mode: qiongli_execution::OrchestrationExecutionMode::Solo,
        status: qiongli_execution::OrchestrationRunStatus::Running,
        generation: 3,
        document_sha256: "3".repeat(64),
        completed_task_count: 1,
        total_task_count: 76,
        next_task_id: Some("A1_5".to_owned()),
        active_task_id: None,
        active_role: None,
        completed_role_count: 0,
        required_role_count: 1,
        host_driven: true,
        recovery_required: false,
        can_continue: true,
        can_pause: true,
        can_resume: false,
        can_recover: false,
        can_cancel: true,
    };
    let orchestration_runs = OrchestrationRunListViewV1 {
        schema_version: 1,
        project_id: orchestration_project_id,
        expected_project_revision: 1,
        runs: vec![orchestration_run.clone()],
    };
    let update = snapshot.update.clone();
    let mut events = vec![
        AppEvent::Snapshot {
            snapshot: snapshot.clone(),
        },
        AppEvent::Preview {
            preview: project_operation,
        },
        AppEvent::SkillsDestinationSelected {
            target_id: format!("skills-target-{}", "1".repeat(64)),
            symbolic_path: "<custom-folder>",
        },
        AppEvent::ContentCustomization {
            customization: AppContentCustomizationV1 {
                profile: "skill-only",
                state: "canonical",
                revision: 0,
                variant_sha256: None,
                cleanup_required: false,
                resources: vec![AppContentPreviewResourceV1 {
                    path: "workflow/SKILL.md".to_owned(),
                    format: "markdown",
                    editable: true,
                    canonical_sha256: "1".repeat(64),
                    current_sha256: "1".repeat(64),
                    overridden: false,
                    content: "# Qiongli\n".to_owned(),
                }],
                guidance: None,
            },
        },
        AppEvent::CaptureInbox {
            inbox: inbox.clone(),
        },
        AppEvent::CaptureCoverage {
            coverage: coverage.clone(),
        },
        AppEvent::ArtifactChanges {
            changes: changes.clone(),
        },
        AppEvent::AcademicGraph {
            readiness: AcademicGraphReadinessV1::from_graph(&graph),
            graph,
            comparison: None,
        },
        AppEvent::AcademicGraphPortfolio {
            portfolio: AcademicGraphPortfolioSnapshotV1 {
                schema_version: 1,
                document_kind: "qiongli-academic-graph-portfolio".to_owned(),
                portfolio_id: format!("gpf_{}", "0".repeat(64)),
                library_revision: 0,
                project_count: 0,
                included_project_count: 0,
                skipped_project_count: 0,
                node_count: 0,
                edge_count: 0,
                projects: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
            },
        },
        AppEvent::AcademicGraphQuery {
            result: graph_query,
        },
        AppEvent::AcademicGraphPath { result: graph_path },
        graph_artifact_opened,
        project_artifact_read,
        AppEvent::CaptureRead {
            capture: capture.into(),
        },
        AppEvent::ProjectDirectorySelected {
            token: "0000000000000000000000000000002a".to_owned(),
            root_label: "canonical-project".to_owned(),
        },
        AppEvent::ProjectMigrationCompleted {
            code: "project-migration-completed",
            snapshot: snapshot.clone(),
            qualification: AppProjectMigrationQualification::verified(
                ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051")
                    .map_err(|_| "app-api-contract-project-id-invalid")?,
                format!("grp_{}", "7".repeat(64)),
                format!("gix_{}", "8".repeat(64)),
            ),
        },
        AppEvent::CaptureFileSelected {
            token: "0000000000000000000000000000002b".to_owned(),
            file_label: "canonical-capture.json".to_owned(),
        },
        AppEvent::CaptureIntakePreview {
            intake,
            preview: intake_operation,
        },
        AppEvent::CaptureConsolidationPreview {
            consolidation,
            preview: consolidation_operation,
        },
    ];
    events.extend(continuity_events);
    events.extend([
        AppEvent::UpdateChanged {
            update,
            close_requested: true,
        },
        AppEvent::McpSelfTestUpdated {
            self_test: AppMcpSelfTestView {
                profile: "full",
                product_version: env!("CARGO_PKG_VERSION"),
                state: "passed",
                checks: [
                    "embedded-contract",
                    "initialize",
                    "tool-registry",
                    "full-dispatch",
                    "provider-readiness",
                    "client-registration",
                ]
                .map(|check| AppMcpSelfTestCheckView {
                    check,
                    status: "ready",
                    code: "canonical-self-test-check-ready",
                    remediation: "none",
                }),
                public_tool_count: full_public_tool_count(),
                enabled_provider_count: 0,
                ready_provider_count: 0,
                discovered_client_count: 2,
                registered_client_count: 2,
            },
        },
        AppEvent::OrchestrationLoaded {
            runs: orchestration_runs.clone(),
        },
        AppEvent::OrchestrationRunUpdated {
            run: orchestration_run,
            runs: orchestration_runs,
        },
        AppEvent::Completed {
            code: "canonical-operation-completed",
            snapshot: snapshot.clone(),
        },
        AppEvent::CaptureOperationCompleted {
            code: "canonical-capture-operation-completed",
            snapshot: Box::new(snapshot.clone()),
            inbox,
            coverage,
            changes,
            delivery: Box::new(None),
            assignment: Box::new(None),
            resolution: Box::new(None),
        },
        AppEvent::Cancelled {
            code: "canonical-operation-cancelled",
        },
        AppEvent::ValidationFailed {
            code: "canonical-validation-failed",
        },
        AppEvent::Failed {
            code: "canonical-operation-failed",
        },
    ]);
    let mut covered_event_types = events
        .iter()
        .map(AppEvent::contract_type)
        .collect::<Vec<_>>();
    covered_event_types.sort_unstable();
    covered_event_types.dedup();
    if covered_event_types.len() != APP_EVENT_VARIANT_COUNT
        || events.iter().any(|event| {
            serde_json::to_value(event)
                .ok()
                .and_then(|value| value.get("type")?.as_str().map(str::to_owned))
                .as_deref()
                != Some(event.contract_type())
        })
    {
        return Err("app-api-contract-event-coverage-incomplete");
    }
    serde_json::to_string_pretty(&AppApiContractFixtureV1 {
        schema_version: APP_API_SCHEMA_VERSION,
        snapshot,
        events,
    })
    .map(|rendered| format!("{rendered}\n"))
    .map_err(|_| "app-api-contract-fixture-serialization-failed")
}

fn canonical_contract_continuity_events(
    project_id: ProjectId,
    graph: &AcademicGraphSnapshotV1,
) -> Result<Vec<AppEvent>, &'static str> {
    let envelope_id = format!("env_{}", "1".repeat(64));
    let child_envelope_id = format!("env_{}", "2".repeat(64));
    let capture_id = format!("cap_{}", "3".repeat(64));
    let derived_capture_id = format!("cap_{}", "4".repeat(64));
    let acknowledgement_id = format!("dack_{}", "5".repeat(64));
    let assignment_intent_id = format!("cai_{}", "6".repeat(64));
    let assignment_receipt_id = format!("car_{}", "7".repeat(64));
    let resolution_item_id = format!("cri_{}", "8".repeat(64));
    let resolution_receipt_id = format!("crr_{}", "9".repeat(64));
    let catalog_id = format!("pca_{}", "a".repeat(64));
    let portfolio_id = format!("gpf_{}", "b".repeat(64));
    let delivery_snapshot_id = format!("dls_{}", "c".repeat(64));
    let assignment_snapshot_id = format!("als_{}", "d".repeat(64));
    let resolution_snapshot_id = format!("rls_{}", "e".repeat(64));
    let delivery_cursor = AppContinuityCursorV1 {
        schema_version: 1,
        cursor_id: format!("apc_{}", "f".repeat(64)),
        kind: AppContinuityCursorKind::Deliveries,
        snapshot_id: delivery_snapshot_id.clone(),
        after_id: envelope_id.clone(),
    };
    let delivery = AppCaptureDeliveryViewV1 {
        schema_version: 1,
        envelope_id: envelope_id.clone(),
        capture_id: capture_id.clone(),
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Connected,
        destination: Some(AppCaptureDeliveryDestinationV1 {
            project_id: project_id.clone(),
            expected_project_revision: 1,
        }),
        state: CaptureDeliveryState::Acknowledged,
        generation: 4,
        attempt_count: 1,
        retry_count: 0,
        created_at_unix: 1,
        updated_at_unix: 4,
        last_reason: CaptureDeliveryReason::DeliveryAcknowledged,
        envelope_sha256: "1".repeat(64),
        record_sha256: "2".repeat(64),
        acknowledgement: Some(AppCaptureDeliveryAcknowledgementV1 {
            acknowledgement_id,
            destination_project_id: project_id.clone(),
            accepted_capture_id: capture_id.clone(),
            expected_project_revision: 1,
            resulting_project_revision: 2,
            acknowledged_at_unix: 4,
        }),
        capabilities: AppCaptureDeliveryCapabilitiesV1 {
            can_retry: false,
            can_cancel: false,
            can_acknowledge: false,
        },
    };
    let delivery_page = AppCaptureDeliveryPageV1 {
        schema_version: 1,
        snapshot_id: delivery_snapshot_id,
        project_id: Some(project_id.clone()),
        entries: vec![delivery.clone()],
        truncated: true,
        next_cursor: Some(delivery_cursor),
    };
    let acknowledgement = AppCaptureDeliveryAcknowledgementPreviewV1 {
        schema_version: 1,
        plan_digest: "3".repeat(64),
        envelope_id: envelope_id.clone(),
        destination_project_id: project_id.clone(),
        accepted_capture_id: capture_id.clone(),
        expected_project_revision: 1,
        resulting_project_revision: 2,
        acknowledged_at_unix: 4,
        expected_generation: 3,
        expected_record_sha256: "4".repeat(64),
        approvals_required: vec!["delivery-acknowledgement".to_owned()],
    };
    let assignment = AppCaptureAssignmentViewV1 {
        schema_version: 1,
        state: CaptureAssignmentStatusState::Completed,
        intent_id: assignment_intent_id.clone(),
        source_envelope_id: envelope_id.clone(),
        source_capture_id: capture_id.clone(),
        target_project_id: project_id.clone(),
        target_project_revision: 1,
        outcome: Some(CaptureAssignmentOutcome::Assigned),
        receipt_id: Some(assignment_receipt_id.clone()),
        derived_capture_id: Some(derived_capture_id.clone()),
        child_envelope_id: Some(child_envelope_id.clone()),
        created_at_unix: 2,
        decided_at_unix: Some(2),
        can_resolve: true,
    };
    let assignment_page = AppCaptureAssignmentPageV1 {
        schema_version: 1,
        snapshot_id: assignment_snapshot_id,
        project_id: Some(project_id.clone()),
        entries: vec![assignment.clone()],
        truncated: false,
        next_cursor: None,
    };
    let assignment_preview = AppCaptureAssignmentPreviewV1 {
        schema_version: 1,
        plan_digest: "5".repeat(64),
        intent_id: assignment_intent_id,
        decision: CaptureAssignmentDecision::Assign,
        outcome: CaptureAssignmentPreviewOutcome::ResolutionRequired,
        binding_effect: CaptureAssignmentBindingEffect::Rebound,
        source_disposition: CaptureDisposition::Contradiction,
        source_envelope_id: envelope_id.clone(),
        source_capture_id: capture_id.clone(),
        source_record_state: CaptureDeliveryState::Queued,
        expected_source_generation: 1,
        target_project_id: project_id.clone(),
        expected_library_revision: 1,
        expected_project_revision: 1,
        target_stage: ProjectStage::Writing,
        derived_capture_id: Some(derived_capture_id.clone()),
        child_envelope_id: Some(child_envelope_id.clone()),
        resolution_required: true,
        decided_at_unix: 2,
        explanation: "Rebind the capture to the selected project and preserve source lineage."
            .to_owned(),
        approvals_required: vec!["assignment-write".to_owned()],
    };
    let resolution_decision = AppCaptureResolutionDecisionV1 {
        item_id: resolution_item_id.clone(),
        kind: CaptureResolutionItemKind::SemanticChange,
        disposition: CaptureResolutionDisposition::AcceptCapture,
    };
    let resolution = AppCaptureResolutionViewV1 {
        schema_version: 1,
        receipt_id: resolution_receipt_id,
        assignment_receipt_id: assignment_receipt_id.clone(),
        source_envelope_id: envelope_id.clone(),
        source_capture_id: capture_id.clone(),
        derived_capture_id: derived_capture_id.clone(),
        child_envelope_id: child_envelope_id.clone(),
        target_project_id: project_id.clone(),
        from_project_revision: 1,
        to_project_revision: 2,
        reviewed_at_unix: 3,
        resolved_at_unix: 4,
        decisions: vec![resolution_decision],
    };
    let resolution_page = AppCaptureResolutionPageV1 {
        schema_version: 1,
        snapshot_id: resolution_snapshot_id,
        project_id: project_id.clone(),
        entries: vec![resolution.clone()],
        truncated: false,
        next_cursor: None,
    };
    let resolution_selection = AppCaptureResolutionSelectionV1 {
        item_id: CaptureResolutionItemId::parse(resolution_item_id.clone())
            .map_err(|_| "app-api-contract-resolution-item-id-invalid")?,
        disposition: CaptureResolutionDisposition::AcceptCapture,
    };
    let resolution_preview = AppCaptureResolutionPreviewV1 {
        schema_version: 1,
        plan_digest: "6".repeat(64),
        assignment_receipt_id,
        source_envelope_id: envelope_id.clone(),
        source_capture_id: capture_id,
        derived_capture_id,
        child_envelope_id,
        target_project_id: project_id.clone(),
        expected_library_revision: 1,
        expected_project_revision: 1,
        next_project_revision: 2,
        reviewed_at_unix: 3,
        items: vec![AppCaptureResolutionItemPreviewV1 {
            item_id: resolution_item_id.clone(),
            kind: CaptureResolutionItemKind::SemanticChange,
            counterpart_state: CaptureResolutionCounterpartState::ExactIdentityDivergent,
            allowed_dispositions: vec![
                CaptureResolutionDisposition::AcceptCurrent,
                CaptureResolutionDisposition::AcceptCapture,
                CaptureResolutionDisposition::RejectCapture,
            ],
            unavailable_dispositions: vec![CaptureResolutionDisposition::RetainBoth],
            source_summary: "Use the accepted capture's bounded semantic change.".to_owned(),
            current_summary: Some("Keep the current project statement.".to_owned()),
            explanation: "The same semantic area has divergent reviewed content.".to_owned(),
        }],
        approvals_required: vec!["academic-review".to_owned(), "filesystem-write".to_owned()],
        exact_replay: false,
    };
    let portfolio = AppPortfolioStatusV1 {
        schema_version: 1,
        state: AppPortfolioCatalogState::Current,
        library_revision: 1,
        catalog_id: Some(catalog_id.clone()),
        catalog_generation: Some(1),
        portfolio_id: Some(portfolio_id.clone()),
        contribution_count: 1,
        project_count: 1,
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
        reason_code: "portfolio-current",
        capabilities: AppPortfolioCapabilitiesV1 {
            can_query: true,
            can_reconcile: true,
            can_rebuild: true,
            can_delete_derived_state: true,
        },
    };
    let query_result = AppPortfolioQueryResultV1 {
        schema_version: 1,
        request_id: format!("pqr_{}", "1".repeat(64)),
        query_id: format!("pqy_{}", "2".repeat(64)),
        catalog_id: catalog_id.clone(),
        portfolio_id: portfolio_id.clone(),
        lineage_digest: format!("plg_{}", "3".repeat(64)),
        matched_project_count: 1,
        matched_node_count: 1,
        matched_edge_count: 1,
        matched_lineage_count: 1,
        projects_truncated: false,
        nodes_truncated: false,
        edges_truncated: false,
        lineage_truncated: false,
        projects: vec![AppPortfolioQueryProjectV1 {
            result_id: format!("project:{}", project_id.as_str()),
            project_id: project_id.clone(),
            display_name: "Canonical article project".to_owned(),
            stage: ProjectStage::Writing,
            lifecycle: ProjectLifecycle::Active,
            health: ProjectHealth::Ready,
            semantic_revision: 1,
            projection_id: graph.projection_id.clone(),
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            lineage_count: 1,
        }],
        nodes: vec![AppPortfolioQueryNodeV1 {
            result_id: format!("node:{}:{}", project_id.as_str(), graph.nodes[0].node_id),
            project_id: project_id.clone(),
            projection_id: graph.projection_id.clone(),
            node: graph.nodes[0].clone(),
        }],
        edges: vec![AppPortfolioQueryEdgeV1 {
            result_id: format!("edge:{}:{}", project_id.as_str(), graph.edges[0].edge_id),
            project_id: project_id.clone(),
            projection_id: graph.projection_id.clone(),
            edge: graph.edges[0].clone(),
        }],
        lineage: vec![AppPortfolioLineageV1 {
            lineage_id: format!("lin_{}", "4".repeat(64)),
            kind: PortfolioLineageKind::Delivery,
            project_ids: vec![project_id.clone()],
            related_ids: vec![envelope_id.clone()],
            occurred_at_unix: 4,
            source: Some(CaptureSource::Codex),
            delivery: Some(CaptureDelivery::Connected),
            delivery_state: Some(CaptureDeliveryState::Acknowledged),
            assignment_outcome: Some(CaptureAssignmentOutcome::Assigned),
            from_project_revision: Some(1),
            to_project_revision: Some(2),
        }],
        next_cursor: None,
    };
    let timeline = AppSemanticTimelineResultV1 {
        schema_version: 1,
        request_id: format!("ptr_{}", "5".repeat(64)),
        query_id: format!("pty_{}", "6".repeat(64)),
        catalog_id: catalog_id.clone(),
        portfolio_id: portfolio_id.clone(),
        timeline_digest: format!("ptl_{}", "7".repeat(64)),
        project_id: Some(project_id.clone()),
        view: SemanticTimelineView::Activity,
        matched_event_count: 1,
        truncated: false,
        events: vec![AppSemanticActivityV1 {
            event_id: format!("pte_{}", "8".repeat(64)),
            kind: SemanticActivityKind::DeliveryAcknowledged,
            occurred_at_unix: 4,
            timestamp_source: SemanticActivityTimestampSource::DeliveryTransitionedAt,
            project_ids: vec![project_id],
            related_ids: vec![envelope_id],
            from_project_revision: Some(1),
            to_project_revision: Some(2),
            lifecycle: None,
            source: Some(CaptureSource::Codex),
            delivery: Some(CaptureDelivery::Connected),
            delivery_state: Some(CaptureDeliveryState::Acknowledged),
            delivery_reason: Some(CaptureDeliveryReason::DeliveryAcknowledged),
            delivery_generation: Some(4),
            assignment_outcome: None,
            resolution_item_id: None,
            resolution_item_kind: None,
            resolution_disposition: None,
        }],
        next_cursor: None,
    };
    let doctor = AppPortfolioDoctorV1 {
        schema_version: 1,
        status: PortfolioDoctorStatus::Equivalent,
        library_revision: 1,
        catalog_id: Some(catalog_id.clone()),
        incremental_portfolio_id: Some(portfolio_id.clone()),
        clean_portfolio_id: portfolio_id.clone(),
        byte_equivalent: true,
        contribution_count: 1,
    };
    let maintenance = AppPortfolioMaintenancePreviewV1 {
        schema_version: 1,
        plan_digest: "7".repeat(64),
        operation: PortfolioMaintenanceOperation::Reconcile,
        expected_library_revision: 1,
        expected_catalog_id: Some(catalog_id.clone()),
        expected_catalog_generation: Some(1),
        current_contribution_count: 1,
        derived_state_only: true,
        explanation:
            "Reconcile rebuildable portfolio contributions without changing academic artifacts."
                .to_owned(),
        approvals_required: vec!["derived-state-write".to_owned()],
    };
    let progress = AppContinuityOperationProgressV1 {
        schema_version: 1,
        operation_id: format!("cop_{}", "9".repeat(64)),
        operation: PortfolioMaintenanceOperation::Reconcile,
        phase: AppContinuityOperationPhase::Running,
        completed_units: 1,
        total_units: 2,
        catalog_id: Some(catalog_id.clone()),
        cancellable: true,
        reason_code: "portfolio-reconcile-running",
    };
    let maintenance_result = AppPortfolioMaintenanceResultV1 {
        schema_version: 1,
        operation_id: format!("cop_{}", "9".repeat(64)),
        operation: PortfolioMaintenanceOperation::Reconcile,
        library_revision: 1,
        catalog_id: Some(catalog_id),
        portfolio_id: Some(portfolio_id),
        catalog_changed: false,
        rebuilt_project_count: 0,
        reused_project_count: 1,
        removed_project_count: 0,
        removed_contribution_count: 0,
        derived_state_only: true,
    };

    Ok(vec![
        AppEvent::CaptureDeliveries {
            page: delivery_page,
        },
        AppEvent::CaptureDeliveryInspected {
            delivery: delivery.clone(),
        },
        AppEvent::CaptureDeliveryUpdated { delivery },
        AppEvent::CaptureDeliveryAcknowledgementPreview {
            acknowledgement,
            preview: canonical_continuity_operation(
                "00000000000000000000000000000031",
                "capture-delivery-acknowledgement",
                "Acknowledge capture delivery",
                "Record the exact destination capture and resulting project revision.",
                "3",
                vec!["delivery-acknowledgement"],
            ),
        },
        AppEvent::CaptureAssignments {
            page: assignment_page,
        },
        AppEvent::CaptureAssignmentInspected {
            assignment: assignment.clone(),
        },
        AppEvent::CaptureAssignmentPreview {
            assignment: assignment_preview,
            preview: canonical_continuity_operation(
                "00000000000000000000000000000032",
                "capture-assignment",
                "Assign capture",
                "Bind the source capture to the selected project and preserve exact lineage.",
                "5",
                vec!["assignment-write"],
            ),
        },
        AppEvent::CaptureResolutions {
            page: resolution_page,
        },
        AppEvent::CaptureResolutionInspected {
            resolution: resolution.clone(),
        },
        AppEvent::CaptureResolutionPlan {
            resolution: resolution_preview.clone(),
        },
        AppEvent::CaptureResolutionPreview {
            resolution: resolution_preview,
            selections: vec![resolution_selection],
            preview: canonical_continuity_operation(
                "00000000000000000000000000000033",
                "capture-resolution",
                "Resolve capture items",
                "Apply every reviewed item disposition to the selected project revision.",
                "6",
                vec!["academic-review", "filesystem-write"],
            ),
        },
        AppEvent::PortfolioStatus { portfolio },
        AppEvent::PortfolioQuery {
            result: query_result,
        },
        AppEvent::SemanticTimeline { result: timeline },
        AppEvent::PortfolioDoctor { doctor },
        AppEvent::PortfolioMaintenancePreview {
            maintenance,
            preview: canonical_continuity_operation(
                "00000000000000000000000000000034",
                "portfolio-reconcile",
                "Reconcile portfolio",
                "Rebuild changed derived contributions against the current Research Library.",
                "7",
                vec!["derived-state-write"],
            ),
        },
        AppEvent::ContinuityOperationProgress { progress },
        AppEvent::PortfolioMaintenanceCompleted {
            result: maintenance_result,
        },
    ])
}

fn canonical_continuity_operation(
    token: &str,
    kind: &'static str,
    title: &'static str,
    summary: &str,
    digest_digit: &str,
    approvals_required: Vec<&'static str>,
) -> AppOperationPreview {
    AppOperationPreview {
        token: token.to_owned(),
        kind,
        title,
        summary: summary.to_owned(),
        display_target: None,
        plan_digest_sha256: Some(digest_digit.repeat(64)),
        approvals_required,
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

fn canonical_contract_graph(
    project_id: ProjectId,
) -> Result<
    (
        AcademicGraphSnapshotV1,
        AcademicGraphQueryResultV1,
        AcademicGraphPathResultV1,
    ),
    &'static str,
> {
    let project_node = AcademicGraphNodeV1::new(
        &project_id,
        AcademicGraphNodeType::Project,
        AcademicGraphIdentityScope::Project,
        project_id.as_str(),
        "Canonical article project",
        vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
        "context/project_manifest.json",
        "project",
    )
    .map_err(|_| "app-api-contract-graph-node-invalid")?;
    let claim_node = AcademicGraphNodeV1::new(
        &project_id,
        AcademicGraphNodeType::Claim,
        AcademicGraphIdentityScope::Project,
        "CLM-001",
        "Portable article state preserves evidence provenance",
        vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
        "manuscript/claims_evidence_map.md",
        "CLM-001",
    )
    .map_err(|_| "app-api-contract-graph-node-invalid")?;
    let edge = AcademicGraphEdgeV1::new(
        &project_id,
        project_node.node_id.clone(),
        AcademicGraphRelation::Contains,
        claim_node.node_id.clone(),
        vec![AcademicGraphLayer::Combined],
        "The canonical article project contains its manuscript claims.",
        "manuscript/claims_evidence_map.md",
        "CLM-001",
        "The fixture records contract shape rather than empirical support.",
        AcademicInferenceStrength::DirectEvidence,
        AcademicGraphConfidence::High,
        AcademicGraphEdgeStatus::Observed,
        None,
    )
    .map_err(|_| "app-api-contract-graph-edge-invalid")?;
    let projection_id = format!("grp_{}", "a".repeat(64));
    let nodes = vec![project_node, claim_node];
    let edges = vec![edge];
    let snapshot = AcademicGraphSnapshotV1 {
        schema_version: 1,
        document_kind: "qiongli-academic-graph".to_owned(),
        projection_id: projection_id.clone(),
        projection_digest: "b".repeat(64),
        project_id: project_id.clone(),
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        project_lifecycle: ProjectLifecycle::Active,
        project_manifest_digest: "c".repeat(64),
        project_semantic_digest: "d".repeat(64),
        graph_source_digest: "e".repeat(64),
        source_count: 1,
        present_source_count: 1,
        node_count: nodes.len(),
        edge_count: edges.len(),
        diagnostic_count: 0,
        sources: vec![AcademicGraphSourceRefV1 {
            source_kind: AcademicGraphSourceKind::ProjectManifest,
            artifact_path: "context/project_manifest.json".to_owned(),
            present: true,
            content_digest: Some("c".repeat(64)),
            size_bytes: 512,
        }],
        nodes: nodes.clone(),
        edges: edges.clone(),
        diagnostics: Vec::<AcademicGraphDiagnosticV1>::new(),
    };
    let index_id = format!("gix_{}", "f".repeat(64));
    let path = AcademicGraphPathResultV1 {
        schema_version: 1,
        document_kind: "qiongli-academic-graph-explanatory-path".to_owned(),
        index_id: index_id.clone(),
        projection_id: projection_id.clone(),
        project_id: project_id.clone(),
        project_revision: 1,
        source_node_id: nodes[0].node_id.clone(),
        target_node_id: nodes[1].node_id.clone(),
        max_hops: 6,
        status: AcademicGraphPathStatus::Found,
        hop_count: 1,
        nodes: nodes.clone(),
        edges: edges.clone(),
        steps: vec![AcademicGraphPathStepV1 {
            sequence: 1,
            from_node_id: nodes[0].node_id.clone(),
            edge_id: edges[0].edge_id.clone(),
            to_node_id: nodes[1].node_id.clone(),
            traversal: AcademicGraphPathTraversal::Forward,
        }],
    };
    let result = AcademicGraphQueryResultV1 {
        schema_version: 1,
        document_kind: "qiongli-academic-graph-query-result".to_owned(),
        index_id,
        projection_id,
        project_id,
        project_revision: 1,
        matched_node_count: nodes.len(),
        matched_edge_count: edges.len(),
        nodes_truncated: false,
        edges_truncated: false,
        nodes,
        edges,
    };
    Ok((snapshot, result, path))
}

fn canonical_contract_capture(project_id: ProjectId) -> Result<ResearchCaptureV1, &'static str> {
    let binding = ProjectBindingV1::new(
        project_id,
        1,
        ProjectStage::Writing,
        "Validate the canonical desktop contract",
        CapturePolicy::ReviewRequired,
    )
    .map_err(|_| "app-api-contract-binding-invalid")?;
    ResearchCaptureDraftV1 {
        binding,
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Connected,
        captured_at_unix: 1,
        summary: "Canonical bounded research capture".to_owned(),
        changes: Vec::new(),
        decisions: Vec::new(),
        evidence: Vec::new(),
        contradictions: Vec::new(),
        next_actions: Vec::new(),
    }
    .into_capture()
    .map_err(|_| "app-api-contract-capture-invalid")
}

fn canonical_contract_inbox(project_id: ProjectId) -> CaptureInboxSnapshotV1 {
    CaptureInboxSnapshotV1 {
        schema_version: CAPTURE_INBOX_SCHEMA_VERSION,
        project_id,
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        pending_review_count: 0,
        stale_count: 0,
        conflicted_count: 0,
        applied_count: 0,
        entries: Vec::new(),
    }
}

fn canonical_contract_coverage(project_id: ProjectId) -> CaptureCoverageSnapshotV1 {
    let sources = [
        CaptureSource::Codex,
        CaptureSource::ClaudeCode,
        CaptureSource::ChatGpt,
        CaptureSource::Cli,
        CaptureSource::Manual,
        CaptureSource::Repository,
        CaptureSource::PortableFile,
    ]
    .into_iter()
    .map(|source| CaptureSourceCoverageV1 {
        source,
        state: CaptureCoverageState::Unknown,
        delivery: CaptureCoverageDelivery::Unknown,
        capture_count: 0,
        pending_review_count: 0,
        current_count: 0,
        stale_count: 0,
        conflicted_count: 0,
        unbound_count: 0,
        latest_capture_id: None,
        last_captured_at_unix: None,
    })
    .collect();
    CaptureCoverageSnapshotV1 {
        schema_version: CAPTURE_COVERAGE_SCHEMA_VERSION,
        project_id,
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        capture_count: 0,
        connected_count: 0,
        repository_backed_count: 0,
        portable_count: 0,
        manual_count: 0,
        pending_review_count: 0,
        current_count: 0,
        stale_count: 0,
        conflicted_count: 0,
        unbound_count: 0,
        unknown_source_count: 7,
        sources,
    }
}

fn canonical_contract_artifact_changes(project_id: ProjectId) -> ArtifactChangeSnapshotV1 {
    let artifacts = [
        (
            RegisteredArtifact::ResearchState,
            "context/research_state.md",
        ),
        (RegisteredArtifact::DecisionLog, "context/decision_log.md"),
        (RegisteredArtifact::StageHandoff, "context/stage_handoff.md"),
        (
            RegisteredArtifact::BoundaryReview,
            "context/boundary_review.md",
        ),
        (RegisteredArtifact::IdeaFunnel, "context/idea_funnel.md"),
        (
            RegisteredArtifact::LiteratureMap,
            "literature/literature_map.md",
        ),
        (
            RegisteredArtifact::ClaimEvidenceLedger,
            "evidence/claim-evidence-ledger.csv",
        ),
        (
            RegisteredArtifact::ManuscriptClaimMap,
            "manuscript/claims_evidence_map.md",
        ),
    ]
    .into_iter()
    .map(
        |(artifact, relative_path)| RegisteredArtifactObservationV1 {
            artifact,
            relative_path: relative_path.to_owned(),
            present: false,
        },
    )
    .collect::<Vec<_>>();
    ArtifactChangeSnapshotV1 {
        schema_version: ARTIFACT_CHANGE_SCHEMA_VERSION,
        project_id,
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        state: ArtifactChangeState::Current,
        registered_artifact_count: artifacts.len(),
        present_artifact_count: 0,
        change_count: 0,
        unattributed_count: 0,
        changes: Vec::new(),
        artifacts,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppOperationPreview {
    token: String,
    kind: &'static str,
    title: &'static str,
    summary: String,
    display_target: Option<String>,
    plan_digest_sha256: Option<String>,
    approvals_required: Vec<&'static str>,
    can_confirm: bool,
    blocked_reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    migration: Option<AppProjectMigrationPreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    migration_rollback: Option<AppProjectMigrationRollbackPreview>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppProjectMigrationPreview {
    mode: &'static str,
    copied_file_count: usize,
    copied_bytes: u64,
    excluded_entry_count: usize,
    source_retained: bool,
    copies_files: bool,
    graph_rebuild_passes: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppProjectMigrationRollbackPreview {
    registration_state: &'static str,
    marker_state: &'static str,
    reconciliation: ProjectMigrationReconciliationV1,
    source_retained: bool,
    destination_removal: String,
    can_rollback: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppProjectMigrationQualification {
    project_id: ProjectId,
    status: &'static str,
    projection_id: Option<String>,
    index_id: Option<String>,
    deterministic_rebuild: bool,
    reason_code: Option<&'static str>,
}

impl AppProjectMigrationQualification {
    pub(crate) fn verified(project_id: ProjectId, projection_id: String, index_id: String) -> Self {
        Self {
            project_id,
            status: "verified",
            projection_id: Some(projection_id),
            index_id: Some(index_id),
            deterministic_rebuild: true,
            reason_code: None,
        }
    }

    pub(crate) const fn rebuild_required(project_id: ProjectId, reason_code: &'static str) -> Self {
        Self {
            project_id,
            status: "rebuild-required",
            projection_id: None,
            index_id: None,
            deterministic_rebuild: false,
            reason_code: Some(reason_code),
        }
    }

    #[cfg(test)]
    pub(crate) const fn deterministic_rebuild(&self) -> bool {
        self.deterministic_rebuild
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppResearchCaptureV1 {
    schema_version: u32,
    capture_id: String,
    binding: AppCaptureBindingV1,
    source: CaptureSource,
    delivery: CaptureDelivery,
    captured_at_unix: u64,
    summary: String,
    changes: Vec<AppSemanticChangeV1>,
    decisions: Vec<AppDecisionCandidateV1>,
    evidence: Vec<AppEvidenceReferenceV1>,
    contradictions: Vec<AppContradictionV1>,
    next_actions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppCaptureBindingV1 {
    schema_version: u32,
    project_id: String,
    base_revision: u64,
    stage: ProjectStage,
    task: String,
    capture_policy: CapturePolicy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppSemanticChangeV1 {
    area: CaptureArea,
    summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppDecisionCandidateV1 {
    relation: DecisionRelation,
    statement: String,
    rationale: String,
    target: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppEvidenceReferenceV1 {
    locator_kind: EvidenceLocatorKind,
    locator: String,
    relevance: String,
    limitation: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppContradictionV1 {
    statement: String,
    conflicts_with: String,
    consequence: String,
}

impl AppSnapshotV1 {
    pub(crate) fn from_desktop(
        snapshot: DesktopSnapshotV1,
        research_library: ResearchLibrarySnapshotV1,
        project_skills: Vec<AppProjectSkillsTargetView>,
    ) -> Result<Self, &'static str> {
        snapshot.validate().map_err(|error| error.code())?;
        let can_apply = snapshot.capabilities.apply;
        let project_available =
            research_library.health != qiongli_project::LibraryHealth::InspectionBlocked;
        let trust = match (snapshot.product.trust, can_apply) {
            (ProductTrustView::PackagedProductControl, _) => AppTrustView {
                mode: "packaged-product",
                label: "Verified packaged product",
                can_apply: true,
                reason_code: "verified-packaged-product-control",
            },
            (ProductTrustView::SourceBuild, true) => AppTrustView {
                mode: "local-installable",
                label: "Local candidate authority",
                can_apply: true,
                reason_code: "local-authority-session",
            },
            (ProductTrustView::SourceBuild, false) => AppTrustView {
                mode: "source-read-only",
                label: "Source build — client changes inspect only",
                can_apply: false,
                reason_code: "source-build-read-only",
            },
        };
        let managed_skills_unavailable =
            snapshot.content.managed_skills_status == StatusCode::Unavailable;
        let mut managed_skills = snapshot
            .content
            .managed_skills
            .into_iter()
            .map(|managed| {
                let (preset, symbolic_path) =
                    if managed.preset == SkillsDestinationPreset::CurrentProject {
                        (
                            SkillsDestinationPreset::CustomFolder.id(),
                            SkillsDestinationPreset::CustomFolder.symbolic_path(),
                        )
                    } else {
                        (managed.preset.id(), managed.preset.symbolic_path())
                    };
                (
                    managed.target_id.clone(),
                    AppManagedSkillsDestinationView {
                        target_id: managed.target_id,
                        preset,
                        symbolic_path,
                        state: managed.state.code(),
                        status: managed.status.code(),
                        profile: managed.profile.map(ProfileKind::id),
                        product_version: managed.product_version,
                        project_id: None,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        for project_target in project_skills {
            let managed = project_target.destination;
            managed_skills.insert(
                managed.target_id.clone(),
                AppManagedSkillsDestinationView {
                    target_id: managed.target_id,
                    preset: SkillsDestinationPreset::CurrentProject.id(),
                    symbolic_path: SkillsDestinationPreset::CurrentProject.symbolic_path(),
                    state: managed.state.code(),
                    status: managed.status.code(),
                    profile: managed.profile.map(ProfileKind::id),
                    product_version: managed.product_version,
                    project_id: Some(project_target.project_id),
                },
            );
        }
        let managed_skills = managed_skills.into_values().collect::<Vec<_>>();
        let managed_skills_status = if managed_skills_unavailable && managed_skills.is_empty() {
            StatusCode::Unavailable.code()
        } else if managed_skills
            .iter()
            .any(|entry| entry.state == ManagedSkillsStateView::Drifted.code())
        {
            StatusCode::Drifted.code()
        } else if managed_skills
            .iter()
            .any(|entry| entry.state == ManagedSkillsStateView::Unmanaged.code())
        {
            StatusCode::Conflict.code()
        } else if managed_skills
            .iter()
            .any(|entry| entry.state == ManagedSkillsStateView::UpdateAvailable.code())
        {
            StatusCode::Attention.code()
        } else {
            StatusCode::Ready.code()
        };
        Ok(Self {
            schema_version: APP_API_SCHEMA_VERSION,
            product: AppProductView {
                version: snapshot.product.version,
                build: snapshot.product.build,
                operating_system: snapshot.product.operating_system.label().to_owned(),
                architecture: snapshot.product.architecture.label().to_owned(),
                trust,
            },
            content: AppContentView {
                status: snapshot.content.status.code(),
                pack_id: snapshot.content.pack_id,
                content_version: snapshot.content.content_version,
                entry_count: snapshot.content.entry_count,
                profiles: snapshot
                    .content
                    .profiles
                    .into_iter()
                    .map(|profile| AppProfileView {
                        id: profile.profile.id(),
                        label: profile_label(profile.profile),
                        description: profile.profile.description(),
                        included_resource_kinds: profile.included_resource_kinds,
                    })
                    .collect(),
                managed_skills: AppManagedSkillsInventoryView {
                    status: managed_skills_status,
                    destinations: managed_skills,
                },
            },
            mcp: AppMcpView {
                status: snapshot.mcp.status.code(),
                profile: snapshot.mcp.profile.id(),
                public_tool_count: snapshot.mcp.public_tool_count,
            },
            cli: AppCliView {
                status: snapshot.cli.status.code(),
                state: snapshot.cli.state.code(),
                installed_version: snapshot.cli.installed_version,
                available_version: snapshot.cli.available_version,
                symbolic_target: snapshot.cli.symbolic_target,
                path_status: snapshot.cli.path_status.code(),
                path_state: snapshot.cli.path_state.code(),
                reason_code: snapshot.cli.reason_code,
                can_install: snapshot.cli.can_install,
                can_test: snapshot.cli.can_test,
            },
            zotero: AppZoteroIntegrationView {
                status: snapshot.zotero.status.code(),
                state: snapshot.zotero.state.code(),
                observation: snapshot.zotero.observation.code(),
                zotero_version: snapshot.zotero.zotero_version,
                connector_available: snapshot.zotero.connector_available,
                companion_available: snapshot.zotero.companion_available,
                companion_version: snapshot.zotero.companion_version,
                available_companion_version: snapshot.zotero.available_companion_version,
                available_companion_sha256: snapshot.zotero.available_companion_sha256,
                available_companion_size_bytes: snapshot.zotero.available_companion_size_bytes,
                endpoint_version: snapshot.zotero.endpoint_version,
                supported_endpoint_version: snapshot.zotero.supported_endpoint_version,
                supported_zotero_min_version: snapshot.zotero.supported_zotero_min_version,
                supported_zotero_max_version: snapshot.zotero.supported_zotero_max_version,
                installation_prepared: snapshot.zotero.installation_prepared,
                fallback_import_available: snapshot.zotero.fallback_import_available,
                fallback_formats: snapshot.zotero.fallback_formats,
                reason_code: snapshot.zotero.reason_code,
                can_prepare_install: snapshot.zotero.can_prepare_install,
                can_reveal: snapshot.zotero.can_reveal,
                can_open_zotero: snapshot.zotero.can_open_zotero,
                can_verify: snapshot.zotero.can_verify,
            },
            configuration: AppConfigurationView {
                status: snapshot.config.status.code(),
                revision: snapshot.config.revision,
                secret_store: snapshot.config.secret_store.code(),
                providers: snapshot
                    .config
                    .providers
                    .into_iter()
                    .map(|provider| AppProviderView {
                        provider: provider.provider.id(),
                        enabled: provider.enabled,
                        readiness: match provider.readiness {
                            ProviderReadinessView::Disabled => "disabled",
                            ProviderReadinessView::Ready => "ready",
                            ProviderReadinessView::NeedsSecret => "needs-secret",
                            ProviderReadinessView::NeedsPublicSetting => "needs-public-setting",
                            ProviderReadinessView::Unavailable => "unavailable",
                        },
                        configuration_fields: provider
                            .provider
                            .configuration_fields()
                            .iter()
                            .map(|field| AppProviderConfigurationFieldView {
                                field: field.id(),
                                configured: match field {
                                    ProviderConfigurationField::ApiKey => {
                                        provider.secret_reference_present
                                    }
                                    ProviderConfigurationField::Email => {
                                        provider.public_setting_present
                                    }
                                },
                            })
                            .collect(),
                    })
                    .collect(),
                legacy_credential: AppLegacyCredentialView {
                    reference_present: snapshot.config.openai_backend.secret_reference_present,
                    cleanup_available: snapshot.config.openai_backend.secret_reference_present,
                },
                cleanup_required: snapshot.config.cleanup_required,
            },
            update: app_update_view(snapshot.update),
            research_library,
            legacy_migration: AppLegacyMigrationStatusView {
                state: snapshot.legacy_migration.state.code(),
                next_action: snapshot.legacy_migration.next_action.code(),
                migration_id: snapshot.legacy_migration.migration_id,
                detected_items: snapshot.legacy_migration.detected_items,
                eligible_items: snapshot.legacy_migration.eligible_items,
                review_items: snapshot.legacy_migration.review_items,
                reason_code: snapshot.legacy_migration.reason_code,
                provider_conflicts: snapshot
                    .legacy_migration
                    .provider_conflicts
                    .into_iter()
                    .map(|conflict| AppLegacyProviderConflictView {
                        provider: conflict.provider.code(),
                        differing_fields: conflict.differing_fields,
                        legacy_secret_present: conflict.legacy_secret_present,
                        current_secret_reference_present: conflict.current_secret_reference_present,
                        default_strategy: "keep-v2",
                    })
                    .collect(),
            },
            integrations: snapshot
                .integrations
                .into_iter()
                .map(app_integration_view)
                .collect(),
            capabilities: AppCapabilityView {
                refresh: snapshot.capabilities.refresh,
                skills_materialize: snapshot.capabilities.skills_materialize,
                integration_discovery: snapshot.capabilities.integration_discovery,
                integration_preview: snapshot.capabilities.integration_preview,
                project_library: project_available,
                project_mutation: project_available,
                capture_inbox: project_available,
                capture_mutation: project_available,
                capture_delivery: project_available,
                capture_resolution: project_available,
                academic_graph: project_available,
                portfolio: project_available,
                timeline: project_available,
                orchestration_inspect: project_available,
                orchestration_control: project_available,
                legacy_credential_cleanup: snapshot.config.openai_backend.secret_reference_present,
                apply: snapshot.capabilities.apply,
            },
        })
    }
}

impl AppIntent {
    pub(crate) fn into_desktop(self) -> Result<DesktopIntent, &'static str> {
        self.validate()?;
        Ok(match self {
            Self::Refresh => DesktopIntent::Refresh,
            Self::RefreshResearchLibrary
            | Self::SelectProjectDirectory
            | Self::SelectProjectCreateDestination { .. }
            | Self::PreviewProjectCreate { .. }
            | Self::PreviewProjectRegister { .. }
            | Self::OpenProject { .. }
            | Self::SelectProjectExportDestination { .. }
            | Self::PreviewProjectExport { .. }
            | Self::SelectProjectImportLocations { .. }
            | Self::PreviewProjectImport { .. }
            | Self::SelectProjectMigrationLocations { .. }
            | Self::PreviewProjectMigration { .. }
            | Self::SelectProjectMigrationRecoveryLocations
            | Self::PreviewProjectMigrationRecovery { .. }
            | Self::SelectProjectMigrationRollbackLocations
            | Self::PreviewProjectMigrationRollback { .. }
            | Self::PreviewProjectRepairManifest { .. }
            | Self::PreviewProjectArchive { .. }
            | Self::PreviewProjectRestore { .. }
            | Self::PreviewProjectRefresh { .. }
            | Self::PreviewProjectUnregister { .. }
            | Self::LoadCaptureInbox { .. }
            | Self::LoadCaptureCoverage { .. }
            | Self::LoadArtifactChanges { .. }
            | Self::LoadAcademicGraph { .. }
            | Self::LoadAcademicGraphPortfolio
            | Self::QueryAcademicGraph { .. }
            | Self::QueryAcademicGraphPath { .. }
            | Self::OpenAcademicGraphArtifact { .. }
            | Self::ReadProjectArtifact { .. }
            | Self::ReadCapture { .. }
            | Self::SelectCaptureFile { .. }
            | Self::PreviewCaptureIntake { .. }
            | Self::PreviewCaptureConsolidation { .. }
            | Self::LoadCaptureDeliveries { .. }
            | Self::InspectCaptureDelivery { .. }
            | Self::RetryCaptureDelivery { .. }
            | Self::CancelCaptureDelivery { .. }
            | Self::PreviewCaptureDeliveryAcknowledgement { .. }
            | Self::LoadCaptureAssignments { .. }
            | Self::InspectCaptureAssignment { .. }
            | Self::PreviewCaptureAssignment { .. }
            | Self::LoadCaptureResolutions { .. }
            | Self::InspectCaptureResolution { .. }
            | Self::PreviewCaptureResolution { .. }
            | Self::LoadPortfolioStatus
            | Self::QueryPortfolio { .. }
            | Self::LoadSemanticTimeline { .. }
            | Self::LoadPortfolioDoctor
            | Self::PreviewPortfolioMaintenance { .. }
            | Self::PollContinuityOperation { .. }
            | Self::CancelContinuityOperation { .. } => {
                return Err("app-project-intent-not-intercepted");
            }
            Self::LoadContentCustomization { .. }
            | Self::PreviewWorkflowResourceReplace { .. }
            | Self::PreviewWorkflowResourceReset { .. }
            | Self::PreviewProjectGuidance { .. } => {
                return Err("app-content-customization-intent-not-intercepted");
            }
            Self::PreviewProjectSkillsMaterialization { .. } => {
                return Err("project-skills-requires-project-service");
            }
            Self::LoadOrchestration { .. } | Self::ControlOrchestration { .. } => {
                return Err("host-handoff-not-ready");
            }
            Self::RefreshIntegrationDiscovery => DesktopIntent::RefreshIntegrationDiscovery,
            Self::RunFullMcpSelfTest => DesktopIntent::RunFullMcpSelfTest,
            Self::PollFullMcpSelfTest => DesktopIntent::PollFullMcpSelfTest,
            Self::CancelFullMcpSelfTest => DesktopIntent::CancelFullMcpSelfTest,
            Self::RefreshZoteroIntegration => DesktopIntent::RefreshZoteroIntegration,
            Self::PreviewZoteroCompanionStage => DesktopIntent::PreviewZoteroCompanionStage,
            Self::RevealZoteroCompanion => DesktopIntent::RevealZoteroCompanion,
            Self::OpenZotero => DesktopIntent::OpenZotero,
            Self::VerifyZoteroIntegration => DesktopIntent::VerifyZoteroIntegration,
            Self::PrepareLegacyMigration {
                provider_resolutions,
            } => {
                if provider_resolutions.len() > 5 {
                    return Err("legacy-provider-resolution-count-invalid");
                }
                DesktopIntent::PrepareLegacyMigration {
                    provider_resolutions: provider_resolutions
                        .into_iter()
                        .map(AppLegacyProviderResolution::into_desktop)
                        .collect(),
                }
            }
            Self::PreviewLegacyMigrationNext => DesktopIntent::PreviewLegacyMigrationNext,
            Self::SelectUpdateStream { stream } => DesktopIntent::SelectUpdateStream {
                stream: stream.into_desktop(),
            },
            Self::CheckForUpdates => DesktopIntent::CheckForUpdates,
            Self::PrepareUpdate => DesktopIntent::PrepareUpdate,
            Self::PollUpdate => DesktopIntent::PollUpdate,
            Self::CancelUpdate => DesktopIntent::CancelUpdate,
            Self::PreviewUpdateInstall => DesktopIntent::PreviewUpdateInstall,
            Self::PreviewCliInstall => DesktopIntent::PreviewCliInstall,
            Self::PreviewCliRemove => DesktopIntent::PreviewCliRemove,
            Self::PreviewCliPathConfigure => DesktopIntent::PreviewCliPathConfigure,
            Self::TestCliCommand => DesktopIntent::TestCliCommand,
            Self::PreviewProviderSettings {
                expected_revision,
                providers_enabled,
                public_setting_changes,
            } => {
                if public_setting_changes.len() > 2 {
                    return Err("provider-public-setting-change-invalid");
                }
                let mut openalex_email = PublicSettingChange::Keep;
                let mut crossref_email = PublicSettingChange::Keep;
                let mut openalex_changed = false;
                let mut crossref_changed = false;
                for change in public_setting_changes {
                    let (provider, change) = change.into_desktop()?;
                    match provider {
                        ProviderKind::OpenAlex if !openalex_changed => {
                            openalex_email = change;
                            openalex_changed = true;
                        }
                        ProviderKind::Crossref if !crossref_changed => {
                            crossref_email = change;
                            crossref_changed = true;
                        }
                        ProviderKind::OpenAlex | ProviderKind::Crossref => {
                            return Err("provider-public-setting-change-duplicate");
                        }
                        ProviderKind::SemanticScholar
                        | ProviderKind::PubMed
                        | ProviderKind::Arxiv => {
                            return Err("provider-public-setting-unsupported");
                        }
                    }
                }
                DesktopIntent::PreviewProviderSettingsPatch(ProviderSettingsPatch {
                    expected_revision,
                    providers_enabled: providers_enabled.into_desktop(),
                    openalex_email,
                    crossref_email,
                })
            }
            Self::PreviewProviderSecretChange {
                provider,
                change,
                value,
            } => {
                let provider = provider.into_desktop();
                if !matches!(
                    provider,
                    ProviderKind::OpenAlex | ProviderKind::SemanticScholar | ProviderKind::PubMed
                ) {
                    return Err("provider-secret-unsupported");
                }
                let change = match (change, value) {
                    (AppProviderSecretChange::Replace, Some(value))
                        if !value.is_empty() && value.len() <= 4096 =>
                    {
                        ProviderSecretChange::Replace(PrivateText::new(value))
                    }
                    (AppProviderSecretChange::Remove, None) => ProviderSecretChange::Remove,
                    _ => return Err("provider-secret-change-invalid"),
                };
                DesktopIntent::PreviewProviderSecretChange { provider, change }
            }
            Self::TestLiteratureProvider { provider } => DesktopIntent::TestLiteratureProvider {
                provider: provider.into_desktop(),
            },
            Self::PreviewRemoveAgentBackendCredential => {
                DesktopIntent::PreviewAgentBackendSecretChange {
                    change: AgentBackendSecretChange::Remove,
                }
            }
            Self::PreviewInstallRecommended => DesktopIntent::PreviewInstallRecommended,
            Self::PreviewInstallSelected { selection } => DesktopIntent::PreviewInstallSelected {
                selection: selection.into_desktop(),
            },
            Self::VerifyIntegrations { selection } => DesktopIntent::VerifyIntegrations {
                selection: selection.into_desktop(),
            },
            Self::PreviewReconcileIntegrations { selection } => {
                DesktopIntent::PreviewReconcileIntegrations {
                    selection: selection.into_desktop(),
                }
            }
            Self::PreviewRemoveIntegrations { selection } => {
                DesktopIntent::PreviewRemoveIntegrations {
                    selection: selection.into_desktop(),
                }
            }
            Self::SelectSkillsDestination => DesktopIntent::SelectSkillsDestination,
            Self::PreviewSkillsPresetMaterialization { profile, preset } => {
                DesktopIntent::PreviewSkillsPresetMaterialization {
                    profile: profile.into_desktop(),
                    preset: preset.into_desktop(),
                }
            }
            Self::VerifySkillsPreset { preset } => DesktopIntent::VerifySkillsPreset {
                preset: preset.into_desktop(),
            },
            Self::PreviewSkillsPresetRemoval { preset } => {
                DesktopIntent::PreviewSkillsPresetRemoval {
                    preset: preset.into_desktop(),
                }
            }
            Self::VerifyManagedSkillsTarget { target_id } => {
                DesktopIntent::VerifyManagedSkillsTarget { target_id }
            }
            Self::PreviewUpdateManagedSkillsTarget { target_id } => {
                DesktopIntent::PreviewManagedSkillsTargetUpdate { target_id }
            }
            Self::PreviewRemoveManagedSkillsTarget { target_id } => {
                DesktopIntent::PreviewManagedSkillsTargetRemoval { target_id }
            }
            Self::PreviewDetachManagedSkillsTarget { target_id } => {
                DesktopIntent::PreviewManagedSkillsTargetDetach { target_id }
            }
            Self::ConfirmOperation { token } => DesktopIntent::ConfirmOperation {
                token: parse_operation_token(&token)?,
            },
            Self::CancelOperation { token } => DesktopIntent::CancelOperation {
                token: parse_operation_token(&token)?,
            },
        })
    }
}

pub(crate) fn app_portable_operation_preview(
    token: String,
    preview: &PortableProjectPreviewV1,
) -> AppOperationPreview {
    let (kind, title, summary) = match preview.operation {
        PortableProjectOperation::Export => (
            "project-export",
            "Export portable article project",
            "Copy portable academic artifacts into a verified directory package. Private paths, recognizable credential files, client state, sessions, chats, and transcripts are excluded.",
        ),
        PortableProjectOperation::Import => (
            "project-import",
            "Import portable article project",
            "Verify every packaged artifact, create a new local project directory, and register the preserved project identity in this Research Library.",
        ),
    };
    AppOperationPreview {
        token,
        kind,
        title,
        summary: summary.to_owned(),
        display_target: Some(preview.destination_label.clone()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_project_operation_preview(
    token: String,
    preview: &ProjectMutationPreviewV1,
) -> AppOperationPreview {
    let (kind, title, summary) = match preview.operation {
        ProjectMutationKind::Register => (
            "project-register",
            "Register article project",
            "Add this portable article project to the private local Research Library. Existing academic artifacts will not be rewritten.",
        ),
        ProjectMutationKind::Archive => (
            "project-archive",
            "Archive article project",
            "Archive this project in the Research Library without deleting its directory or academic artifacts.",
        ),
        ProjectMutationKind::Restore => (
            "project-restore",
            "Restore article project",
            "Return this archived project to the active Research Library.",
        ),
        ProjectMutationKind::Refresh => (
            "project-refresh",
            "Refresh academic revision",
            "Inspect the canonical article artifacts and advance the semantic revision only when their academic content changed.",
        ),
        ProjectMutationKind::Unregister => (
            "project-unregister",
            "Unregister article project",
            "Remove only the private Research Library entry. The portable project manifest and all academic artifacts remain untouched.",
        ),
        ProjectMutationKind::Create => (
            "project-create",
            "Create article project",
            "Create and register a portable article project after explicit confirmation.",
        ),
        ProjectMutationKind::RepairManifest => (
            "project-repair-manifest",
            "Repair portable project manifest",
            "Rebuild the missing portable manifest from the private Research Library entry without changing academic artifacts.",
        ),
    };
    AppOperationPreview {
        token,
        kind,
        title,
        summary: summary.to_owned(),
        display_target: Some(preview.root_label.clone()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_project_guidance_operation_preview(
    token: String,
    plan_digest: String,
) -> AppOperationPreview {
    AppOperationPreview {
        token,
        kind: "project-guidance",
        title: "Project guidance preview",
        summary: "Write user preferences as advisory project guidance without changing the verified Skill or Plugin files.".to_owned(),
        display_target: Some("<project>/.qiongli/local_guidance.md".to_owned()),
        plan_digest_sha256: Some(plan_digest),
        approvals_required: vec!["filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_workflow_variant_operation_preview(
    token: String,
    plan_digest: String,
    path: &str,
    reset: bool,
) -> AppOperationPreview {
    AppOperationPreview {
        token,
        kind: if reset {
            "workflow-variant-reset"
        } else {
            "workflow-variant-update"
        },
        title: if reset {
            "Reset workflow resource"
        } else {
            "Update workflow resource"
        },
        summary: if reset {
            "Reset this editable workflow resource to its verified canonical content."
        } else {
            "Save this Markdown resource as a receipt-bound local workflow variant without changing canonical packaged content."
        }
        .to_owned(),
        display_target: Some(format!("<qiongli-state>/workflow-variant/{path}")),
        plan_digest_sha256: Some(plan_digest),
        approvals_required: vec!["filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_project_migration_operation_preview(
    token: String,
    preview: &ProjectMigrationPreviewV1,
) -> AppOperationPreview {
    AppOperationPreview {
        token,
        kind: "project-migration",
        title: "Migrate Qiongli 1.x article project",
        summary: format!(
            "Copy {} verified academic file(s) ({} bytes) into a new Qiongli 2 project, exclude {} legacy/private entry or entries, retain the source unchanged, register the destination, and verify two deterministic graph-index rebuilds.",
            preview.copied_file_count, preview.copied_bytes, preview.excluded_entry_count
        ),
        display_target: Some(preview.destination_label.clone()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: Some(AppProjectMigrationPreview {
            mode: "copy",
            copied_file_count: preview.copied_file_count,
            copied_bytes: preview.copied_bytes,
            excluded_entry_count: preview.excluded_entry_count,
            source_retained: preview.source_retained,
            copies_files: true,
            graph_rebuild_passes: 2,
        }),
        migration_rollback: None,
    }
}

pub(crate) fn app_project_migration_recovery_operation_preview(
    token: String,
    preview: &ProjectMigrationRecoveryPreviewV1,
) -> AppOperationPreview {
    AppOperationPreview {
        token,
        kind: "project-migration-recovery",
        title: "Resume interrupted project migration",
        summary: format!(
            "Verify the unchanged source and the already committed Qiongli 2 copy ({} file(s), {} bytes, {} excluded entry or entries), complete Research Library registration without copying again, and verify two deterministic graph-index rebuilds.",
            preview.copied_file_count, preview.copied_bytes, preview.excluded_entry_count
        ),
        display_target: Some(preview.destination_label.clone()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: Some(AppProjectMigrationPreview {
            mode: "recovery",
            copied_file_count: preview.copied_file_count,
            copied_bytes: preview.copied_bytes,
            excluded_entry_count: preview.excluded_entry_count,
            source_retained: preview.source_retained,
            copies_files: false,
            graph_rebuild_passes: 2,
        }),
        migration_rollback: None,
    }
}

pub(crate) fn app_project_migration_rollback_operation_preview(
    token: String,
    preview: &ProjectMigrationRollbackPreviewV1,
) -> AppOperationPreview {
    let blocked_reason = preview
        .blocked_reason
        .as_deref()
        .map(project_migration_rollback_blocked_reason);
    AppOperationPreview {
        token,
        kind: "project-migration-rollback",
        title: "Roll back migrated Qiongli 2 project",
        summary: format!(
            "Reconcile {} matching artifact(s), {} changed or missing artifact(s), and {} continuity gap(s). Unregister and remove only the exact unchanged migration-owned Qiongli 2 destination while retaining the Qiongli 1.x source.",
            preview.reconciliation.matched_artifact_count,
            preview.reconciliation.drifted_artifact_count,
            preview.reconciliation.continuity_gap_count
        ),
        display_target: Some(preview.destination_label.clone()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: if preview.can_rollback {
            vec!["filesystem-write"]
        } else {
            Vec::new()
        },
        can_confirm: preview.can_rollback,
        blocked_reason,
        migration: None,
        migration_rollback: Some(AppProjectMigrationRollbackPreview {
            registration_state: match preview.registration_state {
                qiongli_project::ProjectMigrationRegistrationState::Registered => "registered",
                qiongli_project::ProjectMigrationRegistrationState::Unregistered => "unregistered",
            },
            marker_state: match preview.marker_state {
                qiongli_project::ProjectMigrationMarkerState::Ready => "ready",
                qiongli_project::ProjectMigrationMarkerState::Missing => "missing",
                qiongli_project::ProjectMigrationMarkerState::Conflicting => "conflicting",
            },
            reconciliation: preview.reconciliation.clone(),
            source_retained: preview.source_retained,
            destination_removal: preview.destination_removal.clone(),
            can_rollback: preview.can_rollback,
        }),
    }
}

fn project_migration_rollback_blocked_reason(reason: &str) -> &'static str {
    match reason {
        "project-migration-rollback-source-drift" => "project-migration-rollback-source-drift",
        "project-migration-rollback-destination-drift" => {
            "project-migration-rollback-destination-drift"
        }
        "project-migration-rollback-marker-conflict" => {
            "project-migration-rollback-marker-conflict"
        }
        _ => "project-migration-rollback-blocked",
    }
}

pub(crate) fn app_capture_intake_operation_preview(
    token: String,
    file_label: String,
    preview: &CaptureIntakePreviewV1,
) -> AppOperationPreview {
    let can_confirm = preview.effect == CaptureIntakeEffect::AppendPendingHistory;
    AppOperationPreview {
        token,
        kind: "capture-intake",
        title: "Import research capture",
        summary: "Verify and append this bounded research capture to the selected project's portable review history. No session, transcript, or private host path is retained.".to_owned(),
        display_target: Some(file_label),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: if can_confirm {
            vec!["filesystem-write"]
        } else {
            Vec::new()
        },
        can_confirm,
        blocked_reason: (!can_confirm).then_some("capture-already-intaken"),
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_capture_consolidation_operation_preview(
    token: String,
    preview: &CaptureConsolidationPreviewV1,
) -> AppOperationPreview {
    let can_confirm = preview.outcome == CaptureConsolidationOutcome::Ready;
    let blocked_reason = match preview.outcome {
        CaptureConsolidationOutcome::Ready => None,
        CaptureConsolidationOutcome::Conflicted => Some("academic-review-conflict"),
        CaptureConsolidationOutcome::AlreadyConsolidated => Some("capture-already-consolidated"),
    };
    AppOperationPreview {
        token,
        kind: "capture-consolidation",
        title: "Consolidate reviewed capture",
        summary: "Apply only the reviewed academic deltas shown in this plan to the canonical research state and decision log, with a portable consolidation receipt.".to_owned(),
        display_target: Some(preview.capture_id.as_str().to_owned()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: if can_confirm {
            vec!["academic-consolidation", "filesystem-write"]
        } else {
            Vec::new()
        },
        can_confirm,
        blocked_reason,
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_capture_delivery_acknowledgement_operation_preview(
    token: String,
    preview: &CaptureDeliveryAcknowledgementPreviewV1,
) -> AppOperationPreview {
    AppOperationPreview {
        token,
        kind: "capture-delivery-acknowledgement",
        title: "Acknowledge capture delivery",
        summary: "Record the exact accepted capture and resulting project revision against the current delivery generation without changing academic artifacts.".to_owned(),
        display_target: Some(preview.envelope_id.as_str().to_owned()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["delivery-acknowledgement"],
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_capture_assignment_operation_preview(
    token: String,
    preview: &CaptureAssignmentPreviewV1,
) -> AppOperationPreview {
    AppOperationPreview {
        token,
        kind: "capture-assignment",
        title: "Assign capture delivery",
        summary: "Record the explicit target decision and exact source-to-child lineage. Canonical academic artifacts remain unchanged until a separate item-scoped resolution is confirmed.".to_owned(),
        display_target: Some(preview.target_project_id.as_str().to_owned()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["assignment-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_capture_resolution_operation_preview(
    token: String,
    preview: &CaptureResolutionPreviewV1,
) -> AppOperationPreview {
    AppOperationPreview {
        token,
        kind: "capture-resolution",
        title: "Apply reviewed capture resolution",
        summary: "Apply the complete item-scoped academic choices shown in this plan, advance exactly one project revision, and retain immutable assignment and resolution lineage.".to_owned(),
        display_target: Some(preview.target_project_id.as_str().to_owned()),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["academic-review", "filesystem-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

pub(crate) fn app_portfolio_maintenance_operation_preview(
    token: String,
    preview: &PortfolioMaintenancePreviewV1,
) -> AppOperationPreview {
    let (kind, title, summary) = match preview.operation {
        PortfolioMaintenanceOperation::Reconcile => (
            "portfolio-reconcile",
            "Reconcile portfolio catalog",
            "Update only changed derived project contributions against the exact current Research Library revision.",
        ),
        PortfolioMaintenanceOperation::FullRebuild => (
            "portfolio-full-rebuild",
            "Rebuild portfolio catalog",
            "Rebuild every derived contribution from registered canonical project artifacts and publish only the complete catalog.",
        ),
        PortfolioMaintenanceOperation::DeleteDerivedState => (
            "portfolio-delete-derived-state",
            "Delete derived portfolio state",
            "Delete only the private rebuildable portfolio catalog. Registered projects and canonical academic artifacts are retained.",
        ),
    };
    AppOperationPreview {
        token,
        kind,
        title,
        summary: summary.to_owned(),
        display_target: preview.expected_catalog_id.clone(),
        plan_digest_sha256: Some(preview.plan_digest.clone()),
        approvals_required: vec!["derived-state-write"],
        can_confirm: true,
        blocked_reason: None,
        migration: None,
        migration_rollback: None,
    }
}

impl From<ResearchCaptureV1> for AppResearchCaptureV1 {
    fn from(capture: ResearchCaptureV1) -> Self {
        Self {
            schema_version: capture.schema_version,
            capture_id: capture.capture_id.as_str().to_owned(),
            binding: capture.binding.into(),
            source: capture.source,
            delivery: capture.delivery,
            captured_at_unix: capture.captured_at_unix,
            summary: capture.summary,
            changes: capture.changes.into_iter().map(Into::into).collect(),
            decisions: capture.decisions.into_iter().map(Into::into).collect(),
            evidence: capture.evidence.into_iter().map(Into::into).collect(),
            contradictions: capture.contradictions.into_iter().map(Into::into).collect(),
            next_actions: capture.next_actions,
        }
    }
}

impl From<ProjectBindingV1> for AppCaptureBindingV1 {
    fn from(binding: ProjectBindingV1) -> Self {
        Self {
            schema_version: binding.schema_version,
            project_id: binding.project_id.as_str().to_owned(),
            base_revision: binding.base_revision,
            stage: binding.stage,
            task: binding.task,
            capture_policy: binding.capture_policy,
        }
    }
}

impl From<SemanticChangeV1> for AppSemanticChangeV1 {
    fn from(change: SemanticChangeV1) -> Self {
        Self {
            area: change.area,
            summary: change.summary,
        }
    }
}

impl From<DecisionCandidateV1> for AppDecisionCandidateV1 {
    fn from(decision: DecisionCandidateV1) -> Self {
        Self {
            relation: decision.relation,
            statement: decision.statement,
            rationale: decision.rationale,
            target: decision.target,
        }
    }
}

impl From<EvidenceReferenceV1> for AppEvidenceReferenceV1 {
    fn from(evidence: EvidenceReferenceV1) -> Self {
        Self {
            locator_kind: evidence.locator_kind,
            locator: evidence.locator,
            relevance: evidence.relevance,
            limitation: evidence.limitation,
        }
    }
}

impl From<ContradictionV1> for AppContradictionV1 {
    fn from(contradiction: ContradictionV1) -> Self {
        Self {
            statement: contradiction.statement,
            conflicts_with: contradiction.conflicts_with,
            consequence: contradiction.consequence,
        }
    }
}

impl AppIntegrationSelection {
    const fn into_desktop(self) -> IntegrationSelection {
        IntegrationSelection {
            codex: self.codex,
            claude_code: self.claude_code,
        }
    }
}

impl AppProfileId {
    pub(crate) const fn into_desktop(self) -> ProfileKind {
        match self {
            Self::SkillOnly => ProfileKind::SkillOnly,
            Self::MarketplaceLite => ProfileKind::MarketplaceLite,
            Self::Full => ProfileKind::Full,
        }
    }
}

impl AppSkillsPreset {
    const fn into_desktop(self) -> SkillsDestinationPreset {
        match self {
            Self::QiongliManaged => SkillsDestinationPreset::QiongliManaged,
            Self::CurrentProject => SkillsDestinationPreset::CurrentProject,
            Self::CustomFolder => SkillsDestinationPreset::CustomFolder,
        }
    }
}

impl AppUpdateStream {
    const fn into_desktop(self) -> UpdateStreamView {
        match self {
            Self::Stable => UpdateStreamView::Stable,
            Self::Beta => UpdateStreamView::Beta,
        }
    }
}

pub(crate) fn app_event(
    event: DesktopEvent,
    current_snapshot: DesktopSnapshotV1,
    research_library: ResearchLibrarySnapshotV1,
    project_skills: Vec<AppProjectSkillsTargetView>,
) -> Result<AppEvent, &'static str> {
    Ok(match event {
        DesktopEvent::SnapshotReplaced(_) => AppEvent::Snapshot {
            snapshot: AppSnapshotV1::from_desktop(
                current_snapshot,
                research_library,
                project_skills,
            )?,
        },
        DesktopEvent::PreviewReady(preview) => AppEvent::Preview {
            preview: app_operation_preview(preview)?,
        },
        DesktopEvent::AgentRunCompleted(_) => AppEvent::Failed {
            code: "app-api-event-unsupported",
        },
        DesktopEvent::Completed { code } => AppEvent::Completed {
            code,
            snapshot: AppSnapshotV1::from_desktop(
                current_snapshot,
                research_library,
                project_skills,
            )?,
        },
        DesktopEvent::Cancelled { code } => AppEvent::Cancelled { code },
        DesktopEvent::ValidationFailed { code } => AppEvent::ValidationFailed { code },
        DesktopEvent::Failed { code } => AppEvent::Failed { code },
        DesktopEvent::UpdateChanged {
            update,
            close_requested,
        } => AppEvent::UpdateChanged {
            update: app_update_view(update),
            close_requested,
        },
        DesktopEvent::SkillsDestinationSelected { target_id, .. } => {
            AppEvent::SkillsDestinationSelected {
                target_id,
                symbolic_path: "<custom-folder>",
            }
        }
        DesktopEvent::McpSelfTestUpdated(self_test) => AppEvent::McpSelfTestUpdated {
            self_test: app_mcp_self_test_view(self_test),
        },
    })
}

fn app_mcp_self_test_view(view: McpSelfTestView) -> AppMcpSelfTestView {
    AppMcpSelfTestView {
        profile: "full",
        product_version: env!("CARGO_PKG_VERSION"),
        state: match view.state {
            McpSelfTestState::Running => "running",
            McpSelfTestState::Passed => "passed",
            McpSelfTestState::Failed => "failed",
            McpSelfTestState::Cancelled => "cancelled",
            McpSelfTestState::TimedOut => "timed-out",
        },
        checks: view.checks.map(|check| AppMcpSelfTestCheckView {
            check: match check.check {
                McpSelfTestCheckId::EmbeddedContract => "embedded-contract",
                McpSelfTestCheckId::Initialize => "initialize",
                McpSelfTestCheckId::ToolRegistry => "tool-registry",
                McpSelfTestCheckId::FullDispatch => "full-dispatch",
                McpSelfTestCheckId::ProviderReadiness => "provider-readiness",
                McpSelfTestCheckId::ClientRegistration => "client-registration",
            },
            status: check.status.code(),
            code: check.code,
            remediation: check.remediation,
        }),
        public_tool_count: view.public_tool_count,
        enabled_provider_count: view.enabled_provider_count,
        ready_provider_count: view.ready_provider_count,
        discovered_client_count: view.discovered_client_count,
        registered_client_count: view.registered_client_count,
    }
}

fn app_update_view(update: UpdateView) -> AppUpdateView {
    AppUpdateView {
        status: update.status.code(),
        selected_stream: update_stream_id(update.selected_stream),
        phase: update_phase_id(update.phase),
        available_version: update.available_version,
        archive_size_bytes: update.archive_size_bytes,
        progress: update.progress.map(|progress| AppUpdateProgressView {
            completed_steps: progress.completed_steps,
            total_steps: progress.total_steps,
            label: progress.label,
            indeterminate: progress.indeterminate,
        }),
        reason_code: update.reason_code,
        remediation: update.remediation.code(),
        can_select_stream: update.can_select_stream,
        can_check: update.can_check,
        can_prepare: update.can_prepare,
        can_install: update.can_install,
        can_cancel: update.can_cancel,
    }
}

const fn update_stream_id(stream: UpdateStreamView) -> &'static str {
    match stream {
        UpdateStreamView::Stable => "stable",
        UpdateStreamView::Beta => "beta",
    }
}

const fn update_phase_id(phase: UpdatePhaseView) -> &'static str {
    match phase {
        UpdatePhaseView::Unavailable => "unavailable",
        UpdatePhaseView::Idle => "idle",
        UpdatePhaseView::Checking => "checking",
        UpdatePhaseView::Current => "current",
        UpdatePhaseView::Available => "available",
        UpdatePhaseView::Downloading => "downloading",
        UpdatePhaseView::Verifying => "verifying",
        UpdatePhaseView::Staging => "staging",
        UpdatePhaseView::ReadyToInstall => "ready-to-install",
        UpdatePhaseView::Installing => "installing",
        UpdatePhaseView::AwaitingRestart => "awaiting-restart",
        UpdatePhaseView::Cancelling => "cancelling",
        UpdatePhaseView::Cancelled => "cancelled",
        UpdatePhaseView::RecoveryRequired => "recovery-required",
        UpdatePhaseView::Failed => "failed",
    }
}

fn app_integration_view(integration: IntegrationView) -> AppIntegrationView {
    let connection = app_connection_view(&integration);
    let legacy_detected = integration.paths[..integration.path_count]
        .iter()
        .flatten()
        .any(|path| {
            path.source == qiongli_ui::IntegrationPathSourceView::LegacyObserved
                && path.state != StatusCode::Missing
        });
    AppIntegrationView {
        target: integration_target_id(integration.target),
        label: integration.target.label(),
        connection,
        client: AppClientView {
            detected: integration.client == StatusCode::Ready,
            status: integration.client.code(),
            version: integration.client_version.map(|version| version.label()),
            compatibility: integration.compatibility.code(),
            minimum_supported_version: minimum_supported_client_version(integration.target),
        },
        plugin: AppPluginVersionView {
            installed_version: integration
                .installed_plugin_version
                .map(|version| version.label()),
            available_version: integration.available_plugin_version.label(),
        },
        discovery: integration.discovery.label(),
        candidate_required: integration.candidate_required,
        legacy_detected,
        migration: AppLegacyMigrationView {
            state: integration.migration.state.code(),
            detected_items: integration.migration.detected_items,
            eligible_items: integration.migration.eligible_items,
            review_items: integration.migration.review_items,
        },
        overall: integration.overall.code(),
        managed_content: AppManagedContentView {
            source: integration.source.code(),
            skills: integration.skills.code(),
            marketplace: integration.marketplace.code(),
            direct_package: integration.direct_package.map(StatusCode::code),
            registration: integration.registration.code(),
            activation: integration.activation_status.code(),
            activation_observation: integration.activation_observation.code(),
            mcp_attachment: integration.mcp_attachment.code(),
            mcp_attachment_observation: integration.mcp_attachment_observation.code(),
        },
        symbolic_location: integration.symbolic_location.label(),
        activation_policy: integration.activation.label(),
        host_action: app_host_action_view(&integration),
        ownership: integration.ownership.label(),
        ownership_state: integration.ownership.code(),
        next_action: integration.next_action.code(),
        evidence_code: integration.evidence_code,
        paths: integration
            .paths
            .into_iter()
            .take(integration.path_count)
            .flatten()
            .map(app_integration_path_view)
            .collect(),
    }
}

fn app_host_action_view(integration: &IntegrationView) -> Option<AppHostActionView> {
    if integration.client != StatusCode::Ready
        || integration.compatibility != qiongli_ui::ClientCompatibilityView::Supported
        || integration.activation_observation == qiongli_ui::IntegrationObservationView::Observed
            && integration.mcp_attachment_observation
                == qiongli_ui::IntegrationObservationView::Observed
    {
        return None;
    }
    if integration.activation_status == StatusCode::Drifted
        || integration.mcp_attachment == StatusCode::Drifted
        || integration.next_action == qiongli_ui::IntegrationActionView::RepairReady
        || integration.activation_observation == qiongli_ui::IntegrationObservationView::Observed
    {
        Some(app_host_refresh_action_for_target(integration.target))
    } else {
        Some(app_host_action_for_target(integration.target))
    }
}

fn app_host_action_for_target(target: IntegrationTarget) -> AppHostActionView {
    app_host_action_for_mode(target, HostPluginMode::Install)
}

fn app_host_refresh_action_for_target(target: IntegrationTarget) -> AppHostActionView {
    app_host_action_for_mode(target, HostPluginMode::Repair)
}

fn app_host_action_for_mode(target: IntegrationTarget, mode: HostPluginMode) -> AppHostActionView {
    AppHostActionView {
        scope: host_plugin_scope(target),
        restart_required: true,
        commands: host_plugin_command_templates(target, mode)
            .iter()
            .map(|command| AppHostCommandView {
                executable: host_plugin_executable_name(target),
                arguments: command.arguments.to_vec(),
            })
            .collect(),
    }
}

fn app_connection_view(integration: &IntegrationView) -> AppConnectionView {
    if integration.compatibility == qiongli_ui::ClientCompatibilityView::Unsupported {
        return AppConnectionView {
            state: "unsupported-client-version",
            label: "Unsupported client version",
            reason_code: "client-version-below-supported-minimum",
        };
    }
    if integration.discovery == qiongli_ui::IntegrationDiscoveryState::Unavailable {
        return AppConnectionView {
            state: "inspection-blocked",
            label: "Inspection blocked",
            reason_code: integration.evidence_code,
        };
    }
    if integration.client != StatusCode::Ready {
        return AppConnectionView {
            state: "client-not-detected",
            label: "Client not detected",
            reason_code: integration.evidence_code,
        };
    }
    if matches!(
        integration.overall,
        StatusCode::Drifted
            | StatusCode::Conflict
            | StatusCode::RecoveryRequired
            | StatusCode::Invalid
            | StatusCode::Insecure
    ) {
        return AppConnectionView {
            state: "needs-repair",
            label: "Needs repair",
            reason_code: integration.evidence_code,
        };
    }
    if integration.registration == StatusCode::Ready
        && integration.activation_observation == qiongli_ui::IntegrationObservationView::Observed
        && integration.mcp_attachment_observation
            == qiongli_ui::IntegrationObservationView::Observed
    {
        return AppConnectionView {
            state: "connected",
            label: "Connected",
            reason_code: "qiongli-plugin-observed-healthy",
        };
    }
    if matches!(
        integration.activation_observation,
        qiongli_ui::IntegrationObservationView::ProbeUnavailable
            | qiongli_ui::IntegrationObservationView::ProbeFailed
    ) || matches!(
        integration.mcp_attachment_observation,
        qiongli_ui::IntegrationObservationView::ProbeUnavailable
            | qiongli_ui::IntegrationObservationView::ProbeFailed
    ) {
        let probe_failed = integration.activation_observation
            == qiongli_ui::IntegrationObservationView::ProbeFailed
            || integration.mcp_attachment_observation
                == qiongli_ui::IntegrationObservationView::ProbeFailed;
        return AppConnectionView {
            state: "inspection-blocked",
            label: "Inspection blocked",
            reason_code: if probe_failed {
                "qiongli-plugin-host-probe-failed"
            } else {
                "qiongli-plugin-host-probe-unavailable"
            },
        };
    }
    if integration.activation_observation == qiongli_ui::IntegrationObservationView::Observed {
        return AppConnectionView {
            state: "activated",
            label: "Activated",
            reason_code: "qiongli-plugin-activated-attachment-not-observable",
        };
    }
    if integration.registration == StatusCode::Ready
        && integration.activation_observation
            == qiongli_ui::IntegrationObservationView::ClientActionRequired
    {
        return AppConnectionView {
            state: "installed-host-action-required",
            label: "Installed, host action required",
            reason_code: "qiongli-plugin-host-action-required",
        };
    }
    if integration.source == StatusCode::Ready {
        return AppConnectionView {
            state: "prepared",
            label: "Prepared",
            reason_code: "qiongli-plugin-prepared",
        };
    }
    AppConnectionView {
        state: "detected-not-connected",
        label: "Detected, not connected",
        reason_code: integration.evidence_code,
    }
}

const fn minimum_supported_client_version(target: IntegrationTarget) -> &'static str {
    match target {
        IntegrationTarget::Codex => "0.144.1",
        IntegrationTarget::ClaudeCode => "2.1.206",
    }
}

fn app_integration_path_view(path: IntegrationPathView) -> AppIntegrationPathView {
    AppIntegrationPathView {
        surface: path.surface.label(),
        scope: path.scope.label(),
        source: path.source.label(),
        state: path.state.code(),
        management: path.management.label(),
        selected: path.selected,
        symbolic_path: path.symbolic_path,
    }
}

fn app_operation_preview(preview: OperationPreview) -> Result<AppOperationPreview, &'static str> {
    if !preview.validate() {
        return Err("operation-preview-invalid");
    }
    let display_target = preview
        .display_target
        .map(|value| value.expose().to_owned());
    if display_target
        .as_deref()
        .is_some_and(|target| !valid_app_operation_display_target(preview.kind, target))
    {
        return Err("operation-preview-target-invalid");
    }
    Ok(AppOperationPreview {
        token: format!("{:032x}", preview.token.value()),
        kind: operation_kind_id(preview.kind),
        title: preview.title,
        summary: preview.summary.to_owned(),
        display_target,
        plan_digest_sha256: preview.plan_digest_sha256,
        approvals_required: preview
            .approvals_required
            .into_iter()
            .map(OperationApproval::label)
            .collect(),
        can_confirm: preview.can_confirm,
        blocked_reason: preview.blocked_reason,
        migration: None,
        migration_rollback: None,
    })
}

fn valid_app_operation_display_target(kind: OperationKind, target: &str) -> bool {
    match kind {
        OperationKind::Activation => {
            let codex = format!(
                "Codex · {CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH} → {CODEX_MARKETPLACE_SYMBOLIC_PATH}"
            );
            let claude = format!(
                "Claude Code · {CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH} → {CLAUDE_MARKETPLACE_SYMBOLIC_PATH}"
            );
            target == codex || target == claude || target == format!("{codex} | {claude}")
        }
        OperationKind::SkillsMaterialization
        | OperationKind::SkillsRemoval
        | OperationKind::SkillsDetach => matches!(
            target,
            "<user-home>/.qiongli-skills" | "<project>/.qiongli-skills" | "<custom-folder>"
        ),
        OperationKind::CliInstall | OperationKind::CliRemove => {
            target == "<user-home>/.local/bin/qiongli"
        }
        OperationKind::CliPathConfigure => matches!(
            target,
            "<user-home>/.zprofile" | "<user-home>/.bash_profile" | "<user-home>/.profile"
        ),
        OperationKind::ZoteroCompanionStage => target
            .strip_prefix("<qiongli-state>/zotero/companion/")
            .is_some_and(|suffix| {
                !suffix.is_empty()
                    && suffix.len() <= 160
                    && suffix
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
            }),
        OperationKind::GlobalSettings
        | OperationKind::ProviderSettings
        | OperationKind::ProviderSecret
        | OperationKind::AgentBackendSettings
        | OperationKind::AgentBackendSecret
        | OperationKind::AgentRun
        | OperationKind::UpdateInstall
        | OperationKind::LegacyMigrationStage
        | OperationKind::LegacyMigrationHostActivation
        | OperationKind::LegacyMigrationCleanup
        | OperationKind::LegacyMigrationFinalize
        | OperationKind::LegacyMigrationRecovery => false,
    }
}

fn parse_operation_token(value: &str) -> Result<OperationToken, &'static str> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("operation-token-invalid");
    }
    u128::from_str_radix(value, 16)
        .map(OperationToken::new)
        .map_err(|_| "operation-token-invalid")
}

const fn profile_label(profile: ProfileKind) -> &'static str {
    match profile {
        ProfileKind::SkillOnly => "Skills",
        ProfileKind::MarketplaceLite => "Plugin Lite",
        ProfileKind::Full => "Full workflow",
    }
}

const fn integration_target_id(target: IntegrationTarget) -> &'static str {
    match target {
        IntegrationTarget::Codex => "codex",
        IntegrationTarget::ClaudeCode => "claude-code",
    }
}

const fn operation_kind_id(kind: OperationKind) -> &'static str {
    match kind {
        OperationKind::Activation => "activation",
        OperationKind::GlobalSettings => "global-settings",
        OperationKind::ProviderSettings => "provider-settings",
        OperationKind::ProviderSecret => "provider-secret",
        OperationKind::AgentBackendSettings => "agent-backend-settings",
        OperationKind::AgentBackendSecret => "agent-backend-secret",
        OperationKind::AgentRun => "agent-run",
        OperationKind::SkillsMaterialization => "skills-materialization",
        OperationKind::SkillsRemoval => "skills-removal",
        OperationKind::SkillsDetach => "skills-detach",
        OperationKind::CliInstall => "cli-install",
        OperationKind::CliRemove => "cli-remove",
        OperationKind::CliPathConfigure => "cli-path-configure",
        OperationKind::ZoteroCompanionStage => "zotero-companion-stage",
        OperationKind::UpdateInstall => "update-install",
        OperationKind::LegacyMigrationStage => "legacy-migration-stage",
        OperationKind::LegacyMigrationHostActivation => "legacy-migration-host-activation",
        OperationKind::LegacyMigrationCleanup => "legacy-migration-cleanup",
        OperationKind::LegacyMigrationFinalize => "legacy-migration-finalize",
        OperationKind::LegacyMigrationRecovery => "legacy-migration-recovery",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn native_host_actions_bind_exact_client_commands_and_scopes() {
        let codex = serde_json::to_value(app_host_action_for_target(IntegrationTarget::Codex))
            .expect("Codex Host action must serialize");
        assert_eq!(codex["scope"], "personal");
        assert_eq!(codex["commands"][0]["executable"], "codex");
        assert_eq!(
            codex["commands"][0]["arguments"],
            json!(["plugin", "add", "--json", "qiongli-next@personal"])
        );

        let claude =
            serde_json::to_value(app_host_action_for_target(IntegrationTarget::ClaudeCode))
                .expect("Claude Code Host action must serialize");
        assert_eq!(claude["scope"], "user");
        assert_eq!(claude["commands"].as_array().map(Vec::len), Some(2));
        for command in claude["commands"].as_array().expect("commands") {
            assert_eq!(command["executable"], "claude");
            assert_eq!(
                command["arguments"]
                    .as_array()
                    .and_then(|arguments| arguments.last()),
                Some(&json!("user"))
            );
        }
    }

    #[test]
    fn native_host_refresh_actions_force_same_version_cache_replacement() {
        let codex =
            serde_json::to_value(app_host_refresh_action_for_target(IntegrationTarget::Codex))
                .expect("Codex refresh action must serialize");
        assert_eq!(codex["commands"].as_array().map(Vec::len), Some(2));
        assert_eq!(
            codex["commands"][0]["arguments"],
            json!(["plugin", "remove", "--json", "qiongli-next@personal"])
        );
        assert_eq!(
            codex["commands"][1]["arguments"],
            json!(["plugin", "add", "--json", "qiongli-next@personal"])
        );

        let claude = serde_json::to_value(app_host_refresh_action_for_target(
            IntegrationTarget::ClaudeCode,
        ))
        .expect("Claude refresh action must serialize");
        assert_eq!(claude["commands"].as_array().map(Vec::len), Some(3));
        assert_eq!(
            claude["commands"][1]["arguments"],
            json!([
                "plugin",
                "uninstall",
                "qiongli-next@qiongli-local",
                "--scope",
                "user",
                "--yes"
            ])
        );
    }

    #[test]
    fn managed_operation_previews_reject_private_targets_at_the_app_boundary() {
        let preview = OperationPreview {
            token: OperationToken::new(1),
            kind: OperationKind::SkillsMaterialization,
            title: "Skills materialization preview",
            summary: "Write the selected embedded profile to the selected destination.",
            display_target: Some(qiongli_ui::PrivateDisplayText::new(
                "/Users/researcher/private-skills".to_owned(),
            )),
            plan_digest_sha256: Some("a".repeat(64)),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        };
        assert_eq!(
            app_operation_preview(preview).unwrap_err(),
            "operation-preview-target-invalid"
        );

        let symbolic = OperationPreview {
            token: OperationToken::new(2),
            kind: OperationKind::SkillsMaterialization,
            title: "Skills materialization preview",
            summary: "Write the selected embedded profile to the selected destination.",
            display_target: Some(qiongli_ui::PrivateDisplayText::new(
                "<custom-folder>".to_owned(),
            )),
            plan_digest_sha256: Some("b".repeat(64)),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        };
        assert_eq!(
            app_operation_preview(symbolic)
                .unwrap()
                .display_target
                .as_deref(),
            Some("<custom-folder>")
        );
    }

    #[test]
    fn parameterized_app_intents_accept_only_camel_case_fields() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-project-create",
            "directoryToken": "0000000000000000000000000000002a",
            "displayName": "Trustworthy research agents",
            "projectKind": "article",
            "stage": "writing"
        }))
        .expect("the TypeScript App API field casing must deserialize");

        assert!(matches!(
            intent,
            AppIntent::PreviewProjectCreate {
                directory_token,
                display_name,
                project_kind: ProjectKind::Article,
                stage: ProjectStage::Writing,
            } if directory_token == "0000000000000000000000000000002a"
                && display_name == "Trustworthy research agents"
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "read-capture",
                "project_id": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "capture_id": format!("cap_{}", "a".repeat(64))
            }))
            .is_err(),
            "snake_case fields must not become a second IPC contract"
        );

        let rollback = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-project-migration-rollback",
            "directoryToken": "0000000000000000000000000000002a"
        }))
        .expect("migration rollback must use only an opaque directory token");
        assert!(matches!(
            rollback,
            AppIntent::PreviewProjectMigrationRollback { directory_token }
                if directory_token == "0000000000000000000000000000002a"
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "preview-project-migration-rollback",
                "directoryToken": "0000000000000000000000000000002a",
                "destinationPath": "/private/migrated-project"
            }))
            .is_err()
        );

        let graph_query = serde_json::from_value::<AppIntent>(json!({
            "action": "query-academic-graph",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "query": {
                "expectedProjectionId": format!("grp_{}", "a".repeat(64)),
                "focusNodeId": format!("nod_{}", "b".repeat(64)),
                "direction": "outgoing",
                "maxDepth": 2,
                "nodeTypes": [],
                "relations": [],
                "layers": [],
                "canonicalId": null,
                "text": null,
                "maxNodes": 100,
                "maxEdges": 200
            }
        }))
        .expect("graph query depth must deserialize through the typed query contract");
        assert!(matches!(
            graph_query,
            AppIntent::QueryAcademicGraph {
                query: AcademicGraphQueryV1 { max_depth: 2, .. },
                ..
            }
        ));

        let graph_open = serde_json::from_value::<AppIntent>(json!({
            "action": "open-academic-graph-artifact",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "expectedProjectRevision": 12,
            "expectedProjectionId": format!("grp_{}", "a".repeat(64)),
            "entity": { "kind": "edge", "id": format!("edg_{}", "b".repeat(64)) }
        }))
        .expect("graph artifact opening must deserialize without accepting a path");
        assert!(matches!(
            graph_open,
            AppIntent::OpenAcademicGraphArtifact {
                expected_project_revision: 12,
                entity: AppAcademicGraphEntity::Edge { .. },
                ..
            }
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "open-academic-graph-artifact",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "expectedProjectionId": format!("grp_{}", "a".repeat(64)),
                "entity": { "kind": "node", "id": format!("nod_{}", "b".repeat(64)) },
                "artifactPath": "/private/research/context/research_state.md"
            }))
            .is_err()
        );

        let graph_read = serde_json::from_value::<AppIntent>(json!({
            "action": "read-project-artifact",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "expectedProjectRevision": 12,
            "reference": {
                "kind": "academic-graph-entity",
                "expectedProjectionId": format!("grp_{}", "a".repeat(64)),
                "entity": { "kind": "edge", "id": format!("edg_{}", "b".repeat(64)) }
            },
            "maxBytes": 65536
        }))
        .expect("project artifact reads must use an opaque graph reference");
        assert!(matches!(
            graph_read,
            AppIntent::ReadProjectArtifact {
                expected_project_revision: 12,
                reference: AppProjectArtifactReference::AcademicGraphEntity { .. },
                max_bytes: 65536,
                ..
            }
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "read-project-artifact",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "reference": {
                    "kind": "registered-artifact",
                    "artifactPath": "/private/research/context/research_state.md",
                    "sourceAnchor": null
                },
                "maxBytes": 65536
            }))
            .expect("unknown strings deserialize before validation")
            .validate()
            .is_err()
        );

        let graph_path = serde_json::from_value::<AppIntent>(json!({
            "action": "query-academic-graph-path",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "query": {
                "expectedProjectionId": format!("grp_{}", "a".repeat(64)),
                "sourceNodeId": format!("nod_{}", "b".repeat(64)),
                "targetNodeId": format!("nod_{}", "c".repeat(64)),
                "maxHops": 6
            }
        }))
        .expect("graph path query must deserialize through the typed query contract");
        assert!(matches!(
            graph_path,
            AppIntent::QueryAcademicGraphPath {
                query: AcademicGraphPathQueryV1 { max_hops: 6, .. },
                ..
            }
        ));

        let orchestration_control = serde_json::from_value::<AppIntent>(json!({
            "action": "control-orchestration",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
            "expectedProjectRevision": 12,
            "runId": format!("run_{}", "2".repeat(32)),
            "expectedGeneration": 3,
            "expectedDocumentSha256": "3".repeat(64),
            "actionName": "pause"
        }))
        .expect("orchestration control must deserialize through the checkpoint reference");
        assert!(matches!(
            orchestration_control,
            AppIntent::ControlOrchestration {
                expected_project_revision: 12,
                expected_generation: 3,
                action_name: AppOrchestrationControlAction::Pause,
                ..
            }
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "control-orchestration",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "runId": format!("run_{}", "2".repeat(32)),
                "expectedGeneration": 3,
                "expectedDocumentSha256": "3".repeat(64),
                "action_name": "pause"
            }))
            .is_err(),
            "snake_case orchestration controls must not become a second IPC contract"
        );
    }

    #[test]
    fn continuity_intents_decode_only_bounded_path_redacted_fields() {
        let deliveries = serde_json::from_value::<AppIntent>(json!({
            "action": "load-capture-deliveries",
            "request": {
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "states": ["queued", "retry-required"],
                "limit": 64
            }
        }))
        .expect("delivery page request must decode");
        assert!(deliveries.validate().is_ok());
        assert!(matches!(
            deliveries,
            AppIntent::LoadCaptureDeliveries {
                request: AppCaptureDeliveryListRequestV1 {
                    project_id: Some(project_id),
                    states,
                    limit: 64,
                    cursor: None,
                }
            } if project_id.as_str() == "prj_018f4d5a3b2c71008a9b0c1d2e3f4051"
                && states == [
                    CaptureDeliveryState::Queued,
                    CaptureDeliveryState::RetryRequired,
                ]
        ));

        let resolution = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-capture-resolution",
            "assignmentReceiptId": format!("car_{}", "2".repeat(64)),
            "reviewedAtUnix": 11,
            "selections": [{
                "itemId": format!("cri_{}", "3".repeat(64)),
                "disposition": "accept-capture"
            }]
        }))
        .expect("item-scoped resolution request must decode");
        assert!(resolution.validate().is_ok());
        assert!(matches!(
            resolution,
            AppIntent::PreviewCaptureResolution {
                reviewed_at_unix: 11,
                selections: Some(selections),
                ..
            } if selections.len() == 1
                && selections[0].disposition == CaptureResolutionDisposition::AcceptCapture
        ));
        let resolution_plan = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-capture-resolution",
            "assignmentReceiptId": format!("car_{}", "2".repeat(64)),
            "reviewedAtUnix": 11
        }))
        .expect("selection-free request must remain a read-only resolution plan");
        assert!(matches!(
            resolution_plan,
            AppIntent::PreviewCaptureResolution {
                selections: None,
                ..
            }
        ));

        let portfolio = serde_json::from_value::<AppIntent>(json!({
            "action": "query-portfolio",
            "request": {
                "catalogId": format!("pca_{}", "4".repeat(64)),
                "filters": {
                    "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                    "evidenceSignal": "contradiction",
                    "text": "causal evidence"
                },
                "limits": {
                    "projects": 32,
                    "nodes": 128,
                    "edges": 128,
                    "lineage": 128,
                    "maxBytes": 2097152
                }
            }
        }))
        .expect("bounded portfolio query request must decode");
        assert!(portfolio.validate().is_ok());
        assert!(matches!(
            portfolio,
            AppIntent::QueryPortfolio {
                request: AppPortfolioQueryRequestV1 {
                    filters: AppPortfolioQueryFiltersV1 {
                        evidence_signal: Some(PortfolioEvidenceSignal::Contradiction),
                        ..
                    },
                    limits: AppPortfolioQueryLimitsV1 {
                        projects: 32,
                        nodes: 128,
                        edges: 128,
                        lineage: 128,
                        max_bytes: 2_097_152,
                    },
                    ..
                }
            }
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "query-portfolio",
                "request": {
                    "catalogId": format!("pca_{}", "4".repeat(64)),
                    "filters": {},
                    "limits": {
                        "projects": 32,
                        "nodes": 128,
                        "edges": 128,
                        "lineage": 128,
                        "maxBytes": 2097152
                    },
                    "projectRoot": "/private/research"
                }
            }))
            .is_err(),
            "private paths must not become part of the native App API request"
        );
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "retry-capture-delivery",
                "envelope_id": format!("env_{}", "1".repeat(64)),
                "expectedGeneration": 2,
                "expectedRecordSha256": "5".repeat(64),
                "retriedAtUnix": 12,
                "cause": "transport-unavailable"
            }))
            .is_err(),
            "snake_case continuity fields must not become a second IPC contract"
        );
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "inspect-capture-delivery",
                "envelopeId": "env_not-a-content-addressed-identity"
            }))
            .is_ok_and(|intent| intent.validate().is_err()),
            "malformed delivery identities must fail native App API validation"
        );
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "preview-capture-resolution",
                "assignmentReceiptId": format!("car_{}", "2".repeat(64)),
                "reviewedAtUnix": 11,
                "selections": [{
                    "itemId": "cri_invalid",
                    "disposition": "accept-capture"
                }]
            }))
            .is_ok_and(|intent| intent.validate().is_err()),
            "malformed resolution item identities must fail native App API validation"
        );
        let oversized_page = serde_json::from_value::<AppIntent>(json!({
            "action": "load-capture-deliveries",
            "request": {
                "states": [
                    "queued",
                    "delivering",
                    "delivered",
                    "acknowledged",
                    "retry-required",
                    "conflicted",
                    "cancelled",
                    "queued"
                ],
                "limit": 64
            }
        }))
        .expect("bounded validation follows structural decoding");
        assert_eq!(
            oversized_page.validate(),
            Err("app-capture-delivery-page-invalid")
        );
        let invalid_catalog = serde_json::from_value::<AppIntent>(json!({
            "action": "query-portfolio",
            "request": {
                "catalogId": "pca_invalid",
                "filters": {},
                "limits": {
                    "projects": 32,
                    "nodes": 128,
                    "edges": 128,
                    "lineage": 128,
                    "maxBytes": 2097152
                }
            }
        }))
        .expect("bounded validation follows structural decoding");
        assert_eq!(
            invalid_catalog.validate(),
            Err("app-portfolio-query-invalid")
        );
    }

    #[test]
    fn managed_skills_app_contract_accepts_only_opaque_target_ids() {
        let target_id = format!("skills-target-{}", "2".repeat(64));
        for action in [
            "verify-managed-skills-target",
            "preview-update-managed-skills-target",
            "preview-remove-managed-skills-target",
            "preview-detach-managed-skills-target",
        ] {
            let intent = serde_json::from_value::<AppIntent>(json!({
                "action": action,
                "targetId": target_id.clone(),
            }))
            .expect("managed Skills actions must decode an opaque target identity");
            assert_eq!(intent.validate(), Ok(()));
            assert!(
                serde_json::from_value::<AppIntent>(json!({
                    "action": action,
                    "targetId": target_id.clone(),
                    "path": "/private/skills",
                }))
                .is_err(),
                "managed Skills actions must reject private paths"
            );
        }

        let invalid = serde_json::from_value::<AppIntent>(json!({
            "action": "verify-managed-skills-target",
            "targetId": "skills-target-invalid",
        }))
        .expect("identity bounds are validated after structural decoding");
        assert_eq!(invalid.validate(), Err("managed-skills-target-id-invalid"));

        let selected = serde_json::to_value(AppEvent::SkillsDestinationSelected {
            target_id,
            symbolic_path: "<custom-folder>",
        })
        .unwrap();
        assert_eq!(selected["type"], "skills-destination-selected");
        assert_eq!(selected["symbolicPath"], "<custom-folder>");
        assert!(selected.get("path").is_none());
    }

    #[test]
    fn continuity_pages_bind_cursors_to_the_complete_native_snapshot() {
        let status = |digit: char| CaptureDeliveryStatusV1 {
            schema_version: 1,
            envelope_id: DeliveryEnvelopeId::parse(format!("env_{}", digit.to_string().repeat(64)))
                .unwrap(),
            capture_id: CaptureId::parse(format!("cap_{}", digit.to_string().repeat(64))).unwrap(),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Connected,
            destination: None,
            state: CaptureDeliveryState::Queued,
            generation: 1,
            attempt_count: 0,
            retry_count: 0,
            created_at_unix: 1,
            updated_at_unix: 1,
            last_reason: CaptureDeliveryReason::DeliveryEnqueued,
            envelope_sha256: digit.to_string().repeat(64),
            record_sha256: digit.to_string().repeat(64),
            acknowledgement: None,
        };
        let statuses = vec![status('1'), status('2')];
        let first = app_capture_delivery_page(
            AppCaptureDeliveryListRequestV1 {
                project_id: None,
                states: Vec::new(),
                limit: 1,
                cursor: None,
            },
            statuses.clone(),
        )
        .expect("first page is projected from the complete native observation");
        assert!(first.truncated);
        assert_eq!(first.entries.len(), 1);
        let cursor = first
            .next_cursor
            .clone()
            .expect("a truncated native page returns one content-bound cursor");
        assert!(cursor.valid_for(AppContinuityCursorKind::Deliveries));
        assert_eq!(cursor.snapshot_id, first.snapshot_id);

        let second = app_capture_delivery_page(
            AppCaptureDeliveryListRequestV1 {
                project_id: None,
                states: Vec::new(),
                limit: 1,
                cursor: Some(cursor.clone()),
            },
            statuses.clone(),
        )
        .expect("the exact snapshot accepts its native cursor");
        assert!(!second.truncated);
        assert_eq!(second.entries.len(), 1);
        assert_ne!(first.entries[0].envelope_id, second.entries[0].envelope_id);

        let mut changed = statuses;
        changed[1].record_sha256 = "3".repeat(64);
        assert_eq!(
            app_capture_delivery_page(
                AppCaptureDeliveryListRequestV1 {
                    project_id: None,
                    states: Vec::new(),
                    limit: 1,
                    cursor: Some(cursor),
                },
                changed,
            )
            .unwrap_err(),
            "app-continuity-cursor-stale"
        );
    }

    #[test]
    fn assignment_projection_closes_resolution_capability_after_receipt_observation() {
        let status = CaptureAssignmentStatusV1 {
            schema_version: 1,
            state: CaptureAssignmentStatusState::Completed,
            intent_id: CaptureAssignmentIntentId::parse(format!("cai_{}", "1".repeat(64))).unwrap(),
            source_envelope_id: DeliveryEnvelopeId::parse(format!("env_{}", "2".repeat(64)))
                .unwrap(),
            source_capture_id: CaptureId::parse(format!("cap_{}", "3".repeat(64))).unwrap(),
            target_project_id: ProjectId::parse("prj_018f4d5a3b2c71008a9b0c1d2e3f4051").unwrap(),
            target_project_revision: 1,
            outcome: Some(CaptureAssignmentOutcome::Assigned),
            receipt_id: Some(
                CaptureAssignmentReceiptId::parse(format!("car_{}", "4".repeat(64))).unwrap(),
            ),
            derived_capture_id: Some(CaptureId::parse(format!("cap_{}", "5".repeat(64))).unwrap()),
            child_envelope_id: Some(
                DeliveryEnvelopeId::parse(format!("env_{}", "6".repeat(64))).unwrap(),
            ),
            created_at_unix: 2,
            decided_at_unix: Some(3),
        };
        let unresolved = serde_json::to_value(app_capture_assignment_view(status.clone(), true))
            .expect("unresolved assignment view serializes");
        let resolved = serde_json::to_value(app_capture_assignment_view(status, false))
            .expect("resolved assignment view serializes");
        assert_eq!(unresolved["canResolve"], true);
        assert_eq!(resolved["canResolve"], false);
    }

    #[test]
    fn app_api_v7_rejects_retired_model_backend_credentials_and_prompts() {
        for retired in [
            json!({
                "action": "preview-agent-backend-credential",
                "apiKey": "openai-private-api-canary"
            }),
            json!({
                "action": "preview-agent-backend-credential",
                "api_key": "wrong-field"
            }),
            json!({
                "action": "preview-agent-run",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "prompt": "private-agent-run-prompt-canary"
            }),
            json!({
                "action": "preview-agent-backend-settings",
                "expectedRevision": 4,
                "enabled": true
            }),
            json!({ "action": "test-open-ai-backend" }),
        ] {
            assert!(
                serde_json::from_value::<AppIntent>(retired).is_err(),
                "retired direct-model values must not cross App API v5"
            );
        }
    }

    #[test]
    fn legacy_backend_credential_removal_remains_available_for_cleanup() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-remove-agent-backend-credential"
        }))
        .expect("credential cleanup must remain available");

        assert!(matches!(
            intent.into_desktop(),
            Ok(DesktopIntent::PreviewAgentBackendSecretChange {
                change: AgentBackendSecretChange::Remove,
            })
        ));
    }

    #[test]
    fn literature_provider_intents_map_without_exposing_secrets_in_app_snapshots() {
        let settings = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-provider-settings",
            "expectedRevision": 7,
            "providersEnabled": {
                "openalex": true,
                "semanticScholar": true,
                "crossref": true,
                "pubmed": false,
                "arxiv": true
            },
            "publicSettingChanges": [
                {
                    "provider": "crossref",
                    "change": "replace",
                    "value": "crossref@example.org"
                }
            ]
        }))
        .expect("provider enablement must use the public App API contract");
        assert!(matches!(
            settings.into_desktop(),
            Ok(DesktopIntent::PreviewProviderSettingsPatch(
                ProviderSettingsPatch {
                    expected_revision: 7,
                    providers_enabled: [true, true, true, false, true],
                    openalex_email: PublicSettingChange::Keep,
                    crossref_email: PublicSettingChange::Replace(value),
                }
            )) if value.expose() == "crossref@example.org"
        ));

        let unsupported_setting = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-provider-settings",
            "expectedRevision": 7,
            "providersEnabled": {
                "openalex": true,
                "semanticScholar": true,
                "crossref": true,
                "pubmed": false,
                "arxiv": true
            },
            "publicSettingChanges": [{
                "provider": "pubmed",
                "change": "remove"
            }]
        }))
        .expect("unsupported public fields cross the decoder for native validation");
        assert!(matches!(
            unsupported_setting.into_desktop(),
            Err("provider-public-setting-unsupported")
        ));

        let secret = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-provider-secret-change",
            "provider": "semantic-scholar",
            "change": "replace",
            "value": "private-provider-key-canary"
        }))
        .expect("supported provider secrets must deserialize");
        assert!(matches!(
            secret.into_desktop(),
            Ok(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::SemanticScholar,
                change: ProviderSecretChange::Replace(value),
            }) if value.expose() == "private-provider-key-canary"
        ));

        let pubmed = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-provider-secret-change",
            "provider": "pubmed",
            "change": "replace",
            "value": "private-pubmed-key-canary"
        }))
        .expect("PubMed credentials must deserialize");
        assert!(matches!(
            pubmed.into_desktop(),
            Ok(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::PubMed,
                change: ProviderSecretChange::Replace(value),
            }) if value.expose() == "private-pubmed-key-canary"
        ));

        let unsupported = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-provider-secret-change",
            "provider": "crossref",
            "change": "replace",
            "value": "private-provider-key-canary"
        }))
        .expect("the boundary reports unsupported provider credentials deterministically");
        assert!(matches!(
            unsupported.into_desktop(),
            Err("provider-secret-unsupported")
        ));
    }

    #[test]
    fn content_preview_resources_are_bounded_utf8() {
        let digest = "1".repeat(64);
        assert!(
            AppContentPreviewResourceV1::from_bytes(
                "workflow/SKILL.md",
                "markdown",
                true,
                &digest,
                &digest,
                b"# Qiongli\n",
            )
            .is_ok()
        );
        assert_eq!(
            AppContentPreviewResourceV1::from_bytes(
                "workflow/SKILL.md",
                "markdown",
                true,
                &digest,
                &digest,
                &[b'x'; 128 * 1024 + 1],
            ),
            Err("content-preview-too-large")
        );
        assert_eq!(
            AppContentPreviewResourceV1::from_bytes(
                "workflow/SKILL.md",
                "markdown",
                true,
                &digest,
                &digest,
                &[0xff],
            ),
            Err("content-preview-not-utf8")
        );
    }

    #[test]
    fn retired_orchestration_execution_intents_do_not_cross_the_app_api() {
        for retired in [
            json!({
                "action": "preview-orchestration-test",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "executionMode": "triad"
            }),
            json!({
                "action": "preview-orchestration-continue",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "expectedProjectRevision": 12,
                "runId": format!("run_{}", "2".repeat(32)),
                "expectedGeneration": 3,
                "expectedDocumentSha256": "3".repeat(64)
            }),
        ] {
            assert!(
                serde_json::from_value::<AppIntent>(retired).is_err(),
                "retired orchestration execution must not cross the App API"
            );
        }
    }

    #[test]
    fn standalone_skills_intents_reject_host_owned_destinations() {
        for preset in ["detected-codex", "detected-claude-code"] {
            for retired in [
                json!({
                    "action": "preview-skills-preset-materialization",
                    "profile": "marketplace-lite",
                    "preset": preset
                }),
                json!({
                    "action": "verify-skills-preset",
                    "preset": preset
                }),
                json!({
                    "action": "preview-skills-preset-removal",
                    "preset": preset
                }),
            ] {
                assert!(
                    serde_json::from_value::<AppIntent>(retired).is_err(),
                    "host-owned Skills must remain under Client Integration"
                );
            }
        }
    }

    #[test]
    fn registered_project_skills_intent_is_path_free_and_requires_project_interception() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-project-skills-materialization",
            "profile": "marketplace-lite",
            "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051"
        }))
        .expect("registered project Skills intent must parse");
        assert!(matches!(
            intent.into_desktop(),
            Err("project-skills-requires-project-service")
        ));
        assert!(
            serde_json::from_value::<AppIntent>(json!({
                "action": "preview-project-skills-materialization",
                "profile": "marketplace-lite",
                "projectId": "prj_018f4d5a3b2c71008a9b0c1d2e3f4051",
                "path": "/Users/private/research"
            }))
            .is_err()
        );
    }

    #[test]
    fn integration_reconciliation_preserves_explicit_selection_and_rejects_old_global_intents() {
        let intent = serde_json::from_value::<AppIntent>(json!({
            "action": "preview-reconcile-integrations",
            "selection": {
                "codex": false,
                "claudeCode": true
            }
        }))
        .expect("selection-bound reconciliation must parse");
        assert!(matches!(
            intent.into_desktop(),
            Ok(DesktopIntent::PreviewReconcileIntegrations {
                selection: IntegrationSelection {
                    codex: false,
                    claude_code: true,
                },
            })
        ));

        for retired in [
            json!({ "action": "preview-repair-all" }),
            json!({
                "action": "preview-update-integrations",
                "selection": {
                    "codex": true,
                    "claudeCode": false
                }
            }),
        ] {
            assert!(
                serde_json::from_value::<AppIntent>(retired).is_err(),
                "global or duplicate Integration reconciliation intents must stay retired"
            );
        }
    }

    #[test]
    fn parameterized_app_events_serialize_camel_case_fields() {
        let selected = serde_json::to_value(AppEvent::ProjectDirectorySelected {
            token: "0000000000000000000000000000002a".to_owned(),
            root_label: "trustworthy-research-agents".to_owned(),
        })
        .expect("app event must serialize");
        assert_eq!(
            selected,
            json!({
                "type": "project-directory-selected",
                "token": "0000000000000000000000000000002a",
                "rootLabel": "trustworthy-research-agents"
            })
        );

        let capture = serde_json::to_value(AppEvent::CaptureFileSelected {
            token: "0000000000000000000000000000002b".to_owned(),
            file_label: "portable-research-capture.json".to_owned(),
        })
        .expect("capture event must serialize");
        assert!(capture.get("fileLabel").is_some());
        assert!(capture.get("file_label").is_none());

        let update = serde_json::to_value(AppEvent::UpdateChanged {
            update: app_update_view(UpdateView {
                status: StatusCode::Ready,
                selected_stream: UpdateStreamView::Stable,
                phase: UpdatePhaseView::Idle,
                available_version: None,
                archive_size_bytes: None,
                progress: None,
                reason_code: "update-ready",
                remediation: qiongli_ui::UpdateRemediation::None,
                can_select_stream: true,
                can_check: true,
                can_prepare: false,
                can_install: false,
                can_cancel: false,
            }),
            close_requested: true,
        })
        .expect("update event must serialize");
        assert_eq!(update.get("closeRequested"), Some(&json!(true)));
        assert!(update.get("close_requested").is_none());
    }

    #[test]
    fn main_window_capability_is_limited_to_update_handoff_close() {
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json"))
                .expect("main window capability must be valid JSON");

        assert_eq!(
            capability.get("permissions"),
            Some(&json!(["core:window:allow-close"]))
        );
    }

    #[test]
    fn operation_tokens_have_one_canonical_ipc_encoding() {
        let token = parse_operation_token("0000000000000000000000000000002a").unwrap();
        assert_eq!(token.value(), 42);
        assert_eq!(parse_operation_token("2a"), Err("operation-token-invalid"));
        assert_eq!(
            parse_operation_token("0000000000000000000000000000002A"),
            Err("operation-token-invalid")
        );
    }

    #[test]
    fn integration_selection_maps_without_hidden_defaults() {
        assert_eq!(
            AppIntegrationSelection {
                codex: true,
                claude_code: false,
            }
            .into_desktop(),
            IntegrationSelection {
                codex: true,
                claude_code: false,
            }
        );
    }

    #[test]
    fn update_state_maps_to_the_bounded_app_contract() {
        let update = app_update_view(UpdateView {
            status: StatusCode::Ready,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::Available,
            available_version: Some("2.0.0-alpha.2".to_owned()),
            archive_size_bytes: Some(24_600_000),
            progress: None,
            reason_code: "trusted-update-available",
            remediation: qiongli_ui::UpdateRemediation::RetryPreparation,
            can_select_stream: true,
            can_check: true,
            can_prepare: true,
            can_install: false,
            can_cancel: false,
        });

        assert_eq!(update.selected_stream, "beta");
        assert_eq!(update.phase, "available");
        assert_eq!(update.available_version.as_deref(), Some("2.0.0-alpha.2"));
        assert_eq!(update.remediation, "retry-update-preparation");
        assert!(update.can_prepare);
    }
}
