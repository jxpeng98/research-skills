# Multi-Client Install Guide (Codex / Claude Code / Gemini)

## 1. Portable Install (No Python Required)

The most portable install path is the shell bootstrapper. It downloads the selected release archive and runs the bundled installer:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --project-dir /path/to/project \
  --target all
```

Requirements:
- `bash`
- `curl` or `wget`
- `tar`

Notes:
- By default this also installs a shell CLI: `research-skills`, `rsk`, `rsw`.
- Default CLI location: `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`.
- Add `--overwrite` when re-installing/upgrading existing targets.
- Use `--no-cli` if you only want the workflow assets.
- Use `--cli-dir <path>` to install the shell CLI elsewhere.
- `--doctor` is optional and only runs when `python3` is available.
- Remote bootstrap only supports `--mode copy`. If you want `--mode link`, clone the repo and use the local installer below.

## 2. Optional Python CLI

If Python is already available on the machine, you can install the updater CLI with `pipx`:

```bash
pipx install research-skills-installer
rsk upgrade --target all --project-dir /path/to/project --doctor
```

## 3. Local Repository Installer

If you already have a repository checkout, you can run the installer directly:

```bash
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --install-cli --doctor
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
- `--install-cli`: install shell CLI commands (`research-skills`, `rsk`, `rsw`).
- `--no-cli`: skip shell CLI installation.
- `--cli-dir <path>`: choose where the shell CLI is installed (default: `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`).
- `--overwrite`: replace existing installation targets.
- `--dry-run`: preview installation actions only.
- `--doctor`: run `python3 -m bridges.orchestrator doctor --cwd <project>` after install when `python3` is available.

## Upgrade

- CLI aliases (after pipx install): `rsk` / `rsw` (same as `research-skills`)
- Shell CLI aliases (after bootstrap install): `rsk` / `rsw` / `research-skills`
- Optional default upstream (omit `--repo`): set `RESEARCH_SKILLS_REPO=<owner>/<repo>`, or add `research-skills.toml` in your project root
- Python-free refresh: rerun `bootstrap_research_skill.sh` with `--overwrite`
- Check updates: `rsk check --repo <owner>/<repo>` (shell CLI or Python CLI; or `python3 scripts/research_skill_update.py check ...`)
- Upgrade (no fork / no git clone required): `rsk upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all` (shell CLI or Python CLI; or `python3 scripts/research_skill_update.py upgrade ...`)
- Full guide: [Upgrade](/guide/upgrade)

## Verify

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
