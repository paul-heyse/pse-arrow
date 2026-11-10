// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Nextest gives each reserved-name admission control a fresh symbol registry.
use symbolica::atom::{EvaluationInfo, NamespacedSymbol, Symbol, SymbolAttribute, SymbolBuilder};

fn reject_preclaimed_head(with_callback: bool) {
    let first = "pse_math::stable_v1_log1p";
    let claimed = "pse_math::stable_v1_log_ratio";
    assert!(Symbol::get_symbol(NamespacedSymbol::try_from(first).unwrap()).is_none());
    let mut builder = SymbolBuilder::new(NamespacedSymbol::try_from(claimed).unwrap())
        .with_attributes(Vec::<SymbolAttribute>::new());
    if with_callback {
        builder =
            builder.with_evaluation_info(EvaluationInfo::new().register(|_args: &[f64]| 42.0));
    }
    let foreign = builder.build().unwrap();
    let error = pse_math::initialize().unwrap_err();
    assert!(error.to_string().contains("already defined"), "{error}");
    assert!(error.to_string().contains(claimed), "{error}");
    // The whole group is checked under one supplier state lock before insertion.
    assert!(Symbol::get_symbol(NamespacedSymbol::try_from(first).unwrap()).is_none());
    assert_eq!(
        Symbol::get_symbol(NamespacedSymbol::try_from(claimed).unwrap()),
        Some(foreign)
    );
    // An owned initialization failure stays a failure; it never adopts foreign callbacks.
    assert!(pse_math::initialize().is_err());
}

#[test]
fn reserved_stable_head_refuses_matching_attributes_with_foreign_callback() {
    reject_preclaimed_head(true);
}

#[test]
fn reserved_stable_head_refuses_matching_attributes_without_callback() {
    reject_preclaimed_head(false);
}
