# Multi-Client Install Guide (Codex / Claude Code / Gemini)

## 1. Install CLI (Recommended)

Using `pipx` is the recommended way to install the Research Skills Orchestrator:

```bash
pipx install research-skills-installer
# After installation, you can initialize your environment:
rsk upgrade --target all --project-dir /path/to/project --doctor
```

## 2. Alternative: Bash Script

If you don't use pipx, you can use the unified installer script directly from the repository:

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

## Upgrade

- CLI aliases (after pipx install): `rsk` / `rsw` (same as `research-skills`)
- Optional default upstream (omit `--repo`): set `RESEARCH_SKILLS_REPO=<owner>/<repo>`, or add `research-skills.toml` in your project root
- Check updates: `rsk check --repo <owner>/<repo>` (or `rsk check` if `RESEARCH_SKILLS_REPO` is set; or `python3 scripts/research_skill_update.py check ...`)
- Upgrade (no fork / no git clone required): `rsk upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all` (or omit `--repo` if `RESEARCH_SKILLS_REPO` is set; or `python3 scripts/research_skill_update.py upgrade ...`)
- Full guide: `guides/basic/upgrade-research-skills.md`

## Verify

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
