use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::{
    AcademicGraphDirection, AcademicGraphIndexService, AcademicGraphLayer, AcademicGraphNodeType,
    AcademicGraphPortfolioService, AcademicGraphQueryV1, AcademicGraphRelation,
    AcademicGraphService, ApprovedProjectMutation, PortableProjectPreviewV1, ProjectId,
    ProjectKind, ProjectMigrationDoctorV1, ProjectMigrationPreviewV1,
    ProjectMigrationRecoveryPreviewV1, ProjectMigrationRollbackPreviewV1, ProjectMutationPreviewV1,
    ProjectRegistrationOptions, ProjectStage, ProjectStateService, ResearchLibrarySnapshotV1,
};
use serde::Serialize;

use crate::command::{CliOutput, CommandEnvironment, config_root};

pub(crate) const PROJECT_USAGE: &str = "Qiongli Research Library\n\nUsage:\n  qiongli project list\n  qiongli project show --project-id <prj_id>\n  qiongli project graph snapshot --project-id <prj_id>\n  qiongli project graph portfolio\n  qiongli project graph query --project-id <prj_id> --expected-projection-id <grp_id> [filters]\n  qiongli project graph doctor --project-id <prj_id>\n  qiongli project portfolio <status|reconcile|rebuild|delete-derived-state|query|timeline|doctor>\n  qiongli project doctor\n  qiongli project doctor repair <preview|apply> --project-id <prj_id> [--expected-plan-digest <sha256> --approve-filesystem-write]\n  qiongli project create preview --root <absolute-path> --name <name> [--kind <article|review|dissertation-article|manuscript>] [--stage <stage>] [--project-id <prj_id>]\n  qiongli project create apply --root <absolute-path> --name <name> [--kind <kind>] [--stage <stage>] --project-id <prj_id> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project register preview --root <absolute-path> [--name <name>] [--kind <kind>] [--stage <stage>] [--project-id <prj_id>]\n  qiongli project register apply --root <absolute-path> [--name <name>] [--kind <kind>] [--stage <stage>] [--project-id <prj_id>] --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project export <preview|apply> --project-id <prj_id> --destination <absolute-path> [--expected-plan-digest <sha256> --approve-filesystem-write]\n  qiongli project import <preview|apply> --source <absolute-path> --root <absolute-path> [--expected-plan-digest <sha256> --approve-filesystem-write]\n  qiongli project migrate preview --source <legacy-absolute-path> --root <new-absolute-path> [--name <name>] [--kind <kind>] [--stage <stage>] [--project-id <prj_id>] [--manifest-created-at-unix <timestamp>]\n  qiongli project migrate apply --source <legacy-absolute-path> --root <new-absolute-path> [--name <name>] [--kind <kind>] [--stage <stage>] --project-id <prj_id> --manifest-created-at-unix <timestamp> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project migrate recover preview --source <legacy-absolute-path> --root <committed-2x-path>\n  qiongli project migrate recover apply --source <legacy-absolute-path> --root <committed-2x-path> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project migrate rollback preview --source <legacy-absolute-path> --root <migration-owned-2x-path>\n  qiongli project migrate rollback apply --source <legacy-absolute-path> --root <migration-owned-2x-path> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project <archive|restore|refresh|unregister> preview --project-id <prj_id>\n  qiongli project <archive|restore|refresh|unregister> apply --project-id <prj_id> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project --help\n\nGraph filters:\n  --focus-node-id <nod_id> --direction <incoming|outgoing|both>\n  --node-type <type> --relation <relation> --layer <layer>\n  --canonical-id <id> --text <text> --max-nodes <1..256> --max-edges <1..512>\n\nPortable export format:\n  A private directory package containing qiongli-portable-project.json and project/.\n  Absolute paths, client configuration, recognizable credential files, sessions, chats, and transcripts are excluded.\n\nLegacy project migration:\n  Copies bounded academic files into a new 2.x project and leaves the source untouched.\n  Legacy .qiongli runtime state and recognizable credential/session files are not copied.\n  Apply must reuse the projectId, manifestCreatedAtUnix, and planDigest returned by preview.\n  Recover resumes an exact committed copy after a process interruption without copying again.\n  Rollback reconciles every copied artifact, refuses destination drift, unregisters the project,\n  and removes only the exact receipt-owned 2.x destination while retaining the 1.x source.\n\nStages:\n  idea | framing | literature | design | analysis | writing | review | submission\n";
pub(crate) const GRAPH_NEIGHBOURHOOD_USAGE: &str =
    "Graph neighbourhood:\n  --max-depth <1..3> requires --focus-node-id and defaults to 1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProjectCliCommand {
    Help,
    List,
    Show(ProjectId),
    GraphSnapshot(ProjectId),
    GraphPortfolio,
    GraphQuery(ProjectGraphQueryOptions),
    GraphDoctor(ProjectId),
    Portfolio(Box<crate::portfolio_cli::PortfolioCliCommand>),
    Doctor,
    PreviewDoctorRepair(ProjectId),
    ApplyDoctorRepair(ProjectId, String),
    Capture(crate::capture_cli::CaptureCliCommand),
    PreviewCreate(ProjectPathOptions),
    ApplyCreate(ProjectPathOptions, String),
    PreviewRegister(ProjectPathOptions),
    ApplyRegister(ProjectPathOptions, String),
    PreviewExport(ProjectExportOptions),
    ApplyExport(ProjectExportOptions, String),
    PreviewImport(ProjectImportOptions),
    ApplyImport(ProjectImportOptions, String),
    PreviewMigration(ProjectMigrationOptions),
    ApplyMigration(ProjectMigrationOptions, String),
    PreviewMigrationRecovery(ProjectMigrationRecoveryOptions),
    ApplyMigrationRecovery(ProjectMigrationRecoveryOptions, String),
    PreviewMigrationRollback(ProjectMigrationRecoveryOptions),
    ApplyMigrationRollback(ProjectMigrationRecoveryOptions, String),
    PreviewLifecycle(ProjectLifecycleCommand, ProjectId),
    ApplyLifecycle(ProjectLifecycleCommand, ProjectId, String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectGraphQueryOptions {
    project_id: ProjectId,
    query: AcademicGraphQueryV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectExportOptions {
    project_id: ProjectId,
    destination: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectImportOptions {
    source: PathBuf,
    root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectMigrationOptions {
    source: PathBuf,
    root: PathBuf,
    project_id: Option<ProjectId>,
    manifest_created_at_unix: Option<u64>,
    display_name: Option<String>,
    project_kind: Option<ProjectKind>,
    stage: Option<ProjectStage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectMigrationRecoveryOptions {
    source: PathBuf,
    root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectPathOptions {
    root: PathBuf,
    project_id: Option<ProjectId>,
    display_name: Option<String>,
    project_kind: Option<ProjectKind>,
    stage: Option<ProjectStage>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectLifecycleCommand {
    Archive,
    Restore,
    Refresh,
    Unregister,
}

pub(crate) fn parse(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a project subcommand is required");
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(ProjectCliCommand::Help),
        "list" | "ls" if args.len() == 1 => Ok(ProjectCliCommand::List),
        "doctor" => parse_doctor(&args[1..]),
        "show" if args.len() == 2 => parse_project_id(&args[1]).map(ProjectCliCommand::Show),
        "show" => parse_project_id_only(&args[1..]).map(ProjectCliCommand::Show),
        "graph" => parse_graph(&args[1..]),
        "portfolio" => crate::portfolio_cli::parse(&args[1..])
            .map(Box::new)
            .map(ProjectCliCommand::Portfolio),
        "capture" => crate::capture_cli::parse(&args[1..]).map(ProjectCliCommand::Capture),
        "create" => parse_path_mutation(&args[1..], true),
        "register" => parse_path_mutation(&args[1..], false),
        "export" => parse_portable_export(&args[1..]),
        "import" => parse_portable_import(&args[1..]),
        "migrate" => parse_project_migration(&args[1..]),
        "archive" => parse_lifecycle(&args[1..], ProjectLifecycleCommand::Archive),
        "restore" => parse_lifecycle(&args[1..], ProjectLifecycleCommand::Restore),
        "refresh" => parse_lifecycle(&args[1..], ProjectLifecycleCommand::Refresh),
        "unregister" => parse_lifecycle(&args[1..], ProjectLifecycleCommand::Unregister),
        "--help" | "list" => Err("unexpected project argument"),
        _ => Err("unknown project subcommand"),
    }
}

pub(crate) fn execute(command: ProjectCliCommand, environment: &CommandEnvironment) -> CliOutput {
    if command == ProjectCliCommand::Help {
        return CliOutput::success_text(format!(
            "{PROJECT_USAGE}\n{GRAPH_NEIGHBOURHOOD_USAGE}\n{}\n{}\n{}\n{}\n{}\n{}",
            crate::portfolio_cli::USAGE,
            crate::capture_cli::CAPTURE_USAGE,
            crate::capture_delivery_cli::USAGE,
            crate::capture_assignment_cli::USAGE,
            crate::capture_resolution_cli::USAGE,
            crate::repository_capture_cli::USAGE
        ));
    }
    if command == ProjectCliCommand::Capture(crate::capture_cli::CaptureCliCommand::Help) {
        return CliOutput::success_text(format!(
            "{}\n{}\n{}\n{}\n{}",
            crate::capture_cli::CAPTURE_USAGE,
            crate::capture_delivery_cli::USAGE,
            crate::capture_assignment_cli::USAGE,
            crate::capture_resolution_cli::USAGE,
            crate::repository_capture_cli::USAGE
        ));
    }
    let root = match config_root(environment) {
        Ok(root) => root,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let service = ProjectStateService::new(root);
    let command = match command {
        ProjectCliCommand::Portfolio(command) => {
            return crate::portfolio_cli::execute(*command, &service);
        }
        command => command,
    };
    let output = match command {
        ProjectCliCommand::Help => unreachable!("help returns before service creation"),
        ProjectCliCommand::List => service.snapshot().map(|library| {
            ProjectCliOutput::Library(ProjectListOutput {
                schema_version: 1,
                command: "project-list",
                library,
            })
        }),
        ProjectCliCommand::Show(project_id) => service.snapshot().and_then(|library| {
            let project = library
                .projects
                .into_iter()
                .find(|project| project.project_id == project_id)
                .ok_or(qiongli_project::ProjectError::ProjectNotRegistered)?;
            Ok(ProjectCliOutput::Project(ProjectShowOutput {
                schema_version: 1,
                command: "project-show",
                library_revision: library.revision,
                project,
            }))
        }),
        ProjectCliCommand::GraphSnapshot(project_id) => AcademicGraphService::new(service.clone())
            .rebuild_projection(&project_id)
            .map(|projection| {
                ProjectCliOutput::GraphSnapshot(ProjectGraphSnapshotOutput {
                    schema_version: 1,
                    command: "project-graph-snapshot",
                    snapshot: projection.graph,
                    readiness: projection.readiness,
                })
            }),
        ProjectCliCommand::GraphPortfolio => AcademicGraphPortfolioService::new(service.clone())
            .rebuild()
            .map(|portfolio| {
                ProjectCliOutput::GraphPortfolio(ProjectGraphPortfolioOutput {
                    schema_version: 1,
                    command: "project-graph-portfolio",
                    portfolio,
                })
            }),
        ProjectCliCommand::GraphQuery(options) => AcademicGraphService::new(service.clone())
            .rebuild_projection(&options.project_id)
            .and_then(|projection| {
                AcademicGraphIndexService::new(service.clone())
                    .rebuild(&options.project_id)
                    .and_then(|index| index.query(&options.query))
                    .map(|result| {
                        ProjectCliOutput::GraphQuery(ProjectGraphQueryOutput {
                            schema_version: 1,
                            command: "project-graph-query",
                            result,
                            readiness: projection.readiness,
                        })
                    })
            }),
        ProjectCliCommand::GraphDoctor(project_id) => {
            let graph_index = AcademicGraphIndexService::new(service.clone());
            AcademicGraphService::new(service.clone())
                .rebuild_projection(&project_id)
                .and_then(|projection| {
                    graph_index.rebuild(&project_id).and_then(|first| {
                        let rebuilt = graph_index.rebuild(&project_id)?;
                        if first.index_id != rebuilt.index_id
                            || first.projection_id != rebuilt.projection_id
                            || first.project_revision != rebuilt.project_revision
                            || first.node_count != rebuilt.node_count
                            || first.edge_count != rebuilt.edge_count
                        {
                            return Err(qiongli_project::ProjectError::RevisionConflict);
                        }
                        Ok(ProjectCliOutput::GraphDoctor(ProjectGraphDoctorOutput {
                            schema_version: 1,
                            command: "project-graph-doctor",
                            project_id: first.project_id,
                            project_revision: first.project_revision,
                            projection_id: first.projection_id,
                            index_id: first.index_id,
                            node_count: first.node_count,
                            edge_count: first.edge_count,
                            deterministic_rebuild: true,
                            persistent_index_state: "none",
                            portable_authority: false,
                            readiness: projection.readiness,
                        }))
                    })
                })
        }
        ProjectCliCommand::Portfolio(_) => {
            unreachable!("portfolio commands return before project dispatch")
        }
        ProjectCliCommand::Doctor => service.snapshot().and_then(|library| {
            let migration_diagnostics = service.migration_doctor()?;
            let blocking = library
                .projects
                .iter()
                .filter(|project| project.health != qiongli_project::ProjectHealth::Ready)
                .count();
            let migration_attention = migration_diagnostics
                .iter()
                .filter(|diagnostic| {
                    diagnostic.status == qiongli_project::ProjectMigrationDoctorStatus::Attention
                })
                .count();
            Ok(ProjectCliOutput::Doctor(ProjectDoctorOutput {
                schema_version: 1,
                command: "project-doctor",
                status: if blocking == 0 && migration_attention == 0 {
                    "ready"
                } else {
                    "attention"
                },
                blocking_projects: blocking,
                migration_attention,
                migration_diagnostics,
                library,
            }))
        }),
        ProjectCliCommand::PreviewDoctorRepair(project_id) => {
            service.preview_repair_manifest(&project_id).map(|plan| {
                ProjectCliOutput::Preview(ProjectPreviewOutput {
                    schema_version: 1,
                    command: "project-doctor-repair-preview",
                    preview: plan.preview().clone(),
                })
            })
        }
        ProjectCliCommand::ApplyDoctorRepair(project_id, digest) => {
            let now = match now_unix() {
                Ok(now) => now,
                Err(error) => return CliOutput::operation_failure(error),
            };
            service
                .preview_repair_manifest(&project_id)
                .and_then(|plan| {
                    service.apply(&plan, &ApprovedProjectMutation::new(digest, true), now)
                })
                .map(|commit| {
                    ProjectCliOutput::Commit(ProjectCommitOutput {
                        schema_version: 1,
                        command: "project-doctor-repair-apply",
                        commit,
                    })
                })
        }
        ProjectCliCommand::Capture(command) => crate::capture_cli::execute(command, &service)
            .map(|output| ProjectCliOutput::Capture(Box::new(output))),
        ProjectCliCommand::PreviewCreate(options) => {
            preview_path(&service, options, true).map(|preview| {
                ProjectCliOutput::Preview(ProjectPreviewOutput {
                    schema_version: 1,
                    command: "project-create-preview",
                    preview,
                })
            })
        }
        ProjectCliCommand::PreviewRegister(options) => {
            preview_path(&service, options, false).map(|preview| {
                ProjectCliOutput::Preview(ProjectPreviewOutput {
                    schema_version: 1,
                    command: "project-register-preview",
                    preview,
                })
            })
        }
        ProjectCliCommand::ApplyCreate(options, digest) => {
            apply_path(&service, options, true, digest).map(|commit| {
                ProjectCliOutput::Commit(ProjectCommitOutput {
                    schema_version: 1,
                    command: "project-create-apply",
                    commit,
                })
            })
        }
        ProjectCliCommand::ApplyRegister(options, digest) => {
            apply_path(&service, options, false, digest).map(|commit| {
                ProjectCliOutput::Commit(ProjectCommitOutput {
                    schema_version: 1,
                    command: "project-register-apply",
                    commit,
                })
            })
        }
        ProjectCliCommand::PreviewExport(options) => service
            .preview_export(&options.project_id, &options.destination)
            .map(|plan| {
                ProjectCliOutput::PortablePreview(ProjectPortablePreviewOutput {
                    schema_version: 1,
                    command: "project-export-preview",
                    preview: plan.preview().clone(),
                })
            }),
        ProjectCliCommand::ApplyExport(options, digest) => {
            let now = match now_unix() {
                Ok(now) => now,
                Err(error) => return CliOutput::operation_failure(error),
            };
            service
                .preview_export(&options.project_id, &options.destination)
                .and_then(|plan| {
                    service.apply_portable(&plan, &ApprovedProjectMutation::new(digest, true), now)
                })
                .map(|commit| {
                    ProjectCliOutput::PortableCommit(ProjectPortableCommitOutput {
                        schema_version: 1,
                        command: "project-export-apply",
                        commit,
                    })
                })
        }
        ProjectCliCommand::PreviewImport(options) => service
            .preview_import(&options.source, &options.root)
            .map(|plan| {
                ProjectCliOutput::PortablePreview(ProjectPortablePreviewOutput {
                    schema_version: 1,
                    command: "project-import-preview",
                    preview: plan.preview().clone(),
                })
            }),
        ProjectCliCommand::ApplyImport(options, digest) => {
            let now = match now_unix() {
                Ok(now) => now,
                Err(error) => return CliOutput::operation_failure(error),
            };
            service
                .preview_import(&options.source, &options.root)
                .and_then(|plan| {
                    service.apply_portable(&plan, &ApprovedProjectMutation::new(digest, true), now)
                })
                .map(|commit| {
                    ProjectCliOutput::PortableCommit(ProjectPortableCommitOutput {
                        schema_version: 1,
                        command: "project-import-apply",
                        commit,
                    })
                })
        }
        ProjectCliCommand::PreviewMigration(options) => {
            preview_migration(&service, options).map(|preview| {
                ProjectCliOutput::MigrationPreview(ProjectMigrationPreviewOutput {
                    schema_version: 1,
                    command: "project-migrate-preview",
                    preview,
                })
            })
        }
        ProjectCliCommand::ApplyMigration(options, digest) => {
            apply_migration(&service, options, digest).map(|commit| {
                ProjectCliOutput::MigrationCommit(ProjectMigrationCommitOutput {
                    schema_version: 1,
                    command: "project-migrate-apply",
                    commit,
                })
            })
        }
        ProjectCliCommand::PreviewMigrationRecovery(options) => service
            .preview_migration_recovery(&options.source, &options.root)
            .map(|plan| {
                ProjectCliOutput::MigrationRecoveryPreview(ProjectMigrationRecoveryPreviewOutput {
                    schema_version: 1,
                    command: "project-migrate-recover-preview",
                    preview: plan.preview().clone(),
                })
            }),
        ProjectCliCommand::ApplyMigrationRecovery(options, digest) => service
            .preview_migration_recovery(&options.source, &options.root)
            .and_then(|plan| {
                service.apply_migration_recovery(&plan, &ApprovedProjectMutation::new(digest, true))
            })
            .map(|commit| {
                ProjectCliOutput::MigrationCommit(ProjectMigrationCommitOutput {
                    schema_version: 1,
                    command: "project-migrate-recover-apply",
                    commit,
                })
            }),
        ProjectCliCommand::PreviewMigrationRollback(options) => service
            .preview_migration_rollback(&options.source, &options.root)
            .map(|plan| {
                ProjectCliOutput::MigrationRollbackPreview(ProjectMigrationRollbackPreviewOutput {
                    schema_version: 1,
                    command: "project-migrate-rollback-preview",
                    preview: plan.preview().clone(),
                })
            }),
        ProjectCliCommand::ApplyMigrationRollback(options, digest) => service
            .preview_migration_rollback(&options.source, &options.root)
            .and_then(|plan| {
                service.apply_migration_rollback(&plan, &ApprovedProjectMutation::new(digest, true))
            })
            .map(|commit| {
                ProjectCliOutput::MigrationRollbackCommit(ProjectMigrationRollbackCommitOutput {
                    schema_version: 1,
                    command: "project-migrate-rollback-apply",
                    commit,
                })
            }),
        ProjectCliCommand::PreviewLifecycle(operation, project_id) => {
            preview_lifecycle(&service, operation, &project_id).map(|preview| {
                ProjectCliOutput::Preview(ProjectPreviewOutput {
                    schema_version: 1,
                    command: lifecycle_preview_command(operation),
                    preview,
                })
            })
        }
        ProjectCliCommand::ApplyLifecycle(operation, project_id, digest) => {
            let now = match now_unix() {
                Ok(now) => now,
                Err(error) => return CliOutput::operation_failure(error),
            };
            preview_lifecycle_plan(&service, operation, &project_id, now)
                .and_then(|plan| {
                    service.apply(&plan, &ApprovedProjectMutation::new(digest, true), now)
                })
                .map(|commit| {
                    ProjectCliOutput::Commit(ProjectCommitOutput {
                        schema_version: 1,
                        command: lifecycle_apply_command(operation),
                        commit,
                    })
                })
        }
    };
    match output {
        Ok(output) => json_output(&output),
        Err(error) => CliOutput::operation_failure(error.reason_code()),
    }
}

fn preview_path(
    service: &ProjectStateService,
    options: ProjectPathOptions,
    create: bool,
) -> Result<ProjectMutationPreviewV1, qiongli_project::ProjectError> {
    let now = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    let root = options.root.clone();
    let registration = registration_options(options);
    let plan = if create {
        service.preview_create(root, registration, now)?
    } else {
        service.preview_register(root, registration, now)?
    };
    Ok(plan.preview().clone())
}

fn apply_path(
    service: &ProjectStateService,
    options: ProjectPathOptions,
    create: bool,
    digest: String,
) -> Result<qiongli_project::ProjectMutationCommitV1, qiongli_project::ProjectError> {
    let now = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    let root = options.root.clone();
    let registration = registration_options(options);
    let plan = if create {
        service.preview_create(root, registration, now)?
    } else {
        service.preview_register(root, registration, now)?
    };
    service.apply(&plan, &ApprovedProjectMutation::new(digest, true), now)
}

fn registration_options(options: ProjectPathOptions) -> ProjectRegistrationOptions {
    ProjectRegistrationOptions {
        project_id: options.project_id,
        display_name: options.display_name,
        project_kind: options.project_kind,
        stage: options.stage,
    }
}

fn preview_migration(
    service: &ProjectStateService,
    options: ProjectMigrationOptions,
) -> Result<ProjectMigrationPreviewV1, qiongli_project::ProjectError> {
    let manifest_created_at_unix = options
        .manifest_created_at_unix
        .map_or_else(now_unix, Ok)
        .map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    let registration = migration_registration_options(&options);
    let plan = service.preview_migrate(
        &options.source,
        &options.root,
        registration,
        manifest_created_at_unix,
    )?;
    Ok(plan.preview().clone())
}

fn apply_migration(
    service: &ProjectStateService,
    options: ProjectMigrationOptions,
    digest: String,
) -> Result<qiongli_project::ProjectMigrationCommitV1, qiongli_project::ProjectError> {
    let applied_at_unix = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    let manifest_created_at_unix = options
        .manifest_created_at_unix
        .expect("migration apply parser requires the previewed manifest timestamp");
    let registration = migration_registration_options(&options);
    let plan = service.preview_migrate(
        &options.source,
        &options.root,
        registration,
        manifest_created_at_unix,
    )?;
    service.apply_migration(
        &plan,
        &ApprovedProjectMutation::new(digest, true),
        applied_at_unix,
    )
}

fn migration_registration_options(options: &ProjectMigrationOptions) -> ProjectRegistrationOptions {
    ProjectRegistrationOptions {
        project_id: options.project_id.clone(),
        display_name: options.display_name.clone(),
        project_kind: options.project_kind,
        stage: options.stage,
    }
}

fn preview_lifecycle(
    service: &ProjectStateService,
    operation: ProjectLifecycleCommand,
    project_id: &ProjectId,
) -> Result<ProjectMutationPreviewV1, qiongli_project::ProjectError> {
    let now = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    Ok(preview_lifecycle_plan(service, operation, project_id, now)?
        .preview()
        .clone())
}

fn preview_lifecycle_plan(
    service: &ProjectStateService,
    operation: ProjectLifecycleCommand,
    project_id: &ProjectId,
    now_unix: u64,
) -> Result<qiongli_project::VerifiedProjectMutation, qiongli_project::ProjectError> {
    match operation {
        ProjectLifecycleCommand::Archive => service.preview_archive(project_id),
        ProjectLifecycleCommand::Restore => service.preview_restore(project_id),
        ProjectLifecycleCommand::Refresh => service.preview_refresh(project_id, now_unix),
        ProjectLifecycleCommand::Unregister => service.preview_unregister(project_id),
    }
}

fn parse_path_mutation(args: &[OsString], create: bool) -> Result<ProjectCliCommand, &'static str> {
    let Some(mode) = args.first().and_then(|value| value.to_str()) else {
        return Err("project mutation mode is required");
    };
    let apply = match mode {
        "preview" => false,
        "apply" => true,
        _ => return Err("project mutation mode must be preview or apply"),
    };
    let mut root = None;
    let mut project_id = None;
    let mut display_name = None;
    let mut project_kind = None;
    let mut stage = None;
    let mut expected_plan_digest = None;
    let mut approved = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("project option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("project option value is required")?;
        match option {
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--project-id" if project_id.is_none() => {
                project_id = Some(parse_project_id(value)?);
            }
            "--name" if display_name.is_none() => {
                display_name = Some(parse_display_name(value)?);
            }
            "--kind" if project_kind.is_none() => {
                project_kind = Some(parse_project_kind(value)?);
            }
            "--stage" if stage.is_none() => stage = Some(parse_project_stage(value)?),
            "--expected-plan-digest" if apply && expected_plan_digest.is_none() => {
                expected_plan_digest = Some(parse_sha256(value)?);
            }
            "--root"
            | "--project-id"
            | "--name"
            | "--kind"
            | "--stage"
            | "--expected-plan-digest" => return Err("project option is unexpected or duplicate"),
            _ => return Err("unknown project option"),
        }
        index += 2;
    }
    if apply && (!approved || expected_plan_digest.is_none()) {
        return Err("project apply requires plan digest and filesystem approval");
    }
    if create && display_name.is_none() {
        return Err("project create requires a display name");
    }
    let options = ProjectPathOptions {
        root: root.ok_or("project root is required")?,
        project_id,
        display_name,
        project_kind,
        stage,
    };
    match (create, apply) {
        (true, false) => Ok(ProjectCliCommand::PreviewCreate(options)),
        (true, true) => Ok(ProjectCliCommand::ApplyCreate(
            options,
            expected_plan_digest.expect("validated above"),
        )),
        (false, false) => Ok(ProjectCliCommand::PreviewRegister(options)),
        (false, true) => Ok(ProjectCliCommand::ApplyRegister(
            options,
            expected_plan_digest.expect("validated above"),
        )),
    }
}

fn parse_doctor(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    if args.is_empty() {
        return Ok(ProjectCliCommand::Doctor);
    }
    if args.first() != Some(&OsString::from("repair")) {
        return Err("unknown project doctor subcommand");
    }
    let (apply, project_id, digest) = parse_identity_mutation(&args[1..])?;
    if apply {
        Ok(ProjectCliCommand::ApplyDoctorRepair(
            project_id,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewDoctorRepair(project_id))
    }
}

fn parse_portable_export(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut project_id = None;
    let mut destination = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project export option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project export option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => project_id = Some(parse_project_id(value)?),
            "--destination" if destination.is_none() => {
                destination = Some(PathBuf::from(value));
            }
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--project-id" | "--destination" | "--expected-plan-digest" => {
                return Err("project export option is unexpected or duplicate");
            }
            _ => return Err("unknown project export option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    let options = ProjectExportOptions {
        project_id: project_id.ok_or("project ID is required")?,
        destination: destination.ok_or("project export destination is required")?,
    };
    if apply {
        Ok(ProjectCliCommand::ApplyExport(
            options,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewExport(options))
    }
}

fn parse_portable_import(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut source = None;
    let mut root = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project import option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project import option value is required")?;
        match option {
            "--source" if source.is_none() => source = Some(PathBuf::from(value)),
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--source" | "--root" | "--expected-plan-digest" => {
                return Err("project import option is unexpected or duplicate");
            }
            _ => return Err("unknown project import option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    let options = ProjectImportOptions {
        source: source.ok_or("project import source is required")?,
        root: root.ok_or("project import root is required")?,
    };
    if apply {
        Ok(ProjectCliCommand::ApplyImport(
            options,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewImport(options))
    }
}

fn parse_project_migration(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    if args.first().and_then(|value| value.to_str()) == Some("rollback") {
        return parse_project_migration_rollback(&args[1..]);
    }
    if args.first().and_then(|value| value.to_str()) == Some("recover") {
        return parse_project_migration_recovery(&args[1..]);
    }
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut source = None;
    let mut root = None;
    let mut project_id = None;
    let mut manifest_created_at_unix = None;
    let mut display_name = None;
    let mut project_kind = None;
    let mut stage = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project migration option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project migration option value is required")?;
        match option {
            "--source" if source.is_none() => source = Some(PathBuf::from(value)),
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--project-id" if project_id.is_none() => {
                project_id = Some(parse_project_id(value)?);
            }
            "--manifest-created-at-unix" if manifest_created_at_unix.is_none() => {
                manifest_created_at_unix = Some(parse_manifest_timestamp(value)?);
            }
            "--name" if display_name.is_none() => {
                display_name = Some(parse_display_name(value)?);
            }
            "--kind" if project_kind.is_none() => {
                project_kind = Some(parse_project_kind(value)?);
            }
            "--stage" if stage.is_none() => stage = Some(parse_project_stage(value)?),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--source"
            | "--root"
            | "--project-id"
            | "--manifest-created-at-unix"
            | "--name"
            | "--kind"
            | "--stage"
            | "--expected-plan-digest" => {
                return Err("project migration option is unexpected or duplicate");
            }
            _ => return Err("unknown project migration option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    if apply && (project_id.is_none() || manifest_created_at_unix.is_none()) {
        return Err(
            "project migration apply requires the previewed project ID and manifest timestamp",
        );
    }
    let options = ProjectMigrationOptions {
        source: source.ok_or("legacy project source is required")?,
        root: root.ok_or("migrated project root is required")?,
        project_id,
        manifest_created_at_unix,
        display_name,
        project_kind,
        stage,
    };
    if apply {
        Ok(ProjectCliCommand::ApplyMigration(
            options,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewMigration(options))
    }
}

fn parse_project_migration_rollback(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut source = None;
    let mut root = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project migration rollback option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project migration rollback option value is required")?;
        match option {
            "--source" if source.is_none() => source = Some(PathBuf::from(value)),
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--source" | "--root" | "--expected-plan-digest" => {
                return Err("project migration rollback option is unexpected or duplicate");
            }
            _ => return Err("unknown project migration rollback option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    let options = ProjectMigrationRecoveryOptions {
        source: source.ok_or("legacy project rollback source is required")?,
        root: root.ok_or("migrated project rollback root is required")?,
    };
    if apply {
        Ok(ProjectCliCommand::ApplyMigrationRollback(
            options,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewMigrationRollback(options))
    }
}

fn parse_project_migration_recovery(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut source = None;
    let mut root = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project migration recovery option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project migration recovery option value is required")?;
        match option {
            "--source" if source.is_none() => source = Some(PathBuf::from(value)),
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--source" | "--root" | "--expected-plan-digest" => {
                return Err("project migration recovery option is unexpected or duplicate");
            }
            _ => return Err("unknown project migration recovery option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    let options = ProjectMigrationRecoveryOptions {
        source: source.ok_or("legacy project recovery source is required")?,
        root: root.ok_or("migrated project recovery root is required")?,
    };
    if apply {
        Ok(ProjectCliCommand::ApplyMigrationRecovery(
            options,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewMigrationRecovery(options))
    }
}

fn parse_identity_mutation(
    args: &[OsString],
) -> Result<(bool, ProjectId, Option<String>), &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut project_id = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => project_id = Some(parse_project_id(value)?),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--project-id" | "--expected-plan-digest" => {
                return Err("project option is unexpected or duplicate");
            }
            _ => return Err("unknown project option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    Ok((apply, project_id.ok_or("project ID is required")?, digest))
}

fn parse_mutation_mode(args: &[OsString]) -> Result<(bool, &[OsString]), &'static str> {
    let Some(mode) = args.first().and_then(|value| value.to_str()) else {
        return Err("project mutation mode is required");
    };
    match mode {
        "preview" => Ok((false, &args[1..])),
        "apply" => Ok((true, &args[1..])),
        _ => Err("project mutation mode must be preview or apply"),
    }
}

fn validate_apply_approval(
    apply: bool,
    approved: bool,
    digest: Option<&String>,
) -> Result<(), &'static str> {
    if apply && (!approved || digest.is_none()) {
        Err("project apply requires plan digest and filesystem approval")
    } else {
        Ok(())
    }
}

fn parse_lifecycle(
    args: &[OsString],
    operation: ProjectLifecycleCommand,
) -> Result<ProjectCliCommand, &'static str> {
    let Some(mode) = args.first().and_then(|value| value.to_str()) else {
        return Err("project mutation mode is required");
    };
    let apply = match mode {
        "preview" => false,
        "apply" => true,
        _ => return Err("project mutation mode must be preview or apply"),
    };
    let mut project_id = None;
    let mut expected_plan_digest = None;
    let mut approved = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("project option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("project option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => {
                project_id = Some(parse_project_id(value)?);
            }
            "--expected-plan-digest" if apply && expected_plan_digest.is_none() => {
                expected_plan_digest = Some(parse_sha256(value)?);
            }
            "--project-id" | "--expected-plan-digest" => {
                return Err("project option is unexpected or duplicate");
            }
            _ => return Err("unknown project option"),
        }
        index += 2;
    }
    let project_id = project_id.ok_or("project ID is required")?;
    if apply {
        if !approved {
            return Err("project apply requires filesystem approval");
        }
        Ok(ProjectCliCommand::ApplyLifecycle(
            operation,
            project_id,
            expected_plan_digest.ok_or("project apply plan digest is required")?,
        ))
    } else {
        Ok(ProjectCliCommand::PreviewLifecycle(operation, project_id))
    }
}

fn parse_graph(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let Some(mode) = args.first().and_then(|value| value.to_str()) else {
        return Err("project graph mode is required");
    };
    if mode == "snapshot" {
        return parse_project_id_only(&args[1..]).map(ProjectCliCommand::GraphSnapshot);
    }
    if mode == "portfolio" {
        return if args.len() == 1 {
            Ok(ProjectCliCommand::GraphPortfolio)
        } else {
            Err("project graph portfolio does not accept arguments")
        };
    }
    if mode == "doctor" {
        return parse_project_id_only(&args[1..]).map(ProjectCliCommand::GraphDoctor);
    }
    if mode != "query" {
        return Err("project graph mode must be snapshot, portfolio, query, or doctor");
    }

    let mut project_id = None;
    let mut projection_id = None;
    let mut focus_node_id = None;
    let mut direction = AcademicGraphDirection::Both;
    let mut direction_set = false;
    let mut max_depth = 1;
    let mut max_depth_set = false;
    let mut node_types = Vec::new();
    let mut relations = Vec::new();
    let mut layers = Vec::new();
    let mut canonical_id = None;
    let mut text = None;
    let mut max_nodes = 100;
    let mut max_nodes_set = false;
    let mut max_edges = 200;
    let mut max_edges_set = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("project graph option is not valid UTF-8")?;
        let value = args
            .get(index + 1)
            .ok_or("project graph option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => {
                project_id = Some(parse_project_id(value)?);
            }
            "--expected-projection-id" if projection_id.is_none() => {
                projection_id = Some(parse_graph_id(value, "grp_")?);
            }
            "--focus-node-id" if focus_node_id.is_none() => {
                focus_node_id = Some(parse_graph_id(value, "nod_")?);
            }
            "--direction" if !direction_set => {
                direction = parse_graph_direction(value)?;
                direction_set = true;
            }
            "--max-depth" if !max_depth_set => {
                max_depth = parse_graph_limit(value, 3)?;
                max_depth_set = true;
            }
            "--node-type" => node_types.push(parse_graph_node_type(value)?),
            "--relation" => relations.push(parse_graph_relation(value)?),
            "--layer" => layers.push(parse_graph_layer(value)?),
            "--canonical-id" if canonical_id.is_none() => {
                canonical_id = Some(parse_graph_text(value)?);
            }
            "--text" if text.is_none() => text = Some(parse_graph_text(value)?),
            "--max-nodes" if !max_nodes_set => {
                max_nodes = parse_graph_limit(value, 256)?;
                max_nodes_set = true;
            }
            "--max-edges" if !max_edges_set => {
                max_edges = parse_graph_limit(value, 512)?;
                max_edges_set = true;
            }
            "--project-id"
            | "--expected-projection-id"
            | "--focus-node-id"
            | "--direction"
            | "--max-depth"
            | "--canonical-id"
            | "--text"
            | "--max-nodes"
            | "--max-edges" => return Err("project graph option is duplicate"),
            _ => return Err("unknown project graph option"),
        }
        index += 2;
    }

    let mut query =
        AcademicGraphQueryV1::new(projection_id.ok_or("expected graph projection ID is required")?)
            .with_node_types(node_types)
            .with_relations(relations)
            .with_layers(layers)
            .with_limits(max_nodes, max_edges);
    if let Some(focus) = focus_node_id {
        query = query.with_focus(focus, direction).with_max_depth(max_depth);
    } else if max_depth_set {
        return Err("project graph max depth requires a focus node");
    }
    if let Some(value) = canonical_id {
        query = query.with_canonical_id(value);
    }
    if let Some(value) = text {
        query = query.with_text(value);
    }
    Ok(ProjectCliCommand::GraphQuery(ProjectGraphQueryOptions {
        project_id: project_id.ok_or("project ID is required")?,
        query,
    }))
}

fn parse_graph_id(value: &OsStr, prefix: &str) -> Result<String, &'static str> {
    let value = value.to_str().ok_or("graph identity is invalid")?;
    if value.strip_prefix(prefix).is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }) {
        Ok(value.to_string())
    } else {
        Err("graph identity is invalid")
    }
}

fn parse_graph_text(value: &OsStr) -> Result<String, &'static str> {
    let value = value.to_str().ok_or("graph filter text is invalid")?;
    if value.is_empty()
        || value.len() > 256
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err("graph filter text is invalid");
    }
    Ok(value.to_string())
}

fn parse_graph_limit(value: &OsStr, maximum: usize) -> Result<usize, &'static str> {
    value
        .to_str()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse().ok())
        .filter(|value| (1..=maximum).contains(value))
        .ok_or("graph query limit is invalid")
}

fn parse_graph_direction(value: &OsStr) -> Result<AcademicGraphDirection, &'static str> {
    match value.to_str() {
        Some("incoming") => Ok(AcademicGraphDirection::Incoming),
        Some("outgoing") => Ok(AcademicGraphDirection::Outgoing),
        Some("both") => Ok(AcademicGraphDirection::Both),
        _ => Err("graph direction is invalid"),
    }
}

fn parse_graph_layer(value: &OsStr) -> Result<AcademicGraphLayer, &'static str> {
    match value.to_str() {
        Some("portfolio") => Ok(AcademicGraphLayer::Portfolio),
        Some("literature") => Ok(AcademicGraphLayer::Literature),
        Some("idea-decision") => Ok(AcademicGraphLayer::IdeaDecision),
        Some("argument") => Ok(AcademicGraphLayer::Argument),
        Some("manuscript") => Ok(AcademicGraphLayer::Manuscript),
        Some("combined") => Ok(AcademicGraphLayer::Combined),
        _ => Err("graph layer is invalid"),
    }
}

fn parse_graph_node_type(value: &OsStr) -> Result<AcademicGraphNodeType, &'static str> {
    match value.to_str() {
        Some("project") => Ok(AcademicGraphNodeType::Project),
        Some("research-question") => Ok(AcademicGraphNodeType::ResearchQuestion),
        Some("idea") => Ok(AcademicGraphNodeType::Idea),
        Some("contribution") => Ok(AcademicGraphNodeType::Contribution),
        Some("concept") => Ok(AcademicGraphNodeType::Concept),
        Some("literature-cluster") => Ok(AcademicGraphNodeType::LiteratureCluster),
        Some("paper") => Ok(AcademicGraphNodeType::Paper),
        Some("claim") => Ok(AcademicGraphNodeType::Claim),
        Some("evidence") => Ok(AcademicGraphNodeType::Evidence),
        Some("decision") => Ok(AcademicGraphNodeType::Decision),
        Some("gap") => Ok(AcademicGraphNodeType::Gap),
        Some("method") => Ok(AcademicGraphNodeType::Method),
        Some("manuscript-section") => Ok(AcademicGraphNodeType::ManuscriptSection),
        Some("artifact") => Ok(AcademicGraphNodeType::Artifact),
        Some("task") => Ok(AcademicGraphNodeType::Task),
        _ => Err("graph node type is invalid"),
    }
}

fn parse_graph_relation(value: &OsStr) -> Result<AcademicGraphRelation, &'static str> {
    match value.to_str() {
        Some("contains") => Ok(AcademicGraphRelation::Contains),
        Some("cites") => Ok(AcademicGraphRelation::Cites),
        Some("cited-by") => Ok(AcademicGraphRelation::CitedBy),
        Some("supports") => Ok(AcademicGraphRelation::Supports),
        Some("weakens") => Ok(AcademicGraphRelation::Weakens),
        Some("contradicts") => Ok(AcademicGraphRelation::Contradicts),
        Some("extends") => Ok(AcademicGraphRelation::Extends),
        Some("defines") => Ok(AcademicGraphRelation::Defines),
        Some("operationalizes") => Ok(AcademicGraphRelation::Operationalizes),
        Some("uses-method") => Ok(AcademicGraphRelation::UsesMethod),
        Some("belongs-to-cluster") => Ok(AcademicGraphRelation::BelongsToCluster),
        Some("complements") => Ok(AcademicGraphRelation::Complements),
        Some("competes-with") => Ok(AcademicGraphRelation::CompetesWith),
        Some("combines-with") => Ok(AcademicGraphRelation::CombinesWith),
        Some("motivates") => Ok(AcademicGraphRelation::Motivates),
        Some("informs") => Ok(AcademicGraphRelation::Informs),
        Some("addresses-gap") => Ok(AcademicGraphRelation::AddressesGap),
        Some("appears-in-section") => Ok(AcademicGraphRelation::AppearsInSection),
        Some("derived-from") => Ok(AcademicGraphRelation::DerivedFrom),
        Some("supersedes") => Ok(AcademicGraphRelation::Supersedes),
        Some("bounded-by") => Ok(AcademicGraphRelation::BoundedBy),
        Some("shares-source") => Ok(AcademicGraphRelation::SharesSource),
        Some("shares-concept") => Ok(AcademicGraphRelation::SharesConcept),
        Some("forked-from") => Ok(AcademicGraphRelation::ForkedFrom),
        Some("extends-project") => Ok(AcademicGraphRelation::ExtendsProject),
        _ => Err("graph relation is invalid"),
    }
}

fn parse_project_id_only(args: &[OsString]) -> Result<ProjectId, &'static str> {
    if args.len() != 2 || args[0] != OsStr::new("--project-id") {
        return Err("exactly one project ID is required");
    }
    parse_project_id(&args[1])
}

fn parse_project_id(value: &OsStr) -> Result<ProjectId, &'static str> {
    ProjectId::parse(value.to_str().ok_or("project ID is invalid")?.to_string())
        .map_err(|_| "project ID is invalid")
}

fn parse_display_name(value: &OsStr) -> Result<String, &'static str> {
    let value = value.to_str().ok_or("project name is invalid")?;
    if value.is_empty()
        || value.len() > 160
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err("project name is invalid");
    }
    Ok(value.to_string())
}

fn parse_project_kind(value: &OsStr) -> Result<ProjectKind, &'static str> {
    match value.to_str() {
        Some("article") => Ok(ProjectKind::Article),
        Some("review") => Ok(ProjectKind::Review),
        Some("dissertation-article") => Ok(ProjectKind::DissertationArticle),
        Some("manuscript") => Ok(ProjectKind::Manuscript),
        _ => Err("project kind is invalid"),
    }
}

fn parse_project_stage(value: &OsStr) -> Result<ProjectStage, &'static str> {
    match value.to_str() {
        Some("idea") => Ok(ProjectStage::Idea),
        Some("framing") => Ok(ProjectStage::Framing),
        Some("literature") => Ok(ProjectStage::Literature),
        Some("design") => Ok(ProjectStage::Design),
        Some("analysis") => Ok(ProjectStage::Analysis),
        Some("writing") => Ok(ProjectStage::Writing),
        Some("review") => Ok(ProjectStage::Review),
        Some("submission") => Ok(ProjectStage::Submission),
        _ => Err("project stage is invalid"),
    }
}

fn parse_sha256(value: &OsStr) -> Result<String, &'static str> {
    let value = value.to_str().ok_or("project plan digest is invalid")?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("project plan digest is invalid");
    }
    Ok(value.to_string())
}

fn parse_manifest_timestamp(value: &OsStr) -> Result<u64, &'static str> {
    value
        .to_str()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse().ok())
        .ok_or("project manifest timestamp must be an unsigned decimal integer")
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "project-clock-unavailable")
}

fn lifecycle_preview_command(operation: ProjectLifecycleCommand) -> &'static str {
    match operation {
        ProjectLifecycleCommand::Archive => "project-archive-preview",
        ProjectLifecycleCommand::Restore => "project-restore-preview",
        ProjectLifecycleCommand::Refresh => "project-refresh-preview",
        ProjectLifecycleCommand::Unregister => "project-unregister-preview",
    }
}

fn lifecycle_apply_command(operation: ProjectLifecycleCommand) -> &'static str {
    match operation {
        ProjectLifecycleCommand::Archive => "project-archive-apply",
        ProjectLifecycleCommand::Restore => "project-restore-apply",
        ProjectLifecycleCommand::Refresh => "project-refresh-apply",
        ProjectLifecycleCommand::Unregister => "project-unregister-apply",
    }
}

fn json_output<T: Serialize>(value: &T) -> CliOutput {
    match serde_json::to_string_pretty(value) {
        Ok(rendered) => CliOutput::success_text(format!("{rendered}\n")),
        Err(_) => CliOutput::operation_failure("output-serialization-failed"),
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum ProjectCliOutput {
    Library(ProjectListOutput),
    Project(ProjectShowOutput),
    GraphSnapshot(ProjectGraphSnapshotOutput),
    GraphPortfolio(ProjectGraphPortfolioOutput),
    GraphQuery(ProjectGraphQueryOutput),
    GraphDoctor(ProjectGraphDoctorOutput),
    Doctor(ProjectDoctorOutput),
    Preview(ProjectPreviewOutput),
    Commit(ProjectCommitOutput),
    PortablePreview(ProjectPortablePreviewOutput),
    PortableCommit(ProjectPortableCommitOutput),
    MigrationPreview(ProjectMigrationPreviewOutput),
    MigrationRecoveryPreview(ProjectMigrationRecoveryPreviewOutput),
    MigrationCommit(ProjectMigrationCommitOutput),
    MigrationRollbackPreview(ProjectMigrationRollbackPreviewOutput),
    MigrationRollbackCommit(ProjectMigrationRollbackCommitOutput),
    Capture(Box<crate::capture_cli::CaptureCliOutput>),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectGraphSnapshotOutput {
    schema_version: u32,
    command: &'static str,
    snapshot: qiongli_project::AcademicGraphSnapshotV1,
    readiness: qiongli_project::AcademicGraphReadinessV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectGraphPortfolioOutput {
    schema_version: u32,
    command: &'static str,
    portfolio: qiongli_project::AcademicGraphPortfolioSnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectGraphDoctorOutput {
    schema_version: u32,
    command: &'static str,
    project_id: ProjectId,
    project_revision: u64,
    projection_id: String,
    index_id: String,
    node_count: usize,
    edge_count: usize,
    deterministic_rebuild: bool,
    persistent_index_state: &'static str,
    portable_authority: bool,
    readiness: qiongli_project::AcademicGraphReadinessV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectGraphQueryOutput {
    schema_version: u32,
    command: &'static str,
    result: qiongli_project::AcademicGraphQueryResultV1,
    readiness: qiongli_project::AcademicGraphReadinessV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectListOutput {
    schema_version: u32,
    command: &'static str,
    library: ResearchLibrarySnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectShowOutput {
    schema_version: u32,
    command: &'static str,
    library_revision: u64,
    project: qiongli_project::ArticleProjectSummaryV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDoctorOutput {
    schema_version: u32,
    command: &'static str,
    status: &'static str,
    blocking_projects: usize,
    migration_attention: usize,
    migration_diagnostics: Vec<ProjectMigrationDoctorV1>,
    library: ResearchLibrarySnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectPreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: ProjectMutationPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: qiongli_project::ProjectMutationCommitV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectPortablePreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: PortableProjectPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectPortableCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: qiongli_project::PortableProjectCommitV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMigrationPreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: ProjectMigrationPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMigrationRecoveryPreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: ProjectMigrationRecoveryPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMigrationCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: qiongli_project::ProjectMigrationCommitV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMigrationRollbackPreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: ProjectMigrationRollbackPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMigrationRollbackCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: qiongli_project::ProjectMigrationRollbackCommitV1,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parser_closes_project_mutation_shape_and_approval() {
        assert!(matches!(
            parse(&args(&[
                "create",
                "preview",
                "--root",
                "/tmp/paper",
                "--name",
                "Paper"
            ])),
            Ok(ProjectCliCommand::PreviewCreate(_))
        ));
        assert_eq!(
            parse(&args(&[
                "archive",
                "apply",
                "--project-id",
                "prj_00000000000000000000000000000000",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            ])),
            Err("project apply requires filesystem approval")
        );
        assert!(
            parse(&args(&[
                "create",
                "preview",
                "--root",
                "/tmp/paper",
                "--name",
                "Paper",
                "--name",
                "Duplicate"
            ]))
            .is_err()
        );
        assert!(matches!(
            parse(&args(&[
                "export",
                "preview",
                "--project-id",
                "prj_00000000000000000000000000000000",
                "--destination",
                "/tmp/portable-paper"
            ])),
            Ok(ProjectCliCommand::PreviewExport(_))
        ));
        assert_eq!(
            parse(&args(&[
                "import",
                "apply",
                "--source",
                "/tmp/portable-paper",
                "--root",
                "/tmp/imported-paper",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            ])),
            Err("project apply requires plan digest and filesystem approval")
        );
        assert!(matches!(
            parse(&args(&[
                "doctor",
                "repair",
                "preview",
                "--project-id",
                "prj_00000000000000000000000000000000"
            ])),
            Ok(ProjectCliCommand::PreviewDoctorRepair(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "migrate",
                "preview",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--name",
                "Migrated paper"
            ])),
            Ok(ProjectCliCommand::PreviewMigration(_))
        ));
        assert_eq!(
            parse(&args(&[
                "migrate",
                "apply",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write"
            ])),
            Err("project migration apply requires the previewed project ID and manifest timestamp")
        );
        assert!(matches!(
            parse(&args(&[
                "migrate",
                "apply",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--project-id",
                "prj_00000000000000000000000000000000",
                "--manifest-created-at-unix",
                "1721337601",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write"
            ])),
            Ok(ProjectCliCommand::ApplyMigration(_, _))
        ));
        assert_eq!(
            parse(&args(&[
                "migrate",
                "preview",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--manifest-created-at-unix",
                "+1721337601"
            ])),
            Err("project manifest timestamp must be an unsigned decimal integer")
        );
        assert!(matches!(
            parse(&args(&[
                "migrate",
                "recover",
                "preview",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper"
            ])),
            Ok(ProjectCliCommand::PreviewMigrationRecovery(_))
        ));
        assert_eq!(
            parse(&args(&[
                "migrate",
                "recover",
                "apply",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            ])),
            Err("project apply requires plan digest and filesystem approval")
        );
        assert!(matches!(
            parse(&args(&[
                "migrate",
                "recover",
                "apply",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write"
            ])),
            Ok(ProjectCliCommand::ApplyMigrationRecovery(_, _))
        ));
        assert_eq!(
            parse(&args(&[
                "migrate",
                "recover",
                "preview",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--source",
                "/tmp/other-paper"
            ])),
            Err("project migration recovery option is unexpected or duplicate")
        );
        assert!(matches!(
            parse(&args(&[
                "migrate",
                "rollback",
                "preview",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper"
            ])),
            Ok(ProjectCliCommand::PreviewMigrationRollback(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "migrate",
                "rollback",
                "apply",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write"
            ])),
            Ok(ProjectCliCommand::ApplyMigrationRollback(_, _))
        ));
        assert_eq!(
            parse(&args(&[
                "migrate",
                "rollback",
                "apply",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            ])),
            Err("project apply requires plan digest and filesystem approval")
        );
    }

    #[test]
    fn parser_closes_graph_snapshot_and_query_filters() {
        assert!(matches!(
            parse(&args(&["graph", "portfolio"])),
            Ok(ProjectCliCommand::GraphPortfolio)
        ));
        assert!(matches!(
            parse(&args(&[
                "graph",
                "doctor",
                "--project-id",
                "prj_00000000000000000000000000000000"
            ])),
            Ok(ProjectCliCommand::GraphDoctor(_))
        ));
        assert_eq!(
            parse(&args(&[
                "graph",
                "portfolio",
                "--project-id",
                "prj_00000000000000000000000000000000"
            ])),
            Err("project graph portfolio does not accept arguments")
        );
        assert!(matches!(
            parse(&args(&[
                "graph",
                "snapshot",
                "--project-id",
                "prj_00000000000000000000000000000000"
            ])),
            Ok(ProjectCliCommand::GraphSnapshot(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "graph",
                "query",
                "--project-id",
                "prj_00000000000000000000000000000000",
                "--expected-projection-id",
                "grp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--focus-node-id",
                "nod_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "--direction",
                "outgoing",
                "--max-depth",
                "2",
                "--node-type",
                "claim",
                "--relation",
                "cites",
                "--layer",
                "manuscript",
                "--text",
                "returns",
                "--max-nodes",
                "25"
            ])),
            Ok(ProjectCliCommand::GraphQuery(_))
        ));
        assert_eq!(
            parse(&args(&[
                "graph",
                "query",
                "--project-id",
                "prj_00000000000000000000000000000000",
                "--expected-projection-id",
                "grp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--max-depth",
                "2"
            ])),
            Err("project graph max depth requires a focus node")
        );
        assert_eq!(
            parse(&args(&[
                "graph",
                "query",
                "--project-id",
                "prj_00000000000000000000000000000000",
                "--expected-projection-id",
                "grp_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--max-nodes",
                "0"
            ])),
            Err("graph query limit is invalid")
        );
        assert_eq!(
            parse(&args(&[
                "graph",
                "query",
                "--project-id",
                "prj_00000000000000000000000000000000"
            ])),
            Err("expected graph projection ID is required")
        );
    }
}
