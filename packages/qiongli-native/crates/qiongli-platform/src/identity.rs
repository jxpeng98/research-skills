use semver::Version;
use serde::{Deserialize, Serialize};

use crate::PlatformError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductId {
    Qiongli,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseChannel {
    Alpha,
    Beta,
    Stable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityProfile {
    SkillOnly,
    Lite,
    Full,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatingSystem {
    Macos,
    Windows,
    Linux,
}

impl OperatingSystem {
    #[must_use]
    pub const fn current() -> Option<Self> {
        if cfg!(target_os = "macos") {
            Some(Self::Macos)
        } else if cfg!(target_os = "windows") {
            Some(Self::Windows)
        } else if cfg!(target_os = "linux") {
            Some(Self::Linux)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Architecture {
    #[serde(rename = "aarch64")]
    Aarch64,
    #[serde(rename = "x86-64")]
    X86_64,
}

impl Architecture {
    #[must_use]
    pub const fn current() -> Option<Self> {
        if cfg!(target_arch = "aarch64") {
            Some(Self::Aarch64)
        } else if cfg!(target_arch = "x86_64") {
            Some(Self::X86_64)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallerKind {
    NativeInstaller,
    PortableArchive,
    PluginBundle,
    Mcpb,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactIdentityV1 {
    pub product: ProductId,
    pub version: String,
    pub channel: ReleaseChannel,
    pub profile: CapabilityProfile,
    pub os: OperatingSystem,
    pub arch: Architecture,
    pub installer_kind: InstallerKind,
}

impl ArtifactIdentityV1 {
    pub fn validate(&self) -> Result<(), PlatformError> {
        if self.version.is_empty() || self.version.len() > 64 || !self.version.is_ascii() {
            return Err(PlatformError::InvalidArtifactIdentity);
        }
        let version =
            Version::parse(&self.version).map_err(|_| PlatformError::InvalidArtifactIdentity)?;
        let prerelease = version.pre.as_str();
        let valid_channel = match self.channel {
            ReleaseChannel::Alpha => valid_numbered_prerelease(prerelease, "alpha"),
            ReleaseChannel::Beta => valid_numbered_prerelease(prerelease, "beta"),
            ReleaseChannel::Stable => prerelease.is_empty(),
        };
        if !valid_channel || !version.build.is_empty() {
            return Err(PlatformError::InvalidArtifactIdentity);
        }
        Ok(())
    }

    pub(crate) fn validate_lite(&self) -> Result<(), PlatformError> {
        self.validate()?;
        if self.profile != CapabilityProfile::Lite {
            return Err(PlatformError::UnsupportedArtifactProfile);
        }
        Ok(())
    }
}

// Local source identity describes bytes/platform only; it never verifies a release grant.
pub(crate) fn local_plugin_identity(version: &str) -> Result<ArtifactIdentityV1, PlatformError> {
    let parsed = Version::parse(version).map_err(|_| PlatformError::InvalidArtifactIdentity)?;
    let channel = if parsed.pre.as_str().starts_with("alpha.") {
        ReleaseChannel::Alpha
    } else if parsed.pre.as_str().starts_with("beta.") {
        ReleaseChannel::Beta
    } else {
        ReleaseChannel::Stable
    };
    let artifact = ArtifactIdentityV1 {
        product: ProductId::Qiongli,
        version: version.to_owned(),
        channel,
        profile: CapabilityProfile::Lite,
        os: OperatingSystem::current().ok_or(PlatformError::InvalidArtifactIdentity)?,
        arch: Architecture::current().ok_or(PlatformError::InvalidArtifactIdentity)?,
        installer_kind: InstallerKind::PluginBundle,
    };
    artifact.validate()?;
    Ok(artifact)
}

fn valid_numbered_prerelease(value: &str, channel: &str) -> bool {
    let Some(sequence) = value
        .strip_prefix(channel)
        .and_then(|value| value.strip_prefix('.'))
    else {
        return false;
    };
    !sequence.is_empty()
        && sequence.bytes().all(|byte| byte.is_ascii_digit())
        && sequence.parse::<u64>().is_ok_and(|number| number > 0)
        && (sequence == "0" || !sequence.starts_with('0'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(version: &str, channel: ReleaseChannel) -> ArtifactIdentityV1 {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: version.to_string(),
            channel,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::Linux,
            arch: Architecture::X86_64,
            installer_kind: InstallerKind::PortableArchive,
        }
    }

    #[test]
    fn channel_and_semver_must_match_exactly() {
        assert!(
            identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .validate()
                .is_ok()
        );
        assert!(
            identity("2.0.0-beta.3", ReleaseChannel::Beta)
                .validate()
                .is_ok()
        );
        assert!(identity("2.0.0", ReleaseChannel::Stable).validate().is_ok());

        for (version, channel) in [
            ("2.0.0", ReleaseChannel::Alpha),
            ("2.0.0-alpha.1", ReleaseChannel::Beta),
            ("2.0.0-alpha.0", ReleaseChannel::Alpha),
            ("2.0.0-alpha.01", ReleaseChannel::Alpha),
            ("2.0.0-alpha.1.extra", ReleaseChannel::Alpha),
            ("2.0.0+local", ReleaseChannel::Stable),
            (
                "2.0.0-alpha.1111111111111111111111111111111111111111111111111111111111111111",
                ReleaseChannel::Alpha,
            ),
        ] {
            assert_eq!(
                identity(version, channel).validate(),
                Err(PlatformError::InvalidArtifactIdentity)
            );
        }
    }

    #[test]
    fn lite_verification_rejects_higher_and_lower_profiles() {
        let mut value = identity("2.0.0-alpha.1", ReleaseChannel::Alpha);
        value.profile = CapabilityProfile::SkillOnly;
        assert_eq!(
            value.validate_lite(),
            Err(PlatformError::UnsupportedArtifactProfile)
        );
        value.profile = CapabilityProfile::Full;
        assert_eq!(
            value.validate_lite(),
            Err(PlatformError::UnsupportedArtifactProfile)
        );
    }
}
