#![cfg_attr(
    not(feature = "desktop"),
    allow(
        dead_code,
        reason = "CLI retains the pure schema and DTO contract; session consumers belong to the desktop"
    )
)]
//! Desktop session owner. Durable observations and project tools have separate owners.
use crate::all_chat_history::{ChatHistory, ChatRecordKind};
#[cfg(debug_assertions)]
use crate::all_chat_research::ResearchContext;
use crate::all_chat_research::{ResearchRequest, ResearchSession, ResearchSnapshot};
use std::sync::{Arc, Mutex};

use futures::channel::mpsc;
use qiongli_execution::{AcpV1Control, AcpV1ControlRequest, AcpV1Update, CancellationToken};
use qiongli_project::{ProjectHealth, ProjectId, ProjectLifecycle, ProjectStateService};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_TURNS: usize = 64;
const INVALID: &str = "all-chat-invalid-request";
const STALE: &str = "all-chat-stale-control";

type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatAgent {
    OfflineDemo,
    Codex,
    Claude,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatStatus {
    Starting,
    Idle,
    Active,
    Closing,
    Closed,
    Interrupted,
}

#[derive(Clone, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChatPrompt {
    #[schemars(length(min = 1, max = 65_536))]
    pub text: String,
    #[schemars(length(max = 65_536))]
    pub context: String,
    // Labels only in 3b. Never interpreted as paths or silently sent to an adapter.
    #[schemars(length(max = 16))]
    pub source_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ChatRequest {
    Read {
        project_id: String,
    },
    Start {
        project_id: String,
        expected_project_revision: u64,
        agent: ChatAgent,
    },
    Prompt {
        run_id: String,
        expected_turn: u64,
        prompt: ChatPrompt,
    },
    Control {
        run_id: String,
        control: AcpV1ControlRequest,
    },
    Close {
        run_id: String,
    },
}

#[derive(Clone, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChatSnapshot {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    #[schemars(regex(pattern = r"^prj_[0-9a-f]{32}$"))]
    pub project_id: String,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub expected_project_revision: u64,
    #[schemars(regex(pattern = r"^run_[0-9a-f]{32}$"))]
    pub run_id: String,
    pub agent: ChatAgent,
    #[schemars(range(min = 1, max = 9_007_199_254_740_991_u64))]
    pub revision: u64,
    pub status: ChatStatus,
    #[schemars(range(min = 1, max = 65))]
    pub next_turn: u64,
    #[schemars(length(max = 64))]
    pub prompts: Vec<ChatPrompt>,
    #[schemars(length(max = 2_048))]
    pub updates: Vec<AcpV1Update>,
    pub error: Option<String>,
}

impl std::fmt::Debug for ChatPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ChatPrompt(<private>)")
    }
}
impl std::fmt::Debug for ChatSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatSnapshot")
            .field("status", &self.status)
            .field("revision", &self.revision)
            .field("content", &"<private>")
            .finish()
    }
}

struct Session {
    history: Arc<Mutex<ChatHistory>>,
    view: Arc<Mutex<ChatSnapshot>>,
    control: AcpV1Control,
    sender: mpsc::Sender<String>,
    stop: CancellationToken,
    research: Option<Arc<Mutex<ResearchSession>>>,
}

/// ponytail: one retained desktop session, capped at 64 turns/8 MiB. Add per-project
/// slots only when concurrent App conversations are a product requirement.
#[derive(Default)]
pub struct DesktopChat {
    session: Option<Session>,
    recovered: Option<(ChatHistory, ChatSnapshot)>,
}

impl Drop for DesktopChat {
    fn drop(&mut self) {
        if let Some(session) = &self.session {
            session.stop.cancel();
        }
    }
}

fn validate_project(projects: Option<&ProjectStateService>, id: &str, revision: u64) -> Result<()> {
    let id = ProjectId::parse(id).map_err(|_| INVALID)?;
    let snapshot = projects
        .ok_or("project-service-unavailable")?
        .snapshot()
        .map_err(|e| e.reason_code())?;
    let project = snapshot
        .projects
        .iter()
        .find(|p| p.project_id == id)
        .ok_or(INVALID)?;
    if project.lifecycle != ProjectLifecycle::Active
        || project.health != ProjectHealth::Ready
        || project.semantic_revision != revision
    {
        return Err("all-chat-project-changed");
    }
    Ok(())
}

pub(crate) fn validate_prompt(prompt: &ChatPrompt) -> Result<()> {
    if prompt.text.trim().is_empty()
        || prompt.text.len() > 64 * 1024
        || prompt.context.len() > 64 * 1024
        || prompt.text.contains('\0')
        || prompt.context.contains('\0')
        || prompt.source_refs.len() > 16
        || prompt
            .source_refs
            .iter()
            .any(|s| s.is_empty() || s.len() > 512 || s.chars().any(char::is_control))
        || prompt
            .source_refs
            .iter()
            .enumerate()
            .any(|(i, s)| prompt.source_refs[..i].contains(s))
    {
        return Err(INVALID);
    }
    Ok(())
}

impl DesktopChat {
    pub(crate) fn review_research_candidate(
        &self,
        candidate: &crate::all_chat_research::ResearchCandidate,
        projects: &ProjectStateService,
    ) -> Result<qiongli_project::VerifiedCaptureIntake> {
        let session = self.session.as_ref().ok_or(STALE)?;
        let view = session.view.lock().map_err(|_| INVALID)?;
        if view.run_id != candidate.run_id
            || candidate.turn_id != view.next_turn - 1
            || !matches!(view.status, ChatStatus::Idle | ChatStatus::Closed)
        {
            return Err(STALE);
        }
        let research = session
            .research
            .as_ref()
            .ok_or(STALE)?
            .lock()
            .map_err(|_| INVALID)?;
        if research
            .candidate
            .as_ref()
            .is_none_or(|current| current.turn_id != candidate.turn_id)
        {
            return Err(STALE);
        }
        let capture = research
            .context
            .capture(projects, candidate, research.captured_at_unix)?;
        let mut sources = research
            .context
            .manifest
            .sources
            .iter()
            .map(|s| (s.selection.artifact_path.clone(), s.content_digest.clone()))
            .collect::<Vec<_>>();
        sources.sort();
        sources.dedup();
        projects
            .preview_capture_from_current_sources(capture, &sources)
            .map_err(|e| e.reason_code())
    }

    pub(crate) fn research(
        &mut self,
        request: ResearchRequest,
        projects: Option<&ProjectStateService>,
    ) -> Result<Option<ResearchSnapshot>> {
        if !cfg!(debug_assertions) {
            return Err("all-chat-development-only");
        }
        let (run_id, dismiss) = match request {
            ResearchRequest::Start {
                project_id,
                expected_project_revision,
                selections,
                context_access: crate::all_chat_research::ContextAccess::SelectedExcerpts,
            } => {
                validate_project(projects, &project_id, expected_project_revision)?;
                #[cfg(debug_assertions)]
                {
                    let projects = projects.ok_or("project-service-unavailable")?;
                    let context = ResearchContext::read(
                        projects,
                        &project_id,
                        expected_project_revision,
                        &selections,
                    )?;
                    self.start_with_research(
                        project_id,
                        expected_project_revision,
                        ChatAgent::OfflineDemo,
                        projects.clone(),
                        Some(context),
                    )?;
                    self.recovered = None;
                    (
                        self.session
                            .as_ref()
                            .ok_or(STALE)?
                            .view
                            .lock()
                            .map_err(|_| INVALID)?
                            .run_id
                            .clone(),
                        None,
                    )
                }
                #[cfg(not(debug_assertions))]
                {
                    let _ = selections;
                    return Err("all-chat-development-only");
                }
            }
            ResearchRequest::Read { run_id } => (run_id, None),
            ResearchRequest::Dismiss { run_id, turn_id } => (run_id, Some(turn_id)),
        };
        qiongli_execution::RunId::parse(&run_id).map_err(|_| INVALID)?;
        let Some(session) = &self.session else {
            return Ok(None);
        };
        let view = session.view.lock().map_err(|_| INVALID)?;
        if view.run_id != run_id {
            return Err(STALE);
        }
        let Some(research) = &session.research else {
            return Ok(None);
        };
        let mut research = research.lock().map_err(|_| INVALID)?;
        if let Some(turn) = dismiss {
            if research
                .candidate
                .as_ref()
                .is_none_or(|c| c.turn_id != turn)
            {
                return Err(STALE);
            }
            research.candidate = None;
        }
        let error = research
            .context
            .revalidate(projects.ok_or("project-service-unavailable")?)
            .err()
            .map(str::to_owned)
            .or_else(|| research.error.clone());
        Ok(Some(ResearchSnapshot {
            run_id,
            manifest_digest: crate::all_chat_research::digest(&research.context.manifest)?,
            manifest: research.context.manifest.clone(),
            candidate: if error.is_none() {
                research
                    .candidate
                    .clone()
                    .filter(|c| c.turn_id == view.next_turn - 1)
            } else {
                None
            },
            error,
        }))
    }

    pub fn execute(
        &mut self,
        request: ChatRequest,
        projects: Option<&ProjectStateService>,
    ) -> Result<Option<ChatSnapshot>> {
        match request {
            ChatRequest::Read { project_id } => {
                ProjectId::parse(&project_id).map_err(|_| INVALID)?;
                if !cfg!(debug_assertions) {
                    return Err("all-chat-development-only");
                }
                if let Some(session) = &self.session {
                    let view = session.view.lock().map_err(|_| INVALID)?;
                    if view.project_id == project_id {
                        return Ok(Some(view.clone()));
                    }
                    if !matches!(view.status, ChatStatus::Closed | ChatStatus::Interrupted) {
                        return Err("all-chat-other-project-active");
                    }
                }
                if self
                    .recovered
                    .as_ref()
                    .is_none_or(|(_, view)| view.project_id != project_id)
                {
                    self.recovered = ChatHistory::load_latest(
                        projects.ok_or("project-service-unavailable")?,
                        &project_id,
                    )?;
                }
                return Ok(self.recovered.as_ref().map(|(_, view)| view.clone()));
            }

            ChatRequest::Start {
                project_id,
                expected_project_revision,
                agent,
            } => {
                validate_project(projects, &project_id, expected_project_revision)?;
                self.start(
                    project_id,
                    expected_project_revision,
                    agent,
                    projects.ok_or("project-service-unavailable")?.clone(),
                )?;
                self.recovered = None;
            }
            ChatRequest::Prompt {
                run_id,
                expected_turn,
                mut prompt,
            } => {
                let session = self.session.as_mut().ok_or(STALE)?;
                let mut view = session.view.lock().map_err(|_| INVALID)?;
                if view.run_id != run_id
                    || view.status != ChatStatus::Idle
                    || view.next_turn != expected_turn
                {
                    return Err(STALE);
                }
                validate_project(projects, &view.project_id, view.expected_project_revision)?;
                validate_prompt(&prompt)?;
                if view.prompts.len() >= MAX_TURNS {
                    return Err("all-chat-capacity");
                }
                // Only the user's explicit text/context is transmitted. Source access is Stage 3c.
                let text = if let Some(research) = &session.research {
                    if !prompt.context.is_empty() || !prompt.source_refs.is_empty() {
                        return Err(INVALID);
                    }
                    let research = research.lock().map_err(|_| INVALID)?;
                    research
                        .context
                        .revalidate(projects.ok_or("project-service-unavailable")?)?;
                    let text = research.context.prompt(&prompt.text)?;
                    prompt.context =
                        serde_json::to_string(&research.context.manifest).map_err(|_| INVALID)?;
                    text
                } else if prompt.context.is_empty() {
                    prompt.text.clone()
                } else {
                    format!(
                        "{}\n\nUser-supplied context:\n{}",
                        prompt.text, prompt.context
                    )
                };
                // Commit intent before making it observable to the provider worker.
                let next = session.history.lock().map_err(|_| INVALID)?.append(
                    &view,
                    ChatRecordKind::Prompt {
                        turn_id: expected_turn,
                        prompt,
                    },
                )?;
                *view = next;
                if session.sender.try_send(text).is_err() {
                    session.stop.cancel();
                    return Err("all-chat-session-interrupted");
                }
            }

            ChatRequest::Control { run_id, control } => {
                let session = self.session.as_ref().ok_or(STALE)?;
                let view = session.view.lock().map_err(|_| INVALID)?;
                if view.run_id != run_id || view.status != ChatStatus::Active {
                    return Err(STALE);
                }
                session.control.apply(control).map_err(|_| STALE)?;
            }
            ChatRequest::Close { run_id } => {
                if let Some((history, view)) = &mut self.recovered
                    && view.run_id == run_id
                    && view.status == ChatStatus::Interrupted
                {
                    *view = history.append(
                        view,
                        ChatRecordKind::State {
                            status: ChatStatus::Closed,
                            error: view.error.clone(),
                        },
                    )?;
                    history.release();
                    return Ok(Some(view.clone()));
                }
                let session = self.session.as_ref().ok_or(STALE)?;
                let mut view = session.view.lock().map_err(|_| INVALID)?;
                if view.run_id != run_id
                    || matches!(view.status, ChatStatus::Closed | ChatStatus::Closing)
                {
                    return Err(STALE);
                }
                let status = if view.status == ChatStatus::Interrupted {
                    ChatStatus::Closed
                } else {
                    ChatStatus::Closing
                };
                *view = session.history.lock().map_err(|_| INVALID)?.append(
                    &view,
                    ChatRecordKind::State {
                        status,
                        error: view.error.clone(),
                    },
                )?;
                session.stop.cancel();
                if status == ChatStatus::Closed {
                    session.history.lock().map_err(|_| INVALID)?.release();
                }
            }
        }
        self.session
            .as_ref()
            .map(|s| s.view.lock().map(|v| v.clone()).map_err(|_| INVALID))
            .transpose()
    }

    #[cfg(not(debug_assertions))]
    fn start(&mut self, _: String, _: u64, _: ChatAgent, _: ProjectStateService) -> Result<()> {
        Err("all-chat-development-only")
    }

    #[cfg(debug_assertions)]
    fn start(
        &mut self,
        project_id: String,
        expected_project_revision: u64,
        agent: ChatAgent,
        projects: ProjectStateService,
    ) -> Result<()> {
        self.start_with_research(project_id, expected_project_revision, agent, projects, None)
    }

    #[cfg(debug_assertions)]
    fn start_with_research(
        &mut self,
        project_id: String,
        expected_project_revision: u64,
        agent: ChatAgent,
        projects: ProjectStateService,
        context: Option<ResearchContext>,
    ) -> Result<()> {
        if let Some(session) = &self.session
            && !matches!(
                session.view.lock().map_err(|_| INVALID)?.status,
                ChatStatus::Closed | ChatStatus::Interrupted
            )
        {
            return Err("all-chat-session-active");
        }
        use futures::StreamExt;
        use qiongli_execution::{AcpV1Client, AgentBackendErrorCode, OrchestrationRole, RunId};
        let client = match agent {
            ChatAgent::OfflineDemo => match &context {
                Some(context) => AcpV1Client::for_development_read_responses(
                    vec![context.demo_response()?; MAX_TURNS],
                    context.read_view(),
                )
                .map_err(|_| INVALID)?,
                None => AcpV1Client::for_development_demo(),
            },
            ChatAgent::Codex | ChatAgent::Claude => return Err("all-chat-capability-unavailable"),
        };
        let lease = ChatHistory::acquire(&projects, &project_id)?;
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| INVALID)?;
        let run_id = format!(
            "run_{}",
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
        );
        let control = AcpV1Control::new(
            RunId::parse(&run_id).map_err(|_| INVALID)?,
            OrchestrationRole::Primary,
        )
        .map_err(|_| INVALID)?;
        let view = Arc::new(Mutex::new(ChatSnapshot {
            schema_version: 1,
            project_id,
            expected_project_revision,
            run_id: run_id.clone(),
            agent,
            revision: 1,
            status: ChatStatus::Starting,
            next_turn: 1,
            prompts: vec![],
            updates: vec![],
            error: None,
        }));
        let history = Arc::new(Mutex::new(ChatHistory::create(
            projects.clone(),
            lease,
            &*view.lock().map_err(|_| INVALID)?,
        )?));
        let worker_history = Arc::clone(&history);
        let stop = CancellationToken::new();
        let (sender, mut receiver) = mpsc::channel::<String>(1);
        let worker_view = Arc::clone(&view);
        let worker_stop = stop.clone();
        let worker_control = control.clone();
        let captured_at_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| INVALID)?
            .as_secs();
        let research = context.map(|context| {
            Arc::new(Mutex::new(ResearchSession {
                context,
                candidate: None,
                error: None,
                captured_at_unix,
            }))
        });
        let worker_research = research.clone();
        // The in-process fixture does not use cwd. Do not expose a research path or
        // create a second workspace lifecycle before the real tool boundary exists.
        let cwd = std::env::temp_dir();
        std::thread::Builder::new()
            .name("qiongli-all-chat".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    futures::executor::block_on(async {
                        let run = client.with_control(worker_control.clone()).with_session(
                            &cwd,
                            worker_stop.clone(),
                            async |session| {
                                set_status(
                                    &worker_view,
                                    &worker_history,
                                    &worker_stop,
                                    ChatStatus::Idle,
                                );
                                while let Some(prompt) = receiver.next().await {
                                    let turn =
                                        session.run_turn(prompt, worker_stop.clone()).await?;
                                    if let Some(research) = &worker_research {
                                        let mut research = research
                                            .lock()
                                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                                        match research
                                            .context
                                            .candidate_from_turn(&projects, &run_id, &turn)
                                        {
                                            Ok(candidate) => {
                                                research.candidate = Some(candidate);
                                                research.error = None;
                                            }
                                            Err(code) => {
                                                research.candidate = None;
                                                research.error = Some(code.into());
                                            }
                                        }
                                    }
                                }
                                Ok(())
                            },
                        );
                        let drive = async {
                            futures::pin_mut!(run);
                            let cancel = worker_stop.cancelled();
                            futures::pin_mut!(cancel);
                            match futures::future::select(run, cancel).await {
                                futures::future::Either::Left((result, _)) => result,
                                futures::future::Either::Right(_) => Ok(()),
                            }
                        };
                        let collect = async {
                            while let Some(update) = worker_control.next_update().await {
                                let mut view = worker_view
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                                let completed = matches!(
                                    &update.kind,
                                    qiongli_execution::AcpV1UpdateKind::Turn {
                                        status: qiongli_execution::AcpV1TurnStatus::Completed,
                                        ..
                                    }
                                );
                                let next = worker_history
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                                    .append(&view, ChatRecordKind::Update { update });
                                match next {
                                    Ok(next) => *view = next,
                                    Err(code) => {
                                        view.status = ChatStatus::Interrupted;
                                        view.error = Some(code.into());
                                        worker_stop.cancel();
                                    }
                                }
                                drop(view);
                                if completed {
                                    set_status(
                                        &worker_view,
                                        &worker_history,
                                        &worker_stop,
                                        ChatStatus::Idle,
                                    );
                                }
                            }
                        };
                        futures::join!(drive, collect).0
                    })
                }));
                let mut view = worker_view
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let closed = view.status == ChatStatus::Closing;
                let status = if closed {
                    ChatStatus::Closed
                } else {
                    ChatStatus::Interrupted
                };
                let error = if closed {
                    None
                } else {
                    Some(
                        match result {
                            Ok(Err(error)) => match error.code {
                                AgentBackendErrorCode::AuthenticationUnavailable => {
                                    "all-chat-authentication-required"
                                }
                                AgentBackendErrorCode::CapabilityUnavailable => {
                                    "all-chat-capability-unavailable"
                                }
                                _ => "all-chat-session-interrupted",
                            },
                            _ => "all-chat-session-interrupted",
                        }
                        .into(),
                    )
                };
                let mut history = worker_history
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                match history.append(&view, ChatRecordKind::State { status, error }) {
                    Ok(next) => *view = next,
                    Err(code) => {
                        view.status = ChatStatus::Interrupted;
                        view.error = Some(code.into());
                    }
                }
                history.release();
            })
            .map_err(|_| "all-chat-worker-unavailable")?;
        self.session = Some(Session {
            history,
            view,
            control,
            sender,
            research,
            stop,
        });
        Ok(())
    }
}

#[cfg(debug_assertions)]
fn set_status(
    view: &Mutex<ChatSnapshot>,
    history: &Mutex<ChatHistory>,
    stop: &CancellationToken,
    status: ChatStatus,
) {
    let mut view = view
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if matches!(
        view.status,
        ChatStatus::Closing | ChatStatus::Closed | ChatStatus::Interrupted
    ) {
        return;
    }
    match history
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .append(
            &view,
            ChatRecordKind::State {
                status,
                error: None,
            },
        ) {
        Ok(next) => *view = next,
        Err(code) => {
            view.status = ChatStatus::Interrupted;
            view.error = Some(code.into());
            stop.cancel();
        }
    }
}

#[derive(JsonSchema)]
#[allow(dead_code)]
struct Contract {
    request: ChatRequest,
    response: Option<ChatSnapshot>,
}

pub fn all_chat_control_schema_json() -> std::result::Result<String, serde_json::Error> {
    let mut schema = schemars::generate::SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator()
        .into_root_schema_for::<Contract>();
    schema.insert(
        "$id".into(),
        "https://qiongli.dev/schemas/app/all-chat-control-v1.json".into(),
    );
    serde_json::to_string_pretty(&schema).map(|s| s + "\n")
}

#[cfg(feature = "desktop")]
pub(crate) struct DesktopChatState {
    pub(crate) chat: Mutex<crate::all_chat_control::DesktopChat>,
    pub(crate) projects: Option<ProjectStateService>,
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub(crate) fn qiongli_all_chat(
    request: crate::all_chat_control::ChatRequest,
    state: tauri::State<'_, DesktopChatState>,
) -> Result<Option<crate::all_chat_control::ChatSnapshot>> {
    state
        .chat
        .lock()
        .map_err(|_| "all-chat-lock-failed")?
        .execute(request, state.projects.as_ref())
}

#[cfg(feature = "desktop")]
#[tauri::command]
pub(crate) fn qiongli_all_chat_research(
    request: ResearchRequest,
    state: tauri::State<'_, DesktopChatState>,
) -> Result<Option<ResearchSnapshot>> {
    state
        .chat
        .lock()
        .map_err(|_| "all-chat-lock-failed")?
        .research(request, state.projects.as_ref())
}

#[cfg(all(test, debug_assertions, feature = "desktop"))]
mod all_chat_ipc_tests {
    use super::*;
    use serde_json::{Value, json};
    use std::time::{Duration, Instant};

    #[test]
    fn all_chat_real_ipc_retains_turns_and_rejects_stale_controls() {
        use qiongli_project::{ApprovedProjectMutation, ProjectKind, ProjectRegistrationOptions};
        let root = std::env::temp_dir().join(format!(
            "qiongli-chat-ipc-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let config =
            qiongli_config::resolve_config_root(Some(root.join("config").as_os_str()), &root)
                .unwrap();
        let projects = ProjectStateService::new(config);
        let plan = projects
            .preview_create(
                root.join("article"),
                ProjectRegistrationOptions::new("Chat fixture", ProjectKind::Article),
                1,
            )
            .unwrap();
        let project_id = plan.preview().project_id.as_str().to_owned();
        projects
            .apply(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        let revision = projects.snapshot().unwrap().projects[0].semantic_revision;
        let app = tauri::test::mock_builder()
            .manage(DesktopChatState {
                chat: Mutex::new(crate::all_chat_control::DesktopChat::default()),
                projects: Some(projects.clone()),
            })
            .invoke_handler(tauri::generate_handler![qiongli_all_chat])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let call = |request: Value| -> std::result::Result<Value, Value> {
            tauri::test::get_ipc_response(
                &webview,
                tauri::webview::InvokeRequest {
                    cmd: "qiongli_all_chat".into(),
                    callback: tauri::ipc::CallbackFn(0),
                    error: tauri::ipc::CallbackFn(1),
                    url: webview.url().unwrap(),
                    body: tauri::ipc::InvokeBody::Json(json!({"request":request})),
                    headers: Default::default(),
                    invoke_key: tauri::test::INVOKE_KEY.into(),
                },
            )
            .map(|body| body.deserialize::<Value>().unwrap())
        };
        let read = json!({"type":"read", "projectId":project_id});
        let wait = |predicate: &dyn Fn(&Value) -> bool| {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                let value = call(read.clone()).unwrap();
                if predicate(&value) {
                    break value;
                }
                assert!(Instant::now() < deadline, "chat did not settle: {value}");
                std::thread::sleep(Duration::from_millis(5));
            }
        };
        assert_eq!(call(read.clone()).unwrap(), Value::Null);
        assert!(call(json!({"type":"start","projectId":project_id,"expectedProjectRevision":revision+1,"agent":"offline_demo"})).is_err());
        assert!(call(json!({"type":"start","projectId":project_id,"expectedProjectRevision":revision,"agent":"codex"})).is_err());
        let start = json!({"type":"start","projectId":project_id,"expectedProjectRevision":revision,"agent":"offline_demo"});
        let initial = call(start.clone()).unwrap();
        let run = initial["runId"].as_str().unwrap();
        assert!(call(start).is_err());
        wait(&|v| v["status"] == "idle");
        let prompt = |turn| json!({"type":"prompt","runId":run,"expectedTurn":turn,"prompt":{"text":"Demonstrate activity", "context":"Synthetic fixture context", "sourceRefs":["fixture-source"]}});
        let mut golden = Vec::new();
        for turn in 1..=2 {
            call(prompt(turn)).unwrap();
            assert!(call(prompt(turn)).is_err());
            let pending = wait(&|v| {
                let updates = v["updates"].as_array().unwrap();
                updates.iter().any(|u| {
                    u["kind"]["type"] == "permission_pending"
                        && u["kind"]["request"]["binding"]["turnId"] == turn
                }) && updates
                    .iter()
                    .any(|u| u["kind"]["type"] == "plan" && u["kind"]["binding"]["turnId"] == turn)
            });
            let permission = pending["updates"]
                .as_array()
                .unwrap()
                .iter()
                .rev()
                .find(|u| u["kind"]["type"] == "permission_pending")
                .unwrap()["kind"]["request"]
                .clone();
            let mut choice = json!({"type":"control", "runId":run, "control":{"type":"permission", "binding":permission["binding"], "requestId":permission["requestId"], "choice":{"type":"select","optionId": if turn == 1 {"allow"} else {"deny"}}}});
            let mut stale = choice.clone();
            stale["control"]["binding"]["turnId"] = json!(turn + 1);
            assert!(call(stale).is_err());
            golden.push(pending);
            call(choice.clone()).unwrap();
            assert!(call(choice.clone()).is_err());
            choice["control"]["choice"]["optionId"] = json!("always");
            assert!(call(choice).is_err());
            let idle = wait(&|v| {
                v["status"] == "idle"
                    && v["updates"].as_array().unwrap().iter().any(|u| {
                        u["kind"]["type"] == "turn"
                            && u["kind"]["status"] == "completed"
                            && u["kind"]["binding"]["turnId"] == turn
                    })
            });
            assert_eq!(idle["prompts"].as_array().unwrap().len(), turn as usize);
        }
        call(prompt(3)).unwrap();
        let pending = wait(&|v| {
            v["updates"].as_array().unwrap().iter().any(|u| {
                u["kind"]["type"] == "permission_pending"
                    && u["kind"]["request"]["binding"]["turnId"] == 3
            })
        });
        let binding = pending["updates"]
            .as_array()
            .unwrap()
            .iter()
            .rev()
            .find(|u| u["kind"]["type"] == "permission_pending")
            .unwrap()["kind"]["request"]["binding"]
            .clone();
        call(json!({"type":"control", "runId":run, "control":{"type":"cancel","binding":binding}}))
            .unwrap();
        let interrupted = wait(&|v| v["status"] == "interrupted");
        assert!(call(prompt(4)).is_err());
        golden.push(interrupted);
        call(json!({"type":"close","runId":run})).unwrap();
        let closed = wait(&|v| v["status"] == "closed");
        golden.push(closed.clone());
        // A fresh App owner reconstructs exactly the committed view without a provider.
        use tauri::Manager;
        *app.state::<DesktopChatState>().chat.lock().unwrap() = DesktopChat::default();
        assert_eq!(call(read.clone()).unwrap(), closed);
        assert!(call(prompt(4)).is_err());
        let schema = crate::all_chat_control_schema_json().unwrap();
        assert_eq!(
            schema,
            include_str!("../schemas/all-chat-control-v1.schema.json")
        );
        // Golden snapshots come from actual IPC, with only random IDs/timing counters normalized.
        fn normalize(value: &mut Value, project: &str, run: &str) {
            match value {
                Value::Object(map) => {
                    for (key, value) in map {
                        if key == "connectionId" {
                            *value = json!("acp-connection-00000000000000000000000000000000");
                        } else if key == "revision" {
                            *value = json!(1);
                        } else {
                            normalize(value, project, run);
                        }
                    }
                }
                Value::Array(values) => {
                    for value in values {
                        normalize(value, project, run);
                    }
                }
                Value::String(s) if s == project => {
                    *s = "prj_00000000000000000000000000000000".into()
                }
                Value::String(s) if s == run => *s = "run_00000000000000000000000000000000".into(),
                _ => {}
            }
        }
        let mut fixture = json!(golden);
        normalize(&mut fixture, &project_id, run);
        let rendered = serde_json::to_string_pretty(&fixture).unwrap() + "\n";
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/all-chat-control-v1.json");
        if std::env::var_os("QIONGLI_UPDATE_CHAT_FIXTURE").is_some() {
            std::fs::write(&path, &rendered).unwrap();
        }
        assert_eq!(rendered, std::fs::read_to_string(path).unwrap());
        // The fixture App session never changes the registered research project.
        assert_eq!(
            projects.snapshot().unwrap().projects[0].semantic_revision,
            revision
        );
        drop(webview);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
