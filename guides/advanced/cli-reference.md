# CLI Command Reference (research-skills)

This document outlines all "executable entry points" (pipx CLI / Python module / Bash scripts) mapping local calls and GitHub CI configurations for the `research-skills` package.

## 0) Command Name Conventions

- `research-skills`: The main CLI (available after pipx/venv installation).
- `rs` / `rsw`: Short aliases (completely equivalent to `research-skills`).

The rest of this document will use `rs` as the example.

---

## 1) How Upstream Repositories are Resolved (Omitting `--repo`)

Many commands need to know "which GitHub repository to query/download releases from." The resolution order for `rs` upstream is as follows (highest to lowest priority):

1. CLI Argument: `--repo <owner/repo|Git URL>`
2. Environment Variable: `RESEARCH_SKILLS_REPO=<owner/repo|Git URL>`
3. Project Configuration File (searched upwards from the current directory or `--project-dir`):
   - `research-skills.toml`
   - `.research-skills.toml`
4. Package Default (inside the pipx installed package): `research_skills/project.toml` (Injected by CI during publishing)
5. If running inside a `research-skills` repository clone: Inferred from git remote (prioritizes `upstream`, then `origin`)

Supported repo formats:

- `owner/repo`
- `https://github.com/owner/repo.git`
- `git@github.com:owner/repo.git`

We highly recommend committing the upstream configuration to your project repository (useful for CI automation):

```toml
# research-skills.toml
[upstream]
repo = "owner/repo"   # Or url = "https://github.com/owner/repo.git"
```

---

## 2) `rs` (Installer & Updater CLI)

### 2.1 `rs check` (Check versions/Available updates)

Use Case:
- Outputs the CLI version, local repo version (if run from a clone), and installed versions across all 3 client directories.
- Optional: Queries the upstream latest release tag and determines if an upgrade is needed.

```bash
rs check [--repo <owner/repo|url>] [--json] [--strict-network]
```

Key Flags:
- `--repo`: Specify upstream (can be omitted, see "Upstream" section).
- `--json`: Output JSON only (useful for CI/Scripts).
- `--strict-network`: Return a failure code if upstream polling fails (defaults to warning and continuing).

Exit Codes:
- `0`: No updates available / upstream check bypassed.
- `1`: Update available.
- `2`: Invalid argument.

### 2.2 `rs upgrade` (Download release & execute installers)

Use Case:
- Downloads the upstream release (defaults to latest tag `.tar.gz`).
- Extracts it and executes `scripts/install_research_skill.sh`.

```bash
rs upgrade \
  [--repo <owner/repo|url>] \
  [--ref <tag-or-branch>] \
  [--ref-type tag|branch] \
  [--target codex|claude|gemini|all] \
  [--mode copy|link] \
  [--project-dir <path>] \
  [--no-overwrite] \
  [--doctor] \
  [--dry-run]
```

Notes:
- `--project-dir` tells the installer where to write the project-level integrations (e.g., `.agent/workflows/`, `CLAUDE.md`, `.gemini/`).
- `--mode link` is suitable for "maintaining a local clone" (symlink-based installation); `--mode copy` is best for one-off installs or CI.
- The command exits with the error code returned by the underlying bash installation script.

### 2.3 `rs align` (Quick Reference Guide)

Use Case: Prints an overview of "what pipx installed / paths modified by upgrades / common commands".

```bash
rs align [--repo <owner/repo|url>]
```

---

## 3) Orchestrator CLI: `python3 -m bridges.orchestrator`

This is the execution entry point for "Parallel Fallbacks & Task-Run Contract Execution".

```bash
python3 -m bridges.orchestrator <mode> [args...]
```

Available modes:

- `doctor`: Environment Preflight Checks
  ```bash
  python3 -m bridges.orchestrator doctor --cwd .
  ```
- `parallel`: 3-Agent Parallel Analysis + Synthesis (Auto-downgrades to dual/single if unavailable)
  ```bash
  python3 -m bridges.orchestrator parallel \
    --prompt "Analyze this study design" \
    --cwd . \
    --summarizer claude \
    --profile-file standards/agent-profiles.example.json \
    --profile default
  ```
- `task-run`: Standard pipeline execution via Task ID (plan -> evidence -> draft -> review -> gates -> write to RESEARCH/)
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F3 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --triad
  ```
  Common parameters:
  - `--venue <name>` / `--context <text>`
  - `--mcp-strict` / `--skills-strict`
  - `--profile-file <path>` + `--profile <name>` (along with `--draft-profile` / `--review-profile` / `--triad-profile`)
- `task-plan`: Renders the dependency execution order based on the contract
  ```bash
  python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic your-topic --cwd .
  ```
- `code-build`: Research Code builder entry point
  ```bash
  python3 -m bridges.orchestrator code-build --method "Staggered DID" --cwd . --domain econ
  ```
- `single`: Single-model execution (Quick debug/runs)
  ```bash
  python3 -m bridges.orchestrator single --prompt "..." --cwd . --model codex
  ```
- `chain`: Iterative refinement (One builds, the other verifies)
  ```bash
  python3 -m bridges.orchestrator chain --prompt "..." --cwd . --generator codex
  ```
- `role`: Execution split by specialized roles
  ```bash
  python3 -m bridges.orchestrator role --cwd . --codex-task "..." --claude-task "..." --gemini-task "..."
  ```

---

## 4) Bash Scripts (Non-pipx)

### 4.1 Installer Script: `./scripts/install_research_skill.sh`

```bash
./scripts/install_research_skill.sh \
  --target all \
  --mode copy \
  --project-dir /path/to/project \
  --overwrite \
  --doctor
```

### 4.2 Release Automation: `./scripts/release_automation.sh`

```bash
./scripts/release_automation.sh pre  --tag v0.1.0-beta.X
./scripts/release_automation.sh post --tag v0.1.0-beta.X
./scripts/release_automation.sh full --tag v0.1.0-beta.X
```

Also executable individually:

```bash
./scripts/release_preflight.sh [--tag v0.1.0-beta.X] [--skip-smoke] [--no-strict]
./scripts/release_postflight.sh --tag v0.1.0-beta.X [--skip-remote] [--skip-ci-status]
```

### 4.3 Beta smoke tests: `./scripts/run_beta_smoke.sh`

```bash
./scripts/run_beta_smoke.sh
```

### 4.4 CI Default Upstream Injector: `./scripts/inject_project_toml.sh`

Executed by GitHub actions during packaging to hardcode the repo slug into `research_skills/project.toml`.

```bash
bash scripts/inject_project_toml.sh

# Or override the repo slug dynamically during builds
RESEARCH_SKILLS_REPO_SLUG="other-owner/other-repo" bash scripts/inject_project_toml.sh
```

---

## 5) Validators (Recommended before CI/Deployment)

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

Project Artifact Validator (run inside your actual project output directory):

```bash
python3 scripts/validate_project_artifacts.py \
  --cwd /path/to/project \
  --topic your-topic \
  --task-id H1 \
  --strict
```
