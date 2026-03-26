---
id: release-packager
stage: I_code
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
  - "Must pass fresh-install reproducibility test"
failure_modes:
  - "Large data files exceed repository limits"
  - "Sensitive data requires synthetic substitution"
tools: [filesystem, code-runtime]
tags: [code, release, packaging, Zenodo, reproducibility, README]
domain_aware: false
---

# Release Packager Skill

Package analysis code, data, and environment for reproducible public release — ensuring that anyone can reproduce your results from the shared materials.

## Related Task IDs

- `I5` (release packaging)

## Output (contract path)

- `RESEARCH/[topic]/release/`

## When to Use

- When preparing for manuscript submission (data/code availability statement)
- When a journal requires reproducibility materials
- When archiving a completed project for long-term preservation
- When sharing materials with collaborators

## Process

### Step 1: Define Package Structure

```
release/
├── README.md                    # Entry point: setup + reproduce instructions
├── LICENSE                      # Code license
├── CITATION.cff                 # Machine-readable citation metadata
├── requirements.txt             # Python dependencies (pinned)
│   or renv.lock                 # R dependencies
│   or environment.yml           # Conda environment
├── Makefile / run.sh            # Single-command reproduction
├── src/                         # Source code
│   ├── 01_clean.py              # Data preparation
│   ├── 02_analyze.py            # Main analysis
│   └── 03_visualize.py          # Figure generation
├── data/
│   ├── raw/                     # Raw data (or fetch script)
│   ├── processed/               # Cleaned data
│   └── README_data.md           # Data documentation
├── results/
│   ├── tables/                  # Output tables
│   ├── figures/                 # Output figures
│   └── expected_output.md       # Hash or summary for verification
├── docs/
│   ├── data_dictionary.md       # Variable codebook
│   └── analysis_plan.md         # Pre-specified analysis plan
└── .zenodo.json                 # Zenodo metadata (optional)
```

### Step 2: Audit and Clean Code

| Check | What to Verify | Action |
|-------|---------------|--------|
| **Absolute paths** | No hardcoded paths like `/Users/name/...` | Convert to relative: `Path(__file__).parent / "data"` |
| **Credentials** | No API keys, passwords, tokens | Move to `.env` (excluded from package) |
| **Hardcoded parameters** | Magic numbers embedded in code | Extract to config file or constants section |
| **Comments** | Code is understandable | Add docstrings, section headers, inline comments |
| **Execution order** | Scripts run in correct sequence | Number scripts: `01_`, `02_`, `03_` |
| **Random seeds** | Reproducibility of stochastic results | Set `seed = 42` (or any fixed value) at top |
| **Output paths** | Results write to `results/`, not scattered locations | Centralize output directories |

### Step 3: Create Environment Specification

Pin ALL dependency versions:

#### Python
```
# requirements.txt — generated with: pip freeze > requirements.txt
pandas==2.1.4
numpy==1.26.2
scipy==1.11.4
statsmodels==0.14.1
matplotlib==3.8.2
seaborn==0.13.0
```

#### R
```r
# Generate lockfile:
renv::init()
renv::snapshot()
# Produces renv.lock with all package versions
```

#### Docker (maximum reproducibility)
```dockerfile
FROM python:3.11-slim
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY src/ ./src/
COPY data/ ./data/
CMD ["python", "src/02_analyze.py"]
```

### Step 4: Handle Data

| Data Situation | Strategy | What to Include |
|---------------|----------|-----------------|
| **Open data, small (<100MB)** | Include directly | Full dataset in `data/raw/` |
| **Open data, large (>100MB)** | Provide download script | `data/fetch_data.sh` or `data/fetch_data.py` |
| **Restricted access** | Provide synthetic data + access instructions | Synthetic data (same structure/distributions) + README |
| **Proprietary / licensed** | Document access procedure | Application instructions, DUA template, expected timeline |
| **Sensitive (human subjects)** | De-identified subset or synthetic | Clear statement of what's shared and why |

For restricted data, include:
```markdown
# Data Access Instructions

The data used in this study cannot be publicly shared due to [reason].

## How to Access
1. Apply to [organization] at [URL]
2. Submit [required documents]
3. Expected timeline: [n] weeks
4. Contact: [email for data access questions]

## Synthetic Data
We provide synthetic data (`data/synthetic/`) that matches the structure
and statistical properties of the original data. Results from synthetic
data will NOT match published results but can verify that code runs correctly.
```

### Step 5: Generate README.md

```markdown
# [Project Title]

## Description
[1–2 paragraph description of the study]

## Citation
If you use this code, please cite:
> [Authors]. ([Year]). [Title]. [Journal]. DOI: [doi]

## Requirements
- Python [version] (tested on 3.11.x) OR R [version]
- OS: [tested on macOS 14, Ubuntu 22.04]
- Memory: [minimum RAM, if relevant]

## Quick Start

### Setup
```bash
git clone [url]
cd [project]
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
```

### Reproduce All Results
```bash
make all
# or
bash run.sh
```

### Individual Steps
```bash
python src/01_clean.py      # Clean raw data → data/processed/
python src/02_analyze.py    # Run analysis → results/tables/
python src/03_visualize.py  # Generate figures → results/figures/
```

## Repository Structure
[annotated file tree]

## Data
[Data availability statement; access instructions if restricted]

## License
Code: [MIT / Apache 2.0]
Data: [CC-BY 4.0 / restricted]
```

### Step 6: Add Metadata for Archival

#### CITATION.cff (GitHub)
```yaml
cff-version: 1.2.0
message: "If you use this software, please cite it as below."
type: software
title: "[Project Title]"
authors:
  - family-names: "[Last]"
    given-names: "[First]"
    orcid: "https://orcid.org/0000-0000-0000-0000"
date-released: "[YYYY-MM-DD]"
doi: "[DOI]"
repository-code: "[URL]"
license: MIT
```

#### .zenodo.json (Zenodo DOI)
```json
{
  "title": "[Project Title]",
  "creators": [{"name": "Last, First", "orcid": "0000-0000-0000-0000"}],
  "description": "[Description]",
  "access_right": "open",
  "license": "MIT",
  "related_identifiers": [
    {"identifier": "[paper DOI]", "relation": "isSupplementTo", "scheme": "doi"}
  ]
}
```

### Step 7: Verify Package (Fresh-Install Test)

| Test | How to Execute | Expected Result |
|------|---------------|-----------------|
| **Fresh environment** | Create new venv, install from requirements.txt | No import errors |
| **End-to-end run** | Run all scripts in sequence | Output matches expected results |
| **Output verification** | Compare output hash/values with expected | Within numerical tolerance |
| **README walkthrough** | Follow README exactly as written | Everything works as described |
| **Cross-platform** | Test on different OS if possible | No OS-specific failures |

## Quality Bar

The release package is **ready** when:

- [ ] README provides complete setup and reproduce instructions
- [ ] Environment specification has pinned versions for ALL dependencies
- [ ] All paths are relative (no absolute paths in any file)
- [ ] No credentials, API keys, or sensitive information in code
- [ ] Data is included or fetch/access instructions provided
- [ ] License file present (for code and data separately if needed)
- [ ] CITATION.cff or .zenodo.json metadata present
- [ ] Random seeds fixed for reproducible stochastic results
- [ ] Fresh-install test passed on clean environment
- [ ] Output matches expected results

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Unpinned dependencies | `pip install pandas` installs different version next year | Pin ALL versions in requirements.txt |
| Absolute paths | Fails on any other machine | Use `pathlib.Path(__file__).parent` or relative paths |
| Missing dependency | Package works on author's machine, fails elsewhere | Fresh-install test catches this |
| Data too large for Git | Push fails or repo is slow | Use Git LFS, or external hosting + fetch script |
| README says "run analysis.py" but file is "02_analyze.py" | Reader can't reproduce | Test README instructions literally |
| No random seed | Stochastic results differ each run | Set explicit seed at top of every script |
