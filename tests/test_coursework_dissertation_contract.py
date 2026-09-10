from __future__ import annotations

import unittest
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "content" / "standards" / "research-workflow-contract.yaml"
WORKFLOW_REFERENCE = ROOT / "content" / "workflow" / "references" / "workflow-contract.md"


def load_contract() -> dict:
    return yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))


def test_contract_declares_coursework_and_dissertation_stages() -> None:
    contract = load_contract()
    stages = {stage["id"]: stage for stage in contract["stages"]}

    assert stages["L"]["name"] == "coursework-learning-assessment"
    assert stages["L"]["phase_type"] == "project-mode"
    assert "assignment/brief.md" in stages["L"]["outputs"]
    assert "coursework/claim_evidence_plan.md" in stages["L"]["outputs"]

    assert stages["M"]["name"] == "dissertation-major-project"
    assert stages["M"]["phase_type"] == "project-mode"
    assert "dissertation/dissertation_plan.md" in stages["M"]["outputs"]
    assert "dissertation/defense_prep.md" in stages["M"]["outputs"]


def test_contract_declares_coursework_and_dissertation_task_ids() -> None:
    task_catalog = load_contract()["task_catalog"]

    expected_l = {
        "L1": "assignment-brief-intake",
        "L2": "rubric-learning-outcome-map",
        "L3": "coursework-outline",
        "L4": "coursework-claim-evidence-plan",
        "L5": "coursework-draft",
        "L6": "coursework-revision",
        "L7": "coursework-final-readiness",
    }
    expected_m = {
        "M1": "dissertation-planning",
        "M2": "dissertation-chapter-architecture",
        "M3": "dissertation-chapter-drafting",
        "M4": "supervisor-feedback-integration",
        "M5": "dissertation-milestone-risk-plan",
        "M6": "dissertation-final-readiness",
        "M7": "viva-defense-preparation",
    }

    for task_id, title in expected_l.items():
        assert task_catalog[task_id]["stage"] == "L"
        assert task_catalog[task_id]["title"] == title
        assert task_catalog[task_id]["outputs"]

    for task_id, title in expected_m.items():
        assert task_catalog[task_id]["stage"] == "M"
        assert task_catalog[task_id]["title"] == title
        assert task_catalog[task_id]["outputs"]


def test_contract_declares_project_types_and_context_refresh_points() -> None:
    contract = load_contract()

    assert "coursework" in contract["academic_project_types"]
    assert "dissertation" in contract["academic_project_types"]
    assert "thesis" in contract["academic_project_types"]

    refresh_points = contract["academic_context_continuity"]["refresh_points"]
    assert "rubric criteria" in refresh_points["L"]
    assert "chapter status" in refresh_points["M"]


def test_generated_workflow_reference_mentions_new_stages_and_tasks() -> None:
    text = WORKFLOW_REFERENCE.read_text(encoding="utf-8")

    assert "- `L`: Coursework and learning assessment" in text
    assert "- `M`: Dissertation and major project" in text
    assert "| `L1` | L | Assignment brief intake | `assignment/brief.md` |" in text
    assert "| `M7` | M | Viva or defense preparation | `dissertation/defense_prep.md` |" in text


def test_coursework_and_dissertation_workflow_docs_exist() -> None:
    coursework = (ROOT / "content" / "workflow" / "workflows" / "coursework.md").read_text(
        encoding="utf-8"
    )
    dissertation = (ROOT / "content" / "workflow" / "workflows" / "dissertation.md").read_text(
        encoding="utf-8"
    )

    assert "Canonical Task IDs" in coursework
    assert "`L1` assignment brief intake" in coursework
    assert "academic integrity" in coursework.lower()
    assert "`M1` dissertation project planning" in dissertation
    assert "supervisor feedback" in dissertation.lower()


def test_platform_routing_mentions_coursework_and_dissertation() -> None:
    routing = (ROOT / "content" / "workflow" / "references" / "platform-routing.md").read_text(
        encoding="utf-8"
    )
    skill = (ROOT / "content" / "workflow" / "SKILL.md").read_text(encoding="utf-8")

    assert "/coursework" in routing
    assert "/dissertation" in routing
    assert "assignment brief" in routing
    assert "supervisor feedback" in routing
    assert "workflows/coursework.md" in skill
    assert "workflows/dissertation.md" in skill


def test_skill_schema_and_registry_include_coursework_and_dissertation_stages() -> None:
    schema = yaml.safe_load(
        (ROOT / "content" / "schemas" / "skill.schema.json").read_text(encoding="utf-8")
    )
    allowed_stages = set(schema["properties"]["stage"]["enum"])
    assert "L_coursework" in allowed_stages
    assert "M_dissertation" in allowed_stages

    registry = yaml.safe_load((ROOT / "content" / "skills" / "registry.yaml").read_text(encoding="utf-8"))
    entries = {entry["id"]: entry for entry in registry["skills"]}
    for skill_id in {
        "assignment-brief-analyzer",
        "rubric-mapper",
        "coursework-architect",
        "coursework-reviser",
        "dissertation-planner",
        "chapter-architect",
        "supervisor-feedback-integrator",
        "dissertation-readiness-checker",
    }:
        assert skill_id in entries
        skill_path = ROOT / "content" / entries[skill_id]["file"]
        assert skill_path.is_file()
        assert skill_path.read_text(encoding="utf-8").startswith("---\n")


def test_skill_docs_generator_knows_new_stages() -> None:
    skill_docs = (
        ROOT / "packages" / "python-qiongli" / "src" / "qiongli" / "skill_docs.py"
    ).read_text(encoding="utf-8")

    assert '"L_coursework"' in skill_docs
    assert '"M_dissertation"' in skill_docs
    assert "Coursework" in skill_docs
    assert "Dissertation" in skill_docs


def test_coursework_and_dissertation_templates_exist_with_missing_information_fields() -> None:
    template_names = [
        "assignment-brief.md",
        "rubric-map.md",
        "learning-outcomes.md",
        "academic-integrity-notes.md",
        "coursework-outline.md",
        "coursework-claim-evidence-plan.md",
        "coursework-revision-plan.md",
        "coursework-submission-checklist.md",
        "dissertation-plan.md",
        "dissertation-chapter-map.md",
        "dissertation-chapter-status.md",
        "supervisor-feedback-log.md",
        "dissertation-milestone-plan.md",
        "dissertation-final-readiness.md",
        "dissertation-defense-prep.md",
    ]

    for name in template_names:
        text = (ROOT / "content" / "templates" / name).read_text(encoding="utf-8")
        assert "Missing Information" in text
        assert "Do not invent" in text


def load_tests(
    _loader: unittest.TestLoader,
    suite: unittest.TestSuite,
    _pattern: str | None,
) -> unittest.TestSuite:
    functions = (
        value
        for name, value in sorted(globals().items())
        if name.startswith("test_") and callable(value)
    )
    suite.addTests(unittest.FunctionTestCase(function) for function in functions)
    return suite
