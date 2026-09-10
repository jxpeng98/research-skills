# Codex routing intent probe

Codex is the primary development and verification Host. The corpus contains 24
paired English/Chinese requests (48 cases), covering applicability, adjacent
intents, bounded scope and continuation. Inputs are synthetic and require neither
private research nor Host registration. The current corpus retains 46 previous
requests and replaces the ambiguous Claim C1 denial pair with explicitly named
Academic Graph verification under new case IDs. Historical snapshots retain the
old requests and scores; the new pair cannot regrade those old inputs.

The probe supplies only `content/workflow/SKILL.md`, a preceding version of that
entry, or no entry. Expected labels are held out of the prompt. V2 separates:

- `route`: primary execution workflow, card or operation reference for the work
  still needed; a capability/permission block does not replace that task with
  an access operation. Applying an already-drafted change is itself an operation.
  A discovery index is not a primary task route.
- `resource_route`: a separate discovery/project-access prerequisite, or `none`.
  This includes currently blocked dependencies and does not authorize access or
  retry. These labels describe prerequisites, not observed resource reads.
- `scope`: `direct` bounded chat work or a `formal` named deliverable/workflow.
- `next_action`: the current text-only response — `answer`, `request_evidence`
  for missing user-supplied material, or `report_blocked` for unavailable project
  tools/independent review or denied access. Future steps belong in the answer.

`answer` contains the actual bounded response and requires separate human review.
These are **self-reported intentions**, not automatic activation, full-card
execution, live project/CAS checks or research-quality acceptance. Only JSONL text
and reasoning items are allowed; tool items, unknown events, failed/incomplete
turns, nonzero exits, altered bindings and missing answers fail closed. Zero tool
calls in this tool-disabled setting do not establish efficient live execution.

Use Python 3.11+, the repository's existing PyYAML environment, and authenticated
Codex CLI (verified with 0.153.4). The default OpenAI provider's configured model
and reasoning effort are retained. Profiles and custom providers need a separate
adapter. Per-invocation isolation disables user config, Host Skill discovery,
plugins, connectors and execution tools without changing installed configuration
or copying credentials. Captures stop on a Host/trace failure or timeout, without
automatic retries; unattempted selected cases stay in the denominator.

```sh
# Offline checks; no model calls.
.venv/bin/python -m unittest tests.test_skill_routing_probe tests.test_academic_quality_evals

# Always use new capture/report directories. Omit --case to capture all 48.
.venv/bin/python evals/skill_routing/probe.py capture /private/tmp/qiongli-intents \
  --case results-interpretation-boundary-en --case results-interpretation-boundary-zh

# Score the captured expectations again, without another model call.
.venv/bin/python evals/skill_routing/probe.py score /private/tmp/qiongli-intents \
  --report /private/tmp/qiongli-intents-score-2

# Explicitly change only expectations; requests and case identities must match.
.venv/bin/python evals/skill_routing/probe.py regrade /private/tmp/qiongli-intents \
  --grading-corpus evals/skill_routing/cases.yaml --report /private/tmp/qiongli-intents-regrade

# Legacy captures: verify all three source hashes against a local commit.
.venv/bin/python evals/skill_routing/probe.py regrade /private/tmp/qiongli-codex-baseline-remaining-2 \
  --legacy-ref 5471486b --adjudications evals/skill_routing/legacy-adjudications.yaml \
  --report /private/tmp/qiongli-legacy-adjudication
```

Capture also scores into `OUTPUT/scores` by default. Other score/regrade commands
use the complete captured selection; they cannot select only passing cases. New
runs snapshot the entry, corpus, prompt instruction, response schema and producer
source. Manifests bind those bytes and record CLI version, configured settings,
invocation and selection; each response binds its prompt and event SHA-256.
Current source edits do not invalidate an intact historical capture. Legacy v1
has no snapshots, so `--legacy-ref` must match entry/corpus/producer hashes; the
old instruction is read as an AST string literal, never executed. V1 is scored
with its original three fields, never upgraded by inventing missing v2 evidence.

Regrading writes a new report, preserving raw captures and original scores. The
report contains normalized observations, generated Evaluation Truth V1 cases,
canonical per-case JSON receipts, per-field results, original/revised expectations
and source/grading/scorer bindings. All assertions and suite success use the
existing `evals/runner/run_suite.py`; there is no second truth evaluator. Hashes
record provenance, not cryptographic attestation against a forged local run.
Treat local events, stderr and model responses as private; do not publish raw
reasoning, credentials or local configuration. Legacy adjudications explicitly
record post-hoc interpretations, not new model accuracy or improved answers.

For a controlled comparison, use identical case IDs and invocation settings in
three NEW captures: `--no-skill`, `--entry-ref <preceding-local-commit>`, and the
default candidate. No-Skill returns `none` for product routing fields; those two
metrics are **unassessed**, not route hits. Compare only common scope/current-action
labels and actual answer quality across all three arms; the unequal aggregate
case scores are not an accuracy ranking. Review source fidelity, unsupported
completion, unnecessary questions, scope expansion and preservation of correct
text. One small sample does not establish superiority or latency improvement.
Actual registered-Codex activation/resource/tool evidence remains separate;
other Hosts adapt shared cases after Codex rather than gain independent workflows.

## Observed resource reads (test-only)

`--read-resources` supplies the current entry and exposes one local MCP tool,
`read_resource(path)`. It serves an immutable in-memory snapshot of canonical
`content/` files with the existing bundle path mapping (`workflow/` is stripped).
It has no arbitrary filesystem or project access. The snapshot includes repository
changes that may be unpublished; running a real capture sends the entry, synthetic
request and any requested resource contents to the configured model service.

```sh
.venv/bin/python evals/skill_routing/probe.py capture /private/tmp/qiongli-resource-reads \
  --read-resources --case results-interpretation-boundary-en \
  --case continuation-academic-graph-denied-read-en --case generic-mean-function-en
```

This lane uses a distinct capture kind, `codex-resource-reading-v1`, plus reader
source and resource snapshots. The producer uses the same configured model and
per-invocation isolation; only the test reader is enabled. No Host registration or
production MCP interface is added. Historical-entry/no-Skill arms are unavailable
in this lane to avoid combining an entry with unrelated current resource bytes.

Scoring requires matched successful MCP start/completion events, exact path and
returned content matching the snapshot, and a final answer after reads. Failed,
malformed, unexpected or incomplete calls fail closed. Tool-shaped answer text
cannot count as a read. Existing intent-only captures continue rejecting tools.
The existing V1 suite checks actual primary/prerequisite reads independently of
self-reported labels. Generic non-Qiongli requests require zero resource reads.
Reports retain paths and total/unique read counts; duplicates are visible, without
inventing an efficiency threshold. Both capture-time reader source and the current
read scorer's hash are recorded separately.

These are observed deliveries through a test interface, not proof of installed
Skill activation, comprehension of every returned word, native MCP resource
support or a live project operation. Supplied denial scenarios remain synthetic.
Keep actual model outcomes distinct from mocked trace/stdio unit checks.

Codex invocation follows the official
[non-interactive mode documentation](https://developers.openai.com/codex/noninteractive)
and [MCP tool configuration](https://developers.openai.com/codex/mcp).
