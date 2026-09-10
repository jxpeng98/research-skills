## Accounting Overlay

### Activation
Apply this overlay when `variable-constructor` is used for accounting variables, archival proxies, disclosure measures, audit measures, accrual models, governance indicators, or capital-market outcome variables.

### Required Context
- Require the target accounting construct, source database, item code or field name, fiscal period, transformation, missingness rule, and sample filter.
- Require whether the variable is an outcome, treatment, moderator, control, mechanism proxy, or validation measure.
- Require the expected sign and the theoretical channel before finalizing the variable definition.

### Subject-Specific Procedure
1. Distinguish the construct from the empirical proxy and state why the proxy is defensible.
2. Record source database, table, item code, units, fiscal timing, restatement treatment, currency or scale transformation, and winsorization.
3. Document merge keys, Compustat/CRSP or equivalent links, duplicate handling, and observation-loss counts.
4. Identify construct-validity threats such as accrual model error, disclosure boilerplate, audit-office coverage, survivorship, and missing fiscal years.
5. Specify at least one alternative proxy or validation check for central variables.

### Reviewer-Risk Checks
- Check whether the variable construction hides researcher discretion in filters, winsorization, or model residuals.
- Check whether fiscal timing creates reverse causality, look-ahead, or mismatched reporting windows.
- Check whether the proxy captures a competing accounting construct.

### Output Requirements
- The variable artifact must include `Construct`, `Proxy`, `Source Item`, `Transformation`, `Fiscal Timing`, `Missingness`, and `Validity Risk`.
- Sample filters must report observations lost and selection-risk notes.
- Central variables must include a robustness or alternative-measure plan.

### Blocked Conditions
- Block variable finalization when source items, fiscal timing, or transformation rules are missing.
- Block coefficient interpretation when the proxy cannot be mapped to the accounting construct.
- Do not infer disclosure, accrual, or audit behavior from a variable whose measurement window follows the outcome.
