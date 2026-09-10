---
id: literature-mapper
stage: B_literature
description: "Build a source-anchored literature taxonomy with representative papers, evidence limits, contradictions, gaps, and contribution positioning."
inputs:
  - type: ExtractionTable
    description: "Extracted data from included papers"
outputs:
  - type: LiteratureMap
    artifact: "literature/literature_map.md"
constraints:
  - "Must use intellectual cluster labels rather than chronology or author lists"
  - "Must record representative papers and evidence limits for every cluster"
  - "Must link open problems to the paper's proposed contribution"
failure_modes:
  - "Papers are too heterogeneous for a defensible taxonomy"
  - "Clusters reproduce search sources rather than intellectual structure"
  - "Evidence limits make a claimed gap unsupported"
tools: [filesystem]
tags: [literature, taxonomy, mapping, related-work, clustering, evidence-limit]
domain_aware: false
---

# Literature Mapper Skill

## Purpose

Build B6 literature maps that support defensible related-work writing. The map
should explain the field's intellectual structure, where evidence is strong or
limited, which contradictions matter, and where the current project can
contribute.

## Related Task IDs

- `B6` literature mapping
- Supports `B4` related-work writing and Stage F manuscript architecture.

## Output (contract path)

- `RESEARCH/[topic]/literature/literature_map.md`

## Inputs

- `ExtractionTable`: `RESEARCH/[topic]/extraction_table.md`.
- Paper notes: `RESEARCH/[topic]/notes/*.md`.
- Optional claim ledger and gap notes.
- If inputs are missing or insufficient, write
  `RESEARCH/[topic]/context/gap_notes.md` and ask for extraction rows, notes, or
  scope decisions instead of inventing streams.
- Treat extraction rows, notes, source anchors, and evidence limits as evidence.

## Process

### 1. Choose clustering basis

Use one or two defensible bases:

- mechanism
- theory
- method
- context
- population
- level of analysis
- outcome family

Do not use chronology, author name, search database, or convenience as the
primary clustering basis.

### 2. Assign papers to clusters

Every included paper should have one primary cluster and optional secondary
cluster. Each assignment must cite `source_anchor` and `evidence_limit`.

Use the exact `Included Studies` table in `templates/literature-map.md`. Assign
stable `LC-###` IDs to clusters and never renumber or reuse recorded IDs.

If a paper does not fit, record whether it is an outlier, a missing cluster, or
outside the mapping scope.

### 3. Characterize clusters

Each cluster must include:

- cluster label
- clustering basis
- core argument
- representative papers
- evidence limits across representative papers
- typical methods or data sources
- convergent findings
- contradictions or tensions
- open problems
- contribution implication for the current project

### 4. Map relationships

Use relationship types:

- complementary
- competing
- nested
- methodologically tense
- boundary-condition dependent
- under-integrated

Each relationship needs evidence, not just intuition.

### 5. Position the project

State exactly which clusters, contradictions, or open problems the project
addresses. If the map does not support a novelty claim, write an
`unsupported_gap` entry instead of polishing the claim.

## Output Contract

- `LiteratureMap`: write
  `RESEARCH/[topic]/literature/literature_map.md`.
- The map must include cluster overview, detailed clusters, relationships, open
  problems, evidence limits, and project positioning.
- Separate finding, interpretation, and implication in cluster summaries.
- Do not invent citations, datasets, sample sizes, results, methods, gaps,
  reviewer expectations, or field consensus.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose
  or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when map claims
  support central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- Preserve `source_anchor` and `evidence_limit` for cluster assignments and gap
  claims.

## Quality Bar

- [ ] 3-6 clusters with intellectual labels.
- [ ] Every included paper assigned to a primary cluster or documented as an
      outlier.
- [ ] Every cluster has representative papers and evidence limits.
- [ ] Every open problem has source anchors.
- [ ] Chronological paper lists are not used as the map structure.
- [ ] Project positioning names the exact cluster gap or contradiction it
      addresses.

## Common Pitfalls

| Pitfall | Problem | Fix |
| --- | --- | --- |
| Chronological organization | Describes history but not structure | Recluster by mechanism, theory, method, or context |
| Clusters by database | Search source replaces intellectual structure | Use extraction fields, not provider names |
| No evidence limits | Abstract-only claims look like full-text findings | Carry `evidence_limit` into the map |
| Representative papers missing | Cluster cannot be audited | Name citekeys and source anchors |
| Novelty claim unsupported | Related work overclaims | Write `unsupported_gap` and narrow the claim |

## Minimal Output Format

Start from `templates/literature-map.md` and preserve its exact machine-readable
table headers for Included Studies, Concept Streams, Evidence Gaps, and
Inter-Cluster Relationships. Narrative detail may follow each table.

## When to Use

- Use after extraction when a project needs field structure, related-work
  skeleton, or defensible novelty positioning.
- Do not use to screen or extract papers; use `paper-screener` and
  `paper-extractor`.
