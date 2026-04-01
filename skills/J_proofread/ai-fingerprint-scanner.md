---
id: ai-fingerprint-scanner
stage: J_proofread
description: "Scan manuscript for AI-generation fingerprints including formulaic transitions, uniform rhythm, and generic hedging."
inputs:
  - type: Manuscript
    description: "Full draft manuscript to scan"
outputs:
  - type: AIDetectionReport
    artifact: "proofread/ai_detection_report.md"
constraints:
  - "Must scan for at least 5 pattern categories"
  - "Must assign severity (high/medium/low) to each flagged passage"
  - "Must produce actionable rewrite directions"
failure_modes:
  - "False positives on naturally repetitive academic prose"
  - "Field-specific conventions mistaken for AI patterns"
tools: [filesystem]
tags: [proofread, ai-detection, fingerprint, writing-patterns]
domain_aware: false
---

# AI Fingerprint Scanner Skill

Scan a manuscript for passages that carry common AI-generation fingerprints, producing a prioritized detection report.

## Purpose

Identify text segments with high probability of being flagged by AI detection tools, enabling targeted rewriting before submission. This is a diagnostic step — it does not rewrite anything.

## When to Use

- After the first complete manuscript draft (F2+)
- Before submission to venues with AI-detection policies
- As the first step in the `/proofread` workflow (J1)

## Related Task IDs

- `J1` (AI fingerprint scan)

## Outputs (contract paths)

- `RESEARCH/[topic]/proofread/ai_detection_report.md`

## Inputs

- Full manuscript text or `proofread/humanized_manuscript.md` from a prior iteration

## Process

### Step 1: Scan for Pattern Categories

Check for each of these AI writing fingerprints:

| # | Pattern Category | Examples | Why It Triggers Detectors |
|---|-----------------|---------|--------------------------|
| 1 | **Formulaic transitions** | "Furthermore", "Moreover", "It is worth noting", "In light of" | AI over-relies on a small set of academic connectives |
| 2 | **Uniform sentence length** | Every sentence 15–25 words with similar clause structure | Humans vary sentence length more widely |
| 3 | **Generic hedging clusters** | "It is important to note that", "This suggests that further research is needed" | Vapid qualification that adds no information |
| 4 | **Paragraph template: point–elaborate–conclude** | Every paragraph follows the same 3-part scaffold | Human writers use more diverse structures |
| 5 | **Lack of field-specific jargon** | Generic vocabulary where domain terms should appear | AI defaults to broad language |
| 6 | **Repetitive list structures** | "First, … Second, … Third, …" used identically across sections | Mechanical enumeration pattern |
| 7 | **Symmetrical parallel constructions** | "X not only … but also …" repeated across passages | Over-polished balance |

### Step 2: Flag Each Passage

For every flagged passage, record:

| Field | What to Capture |
|-------|----------------|
| **Section** | Which manuscript section (Introduction, Methods, etc.) |
| **Paragraph** | Paragraph number within section |
| **Excerpt** | The exact flagged text (15–50 words) |
| **Pattern type** | Which category from Step 1 |
| **Severity** | `high` / `medium` / `low` |
| **Rewrite direction** | One-line suggestion for how to humanize |

### Step 3: Prioritize and Summarize

- Count passages by severity level
- Identify the sections with highest concentration
- Flag any passages detected by multiple pattern categories (highest priority)

### Step 4: Multi-Agent Consensus (recommended)

When using multi-agent mode:
1. Run 2–3 agents independently on the same manuscript
2. Merge results — passages flagged by ≥ 2 agents are automatically promoted to `high`
3. Use `parallel --summarizer` to consolidate

## Quality Bar

The detection report is **ready** when:

- [ ] All pattern categories from the checklist have been scanned
- [ ] Every flagged passage has section, paragraph, excerpt, pattern, severity, and direction
- [ ] Severity distribution is documented (count per level)
- [ ] Sections with highest AI fingerprint density are identified
- [ ] No section of the manuscript has been skipped

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Flagging domain-standard phrasing | "We conducted a regression analysis" is normal | Cross-check against field norms before flagging |
| Missing structural patterns | Only checking word-level, not paragraph-level | Explicitly check paragraph rhythm and template patterns |
| Over-sensitivity to hedging | All academic papers hedge | Only flag hedging that is generic and adds no information |
| Ignoring Methods section | Methods naturally use formulaic language | Lower severity threshold for Methods but still scan |

## Output Template

```markdown
---
task_id: J1
template_type: ai_detection_report
topic: <topic>
primary_artifact: proofread/ai_detection_report.md
---

# AI Detection Report

## Summary
- Total passages flagged: [n]
- High severity: [n] | Medium: [n] | Low: [n]
- Highest-density sections: [list]

## Flagged Passages

| # | Section | ¶ | Excerpt | Pattern | Severity | Rewrite Direction |
|---|---------|---|---------|---------|----------|-------------------|
| 1 | Introduction | 3 | "Furthermore, it is important to note that..." | Formulaic transition + generic hedging | high | Replace with field-specific connective; drop empty qualifier |

## Multi-Agent Consensus (if applicable)
| # | Agent 1 | Agent 2 | Agent 3 | Consensus |
|---|---------|---------|---------|-----------|
```
