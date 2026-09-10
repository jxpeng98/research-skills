## Finance Overlay

### Activation
Apply this overlay when `manuscript-architect` is used for finance manuscripts in asset pricing, corporate finance, market microstructure, risk, event-study, theory, or methods work.

### Required Context
- Require the finance claim type, target venue family, asset universe or firm sample, return or accounting data source, event timing, benchmark model, and identification strategy.
- Require whether the central claim is alpha, risk compensation, mispricing, liquidity, corporate policy, market response, or causal mechanism.
- Require current evidence on return construction, factor exposure, bias controls, and inference.

### Subject-Specific Procedure
1. Architect the introduction around the finance contribution and the evidence that distinguishes risk, mispricing, policy, liquidity, or causal mechanism.
2. Connect each central claim to return construction, benchmark choice, factor exposure, event window, identification, and robustness evidence.
3. For asset pricing claims, state the factor model, portfolio construction, rebalancing, delisting treatment, and risk-adjusted benchmark.
4. For event studies, state event dates, estimation window, event window, confounding-event rules, abnormal-return model, and leakage checks.
5. For corporate finance claims, separate association, risk-adjusted evidence, and causal identification.

### Reviewer-Risk Checks
- Check whether the manuscript mistakes abnormal returns for alpha without a defensible benchmark.
- Check whether look-ahead bias, survivorship, stale prices, event-date leakage, or overlapping observations could drive results.
- Check whether causal language outruns the corporate finance design.

### Output Requirements
- The manuscript architecture must include `Claim Type`, `Return/Risk Construction`, `Benchmark`, `Bias Controls`, and `Inference Plan`.
- Every central result must name the benchmark, risk adjustment, timing logic, and robustness evidence.
- Missing event-window, factor-model, or bias-control evidence must become revision tasks.

### Blocked Conditions
- Block alpha, mispricing, or risk-adjusted claims when benchmark and factor exposure are unspecified.
- Block event-study claims when event dates, windows, or leakage controls are absent.
- Do not treat undergraduate-and-above work as journal-ready without doctoral-level finance evidence standards.
