# Platform Routing

Use this mapping to keep behavior consistent across tools. Explicit workflow
commands are helpful shortcuts, but Qiongli routing does not require explicit
`$qiongli`, `/qiongli`, `/paper`, `/lit-review`, or slash-command invocation when the user is
doing academic research lifecycle work.

## Cross-Platform Trigger Contract

Route to Qiongli when the user request involves academic research artifacts,
judgment, or outputs:

- topic framing, research question narrowing, hypothesis, contribution, theory, or
  venue fit
- paper/PDF/note reading, citation handling, bibliography work, literature search,
  literature screening, extraction, synthesis, or gap mapping
- study design, variable construction, instruments, data management, robustness
  checks, preregistration, ethics, or IRB text
- manuscript/proposal/abstract/table/figure/discussion/rebuttal/submission text
- statistical interpretation, effect sizes, diagnostics, model output, robustness,
  meta-analysis, or evidence synthesis
- academic analysis code, notebooks, R/Stata/Python/Julia/MATLAB scripts, Quarto,
  or replication packages when they affect data, models, tables, figures, results,
  or reproducibility
- proofread, de-AI rewriting, citation-risk checking, reviewer response,
  peer-review simulation, fatal-flaw analysis, or academic presentations
- coursework, assignment briefs, marking rubrics, learning outcomes, capstone
  coursework, dissertations, theses, dissertation handbooks, supervisor
  feedback, viva preparation, or defense preparation when they require academic
  claim, evidence, source, method, or integrity judgment

Do not route to Qiongli for generic software feature work, generic file cleanup,
format conversion without scholarly interpretation, or prose edits with no claim,
evidence, citation, venue, method, or reviewer-risk consequence.

For timed exams, quizzes, or assessed problem sets, use concept explanation and
study support rather than drafting an answer for submission.

## Ambiguity Trigger

When an academic request is vague or the user asks for judgment, run a light
boundary/grill pass before producing a final artifact. Trigger phrases include:

- English: "I don't know how to start", "not sure", "help me decide",
  "which direction", "is this reasonable", "what should I do next"
- Chinese: "帮我判断", "不知道怎么做", "不确定", "方向不清楚", "帮我想想",
  "这样是否合理"

The light pass must inspect available artifacts first. If a consequential boundary
remains unresolved, ask one blocking academic question with a recommended answer
and rationale; otherwise proceed using the existing evidence and decisions. Escalate to a deep
stage-aware grill loop when the user explicitly asks to be grilled, stress-tested,
challenged like Reviewer 2, or checked for fatal flaws.

## Natural Request Routing Examples

| User intent | Route |
|---|---|
| "Read this paper / PDF / DOI" | `B2` or `/paper-read` |
| "Find gaps / I don't know where to start" | Stage A + ambiguity grill, then `A4` or `/find-gap` |
| "Run a literature review" | `B1` or `/lit-review` |
| "Improve related work" | `B4` or `/academic-write related-work` |
| "Design the study / variables / robustness" | `C1`, `C3`, `C3_5`, or `/study-design` |
| "Interpret these results" | `F3`, `F4`, `F5`, or `stats-engine` depending on artifact |
| "Modify this analysis script / notebook" | Stage I at the requested focus; reuse existing decisions and check affected behavior. Use `I5 -> I6 -> I7 -> I8` for an explicit full workflow |
| "Proofread / make it less AI-like" | Stage J or `/proofread` |
| "Prepare submission / cover letter" | `H1` or `/submission-prep` |
| "Reply to reviewer comments" | `H2`, `H2_5`, or `/rebuttal` |
| "Make slides" | Stage K or `/academic-present` |
| "Analyze this assignment brief / coursework rubric" | Stage L or `/coursework` |
| "Plan or revise my coursework essay/report/case analysis" | `L1-L7` or `/coursework` |
| "Plan my dissertation / thesis / capstone" | Stage M or `/dissertation` |
| "Integrate supervisor feedback / prepare viva questions" | `M4`, `M7`, or `/dissertation` |

## Claude Desktop / Claude.ai

- Use the installed `qiongli` skill as the user-visible main entry. Natural
  academic requests should route through this contract even when the user does
  not type a command.
- When a direct Desktop plugin exposes workflow command wrappers, `/qiongli` is
  the unified entry router and should delegate to the same canonical workflow
  files listed below.
- Treat the direct Desktop plugin as the preferred install when available. Treat
  focused Desktop/Web skill ZIPs as fallback skill-only packages for manual skill
  upload.
- Literature provider tools require the Qiongli Literature Provider MCPB, the
  direct plugin bundled literature MCP when visible, or another configured
  provider MCP. Do not claim `provider_connected` from skill instructions alone.
- Full orchestration tools require the native Full MCP server; Desktop skill ZIPs
  and the literature MCPB do not provide project orchestration.

## Claude Code

- `/qiongli` -> unified entry router that delegates by user intent
- `A1–A5` -> `/paper` (master router picks framing tasks)
- `A3` -> `/build-framework`
- `A4` -> `/find-gap`
- `B1` -> `/lit-review`
- `B2` -> `/paper-read`
- `B4` -> `/academic-write related-work [topic]`
- `C1–C5` -> `/study-design`
- `D1–D3` -> `/ethics-check`
- `E1–E5` -> `/synthesize`
- `F2` -> `/academic-write [section] [topic]`
- `F3` -> `/paper-write`
- `G1–G4` -> `/submission-prep` (reporting checks)
- `H1` -> `/submission-prep`
- `H2` -> `/rebuttal`
- `H3–H4` -> `/paper` (peer-review simulation, fatal-flaw)
- `I1–I8` -> `/code-build`
- `J1–J4` -> `/proofread`
- `K1–K4` -> `/academic-present`
- `L1–L7` -> `/coursework`
- `M1–M7` -> `/dissertation`
- Natural academic requests should route to the same task IDs even when the user
  does not type the command wrapper.
- If the request is ambiguous, use the Ambiguity Trigger before drafting.
- If full Qiongli MCP tools are installed and the request involves multi-agent
  coordination, independent review, handoff, strict gates, or auditable run artifacts,
  call `qiongli_orchestrator_route` before running a skill-only workflow. Follow
  its returned host-driven `project_list -> doctor -> start -> read/submit`
  sequence.

## Codex

- Use `$qiongli` when explicit invocation is available, but natural academic
  requests should still route to Qiongli.
- Infer `paper_type`, `task_id`, `topic` and optional `venue` from the request
  and existing context; ask only for a missing value the selected task needs.
- Follow artifact paths from workflow contract
- Assign reviewer or implementation roles by observed capabilities and evidence
  needs. Historical `primary_agent` / `review_agent` / `fallback_agent` preferences
  are not reasons to replace the user's configured model. Use
  `skills/Z_cross_cutting/model-collaborator.md` for independent review.
- For presentation tasks (`K1`–`K4`), specify backend: `slidev`, `beamer`, or `pptx`
- For coursework tasks (`L1`–`L7`), preserve assignment brief, rubric, learning
  outcomes, word count, source rules, and AI-policy status before drafting.
- For dissertation tasks (`M1`–`M7`), preserve degree level, chapter status,
  supervisor feedback, ethics dependencies, and milestone risks.
- For academic code, prioritize estimand, data lineage, diagnostics, manuscript
  tables/figures, and reproducibility over generic software scaffolding.
- If full Qiongli MCP tools are installed and the request involves multi-agent
  coordination, independent review, handoff, strict gates, or auditable run artifacts,
  call `qiongli_orchestrator_route` before running a skill-only workflow. Follow
  its returned host-driven `project_list -> doctor -> start -> read/submit`
  sequence.

## CLI And Portable Packages

- Slash-style commands remain stable skill-workflow entry points. Use
  `qiongli mcp serve --transport stdio --profile full` for host-driven project
  orchestration.
- Generic task prompt pattern: `Task {ID} on RESEARCH/[topic] using outputs defined in the active contract.`
- Task packets from other platforms should preserve `paper_type`, `stage`,
  `task_id`, `topic`, `academic_project_type`, artifact paths, and open grill
  issues.
- Orchestrator runs should carry boundary decisions and stage handoff risks into
  downstream agents rather than resetting context.

## Orchestrator MCP Escalation

Use skill-only execution for small single-agent drafting, reading, or local
editing tasks. Escalate through full MCP when the task needs runtime
coordination:

- call `qiongli_orchestrator_route` with the user's request and active platform
- call `qiongli_project_list` to select the exact registered project revision
- run `qiongli_orchestration_doctor` with the active host descriptor
- call `qiongli_orchestration_start`; execute the returned bounded handoff in
  the active Codex or Claude host
- use `qiongli_orchestration_read` for authenticated project evidence and
  `qiongli_orchestration_submit` to advance the exact checkpoint

## Worker Adapter Routing

When a handoff includes worker orchestration, use the canonical `worker_plan`.
Adapters only change dispatch mechanics:

- `generic_prompt`: portable packet for any runtime or manual dispatch.
- `codex_subagent`: Codex native subagent dispatch when available.
- `claude_cowork`: Claude native cowork dispatch when available.

If native dispatch is unavailable, record the degradation and prepare the same
packet through `generic_prompt` for an authorized reviewer. Work performed by the
same conversation remains self-review, not an independently executed packet. Do not change Task IDs, outputs, quality gates,
required skills, or MCP evidence when switching adapters.

## Portable Skill Installs

- `qiongli-workflow` portable packages must include this routing contract.
- Desktop or web skill-only installs can route and write artifacts, but they
  should not claim `provider_connected` literature search unless a provider MCP
  or MCPB is available. Platform-native search alone is `native_only`; if no
  provider MCP/MCPB and no platform-native search is available, record
  `strategy_only`.

### Project-Local Guidance

Before skill-only execution, check the current project root for:

- `.qiongli/guidance_manifest.yaml`
- `.qiongli/local_guidance.md`
- `.qiongli/guidance.d/*.md`

If `.qiongli/guidance_manifest.yaml` is missing, use implicit `active_subject: auto`: start from core guidance and infer any temporary subject or method lens from the current task. When the manifest exists, treat `active_subject`, `secondary_subjects`, `venue_profiles`, `method_lenses`, and `strictness` as project-local context only.

Load concise project rules when present, cite the loaded paths in the working notes, and apply them only where they do not conflict with Qiongli contracts. Project-local guidance must never override canonical workflow contracts, required outputs, evidence gates, quality gates, MCP evidence requirements, safety constraints, or the task packet. If local guidance conflicts with any required output or gate, follow the canonical requirement and record the conflict.

### Runtime Subject Refinement

Qiongli installs as an adaptive core workflow. Start from `active_subject: auto`
unless `.qiongli/guidance_manifest.yaml` says otherwise. During a task, infer
whether the request needs core-only guidance, a borrowed method lens, a suggested
subject, a confirmed subject, or a locked subject.

Do not switch the whole project subject from a single method signal. A management
paper that uses an event study borrows finance event-study diagnostics; it is not
automatically a finance paper. A political science paper that uses DID borrows
economics identification diagnostics; it is not automatically an economics paper.

Use `subject_refinement.borrowed_lenses` as temporary method guidance. Use
`subject_refinement.primary_subject` as the temporary subject only when the
decision is `suggest_subject`, `confirm_subject`, or `lock_subject`. Persist
changes only through project-local guidance proposals or an explicit
`subject_mode: confirmed` or `subject_mode: locked` manifest.

### Subject Domain Packs

When Qiongli is installed through the CLI with a subject package, treat the subject-installed domain profile as the specialization layer for the canonical workflow. If `SUBJECT_MANIFEST.json` names `economics`, load `skills/domain-profiles/economics.yaml`; if it names `finance`, load `skills/domain-profiles/finance.yaml`. If project guidance is `active_subject: auto`, infer a temporary economics or finance domain from the task context, then apply the same profile rules.

Domain profiles refine canonical contracts; they do not replace them. For each matched method, apply `canonical_references` as method anchors, `gate_relevance` as Q1-Q4 routing hints, `diagnostic_artifacts` as required local evidence, and `failure_triggers` as blocker language for unsupported claims.


## Native 2.x compatibility and recovery

The Host runs the configured model, including a third-party model behind a
compatible API. Model names, reasoning effort, authentication and API wire
formats belong to that Host. Skills use the visible tool interface, not raw
provider-specific tool-call text. A configured endpoint does not imply support
for every document, vision, search or multi-agent feature of the Host.

`python -m bridges.orchestrator`, legacy `qiongli provider setup/doctor`,
`qiongli install --target ... --parts mcp`, and controller flags such as
`--mcp-strict` / `--skills-strict` belong to 1.x compatibility guidance. They
are not native 2.x commands or Plugin dependencies. If encountered in older
cards, use the current Host workflow and exposed tool schemas; do not run or
install the old controller to make a native task work. An MCP capability label
inside a card is a requirement to resolve, not proof of a callable tool.

When Qiongli tools are absent, explain that the installed Plugin must be enabled
and the Host reconnected, or its MCP configured to launch the absolute installed
CLI with `mcp serve --profile full --transport stdio`. Installing a CLI alone
does not register or refresh a Plugin. Do not change Host configuration as a
side effect of research. Missing MCP blocks live project operations but does not
block a bounded answer from supplied material; no persistence or live revision
verification may be claimed for that answer.
