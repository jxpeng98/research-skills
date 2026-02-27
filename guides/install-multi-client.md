# Multi-Client Install Guide (Codex / Claude Code / Gemini)

Use the unified installer:

```bash
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor
```

## Target behaviors

- `codex`
  - Installs `research-paper-workflow` into `${CODEX_HOME:-~/.codex}/skills/research-paper-workflow`.
- `claude`
  - Installs `research-paper-workflow` into `${CLAUDE_CODE_HOME:-~/.claude}/skills/research-paper-workflow`.
  - Copies `.agent/workflows/*.md` into `<project>/.agent/workflows/`.
  - Copies `CLAUDE.md` to `<project>/CLAUDE.md` (or `CLAUDE.research-skills.md` if `CLAUDE.md` already exists and `--overwrite` is not used).
- `gemini`
  - Installs `research-paper-workflow` into `${GEMINI_HOME:-~/.gemini}/skills/research-paper-workflow`.
  - Creates `<project>/.gemini/research-skills.md` with orchestrator quickstart commands.
  - Copies `standards/agent-profiles.example.json` to `<project>/.gemini/agent-profiles.example.json`.

## Common flags

- `--mode copy|link`: copy files or create symlinks.
- `--overwrite`: replace existing installation targets.
- `--dry-run`: preview installation actions only.
- `--doctor`: run `python3 -m bridges.orchestrator doctor --cwd <project>` after install.

## Verify

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
