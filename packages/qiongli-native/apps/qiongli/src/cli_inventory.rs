//! Read-only cross-channel discovery and user-operated migration guidance.
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::command::CommandEnvironment;

const LIMIT: u64 = 1_048_576;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CliInstallation {
    pub channel: &'static str,
    pub version: Option<String>,
    pub version_source: &'static str,
    pub running: bool,
    pub active_commands: Vec<String>,
    pub entries: Vec<PathBuf>,
    pub identity: PathBuf,
    pub manager_root: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CliInventory {
    pub schema_version: u32,
    pub scope: &'static str,
    pub attention: bool,
    pub installations: Vec<CliInstallation>,
    pub cleanup: &'static str,
}

fn read_small(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > LIMIT {
        return None;
    }
    let file = fs::File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut text = String::new();
    file.take(LIMIT + 1).read_to_string(&mut text).ok()?;
    (text.len() as u64 <= LIMIT).then_some(text)
}

fn metadata_field(text: &str, field: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.strip_prefix(field).map(str::trim).map(str::to_owned))
}

fn version_text(value: &str) -> Option<String> {
    (!value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".+-".contains(&byte)))
    .then(|| value.to_owned())
}

fn classify(path: &Path) -> CliInstallation {
    let real = fs::canonicalize(path).unwrap_or_else(|_| path.to_owned());
    let mut item = CliInstallation {
        channel: "unknown",
        version: None,
        version_source: "unavailable",
        running: false,
        active_commands: vec![],
        entries: vec![path.to_owned()],
        identity: real.clone(),
        manager_root: None,
    };
    if path.parent().and_then(Path::file_name) == Some(OsStr::new("shims")) {
        item.channel = "shim";
        return item;
    }
    // Metadata is evidence of a package version, not proof of arbitrary binary bytes.
    for parent in real.ancestors().skip(1).take(5) {
        if let Some(text) = read_small(&parent.join("package.json"))
            && let Ok(meta) = serde_json::from_str::<serde_json::Value>(&text)
            && meta["name"] == "qiongli"
            && (real == parent.join("bin/qiongli.mjs") || real.starts_with(parent.join("native")))
        {
            item.channel = "npm";
            item.version = meta["version"].as_str().and_then(version_text);
            item.version_source = "package-metadata";
            item.identity = parent.to_owned();
            item.manager_root = parent.parent().and_then(Path::parent).map(|root| {
                if root.file_name() == Some(OsStr::new("lib")) {
                    root.parent().unwrap_or(root).to_owned()
                } else {
                    root.to_owned()
                }
            });
            return item;
        }
        if parent.file_name() == Some(OsStr::new("site-packages")) {
            identify_python(&mut item, parent);
            return item;
        }
    }
    // Windows npm shims sit next to node_modules instead of linking to the launcher.
    if let Some(parent) = path.parent() {
        let launcher = parent.join("node_modules/qiongli/bin/qiongli.mjs");
        if read_small(path).is_some_and(|text| {
            text.replace('\\', "/")
                .contains("node_modules/qiongli/bin/qiongli.mjs")
        }) && launcher.is_file()
        {
            let mut npm = classify(&launcher);
            npm.entries = item.entries;
            return npm;
        }
        if let Some(root) = parent.parent() {
            if let Some(text) = read_small(&root.join(".crates2.json"))
                && let Ok(meta) = serde_json::from_str::<serde_json::Value>(&text)
                && let Some(installs) = meta["installs"].as_object()
            {
                let matches = installs
                    .iter()
                    .filter(|(name, value)| {
                        name.starts_with("qiongli ")
                            && value["bins"].as_array().is_some_and(|bins| {
                                bins.iter().any(|bin| {
                                    bin.as_str().is_some_and(|name| {
                                        path.file_name() == Some(OsStr::new(name))
                                    })
                                })
                            })
                    })
                    .collect::<Vec<_>>();
                if let [(name, _)] = matches.as_slice() {
                    item.channel = "cargo";
                    item.version = name.split_whitespace().nth(1).and_then(version_text);
                    item.version_source = "package-metadata";
                    item.identity = root.join(".crates2.json");
                    item.manager_root = Some(root.to_owned());
                    return item;
                }
            }
            let mut sites = vec![root.join("Lib/site-packages")];
            if let Ok(children) = fs::read_dir(root.join("lib")) {
                sites.extend(
                    children
                        .take(128)
                        .flatten()
                        .filter(|entry| entry.file_name().to_string_lossy().starts_with("python"))
                        .map(|entry| entry.path().join("site-packages")),
                );
            }
            // Only associate a Python entry when its wrapper or binary belongs to that environment.
            let python_wrapper = read_small(path).is_some_and(|text| {
                text.contains("from qiongli_native import main")
                    || text.contains("from qiongli.cli import main")
            });
            if python_wrapper
                || (cfg!(windows) && parent.file_name() == Some(OsStr::new("Scripts")))
            {
                for site in sites {
                    identify_python(&mut item, &site);
                    if item.channel == "pypi" {
                        return item;
                    }
                }
            }
        }
    }
    item
}

fn identify_python(item: &mut CliInstallation, site: &Path) {
    let Ok(children) = fs::read_dir(site) else {
        return;
    };
    let versions = children
        .take(4096)
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".dist-info"))
        .filter_map(|entry| read_small(&entry.path().join("METADATA")))
        .filter(|text| metadata_field(text, "Name:").as_deref() == Some("qiongli"))
        .filter_map(|text| {
            metadata_field(&text, "Version:")
                .as_deref()
                .and_then(version_text)
        })
        .collect::<Vec<_>>();
    if versions.is_empty() {
        return;
    }
    item.channel = "pypi";
    item.version = (versions.len() == 1).then(|| versions[0].clone());
    item.version_source = if versions.len() == 1 {
        "package-metadata"
    } else {
        "ambiguous-package-metadata"
    };
    item.identity = fs::canonicalize(site)
        .unwrap_or_else(|_| site.to_owned())
        .join("qiongli_native");
    item.manager_root = site.parent().and_then(Path::parent).map(|parent| {
        if parent.file_name() == Some(OsStr::new("lib")) {
            parent.parent().unwrap_or(parent).to_owned()
        } else {
            parent.to_owned()
        }
    });
}

fn npm_prefix(home: &Path) -> Option<PathBuf> {
    let text = read_small(&home.join(".npmrc"))?;
    let value = text
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(key, _)| key.trim() == "prefix")
        .map(|(_, value)| value.trim().trim_matches(['\'', '"']))
        .next_back()?;
    if let Some(relative) = value.strip_prefix("~/") {
        Some(home.join(relative))
    } else {
        let path = PathBuf::from(value);
        path.is_absolute().then_some(path)
    }
}

pub(crate) fn discover(environment: &CommandEnvironment) -> CliInventory {
    let mut directories = environment
        .cli_search_path()
        .map(std::env::split_paths)
        .into_iter()
        .flatten()
        .filter(|path| path.is_absolute())
        .collect::<Vec<_>>();
    let path_count = directories.len();
    directories.extend_from_slice(environment.cli_extra_bins());
    if let Some(home) = environment.platform_home() {
        directories.extend([
            home.join(".local/bin"),
            home.join(".cargo/bin"),
            home.join(".node/bin"),
            home.join("AppData/Roaming/npm"),
        ]);
        if let Some(prefix) = npm_prefix(home) {
            directories.push(if cfg!(windows) {
                prefix
            } else {
                prefix.join("bin")
            });
        }
    }
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();
    let mut active = BTreeSet::new();
    for (index, directory) in directories.iter().take(512).enumerate() {
        for command in ["qiongli", "ql"] {
            let names = if cfg!(windows) {
                vec![
                    format!("{command}.exe"),
                    format!("{command}.cmd"),
                    format!("{command}.ps1"),
                ]
            } else {
                vec![command.to_owned()]
            };
            for name in names {
                let path = directory.join(name);
                if !crate::cli_install::is_executable_file(&path) || !seen.insert(path.clone()) {
                    continue;
                }
                let mut item = classify(&path);
                if index < path_count && active.insert(command) {
                    item.active_commands.push(command.to_owned());
                }
                entries.push(item);
            }
        }
    }
    if let Some(executable) = environment.cli_executable() {
        let mut running = classify(executable);
        running.running = true;
        running.version = Some(env!("CARGO_PKG_VERSION").to_owned());
        running.version_source = "running-executable";
        entries.push(running);
    }
    let mut installations: Vec<CliInstallation> = Vec::new();
    for mut item in entries {
        if let Some(existing) = installations
            .iter_mut()
            .find(|entry| entry.identity == item.identity)
        {
            existing.entries.append(&mut item.entries);
            existing.active_commands.append(&mut item.active_commands);
            if item.running {
                existing.running = true;
                existing.version = item.version;
                existing.version_source = item.version_source;
            }
            existing.entries.sort();
            existing.entries.dedup();
        } else {
            installations.push(item);
        }
    }
    let attention = installations
        .iter()
        .filter(|item| item.channel != "shim")
        .count()
        > 1
        || installations.iter().any(|item| item.channel == "shim");
    CliInventory {
        schema_version: 1,
        scope: "PATH-and-known-user-prefixes; aliases-functions-and-unlisted-environments-not-resolved",
        attention,
        installations,
        cleanup: "user-operated-only",
    }
}

impl CliInventory {
    pub(crate) fn redact(mut self) -> Self {
        for (index, item) in self.installations.iter_mut().enumerate() {
            item.identity = PathBuf::from(format!("cli-installation-{}", index + 1));
            item.entries = (0..item.entries.len())
                .map(|entry| PathBuf::from(format!("cli-entry-{}-{}", index + 1, entry + 1)))
                .collect();
            item.manager_root = item
                .manager_root
                .as_ref()
                .map(|_| PathBuf::from(format!("cli-manager-{}", index + 1)));
        }
        self
    }
}

/// Interactive review produces instructions only; it never changes installed files or PATH.
pub fn review_cli_installations(environment: &CommandEnvironment) -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other(
            "interactive-terminal-required; use install inventory --paths exact",
        ));
    }
    review(
        &discover(environment),
        &mut io::stdin().lock(),
        &mut io::stdout().lock(),
    )
}

fn choice(reader: &mut impl BufRead, writer: &mut impl Write, prompt: &str) -> io::Result<String> {
    write!(writer, "{prompt}")?;
    writer.flush()?;
    let mut line = String::new();
    reader.take(128).read_line(&mut line)?;
    if !line.ends_with('\n') {
        return Err(io::Error::other("selection-cancelled; no changes made"));
    }
    Ok(line.trim().to_owned())
}

fn display(path: &Path) -> String {
    // JSON quoting keeps filenames from injecting terminal control sequences.
    serde_json::to_string(&path.to_string_lossy()).unwrap_or_default()
}

fn review(
    inventory: &CliInventory,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> io::Result<()> {
    writeln!(
        writer,
        "Qiongli installation review — no files or settings will be changed."
    )?;
    for (index, item) in inventory.installations.iter().enumerate() {
        writeln!(
            writer,
            "{}. {} | {} | running={} | PATH={:?}",
            index + 1,
            item.channel,
            serde_json::to_string(&item.version).unwrap_or_default(),
            item.running,
            item.active_commands
        )?;
        for path in &item.entries {
            writeln!(writer, "   {}", display(path))?;
        }
    }
    writeln!(
        writer,
        "Package versions come from metadata. Unknown binaries are not executed. Shell aliases and unlisted environments need separate review."
    )?;
    let primary = choice(
        reader,
        writer,
        "Preferred installation number [Enter: keep current setup]: ",
    )?;
    if primary.is_empty() {
        return writeln!(writer, "Kept current setup. No changes made.");
    }
    let index = primary
        .parse::<usize>()
        .ok()
        .and_then(|index| index.checked_sub(1))
        .filter(|index| *index < inventory.installations.len())
        .ok_or_else(|| io::Error::other("invalid-installation-selection; no changes made"))?;
    let selected = &inventory.installations[index];
    if selected.channel == "shim" {
        return Err(io::Error::other(
            "select-an-installation-not-a-shim; no changes made",
        ));
    }
    writeln!(
        writer,
        "Preferred installation: {}. Verify its full command path before changing PATH; check both qiongli and ql and any Host MCP command.",
        display(&selected.identity)
    )?;
    for (other, item) in inventory.installations.iter().enumerate() {
        if other == index || item.channel == "shim" {
            continue;
        }
        let action = choice(
            reader,
            writer,
            &format!(
                "Installation {}: [k]eep, [a]rchive guidance, [u]ninstall guidance [k]: ",
                other + 1
            ),
        )?;
        match action.as_str() {
            "" | "k" => writeln!(writer, "Keep {}.", display(&item.identity))?,
            "a" => {
                if ["npm", "pypi", "cargo"].contains(&item.channel) {
                    writeln!(
                        writer,
                        "Keep this package in place. Record its version and original manager environment for reinstalling; do not move package-manager files."
                    )?;
                } else {
                    writeln!(
                        writer,
                        "Confirm ownership of {} first. For a standalone release, copy the complete release bundle to a separate backup and verify checksums. Copying does not deactivate the old command; no move or deletion is proposed.",
                        display(&item.identity)
                    )?;
                }
            }
            "u" => {
                writeln!(
                    writer,
                    "Selected for manual review: {} ({})",
                    display(&item.identity),
                    item.channel
                )?;
                if let Some(root) = &item.manager_root {
                    writeln!(
                        writer,
                        "Original manager environment/prefix: {}",
                        display(root)
                    )?;
                }
                let instruction = match item.channel {
                    "npm" => "Use npm uninstall -g qiongli with the original --prefix.",
                    "pypi" => {
                        "Use that environment's Python -m pip uninstall qiongli after confirming package ownership; use pipx uninstall only if pipx owns it."
                    }
                    "cargo" => "Use cargo uninstall qiongli with the original --root.",
                    _ => {
                        "Ownership is unknown. Review the exact files manually; no uninstall command is available."
                    }
                };
                writeln!(
                    writer,
                    "{instruction} Check shared command files and Host references first: uninstalling an old package can remove a newer shared entry. Research data, configuration and Plugin caches are excluded."
                )?;
            }
            _ => return Err(io::Error::other("invalid-action; no changes made")),
        }
    }
    writeln!(
        writer,
        "Review complete. No files were deleted, moved or archived; PATH and Host settings are unchanged."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_groups_package_entries_and_never_executes_unknown_commands() {
        let root = std::env::temp_dir().join(format!(
            "qiongli-cli-inventory-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let write = |relative: &str, content: &str| {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, content).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
            }
            path
        };
        let executable_name = if cfg!(windows) {
            "qiongli.exe"
        } else {
            "qiongli"
        };
        let unknown = write(
            &format!("old/bin/{executable_name}"),
            "#!/bin/sh\nexit 99\n",
        );
        let npm = write(
            "npm wrong prefix/lib/node_modules/qiongli/bin/qiongli.mjs",
            "launcher",
        );
        write(
            "npm wrong prefix/lib/node_modules/qiongli/package.json",
            r#"{"name":"qiongli","version":"2.0.0-beta.2"}"#,
        );
        let current = write(
            "npm wrong prefix/lib/node_modules/qiongli/native/test/qiongli",
            "binary",
        );
        let cargo = write(&format!("cargo/bin/{executable_name}"), "binary");
        write(
            "cargo/.crates2.json",
            &format!(
                r#"{{"installs":{{"qiongli 2.0.0-alpha.8 (registry+test)":{{"bins":["{executable_name}"]}}}}}}"#
            ),
        );
        let python = write(
            &format!("python/bin/{executable_name}"),
            "#!/python\nfrom qiongli_native import main\n",
        );
        write(
            "python/lib/python3.12/site-packages/qiongli-2.0.0a7.dist-info/METADATA",
            "Name: qiongli\nVersion: 2.0.0a7\n",
        );
        let npm_bin = root.join("npm wrong prefix/bin");
        fs::create_dir(&npm_bin).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&npm, npm_bin.join("qiongli")).unwrap();
            std::os::unix::fs::symlink(&npm, npm_bin.join("ql")).unwrap();
        }
        #[cfg(windows)]
        {
            // Check the same npm metadata owner directly on Windows.
            assert_eq!(classify(&npm).channel, "npm");
        }
        let windows_shim = write(
            "windows-npm/qiongli.cmd",
            r#"@node "%dp0%\node_modules\qiongli\bin\qiongli.mjs" %*"#,
        );
        write(
            "windows-npm/node_modules/qiongli/bin/qiongli.mjs",
            "launcher",
        );
        write(
            "windows-npm/node_modules/qiongli/package.json",
            r#"{"name":"qiongli","version":"2.0.0-beta.2"}"#,
        );
        assert_eq!(classify(&windows_shim).channel, "npm");
        write(".npmrc", "prefix=\"~/npm wrong prefix\"\n");
        let search = std::env::join_paths([
            unknown.parent().unwrap(),
            python.parent().unwrap(),
            cargo.parent().unwrap(),
            &npm_bin,
        ])
        .unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(root.clone()), None)
            .with_cli_paths(search, current);
        let inventory = discover(&environment);
        assert!(inventory.attention);
        assert_eq!(inventory.installations.len(), 4);
        assert_eq!(inventory.installations[0].active_commands, ["qiongli"]);
        let npm = inventory
            .installations
            .iter()
            .find(|item| item.channel == "npm")
            .unwrap();
        assert!(npm.running);
        assert_eq!(npm.version.as_deref(), Some(env!("CARGO_PKG_VERSION")));
        assert_eq!(
            npm.manager_root.as_ref(),
            Some(&root.join("npm wrong prefix"))
        );
        assert_eq!(
            inventory
                .installations
                .iter()
                .find(|item| item.channel == "pypi")
                .unwrap()
                .version
                .as_deref(),
            Some("2.0.0a7")
        );
        assert_eq!(
            inventory
                .installations
                .iter()
                .find(|item| item.channel == "cargo")
                .unwrap()
                .version
                .as_deref(),
            Some("2.0.0-alpha.8")
        );
        assert_eq!(
            fs::read_to_string(&unknown).unwrap(),
            "#!/bin/sh\nexit 99\n"
        );
        assert!(root.join(".npmrc").exists());
        assert!(
            !serde_json::to_string(&inventory.redact())
                .unwrap()
                .contains(root.to_string_lossy().as_ref())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migration_choices_are_scoped_read_only_and_paths_redact() {
        assert!(version_text("/private/example").is_none());
        assert!(version_text("\u{1b}[31mversion").is_none());
        let item = CliInstallation {
            channel: "npm",
            version: Some("2.0.0-beta.2".into()),
            version_source: "package-metadata",
            running: true,
            active_commands: vec![],
            entries: vec![PathBuf::from("/private/example/bin/qiongli")],
            identity: PathBuf::from("/private/example/npm"),
            manager_root: Some(PathBuf::from("/private/example")),
        };
        let inventory = CliInventory {
            schema_version: 1,
            scope: "test",
            attention: true,
            installations: vec![
                item.clone(),
                CliInstallation {
                    channel: "unknown",
                    ..item
                },
            ],
            cleanup: "user-operated-only",
        };
        for input in ["\n", "1\nk\n", "1\na\n", "1\nu\n", "2\na\n"] {
            let mut output = vec![];
            review(&inventory, &mut io::Cursor::new(input), &mut output).unwrap();
            assert!(String::from_utf8(output).unwrap().contains("No "));
        }
        for input in ["", "0\n", "3\n", "1\n", "1\ny\n", "-1\n"] {
            assert!(review(&inventory, &mut io::Cursor::new(input), &mut vec![]).is_err());
        }
        assert!(
            !serde_json::to_string(&inventory.redact())
                .unwrap()
                .contains("/private/example")
        );
    }
}
