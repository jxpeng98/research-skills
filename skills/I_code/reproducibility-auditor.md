# Reproducibility Auditor Skill

Audit a project for computational reproducibility and document a rerun recipe.

## Related Task IDs

- `I4` (reproducibility audit)

## Output (contract path)

- `RESEARCH/[topic]/code/reproducibility_audit.md`

## Inputs

- Current project under `RESEARCH/[topic]/`
- Entry commands used to generate results/figures

## Audit checklist

- **Environment**: language versions + key dependencies pinned
- **Randomness**: seeds controlled; nondeterminism documented
- **Data provenance**: where data came from; hashes/versions when possible
- **Execution**: one-command rerun path (or a short ordered list)
- **Outputs**: paths are stable and documented

## Minimal audit format (`code/reproducibility_audit.md`)

```markdown
# Reproducibility Audit

## Environment

## Data provenance

## Run instructions
1. ...

## Determinism notes

## Known failure points
```
