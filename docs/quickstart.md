# Quick Start Guide

This guide focuses on stable user-facing entrypoints. You do not need to understand `skills/`, `roles/`, or `pipelines/` to start using the system.

## 1. Pick an Entry Mode

Use one of these stable entrypoints:

| Entry mode | Use when | Entry |
|---|---|---|
| Claude Code commands | You want slash-command UX inside a project | `.agent/workflows/*.md` commands such as `/paper`, `/lit-review`, `/paper-write` |
| Orchestrator CLI | You want explicit task routing and validation | `python3 -m bridges.orchestrator task-plan|task-run|doctor` |
| Portable skill package | You want the cross-client installable entry skill | `research-paper-workflow/` |

## 2. Validate Local Readiness

From the repo root:

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 scripts/validate_research_standard.py --strict
```

Use `doctor` to check runtime CLIs, API keys, and MCP command wiring.
Use the validator to confirm the repo standards are internally consistent.

## 3. Choose a Paper Type

The canonical paper-type pipelines are:

| Paper type | Pipeline | Typical use |
|---|---|---|
| `systematic-review` | `systematic-review-prisma` | PRISMA-style evidence review |
| `empirical` | `empirical-study` | Standard empirical research paper |
| `empirical` | `rct-prereg` | RCT with preregistration and reporting checks |
| `theory` | `theory-paper` | Conceptual or theory-building paper |
| `methods` | `code-first-methods` | Methods paper where code is a first-class deliverable |

## 4. Plan Before You Run

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

## 5. Run a Canonical Task

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

## 6. Use Claude Code Commands When You Want UX

If you prefer command entrypoints instead of explicit Task IDs, use the Claude workflow commands after installation into your project:

```text
/paper
/lit-review
/paper-read
/find-gap
/study-design
/paper-write
/submission-prep
```

These commands are entry UX only. The canonical task and artifact truth still lives in `standards/`.

## 7. Know When to Switch to Maintainer Docs

Use this guide for operation.
Switch to maintainer docs only when you are changing the system itself:

- Architecture and layer model: [Architecture](/architecture)
- Edit rules and decision boundaries: [Conventions](/conventions)
- Where to modify specific behavior: [Extend Research Skills](/advanced/extend-research-skills)
