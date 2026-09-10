## Economics Overlay

### Activation
Apply this overlay when `study-designer` is used for an economics subject package, especially for empirical designs that may become causal, structural, panel, policy, DID, IV, RD, synthetic-control, or matching claims.

### Required Context
- Require the research question, paper type, outcome, treatment or shock, unit of observation, time period, comparison group, and target venue family.
- Require the estimand before the estimating equation, and state whether the paper seeks descriptive, causal reduced-form, structural, predictive, or theory-only evidence.
- Require the assignment mechanism, source of identifying variation, treatment timing, and data structure before recommending an estimator.

### Subject-Specific Procedure
1. Name the estimand, identifying variation, comparison group, and identifying assumption in separate sentences.
2. Match the design to the economics method family: DID, IV, RD, event study, panel FE, synthetic control, matching, structural, or descriptive.
3. For DID or event-study designs, specify treatment timing, anticipation, spillover, pretrend, staggered-adoption, and heterogeneous-effect risks before model choice.
4. For IV designs, state relevance, exclusion, monotonicity or LATE scope, first-stage diagnostics, and weak-instrument risk.
5. For RD designs, state running variable, cutoff, bandwidth logic, manipulation checks, and local estimand.
6. Record the main identification threat and the design feature or blocked diagnostic that addresses it.

### Reviewer-Risk Checks
- Check whether the design confuses an association, a policy comparison, and a causal claim.
- Check whether controls are post-treatment, whether fixed effects absorb the identifying variation, and whether standard errors match the assignment or shock level.
- Check whether the strongest alternative explanation would still survive the proposed robustness plan.

### Output Requirements
- The study design artifact must include `Estimand`, `Identifying Variation`, `Comparison Group`, `Assumptions`, `Diagnostics`, and `Blocked Checks`.
- The recommended model must be justified by treatment timing and data structure, not by convention alone.
- Missing pretrends, first-stage evidence, bandwidth diagnostics, or clustering logic must be written as gaps.

### Blocked Conditions
- Block causal language when identifying variation, comparison group, or assignment timing is missing.
- Block estimator recommendations when the data structure needed for DID, IV, RD, or panel inference is not specified.
- Do not upgrade descriptive economics patterns into causal interpretation without a named identifying assumption and diagnostic plan.
