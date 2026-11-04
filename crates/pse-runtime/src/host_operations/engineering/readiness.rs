// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive local routing facts, shared by original publishers and independent readers.
use std::path::PathBuf;

/// Observations published by an original managed engineering frontend.
/// Decoding these facts neither authenticates a peer nor creates a native launch owner.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringFrontendReady {
    /// Local readiness document interpretation.
    pub version: pse_model::document::Version<1>,
    /// Checkout whose native manager must authenticate the original payload.
    pub root: PathBuf,
    /// Original managed invocation, independently checked on every connection.
    pub invocation: pse_host::envelope::CliInvocationId,
    /// Socket spelling; kernel peer and executable checks still govern its use.
    pub socket: PathBuf,
    /// Recorded payload process; a number alone grants no process control.
    pub process: u32,
    /// Recorded native image content identity.
    pub build: pse_ids::ContentHash,
    /// Observed image bytes and topology, independently reobserved by the receiver.
    pub artifact: pse_buildinfo::FileObservation,
}

/// Observations published by the retained canonical fixture keeper.
/// Only the live original native service/worker owners can settle or dispose the fixture.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringFixtureReady {
    /// Local readiness document interpretation.
    pub version: pse_model::document::Version<1>,
    /// Existing fixture configuration, never an instruction for a client to start a database.
    pub state: PathBuf,
    /// Existing database selected by the original keeper.
    pub database: String,
    /// Borrowed selection data; native descriptor admission remains with its original owner.
    pub fixture: pse_operations::native_fixture::FixtureSelection,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "passive readiness fixtures use fixed original documents"
    )]
    use super::*;

    #[test]
    fn frontend_readiness_keeps_original_json_fields_and_strict_interpretation() {
        let id = pse_host::envelope::CliInvocationId::new();
        let original = serde_json::json!({
            "version":1,"root":"/original/checkout","invocation":id,"socket":"/original/socket",
            "process":42,"build":pse_ids::ContentHash::from_bytes([17;32]),
            "artifact":{"path":"/original/image","sha256":"11".repeat(32),
                "canonical":"/original/image","mode":320}
        });
        let facts: EngineeringFrontendReady = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(serde_json::to_value(facts).unwrap(), original);
        let mut wrong = original.clone();
        wrong["version"] = serde_json::json!(2);
        assert!(serde_json::from_value::<EngineeringFrontendReady>(wrong).is_err());
        let mut unknown = original.clone();
        unknown["authenticated"] = serde_json::json!(true);
        assert!(serde_json::from_value::<EngineeringFrontendReady>(unknown).is_err());
        let mut missing = original;
        missing.as_object_mut().unwrap().remove("artifact");
        assert!(serde_json::from_value::<EngineeringFrontendReady>(missing).is_err());
    }

    #[test]
    fn fixture_readiness_decoding_does_not_admit_a_fabricated_borrowed_selection() {
        let directory = tempfile::tempdir().unwrap();
        let state = directory.path().join("not-created");
        let original = serde_json::json!({"version":1,"state":state,"database":"original",
            "fixture":{"version":1,"state":state,"device":1,"inode":2,"nonce":"recorded-only"}});
        let bytes = serde_json::to_vec(&original).unwrap();
        let facts: EngineeringFixtureReady = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_value(&facts).unwrap(), original);
        assert!(
            pse_operations::native_fixture::FixtureSelection::read(&facts.fixture.descriptor())
                .is_err()
        );
        assert!(
            !state.exists(),
            "decoding creates neither state nor a native service"
        );
        assert_eq!(
            bytes,
            serde_json::to_vec(&original).unwrap(),
            "original byte identity stays separate from decoded observations"
        );
    }
}
