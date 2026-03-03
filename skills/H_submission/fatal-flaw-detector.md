# Fatal Flaw Detector Skill

Constructive desk-reject analysis: identify flaws likely to cause immediate rejection and propose mitigations.

## Related Task IDs

- `H4` (fatal flaw analysis)

## Output (contract path)

- `RESEARCH/[topic]/revision/fatal_flaw_analysis.md`

## Inputs

- `paper_type`, target `venue`
- Latest `manuscript/manuscript.md`
- Any available methods/results artifacts

## Failure modes to test (examples)

- Novelty not defensible (positioning/gap unsupported)
- RQ-method mismatch or unclear contribution
- Identification/validity threats unaddressed
- Underpowered / non-credible evidence for the claim strength
- Reporting checklist gaps that signal low rigor
- Reproducibility absent (code/data availability inconsistent with claims)

## Minimal output format (`revision/fatal_flaw_analysis.md`)

```markdown
# Fatal Flaw Analysis

| flaw | why it is fatal | evidence/location | fix | feasibility |
|---|---|---|---|---|
```
