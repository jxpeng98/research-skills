## Economics Overlay

### Activation
Apply this overlay when `analysis-interpreter` is used for economics results, especially regression tables, event studies, treatment effects, structural estimates, or policy-relevant empirical findings.

### Required Context
- Require the estimand, coefficient scale, outcome unit, baseline mean, comparison group, standard-error plan, and identification strategy.
- Require the current claim type: descriptive, causal reduced-form, structural, predictive, or theory implication.
- Require the main table or figure mapping before interpreting signs, magnitudes, or null results.

### Subject-Specific Procedure
1. Interpret each central coefficient as an estimand with units, not only a sign, p-value, or star level.
2. Convert magnitudes into economically meaningful scales: baseline mean shares, percentage changes, policy costs, welfare-relevant units, or implied elasticities when defensible.
3. Tie precision to inference by reporting confidence intervals, standard errors, clustering level, and minimum detectable effect for null or noisy estimates when possible.
4. Link each causal interpretation back to identifying variation, comparison group, and robustness evidence.
5. Separate mechanism evidence from treatment-effect evidence and avoid using heterogeneous effects as mechanisms without a design.

### Reviewer-Risk Checks
- Check whether the interpretation overstates causality relative to the design.
- Check whether economic magnitude is absent, unitless, or detached from the outcome scale.
- Check whether null results are interpreted as no effect without power, precision, or confidence-interval discussion.

### Output Requirements
- The interpretation artifact must include `Estimand`, `Magnitude`, `Inference`, `Identification Support`, and `Claim Calibration`.
- For every headline result, state what evidence supports the claim and what evidence only remains suggestive.
- Record missing magnitude, standard-error, or robustness evidence as a gap note.

### Blocked Conditions
- Block policy or welfare claims when the design only supports descriptive association.
- Block strong null claims when precision, power, or confidence intervals are not available.
- Do not interpret statistical significance as substantive importance without economic magnitude.
