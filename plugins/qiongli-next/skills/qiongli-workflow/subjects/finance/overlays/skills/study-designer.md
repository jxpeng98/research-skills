## Finance Overlay

### Activation
Apply this overlay when `study-designer` is used for finance research involving returns, firms, portfolios, events, liquidity, risk, corporate policy, or financial markets.

### Required Context
- Require the finance claim type, asset universe or firm sample, data frequency, return construction, benchmark model, event timing, comparison group, and target venue family.
- Require whether the study is asset pricing, corporate finance, market microstructure, risk management, event study, theory, or methods.
- Require the planned standard-error or test structure before selecting estimators.

### Subject-Specific Procedure
1. Classify the claim as descriptive, asset pricing, risk-adjusted performance, event study, corporate finance, market microstructure, or causal.
2. Specify return construction, delisting treatment, rebalancing, portfolio sorts, factor model, and benchmark before interpreting performance.
3. For event studies, specify event definition, estimation window, event window, benchmark model, confounding events, and announcement leakage risk.
4. Audit look-ahead bias, survivorship bias, stale prices, overlapping observations, and sample-selection channels.
5. Match inference to the design: Fama-MacBeth, clustered panel errors, Newey-West, bootstrap, or event-time tests.

### Reviewer-Risk Checks
- Check whether the design can separate risk compensation from mispricing or corporate policy effects.
- Check whether timing or data availability would allow future information into signals, portfolios, or event classification.
- Check whether the comparison group and benchmark match the claim.

### Output Requirements
- The study design artifact must include `Claim Classification`, `Data Timing`, `Return Construction`, `Benchmark`, `Bias Controls`, and `Inference`.
- Asset pricing and event-study designs must state factor models and event windows explicitly.
- Missing delisting, survivorship, benchmark, or look-ahead checks must be recorded as blocked diagnostics.

### Blocked Conditions
- Block risk-adjusted claims when benchmark, factor set, or return construction is missing.
- Block causal corporate finance claims when identification is only cross-sectional association.
- Do not recommend a model until data timing and leakage risks are clear.
