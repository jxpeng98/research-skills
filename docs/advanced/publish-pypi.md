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

### 0.3 TestPyPI Trusted Publisher

This repository includes a dedicated TestPyPI workflow: `.github/workflows/publish-testpypi.yml`.

1. Log in to [test.pypi.org](https://test.pypi.org).
2. Go to Account settings → Publishing.
3. Add a pending publisher (or add a publisher under the existing project) with:
   - **PyPI Project Name**: `research-skills-installer`
   - **Owner**: `jxpeng98`
   - **Repository name**: `research-skills`
   - **Workflow name**: `publish-testpypi.yml`
   - **Environment name**: `testpypi`
4. In GitHub repository Settings → Environments, create environment `testpypi`.

---

## 1) Routine Publishing Workflow

### 1.1 Prepare a Publish-Ready Release

Recommended one-command local flow:

```bash
./scripts/release_ready.sh --version 0.2.0
./scripts/release_ready.sh --version 0.2.0b1 --from-tag v0.2.0
```

`release_ready.sh` chains these steps:

- `bump-version.sh` to sync release versions across:

- `pyproject.toml`
- `research_skills/__init__.py`
- `research-paper-workflow/VERSION`
- `skills/registry.yaml`

- `release_automation.sh pre` to run strict validator, repository unit tests, release smoke, and release note draft generation
- `pypi_preflight.sh` to build the package, run `twine check`, and install-smoke the generated wheel

Pass either a stable version such as `0.2.0` or a beta version such as `0.2.0b1`.
The version sync step normalizes it into three synchronized forms:

| Layer | Stable | Beta |
|------|------|------|
| PyPI package | `0.2.0` | `0.2.0b1` |
| Skill metadata / registry | `0.2.0` | `0.2.0-beta.1` |
| Portable skill `VERSION` / git tag | `v0.2.0` | `v0.2.0-beta.1` |

Package version format follows [PEP 440](https://peps.python.org/pep-0440/), while skill metadata uses SemVer-compatible prerelease syntax.

Currently the release tooling supports `stable` and `beta` only.

If you need the old manual entrypoints, they still exist:

```bash
./scripts/bump-version.sh 0.2.0
./scripts/release_automation.sh pre --tag v0.2.0
bash scripts/pypi_preflight.sh
```

### 1.2 Commit + TestPyPI Validation

```bash
git add pyproject.toml research_skills/__init__.py research-paper-workflow/VERSION skills/registry.yaml skills release/v0.2.0.md
git commit -m "chore: prepare release 0.2.0"
```

Then run the GitHub Actions `Publish to TestPyPI` workflow and verify install / CLI behavior from TestPyPI before creating the production tag.

---

## 2) Tag + Publish to PyPI

After TestPyPI validation passes:

```bash
git tag v0.2.0
git push origin main --tags
```

> **Note**: The tag format MUST start with `v*` and use repo release syntax (for example `v0.2.0` or `v0.2.0-beta.1`) for the GitHub Actions `publish-pypi.yml` workflow to trigger.

After pushing the tag, GitHub Actions will automatically:

1. Checkout the code.
2. Run `inject_project_toml.sh` (injects the current repository slug into `research_skills/project.toml` so the installed CLI knows its upstream default).
3. `python -m build` to build the sdist and wheel.
4. `twine check` to validate package metadata.
5. Publish to PyPI using the Trusted Publisher mechanism.

You can monitor the progress on the **Actions** page of your GitHub repository.

---

## 3) Local Verification (Manual / Optional)

If you want to run package checks outside `release_ready.sh`, use:

```bash
bash scripts/pypi_preflight.sh
bash scripts/pypi_preflight.sh --no-build
```

Equivalent manual build steps:

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
rsk check --repo <owner>/<repo>
```

---

## 4) Publishing to TestPyPI (Recommended Before Production)

Use the GitHub Actions workflow (manual trigger, no tag required):

1. Open GitHub Actions.
2. Select **Publish to TestPyPI**.
3. Click **Run workflow** on the target branch.

The workflow will build, validate, and publish with Trusted Publishing to TestPyPI.

Install and verify from TestPyPI:

```bash
pip install --index-url https://test.pypi.org/simple/ research-skills-installer
```

Recommended order:

- Run **Publish to TestPyPI** and validate install/CLI behavior.
- After validation passes, push release tag `v*` to trigger **Publish to PyPI**.

---

## 5) Complete Release Checklist

When cutting a release, follow these steps:

- [ ] Confirm all features are merged into `main`.
- [ ] Ensure CI is passing (Green `ci.yml`).
- [ ] Run `./scripts/release_ready.sh --version <version>`.
- [ ] Commit release-prep changes.
- [ ] Run GitHub Actions `Publish to TestPyPI` and validate package installation from TestPyPI.
- [ ] Create a tag: `git tag v<version>`
- [ ] Push: `git push origin main --tags`
- [ ] Confirm the `Publish to PyPI` workflow succeeded on GitHub Actions.
- [ ] Run release postflight: `./scripts/release_automation.sh post --tag v<version>`
- [ ] Verify installation: `pipx install research-skills-installer && rsk --help`

---

## 6) Frequently Asked Questions (FAQ)

### Q: I pushed a tag but Actions did not trigger?

Ensure the tag format starts with `v` (e.g., `v0.1.0-beta.7`) and that the `.github/workflows/publish-pypi.yml` workflow file is present on the `main` branch.

### Q: PyPI publishing failed with "403 Forbidden"?

This is typically an issue with the Trusted Publisher setup:

- Make sure the workflow name on PyPI exactly matches `publish-pypi.yml`.
- Make sure the GitHub environment name exactly matches `pypi`.
- Verify the owner and repository names are correct.

### Q: Upload failed because the version number already exists?

PyPI does not allow overwriting published versions. If you need to ship a fix, you must increment the version number (e.g., `0.1.0b7` → `0.1.0b8`).

### Q: Is TestPyPI auto-triggered?

No. `publish-testpypi.yml` is `workflow_dispatch` only (manual trigger), so it does not create extra tags.
Production PyPI remains tag-triggered via `publish-pypi.yml` (`v*`).

### Q: How do I recall/withdraw a published version?

You can "yank" a version from the PyPI project page (it will not be deleted from users who already installed it, but new `pip install` commands will skip a yanked version by default).
