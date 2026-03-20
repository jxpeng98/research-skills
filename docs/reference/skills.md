# Skills Guide

This page is the user-facing map of what the `skills/` layer contains.

It is meant to answer questions such as:

- Which part of the system handles my current research problem?
- What does each stage actually contain?
- Which skills are canonical and auto-routed?
- Which markdown cards are supplemental helpers or mirrors rather than primary routed skills?

::: tip Canonical Source
The canonical routed skill list lives in `skills/registry.yaml`. The tables below summarize that registry for human readers.
:::

## How Users Should Read The Skills Layer

- A **workflow command** such as `/paper` or `/code-build` is the entry UX.
- A **Task ID** such as `B2`, `F3`, or `I6` is the contract-level unit of work.
- A **skill** is the reusable execution behavior that the orchestrator injects behind the scenes through `required_skills` and `required_skill_cards`.

In other words, most users should not manually choose raw markdown skill files one by one.
You usually choose:

1. a workflow entrypoint, or
2. a Task ID via `task-plan` / `task-run`.

Then the system decides which skills to load.

If you need exact runtime flags, use [CLI Reference](/reference/cli).
If you need to understand how agents and skills interact at runtime, use [Agent + Skill Collaboration](/advanced/agent-skill-collaboration).
If you need to modify the system, use [Extend Research Skills](/advanced/extend-research-skills).
If you want scenario-driven routes such as "systematic review", "methods paper", or "rebuttal prep", use [Task Recipes](/guide/task-recipes).

## Important Boundaries

- The current internal skill registry covers stages `A` through `I`, plus `Z_cross_cutting`.
- `J`-level proofread and polishing entrypoints live at the workflow layer today; they are not a separate top-level skill stage in the internal registry.
- Some markdown files under `skills/` are **supplemental cards** or **mirror copies** for the Stage-I code lane. They are documented below, but they are not all separate routed skills.

## Stage Overview

| Stage | Focus | Typical user intent |
|---|---|---|
| `A_framing` | topic framing, questions, theory, gap, venue | "What exactly is my contribution?" |
| `B_literature` | search, screen, extract, cite, map | "What does the literature say, and how do I build a corpus?" |
| `C_design` | design, variables, robustness, datasets | "How should this study be designed and operationalized?" |
| `D_ethics` | IRB, privacy, governance | "What ethics and data-protection materials do I need?" |
| `E_synthesis` | evidence synthesis, quality, bias | "How do I combine and rate evidence?" |
| `F_writing` | manuscript building, tables, figures, results writing | "How do I turn analysis into publishable text?" |
| `G_compliance` | reporting checklists, tone, PRISMA | "Is this compliant and submission-ready?" |
| `H_submission` | submission package, rebuttal, review simulation | "How do I package, defend, and stress-test the paper?" |
| `I_code` | academic code, stats, reproducibility | "How do I implement and verify research code?" |
| `Z_cross_cutting` | metadata, model collaboration, self-critique | "How do I improve quality across stages?" |

## Canonical Skills By Stage

### A. Framing

Use Stage A when you are still defining the research question, contribution, theory anchor, or venue positioning.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `question-refiner` | the topic is still vague or too broad | refined RQs, scope, inclusion/exclusion, search terms |
| `hypothesis-generator` | you need explicit hypotheses or propositions | testable hypotheses with mechanisms and boundaries |
| `theory-mapper` | you need a conceptual model or theory scaffold | theory map or Mermaid conceptual diagram |
| `gap-analyzer` | you need to justify novelty from prior work | prioritized gap analysis |
| `venue-analyzer` | you already know the paper direction and need venue fit | venue-fit memo, constraints, reviewer expectations |

### B. Literature

Use Stage B when you are building or maintaining the literature base for a topic, especially systematic or reproducible reviews.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `academic-searcher` | you need reproducible search design and retrieval | search query plan, search results, search log |
| `paper-screener` | you need inclusion/exclusion decisions | screening log and PRISMA-ready counts |
| `paper-extractor` | you need structured notes from included papers | extraction table and per-paper notes |
| `citation-snowballer` | seed papers are known but coverage is incomplete | forward/backward citation expansion log |
| `fulltext-fetcher` | you have candidate papers but not the full text | full-text status and retrieval record |
| `citation-formatter` | references need to be normalized for writing | bibliography, citekeys, BibTeX-style outputs |
| `concept-extractor` | search concepts and controlled vocabulary need refinement | concept map and Boolean term set |
| `literature-mapper` | you need a taxonomy of streams, mechanisms, or clusters | literature map and open-problem structure |
| `reference-manager-bridge` | you need to exchange references with Zotero or similar tools | RIS/CSLJSON export or synced bibliography |

### C. Design

Use Stage C when the question is already clear and the next problem is design validity, data feasibility, and operationalization.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `study-designer` | you need the main study design package | design spec, analysis plan, instruments, prereg handoff |
| `rival-hypothesis-designer` | you need to stress-test the design against alternatives | rival-explanation matrix |
| `robustness-planner` | the empirical design needs sensitivity or robustness planning | robustness and sensitivity plan |
| `dataset-finder` | data availability is uncertain | dataset feasibility and access plan |
| `variable-constructor` | constructs must be turned into auditable variables | variable specification and coding rules |

### D. Ethics

Use Stage D when the study touches human participants, sensitive data, governance, or data-release constraints.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `ethics-irb-helper` | you need the formal ethics package | IRB-facing materials, consent, recruitment, governance |
| `deidentification-planner` | privacy protection needs a technical plan | deidentification/privacy control plan |

### E. Synthesis

Use Stage E when the evidence base already exists and the task is to combine, rate, or stress-test that evidence.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `evidence-synthesizer` | you need narrative synthesis or meta-analysis | synthesis narrative or pooled evidence result |
| `quality-assessor` | you need risk-of-bias and certainty judgments | quality/risk-of-bias assessment |
| `publication-bias-checker` | you need to test for publication bias | publication-bias report |

### F. Writing

Use Stage F when the main question is turning evidence and analysis into sections, tables, figures, and readable claims.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `manuscript-architect` | you need the overall paper structure or section drafting plan | outline, section plan, draft spine |
| `analysis-interpreter` | you need results text that preserves uncertainty | bounded results narrative |
| `effect-size-interpreter` | coefficients need substantive interpretation | effect-size translation in human terms |
| `table-generator` | results must become paper-ready tables | publication-ready tables |
| `figure-specifier` | you need exact figure logic before plotting | figure specification and code guidance |
| `meta-optimizer` | title, abstract, and keywords need discoverability polish | optimized title/abstract/keywords |

### G. Compliance

Use Stage G when the paper exists and now needs formal checklist coverage, tone cleanup, or reporting verification.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `prisma-checker` | the review follows PRISMA-style evidence reporting | PRISMA completeness report |
| `reporting-checker` | you need CONSORT/STROBE/COREQ/SRQR/TRIPOD-style checking | reporting-guideline coverage report |
| `tone-normalizer` | text is too soft, too fluffy, or too absolute | concise academic tone revision log |

### H. Submission

Use Stage H when the manuscript is near submission or already under review.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `submission-packager` | journal/package artifacts need assembling | cover letter, disclosures, supplements bundle |
| `rebuttal-assistant` | reviewer comments need structured responses | point-by-point rebuttal matrix |
| `peer-review-simulation` | you want independent reviewer-style stress tests | multi-persona review memo |
| `fatal-flaw-detector` | you want a harsh desk-reject risk scan | fatal-flaw analysis |
| `reviewer-empathy-checker` | responses are technically correct but sound defensive | reviewer-response tone check |

### I. Code

Use Stage I for academic code, data workflows, statistical execution, and reproducibility. This lane is stricter than general engineering prompts.

The core strict sequence is:

1. `code-specification`
2. `code-planning`
3. `code-execution`
4. `code-review`
5. `reproducibility-auditor`

That sequence is what `code-build --focus full` is designed to reinforce.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `code-builder` | you need to turn a method into executable research code | implementation package aligned to academic method |
| `data-cleaning-planner` | raw data cleaning must become an auditable procedure | cleaning rules and transformation plan |
| `data-merge-planner` | multiple data sources must be joined safely | merge strategy and provenance controls |
| `code-specification` | coding must start from a low-freedom contract | `code/code_specification.md` |
| `code-planning` | execution needs zero-decision, stepwise planning | `code/plan.md` |
| `code-execution` | planned implementation and profiling must be logged | `code/performance_profile.md` |
| `code-review` | a second pass must audit logic and statistical validity | `code/code_review.md` |
| `reproducibility-auditor` | reruns, seeds, and environment evidence must be checked | `code/reproducibility_audit.md` |
| `stats-engine` | you need modeling and diagnostics rather than general coding | `analysis/stats_report.md` |

### Z. Cross-Cutting

Use Stage Z when the need cuts across stages rather than belonging to one paper section.

| Skill | Use it when | Typical outcome |
|---|---|---|
| `metadata-enricher` | metadata across notes, references, and outputs is inconsistent | normalized DOI/author/venue/year metadata |
| `model-collaborator` | you need Codex, Claude, and Gemini to cross-check or shard work | multi-model execution and review plan |
| `self-critique` | you want structured red-teaming and revision pressure | critique questions and revision loop guidance |

## Supplemental Cards And Mirror Files

Not every markdown file under `skills/` is a primary routed skill.

### Supplemental Manual Cards

These are useful reference or helper cards, but they are not all first-class entries in the current registry:

| File | Role |
|---|---|
| `skills/C_design/data-dictionary-builder.md` | builds a structured data dictionary |
| `skills/C_design/data-management-plan.md` | writes FAIR-style data management plans |
| `skills/C_design/prereg-writer.md` | drafts preregistration materials |
| `skills/C_design/variable-operationalizer.md` | maps constructs to measurable variables |
| `skills/H_submission/credit-taxonomy-helper.md` | prepares CRediT contribution statements |
| `skills/I_code/release-packager.md` | packages code/data/environment for archival release |

### Stage-I Mirror Directories

These mirror the canonical Stage-I cards so prompts can stay close to the execution lane:

- `skills/I_code/build/`
- `skills/I_code/planning/`
- `skills/I_code/run/`
- `skills/I_code/qa/`

Treat the canonical top-level files under `skills/I_code/` as the main reference unless you are editing the implementation details of the Stage-I lane itself.

### Cross-Cutting Alias

`skills/Z_cross_cutting/tone-normalizer.md` is a cross-cutting alias to the canonical compliance-oriented tone normalization behavior at `skills/G_compliance/tone-normalizer.md`.

## Domain Profiles

The base skill system stays generic. Domain specialization is injected at runtime through `skills/domain-profiles/*.yaml`.

Current shipped profiles include:

- `biomedical`
- `cs-ai`
- `ecology-environmental`
- `economics`
- `education`
- `epidemiology`
- `finance`
- `political-science`
- `psychology`

Use domain profiles when:

- the default framing or design logic is too generic
- the code lane needs domain-specific diagnostics
- reporting or venue expectations differ by field

For example, the Stage-I code lane can load field-specific method checks through `--domain`.

## Which Page Should You Use Next?

- Need command syntax: [CLI Reference](/reference/cli)
- Need to understand layer boundaries: [Conventions](/conventions)
- Need to understand runtime cooperation: [Agent + Skill Collaboration](/advanced/agent-skill-collaboration)
- Need to change or add skills: [Extend Research Skills](/advanced/extend-research-skills)
