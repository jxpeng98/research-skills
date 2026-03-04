# Upgrade / Auto-Upgrade Guide (No Fork Required)

This guide explains how to:
1) Check for new versions.
2) Automate the upgrade process.
3) Complete the upgrade without needing to fork or `git clone` the repository.

## 0) Standardize as a pip package (Recommended)

This repository provides a `pyproject.toml` file, allowing it to be published and installed as a standard pip package (using `pipx` to install the CLI is highly recommended).

After publishing to PyPI (or an internal registry), the standard usage on your machine is:

```bash
pipx install research-skills-installer
# This provides 3 equivalent commands (choose any):
# - research-skills
# - rsk
# - rsw
# You can also set `RESEARCH_SKILLS_REPO=<owner>/<repo>` to omit the --repo flag
rsk check --repo <owner>/<repo>
rsk upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all --doctor
```

> Note: pip installs/upgrades the "updater CLI." The actual process of overwriting the skill/workflow files into the client directories and your project is still performed by `rsk upgrade` (or `research-skills upgrade`). This keeps the process explicit and prevents background file modifications during a pip install.

## 1) What exactly are you upgrading?

This project has two types of "installation targets":

- **Local skill directories for the 3 clients** (so Codex / Claude Code / Gemini recognize the skill)
  - Codex: `${CODEX_HOME:-~/.codex}/skills/research-paper-workflow`
  - Claude: `${CLAUDE_CODE_HOME:-~/.claude}/skills/research-paper-workflow`
  - Gemini: `${GEMINI_HOME:-~/.gemini}/skills/research-paper-workflow`
- **In-project integration files** (so Claude Code's `/...` commands work inside your project)
  - `<project>/.agent/workflows/*.md`
  - `<project>/CLAUDE.md` (or `CLAUDE.research-skills.md`)
  - `<project>/.gemini/research-skills.md`

Upgrading simply means **overwriting these target paths with the new version** (which usually requires `--overwrite`).

---

## 2) Checking for new versions (Recommended)

```bash
# If RESEARCH_SKILLS_REPO is set, --repo can be omitted
rsk check --repo <owner>/<repo>
# Or run within the repository (equivalent):
python3 scripts/research_skill_update.py check --repo <owner>/<repo>
```

Details:
- `--repo` is used to query the latest GitHub release tag.
- If it detects that "local/installed version < latest version", the command returns exit code `1` (which is useful for automation scripts).
- You can set a default upstream to omit `--repo`:
  - Envrionment variable: `export RESEARCH_SKILLS_REPO=<owner>/<repo>`
  - If you run this inside a `research-skills` clone with a configured git remote (prioritizes `upstream`, then `origin`), `--repo` can be omitted.
  - Or add a `research-skills.toml` file to your project root (easy to commit to your project repo, great for CI).

Example (in project root):

```toml
# research-skills.toml
[upstream]
repo = "<owner>/<repo>" # Or Git URL
```

Afterward, you can run:

```bash
rsk check
rsk upgrade --project-dir . --target all --doctor
```

---

## 3) Automatic Upgrade (No fork, no git clone required)

This directly downloads the GitHub release archive and executes the installation script inside it:

```bash
# If RESEARCH_SKILLS_REPO is set, --repo can be omitted
rsk upgrade \
  --repo <owner>/<repo> \
  --project-dir /path/to/your/project \
  --target all \
  --mode copy \
  --doctor

# Or run within the repository (equivalent):
python3 scripts/research_skill_update.py upgrade \
  --repo <owner>/<repo> \
  --project-dir /path/to/your/project \
  --target all \
  --mode copy \
  --doctor
```

Key points:
- This method **does not rely on git** and does not require you to clone the repository locally.
- For private repositories or if you hit API rate limits, it is recommended to set: `GITHUB_TOKEN` or `GH_TOKEN`.
- It defaults to using the "latest release tag", but you can specify a version:
  - `--ref v0.1.0-beta.6 --ref-type tag`
  - `--ref main --ref-type branch`

It is recommended to restart your clients (Codex / Claude Code / Gemini CLI) after upgrading.

---

## 4) Alternative "Auto-Upgrade": Link Installation + Git Pull (Best for long-term maintenance)

If you are willing to keep a local clone of the repository (no fork needed, just clone once), this is recommended:

1) When installing, use `--mode link` (creates symlinks, meaning future updates don't require re-running the install script):

```bash
./scripts/install_research_skill.sh --target all --mode link --project-dir /path/to/project --overwrite
```

2) When updating, simply run:

```bash
git pull
```

Because the installation targets are symlinks, updating the repository contents automatically syncs the skill and workflows to the latest version across all 3 clients.

---

## 5) Automation Suggestions (Optional)

You can use cron/CI to do a "weekly check + upgrade if available":

1) Check periodically:
```bash
rsk check --repo <owner>/<repo>
```
2) If exit code is 1, execute upgrade:
```bash
rsk upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all
```

If you want this upgrade detection integrated as a Codex Automation (run periodically and generate inbox results), just let me know the run frequency and target project paths.
