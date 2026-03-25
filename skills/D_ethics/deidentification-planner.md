---
id: deidentification-planner
stage: D_ethics
description: "Design technical privacy measures via k-anonymity, differential privacy, or secure data pipelines for sensitive data."
inputs:
  - type: DesignSpec
    description: "Study design with data types"
  - type: EthicsPackage
    description: "Ethics review requirements"
outputs:
  - type: DeidentificationPlan
    artifact: "compliance/deidentification_plan.md"
constraints:
  - "Must specify re-identification risk threshold"
  - "Must document data transformation procedures"
  - "Must preserve analytical utility while reducing disclosure risk"
failure_modes:
  - "Small dataset makes k-anonymity impractical"
  - "Utility loss from aggressive anonymization makes analysis meaningless"
  - "Quasi-identifier combinations uniquely identify individuals despite field-level anonymization"
tools: [filesystem]
tags: [ethics, privacy, deidentification, k-anonymity, differential-privacy]
domain_aware: false
---

# Deidentification Planner Skill

Plan concrete technical de-identification steps for sensitive data, balancing privacy protection against analytical utility.

## Related Task IDs

- `D3` (de-identification plan)

## Output (contract path)

- `RESEARCH/[topic]/compliance/deidentification_plan.md`

## When to Use

- When handling any dataset containing identifiable information
- Before data sharing or archiving
- When IRB requires a documented de-identification procedure
- When working with health data (HIPAA), student records (FERPA), or EU personal data (GDPR)

## Process

### Step 1: Classify Every Field

Categorize each variable/field in your dataset:

| Category | Definition | Examples | Default Action |
|----------|-----------|----------|----------------|
| **Direct identifier** | Uniquely identifies a person on its own | Name, SSN, email, phone, student ID, medical record # | **Remove** or replace with study ID |
| **Quasi-identifier** | Could identify someone in combination | Age, ZIP code, gender, job title, admission date, organization | **Transform** (generalize, suppress, perturb) |
| **Sensitive attribute** | What you want to protect from disclosure | Diagnosis, salary, test score, criminal record, interview quotes | **Retain** but protect via the other categories |
| **Non-identifying** | Cannot contribute to re-identification | Aggregate statistics, generic categorical codes | **Retain** unchanged |

> **Critical**: The re-identification risk comes from *combinations* of quasi-identifiers, not individual fields. A 45-year-old male software engineer in Boulder, CO may be unique.

### Step 2: Choose De-identification Standard

| Standard | When to Use | Key Requirement |
|----------|------------|-----------------|
| **HIPAA Safe Harbor** | US health data | Remove all 18 specified identifier types |
| **HIPAA Expert Determination** | US health; Safe Harbor too restrictive | Qualified expert certifies low re-identification risk |
| **GDPR Anonymization** | EU data | Data must be irreversibly non-identifiable |
| **GDPR Pseudonymization** | EU data; need to re-link | Separate key-code mapping; treated as personal data |
| **Statistical disclosure control** | Survey/census microdata | k-anonymity, l-diversity, or t-closeness applied |

#### HIPAA Safe Harbor: The 18 Identifiers

All of the following must be removed or generalized:

1. Names
2. Geographic data smaller than state (ZIP codes: generalize to first 3 digits if population >20,000)
3. Dates (except year) related to an individual
4. Phone numbers
5. Fax numbers
6. Email addresses
7. Social Security numbers
8. Medical record numbers
9. Health plan beneficiary numbers
10. Account numbers
11. Certificate/license numbers
12. Vehicle identifiers
13. Device identifiers
14. Web URLs
15. IP addresses
16. Biometric identifiers
17. Full-face photos
18. Any other unique identifying number

### Step 3: Apply Transformation Techniques

| Technique | How It Works | When to Use | Utility Impact |
|-----------|-------------|------------|----------------|
| **Suppression** | Remove the field entirely | Direct identifiers; rare categories | High — field lost |
| **Generalization** | Replace specific values with broader categories | Age → age range; ZIP → first 3 digits | Medium |
| **Top/bottom coding** | Cap extreme values | Age >89 → "90+"; income >$200K → ">$200K" | Low |
| **Microaggregation** | Replace small groups with group means | Continuous quasi-identifiers | Medium |
| **Data swapping** | Swap values between similar records | Geographic or demographic fields | Low if done well |
| **Noise addition** | Add calibrated random noise | Continuous variables in statistical databases | Low-Medium |
| **Differential privacy** | Add noise with formal privacy guarantee (ε-DP) | Large datasets; query-based access; strong guarantees needed | Depends on ε |
| **Pseudonymization** | Replace identifiers with codes; retain key separately | When re-linkage is needed for longitudinal data | None (on analysis data) |

### Step 4: Verify k-Anonymity (or Equivalent)

**k-anonymity**: Every combination of quasi-identifiers must appear in at least *k* records.

| k value | Protection level | When appropriate |
|---------|-----------------|-----------------|
| k = 3 | Minimal | Low-sensitivity data, large N |
| k = 5 | Standard | Most research use cases |
| k = 10+ | Strong | Health data, legal data, public release |

**Verification procedure**:
1. Select all quasi-identifier columns
2. Group by each unique combination
3. Count records per group
4. If any group < k, apply further generalization or suppression
5. Document the final k achieved and which fields were transformed

**When k-anonymity is insufficient**: Small datasets, high-dimensional data, or motivated attackers may require:
- **l-diversity**: Each equivalence class has ≥ l distinct sensitive values
- **t-closeness**: Distribution of sensitive attribute in each class is within threshold t of the overall distribution

### Step 5: Assess Residual Risk and Utility

| Risk Factor | Assessment | Mitigation |
|-------------|-----------|-------------|
| Motivated attacker with auxiliary data | High / Medium / Low | More aggressive generalization |
| Data published vs controlled access | Published = higher risk | Use controlled access for higher granularity |
| Longitudinal or linkable datasets | Re-identification easier over time | Separate temporal identifiers |
| Small population / rare combination | May be unique despite k-anonymity | Suppress rare combinations |

**Utility assessment**: After transformation, verify that:
- [ ] Key statistical relationships are preserved (compare before/after correlations)
- [ ] Effect sizes do not change direction
- [ ] Sample sizes per analysis cell remain sufficient
- [ ] No analysis-critical variable was suppressed

## Quality Bar

The de-identification plan is **ready** when:

- [ ] Every field classified (direct / quasi / sensitive / non-identifying)
- [ ] De-identification standard selected and justified
- [ ] Transformation documented per-field with technique and parameters
- [ ] k-anonymity (or equivalent) verified and documented
- [ ] Residual risk assessment completed
- [ ] Utility impact assessment confirms analytical validity is preserved
- [ ] Key-code mapping procedure documented (if pseudonymization used)

## Minimal Output Format

```markdown
# De-identification Plan

## Dataset: [name]
## Standard: [HIPAA Safe Harbor / GDPR Anonymization / k-anonymity]

## Field Classification

| Field | Category | Transformation | Parameters | Notes |
|-------|----------|---------------|------------|-------|
| name | Direct identifier | Suppression | Removed | |
| age | Quasi-identifier | Generalization | 5-year bands | |
| zip_code | Quasi-identifier | Top-coding | First 3 digits | |
| diagnosis | Sensitive | Retained | — | Protected by QI transforms |

## Verification
- k-anonymity achieved: k = [value]
- Smallest equivalence class: [n] records
- Fields requiring further transformation: [list or "none"]

## Residual Risk Assessment
| Risk | Level | Mitigation |
|------|-------|------------|

## Utility Impact
- Correlation preservation: [% of key relationships preserved]
- Sample size impact: [any cells dropped below threshold]
```
