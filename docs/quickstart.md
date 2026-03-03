# Quick Start Guide

## 1. Explore the Framework

```
research-skills/
├── skills/                # All 50 skill files organized by stage (A-I, Z)
│   ├── A_framing/         # Research question, theory, positioning
│   ├── B_literature/      # Search, screen, extract, cite
│   ├── C_design/          # Study design, analysis, robustness
│   ├── D_ethics/          # IRB, privacy, deidentification
│   ├── E_synthesis/       # Quality assessment, synthesis, bias
│   ├── F_writing/         # Manuscript, tables, figures, meta
│   ├── G_compliance/      # PRISMA, reporting, tone
│   ├── H_submission/      # Package, rebuttal, review, CRediT
│   ├── I_code/            # Spec, plan, build, review, release
│   ├── Z_cross_cutting/   # Multi-agent, metadata, QA, tone
│   ├── domain-profiles/   # Domain-specific configs (10 domains)
│   └── registry.yaml      # Machine-readable index of all skills
├── pipelines/             # Abstract pipeline DAGs
├── roles/                 # Research team role configs
├── schemas/               # JSON schemas + artifact type vocab
├── eval/                  # Golden test cases + rubrics + runner
└── docs/                  # This documentation
```

## 2. Run a Pipeline

Choose a pipeline that matches your paper type:

| Pipeline | Paper Type |
|---|---|
| `systematic-review-prisma` | Systematic review |
| `empirical-study` | Standard empirical paper |
| `theory-paper` | Theory / conceptual paper |
| `rct-prereg` | RCT with preregistration |
| `code-first-methods` | Code-centric methods paper |

## 3. Select a Domain Profile

Specify your domain via `--domain economics` (or similar) to activate domain-specific:
- Library recommendations
- Statistical diagnostics
- Reporting guidelines
- Venue norms

Available: `economics`, `psychology`, `biomedical`, `cs-ai`, `education`, `epidemiology`, `finance`, `political-science`, `ecology-environmental`

## 4. Optional: Select a Role

Use `--role pi` to configure quality thresholds and preferred skills:
- `pi` — Principal Investigator
- `methods-lead` — Methods specialist
- `literature-ra` — Literature research assistant
- `statistician` — Statistical analysis focus
- `science-writer` — Writing and presentation
- `compliance-officer` — Reporting and ethics

## 5. Validate Your Setup

```bash
python3 scripts/validate_research_standard.py
```
