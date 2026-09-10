# Bounded research evidence journey

These two Evaluation Truth V1 cases share one explicitly synthetic abstract and
source registry. They check declared evidence links and requested claim coverage,
not academic truth or real Host execution. No private research or model call is
needed. Use the existing single-case runner with the shared fixture:

```sh
.venv/bin/python evals/runner/run_eval.py evals/research_journey/cases/reading-to-manuscript.yaml \
  evals/research_journey/fixtures/reading-to-manuscript --json-receipt /tmp/reading-journey.json
.venv/bin/python evals/runner/run_eval.py evals/research_journey/cases/source-to-paragraph.yaml \
  evals/research_journey/fixtures/reading-to-manuscript --json-receipt /tmp/paragraph-journey.json
.venv/bin/python -m unittest tests.test_research_journey_evals -v
```

The reading task requires reading observations. The paragraph task consumes the
source directly and does not inspect or require a new reading artifact. Scope is
declared before evaluation; deleting a required step from a failed result does
not turn it into a completed narrower task. Reordering rows, reusing evidence,
adding annotations and safely paraphrasing prose are allowed. Both requests need
the association C1 and causal limitation C2. Unused C3 can remain `needs_evidence`.

The fixed inputs are `source.md`, `sources.csv`, `references.bib` and
`required_claims.csv`; exact-byte digests bind them. Changing a source invalidates
this checkpoint, not the scientific claim itself. Review the affected claim and
bind fresh evidence before claiming verification again; never silently update a
digest to obtain a pass. Outputs are not compared to an exact prose answer.

`reading.csv` contains tabular observations of a note, summary and matrix;
`ledger.csv` retains the canonical evidence-ledger columns plus evidence scope;
`manuscript.csv` puts a passage alongside its declared claim/source mapping.
These CSVs are **test observations**, not replacements for canonical Markdown
artifacts or evidence that arbitrary Markdown has been parsed. Future model
observations must bind their projection to the actual answer and source bytes.

The case enforces source identity, locator and artifact binding, abstract-only
scope, supported status for used claims, nonempty narrative fields, and inclusion
of requested claim IDs. Cross-artifact checks compare whole tuples so swapping
sources between claims cannot hide behind matching individual column values.
In a multi-column `subset`, unused incomplete ledger rows may remain; they cannot
support a complete active claim. Paths are data in these tuples; only the pinned
source registry can establish their binding to a checked source file.

Semantic review remains necessary: a syntactically valid locator need not support
the claim, a nonempty passage may misstate the study, and a model may omit an
unmarked claim from its observations. A passing receipt proves only the declared
structural checks. It does not establish full B2 completion, causal validity,
submission readiness, installed Skill activation or live approval/CAS behavior.
The status-only full-cycle harness remains historical coverage, not this gate.
