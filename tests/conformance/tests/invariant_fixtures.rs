// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Runtime fixture discovery with libtest-compatible names and selection.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test runner assertions and explicit harness failure controls"
)]
#[path = "../src/fixture.rs"]
mod fixture;
use libtest_mimic::{Arguments, Trial};
use std::{collections::BTreeSet, process::ExitCode, sync::Arc};

fn main() -> ExitCode {
    let args = Arguments::from_args();
    let trials = if std::env::var_os("PSE_FIXTURE_HARNESS_PROBE").is_some() {
        vec![
            Trial::test("probe::pass", || Ok(())),
            Trial::test("probe::fail", || Err("intentional harness failure".into())),
        ]
    } else {
        discover()
    };
    let selected = trials
        .iter()
        .any(|trial| !args.is_filtered_out(trial) && !args.is_ignored(trial));
    let result = libtest_mimic::run(&args, trials);
    if result.has_failed() || (!args.list && !selected) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
fn discover() -> Vec<Trial> {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let mut trials = vec![
        Trial::test("fixture_harness_protocol", protocol),
        Trial::test(
            "fixture::generate::unit::declared_fixture_literals_decode_with_semantic_fields",
            || {
                fixture::validate_literals();
                Ok(())
            },
        ),
    ];
    let mut names = BTreeSet::new();
    for invariant in registry.invariants() {
        for case in ["valid", "violating"] {
            let name = format!(
                "generated::{}_{}",
                invariant.qualified_name().replace([':', '.', '-'], "_"),
                case
            );
            assert!(
                names.insert(name.clone()),
                "duplicate fixture test identity: {name}"
            );
            let registry = Arc::clone(&registry);
            let invariant = invariant.clone();
            trials.push(Trial::test(name, move || {
                let path = fixture::directory(&invariant).join(format!("{case}.yaml"));
                let input = fixture::Fixture::load(&path);
                assert_eq!(input.invariant, invariant.qualified_name());
                let expected = input.expected(&registry);
                assert_eq!(expected.is_empty(), case == "valid");
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                let actual = runtime.block_on(fixture::execute(&registry, &input));
                assert_eq!(actual, expected, "{} / {case}", invariant.qualified_name());
                Ok(())
            }));
        }
    }
    let expected: BTreeSet<_> = registry
        .invariants()
        .iter()
        .map(fixture::directory)
        .collect();
    trials.push(Trial::test(
        "every_registered_invariant_has_exactly_its_fixture_directory",
        move || {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/invariants");
            let actual: BTreeSet<_> = std::fs::read_dir(root)?
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.is_dir())
                .collect();
            assert_eq!(expected, actual);
            Ok(())
        },
    ));
    trials.sort_by(|left, right| left.name().cmp(right.name()));
    trials
}
fn protocol() -> Result<(), libtest_mimic::Failed> {
    let executable = std::env::current_exe()?;
    let invoke = |args: &[&str]| {
        std::process::Command::new(&executable)
            .env("PSE_FIXTURE_HARNESS_PROBE", "1")
            .args(args)
            .output()
            .unwrap()
    };
    let listed = invoke(&["--list", "--format", "terse"]);
    assert!(listed.status.success());
    assert_eq!(
        String::from_utf8(listed.stdout).unwrap(),
        "probe::pass: test\nprobe::fail: test\n"
    );
    assert!(invoke(&["probe::pass", "--exact"]).status.success());
    assert!(!invoke(&["probe::fail", "--exact"]).status.success());
    assert!(!invoke(&["absent", "--exact"]).status.success());
    assert!(invoke(&["absent", "--exact", "--list"]).status.success());
    assert!(invoke(&["probe", "--skip", "probe::fail"]).status.success());
    Ok(())
}
