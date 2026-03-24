# Paper Type Playbooks

This page gives standard example routes for the five canonical paper types:

- `systematic-review`
- `empirical`
- `qualitative`
- `methods`
- `theory`

These are not the only valid routes.
They are the recommended defaults when you want a defensible baseline workflow.

## How To Read These Examples

Each playbook includes:

- a recommended route
- a narrower route for lighter work
- key skills usually involved
- typical outputs
- a starter command

Use them as operating defaults, then narrow or deepen based on your actual constraints.

## 1. Systematic Review

### Use this when

You are building a PRISMA-style review, evidence synthesis, or structured related-work base with transparent search and screening logic.

### Recommended route

1. `A1`: clarify question and scope
2. `B1`: run reproducible search
3. `B1_5`: refine concepts and Boolean logic
4. `B2`: extract papers
5. `B3`: map the literature
6. `E1`: synthesize evidence
7. `E2`: assess quality / risk of bias
8. `G1`: run PRISMA check
9. `F3`: write the review manuscript

### Narrower route

Use this when you already have a stable corpus:

1. `B2`
2. `B3`
3. `E1`
4. `F3`

### Typical skills

- `academic-searcher`
- `concept-extractor`
- `paper-screener`
- `paper-extractor`
- `literature-mapper`
- `evidence-synthesizer`
- `quality-assessor`
- `prisma-checker`
- `manuscript-architect`

### Typical outputs

- search log
- screening log
- extraction table
- literature map
- synthesis memo or meta-analytic result
- quality assessment
- PRISMA compliance report
- manuscript draft

### Starter command

```bash
python3 -m bridges.orchestrator task-run \
  --task-id B1 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd . \
  --research-depth deep
```

### Common narrowing rule

If the review starts producing too many auxiliary artifacts, stay with:

- `B2`
- `B3`
- `E1`
- `F2` or `F3`

and use `--focus-output` plus `--output-budget`.

## 2. Empirical Paper

### Use this when

You are writing a standard empirical paper with a design, dataset, analysis, interpretation, and submission path.

### Recommended route

1. `A1`: define question
2. `A1_5`: generate hypotheses
3. `C1`: build the design
4. `C2` / `C3`: operationalize variables and validate data path
5. `C4`: specify robustness logic
6. `I1` / `I2` / `I3` or Stage-I code lane if implementation is substantial
7. `F1`: manuscript structure
8. `F3`: full draft
9. `F4`: tables/figures/results support
10. `G2`: reporting check
11. `H1`: submission package

### Narrower route

Use this when the study is already run and you mainly need writing plus checks:

1. `F1`
2. `F3`
3. `F4`
4. `G2`
5. `H1`

### Typical skills

- `question-refiner`
- `hypothesis-generator`
- `study-designer`
- `variable-constructor`
- `dataset-finder`
- `robustness-planner`
- `analysis-interpreter`
- `table-generator`
- `figure-specifier`
- `reporting-checker`

### Typical outputs

- question and hypothesis set
- design spec
- variable / dataset plan
- robustness plan
- manuscript draft
- tables and figure specs
- reporting compliance memo
- submission bundle

### Starter command

```bash
python3 -m bridges.orchestrator task-run \
  --task-id C1 \
  --paper-type empirical \
  --topic policy-effects \
  --cwd .
```

### Common decision rule

If code is light, stay in the writing/design route.
If code becomes central to the paper, switch into the full Stage-I code lane instead of using generic drafting alone.

## 3. Qualitative Paper

### Use this when

The paper’s core evidence comes from interviews, case studies, ethnography, documents, or process tracing, and the goal is analytic depth about meaning, mechanism, or process rather than statistical estimation.

### Recommended route

1. `A1`: refine the qualitative research question, setting, and unit of analysis
2. `A1_5`: define working propositions or sensitizing concepts
3. `A3`: anchor the theoretical lens or process framing
4. `A4`: specify the qualitative gap and expected contribution
5. `B2`: targeted paper reading
6. `B6`: literature map around mechanism, process, and rival explanations
7. `C1`: build the qualitative design
8. `C2`: write interview / observation / document protocols
9. `C3`: lock coding, memoing, and comparison logic
10. `C1_5`: define rival interpretations / disconfirming cases
11. `D1`: ethics and data governance package
12. `F1`: manuscript structure
13. `F3`: full qualitative draft
14. `G1`: reporting check (SRQR / COREQ)
15. `H4`: fatal-flaw stress test

### Narrower route

Use this when fieldwork is done and you mainly need analysis-to-manuscript conversion:

1. `C3`
2. `F1`
3. `F3`
4. `G1`
5. `H1`

### Typical skills

- `question-refiner`
- `hypothesis-generator`
- `theory-mapper`
- `gap-analyzer`
- `paper-extractor`
- `literature-mapper`
- `study-designer`
- `rival-hypothesis-designer`
- `analysis-interpreter`
- `reporting-checker`
- `manuscript-architect`

### Typical outputs

- qualitative RQ set and contribution memo
- case / participant sampling logic
- interview guide or observation protocol
- coding and memoing plan
- findings interpretation memo
- manuscript draft
- SRQR / COREQ checklist
- fatal-flaw memo

### Starter command

```bash
python3 -m bridges.orchestrator task-run \
  --task-id C1 \
  --paper-type qualitative \
  --topic platform-governance-practices \
  --domain business-management \
  --cwd .
```

### Common decision rule

Use the qualitative route when the paper needs deep explanation of process, meaning, interpretation, or mechanism and the evidence base is interviews, cases, fieldnotes, or documents rather than a model-ready dataset.

## 4. Methods Paper

### Use this when

The core contribution is a method, algorithm, pipeline, or code-supported procedure, and the code is first-class evidence.

### Recommended route

1. `A1`: define problem and contribution claim
2. `A3`: frame theory or methodological positioning
3. `C1`: state evaluation design
4. `I5`: code specification
5. `I6`: zero-decision plan
6. `I7`: implementation and profiling
7. `I8`: review logic and statistical validity
8. `I4`: reproducibility audit
9. `F1`: manuscript structure
10. `F3`: methods paper draft
11. `H3`: peer-review simulation for harsh stress test

### Narrower route

Use this when you are still locking implementation before building:

1. `A1`
2. `C1`
3. `I5`
4. `I6`

### Typical skills

- `theory-mapper`
- `study-designer`
- `code-specification`
- `code-planning`
- `code-execution`
- `code-review`
- `reproducibility-auditor`
- `stats-engine`
- `manuscript-architect`

### Typical outputs

- method positioning memo
- evaluation design
- code specification
- execution plan
- performance profile
- code review
- reproducibility audit
- methods manuscript draft

### Starter command

```bash
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" \
  --topic policy-effects \
  --domain economics \
  --focus full \
  --paper-type methods \
  --cwd .
```

### Common decision rule

If you are unsure whether the code lane is necessary, ask:

- Is code a core contribution?
- Will reviewers evaluate reproducibility and implementation quality directly?
- Does the paper need strict audit artifacts such as `code_review.md` and `reproducibility_audit.md`?

If yes, use the Stage-I route.

## 5. Theory Paper

### Use this when

The paper’s main contribution is conceptual, theoretical, or mechanism-building rather than data-heavy execution.

### Recommended route

1. `A1`: refine the core question
2. `A1_5`: turn it into propositions
3. `A2`: map the theory base
4. `A4`: identify unresolved theoretical gap
5. `B2`: targeted literature extraction
6. `E1`: synthesize conceptual evidence
7. `F1`: design the manuscript logic
8. `F3`: full theory draft
9. `G4`: tone tightening
10. `H4`: fatal-flaw stress test

### Narrower route

Use this when the theory base is already stable:

1. `A2`
2. `A4`
3. `F1`
4. `F3`

### Typical skills

- `question-refiner`
- `hypothesis-generator`
- `theory-mapper`
- `gap-analyzer`
- `paper-extractor`
- `evidence-synthesizer`
- `manuscript-architect`
- `tone-normalizer`
- `fatal-flaw-detector`

### Typical outputs

- refined conceptual question
- propositions
- theory map
- theoretical gap memo
- theory manuscript draft
- style normalization log
- fatal-flaw memo

### Starter command

```bash
python3 -m bridges.orchestrator task-run \
  --task-id A2 \
  --paper-type theory \
  --topic organizational-ai-governance \
  --cwd .
```

### Common decision rule

Do not over-import the code lane into a theory paper unless the method or simulation is itself part of the contribution.

## Cross-Playbook Advice

### When to go deeper

Go deeper when:

- reviewers will expect reproducibility or checklist evidence
- the paper type itself has strong reporting standards
- the evidence base is contested or heterogeneous
- you need stronger adversarial review

### When to stay narrow

Stay narrow when:

- you already have stable inputs
- the task is a revision rather than a greenfield build
- artifact sprawl is becoming expensive
- you only need one core deliverable

Useful controls:

- `--focus-output`
- `--output-budget`
- `--research-depth deep`
- `--only-target`

## Related Pages

- Need scenario-based task guidance: [Task Recipes](/guide/task-recipes)
- Need the global skill map: [Skills Guide](/reference/skills)
- Need exact command flags: [CLI Reference](/reference/cli)
