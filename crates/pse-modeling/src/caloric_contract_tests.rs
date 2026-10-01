// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Caloric increments retain their declared entropy and heat-capacity distinctions.
use crate::kernel_types::{physical, source};
use crate::*;
use pse_ids::SemanticId;
use pse_quantity::{QuantityTypeId, scheme::Scheme};
use std::collections::BTreeMap;

fn id(hex: &str) -> QuantityTypeId {
    QuantityTypeId::from_id(SemanticId::parse_hex(hex).unwrap())
}
fn preconditions() -> pse_quantity::PhysicalPreconditions {
    pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
        .unwrap()
}
/// The entropy increment ∫cp dT/T is a molar entropy difference through one declared
/// reinterpretation: the physical document defines molar entropy as a molar heat capacity
/// taken over a logarithmic temperature increment, which ln(T/T0) and (T − T0)/T are by
/// its registered rules. The factored closed form is exact.
#[test]
fn ds_increment_types_as_entropy_difference() {
    let (registry, _) = physical();
    let preconditions = preconditions();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        preconditions: &preconditions,
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    // Static evaluation of the same increment, inline: coefficients and bounds are
    // typed parameters, and the bounds bracket 125.66848739118448 J/(mol K), computed
    // independently as 10·ln(4/3) + Σ c_k (T^k − T0^k)/k.
    let execute = |lo: &str, hi: &str| {
        let increment = "(c1*log(T/T0) + (c2*T + c3*T*T/2 + c3*T*T0/2 + c4*T*T*T/3 + c4*T*T*T0/3 + c4*T*T0*T0/3 + c5*T*T*T*T/4 + c5*T*T*T*T0/4 + c5*T*T*T0*T0/4 + c5*T*T0*T0*T0/4)*r where r = (T - T0)/T)";
        let text = format!(
            "package p {{
            def Child(T0: Temperature, T: Temperature, c1: MolarCp, c2: MolarCp/Temperature, c3: MolarCp/Temperature^2, c4: MolarCp/Temperature^3, c5: MolarCp/Temperature^4, lo: DeltaS, hi: DeltaS) {{
                require {increment} > lo and {increment} < hi : \"dippr100\";
                var x: Scalar; eq e: x == 1;
            }}
            def Root {{ child inner: Child = Child(T0=300{{K}}, T=400{{K}}, c1=10{{J/(K*mol)}}, c2=0.5{{J/(K^2*mol)}}, c3=0.001{{J/(K^3*mol)}}, c4=0.000002{{J/(K^4*mol)}}, c5=0.000000003{{J/(K^5*mol)}}, lo={lo}, hi={hi}); }}
        }}"
        );
        let package = check(&source(&text), &context)?;
        specialize(
            &package,
            package.names["p.Root"],
            InstanceId::from_id(SemanticId::NIL),
            &Bindings::default(),
            Limits::default(),
        )
    };
    execute("125.668487391{J/(K*mol)}", "125.668487392{J/(K*mol)}").unwrap();
    assert!(execute("126{J/(K*mol)}", "127{J/(K*mol)}").is_err());
    // The increment's pieces carry their declared kinds: ln(T/T0) and (T − T0)/T are
    // logarithmic temperature increments; T/T0 is a temperature ratio.
    let at = DeclarationId::from(SemanticId::NIL);
    let empty = check(&source("package q {}"), &context).unwrap();
    let temperature = Type::Quantity(Scheme::Concrete(id("c64b96975a4a59755f8711d3bf628bc9")));
    let env = BTreeMap::from([
        ("T0".into(), temperature.clone()),
        ("T".into(), temperature),
    ]);
    let increment = |text: &str| {
        let expression = pse_authoring::dsl::parse_expr(text).unwrap();
        expression::infer(&expression, &env, &empty, &context, at, None).unwrap()
    };
    let kind = |ty: &Type| {
        let Type::Quantity(Scheme::Concrete(q)) = ty else {
            panic!("{ty:?}");
        };
        registry.quantity_type(*q).unwrap().key.kind
    };
    assert_eq!(
        kind(&increment("log(T/T0)")),
        kind(&increment("(T - T0)/T"))
    );
    assert_ne!(kind(&increment("T/T0")), kind(&increment("log(T/T0)")));
}

#[test]
fn returning_heat_capacity_where_entropy_is_expected_is_refused() {
    let (registry, _) = physical();
    let preconditions = preconditions();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        preconditions: &preconditions,
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    let form = |body: &str| {
        check(
            &source(&format!(
                "package bad {{ fn ds(T0: Temperature, T: Temperature, c1: MolarCp, c2: MolarCp/Temperature) -> DeltaS = {body}; }}"
            )),
            &context,
        )
    };
    // Heat-capacity terms do not carry the declared logarithmic entropy increment.
    for body in [
        "c1",
        "c2*(T - T0)",
        "c1*log(T/T0) + c2*(T - T0)",
        "c1*(T/T0)",
        "c2*T - c2*T0",
    ] {
        assert!(form(body).is_err(), "{body}");
    }
    // With the declared reinterpretation stated in the body, the same terms are entropy.
    assert!(form("c1*log(T/T0) + c2*T*r where r = (T - T0)/T").is_ok());
    // A literal has no kind of its own: the expected type selects it (§8.3).
    assert!(form("5{J/(K*mol)}").is_ok());
}
