# Peer Review Simulation Skill

Simulate independent peer reviews using distinct reviewer personas and consolidate findings into an action plan.

## Related Task IDs

- `H3` (peer review simulation)

## Output (contract path)

- `RESEARCH/[topic]/revision/peer_review_simulation.md`

## Inputs (minimum)

- Target `venue` (or closest venue archetype)
- `paper_type`
- `manuscript/manuscript.md` (or the latest draft)
- Key artifacts (as available): `study_design.md`, `analysis_plan.md`, `synthesis.md`, `reporting_checklist.md`

## Procedure

1. Run at least 3 personas *independently*:
   - Methodologist (design/identification/statistics)
   - Domain expert (novelty/positioning/coverage)
   - “Reviewer 2” (clarity, skepticism, missing citations)
2. For each persona, produce:
   - summary + overall recommendation
   - 3–8 major issues (actionable)
   - minor issues (style/clarity)
3. Consolidate:
   - deduplicate issues
   - assign severity: `fatal / major / minor`
   - map each issue to an edit location + owner + next step

## Minimal output format (`revision/peer_review_simulation.md`)

```markdown
# Peer Review Simulation

## Persona reviews
### Methodologist
- Summary:
- Major:
- Minor:

### Domain expert
...

### Reviewer 2
...

## Consolidated issues
| severity | issue | evidence/location | fix | owner |
|---|---|---|---|---|
```
