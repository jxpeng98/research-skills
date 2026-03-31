# Multi-Client Install Guide (Codex / Claude Code / Gemini)

::: warning Full Functionality Requirement
You do not need to preinstall Python for first install anymore, but the full runtime still depends on:

- `python3`
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`

Without them, asset installation and shell `rsk` maintenance commands still work, but orchestrator execution, `doctor`, validators, and full multi-model flows will be limited.
:::

## 1. Choose `partial` Or `full`

The bootstrap installers now explain these choices interactively if you omit `--profile`.

| Profile | What it installs | Python required before install | Result after install |
|---|---|---|---|
| `partial` | skills, workflows, project integration files | No | Assets are ready; orchestrator is not |
| `full` | `partial` + shell CLI + Python 3.12 when needed + `doctor` | No | Orchestrator runtime is ready |

How `full` works:

- If `python3 >= 3.12` already exists, bootstrap reuses it.
- If Python is missing or too old, bootstrap installs `mise`, then installs `python@3.12`.
- On Windows, PowerShell installs directly and only installs Git for Windows via `winget` when shell CLI wrappers need Bash.

## 2. Run The Recommended One-Click Bootstrap

### Linux / macOS

Prompt for `partial` or `full`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --project-dir "$PWD" --target all
```

Force `partial`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Force `full`:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Install the latest beta / prerelease:

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --beta --profile full --project-dir "$PWD" --target all
```

### Windows PowerShell

Download and prompt for `partial` or `full`:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -ProjectDir "$PWD" -Target all
```

Force `partial`:

```powershell
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

Force `full`:

```powershell
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

Install the latest beta / prerelease:

```powershell
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Beta -Profile full -ProjectDir "$PWD" -Target all
```

What bootstrap installs:

- workflow assets for Codex / Claude Code / Gemini
- project integration files such as `.agent/workflows/`, `CLAUDE.md`, `.gemini/`
- shell CLI commands `research-skills`, `rsk`, `rsw` in `full` mode

## 3. Optional: Prepare Python Yourself With `mise`

You only need this if you want to manage Python manually instead of letting `full` bootstrap handle it.

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
# Windows
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

## 4. Optional Local Installers

If Python is already available, you can use the local cross-platform Python installer:

```bash
python3 scripts/bootstrap_research_skill.py --profile partial --project-dir .
python3 scripts/bootstrap_research_skill.py --profile full --project-dir .
```

If you already have a local repository checkout on Linux or macOS, you can also use the local shell installer:

```bash
./scripts/install_research_skill.sh --profile partial --target all --project-dir /path/to/project
./scripts/install_research_skill.sh --profile full --target all --project-dir /path/to/project
```

The `pip` / `pipx` path is still available for the updater CLI, but it is no longer the recommended first-install path:

```bash
pipx install research-skills-installer
rsk upgrade --target all --project-dir /path/to/project --doctor
```

## 5. Target Behaviors

- Project defaults
  - Copies `.env.example` to `<project>/.env`.
- `codex`
  - Installs `research-paper-workflow` into `${CODEX_HOME:-~/.codex}/skills/research-paper-workflow`.
- `claude`
  - Installs `research-paper-workflow` into `${CLAUDE_CODE_HOME:-~/.claude}/skills/research-paper-workflow`.
  - Copies `.agent/workflows/*.md` into `<project>/.agent/workflows/`.
  - Copies `CLAUDE.md` to `<project>/CLAUDE.md`, or `CLAUDE.research-skills.md` if `CLAUDE.md` already exists and `--overwrite` is not used.
- `gemini`
  - Installs `research-paper-workflow` into `${GEMINI_HOME:-~/.gemini}/skills/research-paper-workflow`.
  - Copies `.gemini/research-skills.md` into `<project>/.gemini/research-skills.md`.
  - Copies `standards/agent-profiles.example.json` to `<project>/.gemini/agent-profiles.example.json`.
- `antigravity`
  - Installs workspace skill into `<project>/.agents/skills/research-paper-workflow`.
  - Installs backward-compatible workspace skill into `<project>/.agent/skills/research-paper-workflow`.
  - Installs global skill into `${ANTIGRAVITY_HOME:-~/.gemini/antigravity}/skills/research-paper-workflow` when the `antigravity` CLI is available.

## 6. Common Flags

- `--profile partial|full`: choose the install preset explicitly instead of using the prompt.
- `--target codex|claude|gemini|antigravity|all`: limit installation scope.
- `--beta`: install the latest beta / prerelease tag when `--ref` is omitted.
- `--mode copy|link`: copy files or create symlinks. Bootstrap uses `copy`.
- `--install-cli`: install shell CLI commands even outside `full`.
- `--no-cli`: skip shell CLI installation even in `full`.
- `--cli-dir <path>`: choose where the shell CLI is installed. Default: `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`.
- `--overwrite`: replace existing installation targets.
- `--dry-run`: preview installation actions only.
- `--doctor`: run `python3 -m bridges.orchestrator doctor --cwd <project>` after install.

## 7. Upgrade And Verify

Refresh an existing install:

```bash
rsk upgrade --target all --project-dir . --doctor
```

Verify readiness:

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```

Python boundary:

- shell `rsk check|upgrade|align` do not require Python
- `--doctor`, `python3 -m bridges.orchestrator ...`, validators, and tests still require Python
