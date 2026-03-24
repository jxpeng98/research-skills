---
id: release-packager
stage: I_code
version: "0.1.0"
description: "Package code, data, environment, and documentation for reproducible release (Zenodo, GitHub, Dataverse)."
inputs:
  - type: AnalysisCode
    description: "Analysis code to package"
  - type: ReproducibilityReport
    description: "Reproducibility audit results"
    required: false
outputs:
  - type: ReleasePackage
    artifact: "release/"
constraints:
  - "Must include README with setup and run instructions"
  - "Must include environment specification (requirements.txt / renv.lock / environment.yml)"
  - "Must verify all paths are relative, not absolute"
failure_modes:
  - "Large data files exceed repository limits"
  - "Sensitive data requires synthetic substitution"
tools: [filesystem, code-runtime]
tags: [code, release, packaging, Zenodo, reproducibility, README]
domain_aware: false
---

# Release Packager Skill

Package analysis code, data, and environment for reproducible public release.

## Related Task IDs

- `I5` (release packaging)

## Output (contract path)

- `RESEARCH/[topic]/release/`

## Package Contents

```
release/
├── README.md           # Setup, run, and reproduce instructions
├── LICENSE             # Code license (MIT, Apache, GPL)
├── requirements.txt    # Python dependencies (or renv.lock / environment.yml)
├── Dockerfile          # Optional containerization
├── src/                # Source code
├── data/               # Data or synthetic data + fetch script
├── results/            # Expected output for verification
└── .zenodo.json        # Zenodo metadata (if applicable)
```

## Procedure

1. **Audit file structure**:
   - Verify all paths are relative
   - Remove absolute paths, hardcoded credentials, API keys
   - Ensure `.gitignore` excludes sensitive files
2. **Create environment specification**:
   - `requirements.txt` with pinned versions (Python)
   - `renv.lock` (R)
   - `environment.yml` (conda)
   - Optional `Dockerfile` for full isolation
3. **Generate README.md**:
   - Project description
   - Prerequisites
   - Installation instructions
   - How to reproduce results
   - Expected output / validation
   - Citation information
4. **Handle data**:
   - If data is open: include directly
   - If restricted: provide synthetic data + fetch script
   - Document data sources and access procedures
5. **Add metadata**:
   - `.zenodo.json` for automatic DOI assignment
   - `CITATION.cff` for GitHub citation
   - License file
6. **Verify package**:
   - Fresh install test (clean venv)
   - End-to-end run produces expected results

## Minimal Output Format

```markdown
# Release Package Checklist

- [ ] README.md with full instructions
- [ ] Environment specification (pinned versions)
- [ ] All paths relative
- [ ] No secrets/credentials in code
- [ ] Data included or fetch script provided
- [ ] License file present
- [ ] Citation metadata (.zenodo.json or CITATION.cff)
- [ ] Fresh-install test passed
```
