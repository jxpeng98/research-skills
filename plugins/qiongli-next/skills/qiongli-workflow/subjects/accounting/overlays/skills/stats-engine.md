## Quality Bar

- [ ] Model choices match the accounting construct, empirical proxy, sample structure, and identifying variation.
- [ ] Accrual, disclosure, audit, governance, tax, and capital-market measures document source items, transformations, fiscal timing, scaling, and missingness.
- [ ] Standard errors reflect firm, year, industry, auditor, office, state, market, or event-level dependence when the design requires it.
- [ ] Fixed effects are justified by the reporting setting and do not absorb the relevant variation without explanation.
- [ ] Winsorization, trimming, restatement handling, and sample screens are reported with observation-loss counts and selection-risk notes.
- [ ] Accrual models and residual-based proxies include distribution checks, influential-observation diagnostics, and alternative proxy robustness.
- [ ] Disclosure or market-reaction designs define event windows, confounding disclosures, fiscal/calendar alignment, and information-availability timing.
- [ ] Construct-validity checks are separated from identification checks before interpreting coefficients.
- [ ] Central tables include at least one measurement robustness check and one sample-construction sensitivity check.

## Common Pitfalls

| Pitfall | Impact | Fix |
|---------|--------|-----|
| Treating a noisy accounting proxy as the latent construct | Coefficients are overinterpreted as reporting behavior | Add construct-proxy mapping and alternative measures |
| Hidden fiscal timing mismatch | Reverse timing or look-ahead undermines inference | Align fiscal years, announcement dates, and outcome windows explicitly |
| Undocumented winsorization | Researcher discretion changes effect size | Report thresholds and sensitivity to scaling or trimming |
| Missing Compustat/CRSP or database link rules | Sample cannot be replicated | State merge keys, coverage, duplicate handling, and dropped observations |
| Unjustified fixed effects | Variation is absorbed or interpretation changes | Explain what each fixed effect controls and what variation remains |
| Wrong clustering level | Precision is misleading in firm-year panels | Cluster by the assignment/reporting channel and test robustness |
| Event-window confounding | Disclosure or market reaction is mixed with other news | Define windows and exclude or flag confounding disclosures |
| Measurement robustness omitted | Reviewers doubt construct validity | Add alternate proxies, validation checks, or sensitivity tables |
