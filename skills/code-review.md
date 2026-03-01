# Code Review Skill

Independent review of research code for correctness, reproducibility, and statistical validity.

## Related Task IDs

- `I8` (code review)

## Output (contract path)

- `RESEARCH/[topic]/code/code_review.md`

## Review checklist

### Correctness
- Does the implementation match the method/spec (I5)?
- Are edge cases handled explicitly?
- Are there silent failure modes (NaNs, empty slices, wrong joins)?

### Statistical validity
- Are estimands and standard errors computed correctly?
- Are assumptions checked (or at least flagged)?
- Are multiple comparisons / leakage risks addressed (if relevant)?

### Reproducibility
- Fixed seeds where appropriate
- Deterministic pipelines documented (versions, configs)
- Clear rerun instructions in `code/documentation/`

### Security / data safety (as applicable)
- No secrets in code
- Safe file I/O paths
- Privacy constraints respected (D3)

## Minimal review format (`code/code_review.md`)

```markdown
# Code Review

## Summary

## High-severity findings
1. ...

## Medium / low findings
- ...

## Reproducibility notes
- ...

## Suggested fixes (ordered)
1. ...
```
