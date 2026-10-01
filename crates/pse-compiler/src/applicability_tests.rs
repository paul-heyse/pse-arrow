// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinary authored record selection, finite specialization and actual library execution.
use crate::{authored_transfer_tests::inputs,workspace::{CompilerWorkspace,Profile,WorkspaceLimits}};
use pse_authoring::{ParseBudget,language::{IdentityPolicy,parse}};
use pse_ids::SemanticId;
use pse_math::binding::CaseValues;
use pse_modeling::{Bindings,Limits,PhysicalScope,specialize::root_instance};
use std::{collections::BTreeMap,sync::{Arc,atomic::AtomicBool}};
const DEFINITIONS:&str=r#"
entity kind source provenance {} entity source evidence {}
entity kind fit {
 attribute source:source;
 attribute applicability:Fn(x:Scalar,s:fit)->Applicability;
 attribute lower:Scalar; attribute upper:Scalar;
 attribute dependencies:Set<fit>={}; attribute value:Scalar=1{1};
}
applicability region(x:Scalar,s:fit) owner fit scope data evidence s.source interval reported x in s.lower..s.upper;
applicability unknown_region(x:Scalar,s:fit) owner fit scope data evidence s.source unknown("No evidence region published");
applicability arbitrary(x:Scalar,s:fit) owner fit scope data evidence s.source region reported(x>=s.lower and x<=s.upper);
entity fit a {source=evidence;applicability=region;lower=1{1};upper=3{1};}
entity fit b {source=evidence;applicability=unknown_region;lower=1{1};upper=3{1};}
entity fit c {source=evidence;applicability=arbitrary;lower=1{1};upper=3{1};}
fn law(x:Scalar,s:fit)->Scalar applicability(s.applicability(x,s)) valid(x>0) =x;
fn integral(x0:Scalar,x1:Scalar,s:fit)->Scalar applicability(applicability_interval(s.applicability(x0,s),s.applicability(x1,s)))=x1-x0;
"#;
fn point(extra:&str,body:&str)->Result<crate::workspace::ModelingPointChecks,String> {
    let source=format!("package p {{{DEFINITIONS} {extra} test point {{{body}}}}}");
    let rows=parse(&source,SemanticId::from_bytes([122;16]),IdentityPolicy::Named,ParseBudget::default()).map_err(|e|e.to_string())?;
    let root=rows.iter().find(|r|r.name=="point").unwrap().declaration_id;
    let mut workspace=CompilerWorkspace::new(inputs(),WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(rows,PhysicalScope::default()).map_err(|e|e.to_string())?;
    let point=workspace.check_modeling_point(root,root_instance(root),Bindings::default(),Limits::default(),&CaseValues {scalars:BTreeMap::new()},Profile::default(),Arc::new(AtomicBool::new(false))).map_err(|e|e.to_string())?;
    assert!(!point.expectations.is_empty());assert!(point.expectations.iter().all(|e|e.passed));Ok(point)
}
fn run(body:&str)->Result<(),String> {point("",body).map(|_|())}
#[test]
fn authored_known_regions_unknown_and_extrapolation_have_distinct_permissions() {
    assert!(run("expect law(2,a)==2 tolerance 1e-12;").is_ok());
    assert!(run("expect law(4,a)==4 tolerance 1e-12;").is_err());
    assert!(run("permission p records(a) allow_unknown true; expect law(4,a)==4 tolerance 1e-12;").is_err());
    assert!(run("permission p records(a) allow_extrapolation true; expect law(4,a)==4 tolerance 1e-12;").is_ok());
    assert!(run("permission p records(b) allow_extrapolation true; expect law(2,b)==2 tolerance 1e-12;").is_err());
    assert!(run("permission p records(b) allow_unknown true; expect law(2,b)==2 tolerance 1e-12;").is_ok());
}
#[test]
fn authored_record_authorization_does_not_widen_to_sibling_record() {
    assert!(run("permission p records(a) allow_unknown true allow_extrapolation true; expect law(2,b)==2 tolerance 1e-12;").is_err());
    assert!(run("permission p families(fit) allow_unknown true; expect law(2,b)==2 tolerance 1e-12;").is_ok());
}
#[test]
fn permitted_claim_cannot_override_authored_mathematical_domain() {
    let error=run("permission p families(fit) allow_unknown true allow_extrapolation true; expect law(-1,b)==-1 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("validity"),"{error}");
}
#[test]
fn declared_interval_coverage_refuses_general_region_with_same_callable_signature() {
    assert!(run("expect integral(1,3,a)==2 tolerance 1e-12;").is_ok());
    let error=run("expect integral(1,3,c)==2 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("explicitly declared interval"),"{error}");
    assert!(run("permission p families(fit) allow_extrapolation true; expect integral(1,4,a)==3 tolerance 1e-12;").is_ok());
}
#[test]
fn active_branch_and_actual_derived_arguments_decide_evidence() {
    assert!(run("expect (if true then law(2,a) else law(2,b))==2 tolerance 1e-12;").is_ok());
    assert!(run("expect law(2+2,a)==4 tolerance 1e-12;").is_err());
    assert!(run("permission p records(a) allow_extrapolation true; expect law(2+2,a)==4 tolerance 1e-12;").is_ok());
}
#[test]
fn authored_cancellation_and_partial_cannot_erase_unknown_evidence() {
    let error=run("expect 0*law(2,b)==0 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("applicability"),"{error}");
    let error=run("expect partial(law,x)(2,b)==1 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("applicability"),"{error}");
}

#[test]
fn authored_selected_dependencies_require_both_flags_at_actual_derived_input() {
    let declarations=r#"
      entity fit root_fit {source=evidence;applicability=region;lower=0{1};upper=10{1};dependencies={a,b};}
      fn required(s:fit,c:Set<fit>)->Set<fit>=s.dependencies;
      fn selected_law(x:Scalar,s:fit)->Scalar applicability(s.applicability(x,s))=if s in selected then s.value*x else 0 where selected=selection_closure({s},{s},required);
    "#;
    for permission in ["","permission p records(a) allow_extrapolation true;","permission p records(b) allow_unknown true;"] {
        let error=point(declarations,&format!("{permission} expect selected_law(2+2,root_fit)==4 tolerance 1e-12;")).unwrap_err();
        assert!(error.contains("applicability"),"{error}");
    }
    let checks=point(declarations,"permission p records(a) allow_extrapolation true; permission q records(b) allow_unknown true; expect selected_law(2+2,root_fit)==4 tolerance 1e-12;").unwrap();
    use pse_model::generated::enums::ModelingApplicabilityOutcome as O;
    assert!(checks.applicability.iter().any(|o|o.outcome==O::OutsideRegion && o.extrapolation_allowed));
    assert!(checks.applicability.iter().any(|o|o.outcome==O::UnknownEvidence && o.unknown_allowed));
    assert!(checks.applicability.iter().filter(|o|!o.inputs.is_empty()).all(|o|o.inputs[0].value==4.));
}
#[test]
fn actually_read_selected_record_without_claim_is_unknown_but_unused_record_does_not_gate() {
    let declarations=r#"
      entity kind bare {attribute value:Scalar;attribute dependencies:Set<bare>={};}
      entity bare consumed {value=2{1};} entity bare unused {value=9{1};}
      fn required_bare(s:bare,c:Set<bare>)->Set<bare>=s.dependencies;
      fn selected_value(s:bare)->Scalar=if s in records then s.value else 0 where records=selection_closure({s},{s},required_bare);
      fn wrapper(x:Scalar,s:fit)->Scalar=law(x,s);
    "#;
    let error=point(declarations,"expect selected_value(consumed)==2 tolerance 1e-12;").unwrap_err();assert!(error.contains("applicability"),"{error}");
    let checks=point(declarations,"permission p records(consumed) allow_unknown true; expect selected_value(consumed)==2 tolerance 1e-12; expect wrapper(2,a)==2 tolerance 1e-12;").unwrap();
    assert!(checks.applicability.iter().any(|o|o.claim.id.is_none() && o.unknown_allowed));
    assert!(checks.applicability.iter().filter(|o|o.claim.id.is_none()).count()==1);
}
#[test]
fn authored_declared_union_prefers_known_applicable_region_over_unknown_then_outside() {
    let declarations=r#"
      applicability disjunction(x:Scalar,s:fit) owner fit scope data evidence s.source union(region(x,s),unknown_region(x,s));
      entity fit union_fit {source=evidence;applicability=disjunction;lower=1{1};upper=3{1};}
    "#;
    let checks=point(declarations,"expect law(2,union_fit)==2 tolerance 1e-12;").unwrap();
    use pse_model::generated::enums::ModelingApplicabilityOutcome as O;
    assert!(checks.applicability.iter().any(|o|o.required && o.outcome==O::Applicable));
    assert!(checks.applicability.iter().any(|o|!o.required && o.outcome==O::UnknownEvidence));
    let error=point(declarations,"permission p records(union_fit) allow_extrapolation true; expect law(4,union_fit)==4 tolerance 1e-12;").unwrap_err();assert!(error.contains("applicability"),"{error}");
    let checks=point(declarations,"permission p records(union_fit) allow_unknown true; expect law(4,union_fit)==4 tolerance 1e-12;").unwrap();
    assert!(checks.applicability.iter().any(|o|o.required && o.outcome==O::UnknownEvidence && o.unknown_allowed));
}
