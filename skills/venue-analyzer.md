# Venue Analyzer Skill

Analyze venue fit and constraints so the paper is scoped and written to match reviewer expectations.

## Related Task IDs

- `A5` (venue analysis)

## Output (contract path)

- `RESEARCH/[topic]/framing/venue_analysis.md`

## Inputs (ask for)

- Candidate venue(s)
- `paper_type` and contribution type
- Whether double-blind / artifact evaluation is required

## Procedure

1. Extract constraints: length, format, required sections, citation style, anonymization.
2. Infer reviewer reward function: novelty vs rigor vs artifact quality.
3. Identify must-not-fail items for the chosen venue.
4. Produce a ranked venue shortlist with justification.
