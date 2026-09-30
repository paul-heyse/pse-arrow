// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed validity envelopes (ADR-0123 Outcome 4, Plan 23 KR7).
use crate::annotation::AnnotationValue;
use crate::envelope::Envelope;
use crate::kernel_types::{physical, try_source};
use crate::*;
use pse_authoring::dsl;
use pse_model::generated::enums::{ExtrapolationPolicy, ModelingValidityLayer};

fn admitted(text: &str) -> Result<CheckedPackage> {
    let (registry, _) = physical();
    let context = TypeContext {
        preconditions: &pse_quantity::PhysicalPreconditions::new(vec![]).unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    check(&try_source(text)?, &context)
}
fn refusal(text: &str) -> String {
    match admitted(text) {
        Ok(_) => panic!("admitted: {text}"),
        Err(error) => error.to_string(),
    }
}
fn root(text: &str, name: &str) -> Result<SpecializedModel> {
    let p = admitted(text)?;
    specialize(
        &p,
        p.names[&format!("p.{name}")],
        InstanceId::from_bytes([7; 16]),
        &Bindings::default(),
        Limits::default(),
    )
}
fn temperature() -> Type {
    let (_, names) = physical();
    Type::Quantity(pse_quantity::scheme::Scheme::Concrete(names["Temperature"]))
}
/// The magnitude of a specialized constant: a zero-argument call of a constant function.
fn constant(model: &SpecializedModel, expression: &dsl::Expr) -> f64 {
    let dsl::ExprKind::NamedCall { name, args } = &expression.kind else {
        panic!("constant call: {expression:?}");
    };
    assert!(args.is_empty());
    match &model.functions[name].body.as_ref().unwrap().kind {
        dsl::ExprKind::Number(number) => number.value,
        other => panic!("constant body: {other:?}"),
    }
}
/// The data-layer observations of a specialized model: target path, envelope owner,
/// bounds and selection.
fn observations(
    model: &SpecializedModel,
) -> Vec<(String, DeclarationId, f64, f64, Option<DeclarationId>)> {
    model
        .annotations
        .iter()
        .filter_map(|a| match &a.value {
            AnnotationValue::Valid {
                lower,
                upper,
                policy,
                layer: ModelingValidityLayer::Data,
                selection,
            } => {
                assert_eq!(*policy, ExtrapolationPolicy::Extrapolate);
                Some((
                    model.symbols[&a.target].lineage.path.clone(),
                    a.lineage.declaration,
                    constant(model, lower),
                    constant(model, upper),
                    *selection,
                ))
            }
            _ => None,
        })
        .collect()
}

/// A bank of two rows whose temperature envelope is declared as data over two typed
/// columns, a form guarding it at one argument and an increment guarding its interval.
const BANK: &str = r#"
 entity kind item {} entity item a {} entity item b {}
 set items: Set<item> = {a, b};
 table cp_data[j: item]: {c: MolarCp, low: Temperature, high: Temperature} envelope T: Temperature in low..high complete_over(j in items);
 dataset bank: cp_data provenance(s, role.given) { [a] = [75{J/(mol*K)}, 250{K}, 400{K}]; [b] = [80{J/(mol*K)}, 300{K}, 500{K}]; }
 fn cp(T: Temperature, p: Row<cp_data>) -> MolarCp guards(p.T: T) valid(T > 0{K}) = p.c;
 fn dh(T0: Temperature, T: Temperature, p: Row<cp_data>) -> MolarCp guards(p.T: [T0, T]) = p.c;"#;
/// A keyed kind declaring an envelope over two of its attributes, a refinement inheriting
/// it, and a form bound through an entity argument.
const KIND: &str = r#"
 entity kind band { key subject: item; attribute c: MolarCp; attribute low: Temperature; attribute high: Temperature; envelope T: Temperature in low..high; }
 entity kind narrow extends band {}
 dataset bands: narrow provenance(s, role.given) { [a] = [75{J/(mol*K)}, 250{K}, 400{K}]; }
 fn band_cp(T: Temperature, s: band) -> MolarCp guards(s.T: T) = s.c;"#;

/// A relation or kind declares an envelope as data; a function guarding it has a generated
/// domain predicate beside its own form-layer domain, and each specialization carries the
/// guard. Nothing is read from column names by convention.
#[test]
fn relation_envelope_generates_function_guard() {
    let text = format!(
        "package p {{ {BANK} {KIND}
 def Root {{ var T: Temperature; var T0: Temperature; var x: MolarCp; eq e: x == cp(T, cp_data[a]) + dh(T0, T, cp_data[b]) + band_cp(T, band[a]); }}
}}"
    );
    let p = admitted(&text).unwrap();
    assert_eq!(
        p.tables[&p.names["p.cp_data"]].envelopes,
        [Envelope {
            owner: p.names["p.cp_data"],
            axis: "T".into(),
            ty: temperature(),
            lower: "low".into(),
            upper: "high".into(),
        }]
    );
    let guard = |function: &str| {
        let f = &p.functions[&p.names[function]];
        assert_eq!(f.envelopes.len(), 1, "{function}");
        let guard = &f.envelopes[0];
        assert_eq!(guard.policy, ExtrapolationPolicy::Reject);
        assert_eq!(guard.selection, None);
        (
            guard.envelope.owner,
            dsl::render_predicate(&guard.predicate),
        )
    };
    // The form layer stays the function's own domain; the data layer is generated.
    assert!(p.functions[&p.names["p.cp"]].validity.is_some());
    assert_eq!(
        guard("p.cp"),
        (
            p.names["p.cp_data"],
            "((T >= p.low) and (T <= p.high))".to_owned()
        )
    );
    // An increment guards its whole integration interval: both endpoints.
    assert_eq!(
        guard("p.dh"),
        (
            p.names["p.cp_data"],
            "(((T0 >= p.low) and (T0 <= p.high)) and ((T >= p.low) and (T <= p.high)))".to_owned()
        )
    );
    // A kind's envelope is its declaration; a refinement inherits it.
    assert_eq!(
        guard("p.band_cp"),
        (
            p.names["p.band.T"],
            "((T >= s.low) and (T <= s.high))".to_owned()
        )
    );
    assert_eq!(
        p.kinds[&p.names["p.narrow"]].envelopes,
        p.kinds[&p.names["p.band"]].envelopes
    );
    // Each specialization carries its guard, rejecting, with the row's bounds.
    let model = root(&text, "Root").unwrap();
    let guards = model
        .functions
        .values()
        .flat_map(|f| &f.envelopes)
        .collect::<Vec<_>>();
    assert_eq!(guards.len(), 3);
    assert!(
        guards
            .iter()
            .all(|g| g.policy == ExtrapolationPolicy::Reject && g.selection.is_none())
    );
    assert!(observations(&model).is_empty() && model.observations.is_empty());
    // Columns named minimum and maximum are ordinary columns: no envelope, no guard.
    let conventional = r#"package p { entity kind item {} entity item a {} set items: Set<item> = {a};
 table fit[j: item]: {c: MolarCp, minimum: Temperature, maximum: Temperature} complete_over(j in items);
 dataset d: fit provenance(s, role.given) { [a] = [75{J/(mol*K)}, 250{K}, 400{K}]; }
 fn cp(T: Temperature, p: Row<fit>) -> MolarCp = p.c; }"#;
    let p = admitted(conventional).unwrap();
    assert!(p.tables[&p.names["p.fit"]].envelopes.is_empty());
    assert!(p.functions[&p.names["p.cp"]].envelopes.is_empty());
    // A guard names an argument carrying a row or entity, an envelope it declares, and
    // arguments of the axis type, one or the two endpoints of an interval.
    for (from, to, expected) in [
        (
            "guards(p.T: T) valid",
            "guards(q.T: T) valid",
            "q, which is not one of its arguments",
        ),
        (
            "guards(p.T: T) valid",
            "guards(p.P: T) valid",
            "guards envelope P of table cp_data, which declares none",
        ),
        (
            "guards(p.T: T) valid",
            "guards(T.T: T) valid",
            "neither a row nor an entity argument",
        ),
        (
            "guards(p.T: [T0, T])",
            "guards(p.T: [T, T])",
            "two distinct arguments",
        ),
        (
            "guards(p.T: T) valid",
            "guards(p.T: T, p.T: T) valid",
            "twice",
        ),
        (
            "guards(p.T: T) valid",
            "guards(p.T: x) valid",
            "x, which is not one of its arguments",
        ),
    ] {
        let error = refusal(&text.replace(from, to));
        assert!(error.contains(expected), "{to}: {error}");
    }
    let pressure = text.replace(
        "fn cp(T: Temperature, p: Row<cp_data>) -> MolarCp guards(p.T: T)",
        "fn cp(T: Temperature, P: Pressure, p: Row<cp_data>) -> MolarCp guards(p.T: P)",
    );
    let error = refusal(&pressure);
    assert!(
        error.contains(
            "guards P by envelope T of table cp_data, whose axis is a Temperature; P is a Pressure"
        ),
        "{error}"
    );
}

/// An envelope's bounds are two distinct typed columns or attributes of exactly the axis
/// quantity type, declared by its relation or kind, and ordered in every row.
#[test]
fn envelope_bounds_must_match_axis_type() {
    let text = format!("package p {{ {BANK} {KIND} }}");
    admitted(&text).unwrap();
    for (from, to, expected) in [
        (
            "high: Temperature} envelope",
            "high: Pressure} envelope",
            "envelope T of table cp_data bounds a Temperature; its upper bound high is a Pressure",
        ),
        (
            "{c: MolarCp, low: Temperature",
            "{c: MolarCp, low: MolarCp",
            "envelope T of table cp_data bounds a Temperature; its lower bound low is a MolarCp",
        ),
        (
            "in low..high complete_over",
            "in low..top complete_over",
            "names upper bound top, which table cp_data does not declare",
        ),
        (
            "in low..high complete_over",
            "in low..low complete_over",
            "bounded by two distinct columns or attributes",
        ),
        (
            "envelope T: Temperature in low..high complete_over",
            "envelope T: Integer in low..high complete_over",
            "bounds a quantity",
        ),
        (
            "envelope T: Temperature in low..high complete_over",
            "envelope T: Temperature in low..high envelope T: Temperature in low..high complete_over",
            "declares envelope T twice",
        ),
        (
            "attribute high: Temperature; envelope",
            "attribute high: Pressure; envelope",
            "envelope T of kind band bounds a Temperature; its upper bound high is a Pressure",
        ),
        (
            "entity kind narrow extends band {}",
            "entity kind narrow extends band { envelope T: Temperature in low..high; }",
            "kind narrow redeclares envelope T, declared by kind band",
        ),
        (
            "entity kind narrow extends band {}",
            "entity kind narrow extends band {} def D { envelope T: Temperature in a..b; }",
            "an envelope declaration belongs to an entity kind",
        ),
        (
            "[b] = [80{J/(mol*K)}, 300{K}, 500{K}]",
            "[b] = [80{J/(mol*K)}, 500{K}, 300{K}]",
            "row cp_data[b] has envelope T reversed: its lower bound low exceeds its upper bound high",
        ),
        (
            "[a] = [75{J/(mol*K)}, 250{K}, 400{K}]; }\n fn band_cp",
            "[a] = [75{J/(mol*K)}, 450{K}, 400{K}]; }\n fn band_cp",
            "has envelope T reversed: its lower bound low exceeds its upper bound high",
        ),
    ] {
        assert!(text.contains(from), "{from}");
        let error = refusal(&text.replace(from, to));
        assert!(error.contains(expected), "{to}: {error}");
    }
}

/// The property package or the analysis selects the data layer's extrapolation policy:
/// one bank row read under packages selecting different policies is a domain predicate
/// under one and an observation under the other. The nearest selection decides, the
/// selection is recorded, and an extrapolation that cannot be observed where it is
/// evaluated is refused.
#[test]
fn consumer_selects_extrapolation_per_layer() {
    let text = format!(
        r#"package p {{ {BANK}
 fn outer(T: Temperature) -> MolarCp = cp(T, cp_data[a]);
 def Rejecting {{ var T: Temperature; var c: MolarCp; eq e: c == cp(T, cp_data[a]); }}
 def Extrapolating {{ extrapolation data extrapolate; var T: Temperature; var c: MolarCp; eq e: c == cp(T, cp_data[a]) + cp(T, cp_data[b]); }}
 def Strict {{ extrapolation data reject; child inner: Rejecting = Rejecting(); }}
 def Passing {{ extrapolation data extrapolate; var T: Temperature; var c: MolarCp; eq e: c == outer(T); }}
 def Local {{ extrapolation data extrapolate; var T: Temperature; var c: MolarCp; eq e: c == cp(t, cp_data[a]) where t = T; }}
 def Branch {{ extrapolation data extrapolate; var T: Temperature; var c: MolarCp; eq e: c == if T > 350{{K}} then cp(T, cp_data[a]) else 1{{J/(mol*K)}}; }}
 test analysis {{ extrapolation data extrapolate; child open: Rejecting = Rejecting(); child strict: Strict = Strict(); expect cp(500{{K}}, cp_data[a]) == 75{{J/(mol*K)}} tolerance 1e-9{{J/(mol*K)}}; }}
}}"#
    );
    let p = admitted(&text).unwrap();
    let selection = |scope: &str| {
        p.children[&p.names[scope]]
            .iter()
            .copied()
            .find(|id| p.declarations[id].value.extrapolation.is_some())
    };
    let table = p.names["p.cp_data"];
    let specialized = |model: &SpecializedModel| {
        model
            .functions
            .iter()
            .filter(|(_, f)| f.id == p.names["p.cp"])
            .map(|(name, f)| {
                (
                    name.clone(),
                    f.envelopes[0].policy,
                    f.envelopes[0].selection,
                )
            })
            .collect::<Vec<_>>()
    };
    // No selection rejects: the guard is a domain predicate and nothing is observed.
    let rejecting = root(&text, "Rejecting").unwrap();
    let rejecting_functions = specialized(&rejecting);
    let [(rejected, ExtrapolationPolicy::Reject, None)] = rejecting_functions.as_slice() else {
        panic!("{rejecting_functions:?}");
    };
    assert!(observations(&rejecting).is_empty());
    // The package selecting extrapolation observes the member each row guards, over the
    // intersection of the rows' envelopes, and records its selection.
    let extrapolating = root(&text, "Extrapolating").unwrap();
    let chosen = specialized(&extrapolating);
    assert_eq!(chosen.len(), 2);
    assert!(chosen.iter().all(|(name, policy, at)| {
        *policy == ExtrapolationPolicy::Extrapolate
            && *at == selection("p.Extrapolating")
            && name != rejected
    }));
    assert_eq!(
        observations(&extrapolating),
        [(
            "Extrapolating.T".to_owned(),
            table,
            300.,
            400.,
            selection("p.Extrapolating")
        )]
    );
    // An analysis selects for the packages beneath it that select nothing; the nearest
    // selection decides. A static argument's membership is decided and recorded.
    let analysis = root(&text, "analysis").unwrap();
    assert_eq!(
        observations(&analysis),
        [(
            "analysis.open.T".to_owned(),
            table,
            250.,
            400.,
            selection("p.analysis")
        )]
    );
    let [observed] = analysis.observations.as_slice() else {
        panic!("{:?}", analysis.observations);
    };
    assert_eq!(
        (
            observed.value,
            observed.lower,
            observed.upper,
            observed.within()
        ),
        (500., 250., 400., false)
    );
    assert_eq!(observed.selection, selection("p.analysis"));
    assert_eq!(observed.lineage.declaration, table);
    // A guarded argument passed on unchanged is observed at the member it came from.
    let passing = root(&text, "Passing").unwrap();
    assert_eq!(
        observations(&passing),
        [(
            "Passing.T".to_owned(),
            table,
            250.,
            400.,
            selection("p.Passing")
        )]
    );
    // An extrapolation that cannot be observed where it is evaluated is refused.
    let error = root(&text, "Local").unwrap_err().to_string();
    assert!(
        error
            .contains("neither a model member, a static value nor an argument passed on unchanged"),
        "{error}"
    );
    let error = root(&text, "Branch").unwrap_err().to_string();
    assert!(error.contains("inside a conditional branch"), "{error}");
}

/// Only the data layer's policy is selected: the form layer never extrapolates, the closure
/// layer's annotation states its own, and a selection belongs once to a definition, test or
/// case. Under a selected extrapolation the form layer's domain and a rejecting closure
/// range still hold, so validity is their intersection.
#[test]
fn selection_is_the_data_layer_of_a_consumer_scope() {
    let text = format!(
        r#"package p {{ {BANK}
 def Root {{ extrapolation data extrapolate; var T: Temperature; var c: MolarCp; eq e: c == cp(T, cp_data[a]); annotation valid T(260{{K}}, 390{{K}}, reject); }}
}}"#
    );
    let model = root(&text, "Root").unwrap();
    let f = model
        .functions
        .values()
        .find(|f| !f.envelopes.is_empty())
        .unwrap();
    assert!(f.validity.is_some(), "the form layer's domain is kept");
    assert_eq!(f.envelopes[0].policy, ExtrapolationPolicy::Extrapolate);
    let layers = model
        .annotations
        .iter()
        .filter_map(|a| match &a.value {
            AnnotationValue::Valid { policy, layer, .. } => Some((*layer, *policy)),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        layers,
        [
            (
                ModelingValidityLayer::Data,
                ExtrapolationPolicy::Extrapolate
            ),
            (ModelingValidityLayer::Closure, ExtrapolationPolicy::Reject),
        ]
        .into()
    );
    for (to, expected) in [
        (
            "extrapolation form extrapolate;",
            "the form layer never extrapolates",
        ),
        (
            "extrapolation form reject;",
            "the form layer never extrapolates",
        ),
        (
            "extrapolation closure extrapolate;",
            "stated by its validity annotation",
        ),
        (
            "extrapolation data extrapolate; extrapolation data reject;",
            "one extrapolation policy per scope selects the data layer",
        ),
    ] {
        let error = refusal(&text.replace("extrapolation data extrapolate;", to));
        assert!(error.contains(expected), "{to}: {error}");
    }
    for placement in [
        "entity kind k { extrapolation data extrapolate; }",
        "interface I { extrapolation data extrapolate; }",
        "extrapolation data extrapolate;",
        "def W { when true { extrapolation data extrapolate; } }",
    ] {
        let error = refusal(&text.replace("def Root", &format!("{placement} def Root")));
        assert!(
            error.contains("selects an extrapolation policy"),
            "{placement}: {error}"
        );
    }
}
