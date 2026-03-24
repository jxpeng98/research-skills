---
id: paper-extractor
stage: B_literature
version: "0.2.2"
description: "Extract structured information from papers including bibliographic metadata, methodology, findings, and contributions."
inputs:
  - type: ScreeningDecisionLog
    description: "Papers that passed screening"
  - type: FullTextAccess
    description: "Full-text PDFs or URLs"
outputs:
  - type: ExtractionTable
    artifact: "extraction_table.md"
  - type: PaperNotes
    artifact: "notes/"
constraints:
  - "Must extract all 7 framework categories (bibliographic, context, theory, methodology, findings, discussion, contributions)"
  - "Must generate one note per paper under notes/"
failure_modes:
  - "Inconsistent reporting across papers"
  - "Missing methodology details in source papers"
tools: [filesystem, extraction-store]
tags: [literature, extraction, data-collection, structured-notes]
domain_aware: false
---

# Paper Extractor Skill

Extract structured information from academic papers for analysis and synthesis.

## Purpose

Systematically extract key information from papers including:
- Bibliographic metadata
- Research design elements
- Findings and contributions
- Methodological details

## Granularity Boundary

The following are structured extraction slots inside `paper-extractor`, not separate top-level skills:

- Methodology extraction
- Dataset / data-source extraction
- Theory extraction
- Limitation extraction

If a new requirement only changes one slot, extend `templates/paper-note.md` or `templates/extraction-table.md` rather than adding another extractor skill.

## Related Task IDs

- `B2` (targeted paper reading)
- `B1` (systematic review pipeline)
- Supports synthesis: `E2` (effect size table), `E5` (integrated synthesis)

## Outputs (contract paths)

- Paper notes → `RESEARCH/[topic]/notes/`
- Extraction rollup → `RESEARCH/[topic]/extraction_table.md`
- (If pooling) effect inputs → `RESEARCH/[topic]/effect_size_table.md`

Template references:
- `templates/paper-note.md`
- `templates/extraction-table.md`
- `templates/effect-size-extraction-table.md` (when pooling)

## Extraction Framework

### 1. Bibliographic Information

| Field | Example |
|-------|---------|
| Title | |
| Authors | (Last, First; Last, First; ...) |
| Year | |
| Venue | Journal/Conference name |
| Volume/Issue | |
| Pages | |
| DOI | |
| URL | |

### 2. Research Context

| Field | Extraction Prompt |
|-------|------------------|
| Research Problem | What problem does this paper address? |
| Motivation | Why is this problem important? |
| Research Gap | What gap in knowledge does this fill? |
| Objectives | What are the stated objectives? |
| Research Questions | What RQs or hypotheses are presented? |

### 3. Theoretical Elements

| Field | Extraction Prompt |
|-------|------------------|
| Theoretical Framework | What theories guide the research? |
| Key Concepts | What are the main constructs/concepts? |
| Conceptual Model | Is there a conceptual/research model? |
| Hypotheses | What hypotheses are tested? |

### 4. Methodology

| Field | Extraction Prompt |
|-------|------------------|
| Research Design | Qualitative/Quantitative/Mixed? Experimental? |
| Population | Who/what is studied? |
| Sampling | How was sample selected? Size? |
| Data Collection | What instruments/methods? |
| Data Sources | Primary/Secondary? What sources? |
| Variables | IVs, DVs, Controls, Moderators, Mediators? |
| Analysis Methods | What analytical techniques? |
| Validity/Reliability | What measures of rigor? |

### 4.5 Dataset / Data Source Slot

| Field | Extraction Prompt |
|-------|------------------|
| Data Source | What dataset, archive, platform, or field setting is used? |
| Access Type | Public / restricted / proprietary / collected by authors? |
| Time Window | What observation period is covered? |
| Unit of Analysis | Individual / firm / country / document / etc. |
| Merge / Linkage | Are multiple sources combined? How? |

### 5. Findings

| Field | Extraction Prompt |
|-------|------------------|
| Key Findings | What are the main results? |
| Hypothesis Support | Which hypotheses supported/rejected? |
| Effect Sizes | What are the reported effect sizes? |
| Statistical Results | Key statistics (p-values, R², etc.) |
| Themes | (For qualitative) What themes emerged? |

### 6. Discussion Elements

| Field | Extraction Prompt |
|-------|------------------|
| Interpretation | How do authors interpret findings? |
| Theory Engagement | How do findings relate to theory? |
| Literature Dialogue | How do findings compare to prior work? |
| Implications | Theoretical and practical implications? |

### 7. Contributions & Limitations

| Field | Extraction Prompt |
|-------|------------------|
| Theoretical Contribution | What new theoretical knowledge? |
| Methodological Contribution | Any methodological innovations? |
| Practical Contribution | What practical applications? |
| Limitations | What limitations acknowledged? |
| Future Research | What future work suggested? |

These slots should be reflected both in the per-paper note and in the rollup extraction table.

## Extraction Output Template

Write one note per paper under `notes/{citekey}.md`, and then roll up key fields into `extraction_table.md`.
Use the paper note template as the canonical note structure instead of creating separate slot-specific note files.

```markdown
# Paper Extraction: [Short Citation]

## Bibliographic Information
| Field | Value |
|-------|-------|
| Title | |
| Authors | |
| Year | |
| Venue | |
| DOI | |

## Research Context
**Problem Statement:**
[Text]

**Research Gap:**
[Text]

**Research Questions:**
1. RQ1: 
2. RQ2:

## Theoretical Framework
**Theories Used:**
- Theory 1: [Brief description]
- Theory 2: [Brief description]

**Key Concepts:**
| Concept | Definition |
|---------|------------|
| | |

## Methodology
| Aspect | Description |
|--------|-------------|
| Design | |
| Population | |
| Sample | N = |
| Data Collection | |
| Analysis | |
| Validity | |

## Key Findings
1. Finding 1: [Description + Evidence]
2. Finding 2: [Description + Evidence]
3. Finding 3: [Description + Evidence]

## Statistical Results (if quantitative)
| Hypothesis | Result | Effect Size | p-value |
|------------|--------|-------------|---------|
| H1 | Supported/Rejected | | |

## Contributions
**Theoretical:**
- 

**Practical:**
- 

## Limitations
1. 
2. 

## Future Research Suggestions
1. 
2. 

## Relevance to My Research
[Personal notes on how this relates to your work]

## Key Quotes
> "[Important quote 1]" (p. X)

> "[Important quote 2]" (p. Y)
```

## Batch Extraction Table

For multiple papers, create summary extraction table:

```markdown
| ID | Authors | Year | RQs | Theory | Method | Sample | Key Findings | Quality |
|----|---------|------|-----|--------|--------|--------|--------------|---------|
| 1 | | | | | | | | A-E |
| 2 | | | | | | | | A-E |
```

At minimum, the rollup should preserve:
- Theory / framework slot
- Method / identification slot
- Dataset / source slot
- Main findings slot
- Limitations slot

## Usage

This skill is called by:
- `/lit-review` - During extraction phase
- `/paper-read` - For deep reading
