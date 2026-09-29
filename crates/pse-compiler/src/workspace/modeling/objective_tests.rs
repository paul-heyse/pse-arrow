// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Multi-objective authoring, levels and generated objective bounds (ADR-0111, I15).
use super::domain_tests::{power, setup};
use super::*;
use pse_math::jets::EvaluationLimits;
use pse_modeling::{ModelingError, ObjectiveRefusal, annotation::ObjectiveSense as Sense};
use std::collections::BTreeSet;

fn prepare(
    workspace: &mut CompilerWorkspace,
    root: DeclarationId,
    level: Option<usize>,
) -> Result<PreparedModeling> {
    let bindings = level.map_or_else(Bindings::default, |level| {
        Bindings::default().with_objective_level(level)
    });
    workspace.prepare_modeling_cancellable(
        root,
        InstanceId::from_id(SemanticId::NIL),
        bindings,
        Limits::default(),
        Arc::new(AtomicBool::new(false)),
    )
}
fn refusal(error: CompileError) -> ObjectiveRefusal {
    let mut error = match error {
        CompileError::Modeling(error) => error,
        other => panic!("expected a modeling refusal, got {other:?}"),
    };
    while let ModelingError::Located { cause, .. } = error {
        error = *cause;
    }
    match error {
        ModelingError::Objective { reason, .. } => reason,
        other => panic!("expected a typed objective refusal, got {other:?}"),
    }
}
fn refused(source: &str, level: Option<usize>) -> ObjectiveRefusal {
    let (mut workspace, root) = setup(source);
    refusal(prepare(&mut workspace, root, level).unwrap_err())
}
/// The symbol whose authored path ends with `suffix`.
fn symbol(model: &PreparedModeling, suffix: &str) -> SemanticId {
    let found = model
        .model
        .symbols
        .values()
        .filter(|s| s.lineage.path.ends_with(suffix))
        .map(|s| s.id)
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 1, "{suffix}: {found:?}");
    found[0]
}
/// Objective (minimization orientation) and constraint values at a complete point.
fn evaluate(
    workspace: &CompilerWorkspace,
    model: &PreparedModeling,
    point: &[(SemanticId, f64)],
) -> (f64, Vec<(SemanticId, f64)>) {
    let cancel = Arc::new(AtomicBool::new(false));
    let plan = model
        .admitted
        .plan(
            &workspace.inputs.quantities,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
    let assembly = Arc::new(
        plan.compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap(),
    );
    let values = CaseValues {
        scalars: model
            .admitted
            .inputs
            .iter()
            .map(|id| {
                let value = point
                    .iter()
                    .find(|(p, _)| p == id)
                    .map(|(_, v)| *v)
                    .or_else(|| match model.model.symbols[id].initial {
                        Some(pse_modeling::specialize::Value::Number { bits, .. }) => {
                            Some(f64::from_bits(bits))
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("no value for input {id}"));
                (*id, value)
            })
            .collect(),
    };
    let mut worker = assembly.worker(BTreeMap::new(), cancel);
    let objective = worker.objective(&values).unwrap();
    let rows = model
        .admitted
        .case
        .rows()
        .iter()
        .map(|r| r.id)
        .zip(worker.constraints(&values).unwrap())
        .collect();
    (objective, rows)
}

const LEXICOGRAPHIC: &str = "package p { def Root {
 var x: Scalar; var y: Scalar; var z: Power;
 let cost: Scalar = x*x + y;
 let risk: Scalar = (y - 1)*(y - 1);
 let power: Power = z;
 eq cap: z <= 10{W};
 annotation objective cost(minimize, priority = 0, absolute_tolerance = 0.5, relative_tolerance = 0.01);
 annotation objective risk(minimize, priority = 1, weight = 2, absolute_tolerance = 0, relative_tolerance = 0.1);
 annotation objective power(maximize, priority = 2);
} }";

#[test]
fn objective_bounds_rows_generated_per_level() {
    let (mut workspace, root) = setup(LEXICOGRAPHIC);
    // Without a selected level, the structure keeps every level as its own objective, in
    // optimization order, for a native lexicographic solve; every level but the last
    // states its degradation, and no bound row is generated.
    let all = prepare(&mut workspace, root, None).unwrap();
    let case = &all.admitted.case;
    assert!(case.lexicographic_levels());
    assert_eq!(
        case.objectives()
            .iter()
            .map(|o| o.sense)
            .collect::<Vec<_>>(),
        vec![
            pse_math::binding::ObjectiveSense::Minimize,
            pse_math::binding::ObjectiveSense::Minimize,
            pse_math::binding::ObjectiveSense::Maximize
        ]
    );
    assert_eq!(case.objectives()[2].quantity, power());
    assert_eq!(
        case.degradations(),
        [
            pse_math::binding::Degradation {
                absolute: 0.5,
                relative: 0.01
            },
            pse_math::binding::Degradation {
                absolute: 0.0,
                relative: 0.1
            }
        ]
    );
    assert!(
        all.model
            .objectives
            .levels
            .iter()
            .all(|l| l.bound.is_none())
    );
    // Each level's members contribute to that level's objective.
    let targets = case
        .instances()
        .iter()
        .flat_map(|i| i.contributions.iter())
        .filter_map(|c| match c.target {
            Target::Objective(level) => Some((level, c.scale.to_bits())),
            Target::Row(_) => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        targets.into_iter().collect::<Vec<_>>(),
        vec![
            (0, 1.0f64.to_bits()),
            (1, 2.0f64.to_bits()),
            (2, 1.0f64.to_bits())
        ]
    );
    let mut keys = BTreeSet::from([case.key()]);
    for level in 0..3 {
        let model = prepare(&mut workspace, root, Some(level)).unwrap();
        let objectives = &model.model.objectives;
        assert_eq!(objectives.selected, Some(level));
        assert_eq!(
            objectives
                .levels
                .iter()
                .map(|l| (l.priority, l.sense, l.members.len()))
                .collect::<Vec<_>>(),
            vec![
                (Some(0), Sense::Minimize, 1),
                (Some(1), Sense::Minimize, 1),
                (Some(2), Sense::Maximize, 1)
            ]
        );
        // Each earlier level, and only those, carries a generated β and its row.
        let bounds = objectives
            .levels
            .iter()
            .filter_map(|l| l.bound.clone())
            .collect::<Vec<_>>();
        assert_eq!(bounds.len(), level);
        for (j, bound) in bounds.iter().enumerate() {
            let parameter = &model.model.symbols[&bound.parameter];
            assert_eq!(parameter.initial, None, "β is bound by the engine");
            assert_eq!(
                parameter.ty,
                pse_modeling::Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                    objectives.levels[j].quantity
                ))
            );
            assert!(model.admitted.inputs.contains(&bound.parameter));
            let row = model
                .admitted
                .case
                .rows()
                .iter()
                .find(|r| r.id == bound.row)
                .unwrap();
            assert_eq!((row.lower, row.upper), (f64::NEG_INFINITY, 0.0));
        }
        // The case structure optimizes the selected level.
        let objective = model.admitted.case.objective().unwrap();
        assert_eq!(
            (objective.quantity, objective.sense),
            if level == 2 {
                (power(), pse_math::binding::ObjectiveSense::Maximize)
            } else {
                (
                    workspace.inputs.quantities.neutral_dimensionless().unwrap(),
                    pse_math::binding::ObjectiveSense::Minimize,
                )
            }
        );
        // Bounds are structure: one structure per level, stable when prepared again.
        assert!(keys.insert(model.admitted.case.key()));
        let again = prepare(&mut workspace, root, Some(level)).unwrap();
        assert_eq!(again.admitted.case.key(), model.admitted.case.key());
        if level == 2 {
            // Level values and bound rows at x = 1, y = 2, z = 3 W, β₀ = 4, β₁ = 5.
            let point = [
                (symbol(&model, ".x"), 1.0),
                (symbol(&model, ".y"), 2.0),
                (symbol(&model, ".z"), 3.0),
                (bounds[0].parameter, 4.0),
                (bounds[1].parameter, 5.0),
            ];
            let (objective, rows) = evaluate(&workspace, &model, &point);
            assert!(
                (objective + 3.0).abs() < 1e-12,
                "maximized power: {objective}"
            );
            let row = |id| rows.iter().find(|(r, _)| *r == id).unwrap().1;
            // cost − β₀ = 3 − 4, and 2·risk − β₁ = 2 − 5.
            assert!((row(bounds[0].row) + 1.0).abs() < 1e-12);
            assert!((row(bounds[1].row) + 3.0).abs() < 1e-12);
            // Each member enters its level with its weight.
            assert_eq!(
                objectives
                    .members
                    .iter()
                    .map(|m| (m.target, m.scale))
                    .collect::<Vec<_>>(),
                vec![
                    (symbol(&model, ".cost"), 1.0),
                    (symbol(&model, ".risk"), 2.0),
                    (symbol(&model, ".power"), 1.0)
                ]
            );
        }
    }
    // A maximized earlier level is bounded from below: `value >= β`.
    let maximized = LEXICOGRAPHIC.replace(
        "annotation objective cost(minimize",
        "annotation objective cost(maximize",
    );
    let (mut workspace, root) = setup(&maximized);
    let model = prepare(&mut workspace, root, Some(1)).unwrap();
    let bound = model.model.objectives.levels[0].bound.clone().unwrap();
    let row = model
        .admitted
        .case
        .rows()
        .iter()
        .find(|r| r.id == bound.row)
        .unwrap();
    assert_eq!((row.lower, row.upper), (0.0, f64::INFINITY));
    // A level beyond the last is refused, as is a level that bounds a later one without
    // both tolerances.
    assert_eq!(
        refused(LEXICOGRAPHIC, Some(3)),
        ObjectiveRefusal::UnknownLevel
    );
    assert_eq!(
        refused(
            &LEXICOGRAPHIC.replace(", absolute_tolerance = 0.5", ""),
            Some(1)
        ),
        ObjectiveRefusal::MissingTolerance
    );
}

#[test]
fn competing_objectives_without_priority_refused() {
    let source = |a: &str, b: &str| {
        format!(
            "package p {{ def Root {{ var x: Scalar; var y: Scalar; let a: Scalar = x*x; let b: Scalar = y*y; annotation objective a(minimize{a}); annotation objective b(minimize{b}); }} }}"
        )
    };
    for (a, b, reason) in [
        ("", "", ObjectiveRefusal::CompetingWithoutPriority),
        (
            ", priority = 1",
            "",
            ObjectiveRefusal::CompetingWithoutPriority,
        ),
        (
            ", weight = 1",
            "",
            ObjectiveRefusal::CompetingWithoutPriority,
        ),
        (
            ", priority = 0, weight = 1",
            ", priority = 0",
            ObjectiveRefusal::MissingWeight,
        ),
    ] {
        assert_eq!(refused(&source(a, b), None), reason, "{a} / {b}");
    }
    // Every member with a priority, or one shared level with weights, is admitted.
    for (a, b, levels) in [
        (", weight = 1", ", weight = 3", 1),
        (
            ", priority = 4, weight = 1",
            ", priority = 4, weight = 3",
            1,
        ),
    ] {
        let (mut workspace, root) = setup(&source(a, b));
        let model = prepare(&mut workspace, root, None).unwrap();
        assert_eq!(model.model.objectives.levels.len(), levels);
        let mut weights = model
            .model
            .objectives
            .members
            .iter()
            .map(|m| m.weight)
            .collect::<Vec<_>>();
        weights.sort_by(f64::total_cmp);
        assert_eq!(weights, vec![1.0, 3.0]);
        // The weighted sum a + 3·b at x = 2, y = 1.
        let point = [(symbol(&model, ".x"), 2.0), (symbol(&model, ".y"), 1.0)];
        assert!((evaluate(&workspace, &model, &point).0 - 7.0).abs() < 1e-12);
    }
    let (mut workspace, root) = setup(&source(", priority = 0", ", priority = 1"));
    assert!(prepare(&mut workspace, root, Some(0)).is_err());
    let (mut workspace, root) = setup(&source(
        ", priority = 0, absolute_tolerance = 0.1, relative_tolerance = 0",
        ", priority = 1",
    ));
    assert!(prepare(&mut workspace, root, Some(1)).is_ok());
}

#[test]
fn dimensional_weighted_sum_refused() {
    let source = |second: &str| {
        format!(
            "package p {{ def Root {{ var x: Scalar; var w: Scalar; var z: Power; let a: Scalar = x*x; let b: Power = z; let c: Scalar = w; eq cap: z <= 1{{W}}; annotation objective a(minimize, weight = 0.25); annotation objective {second}; }} }}"
        )
    };
    // A dimensional member of a weighted sum without a normalization (PS-01).
    assert_eq!(
        refused(&source("b(maximize, weight = 0.5)"), None),
        ObjectiveRefusal::DimensionalSum
    );
    // A normalization divides the member through the package's quantity operations; a
    // kind without a declared ratio to itself cannot be normalized, and nothing invents one.
    let (mut workspace, root) = setup(&source("b(maximize, weight = 0.5, normalization = 2{W})"));
    let error = prepare(&mut workspace, root, None).unwrap_err().to_string();
    assert!(
        error.contains("no unique registered quantity operation"),
        "{error}"
    );
    // Normalized and weighted, the level minimizes 0.25·a − 0.5·c/2: the maximized
    // member enters with a negative scale.
    let (mut workspace, root) = setup(&source("c(maximize, weight = 0.5, normalization = 2)"));
    let model = prepare(&mut workspace, root, None).unwrap();
    let neutral = workspace.inputs.quantities.neutral_dimensionless().unwrap();
    let level = &model.model.objectives.levels[0];
    assert_eq!((level.sense, level.quantity), (Sense::Minimize, neutral));
    let c = model
        .model
        .objectives
        .members
        .iter()
        .find(|m| m.target == symbol(&model, ".c"))
        .unwrap();
    assert_eq!(c.scale, -0.5);
    assert_ne!(c.term, c.target);
    let normalization = &model.model.symbols[&c.normalization.unwrap()];
    assert!(matches!(
        normalization.initial,
        Some(pse_modeling::specialize::Value::Number { bits, .. }) if f64::from_bits(bits) == 2.0
    ));
    let point = [
        (symbol(&model, ".x"), 2.0),
        (symbol(&model, ".w"), 6.0),
        (symbol(&model, ".z"), 0.5),
    ];
    assert!((evaluate(&workspace, &model, &point).0 - (1.0 - 1.5)).abs() < 1e-12);
    // Invalid weights and normalizations are refused.
    assert_eq!(
        refused(
            &source("c(maximize, weight = 0.5, normalization = 0)"),
            None
        ),
        ObjectiveRefusal::InvalidNormalization
    );
    assert_eq!(
        refused(&source("c(maximize, weight = -1)"), None),
        ObjectiveRefusal::InvalidWeight
    );
    // A single dimensional objective needs no normalization; its level keeps its type.
    let (mut workspace, root) = setup(
        "package p { def Root { var z: Power; let b: Power = z; eq cap: z <= 1{W}; annotation objective b(maximize); } }",
    );
    let model = prepare(&mut workspace, root, None).unwrap();
    assert_eq!(model.model.objectives.members[0].term, symbol(&model, ".b"));
    assert_eq!(model.admitted.case.objective().unwrap().quantity, power());
}

#[test]
fn zero_tolerance_refused_on_staged_level() {
    let source = |relative: &str| {
        format!(
            "package p {{ def Root {{ var x: Scalar; var y: Scalar; let a: Scalar = x*x; let b: Scalar = y*y; annotation objective a(minimize, priority = 0, absolute_tolerance = 0, relative_tolerance = {relative}); annotation objective b(minimize, priority = 1); }} }}"
        )
    };
    // Staging level 1 bounds level 0 with a zero tolerance: native LP and MILP only.
    let reason = refused(&source("0"), Some(1));
    assert_eq!(reason, ObjectiveRefusal::ZeroTolerance);
    assert!(reason.is_unsupported());
    // The declared zero tolerance is recorded on its level for the native route.
    let (mut workspace, root) = setup(&source("0"));
    let model = prepare(&mut workspace, root, Some(0)).unwrap();
    let level = &model.model.objectives.levels[0];
    assert_eq!(level.relative_tolerance, Some(0.0));
    assert_eq!(level.absolute_tolerance, Some(0.0));
    // A positive relative tolerance admits the staged level; whether max(abs, rel·|f*|)
    // is positive at the optimum is the engine's check.
    let (mut workspace, root) = setup(&source("0.01"));
    assert!(prepare(&mut workspace, root, Some(1)).is_ok());
    // Negative tolerances are refused wherever they are declared.
    assert_eq!(
        refused(&source("-0.01"), None),
        ObjectiveRefusal::InvalidTolerance
    );
}
