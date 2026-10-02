// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Evidence is projected through the registry and failures preserve typed authorization.
use super::results::{ModelingCheck, applicability_checks};
use pse_ids::SemanticId;
use pse_model::applicability::{Claim, Node, Permission, Region};
use pse_model::generated::enums::{
    ModelingApplicabilityOutcome as Outcome, ModelingPermissionTarget as Target,
    ModelingValidityLayer as Layer,
};
use pse_relations::columnar::RelationRow;
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn observed() -> pse_model::applicability::Observation {
    let plan = Node {
        claim: Claim {
            id: None,
            coverage: Some(id(2)),
            owner: id(3),
            owner_lineage: vec![id(3)],
            evidence: Some(id(4)),
            form: id(5),
            call: id(6),
            records: vec![id(7)],
            dependencies: vec![id(8)],
            layer: Layer::Data,
            basis: None,
            reason: Some("Published record lacks a scientific applicability region".into()),
        },
        region: Region::Unknown,
        dependencies: vec![],
        inputs: vec![("T".into(), 0, id(9))],
        permissions: vec![Permission {
            id: id(10),
            scope: id(11),
            target_kind: Target::Records,
            targets: vec![id(7)],
            allow_unknown: true,
            allow_extrapolation: false,
        }],
    };
    let mut observation = plan.assess(&[], &[412.5]).observations.remove(0);
    observation.instance = Some(id(12));
    observation
}
#[test]
fn applicability_check_registry_roundtrip_preserves_physical_inputs_and_claim_lineage() {
    let observation = observed();
    let checks = applicability_checks(id(13).into(), std::slice::from_ref(&observation));
    let check = &checks[0];
    assert_eq!(check.applicability_outcome, Some(Outcome::UnknownEvidence));
    assert_eq!(check.claim_id, None);
    assert_eq!(check.claim_owner, Some(id(3)));
    assert_eq!(check.claim_owner_lineage, observation.claim.owner_lineage);
    assert_eq!(check.call_id, Some(id(6)));
    assert_eq!(check.coverage_id, Some(id(2)));
    assert_eq!(check.evidence_id, Some(id(4)));
    assert_eq!(check.form_id, Some(id(5)));
    assert_eq!(check.selected_records, vec![id(7)]);
    assert_eq!(check.dependencies, vec![id(8)]);
    assert_eq!(check.permission_ids, vec![id(10)]);
    assert_eq!(check.observation_instance, Some(id(12)));
    assert_eq!(check.applicability_required, Some(true));
    assert_eq!(check.applicability_reason, observation.claim.reason);
    assert_eq!(check.applicability_permissions[0].scope, id(11));
    assert_eq!(check.applicability_permissions[0].targets, vec![id(7)]);
    assert_eq!(check.input_values[0].quantity_type, id(9));
    assert_eq!(check.input_values[0].name, "T");
    assert_eq!(check.input_values[0].value, 412.5);
    assert!(check.satisfied);
    assert_eq!(check.unknown_allowed, Some(true));
    assert_eq!(check.extrapolation_allowed, Some(false));
    let registry = pse_schema::registry().unwrap();
    let validation = pse_relations::validate::ValidationContext::local(registry).unwrap();
    let mut builder = ModelingCheck::builder(registry, 1, &validation).unwrap();
    ModelingCheck::push(&mut builder, check.clone()).unwrap();
    let batch = ModelingCheck::finish(builder).unwrap();
    assert_eq!(ModelingCheck::rows(&batch).unwrap(), checks);
}

#[test]
fn inherited_family_authorization_roundtrip_retains_concrete_owner_and_exact_named_target() {
    let mut observation = observed();
    observation.claim.owner_lineage = vec![id(3), id(23), id(24)];
    observation.permissions[0].target_kind = Target::Families;
    observation.permissions[0].targets = vec![id(23)];
    assert!(observation.permissions[0].covers(&observation.claim));
    let mut distinct_lineage = observation.clone();
    distinct_lineage.claim.owner_lineage = vec![id(3), id(23), id(25)];
    let checks = applicability_checks(id(13).into(), &[observation.clone(), distinct_lineage]);
    assert_ne!(checks[0].target_id, checks[1].target_id);
    assert_eq!(checks[0].claim_owner, Some(id(3)));
    assert_eq!(checks[0].claim_owner_lineage, vec![id(3), id(23), id(24)]);
    assert_eq!(checks[0].applicability_permissions[0].targets, vec![id(23)]);
    let registry = pse_schema::registry().unwrap();
    let validation = pse_relations::validate::ValidationContext::local(registry).unwrap();
    let mut builder = ModelingCheck::builder(registry, 2, &validation).unwrap();
    for check in &checks {
        ModelingCheck::push(&mut builder, check.clone()).unwrap();
    }
    assert_eq!(
        ModelingCheck::rows(&ModelingCheck::finish(builder).unwrap()).unwrap(),
        checks
    );
    let encoded = serde_json::to_vec(&observation).unwrap();
    let decoded: pse_model::applicability::Observation = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, observation);
}
#[test]
fn structured_refusal_serialization_preserves_instance_scope_and_independent_permissions() {
    let mut observation = observed();
    observation.permissions.clear();
    observation.unknown_allowed = false;
    observation.admitted = false;
    let mut diagnostic = pse_model::diagnostic::BoundaryDiagnostic::new(
        pse_model::diagnostic::BoundaryClass::TrialRejected,
        pse_diagnostics::DiagnosticStage::Applicability,
        vec![id(5), id(7)],
        pse_diagnostics::DiagnosticRule::MathApplicability,
    );
    diagnostic.applicability.push(observation.clone());
    let encoded = serde_json::to_vec(&diagnostic).unwrap();
    let decoded: pse_model::diagnostic::BoundaryDiagnostic =
        serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.applicability, vec![observation]);
    let permission = observed().permissions.remove(0);
    assert_eq!(permission.scope, id(11));
    assert_eq!(permission.targets, vec![id(7)]);
    assert!(permission.allow_unknown);
    assert!(!permission.allow_extrapolation);
}

#[test]
fn repeated_form_instances_inputs_and_informational_alternatives_have_distinct_transport_identity()
{
    let first = observed();
    let mut other_instance = first.clone();
    other_instance.instance = Some(id(21));
    let mut other_input = first.clone();
    other_input.inputs[0].value = 450.;
    let mut alternative = first.clone();
    alternative.required = false;
    alternative.permissions.clear();
    alternative.unknown_allowed = false;
    let checks = applicability_checks(
        id(22).into(),
        &[first, other_instance, other_input, alternative],
    );
    let ids = checks
        .iter()
        .map(|c| c.target_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 4);
    assert!(checks.iter().all(|c| c.call_id == Some(id(6))));
    assert_eq!(checks[3].applicability_required, Some(false));
    assert_eq!(
        checks[3].applicability_outcome,
        Some(Outcome::UnknownEvidence)
    );
    assert!(checks[3].applicability_permissions.is_empty());
    let registry = pse_schema::registry().unwrap();
    let validation = pse_relations::validate::ValidationContext::local(registry).unwrap();
    let mut builder = ModelingCheck::builder(registry, 4, &validation).unwrap();
    for check in &checks {
        ModelingCheck::push(&mut builder, check.clone()).unwrap();
    }
    let batch = ModelingCheck::finish(builder).unwrap();
    assert_eq!(ModelingCheck::rows(&batch).unwrap(), checks);
}
