## Quality Bar

- [ ] The causal estimand, identifying variation, accounting construct, empirical proxy, and disclosure or reporting institution are stated before estimator choice.
- [ ] Panel, event-study, DID, IV, or RD models align treatment timing with fiscal-year reporting windows and capital-market outcome windows.
- [ ] Standard errors are justified for firm, industry, auditor, market, state, mandate, or event-level dependence; cluster choice matches assignment and reporting channels.
- [ ] Archival accounting designs report sample filters, winsorization, missingness handling, source items, fiscal timing, and alternative proxy checks; the output must explicitly describe archival accounting measurement risk.
- [ ] Robustness includes at least one construct-validity check and one identification-threat check when causal language is used.
- [ ] Disclosure event studies define announcement dates, event windows, confounding news, information availability, and abnormal return benchmarks.
- [ ] Capital-market outcomes distinguish price reaction, liquidity, cost of capital, analyst behavior, and financing channels.
- [ ] Matched samples, policy comparisons, and staggered mandates state comparison group, timing, pretrend, and selection-risk diagnostics.
- [ ] Result interpretation says which standard is being satisfied: economics identification, accounting measurement, or both.

## Common Pitfalls

| Pitfall | Impact | Fix |
|---------|--------|-----|
| Accounting proxy treated as construct | Measurement risk is hidden behind causal language | Add construct-proxy map and alternative measures |
| Naive TWFE under staggered disclosure mandates | Treatment effects are biased under heterogeneous timing | Add cohort/event-time diagnostics or heterogeneity-robust estimator |
| Fiscal/calendar mismatch | Treatment, reporting, and market response windows do not align | Document fiscal-year and event-date alignment rules |
| Wrong clustering level | Precision ignores mandate, auditor, industry, or market dependence | Cluster at the assignment/reporting channel and test sensitivity |
| Capital-market mechanism assumed | Disclosure effects are interpreted without outcome-channel evidence | Separate price, liquidity, analyst, financing, and governance channels |
| Sample filters undocumented | Archival results are hard to replicate and may be selected | Report observation loss and selection-risk checks |
| Measurement robustness omitted | Accounting reviewers doubt construct validity | Add alternate proxies, source-item checks, or model residual sensitivity |
| Identification robustness omitted | Economics reviewers doubt causal interpretation | Add pretrends, placebo timing, falsification outcomes, or sensitivity analysis |
