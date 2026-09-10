from contextlib import redirect_stdout
import csv
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest

import yaml

from evals.runner import run_eval


ROOT = Path(__file__).resolve().parents[1] / "evals/research_journey"
FIXTURE = ROOT / "fixtures/reading-to-manuscript"


def edit_rows(path, change):
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        fields, rows = reader.fieldnames, list(reader)
    change(rows)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


class ResearchJourneyEvalTests(unittest.TestCase):
    def evaluate(self, root, fixture, name="reading-to-manuscript", case=None):
        receipt = root / "receipt.json"
        with redirect_stdout(io.StringIO()):
            code = run_eval.main([str(case or ROOT / "cases" / f"{name}.yaml"),
                                  str(fixture), "--json-receipt", str(receipt)])
        result = json.loads(receipt.read_text())
        self.assertEqual(code == 0, result["case"]["status"] == "pass")
        return result

    def test_scoped_journeys_allow_reuse_paraphrase_and_unrelated_pending_claims(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture = root / "output"
            shutil.copytree(FIXTURE, fixture)
            result = self.evaluate(root, fixture)
            self.assertEqual("pass", result["case"]["status"])
            self.assertEqual(16, result["summary"]["executed_assertions"])
            # Output prose and row order are free; pending C3 remains unused in the ledger.
            edit_rows(fixture / "reading.csv", lambda rows: rows.reverse())
            edit_rows(fixture / "manuscript.csv", lambda rows: rows[0].update(
                passage="The supplied abstract reports a positive association (r = .32)."))
            self.assertEqual("pass", self.evaluate(root, fixture)["case"]["status"])
            # A narrower, predeclared task uses source evidence directly; no new B2 artifacts.
            (fixture / "reading.csv").unlink()
            direct = self.evaluate(root, fixture, "source-to-paragraph")
            self.assertEqual("pass", direct["case"]["status"])
            self.assertEqual(13, direct["summary"]["executed_assertions"])
            self.assertNotIn("reading", {a["output_id"] for a in direct["assertions"]})
            # The same omission cannot silently reduce the explicit reading task.
            full = self.evaluate(root, fixture)
            self.assertEqual(1, full["summary"]["required_missing"])

    def test_missing_stale_and_mislinked_evidence_never_passes(self):
        mutations = [
            ("missing-source", "required-artifact-missing", lambda f: (f / "source.md").unlink()),
            ("changed-source", "file-digest-failed", lambda f: (f / "source.md").write_text("Changed source")),
            ("changed-registry", "file-digest-failed", lambda f: (f / "sources.csv").write_text("Invented registry")),
            ("unknown-source", "citation-identity-failed", lambda f: edit_rows(f / "ledger.csv", lambda r: r[0].update(source_id="Missing2026"))),
            ("untracked-claim", "cross-artifact-consistency-failed", lambda f: edit_rows(f / "manuscript.csv", lambda r: r.append(dict(r[0], claim_id="C999")))),
            ("missing-requested-claim", "cross-artifact-consistency-failed", lambda f: edit_rows(f / "manuscript.csv", lambda r: r.pop())),
            ("duplicate-claim", "cross-artifact-consistency-failed", lambda f: edit_rows(f / "manuscript.csv", lambda r: r.append(dict(r[0])))),
            ("missing-active-source", "cross-artifact-consistency-failed", lambda f: edit_rows(f / "ledger.csv", lambda r: r[0].update(source_location=""))),
            ("blank-active-link", "assertion-evidence-unavailable", lambda f: edit_rows(f / "manuscript.csv", lambda r: r[0].update(source_location=""))),
            ("unavailable-artifact", "cross-artifact-consistency-failed", lambda f: edit_rows(f / "manuscript.csv", lambda r: r[0].update(artifact_path="missing.md"))),
            ("invented-full-text", "cross-artifact-consistency-failed", lambda f: edit_rows(f / "manuscript.csv", lambda r: r[0].update(evidence_limit="full_text"))),
            ("pending-as-supported", "field-constraint-failed", lambda f: edit_rows(f / "manuscript.csv", lambda r: r[0].update(status="needs_evidence"))),
            ("unbound-reading", "cross-artifact-consistency-failed", lambda f: edit_rows(f / "reading.csv", lambda r: r[0].update(source_location="SyntheticStudy2026:invented"))),
            ("empty-passage", "schema-failed", lambda f: edit_rows(f / "manuscript.csv", lambda r: r[0].update(passage=" "))),
            ("empty-note", "schema-failed", lambda f: edit_rows(f / "reading.csv", lambda r: r[0].update(reading_note=""))),
            ("malformed-csv", "assertion-evidence-unavailable", lambda f: (f / "reading.csv").write_text("header\na,b\n")),
            ("missing-column", "assertion-evidence-unavailable", lambda f: (f / "manuscript.csv").write_text((f / "manuscript.csv").read_text().replace("source_location", "missing_column", 1))),
        ]
        for name, reason, mutate in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                fixture = root / "output"
                shutil.copytree(FIXTURE, fixture)
                mutate(fixture)
                result = self.evaluate(root, fixture)
                self.assertNotEqual("pass", result["case"]["status"])
                self.assertIn(reason, {a["reason_code"] for a in result["assertions"]})

    def test_tuple_links_preserve_pairings_and_reject_invalid_contracts(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture = root / "output"
            shutil.copytree(FIXTURE, fixture)
            # Both columns' multisets still match after a swap; the paired link must fail.
            def swap(rows):
                rows[0]["source_location"], rows[1]["source_location"] = rows[1]["source_location"], rows[0]["source_location"]
            edit_rows(fixture / "manuscript.csv", swap)
            result = self.evaluate(root, fixture)
            failures = [a for a in result["assertions"] if a["status"] != "pass"]
            self.assertEqual([("manuscript", "cross-artifact-consistency-failed")],
                             [(a["output_id"], a["reason_code"]) for a in failures])
            # Exact equality and legacy scalar subset still reject blank comparison values.
            for field, relation in ((["claim_id", "source_id"], "equal"), ("source_id", "subset")):
                case = {"schema_version": "1.0", "case_id": "tuple-boundaries", "pipeline": "research-journey",
                        "input": {"topic": "strict legacy and equality"},
                        "expected_outputs": {"links": {"artifact": "manuscript.csv", "required": True,
                        "assertions": [{"type": "cross_artifact_consistency", "field": field,
                            "other_artifact": "ledger.csv", "other_field": field, "relation": relation}]}}}
                path = root / "case.yaml"
                path.write_text(yaml.safe_dump(case))
                result = self.evaluate(root, fixture, case=path)
                self.assertEqual("blocked", result["case"]["status"])
            assertion = case["expected_outputs"]["links"]["assertions"][0]
            for field, other in (([], ["claim_id"]), (["claim_id", "claim_id"], ["claim_id", "source_id"]),
                                 (["claim_id", "source_id"], "claim_id"), ([None], ["claim_id"])):
                assertion.update(field=field, other_field=other)
                path.write_text(yaml.safe_dump(case))
                result = self.evaluate(root, fixture, case=path)
                self.assertEqual("blocked", result["case"]["status"])
                self.assertEqual("assertion-config-invalid", result["assertions"][0]["reason_code"])


if __name__ == "__main__":
    unittest.main()
