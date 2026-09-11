//! Terminal presentation over the existing command results; never a write owner.
use std::ffi::{OsStr, OsString};

use qiongli_content::EmbeddedContent;
use serde_json::Value;

use crate::{CliOutput, CommandEnvironment, ProductAction};

pub fn prepare_cli_action(
    mut args: Vec<OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    terminal: bool,
) -> ProductAction {
    let mut text_mode = None;
    while let Some(mode) = args.first().and_then(|arg| output_mode(arg)) {
        if text_mode.replace(mode).is_some() {
            return output_option_error();
        }
        args.remove(0);
    }
    // A valid trailing value such as --name "--text" belongs to its command.
    // The existing `paths --json` option must also retain its original meaning.
    while let Some(mode) = args.last().and_then(|arg| output_mode(arg)) {
        let paths_json = args == [OsString::from("paths"), OsString::from("--json")];
        if !paths_json && crate::command::accepts_arguments(&args) {
            break;
        }
        if text_mode.replace(mode).is_some() {
            return output_option_error();
        }
        args.pop();
    }
    if text_mode.is_some() && args.is_empty() {
        args.push("--help".into());
    }
    let readable = text_mode.unwrap_or(terminal);
    if args == [OsString::from("paths")] && (readable || text_mode == Some(false)) {
        args.push("--json".into());
    }
    // Validate output options before executing any command. Streaming and
    // interactive commands have their own protocols and cannot be reformatted.
    if text_mode.is_some()
        && (args
            .first()
            .is_some_and(|arg| arg == "setup" || arg == "ui")
            || args.starts_with(&["mcp".into(), "serve".into()])
            || args.starts_with(&["install".into(), "review".into()])
            || args.starts_with(&["install".into(), "migrate".into()]))
    {
        return ProductAction::Output(CliOutput::usage_text(
            "output options apply to queries, not interactive or streaming commands",
        ));
    }
    match crate::prepare_action(args, environment, content) {
        ProductAction::Output(output) if readable => ProductAction::Output(readable_output(output)),
        action => action,
    }
}

fn output_mode(arg: &OsStr) -> Option<bool> {
    match arg.to_str() {
        Some("--text") => Some(true),
        Some("--json") => Some(false),
        _ => None,
    }
}

fn output_option_error() -> ProductAction {
    ProductAction::Output(CliOutput::usage_text("choose --json or --text once"))
}

fn readable_output(output: CliOutput) -> CliOutput {
    if output.stdout().is_empty()
        && let Some(code) = output.stderr().trim().strip_prefix("error: ")
        && !code.is_empty()
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        let message = match code {
            "source-build-read-only" | "native-release-authority-unavailable" =>
                "This operation requires a verified managed installation. For registry packages, use the original package manager to upgrade or remove the CLI.".to_owned(),
            "managed-skills-plan-required" =>
                "Preview managed Skills changes with `qiongli app plan --help`, then approve the exact plan.".to_owned(),
            _ => label(code),
        };
        let message = format!("error: {message}\n  Code: {code}\n");
        return output.with_stderr(message);
    }
    let Ok(value) = serde_json::from_str::<Value>(output.stdout()) else {
        return output;
    };
    let command = value["command"].as_str().unwrap_or("result");
    let mut text = String::new();
    match command {
        "status" => {
            text.push_str(&format!(
                "Qiongli {}\n\n",
                scalar(&value["product_version"])
            ));
            row(&mut text, "Content", &value["content"]["state"]);
            row(
                &mut text,
                "Content version",
                &value["content"]["content_version"],
            );
            row(&mut text, "Configuration", &value["config"]["state"]);
            row(
                &mut text,
                "Default profile",
                &value["config"]["default_profile"],
            );
            text.push_str("\nNext: qiongli doctor  |  qiongli project  |  qiongli mcp --help\n");
        }
        "doctor" => {
            text.push_str(&format!(
                "Installation health: {}\n\n",
                scalar(&value["overall"])
            ));
            for check in array(&value["checks"]) {
                // The legacy wire check concerns in-process execution, not Full MCP.
                if check["id"] == "full-runtime"
                    && check["remediation"] == "upgrade-to-r4-full-runtime"
                {
                    text.push_str("  [deferred    ] Legacy in-process runtime\n      Models run in your Host. Connect Full MCP: qiongli mcp serve --profile full\n");
                    continue;
                }
                text.push_str(&format!(
                    "  [{:<12}] {}{}\n",
                    scalar(&check["state"]),
                    label(check["id"].as_str().unwrap_or("check")),
                    if check["blocking"] == true {
                        " (blocking)"
                    } else {
                        ""
                    }
                ));
                if check["state"] != "ready" {
                    row(&mut text, "    Reason", &check["code"]);
                    if check["remediation"]
                        .as_str()
                        .is_some_and(|value| value != "none")
                    {
                        text.push_str(&format!(
                            "      Next: {}\n",
                            label(check["remediation"].as_str().unwrap_or_default())
                        ));
                    }
                }
            }
            if !value["cli"].is_null() {
                installations(&mut text, &value["cli"]);
            }
            if let Some(paths) = value.get("paths") {
                paths_text(&mut text, paths);
            }
        }
        "install-inventory" => {
            installations(&mut text, &value["cli"]);
            text.push_str("\nHost discovery\n");
            for client in array(&value["inventory"]["clients"]) {
                text.push_str(&format!(
                    "\n  {}: {}\n",
                    scalar(&client["client"]),
                    scalar(&client["readiness"])
                ));
                row(&mut text, "    Host", &client["host_presence"]);
                row(
                    &mut text,
                    "    Plugin version",
                    &client["installed_plugin_version"],
                );
                row(&mut text, "    Reason", &client["reason_code"]);
            }
            text.push_str("\nNext: qiongli setup  |  qiongli install list --paths exact\n");
        }
        "project-list" => {
            let library = &value["library"];
            let projects = array(&library["projects"]);
            text.push_str(&format!(
                "Research projects: {} ({})\n",
                projects.len(),
                scalar(&library["health"])
            ));
            for project in projects {
                text.push_str(&format!(
                    "\n  {}\n    {} | {} | {}\n    ID: {}\n",
                    scalar(&project["displayName"]),
                    scalar(&project["stage"]),
                    scalar(&project["lifecycle"]),
                    scalar(&project["health"]),
                    scalar(&project["projectId"])
                ));
            }
            if projects.is_empty() {
                text.push_str("\nNo registered projects. Start with:\n  qiongli project create --help\n  qiongli project register --help\n");
            } else {
                text.push_str("\nNext: qiongli project show <id>\n");
            }
        }
        "content-list" => {
            text.push_str(&format!(
                "Embedded research content {}\n\n",
                scalar(&value["content_version"])
            ));
            for profile in array(&value["profiles"]) {
                let id = scalar(&profile["id"]);
                let purpose = match id.as_str() {
                    "skill-only" => "Workflow, Skills and research references",
                    "marketplace-lite" => "Research content and Lite MCP (alias: lite)",
                    "full" => "Research content and Full MCP",
                    _ => "Embedded content profile",
                };
                text.push_str(&format!("  {id:<18} {purpose}\n"));
            }
            text.push_str("\nConnect a Host: qiongli mcp --help\n");
        }
        "paths" => paths_text(&mut text, &value["paths"]),
        _ => {
            text.push_str(&format!("{}\n\n", label(command)));
            fields(&mut text, &value, 0);
        }
    }
    text.push_str("\nUse --json for the complete structured result.\n");
    output.with_stdout(text)
}

fn installations(text: &mut String, inventory: &Value) {
    let entries = array(&inventory["installations"]);
    text.push_str(&format!("\nCLI installations: {}\n", entries.len()));
    for (index, item) in entries.iter().enumerate() {
        text.push_str(&format!(
            "\n  {}. {}  {}{}\n",
            index + 1,
            scalar(&item["channel"]),
            scalar(&item["version"]),
            if item["running"] == true {
                "  (running now)"
            } else {
                ""
            }
        ));
        row(text, "    Version source", &item["version_source"]);
        if !array(&item["active_commands"]).is_empty() {
            row(text, "    First on PATH", &item["active_commands"]);
        }
        if array(&item["entries"])
            .first()
            .and_then(Value::as_str)
            .is_some_and(|path| path.starts_with("cli-entry-"))
        {
            text.push_str("    Location hidden; use --paths exact to show it.\n");
        } else {
            for path in array(&item["entries"]) {
                text.push_str(&format!("    {}\n", scalar(path)));
            }
        }
    }
    text.push_str("\nDetection covers PATH and known locations; aliases and other environments may need review.\nArchive and removal are user-operated.\n");
}

fn paths_text(text: &mut String, paths: &Value) {
    text.push_str("\nResolved paths (exact locations)\n");
    for path in array(paths) {
        text.push_str(&format!(
            "\n  {}\n    {}\n    State: {} | Safety: {}\n",
            scalar(&path["label"]),
            scalar(&path["exact_path"]),
            scalar(&path["file_type"]),
            scalar(&path["safety"])
        ));
    }
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

fn row(text: &mut String, name: &str, value: &Value) {
    text.push_str(&format!("  {name}: {}\n", scalar(value)));
}

fn safe(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            if ch.is_control() || matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
                ch.escape_default().collect::<Vec<_>>()
            } else {
                vec![ch]
            }
        })
        .collect()
}

fn scalar(value: &Value) -> String {
    match value {
        Value::Null => "not available".into(),
        Value::Bool(value) => if *value { "yes" } else { "no" }.into(),
        Value::String(value) => safe(value),
        Value::Array(values) if values.is_empty() => "none".into(),
        Value::Array(values) => values.iter().map(scalar).collect::<Vec<_>>().join(", "),
        other => safe(&other.to_string()),
    }
}

fn label(value: &str) -> String {
    let mut text = String::new();
    for ch in safe(value).chars() {
        if ch == '_' || ch == '-' {
            text.push(' ');
        } else if ch.is_ascii_uppercase() && !text.is_empty() {
            text.push(' ');
            text.push(ch.to_ascii_lowercase());
        } else {
            text.push(ch);
        }
    }
    if let Some(first) = text.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    text
}

// Preserve every field of plans, receipts and less common results. Only known
// read-only overviews above are summarized; approval IDs/digests are never cut.
fn fields(text: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                if indent == 0
                    && matches!(key.as_str(), "schema_version" | "schemaVersion" | "command")
                {
                    continue;
                }
                text.push_str(&format!("{}{}:", " ".repeat(indent), label(key)));
                if value.is_object()
                    || (value.is_array() && array(value).iter().any(Value::is_object))
                {
                    text.push('\n');
                    fields(text, value, indent + 2);
                } else {
                    text.push_str(&format!(" {}\n", scalar(value)));
                }
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                text.push_str(&format!("{}{}.\n", " ".repeat(indent), index + 1));
                fields(text, value, indent + 2);
            }
        }
        _ => text.push_str(&format!("{}{}\n", " ".repeat(indent), scalar(value))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readable_results_preserve_exit_status_redaction_and_approval_values() {
        let json = serde_json::json!({"command":"project-create-preview", "preview": {
            "displayName":"A\u{001b}[2J\nB", "planDigest":"a".repeat(64),
            "projectId":"prj_00000000000000000000000000000001", "warnings":["review first"]
        }});
        let output = readable_output(CliOutput::success_text(json.to_string()));
        assert!(!output.stdout().contains('\u{001b}'));
        assert!(output.stdout().contains("\\nB"));
        assert!(output.stdout().contains(&"a".repeat(64)));
        assert!(output.stdout().contains("review first"));
        let output = CliOutput::operation_failure("failure").with_stdout(serde_json::json!({
            "command":"doctor", "overall":"attention", "checks":[{"id":"global-config", "state":"insecure", "blocking":true,"code":"global-config-insecure","remediation":"repair-permissions"},{"id":"full-runtime","state":"deferred","remediation":"upgrade-to-r4-full-runtime"}]
        }).to_string());
        let output = readable_output(output);
        assert_eq!(output.exit_code(), 1);
        assert!(output.stdout().contains("(blocking)"));
        assert!(output.stdout().contains("Repair permissions"));
        assert!(output.stdout().contains("Models run in your Host"));
        assert!(!output.stdout().contains("r4"));
        assert_eq!(output.stderr(), "error: failure\n");
    }
}
