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
- Installer mode selection is `--mode copy|link`. Remote bootstrap only supports `--mode copy`.
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

## Global-First Behaviors & What Gets Installed

Default install/upgrade behavior is purely **global**. Your project directories remain clean.

The installer does two things:
1. **Installs the Core Package:** `research-paper-workflow` is placed into the specific home directories of your AI clients (e.g. `~/.claude/skills/`, `~/.gemini/skills/`).
2. **Registers Slash Commands:** It drops lightweight symlinks into the client's discovery paths (e.g. `~/.claude/commands/paper.md` and `~/.gemini/workflows/lit-review.md`).

This means commands like `/paper` and `/study-design` become natively recognized by the AI engines **no matter what folder you are working in**.

_Project-local files (like `.env`) are only written when you explicitly run `rsk init --project-dir .`._

Home directory overrides:
- `CODEX_HOME`: root directory for Codex skill installation.
- `CLAUDE_CODE_HOME`: root directory for Claude Code skill installation.
- `GEMINI_HOME`: root directory for Gemini skill installation.
- `ANTIGRAVITY_HOME`: root directory for Antigravity global skill installation.

## Common flags

- `--install-cli`: install shell CLI commands (`research-skills`, `rsk`, `rsw`).
- `--no-cli`: skip shell CLI installation.
- `--cli-dir <path>`: choose where the shell CLI is installed (default: `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`).
- `--overwrite`: replace existing installation targets.
- `--dry-run`: preview installation actions only.
- `--doctor`: run `python3 -m bridges.orchestrator doctor --cwd <project>` after install when `python3` is available.

## Zero-Config Usage

Because commands are registered globally, using the system for a new paper is incredibly straightforward:

1. Create an empty directory for your new paper: `mkdir my-new-paper && cd my-new-paper`
2. Start the AI: `claude` or `gemini`
3. Execute a workflow directly: type `/paper` or `/lit-review`.

## Upgrade

- Check updates: `rsk check --repo <owner>/<repo>`
- Upgrade (no fork / no git clone required): `rsk upgrade --repo <owner>/<repo> --target all` for global refresh.
- Full guide: `guides/basic/upgrade-research-skills.md`

## Verify

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
