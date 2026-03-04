---
description: Build academic research code from paper descriptions or methodology names
---

# Code Build Workflow

Generates research code supporting multiple disciplines and complexity levels.

Canonical Task IDs from `standards/research-workflow-contract.yaml`:
- `I1` method implementation
- `I2` reproduction
- `I3` data pipeline

## Usage

```bash
/code-build [method_name] --domain [domain] --tier [standard|advanced]
```

## Options

- `method_name`: The specific methodology (e.g., "Difference-in-Differences", "DSGE")
- `--domain`: `finance`, `econ`, `metrics`, `cs` (Auto-detected if omitted)
- `--tier`: 
  - `standard`: Use standard libraries (Tier 1)
  - `advanced`: Implement from scratch/equations (Tier 2)
- `--from`: URL or Path to paper PDF (optional)

## Workflow Steps

1. **Analysis**: 
   - Identify domain and methodology complexity
   - Determine Tier (Standard vs Advanced)

2. **Prompt Generation**:
   - if Tier 1: Select relevant library template
   - if Tier 2: Extract equations and formulate implementation plan

3. **Code Generation** (via `model-collaborator`):
   - **Codex**: Implementation logic
   - **Gemini**: Documentation and explanations

4. **Output**:
   - Python/R script file
   - Verification test
   - Explanation of implementation details

## Examples

### Standard Econometrics (Tier 1)
```bash
/code-build "Staggered DID" --domain econ
```
> Generates Python (`pyfixest`) or R (`did`) code using standard packages.

### Advanced Structural Model (Tier 2)
```bash
/code-build "Dynamic Discrete Choice" --domain econ --tier advanced
```
> Generates JAX/scipy code implementing the Bellman equation and likelihood maximization from scratch.

### Financial Derivative (Tier 1)
```bash
/code-build "GARCH(1,1)" --domain finance
```
> Generates code using `arch` package.

### Custom Estimator (Tier 2)
```bash
/code-build "Custom Moment-based Estimator" --tier advanced
```
> Generates GMM implementation using PyTorch/JAX autodiff.
