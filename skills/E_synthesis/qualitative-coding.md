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
