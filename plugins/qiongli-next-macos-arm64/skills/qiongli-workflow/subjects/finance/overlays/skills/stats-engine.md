## Quality Bar

- [ ] Asset pricing claims define test assets, factor model, return frequency, sample window, portfolio formation date, rebalancing rule, and risk-adjusted benchmark; the report must explicitly label the asset pricing evidence standard being used.
- [ ] Return construction handles delisting returns, dividends, splits, stale prices, survivorship bias, microstructure filters, and look-ahead bias.
- [ ] Event studies define event dates, estimation windows, event windows, expected-return model, confounding-event filters, CAR/BHAR tests, and announcement leakage checks.
- [ ] Corporate finance panels document fixed effects, clustering, timing, treatment definition, comparison group, and identification threats.
- [ ] Inference matches the data structure: Fama-MacBeth, Newey-West, two-way clustering, portfolio-sort tests, bootstrap, or event-time corrections.
- [ ] Factor exposure, benchmark sensitivity, and alternative risk models are reported for abnormal return, alpha, or risk-adjusted claims.
- [ ] Overlapping observations, repeated events, cross-sectional dependence, and calendar-time clustering are handled explicitly.
- [ ] Causal corporate policy claims separate identification evidence from risk-adjusted association.
- [ ] Every finance result states whether it supports alpha, risk compensation, mispricing, liquidity, corporate policy, or descriptive market behavior.

## Common Pitfalls

| Pitfall | Impact | Fix |
|---------|--------|-----|
| Look-ahead bias | Creates false predictability or overstated alpha | Construct signals using only information available at portfolio formation |
| Survivorship bias | Overstates returns and weakens external validity | Include delisted, failed, or missing firms when the research question requires them |
| Weak benchmark model | Mislabels risk compensation as abnormal performance | Test alternative factor models and report risk-adjusted sensitivity |
| Overlapping returns | Understates standard errors | Use Newey-West, block bootstrap, or design-appropriate corrections |
| Event-date leakage | Confounds CAR interpretation | Audit announcement timing, anticipation, and nearby events |
| Stale prices or microstructure noise | Creates spurious short-window returns | Add liquidity screens, trade timing checks, or lower-frequency robustness |
| Fama-MacBeth used mechanically | Cross-sectional inference may not match design | State pricing test, risk premia logic, and time-series correction |
| Causal overclaiming in corporate finance | Reviewer rejects interpretation despite strong association | Add identification evidence or narrow to association/risk-adjusted result |
