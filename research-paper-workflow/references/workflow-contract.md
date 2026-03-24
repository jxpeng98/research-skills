# Workflow Contract

Use this contract as the single source of truth for:
- `paper_type`
- `stage`
- `task_id`
- expected outputs under `RESEARCH/[topic]/`

## Paper Types

- `empirical`
- `qualitative`
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
- `J`: Proofread and de-AI
- `H`: Submission and revision
- `I`: Research code support

## Task IDs (Canonical)

| Task ID | Stage | Purpose | Primary Output |
|---|---|---|---|
| `A1` | A | Research question | `framing/research_question.md` |
| `A1_5` | A | Hypothesis generation | `framing/hypothesis.md` |
| `A2` | A | Contribution statement | `framing/contribution_statement.md` |
| `A3` | A | Theoretical framing | `theoretical_framework.md` |
| `A4` | A | Gap analysis | `gap_analysis.md` |
| `A5` | A | Target venue analysis | `framing/venue_analysis.md` |
| `B1` | B | Systematic review | `protocol.md`, `search_strategy.md`, `search_log.md`, `search_results.csv`, `screening/`, `notes/`, `bibliography.bib`, `extraction_table.md`, `quality_table.md`, `synthesis_matrix.md`, `synthesis.md` |
| `B2` | B | Targeted paper reading | `notes/`, `bibliography.bib` |
| `B1_5` | B | Concept/keyword extraction | `literature/concept_extraction.md` |
| `B3` | B | Citation snowballing | `snowball_log.md` |
| `B4` | B | Related work writing | `manuscript/manuscript.md` |
| `B5` | B | Citation management | `bibliography.bib`, `references.ris`, `references.json` |
| `B6` | B | Literature mapping | `literature/literature_map.md` |
| `C1` | C | Study design | `study_design.md` |
| `C1_5` | C | Rival hypothesis design | `design/rival_hypotheses.md` |
| `C2` | C | Instruments | `instruments/` |
| `C3` | C | Analysis plan | `analysis_plan.md`, `design/variable_spec.md` |
| `C3_5` | C | Robustness check plan | `design/robustness_plan.md` |
| `C4` | C | Data management plan | `data_management_plan.md`, `design/dataset_plan.md` |
| `C5` | C | Preregistration draft | `preregistration.md` |
| `D1` | D | Ethics pack | `ethics_irb.md` |
| `D2` | D | Ethics statements | `manuscript/manuscript.md` |
| `D3` | D | Participant de-identification plan | `compliance/deidentification_plan.md` |
| `E1` | E | Synthesis strategy | `meta_analysis_plan.md` |
| `E2` | E | Effect size table | `effect_size_table.md` |
| `E3` | E | Meta-analysis write-up | `meta_analysis_results.md` |
| `E3_5` | E | Publication bias assessment | `synthesis/publication_bias.md` |
| `E4` | E | Certainty grading | `grade_sof.md` |
| `E5` | E | Integrated synthesis | `synthesis.md` |
| `F1` | F | Manuscript outline | `manuscript/outline.md` |
| `F2` | F | Single section writing | `manuscript/manuscript.md` |
| `F3` | F | Full draft | `manuscript/manuscript.md`, `manuscript/results_interpretation.md`, `manuscript/effect_interpretation.md` |
| `F4` | F | Claim-evidence map | `manuscript/claims_evidence_map.md` |
| `F5` | F | Figures/tables plan | `manuscript/figures_tables_plan.md`, `manuscript/tables/`, `manuscript/figures/` |
| `F6` | F | Abstract & Title Optimization | `manuscript/meta_optimization.md` |
| `G1` | G | Reporting completeness | `reporting_checklist.md` |
| `G2` | G | PRISMA compliance | `prisma_checklist.md` |
| `G3` | G | Cross-section integrity | `manuscript/claims_evidence_map.md` |
| `G4` | G | Tone & Style Normalization | `compliance/tone_normalization.md` |
| `H1` | H | Submission package | `submission/cover_letter.md`, `submission/submission_checklist.md`, `submission/title_page.md`, `submission/highlights.md`, `submission/suggested_reviewers.md`, `submission/author_contributions_credit.md`, `submission/funding_statement.md`, `submission/coi_statement.md`, `submission/data_availability.md`, `submission/ai_disclosure.md`, `submission/supplementary_inventory.md` |
| `H2` | H | Rebuttal package | `revision/` |
| `H2_5` | H | Reviewer empathy check | `revision/reviewer_empathy_check.md` |
| `H3` | H | Peer review simulation | `revision/peer_review_simulation.md` |
| `H4` | H | Fatal flaw analysis | `revision/fatal_flaw_analysis.md` |
| `I1` | I | Method implementation | `analysis/` |
| `I2` | I | Reproduction | `analysis/` |
| `I3` | I | Data pipeline | `analysis/`, `data/cleaning_plan.md`, `data/merge_plan.md` |
| `I4` | I | Code reproducibility audit | `code/reproducibility_audit.md` |
| `I5` | I | Code specification | `code/code_specification.md` |
| `I6` | I | Code planning | `code/plan.md` |
| `I7` | I | Code execution | `code/performance_profile.md` |
| `I8` | I | Code review | `code/code_review.md` |
| `J1` | J | AI fingerprint scan | `proofread/ai_detection_report.md` |
| `J2` | J | Human-voice rewrite | `proofread/humanized_manuscript.md` |
| `J3` | J | Similarity & originality check | `proofread/similarity_report.md` |
| `J4` | J | Final proofread | `proofread/proofread_checklist.md` |

## Quality Gates

- `Q1`: RQ-method alignment
- `Q2`: Claim-evidence traceability
- `Q3`: Reporting checklist complete
- `Q4`: Reproducibility baseline documented

## Stage Guides

Use these when you need more operational detail than the task table provides:

- `references/stage-A-framing.md`
- `references/stage-B-literature.md`
- `references/stage-C-design.md`
- `references/stage-D-ethics.md`
- `references/stage-E-synthesis.md`
- `references/stage-F-writing.md`
- `references/stage-G-compliance.md`
- `references/stage-J-proofread.md`
- `references/stage-H-submission.md`
- `references/stage-I-code.md`
