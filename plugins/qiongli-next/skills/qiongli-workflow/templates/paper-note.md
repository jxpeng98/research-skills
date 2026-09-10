# Paper Note Template

<!--
Usage: Copy this template for each paper you read.
Save to: RESEARCH/[topic]/notes/[citekey].md
-->

# [Paper Title]

## Metadata

| Field | Value |
|-------|-------|
| **Authors** | |
| **Year** | |
| **Venue** | |
| **DOI** | |
| **URL** | |
| **Evidence Rating** | [ ] A [ ] B [ ] C [ ] D [ ] E |
| **Evidence Limit** | full_text / abstract_only / metadata_only / unavailable |
| **Retrieval Status** | retrieved_oa / retrieved_preprint / abstract_only / not_retrieved:<reason> |
| **Version Read** | published / accepted / submitted / preprint / abstract |
| **Read Date** | |

## Evidence Boundary

Do not invent citations, page numbers, sample sizes, methods, results, effect sizes, datasets, author claims, or implications. If the paper is not available as full text, mark `evidence_limit: abstract_only` or `evidence_limit: metadata_only` and leave unavailable fields as gaps.

Metadata search coverage and full-text access are separate. A paper found through OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv remains `abstract_only` or `metadata_only` until the retrieval manifest records a readable full-text version.

### Source Anchors

| Claim ID | Claim Type | Source Anchor | Inference Strength | Notes |
|---|---|---|---|---|
| C1 | author_claim / extracted_fact / interpretation / project_relevance | section/table/page/abstract/metadata | direct_evidence / reasonable_inference / unsupported_gap | |
| C2 | author_claim / extracted_fact / interpretation / project_relevance | section/table/page/abstract/metadata | direct_evidence / reasonable_inference / unsupported_gap | |

## Extraction Slot Summary

| Slot | Quick Capture |
|---|---|
| Theory | |
| Method / Identification | |
| Dataset / Data Source | |
| Main Finding | |
| Limitations | |

## Quick Summary

<!-- 2-3 sentence summary of the paper -->

### Author Claims

<!-- Claims the authors explicitly make. Each bullet should cite a Source Anchors row. -->

- [C1]

### Agent Interpretation

<!-- Your interpretation of what the paper means for the project. Keep inference strength visible. -->

- [C2]

## Research Problem

<!-- What problem does this paper address? Why is it important? -->

### Gap Addressed

<!-- What gap in knowledge does this paper fill? -->

## Research Questions / Hypotheses

1. **RQ1/H1**:
2. **RQ2/H2**:
3. **RQ3/H3**:

## Theoretical Framework

<!-- What theories guide this research? -->

### Key Concepts

| Concept | Definition |
|---------|------------|
| | |
| | |

## Methodology

| Aspect | Description |
|--------|-------------|
| **Research Design** | |
| **Population** | |
| **Sample** | N = |
| **Sampling Method** | |
| **Data Collection** | |
| **Data Analysis** | |
| **Validity/Reliability** | |

### Variables (for quantitative)

| Role | Variable | Measurement |
|------|----------|-------------|
| IV | | |
| DV | | |
| Control | | |
| Moderator | | |
| Mediator | | |

### Dataset / Data Source

| Field | Value |
|---|---|
| Dataset / Archive / Platform | |
| Access Type | Public / Restricted / Proprietary / Author-collected |
| Time Window | |
| Unit of Analysis | |
| Multi-source Merge | Y / N |
| Provenance Notes | |

### Core Concepts (for qualitative/theoretical)

| Concept | Definition | Operationalization |
|---------|------------|-------------------|
| | | |
| | | |
| | | |

## Key Findings

### Finding 1
<!-- Description + supporting evidence -->

### Finding 2
<!-- Description + supporting evidence -->

### Finding 3
<!-- Description + supporting evidence -->

### Statistical Results (if quantitative)

| Hypothesis | Result | Statistic | Effect Size | p-value |
|------------|--------|-----------|-------------|---------|
| H1 | Supported/Rejected | | | |
| H2 | | | | |

### Themes/Narratives (if qualitative)

| Theme | Description | Supporting Evidence |
|-------|-------------|---------------------|
| Theme 1 | | "Quote" (p. X) |
| Theme 2 | | "Quote" (p. Y) |
| Theme 3 | | "Quote" (p. Z) |

## Contributions

### Theoretical Contributions
-

### Practical Implications
-

## Limitations

<!-- Authors' acknowledged limitations and your observations -->

1.
2.
3.

### Limitation Classification

| Type | Note |
|---|---|
| Measurement | |
| Identification / Design | |
| Sample / External validity | |
| Data availability / quality | |

## Future Research Directions

<!-- What do the authors suggest? What remains unanswered? -->

1.
2.

## Connections to My Research

<!-- How does this paper relate to your work? -->

### Relevance Score: [ ] High [ ] Medium [ ] Low

### How It Informs My Research
-

### Reusable Citation Points

| Use Case | Supported Point | Source Anchor | Evidence Limit | Inference Strength |
|---|---|---|---|---|
| background / gap / method / finding / limitation | | C1 | full_text / abstract_only / metadata_only | direct_evidence / reasonable_inference / unsupported_gap |

### Questions It Raises
-

## Uncertainty Register

| Missing or Unverified Item | Why It Matters | Required Evidence | Current Handling |
|---|---|---|---|
| | | full text / supplement / dataset documentation / author clarification | leave as unsupported_gap |

## Key Quotes

> "[Quote 1]" (p. X)

> "[Quote 2]" (p. Y)

## Critical Notes

<!-- Your critical evaluation of the paper -->

### Strengths
-

### Weaknesses
-

## BibTeX (Optional)

<!-- Optional if you manage references in CSL-JSON, RIS, Zotero, or another system. -->

```bibtex
@article{citekey,
  author = {},
  title = {},
  journal = {},
  year = {},
  volume = {},
  number = {},
  pages = {},
  doi = {}
}
```

---
*Note created: [Date]*
*Last updated: [Date]*
