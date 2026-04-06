# External Capability Borrowing Framework

As the `research-skills` ecosystem grows, you will frequently encounter useful prompts, workflow structures, or capabilities created by the broader AI or academic community. 

This guide defines the rubric for assessing and integrating external ideas into the `research-paper-workflow` system without causing structural drift or violating our `task_id` contracts.

## 1. Classification & Intake

When evaluating a new external resource (e.g., a viral "Lit Review Prompt" or a new "Data Extraction Agent"), first classify what exactly you are borrowing:

1. **Provider / Tool Requirement:** Does it rely on a specific API (like searching Semantic Scholar)? 
   *➔ This belongs in the MCP bridge layer, not the prompt.*
2. **Workflow Structure:** Is it a sequence of actions (e.g., "Draft outline -> critique -> rewrite")?
   *➔ This belongs in the orchestrator pipeline definitions or multi-agent collaboration instructions.*
3. **Evaluation Rubric:** Is it a set of criteria for what makes a "good" output?
   *➔ This should be merged into our `standards/` or the specific stage's quality gates.*
4. **Canonical Skill (Prompt Body):** Is it deep domain expertise on how to execute a specific task?
   *➔ This can be adapted into a Native Skill Markdown file.*

## 2. The Golden Rule: Map to `task_id`

**Never copy-paste an external prompt wholesale if it breaks the system's contract boundary.**

Every capability in this system is bound by the `workflow-contract`. If an external prompt tries to "Write the literature review and output a PDF", it violates our contract (which mandates strict separation of research `C2`, outline `D2`, and drafting `F2`).

**Action:** You must decompose the borrowed capability and map it to existing `task_id` endpoints. If the capability covers a completely new domain, propose a new `task_id` rather than shoehorning it into an existing one.

## 3. Maintainer Checklist for Integration

Before opening a PR to merge borrowed capabilities, verify:

- [ ] **Provenance:** Is the original author or system credited in the skill `metadata` or file comments?
- [ ] **Decoupling:** Have you removed any "Roleplay" or "You are a helpful assistant" fluff? (Our system's core templates handle identity).
- [ ] **Token Efficiency:** Is the borrowed prompt condensed using our standard density practices?
- [ ] **Format Consistency:** Does it use `<output_requirements>` instead of unstructured trailing instructions?
- [ ] **Contract Alignment:** Does the prompt direct the model to save its output to the canonical path specified in the `task_id` contract?

## 4. Native Skill vs. MCP Bridge

Use the following rubric to decide where the code lives:

| Characteristic | Decision | Example |
|---|---|---|
| Requires live network access, private databases, or heavy compute | **MCP Bridge** | "Search Arxiv", "Run Python regression" |
| Rules of thumb, phrasing structures, theoretical methodologies | **Native Skill (`.md`)** | "How to write a PRISMA flowchart", "How to critique a methodology" |
| Pure formatting, LaTeX templates, Word conversions | **Post-processing script** | "Convert markdown to submission-ready PDF" |

By adhering to this framework, the core system remains predictable, modular, and orchestrator-agnostic, even as we ingest the best ideas from the community.
