#![cfg_attr(
    not(feature = "desktop"),
    allow(
        dead_code,
        reason = "CLI retains the pure schema and DTO contract; session consumers belong to the desktop"
    )
)]
//! Canonical private event log; the App view is a disposable projection, never a second commit.
use qiongli_execution::{
    AcpV1PermissionChoice, AcpV1PermissionKind, AcpV1TurnStatus, AcpV1Update, AcpV1UpdateKind,
    OrchestrationRole, RunId,
};
use qiongli_project::{MAX_CHAT_DOCUMENT_BYTES, ProjectChatLease, ProjectId, ProjectStateService};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::all_chat_control::{ChatAgent, ChatPrompt, ChatSnapshot, ChatStatus, validate_prompt};

const INVALID: &str = "all-chat-history-invalid";
const STORAGE: &str = "all-chat-history-unavailable";
const MAX_RECORDS: usize = 2304;
// Leave room for close/interruption even when the last public update fills the log.
const RECOVERY_RESERVE_BYTES: usize = 4096;
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Deserialize, JsonSchema, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum ChatRecordKind {
    Prompt {
        turn_id: u64,
        prompt: ChatPrompt,
    },
    Update {
        update: AcpV1Update,
    },
    State {
        status: ChatStatus,
        error: Option<String>,
    },
}

#[derive(Clone, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ChatRecord {
    sequence: u64,
    recorded_at_unix_ms: u64,
    // Causality is owned locally; provider updates also carry their exact turn binding.
    caused_by_turn: u64,
    kind: ChatRecordKind,
}

#[derive(Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ChatDocument {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    project_id: String,
    expected_project_revision: u64,
    run_id: String,
    agent: ChatAgent,
    created_at_unix_ms: u64,
    #[schemars(length(max = 2304))]
    records: Vec<ChatRecord>,
}

pub(crate) struct ChatHistory {
    document: ChatDocument,
    digest: String,
    projects: ProjectStateService,
    lease: Option<ProjectChatLease>,
    failed: bool,
}

impl ChatHistory {
    pub fn acquire(projects: &ProjectStateService, project_id: &str) -> Result<ProjectChatLease> {
        projects
            .acquire_chat_lease(&ProjectId::parse(project_id).map_err(|_| INVALID)?)
            .map_err(|_| STORAGE)
    }

    #[cfg(any(debug_assertions, test))]
    pub fn create(
        projects: ProjectStateService,
        lease: ProjectChatLease,
        view: &ChatSnapshot,
    ) -> Result<Self> {
        let mut history = Self {
            document: ChatDocument {
                schema_version: 1,
                project_id: view.project_id.clone(),
                expected_project_revision: view.expected_project_revision,
                run_id: view.run_id.clone(),
                agent: view.agent,
                created_at_unix_ms: now()?,
                records: vec![],
            },
            digest: String::new(),
            projects,
            lease: Some(lease),
            failed: false,
        };
        // Validate every existing file before creating another run. Corrupt/future files
        // are preserved and cannot be silently bypassed by starting a fresh session.
        for entry in history
            .projects
            .list_chat_checkpoints(history.lease.as_ref().ok_or(STORAGE)?)
            .map_err(|_| STORAGE)?
        {
            let (prior, _) = decode(
                entry.document().bytes(),
                &view.project_id,
                entry.checkpoint_id(),
            )?;
            history.document.created_at_unix_ms = history
                .document
                .created_at_unix_ms
                .max(prior.created_at_unix_ms + 1);
        }
        if history.document.created_at_unix_ms > 9_007_199_254_740_991 {
            return Err(INVALID);
        }
        let bytes = encode(&history.document)?;
        let commit = history
            .projects
            .replace_chat_checkpoint(
                history.lease.as_ref().ok_or(STORAGE)?,
                &view.run_id,
                None,
                &bytes,
            )
            .map_err(|_| STORAGE)?;
        history.digest = commit.document_sha256;
        Ok(history)
    }

    pub fn load_latest(
        projects: &ProjectStateService,
        project_id: &str,
    ) -> Result<Option<(Self, ChatSnapshot)>> {
        let lease = Self::acquire(projects, project_id)?;
        let mut latest: Option<(ChatDocument, ChatSnapshot, String)> = None;
        for entry in projects
            .list_chat_checkpoints(&lease)
            .map_err(|_| STORAGE)?
        {
            let (document, view) =
                decode(entry.document().bytes(), project_id, entry.checkpoint_id())?;
            if latest.as_ref().is_none_or(|(prior, _, _)| {
                (document.created_at_unix_ms, &document.run_id)
                    > (prior.created_at_unix_ms, &prior.run_id)
            }) {
                latest = Some((document, view, entry.document().sha256().to_owned()));
            }
        }
        let Some((document, mut view, digest)) = latest else {
            return Ok(None);
        };
        let mut history = Self {
            document,
            digest,
            projects: projects.clone(),
            lease: Some(lease),
            failed: false,
        };
        // Possession of the OS lease proves there is no live owner. Never resume or
        // resend any saved request, permission, tool call or project mutation.
        if !matches!(view.status, ChatStatus::Closed | ChatStatus::Interrupted) {
            view = history.append(
                &view,
                ChatRecordKind::State {
                    status: ChatStatus::Interrupted,
                    error: Some("all-chat-restart-interrupted".into()),
                },
            )?;
        }
        history.release();
        Ok(Some((history, view)))
    }

    pub fn release(&mut self) {
        self.lease = None;
    }

    pub fn append(&mut self, current: &ChatSnapshot, kind: ChatRecordKind) -> Result<ChatSnapshot> {
        if self.failed {
            return Err(STORAGE);
        }
        if current.revision != self.document.records.len() as u64 + 1
            || current.run_id != self.document.run_id
            || current.project_id != self.document.project_id
        {
            return Err(INVALID);
        }
        let mut next = current.clone();
        if !apply(&mut next, &kind)? {
            return Ok(next);
        }
        let finalizing = matches!(
            &kind,
            ChatRecordKind::State {
                status: ChatStatus::Closing | ChatStatus::Closed | ChatStatus::Interrupted,
                ..
            }
        );
        if self.document.records.len() >= MAX_RECORDS - if finalizing { 0 } else { 3 } {
            return Err("all-chat-history-capacity");
        }
        let timestamp = now()?.max(
            self.document
                .records
                .last()
                .map_or(self.document.created_at_unix_ms, |r| r.recorded_at_unix_ms),
        );
        let record = ChatRecord {
            sequence: self.document.records.len() as u64 + 1,
            recorded_at_unix_ms: timestamp,
            caused_by_turn: next.next_turn - 1,
            kind,
        };
        if self.lease.is_none() {
            self.lease = Some(Self::acquire(&self.projects, &current.project_id)?);
        }
        self.document.records.push(record);
        let bytes = match encode(&self.document) {
            Ok(bytes)
                if finalizing
                    || bytes.len() <= MAX_CHAT_DOCUMENT_BYTES - RECOVERY_RESERVE_BYTES =>
            {
                bytes
            }
            Ok(_) => {
                self.document.records.pop();
                return Err("all-chat-history-capacity");
            }
            Err(error) => {
                self.document.records.pop();
                return Err(error);
            }
        };
        let lease = self.lease.as_ref().ok_or(STORAGE)?;
        match self
            .projects
            .replace_chat_checkpoint(lease, &next.run_id, Some(&self.digest), &bytes)
        {
            Ok(commit) => {
                self.digest = commit.document_sha256;
                Ok(next)
            }
            Err(_) => {
                // A failed fsync/readback may follow a successful rename. Stop this
                // writer; next startup reads the actual bytes instead of retrying a send.
                self.failed = true;
                self.document.records.pop();
                Err(STORAGE)
            }
        }
    }
}

fn now() -> Result<u64> {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| STORAGE)?
        .as_millis();
    u64::try_from(millis)
        .ok()
        .filter(|n| *n <= 9_007_199_254_740_991)
        .ok_or(STORAGE)
}

// ponytail: atomic full-document replacement, bounded to 64 turns/8 MiB; use segments only if measured write latency requires them.
fn encode(document: &ChatDocument) -> Result<Vec<u8>> {
    let bytes = serde_json_canonicalizer::to_vec(document).map_err(|_| INVALID)?;
    if bytes.len() > MAX_CHAT_DOCUMENT_BYTES {
        return Err("all-chat-history-capacity");
    }
    Ok(bytes)
}

fn decode(bytes: &[u8], project: &str, run: &str) -> Result<(ChatDocument, ChatSnapshot)> {
    if bytes.len() > MAX_CHAT_DOCUMENT_BYTES {
        return Err(INVALID);
    }
    let document: ChatDocument = serde_json::from_slice(bytes).map_err(|_| INVALID)?;
    // Canonical equality also rejects duplicate JSON keys, not merely unknown fields.
    if document.schema_version != 1
        || encode(&document)? != bytes
        || document.project_id != project
        || document.run_id != run
        || document.expected_project_revision == 0
        || document.expected_project_revision > 9_007_199_254_740_991
        || document.created_at_unix_ms == 0
        || document.created_at_unix_ms > 9_007_199_254_740_991
        || document.records.len() > MAX_RECORDS
    {
        return Err(INVALID);
    }
    ProjectId::parse(project).map_err(|_| INVALID)?;
    RunId::parse(run).map_err(|_| INVALID)?;
    let mut view = ChatSnapshot {
        schema_version: 1,
        project_id: project.into(),
        expected_project_revision: document.expected_project_revision,
        run_id: run.into(),
        agent: document.agent,
        revision: 1,
        status: ChatStatus::Starting,
        next_turn: 1,
        prompts: vec![],
        updates: vec![],
        error: None,
    };
    let mut timestamp = document.created_at_unix_ms;
    for (index, record) in document.records.iter().enumerate() {
        if record.sequence != index as u64 + 1
            || record.recorded_at_unix_ms < timestamp
            || record.recorded_at_unix_ms > 9_007_199_254_740_991
            || !apply(&mut view, &record.kind)?
            || record.caused_by_turn != view.next_turn - 1
        {
            return Err(INVALID);
        }
        timestamp = record.recorded_at_unix_ms;
    }
    Ok((document, view))
}

fn apply(view: &mut ChatSnapshot, kind: &ChatRecordKind) -> Result<bool> {
    match kind {
        ChatRecordKind::Prompt { turn_id, prompt } => {
            validate_prompt(prompt).map_err(|_| INVALID)?;
            if view.status != ChatStatus::Idle
                || *turn_id != view.next_turn
                || view.prompts.len() >= 64
            {
                return Err(INVALID);
            }
            view.prompts.push(prompt.clone());
            view.next_turn += 1;
            view.status = ChatStatus::Active;
        }
        ChatRecordKind::State { status, error } => {
            if error.as_ref().is_some_and(|code| {
                !matches!(
                    code.as_str(),
                    "all-chat-restart-interrupted"
                        | "all-chat-session-interrupted"
                        | "all-chat-authentication-required"
                        | "all-chat-capability-unavailable"
                        | "all-chat-stream-invalid"
                )
            }) {
                return Err(INVALID);
            }
            if view.status == *status && view.error == *error {
                return Ok(false);
            }
            let valid = match status {
                ChatStatus::Idle => {
                    view.status == ChatStatus::Starting
                        || view.status == ChatStatus::Active
                            && view.updates.last().is_some_and(|u| {
                                matches!(
                                    u.kind,
                                    AcpV1UpdateKind::Turn {
                                        status: AcpV1TurnStatus::Completed,
                                        ..
                                    }
                                )
                            })
                }
                ChatStatus::Closing => matches!(
                    view.status,
                    ChatStatus::Starting | ChatStatus::Idle | ChatStatus::Active
                ),
                ChatStatus::Closed => {
                    matches!(view.status, ChatStatus::Closing | ChatStatus::Interrupted)
                }
                ChatStatus::Interrupted => {
                    !matches!(view.status, ChatStatus::Closed | ChatStatus::Interrupted)
                }
                _ => false,
            };
            if !valid {
                return Err(INVALID);
            }
            view.status = *status;
            view.error = error.clone();
        }
        ChatRecordKind::Update { update } => {
            if let Some(prior) = view
                .updates
                .iter()
                .find(|prior| prior.sequence == update.sequence)
            {
                return if prior == update {
                    Ok(false)
                } else {
                    Err(INVALID)
                };
            }
            validate_update(view, update)?;
            view.updates.push(update.clone());
        }
    }
    view.revision += 1;
    if serde_json::to_vec(view).map_err(|_| INVALID)?.len() > MAX_CHAT_DOCUMENT_BYTES {
        return Err("all-chat-history-capacity");
    }
    Ok(true)
}

fn opaque(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
}
fn label(s: &str) -> bool {
    !s.is_empty() && s.len() <= 4096 && !s.chars().any(char::is_control)
}
fn validate_update(view: &ChatSnapshot, update: &AcpV1Update) -> Result<()> {
    if matches!(view.status, ChatStatus::Closed | ChatStatus::Interrupted)
        || update.schema_version != 1
        || update.run_id.as_str() != view.run_id
        || update.role != OrchestrationRole::Primary
        || update.sequence != view.updates.len() as u64 + 1
        || view.updates.len() >= 2048
        || update.connection_id.len() != 32
        || !update
            .connection_id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        || view
            .updates
            .first()
            .is_some_and(|first| first.connection_id != update.connection_id)
    {
        return Err(INVALID);
    }
    let binding = match &update.kind {
        AcpV1UpdateKind::Session { info } => {
            let lists = [&info.auth_method_ids, &info.mode_ids, &info.model_ids];
            if lists.iter().any(|ids| {
                ids.len() > 64
                    || ids
                        .iter()
                        .enumerate()
                        .any(|(i, id)| !opaque(id) || ids[..i].contains(id))
            }) || [&info.adapter, &info.current_mode_id, &info.current_model_id]
                .iter()
                .any(|id| id.as_ref().is_some_and(|s| !opaque(s)))
                || info.load_enabled
                || info.resume_enabled
                || info.mode_selection_enabled
                || info.model_selection_enabled
            {
                return Err(INVALID);
            }
            return Ok(());
        }
        AcpV1UpdateKind::PermissionPending { request } => &request.binding,
        AcpV1UpdateKind::Turn { binding, .. }
        | AcpV1UpdateKind::Text { binding, .. }
        | AcpV1UpdateKind::Plan { binding, .. }
        | AcpV1UpdateKind::Tool { binding, .. }
        | AcpV1UpdateKind::PermissionResolved { binding, .. } => binding,
    };
    if binding.connection_id != update.connection_id
        || binding.run_id != update.run_id
        || binding.role != update.role
        || binding.turn_id != view.next_turn - 1
        || !opaque(&binding.session_id)
    {
        return Err(INVALID);
    }
    let last_turn = view.updates.iter().rev().find_map(|u| {
        if let AcpV1UpdateKind::Turn { binding, status } = &u.kind {
            Some((binding, *status))
        } else {
            None
        }
    });
    if matches!(
        update.kind,
        AcpV1UpdateKind::Turn {
            status: AcpV1TurnStatus::Running,
            ..
        }
    ) {
        if !matches!(view.status, ChatStatus::Active | ChatStatus::Closing)
            || last_turn.is_some_and(|(prior, status)| {
                prior.session_id != binding.session_id
                    || prior.turn_id + 1 != binding.turn_id
                    || status != AcpV1TurnStatus::Completed
            })
            || last_turn.is_none() && binding.turn_id != 1
        {
            return Err(INVALID);
        }
    } else if !last_turn
        .is_some_and(|(prior, status)| prior == binding && status == AcpV1TurnStatus::Running)
    {
        return Err(INVALID);
    }
    match &update.kind {
        AcpV1UpdateKind::Text { content, .. } if content.is_empty() || content.len() > 65536 => return Err(INVALID),
        AcpV1UpdateKind::Plan { entries, .. } if entries.len() > 64 || entries.iter().any(|entry| !label(&entry.content)) => return Err(INVALID),
        AcpV1UpdateKind::Tool { tool_call_id, title, .. } if !opaque(tool_call_id) || title.as_ref().is_some_and(|s| !label(s)) => return Err(INVALID),
        AcpV1UpdateKind::PermissionPending { request } => {
            if pending(view).is_some() || !opaque(&request.tool_call_id) || !label(&request.title) || request.request_id == 0 || request.request_id > 9_007_199_254_740_991
                || view.updates.iter().any(|u| matches!(&u.kind, AcpV1UpdateKind::PermissionPending { request: prior } if prior.request_id >= request.request_id))
                || request.options.is_empty() || request.options.len() > 16 || request.options.iter().enumerate().any(|(i, option)| !opaque(&option.option_id) || !label(&option.name) || request.options[..i].iter().any(|p| p.option_id == option.option_id) || option.enabled != matches!(option.kind, AcpV1PermissionKind::AllowOnce | AcpV1PermissionKind::RejectOnce)) { return Err(INVALID); }
        }
        AcpV1UpdateKind::PermissionResolved { request_id, choice, .. } => {
            let request = pending(view).filter(|p| p.binding == *binding && p.request_id == *request_id).ok_or(INVALID)?;
            if let AcpV1PermissionChoice::Select { option_id } = choice
                && !request.options.iter().any(|o| o.option_id == *option_id && o.enabled) { return Err(INVALID); }
        }
        _ => {}
    }
    Ok(())
}

fn pending(view: &ChatSnapshot) -> Option<&qiongli_execution::AcpV1PermissionRequest> {
    for update in view.updates.iter().rev() {
        match &update.kind {
            AcpV1UpdateKind::PermissionPending { request } => return Some(request),
            AcpV1UpdateKind::PermissionResolved { .. } | AcpV1UpdateKind::Turn { .. } => {
                return None;
            }
            _ => {}
        }
    }
    None
}

pub fn all_chat_history_schema_json() -> std::result::Result<String, serde_json::Error> {
    let mut schema = schemars::generate::SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator()
        .into_root_schema_for::<ChatDocument>();
    schema.insert(
        "$id".into(),
        "https://qiongli.dev/schemas/project/all-chat-history-v1.json".into(),
    );
    serde_json::to_string_pretty(&schema).map(|s| s + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use qiongli_project::{ApprovedProjectMutation, ProjectKind, ProjectRegistrationOptions};
    use serde_json::json;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    fn fixture() -> (PathBuf, ProjectStateService, ChatSnapshot) {
        let root = std::env::temp_dir().join(format!(
            "qiongli-chat-history-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let config =
            qiongli_config::resolve_config_root(Some(root.join("config").as_os_str()), &root)
                .unwrap();
        let projects = ProjectStateService::new(config);
        let plan = projects
            .preview_create(
                root.join("article"),
                ProjectRegistrationOptions::new("History fixture", ProjectKind::Article),
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
        let view = ChatSnapshot {
            schema_version: 1,
            project_id,
            expected_project_revision: 1,
            run_id: format!("run_{}", "a".repeat(32)),
            agent: ChatAgent::OfflineDemo,
            revision: 1,
            status: ChatStatus::Starting,
            next_turn: 1,
            prompts: vec![],
            updates: vec![],
            error: None,
        };
        (root, projects, view)
    }

    fn intent() -> ChatRecordKind {
        ChatRecordKind::Prompt {
            turn_id: 1,
            prompt: ChatPrompt {
                text: "PRIVATE_CHAT_CANARY".into(),
                context: "Private supplied context".into(),
                source_refs: vec!["source-label".into()],
            },
        }
    }

    #[test]
    fn all_chat_history_recovers_committed_intent_without_replay_and_excludes_export() {
        let (root, projects, mut view) = fixture();
        let lease = ChatHistory::acquire(&projects, &view.project_id).unwrap();
        let mut history = ChatHistory::create(projects.clone(), lease, &view).unwrap();
        view = history
            .append(
                &view,
                ChatRecordKind::State {
                    status: ChatStatus::Idle,
                    error: None,
                },
            )
            .unwrap();
        // Crash after commit, before the caller accepts the projection or sends the prompt.
        let committed = history.append(&view, intent()).unwrap();
        assert_eq!(view.prompts.len(), 0);
        assert_eq!(committed.prompts.len(), 1);
        drop(history);
        let (history, restored) = ChatHistory::load_latest(&projects, &view.project_id)
            .unwrap()
            .unwrap();
        assert_eq!(restored.status, ChatStatus::Interrupted);
        assert_eq!(restored.prompts[0].text, "PRIVATE_CHAT_CANARY");
        assert!(!format!("{restored:?} {:?}", restored.prompts[0]).contains("PRIVATE_CHAT_CANARY"));
        assert!(restored.updates.is_empty());
        assert_eq!(
            restored.error.as_deref(),
            Some("all-chat-restart-interrupted")
        );
        let path = root
            .join("article/.qiongli/all-chat")
            .join(format!("{}.json", view.run_id));
        let saved = fs::read(&path).unwrap();
        drop(history);
        let (_, second) = ChatHistory::load_latest(&projects, &view.project_id)
            .unwrap()
            .unwrap();
        assert_eq!(second.revision, restored.revision);
        assert_eq!(fs::read(&path).unwrap(), saved);
        // Export executes its real privacy filter, including private chat and lock files.
        let id = ProjectId::parse(&view.project_id).unwrap();
        let export = projects.preview_export(&id, root.join("export")).unwrap();
        let debug = format!("{export:?} {:?}", projects.snapshot().unwrap());
        assert!(!debug.contains("PRIVATE_CHAT_CANARY"));
        projects
            .apply_portable(
                &export,
                &ApprovedProjectMutation::new(export.preview().plan_digest.clone(), true),
                2,
            )
            .unwrap();
        assert!(!root.join("export/project/.qiongli").exists());
        // Starting another run archives the prior bytes, and loads only the newest run.
        view.run_id = format!("run_{}", "b".repeat(32));
        view.status = ChatStatus::Starting;
        let lease = ChatHistory::acquire(&projects, &view.project_id).unwrap();
        drop(ChatHistory::create(projects.clone(), lease, &view).unwrap());
        assert_eq!(fs::read(&path).unwrap(), saved);
        assert_eq!(
            ChatHistory::load_latest(&projects, &view.project_id)
                .unwrap()
                .unwrap()
                .1
                .run_id,
            view.run_id
        );
        // Deliberate deletion is exact and performed only with writers stopped.
        fs::remove_file(
            root.join("article/.qiongli/all-chat")
                .join(format!("{}.json", view.run_id)),
        )
        .unwrap();
        assert_eq!(
            ChatHistory::load_latest(&projects, &view.project_id)
                .unwrap()
                .unwrap()
                .1
                .prompts
                .len(),
            1
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn all_chat_history_rejects_corruption_future_versions_and_stale_writes_without_overwrite() {
        let (root, projects, mut view) = fixture();
        let lease = ChatHistory::acquire(&projects, &view.project_id).unwrap();
        let mut history = ChatHistory::create(projects.clone(), lease, &view).unwrap();
        view = history
            .append(
                &view,
                ChatRecordKind::State {
                    status: ChatStatus::Idle,
                    error: None,
                },
            )
            .unwrap();
        view = history.append(&view, intent()).unwrap();
        let bytes = encode(&history.document).unwrap();
        let mut future: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        future["schemaVersion"] = json!(2);
        let path = root
            .join("article/.qiongli/all-chat")
            .join(format!("{}.json", view.run_id));
        let corrupt = serde_json_canonicalizer::to_vec(&future).unwrap();
        fs::write(&path, &corrupt).unwrap();
        // CAS failure poisons the writer: it must never retry a send after uncertain commit.
        assert!(
            history
                .append(
                    &view,
                    ChatRecordKind::State {
                        status: ChatStatus::Interrupted,
                        error: None
                    }
                )
                .is_err()
        );
        assert!(history.failed);
        drop(history);
        for bad in [
            b"{".to_vec(),
            bytes[..bytes.len() - 1].to_vec(),
            corrupt,
            [b"{\"schemaVersion\":1,".as_slice(), &bytes[1..]].concat(),
        ] {
            fs::write(&path, &bad).unwrap();
            assert!(ChatHistory::load_latest(&projects, &view.project_id).is_err());
            let lease = ChatHistory::acquire(&projects, &view.project_id).unwrap();
            assert!(ChatHistory::create(projects.clone(), lease, &view).is_err());
            assert_eq!(fs::read(&path).unwrap(), bad);
        }
        for mutate in [
            |v: &mut serde_json::Value| v["records"][0]["sequence"] = json!(2),
            |v: &mut serde_json::Value| v["records"][0]["recordedAtUnixMs"] = json!(0),
            |v: &mut serde_json::Value| v["records"][1]["causedByTurn"] = json!(2),
            |v: &mut serde_json::Value| v["records"][1]["kind"]["turnId"] = json!(2),
        ] {
            let mut value = serde_json::from_slice(&bytes).unwrap();
            mutate(&mut value);
            assert!(
                decode(
                    &serde_json_canonicalizer::to_vec(&value).unwrap(),
                    &view.project_id,
                    &view.run_id
                )
                .is_err()
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn all_chat_history_deduplicates_bound_updates_and_freezes_schema_and_fixture() {
        let (root, projects, mut view) = fixture();
        let lease = ChatHistory::acquire(&projects, &view.project_id).unwrap();
        let mut history = ChatHistory::create(projects, lease, &view).unwrap();
        view = history
            .append(
                &view,
                ChatRecordKind::State {
                    status: ChatStatus::Idle,
                    error: None,
                },
            )
            .unwrap();
        view = history.append(&view, intent()).unwrap();
        let update: AcpV1Update = serde_json::from_value(json!({"schemaVersion":1,"connectionId":"0".repeat(32),"runId":view.run_id,"role":"primary","sequence":1,"kind":{"type":"turn","binding":{"connectionId":"0".repeat(32),"runId":view.run_id,"role":"primary","sessionId":"fixture","turnId":1},"status":"running"}})).unwrap();
        view = history
            .append(
                &view,
                ChatRecordKind::Update {
                    update: update.clone(),
                },
            )
            .unwrap();
        let original = encode(&history.document).unwrap();
        assert_eq!(
            history
                .append(
                    &view,
                    ChatRecordKind::Update {
                        update: update.clone()
                    }
                )
                .unwrap()
                .revision,
            view.revision
        );
        assert_eq!(encode(&history.document).unwrap(), original);
        let mut conflicting = update.clone();
        conflicting.connection_id = "1".repeat(32);
        assert!(
            history
                .append(
                    &view,
                    ChatRecordKind::Update {
                        update: conflicting
                    }
                )
                .is_err()
        );
        let mut stale = update;
        stale.sequence = 2;
        if let AcpV1UpdateKind::Turn { binding, status } = &mut stale.kind {
            binding.turn_id = 2;
            *status = AcpV1TurnStatus::Completed;
        }
        assert!(
            history
                .append(&view, ChatRecordKind::Update { update: stale })
                .is_err()
        );
        // Stable private fixture comes from the same Rust serializer and recovery decoder.
        let mut document: ChatDocument =
            serde_json::from_slice(&encode(&history.document).unwrap()).unwrap();
        document.project_id = format!("prj_{}", "0".repeat(32));
        document.run_id = format!("run_{}", "0".repeat(32));
        document.created_at_unix_ms = 1;
        for record in &mut document.records {
            record.recorded_at_unix_ms = record.sequence;
            if let ChatRecordKind::Update { update } = &mut record.kind {
                update.run_id = RunId::parse(&document.run_id).unwrap();
                if let AcpV1UpdateKind::Turn { binding, .. } = &mut update.kind {
                    binding.run_id = update.run_id.clone();
                }
            }
        }
        let fixture_bytes = encode(&document).unwrap();
        assert_eq!(
            decode(&fixture_bytes, &document.project_id, &document.run_id)
                .unwrap()
                .1
                .prompts
                .len(),
            1
        );
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let schema = all_chat_history_schema_json().unwrap();
        if std::env::var_os("QIONGLI_UPDATE_CHAT_HISTORY_FIXTURE").is_some() {
            fs::write(
                base.join("tests/fixtures/all-chat-history-v1.json"),
                &fixture_bytes,
            )
            .unwrap();
            fs::write(
                base.join("schemas/all-chat-history-v1.schema.json"),
                &schema,
            )
            .unwrap();
        }
        assert_eq!(
            fs::read(base.join("tests/fixtures/all-chat-history-v1.json")).unwrap(),
            fixture_bytes
        );
        assert_eq!(
            fs::read_to_string(base.join("schemas/all-chat-history-v1.schema.json")).unwrap(),
            schema
        );
        // Reserve closure space at capacity: repeated bounded plan updates are valid
        // observations, and rejected input never changes the last committed bytes.
        let projects = history.projects.clone();
        let mut value = serde_json::to_value(&view.updates[0]).unwrap();
        let binding = value["kind"]["binding"].clone();
        value["kind"] = json!({"type":"plan", "binding":binding, "entries":vec![json!({"content":"x".repeat(4096),"status":"pending"});64]});
        for sequence in 2..=40 {
            value["sequence"] = json!(sequence);
            let before = encode(&history.document).unwrap();
            match history.append(
                &view,
                ChatRecordKind::Update {
                    update: serde_json::from_value(value.clone()).unwrap(),
                },
            ) {
                Ok(next) => view = next,
                Err(error) => {
                    assert_eq!(error, "all-chat-history-capacity");
                    assert_eq!(encode(&history.document).unwrap(), before);
                    break;
                }
            }
        }
        assert!(history.document.records.len() < 40);
        // Fill the remaining gap with valid text; the last attempted record would
        // leave only ten bytes, too little for an interruption receipt.
        loop {
            value["sequence"] = json!(view.updates.len() + 1);
            value["kind"] = json!({"type":"text", "binding":binding, "content":"x"});
            let record = ChatRecord {
                sequence: history.document.records.len() as u64 + 1,
                recorded_at_unix_ms: now().unwrap(),
                caused_by_turn: 1,
                kind: ChatRecordKind::Update {
                    update: serde_json::from_value(value.clone()).unwrap(),
                },
            };
            history.document.records.push(record);
            let gap = MAX_CHAT_DOCUMENT_BYTES - encode(&history.document).unwrap().len() - 10;
            history.document.records.pop();
            value["kind"]["content"] = json!("x".repeat(gap.min(65536)));
            let result = history.append(
                &view,
                ChatRecordKind::Update {
                    update: serde_json::from_value(value.clone()).unwrap(),
                },
            );
            if gap <= 65536 {
                assert_eq!(result.unwrap_err(), "all-chat-history-capacity");
                break;
            }
            view = result.unwrap();
        }
        drop(history);
        assert_eq!(
            ChatHistory::load_latest(&projects, &view.project_id)
                .unwrap()
                .unwrap()
                .1
                .status,
            ChatStatus::Interrupted
        );
        document.records = vec![document.records[0].clone(); MAX_RECORDS + 1];
        assert!(
            decode(
                &encode(&document).unwrap(),
                &document.project_id,
                &document.run_id
            )
            .is_err()
        );
        document.records[0].kind = ChatRecordKind::Prompt {
            turn_id: 1,
            prompt: ChatPrompt {
                text: "x".repeat(MAX_CHAT_DOCUMENT_BYTES),
                context: String::new(),
                source_refs: vec![],
            },
        };
        assert_eq!(encode(&document).unwrap_err(), "all-chat-history-capacity");
        fs::remove_dir_all(root).unwrap();
    }
}
