---
id: tone-normalizer
stage: G_compliance
description: "Ruthlessly cut fluff, transition words, hedging, and absolute claims to produce concise, objective academic tone."
inputs:
  - type: Manuscript
    description: "Draft text requiring tone normalization"
outputs:
  - type: ToneNormalization
    artifact: "compliance/tone_normalization.md"
constraints:
  - "Must flag every hedge word and absolute claim"
  - "Must preserve technical precision while improving readability"
  - "Must not strip domain-appropriate hedging"
failure_modes:
  - "Over-cutting removes necessary nuance"
  - "Domain-specific hedging conventions not respected"
  - "Author voice completely erased (reads like a machine)"
tools: [filesystem]
tags: [compliance, tone, academic-writing, editing, conciseness]
domain_aware: false
---

# Tone Normalizer Skill

Rewrite text into concise, objective academic tone: reduce overclaiming, cut fluff, calibrate hedging, and ensure every sentence earns its place.

## Purpose

Ruthlessly cut fluff, transition words, hedging, and absolute claims to produce concise, objective academic tone.

## Related Task IDs

- `G4` (tone normalization)

## Output (contract path)

- `RESEARCH/[topic]/compliance/tone_normalization.md`

## When to Use

- After manuscript draft is complete (but before final submission)
- After AI-assisted writing (AI text tends to be wordy and pattern-heavy)
- When reviewer feedback says "needs tightening" or "writing is unclear"
- Between R&R rounds to polish revised sections

## Process

### Step 1: Scan for High-Risk Patterns

Sweep the manuscript for patterns in each category:

#### Category A: Overclaiming (Absolute Claims)

| Pattern | Example | Fix |
|---------|---------|-----|
| "proves" | "This proves that..." | "This provides evidence that..." / "is consistent with..." |
| "clearly shows" | "Clearly shows a relationship" | "Indicates a relationship" / "The results suggest" |
| "always / never" | "X always leads to Y" | "X tends to / is associated with Y" |
| "significant" (without stats) | "A significant improvement" | "A notable / substantial improvement" or report the test |
| "ground-breaking / revolutionary" | "Our ground-breaking approach" | Remove — let the reader decide |
| "unique / novel / first" | "We are the first to..." | "To our knowledge, this is among the first..." (if truly warranted) |
| "undeniable" | "The evidence is undeniable" | "The evidence is consistent across [n] specifications" |

#### Category B: Excessive Hedging (Undermines Confidence)

| Pattern | Example | Fix |
|---------|---------|-----|
| Triple hedging | "It could potentially seem to suggest that perhaps..." | "This suggests..." (one hedge is enough) |
| Modal stacking | "It might possibly be the case that..." | "It may be that..." |
| "Arguably" | "This is arguably important" | "This is important because [reason]" |
| "In some sense" | "In some sense, the results indicate..." | "The results indicate..." |

> **Rule**: One hedge per claim is appropriate. Zero hedges = overclaiming. Three+ hedges = author doubts their own work.

#### Category C: Filler and Fluff

| Pattern | Replace With |
|---------|-------------|
| "It is important to note that" | [Delete — just state the note] |
| "It should be mentioned that" | [Delete] |
| "In order to" | "To" |
| "Due to the fact that" | "Because" |
| "At this point in time" | "Currently" / "Now" |
| "A large number of" | "Many" / "[specific number]" |
| "The results of the analysis show that" | "[The specific result]" |
| "With regard to" / "In terms of" | "[specific preposition]" or restructure |
| "As a matter of fact" | [Delete] |
| "Needless to say" | [Delete — if needless, don't say it] |

#### Category D: AI-Generated Text Patterns

| Pattern | Why It's a Problem | Fix |
|---------|-------------------|-----|
| "Delve into" / "delve deeper" | AI fingerprint — rarely used in natural academic writing | "Examine" / "Investigate" / "Analyze" |
| "Landscape" (non-geographic) | "The research landscape" — overused by AI | "The field" / "The literature" / "Current research" |
| "Multifaceted" | AI favorite | "Complex" / specify the facets instead |
| "Tapestry" / "rich tapestry" | Almost never in real academic papers | Remove metaphor; be specific |
| "It's worth noting" | AI filler | [Delete — just note it] |
| "In conclusion" at every paragraph end | AI structural tic | Remove or vary: "Overall" / "Taken together" |
| "Sheds light on" | Overused in AI output | "Reveals" / "Clarifies" / "Demonstrates" |
| Excessive parallelism | Three or more parallel constructions in every section | Vary sentence structure |

### Step 2: Apply Calibrated Rewrites

For each flagged passage, produce a rewrite that:

1. **Preserves meaning** — don't change the claim
2. **Calibrates confidence** — match claim strength to evidence strength
3. **Reduces words** — target 20–30% reduction per flagged passage
4. **Maintains voice** — don't make everything sound identical

#### Confidence Calibration Guide

| Evidence Strength | Appropriate Language |
|------------------|---------------------|
| RCT / strong causal design | "demonstrates" / "shows" / "we find that" |
| Observational with controls | "is associated with" / "suggests" / "indicates" |
| Correlational / cross-sectional | "is related to" / "we observe a correlation" |
| Exploratory / qualitative | "suggests" / "our analysis points to" / "informants describe" |
| Single case / anecdotal | "appears to" / "in this case" |
| Speculation / future direction | "may" / "further research could" / "it is plausible that" |

### Step 3: Enforce Style Rules

| Rule | Check | Action |
|------|-------|--------|
| One idea per paragraph | Does each paragraph have one topic sentence? | Split paragraphs with multiple topics |
| Define acronyms once | First use fully spelled out; subsequent uses abbreviated | Find duplicated definitions |
| Active voice where possible | "We measured X" not "X was measured by us" | Rewrite passive constructions (unless convention demands passive) |
| Consistent terminology | Same concept = same word throughout | Make a terminology table |
| Numbers | Spell out 1–9; use numerals for 10+; always numeral with units | Check venue style guide |
| Citation-free claims | "Previous research has shown..." without citation | Add citation or remove claim |

### Step 4: Generate the Tone Normalization Log

Record every change for author review:

| # | Location | Original | Issue | Rewrite | Category |
|---|----------|----------|-------|---------|----------|
| 1 | § 1, ¶ 2 | "This clearly proves" | Overclaiming | "This evidence supports" | A |
| 2 | § 3, ¶ 1 | "It could potentially suggest that perhaps" | Triple hedge | "This suggests" | B |
| 3 | § 4, ¶ 3 | "delve into the multifaceted landscape" | AI pattern | "examine the field" | D |

## Quality Bar

The tone normalization is **ready** when:

- [ ] All 4 pattern categories scanned (overclaim, underhedge, filler, AI)
- [ ] Every rewrite preserves the original meaning
- [ ] Confidence language calibrated to evidence strength
- [ ] AI-generated text patterns removed or rewritten
- [ ] 20–30% word reduction achieved in flagged passages
- [ ] Author voice preserved (not robotified)
- [ ] Log compiled with original → rewrite pairs for author review

## Minimal Output Format

```markdown
# Tone Normalization Log

## Summary
- Passages flagged: [n]
- Estimated word reduction: [%]
- Categories: Overclaiming [n], Excess hedging [n], Filler [n], AI patterns [n]

## Changes

| # | Location | Original | Issue | Rewrite | Category |
|---|----------|----------|-------|---------|----------|

## Terminology Consistency
| Term used | Inconsistent variants found | Standardized to |
|-----------|---------------------------|-----------------|

## Confidence Calibration Issues
| Location | Claim strength | Evidence strength | Adjustment |
|----------|---------------|-------------------|------------|
```

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 过度删减 | 删减了必要的 hedging | 保留 evidence-based hedging（如 may, suggest） |
| 中文论文英文味 | 直译导致语体不对 | 区分中英文学术 register |
| 去掉所有 transition | 段落间缺少衔接 | 去的是 filler，留 logical connector |
| 统一 hedge 为 may | 所有句子用同一个 hedging | 变换用词：suggests, indicates, appears to |
| 忽略学科惯例 | 某些学科允许主动语态 | 核查目标期刊的 style preference |
