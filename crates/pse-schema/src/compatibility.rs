// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Distinct recorded, consumer, mutation and transformation admission products.
mod recorded;
use crate::fingerprint::{SEMANTIC_VERSION, SemanticContract};
use pse_ids::{ContentHash, SemanticId};
use std::collections::BTreeMap;

/// A durable declaration cannot establish the requested capability.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CompatibilityError {
    /// Portable meaning or a declared transformation is required.
    #[error("durable contract requires migration: {0}")]
    MigrationRequired(String),
    /// The reader cannot execute this physical representation.
    #[error("unsupported durable encoding: {0}")]
    UnsupportedEncoding(String),
    /// A consumer or writer does not admit this independently verified meaning.
    #[error("incompatible durable contract: {0}")]
    Incompatible(String),
    /// Recorded meaning, support or physical observations contradict one another.
    #[error("malformed durable contract: {0}")]
    Malformed(String),
}
pse_diagnostics::impl_diagnostic! {
    CompatibilityError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::SchemaInvalidDeclaration) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
pse_columnar::impl_native_error!(CompatibilityError);
/// Supported portable semantic interpretation.
pub const FORMAT: &str = "pse.semantic-contract.v2";
/// Complete portable semantic support graph.
pub const KEY_CONTRACT: &str = "pse.contract.semantic";
/// Portable semantic interpretation marker.
pub const KEY_FORMAT: &str = "pse.contract.semantic_format";
/// Native layout identity; executable expression codecs have no semantic authority.
pub const KEY_ENCODING: &str = "pse.contract.execution_encoding";

/// Canonical, independently verified recorded meaning. No current registry is consulted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedRecordedContract {
    contract: SemanticContract,
    identity: ContentHash,
}
impl VerifiedRecordedContract {
    /// Verify a complete declaration supplied by an authoritative producer.
    /// # Errors
    /// Malformed roots, declarations or support closure.
    pub fn from_contract(contract: SemanticContract) -> Result<Self, CompatibilityError> {
        if contract.version != SEMANTIC_VERSION {
            return Err(CompatibilityError::MigrationRequired(format!(
                "semantic version {}",
                contract.version
            )));
        }
        recorded::validate(&contract)?;
        let identity = contract.identity().map_err(malformed)?;
        Ok(Self { contract, identity })
    }
    /// Verified portable support graph, retaining historical enum members.
    pub fn contract(&self) -> &SemanticContract {
        &self.contract
    }
    /// Original canonical identity; reading never substitutes today's registry identity.
    pub fn identity(&self) -> ContentHash {
        self.identity
    }
    /// Verify observed physical fields against this meaning and its recorded encoding.
    /// # Errors
    /// Contradictory fields, hidden enum domains, extensions or layout identity.
    pub fn verify_fields(
        &self,
        relation: SemanticId,
        schema: &arrow_schema::Schema,
        encoding: ContentHash,
    ) -> Result<(), CompatibilityError> {
        recorded::observed_fields(&self.contract, relation, schema)?;
        if crate::fingerprint::encoding_fields(schema.fields().iter().map(AsRef::as_ref))
            .map_err(malformed)?
            != encoding
        {
            return Err(CompatibilityError::Malformed(
                "recorded execution encoding contradicts observed fields".into(),
            ));
        }
        Ok(())
    }
    /// Select a consumed root closure from verified recorded declarations.
    /// # Errors
    /// A requested root or its support is absent.
    pub fn select_roots(
        &self,
        roots: &std::collections::BTreeSet<SemanticId>,
    ) -> Result<Self, CompatibilityError> {
        Self::from_contract(recorded::select(&self.contract, roots)?)
    }
    /// Admit only the meaning the consumer explicitly requests.
    /// # Errors
    /// Unknown consumed fields/domains or changed constraints/reference meaning.
    pub fn project(
        &self,
        consumer: &SemanticContract,
    ) -> Result<ConsumerProjection, CompatibilityError> {
        recorded::validate(consumer)?;
        let fields = recorded::project(&self.contract, consumer)?;
        Ok(ConsumerProjection {
            source: self.identity,
            consumer: consumer.identity().map_err(malformed)?,
            fields,
        })
    }
    /// Authorize exact writes separately from directional reads.
    /// # Errors
    /// Any declaration or encoding difference.
    pub fn admit_exact_write(
        &self,
        expected: &SemanticContract,
        observed_encoding: ContentHash,
        supported_encoding: ContentHash,
    ) -> Result<ExactWriteAdmission, CompatibilityError> {
        if &self.contract != expected {
            return Err(CompatibilityError::Incompatible(
                "exact write support closure differs".into(),
            ));
        }
        if observed_encoding != supported_encoding {
            return Err(CompatibilityError::UnsupportedEncoding(
                observed_encoding.to_string(),
            ));
        }
        Ok(ExactWriteAdmission {
            semantic: self.identity,
            encoding: observed_encoding,
        })
    }
}
/// A transient source-slot projection; it cannot authorize mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsumerProjection {
    source: ContentHash,
    consumer: ContentHash,
    fields: BTreeMap<SemanticId, Vec<Option<usize>>>,
}
impl ConsumerProjection {
    /// Verified source identity.
    pub fn source_identity(&self) -> ContentHash {
        self.source
    }
    /// Consumed declaration identity.
    pub fn consumer_identity(&self) -> ContentHash {
        self.consumer
    }
    /// Source field slots in target order. None is only a declared historical descriptor rule.
    pub fn field_slots(&self, relation: SemanticId) -> Option<&[Option<usize>]> {
        self.fields.get(&relation).map(Vec::as_slice)
    }
}
/// Exact declaration/layout admission. The catalog additionally binds selected table state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactWriteAdmission {
    semantic: ContentHash,
    encoding: ContentHash,
}
impl ExactWriteAdmission {
    /// Exact expected portable identity.
    pub fn semantic_identity(&self) -> ContentHash {
        self.semantic
    }
    /// Exact supported layout identity.
    pub fn encoding_identity(&self) -> ContentHash {
        self.encoding
    }
}
/// Exact source/target pair for an explicitly declared transformation.
/// This pair does not imply that a transformation's values or effects have been validated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationAdmission {
    source: VerifiedRecordedContract,
    target: VerifiedRecordedContract,
}
impl MigrationAdmission {
    /// Bind independently verified declarations; execution still validates ordered operations.
    pub fn new(source: VerifiedRecordedContract, target: VerifiedRecordedContract) -> Self {
        Self { source, target }
    }
    /// Exact recorded source.
    pub fn source(&self) -> &VerifiedRecordedContract {
        &self.source
    }
    /// Exact target declaration.
    pub fn target(&self) -> &VerifiedRecordedContract {
        &self.target
    }
    /// Original source identity.
    pub fn source_identity(&self) -> ContentHash {
        self.source.identity()
    }
    /// Expected target identity.
    pub fn target_identity(&self) -> ContentHash {
        self.target.identity()
    }
}
/// Decode canonical recorded metadata before consulting any receiving consumer.
/// # Errors
/// Unsupported interpretation, malformed closure or mismatched identity.
pub fn verify_recorded(
    format: Option<&str>,
    json: Option<&str>,
    hash: ContentHash,
) -> Result<VerifiedRecordedContract, CompatibilityError> {
    if format != Some(FORMAT) {
        return Err(CompatibilityError::MigrationRequired(
            format.unwrap_or("unversioned contract").into(),
        ));
    }
    let json =
        json.ok_or_else(|| CompatibilityError::Malformed("semantic witness absent".into()))?;
    let value: SemanticContract = serde_json::from_str(json).map_err(malformed)?;
    if pse_columnar::native_field::canonical_json(&value).map_err(malformed)? != json {
        return Err(CompatibilityError::Malformed("noncanonical witness".into()));
    }
    let verified = VerifiedRecordedContract::from_contract(value)?;
    if verified.identity != hash {
        return Err(CompatibilityError::Malformed(
            "semantic digest disagrees".into(),
        ));
    }
    Ok(verified)
}
fn malformed(error: impl std::fmt::Display) -> CompatibilityError {
    CompatibilityError::Malformed(error.to_string())
}
#[cfg(test)]
mod tests;
