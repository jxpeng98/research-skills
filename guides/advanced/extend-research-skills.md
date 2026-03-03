# How to Extend / Modify research-skills (Fine-Grained Iteration by Stage/Task/Skill)

This guide answers two questions:
1) How does the current workflow operate as a whole (workflow summary)?
2) If you want to "modify a specific part," where should you make changes, how should you do it, and how can you verify that you haven't broken the consistency across the three clients?

> Core Principle: **Contract-first**. All "hard constraints" ensuring cross-client consistency are defined in `standards/`. Everything else is a detailed elaboration (stage playbooks / skills / templates) to be filled out as needed.

---

## 1) Workflow Overview (A–I)

### 1.1 Runtime Flow (Skill + MCP + Agents)

```text
User intent (Natural Language)
  -> Selects paper_type + task_id + topic
  -> Parses standard contract (contract: outputs / gates)
  -> Parses capability map (capability-map: required_skills / agents / mcp)
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
2. **Change "Who Does This Task / How It is Coordinated"** (Three-client routing, MCP dependencies, required_skills): Edit `standards/mcp-agent-capability-map.yaml`
3. **Change "Definition of Done"** (DoD, granular steps, checklists): Edit `research-paper-workflow/references/stage-*.md`
4. **Change "Specific Execution Specs / Output Formats"** (Skill descriptions, reusable structures): Edit `skills/*.md` and/or `templates/*`
5. **Change "Claude Code Menus / Routing Experience"** (Command entry points / Menu items): Edit `.agent/workflows/*.md`
6. **Change "Orchestrator Behavior"** (task-run injection, concurrency strategies, external MCP command protocols): Edit `bridges/*.py`

> Rule of Thumb: **Change the contract first (if artifacts/paths change) → then update the capability-map (who does it) → then add details to stage playbooks / skills / templates (how to do it) → finally, update workflows (interaction layer).**

---

## 3) Common Modification Scenarios (Checklist)

### 3.1 "I want to make a Task granular/specialized, but without changing the output path."

Focus your edits here (Top to bottom priority):
1. `research-paper-workflow/references/stage-<X>-*.md`: Add check lists, DoD, and common failure modes.
2. `skills/<skill>.md`: Add low-freedom output structures (tables, specific fields, formatting rules).
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
   - `skills/`: Add the output format and "definition of done."

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

### 3.4 "I want to add a new Skill (More granular module)."

1. Create a new skill specification file: `skills/<skill-name>.md`
2. Register it in the orchestration map: `standards/mcp-agent-capability-map.yaml`
   - Add `<skill-name>` to `skill_registry`
   - Add an entry to `skill_catalog` (file/category/focus/default_outputs)
   - Include the skill in the related task's `task_skill_mapping.<task_id>.required_skills`
3. If the skill requires a stable structure: push the structure down into `templates/` and reference the template path within the skill markdown.

### 3.5 "I want to integrate a new external MCP (e.g., local tools for scholar/search/stats)."

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
- **Step Two**: Add `references/stage-*.md` (DoD), `skills/` (execution specs), `templates/` (structured templates).
- **Step Three**: Fix `.agent/workflows/` (Interaction layer routing/menus).
- **Step Four**: Run a full validation suite and update release notes (if issuing a beta).
