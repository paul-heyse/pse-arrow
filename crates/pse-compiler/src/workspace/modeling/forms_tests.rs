// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Constraint-form and disjunction lowerings at preparation (ADR-0104).
use super::domain_tests::setup;
use super::*;
use pse_math::jets::EvaluationLimits;
use pse_model::{forms::NativeConstraint, generated::enums::NativeConstraintForm};
use pse_modeling::{ModelingError, RealizationRefusal};

struct Case {
    model: PreparedModeling,
    prepared: PreparedCase,
    values: CaseValues,
}
impl Case {
    fn new(
        workspace: &mut CompilerWorkspace,
        root: DeclarationId,
        bounds: &[(&str, Option<f64>, Option<f64>)],
    ) -> Result<Self> {
        let case = ModelingCaseBindings {
            values: BTreeMap::new(),
            variables: bounds
                .iter()
                .map(|(path, lower, upper)| {
                    (
                        (*path).to_owned(),
                        ModelingVariableState {
                            fixed: None,
                            lower: Some(*lower),
                            upper: Some(*upper),
                        },
                    )
                })
                .collect(),
        };
        let (model, prepared, values, _) = workspace.prepare_modeling_case_cancellable(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
            &case,
            DerivativeOrder::First,
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )?;
        Ok(Self {
            model,
            prepared,
            values,
        })
    }
    /// The specialized symbol at a demanded case path, or whose authored path ends with
    /// `suffix`.
    fn symbol(&self, suffix: &str) -> SemanticId {
        if let Some(id) = self.model.model.paths.get(suffix) {
            return *id;
        }
        let found = self
            .model
            .model
            .symbols
            .values()
            .filter(|s| s.lineage.path.ends_with(suffix))
            .map(|s| s.id)
            .collect::<Vec<_>>();
        assert_eq!(found.len(), 1, "{suffix}: {found:?}");
        found[0]
    }
    /// Do the lowered rows hold at this complete point (free values by path suffix)?
    fn feasible(&self, point: &[(&str, f64)]) -> bool {
        let mut values = self.values.clone();
        for v in self.prepared.plan.structure().variables() {
            values.scalars.entry(v.port.id).or_insert(0.0);
        }
        for (suffix, value) in point {
            values.scalars.insert(self.symbol(suffix), *value);
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let assembly = Arc::new(
            self.prepared
                .plan
                .compile(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let rows = assembly
            .worker(BTreeMap::new(), cancel)
            .constraints(&values)
            .unwrap();
        self.prepared
            .plan
            .structure()
            .rows()
            .iter()
            .zip(rows)
            .all(|(row, value)| value >= row.lower - 1e-9 && value <= row.upper + 1e-9)
    }
}
fn refusal(error: CompileError) -> (String, RealizationRefusal) {
    match error {
        CompileError::Modeling(ModelingError::Realization {
            subject, reason, ..
        }) => (subject, reason),
        other => panic!("expected a typed realization refusal, got {other:?}"),
    }
}
const GDP: &str = "package p { def Root { var x: Power; var y: Power; eq total: y <= 30{W}; disjunction route { alternative low { eq cap: x <= 2{W}; } alternative high { eq floor: x >= 5{W}; eq link: y == 2*x; } } realize how on route using REALIZATION; } }";

#[test]
fn bigm_derived_from_bounds() {
    let (mut w, root) = setup(&GDP.replace("REALIZATION", "bigm(derived)"));
    let box_ = [("x", Some(0.0), Some(10.0)), ("y", Some(0.0), Some(30.0))];
    let case = Case::new(&mut w, root, &box_).unwrap();
    // sup(x − 2) = 8, inf(x − 5) = −5, y − 2x ∈ [−20, 30]; widened outward by 1e-6.
    for (suffix, expected) in [
        ("cap.big_m_upper", 8.0_f64),
        ("floor.big_m_lower", -5.0),
        ("link.big_m_upper", 30.0),
        ("link.big_m_lower", -20.0),
    ] {
        let m = case.prepared.derived.values[&case.symbol(suffix)];
        assert!(m.abs() >= expected.abs(), "{suffix}: {m}");
        assert!((m - expected * (1.0 + 1e-6)).abs() < 1e-9, "{suffix}: {m}");
        assert_eq!(case.values.scalars[&case.symbol(suffix)], m);
    }
    let lowering = case
        .model
        .model
        .lowerings
        .iter()
        .find(|l| {
            l.realization == pse_model::generated::enums::ModelingRealizationPolicy::DerivedBigM
        })
        .unwrap();
    assert_eq!(
        lowering.equivalence,
        pse_modeling::specialize::Equivalence::Exact
    );
    // The lowered rows admit every point of the selected alternative and nothing else.
    for x in [0.0_f64, 1.0, 2.0, 3.0, 5.0, 7.5, 10.0] {
        for y in [0.0_f64, 10.0, 15.0, 20.0, 30.0] {
            let low = x <= 2.0;
            let high = x >= 5.0 && (y - 2.0 * x).abs() < 1e-12;
            for (select, holds) in [((1.0, 0.0), low), ((0.0, 1.0), high)] {
                let point = [
                    ("Root.x", x),
                    ("Root.y", y),
                    ("route.low", select.0),
                    ("route.high", select.1),
                ];
                assert_eq!(
                    case.feasible(&point),
                    holds,
                    "x={x} y={y} select={select:?}"
                );
            }
        }
    }
    // An unbounded box leaves no finite enclosure: the refusal names the row.
    let error = Case::new(
        &mut w,
        root,
        &[("x", Some(0.0), None), ("y", Some(0.0), Some(30.0))],
    )
    .err()
    .unwrap();
    let (subject, reason) = refusal(error);
    assert_eq!(reason, RealizationRefusal::UnboundedInterval);
    assert!(
        subject.contains("cap") || subject.contains("link"),
        "{subject}"
    );
}

#[test]
fn derived_parameters_rebind_with_the_values_they_consume() {
    let (mut w, root) = setup(
        "package p { def Root { param p: Power = 2{W}; var x: Power; disjunction route { alternative low { eq cap: x <= p; } alternative high { eq floor: x >= 5{W}; } } realize how on route using bigm(derived); } }",
    );
    let case = Case::new(&mut w, root, &[("x", Some(0.0), Some(10.0))]).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let (p, x) = (case.symbol("Root.p"), case.symbol("Root.x"));
    let (cap, floor) = (
        case.symbol("cap.big_m_upper"),
        case.symbol("floor.big_m_lower"),
    );
    let first = &case.prepared;
    // sup(x − p) = 8 at p = 2 over x ∈ [0, 10], widened outward.
    assert!((first.derived.values[&cap] - 8.0 * (1.0 + 1e-6)).abs() < 1e-9);
    let with = |id: SemanticId, value: f64| {
        let mut values = case.values.clone();
        values.scalars.insert(id, value);
        values
    };
    // A free start is consumed by nothing: the derivation and every product are shared.
    let start = with(x, 3.0);
    assert!(first.values_match(&start));
    let shared = first.rebind(&start, &cancel).unwrap();
    assert_eq!(shared.derived, first.derived);
    assert!(Arc::ptr_eq(&shared.presolve, &first.presolve));
    // The parameter the enclosure consumed recomputes the derived M; the stale derived
    // entries in the caller's values are replaced, and the structure stays shared.
    let changed = with(p, 4.0);
    assert!(!first.values_match(&changed));
    let rebound = first.rebind(&changed, &cancel).unwrap();
    assert!((rebound.derived.values[&cap] - 6.0 * (1.0 + 1e-6)).abs() < 1e-9);
    assert_eq!(rebound.derived.values[&floor], first.derived.values[&floor]);
    assert!(Arc::ptr_eq(&rebound.derivation, &first.derivation));
    assert!(Arc::ptr_eq(&rebound.plan, &first.plan));
    assert!(!Arc::ptr_eq(&rebound.presolve, &first.presolve));
    assert!(rebound.values_match(&changed));
    assert_eq!(
        rebound.complete(&changed).scalars[&cap],
        rebound.derived.values[&cap]
    );
    // A fresh preparation at the changed value agrees with the rebind.
    let structure = case
        .model
        .bound_structure(&BTreeMap::from([(
            x,
            ModelingVariableState {
                fixed: None,
                lower: Some(Some(0.0)),
                upper: Some(Some(10.0)),
            },
        )]))
        .unwrap()
        .structure;
    let fresh = w
        .prepare_modeling_view(
            &case.model,
            structure,
            &changed,
            DerivativeOrder::First,
            Profile::default(),
            &cancel,
        )
        .unwrap();
    assert_eq!(fresh.derived, rebound.derived);
    assert_eq!(fresh.presolve.values, rebound.presolve.values);
    assert_eq!(fresh.coefficient_values, rebound.coefficient_values);
    assert_eq!(fresh.facts, rebound.facts);
}

#[test]
fn hull_requires_finite_bounds() {
    let (mut w, root) = setup(&GDP.replace("REALIZATION", "hull"));
    let error = Case::new(
        &mut w,
        root,
        &[("x", Some(0.0), Some(10.0)), ("y", Some(0.0), None)],
    )
    .err()
    .unwrap();
    let (subject, reason) = refusal(error);
    assert_eq!(reason, RealizationRefusal::InfiniteBound);
    assert!(subject.ends_with(".y"), "{subject}");
    let case = Case::new(
        &mut w,
        root,
        &[("x", Some(0.0), Some(10.0)), ("y", Some(0.0), Some(30.0))],
    )
    .unwrap();
    let low = case.symbol("route.low");
    assert!(case.prepared.plan.structure().native().is_empty());
    // Selected alternatives carry the disaggregated copies; the other copies vanish.
    let copies = |x: f64, y: f64, low: bool| {
        let (l, h) = if low { (1.0, 0.0) } else { (0.0, 1.0) };
        [
            ("Root.x", x),
            ("Root.y", y),
            ("route.low", l),
            ("route.high", h),
            ("x.hull_0", x * l),
            ("x.hull_1", x * h),
            ("y.hull_0", y * l),
            ("y.hull_1", y * h),
        ]
    };
    assert!(case.feasible(&copies(1.5, 7.0, true)));
    assert!(case.feasible(&copies(6.0, 12.0, false)));
    assert!(!case.feasible(&copies(3.0, 6.0, true)));
    assert!(!case.feasible(&copies(6.0, 13.0, false)));
    assert!(
        case.model.model.symbols[&low].domain
            == pse_model::generated::enums::ModelingVariableDomain::Binary
    );
}

#[test]
fn logic_propositions_lower_exactly() {
    type Truth = fn(bool, bool, bool) -> bool;
    let propositions: [(&str, Truth); 7] = [
        ("a implies (b or not c)", |a, b, c| !a || b || !c),
        ("(a and b) or c", |a, b, c| a && b || c),
        ("a xor (b and c)", |a, b, c| a != (b && c)),
        ("not (a or b) or c", |a, b, c| !(a || b) || c),
        ("exactly(2, a, b, c)", |a, b, c| {
            u8::from(a) + u8::from(b) + u8::from(c) == 2
        }),
        ("a and not b", |a, b, _| a && !b),
        ("(a implies b) and (b implies c)", |a, b, c| {
            (!a || b) && (!b || c)
        }),
    ];
    for (proposition, truth) in propositions {
        let (mut w, root) = setup(&format!(
            "package p {{ def Root {{ var a: Indicator in binary; var b: Indicator in binary; var c: Indicator in binary; logic rule: {proposition}; }} }}"
        ));
        let case = Case::new(&mut w, root, &[]).unwrap();
        let resultants = case
            .model
            .model
            .symbols
            .values()
            .filter(|s| s.lineage.path.contains(".rule.resultant_"))
            .map(|s| s.lineage.path.rsplit_once(".rule.").unwrap().1.to_owned())
            .collect::<Vec<_>>();
        for assignment in 0u8..8 {
            let (a, b, c) = (
                assignment & 1 != 0,
                assignment & 2 != 0,
                assignment & 4 != 0,
            );
            let base = [
                ("Root.a", f64::from(u8::from(a))),
                ("Root.b", f64::from(u8::from(b))),
                ("Root.c", f64::from(u8::from(c))),
            ];
            // The lowering is exact iff some binary resultant assignment is feasible
            // exactly when the proposition holds.
            let satisfiable = (0u32..1 << resultants.len()).any(|aux| {
                let mut point = base.to_vec();
                let names = resultants
                    .iter()
                    .map(|r| format!("rule.{r}"))
                    .collect::<Vec<_>>();
                for (k, name) in names.iter().enumerate() {
                    point.push((name.as_str(), f64::from(aux >> k & 1)));
                }
                case.feasible(&point)
            });
            assert_eq!(satisfiable, truth(a, b, c), "{proposition} at {a} {b} {c}");
        }
    }
    // The native realization keeps and/or resultants for a constraint handler.
    let (mut w, root) = setup(
        "package p { def Root { var a: Indicator in binary; var b: Indicator in binary; var c: Indicator in binary; logic rule: a or (b and not c); realize r on rule using native; } }",
    );
    let case = Case::new(&mut w, root, &[]).unwrap();
    let forms = case
        .prepared
        .plan
        .structure()
        .native()
        .iter()
        .map(NativeConstraint::form)
        .collect::<Vec<_>>();
    assert_eq!(
        forms,
        vec![NativeConstraintForm::And, NativeConstraintForm::Or]
    );
    assert_eq!(
        case.prepared.facts.native,
        vec![NativeConstraintForm::And, NativeConstraintForm::Or]
    );
}

#[test]
fn hull_perspective_for_nonlinear_disjuncts() {
    let text = "package p { def Root { var x: Scalar; var y: Scalar; disjunction route { alternative low { eq cap: x <= 2; } alternative curved { eq square: y*y == x*x; } } realize how on route using REALIZATION; } }";
    let box_ = [("x", Some(0.0), Some(10.0)), ("y", Some(0.0), Some(10.0))];
    // Plain hull refuses the nonlinear row, naming it.
    let (mut w, root) = setup(&text.replace("REALIZATION", "hull"));
    let error = w
        .admit_modeling(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        )
        .err()
        .unwrap();
    let (subject, reason) = refusal(error);
    assert_eq!(reason, RealizationRefusal::Nonlinear);
    assert!(subject.ends_with("square"), "{subject}");
    // hull(epsilon) states its O(epsilon) perspective approximation.
    let (mut w, root) = setup(&text.replace("REALIZATION", "hull(0.001)"));
    let case = Case::new(&mut w, root, &box_).unwrap();
    let lowering = case.model.model.lowerings.last().unwrap();
    assert_eq!(
        lowering.equivalence,
        pse_modeling::specialize::Equivalence::Perspective { epsilon: 0.001 }
    );
    // The selected curved alternative with its copies at x = y = 3 holds (y+ε)·g(v/(y+ε)).
    let point = |copy: f64| {
        [
            ("Root.x", 3.0),
            ("Root.y", copy),
            ("route.low", 0.0),
            ("route.curved", 1.0),
            ("x.hull_0", 0.0),
            ("x.hull_1", 3.0),
            ("y.hull_0", 0.0),
            ("y.hull_1", copy),
        ]
    };
    assert!(case.feasible(&point(3.0)));
    assert!(!case.feasible(&point(5.0)));
}

#[test]
fn cardinality_and_ordered_sets_lower_exactly() {
    let (mut w, root) = setup(
        "package p { entity kind k {} entity k a {} entity k b {} entity k c {} set ks: Set<k> = {a, b, c}; def Root { var y[i in ks]: Indicator in binary; atmost two[i in ks]: 2 of y[i]; } }",
    );
    let unbounded = [
        ("y[a]", None, None),
        ("y[b]", None, None),
        ("y[c]", None, None),
    ];
    let case = Case::new(&mut w, root, &unbounded).unwrap();
    for assignment in 0u8..8 {
        let bit = |k: u8| f64::from(u8::from(assignment & k != 0));
        let point = [("y[a]", bit(1)), ("y[b]", bit(2)), ("y[c]", bit(4))];
        assert_eq!(
            case.feasible(&point),
            assignment.count_ones() <= 2,
            "{assignment}"
        );
    }
    // SOS1 over bounded flows: one nonzero member, selected by its binary.
    let (mut w, root) = setup(
        "package p { entity kind source provenance { attribute title: Text; } enum role { given } entity source s { title = \"synthetic test data\" } entity kind k {} entity k a {} entity k b {} set ks: Set<k> = {a, b}; table rank[i: k]: Scalar complete_over(i in ks); dataset r: rank provenance(s, role.given) { [a] = [1]; [b] = [2]; } def Root { var f[i in ks]: Power; sos1 one[i in ks]: f[i] weight rank[i]; } }",
    );
    let bounds = [
        ("f[a]", Some(0.0), Some(4.0)),
        ("f[b]", Some(0.0), Some(4.0)),
    ];
    let case = Case::new(&mut w, root, &bounds).unwrap();
    let check = |fa: f64, fb: f64, za: f64, zb: f64| {
        case.feasible(&[
            ("f[a]", fa),
            ("f[b]", fb),
            ("one.selector_0", za),
            ("one.selector_1", zb),
        ])
    };
    assert!(check(3.0, 0.0, 1.0, 0.0));
    assert!(check(0.0, 2.0, 0.0, 1.0));
    assert!(!check(3.0, 2.0, 1.0, 1.0));
    assert!(!check(3.0, 2.0, 1.0, 0.0));
}

#[test]
fn nested_disjunctions_lower_inner_first() {
    let (mut w, root) = setup(
        "package p { def Root { var x: Power; disjunction route { alternative off { eq zero: x <= 0{W}; } alternative on { disjunction mode { alternative slow { eq band: x <= 3{W}; } alternative fast { eq band: x >= 6{W}; } } } } realize outer on route using bigm(derived); realize inner on route.on.mode using bigm(derived); logic never_fast: not route.on.mode.fast; } }",
    );
    let case = Case::new(&mut w, root, &[("x", Some(0.0), Some(10.0))]).unwrap();
    let model = &case.model.model;
    let position = |row_name: &str| {
        model
            .lowerings
            .iter()
            .position(|l| {
                l.rows.iter().any(|r| {
                    model
                        .equations
                        .iter()
                        .any(|e| e.id == *r && e.lineage.path.ends_with(row_name))
                })
            })
            .unwrap()
    };
    assert!(position("band") < position("zero"));
    // The inner selection follows its owner; the logic forbids the fast mode.
    let point = |x: f64, off: f64, on: f64, slow: f64, fast: f64| {
        case.feasible(&[
            ("Root.x", x),
            ("Root.route.off", off),
            ("Root.route.on", on),
            ("route.on.mode.slow", slow),
            ("route.on.mode.fast", fast),
        ])
    };
    assert!(point(0.0, 1.0, 0.0, 0.0, 0.0));
    assert!(point(2.0, 0.0, 1.0, 1.0, 0.0));
    assert!(!point(2.0, 1.0, 0.0, 1.0, 0.0));
    assert!(!point(0.0, 0.0, 1.0, 0.0, 0.0));
    assert!(!point(7.0, 0.0, 1.0, 0.0, 1.0));
    assert!(!point(4.0, 0.0, 1.0, 1.0, 0.0));
}

#[test]
fn form_realizations_are_declared_and_admitted() {
    for (text, message) in [
        (
            "package p { def Root { var x: Power; disjunction route { alternative a { eq e: x <= 1{W}; } alternative b { eq e: x >= 2{W}; } } } }",
            "a disjunction requires a declared realization",
        ),
        (
            "package p { def Root { var x: Power; disjunction route { alternative a { eq e: x <= 1{W}; } alternative b { eq e: x >= 2{W}; } } realize r on route using linear; } }",
            "linear realization does not apply to this form",
        ),
        (
            "package p { def Root { var x: Power; var y: Indicator; eq e when y: x <= 1{W}; } }",
            "must name a binary variable",
        ),
    ] {
        let (mut w, root) = setup(text);
        let error = w
            .admit_modeling(
                root,
                InstanceId::from_id(SemanticId::NIL),
                Bindings::default(),
                Limits::default(),
            )
            .err()
            .unwrap();
        assert!(error.to_string().contains(message), "{error}");
    }
}

/// The authored CHKS smoothing function the reference `math` package declares.
const SMOOTH_MIN: &str = "fn smooth_min<Q>(a: Q, b: Q, eps: Delta<Q>) -> Q valid(eps > eps - eps) = a - eps*(d + sqrt(d*d + 1))/2 where d = (a - b)/eps;";
fn complementarity(realization: &str) -> String {
    format!(
        "package p {{ {SMOOTH_MIN} def Root {{ param eps: Scalar = 0.1; var a: Scalar; var b: Scalar; complements c: (a >= 0, b >= 0); {realization} }} }}"
    )
}

#[test]
fn complementarity_lowerings_keep_members_nonnegative() {
    use pse_model::generated::enums::{
        ModelingRealizationPolicy as Policy, ModelingStructuralRequirement as Requirement,
    };
    use pse_modeling::specialize::Equivalence;
    let lowered = |realization: &str| {
        let (mut w, root) = setup(&complementarity(realization));
        let case = Case::new(&mut w, root, &[]).unwrap();
        let lowering = case.model.model.lowerings[0].clone();
        (case, lowering)
    };
    // smooth(f, width): the one row f(a, b, width) == 0, whose zero set keeps both
    // members positive.
    let (smooth, lowering) = lowered("realize r on c using smooth(smooth_min, eps);");
    assert_eq!(
        (lowering.realization, lowering.equivalence),
        (Policy::Smooth, Equivalence::Smoothed)
    );
    assert_eq!(lowering.rows.len(), 1);
    assert!(smooth.prepared.plan.structure().requirements().is_empty());
    // The width stays a parameter: continuing it rebinds a value, not the structure.
    assert!(
        smooth
            .model
            .admitted
            .inputs
            .contains(&smooth.symbol(".eps"))
    );
    // disjunctive: nonnegative slack columns equal to the members, one native SOS1.
    let (disjunctive, lowering) = lowered("realize r on c using disjunctive;");
    assert_eq!(
        (lowering.realization, lowering.equivalence),
        (Policy::Disjunctive, Equivalence::Native)
    );
    assert_eq!(lowering.variables.len(), 2);
    let structure = disjunctive.prepared.plan.structure();
    for slack in &lowering.variables {
        let variable = structure
            .variables()
            .iter()
            .find(|v| v.port.id == *slack)
            .unwrap();
        assert_eq!((variable.lower, variable.upper), (Some(0.0), None));
    }
    assert!(matches!(
        structure.native(),
        [NativeConstraint::Sos { form: NativeConstraintForm::Sos1, members }]
            if members.iter().map(|(id, _)| *id).collect::<Vec<_>>() == lowering.variables
    ));
    // The slack columns equal the members; their lower bounds, asserted above, keep the
    // members nonnegative.
    for (a, b) in [(0.0, 2.0), (-1.0, 2.0)] {
        let point = [
            ("Root.a", a),
            ("Root.b", b),
            (".slack_first", a),
            (".slack_second", b),
        ];
        assert!(disjunctive.feasible(&point), "{point:?}");
        assert!(!disjunctive.feasible(&[
            ("Root.a", a),
            ("Root.b", b),
            (".slack_first", a + 1.0),
            (".slack_second", b),
        ]));
    }
    // penalty(l1): nonnegative members, a·b <= 0, and the l1 exact-penalty requirement,
    // which is structure.
    let (penalty, lowering) = lowered("realize r on c using penalty(l1);");
    assert_eq!(
        (lowering.realization, lowering.equivalence),
        (Policy::PenaltyL1, Equivalence::ExactPenalty)
    );
    let structure = penalty.prepared.plan.structure();
    assert_eq!(structure.requirements(), [Requirement::L1ExactPenalty]);
    for (a, b, holds) in [(0.0, 3.0, true), (2.0, 0.0, true), (1.0, 1.0, false)] {
        assert_eq!(
            penalty.feasible(&[("Root.a", a), ("Root.b", b)]),
            holds,
            "a={a} b={b}"
        );
    }
    assert_ne!(
        structure.key(),
        smooth.prepared.plan.structure().key(),
        "the requirement is structure"
    );
    // No default realization; a realization of another form does not apply.
    for (realization, message) in [
        ("", "a complementarity requires a declared realization"),
        (
            "realize r on c using hull;",
            "hull realization does not apply to this form",
        ),
        ("realize r on c using smooth(smooth_min, 0.1{W});", ""),
    ] {
        let (mut w, root) = setup(&complementarity(realization));
        let error = Case::new(&mut w, root, &[]).err().unwrap();
        assert!(error.to_string().contains(message), "{error}");
    }
}

#[test]
fn smooth_complementarity_row_vanishes_on_eps_sq_over_4() {
    // CHKS: f(a, b, eps) = (a + b − sqrt((a − b)² + eps²))/2 vanishes exactly where
    // a·b = eps²/4 with a, b > 0.
    let (mut w, root) = setup(&complementarity(
        "realize r on c using smooth(smooth_min, eps);",
    ));
    let mut case = Case::new(&mut w, root, &[]).unwrap();
    for eps in [0.1_f64, 0.01] {
        // The width is a value: rebinding it keeps the prepared structure.
        let width = case.symbol(".eps");
        case.values.scalars.insert(width, eps);
        for a in [eps / 2.0, 0.3, 2.0] {
            let b = eps * eps / (4.0 * a);
            assert!(
                case.feasible(&[("Root.a", a), ("Root.b", b)]),
                "eps={eps} a={a}"
            );
            assert!(
                !case.feasible(&[("Root.a", a), ("Root.b", 2.0 * b)]),
                "eps={eps} a={a}"
            );
        }
    }
}
