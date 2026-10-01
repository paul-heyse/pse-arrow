// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent evidence/permission truth tables, with exact named target controls.
use crate::applicability::*;
use crate::generated::enums::{ModelingApplicabilityOutcome as Outcome,ModelingPermissionTarget as Target,ModelingValidityLayer as Layer};
use pse_ids::SemanticId;
fn id(n:u8)->SemanticId {SemanticId::from_bytes([n;16])}
fn claim(n:u8)->Claim {Claim {id:Some(id(n)),coverage:None,owner:id(10),evidence:Some(id(11)),form:id(12),call:id(n+30),records:vec![id(n+50)],dependencies:vec![],layer:Layer::Data,basis:None,reason:None}}
fn node(n:u8,region:Region)->Node {Node {claim:claim(n),region,dependencies:vec![],inputs:vec![("T".into(),0,id(80))],permissions:vec![]}}
fn permission(n:u8,unknown:bool,extrapolate:bool)->Permission {Permission {id:id(n),scope:id(90),target_kind:Target::Families,targets:vec![id(10)],allow_unknown:unknown,allow_extrapolation:extrapolate}}
#[test]
fn named_unknown_and_outside_permissions_are_independent() {
    for (region,predicate) in [(Region::Unknown,false),(Region::Predicate(0),false)] {
        for unknown in [false,true] {for outside in [false,true] {
            let mut n=node(1,region.clone());n.permissions.push(permission(2,unknown,outside));
            let a=n.assess(&[predicate],&[400.]);
            assert_eq!(a.refused.is_empty(),if matches!(region,Region::Unknown){unknown}else{outside});
            assert_eq!(a.observations[0].inputs[0].quantity_type,id(80));
            assert_eq!(a.observations[0].permissions[0].scope,id(90));
        }}
    }
}
#[test]
fn required_mixed_dependencies_need_both_permissions_and_keep_all_lineage() {
    let mut root=node(1,Region::Unrestricted);
    root.dependencies=vec![node(2,Region::Unknown),node(3,Region::Predicate(0))];
    for unknown in [false,true] {for outside in [false,true] {
        let p=permission(4,unknown,outside);for n in &mut root.dependencies {n.permissions=vec![p.clone()];}
        let a=root.assess(&[false],&[450.]);
        assert_eq!(a.refused.is_empty(),unknown&&outside);
        assert_eq!(a.observations.len(),3);
        assert_eq!(a.observations[1].claim.id,Some(id(2)));
        assert_eq!(a.observations[2].outcome,Outcome::OutsideRegion);
        assert_eq!(a.observations[2].inputs[0].value,450.);
    }}
}
#[test]
fn declared_union_prefers_applicable_then_unknown_then_outside() {
    for (regions,outcome) in [
        (vec![Region::Predicate(0),Region::Unknown,Region::Unrestricted],Outcome::Applicable),
        (vec![Region::Predicate(0),Region::Unknown],Outcome::UnknownEvidence),
        (vec![Region::Predicate(0),Region::Predicate(1)],Outcome::OutsideRegion),
    ] {
        let alternatives=regions.into_iter().enumerate().map(|(i,r)|node(i as u8+2,r)).collect();
        let union=node(1,Region::Union(alternatives));
        let a=union.assess(&[false,false],&[1.]);
        assert_eq!(a.observations[0].outcome,outcome);
        assert_eq!(a.observations[1..].iter().filter(|o|o.required).count(),1);
        assert_eq!(a.refused.len(),usize::from(outcome!=Outcome::Applicable));
    }
}
#[test]
fn independent_fits_never_gain_implicit_union_acceptance() {
    let a=node(1,Region::Predicate(0)).assess(&[false],&[1.]);
    let b=node(2,Region::Unrestricted).assess(&[],&[1.]);
    assert_eq!(a.refused,vec![0]);assert!(b.refused.is_empty());
}
#[test]
fn record_permission_cannot_cover_sibling_or_unrelated_family() {
    let mut n=node(1,Region::Unknown);
    let mut p=permission(2,true,true);p.target_kind=Target::Records;p.targets=vec![id(52)];n.permissions=vec![p.clone()];
    assert_eq!(n.assess(&[],&[0.]).refused,vec![0]);
    p.targets=vec![id(51)];n.permissions=vec![p.clone()];assert!(n.assess(&[],&[0.]).refused.is_empty());
    p.target_kind=Target::Families;p.targets=vec![id(99)];n.permissions=vec![p];assert_eq!(n.assess(&[],&[0.]).refused,vec![0]);
}
#[test]
fn absent_claim_identity_is_unknown_and_retains_record_and_reason() {
    let mut n=node(1,Region::Unknown);n.claim.id=None;n.claim.reason=Some("no source claim".into());
    let a=n.assess(&[],&[2.]);assert_eq!(a.observations[0].outcome,Outcome::UnknownEvidence);
    assert_eq!(a.observations[0].claim.records,vec![id(51)]);assert_eq!(a.refused,vec![0]);
}
#[test]
fn complete_evidence_and_authorization_change_semantic_identity() {
    let n=node(1,Region::Unknown);
    let frame=|n:&Node| {let mut h=pse_ids::FramedHasher::new(pse_ids::Frame::ModelingApplicabilityCallV1);n.frame(&mut h);h.finish_id()};
    let mut changed=n.clone();changed.claim.evidence=Some(id(99));assert_ne!(frame(&n),frame(&changed));
    changed=n.clone();changed.permissions=vec![permission(2,true,false)];assert_ne!(frame(&n),frame(&changed));
    changed=n.clone();changed.claim.coverage=Some(id(98));assert_ne!(frame(&n),frame(&changed));
}

#[test]
fn union_winner_is_applicable_only_with_its_required_dependencies() {
    let mut incomplete=node(2,Region::Unrestricted);incomplete.dependencies=vec![node(3,Region::Unknown)];
    let complete=node(4,Region::Unrestricted);
    let union=node(1,Region::Union(vec![incomplete,complete]));
    let a=union.assess(&[],&[1.]);assert!(a.refused.is_empty());assert_eq!(a.observations[0].outcome,Outcome::Applicable);
    assert!(a.observations.iter().filter(|o|o.claim.id==Some(id(3))).all(|o|!o.required));
}

#[test]
fn union_winning_outside_child_and_unknown_dependency_require_both_flags() {
    let mut alternative=node(2,Region::Predicate(0));alternative.dependencies.push(node(3,Region::Unknown));
    let mut union=node(1,Region::Union(vec![alternative]));
    for (unknown,outside) in [(true,false),(false,true),(true,true)] {
        let p=permission(4,unknown,outside);union.permissions=vec![p.clone()];
        if let Region::Union(children)=&mut union.region {children[0].permissions=vec![p.clone()];children[0].dependencies[0].permissions=vec![p];}
        let a=union.assess(&[false],&[4.]);assert_eq!(a.refused.is_empty(),unknown&&outside);
        assert!(a.observations.iter().all(|o|o.required));
        assert_eq!(a.observations[1].outcome,Outcome::OutsideRegion);assert_eq!(a.observations[2].outcome,Outcome::UnknownEvidence);
    }
}
#[test]
fn nested_winning_union_recursively_retains_each_required_dependency() {
    let mut outside=node(3,Region::Predicate(0));outside.dependencies.push(node(4,Region::Unknown));
    let inner=node(2,Region::Union(vec![outside]));let mut outer=node(1,Region::Union(vec![inner]));
    fn permit(n:&mut Node,p:&Permission) {n.permissions=vec![p.clone()];if let Region::Union(c)=&mut n.region {for n in c {permit(n,p);}}for n in &mut n.dependencies {permit(n,p);}}
    permit(&mut outer,&permission(5,true,false));let a=outer.assess(&[false],&[4.]);
    assert!(!a.refused.is_empty());assert_eq!(a.observations.len(),4);assert!(a.observations.iter().all(|o|o.required));
    permit(&mut outer,&permission(5,true,true));assert!(outer.assess(&[false],&[4.]).refused.is_empty());
}
