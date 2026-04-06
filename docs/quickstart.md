# Quick Start Guide

This guide focuses on stable user-facing entrypoints. You do not need to understand `skills/`, `roles/`, or `pipelines/` to start using the system.

If you do want a detailed map of what each internal skill section contains, see [Skills Guide](/reference/skills).
If you want scenario-driven routes such as literature review, empirical design, qualitative fieldwork writing, code-first methods, or rebuttal prep, see [Task Recipes](/guide/task-recipes).

::: warning Full Functionality Requirement
If you want the full system, install and configure all of the following:

- `python3`
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`

Without them, you can still install workflow assets and use shell `rsk check|upgrade|align`, but `doctor`, validators, tests, and full orchestrator execution will be limited.
:::

## 1. Global One-Click Install

The recommended path is the one-click bootstrap. You do not need to manually preinstall Python first or copy files into your projects.

Linux / macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --target all
```

Windows PowerShell:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Target all
```

The bootstrap will install `research-paper-workflow` globally into the respective configuration directories of Codex, Claude Code, and Gemini. It will also create global symlinks for Slash commands.

## 2. Zero-Config Usage

Because the commands are registered globally, using the system is now instantaneous.

1. **Create an empty workspace:** `mkdir my-new-paper && cd my-new-paper`
2. **Start your AI agent:** Type `claude` or `gemini` in your terminal.
3. **Execute a workflow:** Type `/paper` or `/lit-review`.

The AI will seamlessly fetch the global skill package in the background.

## 3. Advanced Entry Modes

| Entry mode | Use when | Entry |
|---|---|---|
| Slash commands | You want `/paper`, `/lit-review`, etc. | Included by default via global symlinks |
| Orchestrator CLI | You want explicit automated task routing and validation | `python3 -m bridges.orchestrator task-plan|task-run|doctor` |
| Installer / updater CLI | You want install or refresh commands after bootstrap | `research-skills`, `rsk`, `rsw` |

## 4. Choose a Paper Type

The canonical paper-type pipelines are:

| Paper type | Pipeline | Typical use |
|---|---|---|
| `systematic-review` | `systematic-review-prisma` | PRISMA-style evidence review |
| `empirical` | `empirical-study` | Standard empirical research paper |
| `qualitative` | `qualitative-study` | Interview, case, ethnographic, or process-oriented qualitative paper |
| `empirical` | `rct-prereg` | RCT with preregistration and reporting checks |
| `theory` | `theory-paper` | Conceptual or theory-building paper |
| `methods` | `code-first-methods` | Methods paper where code is a first-class deliverable |

## 5. Plan Before You Run

Inspect prerequisites and routing before execution:

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

`task-plan` shows:

- contract outputs
- prerequisite tasks
- functional owner
- functional handoff trace
- runtime plan (`draft` / `review` / `fallback`)

## 6. Run a Canonical Task

Execute one task with routing, MCP evidence collection, and review:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

Common options:

- `--mcp-strict`: block if required MCP providers are unavailable
- `--skills-strict`: block if required internal skill specs are missing
- `--triad`: request a third independent audit when available
- `--profile`, `--draft-profile`, `--review-profile`, `--triad-profile`: select execution profiles
- `--focus-output` and `--output-budget`: reduce auxiliary artifact spread for this run by keeping only a smaller active output set
- `--research-depth deep` plus `--max-rounds`: force a narrower, more adversarial evidence-expansion and revision process

## 7. Use Slash Commands When You Want UX

After `rsk upgrade`, workflow slash-commands are globally registered via symlinks:

- **Claude Code**: `~/.claude/commands/*.md`
- **Gemini CLI**: `~/.gemini/workflows/*.md`

No per-project setup required. Available commands:

```text
/paper
/lit-review
/paper-read
/find-gap
/study-design
/paper-write
/submission-prep
/academic-present
```

These commands are entry UX only. The canonical task and artifact truth still lives in `standards/`.

To remove all symlinks: `rsk clean --globals`.

## 8. Know When to Switch to Maintainer Docs

Use this guide for operation.
Switch to maintainer docs only when you are changing the system itself:

- Architecture and layer model: [Architecture](/architecture)
- Edit rules and decision boundaries: [Conventions](/conventions)
- Where to modify specific behavior: [Extend Research Skills](/advanced/extend-research-skills)
