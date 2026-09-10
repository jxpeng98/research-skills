#!/usr/bin/env Rscript

# Random/fixed-effects meta-analysis template using metafor.
#
# Input CSV (minimum columns):
#   - outcome_id
#   - study_id
#   - yi   (effect size on the analysis scale, e.g., log(OR), SMD, Fisher's z)
#   - sei  (standard error of yi)
#
# Usage:
#   Rscript meta_analysis_metafor.R effect_sizes.csv results_dir
#
# Notes:
#   - Prefer REML for tau^2 by default; switch to DL for classic DerSimonian–Laird.
#   - Convert back from log scales after pooling (e.g., OR = exp(yi)).

args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 1) {
  stop("Usage: Rscript meta_analysis_metafor.R effect_sizes.csv [results_dir]")
}

csv_path <- args[[1]]
out_dir <- ifelse(length(args) >= 2, args[[2]], "meta_analysis_outputs")

if (!dir.exists(out_dir)) dir.create(out_dir, recursive = TRUE, showWarnings = FALSE)

suppressPackageStartupMessages(library(metafor))

df <- read.csv(csv_path, stringsAsFactors = FALSE)
required <- c("outcome_id", "study_id", "yi", "sei")
missing <- setdiff(required, names(df))
if (length(missing) > 0) stop(paste("Missing columns:", paste(missing, collapse = ", ")))

df$outcome_id <- as.character(df$outcome_id)
df$study_id <- as.character(df$study_id)

outcomes <- sort(unique(df$outcome_id))
results <- data.frame(
  outcome_id = character(),
  k = integer(),
  pooled_yi = numeric(),
  ci_low = numeric(),
  ci_high = numeric(),
  tau2 = numeric(),
  i2 = numeric(),
  stringsAsFactors = FALSE
)

for (outcome in outcomes) {
  d <- df[df$outcome_id == outcome, ]
  if (nrow(d) < 2) {
    message(sprintf("Skipping outcome %s (k < 2)", outcome))
    next
  }

  res <- rma(yi = yi, sei = sei, data = d, method = "REML")

  message("\nOutcome: ", outcome)
  print(res)

  png(filename = file.path(out_dir, paste0("forest_", outcome, ".png")), width = 1200, height = 800)
  forest(res, slab = d$study_id)
  dev.off()

  png(filename = file.path(out_dir, paste0("funnel_", outcome, ".png")), width = 1000, height = 800)
  funnel(res)
  dev.off()

  # Egger-style test (interpret cautiously; typically needs ~10+ studies)
  egger <- tryCatch(regtest(res, model = "rma"), error = function(e) NULL)
  if (!is.null(egger)) {
    capture.output(egger, file = file.path(out_dir, paste0("egger_", outcome, ".txt")))
  }

  loo <- tryCatch(leave1out(res), error = function(e) NULL)
  if (!is.null(loo)) {
    write.csv(loo, file = file.path(out_dir, paste0("leave1out_", outcome, ".csv")), row.names = FALSE)
  }

  results <- rbind(
    results,
    data.frame(
      outcome_id = outcome,
      k = nrow(d),
      pooled_yi = as.numeric(res$b),
      ci_low = res$ci.lb,
      ci_high = res$ci.ub,
      tau2 = res$tau2,
      i2 = res$I2,
      stringsAsFactors = FALSE
    )
  )
}

write.csv(results, file = file.path(out_dir, "meta_analysis_summary.csv"), row.names = FALSE)
message("\nWrote: ", file.path(out_dir, "meta_analysis_summary.csv"))

