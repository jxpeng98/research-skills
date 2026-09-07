#![cfg_attr(
    any(test, not(feature = "desktop")),
    allow(
        dead_code,
        reason = "the Tauri shell is excluded from CLI-only builds and core library unit tests"
    )
)]

use qiongli_config::{
    ConfigError, ConfigState, EmailAddress, GlobalSettings, LegacyProviderId,
    LegacyProviderResolution, LegacyProviderResolutionStrategy, LegacyProviderSecret,
    ProviderReadiness, RedactedProviderStatus, SecretRef, SecretStore, SecretStoreStatus,
    SecretValue, UpdateStateStore, UpdateStreamPreference, UpdateTransactionPhase,
    WorkflowVariantStore,
};
use qiongli_content::{
    EmbeddedContent, MaterializationReceiptV1, MaterializationTarget, ProfileId, WorkflowOverrides,
    approve_materialization_target, verify_materialization,
};
use qiongli_execution::{
    AgentFinishReason, AgentRunResultV1, BackendControlService, BackendReadinessV1,
    CancellationToken as AgentCancellationToken, openai_backend_metadata_status,
    openai_backend_status,
};
#[cfg(test)]
use qiongli_platform::discover_legacy_migration;
use qiongli_platform::{
    ApprovalRequirement, Architecture, CLAUDE_MARKETPLACE_SYMBOLIC_PATH,
    CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH, CODEX_MARKETPLACE_SYMBOLIC_PATH,
    CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH, ClientActionReadiness, ClientActivationCoordinator,
    ClientActivationDisposition, ClientActivationHandle, ClientActivationPreview,
    ClientActivationTarget, ClientComponentState, ClientDiscoveryState, ClientInventoryEntryV1,
    ClientKind, ClientOwnershipState, ClientPathManagement, ClientPathScope, ClientPathSource,
    ClientPathState, ClientPathSurface, InstallPlanMetadataV1, LegacyMigrationInventoryV1,
    LegacyMigrationItemState, LegacyMigrationReadiness, LegacyMigrationState, LegacyMigrationStore,
    OperatingSystem, PackagedProductBatchInstallPreview, PackagedProductInstallEffect,
    PackagedProductInstallPreview, PackagedProductInstallVerification,
    PackagedProductVerificationInput, TrustedPublicKey, VerifiedLaunchGrant,
    VerifiedNativeReleaseCandidate, VerifiedPackagedProduct, ZOTERO_COMPANION_ZOTERO_MAX_VERSION,
    ZOTERO_COMPANION_ZOTERO_MIN_VERSION, ZoteroCompanionStageEffect, ZoteroCompanionStagePlan,
    apply_native_release_candidate_local, apply_packaged_product_batch_install_with_overrides,
    apply_packaged_product_install_with_overrides, apply_zotero_companion_stage,
    approve_claude_plugin_bundle_target, approve_codex_plugin_bundle_target, approve_install_plan,
    discover_legacy_migration_with_config, packaged_product_control_path,
    preview_client_activation, preview_packaged_product_batch_install_with_variant,
    preview_packaged_product_install_with_variant, preview_zotero_companion_stage,
    remove_packaged_product_install, verify_claude_plugin_bundle, verify_codex_plugin_bundle,
    verify_packaged_product, verify_packaged_product_install_with_variant,
    verify_receipt_owned_packaged_product_install, verify_zotero_companion_stage,
};
use qiongli_runtime::mcp::MCP_PROTOCOL_VERSION;
use qiongli_runtime::providers::{ProviderAccess, ProviderAvailability, ProviderId};
use qiongli_runtime::zotero::companion::{
    CompanionClient, DEFAULT_CONNECTOR_URL, ZoteroIntegrationState,
};
use qiongli_runtime::{
    FULL_PROJECT_PUBLIC_TOOL_NAMES, FullProjectToolRegistry, LITE_PUBLIC_TOOL_NAMES,
};
use qiongli_ui::{
    ActivationPolicy, AgentBackendReadinessView, AgentBackendSecretChange,
    AgentBackendSettingsPatch, AgentBackendView, AgentRunDraft, AgentRunResultView,
    ArchitectureView, CapabilityView, CliInstallStateView, CliPathStateView, CliView,
    ClientCompatibilityView, ClientVersionView, ConfigView, ContentView,
    DESKTOP_SNAPSHOT_SCHEMA_VERSION, DesktopEvent, DesktopIntent, DesktopService,
    DesktopSnapshotV1, DiagnosticCheckId, DiagnosticCheckView, DiagnosticPathView,
    EMPTY_INTEGRATION_PATHS, GlobalSettingsPatch, IntegrationActionView, IntegrationDiscoveryState,
    IntegrationMigrationStateView, IntegrationMigrationView, IntegrationObservationView,
    IntegrationOwnershipView, IntegrationPathManagementView, IntegrationPathScopeView,
    IntegrationPathSourceView, IntegrationPathSurfaceView, IntegrationPathView,
    IntegrationSelection, IntegrationTarget, IntegrationView, LegacyMigrationActionView,
    LegacyMigrationStateView, LegacyMigrationView, LegacyProviderConflictView,
    LegacyProviderResolutionStrategyView, LegacyProviderResolutionView, LegacyProviderView,
    MAX_INTEGRATION_PATHS, ManagedSkillsStateView, ManagedSkillsView, McpSelfTestCheckId,
    McpSelfTestCheckView, McpSelfTestState, McpSelfTestView, McpView, OperatingSystemView,
    OperationApproval, OperationKind, OperationPreview, OperationToken, PrivateDisplayText,
    ProductTrustView, ProductVersionChannelView, ProductVersionView, ProductView, ProfileKind,
    ProfileView, ProviderKind, ProviderReadinessView, ProviderSecretChange, ProviderSettingsPatch,
    ProviderView, PublicSettingChange, RemediationCode, SkillsDestinationPreset, StatusCode,
    SymbolicLocation, UpdatePhaseView, UpdateProgressView, UpdateRemediation, UpdateStreamView,
    UpdateView, ZOTERO_FALLBACK_FORMATS, ZoteroIntegrationStateView, ZoteroIntegrationView,
    ZoteroObservationView,
};
use serde_json::json;
use sha2::{Digest, Sha256};

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(test)]
use std::sync::Condvar;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::agent_run::{FullAgentRunRequest, FullAgentRunService, readiness_reason_code};
use crate::cli_install::{
    CliInstallPlan, CliInstallState, CliPathConfigurePlan, CliPathState, CliRemovalEffect,
    CliRemovalPlan, apply_cli_install, apply_cli_path_configure, apply_cli_remove,
    bundled_cli_path, cli_target_matches_bundled, inspect_cli_install,
    installed_cli_product_authority, preview_cli_install, preview_cli_path_configure,
    preview_cli_remove,
};
use crate::command::{CommandEnvironment, config_root, config_store};
use crate::desktop_api::{
    AppCaptureAssignmentDecision, AppCaptureAssignmentListRequestV1, AppCaptureAssignmentPageV1,
    AppCaptureAssignmentPreviewV1, AppCaptureAssignmentViewV1,
    AppCaptureDeliveryAcknowledgementPreviewV1, AppCaptureDeliveryListRequestV1,
    AppCaptureDeliveryPageV1, AppCaptureDeliveryViewV1, AppCaptureResolutionListRequestV1,
    AppCaptureResolutionPageV1, AppCaptureResolutionPreviewV1, AppCaptureResolutionSelectionV1,
    AppCaptureResolutionViewV1, AppContinuityOperationPhase, AppContinuityOperationProgressV1,
    AppEvent, AppOperationPreview, AppPortfolioCatalogState, AppPortfolioDoctorV1,
    AppPortfolioMaintenanceOperation, AppPortfolioMaintenancePreviewV1,
    AppPortfolioMaintenanceResultV1, AppPortfolioQueryRequestV1, AppPortfolioQueryResultV1,
    AppPortfolioStatusV1, AppProjectMigrationQualification, AppProjectSkillsTargetView,
    AppResearchCaptureV1, AppSemanticTimelineRequestV1, AppSemanticTimelineResultV1, AppSnapshotV1,
    app_capture_assignment_operation_preview, app_capture_assignment_page,
    app_capture_assignment_preview, app_capture_assignment_view,
    app_capture_consolidation_operation_preview,
    app_capture_delivery_acknowledgement_operation_preview,
    app_capture_delivery_acknowledgement_preview, app_capture_delivery_page,
    app_capture_delivery_view, app_capture_intake_operation_preview,
    app_capture_resolution_operation_preview, app_capture_resolution_page,
    app_capture_resolution_preview, app_capture_resolution_view, app_continuity_operation_progress,
    app_event, app_portable_operation_preview, app_portfolio_current_status,
    app_portfolio_deletion_result, app_portfolio_doctor,
    app_portfolio_maintenance_operation_preview, app_portfolio_maintenance_preview,
    app_portfolio_query, app_portfolio_query_result, app_portfolio_reconciliation_result,
    app_portfolio_unavailable_status, app_project_guidance_operation_preview,
    app_project_migration_operation_preview, app_project_migration_recovery_operation_preview,
    app_project_migration_rollback_operation_preview, app_project_operation_preview,
    app_semantic_timeline_query, app_semantic_timeline_result,
    app_workflow_variant_operation_preview, serialize_app_api_contract_fixture,
};
use crate::managed_content::managed_skills_target_id;
use crate::mcp::{FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES, FullMcpServer, full_server};
use qiongli_project::{
    AcademicGraphArtifactTarget, AcademicGraphComparisonService, AcademicGraphEntityKind,
    AcademicGraphIndexService, AcademicGraphPathQueryV1, AcademicGraphPathResultV1,
    AcademicGraphPortfolioService, AcademicGraphPortfolioSnapshotV1, AcademicGraphQueryResultV1,
    AcademicGraphQueryV1, AcademicGraphReadinessV1, AcademicGraphRevisionComparisonV1,
    AcademicGraphService, AcademicGraphSnapshotV1, ApprovedCaptureAssignment,
    ApprovedCaptureConsolidation, ApprovedCaptureDeliveryAcknowledgement, ApprovedCaptureIntake,
    ApprovedCaptureResolution, ApprovedPortfolioMaintenance, ApprovedProjectMutation,
    ArtifactChangeSnapshotV1, CaptureAssignmentOutcome, CaptureAssignmentStatusV1,
    CaptureConsolidationPreviewV1, CaptureCoverageSnapshotV1,
    CaptureDeliveryAcknowledgementRequestV1, CaptureDeliveryRetryCause, CaptureId,
    CaptureInboxSnapshotV1, CaptureIntakePreviewV1, CaptureResolutionSelectionSetV1,
    IncrementalPortfolioService, LibraryHealth, PortfolioCancellationToken,
    PortfolioMaintenanceOperation, PortfolioQueryService, ProjectArtifactViewV1, ProjectError,
    ProjectHealth, ProjectId, ProjectKind, ProjectLifecycle, ProjectMutationKind,
    ProjectRegistrationOptions, ProjectStage, ProjectStateService, ResearchLibrarySnapshotV1,
    SemanticTimelineService, VerifiedCaptureAssignment, VerifiedCaptureConsolidation,
    VerifiedCaptureDeliveryAcknowledgement, VerifiedCaptureIntake, VerifiedCaptureResolution,
    VerifiedPortableProjectOperation, VerifiedPortfolioMaintenance, VerifiedProjectMigration,
    VerifiedProjectMigrationRecovery, VerifiedProjectMigrationRollback, VerifiedProjectMutation,
    read_portable_capture_packet,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopLaunchError;

#[cfg(all(feature = "desktop", not(test)))]
mod tauri_adapter;
#[cfg(all(feature = "desktop", not(test)))]
use tauri_adapter::run_tauri_application;

#[cfg(any(test, not(feature = "desktop")))]
fn run_tauri_application(
    _service: NativeDesktopService,
    _project_service: Option<ProjectStateService>,
) -> Result<(), DesktopLaunchError> {
    Err(DesktopLaunchError)
}

const ACTIVATION_PLAN_TTL_SECONDS: u64 = 600;
const MCP_SELF_TEST_TIMEOUT: Duration = Duration::from_secs(5);
const ACTIVATION_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];

pub fn run_desktop(
    mut environment: CommandEnvironment,
    content: EmbeddedContent,
) -> Result<(), DesktopLaunchError> {
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    let environment = environment.without_project_context();
    let product_control = running_packaged_product(&environment, &content);
    let service = NativeDesktopService::new_with_packaged_product(
        environment,
        content,
        Vec::new(),
        product_control,
    );
    run_tauri_application(service, project_service)
}

pub fn run_desktop_with_activation_sessions(
    mut environment: CommandEnvironment,
    content: EmbeddedContent,
    sessions: Vec<DesktopActivationSession>,
) -> Result<(), DesktopLaunchError> {
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    let environment = environment.without_project_context();
    if sessions.len() > 2
        || sessions.iter().enumerate().any(|(index, session)| {
            sessions[..index]
                .iter()
                .any(|prior| prior.target == session.target)
        })
    {
        return Err(DesktopLaunchError);
    }
    let service = NativeDesktopService::new(environment, content, sessions);
    run_tauri_application(service, project_service)
}

pub fn run_desktop_with_candidate_sessions(
    mut environment: CommandEnvironment,
    content: EmbeddedContent,
    sessions: Vec<DesktopCandidateSession>,
) -> Result<(), DesktopLaunchError> {
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    let environment = environment.without_project_context();
    if sessions.len() > 2
        || sessions.iter().enumerate().any(|(index, session)| {
            sessions[..index]
                .iter()
                .any(|prior| prior.target == session.target)
        })
    {
        return Err(DesktopLaunchError);
    }
    let service = NativeDesktopService::new_with_candidate_sessions(environment, content, sessions);
    run_tauri_application(service, project_service)
}

struct ProjectDesktopState {
    service: Option<ProjectStateService>,
    selected_location: Option<SelectedProjectLocation>,
    pending: Option<PendingProjectOperation>,
    continuity_operations: BTreeMap<String, DesktopContinuityOperation>,
    #[cfg(test)]
    continuity_worker_gate: Option<Arc<(Mutex<bool>, Condvar)>>,
    academic_graph_history: BTreeMap<ProjectId, AcademicGraphSnapshotV1>,
}

enum SelectedProjectLocation {
    Register {
        token: String,
        root: PathBuf,
    },
    Create {
        token: String,
        root: PathBuf,
    },
    Export {
        token: String,
        project_id: ProjectId,
        destination: PathBuf,
    },
    Import {
        token: String,
        source: PathBuf,
        destination: PathBuf,
    },
    Migration {
        token: String,
        source: PathBuf,
        destination: PathBuf,
    },
    MigrationRecovery {
        token: String,
        source: PathBuf,
        destination: PathBuf,
    },
    MigrationRollback {
        token: String,
        source: PathBuf,
        destination: PathBuf,
    },
    CaptureIntake {
        token: String,
        project_id: ProjectId,
        source: PathBuf,
    },
}

enum PendingProjectOperation {
    Guidance {
        token: String,
        project_id: ProjectId,
        expected_sha256: Option<String>,
        content: String,
    },
    Mutation {
        token: String,
        plan: VerifiedProjectMutation,
    },
    Portable {
        token: String,
        plan: VerifiedPortableProjectOperation,
    },
    Migration {
        token: String,
        plan: VerifiedProjectMigration,
    },
    MigrationRecovery {
        token: String,
        plan: VerifiedProjectMigrationRecovery,
    },
    MigrationRollback {
        token: String,
        plan: VerifiedProjectMigrationRollback,
    },
    CaptureIntake {
        token: String,
        plan: Box<VerifiedCaptureIntake>,
    },
    CaptureConsolidation {
        token: String,
        plan: Box<VerifiedCaptureConsolidation>,
    },
    CaptureDeliveryAcknowledgement {
        token: String,
        plan: Box<VerifiedCaptureDeliveryAcknowledgement>,
    },
    CaptureAssignment {
        token: String,
        plan: Box<VerifiedCaptureAssignment>,
    },
    CaptureResolution {
        token: String,
        plan: Box<VerifiedCaptureResolution>,
        selections: CaptureResolutionSelectionSetV1,
    },
    PortfolioMaintenance {
        token: String,
        plan: VerifiedPortfolioMaintenance,
    },
}

#[derive(Debug, Eq, PartialEq)]
struct ConfirmedProjectOperation {
    code: &'static str,
    capture_project_id: Option<ProjectId>,
    continuity: Option<ConfirmedCaptureContinuity>,
    continuity_operation: Option<AppContinuityOperationProgressV1>,
    migration_qualification: Option<AppProjectMigrationQualification>,
}

#[derive(Debug, Eq, PartialEq)]
enum ConfirmedCaptureContinuity {
    Delivery(AppCaptureDeliveryViewV1),
    Assignment(AppCaptureAssignmentViewV1),
    Resolution(AppCaptureResolutionViewV1),
}

const MAX_DESKTOP_CONTINUITY_OPERATIONS: usize = 32;

struct DesktopContinuityOperation {
    cancellation: PortfolioCancellationToken,
    record: Arc<Mutex<DesktopContinuityOperationRecord>>,
}

#[derive(Clone)]
struct DesktopContinuityOperationRecord {
    operation_id: String,
    operation: PortfolioMaintenanceOperation,
    phase: AppContinuityOperationPhase,
    completed_units: usize,
    total_units: usize,
    catalog_id: Option<String>,
    cancellable: bool,
    reason_code: &'static str,
    result: Option<AppPortfolioMaintenanceResultV1>,
}

#[derive(Debug, Eq, PartialEq)]
enum DesktopContinuityPoll {
    Progress(AppContinuityOperationProgressV1),
    Completed(AppPortfolioMaintenanceResultV1),
}

impl ProjectDesktopState {
    fn preview_local_guidance(
        &mut self,
        project_id: ProjectId,
        expected_sha256: Option<String>,
        content: String,
    ) -> Result<AppOperationPreview, &'static str> {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let snapshot = service.snapshot().map_err(|error| error.reason_code())?;
        let project = snapshot
            .projects
            .iter()
            .find(|project| project.project_id == project_id)
            .ok_or("project-not-registered")?;
        if project.lifecycle != ProjectLifecycle::Active || project.health != ProjectHealth::Ready {
            return Err("project-guidance-project-not-ready");
        }
        let current = service
            .read_local_guidance(&project_id)
            .map_err(|error| error.reason_code())?;
        let current_sha256 = current
            .as_deref()
            .map(|value| lower_hex(&Sha256::digest(value.as_bytes())));
        if current_sha256.as_deref() != expected_sha256.as_deref() {
            return Err("project-guidance-revision-conflict");
        }
        let token = project_app_token()?;
        let digest = project_guidance_digest(&project_id, expected_sha256.as_deref(), &content);
        self.pending = Some(PendingProjectOperation::Guidance {
            token: token.clone(),
            project_id,
            expected_sha256,
            content,
        });
        Ok(app_project_guidance_operation_preview(token, digest))
    }

    const fn new(service: Option<ProjectStateService>) -> Self {
        Self {
            service,
            selected_location: None,
            pending: None,
            continuity_operations: BTreeMap::new(),
            #[cfg(test)]
            continuity_worker_gate: None,
            academic_graph_history: BTreeMap::new(),
        }
    }

    fn snapshot(&self) -> ResearchLibrarySnapshotV1 {
        project_snapshot(&self.service)
    }

    fn capture_inbox(
        &self,
        project_id: &ProjectId,
    ) -> Result<CaptureInboxSnapshotV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .capture_inbox(project_id)
            .map_err(|error| error.reason_code())
    }

    fn capture_coverage(
        &self,
        project_id: &ProjectId,
    ) -> Result<CaptureCoverageSnapshotV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .capture_coverage(project_id)
            .map_err(|error| error.reason_code())
    }

    fn artifact_changes(
        &self,
        project_id: &ProjectId,
    ) -> Result<ArtifactChangeSnapshotV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .artifact_changes(project_id)
            .map_err(|error| error.reason_code())
    }

    fn academic_graph(
        &mut self,
        project_id: &ProjectId,
    ) -> Result<
        (
            AcademicGraphSnapshotV1,
            AcademicGraphReadinessV1,
            Option<AcademicGraphRevisionComparisonV1>,
        ),
        &'static str,
    > {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let projection = AcademicGraphService::new(projects.clone())
            .rebuild_projection(project_id)
            .map_err(|error| error.reason_code())?;
        let graph = projection.graph;
        let comparison = self
            .academic_graph_history
            .get(project_id)
            .filter(|before| before.project_revision <= graph.project_revision)
            .map(|before| AcademicGraphComparisonService::compare(before, &graph))
            .transpose()
            .map_err(|error| error.reason_code())?;
        self.academic_graph_history
            .insert(project_id.clone(), graph.clone());
        Ok((graph, projection.readiness, comparison))
    }

    fn query_academic_graph(
        &self,
        project_id: &ProjectId,
        query: &AcademicGraphQueryV1,
    ) -> Result<AcademicGraphQueryResultV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphIndexService::new(projects.clone())
            .rebuild(project_id)
            .and_then(|index| index.query(query))
            .map_err(|error| error.reason_code())
    }

    fn academic_graph_portfolio(&self) -> Result<AcademicGraphPortfolioSnapshotV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphPortfolioService::new(projects.clone())
            .rebuild()
            .map_err(|error| error.reason_code())
    }

    fn query_academic_graph_path(
        &self,
        project_id: &ProjectId,
        query: &AcademicGraphPathQueryV1,
    ) -> Result<AcademicGraphPathResultV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphIndexService::new(projects.clone())
            .rebuild(project_id)
            .and_then(|index| index.explanatory_path(query))
            .map_err(|error| error.reason_code())
    }

    fn resolve_academic_graph_artifact(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        expected_projection_id: &str,
        entity_kind: AcademicGraphEntityKind,
        entity_id: &str,
    ) -> Result<AcademicGraphArtifactTarget, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphService::new(projects.clone())
            .resolve_artifact(
                project_id,
                expected_project_revision,
                expected_projection_id,
                entity_kind,
                entity_id,
            )
            .map_err(|error| error.reason_code())
    }

    fn read_academic_graph_artifact(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        expected_projection_id: &str,
        entity_kind: AcademicGraphEntityKind,
        entity_id: &str,
        max_bytes: usize,
    ) -> Result<ProjectArtifactViewV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphService::new(projects.clone())
            .read_graph_artifact(
                project_id,
                expected_project_revision,
                expected_projection_id,
                entity_kind,
                entity_id,
                max_bytes,
            )
            .map_err(|error| error.reason_code())
    }

    fn read_registered_project_artifact(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        artifact_path: &str,
        source_anchor: Option<&str>,
        max_bytes: usize,
    ) -> Result<ProjectArtifactViewV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        AcademicGraphService::new(projects.clone())
            .read_registered_artifact(
                project_id,
                expected_project_revision,
                artifact_path,
                source_anchor,
                max_bytes,
            )
            .map_err(|error| error.reason_code())
    }

    fn read_capture(
        &self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
    ) -> Result<AppResearchCaptureV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .read_capture(project_id, capture_id)
            .map_err(|error| error.reason_code())?
            .map(Into::into)
            .ok_or("capture-not-found")
    }

    fn capture_deliveries(
        &self,
        request: AppCaptureDeliveryListRequestV1,
    ) -> Result<AppCaptureDeliveryPageV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let library = projects.snapshot().map_err(|error| error.reason_code())?;
        let observed = if let Some(project_id) = request.project_id() {
            projects.list_capture_deliveries_for_project(project_id)
        } else {
            projects.list_capture_deliveries()
        }
        .map_err(|error| error.reason_code())?;
        let confirmed = if let Some(project_id) = request.project_id() {
            projects.list_capture_deliveries_for_project(project_id)
        } else {
            projects.list_capture_deliveries()
        }
        .map_err(|error| error.reason_code())?;
        if observed != confirmed
            || projects.snapshot().map_err(|error| error.reason_code())? != library
        {
            return Err(ProjectError::RevisionConflict.reason_code());
        }
        app_capture_delivery_page(request, observed)
    }

    fn inspect_capture_delivery(
        &self,
        envelope_id: &qiongli_project::DeliveryEnvelopeId,
    ) -> Result<AppCaptureDeliveryViewV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let library = projects.snapshot().map_err(|error| error.reason_code())?;
        let observed = projects
            .inspect_capture_delivery(envelope_id)
            .map_err(|error| error.reason_code())?;
        let confirmed = projects
            .inspect_capture_delivery(envelope_id)
            .map_err(|error| error.reason_code())?;
        if observed != confirmed
            || projects.snapshot().map_err(|error| error.reason_code())? != library
        {
            return Err(ProjectError::RevisionConflict.reason_code());
        }
        observed
            .map(app_capture_delivery_view)
            .ok_or("capture-delivery-not-found")
    }

    fn retry_capture_delivery(
        &self,
        envelope_id: &qiongli_project::DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: &str,
        retried_at_unix: u64,
        cause: CaptureDeliveryRetryCause,
    ) -> Result<AppCaptureDeliveryViewV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .retry_capture_delivery(
                envelope_id,
                expected_generation,
                expected_record_sha256,
                retried_at_unix,
                cause,
            )
            .map(app_capture_delivery_view)
            .map_err(|error| error.reason_code())
    }

    fn cancel_capture_delivery(
        &self,
        envelope_id: &qiongli_project::DeliveryEnvelopeId,
        expected_generation: u64,
        expected_record_sha256: &str,
        cancelled_at_unix: u64,
    ) -> Result<AppCaptureDeliveryViewV1, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .cancel_capture_delivery(
                envelope_id,
                expected_generation,
                expected_record_sha256,
                cancelled_at_unix,
            )
            .map(app_capture_delivery_view)
            .map_err(|error| error.reason_code())
    }

    fn preview_capture_delivery_acknowledgement(
        &mut self,
        request: CaptureDeliveryAcknowledgementRequestV1,
        expected_generation: u64,
        expected_record_sha256: &str,
    ) -> Result<
        (
            AppCaptureDeliveryAcknowledgementPreviewV1,
            AppOperationPreview,
        ),
        &'static str,
    > {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_capture_delivery_acknowledgement(
                &request,
                expected_generation,
                expected_record_sha256,
            )
            .map_err(|error| error.reason_code())?;
        let acknowledgement = app_capture_delivery_acknowledgement_preview(plan.preview());
        let token = project_app_token()?;
        let preview =
            app_capture_delivery_acknowledgement_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::CaptureDeliveryAcknowledgement {
            token,
            plan: Box::new(plan),
        });
        Ok((acknowledgement, preview))
    }

    fn capture_assignments(
        &self,
        request: AppCaptureAssignmentListRequestV1,
    ) -> Result<AppCaptureAssignmentPageV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        if let Some(project_id) = request.project_id() {
            projects
                .resolve_project_root(project_id)
                .map_err(|error| error.reason_code())?;
        }
        let library = projects.snapshot().map_err(|error| error.reason_code())?;
        let observed = projects
            .list_capture_assignments()
            .map_err(|error| error.reason_code())?;
        let resolvable = Self::resolvable_assignment_receipt_ids(projects, &library, &observed)?;
        let confirmed = projects
            .list_capture_assignments()
            .map_err(|error| error.reason_code())?;
        let confirmed_resolvable =
            Self::resolvable_assignment_receipt_ids(projects, &library, &confirmed)?;
        if observed != confirmed
            || resolvable != confirmed_resolvable
            || projects.snapshot().map_err(|error| error.reason_code())? != library
        {
            return Err(ProjectError::RevisionConflict.reason_code());
        }
        app_capture_assignment_page(request, observed, &resolvable)
    }

    fn inspect_capture_assignment(
        &self,
        intent_id: &qiongli_project::CaptureAssignmentIntentId,
    ) -> Result<AppCaptureAssignmentViewV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let library = projects.snapshot().map_err(|error| error.reason_code())?;
        let observed = projects
            .inspect_capture_assignment(intent_id)
            .map_err(|error| error.reason_code())?
            .ok_or("capture-assignment-not-found")?;
        let resolvable = Self::resolvable_assignment_receipt_ids(
            projects,
            &library,
            std::slice::from_ref(&observed),
        )?;
        let confirmed = projects
            .inspect_capture_assignment(intent_id)
            .map_err(|error| error.reason_code())?
            .ok_or("capture-assignment-not-found")?;
        let confirmed_resolvable = Self::resolvable_assignment_receipt_ids(
            projects,
            &library,
            std::slice::from_ref(&confirmed),
        )?;
        if observed != confirmed
            || resolvable != confirmed_resolvable
            || projects.snapshot().map_err(|error| error.reason_code())? != library
        {
            return Err(ProjectError::RevisionConflict.reason_code());
        }
        let can_resolve = observed
            .receipt_id
            .as_ref()
            .is_some_and(|receipt_id| resolvable.contains(receipt_id.as_str()));
        Ok(app_capture_assignment_view(observed, can_resolve))
    }

    fn preview_capture_assignment(
        &mut self,
        source_envelope_id: &qiongli_project::DeliveryEnvelopeId,
        target_project_id: &ProjectId,
        decision: AppCaptureAssignmentDecision,
        decided_at_unix: u64,
    ) -> Result<(AppCaptureAssignmentPreviewV1, AppOperationPreview), &'static str> {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_capture_assignment(
                source_envelope_id,
                target_project_id,
                decision.into_project(),
                decided_at_unix,
            )
            .map_err(|error| error.reason_code())?;
        let assignment = app_capture_assignment_preview(plan.preview());
        let token = project_app_token()?;
        let preview = app_capture_assignment_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::CaptureAssignment {
            token,
            plan: Box::new(plan),
        });
        Ok((assignment, preview))
    }

    fn resolvable_assignment_receipt_ids(
        projects: &ProjectStateService,
        library: &ResearchLibrarySnapshotV1,
        assignments: &[CaptureAssignmentStatusV1],
    ) -> Result<BTreeSet<String>, &'static str> {
        let active_projects = library
            .projects
            .iter()
            .filter(|project| project.lifecycle == ProjectLifecycle::Active)
            .map(|project| project.project_id.clone())
            .collect::<BTreeSet<_>>();
        let target_projects = assignments
            .iter()
            .filter(|assignment| {
                assignment.outcome == Some(CaptureAssignmentOutcome::Assigned)
                    && assignment.receipt_id.is_some()
                    && assignment.derived_capture_id.is_some()
                    && assignment.child_envelope_id.is_some()
                    && active_projects.contains(&assignment.target_project_id)
            })
            .map(|assignment| assignment.target_project_id.clone())
            .collect::<BTreeSet<_>>();
        let mut resolved = BTreeSet::new();
        for project_id in target_projects {
            for receipt in projects
                .list_capture_resolutions(&project_id)
                .map_err(|error| error.reason_code())?
            {
                resolved.insert(receipt.receipt.assignment_receipt_id.as_str().to_owned());
            }
        }
        Ok(assignments
            .iter()
            .filter(|assignment| {
                assignment.outcome == Some(CaptureAssignmentOutcome::Assigned)
                    && assignment.derived_capture_id.is_some()
                    && assignment.child_envelope_id.is_some()
                    && active_projects.contains(&assignment.target_project_id)
            })
            .filter_map(|assignment| assignment.receipt_id.as_ref())
            .map(|receipt_id| receipt_id.as_str().to_owned())
            .filter(|receipt_id| !resolved.contains(receipt_id))
            .collect())
    }

    fn capture_resolutions(
        &self,
        request: AppCaptureResolutionListRequestV1,
    ) -> Result<AppCaptureResolutionPageV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let library = projects.snapshot().map_err(|error| error.reason_code())?;
        let observed = projects
            .list_capture_resolutions(request.project_id())
            .map_err(|error| error.reason_code())?;
        let confirmed = projects
            .list_capture_resolutions(request.project_id())
            .map_err(|error| error.reason_code())?;
        if observed != confirmed
            || projects.snapshot().map_err(|error| error.reason_code())? != library
        {
            return Err(ProjectError::RevisionConflict.reason_code());
        }
        app_capture_resolution_page(request, observed)
    }

    fn inspect_capture_resolution(
        &self,
        project_id: &ProjectId,
        receipt_id: &qiongli_project::CaptureResolutionReceiptId,
    ) -> Result<AppCaptureResolutionViewV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let library = projects.snapshot().map_err(|error| error.reason_code())?;
        let observed = projects
            .inspect_capture_resolution(project_id, receipt_id)
            .map_err(|error| error.reason_code())?;
        let confirmed = projects
            .inspect_capture_resolution(project_id, receipt_id)
            .map_err(|error| error.reason_code())?;
        if observed != confirmed
            || projects.snapshot().map_err(|error| error.reason_code())? != library
        {
            return Err(ProjectError::RevisionConflict.reason_code());
        }
        observed
            .map(app_capture_resolution_view)
            .ok_or("capture-resolution-not-found")
    }

    fn preview_capture_resolution(
        &mut self,
        assignment_receipt_id: &qiongli_project::CaptureAssignmentReceiptId,
        reviewed_at_unix: u64,
        selections: Vec<AppCaptureResolutionSelectionV1>,
    ) -> Result<
        (
            AppCaptureResolutionPreviewV1,
            Vec<AppCaptureResolutionSelectionV1>,
            AppOperationPreview,
        ),
        &'static str,
    > {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_capture_resolution(assignment_receipt_id, reviewed_at_unix)
            .map_err(|error| error.reason_code())?;
        let domain_selections = selections
            .iter()
            .cloned()
            .map(AppCaptureResolutionSelectionV1::into_project)
            .collect();
        let selection_set =
            CaptureResolutionSelectionSetV1::new(plan.resolution_plan(), domain_selections)
                .map_err(|error| error.reason_code())?;
        let resolution = app_capture_resolution_preview(plan.preview());
        let token = project_app_token()?;
        let preview = app_capture_resolution_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::CaptureResolution {
            token,
            plan: Box::new(plan),
            selections: selection_set,
        });
        Ok((resolution, selections, preview))
    }

    fn capture_resolution_plan(
        &self,
        assignment_receipt_id: &qiongli_project::CaptureAssignmentReceiptId,
        reviewed_at_unix: u64,
    ) -> Result<AppCaptureResolutionPreviewV1, &'static str> {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        service
            .preview_capture_resolution(assignment_receipt_id, reviewed_at_unix)
            .map(|plan| app_capture_resolution_preview(plan.preview()))
            .map_err(|error| error.reason_code())
    }

    fn portfolio_status(&self) -> Result<AppPortfolioStatusV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let library = projects.snapshot().map_err(|error| error.reason_code())?;
        let portfolio = IncrementalPortfolioService::new(projects.clone());
        let status = match portfolio.current() {
            Ok(current) => {
                let confirmed = portfolio.current().map_err(|error| error.reason_code())?;
                if current != confirmed {
                    return Err(ProjectError::RevisionConflict.reason_code());
                }
                app_portfolio_current_status(&current)
            }
            Err(ProjectError::RecoveryRequired) => {
                app_portfolio_unavailable_status(&library, AppPortfolioCatalogState::Missing)
            }
            Err(ProjectError::RevisionConflict | ProjectError::PortfolioCatalogConflict) => {
                app_portfolio_unavailable_status(&library, AppPortfolioCatalogState::Stale)
            }
            Err(
                ProjectError::InvalidPortfolioCatalog
                | ProjectError::LockBusy
                | ProjectError::PersistenceFailed(_),
            ) => app_portfolio_unavailable_status(
                &library,
                AppPortfolioCatalogState::RecoveryRequired,
            ),
            Err(error) => return Err(error.reason_code()),
        };
        if projects.snapshot().map_err(|error| error.reason_code())? != library {
            return Err(ProjectError::RevisionConflict.reason_code());
        }
        Ok(status)
    }

    fn query_portfolio(
        &self,
        request: AppPortfolioQueryRequestV1,
    ) -> Result<AppPortfolioQueryResultV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let query = app_portfolio_query(request)?;
        PortfolioQueryService::new(projects.clone())
            .query(&query)
            .map(app_portfolio_query_result)
            .map_err(|error| error.reason_code())
    }

    fn semantic_timeline(
        &self,
        request: AppSemanticTimelineRequestV1,
    ) -> Result<AppSemanticTimelineResultV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        let query = app_semantic_timeline_query(request)?;
        SemanticTimelineService::new(projects.clone())
            .query(&query)
            .map(app_semantic_timeline_result)
            .map_err(|error| error.reason_code())
    }

    fn portfolio_doctor(&self) -> Result<AppPortfolioDoctorV1, &'static str> {
        let projects = self.service.as_ref().ok_or("project-service-unavailable")?;
        IncrementalPortfolioService::new(projects.clone())
            .doctor_compare()
            .map(app_portfolio_doctor)
            .map_err(|error| error.reason_code())
    }

    fn preview_portfolio_maintenance(
        &mut self,
        operation: AppPortfolioMaintenanceOperation,
    ) -> Result<(AppPortfolioMaintenancePreviewV1, AppOperationPreview), &'static str> {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let portfolio = IncrementalPortfolioService::new(service.clone());
        let operation = operation.into_project();
        let plan = match operation {
            PortfolioMaintenanceOperation::Reconcile => portfolio.preview_reconcile(),
            PortfolioMaintenanceOperation::FullRebuild => portfolio.preview_full_rebuild(),
            PortfolioMaintenanceOperation::DeleteDerivedState => {
                portfolio.preview_delete_derived_state()
            }
        }
        .map_err(|error| error.reason_code())?;
        let maintenance = app_portfolio_maintenance_preview(plan.preview());
        let token = project_app_token()?;
        let preview = app_portfolio_maintenance_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::PortfolioMaintenance { token, plan });
        Ok((maintenance, preview))
    }

    fn start_portfolio_maintenance(
        &mut self,
        service: ProjectStateService,
        token: &str,
        plan: VerifiedPortfolioMaintenance,
    ) -> Result<AppContinuityOperationProgressV1, &'static str> {
        if self.continuity_operations.len() >= MAX_DESKTOP_CONTINUITY_OPERATIONS {
            return Err("continuity-operation-limit-reached");
        }
        let preview = plan.preview().clone();
        let operation_id = portfolio_continuity_operation_id(token, &preview.plan_digest);
        let worker_operation_id = operation_id.clone();
        if self.continuity_operations.contains_key(&operation_id) {
            return Err("continuity-operation-identity-conflict");
        }
        let started_at_unix = now_unix()?;
        let library = service.snapshot().map_err(|error| error.reason_code())?;
        let total_units = library
            .projects
            .len()
            .max(preview.current_contribution_count)
            .max(1);
        let cancellation = PortfolioCancellationToken::new();
        let record = Arc::new(Mutex::new(DesktopContinuityOperationRecord {
            operation_id: operation_id.clone(),
            operation: preview.operation,
            phase: AppContinuityOperationPhase::Queued,
            completed_units: 0,
            total_units,
            catalog_id: preview.expected_catalog_id.clone(),
            cancellable: true,
            reason_code: "portfolio-operation-queued",
            result: None,
        }));
        self.continuity_operations.insert(
            operation_id,
            DesktopContinuityOperation {
                cancellation: cancellation.clone(),
                record: Arc::clone(&record),
            },
        );
        let queued = record
            .lock()
            .map_err(|_| "continuity-operation-lock-failed")?
            .progress();
        let operation = preview.operation;
        let worker_record = Arc::clone(&record);
        #[cfg(test)]
        let worker_gate = self.continuity_worker_gate.clone();
        let spawn = thread::Builder::new()
            .name("qiongli-portfolio-maintenance".to_owned())
            .spawn(move || {
                #[cfg(test)]
                if let Some(gate) = worker_gate {
                    let (released, signal) = &*gate;
                    let Ok(mut released) = released.lock() else {
                        update_continuity_record(&worker_record, |current| {
                            current.phase = AppContinuityOperationPhase::Failed;
                            current.cancellable = false;
                            current.reason_code = "continuity-operation-test-gate-failed";
                        });
                        return;
                    };
                    while !*released {
                        let Ok(next) = signal.wait(released) else {
                            update_continuity_record(&worker_record, |current| {
                                current.phase = AppContinuityOperationPhase::Failed;
                                current.cancellable = false;
                                current.reason_code = "continuity-operation-test-gate-failed";
                            });
                            return;
                        };
                        released = next;
                    }
                }
                update_continuity_record(&worker_record, |current| {
                    if cancellation.is_cancelled() {
                        current.phase = AppContinuityOperationPhase::Cancelled;
                        current.cancellable = false;
                        current.reason_code = "portfolio-operation-cancelled";
                    } else {
                        current.phase = AppContinuityOperationPhase::Running;
                        current.cancellable = true;
                        current.reason_code = "portfolio-operation-running";
                    }
                });
                if cancellation.is_cancelled() {
                    return;
                }
                let portfolio = IncrementalPortfolioService::new(service);
                let approval = ApprovedPortfolioMaintenance::new(preview.plan_digest.clone(), true);
                let result = match operation {
                    PortfolioMaintenanceOperation::Reconcile => portfolio
                        .apply_reconcile(&plan, &approval, started_at_unix, &cancellation)
                        .map(|reconciliation| {
                            app_portfolio_reconciliation_result(
                                worker_operation_id.clone(),
                                operation,
                                reconciliation,
                            )
                        }),
                    PortfolioMaintenanceOperation::FullRebuild => portfolio
                        .apply_full_rebuild(&plan, &approval, started_at_unix, &cancellation)
                        .map(|reconciliation| {
                            app_portfolio_reconciliation_result(
                                worker_operation_id.clone(),
                                operation,
                                reconciliation,
                            )
                        }),
                    PortfolioMaintenanceOperation::DeleteDerivedState => {
                        if cancellation.is_cancelled() {
                            Err(ProjectError::OperationCancelled)
                        } else {
                            portfolio
                                .apply_delete_derived_state(&plan, &approval)
                                .map(|deletion| {
                                    app_portfolio_deletion_result(
                                        worker_operation_id.clone(),
                                        deletion,
                                    )
                                })
                        }
                    }
                };
                update_continuity_record(&worker_record, |current| match result {
                    Ok(result) => {
                        current.phase = AppContinuityOperationPhase::Completed;
                        current.completed_units = current.total_units;
                        current.cancellable = false;
                        current.reason_code = "portfolio-operation-completed";
                        current.result = Some(result);
                    }
                    Err(ProjectError::OperationCancelled) => {
                        current.phase = AppContinuityOperationPhase::Cancelled;
                        current.cancellable = false;
                        current.reason_code = "portfolio-operation-cancelled";
                    }
                    Err(
                        ProjectError::RecoveryRequired
                        | ProjectError::InvalidPortfolioCatalog
                        | ProjectError::PersistenceFailed(_),
                    ) => {
                        current.phase = AppContinuityOperationPhase::RecoveryRequired;
                        current.cancellable = false;
                        current.reason_code = "portfolio-operation-recovery-required";
                    }
                    Err(error) => {
                        current.phase = AppContinuityOperationPhase::Failed;
                        current.cancellable = false;
                        current.reason_code = error.reason_code();
                    }
                });
            });
        if spawn.is_err() {
            update_continuity_record(&record, |current| {
                current.phase = AppContinuityOperationPhase::Failed;
                current.cancellable = false;
                current.reason_code = "continuity-operation-spawn-failed";
            });
        }
        Ok(queued)
    }

    fn poll_continuity_operation(
        &self,
        operation_id: &str,
    ) -> Result<DesktopContinuityPoll, &'static str> {
        let operation = self
            .continuity_operations
            .get(operation_id)
            .ok_or("continuity-operation-not-found")?;
        let record = operation
            .record
            .lock()
            .map_err(|_| "continuity-operation-lock-failed")?;
        if let Some(result) = record.result.clone() {
            Ok(DesktopContinuityPoll::Completed(result))
        } else {
            Ok(DesktopContinuityPoll::Progress(record.progress()))
        }
    }

    fn cancel_continuity_operation(
        &self,
        operation_id: &str,
    ) -> Result<DesktopContinuityPoll, &'static str> {
        let operation = self
            .continuity_operations
            .get(operation_id)
            .ok_or("continuity-operation-not-found")?;
        let mut record = operation
            .record
            .lock()
            .map_err(|_| "continuity-operation-lock-failed")?;
        if let Some(result) = record.result.clone() {
            return Ok(DesktopContinuityPoll::Completed(result));
        }
        if matches!(
            record.phase,
            AppContinuityOperationPhase::Queued | AppContinuityOperationPhase::Running
        ) {
            operation.cancellation.cancel();
            record.cancellable = false;
            record.reason_code = "portfolio-operation-cancellation-requested";
        }
        Ok(DesktopContinuityPoll::Progress(record.progress()))
    }

    fn select_register_root(&mut self, root: PathBuf) -> Result<(String, String), &'static str> {
        let root = resolve_selected_article_project_root(root)?;
        let token = project_app_token()?;
        let root_label = project_app_root_label(&root);
        self.selected_location = Some(SelectedProjectLocation::Register {
            token: token.clone(),
            root,
        });
        Ok((token, root_label))
    }

    fn select_create_root(&mut self, root: PathBuf) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&root);
        self.selected_location = Some(SelectedProjectLocation::Create {
            token: token.clone(),
            root,
        });
        Ok((token, root_label))
    }

    fn select_export_destination(
        &mut self,
        project_id: ProjectId,
        destination: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&destination);
        self.selected_location = Some(SelectedProjectLocation::Export {
            token: token.clone(),
            project_id,
            destination,
        });
        Ok((token, root_label))
    }

    fn select_import_locations(
        &mut self,
        source: PathBuf,
        destination: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&destination);
        self.selected_location = Some(SelectedProjectLocation::Import {
            token: token.clone(),
            source,
            destination,
        });
        Ok((token, root_label))
    }

    fn select_migration_locations(
        &mut self,
        source: PathBuf,
        destination: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&destination);
        self.selected_location = Some(SelectedProjectLocation::Migration {
            token: token.clone(),
            source,
            destination,
        });
        Ok((token, root_label))
    }

    fn select_migration_recovery_locations(
        &mut self,
        source: PathBuf,
        destination: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&destination);
        self.selected_location = Some(SelectedProjectLocation::MigrationRecovery {
            token: token.clone(),
            source,
            destination,
        });
        Ok((token, root_label))
    }

    fn select_migration_rollback_locations(
        &mut self,
        source: PathBuf,
        destination: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let root_label = project_app_root_label(&destination);
        self.selected_location = Some(SelectedProjectLocation::MigrationRollback {
            token: token.clone(),
            source,
            destination,
        });
        Ok((token, root_label))
    }

    fn select_capture_file(
        &mut self,
        project_id: ProjectId,
        source: PathBuf,
    ) -> Result<(String, String), &'static str> {
        let token = project_app_token()?;
        let file_label = project_app_root_label(&source);
        self.selected_location = Some(SelectedProjectLocation::CaptureIntake {
            token: token.clone(),
            project_id,
            source,
        });
        Ok((token, file_label))
    }

    fn preview_create(
        &mut self,
        directory_token: &str,
        display_name: String,
        project_kind: ProjectKind,
        stage: ProjectStage,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Create { token, root }) = self.selected_location.take()
        else {
            return Err("project-create-destination-selection-invalid");
        };
        if token != directory_token {
            return Err("project-create-destination-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_create(
                root,
                ProjectRegistrationOptions::new(display_name, project_kind).with_stage(stage),
                now_unix()?,
            )
            .map_err(|error| error.reason_code())?;
        self.store_preview(plan)
    }

    fn preview_register(
        &mut self,
        directory_token: &str,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Register { token, root }) = self.selected_location.take()
        else {
            return Err("project-directory-selection-invalid");
        };
        if token != directory_token {
            return Err("project-directory-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_register(root, ProjectRegistrationOptions::existing(), now_unix()?)
            .map_err(|error| error.reason_code())?;
        self.store_preview(plan)
    }

    fn preview_export(
        &mut self,
        directory_token: &str,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Export {
            token,
            project_id,
            destination,
        }) = self.selected_location.take()
        else {
            return Err("project-export-destination-selection-invalid");
        };
        if token != directory_token {
            return Err("project-export-destination-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_export(&project_id, destination)
            .map_err(|error| error.reason_code())?;
        self.store_portable_preview(plan)
    }

    fn preview_import(
        &mut self,
        directory_token: &str,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Import {
            token,
            source,
            destination,
        }) = self.selected_location.take()
        else {
            return Err("project-import-location-selection-invalid");
        };
        if token != directory_token {
            return Err("project-import-location-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_import(source, destination)
            .map_err(|error| error.reason_code())?;
        self.store_portable_preview(plan)
    }

    fn preview_migration(
        &mut self,
        directory_token: &str,
        display_name: String,
        project_kind: ProjectKind,
        stage: ProjectStage,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::Migration {
            token,
            source,
            destination,
        }) = self.selected_location.take()
        else {
            return Err("project-migration-location-selection-invalid");
        };
        if token != directory_token {
            return Err("project-migration-location-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_migrate(
                source,
                destination,
                ProjectRegistrationOptions::new(display_name, project_kind).with_stage(stage),
                now_unix()?,
            )
            .map_err(|error| error.reason_code())?;
        self.store_migration_preview(plan)
    }

    fn preview_migration_recovery(
        &mut self,
        directory_token: &str,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::MigrationRecovery {
            token,
            source,
            destination,
        }) = self.selected_location.take()
        else {
            return Err("project-migration-recovery-location-selection-invalid");
        };
        if token != directory_token {
            return Err("project-migration-recovery-location-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_migration_recovery(source, destination)
            .map_err(|error| error.reason_code())?;
        self.store_migration_recovery_preview(plan)
    }

    fn preview_migration_rollback(
        &mut self,
        directory_token: &str,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let Some(SelectedProjectLocation::MigrationRollback {
            token,
            source,
            destination,
        }) = self.selected_location.take()
        else {
            return Err("project-migration-rollback-location-selection-invalid");
        };
        if token != directory_token {
            return Err("project-migration-rollback-location-selection-invalid");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_migration_rollback(source, destination)
            .map_err(|error| error.reason_code())?;
        self.store_migration_rollback_preview(plan)
    }

    fn preview_capture_intake(
        &mut self,
        file_token: &str,
    ) -> Result<(CaptureIntakePreviewV1, AppOperationPreview), &'static str> {
        let Some(SelectedProjectLocation::CaptureIntake {
            token,
            project_id,
            source,
        }) = self.selected_location.take()
        else {
            return Err("capture-file-selection-invalid");
        };
        if token != file_token {
            return Err("capture-file-selection-invalid");
        }
        let file_label = project_app_root_label(&source);
        let capture = read_portable_capture_packet(source).map_err(|error| error.reason_code())?;
        if capture.binding.project_id != project_id {
            return Err("capture-project-mismatch");
        }
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_capture(capture)
            .map_err(|error| error.reason_code())?;
        self.store_capture_preview(plan, file_label)
    }

    fn store_capture_preview(
        &mut self,
        plan: VerifiedCaptureIntake,
        file_label: String,
    ) -> Result<(CaptureIntakePreviewV1, AppOperationPreview), &'static str> {
        let intake = plan.preview().clone();
        let token = project_app_token()?;
        let preview = app_capture_intake_operation_preview(token.clone(), file_label, &intake);
        self.pending = Some(PendingProjectOperation::CaptureIntake {
            token,
            plan: Box::new(plan),
        });
        Ok((intake, preview))
    }

    fn preview_capture_consolidation(
        &mut self,
        project_id: &ProjectId,
        capture_id: &CaptureId,
    ) -> Result<(CaptureConsolidationPreviewV1, AppOperationPreview), &'static str> {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = service
            .preview_capture_consolidation(project_id, capture_id, now_unix()?)
            .map_err(|error| error.reason_code())?;
        let consolidation = plan.preview().clone();
        let token = project_app_token()?;
        let preview = app_capture_consolidation_operation_preview(token.clone(), &consolidation);
        self.pending = Some(PendingProjectOperation::CaptureConsolidation {
            token,
            plan: Box::new(plan),
        });
        Ok((consolidation, preview))
    }

    fn resolve_root(
        &self,
        project_id: &ProjectId,
    ) -> Result<qiongli_project::RegisteredProjectRoot, &'static str> {
        self.service
            .as_ref()
            .ok_or("project-service-unavailable")?
            .resolve_project_root(project_id)
            .map_err(|error| error.reason_code())
    }

    fn registered_project_skills_target(
        &self,
        project_id: &ProjectId,
    ) -> Result<(qiongli_project::RegisteredProjectRoot, u64, u64), &'static str> {
        let service = self
            .service
            .as_ref()
            .ok_or("project-skills-project-service-unavailable")?;
        let snapshot = service.snapshot().map_err(|error| error.reason_code())?;
        let project = snapshot
            .projects
            .iter()
            .find(|project| &project.project_id == project_id)
            .ok_or("project-skills-project-not-registered")?;
        if project.lifecycle != ProjectLifecycle::Active {
            return Err("project-skills-project-archived");
        }
        if project.health != ProjectHealth::Ready {
            return Err("project-skills-project-not-ready");
        }
        let expected_project_revision = project.semantic_revision;
        let root = service
            .resolve_project_root(project_id)
            .map_err(|error| error.reason_code())?;
        Ok((root, snapshot.revision, expected_project_revision))
    }

    fn preview_lifecycle(
        &mut self,
        project_id: &ProjectId,
        operation: ProjectMutationKind,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let service = self.service.as_ref().ok_or("project-service-unavailable")?;
        let plan = match operation {
            ProjectMutationKind::Archive => service.preview_archive(project_id),
            ProjectMutationKind::Restore => service.preview_restore(project_id),
            ProjectMutationKind::Refresh => service.preview_refresh(project_id, now_unix()?),
            ProjectMutationKind::RepairManifest => service.preview_repair_manifest(project_id),
            ProjectMutationKind::Unregister => service.preview_unregister(project_id),
            ProjectMutationKind::Register | ProjectMutationKind::Create => {
                return Err("project-operation-invalid");
            }
        }
        .map_err(|error| error.reason_code())?;
        self.store_preview(plan)
    }

    fn store_preview(
        &mut self,
        plan: VerifiedProjectMutation,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let token = project_app_token()?;
        let preview = app_project_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::Mutation { token, plan });
        Ok(preview)
    }

    fn store_portable_preview(
        &mut self,
        plan: VerifiedPortableProjectOperation,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let token = project_app_token()?;
        let preview = app_portable_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::Portable { token, plan });
        Ok(preview)
    }

    fn store_migration_preview(
        &mut self,
        plan: VerifiedProjectMigration,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let token = project_app_token()?;
        let preview = app_project_migration_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::Migration { token, plan });
        Ok(preview)
    }

    fn store_migration_recovery_preview(
        &mut self,
        plan: VerifiedProjectMigrationRecovery,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let token = project_app_token()?;
        let preview =
            app_project_migration_recovery_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::MigrationRecovery { token, plan });
        Ok(preview)
    }

    fn store_migration_rollback_preview(
        &mut self,
        plan: VerifiedProjectMigrationRollback,
    ) -> Result<crate::desktop_api::AppOperationPreview, &'static str> {
        let token = project_app_token()?;
        let preview =
            app_project_migration_rollback_operation_preview(token.clone(), plan.preview());
        self.pending = Some(PendingProjectOperation::MigrationRollback { token, plan });
        Ok(preview)
    }

    fn confirm(&mut self, token: &str) -> Option<Result<ConfirmedProjectOperation, &'static str>> {
        if self.pending.as_ref().map(PendingProjectOperation::token) != Some(token) {
            return None;
        }
        let pending = self.pending.take().expect("pending token checked above");
        let result = (|| {
            let service = self.service.clone().ok_or("project-service-unavailable")?;
            match pending {
                PendingProjectOperation::Guidance {
                    project_id,
                    expected_sha256,
                    content,
                    ..
                } => {
                    service
                        .replace_local_guidance(&project_id, expected_sha256.as_deref(), &content)
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: "project-guidance-updated",
                        capture_project_id: None,
                        continuity: None,
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::Mutation { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let commit = service
                        .apply(
                            &plan,
                            &ApprovedProjectMutation::new(digest, true),
                            now_unix()?,
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: project_completion_code(commit.operation),
                        capture_project_id: None,
                        continuity: None,
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::Portable { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let commit = service
                        .apply_portable(
                            &plan,
                            &ApprovedProjectMutation::new(digest, true),
                            now_unix()?,
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: match commit.operation {
                            qiongli_project::PortableProjectOperation::Export => {
                                "project-export-completed"
                            }
                            qiongli_project::PortableProjectOperation::Import => {
                                "project-import-completed"
                            }
                        },
                        capture_project_id: None,
                        continuity: None,
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::Migration { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let commit = service
                        .apply_migration(
                            &plan,
                            &ApprovedProjectMutation::new(digest, true),
                            now_unix()?,
                        )
                        .map_err(|error| error.reason_code())?;
                    let qualification = qualify_project_migration(&service, &commit.project_id);
                    Ok(ConfirmedProjectOperation {
                        code: "project-migration-completed",
                        capture_project_id: None,
                        continuity: None,
                        continuity_operation: None,
                        migration_qualification: Some(qualification),
                    })
                }
                PendingProjectOperation::MigrationRecovery { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let commit = service
                        .apply_migration_recovery(
                            &plan,
                            &ApprovedProjectMutation::new(digest, true),
                        )
                        .map_err(|error| error.reason_code())?;
                    let qualification = qualify_project_migration(&service, &commit.project_id);
                    Ok(ConfirmedProjectOperation {
                        code: "project-migration-recovered",
                        capture_project_id: None,
                        continuity: None,
                        continuity_operation: None,
                        migration_qualification: Some(qualification),
                    })
                }
                PendingProjectOperation::MigrationRollback { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    service
                        .apply_migration_rollback(
                            &plan,
                            &ApprovedProjectMutation::new(digest, true),
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: "project-migration-rolled-back",
                        capture_project_id: None,
                        continuity: None,
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::CaptureIntake { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let project_id = plan.preview().project_id.clone();
                    service
                        .apply_capture(
                            &plan,
                            &ApprovedCaptureIntake::new(digest, true),
                            now_unix()?,
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: "capture-intake-completed",
                        capture_project_id: Some(project_id),
                        continuity: None,
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::CaptureConsolidation { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let project_id = plan.preview().project_id.clone();
                    service
                        .apply_capture_consolidation(
                            &plan,
                            &ApprovedCaptureConsolidation::new(digest, true, true),
                        )
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: "capture-consolidation-completed",
                        capture_project_id: Some(project_id),
                        continuity: None,
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::CaptureDeliveryAcknowledgement { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let project_id = plan.preview().destination_project_id.clone();
                    let delivery = service
                        .apply_capture_delivery_acknowledgement(
                            &plan,
                            &ApprovedCaptureDeliveryAcknowledgement::new(digest, true),
                        )
                        .map(app_capture_delivery_view)
                        .map_err(|error| error.reason_code())?;
                    Ok(ConfirmedProjectOperation {
                        code: "capture-delivery-acknowledged",
                        capture_project_id: Some(project_id),
                        continuity: Some(ConfirmedCaptureContinuity::Delivery(delivery)),
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::CaptureAssignment { plan, .. } => {
                    let digest = plan.preview().plan_digest.clone();
                    let project_id = plan.preview().target_project_id.clone();
                    let commit = service
                        .apply_capture_assignment(
                            &plan,
                            &ApprovedCaptureAssignment::new(digest, true),
                        )
                        .map_err(|error| error.reason_code())?;
                    let status = service
                        .inspect_capture_assignment(&commit.intent_id)
                        .map_err(|error| error.reason_code())?
                        .ok_or("capture-assignment-not-found")?;
                    let library = service.snapshot().map_err(|error| error.reason_code())?;
                    let resolvable = Self::resolvable_assignment_receipt_ids(
                        &service,
                        &library,
                        std::slice::from_ref(&status),
                    )?;
                    let can_resolve = status
                        .receipt_id
                        .as_ref()
                        .is_some_and(|receipt_id| resolvable.contains(receipt_id.as_str()));
                    let assignment = app_capture_assignment_view(status, can_resolve);
                    Ok(ConfirmedProjectOperation {
                        code: "capture-assignment-completed",
                        capture_project_id: Some(project_id),
                        continuity: Some(ConfirmedCaptureContinuity::Assignment(assignment)),
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::CaptureResolution {
                    plan, selections, ..
                } => {
                    let digest = plan.preview().plan_digest.clone();
                    let selection_digest = selections.selection_digest.clone();
                    let project_id = plan.preview().target_project_id.clone();
                    let resolved_at_unix = now_unix()?.max(plan.preview().reviewed_at_unix);
                    let commit = service
                        .apply_capture_resolution(
                            &plan,
                            &selections,
                            &ApprovedCaptureResolution::new(digest, selection_digest, true, true),
                            resolved_at_unix,
                        )
                        .map_err(|error| error.reason_code())?;
                    let resolution = service
                        .inspect_capture_resolution(&project_id, &commit.receipt_id)
                        .map_err(|error| error.reason_code())?
                        .map(app_capture_resolution_view)
                        .ok_or("capture-resolution-not-found")?;
                    Ok(ConfirmedProjectOperation {
                        code: "capture-resolution-completed",
                        capture_project_id: Some(project_id),
                        continuity: Some(ConfirmedCaptureContinuity::Resolution(resolution)),
                        continuity_operation: None,
                        migration_qualification: None,
                    })
                }
                PendingProjectOperation::PortfolioMaintenance { token, plan } => {
                    let progress =
                        self.start_portfolio_maintenance(service.clone(), &token, plan)?;
                    Ok(ConfirmedProjectOperation {
                        code: "portfolio-maintenance-started",
                        capture_project_id: None,
                        continuity: None,
                        continuity_operation: Some(progress),
                        migration_qualification: None,
                    })
                }
            }
        })();
        Some(result)
    }

    fn cancel(&mut self, token: &str) -> bool {
        if self.pending.as_ref().map(PendingProjectOperation::token) != Some(token) {
            return false;
        }
        self.pending = None;
        true
    }
}

impl PendingProjectOperation {
    fn token(&self) -> &str {
        match self {
            Self::Guidance { token, .. }
            | Self::Mutation { token, .. }
            | Self::Portable { token, .. }
            | Self::Migration { token, .. }
            | Self::MigrationRecovery { token, .. }
            | Self::MigrationRollback { token, .. }
            | Self::CaptureIntake { token, .. }
            | Self::CaptureConsolidation { token, .. }
            | Self::CaptureDeliveryAcknowledgement { token, .. }
            | Self::CaptureAssignment { token, .. }
            | Self::CaptureResolution { token, .. }
            | Self::PortfolioMaintenance { token, .. } => token,
        }
    }
}

impl DesktopContinuityOperationRecord {
    fn progress(&self) -> AppContinuityOperationProgressV1 {
        app_continuity_operation_progress(
            self.operation_id.clone(),
            self.operation,
            self.phase,
            self.completed_units,
            self.total_units,
            self.catalog_id.clone(),
            self.cancellable,
            self.reason_code,
        )
    }
}

fn update_continuity_record(
    record: &Arc<Mutex<DesktopContinuityOperationRecord>>,
    update: impl FnOnce(&mut DesktopContinuityOperationRecord),
) {
    if let Ok(mut current) = record.lock() {
        update(&mut current);
    }
}

fn portfolio_continuity_operation_id(token: &str, plan_digest: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"qiongli-desktop-continuity-operation-v1\0");
    hasher.update(token.as_bytes());
    hasher.update([0]);
    hasher.update(plan_digest.as_bytes());
    format!("cop_{:x}", hasher.finalize())
}

fn qualify_project_migration(
    service: &ProjectStateService,
    project_id: &ProjectId,
) -> AppProjectMigrationQualification {
    let index = AcademicGraphIndexService::new(service.clone());
    let first = match index.rebuild(project_id) {
        Ok(index) => index,
        Err(error) => {
            return AppProjectMigrationQualification::rebuild_required(
                project_id.clone(),
                error.reason_code(),
            );
        }
    };
    let second = match index.rebuild(project_id) {
        Ok(index) => index,
        Err(error) => {
            return AppProjectMigrationQualification::rebuild_required(
                project_id.clone(),
                error.reason_code(),
            );
        }
    };
    if first.index_id != second.index_id
        || first.projection_id != second.projection_id
        || first.projection_digest != second.projection_digest
        || first.project_id != second.project_id
        || first.project_revision != second.project_revision
        || first.project_semantic_digest != second.project_semantic_digest
        || first.node_count != second.node_count
        || first.edge_count != second.edge_count
    {
        return AppProjectMigrationQualification::rebuild_required(
            project_id.clone(),
            "project-migration-graph-rebuild-nondeterministic",
        );
    }
    AppProjectMigrationQualification::verified(
        project_id.clone(),
        first.projection_id,
        first.index_id,
    )
}

fn project_app_token() -> Result<String, &'static str> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "project-random-unavailable")?;
    let mut token = String::with_capacity(32);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut token, "{byte:02x}").map_err(|_| "project-token-invalid")?;
    }
    Ok(token)
}

fn project_app_root_label(root: &Path) -> String {
    root.file_name()
        .and_then(|value| value.to_str())
        .filter(|value| {
            !value.is_empty() && value.len() <= 160 && !value.chars().any(char::is_control)
        })
        .unwrap_or("Article project")
        .to_owned()
}

fn resolve_selected_article_project_root(selected: PathBuf) -> Result<PathBuf, &'static str> {
    if selected
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("RESEARCH"))
    {
        return Ok(selected);
    }

    let research_root = if selected
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("RESEARCH"))
    {
        selected.clone()
    } else {
        selected.join("RESEARCH")
    };
    let metadata = match std::fs::symlink_metadata(&research_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(selected),
        Err(_) => return Err("article-project-discovery-failed"),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("article-project-research-root-unsafe");
    }

    let entries =
        std::fs::read_dir(&research_root).map_err(|_| "article-project-discovery-failed")?;
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| "article-project-discovery-failed")?;
        let file_type = entry
            .file_type()
            .map_err(|_| "article-project-discovery-failed")?;
        if file_type.is_dir() && !file_type.is_symlink() {
            candidates.push(entry.path());
        }
    }
    candidates.sort();
    match candidates.as_slice() {
        [root] => Ok(root.clone()),
        [] => Err("article-project-not-found-under-research"),
        _ => Err("multiple-article-projects-found-select-topic"),
    }
}

fn article_project_root_in_workspace(workspace: &Path, suggested_name: &str) -> PathBuf {
    if workspace
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("RESEARCH"))
    {
        workspace.join(suggested_name)
    } else {
        workspace.join("RESEARCH").join(suggested_name)
    }
}

fn validate_project_dialog_name(value: &str) -> Result<(), &'static str> {
    if value.is_empty()
        || value.len() > 160
        || value.trim() != value
        || value.chars().any(char::is_control)
        || value.contains(['/', '\\'])
    {
        Err("project-name-invalid")
    } else {
        Ok(())
    }
}

const fn project_completion_code(operation: ProjectMutationKind) -> &'static str {
    match operation {
        ProjectMutationKind::Register => "project-registration-completed",
        ProjectMutationKind::Create => "project-creation-completed",
        ProjectMutationKind::RepairManifest => "project-manifest-repair-completed",
        ProjectMutationKind::Archive => "project-archive-completed",
        ProjectMutationKind::Restore => "project-restore-completed",
        ProjectMutationKind::Refresh => "project-refresh-completed",
        ProjectMutationKind::Unregister => "project-unregister-completed",
    }
}

pub(crate) fn app_snapshot_json(
    environment: &CommandEnvironment,
    expected_content: &EmbeddedContent,
) -> Result<String, &'static str> {
    let content = crate::embedded_content().map_err(|_| "desktop-content-load-failed")?;
    if content.pack().pack_sha256() != expected_content.pack().pack_sha256() {
        return Err("desktop-content-identity-mismatch");
    }
    let mut environment = environment.clone();
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    let product_control = running_packaged_product(&environment, &content);
    let mut service = NativeDesktopService::new_with_packaged_product(
        environment,
        content,
        Vec::new(),
        product_control,
    );
    let project_skills =
        app_project_skills_targets(&service.environment, &project_service, &service.content);
    let snapshot = AppSnapshotV1::from_desktop(
        service.snapshot(),
        project_snapshot(&project_service),
        project_skills,
    )?;
    serde_json::to_string_pretty(&snapshot)
        .map(|rendered| format!("{rendered}\n"))
        .map_err(|_| "app-snapshot-serialization-failed")
}

pub(crate) fn app_read_project_artifact_json(
    environment: &CommandEnvironment,
    expected_content: &EmbeddedContent,
    project_id: &ProjectId,
    expected_project_revision: u64,
    expected_projection_id: &str,
    entity_kind: AcademicGraphEntityKind,
    entity_id: &str,
) -> Result<String, &'static str> {
    let content = crate::embedded_content().map_err(|_| "desktop-content-load-failed")?;
    if content.pack().pack_sha256() != expected_content.pack().pack_sha256() {
        return Err("desktop-content-identity-mismatch");
    }
    let projects = project_state_service(environment).ok_or("project-service-unavailable")?;
    let artifact = AcademicGraphService::new(projects)
        .read_graph_artifact(
            project_id,
            expected_project_revision,
            expected_projection_id,
            entity_kind,
            entity_id,
            64 * 1_024,
        )
        .map_err(|error| error.reason_code())?;
    serde_json::to_string_pretty(&AppEvent::ProjectArtifactRead { artifact })
        .map(|rendered| format!("{rendered}\n"))
        .map_err(|_| "app-event-serialization-failed")
}

pub(crate) fn app_verify_integrations_json(
    environment: &CommandEnvironment,
    expected_content: &EmbeddedContent,
    codex: bool,
    claude_code: bool,
) -> Result<String, &'static str> {
    if !codex && !claude_code {
        return Err("integration-selection-required");
    }
    app_read_only_event_json(
        environment,
        expected_content,
        DesktopIntent::VerifyIntegrations {
            selection: IntegrationSelection { codex, claude_code },
        },
    )
}

pub(crate) fn app_verify_skills_json(
    environment: &CommandEnvironment,
    expected_content: &EmbeddedContent,
    qiongli_managed: bool,
) -> Result<String, &'static str> {
    app_read_only_event_json(
        environment,
        expected_content,
        DesktopIntent::VerifySkillsPreset {
            preset: if qiongli_managed {
                SkillsDestinationPreset::QiongliManaged
            } else {
                SkillsDestinationPreset::CurrentProject
            },
        },
    )
}

pub(crate) fn app_verify_managed_skills_target_json(
    environment: &CommandEnvironment,
    expected_content: &EmbeddedContent,
    target_id: String,
) -> Result<String, &'static str> {
    app_read_only_event_json(
        environment,
        expected_content,
        DesktopIntent::VerifyManagedSkillsTarget { target_id },
    )
}

fn app_read_only_event_json(
    environment: &CommandEnvironment,
    expected_content: &EmbeddedContent,
    intent: DesktopIntent,
) -> Result<String, &'static str> {
    let content = crate::embedded_content().map_err(|_| "desktop-content-load-failed")?;
    if content.pack().pack_sha256() != expected_content.pack().pack_sha256() {
        return Err("desktop-content-identity-mismatch");
    }
    let mut environment = environment.clone();
    environment.detect_client_versions();
    let project_service = project_state_service(&environment);
    let product_control = running_packaged_product(&environment, &content);
    let mut service = NativeDesktopService::new_with_packaged_product(
        environment,
        content,
        Vec::new(),
        product_control,
    );
    let event = service.execute(intent);
    let current_snapshot = service.snapshot();
    let project_skills =
        app_project_skills_targets(&service.environment, &project_service, &service.content);
    let event = app_event(
        event,
        current_snapshot,
        project_snapshot(&project_service),
        project_skills,
    )?;
    serde_json::to_string_pretty(&event)
        .map(|rendered| format!("{rendered}\n"))
        .map_err(|_| "app-event-serialization-failed")
}

/// Generates the deterministic, path-free Rust side of the frontend IPC contract gate.
///
/// This intentionally uses an empty environment rather than process discovery so the
/// fixture cannot depend on user configuration, installed clients, or network state.
#[doc(hidden)]
pub fn app_api_contract_fixture_json() -> Result<String, &'static str> {
    let content = crate::embedded_content().map_err(|_| "desktop-content-load-failed")?;
    let mut service = NativeDesktopService::new(CommandEnvironment::default(), content, Vec::new());
    let research_library = ResearchLibrarySnapshotV1 {
        schema_version: qiongli_project::RESEARCH_LIBRARY_SCHEMA_VERSION,
        revision: 0,
        health: LibraryHealth::Empty,
        projects: Vec::new(),
    };
    let snapshot = AppSnapshotV1::from_desktop(service.snapshot(), research_library, Vec::new())?;
    serialize_app_api_contract_fixture(snapshot)
}

pub(crate) fn validate_desktop_startup(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<(), DesktopLaunchError> {
    let _window_entrypoint: fn(
        CommandEnvironment,
        EmbeddedContent,
    ) -> Result<(), DesktopLaunchError> = run_desktop;
    let owned_content = crate::embedded_content().map_err(|_| DesktopLaunchError)?;
    if owned_content.pack().pack_sha256() != content.pack().pack_sha256() {
        return Err(DesktopLaunchError);
    }
    let mut detected_environment = environment.clone().without_project_context();
    detected_environment.detect_client_versions();
    let project_service = project_state_service(&detected_environment);
    let mut service = NativeDesktopService::new(detected_environment, owned_content, Vec::new());
    service
        .snapshot()
        .validate()
        .map_err(|_| DesktopLaunchError)?;
    let project_skills =
        app_project_skills_targets(&service.environment, &project_service, &service.content);
    AppSnapshotV1::from_desktop(
        service.snapshot(),
        project_snapshot(&project_service),
        project_skills,
    )
    .map_err(|_| DesktopLaunchError)?;
    Ok(())
}

fn project_state_service(environment: &CommandEnvironment) -> Option<ProjectStateService> {
    crate::command::config_root(environment)
        .ok()
        .map(ProjectStateService::new)
}

fn project_snapshot(service: &Option<ProjectStateService>) -> ResearchLibrarySnapshotV1 {
    service
        .as_ref()
        .and_then(|service| service.snapshot().ok())
        .unwrap_or(ResearchLibrarySnapshotV1 {
            schema_version: qiongli_project::RESEARCH_LIBRARY_SCHEMA_VERSION,
            revision: 0,
            health: LibraryHealth::InspectionBlocked,
            projects: Vec::new(),
        })
}

fn app_project_skills_targets(
    environment: &CommandEnvironment,
    service: &Option<ProjectStateService>,
    content: &EmbeddedContent,
) -> Vec<AppProjectSkillsTargetView> {
    let Some(service) = service.as_ref() else {
        return Vec::new();
    };
    let Ok(root) = config_root(environment) else {
        return Vec::new();
    };
    let Ok(registry) = crate::managed_content::load_managed_content_registry(root.state_root())
    else {
        return Vec::new();
    };
    let Ok(variant) = WorkflowVariantStore::new(root).load(content.pack()) else {
        return Vec::new();
    };
    let Ok(project_roots) = service.resolvable_project_roots() else {
        return Vec::new();
    };
    let mut targets = project_roots
        .into_iter()
        .map(|(project_id, root)| {
            let target = root
                .path()
                .join(".qiongli-skills")
                .to_string_lossy()
                .into_owned();
            let destination = registry
                .entries
                .binary_search_by(|entry| entry.target.cmp(&target))
                .ok()
                .map(|index| {
                    managed_skills_entry_view(
                        &registry.entries[index],
                        SkillsDestinationPreset::CurrentProject,
                        content,
                        variant.variant_sha256(),
                    )
                })
                .unwrap_or_else(|| {
                    let (state, status) = unregistered_managed_skills_state(Path::new(&target));
                    ManagedSkillsView {
                        target_id: managed_skills_target_id(&target),
                        preset: SkillsDestinationPreset::CurrentProject,
                        state,
                        status,
                        profile: None,
                        product_version: None,
                    }
                });
            AppProjectSkillsTargetView {
                project_id: project_id.as_str().to_owned(),
                destination,
            }
        })
        .collect::<Vec<_>>();
    targets.sort_by(|left, right| left.destination.target_id.cmp(&right.destination.target_id));
    targets
}

fn apply_app_project_skills_preview_target(
    mut event: DesktopEvent,
    target_id: Option<&str>,
    project_skills: &[AppProjectSkillsTargetView],
) -> DesktopEvent {
    let is_registered_project_target = target_id.is_some_and(|target_id| {
        project_skills
            .iter()
            .any(|project| project.destination.target_id == target_id)
    });
    if !is_registered_project_target {
        return event;
    }
    if let DesktopEvent::PreviewReady(preview) = &mut event
        && matches!(
            preview.kind,
            OperationKind::SkillsMaterialization
                | OperationKind::SkillsRemoval
                | OperationKind::SkillsDetach
        )
    {
        preview.display_target = Some(PrivateDisplayText::new(
            "<project>/.qiongli-skills".to_owned(),
        ));
    }
    event
}

pub struct DesktopActivationSession {
    target: IntegrationTarget,
    handle: ClientActivationHandle,
    grant: VerifiedLaunchGrant,
    trusted_keys: Vec<TrustedPublicKey>,
    minimum_generation: u64,
    pending: Option<ClientActivationPreview>,
}

impl DesktopActivationSession {
    #[must_use]
    pub fn new(
        handle: ClientActivationHandle,
        grant: VerifiedLaunchGrant,
        trusted_keys: Vec<TrustedPublicKey>,
        minimum_generation: u64,
    ) -> Self {
        let target = integration_target(handle.target());
        Self {
            target,
            handle,
            grant,
            trusted_keys,
            minimum_generation,
            pending: None,
        }
    }

    fn preview(
        &mut self,
        token: OperationToken,
        now_unix: u64,
    ) -> Result<OperationPreview, &'static str> {
        let expires_at_unix = now_unix
            .saturating_add(ACTIVATION_PLAN_TTL_SECONDS)
            .min(self.grant.grant().expires_at_unix);
        let plan_id = match self.target {
            IntegrationTarget::Codex => "desktop-activate-codex",
            IntegrationTarget::ClaudeCode => "desktop-activate-claude-code",
        };
        let preview = preview_client_activation(
            &self.handle,
            InstallPlanMetadataV1 {
                plan_id: plan_id.to_owned(),
                created_at_unix: now_unix,
                expires_at_unix,
            },
            &self.grant,
            &self.trusted_keys,
            self.minimum_generation,
            now_unix,
        )
        .map_err(|error| error.reason_code())?;
        let digest = preview.plan().plan().semantic_digest_sha256.clone();
        self.pending = Some(preview);
        Ok(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: match self.target {
                IntegrationTarget::Codex => "Codex activation preview",
                IntegrationTarget::ClaudeCode => "Claude Code activation preview",
            },
            summary: "Register the verified local Qiongli source. Client-owned enablement remains a host action.",
            display_target: Some(integration_display_target(self.target)),
            plan_digest_sha256: Some(digest),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm(&mut self, now_unix: u64) -> Result<&'static str, &'static str> {
        let preview = self
            .pending
            .take()
            .ok_or("desktop-activation-preview-missing")?;
        let approval = approve_install_plan(preview.plan(), &ACTIVATION_APPROVALS, now_unix)
            .map_err(|error| error.reason_code())?;
        let commit = ClientActivationCoordinator::new(self.handle.clone())
            .apply(&preview, &approval, now_unix)
            .map_err(|error| error.reason_code())?;
        Ok(match commit.disposition {
            ClientActivationDisposition::Activated => "client-activation-applied",
            ClientActivationDisposition::AlreadyActive => "client-activation-already-active",
            ClientActivationDisposition::Repaired => "client-activation-repaired",
            ClientActivationDisposition::AlreadyHealthy => "client-activation-already-healthy",
        })
    }

    fn cancel(&mut self) {
        self.pending = None;
    }
}

impl Debug for DesktopActivationSession {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopActivationSession")
            .field("target", &self.target)
            .field("pending", &self.pending.is_some())
            .finish()
    }
}

pub struct DesktopCandidateSession {
    target: IntegrationTarget,
    candidate: VerifiedNativeReleaseCandidate,
    pending: bool,
}

impl DesktopCandidateSession {
    #[must_use]
    pub fn new(candidate: VerifiedNativeReleaseCandidate) -> Self {
        Self {
            target: integration_target(candidate.target()),
            candidate,
            pending: false,
        }
    }

    fn preview(
        &mut self,
        token: OperationToken,
        now_unix: u64,
    ) -> Result<OperationPreview, &'static str> {
        if now_unix < self.candidate.candidate().not_before_unix {
            return Err("native-release-candidate-not-yet-valid");
        }
        if now_unix >= self.candidate.candidate().expires_at_unix {
            return Err("native-release-candidate-expired");
        }
        self.pending = true;
        Ok(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: match self.target {
                IntegrationTarget::Codex => "Codex candidate installation preview",
                IntegrationTarget::ClaudeCode => "Claude Code candidate installation preview",
            },
            summary: "Install the verified native payload and fixed local Qiongli source, then register it. Client-owned enablement remains a host action.",
            display_target: Some(integration_display_target(self.target)),
            plan_digest_sha256: Some(crate::candidate_cli::candidate_approval_digest(
                self.candidate.signed_payload_sha256(),
                self.candidate.target(),
            )),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm(
        &mut self,
        content: &EmbeddedContent,
        home: &std::path::Path,
        now_unix: u64,
    ) -> Result<&'static str, &'static str> {
        if !std::mem::take(&mut self.pending) {
            return Err("desktop-candidate-preview-missing");
        }
        let commit =
            apply_native_release_candidate_local(content.pack(), &self.candidate, home, now_unix)
                .map_err(|error| error.reason_code())?;
        Ok(match commit.payload.disposition {
            qiongli_platform::InstallDisposition::Applied => "native-candidate-install-applied",
            qiongli_platform::InstallDisposition::AlreadyApplied => {
                "native-candidate-install-already-applied"
            }
            qiongli_platform::InstallDisposition::Repaired => "native-candidate-install-repaired",
            qiongli_platform::InstallDisposition::AlreadyHealthy => {
                "native-candidate-install-already-healthy"
            }
        })
    }

    fn cancel(&mut self) {
        self.pending = false;
    }
}

impl Debug for DesktopCandidateSession {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopCandidateSession")
            .field("target", &self.target)
            .field("candidate_digest", &"<verified-candidate>")
            .field("pending", &self.pending)
            .finish()
    }
}

#[derive(Clone, Copy)]
struct McpSelfTestCounts {
    enabled_providers: usize,
    ready_providers: usize,
    discovered_clients: usize,
    registered_clients: usize,
}

struct McpSelfTestInput {
    server: FullMcpServer,
    counts: McpSelfTestCounts,
}

trait McpSelfTestExecutor: Send + Sync {
    fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView;
}

struct NativeMcpSelfTestExecutor;

impl McpSelfTestExecutor for NativeMcpSelfTestExecutor {
    fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView {
        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }

        let initialize = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }));
        let initialize_ready = initialize.as_ref().is_some_and(|response| {
            response
                .pointer("/result/protocolVersion")
                .and_then(|value| value.as_str())
                == Some(MCP_PROTOCOL_VERSION)
                && response.pointer("/result/capabilities/tools").is_some()
        });

        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }
        let tools = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }));
        let tools_ready = tools.as_ref().is_some_and(|response| {
            response
                .pointer("/result/tools")
                .and_then(|value| value.as_array())
                .is_some_and(|tools| {
                    tools.len() == full_public_tool_count()
                        && tools
                            .iter()
                            .zip(full_public_tool_names())
                            .all(|(tool, expected)| {
                                tool.get("name").and_then(|value| value.as_str()) == Some(expected)
                            })
                })
        });

        if cancelled.load(Ordering::Acquire) {
            return terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts);
        }
        let full_dispatch = input.server.handle(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "qiongli_orchestrator_route",
                "arguments": {
                    "request": "run an auditable multi-agent review",
                    "platform": "codex"
                }
            }
        }));
        let full_ready = full_dispatch.as_ref().is_some_and(|response| {
            response
                .pointer("/result/structuredContent")
                .is_some_and(|route| {
                    route.get("route").and_then(|value| value.as_str()) == Some("orchestrator_mcp")
                        && route
                            .get("requires_full_runtime")
                            .and_then(|value| value.as_bool())
                            == Some(true)
                        && route.get("upgrade").is_none()
                        && route.get("preview_only").is_none()
                })
        });

        let provider_check = if input.counts.enabled_providers == 0 {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Attention,
                "no-provider-enabled",
            )
        } else if input.counts.ready_providers == input.counts.enabled_providers {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Ready,
                "enabled-providers-ready",
            )
        } else {
            mcp_check(
                McpSelfTestCheckId::ProviderReadiness,
                StatusCode::Attention,
                "provider-configuration-attention",
            )
        };
        let client_check = if input.counts.discovered_clients == 0 {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Attention,
                "no-client-discovered",
            )
        } else if input.counts.registered_clients == input.counts.discovered_clients {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Ready,
                "discovered-clients-registered",
            )
        } else {
            mcp_check(
                McpSelfTestCheckId::ClientRegistration,
                StatusCode::Attention,
                "client-registration-attention",
            )
        };
        let checks = [
            mcp_check(
                McpSelfTestCheckId::EmbeddedContract,
                StatusCode::Ready,
                "embedded-contract-ready",
            ),
            mcp_check(
                McpSelfTestCheckId::Initialize,
                if initialize_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if initialize_ready {
                    "mcp-initialize-ready"
                } else {
                    "mcp-initialize-invalid"
                },
            ),
            mcp_check(
                McpSelfTestCheckId::ToolRegistry,
                if tools_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if tools_ready {
                    "exact-tool-registry-ready"
                } else {
                    "tool-registry-drifted"
                },
            ),
            mcp_check(
                McpSelfTestCheckId::FullDispatch,
                if full_ready {
                    StatusCode::Ready
                } else {
                    StatusCode::Invalid
                },
                if full_ready {
                    "full-dispatch-ready"
                } else {
                    "full-dispatch-failed"
                },
            ),
            provider_check,
            client_check,
        ];
        let state = if initialize_ready && tools_ready && full_ready {
            McpSelfTestState::Passed
        } else {
            McpSelfTestState::Failed
        };
        mcp_self_test_view(state, checks, input.counts)
    }
}

struct ActiveMcpSelfTest {
    receiver: mpsc::Receiver<McpSelfTestView>,
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
    running: McpSelfTestView,
}

struct ActiveDesktopUpdate {
    receiver: mpsc::Receiver<DesktopUpdateWorkerMessage>,
}

enum DesktopUpdateWorkerMessage {
    Progress(UpdateView),
    Finished(Result<DesktopUpdateOutcome, &'static str>),
}

enum DesktopUpdateOutcome {
    Checked(crate::update_cli::DesktopUpdateCheck),
    Prepared(crate::update_cli::DesktopPreparedUpdate),
    Cancelled(UpdateStreamView),
    InstallHandoff(crate::update_cli::DesktopInstallHandoff),
}

fn mcp_check(
    check: McpSelfTestCheckId,
    status: StatusCode,
    code: &'static str,
) -> McpSelfTestCheckView {
    McpSelfTestCheckView {
        check,
        status,
        code,
        remediation: mcp_check_remediation(check, status),
    }
}

const fn mcp_check_remediation(check: McpSelfTestCheckId, status: StatusCode) -> &'static str {
    if matches!(status, StatusCode::Ready | StatusCode::Missing) {
        return "none";
    }
    match check {
        McpSelfTestCheckId::EmbeddedContract | McpSelfTestCheckId::ToolRegistry => {
            "reinstall-qiongli"
        }
        McpSelfTestCheckId::Initialize => "upgrade-qiongli",
        McpSelfTestCheckId::FullDispatch => "retry-mcp-self-test",
        McpSelfTestCheckId::ProviderReadiness => "configure-enabled-providers",
        McpSelfTestCheckId::ClientRegistration => "refresh-integration-discovery",
    }
}

fn mcp_self_test_view(
    state: McpSelfTestState,
    checks: [McpSelfTestCheckView; 6],
    counts: McpSelfTestCounts,
) -> McpSelfTestView {
    McpSelfTestView {
        state,
        checks,
        public_tool_count: full_public_tool_count(),
        enabled_provider_count: counts.enabled_providers,
        ready_provider_count: counts.ready_providers,
        discovered_client_count: counts.discovered_clients,
        registered_client_count: counts.registered_clients,
    }
}

fn full_public_tool_names() -> impl Iterator<Item = &'static str> {
    LITE_PUBLIC_TOOL_NAMES
        .into_iter()
        .chain(FULL_PROJECT_PUBLIC_TOOL_NAMES)
        .chain(FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES)
}

pub(crate) const fn full_public_tool_count() -> usize {
    LITE_PUBLIC_TOOL_NAMES.len()
        + FULL_PROJECT_PUBLIC_TOOL_NAMES.len()
        + FULL_HOST_ORCHESTRATION_CONTROL_TOOL_NAMES.len()
}

fn pending_mcp_self_test(counts: McpSelfTestCounts) -> McpSelfTestView {
    mcp_self_test_view(
        McpSelfTestState::Running,
        McpSelfTestCheckId::ALL
            .map(|check| mcp_check(check, StatusCode::Missing, "self-test-check-pending")),
        counts,
    )
}

fn terminal_mcp_self_test(state: McpSelfTestState, counts: McpSelfTestCounts) -> McpSelfTestView {
    let (status, code) = match state {
        McpSelfTestState::Cancelled => (StatusCode::Missing, "self-test-cancelled"),
        McpSelfTestState::TimedOut => (StatusCode::Blocked, "self-test-timed-out"),
        McpSelfTestState::Failed => (StatusCode::Invalid, "self-test-failed"),
        McpSelfTestState::Running | McpSelfTestState::Passed => {
            (StatusCode::Invalid, "self-test-state-invalid")
        }
    };
    let mut view = mcp_self_test_view(
        state,
        McpSelfTestCheckId::ALL.map(|check| mcp_check(check, status, code)),
        counts,
    );
    for check in &mut view.checks {
        check.remediation = if state == McpSelfTestState::Cancelled {
            "none"
        } else {
            "retry-mcp-self-test"
        };
    }
    view
}

fn contract_failure_mcp_self_test(counts: McpSelfTestCounts) -> McpSelfTestView {
    let mut view = terminal_mcp_self_test(McpSelfTestState::Failed, counts);
    view.public_tool_count = 0;
    view.checks[0] = mcp_check(
        McpSelfTestCheckId::EmbeddedContract,
        StatusCode::Invalid,
        "embedded-contract-invalid",
    );
    view
}

trait FolderPicker: Send {
    fn pick_folder(&mut self) -> Option<PathBuf>;

    fn pick_project_folder(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_create_destination(&mut self, _suggested_name: &str) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_export_destination(&mut self, _suggested_name: &str) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_import_source(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_import_destination(&mut self, _suggested_name: &str) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_migration_source(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_migration_destination(&mut self, _suggested_name: &str) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_migration_recovery_source(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_migration_recovery_destination(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_migration_rollback_source(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_project_migration_rollback_destination(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }

    fn pick_capture_file(&mut self) -> Option<PathBuf> {
        self.pick_folder()
    }
}

struct NativeFolderPicker;

#[cfg(not(feature = "desktop"))]
impl FolderPicker for NativeFolderPicker {
    fn pick_folder(&mut self) -> Option<PathBuf> {
        None
    }
}

#[cfg(feature = "desktop")]
impl FolderPicker for NativeFolderPicker {
    fn pick_folder(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a Qiongli Skills destination")
            .pick_folder()
    }

    fn pick_project_folder(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a workspace, RESEARCH folder, or article topic folder")
            .pick_folder()
    }

    fn pick_project_create_destination(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose the workspace for the new Qiongli article project")
            .pick_folder()
            .map(|workspace| article_project_root_in_workspace(&workspace, suggested_name))
    }

    fn pick_project_export_destination(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a destination for the portable Qiongli project")
            .set_file_name(suggested_name)
            .save_file()
    }

    fn pick_project_import_source(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a portable Qiongli project package")
            .pick_folder()
    }

    fn pick_project_import_destination(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a location for the imported Qiongli project")
            .set_file_name(suggested_name)
            .save_file()
    }

    fn pick_project_migration_source(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose the Qiongli 1.x project to migrate")
            .pick_folder()
    }

    fn pick_project_migration_destination(&mut self, suggested_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a new location for the Qiongli 2 project")
            .set_file_name(suggested_name)
            .save_file()
    }

    fn pick_project_migration_recovery_source(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose the unchanged Qiongli 1.x migration source")
            .pick_folder()
    }

    fn pick_project_migration_recovery_destination(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose the committed Qiongli 2 migration destination")
            .pick_folder()
    }

    fn pick_project_migration_rollback_source(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose the unchanged Qiongli 1.x source to retain")
            .pick_folder()
    }

    fn pick_project_migration_rollback_destination(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose the exact Qiongli 2 migrated copy to remove")
            .pick_folder()
    }

    fn pick_capture_file(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Choose a portable Qiongli research capture")
            .add_filter("Qiongli research capture", &["json"])
            .pick_file()
    }
}

struct NativeDesktopService {
    environment: CommandEnvironment,
    content: EmbeddedContent,
    active_operation: Option<PendingDesktopOperation>,
    folder_picker: Box<dyn FolderPicker>,
    selected_skills_target: Option<MaterializationTarget>,
    secret_store: Arc<dyn SecretStore>,
    mcp_self_test: Option<ActiveMcpSelfTest>,
    mcp_self_test_executor: Arc<dyn McpSelfTestExecutor>,
    mcp_self_test_timeout: Duration,
    update_view: UpdateView,
    active_update: Option<ActiveDesktopUpdate>,
    activation_sessions: Vec<DesktopActivationSession>,
    candidate_sessions: Vec<DesktopCandidateSession>,
    packaged_product: PackagedProductState,
    host_observations: [HostIntegrationObservation; 2],
    zotero: ZoteroIntegrationView,
    cli_path_test: Option<(CliPathState, &'static str)>,
    direct_backend_experiment_enabled: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum HostProbeState {
    #[default]
    NotRun,
    Observed,
    HostActionRequired,
    CacheDrift,
    CommandFailed,
    ProbeUnavailable,
    ProbeFailed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct HostIntegrationObservation {
    activation: HostProbeState,
    mcp_attachment: HostProbeState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostPluginMode {
    Install,
    Repair,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HostPluginCommandTemplate {
    pub(crate) arguments: &'static [&'static str],
}

const CODEX_HOST_PLUGIN_ID: &str = "qiongli-next@personal";
const CLAUDE_HOST_PLUGIN_ID: &str = "qiongli-next@qiongli-local";
const HOST_MCP_ID: &str = "qiongli-next";
const CLAUDE_SKILL_ID: &str = "qiongli-workflow";
const CLAUDE_MARKETPLACE_DISPLAY_SOURCE: &str = "$HOME/.qiongli/plugins/claude-code/qiongli-local";

const CODEX_PLUGIN_INSTALL_COMMANDS: &[HostPluginCommandTemplate] = &[HostPluginCommandTemplate {
    arguments: &["plugin", "add", "--json", CODEX_HOST_PLUGIN_ID],
}];
const CODEX_PLUGIN_REPAIR_COMMANDS: &[HostPluginCommandTemplate] = &[
    HostPluginCommandTemplate {
        arguments: &["plugin", "remove", "--json", CODEX_HOST_PLUGIN_ID],
    },
    HostPluginCommandTemplate {
        arguments: &["plugin", "add", "--json", CODEX_HOST_PLUGIN_ID],
    },
];
const CLAUDE_PLUGIN_INSTALL_COMMANDS: &[HostPluginCommandTemplate] = &[
    HostPluginCommandTemplate {
        arguments: &[
            "plugin",
            "marketplace",
            "add",
            CLAUDE_MARKETPLACE_DISPLAY_SOURCE,
            "--scope",
            "user",
        ],
    },
    HostPluginCommandTemplate {
        arguments: &[
            "plugin",
            "install",
            CLAUDE_HOST_PLUGIN_ID,
            "--scope",
            "user",
        ],
    },
];
const CLAUDE_PLUGIN_REPAIR_COMMANDS: &[HostPluginCommandTemplate] = &[
    HostPluginCommandTemplate {
        arguments: &[
            "plugin",
            "marketplace",
            "add",
            CLAUDE_MARKETPLACE_DISPLAY_SOURCE,
            "--scope",
            "user",
        ],
    },
    HostPluginCommandTemplate {
        arguments: &[
            "plugin",
            "uninstall",
            CLAUDE_HOST_PLUGIN_ID,
            "--scope",
            "user",
            "--yes",
        ],
    },
    HostPluginCommandTemplate {
        arguments: &[
            "plugin",
            "install",
            CLAUDE_HOST_PLUGIN_ID,
            "--scope",
            "user",
        ],
    },
];

pub(crate) const fn host_plugin_executable_name(target: IntegrationTarget) -> &'static str {
    match target {
        IntegrationTarget::Codex => "codex",
        IntegrationTarget::ClaudeCode => "claude",
    }
}

pub(crate) const fn host_plugin_scope(target: IntegrationTarget) -> &'static str {
    match target {
        IntegrationTarget::Codex => "personal",
        IntegrationTarget::ClaudeCode => "user",
    }
}

pub(crate) const fn host_plugin_command_templates(
    target: IntegrationTarget,
    mode: HostPluginMode,
) -> &'static [HostPluginCommandTemplate] {
    match (target, mode) {
        (IntegrationTarget::Codex, HostPluginMode::Install) => CODEX_PLUGIN_INSTALL_COMMANDS,
        (IntegrationTarget::Codex, HostPluginMode::Repair) => CODEX_PLUGIN_REPAIR_COMMANDS,
        (IntegrationTarget::ClaudeCode, HostPluginMode::Install) => CLAUDE_PLUGIN_INSTALL_COMMANDS,
        (IntegrationTarget::ClaudeCode, HostPluginMode::Repair) => CLAUDE_PLUGIN_REPAIR_COMMANDS,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HostPluginCommand {
    arguments: Vec<OsString>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostPluginPlan {
    target: ClientActivationTarget,
    mode: Option<HostPluginMode>,
    executable: PathBuf,
    commands: Vec<HostPluginCommand>,
    plan_digest_sha256: String,
}

fn host_plugin_mode(
    effect: PackagedProductInstallEffect,
    observation: HostIntegrationObservation,
) -> Result<Option<HostPluginMode>, &'static str> {
    let install_or_repair = || {
        if effect == PackagedProductInstallEffect::Repair {
            HostPluginMode::Repair
        } else {
            HostPluginMode::Install
        }
    };
    match observation.activation {
        HostProbeState::HostActionRequired => Ok(Some(install_or_repair())),
        HostProbeState::CacheDrift | HostProbeState::CommandFailed => {
            Ok(Some(if effect == PackagedProductInstallEffect::Install {
                HostPluginMode::Install
            } else {
                HostPluginMode::Repair
            }))
        }
        HostProbeState::Observed => {
            match observation.mcp_attachment {
                HostProbeState::Observed => Ok((effect == PackagedProductInstallEffect::Repair)
                    .then_some(HostPluginMode::Repair)),
                HostProbeState::HostActionRequired
                | HostProbeState::CacheDrift
                | HostProbeState::CommandFailed => Ok(Some(HostPluginMode::Repair)),
                HostProbeState::NotRun => Err("host-plugin-probe-not-run"),
                HostProbeState::ProbeUnavailable => Err("host-plugin-probe-unavailable"),
                HostProbeState::ProbeFailed => Err("host-plugin-probe-failed"),
            }
        }
        HostProbeState::NotRun => Err("host-plugin-probe-not-run"),
        HostProbeState::ProbeUnavailable => Err("host-plugin-probe-unavailable"),
        HostProbeState::ProbeFailed => Err("host-plugin-probe-failed"),
    }
}

fn prepare_host_plugin_plan(
    environment: &CommandEnvironment,
    target: ClientActivationTarget,
    effect: PackagedProductInstallEffect,
    observation: HostIntegrationObservation,
) -> Result<HostPluginPlan, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let (name, version) = match target {
        ClientActivationTarget::Codex => (
            "codex",
            environment
                .codex_host_version()
                .ok_or("host-plugin-client-unavailable")?,
        ),
        ClientActivationTarget::ClaudeCode => (
            "claude",
            environment
                .claude_host_version()
                .ok_or("host-plugin-client-unavailable")?,
        ),
    };
    let executable = environment
        .client_executable(name)
        .ok_or("host-plugin-executable-unavailable")?;
    let executable = resolve_host_plugin_executable(home, name, &executable)?;
    let config_root = match target {
        ClientActivationTarget::Codex => environment
            .codex_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".codex")),
        ClientActivationTarget::ClaudeCode => environment
            .claude_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".claude")),
    };
    build_host_plugin_plan(
        executable,
        home,
        &config_root,
        target,
        host_plugin_mode(effect, observation)?,
        version,
        observation,
    )
}

fn resolve_host_plugin_executable(
    home: &Path,
    name: &str,
    discovered: &Path,
) -> Result<PathBuf, &'static str> {
    let canonical =
        fs::canonicalize(discovered).map_err(|_| "host-plugin-executable-unavailable")?;
    if canonical.file_name().and_then(OsStr::to_str) != Some("mise") {
        return Ok(canonical);
    }
    let installed = match name {
        "codex" => home.join(".local/share/mise/installs/codex/latest/bin/codex"),
        "claude" => home.join(".local/share/mise/installs/claude-code/latest/claude"),
        _ => return Err("host-plugin-executable-unavailable"),
    };
    fs::canonicalize(installed).map_err(|_| "host-plugin-executable-unavailable")
}

fn build_host_plugin_plan(
    executable: PathBuf,
    home: &Path,
    config_root: &Path,
    target: ClientActivationTarget,
    mode: Option<HostPluginMode>,
    client_version: crate::command::DetectedClientVersion,
    observation: HostIntegrationObservation,
) -> Result<HostPluginPlan, &'static str> {
    let view_target = integration_target(target);
    let marketplace_source = home.join(".qiongli/plugins/claude-code/qiongli-local");
    let commands = mode
        .map(|mode| {
            host_plugin_command_templates(view_target, mode)
                .iter()
                .map(|template| HostPluginCommand {
                    arguments: template
                        .arguments
                        .iter()
                        .map(|argument| {
                            if *argument == CLAUDE_MARKETPLACE_DISPLAY_SOURCE {
                                marketplace_source.as_os_str().to_os_string()
                            } else {
                                OsString::from(argument)
                            }
                        })
                        .collect(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut digest = Sha256::new();
    for field in [
        b"qiongli-host-plugin-plan-v1".as_slice(),
        host_plugin_target_id(target).as_bytes(),
        mode.map_or("verify", host_plugin_mode_id).as_bytes(),
        executable.as_os_str().as_encoded_bytes(),
        home.as_os_str().as_encoded_bytes(),
        config_root.as_os_str().as_encoded_bytes(),
        env!("CARGO_PKG_VERSION").as_bytes(),
        host_plugin_scope(view_target).as_bytes(),
        host_probe_state_id(observation.activation).as_bytes(),
        host_probe_state_id(observation.mcp_attachment).as_bytes(),
    ] {
        update_host_digest_field(&mut digest, field);
    }
    update_host_digest_field(&mut digest, &client_version.major.to_le_bytes());
    update_host_digest_field(&mut digest, &client_version.minor.to_le_bytes());
    update_host_digest_field(&mut digest, &client_version.patch.to_le_bytes());
    for command in &commands {
        update_host_digest_field(&mut digest, b"command");
        for argument in &command.arguments {
            update_host_digest_field(&mut digest, argument.as_encoded_bytes());
        }
    }
    Ok(HostPluginPlan {
        target,
        mode,
        executable,
        commands,
        plan_digest_sha256: format!("{:x}", digest.finalize()),
    })
}

fn update_host_digest_field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value);
}

const fn host_plugin_target_id(target: ClientActivationTarget) -> &'static str {
    match target {
        ClientActivationTarget::Codex => "codex",
        ClientActivationTarget::ClaudeCode => "claude-code",
    }
}

const fn host_plugin_mode_id(mode: HostPluginMode) -> &'static str {
    match mode {
        HostPluginMode::Install => "install",
        HostPluginMode::Repair => "repair",
    }
}

const fn host_probe_state_id(state: HostProbeState) -> &'static str {
    match state {
        HostProbeState::NotRun => "not-run",
        HostProbeState::Observed => "observed",
        HostProbeState::HostActionRequired => "host-action-required",
        HostProbeState::CacheDrift => "cache-drift",
        HostProbeState::CommandFailed => "command-failed",
        HostProbeState::ProbeUnavailable => "probe-unavailable",
        HostProbeState::ProbeFailed => "probe-failed",
    }
}

fn packaged_host_plan_digest(native_digest: &str, plans: &[HostPluginPlan]) -> String {
    let mut digest = Sha256::new();
    update_host_digest_field(&mut digest, b"qiongli-packaged-host-plan-v1");
    update_host_digest_field(&mut digest, native_digest.as_bytes());
    for plan in plans {
        update_host_digest_field(&mut digest, plan.plan_digest_sha256.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

pub(crate) fn prepare_host_plugin_plans(
    environment: &CommandEnvironment,
    installs: &[(ClientActivationTarget, PackagedProductInstallEffect)],
) -> Result<Vec<HostPluginPlan>, &'static str> {
    installs
        .iter()
        .map(|(target, effect)| {
            let observation = match target {
                ClientActivationTarget::Codex => probe_codex_host_result(environment),
                ClientActivationTarget::ClaudeCode => probe_claude_host_result(environment),
            }?;
            prepare_host_plugin_plan(environment, *target, *effect, observation)
        })
        .collect()
}

pub(crate) fn host_plugin_plans_digest(plans: &[HostPluginPlan]) -> String {
    let mut digest = Sha256::new();
    update_host_digest_field(&mut digest, b"qiongli-host-plugin-plan-set-v1");
    for plan in plans {
        update_host_digest_field(&mut digest, plan.plan_digest_sha256.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

pub(crate) fn execute_host_plugin_plan_set(
    environment: &CommandEnvironment,
    plans: &[HostPluginPlan],
) -> Result<(), &'static str> {
    execute_host_plugin_plans(
        environment,
        plans,
        &mut [HostIntegrationObservation::default(); 2],
    )
}

fn execute_host_plugin_plans(
    environment: &CommandEnvironment,
    plans: &[HostPluginPlan],
    observations: &mut [HostIntegrationObservation; 2],
) -> Result<(), &'static str> {
    for plan in plans {
        observations[host_plugin_target_index(plan.target)] = HostIntegrationObservation::default();
    }
    for plan in plans {
        let index = host_plugin_target_index(plan.target);
        for command in &plan.commands {
            if let Err(error) = bounded_host_os_command_with_timeout(
                environment,
                &plan.executable,
                &command.arguments,
                Duration::from_secs(30),
            ) {
                observations[index] = HostIntegrationObservation {
                    activation: HostProbeState::CommandFailed,
                    mcp_attachment: HostProbeState::CommandFailed,
                };
                return Err(error.reason_code());
            }
        }
        let observation = match plan.target {
            ClientActivationTarget::Codex => probe_codex_host_result(environment),
            ClientActivationTarget::ClaudeCode => probe_claude_host_result(environment),
        };
        let observation = match observation {
            Ok(observation) => observation,
            Err(code) => {
                observations[index] = HostIntegrationObservation {
                    activation: HostProbeState::ProbeFailed,
                    mcp_attachment: HostProbeState::ProbeFailed,
                };
                return Err(code);
            }
        };
        observations[index] = observation;
        if !host_observation_ready(observation) {
            return Err(match plan.target {
                ClientActivationTarget::Codex => "codex-host-observation-not-ready",
                ClientActivationTarget::ClaudeCode => "claude-host-observation-not-ready",
            });
        }
    }
    Ok(())
}

const fn host_plugin_target_index(target: ClientActivationTarget) -> usize {
    match target {
        ClientActivationTarget::Codex => 0,
        ClientActivationTarget::ClaudeCode => 1,
    }
}

struct PackagedProductState {
    product: Option<VerifiedPackagedProduct>,
    blocked_reason: &'static str,
    pending: Option<PendingPackagedProductOperation>,
}

fn integration_verification_needs_reconciliation(code: &str) -> bool {
    matches!(
        code,
        "packaged-product-source-invalid"
            | "packaged-product-activation-invalid"
            | "packaged-product-replace-required"
            | "packaged-product-recovery-required"
    )
}

enum PendingPackagedProductOperation {
    Install {
        packaged: PackagedProductInstallPreview,
        host: HostPluginPlan,
    },
    BatchInstall {
        packaged: PackagedProductBatchInstallPreview,
        hosts: Vec<HostPluginPlan>,
    },
    Remove {
        selection: IntegrationSelection,
        verifications: Vec<PackagedProductInstallVerification>,
    },
}

impl PackagedProductState {
    fn read_only(blocked_reason: &'static str) -> Self {
        Self {
            product: None,
            blocked_reason,
            pending: None,
        }
    }

    fn verified(product: VerifiedPackagedProduct) -> Self {
        Self {
            product: Some(product),
            blocked_reason: "none",
            pending: None,
        }
    }

    fn preview(
        &mut self,
        environment: &CommandEnvironment,
        observation: HostIntegrationObservation,
        token: OperationToken,
        target: IntegrationTarget,
        workflow_variant_sha256: Option<&str>,
    ) -> Result<OperationPreview, &'static str> {
        let Some(product) = self.product.as_ref() else {
            return Ok(blocked_product_preview(token, target, self.blocked_reason));
        };
        let preview = preview_packaged_product_install_with_variant(
            product,
            activation_target(target),
            workflow_variant_sha256,
        )
        .map_err(|error| error.reason_code())?;
        let can_confirm = preview.can_apply;
        let host = can_confirm
            .then(|| {
                prepare_host_plugin_plan(
                    environment,
                    activation_target(target),
                    preview.effect,
                    observation,
                )
            })
            .transpose()?;
        let blocked_reason = match preview.effect {
            PackagedProductInstallEffect::Install
            | PackagedProductInstallEffect::Repair
            | PackagedProductInstallEffect::AlreadyCurrent => None,
            PackagedProductInstallEffect::ReplaceRequired => {
                Some("packaged-product-replace-required")
            }
            PackagedProductInstallEffect::RecoveryRequired => {
                Some("packaged-product-recovery-required")
            }
        };
        let operation = OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: match target {
                IntegrationTarget::Codex => "Codex packaged installation preview",
                IntegrationTarget::ClaudeCode => "Claude Code packaged installation preview",
            },
            summary: match preview.effect {
                PackagedProductInstallEffect::Install => {
                    "Install the receipt-owned qiongli-next Lite source, then let this App run the approved official Host CLI plan and verify fresh Ready evidence."
                }
                PackagedProductInstallEffect::Repair => {
                    "Repair the receipt-owned qiongli-next source, then let this App run the approved official Host CLI repair plan and verify fresh Ready evidence."
                }
                PackagedProductInstallEffect::AlreadyCurrent => {
                    "Keep the current receipt-owned qiongli-next source, run the approved official Host CLI plan if required, and verify fresh Ready evidence."
                }
                PackagedProductInstallEffect::ReplaceRequired => {
                    "An unmanaged or drifted qiongli-next installation was preserved and requires an explicit replacement workflow."
                }
                PackagedProductInstallEffect::RecoveryRequired => {
                    "A prior qiongli-next transaction requires recovery before installation can continue."
                }
            },
            display_target: Some(host.as_ref().map_or_else(
                || integration_display_target(target),
                |host| integration_host_plan_display_targets(std::slice::from_ref(host)),
            )),
            plan_digest_sha256: host.as_ref().map(|host| {
                packaged_host_plan_digest(&preview.plan_digest_sha256, std::slice::from_ref(host))
            }),
            approvals_required: if can_confirm {
                OperationApproval::ACTIVATION.to_vec()
            } else {
                Vec::new()
            },
            can_confirm,
            blocked_reason,
        };
        self.pending = host.map(|host| PendingPackagedProductOperation::Install {
            packaged: preview,
            host,
        });
        Ok(operation)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the packaged batch preview binds UI copy, selection, Host evidence, and variant identity"
    )]
    fn preview_batch(
        &mut self,
        environment: &CommandEnvironment,
        observations: &[HostIntegrationObservation; 2],
        token: OperationToken,
        selection: IntegrationSelection,
        title: &'static str,
        summary: &'static str,
        workflow_variant_sha256: Option<&str>,
    ) -> Result<OperationPreview, &'static str> {
        let Some(product) = self.product.as_ref() else {
            return Ok(blocked_batch_product_preview(
                token,
                selection,
                title,
                self.blocked_reason,
            ));
        };
        let targets = selected_activation_targets(selection)?;
        let preview = preview_packaged_product_batch_install_with_variant(
            product,
            &targets,
            workflow_variant_sha256,
        )
        .map_err(|error| error.reason_code())?;
        let hosts = preview
            .can_apply
            .then(|| {
                preview
                    .installs
                    .iter()
                    .map(|install| {
                        prepare_host_plugin_plan(
                            environment,
                            install.target,
                            install.effect,
                            host_observation_for_target(observations, install.target),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        let blocked_reason = (!preview.can_apply).then_some(
            if preview
                .installs
                .iter()
                .any(|install| install.effect == PackagedProductInstallEffect::RecoveryRequired)
            {
                "packaged-product-recovery-required"
            } else {
                "packaged-product-replace-required"
            },
        );
        let operation = OperationPreview {
            token,
            kind: OperationKind::Activation,
            title,
            summary,
            display_target: Some(hosts.as_ref().map_or_else(
                || integration_display_targets(&targets),
                |hosts| integration_host_plan_display_targets(hosts),
            )),
            plan_digest_sha256: hosts
                .as_ref()
                .map(|hosts| packaged_host_plan_digest(&preview.plan_digest_sha256, hosts)),
            approvals_required: if preview.can_apply {
                OperationApproval::ACTIVATION.to_vec()
            } else {
                Vec::new()
            },
            can_confirm: preview.can_apply,
            blocked_reason,
        };
        self.pending = hosts.map(|hosts| PendingPackagedProductOperation::BatchInstall {
            packaged: preview,
            hosts,
        });
        Ok(operation)
    }

    fn verify(
        &self,
        selection: IntegrationSelection,
        workflow_variant_sha256: Option<&str>,
    ) -> Result<&'static str, &'static str> {
        let product = self
            .product
            .as_ref()
            .ok_or("packaged-product-authority-unavailable")?;
        for target in selected_activation_targets(selection)? {
            verify_packaged_product_install_with_variant(product, target, workflow_variant_sha256)
                .map_err(|error| error.reason_code())?;
        }
        Ok("packaged-product-install-verified")
    }

    fn preview_remove(
        &mut self,
        token: OperationToken,
        selection: IntegrationSelection,
    ) -> Result<OperationPreview, &'static str> {
        let Some(product) = self.product.as_ref() else {
            return Ok(blocked_batch_product_preview(
                token,
                selection,
                "Remove selected integrations",
                self.blocked_reason,
            ));
        };
        let targets = selected_activation_targets(selection)?;
        let verifications = targets
            .iter()
            .copied()
            .map(|target| {
                verify_receipt_owned_packaged_product_install(product, target)
                    .map_err(|error| error.reason_code())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let digest = packaged_product_removal_digest(product, &verifications);
        self.pending = Some(PendingPackagedProductOperation::Remove {
            selection,
            verifications,
        });
        Ok(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: "Remove selected integrations",
            summary: "Remove only receipt-owned qiongli-next registrations and plugin sources for the selected clients.",
            display_target: Some(integration_display_targets(&targets)),
            plan_digest_sha256: Some(digest),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm(
        &mut self,
        content: &EmbeddedContent,
        environment: &CommandEnvironment,
        observation: HostIntegrationObservation,
        target: IntegrationTarget,
        now_unix: u64,
        overrides: Option<&WorkflowOverrides>,
    ) -> Result<(&'static str, Vec<HostPluginPlan>), &'static str> {
        let product = self
            .product
            .as_ref()
            .ok_or("packaged-product-authority-unavailable")?;
        let pending = self
            .pending
            .take()
            .ok_or("packaged-product-preview-missing")?;
        let PendingPackagedProductOperation::Install {
            packaged: preview,
            host,
        } = pending
        else {
            return Err("packaged-product-preview-invalid");
        };
        if preview.target != activation_target(target) {
            return Err("packaged-product-preview-invalid");
        }
        let current_host =
            prepare_host_plugin_plan(environment, preview.target, preview.effect, observation)?;
        if current_host != host {
            return Err("host-plugin-plan-changed");
        }
        let commit = apply_packaged_product_install_with_overrides(
            content.pack(),
            product,
            &preview,
            now_unix,
            overrides,
        )
        .map_err(|error| error.reason_code())?;
        Ok((
            match commit.disposition {
                qiongli_platform::PackagedProductInstallDisposition::Installed => {
                    "packaged-product-install-applied"
                }
                qiongli_platform::PackagedProductInstallDisposition::AlreadyCurrent => {
                    "packaged-product-install-already-current"
                }
            },
            vec![host],
        ))
    }

    fn confirm_batch(
        &mut self,
        content: &EmbeddedContent,
        environment: &CommandEnvironment,
        observations: &[HostIntegrationObservation; 2],
        selection: IntegrationSelection,
        now_unix: u64,
        overrides: Option<&WorkflowOverrides>,
    ) -> Result<(&'static str, Vec<HostPluginPlan>), &'static str> {
        let product = self
            .product
            .as_ref()
            .ok_or("packaged-product-authority-unavailable")?;
        let pending = self
            .pending
            .take()
            .ok_or("packaged-product-preview-missing")?;
        let PendingPackagedProductOperation::BatchInstall {
            packaged: preview,
            hosts,
        } = pending
        else {
            return Err("packaged-product-preview-invalid");
        };
        if preview
            .installs
            .iter()
            .map(|install| install.target)
            .collect::<Vec<_>>()
            != selected_activation_targets(selection)?
        {
            return Err("packaged-product-preview-invalid");
        }
        let current_hosts = preview
            .installs
            .iter()
            .map(|install| {
                prepare_host_plugin_plan(
                    environment,
                    install.target,
                    install.effect,
                    host_observation_for_target(observations, install.target),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        if current_hosts != hosts {
            return Err("host-plugin-plan-changed");
        }
        let commit = apply_packaged_product_batch_install_with_overrides(
            content.pack(),
            product,
            &preview,
            now_unix,
            overrides,
        )
        .map_err(|error| error.reason_code())?;
        Ok((
            if commit.installs.iter().all(|install| {
                install.disposition
                    == qiongli_platform::PackagedProductInstallDisposition::AlreadyCurrent
            }) {
                "packaged-product-batch-already-current"
            } else {
                "packaged-product-batch-applied"
            },
            hosts,
        ))
    }

    fn confirm_remove(
        &mut self,
        selection: IntegrationSelection,
        now_unix: u64,
    ) -> Result<&'static str, &'static str> {
        let product = self
            .product
            .as_ref()
            .ok_or("packaged-product-authority-unavailable")?;
        let pending = self
            .pending
            .take()
            .ok_or("packaged-product-preview-missing")?;
        let PendingPackagedProductOperation::Remove {
            selection: expected_selection,
            verifications,
        } = pending
        else {
            return Err("packaged-product-preview-invalid");
        };
        if expected_selection != selection {
            return Err("packaged-product-preview-invalid");
        }
        let targets = selected_activation_targets(selection)?;
        let current = targets
            .iter()
            .copied()
            .map(|target| {
                verify_receipt_owned_packaged_product_install(product, target)
                    .map_err(|error| error.reason_code())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if current != verifications {
            return Err("packaged-product-preview-invalid");
        }
        for target in targets {
            remove_packaged_product_install(product, target, now_unix)
                .map_err(|error| error.reason_code())?;
        }
        Ok("packaged-product-install-removed")
    }

    fn cancel(&mut self) {
        self.pending = None;
    }
}

enum PendingDesktopOperation {
    Blocked(OperationToken),
    GlobalSettings {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
    },
    ProviderSettings {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
    },
    ProviderSecret {
        token: OperationToken,
        expected_revision: u64,
        provider: ProviderKind,
        replacement: GlobalSettings,
        secret_ref: SecretRef,
        replacement_value: Option<SecretValue>,
        previous_value: Option<SecretValue>,
    },
    AgentBackendSettings {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
    },
    AgentBackendSecret {
        token: OperationToken,
        expected_revision: u64,
        replacement: GlobalSettings,
        secret_ref: SecretRef,
        replacement_value: Option<SecretValue>,
        previous_value: Option<SecretValue>,
    },
    AgentRun {
        token: OperationToken,
        request: FullAgentRunRequest,
    },
    SkillsMaterialization {
        token: OperationToken,
        profile: ProfileKind,
        target: MaterializationTarget,
        expected_variant_sha256: Option<String>,
        project_binding: Option<RegisteredProjectSkillsBinding>,
    },
    SkillsRemoval {
        token: OperationToken,
        target: MaterializationTarget,
        expected_receipt: MaterializationReceiptV1,
    },
    SkillsDetach {
        token: OperationToken,
        target_id: String,
        expected_profile: ProfileKind,
        expected_receipt_sha256: String,
    },
    WorkflowVariantChange {
        token: OperationToken,
        expected_revision: u64,
        expected_variant_sha256: Option<String>,
        expected_current_sha256: String,
        path: String,
        replacement: Option<Vec<u8>>,
    },
    CliInstall {
        token: OperationToken,
        plan: CliInstallPlan,
    },
    CliRemove {
        token: OperationToken,
        plan: CliRemovalPlan,
    },
    CliPathConfigure {
        token: OperationToken,
        plan: CliPathConfigurePlan,
    },
    ZoteroCompanionStage {
        token: OperationToken,
        plan: Box<ZoteroCompanionStagePlan>,
    },
    Activation {
        token: OperationToken,
        target: IntegrationTarget,
    },
    Candidate {
        token: OperationToken,
        target: IntegrationTarget,
    },
    PackagedProduct {
        token: OperationToken,
        target: IntegrationTarget,
    },
    PackagedProductBatch {
        token: OperationToken,
        selection: IntegrationSelection,
    },
    IntegrationReconciliation {
        token: OperationToken,
        selection: IntegrationSelection,
        transaction_id: String,
        journal_sha256: String,
        expected_variant_sha256: Option<String>,
        hosts: Vec<HostPluginPlan>,
    },
    PackagedProductRemoval {
        token: OperationToken,
        selection: IntegrationSelection,
    },
    UpdateInstall {
        token: OperationToken,
        expected_revision: u64,
    },
    LegacyMigration {
        token: OperationToken,
        command: crate::legacy_migration_cli::LegacyMigrationCliCommand,
        completion_code: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegisteredProjectSkillsBinding {
    project_id: ProjectId,
    expected_library_revision: u64,
    expected_project_revision: u64,
    target_id: String,
}

impl Drop for NativeDesktopService {
    fn drop(&mut self) {
        if let Some(active) = &self.mcp_self_test {
            active.cancelled.store(true, Ordering::Release);
        }
    }
}

impl PendingDesktopOperation {
    const fn token(&self) -> OperationToken {
        match self {
            Self::Blocked(token)
            | Self::GlobalSettings { token, .. }
            | Self::ProviderSettings { token, .. }
            | Self::ProviderSecret { token, .. }
            | Self::AgentBackendSettings { token, .. }
            | Self::AgentBackendSecret { token, .. }
            | Self::AgentRun { token, .. }
            | Self::SkillsMaterialization { token, .. }
            | Self::SkillsRemoval { token, .. }
            | Self::SkillsDetach { token, .. }
            | Self::WorkflowVariantChange { token, .. }
            | Self::CliInstall { token, .. }
            | Self::CliRemove { token, .. }
            | Self::CliPathConfigure { token, .. }
            | Self::ZoteroCompanionStage { token, .. }
            | Self::Activation { token, .. }
            | Self::Candidate { token, .. }
            | Self::PackagedProduct { token, .. }
            | Self::PackagedProductBatch { token, .. }
            | Self::IntegrationReconciliation { token, .. }
            | Self::PackagedProductRemoval { token, .. }
            | Self::UpdateInstall { token, .. }
            | Self::LegacyMigration { token, .. } => *token,
        }
    }
}

impl NativeDesktopService {
    fn new(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        activation_sessions: Vec<DesktopActivationSession>,
    ) -> Self {
        Self::new_with_packaged_product(
            environment,
            content,
            activation_sessions,
            PackagedProductState::read_only("source-build-read-only"),
        )
    }

    fn new_with_packaged_product(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        activation_sessions: Vec<DesktopActivationSession>,
        packaged_product: PackagedProductState,
    ) -> Self {
        let update_view = update_snapshot(&environment);
        let zotero = zotero_service_snapshot(&environment);
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker: Box::new(NativeFolderPicker),
            selected_skills_target: None,
            secret_store: crate::credential_store::native_secret_store(),
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            update_view,
            active_update: None,
            activation_sessions,
            candidate_sessions: Vec::new(),
            packaged_product,
            host_observations: [HostIntegrationObservation::default(); 2],
            zotero,
            cli_path_test: None,
            direct_backend_experiment_enabled: false,
        }
    }

    fn new_with_candidate_sessions(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        candidate_sessions: Vec<DesktopCandidateSession>,
    ) -> Self {
        let update_view = update_snapshot(&environment);
        let zotero = zotero_service_snapshot(&environment);
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker: Box::new(NativeFolderPicker),
            selected_skills_target: None,
            secret_store: crate::credential_store::native_secret_store(),
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            update_view,
            active_update: None,
            activation_sessions: Vec::new(),
            candidate_sessions,
            packaged_product: PackagedProductState::read_only("candidate-session-only"),
            host_observations: [HostIntegrationObservation::default(); 2],
            zotero,
            cli_path_test: None,
            direct_backend_experiment_enabled: false,
        }
    }

    #[cfg(test)]
    fn new_with_folder_picker(
        environment: CommandEnvironment,
        content: EmbeddedContent,
        folder_picker: Box<dyn FolderPicker>,
    ) -> Self {
        let update_view = update_snapshot(&environment);
        let zotero = zotero_service_snapshot(&environment);
        Self {
            environment,
            content,
            active_operation: None,
            folder_picker,
            selected_skills_target: None,
            secret_store: crate::credential_store::native_secret_store(),
            mcp_self_test: None,
            mcp_self_test_executor: Arc::new(NativeMcpSelfTestExecutor),
            mcp_self_test_timeout: MCP_SELF_TEST_TIMEOUT,
            update_view,
            active_update: None,
            activation_sessions: Vec::new(),
            candidate_sessions: Vec::new(),
            packaged_product: PackagedProductState::read_only("source-build-read-only"),
            host_observations: [HostIntegrationObservation::default(); 2],
            zotero,
            cli_path_test: None,
            direct_backend_experiment_enabled: false,
        }
    }

    fn start_mcp_self_test(&mut self) -> DesktopEvent {
        if let Some(active) = &self.mcp_self_test {
            return DesktopEvent::McpSelfTestUpdated(active.running.clone());
        }
        let snapshot = build_snapshot(&self.environment, &self.content, self.secret_store.as_ref());
        let counts = mcp_self_test_counts(&snapshot);
        let server = match full_server(&self.environment, &self.content) {
            Ok(server) => server,
            Err(_) => {
                return DesktopEvent::McpSelfTestUpdated(contract_failure_mcp_self_test(counts));
            }
        };
        // The bounded offline test exercises only the Full protocol contract, exact registry,
        // and the read-only Full route. Provider readiness is projected from the snapshot above,
        // so the route must not resolve credentials or synchronously prompt the OS credential
        // store on the UI thread.
        let running = pending_mcp_self_test(counts);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let executor = Arc::clone(&self.mcp_self_test_executor);
        let (sender, receiver) = mpsc::sync_channel(1);
        let spawn = thread::Builder::new()
            .name("qiongli-mcp-self-test".to_owned())
            .spawn(move || {
                let result = executor.run(McpSelfTestInput { server, counts }, worker_cancelled);
                let _ = sender.send(result);
            });
        if spawn.is_err() {
            return DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                McpSelfTestState::Failed,
                counts,
            ));
        }
        let now = Instant::now();
        self.mcp_self_test = Some(ActiveMcpSelfTest {
            receiver,
            cancelled,
            deadline: now.checked_add(self.mcp_self_test_timeout).unwrap_or(now),
            running: running.clone(),
        });
        DesktopEvent::McpSelfTestUpdated(running)
    }

    fn poll_mcp_self_test(&mut self) -> DesktopEvent {
        let Some(active) = self.mcp_self_test.as_ref() else {
            return DesktopEvent::Failed {
                code: "mcp-self-test-not-running",
            };
        };
        if Instant::now() >= active.deadline {
            let active = self
                .mcp_self_test
                .take()
                .expect("validated MCP self-test remains active");
            active.cancelled.store(true, Ordering::Release);
            return DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                McpSelfTestState::TimedOut,
                mcp_counts_from_view(&active.running),
            ));
        }
        match active.receiver.try_recv() {
            Ok(result) => {
                self.mcp_self_test = None;
                DesktopEvent::McpSelfTestUpdated(result)
            }
            Err(mpsc::TryRecvError::Empty) => {
                DesktopEvent::McpSelfTestUpdated(active.running.clone())
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                let counts = mcp_counts_from_view(&active.running);
                self.mcp_self_test = None;
                DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
                    McpSelfTestState::Failed,
                    counts,
                ))
            }
        }
    }

    fn cancel_mcp_self_test(&mut self) -> DesktopEvent {
        let Some(active) = self.mcp_self_test.take() else {
            return DesktopEvent::Failed {
                code: "mcp-self-test-not-running",
            };
        };
        active.cancelled.store(true, Ordering::Release);
        DesktopEvent::McpSelfTestUpdated(terminal_mcp_self_test(
            McpSelfTestState::Cancelled,
            mcp_counts_from_view(&active.running),
        ))
    }

    fn select_update_stream(&mut self, stream: UpdateStreamView) -> DesktopEvent {
        if self.active_update.is_some() || !self.update_view.can_select_stream {
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        }
        let result = with_native_update_context(&self.environment, |store, authority, content| {
            crate::update_cli::desktop_select_stream(
                store,
                update_stream_preference(stream),
                authority,
                crate::embedded_macos_team_id(),
                &self.environment,
                content,
            )
        });
        self.update_view = match result {
            Ok(()) => update_snapshot(&self.environment),
            Err(code) => update_failure_view(&self.environment, code),
        };
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn start_update_check(&mut self) -> DesktopEvent {
        if self.active_update.is_some() || !self.update_view.can_check {
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        }
        let running = update_busy_view(
            &self.update_view,
            UpdatePhaseView::Checking,
            1,
            "Checking signed update metadata",
        );
        let environment = self.environment.clone();
        let (sender, receiver) = mpsc::channel();
        let spawn = thread::Builder::new()
            .name("qiongli-update-check".to_owned())
            .spawn(move || {
                let result =
                    with_native_update_context(&environment, |store, authority, content| {
                        crate::update_cli::desktop_check(
                            store,
                            authority,
                            crate::embedded_macos_team_id(),
                            &environment,
                            content,
                        )
                        .map(DesktopUpdateOutcome::Checked)
                    });
                let _ = sender.send(DesktopUpdateWorkerMessage::Finished(result));
            });
        if spawn.is_err() {
            self.update_view =
                update_failure_view(&self.environment, "native-update-worker-unavailable");
        } else {
            self.update_view = running;
            self.active_update = Some(ActiveDesktopUpdate { receiver });
        }
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn start_update_preparation(&mut self) -> DesktopEvent {
        if self.active_update.is_some() || !self.update_view.can_prepare {
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        }
        let running = update_busy_view(
            &self.update_view,
            UpdatePhaseView::Downloading,
            1,
            "Downloading signed package",
        );
        let environment = self.environment.clone();
        let target_version = self.update_view.available_version.clone();
        let archive_size_bytes = self.update_view.archive_size_bytes;
        let selected_stream = self.update_view.selected_stream;
        let (sender, receiver) = mpsc::channel();
        let worker_sender = sender.clone();
        let spawn = thread::Builder::new()
            .name("qiongli-update-prepare".to_owned())
            .spawn(move || {
                let result =
                    with_native_update_context(&environment, |store, authority, content| {
                        crate::update_cli::desktop_prepare(
                            store,
                            authority,
                            crate::embedded_macos_team_id(),
                            &environment,
                            content,
                            |stage| {
                                let update = update_preparation_progress_view(
                                    selected_stream,
                                    target_version.clone(),
                                    archive_size_bytes,
                                    stage,
                                );
                                let _ = worker_sender
                                    .send(DesktopUpdateWorkerMessage::Progress(update));
                            },
                        )
                        .map(DesktopUpdateOutcome::Prepared)
                    });
                let _ = sender.send(DesktopUpdateWorkerMessage::Finished(result));
            });
        if spawn.is_err() {
            self.update_view =
                update_failure_view(&self.environment, "native-update-worker-unavailable");
        } else {
            self.update_view = running;
            self.active_update = Some(ActiveDesktopUpdate { receiver });
        }
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn poll_update(&mut self) -> DesktopEvent {
        let Some(result) = self
            .active_update
            .as_ref()
            .map(|active| active.receiver.try_recv())
        else {
            self.update_view = update_snapshot(&self.environment);
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        };
        match result {
            Ok(DesktopUpdateWorkerMessage::Progress(update)) => {
                self.update_view = update;
                DesktopEvent::UpdateChanged {
                    update: self.update_view.clone(),
                    close_requested: false,
                }
            }
            Ok(DesktopUpdateWorkerMessage::Finished(result)) => {
                self.active_update = None;
                let (update, close_requested) = match result {
                    Ok(DesktopUpdateOutcome::Checked(checked)) => {
                        (update_checked_view(checked), false)
                    }
                    Ok(DesktopUpdateOutcome::Prepared(prepared)) => {
                        (update_prepared_view(&self.environment, prepared), false)
                    }
                    Ok(DesktopUpdateOutcome::Cancelled(stream)) => {
                        (update_cancelled_view(stream), false)
                    }
                    Ok(DesktopUpdateOutcome::InstallHandoff(handoff)) => {
                        (update_install_handoff_view(handoff), true)
                    }
                    Err(code) => (update_failure_view(&self.environment, code), false),
                };
                self.update_view = update;
                DesktopEvent::UpdateChanged {
                    update: self.update_view.clone(),
                    close_requested,
                }
            }
            Err(mpsc::TryRecvError::Empty) => DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            },
            Err(mpsc::TryRecvError::Disconnected) => {
                self.active_update = None;
                self.update_view =
                    update_failure_view(&self.environment, "native-update-worker-unavailable");
                DesktopEvent::UpdateChanged {
                    update: self.update_view.clone(),
                    close_requested: false,
                }
            }
        }
    }

    fn cancel_update(&mut self) -> DesktopEvent {
        if !self.update_view.can_cancel {
            return DesktopEvent::UpdateChanged {
                update: self.update_view.clone(),
                close_requested: false,
            };
        }
        let selected_stream = self.update_view.selected_stream;
        self.update_view = update_busy_view(
            &self.update_view,
            UpdatePhaseView::Cancelling,
            1,
            "Removing staged update bytes",
        );
        let environment = self.environment.clone();
        let (sender, receiver) = mpsc::channel();
        let spawn = thread::Builder::new()
            .name("qiongli-update-cancel".to_owned())
            .spawn(move || {
                let result =
                    with_native_update_context(&environment, |store, authority, content| {
                        crate::update_cli::desktop_cancel(
                            store,
                            authority,
                            crate::embedded_macos_team_id(),
                            &environment,
                            content,
                        )
                        .map(|()| DesktopUpdateOutcome::Cancelled(selected_stream))
                    });
                let _ = sender.send(DesktopUpdateWorkerMessage::Finished(result));
            });
        self.active_update = None;
        if spawn.is_err() {
            self.update_view =
                update_failure_view(&self.environment, "native-update-worker-unavailable");
        } else {
            self.active_update = Some(ActiveDesktopUpdate { receiver });
        }
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn preview_update_install(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        if self.active_update.is_some() {
            return DesktopEvent::Failed {
                code: "native-update-transaction-active",
            };
        }
        let store = match update_store(&self.environment) {
            Ok(store) => store,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let Some(transaction) = loaded.state.active_transaction else {
            return DesktopEvent::Failed {
                code: "native-update-transaction-missing",
            };
        };
        if !matches!(
            transaction.phase,
            UpdateTransactionPhase::Staged | UpdateTransactionPhase::ReconciliationPrepared
        ) {
            return DesktopEvent::Failed {
                code: "native-update-transaction-not-installable",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = update_install_digest(
            loaded.revision,
            &transaction.transaction_id,
            &transaction.target_version,
        );
        self.active_operation = Some(PendingDesktopOperation::UpdateInstall {
            token,
            expected_revision: loaded.revision,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::UpdateInstall,
            title: "Install Qiongli update",
            summary: "Quit Qiongli, atomically activate the verified application and managed content, then roll back automatically if the new runtime fails health checks.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn start_update_install(&mut self, expected_revision: u64) -> DesktopEvent {
        let loaded = match update_store(&self.environment)
            .and_then(|store| store.load().map_err(|error| error.reason_code()))
        {
            Ok(value) => value,
            Err(code) => {
                return DesktopEvent::UpdateChanged {
                    update: update_failure_view(&self.environment, code),
                    close_requested: false,
                };
            }
        };
        if loaded.revision != expected_revision {
            return DesktopEvent::UpdateChanged {
                update: update_failure_view(&self.environment, "revision-conflict"),
                close_requested: false,
            };
        }
        let running = update_busy_view(
            &self.update_view,
            UpdatePhaseView::Installing,
            4,
            "Handing off to the native update helper",
        );
        let environment = self.environment.clone();
        let (sender, receiver) = mpsc::channel();
        let spawn = thread::Builder::new()
            .name("qiongli-update-install".to_owned())
            .spawn(move || {
                let result =
                    with_native_update_context(&environment, |store, authority, content| {
                        crate::update_cli::desktop_install(
                            store,
                            authority,
                            crate::embedded_macos_team_id(),
                            &environment,
                            content,
                        )
                        .map(DesktopUpdateOutcome::InstallHandoff)
                    });
                let _ = sender.send(DesktopUpdateWorkerMessage::Finished(result));
            });
        if spawn.is_err() {
            self.update_view =
                update_failure_view(&self.environment, "native-update-worker-unavailable");
        } else {
            self.update_view = running;
            self.active_update = Some(ActiveDesktopUpdate { receiver });
        }
        DesktopEvent::UpdateChanged {
            update: self.update_view.clone(),
            close_requested: false,
        }
    }

    fn next_operation_token() -> Result<OperationToken, &'static str> {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| "operation-token-unavailable")?;
        Ok(OperationToken::new(u128::from_le_bytes(bytes)))
    }

    fn issue_preview(
        &mut self,
        title: &'static str,
        summary: &'static str,
        blocked_reason: &'static str,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        self.active_operation = Some(PendingDesktopOperation::Blocked(token));
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::GlobalSettings,
            title,
            summary,
            display_target: None,
            plan_digest_sha256: None,
            approvals_required: Vec::new(),
            can_confirm: false,
            blocked_reason: Some(blocked_reason),
        })
    }

    fn workflow_variant_store(&self) -> Result<WorkflowVariantStore, &'static str> {
        config_root(&self.environment)
            .map(WorkflowVariantStore::new)
            .map_err(|error| error.reason_code())
    }

    fn load_workflow_variant(&self) -> Result<qiongli_config::LoadedWorkflowVariant, &'static str> {
        self.workflow_variant_store()?
            .load(self.content.pack())
            .map_err(|error| error.reason_code())
    }

    fn preview_workflow_variant_change(
        &mut self,
        expected_revision: u64,
        expected_variant_sha256: Option<String>,
        path: String,
        expected_current_sha256: String,
        replacement: Option<String>,
    ) -> Result<AppOperationPreview, &'static str> {
        self.cancel_active_operation();
        let store = self.workflow_variant_store()?;
        let preview = match replacement.as_deref() {
            Some(content) => store.preview_replace_resource(
                self.content.pack(),
                expected_revision,
                expected_variant_sha256.as_deref(),
                &expected_current_sha256,
                &path,
                content.as_bytes(),
            ),
            None => store.preview_reset_resource(
                self.content.pack(),
                expected_revision,
                expected_variant_sha256.as_deref(),
                &expected_current_sha256,
                &path,
            ),
        }
        .map_err(|error| error.reason_code())?;
        let token = Self::next_operation_token()?;
        let reset = replacement.is_none();
        let operation = app_workflow_variant_operation_preview(
            format!("{:032x}", token.value()),
            preview.plan_digest_sha256().to_owned(),
            &path,
            reset,
        );
        self.active_operation = Some(PendingDesktopOperation::WorkflowVariantChange {
            token,
            expected_revision,
            expected_variant_sha256,
            expected_current_sha256,
            path,
            replacement: replacement.map(String::into_bytes),
        });
        Ok(operation)
    }

    fn preview_global_settings(&mut self, patch: GlobalSettingsPatch) -> DesktopEvent {
        self.cancel_active_operation();
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        if loaded.revision != patch.expected_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }

        let mut replacement = loaded.settings.clone();
        replacement.default_profile = profile_to_content(patch.default_profile);
        if replacement == loaded.settings {
            return self.issue_preview(
                "Global settings preview",
                "The current product-wide default profile is already active. No configuration change will be made.",
                "global-settings-unchanged",
            );
        }

        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = global_settings_patch_digest(patch.expected_revision, &replacement);
        self.active_operation = Some(PendingDesktopOperation::GlobalSettings {
            token,
            expected_revision: patch.expected_revision,
            replacement,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::GlobalSettings,
            title: "Global settings preview",
            summary: "Atomically update the product-wide default profile without changing literature provider configuration.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::ClientConfigChange],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_provider_settings(&mut self, patch: ProviderSettingsPatch) -> DesktopEvent {
        self.cancel_active_operation();
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        if loaded.revision != patch.expected_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }
        let mut replacement = loaded.settings.clone();
        replacement.providers.openalex.enabled = patch.providers_enabled[0];
        replacement.providers.semantic_scholar.enabled = patch.providers_enabled[1];
        replacement.providers.crossref.enabled = patch.providers_enabled[2];
        replacement.providers.pubmed.enabled = patch.providers_enabled[3];
        replacement.providers.arxiv.enabled = patch.providers_enabled[4];
        if let Err(code) = apply_public_email_change(
            &mut replacement.providers.openalex.email,
            patch.openalex_email,
        ) {
            return DesktopEvent::ValidationFailed { code };
        }
        if let Err(code) = apply_public_email_change(
            &mut replacement.providers.crossref.email,
            patch.crossref_email,
        ) {
            return DesktopEvent::ValidationFailed { code };
        }
        if replacement == loaded.settings {
            return DesktopEvent::ValidationFailed {
                code: "provider-settings-unchanged",
            };
        }

        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = global_settings_patch_digest(patch.expected_revision, &replacement);
        self.active_operation = Some(PendingDesktopOperation::ProviderSettings {
            token,
            expected_revision: patch.expected_revision,
            replacement,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::ProviderSettings,
            title: "Literature provider settings preview",
            summary: "Atomically update provider enablement and supported public contact settings without changing global product defaults.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::ClientConfigChange],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_provider_secret(
        &mut self,
        provider: ProviderKind,
        change: ProviderSecretChange,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        if self.secret_store.status() != SecretStoreStatus::Available
            || !matches!(
                provider,
                ProviderKind::OpenAlex | ProviderKind::SemanticScholar | ProviderKind::PubMed
            )
        {
            return DesktopEvent::ValidationFailed {
                code: "provider-secret-store-unavailable",
            };
        }
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let current_ref = match provider {
            ProviderKind::OpenAlex => loaded.settings.providers.openalex.api_key_ref.as_ref(),
            ProviderKind::SemanticScholar => loaded
                .settings
                .providers
                .semantic_scholar
                .api_key_ref
                .as_ref(),
            ProviderKind::PubMed => loaded.settings.providers.pubmed.api_key_ref.as_ref(),
            ProviderKind::Crossref | ProviderKind::Arxiv => None,
        };
        let (secret_ref, replacement_value, previous_value) = match change {
            ProviderSecretChange::Replace(value) => {
                let replacement_value = match SecretValue::new(value.expose().as_bytes().to_vec()) {
                    Ok(value) => value,
                    Err(_) => {
                        return DesktopEvent::ValidationFailed {
                            code: "provider-secret-invalid",
                        };
                    }
                };
                let secret_ref = match current_ref.cloned().map_or_else(new_secret_ref, Ok) {
                    Ok(reference) => reference,
                    Err(code) => return DesktopEvent::Failed { code },
                };
                let previous_value = self.secret_store.resolve(&secret_ref).ok();
                (secret_ref, Some(replacement_value), previous_value)
            }
            ProviderSecretChange::Remove => {
                let Some(secret_ref) = current_ref.cloned() else {
                    return DesktopEvent::ValidationFailed {
                        code: "provider-secret-not-configured",
                    };
                };
                let previous_value = self.secret_store.resolve(&secret_ref).ok();
                (secret_ref, None, previous_value)
            }
        };
        let mut replacement = loaded.settings.clone();
        match provider {
            ProviderKind::OpenAlex => {
                replacement.providers.openalex.api_key_ref =
                    replacement_value.as_ref().map(|_| secret_ref.clone());
            }
            ProviderKind::SemanticScholar => {
                replacement.providers.semantic_scholar.api_key_ref =
                    replacement_value.as_ref().map(|_| secret_ref.clone());
            }
            ProviderKind::PubMed => {
                replacement.providers.pubmed.api_key_ref =
                    replacement_value.as_ref().map(|_| secret_ref.clone());
            }
            ProviderKind::Crossref | ProviderKind::Arxiv => {
                return DesktopEvent::ValidationFailed {
                    code: "provider-secret-unsupported",
                };
            }
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = provider_secret_digest(
            loaded.revision,
            provider,
            &secret_ref,
            replacement_value.is_some(),
        );
        self.active_operation = Some(PendingDesktopOperation::ProviderSecret {
            token,
            expected_revision: loaded.revision,
            provider,
            replacement,
            secret_ref,
            replacement_value,
            previous_value,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::ProviderSecret,
            title: "Provider credential preview",
            summary: "Save, replace, or remove the selected API key in the OS credential store while persisting only its opaque reference in configuration.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![
                OperationApproval::SecretStoreWrite,
                OperationApproval::ClientConfigChange,
            ],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn test_literature_provider(&mut self, provider: ProviderKind) -> DesktopEvent {
        self.cancel_active_operation();
        let loaded = match config_store(&self.environment).and_then(|store| store.load()) {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let access =
            ProviderAccess::from_global_settings(&loaded.settings, self.secret_store.as_ref());
        let provider_id = match provider {
            ProviderKind::OpenAlex => ProviderId::OpenAlex,
            ProviderKind::SemanticScholar => ProviderId::SemanticScholar,
            ProviderKind::Crossref => ProviderId::Crossref,
            ProviderKind::PubMed => ProviderId::PubMed,
            ProviderKind::Arxiv => ProviderId::Arxiv,
        };
        match access.availability(provider_id) {
            ProviderAvailability::Ready => DesktopEvent::Completed {
                code: "literature-provider-ready",
            },
            ProviderAvailability::Disabled => DesktopEvent::Failed {
                code: "literature-provider-disabled",
            },
            ProviderAvailability::NeedsSecret => DesktopEvent::Failed {
                code: "literature-provider-key-required",
            },
            ProviderAvailability::NeedsPublicSetting => DesktopEvent::Failed {
                code: "literature-provider-email-required",
            },
            ProviderAvailability::SecretStoreUnavailable => DesktopEvent::Failed {
                code: "literature-provider-secret-store-unavailable",
            },
        }
    }

    fn preview_agent_backend_settings(&mut self, patch: AgentBackendSettingsPatch) -> DesktopEvent {
        self.cancel_active_operation();
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        if loaded.revision != patch.expected_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }
        let mut replacement = loaded.settings.clone();
        replacement.agent_backends.openai.enabled = patch.openai_enabled;
        if replacement == loaded.settings {
            return DesktopEvent::ValidationFailed {
                code: "agent-backend-settings-unchanged",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = agent_backend_settings_digest(patch.expected_revision, &replacement);
        self.active_operation = Some(PendingDesktopOperation::AgentBackendSettings {
            token,
            expected_revision: patch.expected_revision,
            replacement,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::AgentBackendSettings,
            title: "Agent backend settings preview",
            summary: "Enable or disable the direct OpenAI backend without testing a connection or changing its credential.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::ClientConfigChange],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_agent_backend_secret(&mut self, change: AgentBackendSecretChange) -> DesktopEvent {
        self.cancel_active_operation();
        if self.secret_store.status() != SecretStoreStatus::Available {
            return DesktopEvent::ValidationFailed {
                code: "agent-backend-secret-store-unavailable",
            };
        }
        let store = match config_store(&self.environment) {
            Ok(store) => store,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let loaded = match store.load() {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let current_ref = loaded.settings.agent_backends.openai.api_key_ref.as_ref();
        let (secret_ref, replacement_value, previous_value) = match change {
            AgentBackendSecretChange::Replace(value) => {
                let replacement_value = match SecretValue::new(value.expose().as_bytes().to_vec()) {
                    Ok(value) => value,
                    Err(_) => {
                        return DesktopEvent::ValidationFailed {
                            code: "agent-backend-secret-invalid",
                        };
                    }
                };
                let secret_ref = match current_ref
                    .cloned()
                    .map_or_else(new_agent_backend_secret_ref, Ok)
                {
                    Ok(reference) => reference,
                    Err(code) => return DesktopEvent::Failed { code },
                };
                let previous_value = self.secret_store.resolve(&secret_ref).ok();
                (secret_ref, Some(replacement_value), previous_value)
            }
            AgentBackendSecretChange::Remove => {
                let Some(secret_ref) = current_ref.cloned() else {
                    return DesktopEvent::ValidationFailed {
                        code: "agent-backend-secret-not-configured",
                    };
                };
                let previous_value = self.secret_store.resolve(&secret_ref).ok();
                (secret_ref, None, previous_value)
            }
        };
        let mut replacement = loaded.settings.clone();
        replacement.agent_backends.openai.api_key_ref =
            replacement_value.as_ref().map(|_| secret_ref.clone());
        if replacement_value.is_none() {
            replacement.agent_backends.openai.enabled = false;
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest =
            agent_backend_secret_digest(loaded.revision, &secret_ref, replacement_value.is_some());
        let replacing_credential = replacement_value.is_some();
        self.active_operation = Some(PendingDesktopOperation::AgentBackendSecret {
            token,
            expected_revision: loaded.revision,
            replacement,
            secret_ref,
            replacement_value,
            previous_value,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::AgentBackendSecret,
            title: if replacing_credential {
                "OpenAI credential preview"
            } else {
                "Remove legacy OpenAI credential"
            },
            summary: if replacing_credential {
                "Save or replace the OpenAI API key in the OS credential store while persisting only its opaque reference in configuration."
            } else {
                "Remove the legacy direct-backend credential and disable that backend. Model execution now belongs to the connected host."
            },
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![
                OperationApproval::SecretStoreWrite,
                OperationApproval::ClientConfigChange,
            ],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn test_openai_backend(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let loaded = match config_store(&self.environment).and_then(|store| store.load()) {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let control = BackendControlService::from_global_settings(
            &loaded.settings,
            Arc::clone(&self.secret_store),
        );
        match control.test_openai_connection(&AgentCancellationToken::new()) {
            Ok(_) => DesktopEvent::Completed {
                code: "openai-backend-connection-passed",
            },
            Err(error) => DesktopEvent::Failed {
                code: error.reason_code(),
            },
        }
    }

    fn preview_agent_run(&mut self, draft: AgentRunDraft) -> DesktopEvent {
        self.cancel_active_operation();
        let project_id = match ProjectId::parse(draft.project_id) {
            Ok(project_id) => project_id,
            Err(_) => {
                return DesktopEvent::ValidationFailed {
                    code: "agent-run-request-invalid",
                };
            }
        };
        let Some(projects) = project_state_service(&self.environment) else {
            return DesktopEvent::Failed {
                code: "project-service-unavailable",
            };
        };
        let snapshot = match projects.snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let Some(project) = snapshot
            .projects
            .iter()
            .find(|project| project.project_id == project_id)
        else {
            return DesktopEvent::Failed {
                code: "project-not-registered",
            };
        };
        if project.semantic_revision != draft.expected_project_revision {
            return DesktopEvent::Failed {
                code: "revision-conflict",
            };
        }
        let loaded = match config_store(&self.environment).and_then(|store| store.load()) {
            Ok(loaded) => loaded,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let readiness =
            openai_backend_status(&loaded.settings, self.secret_store.as_ref()).readiness;
        if readiness != BackendReadinessV1::Ready {
            return DesktopEvent::Failed {
                code: readiness_reason_code(readiness),
            };
        }
        if FullProjectToolRegistry::from_embedded_content(&self.content).is_err() {
            return DesktopEvent::Failed {
                code: "agent-run-tools-unavailable",
            };
        }
        let digest = agent_run_digest(
            &project_id,
            draft.expected_project_revision,
            draft.prompt.expose(),
        );
        let request = match FullAgentRunRequest::new(
            project_id,
            draft.expected_project_revision,
            draft.prompt,
            true,
        ) {
            Ok(request) => request,
            Err(error) => {
                return DesktopEvent::ValidationFailed {
                    code: error.reason_code(),
                };
            }
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        self.active_operation = Some(PendingDesktopOperation::AgentRun { token, request });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::AgentRun,
            title: "Run project query with OpenAI",
            summary: "Send this prompt and any redacted read-only project tool results to OpenAI. The run is bound to the selected project revision and cannot write project files.",
            display_target: None,
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::NetworkRequest],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn select_skills_destination(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(path) = self.folder_picker.pick_folder() else {
            return DesktopEvent::Cancelled {
                code: "skills-destination-selection-cancelled",
            };
        };
        let target = match approve_materialization_target(&path) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let display_path = PrivateDisplayText::new(display_path(&path));
        let target_id = managed_skills_target_id(&path.to_string_lossy());
        self.selected_skills_target = Some(target);
        DesktopEvent::SkillsDestinationSelected {
            display_path,
            target_id,
        }
    }

    fn preview_skills_materialization(&mut self, profile: ProfileKind) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let variant = match self.load_workflow_variant() {
            Ok(variant) => variant,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let display_target =
            PrivateDisplayText::new(self.managed_skills_symbolic_target(target.path()));
        let digest = skills_materialization_digest(
            self.content.pack().pack_sha256(),
            variant.variant_sha256(),
            profile,
            target.path(),
        );
        self.active_operation = Some(PendingDesktopOperation::SkillsMaterialization {
            token,
            profile,
            target,
            expected_variant_sha256: variant.variant_sha256().map(str::to_owned),
            project_binding: None,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::SkillsMaterialization,
            title: "Skills materialization preview",
            summary: "Write the selected embedded profile to the explicitly selected folder and verify its managed receipt.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_registered_project_skills_materialization(
        &mut self,
        profile: ProfileKind,
        project_root: &Path,
        project_id: ProjectId,
        expected_library_revision: u64,
        expected_project_revision: u64,
    ) -> DesktopEvent {
        let target = match approve_materialization_target(project_root.join(".qiongli-skills")) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::ValidationFailed {
                    code: error.reason_code(),
                };
            }
        };
        let target_id = managed_skills_target_id(&target.path().to_string_lossy());
        self.selected_skills_target = Some(target);
        match self.preview_skills_materialization(profile) {
            DesktopEvent::PreviewReady(mut preview) => {
                let Some(PendingDesktopOperation::SkillsMaterialization {
                    project_binding, ..
                }) = self.active_operation.as_mut()
                else {
                    return DesktopEvent::Failed {
                        code: "project-skills-preview-invalid",
                    };
                };
                *project_binding = Some(RegisteredProjectSkillsBinding {
                    project_id,
                    expected_library_revision,
                    expected_project_revision,
                    target_id,
                });
                preview.display_target = Some(PrivateDisplayText::new(
                    "<project>/.qiongli-skills".to_owned(),
                ));
                DesktopEvent::PreviewReady(preview)
            }
            event => event,
        }
    }

    fn validate_registered_project_skills_confirmation(
        &self,
        token: OperationToken,
        projects: &Option<ProjectStateService>,
    ) -> Result<(), &'static str> {
        let Some(PendingDesktopOperation::SkillsMaterialization {
            token: pending_token,
            project_binding: Some(binding),
            ..
        }) = self.active_operation.as_ref()
        else {
            return Ok(());
        };
        if *pending_token != token {
            return Ok(());
        }
        let projects = projects
            .as_ref()
            .ok_or("project-skills-project-service-unavailable")?;
        let snapshot = projects.snapshot().map_err(|error| error.reason_code())?;
        let project = snapshot
            .projects
            .iter()
            .find(|project| project.project_id == binding.project_id)
            .ok_or("project-skills-project-not-registered")?;
        if project.lifecycle != ProjectLifecycle::Active {
            return Err("project-skills-project-archived");
        }
        if project.health != ProjectHealth::Ready {
            return Err("project-skills-project-not-ready");
        }
        if snapshot.revision != binding.expected_library_revision {
            return Err("project-skills-library-revision-conflict");
        }
        if project.semantic_revision != binding.expected_project_revision {
            return Err("project-skills-project-revision-conflict");
        }
        let root = projects
            .resolve_project_root(&binding.project_id)
            .map_err(|error| error.reason_code())?;
        let target = root.path().join(".qiongli-skills");
        if managed_skills_target_id(&target.to_string_lossy()) != binding.target_id {
            return Err("project-skills-target-changed");
        }
        Ok(())
    }

    fn verify_skills_materialization(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        match verify_materialization(&target) {
            Ok(_) => DesktopEvent::Completed {
                code: "skills-materialization-verified",
            },
            Err(error) => DesktopEvent::Failed {
                code: error.reason_code(),
            },
        }
    }

    fn preview_skills_removal(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let Some(selected) = self.selected_skills_target.as_ref() else {
            return DesktopEvent::ValidationFailed {
                code: "skills-destination-required",
            };
        };
        let target = match approve_materialization_target(selected.path()) {
            Ok(target) => target,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let receipt = match verify_materialization(&target) {
            Ok(receipt) => receipt,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let display_target =
            PrivateDisplayText::new(self.managed_skills_symbolic_target(target.path()));
        let digest = skills_removal_digest(&receipt, target.path());
        self.active_operation = Some(PendingDesktopOperation::SkillsRemoval {
            token,
            target,
            expected_receipt: receipt,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::SkillsRemoval,
            title: "Skills removal preview",
            summary: "Remove only the selected Qiongli-managed materialization after re-verifying its complete receipt.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_cli_install(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        if self.packaged_product.product.is_none() {
            return DesktopEvent::Failed {
                code: self.packaged_product.blocked_reason,
            };
        }
        let Some(home) = self.environment.platform_home() else {
            return DesktopEvent::Failed {
                code: "qiongli-cli-home-unavailable",
            };
        };
        let Some(source) = bundled_cli_path(Some(home)) else {
            return DesktopEvent::Failed {
                code: "qiongli-cli-bundle-unavailable",
            };
        };
        let plan = match preview_cli_install(home, &source, env!("CARGO_PKG_VERSION")) {
            Ok(plan) => plan,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let display_target = PrivateDisplayText::new("<user-home>/.local/bin/qiongli".to_owned());
        let plan_sha256 = plan.plan_sha256().to_owned();
        self.active_operation = Some(PendingDesktopOperation::CliInstall { token, plan });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::CliInstall,
            title: "Install Qiongli CLI",
            summary: "Install the exact native CLI bundled with this App into the user CLI directory. An existing unmanaged qiongli command at that target is retained as a private backup.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(plan_sha256),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_cli_remove(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        if self.packaged_product.product.is_none() {
            return DesktopEvent::Failed {
                code: self.packaged_product.blocked_reason,
            };
        }
        let Some(home) = self.environment.platform_home() else {
            return DesktopEvent::Failed {
                code: "qiongli-cli-home-unavailable",
            };
        };
        let path = std::env::var_os("PATH");
        let shell = std::env::var_os("SHELL");
        let plan = match preview_cli_remove(home, path.as_deref(), shell.as_deref()) {
            Ok(plan) => plan,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let plan_sha256 = plan.plan_sha256().to_owned();
        self.active_operation = Some(PendingDesktopOperation::CliRemove { token, plan });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::CliRemove,
            title: "Remove Qiongli CLI",
            summary: "Remove only the receipt-owned native CLI. Restore a retained unmanaged predecessor only when its receipt-bound bytes still match.",
            display_target: Some(PrivateDisplayText::new(
                "<user-home>/.local/bin/qiongli".to_owned(),
            )),
            plan_digest_sha256: Some(plan_sha256),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn preview_cli_path_configure(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        if self.packaged_product.product.is_none() {
            return DesktopEvent::Failed {
                code: self.packaged_product.blocked_reason,
            };
        }
        let Some(home) = self.environment.platform_home() else {
            return DesktopEvent::Failed {
                code: "qiongli-cli-home-unavailable",
            };
        };
        let shell = std::env::var_os("SHELL");
        let plan = match preview_cli_path_configure(home, shell.as_deref()) {
            Ok(plan) => plan,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let display_target =
            PrivateDisplayText::new(format!("<user-home>/{}", plan.profile_name()));
        let plan_sha256 = plan.plan_sha256().to_owned();
        self.active_operation = Some(PendingDesktopOperation::CliPathConfigure { token, plan });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::CliPathConfigure,
            title: "Configure Qiongli CLI PATH",
            summary: "Append one receipt-bound Qiongli marker to the selected supported shell profile after verifying its exact previewed bytes.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(plan_sha256),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn refresh_zotero_integration(&mut self) -> DesktopEvent {
        self.zotero = refreshed_zotero_service_snapshot(&self.environment);
        DesktopEvent::SnapshotReplaced(Box::new(self.snapshot()))
    }

    fn preview_zotero_companion_stage(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        if detect_zotero_application(&self.environment)
            .as_ref()
            .is_some_and(|application| {
                zotero_version_is_incompatible(application.version.as_deref())
            })
        {
            return DesktopEvent::Failed {
                code: "zotero-version-incompatible",
            };
        }
        let artifact = match crate::embedded_zotero_companion() {
            Ok(artifact) => artifact,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let root = match config_root(&self.environment) {
            Ok(root) => root,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        let plan = match preview_zotero_companion_stage(root.state_root(), &artifact) {
            Ok(plan) => plan,
            Err(error) => {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
        };
        if plan.effect() == ZoteroCompanionStageEffect::AlreadyCurrent {
            self.zotero = zotero_service_snapshot(&self.environment);
            return DesktopEvent::Completed {
                code: "zotero-companion-already-prepared",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let plan_digest_sha256 = plan.plan_digest_sha256().to_owned();
        let display_target = PrivateDisplayText::new(format!(
            "<qiongli-state>/zotero/companion/{}-{}",
            plan.companion_version(),
            &plan.artifact_sha256()[..16]
        ));
        self.active_operation = Some(PendingDesktopOperation::ZoteroCompanionStage {
            token,
            plan: Box::new(plan),
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::ZoteroCompanionStage,
            title: "Prepare Zotero Companion installation",
            summary: "Copy the verified Companion XPI and its receipt into Qiongli-owned state. Zotero remains responsible for plugin confirmation, activation, and profile changes.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(plan_digest_sha256),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn verify_zotero_integration(&mut self) -> DesktopEvent {
        let mut view = refreshed_zotero_service_snapshot(&self.environment);
        let client = match CompanionClient::with_timeout(
            DEFAULT_CONNECTOR_URL,
            Duration::from_millis(900),
        ) {
            Ok(client) => client,
            Err(_) => {
                view.status = StatusCode::Unavailable;
                view.state = ZoteroIntegrationStateView::NotObservable;
                view.observation = ZoteroObservationView::NotObservable;
                view.reason_code = "zotero-loopback-probe-unavailable";
                self.zotero = view;
                return DesktopEvent::SnapshotReplaced(Box::new(self.snapshot()));
            }
        };
        let status = client.probe(true);
        apply_zotero_live_observation(&mut view, &status);
        self.zotero = view;
        DesktopEvent::SnapshotReplaced(Box::new(self.snapshot()))
    }

    fn staged_zotero_companion_path(&self) -> Result<PathBuf, &'static str> {
        let root = config_root(&self.environment).map_err(|error| error.reason_code())?;
        let artifact = crate::embedded_zotero_companion().map_err(|error| error.reason_code())?;
        verify_zotero_companion_stage(root.state_root(), &artifact)
            .map_err(|error| error.reason_code())?
            .map(|stage| stage.xpi_path())
            .ok_or("zotero-companion-installation-not-prepared")
    }

    fn zotero_application_path(&self) -> Result<PathBuf, &'static str> {
        detect_zotero_application(&self.environment)
            .map(|application| application.path)
            .ok_or("zotero-application-not-detected")
    }

    fn select_skills_preset_target(
        &self,
        preset: SkillsDestinationPreset,
    ) -> Result<MaterializationTarget, &'static str> {
        let path = match preset {
            SkillsDestinationPreset::QiongliManaged => self
                .environment
                .platform_home()
                .ok_or("skills-home-unavailable")?
                .join(".qiongli-skills"),
            SkillsDestinationPreset::CurrentProject => self
                .environment
                .project_root()
                .ok_or("skills-project-unavailable")?
                .join(".qiongli-skills"),
            SkillsDestinationPreset::CustomFolder => self
                .selected_skills_target
                .as_ref()
                .ok_or("skills-destination-required")?
                .path()
                .to_path_buf(),
            SkillsDestinationPreset::DetectedCodex
            | SkillsDestinationPreset::DetectedClaudeCode => {
                return Err("skills-preset-client-managed");
            }
        };
        approve_materialization_target(path).map_err(|error| error.reason_code())
    }

    fn preview_skills_preset_materialization(
        &mut self,
        profile: ProfileKind,
        preset: SkillsDestinationPreset,
    ) -> DesktopEvent {
        match preset {
            SkillsDestinationPreset::DetectedCodex => {
                self.preview_activation(IntegrationTarget::Codex)
            }
            SkillsDestinationPreset::DetectedClaudeCode => {
                self.preview_activation(IntegrationTarget::ClaudeCode)
            }
            _ => match self.select_skills_preset_target(preset) {
                Ok(target) => {
                    self.selected_skills_target = Some(target);
                    self.preview_skills_materialization(profile)
                }
                Err(code) => DesktopEvent::ValidationFailed { code },
            },
        }
    }

    fn verify_skills_preset(&mut self, preset: SkillsDestinationPreset) -> DesktopEvent {
        match preset {
            SkillsDestinationPreset::DetectedCodex => {
                self.verify_packaged_integrations(IntegrationSelection {
                    codex: true,
                    claude_code: false,
                })
            }
            SkillsDestinationPreset::DetectedClaudeCode => {
                self.verify_packaged_integrations(IntegrationSelection {
                    codex: false,
                    claude_code: true,
                })
            }
            _ => match self.select_skills_preset_target(preset) {
                Ok(target) => {
                    self.selected_skills_target = Some(target);
                    self.verify_skills_materialization()
                }
                Err(code) => DesktopEvent::ValidationFailed { code },
            },
        }
    }

    fn preview_skills_preset_removal(&mut self, preset: SkillsDestinationPreset) -> DesktopEvent {
        match preset {
            SkillsDestinationPreset::DetectedCodex => {
                self.preview_packaged_product_removal(IntegrationSelection {
                    codex: true,
                    claude_code: false,
                })
            }
            SkillsDestinationPreset::DetectedClaudeCode => {
                self.preview_packaged_product_removal(IntegrationSelection {
                    codex: false,
                    claude_code: true,
                })
            }
            _ => match self.select_skills_preset_target(preset) {
                Ok(target) => {
                    self.selected_skills_target = Some(target);
                    self.preview_skills_removal()
                }
                Err(code) => DesktopEvent::ValidationFailed { code },
            },
        }
    }

    fn resolve_managed_skills_target(
        &self,
        target_id: &str,
    ) -> Result<
        (
            MaterializationTarget,
            ProfileKind,
            ManagedSkillsStateView,
            String,
        ),
        &'static str,
    > {
        target_id
            .strip_prefix("skills-target-")
            .filter(|digest| {
                digest.len() == 64
                    && digest
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
            .ok_or("managed-skills-target-id-invalid")?;
        let root = config_root(&self.environment).map_err(|error| error.reason_code())?;
        let registry = crate::managed_content::load_managed_content_registry(root.state_root())?;
        let variant = WorkflowVariantStore::new(root)
            .load(self.content.pack())
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
        let target = approve_materialization_target(Path::new(&entry.target))
            .map_err(|error| error.reason_code())?;
        let view = managed_skills_entry_view(
            entry,
            SkillsDestinationPreset::CustomFolder,
            &self.content,
            variant.variant_sha256(),
        );
        Ok((
            target,
            profile_from_content(entry.profile),
            view.state,
            entry.receipt_sha256.clone(),
        ))
    }

    fn verify_managed_skills_target(&mut self, target_id: &str) -> DesktopEvent {
        self.cancel_active_operation();
        match self.resolve_managed_skills_target(target_id) {
            Ok((
                target,
                _,
                ManagedSkillsStateView::Current | ManagedSkillsStateView::UpdateAvailable,
                _,
            )) => {
                self.selected_skills_target = Some(target);
                DesktopEvent::Completed {
                    code: "managed-skills-target-verified",
                }
            }
            Ok((_, _, ManagedSkillsStateView::Drifted, _)) => DesktopEvent::Completed {
                code: "managed-skills-target-drift-confirmed",
            },
            Ok((_, _, ManagedSkillsStateView::Missing, _)) => DesktopEvent::Failed {
                code: "managed-skills-target-not-installed",
            },
            Ok((_, _, ManagedSkillsStateView::Unmanaged, _)) => DesktopEvent::Failed {
                code: "managed-skills-target-not-registered",
            },
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_managed_skills_target_update(&mut self, target_id: &str) -> DesktopEvent {
        self.cancel_active_operation();
        match self.resolve_managed_skills_target(target_id) {
            Ok((_, _, ManagedSkillsStateView::Current, _)) => DesktopEvent::Completed {
                code: "managed-skills-target-already-current",
            },
            Ok((target, profile, ManagedSkillsStateView::UpdateAvailable, _)) => {
                self.selected_skills_target = Some(target);
                self.preview_skills_materialization(profile)
            }
            Ok((_, _, ManagedSkillsStateView::Drifted, _)) => DesktopEvent::Failed {
                code: "managed-skills-target-drifted",
            },
            Ok((_, _, ManagedSkillsStateView::Missing, _)) => DesktopEvent::Failed {
                code: "managed-skills-target-not-installed",
            },
            Ok((_, _, ManagedSkillsStateView::Unmanaged, _)) => DesktopEvent::Failed {
                code: "managed-skills-target-not-registered",
            },
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_managed_skills_target_removal(&mut self, target_id: &str) -> DesktopEvent {
        self.cancel_active_operation();
        match self.resolve_managed_skills_target(target_id) {
            Ok((
                target,
                _,
                ManagedSkillsStateView::Current | ManagedSkillsStateView::UpdateAvailable,
                _,
            )) => {
                self.selected_skills_target = Some(target);
                self.preview_skills_removal()
            }
            Ok((_, _, ManagedSkillsStateView::Drifted, _)) => DesktopEvent::Failed {
                code: "managed-skills-target-drifted",
            },
            Ok((_, _, ManagedSkillsStateView::Missing, _)) => DesktopEvent::Failed {
                code: "managed-skills-target-not-installed",
            },
            Ok((_, _, ManagedSkillsStateView::Unmanaged, _)) => DesktopEvent::Failed {
                code: "managed-skills-target-not-registered",
            },
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_managed_skills_target_detach(&mut self, target_id: &str) -> DesktopEvent {
        self.cancel_active_operation();
        let (target, profile, state, expected_receipt_sha256) =
            match self.resolve_managed_skills_target(target_id) {
                Ok(resolved) => resolved,
                Err(code) => return DesktopEvent::Failed { code },
            };
        if state != ManagedSkillsStateView::Drifted {
            return DesktopEvent::Failed {
                code: "managed-skills-target-not-drifted",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let digest = skills_detach_digest(target_id, profile, &expected_receipt_sha256);
        let display_target =
            PrivateDisplayText::new(self.managed_skills_symbolic_target(target.path()));
        self.active_operation = Some(PendingDesktopOperation::SkillsDetach {
            token,
            target_id: target_id.to_owned(),
            expected_profile: profile,
            expected_receipt_sha256,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::SkillsDetach,
            title: "Preserve and detach managed Skills",
            summary: "Remove only Qiongli's ownership record. Every file in the drifted destination is retained unchanged and becomes user-managed.",
            display_target: Some(display_target),
            plan_digest_sha256: Some(digest),
            approvals_required: vec![OperationApproval::FilesystemWrite],
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn managed_skills_symbolic_target(&self, target: &Path) -> String {
        if self
            .environment
            .platform_home()
            .is_some_and(|home| target == home.join(".qiongli-skills"))
        {
            return "<user-home>/.qiongli-skills".to_owned();
        }
        if self
            .environment
            .project_root()
            .is_some_and(|project| target == project.join(".qiongli-skills"))
        {
            return "<project>/.qiongli-skills".to_owned();
        }
        "<custom-folder>".to_owned()
    }

    fn preview_packaged_product_batch(
        &mut self,
        selection: IntegrationSelection,
        title: &'static str,
        summary: &'static str,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        if selection.is_empty() {
            return DesktopEvent::ValidationFailed {
                code: "integration-selection-required",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let variant = match self.load_workflow_variant() {
            Ok(variant) => variant,
            Err(code) => return DesktopEvent::Failed { code },
        };
        match self.packaged_product.preview_batch(
            &self.environment,
            &self.host_observations,
            token,
            selection,
            title,
            summary,
            variant.variant_sha256(),
        ) {
            Ok(preview) => {
                self.active_operation = if preview.can_confirm {
                    Some(PendingDesktopOperation::PackagedProductBatch { token, selection })
                } else {
                    Some(PendingDesktopOperation::Blocked(token))
                };
                DesktopEvent::PreviewReady(preview)
            }
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_integration_reconciliation(
        &mut self,
        selection: IntegrationSelection,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        self.refresh_host_integration_observations(selection);
        let integrations = self.snapshot().integrations;
        match integration_reconcile_required(
            [
                (integrations[0].next_action, integrations[0].compatibility),
                (integrations[1].next_action, integrations[1].compatibility),
            ],
            selection,
        ) {
            Err(code) => DesktopEvent::ValidationFailed { code },
            Ok(false) => DesktopEvent::Completed {
                code: "packaged-product-reconcile-not-required",
            },
            Ok(true) => self.preview_integration_content_reconciliation(selection),
        }
    }

    fn preview_integration_content_reconciliation(
        &mut self,
        selection: IntegrationSelection,
    ) -> DesktopEvent {
        let variant = match self.load_workflow_variant() {
            Ok(variant) => variant,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let targets = match selected_activation_targets(selection) {
            Ok(targets) => targets,
            Err(code) => return DesktopEvent::ValidationFailed { code },
        };
        let Some(product) = self.packaged_product.product.as_ref() else {
            return DesktopEvent::Failed {
                code: self.packaged_product.blocked_reason,
            };
        };
        let mut replacements = Vec::new();
        for target in &targets {
            if verify_packaged_product_install_with_variant(
                product,
                *target,
                variant.variant_sha256(),
            )
            .is_ok()
            {
                continue;
            }
            if let Err(error) = verify_receipt_owned_packaged_product_install(product, *target) {
                return DesktopEvent::Failed {
                    code: error.reason_code(),
                };
            }
            replacements.push(*target);
        }
        if replacements.is_empty() {
            return self.preview_packaged_product_batch(
                selection,
                "Update or repair selected integrations",
                "Run the approved official Host CLI repair plan and verify fresh Ready evidence.",
            );
        }

        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let transaction_id = format!("update-{:032x}", token.value());
        let hosts = match targets
            .iter()
            .map(|target| {
                prepare_host_plugin_plan(
                    &self.environment,
                    *target,
                    PackagedProductInstallEffect::Repair,
                    HostIntegrationObservation {
                        activation: HostProbeState::CacheDrift,
                        mcp_attachment: HostProbeState::CacheDrift,
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(hosts) => hosts,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let store = match update_store(&self.environment) {
            Ok(store) => store,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let claude_config_root = self
            .environment
            .claude_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| product.home().join(".claude"));
        let Some(codex_grant) = product
            .capability(ClientActivationTarget::Codex)
            .map(|capability| capability.grant())
        else {
            return DesktopEvent::Failed {
                code: "packaged-product-authority-unavailable",
            };
        };
        let Some(claude_grant) = product
            .capability(ClientActivationTarget::ClaudeCode)
            .map(|capability| capability.grant())
        else {
            return DesktopEvent::Failed {
                code: "packaged-product-authority-unavailable",
            };
        };
        let now_unix = match now_unix() {
            Ok(now_unix) => now_unix,
            Err(code) => return DesktopEvent::Failed { code },
        };
        if let Err(code) = crate::update_reconcile::prepare_reconciliation_transaction_root(
            &store,
            &transaction_id,
        ) {
            return DesktopEvent::Failed { code };
        }
        let prepared = crate::update_reconcile::prepare_update_reconciliation(
            &crate::update_reconcile::ReconciliationPreparation {
                store: &store,
                transaction_id: &transaction_id,
                target_version: &product.artifact().version,
                content: &self.content,
                platform_home: product.home(),
                claude_config_root: &claude_config_root,
                source_binary: product.current_executable(),
                codex_grant,
                claude_grant,
                workflow_overrides: variant.overrides(),
                targets: &replacements,
                now_unix,
            },
        );
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(code) => {
                let _ = crate::update_reconcile::discard_prepared_reconciliation(
                    &store,
                    &transaction_id,
                );
                let _ = crate::update_reconcile::remove_reconciliation_transaction_root(
                    &store,
                    &transaction_id,
                );
                return DesktopEvent::Failed { code };
            }
        };
        let plan_digest_sha256 = packaged_host_plan_digest(&prepared.journal_sha256, &hosts);
        self.active_operation = Some(PendingDesktopOperation::IntegrationReconciliation {
            token,
            selection,
            transaction_id,
            journal_sha256: prepared.journal_sha256,
            expected_variant_sha256: variant.variant_sha256().map(str::to_owned),
            hosts: hosts.clone(),
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind: OperationKind::Activation,
            title: "Update or repair selected integrations",
            summary: "Atomically apply the current Workflow and Skills content, run the official Host CLI repair plan, and verify fresh Ready evidence.",
            display_target: Some(integration_host_plan_display_targets(&hosts)),
            plan_digest_sha256: Some(plan_digest_sha256),
            approvals_required: OperationApproval::ACTIVATION.to_vec(),
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn confirm_integration_content_reconciliation(
        &mut self,
        selection: IntegrationSelection,
        transaction_id: &str,
        expected_journal_sha256: &str,
        expected_variant_sha256: Option<&str>,
        hosts: &[HostPluginPlan],
    ) -> DesktopEvent {
        let variant = match self.load_workflow_variant() {
            Ok(variant) if variant.variant_sha256() == expected_variant_sha256 => variant,
            Ok(_) => {
                return DesktopEvent::Failed {
                    code: "workflow-variant-digest-conflict",
                };
            }
            Err(code) => return DesktopEvent::Failed { code },
        };
        self.refresh_host_integration_observations(selection);
        let current_hosts = match hosts
            .iter()
            .map(|host| {
                prepare_host_plugin_plan(
                    &self.environment,
                    host.target,
                    PackagedProductInstallEffect::Repair,
                    HostIntegrationObservation {
                        activation: HostProbeState::CacheDrift,
                        mcp_attachment: HostProbeState::CacheDrift,
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(hosts) => hosts,
            Err(code) => return DesktopEvent::Failed { code },
        };
        if current_hosts != hosts {
            return DesktopEvent::Failed {
                code: "host-plugin-plan-changed",
            };
        }
        let store = match update_store(&self.environment) {
            Ok(store) => store,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let journal =
            match crate::update_reconcile::load_reconciliation_journal(&store, transaction_id) {
                Ok(journal) => journal,
                Err(code) => return DesktopEvent::Failed { code },
            };
        let journal_sha256 = match crate::update_reconcile::reconciliation_journal_sha256(&journal)
        {
            Ok(digest) => digest,
            Err(code) => return DesktopEvent::Failed { code },
        };
        if journal_sha256 != expected_journal_sha256
            || crate::update_reconcile::verify_prepared_reconciliation(&journal).is_err()
        {
            return DesktopEvent::Failed {
                code: "native-update-reconciliation-invalid",
            };
        }
        if let Err(code) = crate::update_reconcile::activate_prepared_reconciliation(&journal) {
            let _ =
                crate::update_reconcile::discard_prepared_reconciliation(&store, transaction_id);
            let _ = crate::update_reconcile::remove_reconciliation_transaction_root(
                &store,
                transaction_id,
            );
            return DesktopEvent::Failed { code };
        }
        let readiness =
            execute_host_plugin_plans(&self.environment, hosts, &mut self.host_observations)
                .and_then(|()| {
                    self.packaged_product
                        .verify(selection, variant.variant_sha256())
                        .map(|_| ())
                });
        if let Err(code) = readiness {
            let recovered = crate::update_reconcile::rollback_active_reconciliation(&journal)
                .and_then(|()| {
                    crate::update_reconcile::cleanup_rolled_back_reconciliation(&journal)
                })
                .and_then(|()| {
                    crate::update_reconcile::remove_committed_reconciliation_journal(
                        &store,
                        transaction_id,
                    )
                })
                .and_then(|()| {
                    crate::update_reconcile::remove_reconciliation_transaction_root(
                        &store,
                        transaction_id,
                    )
                });
            return DesktopEvent::Failed {
                code: if recovered.is_ok() {
                    code
                } else {
                    "native-update-reconciliation-recovery-required"
                },
            };
        }
        let cleanup = crate::update_reconcile::cleanup_committed_reconciliation(&journal)
            .and_then(|()| {
                crate::update_reconcile::remove_committed_reconciliation_journal(
                    &store,
                    transaction_id,
                )
            })
            .and_then(|()| {
                crate::update_reconcile::remove_reconciliation_transaction_root(
                    &store,
                    transaction_id,
                )
            });
        match cleanup {
            Ok(()) => DesktopEvent::Completed {
                code: "qiongli-plugin-ready-restart-host",
            },
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_integration_installation(
        &mut self,
        selection: IntegrationSelection,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        self.refresh_host_integration_observations(selection);
        let integrations = self.snapshot().integrations;
        match integration_install_required(
            [
                (
                    integrations[0].next_action,
                    integrations[0].compatibility,
                ),
                (
                    integrations[1].next_action,
                    integrations[1].compatibility,
                ),
            ],
            selection,
        ) {
            Err(code) => DesktopEvent::ValidationFailed { code },
            Ok(false) => DesktopEvent::Completed {
                code: "packaged-product-install-not-required",
            },
            Ok(true) => self.preview_packaged_product_batch(
                selection,
                "Install selected integrations",
                "Install only the selected missing clients while preserving selected current or receipt-owned repair targets in one compensating transaction.",
            ),
        }
    }

    fn prepare_legacy_migration(
        &mut self,
        provider_resolutions: Vec<LegacyProviderResolutionView>,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        let migration = self.snapshot().legacy_migration;
        if migration.next_action != LegacyMigrationActionView::Start {
            return DesktopEvent::ValidationFailed {
                code: if migration.next_action == LegacyMigrationActionView::Review {
                    "legacy-migration-review-required"
                } else {
                    "legacy-migration-start-state-invalid"
                },
            };
        }
        if self.packaged_product.product.is_none() {
            return DesktopEvent::Failed {
                code: self.packaged_product.blocked_reason,
            };
        }
        match crate::legacy_migration_cli::execute_with_secret_store(
            crate::legacy_migration_cli::LegacyMigrationCliCommand::Preview {
                provider_resolutions: provider_resolutions
                    .into_iter()
                    .map(legacy_provider_resolution)
                    .collect(),
            },
            &self.environment,
            &self.content,
            self.secret_store.as_ref(),
        ) {
            Ok(crate::legacy_migration_cli::LegacyMigrationCliOutput::Preview { .. }) => {
                DesktopEvent::Completed {
                    code: "legacy-migration-preview-ready",
                }
            }
            Ok(_) => DesktopEvent::Failed {
                code: "legacy-migration-preview-output-invalid",
            },
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_legacy_migration_next(&mut self) -> DesktopEvent {
        self.cancel_active_operation();
        let migration = self.snapshot().legacy_migration;
        let Some(migration_id) = migration.migration_id else {
            return DesktopEvent::ValidationFailed {
                code: match migration.next_action {
                    LegacyMigrationActionView::Start => "legacy-migration-preview-required",
                    LegacyMigrationActionView::Review => "legacy-migration-review-required",
                    _ => "legacy-migration-id-unavailable",
                },
            };
        };
        let plan = match crate::legacy_migration_cli::execute_with_secret_store(
            crate::legacy_migration_cli::LegacyMigrationCliCommand::Status {
                migration_id: migration_id.clone(),
            },
            &self.environment,
            &self.content,
            self.secret_store.as_ref(),
        ) {
            Ok(crate::legacy_migration_cli::LegacyMigrationCliOutput::Status { plan, .. }) => plan,
            Ok(_) => {
                return DesktopEvent::Failed {
                    code: "legacy-migration-status-output-invalid",
                };
            }
            Err(code) => return DesktopEvent::Failed { code },
        };
        let mut cleanup_approvals = vec![OperationApproval::FilesystemWrite];
        let client_config_change = plan
            .required_approvals
            .contains(&qiongli_platform::LegacyMigrationApproval::ClientConfigChange);
        if client_config_change {
            cleanup_approvals.push(OperationApproval::ClientConfigChange);
        }
        let mut stage_approvals = cleanup_approvals.clone();
        let secret_store_write = plan
            .required_approvals
            .contains(&qiongli_platform::LegacyMigrationApproval::SecretStoreWrite);
        if secret_store_write {
            stage_approvals.push(OperationApproval::SecretStoreWrite);
        }
        let (kind, title, summary, command, completion_code, approvals) =
            match migration.next_action {
                LegacyMigrationActionView::Apply => (
                    OperationKind::LegacyMigrationStage,
                    "Stage the Qiongli 2.x replacement",
                    "Install the exact 2.x client integration and convert recognized provider settings. Plaintext provider keys move to the secure store; all 1.x sources remain until verification.",
                    crate::legacy_migration_cli::LegacyMigrationCliCommand::Apply {
                        migration_id,
                        expected_plan_digest: plan.plan_sha256.clone(),
                        approve_filesystem_write: true,
                        approve_client_config_change: client_config_change,
                        approve_secret_store_write: secret_store_write,
                    },
                    "legacy-migration-awaiting-host-activation",
                    stage_approvals,
                ),
                LegacyMigrationActionView::ConfirmHostActivation => (
                    OperationKind::LegacyMigrationHostActivation,
                    "Verify the Qiongli 2.x replacement",
                    "Verify the exact packaged client installation plus converted provider settings and secure references. Legacy content remains untouched until every applicable check succeeds.",
                    crate::legacy_migration_cli::LegacyMigrationCliCommand::Continue {
                        migration_id,
                        action: crate::legacy_migration_cli::LegacyMigrationContinueAction::ConfirmHostActivation,
                    },
                    "legacy-migration-cleanup-ready",
                    vec![OperationApproval::HostTrust],
                ),
                LegacyMigrationActionView::Cleanup => (
                    OperationKind::LegacyMigrationCleanup,
                    "Remove verified Qiongli 1.x surfaces",
                    "Back up and remove only recognized Qiongli 1.x plugin, Skills, marketplace, MCP, and provider-config sources after their 2.x replacements have been verified.",
                    crate::legacy_migration_cli::LegacyMigrationCliCommand::Continue {
                        migration_id,
                        action: crate::legacy_migration_cli::LegacyMigrationContinueAction::Cleanup,
                    },
                    "legacy-migration-complete",
                    cleanup_approvals.clone(),
                ),
                LegacyMigrationActionView::Finalize => (
                    OperationKind::LegacyMigrationFinalize,
                    "Finalize Qiongli 1.x migration",
                    "Re-verify the 2.x installation and remove only the transaction-owned recovery backup. The migration receipt remains available.",
                    crate::legacy_migration_cli::LegacyMigrationCliCommand::Continue {
                        migration_id,
                        action: crate::legacy_migration_cli::LegacyMigrationContinueAction::Finalize,
                    },
                    "legacy-migration-finalized",
                    vec![OperationApproval::FilesystemWrite],
                ),
                LegacyMigrationActionView::Recover => (
                    OperationKind::LegacyMigrationRecovery,
                    "Recover interrupted Qiongli 1.x cleanup",
                    "Restore recognized 1.x content from the transaction-owned backup without overwriting content that changed after the migration.",
                    crate::legacy_migration_cli::LegacyMigrationCliCommand::Recover {
                        migration_id,
                    },
                    "legacy-migration-recovered-review-required",
                    cleanup_approvals,
                ),
                LegacyMigrationActionView::None
                | LegacyMigrationActionView::Start
                | LegacyMigrationActionView::Review => {
                    return DesktopEvent::ValidationFailed {
                        code: "legacy-migration-next-action-unavailable",
                    };
                }
            };
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        if self.packaged_product.product.is_none() {
            self.active_operation = Some(PendingDesktopOperation::Blocked(token));
            return DesktopEvent::PreviewReady(OperationPreview {
                token,
                kind,
                title,
                summary,
                display_target: None,
                plan_digest_sha256: None,
                approvals_required: Vec::new(),
                can_confirm: false,
                blocked_reason: Some(self.packaged_product.blocked_reason),
            });
        }
        if migration.next_action == LegacyMigrationActionView::Apply {
            let targets = crate::legacy_migration_cli::migration_targets(&plan);
            if !targets.is_empty() {
                let Some(product) = self.packaged_product.product.as_ref() else {
                    self.active_operation = Some(PendingDesktopOperation::Blocked(token));
                    return DesktopEvent::PreviewReady(OperationPreview {
                        token,
                        kind,
                        title,
                        summary,
                        display_target: None,
                        plan_digest_sha256: None,
                        approvals_required: Vec::new(),
                        can_confirm: false,
                        blocked_reason: Some(self.packaged_product.blocked_reason),
                    });
                };
                match preview_packaged_product_batch_install_with_variant(product, &targets, None) {
                    Ok(preview) if !preview.can_apply => {
                        let blocked_reason = if preview.installs.iter().any(|install| {
                            install.effect == PackagedProductInstallEffect::RecoveryRequired
                        }) {
                            "legacy-migration-host-install-recovery-required"
                        } else {
                            "legacy-migration-current-install-replacement-required"
                        };
                        self.active_operation = Some(PendingDesktopOperation::Blocked(token));
                        return DesktopEvent::PreviewReady(OperationPreview {
                            token,
                            kind,
                            title,
                            summary,
                            display_target: None,
                            plan_digest_sha256: None,
                            approvals_required: Vec::new(),
                            can_confirm: false,
                            blocked_reason: Some(blocked_reason),
                        });
                    }
                    Err(error) => {
                        self.active_operation = Some(PendingDesktopOperation::Blocked(token));
                        return DesktopEvent::PreviewReady(OperationPreview {
                            token,
                            kind,
                            title,
                            summary,
                            display_target: None,
                            plan_digest_sha256: None,
                            approvals_required: Vec::new(),
                            can_confirm: false,
                            blocked_reason: Some(error.reason_code()),
                        });
                    }
                    Ok(_) => {}
                }
            }
        }
        self.active_operation = Some(PendingDesktopOperation::LegacyMigration {
            token,
            command,
            completion_code,
        });
        DesktopEvent::PreviewReady(OperationPreview {
            token,
            kind,
            title,
            summary,
            display_target: None,
            plan_digest_sha256: Some(plan.plan_sha256),
            approvals_required: approvals,
            can_confirm: true,
            blocked_reason: None,
        })
    }

    fn recommended_integration_selection(&mut self) -> IntegrationSelection {
        let integrations = self.snapshot().integrations;
        IntegrationSelection {
            codex: integrations[0].next_action == IntegrationActionView::InstallReady,
            claude_code: integrations[1].next_action == IntegrationActionView::InstallReady,
        }
    }

    fn refresh_host_integration_observations(&mut self, selection: IntegrationSelection) {
        self.environment.detect_client_versions();
        probe_host_integrations(&self.environment, selection, &mut self.host_observations);
    }

    fn verify_packaged_integrations(&mut self, selection: IntegrationSelection) -> DesktopEvent {
        self.cancel_active_operation();
        if selection.is_empty() {
            return DesktopEvent::ValidationFailed {
                code: "integration-selection-required",
            };
        }
        if self.packaged_product.product.is_none() {
            self.refresh_host_integration_observations(selection);
            return DesktopEvent::Completed {
                code: "integration-inventory-refreshed-host-probed-read-only",
            };
        }
        let variant = match self.load_workflow_variant() {
            Ok(variant) => variant,
            Err(code) => return DesktopEvent::Failed { code },
        };
        match self
            .packaged_product
            .verify(selection, variant.variant_sha256())
        {
            Ok(_) => {
                self.refresh_host_integration_observations(selection);
                DesktopEvent::Completed {
                    code: "integration-files-verified-host-probed",
                }
            }
            Err(code) if integration_verification_needs_reconciliation(code) => {
                // Verification is an observation boundary. A stale, drifted, or
                // older receipt-owned source must be represented in the refreshed
                // integration snapshot so the user can reconcile it explicitly;
                // it is not an execution failure.
                self.refresh_host_integration_observations(selection);
                DesktopEvent::Completed {
                    code: "integration-inventory-refreshed-reconciliation-required",
                }
            }
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_packaged_product_removal(
        &mut self,
        selection: IntegrationSelection,
    ) -> DesktopEvent {
        self.cancel_active_operation();
        if selection.is_empty() {
            return DesktopEvent::ValidationFailed {
                code: "integration-selection-required",
            };
        }
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        match self.packaged_product.preview_remove(token, selection) {
            Ok(preview) => {
                self.active_operation = if preview.can_confirm {
                    Some(PendingDesktopOperation::PackagedProductRemoval { token, selection })
                } else {
                    Some(PendingDesktopOperation::Blocked(token))
                };
                DesktopEvent::PreviewReady(preview)
            }
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn preview_activation(&mut self, target: IntegrationTarget) -> DesktopEvent {
        self.cancel_active_operation();
        let token = match Self::next_operation_token() {
            Ok(token) => token,
            Err(code) => return DesktopEvent::Failed { code },
        };
        let now_unix = match now_unix() {
            Ok(now_unix) => now_unix,
            Err(code) => return DesktopEvent::Failed { code },
        };
        if let Some(session) = self
            .candidate_sessions
            .iter_mut()
            .find(|session| session.target == target)
        {
            return match session.preview(token, now_unix) {
                Ok(preview) => {
                    self.active_operation =
                        Some(PendingDesktopOperation::Candidate { token, target });
                    DesktopEvent::PreviewReady(preview)
                }
                Err(code) => DesktopEvent::Failed { code },
            };
        }
        let Some(session) = self
            .activation_sessions
            .iter_mut()
            .find(|session| session.target == target)
        else {
            self.refresh_host_integration_observations(integration_selection(target));
            let variant = match self.load_workflow_variant() {
                Ok(variant) => variant,
                Err(code) => return DesktopEvent::Failed { code },
            };
            return match self.packaged_product.preview(
                &self.environment,
                self.host_observations[host_plugin_target_index(activation_target(target))],
                token,
                target,
                variant.variant_sha256(),
            ) {
                Ok(preview) => {
                    self.active_operation = if preview.can_confirm {
                        Some(PendingDesktopOperation::PackagedProduct { token, target })
                    } else {
                        Some(PendingDesktopOperation::Blocked(token))
                    };
                    DesktopEvent::PreviewReady(preview)
                }
                Err(code) => DesktopEvent::Failed { code },
            };
        };
        match session.preview(token, now_unix) {
            Ok(preview) => {
                self.active_operation = Some(PendingDesktopOperation::Activation { token, target });
                DesktopEvent::PreviewReady(preview)
            }
            Err(code) => DesktopEvent::Failed { code },
        }
    }

    fn cancel_active_operation(&mut self) {
        if let Some(PendingDesktopOperation::Activation { target, .. }) = &self.active_operation
            && let Some(session) = self
                .activation_sessions
                .iter_mut()
                .find(|session| session.target == *target)
        {
            session.cancel();
        }
        if let Some(PendingDesktopOperation::Candidate { target, .. }) = &self.active_operation
            && let Some(session) = self
                .candidate_sessions
                .iter_mut()
                .find(|session| session.target == *target)
        {
            session.cancel();
        }
        if matches!(
            self.active_operation,
            Some(
                PendingDesktopOperation::PackagedProduct { .. }
                    | PendingDesktopOperation::PackagedProductBatch { .. }
                    | PendingDesktopOperation::PackagedProductRemoval { .. }
            )
        ) {
            self.packaged_product.cancel();
        }
        if let Some(PendingDesktopOperation::IntegrationReconciliation { transaction_id, .. }) =
            &self.active_operation
            && let Ok(store) = update_store(&self.environment)
        {
            let _ =
                crate::update_reconcile::discard_prepared_reconciliation(&store, transaction_id);
            let _ = crate::update_reconcile::remove_reconciliation_transaction_root(
                &store,
                transaction_id,
            );
        }
        self.active_operation = None;
    }
}

fn apply_public_email_change(
    current: &mut Option<EmailAddress>,
    change: PublicSettingChange,
) -> Result<(), &'static str> {
    match change {
        PublicSettingChange::Keep => Ok(()),
        PublicSettingChange::Clear => {
            *current = None;
            Ok(())
        }
        PublicSettingChange::Replace(value) => {
            *current = Some(
                EmailAddress::parse(value.expose())
                    .map_err(|_| "provider-public-setting-invalid")?,
            );
            Ok(())
        }
    }
}

fn global_settings_patch_digest(expected_revision: u64, settings: &GlobalSettings) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-GLOBAL-SETTINGS-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hash_component(
        &mut hasher,
        profile_from_content(settings.default_profile)
            .id()
            .as_bytes(),
    );
    hasher.update([
        u8::from(settings.providers.openalex.enabled),
        u8::from(settings.providers.semantic_scholar.enabled),
        u8::from(settings.providers.crossref.enabled),
        u8::from(settings.providers.pubmed.enabled),
        u8::from(settings.providers.arxiv.enabled),
        u8::from(settings.providers.openalex.api_key_ref.is_some()),
        u8::from(settings.providers.semantic_scholar.api_key_ref.is_some()),
        u8::from(settings.providers.pubmed.api_key_ref.is_some()),
    ]);
    hash_optional_email(&mut hasher, settings.providers.openalex.email.as_ref());
    hash_optional_email(&mut hasher, settings.providers.crossref.email.as_ref());
    lower_hex(&hasher.finalize())
}

fn new_secret_ref() -> Result<SecretRef, &'static str> {
    let mut identifier = [0_u8; 16];
    getrandom::fill(&mut identifier).map_err(|_| "provider-secret-reference-unavailable")?;
    SecretRef::parse(&format!("qsr1_{}", lower_hex(&identifier)))
        .map_err(|_| "provider-secret-reference-unavailable")
}

fn new_agent_backend_secret_ref() -> Result<SecretRef, &'static str> {
    let mut identifier = [0_u8; 16];
    getrandom::fill(&mut identifier).map_err(|_| "agent-backend-secret-reference-unavailable")?;
    SecretRef::parse(&format!("qsr1_{}", lower_hex(&identifier)))
        .map_err(|_| "agent-backend-secret-reference-unavailable")
}

fn provider_secret_digest(
    expected_revision: u64,
    provider: ProviderKind,
    secret_ref: &SecretRef,
    replacing: bool,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-PROVIDER-SECRET-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hasher.update([provider as u8, u8::from(replacing)]);
    hash_component(&mut hasher, secret_ref.storage_key().as_bytes());
    lower_hex(&hasher.finalize())
}

fn agent_backend_settings_digest(expected_revision: u64, settings: &GlobalSettings) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-AGENT-BACKEND-SETTINGS-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hasher.update([
        u8::from(settings.agent_backends.openai.enabled),
        u8::from(settings.agent_backends.openai.api_key_ref.is_some()),
    ]);
    lower_hex(&hasher.finalize())
}

fn agent_backend_secret_digest(
    expected_revision: u64,
    secret_ref: &SecretRef,
    replacing: bool,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-AGENT-BACKEND-SECRET-V1\0");
    hasher.update(expected_revision.to_be_bytes());
    hasher.update([u8::from(replacing)]);
    hash_component(&mut hasher, secret_ref.storage_key().as_bytes());
    lower_hex(&hasher.finalize())
}

fn agent_run_digest(project_id: &ProjectId, project_revision: u64, prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-AGENT-RUN-V1\0");
    hash_component(&mut hasher, project_id.as_str().as_bytes());
    hasher.update(project_revision.to_be_bytes());
    hash_component(&mut hasher, prompt.as_bytes());
    lower_hex(&hasher.finalize())
}

fn project_guidance_digest(
    project_id: &ProjectId,
    expected_sha256: Option<&str>,
    content: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-PROJECT-GUIDANCE-V1\0");
    hash_component(&mut hasher, project_id.as_str().as_bytes());
    hash_component(&mut hasher, expected_sha256.unwrap_or_default().as_bytes());
    hash_component(&mut hasher, content.as_bytes());
    lower_hex(&hasher.finalize())
}

fn agent_run_result_view(result: AgentRunResultV1) -> AgentRunResultView {
    AgentRunResultView {
        schema_version: result.schema_version,
        run_id: result.run_id.as_str().to_owned(),
        backend_id: result.backend_id.as_str().to_owned(),
        model: result.model,
        finish_reason: match result.finish_reason {
            AgentFinishReason::Stop => "stop",
            AgentFinishReason::Length => "length",
            AgentFinishReason::ToolRequest => "tool-request",
        },
        content: PrivateDisplayText::new(result.content),
        input_tokens: result.provider_usage.input_tokens,
        output_tokens: result.provider_usage.output_tokens,
        cached_input_tokens: result.provider_usage.cached_input_tokens,
        model_turns: result.execution_usage.model_turns,
        tool_calls: result.execution_usage.tool_calls,
        network_requests: result.execution_usage.network_requests,
        audited_tool_calls: result.tool_audits.len(),
    }
}

fn skills_materialization_digest(
    pack_sha256: &str,
    variant_sha256: Option<&str>,
    profile: ProfileKind,
    path: &Path,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-SKILLS-MATERIALIZATION-V2\0");
    hash_component(&mut hasher, pack_sha256.as_bytes());
    hash_component(
        &mut hasher,
        variant_sha256.unwrap_or("canonical").as_bytes(),
    );
    hash_component(&mut hasher, profile.id().as_bytes());
    hash_path(&mut hasher, path);
    lower_hex(&hasher.finalize())
}

fn skills_removal_digest(receipt: &MaterializationReceiptV1, path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-SKILLS-REMOVAL-V1\0");
    hash_component(&mut hasher, receipt.pack_sha256.as_bytes());
    hash_component(&mut hasher, receipt.content_root_sha256.as_bytes());
    hash_component(
        &mut hasher,
        profile_from_content(receipt.profile).id().as_bytes(),
    );
    hasher.update(
        u64::try_from(receipt.entries.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for entry in &receipt.entries {
        hash_component(&mut hasher, entry.path.as_bytes());
        hash_component(&mut hasher, entry.sha256.as_bytes());
    }
    hash_path(&mut hasher, path);
    lower_hex(&hasher.finalize())
}

fn skills_detach_digest(
    target_id: &str,
    profile: ProfileKind,
    expected_receipt_sha256: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-SKILLS-DETACH-V1\0");
    hash_component(&mut hasher, target_id.as_bytes());
    hash_component(&mut hasher, profile.id().as_bytes());
    hash_component(&mut hasher, expected_receipt_sha256.as_bytes());
    lower_hex(&hasher.finalize())
}

fn packaged_product_removal_digest(
    product: &VerifiedPackagedProduct,
    verifications: &[PackagedProductInstallVerification],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-PACKAGED-PRODUCT-REMOVAL-V1\0");
    hash_component(&mut hasher, product.control_sha256().as_bytes());
    for verification in verifications {
        hasher.update([verification.target as u8]);
        hash_component(
            &mut hasher,
            verification.activation_transaction_id.as_bytes(),
        );
        hash_component(&mut hasher, verification.source.binary_sha256.as_bytes());
        hash_component(
            &mut hasher,
            verification.source.resource_pack_sha256.as_bytes(),
        );
    }
    lower_hex(&hasher.finalize())
}

fn hash_optional_email(hasher: &mut Sha256, value: Option<&EmailAddress>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            hash_component(hasher, value.as_str().as_bytes());
        }
        None => hasher.update([0]),
    }
}

fn hash_component(hasher: &mut Sha256, value: &[u8]) {
    hasher.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(value);
}

#[cfg(unix)]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    use std::os::unix::ffi::OsStrExt;

    hash_component(hasher, path.as_os_str().as_bytes());
}

#[cfg(windows)]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    use std::os::windows::ffi::OsStrExt;

    let encoded = path
        .as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    hash_component(hasher, &encoded);
}

#[cfg(not(any(unix, windows)))]
fn hash_path(hasher: &mut Sha256, path: &Path) {
    hash_component(hasher, path.to_string_lossy().as_bytes());
}

fn display_path(path: &Path) -> String {
    let mut display = path
        .to_string_lossy()
        .chars()
        .map(|character| {
            if character.is_control() {
                '�'
            } else {
                character
            }
        })
        .take(1_024)
        .collect::<String>();
    if display.is_empty() {
        display.push_str("<selected-folder>");
    }
    display
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    value
}

fn mcp_self_test_counts(snapshot: &DesktopSnapshotV1) -> McpSelfTestCounts {
    McpSelfTestCounts {
        enabled_providers: snapshot
            .config
            .providers
            .iter()
            .filter(|provider| provider.enabled)
            .count(),
        ready_providers: snapshot
            .config
            .providers
            .iter()
            .filter(|provider| {
                provider.enabled && provider.readiness == ProviderReadinessView::Ready
            })
            .count(),
        discovered_clients: snapshot
            .integrations
            .iter()
            .filter(|integration| {
                !matches!(
                    integration.discovery,
                    IntegrationDiscoveryState::NotDiscovered
                        | IntegrationDiscoveryState::Unavailable
                )
            })
            .count(),
        registered_clients: snapshot
            .integrations
            .iter()
            .filter(|integration| {
                integration.registration == StatusCode::Ready
                    && !matches!(
                        integration.discovery,
                        IntegrationDiscoveryState::NotDiscovered
                            | IntegrationDiscoveryState::Unavailable
                    )
            })
            .count(),
    }
}

const fn mcp_counts_from_view(view: &McpSelfTestView) -> McpSelfTestCounts {
    McpSelfTestCounts {
        enabled_providers: view.enabled_provider_count,
        ready_providers: view.ready_provider_count,
        discovered_clients: view.discovered_client_count,
        registered_clients: view.registered_client_count,
    }
}

pub(crate) fn update_store(
    environment: &CommandEnvironment,
) -> Result<UpdateStateStore, &'static str> {
    let root = config_root(environment).map_err(|error| error.reason_code())?;
    Ok(UpdateStateStore::new(root, default_update_stream()))
}

fn default_update_stream() -> UpdateStreamPreference {
    if env!("CARGO_PKG_VERSION").contains("-alpha.") || env!("CARGO_PKG_VERSION").contains("-beta.")
    {
        UpdateStreamPreference::Beta
    } else {
        UpdateStreamPreference::Stable
    }
}

fn with_native_update_context<T>(
    environment: &CommandEnvironment,
    operation: impl FnOnce(
        &UpdateStateStore,
        Option<&qiongli_platform::NativeReleaseAuthority>,
        &EmbeddedContent,
    ) -> Result<T, &'static str>,
) -> Result<T, &'static str> {
    let store = update_store(environment)?;
    let authority = crate::embedded_release_authority()
        .map_err(|_| "native-update-release-authority-invalid")?;
    let content = crate::embedded_content().map_err(|_| "desktop-content-load-failed")?;
    operation(&store, authority.as_ref(), &content)
}

fn update_snapshot(environment: &CommandEnvironment) -> UpdateView {
    let stream = update_stream_view(default_update_stream());
    if OperatingSystem::current() != Some(OperatingSystem::Macos)
        || Architecture::current() != Some(Architecture::Aarch64)
    {
        return update_unavailable_view(
            stream,
            "native-update-target-unsupported",
            UpdateRemediation::UseSupportedPlatform,
        );
    }
    let authority = match crate::embedded_release_authority() {
        Ok(Some(authority)) => authority,
        Ok(None) => {
            return update_unavailable_view(
                stream,
                "native-update-release-authority-unavailable",
                UpdateRemediation::InstallTrustedRelease,
            );
        }
        Err(_) => {
            return update_unavailable_view(
                stream,
                "native-update-release-authority-invalid",
                UpdateRemediation::ReinstallApplication,
            );
        }
    };
    if authority
        .validate_product_version(env!("CARGO_PKG_VERSION"))
        .is_err()
    {
        return update_unavailable_view(
            stream,
            "native-update-release-authority-invalid",
            UpdateRemediation::InstallTrustedRelease,
        );
    }
    if crate::embedded_macos_team_id().is_none() {
        return update_unavailable_view(
            stream,
            "native-update-local-build-unavailable",
            UpdateRemediation::InstallTrustedRelease,
        );
    }
    let store = match update_store(environment) {
        Ok(store) => store,
        Err(code) => return update_failure_base(stream, None, code, false, false),
    };
    let loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(ConfigError::RecoveryRequired) => {
            return UpdateView {
                status: StatusCode::RecoveryRequired,
                selected_stream: stream,
                phase: UpdatePhaseView::RecoveryRequired,
                available_version: None,
                archive_size_bytes: None,
                progress: None,
                reason_code: "native-update-recovery-required",
                remediation: UpdateRemediation::RestartApplication,
                can_select_stream: false,
                can_check: false,
                can_prepare: false,
                can_install: false,
                can_cancel: false,
            };
        }
        Err(error) => {
            return update_failure_base(stream, None, error.reason_code(), false, false);
        }
    };
    let stream = update_stream_view(loaded.state.selected_stream);
    let Some(transaction) = loaded.state.active_transaction else {
        return update_idle_view(stream);
    };
    match transaction.phase {
        UpdateTransactionPhase::Downloading | UpdateTransactionPhase::Cancelling => {
            update_failure_base(
                stream,
                Some(transaction.target_version),
                "native-update-preparation-interrupted",
                true,
                false,
            )
        }
        UpdateTransactionPhase::Downloaded | UpdateTransactionPhase::Verified => {
            update_failure_base(
                stream,
                Some(transaction.target_version),
                "native-update-preparation-interrupted",
                true,
                true,
            )
        }
        UpdateTransactionPhase::Staged | UpdateTransactionPhase::ReconciliationPrepared => {
            UpdateView {
                status: StatusCode::Attention,
                selected_stream: stream,
                phase: UpdatePhaseView::ReadyToInstall,
                available_version: Some(transaction.target_version),
                archive_size_bytes: None,
                progress: Some(UpdateProgressView {
                    completed_steps: 3,
                    total_steps: 4,
                    label: "Verified and prepared",
                    indeterminate: false,
                }),
                reason_code: "update-ready-to-install",
                remediation: UpdateRemediation::None,
                can_select_stream: false,
                can_check: false,
                can_prepare: false,
                can_install: true,
                can_cancel: true,
            }
        }
        UpdateTransactionPhase::AwaitingExit
        | UpdateTransactionPhase::Activating
        | UpdateTransactionPhase::HealthWindow => UpdateView {
            status: StatusCode::Busy,
            selected_stream: stream,
            phase: UpdatePhaseView::AwaitingRestart,
            available_version: Some(transaction.target_version),
            archive_size_bytes: None,
            progress: Some(UpdateProgressView {
                completed_steps: 4,
                total_steps: 4,
                label: "Completing application replacement",
                indeterminate: true,
            }),
            reason_code: "update-restart-in-progress",
            remediation: UpdateRemediation::RestartApplication,
            can_select_stream: false,
            can_check: false,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        },
        UpdateTransactionPhase::RecoveryRequired => UpdateView {
            status: StatusCode::RecoveryRequired,
            selected_stream: stream,
            phase: UpdatePhaseView::RecoveryRequired,
            available_version: Some(transaction.target_version),
            archive_size_bytes: None,
            progress: None,
            reason_code: "native-update-recovery-required",
            remediation: UpdateRemediation::ReinstallApplication,
            can_select_stream: false,
            can_check: false,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        },
    }
}

fn update_idle_view(stream: UpdateStreamView) -> UpdateView {
    UpdateView {
        status: StatusCode::Ready,
        selected_stream: stream,
        phase: UpdatePhaseView::Idle,
        available_version: None,
        archive_size_bytes: None,
        progress: None,
        reason_code: "update-ready",
        remediation: UpdateRemediation::None,
        can_select_stream: true,
        can_check: true,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    }
}

fn update_unavailable_view(
    stream: UpdateStreamView,
    reason_code: &'static str,
    remediation: UpdateRemediation,
) -> UpdateView {
    UpdateView {
        status: StatusCode::Unavailable,
        selected_stream: stream,
        phase: UpdatePhaseView::Unavailable,
        available_version: None,
        archive_size_bytes: None,
        progress: None,
        reason_code,
        remediation,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    }
}

fn update_busy_view(
    previous: &UpdateView,
    phase: UpdatePhaseView,
    completed_steps: u8,
    label: &'static str,
) -> UpdateView {
    UpdateView {
        status: StatusCode::Busy,
        selected_stream: previous.selected_stream,
        phase,
        available_version: previous.available_version.clone(),
        archive_size_bytes: previous.archive_size_bytes,
        progress: Some(UpdateProgressView {
            completed_steps,
            total_steps: 4,
            label,
            indeterminate: matches!(
                phase,
                UpdatePhaseView::Checking
                    | UpdatePhaseView::Downloading
                    | UpdatePhaseView::Installing
                    | UpdatePhaseView::AwaitingRestart
                    | UpdatePhaseView::Cancelling
            ),
        }),
        reason_code: match phase {
            UpdatePhaseView::Checking => "update-checking",
            UpdatePhaseView::Downloading => "update-downloading",
            UpdatePhaseView::Verifying => "update-verifying",
            UpdatePhaseView::Staging => "update-staging",
            UpdatePhaseView::Installing => "update-installing",
            UpdatePhaseView::AwaitingRestart => "update-restarting",
            UpdatePhaseView::Cancelling => "update-cancelling",
            _ => "update-busy",
        },
        remediation: UpdateRemediation::None,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        // The in-process preparation worker does not yet expose cooperative
        // cancellation. Advertising Cancel here would start a second worker
        // while download/verification/staging still owns the transaction.
        // Interrupted or fully staged transactions remain cancellable through
        // `update_snapshot`, after no preparation worker is active.
        can_cancel: false,
    }
}

fn update_preparation_progress_view(
    stream: UpdateStreamView,
    available_version: Option<String>,
    archive_size_bytes: Option<u64>,
    stage: crate::update_cli::DesktopUpdatePreparationStage,
) -> UpdateView {
    let base = UpdateView {
        status: StatusCode::Busy,
        selected_stream: stream,
        phase: UpdatePhaseView::Downloading,
        available_version,
        archive_size_bytes,
        progress: None,
        reason_code: "update-downloading",
        remediation: UpdateRemediation::None,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    };
    match stage {
        crate::update_cli::DesktopUpdatePreparationStage::Downloading => update_busy_view(
            &base,
            UpdatePhaseView::Downloading,
            1,
            "Downloading signed package",
        ),
        crate::update_cli::DesktopUpdatePreparationStage::Verifying => update_busy_view(
            &base,
            UpdatePhaseView::Verifying,
            2,
            "Verifying signatures and evidence",
        ),
        crate::update_cli::DesktopUpdatePreparationStage::Staging => update_busy_view(
            &base,
            UpdatePhaseView::Staging,
            3,
            "Preparing the native application",
        ),
    }
}

fn update_checked_view(checked: crate::update_cli::DesktopUpdateCheck) -> UpdateView {
    let stream = update_stream_view(checked.selected_stream);
    match checked.disposition {
        crate::update_cli::DesktopUpdateCheckDisposition::Current => UpdateView {
            status: StatusCode::Ready,
            selected_stream: stream,
            phase: UpdatePhaseView::Current,
            available_version: Some(checked.target_version),
            archive_size_bytes: None,
            progress: None,
            reason_code: "update-current",
            remediation: UpdateRemediation::None,
            can_select_stream: true,
            can_check: true,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        },
        crate::update_cli::DesktopUpdateCheckDisposition::Available => UpdateView {
            status: StatusCode::Attention,
            selected_stream: stream,
            phase: UpdatePhaseView::Available,
            available_version: Some(checked.target_version),
            archive_size_bytes: Some(checked.archive_size_bytes),
            progress: None,
            reason_code: "update-available",
            remediation: UpdateRemediation::None,
            can_select_stream: true,
            can_check: true,
            can_prepare: true,
            can_install: false,
            can_cancel: false,
        },
    }
}

fn update_prepared_view(
    environment: &CommandEnvironment,
    prepared: crate::update_cli::DesktopPreparedUpdate,
) -> UpdateView {
    let mut view = update_snapshot(environment);
    view.available_version = Some(prepared.target_version);
    view
}

fn update_install_handoff_view(handoff: crate::update_cli::DesktopInstallHandoff) -> UpdateView {
    UpdateView {
        status: StatusCode::Busy,
        selected_stream: update_stream_view(handoff.selected_stream),
        phase: UpdatePhaseView::AwaitingRestart,
        available_version: Some(handoff.target_version),
        archive_size_bytes: None,
        progress: Some(UpdateProgressView {
            completed_steps: 4,
            total_steps: 4,
            label: "Closing Qiongli for atomic replacement",
            indeterminate: true,
        }),
        reason_code: "update-helper-launched",
        remediation: UpdateRemediation::RestartApplication,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    }
}

fn update_cancelled_view(stream: UpdateStreamView) -> UpdateView {
    UpdateView {
        status: StatusCode::Missing,
        selected_stream: stream,
        phase: UpdatePhaseView::Cancelled,
        available_version: None,
        archive_size_bytes: None,
        progress: None,
        reason_code: "update-cancelled",
        remediation: UpdateRemediation::RetryCheck,
        can_select_stream: true,
        can_check: true,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    }
}

fn update_failure_view(environment: &CommandEnvironment, code: &'static str) -> UpdateView {
    let store = update_store(environment).ok();
    let loaded = store.as_ref().and_then(|store| store.load().ok());
    let stream = loaded.as_ref().map_or_else(
        || update_stream_view(default_update_stream()),
        |loaded| update_stream_view(loaded.state.selected_stream),
    );
    let transaction = loaded
        .as_ref()
        .and_then(|loaded| loaded.state.active_transaction.as_ref());
    let available_version = transaction.map(|transaction| transaction.target_version.clone());
    let can_cancel = transaction.is_some_and(|transaction| {
        matches!(
            transaction.phase,
            UpdateTransactionPhase::Downloading
                | UpdateTransactionPhase::Downloaded
                | UpdateTransactionPhase::Verified
                | UpdateTransactionPhase::Staged
                | UpdateTransactionPhase::ReconciliationPrepared
                | UpdateTransactionPhase::Cancelling
        )
    });
    let can_prepare = transaction.is_some_and(|transaction| {
        matches!(
            transaction.phase,
            UpdateTransactionPhase::Downloaded
                | UpdateTransactionPhase::Verified
                | UpdateTransactionPhase::Staged
                | UpdateTransactionPhase::ReconciliationPrepared
        )
    });
    update_failure_base(stream, available_version, code, can_cancel, can_prepare)
}

fn update_failure_base(
    stream: UpdateStreamView,
    available_version: Option<String>,
    code: &'static str,
    can_cancel: bool,
    can_prepare: bool,
) -> UpdateView {
    let remediation = update_remediation(code, can_cancel, can_prepare);
    UpdateView {
        status: if code.contains("recovery-required") {
            StatusCode::RecoveryRequired
        } else {
            StatusCode::Blocked
        },
        selected_stream: stream,
        phase: if code.contains("recovery-required") {
            UpdatePhaseView::RecoveryRequired
        } else {
            UpdatePhaseView::Failed
        },
        available_version,
        archive_size_bytes: None,
        progress: None,
        reason_code: code,
        remediation,
        can_select_stream: !can_cancel && !can_prepare,
        can_check: !can_cancel && !can_prepare,
        can_prepare,
        can_install: false,
        can_cancel,
    }
}

fn update_remediation(
    code: &'static str,
    can_cancel: bool,
    can_prepare: bool,
) -> UpdateRemediation {
    if code == "native-update-target-unsupported" {
        UpdateRemediation::UseSupportedPlatform
    } else if code.contains("release-authority") || code.contains("macos-team-id") {
        UpdateRemediation::InstallTrustedRelease
    } else if code == "native-update-installation-location-not-writable"
        || code == "native-update-installation-location-unsafe"
    {
        UpdateRemediation::MoveToApplications
    } else if code.contains("recovery-required")
        || code.contains("atomic-replacement")
        || code.contains("health-check")
    {
        UpdateRemediation::ReinstallApplication
    } else if can_prepare {
        UpdateRemediation::RetryPreparation
    } else if can_cancel {
        UpdateRemediation::CancelAndRetry
    } else {
        UpdateRemediation::RetryCheck
    }
}

const fn update_stream_view(stream: UpdateStreamPreference) -> UpdateStreamView {
    match stream {
        UpdateStreamPreference::Stable => UpdateStreamView::Stable,
        UpdateStreamPreference::Beta => UpdateStreamView::Beta,
    }
}

const fn update_stream_preference(stream: UpdateStreamView) -> UpdateStreamPreference {
    match stream {
        UpdateStreamView::Stable => UpdateStreamPreference::Stable,
        UpdateStreamView::Beta => UpdateStreamPreference::Beta,
    }
}

fn update_install_digest(revision: u64, transaction_id: &str, target_version: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-DESKTOP-UPDATE-INSTALL-V1\0");
    hasher.update(revision.to_be_bytes());
    hash_component(&mut hasher, transaction_id.as_bytes());
    hash_component(&mut hasher, target_version.as_bytes());
    lower_hex(&hasher.finalize())
}

fn apply_desired_workflow_variant(
    integrations: &mut [IntegrationView; 2],
    product: &VerifiedPackagedProduct,
    workflow_variant_sha256: Option<&str>,
) {
    for integration in integrations {
        if integration.discovery != IntegrationDiscoveryState::Managed {
            continue;
        }
        if verify_packaged_product_install_with_variant(
            product,
            activation_target(integration.target),
            workflow_variant_sha256,
        )
        .is_ok()
        {
            if workflow_variant_sha256.is_some() {
                integration.evidence_code = "client-managed-customized-current";
            }
            continue;
        }
        if verify_receipt_owned_packaged_product_install(
            product,
            activation_target(integration.target),
        )
        .is_err()
        {
            continue;
        }
        integration.discovery = IntegrationDiscoveryState::Drifted;
        integration.source = StatusCode::Drifted;
        integration.registration = StatusCode::Drifted;
        integration.overall = StatusCode::Drifted;
        integration.next_action = IntegrationActionView::RepairReady;
        integration.evidence_code = "client-managed-content-update-available";
    }
}

impl DesktopService for NativeDesktopService {
    fn snapshot(&mut self) -> DesktopSnapshotV1 {
        let mut snapshot =
            build_snapshot(&self.environment, &self.content, self.secret_store.as_ref());
        snapshot.update = self.update_view.clone();
        snapshot.zotero = self.zotero.clone();
        snapshot.capabilities.apply = !self.activation_sessions.is_empty()
            || !self.candidate_sessions.is_empty()
            || self.packaged_product.product.is_some();
        snapshot.product.trust = if self.packaged_product.product.is_some() {
            ProductTrustView::PackagedProductControl
        } else {
            ProductTrustView::SourceBuild
        };
        if snapshot.cli.can_install && self.packaged_product.product.is_none() {
            snapshot.cli.reason_code = "qiongli-cli-install-authority-required";
        }
        snapshot.cli.can_install &= self.packaged_product.product.is_some();
        if snapshot.cli.can_test
            && let Some((path_state, reason_code)) = self.cli_path_test
        {
            snapshot.cli.path_state = match path_state {
                CliPathState::Active => CliPathStateView::Active,
                CliPathState::Configured => CliPathStateView::Configured,
                CliPathState::NotConfigured => CliPathStateView::NotConfigured,
                CliPathState::Shadowed => CliPathStateView::Shadowed,
                CliPathState::VersionMismatch => CliPathStateView::VersionMismatch,
                CliPathState::NotObservable => CliPathStateView::NotObservable,
            };
            snapshot.cli.path_status = match path_state {
                CliPathState::Active | CliPathState::Configured => StatusCode::Ready,
                CliPathState::NotConfigured
                | CliPathState::Shadowed
                | CliPathState::VersionMismatch => StatusCode::Attention,
                CliPathState::NotObservable => StatusCode::Disabled,
            };
            snapshot.cli.reason_code = reason_code;
        }
        if let (Some(product), Ok(variant)) = (
            self.packaged_product.product.as_ref(),
            self.load_workflow_variant(),
        ) {
            apply_desired_workflow_variant(
                &mut snapshot.integrations,
                product,
                variant.variant_sha256(),
            );
        }
        for (integration, observation) in
            snapshot.integrations.iter_mut().zip(self.host_observations)
        {
            apply_host_observation(integration, observation);
        }
        for integration in &mut snapshot.integrations {
            let authority_available = self
                .activation_sessions
                .iter()
                .any(|session| session.target == integration.target)
                || self
                    .candidate_sessions
                    .iter()
                    .any(|session| session.target == integration.target)
                || self
                    .packaged_product
                    .product
                    .as_ref()
                    .and_then(|product| product.capability(activation_target(integration.target)))
                    .is_some();
            integration.candidate_required = integration.discovery
                == IntegrationDiscoveryState::DiscoveredUnmanaged
                && !authority_available;
        }
        snapshot
    }

    fn execute(&mut self, intent: DesktopIntent) -> DesktopEvent {
        match intent {
            DesktopIntent::Refresh => {
                self.environment.detect_client_versions();
                self.cli_path_test = None;
                DesktopEvent::SnapshotReplaced(Box::new(self.snapshot()))
            }
            DesktopIntent::RunFullMcpSelfTest => self.start_mcp_self_test(),
            DesktopIntent::PollFullMcpSelfTest => self.poll_mcp_self_test(),
            DesktopIntent::CancelFullMcpSelfTest => self.cancel_mcp_self_test(),
            DesktopIntent::SelectUpdateStream { stream } => self.select_update_stream(stream),
            DesktopIntent::CheckForUpdates => self.start_update_check(),
            DesktopIntent::PrepareUpdate => self.start_update_preparation(),
            DesktopIntent::PollUpdate => self.poll_update(),
            DesktopIntent::CancelUpdate => self.cancel_update(),
            DesktopIntent::PreviewUpdateInstall => self.preview_update_install(),
            DesktopIntent::PreviewCliInstall => self.preview_cli_install(),
            DesktopIntent::PreviewCliRemove => self.preview_cli_remove(),
            DesktopIntent::PreviewCliPathConfigure => self.preview_cli_path_configure(),
            DesktopIntent::TestCliCommand => {
                self.cli_path_test = Some(test_cli_shell_command(&self.environment));
                DesktopEvent::SnapshotReplaced(Box::new(self.snapshot()))
            }
            DesktopIntent::RefreshIntegrationDiscovery => {
                self.refresh_host_integration_observations(IntegrationSelection::ALL);
                DesktopEvent::SnapshotReplaced(Box::new(self.snapshot()))
            }
            DesktopIntent::RefreshZoteroIntegration => self.refresh_zotero_integration(),
            DesktopIntent::PreviewZoteroCompanionStage => self.preview_zotero_companion_stage(),
            DesktopIntent::VerifyZoteroIntegration => self.verify_zotero_integration(),
            DesktopIntent::RevealZoteroCompanion | DesktopIntent::OpenZotero => {
                DesktopEvent::Failed {
                    code: "desktop-shell-action-required",
                }
            }
            DesktopIntent::PrepareLegacyMigration {
                provider_resolutions,
            } => self.prepare_legacy_migration(provider_resolutions),
            DesktopIntent::PreviewLegacyMigrationNext => self.preview_legacy_migration_next(),
            DesktopIntent::PreviewGlobalSettingsPatch(patch) => self.preview_global_settings(patch),
            DesktopIntent::PreviewProviderSettingsPatch(patch) => {
                self.preview_provider_settings(patch)
            }
            DesktopIntent::PreviewProviderSecretChange { provider, change } => {
                self.preview_provider_secret(provider, change)
            }
            DesktopIntent::PreviewAgentBackendSettingsPatch(patch) => {
                if self.direct_backend_experiment_enabled {
                    self.preview_agent_backend_settings(patch)
                } else {
                    DesktopEvent::Failed {
                        code: "host-driven-execution-required",
                    }
                }
            }
            DesktopIntent::PreviewAgentBackendSecretChange { change } => {
                if matches!(&change, AgentBackendSecretChange::Remove)
                    || self.direct_backend_experiment_enabled
                {
                    self.preview_agent_backend_secret(change)
                } else {
                    DesktopEvent::Failed {
                        code: "host-driven-execution-required",
                    }
                }
            }
            DesktopIntent::PreviewAgentRun(draft) => {
                if self.direct_backend_experiment_enabled {
                    self.preview_agent_run(draft)
                } else {
                    DesktopEvent::Failed {
                        code: "host-driven-execution-required",
                    }
                }
            }
            DesktopIntent::TestOpenAiBackend => {
                if self.direct_backend_experiment_enabled {
                    self.test_openai_backend()
                } else {
                    DesktopEvent::Failed {
                        code: "host-driven-execution-required",
                    }
                }
            }
            DesktopIntent::TestLiteratureProvider { provider } => {
                self.test_literature_provider(provider)
            }
            DesktopIntent::SelectSkillsDestination => self.select_skills_destination(),
            DesktopIntent::PreviewSkillsMaterialization { profile } => {
                self.preview_skills_materialization(profile)
            }
            DesktopIntent::VerifySkillsMaterialization => self.verify_skills_materialization(),
            DesktopIntent::PreviewSkillsRemoval => self.preview_skills_removal(),
            DesktopIntent::PreviewSkillsPresetMaterialization { profile, preset } => {
                self.preview_skills_preset_materialization(profile, preset)
            }
            DesktopIntent::VerifySkillsPreset { preset } => self.verify_skills_preset(preset),
            DesktopIntent::PreviewSkillsPresetRemoval { preset } => {
                self.preview_skills_preset_removal(preset)
            }
            DesktopIntent::VerifyManagedSkillsTarget { target_id } => {
                self.verify_managed_skills_target(&target_id)
            }
            DesktopIntent::PreviewManagedSkillsTargetUpdate { target_id } => {
                self.preview_managed_skills_target_update(&target_id)
            }
            DesktopIntent::PreviewManagedSkillsTargetRemoval { target_id } => {
                self.preview_managed_skills_target_removal(&target_id)
            }
            DesktopIntent::PreviewManagedSkillsTargetDetach { target_id } => {
                self.preview_managed_skills_target_detach(&target_id)
            }
            DesktopIntent::PreviewProviderPublicSetting {
                provider,
                public_email,
            } => {
                if !matches!(provider, ProviderKind::Crossref | ProviderKind::OpenAlex)
                    || EmailAddress::parse(public_email.expose()).is_err()
                {
                    return DesktopEvent::ValidationFailed {
                        code: "provider-public-setting-invalid",
                    };
                }
                self.issue_preview(
                    "Provider setting preview",
                    "The public contact email is valid. No value was stored.",
                    "config-write-unavailable",
                )
            }
            DesktopIntent::PreviewIntegration { target } => self.preview_activation(target),
            DesktopIntent::PreviewInstallRecommended => {
                let selection = self.recommended_integration_selection();
                if selection.is_empty() {
                    DesktopEvent::Completed {
                        code: "packaged-product-install-not-required",
                    }
                } else {
                    self.preview_integration_installation(selection)
                }
            }
            DesktopIntent::PreviewInstallSelected { selection } => {
                self.preview_integration_installation(selection)
            }
            DesktopIntent::VerifyIntegrations { selection } => {
                self.verify_packaged_integrations(selection)
            }
            DesktopIntent::PreviewReconcileIntegrations { selection } => {
                self.preview_integration_reconciliation(selection)
            }
            DesktopIntent::PreviewRemoveIntegrations { selection } => {
                self.preview_packaged_product_removal(selection)
            }
            DesktopIntent::ConfirmOperation { token } => {
                let Some(operation) = self.active_operation.as_ref() else {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                };
                if operation.token() != token {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                let operation = self
                    .active_operation
                    .take()
                    .expect("validated active operation remains available");
                match operation {
                    PendingDesktopOperation::Blocked(_) => DesktopEvent::Failed {
                        code: "desktop-apply-unavailable",
                    },
                    PendingDesktopOperation::GlobalSettings {
                        expected_revision,
                        replacement,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: if outcome.cleanup_required {
                                    "global-settings-updated-cleanup-required"
                                } else {
                                    "global-settings-updated"
                                },
                            },
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::ProviderSettings {
                        expected_revision,
                        replacement,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: if outcome.cleanup_required {
                                    "provider-settings-updated-cleanup-required"
                                } else {
                                    "provider-settings-updated"
                                },
                            },
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::AgentBackendSettings {
                        expected_revision,
                        replacement,
                        ..
                    } => {
                        if !self.direct_backend_experiment_enabled {
                            return DesktopEvent::Failed {
                                code: "host-driven-execution-required",
                            };
                        }
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: if outcome.cleanup_required {
                                    "agent-backend-settings-updated-cleanup-required"
                                } else {
                                    "agent-backend-settings-updated"
                                },
                            },
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::ProviderSecret {
                        expected_revision,
                        provider,
                        replacement,
                        secret_ref,
                        replacement_value,
                        previous_value,
                        ..
                    } => {
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let credential_write = if let Some(value) = replacement_value.as_ref() {
                            self.secret_store.store(&secret_ref, value)
                        } else if previous_value.is_some() {
                            self.secret_store.remove(&secret_ref)
                        } else {
                            Ok(())
                        };
                        if let Err(error) = credential_write {
                            return DesktopEvent::Failed {
                                code: error.remediation_code(),
                            };
                        }
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: match (
                                    provider,
                                    replacement_value.is_some(),
                                    outcome.cleanup_required,
                                ) {
                                    (_, _, true) => "provider-secret-updated-cleanup-required",
                                    (ProviderKind::OpenAlex, true, false) => {
                                        "openalex-api-key-saved"
                                    }
                                    (ProviderKind::SemanticScholar, true, false) => {
                                        "semantic-scholar-api-key-saved"
                                    }
                                    (ProviderKind::OpenAlex, false, false) => {
                                        "openalex-api-key-removed"
                                    }
                                    (ProviderKind::SemanticScholar, false, false) => {
                                        "semantic-scholar-api-key-removed"
                                    }
                                    (ProviderKind::PubMed, true, false) => "pubmed-api-key-saved",
                                    (ProviderKind::PubMed, false, false) => {
                                        "pubmed-api-key-removed"
                                    }
                                    (ProviderKind::Crossref | ProviderKind::Arxiv, _, false) => {
                                        "provider-secret-updated"
                                    }
                                },
                            },
                            Err(error) => {
                                let compensated = if let Some(previous) = previous_value.as_ref() {
                                    self.secret_store.store(&secret_ref, previous).is_ok()
                                } else if replacement_value.is_some() {
                                    self.secret_store.remove(&secret_ref).is_ok()
                                } else {
                                    true
                                };
                                DesktopEvent::Failed {
                                    code: if compensated {
                                        error.reason_code()
                                    } else {
                                        "provider-secret-recovery-required"
                                    },
                                }
                            }
                        }
                    }
                    PendingDesktopOperation::AgentBackendSecret {
                        expected_revision,
                        replacement,
                        secret_ref,
                        replacement_value,
                        previous_value,
                        ..
                    } => {
                        if replacement_value.is_some() && !self.direct_backend_experiment_enabled {
                            return DesktopEvent::Failed {
                                code: "host-driven-execution-required",
                            };
                        }
                        let store = match config_store(&self.environment) {
                            Ok(store) => store,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let credential_write = if let Some(value) = replacement_value.as_ref() {
                            self.secret_store.store(&secret_ref, value)
                        } else if previous_value.is_some() {
                            self.secret_store.remove(&secret_ref)
                        } else {
                            Ok(())
                        };
                        if let Err(error) = credential_write {
                            return DesktopEvent::Failed {
                                code: error.remediation_code(),
                            };
                        }
                        match store.replace(expected_revision, replacement) {
                            Ok(outcome) => DesktopEvent::Completed {
                                code: match (replacement_value.is_some(), outcome.cleanup_required)
                                {
                                    (_, true) => "agent-backend-secret-updated-cleanup-required",
                                    (true, false) => "openai-api-key-saved",
                                    (false, false) => "openai-api-key-removed",
                                },
                            },
                            Err(error) => {
                                let compensated = if let Some(previous) = previous_value.as_ref() {
                                    self.secret_store.store(&secret_ref, previous).is_ok()
                                } else if replacement_value.is_some() {
                                    self.secret_store.remove(&secret_ref).is_ok()
                                } else {
                                    true
                                };
                                DesktopEvent::Failed {
                                    code: if compensated {
                                        error.reason_code()
                                    } else {
                                        "agent-backend-secret-recovery-required"
                                    },
                                }
                            }
                        }
                    }
                    PendingDesktopOperation::AgentRun { request, .. } => {
                        if !self.direct_backend_experiment_enabled {
                            return DesktopEvent::Failed {
                                code: "host-driven-execution-required",
                            };
                        }
                        let Some(projects) = project_state_service(&self.environment) else {
                            return DesktopEvent::Failed {
                                code: "project-service-unavailable",
                            };
                        };
                        let registry =
                            match FullProjectToolRegistry::from_embedded_content(&self.content) {
                                Ok(registry) => registry,
                                Err(_) => {
                                    return DesktopEvent::Failed {
                                        code: "agent-run-tools-unavailable",
                                    };
                                }
                            };
                        let loaded =
                            match config_store(&self.environment).and_then(|store| store.load()) {
                                Ok(loaded) => loaded,
                                Err(error) => {
                                    return DesktopEvent::Failed {
                                        code: error.reason_code(),
                                    };
                                }
                            };
                        let service = FullAgentRunService::new(projects, registry);
                        match service.run_openai(
                            request,
                            &loaded.settings,
                            Arc::clone(&self.secret_store),
                        ) {
                            Ok(result) => {
                                DesktopEvent::AgentRunCompleted(agent_run_result_view(result))
                            }
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::SkillsMaterialization {
                        profile,
                        target,
                        expected_variant_sha256,
                        ..
                    } => {
                        let root = match config_root(&self.environment) {
                            Ok(root) => root,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let variant = match WorkflowVariantStore::new(root.clone())
                            .load(self.content.pack())
                        {
                            Ok(variant)
                                if variant.variant_sha256()
                                    == expected_variant_sha256.as_deref() =>
                            {
                                variant
                            }
                            Ok(_) => {
                                return DesktopEvent::Failed {
                                    code: "workflow-variant-digest-conflict",
                                };
                            }
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match crate::managed_content::apply_managed_materialization_with_overrides(
                            &root,
                            &self.content,
                            &target,
                            profile_to_content(profile),
                            variant.overrides(),
                        ) {
                            Ok(_) => DesktopEvent::Completed {
                                code: "skills-materialization-completed",
                            },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::SkillsRemoval {
                        target,
                        expected_receipt,
                        ..
                    } => {
                        let root = match config_root(&self.environment) {
                            Ok(root) => root,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        let variant = match WorkflowVariantStore::new(root.clone())
                            .load(self.content.pack())
                        {
                            Ok(variant) => variant,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match crate::managed_content::remove_managed_materialization_with_overrides(
                            root.state_root(),
                            &self.content,
                            &target,
                            &expected_receipt,
                            variant.overrides(),
                        ) {
                            Ok(_) => DesktopEvent::Completed {
                                code: "skills-materialization-removed",
                            },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::SkillsDetach {
                        target_id,
                        expected_profile,
                        expected_receipt_sha256,
                        ..
                    } => {
                        let (target, profile, state, receipt_sha256) =
                            match self.resolve_managed_skills_target(&target_id) {
                                Ok(resolved) => resolved,
                                Err(code) => return DesktopEvent::Failed { code },
                            };
                        if profile != expected_profile
                            || state != ManagedSkillsStateView::Drifted
                            || receipt_sha256 != expected_receipt_sha256
                        {
                            return DesktopEvent::Failed {
                                code: "managed-operation-precondition-changed",
                            };
                        }
                        let root = match config_root(&self.environment) {
                            Ok(root) => root,
                            Err(error) => {
                                return DesktopEvent::Failed {
                                    code: error.reason_code(),
                                };
                            }
                        };
                        match crate::managed_content::detach_managed_materialization(
                            root.state_root(),
                            &target,
                            &expected_receipt_sha256,
                        ) {
                            Ok(()) => DesktopEvent::Completed {
                                code: "managed-skills-target-detached-preserved",
                            },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::WorkflowVariantChange {
                        expected_revision,
                        expected_variant_sha256,
                        expected_current_sha256,
                        path,
                        replacement,
                        ..
                    } => {
                        let store = match self.workflow_variant_store() {
                            Ok(store) => store,
                            Err(code) => return DesktopEvent::Failed { code },
                        };
                        let reset = replacement.is_none();
                        let result = match replacement {
                            Some(content) => store.replace_resource(
                                self.content.pack(),
                                expected_revision,
                                expected_variant_sha256.as_deref(),
                                &expected_current_sha256,
                                &path,
                                content,
                            ),
                            None => store.reset_resource(
                                self.content.pack(),
                                expected_revision,
                                expected_variant_sha256.as_deref(),
                                &expected_current_sha256,
                                &path,
                            ),
                        };
                        match result {
                            Ok(commit) => DesktopEvent::Completed {
                                code: match (reset, commit.cleanup_required) {
                                    (true, true) => "workflow-variant-reset-cleanup-required",
                                    (true, false) => "workflow-variant-reset",
                                    (false, true) => "workflow-variant-updated-cleanup-required",
                                    (false, false) => "workflow-variant-updated",
                                },
                            },
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::CliInstall { plan, .. } => {
                        match apply_cli_install(&plan) {
                            Ok(code) => {
                                self.cli_path_test =
                                    Some(test_cli_shell_command(&self.environment));
                                DesktopEvent::Completed { code }
                            }
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::CliRemove { plan, .. } => {
                        match apply_cli_remove(&plan) {
                            Ok(CliRemovalEffect::Removed) => DesktopEvent::Completed {
                                code: "qiongli-cli-removed",
                            },
                            Ok(CliRemovalEffect::RestoredPredecessor) => DesktopEvent::Completed {
                                code: "qiongli-cli-predecessor-restored",
                            },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::CliPathConfigure { plan, .. } => {
                        match apply_cli_path_configure(&plan) {
                            Ok(code) => {
                                self.cli_path_test =
                                    Some(test_cli_shell_command(&self.environment));
                                DesktopEvent::Completed { code }
                            }
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::ZoteroCompanionStage { plan, .. } => {
                        let expected = plan.plan_digest_sha256().to_owned();
                        match apply_zotero_companion_stage(&plan, &expected, true) {
                            Ok(_) => {
                                self.zotero = zotero_service_snapshot(&self.environment);
                                DesktopEvent::Completed {
                                    code: "zotero-companion-installation-prepared",
                                }
                            }
                            Err(error) => DesktopEvent::Failed {
                                code: error.reason_code(),
                            },
                        }
                    }
                    PendingDesktopOperation::Activation { target, .. } => {
                        let Some(session) = self
                            .activation_sessions
                            .iter_mut()
                            .find(|session| session.target == target)
                        else {
                            return DesktopEvent::Failed {
                                code: "desktop-activation-session-missing",
                            };
                        };
                        match now_unix().and_then(|now_unix| session.confirm(now_unix)) {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::Candidate { target, .. } => {
                        let Some(home) = self.environment.platform_home() else {
                            return DesktopEvent::Failed {
                                code: "native-candidate-home-unavailable",
                            };
                        };
                        let Some(session) = self
                            .candidate_sessions
                            .iter_mut()
                            .find(|session| session.target == target)
                        else {
                            return DesktopEvent::Failed {
                                code: "desktop-candidate-session-missing",
                            };
                        };
                        match now_unix()
                            .and_then(|now_unix| session.confirm(&self.content, home, now_unix))
                        {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::PackagedProduct { target, .. } => {
                        let selection = integration_selection(target);
                        self.refresh_host_integration_observations(selection);
                        let observation = self.host_observations
                            [host_plugin_target_index(activation_target(target))];
                        let variant = match self.load_workflow_variant() {
                            Ok(variant) => variant,
                            Err(code) => return DesktopEvent::Failed { code },
                        };
                        match now_unix().and_then(|now_unix| {
                            self.packaged_product.confirm(
                                &self.content,
                                &self.environment,
                                observation,
                                target,
                                now_unix,
                                variant.overrides(),
                            )
                        }) {
                            Ok((_code, plans)) => match execute_host_plugin_plans(
                                &self.environment,
                                &plans,
                                &mut self.host_observations,
                            ) {
                                Ok(()) => DesktopEvent::Completed {
                                    code: "qiongli-plugin-ready-restart-host",
                                },
                                Err(code) => DesktopEvent::Failed { code },
                            },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::PackagedProductBatch { selection, .. } => {
                        self.refresh_host_integration_observations(selection);
                        let variant = match self.load_workflow_variant() {
                            Ok(variant) => variant,
                            Err(code) => return DesktopEvent::Failed { code },
                        };
                        match now_unix().and_then(|now_unix| {
                            self.packaged_product.confirm_batch(
                                &self.content,
                                &self.environment,
                                &self.host_observations,
                                selection,
                                now_unix,
                                variant.overrides(),
                            )
                        }) {
                            Ok((_code, plans)) => match execute_host_plugin_plans(
                                &self.environment,
                                &plans,
                                &mut self.host_observations,
                            ) {
                                Ok(()) => DesktopEvent::Completed {
                                    code: "qiongli-plugin-ready-restart-host",
                                },
                                Err(code) => DesktopEvent::Failed { code },
                            },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::IntegrationReconciliation {
                        selection,
                        transaction_id,
                        journal_sha256,
                        expected_variant_sha256,
                        hosts,
                        ..
                    } => self.confirm_integration_content_reconciliation(
                        selection,
                        &transaction_id,
                        &journal_sha256,
                        expected_variant_sha256.as_deref(),
                        &hosts,
                    ),
                    PendingDesktopOperation::PackagedProductRemoval { selection, .. } => {
                        match now_unix().and_then(|now_unix| {
                            self.packaged_product.confirm_remove(selection, now_unix)
                        }) {
                            Ok(code) => DesktopEvent::Completed { code },
                            Err(code) => DesktopEvent::Failed { code },
                        }
                    }
                    PendingDesktopOperation::UpdateInstall {
                        expected_revision, ..
                    } => self.start_update_install(expected_revision),
                    PendingDesktopOperation::LegacyMigration {
                        command,
                        completion_code,
                        ..
                    } => match crate::legacy_migration_cli::execute_with_secret_store(
                        command,
                        &self.environment,
                        &self.content,
                        self.secret_store.as_ref(),
                    ) {
                        Ok(_) => DesktopEvent::Completed {
                            code: completion_code,
                        },
                        Err(code) => DesktopEvent::Failed { code },
                    },
                }
            }
            DesktopIntent::CancelOperation { token } => {
                if self
                    .active_operation
                    .as_ref()
                    .map(PendingDesktopOperation::token)
                    != Some(token)
                {
                    return DesktopEvent::Failed {
                        code: "operation-token-invalid",
                    };
                }
                self.cancel_active_operation();
                DesktopEvent::Cancelled {
                    code: "operation-preview-cancelled",
                }
            }
        }
    }
}

fn managed_skills_snapshot(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> (StatusCode, Vec<ManagedSkillsView>) {
    let root = match config_root(environment) {
        Ok(root) => root,
        Err(_) => return (StatusCode::Unavailable, Vec::new()),
    };
    let registry = match crate::managed_content::load_managed_content_registry(root.state_root()) {
        Ok(registry) => registry,
        Err(_) => return (StatusCode::Unavailable, Vec::new()),
    };
    let variant = match WorkflowVariantStore::new(root).load(content.pack()) {
        Ok(variant) => variant,
        Err(_) => return (StatusCode::Unavailable, Vec::new()),
    };
    let mut presets = BTreeMap::new();
    if let Some(home) = environment.platform_home() {
        presets.insert(
            home.join(".qiongli-skills").to_string_lossy().into_owned(),
            SkillsDestinationPreset::QiongliManaged,
        );
    }
    if let Some(project) = environment.project_root() {
        presets
            .entry(
                project
                    .join(".qiongli-skills")
                    .to_string_lossy()
                    .into_owned(),
            )
            .or_insert(SkillsDestinationPreset::CurrentProject);
    }

    let mut managed = registry
        .entries
        .iter()
        .map(|entry| {
            let preset = presets
                .remove(&entry.target)
                .unwrap_or(SkillsDestinationPreset::CustomFolder);
            managed_skills_entry_view(entry, preset, content, variant.variant_sha256())
        })
        .collect::<Vec<_>>();
    managed.extend(presets.into_iter().map(|(target, preset)| {
        let (state, status) = unregistered_managed_skills_state(Path::new(&target));
        ManagedSkillsView {
            target_id: managed_skills_target_id(&target),
            preset,
            state,
            status,
            profile: None,
            product_version: None,
        }
    }));
    managed.sort_by(|left, right| left.target_id.cmp(&right.target_id));
    let status = if managed
        .iter()
        .any(|entry| entry.state == ManagedSkillsStateView::Drifted)
    {
        StatusCode::Drifted
    } else if managed
        .iter()
        .any(|entry| entry.state == ManagedSkillsStateView::Unmanaged)
    {
        StatusCode::Conflict
    } else if managed
        .iter()
        .any(|entry| entry.state == ManagedSkillsStateView::UpdateAvailable)
    {
        StatusCode::Attention
    } else {
        StatusCode::Ready
    };
    (status, managed)
}

fn unregistered_managed_skills_state(path: &Path) -> (ManagedSkillsStateView, StatusCode) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return (ManagedSkillsStateView::Missing, StatusCode::Missing);
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return (ManagedSkillsStateView::Unmanaged, StatusCode::Conflict);
    }
    match fs::read_dir(path) {
        Ok(mut entries) => match entries.next() {
            None => (ManagedSkillsStateView::Missing, StatusCode::Missing),
            Some(_) => (ManagedSkillsStateView::Unmanaged, StatusCode::Conflict),
        },
        Err(_) => (ManagedSkillsStateView::Unmanaged, StatusCode::Conflict),
    }
}

fn managed_skills_entry_view(
    entry: &crate::managed_content::ManagedContentEntryV1,
    preset: SkillsDestinationPreset,
    content: &EmbeddedContent,
    expected_variant_sha256: Option<&str>,
) -> ManagedSkillsView {
    let (state, status) = match crate::managed_content::observe_managed_skills_entry_with_variant(
        content,
        entry,
        expected_variant_sha256,
    ) {
        Ok(observation) => match observation.state {
            crate::managed_content::ManagedSkillsEntryState::Current => {
                (ManagedSkillsStateView::Current, StatusCode::Ready)
            }
            crate::managed_content::ManagedSkillsEntryState::UpdateAvailable => (
                ManagedSkillsStateView::UpdateAvailable,
                StatusCode::Attention,
            ),
        },
        Err(_) => (ManagedSkillsStateView::Drifted, StatusCode::Drifted),
    };
    ManagedSkillsView {
        target_id: managed_skills_target_id(&entry.target),
        preset,
        state,
        status,
        profile: Some(profile_from_content(entry.profile)),
        product_version: Some(entry.product_version.clone()),
    }
}

fn build_snapshot(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    secret_store: &dyn SecretStore,
) -> DesktopSnapshotV1 {
    let manifest = content.pack().manifest();
    let profiles = ProfileKind::ALL.map(|profile| ProfileView {
        profile,
        included_resource_kinds: content
            .profiles()
            .iter()
            .find(|candidate| profile_from_content(candidate.id) == profile)
            .map_or(0, |candidate| candidate.included_resource_kinds.len()),
    });
    let (config, _config_diagnostic) = config_snapshot(environment, secret_store);
    let ([(codex, _codex_diagnostic), (claude, _claude_diagnostic)], legacy_migration) =
        integration_snapshots(environment);
    let inspection =
        crate::product_diagnostics::inspect_product(environment, content, secret_store.status());
    let diagnostics = inspection.checks.map(diagnostic_check_view);
    let diagnostic_paths = inspection
        .paths
        .into_iter()
        .map(diagnostic_path_view)
        .collect();
    let (managed_skills_status, managed_skills) = managed_skills_snapshot(environment, content);
    let config_edit = matches!(config.status, StatusCode::Missing | StatusCode::Ready)
        && config.revision.is_some();
    DesktopSnapshotV1 {
        schema_version: DESKTOP_SNAPSHOT_SCHEMA_VERSION,
        product: ProductView {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            build: crate::embedded_source_commit()
                .unwrap_or("source-build")
                .to_owned(),
            operating_system: operating_system_view(OperatingSystem::current()),
            architecture: architecture_view(Architecture::current()),
            trust: ProductTrustView::SourceBuild,
        },
        content: ContentView {
            status: StatusCode::Ready,
            pack_id: manifest.pack_id.clone(),
            content_version: manifest.content_version.clone(),
            entry_count: manifest.entries.len(),
            profiles,
            managed_skills_status,
            managed_skills,
        },
        mcp: McpView {
            status: StatusCode::Ready,
            profile: ProfileKind::MarketplaceLite,
            public_tool_count: LITE_PUBLIC_TOOL_NAMES.len(),
        },
        cli: cli_snapshot(environment),
        zotero: zotero_integration_snapshot(),
        config,
        update: update_snapshot(environment),
        legacy_migration,
        integrations: [codex, claude],
        diagnostics,
        diagnostic_paths,
        capabilities: CapabilityView {
            refresh: true,
            config_edit,
            skills_materialize: OperatingSystem::current().is_some(),
            provider_preview: true,
            mcp_self_test: true,
            integration_discovery: true,
            integration_preview: true,
            apply: false,
        },
    }
}

fn zotero_integration_snapshot() -> ZoteroIntegrationView {
    let artifact = crate::embedded_zotero_companion().ok();
    let available_companion_version = artifact
        .as_ref()
        .map(|artifact| artifact.manifest().companion_version.clone());
    let available_companion_sha256 = artifact
        .as_ref()
        .map(|artifact| artifact.manifest().artifact_sha256.clone());
    let available_companion_size_bytes = artifact
        .as_ref()
        .map(|artifact| artifact.xpi_bytes().len() as u64);
    let can_prepare_install = available_companion_version.is_some();
    ZoteroIntegrationView {
        status: StatusCode::Disabled,
        state: ZoteroIntegrationStateView::NotObserved,
        observation: ZoteroObservationView::NotObserved,
        zotero_version: None,
        connector_available: false,
        companion_available: false,
        companion_version: None,
        available_companion_version,
        available_companion_sha256,
        available_companion_size_bytes,
        endpoint_version: None,
        supported_endpoint_version:
            qiongli_runtime::zotero::companion::SUPPORTED_COMPANION_ENDPOINT_VERSION,
        supported_zotero_min_version: ZOTERO_COMPANION_ZOTERO_MIN_VERSION,
        supported_zotero_max_version: ZOTERO_COMPANION_ZOTERO_MAX_VERSION,
        installation_prepared: false,
        fallback_import_available: true,
        fallback_formats: ZOTERO_FALLBACK_FORMATS,
        reason_code: "zotero-integration-not-observed",
        can_prepare_install,
        can_reveal: false,
        can_open_zotero: false,
        can_verify: true,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DetectedZoteroApplication {
    path: PathBuf,
    version: Option<String>,
}

fn zotero_service_snapshot(environment: &CommandEnvironment) -> ZoteroIntegrationView {
    let mut view = zotero_integration_snapshot();
    let application = detect_zotero_application(environment);
    view.can_open_zotero = application.is_some();
    let Ok(root) = config_root(environment) else {
        return view;
    };
    let Ok(artifact) = crate::embedded_zotero_companion() else {
        return view;
    };
    match verify_zotero_companion_stage(root.state_root(), &artifact) {
        Ok(Some(_)) => {
            view.status = StatusCode::Attention;
            view.state = ZoteroIntegrationStateView::RestartRequired;
            view.zotero_version = application.and_then(|application| application.version);
            view.installation_prepared = true;
            view.can_reveal = true;
            view.reason_code = "zotero-companion-installation-prepared";
        }
        Ok(None) => {}
        Err(_) => {
            view.status = StatusCode::Attention;
            view.state = ZoteroIntegrationStateView::NotObservable;
            view.observation = ZoteroObservationView::NotObservable;
            view.reason_code = "zotero-companion-stage-not-observable";
        }
    }
    view
}

fn refreshed_zotero_service_snapshot(environment: &CommandEnvironment) -> ZoteroIntegrationView {
    let mut view = zotero_service_snapshot(environment);
    if view.state == ZoteroIntegrationStateView::NotObservable {
        return view;
    }
    let application = detect_zotero_application(environment);
    view.can_open_zotero = application.is_some();
    view.zotero_version = application
        .as_ref()
        .and_then(|application| application.version.clone());
    view.observation = ZoteroObservationView::Observed;
    if zotero_version_is_incompatible(view.zotero_version.as_deref()) {
        view.status = StatusCode::Attention;
        view.state = ZoteroIntegrationStateView::ZoteroIncompatible;
        view.can_prepare_install = false;
        view.reason_code = "zotero-version-incompatible";
    } else if view.installation_prepared {
        view.status = StatusCode::Attention;
        view.state = ZoteroIntegrationStateView::RestartRequired;
        view.reason_code = "zotero-companion-restart-required";
    } else if application.is_some() {
        view.status = StatusCode::Attention;
        view.state = ZoteroIntegrationStateView::ZoteroNotRunning;
        view.reason_code = "zotero-not-running";
    } else {
        view.status = StatusCode::Missing;
        view.state = ZoteroIntegrationStateView::ZoteroNotDetected;
        view.reason_code = "zotero-application-not-detected";
    }
    view
}

fn apply_zotero_live_observation(
    view: &mut ZoteroIntegrationView,
    status: &qiongli_runtime::zotero::companion::ZoteroStatus,
) {
    view.observation = ZoteroObservationView::Observed;
    view.connector_available = status.connector.available;
    view.companion_available = status.companion.available;
    view.companion_version = status.companion.version.clone();
    view.endpoint_version = status.companion.endpoint_version.clone();
    if zotero_version_is_incompatible(view.zotero_version.as_deref()) {
        view.status = StatusCode::Attention;
        view.state = ZoteroIntegrationStateView::ZoteroIncompatible;
        view.can_prepare_install = false;
        view.reason_code = "zotero-version-incompatible";
        return;
    }
    let update_available = view
        .companion_version
        .as_deref()
        .and_then(|version| semver::Version::parse(version).ok())
        .zip(
            view.available_companion_version
                .as_deref()
                .and_then(|version| semver::Version::parse(version).ok()),
        )
        .is_some_and(|(installed, available)| installed < available);
    match status.integration_state() {
        ZoteroIntegrationState::Disabled => {
            view.status = StatusCode::Disabled;
            view.state = ZoteroIntegrationStateView::Disabled;
            view.reason_code = "zotero-local-integration-disabled";
        }
        ZoteroIntegrationState::ZoteroNotRunning => {
            if view.installation_prepared {
                view.status = StatusCode::Attention;
                view.state = ZoteroIntegrationStateView::RestartRequired;
                view.reason_code = "zotero-companion-restart-required";
            } else if view.can_open_zotero {
                view.status = StatusCode::Attention;
                view.state = ZoteroIntegrationStateView::ZoteroNotRunning;
                view.reason_code = "zotero-not-running";
            } else {
                view.status = StatusCode::Missing;
                view.state = ZoteroIntegrationStateView::ZoteroNotDetected;
                view.reason_code = "zotero-application-not-detected";
            }
        }
        ZoteroIntegrationState::CompanionMissing => {
            if view.installation_prepared {
                view.status = StatusCode::Attention;
                view.state = ZoteroIntegrationStateView::RestartRequired;
                view.reason_code = "zotero-companion-restart-required";
            } else {
                view.status = StatusCode::Missing;
                view.state = ZoteroIntegrationStateView::CompanionMissing;
                view.reason_code = "zotero-companion-missing";
            }
        }
        ZoteroIntegrationState::CompanionIncompatible => {
            view.status = StatusCode::Attention;
            if update_available && view.installation_prepared {
                view.state = ZoteroIntegrationStateView::RestartRequired;
                view.reason_code = "zotero-companion-restart-required";
            } else if update_available {
                view.state = ZoteroIntegrationStateView::CompanionUpdateAvailable;
                view.reason_code = "zotero-companion-update-required";
            } else {
                view.state = ZoteroIntegrationStateView::CompanionIncompatible;
                view.reason_code = "zotero-companion-endpoint-incompatible";
            }
        }
        ZoteroIntegrationState::Ready => {
            if update_available && view.installation_prepared {
                view.status = StatusCode::Attention;
                view.state = ZoteroIntegrationStateView::RestartRequired;
                view.reason_code = "zotero-companion-restart-required";
            } else if update_available {
                view.status = StatusCode::Attention;
                view.state = ZoteroIntegrationStateView::CompanionUpdateAvailable;
                view.reason_code = "zotero-companion-update-available";
            } else {
                view.status = StatusCode::Ready;
                view.state = ZoteroIntegrationStateView::Ready;
                view.reason_code = "zotero-companion-ready";
            }
        }
    }
}

fn zotero_version_is_incompatible(version: Option<&str>) -> bool {
    let Some(version) = version.and_then(|value| semver::Version::parse(value).ok()) else {
        return false;
    };
    version.major < 8 || version.major > 9 || version.major == 9 && version.minor > 0
}

#[cfg(target_os = "macos")]
fn detect_zotero_application(
    environment: &CommandEnvironment,
) -> Option<DetectedZoteroApplication> {
    let mut candidates = vec![PathBuf::from("/Applications/Zotero.app")];
    if let Some(home) = environment.platform_home() {
        candidates.push(home.join("Applications/Zotero.app"));
    }
    candidates.into_iter().find_map(|path| {
        let metadata = std::fs::symlink_metadata(&path).ok()?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return None;
        }
        let version = read_macos_zotero_version(&path.join("Contents/Info.plist"));
        Some(DetectedZoteroApplication { path, version })
    })
}

#[cfg(target_os = "macos")]
fn read_macos_zotero_version(plist: &Path) -> Option<String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let metadata = std::fs::symlink_metadata(plist).ok()?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 256 * 1024 {
        return None;
    }
    let bytes = std::fs::read(plist).ok()?;
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
                    .map(|version| version.to_string());
            }
            Event::Eof => return None,
            _ => saw_version_key = false,
        }
    }
}

#[cfg(target_os = "linux")]
fn detect_zotero_application(
    environment: &CommandEnvironment,
) -> Option<DetectedZoteroApplication> {
    let mut candidates = vec![
        PathBuf::from("/usr/bin/zotero"),
        PathBuf::from("/opt/zotero/zotero"),
    ];
    if let Some(home) = environment.platform_home() {
        candidates.push(home.join(".local/bin/zotero"));
    }
    candidates.into_iter().find_map(|path| {
        let metadata = std::fs::symlink_metadata(&path).ok()?;
        (!metadata.file_type().is_symlink() && metadata.is_file()).then_some(
            DetectedZoteroApplication {
                path,
                version: None,
            },
        )
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn detect_zotero_application(
    _environment: &CommandEnvironment,
) -> Option<DetectedZoteroApplication> {
    None
}

fn cli_snapshot(environment: &CommandEnvironment) -> CliView {
    let source = bundled_cli_path(environment.platform_home());
    let process_path = std::env::var_os("PATH");
    let process_shell = std::env::var_os("SHELL");
    let inspection = inspect_cli_install(
        environment.platform_home(),
        source.as_deref(),
        env!("CARGO_PKG_VERSION"),
        process_path.as_deref(),
        process_shell.as_deref(),
    );
    let state = match inspection.state {
        CliInstallState::Missing => CliInstallStateView::Missing,
        CliInstallState::InstalledCurrent => CliInstallStateView::InstalledCurrent,
        CliInstallState::UpdateAvailable => CliInstallStateView::UpdateAvailable,
        CliInstallState::Unavailable => CliInstallStateView::Unavailable,
        CliInstallState::Conflict => CliInstallStateView::Conflict,
    };
    let path_state = match inspection.path_state {
        CliPathState::Active => CliPathStateView::Active,
        CliPathState::Configured => CliPathStateView::Configured,
        CliPathState::NotConfigured => CliPathStateView::NotConfigured,
        CliPathState::Shadowed => CliPathStateView::Shadowed,
        CliPathState::VersionMismatch => CliPathStateView::VersionMismatch,
        CliPathState::NotObservable => CliPathStateView::NotObservable,
    };
    CliView {
        status: match inspection.state {
            CliInstallState::InstalledCurrent => StatusCode::Ready,
            CliInstallState::Missing => StatusCode::Missing,
            CliInstallState::UpdateAvailable => StatusCode::Attention,
            CliInstallState::Unavailable => StatusCode::Unavailable,
            CliInstallState::Conflict => StatusCode::Conflict,
        },
        state,
        installed_version: inspection.installed_version,
        available_version: inspection.available_version,
        symbolic_target: if cfg!(windows) {
            "<user-home>/AppData/Local/Qiongli/bin/qiongli.exe"
        } else {
            "<user-home>/.local/bin/qiongli"
        },
        path_status: match inspection.path_state {
            CliPathState::Active | CliPathState::Configured => StatusCode::Ready,
            CliPathState::NotConfigured
            | CliPathState::Shadowed
            | CliPathState::VersionMismatch => StatusCode::Attention,
            CliPathState::NotObservable => StatusCode::Disabled,
        },
        path_state,
        reason_code: inspection.reason_code,
        can_install: inspection.can_install,
        can_test: inspection.can_test,
    }
}

fn diagnostic_check_view(
    check: crate::product_diagnostics::ProductDoctorCheckV1,
) -> DiagnosticCheckView {
    use crate::product_diagnostics::{ProductDoctorCheckId as Check, ProductDoctorStatus as State};

    let id = match check.id {
        Check::EmbeddedContent => DiagnosticCheckId::EmbeddedContent,
        Check::GlobalConfig => DiagnosticCheckId::GlobalConfig,
        Check::SecureStore => DiagnosticCheckId::SecureStore,
        Check::ManagedContent => DiagnosticCheckId::ManagedContent,
        Check::CodexLocal => DiagnosticCheckId::CodexLocal,
        Check::ClaudeCodeLocal => DiagnosticCheckId::ClaudeCodeLocal,
        Check::LiteMcp => DiagnosticCheckId::LiteMcp,
        Check::LiteratureProviders => DiagnosticCheckId::LiteratureProviders,
        Check::UpdateRecovery => DiagnosticCheckId::UpdateRecovery,
        Check::FullRuntime => DiagnosticCheckId::FullRuntime,
    };
    DiagnosticCheckView {
        check: id,
        status: match check.status {
            State::Ready => StatusCode::Ready,
            State::Attention => StatusCode::Attention,
            State::Missing => StatusCode::Missing,
            State::Unavailable => StatusCode::Unavailable,
            State::Invalid => StatusCode::Invalid,
            State::FutureSchema => StatusCode::FutureSchema,
            State::Insecure => StatusCode::Insecure,
            State::Busy => StatusCode::Busy,
            State::WriteUnsupported => StatusCode::WriteUnsupported,
            State::RecoveryRequired => StatusCode::RecoveryRequired,
            State::Deferred => StatusCode::Disabled,
        },
        blocking: check.blocking,
        remediation: diagnostic_remediation(id, check.remediation),
    }
}

fn diagnostic_remediation(check: DiagnosticCheckId, remediation: &'static str) -> RemediationCode {
    match remediation {
        "none" => RemediationCode::None,
        "inspect-global-config" | "create-global-config" => RemediationCode::InspectGlobalConfig,
        "upgrade-qiongli" => RemediationCode::UpgradeQiongli,
        "repair-global-config-permissions" => RemediationCode::RepairGlobalConfigPermissions,
        "retry-global-config" => RemediationCode::RetryGlobalConfig,
        "recover-global-config" => RemediationCode::RecoverGlobalConfig,
        "use-supported-platform" => RemediationCode::UseSupportedPlatform,
        "use-supported-secure-store" => RemediationCode::UseSupportedSecureStore,
        "inspect-managed-content" => RemediationCode::InspectManagedContent,
        "install-supported-client" => RemediationCode::InstallSupportedClient,
        "install-client-integration" => RemediationCode::InstallClientIntegration,
        "resolve-client-conflict" => RemediationCode::ResolveClientConflict,
        "repair-client-integration" => RemediationCode::RepairClientIntegration,
        "configure-literature-providers" => RemediationCode::ConfigureLiteratureProviders,
        "inspect-update-state" => RemediationCode::InspectUpdateState,
        "reinstall-qiongli" => RemediationCode::ReinstallQiongli,
        "upgrade-to-r4-full-runtime" => RemediationCode::UpgradeToFullRuntime,
        "retry-mcp-self-test" => RemediationCode::RetryLiteMcpSelfTest,
        "inspect-client-paths" if check == DiagnosticCheckId::CodexLocal => {
            RemediationCode::InspectCodexLocal
        }
        "inspect-client-paths" if check == DiagnosticCheckId::ClaudeCodeLocal => {
            RemediationCode::InspectClaudeCodeLocal
        }
        _ if check == DiagnosticCheckId::CodexLocal => RemediationCode::InspectCodexLocal,
        _ if check == DiagnosticCheckId::ClaudeCodeLocal => RemediationCode::InspectClaudeCodeLocal,
        _ => RemediationCode::UseSupportedPlatform,
    }
}

fn diagnostic_path_view(
    path: crate::product_diagnostics::ProductPathInspectionV1,
) -> DiagnosticPathView {
    use crate::product_diagnostics::{
        ProductPathFileType as FileType, ProductPathSafety as Safety,
    };

    let status = match (path.safety, path.file_type, path.type_matches_expected) {
        (Safety::Unsafe, _, _) | (_, FileType::Other, _) | (_, _, Some(false)) => {
            StatusCode::Invalid
        }
        (Safety::Unavailable, _, _) | (_, FileType::Unavailable, _) => StatusCode::Unavailable,
        (_, FileType::Missing, _) => StatusCode::Missing,
        _ => StatusCode::Ready,
    };
    let exact = Path::new(&path.exact_path);
    let reveal_path = display_path(if path.file_type == FileType::Directory {
        exact
    } else {
        exact.parent().unwrap_or(exact)
    });
    DiagnosticPathView {
        id: path.id,
        label: path.label,
        symbolic_path: path.symbolic_path,
        exact_path: PrivateDisplayText::new(path.exact_path),
        reveal_path: PrivateDisplayText::new(reveal_path),
        details: format!(
            "Group: {:?} · Scope: {:?} · Source: {:?} · Type: {:?} (expected {}, match {:?}) · Owner: {:?} · Writability: {:?} · Safety: {:?}",
            path.group,
            path.scope,
            path.source,
            path.file_type,
            path.expected_type,
            path.type_matches_expected,
            path.owner,
            path.writability,
            path.safety,
        ),
        selected: path.selected,
        status,
        resolved_target: path.resolved_target.map(PrivateDisplayText::new),
    }
}

const fn integration_target(target: ClientActivationTarget) -> IntegrationTarget {
    match target {
        ClientActivationTarget::Codex => IntegrationTarget::Codex,
        ClientActivationTarget::ClaudeCode => IntegrationTarget::ClaudeCode,
    }
}

const fn activation_target(target: IntegrationTarget) -> ClientActivationTarget {
    match target {
        IntegrationTarget::Codex => ClientActivationTarget::Codex,
        IntegrationTarget::ClaudeCode => ClientActivationTarget::ClaudeCode,
    }
}

const fn integration_selection(target: IntegrationTarget) -> IntegrationSelection {
    match target {
        IntegrationTarget::Codex => IntegrationSelection {
            codex: true,
            claude_code: false,
        },
        IntegrationTarget::ClaudeCode => IntegrationSelection {
            codex: false,
            claude_code: true,
        },
    }
}

fn selected_activation_targets(
    selection: IntegrationSelection,
) -> Result<Vec<ClientActivationTarget>, &'static str> {
    let mut targets = Vec::with_capacity(2);
    if selection.codex {
        targets.push(ClientActivationTarget::Codex);
    }
    if selection.claude_code {
        targets.push(ClientActivationTarget::ClaudeCode);
    }
    if targets.is_empty() {
        return Err("integration-selection-required");
    }
    Ok(targets)
}

fn integration_display_target(target: IntegrationTarget) -> PrivateDisplayText {
    integration_display_targets(&[activation_target(target)])
}

fn integration_display_targets(targets: &[ClientActivationTarget]) -> PrivateDisplayText {
    let targets = targets
        .iter()
        .map(|target| match target {
            ClientActivationTarget::Codex => format!(
                "Codex · {CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH} → {CODEX_MARKETPLACE_SYMBOLIC_PATH}"
            ),
            ClientActivationTarget::ClaudeCode => format!(
                "Claude Code · {CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH} → {CLAUDE_MARKETPLACE_SYMBOLIC_PATH}"
            ),
        })
        .collect::<Vec<_>>()
        .join(" | ");
    PrivateDisplayText::new(targets)
}

fn integration_host_plan_display_targets(plans: &[HostPluginPlan]) -> PrivateDisplayText {
    PrivateDisplayText::new(
        plans
            .iter()
            .map(|plan| {
                let target = integration_target(plan.target);
                let (label, plugin, source, marketplace) = match plan.target {
                    ClientActivationTarget::Codex => (
                        "Codex",
                        CODEX_HOST_PLUGIN_ID,
                        CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH,
                        CODEX_MARKETPLACE_SYMBOLIC_PATH,
                    ),
                    ClientActivationTarget::ClaudeCode => (
                        "Claude Code",
                        CLAUDE_HOST_PLUGIN_ID,
                        CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH,
                        CLAUDE_MARKETPLACE_SYMBOLIC_PATH,
                    ),
                };
                format!(
                    "{label} · {} {} · scope {} · Plugin {plugin} {} · {source} → {marketplace}",
                    host_plugin_executable_name(target),
                    plan.mode.map_or("verify", host_plugin_mode_id),
                    host_plugin_scope(target),
                    env!("CARGO_PKG_VERSION"),
                )
            })
            .collect::<Vec<_>>()
            .join(" | "),
    )
}

fn blocked_batch_product_preview(
    token: OperationToken,
    selection: IntegrationSelection,
    title: &'static str,
    blocked_reason: &'static str,
) -> OperationPreview {
    let display_target = selected_activation_targets(selection)
        .ok()
        .map(|targets| integration_display_targets(&targets));
    OperationPreview {
        token,
        kind: OperationKind::Activation,
        title,
        summary: "The selected clients were inspected. This process has no verified packaged-product installation authority.",
        display_target,
        plan_digest_sha256: None,
        approvals_required: Vec::new(),
        can_confirm: false,
        blocked_reason: Some(blocked_reason),
    }
}

fn blocked_product_preview(
    token: OperationToken,
    target: IntegrationTarget,
    blocked_reason: &'static str,
) -> OperationPreview {
    OperationPreview {
        token,
        kind: OperationKind::Activation,
        title: match target {
            IntegrationTarget::Codex => "Codex installation preview",
            IntegrationTarget::ClaudeCode => "Claude Code installation preview",
        },
        summary: "The local target was inspected. This process has no verified packaged-product installation authority.",
        display_target: Some(integration_display_target(target)),
        plan_digest_sha256: None,
        approvals_required: Vec::new(),
        can_confirm: false,
        blocked_reason: Some(blocked_reason),
    }
}

fn running_packaged_product(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> PackagedProductState {
    match verify_running_packaged_product(environment, content) {
        Ok(product) => PackagedProductState::verified(product),
        Err(reason_code) => PackagedProductState::read_only(reason_code),
    }
}

pub(crate) fn verify_running_packaged_product(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<VerifiedPackagedProduct, &'static str> {
    let authority = match crate::embedded_release_authority() {
        Ok(Some(authority)) => authority,
        Ok(None) => return Err("source-build-read-only"),
        Err(error) => return Err(error.reason_code()),
    };
    let Some(source_commit) = crate::embedded_source_commit() else {
        return Err("source-build-read-only");
    };
    let Some(home) = environment.platform_home() else {
        return Err("packaged-product-home-invalid");
    };
    let process_executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => return Err("packaged-product-executable-invalid"),
    };
    if process_executable.starts_with(home.join(".qiongli/native/payloads")) {
        return qiongli_platform::verify_native_packaged_product(
            content.pack(),
            &authority,
            home,
            &process_executable,
            env!("CARGO_PKG_VERSION"),
            source_commit,
            now_unix()?,
        )
        .map_err(|error| error.reason_code());
    }
    let direct_manifest_path = running_desktop_manifest_path(&process_executable);
    let (current_executable, desktop_manifest_path, expected_control_sha256) =
        if direct_manifest_path.is_file() {
            (process_executable, direct_manifest_path, None)
        } else {
            let binding = match installed_cli_product_authority(home, &process_executable) {
                Ok(binding) => binding,
                Err("qiongli-cli-not-managed-executable") => return Err("source-build-read-only"),
                Err(code) => return Err(code),
            };
            (
                binding.packaged_executable().to_path_buf(),
                binding.desktop_manifest_path().to_path_buf(),
                Some(binding.control_sha256().to_string()),
            )
        };
    let control_path = match packaged_product_control_path(&desktop_manifest_path) {
        Ok(path) => path,
        Err(error) => return Err(error.reason_code()),
    };
    let now_unix = now_unix()?;
    match verify_packaged_product(&PackagedProductVerificationInput {
        current_executable: &current_executable,
        desktop_manifest_path: &desktop_manifest_path,
        control_path: &control_path,
        release_authority: &authority,
        pack: content.pack(),
        product_version: env!("CARGO_PKG_VERSION"),
        product_source_commit: source_commit,
        home,
        now_unix,
    }) {
        Ok(product)
            if expected_control_sha256
                .as_deref()
                .is_none_or(|expected| expected == product.control_sha256()) =>
        {
            Ok(product)
        }
        Ok(_) => Err("qiongli-cli-product-authority-changed"),
        Err(error) => Err(error.reason_code()),
    }
}

fn running_desktop_manifest_path(current_executable: &Path) -> PathBuf {
    let parent = current_executable.parent().unwrap_or(current_executable);
    if cfg!(target_os = "macos") {
        parent
            .join("../Resources")
            .join(qiongli_platform::DESKTOP_PACKAGE_MANIFEST_FILE)
    } else {
        parent.join(qiongli_platform::DESKTOP_PACKAGE_MANIFEST_FILE)
    }
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "system-clock-unavailable")
}

fn config_snapshot(
    environment: &CommandEnvironment,
    secret_store: &dyn SecretStore,
) -> (ConfigView, DiagnosticCheckView) {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return unavailable_config(error),
    };
    let loaded = store.load().ok();
    let status = store.status();
    let view_status = config_status(status.state);
    let providers = status
        .providers
        .as_ref()
        .map_or_else(unavailable_providers, |providers| {
            [
                provider_view(
                    ProviderKind::OpenAlex,
                    &providers.openalex,
                    loaded
                        .as_ref()
                        .is_some_and(|loaded| loaded.settings.providers.openalex.email.is_some()),
                ),
                provider_view(
                    ProviderKind::SemanticScholar,
                    &providers.semantic_scholar,
                    false,
                ),
                provider_view(
                    ProviderKind::Crossref,
                    &providers.crossref,
                    loaded
                        .as_ref()
                        .is_some_and(|loaded| loaded.settings.providers.crossref.email.is_some()),
                ),
                provider_view(ProviderKind::PubMed, &providers.pubmed, false),
                provider_view(ProviderKind::Arxiv, &providers.arxiv, false),
            ]
        });
    let openai_backend = loaded.as_ref().map_or(
        AgentBackendView {
            enabled: false,
            readiness: AgentBackendReadinessView::Disabled,
            secret_reference_present: false,
            test_available: false,
        },
        |loaded| {
            let backend = openai_backend_metadata_status(&loaded.settings, secret_store.status());
            AgentBackendView {
                enabled: backend.enabled,
                readiness: match backend.readiness {
                    BackendReadinessV1::Disabled => AgentBackendReadinessView::Disabled,
                    BackendReadinessV1::NeedsSecretReference => {
                        AgentBackendReadinessView::NeedsSecretReference
                    }
                    BackendReadinessV1::SecretStoreUnavailable => {
                        AgentBackendReadinessView::SecretStoreUnavailable
                    }
                    BackendReadinessV1::CredentialUnverified => {
                        AgentBackendReadinessView::CredentialUnverified
                    }
                    BackendReadinessV1::CredentialMissing => {
                        AgentBackendReadinessView::CredentialMissing
                    }
                    BackendReadinessV1::CredentialInvalid => {
                        AgentBackendReadinessView::CredentialInvalid
                    }
                    BackendReadinessV1::Ready => AgentBackendReadinessView::Ready,
                },
                secret_reference_present: loaded
                    .settings
                    .agent_backends
                    .openai
                    .api_key_ref
                    .is_some(),
                test_available: backend.test_available,
            }
        },
    );
    let diagnostic = DiagnosticCheckView {
        check: DiagnosticCheckId::GlobalConfig,
        status: view_status,
        blocking: config_is_blocking(status.state),
        remediation: config_remediation(status.state),
    };
    (
        ConfigView {
            status: view_status,
            revision: status.revision,
            default_profile: status.default_profile.map(profile_from_content),
            secret_store: match secret_store.status() {
                SecretStoreStatus::Available => StatusCode::Ready,
                SecretStoreStatus::Unavailable => StatusCode::Unavailable,
            },
            providers,
            openai_backend,
            cleanup_required: status.cleanup_required,
        },
        diagnostic,
    )
}

fn unavailable_config(error: ConfigError) -> (ConfigView, DiagnosticCheckView) {
    let (status, remediation) = if error == ConfigError::HomeUnavailable {
        (StatusCode::Unavailable, RemediationCode::HomeUnavailable)
    } else {
        (StatusCode::Invalid, RemediationCode::InspectGlobalConfig)
    };
    (
        ConfigView {
            status,
            revision: None,
            default_profile: None,
            secret_store: StatusCode::Unavailable,
            providers: unavailable_providers(),
            openai_backend: AgentBackendView {
                enabled: false,
                readiness: AgentBackendReadinessView::SecretStoreUnavailable,
                secret_reference_present: false,
                test_available: false,
            },
            cleanup_required: false,
        },
        DiagnosticCheckView {
            check: DiagnosticCheckId::GlobalConfig,
            status,
            blocking: false,
            remediation,
        },
    )
}

fn provider_view(
    provider: ProviderKind,
    status: &RedactedProviderStatus,
    public_setting_present: bool,
) -> ProviderView {
    ProviderView {
        provider,
        enabled: status.enabled,
        readiness: match status.readiness {
            ProviderReadiness::Disabled => ProviderReadinessView::Disabled,
            ProviderReadiness::Ready => ProviderReadinessView::Ready,
            ProviderReadiness::NeedsSecret => ProviderReadinessView::NeedsSecret,
            ProviderReadiness::NeedsPublicSetting => ProviderReadinessView::NeedsPublicSetting,
        },
        public_setting_present,
        secret_reference_present: status.secret_ref_present,
    }
}

fn unavailable_providers() -> [ProviderView; 5] {
    ProviderKind::ALL.map(|provider| ProviderView {
        provider,
        enabled: false,
        readiness: ProviderReadinessView::Unavailable,
        public_setting_present: false,
        secret_reference_present: false,
    })
}

const fn legacy_provider_view(provider: LegacyProviderId) -> LegacyProviderView {
    match provider {
        LegacyProviderId::OpenAlex => LegacyProviderView::OpenAlex,
        LegacyProviderId::SemanticScholar => LegacyProviderView::SemanticScholar,
        LegacyProviderId::Crossref => LegacyProviderView::Crossref,
        LegacyProviderId::Pubmed => LegacyProviderView::Pubmed,
        LegacyProviderId::Arxiv => LegacyProviderView::Arxiv,
    }
}

const fn legacy_provider_resolution(
    resolution: LegacyProviderResolutionView,
) -> LegacyProviderResolution {
    LegacyProviderResolution {
        provider: match resolution.provider {
            LegacyProviderView::OpenAlex => LegacyProviderId::OpenAlex,
            LegacyProviderView::SemanticScholar => LegacyProviderId::SemanticScholar,
            LegacyProviderView::Crossref => LegacyProviderId::Crossref,
            LegacyProviderView::Pubmed => LegacyProviderId::Pubmed,
            LegacyProviderView::Arxiv => LegacyProviderId::Arxiv,
        },
        strategy: match resolution.strategy {
            LegacyProviderResolutionStrategyView::KeepV2 => {
                LegacyProviderResolutionStrategy::KeepV2
            }
            LegacyProviderResolutionStrategyView::UseLegacy => {
                LegacyProviderResolutionStrategy::UseLegacy
            }
            LegacyProviderResolutionStrategyView::MergeCompatible => {
                LegacyProviderResolutionStrategy::MergeCompatible
            }
        },
    }
}

fn legacy_provider_conflicts(
    migration: &qiongli_platform::LegacyMigrationInventory,
    environment: &CommandEnvironment,
) -> Vec<LegacyProviderConflictView> {
    let Some(legacy) = migration.legacy_provider_config() else {
        return Vec::new();
    };
    let Ok(loaded) = config_store(environment).and_then(|store| store.load()) else {
        return Vec::new();
    };
    let Ok(secret_values) = legacy.secret_values() else {
        return Vec::new();
    };
    let secret_refs = secret_values
        .into_iter()
        .filter_map(|(provider, _)| {
            let primary = match provider {
                LegacyProviderSecret::OpenAlex => "11111111111111111111111111111111",
                LegacyProviderSecret::SemanticScholar => "22222222222222222222222222222222",
                LegacyProviderSecret::Pubmed => "33333333333333333333333333333333",
            };
            let current = match provider {
                LegacyProviderSecret::OpenAlex => {
                    loaded.settings.providers.openalex.api_key_ref.as_ref()
                }
                LegacyProviderSecret::SemanticScholar => loaded
                    .settings
                    .providers
                    .semantic_scholar
                    .api_key_ref
                    .as_ref(),
                LegacyProviderSecret::Pubmed => {
                    loaded.settings.providers.pubmed.api_key_ref.as_ref()
                }
            };
            let primary = SecretRef::parse(&format!("qsr1_{primary}")).ok()?;
            let reference = if current == Some(&primary) {
                SecretRef::parse("qsr1_eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee").ok()?
            } else {
                primary
            };
            Some((provider, reference))
        })
        .collect::<Vec<_>>();
    legacy
        .provider_conflicts(&loaded, &secret_refs)
        .unwrap_or_default()
        .into_iter()
        .map(|conflict| LegacyProviderConflictView {
            provider: legacy_provider_view(conflict.provider),
            differing_fields: conflict.differing_fields,
            legacy_secret_present: conflict.legacy_secret_present,
            current_secret_reference_present: conflict.current_secret_reference_present,
        })
        .collect()
}

fn integration_snapshots(
    environment: &CommandEnvironment,
) -> (
    [(IntegrationView, DiagnosticCheckView); 2],
    LegacyMigrationView,
) {
    let Some(inventory) = environment.client_inventory() else {
        return (
            [
                unavailable_integration(
                    IntegrationTarget::Codex,
                    StatusCode::Unavailable,
                    RemediationCode::HomeUnavailable,
                ),
                unavailable_integration(
                    IntegrationTarget::ClaudeCode,
                    StatusCode::Unavailable,
                    RemediationCode::HomeUnavailable,
                ),
            ],
            LegacyMigrationView {
                state: LegacyMigrationStateView::Unavailable,
                next_action: LegacyMigrationActionView::None,
                migration_id: None,
                detected_items: 0,
                eligible_items: 0,
                review_items: 0,
                reason_code: "legacy-migration-home-unavailable",
                provider_conflicts: Vec::new(),
            },
        );
    };
    let clients = &inventory.summary().clients;
    let config_root = crate::command::config_root(environment).ok();
    let migration = discover_legacy_migration_with_config(&inventory, config_root.as_ref());
    let migration_view = legacy_migration_view(
        &migration,
        legacy_provider_conflicts(&migration, environment),
    );
    (
        [
            integration_snapshot(
                &clients[0],
                environment.codex_host_version(),
                integration_migration_view(migration.summary(), ClientKind::Codex),
            ),
            integration_snapshot(
                &clients[1],
                environment.claude_host_version(),
                integration_migration_view(migration.summary(), ClientKind::ClaudeCode),
            ),
        ],
        migration_view,
    )
}

fn legacy_migration_view(
    migration: &qiongli_platform::LegacyMigrationInventory,
    provider_conflicts: Vec<LegacyProviderConflictView>,
) -> LegacyMigrationView {
    let summary = migration.summary();
    let base = |state, next_action, migration_id, reason_code| LegacyMigrationView {
        state,
        next_action,
        migration_id,
        detected_items: summary.detected_item_count,
        eligible_items: summary.eligible_item_count,
        review_items: summary.review_item_count,
        reason_code,
        provider_conflicts: provider_conflicts.clone(),
    };
    let store = match LegacyMigrationStore::for_inventory(migration) {
        Ok(store) => store,
        Err(error) => {
            return base(
                LegacyMigrationStateView::Unavailable,
                LegacyMigrationActionView::None,
                None,
                error.reason_code(),
            );
        }
    };
    let latest = match store.load_latest() {
        Ok(latest) => latest,
        Err(error) => {
            return base(
                LegacyMigrationStateView::RecoveryRequired,
                LegacyMigrationActionView::Review,
                None,
                error.reason_code(),
            );
        }
    };
    let Some((plan, receipt)) = latest else {
        return match summary.readiness {
            LegacyMigrationReadiness::NotDetected => base(
                LegacyMigrationStateView::NotDetected,
                LegacyMigrationActionView::None,
                None,
                "legacy-migration-not-detected",
            ),
            LegacyMigrationReadiness::Ready => base(
                LegacyMigrationStateView::Available,
                LegacyMigrationActionView::Start,
                None,
                "legacy-migration-available",
            ),
            LegacyMigrationReadiness::ReviewRequired => base(
                LegacyMigrationStateView::ReviewRequired,
                LegacyMigrationActionView::Review,
                None,
                "legacy-migration-review-required",
            ),
        };
    };
    let migration_id = Some(receipt.migration_id.clone());
    if receipt.state == LegacyMigrationState::PreviewReady
        && now_unix().is_ok_and(|now| now > plan.expires_at_unix)
    {
        return if summary.readiness == LegacyMigrationReadiness::ReviewRequired {
            base(
                LegacyMigrationStateView::ReviewRequired,
                LegacyMigrationActionView::Review,
                None,
                "legacy-migration-preview-expired-review-required",
            )
        } else {
            base(
                LegacyMigrationStateView::Available,
                LegacyMigrationActionView::Start,
                None,
                "legacy-migration-preview-expired",
            )
        };
    }
    let cleanup_journal = match store.cleanup_journal_present(&plan.plan_id) {
        Ok(present) => present,
        Err(error) => {
            return base(
                LegacyMigrationStateView::RecoveryRequired,
                LegacyMigrationActionView::Review,
                migration_id,
                error.reason_code(),
            );
        }
    };
    if cleanup_journal && receipt.state != LegacyMigrationState::Complete {
        return base(
            LegacyMigrationStateView::RecoveryRequired,
            LegacyMigrationActionView::Recover,
            migration_id,
            "legacy-migration-cleanup-interrupted",
        );
    }
    match receipt.state {
        LegacyMigrationState::Detected => base(
            LegacyMigrationStateView::Available,
            LegacyMigrationActionView::Start,
            None,
            "legacy-migration-available",
        ),
        LegacyMigrationState::PreviewReady => base(
            LegacyMigrationStateView::PreviewReady,
            LegacyMigrationActionView::Apply,
            migration_id,
            "legacy-migration-preview-ready",
        ),
        LegacyMigrationState::Staged => base(
            LegacyMigrationStateView::Staged,
            LegacyMigrationActionView::ConfirmHostActivation,
            migration_id,
            "legacy-migration-install-staged",
        ),
        LegacyMigrationState::AwaitingClientActivation => base(
            LegacyMigrationStateView::AwaitingClientActivation,
            LegacyMigrationActionView::ConfirmHostActivation,
            migration_id,
            "legacy-migration-awaiting-host-activation",
        ),
        LegacyMigrationState::VerificationRequired => base(
            LegacyMigrationStateView::VerificationRequired,
            LegacyMigrationActionView::ConfirmHostActivation,
            migration_id,
            "legacy-migration-verification-required",
        ),
        LegacyMigrationState::CleanupReady => base(
            LegacyMigrationStateView::CleanupReady,
            LegacyMigrationActionView::Cleanup,
            migration_id,
            "legacy-migration-cleanup-ready",
        ),
        LegacyMigrationState::Complete if cleanup_journal => base(
            LegacyMigrationStateView::Complete,
            LegacyMigrationActionView::Finalize,
            migration_id,
            "legacy-migration-complete-finalization-pending",
        ),
        LegacyMigrationState::Complete if receipt.unresolved_item_count > 0 => base(
            LegacyMigrationStateView::ReviewRequired,
            LegacyMigrationActionView::Review,
            migration_id,
            "legacy-migration-complete-with-unresolved-items",
        ),
        LegacyMigrationState::Complete if summary.detected_item_count > 0 => {
            match summary.readiness {
                LegacyMigrationReadiness::ReviewRequired => base(
                    LegacyMigrationStateView::ReviewRequired,
                    LegacyMigrationActionView::Review,
                    None,
                    "legacy-migration-content-reappeared-review-required",
                ),
                LegacyMigrationReadiness::Ready => base(
                    LegacyMigrationStateView::Available,
                    LegacyMigrationActionView::Start,
                    None,
                    "legacy-migration-content-reappeared",
                ),
                LegacyMigrationReadiness::NotDetected => unreachable!(),
            }
        }
        LegacyMigrationState::Complete => base(
            LegacyMigrationStateView::Complete,
            LegacyMigrationActionView::None,
            migration_id,
            "legacy-migration-complete",
        ),
        LegacyMigrationState::RecoveryRequired if cleanup_journal => base(
            LegacyMigrationStateView::RecoveryRequired,
            LegacyMigrationActionView::Recover,
            migration_id,
            "legacy-migration-recovery-required",
        ),
        LegacyMigrationState::RecoveryRequired => base(
            LegacyMigrationStateView::RecoveryRequired,
            LegacyMigrationActionView::Review,
            migration_id,
            "legacy-migration-recovery-review-required",
        ),
        LegacyMigrationState::ReviewRequired => base(
            LegacyMigrationStateView::ReviewRequired,
            LegacyMigrationActionView::Review,
            migration_id,
            "legacy-migration-review-required",
        ),
    }
}

fn integration_snapshot(
    inventory: &ClientInventoryEntryV1,
    version: Option<crate::command::DetectedClientVersion>,
    migration: IntegrationMigrationView,
) -> (IntegrationView, DiagnosticCheckView) {
    let (target, check, symbolic_location, activation, remediation) = match inventory.client {
        ClientKind::Codex => (
            IntegrationTarget::Codex,
            DiagnosticCheckId::CodexLocal,
            SymbolicLocation::CodexMarketplace,
            ActivationPolicy::ClientActionRequired,
            RemediationCode::InspectCodexLocal,
        ),
        ClientKind::ClaudeCode => (
            IntegrationTarget::ClaudeCode,
            DiagnosticCheckId::ClaudeCodeLocal,
            SymbolicLocation::ClaudeMarketplace,
            ActivationPolicy::ReloadOrClientActionRequired,
            RemediationCode::InspectClaudeCodeLocal,
        ),
    };
    let source = component_status(inventory.components.plugin_source);
    let full_mcp = component_status(inventory.components.full_mcp);
    let marketplace = component_status(inventory.components.marketplace);
    let registration = component_status(inventory.components.registration);
    let compatibility = client_compatibility(inventory.client, inventory.discovery, version);
    let next_action = if compatibility == ClientCompatibilityView::Unsupported {
        IntegrationActionView::UpgradeClient
    } else {
        action_view(inventory.readiness)
    };
    let direct_package = (inventory.client == ClientKind::ClaudeCode)
        .then(|| component_status(inventory.components.skills));
    let overall = match (inventory.discovery, compatibility) {
        (ClientDiscoveryState::Detected, ClientCompatibilityView::Unsupported) => {
            StatusCode::Blocked
        }
        (ClientDiscoveryState::NotDetected, _) => StatusCode::Missing,
        (ClientDiscoveryState::Unavailable, _) => StatusCode::Unavailable,
        (ClientDiscoveryState::Detected, _) => {
            integration_overall(source, marketplace, direct_package, registration, full_mcp)
        }
    };
    let (activation_status, activation_observation) = integration_activation(registration);
    let (mcp_attachment, mcp_attachment_observation) =
        integration_mcp_attachment(inventory.discovery, full_mcp, registration);
    let (paths, path_count) = integration_paths(inventory);
    integration_result(
        IntegrationView {
            target,
            client_version: version.map(|version| ClientVersionView {
                major: version.major,
                minor: version.minor,
                patch: version.patch,
            }),
            compatibility,
            installed_plugin_version: inventory
                .installed_plugin_version
                .as_deref()
                .and_then(product_version_view),
            available_plugin_version: available_product_version_view(),
            discovery: match inventory.discovery {
                ClientDiscoveryState::NotDetected => IntegrationDiscoveryState::NotDiscovered,
                ClientDiscoveryState::Unavailable => IntegrationDiscoveryState::Unavailable,
                ClientDiscoveryState::Detected => integration_discovery(true, registration),
            },
            candidate_required: false,
            migration,
            client: match inventory.discovery {
                ClientDiscoveryState::Detected => StatusCode::Ready,
                ClientDiscoveryState::NotDetected => StatusCode::Missing,
                ClientDiscoveryState::Unavailable => StatusCode::Unavailable,
            },
            overall,
            source,
            skills: component_status(inventory.components.skills),
            marketplace,
            direct_package,
            registration,
            activation_status,
            activation_observation,
            mcp_attachment,
            mcp_attachment_observation,
            symbolic_location,
            activation,
            ownership: ownership_view(inventory.ownership),
            next_action,
            evidence_code: if compatibility == ClientCompatibilityView::Unsupported {
                "client-version-below-supported-minimum"
            } else {
                integration_evidence_code(&inventory.reason_code)
            },
            path_count,
            paths,
        },
        check,
        remediation,
    )
}

fn integration_migration_view(
    migration: &LegacyMigrationInventoryV1,
    client: ClientKind,
) -> IntegrationMigrationView {
    let items = migration
        .items
        .iter()
        .filter(|item| item.client == Some(client));
    let mut detected_items = 0;
    let mut eligible_items = 0;
    let mut review_items = 0;
    for item in items {
        match item.state {
            LegacyMigrationItemState::Missing => {}
            LegacyMigrationItemState::Eligible => {
                detected_items += 1;
                eligible_items += 1;
            }
            LegacyMigrationItemState::ReviewRequired | LegacyMigrationItemState::Unavailable => {
                detected_items += 1;
                review_items += 1;
            }
        }
    }
    let state = if review_items > 0 {
        IntegrationMigrationStateView::ReviewRequired
    } else if detected_items > 0 {
        IntegrationMigrationStateView::Available
    } else {
        IntegrationMigrationStateView::NotDetected
    };
    IntegrationMigrationView {
        state,
        detected_items,
        eligible_items,
        review_items,
    }
}

pub(crate) fn detected_client_compatibility(
    client: ClientKind,
    version: Option<crate::command::DetectedClientVersion>,
) -> ClientCompatibilityView {
    let Some(version) = version else {
        return ClientCompatibilityView::NotEvaluated;
    };
    let minimum = match client {
        ClientKind::Codex => (0, 144, 1),
        ClientKind::ClaudeCode => (2, 1, 206),
    };
    if (version.major, version.minor, version.patch) >= minimum {
        ClientCompatibilityView::Supported
    } else {
        ClientCompatibilityView::Unsupported
    }
}

fn client_compatibility(
    client: ClientKind,
    discovery: ClientDiscoveryState,
    version: Option<crate::command::DetectedClientVersion>,
) -> ClientCompatibilityView {
    if discovery != ClientDiscoveryState::Detected {
        return ClientCompatibilityView::NotEvaluated;
    }
    detected_client_compatibility(client, version)
}

pub(crate) fn managed_integration_version_is_unsupported(
    environment: &CommandEnvironment,
    target: ClientActivationTarget,
) -> bool {
    let compatibility = match target {
        ClientActivationTarget::Codex => {
            detected_client_compatibility(ClientKind::Codex, environment.codex_host_version())
        }
        ClientActivationTarget::ClaudeCode => {
            detected_client_compatibility(ClientKind::ClaudeCode, environment.claude_host_version())
        }
    };
    compatibility == ClientCompatibilityView::Unsupported
}

const fn integration_activation(
    registration: StatusCode,
) -> (StatusCode, IntegrationObservationView) {
    match registration {
        StatusCode::Ready => (
            StatusCode::Attention,
            IntegrationObservationView::ClientActionRequired,
        ),
        StatusCode::Missing => (StatusCode::Missing, IntegrationObservationView::Missing),
        StatusCode::Unavailable | StatusCode::Blocked | StatusCode::Insecure => {
            (registration, IntegrationObservationView::InspectionBlocked)
        }
        status => (status, IntegrationObservationView::NotObservable),
    }
}

const fn integration_mcp_attachment(
    discovery: ClientDiscoveryState,
    full_mcp: StatusCode,
    registration: StatusCode,
) -> (StatusCode, IntegrationObservationView) {
    if matches!(discovery, ClientDiscoveryState::Unavailable) {
        return (
            StatusCode::Unavailable,
            IntegrationObservationView::InspectionBlocked,
        );
    }
    if !matches!(full_mcp, StatusCode::Ready) || !matches!(registration, StatusCode::Ready) {
        return (StatusCode::Missing, IntegrationObservationView::Missing);
    }
    (
        StatusCode::Attention,
        IntegrationObservationView::NotObservable,
    )
}

fn probe_host_integrations(
    environment: &CommandEnvironment,
    selection: IntegrationSelection,
    observations: &mut [HostIntegrationObservation; 2],
) {
    if selection.codex {
        observations[0] = probe_codex_host(environment);
    }
    if selection.claude_code {
        observations[1] = probe_claude_host(environment);
    }
}

fn host_observation_for_target(
    observations: &[HostIntegrationObservation; 2],
    target: ClientActivationTarget,
) -> HostIntegrationObservation {
    observations[match target {
        ClientActivationTarget::Codex => 0,
        ClientActivationTarget::ClaudeCode => 1,
    }]
}

fn probe_codex_host(environment: &CommandEnvironment) -> HostIntegrationObservation {
    probe_codex_host_result(environment).unwrap_or(HostIntegrationObservation {
        activation: HostProbeState::ProbeFailed,
        mcp_attachment: HostProbeState::ProbeFailed,
    })
}

fn probe_codex_host_result(
    environment: &CommandEnvironment,
) -> Result<HostIntegrationObservation, &'static str> {
    if environment.codex_host_version().is_none() {
        return Ok(HostIntegrationObservation {
            activation: HostProbeState::ProbeUnavailable,
            mcp_attachment: HostProbeState::ProbeUnavailable,
        });
    }
    let Some(discovered) = environment.client_executable("codex") else {
        return Ok(HostIntegrationObservation {
            activation: HostProbeState::ProbeUnavailable,
            mcp_attachment: HostProbeState::ProbeUnavailable,
        });
    };
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let executable = resolve_host_plugin_executable(home, "codex", &discovered)?;
    let expected_source = home.join(".qiongli/plugins/codex/qiongli-next");
    let plugin_list =
        bounded_host_command_result(environment, &executable, &["plugin", "list", "--json"])
            .map_err(HostCommandFailure::reason_code)?;
    let activation = codex_plugin_state(&plugin_list, env!("CARGO_PKG_VERSION"), &expected_source)?;
    if activation != HostProbeState::Observed {
        return Ok(HostIntegrationObservation {
            activation,
            mcp_attachment: activation,
        });
    }
    if !codex_host_cache_matches_managed_source(environment) {
        return Ok(HostIntegrationObservation {
            activation: HostProbeState::CacheDrift,
            mcp_attachment: HostProbeState::CacheDrift,
        });
    }
    let mcp_list =
        bounded_host_command_result(environment, &executable, &["mcp", "list", "--json"])
            .map_err(HostCommandFailure::reason_code)?;
    Ok(HostIntegrationObservation {
        activation: HostProbeState::Observed,
        mcp_attachment: codex_mcp_state(&mcp_list)?,
    })
}

fn probe_claude_host(environment: &CommandEnvironment) -> HostIntegrationObservation {
    probe_claude_host_result(environment).unwrap_or(HostIntegrationObservation {
        activation: HostProbeState::ProbeFailed,
        mcp_attachment: HostProbeState::ProbeFailed,
    })
}

fn probe_claude_host_result(
    environment: &CommandEnvironment,
) -> Result<HostIntegrationObservation, &'static str> {
    if environment.claude_host_version().is_none() {
        return Ok(HostIntegrationObservation {
            activation: HostProbeState::ProbeUnavailable,
            mcp_attachment: HostProbeState::ProbeUnavailable,
        });
    }
    let Some(discovered) = environment.client_executable("claude") else {
        return Ok(HostIntegrationObservation {
            activation: HostProbeState::ProbeUnavailable,
            mcp_attachment: HostProbeState::ProbeUnavailable,
        });
    };
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let executable = resolve_host_plugin_executable(home, "claude", &discovered)?;
    let expected_cache = environment
        .claude_config_root()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".claude"))
        .join("plugins/cache/qiongli-local/qiongli-next")
        .join(env!("CARGO_PKG_VERSION"));
    let plugin_list =
        bounded_host_command_result(environment, &executable, &["plugin", "list", "--json"])
            .map_err(HostCommandFailure::reason_code)?;
    let activation = claude_plugin_state(&plugin_list, env!("CARGO_PKG_VERSION"), &expected_cache)?;
    if activation != HostProbeState::Observed {
        return Ok(HostIntegrationObservation {
            activation,
            mcp_attachment: activation,
        });
    }
    if !claude_host_cache_matches_managed_source(environment) {
        return Ok(HostIntegrationObservation {
            activation: HostProbeState::CacheDrift,
            mcp_attachment: HostProbeState::CacheDrift,
        });
    }
    let details = bounded_host_command_result(
        environment,
        &executable,
        &["plugin", "details", CLAUDE_HOST_PLUGIN_ID],
    )
    .map_err(HostCommandFailure::reason_code)?;
    Ok(HostIntegrationObservation {
        activation: HostProbeState::Observed,
        mcp_attachment: claude_component_state(&details)?,
    })
}

#[derive(serde::Deserialize)]
struct CodexPluginInventory {
    installed: Vec<CodexInstalledPlugin>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexInstalledPlugin {
    plugin_id: String,
    version: String,
    installed: bool,
    enabled: bool,
    source: CodexPluginSource,
}

#[derive(serde::Deserialize)]
struct CodexPluginSource {
    source: String,
    path: PathBuf,
}

#[derive(serde::Deserialize)]
struct CodexMcpInventoryEntry {
    name: String,
    enabled: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeInstalledPlugin {
    id: String,
    version: String,
    enabled: bool,
    scope: String,
    install_path: PathBuf,
}

fn codex_plugin_state(
    output: &str,
    expected_version: &str,
    expected_source: &Path,
) -> Result<HostProbeState, &'static str> {
    let inventory: CodexPluginInventory =
        serde_json::from_str(output).map_err(|_| "codex-plugin-inventory-json-invalid")?;
    let mut plugins = inventory
        .installed
        .iter()
        .filter(|plugin| plugin.plugin_id == CODEX_HOST_PLUGIN_ID);
    let Some(plugin) = plugins.next() else {
        return Ok(HostProbeState::HostActionRequired);
    };
    if plugins.next().is_some() {
        return Ok(HostProbeState::CacheDrift);
    }
    Ok(
        if plugin.version == expected_version
            && plugin.installed
            && plugin.enabled
            && plugin.source.source == "local"
            && observed_path_matches(expected_source, &plugin.source.path)
        {
            HostProbeState::Observed
        } else {
            HostProbeState::CacheDrift
        },
    )
}

fn codex_mcp_state(output: &str) -> Result<HostProbeState, &'static str> {
    let inventory: Vec<CodexMcpInventoryEntry> =
        serde_json::from_str(output).map_err(|_| "codex-mcp-inventory-json-invalid")?;
    let mut entries = inventory.iter().filter(|entry| entry.name == HOST_MCP_ID);
    let Some(entry) = entries.next() else {
        return Ok(HostProbeState::HostActionRequired);
    };
    Ok(if entry.enabled && entries.next().is_none() {
        HostProbeState::Observed
    } else {
        HostProbeState::CacheDrift
    })
}

fn claude_plugin_state(
    output: &str,
    expected_version: &str,
    expected_cache: &Path,
) -> Result<HostProbeState, &'static str> {
    let inventory: Vec<ClaudeInstalledPlugin> =
        serde_json::from_str(output).map_err(|_| "claude-plugin-inventory-json-invalid")?;
    let mut plugins = inventory
        .iter()
        .filter(|plugin| plugin.id == CLAUDE_HOST_PLUGIN_ID);
    let Some(plugin) = plugins.next() else {
        return Ok(HostProbeState::HostActionRequired);
    };
    if plugins.next().is_some() {
        return Ok(HostProbeState::CacheDrift);
    }
    Ok(
        if plugin.version == expected_version
            && plugin.enabled
            && plugin.scope == "user"
            && observed_path_matches(expected_cache, &plugin.install_path)
        {
            HostProbeState::Observed
        } else {
            HostProbeState::CacheDrift
        },
    )
}

fn claude_component_state(output: &str) -> Result<HostProbeState, &'static str> {
    if !output
        .lines()
        .any(|line| line.trim() == "Component inventory")
    {
        return Err("claude-plugin-details-invalid");
    }
    let skills = claude_component_line(output, "Skills")?;
    let mcp = claude_component_line(output, "MCP servers")?;
    Ok(
        if skills.0 == 1
            && skills.1.as_slice() == [CLAUDE_SKILL_ID]
            && mcp.0 == 1
            && mcp.1.first() == Some(&HOST_MCP_ID)
        {
            HostProbeState::Observed
        } else {
            HostProbeState::HostActionRequired
        },
    )
}

fn claude_component_line<'a>(
    output: &'a str,
    label: &str,
) -> Result<(usize, Vec<&'a str>), &'static str> {
    let mut lines = output
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix(label).map(str::trim));
    let line = lines.next().ok_or("claude-plugin-details-invalid")?;
    if lines.next().is_some() {
        return Err("claude-plugin-details-invalid");
    }
    let count_end = line
        .strip_prefix('(')
        .and_then(|line| line.find(')').map(|end| (line, end)))
        .ok_or("claude-plugin-details-invalid")?;
    let count = count_end
        .0
        .get(..count_end.1)
        .and_then(|count| count.parse::<usize>().ok())
        .ok_or("claude-plugin-details-invalid")?;
    let components = count_end
        .0
        .get(count_end.1 + 1..)
        .ok_or("claude-plugin-details-invalid")?
        .split_whitespace()
        .collect();
    Ok((count, components))
}

fn observed_path_matches(expected: &Path, observed: &Path) -> bool {
    if !observed.is_absolute() {
        return false;
    }
    match (fs::canonicalize(expected), fs::canonicalize(observed)) {
        (Ok(expected), Ok(observed)) => expected == observed,
        _ => expected == observed,
    }
}

fn codex_host_cache_matches_managed_source(environment: &CommandEnvironment) -> bool {
    let Some(home) = environment.platform_home() else {
        return false;
    };
    let config_root = environment
        .codex_config_root()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".codex"));
    let managed =
        verified_codex_bundle_receipt_sha256(&home.join(".qiongli/plugins/codex/qiongli-next"));
    let cached = verified_codex_bundle_receipt_sha256(
        &config_root
            .join("plugins/cache/personal/qiongli-next")
            .join(env!("CARGO_PKG_VERSION")),
    );
    verified_bundle_receipts_match(managed.as_deref(), cached.as_deref())
}

fn claude_host_cache_matches_managed_source(environment: &CommandEnvironment) -> bool {
    let Some(home) = environment.platform_home() else {
        return false;
    };
    let config_root = environment
        .claude_config_root()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".claude"));
    let managed = verified_claude_bundle_receipt_sha256(
        &home.join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next"),
    );
    let cached = verified_claude_bundle_receipt_sha256(
        &config_root
            .join("plugins/cache/qiongli-local/qiongli-next")
            .join(env!("CARGO_PKG_VERSION")),
    );
    verified_bundle_receipts_match(managed.as_deref(), cached.as_deref())
}

fn verified_bundle_receipts_match(managed: Option<&str>, cached: Option<&str>) -> bool {
    matches!((managed, cached), (Some(managed), Some(cached)) if managed == cached)
}

fn verified_codex_bundle_receipt_sha256(path: &Path) -> Option<String> {
    let target = approve_codex_plugin_bundle_target(path).ok()?;
    verify_codex_plugin_bundle(&target)
        .ok()
        .map(|bundle| bundle.receipt_sha256().to_owned())
}

fn verified_claude_bundle_receipt_sha256(path: &Path) -> Option<String> {
    let target = approve_claude_plugin_bundle_target(path).ok()?;
    verify_claude_plugin_bundle(&target)
        .ok()
        .map(|bundle| bundle.receipt_sha256().to_owned())
}

fn test_cli_shell_command(environment: &CommandEnvironment) -> (CliPathState, &'static str) {
    let Some(home) = environment.platform_home() else {
        return (
            CliPathState::NotObservable,
            "qiongli-cli-shell-test-home-unavailable",
        );
    };
    let target = if cfg!(windows) {
        home.join("AppData/Local/Qiongli/bin/qiongli.exe")
    } else {
        home.join(".local/bin/qiongli")
    };
    if !target.is_file() {
        return (
            CliPathState::NotObservable,
            "qiongli-cli-shell-test-target-missing",
        );
    }
    let Some(shell) = supported_cli_test_shell() else {
        return (
            CliPathState::NotObservable,
            "qiongli-cli-shell-test-unavailable",
        );
    };
    let Some(output) = bounded_host_command(
        environment,
        &shell,
        &[
            "-lic",
            "resolved=\"$(command -v qiongli 2>/dev/null || true)\"; printf '__QIONGLI_COMMAND__=%s\\n' \"$resolved\"",
        ],
    ) else {
        return (CliPathState::NotObservable, "qiongli-cli-shell-test-failed");
    };
    let (state, reason_code) = classify_cli_shell_resolution(&target, &output);
    if state != CliPathState::Active {
        return (state, reason_code);
    }
    let Some(bundled) = bundled_cli_path(Some(home)) else {
        return (
            CliPathState::NotObservable,
            "qiongli-cli-bundle-unavailable",
        );
    };
    if !cli_target_matches_bundled(home, &bundled).unwrap_or(false) {
        return (
            CliPathState::VersionMismatch,
            "qiongli-cli-shell-version-mismatch",
        );
    }
    let Some(version) = bounded_host_command(environment, &target, &["--version"]) else {
        return (
            CliPathState::NotObservable,
            "qiongli-cli-shell-version-unavailable",
        );
    };
    classify_cli_shell_version(&version)
}

fn supported_cli_test_shell() -> Option<PathBuf> {
    let configured = std::env::var_os("SHELL").map(PathBuf::from);
    configured
        .filter(|path| {
            path.is_file()
                && matches!(
                    path.to_str(),
                    Some("/bin/zsh" | "/bin/bash" | "/usr/bin/zsh" | "/usr/bin/bash")
                )
        })
        .or_else(|| {
            let fallback = if cfg!(target_os = "macos") {
                PathBuf::from("/bin/zsh")
            } else {
                PathBuf::from("/bin/bash")
            };
            fallback.is_file().then_some(fallback)
        })
}

fn classify_cli_shell_resolution(target: &Path, output: &str) -> (CliPathState, &'static str) {
    const COMMAND_MARKER: &str = "__QIONGLI_COMMAND__=";

    let Some(candidate) = output
        .lines()
        .rev()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(COMMAND_MARKER))
        .filter(|value| !value.is_empty())
        .map(Path::new)
        .filter(|path| {
            path.is_absolute() && path.file_name().and_then(OsStr::to_str) == Some("qiongli")
        })
    else {
        return (
            CliPathState::NotConfigured,
            "qiongli-cli-shell-command-missing",
        );
    };
    let target_identity = fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    let candidate_identity =
        fs::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf());
    if target_identity == candidate_identity {
        (CliPathState::Active, "qiongli-cli-shell-command-active")
    } else {
        (CliPathState::Shadowed, "qiongli-cli-shell-command-shadowed")
    }
}

fn classify_cli_shell_version(output: &str) -> (CliPathState, &'static str) {
    if output.contains(env!("CARGO_PKG_VERSION")) {
        (CliPathState::Active, "qiongli-cli-shell-command-active")
    } else {
        (
            CliPathState::VersionMismatch,
            "qiongli-cli-shell-version-mismatch",
        )
    }
}

fn bounded_host_command(
    environment: &CommandEnvironment,
    executable: &Path,
    arguments: &[&str],
) -> Option<String> {
    bounded_host_command_result(environment, executable, arguments).ok()
}

fn bounded_host_command_result(
    environment: &CommandEnvironment,
    executable: &Path,
    arguments: &[&str],
) -> Result<String, HostCommandFailure> {
    bounded_host_command_with_timeout(environment, executable, arguments, Duration::from_secs(5))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HostCommandFailure {
    Spawn,
    Timeout,
    Wait,
    OutputRead,
    OutputTooLarge,
    InvalidUtf8,
    NonZeroExit,
    Environment,
}

impl HostCommandFailure {
    const fn reason_code(self) -> &'static str {
        match self {
            Self::Spawn => "host-command-spawn-failed",
            Self::Timeout => "host-command-timeout",
            Self::Wait => "host-command-wait-failed",
            Self::OutputRead => "host-command-output-read-failed",
            Self::OutputTooLarge => "host-command-output-too-large",
            Self::InvalidUtf8 => "host-command-output-not-utf8",
            Self::NonZeroExit => "host-command-nonzero-exit",
            Self::Environment => "host-command-environment-invalid",
        }
    }
}

fn bounded_host_command_with_timeout(
    environment: &CommandEnvironment,
    executable: &Path,
    arguments: &[&str],
    timeout: Duration,
) -> Result<String, HostCommandFailure> {
    bounded_host_os_command_with_timeout(
        environment,
        executable,
        &arguments.iter().map(OsString::from).collect::<Vec<_>>(),
        timeout,
    )
}

#[allow(
    clippy::disallowed_methods,
    reason = "ARC-213 permits only resolved official Host CLIs, the managed CLI, or a fixed login shell through this bounded launcher"
)]
fn bounded_host_os_command_with_timeout(
    environment: &CommandEnvironment,
    executable: &Path,
    arguments: &[OsString],
    timeout: Duration,
) -> Result<String, HostCommandFailure> {
    const MAX_HOST_PROBE_OUTPUT_BYTES: usize = 512 * 1024;

    let mut command = Command::new(executable);
    command
        .env_clear()
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("NO_COLOR", "1")
        .env("LC_ALL", "C")
        .env("PATH", host_command_search_path(environment, executable)?);
    if let Some(home) = environment.platform_home() {
        command.current_dir(home).env("HOME", home);
        #[cfg(windows)]
        command.env("USERPROFILE", home);
    }
    if let Some(root) = environment.codex_config_root() {
        command.env("CODEX_HOME", root);
    }
    if let Some(root) = environment.claude_config_root() {
        command.env("CLAUDE_CONFIG_DIR", root);
    }
    let mut child = command.spawn().map_err(|_| HostCommandFailure::Spawn)?;
    let stdout = child.stdout.take().ok_or(HostCommandFailure::OutputRead)?;
    let stderr = child.stderr.take().ok_or(HostCommandFailure::OutputRead)?;
    let stdout_reader =
        thread::spawn(move || read_bounded_host_output(stdout, MAX_HOST_PROBE_OUTPUT_BYTES));
    let stderr_reader =
        thread::spawn(move || read_bounded_host_output(stderr, MAX_HOST_PROBE_OUTPUT_BYTES));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(HostCommandFailure::Timeout);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(HostCommandFailure::Wait);
            }
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| HostCommandFailure::OutputRead)??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| HostCommandFailure::OutputRead)??;
    if !status?.success() {
        return Err(HostCommandFailure::NonZeroExit);
    }
    let stdout = String::from_utf8(stdout).map_err(|_| HostCommandFailure::InvalidUtf8)?;
    String::from_utf8(stderr).map_err(|_| HostCommandFailure::InvalidUtf8)?;
    Ok(stdout)
}

fn read_bounded_host_output(
    mut reader: impl std::io::Read,
    maximum_bytes: usize,
) -> Result<Vec<u8>, HostCommandFailure> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8 * 1024];
    let mut exceeded = false;
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|_| HostCommandFailure::OutputRead)?;
        if read == 0 {
            break;
        }
        if output.len().saturating_add(read) <= maximum_bytes {
            output.extend_from_slice(&buffer[..read]);
        } else {
            exceeded = true;
        }
    }
    if exceeded {
        Err(HostCommandFailure::OutputTooLarge)
    } else {
        Ok(output)
    }
}

fn host_command_search_path(
    environment: &CommandEnvironment,
    executable: &Path,
) -> Result<OsString, HostCommandFailure> {
    let mut paths = Vec::new();
    if let Some(parent) = executable.parent() {
        paths.push(parent.to_path_buf());
    }
    if let Some(home) = environment.platform_home() {
        paths.extend([
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
    paths.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ]);
    #[cfg(unix)]
    paths.extend([
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/usr/sbin"),
        PathBuf::from("/sbin"),
    ]);
    paths.sort();
    paths.dedup();
    std::env::join_paths(paths).map_err(|_| HostCommandFailure::Environment)
}

fn apply_host_observation(
    integration: &mut IntegrationView,
    observation: HostIntegrationObservation,
) {
    match observation.activation {
        HostProbeState::NotRun => {}
        HostProbeState::Observed => {
            integration.activation_status = StatusCode::Ready;
            integration.activation_observation = IntegrationObservationView::Observed;
        }
        HostProbeState::HostActionRequired => {
            integration.activation_status = StatusCode::Attention;
            integration.activation_observation = IntegrationObservationView::ClientActionRequired;
            if integration.next_action == IntegrationActionView::Current {
                integration.next_action = IntegrationActionView::InstallReady;
            }
            integration.evidence_code = "qiongli-plugin-host-action-required";
        }
        HostProbeState::CacheDrift => {
            integration.activation_status = StatusCode::Drifted;
            integration.activation_observation = IntegrationObservationView::ClientActionRequired;
            integration.next_action = IntegrationActionView::RepairReady;
            integration.evidence_code = "qiongli-plugin-cache-drift";
        }
        HostProbeState::CommandFailed => {
            integration.activation_status = StatusCode::Attention;
            integration.activation_observation = IntegrationObservationView::ProbeFailed;
            integration.next_action = IntegrationActionView::RepairReady;
            integration.evidence_code = "qiongli-plugin-command-failed";
        }
        HostProbeState::ProbeUnavailable => {
            integration.activation_status = StatusCode::Unavailable;
            integration.activation_observation = IntegrationObservationView::ProbeUnavailable;
            integration.evidence_code = "qiongli-plugin-probe-unavailable";
        }
        HostProbeState::ProbeFailed => {
            integration.activation_status = StatusCode::Unavailable;
            integration.activation_observation = IntegrationObservationView::ProbeFailed;
            integration.evidence_code = "qiongli-plugin-probe-failed";
        }
    }
    match observation.mcp_attachment {
        HostProbeState::NotRun => {}
        HostProbeState::Observed => {
            integration.mcp_attachment = StatusCode::Ready;
            integration.mcp_attachment_observation = IntegrationObservationView::Observed;
        }
        HostProbeState::HostActionRequired => {
            integration.mcp_attachment = StatusCode::Attention;
            integration.mcp_attachment_observation =
                IntegrationObservationView::ClientActionRequired;
            if observation.activation == HostProbeState::Observed {
                integration.next_action = IntegrationActionView::RepairReady;
                integration.evidence_code = "qiongli-plugin-components-missing";
            }
        }
        HostProbeState::CacheDrift => {
            integration.mcp_attachment = StatusCode::Drifted;
            integration.mcp_attachment_observation =
                IntegrationObservationView::ClientActionRequired;
            integration.next_action = IntegrationActionView::RepairReady;
            integration.evidence_code = "qiongli-plugin-cache-drift";
        }
        HostProbeState::CommandFailed => {
            integration.mcp_attachment = StatusCode::Attention;
            integration.mcp_attachment_observation = IntegrationObservationView::ProbeFailed;
            integration.next_action = IntegrationActionView::RepairReady;
            integration.evidence_code = "qiongli-plugin-command-failed";
        }
        HostProbeState::ProbeUnavailable => {
            integration.mcp_attachment = StatusCode::Unavailable;
            integration.mcp_attachment_observation = IntegrationObservationView::ProbeUnavailable;
            integration.evidence_code = "qiongli-plugin-probe-unavailable";
        }
        HostProbeState::ProbeFailed => {
            integration.mcp_attachment = StatusCode::Unavailable;
            integration.mcp_attachment_observation = IntegrationObservationView::ProbeFailed;
            integration.evidence_code = "qiongli-plugin-probe-failed";
        }
    }
    integration.overall = integration_runtime_overall(integration);
}

fn host_observation_ready(observation: HostIntegrationObservation) -> bool {
    observation.activation == HostProbeState::Observed
        && observation.mcp_attachment == HostProbeState::Observed
}

fn integration_runtime_overall(integration: &IntegrationView) -> StatusCode {
    let states = [
        integration.source,
        integration.skills,
        integration.marketplace,
        integration.direct_package.unwrap_or(StatusCode::Ready),
        integration.registration,
        integration.activation_status,
        integration.mcp_attachment,
    ];
    for priority in [
        StatusCode::RecoveryRequired,
        StatusCode::Conflict,
        StatusCode::Drifted,
        StatusCode::Invalid,
        StatusCode::Insecure,
        StatusCode::Blocked,
        StatusCode::FutureSchema,
        StatusCode::Busy,
        StatusCode::Unavailable,
        StatusCode::WriteUnsupported,
        StatusCode::Attention,
        StatusCode::Disabled,
        StatusCode::Missing,
    ] {
        if states.contains(&priority) {
            return priority;
        }
    }
    StatusCode::Ready
}

fn available_product_version_view() -> ProductVersionView {
    product_version_view(crate::DESKTOP_PRODUCT_VERSION).unwrap_or(ProductVersionView {
        major: 0,
        minor: 0,
        patch: 0,
        channel: ProductVersionChannelView::Alpha,
        prerelease_number: None,
    })
}

fn product_version_view(value: &str) -> Option<ProductVersionView> {
    let version = semver::Version::parse(value).ok()?;
    let prerelease = version.pre.as_str();
    let (channel, prerelease_number) = if prerelease.is_empty() {
        (ProductVersionChannelView::Stable, None)
    } else if let Some(number) = prerelease.strip_prefix("alpha.") {
        (ProductVersionChannelView::Alpha, Some(number.parse().ok()?))
    } else {
        let number = prerelease.strip_prefix("beta.")?;
        (ProductVersionChannelView::Beta, Some(number.parse().ok()?))
    };
    Some(ProductVersionView {
        major: version.major,
        minor: version.minor,
        patch: version.patch,
        channel,
        prerelease_number,
    })
}

fn integration_paths(
    inventory: &ClientInventoryEntryV1,
) -> ([Option<IntegrationPathView>; MAX_INTEGRATION_PATHS], usize) {
    let mut paths = EMPTY_INTEGRATION_PATHS;
    for (slot, candidate) in paths.iter_mut().zip(&inventory.paths) {
        *slot = Some(IntegrationPathView {
            surface: match candidate.surface {
                ClientPathSurface::ClientConfig => IntegrationPathSurfaceView::ClientConfig,
                ClientPathSurface::SkillsRoot => IntegrationPathSurfaceView::SkillsRoot,
                ClientPathSurface::SkillsPackage => IntegrationPathSurfaceView::SkillsPackage,
                ClientPathSurface::PluginMarketplace => {
                    IntegrationPathSurfaceView::PluginMarketplace
                }
                ClientPathSurface::PluginSource => IntegrationPathSurfaceView::PluginSource,
                ClientPathSurface::StandaloneMcp => IntegrationPathSurfaceView::StandaloneMcp,
            },
            scope: match candidate.scope {
                ClientPathScope::User => IntegrationPathScopeView::User,
                ClientPathScope::Project => IntegrationPathScopeView::Project,
                ClientPathScope::Managed => IntegrationPathScopeView::Managed,
                ClientPathScope::Custom => IntegrationPathScopeView::Custom,
                ClientPathScope::Legacy => IntegrationPathScopeView::Legacy,
            },
            source: match candidate.source {
                ClientPathSource::EnvironmentOverride => {
                    IntegrationPathSourceView::EnvironmentOverride
                }
                ClientPathSource::OfficialDefault => IntegrationPathSourceView::OfficialDefault,
                ClientPathSource::ProjectContext => IntegrationPathSourceView::ProjectContext,
                ClientPathSource::QiongliManaged => IntegrationPathSourceView::QiongliManaged,
                ClientPathSource::ExplicitCustom => IntegrationPathSourceView::ExplicitCustom,
                ClientPathSource::LegacyObserved => IntegrationPathSourceView::LegacyObserved,
            },
            state: path_status(candidate.state),
            management: match candidate.management {
                ClientPathManagement::Supported => IntegrationPathManagementView::Supported,
                ClientPathManagement::InspectOnly => IntegrationPathManagementView::InspectOnly,
                ClientPathManagement::LegacyOnly => IntegrationPathManagementView::LegacyOnly,
                ClientPathManagement::Unsafe => IntegrationPathManagementView::Unsafe,
                ClientPathManagement::Unavailable => IntegrationPathManagementView::Unavailable,
            },
            selected: candidate.selected,
            symbolic_path: candidate.symbolic_path.display(),
        });
    }
    (paths, inventory.paths.len())
}

const fn component_status(state: ClientComponentState) -> StatusCode {
    match state {
        ClientComponentState::Missing => StatusCode::Missing,
        ClientComponentState::Ready => StatusCode::Ready,
        ClientComponentState::Conflict => StatusCode::Conflict,
        ClientComponentState::Drifted => StatusCode::Drifted,
        ClientComponentState::RecoveryRequired => StatusCode::RecoveryRequired,
        ClientComponentState::Unavailable => StatusCode::Unavailable,
    }
}

const fn path_status(state: ClientPathState) -> StatusCode {
    match state {
        ClientPathState::Missing => StatusCode::Missing,
        ClientPathState::Directory | ClientPathState::File | ClientPathState::Symlink => {
            StatusCode::Ready
        }
        ClientPathState::Invalid | ClientPathState::Unsafe => StatusCode::Invalid,
        ClientPathState::Unavailable => StatusCode::Unavailable,
    }
}

const fn ownership_view(state: ClientOwnershipState) -> IntegrationOwnershipView {
    match state {
        ClientOwnershipState::NotInstalled => IntegrationOwnershipView::NotInstalled,
        ClientOwnershipState::QiongliManaged => IntegrationOwnershipView::QiongliManaged,
        ClientOwnershipState::Unmanaged => IntegrationOwnershipView::Unmanaged,
        ClientOwnershipState::Mixed => IntegrationOwnershipView::Mixed,
        ClientOwnershipState::Unknown => IntegrationOwnershipView::Unknown,
    }
}

const fn action_view(state: ClientActionReadiness) -> IntegrationActionView {
    match state {
        ClientActionReadiness::InspectOnly => IntegrationActionView::InspectOnly,
        ClientActionReadiness::InstallReady => IntegrationActionView::InstallReady,
        ClientActionReadiness::Current => IntegrationActionView::Current,
        ClientActionReadiness::RepairReady => IntegrationActionView::RepairReady,
        ClientActionReadiness::ResolveConflict => IntegrationActionView::ResolveConflict,
        ClientActionReadiness::Unavailable => IntegrationActionView::Unavailable,
    }
}

fn integration_install_required(
    states: [(IntegrationActionView, ClientCompatibilityView); 2],
    selection: IntegrationSelection,
) -> Result<bool, &'static str> {
    if selection.is_empty() {
        return Err("integration-selection-required");
    }
    let mut install_required = false;
    let mut repair_selected = false;
    for (selected, (action, compatibility)) in [
        (selection.codex, states[0]),
        (selection.claude_code, states[1]),
    ] {
        if !selected {
            continue;
        }
        if compatibility == ClientCompatibilityView::Unsupported {
            return Err("integration-client-version-unsupported");
        }
        match action {
            IntegrationActionView::InstallReady => install_required = true,
            IntegrationActionView::Current => {}
            IntegrationActionView::RepairReady => repair_selected = true,
            _ => return Err("integration-install-selection-invalid"),
        }
    }
    if repair_selected && !install_required {
        return Err("integration-install-selection-invalid");
    }
    Ok(install_required)
}

fn integration_reconcile_required(
    states: [(IntegrationActionView, ClientCompatibilityView); 2],
    selection: IntegrationSelection,
) -> Result<bool, &'static str> {
    if selection.is_empty() {
        return Err("integration-selection-required");
    }
    let mut repair_required = false;
    for (selected, (action, compatibility)) in [
        (selection.codex, states[0]),
        (selection.claude_code, states[1]),
    ] {
        if !selected {
            continue;
        }
        if compatibility == ClientCompatibilityView::Unsupported {
            return Err("integration-client-version-unsupported");
        }
        match action {
            IntegrationActionView::Current => {}
            IntegrationActionView::RepairReady => repair_required = true,
            _ => return Err("integration-reconcile-selection-invalid"),
        }
    }
    Ok(repair_required)
}

fn integration_evidence_code(reason: &str) -> &'static str {
    match reason {
        "client-not-detected" => "client-not-detected",
        "client-detected-install-ready" => "client-detected-install-ready",
        "client-managed-current" => "client-managed-current",
        "client-managed-repair-ready" => "client-managed-repair-ready",
        "client-registration-conflict" => "client-registration-conflict",
        "client-inventory-unavailable" => "client-inventory-unavailable",
        _ => "client-adapter-discovery-unavailable",
    }
}

fn integration_result(
    view: IntegrationView,
    check: DiagnosticCheckId,
    remediation: RemediationCode,
) -> (IntegrationView, DiagnosticCheckView) {
    (
        view,
        DiagnosticCheckView {
            check,
            status: view.overall,
            blocking: integration_is_blocking(view.overall),
            remediation: if view.overall == StatusCode::Ready {
                RemediationCode::None
            } else {
                remediation
            },
        },
    )
}

const fn integration_discovery(
    client_discovered: bool,
    registration: StatusCode,
) -> IntegrationDiscoveryState {
    if !client_discovered {
        return IntegrationDiscoveryState::NotDiscovered;
    }
    match registration {
        StatusCode::Ready => IntegrationDiscoveryState::Managed,
        StatusCode::Drifted => IntegrationDiscoveryState::Drifted,
        StatusCode::Conflict => IntegrationDiscoveryState::Conflict,
        StatusCode::RecoveryRequired => IntegrationDiscoveryState::RecoveryRequired,
        StatusCode::Missing => IntegrationDiscoveryState::DiscoveredUnmanaged,
        StatusCode::Attention
        | StatusCode::Unavailable
        | StatusCode::Disabled
        | StatusCode::Blocked
        | StatusCode::Invalid
        | StatusCode::FutureSchema
        | StatusCode::Insecure
        | StatusCode::Busy
        | StatusCode::WriteUnsupported => IntegrationDiscoveryState::Unavailable,
    }
}

fn unavailable_integration(
    target: IntegrationTarget,
    status: StatusCode,
    remediation: RemediationCode,
) -> (IntegrationView, DiagnosticCheckView) {
    let (check, symbolic_location, activation, direct_package) = match target {
        IntegrationTarget::Codex => (
            DiagnosticCheckId::CodexLocal,
            SymbolicLocation::CodexMarketplace,
            ActivationPolicy::ClientActionRequired,
            None,
        ),
        IntegrationTarget::ClaudeCode => (
            DiagnosticCheckId::ClaudeCodeLocal,
            SymbolicLocation::ClaudeMarketplace,
            ActivationPolicy::ReloadOrClientActionRequired,
            Some(status),
        ),
    };
    let view = IntegrationView {
        target,
        client_version: None,
        compatibility: ClientCompatibilityView::NotEvaluated,
        installed_plugin_version: None,
        available_plugin_version: available_product_version_view(),
        discovery: IntegrationDiscoveryState::Unavailable,
        candidate_required: false,
        migration: IntegrationMigrationView {
            state: IntegrationMigrationStateView::Unavailable,
            detected_items: 0,
            eligible_items: 0,
            review_items: 0,
        },
        client: status,
        overall: status,
        source: status,
        skills: status,
        marketplace: status,
        direct_package,
        registration: status,
        activation_status: status,
        activation_observation: IntegrationObservationView::InspectionBlocked,
        mcp_attachment: status,
        mcp_attachment_observation: IntegrationObservationView::InspectionBlocked,
        symbolic_location,
        activation,
        ownership: IntegrationOwnershipView::Unknown,
        next_action: IntegrationActionView::Unavailable,
        evidence_code: "client-inventory-home-unavailable",
        path_count: 0,
        paths: EMPTY_INTEGRATION_PATHS,
    };
    (
        view,
        DiagnosticCheckView {
            check,
            status,
            blocking: false,
            remediation,
        },
    )
}

const fn profile_from_content(profile: ProfileId) -> ProfileKind {
    match profile {
        ProfileId::SkillOnly => ProfileKind::SkillOnly,
        ProfileId::MarketplaceLite => ProfileKind::MarketplaceLite,
        ProfileId::Full => ProfileKind::Full,
    }
}

const fn profile_to_content(profile: ProfileKind) -> ProfileId {
    match profile {
        ProfileKind::SkillOnly => ProfileId::SkillOnly,
        ProfileKind::MarketplaceLite => ProfileId::MarketplaceLite,
        ProfileKind::Full => ProfileId::Full,
    }
}

const fn operating_system_view(os: Option<OperatingSystem>) -> OperatingSystemView {
    match os {
        Some(OperatingSystem::Linux) => OperatingSystemView::Linux,
        Some(OperatingSystem::Macos) => OperatingSystemView::MacOs,
        Some(OperatingSystem::Windows) => OperatingSystemView::Windows,
        None => OperatingSystemView::Unsupported,
    }
}

const fn architecture_view(architecture: Option<Architecture>) -> ArchitectureView {
    match architecture {
        Some(Architecture::Aarch64) => ArchitectureView::Aarch64,
        Some(Architecture::X86_64) => ArchitectureView::X86_64,
        None => ArchitectureView::Unsupported,
    }
}

const fn config_status(state: ConfigState) -> StatusCode {
    match state {
        ConfigState::Missing => StatusCode::Missing,
        ConfigState::Ready => StatusCode::Ready,
        ConfigState::Invalid => StatusCode::Invalid,
        ConfigState::FutureSchema => StatusCode::FutureSchema,
        ConfigState::Insecure => StatusCode::Insecure,
        ConfigState::Busy => StatusCode::Busy,
        ConfigState::RecoveryRequired => StatusCode::RecoveryRequired,
        ConfigState::WriteUnsupported => StatusCode::WriteUnsupported,
    }
}

const fn config_is_blocking(state: ConfigState) -> bool {
    !matches!(state, ConfigState::Missing | ConfigState::Ready)
}

const fn config_remediation(state: ConfigState) -> RemediationCode {
    match state {
        ConfigState::Missing | ConfigState::Ready => RemediationCode::None,
        ConfigState::Invalid => RemediationCode::InspectGlobalConfig,
        ConfigState::FutureSchema => RemediationCode::UpgradeQiongli,
        ConfigState::Insecure => RemediationCode::RepairGlobalConfigPermissions,
        ConfigState::Busy => RemediationCode::RetryGlobalConfig,
        ConfigState::RecoveryRequired => RemediationCode::RecoverGlobalConfig,
        ConfigState::WriteUnsupported => RemediationCode::UseSupportedPlatform,
    }
}

fn integration_overall(
    source: StatusCode,
    marketplace: StatusCode,
    direct_package: Option<StatusCode>,
    registration: StatusCode,
    full_mcp: StatusCode,
) -> StatusCode {
    let states = [
        source,
        marketplace,
        direct_package.unwrap_or(StatusCode::Ready),
        registration,
        full_mcp,
    ];
    for priority in [
        StatusCode::RecoveryRequired,
        StatusCode::Conflict,
        StatusCode::Drifted,
        StatusCode::Invalid,
        StatusCode::Busy,
        StatusCode::Unavailable,
    ] {
        if states.contains(&priority) {
            return priority;
        }
    }
    if states.iter().all(|state| *state == StatusCode::Ready) {
        StatusCode::Ready
    } else {
        StatusCode::Missing
    }
}

const fn integration_is_blocking(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::RecoveryRequired
            | StatusCode::Conflict
            | StatusCode::Drifted
            | StatusCode::Invalid
            | StatusCode::Insecure
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::SecretRef;

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    fn supported_integration_states(
        actions: [IntegrationActionView; 2],
    ) -> [(IntegrationActionView, ClientCompatibilityView); 2] {
        actions.map(|action| (action, ClientCompatibilityView::Supported))
    }

    struct FakeFolderPicker {
        path: Option<PathBuf>,
    }

    struct CancelAwareExecutor;

    #[test]
    fn cli_shell_resolution_distinguishes_active_missing_and_shadowed_commands() {
        let root = isolated_root("cli-shell-resolution");
        let managed = root.join("home/.local/bin/qiongli");
        let legacy = root.join("mise/shims/qiongli");
        fs::create_dir_all(managed.parent().unwrap()).unwrap();
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&managed, b"managed").unwrap();
        fs::write(&legacy, b"legacy").unwrap();

        assert_eq!(
            classify_cli_shell_resolution(
                &managed,
                &format!(
                    "shell startup notice\n__QIONGLI_COMMAND__={}\n",
                    managed.display()
                ),
            ),
            (CliPathState::Active, "qiongli-cli-shell-command-active")
        );
        assert_eq!(
            classify_cli_shell_resolution(&managed, "shell startup notice\n__QIONGLI_COMMAND__=\n",),
            (
                CliPathState::NotConfigured,
                "qiongli-cli-shell-command-missing"
            )
        );
        assert_eq!(
            classify_cli_shell_resolution(
                &managed,
                &format!("__QIONGLI_COMMAND__={}\n", legacy.display()),
            ),
            (CliPathState::Shadowed, "qiongli-cli-shell-command-shadowed")
        );
        assert_eq!(
            classify_cli_shell_resolution(
                &managed,
                &format!("untrusted startup output: {}\n", managed.display()),
            ),
            (
                CliPathState::NotConfigured,
                "qiongli-cli-shell-command-missing"
            ),
            "unmarked shell startup output must not be mistaken for command -v evidence"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cli_shell_version_mismatch_is_never_collapsed_into_not_observable() {
        assert_eq!(
            classify_cli_shell_version(&format!("qiongli {}\n", env!("CARGO_PKG_VERSION"))),
            (CliPathState::Active, "qiongli-cli-shell-command-active")
        );
        assert_eq!(
            classify_cli_shell_version("qiongli 1.19.0-beta.1\n"),
            (
                CliPathState::VersionMismatch,
                "qiongli-cli-shell-version-mismatch"
            )
        );
    }

    #[test]
    fn host_probe_parsers_require_current_qiongli_next_activation() {
        let root = isolated_root("host-probe-json");
        let codex_source = root.join("home/.qiongli/plugins/codex/qiongli-next");
        let claude_cache =
            root.join("home/.claude/plugins/cache/qiongli-local/qiongli-next/2.0.0-alpha.3");
        fs::create_dir_all(&codex_source).unwrap();
        fs::create_dir_all(&claude_cache).unwrap();

        let codex = serde_json::json!({
            "installed": [{
                "pluginId": "qiongli-next@personal",
                "version": "2.0.0-alpha.3",
                "installed": true,
                "enabled": true,
                "source": { "source": "local", "path": codex_source }
            }]
        })
        .to_string();
        assert_eq!(
            codex_plugin_state(&codex, "2.0.0-alpha.3", &codex_source),
            Ok(HostProbeState::Observed)
        );
        assert_eq!(
            codex_plugin_state(&codex, "2.0.0-alpha.2", &codex_source),
            Ok(HostProbeState::CacheDrift)
        );
        assert_eq!(
            codex_mcp_state(r#"[{"name":"qiongli-next","enabled":true}]"#),
            Ok(HostProbeState::Observed)
        );
        assert_eq!(
            codex_mcp_state("not-json"),
            Err("codex-mcp-inventory-json-invalid")
        );

        let claude = serde_json::json!([{
            "id": "qiongli-next@qiongli-local",
            "version": "2.0.0-alpha.3",
            "enabled": true,
            "scope": "user",
            "installPath": claude_cache
        }])
        .to_string();
        assert_eq!(
            claude_plugin_state(&claude, "2.0.0-alpha.3", &claude_cache),
            Ok(HostProbeState::Observed)
        );
        assert_eq!(
            claude_plugin_state(
                &claude.replace("\"user\"", "\"project\""),
                "2.0.0-alpha.3",
                &claude_cache
            ),
            Ok(HostProbeState::CacheDrift)
        );
        assert_eq!(
            claude_component_state(
                "Component inventory\n  Skills (1) qiongli-workflow\n  Agents (0)\n  MCP servers (1) qiongli-next (tool schemas resolved at runtime; not counted)\n"
            ),
            Ok(HostProbeState::Observed)
        );
        assert_eq!(
            claude_component_state(
                "Component inventory\n  Skills (0)\n  MCP servers (1) qiongli-next\n"
            ),
            Ok(HostProbeState::HostActionRequired)
        );
        assert_eq!(
            claude_component_state("unstructured output"),
            Err("claude-plugin-details-invalid")
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn host_plugin_plans_bind_fixed_argv_state_and_target_order() {
        let root = isolated_root("host-plugin-plan");
        let home = root.join("home");
        let codex = root.join("bin/codex");
        let claude = root.join("bin/claude");
        fs::create_dir_all(codex.parent().unwrap()).unwrap();
        fs::write(&codex, b"codex").unwrap();
        fs::write(&claude, b"claude").unwrap();
        let version = crate::command::DetectedClientVersion {
            major: 2,
            minor: 1,
            patch: 222,
        };
        let missing = HostIntegrationObservation {
            activation: HostProbeState::HostActionRequired,
            mcp_attachment: HostProbeState::HostActionRequired,
        };
        let ready = HostIntegrationObservation {
            activation: HostProbeState::Observed,
            mcp_attachment: HostProbeState::Observed,
        };
        assert_eq!(
            host_plugin_mode(PackagedProductInstallEffect::Repair, ready),
            Ok(Some(HostPluginMode::Repair))
        );
        assert_eq!(
            host_plugin_mode(PackagedProductInstallEffect::AlreadyCurrent, ready),
            Ok(None)
        );

        let codex_plan = build_host_plugin_plan(
            codex,
            &home,
            &home.join(".codex"),
            ClientActivationTarget::Codex,
            Some(HostPluginMode::Install),
            version,
            missing,
        )
        .unwrap();
        assert_eq!(
            codex_plan.commands[0].arguments,
            ["plugin", "add", "--json", "qiongli-next@personal"].map(OsString::from)
        );

        let claude_plan = build_host_plugin_plan(
            claude,
            &home,
            &home.join(".claude"),
            ClientActivationTarget::ClaudeCode,
            Some(HostPluginMode::Repair),
            version,
            missing,
        )
        .unwrap();
        assert_eq!(claude_plan.commands.len(), 3);
        assert_eq!(
            claude_plan.commands[0].arguments,
            [
                OsString::from("plugin"),
                OsString::from("marketplace"),
                OsString::from("add"),
                home.join(".qiongli/plugins/claude-code/qiongli-local")
                    .into_os_string(),
                OsString::from("--scope"),
                OsString::from("user"),
            ]
        );
        assert_ne!(
            codex_plan.plan_digest_sha256,
            claude_plan.plan_digest_sha256
        );
        let approval_target =
            integration_host_plan_display_targets(&[codex_plan.clone(), claude_plan.clone()]);
        let approval_detail = approval_target.expose();
        assert!(approval_detail.contains("codex install · scope personal"));
        assert!(approval_detail.contains("claude repair · scope user"));
        assert!(approval_detail.contains(env!("CARGO_PKG_VERSION")));

        let changed_state = build_host_plugin_plan(
            codex_plan.executable.clone(),
            &home,
            &home.join(".codex"),
            ClientActivationTarget::Codex,
            Some(HostPluginMode::Install),
            version,
            HostIntegrationObservation {
                activation: HostProbeState::Observed,
                mcp_attachment: HostProbeState::Observed,
            },
        )
        .unwrap();
        assert_ne!(
            codex_plan.plan_digest_sha256,
            changed_state.plan_digest_sha256
        );
        let changed_mode = build_host_plugin_plan(
            codex_plan.executable.clone(),
            &home,
            &home.join(".codex"),
            ClientActivationTarget::Codex,
            Some(HostPluginMode::Repair),
            version,
            missing,
        )
        .unwrap();
        let changed_root = build_host_plugin_plan(
            codex_plan.executable.clone(),
            &home,
            &home.join("alternate-codex-home"),
            ClientActivationTarget::Codex,
            Some(HostPluginMode::Install),
            version,
            missing,
        )
        .unwrap();
        assert_ne!(
            codex_plan.plan_digest_sha256,
            changed_mode.plan_digest_sha256
        );
        assert_ne!(
            codex_plan.plan_digest_sha256,
            changed_root.plan_digest_sha256
        );
        assert_eq!(
            selected_activation_targets(IntegrationSelection::ALL).unwrap(),
            [
                ClientActivationTarget::Codex,
                ClientActivationTarget::ClaudeCode
            ]
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn host_plugin_discovery_resolves_mise_dispatch_shims_to_the_exact_client() {
        let root = isolated_root("host-plugin-mise-shim");
        let home = root.join("home");
        let mise = home.join(".local/bin/mise");
        let shim = home.join(".local/share/mise/shims/claude");
        let installed = home.join(".local/share/mise/installs/claude-code/2.1.222/claude");
        let latest = home.join(".local/share/mise/installs/claude-code/latest");
        fs::create_dir_all(mise.parent().unwrap()).unwrap();
        fs::create_dir_all(shim.parent().unwrap()).unwrap();
        fs::create_dir_all(installed.parent().unwrap()).unwrap();
        fs::write(&mise, b"mise").unwrap();
        fs::write(&installed, b"claude").unwrap();
        std::os::unix::fs::symlink(&mise, &shim).unwrap();
        std::os::unix::fs::symlink(installed.parent().unwrap(), &latest).unwrap();

        assert_eq!(
            resolve_host_plugin_executable(&home, "claude", &shim).unwrap(),
            fs::canonicalize(installed).unwrap()
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn host_plugin_batch_clears_old_evidence_and_stops_after_first_failure() {
        use std::os::unix::fs::PermissionsExt;

        let root = isolated_root("host-plugin-partial-failure");
        let home = root.join("home");
        let bin = root.join("bin");
        let codex = bin.join("codex");
        let claude = bin.join("claude");
        let codex_log = root.join("codex.log");
        let claude_log = root.join("claude.log");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&bin).unwrap();
        fs::write(
            &codex,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 7\n",
                codex_log.display()
            ),
        )
        .unwrap();
        fs::write(
            &claude,
            format!("#!/bin/sh\nprintf reached > '{}'\n", claude_log.display()),
        )
        .unwrap();
        fs::set_permissions(&codex, fs::Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(&claude, fs::Permissions::from_mode(0o700)).unwrap();
        let version = crate::command::DetectedClientVersion {
            major: 2,
            minor: 1,
            patch: 222,
        };
        let observation = HostIntegrationObservation {
            activation: HostProbeState::HostActionRequired,
            mcp_attachment: HostProbeState::HostActionRequired,
        };
        let plans = [
            build_host_plugin_plan(
                codex,
                &home,
                &home.join(".codex"),
                ClientActivationTarget::Codex,
                Some(HostPluginMode::Install),
                version,
                observation,
            )
            .unwrap(),
            build_host_plugin_plan(
                claude,
                &home,
                &home.join(".claude"),
                ClientActivationTarget::ClaudeCode,
                Some(HostPluginMode::Install),
                version,
                observation,
            )
            .unwrap(),
        ];
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let mut observations = [HostIntegrationObservation {
            activation: HostProbeState::Observed,
            mcp_attachment: HostProbeState::Observed,
        }; 2];

        assert_eq!(
            execute_host_plugin_plans(&environment, &plans, &mut observations),
            Err("host-command-nonzero-exit")
        );
        assert_eq!(
            observations[0],
            HostIntegrationObservation {
                activation: HostProbeState::CommandFailed,
                mcp_attachment: HostProbeState::CommandFailed,
            }
        );
        assert_eq!(observations[1], HostIntegrationObservation::default());
        assert_eq!(
            fs::read_to_string(codex_log).unwrap(),
            "plugin\nadd\n--json\nqiongli-next@personal\n"
        );
        assert!(!claude_log.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn host_cache_receipts_fail_closed_when_missing_or_drifted() {
        assert!(verified_bundle_receipts_match(
            Some("managed-receipt"),
            Some("managed-receipt")
        ));
        assert!(!verified_bundle_receipts_match(
            Some("managed-receipt"),
            Some("stale-receipt")
        ));
        assert!(!verified_bundle_receipts_match(
            Some("managed-receipt"),
            None
        ));
        assert!(!verified_bundle_receipts_match(None, None));
    }

    #[test]
    fn host_readiness_state_matrix_fails_closed_until_both_probes_are_observed() {
        assert_eq!(
            integration_mcp_attachment(
                ClientDiscoveryState::Detected,
                StatusCode::Ready,
                StatusCode::Ready,
            ),
            (
                StatusCode::Attention,
                IntegrationObservationView::NotObservable,
            )
        );
        assert_eq!(
            integration_mcp_attachment(
                ClientDiscoveryState::Detected,
                StatusCode::Ready,
                StatusCode::Missing,
            ),
            (StatusCode::Missing, IntegrationObservationView::Missing)
        );

        let environment = CommandEnvironment::with_paths(None, None, None);
        assert_eq!(
            probe_codex_host(&environment),
            HostIntegrationObservation {
                activation: HostProbeState::ProbeUnavailable,
                mcp_attachment: HostProbeState::ProbeUnavailable,
            }
        );

        let (mut integration, _) = unavailable_integration(
            IntegrationTarget::Codex,
            StatusCode::Unavailable,
            RemediationCode::InspectCodexLocal,
        );
        integration.client = StatusCode::Ready;
        integration.source = StatusCode::Ready;
        integration.skills = StatusCode::Ready;
        integration.marketplace = StatusCode::Ready;
        integration.direct_package = None;
        integration.registration = StatusCode::Ready;
        integration.activation_status = StatusCode::Attention;
        integration.activation_observation = IntegrationObservationView::NotObservable;
        integration.mcp_attachment = StatusCode::Attention;
        integration.mcp_attachment_observation = IntegrationObservationView::NotObservable;

        apply_host_observation(&mut integration, HostIntegrationObservation::default());
        assert_eq!(integration.overall, StatusCode::Attention);

        apply_host_observation(
            &mut integration,
            HostIntegrationObservation {
                activation: HostProbeState::CacheDrift,
                mcp_attachment: HostProbeState::CacheDrift,
            },
        );
        assert_eq!(integration.activation_status, StatusCode::Drifted);
        assert_eq!(integration.mcp_attachment, StatusCode::Drifted);
        assert_eq!(integration.next_action, IntegrationActionView::RepairReady);
        assert_eq!(integration.evidence_code, "qiongli-plugin-cache-drift");
        assert_eq!(integration.overall, StatusCode::Drifted);

        apply_host_observation(
            &mut integration,
            HostIntegrationObservation {
                activation: HostProbeState::HostActionRequired,
                mcp_attachment: HostProbeState::HostActionRequired,
            },
        );
        assert_eq!(integration.activation_status, StatusCode::Attention);
        assert_eq!(
            integration.activation_observation,
            IntegrationObservationView::ClientActionRequired
        );
        assert_eq!(integration.overall, StatusCode::Attention);

        apply_host_observation(
            &mut integration,
            HostIntegrationObservation {
                activation: HostProbeState::ProbeUnavailable,
                mcp_attachment: HostProbeState::ProbeFailed,
            },
        );
        assert_eq!(integration.activation_status, StatusCode::Unavailable);
        assert_eq!(
            integration.activation_observation,
            IntegrationObservationView::ProbeUnavailable
        );
        assert_eq!(
            integration.mcp_attachment_observation,
            IntegrationObservationView::ProbeFailed
        );
        assert_eq!(integration.overall, StatusCode::Unavailable);

        apply_host_observation(
            &mut integration,
            HostIntegrationObservation {
                activation: HostProbeState::Observed,
                mcp_attachment: HostProbeState::Observed,
            },
        );
        assert_eq!(integration.activation_status, StatusCode::Ready);
        assert_eq!(integration.mcp_attachment, StatusCode::Ready);
        assert_eq!(integration.overall, StatusCode::Ready);
    }

    #[cfg(unix)]
    #[test]
    fn host_command_failures_are_stable_and_bounded() {
        let root = isolated_root("bounded-host-command");
        fs::create_dir_all(&root).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(root.clone()), None);

        assert_eq!(
            bounded_host_command_with_timeout(
                &environment,
                Path::new("/bin/sh"),
                &["-c", "exit 7"],
                Duration::from_secs(1),
            ),
            Err(HostCommandFailure::NonZeroExit)
        );
        assert_eq!(
            bounded_host_command_with_timeout(
                &environment,
                Path::new("/bin/sh"),
                &["-c", "sleep 1"],
                Duration::from_millis(20),
            ),
            Err(HostCommandFailure::Timeout)
        );
        assert_eq!(
            bounded_host_command_with_timeout(
                &environment,
                Path::new("/definitely/missing/qiongli-host"),
                &[],
                Duration::from_secs(1),
            ),
            Err(HostCommandFailure::Spawn)
        );
        assert_eq!(
            bounded_host_command_with_timeout(
                &environment,
                Path::new("/bin/sh"),
                &["-c", "head -c 524289 /dev/zero"],
                Duration::from_secs(2),
            ),
            Err(HostCommandFailure::OutputTooLarge)
        );
        assert_eq!(
            bounded_host_command_with_timeout(
                &environment,
                Path::new("/usr/bin/printf"),
                &["\\377"],
                Duration::from_secs(1),
            ),
            Err(HostCommandFailure::InvalidUtf8)
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn integration_previews_name_the_managed_source_and_registry_destinations() {
        let display = integration_display_targets(&[
            ClientActivationTarget::Codex,
            ClientActivationTarget::ClaudeCode,
        ]);
        let display = display.expose();

        assert!(display.contains(CODEX_PLUGIN_SOURCE_SYMBOLIC_PATH));
        assert!(display.contains(CODEX_MARKETPLACE_SYMBOLIC_PATH));
        assert!(display.contains(CLAUDE_PLUGIN_SOURCE_SYMBOLIC_PATH));
        assert!(display.contains(CLAUDE_MARKETPLACE_SYMBOLIC_PATH));
    }

    fn json_string_value_containing<'a>(
        value: &'a serde_json::Value,
        needle: &str,
    ) -> Option<&'a str> {
        match value {
            serde_json::Value::String(observed) => {
                observed.contains(needle).then_some(observed.as_str())
            }
            serde_json::Value::Array(values) => values
                .iter()
                .find_map(|value| json_string_value_containing(value, needle)),
            serde_json::Value::Object(values) => values
                .values()
                .find_map(|value| json_string_value_containing(value, needle)),
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
                None
            }
        }
    }

    #[derive(Default)]
    struct TestSecretStore {
        values: Mutex<BTreeMap<String, Vec<u8>>>,
    }

    #[derive(Default)]
    struct ResolveCountingSecretStore {
        calls: AtomicU64,
    }

    impl SecretStore for ResolveCountingSecretStore {
        fn status(&self) -> SecretStoreStatus {
            SecretStoreStatus::Available
        }

        fn resolve(
            &self,
            _secret_ref: &SecretRef,
        ) -> Result<SecretValue, qiongli_config::SecretStoreError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(qiongli_config::SecretStoreError::NotFound)
        }
    }

    impl SecretStore for TestSecretStore {
        fn status(&self) -> SecretStoreStatus {
            SecretStoreStatus::Available
        }

        fn resolve(
            &self,
            secret_ref: &SecretRef,
        ) -> Result<SecretValue, qiongli_config::SecretStoreError> {
            let values = self
                .values
                .lock()
                .map_err(|_| qiongli_config::SecretStoreError::Unavailable)?;
            let value = values
                .get(secret_ref.storage_key())
                .cloned()
                .ok_or(qiongli_config::SecretStoreError::NotFound)?;
            SecretValue::new(value).map_err(|_| qiongli_config::SecretStoreError::PersistenceFailed)
        }

        fn store(
            &self,
            secret_ref: &SecretRef,
            value: &SecretValue,
        ) -> Result<(), qiongli_config::SecretStoreError> {
            self.values
                .lock()
                .map_err(|_| qiongli_config::SecretStoreError::Unavailable)?
                .insert(
                    secret_ref.storage_key().to_owned(),
                    value.as_bytes().to_vec(),
                );
            Ok(())
        }

        fn remove(&self, secret_ref: &SecretRef) -> Result<(), qiongli_config::SecretStoreError> {
            self.values
                .lock()
                .map_err(|_| qiongli_config::SecretStoreError::Unavailable)?
                .remove(secret_ref.storage_key())
                .map(|_| ())
                .ok_or(qiongli_config::SecretStoreError::NotFound)
        }
    }

    impl McpSelfTestExecutor for CancelAwareExecutor {
        fn run(&self, input: McpSelfTestInput, cancelled: Arc<AtomicBool>) -> McpSelfTestView {
            while !cancelled.load(Ordering::Acquire) {
                thread::yield_now();
            }
            terminal_mcp_self_test(McpSelfTestState::Cancelled, input.counts)
        }
    }

    impl FolderPicker for FakeFolderPicker {
        fn pick_folder(&mut self) -> Option<PathBuf> {
            self.path.take()
        }
    }

    #[test]
    fn startup_validation_is_repeatable_without_a_config_home() {
        let environment = CommandEnvironment::with_paths(None, None, None);
        let content = crate::embedded_content().expect("embedded content must load");

        assert_eq!(validate_desktop_startup(&environment, &content), Ok(()));
        assert_eq!(validate_desktop_startup(&environment, &content), Ok(()));
    }

    #[test]
    fn app_api_contract_fixture_is_deterministic_and_covers_every_event_variant() {
        let first = app_api_contract_fixture_json().expect("contract fixture must serialize");
        let second = app_api_contract_fixture_json().expect("contract fixture must be repeatable");
        assert_eq!(first, second);

        let fixture: serde_json::Value =
            serde_json::from_str(&first).expect("contract fixture must be JSON");
        assert_eq!(
            fixture["schemaVersion"],
            json!(crate::desktop_api::APP_API_SCHEMA_VERSION)
        );
        assert_eq!(
            fixture["snapshot"]["schemaVersion"],
            json!(crate::desktop_api::APP_API_SCHEMA_VERSION)
        );
        let event_types = fixture["events"]
            .as_array()
            .expect("contract events must be an array")
            .iter()
            .map(|event| {
                event["type"]
                    .as_str()
                    .expect("every contract event must be tagged")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            event_types,
            [
                "snapshot",
                "preview",
                "skills-destination-selected",
                "content-customization",
                "capture-inbox",
                "capture-coverage",
                "artifact-changes",
                "academic-graph",
                "academic-graph-portfolio",
                "academic-graph-query",
                "academic-graph-path",
                "academic-graph-artifact-opened",
                "project-artifact-read",
                "capture-read",
                "project-directory-selected",
                "project-migration-completed",
                "capture-file-selected",
                "capture-intake-preview",
                "capture-consolidation-preview",
                "capture-deliveries",
                "capture-delivery-inspected",
                "capture-delivery-updated",
                "capture-delivery-acknowledgement-preview",
                "capture-assignments",
                "capture-assignment-inspected",
                "capture-assignment-preview",
                "capture-resolutions",
                "capture-resolution-inspected",
                "capture-resolution-plan",
                "capture-resolution-preview",
                "portfolio-status",
                "portfolio-query",
                "semantic-timeline",
                "portfolio-doctor",
                "portfolio-maintenance-preview",
                "continuity-operation-progress",
                "portfolio-maintenance-completed",
                "update-changed",
                "mcp-self-test-updated",
                "orchestration-loaded",
                "orchestration-run-updated",
                "completed",
                "capture-operation-completed",
                "cancelled",
                "validation-failed",
                "failed",
            ]
        );

        let mut host_path_canaries = vec![
            (
                "current directory",
                std::env::current_dir().expect("test directory must resolve"),
            ),
            (
                "current executable",
                std::env::current_exe().expect("test executable must resolve"),
            ),
        ];
        for (label, variable) in [
            ("home directory", "HOME"),
            ("Windows home directory", "USERPROFILE"),
            ("Qiongli config directory", "QIONGLI_CONFIG_HOME"),
            ("Codex config directory", "CODEX_HOME"),
            ("Claude config directory", "CLAUDE_CONFIG_DIR"),
            ("XDG config directory", "XDG_CONFIG_HOME"),
            ("Windows roaming config directory", "APPDATA"),
            ("Windows local config directory", "LOCALAPPDATA"),
        ] {
            let Some(path) = std::env::var_os(variable).map(PathBuf::from) else {
                continue;
            };
            if path.is_absolute() && path.components().count() > 1 {
                host_path_canaries.push((label, path));
            }
        }
        for (label, path) in host_path_canaries {
            let path = path.to_string_lossy();
            assert!(
                json_string_value_containing(&fixture, path.as_ref()).is_none(),
                "contract fixture must not expose the test-process {label}"
            );
        }
        for forbidden_field in [
            "apiKey",
            "prompt",
            "transcript",
            "providerResponse",
            "sessionId",
            "projectRoot",
            "rootPath",
        ] {
            assert!(
                !first.contains(&format!("\"{forbidden_field}\"")),
                "App API v5 must not expose the private field {forbidden_field}"
            );
        }
    }

    #[test]
    fn app_api_contract_path_canary_search_handles_escaped_windows_values() {
        let fixture: serde_json::Value = serde_json::from_str(
            r#"{"nested":[{"value":"prefix C:\\Users\\Researcher\\AppData suffix"}]}"#,
        )
        .expect("escaped Windows fixture must parse");

        assert_eq!(
            json_string_value_containing(&fixture, r"C:\Users\Researcher\AppData"),
            Some(r"prefix C:\Users\Researcher\AppData suffix")
        );
    }

    #[test]
    fn read_only_snapshot_is_valid_and_creates_no_state() {
        let root = isolated_root("snapshot");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(
            Some(OsString::from(&config)),
            Some(home.clone()),
            Some(root.join("claude-config")),
        );
        let content = crate::embedded_content().unwrap();

        let snapshot = build_snapshot(
            &environment,
            &content,
            &qiongli_config::UnavailableSecretStore,
        );

        assert_eq!(snapshot.validate(), Ok(()));
        assert_eq!(snapshot.mcp.public_tool_count, LITE_PUBLIC_TOOL_NAMES.len());
        assert_eq!(
            snapshot.zotero.state,
            ZoteroIntegrationStateView::NotObserved
        );
        assert_eq!(
            snapshot.zotero.observation,
            ZoteroObservationView::NotObserved
        );
        assert!(!snapshot.zotero.connector_available);
        assert!(!snapshot.zotero.companion_available);
        assert!(snapshot.zotero.fallback_import_available);
        assert_eq!(
            snapshot.zotero.available_companion_version.as_deref(),
            Some("0.3.0")
        );
        assert!(snapshot.zotero.can_prepare_install);
        assert!(!config.exists());
        assert!(!home.join(".qiongli").exists());
        assert!(!home.join(".agents").exists());
        assert!(!home.join(".claude").exists());
        assert!(!root.join("claude-config").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn zotero_companion_handoff_is_approval_gated_and_stays_out_of_the_profile() {
        let root = isolated_root("zotero-companion-handoff");
        let home = root.join("home");
        let configured = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(
            Some(OsString::from(&configured)),
            Some(home.clone()),
            None,
        );
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());

        let DesktopEvent::PreviewReady(preview) =
            service.execute(DesktopIntent::PreviewZoteroCompanionStage)
        else {
            panic!("the embedded Companion must produce a confirmation preview");
        };
        assert_eq!(preview.kind, OperationKind::ZoteroCompanionStage);
        assert_eq!(
            preview.approvals_required,
            [OperationApproval::FilesystemWrite]
        );
        assert!(preview.can_confirm);
        assert!(preview.validate());
        assert!(!configured.exists(), "preview must remain read-only");
        assert!(!home.join(".zotero").exists());
        assert!(!home.join("Zotero").exists());
        assert!(!home.join("Library/Application Support/Zotero").exists());

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "zotero-companion-installation-prepared",
            }
        );
        let staged_xpi = service
            .staged_zotero_companion_path()
            .expect("confirmed handoff must resolve its receipt-owned XPI");
        assert!(staged_xpi.starts_with(configured.join("v2/zotero/companion")));
        assert!(staged_xpi.is_file());
        assert!(!home.join(".zotero").exists());
        assert!(!home.join("Zotero").exists());
        assert!(!home.join("Library/Application Support/Zotero").exists());
        assert!(service.snapshot().zotero.installation_prepared);
        assert!(service.snapshot().zotero.can_reveal);

        assert_eq!(
            service.execute(DesktopIntent::PreviewZoteroCompanionStage),
            DesktopEvent::Completed {
                code: "zotero-companion-already-prepared",
            }
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn zotero_version_support_matches_the_bundled_manifest_range() {
        assert!(!zotero_version_is_incompatible(None));
        assert!(zotero_version_is_incompatible(Some("7.0.15")));
        assert!(!zotero_version_is_incompatible(Some("8.0.0")));
        assert!(!zotero_version_is_incompatible(Some("8.9.1")));
        assert!(!zotero_version_is_incompatible(Some("9.0.12")));
        assert!(zotero_version_is_incompatible(Some("9.1.0")));
        assert!(zotero_version_is_incompatible(Some("10.0.0")));
    }

    #[test]
    fn legacy_companion_endpoint_is_presented_as_a_bundled_update() {
        let mut view = zotero_integration_snapshot();
        let status = qiongli_runtime::zotero::companion::ZoteroStatus {
            status: "ok".to_owned(),
            error_code: None,
            connector: qiongli_runtime::zotero::companion::ProbeStatus {
                available: true,
                status: Some(200),
            },
            companion: qiongli_runtime::zotero::companion::CompanionProbeStatus {
                available: true,
                status: Some(200),
                version: Some("0.2.2".to_owned()),
                endpoint_version: Some("1".to_owned()),
            },
            fallback_import_files: qiongli_runtime::zotero::companion::ImportFileFallback {
                available: true,
                formats: vec![
                    "references.json".to_owned(),
                    "references.ris".to_owned(),
                    "bibliography.bib".to_owned(),
                    "zotero-import-report.md".to_owned(),
                ],
            },
        };

        apply_zotero_live_observation(&mut view, &status);

        assert_eq!(
            view.state,
            ZoteroIntegrationStateView::CompanionUpdateAvailable
        );
        assert_eq!(view.reason_code, "zotero-companion-update-required");
        assert!(view.can_prepare_install);
    }

    #[test]
    fn zotero_acceptance_state_matrix_never_treats_staged_or_stale_evidence_as_ready() {
        let observed = |status: &str,
                        connector_available: bool,
                        companion_available: bool,
                        companion_version: Option<&str>,
                        endpoint_version: Option<&str>| {
            qiongli_runtime::zotero::companion::ZoteroStatus {
                status: status.to_owned(),
                error_code: None,
                connector: qiongli_runtime::zotero::companion::ProbeStatus {
                    available: connector_available,
                    status: connector_available.then_some(200),
                },
                companion: qiongli_runtime::zotero::companion::CompanionProbeStatus {
                    available: companion_available,
                    status: companion_available.then_some(200),
                    version: companion_version.map(str::to_owned),
                    endpoint_version: endpoint_version.map(str::to_owned),
                },
                fallback_import_files: qiongli_runtime::zotero::companion::ImportFileFallback {
                    available: true,
                    formats: vec![
                        "references.json".to_owned(),
                        "references.ris".to_owned(),
                        "bibliography.bib".to_owned(),
                        "zotero-import-report.md".to_owned(),
                    ],
                },
            }
        };

        let mut not_running = zotero_integration_snapshot();
        not_running.can_open_zotero = true;
        apply_zotero_live_observation(
            &mut not_running,
            &observed("fallback_only", false, false, None, None),
        );
        assert_eq!(
            not_running.state,
            ZoteroIntegrationStateView::ZoteroNotRunning
        );

        let mut missing = zotero_integration_snapshot();
        apply_zotero_live_observation(
            &mut missing,
            &observed("companion_missing", true, false, None, None),
        );
        assert_eq!(missing.state, ZoteroIntegrationStateView::CompanionMissing);

        let mut incompatible = zotero_integration_snapshot();
        apply_zotero_live_observation(
            &mut incompatible,
            &observed("ok", true, true, Some("0.3.0"), Some("1")),
        );
        assert_eq!(
            incompatible.state,
            ZoteroIntegrationStateView::CompanionIncompatible
        );

        let mut update = zotero_integration_snapshot();
        apply_zotero_live_observation(
            &mut update,
            &observed("ok", true, true, Some("0.2.2"), Some("1")),
        );
        assert_eq!(
            update.state,
            ZoteroIntegrationStateView::CompanionUpdateAvailable
        );

        let mut staged = zotero_integration_snapshot();
        staged.installation_prepared = true;
        staged.can_reveal = true;
        apply_zotero_live_observation(
            &mut staged,
            &observed("companion_missing", true, false, None, None),
        );
        assert_eq!(staged.state, ZoteroIntegrationStateView::RestartRequired);

        let mut ready = zotero_integration_snapshot();
        apply_zotero_live_observation(
            &mut ready,
            &observed("ok", true, true, Some("0.3.0"), Some("2")),
        );
        assert_eq!(ready.state, ZoteroIntegrationStateView::Ready);

        apply_zotero_live_observation(
            &mut ready,
            &observed("companion_missing", true, false, None, None),
        );
        assert_eq!(
            ready.state,
            ZoteroIntegrationStateView::CompanionMissing,
            "removal must clear a previously ready live observation"
        );

        let mut disabled = zotero_integration_snapshot();
        apply_zotero_live_observation(
            &mut disabled,
            &observed("disabled", false, false, None, None),
        );
        assert_eq!(disabled.state, ZoteroIntegrationStateView::Disabled);
        assert!(disabled.fallback_import_available);
    }

    #[test]
    fn legacy_content_is_reported_as_migration_input_not_current_installation() {
        let root = isolated_root("legacy-migration-snapshot");
        let home = root.join("home");
        let plugin = home.join(".agents/plugins/qiongli");
        fs::create_dir_all(plugin.join("skills/qiongli-workflow")).unwrap();
        fs::write(
            plugin.join(".qiongli-managed.json"),
            serde_json::to_vec(&json!({
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
        let environment = CommandEnvironment::with_paths(None, Some(home), None)
            .with_inventory_context(None, None, true, false);
        let content = crate::embedded_content().unwrap();

        let snapshot = build_snapshot(
            &environment,
            &content,
            &qiongli_config::UnavailableSecretStore,
        );
        assert_eq!(
            snapshot.legacy_migration.state,
            LegacyMigrationStateView::Available
        );
        assert_eq!(
            snapshot.legacy_migration.next_action,
            LegacyMigrationActionView::Start
        );
        assert_eq!(snapshot.integrations[0].installed_plugin_version, None);
        assert_eq!(snapshot.integrations[0].source, StatusCode::Missing);
        assert_eq!(
            snapshot.integrations[0].migration.state,
            IntegrationMigrationStateView::Available
        );

        let client_inventory = environment.client_inventory().unwrap();
        let inventory = discover_legacy_migration(&client_inventory);
        let created_at_unix = now_unix().unwrap();
        let plan = qiongli_platform::preview_legacy_migration(
            &inventory,
            qiongli_platform::LegacyMigrationPlanInput {
                plan_id: "migration-desktop-preview",
                product_version: "2.0.0-alpha.2",
                source_commit: "0123456789abcdef0123456789abcdef01234567",
                resource_pack_sha256:
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                created_at_unix,
                provider_resolutions: &[],
            },
        )
        .unwrap();
        let receipt = qiongli_platform::initial_legacy_migration_receipt_from_plan(&plan).unwrap();
        LegacyMigrationStore::for_inventory(&inventory)
            .unwrap()
            .persist_preview(&plan, &receipt)
            .unwrap();

        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        assert_eq!(
            service.snapshot().legacy_migration.state,
            LegacyMigrationStateView::PreviewReady
        );
        let DesktopEvent::PreviewReady(preview) =
            service.execute(DesktopIntent::PreviewLegacyMigrationNext)
        else {
            panic!("source build must return a blocked migration preview");
        };
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("source-build-read-only"));
        assert!(preview.validate());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn snapshot_exposes_detected_client_versions_without_dynamic_output() {
        let root = isolated_root("client-versions");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None)
            .with_inventory_context(None, None, true, true)
            .with_client_versions(
                Some(crate::command::DetectedClientVersion {
                    major: 0,
                    minor: 144,
                    patch: 4,
                }),
                Some(crate::command::DetectedClientVersion {
                    major: 2,
                    minor: 1,
                    patch: 209,
                }),
            );
        let content = crate::embedded_content().unwrap();

        let snapshot = build_snapshot(
            &environment,
            &content,
            &qiongli_config::UnavailableSecretStore,
        );

        assert_eq!(
            snapshot.integrations[0].client_version,
            Some(ClientVersionView {
                major: 0,
                minor: 144,
                patch: 4,
            })
        );
        assert_eq!(
            snapshot.integrations[1].client_version,
            Some(ClientVersionView {
                major: 2,
                minor: 1,
                patch: 209,
            })
        );
        assert_eq!(
            snapshot
                .integrations
                .map(|integration| integration.compatibility),
            [
                ClientCompatibilityView::Supported,
                ClientCompatibilityView::Supported,
            ]
        );
        assert_eq!(
            snapshot
                .integrations
                .map(|integration| integration.mcp_attachment_observation),
            [
                IntegrationObservationView::Missing,
                IntegrationObservationView::Missing,
            ],
            "plugin-source discovery must not be reused as Lite MCP attachment evidence"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn client_compatibility_policy_rejects_versions_below_the_accepted_floor() {
        let codex = crate::command::DetectedClientVersion {
            major: 0,
            minor: 144,
            patch: 0,
        };
        let claude = crate::command::DetectedClientVersion {
            major: 2,
            minor: 1,
            patch: 205,
        };

        assert_eq!(
            client_compatibility(
                ClientKind::Codex,
                ClientDiscoveryState::Detected,
                Some(codex),
            ),
            ClientCompatibilityView::Unsupported
        );
        assert_eq!(
            client_compatibility(
                ClientKind::ClaudeCode,
                ClientDiscoveryState::Detected,
                Some(claude),
            ),
            ClientCompatibilityView::Unsupported
        );
        assert_eq!(
            client_compatibility(ClientKind::Codex, ClientDiscoveryState::Detected, None),
            ClientCompatibilityView::NotEvaluated
        );

        let root = isolated_root("unsupported-client-projection");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None)
            .with_inventory_context(None, None, true, false)
            .with_client_versions(Some(codex), None);
        let content = crate::embedded_content().unwrap();
        let snapshot = build_snapshot(
            &environment,
            &content,
            &qiongli_config::UnavailableSecretStore,
        );
        assert_eq!(
            (
                snapshot.integrations[0].next_action,
                snapshot.integrations[0].overall,
                snapshot.integrations[0].evidence_code,
            ),
            (
                IntegrationActionView::UpgradeClient,
                StatusCode::Blocked,
                "client-version-below-supported-minimum",
            )
        );
        assert_eq!(snapshot.validate(), Ok(()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn production_previews_never_enable_apply_or_echo_private_input() {
        let environment = CommandEnvironment::default();
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        assert!(!service.snapshot().capabilities.apply);

        let event = service.execute(DesktopIntent::PreviewProviderPublicSetting {
            provider: ProviderKind::Crossref,
            public_email: qiongli_ui::PrivateText::new("private@example.org".to_owned()),
        });

        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("valid public setting should produce a preview");
        };
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("config-write-unavailable"));
        assert!(preview.plan_digest_sha256.is_none());
        assert!(preview.approvals_required.is_empty());
        assert!(!format!("{preview:?}").contains("private@example.org"));

        let wrong_token = if preview.token == OperationToken::new(99) {
            OperationToken::new(100)
        } else {
            OperationToken::new(99)
        };
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation { token: wrong_token }),
            DesktopEvent::Failed {
                code: "operation-token-invalid",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::CancelOperation {
                token: preview.token,
            }),
            DesktopEvent::Cancelled {
                code: "operation-preview-cancelled",
            }
        );
    }

    #[test]
    fn source_build_integration_preview_is_explicitly_read_only() {
        let root = isolated_root("source-build-integration-preview");
        let home = root.join("home");
        fs::create_dir_all(home.join(".codex")).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());

        let event = service.execute(DesktopIntent::PreviewIntegration {
            target: IntegrationTarget::Codex,
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("source builds must return a bounded read-only preview");
        };
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("source-build-read-only"));
        assert_eq!(
            preview
                .display_target
                .as_ref()
                .map(PrivateDisplayText::expose),
            Some(integration_display_target(IntegrationTarget::Codex).expose())
        );
        assert_ne!(
            preview.blocked_reason,
            Some("production-activation-session-unavailable")
        );
        assert!(preview.plan_digest_sha256.is_none());
        assert!(preview.approvals_required.is_empty());
        assert!(!root.join("home/.qiongli").exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_build_integration_verify_refreshes_observation_without_apply_authority() {
        let root = isolated_root("source-build-integration-verify");
        let home = root.join("home");
        fs::create_dir_all(home.join(".codex")).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());

        assert_eq!(
            service.execute(DesktopIntent::VerifyIntegrations {
                selection: IntegrationSelection {
                    codex: true,
                    claude_code: false,
                },
            }),
            DesktopEvent::Completed {
                code: "integration-inventory-refreshed-host-probed-read-only",
            }
        );
        assert!(!service.snapshot().capabilities.apply);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn only_recoverable_packaged_verification_states_become_reconciliation() {
        for code in [
            "packaged-product-source-invalid",
            "packaged-product-activation-invalid",
            "packaged-product-replace-required",
            "packaged-product-recovery-required",
        ] {
            assert!(
                integration_verification_needs_reconciliation(code),
                "{code}"
            );
        }
        for code in [
            "packaged-product-evidence-unavailable",
            "packaged-product-authority-unavailable",
            "integration-selection-required",
        ] {
            assert!(
                !integration_verification_needs_reconciliation(code),
                "{code}"
            );
        }
    }

    #[test]
    fn integration_installation_requires_a_selected_missing_target() {
        let actions = [
            IntegrationActionView::RepairReady,
            IntegrationActionView::InstallReady,
        ];

        assert_eq!(
            integration_install_required(
                supported_integration_states(actions),
                IntegrationSelection {
                    codex: true,
                    claude_code: true,
                },
            ),
            Ok(true)
        );
        assert_eq!(
            integration_install_required(
                supported_integration_states(actions),
                IntegrationSelection {
                    codex: true,
                    claude_code: false,
                },
            ),
            Err("integration-install-selection-invalid")
        );
        assert_eq!(
            integration_install_required(
                supported_integration_states([
                    IntegrationActionView::Current,
                    IntegrationActionView::Current,
                ]),
                IntegrationSelection {
                    codex: true,
                    claude_code: false,
                },
            ),
            Ok(false)
        );
        assert_eq!(
            integration_install_required(
                supported_integration_states(actions),
                IntegrationSelection {
                    codex: false,
                    claude_code: false,
                },
            ),
            Err("integration-selection-required")
        );
    }

    #[test]
    fn integration_reconciliation_is_scoped_to_selected_receipt_owned_targets() {
        let actions = [
            IntegrationActionView::InstallReady,
            IntegrationActionView::RepairReady,
        ];

        assert_eq!(
            integration_reconcile_required(
                supported_integration_states(actions),
                IntegrationSelection {
                    codex: true,
                    claude_code: false,
                },
            ),
            Err("integration-reconcile-selection-invalid")
        );
        assert_eq!(
            integration_reconcile_required(
                supported_integration_states(actions),
                IntegrationSelection {
                    codex: false,
                    claude_code: true,
                },
            ),
            Ok(true)
        );
        assert_eq!(
            integration_reconcile_required(
                supported_integration_states([
                    IntegrationActionView::Current,
                    IntegrationActionView::RepairReady,
                ]),
                IntegrationSelection {
                    codex: true,
                    claude_code: false,
                },
            ),
            Ok(false)
        );
        assert_eq!(
            integration_reconcile_required(
                supported_integration_states([
                    IntegrationActionView::RepairReady,
                    IntegrationActionView::RepairReady,
                ]),
                IntegrationSelection {
                    codex: false,
                    claude_code: false,
                },
            ),
            Err("integration-selection-required")
        );
    }

    #[test]
    fn unsupported_client_versions_cannot_bypass_install_or_repair_preconditions() {
        let install_states = [
            (
                IntegrationActionView::InstallReady,
                ClientCompatibilityView::Unsupported,
            ),
            (
                IntegrationActionView::Current,
                ClientCompatibilityView::Supported,
            ),
        ];
        assert_eq!(
            integration_install_required(
                install_states,
                IntegrationSelection {
                    codex: true,
                    claude_code: false,
                },
            ),
            Err("integration-client-version-unsupported")
        );

        let reconcile_states = [
            (
                IntegrationActionView::RepairReady,
                ClientCompatibilityView::Unsupported,
            ),
            (
                IntegrationActionView::Current,
                ClientCompatibilityView::Supported,
            ),
        ];
        assert_eq!(
            integration_reconcile_required(
                reconcile_states,
                IntegrationSelection {
                    codex: true,
                    claude_code: false,
                },
            ),
            Err("integration-client-version-unsupported")
        );
    }

    #[test]
    fn source_session_discovers_clients_without_install_authority() {
        assert_eq!(
            integration_discovery(true, StatusCode::Ready),
            IntegrationDiscoveryState::Managed
        );
        assert_eq!(
            integration_discovery(true, StatusCode::Drifted),
            IntegrationDiscoveryState::Drifted
        );
        assert_eq!(
            integration_discovery(true, StatusCode::Conflict),
            IntegrationDiscoveryState::Conflict
        );
        assert_eq!(
            integration_discovery(true, StatusCode::RecoveryRequired),
            IntegrationDiscoveryState::RecoveryRequired
        );
        let root = isolated_root("source-discovery");
        let home = root.join("home");
        let claude_config = home.join(".claude");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(None, Some(home.clone()), Some(claude_config.clone()));
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment.clone(), content, Vec::new());

        let missing = service.snapshot();
        assert!(missing.integrations.iter().all(|integration| {
            integration.discovery == IntegrationDiscoveryState::NotDiscovered
                && !integration.candidate_required
                && integration.ownership == IntegrationOwnershipView::NotInstalled
                && integration.next_action == IntegrationActionView::InspectOnly
                && integration.path_count >= 5
        }));

        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(&claude_config).unwrap();
        let discovered = service.snapshot();
        assert!(discovered.integrations.iter().all(|integration| {
            integration.discovery == IntegrationDiscoveryState::DiscoveredUnmanaged
                && integration.candidate_required
                && integration.registration == StatusCode::Missing
                && integration.next_action == IntegrationActionView::InstallReady
                && integration.paths[..integration.path_count]
                    .iter()
                    .all(Option::is_some)
        }));
        assert!(!discovered.capabilities.apply);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn full_mcp_self_test_uses_exact_registry_and_full_only_dispatch() {
        let root = isolated_root("mcp-self-test");
        let home = root.join("home");
        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(home.join(".claude")).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());

        let DesktopEvent::McpSelfTestUpdated(started) =
            service.execute(DesktopIntent::RunFullMcpSelfTest)
        else {
            panic!("self-test must return bounded progress");
        };
        assert_eq!(started.state, McpSelfTestState::Running);
        let completed = (0..1_000)
            .find_map(|_| {
                let DesktopEvent::McpSelfTestUpdated(view) =
                    service.execute(DesktopIntent::PollFullMcpSelfTest)
                else {
                    panic!("self-test poll must return typed progress");
                };
                if view.state == McpSelfTestState::Running {
                    thread::sleep(Duration::from_millis(1));
                    None
                } else {
                    Some(view)
                }
            })
            .expect("bounded self-test must complete");
        assert_eq!(completed.state, McpSelfTestState::Passed);
        assert!(completed.validate());
        assert_eq!(completed.public_tool_count, full_public_tool_count());
        assert!(
            completed.checks[..4]
                .iter()
                .all(|check| check.status == StatusCode::Ready)
        );
        assert_eq!(completed.discovered_client_count, 2);
        assert_eq!(completed.registered_client_count, 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn full_mcp_self_test_does_not_resolve_provider_credentials() {
        let root = isolated_root("mcp-self-test-no-credential-read");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let mut settings = GlobalSettings::default();
        settings.providers.openalex.enabled = true;
        settings.providers.openalex.api_key_ref =
            Some(SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap());
        store.replace(0, settings).unwrap();

        let credential_store = Arc::new(ResolveCountingSecretStore::default());
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        service.secret_store = credential_store.clone();

        let DesktopEvent::McpSelfTestUpdated(started) =
            service.execute(DesktopIntent::RunFullMcpSelfTest)
        else {
            panic!("self-test must start without resolving credentials");
        };
        assert_eq!(started.state, McpSelfTestState::Running);
        let completed = (0..1_000)
            .find_map(|_| {
                let DesktopEvent::McpSelfTestUpdated(view) =
                    service.execute(DesktopIntent::PollFullMcpSelfTest)
                else {
                    panic!("self-test poll must return typed progress");
                };
                if view.state == McpSelfTestState::Running {
                    thread::sleep(Duration::from_millis(1));
                    None
                } else {
                    Some(view)
                }
            })
            .expect("bounded self-test must complete");
        assert_eq!(completed.state, McpSelfTestState::Passed);
        assert_eq!(credential_store.calls.load(Ordering::Relaxed), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_snapshot_does_not_resolve_legacy_backend_credentials() {
        let root = isolated_root("desktop-snapshot-no-credential-read");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let mut settings = GlobalSettings::default();
        settings.agent_backends.openai.enabled = true;
        settings.agent_backends.openai.api_key_ref =
            Some(SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap());
        store.replace(0, settings).unwrap();

        let credential_store = Arc::new(ResolveCountingSecretStore::default());
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        service.secret_store = credential_store.clone();

        for _ in 0..2 {
            let backend = service.snapshot().config.openai_backend;
            assert_eq!(
                backend.readiness,
                AgentBackendReadinessView::CredentialUnverified
            );
            assert!(backend.test_available);
        }
        assert_eq!(credential_store.calls.load(Ordering::Relaxed), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn full_mcp_self_test_supports_cancel_and_fixed_timeout() {
        let root = isolated_root("mcp-self-test-timeout");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new(environment, content, Vec::new());
        service.mcp_self_test_executor = Arc::new(CancelAwareExecutor);

        let _ = service.execute(DesktopIntent::RunFullMcpSelfTest);
        let DesktopEvent::McpSelfTestUpdated(cancelled) =
            service.execute(DesktopIntent::CancelFullMcpSelfTest)
        else {
            panic!("cancellation must return a typed result");
        };
        assert_eq!(cancelled.state, McpSelfTestState::Cancelled);

        service.mcp_self_test_timeout = Duration::ZERO;
        let _ = service.execute(DesktopIntent::RunFullMcpSelfTest);
        let DesktopEvent::McpSelfTestUpdated(timed_out) =
            service.execute(DesktopIntent::PollFullMcpSelfTest)
        else {
            panic!("timeout must return a typed result");
        };
        assert_eq!(timed_out.state, McpSelfTestState::TimedOut);
        assert!(timed_out.validate());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provider_settings_preview_and_confirm_preserve_secret_references() {
        let root = isolated_root("global-settings");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let mut initial = GlobalSettings::default();
        initial.providers.openalex.api_key_ref =
            Some(SecretRef::parse("qsr1_0123456789abcdef0123456789abcdef").unwrap());
        let initial_outcome = store.replace(0, initial.clone()).unwrap();
        assert_eq!(initial_outcome.revision, 1);

        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        let event = service.execute(DesktopIntent::PreviewProviderSettingsPatch(
            ProviderSettingsPatch {
                expected_revision: 1,
                providers_enabled: [true, false, true, false, true],
                openalex_email: PublicSettingChange::Replace(qiongli_ui::PrivateText::new(
                    "openalex-private-canary@example.org".to_owned(),
                )),
                crossref_email: PublicSettingChange::Replace(qiongli_ui::PrivateText::new(
                    "crossref-private-canary@example.org".to_owned(),
                )),
            },
        ));
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("valid settings must produce a preview");
        };
        assert_eq!(preview.kind, OperationKind::ProviderSettings);
        assert_eq!(
            preview.approvals_required,
            vec![OperationApproval::ClientConfigChange]
        );
        assert!(!format!("{preview:?}").contains("private-canary"));

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "provider-settings-updated",
            }
        );
        let committed = store.load().unwrap();
        assert_eq!(committed.revision, 2);
        assert_eq!(
            committed.settings.default_profile,
            ProfileId::MarketplaceLite
        );
        assert_eq!(
            committed.settings.providers.openalex.api_key_ref,
            initial.providers.openalex.api_key_ref
        );
        assert_eq!(
            committed
                .settings
                .providers
                .openalex
                .email
                .as_ref()
                .map(EmailAddress::as_str),
            Some("openalex-private-canary@example.org")
        );
        assert_eq!(
            committed
                .settings
                .providers
                .crossref
                .email
                .as_ref()
                .map(EmailAddress::as_str),
            Some("crossref-private-canary@example.org")
        );
        let DesktopEvent::PreviewReady(global_preview) = service.execute(
            DesktopIntent::PreviewGlobalSettingsPatch(GlobalSettingsPatch {
                expected_revision: 2,
                default_profile: ProfileKind::Full,
            }),
        ) else {
            panic!("global defaults must preview independently");
        };
        assert!(matches!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: global_preview.token,
            }),
            DesktopEvent::Completed { .. }
        ));
        let separated = store.load().unwrap();
        assert_eq!(separated.settings.default_profile, ProfileId::Full);
        assert_eq!(
            separated.settings.providers.openalex.api_key_ref,
            initial.providers.openalex.api_key_ref
        );
        assert!(separated.settings.providers.openalex.enabled);
        assert!(separated.settings.providers.crossref.enabled);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provider_secret_save_replace_restart_and_remove_never_persist_raw_values() {
        let root = isolated_root("provider-secret-lifecycle");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let settings_store = config_store(&environment).unwrap();
        let mut initial = GlobalSettings::default();
        initial.providers.openalex.enabled = true;
        initial.providers.pubmed.enabled = true;
        settings_store.replace(0, initial).unwrap();
        let credential_store = Arc::new(TestSecretStore::default());
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment.clone(),
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        service.secret_store = credential_store.clone();

        let first_secret = "openalex-first-private-canary";
        let event = service.execute(DesktopIntent::PreviewProviderSecretChange {
            provider: ProviderKind::OpenAlex,
            change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                first_secret.to_owned(),
            )),
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("a valid provider secret must preview");
        };
        assert_eq!(preview.kind, OperationKind::ProviderSecret);
        assert!(!format!("{preview:?}").contains(first_secret));
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "openalex-api-key-saved",
            }
        );
        let first = settings_store.load().unwrap();
        let reference = first
            .settings
            .providers
            .openalex
            .api_key_ref
            .clone()
            .expect("configuration stores an opaque reference");
        let settings_bytes = fs::read(
            config_root(&environment)
                .unwrap()
                .state_root()
                .join(qiongli_config::GLOBAL_SETTINGS_FILE),
        )
        .unwrap();
        assert!(
            !settings_bytes
                .windows(first_secret.len())
                .any(|bytes| bytes == first_secret.as_bytes())
        );

        let content = crate::embedded_content().unwrap();
        let mut restarted = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        restarted.secret_store = credential_store.clone();
        assert_eq!(
            restarted.execute(DesktopIntent::TestLiteratureProvider {
                provider: ProviderKind::OpenAlex,
            }),
            DesktopEvent::Completed {
                code: "literature-provider-ready",
            }
        );

        let replacement_secret = "openalex-second-private-canary";
        let DesktopEvent::PreviewReady(replace_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::OpenAlex,
                change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                    replacement_secret.to_owned(),
                )),
            })
        else {
            panic!("credential replacement must preview");
        };
        assert!(matches!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: replace_preview.token,
            }),
            DesktopEvent::Completed { .. }
        ));
        assert_eq!(
            credential_store.resolve(&reference).unwrap().as_bytes(),
            replacement_secret.as_bytes()
        );

        let rollback_secret = "openalex-rollback-private-canary";
        let DesktopEvent::PreviewReady(rollback_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::OpenAlex,
                change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                    rollback_secret.to_owned(),
                )),
            })
        else {
            panic!("rollback credential must preview");
        };
        let external = settings_store.load().unwrap();
        settings_store
            .replace(external.revision, external.settings)
            .unwrap();
        assert_eq!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: rollback_preview.token,
            }),
            DesktopEvent::Failed {
                code: "revision-conflict",
            }
        );
        assert_eq!(
            credential_store.resolve(&reference).unwrap().as_bytes(),
            replacement_secret.as_bytes()
        );

        let DesktopEvent::PreviewReady(remove_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::OpenAlex,
                change: ProviderSecretChange::Remove,
            })
        else {
            panic!("credential removal must preview");
        };
        assert_eq!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: remove_preview.token,
            }),
            DesktopEvent::Completed {
                code: "openalex-api-key-removed",
            }
        );
        assert!(
            settings_store
                .load()
                .unwrap()
                .settings
                .providers
                .openalex
                .api_key_ref
                .is_none()
        );
        assert!(matches!(
            credential_store.resolve(&reference),
            Err(qiongli_config::SecretStoreError::NotFound)
        ));
        let DesktopEvent::PreviewReady(semantic_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::SemanticScholar,
                change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                    "semantic-scholar-private-canary".to_owned(),
                )),
            })
        else {
            panic!("Semantic Scholar credential must preview");
        };
        assert_eq!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: semantic_preview.token,
            }),
            DesktopEvent::Completed {
                code: "semantic-scholar-api-key-saved",
            }
        );
        assert!(
            settings_store
                .load()
                .unwrap()
                .settings
                .providers
                .semantic_scholar
                .api_key_ref
                .is_some()
        );
        let DesktopEvent::PreviewReady(pubmed_preview) =
            restarted.execute(DesktopIntent::PreviewProviderSecretChange {
                provider: ProviderKind::PubMed,
                change: ProviderSecretChange::Replace(qiongli_ui::PrivateText::new(
                    "pubmed-private-canary".to_owned(),
                )),
            })
        else {
            panic!("PubMed credential must preview");
        };
        assert_eq!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: pubmed_preview.token,
            }),
            DesktopEvent::Completed {
                code: "pubmed-api-key-saved",
            }
        );
        assert_eq!(
            restarted.execute(DesktopIntent::TestLiteratureProvider {
                provider: ProviderKind::PubMed,
            }),
            DesktopEvent::Completed {
                code: "literature-provider-ready",
            }
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn direct_backend_experiment_is_quarantined_and_uses_the_secure_store_when_enabled() {
        let root = isolated_root("agent-backend-secret-lifecycle");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let settings_store = config_store(&environment).unwrap();
        let credential_store = Arc::new(TestSecretStore::default());
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment.clone(),
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        service.secret_store = credential_store.clone();

        assert_eq!(
            service.execute(DesktopIntent::PreviewAgentBackendSettingsPatch(
                AgentBackendSettingsPatch {
                    expected_revision: 0,
                    openai_enabled: true,
                },
            )),
            DesktopEvent::Failed {
                code: "host-driven-execution-required",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::PreviewAgentBackendSecretChange {
                change: AgentBackendSecretChange::Replace(qiongli_ui::PrivateText::new(
                    "unreachable-private-canary".to_owned(),
                )),
            }),
            DesktopEvent::Failed {
                code: "host-driven-execution-required",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::PreviewAgentRun(AgentRunDraft {
                project_id: "unreachable-project".to_owned(),
                expected_project_revision: 1,
                prompt: qiongli_ui::PrivateText::new("unreachable-prompt".to_owned()),
            })),
            DesktopEvent::Failed {
                code: "host-driven-execution-required",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::TestOpenAiBackend),
            DesktopEvent::Failed {
                code: "host-driven-execution-required",
            }
        );

        service.direct_backend_experiment_enabled = true;
        let DesktopEvent::PreviewReady(settings_preview) = service.execute(
            DesktopIntent::PreviewAgentBackendSettingsPatch(AgentBackendSettingsPatch {
                expected_revision: 0,
                openai_enabled: true,
            }),
        ) else {
            panic!("backend enablement must preview");
        };
        assert_eq!(settings_preview.kind, OperationKind::AgentBackendSettings);
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: settings_preview.token,
            }),
            DesktopEvent::Completed {
                code: "agent-backend-settings-updated",
            }
        );
        assert_eq!(
            service.snapshot().config.openai_backend.readiness,
            AgentBackendReadinessView::NeedsSecretReference
        );
        assert_eq!(
            service.execute(DesktopIntent::TestOpenAiBackend),
            DesktopEvent::Failed {
                code: "agent-backend-secret-reference-missing",
            }
        );

        let secret = "openai-source-build-private-canary";
        let DesktopEvent::PreviewReady(secret_preview) =
            service.execute(DesktopIntent::PreviewAgentBackendSecretChange {
                change: AgentBackendSecretChange::Replace(qiongli_ui::PrivateText::new(
                    secret.to_owned(),
                )),
            })
        else {
            panic!("backend credential must preview");
        };
        assert_eq!(secret_preview.kind, OperationKind::AgentBackendSecret);
        assert!(!format!("{secret_preview:?}").contains(secret));
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: secret_preview.token,
            }),
            DesktopEvent::Completed {
                code: "openai-api-key-saved",
            }
        );

        let committed = settings_store.load().unwrap();
        let reference = committed
            .settings
            .agent_backends
            .openai
            .api_key_ref
            .clone()
            .expect("configuration must retain only an opaque reference");
        assert_eq!(
            credential_store.resolve(&reference).unwrap().as_bytes(),
            secret.as_bytes()
        );
        let settings_bytes = fs::read(
            config_root(&environment)
                .unwrap()
                .state_root()
                .join(qiongli_config::GLOBAL_SETTINGS_FILE),
        )
        .unwrap();
        assert!(
            !settings_bytes
                .windows(secret.len())
                .any(|bytes| bytes == secret.as_bytes())
        );
        let backend = service.snapshot().config.openai_backend;
        assert_eq!(
            backend.readiness,
            AgentBackendReadinessView::CredentialUnverified
        );
        assert!(backend.test_available);

        let projects = project_state_service(&environment).unwrap();
        let project_plan = projects
            .preview_create(
                root.join("agent-run-article"),
                ProjectRegistrationOptions::new("Agent Run Article", ProjectKind::Article),
                1,
            )
            .unwrap();
        let project_id = project_plan.preview().project_id.clone();
        projects
            .apply(
                &project_plan,
                &ApprovedProjectMutation::new(project_plan.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        let prompt = "private-agent-run-preview-canary";
        let DesktopEvent::PreviewReady(run_preview) =
            service.execute(DesktopIntent::PreviewAgentRun(AgentRunDraft {
                project_id: project_id.as_str().to_owned(),
                expected_project_revision: 1,
                prompt: qiongli_ui::PrivateText::new(prompt.to_owned()),
            }))
        else {
            panic!("ready project and backend must produce a run preview");
        };
        assert_eq!(run_preview.kind, OperationKind::AgentRun);
        assert_eq!(
            run_preview.approvals_required,
            vec![OperationApproval::NetworkRequest]
        );
        assert!(run_preview.can_confirm);
        assert!(!format!("{run_preview:?}").contains(prompt));
        assert_eq!(
            service.execute(DesktopIntent::CancelOperation {
                token: run_preview.token,
            }),
            DesktopEvent::Cancelled {
                code: "operation-preview-cancelled",
            }
        );

        service.direct_backend_experiment_enabled = false;
        let DesktopEvent::PreviewReady(remove_preview) =
            service.execute(DesktopIntent::PreviewAgentBackendSecretChange {
                change: AgentBackendSecretChange::Remove,
            })
        else {
            panic!("backend credential removal must preview");
        };
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: remove_preview.token,
            }),
            DesktopEvent::Completed {
                code: "openai-api-key-removed",
            }
        );
        assert!(matches!(
            credential_store.resolve(&reference),
            Err(qiongli_config::SecretStoreError::NotFound)
        ));
        assert!(
            settings_store
                .load()
                .unwrap()
                .settings
                .agent_backends
                .openai
                .api_key_ref
                .is_none()
        );
        assert!(
            !settings_store
                .load()
                .unwrap()
                .settings
                .agent_backends
                .openai
                .enabled
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn global_settings_confirmation_rejects_a_stale_revision() {
        let root = isolated_root("global-settings-stale");
        let home = root.join("home");
        let config = root.join("configured");
        fs::create_dir_all(&home).unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home), None);
        let store = config_store(&environment).unwrap();
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );

        let event = service.execute(DesktopIntent::PreviewGlobalSettingsPatch(
            GlobalSettingsPatch {
                expected_revision: 0,
                default_profile: ProfileKind::Full,
            },
        ));
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("missing settings must preview from revision zero");
        };
        let external = GlobalSettings {
            default_profile: ProfileId::SkillOnly,
            ..GlobalSettings::default()
        };
        store.replace(0, external.clone()).unwrap();

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Failed {
                code: "revision-conflict",
            }
        );
        assert_eq!(store.load().unwrap().settings, external);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unchanged_global_settings_produce_a_read_only_preview() {
        let root = isolated_root("global-settings-read-only-preview");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );

        let event = service.execute(DesktopIntent::PreviewGlobalSettingsPatch(
            GlobalSettingsPatch {
                expected_revision: 0,
                default_profile: ProfileKind::MarketplaceLite,
            },
        ));
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("unchanged settings must produce a visible read-only preview");
        };
        assert_eq!(preview.kind, OperationKind::GlobalSettings);
        assert!(!preview.can_confirm);
        assert_eq!(preview.blocked_reason, Some("global-settings-unchanged"));
        assert!(preview.plan_digest_sha256.is_none());
        assert!(preview.approvals_required.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selected_skills_destination_materializes_and_verifies_without_debug_path() {
        let root = isolated_root("skills-destination");
        let home = root.join("home");
        let target = root.join("selected-private-canary");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&target).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home.clone()), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker {
                path: Some(target.clone()),
            }),
        );

        let selected = service.execute(DesktopIntent::SelectSkillsDestination);
        assert!(!format!("{selected:?}").contains("selected-private-canary"));
        let DesktopEvent::SkillsDestinationSelected { target_id, .. } = selected else {
            panic!("custom Skills selection must return an opaque target identity");
        };

        let event = service.execute(DesktopIntent::PreviewSkillsPresetMaterialization {
            profile: ProfileKind::SkillOnly,
            preset: SkillsDestinationPreset::CustomFolder,
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("selected Skills destination must preview");
        };
        assert_eq!(preview.kind, OperationKind::SkillsMaterialization);
        assert_eq!(
            preview.approvals_required,
            vec![OperationApproval::FilesystemWrite]
        );
        assert_eq!(
            preview
                .display_target
                .as_ref()
                .map(PrivateDisplayText::expose),
            Some("<custom-folder>")
        );
        assert!(!format!("{preview:?}").contains("selected-private-canary"));

        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-completed",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::VerifySkillsPreset {
                preset: SkillsDestinationPreset::CustomFolder,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-verified",
            }
        );
        let approved = approve_materialization_target(&target).unwrap();
        let receipt = verify_materialization(&approved).unwrap();
        assert_eq!(receipt.profile, ProfileId::SkillOnly);
        let installed = service.snapshot();
        assert!(installed.content.managed_skills.iter().any(|entry| {
            entry.target_id == target_id
                && entry.preset == SkillsDestinationPreset::CustomFolder
                && entry.state == ManagedSkillsStateView::Current
        }));

        drop(service);
        let environment = CommandEnvironment::with_paths(None, Some(home), None);
        let content = crate::embedded_content().unwrap();
        let mut restarted = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        assert_eq!(
            restarted.execute(DesktopIntent::VerifyManagedSkillsTarget {
                target_id: target_id.clone(),
            }),
            DesktopEvent::Completed {
                code: "managed-skills-target-verified",
            }
        );
        assert_eq!(
            restarted.execute(DesktopIntent::PreviewManagedSkillsTargetUpdate {
                target_id: target_id.clone(),
            }),
            DesktopEvent::Completed {
                code: "managed-skills-target-already-current",
            }
        );
        assert_eq!(
            restarted.execute(DesktopIntent::VerifyManagedSkillsTarget {
                target_id: "skills-target-invalid".to_owned(),
            }),
            DesktopEvent::Failed {
                code: "managed-skills-target-id-invalid",
            }
        );
        assert_eq!(
            restarted.execute(DesktopIntent::VerifyManagedSkillsTarget {
                target_id: format!("skills-target-{}", "0".repeat(64)),
            }),
            DesktopEvent::Failed {
                code: "managed-skills-target-not-registered",
            }
        );

        let removal =
            restarted.execute(DesktopIntent::PreviewManagedSkillsTargetRemoval { target_id });
        let DesktopEvent::PreviewReady(removal_preview) = removal else {
            panic!("verified Skills destination must preview removal");
        };
        assert_eq!(removal_preview.kind, OperationKind::SkillsRemoval);
        assert_eq!(
            removal_preview
                .display_target
                .as_ref()
                .map(PrivateDisplayText::expose),
            Some("<custom-folder>")
        );
        assert!(!format!("{removal_preview:?}").contains("selected-private-canary"));
        assert_eq!(
            restarted.execute(DesktopIntent::ConfirmOperation {
                token: removal_preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-removed",
            }
        );
        assert!(!target.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn qiongli_managed_skills_preset_needs_no_folder_picker() {
        let root = isolated_root("skills-managed-preset");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(home.clone()), None);
        let content = crate::embedded_content().unwrap();
        let mut service = NativeDesktopService::new_with_folder_picker(
            environment,
            content,
            Box::new(FakeFolderPicker { path: None }),
        );
        let before = service.snapshot();
        let managed_before = before
            .content
            .managed_skills
            .iter()
            .find(|entry| entry.preset == SkillsDestinationPreset::QiongliManaged)
            .unwrap();
        assert_eq!(managed_before.state, ManagedSkillsStateView::Missing);
        assert!(!format!("{managed_before:?}").contains(root.to_str().unwrap()));

        let event = service.execute(DesktopIntent::PreviewSkillsPresetMaterialization {
            profile: ProfileKind::SkillOnly,
            preset: SkillsDestinationPreset::QiongliManaged,
        });
        let DesktopEvent::PreviewReady(preview) = event else {
            panic!("Qiongli Managed must preview without a folder picker");
        };
        assert_eq!(preview.kind, OperationKind::SkillsMaterialization);
        assert_eq!(
            preview
                .display_target
                .as_ref()
                .map(PrivateDisplayText::expose),
            Some("<user-home>/.qiongli-skills")
        );
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-completed",
            }
        );
        assert_eq!(
            service.execute(DesktopIntent::VerifySkillsPreset {
                preset: SkillsDestinationPreset::QiongliManaged,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-verified",
            }
        );
        assert!(home.join(".qiongli-skills").is_dir());
        let installed = service.snapshot();
        let managed_installed = installed
            .content
            .managed_skills
            .iter()
            .find(|entry| entry.preset == SkillsDestinationPreset::QiongliManaged)
            .unwrap();
        assert_eq!(managed_installed.state, ManagedSkillsStateView::Current);
        assert_eq!(managed_installed.profile, Some(ProfileKind::SkillOnly));
        assert_eq!(managed_installed.status, StatusCode::Ready);
        assert!(!format!("{managed_installed:?}").contains(root.to_str().unwrap()));

        let managed_target = home.join(".qiongli-skills");
        fs::write(managed_target.join(".qiongli-managed.json"), b"{}").unwrap();
        fs::write(
            managed_target.join("retained-user-change.txt"),
            b"retain-this-user-change",
        )
        .unwrap();
        let drifted = service.snapshot();
        let managed_drifted = drifted
            .content
            .managed_skills
            .iter()
            .find(|entry| entry.preset == SkillsDestinationPreset::QiongliManaged)
            .unwrap();
        assert_eq!(managed_drifted.state, ManagedSkillsStateView::Drifted);
        assert_eq!(drifted.content.managed_skills_status, StatusCode::Drifted);
        let target_id = managed_drifted.target_id.clone();
        assert_eq!(
            service.execute(DesktopIntent::VerifyManagedSkillsTarget {
                target_id: target_id.clone(),
            }),
            DesktopEvent::Completed {
                code: "managed-skills-target-drift-confirmed",
            }
        );
        let DesktopEvent::PreviewReady(detach_preview) =
            service.execute(DesktopIntent::PreviewManagedSkillsTargetDetach {
                target_id: target_id.clone(),
            })
        else {
            panic!("drifted Skills destination must offer a preserve-and-detach preview");
        };
        assert_eq!(detach_preview.kind, OperationKind::SkillsDetach);
        assert_eq!(
            detach_preview
                .display_target
                .as_ref()
                .map(PrivateDisplayText::expose),
            Some("<user-home>/.qiongli-skills")
        );
        assert!(!format!("{detach_preview:?}").contains(root.to_str().unwrap()));
        assert_eq!(
            service.execute(DesktopIntent::ConfirmOperation {
                token: detach_preview.token,
            }),
            DesktopEvent::Completed {
                code: "managed-skills-target-detached-preserved",
            }
        );
        assert_eq!(
            fs::read(managed_target.join(".qiongli-managed.json")).unwrap(),
            b"{}"
        );
        assert_eq!(
            fs::read(managed_target.join("retained-user-change.txt")).unwrap(),
            b"retain-this-user-change"
        );
        let detached = service.snapshot();
        let managed_detached = detached
            .content
            .managed_skills
            .iter()
            .find(|entry| entry.preset == SkillsDestinationPreset::QiongliManaged)
            .unwrap();
        assert_eq!(managed_detached.target_id, target_id);
        assert_eq!(managed_detached.state, ManagedSkillsStateView::Unmanaged);
        assert_eq!(managed_detached.status, StatusCode::Conflict);
        assert_eq!(detached.content.managed_skills_status, StatusCode::Conflict);
        assert!(managed_target.is_dir());
        let state_root = config_root(&service.environment).unwrap();
        assert!(
            crate::managed_content::load_managed_content_registry(state_root.state_root())
                .unwrap()
                .entries
                .is_empty()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registered_project_skills_converge_between_cli_and_desktop_after_restart() {
        let root = isolated_root("desktop-explicit-project-context");
        let home = root.join("home");
        let configured = root.join("configured");
        let project = root.join("project");
        fs::create_dir_all(&home).unwrap();
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut project_state =
            ProjectDesktopState::new(Some(ProjectStateService::new(config_root.clone())));
        let (create_token, _) = project_state.select_create_root(project.clone()).unwrap();
        project_state
            .preview_create(
                &create_token,
                "Explicit Skills project".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let create_operation_token = project_state.pending.as_ref().unwrap().token().to_owned();
        project_state
            .confirm(&create_operation_token)
            .unwrap()
            .unwrap();
        let project_id = project_state.snapshot().projects[0].project_id.clone();

        let cli_environment = CommandEnvironment::with_paths(
            Some(configured.clone().into_os_string()),
            Some(home.clone()),
            None,
        )
        .with_inventory_context(None, Some(project.clone()), false, false);
        let content = crate::embedded_content().unwrap();
        let mut cli_context_service =
            NativeDesktopService::new(cli_environment.clone(), content, Vec::new());
        let DesktopEvent::PreviewReady(preview) =
            cli_context_service.execute(DesktopIntent::PreviewSkillsPresetMaterialization {
                profile: ProfileKind::SkillOnly,
                preset: SkillsDestinationPreset::CurrentProject,
            })
        else {
            panic!("an explicit CLI project context must support current-project Skills");
        };
        assert_eq!(
            cli_context_service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-completed",
            }
        );
        let target_id = cli_context_service
            .snapshot()
            .content
            .managed_skills
            .iter()
            .find(|entry| entry.preset == SkillsDestinationPreset::CurrentProject)
            .unwrap()
            .target_id
            .clone();
        assert!(project.join(".qiongli-skills").is_dir());
        drop(cli_context_service);

        let mut desktop_service = NativeDesktopService::new(
            cli_environment.clone().without_project_context(),
            crate::embedded_content().unwrap(),
            Vec::new(),
        );
        let desktop_snapshot = desktop_service.snapshot();
        assert!(
            desktop_snapshot
                .content
                .managed_skills
                .iter()
                .all(|entry| entry.preset != SkillsDestinationPreset::CurrentProject)
        );
        let retained = desktop_snapshot
            .content
            .managed_skills
            .iter()
            .find(|entry| entry.target_id == target_id)
            .expect("receipt-owned project Skills must remain manageable by opaque target");
        assert_eq!(retained.preset, SkillsDestinationPreset::CustomFolder);
        assert_eq!(retained.state, ManagedSkillsStateView::Current);
        let project_service = Some(ProjectStateService::new(config_root));
        let project_skills = app_project_skills_targets(
            &desktop_service.environment,
            &project_service,
            &desktop_service.content,
        );
        let DesktopEvent::PreviewReady(removal_preview) =
            desktop_service.execute(DesktopIntent::PreviewManagedSkillsTargetRemoval {
                target_id: target_id.clone(),
            })
        else {
            panic!("a receipt-owned registered project target must be removable");
        };
        assert_eq!(
            removal_preview
                .display_target
                .as_ref()
                .map(PrivateDisplayText::expose),
            Some("<custom-folder>")
        );
        let DesktopEvent::PreviewReady(project_removal_preview) =
            apply_app_project_skills_preview_target(
                DesktopEvent::PreviewReady(removal_preview),
                Some(&target_id),
                &project_skills,
            )
        else {
            panic!("the App projection must preserve the native preview");
        };
        assert_eq!(
            project_removal_preview
                .display_target
                .as_ref()
                .map(PrivateDisplayText::expose),
            Some("<project>/.qiongli-skills")
        );
        assert_eq!(
            desktop_service.execute(DesktopIntent::CancelOperation {
                token: project_removal_preview.token,
            }),
            DesktopEvent::Cancelled {
                code: "operation-preview-cancelled",
            }
        );
        let app_snapshot = AppSnapshotV1::from_desktop(
            desktop_snapshot,
            project_snapshot(&project_service),
            project_skills,
        )
        .unwrap();
        let app_snapshot = serde_json::to_value(app_snapshot).unwrap();
        let project_target = app_snapshot["content"]["managedSkills"]["destinations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|destination| destination["targetId"] == target_id)
            .unwrap();
        assert_eq!(project_target["preset"], "current-project");
        assert_eq!(project_target["projectId"], project_id.as_str());
        assert_eq!(project_target["symbolicPath"], "<project>/.qiongli-skills");
        assert!(
            json_string_value_containing(&app_snapshot, root.to_str().unwrap()).is_none(),
            "the App snapshot must not expose a registered project path"
        );
        assert_eq!(
            desktop_service.execute(DesktopIntent::VerifyManagedSkillsTarget {
                target_id: target_id.clone(),
            }),
            DesktopEvent::Completed {
                code: "managed-skills-target-verified",
            }
        );
        let registered_root = project_service
            .as_ref()
            .unwrap()
            .resolve_project_root(&project_id)
            .unwrap();
        let binding_snapshot = project_snapshot(&project_service);
        let expected_project_revision = binding_snapshot.projects[0].semantic_revision;
        let DesktopEvent::PreviewReady(preview) = desktop_service
            .preview_registered_project_skills_materialization(
                ProfileKind::SkillOnly,
                registered_root.path(),
                project_id,
                binding_snapshot.revision,
                expected_project_revision,
            )
        else {
            panic!("registered project Skills must be previewable without process CWD");
        };
        assert_eq!(
            preview
                .display_target
                .as_ref()
                .map(PrivateDisplayText::expose),
            Some("<project>/.qiongli-skills")
        );
        assert!(!format!("{preview:?}").contains(root.to_str().unwrap()));
        assert_eq!(
            desktop_service
                .validate_registered_project_skills_confirmation(preview.token, &project_service,),
            Ok(())
        );
        assert_eq!(
            desktop_service.execute(DesktopIntent::ConfirmOperation {
                token: preview.token,
            }),
            DesktopEvent::Completed {
                code: "skills-materialization-completed",
            }
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registered_project_skills_preview_and_confirmation_revalidate_project_state() {
        let root = isolated_root("registered-project-skills-preconditions");
        let home = root.join("home");
        let configured = root.join("configured");
        let project = root.join("project");
        fs::create_dir_all(&home).unwrap();
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut projects =
            ProjectDesktopState::new(Some(ProjectStateService::new(config_root.clone())));
        let (create_token, _) = projects.select_create_root(project.clone()).unwrap();
        projects
            .preview_create(
                &create_token,
                "Skills preconditions".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let create_operation_token = projects.pending.as_ref().unwrap().token().to_owned();
        projects.confirm(&create_operation_token).unwrap().unwrap();
        let project_id = projects.snapshot().projects[0].project_id.clone();
        let (project_root, library_revision, project_revision) = projects
            .registered_project_skills_target(&project_id)
            .unwrap();
        let environment = CommandEnvironment::with_paths(
            Some(configured.clone().into_os_string()),
            Some(home),
            None,
        );
        let mut desktop =
            NativeDesktopService::new(environment, crate::embedded_content().unwrap(), Vec::new());
        let DesktopEvent::PreviewReady(drift_preview) = desktop
            .preview_registered_project_skills_materialization(
                ProfileKind::SkillOnly,
                project_root.path(),
                project_id.clone(),
                library_revision,
                project_revision,
            )
        else {
            panic!("ready registered project must produce a bound Skills preview");
        };

        fs::write(
            project.join("context/research_state.md"),
            "RQ: Changed after the Skills preview.\n",
        )
        .unwrap();
        assert!(matches!(
            projects.registered_project_skills_target(&project_id),
            Err("project-skills-project-not-ready")
        ));
        assert_eq!(
            desktop.validate_registered_project_skills_confirmation(
                drift_preview.token,
                &projects.service,
            ),
            Err("project-skills-project-not-ready")
        );
        assert!(!project.join(".qiongli-skills").exists());

        projects
            .preview_lifecycle(&project_id, ProjectMutationKind::Refresh)
            .unwrap();
        let refresh_token = projects.pending.as_ref().unwrap().token().to_owned();
        projects.confirm(&refresh_token).unwrap().unwrap();
        let (project_root, library_revision, project_revision) = projects
            .registered_project_skills_target(&project_id)
            .unwrap();
        let DesktopEvent::PreviewReady(archive_preview) = desktop
            .preview_registered_project_skills_materialization(
                ProfileKind::SkillOnly,
                project_root.path(),
                project_id.clone(),
                library_revision,
                project_revision,
            )
        else {
            panic!("refreshed project must produce a new bound Skills preview");
        };
        projects
            .preview_lifecycle(&project_id, ProjectMutationKind::Archive)
            .unwrap();
        let archive_token = projects.pending.as_ref().unwrap().token().to_owned();
        projects.confirm(&archive_token).unwrap().unwrap();
        assert!(matches!(
            projects.registered_project_skills_target(&project_id),
            Err("project-skills-project-archived")
        ));
        assert_eq!(
            desktop.validate_registered_project_skills_confirmation(
                archive_preview.token,
                &projects.service,
            ),
            Err("project-skills-project-archived")
        );
        assert!(!project.join(".qiongli-skills").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_update_views_are_bounded_and_recovery_oriented() {
        let current = update_checked_view(crate::update_cli::DesktopUpdateCheck {
            disposition: crate::update_cli::DesktopUpdateCheckDisposition::Current,
            selected_stream: UpdateStreamPreference::Stable,
            target_version: "2.0.0".to_owned(),
            archive_size_bytes: 24 * 1024 * 1024,
        });
        assert!(current.validate());
        assert_eq!(current.phase, UpdatePhaseView::Current);
        assert!(!current.can_prepare);

        let available = update_checked_view(crate::update_cli::DesktopUpdateCheck {
            disposition: crate::update_cli::DesktopUpdateCheckDisposition::Available,
            selected_stream: UpdateStreamPreference::Beta,
            target_version: "2.0.0-alpha.2".to_owned(),
            archive_size_bytes: 24 * 1024 * 1024,
        });
        assert!(available.validate());
        assert_eq!(available.phase, UpdatePhaseView::Available);
        assert!(available.can_prepare);

        let offline = update_failure_base(
            UpdateStreamView::Beta,
            None,
            "native-update-manifest-timeout",
            false,
            false,
        );
        assert!(offline.validate());
        assert_eq!(offline.remediation, UpdateRemediation::RetryCheck);
        assert!(offline.can_check);

        let expired = update_failure_base(
            UpdateStreamView::Beta,
            None,
            "native-update-manifest-expired",
            false,
            false,
        );
        assert!(expired.validate());
        assert_eq!(expired.remediation, UpdateRemediation::RetryCheck);

        let corrupt = update_failure_base(
            UpdateStreamView::Beta,
            Some("2.0.0-alpha.2".to_owned()),
            "native-update-archive-digest-mismatch",
            true,
            false,
        );
        assert!(corrupt.validate());
        assert_eq!(corrupt.remediation, UpdateRemediation::CancelAndRetry);
        assert!(corrupt.can_cancel);

        let read_only = update_failure_base(
            UpdateStreamView::Beta,
            Some("2.0.0-alpha.2".to_owned()),
            "native-update-installation-location-not-writable",
            true,
            true,
        );
        assert!(read_only.validate());
        assert_eq!(read_only.remediation, UpdateRemediation::MoveToApplications);

        let health_failure = update_failure_base(
            UpdateStreamView::Beta,
            Some("2.0.0-alpha.2".to_owned()),
            "native-update-health-check-failed",
            false,
            false,
        );
        assert!(health_failure.validate());
        assert_eq!(
            health_failure.remediation,
            UpdateRemediation::ReinstallApplication
        );

        let cancelling = update_busy_view(
            &available,
            UpdatePhaseView::Cancelling,
            1,
            "Removing staged update bytes",
        );
        assert!(cancelling.validate());
        assert_eq!(cancelling.phase, UpdatePhaseView::Cancelling);

        for phase in [
            UpdatePhaseView::Downloading,
            UpdatePhaseView::Verifying,
            UpdatePhaseView::Staging,
        ] {
            let preparing = update_busy_view(&available, phase, 1, "Preparing update");
            assert!(preparing.validate());
            assert!(
                !preparing.can_cancel,
                "an active preparation worker must not race a cancellation worker"
            );
        }

        let restarting = UpdateView {
            status: StatusCode::Busy,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::AwaitingRestart,
            available_version: Some("2.0.0-alpha.2".to_owned()),
            archive_size_bytes: None,
            progress: Some(UpdateProgressView {
                completed_steps: 4,
                total_steps: 4,
                label: "Completing application replacement",
                indeterminate: true,
            }),
            reason_code: "update-restart-in-progress",
            remediation: UpdateRemediation::RestartApplication,
            can_select_stream: false,
            can_check: false,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        };
        assert!(restarting.validate());
    }

    #[test]
    fn desktop_update_install_confirmation_is_revision_and_transaction_bound() {
        let first = update_install_digest(
            7,
            "update-0123456789abcdef0123456789abcdef",
            "2.0.0-alpha.2",
        );
        assert_eq!(first.len(), 64);
        assert_eq!(
            first,
            update_install_digest(
                7,
                "update-0123456789abcdef0123456789abcdef",
                "2.0.0-alpha.2",
            )
        );
        assert_ne!(
            first,
            update_install_digest(
                8,
                "update-0123456789abcdef0123456789abcdef",
                "2.0.0-alpha.2",
            )
        );
        assert_ne!(
            first,
            update_install_digest(
                7,
                "update-fedcba9876543210fedcba9876543210",
                "2.0.0-alpha.2",
            )
        );
    }

    #[test]
    fn project_workspace_selection_resolves_one_canonical_article_root() {
        let root = isolated_root("project-workspace-selection");
        let workspace = root.join("workspace");
        let research = workspace.join("RESEARCH");
        let article = research.join("article-topic");
        create_private_directory(&workspace);
        create_private_directory(&research);
        create_private_directory(&article);

        assert_eq!(
            resolve_selected_article_project_root(workspace.clone()).unwrap(),
            article
        );
        assert_eq!(
            resolve_selected_article_project_root(research.clone()).unwrap(),
            article
        );
        assert_eq!(
            resolve_selected_article_project_root(article.clone()).unwrap(),
            article
        );
        assert_eq!(
            article_project_root_in_workspace(&workspace, "new-article"),
            research.join("new-article")
        );
        assert_eq!(
            article_project_root_in_workspace(&research, "new-article"),
            research.join("new-article")
        );

        let second = research.join("second-topic");
        create_private_directory(&second);
        assert_eq!(
            resolve_selected_article_project_root(workspace),
            Err("multiple-article-projects-found-select-topic")
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_registers_and_archives_through_previewed_mutations() {
        let root = isolated_root("project-library");
        let home = root.join("home");
        let configured = root.join("configured");
        let project = root.join("article-project");
        create_private_directory(&home);
        create_private_directory(&project);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (directory_token, root_label) = state.select_register_root(project.clone()).unwrap();
        assert_eq!(root_label, "article-project");
        assert_eq!(directory_token.len(), 32);
        assert!(!directory_token.contains("article-project"));

        state.preview_register(&directory_token).unwrap();
        let register_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&register_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-registration-completed");
        assert_eq!(confirmed.capture_project_id, None);
        let registered = state.snapshot();
        assert_eq!(registered.projects.len(), 1);
        let project_id = registered.projects[0].project_id.clone();

        state
            .preview_lifecycle(&project_id, ProjectMutationKind::Archive)
            .unwrap();
        let archive_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&archive_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-archive-completed");
        assert_eq!(confirmed.capture_project_id, None);
        assert_eq!(
            state.snapshot().projects[0].lifecycle,
            qiongli_project::ProjectLifecycle::Archived
        );

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_compares_only_against_its_validated_graph_baseline() {
        let root = isolated_root("project-graph-comparison");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("article-project");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root).unwrap();
        state
            .preview_create(
                &create_token,
                "Graph comparison article".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&operation_token).unwrap().unwrap();
        let project_id = state.snapshot().projects[0].project_id.clone();

        let (first, _, baseline) = state.academic_graph(&project_id).unwrap();
        assert!(baseline.is_none());
        let (second, _, comparison) = state.academic_graph(&project_id).unwrap();
        let comparison = comparison.expect("second load compares the validated baseline");
        assert_eq!(first, second);
        assert_eq!(comparison.before_projection_id, first.projection_id);
        assert_eq!(comparison.after_projection_id, second.projection_id);
        assert!(!comparison.has_changes);

        let portfolio = state.academic_graph_portfolio().unwrap();
        assert_eq!(portfolio.included_project_count, 1);
        assert_eq!(portfolio.project_count, 1);
        assert_eq!(portfolio.node_count, 1);
        assert_eq!(portfolio.edge_count, 0);

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_projects_current_portfolio_query_timeline_and_doctor() {
        let root = isolated_root("desktop-continuity-read-projection");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("article-project");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root.clone()).unwrap();
        state
            .preview_create(
                &create_token,
                "Continuity read projection".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&operation_token).unwrap().unwrap();
        let project_id = state.snapshot().projects[0].project_id.clone();

        let missing = serde_json::to_value(state.portfolio_status().unwrap()).unwrap();
        assert_eq!(missing["state"], "missing");
        assert_eq!(missing["capabilities"]["canQuery"], false);

        let current = IncrementalPortfolioService::new(state.service.as_ref().unwrap().clone())
            .reconcile(now_unix().unwrap())
            .unwrap()
            .snapshot;
        let status = serde_json::to_value(state.portfolio_status().unwrap()).unwrap();
        assert_eq!(status["state"], "current");
        assert_eq!(status["catalogId"], current.catalog.catalog_id.as_str());
        assert_eq!(status["capabilities"]["canQuery"], true);

        let query_intent = serde_json::from_value::<crate::desktop_api::AppIntent>(json!({
            "action": "query-portfolio",
            "request": {
                "catalogId": current.catalog.catalog_id.as_str(),
                "filters": {
                    "projectId": project_id.as_str(),
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
        .unwrap();
        let crate::desktop_api::AppIntent::QueryPortfolio { request } = query_intent else {
            panic!("query intent must preserve its typed request");
        };
        let result = serde_json::to_value(state.query_portfolio(request).unwrap()).unwrap();
        assert_eq!(result["catalogId"], status["catalogId"]);
        assert_eq!(result["matchedProjectCount"], 1);

        let timeline_intent = serde_json::from_value::<crate::desktop_api::AppIntent>(json!({
            "action": "load-semantic-timeline",
            "request": {
                "catalogId": status["catalogId"].as_str().unwrap(),
                "projectId": project_id.as_str(),
                "view": "revision-history",
                "limit": 64,
                "maxBytes": 2097152
            }
        }))
        .unwrap();
        let crate::desktop_api::AppIntent::LoadSemanticTimeline { request } = timeline_intent
        else {
            panic!("timeline intent must preserve its typed request");
        };
        let timeline = serde_json::to_value(state.semantic_timeline(request).unwrap()).unwrap();
        assert_eq!(timeline["catalogId"], status["catalogId"]);
        assert_eq!(timeline["projectId"], project_id.as_str());
        assert!(timeline["matchedEventCount"].as_u64().unwrap() >= 2);

        let doctor = serde_json::to_value(state.portfolio_doctor().unwrap()).unwrap();
        assert_eq!(doctor["status"], "equivalent");
        assert_eq!(doctor["byteEquivalent"], true);
        for forbidden in [
            "projectRoot",
            "rootPath",
            project_root.to_string_lossy().as_ref(),
        ] {
            assert!(
                !status.to_string().contains(forbidden)
                    && !result.to_string().contains(forbidden)
                    && !timeline.to_string().contains(forbidden)
                    && !doctor.to_string().contains(forbidden)
            );
        }

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_reconciles_portfolio_through_a_stable_operation_result() {
        let root = isolated_root("desktop-portfolio-reconcile");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("article-project");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let service = ProjectStateService::new(config_root);
        let mut state = ProjectDesktopState::new(Some(service.clone()));

        let (create_token, _) = state.select_create_root(project_root).unwrap();
        state
            .preview_create(
                &create_token,
                "Portfolio reconciliation".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let create_operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&create_operation_token).unwrap().unwrap();
        assert_eq!(
            serde_json::to_value(state.portfolio_status().unwrap()).unwrap()["state"],
            "missing"
        );

        let (maintenance, preview) = state
            .preview_portfolio_maintenance(AppPortfolioMaintenanceOperation::Reconcile)
            .unwrap();
        let maintenance = serde_json::to_value(maintenance).unwrap();
        let preview = serde_json::to_value(preview).unwrap();
        assert_eq!(maintenance["operation"], "reconcile");
        assert_eq!(maintenance["derivedStateOnly"], true);
        assert_eq!(preview["kind"], "portfolio-reconcile");
        assert_eq!(preview["canConfirm"], true);
        assert_eq!(
            serde_json::to_value(state.portfolio_status().unwrap()).unwrap()["state"],
            "missing",
            "preview must not write derived portfolio state"
        );
        assert!(state.confirm("00000000000000000000000000000000").is_none());

        let token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&token).unwrap().unwrap();
        assert_eq!(confirmed.code, "portfolio-maintenance-started");
        let progress = confirmed.continuity_operation.unwrap();
        let progress = serde_json::to_value(progress).unwrap();
        assert_eq!(progress["operation"], "reconcile");
        assert_eq!(progress["phase"], "queued");
        let operation_id = progress["operationId"].as_str().unwrap().to_owned();

        let completed = wait_for_portfolio_completion(&state, &operation_id);
        let completed_json = serde_json::to_value(&completed).unwrap();
        assert_eq!(completed_json["operationId"], operation_id);
        assert_eq!(completed_json["operation"], serde_json::json!("reconcile"));
        assert!(completed_json["catalogId"].is_string());
        assert_eq!(completed_json["rebuiltProjectCount"], 1);
        assert_eq!(completed_json["derivedStateOnly"], true);
        assert_eq!(
            state.poll_continuity_operation(&operation_id).unwrap(),
            DesktopContinuityPoll::Completed(completed.clone()),
            "terminal polling must return the same result without restarting work"
        );
        assert_eq!(
            serde_json::to_value(state.portfolio_status().unwrap()).unwrap()["state"],
            "current"
        );

        let restarted = ProjectDesktopState::new(Some(service));
        assert_eq!(
            restarted.poll_continuity_operation(&operation_id),
            Err("continuity-operation-not-found"),
            "process-local operations must not be resumed after an app restart"
        );

        drop(restarted);
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_cancels_queued_portfolio_work_before_any_write() {
        let root = isolated_root("desktop-portfolio-cancel");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("article-project");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root).unwrap();
        state
            .preview_create(
                &create_token,
                "Cancelled portfolio operation".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let create_operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&create_operation_token).unwrap().unwrap();

        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        state.continuity_worker_gate = Some(Arc::clone(&gate));
        state
            .preview_portfolio_maintenance(AppPortfolioMaintenanceOperation::Reconcile)
            .unwrap();
        let token = state.pending.as_ref().unwrap().token().to_owned();
        let progress = state
            .confirm(&token)
            .unwrap()
            .unwrap()
            .continuity_operation
            .unwrap();
        let operation_id = serde_json::to_value(progress).unwrap()["operationId"]
            .as_str()
            .unwrap()
            .to_owned();

        for _ in 0..2 {
            let DesktopContinuityPoll::Progress(cancelled) =
                state.cancel_continuity_operation(&operation_id).unwrap()
            else {
                panic!("queued cancellation must remain observable as progress");
            };
            let cancelled = serde_json::to_value(cancelled).unwrap();
            assert_eq!(
                cancelled["reasonCode"],
                "portfolio-operation-cancellation-requested"
            );
            assert_eq!(cancelled["cancellable"], false);
        }
        assert_eq!(
            serde_json::to_value(state.portfolio_status().unwrap()).unwrap()["state"],
            "missing"
        );

        let (released, signal) = &*gate;
        *released.lock().unwrap() = true;
        signal.notify_all();
        let cancelled = wait_for_portfolio_cancellation(&state, &operation_id);
        let cancelled_json = serde_json::to_value(&cancelled).unwrap();
        assert_eq!(cancelled_json["phase"], "cancelled");
        assert_eq!(cancelled_json["completedUnits"], 0);
        assert_eq!(
            state.poll_continuity_operation(&operation_id).unwrap(),
            DesktopContinuityPoll::Progress(cancelled)
        );
        assert_eq!(
            serde_json::to_value(state.portfolio_status().unwrap()).unwrap()["state"],
            "missing",
            "cancelled work must not publish derived state"
        );

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_deletes_only_derived_portfolio_state() {
        let root = isolated_root("desktop-portfolio-delete");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("article-project");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root.clone()).unwrap();
        state
            .preview_create(
                &create_token,
                "Derived state deletion".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let create_operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&create_operation_token).unwrap().unwrap();
        IncrementalPortfolioService::new(state.service.as_ref().unwrap().clone())
            .reconcile(now_unix().unwrap())
            .unwrap();
        let project_files_before = collect_directory_files(&project_root);

        let (maintenance, _) = state
            .preview_portfolio_maintenance(AppPortfolioMaintenanceOperation::DeleteDerivedState)
            .unwrap();
        assert_eq!(
            serde_json::to_value(maintenance).unwrap()["operation"],
            "delete-derived-state"
        );
        assert_eq!(
            serde_json::to_value(state.portfolio_status().unwrap()).unwrap()["state"],
            "current",
            "delete preview must not mutate the current catalog"
        );

        let token = state.pending.as_ref().unwrap().token().to_owned();
        let operation_id = state
            .confirm(&token)
            .unwrap()
            .unwrap()
            .continuity_operation
            .unwrap();
        let operation_id = serde_json::to_value(operation_id).unwrap()["operationId"]
            .as_str()
            .unwrap()
            .to_owned();
        let completed = wait_for_portfolio_completion(&state, &operation_id);
        let completed = serde_json::to_value(completed).unwrap();
        assert_eq!(
            completed["operation"],
            serde_json::json!("delete-derived-state")
        );
        assert_eq!(completed["catalogId"], serde_json::Value::Null);
        assert_eq!(completed["portfolioId"], serde_json::Value::Null);
        assert_eq!(completed["removedContributionCount"], 1);
        assert_eq!(completed["derivedStateOnly"], true);
        assert_eq!(
            serde_json::to_value(state.portfolio_status().unwrap()).unwrap()["state"],
            "missing"
        );
        assert_eq!(
            collect_directory_files(&project_root),
            project_files_before,
            "derived-state deletion must not change canonical project files"
        );

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_closes_delivery_mutations_through_exact_native_state() {
        let root = isolated_root("desktop-delivery-mutations");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("private-project-path-canary");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root.clone()).unwrap();
        state
            .preview_create(
                &create_token,
                "Delivery mutation project".to_owned(),
                ProjectKind::Article,
                ProjectStage::Literature,
            )
            .unwrap();
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&operation_token).unwrap().unwrap();
        let project_id = state.snapshot().projects[0].project_id.clone();
        let base = now_unix().unwrap();
        let capture = qiongli_project::ResearchCaptureDraftV1 {
            binding: qiongli_project::ProjectBindingV1::new(
                project_id.clone(),
                1,
                ProjectStage::Literature,
                "Delivery acknowledgement fixture",
                qiongli_project::CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: qiongli_project::CaptureSource::Codex,
            delivery: qiongli_project::CaptureDelivery::Connected,
            captured_at_unix: base,
            summary: "Retain the accepted capture identity across delivery restart.".to_owned(),
            changes: vec![qiongli_project::SemanticChangeV1 {
                area: qiongli_project::CaptureArea::Literature,
                summary: "Track exact acknowledgement revisions.".to_owned(),
            }],
            decisions: Vec::new(),
            evidence: Vec::new(),
            contradictions: Vec::new(),
            next_actions: vec!["Confirm the destination evidence.".to_owned()],
        }
        .into_capture()
        .unwrap();
        let service = state.service.as_ref().unwrap().clone();
        let intake = service.preview_capture(capture.clone()).unwrap();
        service
            .apply_capture(
                &intake,
                &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                base + 1,
            )
            .unwrap();
        let envelope = qiongli_project::CaptureDeliveryEnvelopeV1::new(
            capture.clone(),
            Some(
                qiongli_project::CaptureDeliveryDestinationV1::new(project_id.clone(), 1).unwrap(),
            ),
            base + 2,
        )
        .unwrap();
        let queued = service.enqueue_capture_delivery(envelope.clone()).unwrap();
        let delivering = service
            .begin_capture_delivery(
                &envelope.envelope_id,
                queued.generation,
                &queued.record_sha256,
                base + 3,
            )
            .unwrap();
        let delivered = service
            .record_capture_delivery(
                &envelope.envelope_id,
                delivering.generation,
                &delivering.record_sha256,
                base + 4,
            )
            .unwrap();
        let request = CaptureDeliveryAcknowledgementRequestV1 {
            envelope_id: envelope.envelope_id.clone(),
            destination_project_id: project_id.clone(),
            accepted_capture_id: capture.capture_id.clone(),
            expected_project_revision: 1,
            resulting_project_revision: 1,
            acknowledged_at_unix: base + 5,
        };
        let (acknowledgement, preview) = state
            .preview_capture_delivery_acknowledgement(
                request,
                delivered.generation,
                &delivered.record_sha256,
            )
            .unwrap();
        let preview_json = serde_json::to_value((&acknowledgement, &preview)).unwrap();
        assert_eq!(preview_json[1]["canConfirm"], true);
        assert_eq!(
            preview_json[0]["planDigest"],
            preview_json[1]["planDigestSha256"]
        );
        assert!(
            !preview_json
                .to_string()
                .contains(project_root.to_string_lossy().as_ref())
        );
        assert_eq!(
            service
                .inspect_capture_delivery(&envelope.envelope_id)
                .unwrap()
                .unwrap(),
            delivered
        );
        assert!(state.confirm("00000000000000000000000000000000").is_none());
        let token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&token).unwrap().unwrap();
        assert_eq!(confirmed.code, "capture-delivery-acknowledged");
        assert!(matches!(
            confirmed.continuity,
            Some(ConfirmedCaptureContinuity::Delivery(_))
        ));
        assert!(state.confirm(&token).is_none());

        let retry_envelope =
            qiongli_project::CaptureDeliveryEnvelopeV1::new(capture, None, base + 6).unwrap();
        let retry_queued = service
            .enqueue_capture_delivery(retry_envelope.clone())
            .unwrap();
        let conflicted = service
            .begin_capture_delivery(
                &retry_envelope.envelope_id,
                retry_queued.generation,
                &retry_queued.record_sha256,
                base + 7,
            )
            .unwrap();
        let retry = state
            .retry_capture_delivery(
                &retry_envelope.envelope_id,
                conflicted.generation,
                &conflicted.record_sha256,
                base + 8,
                CaptureDeliveryRetryCause::ConflictResolved,
            )
            .unwrap();
        let retry_json = serde_json::to_value(&retry).unwrap();
        assert_eq!(retry_json["state"], "retry-required");
        assert_eq!(
            state.cancel_capture_delivery(
                &retry_envelope.envelope_id,
                conflicted.generation,
                &conflicted.record_sha256,
                base + 9,
            ),
            Err(ProjectError::RevisionConflict.reason_code())
        );
        let cancelled = state
            .cancel_capture_delivery(
                &retry_envelope.envelope_id,
                retry_json["generation"].as_u64().unwrap(),
                retry_json["recordSha256"].as_str().unwrap(),
                base + 9,
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(cancelled).unwrap()["state"],
            "cancelled"
        );

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_confirms_assignment_and_complete_resolution_once() {
        let root = isolated_root("desktop-assignment-resolution");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("assignment-project");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root.clone()).unwrap();
        state
            .preview_create(
                &create_token,
                "Assignment resolution project".to_owned(),
                ProjectKind::Article,
                ProjectStage::Literature,
            )
            .unwrap();
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&operation_token).unwrap().unwrap();
        let project_id = state.snapshot().projects[0].project_id.clone();
        let base = now_unix().unwrap();
        let capture = qiongli_project::ResearchCaptureDraftV1 {
            binding: qiongli_project::ProjectBindingV1::new(
                project_id.clone(),
                1,
                ProjectStage::Literature,
                "Assignment resolution fixture",
                qiongli_project::CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: qiongli_project::CaptureSource::ClaudeCode,
            delivery: qiongli_project::CaptureDelivery::Portable,
            captured_at_unix: base,
            summary: "Resolve exact item-scoped academic changes.".to_owned(),
            changes: vec![qiongli_project::SemanticChangeV1 {
                area: qiongli_project::CaptureArea::Literature,
                summary: "Add the reviewed continuity finding.".to_owned(),
            }],
            decisions: vec![qiongli_project::DecisionCandidateV1 {
                relation: qiongli_project::DecisionRelation::Candidate,
                statement: "Use exact lineage identities.".to_owned(),
                rationale: "Fuzzy merging would erase causal evidence.".to_owned(),
                target: None,
            }],
            evidence: Vec::new(),
            contradictions: Vec::new(),
            next_actions: vec!["Review every proposed item.".to_owned()],
        }
        .into_capture()
        .unwrap();
        let source =
            qiongli_project::CaptureDeliveryEnvelopeV1::new(capture, None, base + 1).unwrap();
        state
            .service
            .as_ref()
            .unwrap()
            .enqueue_capture_delivery(source.clone())
            .unwrap();

        let (assignment, preview) = state
            .preview_capture_assignment(
                &source.envelope_id,
                &project_id,
                AppCaptureAssignmentDecision::Assign,
                base + 2,
            )
            .unwrap();
        let assignment_json = serde_json::to_value((&assignment, &preview)).unwrap();
        assert_eq!(
            assignment_json[0]["planDigest"],
            assignment_json[1]["planDigestSha256"]
        );
        assert_eq!(
            assignment_json[1]["approvalsRequired"],
            json!(["assignment-write"])
        );
        let cancelled_token = state.pending.as_ref().unwrap().token().to_owned();
        let mut restarted = ProjectDesktopState::new(state.service.clone());
        assert!(restarted.confirm(&cancelled_token).is_none());
        assert!(state.cancel(&cancelled_token));
        assert!(
            state
                .service
                .as_ref()
                .unwrap()
                .list_capture_assignments()
                .unwrap()
                .is_empty()
        );

        state
            .preview_capture_assignment(
                &source.envelope_id,
                &project_id,
                AppCaptureAssignmentDecision::Assign,
                base + 2,
            )
            .unwrap();
        let assignment_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&assignment_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "capture-assignment-completed");
        let Some(ConfirmedCaptureContinuity::Assignment(assignment)) = confirmed.continuity else {
            panic!("confirmed assignment must return the affected native record");
        };
        assert_eq!(
            serde_json::to_value(&assignment).unwrap()["canResolve"],
            true
        );
        assert!(state.confirm(&assignment_token).is_none());

        let assignment_receipt_id = state
            .service
            .as_ref()
            .unwrap()
            .list_capture_assignments()
            .unwrap()[0]
            .receipt_id
            .clone()
            .unwrap();
        assert!(state.pending.is_none());
        let item_plan = state
            .capture_resolution_plan(&assignment_receipt_id, base + 3)
            .unwrap();
        assert!(
            !serde_json::to_value(&item_plan).unwrap()["items"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            state.pending.is_none(),
            "read-only resolution planning must not create a confirmable operation"
        );
        let domain_preview = state
            .service
            .as_ref()
            .unwrap()
            .preview_capture_resolution(&assignment_receipt_id, base + 3)
            .unwrap();
        let selection_json = domain_preview
            .preview()
            .items
            .iter()
            .map(|item| {
                json!({
                    "itemId": item.item.item_id.as_str(),
                    "disposition": "accept-capture"
                })
            })
            .collect::<Vec<_>>();
        let intent = serde_json::from_value::<crate::desktop_api::AppIntent>(json!({
            "action": "preview-capture-resolution",
            "assignmentReceiptId": assignment_receipt_id.as_str(),
            "reviewedAtUnix": base + 3,
            "selections": selection_json
        }))
        .unwrap();
        let crate::desktop_api::AppIntent::PreviewCaptureResolution { selections, .. } = intent
        else {
            panic!("resolution selections must retain the strict App type");
        };
        let (resolution, echoed, preview) = state
            .preview_capture_resolution(
                &assignment_receipt_id,
                base + 3,
                selections.expect("explicit selections must remain present"),
            )
            .unwrap();
        let resolution_json = serde_json::to_value((&resolution, &echoed, &preview)).unwrap();
        assert_eq!(
            resolution_json[0]["items"].as_array().unwrap().len(),
            resolution_json[1].as_array().unwrap().len()
        );
        assert_eq!(
            resolution_json[0]["planDigest"],
            resolution_json[2]["planDigestSha256"]
        );
        assert!(
            !resolution_json
                .to_string()
                .contains(project_root.to_string_lossy().as_ref())
        );
        let resolution_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&resolution_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "capture-resolution-completed");
        assert!(matches!(
            confirmed.continuity,
            Some(ConfirmedCaptureContinuity::Resolution(_))
        ));
        assert_eq!(state.snapshot().projects[0].semantic_revision, 2);
        assert_eq!(
            serde_json::to_value(
                state
                    .inspect_capture_assignment(
                        &state
                            .service
                            .as_ref()
                            .unwrap()
                            .list_capture_assignments()
                            .unwrap()[0]
                            .intent_id,
                    )
                    .unwrap(),
            )
            .unwrap()["canResolve"],
            false
        );

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_creates_exports_and_imports_portable_projects() {
        let root = isolated_root("project-portable");
        let source_home = root.join("source-home");
        let source_configured = root.join("source-configured");
        let source_project = root.join("source-project");
        let portable_package = root.join("portable-package");
        let imported_project = root.join("imported-project");
        create_private_directory(&source_home);

        let source_config_root =
            qiongli_config::resolve_config_root(Some(source_configured.as_os_str()), &source_home)
                .unwrap();
        let mut source_state =
            ProjectDesktopState::new(Some(ProjectStateService::new(source_config_root)));

        let (create_directory_token, _) = source_state
            .select_create_root(source_project.clone())
            .unwrap();
        source_state
            .preview_create(
                &create_directory_token,
                "Portable article".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let create_token = source_state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = source_state.confirm(&create_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-creation-completed");
        assert_eq!(confirmed.capture_project_id, None);
        let project_id = source_state.snapshot().projects[0].project_id.clone();

        let (export_directory_token, _) = source_state
            .select_export_destination(project_id.clone(), portable_package.clone())
            .unwrap();
        source_state
            .preview_export(&export_directory_token)
            .unwrap();
        let export_token = source_state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = source_state.confirm(&export_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-export-completed");
        assert_eq!(confirmed.capture_project_id, None);
        assert!(
            portable_package
                .join("qiongli-portable-project.json")
                .is_file()
        );

        let destination_home = root.join("destination-home");
        let destination_configured = root.join("destination-configured");
        create_private_directory(&destination_home);
        let destination_config_root = qiongli_config::resolve_config_root(
            Some(destination_configured.as_os_str()),
            &destination_home,
        )
        .unwrap();
        let mut destination_state =
            ProjectDesktopState::new(Some(ProjectStateService::new(destination_config_root)));

        let (import_directory_token, _) = destination_state
            .select_import_locations(portable_package, imported_project.clone())
            .unwrap();
        destination_state
            .preview_import(&import_directory_token)
            .unwrap();
        let import_token = destination_state
            .pending
            .as_ref()
            .unwrap()
            .token()
            .to_owned();
        let confirmed = destination_state.confirm(&import_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-import-completed");
        assert_eq!(confirmed.capture_project_id, None);

        let imported = destination_state.snapshot();
        assert_eq!(imported.projects.len(), 1);
        assert_eq!(imported.projects[0].project_id, project_id);
        assert!(
            imported_project
                .join("context/project_manifest.json")
                .is_file()
        );

        drop(source_state);
        drop(destination_state);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_migrates_recovers_and_qualifies_graph_rebuilds() {
        let root = isolated_root("project-migration");
        let source = root.join("legacy-project");
        let destination = root.join("migrated-project");
        let first_home = root.join("first-home");
        let first_configured = root.join("first-configured");
        create_private_directory(&first_home);
        create_private_directory(&source);
        create_private_directory(&source.join("context"));
        fs::write(
            source.join("context/research_state.md"),
            b"# Research State\n\nRQ: Can project migration remain restart-safe?\n",
        )
        .unwrap();
        let source_before = fs::read(source.join("context/research_state.md")).unwrap();
        let first_config_root =
            qiongli_config::resolve_config_root(Some(first_configured.as_os_str()), &first_home)
                .unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(first_config_root)));

        let (directory_token, destination_label) = state
            .select_migration_locations(source.clone(), destination.clone())
            .unwrap();
        assert_eq!(destination_label, "migrated-project");
        let preview = state
            .preview_migration(
                &directory_token,
                "Migrated project".to_owned(),
                ProjectKind::Article,
                ProjectStage::Literature,
            )
            .unwrap();
        let preview_json = serde_json::to_value(preview).unwrap();
        assert_eq!(preview_json["kind"], "project-migration");
        assert_eq!(preview_json["displayTarget"], "migrated-project");
        assert!(
            preview_json["summary"]
                .as_str()
                .unwrap()
                .contains("retain the source unchanged")
        );
        assert!(json_string_value_containing(&preview_json, source.to_str().unwrap()).is_none());
        assert!(
            json_string_value_containing(&preview_json, destination.to_str().unwrap()).is_none()
        );

        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&operation_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-migration-completed");
        assert_eq!(confirmed.capture_project_id, None);
        assert!(
            confirmed
                .migration_qualification
                .as_ref()
                .is_some_and(AppProjectMigrationQualification::deterministic_rebuild)
        );
        assert_eq!(
            fs::read(source.join("context/research_state.md")).unwrap(),
            source_before
        );
        assert!(
            destination
                .join(".qiongli/v2/project-migration-registered.json")
                .is_file()
        );
        let project_id = state.snapshot().projects[0].project_id.clone();
        let (first_graph, first_readiness, _) = state.academic_graph(&project_id).unwrap();
        assert_ne!(
            first_readiness.state,
            qiongli_project::AcademicGraphReadinessState::Stale
        );
        drop(state);

        let marker = destination.join(".qiongli/v2/project-migration-registered.json");
        fs::remove_file(&marker).unwrap();
        let recovery_home = root.join("recovery-home");
        let recovery_configured = root.join("recovery-configured");
        create_private_directory(&recovery_home);
        let recovery_config_root = qiongli_config::resolve_config_root(
            Some(recovery_configured.as_os_str()),
            &recovery_home,
        )
        .unwrap();
        let mut recovered =
            ProjectDesktopState::new(Some(ProjectStateService::new(recovery_config_root)));
        let (recovery_directory_token, _) = recovered
            .select_migration_recovery_locations(source.clone(), destination.clone())
            .unwrap();
        let recovery_preview = recovered
            .preview_migration_recovery(&recovery_directory_token)
            .unwrap();
        let recovery_json = serde_json::to_value(recovery_preview).unwrap();
        assert_eq!(recovery_json["kind"], "project-migration-recovery");
        assert!(
            recovery_json["summary"]
                .as_str()
                .unwrap()
                .contains("without copying again")
        );
        let recovery_token = recovered.pending.as_ref().unwrap().token().to_owned();
        let confirmed = recovered.confirm(&recovery_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-migration-recovered");
        assert!(
            confirmed
                .migration_qualification
                .as_ref()
                .is_some_and(AppProjectMigrationQualification::deterministic_rebuild)
        );
        assert_eq!(recovered.snapshot().projects.len(), 1);
        assert!(marker.is_file());
        let (recovered_graph, recovered_readiness, _) =
            recovered.academic_graph(&project_id).unwrap();
        assert_ne!(
            recovered_readiness.state,
            qiongli_project::AcademicGraphReadinessState::Stale
        );
        assert_eq!(recovered_graph.projection_id, first_graph.projection_id);
        assert_eq!(
            recovered_graph.projection_digest,
            first_graph.projection_digest
        );

        let (rollback_directory_token, _) = recovered
            .select_migration_rollback_locations(source.clone(), destination.clone())
            .unwrap();
        let rollback_preview = recovered
            .preview_migration_rollback(&rollback_directory_token)
            .unwrap();
        let rollback_json = serde_json::to_value(rollback_preview).unwrap();
        assert_eq!(rollback_json["kind"], "project-migration-rollback");
        assert_eq!(rollback_json["canConfirm"], true);
        assert_eq!(rollback_json["migrationRollback"]["sourceRetained"], true);
        assert_eq!(
            rollback_json["migrationRollback"]["reconciliation"]["driftedArtifactCount"],
            0
        );
        assert!(json_string_value_containing(&rollback_json, source.to_str().unwrap()).is_none());
        assert!(
            json_string_value_containing(&rollback_json, destination.to_str().unwrap()).is_none()
        );
        let rollback_token = recovered.pending.as_ref().unwrap().token().to_owned();
        let confirmed = recovered.confirm(&rollback_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "project-migration-rolled-back");
        assert!(confirmed.migration_qualification.is_none());
        assert!(recovered.snapshot().projects.is_empty());
        assert!(!destination.exists());
        assert_eq!(
            fs::read(source.join("context/research_state.md")).unwrap(),
            source_before
        );

        drop(recovered);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_desktop_state_intakes_reads_and_consolidates_capture_without_exposing_path() {
        let root = isolated_root("capture-inbox");
        let home = root.join("home");
        let configured = root.join("configured");
        let project_root = root.join("article-project");
        let capture_path = root.join("private-capture-packet.json");
        create_private_directory(&home);
        let config_root =
            qiongli_config::resolve_config_root(Some(configured.as_os_str()), &home).unwrap();
        let mut state = ProjectDesktopState::new(Some(ProjectStateService::new(config_root)));

        let (create_token, _) = state.select_create_root(project_root).unwrap();
        state
            .preview_create(
                &create_token,
                "Capture article".to_owned(),
                ProjectKind::Article,
                ProjectStage::Idea,
            )
            .unwrap();
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        state.confirm(&operation_token).unwrap().unwrap();
        let project_id = state.snapshot().projects[0].project_id.clone();

        let capture = qiongli_project::ResearchCaptureDraftV1 {
            binding: qiongli_project::ProjectBindingV1::new(
                project_id.clone(),
                1,
                ProjectStage::Idea,
                "Refine the article framing",
                qiongli_project::CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: qiongli_project::CaptureSource::Codex,
            delivery: qiongli_project::CaptureDelivery::Portable,
            captured_at_unix: now_unix().unwrap(),
            summary: "Separate the literature synthesis from the working thesis.".to_owned(),
            changes: vec![qiongli_project::SemanticChangeV1 {
                area: qiongli_project::CaptureArea::Literature,
                summary: "Organize the literature around cross-client research continuity."
                    .to_owned(),
            }],
            decisions: vec![qiongli_project::DecisionCandidateV1 {
                relation: qiongli_project::DecisionRelation::Candidate,
                statement: "Treat the article project as the durable unit.".to_owned(),
                rationale: "Sessions remain execution context rather than research memory."
                    .to_owned(),
                target: None,
            }],
            evidence: vec![qiongli_project::EvidenceReferenceV1 {
                locator_kind: qiongli_project::EvidenceLocatorKind::Doi,
                locator: "10.1000/capture-inbox".to_owned(),
                relevance: "Provides a bounded citation anchor for the refinement.".to_owned(),
                limitation: Some("Architecture evidence only.".to_owned()),
            }],
            contradictions: Vec::new(),
            next_actions: vec!["Review the framing against current literature.".to_owned()],
        }
        .into_capture()
        .unwrap();
        fs::write(&capture_path, capture.to_canonical_json().unwrap()).unwrap();

        let (file_token, file_label) = state
            .select_capture_file(project_id.clone(), capture_path.clone())
            .unwrap();
        assert_eq!(file_label, "private-capture-packet.json");
        assert!(!file_token.contains("private-capture-packet"));
        let (intake, preview) = state.preview_capture_intake(&file_token).unwrap();
        assert_eq!(
            intake.effect,
            qiongli_project::CaptureIntakeEffect::AppendPendingHistory
        );
        let preview_json = serde_json::to_value(&preview).unwrap();
        assert_eq!(preview_json["canConfirm"], true);
        assert!(!format!("{preview:?}").contains(&capture_path.to_string_lossy().to_string()));

        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&operation_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "capture-intake-completed");
        assert_eq!(confirmed.capture_project_id, Some(project_id.clone()));
        let inbox = state.capture_inbox(&project_id).unwrap();
        assert_eq!(inbox.pending_review_count, 1);
        assert_eq!(inbox.entries[0].capture_id, capture.capture_id);

        let read = state
            .read_capture(&project_id, &capture.capture_id)
            .unwrap();
        let read_json = serde_json::to_value(read).unwrap();
        assert_eq!(read_json["schemaVersion"], 1);
        assert_eq!(read_json["binding"]["projectId"], project_id.as_str());
        assert!(read_json.get("document_kind").is_none());
        assert!(
            !read_json
                .to_string()
                .contains(&capture_path.to_string_lossy().to_string())
        );

        let (consolidation, preview) = state
            .preview_capture_consolidation(&project_id, &capture.capture_id)
            .unwrap();
        assert_eq!(
            consolidation.outcome,
            qiongli_project::CaptureConsolidationOutcome::Ready
        );
        let preview_json = serde_json::to_value(&preview).unwrap();
        assert_eq!(preview_json["canConfirm"], true);
        assert_eq!(
            preview_json["approvalsRequired"],
            serde_json::json!(["academic-consolidation", "filesystem-write"])
        );
        let operation_token = state.pending.as_ref().unwrap().token().to_owned();
        let confirmed = state.confirm(&operation_token).unwrap().unwrap();
        assert_eq!(confirmed.code, "capture-consolidation-completed");
        assert_eq!(confirmed.capture_project_id, Some(project_id.clone()));
        let inbox = state.capture_inbox(&project_id).unwrap();
        assert_eq!(inbox.applied_count, 1);
        assert_eq!(inbox.project_revision, 2);

        drop(state);
        fs::remove_dir_all(root).unwrap();
    }

    fn isolated_root(name: &str) -> PathBuf {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let fixture_base = native_root.join("target/qiongli-desktop-service-tests");
        fs::create_dir_all(&fixture_base).expect("desktop fixture base must be created");
        let root = fixture_base.join(format!(
            "qiongli-r3f-{name}-{}-{sequence}",
            std::process::id()
        ));
        create_private_directory(&root);
        root
    }

    fn wait_for_portfolio_completion(
        state: &ProjectDesktopState,
        operation_id: &str,
    ) -> AppPortfolioMaintenanceResultV1 {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match state.poll_continuity_operation(operation_id).unwrap() {
                DesktopContinuityPoll::Completed(result) => return result,
                DesktopContinuityPoll::Progress(progress) => {
                    let progress = serde_json::to_value(progress).unwrap();
                    assert!(
                        matches!(progress["phase"].as_str(), Some("queued" | "running")),
                        "portfolio operation terminated without a completion result: {}",
                        progress["phase"]
                    );
                    thread::sleep(Duration::from_millis(1));
                }
            }
        }
        panic!("portfolio operation did not complete within the bounded test poll");
    }

    fn wait_for_portfolio_cancellation(
        state: &ProjectDesktopState,
        operation_id: &str,
    ) -> AppContinuityOperationProgressV1 {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match state.poll_continuity_operation(operation_id).unwrap() {
                DesktopContinuityPoll::Progress(progress)
                    if serde_json::to_value(&progress).unwrap()["phase"] == "cancelled" =>
                {
                    return progress;
                }
                DesktopContinuityPoll::Progress(progress) => {
                    let progress = serde_json::to_value(progress).unwrap();
                    assert!(
                        matches!(progress["phase"].as_str(), Some("queued" | "running")),
                        "portfolio cancellation terminated unexpectedly: {}",
                        progress["phase"]
                    );
                    thread::sleep(Duration::from_millis(1));
                }
                DesktopContinuityPoll::Completed(_) => {
                    panic!("cancelled portfolio work must not complete")
                }
            }
        }
        panic!("portfolio cancellation did not finish within the bounded test poll");
    }

    fn collect_directory_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn collect(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
            let mut entries = fs::read_dir(directory)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    collect(root, &path, files);
                } else if path.is_file() {
                    files.insert(
                        path.strip_prefix(root).unwrap().to_path_buf(),
                        fs::read(path).unwrap(),
                    );
                }
            }
        }

        let mut files = BTreeMap::new();
        collect(root, root, &mut files);
        files
    }

    #[cfg(unix)]
    fn create_private_directory(path: &Path) {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder
            .create(path)
            .expect("private directory must be created");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .expect("private directory mode must be retained");
    }

    #[cfg(windows)]
    fn create_private_directory(path: &Path) {
        qiongli_windows_security::create_owner_only_directory(path)
            .expect("owner-only Windows directory must be created");
    }

    #[cfg(not(any(unix, windows)))]
    fn create_private_directory(path: &Path) {
        fs::create_dir(path).expect("private directory must be created");
    }
}
