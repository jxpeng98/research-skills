---
id: data-management-plan
stage: C_design
version: "0.2.0"
description: "Generate FAIR-compliant data management plans specifying storage, backup, retention, sharing, and archival strategy."
inputs:
  - type: DesignSpec
    description: "Study design with data types and volumes"
  - type: EthicsPackage
    description: "Ethics requirements constraining data handling"
    required: false
outputs:
  - type: DataManagementPlan
    artifact: "data_management_plan.md"
constraints:
  - "Must comply with FAIR principles (Findable, Accessible, Interoperable, Reusable)"
  - "Must specify retention periods and destruction procedures"
  - "Must identify repository for data deposit"
failure_modes:
  - "Institutional DMP requirements unknown"
  - "Data too sensitive for open deposit"
tools: [filesystem]
tags: [design, data-management, FAIR, storage, archival]
domain_aware: false
---

# Data Management Plan Skill

Generate a complete Data Management Plan (DMP) following FAIR principles and funder requirements.

## Related Task IDs

- `C3` (data management planning)

## Output (contract path)

- `RESEARCH/[topic]/data_management_plan.md`

## Inputs

- **Study design** (`study_design.md`) — data types, expected volume, sensitivity level
- **Ethics requirements** (optional) — constraints from IRB/GDPR

## Procedure

1. **Data Description**
   - Enumerate all data types (survey, experimental, administrative, secondary)
   - Estimate volume and format per type
   - Classify sensitivity level (public / restricted / confidential)

2. **Collection & Organization**
   - Specify file naming conventions
   - Define folder structure
   - Document version control procedures

3. **Storage & Security**
   - Primary and backup storage locations
   - Encryption and access control measures
   - Compliance with institutional IT policies

4. **Preservation & Sharing**
   - Repository selection (Zenodo, Dryad, ICPSR, domain-specific)
   - License selection (CC-BY, CC0, restricted)
   - Embargo period justification if applicable
   - Persistent identifier (DOI) assignment

5. **Retention & Destruction**
   - Minimum retention period (typically 5-10 years post-publication)
   - Destruction protocol for sensitive data
   - Documentation of what is retained vs. destroyed

6. **Responsibilities & Resources**
   - Data steward assignment
   - Estimated storage costs
   - Training requirements

## Minimal Output Format

```markdown
# Data Management Plan

## 1. Data Description
| Data Type | Format | Volume | Sensitivity |
|---|---|---|---|

## 2. Collection & Organization
...

## 3. Storage & Security
...

## 4. Preservation & Sharing
| Repository | License | Embargo | DOI |
|---|---|---|---|

## 5. Retention & Destruction
...

## 6. Responsibilities
...
```
