# Workflow Contract

Use this contract as the single source of truth for:
- `paper_type`
- `stage`
- `task_id`
- expected outputs under `RESEARCH/[topic]/`

## Paper Types

- `empirical`
- `systematic-review`
- `methods`
- `theory`

## Stages

- `A`: Framing and positioning
- `B`: Literature and related work
- `C`: Study design and analysis planning
- `D`: Ethics and IRB
- `E`: Evidence synthesis
- `F`: Manuscript writing
- `G`: Polishing and compliance
- `H`: Submission and revision
- `I`: Research code support

## Task IDs (Canonical)

| Task ID | Stage | Purpose | Primary Output |
|---|---|---|---|
| `A1` | A | Research question | `framing/research_question.md` |
| `A2` | A | Contribution statement | `framing/contribution_statement.md` |
| `A3` | A | Theoretical framing | `theoretical_framework.md` |
| `A4` | A | Gap analysis | `gap_analysis.md` |
| `B1` | B | Systematic review | `search_log.md`, `screening/`, `synthesis.md` |
| `B2` | B | Targeted paper reading | `notes/`, `bibliography.bib` |
| `B3` | B | Citation snowballing | `snowball_log.md` |
| `B4` | B | Related work writing | `manuscript/manuscript.md` |
| `B5` | B | Citation management | `bibliography.bib` |
| `C1` | C | Study design | `study_design.md` |
| `C2` | C | Instruments | `instruments/` |
| `C3` | C | Analysis plan | `analysis_plan.md` |
| `C4` | C | Data management plan | `data_management_plan.md` |
| `C5` | C | Preregistration draft | `preregistration.md` |
| `D1` | D | Ethics pack | `ethics_irb.md` |
| `D2` | D | Ethics statements | `manuscript/manuscript.md` |
| `E1` | E | Synthesis strategy | `meta_analysis_plan.md` |
| `E2` | E | Effect size table | `effect_size_table.md` |
| `E3` | E | Meta-analysis write-up | `meta_analysis_results.md` |
| `E4` | E | Certainty grading | `grade_sof.md` |
| `E5` | E | Integrated synthesis | `synthesis.md` |
| `F1` | F | Manuscript outline | `manuscript/outline.md` |
| `F2` | F | Single section writing | `manuscript/manuscript.md` |
| `F3` | F | Full draft | `manuscript/manuscript.md` |
| `F4` | F | Claim-evidence map | `manuscript/claims_evidence_map.md` |
| `F5` | F | Figures/tables plan | `manuscript/figures_tables_plan.md` |
| `G1` | G | Reporting completeness | `reporting_checklist.md` |
| `G2` | G | PRISMA compliance | `prisma_checklist.md` |
| `G3` | G | Cross-section integrity | `manuscript/claims_evidence_map.md` |
| `H1` | H | Submission package | `submission/` |
| `H2` | H | Rebuttal package | `revision/` |
| `I1` | I | Method implementation | `analysis/` |
| `I2` | I | Reproduction | `analysis/` |
| `I3` | I | Data pipeline | `analysis/` |

## Quality Gates

- `Q1`: RQ-method alignment
- `Q2`: Claim-evidence traceability
- `Q3`: Reporting checklist complete
- `Q4`: Reproducibility baseline documented

