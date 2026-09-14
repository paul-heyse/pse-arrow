// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Structural round trips for 1024 recursively generated expressions per run.

use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, RngSeed};
use pse_authoring::dsl::{parse_expr, render_expr};

fn expressions() -> impl Strategy<Value = String> {
    prop_oneof![
        (0_u32..100_000).prop_map(|n| n.to_string()),
        "var_[a-z_]{0,8}"
    ]
    .prop_recursive(6, 128, 3, |inner| {
        prop_oneof![
            (
                inner.clone(),
                inner.clone(),
                prop_oneof![Just("+"), Just("-"), Just("*"), Just("/"), Just("^")]
            )
                .prop_map(|(left, right, op)| format!("({left}) {op} ({right})")),
            inner.clone().prop_map(|value| format!("-({value})")),
            inner.clone().prop_map(|value| format!("exp({value})")),
            (inner.clone(), inner)
                .prop_map(|(left, right)| format!("weighted_mean(1, {left}, 2, {right})")),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases:1024, rng_seed:RngSeed::Fixed(0x5053_452d_4453_4c31), failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("proptest-regressions"))), ..ProptestConfig::default() })]
    #[test]
    fn parse_render_preserves_structure(text in expressions()) {
        let parsed=parse_expr(&text).map_err(|error|TestCaseError::fail(format!("{text}: {error}")))?;
        let rendered=render_expr(&parsed);
        let reparsed=parse_expr(&rendered).map_err(|error|TestCaseError::fail(format!("{rendered}: {error}")))?;
        prop_assert!(parsed.structural_eq(&reparsed),"{} -> {}",text,rendered);
    }
}

#[cfg(feature = "arbitrary")]
proptest! {
    #![proptest_config(ProptestConfig { cases:1024, rng_seed:RngSeed::Fixed(0x5053_452d_4153_5431), failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("proptest-regressions"))), ..ProptestConfig::default() })]
    #[test]
    fn arbitrary_ast_render_parse_preserves_structure(ast in any::<pse_authoring::dsl::Expr>()) {
        let rendered = render_expr(&ast);
        let reparsed = parse_expr(&rendered).map_err(|error| TestCaseError::fail(format!("{rendered}: {error}")))?;
        prop_assert!(ast.structural_eq(&reparsed),"{rendered}");
    }
}
