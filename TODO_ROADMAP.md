# Optimization Roadmap TODO

> Last reorganized: 2026-03-25
> This file now tracks only three things:
> 1. verified completed milestones
> 2. active remaining work
> 3. deferred future bets
>
> Historical scoring snapshots, stale availability notes, and duplicated phase writeups have been removed from the main body.

---

## Snapshot

- Verified completed workstreams: 11
- Verified-but-not-fully-accepted items: 4
- Active TODOs: 35
- Deferred future bets: 5

---

## Verification Basis

Checked against current repository structure on 2026-03-25.

Verified from source:
- `standards/research-workflow-contract.yaml`
- `standards/mcp-agent-capability-map.yaml`
- `bridges/orchestrator.py`
- `bridges/codex_bridge.py`
- `bridges/claude_bridge.py`
- `bridges/gemini_bridge.py`
- `research_skills/skill_docs.py`
- `research_skills/cli.py`
- `scripts/release_ready.sh`
- `scripts/bump-version.sh`
- `scripts/release_preflight.sh`
- `scripts/run_literature_smoke.sh`
- `scripts/generate_release_notes.sh`
- `scripts/install_research_skill.sh`
- `scripts/sync_versions.py`
- `docs/conventions.md`
- `guides/advanced/extend-research-skills.md`
- `research-paper-workflow/SKILL.md`
- `tests/test_orchestrator_workflows.py`
- `tests/test_literature_pipeline_integration.py`
- `tests/test_skill_doc_generation.py`
- `tests/test_sync_versions.py`

Key facts confirmed from current repo:
- `platform_mapping.claude_code` already covers the full current task set in `task_catalog`
- `sequence_index` already exists in the canonical stage contract
- functional-agent routing is already present in capability map, pipelines, validator, and orchestrator
- `runtime_options` already flow into Codex / Claude / Gemini bridge construction
- `team-run` already exists as a distinct fanout/fanin execution mode
- `--domain` runtime injection already exists
- native reference MCP scripts already exist for `scholarly-search`, `citation-graph`, `metadata-registry`, and `fulltext-retrieval`
- release preparation is already unified through `scripts/release_ready.sh`
- `skills/registry.yaml` is now the single canonical skill version source
- the literature pipeline now has a dedicated integration smoke wired into the main smoke entrypoint

---

## Verified Completed Milestones

### 1. Architecture and Contract Foundation

- [x] Clarified `portable skill package` vs `repo-internal skill spec`
- [x] Introduced explicit functional-agent ownership alongside runtime-agent routing
- [x] Added explicit stage ordering via `sequence_index`
- [x] Aligned capability-map outputs with contract artifact paths
- [x] Expanded `platform_mapping` to full task coverage
- [x] Upgraded validator and docs generation around the new structure

### 2. Empirical / Interpretation Lifecycle

- [x] Added dataset / variable / cleaning / merge planning capabilities
- [x] Added `analysis-interpreter` and `ResultInterpretation` artifacts
- [x] Expanded `empirical-study.yaml` with explicit data-prep and interpretation nodes
- [x] Expanded `code-first-methods.yaml` to bridge code execution into interpretation and writing
- [x] Added academic code review ownership for `I8`

### 3. Multi-Agent Execution MVP

- [x] Added `team-run` as a distinct mode instead of overloading `parallel`
- [x] Added shard directory rules and worker failure semantics to the contract
- [x] Added planner / fanout / merge / failure-path tests for `team-run`
- [x] Added MVP task support for `B1` and `H3`
- [x] Preserved backward compatibility for existing `parallel` and `task-run`

### 4. Qualitative and Domain Support

- [x] Strengthened analytical-depth requirements in the writing layer
- [x] Added `qualitative` as a first-class paper type
- [x] Added `pipelines/qualitative-study.yaml`
- [x] Added `business-management` qualitative domain support
- [x] Added runtime domain injection through `--domain`

### 5. Chinese User Layer

- [x] Added `summary_zh`, `display_name_zh`, and `when_to_use_zh` into the skill registry
- [x] Switched skill reference pages to generated zh/en documentation
- [x] Updated orchestrator skill-card rendering to prefer localized metadata in Chinese mode

### 6. Release Automation

- [x] Unified version sync across package metadata, registry, and workflow `VERSION`
- [x] Added publish-ready local entrypoint via `scripts/release_ready.sh`
- [x] Updated release notes template, runbook, and publish docs to the new flow
- [x] Added tests for version normalization / sync behavior

### 7. Skill Metadata Simplification

- [x] Removed deprecated `version` from all `skills/*.md` frontmatter
- [x] Made `skills/registry.yaml` the only canonical skill version source
- [x] Updated validator to reject reintroduction of skill frontmatter `version`
- [x] Updated release/publish docs to reflect registry-only skill version sync

### 8. Literature MCP Baseline and Smoke

- [x] Added builtin `metadata-registry` reference provider with optional external enrichment overlay
- [x] Added builtin `fulltext-retrieval` planning stub for `retrieval_manifest.csv` and `screening/full_text.md`
- [x] Unified the four-layer literature contract across standards, Stage B docs, and user-facing setup guides
- [x] Added an end-to-end builtin literature integration smoke
- [x] Wired literature smoke into the main beta/release smoke entrypoint

### 9. Cross-Platform Workflow Consistency

- [x] Removed non-standard `argument-hint` from all workflow frontmatter (Gemini compatibility)
- [x] Enhanced `.gemini/research-skills.md` with task-ID routing, skill loading strategy, and architecture references
- [x] Added `validate_cross_platform_consistency()` CI guard rail to catch future regressions
- [x] Verified all 14 workflows discoverable across Claude, Codex, and Gemini

### 10. Skill Corpus Deep Enrichment (Tier 1–3)

- [x] Enriched 17 Tier 2 skills to Tier 1 standard (Round 1)
- [x] Deepened 8 existing Tier 1 skills with section templates, decision matrices, and quality bars (Round 2)
- [x] Rewrote 9 remaining Tier 3 stubs to full Tier 1 standard (Round 3)
- [x] Total skill corpus: 14,141 lines across 54 registered skills
- [x] All skills now contain: process substeps, output templates, quality bars, pitfalls, academic methodology references

### 11. Stage K_presentation — Academic Presentation

- [x] Created `skills/K_presentation/` with 4 skill files: `presentation-planner`, `slide-architect`, `slidev-scholarly-builder`, `beamer-builder`
- [x] Registered all 4 in `skills/registry.yaml`
- [x] Created `.agent/workflows/academic-present.md` slash-command workflow
- [x] Integrated `slidev-theme-scholarly` layouts, components, and presets
- [x] Updated README.md, README_CN.md, and `.gemini/research-skills.md`

---

## Verified but Not Yet Fully Accepted

- [ ] `team-run` still needs real-agent acceptance evidence for `B1`
- [ ] `team-run` still needs real-agent acceptance evidence for `H3`
- [ ] `task-run` still does not execute a distinct post-run filesystem `validator-gate`
- [ ] `skills/Z_cross_cutting/model-collaborator.md` is still code-first rather than research-wide

---

## Active TODO List

### P0 Blocking

- [ ] Create Stage J skill directory and skill cards
  - create `skills/J_proofread/` with 4 skill cards: `ai-fingerprint-scanner.md` (J1), `human-voice-rewriter.md` (J2), `similarity-checker.md` (J3), `final-proofreader.md` (J4)
  - register all 4 in `skills/registry.yaml`
  - add `"J_proofread"` to `EXPECTED_SKILL_STAGES` in the validator
  - these are the only task IDs (J1–J4) with zero backing skill files

- [ ] Implement a real `validator-gate` in `task-run`
  - add a dedicated post-run validation step
  - verify `required_outputs` under `RESEARCH/[topic]/`
  - surface explicit `validation_status` and missing artifact list
  - add `--skip-validation` with warning output

- [ ] Complete real-agent acceptance for `team-run`
  - run one real acceptance path for `B1`
  - run one real acceptance path for `H3`
  - record receipts plus degrade/block observations

### P1 Near-Term

- [ ] Replace regex-style YAML parsing in `bridges/orchestrator.py`
  - move contract/capability-map parsing to a safe structured parser
  - emit clear parse failures instead of silent extraction drift

- [ ] Finish collaboration-layer docs for `team-run`
  - update `docs/advanced/agent-skill-collaboration.md`
  - update `docs/zh/advanced/agent-skill-collaboration.md`
  - keep `guides/advanced/*` mirrors aligned

- [ ] Generalize `model-collaborator` from code-only to research-wide collaboration language

- [ ] Collapse duplicate / alias skill layouts into one canonical-file rule
  - clean up `skills/I_code/build/`, `skills/I_code/planning/`, `skills/I_code/run/`, and `skills/I_code/qa/`
  - keep only canonical skill paths in `skills/registry.yaml`
  - add validator checks so registry entries cannot point at alias/stub copies

- [ ] Make alias semantics explicit in skill metadata
  - add `canonical`, `alias_of`, and/or `deprecated` fields to `skills/registry.yaml`
  - stop relying on prose-only notes like "canonical version is ..."
  - make docs generation and orchestrator rendering understand aliases directly

- [ ] Standardize a strict skill file skeleton
  - freeze one canonical frontmatter shape for repo-internal skills
  - require a shared body outline such as `Purpose`, `When To Use`, `Process`, `Outputs`, `Quality Bar`
  - add validator checks for missing required sections

- [ ] Move more user-facing skill metadata out of markdown prose and into the registry
  - evaluate adding `display_name`, `when_to_use`, `canonical/alias`, and `deprecated` fields
  - keep markdown focused on execution guidance rather than duplicated metadata
  - keep generated docs and orchestrator skill cards registry-driven

- [ ] Improve install/upgrade ergonomics
  - add part-level control (`--parts` or equivalent) to `scripts/install_research_skill.sh`
  - pass the same control through `rsk upgrade`
  - add `rsk doctor`
  - add `rsk init`

- [ ] Add `statement-generator` skill for Stage D2
  - generate ethics, data availability, CRediT, funding, COI, and AI disclosure statements from structured inputs
  - currently D2 is handled ad hoc by manuscript-architect with no dedicated guidance

- [ ] Add `effect-size-calculator` skill for Stage E2
  - standardized extraction: Cohen's d/g, OR/RR conversions, extraction from p-values/F-stats/medians
  - currently E2 reuses the broad evidence-synthesizer without dedicated conversion guidance

- [ ] Add `/compliance-check` workflow for standalone Stage G access
  - G1 (reporting checklist), G2 (PRISMA), G3 (cross-section), G4 (tone) are only reachable via `/paper` routing
  - compliance checks are frequently needed independently after drafting

- [ ] Add `contribution-crafter` skill for Stage A2
  - structured contribution statement: theoretical + empirical + practical + methodological
  - currently A2 has no dedicated skill file

- [ ] Register 5 unregistered skills in `registry.yaml`
  - `C_design/data-dictionary-builder.md`, `C_design/data-management-plan.md`, `C_design/prereg-writer.md`, `C_design/variable-operationalizer.md`, `H_submission/credit-taxonomy-helper.md`
  - these files exist on disk but are not in the registry → invisible to orchestrator and docs generation
  - remove duplicate `Z_cross_cutting/tone-normalizer.md` (canonical is `G_compliance/tone-normalizer.md`)

- [x] ~~Enrich 11 thin skills to match Tier 1 quality standard~~ (Completed in Milestone 10)

### P2 Mid-Term

- [ ] Finish MCP/provider hardening
  - make the builtin-vs-external matrix explicit for every MCP slot, not just literature search
  - document when users should wire an external MCP directly versus dropping in a thin local wrapper/stub
  - remove stale `search_web` / `read_url_content` wording from literature skills and fully align prose with provider-layer terminology

- [ ] Consolidate MCP integration for literature search
  - add source-specific merge policy and provenance strategy for `OpenAlex` / `Crossref` enrichment
  - decide whether `fulltext-retrieval` should remain a planning stub by default or grow a stronger resolver abstraction
  - tighten external-provider handoff contracts so builtin literature artifacts can be consumed cleanly by bridge-level wrappers

- [ ] Add resume/checkpoint support for long literature flows
  - `paper-screener`
  - `fulltext-fetcher`
  - related workflow and user docs

- [ ] Close the YAML <-> portable markdown contract gap
  - auto-generate `research-paper-workflow/references/workflow-contract.md`
  - or enforce stronger YAML/MD equivalence validation

- [ ] Expand integration smoke beyond the builtin literature baseline
  - add real bridge command construction checks
  - add recorded external-provider output parsing checks
  - decide which smoke stages should be part of release gating versus optional maintainers' checks

- [ ] Add empirical acceptance / eval coverage for the refactored pipelines
  - `systematic-review-prisma`
  - `empirical-study`
  - `theory-paper`

- [ ] Add a skill-structure lint layer
  - enforce maximum size / section-count heuristics for repo-internal skill files
  - prevent registry-summary duplication inside markdown bodies
  - keep alias skills as thin stubs instead of silently growing into duplicate canonical specs

- [ ] Define an external skill-set borrowing framework
  - classify imported ideas into `provider`, `workflow`, `rubric`, `interaction`, or `canonical skill` before implementation
  - require borrowed capabilities to map back to existing `task_id`, artifact paths, and provider-layer contracts instead of copying prompt bodies wholesale
  - document when an external capability should become an MCP bridge versus a new canonical skill
  - prioritize borrowing evaluation rubrics, review checklists, and workflow structure before borrowing prose-heavy skill bodies
  - add a maintainer-facing intake checklist for assessing drift risk, overlap with existing skills, and canonicalization cost

### P3 Strategic

- [ ] Upgrade Q1-Q4 from labels into executable semantic gates

- [ ] Add economics / finance method packs plus executable validators

- [ ] Add runtime step toggles
  - `--only`
  - `--skip`

- [ ] Improve confidence scoring using agreement + validation outcomes

- [ ] Add metrics / timing output for orchestration runs

- [ ] Add `discussion-writer` skill for structured Discussion sections
  - findings summary → theory dialogue → practical implications → limitations → future work
  - currently Discussion is part of manuscript-architect but lacks dedicated scaffolding

- [ ] Add `limitation-auditor` skill for proactive limitation identification
  - selection bias, measurement, generalizability, design-specific threats
  - run before submission to anticipate reviewer objections

- [ ] Add `qualitative-coding` skill for iterative coding workflow
  - codebook management, inter-rater reliability, theme development
  - currently qualitative analysis relies on general evidence-synthesizer guidance

---

## Deferred Future Bets

- [ ] Standalone power-analysis template
- [ ] Broader official MCP integrations beyond the current reference providers
- [ ] More domain method packs beyond economics / finance
- [ ] Persistent metrics dashboard or `jsonl` trend logging
- [ ] Stronger reproducibility execution layer for generated research code

---

## Roadmap Maintenance Rules

- Keep this file focused on remaining work and verified completed milestones.
- When an item is completed, move it to `Verified Completed Milestones` instead of leaving a stale open task behind.
- Do not duplicate the same task in both a phase narrative and a separate P0-P3 backlog.
- If a task is superseded by a structural redesign, replace the old task with the new canonical one instead of preserving both.
- Prefer one canonical TODO entry per problem, with nearby context if needed, rather than repeating the same work item across multiple sections.
