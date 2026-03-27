# Multi-Client Install Guide (Codex / Claude Code / Gemini)

::: warning Full Functionality Requirement
Portable install does not require Python, but full functionality does.

For the complete system, install and configure:

- `python3`
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`

Without them, installation and shell `rsk` maintenance commands still work, but orchestrator execution, `doctor`, validators, and full multi-model flows will be incomplete.
:::

## 0. Preliminary: Install Python First (Recommended)

Python is mainly for the orchestrator runtime. If you only want to install workflow assets into a project, `partial` install can work without Python. If you want `doctor`, validators, and `python3 -m bridges.orchestrator ...`, install `Python >= 3.12`. The packaged Python CLI in this repo currently declares `requires-python = ">=3.12"`.

Recommended path: use `mise`.

If `mise` is not installed yet, install and activate it first:

```bash
# Linux / macOS
curl https://mise.run | sh
```

```bash
# bash
echo 'eval "$(mise activate bash)"' >> ~/.bashrc
source ~/.bashrc
```

```bash
# zsh
echo 'eval "$(mise activate zsh)"' >> "${ZDOTDIR-$HOME}/.zshrc"
source "${ZDOTDIR-$HOME}/.zshrc"
```

```powershell
# Windows (PowerShell)
scoop install mise
```

```powershell
# Windows alternative
winget install jdx.mise
```

```bash
mise install python@3.12
mise use -g python@3.12
python3 --version
```

Notes:
- If `mise` is not available on the machine yet, preinstall `mise` before running the Python commands above.
- This enables `python3 -m bridges.orchestrator ...`, `--doctor`, validators, and tests immediately.
- Even if you start with the shell bootstrapper, pre-installing Python avoids a second round of environment setup when you later need upgrade or troubleshooting flows.

## 1. Zero-Python Bootstrap (Recommended For First Install)

If the machine may not have Python yet, start with a bootstrap entrypoint. If you omit `--profile`, the script explains `partial` vs `full` and prompts you to choose.

Unified profile behavior:

| Profile | What it installs | Python required before install | Orchestrator ready after install |
|---|---|---|---|
| `partial` | skills, workflows, project integration files | No | No |
| `full` | `partial` + shell CLI + Python 3.12 via `mise` when needed + `doctor` | No | Yes |

Linux / macOS, one-click bootstrap:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --project-dir /path/to/project \
  --target all
```

Windows PowerShell, one-click bootstrap:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -ProjectDir "C:\path\to\project" -Target all
```

Profile behavior:
- `partial`: install workflow assets and project integration files only.
- `full`: install workflow assets, enable shell CLI installation, ensure Python 3.12 is available via `mise`, and run doctor so the orchestrator runtime is ready. On Windows bootstrap, PowerShell now performs the install directly and only installs Git for Windows via `winget` when shell CLI wrappers need Bash.

## 2. Cross-Platform Python Installer (Optional Once Python Exists)

Once Python is available, you can also use the local Python installer. It works on Linux, macOS, and Windows without requiring Bash.

Partial install:

```bash
python3 scripts/bootstrap_research_skill.py --profile partial --project-dir .
```

Full install:

```bash
python3 scripts/bootstrap_research_skill.py --profile full --project-dir .
```

Profile behavior:
- `partial`: install workflow assets and project integration files only.
- `full`: install workflow assets, attempt shell CLI installation when supported, print readiness hints, and run doctor.

Notes:
- The shell bootstrap path still requires `bash` on Linux/macOS.
- `rsk upgrade` is now Python-based and does not require Bash.
- On Windows, the PowerShell bootstrap performs the install directly and installs Git for Windows via `winget` only when shell CLI wrappers need Bash.
- The shell bootstrap path installs shell CLI commands by default in `full` mode and skips them in `partial` mode.

## 3. Optional Python CLI

If Python is already available on the machine, you can install the updater CLI with `pipx`:

```bash
pipx install research-skills-installer
rsk upgrade --target all --project-dir /path/to/project --doctor
```

This `pip` / `pipx` path is retained as an optional distribution for the updater CLI. It is not the recommended first-install path anymore.

## 4. Local Repository Installer

If you already have a repository checkout, you can run the installer directly:

```bash
./scripts/install_research_skill.sh --profile partial --target all --project-dir /path/to/project
./scripts/install_research_skill.sh --profile full --target all --project-dir /path/to/project
```

## Target behaviors

- project-level defaults
  - Copies `.env.example` to `<project>/.env` by default so the project starts with an editable runtime config file.
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
- `antigravity`
  - Checks whether the `antigravity` CLI is available on `PATH` before global installation.
  - Installs the workspace skill into `<project>/.agents/skills/research-paper-workflow`.
  - Installs the backward-compatible workspace skill into `<project>/.agent/skills/research-paper-workflow`.
  - Installs the global skill into `${ANTIGRAVITY_HOME:-~/.gemini/antigravity}/skills/research-paper-workflow` when the CLI is available.

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
