// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Unit literals flatten into one canonical product whose identity is spelling-independent
//! (ADR-0124, Plan 23 KR1).

use super::{ExprKind, parse_expr, render_expr};
use pse_ids::SemanticId;
use pse_quantity::{
    DimensionVector, QuantityRegistry, QuantityRegistryBuilder, Ratio, Unit, UnitFactor, UnitId,
    UnitProduct, unit_product_id,
};

fn unit_of(text: &str) -> UnitProduct {
    let expression = parse_expr(text).unwrap();
    let ExprKind::Number(number) = expression.kind else {
        panic!("{text} is not a number");
    };
    number.unit.unwrap()
}

fn atomic(n: u8, symbol: &str, dimension: DimensionVector) -> Unit {
    Unit {
        id: UnitId::from_id(SemanticId::from_bytes([n; 16])),
        symbol: symbol.into(),
        dimension,
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: None,
        definition: None,
    }
}

/// Atomic J, K, mol and m only: no composite unit is registered.
fn registry() -> QuantityRegistry {
    use pse_quantity::BaseDimension as B;
    let energy = DimensionVector::base(B::Mass)
        .mul(
            &DimensionVector::base(B::Length)
                .pow(Ratio::new(2, 1).unwrap())
                .unwrap(),
        )
        .unwrap()
        .div(
            &DimensionVector::base(B::Time)
                .pow(Ratio::new(2, 1).unwrap())
                .unwrap(),
        )
        .unwrap();
    let mut builder = QuantityRegistryBuilder::new();
    builder
        .unit(atomic(1, "J", energy))
        .unit(atomic(2, "K", DimensionVector::base(B::Temperature)))
        .unit(atomic(3, "mol", DimensionVector::base(B::Amount)))
        .unit(atomic(4, "m", DimensionVector::base(B::Length)));
    builder.build().unwrap()
}

#[test]
fn unit_product_is_order_independent() {
    let registry = registry();
    let spellings = [
        "1{J/(K*mol)}",
        "1{J/(mol*K)}",
        "1{J/mol/K}",
        "1{J*mol^-1*K^-1}",
        "1{K^-1*J/mol}",
        "1{(mol*K/J)^-1}",
    ];
    let products = spellings.map(unit_of);
    let composed = products
        .iter()
        .map(|product| registry.compose(product).unwrap())
        .collect::<Vec<_>>();
    for (spelling, (product, unit)) in spellings.iter().zip(products.iter().zip(&composed)) {
        assert_eq!(product, &products[0], "{spelling}");
        assert_eq!(unit.id, composed[0].id, "{spelling}");
    }
    // The identity is the catalog-frame derivation over the canonical atomic factors.
    let id = |n: u8| UnitId::from_id(SemanticId::from_bytes([n; 16]));
    let minus_one = Ratio::new(-1, 1).unwrap();
    assert_eq!(
        composed[0].id,
        unit_product_id(&[
            UnitFactor {
                unit: id(1),
                exponent: Ratio::ONE
            },
            UnitFactor {
                unit: id(2),
                exponent: minus_one
            },
            UnitFactor {
                unit: id(3),
                exponent: minus_one
            },
        ])
    );
    // A different product is a different identity; a lone atomic unit is itself.
    assert_ne!(
        registry.compose(&unit_of("1{J/mol}")).unwrap().id,
        composed[0].id
    );
    assert_eq!(registry.compose(&unit_of("1{K}")).unwrap().id, id(2));
    assert_eq!(products[0].to_string(), "J/(K*mol)");
}

#[test]
fn rational_unit_exponents_canonicalize() {
    let registry = registry();
    for (left, right) in [
        ("1{m^(1/2)*m^(1/2)}", "1{m}"),
        ("1{m^(2/4)}", "1{m^(1/2)}"),
        ("1{(m^3)^(1/3)}", "1{m}"),
        ("1{m^(3/2)/m^(1/2)}", "1{m}"),
        ("1{J^(1/2)*J^(1/2)/K}", "1{J/K}"),
        ("1{m^(-1/2)}", "1{1/m^(1/2)}"),
        ("1{m/m}", "1{1}"),
    ] {
        let (a, b) = (unit_of(left), unit_of(right));
        assert_eq!(a, b, "{left} and {right}");
        assert_eq!(
            registry.compose(&a).unwrap().id,
            registry.compose(&b).unwrap().id,
            "{left} and {right}"
        );
    }
    let half = registry.compose(&unit_of("1{m^(1/2)}")).unwrap();
    assert_eq!(
        half.dimension,
        DimensionVector::base(pse_quantity::BaseDimension::Length)
            .pow(Ratio::new(1, 2).unwrap())
            .unwrap()
    );
    // Rendering is canonical and parses back to the same product.
    for text in [
        "1{m^(3/2)/(K*mol^2)}",
        "1{1/m^(1/2)}",
        "1{1}",
        "1{mol*J^(2/3)}",
    ] {
        let rendered = render_expr(&parse_expr(text).unwrap());
        assert_eq!(unit_of(&rendered), unit_of(text), "{text} -> {rendered}");
    }
    assert_eq!(unit_of("1{m^(2/4)}").to_string(), "m^(1/2)");
    // A decimal exponent is never guessed to be a rational; other malformed forms refuse.
    for text in [
        "1{m^0.5}",
        "1{m^(1/0)}",
        "1{m^}",
        "1{m^2^2}",
        "1{2*m}",
        "1{m/}",
        "1{(m}",
    ] {
        assert!(parse_expr(text).is_err(), "{text}");
    }
}

#[test]
fn logic_uses_shared_quoted_paths_counts_and_precedence() {
    use super::{Proposition, parse_proposition};
    let value =
        parse_proposition("'left valve'[i] implies right or not closed xor spare and ready")
            .unwrap();
    assert!(
        matches!(value, Proposition::Implies(_, right) if matches!(*right, Proposition::Or(_)))
    );
    let exactly = parse_proposition("exactly(1+1, route.'first stage'[i], standby)").unwrap();
    assert!(
        matches!(exactly, Proposition::Exactly(count, values) if matches!(count.kind, ExprKind::Binary { .. }) && values.len() == 2)
    );
    for invalid in ["", "a and", "exactly(2)", "exactly(1,)", "a b", "a[i"] {
        assert!(parse_proposition(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn typed_callees_preserve_quoted_indexed_receivers_and_partial_paths() {
    let expression = parse_expr("'unit bank'[i].f(x)").unwrap();
    let ExprKind::NamedCall { name, .. } = &expression.kind else {
        panic!("named call");
    };
    assert_eq!(name.segments[0].name, "unit bank");
    assert_eq!(name.segments[0].indices.len(), 1);
    assert_eq!(name.segments[1].name, "f");
    let source = render_expr(&expression);
    assert_eq!(parse_expr(&source).unwrap().kind, expression.kind);
    let partial = parse_expr("partial('unit bank'[i].f, x)(y)").unwrap();
    let ExprKind::Partial { function, .. } = partial.kind else {
        panic!("partial");
    };
    assert_eq!(super::render_path(&function), super::render_path(name));
}
