# Stage J — Proofread & De-AI (J1–J4)

This stage uses **multi-AI collaboration** to remove AI-generated text fingerprints, reduce similarity scores, and perform final proofreading before submission.

## Canonical outputs (contract paths)

- `J1` → `proofread/ai_detection_report.md`
- `J2` → `proofread/humanized_manuscript.md`
- `J3` → `proofread/similarity_report.md`
- `J4` → `proofread/proofread_checklist.md`

## Quality gate focus

- `Q2` (claim-evidence traceability): rewriting must NOT alter evidence or weaken claims.
- `Q5` (AI-detection score): target < 15% on major detectors after J2.

## Multi-AI Collaboration Protocol

Use `task-run --triad` or `parallel` mode for J1–J2 iteration:

1. **Drafter agent** rewrites flagged passages using diverse sentence structures, varied vocabulary, and discipline-specific idioms
2. **Reviewer agent** independently scans the rewrite for residual AI patterns (repetitive transitions, uniform sentence length, hedging clusters)
3. **Auditor agent** verifies scientific accuracy is preserved — no meaning drift, no evidence distortion

Iterate until the reviewer reports an acceptable confidence score.

---

## J1 — AI Fingerprint Scan

**Goal**: Identify passages with high AI-generation probability.

**Definition of done**
- Full manuscript scanned for common AI writing patterns:
  - Overuse of transitional phrases ("Furthermore", "Moreover", "It is worth noting")
  - Uniform sentence length and paragraph rhythm
  - Generic hedging clusters ("it is important to note that")
  - Lack of field-specific jargon or idiosyncratic phrasing
  - Formulaic structure (point–elaboration–conclusion in every paragraph)
- Each flagged passage includes:
  - Location (section + paragraph number)
  - Pattern type detected
  - Severity (high / medium / low)
  - Suggested rewrite direction

Write into: `proofread/ai_detection_report.md`.

**Recommended multi-AI approach**:
- Run 2–3 different agents independently to scan the manuscript
- Merge their detection results — passages flagged by multiple agents are highest priority
- Use `parallel --summarizer` to consolidate findings

---

## J2 — Human-Voice Rewrite

**Goal**: Rewrite flagged passages to sound authentically human-authored.

**Definition of done**
- All high-severity passages from J1 are rewritten
- Rewriting strategies applied:
  - Vary sentence length deliberately (short declarative + long complex)
  - Replace generic transitions with field-specific connectives
  - Inject author voice: personal stance markers, disciplinary conventions
  - Use concrete examples instead of abstract generalizations
  - Break formulaic paragraph structures
  - Add deliberate stylistic imperfections (parenthetical asides, rhetorical questions where appropriate)
- Scientific accuracy verified:
  - No claim distortion
  - No evidence omission
  - Statistical values and citations unchanged
  - Technical terminology preserved

Write into: `proofread/humanized_manuscript.md` (full revised manuscript).

**Recommended multi-AI approach**:
- **Agent 1 (Drafter)**: Performs the rewrite
- **Agent 2 (Reviewer)**: Re-scans the rewritten text for residual AI patterns
- **Agent 3 (Auditor)**: Diff-checks against original to ensure meaning preservation
- Loop until reviewer confidence ≥ 85%

---

## J3 — Similarity & Originality Check

**Goal**: Identify and reduce text overlap with known sources.

**Definition of done**
- Self-plagiarism check: flag reused text from author's prior publications
- Cross-reference check: flag passages too close to cited sources (paraphrase quality)
- Boilerplate detection: flag generic method descriptions that could trigger similarity tools
- Each flagged passage includes:
  - Overlap type (verbatim / close paraphrase / structural)
  - Suggested rewrite or proper attribution
- Overall similarity estimate documented

Write into: `proofread/similarity_report.md`.

---

## J4 — Final Proofread

**Goal**: Language-level polish — the last pass before submission.

**Definition of done**
- Grammar and syntax errors corrected
- Consistent use of:
  - Tense (past for methods/results, present for discussion/established facts)
  - Voice (active preferred, passive where discipline expects it)
  - Spelling convention (US vs. UK — match venue requirements)
  - Number formatting (numerals vs. spelled out)
- Acronyms defined on first use and used consistently thereafter
- Reference formatting matches venue style guide
- Figure/table cross-references verified
- No orphaned citations or dangling references

Write into: `proofread/proofread_checklist.md` with:
- Corrections log (location → issue → fix)
- Style decisions adopted
- Remaining items for author review
