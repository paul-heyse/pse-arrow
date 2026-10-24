// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Contract observations retain source-owned meaning through context, location and transport.
use crate::{
    ModelingContractReason as Reason, ModelingError, data, invalid, invalid_reason, kernel_types,
};
use pse_diagnostics::{DiagnosticCode, DiagnosticRule};
use pse_ids::{ContentHash, SemanticId};
use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation};

#[test]
fn contract_reason_survives_context_location_and_diagnostic_codec() {
    let rows = kernel_types::source("package p { def Root { param gain:Scalar=2; } }");
    let gain = rows.iter().find(|row| row.name == "gain").unwrap();
    let revision = ContentHash::from_bytes([9; 32]);
    for detail in ["old human wording", "entirely revised human explanation"] {
        let error = data::context(
            invalid_reason(
                gain.declaration_id,
                Reason::TemporalScalarDefaultMismatch,
                detail,
            ),
            "parameter gain",
        )
        .located(&rows);
        let diagnostic = error.boundary_diagnostic().with_revision(revision);
        assert_eq!(diagnostic.code, DiagnosticCode::ValidationInvariant);
        assert_eq!(diagnostic.rule, DiagnosticRule::ModelingContract);
        assert_eq!(diagnostic.class, BoundaryClass::InvalidModel);
        assert_eq!(diagnostic.sources, [gain.declaration_id.as_id()]);
        assert!(matches!(diagnostic.observations.get("reason"),
            Some(Observation::Text(reason)) if reason == Reason::TemporalScalarDefaultMismatch.as_str()));
        assert_eq!(diagnostic.locations.len(), 1);
        let location = &diagnostic.locations[0];
        assert_eq!(location.revision, Some(revision));
        assert_eq!(
            location.start,
            Some(u32::try_from(gain.source_start).unwrap())
        );
        assert_eq!(location.end, Some(u32::try_from(gain.source_end).unwrap()));
        let encoded = serde_json::to_vec(&diagnostic).unwrap();
        let decoded: BoundaryDiagnostic = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            serde_json::to_value(diagnostic).unwrap()
        );
        assert!(error.to_string().contains(detail));
    }
}

#[test]
fn authored_requirement_retains_record_and_requirement_without_fabricating_a_location() {
    let rows = kernel_types::source("package p { entity kind item {} entity item sample {} }");
    let record = rows.iter().find(|row| row.name == "sample").unwrap();
    let requirement = SemanticId::from_bytes([5; 16]);
    let reason = Reason::AuthoredRequirement { requirement };
    let error = invalid_reason(record.declaration_id, reason, "authored detail").located(&rows);
    let diagnostic = error.boundary_diagnostic();
    assert!(diagnostic.sources.contains(&record.declaration_id.as_id()));
    assert!(diagnostic.sources.contains(&requirement));
    assert_eq!(diagnostic.locations.len(), 1);
    assert_eq!(
        diagnostic.locations[0].source,
        record.declaration_id.as_id()
    );
    assert!(matches!(error, ModelingError::Located { cause, .. }
        if matches!(*cause, ModelingError::Contract { reason: actual, .. } if actual == reason)));
}

#[test]
fn untouched_contracts_are_explicitly_unclassified() {
    let error = invalid(SemanticId::NIL, "ambiguous name is only human detail here");
    assert!(matches!(
        error,
        ModelingError::Contract {
            reason: Reason::Unclassified,
            ..
        }
    ));
    assert!(
        matches!(error.boundary_diagnostic().observations.get("reason"),
        Some(Observation::Text(reason)) if reason == "unclassified")
    );
}
