## Economics Overlay

### Activation
Apply this overlay when `robustness-planner` is used for economics designs or results that depend on identification, estimand stability, estimator choice, or causal interpretation.

### Required Context
- Require the primary estimand, baseline specification, identifying variation, comparison group, treatment timing, and current claim strength.
- Require the main validity threat before listing robustness checks.
- Require the estimator family, standard-error plan, and planned tables or figures so robustness checks can be tied to reported evidence.

### Subject-Specific Procedure
1. Start with the main identification threat, then select diagnostics that directly test or bound that threat.
2. Separate diagnostic checks, falsification tests, placebo tests, sensitivity analysis, and alternative estimators.
3. For DID, include pretrend or event-study diagnostics, anticipation checks, spillover checks, alternative treatment timing, and modern staggered-adoption estimators when heterogeneous effects are plausible.
4. For IV, include first-stage strength, reduced form, overidentification logic when available, weak-IV robust inference, and exclusion-restriction threats.
5. For RD, include bandwidth sensitivity, manipulation checks, covariate balance, donut tests, and polynomial-order discipline.
6. State how each robustness result would alter the estimand, interpretation, or causal claim.

### Reviewer-Risk Checks
- Check whether robustness is a generic checklist rather than a targeted response to the core identification threat.
- Check whether the plan hides specification search, selectively reports successful alternatives, or changes samples without explaining estimand drift.
- Check whether the standard errors and clustering level remain valid across robustness specifications.

### Output Requirements
- The robustness plan must group checks by `Threat`, `Diagnostic`, `Expected Evidence`, and `Interpretation If Failed`.
- Each robustness check must say whether it tests identification, measurement, functional form, sample construction, inference, or external validity.
- Missing data or diagnostics must be recorded as blocked robustness checks rather than replaced with unsupported reassurance.

### Blocked Conditions
- Block a pass verdict when the main validity threat has no targeted diagnostic, falsification test, or sensitivity analysis.
- Block causal upgrades when robustness checks only vary controls or samples without addressing identifying variation.
- Do not treat statistical significance across many specifications as evidence of identification.
