use std::fmt::{self, Debug, Display, Formatter};

use ed25519_dalek::{Signature, VerifyingKey};
use qiongli_content::LoadedResourcePack;
use serde::{Deserialize, Serialize};

use crate::grant::{decode_fixed_hex, is_lower_hex, sha256_hex, valid_identifier};
use crate::{
    ArtifactIdentityV1, GrantMode, GrantVerificationContext, IntegrationScope,
    NativeArtifactTarget, NativePortableArchiveTarget, ReleaseChannel, SignatureAlgorithm,
    SignedLaunchGrantV1, TrustedPublicKey, VerifiedLaunchGrant, VerifiedNativePortableArchive,
    native_portable_archive_file_name, verify_native_artifact, verify_native_portable_archive,
};

pub const NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION: u32 = 1;

pub const MAX_NATIVE_RELEASE_ENVELOPE_BYTES: usize = 256 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 129 * 1024 * 1024;
const MAX_KEY_ID_BYTES: usize = 64;
const MAX_TRUSTED_RELEASE_KEYS: usize = 16;
const ED25519_PUBLIC_KEY_BYTES: usize = 32;
const ED25519_SIGNATURE_BYTES: usize = 64;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const SIGNING_DOMAIN: &[u8] = b"QIONGLI-NATIVE-RELEASE-ENVELOPE-V1\0";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReleaseEnvelopeV1 {
    pub schema_version: u32,
    pub generation: u64,
    pub artifact: ArtifactIdentityV1,
    pub archive_file_name: String,
    pub archive_size_bytes: u64,
    pub archive_sha256: String,
    pub artifact_manifest_sha256: String,
    pub resource_pack_sha256: String,
    pub artifact_content_root_sha256: String,
    pub binary_sha256: String,
    pub signed_launch_grant: SignedLaunchGrantV1,
    pub not_before_unix: u64,
    pub expires_at_unix: u64,
}

impl NativeReleaseEnvelopeV1 {
    fn validate(&self) -> Result<(), NativeReleaseError> {
        if self.schema_version != NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION {
            return Err(NativeReleaseError::UnsupportedSchema);
        }
        self.artifact
            .validate_lite()
            .map_err(|_| NativeReleaseError::InvalidEnvelope)?;
        self.signed_launch_grant
            .validate_structure()
            .map_err(|_| NativeReleaseError::InvalidEnvelope)?;
        let grant = &self.signed_launch_grant.grant;
        let expected_file_name = native_portable_archive_file_name(&self.artifact)
            .map_err(|_| NativeReleaseError::InvalidEnvelope)?;
        if self.generation == 0
            || self.generation > JCS_MAX_SAFE_INTEGER
            || self.archive_file_name != expected_file_name
            || self.archive_size_bytes == 0
            || self.archive_size_bytes > MAX_ARCHIVE_BYTES
            || !is_lower_hex(&self.archive_sha256, 64)
            || !is_lower_hex(&self.artifact_manifest_sha256, 64)
            || !is_lower_hex(&self.resource_pack_sha256, 64)
            || !is_lower_hex(&self.artifact_content_root_sha256, 64)
            || !is_lower_hex(&self.binary_sha256, 64)
            || self.not_before_unix > JCS_MAX_SAFE_INTEGER
            || self.expires_at_unix > JCS_MAX_SAFE_INTEGER
            || self.not_before_unix >= self.expires_at_unix
            || self.not_before_unix < grant.not_before_unix
            || self.expires_at_unix > grant.expires_at_unix
            || self.artifact != grant.artifact
            || self.binary_sha256 != grant.binary_sha256
            || self.resource_pack_sha256 != grant.resource_pack_sha256
        {
            return Err(NativeReleaseError::InvalidEnvelope);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReleaseSignatureV1 {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub value_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedNativeReleaseEnvelopeV1 {
    pub envelope: NativeReleaseEnvelopeV1,
    pub signature: NativeReleaseSignatureV1,
}

impl SignedNativeReleaseEnvelopeV1 {
    pub fn from_json(input: &[u8]) -> Result<Self, NativeReleaseError> {
        if input.len() > MAX_NATIVE_RELEASE_ENVELOPE_BYTES {
            return Err(NativeReleaseError::InputTooLarge);
        }
        let signed =
            serde_json::from_slice::<Self>(input).map_err(|_| NativeReleaseError::InvalidJson)?;
        signed.validate_structure()?;
        if signed.to_canonical_json()? != input {
            return Err(NativeReleaseError::NonCanonicalJson);
        }
        Ok(signed)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, NativeReleaseError> {
        self.validate_structure()?;
        let bytes = serde_json_canonicalizer::to_vec(self)
            .map_err(|_| NativeReleaseError::CanonicalSerializationFailed)?;
        if bytes.len() > MAX_NATIVE_RELEASE_ENVELOPE_BYTES {
            return Err(NativeReleaseError::InputTooLarge);
        }
        Ok(bytes)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn verify(
        &self,
        release_keys: &[TrustedReleasePublicKey],
        launch_grant_keys: &[TrustedPublicKey],
        context: &NativeReleaseVerificationContext<'_>,
        pack: &LoadedResourcePack<'_>,
        archive_target: &NativePortableArchiveTarget,
    ) -> Result<VerifiedNativeReleaseEnvelope, NativeReleaseError> {
        let signing_bytes = self.verify_release_authority(release_keys, context)?;
        if archive_target.artifact() != context.expected_artifact {
            return Err(NativeReleaseError::ReleaseArtifactMismatch);
        }

        let archive = verify_native_portable_archive(pack, archive_target)
            .map_err(|_| NativeReleaseError::ArchiveInvalid)?;
        let manifest = archive.payload().manifest();
        if archive.artifact() != &self.envelope.artifact
            || archive.file_name() != self.envelope.archive_file_name
            || archive.size_bytes() != self.envelope.archive_size_bytes
            || archive.archive_sha256() != self.envelope.archive_sha256
            || archive.manifest_sha256() != self.envelope.artifact_manifest_sha256
            || manifest.content.pack_sha256 != self.envelope.resource_pack_sha256
            || manifest.artifact_content_root_sha256 != self.envelope.artifact_content_root_sha256
            || manifest.binary_sha256 != self.envelope.binary_sha256
            || pack.pack_sha256() != self.envelope.resource_pack_sha256
        {
            return Err(NativeReleaseError::ReleasePayloadMismatch);
        }

        let launch_grant = self.verify_launch_grant(launch_grant_keys, context)?;

        Ok(VerifiedNativeReleaseEnvelope {
            signed: self.clone(),
            signed_payload_sha256: sha256_hex(&signing_bytes),
            release_key_id: self.signature.key_id.clone(),
            verified_at_unix: context.now_unix,
            archive_target: archive_target.clone(),
            archive,
            launch_grant,
        })
    }

    /// Revalidates an extracted payload against its signed release without retaining
    /// the archive. This returns only the scoped launch grant: it does not bind the
    /// calling process, establish candidate source provenance, or approve writes.
    pub fn verify_extracted_artifact(
        &self,
        release_keys: &[TrustedReleasePublicKey],
        launch_grant_keys: &[TrustedPublicKey],
        context: &NativeReleaseVerificationContext<'_>,
        pack: &LoadedResourcePack<'_>,
        target: &NativeArtifactTarget,
    ) -> Result<VerifiedLaunchGrant, NativeReleaseError> {
        self.verify_release_authority(release_keys, context)?;
        if target.artifact() != context.expected_artifact {
            return Err(NativeReleaseError::ReleaseArtifactMismatch);
        }
        let payload = verify_native_artifact(pack, target)
            .map_err(|_| NativeReleaseError::ReleasePayloadMismatch)?;
        let manifest = payload.manifest();
        if manifest.artifact != self.envelope.artifact
            || payload.manifest_sha256() != self.envelope.artifact_manifest_sha256
            || manifest.content.pack_sha256 != self.envelope.resource_pack_sha256
            || manifest.artifact_content_root_sha256 != self.envelope.artifact_content_root_sha256
            || manifest.binary_sha256 != self.envelope.binary_sha256
            || pack.pack_sha256() != self.envelope.resource_pack_sha256
        {
            return Err(NativeReleaseError::ReleasePayloadMismatch);
        }
        self.verify_launch_grant(launch_grant_keys, context)
    }

    fn verify_release_authority(
        &self,
        release_keys: &[TrustedReleasePublicKey],
        context: &NativeReleaseVerificationContext<'_>,
    ) -> Result<Vec<u8>, NativeReleaseError> {
        self.validate_structure()?;
        validate_release_keys(release_keys)?;
        let key = release_keys
            .iter()
            .find(|key| key.key_id == self.signature.key_id)
            .ok_or(NativeReleaseError::ReleaseKeyUntrusted)?;
        if !key.authorizes_generation(self.envelope.generation) {
            return Err(NativeReleaseError::ReleaseKeyGenerationUnavailable);
        }
        let signature_bytes =
            decode_fixed_hex::<ED25519_SIGNATURE_BYTES>(&self.signature.value_hex)
                .ok_or(NativeReleaseError::InvalidEnvelope)?;
        let signing_bytes = native_release_envelope_signing_bytes(&self.envelope)?;
        let verifying_key = VerifyingKey::from_bytes(&key.public_key)
            .map_err(|_| NativeReleaseError::ReleaseSignatureInvalid)?;
        verifying_key
            .verify_strict(&signing_bytes, &Signature::from_bytes(&signature_bytes))
            .map_err(|_| NativeReleaseError::ReleaseSignatureInvalid)?;

        if context.now_unix < self.envelope.not_before_unix {
            return Err(NativeReleaseError::ReleaseNotYetValid);
        }
        if context.now_unix >= self.envelope.expires_at_unix {
            return Err(NativeReleaseError::ReleaseExpired);
        }
        if self.envelope.generation < context.minimum_release_generation {
            return Err(NativeReleaseError::ReleaseReplayed);
        }
        if self.envelope.artifact.channel != context.expected_channel {
            return Err(NativeReleaseError::ReleaseChannelMismatch);
        }
        if &self.envelope.artifact != context.expected_artifact {
            return Err(NativeReleaseError::ReleaseArtifactMismatch);
        }

        Ok(signing_bytes)
    }

    fn verify_launch_grant(
        &self,
        launch_grant_keys: &[TrustedPublicKey],
        context: &NativeReleaseVerificationContext<'_>,
    ) -> Result<VerifiedLaunchGrant, NativeReleaseError> {
        let launch_context = GrantVerificationContext {
            now_unix: context.now_unix,
            minimum_generation: context.minimum_launch_grant_generation,
            expected_artifact: &self.envelope.artifact,
            binary_sha256: &self.envelope.binary_sha256,
            resource_pack_sha256: &self.envelope.resource_pack_sha256,
            requested_mode: context.requested_mode,
            requested_scope: context.requested_scope,
        };
        self.envelope
            .signed_launch_grant
            .verify(launch_grant_keys, &launch_context)
            .map_err(|_| NativeReleaseError::LaunchGrantInvalid)
    }

    fn validate_structure(&self) -> Result<(), NativeReleaseError> {
        self.envelope.validate()?;
        if self.signature.algorithm != SignatureAlgorithm::Ed25519
            || !valid_identifier(&self.signature.key_id, MAX_KEY_ID_BYTES)
            || !is_lower_hex(
                &self.signature.value_hex,
                ED25519_SIGNATURE_BYTES.saturating_mul(2),
            )
        {
            return Err(NativeReleaseError::InvalidEnvelope);
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TrustedReleasePublicKey {
    key_id: String,
    public_key: [u8; ED25519_PUBLIC_KEY_BYTES],
    minimum_generation: u64,
    maximum_generation_exclusive: Option<u64>,
}

impl TrustedReleasePublicKey {
    pub fn new(
        key_id: impl Into<String>,
        public_key: [u8; ED25519_PUBLIC_KEY_BYTES],
        minimum_generation: u64,
        maximum_generation_exclusive: Option<u64>,
    ) -> Result<Self, NativeReleaseError> {
        let key_id = key_id.into();
        if !valid_identifier(&key_id, MAX_KEY_ID_BYTES)
            || minimum_generation == 0
            || minimum_generation > JCS_MAX_SAFE_INTEGER
            || maximum_generation_exclusive.is_some_and(|maximum| {
                maximum <= minimum_generation || maximum > JCS_MAX_SAFE_INTEGER
            })
            || VerifyingKey::from_bytes(&public_key).is_err()
        {
            return Err(NativeReleaseError::InvalidTrustedReleaseKey);
        }
        Ok(Self {
            key_id,
            public_key,
            minimum_generation,
            maximum_generation_exclusive,
        })
    }

    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    #[must_use]
    pub const fn minimum_generation(&self) -> u64 {
        self.minimum_generation
    }

    #[must_use]
    pub const fn maximum_generation_exclusive(&self) -> Option<u64> {
        self.maximum_generation_exclusive
    }

    pub(crate) fn authorizes_generation(&self, generation: u64) -> bool {
        generation >= self.minimum_generation
            && self
                .maximum_generation_exclusive
                .is_none_or(|maximum| generation < maximum)
    }

    pub(crate) fn verifies_signature(
        &self,
        signing_bytes: &[u8],
        signature_bytes: &[u8; ED25519_SIGNATURE_BYTES],
    ) -> bool {
        VerifyingKey::from_bytes(&self.public_key).is_ok_and(|verifying_key| {
            verifying_key
                .verify_strict(signing_bytes, &Signature::from_bytes(signature_bytes))
                .is_ok()
        })
    }
}

impl Debug for TrustedReleasePublicKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TrustedReleasePublicKey")
            .field("key_id", &self.key_id)
            .field("minimum_generation", &self.minimum_generation)
            .field(
                "maximum_generation_exclusive",
                &self.maximum_generation_exclusive,
            )
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NativeReleaseVerificationContext<'a> {
    pub now_unix: u64,
    pub minimum_release_generation: u64,
    pub minimum_launch_grant_generation: u64,
    pub expected_artifact: &'a ArtifactIdentityV1,
    pub expected_channel: ReleaseChannel,
    pub requested_mode: GrantMode,
    pub requested_scope: IntegrationScope,
}

#[derive(Clone, Debug)]
pub struct VerifiedNativeReleaseEnvelope {
    signed: SignedNativeReleaseEnvelopeV1,
    signed_payload_sha256: String,
    release_key_id: String,
    verified_at_unix: u64,
    archive_target: NativePortableArchiveTarget,
    archive: VerifiedNativePortableArchive,
    launch_grant: VerifiedLaunchGrant,
}

impl VerifiedNativeReleaseEnvelope {
    #[must_use]
    pub const fn envelope(&self) -> &NativeReleaseEnvelopeV1 {
        &self.signed.envelope
    }

    #[must_use]
    pub const fn signed_envelope(&self) -> &SignedNativeReleaseEnvelopeV1 {
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
    pub const fn archive(&self) -> &VerifiedNativePortableArchive {
        &self.archive
    }

    #[must_use]
    pub const fn launch_grant(&self) -> &VerifiedLaunchGrant {
        &self.launch_grant
    }

    pub(crate) const fn archive_target(&self) -> &NativePortableArchiveTarget {
        &self.archive_target
    }
}

pub fn build_native_release_envelope(
    generation: u64,
    archive: &VerifiedNativePortableArchive,
    signed_launch_grant: &SignedLaunchGrantV1,
    not_before_unix: u64,
    expires_at_unix: u64,
) -> Result<NativeReleaseEnvelopeV1, NativeReleaseError> {
    let manifest = archive.payload().manifest();
    let envelope = NativeReleaseEnvelopeV1 {
        schema_version: NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION,
        generation,
        artifact: archive.artifact().clone(),
        archive_file_name: archive.file_name().to_string(),
        archive_size_bytes: archive.size_bytes(),
        archive_sha256: archive.archive_sha256().to_string(),
        artifact_manifest_sha256: archive.manifest_sha256().to_string(),
        resource_pack_sha256: manifest.content.pack_sha256.clone(),
        artifact_content_root_sha256: manifest.artifact_content_root_sha256.clone(),
        binary_sha256: manifest.binary_sha256.clone(),
        signed_launch_grant: signed_launch_grant.clone(),
        not_before_unix,
        expires_at_unix,
    };
    envelope.validate()?;
    Ok(envelope)
}

pub fn native_release_envelope_signing_bytes(
    envelope: &NativeReleaseEnvelopeV1,
) -> Result<Vec<u8>, NativeReleaseError> {
    envelope.validate()?;
    let canonical = serde_json_canonicalizer::to_vec(envelope)
        .map_err(|_| NativeReleaseError::CanonicalSerializationFailed)?;
    let mut output = Vec::with_capacity(SIGNING_DOMAIN.len().saturating_add(canonical.len()));
    output.extend_from_slice(SIGNING_DOMAIN);
    output.extend_from_slice(&canonical);
    Ok(output)
}

pub(crate) fn validate_release_keys(
    keys: &[TrustedReleasePublicKey],
) -> Result<(), NativeReleaseError> {
    if keys.len() > MAX_TRUSTED_RELEASE_KEYS {
        return Err(NativeReleaseError::InvalidReleaseKeySet);
    }
    for (index, key) in keys.iter().enumerate() {
        if keys[..index].iter().any(|prior| prior.key_id == key.key_id) {
            return Err(NativeReleaseError::InvalidReleaseKeySet);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeReleaseError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    UnsupportedSchema,
    InvalidEnvelope,
    InvalidTrustedReleaseKey,
    InvalidReleaseKeySet,
    ReleaseKeyUntrusted,
    ReleaseKeyGenerationUnavailable,
    ReleaseSignatureInvalid,
    ReleaseNotYetValid,
    ReleaseExpired,
    ReleaseReplayed,
    ReleaseChannelMismatch,
    ReleaseArtifactMismatch,
    ArchiveInvalid,
    ReleasePayloadMismatch,
    LaunchGrantInvalid,
    CanonicalSerializationFailed,
}

impl NativeReleaseError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "native-release-envelope-too-large",
            Self::InvalidJson => "native-release-envelope-json-invalid",
            Self::NonCanonicalJson => "native-release-envelope-json-noncanonical",
            Self::UnsupportedSchema => "native-release-envelope-schema-unsupported",
            Self::InvalidEnvelope => "native-release-envelope-invalid",
            Self::InvalidTrustedReleaseKey => "native-release-key-invalid",
            Self::InvalidReleaseKeySet => "native-release-key-set-invalid",
            Self::ReleaseKeyUntrusted => "native-release-key-untrusted",
            Self::ReleaseKeyGenerationUnavailable => "native-release-key-generation-unavailable",
            Self::ReleaseSignatureInvalid => "native-release-signature-invalid",
            Self::ReleaseNotYetValid => "native-release-not-yet-valid",
            Self::ReleaseExpired => "native-release-expired",
            Self::ReleaseReplayed => "native-release-generation-stale",
            Self::ReleaseChannelMismatch => "native-release-channel-mismatch",
            Self::ReleaseArtifactMismatch => "native-release-artifact-mismatch",
            Self::ArchiveInvalid => "native-release-archive-invalid",
            Self::ReleasePayloadMismatch => "native-release-payload-mismatch",
            Self::LaunchGrantInvalid => "native-release-launch-grant-invalid",
            Self::CanonicalSerializationFailed => "native-release-canonicalization-failed",
        }
    }
}

impl Display for NativeReleaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for NativeReleaseError {}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::Value;

    use super::*;
    use crate::{
        GrantSignatureV1, LaunchGrantV1, ReleaseChannel, current_target_native_artifact_identity,
        launch_grant_signing_bytes,
    };

    const NOW: u64 = 1_750_000_000;

    fn encode_hex<const N: usize>(bytes: &[u8; N]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(N * 2);
        for byte in bytes {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    fn signed_release() -> SignedNativeReleaseEnvelopeV1 {
        let artifact =
            current_target_native_artifact_identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .expect("current test target must resolve");
        let grant = LaunchGrantV1 {
            schema_version: 1,
            generation: 7,
            artifact: artifact.clone(),
            binary_sha256: "1".repeat(64),
            resource_pack_sha256: "2".repeat(64),
            allowed_modes: vec![GrantMode::LiteMcp],
            integration_scopes: vec![IntegrationScope::CodexLocal],
            not_before_unix: NOW - 60,
            expires_at_unix: NOW + 3_600,
        };
        let launch_key = SigningKey::from_bytes(&[31_u8; 32]);
        let launch_signature = launch_key.sign(&launch_grant_signing_bytes(&grant).unwrap());
        let signed_launch_grant = SignedLaunchGrantV1 {
            grant,
            signature: GrantSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "launch-test-key".to_string(),
                value_hex: encode_hex(&launch_signature.to_bytes()),
            },
        };
        let envelope = NativeReleaseEnvelopeV1 {
            schema_version: NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION,
            generation: 9,
            archive_file_name: native_portable_archive_file_name(&artifact).unwrap(),
            artifact,
            archive_size_bytes: 1_024,
            archive_sha256: "3".repeat(64),
            artifact_manifest_sha256: "4".repeat(64),
            resource_pack_sha256: "2".repeat(64),
            artifact_content_root_sha256: "5".repeat(64),
            binary_sha256: "1".repeat(64),
            signed_launch_grant,
            not_before_unix: NOW - 30,
            expires_at_unix: NOW + 1_800,
        };
        let release_key = SigningKey::from_bytes(&[32_u8; 32]);
        let signature =
            release_key.sign(&native_release_envelope_signing_bytes(&envelope).unwrap());
        SignedNativeReleaseEnvelopeV1 {
            envelope,
            signature: NativeReleaseSignatureV1 {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "release-test-key".to_string(),
                value_hex: encode_hex(&signature.to_bytes()),
            },
        }
    }

    #[test]
    fn signed_release_json_is_strict_bounded_and_canonical() {
        let release = signed_release();
        let canonical = release.to_canonical_json().unwrap();
        assert_eq!(
            SignedNativeReleaseEnvelopeV1::from_json(&canonical).unwrap(),
            release
        );
        assert!(
            native_release_envelope_signing_bytes(&release.envelope)
                .unwrap()
                .starts_with(SIGNING_DOMAIN)
        );

        let mut noncanonical = canonical.clone();
        noncanonical.push(b'\n');
        assert_eq!(
            SignedNativeReleaseEnvelopeV1::from_json(&noncanonical),
            Err(NativeReleaseError::NonCanonicalJson)
        );

        let mut unknown: Value = serde_json::from_slice(&canonical).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_string(), Value::Bool(true));
        assert_eq!(
            SignedNativeReleaseEnvelopeV1::from_json(&serde_json::to_vec(&unknown).unwrap()),
            Err(NativeReleaseError::InvalidJson)
        );
        assert_eq!(
            SignedNativeReleaseEnvelopeV1::from_json(&vec![b' '; 300 * 1024]),
            Err(NativeReleaseError::InputTooLarge)
        );
    }

    #[test]
    fn release_key_policy_is_windowed_bounded_and_redacted() {
        let signing_key = SigningKey::from_bytes(&[32_u8; 32]);
        let public_key = signing_key.verifying_key().to_bytes();
        let trusted = TrustedReleasePublicKey::new("release-key", public_key, 9, Some(12))
            .expect("valid release key must approve");
        assert!(trusted.authorizes_generation(9));
        assert!(trusted.authorizes_generation(11));
        assert!(!trusted.authorizes_generation(8));
        assert!(!trusted.authorizes_generation(12));
        assert!(!format!("{trusted:?}").contains("public_key"));

        assert_eq!(
            TrustedReleasePublicKey::new("INVALID", public_key, 9, None),
            Err(NativeReleaseError::InvalidTrustedReleaseKey)
        );
        assert_eq!(
            TrustedReleasePublicKey::new("release-key", public_key, 0, None),
            Err(NativeReleaseError::InvalidTrustedReleaseKey)
        );
        assert_eq!(
            TrustedReleasePublicKey::new("release-key", public_key, 9, Some(9)),
            Err(NativeReleaseError::InvalidTrustedReleaseKey)
        );
        assert_eq!(
            validate_release_keys(&[trusted.clone(), trusted]),
            Err(NativeReleaseError::InvalidReleaseKeySet)
        );
    }

    #[test]
    fn release_errors_expose_only_stable_reason_codes() {
        for error in [
            NativeReleaseError::ArchiveInvalid,
            NativeReleaseError::ReleasePayloadMismatch,
            NativeReleaseError::LaunchGrantInvalid,
        ] {
            assert_eq!(error.to_string(), error.reason_code());
            assert!(!error.to_string().contains('/'));
            assert!(!error.to_string().contains('\\'));
        }
    }
}
