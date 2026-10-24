// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored composition evidence is independent of conserved zeros and mass data.
use crate::{CheckedPackage, PhysicalScope, TypeContext, check, kernel_types, specialize::Value};

fn admit(body: &str) -> Result<CheckedPackage, crate::ModelingError> {
    let source = format!(
        "{}\n{body}",
        crate::scientific_inputs::family_sources("composition")
    );
    let rows = crate::scientific_inputs::parse_sources(
        &source,
        pse_authoring::language::IdentityPolicy::Named,
    );
    let (registry, _) = kernel_types::physical();
    let checks =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    check(
        &rows,
        &TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &registry,
            preconditions: &checks,
            scope: &PhysicalScope::default(),
        },
    )
}
fn assert_requirement(error: &crate::ModelingError, expected: &str, record: &str, body: &str) {
    let mut cause = error;
    while let crate::ModelingError::Located { cause: inner, .. } = cause {
        cause = inner;
    }
    let crate::ModelingError::Contract {
        declaration,
        reason: crate::ModelingContractReason::AuthoredRequirement { requirement },
        ..
    } = cause
    else {
        panic!("expected authored requirement: {error:?}");
    };
    let expected_requirement = pse_ids::SemanticId::parse_hex(expected).unwrap();
    assert_eq!(*requirement, expected_requirement);
    let rows = crate::scientific_inputs::parse_sources(
        body,
        pse_authoring::language::IdentityPolicy::Named,
    );
    let expected_record = rows
        .iter()
        .find(|row| row.name == record)
        .unwrap()
        .declaration_id
        .as_id();
    assert_eq!(*declaration, expected_record);
    let diagnostic = error.boundary_diagnostic();
    assert_eq!(
        diagnostic.code,
        pse_diagnostics::DiagnosticCode::ValidationInvariant
    );
    assert_eq!(
        diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingContract
    );
    assert!(diagnostic.sources.contains(&expected_record));
    assert!(diagnostic.sources.contains(&expected_requirement));
}
const PARTICIPANTS: &str = r#"package p {
 use chemistry @"1.0.0"; use provenance @"1.0.0";
 entity provenance.source source {title="complete composition and as-authored extent controls"}
 entity chemistry.element c {}
 entity chemistry.element h {}
 entity chemistry.species a {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}
 entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}
 dataset counts:chemistry.formula provenance(source,provenance.Role.published) {[a,c]=[1{1}];[b,c]=[1{1}];}
 entity chemistry.extent_convention conversion_extent {normalization=chemistry.ExtentNormalization.AsAuthored}
 entity chemistry.reaction conversion {extent=conversion_extent}
 dataset coefficients:chemistry.stoichiometry provenance(source,provenance.Role.published) {[conversion,a]=[-1{1}];[conversion,b]=[1{1}];}
}"#;

#[test]
fn scientific_composition_unknown_and_complete_empty_are_distinct() {
    let package = admit(
        r#"package p {
 use chemistry @"1.0.0";
 entity chemistry.element e {}
 entity chemistry.species unknown {}
 entity chemistry.species empty {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}
 }"#,
    )
    .unwrap();
    let unknown = package.record(package.names["p.unknown"]).unwrap();
    let empty = package.record(package.names["p.empty"]).unwrap();
    assert_eq!(unknown.values["molar_mass"], Value::Missing);
    assert_eq!(unknown.values["charge"], Value::Missing);
    assert_ne!(unknown.values["composition"], empty.values["composition"]);
    let Value::Number { bits, .. } = empty.values["molar_mass"] else {
        panic!("complete empty elemental mass")
    };
    assert_eq!(f64::from_bits(bits), 0.0);
}
#[test]
fn scientific_composition_sparse_formulas_conserve_without_atomic_weights() {
    let package = admit(PARTICIPANTS).unwrap();
    for name in ["p.a", "p.b"] {
        assert_eq!(
            package.record(package.names[name]).unwrap().values["molar_mass"],
            Value::Missing
        );
    }
    // No hydrogen rows are stored: complete elemental vectors supply conserved zeros.
    let error = admit(&PARTICIPANTS.replace("[b,c]=[1{1}]", "[b,c]=[2{1}]")).unwrap_err();
    assert_requirement(
        &error,
        "01a0f00baca572638a494b0dad768f4e",
        "conversion",
        PARTICIPANTS,
    );
}
#[test]
fn scientific_composition_unknown_participants_and_unknown_charge_refuse_independently() {
    admit(PARTICIPANTS).unwrap();
    let unknown = PARTICIPANTS.replace(
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}",
        "entity chemistry.species b {charge=0{1}}",
    ).replace("[b,c]=[1{1}];", "");
    let error = admit(&unknown).unwrap_err();
    assert_requirement(
        &error,
        "9066ffbd8c2e5021985af0f6c35e1b2b",
        "conversion",
        PARTICIPANTS,
    );
    let error = admit(&PARTICIPANTS.replace(
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}",
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete}",
    )).unwrap_err();
    assert_requirement(
        &error,
        "b28738e2629058a78763c9ea9596624b",
        "conversion",
        PARTICIPANTS,
    );
    let error = admit(&PARTICIPANTS.replace(
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}",
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=1{1}}",
    )).unwrap_err();
    assert_requirement(
        &error,
        "01a0f00baca572638a494b0eec9e5b89",
        "conversion",
        PARTICIPANTS,
    );
}
#[test]
fn scientific_composition_apparent_and_lumped_mappings_conserve_elements_and_charge() {
    for (kind, relation, conservation, completeness) in [
        (
            "apparent",
            "dissociation",
            "0204550929b6584382a39bc4a00239a4",
            "01a0f00baca572638a494b0bc8fbc911",
        ),
        (
            "lumped",
            "lumping",
            "68c39c7af73252a5b29a8e967d6e151e",
            "e97f0933fcbc520b8c8af8d89a5ae9ee",
        ),
    ] {
        let body = format!(
            r#"package p {{
 use chemistry @"1.0.0"; use provenance @"1.0.0";
 entity provenance.source source {{title="independent charge and element mapping control"}}
 entity chemistry.element c {{}}
 entity chemistry.species true_species {{composition=chemistry.CompositionKnowledge.Complete,charge=0{{1}}}}
 entity chemistry.{kind} apparent_species {{composition=chemistry.CompositionKnowledge.Complete,charge=0{{1}}}}
 dataset counts:chemistry.formula provenance(source,provenance.Role.published) {{[true_species,c]=[2{{1}}];[apparent_species,c]=[1{{1}}];}}
 dataset map:chemistry.{relation} provenance(source,provenance.Role.published) {{[apparent_species,true_species]=[1{{1}}];}}
}}"#
        );
        let error = admit(&body).unwrap_err();
        assert_requirement(&error, conservation, "apparent_species", &body);
        admit(&body.replace("[true_species,c]=[2{1}]", "[true_species,c]=[1{1}]")).unwrap();
        let error = admit(&body.replace(
            "entity chemistry.species true_species {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}",
            "entity chemistry.species true_species {charge=0{1}}",
        )).unwrap_err();
        assert_requirement(&error, completeness, "apparent_species", &body);
    }
}

#[test]
fn scientific_composition_extent_identity_belongs_to_one_authored_reaction() {
    let text=PARTICIPANTS.replace("entity chemistry.reaction conversion {extent=conversion_extent}","entity chemistry.reaction conversion {extent=conversion_extent}\nentity chemistry.reaction other {extent=conversion_extent}");
    let error = admit(&text).unwrap_err();
    assert!(error.to_string().contains("unique"), "{error}");
}
