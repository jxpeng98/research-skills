---
id: venue-analyzer
stage: A_framing
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
  - "Must assess reviewer reward function (novelty vs rigor vs artifact)"
failure_modes:
  - "Venue information not publicly available"
  - "Paper type mismatch with venue scope"
  - "Ignoring recent scope changes or special issue criteria"
tools: [filesystem, scholarly-search]
tags: [framing, venue, journal-selection, formatting]
domain_aware: true
---

# Venue Analyzer Skill

Analyze venue fit and constraints so the paper is scoped and written to match reviewer expectations before a word is written.

## Related Task IDs

- `A5` (venue analysis)

## Output (contract path)

- `RESEARCH/[topic]/framing/venue_analysis.md`

## When to Use

- At the very start of a project (affects RQ scope, methods choice, page limits)
- When deciding between 2–4 candidate venues
- After a rejection — to reposition for a different venue

## Process

### Step 1: Collect Venue Intelligence

For each candidate venue, extract:

#### Basic Profile

| Dimension | What to Find | Where to Look |
|-----------|-------------|---------------|
| **Scope & Aims** | Topics, methods, disciplinary focus | "About" / "Aims & Scope" page |
| **Paper types accepted** | Full paper, short paper, research letter, review, methods paper | Author guidelines |
| **Impact metrics** | JIF, CiteScore, h-index, acceptance rate | Journal website, Scopus, Google Scholar Metrics |
| **Review model** | Single-blind, double-blind, open review | Author guidelines |
| **Turnaround** | Desk-reject rate, time to first decision | Editorial reports, Publons, peer reports |
| **OA policy** | Gold OA, hybrid, green, APC cost | Sherpa/RoMEO, journal website |
| **Indexing** | WoS, Scopus, SSCI, SCI, ESCI, ABDC, ABS, FT50, CNKI | Multiple databases |

#### Formatting Constraints

| Constraint | Details | Impact on Writing |
|-----------|---------|-------------------|
| **Page/word limit** | e.g., 12,000 words / 40 pages / 8 pages + refs | Determines depth of each section |
| **Abstract format** | Structured (Background/Methods/Results/Conclusions) vs narrative | Affects `/paper-write` F6 output |
| **Reference style** | APA 7th, Chicago, Vancouver, IEEE, numbered | Set before writing → feeds `citation-formatter` |
| **Figure/table policy** | Max count, resolution, color charges, format | Affects figure-specifier decisions |
| **Anonymization** | Double-blind: remove author info, self-citation handling | Must scrub before submission |
| **Data/code policy** | Required, encouraged, or optional | Feeds D3 + I4 decisions |
| **Supplementary materials** | Format, page limit, separate file requirements | Plan appendix content early |
| **Section structure** | Required sections (e.g., IMRAD, or custom) | Must match expected layout |
| **Ethics statement** | Required, recommended, or field-specific | Feeds D1 ethics check |
| **AI disclosure** | Required, recommended, or no policy yet | Draft early; requirements are expanding |

### Step 2: Analyze the Reviewer Reward Function

Different venues reward different strengths. Infer the reward function from recent publications:

| Dimension | Weight | How to Infer |
|-----------|--------|-------------|
| **Novelty** | High / Medium / Low | Do published papers emphasize "first to show" or "better understanding"? |
| **Methodological rigor** | High / Medium / Low | Do papers have extensive robustness sections? Are null results published? |
| **Theoretical contribution** | High / Medium / Low | Are papers expected to extend or build theory, or can they be empirical-only? |
| **Practical relevance** | High / Medium / Low | Are managerial/policy implications required or perfunctory? |
| **Artifact quality** | High / Medium / Low | Does the venue require code, data, replication packages? |
| **Writing quality** | High / Medium / Low | Is "well-written" a frequent review criterion? |

**Method**: Read 3–5 recent papers in the venue similar to your topic. Note:
- How long is each section (proportional emphasis)?
- What do Discussion sections emphasize?
- Are limitations perfunctory or substantial?
- How theoretical vs empirical are the contributions?

### Step 3: Identify Must-Not-Fail Items

Every venue has implicit "desk-reject triggers." Common ones:

| Must-Not-Fail | How to Check | Consequence |
|---------------|-------------|-------------|
| **Scope match** | Your RQ fits the published scope statement | Desk reject if out of scope |
| **Contribution clarity** | First 2 pages clearly state what's new | Reviewer confusion → reject |
| **Method appropriateness** | Method fits venue norms (e.g., MIS Quarterly expects rigorous quantitative or well-justified qualitative) | "Wrong venue" verdict |
| **Length compliance** | Within word/page limits | Administrative desk reject |
| **Anonymization** | No author-identifying information in blind review | Administrative desk reject |
| **Reference recency** | Cites recent work (last 2–3 years) in the venue | "Not current" verdict |
| **Self-citation ratio** | Not excessive (<15% of references typically) | May break anonymization |
| **Reporting guideline** | Venue-required checklist submitted (CONSORT, PRISMA, etc.) | Administrative desk reject |

### Step 4: Rank and Recommend

For 2–4 candidate venues, fill in the comparison:

| Dimension | Venue A | Venue B | Venue C |
|-----------|---------|---------|---------|
| Scope fit | ✅ Core | ⚠️ Peripheral | ✅ Core |
| Method fit | ✅ Strong | ✅ Strong | ⚠️ Theory-heavy |
| Reviewer reward | Rigor | Novelty | Theory |
| Acceptance rate | ~15% | ~8% | ~20% |
| Turnaround (months) | 3–4 | 6–8 | 2–3 |
| Word limit | 12,000 | 8,000 | 15,000 |
| Impact | JIF 5.2 | FT50 | ABDC A |
| Recommendation | **Primary** | Backup | Backup |

Justify the recommendation by connecting scope + method + reward function + your paper's strengths.

## Quality Bar

The venue analysis is **ready** when:

- [ ] At least 2 venues compared with full profiles
- [ ] Formatting constraints extracted and documented
- [ ] Reviewer reward function inferred from reading recent publications
- [ ] Must-not-fail items identified per venue
- [ ] Clear recommendation with justification
- [ ] Formatting decisions (reference style, abstract format, section structure) propagated to downstream skills

## Minimal Output Format

```markdown
# Venue Analysis

## Candidate Venues

### Venue A: [Name]
- **Scope fit**: ...
- **Paper type**: ...
- **Review model**: ...
- **Impact**: JIF = ..., acceptance rate = ...

#### Formatting Constraints
| Dimension | Requirement |
|-----------|-------------|

#### Reviewer Reward Function
| Dimension | Weight |
|-----------|--------|

#### Must-Not-Fail Items
1. ...
2. ...

### Venue B: [Name]
...

## Comparison Matrix

| Dimension | Venue A | Venue B | Venue C |
|-----------|---------|---------|---------|

## Recommendation
**Primary venue**: [name] — [justification]
**Backup venue**: [name] — [rationale for backup]

## Downstream Propagation
- Citation style: [APA 7th / ...]
- Abstract format: [structured / narrative]
- Section structure: [IMRAD / custom]
- Word limit: [n]
```
