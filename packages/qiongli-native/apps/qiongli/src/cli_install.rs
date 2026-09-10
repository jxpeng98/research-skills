use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const CLI_RECEIPT_SCHEMA_VERSION: u32 = 3;
const LEGACY_CLI_RECEIPT_SCHEMA_VERSION: u32 = 1;
const AUTHORITY_CLI_RECEIPT_SCHEMA_VERSION: u32 = 2;
const MAX_RECEIPT_BYTES: u64 = 16 * 1024;
const MAX_SHELL_PROFILE_BYTES: u64 = 256 * 1024;
const CLI_PATH_RECEIPT_SCHEMA_VERSION: u32 = 1;
const CLI_PATH_MARKER: &str = concat!(
    "# >>> qiongli managed cli path >>>\n",
    "export PATH=\"$HOME/.local/bin:$PATH\"\n",
    "# <<< qiongli managed cli path <<<\n"
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CliInstallState {
    Missing,
    InstalledCurrent,
    UpdateAvailable,
    Unavailable,
    Conflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CliPathState {
    Active,
    Configured,
    NotConfigured,
    Shadowed,
    VersionMismatch,
    NotObservable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CliInstallInspection {
    pub(crate) state: CliInstallState,
    pub(crate) installed_version: Option<String>,
    pub(crate) available_version: String,
    pub(crate) target: PathBuf,
    pub(crate) path_state: CliPathState,
    pub(crate) reason_code: &'static str,
    pub(crate) can_install: bool,
    pub(crate) can_test: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct CliInstallPlan {
    home: PathBuf,
    source: PathBuf,
    pub(crate) target: PathBuf,
    pub(crate) receipt_path: PathBuf,
    pub(crate) product_version: String,
    pub(crate) source_sha256: String,
    expected_target: TargetObservation,
    previous_managed: bool,
    previous_receipt_sha256: Option<String>,
    retained_backup_name: Option<String>,
    retained_backup_sha256: Option<String>,
    packaged_authority: Option<CliProductAuthorityBinding>,
    plan_sha256: String,
}

impl CliInstallPlan {
    pub(crate) fn plan_sha256(&self) -> &str {
        &self.plan_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CliRemovalEffect {
    Removed,
    RestoredPredecessor,
}

#[derive(Clone, Debug)]
pub(crate) struct CliRemovalPlan {
    home: PathBuf,
    target: PathBuf,
    receipt_path: PathBuf,
    receipt_sha256: String,
    installed_sha256: String,
    retained_backup_name: Option<String>,
    retained_backup_sha256: Option<String>,
    search_path: Option<OsString>,
    shell: Option<OsString>,
    expected_path_state: CliPathState,
    plan_sha256: String,
}

impl CliRemovalPlan {
    pub(crate) fn plan_sha256(&self) -> &str {
        &self.plan_sha256
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CliPathConfigurePlan {
    home: PathBuf,
    target: PathBuf,
    install_receipt_sha256: String,
    installed_sha256: String,
    profile_name: String,
    profile_path: PathBuf,
    previous_profile_bytes: Option<Vec<u8>>,
    expected_profile_sha256: Option<String>,
    next_profile_bytes: Vec<u8>,
    next_profile_sha256: String,
    path_receipt_path: PathBuf,
    plan_sha256: String,
}

impl CliPathConfigurePlan {
    pub(crate) fn plan_sha256(&self) -> &str {
        &self.plan_sha256
    }

    pub(crate) fn profile_name(&self) -> &str {
        &self.profile_name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CliProductAuthorityBinding {
    packaged_executable: PathBuf,
    desktop_manifest_path: PathBuf,
    control_sha256: String,
}

impl CliProductAuthorityBinding {
    pub(crate) fn packaged_executable(&self) -> &Path {
        &self.packaged_executable
    }

    pub(crate) fn desktop_manifest_path(&self) -> &Path {
        &self.desktop_manifest_path
    }

    pub(crate) fn control_sha256(&self) -> &str {
        &self.control_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TargetObservation {
    Missing,
    RegularFile(String),
    Symlink(String),
    Unsupported,
}

impl TargetObservation {
    fn fingerprint(&self) -> String {
        match self {
            Self::Missing => "missing".to_owned(),
            Self::RegularFile(sha256) => format!("file:{sha256}"),
            Self::Symlink(sha256) => format!("symlink:{sha256}"),
            Self::Unsupported => "unsupported".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CliInstallReceiptV1 {
    schema_version: u32,
    pub(crate) product_version: String,
    pub(crate) installed_sha256: String,
    target_name: String,
    retained_backup_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retained_backup_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    packaged_authority: Option<CliProductAuthorityReceiptV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CliProductAuthorityReceiptV1 {
    packaged_executable: String,
    desktop_manifest_path: String,
    control_sha256: String,
}

#[derive(Serialize)]
struct CliInstallPlanDigest<'a> {
    schema_version: u32,
    product_version: &'a str,
    source_sha256: &'a str,
    target_name: &'a str,
    expected_target: String,
    previous_managed: bool,
    previous_receipt_sha256: Option<&'a str>,
    retained_backup_name: Option<&'a str>,
    retained_backup_sha256: Option<&'a str>,
    packaged_executable: Option<&'a str>,
    desktop_manifest_path: Option<&'a str>,
    control_sha256: Option<&'a str>,
}

#[derive(Serialize)]
struct CliRemovalPlanDigest<'a> {
    schema_version: u32,
    target_name: &'a str,
    receipt_sha256: &'a str,
    installed_sha256: &'a str,
    retained_backup_name: Option<&'a str>,
    retained_backup_sha256: Option<&'a str>,
    path_state: &'static str,
}

#[derive(Serialize)]
struct CliPathConfigurePlanDigest<'a> {
    schema_version: u32,
    installed_sha256: &'a str,
    install_receipt_sha256: &'a str,
    profile_name: &'a str,
    expected_profile_sha256: Option<&'a str>,
    next_profile_sha256: &'a str,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CliPathReceiptV1 {
    schema_version: u32,
    installed_sha256: String,
    profile_name: String,
    profile_before_sha256: Option<String>,
    profile_after_sha256: String,
}

pub(crate) fn bundled_cli_path(home: Option<&Path>) -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    bundled_cli_path_for(&executable, home)
}

fn bundled_cli_path_for(executable: &Path, home: Option<&Path>) -> Option<PathBuf> {
    if let Some(home) = home {
        if executable.starts_with(home.join(".qiongli/native/payloads")) {
            return Some(executable.to_path_buf());
        }
        if let Ok(Some(authority)) = crate::embedded_release_authority()
            && let Ok(artifact) = qiongli_platform::current_target_native_artifact_identity(
                env!("CARGO_PKG_VERSION"),
                authority.channel(),
            )
            && let Ok(Some(source)) = installed_native_cli_source(home, executable, &artifact)
        {
            return Some(source);
        }
    }

    let bundled_name = if cfg!(windows) {
        "qiongli-cli.exe"
    } else {
        "qiongli-cli"
    };
    if executable.file_name() == Some(OsStr::new(bundled_name)) {
        return Some(executable.to_path_buf());
    }
    if let Some(binding) =
        home.and_then(|home| installed_cli_product_authority(home, executable).ok())
    {
        return Some(binding.packaged_executable().to_path_buf());
    }
    let parent = executable.parent()?;
    Some(parent.join(bundled_name))
}

pub(crate) fn cli_target_matches_bundled(
    home: &Path,
    bundled: &Path,
) -> Result<bool, &'static str> {
    validate_install_roots(home)?;
    let target = cli_target(home);
    validate_target_ancestors(home, &target)?;
    let target_sha256 = match observe_target(&target)? {
        TargetObservation::RegularFile(sha256) => sha256,
        TargetObservation::Missing
        | TargetObservation::Symlink(_)
        | TargetObservation::Unsupported => return Ok(false),
    };
    Ok(target_sha256 == regular_file_sha256(bundled)?)
}

/// Resolves only the receipt-owned command copy to its fixed native source.
/// This is an identity hint, not authority: the caller must verify that source's
/// active installation and signed candidate with embedded identity and trust roots.
pub(crate) fn installed_native_cli_source(
    home: &Path,
    current_executable: &Path,
    artifact: &qiongli_platform::ArtifactIdentityV1,
) -> Result<Option<PathBuf>, &'static str> {
    let target = cli_target(home);
    let Ok(installed) = fs::canonicalize(&target) else {
        return Ok(None);
    };
    let current = fs::canonicalize(current_executable)
        .map_err(|_| "qiongli-cli-product-authority-unavailable")?;
    if current != installed {
        return Ok(None);
    }
    validate_install_roots(home)?;
    validate_target_ancestors(home, &target)?;
    let receipt = read_receipt(&cli_receipt_path(home))?
        .ok_or("qiongli-cli-product-authority-unavailable")?;
    if receipt.packaged_authority.is_some() || receipt.schema_version != CLI_RECEIPT_SCHEMA_VERSION
    {
        return Ok(None);
    }
    if receipt.product_version != artifact.version
        || regular_file_sha256(&target)? != receipt.installed_sha256
        || regular_file_sha256(&current)? != receipt.installed_sha256
    {
        return Err("qiongli-cli-product-authority-changed");
    }
    let source = home
        .join(".qiongli/native/payloads")
        .join(qiongli_platform::native_artifact_id(artifact).map_err(|error| error.reason_code())?)
        .join(
            qiongli_platform::native_artifact_binary_path(artifact)
                .map_err(|error| error.reason_code())?,
        );
    if regular_file_sha256(&source)? != receipt.installed_sha256 {
        return Err("qiongli-cli-product-authority-changed");
    }
    Ok(Some(source))
}

pub(crate) fn installed_cli_product_authority(
    home: &Path,
    current_executable: &Path,
) -> Result<CliProductAuthorityBinding, &'static str> {
    validate_install_roots(home)?;
    let target = cli_target(home);
    let current = fs::canonicalize(current_executable)
        .map_err(|_| "qiongli-cli-product-authority-unavailable")?;
    if current_executable != target {
        let installed = match fs::canonicalize(&target) {
            Ok(installed) => installed,
            Err(_) => return Err("qiongli-cli-not-managed-executable"),
        };
        if current != installed {
            return Err("qiongli-cli-not-managed-executable");
        }
    }
    let receipt = read_receipt(&cli_receipt_path(home))?
        .ok_or("qiongli-cli-product-authority-unavailable")?;
    if receipt.schema_version != CLI_RECEIPT_SCHEMA_VERSION
        || regular_file_sha256(&current)? != receipt.installed_sha256
    {
        return Err("qiongli-cli-product-authority-unavailable");
    }
    let recorded = receipt
        .packaged_authority
        .ok_or("qiongli-cli-product-authority-unavailable")?;
    let packaged_executable = validated_absolute_path(&recorded.packaged_executable)?;
    let desktop_manifest_path = validated_absolute_path(&recorded.desktop_manifest_path)?;
    let packaged_executable = fs::canonicalize(packaged_executable)
        .map_err(|_| "qiongli-cli-product-authority-unavailable")?;
    let desktop_manifest_path = fs::canonicalize(desktop_manifest_path)
        .map_err(|_| "qiongli-cli-product-authority-unavailable")?;
    let expected_manifest =
        fs::canonicalize(packaged_manifest_for_executable(&packaged_executable))
            .map_err(|_| "qiongli-cli-product-authority-unavailable")?;
    if desktop_manifest_path != expected_manifest
        || regular_file_sha256(&packaged_executable)? != receipt.installed_sha256
    {
        return Err("qiongli-cli-product-authority-changed");
    }
    let control_path = qiongli_platform::packaged_product_control_path(&desktop_manifest_path)
        .map_err(|error| error.reason_code())?;
    if regular_file_sha256(&control_path)? != recorded.control_sha256 {
        return Err("qiongli-cli-product-authority-changed");
    }
    Ok(CliProductAuthorityBinding {
        packaged_executable,
        desktop_manifest_path,
        control_sha256: recorded.control_sha256,
    })
}

fn detect_packaged_authority(
    source: &Path,
) -> Result<Option<CliProductAuthorityBinding>, &'static str> {
    let manifest = packaged_manifest_for_executable(source);
    let metadata = match fs::symlink_metadata(&manifest) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("qiongli-cli-product-authority-invalid"),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("qiongli-cli-product-authority-invalid");
    }
    let packaged_executable =
        fs::canonicalize(source).map_err(|_| "qiongli-cli-product-authority-invalid")?;
    let desktop_manifest_path =
        fs::canonicalize(manifest).map_err(|_| "qiongli-cli-product-authority-invalid")?;
    let control_path = qiongli_platform::packaged_product_control_path(&desktop_manifest_path)
        .map_err(|_| "qiongli-cli-product-authority-invalid")?;
    let control_sha256 =
        regular_file_sha256(&control_path).map_err(|_| "qiongli-cli-product-authority-invalid")?;
    Ok(Some(CliProductAuthorityBinding {
        packaged_executable,
        desktop_manifest_path,
        control_sha256,
    }))
}

fn packaged_manifest_for_executable(executable: &Path) -> PathBuf {
    let parent = executable.parent().unwrap_or(executable);
    if cfg!(target_os = "macos") {
        parent
            .join("../Resources")
            .join(qiongli_platform::DESKTOP_PACKAGE_MANIFEST_FILE)
    } else {
        parent.join(qiongli_platform::DESKTOP_PACKAGE_MANIFEST_FILE)
    }
}

fn validated_absolute_path(value: &str) -> Result<PathBuf, &'static str> {
    let path = PathBuf::from(value);
    if !path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::CurDir | std::path::Component::ParentDir
            )
        })
    {
        return Err("qiongli-cli-product-authority-invalid");
    }
    Ok(path)
}

fn receipt_matches_authority(
    receipt: &CliInstallReceiptV1,
    available: &CliProductAuthorityBinding,
) -> bool {
    receipt.schema_version == CLI_RECEIPT_SCHEMA_VERSION
        && receipt.packaged_authority.as_ref().is_some_and(|recorded| {
            Path::new(&recorded.packaged_executable) == available.packaged_executable.as_path()
                && Path::new(&recorded.desktop_manifest_path)
                    == available.desktop_manifest_path.as_path()
                && recorded.control_sha256 == available.control_sha256
        })
}

pub(crate) fn inspect_cli_install(
    home: Option<&Path>,
    source: Option<&Path>,
    product_version: &str,
    search_path: Option<&OsStr>,
    shell: Option<&OsStr>,
) -> CliInstallInspection {
    let Some(home) = home else {
        return unavailable_inspection(product_version, "qiongli-cli-home-unavailable");
    };
    let target = cli_target(home);
    let Some(source) = source else {
        return unavailable_inspection_with_target(
            product_version,
            target,
            "qiongli-cli-bundle-unavailable",
        );
    };
    let source_sha256 = match regular_file_sha256(source) {
        Ok(sha256) => sha256,
        Err(code) => {
            return unavailable_inspection_with_target(product_version, target, code);
        }
    };
    // Product control is required before any install or replacement plan can be
    // authorized, but it is not required for the read-only shell observation.
    // Keep inspecting an already installed target so source/test builds can
    // diagnose PATH shadowing without gaining mutation authority.
    let available_authority = detect_packaged_authority(source).ok().flatten();
    let receipt_path = cli_receipt_path(home);
    let receipt = read_receipt(&receipt_path).ok().flatten();
    let target_observation = match observe_target(&target) {
        Ok(observation) => observation,
        Err(code) => {
            return CliInstallInspection {
                state: CliInstallState::Conflict,
                installed_version: None,
                available_version: product_version.to_owned(),
                path_state: observe_path_state(home, &target, search_path, shell),
                target,
                reason_code: code,
                can_install: false,
                can_test: false,
            };
        }
    };
    let path_state = observe_path_state(home, &target, search_path, shell);
    let receipt_version = match (&target_observation, receipt.as_ref()) {
        (TargetObservation::RegularFile(target_sha256), Some(receipt))
            if receipt.installed_sha256 == *target_sha256 =>
        {
            Some(receipt.product_version.clone())
        }
        _ => None,
    };
    let authority_upgrade_required = available_authority.as_ref().is_some_and(|available| {
        receipt
            .as_ref()
            .is_none_or(|receipt| !receipt_matches_authority(receipt, available))
    });
    match target_observation {
        TargetObservation::Missing => CliInstallInspection {
            state: CliInstallState::Missing,
            installed_version: None,
            available_version: product_version.to_owned(),
            target,
            path_state,
            reason_code: "qiongli-cli-not-installed",
            can_install: true,
            can_test: false,
        },
        TargetObservation::RegularFile(ref target_sha256)
            if *target_sha256 == source_sha256 && authority_upgrade_required =>
        {
            CliInstallInspection {
                state: CliInstallState::UpdateAvailable,
                installed_version: receipt_version,
                available_version: product_version.to_owned(),
                target,
                path_state,
                reason_code: "qiongli-cli-authority-upgrade-available",
                can_install: true,
                can_test: true,
            }
        }
        TargetObservation::RegularFile(ref target_sha256) if *target_sha256 == source_sha256 => {
            CliInstallInspection {
                state: CliInstallState::InstalledCurrent,
                installed_version: Some(
                    receipt
                        .filter(|receipt| receipt.installed_sha256 == *target_sha256)
                        .map_or_else(
                            || product_version.to_owned(),
                            |receipt| receipt.product_version,
                        ),
                ),
                available_version: product_version.to_owned(),
                target,
                path_state,
                reason_code: if path_state == CliPathState::Active {
                    "qiongli-cli-installed-current"
                } else if path_state == CliPathState::Configured {
                    "qiongli-cli-installed-shell-configured"
                } else {
                    "qiongli-cli-installed-path-attention"
                },
                can_install: false,
                can_test: true,
            }
        }
        TargetObservation::Unsupported => CliInstallInspection {
            state: CliInstallState::Conflict,
            installed_version: None,
            available_version: product_version.to_owned(),
            target,
            path_state,
            reason_code: "qiongli-cli-target-type-unsupported",
            can_install: false,
            can_test: false,
        },
        TargetObservation::RegularFile(_) => CliInstallInspection {
            state: CliInstallState::UpdateAvailable,
            installed_version: receipt_version,
            available_version: product_version.to_owned(),
            target,
            path_state,
            reason_code: "qiongli-cli-replacement-available",
            can_install: true,
            can_test: true,
        },
        TargetObservation::Symlink(_) => CliInstallInspection {
            state: CliInstallState::Conflict,
            installed_version: receipt_version,
            available_version: product_version.to_owned(),
            target,
            path_state,
            reason_code: "qiongli-cli-target-symlink-unsupported",
            can_install: false,
            can_test: false,
        },
    }
}

pub(crate) fn preview_cli_install(
    home: &Path,
    source: &Path,
    product_version: &str,
) -> Result<CliInstallPlan, &'static str> {
    validate_install_roots(home)?;
    let source_sha256 = regular_file_sha256(source)?;
    let target = cli_target(home);
    validate_target_ancestors(home, &target)?;
    let expected_target = observe_target(&target)?;
    if matches!(
        expected_target,
        TargetObservation::Unsupported | TargetObservation::Symlink(_)
    ) {
        return Err("qiongli-cli-target-type-unsupported");
    }
    let receipt_path = cli_receipt_path(home);
    let observed_receipt = read_receipt_with_digest(&receipt_path)?;
    let previous_receipt_sha256 = observed_receipt.as_ref().map(|(_, digest)| digest.clone());
    let previous_receipt = observed_receipt.map(|(receipt, _)| receipt);
    let previous_managed =
        previous_receipt
            .as_ref()
            .is_some_and(|receipt| match &expected_target {
                TargetObservation::RegularFile(sha256) => receipt.installed_sha256 == *sha256,
                TargetObservation::Missing
                | TargetObservation::Symlink(_)
                | TargetObservation::Unsupported => false,
            });
    let (retained_backup_name, retained_backup_sha256) = if previous_managed {
        retained_backup_binding(home, previous_receipt.as_ref().expect("managed receipt"))?
    } else {
        (None, None)
    };
    let target_name = target
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or("qiongli-cli-target-invalid")?;
    let packaged_authority = detect_packaged_authority(source)?;
    let packaged_executable = packaged_authority
        .as_ref()
        .map(|authority| {
            authority
                .packaged_executable
                .to_str()
                .ok_or("qiongli-cli-product-authority-invalid")
        })
        .transpose()?;
    let desktop_manifest_path = packaged_authority
        .as_ref()
        .map(|authority| {
            authority
                .desktop_manifest_path
                .to_str()
                .ok_or("qiongli-cli-product-authority-invalid")
        })
        .transpose()?;
    let digest_payload = CliInstallPlanDigest {
        schema_version: CLI_RECEIPT_SCHEMA_VERSION,
        product_version,
        source_sha256: &source_sha256,
        target_name,
        expected_target: expected_target.fingerprint(),
        previous_managed,
        previous_receipt_sha256: previous_receipt_sha256.as_deref(),
        retained_backup_name: retained_backup_name.as_deref(),
        retained_backup_sha256: retained_backup_sha256.as_deref(),
        packaged_executable,
        desktop_manifest_path,
        control_sha256: packaged_authority
            .as_ref()
            .map(|authority| authority.control_sha256.as_str()),
    };
    let encoded =
        serde_json::to_vec(&digest_payload).map_err(|_| "qiongli-cli-plan-serialization-failed")?;
    let plan_sha256 = sha256_bytes(&encoded);
    Ok(CliInstallPlan {
        home: home.to_path_buf(),
        source: source.to_path_buf(),
        target,
        receipt_path,
        product_version: product_version.to_owned(),
        source_sha256,
        expected_target,
        previous_managed,
        previous_receipt_sha256,
        retained_backup_name,
        retained_backup_sha256,
        packaged_authority,
        plan_sha256,
    })
}

pub(crate) fn apply_cli_install(plan: &CliInstallPlan) -> Result<&'static str, &'static str> {
    verify_cli_install_plan(plan)?;

    let bin_dir = plan.target.parent().ok_or("qiongli-cli-target-invalid")?;
    create_private_directory_chain(&plan.home, bin_dir)?;
    let backup_path = if plan.expected_target == TargetObservation::Missing {
        None
    } else {
        let backup_root = plan
            .receipt_path
            .parent()
            .ok_or("qiongli-cli-receipt-path-invalid")?
            .join("backups");
        create_private_directory_chain(&plan.home, &backup_root)?;
        let suffix = &plan.plan_sha256[..12];
        let backup = backup_root.join(format!("preinstall-{suffix}-qiongli"));
        if fs::symlink_metadata(&backup).is_ok() {
            return Err("qiongli-cli-backup-conflict");
        }
        fs::rename(&plan.target, &backup).map_err(|_| "qiongli-cli-backup-failed")?;
        Some(backup)
    };

    let temporary = bin_dir.join(format!(".qiongli-install-{}", &plan.plan_sha256[..12]));
    if fs::symlink_metadata(&temporary).is_ok() {
        restore_previous_target(&plan.target, backup_path.as_deref());
        return Err("qiongli-cli-temporary-conflict");
    }
    if let Err(code) = copy_executable(&plan.source, &temporary).and_then(|()| {
        fs::rename(&temporary, &plan.target).map_err(|_| "qiongli-cli-commit-failed")
    }) {
        let _ = fs::remove_file(&temporary);
        restore_previous_target(&plan.target, backup_path.as_deref());
        return Err(code);
    }
    if regular_file_sha256(&plan.target).ok().as_deref() != Some(&plan.source_sha256) {
        let _ = fs::remove_file(&plan.target);
        restore_previous_target(&plan.target, backup_path.as_deref());
        return Err("qiongli-cli-install-verification-failed");
    }

    let receipt = install_receipt(plan, backup_path.as_deref())?;
    if let Err(code) = write_receipt(&plan.home, &plan.receipt_path, &receipt, &plan.plan_sha256) {
        let _ = fs::remove_file(&plan.target);
        restore_previous_target(&plan.target, backup_path.as_deref());
        return Err(code);
    }
    if plan.previous_managed
        && let Some(backup) = backup_path
    {
        let _ = fs::remove_file(backup);
    }
    Ok(if plan.expected_target == TargetObservation::Missing {
        "qiongli-cli-installed"
    } else {
        "qiongli-cli-updated"
    })
}

pub(crate) fn verify_cli_install_plan(plan: &CliInstallPlan) -> Result<(), &'static str> {
    validate_install_roots(&plan.home)?;
    validate_target_ancestors(&plan.home, &plan.target)?;
    if regular_file_sha256(&plan.source)? != plan.source_sha256 {
        return Err("qiongli-cli-bundle-changed");
    }
    if detect_packaged_authority(&plan.source)? != plan.packaged_authority {
        return Err("qiongli-cli-product-authority-changed");
    }
    if observe_target(&plan.target)? != plan.expected_target {
        return Err("qiongli-cli-target-changed");
    }

    let observed_receipt = read_receipt_with_digest(&plan.receipt_path)?;
    if observed_receipt.as_ref().map(|(_, digest)| digest.as_str())
        != plan.previous_receipt_sha256.as_deref()
    {
        return Err("qiongli-cli-receipt-changed");
    }
    if plan.previous_managed {
        let receipt = &observed_receipt
            .as_ref()
            .ok_or("qiongli-cli-receipt-changed")?
            .0;
        let retained = retained_backup_binding(&plan.home, receipt)?;
        if retained.0 != plan.retained_backup_name || retained.1 != plan.retained_backup_sha256 {
            return Err("qiongli-cli-backup-changed");
        }
    }

    Ok(())
}

fn install_receipt(
    plan: &CliInstallPlan,
    backup_path: Option<&Path>,
) -> Result<CliInstallReceiptV1, &'static str> {
    let (retained_backup_name, retained_backup_sha256) = if plan.previous_managed {
        (
            plan.retained_backup_name.clone(),
            plan.retained_backup_sha256.clone(),
        )
    } else {
        let name = backup_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(OsStr::to_str)
            .map(str::to_owned);
        let sha256 = match &plan.expected_target {
            TargetObservation::RegularFile(sha256) => Some(sha256.clone()),
            TargetObservation::Missing
            | TargetObservation::Symlink(_)
            | TargetObservation::Unsupported => None,
        };
        (name, sha256)
    };
    Ok(CliInstallReceiptV1 {
        schema_version: CLI_RECEIPT_SCHEMA_VERSION,
        product_version: plan.product_version.clone(),
        installed_sha256: plan.source_sha256.clone(),
        target_name: plan
            .target
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or("qiongli-cli-target-invalid")?
            .to_owned(),
        retained_backup_name,
        retained_backup_sha256,
        packaged_authority: plan.packaged_authority.as_ref().map(|authority| {
            CliProductAuthorityReceiptV1 {
                packaged_executable: authority.packaged_executable.to_string_lossy().into_owned(),
                desktop_manifest_path: authority
                    .desktop_manifest_path
                    .to_string_lossy()
                    .into_owned(),
                control_sha256: authority.control_sha256.clone(),
            }
        }),
    })
}

pub(crate) fn stage_cli_update(
    plan: &CliInstallPlan,
    staged_binary: &Path,
    staged_receipt: &Path,
) -> Result<(), &'static str> {
    verify_cli_install_plan(plan)?;
    if !plan.previous_managed {
        return Err("qiongli-cli-not-managed");
    }
    for path in [staged_binary, staged_receipt] {
        match fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err("qiongli-cli-temporary-conflict"),
        }
    }
    copy_executable(&plan.source, staged_binary)?;
    if regular_file_sha256(staged_binary)? != plan.source_sha256 {
        return Err("qiongli-cli-install-verification-failed");
    }
    write_receipt(
        &plan.home,
        staged_receipt,
        &install_receipt(plan, None)?,
        &plan.plan_sha256,
    )
}

pub(crate) fn preview_cli_remove(
    home: &Path,
    search_path: Option<&OsStr>,
    shell: Option<&OsStr>,
) -> Result<CliRemovalPlan, &'static str> {
    validate_install_roots(home)?;
    let target = cli_target(home);
    validate_target_ancestors(home, &target)?;
    let receipt_path = cli_receipt_path(home);
    let receipt = read_receipt(&receipt_path)?.ok_or("qiongli-cli-not-managed")?;
    let installed_sha256 = match observe_target(&target)? {
        TargetObservation::RegularFile(sha256) if sha256 == receipt.installed_sha256 => sha256,
        TargetObservation::Missing => return Err("qiongli-cli-managed-target-missing"),
        TargetObservation::RegularFile(_) => return Err("qiongli-cli-managed-target-drifted"),
        TargetObservation::Symlink(_) => return Err("qiongli-cli-managed-target-symlinked"),
        TargetObservation::Unsupported => return Err("qiongli-cli-target-type-unsupported"),
    };
    let receipt_sha256 = regular_file_sha256(&receipt_path)?;
    let (retained_backup_name, retained_backup_sha256) = retained_backup_binding(home, &receipt)?;
    let expected_path_state = observe_path_state(home, &target, search_path, shell);
    if expected_path_state == CliPathState::Shadowed {
        return Err("qiongli-cli-shadowed-removal-refused");
    }
    let target_name = target
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or("qiongli-cli-target-invalid")?;
    let encoded = serde_json::to_vec(&CliRemovalPlanDigest {
        schema_version: CLI_RECEIPT_SCHEMA_VERSION,
        target_name,
        receipt_sha256: &receipt_sha256,
        installed_sha256: &installed_sha256,
        retained_backup_name: retained_backup_name.as_deref(),
        retained_backup_sha256: retained_backup_sha256.as_deref(),
        path_state: cli_path_state_id(expected_path_state),
    })
    .map_err(|_| "qiongli-cli-plan-serialization-failed")?;
    Ok(CliRemovalPlan {
        home: home.to_path_buf(),
        target,
        receipt_path,
        receipt_sha256,
        installed_sha256,
        retained_backup_name,
        retained_backup_sha256,
        search_path: search_path.map(OsStr::to_os_string),
        shell: shell.map(OsStr::to_os_string),
        expected_path_state,
        plan_sha256: sha256_bytes(&encoded),
    })
}

pub(crate) fn apply_cli_remove(plan: &CliRemovalPlan) -> Result<CliRemovalEffect, &'static str> {
    validate_install_roots(&plan.home)?;
    validate_target_ancestors(&plan.home, &plan.target)?;
    if regular_file_sha256(&plan.receipt_path)? != plan.receipt_sha256 {
        return Err("qiongli-cli-receipt-changed");
    }
    let receipt = read_receipt(&plan.receipt_path)?.ok_or("qiongli-cli-not-managed")?;
    if receipt.installed_sha256 != plan.installed_sha256
        || regular_file_sha256(&plan.target)? != plan.installed_sha256
    {
        return Err("qiongli-cli-managed-target-drifted");
    }
    let path_state = observe_path_state(
        &plan.home,
        &plan.target,
        plan.search_path.as_deref(),
        plan.shell.as_deref(),
    );
    if path_state == CliPathState::Shadowed {
        return Err("qiongli-cli-shadowed-removal-refused");
    }
    if path_state != plan.expected_path_state {
        return Err("qiongli-cli-removal-precondition-changed");
    }
    let backup_path = plan
        .retained_backup_name
        .as_deref()
        .map(|name| plan.home.join(".qiongli/v2/cli/backups").join(name));
    if let (Some(path), Some(expected)) = (
        backup_path.as_deref(),
        plan.retained_backup_sha256.as_deref(),
    ) && regular_file_sha256(path)? != expected
    {
        return Err("qiongli-cli-backup-changed");
    }
    if backup_path.is_some() != plan.retained_backup_sha256.is_some() {
        return Err("qiongli-cli-removal-plan-invalid");
    }

    let bin_dir = plan.target.parent().ok_or("qiongli-cli-target-invalid")?;
    let staged = bin_dir.join(format!(".qiongli-remove-{}", &plan.plan_sha256[..12]));
    if fs::symlink_metadata(&staged).is_ok() {
        return Err("qiongli-cli-temporary-conflict");
    }
    fs::rename(&plan.target, &staged).map_err(|_| "qiongli-cli-removal-stage-failed")?;
    if let Some(backup) = backup_path.as_deref()
        && fs::rename(backup, &plan.target).is_err()
    {
        let _ = fs::rename(&staged, &plan.target);
        return Err("qiongli-cli-restore-failed");
    }
    if fs::remove_file(&plan.receipt_path).is_err() {
        if let Some(backup) = backup_path.as_deref() {
            let _ = fs::rename(&plan.target, backup);
        }
        let _ = fs::rename(&staged, &plan.target);
        return Err("qiongli-cli-receipt-remove-failed");
    }
    fs::remove_file(&staged).map_err(|_| "qiongli-cli-removal-cleanup-failed")?;
    Ok(if backup_path.is_some() {
        CliRemovalEffect::RestoredPredecessor
    } else {
        CliRemovalEffect::Removed
    })
}

pub(crate) fn preview_cli_path_configure(
    home: &Path,
    shell: Option<&OsStr>,
) -> Result<CliPathConfigurePlan, &'static str> {
    validate_install_roots(home)?;
    let target = cli_target(home);
    validate_target_ancestors(home, &target)?;
    let install_receipt_path = cli_receipt_path(home);
    let install_receipt = read_receipt(&install_receipt_path)?.ok_or("qiongli-cli-not-managed")?;
    let installed_sha256 = match observe_target(&target)? {
        TargetObservation::RegularFile(sha256) if sha256 == install_receipt.installed_sha256 => {
            sha256
        }
        TargetObservation::Missing => return Err("qiongli-cli-managed-target-missing"),
        TargetObservation::RegularFile(_) => return Err("qiongli-cli-managed-target-drifted"),
        TargetObservation::Symlink(_) => return Err("qiongli-cli-managed-target-symlinked"),
        TargetObservation::Unsupported => return Err("qiongli-cli-target-type-unsupported"),
    };
    let profile_name = supported_profile_name(shell)?;
    let profile_path = home.join(profile_name);
    let current = read_profile_for_update(home, &profile_path)?;
    let current_bytes = current
        .as_ref()
        .map_or(&[][..], |(bytes, _)| bytes.as_slice());
    if String::from_utf8_lossy(current_bytes).contains(CLI_PATH_MARKER) {
        return Err("qiongli-cli-path-already-configured");
    }
    let mut next_profile_bytes = current_bytes.to_vec();
    if !next_profile_bytes.is_empty() && !next_profile_bytes.ends_with(b"\n") {
        next_profile_bytes.push(b'\n');
    }
    if !next_profile_bytes.is_empty() {
        next_profile_bytes.push(b'\n');
    }
    next_profile_bytes.extend_from_slice(CLI_PATH_MARKER.as_bytes());
    if next_profile_bytes.len() as u64 > MAX_SHELL_PROFILE_BYTES {
        return Err("qiongli-cli-shell-profile-too-large");
    }
    let previous_profile_bytes = current.as_ref().map(|(bytes, _)| bytes.clone());
    let expected_profile_sha256 = current.map(|(_, sha256)| sha256);
    let next_profile_sha256 = sha256_bytes(&next_profile_bytes);
    let install_receipt_sha256 = regular_file_sha256(&install_receipt_path)?;
    let encoded = serde_json::to_vec(&CliPathConfigurePlanDigest {
        schema_version: CLI_PATH_RECEIPT_SCHEMA_VERSION,
        installed_sha256: &installed_sha256,
        install_receipt_sha256: &install_receipt_sha256,
        profile_name,
        expected_profile_sha256: expected_profile_sha256.as_deref(),
        next_profile_sha256: &next_profile_sha256,
    })
    .map_err(|_| "qiongli-cli-plan-serialization-failed")?;
    Ok(CliPathConfigurePlan {
        home: home.to_path_buf(),
        target,
        install_receipt_sha256,
        installed_sha256,
        profile_name: profile_name.to_owned(),
        profile_path,
        previous_profile_bytes,
        expected_profile_sha256,
        next_profile_bytes,
        next_profile_sha256,
        path_receipt_path: cli_path_receipt_path(home),
        plan_sha256: sha256_bytes(&encoded),
    })
}

pub(crate) fn apply_cli_path_configure(
    plan: &CliPathConfigurePlan,
) -> Result<&'static str, &'static str> {
    validate_install_roots(&plan.home)?;
    validate_target_ancestors(&plan.home, &plan.target)?;
    if regular_file_sha256(&cli_receipt_path(&plan.home))? != plan.install_receipt_sha256
        || regular_file_sha256(&plan.target)? != plan.installed_sha256
    {
        return Err("qiongli-cli-path-precondition-changed");
    }
    let current = read_profile_for_update(&plan.home, &plan.profile_path)?;
    if current.as_ref().map(|(_, sha256)| sha256) != plan.expected_profile_sha256.as_ref() {
        return Err("qiongli-cli-shell-profile-changed");
    }
    if sha256_bytes(&plan.next_profile_bytes) != plan.next_profile_sha256 {
        return Err("qiongli-cli-path-plan-invalid");
    }
    write_profile_atomically(
        &plan.home,
        &plan.profile_path,
        &plan.next_profile_bytes,
        &plan.plan_sha256,
    )?;
    let receipt = CliPathReceiptV1 {
        schema_version: CLI_PATH_RECEIPT_SCHEMA_VERSION,
        installed_sha256: plan.installed_sha256.clone(),
        profile_name: plan.profile_name.clone(),
        profile_before_sha256: plan.expected_profile_sha256.clone(),
        profile_after_sha256: plan.next_profile_sha256.clone(),
    };
    if let Err(code) = write_path_receipt(
        &plan.home,
        &plan.path_receipt_path,
        &receipt,
        &plan.plan_sha256,
    ) {
        rollback_profile(
            &plan.home,
            &plan.profile_path,
            plan.previous_profile_bytes.as_deref(),
            &plan.plan_sha256,
        );
        return Err(code);
    }
    Ok("qiongli-cli-path-configured")
}

fn unavailable_inspection(
    product_version: &str,
    reason_code: &'static str,
) -> CliInstallInspection {
    unavailable_inspection_with_target(
        product_version,
        PathBuf::from("<user-home>/.local/bin/qiongli"),
        reason_code,
    )
}

fn unavailable_inspection_with_target(
    product_version: &str,
    target: PathBuf,
    reason_code: &'static str,
) -> CliInstallInspection {
    CliInstallInspection {
        state: CliInstallState::Unavailable,
        installed_version: None,
        available_version: product_version.to_owned(),
        target,
        path_state: CliPathState::NotObservable,
        reason_code,
        can_install: false,
        can_test: false,
    }
}

pub(crate) fn cli_target(home: &Path) -> PathBuf {
    if cfg!(windows) {
        home.join("AppData/Local/Qiongli/bin/qiongli.exe")
    } else {
        home.join(".local/bin/qiongli")
    }
}

fn cli_receipt_path(home: &Path) -> PathBuf {
    home.join(".qiongli/v2/cli/install-receipt.json")
}

fn cli_path_receipt_path(home: &Path) -> PathBuf {
    home.join(".qiongli/v2/cli/path-receipt.json")
}

fn supported_profile_name(shell: Option<&OsStr>) -> Result<&'static str, &'static str> {
    match shell
        .and_then(|value| Path::new(value).file_name())
        .and_then(OsStr::to_str)
    {
        Some("zsh") => Ok(".zprofile"),
        Some("bash") => Ok(".bash_profile"),
        None if cfg!(target_os = "macos") => Ok(".zprofile"),
        None => Ok(".profile"),
        _ => Err("qiongli-cli-shell-unsupported"),
    }
}

fn read_profile_for_update(
    home: &Path,
    path: &Path,
) -> Result<Option<(Vec<u8>, String)>, &'static str> {
    if path.parent() != Some(home) {
        return Err("qiongli-cli-shell-profile-invalid");
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("qiongli-cli-shell-profile-unavailable"),
    };
    if metadata.file_type().is_symlink() {
        return Err("qiongli-cli-shell-profile-symlinked");
    }
    if !metadata.is_file() {
        return Err("qiongli-cli-shell-profile-invalid");
    }
    if metadata.len() > MAX_SHELL_PROFILE_BYTES {
        return Err("qiongli-cli-shell-profile-too-large");
    }
    let bytes = fs::read(path).map_err(|_| "qiongli-cli-shell-profile-unavailable")?;
    String::from_utf8(bytes.clone()).map_err(|_| "qiongli-cli-shell-profile-not-utf8")?;
    let sha256 = sha256_bytes(&bytes);
    Ok(Some((bytes, sha256)))
}

fn write_profile_atomically(
    home: &Path,
    path: &Path,
    bytes: &[u8],
    token: &str,
) -> Result<(), &'static str> {
    if path.parent() != Some(home) || bytes.len() as u64 > MAX_SHELL_PROFILE_BYTES {
        return Err("qiongli-cli-shell-profile-invalid");
    }
    let previous_permissions = fs::symlink_metadata(path)
        .ok()
        .map(|metadata| metadata.permissions());
    let temporary = home.join(format!(".qiongli-path-{}", &token[..12]));
    if fs::symlink_metadata(&temporary).is_ok() {
        return Err("qiongli-cli-shell-profile-temporary-conflict");
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|_| "qiongli-cli-shell-profile-write-failed")?;
    if file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        let _ = fs::remove_file(&temporary);
        return Err("qiongli-cli-shell-profile-write-failed");
    }
    if let Some(permissions) = previous_permissions
        && fs::set_permissions(&temporary, permissions).is_err()
    {
        let _ = fs::remove_file(&temporary);
        return Err("qiongli-cli-shell-profile-write-failed");
    }
    fs::rename(&temporary, path).map_err(|_| "qiongli-cli-shell-profile-write-failed")
}

fn write_path_receipt(
    home: &Path,
    path: &Path,
    receipt: &CliPathReceiptV1,
    token: &str,
) -> Result<(), &'static str> {
    if fs::symlink_metadata(path).is_ok() {
        return Err("qiongli-cli-path-receipt-conflict");
    }
    let parent = path.parent().ok_or("qiongli-cli-receipt-path-invalid")?;
    create_private_directory_chain(home, parent)?;
    let temporary = parent.join(format!(".path-receipt-{}.json", &token[..12]));
    let bytes =
        serde_json::to_vec_pretty(receipt).map_err(|_| "qiongli-cli-path-receipt-write-failed")?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|_| "qiongli-cli-path-receipt-write-failed")?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "qiongli-cli-path-receipt-write-failed")?;
    fs::rename(&temporary, path).map_err(|_| "qiongli-cli-path-receipt-write-failed")
}

fn rollback_profile(home: &Path, path: &Path, previous: Option<&[u8]>, token: &str) {
    if let Some(bytes) = previous {
        let _ = write_profile_atomically(home, path, bytes, token);
    } else {
        let _ = fs::remove_file(path);
    }
}

fn retained_backup_binding(
    home: &Path,
    receipt: &CliInstallReceiptV1,
) -> Result<(Option<String>, Option<String>), &'static str> {
    let Some(name) = receipt.retained_backup_name.as_deref() else {
        if receipt.retained_backup_sha256.is_some() {
            return Err("qiongli-cli-receipt-invalid");
        }
        return Ok((None, None));
    };
    if !valid_backup_name(name) {
        return Err("qiongli-cli-receipt-invalid");
    }
    let path = home.join(".qiongli/v2/cli/backups").join(name);
    let observed = regular_file_sha256(&path).map_err(|_| "qiongli-cli-backup-changed")?;
    if receipt
        .retained_backup_sha256
        .as_deref()
        .is_some_and(|expected| expected != observed)
    {
        return Err("qiongli-cli-backup-changed");
    }
    Ok((Some(name.to_owned()), Some(observed)))
}

fn valid_backup_name(name: &str) -> bool {
    name.starts_with("preinstall-")
        && name.ends_with("-qiongli")
        && name.len() <= 128
        && Path::new(name).components().count() == 1
}

fn validate_install_roots(home: &Path) -> Result<(), &'static str> {
    if !home.is_absolute() {
        return Err("qiongli-cli-home-invalid");
    }
    let metadata = fs::symlink_metadata(home).map_err(|_| "qiongli-cli-home-unavailable")?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err("qiongli-cli-home-unsafe");
    }
    Ok(())
}

fn validate_target_ancestors(home: &Path, target: &Path) -> Result<(), &'static str> {
    if !target.starts_with(home) {
        return Err("qiongli-cli-target-invalid");
    }
    let mut current = target.parent();
    while let Some(path) = current {
        if path == home {
            break;
        }
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("qiongli-cli-target-ancestor-symlink");
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err("qiongli-cli-target-ancestor-invalid");
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("qiongli-cli-target-unavailable"),
        }
        current = path.parent();
    }
    Ok(())
}

fn create_private_directory_chain(home: &Path, target: &Path) -> Result<(), &'static str> {
    if !target.starts_with(home) {
        return Err("qiongli-cli-directory-invalid");
    }
    let relative = target
        .strip_prefix(home)
        .map_err(|_| "qiongli-cli-directory-invalid")?;
    let mut current = home.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err("qiongli-cli-directory-unsafe");
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                create_private_directory(&current)?;
            }
            Err(_) => return Err("qiongli-cli-directory-unavailable"),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt;

    fs::DirBuilder::new()
        .mode(0o700)
        .create(path)
        .map_err(|_| "qiongli-cli-directory-create-failed")
}

#[cfg(not(unix))]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    fs::create_dir(path).map_err(|_| "qiongli-cli-directory-create-failed")
}

fn observe_target(path: &Path) -> Result<TargetObservation, &'static str> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let target = fs::read_link(path).map_err(|_| "qiongli-cli-target-unreadable")?;
            Ok(TargetObservation::Symlink(sha256_bytes(
                target.as_os_str().as_encoded_bytes(),
            )))
        }
        Ok(metadata) if metadata.is_file() => {
            Ok(TargetObservation::RegularFile(regular_file_sha256(path)?))
        }
        Ok(_) => Ok(TargetObservation::Unsupported),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(TargetObservation::Missing)
        }
        Err(_) => Err("qiongli-cli-target-unavailable"),
    }
}

pub(crate) fn regular_file_sha256(path: &Path) -> Result<String, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "qiongli-cli-file-unavailable")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("qiongli-cli-file-invalid");
    }
    let mut file = File::open(path).map_err(|_| "qiongli-cli-file-unavailable")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| "qiongli-cli-file-unavailable")?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn read_receipt(path: &Path) -> Result<Option<CliInstallReceiptV1>, &'static str> {
    Ok(read_receipt_with_digest(path)?.map(|(receipt, _)| receipt))
}

fn read_receipt_with_digest(
    path: &Path,
) -> Result<Option<(CliInstallReceiptV1, String)>, &'static str> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("qiongli-cli-receipt-unavailable"),
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_RECEIPT_BYTES
    {
        return Err("qiongli-cli-receipt-invalid");
    }
    let bytes = fs::read(path).map_err(|_| "qiongli-cli-receipt-unavailable")?;
    let receipt: CliInstallReceiptV1 =
        serde_json::from_slice(&bytes).map_err(|_| "qiongli-cli-receipt-invalid")?;
    if !matches!(
        receipt.schema_version,
        LEGACY_CLI_RECEIPT_SCHEMA_VERSION
            | AUTHORITY_CLI_RECEIPT_SCHEMA_VERSION
            | CLI_RECEIPT_SCHEMA_VERSION
    ) || receipt.product_version.is_empty()
        || receipt.product_version.len() > 128
        || receipt.installed_sha256.len() != 64
        || !receipt
            .installed_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || receipt.target_name
            != if cfg!(windows) {
                "qiongli.exe"
            } else {
                "qiongli"
            }
        || receipt.schema_version == LEGACY_CLI_RECEIPT_SCHEMA_VERSION
            && receipt.packaged_authority.is_some()
        || receipt.schema_version < CLI_RECEIPT_SCHEMA_VERSION
            && receipt.retained_backup_sha256.is_some()
        || receipt.schema_version == CLI_RECEIPT_SCHEMA_VERSION
            && (receipt.retained_backup_name.is_some() != receipt.retained_backup_sha256.is_some())
        || receipt
            .retained_backup_name
            .as_deref()
            .is_some_and(|name| !valid_backup_name(name))
        || receipt
            .retained_backup_sha256
            .as_deref()
            .is_some_and(|sha256| !valid_sha256(sha256))
        || receipt
            .packaged_authority
            .as_ref()
            .is_some_and(|authority| {
                validated_absolute_path(&authority.packaged_executable).is_err()
                    || validated_absolute_path(&authority.desktop_manifest_path).is_err()
                    || authority.control_sha256.len() != 64
                    || !authority
                        .control_sha256
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
    {
        return Err("qiongli-cli-receipt-invalid");
    }
    Ok(Some((receipt, sha256_bytes(&bytes))))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn write_receipt(
    home: &Path,
    path: &Path,
    receipt: &CliInstallReceiptV1,
    token: &str,
) -> Result<(), &'static str> {
    let parent = path.parent().ok_or("qiongli-cli-receipt-path-invalid")?;
    create_private_directory_chain(home, parent)?;
    let temporary = parent.join(format!(".install-receipt-{}.json", &token[..12]));
    let bytes =
        serde_json::to_vec_pretty(receipt).map_err(|_| "qiongli-cli-receipt-write-failed")?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|_| "qiongli-cli-receipt-write-failed")?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "qiongli-cli-receipt-write-failed")?;
    fs::rename(&temporary, path).map_err(|_| "qiongli-cli-receipt-write-failed")
}

fn copy_executable(source: &Path, target: &Path) -> Result<(), &'static str> {
    fs::copy(source, target).map_err(|_| "qiongli-cli-copy-failed")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(target, fs::Permissions::from_mode(0o755))
            .map_err(|_| "qiongli-cli-permissions-failed")?;
    }
    OpenOptions::new()
        .write(true)
        .open(target)
        .and_then(|file| file.sync_all())
        .map_err(|_| "qiongli-cli-copy-failed")
}

fn restore_previous_target(target: &Path, backup: Option<&Path>) {
    if let Some(backup) = backup {
        let _ = fs::rename(backup, target);
    }
}

fn observe_path_state(
    home: &Path,
    target: &Path,
    search_path: Option<&OsStr>,
    shell: Option<&OsStr>,
) -> CliPathState {
    let process_state = observe_process_path_state(target, search_path);
    if matches!(process_state, CliPathState::Active | CliPathState::Shadowed) {
        return process_state;
    }
    observe_shell_profile_path_state(home, shell).unwrap_or(process_state)
}

const fn cli_path_state_id(state: CliPathState) -> &'static str {
    match state {
        CliPathState::Active => "active",
        CliPathState::Configured => "configured",
        CliPathState::NotConfigured => "not-configured",
        CliPathState::Shadowed => "shadowed",
        CliPathState::VersionMismatch => "version-mismatch",
        CliPathState::NotObservable => "not-observable",
    }
}

fn observe_process_path_state(target: &Path, search_path: Option<&OsStr>) -> CliPathState {
    let Some(search_path) = search_path else {
        return CliPathState::NotObservable;
    };
    let Some(target_parent) = target.parent() else {
        return CliPathState::NotObservable;
    };
    let directories = std::env::split_paths(search_path).collect::<Vec<_>>();
    if !directories.iter().any(|path| path == target_parent) {
        return CliPathState::NotConfigured;
    }
    for directory in directories {
        let candidate = directory.join(if cfg!(windows) {
            "qiongli.exe"
        } else {
            "qiongli"
        });
        if is_executable_file(&candidate) {
            return if candidate == target {
                CliPathState::Active
            } else {
                CliPathState::Shadowed
            };
        }
    }
    CliPathState::Active
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShellProfilePathDirective {
    ManagedBin,
    KnownShadow,
}

fn observe_shell_profile_path_state(home: &Path, shell: Option<&OsStr>) -> Option<CliPathState> {
    let shell_name = shell
        .and_then(|value| Path::new(value).file_name())
        .and_then(OsStr::to_str);
    let profile_names: &[&str] = match shell_name {
        Some("zsh") => &[".zshenv", ".zprofile", ".zshrc"],
        Some("bash") => &[".bash_profile", ".profile", ".bashrc"],
        // Finder-launched macOS Apps commonly do not inherit SHELL even though
        // the user's Terminal starts zsh. Inspect the supported login and
        // interactive profiles in their real startup order instead of
        // incorrectly reporting an existing .zshrc entry as missing.
        _ if cfg!(target_os = "macos") => &[
            ".zshenv",
            ".zprofile",
            ".zshrc",
            ".bash_profile",
            ".profile",
            ".bashrc",
        ],
        _ => &[
            ".profile",
            ".bash_profile",
            ".bashrc",
            ".zshenv",
            ".zprofile",
            ".zshrc",
        ],
    };
    let known_shadow_present = [
        home.join(".local/share/mise/shims/qiongli"),
        home.join(".pyenv/shims/qiongli"),
        home.join(".local/share/pnpm/qiongli"),
        home.join("Library/pnpm/qiongli"),
        home.join(".cargo/bin/qiongli"),
        home.join(".npm-global/bin/qiongli"),
        home.join(".bun/bin/qiongli"),
    ]
    .iter()
    .any(|candidate| is_executable_file(candidate));
    let mut managed_seen = false;
    let mut latest = None;
    for name in profile_names {
        let Some(contents) = read_shell_profile(home, &home.join(name)) else {
            continue;
        };
        for line in contents.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if prepends_path(line, ".local/bin") && !line.contains(".local/share/") {
                managed_seen = true;
                latest = Some(ShellProfilePathDirective::ManagedBin);
            } else if known_shadow_present
                && ([
                    ".local/share/mise/shims",
                    ".pyenv/shims",
                    ".local/share/pnpm",
                    "Library/pnpm",
                    ".cargo/bin",
                    ".npm-global/bin",
                    ".bun/bin",
                ]
                .iter()
                .any(|component| prepends_path(line, component))
                    || (line.contains("mise") && line.contains("activate")))
            {
                latest = Some(ShellProfilePathDirective::KnownShadow);
            }
        }
    }
    match (managed_seen, latest) {
        (true, Some(ShellProfilePathDirective::ManagedBin)) => Some(CliPathState::Configured),
        (true, Some(ShellProfilePathDirective::KnownShadow)) => Some(CliPathState::Shadowed),
        _ => None,
    }
}

fn prepends_path(line: &str, component: &str) -> bool {
    let Some(assignment_offset) =
        path_assignment_offset(line, "PATH").or_else(|| path_assignment_offset(line, "path"))
    else {
        return false;
    };
    let Some(component_offset) = line.find(component) else {
        return false;
    };
    if component_offset < assignment_offset {
        return false;
    }
    let assignment = &line[assignment_offset..];
    let inherited_path_offset = assignment
        .find("$PATH")
        .or_else(|| assignment.find("${PATH}"))
        .or_else(|| assignment.find("$path"))
        .or_else(|| assignment.find("${path"))
        .map(|offset| assignment_offset + offset)
        .unwrap_or(usize::MAX);
    component_offset < inherited_path_offset
}

fn path_assignment_offset(line: &str, variable: &str) -> Option<usize> {
    line.match_indices(variable).find_map(|(offset, _)| {
        let boundary_is_valid = line[..offset]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_');
        let assignment_follows = line[offset + variable.len()..]
            .trim_start()
            .starts_with('=');
        (boundary_is_valid && assignment_follows).then_some(offset)
    })
}

fn read_shell_profile(home: &Path, path: &Path) -> Option<String> {
    let canonical_home = fs::canonicalize(home).ok()?;
    let canonical_path = fs::canonicalize(path).ok()?;
    if !canonical_path.starts_with(canonical_home) {
        return None;
    }
    let metadata = fs::metadata(&canonical_path).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_SHELL_PROFILE_BYTES {
        return None;
    }
    String::from_utf8(fs::read(canonical_path).ok()?).ok()
}

#[cfg(unix)]
pub(crate) fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
pub(crate) fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn native_command_copy_resolves_only_with_matching_receipt_and_payload() {
        let home = test_root("native-command");
        let artifact = qiongli_platform::current_target_native_artifact_identity(
            "2.0.0-alpha.5",
            qiongli_platform::ReleaseChannel::Alpha,
        )
        .unwrap();
        let source = home
            .join(".qiongli/native/payloads")
            .join(qiongli_platform::native_artifact_id(&artifact).unwrap())
            .join(qiongli_platform::native_artifact_binary_path(&artifact).unwrap());
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, b"native test executable").unwrap();
        let plan = preview_cli_install(&home, &source, &artifact.version).unwrap();
        apply_cli_install(&plan).unwrap();
        let installed = cli_target(&home);
        assert_eq!(
            bundled_cli_path_for(&source, Some(&home)),
            Some(source.clone())
        );
        assert_eq!(
            installed_native_cli_source(&home, &installed, &artifact).unwrap(),
            Some(source.clone())
        );
        let receipt_path = cli_receipt_path(&home);
        let current_receipt = fs::read(&receipt_path).unwrap();
        let mut legacy: serde_json::Value = serde_json::from_slice(&current_receipt).unwrap();
        legacy["schema_version"] = serde_json::json!(AUTHORITY_CLI_RECEIPT_SCHEMA_VERSION);
        fs::write(&receipt_path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert_eq!(
            installed_native_cli_source(&home, &installed, &artifact).unwrap(),
            None
        );
        fs::write(&receipt_path, current_receipt).unwrap();
        let copy = home.join("unmanaged-copy");
        fs::copy(&installed, &copy).unwrap();
        assert_eq!(
            installed_native_cli_source(&home, &copy, &artifact).unwrap(),
            None
        );
        fs::write(&installed, b"changed command").unwrap();
        assert!(installed_native_cli_source(&home, &installed, &artifact).is_err());
        fs::copy(&source, &installed).unwrap();
        fs::write(&source, b"changed payload").unwrap();
        assert!(installed_native_cli_source(&home, &installed, &artifact).is_err());
        fs::remove_file(cli_receipt_path(&home)).unwrap();
        assert!(installed_native_cli_source(&home, &installed, &artifact).is_err());
        fs::remove_dir_all(home).unwrap();
    }

    fn test_root(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "qiongli-cli-install-{name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_source(root: &Path, contents: &[u8]) -> PathBuf {
        let source = root.join(if cfg!(windows) {
            "qiongli-cli.exe"
        } else {
            "qiongli-cli"
        });
        fs::write(&source, contents).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        }
        source
    }

    fn write_packaged_source(root: &Path, contents: &[u8]) -> PathBuf {
        let source = if cfg!(target_os = "macos") {
            root.join("Qiongli.app/Contents/MacOS/qiongli-cli")
        } else if cfg!(windows) {
            root.join("Qiongli/qiongli-cli.exe")
        } else {
            root.join("Qiongli/qiongli-cli")
        };
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, contents).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let manifest = packaged_manifest_for_executable(&source);
        fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        fs::write(&manifest, b"bounded desktop manifest").unwrap();
        let control = qiongli_platform::packaged_product_control_path(&manifest).unwrap();
        fs::write(control, b"bounded product control").unwrap();
        source
    }

    #[test]
    fn installs_bundled_cli_and_reports_current_version() {
        let root = test_root("install");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();

        assert_eq!(apply_cli_install(&plan), Ok("qiongli-cli-installed"));
        let target = cli_target(&home);
        let search_path = std::env::join_paths([target.parent().unwrap()]).unwrap();
        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(&search_path),
            None,
        );
        assert_eq!(inspection.state, CliInstallState::InstalledCurrent);
        assert_eq!(
            inspection.installed_version.as_deref(),
            Some("2.0.0-alpha.2")
        );
        assert_eq!(inspection.path_state, CliPathState::Active);
        assert!(inspection.can_test);
        assert!(cli_target_matches_bundled(&home, &source).unwrap());
        assert_eq!(fs::read(target).unwrap(), b"native-cli-v2");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn installed_cli_revalidates_its_receipt_bound_app_authority() {
        let root = test_root("product-authority");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_packaged_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        assert!(plan.packaged_authority.is_some());
        apply_cli_install(&plan).unwrap();

        let installed = home.join(if cfg!(windows) {
            "AppData/Local/Qiongli/bin/qiongli.exe"
        } else {
            ".local/bin/qiongli"
        });
        let binding = installed_cli_product_authority(&home, &installed).unwrap();
        assert_eq!(
            bundled_cli_path_for(&installed, Some(&home)),
            Some(fs::canonicalize(&source).unwrap())
        );
        assert_eq!(
            binding.packaged_executable(),
            fs::canonicalize(&source).unwrap()
        );
        assert_eq!(
            binding.desktop_manifest_path(),
            fs::canonicalize(packaged_manifest_for_executable(&source)).unwrap()
        );

        let control =
            qiongli_platform::packaged_product_control_path(binding.desktop_manifest_path())
                .unwrap();
        fs::write(control, b"tampered product control").unwrap();
        assert_eq!(
            installed_cli_product_authority(&home, &installed).unwrap_err(),
            "qiongli-cli-product-authority-changed"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_current_product_control_keeps_read_only_shell_test_available() {
        let root = test_root("read-only-shell-test");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_packaged_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();

        let manifest = packaged_manifest_for_executable(&source);
        let control = qiongli_platform::packaged_product_control_path(&manifest).unwrap();
        fs::remove_file(control).unwrap();
        let target = cli_target(&home);
        let search_path = std::env::join_paths([target.parent().unwrap()]).unwrap();
        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(&search_path),
            None,
        );

        assert_eq!(inspection.state, CliInstallState::InstalledCurrent);
        assert!(!inspection.can_install);
        assert!(inspection.can_test);
        assert_eq!(inspection.path_state, CliPathState::Active);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_cli_receipt_is_offered_a_no_binary_change_authority_upgrade() {
        let root = test_root("legacy-authority-upgrade");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_packaged_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();
        let receipt_path = cli_receipt_path(&home);
        let mut receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
        receipt["schema_version"] = serde_json::json!(1);
        receipt
            .as_object_mut()
            .unwrap()
            .remove("packaged_authority");
        fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();

        let inspection =
            inspect_cli_install(Some(&home), Some(&source), "2.0.0-alpha.2", None, None);
        assert_eq!(inspection.state, CliInstallState::UpdateAvailable);
        assert_eq!(
            inspection.reason_code,
            "qiongli-cli-authority-upgrade-available"
        );
        assert!(inspection.can_install);
        assert!(inspection.can_test);

        let upgrade = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&upgrade).unwrap();
        installed_cli_product_authority(&home, &cli_target(&home)).unwrap();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn preserves_unmanaged_cli_before_replacement() {
        let root = test_root("backup");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let target = cli_target(&home);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, b"legacy-cli").unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let inspection =
            inspect_cli_install(Some(&home), Some(&source), "2.0.0-alpha.2", None, None);
        assert_eq!(inspection.state, CliInstallState::UpdateAvailable);
        assert!(inspection.can_test);
        assert!(!cli_target_matches_bundled(&home, &source).unwrap());
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();

        assert_eq!(apply_cli_install(&plan), Ok("qiongli-cli-updated"));
        assert_eq!(fs::read(target).unwrap(), b"native-cli-v2");
        let backups = fs::read_dir(home.join(".qiongli/v2/cli/backups"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(backups[0].path()).unwrap(), b"legacy-cli");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn update_refuses_receipt_and_retained_backup_drift_before_writes() {
        let root = test_root("update-receipt-cas");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-old");
        let target = cli_target(&home);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, b"preexisting-cli").unwrap();
        apply_cli_install(&preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap()).unwrap();
        let receipt_path = cli_receipt_path(&home);
        let old_receipt = fs::read(&receipt_path).unwrap();
        let receipt = read_receipt(&receipt_path).unwrap().unwrap();
        let backup = home
            .join(".qiongli/v2/cli/backups")
            .join(receipt.retained_backup_name.as_ref().unwrap());
        fs::write(&source, b"native-cli-new").unwrap();
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.3").unwrap();
        let mut changed = receipt;
        changed.product_version = "2.0.0-alpha.1".into();
        let changed_bytes = serde_json::to_vec(&changed).unwrap();
        fs::write(&receipt_path, &changed_bytes).unwrap();
        assert_eq!(apply_cli_install(&plan), Err("qiongli-cli-receipt-changed"));
        assert_eq!(fs::read(&target).unwrap(), b"native-cli-old");
        assert_eq!(fs::read(&receipt_path).unwrap(), changed_bytes);
        assert_ne!(
            plan.plan_sha256(),
            preview_cli_install(&home, &source, "2.0.0-alpha.3")
                .unwrap()
                .plan_sha256()
        );
        fs::remove_file(&receipt_path).unwrap();
        assert_eq!(apply_cli_install(&plan), Err("qiongli-cli-receipt-changed"));
        assert!(!receipt_path.exists());
        assert_eq!(fs::read(&target).unwrap(), b"native-cli-old");
        let unowned_plan = preview_cli_install(&home, &source, "2.0.0-alpha.3").unwrap();
        fs::write(&receipt_path, &old_receipt).unwrap();
        assert_eq!(
            apply_cli_install(&unowned_plan),
            Err("qiongli-cli-receipt-changed")
        );
        assert_eq!(fs::read(&target).unwrap(), b"native-cli-old");
        assert_eq!(fs::read(&receipt_path).unwrap(), old_receipt);
        fs::write(&backup, b"modified-backup-canary").unwrap();
        assert_eq!(apply_cli_install(&plan), Err("qiongli-cli-backup-changed"));
        assert_eq!(fs::read(&target).unwrap(), b"native-cli-old");
        assert_eq!(fs::read(&receipt_path).unwrap(), old_receipt);
        assert_eq!(fs::read(&backup).unwrap(), b"modified-backup-canary");
        fs::write(&backup, b"preexisting-cli").unwrap();
        assert_eq!(apply_cli_install(&plan), Ok("qiongli-cli-updated"));
        assert_eq!(fs::read(&target).unwrap(), b"native-cli-new");
        assert_eq!(fs::read(&backup).unwrap(), b"preexisting-cli");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_target_changed_after_preview() {
        let root = test_root("drift");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        let target = cli_target(&home);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, b"concurrent-change").unwrap();

        assert_eq!(apply_cli_install(&plan), Err("qiongli-cli-target-changed"));
        assert_eq!(fs::read(target).unwrap(), b"concurrent-change");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reports_shadowing_by_an_earlier_path_entry() {
        let root = test_root("shadowed");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();
        let earlier = root.join("earlier");
        fs::create_dir(&earlier).unwrap();
        let other = earlier.join(if cfg!(windows) {
            "qiongli.exe"
        } else {
            "qiongli"
        });
        fs::write(&other, b"old-cli").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&other, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let managed_bin = cli_target(&home).parent().unwrap().to_path_buf();
        let search_path = std::env::join_paths([earlier, managed_bin]).unwrap();

        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(&search_path),
            None,
        );
        assert_eq!(inspection.state, CliInstallState::InstalledCurrent);
        assert_eq!(inspection.path_state, CliPathState::Shadowed);
        assert_eq!(
            inspection.reason_code,
            "qiongli-cli-installed-path-attention"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn observes_managed_bin_from_zsh_profile_without_using_gui_path() {
        let root = test_root("zsh-profile");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        fs::write(
            home.join(".zshrc"),
            concat!(
                "# research-skills bootstrap path\n",
                "export PATH=\"$HOME/.local/share/mise/shims:$PATH\"\n",
                "\n",
                "# research-skills bootstrap path\n",
                "export PATH=\"$HOME/.local/bin:$PATH\"\n"
            ),
        )
        .unwrap();
        let shim_dir = home.join(".local/share/mise/shims");
        fs::create_dir_all(&shim_dir).unwrap();
        let shim = shim_dir.join("qiongli");
        fs::write(&shim, b"legacy-qiongli").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();
        let search_path = std::env::join_paths([PathBuf::from("/usr/bin")]).unwrap();

        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(&search_path),
            Some(OsStr::new("/bin/zsh")),
        );
        assert_eq!(inspection.state, CliInstallState::InstalledCurrent);
        assert_eq!(inspection.path_state, CliPathState::Configured);
        assert_eq!(
            inspection.reason_code,
            "qiongli-cli-installed-shell-configured"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn observes_zsh_profile_when_gui_process_has_no_shell_variable() {
        let root = test_root("zsh-profile-no-shell");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        fs::write(
            home.join(".zshrc"),
            concat!(
                "export PATH=\"$HOME/.local/share/mise/shims:$PATH\"\n",
                "export PATH=\"$HOME/.local/bin:$PATH\"\n"
            ),
        )
        .unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();

        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(OsStr::new("/usr/bin")),
            None,
        );
        assert_eq!(inspection.state, CliInstallState::InstalledCurrent);
        assert_eq!(inspection.path_state, CliPathState::Configured);
        assert_eq!(
            inspection.reason_code,
            "qiongli-cli-installed-shell-configured"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn observes_managed_bin_from_zsh_path_array_in_zshenv() {
        let root = test_root("zsh-path-array");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        fs::write(
            home.join(".zshenv"),
            "typeset -U path PATH\npath=(\"$HOME/.local/bin\" $path)\n",
        )
        .unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();

        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(OsStr::new("/usr/bin")),
            Some(OsStr::new("/bin/zsh")),
        );
        assert_eq!(inspection.path_state, CliPathState::Configured);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn path_directive_parser_ignores_unrelated_zsh_path_variables() {
        assert!(prepends_path(
            "path=(\"$HOME/.local/bin\" $path)",
            ".local/bin"
        ));
        assert!(prepends_path(
            "export PATH=\"$HOME/.local/bin:$PATH\"",
            ".local/bin"
        ));
        assert!(!prepends_path(
            "export FPATH=\"$HOME/.local/bin:$FPATH\"",
            ".local/bin"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn observes_managed_bin_from_a_home_local_symlinked_profile() {
        use std::os::unix::fs::symlink;

        let root = test_root("symlinked-zsh-profile");
        let home = root.join("home");
        let dotfiles = home.join("dotfiles");
        fs::create_dir_all(&dotfiles).unwrap();
        fs::write(
            dotfiles.join("zshrc"),
            "export PATH=\"$HOME/.local/bin:$PATH\"\n",
        )
        .unwrap();
        symlink(dotfiles.join("zshrc"), home.join(".zshrc")).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();

        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(OsStr::new("/usr/bin")),
            Some(OsStr::new("/bin/zsh")),
        );
        assert_eq!(inspection.path_state, CliPathState::Configured);
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn reports_mise_shim_configured_after_managed_bin_as_shadowed() {
        let root = test_root("zsh-mise-shadow");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        fs::write(
            home.join(".zshrc"),
            concat!(
                "export PATH=\"$HOME/.local/bin:$PATH\"\n",
                "export PATH=\"$HOME/.local/share/mise/shims:$PATH\"\n"
            ),
        )
        .unwrap();
        let shim_dir = home.join(".local/share/mise/shims");
        fs::create_dir_all(&shim_dir).unwrap();
        let shim = shim_dir.join("qiongli");
        fs::write(&shim, b"legacy-qiongli").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();
        let search_path = std::env::join_paths([PathBuf::from("/usr/bin")]).unwrap();

        let inspection = inspect_cli_install(
            Some(&home),
            Some(&source),
            "2.0.0-alpha.2",
            Some(&search_path),
            Some(OsStr::new("/bin/zsh")),
        );
        assert_eq!(inspection.state, CliInstallState::InstalledCurrent);
        assert_eq!(inspection.path_state, CliPathState::Shadowed);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ignores_stale_receipt_version_after_target_is_replaced() {
        let root = test_root("stale-receipt");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        let plan = preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap();
        apply_cli_install(&plan).unwrap();
        fs::write(cli_target(&home), b"legacy-cli").unwrap();

        let inspection =
            inspect_cli_install(Some(&home), Some(&source), "2.0.0-alpha.2", None, None);
        assert_eq!(inspection.state, CliInstallState::UpdateAvailable);
        assert_eq!(inspection.installed_version, None);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_remove_deletes_owned_install_and_refuses_drift() {
        let root = test_root("remove");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        apply_cli_install(&preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap()).unwrap();

        let removal = preview_cli_remove(&home, None, None).unwrap();
        fs::write(cli_target(&home), b"drifted").unwrap();
        assert_eq!(
            apply_cli_remove(&removal),
            Err("qiongli-cli-managed-target-drifted")
        );
        assert_eq!(
            preview_cli_remove(&home, None, None).unwrap_err(),
            "qiongli-cli-managed-target-drifted"
        );

        fs::write(cli_target(&home), b"native-cli-v2").unwrap();
        let removal = preview_cli_remove(&home, None, None).unwrap();
        assert_eq!(apply_cli_remove(&removal), Ok(CliRemovalEffect::Removed));
        assert!(!cli_target(&home).exists());
        assert!(!cli_receipt_path(&home).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_update_preserves_and_remove_restores_exact_unmanaged_predecessor() {
        let root = test_root("restore-predecessor");
        let home = root.join("home");
        let target = cli_target(&home);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, b"legacy-cli-v1").unwrap();
        let first = write_source(&root, b"native-cli-v2-a");
        apply_cli_install(&preview_cli_install(&home, &first, "2.0.0-alpha.2").unwrap()).unwrap();
        let second = root.join("qiongli-cli-next");
        fs::write(&second, b"native-cli-v2-b").unwrap();
        apply_cli_install(&preview_cli_install(&home, &second, "2.0.0-alpha.3").unwrap()).unwrap();

        let removal = preview_cli_remove(&home, None, None).unwrap();
        assert_eq!(
            apply_cli_remove(&removal),
            Ok(CliRemovalEffect::RestoredPredecessor)
        );
        assert_eq!(fs::read(cli_target(&home)).unwrap(), b"legacy-cli-v1");
        assert!(!cli_receipt_path(&home).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    #[allow(
        clippy::disallowed_methods,
        reason = "A3 verifies the exact managed CLI target in a fixed fresh zsh login shell"
    )]
    fn path_configuration_is_profile_digest_bound_and_login_shell_visible() {
        let root = test_root("path-configure");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        fs::write(home.join(".zprofile"), "export KEEP_ME=1\n").unwrap();
        let source = write_source(&root, b"native-cli-v2");
        apply_cli_install(&preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap()).unwrap();

        let plan = preview_cli_path_configure(&home, Some(OsStr::new("/bin/zsh"))).unwrap();
        assert_eq!(plan.profile_name(), ".zprofile");
        fs::write(home.join(".zprofile"), "export CONCURRENT=1\n").unwrap();
        assert_eq!(
            apply_cli_path_configure(&plan),
            Err("qiongli-cli-shell-profile-changed")
        );
        fs::write(home.join(".zprofile"), "export KEEP_ME=1\n").unwrap();
        let plan = preview_cli_path_configure(&home, Some(OsStr::new("/bin/zsh"))).unwrap();
        assert_eq!(
            apply_cli_path_configure(&plan),
            Ok("qiongli-cli-path-configured")
        );
        let profile = fs::read_to_string(home.join(".zprofile")).unwrap();
        assert!(profile.contains("export KEEP_ME=1"));
        assert_eq!(profile.matches(CLI_PATH_MARKER).count(), 1);

        if Path::new("/bin/zsh").is_file() {
            let output = std::process::Command::new("/bin/zsh")
                .args(["-l", "-c", "command -v qiongli"])
                .env("HOME", &home)
                .env("ZDOTDIR", &home)
                .env("PATH", "/usr/bin:/bin")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).trim(),
                cli_target(&home).to_string_lossy()
            );
        }
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    #[allow(
        clippy::disallowed_methods,
        reason = "A3 verifies the exact managed CLI target in a fixed fresh bash login shell"
    )]
    fn bash_path_configuration_is_visible_to_a_fresh_login_shell() {
        let root = test_root("bash-path-configure");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        fs::write(home.join(".bash_profile"), "export KEEP_ME=1\n").unwrap();
        let source = write_source(&root, b"native-cli-v2");
        apply_cli_install(&preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap()).unwrap();

        let plan = preview_cli_path_configure(&home, Some(OsStr::new("/bin/bash"))).unwrap();
        assert_eq!(plan.profile_name(), ".bash_profile");
        assert_eq!(
            apply_cli_path_configure(&plan),
            Ok("qiongli-cli-path-configured")
        );

        if Path::new("/bin/bash").is_file() {
            let output = std::process::Command::new("/bin/bash")
                .args(["-l", "-c", "command -v qiongli"])
                .env("HOME", &home)
                .env("PATH", "/usr/bin:/bin")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).trim(),
                cli_target(&home).to_string_lossy()
            );
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn removal_refuses_an_explicitly_shadowed_managed_cli() {
        let root = test_root("shadowed-remove");
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let source = write_source(&root, b"native-cli-v2");
        apply_cli_install(&preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap()).unwrap();
        let shadow_dir = root.join("shadow");
        fs::create_dir(&shadow_dir).unwrap();
        let shadow = shadow_dir.join(if cfg!(windows) {
            "qiongli.exe"
        } else {
            "qiongli"
        });
        fs::write(&shadow, b"legacy-cli").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&shadow, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let managed_bin = cli_target(&home).parent().unwrap().to_path_buf();
        let search_path = std::env::join_paths([shadow_dir, managed_bin]).unwrap();

        assert_eq!(
            preview_cli_remove(&home, Some(&search_path), None).unwrap_err(),
            "qiongli-cli-shadowed-removal-refused"
        );
        assert!(cli_target(&home).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn path_configuration_rejects_symlink_oversize_and_non_utf8_profiles() {
        use std::os::unix::fs::symlink;

        for (name, prepare, expected) in [
            ("symlink", 0_u8, "qiongli-cli-shell-profile-symlinked"),
            ("oversize", 1_u8, "qiongli-cli-shell-profile-too-large"),
            ("non-utf8", 2_u8, "qiongli-cli-shell-profile-not-utf8"),
        ] {
            let root = test_root(name);
            let home = root.join("home");
            fs::create_dir(&home).unwrap();
            let source = write_source(&root, b"native-cli-v2");
            apply_cli_install(&preview_cli_install(&home, &source, "2.0.0-alpha.2").unwrap())
                .unwrap();
            match prepare {
                0 => {
                    fs::write(home.join("real-profile"), b"safe\n").unwrap();
                    symlink(home.join("real-profile"), home.join(".zprofile")).unwrap();
                }
                1 => fs::write(
                    home.join(".zprofile"),
                    vec![b'x'; MAX_SHELL_PROFILE_BYTES as usize + 1],
                )
                .unwrap(),
                _ => fs::write(home.join(".zprofile"), [0xff, 0xfe]).unwrap(),
            }
            assert_eq!(
                preview_cli_path_configure(&home, Some(OsStr::new("/bin/zsh"))).unwrap_err(),
                expected
            );
            let _ = fs::remove_dir_all(root);
        }
    }
}
