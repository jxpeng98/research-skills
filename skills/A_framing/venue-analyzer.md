---
id: venue-analyzer
stage: A_framing
version: "0.2.1"
description: "Analyze venue fit, formatting constraints, and reviewer expectations to scope and position a paper for a target publication."
inputs:
  - type: UserQuery
    description: "Candidate venue(s) and paper type"
  - type: RQSet
    description: "Research questions or contribution type"
outputs:
  - type: VenueAnalysis
    artifact: "framing/venue_analysis.md"
constraints:
  - "Must extract length, format, citation style, anonymization requirements"
  - "Must identify must-not-fail items for the chosen venue"
failure_modes:
  - "Venue information not publicly available"
  - "Paper type mismatch with venue scope"
tools: [filesystem, scholarly-search]
tags: [framing, venue, journal-selection, formatting]
domain_aware: true
---

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
