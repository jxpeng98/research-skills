import json
import os
import sys
from pathlib import Path

CONFIG_FILE = Path.home() / ".config" / "research-skills" / "config.json"

TRANSLATIONS = {
    "en": {
        "prompt_lang": "Select CLI Language / 请选择 CLI 语言 [en/zh-CN] (default: en): ",
        "lang_set": "Language set to {lang}.",
        "parallel_execution": "## Parallel Execution Analysis",
        "parallel_execution_dual": "## Parallel Execution (dual)",
        "parallel_degraded": "- Automatically degraded to dual-agent mode.",
        "failed_agents": "- Failed agents: {agents}",
        "synthesis": "## Synthesis",
        "missing_capabilities": "## ⚠️ Capabilities Missing:",
        "unknown_profile": "## ❌ Unknown agent profile -> {profile}",
        "task_run_start": "## Task Run Mode",
        "agent_profiles": "## Agent Profiles",
        "draft": "## Draft ({agent})",
        "review": "## Review ({agent})",
        "triad_audit": "## Triad Independent Audit ({agent})",
        "doctor_fail": "Research environment missing required elements. See details above.",
        "doctor_summary": "## Doctor Summary",
        "check_details": "## Check Details",
        "recommendations": "## Recommendations",
        "task_packet": "## Task Packet",
        "mcp_evidence": "## MCP Evidence",
        "skill_cards": "## Skill Cards",
        "routing": "## Routing",
        "revision_history": "## Revision History",
    },
    "zh-CN": {
        "prompt_lang": "Select CLI Language / 请选择 CLI 语言 [en/zh-CN] (default: zh-CN): ",
        "lang_set": "界面语言已设为 {lang}。",
        "parallel_execution": "## 并发执行分析 (Parallel Execution)",
        "parallel_execution_dual": "## 并发执行分析 (双重/Dual)",
        "parallel_degraded": "- 已自动降级为双端执行模式。",
        "failed_agents": "- 失败的 Agent: {agents}",
        "synthesis": "## 综合归纳 (Synthesis)",
        "missing_capabilities": "## ⚠️ 缺少必要能力或映射:",
        "unknown_profile": "## ❌ 未知的设定预案 (Profile) -> {profile}",
        "task_run_start": "## 任务执行模式 (Task Run)",
        "agent_profiles": "## 运行预案 (Agent Profiles)",
        "draft": "## 起草阶段草稿 ({agent})",
        "review": "## 复核阶段审查 ({agent})",
        "triad_audit": "## 三方独立审查 ({agent})",
        "doctor_fail": "研究环境缺失必要组件，请查看上述诊断详情。",
        "doctor_summary": "## 环境诊断总结 (Doctor Summary)",
        "check_details": "## 检查详情 (Check Details)",
        "recommendations": "## 建议与修复项 (Recommendations)",
        "task_packet": "## 任务数据包 (Task Packet)",
        "mcp_evidence": "## MCP 证据采集 (MCP Evidence)",
        "skill_cards": "## 技能规范卡 (Skill Cards)",
        "routing": "## 路由路径 (Routing)",
        "revision_history": "## 修订历程 (Revision History)",
    }
}

_current_lang = None

def _load_config() -> dict:
    if not CONFIG_FILE.exists():
        return {}
    try:
        with open(CONFIG_FILE, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception:
        return {}

def _save_config(config: dict) -> None:
    CONFIG_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(CONFIG_FILE, "w", encoding="utf-8") as f:
        json.dump(config, f, indent=2, ensure_ascii=False)

def get_language() -> str:
    global _current_lang
    if _current_lang:
        return _current_lang

    env_lang = os.environ.get("RESEARCH_CLI_LANG")
    if env_lang in TRANSLATIONS:
        _current_lang = env_lang
        return _current_lang

    config = _load_config()
    saved_lang = config.get("cli_lang")
    if saved_lang in TRANSLATIONS:
        _current_lang = saved_lang
        return _current_lang

    if not sys.stdout.isatty():
        _current_lang = "en"
        return _current_lang

    prompt_msg = TRANSLATIONS["en"]["prompt_lang"]
    user_input = input(prompt_msg).strip()
    
    if user_input in TRANSLATIONS:
        _current_lang = user_input
    else:
        _current_lang = "en"
    
    config["cli_lang"] = _current_lang
    _save_config(config)
    print(get_text("lang_set", lang=_current_lang))
    return _current_lang

def get_text(key: str, **kwargs) -> str:
    lang = get_language()
    dict_map = TRANSLATIONS.get(lang, TRANSLATIONS["en"])
    text = dict_map.get(key, TRANSLATIONS["en"].get(key, key))
    if kwargs:
        try:
            text = text.format(**kwargs)
        except KeyError:
            pass
    return text
