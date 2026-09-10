# Evaluation Truth V1

## 1. Scope / Trigger

Use this contract when changing `evals/runner/run_eval.py`,
`evals/runner/run_suite.py`, cases under `evals/**/cases/`, or their CI owner.
It prevents an eval case or suite from passing when evidence is missing,
malformed, skipped, unreadable, or never checked.

## 2. Signatures

```python
run_case(
    case_path: str | Path,
    output_dir: str | Path | None = None,
) -> bool

main(argv: list[str] | None = None) -> int
```

```python
run_evals(
    case_dir: Path,
    fixture_root: Path | None = None,
    receipt_root: Path | None = None,
) -> EvalRunResult
```

```text
python evals/runner/run_eval.py CASE [OUTPUT_DIR]
    [--json-receipt PATH] [--junit-receipt PATH]

python evals/runner/run_suite.py [CASE_DIR] [--fixture-root PATH]
    [--receipt-root PATH]
```

The single-case Python API remains boolean; the suite returns `EvalRunResult`.
The CLI exits `0` only on success. Receipts remain opt-in; without receipt
arguments, the Python and CLI paths remain read-only. `receipt_root` writes one
existing canonical JSON receipt per case, named `<case-file-stem>.json`.

## 3. Contracts

The suite result preserves `case_count`, `passed_cases`, `failed_cases`, and a
derived `success`. With no arguments, the command resolves the repository's 12
academic-quality cases and fixtures from its own file location, independent of
the process working directory. Explicit case and fixture roots remain available
for focused runs. Cases are sorted by filename and each delegates to `run_case`
(or its CLI with `--json-receipt` when receipts are requested), evaluating once;
the suite succeeds only when at least one case ran and every case passed.

`tooling/scripts/run_academic_quality_evals.py` and the root `scripts/` entry
remain compatibility shims over this owner; they must not retain a second batch
loop or success predicate. `.github/workflows/evaluation-truth.yml` invokes the
canonical command directly for `2.x` pushes and pull requests. Native CI remains
free of Python and Node startup.

The checked-in adversarial corpus has exactly six families under
`tests/fixtures/eval_truth_v1/`: `empty-project`, `missing-artifact`, `malformed`,
`keyword-only`, `contradictory`, and `stale-artifact`. Each asserts a structured
status, stable reason code, and owned counter or assertion reason.

Each case has `schema_version: "1.0"`, an `input` object with a non-empty string
`topic`, and a non-empty `expected_outputs` object. `input.topic` is validated
before resolving the output root even when the caller supplies `output_dir`;
an explicit evidence directory does not waive the case-input contract.
Every expected output contains:

- a non-empty relative `artifact` path;
- `required` as an explicit boolean;
- a non-empty `assertions` list;
- assertions whose fields exactly match one supported type:

| Type | Required fields | Executable condition |
|---|---|---|
| `contains_all` | non-empty string `values` | every value occurs case-insensitively |
| `contains_any` | non-empty string `values` | at least one value occurs case-insensitively |
| `schema` | case-relative JSON `schema` path | JSON/YAML value or CSV row array satisfies the existing supported Schema subset |
| `field_constraint` | CSV `field`, non-empty `allowed_values` | every non-empty row value belongs to the allowlist |
| `count_conservation` | `total` label, non-empty `parts` labels | unique `Label: n = N` counts satisfy total = sum(parts) |
| `cross_artifact_consistency` | CSV `field`, output-relative `other_artifact`, `other_field`, `relation` | value/tuple multisets are `equal` or primary is a `subset` |
| `locator_syntax` | CSV `field` | every present locator is `p. N`, `pp. N-N`, or `citekey:anchor` |
| `citation_identity` | output-relative `bibliography` | every paper/theory source ID exists as a BibTeX citekey |
| `file_digest` | 64-hex `sha256` | SHA-256 matches the artifact's exact bytes |

Assertion types reject missing and extra fields. String lists are non-empty and
duplicate-free. `schema` uses the tested subset owned by
`tooling/scripts/validate_capability_contract.py`; it is not a claim of full
JSON Schema Draft 2020-12 support. Citation identity reuses
`tooling/scripts/audit_citation_risk.py` and remains separate from locator,
availability, relevance, and claim-support semantics.

Cross-artifact fields may be strings or nonempty, duplicate-free lists of equal
length. Lists compare the selected cells as one tuple; row order is irrelevant
but duplicate counts remain significant. Primary cells must be nonempty. For
multi-column `subset` only, incomplete rows on the other side may remain unused:
they cannot match a complete primary tuple. Scalar comparisons and `equal` keep
their existing blank-value rejection. Missing columns, malformed rows and empty
tables always block. CSV schema inputs reuse the same strict CSV parser; cells
remain strings with surrounding whitespace stripped, without type inference.

The bounded cases in `evals/research_journey/` verify both reading-to-writing and
direct-source scopes with shared synthetic inputs. Requiredness follows each
declared task; supported active claim links, requested claim IDs and source byte
bindings remain constraints. These are test observations, not parsed production
Markdown or scientific entailment. Free prose needs separate semantic review.

The primary artifact, bibliography, and cross-artifact references remain under
the output root. Schema references remain under the case directory. Absolute
or escaping paths are blocked. A digest may read binary bytes; text, CSV, JSON,
and YAML validators decode only their owned formats.

A case succeeds only when all five conditions hold:

```text
required_missing == 0
executed_assertions > 0
failed_assertions == 0
blocked_assertions == 0
unknown_validation_types == 0
```

`must_contain` and scalar `validation` are not V1 compatibility inputs; reject
them rather than maintaining a second parser.

### Receipt contract

Evaluation produces one ordered internal outcome list. The boolean API, JSON,
and JUnit projections consume that same result; renderers never recalculate
truth. JSON receipt version `1.0` has this shape:

```json
{
  "receipt_version": "1.0",
  "case": {
    "id": "case-id",
    "pipeline": "pipeline-id",
    "schema_version": "1.0",
    "status": "blocked",
    "reason_code": "assertion-blocked"
  },
  "summary": {
    "required_missing": 0,
    "executed_assertions": 2,
    "failed_assertions": 1,
    "blocked_assertions": 1,
    "unknown_validation_types": 0
  },
  "assertions": [
    {
      "output_id": "ledger",
      "index": 0,
      "type": "field_constraint",
      "status": "pass",
      "reason_code": "assertion-passed",
      "message": "Assertion passed.",
      "evidence": [
        {
          "role": "artifact",
          "path": "ledger.csv",
          "sha256": "0123456789abcdef..."
        }
      ]
    }
  ]
}
```

Assertion status is `pass`, `fail`, `blocked`, or `skip`; case status omits
`skip`. Configured assertions retain their zero-based index. Synthetic
`case-contract` and `output-contract` outcomes use `index: null`.
Case-load/contract failures always emit a blocked synthetic outcome. An
otherwise empty or all-skipped result adds a blocked
`no-assertions-executed` case outcome so JUnit cannot appear green.

Reason codes are stable lower-case hyphenated tokens. They include
`case-passed`, `case-load-failed`, `case-contract-invalid`,
`schema-version-unsupported`, `expected-outputs-invalid`,
`no-assertions-executed`, `assertion-passed`, `<assertion-type>-failed`,
`assertion-config-invalid`, `assertion-type-unknown`,
`assertion-evidence-unavailable`, `output-contract-invalid`,
`artifact-path-invalid`, required/optional artifact `missing`/`empty` codes,
`artifact-unreadable`, and `artifact-kind-invalid`. Portable messages are fixed
by reason code; detailed values and exception text remain human-output only.

Evidence roles are `case`, `artifact`, `schema`, `other-artifact`, and
`bibliography`. Each entry has a contained root-relative POSIX path and an
exact-byte SHA-256 only when the file exists. Primary evidence precedes any
referenced evidence. Receipts never contain raw artifact content, topics,
exception strings, absolute paths, credentials, timestamps, timings, random
IDs, or Host output.

JUnit emits one `<testsuite>` and one `<testcase>` per assertion/synthetic
outcome. `fail` maps to `<failure>`, `blocked` to `<error>`, `skip` to
`<skipped>`, and `pass` has no result child. Suite properties carry case
identity/status/reason and all five counters; testcase properties carry
assertion identity/status/reason and flattened evidence. Counts come from the
emitted outcomes.

JSON uses sorted keys, UTF-8, two-space indentation, and one trailing newline.
JUnit uses standard-library XML, a declaration, stable insertion order,
two-space indentation, and one trailing newline. Neither format emits volatile
fields. Each requested file is rendered in memory, written to a sibling
temporary file, then atomically replaced; parent directories are created.

## 4. Validation & Error Matrix

| Condition | Result |
|---|---|
| Missing optional artifact | SKIP; allowed only if another assertion executes |
| Missing or empty required artifact | `required_missing += 1`; false |
| Completed scientific/content comparison is false | `failed_assertions += 1`; false |
| Malformed case/assertion, path escape, unreadable or unparseable input | blocked; false |
| Missing referenced schema, bibliography, or cross-artifact | blocked; false |
| Empty CSV/applicable set, missing field/count, or duplicate count label | blocked; false |
| Unknown assertion type | unknown and blocked; false |
| Unsupported schema version or malformed YAML | false |
| Missing, non-object, or empty `input.topic` | blocked as `case-contract-invalid`; false, including with explicit `output_dir` |
| All artifacts skipped | `executed_assertions == 0`; false |
| JSON and JUnit destinations resolve to the same path | reject before evaluation; CLI exit `2` |
| Receipt parent cannot be created or target cannot be replaced | no partial target; CLI non-success |
| One of two receipt writes fails | per-file atomicity only; CLI non-success |
| Suite contains no `*.yaml` cases | zero cases; suite false; CLI exit `1` |
| Any delegated case is false/blocked | failed count increments; suite false; CLI exit `1` |
| Suite starts outside the repository cwd | defaults still resolve from `run_suite.py` |

## 5. Good / Base / Bad Cases

- Good: required output exists and every configured typed assertion executes
  against non-empty applicable evidence and passes; requested JSON/JUnit
  receipts agree and are byte-identical on rerun.
- Base: an optional output is absent while at least one required assertion
  executes and passes; its configured assertion is recorded as `skip`.
- Bad: an empty directory, missing `input.topic`, all-optional absence, unknown
  assertion, parse error, or failed assertion returns false and produces
  non-green receipts when receipt output is requested.
- Suite good: the default 12-case academic-quality corpus reports
  `12 passed, 0 failed` and exits zero.
- Suite base: explicit case and fixture roots preserve the same result fields
  through the canonical API and both legacy entry paths.
- Suite bad: an empty suite or any false/blocked delegated case exits non-zero.

## 6. Tests Required

Run:

```bash
python -m unittest tests.test_eval_cases tests.test_academic_quality_evals tests.test_research_journey_evals -v
python evals/runner/run_suite.py
```

The focused owner must assert all four cases pass minimal valid fixtures and
must cover required absence, zero execution, malformed/unknown assertions,
unsupported versions, read errors, malformed YAML, and optional presence and
absence. It also executes every scientific validator against temporary
JSON/YAML, CSV, Markdown, BibTeX, and binary artifacts; each semantic mismatch
must fail, while malformed configuration/data and path escapes must block.
Required-evidence deletion and count-conservation mutation coverage must first
evaluate the same freshly materialized case as `case-passed` with all nine
assertions executed and all failure counters zero. Deleting `record.json` must
then produce `required-artifact-missing`; changing the conserved total from 5
to 6 must produce `count-conservation-failed`, with each test asserting the
exact case reason, counters, and sole non-passing outcome.
The focused owner must also prove that missing `input.topic` fails with both a
derived and explicit output directory. Receipt tests must parse both formats,
compare outcome status/reason ordering,
verify exact evidence digests and reference roles, assert JUnit counts, run
twice for byte equality, reject raw/absolute canaries, exercise each flag alone
and together, preserve the no-flag default, reject a shared target, and prove a
write failure leaves no temporary file. Run the repository unit-test command
once after freezing the product/test diff; the roadmap slice closes only after
that command passes.

The focused owner must also execute all six adversarial fixture families and
assert their exact structured status, case reason, relevant counters, and
assertion reasons. Suite tests must prove default-path cwd independence, empty
and failed-suite non-success, legacy parity, single-module batch ownership, and
that the dedicated `2.x` workflow invokes only
`python evals/runner/run_suite.py` while Native CI stays Python/Node-free.

## 7. Wrong vs Correct

Wrong: relying on an explicit output directory while omitting the case input
creates a context-free case that can appear valid only on one caller path.

```yaml
schema_version: "1.0"
expected_outputs:
  quality:
    artifact: quality.md
    required: true
    assertions:
      - type: contains_all
        values: [finding]
```

Correct: every caller receives the same validated case identity and topic.

```yaml
schema_version: "1.0"
input:
  topic: "bounded academic quality case"
expected_outputs:
  quality:
    artifact: quality.md
    required: true
    assertions:
      - type: contains_all
        values: [finding]
```

Wrong: free-form text can be ignored and does not declare requiredness.

```yaml
- artifact: result.md
  validation: "contains alpha or beta"
```

Correct: requiredness and alternatives are machine-executable.

```yaml
- artifact: result.md
  required: true
  assertions:
    - type: contains_any
      values: [alpha, beta]
```

Correct scientific example: configured PRISMA labels are parsed once and their
equation is executable.

```yaml
- artifact: screening/prisma_flow.md
  required: true
  assertions:
    - type: count_conservation
      total: Records screened
      parts: [Records excluded, Reports sought for retrieval]
```

Wrong: placing raw research evidence or an absolute machine path in a receipt
makes it non-portable and can leak restricted data.

```json
{"evidence": [{"path": "/Users/me/project/result.md", "content": "..."}]}
```

Correct: identify available evidence by contained relative path and exact-byte
digest.

```json
{"evidence": [{"role": "artifact", "path": "result.md", "sha256": "..."}]}
```

Wrong: CI reaches the suite only through test discovery or a legacy wrapper.

```yaml
run: python scripts/run_academic_quality_evals.py evals/academic_quality/cases
```

Correct: the dedicated `2.x` workflow names the truth owner directly.

```yaml
run: python evals/runner/run_suite.py
```
