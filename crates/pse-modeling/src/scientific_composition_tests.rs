// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored composition evidence is independent of conserved zeros and mass data.
use crate::{CheckedPackage, PhysicalScope, TypeContext, check, kernel_types, specialize::Value};

fn admit(body: &str) -> Result<CheckedPackage, String> {
    let sources = [
        include_str!("../../../packages/reference/physical/models/compatibility.pse"),
        include_str!("../../../packages/reference/physical/models/chemistry.pse"),
        include_str!("../../../packages/reference/domain/models/provenance.pse"),
        body,
    ];
    let rows = kernel_types::try_source(&sources.join("\n")).map_err(|error| error.to_string())?;
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
    .map_err(|error| error.to_string())
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
    assert!(
        error.contains("a reaction conserves every element"),
        "{error}"
    );
}
#[test]
fn scientific_composition_unknown_participants_and_unknown_charge_refuse_independently() {
    let unknown = PARTICIPANTS.replace(
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}",
        "entity chemistry.species b {charge=0{1}}",
    ).replace("[b,c]=[1{1}];", "");
    let error = admit(&unknown).unwrap_err();
    assert!(error.contains("IncompleteComposition"), "{error}");
    let error = admit(&PARTICIPANTS.replace(
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}",
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete}",
    )).unwrap_err();
    assert!(error.contains("IncompleteCharge"), "{error}");
    let error = admit(&PARTICIPANTS.replace(
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}",
        "entity chemistry.species b {composition=chemistry.CompositionKnowledge.Complete,charge=1{1}}",
    )).unwrap_err();
    assert!(error.contains("a reaction conserves charge"), "{error}");
}
#[test]
fn scientific_composition_apparent_and_lumped_mappings_conserve_elements_and_charge() {
    for (kind, relation, label) in [
        (
            "apparent",
            "dissociation",
            "an apparent mapping conserves every element",
        ),
        (
            "lumped",
            "lumping",
            "a lumped mapping conserves every element",
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
        assert!(error.contains(label), "{error}");
        admit(&body.replace("[true_species,c]=[2{1}]", "[true_species,c]=[1{1}]")).unwrap();
        let error = admit(&body.replace(
            "entity chemistry.species true_species {composition=chemistry.CompositionKnowledge.Complete,charge=0{1}}",
            "entity chemistry.species true_species {charge=0{1}}",
        )).unwrap_err();
        assert!(error.contains("IncompleteComposition"), "{error}");
    }
}

#[test]
fn scientific_composition_extent_identity_belongs_to_one_authored_reaction() {
    let text=PARTICIPANTS.replace("entity chemistry.reaction conversion {extent=conversion_extent}","entity chemistry.reaction conversion {extent=conversion_extent}\nentity chemistry.reaction other {extent=conversion_extent}");
    let error = admit(&text).unwrap_err();
    assert!(error.contains("unique"), "{error}");
}
