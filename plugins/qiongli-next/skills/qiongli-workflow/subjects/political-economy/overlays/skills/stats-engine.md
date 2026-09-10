## Political Economy Overlay

### Activation
Apply this overlay when `stats-engine` is used for political economy designs involving institutions, policy timing, actors, distributional conflict, voting, state capacity, or economic outcomes.

### Required Context
- Require unit of analysis, institution or policy timing, actor exposure, outcome, comparison group, clustering level, and mechanism evidence.
- Require whether the estimate supports descriptive association, causal effect, mechanism evidence, or policy interpretation.
- Require the political assignment process or case-selection logic before specifying inference.

### Subject-Specific Procedure
1. Match statistical design to the political mechanism and institution: panel, DID, IV, RD, event history, survey, text, qualitative comparison, or mixed evidence.
2. Specify whether treatment varies by institution, actor, geography, policy, or time, then set clustering or dependence adjustments accordingly.
3. Check timing, anticipation, spillovers, strategic adaptation, and policy bundling before interpreting effects.
4. Distinguish outcome effects from mechanism tests using mediating evidence, subgroup tests, process evidence, or archival validation.
5. Report how statistical uncertainty changes claim strength for political mechanisms and distributional conflict.

### Reviewer-Risk Checks
- Check whether standard errors ignore clustered institutions, repeated actors, geography, or policy shocks.
- Check whether estimates identify a policy effect but not the political mechanism claimed.
- Check whether distributional winners and losers are inferred without subgroup or mechanism evidence.

### Output Requirements
- Statistical output must include `Assignment/Timing`, `Unit`, `Clustering`, `Mechanism Test`, `Rival Explanation`, and `Claim Strength`.
- Every model must state what political mechanism it can and cannot support.
- Missing timing, clustering, or mechanism evidence must be marked as blocked inference.

### Blocked Conditions
- Block causal political economy claims when assignment, timing, or comparison logic is absent.
- Block mechanism claims when statistical output observes only the policy or economic outcome.
- Do not use statistical precision to hide weak actor or institutional logic.
