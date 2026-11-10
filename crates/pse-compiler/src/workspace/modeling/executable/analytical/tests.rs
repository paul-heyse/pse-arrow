// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored source consumers exercise selection, original guards and actual native math together.
use super::*;
use crate::workspace::modeling::tests::setup as source_workspace;
fn scope() -> pse_kernels::ExecutionScope {
    pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None)
}
fn frontier(
    workspace: &mut CompilerWorkspace,
    root: DeclarationId,
    demand: &[&str],
) -> ModelingPreparationFrontier {
    workspace
        .plan_modeling_cancellable(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings {
                demand: demand.iter().map(|value| (*value).into()).collect(),
                ..Default::default()
            },
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
}
#[test]
fn analytical_original_indexed_source_preserves_selected_domain_and_whole_output() {
    let source = "package p {entity kind item {} entity item a {} entity item b {} set all:Set<item>={a,b};def Root {var A[i in all,j in all]:Scalar;var F[j in all]:Scalar;let dot:Scalar=sum(k in all | A[a,k]*F[k]);let renamed:Scalar=sum(j in all | A[a,j]*F[j]);let external:Scalar=sum(k in all | A[b,k]*F[k]);let shifted:Scalar=2*sum(k in all | A[a,k]*F[k]);}}";
    let (mut workspace, _, _, root) = source_workspace(source);
    let frontier = frontier(
        &mut workspace,
        root,
        &["dot", "renamed", "external", "shifted"],
    );
    let member = frontier.model().paths["dot"];
    let product = workspace
        .prepare_analytical_member(
            &frontier,
            member,
            AnalyticalOperation::Indexed,
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let AnalyticalObservation::Indexed(indexed) = &product.observation else {
        panic!("indexed")
    };
    assert_eq!(indexed.original_to_input.len(), 1);
    assert_eq!(product.region().indexed.len(), 1);
    assert!(product.source.indexed_context.is_some());
    let canonical = indexed.canonical.clone();
    let original_mapping = indexed.original_to_input.clone();
    let renamed = workspace
        .prepare_analytical_member(
            &frontier,
            frontier.model().paths["renamed"],
            AnalyticalOperation::Indexed,
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let AnalyticalObservation::Indexed(renamed) = renamed.observation else {
        panic!("indexed")
    };
    assert_eq!(
        renamed.canonical, canonical,
        "original j/k names share one admitted dummy alphabet"
    );
    assert_ne!(
        renamed.original_to_input, original_mapping,
        "original source binder identities remain separately attributed"
    );
    let external = workspace
        .prepare_analytical_member(
            &frontier,
            frontier.model().paths["external"],
            AnalyticalOperation::Indexed,
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let AnalyticalObservation::Indexed(external) = external.observation else {
        panic!("indexed")
    };
    assert_ne!(
        external.canonical, canonical,
        "fixed external physical member is never a dummy candidate"
    );

    assert!(
        workspace
            .prepare_analytical_member(
                &frontier,
                frontier.model().paths["shifted"],
                AnalyticalOperation::Indexed,
                EvaluationLimits::default(),
                &scope()
            )
            .is_err(),
        "an enclosing scalar expression is not the original whole contraction"
    );
}
#[test]
fn analytical_actual_shomate_source_derivatives_series_and_original_context() {
    let record = "correlations.shomate[chem.nitrogen,properties.heat_capacity,compatibility.PhaseType.vaporPhase,references.analytic_constant_fit]";
    let source = format!(
        "{}\npackage analytical_fixture {{use properties @\"1.0.0\";use correlations @\"1.0.0\";use compatibility @\"1.0.0\";use chem @\"1.0.0\";use references @\"1.0.0\";use provenance @\"1.0.0\"; dataset analytic_shomate:correlations.shomate bind(property=properties.heat_capacity,source=references.analytic_identities,parameterization=references.analytic_constant_fit,dependencies={{}},conventions={{}}) provenance(references.analytic_identities,provenance.Role.synthetic) {{[chem.nitrogen,compatibility.PhaseType.vaporPhase]=[200{{K}},1000{{K}},1{{J/(mol*K)}},0.001{{J/(mol*K^2)}},0.000002{{J/(mol*K^3)}},0.000000003{{J/(mol*K^4)}},10{{J*K/mol}}];}} test Root fixture {{dof 1;route steady;procedure check;}} {{var T:Temperature;let T0:Temperature=298.15{{K}};let h:DeltaH=properties.caloric_dh({record},T0,T);let s:DeltaS=properties.caloric_ds({record},T0,T);let cp:MolarCp=properties.caloric_cp({record},T);let interval:DeltaTemperature=300{{K}};let entropy_step:DeltaS=cp*(interval/T);let zero:DeltaH=0{{J/mol}};let h0:DeltaH=properties.caloric_dh({record},T0,T0);}}}}",
        crate::authored_transfer_tests::reference_sources()
    );
    // Consume current authored Shomate functions, with independently specified
    // synthetic keyed coefficients. External parameter transport is a separate
    // package/data qualification, as in the existing reference-source fixture.
    let rows = crate::authored_transfer_tests::rows(&source);
    let root = crate::authored_transfer_tests::root(&rows, "analytical_fixture", "Root");
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    let frontier = frontier(
        &mut workspace,
        root,
        &[
            "T",
            "T0",
            "h",
            "s",
            "cp",
            "entropy_step",
            "interval",
            "h0",
            "zero",
        ],
    );
    let paths = &frontier.model().paths;
    for (name, target) in [("h", "cp"), ("s", "entropy_step")] {
        let result = workspace
            .prepare_analytical_member(
                &frontier,
                paths[name],
                AnalyticalOperation::Identity {
                    other: paths[target],
                    coordinates: vec![paths["T"]],
                    multiplier: if name == "s" {
                        Some(paths["interval"])
                    } else {
                        None
                    },
                },
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap();
        assert!(
            matches!(
                result.observation,
                AnalyticalObservation::Identity {
                    meaning: pse_math::analytical::IdentityMeaning::Zero,
                    ..
                }
            ),
            "{name}: {:?}",
            result.observation
        );
        assert!(!result.source.guards.is_empty());
        assert!(!result.source.occurrences.is_empty());
        assert_eq!(result.source.consumed_physical, frontier.physical);
        assert_eq!(result.source.coordinates[0].member, paths["T"]);
    }
    let series = workspace
        .prepare_analytical_member(
            &frontier,
            paths["s"],
            AnalyticalOperation::Series {
                coordinate: paths["T"],
                point_member: paths["T0"],
                order: 5,
                relative: false,
            },
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    let AnalyticalObservation::Series(series) = series.observation else {
        panic!("series")
    };
    assert!(!series.exactly_terminating);
    assert_eq!(series.coefficients.len(), 5);
    assert_eq!(series.absolute_order.as_ref().unwrap().numerator, "6");
    assert!(series.absolute_order.is_some());
    let anchor = workspace
        .prepare_analytical_member(
            &frontier,
            paths["h0"],
            AnalyticalOperation::Identity {
                other: paths["zero"],
                coordinates: vec![],
                multiplier: None,
            },
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    assert!(matches!(
        anchor.observation,
        AnalyticalObservation::Identity {
            meaning: pse_math::analytical::IdentityMeaning::Zero,
            ..
        }
    ));
}

#[test]
fn analytical_exact_source_bits_are_distinct_from_numerical_body_identity() {
    let (mut workspace, _, _, root) = source_workspace(
        "package p {def Root {var x:Scalar;let f:Scalar=0.1*x;let c:Scalar=0.1;}}",
    );
    let frontier = frontier(&mut workspace, root, &["x", "f", "c"]);
    let member = frontier.model().paths["f"];
    let product = workspace
        .prepare_analytical_member(
            &frontier,
            member,
            AnalyticalOperation::Identity {
                other: frontier.model().paths["c"],
                coordinates: vec![frontier.model().paths["x"]],
                multiplier: None,
            },
            EvaluationLimits::default(),
            &scope(),
        )
        .unwrap();
    assert!(matches!(
        product.observation,
        AnalyticalObservation::Identity {
            meaning: pse_math::analytical::IdentityMeaning::Zero,
            ..
        }
    ));
    let exact = product.body.as_ref().unwrap();
    assert_ne!(exact.spec.policy, pse_math::binding::guarded_real_policy());
    assert!(!exact.has_portable_payload());
    let region = product.region();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: &workspace.inputs.quantities,
        preconditions: &workspace.inputs.preconditions,
        scope: &frontier.projection.physical_scope,
    };
    let mut refuse = |_: &str| Err(MathError::Contract("test provider".into()));
    let providers = crate::typed_math::prepare_selected_providers(
        &region.syntax,
        &region.support,
        &mut refuse,
        &BTreeMap::new(),
        &context,
    )
    .unwrap();
    let ordinary = crate::typed_math::Request {
        definition: product.source.declaration,
        syntax: &region.syntax,
        inputs: &region.inputs,
        support: &region.support,
        providers: &providers,
        physical: exact.spec.physical,
        structure: exact.spec.structure,
        limits: frontier.projection.body_limits,
    }
    .admit(
        &workspace.inputs.quantities,
        workspace.inputs.preconditions.as_ref(),
        scope().cancellation(),
    )
    .unwrap();
    assert_ne!(ordinary.spec.definition, exact.spec.definition);
    assert_ne!(ordinary.spec.key(), exact.spec.key());
}
