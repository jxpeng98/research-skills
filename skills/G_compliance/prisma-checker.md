---
id: prisma-checker
stage: G_compliance
version: "0.1.0"
description: "Verify PRISMA 2020 flow diagram completeness, checklist coverage, and reporting transparency for systematic reviews."
inputs:
  - type: Manuscript
    description: "Draft systematic review manuscript"
  - type: PRISMAFlowData
    description: "Screening flow data"
outputs:
  - type: PRISMAChecklist
    artifact: "prisma_checklist.md"
constraints:
  - "Must check all 27 PRISMA 2020 items"
  - "Must flag missing items with specific remediation"
failure_modes:
  - "Incomplete screening data prevents flow verification"
  - "Ambiguous reporting makes assessment uncertain"
tools: [filesystem, reporting-guidelines]
tags: [compliance, PRISMA, systematic-review, checklist, reporting]
domain_aware: false
---

# PRISMA Checker Skill

Verify PRISMA 2020 compliance and consistency across systematic review artifacts.

## Purpose

Ensure systematic review meets PRISMA 2020 requirements by:
- Completing the PRISMA checklist
- Verifying count consistency across documents
- Identifying missing or incomplete items
- Generating compliance report

## Process

### Step 1: Gather Review Artifacts

Locate and verify existence of required files:

```markdown
RESEARCH/[topic]/
├── protocol.md              ← Required
├── search_strategy.md       ← Required
├── search_log.md            ← Required
├── screening/
│   ├── title_abstract.md    ← Required
│   ├── full_text.md         ← Required
│   └── prisma_flow.md       ← Required
├── notes/                   ← Required (non-empty)
├── extraction_table.md      ← Required
├── quality_table.md         ← Required
├── meta_analysis_plan.md    ← Optional (if quantitative pooling)
├── effect_size_table.md     ← Optional (if quantitative pooling)
├── meta_analysis_results.md ← Optional (if quantitative pooling)
├── synthesis_matrix.md      ← Required
├── synthesis.md             ← Required
└── bibliography.bib         ← Required
```

**Status for each file:**
- [ ] Exists
- [ ] Non-empty
- [ ] Properly formatted

### Step 2: Count Consistency Check

Extract and verify counts across documents:

```markdown
## Count Extraction

| Source | Field | Value |
|--------|-------|-------|
| search_log.md | Semantic Scholar results | |
| search_log.md | arXiv results | |
| search_log.md | Google Scholar results | |
| search_log.md | Other sources | |
| search_log.md | Total identified | |
| search_log.md | Duplicates removed | |
| prisma_flow.md | Records after dedup | |
| title_abstract.md | Records screened | |
| title_abstract.md | Records excluded | |
| full_text.md | Reports sought | |
| full_text.md | Reports not retrieved | |
| full_text.md | Reports assessed | |
| full_text.md | Reports excluded | |
| prisma_flow.md | Studies included | |
| extraction_table.md | Papers extracted | |
| quality_table.md | Papers assessed | |
| synthesis_matrix.md | Papers synthesized | |
| bibliography.bib | BibTeX entries | |
```

**Consistency Rules:**
| Check | Formula | Status |
|-------|---------|--------|
| Pre-dedup total | Sum of all database results = Total identified | ✓/✗ |
| Post-dedup | Total - Duplicates = Records after dedup | ✓/✗ |
| Screening flow | Screened = Included + Excluded | ✓/✗ |
| Full-text flow | Sought = Retrieved + Not Retrieved | ✓/✗ |
| Final count | Included = Extracted = Assessed = Synthesized | ✓/✗ |
| Bibliography | BibTeX entries ≥ Included studies | ✓/✗ |

### Step 3: PRISMA Checklist Completion

For each PRISMA 2020 item, verify:

**Scoring:**
- ✓ Fully addressed (location documented)
- ○ Partially addressed (needs improvement)
- ✗ Not addressed (missing)
- N/A Not applicable (with justification)

### Step 4: Quality Indicators

Check for best practices:

**Protocol & Registration:**
- [ ] Protocol registered (PROSPERO/OSF ID present)
- [ ] Amendments documented with rationale
- [ ] Deviations from protocol explained

**Search Transparency:**
- [ ] Full search strings reproducible
- [ ] All databases with dates documented
- [ ] Snowballing/grey literature documented

**Screening Rigor:**
- [ ] Inclusion/exclusion criteria explicit
- [ ] All exclusion reasons documented
- [ ] PRISMA flow diagram complete

**Assessment Quality:**
- [ ] Appropriate RoB tool used per design
- [ ] All studies assessed
- [ ] GRADE/certainty assessment (if applicable)

**Synthesis Transparency:**
- [ ] Synthesis approach justified
- [ ] Heterogeneity addressed
- [ ] Effect measures and synthesis methods clearly reported (PRISMA 12–15, 20b–20d)
- [ ] Meta-analysis artifacts present if pooling was performed (plan, effect sizes, results)
- [ ] Limitations acknowledged

### Step 5: Generate Compliance Report

## Output Format

```markdown
# PRISMA Compliance Report

## Review: [Topic]
## Date: [Date]

---

## Executive Summary

| Category | Score | Status |
|----------|-------|--------|
| Artifact Completeness | X/12 files | ✓/○/✗ |
| Count Consistency | X/6 checks | ✓/○/✗ |
| PRISMA Checklist | X/40 items | ✓/○/✗ |
| Best Practices | X/15 items | ✓/○/✗ |
| **Overall Compliance** | **X%** | **Ready/Needs Work** |

---

## 1. Artifact Completeness

| File | Exists | Non-empty | Formatted |
|------|--------|-----------|-----------|
| protocol.md | ✓/✗ | ✓/✗ | ✓/✗ |
| search_strategy.md | ✓/✗ | ✓/✗ | ✓/✗ |
| ... | | | |

**Missing Files:**
- [List any missing required files]

---

## 2. Count Consistency

### Extracted Counts
| Document | Field | Value |
|----------|-------|-------|
| search_log.md | Total identified | |
| prisma_flow.md | Records after dedup | |
| ... | | |

### Consistency Checks
| Check | Expected | Actual | Status |
|-------|----------|--------|--------|
| Pre-dedup total | X | Y | ✓/✗ |
| Screening flow | X | Y | ✓/✗ |
| ... | | | |

**Inconsistencies Found:**
- [List any count mismatches with specific files/locations]

---

## 3. PRISMA Checklist Summary

| Section | Addressed | Partial | Missing |
|---------|-----------|---------|---------|
| Title | /1 | | |
| Abstract | /1 | | |
| Introduction | /2 | | |
| Methods | /17 | | |
| Results | /11 | | |
| Discussion | /4 | | |
| Other | /4 | | |
| **Total** | **/40** | | |

**Critical Missing Items:**
- [ ] Item X: [Description]
- [ ] Item Y: [Description]

---

## 4. Best Practices Assessment

### Protocol & Registration
| Item | Status | Notes |
|------|--------|-------|
| Protocol registered | ✓/✗ | ID: [if applicable] |
| Amendments logged | ✓/✗ | |

### Search Transparency
| Item | Status | Notes |
|------|--------|-------|
| Reproducible queries | ✓/✗ | |
| All sources dated | ✓/✗ | |

[Continue for all categories...]

---

## 5. Action Items

### High Priority (Must fix before submission)
1. [ ] [Action item]
2. [ ] [Action item]

### Medium Priority (Should address)
1. [ ] [Action item]

### Low Priority (Nice to have)
1. [ ] [Action item]

---

## 6. Submission Readiness

| Requirement | Status |
|-------------|--------|
| PRISMA checklist complete | ✓/✗ |
| Flow diagram accurate | ✓/✗ |
| All counts consistent | ✓/✗ |
| Protocol registered | ✓/✗ |
| No critical issues | ✓/✗ |

**Recommendation:** [Ready for submission / Address X issues first]

---

*Compliance check completed: [Date]*
*Checker version: 1.0*
```

## Checklist Reference

The full PRISMA 2020 checklist is available in:
`templates/prisma-checklist.md`

## Usage

This skill is called by:
- `/lit-review` Phase 9 - Final compliance verification
- Pre-submission quality check
