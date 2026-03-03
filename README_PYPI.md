# research-skills-installer

`research-skills-installer` is a lightweight CLI for installing and upgrading **Research Skills** assets in your project for Codex, Claude Code, and Gemini workflows.

## What it does

- Install workflow/skill assets into your project
- Upgrade assets to newer upstream versions
- Support `codex`, `claude`, `gemini`, or `all` targets
- Run doctor checks before/after installation

## Installation

```bash
pip install research-skills-installer
```

Or with `pipx`:

```bash
pipx install research-skills-installer
```

## CLI

Main command and aliases:

- `research-skills`
- `rs`
- `rsw`

### Check updates

```bash
rs check
```

### Upgrade assets

```bash
rs upgrade --project-dir /path/to/project --target all --doctor
```

The package includes a default upstream repo (`jxpeng98/research-skills`), so `--repo` is optional.
Use `--repo` only when you want to override the default.

## Override default repo (optional)

The CLI resolves upstream repo in this order:

1. `--repo` argument
2. `RESEARCH_SKILLS_REPO` environment variable
3. `research-skills.toml` or `.research-skills.toml` in your project path
4. Packaged default (`research_skills/project.toml`)

### Option A: Global override

Add this to your shell profile (`~/.zshrc`, `~/.bashrc`, etc.):

```bash
export RESEARCH_SKILLS_REPO="<owner>/<repo>"
```

Then reload shell:

```bash
source ~/.zshrc
```

Now you can run:

```bash
rs check
rs upgrade --project-dir /path/to/project --target all --doctor
```

### Option B: Project-level override

Create `research-skills.toml` in your project root:

```toml
[upstream]
repo = "jxpeng98/research-skills"
url = "https://github.com/<owner>/<repo>"
```

This keeps the override local to that project.

## Typical usage

```bash
# Install from PyPI
pipx install research-skills-installer

# Upgrade assets into your project
rs upgrade --project-dir /path/to/project --target all --doctor
```

## Links

- Repository: https://github.com/jxpeng98/research-skills
- Issues: https://github.com/jxpeng98/research-skills/issues
