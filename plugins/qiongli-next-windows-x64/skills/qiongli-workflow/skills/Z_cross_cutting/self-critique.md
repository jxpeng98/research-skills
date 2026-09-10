---
id: self-critique
stage: Z_cross_cutting
description: "Review research artifacts for concrete claim, evidence and method defects; revise affected work and stop when the applicable checks are satisfied."
inputs:
  - type: AnyArtifact
    description: "Any output requiring quality assurance"
outputs:
  - type: CritiqueLog
    artifact: "review/self_critique_log.md"
constraints:
  - "Must apply structured questioning protocol"
  - "Must preserve unresolved blockers and stop when the applicable review contract is satisfied or progress needs unavailable evidence"
failure_modes:
  - "Circular critique without convergence"
  - "Inability to identify own systematic biases"
tools: [filesystem]
tags: [cross-cutting, critique, red-team, quality-assurance, Socratic]
domain_aware: false
---

# Self-Critique Skill

Review the requested research output for substantive defects without expanding
the assignment into an unrequested debate or full-stage audit.

## Purpose

Check claim-evidence alignment, methodological validity and the requested output
contract. Use adversarial review when requested or required; routine quality
checking does not require a reviewer persona or additional model.

## Inputs

- `AnyArtifact`: Any output requiring quality assurance
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

1. Identify the requested artifact, relevant stage and applicable review contract.
   Reuse the current draft and prior findings; do not generate a new draft just
   to review an existing one.
2. Check the source evidence and only the relevant stage questions below. They
   are lenses, not a checklist to exhaust or a quota of questions to invent.
3. For an ordinary check, review once. Fix concrete defects and verify the
   affected claims or outputs. Stop when those checks pass; repeat a broader
   review only for changed inputs, new evidence or the formal contract below.
4. Preserve unresolved blockers. If a fix requires unavailable evidence or a
   user decision, report the gap instead of continuing the same debate. A clean
   review may report no findings; missing evidence is not a PASS.
5. In formal runs, keep `review/self_critique_log.md` as the canonical issue
   memory. A direct answer can report its check in chat without creating a log.

## Multi-Round Self-Loop Contract

When this skill is active inside orchestrated research runs, treat critique as a stateful loop rather than isolated review comments.

The minimum counts below apply to those formal runs, not every mention of
critique, every small edit or every loaded skill. Honor the run's configured
limits; reaching a limit with unresolved issues means blocked or incomplete.

- Run a draft -> review -> targeted revision -> review loop until the reviewer passes after the minimum review count or the maximum revision count is reached.
- Standard runs require at least 2 review passes before convergence when revision rounds are available; deep runs require at least 3 review passes.
- A `BLOCK` verdict always remains blocking, even with high confidence. Confidence records certainty; it does not convert a blocker into a pass.
- A `PASS` before the minimum review count triggers a stability review of the same current draft, not immediate termination.
- Carry unresolved issues forward into the next round.
- Mark each issue as `open`, `partial`, `resolved`, or `superseded`.
- Reuse existing issue IDs when the same problem persists.
- Add new issue IDs only for genuinely new failures introduced by the revision.
- Record the artifact path, section, evidence basis, required fix, and round-to-round status change.

Use `review/self_critique_log.md` as a compact register like:

| Issue ID | Opened In Round | Status | Severity | Artifact / Section | Critique | Required Fix | Resolution Evidence |
|---|---|---|---|---|---|---|---|
| SC-01 | 0 | open | high | `manuscript/manuscript.md` / Discussion | Claim exceeds results evidence | Narrow causal language | Pending |

## Stage-Aware Grill Contract

When a task enters self-critique through a Qiongli grill request, use the same
light automatic grill and deep grill distinction as `boundary-interviewer`.

- Light automatic grill applies when the user is unsure, a stage starts with
  vague scope, a handoff contains open risks, or a claim/method/evidence/code
  decision changes.
- Deep grill applies when the user explicitly asks to be grilled, stress-tested,
  challenged like Reviewer 2, or checked for fatal flaws.
- Every critique question must target the current stage lens, include a
  recommended answer or required fix, and record whether the issue is open,
  partial, resolved, or superseded.
- A deep grill may reopen prior-stage issues only when it records the evidence or
  user decision that triggered the revisit.

## Cross-Stage Grill Memory

Self-critique issues are part of the cross-stage grill memory. Before starting a
new critique loop, inspect:

- `context/boundary_review.md`
- `context/decision_log.md`
- `context/stage_handoff.md`
- `review/self_critique_log.md`

If a prior issue affects the current artifact, keep the same issue ID and update
its status instead of creating a duplicate. Open issues that cannot be resolved in
the current stage must be copied into `context/stage_handoff.md` under `Open
Grill Issues` with a concrete `Revisit Trigger`.

## Stage-Specific Critique Questions

### Stage A: Framing & Positioning
- **Focus:** Funnel narrowing and gap validation.
- *Q1:* "Can the current RQ be answered in a single sentence? If not, is it overlapping and too massive? How can we constrain the boundaries (population, context, time) to cut the scope in half?"
- *Q2:* "Is this Gap simply because no one has done it (likely too difficult or meaningless), or involves a new data/method/theoretical dividend?"
- *Q3:* "Who cares? If this research is completely successful, which specific scholars or domains will cite it? Why?"
- *Q4:* "Does the framing rely on buzzwords without clear definitions? Which core concepts are assumed but actually contested in the literature?"
- *Q5:* "Are the proposed boundaries (e.g., geographic, temporal) justified theoretically, or merely chosen for convenience? What bias does this introduce?"

### Stage B: Literature Review
- **Focus:** Critical synthesis vs. passive summary.
- *Q1:* "Are we synthesizing or just summarizing? Have we identified the contradictions and conflicts in existing literature, rather than just agreeing with them?"
- *Q2:* "Have we been too lenient towards the highly-cited classic papers? What are their fundamental, unacknowledged limitations?"
- *Q3:* "Does the current literature review naturally and irrefutably logically lead to the proposed RQ (A1) as the only logical next step?"
- *Q4 (Confirmation Bias Check):* "Are the search keywords structurally biased towards confirming our hypotheses? What opposing search terms (null hypothesis literature) must be added?"
- *Q5:* "Are we overly reliant on WEIRD (Western, Educated, Industrialized, Rich, Democratic) samples masquerading as universal findings?"

### Stage C: Study Design
- **Focus:** Red teaming internal and external validity threats.
- *Q1 (Red Team Challenge):* "Assume this research ultimately fails or yields the exact opposite conclusion. What is the most likely fatal design/methodological flaw that caused it?"
- *Q2:* "Are we claiming causality or correlation? Do we have critical omitted confounding variables that account for the observed effect?"
- *Q3:* "Do our measurement instruments (proxies) accurately represent the abstract constructs defined in A3? Score this out of 10 and justify."
- *Q4 (Rival Hypotheses):* "What are the top 3 competing hypotheses, and exactly which variables/methods rule them out?"
- *Q5:* "Is the sample size justified by a rigorous power analysis, or based on 'rule of thumb'? What is the Minimum Detectable Effect?"

### Stage D: Ethics & IRB
- **Focus:** Extreme edge cases and participant safety.
- *Q1:* "Are there any edge cases (like de-anonymization attacks via joining multiple datasets over time) that could lead to participant data leakage? How do we defend against this mathematically or structurally?"
- *Q2:* "Is the language in the informed consent form at an 8th-grade reading level, or is it filled with academic jargon?"
- *Q3:* "Does the research involve vulnerable populations indirectly? Even if not the primary target, could they be disproportionately affected?"
- *Q4:* "What is the potential dual-use nature of these findings? Can this methodology be weaponized?"

### Stage E: Evidence Synthesis
- **Focus:** Publication bias and heterogeneity.
- *Q1 (Devil's Advocate):* "Argue that the massive aggregated effect size is 100% due to publication bias, file-drawer effect, and p-hacking. How does our data formally refute this?"
- *Q2:* "Is the heterogeneity between studies so high that we are essentially comparing apples to oranges? Justify the decision to pool mathematically (e.g., $I^2$ threshold)."
- *Q3:* "How sensitive is the overall conclusion to the removal of the specific single largest or most extreme study? (Leave-one-out sensitivity)."
- *Q4:* "Are we trusting the reported standard errors of primary studies blindly, or detecting reporting anomalies?"

### Stage F: Manuscript Writing
- **Focus:** Claim-evidence causal integrity.
- *Q1:* "Do the claims in the Discussion section drastically exceed the mathematical data support provided in the Results section?"
- *Q2:* "Does each major analytical paragraph move beyond description to explain at least one of: mechanism, tension, alternative explanation, boundary condition, or implication?"
- *Q3:* "Where are we merely restating results, themes, or citations instead of interpreting why the pattern matters?"
- *Q4:* "Are alternative explanations, contradictory evidence, and null cases confronted explicitly rather than buried in vague caveats?"
- *Q5:* "Comparing the promises made in the Introduction with the Conclusion, did we actually fulfill those promises without moving the goalposts?"
- *Q6:* "Is the limitations section honest and specific about boundary conditions, or just boilerplate text apologizing for basic boundaries?"

### Stage G: Polish & Compliance
- **Focus:** Harsh copy-editing and logical flow.
- *Q1 (Tone Check):* "Remove all unnecessary emphatic words (e.g., 'definitely', 'proves') and replace them with objective academic terms (e.g., 'suggests', 'indicates')."
- *Q2 (Logic Jump Check):* "If we strip out all transitional conjunctions (e.g., 'Therefore', 'Thus'), is the logical connection between paragraphs still solid?"
- *Q3:* "Are the active and passive voices mixed arbitrarily? Where the researchers acted, did they use active voice to take accountability?"
- *Q4:* "Is the manuscript bloated? Is there any paragraph that does not directly serve to establish the gap, methods, results, or interpretation?"

### Stage H: Submission & Revision
- **Focus:** De-escalation and reviewer empathy.
- *Q1 (Empathy Check):* "Roleplay as Reviewer 2. Would I feel this Response is brushing me off, or does it genuinely address my core concerns?"
- *Q2:* "If the reviewer asks for supplemental experiments that are impossible to conduct, is our alternative argumentation sufficiently convincing and gracious?"
- *Q3 (Tone Neutralization):* "Identify and eliminate any defensive, snarky, or argumentative tone in the rebuttal draft."
- *Q4:* "Did the revisions requested introduce contradictory statements in different parts of the manuscript (e.g., fixing methods but forgetting to update the abstract)?"

### Stage I: Code & Implementation
- **Focus:** Robustness and reproducibility.
- *Q1:* "Are there any hardcoded paths or magic numbers in the code? Is the random seed fixed globally for ALL stochastic operations?"
- *Q2:* "If 10% of the input data turns out to be NaN, will this data pipeline fail gracefully or silently produce incorrect aggregated results?"
- *Q3:* "Are the computational environment dependencies explicitly pinned (e.g., requirements.txt, Dockerfile) to prevent 'works on my machine' syndrome?"
- *Q4:* "Is there an unacknowledged O(N^2) or worse operation that will cause the code to hang if the dataset size scales 10x?"

### Stage K: Academic Presentation
- **Focus:** Audience fit, visual evidence integrity, and claim compression.
- *Q1:* "Which claim is likely to be oversimplified on slides, and what evidence or caveat must stay visible?"
- *Q2:* "Does every figure or table support the spoken argument, or is it decorative complexity?"
- *Q3:* "What would a skeptical audience member challenge first, and is the answer already on a backup slide or speaker note?"

### Stage J: Scholarly Proofreading
- **Focus:** Human scholarly voice, originality, and final integrity.
- *Q1:* "Which sentences sound polished but empty, and what concrete claim or evidence should replace them?"
- *Q2:* "Does the humanized text preserve citations, hedging, and claim strength from the source draft?"
- *Q3:* "Could any rewrite change the meaning of methods, results, limitations, or author responsibility?"

## Usage

This skill is injected into tasks by the `mcp-agent-capability-map.yaml` and should be called:
- By the **Reviewer Agent** during the `review-agent-check` phase of orchestrator runs.
- Via role-play instructions in `/paper-write`, `/study-design`, and other generative commands.

## Output Contract

- `CritiqueLog`: write `RESEARCH/[topic]/review/self_critique_log.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

## Quality Bar

- [ ] 已完成当前任务适用的检查；正式编排运行满足其最低复核轮数
- [ ] 每个 critique 点附带具体修正建议
- [ ] 每轮保留并更新 issue lineage，而不是把 critique 重置
- [ ] Overclaiming 已被识别并降级表述
- [ ] 自相矛盾点已消解或标注为 limitation
- [ ] Critique log 记录了改进前后的对比与 issue 状态迁移

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 走过场 | Critique 只说整体不错 | 说明检查了哪些主张与证据；只报告实际问题，不凑数量 |
| 过度自我批评 | 导致不敢下结论 | 区分 fatal flaw vs. minor improvement |
| 只关注表面 | 挑错别字不挑逻辑 | 按逻辑 → 证据 → 表述优先级 |
| 无 action | 批判完但不修改 | 每条 critique 必须附带 action item |
| Critique 同质化 | 相同输入反复发现同一问题 | 复用 issue ID；缺少新证据或可行修复时说明阻塞并停止 |

## When to Use

- 需要主动提高 red-teaming 强度时
- 产出可能存在浅层推理或过度主张时
- 投稿前做最后一轮逻辑检查时
- 需要 Socratic questioning 来压力测试结论时
