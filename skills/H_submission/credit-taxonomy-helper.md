---
id: credit-taxonomy-helper
stage: H_submission
version: "0.1.0"
description: "Generate CRediT (Contributor Roles Taxonomy) author contribution statements from project roles."
inputs:
  - type: Manuscript
    description: "Finalized manuscript with author list"
  - type: UserQuery
    description: "Author roles and contributions"
outputs:
  - type: CRediTStatement
    artifact: "submission/credit_statement.md"
constraints:
  - "Must use all 14 CRediT roles"
  - "Must ensure every author has at least one role"
  - "Must flag potential authorship order disagreements"
failure_modes:
  - "Author contributions unknown or disputed"
  - "Solo author edge case"
tools: [filesystem]
tags: [submission, CRediT, authorship, contributions, taxonomy]
domain_aware: false
---

# CRediT Taxonomy Helper Skill

Generate CRediT author contribution statements following ICMJE and journal requirements.

## Related Task IDs

- `H1` (submission packaging — author contributions)

## Output (contract path)

- `RESEARCH/[topic]/submission/credit_statement.md`

## The 14 CRediT Roles

1. Conceptualization
2. Methodology
3. Software
4. Validation
5. Formal analysis
6. Investigation
7. Resources
8. Data Curation
9. Writing – original draft
10. Writing – review & editing
11. Visualization
12. Supervision
13. Project administration
14. Funding acquisition

## Procedure

1. **Collect author information**: names, affiliations, ORCID
2. **Map each author** to applicable CRediT roles (lead / supporting / equal)
3. **Verify completeness**:
   - Every author has ≥ 1 role
   - Every applicable role has ≥ 1 author
4. **Generate statement** in venue-required format:
   - Tabular format (for journals that require it)
   - Narrative format (for journals preferring prose)
5. **Cross-check** against ICMJE authorship criteria (substantial contribution, drafting/revising, final approval, accountability)

## Minimal Output Format

```markdown
# CRediT Author Contributions

| Author | Conceptualization | Methodology | Software | ... | Funding |
|---|---|---|---|---|---|
| A. Smith | Lead | Supporting | — | ... | Lead |

## Narrative Format
A. Smith: Conceptualization (lead), Methodology (supporting), Funding acquisition (lead).
```
