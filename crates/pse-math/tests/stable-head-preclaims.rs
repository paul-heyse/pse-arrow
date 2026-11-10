// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Nextest gives each reserved-name admission control a fresh symbol registry.
use symbolica::atom::{EvaluationInfo, NamespacedSymbol, Symbol, SymbolAttribute, SymbolBuilder};

fn reject_preclaimed_head(with_callback: bool) -> Result<(), Box<dyn std::error::Error>> {
    let first = "pse_math::stable_v1_log1p";
    let claimed = "pse_math::stable_v1_log_ratio";
    assert!(Symbol::get_symbol(NamespacedSymbol::try_from(first)?).is_none());
    let mut builder = SymbolBuilder::new(NamespacedSymbol::try_from(claimed)?)
        .with_attributes(Vec::<SymbolAttribute>::new());
    if with_callback {
        builder =
            builder.with_evaluation_info(EvaluationInfo::new().register(|_args: &[f64]| 42.0));
    }
    let foreign = builder.build().map_err(|error| error.to_string())?;
    let Err(error) = pse_math::initialize() else {
        return Err("initialization accepted a foreign reserved head".into());
    };
    assert!(error.to_string().contains("already defined"), "{error}");
    assert!(error.to_string().contains(claimed), "{error}");
    // The whole group is checked under one supplier state lock before insertion.
    assert!(Symbol::get_symbol(NamespacedSymbol::try_from(first)?).is_none());
    assert_eq!(
        Symbol::get_symbol(NamespacedSymbol::try_from(claimed)?),
        Some(foreign)
    );
    // An owned initialization failure stays a failure; it never adopts foreign callbacks.
    assert!(pse_math::initialize().is_err());
    Ok(())
}

#[test]
fn reserved_stable_head_refuses_matching_attributes_with_foreign_callback()
-> Result<(), Box<dyn std::error::Error>> {
    reject_preclaimed_head(true)
}

#[test]
fn reserved_stable_head_refuses_matching_attributes_without_callback()
-> Result<(), Box<dyn std::error::Error>> {
    reject_preclaimed_head(false)
}
