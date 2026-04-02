# Optimization Roadmap TODO

> Last reorganized: 2026-04-02
> This file now tracks only three things:
> 1. verified completed milestones
> 2. active remaining work
> 3. deferred future bets
>
> Historical scoring snapshots, stale availability notes, and duplicated phase writeups have been removed from the main body.

---

## Snapshot

- Verified completed workstreams: 24
- Verified-but-not-fully-accepted items: 2
- Active TODOs: 8
- Deferred future bets: 5

---

## Verification Basis

Checked against current repository structure on 2026-04-02.

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
- standalone `/compliance-check` workflow already exists under `.agent/workflows/`
- `contribution-crafter`, `statement-generator`, `effect-size-calculator`, `qualitative-coding`, `discussion-writer`, and `limitation-auditor` already exist in the skill corpus
- newly added workflow skills now align to canonical contract paths; remaining literature work is concentrated in provider-side consolidation decisions

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

### 12. Structure Guardrails and Content Completion

- [x] Deleted `skills/I_code/{build,planning,qa,run}/` alias subdirectories (7 duplicate files)
- [x] Deleted duplicate `skills/Z_cross_cutting/tone-normalizer.md` (canonical remains in `G_compliance/`)
- [x] Added `alias_of`/`canonical`/`deprecated` semantic fields to `skills/registry.yaml`
- [x] Implemented `validate_skill_structure()` in validator — enforces `## Purpose` + `## Process` sections
- [x] Added metadata drift check — frontmatter `description` vs registry `summary` consistency
- [x] Registered 5 orphan skills: `data-dictionary-builder`, `data-management-plan`, `prereg-writer`, `variable-operationalizer`, `credit-taxonomy-helper`
- [x] Created `skills/J_proofread/` with 4 skill files: `ai-fingerprint-scanner`, `human-voice-rewriter`, `similarity-checker`, `final-proofreader`
- [x] Added 6 new artifact types to `schemas/artifact-types.yaml`
- [x] Standardized headings in 53 legacy skill files (44 `## Purpose` added, 14 `## Process` renames)
- [x] Validator now runs cleanly at `5174 passed, 0 failed, 0 warnings`

### 13. Runtime Validator-Gate

- [x] Implemented `_validator_gate()` in `bridges/orchestrator.py`
- [x] Checks `required_outputs` existence under `RESEARCH/[topic]/` after task-run
- [x] Reports found/missing counts in routing notes
- [x] Adjusts confidence score proportional to missing outputs
- [x] Returns `validator_gate` dict in `CollaborationResult.data`
- [x] Extended `scripts/validate_project_artifacts.py` task coverage to include `J*` and `K*`
- [x] Added direct validator tests for project-artifact task-ID coverage
- [x] All 101 unit tests pass

### 14. Skill Surface Expansion Beyond Core MVP

- [x] Added `contribution-crafter` for Stage A2
- [x] Added `statement-generator` for Stage D2
- [x] Added `effect-size-calculator` for Stage E2
- [x] Added standalone `/compliance-check` workflow for Stage G access
- [x] Added `discussion-writer`, `limitation-auditor`, and `qualitative-coding` skill files
- [x] Registered the above skills in `skills/registry.yaml` and surfaced them in generated skill docs

### 15. Contract and Eval Hardening

- [x] Aligned `discussion-writer`, `limitation-auditor`, and `qualitative-coding` to canonical contract artifact paths
- [x] Added golden eval coverage for `systematic-review-prisma`, `empirical-study`, and `theory-paper`
- [x] Added eval drift tests so cases must reference real pipelines and pipeline-owned skills

### 16. Team-Run Acceptance and Documentation

- [x] Captured real `team-run` acceptance receipts for `B1` and `H3`
- [x] Recorded concrete block/degrade observations from local runtime conditions
- [x] Added a reusable receipt helper for future `team-run` acceptance runs
- [x] Updated collaboration-layer docs for `team-run` in EN/ZH mirrors

### 17. Structured YAML Parsing Hardening

- [x] Replaced regex-style YAML extraction in `bridges/orchestrator.py` with structured parsing helpers
- [x] Added regression coverage for domain-profile loading and Stage-I frontmatter parsing drift

### 18. Registry-Driven Skill Metadata

- [x] Added `display_name` and `when_to_use` as English user-facing registry fields
- [x] Promoted `canonical`, `alias_of`, and `deprecated` into validated registry semantics
- [x] Switched generated skill docs and orchestrator skill cards to consume registry-driven user-facing metadata

### 19. Research-Wide Collaboration Semantics

- [x] Generalized `model-collaborator` wording from code-only framing to research-wide collaboration
- [x] Routed `model-collaborator` into explicit multi-agent tasks `B1` and `H3`
- [x] Updated collaboration guidance so systematic review and rebuttal paths surface the shared multi-agent layer

### 20. Installer Ergonomics

- [x] Added `--parts` support to the shell installer, Python bootstrap, and universal installer
- [x] Passed part-level install control through `rsk upgrade`
- [x] Added Python `rsk doctor` and `rsk init` entrypoints for project bootstrap and environment checks
- [x] Added installer/CLI regression coverage for selective install surfaces and new subcommands

### 21. Literature Metadata Merge Hardening

- [x] Added field-aware merge policy for `OpenAlex` / `Crossref` metadata enrichment
- [x] Exposed enrichment merge traces and policy version in `reference_state.external_enrichment`
- [x] Added regression coverage for provider-specific field precedence and provenance outcomes

### 22. Provider and Release Gating Hardening

- [x] Made the builtin-vs-external matrix explicit for every MCP slot
- [x] Documented when to wire an external MCP versus using builtin/stub overlays
- [x] Removed stale `search_web` / `read_url_content` wording from literature skills
- [x] Added real bridge command construction checks
- [x] Added recorded external-provider output parsing checks
- [x] Split release smoke into release-tier versus maintainer-tier checks

### 23. Fulltext Resolver Handoff Contract

- [x] Added a versioned resolver handoff contract for `fulltext-retrieval`
- [x] Accepted bridge-friendly row aliases such as `reference_id`, `status`, `pdf_path`, `rights`, and `version`
- [x] Exposed `external_resolution.merge_trace` for resolver merge auditing

### 24. Literature Resume Checkpoints

- [x] Added builtin `screening-tracker` checkpoint stub at `scripts/mcp_screening_tracker.py`
- [x] Derived repo-local resume checkpoints from `screening/*.md` and `retrieval_manifest.csv`
- [x] Documented when builtin screening checkpoints are enough versus when an external reviewer tracker is still required

---

---

## Active TODO List

### P0 Blocking

- [x] ~~Create Stage J skill directory and skill cards~~ (Completed in Milestone 12)

- [x] ~~Implement a real `validator-gate` in `task-run`~~ (Completed in Milestone 13)
  - Follow-through: `--skip-validation` now disables strict MCP/skill checks early and marks the validator gate as skipped with explicit warning output

- [x] ~~Complete real-agent acceptance for `team-run`~~ (Completed in Milestone 16)

### P1 Near-Term

- [x] ~~Replace regex-style YAML parsing in `bridges/orchestrator.py`~~ (Completed in Milestone 17)

- [x] ~~Finish collaboration-layer docs for `team-run`~~ (Completed in Milestone 16)

- [x] ~~Generalize `model-collaborator` from code-only to research-wide collaboration language~~ (Completed in Milestone 19)

- [x] ~~Collapse duplicate / alias skill layouts into one canonical-file rule~~ (Completed in Milestone 12)

- [x] ~~Make alias semantics explicit in skill metadata~~ (Completed in Milestone 12)

- [x] ~~Standardize a strict skill file skeleton~~ (Completed in Milestone 12)

- [x] ~~Move more user-facing skill metadata out of markdown prose and into the registry~~ (Completed in Milestone 18)

- [x] ~~Improve install/upgrade ergonomics~~ (Completed in Milestone 20)

- [x] ~~Register 5 unregistered skills in `registry.yaml`~~ (Completed in Milestone 12)

- [x] ~~Enrich 11 thin skills to match Tier 1 quality standard~~ (Completed in Milestone 10)

### P2 Mid-Term

- [ ] Consolidate MCP integration for literature search
  - [x] add source-specific merge policy and provenance strategy for `OpenAlex` / `Crossref` enrichment
  - decide whether `fulltext-retrieval` should remain a planning stub by default or grow a stronger resolver abstraction
  - [x] tighten external-provider handoff contracts so builtin literature artifacts can be consumed cleanly by bridge-level wrappers

- [x] Close the YAML <-> portable markdown contract gap
  - [x] auto-generate `research-paper-workflow/references/workflow-contract.md`
  - [x] enforce stronger YAML/MD equivalence validation

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
