# How to Extend / Modify research-skills (Fine-Grained Iteration by Stage/Task/Skill)

This guide answers two questions:
1) How does the current workflow operate as a whole (workflow summary)?
2) If you want to "modify a specific part," where should you make changes, how should you do it, and how can you verify that you haven't broken the consistency across the three clients?

> Core Principle: **Contract-first**. All "hard constraints" ensuring cross-client consistency are defined in `standards/`. Everything else is a detailed elaboration (stage playbooks / skills / templates) to be filled out as needed.

---

## 0) Terminology and Layer Model

Before editing anything, distinguish these layers clearly:

- **Contract**: `standards/research-workflow-contract.yaml`
  - Canonical tasks, artifact paths, and quality gates.
- **Capability Map**: `standards/mcp-agent-capability-map.yaml`
  - Which skills, MCP providers, and runtime agents a task uses.
- **Internal Skill Specs**: `skills/*/*.md`
  - Repo-internal reusable execution specifications. These are not automatically portable client skills.
- **Portable Skill Package**: `research-paper-workflow/`
  - Cross-client entry skill distributed to Codex / Claude / Gemini.
- **Functional Agents**: currently represented mainly by `roles/` and pipeline ownership patterns
  - This is the research responsibility layer: literature, methods, writing, compliance, etc.
- **Runtime Agents**: `codex`, `claude`, `gemini`
  - These are model executors selected by the capability map and bridges.
- **Pipelines**: `pipelines/`
  - Abstract DAG definitions and handoff plans.
- **Workflows**: `.agent/workflows/`
  - Claude Code interaction layer and user entrypoints.
- **Bridges**: `bridges/`
  - Runtime adapters and orchestration behavior.

Rule of thumb:
- If you are changing what the system must produce, change the contract.
- If you are changing who owns or routes a task, change the capability map and possibly `roles/`.
- If you are changing how a reusable step is executed, change an internal skill spec.
- If you are changing the end-user entry skill installed to clients, change `research-paper-workflow/`.

## 0.1) Old Structure -> Current Structure Mapping

If you worked on an earlier version of the repo, use this map first:

| Old mental model | Current structure | Edit first |
|---|---|---|
| "The skill package is the system" | `research-paper-workflow/` is only the portable distribution surface | `standards/` or `skills/`, not the portable package |
| "Agent means Claude/Codex/Gemini" | Split into `functional agents` (`roles/`) and `runtime agents` (`standards/` + `bridges/`) | `roles/` for ownership, `capability map` for runtime routing |
| "Slash command = workflow truth" | `.agent/workflows/` is only the entry layer; `pipelines/` is the DAG layer | `pipelines/` first, then `.agent/workflows/` |
| "A new micro-step needs a new skill file" | Many changes belong in templates, parent skill fields, or MCP/provider logic | `templates/`, parent skill markdown, or MCP/bridge layer |
| "Bridges can patch missing standards" | `bridges/` should execute standards, not replace them | Fix `standards/` first |

## 0.2) Stable Entry vs Internal Layers

For users, the stable entrypoints are:

- `research-paper-workflow/` as the portable client skill package
- `.agent/workflows/*.md` as Claude Code command entrypoints
- `python3 -m bridges.orchestrator task-plan|task-run|doctor` as orchestration entrypoints

For maintainers, the internal layers are:

- `standards/` for truth
- `roles/` and `skills/` for reusable responsibility and execution behavior
- `pipelines/` for DAG sequencing
- `bridges/` for runtime behavior

If a user-visible command changes but the underlying truth did not, update the entry layer only.
If the truth changes, start upstream and let the entry layers follow.

---

## 1) Workflow Overview (A–I)

### 1.1 Runtime Flow (Skill + MCP + Agents)

```text
User intent (Natural Language)
  -> Selects paper_type + task_id + topic
  -> Parses standard contract (contract: outputs / gates)
  -> Parses capability map (capability-map: required_skills / runtime agents / mcp)
  -> Resolves functional ownership (roles / pipeline responsibility)
  -> plan
  -> mcp-evidence
  -> primary-agent-draft
  -> review-agent-check
  -> validator-gate
  -> Writes contract outputs into RESEARCH/[topic]/
```

Single Source of Truth:
- The contract (Tasks and Outputs): `standards/research-workflow-contract.yaml`
- The orchestration (Routing of skills/mcp/agents): `standards/mcp-agent-capability-map.yaml`
- Functional ownership conventions: `roles/` + `pipelines/`
- Stage DoD (Detailed "Definition of Done"): `research-paper-workflow/references/stage-*.md`

### 1.2 Stage Map (Recommended starting point)

| Stage | Goal | Task IDs | Detailed Guide |
|---|---|---|---|
| A | Topic Positioning / RQs / Contributions / Theory / Gaps | `A1–A5` | `research-paper-workflow/references/stage-A-framing.md` |
| B | Literature & Related Work (includes systematic review pipeline) | `B1–B6` | `research-paper-workflow/references/stage-B-literature.md` |
| C | Study Design & Analysis Plan | `C1–C5` | `research-paper-workflow/references/stage-C-design.md` |
| D | Ethics / IRB / Compliance Materials | `D1–D3` | `research-paper-workflow/references/stage-D-ethics.md` |
| E | Evidence Synthesis / Meta-Analysis | `E1–E5` | `research-paper-workflow/references/stage-E-synthesis.md` |
| F | Writing (Outline → Paragraphs → Full Text) | `F1–F6` | `research-paper-workflow/references/stage-F-writing.md` |
| G | Polish & Consistency / Compliance Checking | `G1–G4` | `research-paper-workflow/references/stage-G-compliance.md` |
| H | Submission Pack & Rebuttal / Peer Review Simulation | `H1–H4` | `research-paper-workflow/references/stage-H-submission.md` |
| I | Research Code Support (spec → plan → execute → review → audit) | `I1–I8` | `research-paper-workflow/references/stage-I-code.md` |

For the recommended minimum coverage (what tasks must be done for a given `paper_type`), see:
- `research-paper-workflow/references/coverage-matrix.md`

---

## 2) What Do You Want to Modify? Choose Your "Change Type" First

Categorizing your changes will directly determine which file layer you should edit:

1. **Change "What Must Be Produced"** (Artifact paths / Task output sets / Quality gates): Edit `standards/research-workflow-contract.yaml`
2. **Change "Who Owns This Task / How It is Coordinated"** (functional owner, three-client routing, MCP dependencies, required_skills): Edit `standards/mcp-agent-capability-map.yaml` and possibly `roles/`
3. **Change "Definition of Done"** (DoD, granular steps, checklists): Edit `research-paper-workflow/references/stage-*.md`
4. **Change "Specific Execution Specs / Output Formats"** (internal skill-spec descriptions, reusable structures): Edit `skills/*/*.md` and/or `templates/*`
5. **Change "Portable Client Entry Skill"** (what Codex / Claude / Gemini users install as the cross-client entry package): Edit `research-paper-workflow/`
6. **Change "Claude Code Menus / Routing Experience"** (command entry points / menu items): Edit `.agent/workflows/*.md`
7. **Change "Orchestrator Behavior"** (task-run injection, concurrency strategies, external MCP command protocols): Edit `bridges/*.py`

> Rule of Thumb: **Change the contract first (if artifacts/paths change) → then update the capability-map (who does it) → then add details to stage playbooks / skills / templates (how to do it) → finally, update workflows (interaction layer).**

## 2.2) Fast Entry Map: "Where Should I Start?"

| You want to change... | Start here | Then check |
|---|---|---|
| Required output paths or stage deliverables | `standards/research-workflow-contract.yaml` | `skills/registry.yaml`, templates, workflow references |
| Functional owner or runtime routing | `standards/mcp-agent-capability-map.yaml` | `roles/`, `pipelines/`, `bridges/` |
| How a reusable step behaves | `skills/*/*.md` | `templates/`, stage references |
| A repeated table / markdown structure | `templates/` | Parent skill markdown |
| External provider / search / stats connector | `bridges/` + MCP registry | capability map task bindings |
| Claude slash-command menu or UX | `.agent/workflows/*.md` | `pipelines/`, README/quickstart |
| Cross-client portable skill behavior | `research-paper-workflow/` | workflow references, README |

## 2.3) Domain Specialization: How to Customize a Direction

In this repo, "specializing a direction" usually means:
- keeping the core task IDs, stage order, and artifact contract unchanged
- making one discipline or methodology more opinionated about libraries, diagnostics, pitfalls, databases, venue norms, and review expectations

In other words, do **not** start from `standards/` unless the contract itself changes. The default entrypoint for specialization is `skills/domain-profiles/`.

| You want to change... | Start here | Escalate only when... |
|---|---|---|
| Recommended libraries, method checklists, diagnostics, pitfalls, databases, venue norms | `skills/domain-profiles/<domain>.yaml` | You add new fields and must update `schemas/domain-profile.schema.json` |
| A brand-new domain | Copy `skills/domain-profiles/custom-template.yaml` into a new profile | You also want it surfaced in help text, examples, or docs |
| More skills should react to the domain | `skills/registry.yaml` + the affected `skills/*/*.md` files | Task routing or required skills change in `standards/mcp-agent-capability-map.yaml` |
| The code lane should consume domain data differently | `skills/I_code/code-builder.md`, `skills/I_code/stats-engine.md`, `skills/I_code/code-review.md` | Runtime injection logic changes in `bridges/*.py` |
| Output paths, task outputs, or quality gates for that domain | `standards/research-workflow-contract.yaml` | This is no longer lightweight specialization; it is a contract change |

Recommended order:
1. Decide whether you are editing an existing domain or introducing a new one.
2. Put domain knowledge into `skills/domain-profiles/<domain>.yaml` first. The most important sections are:
   - `libraries`
   - `method_templates`
   - `stats_diagnostics`
   - `reporting_guidelines` / `reporting_standards`
   - `common_pitfalls`
   - `visualization_templates`
   - `default_databases`
   - `methodology_priors`
   - `venue_norms`
3. Decide whether the change should affect only the **I-stage code lane**.
   - If yes, the domain profile alone is often enough.
   - If no, update the relevant `domain_aware` skill specs as well.
4. If you introduce new profile fields, update `schemas/domain-profile.schema.json`.
5. Only move up to `standards/` when you are adding a skill, changing task routing, or changing contract artifacts.

Skill files commonly touched during broader domain specialization:
- `skills/A_framing/venue-analyzer.md`
- `skills/B_literature/concept-extractor.md`
- `skills/C_design/study-designer.md`
- `skills/C_design/robustness-planner.md`
- `skills/C_design/dataset-finder.md`
- `skills/C_design/variable-constructor.md`
- `skills/F_writing/manuscript-architect.md`
- `skills/G_compliance/reporting-checker.md`
- `skills/H_submission/submission-packager.md`
- `skills/I_code/code-builder.md`
- `skills/I_code/stats-engine.md`
- `skills/I_code/code-review.md`

Avoid these mistakes:
- Editing `standards/research-workflow-contract.yaml` when you only needed better domain knowledge
- Copy-pasting the same checklist into multiple skill files instead of centralizing it in `domain-profiles/`
- Adding new profile fields without updating the schema or the consuming skills
- Creating a new top-level skill before confirming that it owns a distinct artifact and routing value

Validation after specialization:

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

If the specialization affects runtime behavior, also do one small manual spot check:
- run `code-build` or `task-run` with the target domain
- verify that the generated output actually reflects the new checklist, diagnostics, and reporting norms

## 2.1) Should This Become a New Top-Level Skill?

Before adding a new file under `skills/`, run this filter:

1. Does it consume typed inputs and produce typed outputs?
2. Does it own a stable artifact path under `RESEARCH/[topic]/`?
3. Will a pipeline or task need to depend on it directly?
4. Does it justify separate failure modes or review logic?

If any answer is "no", do not add a new top-level skill yet.

Default destinations when the answer is "no":

- **Provider/API/database adapter**: add or reuse an MCP/provider entry in `standards/mcp-agent-capability-map.yaml`
- **Micro-step inside an existing capability**: expand the parent skill markdown and its templates
- **Field-level extraction variant**: add structured slots to `templates/paper-note.md` / `templates/extraction-table.md`
- **Formatting helper without its own contract artifact**: keep it as a subsection of the parent skill

Examples that should stay out of `skills/`:

- `semantic-scholar-search`
- `crossref-search`
- `database-connector`
- `query-builder`
- `keyword-expander`
- `methodology-extractor`
- `dataset-extractor`
- `theory-extractor`
- `limitation-extractor`

---

## 3) Common Modification Scenarios (Checklist)

### 3.1 "I want to make a Task granular/specialized, but without changing the output path."

Focus your edits here (Top to bottom priority):
1. `research-paper-workflow/references/stage-<X>-*.md`: Add check lists, DoD, and common failure modes.
2. `skills/<A-I_stage>/<skill>.md`: Add low-freedom output structures (tables, specific fields, formatting rules) to the internal skill spec.
3. `templates/<template>.md`: Extract repetitive structures into a template (makes it stable and reusable).

**DO NOT CHANGE**:
- Task ID, stage tags, contract output paths (unless you are doing scenario 3.2).

### 3.2 "I want to add/change an artifact file (Output Path Change)."

You must make coordinated updates across the entire chain:
1. `standards/research-workflow-contract.yaml`
   - Update the corresponding `task_catalog.<ID>.outputs`
   - If it is a core artifact for the stage, update `stages.<stage>.outputs` and `artifacts.required_core`
2. `standards/mcp-agent-capability-map.yaml`
   - If this is the default output of a specific skill, update `skill_catalog.<skill>.default_outputs`
3. Interaction Layer (As needed)
   - `.agent/workflows/*.md`: Ensure the new path is written/referenced.
   - `research-paper-workflow/references/workflow-contract.md`: Update the task table (to maintain portable skill consistency).
4. Artifact Structure (As needed)
   - `templates/`: Add the template.
   - `skills/`: Add the output format and "definition of done" to the internal skill spec.

### 3.3 "I want to add a completely new Task ID."

This is a "Standard Layer" change. Proceed with caution, as it affects the consistency across all platforms.

Steps:
1. `standards/research-workflow-contract.yaml`
   - Add the new Task to `task_catalog` (stage/outputs).
2. `standards/mcp-agent-capability-map.yaml`
   - Add `required_skills` under `task_skill_mapping`.
   - Add `required_mcp`, agent routing, and `quality_gates` under `task_execution`.
3. `research-paper-workflow/references/workflow-contract.md`
   - Add the new task to the task table.
4. `.agent/workflows/paper.md`
   - Add an entry point to the menu (so users can select it).
5. Update local consistency validations (otherwise CI will fail):
   - `scripts/validate_research_standard.py`: Add the new Task ID to the expected sets.
6. Optional: Create/Update the stage playbook (to explain the DoD).

### 3.4 "I want to add a new internal Skill Spec (More granular module)."

Start by applying the top-level skill filter in section `2.1`.

Only proceed when the new capability clearly owns its own typed artifact and direct orchestration value.

1. Create a new skill specification file: `skills/<A-I_stage>/<skill-name>.md`
2. Register it in the orchestration map: `standards/mcp-agent-capability-map.yaml`
   - Add `<skill-name>` to `skill_registry`
   - Add an entry to `skill_catalog` (file/category/focus/default_outputs)
   - Include the skill in the related task's `task_skill_mapping.<task_id>.required_skills`
3. If the skill requires a stable structure: push the structure down into `templates/` and reference the template path within the skill markdown.

If you instead need a client-installable entry skill, edit or create a package under `research-paper-workflow/` style rather than adding it to `skills/`.

If the capability turns out to be a provider wrapper or a field-level sub-extractor, stop and:

1. Extend the parent skill markdown.
2. Extend the relevant template(s).
3. Route the provider dependency through `mcp_registry` instead of `skill_registry`.

### 3.5 "I want to change who owns a task without changing the runtime model."

1. Start with `roles/` to clarify the functional-owner concept and preferred skill set.
2. Update `standards/mcp-agent-capability-map.yaml` if task ownership, required skills, or review expectations change.
3. Update `pipelines/` if the handoff sequence between literature / methods / writing / compliance responsibilities changes.
4. Only touch `bridges/` if the runtime execution logic itself must change.

### 3.6 "I want to integrate a new external MCP (e.g., local tools for scholar/search/stats)."

1. Add the provider name to `mcp_registry` in `standards/mcp-agent-capability-map.yaml`.
2. Reference it under `task_execution.<task_id>.required_mcp` for relevant tasks.
3. Inject the command via an environment variable at runtime (the command should accept JSON via stdin, and return JSON via stdout):
   - Variable name: `RESEARCH_MCP_<PROVIDER>_CMD`
   - Check conventions in: `bridges/mcp_connectors.py`

---

## 4) Post-Change Verification (Run Locally / Same as CI)

Run this at least once after making changes:

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

Optional (if you have the CLIs for all three clients configured locally, and API keys are set up):

```bash
./scripts/run_beta_smoke.sh
python3 -m bridges.orchestrator doctor --cwd .
```

---

## 5) Recommended Commit Granularity (Avoid breaking everything at once)

- **Step One**: Only modify `standards/` (contract / capability-map), run validator.
- **Step Two**: Add `references/stage-*.md` (DoD), `skills/*/` (internal execution specs), `templates/` (structured templates).
- **Step Three**: Fix `.agent/workflows/` (Interaction layer routing/menus).
- **Step Four**: Fix `research-paper-workflow/` if the portable client-facing package also needs to reflect the change.
- **Step Five**: Run a full validation suite and update release notes (if issuing a beta).
