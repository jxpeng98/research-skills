# CLI Command Reference (research-skills)

This document outlines all "executable entry points" (pipx CLI / Python module / Bash scripts) mapping local calls and GitHub CI configurations for the `research-skills` package.

## 0) Command Name Conventions

- `research-skills`: The main CLI (available after pipx/venv installation, or after shell bootstrap install).
- `rsk` / `rsw`: Short aliases (completely equivalent to `research-skills`).

The rest of this document will use `rsk` as the example.

---

## 1) How Upstream Repositories are Resolved (Omitting `--repo`)

Many commands need to know "which GitHub repository to query/download releases from." The resolution order for `rsk` upstream is as follows (highest to lowest priority):

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

## 2) `rsk` (Installer & Updater CLI)

There are two distributions of this CLI:
- Python CLI: installed via `pip`/`pipx`
- Shell CLI: installed by `bootstrap_research_skill.sh` into `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}` by default

### 2.1 `rsk check` (Check versions/Available updates)

Use Case:
- Outputs the CLI version, local repo version (if run from a clone), and installed versions across all 3 client directories.
- Optional: Queries the upstream latest release tag and determines if an upgrade is needed.

```bash
rsk check [--repo <owner/repo|url>] [--json] [--strict-network]
```

Key Flags:
- `--repo`: Specify upstream (can be omitted, see "Upstream" section).
- `--json`: Output JSON only (useful for CI/Scripts).
- `--strict-network`: Return a failure code if upstream polling fails (defaults to warning and continuing).

Exit Codes:
- `0`: No updates available / upstream check bypassed.
- `1`: Update available.
- `2`: Invalid argument.

### 2.2 `rsk upgrade` (Download release & execute installers)

Use Case:
- Downloads the upstream release (defaults to latest tag `.tar.gz`).
- Extracts it and executes `scripts/install_research_skill.sh`.

```bash
rsk upgrade \
  [--repo <owner/repo|url>] \
  [--ref <tag-or-branch>] \
  [--ref-type tag|branch] \
  [--target codex|claude|gemini|antigravity|all] \
  [--project-dir <path>] \
  [--no-overwrite] \
  [--doctor] \
  [--dry-run]
```

Notes:
- `--project-dir` tells the installer where to write the project-level integrations (e.g., `.agent/workflows/`, `.agents/skills/`, `CLAUDE.md`, `.gemini/`).
- Shell CLI uses the bundled bootstrap helper and does not require Python.
- Treat shell-CLI `upgrade` as copy-mode refresh. If you need symlink-based `link` installs, use the local installer directly.
- The command exits with the error code returned by the underlying installer.

### 2.3 `rsk align` (Quick Reference Guide)

Use Case: Prints an overview of "what pipx installed / paths modified by upgrades / common commands".

```bash
rsk align [--repo <owner/repo|url>]
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
  - `--domain <name>`: inject a runtime domain profile (for example `econ`, `cs`, `psychology`) into the task packet and prompts
  - `--venue <name>` / `--context <text>`
  - `--mcp-strict` / `--skills-strict`
  - `--profile-file <path>` + `--profile <name>` (along with `--draft-profile` / `--review-profile` / `--triad-profile`)
  - `--focus-output <path>` (repeatable) + `--output-budget <n>`: narrow this run to a smaller active output set and defer the rest of the contract outputs explicitly
  - `--research-depth standard|deep` + `--max-rounds <n>`: increase evidence-expansion pressure and enforce a deeper review/revision loop
  - `--only-target <id>` (repeatable): for structured Stage-I tasks `I4`-`I8`, reload the existing artifact under `RESEARCH/[topic]/code/` and rerun only the named actionable targets
  - Built-in profiles now include `focused-delivery` and `deep-research` in addition to `default`, `rapid-draft`, and `strict-review`

  Example: reduce artifact sprawl but keep stronger review pressure
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F3 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --focus-output manuscript/manuscript.md \
    --research-depth deep \
    --draft-profile deep-research \
    --review-profile strict-review \
    --triad-profile deep-research \
    --triad \
    --max-rounds 4
  ```
  Example: rerun only a blocked Stage-I planning step
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id I6 \
    --paper-type methods \
    --topic llm-bias \
    --cwd . \
    --only-target S1
  ```
- `task-plan`: Renders the dependency execution order based on the contract
  ```bash
  python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic your-topic --cwd .
  ```
- `code-build`: Academic code workflow entry point
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Staggered DID" \
    --topic policy-effects \
    --domain econ \
    --focus full \
    --cwd .
  ```
  Key parameters:
  - `--topic <slug>`: when present, `code-build` routes into strict Stage-I workflow instead of the legacy prompt-only path
  - `--focus <name>`: map into `I1`/`I2`/`I3`/`I4`/`I5`/`I6`/`I7`/`I8`, or use `full` for `I5 -> I6 -> I7 -> I8`
  - `--domain <name>`: inject the matching `skills/domain-profiles/*.yaml`
  - `--paper-type <type>`: workflow paper type used by strict Stage-I routing
  - `--triad`: add the third independent audit on the final strict review pass
  - `--paper <path-or-url>`: optional paper reference carried into the task context
  - `--only-target <selector>` (repeatable): targeted follow-up mode
    - single-stage focus: use bare target IDs such as `S1` or `P1-01`
    - `--focus full`: use `STAGE_ID:TARGET` selectors such as `I5:decision-1` or `I8:P1-01`

  Example: run only the spec phase for an advanced CS method
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Transformer Fine-Tuning" \
    --topic llm-bias \
    --domain cs \
    --tier advanced \
    --focus code_specification \
    --paper-type methods \
    --cwd .
  ```
  Example: rerun only specific full-flow targets
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Transformer Fine-Tuning" \
    --topic llm-bias \
    --domain cs \
    --focus full \
    --only-target I5:decision-1 \
    --only-target I8:P1-01 \
    --cwd .
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

### 4.1 Remote Bootstrap Installer: `./scripts/bootstrap_research_skill.sh`

Use case:
- Install or refresh skills on machines without Python.
- Downloads a GitHub release/branch archive, extracts it, and then runs `scripts/install_research_skill.sh` from that archive.

```bash
./scripts/bootstrap_research_skill.sh \
  --repo owner/repo \
  --target all \
  --project-dir /path/to/project \
  --overwrite
```

Notes:
- Requires `bash` and either `curl` or `wget`, plus `tar`.
- Supports `--ref <tag-or-branch>` with `--ref-type tag|branch`.
- Installs shell CLI commands by default: `research-skills`, `rsk`, `rsw`.
- Use `--no-cli` to skip shell CLI installation, or `--cli-dir <path>` to choose the install location.
- Remote bootstrap supports `--mode copy` only.
- `--doctor` auto-skips when `python3` is unavailable.

### 4.2 Installer Script: `./scripts/install_research_skill.sh`

```bash
./scripts/install_research_skill.sh \
  --target all \
  --mode copy \
  --project-dir /path/to/project \
  --install-cli \
  --overwrite \
  --doctor
```

Notes:
- This is the local-repository installer.
- The copy/link install path no longer requires Python.
- Add `--install-cli` to also install the shell CLI into `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}` or `--cli-dir <path>`.
- `--doctor` runs `python3 -m bridges.orchestrator doctor --cwd <project>` only when `python3` exists.

### 4.3 Release Automation: `./scripts/release_automation.sh`

```bash
./scripts/release_automation.sh publish --version 0.1.0 --from-tag v0.1.0-beta.X
./scripts/release_automation.sh pre  --tag v0.1.0-beta.X
./scripts/release_automation.sh post --tag v0.1.0-beta.X --create-release
```

Recommended:

- use `publish` for the full end-to-end release path
- use `pre` / `post` only when you need to split release prep and release acceptance into separate phases manually
- stable releases publish from the matching `CHANGELOG.md` section
- beta / prerelease releases publish from `release/<tag>.md`

Also executable individually:

```bash
./scripts/release_preflight.sh [--tag v0.1.0-beta.X] [--skip-smoke] [--maintainer-smoke] [--no-strict]
./scripts/release_postflight.sh --tag v0.1.0-beta.X [--skip-remote] [--skip-ci-status] [--wait-ci] [--create-release]
```

### 4.4 Beta smoke tests: `./scripts/run_beta_smoke.sh`

```bash
./scripts/run_beta_smoke.sh
./scripts/run_beta_smoke.sh --tier release
./scripts/run_beta_smoke.sh --tier maintainer
```

This smoke entrypoint supports two tiers:

- `release`: built-in literature pipeline smoke + `doctor`
- `maintainer`: everything in `release`, plus `parallel` and `task-run` profile-path checks

Release preflight now uses the `release` tier by default. Use `--maintainer-smoke` in release tooling when you explicitly want the heavier maintainer checks.

### 4.5 Literature smoke: `./scripts/run_literature_smoke.sh`

```bash
./scripts/run_literature_smoke.sh
```

### 4.6 CI Default Upstream Injector: `./scripts/inject_project_toml.sh`

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
