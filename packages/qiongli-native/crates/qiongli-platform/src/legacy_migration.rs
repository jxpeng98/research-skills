use std::fmt::{self, Debug, Formatter};
use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};

use qiongli_config::{
    ConfigRoot, LEGACY_PROVIDER_CONFIG_FILE, LegacyProviderConfig, LegacyProviderResolution,
    inspect_legacy_provider_config, resolve_config_root,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{
    ClientActivationTarget, ClientInventory, ClientKind, ClientPathId, ClientPathState,
    VerifiedPackagedProduct, verify_packaged_product_install,
};

pub const LEGACY_MIGRATION_INVENTORY_SCHEMA_VERSION: u32 = 1;
pub const LEGACY_MIGRATION_PLAN_SCHEMA_VERSION: u32 = 2;
pub const LEGACY_MIGRATION_RECEIPT_SCHEMA_VERSION: u32 = 1;

const LEGACY_MIGRATION_PLAN_TTL_SECONDS: u64 = 15 * 60;
const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_MARKER_BYTES: u64 = 64 * 1024;
const MAX_SKILL_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SHARED_CONFIG_BYTES: u64 = 2 * 1024 * 1024;
const MAX_TREE_ENTRIES: usize = 8_192;
const MAX_TREE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_STORED_MIGRATIONS: usize = 64;
const LEGACY_CLEANUP_JOURNAL_SCHEMA_VERSION: u32 = 1;
const CODEX_LEGACY_MCP_BLOCK: &str = concat!(
    "# BEGIN QIONGLI MANAGED MCP\n",
    "[mcp_servers.qiongli]\n",
    "command = \"qiongli\"\n",
    "args = [\"mcp\", \"serve\", \"--transport\", \"stdio\"]\n",
    "# END QIONGLI MANAGED MCP\n"
);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationItemId {
    CodexPluginSource,
    CodexStandaloneSkills,
    CodexMarketplaceEntry,
    CodexStandaloneMcp,
    ClaudePluginSource,
    ClaudeStandaloneSkills,
    ClaudeMarketplaceEntry,
    ClaudeStandaloneMcp,
    LegacyProviderConfig,
}

impl LegacyMigrationItemId {
    #[must_use]
    pub const fn client(self) -> Option<ClientKind> {
        match self {
            Self::CodexPluginSource
            | Self::CodexStandaloneSkills
            | Self::CodexMarketplaceEntry
            | Self::CodexStandaloneMcp => Some(ClientKind::Codex),
            Self::ClaudePluginSource
            | Self::ClaudeStandaloneSkills
            | Self::ClaudeMarketplaceEntry
            | Self::ClaudeStandaloneMcp => Some(ClientKind::ClaudeCode),
            Self::LegacyProviderConfig => None,
        }
    }

    #[must_use]
    pub const fn path_id(self) -> Option<ClientPathId> {
        match self {
            Self::CodexPluginSource => Some(ClientPathId::CodexLegacyPluginSource),
            Self::CodexStandaloneSkills => Some(ClientPathId::CodexLegacySkills),
            Self::CodexMarketplaceEntry => Some(ClientPathId::CodexMarketplace),
            Self::CodexStandaloneMcp => Some(ClientPathId::CodexLegacyMcpConfig),
            Self::ClaudePluginSource => Some(ClientPathId::ClaudeLegacyPluginSource),
            Self::ClaudeStandaloneSkills => Some(ClientPathId::ClaudeLegacySkills),
            Self::ClaudeMarketplaceEntry => Some(ClientPathId::ClaudeMarketplace),
            Self::ClaudeStandaloneMcp => Some(ClientPathId::ClaudeLegacyMcpConfig),
            Self::LegacyProviderConfig => None,
        }
    }

    #[must_use]
    pub const fn symbolic_path(self) -> &'static str {
        match self {
            Self::CodexPluginSource => "<user-home>/.agents/plugins/qiongli",
            Self::CodexStandaloneSkills => "<codex-config>/skills/qiongli-workflow",
            Self::CodexMarketplaceEntry => "<user-home>/.agents/plugins/marketplace.json",
            Self::CodexStandaloneMcp => "<codex-config>/config.toml",
            Self::ClaudePluginSource => {
                "<user-home>/.qiongli/plugins/claude-code/qiongli-local/plugins/qiongli"
            }
            Self::ClaudeStandaloneSkills => "<claude-config>/skills/qiongli-workflow",
            Self::ClaudeMarketplaceEntry => {
                "<user-home>/.qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json"
            }
            Self::ClaudeStandaloneMcp => "<user-home>/.claude.json",
            Self::LegacyProviderConfig => "<qiongli-config>/providers.json",
        }
    }

    const fn is_plugin(self) -> bool {
        matches!(self, Self::CodexPluginSource | Self::ClaudePluginSource)
    }

    const fn is_marketplace(self) -> bool {
        matches!(
            self,
            Self::CodexMarketplaceEntry | Self::ClaudeMarketplaceEntry
        )
    }

    const fn is_mcp(self) -> bool {
        matches!(self, Self::CodexStandaloneMcp | Self::ClaudeStandaloneMcp)
    }

    const fn is_provider_config(self) -> bool {
        matches!(self, Self::LegacyProviderConfig)
    }

    const fn expected_platform(self) -> Option<&'static str> {
        match self.client() {
            Some(ClientKind::Codex) => Some("codex"),
            Some(ClientKind::ClaudeCode) => Some("claude"),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationClassification {
    UserData,
    SupportedSetting,
    Secret,
    GeneratedInstallation,
    HostRegistration,
    Ephemeral,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationAction {
    None,
    Convert,
    Regenerate,
    RemoveAfterVerify,
    Preserve,
    Review,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationItemState {
    Missing,
    Eligible,
    ReviewRequired,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationOwnershipEvidence {
    None,
    ManagedMarker,
    SkillManifest,
    MarketplaceEntry,
    ManagedMcp,
    LegacyProviderDocument,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationReadiness {
    NotDetected,
    Ready,
    ReviewRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyMigrationItemV1 {
    pub item_id: LegacyMigrationItemId,
    pub client: Option<ClientKind>,
    pub symbolic_path: String,
    pub classification: LegacyMigrationClassification,
    pub state: LegacyMigrationItemState,
    pub ownership_evidence: LegacyMigrationOwnershipEvidence,
    pub proposed_action: LegacyMigrationAction,
    pub content_sha256: Option<String>,
    pub container_sha256: Option<String>,
    pub reason_code: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyMigrationInventoryV1 {
    pub schema_version: u32,
    pub readiness: LegacyMigrationReadiness,
    pub detected_item_count: usize,
    pub eligible_item_count: usize,
    pub review_item_count: usize,
    pub items: Vec<LegacyMigrationItemV1>,
}

#[derive(Clone)]
pub struct LegacyMigrationInventory {
    summary: LegacyMigrationInventoryV1,
    home: PathBuf,
    private_paths: Vec<(LegacyMigrationItemId, PathBuf)>,
    legacy_provider_config: Option<LegacyProviderConfig>,
}

impl LegacyMigrationInventory {
    #[must_use]
    pub const fn summary(&self) -> &LegacyMigrationInventoryV1 {
        &self.summary
    }

    #[must_use]
    pub fn exact_path(&self, id: LegacyMigrationItemId) -> Option<&Path> {
        self.private_paths
            .iter()
            .find_map(|(candidate, path)| (*candidate == id).then_some(path.as_path()))
    }

    #[must_use]
    pub const fn legacy_provider_config(&self) -> Option<&LegacyProviderConfig> {
        self.legacy_provider_config.as_ref()
    }

    fn home(&self) -> &Path {
        &self.home
    }
}

impl Debug for LegacyMigrationInventory {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyMigrationInventory")
            .field("summary", &self.summary)
            .field("private_path_count", &self.private_paths.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationApproval {
    FilesystemWrite,
    ClientConfigChange,
    SecretStoreWrite,
    HostActivationConfirmed,
    GlobalSettingsVerified,
    LegacyCleanup,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationState {
    Detected,
    PreviewReady,
    Staged,
    AwaitingClientActivation,
    VerificationRequired,
    CleanupReady,
    Complete,
    RecoveryRequired,
    ReviewRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyMigrationPlanItemV1 {
    pub item_id: LegacyMigrationItemId,
    pub classification: LegacyMigrationClassification,
    pub action: LegacyMigrationAction,
    pub observed_sha256: Option<String>,
    pub observed_container_sha256: Option<String>,
    pub reason_code: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyMigrationPlanV1 {
    pub schema_version: u32,
    pub plan_id: String,
    pub product_version: String,
    pub source_commit: String,
    pub resource_pack_sha256: String,
    pub created_at_unix: u64,
    pub expires_at_unix: u64,
    pub inventory_sha256: String,
    pub required_approvals: Vec<LegacyMigrationApproval>,
    pub eligible_item_count: usize,
    pub review_item_count: usize,
    pub items: Vec<LegacyMigrationPlanItemV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_resolutions: Vec<LegacyProviderResolution>,
    pub plan_sha256: String,
}

impl LegacyMigrationPlanV1 {
    pub fn to_canonical_json(&self) -> Result<Vec<u8>, LegacyMigrationContractError> {
        validate_plan_fields(self)?;
        if self.plan_sha256 != plan_digest(self)? {
            return Err(LegacyMigrationContractError::DigestMismatch);
        }
        canonical_json(self)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, LegacyMigrationContractError> {
        if bytes.len() as u64 > MAX_SHARED_CONFIG_BYTES {
            return Err(LegacyMigrationContractError::DocumentTooLarge);
        }
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|_| LegacyMigrationContractError::DocumentInvalid)?;
        if value.to_canonical_json()?.as_slice() != bytes {
            return Err(LegacyMigrationContractError::NonCanonical);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyMigrationReceiptItemV1 {
    pub item_id: LegacyMigrationItemId,
    pub state: LegacyMigrationReceiptItemState,
    pub result_code: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LegacyMigrationReceiptItemState {
    Pending,
    Staged,
    AwaitingActivation,
    Verified,
    Cleaned,
    Preserved,
    ReviewRequired,
    RecoveryRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyMigrationReceiptV1 {
    pub schema_version: u32,
    pub migration_id: String,
    pub state: LegacyMigrationState,
    pub product_version: String,
    pub source_commit: String,
    pub resource_pack_sha256: String,
    pub plan_sha256: String,
    pub eligible_item_count: usize,
    pub completed_item_count: usize,
    pub unresolved_item_count: usize,
    pub items: Vec<LegacyMigrationReceiptItemV1>,
    pub receipt_sha256: String,
}

impl LegacyMigrationReceiptV1 {
    pub fn to_canonical_json(&self) -> Result<Vec<u8>, LegacyMigrationContractError> {
        validate_receipt_fields(self)?;
        if self.receipt_sha256 != receipt_digest(self)? {
            return Err(LegacyMigrationContractError::DigestMismatch);
        }
        canonical_json(self)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, LegacyMigrationContractError> {
        if bytes.len() as u64 > MAX_SHARED_CONFIG_BYTES {
            return Err(LegacyMigrationContractError::DocumentTooLarge);
        }
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|_| LegacyMigrationContractError::DocumentInvalid)?;
        if value.to_canonical_json()?.as_slice() != bytes {
            return Err(LegacyMigrationContractError::NonCanonical);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug)]
pub struct ApprovedLegacyMigrationPlan {
    plan: LegacyMigrationPlanV1,
    approvals: Vec<LegacyMigrationApproval>,
}

impl ApprovedLegacyMigrationPlan {
    #[must_use]
    pub const fn plan(&self) -> &LegacyMigrationPlanV1 {
        &self.plan
    }

    #[must_use]
    pub fn approvals(&self) -> &[LegacyMigrationApproval] {
        &self.approvals
    }

    #[must_use]
    pub fn has_approval(&self, approval: LegacyMigrationApproval) -> bool {
        self.approvals.contains(&approval)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyMigrationContractError {
    DocumentInvalid,
    DocumentTooLarge,
    NonCanonical,
    InvalidIdentity,
    InvalidTimestamp,
    InvalidInventory,
    ApprovalMissing,
    DigestMismatch,
}

#[derive(Clone)]
pub struct LegacyMigrationStore {
    root: PathBuf,
}

impl Debug for LegacyMigrationStore {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyMigrationStore")
            .finish_non_exhaustive()
    }
}

impl LegacyMigrationStore {
    pub fn for_inventory(
        inventory: &LegacyMigrationInventory,
    ) -> Result<Self, LegacyMigrationPersistenceError> {
        let home = inventory.home();
        if !home.is_absolute()
            || home
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(LegacyMigrationPersistenceError::UnsafePath);
        }
        let home =
            fs::canonicalize(home).map_err(|_| LegacyMigrationPersistenceError::PathUnavailable)?;
        if !fs::symlink_metadata(&home).is_ok_and(|metadata| {
            metadata.file_type().is_dir() && !metadata.file_type().is_symlink()
        }) {
            return Err(LegacyMigrationPersistenceError::UnsafePath);
        }
        Ok(Self {
            root: home.join(".qiongli/v2/migrations/1x-to-2x"),
        })
    }

    pub fn persist_preview(
        &self,
        plan: &LegacyMigrationPlanV1,
        receipt: &LegacyMigrationReceiptV1,
    ) -> Result<(), LegacyMigrationPersistenceError> {
        if receipt.migration_id != plan.plan_id
            || receipt.plan_sha256 != plan.plan_sha256
            || plan.to_canonical_json().is_err()
            || receipt.to_canonical_json().is_err()
        {
            return Err(LegacyMigrationPersistenceError::ContractInvalid);
        }
        let transaction = self.transaction_root(&plan.plan_id)?;
        prepare_private_directory(&self.root)?;
        if path_exists(&transaction)? {
            return Err(LegacyMigrationPersistenceError::Conflict);
        }
        create_private_directory(&transaction)?;
        let lock = StoreLock::acquire(&transaction)?;
        let result = (|| {
            write_new_private(
                &transaction.join("plan.json"),
                &plan
                    .to_canonical_json()
                    .map_err(|_| LegacyMigrationPersistenceError::ContractInvalid)?,
            )?;
            write_new_private(
                &transaction.join("receipt.json"),
                &receipt
                    .to_canonical_json()
                    .map_err(|_| LegacyMigrationPersistenceError::ContractInvalid)?,
            )
        })();
        drop(lock);
        if result.is_err() {
            let _ = fs::remove_dir_all(&transaction);
        }
        result
    }

    pub fn load_plan(
        &self,
        plan_id: &str,
    ) -> Result<LegacyMigrationPlanV1, LegacyMigrationPersistenceError> {
        let root = self.transaction_root(plan_id)?;
        let bytes = read_private_contract(&root.join("plan.json"))?;
        LegacyMigrationPlanV1::from_json(&bytes)
            .map_err(|_| LegacyMigrationPersistenceError::ContractInvalid)
    }

    pub fn load_receipt(
        &self,
        plan_id: &str,
    ) -> Result<LegacyMigrationReceiptV1, LegacyMigrationPersistenceError> {
        let root = self.transaction_root(plan_id)?;
        let bytes = read_private_contract(&root.join("receipt.json"))?;
        LegacyMigrationReceiptV1::from_json(&bytes)
            .map_err(|_| LegacyMigrationPersistenceError::ContractInvalid)
    }

    pub fn load_latest(
        &self,
    ) -> Result<
        Option<(LegacyMigrationPlanV1, LegacyMigrationReceiptV1)>,
        LegacyMigrationPersistenceError,
    > {
        let metadata = match fs::symlink_metadata(&self.root) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(LegacyMigrationPersistenceError::PathUnavailable),
        };
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err(LegacyMigrationPersistenceError::UnsafePath);
        }

        let entries = fs::read_dir(&self.root)
            .map_err(|_| LegacyMigrationPersistenceError::PathUnavailable)?;
        let mut latest: Option<(LegacyMigrationPlanV1, LegacyMigrationReceiptV1)> = None;
        let mut transaction_count = 0_usize;
        for entry in entries {
            let entry = entry.map_err(|_| LegacyMigrationPersistenceError::PathUnavailable)?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| LegacyMigrationPersistenceError::ContractInvalid)?;
            if name == ".DS_Store" {
                continue;
            }
            let file_type = entry
                .file_type()
                .map_err(|_| LegacyMigrationPersistenceError::PathUnavailable)?;
            if !file_type.is_dir() || file_type.is_symlink() || !valid_identifier(&name) {
                return Err(LegacyMigrationPersistenceError::ContractInvalid);
            }
            transaction_count += 1;
            if transaction_count > MAX_STORED_MIGRATIONS {
                return Err(LegacyMigrationPersistenceError::ContractInvalid);
            }

            let plan = self.load_plan(&name)?;
            let receipt = self.load_receipt(&name)?;
            if receipt.migration_id != plan.plan_id || receipt.plan_sha256 != plan.plan_sha256 {
                return Err(LegacyMigrationPersistenceError::ContractInvalid);
            }
            let replace = latest.as_ref().is_none_or(|(current, _)| {
                (plan.created_at_unix, plan.plan_id.as_str())
                    > (current.created_at_unix, current.plan_id.as_str())
            });
            if replace {
                latest = Some((plan, receipt));
            }
        }
        Ok(latest)
    }

    pub fn cleanup_journal_present(
        &self,
        plan_id: &str,
    ) -> Result<bool, LegacyMigrationPersistenceError> {
        let journal = self.transaction_root(plan_id)?.join("cleanup-journal.json");
        match fs::symlink_metadata(journal) {
            Ok(metadata)
                if metadata.file_type().is_file()
                    && !metadata.file_type().is_symlink()
                    && metadata.len() <= MAX_SHARED_CONFIG_BYTES =>
            {
                Ok(true)
            }
            Ok(_) => Err(LegacyMigrationPersistenceError::ContractInvalid),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(_) => Err(LegacyMigrationPersistenceError::PathUnavailable),
        }
    }

    pub fn replace_receipt(
        &self,
        expected_receipt_sha256: &str,
        receipt: &LegacyMigrationReceiptV1,
    ) -> Result<(), LegacyMigrationPersistenceError> {
        if !valid_lower_hex(expected_receipt_sha256, 64) || receipt.to_canonical_json().is_err() {
            return Err(LegacyMigrationPersistenceError::ContractInvalid);
        }
        let root = self.transaction_root(&receipt.migration_id)?;
        let _lock = StoreLock::acquire(&root)?;
        let observed = self.load_receipt(&receipt.migration_id)?;
        if observed.receipt_sha256 != expected_receipt_sha256 {
            return Err(LegacyMigrationPersistenceError::Conflict);
        }
        if !matches!(
            advance_legacy_migration_receipt(&observed, receipt.state, receipt.items.clone()),
            Ok(expected) if expected == *receipt
        ) {
            return Err(LegacyMigrationPersistenceError::ContractInvalid);
        }
        replace_private(
            &root.join("receipt.json"),
            &receipt
                .to_canonical_json()
                .map_err(|_| LegacyMigrationPersistenceError::ContractInvalid)?,
        )
    }

    fn transaction_root(&self, plan_id: &str) -> Result<PathBuf, LegacyMigrationPersistenceError> {
        if !valid_identifier(plan_id) {
            return Err(LegacyMigrationPersistenceError::UnsafePath);
        }
        Ok(self.root.join(plan_id))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyMigrationPersistenceError {
    UnsafePath,
    PathUnavailable,
    ContractInvalid,
    Conflict,
    PersistenceFailed,
}

impl LegacyMigrationPersistenceError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::UnsafePath => "legacy-migration-store-path-unsafe",
            Self::PathUnavailable => "legacy-migration-store-unavailable",
            Self::ContractInvalid => "legacy-migration-store-contract-invalid",
            Self::Conflict => "legacy-migration-store-conflict",
            Self::PersistenceFailed => "legacy-migration-store-write-failed",
        }
    }
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(root: &Path) -> Result<Self, LegacyMigrationPersistenceError> {
        let path = root.join(".lock");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        let mut file = options.open(&path).map_err(|error| match error.kind() {
            std::io::ErrorKind::AlreadyExists => LegacyMigrationPersistenceError::Conflict,
            _ => LegacyMigrationPersistenceError::PersistenceFailed,
        })?;
        file.write_all(b"qiongli-legacy-migration-lock\n")
            .and_then(|()| file.sync_all())
            .map_err(|_| LegacyMigrationPersistenceError::PersistenceFailed)?;
        Ok(Self { path })
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn prepare_private_directory(path: &Path) -> Result<(), LegacyMigrationPersistenceError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if matches!(
            component,
            std::path::Component::Prefix(_) | std::path::Component::RootDir
        ) {
            continue;
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            }
            Ok(_) => return Err(LegacyMigrationPersistenceError::UnsafePath),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                create_private_directory(&current)?;
            }
            Err(_) => return Err(LegacyMigrationPersistenceError::PathUnavailable),
        }
    }
    Ok(())
}

fn create_private_directory(path: &Path) -> Result<(), LegacyMigrationPersistenceError> {
    #[cfg(unix)]
    let mut builder = fs::DirBuilder::new();
    #[cfg(not(unix))]
    let builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    builder
        .create(path)
        .map_err(|_| LegacyMigrationPersistenceError::PersistenceFailed)
}

fn write_new_private(path: &Path, bytes: &[u8]) -> Result<(), LegacyMigrationPersistenceError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::AlreadyExists => LegacyMigrationPersistenceError::Conflict,
        _ => LegacyMigrationPersistenceError::PersistenceFailed,
    })?;
    if file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .is_err()
    {
        let _ = fs::remove_file(path);
        return Err(LegacyMigrationPersistenceError::PersistenceFailed);
    }
    sync_parent(path)
}

fn replace_private(path: &Path, bytes: &[u8]) -> Result<(), LegacyMigrationPersistenceError> {
    let staging = path.with_extension("json.stage");
    if path_exists(&staging)? {
        return Err(LegacyMigrationPersistenceError::Conflict);
    }
    write_new_private(&staging, bytes)?;
    if fs::rename(&staging, path).is_err() {
        let _ = fs::remove_file(&staging);
        return Err(LegacyMigrationPersistenceError::PersistenceFailed);
    }
    sync_parent(path)
}

fn read_private_contract(path: &Path) -> Result<Vec<u8>, LegacyMigrationPersistenceError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| LegacyMigrationPersistenceError::PathUnavailable)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_SHARED_CONFIG_BYTES
    {
        return Err(LegacyMigrationPersistenceError::ContractInvalid);
    }
    read_bounded(path, MAX_SHARED_CONFIG_BYTES)
        .map_err(|_| LegacyMigrationPersistenceError::ContractInvalid)
}

fn path_exists(path: &Path) -> Result<bool, LegacyMigrationPersistenceError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(LegacyMigrationPersistenceError::PathUnavailable),
    }
}

fn sync_parent(path: &Path) -> Result<(), LegacyMigrationPersistenceError> {
    let parent = path
        .parent()
        .ok_or(LegacyMigrationPersistenceError::UnsafePath)?;
    #[cfg(windows)]
    {
        // Rust's standard `File::open` cannot open a directory on Windows.
        // The file itself has already been flushed before this durability
        // boundary, so retain that guarantee and skip the unsupported parent
        // directory flush on this platform.
        let _ = parent;
        Ok(())
    }
    #[cfg(not(windows))]
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| LegacyMigrationPersistenceError::PersistenceFailed)
}

impl LegacyMigrationContractError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::DocumentInvalid => "legacy-migration-contract-invalid",
            Self::DocumentTooLarge => "legacy-migration-contract-too-large",
            Self::NonCanonical => "legacy-migration-contract-non-canonical",
            Self::InvalidIdentity => "legacy-migration-identity-invalid",
            Self::InvalidTimestamp => "legacy-migration-timestamp-invalid",
            Self::InvalidInventory => "legacy-migration-inventory-invalid",
            Self::ApprovalMissing => "legacy-migration-approval-missing",
            Self::DigestMismatch => "legacy-migration-digest-mismatch",
        }
    }
}

#[derive(Clone, Copy)]
pub struct LegacyMigrationPlanInput<'a> {
    pub plan_id: &'a str,
    pub product_version: &'a str,
    pub source_commit: &'a str,
    pub resource_pack_sha256: &'a str,
    pub created_at_unix: u64,
    pub provider_resolutions: &'a [LegacyProviderResolution],
}

pub fn preview_legacy_migration(
    inventory: &LegacyMigrationInventory,
    input: LegacyMigrationPlanInput<'_>,
) -> Result<LegacyMigrationPlanV1, LegacyMigrationContractError> {
    let inventory_sha256 = sha256_hex(&canonical_json(inventory.summary())?);
    let items = inventory
        .summary()
        .items
        .iter()
        .map(|item| LegacyMigrationPlanItemV1 {
            item_id: item.item_id,
            classification: item.classification,
            action: item.proposed_action,
            observed_sha256: item.content_sha256.clone(),
            observed_container_sha256: item.container_sha256.clone(),
            reason_code: item.reason_code.clone(),
        })
        .collect::<Vec<_>>();
    let mut required_approvals = Vec::new();
    if inventory.summary().eligible_item_count > 0 {
        required_approvals.push(LegacyMigrationApproval::FilesystemWrite);
        if items.iter().any(|item| {
            matches!(
                item.action,
                LegacyMigrationAction::Convert | LegacyMigrationAction::RemoveAfterVerify
            ) && matches!(
                item.classification,
                LegacyMigrationClassification::HostRegistration
                    | LegacyMigrationClassification::SupportedSetting
                    | LegacyMigrationClassification::Secret
            )
        }) {
            required_approvals.push(LegacyMigrationApproval::ClientConfigChange);
        }
        if items.iter().any(|item| {
            item.item_id.client().is_some()
                && matches!(
                    item.action,
                    LegacyMigrationAction::Convert
                        | LegacyMigrationAction::Regenerate
                        | LegacyMigrationAction::RemoveAfterVerify
                )
        }) {
            required_approvals.push(LegacyMigrationApproval::HostActivationConfirmed);
        }
        if items.iter().any(|item| {
            item.item_id.is_provider_config()
                && item.classification == LegacyMigrationClassification::Secret
                && item.action == LegacyMigrationAction::Convert
        }) {
            required_approvals.push(LegacyMigrationApproval::SecretStoreWrite);
        }
        if items.iter().any(|item| {
            item.item_id.is_provider_config() && item.action == LegacyMigrationAction::Convert
        }) {
            required_approvals.push(LegacyMigrationApproval::GlobalSettingsVerified);
        }
        required_approvals.sort_unstable();
        required_approvals.push(LegacyMigrationApproval::LegacyCleanup);
        required_approvals.sort_unstable();
    }
    let mut plan = LegacyMigrationPlanV1 {
        schema_version: LEGACY_MIGRATION_PLAN_SCHEMA_VERSION,
        plan_id: input.plan_id.to_owned(),
        product_version: input.product_version.to_owned(),
        source_commit: input.source_commit.to_owned(),
        resource_pack_sha256: input.resource_pack_sha256.to_owned(),
        created_at_unix: input.created_at_unix,
        expires_at_unix: input
            .created_at_unix
            .checked_add(LEGACY_MIGRATION_PLAN_TTL_SECONDS)
            .ok_or(LegacyMigrationContractError::InvalidTimestamp)?,
        inventory_sha256,
        required_approvals,
        eligible_item_count: inventory.summary().eligible_item_count,
        review_item_count: inventory.summary().review_item_count,
        items,
        provider_resolutions: input.provider_resolutions.to_vec(),
        plan_sha256: String::new(),
    };
    validate_plan_fields(&plan)?;
    plan.plan_sha256 = plan_digest(&plan)?;
    Ok(plan)
}

pub fn approve_legacy_migration_plan(
    plan: LegacyMigrationPlanV1,
    inventory: &LegacyMigrationInventory,
    now_unix: u64,
    approvals: &[LegacyMigrationApproval],
) -> Result<ApprovedLegacyMigrationPlan, LegacyMigrationContractError> {
    validate_plan_fields(&plan)?;
    if plan.plan_sha256 != plan_digest(&plan)? {
        return Err(LegacyMigrationContractError::DigestMismatch);
    }
    if now_unix < plan.created_at_unix || now_unix > plan.expires_at_unix {
        return Err(LegacyMigrationContractError::InvalidTimestamp);
    }
    let inventory_sha256 = sha256_hex(&canonical_json(inventory.summary())?);
    if plan.inventory_sha256 != inventory_sha256
        || plan.items.len() != inventory.summary().items.len()
        || plan
            .items
            .iter()
            .zip(&inventory.summary().items)
            .any(|(planned, observed)| {
                planned.item_id != observed.item_id
                    || planned.classification != observed.classification
                    || planned.action != observed.proposed_action
                    || planned.observed_sha256 != observed.content_sha256
                    || planned.observed_container_sha256 != observed.container_sha256
                    || planned.reason_code != observed.reason_code
            })
    {
        return Err(LegacyMigrationContractError::InvalidInventory);
    }
    if !unique_sorted_approvals(approvals)
        || approvals
            .iter()
            .any(|approval| !plan.required_approvals.contains(approval))
        || plan.required_approvals.iter().any(|required| {
            matches!(
                required,
                LegacyMigrationApproval::FilesystemWrite
                    | LegacyMigrationApproval::ClientConfigChange
                    | LegacyMigrationApproval::SecretStoreWrite
            ) && !approvals.contains(required)
        })
    {
        return Err(LegacyMigrationContractError::ApprovalMissing);
    }
    Ok(ApprovedLegacyMigrationPlan {
        plan,
        approvals: approvals.to_vec(),
    })
}

pub fn grant_legacy_migration_approval(
    mut approved: ApprovedLegacyMigrationPlan,
    approval: LegacyMigrationApproval,
) -> Result<ApprovedLegacyMigrationPlan, LegacyMigrationContractError> {
    if !approved.plan.required_approvals.contains(&approval) {
        return Err(LegacyMigrationContractError::ApprovalMissing);
    }
    if !approved.approvals.contains(&approval) {
        approved.approvals.push(approval);
        approved.approvals.sort_unstable();
    }
    Ok(approved)
}

#[derive(Clone, Debug)]
pub struct VerifiedLegacyMigrationCutover {
    approved: ApprovedLegacyMigrationPlan,
    verified_clients: Vec<ClientKind>,
}

impl VerifiedLegacyMigrationCutover {
    #[must_use]
    pub const fn approved_plan(&self) -> &ApprovedLegacyMigrationPlan {
        &self.approved
    }

    #[must_use]
    pub fn verified_clients(&self) -> &[ClientKind] {
        &self.verified_clients
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyMigrationCutoverError {
    ApprovalMissing,
    ProductIdentityMismatch,
    ProductVerificationFailed,
}

impl LegacyMigrationCutoverError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ApprovalMissing => "legacy-migration-host-approval-missing",
            Self::ProductIdentityMismatch => "legacy-migration-product-identity-mismatch",
            Self::ProductVerificationFailed => "legacy-migration-product-verification-failed",
        }
    }
}

pub fn verify_legacy_migration_cutover(
    approved: ApprovedLegacyMigrationPlan,
    product: &VerifiedPackagedProduct,
) -> Result<VerifiedLegacyMigrationCutover, LegacyMigrationCutoverError> {
    let client_cutover_required = approved.plan().items.iter().any(|item| {
        item.item_id.client().is_some()
            && matches!(
                item.action,
                LegacyMigrationAction::Convert
                    | LegacyMigrationAction::Regenerate
                    | LegacyMigrationAction::RemoveAfterVerify
            )
    });
    let global_settings_required = approved.plan().items.iter().any(|item| {
        item.item_id.is_provider_config() && item.action == LegacyMigrationAction::Convert
    });
    if (client_cutover_required
        && !approved.has_approval(LegacyMigrationApproval::HostActivationConfirmed))
        || (global_settings_required
            && !approved.has_approval(LegacyMigrationApproval::GlobalSettingsVerified))
    {
        return Err(LegacyMigrationCutoverError::ApprovalMissing);
    }
    let plan = approved.plan();
    if product.artifact().version != plan.product_version
        || product.product_source_commit() != plan.source_commit
        || product.resource_pack_sha256() != plan.resource_pack_sha256
    {
        return Err(LegacyMigrationCutoverError::ProductIdentityMismatch);
    }
    let mut verified_clients = Vec::new();
    for client in [ClientKind::Codex, ClientKind::ClaudeCode] {
        if plan.items.iter().any(|item| {
            item.item_id.client() == Some(client)
                && matches!(
                    item.action,
                    LegacyMigrationAction::Convert
                        | LegacyMigrationAction::Regenerate
                        | LegacyMigrationAction::RemoveAfterVerify
                )
        }) {
            let target = match client {
                ClientKind::Codex => ClientActivationTarget::Codex,
                ClientKind::ClaudeCode => ClientActivationTarget::ClaudeCode,
            };
            verify_packaged_product_install(product, target)
                .map_err(|_| LegacyMigrationCutoverError::ProductVerificationFailed)?;
            verified_clients.push(client);
        }
    }
    Ok(VerifiedLegacyMigrationCutover {
        approved,
        verified_clients,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyMigrationCleanupPreview {
    pub migration_id: String,
    pub item_count: usize,
    pub cleanup_sha256: String,
}

#[derive(Clone)]
pub struct PreparedLegacyMigrationCleanup {
    preview: LegacyMigrationCleanupPreview,
    transaction_root: PathBuf,
    entries: Vec<LegacyCleanupEntry>,
}

impl PreparedLegacyMigrationCleanup {
    #[must_use]
    pub const fn preview(&self) -> &LegacyMigrationCleanupPreview {
        &self.preview
    }
}

impl Debug for PreparedLegacyMigrationCleanup {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedLegacyMigrationCleanup")
            .field("preview", &self.preview)
            .field("entry_count", &self.entries.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyMigrationCleanupCommit {
    pub migration_id: String,
    pub cleanup_sha256: String,
    pub cleaned_items: Vec<LegacyMigrationItemId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyMigrationCleanupRecovery {
    pub migration_id: String,
    pub restored_items: Vec<LegacyMigrationItemId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyMigrationCleanupFinalization {
    pub migration_id: String,
    pub removed_compensation_items: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyMigrationCleanupError {
    ApprovalMissing,
    InventoryChanged,
    StoreInvalid,
    UnsafePath,
    BackupConflict,
    PersistenceFailed,
    CompensationFailed,
}

impl LegacyMigrationCleanupError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ApprovalMissing => "legacy-migration-cleanup-approval-missing",
            Self::InventoryChanged => "legacy-migration-cleanup-inventory-changed",
            Self::StoreInvalid => "legacy-migration-cleanup-store-invalid",
            Self::UnsafePath => "legacy-migration-cleanup-path-unsafe",
            Self::BackupConflict => "legacy-migration-cleanup-backup-conflict",
            Self::PersistenceFailed => "legacy-migration-cleanup-write-failed",
            Self::CompensationFailed => "legacy-migration-cleanup-recovery-required",
        }
    }
}

#[derive(Clone)]
struct LegacyCleanupEntry {
    item_id: LegacyMigrationItemId,
    path: PathBuf,
    content_sha256: String,
    container_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct LegacyCleanupJournalV1 {
    schema_version: u32,
    migration_id: String,
    cleanup_sha256: String,
    items: Vec<LegacyCleanupJournalItemV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct LegacyCleanupJournalItemV1 {
    item_id: LegacyMigrationItemId,
    backup_name: String,
    original_content_sha256: String,
    original_container_sha256: Option<String>,
    cleaned_container_sha256: Option<String>,
    applied: bool,
}

#[derive(Serialize)]
struct LegacyCleanupDigestItem<'a> {
    item_id: LegacyMigrationItemId,
    content_sha256: &'a str,
    container_sha256: Option<&'a str>,
}

pub fn prepare_legacy_migration_cleanup(
    cutover: &VerifiedLegacyMigrationCutover,
    inventory: &LegacyMigrationInventory,
) -> Result<PreparedLegacyMigrationCleanup, LegacyMigrationCleanupError> {
    let approved = cutover.approved_plan();
    if !approved.has_approval(LegacyMigrationApproval::LegacyCleanup) {
        return Err(LegacyMigrationCleanupError::ApprovalMissing);
    }
    let plan = approved.plan();
    if !staged_inventory_matches(plan, inventory) {
        return Err(LegacyMigrationCleanupError::InventoryChanged);
    }
    let store = LegacyMigrationStore::for_inventory(inventory)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    let persisted = store
        .load_plan(&plan.plan_id)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    if persisted != *plan {
        return Err(LegacyMigrationCleanupError::StoreInvalid);
    }
    let mut entries = Vec::new();
    for planned in &plan.items {
        if planned.action != LegacyMigrationAction::RemoveAfterVerify
            && !(planned.item_id.is_provider_config()
                && planned.action == LegacyMigrationAction::Convert)
        {
            continue;
        }
        if let Some(client) = planned.item_id.client()
            && !cutover.verified_clients.contains(&client)
        {
            return Err(LegacyMigrationCleanupError::InventoryChanged);
        }
        let path = inventory
            .exact_path(planned.item_id)
            .ok_or(LegacyMigrationCleanupError::UnsafePath)?
            .to_path_buf();
        let content_sha256 = planned
            .observed_sha256
            .clone()
            .ok_or(LegacyMigrationCleanupError::InventoryChanged)?;
        let observed = inventory
            .summary()
            .items
            .iter()
            .find(|observed| observed.item_id == planned.item_id)
            .ok_or(LegacyMigrationCleanupError::InventoryChanged)?;
        entries.push(LegacyCleanupEntry {
            item_id: planned.item_id,
            path,
            content_sha256,
            container_sha256: if planned.item_id.is_marketplace() || planned.item_id.is_mcp() {
                observed.container_sha256.clone()
            } else {
                planned.observed_container_sha256.clone()
            },
        });
    }
    let digest_items = entries
        .iter()
        .map(|entry| LegacyCleanupDigestItem {
            item_id: entry.item_id,
            content_sha256: &entry.content_sha256,
            container_sha256: entry.container_sha256.as_deref(),
        })
        .collect::<Vec<_>>();
    let cleanup_sha256 = sha256_hex(
        &canonical_json(&digest_items)
            .map_err(|_| LegacyMigrationCleanupError::InventoryChanged)?,
    );
    Ok(PreparedLegacyMigrationCleanup {
        preview: LegacyMigrationCleanupPreview {
            migration_id: plan.plan_id.clone(),
            item_count: entries.len(),
            cleanup_sha256,
        },
        transaction_root: store
            .transaction_root(&plan.plan_id)
            .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?,
        entries,
    })
}

pub fn apply_legacy_migration_cleanup(
    prepared: &PreparedLegacyMigrationCleanup,
) -> Result<LegacyMigrationCleanupCommit, LegacyMigrationCleanupError> {
    let _lock = StoreLock::acquire(&prepared.transaction_root)
        .map_err(|_| LegacyMigrationCleanupError::BackupConflict)?;
    let backup_root = prepared.transaction_root.join("cleanup-backup");
    let journal_path = prepared.transaction_root.join("cleanup-journal.json");
    if path_exists(&backup_root).map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?
        || path_exists(&journal_path).map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?
    {
        return Err(LegacyMigrationCleanupError::BackupConflict);
    }
    create_private_directory(&backup_root)
        .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;

    let mut replacements = Vec::with_capacity(prepared.entries.len());
    for entry in &prepared.entries {
        match prepare_cleanup_replacement(entry, &backup_root) {
            Ok(replacement) => replacements.push(replacement),
            Err(error) => {
                let _ = fs::remove_dir_all(&backup_root);
                return Err(error);
            }
        }
    }
    let mut journal = LegacyCleanupJournalV1 {
        schema_version: LEGACY_CLEANUP_JOURNAL_SCHEMA_VERSION,
        migration_id: prepared.preview.migration_id.clone(),
        cleanup_sha256: prepared.preview.cleanup_sha256.clone(),
        items: replacements
            .iter()
            .map(|replacement| replacement.journal.clone())
            .collect(),
    };
    write_new_private(
        &journal_path,
        &canonical_json(&journal).map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?,
    )
    .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;

    for (index, replacement) in replacements.iter().enumerate() {
        if apply_cleanup_replacement(replacement, &backup_root).is_err() {
            return if compensate_cleanup(&replacements[..index], &backup_root).is_ok() {
                let _ = fs::remove_file(&journal_path);
                let _ = fs::remove_dir_all(&backup_root);
                Err(LegacyMigrationCleanupError::PersistenceFailed)
            } else {
                Err(LegacyMigrationCleanupError::CompensationFailed)
            };
        }
        journal.items[index].applied = true;
        if replace_private(
            &journal_path,
            &canonical_json(&journal)
                .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?,
        )
        .is_err()
        {
            return if compensate_cleanup(&replacements[..=index], &backup_root).is_ok() {
                let _ = fs::remove_file(&journal_path);
                let _ = fs::remove_dir_all(&backup_root);
                Err(LegacyMigrationCleanupError::PersistenceFailed)
            } else {
                Err(LegacyMigrationCleanupError::CompensationFailed)
            };
        }
    }
    Ok(LegacyMigrationCleanupCommit {
        migration_id: prepared.preview.migration_id.clone(),
        cleanup_sha256: prepared.preview.cleanup_sha256.clone(),
        cleaned_items: prepared.entries.iter().map(|entry| entry.item_id).collect(),
    })
}

pub fn recover_legacy_migration_cleanup(
    inventory: &LegacyMigrationInventory,
    migration_id: &str,
) -> Result<LegacyMigrationCleanupRecovery, LegacyMigrationCleanupError> {
    let store = LegacyMigrationStore::for_inventory(inventory)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    let transaction_root = store
        .transaction_root(migration_id)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    let _lock = StoreLock::acquire(&transaction_root)
        .map_err(|_| LegacyMigrationCleanupError::BackupConflict)?;
    let journal_path = transaction_root.join("cleanup-journal.json");
    let backup_root = transaction_root.join("cleanup-backup");
    let bytes = read_private_contract(&journal_path)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    let journal: LegacyCleanupJournalV1 =
        serde_json::from_slice(&bytes).map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    if journal.schema_version != LEGACY_CLEANUP_JOURNAL_SCHEMA_VERSION
        || journal.migration_id != migration_id
        || !valid_lower_hex(&journal.cleanup_sha256, 64)
        || journal
            .items
            .windows(2)
            .any(|pair| pair[0].item_id >= pair[1].item_id)
        || canonical_json(&journal).map_err(|_| LegacyMigrationCleanupError::StoreInvalid)? != bytes
    {
        return Err(LegacyMigrationCleanupError::StoreInvalid);
    }
    let mut restored_items = Vec::new();
    for item in journal.items.iter().rev() {
        if item.backup_name != cleanup_backup_name(item.item_id)
            || !valid_lower_hex(&item.original_content_sha256, 64)
            || item
                .original_container_sha256
                .as_ref()
                .is_some_and(|digest| !valid_lower_hex(digest, 64))
            || item
                .cleaned_container_sha256
                .as_ref()
                .is_some_and(|digest| !valid_lower_hex(digest, 64))
        {
            return Err(LegacyMigrationCleanupError::StoreInvalid);
        }
        let original_path = inventory
            .exact_path(item.item_id)
            .ok_or(LegacyMigrationCleanupError::UnsafePath)?;
        let backup = backup_root.join(&item.backup_name);
        if item.item_id.is_provider_config() {
            let backup_bytes = read_bounded(&backup, MAX_SHARED_CONFIG_BYTES)
                .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
            if sha256_hex(&backup_bytes) != item.original_content_sha256 {
                return Err(LegacyMigrationCleanupError::CompensationFailed);
            }
            match (
                path_exists(original_path)
                    .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?,
                path_exists(&backup)
                    .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?,
            ) {
                (true, false) => {}
                (false, true) if current_path_is_file(&backup) => {
                    fs::rename(&backup, original_path)
                        .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
                    sync_parent(original_path)
                        .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
                    restored_items.push(item.item_id);
                }
                _ => return Err(LegacyMigrationCleanupError::CompensationFailed),
            }
        } else if item.item_id.is_marketplace() || item.item_id.is_mcp() {
            let backup_bytes = read_bounded(&backup, MAX_SHARED_CONFIG_BYTES)
                .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
            if item.original_container_sha256.as_deref() != Some(sha256_hex(&backup_bytes).as_str())
            {
                return Err(LegacyMigrationCleanupError::CompensationFailed);
            }
            let current = read_bounded(original_path, MAX_SHARED_CONFIG_BYTES)
                .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
            let current_sha256 = sha256_hex(&current);
            if item.original_container_sha256.as_deref() == Some(&current_sha256) {
                continue;
            }
            if item.cleaned_container_sha256.as_deref() != Some(&current_sha256) {
                return Err(LegacyMigrationCleanupError::CompensationFailed);
            }
            replace_shared_file(original_path, &backup_bytes)
                .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
            restored_items.push(item.item_id);
        } else {
            let original_exists = path_exists(original_path)
                .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
            let backup_exists = path_exists(&backup)
                .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
            match (original_exists, backup_exists) {
                (true, false) => {}
                (false, true) if current_path_is_directory(&backup) => {
                    if directory_sha256(&backup).ok().as_deref()
                        != Some(item.original_content_sha256.as_str())
                    {
                        return Err(LegacyMigrationCleanupError::CompensationFailed);
                    }
                    fs::rename(&backup, original_path)
                        .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
                    sync_parent(original_path)
                        .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
                    restored_items.push(item.item_id);
                }
                _ => return Err(LegacyMigrationCleanupError::CompensationFailed),
            }
        }
    }
    fs::remove_file(&journal_path).map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
    fs::remove_dir_all(&backup_root).map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
    sync_parent(&journal_path).map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
    restored_items.sort_unstable();
    Ok(LegacyMigrationCleanupRecovery {
        migration_id: migration_id.to_owned(),
        restored_items,
    })
}

pub fn finalize_legacy_migration_cleanup(
    inventory: &LegacyMigrationInventory,
    receipt: &LegacyMigrationReceiptV1,
) -> Result<LegacyMigrationCleanupFinalization, LegacyMigrationCleanupError> {
    if receipt.state != LegacyMigrationState::Complete
        || receipt.to_canonical_json().is_err()
        || receipt.unresolved_item_count != 0
    {
        return Err(LegacyMigrationCleanupError::StoreInvalid);
    }
    let store = LegacyMigrationStore::for_inventory(inventory)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    let persisted = store
        .load_receipt(&receipt.migration_id)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    if persisted != *receipt {
        return Err(LegacyMigrationCleanupError::StoreInvalid);
    }
    let transaction_root = store
        .transaction_root(&receipt.migration_id)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    let _lock = StoreLock::acquire(&transaction_root)
        .map_err(|_| LegacyMigrationCleanupError::BackupConflict)?;
    let journal_path = transaction_root.join("cleanup-journal.json");
    let backup_root = transaction_root.join("cleanup-backup");
    let bytes = read_private_contract(&journal_path)
        .map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    let journal: LegacyCleanupJournalV1 =
        serde_json::from_slice(&bytes).map_err(|_| LegacyMigrationCleanupError::StoreInvalid)?;
    if journal.schema_version != LEGACY_CLEANUP_JOURNAL_SCHEMA_VERSION
        || journal.migration_id != receipt.migration_id
        || journal.items.iter().any(|item| !item.applied)
        || canonical_json(&journal).map_err(|_| LegacyMigrationCleanupError::StoreInvalid)? != bytes
    {
        return Err(LegacyMigrationCleanupError::StoreInvalid);
    }
    for item in &journal.items {
        let observed = inventory
            .summary()
            .items
            .iter()
            .find(|observed| observed.item_id == item.item_id)
            .ok_or(LegacyMigrationCleanupError::InventoryChanged)?;
        if observed.state != LegacyMigrationItemState::Missing {
            return Err(LegacyMigrationCleanupError::InventoryChanged);
        }
        let backup = backup_root.join(&item.backup_name);
        if item.item_id.is_provider_config() {
            let backup_bytes = read_bounded(&backup, MAX_SHARED_CONFIG_BYTES)
                .map_err(|_| LegacyMigrationCleanupError::InventoryChanged)?;
            if sha256_hex(&backup_bytes) != item.original_content_sha256 {
                return Err(LegacyMigrationCleanupError::InventoryChanged);
            }
        } else if item.item_id.is_marketplace() || item.item_id.is_mcp() {
            let backup_bytes = read_bounded(&backup, MAX_SHARED_CONFIG_BYTES)
                .map_err(|_| LegacyMigrationCleanupError::InventoryChanged)?;
            if item.original_container_sha256.as_deref() != Some(sha256_hex(&backup_bytes).as_str())
            {
                return Err(LegacyMigrationCleanupError::InventoryChanged);
            }
        } else if directory_sha256(&backup).ok().as_deref()
            != Some(item.original_content_sha256.as_str())
        {
            return Err(LegacyMigrationCleanupError::InventoryChanged);
        }
    }
    fs::remove_file(&journal_path).map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
    fs::remove_dir_all(&backup_root).map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
    sync_parent(&journal_path).map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
    Ok(LegacyMigrationCleanupFinalization {
        migration_id: receipt.migration_id.clone(),
        removed_compensation_items: journal.items.len(),
    })
}

struct CleanupReplacement {
    original_path: PathBuf,
    cleaned_bytes: Option<Vec<u8>>,
    journal: LegacyCleanupJournalItemV1,
}

fn prepare_cleanup_replacement(
    entry: &LegacyCleanupEntry,
    backup_root: &Path,
) -> Result<CleanupReplacement, LegacyMigrationCleanupError> {
    let backup_name = cleanup_backup_name(entry.item_id).to_owned();
    if entry.item_id.is_provider_config() {
        if !current_cleanup_item_matches(entry) {
            return Err(LegacyMigrationCleanupError::InventoryChanged);
        }
        Ok(CleanupReplacement {
            original_path: entry.path.clone(),
            cleaned_bytes: None,
            journal: LegacyCleanupJournalItemV1 {
                item_id: entry.item_id,
                backup_name,
                original_content_sha256: entry.content_sha256.clone(),
                original_container_sha256: None,
                cleaned_container_sha256: None,
                applied: false,
            },
        })
    } else if entry.item_id.is_marketplace() || entry.item_id.is_mcp() {
        let original = read_bounded(&entry.path, MAX_SHARED_CONFIG_BYTES)
            .map_err(|_| LegacyMigrationCleanupError::InventoryChanged)?;
        if entry.container_sha256.as_deref() != Some(sha256_hex(&original).as_str())
            || !current_cleanup_item_matches(entry)
        {
            return Err(LegacyMigrationCleanupError::InventoryChanged);
        }
        let cleaned = clean_shared_config(entry.item_id, &original)?;
        let cleaned_container_sha256 = sha256_hex(&cleaned);
        write_new_private(&backup_root.join(&backup_name), &original)
            .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
        Ok(CleanupReplacement {
            original_path: entry.path.clone(),
            cleaned_bytes: Some(cleaned),
            journal: LegacyCleanupJournalItemV1 {
                item_id: entry.item_id,
                backup_name,
                original_content_sha256: entry.content_sha256.clone(),
                original_container_sha256: entry.container_sha256.clone(),
                cleaned_container_sha256: Some(cleaned_container_sha256),
                applied: false,
            },
        })
    } else {
        if !current_cleanup_item_matches(entry) {
            return Err(LegacyMigrationCleanupError::InventoryChanged);
        }
        Ok(CleanupReplacement {
            original_path: entry.path.clone(),
            cleaned_bytes: None,
            journal: LegacyCleanupJournalItemV1 {
                item_id: entry.item_id,
                backup_name,
                original_content_sha256: entry.content_sha256.clone(),
                original_container_sha256: None,
                cleaned_container_sha256: None,
                applied: false,
            },
        })
    }
}

fn current_cleanup_item_matches(entry: &LegacyCleanupEntry) -> bool {
    if entry.item_id.is_provider_config() {
        return current_path_is_file(&entry.path)
            && read_bounded(&entry.path, MAX_SHARED_CONFIG_BYTES)
                .is_ok_and(|bytes| sha256_hex(&bytes) == entry.content_sha256);
    }
    let observed = if entry.item_id.is_marketplace() {
        discover_marketplace_item(
            entry.item_id,
            Some(ClientPathState::File),
            Some(&entry.path),
        )
    } else if entry.item_id.is_mcp() {
        discover_mcp_item(
            entry.item_id,
            Some(ClientPathState::File),
            Some(&entry.path),
        )
    } else {
        discover_directory_item(
            entry.item_id,
            Some(ClientPathState::Directory),
            Some(&entry.path),
        )
    };
    observed.state == LegacyMigrationItemState::Eligible
        && observed.content_sha256.as_deref() == Some(&entry.content_sha256)
        && observed.container_sha256 == entry.container_sha256
}

fn clean_shared_config(
    item_id: LegacyMigrationItemId,
    original: &[u8],
) -> Result<Vec<u8>, LegacyMigrationCleanupError> {
    match item_id {
        LegacyMigrationItemId::CodexMarketplaceEntry
        | LegacyMigrationItemId::ClaudeMarketplaceEntry => {
            let mut document: serde_json::Value = serde_json::from_slice(original)
                .map_err(|_| LegacyMigrationCleanupError::InventoryChanged)?;
            let plugins = document
                .get_mut("plugins")
                .and_then(serde_json::Value::as_array_mut)
                .ok_or(LegacyMigrationCleanupError::InventoryChanged)?;
            let before = plugins.len();
            plugins.retain(|entry| {
                entry.get("name").and_then(serde_json::Value::as_str) != Some("qiongli")
            });
            if before.saturating_sub(plugins.len()) != 1 {
                return Err(LegacyMigrationCleanupError::InventoryChanged);
            }
            let mut bytes = serde_json::to_vec_pretty(&document)
                .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
            bytes.push(b'\n');
            Ok(bytes)
        }
        LegacyMigrationItemId::CodexStandaloneMcp => {
            let text = std::str::from_utf8(original)
                .map_err(|_| LegacyMigrationCleanupError::InventoryChanged)?;
            if text.matches(CODEX_LEGACY_MCP_BLOCK).count() != 1 {
                return Err(LegacyMigrationCleanupError::InventoryChanged);
            }
            Ok(text.replacen(CODEX_LEGACY_MCP_BLOCK, "", 1).into_bytes())
        }
        LegacyMigrationItemId::ClaudeStandaloneMcp => {
            let mut document: serde_json::Value = serde_json::from_slice(original)
                .map_err(|_| LegacyMigrationCleanupError::InventoryChanged)?;
            let servers = document
                .get_mut("mcpServers")
                .and_then(serde_json::Value::as_object_mut)
                .ok_or(LegacyMigrationCleanupError::InventoryChanged)?;
            if servers.remove("qiongli").is_none() {
                return Err(LegacyMigrationCleanupError::InventoryChanged);
            }
            let mut bytes = serde_json::to_vec_pretty(&document)
                .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
            bytes.push(b'\n');
            Ok(bytes)
        }
        _ => Err(LegacyMigrationCleanupError::InventoryChanged),
    }
}

fn apply_cleanup_replacement(
    replacement: &CleanupReplacement,
    backup_root: &Path,
) -> Result<(), LegacyMigrationCleanupError> {
    let backup = backup_root.join(&replacement.journal.backup_name);
    match replacement.cleaned_bytes.as_deref() {
        None => {
            if path_exists(&backup).map_err(|_| LegacyMigrationCleanupError::UnsafePath)?
                || !(if replacement.journal.item_id.is_provider_config() {
                    current_path_is_file(&replacement.original_path)
                } else {
                    current_path_is_directory(&replacement.original_path)
                })
            {
                return Err(LegacyMigrationCleanupError::InventoryChanged);
            }
            fs::rename(&replacement.original_path, &backup)
                .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
            sync_parent(&replacement.original_path)
                .and_then(|()| sync_parent(&backup))
                .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)
        }
        Some(cleaned) => {
            let observed = read_bounded(&replacement.original_path, MAX_SHARED_CONFIG_BYTES)
                .map_err(|_| LegacyMigrationCleanupError::InventoryChanged)?;
            if replacement.journal.original_container_sha256.as_deref()
                != Some(sha256_hex(&observed).as_str())
            {
                return Err(LegacyMigrationCleanupError::InventoryChanged);
            }
            replace_shared_file(&replacement.original_path, cleaned)
        }
    }
}

fn compensate_cleanup(
    replacements: &[CleanupReplacement],
    backup_root: &Path,
) -> Result<(), LegacyMigrationCleanupError> {
    for replacement in replacements.iter().rev() {
        let backup = backup_root.join(&replacement.journal.backup_name);
        match replacement.cleaned_bytes.as_deref() {
            None => {
                if path_exists(&replacement.original_path)
                    .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?
                    || !(if replacement.journal.item_id.is_provider_config() {
                        current_path_is_file(&backup)
                    } else {
                        current_path_is_directory(&backup)
                    })
                {
                    return Err(LegacyMigrationCleanupError::CompensationFailed);
                }
                fs::rename(&backup, &replacement.original_path)
                    .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
                sync_parent(&replacement.original_path)
                    .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
            }
            Some(_) => {
                let current = read_bounded(&replacement.original_path, MAX_SHARED_CONFIG_BYTES)
                    .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
                if replacement.journal.cleaned_container_sha256.as_deref()
                    != Some(sha256_hex(&current).as_str())
                {
                    return Err(LegacyMigrationCleanupError::CompensationFailed);
                }
                let original = read_bounded(&backup, MAX_SHARED_CONFIG_BYTES)
                    .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
                if replacement.journal.original_container_sha256.as_deref()
                    != Some(sha256_hex(&original).as_str())
                {
                    return Err(LegacyMigrationCleanupError::CompensationFailed);
                }
                replace_shared_file(&replacement.original_path, &original)
                    .map_err(|_| LegacyMigrationCleanupError::CompensationFailed)?;
            }
        }
    }
    Ok(())
}

fn replace_shared_file(path: &Path, bytes: &[u8]) -> Result<(), LegacyMigrationCleanupError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| LegacyMigrationCleanupError::UnsafePath)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(LegacyMigrationCleanupError::UnsafePath);
    }
    let parent = path
        .parent()
        .ok_or(LegacyMigrationCleanupError::UnsafePath)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(LegacyMigrationCleanupError::UnsafePath)?;
    let staging = parent.join(format!(".{file_name}.qiongli-legacy-cleanup-stage"));
    if path_exists(&staging).map_err(|_| LegacyMigrationCleanupError::UnsafePath)? {
        return Err(LegacyMigrationCleanupError::BackupConflict);
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options
        .open(&staging)
        .map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)?;
    if file
        .set_permissions(metadata.permissions())
        .and_then(|()| file.write_all(bytes))
        .and_then(|()| file.sync_all())
        .is_err()
    {
        let _ = fs::remove_file(&staging);
        return Err(LegacyMigrationCleanupError::PersistenceFailed);
    }
    drop(file);
    if fs::rename(&staging, path).is_err() {
        let _ = fs::remove_file(&staging);
        return Err(LegacyMigrationCleanupError::PersistenceFailed);
    }
    sync_parent(path).map_err(|_| LegacyMigrationCleanupError::PersistenceFailed)
}

fn current_path_is_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.file_type().is_dir() && !metadata.file_type().is_symlink())
}

fn current_path_is_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.file_type().is_file() && !metadata.file_type().is_symlink())
}

const fn cleanup_backup_name(item_id: LegacyMigrationItemId) -> &'static str {
    match item_id {
        LegacyMigrationItemId::CodexPluginSource => "codex-plugin-source",
        LegacyMigrationItemId::CodexStandaloneSkills => "codex-standalone-skills",
        LegacyMigrationItemId::CodexMarketplaceEntry => "codex-marketplace.json",
        LegacyMigrationItemId::CodexStandaloneMcp => "codex-mcp-config",
        LegacyMigrationItemId::ClaudePluginSource => "claude-plugin-source",
        LegacyMigrationItemId::ClaudeStandaloneSkills => "claude-standalone-skills",
        LegacyMigrationItemId::ClaudeMarketplaceEntry => "claude-marketplace.json",
        LegacyMigrationItemId::ClaudeStandaloneMcp => "claude-mcp-config.json",
        LegacyMigrationItemId::LegacyProviderConfig => "legacy-providers.json",
    }
}

pub fn initial_legacy_migration_receipt(
    approved: &ApprovedLegacyMigrationPlan,
) -> Result<LegacyMigrationReceiptV1, LegacyMigrationContractError> {
    initial_legacy_migration_receipt_from_plan(approved.plan())
}

pub fn initial_legacy_migration_receipt_from_plan(
    plan: &LegacyMigrationPlanV1,
) -> Result<LegacyMigrationReceiptV1, LegacyMigrationContractError> {
    plan.to_canonical_json()?;
    let items = plan
        .items
        .iter()
        .map(|item| {
            let (state, result_code) = match item.action {
                LegacyMigrationAction::Review => (
                    LegacyMigrationReceiptItemState::ReviewRequired,
                    "legacy-migration-item-review-required",
                ),
                LegacyMigrationAction::None | LegacyMigrationAction::Preserve => (
                    LegacyMigrationReceiptItemState::Preserved,
                    "legacy-migration-item-no-change",
                ),
                LegacyMigrationAction::Convert
                | LegacyMigrationAction::Regenerate
                | LegacyMigrationAction::RemoveAfterVerify => (
                    LegacyMigrationReceiptItemState::Pending,
                    "legacy-migration-item-pending",
                ),
            };
            LegacyMigrationReceiptItemV1 {
                item_id: item.item_id,
                state,
                result_code: result_code.to_owned(),
            }
        })
        .collect::<Vec<_>>();
    let mut receipt = LegacyMigrationReceiptV1 {
        schema_version: LEGACY_MIGRATION_RECEIPT_SCHEMA_VERSION,
        migration_id: plan.plan_id.clone(),
        state: if plan.eligible_item_count == 0 && plan.review_item_count > 0 {
            LegacyMigrationState::ReviewRequired
        } else {
            LegacyMigrationState::PreviewReady
        },
        product_version: plan.product_version.clone(),
        source_commit: plan.source_commit.clone(),
        resource_pack_sha256: plan.resource_pack_sha256.clone(),
        plan_sha256: plan.plan_sha256.clone(),
        eligible_item_count: plan.eligible_item_count,
        completed_item_count: 0,
        unresolved_item_count: plan.review_item_count,
        items,
        receipt_sha256: String::new(),
    };
    receipt.receipt_sha256 = receipt_digest(&receipt)?;
    Ok(receipt)
}

pub fn resume_legacy_migration_plan(
    plan: LegacyMigrationPlanV1,
    inventory: &LegacyMigrationInventory,
    receipt: &LegacyMigrationReceiptV1,
    approvals: &[LegacyMigrationApproval],
) -> Result<ApprovedLegacyMigrationPlan, LegacyMigrationContractError> {
    validate_plan_fields(&plan)?;
    if plan.plan_sha256 != plan_digest(&plan)?
        || receipt.plan_sha256 != plan.plan_sha256
        || receipt.migration_id != plan.plan_id
        || receipt.product_version != plan.product_version
        || receipt.source_commit != plan.source_commit
        || receipt.resource_pack_sha256 != plan.resource_pack_sha256
        || receipt.to_canonical_json().is_err()
        || matches!(
            receipt.state,
            LegacyMigrationState::Detected | LegacyMigrationState::PreviewReady
        )
        || !unique_sorted_approvals(approvals)
        || approvals
            .iter()
            .any(|approval| !plan.required_approvals.contains(approval))
    {
        return Err(LegacyMigrationContractError::DocumentInvalid);
    }
    // Apply may add the 2.x registration to a shared marketplace or MCP
    // container. The legacy item itself must remain byte-identical, while
    // product-owned additions to that shared container are allowed. After
    // cleanup, completion/finalization deliberately observes missing legacy
    // items and uses the cleanup journal instead.
    if !matches!(
        receipt.state,
        LegacyMigrationState::Complete | LegacyMigrationState::RecoveryRequired
    ) && !staged_inventory_matches(&plan, inventory)
    {
        return Err(LegacyMigrationContractError::InvalidInventory);
    }
    Ok(ApprovedLegacyMigrationPlan {
        plan,
        approvals: approvals.to_vec(),
    })
}

fn staged_inventory_matches(
    plan: &LegacyMigrationPlanV1,
    inventory: &LegacyMigrationInventory,
) -> bool {
    plan.items
        .iter()
        .filter(|planned| {
            matches!(
                planned.action,
                LegacyMigrationAction::Convert
                    | LegacyMigrationAction::Regenerate
                    | LegacyMigrationAction::RemoveAfterVerify
            )
        })
        .all(|planned| {
            inventory
                .summary()
                .items
                .iter()
                .find(|observed| observed.item_id == planned.item_id)
                .is_some_and(|observed| {
                    observed.state == LegacyMigrationItemState::Eligible
                        && observed.classification == planned.classification
                        && observed.proposed_action == planned.action
                        && observed.content_sha256 == planned.observed_sha256
                        && observed.reason_code == planned.reason_code
                        && ((planned.item_id.is_marketplace() || planned.item_id.is_mcp())
                            || observed.container_sha256 == planned.observed_container_sha256)
                })
        })
}

pub fn advance_legacy_migration_receipt(
    current: &LegacyMigrationReceiptV1,
    next_state: LegacyMigrationState,
    items: Vec<LegacyMigrationReceiptItemV1>,
) -> Result<LegacyMigrationReceiptV1, LegacyMigrationContractError> {
    current.to_canonical_json()?;
    if !valid_state_transition(current.state, next_state)
        || items.len() != current.items.len()
        || items
            .iter()
            .zip(&current.items)
            .any(|(next, prior)| next.item_id != prior.item_id)
    {
        return Err(LegacyMigrationContractError::DocumentInvalid);
    }
    let completed_item_count = items
        .iter()
        .filter(|item| item.state == LegacyMigrationReceiptItemState::Cleaned)
        .count();
    let unresolved_item_count = items
        .iter()
        .filter(|item| {
            matches!(
                item.state,
                LegacyMigrationReceiptItemState::ReviewRequired
                    | LegacyMigrationReceiptItemState::RecoveryRequired
            )
        })
        .count();
    if next_state == LegacyMigrationState::CleanupReady
        && items
            .iter()
            .filter(|item| {
                !matches!(
                    item.state,
                    LegacyMigrationReceiptItemState::Preserved
                        | LegacyMigrationReceiptItemState::ReviewRequired
                )
            })
            .any(|item| item.state != LegacyMigrationReceiptItemState::Verified)
        || next_state == LegacyMigrationState::Complete
            && (completed_item_count != current.eligible_item_count || unresolved_item_count != 0)
    {
        return Err(LegacyMigrationContractError::DocumentInvalid);
    }
    let mut next = current.clone();
    next.state = next_state;
    next.completed_item_count = completed_item_count;
    next.unresolved_item_count = unresolved_item_count;
    next.items = items;
    next.receipt_sha256 = String::new();
    validate_receipt_fields(&next)?;
    next.receipt_sha256 = receipt_digest(&next)?;
    Ok(next)
}

fn valid_state_transition(current: LegacyMigrationState, next: LegacyMigrationState) -> bool {
    current == next
        || matches!(
            (current, next),
            (
                LegacyMigrationState::Detected,
                LegacyMigrationState::PreviewReady | LegacyMigrationState::ReviewRequired
            ) | (
                LegacyMigrationState::PreviewReady,
                LegacyMigrationState::Staged
                    | LegacyMigrationState::ReviewRequired
                    | LegacyMigrationState::RecoveryRequired
            ) | (
                LegacyMigrationState::Staged,
                LegacyMigrationState::AwaitingClientActivation
                    | LegacyMigrationState::VerificationRequired
                    | LegacyMigrationState::RecoveryRequired
            ) | (
                LegacyMigrationState::AwaitingClientActivation,
                LegacyMigrationState::VerificationRequired | LegacyMigrationState::RecoveryRequired
            ) | (
                LegacyMigrationState::VerificationRequired,
                LegacyMigrationState::CleanupReady | LegacyMigrationState::RecoveryRequired
            ) | (
                LegacyMigrationState::CleanupReady,
                LegacyMigrationState::Complete | LegacyMigrationState::RecoveryRequired
            ) | (
                LegacyMigrationState::Complete,
                LegacyMigrationState::RecoveryRequired
            ) | (
                LegacyMigrationState::RecoveryRequired,
                LegacyMigrationState::Staged
                    | LegacyMigrationState::AwaitingClientActivation
                    | LegacyMigrationState::VerificationRequired
                    | LegacyMigrationState::CleanupReady
            )
        )
}

#[derive(Serialize)]
struct LegacyMigrationPlanDigest<'a> {
    schema_version: u32,
    plan_id: &'a str,
    product_version: &'a str,
    source_commit: &'a str,
    resource_pack_sha256: &'a str,
    created_at_unix: u64,
    expires_at_unix: u64,
    inventory_sha256: &'a str,
    required_approvals: &'a [LegacyMigrationApproval],
    eligible_item_count: usize,
    review_item_count: usize,
    items: &'a [LegacyMigrationPlanItemV1],
    #[serde(skip_serializing_if = "slice_is_empty")]
    provider_resolutions: &'a [LegacyProviderResolution],
}

#[derive(Serialize)]
struct LegacyMigrationReceiptDigest<'a> {
    schema_version: u32,
    migration_id: &'a str,
    state: LegacyMigrationState,
    product_version: &'a str,
    source_commit: &'a str,
    resource_pack_sha256: &'a str,
    plan_sha256: &'a str,
    eligible_item_count: usize,
    completed_item_count: usize,
    unresolved_item_count: usize,
    items: &'a [LegacyMigrationReceiptItemV1],
}

fn plan_digest(plan: &LegacyMigrationPlanV1) -> Result<String, LegacyMigrationContractError> {
    let payload = LegacyMigrationPlanDigest {
        schema_version: plan.schema_version,
        plan_id: &plan.plan_id,
        product_version: &plan.product_version,
        source_commit: &plan.source_commit,
        resource_pack_sha256: &plan.resource_pack_sha256,
        created_at_unix: plan.created_at_unix,
        expires_at_unix: plan.expires_at_unix,
        inventory_sha256: &plan.inventory_sha256,
        required_approvals: &plan.required_approvals,
        eligible_item_count: plan.eligible_item_count,
        review_item_count: plan.review_item_count,
        items: &plan.items,
        provider_resolutions: &plan.provider_resolutions,
    };
    Ok(sha256_hex(&canonical_json(&payload)?))
}

fn receipt_digest(
    receipt: &LegacyMigrationReceiptV1,
) -> Result<String, LegacyMigrationContractError> {
    let payload = LegacyMigrationReceiptDigest {
        schema_version: receipt.schema_version,
        migration_id: &receipt.migration_id,
        state: receipt.state,
        product_version: &receipt.product_version,
        source_commit: &receipt.source_commit,
        resource_pack_sha256: &receipt.resource_pack_sha256,
        plan_sha256: &receipt.plan_sha256,
        eligible_item_count: receipt.eligible_item_count,
        completed_item_count: receipt.completed_item_count,
        unresolved_item_count: receipt.unresolved_item_count,
        items: &receipt.items,
    };
    Ok(sha256_hex(&canonical_json(&payload)?))
}

fn validate_plan_fields(plan: &LegacyMigrationPlanV1) -> Result<(), LegacyMigrationContractError> {
    if !matches!(
        plan.schema_version,
        1 | LEGACY_MIGRATION_PLAN_SCHEMA_VERSION
    ) || (plan.schema_version == 1 && !plan.provider_resolutions.is_empty())
        || !valid_identifier(&plan.plan_id)
        || !valid_product_identity(
            &plan.product_version,
            &plan.source_commit,
            &plan.resource_pack_sha256,
        )
        || !valid_lower_hex(&plan.inventory_sha256, 64)
        || plan.created_at_unix > JCS_MAX_SAFE_INTEGER
        || plan.expires_at_unix > JCS_MAX_SAFE_INTEGER
        || plan.expires_at_unix
            != plan
                .created_at_unix
                .checked_add(LEGACY_MIGRATION_PLAN_TTL_SECONDS)
                .ok_or(LegacyMigrationContractError::InvalidTimestamp)?
        || !unique_sorted_approvals(&plan.required_approvals)
        || !unique_plan_items(&plan.items)
        || !plan
            .provider_resolutions
            .windows(2)
            .all(|pair| pair[0].provider < pair[1].provider)
        || plan.eligible_item_count
            != plan
                .items
                .iter()
                .filter(|item| {
                    matches!(
                        item.action,
                        LegacyMigrationAction::Convert
                            | LegacyMigrationAction::Regenerate
                            | LegacyMigrationAction::RemoveAfterVerify
                    )
                })
                .count()
        || plan.review_item_count
            != plan
                .items
                .iter()
                .filter(|item| item.action == LegacyMigrationAction::Review)
                .count()
        || plan.items.iter().any(|item| {
            !valid_reason_code(&item.reason_code)
                || item
                    .observed_sha256
                    .as_ref()
                    .is_some_and(|digest| !valid_lower_hex(digest, 64))
                || item
                    .observed_container_sha256
                    .as_ref()
                    .is_some_and(|digest| !valid_lower_hex(digest, 64))
        })
    {
        return Err(LegacyMigrationContractError::DocumentInvalid);
    }
    Ok(())
}

const fn slice_is_empty<T>(value: &&[T]) -> bool {
    value.is_empty()
}

fn validate_receipt_fields(
    receipt: &LegacyMigrationReceiptV1,
) -> Result<(), LegacyMigrationContractError> {
    if receipt.schema_version != LEGACY_MIGRATION_RECEIPT_SCHEMA_VERSION
        || !valid_identifier(&receipt.migration_id)
        || !valid_product_identity(
            &receipt.product_version,
            &receipt.source_commit,
            &receipt.resource_pack_sha256,
        )
        || !valid_lower_hex(&receipt.plan_sha256, 64)
        || receipt.completed_item_count > receipt.eligible_item_count
        || receipt.unresolved_item_count
            != receipt
                .items
                .iter()
                .filter(|item| {
                    matches!(
                        item.state,
                        LegacyMigrationReceiptItemState::ReviewRequired
                            | LegacyMigrationReceiptItemState::RecoveryRequired
                    )
                })
                .count()
        || !unique_receipt_items(&receipt.items)
        || receipt
            .items
            .iter()
            .any(|item| !valid_reason_code(&item.result_code))
    {
        return Err(LegacyMigrationContractError::DocumentInvalid);
    }
    Ok(())
}

fn valid_product_identity(version: &str, source_commit: &str, resource_pack_sha256: &str) -> bool {
    Version::parse(version).is_ok_and(|version| version.major == 2)
        && (7..=64).contains(&source_commit.len())
        && source_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && valid_lower_hex(resource_pack_sha256, 64)
}

fn valid_identifier(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_reason_code(value: &str) -> bool {
    valid_identifier(value)
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn unique_sorted_approvals(approvals: &[LegacyMigrationApproval]) -> bool {
    approvals.windows(2).all(|pair| pair[0] < pair[1])
}

fn unique_plan_items(items: &[LegacyMigrationPlanItemV1]) -> bool {
    items
        .windows(2)
        .all(|pair| pair[0].item_id < pair[1].item_id)
}

fn unique_receipt_items(items: &[LegacyMigrationReceiptItemV1]) -> bool {
    items
        .windows(2)
        .all(|pair| pair[0].item_id < pair[1].item_id)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, LegacyMigrationContractError> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|_| LegacyMigrationContractError::DocumentInvalid)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyMigrationError {
    PathUnavailable,
    DocumentInvalid,
    DocumentTooLarge,
    TreeTooLarge,
    UnsupportedPath,
}

impl LegacyMigrationError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::PathUnavailable => "legacy-migration-path-unavailable",
            Self::DocumentInvalid => "legacy-migration-document-invalid",
            Self::DocumentTooLarge => "legacy-migration-document-too-large",
            Self::TreeTooLarge => "legacy-migration-tree-too-large",
            Self::UnsupportedPath => "legacy-migration-path-unsupported",
        }
    }
}

#[must_use]
pub fn discover_legacy_migration(client_inventory: &ClientInventory) -> LegacyMigrationInventory {
    let config_root = resolve_config_root(None, client_inventory.home()).ok();
    discover_legacy_migration_with_config(client_inventory, config_root.as_ref())
}

#[must_use]
pub fn discover_legacy_migration_with_config(
    client_inventory: &ClientInventory,
    config_root: Option<&ConfigRoot>,
) -> LegacyMigrationInventory {
    let mut items = Vec::with_capacity(9);
    let mut private_paths = Vec::with_capacity(9);
    for item_id in [
        LegacyMigrationItemId::CodexPluginSource,
        LegacyMigrationItemId::CodexStandaloneSkills,
        LegacyMigrationItemId::CodexMarketplaceEntry,
        LegacyMigrationItemId::CodexStandaloneMcp,
        LegacyMigrationItemId::ClaudePluginSource,
        LegacyMigrationItemId::ClaudeStandaloneSkills,
        LegacyMigrationItemId::ClaudeMarketplaceEntry,
        LegacyMigrationItemId::ClaudeStandaloneMcp,
    ] {
        let path_id = item_id
            .path_id()
            .expect("client migration item always has a client inventory path");
        let observed = client_inventory
            .summary()
            .clients
            .iter()
            .flat_map(|client| client.paths.iter())
            .find(|candidate| candidate.id == path_id);
        let path = client_inventory.exact_path(path_id).map(Path::to_path_buf);
        if let Some(path) = path.as_ref() {
            private_paths.push((item_id, path.clone()));
        }
        let observed = observed.map(|value| value.state);
        items.push(if item_id.is_marketplace() {
            discover_marketplace_item(item_id, observed, path.as_deref())
        } else if item_id.is_mcp() {
            discover_mcp_item(item_id, observed, path.as_deref())
        } else {
            discover_directory_item(item_id, observed, path.as_deref())
        });
    }
    let (provider_item, legacy_provider_config) = discover_provider_config_item(config_root);
    if let Some(config_root) = config_root {
        private_paths.push((
            LegacyMigrationItemId::LegacyProviderConfig,
            config_root
                .compatibility_root()
                .join(LEGACY_PROVIDER_CONFIG_FILE),
        ));
    }
    items.push(provider_item);
    let detected_item_count = items
        .iter()
        .filter(|item| item.state != LegacyMigrationItemState::Missing)
        .count();
    let eligible_item_count = items
        .iter()
        .filter(|item| item.state == LegacyMigrationItemState::Eligible)
        .count();
    let review_item_count = items
        .iter()
        .filter(|item| {
            matches!(
                item.state,
                LegacyMigrationItemState::ReviewRequired | LegacyMigrationItemState::Unavailable
            )
        })
        .count();
    let readiness = if review_item_count > 0 {
        LegacyMigrationReadiness::ReviewRequired
    } else if detected_item_count > 0 {
        LegacyMigrationReadiness::Ready
    } else {
        LegacyMigrationReadiness::NotDetected
    };
    LegacyMigrationInventory {
        summary: LegacyMigrationInventoryV1 {
            schema_version: LEGACY_MIGRATION_INVENTORY_SCHEMA_VERSION,
            readiness,
            detected_item_count,
            eligible_item_count,
            review_item_count,
            items,
        },
        home: client_inventory.home().to_path_buf(),
        private_paths,
        legacy_provider_config,
    }
}

fn discover_provider_config_item(
    config_root: Option<&ConfigRoot>,
) -> (LegacyMigrationItemV1, Option<LegacyProviderConfig>) {
    let mut item = LegacyMigrationItemV1 {
        item_id: LegacyMigrationItemId::LegacyProviderConfig,
        client: None,
        symbolic_path: LegacyMigrationItemId::LegacyProviderConfig
            .symbolic_path()
            .to_owned(),
        classification: LegacyMigrationClassification::SupportedSetting,
        state: LegacyMigrationItemState::Missing,
        ownership_evidence: LegacyMigrationOwnershipEvidence::None,
        proposed_action: LegacyMigrationAction::None,
        content_sha256: None,
        container_sha256: None,
        reason_code: "legacy-migration-item-missing".to_owned(),
    };
    let Some(config_root) = config_root else {
        item.state = LegacyMigrationItemState::Unavailable;
        item.proposed_action = LegacyMigrationAction::Review;
        item.reason_code = "legacy-provider-config-root-unavailable".to_owned();
        return (item, None);
    };
    match inspect_legacy_provider_config(config_root) {
        Ok(None) => (item, None),
        Ok(Some(config)) => {
            item.classification = if config.summary().secret_count > 0 {
                LegacyMigrationClassification::Secret
            } else {
                LegacyMigrationClassification::SupportedSetting
            };
            item.state = LegacyMigrationItemState::Eligible;
            item.ownership_evidence = LegacyMigrationOwnershipEvidence::LegacyProviderDocument;
            item.proposed_action = LegacyMigrationAction::Convert;
            item.content_sha256 = Some(config.summary().content_sha256.clone());
            item.reason_code = "legacy-provider-config-eligible".to_owned();
            (item, Some(config))
        }
        Err(error) => {
            item.state = LegacyMigrationItemState::ReviewRequired;
            item.proposed_action = LegacyMigrationAction::Review;
            item.reason_code = error.reason_code().to_owned();
            (item, None)
        }
    }
}

fn discover_directory_item(
    item_id: LegacyMigrationItemId,
    observed: Option<ClientPathState>,
    path: Option<&Path>,
) -> LegacyMigrationItemV1 {
    let mut item = LegacyMigrationItemV1 {
        item_id,
        client: item_id.client(),
        symbolic_path: item_id.symbolic_path().to_owned(),
        classification: LegacyMigrationClassification::GeneratedInstallation,
        state: LegacyMigrationItemState::Missing,
        ownership_evidence: LegacyMigrationOwnershipEvidence::None,
        proposed_action: LegacyMigrationAction::None,
        content_sha256: None,
        container_sha256: None,
        reason_code: "legacy-migration-item-missing".to_owned(),
    };
    let Some(observed) = observed else {
        item.state = LegacyMigrationItemState::Unavailable;
        item.proposed_action = LegacyMigrationAction::Review;
        item.reason_code = "legacy-migration-inventory-incomplete".to_owned();
        return item;
    };
    match observed {
        ClientPathState::Missing => return item,
        ClientPathState::Symlink | ClientPathState::Invalid | ClientPathState::Unsafe => {
            item.state = LegacyMigrationItemState::ReviewRequired;
            item.proposed_action = LegacyMigrationAction::Review;
            item.reason_code = "legacy-migration-path-review-required".to_owned();
            return item;
        }
        ClientPathState::Unavailable => {
            item.state = LegacyMigrationItemState::Unavailable;
            item.proposed_action = LegacyMigrationAction::Review;
            item.reason_code = "legacy-migration-path-unavailable".to_owned();
            return item;
        }
        ClientPathState::File => {
            item.state = LegacyMigrationItemState::ReviewRequired;
            item.proposed_action = LegacyMigrationAction::Review;
            item.reason_code = "legacy-migration-directory-required".to_owned();
            return item;
        }
        ClientPathState::Directory => {}
    }
    let Some(path) = path else {
        item.state = LegacyMigrationItemState::Unavailable;
        item.proposed_action = LegacyMigrationAction::Review;
        item.reason_code = "legacy-migration-path-unavailable".to_owned();
        return item;
    };
    let ownership = if item_id.is_plugin() {
        verify_plugin_marker(
            path,
            item_id
                .expected_platform()
                .expect("plugin migration item always has a client"),
        )
        .map(|()| LegacyMigrationOwnershipEvidence::ManagedMarker)
    } else {
        verify_skill_manifest(path).map(|()| LegacyMigrationOwnershipEvidence::SkillManifest)
    };
    let evidence = match ownership {
        Ok(evidence) => evidence,
        Err(error) => {
            item.state = LegacyMigrationItemState::ReviewRequired;
            item.proposed_action = LegacyMigrationAction::Review;
            item.reason_code = error.reason_code().to_owned();
            return item;
        }
    };
    match directory_sha256(path) {
        Ok(digest) => {
            item.state = LegacyMigrationItemState::Eligible;
            item.ownership_evidence = evidence;
            item.proposed_action = LegacyMigrationAction::RemoveAfterVerify;
            item.content_sha256 = Some(digest);
            item.reason_code = "legacy-migration-item-eligible".to_owned();
        }
        Err(error) => {
            item.state = LegacyMigrationItemState::ReviewRequired;
            item.proposed_action = LegacyMigrationAction::Review;
            item.reason_code = error.reason_code().to_owned();
        }
    }
    item
}

fn discover_marketplace_item(
    item_id: LegacyMigrationItemId,
    observed: Option<ClientPathState>,
    path: Option<&Path>,
) -> LegacyMigrationItemV1 {
    let mut item = shared_config_item(item_id);
    let Some(path) = shared_config_path(&mut item, observed, path) else {
        return item;
    };
    let bytes = match read_bounded(path, MAX_SHARED_CONFIG_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => {
            require_review(&mut item, error.reason_code());
            return item;
        }
    };
    item.container_sha256 = Some(sha256_hex(&bytes));
    let document: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(document) => document,
        Err(_) => {
            require_review(
                &mut item,
                LegacyMigrationError::DocumentInvalid.reason_code(),
            );
            return item;
        }
    };
    let Some(plugins) = document
        .get("plugins")
        .and_then(serde_json::Value::as_array)
    else {
        require_review(&mut item, "legacy-migration-marketplace-invalid");
        return item;
    };
    let entries = plugins
        .iter()
        .filter(|entry| entry.get("name").and_then(serde_json::Value::as_str) == Some("qiongli"))
        .collect::<Vec<_>>();
    if entries.is_empty() {
        item.container_sha256 = None;
        return item;
    }
    if entries.len() != 1 || !recognized_legacy_marketplace_entry(item_id, entries[0]) {
        require_review(
            &mut item,
            "legacy-migration-marketplace-entry-review-required",
        );
        return item;
    }
    let canonical = match serde_json_canonicalizer::to_vec(entries[0]) {
        Ok(canonical) => canonical,
        Err(_) => {
            require_review(
                &mut item,
                LegacyMigrationError::DocumentInvalid.reason_code(),
            );
            return item;
        }
    };
    mark_eligible(
        &mut item,
        LegacyMigrationOwnershipEvidence::MarketplaceEntry,
        sha256_hex(&canonical),
    );
    item
}

fn discover_mcp_item(
    item_id: LegacyMigrationItemId,
    observed: Option<ClientPathState>,
    path: Option<&Path>,
) -> LegacyMigrationItemV1 {
    let mut item = shared_config_item(item_id);
    let Some(path) = shared_config_path(&mut item, observed, path) else {
        return item;
    };
    let bytes = match read_bounded(path, MAX_SHARED_CONFIG_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => {
            require_review(&mut item, error.reason_code());
            return item;
        }
    };
    item.container_sha256 = Some(sha256_hex(&bytes));
    match item_id {
        LegacyMigrationItemId::CodexStandaloneMcp => discover_codex_mcp_block(&mut item, &bytes),
        LegacyMigrationItemId::ClaudeStandaloneMcp => discover_claude_mcp_entry(&mut item, &bytes),
        _ => require_review(
            &mut item,
            LegacyMigrationError::DocumentInvalid.reason_code(),
        ),
    }
    if item.state == LegacyMigrationItemState::Missing {
        item.container_sha256 = None;
    }
    item
}

fn shared_config_item(item_id: LegacyMigrationItemId) -> LegacyMigrationItemV1 {
    LegacyMigrationItemV1 {
        item_id,
        client: item_id.client(),
        symbolic_path: item_id.symbolic_path().to_owned(),
        classification: LegacyMigrationClassification::HostRegistration,
        state: LegacyMigrationItemState::Missing,
        ownership_evidence: LegacyMigrationOwnershipEvidence::None,
        proposed_action: LegacyMigrationAction::None,
        content_sha256: None,
        container_sha256: None,
        reason_code: "legacy-migration-item-missing".to_owned(),
    }
}

fn shared_config_path<'a>(
    item: &mut LegacyMigrationItemV1,
    observed: Option<ClientPathState>,
    path: Option<&'a Path>,
) -> Option<&'a Path> {
    match observed {
        Some(ClientPathState::Missing) => None,
        Some(ClientPathState::File) => match path {
            Some(path) => Some(path),
            None => {
                item.state = LegacyMigrationItemState::Unavailable;
                item.proposed_action = LegacyMigrationAction::Review;
                item.reason_code = "legacy-migration-path-unavailable".to_owned();
                None
            }
        },
        Some(ClientPathState::Unavailable) | None => {
            item.state = LegacyMigrationItemState::Unavailable;
            item.proposed_action = LegacyMigrationAction::Review;
            item.reason_code = "legacy-migration-path-unavailable".to_owned();
            None
        }
        Some(
            ClientPathState::Directory
            | ClientPathState::Symlink
            | ClientPathState::Invalid
            | ClientPathState::Unsafe,
        ) => {
            require_review(item, "legacy-migration-shared-config-review-required");
            None
        }
    }
}

fn recognized_legacy_marketplace_entry(
    item_id: LegacyMigrationItemId,
    entry: &serde_json::Value,
) -> bool {
    match item_id {
        LegacyMigrationItemId::CodexMarketplaceEntry => {
            entry
                .get("source")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|source| {
                    source.get("source").and_then(serde_json::Value::as_str) == Some("local")
                        && source.get("path").and_then(serde_json::Value::as_str)
                            == Some("./plugins/qiongli")
                })
                && entry
                    .get("metadata")
                    .and_then(serde_json::Value::as_object)
                    .is_some_and(|metadata| {
                        metadata
                            .get("managedBy")
                            .and_then(serde_json::Value::as_str)
                            == Some("qiongli-cli")
                            && metadata.get("surface").and_then(serde_json::Value::as_str)
                                == Some("plugin")
                    })
        }
        LegacyMigrationItemId::ClaudeMarketplaceEntry => {
            entry.get("source").and_then(serde_json::Value::as_str) == Some("./plugins/qiongli")
                && entry
                    .get("version")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|version| Version::parse(version).ok())
                    .is_some_and(|version| version.major == 1)
        }
        _ => false,
    }
}

fn discover_codex_mcp_block(item: &mut LegacyMigrationItemV1, bytes: &[u8]) {
    let Ok(text) = std::str::from_utf8(bytes) else {
        require_review(item, LegacyMigrationError::DocumentInvalid.reason_code());
        return;
    };
    let begin_count = text.matches("# BEGIN QIONGLI MANAGED MCP").count();
    let end_count = text.matches("# END QIONGLI MANAGED MCP").count();
    let block_count = text.matches(CODEX_LEGACY_MCP_BLOCK).count();
    if begin_count == 0 && end_count == 0 && !text.contains("[mcp_servers.qiongli]") {
        return;
    }
    if begin_count == 1 && end_count == 1 && block_count == 1 {
        mark_eligible(
            item,
            LegacyMigrationOwnershipEvidence::ManagedMcp,
            sha256_hex(CODEX_LEGACY_MCP_BLOCK.as_bytes()),
        );
    } else {
        require_review(item, "legacy-migration-mcp-entry-review-required");
    }
}

fn discover_claude_mcp_entry(item: &mut LegacyMigrationItemV1, bytes: &[u8]) {
    let document: serde_json::Value = match serde_json::from_slice(bytes) {
        Ok(document) => document,
        Err(_) => {
            require_review(item, LegacyMigrationError::DocumentInvalid.reason_code());
            return;
        }
    };
    let Some(servers) = document
        .get("mcpServers")
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    let Some(entry) = servers.get("qiongli") else {
        return;
    };
    let expected_args = ["mcp", "serve", "--transport", "stdio"];
    let recognized = entry.get("command").and_then(serde_json::Value::as_str) == Some("qiongli")
        && entry.get("type").and_then(serde_json::Value::as_str) == Some("stdio")
        && entry
            .get("args")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|args| {
                args.len() == expected_args.len()
                    && args.iter().zip(expected_args).all(|(actual, expected)| {
                        actual.as_str().is_some_and(|actual| actual == expected)
                    })
            });
    if !recognized {
        require_review(item, "legacy-migration-mcp-entry-review-required");
        return;
    }
    match serde_json_canonicalizer::to_vec(entry) {
        Ok(canonical) => mark_eligible(
            item,
            LegacyMigrationOwnershipEvidence::ManagedMcp,
            sha256_hex(&canonical),
        ),
        Err(_) => require_review(item, LegacyMigrationError::DocumentInvalid.reason_code()),
    }
}

fn require_review(item: &mut LegacyMigrationItemV1, reason_code: &str) {
    item.state = LegacyMigrationItemState::ReviewRequired;
    item.proposed_action = LegacyMigrationAction::Review;
    item.reason_code = reason_code.to_owned();
}

fn mark_eligible(
    item: &mut LegacyMigrationItemV1,
    evidence: LegacyMigrationOwnershipEvidence,
    content_sha256: String,
) {
    item.state = LegacyMigrationItemState::Eligible;
    item.ownership_evidence = evidence;
    item.proposed_action = LegacyMigrationAction::RemoveAfterVerify;
    item.content_sha256 = Some(content_sha256);
    item.reason_code = "legacy-migration-item-eligible".to_owned();
}

fn verify_plugin_marker(path: &Path, expected_platform: &str) -> Result<(), LegacyMigrationError> {
    let bytes = read_bounded(&path.join(".qiongli-managed.json"), MAX_MARKER_BYTES)?;
    let marker: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| LegacyMigrationError::DocumentInvalid)?;
    if marker["managed_by"] != "qiongli-cli"
        || marker["plugin"] != "qiongli"
        || marker["surface"] != "plugin"
        || marker["platform"] != expected_platform
    {
        return Err(LegacyMigrationError::DocumentInvalid);
    }
    let version = marker["version"]
        .as_str()
        .ok_or(LegacyMigrationError::DocumentInvalid)?;
    let version = Version::parse(version).map_err(|_| LegacyMigrationError::DocumentInvalid)?;
    if version.major != 1 {
        return Err(LegacyMigrationError::DocumentInvalid);
    }
    Ok(())
}

fn verify_skill_manifest(path: &Path) -> Result<(), LegacyMigrationError> {
    let skill = read_bounded(&path.join("SKILL.md"), MAX_SKILL_MANIFEST_BYTES)?;
    let skill = std::str::from_utf8(&skill).map_err(|_| LegacyMigrationError::DocumentInvalid)?;
    let identifies_qiongli = skill.lines().take(24).any(|line| {
        let line = line.trim();
        line == "name: qiongli"
            || line == "name: qiongli-workflow"
            || line.starts_with("description: \"Qiongli version: v1.")
    });
    let contains_v1_identity =
        skill.contains("Qiongli version: v1.") || skill.contains("workflow version: `v1.");
    if !identifies_qiongli || !contains_v1_identity {
        return Err(LegacyMigrationError::DocumentInvalid);
    }
    Ok(())
}

fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, LegacyMigrationError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| LegacyMigrationError::PathUnavailable)?;
    if !metadata.file_type().is_file() {
        return Err(LegacyMigrationError::UnsupportedPath);
    }
    if metadata.len() > maximum {
        return Err(LegacyMigrationError::DocumentTooLarge);
    }
    let file = File::open(path).map_err(|_| LegacyMigrationError::PathUnavailable)?;
    let mut bytes = Vec::new();
    file.take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| LegacyMigrationError::PathUnavailable)?;
    if bytes.len() as u64 > maximum {
        return Err(LegacyMigrationError::DocumentTooLarge);
    }
    Ok(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn directory_sha256(root: &Path) -> Result<String, LegacyMigrationError> {
    let metadata = fs::symlink_metadata(root).map_err(|_| LegacyMigrationError::PathUnavailable)?;
    if !metadata.file_type().is_dir() {
        return Err(LegacyMigrationError::UnsupportedPath);
    }
    let mut entries = Vec::new();
    collect_tree(root, root, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut total_bytes = 0_u64;
    let mut hasher = Sha256::new();
    for (relative, path, is_directory) in entries {
        if is_directory {
            hasher.update(b"d\0");
            hasher.update(relative.as_bytes());
            hasher.update(b"\0");
            continue;
        }
        let metadata =
            fs::symlink_metadata(&path).map_err(|_| LegacyMigrationError::PathUnavailable)?;
        total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or(LegacyMigrationError::TreeTooLarge)?;
        if total_bytes > MAX_TREE_BYTES {
            return Err(LegacyMigrationError::TreeTooLarge);
        }
        hasher.update(b"f\0");
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        hasher.update(metadata.len().to_be_bytes());
        let mut file = File::open(&path).map_err(|_| LegacyMigrationError::PathUnavailable)?;
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|_| LegacyMigrationError::PathUnavailable)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_tree(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<(String, PathBuf, bool)>,
) -> Result<(), LegacyMigrationError> {
    let reader = fs::read_dir(directory).map_err(|_| LegacyMigrationError::PathUnavailable)?;
    for entry in reader {
        let entry = entry.map_err(|_| LegacyMigrationError::PathUnavailable)?;
        if entries.len() >= MAX_TREE_ENTRIES {
            return Err(LegacyMigrationError::TreeTooLarge);
        }
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|_| LegacyMigrationError::PathUnavailable)?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| LegacyMigrationError::UnsupportedPath)?;
        let relative = relative
            .to_str()
            .ok_or(LegacyMigrationError::UnsupportedPath)?
            .replace(std::path::MAIN_SEPARATOR, "/");
        if relative.is_empty()
            || relative.starts_with('/')
            || relative
                .split('/')
                .any(|part| part.is_empty() || part == "..")
        {
            return Err(LegacyMigrationError::UnsupportedPath);
        }
        if metadata.file_type().is_symlink() {
            return Err(LegacyMigrationError::UnsupportedPath);
        }
        if metadata.file_type().is_dir() {
            entries.push((relative, path.clone(), true));
            collect_tree(root, &path, entries)?;
        } else if metadata.file_type().is_file() {
            entries.push((relative, path, false));
        } else {
            return Err(LegacyMigrationError::UnsupportedPath);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::{ClientInventoryInput, discover_client_inventory};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        home: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "qiongli-legacy-migration-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let home = root.join("home");
            fs::create_dir(&home).unwrap();
            Self { root, home }
        }

        fn inventory(&self) -> LegacyMigrationInventory {
            let clients = discover_client_inventory(ClientInventoryInput::new(&self.home));
            discover_legacy_migration(&clients)
        }

        fn write_plugin(&self, relative: &str, platform: &str) {
            let plugin = self.home.join(relative);
            fs::create_dir_all(plugin.join("skills/qiongli-workflow")).unwrap();
            let marker = serde_json::json!({
                "managed_by": "qiongli-cli",
                "plugin": "qiongli",
                "surface": "plugin",
                "platform": platform,
                "version": "1.19.0-beta.1"
            });
            fs::write(
                plugin.join(".qiongli-managed.json"),
                serde_json::to_vec(&marker).unwrap(),
            )
            .unwrap();
            fs::write(plugin.join("skills/qiongli-workflow/data"), b"legacy").unwrap();
        }

        fn write_skill(&self, relative: &str) {
            let skill = self.home.join(relative);
            fs::create_dir_all(&skill).unwrap();
            fs::write(
                skill.join("SKILL.md"),
                b"---\nname: qiongli\ndescription: \"Qiongli version: v1.19.0-beta.1\"\n---\n",
            )
            .unwrap();
        }

        fn write_marketplaces(&self) {
            let codex = self.home.join(".agents/plugins/marketplace.json");
            fs::create_dir_all(codex.parent().unwrap()).unwrap();
            fs::write(
                codex,
                serde_json::to_vec(&serde_json::json!({
                    "name": "personal",
                    "plugins": [{
                        "name": "qiongli",
                        "source": {"source": "local", "path": "./plugins/qiongli"},
                        "metadata": {"managedBy": "qiongli-cli", "surface": "plugin"}
                    }]
                }))
                .unwrap(),
            )
            .unwrap();

            let claude = self
                .home
                .join(".qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json");
            fs::create_dir_all(claude.parent().unwrap()).unwrap();
            fs::write(
                claude,
                serde_json::to_vec(&serde_json::json!({
                    "name": "qiongli-local",
                    "plugins": [{
                        "name": "qiongli",
                        "version": "1.19.0-beta.1",
                        "source": "./plugins/qiongli"
                    }]
                }))
                .unwrap(),
            )
            .unwrap();
        }

        fn add_current_marketplace_entries(&self) {
            for (relative, entry) in [
                (
                    ".agents/plugins/marketplace.json",
                    serde_json::json!({
                        "name": "qiongli-next",
                        "source": {"source": "local", "path": "./plugins/qiongli-next"},
                        "metadata": {"managedBy": "qiongli", "surface": "plugin"}
                    }),
                ),
                (
                    ".qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json",
                    serde_json::json!({
                        "name": "qiongli-next",
                        "version": "2.0.0-alpha.2",
                        "source": "./plugins/qiongli-next"
                    }),
                ),
            ] {
                let path = self.home.join(relative);
                let mut document: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                document["plugins"].as_array_mut().unwrap().push(entry);
                fs::write(path, serde_json::to_vec(&document).unwrap()).unwrap();
            }
        }

        fn write_standalone_mcp(&self) {
            let codex = self.home.join(".codex/config.toml");
            fs::create_dir_all(codex.parent().unwrap()).unwrap();
            fs::write(
                codex,
                concat!(
                    "model = \"host-owned\"\n\n",
                    "# BEGIN QIONGLI MANAGED MCP\n",
                    "[mcp_servers.qiongli]\n",
                    "command = \"qiongli\"\n",
                    "args = [\"mcp\", \"serve\", \"--transport\", \"stdio\"]\n",
                    "# END QIONGLI MANAGED MCP\n"
                ),
            )
            .unwrap();
            fs::write(
                self.home.join(".claude.json"),
                serde_json::to_vec(&serde_json::json!({
                    "theme": "dark",
                    "mcpServers": {
                        "qiongli": {
                            "command": "qiongli",
                            "args": ["mcp", "serve", "--transport", "stdio"],
                            "type": "stdio"
                        }
                    }
                }))
                .unwrap(),
            )
            .unwrap();
        }

        fn write_provider_config(&self) {
            let path = self.home.join(".config/qiongli/providers.json");
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(
                &path,
                br#"{
  "version": 1,
  "providers": {
    "crossref": {"email": "researcher@example.org"},
    "arxiv": {"enabled": false}
  }
}"#,
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;

                fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).ok();
        }
    }

    #[test]
    fn empty_home_has_no_migration() {
        let fixture = Fixture::new("empty");
        let inventory = fixture.inventory();
        assert_eq!(
            inventory.summary().readiness,
            LegacyMigrationReadiness::NotDetected
        );
        assert_eq!(inventory.summary().detected_item_count, 0);
        assert_eq!(inventory.summary().eligible_item_count, 0);
    }

    #[test]
    fn recognized_plugins_and_skills_are_eligible() {
        let fixture = Fixture::new("eligible");
        fixture.write_plugin(".agents/plugins/qiongli", "codex");
        fixture.write_plugin(
            ".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli",
            "claude",
        );
        fixture.write_skill(".codex/skills/qiongli-workflow");
        fixture.write_skill(".claude/skills/qiongli-workflow");

        let inventory = fixture.inventory();
        assert_eq!(
            inventory.summary().readiness,
            LegacyMigrationReadiness::Ready
        );
        assert_eq!(inventory.summary().detected_item_count, 4);
        assert_eq!(inventory.summary().eligible_item_count, 4);
        assert_eq!(inventory.summary().review_item_count, 0);
        assert!(
            inventory
                .summary()
                .items
                .iter()
                .filter(|item| item.state == LegacyMigrationItemState::Eligible)
                .all(|item| {
                    item.content_sha256
                        .as_ref()
                        .is_some_and(|value| value.len() == 64)
                })
        );
        assert!(!format!("{inventory:?}").contains(fixture.home.to_string_lossy().as_ref()));
    }

    #[test]
    fn recognized_shared_entries_are_item_scoped_and_eligible() {
        let fixture = Fixture::new("shared");
        fixture.write_marketplaces();
        fixture.write_standalone_mcp();

        let inventory = fixture.inventory();
        assert_eq!(
            inventory.summary().readiness,
            LegacyMigrationReadiness::Ready
        );
        assert_eq!(inventory.summary().detected_item_count, 4);
        assert_eq!(inventory.summary().eligible_item_count, 4);
        for item_id in [
            LegacyMigrationItemId::CodexMarketplaceEntry,
            LegacyMigrationItemId::CodexStandaloneMcp,
            LegacyMigrationItemId::ClaudeMarketplaceEntry,
            LegacyMigrationItemId::ClaudeStandaloneMcp,
        ] {
            let item = inventory
                .summary()
                .items
                .iter()
                .find(|item| item.item_id == item_id)
                .unwrap();
            assert_eq!(
                item.classification,
                LegacyMigrationClassification::HostRegistration
            );
            assert_eq!(item.state, LegacyMigrationItemState::Eligible);
            assert_eq!(item.content_sha256.as_ref().map(String::len), Some(64));
            assert_eq!(item.container_sha256.as_ref().map(String::len), Some(64));
        }
    }

    #[test]
    fn drifted_shared_entry_requires_review_without_blocking_known_items() {
        let fixture = Fixture::new("shared-drift");
        fixture.write_marketplaces();
        fixture.write_standalone_mcp();
        fs::write(
            fixture.home.join(".claude.json"),
            br#"{"mcpServers":{"qiongli":{"command":"custom-wrapper","args":[]}}}"#,
        )
        .unwrap();

        let inventory = fixture.inventory();
        assert_eq!(
            inventory.summary().readiness,
            LegacyMigrationReadiness::ReviewRequired
        );
        assert_eq!(inventory.summary().eligible_item_count, 3);
        assert_eq!(inventory.summary().review_item_count, 1);
        let item = inventory
            .summary()
            .items
            .iter()
            .find(|item| item.item_id == LegacyMigrationItemId::ClaudeStandaloneMcp)
            .unwrap();
        assert_eq!(item.state, LegacyMigrationItemState::ReviewRequired);
        assert_eq!(item.proposed_action, LegacyMigrationAction::Review);
    }

    #[test]
    fn unproven_or_symlinked_content_requires_review() {
        let fixture = Fixture::new("review");
        let unproven = fixture.home.join(".agents/plugins/qiongli");
        fs::create_dir_all(&unproven).unwrap();
        fs::write(unproven.join("custom.txt"), b"user-owned").unwrap();

        let inventory = fixture.inventory();
        assert_eq!(
            inventory.summary().readiness,
            LegacyMigrationReadiness::ReviewRequired
        );
        let item = inventory
            .summary()
            .items
            .iter()
            .find(|item| item.item_id == LegacyMigrationItemId::CodexPluginSource)
            .unwrap();
        assert_eq!(item.state, LegacyMigrationItemState::ReviewRequired);
        assert_eq!(item.proposed_action, LegacyMigrationAction::Review);
    }

    fn plan_input<'a>() -> LegacyMigrationPlanInput<'a> {
        LegacyMigrationPlanInput {
            plan_id: "migration-0001",
            product_version: "2.0.0-alpha.2",
            source_commit: "0123456789abcdef0123456789abcdef01234567",
            resource_pack_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            created_at_unix: 1_800_000_000,
            provider_resolutions: &[],
        }
    }

    #[test]
    fn migration_plan_binds_inventory_identity_and_approvals() {
        let fixture = Fixture::new("plan");
        fixture.write_plugin(".agents/plugins/qiongli", "codex");
        fixture.write_marketplaces();
        let inventory = fixture.inventory();
        let plan = preview_legacy_migration(&inventory, plan_input()).unwrap();

        assert_eq!(plan.eligible_item_count, 3);
        assert_eq!(
            plan.required_approvals,
            vec![
                LegacyMigrationApproval::FilesystemWrite,
                LegacyMigrationApproval::ClientConfigChange,
                LegacyMigrationApproval::HostActivationConfirmed,
                LegacyMigrationApproval::LegacyCleanup,
            ]
        );
        assert_eq!(plan.plan_sha256.len(), 64);
        let canonical = plan.to_canonical_json().unwrap();
        assert_eq!(LegacyMigrationPlanV1::from_json(&canonical).unwrap(), plan);

        assert_eq!(
            approve_legacy_migration_plan(
                plan.clone(),
                &inventory,
                plan.created_at_unix,
                &[LegacyMigrationApproval::FilesystemWrite],
            )
            .unwrap_err(),
            LegacyMigrationContractError::ApprovalMissing
        );
        let approved = approve_legacy_migration_plan(
            plan.clone(),
            &inventory,
            plan.created_at_unix,
            &plan.required_approvals,
        )
        .unwrap();
        let receipt = initial_legacy_migration_receipt(&approved).unwrap();
        assert_eq!(receipt.state, LegacyMigrationState::PreviewReady);
        assert_eq!(receipt.eligible_item_count, 3);
        assert_eq!(receipt.receipt_sha256.len(), 64);
        let canonical = receipt.to_canonical_json().unwrap();
        assert_eq!(
            LegacyMigrationReceiptV1::from_json(&canonical).unwrap(),
            receipt
        );
    }

    #[test]
    fn changed_inventory_and_expired_plan_are_rejected() {
        let fixture = Fixture::new("stale-plan");
        fixture.write_plugin(".agents/plugins/qiongli", "codex");
        let inventory = fixture.inventory();
        let plan = preview_legacy_migration(&inventory, plan_input()).unwrap();

        fixture.write_skill(".codex/skills/qiongli-workflow");
        let changed = fixture.inventory();
        assert_eq!(
            approve_legacy_migration_plan(
                plan.clone(),
                &changed,
                plan.created_at_unix,
                &plan.required_approvals,
            )
            .unwrap_err(),
            LegacyMigrationContractError::InvalidInventory
        );
        assert_eq!(
            approve_legacy_migration_plan(
                plan.clone(),
                &inventory,
                plan.expires_at_unix + 1,
                &plan.required_approvals,
            )
            .unwrap_err(),
            LegacyMigrationContractError::InvalidTimestamp
        );
    }

    #[test]
    fn resumed_plan_accepts_current_entries_but_rejects_legacy_item_drift() {
        let fixture = Fixture::new("staged-shared-container");
        fixture.write_marketplaces();
        let inventory = fixture.inventory();
        let plan = preview_legacy_migration(&inventory, plan_input()).unwrap();
        let approved = approve_legacy_migration_plan(
            plan.clone(),
            &inventory,
            plan.created_at_unix,
            &[
                LegacyMigrationApproval::FilesystemWrite,
                LegacyMigrationApproval::ClientConfigChange,
            ],
        )
        .unwrap();
        let preview = initial_legacy_migration_receipt(&approved).unwrap();
        let staged = advance_legacy_migration_receipt(
            &preview,
            LegacyMigrationState::Staged,
            preview
                .items
                .iter()
                .map(|item| LegacyMigrationReceiptItemV1 {
                    item_id: item.item_id,
                    state: if item.state == LegacyMigrationReceiptItemState::Pending {
                        LegacyMigrationReceiptItemState::Staged
                    } else {
                        item.state
                    },
                    result_code: "legacy-migration-item-staged".to_owned(),
                })
                .collect(),
        )
        .unwrap();
        let awaiting = advance_legacy_migration_receipt(
            &staged,
            LegacyMigrationState::AwaitingClientActivation,
            staged
                .items
                .iter()
                .map(|item| LegacyMigrationReceiptItemV1 {
                    item_id: item.item_id,
                    state: if item.state == LegacyMigrationReceiptItemState::Staged {
                        LegacyMigrationReceiptItemState::AwaitingActivation
                    } else {
                        item.state
                    },
                    result_code: "legacy-migration-awaiting-host-activation".to_owned(),
                })
                .collect(),
        )
        .unwrap();

        fixture.add_current_marketplace_entries();
        let staged_inventory = fixture.inventory();
        let approvals = [
            LegacyMigrationApproval::FilesystemWrite,
            LegacyMigrationApproval::ClientConfigChange,
            LegacyMigrationApproval::HostActivationConfirmed,
        ];
        resume_legacy_migration_plan(plan.clone(), &staged_inventory, &awaiting, &approvals)
            .unwrap();

        let codex = fixture.home.join(".agents/plugins/marketplace.json");
        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&codex).unwrap()).unwrap();
        document["plugins"][0]["source"]["path"] = serde_json::json!("./plugins/custom");
        fs::write(codex, serde_json::to_vec(&document).unwrap()).unwrap();
        assert_eq!(
            resume_legacy_migration_plan(plan, &fixture.inventory(), &awaiting, &approvals)
                .unwrap_err(),
            LegacyMigrationContractError::InvalidInventory
        );
    }

    #[test]
    fn review_only_inventory_produces_review_receipt_without_write_approvals() {
        let fixture = Fixture::new("review-plan");
        let unproven = fixture.home.join(".agents/plugins/qiongli");
        fs::create_dir_all(&unproven).unwrap();
        fs::write(unproven.join("custom.txt"), b"user-owned").unwrap();
        let inventory = fixture.inventory();
        let plan = preview_legacy_migration(&inventory, plan_input()).unwrap();
        assert!(plan.required_approvals.is_empty());
        let approved = approve_legacy_migration_plan(plan, &inventory, 1_800_000_001, &[]).unwrap();
        let receipt = initial_legacy_migration_receipt(&approved).unwrap();
        assert_eq!(receipt.state, LegacyMigrationState::ReviewRequired);
        assert_eq!(receipt.unresolved_item_count, 1);
    }

    #[test]
    fn migration_store_persists_canonical_contracts_with_compare_and_swap() {
        let fixture = Fixture::new("store");
        fixture.write_plugin(".agents/plugins/qiongli", "codex");
        let inventory = fixture.inventory();
        let plan = preview_legacy_migration(&inventory, plan_input()).unwrap();
        let approved = approve_legacy_migration_plan(
            plan.clone(),
            &inventory,
            plan.created_at_unix,
            &[LegacyMigrationApproval::FilesystemWrite],
        )
        .unwrap();
        let receipt = initial_legacy_migration_receipt(&approved).unwrap();
        let store = LegacyMigrationStore::for_inventory(&inventory).unwrap();
        store.persist_preview(&plan, &receipt).unwrap();

        assert_eq!(store.load_plan(&plan.plan_id).unwrap(), plan);
        assert_eq!(store.load_receipt(&plan.plan_id).unwrap(), receipt);
        store
            .replace_receipt(&receipt.receipt_sha256, &receipt)
            .unwrap();
        assert_eq!(
            store
                .replace_receipt(
                    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    &receipt,
                )
                .unwrap_err(),
            LegacyMigrationPersistenceError::Conflict
        );
        assert_eq!(
            store.persist_preview(&plan, &receipt).unwrap_err(),
            LegacyMigrationPersistenceError::Conflict
        );
        assert!(!format!("{store:?}").contains(fixture.home.to_string_lossy().as_ref()));
    }

    #[test]
    fn migration_store_loads_the_latest_bounded_transaction() {
        let fixture = Fixture::new("latest-store");
        fixture.write_plugin(".agents/plugins/qiongli", "codex");
        let inventory = fixture.inventory();
        let store = LegacyMigrationStore::for_inventory(&inventory).unwrap();
        assert_eq!(store.load_latest().unwrap(), None);

        let first = preview_legacy_migration(&inventory, plan_input()).unwrap();
        let first_receipt = initial_legacy_migration_receipt_from_plan(&first).unwrap();
        store.persist_preview(&first, &first_receipt).unwrap();
        let second = preview_legacy_migration(
            &inventory,
            LegacyMigrationPlanInput {
                plan_id: "migration-0002",
                product_version: "2.0.0-alpha.2",
                source_commit: "0123456789abcdef0123456789abcdef01234567",
                resource_pack_sha256:
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                created_at_unix: 1_800_000_001,
                provider_resolutions: &[],
            },
        )
        .unwrap();
        let second_receipt = initial_legacy_migration_receipt_from_plan(&second).unwrap();
        store.persist_preview(&second, &second_receipt).unwrap();

        assert_eq!(
            store.load_latest().unwrap(),
            Some((second.clone(), second_receipt))
        );
        assert!(!store.cleanup_journal_present(&second.plan_id).unwrap());
    }

    #[test]
    fn receipt_cannot_complete_before_verified_cleanup() {
        let fixture = Fixture::new("receipt-transition");
        fixture.write_plugin(".agents/plugins/qiongli", "codex");
        let inventory = fixture.inventory();
        let plan = preview_legacy_migration(&inventory, plan_input()).unwrap();
        let approved = approve_legacy_migration_plan(
            plan.clone(),
            &inventory,
            plan.created_at_unix,
            &[LegacyMigrationApproval::FilesystemWrite],
        )
        .unwrap();
        let receipt = initial_legacy_migration_receipt(&approved).unwrap();
        assert_eq!(
            advance_legacy_migration_receipt(
                &receipt,
                LegacyMigrationState::Complete,
                receipt.items.clone(),
            )
            .unwrap_err(),
            LegacyMigrationContractError::DocumentInvalid
        );

        let staged = advance_legacy_migration_receipt(
            &receipt,
            LegacyMigrationState::Staged,
            receipt
                .items
                .iter()
                .map(|item| LegacyMigrationReceiptItemV1 {
                    item_id: item.item_id,
                    state: if item.state == LegacyMigrationReceiptItemState::Pending {
                        LegacyMigrationReceiptItemState::Staged
                    } else {
                        item.state
                    },
                    result_code: "legacy-migration-item-staged".to_owned(),
                })
                .collect(),
        )
        .unwrap();
        let verification = advance_legacy_migration_receipt(
            &staged,
            LegacyMigrationState::VerificationRequired,
            staged
                .items
                .iter()
                .map(|item| LegacyMigrationReceiptItemV1 {
                    item_id: item.item_id,
                    state: if item.state == LegacyMigrationReceiptItemState::Staged {
                        LegacyMigrationReceiptItemState::Verified
                    } else {
                        item.state
                    },
                    result_code: "legacy-migration-item-verified".to_owned(),
                })
                .collect(),
        )
        .unwrap();
        let cleanup_ready = advance_legacy_migration_receipt(
            &verification,
            LegacyMigrationState::CleanupReady,
            verification.items.clone(),
        )
        .unwrap();
        let complete = advance_legacy_migration_receipt(
            &cleanup_ready,
            LegacyMigrationState::Complete,
            cleanup_ready
                .items
                .iter()
                .map(|item| LegacyMigrationReceiptItemV1 {
                    item_id: item.item_id,
                    state: if item.state == LegacyMigrationReceiptItemState::Verified {
                        LegacyMigrationReceiptItemState::Cleaned
                    } else {
                        item.state
                    },
                    result_code: "legacy-migration-item-cleaned".to_owned(),
                })
                .collect(),
        )
        .unwrap();
        assert_eq!(complete.state, LegacyMigrationState::Complete);
        assert_eq!(complete.completed_item_count, 1);
        assert_eq!(complete.receipt_sha256.len(), 64);
    }

    fn full_cleanup_fixture(
        label: &str,
    ) -> (
        Fixture,
        LegacyMigrationInventory,
        PreparedLegacyMigrationCleanup,
    ) {
        let fixture = Fixture::new(label);
        fixture.write_plugin(".agents/plugins/qiongli", "codex");
        fixture.write_skill(".codex/skills/qiongli-workflow");
        fixture.write_plugin(
            ".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli",
            "claude",
        );
        fixture.write_skill(".claude/skills/qiongli-workflow");
        fixture.write_marketplaces();
        fixture.write_standalone_mcp();
        fixture.write_provider_config();
        let inventory = fixture.inventory();
        let plan = preview_legacy_migration(&inventory, plan_input()).unwrap();
        let approved = approve_legacy_migration_plan(
            plan.clone(),
            &inventory,
            plan.created_at_unix,
            &[
                LegacyMigrationApproval::FilesystemWrite,
                LegacyMigrationApproval::ClientConfigChange,
            ],
        )
        .unwrap();
        let receipt = initial_legacy_migration_receipt(&approved).unwrap();
        LegacyMigrationStore::for_inventory(&inventory)
            .unwrap()
            .persist_preview(&plan, &receipt)
            .unwrap();
        let approved = grant_legacy_migration_approval(
            approved,
            LegacyMigrationApproval::HostActivationConfirmed,
        )
        .unwrap();
        let approved = grant_legacy_migration_approval(
            approved,
            LegacyMigrationApproval::GlobalSettingsVerified,
        )
        .unwrap();
        let approved =
            grant_legacy_migration_approval(approved, LegacyMigrationApproval::LegacyCleanup)
                .unwrap();
        let cutover = VerifiedLegacyMigrationCutover {
            approved,
            verified_clients: vec![ClientKind::Codex, ClientKind::ClaudeCode],
        };
        fixture.add_current_marketplace_entries();
        let staged_inventory = fixture.inventory();
        let prepared = prepare_legacy_migration_cleanup(&cutover, &staged_inventory).unwrap();
        (fixture, staged_inventory, prepared)
    }

    #[test]
    fn cleanup_removes_only_recognized_legacy_surfaces_and_keeps_backups() {
        let (fixture, _inventory, prepared) = full_cleanup_fixture("cleanup");
        let commit = apply_legacy_migration_cleanup(&prepared).unwrap();
        assert_eq!(commit.cleaned_items.len(), 9);
        assert_eq!(commit.cleanup_sha256, prepared.preview().cleanup_sha256);
        assert!(!fixture.home.join(".agents/plugins/qiongli").exists());
        assert!(!fixture.home.join(".codex/skills/qiongli-workflow").exists());
        assert!(
            !fixture
                .home
                .join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli")
                .exists()
        );
        assert!(
            !fixture
                .home
                .join(".claude/skills/qiongli-workflow")
                .exists()
        );

        let codex_marketplace: serde_json::Value = serde_json::from_slice(
            &fs::read(fixture.home.join(".agents/plugins/marketplace.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(codex_marketplace["name"], "personal");
        assert_eq!(codex_marketplace["plugins"].as_array().unwrap().len(), 1);
        assert_eq!(codex_marketplace["plugins"][0]["name"], "qiongli-next");
        let codex_config = fs::read_to_string(fixture.home.join(".codex/config.toml")).unwrap();
        assert!(codex_config.contains("model = \"host-owned\""));
        assert!(!codex_config.contains("QIONGLI MANAGED MCP"));
        let claude_config: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture.home.join(".claude.json")).unwrap()).unwrap();
        assert_eq!(claude_config["theme"], "dark");
        assert!(claude_config["mcpServers"]["qiongli"].is_null());
        let claude_marketplace: serde_json::Value = serde_json::from_slice(
            &fs::read(fixture.home.join(
                ".qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json",
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(claude_marketplace["plugins"].as_array().unwrap().len(), 1);
        assert_eq!(claude_marketplace["plugins"][0]["name"], "qiongli-next");

        let backup = fixture
            .home
            .join(".qiongli/v2/migrations/1x-to-2x/migration-0001/cleanup-backup");
        assert!(backup.join("codex-plugin-source").is_dir());
        assert!(backup.join("codex-marketplace.json").is_file());
        assert!(backup.join("claude-standalone-skills").is_dir());
        assert!(backup.join("legacy-providers.json").is_file());
        assert!(!fixture.home.join(".config/qiongli/providers.json").exists());
    }

    #[test]
    fn cleanup_failure_compensates_already_moved_directories() {
        let (fixture, _inventory, prepared) = full_cleanup_fixture("cleanup-compensate");
        let marketplace = fixture.home.join(".agents/plugins/marketplace.json");
        let staging = marketplace
            .parent()
            .unwrap()
            .join(".marketplace.json.qiongli-legacy-cleanup-stage");
        fs::write(&staging, b"concurrent-owner").unwrap();

        assert_eq!(
            apply_legacy_migration_cleanup(&prepared).unwrap_err(),
            LegacyMigrationCleanupError::PersistenceFailed
        );
        assert!(fixture.home.join(".agents/plugins/qiongli").is_dir());
        assert!(fixture.home.join(".codex/skills/qiongli-workflow").is_dir());
        let codex_marketplace: serde_json::Value =
            serde_json::from_slice(&fs::read(marketplace).unwrap()).unwrap();
        assert_eq!(codex_marketplace["plugins"].as_array().unwrap().len(), 2);
        assert_eq!(fs::read(staging).unwrap(), b"concurrent-owner");
    }

    #[test]
    fn durable_cleanup_journal_restores_after_restart() {
        let (fixture, _inventory, prepared) = full_cleanup_fixture("cleanup-recover");
        apply_legacy_migration_cleanup(&prepared).unwrap();
        let after_cleanup = fixture.inventory();
        let recovery = recover_legacy_migration_cleanup(&after_cleanup, "migration-0001").unwrap();
        assert_eq!(recovery.restored_items.len(), 9);

        let restored = fixture.inventory();
        assert_eq!(restored.summary().eligible_item_count, 9);
        assert_eq!(restored.summary().review_item_count, 0);
        let transaction = fixture
            .home
            .join(".qiongli/v2/migrations/1x-to-2x/migration-0001");
        assert!(!transaction.join("cleanup-backup").exists());
        assert!(!transaction.join("cleanup-journal.json").exists());
        assert!(transaction.join("plan.json").is_file());
        assert!(transaction.join("receipt.json").is_file());
    }

    #[test]
    fn completed_receipt_finalizes_only_transaction_owned_backups() {
        let (fixture, inventory, prepared) = full_cleanup_fixture("cleanup-finalize");
        apply_legacy_migration_cleanup(&prepared).unwrap();
        let store = LegacyMigrationStore::for_inventory(&inventory).unwrap();
        let mut receipt = store.load_receipt("migration-0001").unwrap();
        for (state, from, to, code) in [
            (
                LegacyMigrationState::Staged,
                LegacyMigrationReceiptItemState::Pending,
                LegacyMigrationReceiptItemState::Staged,
                "legacy-migration-item-staged",
            ),
            (
                LegacyMigrationState::VerificationRequired,
                LegacyMigrationReceiptItemState::Staged,
                LegacyMigrationReceiptItemState::Verified,
                "legacy-migration-item-verified",
            ),
            (
                LegacyMigrationState::CleanupReady,
                LegacyMigrationReceiptItemState::Verified,
                LegacyMigrationReceiptItemState::Verified,
                "legacy-migration-item-verified",
            ),
            (
                LegacyMigrationState::Complete,
                LegacyMigrationReceiptItemState::Verified,
                LegacyMigrationReceiptItemState::Cleaned,
                "legacy-migration-item-cleaned",
            ),
        ] {
            let prior_sha256 = receipt.receipt_sha256.clone();
            let items = receipt
                .items
                .iter()
                .map(|item| LegacyMigrationReceiptItemV1 {
                    item_id: item.item_id,
                    state: if item.state == from { to } else { item.state },
                    result_code: code.to_owned(),
                })
                .collect();
            receipt = advance_legacy_migration_receipt(&receipt, state, items).unwrap();
            store.replace_receipt(&prior_sha256, &receipt).unwrap();
        }

        let after_cleanup = fixture.inventory();
        let finalized = finalize_legacy_migration_cleanup(&after_cleanup, &receipt).unwrap();
        assert_eq!(finalized.removed_compensation_items, 9);
        let transaction = fixture
            .home
            .join(".qiongli/v2/migrations/1x-to-2x/migration-0001");
        assert!(!transaction.join("cleanup-backup").exists());
        assert!(!transaction.join("cleanup-journal.json").exists());
        assert!(transaction.join("plan.json").is_file());
        assert!(transaction.join("receipt.json").is_file());
    }
}
