# Paper Reading Summary Template

<!--
Usage: Maintain this file as the project-level summary of targeted B2 paper reading.
Save to: RESEARCH/[topic]/literature/paper_reading_summary.md
Source notes: RESEARCH/[topic]/notes/[citekey].md
-->

# Paper Reading Summary

## Evidence Boundary

Do not invent citations, page numbers, sample sizes, methods, results, effect sizes, datasets, author claims, or implications. Do not upgrade an inference into a fact. If a claim cannot be traced to a paper note, source section, quote, table, abstract, or metadata field, record it as `unsupported_gap` in the uncertainty register.

Use these controlled labels:

- `evidence_limit`: `full_text`, `abstract_only`, `metadata_only`, `unavailable`
- `inference_strength`: `direct_evidence`, `reasonable_inference`, `unsupported_gap`
- `source_anchor`: citekey plus section, page, table, quote ID, abstract, or metadata field

## Corpus Overview

| Metric | Value | source_anchor | evidence_limit |
|---|---|---|---|
| Papers summarized | | notes/ index | full_text / abstract_only / metadata_only |
| Full-text notes | | retrieval_manifest.csv | full_text |
| Abstract-only notes | | retrieval_manifest.csv | abstract_only |
| Metadata-only notes | | retrieval_manifest.csv | metadata_only |

## Theme Clusters

| Theme | Papers | Grounded Summary | source_anchor | evidence_limit | inference_strength |
|---|---|---|---|---|---|
| | | | citekey:section/table/page | full_text / abstract_only / metadata_only | direct_evidence / reasonable_inference / unsupported_gap |

## Method And Data Patterns

| Pattern | Papers | What Is Known | source_anchor | evidence_limit | inference_strength |
|---|---|---|---|---|---|
| Method / identification | | | citekey:method section | full_text / abstract_only / metadata_only | direct_evidence / reasonable_inference / unsupported_gap |
| Dataset / source | | | citekey:data section | full_text / abstract_only / metadata_only | direct_evidence / reasonable_inference / unsupported_gap |

## Stable Findings

Only list a finding as stable when multiple notes support it or when the summary explicitly says it is a single-paper finding.

| Finding | Supporting Papers | Boundary | source_anchor | evidence_limit | inference_strength |
|---|---|---|---|---|---|
| | | single-paper / multi-paper / tentative | citekey:section/table/page | full_text / abstract_only / metadata_only | direct_evidence / reasonable_inference |

## Contradictions And Contested Claims

| Claim Area | Paper A | Paper B | Nature of Tension | source_anchor | Current Handling |
|---|---|---|---|---|---|
| | | | methods differ / populations differ / findings conflict / definitions differ | citekey:anchor; citekey:anchor | keep as uncertainty / needs B3 / needs B6 / needs E synthesis |

## Research Gaps

| Gap | Evidence For Gap | Why It Matters | source_anchor | inference_strength |
|---|---|---|---|---|
| | | | citekey:limitations/future research/gap section | direct_evidence / reasonable_inference / unsupported_gap |

## Implications For Current Project

Separate what papers say from how the current project may use it.

| Project Use | Grounded Input | Project Interpretation | source_anchor | inference_strength |
|---|---|---|---|---|
| framing / theory / method / related work / limitations | | | citekey:anchor | direct_evidence / reasonable_inference / unsupported_gap |

## Writing-Ready Citation Points

| Draft Location | Citation Point | Candidate Citation(s) | source_anchor | evidence_limit | Boundary Note |
|---|---|---|---|---|---|
| introduction / related work / methods / discussion | | | citekey:anchor | full_text / abstract_only / metadata_only | Do not cite beyond what the anchor supports. |

## Uncertainty Register

| unsupported_gap | Affected Summary Claim | Missing Evidence | Next Action | Do Not Claim As Fact |
|---|---|---|---|---|
| | | full text / supplement / dataset documentation / additional papers | retrieve / screen / snowball / ask user | |

---
*Summary created: [Date]*
*Last updated: [Date]*
