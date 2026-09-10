---
id: qualitative-coding
stage: E_synthesis
description: "Extract phenomena, perform thematic/grounded theory coding, and build a codebook from qualitative transcripts or extracted text."
inputs:
  - type: ExtractedText
    description: "Raw qualitative data or extracted quotes"
  - type: ResearchContext
    description: "Research question and domain context"
outputs:
  - type: DataDictionary
    artifact: "synthesis/qualitative_data_dictionary.md"
  - type: ThematicCodebook
    artifact: "synthesis/thematic_codebook.md"
constraints:
  - "Must employ systematically documented codes with clear definitions"
  - "Must establish an audit trail mapping quotes to codes and themes"
failure_modes:
  - "Shallow descriptive coding instead of interpretive/analytic coding"
  - "Forced fitting of predefined themes instead of emergent coding"
tools: [filesystem]
tags: [qualitative, thematic-analysis, grounded-theory, codebook, synthesis]
domain_aware: true
---

# Qualitative Coding Skill

Extract phenomena, perform thematic/grounded theory coding, and build a codebook from qualitative transcripts or extracted text.

## Purpose

To systematically analyze non-numerical data (e.g., interview transcripts, open-ended survey responses, textual study data) by identifying and interpreting patterns of meaning (themes).

## When to Use

Use after raw qualitative data has been collected or extracted, but before the final synthesis narrative is drafted. Ideal for Grounded Theory, Thematic Analysis, or Interpretative Phenomenological Analysis (IPA).

## Expected Inputs

- `RESEARCH/[topic]/raw_data/` (or extracted text/quotes from studies)
- `RESEARCH/[topic]/research_context.md` (for the framing of the RQs)

## Inputs

- `ExtractedText`: Raw qualitative data or extracted quotes
- `ResearchContext`: Research question and domain context
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

### Step 1: Data Familiarization
- Read through the raw excerpts or qualitative data.
- Note initial ideas, potential meanings, and context-specific jargon.

### Step 2: Open / Initial Coding
- Perform line-by-line or segment-by-segment coding.
- Generate descriptive codes closely aligned with the participants' own words (In Vivo coding).
- Document each code with a short definition.

### Step 3: Axial / Focused Coding
- Group initial codes into broader categories or sub-themes.
- Analyze relationships between the categories (e.g., conditions, actions/interactions, consequences).
- Eliminate redundant codes and refine definitions.

### Step 4: Thematic Development / Selective Coding
- Synthesize focused codes into overarching analytic themes.
- State how these themes address the primary research questions.
- Identify a "core category" if conducting grounded theory.

### Step 5: Codebook Generation
Output a structured codebook (`synthesis/thematic_codebook.md`) containing:
- **Theme name**
- **Definition** (what the theme means in context)
- **Sub-themes/Categories**
- **Exemplary quotes** (linking back to the raw text for auditability)

Also maintain a qualitative data dictionary in `synthesis/qualitative_data_dictionary.md` so constructs, participant groups, source segments, and coding conventions remain traceable during later manuscript drafting.

## Output Contract

- `DataDictionary`: write `RESEARCH/[topic]/synthesis/qualitative_data_dictionary.md`.
- `ThematicCodebook`: write `RESEARCH/[topic]/synthesis/thematic_codebook.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when producing, revising, or validating central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source pointers; unsupported central claims become `gap_note` rows and `RESEARCH/[topic]/context/gap_notes.md` entries.
- For final writing, proofread, submission, rebuttal, citation, or presentation-facing outputs, apply `references/citation-risk-policy.md` and write or update `RESEARCH/[topic]/proofread/citation-risk-report.md` when citation risk is material.

## Quality Bar

- [ ] Codes are distinct, well-defined, and cover the depth of the data
- [ ] Themes are interpretative, not simply descriptive topic summaries
- [ ] Exemplary quotes are provided for all major themes to ensure traceability
- [ ] Analysis directly addresses the overarching research question

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Descriptive Themes | Summarizing topics instead of identifying underlying meaning | Focus on *why* and *how* rather than just *what* |
| Over-coding | Creating too many fragmented codes without connection | Group codes using axial coding techniques |
| Forced Fitting | Imposing existing theories onto data that doesn't fit | Ground codes firmly in the raw data (emergent coding) |
