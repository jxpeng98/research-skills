# Stage I — Research Code Support (I1–I8)

This stage makes the computational parts reproducible: implementation, pipelines, audits, and cross-model review.

## Canonical outputs (contract paths)

- `I1` → `analysis/` (method implementation)
- `I2` → `analysis/` (reproduction)
- `I3` → `analysis/` (data pipeline)
- `I4` → `code/reproducibility_audit.md`
- `I5` → `code/code_specification.md`
- `I6` → `code/plan.md`
- `I7` → `code/performance_profile.md`, `code/container_config/`, `code/documentation/`, `analysis/`
- `I8` → `code/code_review.md`

## Quality gate focus

- `Q4` (reproducibility baseline) is the primary gate in this stage.

---

## Recommended workflow pattern (CCG-style)

Use a low-freedom sequence for reliability:

1. **Specification (`I5`)**: extract constraints (I/O schema, invariants, edge cases, metrics, tooling)
2. **Planning (`I6`)**: produce a zero-decision plan (tasks, dependencies, checkpoints)
3. **Execution (`I7`)**: implement + profile + document
4. **Independent review (`I8`)**: separate model reviews logic + stats validity + failure cases
5. **Audit (`I4`)**: seeds, versions, determinism, data provenance, rerun instructions

---

## What “done” looks like for code artifacts

### `analysis/`
- Contains runnable scripts/notebooks with clear entrypoints
- Includes a minimal dataset stub or synthetic data generator for verification
- Writes outputs to a predictable location (avoid hidden state)

### `code/container_config/`
- Optional but recommended when dependencies are fragile
- Minimal `Dockerfile` or environment instructions

### `code/documentation/`
- `README.md` for how to run, reproduce, and interpret outputs

---

## Multi-model collaboration (Codex / Claude / Gemini)

Use the orchestrator to split roles:

- Codex: implementation + execution
- Claude: narrative documentation + reasoning checks
- Gemini: independent review and edge-case probing

When triad is unavailable, fall back to:
- dual-chain (generate → verify), or
- single-agent with explicit self-critique log.

