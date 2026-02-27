# Code Builder Skill

Academic code generation with multi-discipline and 2-tier complexity support.

## Purpose

Systematic conversion of academic methodologies into executable code (Python/R/Stata).

## Domain Support

| Domain | Key Libraries (Python) | Key Libraries (R) |
|--------|------------------------|-------------------|
| **Finance** | `arch`, `QuantLib`, `PyPortfolioOpt` | `rugarch`, `PerformanceAnalytics` |
| **Economics** | `pyblp`, `linearmodels` | `fixest`, `plm`, `mlogit` |
| **Econometrics** | `statsmodels`, `rdrobust` | `rdrobust`, `brms`, `lavaan` |
| **CS/AI** | `torch`, `jax`, `polars` | `torch`, `data.table` |

## Construction Strategy

### Tier 1: Standard (Template Matching)

For well-defined, standard methodologies (e.g., OLS, ARIMA, Transformer).

**Process:**
1. Identify standard method name
2. Select standard library
3. Apply template with user data schema

### Tier 2: Advanced (Decomposition & Reconstruction)

For novel methods, structural models, or complex custom estimators.

**Process (MDR):**
1. **Extraction**: Extract rigorous mathematical definition (equations, algorithms)
2. **Translation**: Map math operations to tensor operations (JAX/PyTorch)
3. **Reconstruction**: Build custom optimization/training loop

## Invocation

```bash
python -m bridges.orchestrator code-build \
  --domain [finance|econ|metrics|cs] \
  --tier [standard|advanced] \
  --method "Method Name" \
  --paper-context "..."
```

## Prompting Strategy

### For Standard Mode
"Generate Python code using `{library}` to implement `{method}`. Assume input data frame `df` with columns `...`."

### For Advanced Mode
"You are implementing a novel method defined by these equations: `{equations}`.
1. Define the model parameters as JAX/PyTorch tensors.
2. Implement the objective function (Likelihood/Loss) exactly as defined.
3. Use an optimizer to minimize this objective.
Do NOT use high-level wrappers; implement from first principles."

## Output Structure

所有生成代码应包含：
1. **Environment Setup**: `requirements.txt` or `install.packages()`
2. **Data Generation**: Synthetic data generation for verification
3. **Implementation**: Core class/function
4. **Validation**: Test case proving correctness
