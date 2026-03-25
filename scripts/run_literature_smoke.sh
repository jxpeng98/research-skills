#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "[literature-smoke] builtin literature stack"
python3 -m unittest tests.test_literature_pipeline_integration -v

echo "[literature-smoke] passed"
