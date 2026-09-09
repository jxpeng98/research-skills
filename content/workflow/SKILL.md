---
name: qiongli
description: "Qiongli version: v2.0.0-alpha.7. Qiongli version: v2.0.0-alpha.2. Cross-platform academic research workflow for Codex, Claude / Claude Code, and CLI. Use for academic research lifecycle work: paper planning, literature review, paper reading, gap finding, study design, manuscript writing, statistics, analysis code, reproducibility, proofread, rebuttal, submission, presentation, and stage-aware grill / critique. Route natural academic requests even when the user does not explicitly invoke $qiongli or a slash command."
---

# Qiongli Academic Workflow

Run a model-agnostic paper workflow using shared Task IDs and artifact contracts.

This is a **self-contained skill package**. All assets needed for execution — workflows, skill specifications, output templates, standards, and agent roles — are bundled in subdirectories of this package. No external repo access is needed.

Installed Qiongli workflow version: `v2.0.0-alpha.7`

## Quick Start

1. Ask for `paper_type`: `empirical`, `qualitative`, `systematic-review`, `methods`, or `theory`.
2. Ask for `task_id` from the contract (for example `F3` or `G1`).
3. If the user is brainstorming or starting from a vague topic, run the Academic Idea Funnel and write `context/idea_funnel.md` before Stage A outputs.
4. Execute the task and write outputs to `RESEARCH/[topic]/` using the exact file paths.
5. Apply quality gates before submission tasks (`H1`, `H2`).
6. When Full MCP tools are available, call `qiongli_orchestrator_route` for multi-agent, independent-review, handoff, strict-gate, or auditable orchestration work before defaulting to skill-only execution.
7. Follow the returned host-driven sequence: select the registered project and exact revision, run `qiongli_orchestration_doctor`, start the run, execute each bounded handoff in the active host, and return candidates through `qiongli_orchestration_submit`.
8. Check project-local guidance before drafting or reviewing: `.qiongli/guidance_manifest.yaml`, `.qiongli/local_guidance.md`, and `.qiongli/guidance.d/*.md`. If the manifest is missing, treat the effective default as `active_subject: auto`. Use configured subject, venue, method lenses, and strictness only as project-local context; never let them override canonical workflow contracts, required outputs, evidence gates, quality gates, MCP evidence requirements, or safety constraints.

## Cross-Platform Trigger Contract

Qiongli should be considered whenever a request belongs to the academic research lifecycle. This does not require explicit `$qiongli`, `/paper`, `/lit-review`, or slash-command invocation. Natural requests like "read this paper and summarize the contribution", "revise the methods section", "check whether this result supports the claim", "modify this analysis script", or "prepare a rebuttal" should route through the relevant Qiongli stage when the task has scholarly claim, evidence, method, venue, citation, reproducibility, or reviewer-risk consequences.

Use Qiongli for:

- framing a research topic, research question, hypothesis, contribution, theory, or venue fit
- reading papers, PDFs, notes, citations, bibliographies, literature folders, or review matrices
- searching, screening, mapping, extracting, or synthesizing literature
- designing studies, variables, instruments, robustness checks, data plans, preregistration, or ethics materials
- writing or revising proposals, manuscript sections, abstracts, tables, figures, claims maps, or discussion text
- interpreting statistics, effect sizes, models, diagnostics, robustness checks, or analysis outputs
- reading or editing academic analysis code, notebooks, Stata scripts, R scripts, Python scripts, Quarto files, or replication packages
- checking PRISMA, reporting compliance, tone, citation support, originality, submission packages, rebuttals, peer review responses, or presentations

Do not force Qiongli for:

- generic software feature work unrelated to research analysis
- generic prose editing without scholarly claim, evidence, venue, citation, or reviewer risk
- file organization or format conversion without academic interpretation
- project maintenance tasks for the Qiongli repository itself unless the user is changing research workflow behavior

### Ambiguity Trigger

When the user appears unsure, underspecified, or asks for judgment in an academic context, run a light boundary/grill step before producing a final artifact. Trigger phrases include: "I don't know how to start", "not sure", "help me decide", "which direction", "is this reasonable", "what should I do next", "帮我判断", "不知道怎么做", "不确定", "方向不清楚", "帮我想想", and "这样是否合理".

The ambiguity response should inspect available artifacts first, then ask one blocking academic question with a recommended answer and rationale. Do not ask a broad questionnaire.

### Platform Routing

- Codex: skill or plugin discovery should route natural academic requests to Qiongli even without `$qiongli`; then load the relevant workflow or skill card. If the Full MCP server exposes `qiongli_orchestrator_route`, call it before multi-agent or auditable work and follow its host-driven `project_list -> doctor -> start -> read/submit` sequence.
- Claude / Claude Code: skill package and plugin metadata should route natural academic requests to the matching task ID or workflow wrapper. If the Full MCP server exposes `qiongli_orchestrator_route`, call it before multi-agent or auditable work and execute returned handoffs in the active Claude host.
- CLI and portable packages: command wrappers remain available, but the same artifact and handoff contracts apply.
- Portable `qiongli-workflow`: synced skill packages must carry this trigger contract for non-plugin installs.

### Project-Local Guidance

Before skill-only execution, check the current project root for:

- `.qiongli/guidance_manifest.yaml`
- `.qiongli/local_guidance.md`
- `.qiongli/guidance.d/*.md`

If `.qiongli/guidance_manifest.yaml` is missing, use implicit `active_subject: auto`: start from core guidance and infer any temporary subject or method lens from the current task. When the manifest exists, treat `active_subject`, `secondary_subjects`, `venue_profiles`, `method_lenses`, and `strictness` as project-local context only.

Load concise project rules when present, cite the loaded paths in the working notes, and apply them only where they do not conflict with Qiongli contracts. Project-local guidance must never override canonical workflow contracts, required outputs, evidence gates, quality gates, MCP evidence requirements, safety constraints, or the task packet. If local guidance conflicts with any required output or gate, follow the canonical requirement and record the conflict.

### Runtime Subject Refinement

Qiongli installs as an adaptive core workflow. Start from `active_subject: auto`
unless `.qiongli/guidance_manifest.yaml` says otherwise. During a task, infer
whether the request needs core-only guidance, a borrowed method lens, a suggested
subject, a confirmed subject, or a locked subject.

Do not switch the whole project subject from a single method signal. A management
paper that uses an event study borrows finance event-study diagnostics; it is not
automatically a finance paper. A political science paper that uses DID borrows
economics identification diagnostics; it is not automatically an economics paper.

Use `subject_refinement.borrowed_lenses` as temporary method guidance. Use
`subject_refinement.primary_subject` as the temporary subject only when the
decision is `suggest_subject`, `confirm_subject`, or `lock_subject`. Persist
changes only through project-local guidance proposals or an explicit
`subject_mode: confirmed` or `subject_mode: locked` manifest.

### Subject Domain Packs

When Qiongli is installed through the CLI with a subject package, treat the subject-installed domain profile as the specialization layer for the canonical workflow. If `SUBJECT_MANIFEST.json` names `economics`, load `skills/domain-profiles/economics.yaml`; if it names `finance`, load `skills/domain-profiles/finance.yaml`. If project guidance is `active_subject: auto`, infer a temporary economics or finance domain from the task context, then apply the same profile rules.

Domain profiles refine canonical contracts; they do not replace them. For each matched method, apply `canonical_references` as method anchors, `gate_relevance` as Q1-Q4 routing hints, `diagnostic_artifacts` as required local evidence, and `failure_triggers` as blocker language for unsupported claims.

## Workflow Entry Points

Explicit workflow commands are optional entry points. In Codex, users can invoke this skill with `/skills` or `$qiongli`, but natural academic requests should also route here. Claude Code surfaces may expose the same workflows as slash-style command wrappers:

```
/paper [topic] [venue]                # Master router — choose paper type + task ID
/paper-lifecycle [topic]              # Full-Cycle lifecycle preview from topic to journal fit
/coursework [assignment brief, task, or topic] # Coursework, rubric, and learning-assessment support
/dissertation [topic, program, or level]       # Dissertation / thesis / major-project support
/lit-review [topic] [year range]     # Systematic literature review (PRISMA)
/paper-read [URL or DOI]             # Deep paper analysis
/find-gap [research area]            # Identify research gaps
/build-framework [theory/concept]    # Build theoretical framework
/academic-write [section] [topic]    # Academic writing assistance
/synthesize [topic] [outcome_id]     # Evidence synthesis / meta-analysis
/paper-write [topic] [type] [venue]  # Full manuscript drafting
/study-design [topic]                # Empirical study design
/ethics-check [topic]                # Ethics / IRB pack
/submission-prep [topic] [venue]     # Submission package
/rebuttal [topic]                    # Rebuttal / response to reviewers
/code-build [method] --domain ...    # Build academic research code
/proofread [topic]                   # AI de-trace / final proofreading
/academic-present [topic]            # Academic presentation preparation
```

## Bundled Workflows

Full workflow definitions are included in the `workflows/` subdirectory of this skill package. When a user invokes any workflow above (for example, `$qiongli run paper` or `/paper` on clients that support workflow wrappers), read the corresponding file from `workflows/<command-name>.md` for the complete execution instructions.

The `workflows/paper.md` file is the **master router** — it maps every Task ID (A1–K4) to the correct sub-workflow or skill card. Start there for any task-ID-based request.

## Skill Directory Structure

The skill system covers the full research lifecycle across 11 stages:

```
skills/
├── A_framing/       (question-refiner, contribution-crafter, hypothesis-generator, theory-mapper, gap-analyzer, venue-analyzer)
├── B_literature/    (academic-searcher, paper-screener, paper-extractor, citation-snowballer, fulltext-fetcher, citation-formatter, concept-extractor, literature-mapper, reference-manager-bridge)
├── C_design/        (study-designer, rival-hypothesis-designer, robustness-planner, dataset-finder, variable-constructor, data-dictionary-builder, data-management-plan, prereg-writer, variable-operationalizer)
├── D_ethics/        (ethics-irb-helper, statement-generator, deidentification-planner)
├── E_synthesis/     (effect-size-calculator, evidence-synthesizer, quality-assessor, publication-bias-checker, qualitative-coding)
├── F_writing/       (manuscript-architect, proposal-writer, analysis-interpreter, effect-size-interpreter, table-generator, figure-specifier, meta-optimizer, discussion-writer)
├── G_compliance/    (prisma-checker, reporting-checker, tone-normalizer)
├── J_proofread/     (ai-fingerprint-scanner, human-voice-rewriter, similarity-checker, final-proofreader)
├── H_submission/    (submission-packager, rebuttal-assistant, peer-review-simulation, fatal-flaw-detector, journal-fit-recommender, reviewer-empathy-checker, credit-taxonomy-helper, limitation-auditor)
├── I_code/          (code-builder, data-cleaning-planner, data-merge-planner, code-specification, code-planning, code-execution, code-review, reproducibility-auditor, stats-engine)
├── K_presentation/  (presentation-planner, slide-architect, slidev-scholarly-builder, beamer-builder)
├── Z_cross_cutting/ (academic-context-maintainer, metadata-enricher, model-collaborator, self-critique)
└── domain-profiles/ (economics, finance, psychology, biomedical, education, cs-ai, ...)
```

## Output Structure

```
RESEARCH/[topic]/
├── context/                 # Project-level research state, idea funnel, boundary review, and decision log
├── framing/                 # A-stage outputs (RQ, hypothesis, contribution, venue)
├── protocol.md              # Research protocol
├── search_log.md            # Reproducible search records
├── screening/               # Screening logs + PRISMA flow
├── notes/                   # Individual paper notes
├── extraction_table.md      # Data extraction table
├── synthesis.md             # Final synthesis report
├── proposal/                # Research proposal / opening report
├── manuscript/              # Outline, draft, claims map, figures plan
├── proofread/               # AI detection, humanization, similarity, final proofread
├── submission/              # Cover letter, checklist, statements
├── revision/                # Rebuttal + response materials
├── analysis/                # Code + data pipelines
├── presentation/            # Slide deck spec, slides.md / slides.tex
└── bibliography.bib         # BibTeX references
```

## Required Behavior

- Use the canonical task and output definitions in `references/workflow-contract.md`.
- Keep stage labels and task IDs unchanged across models.
- Before unresolved Stage A work, `/paper` routing, or `/find-gap` from a vague topic, run the Academic Idea Funnel and write `RESEARCH/[topic]/context/idea_funnel.md`; then hand off locked or unresolved scholarly boundaries into `RESEARCH/[topic]/context/boundary_review.md`.
- Do not infer stage order alphabetically when the contract exposes explicit ordering metadata.
- When `self-critique` is one of the required skills, preserve critique history across revision rounds, treat `review/self_critique_log.md` as the canonical issue register, require the configured minimum review passes before convergence, and never convert a `BLOCK` verdict into `PASS` because of high confidence alone.
- Writing Harness Contract: when producing or revising Stage F writing through a workflow, skill card, role prompt, or direct agent instruction, first lock the boundary and Story Spine, then write in section or paragraph-cluster chunks. Use a write -> review -> confirm loop for each chunk; do not draft the whole artifact in one uninterrupted pass. Block or ask the next blocking boundary/grill question when there is mainline drift, missing support, generic or vague claims, evidence-free generalization, or a logic jump.
- If a requested output is missing prerequisites, create a gap note and ask whether to:
  1. continue with placeholders, or
  2. run the prerequisite task first.
- Keep claims, methods, and evidence aligned (run integrity checks for stage `G`).
- Track central claims in `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` using `references/evidence-ledger-contract.md`; unsupported central claims become gap notes, not invented citations.
- Before final writing, proofread, submission, rebuttal, or presentation-facing outputs, apply `references/citation-risk-policy.md` when citation support is material.
- At high-risk stage transitions, write `RESEARCH/[topic]/context/stage_handoff.md` using `references/stage-handoff-contract.md`.
- At every major stage close, run `academic-context-maintainer`, keep `main_question_or_thesis` and `contribution_claim` current, and preserve the exact bundled template headings and stable-ID columns (including Decision ID, Idea ID, Cluster ID, Citekey, Gap ID, and Claim ID) so canonical artifacts remain graph-readable.
- For graph-building or repair requests, and at a major stage close that changed graph-bearing artifacts, load `references/academic-graph-continuity.md` and follow its inspect -> preview -> apply -> refresh -> verify sequence.
- After changing a graph-bearing artifact in a registered project, report that an explicit App or CLI project refresh is required before the Academic Graph is current. After that refresh, use `qiongli_project_graph_snapshot` or `qiongli_project_graph_query` when Full MCP exposes them to verify semantic nodes, non-`contains` relations, diagnostics, and the refreshed revision instead of inferring graph state from prose or file presence.
- Use `venue-profiles/` when a target venue profile is available; otherwise create a venue gap note instead of assuming community-specific expectations.
- Use `H5` journal-fit-recommender for manuscript-first reverse journal fit when an existing draft needs ranked venue recommendations; block best-journal claims when manuscript evidence, methods, claim maps, or venue profiles are missing.
- Apply `references/academic-output-rubric.md` whenever producing scholarly prose, synthesis, design, review, or submission artifacts.
- Treat host-orchestration metadata as audit-relevant: preserve the exact project revision, checkpoint generation, document digest, handoff digest, execution mode, candidate role, evidence references, and declared limitations.
- Use `--mcp-strict` and `--skills-strict` for authoritative controller-aware runs; avoid `--skip-validation` for submission-facing, Stage-I code, or final manuscript outputs.
- In solo mode, record role-specific gate intent: Codex-only writing should cover evidence ledger, citation risk, claim calibration, and scholarly voice checks; Claude-only engineering should cover implementation intent, declared write set, failing-test-first discipline, command evidence, and rollback notes; Antigravity-only writing should cover story spine, claim support, and reviewer self-critique checks. Current offline audits hard-block missing claim-map artifacts for Codex-only writing, missing story-spine artifacts for Antigravity-only writing, and missing implementation-intent artifacts for Claude/Antigravity-only code unless solo gates are `off`.
- In Codex-Claude duo mode, record blocking disagreements with a disagreement matrix and resolve them by evidence, method risk, implementation validity, and downstream publication impact.
- When a workflow references `templates/<name>.md`, load the template from the `templates/` subdirectory of this package.

## Literature Provider Configuration

- CLI, Codex, and Claude Code installs can configure external literature providers with `qiongli provider setup` and audit them with `qiongli provider doctor`.
- Marketplace Lite plugin installs include a Rust-built self-contained local Literature Provider MCP runtime. They do not require user-installed Node, Python, npm, pip, Cargo, Rust, or the full Qiongli CLI for provider search tools.
- Full local orchestration is provided by the native Full MCP server. It returns bounded handoffs to Codex or Claude Code and never launches model processes itself; use `qiongli mcp serve --transport stdio --profile full` for doctor, checkpoint, evidence-read, and submission tools.
- Zotero local-library search and direct writes require Zotero Desktop plus the Qiongli Zotero Companion. Without the Companion, use the Lite MCP import-file export tools.
- In bundled MCP installs, do not expect the client MCP settings UI to inject provider keys into the plugin-bundled MCP server. Use `qiongli_config_status` to find the shared provider config path, then use `qiongli_configure_provider` to open a local browser setup page. Use `qiongli_save_provider_config` only for explicit scripted writes.
- Keep provider secrets out of `.mcp.json`, plugin manifests, release ZIPs, and research artifacts. The bundled provider server reads the shared provider config or explicit provider environment variables at runtime.
- In Codex plugin sessions, before declaring `strategy_only` for literature search, literature review, paper screening, citation snowballing, or evidence synthesis, attempt the visible `qiongli_literature_status` MCP tool. If it returns `capability_mode: provider_connected`, proceed with provider-backed literature workflow. If the tool is not visible in the session, state that Qiongli MCP tools are not visible and recommend restarting Codex or installing the explicit standalone MCP fallback with `qiongli install --target codex --parts mcp`. If provider preflight is unavailable or non-provider-connected but platform-native search is usable, write `qiongli_search_plan` with `search_execution_mode: native_only`.
- Literature workflows must create or update `qiongli_search_plan` after the `qiongli_literature_status` preflight and before search execution. The plan records `search_execution_mode` as exactly one of `hybrid_search`, `provider_connected`, `native_only`, or `strategy_only`; it separately records `provider_capability_mode` as `provider_connected` or `strategy_only` to show whether the MCP/provider layer has configured academic provider access.
- Do not use `qiongli_collect_evidence` to judge built-in literature provider configuration. That tool is a filesystem/builtin/external-command evidence adapter; direct provider names such as `openalex` require a separate `RESEARCH_MCP_OPENALEX_CMD`. Use `qiongli_literature_status`, `qiongli_config_status`, `qiongli_test_provider`, and `qiongli_literature_search` to judge OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv provider availability.
- Use `hybrid_search` when provider calls and platform-native search are both available and useful. Use `provider_connected` when the search run is provider-only. Use `native_only` when the active agent has platform native search but no provider-connected MCP. Use `strategy_only` only when neither provider MCP nor platform-native search is available and the workflow can only draft a search strategy or work from supplied corpus.
- MCP servers must not call Codex or Claude native search directly. The active agent executes `native_search_queries` from `qiongli_search_plan`; the MCP provider layer only performs provider calls and returns provider records. Do not hide native search behind a provider adapter.
- Preserve distinct provenance labels in `search_log.md`, `search_results.csv`, and diagnostics: provider records use labels such as `mcp:openalex`, `mcp:semantic_scholar`, `mcp:crossref`, `mcp:pubmed`, and `mcp:arxiv`; platform-native records use `native:codex_web_search` or `native:claude_web_search`; user-supplied files, notes, bibliographies, or pasted citations use `user_corpus`.
- Treat `provider_connected` as the only mode where configured external academic provider credentials are available to the local runtime.
- Treat `strategy_only` as a constrained mode: draft the search strategy or use user-supplied corpus, record the limitation, and do not claim review-grade external provider or native-search coverage.
- Claude Desktop/Web focused ZIPs are skill-only packages kept within the 180-file upload budget. They contain workflows/prompts/templates, store no secrets, and cannot execute OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv API calls by themselves.
- For a manual Desktop install, upload the `qiongli-claude-desktop-skill-*.zip` first, then add a manual MCP install when provider calls or local orchestration are required. The skill ZIP supplies agent instructions, workflows/prompts/templates, and subject overlays; MCP supplies tool calls.
- Desktop/Web users need the Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) or another configured provider MCP before claiming `provider_connected` literature search. The MCPB is the separate local Claude Desktop provider for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv configuration/search. Its primary package uses the Rust Lite MCP executable, not a user-installed Node or Python runtime. arXiv is enabled without credentials. Platform-native search alone is `native_only`, not `provider_connected`; if no provider MCP/MCPB and no platform-native search is available, record the run as `strategy_only`.
- The literature MCPB provides literature MCP tools only. It does not expose project orchestration. To add orchestration, install the native Full MCP server with `qiongli mcp serve --transport stdio --profile full`; the active Codex or Claude host executes each returned handoff and submits a bounded candidate back to Qiongli.

## Skill Loading Strategy

Three-tier loading for token efficiency. All paths are relative to this skill package directory:

1. **Quick lookup (~6KB):** Use `skills-summary.md` — skill names + one-line descriptions per stage. Use this to identify which skill to invoke.
2. **Default reference (~19KB):** Use `skills-core.md` — consolidated process descriptions, templates, and output formats. Use this when executing a skill.
3. **Full specification:** Load `skills/[stage]/[skill-name].md` — detailed edge cases, error recovery, quality bars, and verbose templates. Use this only when the core reference is insufficient.

## Bundled Assets

This package includes the following subdirectories:

| Directory | Contents |
|-----------|----------|
| `workflows/` | 17 workflow definitions (slash commands) |
| `references/` | Stage playbooks + workflow contract |
| `skills/` | 72 detailed skill spec files across 13 stage directories |
| `skills-summary.md` | Quick-reference skill index (~3KB) |
| `skills-core.md` | Consolidated skill reference (~19KB) |
| `templates/` | 50+ output templates for manuscripts, submissions, ethics, evidence, and handoffs |
| `standards/` | Canonical contract YAML + capability map + agent profiles |
| `roles/` | 10 agent role definitions for orchestrator execution |
| `venue-profiles/` | Venue expectation profiles for CHI, ACL, NeurIPS, Nature, JAMA, and AOM |

## References

- Task model + outputs: `references/workflow-contract.md`
- Full-Cycle lifecycle gate contract: `references/full-cycle-workflow-harness.md`
- Platform routing map: `references/platform-routing.md`
- Coverage matrix: `references/coverage-matrix.md`
- Academic output rubric: `references/academic-output-rubric.md`
- Academic Graph continuity: `references/academic-graph-continuity.md`
- Evidence ledger contract: `references/evidence-ledger-contract.md`
- Citation risk policy: `references/citation-risk-policy.md`
- Stage handoff contract: `references/stage-handoff-contract.md`
- Method diagnostic contract: `references/method-diagnostic-contract.md`
- Stage playbooks:
  - `references/stage-A-framing.md` (tasks A1–A5)
  - `references/stage-B-literature.md` (tasks B1–B6)
  - `references/stage-C-design.md` (tasks C1–C5)
  - `references/stage-D-ethics.md` (tasks D1–D3)
  - `references/stage-E-synthesis.md` (tasks E1–E5)
  - `references/stage-F-writing.md` (tasks F1–F6)
  - `references/stage-G-compliance.md` (tasks G1–G4)
  - `references/stage-J-proofread.md` (tasks J1–J4)
  - `references/stage-H-submission.md` (tasks H1–H5)
  - `references/stage-I-code.md` (tasks I1–I9)
  - `references/stage-K-presentation.md` (tasks K1–K4)
