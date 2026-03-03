# Research Skills Framework — Conventions

## Skill Naming

- `kebab-case` for all skill file names (e.g., `gap-analyzer.md`)
- Skill ID matches filename without `.md` extension
- Stage prefix in directory path (`A_framing/`, `B_literature/`, etc.)

## YAML Frontmatter Contract

Every skill `.md` file MUST begin with YAML frontmatter containing:

```yaml
---
id: skill-name
stage: A_framing          # Stage directory name
version: "1.0.0"          # Semver
description: "One-line description"
inputs:
  - type: ArtifactType    # From schemas/artifact-types.yaml
    description: "..."
    required: false        # Optional, defaults to true
outputs:
  - type: ArtifactType
    artifact: "path/file.md"
constraints:
  - "Must ..."
failure_modes:
  - "When ..."
tools: [filesystem, scholarly-search]
tags: [stage, topic, method]
domain_aware: false
---
```

## Artifact Types

All typed artifact names are defined in `schemas/artifact-types.yaml`. Use these types consistently in frontmatter `inputs` and `outputs`, registry entries, and pipeline step definitions.

## Pipeline DAGs vs Workflow Slash-Commands

| Layer | Directory | Purpose |
|---|---|---|
| **Pipelines** | `pipelines/` | Abstract DAGs defining step sequence + dependencies |
| **Workflows** | `.agent/workflows/` | Claude Code slash-command execution layer |

Pipelines reference skill IDs; workflows call skills directly.

## Roles

Role configs in `roles/` define preferred skills, quality thresholds, and tone for different research team members. Pass `--role pi` to adjust behavior.

## Domain Profiles

Domain profiles in `skills/domain-profiles/` customize skill behavior for specific disciplines. Each profile includes:
- Library recommendations per language
- Method templates with checklists
- Statistical diagnostics
- Reporting guidelines
- Default databases and query syntax
- Methodology priors
- Venue norms

## Adding a New Skill

1. Create `skills/{STAGE}/{skill-name}.md` with full YAML frontmatter
2. Add entry to `skills/registry.yaml`
3. Add new artifact types to `schemas/artifact-types.yaml` if needed
4. Add to relevant pipelines in `pipelines/`
5. Run `python3 scripts/validate_research_standard.py` to verify

## Adding a New Domain Profile

1. Copy `skills/domain-profiles/custom-template.yaml`
2. Fill in all sections following the schema in `schemas/domain-profile.schema.json`
3. Run validator to confirm
