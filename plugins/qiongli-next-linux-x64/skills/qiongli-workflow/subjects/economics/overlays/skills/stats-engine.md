## Quality Bar

- [ ] The estimand, identifying variation, comparison group, treatment timing, and identifying assumption are explicit before estimator choice.
- [ ] Standard errors use the economics-appropriate dependence structure; clustered standard errors should be at the treatment assignment, shock, market, school, region, firm, or panel level when variation requires it.
- [ ] DID designs include pretrend or event-study diagnostics, anticipation and spillover checks, and avoid naive TWFE when staggered adoption with heterogeneous effects is plausible.
- [ ] DID designs with multiple periods state whether the estimator follows cohort, imputation, interaction-weighted, or other heterogeneity-robust logic.
- [ ] IV designs report first-stage strength, reduced form, exclusion rationale, LATE scope, monotonicity concerns, and weak-instrument robust checks where applicable.
- [ ] RD designs report bandwidth choice, manipulation checks, covariate balance, local estimand, and bandwidth or polynomial sensitivity.
- [ ] Regression tables separate baseline, fixed effects, controls, mechanism tests, placebo tests, and robustness specifications.
- [ ] Null results discuss confidence intervals, power, minimum detectable effects, and whether precision can rule out economically meaningful magnitudes.
- [ ] Robustness checks are tied to the main identification threat rather than offered as a generic list.

## Common Pitfalls

| Pitfall | Impact | Fix |
|---------|--------|-----|
| Naive TWFE under staggered adoption | Biased treatment effects under heterogeneous timing or effects | Use cohort/event-time estimators such as Callaway-Sant'Anna, Sun-Abraham, imputation, or justify TWFE limits |
| Wrong clustering level | Over-rejection and misleading precision | Cluster at the treatment assignment or shock level and explain dependence |
| Weak instruments | Biased and unstable IV estimates | Report first-stage diagnostics, reduced form, and weak-IV robust inference |
| Bad controls | Post-treatment adjustment bias | Use only pre-treatment controls for causal specifications |
| Unstated estimand | Readers cannot tell what parameter is identified | State ATE, ATT, LATE, ITT, elasticity, structural parameter, or descriptive association |
| Selective robustness | Specification search hidden as validation | Predeclare primary specification and show bounded sensitivity |
| Mechanism from heterogeneity alone | Subgroup effects are overread as channels | Add mechanism evidence or narrow the claim |
| Strong null claim from imprecision | Noisy estimates are misread as no effect | Report confidence intervals and minimum detectable effects |
