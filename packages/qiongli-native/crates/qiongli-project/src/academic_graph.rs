use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Formatter};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::academic_graph_extract::extract_academic_artifact;
use crate::academic_graph_readiness::{
    ACADEMIC_GRAPH_PROJECTION_DOCUMENT_KIND, ACADEMIC_GRAPH_PROJECTION_SCHEMA_VERSION,
    AcademicGraphProjectionV1, AcademicGraphReadinessV1,
};
use crate::json::parse_unique_json;
use crate::model::{ProjectId, ProjectLifecycle, ProjectStage, valid_lower_hex};
use crate::storage::{
    GRAPH_SEMANTIC_LINKS_RELATIVE_PATH, SEMANTIC_ARTIFACTS, project_root_from_string,
    read_academic_graph_artifact, read_graph_semantic_links, read_manifest, read_semantic_artifact,
    resolve_academic_graph_artifact_path, semantic_digest,
};
use crate::{CaptureId, ProjectError, ProjectStateService};

pub const ACADEMIC_GRAPH_SCHEMA_VERSION: u32 = 1;
pub const ACADEMIC_GRAPH_DOCUMENT_KIND: &str = "qiongli-academic-graph";
pub const PROJECT_ARTIFACT_VIEW_SCHEMA_VERSION: u32 = 1;
pub const PROJECT_ARTIFACT_VIEW_DOCUMENT_KIND: &str = "qiongli-project-artifact-view";
pub const MIN_PROJECT_ARTIFACT_VIEW_BYTES: usize = 1_024;
pub const MAX_PROJECT_ARTIFACT_VIEW_BYTES: usize = 256 * 1_024;
const SEMANTIC_NODE_DOCUMENT_KIND: &str = "qiongli-academic-graph-node";
const SEMANTIC_EDGE_DOCUMENT_KIND: &str = "qiongli-academic-semantic-link";
const PROJECT_MANIFEST_RELATIVE_PATH: &str = "context/project_manifest.json";
const MAX_GRAPH_RECORDS: usize = 2_048;
pub(crate) const MAX_GRAPH_NODES: usize = 4_096;
pub(crate) const MAX_GRAPH_EDGES: usize = 4_096;
const MAX_GRAPH_DIAGNOSTICS: usize = 4_096;
const MAX_GRAPH_LINE_BYTES: usize = 32 * 1024;
const MAX_CANONICAL_ID_BYTES: usize = 512;
const MAX_LABEL_BYTES: usize = 1_024;
const MAX_PATH_BYTES: usize = 512;
const MAX_ANCHOR_BYTES: usize = 512;
const MAX_RATIONALE_BYTES: usize = 4 * 1024;
const MAX_EVIDENCE_LIMIT_BYTES: usize = 2 * 1024;
const MAX_GRAPH_SNAPSHOT_BYTES: usize = 4 * 1024 * 1024;
const MAX_GRAPH_SOURCE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphNodeType {
    Project,
    ResearchQuestion,
    Idea,
    Contribution,
    Concept,
    LiteratureCluster,
    Paper,
    Claim,
    Evidence,
    Decision,
    Gap,
    Method,
    ManuscriptSection,
    Artifact,
    Task,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphRelation {
    Contains,
    Cites,
    CitedBy,
    Supports,
    Weakens,
    Contradicts,
    Extends,
    Defines,
    Operationalizes,
    UsesMethod,
    BelongsToCluster,
    Complements,
    CompetesWith,
    CombinesWith,
    Motivates,
    Informs,
    AddressesGap,
    AppearsInSection,
    DerivedFrom,
    Supersedes,
    BoundedBy,
    SharesSource,
    SharesConcept,
    ForkedFrom,
    ExtendsProject,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphLayer {
    Portfolio,
    Literature,
    IdeaDecision,
    Argument,
    Manuscript,
    Combined,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcademicInferenceStrength {
    DirectEvidence,
    ReasonableInference,
    UnsupportedGap,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphConfidence {
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphEdgeStatus {
    Observed,
    Proposed,
    Reviewed,
    Rejected,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphIdentityScope {
    Project,
    Global,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphSourceKind {
    ProjectManifest,
    RegisteredArtifact,
    SemanticLinks,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphDiagnosticCode {
    MissingStableId,
    AmbiguousRelation,
    UnsupportedRelation,
    DanglingNode,
    ConflictingIdentity,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcademicGraphEntityKind {
    Node,
    Edge,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectArtifactFormat {
    Markdown,
    Csv,
    Json,
    JsonLines,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectArtifactViewV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub projection_id: Option<String>,
    pub entity_kind: Option<AcademicGraphEntityKind>,
    pub entity_id: Option<String>,
    pub artifact_path: String,
    pub source_anchor: Option<String>,
    pub format: ProjectArtifactFormat,
    pub content_digest: String,
    pub source_size_bytes: u64,
    pub content: String,
    pub content_size_bytes: u64,
    pub start_line: u64,
    pub end_line: u64,
    pub anchor_line: Option<u64>,
    pub anchor_matched: bool,
    pub truncated_before: bool,
    pub truncated_after: bool,
}

#[derive(Clone)]
pub struct AcademicGraphArtifactTarget {
    path: PathBuf,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub projection_id: String,
    pub entity_kind: AcademicGraphEntityKind,
    pub entity_id: String,
    pub artifact_path: String,
    pub source_anchor: String,
}

impl AcademicGraphArtifactTarget {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Debug for AcademicGraphArtifactTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AcademicGraphArtifactTarget")
            .field("path", &"<registered-academic-artifact>")
            .field("project_id", &self.project_id)
            .field("project_revision", &self.project_revision)
            .field("projection_id", &self.projection_id)
            .field("entity_kind", &self.entity_kind)
            .field("entity_id", &self.entity_id)
            .field("artifact_path", &self.artifact_path)
            .field("source_anchor", &self.source_anchor)
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcademicGraphSourceRefV1 {
    pub source_kind: AcademicGraphSourceKind,
    pub artifact_path: String,
    pub present: bool,
    pub content_digest: Option<String>,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcademicGraphNodeV1 {
    pub node_id: String,
    pub node_type: AcademicGraphNodeType,
    pub identity_scope: AcademicGraphIdentityScope,
    pub canonical_id: String,
    pub label: String,
    pub layers: Vec<AcademicGraphLayer>,
    pub artifact_path: String,
    pub source_anchor: String,
}

impl AcademicGraphNodeV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: &ProjectId,
        node_type: AcademicGraphNodeType,
        identity_scope: AcademicGraphIdentityScope,
        canonical_id: impl Into<String>,
        label: impl Into<String>,
        mut layers: Vec<AcademicGraphLayer>,
        artifact_path: impl Into<String>,
        source_anchor: impl Into<String>,
    ) -> Result<Self, ProjectError> {
        layers.sort_unstable();
        layers.dedup();
        let canonical_id = canonical_id.into();
        let node_id = node_id(project_id, node_type, identity_scope, &canonical_id)?;
        let node = Self {
            node_id,
            node_type,
            identity_scope,
            canonical_id,
            label: label.into(),
            layers,
            artifact_path: artifact_path.into(),
            source_anchor: source_anchor.into(),
        };
        node.validate(project_id)?;
        Ok(node)
    }

    pub(crate) fn validate(&self, project_id: &ProjectId) -> Result<(), ProjectError> {
        if !valid_graph_id(&self.node_id, "nod_")
            || !valid_canonical_id(&self.canonical_id)
            || !valid_text(&self.label, MAX_LABEL_BYTES)
            || !valid_layers(&self.layers)
            || !valid_artifact_path(&self.artifact_path)
            || !valid_anchor(&self.source_anchor)
            || (self.identity_scope == AcademicGraphIdentityScope::Global
                && !matches!(
                    self.node_type,
                    AcademicGraphNodeType::Project
                        | AcademicGraphNodeType::Paper
                        | AcademicGraphNodeType::Concept
                        | AcademicGraphNodeType::Method
                ))
            || (self.node_type == AcademicGraphNodeType::Project
                && match self.identity_scope {
                    AcademicGraphIdentityScope::Project => self.canonical_id != project_id.as_str(),
                    AcademicGraphIdentityScope::Global => {
                        self.canonical_id == project_id.as_str()
                            || ProjectId::parse(self.canonical_id.clone()).is_err()
                    }
                })
            || (self.node_type == AcademicGraphNodeType::Artifact
                && self.canonical_id != self.artifact_path)
            || self.node_id
                != node_id(
                    project_id,
                    self.node_type,
                    self.identity_scope,
                    &self.canonical_id,
                )?
        {
            return Err(ProjectError::InvalidGraphDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcademicGraphEdgeV1 {
    pub edge_id: String,
    pub source_node_id: String,
    pub relation: AcademicGraphRelation,
    pub target_node_id: String,
    pub layers: Vec<AcademicGraphLayer>,
    pub rationale: String,
    pub artifact_path: String,
    pub source_anchor: String,
    pub evidence_limit: String,
    pub inference_strength: AcademicInferenceStrength,
    pub confidence: AcademicGraphConfidence,
    pub status: AcademicGraphEdgeStatus,
    pub created_from_capture: Option<CaptureId>,
}

impl AcademicGraphEdgeV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: &ProjectId,
        source_node_id: impl Into<String>,
        relation: AcademicGraphRelation,
        target_node_id: impl Into<String>,
        mut layers: Vec<AcademicGraphLayer>,
        rationale: impl Into<String>,
        artifact_path: impl Into<String>,
        source_anchor: impl Into<String>,
        evidence_limit: impl Into<String>,
        inference_strength: AcademicInferenceStrength,
        confidence: AcademicGraphConfidence,
        status: AcademicGraphEdgeStatus,
        created_from_capture: Option<CaptureId>,
    ) -> Result<Self, ProjectError> {
        layers.sort_unstable();
        layers.dedup();
        let source_node_id = source_node_id.into();
        let target_node_id = target_node_id.into();
        let artifact_path = artifact_path.into();
        let source_anchor = source_anchor.into();
        let edge_id = edge_id(
            project_id,
            &source_node_id,
            relation,
            &target_node_id,
            &artifact_path,
            &source_anchor,
        )?;
        let edge = Self {
            edge_id,
            source_node_id,
            relation,
            target_node_id,
            layers,
            rationale: rationale.into(),
            artifact_path,
            source_anchor,
            evidence_limit: evidence_limit.into(),
            inference_strength,
            confidence,
            status,
            created_from_capture,
        };
        edge.validate(project_id)?;
        Ok(edge)
    }

    pub(crate) fn validate(&self, project_id: &ProjectId) -> Result<(), ProjectError> {
        if !valid_graph_id(&self.edge_id, "edg_")
            || !valid_graph_id(&self.source_node_id, "nod_")
            || !valid_graph_id(&self.target_node_id, "nod_")
            || self.source_node_id == self.target_node_id
            || !valid_layers(&self.layers)
            || !valid_text(&self.rationale, MAX_RATIONALE_BYTES)
            || !valid_artifact_path(&self.artifact_path)
            || !valid_anchor(&self.source_anchor)
            || !valid_text(&self.evidence_limit, MAX_EVIDENCE_LIMIT_BYTES)
            || (self.inference_strength == AcademicInferenceStrength::UnsupportedGap
                && self.confidence == AcademicGraphConfidence::High)
            || self
                .created_from_capture
                .as_ref()
                .is_some_and(|capture_id| CaptureId::parse(capture_id.as_str()).is_err())
            || self.edge_id
                != edge_id(
                    project_id,
                    &self.source_node_id,
                    self.relation,
                    &self.target_node_id,
                    &self.artifact_path,
                    &self.source_anchor,
                )?
        {
            return Err(ProjectError::InvalidGraphDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcademicGraphDiagnosticV1 {
    pub code: AcademicGraphDiagnosticCode,
    pub artifact_path: String,
    pub source_anchor: Option<String>,
    pub related_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcademicGraphSnapshotV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub projection_id: String,
    pub projection_digest: String,
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub project_stage: ProjectStage,
    pub project_lifecycle: ProjectLifecycle,
    pub project_manifest_digest: String,
    pub project_semantic_digest: String,
    pub graph_source_digest: String,
    pub source_count: usize,
    pub present_source_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
    pub diagnostic_count: usize,
    pub sources: Vec<AcademicGraphSourceRefV1>,
    pub nodes: Vec<AcademicGraphNodeV1>,
    pub edges: Vec<AcademicGraphEdgeV1>,
    pub diagnostics: Vec<AcademicGraphDiagnosticV1>,
}

impl AcademicGraphSnapshotV1 {
    pub(crate) fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version != ACADEMIC_GRAPH_SCHEMA_VERSION
            || self.document_kind != ACADEMIC_GRAPH_DOCUMENT_KIND
            || self.project_revision == 0
            || !valid_lower_hex(&self.projection_digest, 64)
            || self.projection_id != format!("grp_{}", self.projection_digest)
            || !valid_lower_hex(&self.project_manifest_digest, 64)
            || !valid_lower_hex(&self.project_semantic_digest, 64)
            || !valid_lower_hex(&self.graph_source_digest, 64)
            || self.source_count != self.sources.len()
            || self.present_source_count
                != self.sources.iter().filter(|source| source.present).count()
            || self.node_count != self.nodes.len()
            || self.edge_count != self.edges.len()
            || self.diagnostic_count != self.diagnostics.len()
            || self.nodes.len() > MAX_GRAPH_NODES
            || self.edges.len() > MAX_GRAPH_EDGES
            || self.diagnostics.len() > MAX_GRAPH_DIAGNOSTICS
        {
            return Err(ProjectError::InvalidGraphDocument);
        }
        self.project_id.validate()?;

        let expected_paths = std::iter::once((
            PROJECT_MANIFEST_RELATIVE_PATH,
            AcademicGraphSourceKind::ProjectManifest,
        ))
        .chain(
            SEMANTIC_ARTIFACTS
                .iter()
                .copied()
                .map(|path| (path, AcademicGraphSourceKind::RegisteredArtifact)),
        )
        .chain(std::iter::once((
            GRAPH_SEMANTIC_LINKS_RELATIVE_PATH,
            AcademicGraphSourceKind::SemanticLinks,
        )))
        .collect::<BTreeMap<_, _>>();
        let observed_paths = self
            .sources
            .iter()
            .map(|source| (source.artifact_path.as_str(), source.source_kind))
            .collect::<BTreeMap<_, _>>();
        if expected_paths != observed_paths
            || !strictly_sorted_by(&self.sources, |source| source.artifact_path.as_str())
            || self.sources.iter().any(|source| {
                !valid_artifact_path(&source.artifact_path)
                    || source.size_bytes > MAX_GRAPH_SOURCE_BYTES
                    || match (&source.content_digest, source.present) {
                        (Some(digest), true) => !valid_lower_hex(digest, 64),
                        (None, false) => source.size_bytes != 0,
                        _ => true,
                    }
            })
        {
            return Err(ProjectError::InvalidGraphDocument);
        }
        let present_paths = self
            .sources
            .iter()
            .filter(|source| source.present)
            .map(|source| source.artifact_path.as_str())
            .collect::<BTreeSet<_>>();

        if !strictly_sorted_by(&self.nodes, |node| node.node_id.as_str())
            || !strictly_sorted_by(&self.edges, |edge| edge.edge_id.as_str())
            || !strictly_sorted_diagnostics(&self.diagnostics)
        {
            return Err(ProjectError::InvalidGraphDocument);
        }
        let mut node_ids = BTreeSet::new();
        for node in &self.nodes {
            node.validate(&self.project_id)?;
            if !present_paths.contains(node.artifact_path.as_str())
                || !node_ids.insert(node.node_id.as_str())
            {
                return Err(ProjectError::InvalidGraphDocument);
            }
        }
        if !self.nodes.iter().any(|node| {
            node.node_type == AcademicGraphNodeType::Project
                && node.identity_scope == AcademicGraphIdentityScope::Project
                && node.canonical_id == self.project_id.as_str()
        }) {
            return Err(ProjectError::InvalidGraphDocument);
        }
        for edge in &self.edges {
            edge.validate(&self.project_id)?;
            if !present_paths.contains(edge.artifact_path.as_str())
                || !node_ids.contains(edge.source_node_id.as_str())
                || !node_ids.contains(edge.target_node_id.as_str())
            {
                return Err(ProjectError::InvalidGraphDocument);
            }
        }
        for diagnostic in &self.diagnostics {
            if !valid_artifact_path(&diagnostic.artifact_path)
                || !present_paths.contains(diagnostic.artifact_path.as_str())
                || diagnostic
                    .source_anchor
                    .as_deref()
                    .is_some_and(|anchor| !valid_anchor(anchor))
                || diagnostic
                    .related_id
                    .as_deref()
                    .is_some_and(|id| !valid_text(id, MAX_CANONICAL_ID_BYTES))
            {
                return Err(ProjectError::InvalidGraphDocument);
            }
        }

        let graph_source_digest =
            canonical_domain_digest(b"qiongli-academic-graph-sources-v1\0", &self.sources)?;
        let semantics = ProjectionSemantics {
            schema_version: self.schema_version,
            project_id: &self.project_id,
            project_revision: self.project_revision,
            project_stage: self.project_stage,
            project_lifecycle: self.project_lifecycle,
            project_manifest_digest: &self.project_manifest_digest,
            project_semantic_digest: &self.project_semantic_digest,
            graph_source_digest: &graph_source_digest,
            sources: &self.sources,
            nodes: &self.nodes,
            edges: &self.edges,
            diagnostics: &self.diagnostics,
        };
        let projection_digest =
            canonical_domain_digest(b"qiongli-academic-graph-projection-v1\0", &semantics)?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| ProjectError::InvalidGraphDocument)?;
        if graph_source_digest != self.graph_source_digest
            || projection_digest != self.projection_digest
            || bytes.len() > MAX_GRAPH_SNAPSHOT_BYTES
        {
            return Err(ProjectError::InvalidGraphDocument);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct AcademicGraphService {
    projects: ProjectStateService,
}

impl AcademicGraphService {
    #[must_use]
    pub const fn new(projects: ProjectStateService) -> Self {
        Self { projects }
    }

    pub fn rebuild(&self, project_id: &ProjectId) -> Result<AcademicGraphSnapshotV1, ProjectError> {
        let library = self.projects.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        let (manifest, observed_manifest_digest) =
            read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *project_id
            || entry.project_id != manifest.project_id
            || entry.display_name != manifest.display_name
            || entry.project_kind != manifest.project_kind
            || entry.semantic_revision != manifest.semantic_revision
            || entry.semantic_digest != manifest.semantic_digest
            || entry.stage != manifest.stage
            || entry.lifecycle != manifest.lifecycle
            || entry.academically_updated_at_unix != manifest.academically_updated_at_unix
            || semantic_digest(&root)? != manifest.semantic_digest
        {
            return Err(ProjectError::RevisionConflict);
        }

        let manifest_bytes = serde_json_canonicalizer::to_vec(&manifest)
            .map_err(|_| ProjectError::InvalidGraphDocument)?;
        let manifest_digest = sha256(&manifest_bytes);

        let mut sources = vec![AcademicGraphSourceRefV1 {
            source_kind: AcademicGraphSourceKind::ProjectManifest,
            artifact_path: PROJECT_MANIFEST_RELATIVE_PATH.to_string(),
            present: true,
            content_digest: Some(manifest_digest.clone()),
            size_bytes: manifest_bytes.len() as u64,
        }];
        let mut artifact_inputs = Vec::new();
        for relative_path in SEMANTIC_ARTIFACTS {
            match read_semantic_artifact(&root, relative_path)? {
                Some((bytes, digest)) => {
                    artifact_inputs.push((relative_path, bytes.clone()));
                    sources.push(AcademicGraphSourceRefV1 {
                        source_kind: AcademicGraphSourceKind::RegisteredArtifact,
                        artifact_path: relative_path.to_string(),
                        present: true,
                        content_digest: Some(digest),
                        size_bytes: bytes.len() as u64,
                    });
                }
                None => sources.push(AcademicGraphSourceRefV1 {
                    source_kind: AcademicGraphSourceKind::RegisteredArtifact,
                    artifact_path: relative_path.to_string(),
                    present: false,
                    content_digest: None,
                    size_bytes: 0,
                }),
            }
        }

        let semantic_link_bytes = read_graph_semantic_links(&root)?;
        let parsed = parse_semantic_records(semantic_link_bytes.as_deref(), project_id)?;
        sources.push(AcademicGraphSourceRefV1 {
            source_kind: AcademicGraphSourceKind::SemanticLinks,
            artifact_path: GRAPH_SEMANTIC_LINKS_RELATIVE_PATH.to_string(),
            present: semantic_link_bytes.is_some(),
            content_digest: semantic_link_bytes
                .as_ref()
                .map(|_| parsed.semantic_digest.clone()),
            size_bytes: if semantic_link_bytes.is_some() {
                parsed.semantic_size_bytes
            } else {
                0
            },
        });
        sources.sort_by(|left, right| left.artifact_path.cmp(&right.artifact_path));

        let present_paths = sources
            .iter()
            .filter(|source| source.present)
            .map(|source| source.artifact_path.as_str())
            .collect::<BTreeSet<_>>();
        let mut nodes = BTreeMap::new();
        let project_node = AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Project,
            AcademicGraphIdentityScope::Project,
            project_id.as_str(),
            manifest.display_name.clone(),
            vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
            PROJECT_MANIFEST_RELATIVE_PATH,
            "#/project_id",
        )?;
        let project_node_id = project_node.node_id.clone();
        nodes.insert(project_node.node_id.clone(), project_node);
        let mut source_node_ids = BTreeMap::from([(
            PROJECT_MANIFEST_RELATIVE_PATH.to_string(),
            project_node_id.clone(),
        )]);

        let mut edges = BTreeMap::new();
        for source in sources.iter().filter(|source| {
            source.present && source.source_kind != AcademicGraphSourceKind::ProjectManifest
        }) {
            let layers = artifact_layers(&source.artifact_path);
            let artifact_node = AcademicGraphNodeV1::new(
                project_id,
                AcademicGraphNodeType::Artifact,
                AcademicGraphIdentityScope::Project,
                &source.artifact_path,
                artifact_label(&source.artifact_path),
                layers.clone(),
                &source.artifact_path,
                "document",
            )?;
            let edge = AcademicGraphEdgeV1::new(
                project_id,
                &project_node_id,
                AcademicGraphRelation::Contains,
                &artifact_node.node_id,
                layers,
                "The registered project contains this canonical graph source.",
                &source.artifact_path,
                "document",
                "Structural containment only; no scholarly support is implied.",
                AcademicInferenceStrength::DirectEvidence,
                AcademicGraphConfidence::High,
                AcademicGraphEdgeStatus::Observed,
                None,
            )?;
            source_node_ids.insert(source.artifact_path.clone(), artifact_node.node_id.clone());
            nodes.insert(artifact_node.node_id.clone(), artifact_node);
            edges.insert(edge.edge_id.clone(), edge);
        }

        let mut diagnostics = Vec::new();
        let mut extracted_edges = Vec::new();
        for (artifact_path, bytes) in artifact_inputs {
            let extracted = extract_academic_artifact(project_id, artifact_path, &bytes)?;
            diagnostics.extend(extracted.diagnostics);
            for node in extracted.nodes {
                if let Some(existing) = nodes.get_mut(&node.node_id) {
                    if !merge_compatible_node(project_id, existing, &node)? && existing != &node {
                        diagnostics.push(AcademicGraphDiagnosticV1 {
                            code: AcademicGraphDiagnosticCode::ConflictingIdentity,
                            artifact_path: node.artifact_path.clone(),
                            source_anchor: Some(node.source_anchor.clone()),
                            related_id: Some(node.node_id.clone()),
                        });
                    }
                } else {
                    nodes.insert(node.node_id.clone(), node);
                }
            }
            extracted_edges.extend(extracted.edges);
        }
        for edge in extracted_edges {
            if !nodes.contains_key(&edge.source_node_id)
                || !nodes.contains_key(&edge.target_node_id)
            {
                diagnostics.push(AcademicGraphDiagnosticV1 {
                    code: AcademicGraphDiagnosticCode::DanglingNode,
                    artifact_path: edge.artifact_path.clone(),
                    source_anchor: Some(edge.source_anchor.clone()),
                    related_id: Some(edge.edge_id.clone()),
                });
                continue;
            }
            match edges.get(&edge.edge_id) {
                Some(existing) if existing != &edge => {
                    diagnostics.push(AcademicGraphDiagnosticV1 {
                        code: AcademicGraphDiagnosticCode::ConflictingIdentity,
                        artifact_path: edge.artifact_path.clone(),
                        source_anchor: Some(edge.source_anchor.clone()),
                        related_id: Some(edge.edge_id.clone()),
                    })
                }
                Some(_) => {}
                None => {
                    edges.insert(edge.edge_id.clone(), edge);
                }
            }
        }

        for node in parsed.nodes {
            node.validate(project_id)?;
            if !present_paths.contains(node.artifact_path.as_str()) {
                return Err(ProjectError::InvalidGraphDocument);
            }
            if let Some(existing) = nodes.get_mut(&node.node_id) {
                if !merge_compatible_node(project_id, existing, &node)? && existing != &node {
                    diagnostics.push(AcademicGraphDiagnosticV1 {
                        code: AcademicGraphDiagnosticCode::ConflictingIdentity,
                        artifact_path: node.artifact_path.clone(),
                        source_anchor: Some(node.source_anchor.clone()),
                        related_id: Some(node.node_id.clone()),
                    });
                }
            } else {
                nodes.insert(node.node_id.clone(), node);
            }
        }
        if nodes.len() > MAX_GRAPH_NODES {
            return Err(ProjectError::InvalidGraphDocument);
        }
        for node in nodes.values().filter(|node| {
            !matches!(
                node.node_type,
                AcademicGraphNodeType::Project | AcademicGraphNodeType::Artifact
            )
        }) {
            let source_node_id = source_node_ids
                .get(&node.artifact_path)
                .ok_or(ProjectError::InvalidGraphDocument)?;
            let edge = AcademicGraphEdgeV1::new(
                project_id,
                source_node_id,
                AcademicGraphRelation::Contains,
                &node.node_id,
                node.layers.clone(),
                "This canonical graph source defines this academic record.",
                &node.artifact_path,
                &node.source_anchor,
                "Structural containment only; no scholarly support is implied.",
                AcademicInferenceStrength::DirectEvidence,
                AcademicGraphConfidence::High,
                AcademicGraphEdgeStatus::Observed,
                None,
            )?;
            match edges.get(&edge.edge_id) {
                Some(existing) if existing != &edge => {
                    diagnostics.push(AcademicGraphDiagnosticV1 {
                        code: AcademicGraphDiagnosticCode::ConflictingIdentity,
                        artifact_path: edge.artifact_path.clone(),
                        source_anchor: Some(edge.source_anchor.clone()),
                        related_id: Some(edge.edge_id.clone()),
                    });
                }
                Some(_) => {}
                None => {
                    edges.insert(edge.edge_id.clone(), edge);
                }
            }
        }
        for edge in parsed.edges {
            edge.validate(project_id)?;
            if !present_paths.contains(edge.artifact_path.as_str())
                || !nodes.contains_key(&edge.source_node_id)
                || !nodes.contains_key(&edge.target_node_id)
            {
                return Err(ProjectError::InvalidGraphDocument);
            }
            match edges.get(&edge.edge_id) {
                Some(existing) if existing != &edge => {
                    diagnostics.push(AcademicGraphDiagnosticV1 {
                        code: AcademicGraphDiagnosticCode::ConflictingIdentity,
                        artifact_path: edge.artifact_path.clone(),
                        source_anchor: Some(edge.source_anchor.clone()),
                        related_id: Some(edge.edge_id.clone()),
                    });
                }
                Some(_) => {}
                None => {
                    edges.insert(edge.edge_id.clone(), edge);
                }
            }
        }
        if edges.len() > MAX_GRAPH_EDGES {
            return Err(ProjectError::InvalidGraphDocument);
        }

        let (manifest_after, observed_manifest_digest_after) =
            read_manifest(&root)?.ok_or(ProjectError::RevisionConflict)?;
        if manifest_after != manifest
            || observed_manifest_digest_after != observed_manifest_digest
            || semantic_digest(&root)? != manifest.semantic_digest
            || read_graph_semantic_links(&root)? != semantic_link_bytes
        {
            return Err(ProjectError::RevisionConflict);
        }
        let library_after = self.projects.store.load()?;
        library_after.validate()?;
        let entry_after = library_after
            .projects
            .iter()
            .find(|candidate| &candidate.project_id == project_id)
            .ok_or(ProjectError::RevisionConflict)?;
        if entry_after.root_path != entry.root_path
            || entry_after.display_name != manifest.display_name
            || entry_after.project_kind != manifest.project_kind
            || entry_after.semantic_revision != manifest.semantic_revision
            || entry_after.semantic_digest != manifest.semantic_digest
            || entry_after.stage != manifest.stage
            || entry_after.lifecycle != manifest.lifecycle
            || entry_after.academically_updated_at_unix != manifest.academically_updated_at_unix
        {
            return Err(ProjectError::RevisionConflict);
        }

        let nodes = nodes.into_values().collect::<Vec<_>>();
        let edges = edges.into_values().collect::<Vec<_>>();
        diagnostics.sort_by(|left, right| {
            left.code
                .cmp(&right.code)
                .then_with(|| left.artifact_path.cmp(&right.artifact_path))
                .then_with(|| left.source_anchor.cmp(&right.source_anchor))
                .then_with(|| left.related_id.cmp(&right.related_id))
        });
        diagnostics.dedup();
        if diagnostics.len() > MAX_GRAPH_DIAGNOSTICS {
            return Err(ProjectError::InvalidGraphDocument);
        }
        let graph_source_digest =
            canonical_domain_digest(b"qiongli-academic-graph-sources-v1\0", &sources)?;
        let semantics = ProjectionSemantics {
            schema_version: ACADEMIC_GRAPH_SCHEMA_VERSION,
            project_id,
            project_revision: manifest.semantic_revision,
            project_stage: manifest.stage,
            project_lifecycle: manifest.lifecycle,
            project_manifest_digest: &manifest_digest,
            project_semantic_digest: &manifest.semantic_digest,
            graph_source_digest: &graph_source_digest,
            sources: &sources,
            nodes: &nodes,
            edges: &edges,
            diagnostics: &diagnostics,
        };
        let projection_digest =
            canonical_domain_digest(b"qiongli-academic-graph-projection-v1\0", &semantics)?;
        let snapshot = AcademicGraphSnapshotV1 {
            schema_version: ACADEMIC_GRAPH_SCHEMA_VERSION,
            document_kind: ACADEMIC_GRAPH_DOCUMENT_KIND.to_string(),
            projection_id: format!("grp_{projection_digest}"),
            projection_digest,
            project_id: project_id.clone(),
            project_revision: manifest.semantic_revision,
            project_stage: manifest.stage,
            project_lifecycle: manifest.lifecycle,
            project_manifest_digest: manifest_digest,
            project_semantic_digest: manifest.semantic_digest,
            graph_source_digest,
            source_count: sources.len(),
            present_source_count: sources.iter().filter(|source| source.present).count(),
            node_count: nodes.len(),
            edge_count: edges.len(),
            diagnostic_count: diagnostics.len(),
            sources,
            nodes,
            edges,
            diagnostics,
        };
        let bytes = serde_json_canonicalizer::to_vec(&snapshot)
            .map_err(|_| ProjectError::InvalidGraphDocument)?;
        if bytes.len() > MAX_GRAPH_SNAPSHOT_BYTES {
            return Err(ProjectError::InvalidGraphDocument);
        }
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn rebuild_projection(
        &self,
        project_id: &ProjectId,
    ) -> Result<AcademicGraphProjectionV1, ProjectError> {
        let graph = self.rebuild(project_id)?;
        let catalog = self.projects.portfolio_catalog_store.rebuild()?;
        let last_successful = catalog.as_ref().and_then(|catalog| {
            catalog
                .contributions
                .iter()
                .find(|contribution| &contribution.project_id == project_id)
                .map(|contribution| &contribution.graph)
        });
        let readiness =
            AcademicGraphReadinessV1::from_graph_and_last_successful(&graph, last_successful);
        Ok(AcademicGraphProjectionV1 {
            schema_version: ACADEMIC_GRAPH_PROJECTION_SCHEMA_VERSION,
            document_kind: ACADEMIC_GRAPH_PROJECTION_DOCUMENT_KIND.to_owned(),
            graph,
            readiness,
        })
    }

    pub fn resolve_artifact(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        expected_projection_id: &str,
        entity_kind: AcademicGraphEntityKind,
        entity_id: &str,
    ) -> Result<AcademicGraphArtifactTarget, ProjectError> {
        let valid_entity_id = match entity_kind {
            AcademicGraphEntityKind::Node => valid_graph_id(entity_id, "nod_"),
            AcademicGraphEntityKind::Edge => valid_graph_id(entity_id, "edg_"),
        };
        if expected_project_revision == 0
            || !valid_graph_id(expected_projection_id, "grp_")
            || !valid_entity_id
        {
            return Err(ProjectError::InvalidGraphQuery);
        }

        let snapshot = self.rebuild(project_id)?;
        if snapshot.project_revision != expected_project_revision
            || snapshot.projection_id != expected_projection_id
        {
            return Err(ProjectError::RevisionConflict);
        }
        let (artifact_path, source_anchor) = match entity_kind {
            AcademicGraphEntityKind::Node => snapshot
                .nodes
                .iter()
                .find(|node| node.node_id == entity_id)
                .map(|node| (node.artifact_path.clone(), node.source_anchor.clone())),
            AcademicGraphEntityKind::Edge => snapshot
                .edges
                .iter()
                .find(|edge| edge.edge_id == entity_id)
                .map(|edge| (edge.artifact_path.clone(), edge.source_anchor.clone())),
        }
        .ok_or(ProjectError::GraphEntityNotFound)?;

        let root = self.projects.resolve_project_root(project_id)?;
        let path = resolve_academic_graph_artifact_path(root.path(), &artifact_path)?;
        let confirmed = self.rebuild(project_id)?;
        if confirmed.project_revision != snapshot.project_revision
            || confirmed.projection_id != snapshot.projection_id
        {
            return Err(ProjectError::RevisionConflict);
        }

        Ok(AcademicGraphArtifactTarget {
            path,
            project_id: project_id.clone(),
            project_revision: snapshot.project_revision,
            projection_id: snapshot.projection_id,
            entity_kind,
            entity_id: entity_id.to_owned(),
            artifact_path,
            source_anchor,
        })
    }

    pub fn read_graph_artifact(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        expected_projection_id: &str,
        entity_kind: AcademicGraphEntityKind,
        entity_id: &str,
        max_bytes: usize,
    ) -> Result<ProjectArtifactViewV1, ProjectError> {
        validate_project_artifact_view_bytes(max_bytes)?;
        let target = self.resolve_artifact(
            project_id,
            expected_project_revision,
            expected_projection_id,
            entity_kind,
            entity_id,
        )?;
        let root = self.projects.resolve_project_root(project_id)?;
        let bytes = read_academic_graph_artifact(root.path(), &target.artifact_path)?;
        let confirmed = self.rebuild(project_id)?;
        if confirmed.project_revision != target.project_revision
            || confirmed.projection_id != target.projection_id
        {
            return Err(ProjectError::RevisionConflict);
        }
        project_artifact_view(
            target.project_id,
            target.project_revision,
            Some(target.projection_id),
            Some(target.entity_kind),
            Some(target.entity_id),
            target.artifact_path,
            Some(target.source_anchor),
            bytes,
            max_bytes,
        )
    }

    pub fn read_registered_artifact(
        &self,
        project_id: &ProjectId,
        expected_project_revision: u64,
        artifact_path: &str,
        source_anchor: Option<&str>,
        max_bytes: usize,
    ) -> Result<ProjectArtifactViewV1, ProjectError> {
        validate_project_artifact_view_bytes(max_bytes)?;
        if expected_project_revision == 0
            || source_anchor.is_some_and(|anchor| {
                anchor.is_empty()
                    || anchor.len() > MAX_ANCHOR_BYTES
                    || anchor.trim() != anchor
                    || anchor.chars().any(char::is_control)
            })
        {
            return Err(ProjectError::InvalidGraphQuery);
        }
        let root = self.projects.resolve_project_root(project_id)?;
        let (before, before_digest) =
            read_manifest(root.path())?.ok_or(ProjectError::ProjectManifestMissing)?;
        if &before.project_id != project_id || before.semantic_revision != expected_project_revision
        {
            return Err(ProjectError::RevisionConflict);
        }
        if semantic_digest(root.path())? != before.semantic_digest {
            return Err(ProjectError::RevisionConflict);
        }
        let bytes = read_academic_graph_artifact(root.path(), artifact_path)?;
        let (after, after_digest) =
            read_manifest(root.path())?.ok_or(ProjectError::RevisionConflict)?;
        if after != before
            || after_digest != before_digest
            || semantic_digest(root.path())? != before.semantic_digest
        {
            return Err(ProjectError::RevisionConflict);
        }
        project_artifact_view(
            project_id.clone(),
            expected_project_revision,
            None,
            None,
            None,
            artifact_path.to_owned(),
            source_anchor.map(str::to_owned),
            bytes,
            max_bytes,
        )
    }
}

fn validate_project_artifact_view_bytes(max_bytes: usize) -> Result<(), ProjectError> {
    if !(MIN_PROJECT_ARTIFACT_VIEW_BYTES..=MAX_PROJECT_ARTIFACT_VIEW_BYTES).contains(&max_bytes) {
        return Err(ProjectError::InvalidGraphQuery);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn project_artifact_view(
    project_id: ProjectId,
    project_revision: u64,
    projection_id: Option<String>,
    entity_kind: Option<AcademicGraphEntityKind>,
    entity_id: Option<String>,
    artifact_path: String,
    source_anchor: Option<String>,
    bytes: Vec<u8>,
    max_bytes: usize,
) -> Result<ProjectArtifactViewV1, ProjectError> {
    let source =
        std::str::from_utf8(&bytes).map_err(|_| ProjectError::ProjectArtifactContentInvalid)?;
    let format = project_artifact_format(&artifact_path)?;
    let anchor_byte = source_anchor
        .as_deref()
        .and_then(|anchor| locate_project_artifact_anchor(source, format, anchor));
    let (start, end) =
        project_artifact_window(source, anchor_byte, source_anchor.as_deref(), max_bytes);
    let content = source[start..end].to_owned();
    let start_line = 1 + source[..start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u64;
    let end_line = start_line + content.bytes().filter(|byte| *byte == b'\n').count() as u64;
    let anchor_line = anchor_byte.map(|offset| {
        1 + source[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count() as u64
    });
    Ok(ProjectArtifactViewV1 {
        schema_version: PROJECT_ARTIFACT_VIEW_SCHEMA_VERSION,
        document_kind: PROJECT_ARTIFACT_VIEW_DOCUMENT_KIND.to_owned(),
        project_id,
        project_revision,
        projection_id,
        entity_kind,
        entity_id,
        artifact_path,
        source_anchor,
        format,
        content_digest: crate::storage::sha256_bytes(&bytes),
        source_size_bytes: bytes.len() as u64,
        content_size_bytes: content.len() as u64,
        content,
        start_line,
        end_line,
        anchor_line,
        anchor_matched: anchor_byte.is_some(),
        truncated_before: start > 0,
        truncated_after: end < source.len(),
    })
}

fn project_artifact_format(artifact_path: &str) -> Result<ProjectArtifactFormat, ProjectError> {
    if artifact_path.ends_with(".md") {
        Ok(ProjectArtifactFormat::Markdown)
    } else if artifact_path.ends_with(".csv") {
        Ok(ProjectArtifactFormat::Csv)
    } else if artifact_path.ends_with(".jsonl") {
        Ok(ProjectArtifactFormat::JsonLines)
    } else if artifact_path.ends_with(".json") {
        Ok(ProjectArtifactFormat::Json)
    } else {
        Err(ProjectError::ProjectArtifactUnsupported)
    }
}

fn locate_project_artifact_anchor(
    source: &str,
    format: ProjectArtifactFormat,
    anchor: &str,
) -> Option<usize> {
    let structured = (|| {
        if format == ProjectArtifactFormat::Csv
            && let Some(line) =
                crate::academic_graph_extract::evidence_ledger_anchor_line(source, anchor)
        {
            return project_artifact_line_offset(source, line);
        }
        if let Some(line) = anchor
            .strip_prefix("line:")
            .or_else(|| anchor.strip_prefix("row:"))
        {
            let line = line.parse::<usize>().ok()?;
            return project_artifact_line_offset(source, line);
        }
        if format == ProjectArtifactFormat::Markdown
            && let Some(field) = anchor.strip_prefix("field:")
        {
            if field.is_empty() {
                return None;
            }
            return source.find(field);
        }
        if format == ProjectArtifactFormat::Json {
            let pointer = anchor.strip_prefix("#/")?;
            let key = pointer
                .rsplit('/')
                .next()?
                .replace("~1", "/")
                .replace("~0", "~");
            return source.find(&format!("\"{key}\""));
        }
        None
    })();
    structured.or_else(|| source.find(anchor))
}

fn project_artifact_line_offset(source: &str, one_based_line: usize) -> Option<usize> {
    if one_based_line == 0 {
        return None;
    }
    if one_based_line == 1 {
        return Some(0);
    }
    source
        .match_indices('\n')
        .nth(one_based_line - 2)
        .map(|(offset, _)| offset + 1)
        .filter(|offset| *offset < source.len())
}

fn project_artifact_window(
    source: &str,
    anchor_byte: Option<usize>,
    source_anchor: Option<&str>,
    max_bytes: usize,
) -> (usize, usize) {
    if source.len() <= max_bytes {
        return (0, source.len());
    }
    let mut start = anchor_byte
        .map(|offset| offset.saturating_sub(max_bytes / 3))
        .unwrap_or(0);
    if let (Some(offset), Some(anchor)) = (anchor_byte, source_anchor) {
        start = start.min(offset);
        let anchor_end = offset.saturating_add(anchor.len());
        if anchor_end > start.saturating_add(max_bytes) {
            start = anchor_end.saturating_sub(max_bytes);
        }
    }
    start = start.min(source.len().saturating_sub(max_bytes));
    while start < source.len() && !source.is_char_boundary(start) {
        start += 1;
    }
    let mut end = start.saturating_add(max_bytes).min(source.len());
    while end > start && !source.is_char_boundary(end) {
        end -= 1;
    }
    (start, end)
}

impl Debug for AcademicGraphService {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AcademicGraphService")
            .field("projects", &"<project-state-service>")
            .finish()
    }
}

#[derive(Serialize)]
struct ProjectionSemantics<'a> {
    schema_version: u32,
    project_id: &'a ProjectId,
    project_revision: u64,
    project_stage: ProjectStage,
    project_lifecycle: ProjectLifecycle,
    project_manifest_digest: &'a str,
    project_semantic_digest: &'a str,
    graph_source_digest: &'a str,
    sources: &'a [AcademicGraphSourceRefV1],
    nodes: &'a [AcademicGraphNodeV1],
    edges: &'a [AcademicGraphEdgeV1],
    diagnostics: &'a [AcademicGraphDiagnosticV1],
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SemanticNodeRecordV1 {
    schema_version: u32,
    document_kind: String,
    project_id: ProjectId,
    node_id: String,
    node_type: AcademicGraphNodeType,
    identity_scope: AcademicGraphIdentityScope,
    canonical_id: String,
    label: String,
    layers: Vec<AcademicGraphLayer>,
    artifact_path: String,
    source_anchor: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SemanticEdgeRecordV1 {
    schema_version: u32,
    document_kind: String,
    project_id: ProjectId,
    edge_id: String,
    source_node_id: String,
    relation: AcademicGraphRelation,
    target_node_id: String,
    layers: Vec<AcademicGraphLayer>,
    rationale: String,
    artifact_path: String,
    source_anchor: String,
    evidence_limit: String,
    inference_strength: AcademicInferenceStrength,
    confidence: AcademicGraphConfidence,
    status: AcademicGraphEdgeStatus,
    created_from_capture: Option<CaptureId>,
}

struct ParsedSemanticRecords {
    nodes: Vec<AcademicGraphNodeV1>,
    edges: Vec<AcademicGraphEdgeV1>,
    semantic_digest: String,
    semantic_size_bytes: u64,
    semantic_bytes: Vec<u8>,
}

fn parse_semantic_records(
    bytes: Option<&[u8]>,
    project_id: &ProjectId,
) -> Result<ParsedSemanticRecords, ProjectError> {
    parse_semantic_records_inner(bytes, Some(project_id))
}

pub(crate) fn canonical_semantic_links_bytes(
    bytes: &[u8],
    expected_project_id: Option<&ProjectId>,
) -> Result<Vec<u8>, ProjectError> {
    parse_semantic_records_inner(Some(bytes), expected_project_id)
        .map(|parsed| parsed.semantic_bytes)
}

fn parse_semantic_records_inner(
    bytes: Option<&[u8]>,
    expected_project_id: Option<&ProjectId>,
) -> Result<ParsedSemanticRecords, ProjectError> {
    let Some(bytes) = bytes else {
        let semantic_bytes = serde_json_canonicalizer::to_vec(&Vec::<serde_json::Value>::new())
            .map_err(|_| ProjectError::InvalidGraphDocument)?;
        return Ok(ParsedSemanticRecords {
            nodes: Vec::new(),
            edges: Vec::new(),
            semantic_digest: domain_digest_bytes(
                b"qiongli-academic-semantic-records-v1\0",
                &semantic_bytes,
            ),
            semantic_size_bytes: 0,
            semantic_bytes,
        });
    };
    let text = std::str::from_utf8(bytes).map_err(|_| ProjectError::InvalidGraphDocument)?;
    let mut nodes = BTreeMap::new();
    let mut edges = BTreeMap::new();
    let mut record_project_ids = BTreeSet::new();
    let mut canonical_records = Vec::new();
    let mut record_count = 0usize;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        record_count = record_count
            .checked_add(1)
            .filter(|count| *count <= MAX_GRAPH_RECORDS)
            .ok_or(ProjectError::InvalidGraphDocument)?;
        if line.len() > MAX_GRAPH_LINE_BYTES {
            return Err(ProjectError::InvalidGraphDocument);
        }
        let value =
            parse_unique_json(line.as_bytes()).map_err(|_| ProjectError::InvalidGraphDocument)?;
        let document_kind = value
            .get("document_kind")
            .and_then(serde_json::Value::as_str)
            .ok_or(ProjectError::InvalidGraphDocument)?;
        match document_kind {
            SEMANTIC_NODE_DOCUMENT_KIND => {
                let record: SemanticNodeRecordV1 = serde_json::from_value(value)
                    .map_err(|_| ProjectError::InvalidGraphDocument)?;
                if record.schema_version != ACADEMIC_GRAPH_SCHEMA_VERSION
                    || record.document_kind != SEMANTIC_NODE_DOCUMENT_KIND
                    || expected_project_id
                        .is_some_and(|project_id| record.project_id != *project_id)
                {
                    return Err(ProjectError::InvalidGraphDocument);
                }
                let record_project_id = record.project_id.clone();
                record_project_ids.insert(record_project_id.clone());
                let node = AcademicGraphNodeV1::new(
                    &record_project_id,
                    record.node_type,
                    record.identity_scope,
                    record.canonical_id,
                    record.label,
                    record.layers,
                    record.artifact_path,
                    record.source_anchor,
                )?;
                if node.node_id != record.node_id
                    || nodes.insert(node.node_id.clone(), node.clone()).is_some()
                {
                    return Err(ProjectError::InvalidGraphDocument);
                }
                canonical_records.push(serde_json::json!({
                    "schemaVersion": ACADEMIC_GRAPH_SCHEMA_VERSION,
                    "documentKind": SEMANTIC_NODE_DOCUMENT_KIND,
                    "projectId": record_project_id,
                    "node": node,
                }));
            }
            SEMANTIC_EDGE_DOCUMENT_KIND => {
                let record: SemanticEdgeRecordV1 = serde_json::from_value(value)
                    .map_err(|_| ProjectError::InvalidGraphDocument)?;
                if record.schema_version != ACADEMIC_GRAPH_SCHEMA_VERSION
                    || record.document_kind != SEMANTIC_EDGE_DOCUMENT_KIND
                    || expected_project_id
                        .is_some_and(|project_id| record.project_id != *project_id)
                {
                    return Err(ProjectError::InvalidGraphDocument);
                }
                let record_project_id = record.project_id.clone();
                record_project_ids.insert(record_project_id.clone());
                let edge = AcademicGraphEdgeV1::new(
                    &record_project_id,
                    record.source_node_id,
                    record.relation,
                    record.target_node_id,
                    record.layers,
                    record.rationale,
                    record.artifact_path,
                    record.source_anchor,
                    record.evidence_limit,
                    record.inference_strength,
                    record.confidence,
                    record.status,
                    record.created_from_capture,
                )?;
                if edge.edge_id != record.edge_id
                    || edges.insert(edge.edge_id.clone(), edge.clone()).is_some()
                {
                    return Err(ProjectError::InvalidGraphDocument);
                }
                canonical_records.push(serde_json::json!({
                    "schemaVersion": ACADEMIC_GRAPH_SCHEMA_VERSION,
                    "documentKind": SEMANTIC_EDGE_DOCUMENT_KIND,
                    "projectId": record_project_id,
                    "edge": edge,
                }));
            }
            _ => return Err(ProjectError::InvalidGraphDocument),
        }
    }
    if record_project_ids.len() > 1 {
        return Err(ProjectError::InvalidGraphDocument);
    }
    canonical_records.sort_by_key(canonical_json_key);
    let semantic_bytes = serde_json_canonicalizer::to_vec(&canonical_records)
        .map_err(|_| ProjectError::InvalidGraphDocument)?;
    Ok(ParsedSemanticRecords {
        nodes: nodes.into_values().collect(),
        edges: edges.into_values().collect(),
        semantic_digest: domain_digest_bytes(
            b"qiongli-academic-semantic-records-v1\0",
            &semantic_bytes,
        ),
        semantic_size_bytes: semantic_bytes.len() as u64,
        semantic_bytes,
    })
}

fn node_id(
    project_id: &ProjectId,
    node_type: AcademicGraphNodeType,
    identity_scope: AcademicGraphIdentityScope,
    canonical_id: &str,
) -> Result<String, ProjectError> {
    #[derive(Serialize)]
    struct Identity<'a> {
        node_type: AcademicGraphNodeType,
        identity_scope: AcademicGraphIdentityScope,
        project_id: Option<&'a ProjectId>,
        canonical_id: &'a str,
    }
    let identity = Identity {
        node_type,
        identity_scope,
        project_id: (identity_scope == AcademicGraphIdentityScope::Project).then_some(project_id),
        canonical_id,
    };
    canonical_domain_digest(b"qiongli-academic-graph-node-v1\0", &identity)
        .map(|digest| format!("nod_{digest}"))
}

fn edge_id(
    project_id: &ProjectId,
    source_node_id: &str,
    relation: AcademicGraphRelation,
    target_node_id: &str,
    artifact_path: &str,
    source_anchor: &str,
) -> Result<String, ProjectError> {
    #[derive(Serialize)]
    struct Identity<'a> {
        project_id: &'a ProjectId,
        source_node_id: &'a str,
        relation: AcademicGraphRelation,
        target_node_id: &'a str,
        artifact_path: &'a str,
        source_anchor: &'a str,
    }
    canonical_domain_digest(
        b"qiongli-academic-graph-edge-v1\0",
        &Identity {
            project_id,
            source_node_id,
            relation,
            target_node_id,
            artifact_path,
            source_anchor,
        },
    )
    .map(|digest| format!("edg_{digest}"))
}

fn merge_compatible_node(
    project_id: &ProjectId,
    existing: &mut AcademicGraphNodeV1,
    incoming: &AcademicGraphNodeV1,
) -> Result<bool, ProjectError> {
    if existing.node_id != incoming.node_id
        || existing.node_type != incoming.node_type
        || existing.identity_scope != incoming.identity_scope
        || existing.canonical_id != incoming.canonical_id
        || existing.label != incoming.label
    {
        return Ok(false);
    }

    existing.layers.extend_from_slice(&incoming.layers);
    existing.layers.sort_unstable();
    existing.layers.dedup();
    existing.validate(project_id)?;
    Ok(true)
}

pub(crate) fn canonical_domain_digest<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, ProjectError> {
    let bytes =
        serde_json_canonicalizer::to_vec(value).map_err(|_| ProjectError::InvalidGraphDocument)?;
    Ok(domain_digest_bytes(domain, &bytes))
}

fn strictly_sorted_by<T, K: Ord + ?Sized>(values: &[T], key: impl Fn(&T) -> &K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

fn strictly_sorted_diagnostics(diagnostics: &[AcademicGraphDiagnosticV1]) -> bool {
    diagnostics.windows(2).all(|pair| {
        (
            pair[0].code,
            pair[0].artifact_path.as_str(),
            pair[0].source_anchor.as_deref(),
            pair[0].related_id.as_deref(),
        ) < (
            pair[1].code,
            pair[1].artifact_path.as_str(),
            pair[1].source_anchor.as_deref(),
            pair[1].related_id.as_deref(),
        )
    })
}

fn domain_digest_bytes(domain: &[u8], bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn sha256(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn canonical_json_key(value: &serde_json::Value) -> Vec<u8> {
    serde_json_canonicalizer::to_vec(value).unwrap_or_default()
}

fn valid_graph_id(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|digest| valid_lower_hex(digest, 64))
}

fn valid_text(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_canonical_id(value: &str) -> bool {
    valid_text(value, MAX_CANONICAL_ID_BYTES) && value.nfc().eq(value.chars())
}

fn valid_layers(layers: &[AcademicGraphLayer]) -> bool {
    !layers.is_empty() && layers.len() <= 6 && layers.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_artifact_path(value: &str) -> bool {
    if !valid_text(value, MAX_PATH_BYTES)
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

fn valid_anchor(value: &str) -> bool {
    let bytes = value.as_bytes();
    let lower = value.to_ascii_lowercase();
    valid_text(value, MAX_ANCHOR_BYTES)
        && value.nfc().eq(value.chars())
        && !value.starts_with(['/', '\\', '~'])
        && !value.contains('\\')
        && !(bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\'))
        && !value.contains("..")
        && !lower.starts_with("file:")
        && !value.contains("://")
}

fn artifact_layers(path: &str) -> Vec<AcademicGraphLayer> {
    let mut layers = match path {
        "literature/literature_map.md" => vec![AcademicGraphLayer::Literature],
        "evidence/claim-evidence-ledger.csv" => vec![AcademicGraphLayer::Argument],
        "manuscript/claims_evidence_map.md" => {
            vec![AcademicGraphLayer::Argument, AcademicGraphLayer::Manuscript]
        }
        GRAPH_SEMANTIC_LINKS_RELATIVE_PATH => vec![AcademicGraphLayer::Combined],
        _ => vec![AcademicGraphLayer::IdeaDecision],
    };
    if !layers.contains(&AcademicGraphLayer::Combined) {
        layers.push(AcademicGraphLayer::Combined);
    }
    layers.sort_unstable();
    layers
}

fn artifact_label(path: &str) -> &'static str {
    match path {
        "context/research_state.md" => "Research state",
        "context/decision_log.md" => "Decision log",
        "context/stage_handoff.md" => "Stage handoff",
        "context/boundary_review.md" => "Boundary review",
        "context/idea_funnel.md" => "Idea funnel",
        "literature/literature_map.md" => "Literature map",
        "evidence/claim-evidence-ledger.csv" => "Claim-evidence ledger",
        "manuscript/claims_evidence_map.md" => "Manuscript claim map",
        GRAPH_SEMANTIC_LINKS_RELATIVE_PATH => "Explicit semantic links",
        _ => "Academic artifact",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use qiongli_config::resolve_config_root;
    use serde_json::json;

    use super::*;
    use crate::{
        AcademicGraphIndexService, AcademicGraphQueryV1, AcademicGraphReadinessState,
        ApprovedProjectMutation, IncrementalPortfolioService, ProjectKind,
        ProjectRegistrationOptions,
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        config: qiongli_config::ConfigRoot,
        project_root: PathBuf,
        project_id: ProjectId,
        projects: ProjectStateService,
        graph: AcademicGraphService,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "qiongli-academic-graph-{}-{nonce}-{}",
                std::process::id(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            fs::create_dir(&home).unwrap();
            let config = resolve_config_root(Some(root.join("config").as_os_str()), &home).unwrap();
            let projects = ProjectStateService::new(config.clone());
            let project_root = root.join("paper");
            let plan = projects
                .preview_create(
                    &project_root,
                    ProjectRegistrationOptions::new("Graph paper", ProjectKind::Article),
                    1,
                )
                .unwrap();
            let project_id = plan.preview().project_id.clone();
            projects
                .apply(
                    &plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    1,
                )
                .unwrap();
            let graph = AcademicGraphService::new(projects.clone());
            Self {
                root,
                config,
                project_root,
                project_id,
                projects,
                graph,
            }
        }

        fn refresh(&self, now_unix: u64) {
            let plan = self
                .projects
                .preview_refresh(&self.project_id, now_unix)
                .unwrap();
            self.projects
                .apply(
                    &plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    now_unix,
                )
                .unwrap();
        }

        fn write_links(&self, records: &[serde_json::Value]) {
            fs::create_dir_all(self.project_root.join("graph")).unwrap();
            let mut bytes = Vec::new();
            for record in records {
                bytes.extend(serde_json::to_vec(record).unwrap());
                bytes.push(b'\n');
            }
            fs::write(
                self.project_root.join(GRAPH_SEMANTIC_LINKS_RELATIVE_PATH),
                bytes,
            )
            .unwrap();
        }

        fn semantic_fixture(&self, rationale: &str, anchor: &str) -> Vec<serde_json::Value> {
            let claim = AcademicGraphNodeV1::new(
                &self.project_id,
                AcademicGraphNodeType::Claim,
                AcademicGraphIdentityScope::Project,
                "claim:C1",
                "Central claim",
                vec![AcademicGraphLayer::Argument],
                "context/research_state.md",
                "claim:C1",
            )
            .unwrap();
            let evidence = AcademicGraphNodeV1::new(
                &self.project_id,
                AcademicGraphNodeType::Evidence,
                AcademicGraphIdentityScope::Project,
                "evidence:E1",
                "Reviewed evidence",
                vec![AcademicGraphLayer::Argument],
                "evidence/claim-evidence-ledger.csv",
                "row:C1",
            )
            .unwrap();
            let edge = AcademicGraphEdgeV1::new(
                &self.project_id,
                &evidence.node_id,
                AcademicGraphRelation::Supports,
                &claim.node_id,
                vec![AcademicGraphLayer::Argument],
                rationale,
                GRAPH_SEMANTIC_LINKS_RELATIVE_PATH,
                anchor,
                "One reviewed source; no causal identification claim.",
                AcademicInferenceStrength::DirectEvidence,
                AcademicGraphConfidence::Medium,
                AcademicGraphEdgeStatus::Reviewed,
                None,
            )
            .unwrap();
            vec![
                node_record(&self.project_id, &claim),
                node_record(&self.project_id, &evidence),
                edge_record(&self.project_id, &edge),
            ]
        }
    }

    #[test]
    fn persisted_successful_build_exposes_stale_sources_across_restart_until_rebuilt() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "- main_question_or_thesis: Initial question\n",
        )
        .unwrap();
        fixture.refresh(2);
        IncrementalPortfolioService::new(fixture.projects.clone())
            .reconcile(3)
            .unwrap();

        let fresh = fixture
            .graph
            .rebuild_projection(&fixture.project_id)
            .unwrap();
        assert_ne!(fresh.readiness.state, AcademicGraphReadinessState::Stale);
        assert_eq!(fresh.readiness.stale_source_count, 0);

        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "- main_question_or_thesis: Revised question\n",
        )
        .unwrap();
        fixture.refresh(4);

        let restarted_projects = ProjectStateService::new(fixture.config.clone());
        let restarted_graph = AcademicGraphService::new(restarted_projects.clone());
        let stale = restarted_graph
            .rebuild_projection(&fixture.project_id)
            .unwrap();
        assert_eq!(stale.readiness.state, AcademicGraphReadinessState::Stale);
        assert_eq!(stale.readiness.last_successful_build.project_revision, 2);
        assert_eq!(stale.readiness.project_revision, 3);
        assert!(stale.readiness.stale_source_count > 0);

        IncrementalPortfolioService::new(restarted_projects)
            .reconcile(5)
            .unwrap();
        let rebuilt = restarted_graph
            .rebuild_projection(&fixture.project_id)
            .unwrap();
        assert_ne!(rebuilt.readiness.state, AcademicGraphReadinessState::Stale);
        assert_eq!(rebuilt.readiness.stale_source_count, 0);
        assert_eq!(
            rebuilt.readiness.last_successful_build.project_revision,
            rebuilt.readiness.project_revision
        );
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn node_record(project_id: &ProjectId, node: &AcademicGraphNodeV1) -> serde_json::Value {
        json!({
            "schema_version": ACADEMIC_GRAPH_SCHEMA_VERSION,
            "document_kind": SEMANTIC_NODE_DOCUMENT_KIND,
            "project_id": project_id,
            "node_id": node.node_id,
            "node_type": node.node_type,
            "identity_scope": node.identity_scope,
            "canonical_id": node.canonical_id,
            "label": node.label,
            "layers": node.layers,
            "artifact_path": node.artifact_path,
            "source_anchor": node.source_anchor,
        })
    }

    fn edge_record(project_id: &ProjectId, edge: &AcademicGraphEdgeV1) -> serde_json::Value {
        json!({
            "schema_version": ACADEMIC_GRAPH_SCHEMA_VERSION,
            "document_kind": SEMANTIC_EDGE_DOCUMENT_KIND,
            "project_id": project_id,
            "edge_id": edge.edge_id,
            "source_node_id": edge.source_node_id,
            "relation": edge.relation,
            "target_node_id": edge.target_node_id,
            "layers": edge.layers,
            "rationale": edge.rationale,
            "artifact_path": edge.artifact_path,
            "source_anchor": edge.source_anchor,
            "evidence_limit": edge.evidence_limit,
            "inference_strength": edge.inference_strength,
            "confidence": edge.confidence,
            "status": edge.status,
            "created_from_capture": edge.created_from_capture,
        })
    }

    #[test]
    fn empty_project_rebuild_is_stable_source_anchored_and_path_free() {
        let fixture = Fixture::new();
        let first = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let second = fixture.graph.rebuild(&fixture.project_id).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.node_count, 1);
        assert_eq!(first.edge_count, 0);
        assert_eq!(first.source_count, 10);
        assert_eq!(first.present_source_count, 1);
        assert!(first.projection_id.starts_with("grp_"));
        let rendered = serde_json::to_string(&first).unwrap();
        assert!(!rendered.contains(fixture.root.to_string_lossy().as_ref()));
        assert!(!rendered.contains("session"));
        assert!(!rendered.contains("transcript"));
    }

    #[test]
    fn semantic_nodes_are_structurally_connected_to_their_canonical_sources() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "- main_question_or_thesis: How does exposure affect returns?\n\
- contribution_claim: Connect exposure to market response.\n",
        )
        .unwrap();
        let concept = AcademicGraphNodeV1::new(
            &fixture.project_id,
            AcademicGraphNodeType::Concept,
            AcademicGraphIdentityScope::Global,
            "concept:market-response",
            "Market response",
            vec![AcademicGraphLayer::Combined],
            GRAPH_SEMANTIC_LINKS_RELATIVE_PATH,
            "node:concept:market-response",
        )
        .unwrap();
        fixture.write_links(&[node_record(&fixture.project_id, &concept)]);
        fixture.refresh(2);

        let first = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let second = fixture.graph.rebuild(&fixture.project_id).unwrap();
        assert_eq!(first, second);

        let semantic_nodes = first
            .nodes
            .iter()
            .filter(|node| {
                !matches!(
                    node.node_type,
                    AcademicGraphNodeType::Project | AcademicGraphNodeType::Artifact
                )
            })
            .collect::<Vec<_>>();
        assert!(!semantic_nodes.is_empty());
        for node in semantic_nodes {
            let artifact = first
                .nodes
                .iter()
                .find(|candidate| {
                    candidate.node_type == AcademicGraphNodeType::Artifact
                        && candidate.artifact_path == node.artifact_path
                })
                .unwrap();
            let edge = first
                .edges
                .iter()
                .find(|edge| {
                    edge.relation == AcademicGraphRelation::Contains
                        && edge.source_node_id == artifact.node_id
                        && edge.target_node_id == node.node_id
                })
                .unwrap();
            assert_eq!(edge.layers, node.layers);
            assert_eq!(edge.artifact_path, node.artifact_path);
            assert_eq!(edge.source_anchor, node.source_anchor);
            assert_eq!(
                edge.inference_strength,
                AcademicInferenceStrength::DirectEvidence
            );
            assert_eq!(edge.confidence, AcademicGraphConfidence::High);
            assert_eq!(edge.status, AcademicGraphEdgeStatus::Observed);
            assert_eq!(edge.created_from_capture, None);
        }
    }

    #[test]
    fn artifact_resolution_derives_the_exact_registered_path_from_a_bound_entity() {
        let fixture = Fixture::new();
        let snapshot = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let project_node = snapshot
            .nodes
            .iter()
            .find(|node| node.node_type == AcademicGraphNodeType::Project)
            .unwrap();

        let target = fixture
            .graph
            .resolve_artifact(
                &fixture.project_id,
                snapshot.project_revision,
                &snapshot.projection_id,
                AcademicGraphEntityKind::Node,
                &project_node.node_id,
            )
            .unwrap();

        assert_eq!(
            target.path(),
            fixture.project_root.join("context/project_manifest.json")
        );
        assert_eq!(target.artifact_path, "context/project_manifest.json");
        assert_eq!(target.source_anchor, "#/project_id");
        assert!(!format!("{target:?}").contains(fixture.root.to_string_lossy().as_ref()));

        let view = fixture
            .graph
            .read_graph_artifact(
                &fixture.project_id,
                snapshot.project_revision,
                &snapshot.projection_id,
                AcademicGraphEntityKind::Node,
                &project_node.node_id,
                MIN_PROJECT_ARTIFACT_VIEW_BYTES,
            )
            .unwrap();
        assert_eq!(view.format, ProjectArtifactFormat::Json);
        assert!(view.anchor_matched);
        assert!(view.anchor_line.is_some());
        assert!(view.content.contains("\"project_id\""));
    }

    #[test]
    fn registered_artifact_reads_are_revision_bound_anchor_centered_and_path_redacted() {
        let fixture = Fixture::new();
        let mut source = (1..=90)
            .map(|line| format!("line {line:03}: bounded project context\n"))
            .collect::<String>();
        source.push_str("ANCHOR-042: exact scholarly evidence\n");
        source.extend((91..=180).map(|line| format!("line {line:03}: bounded project context\n")));
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            source.as_bytes(),
        )
        .unwrap();
        fixture.refresh(2);
        let snapshot = fixture.graph.rebuild(&fixture.project_id).unwrap();

        let artifact = fixture
            .graph
            .read_registered_artifact(
                &fixture.project_id,
                snapshot.project_revision,
                "context/research_state.md",
                Some("ANCHOR-042"),
                MIN_PROJECT_ARTIFACT_VIEW_BYTES,
            )
            .unwrap();

        assert_eq!(artifact.project_id, fixture.project_id);
        assert_eq!(artifact.project_revision, snapshot.project_revision);
        assert_eq!(artifact.projection_id, None);
        assert_eq!(artifact.entity_kind, None);
        assert_eq!(artifact.entity_id, None);
        assert_eq!(artifact.format, ProjectArtifactFormat::Markdown);
        assert!(artifact.content.contains("ANCHOR-042"));
        assert!(artifact.anchor_matched);
        assert!(artifact.anchor_line.is_some());
        assert!(artifact.truncated_before);
        assert!(artifact.truncated_after);
        assert!(artifact.content_size_bytes <= MIN_PROJECT_ARTIFACT_VIEW_BYTES as u64);
        assert_eq!(
            artifact.content_digest,
            crate::storage::sha256_bytes(source.as_bytes())
        );
        let rendered = serde_json::to_string(&artifact).unwrap();
        assert!(!rendered.contains(fixture.root.to_string_lossy().as_ref()));
        assert!(!rendered.contains(fixture.project_root.to_string_lossy().as_ref()));

        assert_eq!(
            fixture
                .graph
                .read_registered_artifact(
                    &fixture.project_id,
                    snapshot.project_revision - 1,
                    "context/research_state.md",
                    None,
                    MIN_PROJECT_ARTIFACT_VIEW_BYTES,
                )
                .unwrap_err(),
            ProjectError::RevisionConflict
        );
        assert_eq!(
            fixture
                .graph
                .read_registered_artifact(
                    &fixture.project_id,
                    snapshot.project_revision,
                    "notes/private.md",
                    None,
                    MIN_PROJECT_ARTIFACT_VIEW_BYTES,
                )
                .unwrap_err(),
            ProjectError::ProjectArtifactUnsupported
        );
        for invalid_budget in [
            MIN_PROJECT_ARTIFACT_VIEW_BYTES - 1,
            MAX_PROJECT_ARTIFACT_VIEW_BYTES + 1,
        ] {
            assert_eq!(
                fixture
                    .graph
                    .read_registered_artifact(
                        &fixture.project_id,
                        snapshot.project_revision,
                        "context/research_state.md",
                        None,
                        invalid_budget,
                    )
                    .unwrap_err(),
                ProjectError::InvalidGraphQuery
            );
        }
    }

    #[test]
    fn structured_artifact_anchors_resolve_to_bounded_source_lines() {
        let markdown =
            "- main_question_or_thesis: Which exposure changes returns?\n- contribution_claim: C\n";
        assert_eq!(
            locate_project_artifact_anchor(
                markdown,
                ProjectArtifactFormat::Markdown,
                "field:main_question_or_thesis",
            ),
            markdown.find("main_question_or_thesis")
        );
        assert_eq!(
            locate_project_artifact_anchor(markdown, ProjectArtifactFormat::Markdown, "line:2"),
            markdown.find("- contribution_claim")
        );
        assert_eq!(
            locate_project_artifact_anchor(markdown, ProjectArtifactFormat::Csv, "row:2"),
            markdown.find("- contribution_claim")
        );
        assert_eq!(
            locate_project_artifact_anchor(markdown, ProjectArtifactFormat::Csv, "row:0"),
            None
        );
        assert_eq!(
            locate_project_artifact_anchor(markdown, ProjectArtifactFormat::Csv, "row:3"),
            None
        );
    }

    #[test]
    fn graph_artifact_reads_preserve_entity_identity_without_returning_a_host_path() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "Claim C1\n",
        )
        .unwrap();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            "claim_id,source_id\nC1,E1\n",
        )
        .unwrap();
        fixture.refresh(2);
        fixture.write_links(&fixture.semantic_fixture("E1 supports C1.", "link:support-C1"));
        fixture.refresh(3);
        let snapshot = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let edge = snapshot
            .edges
            .iter()
            .find(|edge| edge.relation == AcademicGraphRelation::Supports)
            .unwrap();

        let artifact = fixture
            .graph
            .read_graph_artifact(
                &fixture.project_id,
                snapshot.project_revision,
                &snapshot.projection_id,
                AcademicGraphEntityKind::Edge,
                &edge.edge_id,
                MIN_PROJECT_ARTIFACT_VIEW_BYTES,
            )
            .unwrap();

        assert_eq!(
            artifact.projection_id.as_deref(),
            Some(snapshot.projection_id.as_str())
        );
        assert_eq!(artifact.entity_kind, Some(AcademicGraphEntityKind::Edge));
        assert_eq!(artifact.entity_id.as_deref(), Some(edge.edge_id.as_str()));
        assert_eq!(artifact.artifact_path, GRAPH_SEMANTIC_LINKS_RELATIVE_PATH);
        assert_eq!(artifact.format, ProjectArtifactFormat::JsonLines);
        assert_eq!(artifact.source_anchor.as_deref(), Some("link:support-C1"));
        assert!(artifact.anchor_matched);
        assert!(artifact.content.contains("link:support-C1"));
        assert!(
            !serde_json::to_string(&artifact)
                .unwrap()
                .contains(fixture.root.to_string_lossy().as_ref())
        );

        let csv = fixture
            .graph
            .read_registered_artifact(
                &fixture.project_id,
                snapshot.project_revision,
                "evidence/claim-evidence-ledger.csv",
                Some("C1,E1"),
                MIN_PROJECT_ARTIFACT_VIEW_BYTES,
            )
            .unwrap();
        assert_eq!(csv.format, ProjectArtifactFormat::Csv);
        assert!(csv.anchor_matched);
        assert!(csv.content.contains("C1,E1"));
    }

    #[test]
    fn registered_artifact_reads_reject_non_utf8_content() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            [0xff, 0xfe, 0xfd],
        )
        .unwrap();
        fixture.refresh(2);
        let revision = fixture.projects.snapshot().unwrap().projects[0].semantic_revision;

        assert_eq!(
            fixture
                .graph
                .read_registered_artifact(
                    &fixture.project_id,
                    revision,
                    "context/research_state.md",
                    None,
                    MIN_PROJECT_ARTIFACT_VIEW_BYTES,
                )
                .unwrap_err(),
            ProjectError::ProjectArtifactContentInvalid
        );
    }

    #[test]
    fn registered_artifact_reader_rejects_oversized_source_bytes() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            vec![b'x'; 4 * 1024 * 1024 + 1],
        )
        .unwrap();

        assert_eq!(
            read_academic_graph_artifact(&fixture.project_root, "context/research_state.md")
                .unwrap_err(),
            ProjectError::DocumentTooLarge
        );
    }

    #[test]
    fn artifact_resolution_is_projection_entity_and_kind_bound() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "Claim C1\n",
        )
        .unwrap();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            "claim_id,source_id\nC1,E1\n",
        )
        .unwrap();
        fixture.refresh(2);
        fixture.write_links(&fixture.semantic_fixture("E1 supports C1.", "link:support-C1"));
        fixture.refresh(3);
        let snapshot = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let edge = snapshot
            .edges
            .iter()
            .find(|edge| edge.relation == AcademicGraphRelation::Supports)
            .unwrap();

        let target = fixture
            .graph
            .resolve_artifact(
                &fixture.project_id,
                snapshot.project_revision,
                &snapshot.projection_id,
                AcademicGraphEntityKind::Edge,
                &edge.edge_id,
            )
            .unwrap();
        assert_eq!(
            target.path(),
            fixture
                .project_root
                .join(GRAPH_SEMANTIC_LINKS_RELATIVE_PATH)
        );
        assert_eq!(target.source_anchor, "link:support-C1");

        assert_eq!(
            fixture
                .graph
                .resolve_artifact(
                    &fixture.project_id,
                    snapshot.project_revision,
                    &snapshot.projection_id,
                    AcademicGraphEntityKind::Node,
                    &edge.edge_id,
                )
                .unwrap_err(),
            ProjectError::InvalidGraphQuery
        );
        assert_eq!(
            fixture
                .graph
                .resolve_artifact(
                    &fixture.project_id,
                    snapshot.project_revision,
                    &format!("grp_{}", "0".repeat(64)),
                    AcademicGraphEntityKind::Edge,
                    &edge.edge_id,
                )
                .unwrap_err(),
            ProjectError::RevisionConflict
        );
        assert_eq!(
            fixture
                .graph
                .resolve_artifact(
                    &fixture.project_id,
                    snapshot.project_revision,
                    &snapshot.projection_id,
                    AcademicGraphEntityKind::Edge,
                    &format!("edg_{}", "0".repeat(64)),
                )
                .unwrap_err(),
            ProjectError::GraphEntityNotFound
        );
    }

    #[cfg(unix)]
    #[test]
    fn artifact_resolution_rejects_a_source_replaced_with_a_symbolic_link() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new();
        let artifact = fixture.project_root.join("context/research_state.md");
        fs::write(&artifact, "Research question: RQ1\n").unwrap();
        fixture.refresh(2);
        let snapshot = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let node = snapshot
            .nodes
            .iter()
            .find(|node| node.artifact_path == "context/research_state.md")
            .unwrap();
        let outside = fixture.root.join("outside.md");
        fs::write(&outside, "not a registered project artifact\n").unwrap();
        fs::remove_file(&artifact).unwrap();
        symlink(&outside, &artifact).unwrap();

        assert_eq!(
            fixture
                .graph
                .resolve_artifact(
                    &fixture.project_id,
                    snapshot.project_revision,
                    &snapshot.projection_id,
                    AcademicGraphEntityKind::Node,
                    &node.node_id,
                )
                .unwrap_err(),
            ProjectError::UnsafeProjectRoot
        );
    }

    #[test]
    fn semantic_record_order_does_not_change_projection_identity() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "Claim C1\n",
        )
        .unwrap();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            "claim_id,source_id\nC1,E1\n",
        )
        .unwrap();
        fixture.refresh(2);
        let records = fixture.semantic_fixture("E1 supports C1.", "link:support-C1");
        fixture.write_links(&records);
        fixture.refresh(3);
        let first = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let reversed = records.into_iter().rev().collect::<Vec<_>>();
        fixture.write_links(&reversed);
        fixture.refresh(4);
        let second = fixture.graph.rebuild(&fixture.project_id).unwrap();
        assert_eq!(first, second);
        let first_links = first
            .sources
            .iter()
            .find(|source| source.source_kind == AcademicGraphSourceKind::SemanticLinks)
            .unwrap();
        let second_links = second
            .sources
            .iter()
            .find(|source| source.source_kind == AcademicGraphSourceKind::SemanticLinks)
            .unwrap();
        assert_eq!(first_links.content_digest, second_links.content_digest);
        assert_eq!(first.node_count, 6);
        assert_eq!(first.edge_count, 6);
    }

    #[test]
    fn rationale_changes_projection_but_anchor_changes_edge_identity() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "Claim C1\n",
        )
        .unwrap();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            "claim_id,source_id\nC1,E1\n",
        )
        .unwrap();
        fixture.refresh(2);
        fixture.write_links(&fixture.semantic_fixture("First rationale.", "link:C1"));
        fixture.refresh(3);
        let first = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let first_support = first
            .edges
            .iter()
            .find(|edge| edge.relation == AcademicGraphRelation::Supports)
            .unwrap();
        fixture.write_links(&fixture.semantic_fixture("Refined rationale.", "link:C1"));
        fixture.refresh(4);
        let rationale_changed = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let rationale_support = rationale_changed
            .edges
            .iter()
            .find(|edge| edge.relation == AcademicGraphRelation::Supports)
            .unwrap();
        assert_eq!(first_support.edge_id, rationale_support.edge_id);
        assert_ne!(first.projection_id, rationale_changed.projection_id);

        fixture.write_links(&fixture.semantic_fixture("Refined rationale.", "link:C1-v2"));
        fixture.refresh(5);
        let anchor_changed = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let anchor_support = anchor_changed
            .edges
            .iter()
            .find(|edge| edge.relation == AcademicGraphRelation::Supports)
            .unwrap();
        assert_ne!(rationale_support.edge_id, anchor_support.edge_id);
    }

    #[test]
    fn malformed_duplicate_and_dangling_records_fail_closed() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "Claim C1\n",
        )
        .unwrap();
        fixture.refresh(2);
        let mut records = fixture.semantic_fixture("Rationale.", "link:C1");
        records[0]["unknown"] = json!(true);
        fixture.write_links(&records);
        assert_eq!(
            fixture.graph.rebuild(&fixture.project_id),
            Err(ProjectError::InvalidGraphDocument)
        );

        let records = fixture.semantic_fixture("Rationale.", "link:C1");
        fixture.write_links(&[records[0].clone(), records[0].clone()]);
        assert_eq!(
            fixture.graph.rebuild(&fixture.project_id),
            Err(ProjectError::InvalidGraphDocument)
        );

        fixture.write_links(&[records[2].clone()]);
        fixture.refresh(3);
        assert_eq!(
            fixture.graph.rebuild(&fixture.project_id),
            Err(ProjectError::InvalidGraphDocument)
        );
    }

    #[test]
    fn unsafe_paths_and_unsupported_high_confidence_are_rejected() {
        let fixture = Fixture::new();
        assert_eq!(
            AcademicGraphNodeV1::new(
                &fixture.project_id,
                AcademicGraphNodeType::Claim,
                AcademicGraphIdentityScope::Project,
                "C1",
                "Claim",
                vec![AcademicGraphLayer::Argument],
                "/private/paper.md",
                "claim:C1",
            ),
            Err(ProjectError::InvalidGraphDocument)
        );
        let left = "nod_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let right = "nod_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        assert_eq!(
            AcademicGraphEdgeV1::new(
                &fixture.project_id,
                left,
                AcademicGraphRelation::Supports,
                right,
                vec![AcademicGraphLayer::Argument],
                "Unsupported relation.",
                "context/research_state.md",
                "claim:C1",
                "Evidence is missing.",
                AcademicInferenceStrength::UnsupportedGap,
                AcademicGraphConfidence::High,
                AcademicGraphEdgeStatus::Proposed,
                None,
            ),
            Err(ProjectError::InvalidGraphDocument)
        );
        for anchor in [
            r"C:\Users\alice\paper.md",
            "C:/Users/alice/paper.md",
            "file:/Users/alice/paper.md",
            "file:C:/Users/alice/paper.md",
            "FILE:relative-paper.md",
        ] {
            assert_eq!(
                AcademicGraphNodeV1::new(
                    &fixture.project_id,
                    AcademicGraphNodeType::Claim,
                    AcademicGraphIdentityScope::Project,
                    "C1",
                    "Claim",
                    vec![AcademicGraphLayer::Argument],
                    "context/research_state.md",
                    anchor,
                ),
                Err(ProjectError::InvalidGraphDocument)
            );
        }
        assert!(
            AcademicGraphNodeV1::new(
                &fixture.project_id,
                AcademicGraphNodeType::Claim,
                AcademicGraphIdentityScope::Project,
                "claim:C2",
                "Claim",
                vec![AcademicGraphLayer::Argument],
                "context/research_state.md",
                "C:1",
            )
            .is_ok()
        );
        assert!(
            AcademicGraphNodeV1::new(
                &fixture.project_id,
                AcademicGraphNodeType::Claim,
                AcademicGraphIdentityScope::Project,
                "claim:C3",
                "Claim",
                vec![AcademicGraphLayer::Argument],
                "context/research_state.md",
                "section:caf\u{e9}",
            )
            .is_ok()
        );
        assert_eq!(
            AcademicGraphNodeV1::new(
                &fixture.project_id,
                AcademicGraphNodeType::Claim,
                AcademicGraphIdentityScope::Project,
                "claim:C4",
                "Claim",
                vec![AcademicGraphLayer::Argument],
                "context/research_state.md",
                "section:cafe\u{301}",
            ),
            Err(ProjectError::InvalidGraphDocument)
        );
    }

    #[test]
    fn explicit_cross_project_lineage_can_reference_an_external_project_identity() {
        let fixture = Fixture::new();
        let external_project_id = ProjectId::parse("prj_11111111111111111111111111111111").unwrap();
        let local_project = AcademicGraphNodeV1::new(
            &fixture.project_id,
            AcademicGraphNodeType::Project,
            AcademicGraphIdentityScope::Project,
            fixture.project_id.as_str(),
            "Graph paper",
            vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
            PROJECT_MANIFEST_RELATIVE_PATH,
            "#/project_id",
        )
        .unwrap();
        let external_project = AcademicGraphNodeV1::new(
            &fixture.project_id,
            AcademicGraphNodeType::Project,
            AcademicGraphIdentityScope::Global,
            external_project_id.as_str(),
            "Parent project",
            vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
            GRAPH_SEMANTIC_LINKS_RELATIVE_PATH,
            "lineage:parent-project",
        )
        .unwrap();
        let lineage = AcademicGraphEdgeV1::new(
            &fixture.project_id,
            &local_project.node_id,
            AcademicGraphRelation::ForkedFrom,
            &external_project.node_id,
            vec![AcademicGraphLayer::Portfolio, AcademicGraphLayer::Combined],
            "This project was explicitly reviewed as a fork of the parent project.",
            GRAPH_SEMANTIC_LINKS_RELATIVE_PATH,
            "lineage:parent-project",
            "The external project must be resolved by Portfolio federation.",
            AcademicInferenceStrength::DirectEvidence,
            AcademicGraphConfidence::High,
            AcademicGraphEdgeStatus::Reviewed,
            None,
        )
        .unwrap();
        fixture.write_links(&[
            node_record(&fixture.project_id, &external_project),
            edge_record(&fixture.project_id, &lineage),
        ]);
        fixture.refresh(2);

        let snapshot = fixture.graph.rebuild(&fixture.project_id).unwrap();
        assert!(snapshot.nodes.contains(&external_project));
        assert!(snapshot.edges.contains(&lineage));

        assert_eq!(
            AcademicGraphNodeV1::new(
                &fixture.project_id,
                AcademicGraphNodeType::Project,
                AcademicGraphIdentityScope::Global,
                fixture.project_id.as_str(),
                "Duplicate local identity",
                vec![AcademicGraphLayer::Portfolio],
                GRAPH_SEMANTIC_LINKS_RELATIVE_PATH,
                "lineage:self",
            ),
            Err(ProjectError::InvalidGraphDocument)
        );
    }

    #[test]
    fn manifest_formatting_does_not_change_the_semantic_projection() {
        let fixture = Fixture::new();
        let first = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let manifest_path = fixture.project_root.join(PROJECT_MANIFEST_RELATIVE_PATH);
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let reformatted = fixture.graph.rebuild(&fixture.project_id).unwrap();
        assert_eq!(first, reformatted);

        let mut drifted = manifest;
        drifted["display_name"] = json!("Unregistered label drift");
        fs::write(&manifest_path, serde_json::to_vec(&drifted).unwrap()).unwrap();
        assert_eq!(
            fixture.graph.rebuild(&fixture.project_id),
            Err(ProjectError::RevisionConflict)
        );
    }

    #[test]
    fn semantic_records_are_bound_to_the_registered_project() {
        let fixture = Fixture::new();
        let global_paper = AcademicGraphNodeV1::new(
            &fixture.project_id,
            AcademicGraphNodeType::Paper,
            AcademicGraphIdentityScope::Global,
            "doi:10.1000/example",
            "Example paper",
            vec![AcademicGraphLayer::Literature],
            GRAPH_SEMANTIC_LINKS_RELATIVE_PATH,
            "node:paper",
        )
        .unwrap();
        let mut record = node_record(&fixture.project_id, &global_paper);
        record["project_id"] = json!("prj_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        fixture.write_links(&[record]);
        assert!(matches!(
            fixture.projects.preview_refresh(&fixture.project_id, 2),
            Err(ProjectError::InvalidGraphDocument)
        ));
        assert_eq!(
            fixture.graph.rebuild(&fixture.project_id),
            Err(ProjectError::InvalidGraphDocument)
        );
    }

    #[test]
    fn semantic_drift_blocks_a_revision_bound_projection() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "Unrefreshed academic change\n",
        )
        .unwrap();
        assert_eq!(
            fixture.graph.rebuild(&fixture.project_id),
            Err(ProjectError::RevisionConflict)
        );
    }

    #[test]
    fn portable_round_trip_rebuilds_the_same_semantic_graph() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "Claim C1\n",
        )
        .unwrap();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            "claim_id,source_id\nC1,E1\n",
        )
        .unwrap();
        fixture.refresh(2);
        fixture.write_links(&fixture.semantic_fixture("E1 supports C1.", "link:C1"));
        fixture.refresh(3);
        let original = fixture.graph.rebuild(&fixture.project_id).unwrap();

        let package = fixture.root.join("portable-package");
        let export = fixture
            .projects
            .preview_export(&fixture.project_id, &package)
            .unwrap();
        let exported = fixture
            .projects
            .apply_portable(
                &export,
                &ApprovedProjectMutation::new(export.preview().plan_digest.clone(), true),
                4,
            )
            .unwrap();
        assert!(!exported.index_rebuild_required);

        let import_home = fixture.root.join("import-home");
        fs::create_dir(&import_home).unwrap();
        let import_config = resolve_config_root(
            Some(fixture.root.join("import-config").as_os_str()),
            &import_home,
        )
        .unwrap();
        let imported_projects = ProjectStateService::new(import_config);
        let imported_root = fixture.root.join("imported-paper");
        let import = imported_projects
            .preview_import(&package, &imported_root)
            .unwrap();
        let imported_commit = imported_projects
            .apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                5,
            )
            .unwrap();
        assert!(imported_commit.index_rebuild_required);
        let imported = AcademicGraphService::new(imported_projects)
            .rebuild(&fixture.project_id)
            .unwrap();
        assert_eq!(original, imported);
        assert!(
            imported_root
                .join(GRAPH_SEMANTIC_LINKS_RELATIVE_PATH)
                .is_file()
        );
        assert!(!imported_root.join(".qiongli/graph-index").exists());
    }

    #[test]
    fn canonical_artifacts_project_entities_support_edges_and_repair_diagnostics() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "# Research State\n\n## Current Research Question / Thesis\n\n- main_question_or_thesis: Does event exposure affect abnormal returns?\n- contribution_claim: Connect event exposure to market response.\n",
        )
        .unwrap();
        fs::write(
            fixture.project_root.join("context/decision_log.md"),
            "# Research Decision Log\n\n| Decision ID | Stage | Status | Decision | Rationale |\n|---|---|---|---|---|\n| DEC-101 | A | locked | Keep the event-study boundary | It matches the evidence |\n|  | B | locked | This row needs a stable ID | Missing identity |\n| DEC-102 | B | uncertain | Keep a tentative rival framing | Needs review |\n",
        )
        .unwrap();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            "claim_id,claim_text,claim_type,evidence_type,source_id,source_location,artifact_path,confidence,limitations,status\n\
C1,\"Event exposure affects returns, conditionally\",finding,paper,Smith2024,p. 4,notes/smith.md,high,Single study,supported\n\
C2,The mechanism remains unsupported,interpretation,gap_note,,,context/gap_notes.md,low,No direct evidence,needs_evidence\n\
C3,A partial claim,finding,paper,Jones2025,p. 2,notes/jones.md,medium,Preliminary,partial\n\
C1,Duplicate claim identity,finding,paper,Dup2026,p. 1,notes/dup.md,high,Duplicate,supported\n",
        )
        .unwrap();
        fixture.refresh(2);

        let first = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let second = fixture.graph.rebuild(&fixture.project_id).unwrap();
        assert_eq!(first, second);
        for (node_type, canonical_id) in [
            (
                AcademicGraphNodeType::ResearchQuestion,
                "research-question:current",
            ),
            (AcademicGraphNodeType::Contribution, "contribution:current"),
            (AcademicGraphNodeType::Decision, "DEC-101"),
            (AcademicGraphNodeType::Decision, "DEC-102"),
            (AcademicGraphNodeType::Claim, "C1"),
            (AcademicGraphNodeType::Claim, "C2"),
            (AcademicGraphNodeType::Claim, "C3"),
            (AcademicGraphNodeType::Evidence, "evidence-source:Smith2024"),
        ] {
            assert!(
                first.nodes.iter().any(|node| {
                    node.node_type == node_type && node.canonical_id == canonical_id
                })
            );
        }
        let claim = first
            .nodes
            .iter()
            .find(|node| {
                node.node_type == AcademicGraphNodeType::Claim && node.canonical_id == "C1"
            })
            .unwrap();
        let evidence = first
            .nodes
            .iter()
            .find(|node| {
                node.node_type == AcademicGraphNodeType::Evidence
                    && node.canonical_id == "evidence-source:Smith2024"
            })
            .unwrap();
        assert!(first.edges.iter().any(|edge| {
            edge.relation == AcademicGraphRelation::Supports
                && edge.source_node_id == evidence.node_id
                && edge.target_node_id == claim.node_id
                && edge.status == AcademicGraphEdgeStatus::Reviewed
        }));
        for code in [
            AcademicGraphDiagnosticCode::MissingStableId,
            AcademicGraphDiagnosticCode::AmbiguousRelation,
            AcademicGraphDiagnosticCode::UnsupportedRelation,
            AcademicGraphDiagnosticCode::ConflictingIdentity,
        ] {
            assert!(
                first
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == code)
            );
        }
        assert!(first.diagnostics.windows(2).all(|pair| {
            (
                pair[0].code,
                &pair[0].artifact_path,
                &pair[0].source_anchor,
                &pair[0].related_id,
            ) <= (
                pair[1].code,
                &pair[1].artifact_path,
                &pair[1].source_anchor,
                &pair[1].related_id,
            )
        }));
        let rendered = serde_json::to_string(&first).unwrap();
        assert!(!rendered.contains("notes/smith.md"));
    }

    #[test]
    fn legacy_context_is_bounded_and_malformed_ledgers_become_diagnostics() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "RQ: Does the legacy question remain projectable?\n",
        )
        .unwrap();
        fs::write(
            fixture.project_root.join("context/decision_log.md"),
            "decision_id,stage,decision\nA1,A,\"Keep the legacy scope, with limits\"\n",
        )
        .unwrap();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            "claim_id,claim_text\nC1,\"unclosed\n",
        )
        .unwrap();
        fixture.refresh(2);

        let snapshot = fixture.graph.rebuild(&fixture.project_id).unwrap();
        assert!(snapshot.nodes.iter().any(|node| {
            node.node_type == AcademicGraphNodeType::ResearchQuestion
                && node.canonical_id == "research-question:current"
        }));
        assert!(snapshot.nodes.iter().any(|node| {
            node.node_type == AcademicGraphNodeType::Decision && node.canonical_id == "A1"
        }));
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|node| node.node_type == AcademicGraphNodeType::Claim)
        );
        assert!(snapshot.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == AcademicGraphDiagnosticCode::UnsupportedRelation
                && diagnostic.artifact_path == "evidence/claim-evidence-ledger.csv"
        }));
    }

    #[test]
    fn canonical_artifacts_remain_authoritative_over_conflicting_explicit_nodes() {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            "claim_id,claim_text,claim_type,evidence_type,source_id,source_location,artifact_path,confidence,limitations,status\n\
C1,Canonical claim,finding,paper,Smith2024,p. 4,notes/smith.md,high,Single study,supported\n",
        )
        .unwrap();
        fixture.refresh(2);
        let explicit = AcademicGraphNodeV1::new(
            &fixture.project_id,
            AcademicGraphNodeType::Claim,
            AcademicGraphIdentityScope::Project,
            "C1",
            "Stale explicit label",
            vec![AcademicGraphLayer::Argument],
            "evidence/claim-evidence-ledger.csv",
            "claim:stale-explicit",
        )
        .unwrap();
        fixture.write_links(&[node_record(&fixture.project_id, &explicit)]);
        fixture.refresh(3);

        let snapshot = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let claims = snapshot
            .nodes
            .iter()
            .filter(|node| {
                node.node_type == AcademicGraphNodeType::Claim && node.canonical_id == "C1"
            })
            .collect::<Vec<_>>();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].label, "Canonical claim");
        assert!(snapshot.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == AcademicGraphDiagnosticCode::ConflictingIdentity
                && diagnostic.related_id.as_deref() == Some(claims[0].node_id.as_str())
        }));
    }

    #[test]
    fn stable_workflow_artifacts_project_cross_layer_entities_and_relations() {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.project_root.join("literature")).unwrap();
        fs::create_dir_all(fixture.project_root.join("manuscript")).unwrap();
        fs::create_dir_all(fixture.project_root.join("evidence")).unwrap();
        let ledger = "claim_id,claim_text,claim_type,evidence_type,source_id,source_location,artifact_path,confidence,limitations,status\n\
CLM-001,Exposure is associated with abnormal returns,finding,paper,Smith2024,Table 1,notes/Smith2024.md,medium,One setting,supported\n\
CLM-001,Exposure is associated with abnormal returns,finding,paper,Jones2025,Table 2,notes/Jones2025.md,medium,Observational,supported\n";
        fs::write(
            fixture
                .project_root
                .join("evidence/claim-evidence-ledger.csv"),
            ledger,
        )
        .unwrap();
        fs::write(
            fixture.project_root.join("context/idea_funnel.md"),
            r#"# Academic Idea Funnel

## Candidate Idea Triage
| Idea ID | One-Sentence Idea | Candidate Gap | Triage Decision |
|---|---|---|---|
| IF-001 | Test whether event exposure changes returns | Existing studies disagree on the mechanism | keep |
| IF-002 | Build an unrelated descriptive catalogue | No causal boundary | reject |

## Recommended Research Idea
- recommended_idea_id: IF-001
"#,
        )
        .unwrap();
        fs::write(
            fixture.project_root.join("context/boundary_review.md"),
            r#"# Boundary Review

## Claim Strength And Evidence Threshold
- claim_strength: associative

## One-Question Academic Loop
| Question ID | Recommended Answer | User Or Artifact Answer | Status |
|---|---|---|---|
| BQ-001 | Narrow the population | Use listed firms only | resolved |

## Locked Decision
| Decision ID | Decision | Rationale | Confidence | Evidence Basis |
|---|---|---|---|---|
| BD-001 | Use an associative claim | Identification is incomplete | medium | design/analysis_plan.md |
"#,
        )
        .unwrap();
        fs::write(
            fixture.project_root.join("literature/literature_map.md"),
            r#"# Literature Map

## Included Studies
| Citekey | Primary Cluster ID | Secondary Cluster IDs | Evidence Limit | Source Anchor |
|---|---|---|---|---|
| Smith2024 | LC-001 | LC-002 | Single study | notes/Smith2024.md#findings |

## Concept Streams
| Cluster ID | Cluster Label | Basis | Core Argument | Representative Papers | Evidence Limits |
|---|---|---|---|---|---|
| LC-001 | Exposure mechanisms | mechanism | Exposure changes investor attention | Smith2024 | One setting |
| LC-002 | Market response | outcome | Returns respond conditionally | Smith2024 | Observational evidence |

## Evidence Gaps
| Gap ID | Open Problem | Cluster IDs | Source Anchors | Project Relevance | Status |
|---|---|---|---|---|---|
| GAP-001 | The mechanism remains uncertain | LC-001; LC-002 | notes/Smith2024.md#limitations | Central motivation | open |

## Inter-Cluster Relationships
| Source Cluster ID | Relation | Target Cluster ID | Source Anchor | Evidence Limit | Status |
|---|---|---|---|---|---|
| LC-001 | complementary | LC-002 | notes/Smith2024.md#discussion | Single study | proposed |
"#,
        )
        .unwrap();
        fs::write(
            fixture
                .project_root
                .join("manuscript/claims_evidence_map.md"),
            r#"# Claim-Evidence Map

| Claim ID | Claim | Claim Type | Evidence Pointer | Citation Keys | Manuscript Location | Confidence | Action |
|---|---|---|---|---|---|---|---|
| CLM-001 | Exposure is associated with abnormal returns | finding | analysis/results.csv#model-1 | Smith2024 | Results, paragraph 2 | medium | hedge |
"#,
        )
        .unwrap();
        fixture.refresh(2);

        let first = fixture.graph.rebuild(&fixture.project_id).unwrap();
        let second = fixture.graph.rebuild(&fixture.project_id).unwrap();
        assert_eq!(first, second);
        for (node_type, canonical_id) in [
            (AcademicGraphNodeType::Idea, "IF-001"),
            (AcademicGraphNodeType::Gap, "idea-gap:IF-001"),
            (AcademicGraphNodeType::Decision, "boundary:claim-strength"),
            (AcademicGraphNodeType::Decision, "BQ-001"),
            (AcademicGraphNodeType::Decision, "BD-001"),
            (AcademicGraphNodeType::LiteratureCluster, "LC-001"),
            (AcademicGraphNodeType::LiteratureCluster, "LC-002"),
            (AcademicGraphNodeType::Paper, "citekey:Smith2024"),
            (AcademicGraphNodeType::Gap, "GAP-001"),
            (AcademicGraphNodeType::Claim, "CLM-001"),
        ] {
            assert!(
                first.nodes.iter().any(|node| {
                    node.node_type == node_type && node.canonical_id == canonical_id
                })
            );
        }
        for relation in [
            AcademicGraphRelation::AddressesGap,
            AcademicGraphRelation::BelongsToCluster,
            AcademicGraphRelation::DerivedFrom,
            AcademicGraphRelation::Complements,
            AcademicGraphRelation::Cites,
        ] {
            assert!(first.edges.iter().any(|edge| edge.relation == relation));
        }
        let paper = first
            .nodes
            .iter()
            .find(|node| node.canonical_id == "citekey:Smith2024")
            .unwrap();
        assert!(paper.layers.contains(&AcademicGraphLayer::Literature));
        assert!(paper.layers.contains(&AcademicGraphLayer::Manuscript));
        assert!(paper.layers.contains(&AcademicGraphLayer::Argument));
        let claims = first
            .nodes
            .iter()
            .filter(|node| node.canonical_id == "CLM-001")
            .collect::<Vec<_>>();
        assert_eq!(claims.len(), 1);
        let supports = first
            .edges
            .iter()
            .filter(|edge| edge.relation == AcademicGraphRelation::Supports)
            .collect::<Vec<_>>();
        assert_eq!(supports.len(), 2);
        for edge in supports {
            assert_eq!(edge.target_node_id, claims[0].node_id);
            let offset = locate_project_artifact_anchor(
                ledger,
                ProjectArtifactFormat::Csv,
                &edge.source_anchor,
            )
            .unwrap();
            let source = first
                .nodes
                .iter()
                .find(|node| node.node_id == edge.source_node_id)
                .unwrap();
            assert!(
                ledger[offset..]
                    .lines()
                    .next()
                    .unwrap()
                    .contains(&source.label)
            );
            assert!(first.edges.iter().any(|origin| {
                origin.source_node_id == source.node_id
                    && origin.relation == AcademicGraphRelation::DerivedFrom
                    && first.nodes.iter().any(|node| {
                        node.node_id == origin.target_node_id
                            && node.canonical_id == format!("citekey:{}", source.label)
                    })
            }));
        }
        assert!(!first.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == AcademicGraphDiagnosticCode::ConflictingIdentity
        }));
        let rendered = serde_json::to_string(&first).unwrap();
        assert!(!rendered.contains("notes/Smith2024.md"));
        assert!(!rendered.contains(fixture.root.to_string_lossy().as_ref()));
    }

    #[test]
    fn graph_index_service_rebuilds_from_the_current_projection_without_portable_state() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "- main_question_or_thesis: Which exposure changes returns?\n",
        )
        .unwrap();
        fixture.refresh(2);

        let service = AcademicGraphIndexService::new(fixture.projects.clone());
        let index = service.rebuild(&fixture.project_id).unwrap();
        let query = AcademicGraphQueryV1::new(index.projection_id.clone())
            .with_canonical_id("research-question:current");
        let result = index.query(&query).unwrap();
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(
            result.nodes[0].node_type,
            AcademicGraphNodeType::ResearchQuestion
        );
        assert!(index.index_id.starts_with("gix_"));
        assert!(!fixture.project_root.join(".qiongli/graph-index").exists());
    }

    #[test]
    fn process_restart_rebuilds_the_same_projection_and_index_identities() {
        let fixture = Fixture::new();
        fs::write(
            fixture.project_root.join("context/research_state.md"),
            "- main_question_or_thesis: Which exposure changes returns?\n",
        )
        .unwrap();
        fixture.refresh(2);

        let first = AcademicGraphIndexService::new(fixture.projects.clone())
            .rebuild(&fixture.project_id)
            .unwrap();
        let restarted_config = resolve_config_root(
            Some(fixture.root.join("config").as_os_str()),
            &fixture.root.join("home"),
        )
        .unwrap();
        let restarted_projects = ProjectStateService::new(restarted_config);
        let restarted = AcademicGraphIndexService::new(restarted_projects)
            .rebuild(&fixture.project_id)
            .unwrap();

        assert_eq!(first.projection_id, restarted.projection_id);
        assert_eq!(first.index_id, restarted.index_id);
        assert_eq!(first.project_revision, restarted.project_revision);
        assert_eq!(first.node_count, restarted.node_count);
        assert_eq!(first.edge_count, restarted.edge_count);
        let first_query = first
            .query(&AcademicGraphQueryV1::new(first.projection_id.clone()))
            .unwrap();
        let restarted_query = restarted
            .query(&AcademicGraphQueryV1::new(restarted.projection_id.clone()))
            .unwrap();
        assert_eq!(first_query, restarted_query);
    }

    #[test]
    fn graph_schema_keeps_every_frozen_node_relation_and_layer_variant() {
        let nodes = [
            AcademicGraphNodeType::Project,
            AcademicGraphNodeType::ResearchQuestion,
            AcademicGraphNodeType::Idea,
            AcademicGraphNodeType::Contribution,
            AcademicGraphNodeType::Concept,
            AcademicGraphNodeType::LiteratureCluster,
            AcademicGraphNodeType::Paper,
            AcademicGraphNodeType::Claim,
            AcademicGraphNodeType::Evidence,
            AcademicGraphNodeType::Decision,
            AcademicGraphNodeType::Gap,
            AcademicGraphNodeType::Method,
            AcademicGraphNodeType::ManuscriptSection,
            AcademicGraphNodeType::Artifact,
            AcademicGraphNodeType::Task,
        ];
        let relations = [
            AcademicGraphRelation::Cites,
            AcademicGraphRelation::CitedBy,
            AcademicGraphRelation::Supports,
            AcademicGraphRelation::Weakens,
            AcademicGraphRelation::Contradicts,
            AcademicGraphRelation::Extends,
            AcademicGraphRelation::Defines,
            AcademicGraphRelation::Operationalizes,
            AcademicGraphRelation::UsesMethod,
            AcademicGraphRelation::BelongsToCluster,
            AcademicGraphRelation::Complements,
            AcademicGraphRelation::CompetesWith,
            AcademicGraphRelation::CombinesWith,
            AcademicGraphRelation::Motivates,
            AcademicGraphRelation::Informs,
            AcademicGraphRelation::AddressesGap,
            AcademicGraphRelation::AppearsInSection,
            AcademicGraphRelation::DerivedFrom,
            AcademicGraphRelation::Supersedes,
            AcademicGraphRelation::BoundedBy,
            AcademicGraphRelation::SharesSource,
            AcademicGraphRelation::SharesConcept,
            AcademicGraphRelation::ForkedFrom,
            AcademicGraphRelation::ExtendsProject,
        ];
        assert_eq!(nodes.len(), 15);
        assert_eq!(relations.len(), 24);
        assert_eq!(
            serde_json::to_string(&AcademicInferenceStrength::DirectEvidence).unwrap(),
            "\"direct_evidence\""
        );
        assert_eq!(
            serde_json::to_string(&AcademicGraphRelation::AppearsInSection).unwrap(),
            "\"appears-in-section\""
        );
    }
}
