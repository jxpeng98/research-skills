use std::fmt::{self, Debug, Display, Formatter};
use std::fs::{self, File};
use std::io::Read as _;
use std::path::{Path, PathBuf};

use qiongli_content::{LoadedResourcePack, WorkflowOverrides};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    ApprovalRequirement, Architecture, ArtifactIdentityV1, CapabilityProfile,
    ClaudeRegistrationExecutor, ClientActivationCoordinator, ClientActivationDisposition,
    ClientActivationState, ClientActivationTarget, CodexRegistrationExecutor,
    GrantVerificationContext, InstallPlanMetadataV1, InstallerKind,
    NativeCandidatePluginSourceDisposition, NativeCandidatePluginSourceVerification,
    NativeClientPluginGrantV1, NativeReleaseAuthority, OperatingSystem, ProductId, ReleaseChannel,
    TrustedPublicKey, VerifiedLaunchGrant, approve_install_plan, discover_claude_user,
    discover_client_activation, discover_codex_user,
    discover_native_candidate_plugin_source_target,
    materialize_packaged_product_plugin_source_with_overrides, parse_desktop_package_manifest,
    prepare_native_candidate_plugin_source_target, preview_client_activation,
    remove_native_candidate_plugin_source, verify_native_candidate_plugin_source,
};

pub const PACKAGED_PRODUCT_CONTROL_SCHEMA_VERSION: u32 = 3;
pub const PACKAGED_PRODUCT_CONTROL_FILE: &str = ".qiongli-product-control.json";

const MAX_CONTROL_BYTES: u64 = 256 * 1024;
const MAX_BINARY_BYTES: u64 = 128 * 1024 * 1024;
const PRODUCT_PLAN_TTL_SECONDS: u64 = 600;
const PRODUCT_APPROVALS: [ApprovalRequirement; 3] = [
    ApprovalRequirement::FilesystemWrite,
    ApprovalRequirement::ClientConfigChange,
    ApprovalRequirement::HostTrust,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackagedProductRecordType {
    QiongliPackagedProductControl,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackagedProductPluginIdentity {
    QiongliNext,
}

impl PackagedProductPluginIdentity {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::QiongliNext => "qiongli-next",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackagedProductSkillsScope {
    MarketplaceLite,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackagedProductActivationExpectation {
    RegisterThenClientEnablement,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackagedProductDesiredStateV1 {
    pub profile: CapabilityProfile,
    pub target_clients: Vec<ClientActivationTarget>,
    pub skills_scope: PackagedProductSkillsScope,
    pub plugin_identity: PackagedProductPluginIdentity,
    pub lite_mcp: bool,
    pub full_mcp_targets: Vec<ClientActivationTarget>,
    pub activation: PackagedProductActivationExpectation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackagedProductControlV1 {
    pub schema_version: u32,
    pub record_type: PackagedProductRecordType,
    pub artifact: ArtifactIdentityV1,
    pub product_source_commit: String,
    pub canonical_binary_sha256: String,
    pub resource_pack_sha256: String,
    pub desired_state: PackagedProductDesiredStateV1,
    pub client_plugins: Vec<NativeClientPluginGrantV1>,
}

impl PackagedProductControlV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, PackagedProductControlError> {
        if input.is_empty() || input.len() as u64 > MAX_CONTROL_BYTES {
            return Err(PackagedProductControlError::ControlInvalid);
        }
        let value = serde_json::from_slice::<Self>(input)
            .map_err(|_| PackagedProductControlError::ControlInvalid)?;
        value.validate()?;
        if canonical_json(&value)? != input {
            return Err(PackagedProductControlError::ControlInvalid);
        }
        Ok(value)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, PackagedProductControlError> {
        self.validate()?;
        canonical_json(self)
    }

    fn validate(&self) -> Result<(), PackagedProductControlError> {
        self.artifact
            .validate_lite()
            .map_err(|_| PackagedProductControlError::ControlInvalid)?;
        if self.schema_version != PACKAGED_PRODUCT_CONTROL_SCHEMA_VERSION
            || self.record_type != PackagedProductRecordType::QiongliPackagedProductControl
            || self.artifact.product != ProductId::Qiongli
            || self.artifact.profile != CapabilityProfile::Lite
            || self.artifact.installer_kind != InstallerKind::NativeInstaller
            || !valid_source_commit(&self.product_source_commit)
            || !valid_digest(&self.canonical_binary_sha256)
            || !valid_digest(&self.resource_pack_sha256)
            || self.desired_state.profile != CapabilityProfile::Lite
            || self.desired_state.target_clients
                != [
                    ClientActivationTarget::Codex,
                    ClientActivationTarget::ClaudeCode,
                ]
            || self.desired_state.skills_scope != PackagedProductSkillsScope::MarketplaceLite
            || self.desired_state.plugin_identity != PackagedProductPluginIdentity::QiongliNext
            || !self.desired_state.lite_mcp
            || self.desired_state.full_mcp_targets
                != [
                    ClientActivationTarget::Codex,
                    ClientActivationTarget::ClaudeCode,
                ]
            || self.desired_state.activation
                != PackagedProductActivationExpectation::RegisterThenClientEnablement
            || self.client_plugins.len() != 2
        {
            return Err(PackagedProductControlError::ControlInvalid);
        }
        let mut plugin_artifact = self.artifact.clone();
        plugin_artifact.installer_kind = InstallerKind::PluginBundle;
        for (plugin, target) in self
            .client_plugins
            .iter()
            .zip(self.desired_state.target_clients.iter().copied())
        {
            let grant = &plugin.signed_launch_grant.grant;
            if plugin.target != target
                || grant.artifact != plugin_artifact
                || grant.binary_sha256 != self.canonical_binary_sha256
                || grant.resource_pack_sha256 != self.resource_pack_sha256
                || grant.allowed_modes.as_slice() != target.allowed_grant_modes()
                || grant.integration_scopes.as_slice() != [target.integration_scope()]
            {
                return Err(PackagedProductControlError::ControlInvalid);
            }
        }
        Ok(())
    }
}

pub struct PackagedProductVerificationInput<'a> {
    pub current_executable: &'a Path,
    pub desktop_manifest_path: &'a Path,
    pub control_path: &'a Path,
    pub release_authority: &'a NativeReleaseAuthority,
    pub pack: &'a LoadedResourcePack<'a>,
    pub product_version: &'a str,
    pub product_source_commit: &'a str,
    pub home: &'a Path,
    pub now_unix: u64,
}

#[derive(Clone)]
pub struct PackagedProductInstallCapability {
    target: ClientActivationTarget,
    grant: VerifiedLaunchGrant,
}

impl PackagedProductInstallCapability {
    #[must_use]
    pub const fn target(&self) -> ClientActivationTarget {
        self.target
    }

    #[must_use]
    pub const fn grant(&self) -> &VerifiedLaunchGrant {
        &self.grant
    }
}

impl Debug for PackagedProductInstallCapability {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PackagedProductInstallCapability")
            .field("target", &self.target)
            .field("grant", &"<verified-launch-grant>")
            .finish()
    }
}

#[derive(Clone)]
pub struct VerifiedPackagedProduct {
    artifact: ArtifactIdentityV1,
    product_source_commit: String,
    resource_pack_sha256: String,
    current_executable: PathBuf,
    home: PathBuf,
    managed_product_root: PathBuf,
    control_sha256: String,
    capabilities: [PackagedProductInstallCapability; 2],
    trusted_keys: Vec<TrustedPublicKey>,
    minimum_generation: u64,
}

impl VerifiedPackagedProduct {
    #[must_use]
    pub const fn artifact(&self) -> &ArtifactIdentityV1 {
        &self.artifact
    }

    #[must_use]
    pub fn product_source_commit(&self) -> &str {
        &self.product_source_commit
    }

    #[must_use]
    pub fn resource_pack_sha256(&self) -> &str {
        &self.resource_pack_sha256
    }

    #[must_use]
    pub fn current_executable(&self) -> &Path {
        &self.current_executable
    }

    #[must_use]
    pub fn home(&self) -> &Path {
        &self.home
    }

    #[must_use]
    pub fn managed_product_root(&self) -> &Path {
        &self.managed_product_root
    }

    #[must_use]
    pub fn control_sha256(&self) -> &str {
        &self.control_sha256
    }

    #[must_use]
    pub fn capability(
        &self,
        target: ClientActivationTarget,
    ) -> Option<&PackagedProductInstallCapability> {
        self.capabilities
            .iter()
            .find(|capability| capability.target == target)
    }

    #[must_use]
    pub fn trusted_keys(&self) -> &[TrustedPublicKey] {
        &self.trusted_keys
    }

    #[must_use]
    pub const fn minimum_generation(&self) -> u64 {
        self.minimum_generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackagedProductInstallEffect {
    Install,
    Repair,
    AlreadyCurrent,
    ReplaceRequired,
    RecoveryRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagedProductInstallPreview {
    pub target: ClientActivationTarget,
    pub effect: PackagedProductInstallEffect,
    pub plan_digest_sha256: String,
    pub can_apply: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagedProductBatchInstallPreview {
    pub installs: Vec<PackagedProductInstallPreview>,
    pub plan_digest_sha256: String,
    pub can_apply: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackagedProductInstallDisposition {
    Installed,
    AlreadyCurrent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagedProductInstallCommit {
    pub target: ClientActivationTarget,
    pub disposition: PackagedProductInstallDisposition,
    pub source: NativeCandidatePluginSourceVerification,
    pub activation_transaction_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagedProductBatchInstallCommit {
    pub installs: Vec<PackagedProductInstallCommit>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagedProductInstallVerification {
    pub target: ClientActivationTarget,
    pub source: NativeCandidatePluginSourceVerification,
    pub activation_transaction_id: String,
}

impl Debug for VerifiedPackagedProduct {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedPackagedProduct")
            .field("artifact", &self.artifact)
            .field(
                "plugin_identity",
                &PackagedProductPluginIdentity::QiongliNext,
            )
            .field("current_executable", &"<verified-packaged-executable>")
            .field("home", &"<verified-current-user-home>")
            .field("managed_product_root", &"<fixed-product-managed-root>")
            .field("control_sha256", &self.control_sha256)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackagedProductControlError {
    ManifestInvalid,
    ControlInvalid,
    ProductMismatch,
    TargetMismatch,
    ExecutableInvalid,
    HomeInvalid,
    GrantInvalid,
    ClientUnavailable,
    ReplaceRequired,
    RecoveryRequired,
    PreviewInvalid,
    SourceInvalid,
    ActivationInvalid,
    CompensationFailed,
    Io,
}

impl PackagedProductControlError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ManifestInvalid => "packaged-product-manifest-invalid",
            Self::ControlInvalid => "packaged-product-control-invalid",
            Self::ProductMismatch => "packaged-product-identity-mismatch",
            Self::TargetMismatch => "packaged-product-target-mismatch",
            Self::ExecutableInvalid => "packaged-product-executable-invalid",
            Self::HomeInvalid => "packaged-product-home-invalid",
            Self::GrantInvalid => "packaged-product-grant-invalid",
            Self::ClientUnavailable => "packaged-product-client-unavailable",
            Self::ReplaceRequired => "packaged-product-replace-required",
            Self::RecoveryRequired => "packaged-product-recovery-required",
            Self::PreviewInvalid => "packaged-product-preview-invalid",
            Self::SourceInvalid => "packaged-product-source-invalid",
            Self::ActivationInvalid => "packaged-product-activation-invalid",
            Self::CompensationFailed => "packaged-product-compensation-failed",
            Self::Io => "packaged-product-evidence-unavailable",
        }
    }
}

impl Display for PackagedProductControlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for PackagedProductControlError {}

pub fn verify_packaged_product(
    input: &PackagedProductVerificationInput<'_>,
) -> Result<VerifiedPackagedProduct, PackagedProductControlError> {
    validate_home(input.home)?;
    let manifest_bytes = read_regular_file(input.desktop_manifest_path, MAX_CONTROL_BYTES)?;
    let manifest = parse_desktop_package_manifest(&manifest_bytes)
        .map_err(|_| PackagedProductControlError::ManifestInvalid)?;
    let control_bytes = read_regular_file(input.control_path, MAX_CONTROL_BYTES)?;
    let control = PackagedProductControlV1::from_json(&control_bytes)?;

    let expected_os =
        OperatingSystem::current().ok_or(PackagedProductControlError::TargetMismatch)?;
    let expected_arch =
        Architecture::current().ok_or(PackagedProductControlError::TargetMismatch)?;
    if manifest.artifact.os != expected_os
        || manifest.artifact.arch != expected_arch
        || manifest.artifact.channel != ReleaseChannel::Alpha
        || manifest.application.product_version != input.product_version
        || manifest.artifact.version != input.product_version
        || manifest.product_source_commit != input.product_source_commit
        || manifest.resource_pack_sha256 != input.pack.pack_sha256()
        || manifest.product_control_sha256.as_deref() != Some(sha256_hex(&control_bytes).as_str())
        || control.artifact != manifest.artifact
        || control.product_source_commit != manifest.product_source_commit
        || control.resource_pack_sha256 != manifest.resource_pack_sha256
    {
        return Err(PackagedProductControlError::ProductMismatch);
    }
    let expected_executable =
        packaged_canonical_executable(input.desktop_manifest_path, expected_os)?;
    let current_executable = fs::canonicalize(input.current_executable)
        .map_err(|_| PackagedProductControlError::ExecutableInvalid)?;
    let expected_executable = fs::canonicalize(expected_executable)
        .map_err(|_| PackagedProductControlError::ExecutableInvalid)?;
    if current_executable != expected_executable {
        return Err(PackagedProductControlError::ExecutableInvalid);
    }
    let binary = read_regular_file(&current_executable, MAX_BINARY_BYTES)?;
    if sha256_hex(&binary) != control.canonical_binary_sha256 {
        return Err(PackagedProductControlError::ExecutableInvalid);
    }

    let mut plugin_artifact = manifest.artifact.clone();
    plugin_artifact.installer_kind = InstallerKind::PluginBundle;
    let capabilities = control
        .client_plugins
        .iter()
        .map(|plugin| {
            let context = GrantVerificationContext {
                now_unix: input.now_unix,
                minimum_generation: input.release_authority.minimum_launch_grant_generation(),
                expected_artifact: &plugin_artifact,
                binary_sha256: &control.canonical_binary_sha256,
                resource_pack_sha256: input.pack.pack_sha256(),
                requested_mode: plugin.target.required_grant_mode(),
                requested_scope: plugin.target.integration_scope(),
            };
            plugin
                .signed_launch_grant
                .verify(input.release_authority.launch_grant_keys(), &context)
                .map(|grant| PackagedProductInstallCapability {
                    target: plugin.target,
                    grant,
                })
                .map_err(|_| PackagedProductControlError::GrantInvalid)
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| PackagedProductControlError::ControlInvalid)?;

    input
        .pack
        .manifest()
        .resolve_profile("marketplace-lite")
        .map_err(|_| PackagedProductControlError::ProductMismatch)?;
    Ok(VerifiedPackagedProduct {
        artifact: manifest.artifact,
        product_source_commit: manifest.product_source_commit,
        resource_pack_sha256: manifest.resource_pack_sha256,
        current_executable,
        home: input.home.to_path_buf(),
        managed_product_root: input.home.join(".qiongli"),
        control_sha256: sha256_hex(&control_bytes),
        capabilities,
        trusted_keys: input.release_authority.launch_grant_keys().to_vec(),
        minimum_generation: input.release_authority.minimum_launch_grant_generation(),
    })
}

/// Establishes the same operation capability from an independently signed native
/// installation. Callers provide embedded identity and current_exe(), never model paths.
#[allow(clippy::too_many_arguments)]
pub fn verify_native_packaged_product(
    pack: &LoadedResourcePack<'_>,
    authority: &NativeReleaseAuthority,
    home: &Path,
    current_executable: &Path,
    product_version: &str,
    source_commit: &str,
    now_unix: u64,
) -> Result<VerifiedPackagedProduct, crate::NativeCandidateLocalInstallError> {
    let artifact =
        crate::current_target_native_artifact_identity(product_version, authority.channel())
            .map_err(|_| crate::NativeCandidateLocalInstallError::InstalledBinaryInvalid)?;
    let mut capabilities = Vec::with_capacity(2);
    let mut identity = None;
    for target in [
        ClientActivationTarget::Codex,
        ClientActivationTarget::ClaudeCode,
    ] {
        let verified = crate::verify_installed_native_candidate_product(
            pack,
            authority,
            &crate::NativeReleaseCandidateVerificationContext {
                now_unix,
                expected_source_commit: source_commit,
                expected_artifact: &artifact,
                requested_target: target,
            },
            home,
            current_executable,
        )?;
        let digest = crate::native_release_candidate_signing_bytes(verified.candidate())
            .map(|bytes| sha256_hex(&bytes))
            .map_err(crate::NativeCandidateLocalInstallError::Candidate)?;
        let current = (digest, verified.current_executable().to_path_buf());
        if identity.as_ref().is_some_and(|prior| prior != &current) {
            return Err(crate::NativeCandidateLocalInstallError::ReceiptClosureInvalid);
        }
        identity = Some(current);
        capabilities.push(PackagedProductInstallCapability {
            target,
            grant: verified.plugin_grant().clone(),
        });
    }
    let (control_sha256, current_executable) =
        identity.ok_or(crate::NativeCandidateLocalInstallError::ReceiptClosureInvalid)?;
    Ok(VerifiedPackagedProduct {
        artifact,
        product_source_commit: source_commit.to_string(),
        resource_pack_sha256: pack.pack_sha256().to_string(),
        current_executable,
        home: home.to_path_buf(),
        managed_product_root: home.join(".qiongli"),
        control_sha256,
        capabilities: capabilities
            .try_into()
            .map_err(|_| crate::NativeCandidateLocalInstallError::ReceiptClosureInvalid)?,
        trusted_keys: authority.launch_grant_keys().to_vec(),
        minimum_generation: authority.minimum_launch_grant_generation(),
    })
}

pub fn preview_packaged_product_install(
    product: &VerifiedPackagedProduct,
    target: ClientActivationTarget,
) -> Result<PackagedProductInstallPreview, PackagedProductControlError> {
    preview_packaged_product_install_with_variant(product, target, None)
}

pub fn preview_packaged_product_install_with_variant(
    product: &VerifiedPackagedProduct,
    target: ClientActivationTarget,
    workflow_variant_sha256: Option<&str>,
) -> Result<PackagedProductInstallPreview, PackagedProductControlError> {
    let _capability = product
        .capability(target)
        .ok_or(PackagedProductControlError::ClientUnavailable)?;
    let handle = discover_client_activation(product.home(), None, target)
        .map_err(|_| PackagedProductControlError::ClientUnavailable)?;
    let discovery = handle.discovery();
    let effect = if matches!(
        discovery.registration,
        ClientActivationState::RecoveryRequired
    ) {
        PackagedProductInstallEffect::RecoveryRequired
    } else if matches!(discovery.source, ClientActivationState::Missing)
        && matches!(discovery.registration, ClientActivationState::Missing)
    {
        PackagedProductInstallEffect::Install
    } else if matches!(discovery.source, ClientActivationState::Ready)
        && matches!(discovery.registration, ClientActivationState::Ready)
    {
        if verify_packaged_product_install_with_variant(product, target, workflow_variant_sha256)
            .is_ok()
        {
            PackagedProductInstallEffect::AlreadyCurrent
        } else {
            PackagedProductInstallEffect::ReplaceRequired
        }
    } else if matches!(discovery.source, ClientActivationState::Ready)
        && matches!(discovery.registration, ClientActivationState::Missing)
        && verify_packaged_product_source_with_variant(product, target, workflow_variant_sha256)
            .is_ok()
    {
        PackagedProductInstallEffect::Repair
    } else {
        PackagedProductInstallEffect::ReplaceRequired
    };
    let plan_digest_sha256 = product_install_digest(
        product,
        target,
        effect,
        discovery.source,
        discovery.registration,
        workflow_variant_sha256,
    );
    Ok(PackagedProductInstallPreview {
        target,
        effect,
        plan_digest_sha256,
        can_apply: matches!(
            effect,
            PackagedProductInstallEffect::Install
                | PackagedProductInstallEffect::Repair
                | PackagedProductInstallEffect::AlreadyCurrent
        ),
    })
}

pub fn preview_packaged_product_batch_install(
    product: &VerifiedPackagedProduct,
    targets: &[ClientActivationTarget],
) -> Result<PackagedProductBatchInstallPreview, PackagedProductControlError> {
    preview_packaged_product_batch_install_with_variant(product, targets, None)
}

pub fn preview_packaged_product_batch_install_with_variant(
    product: &VerifiedPackagedProduct,
    targets: &[ClientActivationTarget],
    workflow_variant_sha256: Option<&str>,
) -> Result<PackagedProductBatchInstallPreview, PackagedProductControlError> {
    validate_target_sequence(targets)?;
    let installs = targets
        .iter()
        .copied()
        .map(|target| {
            preview_packaged_product_install_with_variant(product, target, workflow_variant_sha256)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let can_apply = installs.iter().all(|preview| preview.can_apply);
    let plan_digest_sha256 = product_batch_install_digest(product, &installs);
    Ok(PackagedProductBatchInstallPreview {
        installs,
        plan_digest_sha256,
        can_apply,
    })
}

pub fn apply_packaged_product_install(
    pack: &LoadedResourcePack<'_>,
    product: &VerifiedPackagedProduct,
    preview: &PackagedProductInstallPreview,
    now_unix: u64,
) -> Result<PackagedProductInstallCommit, PackagedProductControlError> {
    apply_packaged_product_install_with_overrides(pack, product, preview, now_unix, None)
}

pub fn apply_packaged_product_install_with_overrides(
    pack: &LoadedResourcePack<'_>,
    product: &VerifiedPackagedProduct,
    preview: &PackagedProductInstallPreview,
    now_unix: u64,
    overrides: Option<&WorkflowOverrides>,
) -> Result<PackagedProductInstallCommit, PackagedProductControlError> {
    let workflow_variant_sha256 = overrides.map(WorkflowOverrides::variant_sha256);
    let current = preview_packaged_product_install_with_variant(
        product,
        preview.target,
        workflow_variant_sha256,
    )?;
    if current != *preview {
        return Err(PackagedProductControlError::PreviewInvalid);
    }
    match preview.effect {
        PackagedProductInstallEffect::ReplaceRequired => {
            return Err(PackagedProductControlError::ReplaceRequired);
        }
        PackagedProductInstallEffect::RecoveryRequired => {
            return Err(PackagedProductControlError::RecoveryRequired);
        }
        PackagedProductInstallEffect::AlreadyCurrent => {
            let verification = verify_packaged_product_install_with_variant(
                product,
                preview.target,
                workflow_variant_sha256,
            )?;
            return Ok(PackagedProductInstallCommit {
                target: preview.target,
                disposition: PackagedProductInstallDisposition::AlreadyCurrent,
                source: verification.source,
                activation_transaction_id: verification.activation_transaction_id,
            });
        }
        PackagedProductInstallEffect::Install | PackagedProductInstallEffect::Repair => {}
    }

    let capability = product
        .capability(preview.target)
        .ok_or(PackagedProductControlError::ClientUnavailable)?;
    let source_target =
        prepare_native_candidate_plugin_source_target(product.home(), preview.target)
            .map_err(|_| PackagedProductControlError::SourceInvalid)?;
    let source = materialize_packaged_product_plugin_source_with_overrides(
        pack,
        preview.target,
        capability.grant(),
        product.current_executable(),
        &source_target,
        overrides,
    )
    .map_err(|_| PackagedProductControlError::SourceInvalid)?;
    let handle = match discover_client_activation(product.home(), None, preview.target) {
        Ok(handle) => handle,
        Err(_) => {
            compensate_source(&source_target, source.disposition)?;
            return Err(PackagedProductControlError::ActivationInvalid);
        }
    };
    let activation_preview = match preview_client_activation(
        &handle,
        InstallPlanMetadataV1 {
            plan_id: format!("packaged-product-install-{}", target_slug(preview.target)),
            created_at_unix: now_unix,
            expires_at_unix: now_unix
                .saturating_add(PRODUCT_PLAN_TTL_SECONDS)
                .min(capability.grant().grant().expires_at_unix),
        },
        capability.grant(),
        product.trusted_keys(),
        product.minimum_generation(),
        now_unix,
    ) {
        Ok(preview) => preview,
        Err(_) => {
            compensate_source(&source_target, source.disposition)?;
            return Err(PackagedProductControlError::ActivationInvalid);
        }
    };
    let approval =
        match approve_install_plan(activation_preview.plan(), &PRODUCT_APPROVALS, now_unix) {
            Ok(approval) => approval,
            Err(_) => {
                compensate_source(&source_target, source.disposition)?;
                return Err(PackagedProductControlError::ActivationInvalid);
            }
        };
    let coordinator = ClientActivationCoordinator::new(handle);
    let activation = match coordinator.apply(&activation_preview, &approval, now_unix) {
        Ok(commit) => commit,
        Err(_) => {
            compensate_source(&source_target, source.disposition)?;
            return Err(PackagedProductControlError::ActivationInvalid);
        }
    };
    let verification = match verify_packaged_product_install_with_variant(
        product,
        preview.target,
        workflow_variant_sha256,
    ) {
        Ok(verification) => verification,
        Err(_) => {
            let registration_removed = coordinator.remove(now_unix).is_ok();
            let source_removed = compensate_source(&source_target, source.disposition).is_ok();
            return Err(if registration_removed && source_removed {
                PackagedProductControlError::ActivationInvalid
            } else {
                PackagedProductControlError::CompensationFailed
            });
        }
    };
    Ok(PackagedProductInstallCommit {
        target: preview.target,
        disposition: match activation.disposition {
            ClientActivationDisposition::Activated | ClientActivationDisposition::Repaired => {
                PackagedProductInstallDisposition::Installed
            }
            ClientActivationDisposition::AlreadyActive
            | ClientActivationDisposition::AlreadyHealthy => {
                PackagedProductInstallDisposition::AlreadyCurrent
            }
        },
        source: verification.source,
        activation_transaction_id: verification.activation_transaction_id,
    })
}

pub fn apply_packaged_product_batch_install(
    pack: &LoadedResourcePack<'_>,
    product: &VerifiedPackagedProduct,
    preview: &PackagedProductBatchInstallPreview,
    now_unix: u64,
) -> Result<PackagedProductBatchInstallCommit, PackagedProductControlError> {
    apply_packaged_product_batch_install_with_overrides(pack, product, preview, now_unix, None)
}

pub fn apply_packaged_product_batch_install_with_overrides(
    pack: &LoadedResourcePack<'_>,
    product: &VerifiedPackagedProduct,
    preview: &PackagedProductBatchInstallPreview,
    now_unix: u64,
    overrides: Option<&WorkflowOverrides>,
) -> Result<PackagedProductBatchInstallCommit, PackagedProductControlError> {
    let targets = preview
        .installs
        .iter()
        .map(|install| install.target)
        .collect::<Vec<_>>();
    let current = preview_packaged_product_batch_install_with_variant(
        product,
        &targets,
        overrides.map(WorkflowOverrides::variant_sha256),
    )?;
    if current != *preview || !preview.can_apply {
        return Err(PackagedProductControlError::PreviewInvalid);
    }

    let mut commits = Vec::with_capacity(preview.installs.len());
    for install in &preview.installs {
        match apply_packaged_product_install_with_overrides(
            pack, product, install, now_unix, overrides,
        ) {
            Ok(commit) => commits.push(commit),
            Err(error) => {
                let compensated =
                    commits
                        .iter()
                        .zip(preview.installs.iter())
                        .rev()
                        .all(|(_, applied)| {
                            compensate_packaged_product_install(
                                product,
                                applied,
                                now_unix,
                                overrides.map(WorkflowOverrides::variant_sha256),
                            )
                            .is_ok()
                        });
                return Err(if compensated {
                    error
                } else {
                    PackagedProductControlError::CompensationFailed
                });
            }
        }
    }
    Ok(PackagedProductBatchInstallCommit { installs: commits })
}

pub fn verify_packaged_product_install(
    product: &VerifiedPackagedProduct,
    target: ClientActivationTarget,
) -> Result<PackagedProductInstallVerification, PackagedProductControlError> {
    verify_packaged_product_install_with_variant(product, target, None)
}

pub fn verify_packaged_product_install_with_variant(
    product: &VerifiedPackagedProduct,
    target: ClientActivationTarget,
    workflow_variant_sha256: Option<&str>,
) -> Result<PackagedProductInstallVerification, PackagedProductControlError> {
    let source =
        verify_packaged_product_source_with_variant(product, target, workflow_variant_sha256)?;
    let handle = discover_client_activation(product.home(), None, target)
        .map_err(|_| PackagedProductControlError::ActivationInvalid)?;
    let activation = ClientActivationCoordinator::new(handle)
        .verify()
        .map_err(|_| PackagedProductControlError::ActivationInvalid)?;
    Ok(PackagedProductInstallVerification {
        target,
        source,
        activation_transaction_id: activation.transaction_id,
    })
}

/// Verifies one receipt-owned Qiongli installation without requiring it to
/// match the currently running packaged product version.
///
/// This is the explicit replacement/removal boundary for an intact older
/// managed installation. The fixed source tree, registration receipt and their
/// cross-receipt binding must all verify. Unmanaged, drifted or partially
/// present installations remain ineligible for removal.
pub fn verify_receipt_owned_packaged_product_install(
    product: &VerifiedPackagedProduct,
    target: ClientActivationTarget,
) -> Result<PackagedProductInstallVerification, PackagedProductControlError> {
    let capability = product
        .capability(target)
        .ok_or(PackagedProductControlError::ClientUnavailable)?;
    let source_target = discover_native_candidate_plugin_source_target(product.home(), target)
        .map_err(|_| PackagedProductControlError::SourceInvalid)?;
    let source = verify_native_candidate_plugin_source(&source_target)
        .map_err(|_| PackagedProductControlError::SourceInvalid)?;
    let current_artifact = &capability.grant().grant().artifact;
    if source.target != target
        || source.artifact.product != ProductId::Qiongli
        || source.artifact.channel != current_artifact.channel
        || source.artifact.profile != CapabilityProfile::Lite
        || source.artifact.os != current_artifact.os
        || source.artifact.arch != current_artifact.arch
        || source.artifact.installer_kind != InstallerKind::PluginBundle
    {
        return Err(PackagedProductControlError::SourceInvalid);
    }

    let activation_transaction_id = match target {
        ClientActivationTarget::Codex => {
            let receipt = CodexRegistrationExecutor::new(
                discover_codex_user(product.home())
                    .map_err(|_| PackagedProductControlError::ActivationInvalid)?,
            )
            .verify()
            .map_err(|_| PackagedProductControlError::ActivationInvalid)?
            .receipt;
            if receipt.artifact != source.artifact
                || receipt.ownership.artifact_digest_sha256 != source.signed_grant_payload_sha256
                || receipt.source_receipt_sha256 != source.receipt_sha256
                || receipt.source_content_root_sha256 != source.package_content_root_sha256
            {
                return Err(PackagedProductControlError::ActivationInvalid);
            }
            receipt.transaction_id
        }
        ClientActivationTarget::ClaudeCode => {
            let receipt = ClaudeRegistrationExecutor::new(
                discover_claude_user(product.home())
                    .map_err(|_| PackagedProductControlError::ActivationInvalid)?,
            )
            .verify()
            .map_err(|_| PackagedProductControlError::ActivationInvalid)?
            .receipt;
            if receipt.artifact != source.artifact
                || receipt.ownership.artifact_digest_sha256 != source.signed_grant_payload_sha256
                || receipt.source_receipt_sha256 != source.receipt_sha256
                || receipt.source_content_root_sha256 != source.package_content_root_sha256
            {
                return Err(PackagedProductControlError::ActivationInvalid);
            }
            receipt.transaction_id
        }
    };
    Ok(PackagedProductInstallVerification {
        target,
        source,
        activation_transaction_id,
    })
}

fn verify_packaged_product_source_with_variant(
    product: &VerifiedPackagedProduct,
    target: ClientActivationTarget,
    workflow_variant_sha256: Option<&str>,
) -> Result<NativeCandidatePluginSourceVerification, PackagedProductControlError> {
    let capability = product
        .capability(target)
        .ok_or(PackagedProductControlError::ClientUnavailable)?;
    let source_target = discover_native_candidate_plugin_source_target(product.home(), target)
        .map_err(|_| PackagedProductControlError::SourceInvalid)?;
    let source = verify_native_candidate_plugin_source(&source_target)
        .map_err(|_| PackagedProductControlError::SourceInvalid)?;
    if source.artifact != capability.grant().grant().artifact
        || source.signed_grant_payload_sha256 != capability.grant().signed_payload_sha256()
        || source.binary_sha256 != capability.grant().grant().binary_sha256
        || source.resource_pack_sha256 != capability.grant().grant().resource_pack_sha256
        || source.workflow_variant_sha256.as_deref() != workflow_variant_sha256
    {
        return Err(PackagedProductControlError::SourceInvalid);
    }
    Ok(source)
}

pub fn remove_packaged_product_install(
    product: &VerifiedPackagedProduct,
    target: ClientActivationTarget,
    now_unix: u64,
) -> Result<PackagedProductInstallVerification, PackagedProductControlError> {
    let verification = verify_receipt_owned_packaged_product_install(product, target)?;
    let handle = discover_client_activation(product.home(), None, target)
        .map_err(|_| PackagedProductControlError::ActivationInvalid)?;
    ClientActivationCoordinator::new(handle)
        .remove(now_unix)
        .map_err(|_| PackagedProductControlError::ActivationInvalid)?;
    let source_target = discover_native_candidate_plugin_source_target(product.home(), target)
        .map_err(|_| PackagedProductControlError::SourceInvalid)?;
    remove_native_candidate_plugin_source(&source_target)
        .map_err(|_| PackagedProductControlError::RecoveryRequired)?;
    Ok(verification)
}

fn compensate_source(
    target: &crate::NativeCandidatePluginSourceTarget,
    disposition: NativeCandidatePluginSourceDisposition,
) -> Result<(), PackagedProductControlError> {
    if disposition == NativeCandidatePluginSourceDisposition::Materialized {
        remove_native_candidate_plugin_source(target)
            .map_err(|_| PackagedProductControlError::CompensationFailed)?;
    }
    Ok(())
}

fn compensate_packaged_product_install(
    product: &VerifiedPackagedProduct,
    preview: &PackagedProductInstallPreview,
    now_unix: u64,
    workflow_variant_sha256: Option<&str>,
) -> Result<(), PackagedProductControlError> {
    match preview.effect {
        PackagedProductInstallEffect::Install => {
            remove_packaged_product_install(product, preview.target, now_unix)?;
        }
        PackagedProductInstallEffect::Repair => {
            let handle = discover_client_activation(product.home(), None, preview.target)
                .map_err(|_| PackagedProductControlError::ActivationInvalid)?;
            ClientActivationCoordinator::new(handle)
                .remove(now_unix)
                .map_err(|_| PackagedProductControlError::CompensationFailed)?;
            verify_packaged_product_source_with_variant(
                product,
                preview.target,
                workflow_variant_sha256,
            )?;
        }
        PackagedProductInstallEffect::AlreadyCurrent => {}
        PackagedProductInstallEffect::ReplaceRequired
        | PackagedProductInstallEffect::RecoveryRequired => {
            return Err(PackagedProductControlError::PreviewInvalid);
        }
    }
    Ok(())
}

fn product_install_digest(
    product: &VerifiedPackagedProduct,
    target: ClientActivationTarget,
    effect: PackagedProductInstallEffect,
    source: ClientActivationState,
    registration: ClientActivationState,
    workflow_variant_sha256: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-PACKAGED-PRODUCT-INSTALL-V2\0");
    hasher.update(product.control_sha256().as_bytes());
    hasher.update([target as u8, effect as u8, source as u8, registration as u8]);
    hasher.update(workflow_variant_sha256.unwrap_or("canonical").as_bytes());
    sha256_hex(&hasher.finalize())
}

fn product_batch_install_digest(
    product: &VerifiedPackagedProduct,
    installs: &[PackagedProductInstallPreview],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"QIONGLI-PACKAGED-PRODUCT-BATCH-INSTALL-V1\0");
    hasher.update(product.control_sha256().as_bytes());
    for install in installs {
        hasher.update([install.target as u8, install.effect as u8]);
        hasher.update(install.plan_digest_sha256.as_bytes());
    }
    sha256_hex(&hasher.finalize())
}

fn validate_target_sequence(
    targets: &[ClientActivationTarget],
) -> Result<(), PackagedProductControlError> {
    if !matches!(
        targets,
        [ClientActivationTarget::Codex]
            | [ClientActivationTarget::ClaudeCode]
            | [
                ClientActivationTarget::Codex,
                ClientActivationTarget::ClaudeCode
            ]
    ) {
        return Err(PackagedProductControlError::TargetMismatch);
    }
    Ok(())
}

const fn target_slug(target: ClientActivationTarget) -> &'static str {
    match target {
        ClientActivationTarget::Codex => "codex",
        ClientActivationTarget::ClaudeCode => "claude-code",
    }
}

pub fn packaged_product_control_path(
    desktop_manifest_path: &Path,
) -> Result<PathBuf, PackagedProductControlError> {
    desktop_manifest_path
        .parent()
        .map(|parent| parent.join(PACKAGED_PRODUCT_CONTROL_FILE))
        .ok_or(PackagedProductControlError::ManifestInvalid)
}

fn packaged_canonical_executable(
    desktop_manifest_path: &Path,
    os: OperatingSystem,
) -> Result<PathBuf, PackagedProductControlError> {
    let manifest_parent = desktop_manifest_path
        .parent()
        .ok_or(PackagedProductControlError::ManifestInvalid)?;
    Ok(match os {
        OperatingSystem::Macos => manifest_parent
            .parent()
            .ok_or(PackagedProductControlError::ManifestInvalid)?
            .join("MacOS/qiongli-cli"),
        OperatingSystem::Windows => manifest_parent.join("qiongli-cli.exe"),
        OperatingSystem::Linux => manifest_parent.join("qiongli-cli"),
    })
}

fn validate_home(home: &Path) -> Result<(), PackagedProductControlError> {
    if !home.is_absolute() || home.parent().is_none() {
        return Err(PackagedProductControlError::HomeInvalid);
    }
    let metadata =
        fs::symlink_metadata(home).map_err(|_| PackagedProductControlError::HomeInvalid)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(PackagedProductControlError::HomeInvalid);
    }
    Ok(())
}

fn read_regular_file(
    path: &Path,
    maximum_bytes: u64,
) -> Result<Vec<u8>, PackagedProductControlError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| PackagedProductControlError::Io)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum_bytes
    {
        return Err(PackagedProductControlError::Io);
    }
    let mut file = File::open(path).map_err(|_| PackagedProductControlError::Io)?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| PackagedProductControlError::Io)?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > maximum_bytes {
        return Err(PackagedProductControlError::Io);
    }
    Ok(bytes)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, PackagedProductControlError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| PackagedProductControlError::ControlInvalid)
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU64, Ordering};

    use ed25519_dalek::{Signer, SigningKey};
    use qiongli_content::{
        BuiltResourcePack, CompatibleProduct, ResourcePackBuildMetadata, build_resource_pack,
        collect_canonical_sources, load_resource_pack,
    };
    use serde_json::json;

    use super::*;
    use crate::{
        DesktopApplicationMetadataV1, DesktopPackageBinaries, DesktopPackageInput,
        GrantSignatureV1, LaunchGrantV1, SignatureAlgorithm, SignedLaunchGrantV1,
        approve_native_artifact_target, compose_desktop_package, compose_native_artifact,
        current_target_native_artifact_identity, launch_grant_signing_bytes,
    };

    const NOW: u64 = 1_780_000_000;
    const VERSION: &str = "2.0.0-alpha.1";
    const SOURCE_COMMIT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
    static BUILT_PACK: OnceLock<BuiltResourcePack> = OnceLock::new();
    static LOADED_PACK: OnceLock<qiongli_content::LoadedResourcePack<'static>> = OnceLock::new();

    struct Fixture {
        root: PathBuf,
        home: PathBuf,
        executable: PathBuf,
        manifest: PathBuf,
        control: PathBuf,
        authority: NativeReleaseAuthority,
        version: String,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            Self::with_version(name, VERSION)
        }

        fn with_version(name: &str, version: &str) -> Self {
            let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/qiongli-product-control-tests");
            fs::create_dir_all(&base).unwrap();
            let root = base.join(format!(
                "{name}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            create_private_directory(&home);
            let assembly = root.join("assembly");
            create_private_directory(&assembly);
            let source_binary = root.join("source-binary");
            let binary = executable_bytes(OperatingSystem::current().unwrap(), b"canonical");
            fs::write(&source_binary, &binary).unwrap();
            set_executable(&source_binary);

            let portable =
                current_target_native_artifact_identity(version, ReleaseChannel::Alpha).unwrap();
            let artifact_id = crate::native_artifact_id(&portable).unwrap();
            let artifact_target =
                approve_native_artifact_target(assembly.join(artifact_id), &portable).unwrap();
            let artifact =
                compose_native_artifact(test_pack(), &portable, &source_binary, &artifact_target)
                    .unwrap();
            let launch = SigningKey::from_bytes(&[42; 32]);
            let release = SigningKey::from_bytes(&[43; 32]);
            let authority = authority(&release, &launch);
            let mut desktop_artifact = portable.clone();
            desktop_artifact.installer_kind = InstallerKind::NativeInstaller;
            let mut plugin_artifact = portable;
            plugin_artifact.installer_kind = InstallerKind::PluginBundle;
            let binary_sha256 = sha256_hex(&binary);
            let clients = [
                ClientActivationTarget::Codex,
                ClientActivationTarget::ClaudeCode,
            ]
            .into_iter()
            .map(|target| NativeClientPluginGrantV1 {
                target,
                signed_launch_grant: signed_grant(
                    &launch,
                    plugin_artifact.clone(),
                    target,
                    &binary_sha256,
                ),
            })
            .collect();
            let control_document = PackagedProductControlV1 {
                schema_version: PACKAGED_PRODUCT_CONTROL_SCHEMA_VERSION,
                record_type: PackagedProductRecordType::QiongliPackagedProductControl,
                artifact: desktop_artifact,
                product_source_commit: SOURCE_COMMIT.to_string(),
                canonical_binary_sha256: binary_sha256,
                resource_pack_sha256: test_pack().pack_sha256().to_string(),
                desired_state: PackagedProductDesiredStateV1 {
                    profile: CapabilityProfile::Lite,
                    target_clients: vec![
                        ClientActivationTarget::Codex,
                        ClientActivationTarget::ClaudeCode,
                    ],
                    skills_scope: PackagedProductSkillsScope::MarketplaceLite,
                    plugin_identity: PackagedProductPluginIdentity::QiongliNext,
                    lite_mcp: true,
                    full_mcp_targets: vec![
                        ClientActivationTarget::Codex,
                        ClientActivationTarget::ClaudeCode,
                    ],
                    activation: PackagedProductActivationExpectation::RegisterThenClientEnablement,
                },
                client_plugins: clients,
            };
            let control_bytes = control_document.to_canonical_json().unwrap();
            let zotero_companion = companion_stub();
            let package = compose_desktop_package(
                DesktopPackageInput::new(
                    &artifact,
                    DesktopPackageBinaries::new(
                        &binary,
                        &executable_bytes(OperatingSystem::current().unwrap(), b"launcher"),
                        &executable_bytes(OperatingSystem::current().unwrap(), b"helper"),
                    ),
                    &png_stub(),
                    b"MIT License\nPermission is hereby granted",
                    SOURCE_COMMIT,
                    DesktopApplicationMetadataV1::new(
                        "Qiongli",
                        "Qiongli 2",
                        "io.github.jxpeng98.qiongli",
                        version,
                        "MIT",
                    ),
                    &zotero_companion,
                )
                .with_product_control(&control_bytes),
            )
            .unwrap();
            let package_root = root.join(package.manifest().package_root.as_str());
            let manifest = root.join(&package.manifest().manifest_path);
            let executable = match OperatingSystem::current().unwrap() {
                OperatingSystem::Macos => package_root.join("Contents/MacOS/qiongli-cli"),
                OperatingSystem::Windows => package_root.join("qiongli-cli.exe"),
                OperatingSystem::Linux => package_root.join("qiongli-cli"),
            };
            fs::create_dir_all(manifest.parent().unwrap()).unwrap();
            fs::create_dir_all(executable.parent().unwrap()).unwrap();
            fs::write(&manifest, package.manifest_bytes()).unwrap();
            let control = packaged_product_control_path(&manifest).unwrap();
            fs::write(&control, control_bytes).unwrap();
            fs::write(&executable, binary).unwrap();
            set_executable(&executable);
            Self {
                root,
                home,
                executable,
                manifest,
                control,
                authority,
                version: version.to_string(),
            }
        }

        fn verify(&self) -> Result<VerifiedPackagedProduct, PackagedProductControlError> {
            self.verify_for_home(&self.home)
        }

        fn verify_for_home(
            &self,
            home: &Path,
        ) -> Result<VerifiedPackagedProduct, PackagedProductControlError> {
            verify_packaged_product(&PackagedProductVerificationInput {
                current_executable: &self.executable,
                desktop_manifest_path: &self.manifest,
                control_path: &self.control,
                release_authority: &self.authority,
                pack: test_pack(),
                product_version: &self.version,
                product_source_commit: SOURCE_COMMIT,
                home,
                now_unix: NOW,
            })
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn verified_package_derives_only_two_bounded_in_memory_capabilities() {
        let fixture = Fixture::new("verified");
        let product = fixture.verify().unwrap();
        assert_eq!(product.artifact().version, fixture.version);
        assert_eq!(product.product_source_commit(), SOURCE_COMMIT);
        assert_eq!(product.resource_pack_sha256(), test_pack().pack_sha256());
        assert!(product.capability(ClientActivationTarget::Codex).is_some());
        assert!(
            product
                .capability(ClientActivationTarget::ClaudeCode)
                .is_some()
        );
        assert!(!fixture.home.join(".qiongli").exists());
        assert!(!format!("{product:?}").contains(fixture.home.to_string_lossy().as_ref()));
    }

    #[test]
    fn executable_control_and_target_tampering_fail_closed() {
        let fixture = Fixture::new("tamper");
        fs::write(
            &fixture.executable,
            executable_bytes(OperatingSystem::current().unwrap(), b"tampered"),
        )
        .unwrap();
        assert_eq!(
            fixture.verify().unwrap_err(),
            PackagedProductControlError::ExecutableInvalid
        );
        assert!(!fixture.home.join(".qiongli").exists());
    }

    #[test]
    fn packaged_install_is_previewed_applied_verified_and_removed_with_receipts() {
        let fixture = Fixture::new("lifecycle");
        let agents = fixture.home.join(".agents");
        create_private_directory(&agents);
        let plugins = agents.join("plugins");
        create_private_directory(&plugins);
        fs::write(
            plugins.join("marketplace.json"),
            br#"{"plugins":[{"category":"Education","name":"qiongli","policy":{"authentication":"ON_INSTALL","installation":"AVAILABLE"},"source":{"path":"./legacy-qiongli","source":"local"}}]}"#,
        )
        .unwrap();
        let product = fixture.verify().unwrap();
        let preview =
            preview_packaged_product_install(&product, ClientActivationTarget::Codex).unwrap();
        assert_eq!(preview.effect, PackagedProductInstallEffect::Install);
        let commit =
            apply_packaged_product_install(test_pack(), &product, &preview, NOW + 1).unwrap();
        assert_eq!(
            commit.disposition,
            PackagedProductInstallDisposition::Installed
        );
        assert!(
            fixture
                .home
                .join(".qiongli/plugins/codex/qiongli-next")
                .is_dir()
        );
        let marketplace: serde_json::Value =
            serde_json::from_slice(&fs::read(plugins.join("marketplace.json")).unwrap()).unwrap();
        let names = marketplace["plugins"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|entry| entry["name"].as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["qiongli", "qiongli-next"]);
        verify_packaged_product_install(&product, ClientActivationTarget::Codex).unwrap();
        assert_eq!(
            preview_packaged_product_install(&product, ClientActivationTarget::Codex)
                .unwrap()
                .effect,
            PackagedProductInstallEffect::AlreadyCurrent
        );
        remove_packaged_product_install(&product, ClientActivationTarget::Codex, NOW + 2).unwrap();
        assert!(
            !fixture
                .home
                .join(".qiongli/plugins/codex/qiongli-next")
                .exists()
        );
        let marketplace: serde_json::Value =
            serde_json::from_slice(&fs::read(plugins.join("marketplace.json")).unwrap()).unwrap();
        assert_eq!(marketplace["plugins"].as_array().unwrap().len(), 1);
        assert_eq!(marketplace["plugins"][0]["name"], "qiongli");
    }

    #[test]
    fn current_product_can_explicitly_remove_an_intact_prior_managed_install() {
        let prior = Fixture::with_version("prior-managed", "2.0.0-alpha.1");
        let prior_product = prior.verify().unwrap();
        let preview =
            preview_packaged_product_install(&prior_product, ClientActivationTarget::Codex)
                .unwrap();
        apply_packaged_product_install(test_pack(), &prior_product, &preview, NOW + 1).unwrap();

        let current = Fixture::with_version("current-product", "2.0.0-alpha.2");
        let current_product = current.verify_for_home(&prior.home).unwrap();
        assert_eq!(
            preview_packaged_product_install(&current_product, ClientActivationTarget::Codex)
                .unwrap()
                .effect,
            PackagedProductInstallEffect::ReplaceRequired
        );
        assert_eq!(
            verify_receipt_owned_packaged_product_install(
                &current_product,
                ClientActivationTarget::Codex,
            )
            .unwrap()
            .source
            .artifact
            .version,
            "2.0.0-alpha.1"
        );

        remove_packaged_product_install(&current_product, ClientActivationTarget::Codex, NOW + 2)
            .unwrap();
        let next =
            preview_packaged_product_install(&current_product, ClientActivationTarget::Codex)
                .unwrap();
        assert_eq!(next.effect, PackagedProductInstallEffect::Install);
        apply_packaged_product_install(test_pack(), &current_product, &next, NOW + 3).unwrap();
        assert_eq!(
            verify_packaged_product_install(&current_product, ClientActivationTarget::Codex)
                .unwrap()
                .source
                .artifact
                .version,
            "2.0.0-alpha.2"
        );
    }

    #[test]
    fn batch_install_uses_one_preview_for_both_clients() {
        let fixture = Fixture::new("batch-lifecycle");
        let product = fixture.verify().unwrap();
        let targets = [
            ClientActivationTarget::Codex,
            ClientActivationTarget::ClaudeCode,
        ];

        let preview = preview_packaged_product_batch_install(&product, &targets).unwrap();
        assert!(preview.can_apply);
        assert_eq!(preview.installs.len(), 2);
        assert!(
            preview
                .installs
                .iter()
                .all(|install| install.effect == PackagedProductInstallEffect::Install)
        );

        let commit =
            apply_packaged_product_batch_install(test_pack(), &product, &preview, NOW + 1).unwrap();
        assert_eq!(commit.installs.len(), 2);
        for target in targets {
            verify_packaged_product_install(&product, target).unwrap();
            remove_packaged_product_install(&product, target, NOW + 2).unwrap();
        }
        assert_eq!(
            preview_packaged_product_batch_install(
                &product,
                &[
                    ClientActivationTarget::ClaudeCode,
                    ClientActivationTarget::Codex,
                ],
            )
            .unwrap_err(),
            PackagedProductControlError::TargetMismatch
        );
    }

    #[test]
    fn batch_install_verifies_with_both_recognized_legacy_marketplaces_present() {
        let fixture = Fixture::new("batch-legacy-marketplaces");
        let agents = fixture.home.join(".agents");
        create_private_directory(&agents);
        let codex_plugins = agents.join("plugins");
        create_private_directory(&codex_plugins);
        fs::write(
            codex_plugins.join("marketplace.json"),
            br#"{"name":"personal","plugins":[{"name":"qiongli","source":{"source":"local","path":"./plugins/qiongli"}}]}"#,
        )
        .unwrap();

        let qiongli = fixture.home.join(".qiongli");
        create_private_directory(&qiongli);
        let plugins = qiongli.join("plugins");
        create_private_directory(&plugins);
        let claude_code = plugins.join("claude-code");
        create_private_directory(&claude_code);
        let marketplace = claude_code.join("qiongli-local");
        create_private_directory(&marketplace);
        let marketplace_metadata = marketplace.join(".claude-plugin");
        create_private_directory(&marketplace_metadata);
        fs::write(
            marketplace_metadata.join("marketplace.json"),
            br#"{"name":"qiongli-local","preserve":{"user":true},"plugins":[{"name":"qiongli","version":"1.19.0-beta.1","source":"./plugins/qiongli"}]}"#,
        )
        .unwrap();

        let product = fixture.verify().unwrap();
        let targets = [
            ClientActivationTarget::Codex,
            ClientActivationTarget::ClaudeCode,
        ];
        let preview = preview_packaged_product_batch_install(&product, &targets).unwrap();
        assert!(preview.can_apply);
        apply_packaged_product_batch_install(test_pack(), &product, &preview, NOW + 1).unwrap();

        for target in targets {
            verify_packaged_product_install(&product, target).unwrap();
        }
        let claude_marketplace: serde_json::Value = serde_json::from_slice(
            &fs::read(marketplace_metadata.join("marketplace.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(claude_marketplace["preserve"]["user"], true);
        assert_eq!(
            claude_marketplace["plugins"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["qiongli", "qiongli-next"]
        );
    }

    #[test]
    fn receipt_owned_source_without_registration_is_repaired() {
        let fixture = Fixture::new("repair");
        let product = fixture.verify().unwrap();
        let capability = product.capability(ClientActivationTarget::Codex).unwrap();
        let target = prepare_native_candidate_plugin_source_target(
            product.home(),
            ClientActivationTarget::Codex,
        )
        .unwrap();
        materialize_packaged_product_plugin_source_with_overrides(
            test_pack(),
            ClientActivationTarget::Codex,
            capability.grant(),
            product.current_executable(),
            &target,
            None,
        )
        .unwrap();
        let preview =
            preview_packaged_product_install(&product, ClientActivationTarget::Codex).unwrap();
        assert_eq!(preview.effect, PackagedProductInstallEffect::Repair);
        apply_packaged_product_install(test_pack(), &product, &preview, NOW + 1).unwrap();
        verify_packaged_product_install(&product, ClientActivationTarget::Codex).unwrap();
    }

    #[test]
    fn changed_client_state_after_preview_is_rejected_before_product_write() {
        let fixture = Fixture::new("changed-state");
        let product = fixture.verify().unwrap();
        let preview =
            preview_packaged_product_install(&product, ClientActivationTarget::Codex).unwrap();
        fs::write(fixture.home.join(".agents"), b"unsafe-parent").unwrap();
        assert_eq!(
            apply_packaged_product_install(test_pack(), &product, &preview, NOW + 1).unwrap_err(),
            PackagedProductControlError::ClientUnavailable
        );
        assert!(
            !fixture
                .home
                .join(".qiongli/plugins/codex/qiongli-next")
                .exists()
        );
    }

    fn signed_grant(
        key: &SigningKey,
        artifact: ArtifactIdentityV1,
        target: ClientActivationTarget,
        binary_sha256: &str,
    ) -> SignedLaunchGrantV1 {
        let grant = LaunchGrantV1 {
            schema_version: 1,
            generation: 7,
            artifact,
            binary_sha256: binary_sha256.to_string(),
            resource_pack_sha256: test_pack().pack_sha256().to_string(),
            allowed_modes: target.allowed_grant_modes().to_vec(),
            integration_scopes: vec![target.integration_scope()],
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        };
        let signature = key.sign(&launch_grant_signing_bytes(&grant).unwrap());
        SignedLaunchGrantV1 {
            grant,
            signature: GrantSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "product-launch-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        }
    }

    fn authority(release: &SigningKey, launch: &SigningKey) -> NativeReleaseAuthority {
        let value = json!({
            "schema_version": 1,
            "channel": "alpha",
            "minimum_release_generation": 7,
            "minimum_launch_grant_generation": 7,
            "release_keys": [{
                "key_id": "product-release-key",
                "public_key_hex": encode_hex(&release.verifying_key().to_bytes()),
                "minimum_generation": 7,
                "maximum_generation_exclusive": 8
            }],
            "launch_grant_keys": [{
                "key_id": "product-launch-key",
                "public_key_hex": encode_hex(&launch.verifying_key().to_bytes())
            }]
        });
        NativeReleaseAuthority::from_json(&serde_json_canonicalizer::to_vec(&value).unwrap())
            .unwrap()
    }

    fn test_pack() -> &'static qiongli_content::LoadedResourcePack<'static> {
        let built = BUILT_PACK.get_or_init(|| {
            let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/qiongli-product-control-pack-source");
            let _ = fs::remove_dir_all(&source);
            for directory in [
                ".claude-plugin",
                ".codex-plugin",
                "distribution",
                "mcp-contracts",
                "roles",
                "schemas",
                "skills",
                "standards",
                "subjects",
                "templates",
                "venue-profiles",
                "workflow",
            ] {
                fs::create_dir_all(source.join(directory)).unwrap();
                match directory {
                    ".codex-plugin" => fs::write(
                        source.join(".codex-plugin/plugin.json"),
                        br#"{"name":"qiongli"}"#,
                    )
                    .unwrap(),
                    ".claude-plugin" => fs::write(
                        source.join(".claude-plugin/plugin.json"),
                        br#"{"name":"qiongli"}"#,
                    )
                    .unwrap(),
                    "workflow" => fs::write(
                        source.join("workflow/SKILL.md"),
                        b"---\nname: qiongli\ndescription: test\n---\n",
                    )
                    .unwrap(),
                    _ => fs::write(source.join(directory).join("entry.txt"), directory).unwrap(),
                }
            }
            fs::write(source.join("skills-core.md"), b"core\n").unwrap();
            fs::write(source.join("skills-summary.md"), b"summary\n").unwrap();
            let resources = collect_canonical_sources(&source).unwrap();
            build_resource_pack(
                &ResourcePackBuildMetadata {
                    pack_id: "qiongli-core".to_string(),
                    content_version: "1.19.0-beta.1".to_string(),
                    source_commit: SOURCE_COMMIT.to_string(),
                    compatible_product: CompatibleProduct {
                        minimum: VERSION.to_string(),
                        maximum_exclusive: "3.0.0".to_string(),
                    },
                },
                &resources,
            )
            .inspect(|_| {
                let _ = fs::remove_dir_all(&source);
            })
            .unwrap()
        });
        LOADED_PACK
            .get_or_init(|| load_resource_pack(built.core_bytes(), built.pack_sha256()).unwrap())
    }

    fn png_stub() -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, 256, 256);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&vec![0; 256 * 256 * 4]).unwrap();
        drop(writer);
        bytes
    }

    fn companion_stub() -> crate::VerifiedZoteroCompanionArtifact {
        let manifest = format!(
            "{{\"manifest_version\":2,\"name\":\"{}\",\"version\":\"0.3.0\",\"applications\":{{\"zotero\":{{\"id\":\"{}\",\"update_url\":\"{}\",\"strict_min_version\":\"{}\",\"strict_max_version\":\"{}\"}}}}}}",
            crate::ZOTERO_COMPANION_DISPLAY_NAME,
            crate::ZOTERO_COMPANION_ID,
            crate::ZOTERO_COMPANION_UPDATE_URL,
            crate::ZOTERO_COMPANION_ZOTERO_MIN_VERSION,
            crate::ZOTERO_COMPANION_ZOTERO_MAX_VERSION,
        );
        crate::compose_zotero_companion_artifact(&[
            crate::ZoteroCompanionSourceEntry {
                path: "README.md",
                bytes: b"# Companion\n",
            },
            crate::ZoteroCompanionSourceEntry {
                path: "bootstrap.js",
                bytes: b"const response = { version: \"0.3.0\", endpoint_version: \"2\" };",
            },
            crate::ZoteroCompanionSourceEntry {
                path: "chrome/content/qiongli-bridge.js",
                bytes: b"const response = { version: \"0.3.0\", endpoint_version: \"2\" };",
            },
            crate::ZoteroCompanionSourceEntry {
                path: "manifest.json",
                bytes: manifest.as_bytes(),
            },
        ])
        .unwrap()
    }

    fn executable_bytes(os: OperatingSystem, suffix: &[u8]) -> Vec<u8> {
        let mut bytes = match os {
            OperatingSystem::Macos => b"\xcf\xfa\xed\xfe".to_vec(),
            OperatingSystem::Windows => b"MZ".to_vec(),
            OperatingSystem::Linux => b"\x7fELF".to_vec(),
        };
        bytes.extend_from_slice(suffix);
        bytes
    }

    fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(N * 2);
        for byte in bytes {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    #[cfg(unix)]
    fn create_private_directory(path: &Path) {
        use std::os::unix::fs::DirBuilderExt;
        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700).create(path).unwrap();
    }

    #[cfg(windows)]
    fn create_private_directory(path: &Path) {
        qiongli_windows_security::create_owner_only_directory(path).unwrap();
    }

    #[cfg(unix)]
    fn set_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    #[cfg(not(unix))]
    fn set_executable(_path: &Path) {}
}
