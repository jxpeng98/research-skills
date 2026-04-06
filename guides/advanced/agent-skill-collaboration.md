# Agent + Skill Collaboration Enhancement Guide

This guide is designed to systematically enhance a specific capability (not limited to code) within `research-skills` while maintaining cross-model consistency.

## 1) Define the Goal: Which Capability to Enhance

First, bind the capability to a standard Task ID (`A1`~`I8`):

- **Topic Selection & Positioning**: `A1`~`A4`
- **Literature & Review**: `B1`~`B5`
- **Study Design / Ethics**: `C1`~`D2`
- **Evidence Synthesis**: `E1`~`E5`
- **Drafting**: `F1`~`F6`
- **Compliance & Proofread**: `G1`~`J4` (Including de-AI and humanization)
- **Submission & Rebuttal**: `H1`~`H4`
- **Code & Replication**: `I1`~`I8` (Includes CCG strict-constraint code engine)

Once the target task is determined, you can reuse the unified orchestration chain: `plan -> mcp-evidence -> primary-agent-draft -> review-agent-check -> validator-gate`.

## 2) Division of Labor Principles (Fixed)

- **Skill**: Methodology and artifact standards (What to do, what to produce).
- **MCP**: Evidence and tooling layer (Where to fetch evidence, how to save).
- **Agent**: Reasoning and execution layer (How to complete the draft and review).

It is recommended to always retain the "dual agent" structure: Primary execution + Independent review.

## 3) How to Enhance a Capability (Standard Workflow)

1. Select the target task (e.g., `E3` or `I2`).
2. Update the following in `standards/mcp-agent-capability-map.yaml`:
   - `required_mcp`
   - `required_skills`
   - `required_skill_cards` (Automatically parsed by `skill_catalog`)
   - `primary_agent/review_agent/fallback_agent`
3. If adding a new skill:
   - Create `skills/<A-I_stage>/<skill-name>.md`
   - Add it to `skill_registry`, `skill_catalog`, and `task_skill_mapping`
4. If adding a new agent runtime:
   - Add a bridge in `bridges/`
   - Integrate it into the runtime router in `bridges/orchestrator.py`
   - Refer to existing implementations: `bridges/claude_bridge.py`
5. Run validations:
   - `python3 scripts/validate_research_standard.py --strict`

## 3.1) External MCP Integration Conventions (Command Mode)

For MCPs other than `filesystem`, `task-run` uses environment variables to inject external commands:

- Variable naming convention: `RESEARCH_MCP_<PROVIDER>_CMD`
- Example: `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`

Execution Protocol:

1. The orchestrator passes JSON to the command's `stdin`:
   - `provider`
   - `task_packet`
2. The external command returns JSON via `stdout`:
   - `status`: `ok|warning|error|not_configured`
   - `summary`: Brief summary string
   - `provenance`: List of sources (optional)
   - `data`: Structured additional info (optional)

If the variable is not configured, the status is `not_configured`; you can use `task-run --mcp-strict` to forcefully block execution.

## 3.2) Skill Injection Conventions (Standardized Skill Cards)

`task-run` will automatically inject `required_skill_cards` from `skill_catalog`. Each card contains at least:

- `skill`: Skill name
- `category`: Skill category (e.g., `evidence-synthesis`, `research-code`)
- `focus`: Primary execution focus
- `file`: Path to the skill specification (`skills/*/*.md`)
- `default_outputs`: Recommended artifact output paths

Use `task-run --skills-strict` to block execution if the skill specification files are missing.

## 3.3) Profile Injection Conventions (Persona / Style / Tool Permissions)

Avoid global fixed configurations; use the "per-run injected" profile mechanism:

- Profile file: `standards/agent-profiles.example.json`
- Parallel mode:
  - `parallel --profile-file ... --profile ... --summarizer-profile ...`
- Task mode:
  - `task-run --profile-file ... --profile ...`
  - `task-run --draft-profile ... --review-profile ... --triad-profile ...`

Priority (High -> Low):

1. Command-line explicit parameters (e.g., `--review-profile strict-review`)
2. `task_overrides` (Override by Task ID)
3. `--profile` (Default profile for this run)
4. Built-in `default` profile

A profile can define:

- `persona`
- `analysis_style` / `draft_style` / `review_style` / `summary_style` / `triad_style`
- `runtime_options` (Agent-specific tool permissions, e.g., Codex sandbox, Claude permission mode, Gemini sandbox)
  - Recommended settings: `non_interactive: true`, `timeout_seconds`
  - Optional strict auth: `require_api_key: true` (Fails fast if key is missing, avoiding getting stuck in login flows)

## 4) Recommended Collaboration Templates by Capability Type

### A. Code Capabilities (`I1`~`I8`)

- **CCG Strict Execution Constraint (I5-I8)**: Drawing from `ccg-workflow`, the code phase is strictly split into Constraint Extraction (I5) -> Decision-free Planning (I6) -> Primary Execution (I7) -> Side-channel Validation (I8).
- Recommended skills: `code-specification`, `code-planning`, `code-execution`, `code-review`
- Recommended MCPs: `code-runtime`, `filesystem`
- Agent combination: Primary `codex` (Executes I7), Review `gemini` (Validates I8)

### B. Systematic Review Capabilities (`B1`)

- Recommended skills: `academic-searcher`, `paper-screener`, `paper-extractor`, `prisma-checker`, `evidence-synthesizer`, `model-collaborator`
- Recommended MCPs: `scholarly-search`, `screening-tracker`, `extraction-store`, `fulltext-retrieval`
- Agent combination: Primary `claude`, Review `codex`

### C. Evidence Synthesis and Meta-Analysis (`E1/E2/E3`)

- Recommended skills: `evidence-synthesizer`, `quality-assessor`, `code-builder`
- Recommended MCPs: `stats-engine`, `extraction-store`
- Agent combination: Primary `codex`, Review `claude`

### D. Drafting and Consistency (`F3/G3`)

- Recommended skills: `manuscript-architect`, `citation-formatter`, `reporting-checker`, `quality-assessor`
- Recommended MCPs: `metadata-registry`, `reporting-guidelines`
- Agent combination: Primary `claude`, Review `codex`

### E. Proofread & De-AI (`J1`~`J4`)

- **Multi-AI Triad Iteration**: Use triad mode to perform iterative de-AI. Drafter rewrites text, Reviewer checks for AI fingerprints, and Auditor ensures scientific accuracy.
- Recommended skills: `proofread-editor`, `ai-detector`, `similarity-checker`
- Agent combination: Primary `claude`, Review `gemini`, Triad `codex` (via `task-run --triad`)

### F. Submission and Rebuttal (`H1`~`H4`)

- **Multi-Role Expert Cross-Review (H3-H4)**: Before final submission, use parallel invocations to simulate harsh reviewers (Methodologist, Domain Expert) across a cross-review (H3) and execute a Fatal Flaw Desktop-reject scan (H4).
- Recommended skills: `submission-packager`, `rebuttal-assistant`, `peer-review-simulation`, `fatal-flaw-detector`, `model-collaborator`
- Recommended MCPs: `submission-kit`, `metadata-registry`, `reporting-guidelines`
- Agent combination: Primary `claude`, Review `gemini/codex`

## 4.1) `team-run` Acceptance Workflow (`B1`, `H3`)

`team-run` is the fanout/fanin execution mode for the current MVP tasks:

- `B1`: planner or fallback partitioning for systematic-review shards
- `H3`: fixed reviewer personas (`methodologist`, `domain_expert`, `reviewer_2`)

Use the receipt helper to capture one real run instead of relying only on mock tests:

```bash
python3 scripts/capture_team_run_acceptance.py \
  --task-id B1 \
  --paper-type systematic-review \
  --topic acceptance-probe \
  --cwd . \
  --max-units 2 \
  --receipt release/acceptance/team-run-b1-local-receipt.md

python3 scripts/capture_team_run_acceptance.py \
  --task-id H3 \
  --paper-type empirical \
  --topic acceptance-probe \
  --cwd . \
  --receipt release/acceptance/team-run-h3-local-receipt.md
```

Interpretation rules:

- `Barrier Status: ok`: all shards reached merge/review.
- `Barrier Status: degraded`: enough shards succeeded to merge; keep the receipt and inspect missing shard notes before trusting the merged output.
- `Barrier Status: blocked`: treat the receipt as environment evidence, not product acceptance. Keep the exact blocking observations.

Current local receipts in this repo show two concrete block classes:

- `B1`: outbound scholarly-search resolution failed and optional external MCP overlays were not configured.
- `H3`: `claude` / `gemini` CLIs were absent from `PATH`, and the Codex worker produced no consumable agent message.

## 5) Execution Entry Points (Unified)

It is recommended to run a pre-flight check first:

```bash
python -m bridges.orchestrator doctor --cwd ./project
```

### Auto vs Interactive Mode
By default, the orchestrator runs in **Auto Mode**, executing all agents and synthesis steps seamlessly.
To enable **Interactive Step-by-Step Mode** (pauses for human `Y/n/p/q` confirmation before invoking any agent), append `-i` or `--interactive` to any command. Note: inside an AI chat terminal (like Claude Code), simply ask the AI to do it "step by step" via natural language instead of using `-i`.

Use `task-run` to execute by task and automatically inject `required_skills + required_skill_cards`:

```bash
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --context "Target venue style and strict claim-evidence alignment" \
  --mcp-strict \
  --skills-strict \
  --triad \
  -i  # (Optional) Interactive step-by-step
```

`--triad` automatically invokes a third runtime agent for an independent audit after the primary draft and review, maintaining a three-end collaboration even during the non-code `A`~`H` phases.

Parallel Analysis Mode (Not restricted by Task ID):

```bash
python -m bridges.orchestrator parallel \
  --prompt "Review the risks, evidence gaps, and improvement priorities for the current study design" \
  --cwd ./project \
  --summarizer claude \
  -i
```

This mode defaults to concurrent multi-agent execution (Codex/Claude/Gemini) followed by a synthesis analysis; it auto-downgrades to dual or single agents if three are not available.

## 6) External Agents vs. Custom Agents?

A hybrid strategy is recommended:

- **External agent/runtime**: Handles the ceiling of general capabilities (Code generation, reasoning, long-context text).
- **Local mappings and constraints**: Ensures consistency and controllability for the specific research scenario (Task ID, Quality Gates, Artifact paths, Skill constraints).

In other words: Outsource the "capability" to external agents, but keep the "standards" locally.
