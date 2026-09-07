use std::fmt::{self, Debug, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use qiongli_content::LoadedResourcePack;
use serde::{Deserialize, Serialize};

use crate::grant::{decode_fixed_hex, is_lower_hex, sha256_hex, valid_identifier};
use crate::native_release::validate_release_keys;
use crate::{
    ArtifactIdentityV1, ClientActivationTarget, GrantMode, GrantVerificationContext, InstallerKind,
    IntegrationScope, NativeArtifactTarget, NativePortableArchiveTarget, NativeReleaseAuthority,
    NativeReleaseSignatureV1, NativeReleaseVerificationContext, SignatureAlgorithm,
    SignedLaunchGrantV1, SignedNativeReleaseEnvelopeV1, VerifiedLaunchGrant,
    VerifiedNativeReleaseEnvelope, native_artifact_id,
};

pub const NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION: u32 = 1;
pub const MAX_NATIVE_RELEASE_CANDIDATE_BYTES: usize = 512 * 1024;
pub const MAX_NATIVE_RELEASE_NOTES_BYTES: usize = 512 * 1024;

const MAX_KEY_ID_BYTES: usize = 64;
const ED25519_SIGNATURE_BYTES: usize = 64;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const SIGNING_DOMAIN: &[u8] = b"QIONGLI-NATIVE-RELEASE-CANDIDATE-V1\0";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeReleaseCandidateStatus {
    AssembledUnpublished,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeClientPluginGrantV1 {
    pub target: ClientActivationTarget,
    pub signed_launch_grant: SignedLaunchGrantV1,
}

impl Debug for NativeClientPluginGrantV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeClientPluginGrantV1")
            .field("target", &self.target)
            .field("artifact", &self.signed_launch_grant.grant.artifact)
            .field("key_id", &self.signed_launch_grant.signature.key_id)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReleaseNotesV1 {
    pub file_name: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReleaseCandidateV1 {
    pub schema_version: u32,
    pub status: NativeReleaseCandidateStatus,
    pub generation: u64,
    pub source_commit: String,
    pub artifact: ArtifactIdentityV1,
    pub signed_portable_release: SignedNativeReleaseEnvelopeV1,
    pub client_plugins: Vec<NativeClientPluginGrantV1>,
    pub release_notes: NativeReleaseNotesV1,
    pub not_before_unix: u64,
    pub expires_at_unix: u64,
}

impl Debug for NativeReleaseCandidateV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeReleaseCandidateV1")
            .field("schema_version", &self.schema_version)
            .field("status", &self.status)
            .field("generation", &self.generation)
            .field("source_commit", &self.source_commit)
            .field("artifact", &self.artifact)
            .field("client_plugins", &self.client_plugins)
            .field("release_notes", &self.release_notes)
            .field("not_before_unix", &self.not_before_unix)
            .field("expires_at_unix", &self.expires_at_unix)
            .finish_non_exhaustive()
    }
}

impl NativeReleaseCandidateV1 {
    fn validate(&self) -> Result<(), NativeReleaseCandidateError> {
        if self.schema_version != NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION {
            return Err(NativeReleaseCandidateError::UnsupportedSchema);
        }
        self.artifact
            .validate_lite()
            .map_err(|_| NativeReleaseCandidateError::InvalidCandidate)?;
        if self.status != NativeReleaseCandidateStatus::AssembledUnpublished
            || self.artifact.installer_kind != InstallerKind::PortableArchive
            || !valid_generation(self.generation)
            || !valid_source_commit(&self.source_commit)
            || self.not_before_unix > JCS_MAX_SAFE_INTEGER
            || self.expires_at_unix > JCS_MAX_SAFE_INTEGER
            || self.not_before_unix >= self.expires_at_unix
        {
            return Err(NativeReleaseCandidateError::InvalidCandidate);
        }
        native_artifact_id(&self.artifact)
            .map_err(|_| NativeReleaseCandidateError::InvalidCandidate)?;
        validate_notes_descriptor(&self.release_notes, &self.artifact)?;

        self.signed_portable_release
            .to_canonical_json()
            .map_err(|_| NativeReleaseCandidateError::InvalidCandidate)?;
        let portable = &self.signed_portable_release.envelope;
        let portable_grant = &portable.signed_launch_grant.grant;
        if portable.artifact != self.artifact
            || portable.generation != self.generation
            || self.not_before_unix < portable.not_before_unix
            || self.expires_at_unix > portable.expires_at_unix
            || portable_grant.allowed_modes.as_slice() != [GrantMode::LiteMcp]
            || portable_grant.integration_scopes.as_slice()
                != [
                    IntegrationScope::CodexLocal,
                    IntegrationScope::ClaudeCodeLocal,
                ]
        {
            return Err(NativeReleaseCandidateError::InvalidCandidate);
        }

        let expected_targets = [
            ClientActivationTarget::Codex,
            ClientActivationTarget::ClaudeCode,
        ];
        if self.client_plugins.len() != expected_targets.len() {
            return Err(NativeReleaseCandidateError::InvalidCandidate);
        }
        let expected_plugin_artifact = plugin_artifact(&self.artifact);
        for (plugin, expected_target) in self.client_plugins.iter().zip(expected_targets) {
            plugin
                .signed_launch_grant
                .to_canonical_json()
                .map_err(|_| NativeReleaseCandidateError::InvalidCandidate)?;
            let grant = &plugin.signed_launch_grant.grant;
            if plugin.target != expected_target
                || grant.artifact != expected_plugin_artifact
                || grant.generation != portable_grant.generation
                || grant.binary_sha256 != portable.binary_sha256
                || grant.resource_pack_sha256 != portable.resource_pack_sha256
                || grant.allowed_modes.as_slice() != expected_target.allowed_grant_modes()
                || grant.integration_scopes.as_slice() != [expected_target.integration_scope()]
                || self.not_before_unix < grant.not_before_unix
                || self.expires_at_unix > grant.expires_at_unix
            {
                return Err(NativeReleaseCandidateError::InvalidCandidate);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedNativeReleaseCandidateV1 {
    pub candidate: NativeReleaseCandidateV1,
    pub signature: NativeReleaseSignatureV1,
}

impl Debug for SignedNativeReleaseCandidateV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignedNativeReleaseCandidateV1")
            .field("candidate", &self.candidate)
            .field("signature_key_id", &self.signature.key_id)
            .finish_non_exhaustive()
    }
}

impl SignedNativeReleaseCandidateV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, NativeReleaseCandidateError> {
        if input.len() > MAX_NATIVE_RELEASE_CANDIDATE_BYTES {
            return Err(NativeReleaseCandidateError::InputTooLarge);
        }
        let signed = serde_json::from_slice::<Self>(input)
            .map_err(|_| NativeReleaseCandidateError::InvalidJson)?;
        signed.validate_structure()?;
        if signed.to_canonical_json()?.as_slice() != input {
            return Err(NativeReleaseCandidateError::NonCanonicalJson);
        }
        Ok(signed)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeReleaseCandidateError> {
        self.validate_structure()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| NativeReleaseCandidateError::CanonicalSerializationFailed)?;
        if bytes.len() > MAX_NATIVE_RELEASE_CANDIDATE_BYTES {
            return Err(NativeReleaseCandidateError::InputTooLarge);
        }
        Ok(bytes)
    }

    pub fn verify(
        &self,
        authority: &NativeReleaseAuthority,
        context: &NativeReleaseCandidateVerificationContext<'_>,
        pack: &LoadedResourcePack<'_>,
        archive_target: &NativePortableArchiveTarget,
        release_notes: &[u8],
    ) -> Result<VerifiedNativeReleaseCandidate, NativeReleaseCandidateError> {
        let signing_bytes = self.verify_authority(authority, context)?;
        if archive_target.artifact() != context.expected_artifact {
            return Err(NativeReleaseCandidateError::CandidateArtifactMismatch);
        }
        verify_release_notes(
            &self.candidate.release_notes,
            &self.candidate.artifact,
            release_notes,
        )?;

        let release_context = NativeReleaseVerificationContext {
            now_unix: context.now_unix,
            minimum_release_generation: authority.minimum_release_generation(),
            minimum_launch_grant_generation: authority.minimum_launch_grant_generation(),
            expected_artifact: context.expected_artifact,
            expected_channel: authority.channel(),
            requested_mode: GrantMode::LiteMcp,
            requested_scope: context.requested_target.integration_scope(),
        };
        let portable_release = self
            .candidate
            .signed_portable_release
            .verify(
                authority.release_keys(),
                authority.launch_grant_keys(),
                &release_context,
                pack,
                archive_target,
            )
            .map_err(|_| NativeReleaseCandidateError::PortableReleaseInvalid)?;

        let plugin_grant = self.verify_plugin_grant(authority, context)?;

        Ok(VerifiedNativeReleaseCandidate {
            signed: self.clone(),
            signed_payload_sha256: sha256_hex(&signing_bytes),
            release_key_id: self.signature.key_id.clone(),
            verified_at_unix: context.now_unix,
            target: context.requested_target,
            portable_release,
            plugin_grant,
        })
    }

    /// Verifies installed bytes and binds them to the caller-supplied process path.
    /// The trusted app boundary must supply current_exe(), embedded source/version,
    /// authority and time. This is not installation approval or release qualification;
    /// release-note and archive bytes are checked by `verify` before installation.
    pub fn verify_installed(
        &self,
        authority: &NativeReleaseAuthority,
        context: &NativeReleaseCandidateVerificationContext<'_>,
        pack: &LoadedResourcePack<'_>,
        target: &NativeArtifactTarget,
        current_executable: &Path,
    ) -> Result<VerifiedInstalledNativeCandidate, NativeReleaseCandidateError> {
        self.verify_authority(authority, context)?;
        let release_context = NativeReleaseVerificationContext {
            now_unix: context.now_unix,
            minimum_release_generation: authority.minimum_release_generation(),
            minimum_launch_grant_generation: authority.minimum_launch_grant_generation(),
            expected_artifact: context.expected_artifact,
            expected_channel: authority.channel(),
            requested_mode: GrantMode::LiteMcp,
            requested_scope: context.requested_target.integration_scope(),
        };
        self.candidate
            .signed_portable_release
            .verify_extracted_artifact(
                authority.release_keys(),
                authority.launch_grant_keys(),
                &release_context,
                pack,
                target,
            )
            .map_err(|_| NativeReleaseCandidateError::PortableReleaseInvalid)?;
        let binary_path = crate::native_artifact_binary_path(context.expected_artifact)
            .map_err(|_| NativeReleaseCandidateError::ExecutableInvalid)?;
        let expected = fs::canonicalize(target.path().join(binary_path))
            .map_err(|_| NativeReleaseCandidateError::ExecutableInvalid)?;
        let current = fs::canonicalize(current_executable)
            .map_err(|_| NativeReleaseCandidateError::ExecutableInvalid)?;
        if current != expected {
            return Err(NativeReleaseCandidateError::ExecutableInvalid);
        }
        let plugin_grant = self.verify_plugin_grant(authority, context)?;
        Ok(VerifiedInstalledNativeCandidate {
            candidate: self.candidate.clone(),
            current_executable: current,
            plugin_grant,
        })
    }

    fn verify_authority(
        &self,
        authority: &NativeReleaseAuthority,
        context: &NativeReleaseCandidateVerificationContext<'_>,
    ) -> Result<Vec<u8>, NativeReleaseCandidateError> {
        self.validate_structure()?;
        validate_release_keys(authority.release_keys())
            .map_err(|_| NativeReleaseCandidateError::InvalidCandidate)?;
        let key = authority
            .release_keys()
            .iter()
            .find(|key| key.key_id() == self.signature.key_id)
            .ok_or(NativeReleaseCandidateError::ReleaseKeyUntrusted)?;
        if !key.authorizes_generation(self.candidate.generation) {
            return Err(NativeReleaseCandidateError::ReleaseKeyGenerationUnavailable);
        }
        let signature_bytes =
            decode_fixed_hex::<ED25519_SIGNATURE_BYTES>(&self.signature.value_hex)
                .ok_or(NativeReleaseCandidateError::InvalidCandidate)?;
        let signing_bytes = native_release_candidate_signing_bytes(&self.candidate)?;
        if !key.verifies_signature(&signing_bytes, &signature_bytes) {
            return Err(NativeReleaseCandidateError::SignatureInvalid);
        }

        if context.now_unix < self.candidate.not_before_unix {
            return Err(NativeReleaseCandidateError::CandidateNotYetValid);
        }
        if context.now_unix >= self.candidate.expires_at_unix {
            return Err(NativeReleaseCandidateError::CandidateExpired);
        }
        if self.candidate.generation < authority.minimum_release_generation() {
            return Err(NativeReleaseCandidateError::CandidateReplayed);
        }
        if self.candidate.artifact.channel != authority.channel() {
            return Err(NativeReleaseCandidateError::CandidateChannelMismatch);
        }
        if &self.candidate.artifact != context.expected_artifact {
            return Err(NativeReleaseCandidateError::CandidateArtifactMismatch);
        }
        if self.candidate.source_commit != context.expected_source_commit {
            return Err(NativeReleaseCandidateError::CandidateSourceMismatch);
        }
        Ok(signing_bytes)
    }

    fn verify_plugin_grant(
        &self,
        authority: &NativeReleaseAuthority,
        context: &NativeReleaseCandidateVerificationContext<'_>,
    ) -> Result<VerifiedLaunchGrant, NativeReleaseCandidateError> {
        let plugin = self
            .candidate
            .client_plugins
            .iter()
            .find(|plugin| plugin.target == context.requested_target)
            .ok_or(NativeReleaseCandidateError::PluginGrantInvalid)?;
        let plugin_artifact = plugin_artifact(context.expected_artifact);
        let grant_context = GrantVerificationContext {
            now_unix: context.now_unix,
            minimum_generation: authority.minimum_launch_grant_generation(),
            expected_artifact: &plugin_artifact,
            binary_sha256: &self
                .candidate
                .signed_portable_release
                .envelope
                .binary_sha256,
            resource_pack_sha256: &self
                .candidate
                .signed_portable_release
                .envelope
                .resource_pack_sha256,
            requested_mode: context.requested_target.required_grant_mode(),
            requested_scope: context.requested_target.integration_scope(),
        };
        plugin
            .signed_launch_grant
            .verify(authority.launch_grant_keys(), &grant_context)
            .map_err(|_| NativeReleaseCandidateError::PluginGrantInvalid)
    }

    fn validate_structure(&self) -> Result<(), NativeReleaseCandidateError> {
        self.candidate.validate()?;
        if self.signature.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.signature.key_id, MAX_KEY_ID_BYTES)
            || !is_lower_hex(
                &self.signature.value_hex,
                ED25519_SIGNATURE_BYTES.saturating_mul(2),
            )
        {
            return Err(NativeReleaseCandidateError::InvalidCandidate);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NativeReleaseCandidateVerificationContext<'a> {
    pub now_unix: u64,
    pub expected_source_commit: &'a str,
    pub expected_artifact: &'a ArtifactIdentityV1,
    pub requested_target: ClientActivationTarget,
}

#[derive(Clone)]
pub struct VerifiedInstalledNativeCandidate {
    candidate: NativeReleaseCandidateV1,
    current_executable: PathBuf,
    plugin_grant: VerifiedLaunchGrant,
}

impl VerifiedInstalledNativeCandidate {
    #[must_use]
    pub const fn candidate(&self) -> &NativeReleaseCandidateV1 {
        &self.candidate
    }

    #[must_use]
    pub fn current_executable(&self) -> &Path {
        &self.current_executable
    }

    #[must_use]
    pub const fn plugin_grant(&self) -> &VerifiedLaunchGrant {
        &self.plugin_grant
    }
}

impl Debug for VerifiedInstalledNativeCandidate {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedInstalledNativeCandidate")
            .field("artifact", &self.candidate.artifact)
            .field("source_commit", &self.candidate.source_commit)
            .field("current_executable", &"<verified-installed-executable>")
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct VerifiedNativeReleaseCandidate {
    signed: SignedNativeReleaseCandidateV1,
    signed_payload_sha256: String,
    release_key_id: String,
    verified_at_unix: u64,
    target: ClientActivationTarget,
    portable_release: VerifiedNativeReleaseEnvelope,
    plugin_grant: VerifiedLaunchGrant,
}

impl VerifiedNativeReleaseCandidate {
    #[must_use]
    pub const fn candidate(&self) -> &NativeReleaseCandidateV1 {
        &self.signed.candidate
    }

    #[must_use]
    pub const fn signed_candidate(&self) -> &SignedNativeReleaseCandidateV1 {
        &self.signed
    }

    #[must_use]
    pub fn signed_payload_sha256(&self) -> &str {
        &self.signed_payload_sha256
    }

    #[must_use]
    pub fn release_key_id(&self) -> &str {
        &self.release_key_id
    }

    #[must_use]
    pub const fn verified_at_unix(&self) -> u64 {
        self.verified_at_unix
    }

    #[must_use]
    pub const fn target(&self) -> ClientActivationTarget {
        self.target
    }

    #[must_use]
    pub const fn portable_release(&self) -> &VerifiedNativeReleaseEnvelope {
        &self.portable_release
    }

    #[must_use]
    pub const fn plugin_grant(&self) -> &VerifiedLaunchGrant {
        &self.plugin_grant
    }
}

impl Debug for VerifiedNativeReleaseCandidate {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedNativeReleaseCandidate")
            .field("artifact", &self.signed.candidate.artifact)
            .field("source_commit", &self.signed.candidate.source_commit)
            .field("signed_payload_sha256", &self.signed_payload_sha256)
            .field("release_key_id", &self.release_key_id)
            .field("verified_at_unix", &self.verified_at_unix)
            .field("target", &self.target)
            .finish()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_native_release_candidate(
    generation: u64,
    source_commit: impl Into<String>,
    signed_portable_release: &SignedNativeReleaseEnvelopeV1,
    client_plugins: [NativeClientPluginGrantV1; 2],
    release_notes: &[u8],
    not_before_unix: u64,
    expires_at_unix: u64,
) -> Result<NativeReleaseCandidateV1, NativeReleaseCandidateError> {
    validate_release_notes_bytes(release_notes)?;
    let artifact = signed_portable_release.envelope.artifact.clone();
    let size_bytes = u64::try_from(release_notes.len())
        .map_err(|_| NativeReleaseCandidateError::ReleaseNotesInvalid)?;
    let candidate = NativeReleaseCandidateV1 {
        schema_version: NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION,
        status: NativeReleaseCandidateStatus::AssembledUnpublished,
        generation,
        source_commit: source_commit.into(),
        artifact: artifact.clone(),
        signed_portable_release: signed_portable_release.clone(),
        client_plugins: Vec::from(client_plugins),
        release_notes: NativeReleaseNotesV1 {
            file_name: native_release_notes_file_name(&artifact)?,
            size_bytes,
            sha256: sha256_hex(release_notes),
        },
        not_before_unix,
        expires_at_unix,
    };
    candidate.validate()?;
    Ok(candidate)
}

pub fn native_release_candidate_signing_bytes(
    candidate: &NativeReleaseCandidateV1,
) -> Result<Vec<u8>, NativeReleaseCandidateError> {
    candidate.validate()?;
    let canonical = serde_json_canonicalizer::to_vec(candidate)
        .map_err(|_| NativeReleaseCandidateError::CanonicalSerializationFailed)?;
    let mut output = Vec::with_capacity(SIGNING_DOMAIN.len().saturating_add(canonical.len()));
    output.extend_from_slice(SIGNING_DOMAIN);
    output.extend_from_slice(&canonical);
    Ok(output)
}

pub fn native_release_candidate_file_name(
    artifact: &ArtifactIdentityV1,
) -> Result<String, NativeReleaseCandidateError> {
    if artifact.installer_kind != InstallerKind::PortableArchive {
        return Err(NativeReleaseCandidateError::InvalidCandidate);
    }
    let artifact_id =
        native_artifact_id(artifact).map_err(|_| NativeReleaseCandidateError::InvalidCandidate)?;
    Ok(format!("{artifact_id}.candidate.json"))
}

pub fn native_release_notes_file_name(
    artifact: &ArtifactIdentityV1,
) -> Result<String, NativeReleaseCandidateError> {
    if artifact.installer_kind != InstallerKind::PortableArchive {
        return Err(NativeReleaseCandidateError::InvalidCandidate);
    }
    let artifact_id =
        native_artifact_id(artifact).map_err(|_| NativeReleaseCandidateError::InvalidCandidate)?;
    Ok(format!("{artifact_id}.release-notes.md"))
}

fn validate_notes_descriptor(
    notes: &NativeReleaseNotesV1,
    artifact: &ArtifactIdentityV1,
) -> Result<(), NativeReleaseCandidateError> {
    if notes.file_name != native_release_notes_file_name(artifact)?
        || notes.size_bytes == 0
        || notes.size_bytes > MAX_NATIVE_RELEASE_NOTES_BYTES as u64
        || !is_lower_hex(&notes.sha256, 64)
    {
        return Err(NativeReleaseCandidateError::ReleaseNotesInvalid);
    }
    Ok(())
}

fn verify_release_notes(
    descriptor: &NativeReleaseNotesV1,
    artifact: &ArtifactIdentityV1,
    bytes: &[u8],
) -> Result<(), NativeReleaseCandidateError> {
    validate_notes_descriptor(descriptor, artifact)?;
    validate_release_notes_bytes(bytes)?;
    let size_bytes =
        u64::try_from(bytes.len()).map_err(|_| NativeReleaseCandidateError::ReleaseNotesInvalid)?;
    if descriptor.size_bytes != size_bytes || descriptor.sha256 != sha256_hex(bytes) {
        return Err(NativeReleaseCandidateError::ReleaseNotesInvalid);
    }
    Ok(())
}

fn validate_release_notes_bytes(bytes: &[u8]) -> Result<(), NativeReleaseCandidateError> {
    if bytes.is_empty()
        || bytes.len() > MAX_NATIVE_RELEASE_NOTES_BYTES
        || bytes.contains(&0)
        || std::str::from_utf8(bytes).is_err()
    {
        return Err(NativeReleaseCandidateError::ReleaseNotesInvalid);
    }
    Ok(())
}

fn plugin_artifact(portable: &ArtifactIdentityV1) -> ArtifactIdentityV1 {
    let mut artifact = portable.clone();
    artifact.installer_kind = InstallerKind::PluginBundle;
    artifact
}

fn valid_generation(value: u64) -> bool {
    value > 0 && value <= JCS_MAX_SAFE_INTEGER
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value, value.len())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeReleaseCandidateError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    UnsupportedSchema,
    InvalidCandidate,
    ReleaseKeyUntrusted,
    ReleaseKeyGenerationUnavailable,
    SignatureInvalid,
    CandidateNotYetValid,
    CandidateExpired,
    CandidateReplayed,
    CandidateChannelMismatch,
    CandidateArtifactMismatch,
    CandidateSourceMismatch,
    ExecutableInvalid,
    ReleaseNotesInvalid,
    PortableReleaseInvalid,
    PluginGrantInvalid,
    CanonicalSerializationFailed,
}

impl NativeReleaseCandidateError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "native-release-candidate-too-large",
            Self::InvalidJson => "native-release-candidate-json-invalid",
            Self::NonCanonicalJson => "native-release-candidate-json-noncanonical",
            Self::UnsupportedSchema => "native-release-candidate-schema-unsupported",
            Self::InvalidCandidate => "native-release-candidate-invalid",
            Self::ReleaseKeyUntrusted => "native-release-candidate-key-untrusted",
            Self::ReleaseKeyGenerationUnavailable => {
                "native-release-candidate-key-generation-unavailable"
            }
            Self::SignatureInvalid => "native-release-candidate-signature-invalid",
            Self::CandidateNotYetValid => "native-release-candidate-not-yet-valid",
            Self::CandidateExpired => "native-release-candidate-expired",
            Self::CandidateReplayed => "native-release-candidate-generation-stale",
            Self::CandidateChannelMismatch => "native-release-candidate-channel-mismatch",
            Self::CandidateArtifactMismatch => "native-release-candidate-artifact-mismatch",
            Self::CandidateSourceMismatch => "native-release-candidate-source-mismatch",
            Self::ExecutableInvalid => "native-release-candidate-executable-invalid",
            Self::ReleaseNotesInvalid => "native-release-candidate-notes-invalid",
            Self::PortableReleaseInvalid => "native-release-candidate-portable-invalid",
            Self::PluginGrantInvalid => "native-release-candidate-plugin-grant-invalid",
            Self::CanonicalSerializationFailed => {
                "native-release-candidate-canonicalization-failed"
            }
        }
    }
}

impl Display for NativeReleaseCandidateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeReleaseCandidateError {}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::Value;

    use super::*;
    use crate::{
        GrantSignatureV1, LaunchGrantV1, NativeReleaseEnvelopeV1, ProductId, ReleaseChannel,
        current_target_native_artifact_identity, launch_grant_signing_bytes,
        native_portable_archive_file_name, native_release_envelope_signing_bytes,
    };

    const NOW: u64 = 1_750_000_000;
    const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const NOTES: &[u8] = b"# Qiongli 2.0.0-alpha.1\n\nLite local preview.\n";

    fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(N * 2);
        for byte in bytes {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    fn sign_grant(grant: LaunchGrantV1, key: &SigningKey) -> SignedLaunchGrantV1 {
        let signature = key.sign(&launch_grant_signing_bytes(&grant).unwrap());
        SignedLaunchGrantV1 {
            grant,
            signature: GrantSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "candidate-launch-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        }
    }

    fn grant(
        artifact: ArtifactIdentityV1,
        scopes: Vec<IntegrationScope>,
        key: &SigningKey,
    ) -> SignedLaunchGrantV1 {
        let allowed_modes = match scopes.as_slice() {
            [IntegrationScope::CodexLocal] => {
                ClientActivationTarget::Codex.allowed_grant_modes().to_vec()
            }
            [IntegrationScope::ClaudeCodeLocal] => ClientActivationTarget::ClaudeCode
                .allowed_grant_modes()
                .to_vec(),
            _ => vec![GrantMode::LiteMcp],
        };
        sign_grant(
            LaunchGrantV1 {
                schema_version: 1,
                generation: 13,
                artifact,
                binary_sha256: "1".repeat(64),
                resource_pack_sha256: "2".repeat(64),
                allowed_modes,
                integration_scopes: scopes,
                not_before_unix: NOW - 60,
                expires_at_unix: NOW + 3_600,
            },
            key,
        )
    }

    fn unsigned_candidate() -> NativeReleaseCandidateV1 {
        let portable =
            current_target_native_artifact_identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .unwrap();
        assert_eq!(portable.product, ProductId::Qiongli);
        let mut plugin = portable.clone();
        plugin.installer_kind = InstallerKind::PluginBundle;
        let launch_key = SigningKey::from_bytes(&[41_u8; 32]);
        let portable_grant = grant(
            portable.clone(),
            vec![
                IntegrationScope::CodexLocal,
                IntegrationScope::ClaudeCodeLocal,
            ],
            &launch_key,
        );
        let envelope = NativeReleaseEnvelopeV1 {
            schema_version: 1,
            generation: 17,
            artifact: portable.clone(),
            archive_file_name: native_portable_archive_file_name(&portable).unwrap(),
            archive_size_bytes: 1_024,
            archive_sha256: "3".repeat(64),
            artifact_manifest_sha256: "4".repeat(64),
            resource_pack_sha256: "2".repeat(64),
            artifact_content_root_sha256: "5".repeat(64),
            binary_sha256: "1".repeat(64),
            signed_launch_grant: portable_grant,
            not_before_unix: NOW - 30,
            expires_at_unix: NOW + 1_800,
        };
        let release_key = SigningKey::from_bytes(&[42_u8; 32]);
        let release_signature =
            release_key.sign(&native_release_envelope_signing_bytes(&envelope).unwrap());
        let signed_release = SignedNativeReleaseEnvelopeV1 {
            envelope,
            signature: NativeReleaseSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "candidate-release-key".to_string(),
                value_hex: encode_hex(&release_signature.to_bytes()),
            },
        };
        build_native_release_candidate(
            17,
            SOURCE_COMMIT,
            &signed_release,
            [
                NativeClientPluginGrantV1 {
                    target: ClientActivationTarget::Codex,
                    signed_launch_grant: grant(
                        plugin.clone(),
                        vec![IntegrationScope::CodexLocal],
                        &launch_key,
                    ),
                },
                NativeClientPluginGrantV1 {
                    target: ClientActivationTarget::ClaudeCode,
                    signed_launch_grant: grant(
                        plugin,
                        vec![IntegrationScope::ClaudeCodeLocal],
                        &launch_key,
                    ),
                },
            ],
            NOTES,
            NOW,
            NOW + 1_200,
        )
        .unwrap()
    }

    fn signed_candidate() -> SignedNativeReleaseCandidateV1 {
        let candidate = unsigned_candidate();
        let release_key = SigningKey::from_bytes(&[42_u8; 32]);
        let signature =
            release_key.sign(&native_release_candidate_signing_bytes(&candidate).unwrap());
        SignedNativeReleaseCandidateV1 {
            candidate,
            signature: NativeReleaseSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "candidate-release-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        }
    }

    #[test]
    fn candidate_json_is_strict_bounded_canonical_and_redacted() {
        let signed = signed_candidate();
        let canonical = signed.to_canonical_json().unwrap();
        assert!(SignedNativeReleaseCandidateV1::from_json(&canonical).unwrap() == signed);
        assert!(
            native_release_candidate_signing_bytes(&signed.candidate)
                .unwrap()
                .starts_with(SIGNING_DOMAIN)
        );
        assert_eq!(
            native_release_candidate_file_name(&signed.candidate.artifact).unwrap(),
            format!(
                "{}.candidate.json",
                native_artifact_id(&signed.candidate.artifact).unwrap()
            )
        );
        assert_eq!(
            native_release_notes_file_name(&signed.candidate.artifact).unwrap(),
            signed.candidate.release_notes.file_name
        );

        let debug = format!("{signed:?}");
        assert!(debug.contains("candidate-release-key"));
        assert!(!debug.contains(&signed.signature.value_hex));
        assert!(
            !debug.contains(
                &signed.candidate.client_plugins[0]
                    .signed_launch_grant
                    .signature
                    .value_hex
            )
        );

        let mut noncanonical = canonical.clone();
        noncanonical.push(b'\n');
        assert_eq!(
            SignedNativeReleaseCandidateV1::from_json(&noncanonical),
            Err(NativeReleaseCandidateError::NonCanonicalJson)
        );
        let mut unknown: Value = serde_json::from_slice(&canonical).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_string(), Value::Bool(true));
        assert_eq!(
            SignedNativeReleaseCandidateV1::from_json(&serde_json::to_vec(&unknown).unwrap()),
            Err(NativeReleaseCandidateError::InvalidJson)
        );
        assert_eq!(
            SignedNativeReleaseCandidateV1::from_json(&vec![b' '; 513 * 1024]),
            Err(NativeReleaseCandidateError::InputTooLarge)
        );
    }

    #[test]
    fn candidate_requires_exact_target_grants_and_identity_closure() {
        let baseline = unsigned_candidate();

        let mut swapped = baseline.clone();
        swapped.client_plugins.swap(0, 1);
        assert_eq!(
            native_release_candidate_signing_bytes(&swapped),
            Err(NativeReleaseCandidateError::InvalidCandidate)
        );

        let mut wrong_scope = baseline.clone();
        wrong_scope.client_plugins[0]
            .signed_launch_grant
            .grant
            .integration_scopes = vec![IntegrationScope::ClaudeCodeLocal];
        assert_eq!(
            native_release_candidate_signing_bytes(&wrong_scope),
            Err(NativeReleaseCandidateError::InvalidCandidate)
        );

        let mut wrong_binary = baseline.clone();
        wrong_binary.client_plugins[1]
            .signed_launch_grant
            .grant
            .binary_sha256 = "9".repeat(64);
        assert_eq!(
            native_release_candidate_signing_bytes(&wrong_binary),
            Err(NativeReleaseCandidateError::InvalidCandidate)
        );

        let mut wrong_kind = baseline.clone();
        wrong_kind.client_plugins[0]
            .signed_launch_grant
            .grant
            .artifact
            .installer_kind = InstallerKind::PortableArchive;
        assert_eq!(
            native_release_candidate_signing_bytes(&wrong_kind),
            Err(NativeReleaseCandidateError::InvalidCandidate)
        );

        let mut wrong_notes = baseline;
        wrong_notes.release_notes.file_name = "release.md".to_string();
        assert_eq!(
            native_release_candidate_signing_bytes(&wrong_notes),
            Err(NativeReleaseCandidateError::ReleaseNotesInvalid)
        );
    }

    #[test]
    fn candidate_errors_expose_only_fixed_reason_codes() {
        for error in [
            NativeReleaseCandidateError::SignatureInvalid,
            NativeReleaseCandidateError::CandidateSourceMismatch,
            NativeReleaseCandidateError::PortableReleaseInvalid,
            NativeReleaseCandidateError::PluginGrantInvalid,
        ] {
            assert_eq!(error.to_string(), error.reason_code());
            assert!(!error.to_string().contains('/'));
            assert!(!error.to_string().contains('\\'));
        }
    }
}
