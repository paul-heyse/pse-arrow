// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit semantic compatibility and encoding admission for durable declarations.
use crate::fingerprint::{SEMANTIC_VERSION, SemanticContract};
use pse_ids::ContentHash;

/// Durable admission failure. No branch silently opens a historical contract.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CompatibilityError {
    /// An older contract needs an explicit migration outside the runtime reader.
    #[error("durable contract requires migration: {0}")]
    MigrationRequired(String),
    /// The meaning agrees, but this reader cannot execute the recorded encoding.
    #[error("unsupported durable encoding: {0}")]
    UnsupportedEncoding(String),
    /// A valid recorded meaning differs from the independently expected meaning.
    #[error("incompatible durable contract: {0}")]
    Incompatible(String),
    /// The recorded witness is missing, contradictory, or malformed.
    #[error("malformed durable contract: {0}")]
    Malformed(String),
}
pse_diagnostics::impl_diagnostic! {
    CompatibilityError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::SchemaInvalidDeclaration) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
pse_columnar::impl_native_error!(CompatibilityError);

/// Current semantic format marker.
pub const FORMAT: &str = "pse.semantic-contract.v2";
/// Recorded complete semantic support graph.
pub const KEY_CONTRACT: &str = "pse.contract.semantic";
/// Recorded semantic interpretation, separate from any storage codec.
pub const KEY_FORMAT: &str = "pse.contract.semantic_format";
/// Native execution layout identity, independent of prose and semantic identity.
pub const KEY_ENCODING: &str = "pse.contract.execution_encoding";

/// Decode and check the recorded witness before comparing any expected digest.
/// # Errors
/// Unsupported historical interpretation or malformed canonical metadata.
pub fn decode(
    format: Option<&str>,
    json: Option<&str>,
    hash: ContentHash,
) -> Result<SemanticContract, CompatibilityError> {
    if format != Some(FORMAT) {
        return Err(CompatibilityError::MigrationRequired(
            format.unwrap_or("unversioned contract").into(),
        ));
    }
    let json =
        json.ok_or_else(|| CompatibilityError::Malformed("semantic witness absent".into()))?;
    let value: SemanticContract =
        serde_json::from_str(json).map_err(|e| CompatibilityError::Malformed(e.to_string()))?;
    if value.version != SEMANTIC_VERSION {
        return Err(CompatibilityError::MigrationRequired(format!(
            "semantic version {}",
            value.version
        )));
    }
    let canonical = pse_columnar::native_field::canonical_json(&value)
        .map_err(|e| CompatibilityError::Malformed(e.to_string()))?;
    let identity = value
        .identity()
        .map_err(|e| CompatibilityError::Malformed(e.to_string()))?;
    if canonical != json
        || identity != hash
        || value.roots.is_empty()
        || !value.roots.iter().all(|r| value.relations.contains_key(r))
    {
        return Err(CompatibilityError::Malformed(
            "canonical witness, roots or digest disagree".into(),
        ));
    }
    Ok(value)
}

/// Compare independently compiled meaning before checking the supported physical layout.
/// # Errors
/// Semantic mismatch or unsupported physical representation.
pub fn require(
    actual: &SemanticContract,
    expected: &SemanticContract,
    encoding: ContentHash,
    supported: ContentHash,
) -> Result<(), CompatibilityError> {
    if actual != expected {
        return Err(CompatibilityError::Incompatible(
            "semantic support closure differs".into(),
        ));
    }
    if encoding != supported {
        return Err(CompatibilityError::UnsupportedEncoding(
            encoding.to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recorded_witness_and_independent_expectation_both_have_to_agree() {
        let registry = crate::registry().unwrap();
        let relation = registry.relation("runtime.solve_runs").unwrap();
        let contract = SemanticContract::new(registry, &[relation.id].into()).unwrap();
        let hash = contract.identity().unwrap();
        let json = pse_columnar::native_field::canonical_json(&contract).unwrap();
        let encoding = crate::fingerprint::encoding_relation(registry, relation).unwrap();
        let actual = decode(Some(FORMAT), Some(&json), hash).unwrap();
        require(&actual, &contract, encoding, encoding).unwrap();
        assert!(matches!(
            decode(None, None, hash),
            Err(CompatibilityError::MigrationRequired(_))
        ));
        assert!(matches!(
            decode(Some(FORMAT), Some(&json), ContentHash::NIL),
            Err(CompatibilityError::Malformed(_))
        ));
        assert!(matches!(
            require(&actual, &contract, ContentHash::NIL, encoding),
            Err(CompatibilityError::UnsupportedEncoding(_))
        ));
        let mut foreign = actual.clone();
        foreign.relations.get_mut(&relation.id).unwrap()["primary_key"] =
            serde_json::json!(["run_id"]);
        assert!(matches!(
            require(&foreign, &contract, encoding, encoding),
            Err(CompatibilityError::Incompatible(_))
        ));
        let foreign_json = pse_columnar::native_field::canonical_json(&foreign).unwrap();
        // A self-consistent foreign witness still cannot impersonate the compiled expectation.
        let foreign = decode(
            Some(FORMAT),
            Some(&foreign_json),
            foreign.identity().unwrap(),
        )
        .unwrap();
        assert!(require(&foreign, &contract, encoding, encoding).is_err());
    }
}
