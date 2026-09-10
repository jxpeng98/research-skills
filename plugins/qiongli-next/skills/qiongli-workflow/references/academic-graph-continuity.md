# Academic Graph Continuity

Use this reference when a user asks to build, connect, repair, or verify an
Academic Graph, and whenever a major stage close changes graph-bearing
artifacts.

## What counts as a connected graph

- `project` and `artifact` nodes are structural inventory, not academic ideas.
- `contains` records file membership. Only a non-`contains` relation expresses
  scholarly topology.
- Readiness therefore needs both `semanticNodeCount > 0` and at least one
  reviewed, source-bound non-`contains` relation. File presence alone is not
  graph continuity.

Prefer the relations already derived from canonical artifacts:

| Canonical artifact | Graph contribution |
|---|---|
| `context/research_state.md` | current question and contribution |
| `context/idea_funnel.md` | stable ideas and candidate gaps |
| `literature/literature_map.md` | papers, clusters, gaps, and reviewed cluster relations |
| `evidence/claim-evidence-ledger.csv` | evidence supporting claims |
| `manuscript/claims_evidence_map.md` | claims citing papers |
| `context/decision_log.md`, `context/boundary_review.md` | decisions and bounded research choices |

`graph/semantic_links.jsonl` is an advanced portable interchange for explicit
records. Do not hand-author it or guess its hashed node and edge IDs when a
canonical artifact can express the same reviewed relationship.

## Safe repair sequence

### 1. Inspect the exact revision

For a registered project, inspect before writing:

```bash
qiongli project graph doctor --project-id <prj_id>
qiongli project graph snapshot --project-id <prj_id>
```

When Full MCP is available, `qiongli_project_graph_snapshot` and
`qiongli_project_graph_query` provide the equivalent revision-bound view.
Record the project revision, readiness state, `semanticNodeCount`, relation
counts, missing/invalid/unsupported sources, and diagnostics.

### 2. Select only reviewed records

Use the smallest canonical artifact that owns the intended relationship.
Reuse every valid stable ID. Create a new stable ID only for a genuinely new
academic record, following the bundled template's exact field or column.

Never invent a citation, evidence source, support direction, confidence,
decision, or relationship. If the available material does not establish a
relation, keep it as an explicit gap and report what evidence is missing.

### 3. Plan a non-destructive normalization

Preserve existing narrative prose and valid structured records. If a legacy
file lacks a supported structure, append or update only the minimum canonical
section. Create a missing canonical file from its bundled template; do not
replace an existing document merely to make it parseable.

Preview the exact files, records, stable IDs, and source anchors before apply.
The preview must distinguish `create`, `append`, and `update`, and name the
semantic relation expected from each reviewed record.

### 4. Apply, refresh, and rebuild

Apply only the previewed changes. In the App, use **Run in client**, refresh the
project revision, and rebuild Academic Graph. With the CLI, preview and apply
the project refresh, then rebuild through snapshot or doctor:

```bash
qiongli project refresh preview --project-id <prj_id>
qiongli project refresh apply --project-id <prj_id> \
  --expected-plan-digest <sha256> --approve-filesystem-write
qiongli project graph snapshot --project-id <prj_id>
qiongli project graph doctor --project-id <prj_id>
```

### 5. Verify the result

Read the refreshed revision rather than inferring success from edited prose.
Report:

- the exact project revision and projection ID;
- `semanticNodeCount`;
- non-`contains` relation count and relation types;
- remaining diagnostics and unsupported gaps;
- the final readiness state.

Do not call the graph connected when it contains only structural nodes or
`contains` edges. If diagnostics remain, identify the next minimum canonical
repair instead of silently broadening or fabricating the research record.
