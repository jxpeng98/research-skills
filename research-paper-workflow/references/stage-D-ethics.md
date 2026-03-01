# Stage D — Ethics / IRB / Compliance (D1–D3)

This stage produces ethics-ready materials and forces early clarity on privacy, consent, and data governance.

## Canonical outputs (contract paths)

- `D1` → `ethics_irb.md`
- `D2` → `manuscript/manuscript.md` (ethics + availability statements section)
- `D3` → `compliance/deidentification_plan.md`

## Quality gate focus

- `Q4` (reproducibility baseline): governance + availability statements should match the actual planned artifacts.

---

## D1 — Ethics / IRB Pack

**Definition of done**
- Participant population and recruitment are clearly described
- Risks are identified and mitigations are stated
- Consent/withdrawal process is documented (or justified as not applicable)
- Data security + access control + retention are specified

Suggested structure: `ethics_irb.md`

```markdown
# Ethics / IRB Pack

## Study overview
- Purpose:
- Population:
- Procedures:

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|---|---:|---:|---|

## Consent
- Consent process:
- Withdrawal:

## Privacy & security
- Data minimization:
- Storage:
- Access control:
- Retention:

## Sensitive data handling
- Identifiers collected:
- De-identification linkage (D3):

## Recruitment materials (if applicable)
- ...
```

---

## D2 — Manuscript-Ready Ethics & Availability Statements

Write statements that match the venue’s required disclosure format.

**Definition of done**
- Ethics approval statement (or exemption rationale)
- Informed consent statement (or not applicable)
- Data availability statement
- Code availability statement
- Competing interests / funding (if applicable)

Write into: `manuscript/manuscript.md` (usually Methods end or a dedicated section).

---

## D3 — De-identification Plan

Treat as a technical privacy plan, not a vague promise.

**Definition of done**
- Data classification (direct identifiers / quasi-identifiers / sensitive attributes)
- Threat model (who could re-identify and how)
- Concrete transformation plan (suppression/generalization/aggregation/perturbation)
- Re-identification risk evaluation approach

Write into: `compliance/deidentification_plan.md`.

Suggested table:

```markdown
| Field | Type (direct/quasi/sensitive) | Action | Rationale |
|---|---|---|---|
```

