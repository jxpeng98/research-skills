---
description: 学术论文写作辅助，支持各个章节的撰写
---

# Academic Writing Assistant

Assist with academic paper writing for different sections.

Canonical Task ID (from the globally installed `research-paper-workflow` skill):
- `F2` single-section writing

## Request

$ARGUMENTS

## Supported Sections

### Title & Keywords
Help generate:
- 3–5 title candidates (venue style aware)
- 5–8 keywords aligned with field taxonomy
- A 1–2 sentence “contribution hook” for the title/abstract consistency

### Abstract
Writing a compelling abstract that:
- Hooks the reader with context/significance (1-2 sentences)
- States the research problem/gap
- Describes the methodology briefly
- Presents key findings/results
- States implications/contributions
- Follows venue-specific word limits (typically 150-300 words)

### Introduction
Writing a compelling introduction that:
- Establishes the research context and background
- Identifies the problem/gap in knowledge
- States the purpose and objectives
- Presents research questions/hypotheses
- Outlines the paper structure
- Highlights significance and contributions

### Related Work
Writing a strong related work section (common in CS/engineering venues) that:
- Organizes prior work into a clear **taxonomy/themes**
- Highlights **tradeoffs** and design space (not a citation dump)
- Positions your paper explicitly: what you do **differently** and **better**
- Builds a bridge from prior work → your method/claims

### Literature Review
Writing a critical literature review that:
- Organizes by themes, not just chronologically
- Synthesizes, not just summarizes
- Critically evaluates sources
- Identifies patterns and trends
- Highlights gaps and contradictions
- Builds toward your research rationale

### Methodology
Writing a rigorous methodology section that:
- Justifies research design choices
- Describes participants/sample
- Details data collection procedures
- Explains analytical approach
- Addresses validity and reliability
- Discusses ethical considerations

### Results
Writing a clear results section that:
- Presents primary outcomes first (with uncertainty)
- References figures/tables explicitly and explains them
- Separates results from interpretation (save for Discussion)
- Reports robustness/sensitivity checks if relevant

### Discussion
Writing an insightful discussion that:
- Interprets key findings
- Relates findings to existing literature
- Engages with theory
- Addresses research questions
- Acknowledges limitations
- Suggests practical implications

### Limitations
Writing an explicit limitations / threats-to-validity section that:
- Enumerates main threats (internal/construct/external/statistical)
- Explains how you mitigated them (and what remains)
- Avoids “cosmetic” limitations that don’t affect conclusions

### Conclusion
Writing a strong conclusion that:
- Summarizes key contributions
- Restates main findings
- Discusses theoretical implications
- Suggests practical applications
- Acknowledges limitations
- Proposes future research directions

## Workflow

### Step 1: Understand the Writing Task

Clarify:
1. Which section is being written?
2. What is the research topic?
3. What content/materials are available?
4. What is the target word count?
5. What citation style is required?
6. What is the target venue/audience?

### Step 2: Gather Required Input

For each section type, request:

**Abstract:**
- Complete draft of other sections (or key points)
- Main findings/results
- Key contribution statement
- Target word limit
- Keywords (if required)

**Introduction:**
- Research context/background
- Problem statement
- Research questions/objectives
- Key contributions

**Related Work:**
- 3–10 seed paper notes from `/paper-read` (or extraction table from `/lit-review`)
- Your intended taxonomy/themes (or ask the assistant to propose one)
- Positioning statement: “We differ from X/Y by…”
- Key comparisons you must cite (seminal + most recent)

**Literature Review:**
- Paper notes from `/paper-read`
- Extraction table from `/lit-review`
- Key themes to cover
- Gap analysis from `/find-gap`

**Methodology:**
- Research design details
- Sample/participants description
- Data collection methods
- Analysis procedures

**Results:**
- Main results (bullets) + key numbers (estimates/CI/p-values if used)
- Figures/tables available (or what you want to show)
- Robustness checks and subgroup analyses (if any)

**Discussion:**
- Key findings
- Literature review content
- Theoretical framework
- Research questions

**Conclusion:**
- Summary of findings
- Contributions
- Limitations
- Future directions

### Step 3: Draft Writing

Apply academic writing best practices:

#### Style Guidelines
- Use formal, objective language
- Avoid first person (unless disciplinary norm)
- Use hedging language appropriately
- Define technical terms
- Maintain consistent terminology
- Use active voice for clarity

#### Structure Guidelines
- Start paragraphs with topic sentences
- Use transitions between paragraphs
- Follow discipline-specific conventions
- Maintain logical flow

#### Citation Guidelines
- Integrate citations smoothly
- Use citation-formatter skill for consistency
- Avoid over-reliance on single sources
- Balance recent and seminal sources

### Step 4: Generate Output

Provide:
1. Draft text for the requested section
2. Inline citation placeholders
3. Suggestions for improvement
4. Missing information flags

## Output Format

```markdown
# [Section Name]: [Topic]

## Draft

[Full draft text with citations]

## Structure Outline

1. Paragraph 1: [Purpose]
2. Paragraph 2: [Purpose]
...

## Citations Used

| Citation | Full Reference |
|----------|---------------|
| (Author, Year) | Full bibliographic entry |

## Improvement Suggestions

- [ ] Suggestion 1
- [ ] Suggestion 2

## Missing Information

- [ ] Need more detail on...
- [ ] Missing citation for...

## Word Count

Current: X words
Target: Y words
```

## Section-Specific Templates

### Abstract Template (IMRAD Structure)

```
[HOOK - Context/Significance - 1-2 sentences]
↓
[PROBLEM - Research gap/question - 1 sentence]
↓
[METHOD - Approach/Design - 1-2 sentences]
↓
[RESULTS - Key findings - 2-3 sentences]
↓
[IMPLICATION - Contribution/Impact - 1-2 sentences]
```

**Abstract Variants by Discipline:**

| Discipline | Structure | Typical Length |
|------------|-----------|----------------|
| STEM/Medical | Structured (Background, Methods, Results, Conclusions) | 250-300 words |
| Social Sciences | Unstructured IMRAD | 150-250 words |
| Humanities | Narrative | 100-200 words |
| CS/AI Conferences | Problem-Method-Result-Impact | 150-200 words |

### Introduction Template

```
[Context/Background - 2-3 paragraphs]
↓
[Problem Statement - 1 paragraph]
↓
[Gap in Literature - 1 paragraph]
↓
[Purpose & Objectives - 1 paragraph]
↓
[Research Questions - list]
↓
[Significance - 1 paragraph]
↓
[Paper Structure - 1 paragraph]
```

### Literature Review Template

```
[Introduction to Review - 1 paragraph]
↓
[Theme 1: Title]
  - Subtheme 1a
  - Subtheme 1b
↓
[Theme 2: Title]
  - Subtheme 2a
  - Subtheme 2b
↓
[Theme 3: Title]
↓
[Synthesis & Gap - 1-2 paragraphs]
↓
[Transition to Your Study]
```

Begin academic writing assistance now.
