use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{FULL_MCP_HOST_PROTOCOL_VERSION, HostFamilyV1, HostReviewResultV1, ToolId};

pub const HOST_ACCEPTANCE_SCHEMA_VERSION: u32 = 1;
pub const HOST_ACCEPTANCE_RECORD_TYPE: &str = "qiongli-alpha2-host-acceptance";

const MAX_FIXTURE_BYTES: usize = 64 * 1024;
const MAX_RECEIPT_BYTES: usize = 256 * 1024;
const MAX_FACTS: usize = 32;
const MAX_TOOL_IDS: usize = 32;
const MAX_TRANSITIONS: usize = 128;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const LEGACY_REQUIRED_TRANSITIONS: [HostAcceptanceTransitionV1; 4] = [
    HostAcceptanceTransitionV1::HandoffIssued,
    HostAcceptanceTransitionV1::CandidateAccepted,
    HostAcceptanceTransitionV1::ReviewAccepted,
    HostAcceptanceTransitionV1::CheckpointPersisted,
];
const OBSERVABLE_REQUIRED_TRANSITIONS: [HostAcceptanceTransitionV1; 3] = [
    HostAcceptanceTransitionV1::CandidateAccepted,
    HostAcceptanceTransitionV1::ReviewAccepted,
    HostAcceptanceTransitionV1::CheckpointPersisted,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAcceptanceError {
    InputTooLarge,
    InvalidJson,
    NonCanonicalJson,
    InvalidFixture,
    InvalidReceipt,
    FixtureMismatch,
    SerializationFailed,
}

impl HostAcceptanceError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InputTooLarge => "host-acceptance-input-too-large",
            Self::InvalidJson => "host-acceptance-json-invalid",
            Self::NonCanonicalJson => "host-acceptance-json-noncanonical",
            Self::InvalidFixture => "host-acceptance-fixture-invalid",
            Self::InvalidReceipt => "host-acceptance-receipt-invalid",
            Self::FixtureMismatch => "host-acceptance-fixture-mismatch",
            Self::SerializationFailed => "host-acceptance-serialization-failed",
        }
    }
}

impl Display for HostAcceptanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl Error for HostAcceptanceError {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostAcceptanceTransitionV1 {
    HandoffIssued,
    CandidateAccepted,
    ReviewAccepted,
    CheckpointPersisted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostAcceptanceProfileScopeV1 {
    IsolatedAcceptance,
    SystemExisting,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostAcceptanceFactV1 {
    pub fact_id: String,
    pub statement: String,
    pub source_anchor: String,
    pub fact_sha256: String,
    pub source_anchor_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostAcceptanceCandidateContractV1 {
    pub minimum_evidence_audit_count: u16,
    pub minimum_known_fact_count: u16,
    pub unresolved_gap_report_required: bool,
    pub review_result_required: bool,
    pub exact_natural_language_assertion: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostAcceptanceRejectionContractV1 {
    pub minimum_stale_project_revision_rejection_count: u16,
    pub minimum_checkpoint_digest_rejection_count: u16,
    pub minimum_undeclared_evidence_rejection_count: u16,
    pub minimum_unknown_field_rejection_count: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostAcceptanceFixtureV1 {
    pub schema_version: u32,
    pub fixture_id: String,
    pub expected_project_revision: u64,
    pub facts: Vec<HostAcceptanceFactV1>,
    pub required_tool_ids: Vec<ToolId>,
    pub required_transitions: Vec<HostAcceptanceTransitionV1>,
    pub candidate_contract: HostAcceptanceCandidateContractV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_contract: Option<HostAcceptanceRejectionContractV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_host_profile_scope: Option<HostAcceptanceProfileScopeV1>,
}

impl HostAcceptanceFixtureV1 {
    pub fn from_canonical_json(input: &[u8]) -> Result<Self, HostAcceptanceError> {
        if input.len() > MAX_FIXTURE_BYTES {
            return Err(HostAcceptanceError::InputTooLarge);
        }
        let fixture =
            serde_json::from_slice::<Self>(input).map_err(|_| HostAcceptanceError::InvalidJson)?;
        fixture.validate()?;
        if fixture.to_canonical_json()? != input {
            return Err(HostAcceptanceError::NonCanonicalJson);
        }
        Ok(fixture)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, HostAcceptanceError> {
        self.validate()?;
        canonical_json(self, MAX_FIXTURE_BYTES)
    }

    pub fn digest(&self) -> Result<String, HostAcceptanceError> {
        Ok(sha256(&self.to_canonical_json()?))
    }

    pub fn fact_set_digest(&self) -> Result<String, HostAcceptanceError> {
        self.validate()?;
        let fact_digests = self
            .facts
            .iter()
            .map(|fact| fact.fact_sha256.as_str())
            .collect::<Vec<_>>();
        Ok(sha256(&canonical_json(&fact_digests, MAX_FIXTURE_BYTES)?))
    }

    fn validate(&self) -> Result<(), HostAcceptanceError> {
        let facts = self.facts.iter().collect::<BTreeSet<_>>();
        let tools = self.required_tool_ids.iter().collect::<BTreeSet<_>>();
        let valid_transition_contract = match &self.rejection_contract {
            None => {
                self.required_transitions == LEGACY_REQUIRED_TRANSITIONS
                    && self.required_host_profile_scope.is_none()
            }
            Some(contract) => {
                self.required_transitions == OBSERVABLE_REQUIRED_TRANSITIONS
                    && self.required_host_profile_scope
                        == Some(HostAcceptanceProfileScopeV1::SystemExisting)
                    && contract.minimum_stale_project_revision_rejection_count > 0
                    && contract.minimum_checkpoint_digest_rejection_count > 0
                    && contract.minimum_undeclared_evidence_rejection_count > 0
                    && contract.minimum_unknown_field_rejection_count > 0
                    && contract.minimum_stale_project_revision_rejection_count <= MAX_FACTS as u16
                    && contract.minimum_checkpoint_digest_rejection_count <= MAX_FACTS as u16
                    && contract.minimum_undeclared_evidence_rejection_count <= MAX_FACTS as u16
                    && contract.minimum_unknown_field_rejection_count <= MAX_FACTS as u16
            }
        };
        if self.schema_version != HOST_ACCEPTANCE_SCHEMA_VERSION
            || !valid_token(&self.fixture_id)
            || self.expected_project_revision == 0
            || self.expected_project_revision > MAX_SAFE_INTEGER
            || self.facts.is_empty()
            || self.facts.len() > MAX_FACTS
            || facts.len() != self.facts.len()
            || !strictly_sorted(&self.facts)
            || self.facts.iter().any(|fact| {
                !valid_token(&fact.fact_id)
                    || !valid_fixture_text(&fact.statement)
                    || !valid_source_anchor(&fact.source_anchor)
                    || fact.fact_sha256 != sha256(fact.statement.as_bytes())
                    || fact.source_anchor_sha256 != sha256(fact.source_anchor.as_bytes())
            })
            || self.required_tool_ids.is_empty()
            || self.required_tool_ids.len() > MAX_TOOL_IDS
            || tools.len() != self.required_tool_ids.len()
            || !strictly_sorted(&self.required_tool_ids)
            || !self
                .required_tool_ids
                .iter()
                .any(|tool| tool.as_str() == "qiongli_project_read")
            || !valid_transition_contract
            || self.candidate_contract.minimum_evidence_audit_count == 0
            || usize::from(self.candidate_contract.minimum_evidence_audit_count) > MAX_FACTS
            || self.candidate_contract.minimum_known_fact_count == 0
            || usize::from(self.candidate_contract.minimum_known_fact_count) > self.facts.len()
            || !self.candidate_contract.unresolved_gap_report_required
            || !self.candidate_contract.review_result_required
            || self.candidate_contract.exact_natural_language_assertion
        {
            return Err(HostAcceptanceError::InvalidFixture);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostAcceptanceStatusV1 {
    Accepted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostAcceptanceCheckpointTransitionV1 {
    pub transition: HostAcceptanceTransitionV1,
    pub from_generation: u64,
    pub to_generation: u64,
    pub from_document_sha256: String,
    pub to_document_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostAcceptanceVerdictV1 {
    pub project_read_observed: bool,
    pub evidence_grounded_candidate: bool,
    pub unresolved_gap_report_observed: bool,
    pub review_result_observed: bool,
    pub checkpoint_persisted: bool,
    pub provider_credential_count: u32,
    pub direct_model_request_count: u32,
    pub qiongli_model_cli_child_count: u32,
    pub private_payload_persisted_count: u32,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub stale_project_revision_rejection_count: u16,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub checkpoint_digest_rejection_count: u16,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub undeclared_evidence_rejection_count: u16,
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub unknown_field_rejection_count: u16,
    #[serde(default, skip_serializing_if = "is_false")]
    pub rejection_state_unchanged: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostAcceptanceReceiptV1 {
    pub schema_version: u32,
    pub record_type: String,
    pub status: HostAcceptanceStatusV1,
    pub publication_allowed: bool,
    pub fixture_id: String,
    pub fixture_sha256: String,
    pub product_version: String,
    pub product_source_commit: String,
    pub binary_sha256: String,
    pub host_family: HostFamilyV1,
    pub host_version: String,
    pub adapter_version: String,
    pub plugin_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_profile_scope: Option<HostAcceptanceProfileScopeV1>,
    pub full_mcp_protocol: String,
    pub observed_tool_ids: Vec<ToolId>,
    pub evidence_audit_count: u16,
    pub evidence_audit_sha256: String,
    pub known_fact_count: u16,
    pub known_fact_set_sha256: String,
    pub accepted_candidate_sha256: String,
    pub review_result: HostReviewResultV1,
    pub checkpoint_transitions: Vec<HostAcceptanceCheckpointTransitionV1>,
    pub verdict: HostAcceptanceVerdictV1,
}

impl HostAcceptanceReceiptV1 {
    pub fn from_canonical_json(input: &[u8]) -> Result<Self, HostAcceptanceError> {
        if input.len() > MAX_RECEIPT_BYTES {
            return Err(HostAcceptanceError::InputTooLarge);
        }
        let receipt =
            serde_json::from_slice::<Self>(input).map_err(|_| HostAcceptanceError::InvalidJson)?;
        receipt.validate()?;
        if receipt.to_canonical_json()? != input {
            return Err(HostAcceptanceError::NonCanonicalJson);
        }
        Ok(receipt)
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, HostAcceptanceError> {
        self.validate()?;
        canonical_json(self, MAX_RECEIPT_BYTES)
    }

    pub fn digest(&self) -> Result<String, HostAcceptanceError> {
        Ok(sha256(&self.to_canonical_json()?))
    }

    pub fn validate_against(
        &self,
        fixture: &HostAcceptanceFixtureV1,
    ) -> Result<(), HostAcceptanceError> {
        fixture.validate()?;
        self.validate()?;
        let rejection_mismatch = fixture.rejection_contract.as_ref().is_some_and(|contract| {
            !self.verdict.rejection_state_unchanged
                || self.verdict.stale_project_revision_rejection_count
                    < contract.minimum_stale_project_revision_rejection_count
                || self.verdict.checkpoint_digest_rejection_count
                    < contract.minimum_checkpoint_digest_rejection_count
                || self.verdict.undeclared_evidence_rejection_count
                    < contract.minimum_undeclared_evidence_rejection_count
                || self.verdict.unknown_field_rejection_count
                    < contract.minimum_unknown_field_rejection_count
        });
        if self.fixture_id != fixture.fixture_id
            || self.fixture_sha256 != fixture.digest()?
            || self.host_profile_scope != fixture.required_host_profile_scope
            || self.evidence_audit_count < fixture.candidate_contract.minimum_evidence_audit_count
            || usize::from(self.known_fact_count) != fixture.facts.len()
            || self.known_fact_set_sha256 != fixture.fact_set_digest()?
            || self.observed_tool_ids != fixture.required_tool_ids
            || self.checkpoint_transitions.len() != fixture.required_transitions.len()
            || self
                .checkpoint_transitions
                .iter()
                .zip(&fixture.required_transitions)
                .any(|(observed, required)| observed.transition != *required)
            || (fixture.candidate_contract.review_result_required
                && self.review_result != HostReviewResultV1::Pass)
            || (fixture.candidate_contract.unresolved_gap_report_required
                && !self.verdict.unresolved_gap_report_observed)
            || rejection_mismatch
        {
            return Err(HostAcceptanceError::FixtureMismatch);
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), HostAcceptanceError> {
        let tools = self.observed_tool_ids.iter().collect::<BTreeSet<_>>();
        if self.schema_version != HOST_ACCEPTANCE_SCHEMA_VERSION
            || self.record_type != HOST_ACCEPTANCE_RECORD_TYPE
            || self.status != HostAcceptanceStatusV1::Accepted
            || self.publication_allowed
            || !valid_token(&self.fixture_id)
            || !valid_sha256(&self.fixture_sha256)
            || !valid_token(&self.product_version)
            || !valid_source_commit(&self.product_source_commit)
            || !valid_sha256(&self.binary_sha256)
            || !matches!(
                self.host_family,
                HostFamilyV1::Codex | HostFamilyV1::ClaudeCode
            )
            || !valid_token(&self.host_version)
            || !valid_token(&self.adapter_version)
            || !valid_sha256(&self.plugin_sha256)
            || self.full_mcp_protocol != FULL_MCP_HOST_PROTOCOL_VERSION
            || self.observed_tool_ids.is_empty()
            || self.observed_tool_ids.len() > MAX_TOOL_IDS
            || tools.len() != self.observed_tool_ids.len()
            || !strictly_sorted(&self.observed_tool_ids)
            || !self
                .observed_tool_ids
                .iter()
                .any(|tool| tool.as_str() == "qiongli_project_read")
            || self.evidence_audit_count == 0
            || !valid_sha256(&self.evidence_audit_sha256)
            || self.known_fact_count == 0
            || !valid_sha256(&self.known_fact_set_sha256)
            || !valid_sha256(&self.accepted_candidate_sha256)
            || self.review_result != HostReviewResultV1::Pass
            || self.checkpoint_transitions.is_empty()
            || self.checkpoint_transitions.len() > MAX_TRANSITIONS
            || !valid_transition_chain(&self.checkpoint_transitions)
            || !self.verdict.project_read_observed
            || !self.verdict.evidence_grounded_candidate
            || !self.verdict.unresolved_gap_report_observed
            || !self.verdict.review_result_observed
            || !self.verdict.checkpoint_persisted
            || self.verdict.provider_credential_count != 0
            || self.verdict.direct_model_request_count != 0
            || self.verdict.qiongli_model_cli_child_count != 0
            || self.verdict.private_payload_persisted_count != 0
            || self.verdict.stale_project_revision_rejection_count > MAX_FACTS as u16
            || self.verdict.checkpoint_digest_rejection_count > MAX_FACTS as u16
            || self.verdict.undeclared_evidence_rejection_count > MAX_FACTS as u16
            || self.verdict.unknown_field_rejection_count > MAX_FACTS as u16
        {
            return Err(HostAcceptanceError::InvalidReceipt);
        }
        Ok(())
    }
}

fn valid_transition_chain(transitions: &[HostAcceptanceCheckpointTransitionV1]) -> bool {
    transitions.iter().enumerate().all(|(index, transition)| {
        transition.from_generation < transition.to_generation
            && transition.to_generation <= MAX_SAFE_INTEGER
            && valid_sha256(&transition.from_document_sha256)
            && valid_sha256(&transition.to_document_sha256)
            && transition.from_document_sha256 != transition.to_document_sha256
            && (index == 0
                || (transitions[index - 1].to_generation == transition.from_generation
                    && transitions[index - 1].to_document_sha256
                        == transition.from_document_sha256))
    })
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+'))
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && valid_lower_hex(value)
}

fn valid_fixture_text(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 1_024
        && value.chars().all(|character| !character.is_control())
}

fn valid_source_anchor(value: &str) -> bool {
    valid_fixture_text(value)
        && (value.starts_with("RESEARCH/") || value.starts_with("graph/"))
        && value.contains('#')
        && !value.contains('\\')
        && !value
            .split('/')
            .any(|component| matches!(component, "" | "." | ".."))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && valid_lower_hex(value)
}

fn valid_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

const fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

const fn is_false(value: &bool) -> bool {
    !*value
}

fn canonical_json<T: Serialize>(
    value: &T,
    maximum_bytes: usize,
) -> Result<Vec<u8>, HostAcceptanceError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| HostAcceptanceError::SerializationFailed)?;
    if bytes.len() > maximum_bytes {
        return Err(HostAcceptanceError::InputTooLarge);
    }
    Ok(bytes)
}

fn sha256(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_json(bytes: &[u8]) -> &[u8] {
        bytes
            .strip_suffix(b"\r\n")
            .or_else(|| bytes.strip_suffix(b"\n"))
            .unwrap_or(bytes)
    }

    fn fixture() -> HostAcceptanceFixtureV1 {
        HostAcceptanceFixtureV1 {
            schema_version: HOST_ACCEPTANCE_SCHEMA_VERSION,
            fixture_id: "alpha2-host-driven-v1".to_owned(),
            expected_project_revision: 1,
            facts: vec![
                HostAcceptanceFactV1 {
                    fact_id: "evidence-extraction".to_owned(),
                    statement: "In the synthetic acceptance corpus, structured evidence extraction reduced unresolved citation gaps from four to one.".to_owned(),
                    source_anchor: "RESEARCH/alpha2-host-acceptance/sources.md#evidence-extraction"
                        .to_owned(),
                    fact_sha256:
                        "61dea63e5d868358e14cf0334ea444cd173ab3932b9b0c02f4395e4efbb66b47"
                            .to_owned(),
                    source_anchor_sha256:
                        "e2fad59fd7a8fe0a0bb05d8b6269441876182453745cb79300c022d5508e717a"
                            .to_owned(),
                },
                HostAcceptanceFactV1 {
                    fact_id: "replication-note".to_owned(),
                    statement: "The synthetic replication note reports that the reduction persisted across two independent review passes.".to_owned(),
                    source_anchor: "RESEARCH/alpha2-host-acceptance/sources.md#replication-note"
                        .to_owned(),
                    fact_sha256:
                        "936f7135f6b01c84bf0dfc571212cba087a0a72654fe18389dad34b66f2d2e49"
                            .to_owned(),
                    source_anchor_sha256:
                        "e9c2dfdf928b4bff539e5c272d6b17bebf3b5cb11669e27bd285f9ab64225230"
                            .to_owned(),
                },
            ],
            required_tool_ids: vec![ToolId::parse("qiongli_project_read").unwrap()],
            required_transitions: LEGACY_REQUIRED_TRANSITIONS.to_vec(),
            candidate_contract: HostAcceptanceCandidateContractV1 {
                minimum_evidence_audit_count: 1,
                minimum_known_fact_count: 1,
                unresolved_gap_report_required: true,
                review_result_required: true,
                exact_natural_language_assertion: false,
            },
            rejection_contract: None,
            required_host_profile_scope: None,
        }
    }

    fn transition(
        transition: HostAcceptanceTransitionV1,
        from_generation: u64,
        from_digest: char,
        to_digest: char,
    ) -> HostAcceptanceCheckpointTransitionV1 {
        HostAcceptanceCheckpointTransitionV1 {
            transition,
            from_generation,
            to_generation: from_generation + 1,
            from_document_sha256: from_digest.to_string().repeat(64),
            to_document_sha256: to_digest.to_string().repeat(64),
        }
    }

    fn receipt(fixture: &HostAcceptanceFixtureV1) -> HostAcceptanceReceiptV1 {
        HostAcceptanceReceiptV1 {
            schema_version: HOST_ACCEPTANCE_SCHEMA_VERSION,
            record_type: HOST_ACCEPTANCE_RECORD_TYPE.to_owned(),
            status: HostAcceptanceStatusV1::Accepted,
            publication_allowed: false,
            fixture_id: fixture.fixture_id.clone(),
            fixture_sha256: fixture.digest().unwrap(),
            product_version: "2.0.0-alpha.2".to_owned(),
            product_source_commit: "5".repeat(40),
            binary_sha256: "6".repeat(64),
            host_family: HostFamilyV1::Codex,
            host_version: "0.144.6".to_owned(),
            adapter_version: "2.0.0-alpha.2".to_owned(),
            plugin_sha256: "7".repeat(64),
            host_profile_scope: fixture.required_host_profile_scope,
            full_mcp_protocol: FULL_MCP_HOST_PROTOCOL_VERSION.to_owned(),
            observed_tool_ids: vec![ToolId::parse("qiongli_project_read").unwrap()],
            evidence_audit_count: 1,
            evidence_audit_sha256: "8".repeat(64),
            known_fact_count: 2,
            known_fact_set_sha256: fixture.fact_set_digest().unwrap(),
            accepted_candidate_sha256: "a".repeat(64),
            review_result: HostReviewResultV1::Pass,
            checkpoint_transitions: vec![
                transition(HostAcceptanceTransitionV1::HandoffIssued, 1, 'b', 'c'),
                transition(HostAcceptanceTransitionV1::CandidateAccepted, 2, 'c', 'd'),
                transition(HostAcceptanceTransitionV1::ReviewAccepted, 3, 'd', 'e'),
                transition(HostAcceptanceTransitionV1::CheckpointPersisted, 4, 'e', 'f'),
            ],
            verdict: HostAcceptanceVerdictV1 {
                project_read_observed: true,
                evidence_grounded_candidate: true,
                unresolved_gap_report_observed: true,
                review_result_observed: true,
                checkpoint_persisted: true,
                provider_credential_count: 0,
                direct_model_request_count: 0,
                qiongli_model_cli_child_count: 0,
                private_payload_persisted_count: 0,
                stale_project_revision_rejection_count: 0,
                checkpoint_digest_rejection_count: 0,
                undeclared_evidence_rejection_count: 0,
                unknown_field_rejection_count: 0,
                rejection_state_unchanged: false,
            },
        }
    }

    #[test]
    fn fixture_and_receipt_round_trip_as_canonical_json() {
        let fixture_file = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../tooling/release/acceptance/fixtures/alpha2-host-driven-v1.json"
        ));
        let fixture_bytes = fixture_json(fixture_file);
        let fixture = HostAcceptanceFixtureV1::from_canonical_json(fixture_bytes).unwrap();
        assert_eq!(fixture, self::fixture());
        let fixture_bytes = fixture.to_canonical_json().unwrap();
        assert_eq!(
            HostAcceptanceFixtureV1::from_canonical_json(&fixture_bytes).unwrap(),
            fixture
        );

        let receipt = receipt(&fixture);
        receipt.validate_against(&fixture).unwrap();
        let receipt_bytes = receipt.to_canonical_json().unwrap();
        assert_eq!(
            HostAcceptanceReceiptV1::from_canonical_json(&receipt_bytes).unwrap(),
            receipt
        );
    }

    #[test]
    fn r5c_c5_fixture_is_canonical_and_revision_bound() {
        let fixture_file = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../tooling/release/acceptance/fixtures/r5c-c5-host-driven-v1.json"
        ));
        let fixture_bytes = fixture_json(fixture_file);
        let fixture = HostAcceptanceFixtureV1::from_canonical_json(fixture_bytes).unwrap();
        assert_eq!(fixture.fixture_id, "r5c-c5-host-driven-v1");
        assert_eq!(fixture.expected_project_revision, 2);
        assert_eq!(fixture.facts.len(), 2);
        assert!(fixture.rejection_contract.is_some());
        assert_eq!(
            fixture.required_host_profile_scope,
            Some(HostAcceptanceProfileScopeV1::SystemExisting)
        );
        assert_eq!(
            fixture.required_transitions,
            OBSERVABLE_REQUIRED_TRANSITIONS
        );
        assert!(fixture.facts.iter().all(|fact| {
            fact.source_anchor
                .starts_with("graph/semantic_links.jsonl#")
        }));

        let mut accepted_receipt = receipt(&fixture);
        accepted_receipt.observed_tool_ids = vec![
            ToolId::parse("qiongli_project_graph_snapshot").unwrap(),
            ToolId::parse("qiongli_project_read").unwrap(),
        ];
        accepted_receipt.evidence_audit_count = 2;
        accepted_receipt.checkpoint_transitions = vec![
            transition(HostAcceptanceTransitionV1::CandidateAccepted, 2, 'b', 'c'),
            transition(HostAcceptanceTransitionV1::ReviewAccepted, 3, 'c', 'd'),
            HostAcceptanceCheckpointTransitionV1 {
                transition: HostAcceptanceTransitionV1::CheckpointPersisted,
                from_generation: 4,
                to_generation: 6,
                from_document_sha256: "d".repeat(64),
                to_document_sha256: "e".repeat(64),
            },
        ];
        accepted_receipt
            .verdict
            .stale_project_revision_rejection_count = 1;
        accepted_receipt.verdict.checkpoint_digest_rejection_count = 1;
        accepted_receipt.verdict.undeclared_evidence_rejection_count = 1;
        accepted_receipt.verdict.unknown_field_rejection_count = 1;
        accepted_receipt.verdict.rejection_state_unchanged = true;
        accepted_receipt.validate_against(&fixture).unwrap();

        accepted_receipt.host_profile_scope =
            Some(HostAcceptanceProfileScopeV1::IsolatedAcceptance);
        assert_eq!(
            accepted_receipt.validate_against(&fixture),
            Err(HostAcceptanceError::FixtureMismatch)
        );
        accepted_receipt.host_profile_scope = fixture.required_host_profile_scope;

        accepted_receipt.verdict.unknown_field_rejection_count = 0;
        assert_eq!(
            accepted_receipt.validate_against(&fixture),
            Err(HostAcceptanceError::FixtureMismatch)
        );

        let mut extra_tool_receipt = receipt(&fixture);
        extra_tool_receipt.observed_tool_ids = vec![
            ToolId::parse("qiongli_project_graph_snapshot").unwrap(),
            ToolId::parse("qiongli_project_list").unwrap(),
            ToolId::parse("qiongli_project_read").unwrap(),
        ];
        extra_tool_receipt.evidence_audit_count = 2;
        extra_tool_receipt.checkpoint_transitions = vec![
            transition(HostAcceptanceTransitionV1::CandidateAccepted, 2, 'b', 'c'),
            transition(HostAcceptanceTransitionV1::ReviewAccepted, 3, 'c', 'd'),
            HostAcceptanceCheckpointTransitionV1 {
                transition: HostAcceptanceTransitionV1::CheckpointPersisted,
                from_generation: 4,
                to_generation: 6,
                from_document_sha256: "d".repeat(64),
                to_document_sha256: "e".repeat(64),
            },
        ];
        extra_tool_receipt
            .verdict
            .stale_project_revision_rejection_count = 1;
        extra_tool_receipt.verdict.checkpoint_digest_rejection_count = 1;
        extra_tool_receipt
            .verdict
            .undeclared_evidence_rejection_count = 1;
        extra_tool_receipt.verdict.unknown_field_rejection_count = 1;
        extra_tool_receipt.verdict.rejection_state_unchanged = true;
        assert_eq!(
            extra_tool_receipt.validate_against(&fixture),
            Err(HostAcceptanceError::FixtureMismatch)
        );

        let mut extra_transition_receipt = extra_tool_receipt;
        extra_transition_receipt.observed_tool_ids = fixture.required_tool_ids.clone();
        extra_transition_receipt
            .checkpoint_transitions
            .push(transition(
                HostAcceptanceTransitionV1::CheckpointPersisted,
                6,
                'e',
                'f',
            ));
        assert_eq!(
            extra_transition_receipt.validate_against(&fixture),
            Err(HostAcceptanceError::FixtureMismatch)
        );
    }

    #[test]
    fn receipt_contains_only_redacted_identifiers_counts_hashes_and_verdicts() {
        let fixture = fixture();
        let serialized = String::from_utf8(receipt(&fixture).to_canonical_json().unwrap()).unwrap();
        for forbidden_key in [
            "providerCredential",
            "providerApiKey",
            "prompt",
            "candidateBody",
            "modelResponse",
            "conversationId",
            "projectId",
            "projectPath",
            "registrationPath",
            "systemRegistration",
            "toolResult",
        ] {
            assert!(!serialized.contains(&format!("\"{forbidden_key}\":")));
        }
    }

    #[test]
    fn receipt_rejects_success_claims_without_observed_tools() {
        let fixture = fixture();
        let mut claimed_success = receipt(&fixture);
        claimed_success.validate_against(&fixture).unwrap();
        // A successful Host exit or tool-shaped response text is not a tool observation.
        claimed_success.observed_tool_ids.clear();
        assert_eq!(
            claimed_success.validate_against(&fixture),
            Err(HostAcceptanceError::InvalidReceipt)
        );
        let bytes = serde_json_canonicalizer::to_vec(&claimed_success).unwrap();
        assert_eq!(
            HostAcceptanceReceiptV1::from_canonical_json(&bytes),
            Err(HostAcceptanceError::InvalidReceipt)
        );
    }

    #[test]
    fn receipt_rejects_direct_execution_and_unknown_fields() {
        let fixture = fixture();
        let mut invalid_receipt = receipt(&fixture);
        invalid_receipt.verdict.direct_model_request_count = 1;
        assert_eq!(
            invalid_receipt.to_canonical_json(),
            Err(HostAcceptanceError::InvalidReceipt)
        );

        let mut value = serde_json::to_value(receipt(&fixture)).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("prompt".to_owned(), serde_json::json!("private"));
        let bytes = serde_json_canonicalizer::to_vec(&value).unwrap();
        assert_eq!(
            HostAcceptanceReceiptV1::from_canonical_json(&bytes),
            Err(HostAcceptanceError::InvalidJson)
        );
    }
}
