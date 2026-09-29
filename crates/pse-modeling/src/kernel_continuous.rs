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
