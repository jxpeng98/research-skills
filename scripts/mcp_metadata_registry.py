#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from bridges.providers.metadata_registry import (
    METADATA_ENRICH_ENV,
    apply_external_enrichment,
    artifact_project_root,
    collect_reference_records,
    dedupe_identifiers,
    extract_context_hints,
    extract_identifiers,
    merge_reference_records,
    read_source_texts,
    summarize_reference_state,
)


def main() -> None:
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            print(json.dumps({"status": "error", "summary": "No input provided", "data": {}}))
            return

        payload = json.loads(raw)
        task_packet = payload.get("task_packet", {})
        if not isinstance(task_packet, dict):
            task_packet = {}
        cwd = Path.cwd()
        project_root = artifact_project_root(task_packet, cwd)
        required_outputs = [
            str(item) for item in task_packet.get("required_outputs", []) if str(item).strip()
        ]

        source_texts: list[tuple[str, str]] = [("task_packet", json.dumps(task_packet, ensure_ascii=False))]
        source_texts.extend(read_source_texts(project_root, required_outputs))

        records: list[dict[str, str]] = []
        provenance: list[str] = []
        for source_name, text in source_texts:
            if source_name != "task_packet":
                provenance.append(source_name)
            records.extend(extract_identifiers(source_name, text))

        deduped = dedupe_identifiers(records)
        context_hints = extract_context_hints(task_packet)
        reference_records = collect_reference_records(project_root, required_outputs)
        merged_records, dedup_log = merge_reference_records(reference_records)
        reference_state = summarize_reference_state(project_root, merged_records)
        enriched_records, enrichment_info = apply_external_enrichment(
            merged_records,
            cwd=project_root,
            context_hints=context_hints,
        )
        if enrichment_info.get("configured"):
            merged_records = enriched_records
            reference_state["external_enrichment"] = {
                "env_name": METADATA_ENRICH_ENV,
                **enrichment_info,
            }
        else:
            reference_state["external_enrichment"] = {
                "configured": False,
                "env_name": METADATA_ENRICH_ENV,
            }

        if deduped or merged_records:
            summary = (
                f"Builtin metadata registry normalized {len(deduped)} identifiers and merged "
                f"{len(merged_records)} reference records from local artifacts."
            )
            if reference_state["external_enrichment"].get("configured"):
                summary += " External enrichment overlay applied."
            status = "ok"
        else:
            summary = (
                "Builtin metadata registry is available, but no usable identifiers or local "
                "reference records were found in the task packet or local artifacts."
            )
            status = "warning"

        enrichment_status = reference_state["external_enrichment"].get("status")
        if enrichment_status == "error":
            status = "warning"
        elif enrichment_status == "warning" and status == "ok":
            status = "warning"

        print(
            json.dumps(
                {
                    "status": status,
                    "summary": summary,
                    "provenance": provenance[:6]
                    + reference_state["external_enrichment"].get("provenance", [])[:2],
                    "data": {
                        "provider_mode": "builtin_local_reference_registry",
                        "project_root": str(project_root),
                        "scanned_sources": [name for name, _ in source_texts],
                        "identifier_count": len(deduped),
                        "identifiers": deduped[:50],
                        "record_count": len(merged_records),
                        "records": merged_records[:100],
                        "reference_state": reference_state,
                        "dedup_log": dedup_log[:100],
                        "context_hints": context_hints,
                    },
                },
                ensure_ascii=False,
            )
        )
    except Exception as exc:
        print(
            json.dumps(
                {
                    "status": "error",
                    "summary": f"Metadata registry provider exception: {exc}",
                    "data": {"error": str(exc)},
                },
                ensure_ascii=False,
            )
        )


if __name__ == "__main__":
    main()
