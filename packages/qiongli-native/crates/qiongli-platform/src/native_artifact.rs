use std::collections::BTreeSet;
use std::fmt::{self, Debug, Display, Formatter};
#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use qiongli_content::{
    LoadedResourcePack, LogicalMode, MaterializationAuthorization, MaterializationTarget,
    ProfileId, approve_materialization_target,
};
use same_file::Handle;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind, OperatingSystem, ProductId,
    ReleaseChannel,
};

pub const NATIVE_ARTIFACT_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_ARTIFACT_MANIFEST_FILE: &str = ".qiongli-native-artifact.json";

const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_BINARY_BYTES: u64 = 128 * 1024 * 1024;
const MAX_ARTIFACT_ID_BYTES: usize = 128;
const CONTENT_ROOT_DOMAIN: &[u8] = b"qiongli-native-artifact-content-root-v1\0";
const TARGET_LOCK_FILE: &str = ".qiongli.qiongli-native-artifact.lock";
static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct NativeArtifactTarget {
    inner: MaterializationTarget,
    artifact: ArtifactIdentityV1,
    artifact_id: String,
}

impl NativeArtifactTarget {
    #[must_use]
    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    #[must_use]
    pub fn authorization(&self) -> MaterializationAuthorization {
        self.inner.authorization()
    }

    #[must_use]
    pub const fn artifact(&self) -> &ArtifactIdentityV1 {
        &self.artifact
    }

    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }
}

impl Debug for NativeArtifactTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeArtifactTarget")
            .field("path", &"<approved-native-artifact>")
            .field("artifact", &self.artifact)
            .field("authorization", &self.authorization())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeArtifactRecordType {
    QiongliNativeArtifact,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeArtifactStatus {
    AssembledUnpublished,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeArtifactContentV1 {
    pub profile: ProfileId,
    pub pack_id: String,
    pub content_version: String,
    pub source_commit: String,
    pub pack_sha256: String,
    pub content_root_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeArtifactEntryV1 {
    pub path: String,
    pub mode: LogicalMode,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeArtifactManifestV1 {
    pub schema_version: u32,
    pub record_type: NativeArtifactRecordType,
    pub status: NativeArtifactStatus,
    pub artifact: ArtifactIdentityV1,
    pub content: NativeArtifactContentV1,
    pub artifact_content_root_sha256: String,
    pub binary_path: String,
    pub binary_sha256: String,
    pub entries: Vec<NativeArtifactEntryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedNativeArtifact {
    manifest: NativeArtifactManifestV1,
    manifest_sha256: String,
}

impl VerifiedNativeArtifact {
    #[must_use]
    pub const fn manifest(&self) -> &NativeArtifactManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }
}

pub(crate) struct NativeArtifactPayload {
    pub(crate) verified: VerifiedNativeArtifact,
    pub(crate) manifest_bytes: Vec<u8>,
    pub(crate) binary_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeArtifactError {
    UnsupportedPlatform,
    InvalidIdentity,
    InvalidTarget,
    UnsafeTarget,
    TargetExists,
    TargetBusy,
    SourceBinaryInvalid,
    SourceBinaryTooLarge,
    ResourcePackInvalid,
    ArtifactMissing,
    ManifestMissing,
    ManifestInvalid,
    ArtifactDrift,
    PersistenceFailed(io::ErrorKind),
    CommitFailed(io::ErrorKind),
    CommittedPersistenceFailed(io::ErrorKind),
    CommittedVerificationFailed,
}

impl NativeArtifactError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "native-artifact-platform-unsupported",
            Self::InvalidIdentity => "native-artifact-identity-invalid",
            Self::InvalidTarget => "native-artifact-target-invalid",
            Self::UnsafeTarget => "native-artifact-target-unsafe",
            Self::TargetExists => "native-artifact-target-exists",
            Self::TargetBusy => "native-artifact-target-busy",
            Self::SourceBinaryInvalid => "native-artifact-binary-invalid",
            Self::SourceBinaryTooLarge => "native-artifact-binary-too-large",
            Self::ResourcePackInvalid => "native-artifact-pack-invalid",
            Self::ArtifactMissing => "native-artifact-missing",
            Self::ManifestMissing => "native-artifact-manifest-missing",
            Self::ManifestInvalid => "native-artifact-manifest-invalid",
            Self::ArtifactDrift => "native-artifact-drift",
            Self::PersistenceFailed(_) => "native-artifact-persistence-failed",
            Self::CommitFailed(_) => "native-artifact-commit-failed",
            Self::CommittedPersistenceFailed(_) => "native-artifact-committed-persistence-failed",
            Self::CommittedVerificationFailed => "native-artifact-committed-verification-failed",
        }
    }
}

impl Display for NativeArtifactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())?;
        match self {
            Self::PersistenceFailed(kind)
            | Self::CommitFailed(kind)
            | Self::CommittedPersistenceFailed(kind) => write!(formatter, " ({kind:?})"),
            _ => Ok(()),
        }
    }
}

impl std::error::Error for NativeArtifactError {}

pub fn current_target_native_artifact_identity(
    version: impl Into<String>,
    channel: ReleaseChannel,
) -> Result<ArtifactIdentityV1, NativeArtifactError> {
    let os = OperatingSystem::current().ok_or(NativeArtifactError::UnsupportedPlatform)?;
    let arch = Architecture::current().ok_or(NativeArtifactError::UnsupportedPlatform)?;
    let artifact = ArtifactIdentityV1 {
        product: ProductId::Qiongli,
        version: version.into(),
        channel,
        profile: CapabilityProfile::Lite,
        os,
        arch,
        installer_kind: InstallerKind::PortableArchive,
    };
    validate_artifact_identity(&artifact)?;
    Ok(artifact)
}

pub fn native_artifact_id(artifact: &ArtifactIdentityV1) -> Result<String, NativeArtifactError> {
    validate_artifact_identity(artifact)?;
    let value = format!(
        "{}-{}-{}-{}-{}-{}-{}",
        product_id(artifact.product),
        artifact.version,
        release_channel(artifact.channel),
        capability_profile(artifact.profile),
        operating_system(artifact.os),
        architecture(artifact.arch),
        installer_kind(artifact.installer_kind),
    );
    if value.len() > MAX_ARTIFACT_ID_BYTES {
        return Err(NativeArtifactError::InvalidIdentity);
    }
    Ok(value)
}

pub fn native_artifact_binary_path(
    artifact: &ArtifactIdentityV1,
) -> Result<&'static str, NativeArtifactError> {
    validate_artifact_identity(artifact)?;
    Ok(binary_relative_path(artifact.os))
}

/// Approves a caller-selected target at a trusted CLI, UI, release, or test boundary.
///
/// Model-generated and MCP-provided paths must not be passed to this function.
pub fn approve_native_artifact_target(
    path: impl AsRef<Path>,
    artifact: &ArtifactIdentityV1,
) -> Result<NativeArtifactTarget, NativeArtifactError> {
    let artifact_id = native_artifact_id(artifact)?;
    let path = path.as_ref();
    if path.file_name().and_then(|leaf| leaf.to_str()) != Some(artifact_id.as_str()) {
        return Err(NativeArtifactError::InvalidTarget);
    }
    let inner =
        approve_materialization_target(path).map_err(|_| NativeArtifactError::UnsafeTarget)?;
    validate_target_parent_security(inner.path())?;
    Ok(NativeArtifactTarget {
        inner,
        artifact: artifact.clone(),
        artifact_id,
    })
}

pub fn compose_native_artifact(
    pack: &LoadedResourcePack<'_>,
    artifact: &ArtifactIdentityV1,
    source_binary: impl AsRef<Path>,
    target: &NativeArtifactTarget,
) -> Result<VerifiedNativeArtifact, NativeArtifactError> {
    validate_artifact_identity(artifact)?;
    validate_resource_pack(pack)?;
    if target.artifact != *artifact || target.artifact_id != native_artifact_id(artifact)? {
        return Err(NativeArtifactError::InvalidTarget);
    }
    revalidate_target(target)?;
    if path_metadata(target.path())?.is_some() {
        return Err(NativeArtifactError::TargetExists);
    }

    let binary_bytes = read_source_binary(source_binary.as_ref())?;
    let binary_path = binary_relative_path(artifact.os).to_string();
    let binary_sha256 = sha256_hex(&binary_bytes);
    let binary_size =
        u64::try_from(binary_bytes.len()).map_err(|_| NativeArtifactError::SourceBinaryTooLarge)?;
    let entries = vec![NativeArtifactEntryV1 {
        path: binary_path.clone(),
        mode: LogicalMode::Executable,
        size_bytes: binary_size,
        sha256: binary_sha256.clone(),
    }];
    let pack_manifest = pack.manifest();
    let manifest = NativeArtifactManifestV1 {
        schema_version: NATIVE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
        record_type: NativeArtifactRecordType::QiongliNativeArtifact,
        status: NativeArtifactStatus::AssembledUnpublished,
        artifact: artifact.clone(),
        content: NativeArtifactContentV1 {
            profile: ProfileId::MarketplaceLite,
            pack_id: pack_manifest.pack_id.clone(),
            content_version: pack_manifest.content_version.clone(),
            source_commit: pack_manifest.source_commit.clone(),
            pack_sha256: pack.pack_sha256().to_string(),
            content_root_sha256: pack_manifest.content_root_sha256.clone(),
        },
        artifact_content_root_sha256: artifact_content_root(&entries),
        binary_path,
        binary_sha256,
        entries,
    };
    validate_manifest_shape(&manifest)?;
    let manifest_bytes = canonical_json(&manifest)?;

    commit_native_artifact_payload(pack, artifact, &manifest_bytes, &binary_bytes, target)
}

pub(crate) fn commit_native_artifact_payload(
    pack: &LoadedResourcePack<'_>,
    artifact: &ArtifactIdentityV1,
    manifest_bytes: &[u8],
    binary_bytes: &[u8],
    target: &NativeArtifactTarget,
) -> Result<VerifiedNativeArtifact, NativeArtifactError> {
    validate_artifact_identity(artifact)?;
    validate_resource_pack(pack)?;
    if target.artifact != *artifact || target.artifact_id != native_artifact_id(artifact)? {
        return Err(NativeArtifactError::InvalidTarget);
    }
    let expected =
        verify_native_artifact_payload(pack, target.artifact_id(), manifest_bytes, binary_bytes)?;
    if expected.manifest.artifact != *artifact {
        return Err(NativeArtifactError::ArtifactDrift);
    }

    let _lock = TargetLock::acquire(target)?;
    revalidate_target(target)?;
    if path_metadata(target.path())?.is_some() {
        return Err(NativeArtifactError::TargetExists);
    }
    let parent = target
        .path()
        .parent()
        .ok_or(NativeArtifactError::InvalidTarget)?;
    let staging = create_staging_directory(parent)?;
    let cleanup = DirectoryCleanup::new(staging.clone());
    write_artifact_tree(&staging, binary_bytes, manifest_bytes, artifact.os)?;
    let staged = verify_artifact_tree(&staging, target.artifact_id(), pack)?;
    if staged != expected {
        return Err(NativeArtifactError::ArtifactDrift);
    }

    revalidate_target(target)?;
    if path_metadata(target.path())?.is_some() {
        return Err(NativeArtifactError::TargetExists);
    }
    rename_no_replace(&staging, target.path())?;
    cleanup.disarm();
    sync_directory(parent).map_err(|error| match error {
        NativeArtifactError::PersistenceFailed(kind) => {
            NativeArtifactError::CommittedPersistenceFailed(kind)
        }
        other => other,
    })?;

    verify_artifact_tree(target.path(), target.artifact_id(), pack)
        .map_err(|_| NativeArtifactError::CommittedVerificationFailed)
}

pub fn verify_native_artifact(
    pack: &LoadedResourcePack<'_>,
    target: &NativeArtifactTarget,
) -> Result<VerifiedNativeArtifact, NativeArtifactError> {
    Ok(read_native_artifact_payload(pack, target)?.verified)
}

pub(crate) fn read_native_artifact_payload(
    pack: &LoadedResourcePack<'_>,
    target: &NativeArtifactTarget,
) -> Result<NativeArtifactPayload, NativeArtifactError> {
    validate_resource_pack(pack)?;
    revalidate_target(target)?;
    let payload = read_artifact_tree_payload(target.path(), target.artifact_id())?;
    verify_manifest_pack(pack, &payload.verified.manifest)?;
    if payload.verified.manifest.artifact != target.artifact {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(payload)
}

fn validate_artifact_identity(artifact: &ArtifactIdentityV1) -> Result<(), NativeArtifactError> {
    artifact
        .validate()
        .map_err(|_| NativeArtifactError::InvalidIdentity)?;
    let current_os = OperatingSystem::current().ok_or(NativeArtifactError::UnsupportedPlatform)?;
    let current_arch = Architecture::current().ok_or(NativeArtifactError::UnsupportedPlatform)?;
    if artifact.product != ProductId::Qiongli
        || artifact.profile != CapabilityProfile::Lite
        || artifact.installer_kind != InstallerKind::PortableArchive
        || artifact.os != current_os
        || artifact.arch != current_arch
    {
        return Err(NativeArtifactError::InvalidIdentity);
    }
    Ok(())
}

fn validate_resource_pack(pack: &LoadedResourcePack<'_>) -> Result<(), NativeArtifactError> {
    pack.manifest()
        .resolve_profile("marketplace-lite")
        .map_err(|_| NativeArtifactError::ResourcePackInvalid)?;
    Ok(())
}

fn validate_manifest_shape(manifest: &NativeArtifactManifestV1) -> Result<(), NativeArtifactError> {
    validate_artifact_identity(&manifest.artifact)
        .map_err(|_| NativeArtifactError::ManifestInvalid)?;
    if manifest.schema_version != NATIVE_ARTIFACT_MANIFEST_SCHEMA_VERSION
        || manifest.record_type != NativeArtifactRecordType::QiongliNativeArtifact
        || manifest.status != NativeArtifactStatus::AssembledUnpublished
        || manifest.content.profile != ProfileId::MarketplaceLite
        || !valid_identifier(&manifest.content.pack_id, 64)
        || manifest.content.content_version.len() > 64
        || Version::parse(&manifest.content.content_version).is_err()
        || !is_lower_hex(&manifest.content.source_commit, 40)
        || !is_lower_hex(&manifest.content.pack_sha256, 64)
        || !is_lower_hex(&manifest.content.content_root_sha256, 64)
        || !is_lower_hex(&manifest.artifact_content_root_sha256, 64)
        || !is_lower_hex(&manifest.binary_sha256, 64)
        || manifest.binary_path != binary_relative_path(manifest.artifact.os)
        || manifest.entries.len() != 1
    {
        return Err(NativeArtifactError::ManifestInvalid);
    }
    let entry = &manifest.entries[0];
    if entry.path != manifest.binary_path
        || entry.mode != LogicalMode::Executable
        || entry.size_bytes == 0
        || entry.size_bytes > MAX_BINARY_BYTES
        || entry.sha256 != manifest.binary_sha256
        || !is_lower_hex(&entry.sha256, 64)
        || artifact_content_root(&manifest.entries) != manifest.artifact_content_root_sha256
    {
        return Err(NativeArtifactError::ManifestInvalid);
    }
    native_artifact_id(&manifest.artifact).map_err(|_| NativeArtifactError::ManifestInvalid)?;
    Ok(())
}

fn verify_artifact_tree(
    root: &Path,
    expected_artifact_id: &str,
    expected_pack: &LoadedResourcePack<'_>,
) -> Result<VerifiedNativeArtifact, NativeArtifactError> {
    let payload = read_artifact_tree_payload(root, expected_artifact_id)?;
    verify_manifest_pack(expected_pack, &payload.verified.manifest)?;
    Ok(payload.verified)
}

fn read_artifact_tree_payload(
    root: &Path,
    expected_artifact_id: &str,
) -> Result<NativeArtifactPayload, NativeArtifactError> {
    verify_directory(root, true)?;
    let root_entries = directory_entries(root)?;
    if root_entries
        != BTreeSet::from([NATIVE_ARTIFACT_MANIFEST_FILE.to_string(), "bin".to_string()])
    {
        return Err(NativeArtifactError::ArtifactDrift);
    }

    let manifest_path = root.join(NATIVE_ARTIFACT_MANIFEST_FILE);
    let manifest_bytes = read_bounded_managed_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        LogicalMode::Regular,
        NativeArtifactError::ManifestMissing,
        NativeArtifactError::ManifestInvalid,
    )?;
    let manifest = parse_manifest_bytes(expected_artifact_id, &manifest_bytes)?;

    let binary_relative = Path::new(&manifest.binary_path);
    let binary_name = binary_relative
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(NativeArtifactError::ManifestInvalid)?;
    let binary_parent = root.join("bin");
    verify_directory(&binary_parent, false)?;
    if directory_entries(&binary_parent)? != BTreeSet::from([binary_name.to_string()]) {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    let binary_path = root.join(binary_relative);
    let binary_bytes = read_bounded_managed_file(
        &binary_path,
        MAX_BINARY_BYTES,
        LogicalMode::Executable,
        NativeArtifactError::ArtifactDrift,
        NativeArtifactError::ArtifactDrift,
    )?;
    let verified = verify_manifest_binary(manifest, &manifest_bytes, &binary_bytes)?;

    Ok(NativeArtifactPayload {
        verified,
        manifest_bytes,
        binary_bytes,
    })
}

pub(crate) fn verify_native_artifact_payload(
    expected_pack: &LoadedResourcePack<'_>,
    expected_artifact_id: &str,
    manifest_bytes: &[u8],
    binary_bytes: &[u8],
) -> Result<VerifiedNativeArtifact, NativeArtifactError> {
    let manifest = verify_manifest_bytes(expected_pack, expected_artifact_id, manifest_bytes)?;
    verify_manifest_binary(manifest, manifest_bytes, binary_bytes)
}

fn verify_manifest_binary(
    manifest: NativeArtifactManifestV1,
    manifest_bytes: &[u8],
    binary_bytes: &[u8],
) -> Result<VerifiedNativeArtifact, NativeArtifactError> {
    let entry = &manifest.entries[0];
    if binary_bytes.len() as u64 != entry.size_bytes || sha256_hex(binary_bytes) != entry.sha256 {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(VerifiedNativeArtifact {
        manifest_sha256: sha256_hex(manifest_bytes),
        manifest,
    })
}

fn verify_manifest_bytes(
    expected_pack: &LoadedResourcePack<'_>,
    expected_artifact_id: &str,
    manifest_bytes: &[u8],
) -> Result<NativeArtifactManifestV1, NativeArtifactError> {
    let manifest = parse_manifest_bytes(expected_artifact_id, manifest_bytes)?;
    verify_manifest_pack(expected_pack, &manifest)?;
    Ok(manifest)
}

fn parse_manifest_bytes(
    expected_artifact_id: &str,
    manifest_bytes: &[u8],
) -> Result<NativeArtifactManifestV1, NativeArtifactError> {
    if manifest_bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(NativeArtifactError::ManifestInvalid);
    }
    let manifest: NativeArtifactManifestV1 =
        serde_json::from_slice(manifest_bytes).map_err(|_| NativeArtifactError::ManifestInvalid)?;
    if canonical_json(&manifest)? != manifest_bytes {
        return Err(NativeArtifactError::ManifestInvalid);
    }
    validate_manifest_shape(&manifest)?;
    if native_artifact_id(&manifest.artifact)? != expected_artifact_id {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(manifest)
}

fn verify_manifest_pack(
    expected_pack: &LoadedResourcePack<'_>,
    manifest: &NativeArtifactManifestV1,
) -> Result<(), NativeArtifactError> {
    validate_resource_pack(expected_pack)?;
    let pack_manifest = expected_pack.manifest();
    if manifest.content.profile != ProfileId::MarketplaceLite
        || manifest.content.pack_id != pack_manifest.pack_id
        || manifest.content.content_version != pack_manifest.content_version
        || manifest.content.source_commit != pack_manifest.source_commit
        || manifest.content.pack_sha256 != expected_pack.pack_sha256()
        || manifest.content.content_root_sha256 != pack_manifest.content_root_sha256
    {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(())
}

/// Checks receipt-owned bytes only; this does not establish signed launch authority.
pub(crate) fn verify_receipt_owned_native_artifact(
    target: &NativeArtifactTarget,
    expected_manifest_sha256: &str,
) -> Result<VerifiedNativeArtifact, NativeArtifactError> {
    revalidate_target(target)?;
    let verified = read_artifact_tree_payload(target.path(), target.artifact_id())?.verified;
    if verified.manifest.artifact != target.artifact
        || verified.manifest_sha256 != expected_manifest_sha256
    {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(verified)
}

fn write_artifact_tree(
    root: &Path,
    binary_bytes: &[u8],
    manifest_bytes: &[u8],
    os: OperatingSystem,
) -> Result<(), NativeArtifactError> {
    let bin = root.join("bin");
    create_private_directory(&bin)?;
    write_new_file(
        &root.join(binary_relative_path(os)),
        binary_bytes,
        LogicalMode::Executable,
    )?;
    write_new_file(
        &root.join(NATIVE_ARTIFACT_MANIFEST_FILE),
        manifest_bytes,
        LogicalMode::Regular,
    )?;
    finalize_directory(&bin)?;
    sync_directory(&bin)?;
    finalize_directory(root)?;
    sync_directory(root)
}

fn read_source_binary(path: &Path) -> Result<Vec<u8>, NativeArtifactError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| NativeArtifactError::SourceBinaryInvalid)?;
    if metadata.file_type().is_symlink()
        || is_reparse_point(&metadata)
        || !metadata.is_file()
        || metadata.len() == 0
    {
        return Err(NativeArtifactError::SourceBinaryInvalid);
    }
    if metadata.len() > MAX_BINARY_BYTES {
        return Err(NativeArtifactError::SourceBinaryTooLarge);
    }
    verify_single_link(path, &metadata).map_err(|_| NativeArtifactError::SourceBinaryInvalid)?;
    validate_source_executable_mode(&metadata)?;
    let mut file = File::open(path).map_err(|_| NativeArtifactError::SourceBinaryInvalid)?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len()).map_err(|_| NativeArtifactError::SourceBinaryTooLarge)?,
    );
    file.read_to_end(&mut bytes)
        .map_err(|_| NativeArtifactError::SourceBinaryInvalid)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(NativeArtifactError::SourceBinaryInvalid);
    }
    Ok(bytes)
}

#[cfg(unix)]
fn validate_source_executable_mode(metadata: &Metadata) -> Result<(), NativeArtifactError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    if metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o111 == 0
    {
        return Err(NativeArtifactError::SourceBinaryInvalid);
    }
    Ok(())
}

#[cfg(windows)]
fn validate_source_executable_mode(_metadata: &Metadata) -> Result<(), NativeArtifactError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn validate_source_executable_mode(_metadata: &Metadata) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

fn directory_entries(path: &Path) -> Result<BTreeSet<String>, NativeArtifactError> {
    let entries =
        fs::read_dir(path).map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))?;
    let mut names = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))?;
        let name = entry
            .file_name()
            .to_str()
            .map(str::to_string)
            .ok_or(NativeArtifactError::ArtifactDrift)?;
        if !names.insert(name) {
            return Err(NativeArtifactError::ArtifactDrift);
        }
    }
    Ok(names)
}

fn read_bounded_managed_file(
    path: &Path,
    limit: u64,
    mode: LogicalMode,
    missing: NativeArtifactError,
    invalid: NativeArtifactError,
) -> Result<Vec<u8>, NativeArtifactError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            missing
        } else {
            NativeArtifactError::PersistenceFailed(error.kind())
        }
    })?;
    if metadata.len() > limit {
        return Err(invalid);
    }
    verify_managed_file_with_metadata(path, &metadata, mode)?;
    let mut file =
        File::open(path).map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))?;
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).map_err(|_| invalid)?);
    Read::by_ref(&mut file)
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))?;
    if bytes.len() as u64 > limit || bytes.len() as u64 != metadata.len() {
        return Err(invalid);
    }
    Ok(bytes)
}

fn verify_managed_file_with_metadata(
    path: &Path,
    metadata: &Metadata,
    mode: LogicalMode,
) -> Result<(), NativeArtifactError> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) || !metadata.is_file() {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    verify_managed_file_security(path, metadata)?;
    verify_single_link(path, metadata)?;
    verify_file_mode(metadata, mode)
}

#[cfg(unix)]
fn verify_managed_file_security(
    _path: &Path,
    metadata: &Metadata,
) -> Result<(), NativeArtifactError> {
    use std::os::unix::fs::MetadataExt;
    if metadata.uid() != rustix::process::geteuid().as_raw() {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(())
}

#[cfg(windows)]
fn verify_managed_file_security(
    path: &Path,
    _metadata: &Metadata,
) -> Result<(), NativeArtifactError> {
    qiongli_windows_security::open_owner_only_file(path)
        .map(|_| ())
        .map_err(|_| NativeArtifactError::ArtifactDrift)
}

#[cfg(not(any(unix, windows)))]
fn verify_managed_file_security(
    _path: &Path,
    _metadata: &Metadata,
) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

#[cfg(unix)]
fn verify_single_link(_path: &Path, metadata: &Metadata) -> Result<(), NativeArtifactError> {
    use std::os::unix::fs::MetadataExt;
    if metadata.nlink() != 1 {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(())
}

#[cfg(windows)]
fn verify_single_link(path: &Path, _metadata: &Metadata) -> Result<(), NativeArtifactError> {
    let file =
        File::open(path).map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))?;
    let facts = qiongli_windows_security::handle_facts(&file)
        .map_err(|_| NativeArtifactError::ArtifactDrift)?;
    if facts.number_of_links != 1 {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn verify_single_link(_path: &Path, _metadata: &Metadata) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

#[cfg(unix)]
fn verify_file_mode(metadata: &Metadata, mode: LogicalMode) -> Result<(), NativeArtifactError> {
    use std::os::unix::fs::PermissionsExt;
    let expected = match mode {
        LogicalMode::Regular => 0o644,
        LogicalMode::Executable => 0o755,
    };
    if metadata.permissions().mode() & 0o777 != expected {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(())
}

#[cfg(windows)]
fn verify_file_mode(_metadata: &Metadata, _mode: LogicalMode) -> Result<(), NativeArtifactError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn verify_file_mode(_metadata: &Metadata, _mode: LogicalMode) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

fn write_new_file(path: &Path, bytes: &[u8], mode: LogicalMode) -> Result<(), NativeArtifactError> {
    let mut file = create_private_new_file(path)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))?;
    drop(file);
    set_file_mode(path, mode)?;
    sync_file_mode(path)
}

#[cfg(unix)]
fn sync_file_mode(path: &Path) -> Result<(), NativeArtifactError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))
}

#[cfg(not(unix))]
fn sync_file_mode(_path: &Path) -> Result<(), NativeArtifactError> {
    Ok(())
}

#[cfg(unix)]
fn create_private_new_file(path: &Path) -> Result<File, NativeArtifactError> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn create_private_new_file(path: &Path) -> Result<File, NativeArtifactError> {
    qiongli_windows_security::create_owner_only_new_file(path).map_err(|error| {
        NativeArtifactError::PersistenceFailed(
            error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
        )
    })
}

#[cfg(not(any(unix, windows)))]
fn create_private_new_file(_path: &Path) -> Result<File, NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

#[cfg(unix)]
fn set_file_mode(path: &Path, mode: LogicalMode) -> Result<(), NativeArtifactError> {
    use std::os::unix::fs::PermissionsExt;
    let bits = match mode {
        LogicalMode::Regular => 0o644,
        LogicalMode::Executable => 0o755,
    };
    fs::set_permissions(path, fs::Permissions::from_mode(bits))
        .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn set_file_mode(_path: &Path, _mode: LogicalMode) -> Result<(), NativeArtifactError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn set_file_mode(_path: &Path, _mode: LogicalMode) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), NativeArtifactError> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn create_private_directory(path: &Path) -> Result<(), NativeArtifactError> {
    qiongli_windows_security::create_owner_only_directory(path)
        .map(|_| ())
        .map_err(|error| {
            NativeArtifactError::PersistenceFailed(
                error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(_path: &Path) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

#[cfg(unix)]
fn finalize_directory(path: &Path) -> Result<(), NativeArtifactError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn finalize_directory(_path: &Path) -> Result<(), NativeArtifactError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn finalize_directory(_path: &Path) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

fn verify_directory(path: &Path, root: bool) -> Result<(), NativeArtifactError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if root && error.kind() == io::ErrorKind::NotFound {
            NativeArtifactError::ArtifactMissing
        } else {
            NativeArtifactError::PersistenceFailed(error.kind())
        }
    })?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_dir() {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    verify_directory_security(path, &metadata)
}

#[cfg(unix)]
fn verify_directory_security(_path: &Path, metadata: &Metadata) -> Result<(), NativeArtifactError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    if metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o777 != 0o755
    {
        return Err(NativeArtifactError::ArtifactDrift);
    }
    Ok(())
}

#[cfg(windows)]
fn verify_directory_security(path: &Path, _metadata: &Metadata) -> Result<(), NativeArtifactError> {
    qiongli_windows_security::open_owner_only_directory(path)
        .map(|_| ())
        .map_err(|_| NativeArtifactError::ArtifactDrift)
}

#[cfg(not(any(unix, windows)))]
fn verify_directory_security(
    _path: &Path,
    _metadata: &Metadata,
) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

fn revalidate_target(target: &NativeArtifactTarget) -> Result<(), NativeArtifactError> {
    let refreshed = approve_native_artifact_target(target.path(), target.artifact())?;
    if refreshed.authorization() != target.authorization()
        || refreshed.artifact_id() != target.artifact_id()
    {
        return Err(NativeArtifactError::UnsafeTarget);
    }
    Ok(())
}

#[cfg(windows)]
fn validate_target_parent_security(path: &Path) -> Result<(), NativeArtifactError> {
    let parent = path.parent().ok_or(NativeArtifactError::InvalidTarget)?;
    qiongli_windows_security::open_owner_only_directory(parent)
        .map(|_| ())
        .map_err(|_| NativeArtifactError::UnsafeTarget)
}

#[cfg(not(windows))]
fn validate_target_parent_security(_path: &Path) -> Result<(), NativeArtifactError> {
    Ok(())
}

fn create_staging_directory(parent: &Path) -> Result<PathBuf, NativeArtifactError> {
    for _ in 0..128 {
        let path = parent.join(format!(
            ".qiongli.native-artifact-stage-{}-{}",
            std::process::id(),
            transaction_id()
        ));
        match create_private_directory(&path) {
            Ok(()) => return Ok(path),
            Err(NativeArtifactError::PersistenceFailed(io::ErrorKind::AlreadyExists)) => {}
            Err(error) => return Err(error),
        }
    }
    Err(NativeArtifactError::PersistenceFailed(
        io::ErrorKind::AlreadyExists,
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn rename_no_replace(source: &Path, destination: &Path) -> Result<(), NativeArtifactError> {
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        source,
        rustix::fs::CWD,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|error| {
        let error = io::Error::from(error);
        NativeArtifactError::CommitFailed(error.kind())
    })
}

#[cfg(windows)]
fn rename_no_replace(source: &Path, destination: &Path) -> Result<(), NativeArtifactError> {
    qiongli_windows_security::move_file_write_through(source, destination, false).map_err(|error| {
        NativeArtifactError::CommitFailed(
            error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
        )
    })
}

#[cfg(all(not(windows), not(any(target_os = "linux", target_os = "macos"))))]
fn rename_no_replace(_source: &Path, _destination: &Path) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), NativeArtifactError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), NativeArtifactError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn sync_directory(_path: &Path) -> Result<(), NativeArtifactError> {
    Err(NativeArtifactError::UnsupportedPlatform)
}

fn path_metadata(path: &Path) -> Result<Option<Metadata>, NativeArtifactError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(NativeArtifactError::PersistenceFailed(error.kind())),
    }
}

fn artifact_content_root(entries: &[NativeArtifactEntryV1]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CONTENT_ROOT_DOMAIN);
    for entry in entries {
        hash_field(&mut hasher, entry.path.as_bytes());
        hash_field(
            &mut hasher,
            match entry.mode {
                LogicalMode::Regular => b"0644",
                LogicalMode::Executable => b"0755",
            },
        );
        hasher.update(entry.size_bytes.to_be_bytes());
        hash_field(&mut hasher, entry.sha256.as_bytes());
    }
    encode_hex(&hasher.finalize())
}

fn hash_field(hasher: &mut Sha256, value: &[u8]) {
    hasher.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(value);
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, NativeArtifactError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| NativeArtifactError::ManifestInvalid)
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_identifier(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

const fn product_id(product: ProductId) -> &'static str {
    match product {
        ProductId::Qiongli => "qiongli",
    }
}

const fn release_channel(channel: ReleaseChannel) -> &'static str {
    match channel {
        ReleaseChannel::Alpha => "alpha",
        ReleaseChannel::Beta => "beta",
        ReleaseChannel::Stable => "stable",
    }
}

const fn capability_profile(profile: CapabilityProfile) -> &'static str {
    match profile {
        CapabilityProfile::SkillOnly => "skill-only",
        CapabilityProfile::Lite => "lite",
        CapabilityProfile::Full => "full",
    }
}

const fn operating_system(os: OperatingSystem) -> &'static str {
    match os {
        OperatingSystem::Macos => "macos",
        OperatingSystem::Windows => "windows",
        OperatingSystem::Linux => "linux",
    }
}

const fn architecture(arch: Architecture) -> &'static str {
    match arch {
        Architecture::Aarch64 => "aarch64",
        Architecture::X86_64 => "x86-64",
    }
}

const fn installer_kind(kind: InstallerKind) -> &'static str {
    match kind {
        InstallerKind::NativeInstaller => "native-installer",
        InstallerKind::PortableArchive => "portable-archive",
        InstallerKind::PluginBundle => "plugin-bundle",
        InstallerKind::Mcpb => "mcpb",
    }
}

const fn binary_relative_path(os: OperatingSystem) -> &'static str {
    if matches!(os, OperatingSystem::Windows) {
        "bin/qiongli.exe"
    } else {
        "bin/qiongli"
    }
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

fn transaction_id() -> u64 {
    NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed)
}

struct TargetLock {
    path: PathBuf,
    identity: Option<Handle>,
}

impl TargetLock {
    fn acquire(target: &NativeArtifactTarget) -> Result<Self, NativeArtifactError> {
        let parent = target
            .path()
            .parent()
            .ok_or(NativeArtifactError::InvalidTarget)?;
        let path = parent.join(TARGET_LOCK_FILE);
        let mut file = match create_private_new_file(&path) {
            Ok(file) => file,
            Err(NativeArtifactError::PersistenceFailed(io::ErrorKind::AlreadyExists)) => {
                return Err(NativeArtifactError::TargetBusy);
            }
            Err(error) => return Err(error),
        };
        let setup = writeln!(file, "{}", std::process::id())
            .and_then(|()| file.sync_all())
            .map_err(|error| NativeArtifactError::PersistenceFailed(error.kind()));
        drop(file);
        if let Err(error) = setup {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        let identity = Handle::from_path(&path).map_err(|error| {
            let _ = fs::remove_file(&path);
            NativeArtifactError::PersistenceFailed(error.kind())
        })?;
        Ok(Self {
            path,
            identity: Some(identity),
        })
    }
}

impl Drop for TargetLock {
    fn drop(&mut self) {
        let still_owned = self.identity.as_ref().is_some_and(|expected| {
            Handle::from_path(&self.path).is_ok_and(|current| &current == expected)
        });
        self.identity.take();
        if still_owned {
            let _ = fs::remove_file(&self.path);
        }
    }
}

struct DirectoryCleanup {
    path: PathBuf,
    armed: bool,
}

impl DirectoryCleanup {
    const fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for DirectoryCleanup {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_identity_and_paths_are_concrete_and_deterministic() {
        let artifact =
            current_target_native_artifact_identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .unwrap();
        let expected = format!(
            "qiongli-2.0.0-alpha.1-alpha-lite-{}-{}-portable-archive",
            operating_system(artifact.os),
            architecture(artifact.arch)
        );
        assert_eq!(native_artifact_id(&artifact).unwrap(), expected);
        assert_eq!(
            native_artifact_binary_path(&artifact).unwrap(),
            binary_relative_path(artifact.os)
        );
        assert!(!expected.contains("current"));
        assert!(!expected.contains("any"));
    }

    #[test]
    fn identity_rejects_wrong_channel_profile_kind_and_target() {
        let artifact =
            current_target_native_artifact_identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .unwrap();
        let mut invalid = artifact.clone();
        invalid.channel = ReleaseChannel::Beta;
        assert_eq!(
            native_artifact_id(&invalid),
            Err(NativeArtifactError::InvalidIdentity)
        );

        let mut invalid = artifact.clone();
        invalid.profile = CapabilityProfile::Full;
        assert_eq!(
            native_artifact_id(&invalid),
            Err(NativeArtifactError::InvalidIdentity)
        );

        let mut invalid = artifact.clone();
        invalid.installer_kind = InstallerKind::PluginBundle;
        assert_eq!(
            native_artifact_id(&invalid),
            Err(NativeArtifactError::InvalidIdentity)
        );

        let mut invalid = artifact;
        invalid.os = match invalid.os {
            OperatingSystem::Macos => OperatingSystem::Linux,
            OperatingSystem::Linux | OperatingSystem::Windows => OperatingSystem::Macos,
        };
        assert_eq!(
            native_artifact_id(&invalid),
            Err(NativeArtifactError::InvalidIdentity)
        );
    }

    #[test]
    fn errors_do_not_expose_paths() {
        let rendered =
            NativeArtifactError::PersistenceFailed(io::ErrorKind::PermissionDenied).to_string();
        assert_eq!(
            rendered,
            "native-artifact-persistence-failed (PermissionDenied)"
        );
        assert!(!rendered.contains('/'));
    }
}
