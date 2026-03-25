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

- Verified completed workstreams: 7
- Verified-but-not-fully-accepted items: 4
- Active TODOs: 19
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
- `scripts/generate_release_notes.sh`
- `scripts/install_research_skill.sh`
- `scripts/sync_versions.py`
- `docs/conventions.md`
- `guides/advanced/extend-research-skills.md`
- `research-paper-workflow/SKILL.md`
- `tests/test_orchestrator_workflows.py`
- `tests/test_skill_doc_generation.py`
- `tests/test_sync_versions.py`

Key facts confirmed from current repo:
- `platform_mapping.claude_code` already covers the full current task set in `task_catalog`
- `sequence_index` already exists in the canonical stage contract
- functional-agent routing is already present in capability map, pipelines, validator, and orchestrator
- `runtime_options` already flow into Codex / Claude / Gemini bridge construction
- `team-run` already exists as a distinct fanout/fanin execution mode
- `--domain` runtime injection already exists
- native reference MCP scripts already exist for `scholarly-search` and `citation-graph`
- release preparation is already unified through `scripts/release_ready.sh`
- `skills/registry.yaml` is now the single canonical skill version source

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

---

## Verified but Not Yet Fully Accepted

- [ ] `team-run` still needs real-agent acceptance evidence for `B1`
- [ ] `team-run` still needs real-agent acceptance evidence for `H3`
- [ ] `task-run` still does not execute a distinct post-run filesystem `validator-gate`
- [ ] `skills/Z_cross_cutting/model-collaborator.md` is still code-first rather than research-wide

---

## Active TODO List

### P0 Blocking

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

### P2 Mid-Term

- [ ] Finish MCP/provider hardening
  - add a native `metadata-registry` reference provider
  - keep `doctor` output explicit about builtin vs env-configured providers
  - make the builtin-vs-external matrix explicit for every MCP slot, not just literature search
  - document when users should wire an external MCP directly versus dropping in a thin local wrapper/stub
  - remove stale `search_web` / `read_url_content` wording from literature skills and fully align prose with provider-layer terminology

- [ ] Consolidate MCP integration for literature search
  - define one canonical provider contract for literature retrieval, citation expansion, metadata enrichment, and fulltext lookup
  - unify provider naming across skills, capability map, doctor output, docs, and user-facing prompts
  - make literature-search routing explicit between `scholarly-search`, `citation-graph`, `metadata-registry`, and `fulltext-retrieval`
  - document clearly which layers already have builtin reference providers (`scholarly-search`, `citation-graph`) and which are external capability slots by default
  - provide a user-facing decision table for zero-config, key-assisted, external MCP, and stub/wrapper-based literature setups
  - standardize search evidence artifacts so query plan, provider hits, dedup log, and retrieval provenance are emitted in one consistent bundle
  - clarify fallback order between builtin search, env-configured MCP providers, and bridge-level wrappers for literature workflows

- [ ] Add resume/checkpoint support for long literature flows
  - `paper-screener`
  - `fulltext-fetcher`
  - related workflow and user docs

- [ ] Close the YAML <-> portable markdown contract gap
  - auto-generate `research-paper-workflow/references/workflow-contract.md`
  - or enforce stronger YAML/MD equivalence validation

- [ ] Add integration smoke tests
  - real bridge command construction
  - real or recorded CLI output parsing
  - minimal end-to-end artifact checks

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
