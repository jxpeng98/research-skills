# CLAUDE Guide Summary

This page distills the operational guidance from `CLAUDE.md` into a maintainer-facing checklist.

## What `CLAUDE.md` Is Doing

`CLAUDE.md` is not just a project introduction. It acts as a maintainer/operator playbook for:

- fast repo orientation
- common runtime commands
- architectural expectations
- quality vocabulary
- collaboration patterns across `codex`, `claude`, and `gemini`

## Maintainer Priorities

### 1. Keep contract truth upstream

When behavior changes, maintainers should prefer fixing:

1. `standards/`
2. `roles/` or `skills/`
3. `templates/`
4. `pipelines/` and `.agent/workflows/`
5. `bridges/`
6. `research-paper-workflow/`

### 2. Treat workflows as entry UX, not truth

Slash commands are convenient entrypoints. Artifact truth, routing truth, and task truth still live in the standards layer.

### 3. Use the repo through stable commands

Common commands pulled from `CLAUDE.md`:

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd .
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

### 4. Think in collaboration modes

The maintainer mental model is:

- single-agent execution for narrow/debug flows
- draft/review fallback for standard task execution
- triad review when independent audit matters
- role split or parallel fanout when work can be decomposed

## When To Open The Original `CLAUDE.md`

Use the original file when you need:

- raw command examples
- the long-form workflow descriptions
- the full project-specific terminology block
- the exact examples for collaboration and stage usage

For most day-to-day navigation, this summary plus [Architecture](/architecture) and [Conventions](/conventions) should be enough.
