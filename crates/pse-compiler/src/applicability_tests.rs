// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinary authored record selection, finite specialization and actual library execution.
use crate::{
    authored_transfer_tests::inputs,
    workspace::{CompilerWorkspace, Profile, WorkspaceLimits},
};
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_ids::SemanticId;
use pse_math::binding::CaseValues;
use pse_modeling::{Bindings, Limits, PhysicalScope, specialize::root_instance};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};
const DEFINITIONS: &str = r#"
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
fn increment(x0:Scalar,x1:Scalar,s:fit)->Scalar applicability(applicability_interval(s.applicability(x0,s),s.applicability(x1,s)))=x1-x0;
"#;
fn point(extra: &str, body: &str) -> Result<crate::workspace::ModelingPointChecks, String> {
    let source = format!("package p {{{DEFINITIONS} {extra} test point {{{body}}}}}");
    let rows = parse(
        &source,
        SemanticId::from_bytes([122; 16]),
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .map_err(|e| e.to_string())?;
    let root = rows
        .iter()
        .find(|r| r.name == "point")
        .unwrap()
        .declaration_id;
    let mut workspace = CompilerWorkspace::new(inputs(), WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .map_err(|e| e.to_string())?;
    let point = workspace
        .check_modeling_point(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .map_err(|e| e.to_string())?;
    assert!(!point.expectations.is_empty());
    assert!(point.expectations.iter().all(|e| e.passed));
    Ok(point)
}
fn run(body: &str) -> Result<(), String> {
    point("", body).map(|_| ())
}
#[test]
fn authored_known_regions_unknown_and_extrapolation_have_distinct_permissions() {
    assert!(run("expect law(2,a)==2 tolerance 1e-12;").is_ok());
    assert!(run("expect law(4,a)==4 tolerance 1e-12;").is_err());
    assert!(
        run("permission p records(a) allow_unknown true; expect law(4,a)==4 tolerance 1e-12;")
            .is_err()
    );
    assert!(
        run(
            "permission p records(a) allow_extrapolation true; expect law(4,a)==4 tolerance 1e-12;"
        )
        .is_ok()
    );
    assert!(
        run(
            "permission p records(b) allow_extrapolation true; expect law(2,b)==2 tolerance 1e-12;"
        )
        .is_err()
    );
    assert!(
        run("permission p records(b) allow_unknown true; expect law(2,b)==2 tolerance 1e-12;")
            .is_ok()
    );
}
#[test]
fn authored_record_authorization_does_not_widen_to_sibling_record() {
    assert!(run("permission p records(a) allow_unknown true allow_extrapolation true; expect law(2,b)==2 tolerance 1e-12;").is_err());
    assert!(
        run("permission p families(fit) allow_unknown true; expect law(2,b)==2 tolerance 1e-12;")
            .is_ok()
    );
}
#[test]
fn permitted_claim_cannot_override_authored_mathematical_domain() {
    let error=run("permission p families(fit) allow_unknown true allow_extrapolation true; expect law(-1,b)==-1 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("validity"), "{error}");
}
#[test]
fn declared_interval_coverage_refuses_general_region_with_same_callable_signature() {
    run("expect increment(1,3,a)==2 tolerance 1e-12;").unwrap();
    let error = run("expect increment(1,3,c)==2 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("explicitly declared interval"), "{error}");
    assert!(run("permission p families(fit) allow_extrapolation true; expect increment(1,4,a)==3 tolerance 1e-12;").is_ok());
}
#[test]
fn active_branch_and_actual_derived_arguments_decide_evidence() {
    assert!(run("expect (if true then law(2,a) else law(2,b))==2 tolerance 1e-12;").is_ok());
    assert!(run("expect law(2+2,a)==4 tolerance 1e-12;").is_err());
    assert!(run("permission p records(a) allow_extrapolation true; expect law(2+2,a)==4 tolerance 1e-12;").is_ok());
}
#[test]
fn authored_cancellation_and_partial_cannot_erase_unknown_evidence() {
    let error = run("expect 0*law(2,b)==0 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let error = run("expect partial(law,x)(2,b)==1 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("applicability"), "{error}");
}

#[test]
fn authored_selected_dependencies_require_both_flags_at_actual_derived_input() {
    let declarations = r#"
      entity fit root_fit {source=evidence;applicability=region;lower=0{1};upper=10{1};dependencies={a,b};}
      fn required(s:fit,c:Set<fit>)->Set<fit>=s.dependencies;
      fn selected_law(x:Scalar,s:fit)->Scalar applicability(s.applicability(x,s))= (if s in selected then s.value*x else 0) where selected=selection_closure(set_of(s),set_of(s),required);
    "#;
    for permission in [
        "",
        "permission p records(a) allow_extrapolation true;",
        "permission p records(b) allow_unknown true;",
    ] {
        let error = point(
            declarations,
            &format!("{permission} expect selected_law(2+2,root_fit)==4 tolerance 1e-12;"),
        )
        .unwrap_err();
        assert!(error.contains("applicability"), "{error}");
    }
    let checks=point(declarations,"permission p records(a) allow_extrapolation true; permission q records(b) allow_unknown true; expect selected_law(2+2,root_fit)==4 tolerance 1e-12;").unwrap();
    use pse_model::generated::enums::ModelingApplicabilityOutcome as O;
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.outcome == O::OutsideRegion && o.extrapolation_allowed)
    );
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.outcome == O::UnknownEvidence && o.unknown_allowed)
    );
    assert!(
        checks
            .applicability
            .iter()
            .filter(|o| !o.inputs.is_empty())
            .all(|o| o.inputs[0].value == 4.)
    );
}
#[test]
fn actually_read_selected_record_without_claim_is_unknown_but_unused_record_does_not_gate() {
    let declarations = r#"
      entity kind bare {attribute value:Scalar;attribute dependencies:Set<bare>={};}
      entity bare consumed {value=2{1};} entity bare unused {value=9{1};}
      fn required_bare(s:bare,c:Set<bare>)->Set<bare>=s.dependencies;
      fn selected_value(s:bare)->Scalar= (if s in records then s.value else 0) where records=selection_closure(set_of(s),set_of(s),required_bare);
      fn wrapper(x:Scalar,s:fit)->Scalar=law(x,s);
    "#;
    let error = point(
        declarations,
        "expect selected_value(consumed)==2 tolerance 1e-12;",
    )
    .unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks=point(declarations,"permission p records(consumed) allow_unknown true; expect selected_value(consumed)==2 tolerance 1e-12; expect wrapper(2,a)==2 tolerance 1e-12;").unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.claim.id.is_none() && o.unknown_allowed)
    );
    assert!(
        checks
            .applicability
            .iter()
            .filter(|o| o.claim.id.is_none())
            .count()
            == 1
    );
}
#[test]
fn authored_declared_union_prefers_known_applicable_region_over_unknown_then_outside() {
    let declarations = r#"
      applicability disjunction(x:Scalar,s:fit) owner fit scope data evidence s.source union(region(x,s),unknown_region(x,s));
      entity fit union_fit {source=evidence;applicability=disjunction;lower=1{1};upper=3{1};}
    "#;
    let checks = point(declarations, "expect law(2,union_fit)==2 tolerance 1e-12;").unwrap();
    use pse_model::generated::enums::ModelingApplicabilityOutcome as O;
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.outcome == O::Applicable)
    );
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| !o.required && o.outcome == O::UnknownEvidence)
    );
    let error=point(declarations,"permission p records(union_fit) allow_extrapolation true; expect law(4,union_fit)==4 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks=point(declarations,"permission p records(union_fit) allow_unknown true; expect law(4,union_fit)==4 tolerance 1e-12;").unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.outcome == O::UnknownEvidence && o.unknown_allowed)
    );
}

#[test]
fn direct_record_read_preserves_runtime_branch_demand_and_direct_member_refusal() {
    let declarations = "fn choose(x:Scalar,s:fit,t:fit)->Scalar=if x>0 then s.value else t.value;";
    let checks = point(declarations, "expect choose(2,a,b)==1 tolerance 1e-12;").unwrap();
    use pse_model::generated::enums::ModelingApplicabilityOutcome as O;
    assert!(
        checks
            .applicability
            .iter()
            .all(|o| o.outcome == O::Applicable)
    );
    let error = point(declarations, "expect choose(-2,a,b)==1 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks = point(
        declarations,
        "permission p records(b) allow_unknown true; expect choose(-2,a,b)==1 tolerance 1e-12;",
    )
    .unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.outcome == O::UnknownEvidence && o.inputs[0].value == -2.)
    );
    let error = point("", "expect b.value==1 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    assert!(
        point(
            "",
            "permission p records(b) allow_unknown true; expect b.value==1 tolerance 1e-12;"
        )
        .is_ok()
    );
}

#[test]
fn annotated_hard_range_is_unconditional_and_has_no_evidence_permission_policy() {
    let declarations =
        "def HardRange(x:Scalar=4) {let actual:Scalar=x;annotation valid actual(1,3);}";
    let error=point(declarations,"permission p families(fit) allow_unknown true allow_extrapolation true; child unit:HardRange=HardRange(); expect unit.actual==4 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("validity"), "{error}");
    let source = "package p {def D {var x:Scalar;annotation valid x(1,3,extrapolate);}}";
    assert!(
        parse(
            source,
            SemanticId::from_bytes([124; 16]),
            IdentityPolicy::Named,
            ParseBudget::default()
        )
        .is_err()
    );
}

#[test]
fn scientific_parameter_default_requires_permission_on_actual_read() {
    let error = point(
        "",
        "param scale:Scalar=b.value; expect scale==1 tolerance 1e-12;",
    )
    .unwrap_err();
    assert!(error.contains("applicability"), "{error}");
}

#[test]
fn permitted_scientific_parameter_default_retains_unknown_observation() {
    let checks = point("", "permission p records(b) allow_unknown true; param scale:Scalar=b.value; expect scale==1 tolerance 1e-12;").unwrap();
    use pse_model::generated::enums::ModelingApplicabilityOutcome as O;
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.outcome == O::UnknownEvidence && o.unknown_allowed)
    );
}

#[test]
fn unused_scientific_parameter_default_does_not_require_permission() {
    let checks = point(
        "",
        "param scale:Scalar=b.value; expect 1==1 tolerance 1e-12;",
    )
    .unwrap();
    assert!(checks.applicability.is_empty());
}

#[test]
fn inactive_scientific_parameter_default_does_not_require_permission() {
    let checks = point(
        "",
        "param scale:Scalar=b.value; expect (if true then 1 else scale)==1 tolerance 1e-12;",
    )
    .unwrap();
    assert!(checks.applicability.is_empty());
    let checks = point("", "param flag:Scalar=1; param scale:Scalar=b.value; expect (if flag>0 then 1 else scale)==1 tolerance 1e-12;").unwrap();
    assert!(checks.applicability.is_empty());
    let error = point("", "param flag:Scalar=-1; param scale:Scalar=b.value; expect (if flag>0 then 1 else scale)==1 tolerance 1e-12;").unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks = point("", "permission p records(b) allow_unknown true; param flag:Scalar=-1; param scale:Scalar=b.value; expect (if flag>0 then 1 else scale)==1 tolerance 1e-12;").unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.unknown_allowed)
    );
}

#[test]
fn actual_parameter_override_and_constructor_record_keep_their_source() {
    let declarations = r#"
      interface Defaults {param scale:Scalar=b.value;}
      def Literal:Defaults {override param scale:Scalar=7{1};}
      def Selected(selected:fit) {param scale:Scalar=selected.value;}
    "#;
    let checks = point(
        declarations,
        "child unit:Defaults=Literal(); expect unit.scale==7 tolerance 1e-12;",
    )
    .unwrap();
    assert!(checks.applicability.is_empty());
    let error = point(
        declarations,
        "child unit:Selected=Selected(selected=b); expect unit.scale==1 tolerance 1e-12;",
    )
    .unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks = point(declarations, "permission p records(b) allow_unknown true; child unit:Selected=Selected(selected=b); expect unit.scale==1 tolerance 1e-12;").unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.unknown_allowed)
    );
}

#[test]
fn numerical_scope_defaults_and_actual_constructor_arguments_keep_evidence() {
    let declarations = "def Scoped(scale:Scalar=b.value) {let actual:Scalar=scale;}";
    let error = point(
        declarations,
        "child unit:Scoped=Scoped(); expect unit.actual==1 tolerance 1e-12;",
    )
    .unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks = point(
        declarations,
        "child unit:Scoped=Scoped(scale=7{1}); expect unit.actual==7 tolerance 1e-12;",
    )
    .unwrap();
    assert!(checks.applicability.is_empty());
    let error = point(
        declarations,
        "child unit:Scoped=Scoped(scale=b.value); expect unit.actual==1 tolerance 1e-12;",
    )
    .unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks = point(declarations, "permission p records(b) allow_unknown true; child unit:Scoped=Scoped(scale=b.value); expect unit.actual==1 tolerance 1e-12;").unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.unknown_allowed)
    );
    for body in [
        "child unit:Scoped=Scoped(); expect 1==1 tolerance 1e-12;",
        "child unit:Scoped=Scoped(); expect (if true then 1 else unit.actual)==1 tolerance 1e-12;",
    ] {
        assert!(point(declarations, body).unwrap().applicability.is_empty());
    }
    let error = point(
        "",
        "param scale:Scalar=b.value; expect scale-scale==0 tolerance 1e-12;",
    )
    .unwrap_err();
    assert!(error.contains("applicability"), "{error}");
}

#[test]
fn parameter_alias_evidence_stays_inside_the_demanded_runtime_branch() {
    let body = "param flag:Scalar=1; param scale:Scalar=b.value; let actual:Scalar=scale; expect (if flag>0 then 1 else actual)==1 tolerance 1e-12;";
    assert!(point("", body).unwrap().applicability.is_empty());
    let active = body.replace("flag:Scalar=1", "flag:Scalar=-1");
    let error = point("", &active).unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks = point(
        "",
        &format!("permission p records(b) allow_unknown true; {active}"),
    )
    .unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.unknown_allowed)
    );
}

#[test]
fn selected_record_without_claim_retains_unknown_through_parameter_replay() {
    let declarations = r#"
      entity kind bare {attribute value:Scalar; attribute dependencies:Set<bare>={};}
      entity bare chosen {value=1{1};}
      fn required_bare(s:bare,c:Set<bare>)->Set<bare>=s.dependencies;
      def Scoped(selected:Set<bare>,record:bare) {param scale:Scalar=record.value where admitted=selection_closure(selected,selected,required_bare); let actual:Scalar=scale;}
    "#;
    let body = "child unit:Scoped=Scoped(selected=selection_closure(set_of(chosen),set_of(chosen),required_bare),record=chosen); expect unit.actual==1 tolerance 1e-12;";
    let error = point(declarations, body).unwrap_err();
    assert!(error.contains("applicability"), "{error}");
    let checks = point(
        declarations,
        &format!("permission p records(chosen) allow_unknown true; {body}"),
    )
    .unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.claim.id.is_none() && o.unknown_allowed)
    );
}

#[test]
fn unused_parameter_selection_edges_do_not_enter_used_default_receipt() {
    let declarations = "fn contextual_dependencies(s:fit,c:Set<fit>)->Set<fit>=if b in c then set_of(b) else s.dependencies;";
    let body = "permission p records(a) allow_unknown true; param unused:Scalar=1{1} where selected=selection_closure(set_of(a),set_of(a,b),contextual_dependencies); param scale:Scalar=a.value where selected=selection_closure(set_of(a),set_of(a),contextual_dependencies); expect scale==1 tolerance 1e-12;";
    let checks = point(declarations, body).unwrap();
    assert!(
        checks
            .applicability
            .iter()
            .any(|o| o.required && o.unknown_allowed)
    );
    assert!(
        checks
            .applicability
            .iter()
            .filter(|o| o.required)
            .all(|o| o.claim.dependencies.is_empty())
    );
}

#[test]
fn constructor_set_selection_receipt_follows_membership_and_reduction_reads() {
    let declarations = r#"
      entity kind bare {attribute value:Scalar; attribute dependencies:Set<bare>={};}
      entity bare chosen {value=1{1};}
      entity bare unrelated {value=2{1};}
      fn required_bare(s:bare,c:Set<bare>)->Set<bare>=s.dependencies;
      def Membership(selected:Set<bare>,record:bare) {
        param scale:Scalar=if record in selected then record.value else 0{1};
        let actual:Scalar=scale;
      }
      def Reduction(selected:Set<bare>) {
        param scale:Scalar=sum(j in selected | j.value);
        let actual:Scalar=scale;
      }
      def MembershipAlias(selected:Set<bare>,record:bare) {
        param scale:Scalar=(if record in forwarded then record.value else 0{1}) where forwarded=selected;
        let actual:Scalar=scale;
      }
      def ReductionAlias(selected:Set<bare>) {
        param scale:Scalar=sum(j in forwarded | j.value) where forwarded=selected;
        let actual:Scalar=scale;
      }
      def ShadowedAlias(selected:Set<bare>,record:bare) {
        param scale:Scalar=(if record in next then record.value else 0{1}) where forwarded=selected,next=forwarded,selected=set_of(unrelated);
        let actual:Scalar=scale;
      }
      def LiteralSet(selected:Set<bare>={}) {
        param scale:Scalar=1;
        let actual:Scalar=scale;
      }
    "#;
    for constructor in [
        "Membership(selected=selection_closure(set_of(chosen),set_of(chosen),required_bare),record=chosen)",
        "Reduction(selected=selection_closure(set_of(chosen),set_of(chosen),required_bare))",
        "MembershipAlias(selected=selection_closure(set_of(chosen),set_of(chosen),required_bare),record=chosen)",
        "ReductionAlias(selected=selection_closure(set_of(chosen),set_of(chosen),required_bare))",
        "ShadowedAlias(selected=selection_closure(set_of(chosen),set_of(chosen),required_bare),record=chosen)",
    ] {
        let kind = constructor.split('(').next().unwrap();
        let child = format!("child unit:{kind}={constructor};");
        let active = format!("{child} expect unit.actual==1 tolerance 1e-12;");
        let error = point(declarations, &active).unwrap_err();
        assert!(error.contains("applicability"), "{kind}: {error}");
        let checks = point(
            declarations,
            &format!("permission p records(chosen) allow_unknown true; {active}"),
        )
        .unwrap();
        assert!(
            checks
                .applicability
                .iter()
                .any(|o| o.required && o.claim.id.is_none() && o.unknown_allowed),
            "{kind}"
        );
        let inactive = format!(
            "{child} param flag:Scalar=1; expect (if flag>0 then 1 else unit.actual)==1 tolerance 1e-12;"
        );
        assert!(
            point(declarations, &inactive)
                .unwrap()
                .applicability
                .is_empty(),
            "{kind}"
        );
        // A second constructor's receipt is unrelated to the actual source read.
        let unused = "child unused:Reduction=Reduction(selected=selection_closure(set_of(unrelated),set_of(unrelated),required_bare));";
        let checks = point(
            declarations,
            &format!("permission p records(chosen) allow_unknown true; {unused} {active}"),
        )
        .unwrap();
        assert!(
            checks
                .applicability
                .iter()
                .filter(|o| o.required)
                .all(|o| o.claim.dependencies.is_empty()),
            "{kind}"
        );
    }
    assert!(
        point(
            declarations,
            "child unit:LiteralSet=LiteralSet(); expect unit.actual==1 tolerance 1e-12;"
        )
        .unwrap()
        .applicability
        .is_empty()
    );
}
