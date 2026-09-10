use std::collections::BTreeMap;
use std::path::{Component, Path};

use unicode_normalization::UnicodeNormalization;

use crate::ProjectError;
use crate::academic_graph::{
    AcademicGraphConfidence, AcademicGraphDiagnosticCode, AcademicGraphDiagnosticV1,
    AcademicGraphEdgeStatus, AcademicGraphEdgeV1, AcademicGraphIdentityScope, AcademicGraphLayer,
    AcademicGraphNodeType, AcademicGraphNodeV1, AcademicGraphRelation, AcademicInferenceStrength,
};
use crate::academic_graph_coverage::{
    AcademicGraphExtractorV1, AcademicGraphPortableAuthorityV1, academic_graph_source_coverage,
};
use crate::model::ProjectId;

const RESEARCH_STATE_PATH: &str = "context/research_state.md";
const DECISION_LOG_PATH: &str = "context/decision_log.md";
const BOUNDARY_REVIEW_PATH: &str = "context/boundary_review.md";
const IDEA_FUNNEL_PATH: &str = "context/idea_funnel.md";
const LITERATURE_MAP_PATH: &str = "literature/literature_map.md";
const EVIDENCE_LEDGER_PATH: &str = "evidence/claim-evidence-ledger.csv";
const MANUSCRIPT_CLAIM_MAP_PATH: &str = "manuscript/claims_evidence_map.md";
const MAX_TABLE_COLUMNS: usize = 16;
const MAX_TABLE_ROWS: usize = 2_048;
const MAX_FIELD_BYTES: usize = 8 * 1_024;
const MAX_RELATED_ID_BYTES: usize = 512;
const MAX_LIST_ITEMS: usize = 64;

const EVIDENCE_COLUMNS: [&str; 10] = [
    "claim_id",
    "claim_text",
    "claim_type",
    "evidence_type",
    "source_id",
    "source_location",
    "artifact_path",
    "confidence",
    "limitations",
    "status",
];

const CLAIM_TYPES: [&str; 6] = [
    "finding",
    "interpretation",
    "implication",
    "method_assumption",
    "limitation",
    "speculation",
];

const EVIDENCE_TYPES: [&str; 6] = [
    "paper",
    "dataset",
    "analysis_result",
    "theory",
    "artifact",
    "gap_note",
];

pub(crate) struct ExtractedAcademicGraph {
    pub(crate) nodes: Vec<AcademicGraphNodeV1>,
    pub(crate) edges: Vec<AcademicGraphEdgeV1>,
    pub(crate) diagnostics: Vec<AcademicGraphDiagnosticV1>,
}

impl ExtractedAcademicGraph {
    fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

pub(crate) fn extract_academic_artifact(
    project_id: &ProjectId,
    artifact_path: &str,
    bytes: &[u8],
) -> Result<ExtractedAcademicGraph, ProjectError> {
    let coverage = academic_graph_source_coverage(artifact_path)
        .filter(|coverage| {
            coverage.portable_authority == AcademicGraphPortableAuthorityV1::CanonicalArtifact
        })
        .ok_or(ProjectError::ProjectArtifactUnsupported)?;
    let extracted = match coverage.extractor {
        AcademicGraphExtractorV1::ResearchState => extract_research_state(project_id, bytes),
        AcademicGraphExtractorV1::DecisionLog => extract_decision_log(project_id, bytes),
        AcademicGraphExtractorV1::StageHandoffStructural => ExtractedAcademicGraph::empty(),
        AcademicGraphExtractorV1::BoundaryReview => extract_boundary_review(project_id, bytes),
        AcademicGraphExtractorV1::IdeaFunnel => extract_idea_funnel(project_id, bytes),
        AcademicGraphExtractorV1::LiteratureMap => extract_literature_map(project_id, bytes),
        AcademicGraphExtractorV1::ClaimEvidenceLedger => extract_evidence_ledger(project_id, bytes),
        AcademicGraphExtractorV1::ManuscriptClaimMap => {
            extract_manuscript_claim_map(project_id, bytes)
        }
        AcademicGraphExtractorV1::ProjectManifest | AcademicGraphExtractorV1::SemanticLinks => {
            return Err(ProjectError::ProjectArtifactUnsupported);
        }
    };
    Ok(extracted)
}

fn extract_research_state(project_id: &ProjectId, bytes: &[u8]) -> ExtractedAcademicGraph {
    let mut projection = ExtractedAcademicGraph::empty();
    let Ok(text) = std::str::from_utf8(bytes) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            RESEARCH_STATE_PATH,
            Some("document"),
            None,
        ));
        return projection;
    };

    let mut question = None;
    let mut contribution = None;
    for (line_index, line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        let field = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        let Some((raw_key, raw_value)) = field.split_once(':') else {
            continue;
        };
        let key = normalize_header(raw_key);
        let value = raw_value.trim();
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            "rq" | "main_question_or_thesis" | "current_research_question" => {
                assign_unique_field(
                    &mut question,
                    value,
                    line_number,
                    "field:main_question_or_thesis",
                    RESEARCH_STATE_PATH,
                    &mut projection.diagnostics,
                );
            }
            "contribution_claim" => {
                assign_unique_field(
                    &mut contribution,
                    value,
                    line_number,
                    "field:contribution_claim",
                    RESEARCH_STATE_PATH,
                    &mut projection.diagnostics,
                );
            }
            _ => {}
        }
    }

    if let Some((label, _)) = question {
        push_node(
            &mut projection,
            project_id,
            AcademicGraphNodeType::ResearchQuestion,
            "research-question:current",
            label,
            vec![
                AcademicGraphLayer::IdeaDecision,
                AcademicGraphLayer::Argument,
                AcademicGraphLayer::Combined,
            ],
            RESEARCH_STATE_PATH,
            "field:main_question_or_thesis",
        );
    } else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::MissingStableId,
            RESEARCH_STATE_PATH,
            Some("field:main_question_or_thesis"),
            Some("research-question:current"),
        ));
    }

    if let Some((label, _)) = contribution {
        push_node(
            &mut projection,
            project_id,
            AcademicGraphNodeType::Contribution,
            "contribution:current",
            label,
            vec![
                AcademicGraphLayer::IdeaDecision,
                AcademicGraphLayer::Argument,
                AcademicGraphLayer::Manuscript,
                AcademicGraphLayer::Combined,
            ],
            RESEARCH_STATE_PATH,
            "field:contribution_claim",
        );
    }
    projection
}

fn assign_unique_field(
    slot: &mut Option<(String, usize)>,
    value: &str,
    line_number: usize,
    anchor: &str,
    artifact_path: &str,
    diagnostics: &mut Vec<AcademicGraphDiagnosticV1>,
) {
    match slot {
        None => *slot = Some((value.to_string(), line_number)),
        Some((current, _)) if current == value => {}
        Some(_) => diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::ConflictingIdentity,
            artifact_path,
            Some(anchor),
            Some(anchor),
        )),
    }
}

fn extract_decision_log(project_id: &ProjectId, bytes: &[u8]) -> ExtractedAcademicGraph {
    let mut projection = ExtractedAcademicGraph::empty();
    let Ok(text) = std::str::from_utf8(bytes) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            DECISION_LOG_PATH,
            Some("document"),
            None,
        ));
        return projection;
    };

    let rows = match decision_rows(text) {
        Ok(rows) => rows,
        Err(anchor) => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                DECISION_LOG_PATH,
                Some(anchor),
                None,
            ));
            return projection;
        }
    };
    let mut seen = BTreeMap::new();
    for row in rows {
        let decision_id = row.decision_id.trim();
        let decision = row.decision.trim();
        let anchor = format!("row:{}", row.line_number);
        if decision.is_empty() {
            continue;
        }
        if !valid_reference_id(decision_id) {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::MissingStableId,
                DECISION_LOG_PATH,
                Some(&anchor),
                safe_related_id(decision_id),
            ));
            continue;
        }
        if seen
            .insert(decision_id.to_string(), decision.to_string())
            .is_some()
        {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::ConflictingIdentity,
                DECISION_LOG_PATH,
                Some(&anchor),
                Some(decision_id),
            ));
            continue;
        }
        if !row.status.is_empty() && !valid_decision_status(&row.status) {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                DECISION_LOG_PATH,
                Some(&anchor),
                Some(decision_id),
            ));
        }
        push_node(
            &mut projection,
            project_id,
            AcademicGraphNodeType::Decision,
            decision_id,
            decision,
            vec![
                AcademicGraphLayer::IdeaDecision,
                AcademicGraphLayer::Combined,
            ],
            DECISION_LOG_PATH,
            format!("decision:{decision_id}"),
        );
    }
    projection
}

struct DecisionRow {
    line_number: usize,
    decision_id: String,
    status: String,
    decision: String,
}

fn decision_rows(text: &str) -> Result<Vec<DecisionRow>, &'static str> {
    let lines = text.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let Some(header) = markdown_cells(line) else {
            continue;
        };
        let normalized = header
            .iter()
            .map(|cell| normalize_header(cell))
            .collect::<Vec<_>>();
        let Some(id_index) = normalized.iter().position(|cell| cell == "decision_id") else {
            continue;
        };
        let Some(decision_index) = normalized.iter().position(|cell| cell == "decision") else {
            continue;
        };
        let status_index = normalized.iter().position(|cell| cell == "status");
        let mut rows = Vec::new();
        for (row_index, line) in lines.iter().enumerate().skip(index + 1) {
            let Some(cells) = markdown_cells(line) else {
                if !rows.is_empty() {
                    break;
                }
                continue;
            };
            if markdown_separator(&cells) {
                continue;
            }
            if cells.len() != header.len() || rows.len() >= MAX_TABLE_ROWS {
                return Err("table");
            }
            rows.push(DecisionRow {
                line_number: row_index + 1,
                decision_id: cells[id_index].clone(),
                status: status_index
                    .and_then(|position| cells.get(position))
                    .cloned()
                    .unwrap_or_default(),
                decision: cells[decision_index].clone(),
            });
        }
        return Ok(rows);
    }

    let records = parse_csv(text).map_err(|()| "csv")?;
    let Some(header) = records.first() else {
        return Err("header");
    };
    let normalized = header
        .fields
        .iter()
        .map(|field| normalize_header(field))
        .collect::<Vec<_>>();
    let id_index = normalized
        .iter()
        .position(|field| field == "decision_id")
        .ok_or("header")?;
    let decision_index = normalized
        .iter()
        .position(|field| field == "decision")
        .ok_or("header")?;
    let status_index = normalized.iter().position(|field| field == "status");
    let header_len = header.fields.len();
    let mut rows = Vec::new();
    for record in records.into_iter().skip(1) {
        if record.fields.len() != header_len {
            return Err("csv");
        }
        rows.push(DecisionRow {
            line_number: record.line_number,
            decision_id: record.fields[id_index].clone(),
            status: status_index
                .and_then(|position| record.fields.get(position))
                .cloned()
                .unwrap_or_default(),
            decision: record.fields[decision_index].clone(),
        });
    }
    Ok(rows)
}

fn extract_idea_funnel(project_id: &ProjectId, bytes: &[u8]) -> ExtractedAcademicGraph {
    let mut projection = ExtractedAcademicGraph::empty();
    let Ok(text) = std::str::from_utf8(bytes) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            IDEA_FUNNEL_PATH,
            Some("document"),
            None,
        ));
        return projection;
    };
    let table = match find_markdown_table(
        text,
        &[
            "idea_id",
            "one_sentence_idea",
            "candidate_gap",
            "triage_decision",
        ],
    ) {
        Ok(table) => table,
        Err(anchor) => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                IDEA_FUNNEL_PATH,
                Some(anchor),
                None,
            ));
            return projection;
        }
    };
    let mut ideas = BTreeMap::new();
    let mut gaps = BTreeMap::new();
    let mut edges = BTreeMap::new();
    for row in &table.rows {
        let idea_id = table_cell(&table, row, "idea_id");
        let label = table_cell(&table, row, "one_sentence_idea");
        let candidate_gap = table_cell(&table, row, "candidate_gap");
        let decision = table_cell(&table, row, "triage_decision").to_ascii_lowercase();
        let anchor = format!("row:{}", row.line_number);
        if idea_id.is_empty() && label.is_empty() {
            continue;
        }
        if !valid_reference_id(idea_id) || label.is_empty() {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::MissingStableId,
                IDEA_FUNNEL_PATH,
                Some(&anchor),
                safe_related_id(idea_id),
            ));
            continue;
        }
        if ideas.contains_key(idea_id) {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::ConflictingIdentity,
                IDEA_FUNNEL_PATH,
                Some(&anchor),
                Some(idea_id),
            ));
            continue;
        }
        if !matches!(decision.as_str(), "keep" | "revise" | "reject") {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                IDEA_FUNNEL_PATH,
                Some(&anchor),
                Some(idea_id),
            ));
        }
        let idea = match AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Idea,
            AcademicGraphIdentityScope::Project,
            idea_id,
            label,
            vec![
                AcademicGraphLayer::IdeaDecision,
                AcademicGraphLayer::Combined,
            ],
            IDEA_FUNNEL_PATH,
            format!("idea:{idea_id}"),
        ) {
            Ok(node) => node,
            Err(_) => {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::UnsupportedRelation,
                    IDEA_FUNNEL_PATH,
                    Some(&anchor),
                    Some(idea_id),
                ));
                continue;
            }
        };
        ideas.insert(idea_id.to_string(), idea.clone());
        if candidate_gap.is_empty() {
            continue;
        }
        let gap_id = format!("idea-gap:{idea_id}");
        let gap = match AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Gap,
            AcademicGraphIdentityScope::Project,
            &gap_id,
            candidate_gap,
            vec![
                AcademicGraphLayer::IdeaDecision,
                AcademicGraphLayer::Literature,
                AcademicGraphLayer::Combined,
            ],
            IDEA_FUNNEL_PATH,
            format!("candidate-gap:{idea_id}"),
        ) {
            Ok(node) => node,
            Err(_) => {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::UnsupportedRelation,
                    IDEA_FUNNEL_PATH,
                    Some(&anchor),
                    Some(idea_id),
                ));
                continue;
            }
        };
        let edge = AcademicGraphEdgeV1::new(
            project_id,
            &idea.node_id,
            AcademicGraphRelation::AddressesGap,
            &gap.node_id,
            vec![
                AcademicGraphLayer::IdeaDecision,
                AcademicGraphLayer::Literature,
                AcademicGraphLayer::Combined,
            ],
            "Candidate idea triage associates this idea with the recorded candidate gap.",
            IDEA_FUNNEL_PATH,
            format!("candidate-gap:{idea_id}"),
            "The idea funnel records a proposed gap; literature evidence may not yet verify it.",
            AcademicInferenceStrength::UnsupportedGap,
            AcademicGraphConfidence::Unknown,
            if decision == "reject" {
                AcademicGraphEdgeStatus::Rejected
            } else {
                AcademicGraphEdgeStatus::Proposed
            },
            None,
        );
        match edge {
            Ok(edge) => {
                gaps.insert(gap_id, gap);
                edges.insert(edge.edge_id.clone(), edge);
            }
            Err(_) => projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                IDEA_FUNNEL_PATH,
                Some(&anchor),
                Some(idea_id),
            )),
        }
    }
    match scalar_value(text, "recommended_idea_id") {
        Ok(Some(recommended)) if !valid_reference_id(&recommended) => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::MissingStableId,
                IDEA_FUNNEL_PATH,
                Some("field:recommended_idea_id"),
                safe_related_id(&recommended),
            ));
        }
        Ok(Some(recommended)) if !ideas.contains_key(&recommended) => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::DanglingNode,
                IDEA_FUNNEL_PATH,
                Some("field:recommended_idea_id"),
                Some(&recommended),
            ));
        }
        Err(()) => projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::ConflictingIdentity,
            IDEA_FUNNEL_PATH,
            Some("field:recommended_idea_id"),
            None,
        )),
        Ok(Some(_)) | Ok(None) => {}
    }
    projection.nodes.extend(ideas.into_values());
    projection.nodes.extend(gaps.into_values());
    projection.edges.extend(edges.into_values());
    projection
}

fn extract_boundary_review(project_id: &ProjectId, bytes: &[u8]) -> ExtractedAcademicGraph {
    let mut projection = ExtractedAcademicGraph::empty();
    let Ok(text) = std::str::from_utf8(bytes) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            BOUNDARY_REVIEW_PATH,
            Some("document"),
            None,
        ));
        return projection;
    };
    let mut decisions = BTreeMap::new();
    match scalar_value(text, "claim_strength") {
        Ok(Some(value)) => {
            insert_boundary_decision(
                project_id,
                &mut projection,
                &mut decisions,
                "boundary:claim-strength",
                format!("Claim strength: {value}"),
                "field:claim_strength",
            );
        }
        Err(()) => projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::ConflictingIdentity,
            BOUNDARY_REVIEW_PATH,
            Some("field:claim_strength"),
            Some("boundary:claim-strength"),
        )),
        Ok(None) => {}
    }
    if let Ok(table) = find_markdown_table(
        text,
        &[
            "question_id",
            "recommended_answer",
            "user_or_artifact_answer",
            "status",
        ],
    ) {
        for row in &table.rows {
            let question_id = table_cell(&table, row, "question_id");
            let user_answer = table_cell(&table, row, "user_or_artifact_answer");
            let recommended = table_cell(&table, row, "recommended_answer");
            let status = table_cell(&table, row, "status").to_ascii_lowercase();
            let answer = if user_answer.is_empty() {
                recommended
            } else {
                user_answer
            };
            if question_id.is_empty() && answer.is_empty() {
                continue;
            }
            let anchor = format!("row:{}", row.line_number);
            if !valid_reference_id(question_id) || answer.is_empty() {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::MissingStableId,
                    BOUNDARY_REVIEW_PATH,
                    Some(&anchor),
                    safe_related_id(question_id),
                ));
                continue;
            }
            if !matches!(
                status.as_str(),
                "open" | "answered" | "accepted" | "locked" | "resolved"
            ) {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::AmbiguousRelation,
                    BOUNDARY_REVIEW_PATH,
                    Some(&anchor),
                    Some(question_id),
                ));
            }
            insert_boundary_decision(
                project_id,
                &mut projection,
                &mut decisions,
                question_id,
                answer,
                format!("boundary-question:{question_id}"),
            );
        }
    }
    if let Ok(table) = find_markdown_table(
        text,
        &[
            "decision_id",
            "decision",
            "rationale",
            "confidence",
            "evidence_basis",
        ],
    ) {
        for row in &table.rows {
            let decision_id = table_cell(&table, row, "decision_id");
            let decision = table_cell(&table, row, "decision");
            if decision_id.is_empty() && decision.is_empty() {
                continue;
            }
            let anchor = format!("row:{}", row.line_number);
            if !valid_reference_id(decision_id) || decision.is_empty() {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::MissingStableId,
                    BOUNDARY_REVIEW_PATH,
                    Some(&anchor),
                    safe_related_id(decision_id),
                ));
                continue;
            }
            insert_boundary_decision(
                project_id,
                &mut projection,
                &mut decisions,
                decision_id,
                decision,
                format!("locked-decision:{decision_id}"),
            );
        }
    }
    projection.nodes.extend(decisions.into_values());
    projection
}

fn insert_boundary_decision(
    project_id: &ProjectId,
    projection: &mut ExtractedAcademicGraph,
    decisions: &mut BTreeMap<String, AcademicGraphNodeV1>,
    decision_id: &str,
    label: impl Into<String>,
    source_anchor: impl Into<String>,
) {
    let source_anchor = source_anchor.into();
    if !valid_reference_id(decision_id) {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::MissingStableId,
            BOUNDARY_REVIEW_PATH,
            Some(&source_anchor),
            safe_related_id(decision_id),
        ));
        return;
    }
    if decisions.contains_key(decision_id) {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::ConflictingIdentity,
            BOUNDARY_REVIEW_PATH,
            Some(&source_anchor),
            Some(decision_id),
        ));
        return;
    }
    match AcademicGraphNodeV1::new(
        project_id,
        AcademicGraphNodeType::Decision,
        AcademicGraphIdentityScope::Project,
        decision_id,
        label,
        vec![
            AcademicGraphLayer::IdeaDecision,
            AcademicGraphLayer::Argument,
            AcademicGraphLayer::Combined,
        ],
        BOUNDARY_REVIEW_PATH,
        &source_anchor,
    ) {
        Ok(node) => {
            decisions.insert(decision_id.to_string(), node);
        }
        Err(_) => projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            BOUNDARY_REVIEW_PATH,
            Some(&source_anchor),
            Some(decision_id),
        )),
    }
}

fn extract_literature_map(project_id: &ProjectId, bytes: &[u8]) -> ExtractedAcademicGraph {
    let mut projection = ExtractedAcademicGraph::empty();
    let Ok(text) = std::str::from_utf8(bytes) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            LITERATURE_MAP_PATH,
            Some("document"),
            None,
        ));
        return projection;
    };
    let cluster_table = match find_markdown_table(
        text,
        &[
            "cluster_id",
            "cluster_label",
            "basis",
            "core_argument",
            "representative_papers",
            "evidence_limits",
        ],
    ) {
        Ok(table) => table,
        Err(anchor) => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                LITERATURE_MAP_PATH,
                Some(anchor),
                None,
            ));
            return projection;
        }
    };
    let mut clusters = BTreeMap::new();
    let mut papers = BTreeMap::new();
    let mut gaps = BTreeMap::new();
    let mut edges = BTreeMap::new();
    for row in &cluster_table.rows {
        let cluster_id = table_cell(&cluster_table, row, "cluster_id");
        let label = table_cell(&cluster_table, row, "cluster_label");
        if cluster_id.is_empty() && label.is_empty() {
            continue;
        }
        let anchor = format!("row:{}", row.line_number);
        if !valid_reference_id(cluster_id) || label.is_empty() {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::MissingStableId,
                LITERATURE_MAP_PATH,
                Some(&anchor),
                safe_related_id(cluster_id),
            ));
            continue;
        }
        if clusters.contains_key(cluster_id) {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::ConflictingIdentity,
                LITERATURE_MAP_PATH,
                Some(&anchor),
                Some(cluster_id),
            ));
            continue;
        }
        match AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::LiteratureCluster,
            AcademicGraphIdentityScope::Project,
            cluster_id,
            label,
            vec![AcademicGraphLayer::Literature, AcademicGraphLayer::Combined],
            LITERATURE_MAP_PATH,
            format!("cluster:{cluster_id}"),
        ) {
            Ok(node) => {
                clusters.insert(cluster_id.to_string(), node);
            }
            Err(_) => projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                LITERATURE_MAP_PATH,
                Some(&anchor),
                Some(cluster_id),
            )),
        }
    }

    if let Ok(study_table) = find_markdown_table(
        text,
        &[
            "citekey",
            "primary_cluster_id",
            "secondary_cluster_ids",
            "evidence_limit",
            "source_anchor",
        ],
    ) {
        for row in &study_table.rows {
            let citekey = clean_citekey(table_cell(&study_table, row, "citekey"));
            let primary = table_cell(&study_table, row, "primary_cluster_id");
            let secondary = table_cell(&study_table, row, "secondary_cluster_ids");
            let evidence_limit = table_cell(&study_table, row, "evidence_limit");
            let source_anchor = table_cell(&study_table, row, "source_anchor");
            if citekey.is_empty() && primary.is_empty() {
                continue;
            }
            let row_anchor = format!("row:{}", row.line_number);
            if !valid_reference_id(citekey) {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::MissingStableId,
                    LITERATURE_MAP_PATH,
                    Some(&row_anchor),
                    safe_related_id(citekey),
                ));
                continue;
            }
            let paper_id = format!("citekey:{citekey}");
            let paper = match AcademicGraphNodeV1::new(
                project_id,
                AcademicGraphNodeType::Paper,
                AcademicGraphIdentityScope::Global,
                &paper_id,
                citekey,
                vec![AcademicGraphLayer::Literature, AcademicGraphLayer::Combined],
                LITERATURE_MAP_PATH,
                format!("paper:{citekey}"),
            ) {
                Ok(node) => node,
                Err(_) => {
                    projection.diagnostics.push(diagnostic(
                        AcademicGraphDiagnosticCode::UnsupportedRelation,
                        LITERATURE_MAP_PATH,
                        Some(&row_anchor),
                        Some(citekey),
                    ));
                    continue;
                }
            };
            if papers.insert(paper_id, paper.clone()).is_some() {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::ConflictingIdentity,
                    LITERATURE_MAP_PATH,
                    Some(&row_anchor),
                    Some(citekey),
                ));
                continue;
            }
            if evidence_limit.is_empty() || source_anchor.is_empty() {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::AmbiguousRelation,
                    LITERATURE_MAP_PATH,
                    Some(&row_anchor),
                    Some(citekey),
                ));
                continue;
            }
            let mut assignments = match reference_list(secondary) {
                Ok(assignments) => assignments,
                Err(()) => {
                    projection.diagnostics.push(diagnostic(
                        AcademicGraphDiagnosticCode::AmbiguousRelation,
                        LITERATURE_MAP_PATH,
                        Some(&row_anchor),
                        Some(citekey),
                    ));
                    Vec::new()
                }
            };
            if !primary.is_empty() {
                assignments.push(primary.to_string());
            }
            assignments.sort();
            assignments.dedup();
            for cluster_id in assignments {
                let Some(cluster) = clusters.get(&cluster_id) else {
                    projection.diagnostics.push(diagnostic(
                        AcademicGraphDiagnosticCode::DanglingNode,
                        LITERATURE_MAP_PATH,
                        Some(&row_anchor),
                        safe_related_id(&cluster_id),
                    ));
                    continue;
                };
                let edge = AcademicGraphEdgeV1::new(
                    project_id,
                    &paper.node_id,
                    AcademicGraphRelation::BelongsToCluster,
                    &cluster.node_id,
                    vec![AcademicGraphLayer::Literature, AcademicGraphLayer::Combined],
                    "The literature map assigns this included study to the named concept stream.",
                    LITERATURE_MAP_PATH,
                    format!("assignment:{citekey}:{cluster_id}"),
                    "The map records an evidence limit and source anchor; the underlying paper remains authoritative.",
                    AcademicInferenceStrength::ReasonableInference,
                    AcademicGraphConfidence::Unknown,
                    AcademicGraphEdgeStatus::Reviewed,
                    None,
                );
                match edge {
                    Ok(edge) => {
                        edges.insert(edge.edge_id.clone(), edge);
                    }
                    Err(_) => projection.diagnostics.push(diagnostic(
                        AcademicGraphDiagnosticCode::UnsupportedRelation,
                        LITERATURE_MAP_PATH,
                        Some(&row_anchor),
                        Some(citekey),
                    )),
                }
            }
        }
    }

    if let Ok(gap_table) = find_markdown_table(
        text,
        &[
            "gap_id",
            "open_problem",
            "cluster_ids",
            "source_anchors",
            "project_relevance",
            "status",
        ],
    ) {
        for row in &gap_table.rows {
            let gap_id = table_cell(&gap_table, row, "gap_id");
            let label = table_cell(&gap_table, row, "open_problem");
            let cluster_ids = table_cell(&gap_table, row, "cluster_ids");
            let source_anchors = table_cell(&gap_table, row, "source_anchors");
            let status = table_cell(&gap_table, row, "status").to_ascii_lowercase();
            if gap_id.is_empty() && label.is_empty() {
                continue;
            }
            let row_anchor = format!("row:{}", row.line_number);
            if !valid_reference_id(gap_id) || label.is_empty() {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::MissingStableId,
                    LITERATURE_MAP_PATH,
                    Some(&row_anchor),
                    safe_related_id(gap_id),
                ));
                continue;
            }
            if !matches!(
                status.as_str(),
                "open" | "unsupported" | "resolved" | "rejected"
            ) {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::AmbiguousRelation,
                    LITERATURE_MAP_PATH,
                    Some(&row_anchor),
                    Some(gap_id),
                ));
            }
            let gap = match AcademicGraphNodeV1::new(
                project_id,
                AcademicGraphNodeType::Gap,
                AcademicGraphIdentityScope::Project,
                gap_id,
                label,
                vec![AcademicGraphLayer::Literature, AcademicGraphLayer::Combined],
                LITERATURE_MAP_PATH,
                format!("gap:{gap_id}"),
            ) {
                Ok(node) => node,
                Err(_) => {
                    projection.diagnostics.push(diagnostic(
                        AcademicGraphDiagnosticCode::UnsupportedRelation,
                        LITERATURE_MAP_PATH,
                        Some(&row_anchor),
                        Some(gap_id),
                    ));
                    continue;
                }
            };
            if gaps.insert(gap_id.to_string(), gap.clone()).is_some() {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::ConflictingIdentity,
                    LITERATURE_MAP_PATH,
                    Some(&row_anchor),
                    Some(gap_id),
                ));
                continue;
            }
            if source_anchors.is_empty() {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::UnsupportedRelation,
                    LITERATURE_MAP_PATH,
                    Some(&row_anchor),
                    Some(gap_id),
                ));
                continue;
            }
            let assignments = match reference_list(cluster_ids) {
                Ok(assignments) => assignments,
                Err(()) => {
                    projection.diagnostics.push(diagnostic(
                        AcademicGraphDiagnosticCode::AmbiguousRelation,
                        LITERATURE_MAP_PATH,
                        Some(&row_anchor),
                        Some(gap_id),
                    ));
                    continue;
                }
            };
            for cluster_id in assignments {
                let Some(cluster) = clusters.get(&cluster_id) else {
                    projection.diagnostics.push(diagnostic(
                        AcademicGraphDiagnosticCode::DanglingNode,
                        LITERATURE_MAP_PATH,
                        Some(&row_anchor),
                        safe_related_id(&cluster_id),
                    ));
                    continue;
                };
                let edge = AcademicGraphEdgeV1::new(
                    project_id,
                    &gap.node_id,
                    AcademicGraphRelation::DerivedFrom,
                    &cluster.node_id,
                    vec![AcademicGraphLayer::Literature, AcademicGraphLayer::Combined],
                    "The literature map derives this open problem from the named concept stream.",
                    LITERATURE_MAP_PATH,
                    format!("gap-cluster:{gap_id}:{cluster_id}"),
                    "The recorded source anchors require inspection before the gap is treated as supported.",
                    AcademicInferenceStrength::UnsupportedGap,
                    AcademicGraphConfidence::Unknown,
                    if matches!(status.as_str(), "resolved" | "rejected") {
                        AcademicGraphEdgeStatus::Rejected
                    } else {
                        AcademicGraphEdgeStatus::Proposed
                    },
                    None,
                );
                if let Ok(edge) = edge {
                    edges.insert(edge.edge_id.clone(), edge);
                }
            }
        }
    }

    if let Ok(relation_table) = find_markdown_table(
        text,
        &[
            "source_cluster_id",
            "relation",
            "target_cluster_id",
            "source_anchor",
            "evidence_limit",
            "status",
        ],
    ) {
        for row in &relation_table.rows {
            extract_cluster_relation(
                project_id,
                &mut projection,
                &clusters,
                &mut edges,
                &relation_table,
                row,
            );
        }
    }
    projection.nodes.extend(clusters.into_values());
    projection.nodes.extend(papers.into_values());
    projection.nodes.extend(gaps.into_values());
    projection.edges.extend(edges.into_values());
    projection
}

fn extract_cluster_relation(
    project_id: &ProjectId,
    projection: &mut ExtractedAcademicGraph,
    clusters: &BTreeMap<String, AcademicGraphNodeV1>,
    edges: &mut BTreeMap<String, AcademicGraphEdgeV1>,
    table: &MarkdownTable,
    row: &MarkdownRow,
) {
    let source_id = table_cell(table, row, "source_cluster_id");
    let target_id = table_cell(table, row, "target_cluster_id");
    let relation_value = table_cell(table, row, "relation").to_ascii_lowercase();
    let source_evidence = table_cell(table, row, "source_anchor");
    let evidence_limit = table_cell(table, row, "evidence_limit");
    let status_value = table_cell(table, row, "status").to_ascii_lowercase();
    if source_id.is_empty() && target_id.is_empty() {
        return;
    }
    let row_anchor = format!("row:{}", row.line_number);
    let Some(source) = clusters.get(source_id) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::DanglingNode,
            LITERATURE_MAP_PATH,
            Some(&row_anchor),
            safe_related_id(source_id),
        ));
        return;
    };
    let Some(target) = clusters.get(target_id) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::DanglingNode,
            LITERATURE_MAP_PATH,
            Some(&row_anchor),
            safe_related_id(target_id),
        ));
        return;
    };
    let relation = match relation_value.as_str() {
        "complementary" | "complements" => AcademicGraphRelation::Complements,
        "competing" | "competes_with" => AcademicGraphRelation::CompetesWith,
        "under_integrated" | "combines_with" => AcademicGraphRelation::CombinesWith,
        "nested" | "extends" => AcademicGraphRelation::Extends,
        _ => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                LITERATURE_MAP_PATH,
                Some(&row_anchor),
                None,
            ));
            return;
        }
    };
    let status = match status_value.as_str() {
        "reviewed" => AcademicGraphEdgeStatus::Reviewed,
        "proposed" | "open" => AcademicGraphEdgeStatus::Proposed,
        "rejected" => AcademicGraphEdgeStatus::Rejected,
        _ => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                LITERATURE_MAP_PATH,
                Some(&row_anchor),
                None,
            ));
            return;
        }
    };
    if source_evidence.is_empty() || evidence_limit.is_empty() {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            LITERATURE_MAP_PATH,
            Some(&row_anchor),
            None,
        ));
        return;
    }
    let edge = AcademicGraphEdgeV1::new(
        project_id,
        &source.node_id,
        relation,
        &target.node_id,
        vec![AcademicGraphLayer::Literature, AcademicGraphLayer::Combined],
        "The literature map records this reviewed relationship between concept streams.",
        LITERATURE_MAP_PATH,
        format!("cluster-relation:{source_id}:{target_id}"),
        "The recorded source anchor and evidence limit remain authoritative in the literature map.",
        AcademicInferenceStrength::ReasonableInference,
        AcademicGraphConfidence::Unknown,
        status,
        None,
    );
    match edge {
        Ok(edge) => {
            edges.insert(edge.edge_id.clone(), edge);
        }
        Err(_) => projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            LITERATURE_MAP_PATH,
            Some(&row_anchor),
            None,
        )),
    }
}

fn extract_manuscript_claim_map(project_id: &ProjectId, bytes: &[u8]) -> ExtractedAcademicGraph {
    let mut projection = ExtractedAcademicGraph::empty();
    let Ok(text) = std::str::from_utf8(bytes) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            MANUSCRIPT_CLAIM_MAP_PATH,
            Some("document"),
            None,
        ));
        return projection;
    };
    let table = match find_markdown_table(text, &["claim_id", "claim", "claim_type"]) {
        Ok(table) => table,
        Err(anchor) => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                MANUSCRIPT_CLAIM_MAP_PATH,
                Some(anchor),
                None,
            ));
            return projection;
        }
    };
    let mut claims = BTreeMap::new();
    let mut papers = BTreeMap::new();
    let mut edges = BTreeMap::new();
    for row in &table.rows {
        let claim_id = table_cell(&table, row, "claim_id");
        let claim_text = table_cell(&table, row, "claim");
        let claim_type = table_cell(&table, row, "claim_type").to_ascii_lowercase();
        let citations = table_cell_any(&table, row, &["citation_keys", "citations"]);
        let evidence = table_cell_any(&table, row, &["evidence_pointer", "evidence"]);
        let confidence = table_cell(&table, row, "confidence").to_ascii_lowercase();
        let action = table_cell(&table, row, "action").to_ascii_lowercase();
        if claim_id.is_empty() && claim_text.is_empty() {
            continue;
        }
        let row_anchor = format!("row:{}", row.line_number);
        if !valid_reference_id(claim_id) || claim_text.is_empty() {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::MissingStableId,
                MANUSCRIPT_CLAIM_MAP_PATH,
                Some(&row_anchor),
                safe_related_id(claim_id),
            ));
            continue;
        }
        if !matches!(
            claim_type.as_str(),
            "background"
                | "method"
                | "result"
                | "finding"
                | "interpretation"
                | "implication"
                | "novelty"
                | "mechanism"
                | "empirical_effect"
                | "robustness"
                | "synthesis"
                | "limitation"
                | "method_assumption"
                | "speculation"
        ) {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                MANUSCRIPT_CLAIM_MAP_PATH,
                Some(&row_anchor),
                Some(claim_id),
            ));
        }
        let claim = match AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Claim,
            AcademicGraphIdentityScope::Project,
            claim_id,
            claim_text,
            vec![
                AcademicGraphLayer::Argument,
                AcademicGraphLayer::Manuscript,
                AcademicGraphLayer::Combined,
            ],
            MANUSCRIPT_CLAIM_MAP_PATH,
            format!("claim:{claim_id}"),
        ) {
            Ok(node) => node,
            Err(_) => {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::UnsupportedRelation,
                    MANUSCRIPT_CLAIM_MAP_PATH,
                    Some(&row_anchor),
                    Some(claim_id),
                ));
                continue;
            }
        };
        if claims.insert(claim_id.to_string(), claim.clone()).is_some() {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::ConflictingIdentity,
                MANUSCRIPT_CLAIM_MAP_PATH,
                Some(&row_anchor),
                Some(claim_id),
            ));
            continue;
        }
        if citations.is_empty() && evidence.is_empty() {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                MANUSCRIPT_CLAIM_MAP_PATH,
                Some(&row_anchor),
                Some(claim_id),
            ));
            continue;
        }
        let citekeys = match citation_list(citations) {
            Ok(citekeys) => citekeys,
            Err(()) => {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::AmbiguousRelation,
                    MANUSCRIPT_CLAIM_MAP_PATH,
                    Some(&row_anchor),
                    Some(claim_id),
                ));
                continue;
            }
        };
        let (edge_confidence, confidence_known) = graph_confidence(&confidence);
        if !confidence_known {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                MANUSCRIPT_CLAIM_MAP_PATH,
                Some(&row_anchor),
                Some(claim_id),
            ));
        }
        let edge_status = match action.as_str() {
            "keep" | "ok" => AcademicGraphEdgeStatus::Reviewed,
            "hedge" | "revise" | "" => AcademicGraphEdgeStatus::Proposed,
            "remove" | "reject" => AcademicGraphEdgeStatus::Rejected,
            _ => {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::AmbiguousRelation,
                    MANUSCRIPT_CLAIM_MAP_PATH,
                    Some(&row_anchor),
                    Some(claim_id),
                ));
                AcademicGraphEdgeStatus::Proposed
            }
        };
        for citekey in citekeys {
            let paper_id = format!("citekey:{citekey}");
            let paper = match AcademicGraphNodeV1::new(
                project_id,
                AcademicGraphNodeType::Paper,
                AcademicGraphIdentityScope::Global,
                &paper_id,
                &citekey,
                vec![
                    AcademicGraphLayer::Literature,
                    AcademicGraphLayer::Manuscript,
                    AcademicGraphLayer::Combined,
                ],
                MANUSCRIPT_CLAIM_MAP_PATH,
                format!("paper:{citekey}"),
            ) {
                Ok(node) => node,
                Err(_) => continue,
            };
            papers.entry(paper_id).or_insert_with(|| paper.clone());
            let edge = AcademicGraphEdgeV1::new(
                project_id,
                &claim.node_id,
                AcademicGraphRelation::Cites,
                &paper.node_id,
                vec![
                    AcademicGraphLayer::Argument,
                    AcademicGraphLayer::Manuscript,
                    AcademicGraphLayer::Combined,
                ],
                "The manuscript claim map records this citation for the claim.",
                MANUSCRIPT_CLAIM_MAP_PATH,
                format!("claim-citation:{claim_id}:{citekey}"),
                "Citation presence is observed; evidential sufficiency remains governed by the claim-evidence ledger.",
                AcademicInferenceStrength::ReasonableInference,
                edge_confidence,
                edge_status,
                None,
            );
            if let Ok(edge) = edge {
                edges.insert(edge.edge_id.clone(), edge);
            }
        }
    }
    projection.nodes.extend(claims.into_values());
    projection.nodes.extend(papers.into_values());
    projection.edges.extend(edges.into_values());
    projection
}

fn extract_evidence_ledger(project_id: &ProjectId, bytes: &[u8]) -> ExtractedAcademicGraph {
    let mut projection = ExtractedAcademicGraph::empty();
    let Ok(text) = std::str::from_utf8(bytes) else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            EVIDENCE_LEDGER_PATH,
            Some("document"),
            None,
        ));
        return projection;
    };
    let records = match parse_csv(text) {
        Ok(records) => records,
        Err(()) => {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                EVIDENCE_LEDGER_PATH,
                Some("csv"),
                None,
            ));
            return projection;
        }
    };
    let Some(header) = records.first() else {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            EVIDENCE_LEDGER_PATH,
            Some("header"),
            None,
        ));
        return projection;
    };
    let normalized_header = header
        .fields
        .iter()
        .map(|field| normalize_header(field))
        .collect::<Vec<_>>();
    if normalized_header.as_slice() != EVIDENCE_COLUMNS {
        projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            EVIDENCE_LEDGER_PATH,
            Some("header"),
            None,
        ));
        return projection;
    }

    let mut claims = BTreeMap::new();
    let mut evidence_nodes = BTreeMap::new();
    let mut support_edges = BTreeMap::new();
    let mut support_records = BTreeMap::new();
    for record in records.into_iter().skip(1) {
        let anchor = format!("row:{}", record.line_number);
        if record.fields.len() != EVIDENCE_COLUMNS.len() {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                None,
            ));
            continue;
        }
        let fields = record
            .fields
            .iter()
            .map(|field| field.trim())
            .collect::<Vec<_>>();
        let claim_id = fields[0];
        let claim_text = fields[1];
        let claim_type = fields[2].to_ascii_lowercase();
        let evidence_type = fields[3].to_ascii_lowercase();
        let source_id = fields[4];
        let source_location = fields[5];
        let referenced_artifact = fields[6];
        let confidence = fields[7].to_ascii_lowercase();
        let status = fields[9].to_ascii_lowercase();

        if !valid_reference_id(claim_id) || claim_text.is_empty() {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::MissingStableId,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                safe_related_id(claim_id),
            ));
            continue;
        }
        if claims.get(claim_id).is_some_and(
            |(existing, existing_type): &(AcademicGraphNodeV1, String)| {
                existing.label != claim_text || existing_type != &claim_type
            },
        ) {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::ConflictingIdentity,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                Some(claim_id),
            ));
            continue;
        }
        let claim = match AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Claim,
            AcademicGraphIdentityScope::Project,
            claim_id,
            claim_text,
            vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
            EVIDENCE_LEDGER_PATH,
            format!("claim:{claim_id}"),
        ) {
            Ok(node) => node,
            Err(_) => {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::UnsupportedRelation,
                    EVIDENCE_LEDGER_PATH,
                    Some(&anchor),
                    Some(claim_id),
                ));
                continue;
            }
        };
        claims
            .entry(claim_id.to_string())
            .or_insert_with(|| (claim.clone(), claim_type.clone()));

        let valid_contract_types = CLAIM_TYPES.contains(&claim_type.as_str())
            && EVIDENCE_TYPES.contains(&evidence_type.as_str());
        if !valid_contract_types {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                Some(claim_id),
            ));
            continue;
        }
        if matches!(status.as_str(), "unsupported" | "needs_evidence")
            || evidence_type == "gap_note"
            || source_id.is_empty()
            || source_location.is_empty()
        {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::UnsupportedRelation,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                Some(claim_id),
            ));
            continue;
        }
        if !matches!(
            status.as_str(),
            "supported" | "verified" | "complete" | "ready" | "ok"
        ) {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                Some(claim_id),
            ));
            continue;
        }
        if !valid_source_id(source_id) || !valid_portable_path(referenced_artifact) {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::DanglingNode,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                Some(claim_id),
            ));
            continue;
        }
        let (graph_confidence, confidence_known) = graph_confidence(&confidence);
        if !confidence_known {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::AmbiguousRelation,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                Some(claim_id),
            ));
        }
        let evidence = match AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Evidence,
            AcademicGraphIdentityScope::Project,
            format!("evidence-source:{source_id}"),
            source_id,
            vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
            EVIDENCE_LEDGER_PATH,
            format!("source:{source_id}"),
        ) {
            Ok(node) => node,
            Err(_) => {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::DanglingNode,
                    EVIDENCE_LEDGER_PATH,
                    Some(&anchor),
                    Some(claim_id),
                ));
                continue;
            }
        };
        evidence_nodes
            .entry(evidence.node_id.clone())
            .or_insert_with(|| evidence.clone());
        let Ok(support_anchor) = evidence_support_anchor(&record.fields) else {
            continue;
        };
        let normalized_record = fields
            .iter()
            .map(|field| (*field).to_string())
            .collect::<Vec<_>>();
        if support_records
            .get(&support_anchor)
            .is_some_and(|existing| existing != &normalized_record)
        {
            projection.diagnostics.push(diagnostic(
                AcademicGraphDiagnosticCode::ConflictingIdentity,
                EVIDENCE_LEDGER_PATH,
                Some(&anchor),
                Some(claim_id),
            ));
            continue;
        }
        support_records
            .entry(support_anchor.clone())
            .or_insert(normalized_record);
        let edge = match AcademicGraphEdgeV1::new(
            project_id,
            &evidence.node_id,
            AcademicGraphRelation::Supports,
            &claim.node_id,
            vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
            "The canonical claim-evidence ledger records this source as supporting evidence.",
            EVIDENCE_LEDGER_PATH,
            &support_anchor,
            "Evidence limitations remain authoritative in the claim-evidence ledger.",
            AcademicInferenceStrength::DirectEvidence,
            graph_confidence,
            AcademicGraphEdgeStatus::Reviewed,
            None,
        ) {
            Ok(edge) => edge,
            Err(_) => {
                projection.diagnostics.push(diagnostic(
                    AcademicGraphDiagnosticCode::UnsupportedRelation,
                    EVIDENCE_LEDGER_PATH,
                    Some(&anchor),
                    Some(claim_id),
                ));
                continue;
            }
        };
        support_edges.entry(edge.edge_id.clone()).or_insert(edge);

        // Only an explicit paper citekey joins the literature/manuscript identity.
        // Other source IDs remain evidence; titles and DOI aliases are not guessed.
        let citekey = clean_citekey(source_id);
        if evidence_type == "paper" && valid_reference_id(citekey) {
            let paper = AcademicGraphNodeV1::new(
                project_id,
                AcademicGraphNodeType::Paper,
                AcademicGraphIdentityScope::Global,
                format!("citekey:{citekey}"),
                citekey,
                vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
                EVIDENCE_LEDGER_PATH,
                format!("source:{source_id}"),
            );
            if let Ok(paper) = paper {
                let origin = AcademicGraphEdgeV1::new(
                    project_id,
                    &evidence.node_id,
                    AcademicGraphRelation::DerivedFrom,
                    &paper.node_id,
                    vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Combined],
                    "The ledger identifies this paper as the evidence source.",
                    EVIDENCE_LEDGER_PATH,
                    &support_anchor,
                    "Source identity only; the ledger's source location and limitations remain authoritative.",
                    AcademicInferenceStrength::DirectEvidence,
                    graph_confidence,
                    AcademicGraphEdgeStatus::Reviewed,
                    None,
                );
                if let Ok(origin) = origin {
                    evidence_nodes.entry(paper.node_id.clone()).or_insert(paper);
                    support_edges
                        .entry(origin.edge_id.clone())
                        .or_insert(origin);
                }
            }
        }
    }

    projection
        .nodes
        .extend(claims.into_values().map(|(node, _)| node));
    projection.nodes.extend(evidence_nodes.into_values());
    projection.edges.extend(support_edges.into_values());
    projection
}

fn evidence_support_anchor(fields: &[String]) -> Result<String, ProjectError> {
    let digest = crate::academic_graph::canonical_domain_digest(
        b"qiongli-evidence-support-anchor-v1",
        &[
            fields[0].trim(),
            fields[4].trim(),
            fields[5].trim(),
            fields[6].trim(),
        ],
    )?;
    Ok(format!("support:{digest}"))
}

pub(crate) fn evidence_ledger_anchor_line(text: &str, anchor: &str) -> Option<usize> {
    let records = parse_csv(text).ok()?;
    let (header, rows) = records.split_first()?;
    if header
        .fields
        .iter()
        .map(|field| normalize_header(field))
        .collect::<Vec<_>>()
        != EVIDENCE_COLUMNS
    {
        return None;
    }
    rows.iter()
        .filter(|row| row.fields.len() == EVIDENCE_COLUMNS.len())
        .find(|row| {
            anchor == format!("claim:{}", row.fields[0].trim())
                || anchor == format!("source:{}", row.fields[4].trim())
                || evidence_support_anchor(&row.fields).ok().as_deref() == Some(anchor)
        })
        .map(|row| row.line_number)
}

#[allow(clippy::too_many_arguments)]
fn push_node(
    projection: &mut ExtractedAcademicGraph,
    project_id: &ProjectId,
    node_type: AcademicGraphNodeType,
    canonical_id: impl Into<String>,
    label: impl Into<String>,
    layers: Vec<AcademicGraphLayer>,
    artifact_path: &str,
    source_anchor: impl Into<String>,
) {
    let canonical_id = canonical_id.into();
    let source_anchor = source_anchor.into();
    match AcademicGraphNodeV1::new(
        project_id,
        node_type,
        AcademicGraphIdentityScope::Project,
        canonical_id.clone(),
        label,
        layers,
        artifact_path,
        source_anchor.clone(),
    ) {
        Ok(node) => projection.nodes.push(node),
        Err(_) => projection.diagnostics.push(diagnostic(
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            artifact_path,
            Some(&source_anchor),
            safe_related_id(&canonical_id),
        )),
    }
}

fn diagnostic(
    code: AcademicGraphDiagnosticCode,
    artifact_path: &str,
    source_anchor: Option<&str>,
    related_id: Option<&str>,
) -> AcademicGraphDiagnosticV1 {
    AcademicGraphDiagnosticV1 {
        code,
        artifact_path: artifact_path.to_string(),
        source_anchor: source_anchor.map(str::to_string),
        related_id: related_id.map(str::to_string),
    }
}

fn safe_related_id(value: &str) -> Option<&str> {
    valid_reference_id(value).then_some(value)
}

fn valid_reference_id(value: &str) -> bool {
    valid_text(value, MAX_RELATED_ID_BYTES)
        && value.nfc().eq(value.chars())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn valid_source_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    valid_text(value, MAX_RELATED_ID_BYTES)
        && value.nfc().eq(value.chars())
        && !value.starts_with(['/', '\\', '~'])
        && !value.contains(['\\', '\0'])
        && !(bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && bytes[2] == b'/')
        && !value.contains("..")
        && !value.to_ascii_lowercase().starts_with("file:")
        && !value.contains("://")
}

fn valid_portable_path(value: &str) -> bool {
    if !valid_text(value, MAX_RELATED_ID_BYTES)
        || value.starts_with(['/', '\\', '~'])
        || value.contains(['\\', ':'])
        || value.ends_with('/')
    {
        return false;
    }
    Path::new(value)
        .components()
        .all(|component| matches!(component, Component::Normal(part) if !part.is_empty()))
}

fn valid_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_decision_status(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    matches!(normalized.as_str(), "locked" | "tentative" | "blocked")
        || normalized
            .strip_prefix("revisit-after-")
            .is_some_and(|stage| {
                !stage.is_empty() && stage.bytes().all(|byte| byte.is_ascii_alphanumeric())
            })
}

fn graph_confidence(value: &str) -> (AcademicGraphConfidence, bool) {
    match value {
        "high" => (AcademicGraphConfidence::High, true),
        "medium" => (AcademicGraphConfidence::Medium, true),
        "low" => (AcademicGraphConfidence::Low, true),
        "unknown" | "" => (AcademicGraphConfidence::Unknown, true),
        _ => (AcademicGraphConfidence::Unknown, false),
    }
}

fn normalize_header(value: &str) -> String {
    let normalized = value
        .trim()
        .trim_matches('`')
        .to_ascii_lowercase()
        .chars()
        .map(|character| match character {
            character if character.is_ascii_alphanumeric() || character == '_' => character,
            _ => '_',
        })
        .collect::<String>();
    let mut collapsed = String::with_capacity(normalized.len());
    for character in normalized.chars() {
        if character != '_' || !collapsed.ends_with('_') {
            collapsed.push(character);
        }
    }
    let normalized = collapsed.trim_matches('_');
    match normalized {
        "claim_atomic" => "claim".to_string(),
        value if value.starts_with("claim_type_") => "claim_type".to_string(),
        value if value.starts_with("evidence_data_") => "evidence".to_string(),
        "citation_s" => "citations".to_string(),
        "location_in_manuscript" => "manuscript_location".to_string(),
        value => value.to_string(),
    }
}

struct MarkdownRow {
    line_number: usize,
    fields: Vec<String>,
}

struct MarkdownTable {
    header: Vec<String>,
    rows: Vec<MarkdownRow>,
}

fn find_markdown_table(
    text: &str,
    required_headers: &[&str],
) -> Result<MarkdownTable, &'static str> {
    let lines = text.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let Some(header_cells) = markdown_cells(line) else {
            continue;
        };
        let header = header_cells
            .iter()
            .map(|cell| normalize_header(cell))
            .collect::<Vec<_>>();
        if !required_headers
            .iter()
            .all(|required| header.iter().any(|cell| cell == required))
        {
            continue;
        }
        let mut rows = Vec::new();
        for (row_index, row_line) in lines.iter().enumerate().skip(index + 1) {
            let Some(fields) = markdown_cells(row_line) else {
                if !rows.is_empty() {
                    break;
                }
                continue;
            };
            if markdown_separator(&fields) {
                continue;
            }
            if fields.len() != header.len() || rows.len() >= MAX_TABLE_ROWS {
                return Err("table");
            }
            rows.push(MarkdownRow {
                line_number: row_index + 1,
                fields,
            });
        }
        return Ok(MarkdownTable { header, rows });
    }
    Err("header")
}

fn table_cell<'a>(table: &'a MarkdownTable, row: &'a MarkdownRow, key: &str) -> &'a str {
    table
        .header
        .iter()
        .position(|header| header == key)
        .and_then(|index| row.fields.get(index))
        .map_or("", |value| value.trim())
}

fn table_cell_any<'a>(table: &'a MarkdownTable, row: &'a MarkdownRow, keys: &[&str]) -> &'a str {
    keys.iter()
        .map(|key| table_cell(table, row, key))
        .find(|value| !value.is_empty())
        .unwrap_or("")
}

fn scalar_value(text: &str, requested_key: &str) -> Result<Option<String>, ()> {
    let mut value = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') {
            continue;
        }
        let field = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        let Some((raw_key, raw_value)) = field.split_once(':') else {
            continue;
        };
        if normalize_header(raw_key) != requested_key {
            continue;
        }
        let candidate = raw_value.trim();
        if candidate.is_empty() {
            continue;
        }
        match value.as_deref() {
            None => value = Some(candidate.to_string()),
            Some(current) if current == candidate => {}
            Some(_) => return Err(()),
        }
    }
    Ok(value)
}

fn reference_list(value: &str) -> Result<Vec<String>, ()> {
    parse_reference_list(value, false)
}

fn citation_list(value: &str) -> Result<Vec<String>, ()> {
    parse_reference_list(value, true)
}

fn parse_reference_list(value: &str, citations: bool) -> Result<Vec<String>, ()> {
    let trimmed = value.trim();
    if trimmed.is_empty() || matches!(trimmed.to_ascii_lowercase().as_str(), "none" | "n/a" | "-") {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    for item in trimmed.split(|character: char| {
        character == ',' || character == ';' || character.is_ascii_whitespace()
    }) {
        let item = if citations {
            clean_citekey(item)
        } else {
            item.trim().trim_matches(['[', ']', '`'])
        };
        if item.is_empty() {
            continue;
        }
        if !valid_reference_id(item) || values.len() >= MAX_LIST_ITEMS {
            return Err(());
        }
        values.push(item.to_string());
    }
    values.sort();
    values.dedup();
    Ok(values)
}

fn clean_citekey(value: &str) -> &str {
    value
        .trim()
        .trim_matches(['[', ']', '`'])
        .trim_start_matches('@')
}

fn markdown_cells(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.contains('|') {
        return None;
    }
    let body = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed.strip_prefix('|').unwrap_or(trimmed));
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in body.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '|' {
            if cells.len() >= MAX_TABLE_COLUMNS || current.len() > MAX_FIELD_BYTES {
                return None;
            }
            cells.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(character);
        }
    }
    if escaped {
        current.push('\\');
    }
    if cells.len() >= MAX_TABLE_COLUMNS || current.len() > MAX_FIELD_BYTES {
        return None;
    }
    cells.push(current.trim().to_string());
    Some(cells)
}

fn markdown_separator(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            !trimmed.is_empty()
                && trimmed
                    .bytes()
                    .all(|byte| matches!(byte, b'-' | b':' | b' '))
        })
}

struct CsvRecord {
    line_number: usize,
    fields: Vec<String>,
}

fn parse_csv(text: &str) -> Result<Vec<CsvRecord>, ()> {
    let bytes = text.as_bytes();
    let mut records = Vec::new();
    let mut fields = Vec::new();
    let mut field = Vec::new();
    let mut index = 0usize;
    let mut line_number = 1usize;
    let mut record_line = 1usize;
    let mut in_quotes = false;
    let mut after_quote = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_quotes {
            if byte == b'"' {
                if bytes.get(index + 1) == Some(&b'"') {
                    field.push(b'"');
                    index += 1;
                } else {
                    in_quotes = false;
                    after_quote = true;
                }
            } else {
                if byte == b'\n' {
                    line_number += 1;
                }
                field.push(byte);
            }
        } else if after_quote {
            match byte {
                b',' => {
                    push_csv_field(&mut fields, &mut field)?;
                    after_quote = false;
                }
                b'\n' => {
                    push_csv_field(&mut fields, &mut field)?;
                    push_csv_record(&mut records, &mut fields, record_line)?;
                    line_number += 1;
                    record_line = line_number;
                    after_quote = false;
                }
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {}
                _ => return Err(()),
            }
        } else {
            match byte {
                b'"' if field.is_empty() => in_quotes = true,
                b'"' => return Err(()),
                b',' => push_csv_field(&mut fields, &mut field)?,
                b'\n' => {
                    push_csv_field(&mut fields, &mut field)?;
                    push_csv_record(&mut records, &mut fields, record_line)?;
                    line_number += 1;
                    record_line = line_number;
                }
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {}
                _ => field.push(byte),
            }
        }
        if field.len() > MAX_FIELD_BYTES {
            return Err(());
        }
        index += 1;
    }
    if in_quotes {
        return Err(());
    }
    if after_quote || !field.is_empty() || !fields.is_empty() {
        push_csv_field(&mut fields, &mut field)?;
        push_csv_record(&mut records, &mut fields, record_line)?;
    }
    Ok(records)
}

fn push_csv_field(fields: &mut Vec<String>, field: &mut Vec<u8>) -> Result<(), ()> {
    if fields.len() >= MAX_TABLE_COLUMNS || field.len() > MAX_FIELD_BYTES {
        return Err(());
    }
    let bytes = std::mem::take(field);
    fields.push(String::from_utf8(bytes).map_err(|_| ())?);
    Ok(())
}

fn push_csv_record(
    records: &mut Vec<CsvRecord>,
    fields: &mut Vec<String>,
    line_number: usize,
) -> Result<(), ()> {
    if records.len() > MAX_TABLE_ROWS {
        return Err(());
    }
    if fields.len() == 1 && fields[0].is_empty() {
        fields.clear();
        return Ok(());
    }
    records.push(CsvRecord {
        line_number,
        fields: std::mem::take(fields),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_preserves_multiple_supports_and_resolves_stable_record_anchors() {
        let project_id = ProjectId::parse("prj_829c159ea9a35837376cb3533a4ab1c9").unwrap();
        let header = EVIDENCE_COLUMNS.join(",");
        let first = "C1,An association,finding,paper,Smith2024,p. 4,notes/smith.md,high,One setting,supported";
        let second = "C1,An association,finding,paper,Jones2025,Table 2,notes/jones.md,medium,Observational,supported";
        let third = "C1,An association,finding,paper,Smith2024,p. 8,notes/smith.md,low,Robustness only,supported";
        let text = format!("{header}\n{first}\n{second}\n{third}\n{first}\n");
        let graph = extract_evidence_ledger(&project_id, text.as_bytes());
        assert!(graph.diagnostics.is_empty());
        assert_eq!(
            graph
                .nodes
                .iter()
                .filter(|node| node.node_type == AcademicGraphNodeType::Claim)
                .count(),
            1
        );
        assert_eq!(
            graph
                .nodes
                .iter()
                .filter(|node| node.node_type == AcademicGraphNodeType::Paper)
                .count(),
            2
        );
        let supports = graph
            .edges
            .iter()
            .filter(|edge| edge.relation == AcademicGraphRelation::Supports)
            .collect::<Vec<_>>();
        assert_eq!(supports.len(), 3);
        let lines = supports
            .iter()
            .map(|edge| evidence_ledger_anchor_line(&text, &edge.source_anchor).unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(lines, [2, 3, 4].into());
        assert_eq!(evidence_ledger_anchor_line(&text, "claim:C1"), Some(2));
        assert_eq!(
            evidence_ledger_anchor_line(&text, "source:Jones2025"),
            Some(3)
        );
        assert_eq!(evidence_ledger_anchor_line(&text, "support:missing"), None);
        let reordered = format!("{header}\n{third}\n{second}\n{first}\n");
        let rebuilt = extract_evidence_ledger(&project_id, reordered.as_bytes());
        assert_eq!(graph.nodes, rebuilt.nodes);
        assert_eq!(graph.edges, rebuilt.edges);

        for invalid in [
            second.replace("An association", "A causal effect"),
            second.replace(",finding,", ",interpretation,"),
            first.replace(",high,", ",low,"),
            first.replace("One setting", "A different limitation"),
        ] {
            let text = format!("{header}\n{first}\n{invalid}\n");
            let graph = extract_evidence_ledger(&project_id, text.as_bytes());
            assert!(
                graph.diagnostics.iter().any(|diagnostic| diagnostic.code
                    == AcademicGraphDiagnosticCode::ConflictingIdentity)
            );
            assert_eq!(
                graph
                    .edges
                    .iter()
                    .filter(|edge| edge.relation == AcademicGraphRelation::Supports)
                    .count(),
                1
            );
        }
        for invalid in [
            first.replace("p. 4", ""),
            first.replace(",supported", ",needs_evidence"),
            first.replace("notes/smith.md", "../private.md"),
        ] {
            let graph =
                extract_evidence_ledger(&project_id, format!("{header}\n{invalid}\n").as_bytes());
            assert!(!graph.diagnostics.is_empty());
            assert!(graph.edges.is_empty());
        }
    }

    #[test]
    fn csv_parser_accepts_quotes_commas_crlf_and_embedded_newlines() {
        let records = parse_csv("a,b,c\r\n1,\"two,too\",\"three\nlines\"\r\n").unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[1].line_number, 2);
        assert_eq!(records[1].fields, ["1", "two,too", "three\nlines"]);
    }

    #[test]
    fn csv_parser_rejects_unclosed_or_trailing_quote_content() {
        assert!(parse_csv("a,b\n1,\"two\n").is_err());
        assert!(parse_csv("a,b\n1,\"two\"x\n").is_err());
        assert!(parse_csv("a,b\n1,t\"wo\n").is_err());
    }

    #[test]
    fn markdown_parser_accepts_escaped_pipes() {
        assert_eq!(
            markdown_cells("| DEC-1 | Keep A \\| B |"),
            Some(vec!["DEC-1".to_string(), "Keep A | B".to_string()])
        );
    }

    #[test]
    fn structural_handoff_and_unregistered_prose_never_manufacture_graph_facts() {
        let project_id = ProjectId::parse("prj_829c159ea9a35837376cb3533a4ab1c9").unwrap();
        let handoff = extract_academic_artifact(
            &project_id,
            "context/stage_handoff.md",
            b"The claim C-1 proves decision D-1.\n@article{invented, title={No inference}}\n",
        )
        .unwrap();
        assert!(handoff.nodes.is_empty());
        assert!(handoff.edges.is_empty());
        assert!(handoff.diagnostics.is_empty());
        assert_eq!(
            extract_academic_artifact(
                &project_id,
                "literature/references.bib",
                b"@article{invented, title={No inference}}",
            )
            .err(),
            Some(ProjectError::ProjectArtifactUnsupported)
        );
    }
}
