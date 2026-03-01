# Tone Normalizer Skill

Rewrite text into concise, objective academic tone and reduce reviewer-triggering overclaim.

## Related Task IDs

- `G4` (tone normalization)

## Output (contract path)

- `RESEARCH/[topic]/compliance/tone_normalization.md`

## Procedure

1. Identify high-risk patterns:
   - absolute claims (“prove”, “guarantee”, “always”)
   - vague intensifiers (“very”, “extremely”, “significant” without stats)
   - citation-free prior-work claims
2. Rewrite with calibrated language:
   - “suggest”, “is consistent with”, “we observe”
3. Enforce style rules:
   - keep paragraphs single-purpose
   - define acronyms once
   - remove filler transitions where possible

## Minimal output format (`compliance/tone_normalization.md`)

```markdown
# Tone Normalization Log

| location | original | issue | rewrite |
|---|---|---|---|
```
