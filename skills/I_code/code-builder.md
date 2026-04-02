---
id: code-builder
stage: I_code
description: "Systematic conversion of academic methodologies into executable code (Python/R/Stata/MATLAB/Julia) with domain-profile-driven library selection."
inputs:
  - type: AnalysisPlan
    description: "Analysis plan or method description"
  - type: DomainProfile
    description: "Domain-specific profile for library selection"
    required: false
outputs:
  - type: AnalysisCode
    artifact: "analysis/"
constraints:
  - "Must include environment setup, synthetic data generation, validation tests, and diagnostic plots"
  - "Must follow domain checklist when domain profile is specified"
failure_modes:
  - "Method too novel for standard library matching"
  - "Language-specific implementation gaps"
tools: [filesystem, code-runtime]
tags: [code, implementation, multi-language, domain-profiles, reproducibility]
domain_aware: true
---

# Code Builder Skill

Academic code generation with multi-discipline, multi-language, and 2-tier complexity support.

## Purpose

Systematic conversion of academic methodologies into executable code (Python/R/Stata/MATLAB/Julia).

## Domain Profiles

Discipline-specific libraries, method templates, diagnostics, and pitfalls are defined in **`skills/domain-profiles/*.yaml`**. When `--domain` is specified, the corresponding profile is injected to guide library selection, method checklists, and common pitfalls.

### Available Domains

| Domain | Profile | Key Methods |
|--------|---------|-------------|
| **Finance** | `domain-profiles/finance.yaml` | GARCH, Event Study, MVO, Factor Models, Options |
| **Economics** | `domain-profiles/economics.yaml` | DID, RD, IV/2SLS, Synthetic Control, BLP |
| **Psychology** | `domain-profiles/psychology.yaml` | SEM, CFA, Mediation, HLM, EFA, IRT |
| **Biomedical** | `domain-profiles/biomedical.yaml` | Cox PH, KM, Meta-Analysis, RCT, Competing Risks |
| **Education** | `domain-profiles/education.yaml` | HLM, IRT, Pre-Post, EDM, Mixed Methods |
| **CS/AI** | `domain-profiles/cs-ai.yaml` | Classification, Deep Learning, LLM Fine-Tuning, RL, GNN |
| **Political Science** | `domain-profiles/political-science.yaml` | Survey, Text-as-Data, Conjoint, ERGM, Causal Impact |
| **Epidemiology** | `domain-profiles/epidemiology.yaml` | DAG Causal, Cohort/Case-Control, SIR/SEIR, IPTW |
| **Ecology** | `domain-profiles/ecology-environmental.yaml` | SDM, Community Analysis, GAM, Occupancy, Capture-Recapture |

To add a new domain: copy `domain-profiles/custom-template.yaml` and fill in the sections.

## Multi-Language Support

| Language | Strengths | Typical Use |
|----------|-----------|-------------|
| **Python** | ML/DL, general-purpose, data pipelines | CS/AI, data science, cross-domain |
| **R** | Statistics, visualization, ecology packages | Social sciences, biomedical, ecology |
| **Stata** | Panel data, causal inference, survey | Economics, political science, epi |
| **MATLAB** | Numerical computing, signal processing, EE | Engineering, physics, neuroscience |
| **Julia** | High-performance scientific computing | Computational science, optimization |

When the user does not specify a language, infer the best fit from the domain profile. If the domain profile lists Stata or MATLAB libraries, consider those as valid targets.

## Process

### Tier 1: Standard (Template Matching)

For well-defined, standard methodologies (e.g., OLS, ARIMA, Transformer, SEM, Cox PH).

**Process:**
1. Identify standard method name
2. Lookup domain profile → select recommended library
3. Apply method template checklist from domain profile
4. Generate code with user data schema

### Tier 2: Advanced (Decomposition & Reconstruction)

For novel methods, structural models, or complex custom estimators.

**Process (MDR):**
1. **Extraction**: Extract rigorous mathematical definition (equations, algorithms)
2. **Translation**: Map math operations to tensor operations (JAX/PyTorch) or computational primitives
3. **Reconstruction**: Build custom optimization/training loop

## Invocation

```bash
python -m bridges.orchestrator code-build \
  --domain [finance|econ|psychology|biomedical|education|cs|polisci|epi|ecology] \
  --tier [standard|advanced] \
  --method "Method Name" \
  --lang [python|r|stata|matlab|julia] \
  --paper-context "..."
```

## Prompting Strategy

### For Standard Mode
"Generate {language} code using `{library}` to implement `{method}`. Assume input data frame `df` with columns `...`. Follow the domain checklist: {checklist_items}."

### For Advanced Mode
"You are implementing a novel method defined by these equations: `{equations}`.
1. Define the model parameters as JAX/PyTorch tensors.
2. Implement the objective function (Likelihood/Loss) exactly as defined.
3. Use an optimizer to minimize this objective.
Do NOT use high-level wrappers; implement from first principles."

## Output Structure

所有生成代码应包含：
1. **Environment Setup**: `requirements.txt` / `install.packages()` / `Project.toml`
2. **Data Generation**: Synthetic data generation for verification
3. **Implementation**: Core class/function
4. **Validation**: Test case proving correctness
5. **Domain Checklist**: Pass/fail status for each domain-profile checklist item
6. **Visualization**: Key diagnostic/result plots per domain profile `visualization_templates`

## Quality Bar

- [ ] 代码可在新环境中 one-command 运行（附 requirements/renv.lock）
- [ ] 所有硬编码路径已替换为相对路径或配置参数
- [ ] 随机种子已固定且有文档说明
- [ ] 核心函数有 docstring 和至少一个 assertion/test
- [ ] 输出与 analysis plan 中预期的 table/figure 一一对应

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 硬编码路径 | 他人无法运行 | 用 config file 或 argparse |
| 缺少 seed | 结果不可复现 | 全局 set.seed() + 记录 |
| 库版本不锁 | 依赖更新后行为变化 | 使用 lock file (pip freeze / renv::snapshot) |
| 魔法数字 | 代码中常数无解释 | 提取为 named constant + 注释来源 |
| 分析与可视化耦合 | 改图就得重跑分析 | 分离 analysis → save results → plot |

## When to Use

- 需要将论文方法/分析计划转化为可执行代码时
- 需要结合 domain profile 选择合适的统计/数据处理库时
- 需要生成可复现的分析脚本（R/Python/Stata/Julia）时
- 需要从 analysis plan 派生数据处理 pipeline 时
