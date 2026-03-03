# PyPI Package Publishing Guide

This guide explains how to publish `research-skills-installer` to PyPI, as well as the complete workflow for routine version releases.

## 0) Prerequisites (One-time Setup)

### 0.1 PyPI Trusted Publisher

This project uses the [Trusted Publisher](https://docs.pypi.org/trusted-publishers/) mechanism for publishing (no manual API Token management needed).

1. Log in to your account at [pypi.org](https://pypi.org).
2. If this is the **first time publishing** (the package does not exist on PyPI yet), go to the [Publishing](https://pypi.org/manage/account/publishing/) page. Under "Add a new pending publisher", fill in the following:
   - **PyPI Project Name**: `research-skills-installer`
   - **Owner**: `<your-github-username>`
   - **Repository name**: `research-skills`
   - **Workflow name**: `publish-pypi.yml`
   - **Environment name**: `pypi`
3. If the **package already exists**, go to the package's Settings → Publishing → "Add a new publisher", and fill in the same details.

### 0.2 GitHub Environment

1. Go to your GitHub repository Settings → Environments.
2. Click "New environment" and name it `pypi`.
3. (Optional) Add protection rules (e.g., only allow deployment from the `main` branch, require approval, etc.).

---

## 1) Routine Publishing Workflow

### 1.1 Update the Version Number

Use the `bump-version.sh` script to simultaneously update `pyproject.toml` and `research_skills/__init__.py`:

```bash
./scripts/bump-version.sh 0.2.0
```

Version string format strictly follows [PEP 440](https://peps.python.org/pep-0440/):

| Stage | Format | Example |
|------|------|------|
| Beta | `X.Y.ZbN` | `0.1.0b7` |
| RC | `X.Y.ZrcN` | `0.1.0rc1` |
| Release | `X.Y.Z` | `0.1.0`, `1.0.0` |

### 1.2 Commit + Tag + Push

```bash
git add pyproject.toml research_skills/__init__.py
git commit -m "chore: bump version to 0.2.0"
git tag v0.2.0
git push origin main --tags
```

> **Note**: The tag format MUST start with `v*` (e.g., `v0.2.0`, `v0.1.0b7`) for the GitHub Actions `publish-pypi.yml` workflow to trigger.

### 1.3 Automatic Build & Publish

After pushing the tag, GitHub Actions will automatically:

1. Checkout the code.
2. Run `inject_project_toml.sh` (injects the current repository slug into `research_skills/project.toml` so the installed CLI knows its upstream default).
3. `python -m build` to build the sdist and wheel.
4. `twine check` to validate package metadata.
5. Publish to PyPI using the Trusted Publisher mechanism.

You can monitor the progress on the **Actions** page of your GitHub repository.

---

## 2) Local Verification (Optional Before Publishing)

If you want to confirm the package builds correctly locally before pushing:

```bash
# Install build tools
pip install build twine

# Inject upstream repo info
bash scripts/inject_project_toml.sh

# Build
python -m build

# Validate
twine check dist/*
```

Local dry-run installation:

```bash
pip install dist/research_skills_installer-*.whl
research-skills --help
rs check --repo <owner>/<repo>
```

---

## 3) Publishing to TestPyPI (Optional)

If you wish to publish to TestPyPI first for testing, you can upload manually:

```bash
# Register at test.pypi.org and obtain an API token first
twine upload --repository testpypi dist/*

# Install and test from TestPyPI
pip install --index-url https://test.pypi.org/simple/ research-skills-installer
```

---

## 4) Complete Release Checklist

When cutting a release, follow these steps:

- [ ] Confirm all features are merged into `main`.
- [ ] Ensure CI is passing (Green `ci.yml`).
- [ ] Run release preflight: `./scripts/release_automation.sh pre --tag v<version>`
- [ ] Update version number: `./scripts/bump-version.sh <version>`
- [ ] Commit version changes.
- [ ] Create a tag: `git tag v<version>`
- [ ] Push: `git push origin main --tags`
- [ ] Confirm the `Publish to PyPI` workflow succeeded on GitHub Actions.
- [ ] Run release postflight: `./scripts/release_automation.sh post --tag v<version>`
- [ ] Verify installation: `pipx install research-skills-installer && rs --help`

---

## 5) Frequently Asked Questions (FAQ)

### Q: I pushed a tag but Actions did not trigger?

Ensure the tag format starts with `v` (e.g., `v0.1.0b7`) and that the `.github/workflows/publish-pypi.yml` workflow file is present on the `main` branch.

### Q: PyPI publishing failed with "403 Forbidden"?

This is typically an issue with the Trusted Publisher setup:
- Make sure the workflow name on PyPI exactly matches `publish-pypi.yml`.
- Make sure the GitHub environment name exactly matches `pypi`.
- Verify the owner and repository names are correct.

### Q: Upload failed because the version number already exists?

PyPI does not allow overwriting published versions. If you need to ship a fix, you must increment the version number (e.g., `0.1.0b7` → `0.1.0b8`).

### Q: How do I recall/withdraw a published version?

You can "yank" a version from the PyPI project page (it will not be deleted from users who already installed it, but new `pip install` commands will skip a yanked version by default).
