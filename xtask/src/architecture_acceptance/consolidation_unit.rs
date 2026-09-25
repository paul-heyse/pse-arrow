// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! CLI forwarding controls. Receipt and successor contracts are exercised by
//! `scripts.tests.test_validation` and `scripts.tests.test_implementation_phase`.
use super::*;

#[test]
fn historical_plan_cannot_authorize_current_execution() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let options = RunOptions {
        plan: 10,
        ..RunOptions::default()
    };
    assert!(run(root, Path::new("unused-output"), &options).is_err());
    assert_eq!(active_plan(root).unwrap(), 14);
}
