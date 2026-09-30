// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::kernel_types::physical;
use crate::*;
use pse_ids::SemanticId;

fn run(text: &str) -> Result<SpecializedModel> {
    let (registry, _) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let package = check(&kernel_types::try_source(text)?, &context)?;
    specialize(
        &package,
        package.names["p.D"],
        InstanceId::from_id(SemanticId::NIL),
        &Bindings::default(),
        Limits::default(),
    )
}
const MODEL: &str = "package p { difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0.5,0.5); difference forward order(1) offsets(0,1) weights(-1,1) quadrature(0.5,0.5); def D { domain t: Scalar from 0 to 2; discretize mesh on t using backward(elements = 4, order = 1); var x[i in t]: Scalar; eq ode[i in t]: d(x[i])/di == 2; eq initial: x[0] == 0; let area: Scalar = integral(i in t | x[i]); eq output: area == 4; } }";

const TEMPORAL: &str = "package p { difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0,1); def Cell(times:Set<Time>={}) {param gain:Scalar=2; var x:Time; when analysis.dynamic {eq ode[i in times]:d(x)/di==gain;} when not analysis.dynamic {eq stationary:x==gain*1{s};} port out:Time=x;} def D {domain t:Time from 0{s} to 2{s}; when analysis.route==analysis.steady {discretize mesh on t using stationary(elements=1,order=1);} when analysis.route==analysis.integrated {discretize mesh on t using integrated(elements=1,order=1);} when analysis.route==analysis.simultaneous {discretize mesh on t using backward(elements=2,order=1);} child cell:Cell=Cell(); evolve time_state on cell using t bind times; when analysis.dynamic {eq initial:cell[0{s}].x==0{s};} let endpoint:Time=cell[2{s}].x;} }";

fn temporal(text: &str, route: analysis::Route) -> Result<SpecializedModel> {
    let (registry, _) = physical();
    let context = TypeContext {
        quantities: &registry,
        preconditions: &pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap(),
        scope: &PhysicalScope::default(),
    };
    let package = check(&kernel_types::try_source(text)?, &context)?;
    specialize(
        &package,
        package.names["p.D"],
        InstanceId::from_id(SemanticId::NIL),
        &Bindings::default().with_analysis(route),
        Limits::default(),
    )
}

#[test]
fn temporal_composition_lifts_states_and_ports_and_shares_scalar_parameters() {
    use crate::analysis::Route;
    use pse_model::generated::enums::ModelingDeclarationKind as Kind;
    for (route, states) in [
        (Route::Steady, 1),
        (Route::Integrated, 1),
        (Route::Simultaneous, 3),
    ] {
        let model = temporal(
            &TEMPORAL.replace("let endpoint:Time=cell[2{s}].x;", ""),
            route,
        )
        .unwrap();
        assert_eq!(
            model
                .symbols
                .values()
                .filter(|symbol| symbol.role == Kind::Variable
                    && !model
                        .derivatives
                        .values()
                        .any(|derivative| derivative.rate == symbol.id))
                .count(),
            states
        );
        assert_eq!(
            model
                .symbols
                .values()
                .filter(|symbol| symbol.role == Kind::Parameter
                    && symbol.lineage.path.ends_with(".gain"))
                .count(),
            1
        );
        assert_eq!(model.ports.len(), states);
        assert_eq!(model.meshes.len(), 1);
        assert_eq!(
            model.integrated.len(),
            usize::from(route == Route::Integrated)
        );
    }
    let spatial = TEMPORAL
        .replace(
            "child cell:Cell=Cell();",
            "set sites:Set<Scalar>={0,1}; child cell[j in sites]:Cell=Cell();",
        )
        .replace("cell[0{s}].x", "cell[0{s},0].x")
        .replace("let endpoint:Time=cell[2{s}].x;", "");
    let model = temporal(&spatial, Route::Simultaneous).unwrap();
    assert_eq!(
        model
            .symbols
            .values()
            .filter(|symbol| symbol.role == Kind::Variable)
            .count(),
        6
    );
    assert_eq!(
        model
            .symbols
            .values()
            .filter(
                |symbol| symbol.role == Kind::Parameter && symbol.lineage.path.ends_with(".gain")
            )
            .count(),
        2
    );
}

#[test]
fn temporal_composition_refuses_competing_axes_bindings_and_wrong_quantity() {
    use crate::analysis::Route;
    for text in [
        TEMPORAL.replace(
            "evolve time_state",
            "evolve other on cell using t bind times; evolve time_state",
        ),
        TEMPORAL.replace("bind times", "bind absent"),
        TEMPORAL.replace("Cell(); evolve", "Cell(times={0{s}}); evolve"),
        TEMPORAL.replace("domain t:Time", "domain t:Scalar"),
        TEMPORAL.replace(
            "using integrated(elements=1,order=1)",
            "using stationary(elements=1,order=1)",
        ),
    ] {
        assert!(
            temporal(
                &text.replace("let endpoint:Time=cell[2{s}].x;", ""),
                Route::Integrated
            )
            .is_err(),
            "{text}"
        );
    }
}

#[test]
fn temporal_composition_shares_nested_parameters_and_refuses_two_global_time_axes() {
    let nested = TEMPORAL
        .replace(
            "def Cell",
            "def Nested {param coefficient:Scalar=3;var y:Scalar;} def Cell",
        )
        .replace(
            "param gain:Scalar=2;",
            "param gain:Scalar=2;child inner:Nested=Nested();",
        )
        .replace("let endpoint:Time=cell[2{s}].x;", "");
    let model = temporal(&nested, analysis::Route::Simultaneous).unwrap();
    assert_eq!(
        model
            .symbols
            .values()
            .filter(|symbol| symbol.role
                == pse_model::generated::enums::ModelingDeclarationKind::Parameter
                && symbol.lineage.path.ends_with(".coefficient"))
            .count(),
        1
    );
    assert_eq!(
        model
            .symbols
            .values()
            .filter(|symbol| symbol.lineage.path.ends_with(".y"))
            .count(),
        3
    );
    let two_axes=TEMPORAL.replace("child cell:Cell=Cell();", "domain u:Time from 0{s} to 2{s};discretize other_mesh on u using backward(elements=2,order=1);child other:Cell=Cell();evolve other_time on other using u bind times;child cell:Cell=Cell();")
        .replace("let endpoint:Time=cell[2{s}].x;", "");
    let refusal = temporal(&two_axes, analysis::Route::Simultaneous)
        .unwrap_err()
        .to_string();
    assert!(refusal.contains("temporal axis"), "{refusal}");
}

#[test]
fn integrated_checks_and_numerical_conditionals_keep_the_runtime_clock() {
    let text=TEMPORAL.replace("let endpoint:Time=cell[2{s}].x;",
        "let observed[i in t]:Time=if i==0{s} then 0{s} else cell[i].x;annotation check observed(i!=0{s} or observed[i]==0{s});annotation check observed(i!=1{s} or observed[i]==2{s});");
    let model = temporal(&text, analysis::Route::Integrated).unwrap();
    let time = specialize::symbol_name(model.integrated.values().next().unwrap().time);
    let checks = model
        .annotations
        .iter()
        .filter_map(|a| match &a.value {
            annotation::AnnotationValue::Check(predicate) => {
                Some(pse_authoring::dsl::render_predicate(predicate))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(checks.len(), 2);
    assert!(
        checks.iter().all(|predicate| predicate.contains(&time)),
        "{checks:?}"
    );
    let observed = model
        .symbols
        .values()
        .filter_map(|symbol| symbol.expression.as_ref())
        .map(pse_authoring::dsl::render_expr)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(observed.contains(&format!("{time} ==")), "{observed}");
}

#[test]
fn temporal_composition_refuses_time_varying_scalar_defaults() {
    let text = TEMPORAL
        .replace(
            "param gain:Scalar=2;",
            "param gain:Scalar=sum(i in times | i/1{s});",
        )
        .replace("let endpoint:Time=cell[2{s}].x;", "");
    let error = temporal(&text, analysis::Route::Simultaneous).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("scalar temporal parameter defaults must agree"),
        "{error}"
    );
}

#[test]
fn continuous_child_literals_resolve_the_admitted_coordinate() {
    let text = "package p { difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0,1); def Cell {var x:Scalar;} def D {domain t:Scalar from 0 to 2; discretize grid on t using backward(elements=2,order=1); child cell[i in t]:Cell=Cell(); eq ode[i in t]:d(cell[i].x)/di==1; eq initial:cell[0].x==0; let endpoint:Scalar=cell[2].x;} }";
    let model = run(text).unwrap();
    assert_eq!(model.equations.len(), 3);
    assert_eq!(model.instances.len(), 4);
    assert!(run(&text.replace("cell[0].x==0", "cell[0.5].x==0")).is_err());
    assert!(run(&text.replace("cell[0].x==0", "cell[0{mol/s}].x==0")).is_err());
}

#[test]
fn continuous_expansion_retains_boundary_and_original_domain() {
    let model = run(MODEL).unwrap();
    let mesh = model.meshes.values().next().unwrap();
    assert_eq!(mesh.points.len(), 5);
    assert!(mesh.derivative[0].is_empty());
    assert_eq!(model.equations.len(), 6);
    assert!((mesh.integral.iter().sum::<f64>() - 2.0).abs() < 1e-14);
    // A linear profile is differentiated exactly by every populated row.
    for weights in &mesh.derivative[1..] {
        let slope = weights.iter().map(|(i, w)| w * (*i as f64)).sum::<f64>();
        assert!((slope - 2.0).abs() < 1e-14);
    }
    let printed = model
        .equations
        .iter()
        .map(|e| pse_authoring::dsl::render_equation(&e.equation))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!printed.contains("derivative"));
    assert!(!printed.contains("integral"));
}

#[test]
fn continuous_coordinate_identity_excludes_bounds_and_includes_mesh() {
    let a = run(MODEL).unwrap();
    let b = run(&MODEL.replace("to 2", "to 3")).unwrap();
    let c = run(&MODEL.replace("elements = 4", "elements = 5")).unwrap();
    let ids = |m: &SpecializedModel| {
        m.meshes
            .values()
            .next()
            .unwrap()
            .points
            .iter()
            .map(specialize::Value::identity)
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&a), ids(&b));
    assert_ne!(ids(&a), ids(&c));
}

#[test]
fn continuous_refuses_invalid_domains_policies_and_budgets() {
    for bad in [
        MODEL.replace("to 2", "to -1"),
        MODEL.replace("elements = 4", "elements = 0"),
        MODEL.replace("order = 1", "order = 2"),
        MODEL.replace("elements = 4", "elements = 99999999"),
        MODEL.replace("Scalar from 0", "Flow from 0"),
        MODEL.replace(
            "var x[i",
            "discretize other on t using forward(elements = 4, order = 1); var x[i",
        ),
    ] {
        assert!(run(&bad).is_err(), "{bad}");
    }
}

#[test]
fn equation_family_annotations_are_checked_before_selection() {
    let model =
        run(&MODEL.replace("eq output:", "annotation scale ode(inverseRSS); eq output:")).unwrap();
    assert_eq!(model.annotations.len(), 4);
    assert!(
        model
            .annotations
            .iter()
            .all(|a| model.equations.iter().any(|e| e.id == a.target))
    );
    for bad in [
        MODEL.replace("eq output:", "annotation scale ode(arbitrary); eq output:"),
        MODEL.replace("eq output:", "annotation start ode(0); eq output:"),
    ] {
        assert!(run(&bad).is_err());
    }
}

#[test]
fn continuous_schemes_are_visible_data_and_lattice_offsets_cross_elements() {
    let source = "package p { difference custom order(2) offsets(-1,1) weights(-0.5,0.5) quadrature(0.16666666666666666,0.6666666666666666,0.16666666666666666); def D { domain x:Scalar from 0 to 2; discretize mesh on x using custom(elements=2,order=2); } }";
    let model = run(source).unwrap();
    let mesh = model.meshes.values().next().unwrap();
    assert_eq!(mesh.derivative.len(), 5);
    assert!(mesh.derivative[0].is_empty());
    assert!(mesh.derivative[4].is_empty());
    for (i, derivative) in mesh.derivative.iter().enumerate().take(4).skip(1) {
        let value = derivative
            .iter()
            .map(|(j, w)| w * (0.5 * *j as f64).powi(2))
            .sum::<f64>();
        assert!((value - i as f64).abs() < 1e-14);
    }
    for bad in [
        source.replace("weights(-0.5,0.5)", "weights(-0.5,1)"),
        source.replace("offsets(-1,1)", "offsets(1,1)"),
        source.replace("using custom", "using backward"),
        source.replace("order=2", "order=1"),
    ] {
        assert!(run(&bad).is_err(), "{bad}");
    }
    assert!(run("package foreign { collocation hidden alpha(0) beta(0) right(false); } package p { def D { domain t:Scalar from 0 to 1; discretize mesh on t using foreign.hidden(elements=1,order=2); } }").is_err());
    assert!(run("package p { collocation bad alpha(-1) beta(0) right(true); def D {} }").is_err());
}

/// Rendered equations as a sorted multiset: row identities and lineage differ between
/// formulations, the realized mathematics must not.
fn rendered(model: &SpecializedModel) -> Vec<String> {
    let mut rows = model
        .equations
        .iter()
        .map(|e| pse_authoring::dsl::render_equation(&e.equation))
        .collect::<Vec<_>>();
    rows.sort();
    rows
}
const REPLICA_SCHEMES: &str = "difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0,1); difference forward order(1) offsets(0,1) weights(-1,1) quadrature(1,0);";

#[test]
fn replica_derivative_reads_the_sibling_replica_at_each_stencil_point() {
    for (scheme, boundary) in [("backward", "cell[0].x==0"), ("forward", "cell[2].x==0")] {
        // The derivative is authored inside the replicated definition, along the coordinate
        // its owner binds; the forward stencil reads replicas instantiated after it.
        let inside = format!(
            "package p {{ {REPLICA_SCHEMES} def Cell(at:Scalar) {{ var x:Scalar; var y:Scalar; eq ode:d(x)/d at==y; eq source:y==1; }} def D {{ domain t:Scalar from 0 to 2; discretize grid on t using {scheme}(elements=4,order=1); child cell[i in t]:Cell=Cell(at=i); eq boundary:{boundary}; }} }}"
        );
        let outside = format!(
            "package p {{ {REPLICA_SCHEMES} def Cell(at:Scalar) {{ var x:Scalar; var y:Scalar; eq source:y==1; }} def D {{ domain t:Scalar from 0 to 2; discretize grid on t using {scheme}(elements=4,order=1); child cell[i in t]:Cell=Cell(at=i); eq ode[i in t]:d(cell[i].x)/di==cell[i].y; eq boundary:{boundary}; }} }}"
        );
        let inside = run(&inside).unwrap();
        let outside = run(&outside).unwrap();
        // Four stencil rows, five sources and one boundary condition.
        assert_eq!(inside.equations.len(), 10, "{scheme}");
        assert_eq!(rendered(&inside), rendered(&outside), "{scheme}");
        let ode = inside
            .equations
            .iter()
            .filter(|e| e.lineage.path.ends_with(".ode"))
            .collect::<Vec<_>>();
        assert_eq!(ode.len(), 4, "{scheme}");
        // Every stencil row reads the state of two distinct replicas.
        for row in ode {
            let text = pse_authoring::dsl::render_equation(&row.equation);
            let states = inside
                .symbols
                .values()
                .filter(|s| {
                    s.lineage.path.ends_with(".x") && text.contains(&specialize::symbol_name(s.id))
                })
                .map(|s| s.lineage.instance)
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(states.len(), 2, "{scheme}: {text}");
        }
    }
}

#[test]
fn replica_derivative_follows_the_replica_through_nested_children() {
    // The replicated unit owns a child whose equation differentiates along the unit's
    // coordinate: the stencil reads the corresponding child of each sibling replica.
    let inside = format!(
        "package p {{ {REPLICA_SCHEMES} def Inner(at:Scalar) {{ var x:Scalar; eq ode:d(x)/d at==1; }} def Cell(at:Scalar) {{ child inner:Inner=Inner(at=at); }} def D {{ domain t:Scalar from 0 to 2; discretize grid on t using backward(elements=2,order=1); child cell[i in t]:Cell=Cell(at=i); eq initial:cell[0].inner.x==0; }} }}"
    );
    let outside = format!(
        "package p {{ {REPLICA_SCHEMES} def Inner(at:Scalar) {{ var x:Scalar; }} def Cell(at:Scalar) {{ child inner:Inner=Inner(at=at); }} def D {{ domain t:Scalar from 0 to 2; discretize grid on t using backward(elements=2,order=1); child cell[i in t]:Cell=Cell(at=i); eq ode[i in t]:d(cell[i].inner.x)/di==1; eq initial:cell[0].inner.x==0; }} }}"
    );
    let inside = run(&inside).unwrap();
    assert_eq!(inside.equations.len(), 3);
    assert_eq!(rendered(&inside), rendered(&run(&outside).unwrap()));
}

#[test]
fn replica_derivative_on_an_integrated_axis_is_the_state_rate() {
    let text = "package p { def Cell(at:Time) { var x:Time; eq ode:d(x)/d at==1; } def D { domain t:Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); child cell[i in t]:Cell=Cell(at=i); eq initial:cell[0{s}].x==0{s}; } }";
    // The standard physical document names Time.
    let (registry, _) = physical();
    let preconditions =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    let context = TypeContext {
        preconditions: &preconditions,
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let package = check(&kernel_types::source(text), &context).unwrap();
    let model = specialize(
        &package,
        package.names["p.D"],
        InstanceId::from_id(SemanticId::NIL),
        &Bindings::default().with_analysis(analysis::Route::Integrated),
        Limits::default(),
    )
    .unwrap();
    // One replica at the integrated coordinate; its state has a rate symbol.
    assert_eq!(model.derivatives.len(), 1);
    assert_eq!(model.initial_equations.len(), 1);
}

#[test]
fn replica_derivative_refuses_a_coordinate_without_a_mesh() {
    // A plain value is not a realized coordinate, whether or not the instance is replicated.
    let text = format!(
        "package p {{ {REPLICA_SCHEMES} def Cell(at:Scalar) {{ var x:Scalar; eq ode:d(x)/d at==1; }} def D {{ child cell:Cell=Cell(at=1); }} }}"
    );
    assert!(run(&text).is_err());
}
