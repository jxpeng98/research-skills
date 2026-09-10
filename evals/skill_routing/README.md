# Codex routing intent probe

Codex is the primary development and verification Host. This corpus contains 24
paired English/Chinese requests (48 cases) for entry applicability, adjacent
intents, bounded scope and continuation. Inputs are synthetic, contain no private
research, and require no Host registration.

The probe supplies only the canonical `content/workflow/SKILL.md`. Codex returns
its intended primary route, scope, next needed action, and an actual text answer.
Expected labels are held out of the prompt. These are **self-reported intentions**,
not observed automatic Skill activation or resource reads. Only JSONL text and
reasoning items are allowed; tool items, unknown events, failed/incomplete turns,
nonzero exits, stale bindings and missing responses fail closed. Do not interpret
zero tool calls in this tool-disabled setup as evidence of efficient live execution.

The primary route belongs to the work still needed. Drafting a corrected passage
uses academic writing; applying an already-drafted change after its approved
revision became stale uses the existing project-operation reference. This does
not turn native runtime routing into an academic semantic classifier.

Use Python 3.11+ with the repository's existing PyYAML environment and an
authenticated Codex CLI. The initial capture is verified with CLI 0.153.4. CLI
authentication is reused; credentials are never copied into the run. The default
OpenAI provider's configured model and reasoning effort are retained. Profiles
and custom providers are currently unsupported, and require a separate adapter.
Per-invocation isolation disables user config, Host Skill discovery, plugins,
connectors and execution tools. It does not change installed configuration.

```sh
# Offline grader and malformed/missing/stale-evidence checks; no model calls.
.venv/bin/python -m unittest tests.test_skill_routing_probe

# Use a NEW directory outside the repository. Omit --case to capture all 48.
.venv/bin/python evals/skill_routing/probe.py capture /private/tmp/qiongli-codex-intents

# Re-score the complete captured selection without another model call.
.venv/bin/python evals/skill_routing/probe.py score /private/tmp/qiongli-codex-intents
```

For a subset, repeat `--case <group-id>-en` or `--case <group-id>-zh`. The summary
always gives both selected and full-corpus counts. A run directory contains the
CLI version, configured model/effort, invocation settings, source hashes, per-case
prompt/trace hashes, raw events, stderr and exit/timing records. Treat those local
logs as private; do not publish credentials, local configuration or raw reasoning.
Host failures, invalid traces and timeouts stop capture; remaining selected cases
stay missing rather than disappearing from the denominator. There is no automatic
retry. Existing output directories
are refused so a later run cannot overwrite earlier evidence.

Scoring generates temporary Evaluation Truth V1 cases and delegates every
schema assertion and suite result to `evals/runner/run_suite.py`. No core scorer,
runtime classifier, model manager or dependency is added. Source bindings cover
the entry, corpus and probe; changed inputs require a new capture. This records
provenance, not cryptographic attestation against a forged local run directory.

Review the actual answers separately for unnecessary questions, scope expansion,
invented evidence, unsupported completion and preservation of correct text.
Passing three intent labels does not establish answer quality, true activation,
full-card execution, research validity, latency superiority or release readiness.
Continuation here uses supplied state; it does not prove live revision/CAS checks.
No-Skill / preceding / candidate comparisons and actual registered-Codex tool
traces remain the next evidence slice. Other Hosts adapt these shared cases and
contracts after Codex; they do not acquire a separate product workflow.

Codex invocation and JSONL behavior follow the official
[non-interactive mode documentation](https://developers.openai.com/codex/noninteractive).
