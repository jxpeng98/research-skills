# Multi-Agent Runtime Guide

Use this guide when you are running `parallel`, `task-run`, `team-run`, or any workflow that coordinates Codex, Claude, and Gemini under the orchestrator.

## What This Guide Covers

This page explains:

- how the orchestrator routes work across multiple runtimes
- when Gemini runs through `broker`, `direct`, or `auto`
- which authentication method is valid for each path
- how to start a stable resident Gemini path for Google-login-only subscriptions
- what is supported on desktop, CI, and headless servers

## Runtime Topology

The orchestrator has three runtime agents:

- `codex`
- `claude`
- `gemini`

For `codex` and `claude`, the runtime model is simple: the orchestrator performs a preflight check, then launches the corresponding CLI directly.

For `gemini`, the runtime model is split:

- `direct`: launch Gemini CLI as a one-shot subprocess
- `broker`: send work to a long-lived local broker
- `auto`: try broker first, then fall back to direct only if broker is unavailable and direct auth is valid

In current builds, the builtin Gemini broker uses a resident `gemini --acp` backend by default.

## Why Gemini Needs Two Paths

Gemini has two materially different automation cases:

1. `API key` or `Vertex` auth:
   works well for non-interactive subprocess calls
2. `Google login` subscription:
   works reliably only when the Gemini session stays resident inside the broker path

This is the distinction that matters:

- a terminal saying `Signed in with Google /auth` is enough for manual interactive Gemini usage
- it is not a reliable signal for orchestrator `direct` mode
- it is acceptable for the resident `broker` path, because the broker keeps one long-lived Gemini ACP process instead of relaunching a fresh subprocess for every task

## Gemini Transport Modes

You can control Gemini transport globally:

```bash
export RESEARCH_GEMINI_TRANSPORT=auto
```

Or per profile through `runtime_options.gemini.transport`.

Supported values:

- `auto`
- `broker`
- `direct`

### `auto`

Use this as the default.

Behavior:

1. check whether the broker is configured and healthy
2. if healthy, route Gemini work to the broker
3. otherwise, try direct Gemini CLI only when non-interactive direct auth is valid

This is the safest setting for mixed environments.

### `broker`

Use this when you want Gemini work to go only through the resident broker.

Behavior:

- requires `RESEARCH_GEMINI_BROKER_URL`
- does not require `GEMINI_API_KEY` on the orchestrator side when broker auth is handled inside the resident Gemini session
- is the correct setting for Google-login-only subscriptions

### `direct`

Use this only when you intentionally want one-shot Gemini subprocess calls.

Behavior:

- bypasses the broker even if one is running
- requires non-interactive direct auth
- should not rely on cached browser login

## Authentication Matrix

| Auth mode | `broker` | `direct` |
|---|---|---|
| `GEMINI_API_KEY` | Supported | Supported |
| Vertex AI env auth | Supported | Supported |
| Cached Google login | Supported through resident ACP broker | Not reliable |

The practical rule is simple:

- `Google login only` => use `broker`
- `API key` or `Vertex` => `auto` or `direct` both work, but `auto` is still preferred

## Starting the Resident Gemini Broker

### Standard desktop flow

Start the broker in a desktop session:

```bash
python3 scripts/gemini_session_broker.py --backend acp --host 127.0.0.1 --port 8767
```

Then point the orchestrator at it:

```bash
export RESEARCH_GEMINI_BROKER_URL=http://127.0.0.1:8767
export RESEARCH_GEMINI_TRANSPORT=broker
```

If you want the orchestrator to fall back to direct auth when the broker is down:

```bash
export RESEARCH_GEMINI_TRANSPORT=auto
```

### Custom ACP command

If Gemini is installed in a nonstandard location or needs extra flags:

```bash
export RESEARCH_GEMINI_ACP_CMD="gemini --acp"
python3 scripts/gemini_session_broker.py --backend acp --host 127.0.0.1 --port 8767
```

### Legacy one-shot backend

If you explicitly need the old broker behavior:

```bash
python3 scripts/gemini_session_broker.py --backend cli --host 127.0.0.1 --port 8767
```

That path does not preserve a resident Google-login session. Use it only for API-key-style automation.

## Running Multi-Agent Workflows

## Automated Smoke Harness

If you want a repeatable local validation instead of running every check manually, use the Codex-first smoke harness:

```bash
python3 scripts/smoke_multi_agent.py \
  --cwd . \
  --transport broker \
  --start-broker \
  --run-parallel
```

What it does:

- runs `doctor`
- probes a real Codex runtime call
- probes a real Gemini runtime call through the selected transport
- optionally runs a Codex-synthesized `parallel` smoke
- writes JSON and Markdown receipts under `output/test_runtime/`

Recommended local variants:

```bash
# Google-login Gemini on desktop
python3 scripts/smoke_multi_agent.py --cwd . --transport broker --start-broker

# Broker first, direct fallback if available
python3 scripts/smoke_multi_agent.py --cwd . --transport auto --start-broker --run-fallback-check

# Direct Gemini only, for API-key or Vertex testing
python3 scripts/smoke_multi_agent.py --cwd . --transport direct --strict-gemini
```

### Preflight

Run `doctor` before a larger workflow:

```bash
python3 -m bridges.orchestrator doctor --cwd .
```

For Gemini, `doctor` now reports three separate lines:

- `Gemini transport`
- `Gemini broker`
- `Gemini direct auth`

Read them independently. A healthy broker with missing direct auth is a valid configuration when transport resolves to broker.

### `task-run`

Typical usage:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

In triad mode, Gemini is just one participant in the runtime plan. If Gemini transport resolves to broker, the orchestrator sends Gemini work to the broker while Codex and Claude continue to run directly.

### `parallel`

Use this when you want independent drafts or reviews from multiple agents. The orchestrator preflights each runtime before spending tokens on a large prompt. That means a broken Gemini direct path no longer needs to consume the full review prompt if broker routing is available.

### `team-run`

`team-run` benefits from the same routing rules. The only extra requirement is consistency: decide whether Gemini is expected to be present for all phases or whether it may be skipped under fallback routing.

## Profile-Level Control

You can pin transport inside an agent profile:

```json
{
  "runtime_options": {
    "gemini": {
      "transport": "broker",
      "approval_mode": "plan"
    }
  }
}
```

Use this when:

- a project must always use resident Google-login Gemini
- a CI profile must always avoid broker-only assumptions
- a local review profile wants Gemini in read-only `plan` mode

## Environment Support Matrix

### Local desktop

Supported:

- `codex` direct
- `claude` direct
- `gemini` direct with API key or Vertex
- `gemini` broker with resident ACP
- `gemini` broker with cached Google login

This is the primary environment for Google-login Gemini automation.

### CI

Supported:

- `codex` direct
- `claude` direct
- `gemini` direct with API key or Vertex

Not recommended:

- resident Google-login Gemini broker

Reason: CI should not depend on browser login or a human-maintained desktop session.

### Pure headless server

Supported:

- `codex` direct
- `claude` direct
- `gemini` direct with API key or Vertex
- `gemini` broker only if the broker itself uses non-interactive auth

Not recommended:

- cached Google-login Gemini broker without a stable resident desktop session

## Failure Modes

### Broker is configured but not healthy

In `broker` mode:

- Gemini is unavailable
- orchestrator reports a broker transport failure

In `auto` mode:

- orchestrator tries direct Gemini only if direct auth is valid

### Direct auth missing

This matters only when transport resolves to `direct`.

If transport resolves to `broker`, missing direct auth is not a failure by itself.

### Auth lost inside the resident broker

The broker marks Gemini auth as lost when the last prompt fails with an auth-like error. Reset the broker state or restart the broker after re-authentication.

## Recommended Defaults

Use these defaults unless you have a stronger reason not to:

- local desktop with Google login only:
  - start the ACP broker
  - set `RESEARCH_GEMINI_TRANSPORT=broker`
- local desktop with API keys:
  - set `RESEARCH_GEMINI_TRANSPORT=auto`
- CI:
  - use API keys or Vertex
  - avoid Google-login-only assumptions

## Related Pages

- [Quickstart](/quickstart)
- [Task Recipes](/guide/task-recipes)
- [Troubleshooting](/guide/troubleshooting)
- [Architecture](/architecture)
