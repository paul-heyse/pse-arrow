// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Retention and identity of actual selected-property table consumers, without solver journeys.
#![allow(
    clippy::unwrap_used,
    reason = "focused retained selection identity controls"
)]
use crate::workspace::{CompilerWorkspace, WorkspaceLimits};
use pse_authoring::dsl::ExprKind;
use pse_ids::SemanticId;
use pse_modeling::{
    Bindings, DeclarationId, Limits, PhysicalScope,
    specialize::{SpecializedModel, Value, root_instance},
};
use std::{
    collections::BTreeSet,
    sync::{Arc, atomic::AtomicBool},
};

const SOURCES: &[&str] = &[
    include_str!("../../../packages/reference/physical/models/chemistry.pse"),
    include_str!("../../../packages/reference/physical/models/compatibility.pse"),
    include_str!("../../../packages/reference/domain/models/constants.pse"),
    include_str!("../../../packages/reference/domain/models/provenance.pse"),
    include_str!("../../../packages/reference/domain/models/properties.pse"),
    include_str!("../../../packages/reference/domain/models/interactions.pse"),
];

fn specialized(fixture: &str, root_name: &str, demand: &[&str]) -> Arc<SpecializedModel> {
    let source = format!("{}\n{fixture}", SOURCES.join("\n"));
    let declarations = crate::authored_transfer_tests::rows(&source);
    let root = crate::authored_transfer_tests::root(&declarations, "retention_fixture", root_name);
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    workspace
        .specialize_modeling(
            root,
            root_instance(root),
            Bindings {
                demand: demand.iter().map(|name| (*name).to_owned()).collect(),
                ..Bindings::default()
            },
            Limits::default(),
        )
        .unwrap()
}

fn call_id(model: &SpecializedModel, suffix: &str) -> SemanticId {
    let expression = model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path.ends_with(suffix))
        .and_then(|symbol| symbol.expression.as_ref())
        .unwrap();
    let ExprKind::NamedCall { name, .. } = &expression.kind else {
        panic!("selected consumer remains a finite call: {expression:?}");
    };
    SemanticId::parse_hex(
        pse_authoring::dsl::render_path(name)
            .strip_prefix("f_")
            .unwrap(),
    )
    .unwrap()
}
fn contexts(model: &SpecializedModel) -> BTreeSet<DeclarationId> {
    model
        .selection_closures
        .values()
        .map(|closure| {
            let Value::Entity { id, .. } = &closure.context else {
                panic!("entity context");
            };
            *id
        })
        .collect()
}
fn child_group(model: &SpecializedModel, suffix: &str) -> pse_ids::ContentHash {
    model
        .instances
        .values()
        .find(|instance| instance.path.ends_with(suffix))
        .unwrap()
        .group
}
fn changed_main_edge() -> String {
    FIXTURE.replace(
        "dependencies={interactions.symmetric_scalar[fit,family,b,c]}",
        "dependencies={interactions.symmetric_scalar[fit,family,a,c]}",
    )
}
fn prepared_body_ids(fixture: &str) -> BTreeSet<pse_ids::ContentHash> {
    let source = format!("{}\n{fixture}", SOURCES.join("\n"));
    let declarations = crate::authored_transfer_tests::rows(&source);
    let root = crate::authored_transfer_tests::root(&declarations, "retention_fixture", "Isolated");
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            root_instance(root),
            Bindings {
                demand: vec!["result".to_owned()],
                ..Bindings::default()
            },
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(!prepared.admitted.bodies.is_empty());
    prepared.admitted.bodies.keys().copied().collect()
}

fn numerical_literals(model: &SpecializedModel) -> Vec<u64> {
    let mut literals = Vec::new();
    for function in model.functions.values() {
        if let Some(body) = &function.body {
            body.walk(|expression| {
                if let ExprKind::Number(number) = &expression.kind {
                    literals.push(number.value.to_bits());
                }
            });
        }
    }
    literals.sort();
    literals
}

#[test]
fn selected_property_table_retains_only_consumed_owner_context_and_first_call_edges() {
    let original = specialized(FIXTURE, "Isolated", &["result"]);
    let changed = specialized(&changed_main_edge(), "Isolated", &["result"]);
    assert_eq!(
        contexts(&original).len(),
        1,
        "unrelated property package must not be retained"
    );
    assert_eq!(contexts(&original), contexts(&changed));
    let before = original.selection_closures.values().next().unwrap();
    let after = changed.selection_closures.values().next().unwrap();
    assert_eq!(before.roots, after.roots);
    assert_eq!(
        before.records, after.records,
        "all three records are roots in both cases"
    );
    assert_ne!(
        before.edges, after.edges,
        "only an unused scientific dependency edge changed"
    );
    let original_id = call_id(&original, ".result");
    let changed_id = call_id(&changed, ".result");
    assert_ne!(
        original_id, changed_id,
        "the first isolated finite call must frame its consumed edges"
    );
    assert_eq!(
        numerical_literals(&original),
        numerical_literals(&changed),
        "the numerical literals did not change when the scientific dependency edge changed"
    );
    assert_ne!(
        original.instances.values().next().unwrap().group,
        changed.instances.values().next().unwrap().group
    );
    assert_ne!(
        prepared_body_ids(FIXTURE),
        prepared_body_ids(&changed_main_edge()),
        "changed scientific edges must reach preparation identity with equal numerical coefficients"
    );
}

#[test]
fn consumed_convention_changes_identity_with_equal_records_edges_and_coefficients() {
    let changed = FIXTURE.replacen("conventions={}", "conventions={coefficient_convention}", 1);
    assert_ne!(changed, FIXTURE);
    let original = specialized(FIXTURE, "Isolated", &["result"]);
    let updated = specialized(&changed, "Isolated", &["result"]);
    assert_eq!(contexts(&original), contexts(&updated));
    let before = original.selection_closures.values().next().unwrap();
    let after = updated.selection_closures.values().next().unwrap();
    assert_eq!(before.roots, after.roots);
    assert_eq!(before.records, after.records);
    assert_eq!(before.edges, after.edges);
    assert_eq!(numerical_literals(&original), numerical_literals(&updated));
    assert_ne!(
        call_id(&original, ".result"),
        call_id(&updated, ".result"),
        "a consumed convention is part of the finite function's scientific meaning"
    );
    assert_ne!(
        original.instances.values().next().unwrap().group,
        updated.instances.values().next().unwrap().group
    );
    assert_ne!(
        prepared_body_ids(FIXTURE),
        prepared_body_ids(&changed),
        "convention-only changes must reach preparation identity"
    );
}

#[test]
fn unrelated_selection_call_order_does_not_change_either_finite_function_identity() {
    let forward = specialized(FIXTURE, "Calls", &["first", "second"]);
    let reverse = specialized(FIXTURE, "Calls", &["second", "first"]);
    assert_eq!(contexts(&forward).len(), 2);
    assert_eq!(contexts(&forward), contexts(&reverse));
    assert_eq!(call_id(&forward, ".first"), call_id(&reverse, ".first"));
    assert_eq!(call_id(&forward, ".second"), call_id(&reverse, ".second"));
}

#[test]
fn late_child_demand_frames_own_selection_without_an_unrelated_instance_edge() {
    let original = specialized(FIXTURE, "Children", &["first.result", "second.result"]);
    let unrelated = FIXTURE.replace(
        "dependencies={interactions.symmetric_scalar[other_fit,family,b,c]}",
        "dependencies={interactions.symmetric_scalar[other_fit,family,a,c]}",
    );
    assert_ne!(unrelated, FIXTURE);
    let changed = specialized(&unrelated, "Children", &["second.result", "first.result"]);
    assert_eq!(
        call_id(&original, ".first.result"),
        call_id(&changed, ".first.result")
    );
    assert_eq!(
        child_group(&original, ".first"),
        child_group(&changed, ".first")
    );
    assert_ne!(
        call_id(&original, ".second.result"),
        call_id(&changed, ".second.result")
    );
    assert_ne!(
        child_group(&original, ".second"),
        child_group(&changed, ".second")
    );
}

const FIXTURE: &str = r#"package retention_fixture {
use chemistry @"1.0.0"; use compatibility @"1.0.0"; use provenance @"1.0.0";
use properties @"1.0.0"; use interactions @"1.0.0";
@id("51000000000000000000000000000001") entity chemistry.species a {}
@id("51000000000000000000000000000002") entity chemistry.species b {}
@id("51000000000000000000000000000003") entity chemistry.species c {}
entity provenance.source evidence {title="Isolated selection identity controls; independently specified equal coefficients"}
entity provenance.source coefficient_convention {title="Synthetic coefficient convention for the isolated identity control"}
entity properties.parameterization fit {title="First scientific selection",source=evidence}
entity properties.parameterization other_fit {title="Unrelated scientific selection",source=evidence}
entity properties.property family {quantity=Scalar,shape=properties.IndexShape.pair,applies={compatibility.PhaseType.liquidPhase}}
dataset rows_ab:interactions.symmetric_scalar bind(parameterization=fit,family=family,source=evidence,dependencies={interactions.symmetric_scalar[fit,family,b,c]},conventions={}) provenance(evidence,provenance.Role.synthetic) {[a,b]=[0.25];}
dataset rows_ac:interactions.symmetric_scalar bind(parameterization=fit,family=family,source=evidence,dependencies={},conventions={}) provenance(evidence,provenance.Role.synthetic) {[a,c]=[0.25];}
dataset rows_bc:interactions.symmetric_scalar bind(parameterization=fit,family=family,source=evidence,dependencies={},conventions={}) provenance(evidence,provenance.Role.synthetic) {[b,c]=[0.25];}
dataset rows_ab_other:interactions.symmetric_scalar bind(parameterization=other_fit,family=family,source=evidence,dependencies={interactions.symmetric_scalar[other_fit,family,b,c]},conventions={}) provenance(evidence,provenance.Role.synthetic) {[a,b]=[0.25];}
dataset rows_ac_other:interactions.symmetric_scalar bind(parameterization=other_fit,family=family,source=evidence,dependencies={},conventions={}) provenance(evidence,provenance.Role.synthetic) {[a,c]=[0.25];}
dataset rows_bc_other:interactions.symmetric_scalar bind(parameterization=other_fit,family=family,source=evidence,dependencies={},conventions={}) provenance(evidence,provenance.Role.synthetic) {[b,c]=[0.25];}
entity properties.selection_context selected_context {roots={interactions.symmetric_scalar[fit,family,a,b],interactions.symmetric_scalar[fit,family,a,c],interactions.symmetric_scalar[fit,family,b,c]},subjects={a,b,c},models={family},rules={}}
entity properties.selection_context other_context {roots={interactions.symmetric_scalar[other_fit,family,a,b],interactions.symmetric_scalar[other_fit,family,a,c],interactions.symmetric_scalar[other_fit,family,b,c]},subjects={a,b,c},models={family},rules={}}
entity properties.property_package model {admitted=selected_context}
entity properties.property_package other_model {admitted=other_context}
set packages:Set<properties.property_package>={model,other_model};
set families:Set<properties.property>={family}; set members:Set<chemistry.species>={a,b,c};
dataset chosen:interactions.symmetric_selection complete_over(k in packages,family in families,i in members,j in members) provenance(evidence,provenance.Role.synthetic) {
[model,family,a,b]=[interactions.symmetric_scalar[fit,family,a,b],missing];[model,family,a,c]=[interactions.symmetric_scalar[fit,family,a,c],missing];[model,family,b,c]=[interactions.symmetric_scalar[fit,family,b,c],missing];
[other_model,family,a,b]=[interactions.symmetric_scalar[other_fit,family,a,b],missing];[other_model,family,a,c]=[interactions.symmetric_scalar[other_fit,family,a,c],missing];[other_model,family,b,c]=[interactions.symmetric_scalar[other_fit,family,b,c],missing];
}
test Isolated {
permission empirical families(interactions.symmetric_scalar) allow_unknown true allow_extrapolation false;
let result:Scalar=interactions.selected_off_diagonal(model,family,a,b);
}
test Calls {
permission empirical families(interactions.symmetric_scalar) allow_unknown true allow_extrapolation false;
let first:Scalar=interactions.selected_off_diagonal(model,family,a,b);
let second:Scalar=interactions.selected_off_diagonal(other_model,family,a,b);
}
def Reader(k:properties.property_package) {
permission empirical families(interactions.symmetric_scalar) allow_unknown true allow_extrapolation false;
let result:Scalar=interactions.selected_off_diagonal(k,family,a,b);
}
test Children {
child first:Reader=Reader(k=model); child second:Reader=Reader(k=other_model);
}
}"#;
