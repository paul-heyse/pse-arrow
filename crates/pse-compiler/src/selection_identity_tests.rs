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
        crate::authored_transfer_tests::inputs(),
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
    SemanticId::parse_hex(name.strip_prefix("f_").unwrap()).unwrap()
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
    FIXTURE.replace("dependencies={bc}", "dependencies={ac}")
}
fn prepared_body_ids(fixture: &str) -> BTreeSet<pse_ids::ContentHash> {
    let source = format!("{}\n{fixture}", SOURCES.join("\n"));
    let declarations = crate::authored_transfer_tests::rows(&source);
    let root = crate::authored_transfer_tests::root(&declarations, "retention_fixture", "Isolated");
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::inputs(),
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
    let find_body = |model: &SpecializedModel, id: SemanticId| {
        model
            .functions
            .values()
            .find(|function| function.id.as_id() == id)
            .unwrap()
            .body
            .clone()
    };
    assert_eq!(
        find_body(&original, original_id),
        find_body(&changed, changed_id),
        "the admitted coefficients and numerical body did not change"
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
    let unrelated = FIXTURE.replace("dependencies={bc_other}", "dependencies={ac_other}");
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
entity properties.parameterization fit {title="First scientific selection",source=evidence}
entity properties.parameterization other_fit {title="Unrelated scientific selection",source=evidence}
entity properties.property family {quantity=Scalar,shape=properties.IndexShape.pair,applies={compatibility.PhaseType.liquidPhase}}
entity interactions.symmetric_scalar ab {parameterization=fit,family=family,first=a,second=b,value=0.25,source=evidence,dependencies={bc},conventions={}}
entity interactions.symmetric_scalar ac {parameterization=fit,family=family,first=a,second=c,value=0.25,source=evidence,dependencies={},conventions={}}
entity interactions.symmetric_scalar bc {parameterization=fit,family=family,first=b,second=c,value=0.25,source=evidence,dependencies={},conventions={}}
entity interactions.symmetric_scalar ab_other {parameterization=other_fit,family=family,first=a,second=b,value=0.25,source=evidence,dependencies={bc_other},conventions={}}
entity interactions.symmetric_scalar ac_other {parameterization=other_fit,family=family,first=a,second=c,value=0.25,source=evidence,dependencies={},conventions={}}
entity interactions.symmetric_scalar bc_other {parameterization=other_fit,family=family,first=b,second=c,value=0.25,source=evidence,dependencies={},conventions={}}
entity properties.selection_context selected_context {roots={ab,ac,bc},subjects={a,b,c},models={family},rules={}}
entity properties.selection_context other_context {roots={ab_other,ac_other,bc_other},subjects={a,b,c},models={family},rules={}}
entity properties.property_package model {admitted=selected_context}
entity properties.property_package other_model {admitted=other_context}
set packages:Set<properties.property_package>={model,other_model};
set families:Set<properties.property>={family}; set members:Set<chemistry.species>={a,b,c};
dataset chosen:interactions.symmetric_selection complete_over(k in packages,family in families,i in members,j in members) provenance(evidence,provenance.Role.synthetic) {
[model,family,a,b]=[ab,missing];[model,family,a,c]=[ac,missing];[model,family,b,c]=[bc,missing];
[other_model,family,a,b]=[ab_other,missing];[other_model,family,a,c]=[ac_other,missing];[other_model,family,b,c]=[bc_other,missing];
}
def Isolated {
permission empirical families(interactions.symmetric_scalar) allow_unknown true allow_extrapolation false;
let result:Scalar=interactions.selected_off_diagonal(model,family,a,b);
}
def Calls {
permission empirical families(interactions.symmetric_scalar) allow_unknown true allow_extrapolation false;
let first:Scalar=interactions.selected_off_diagonal(model,family,a,b);
let second:Scalar=interactions.selected_off_diagonal(other_model,family,a,b);
}
def Reader(k:properties.property_package) {
permission empirical families(interactions.symmetric_scalar) allow_unknown true allow_extrapolation false;
let result:Scalar=interactions.selected_off_diagonal(k,family,a,b);
}
def Children {
child first:Reader=Reader(model); child second:Reader=Reader(other_model);
}
}"#;
