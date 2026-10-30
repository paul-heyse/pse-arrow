// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn scope(tools: &Path) -> FinalizerUseScope {
    FinalizerUseScope::new(
        tools.parent().unwrap().parent().unwrap().to_owned(),
        tools
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("state/invocations/original"),
        CliInvocationId::new(),
        "original-manager".into(),
        LaunchIntent::fresh().unwrap(),
        UnitBinding {
            invocation: vec![1; 16],
            group: "/original-controller".into(),
            device: 1,
            inode: 2,
        },
        transfer::identity().unwrap(),
    )
    .unwrap()
}

#[test]
fn finalizer_binding_retires_only_its_same_current_original_claim() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let original = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let original_path = original.record.clone();
    drop(original);
    let usage = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let local_path = usage.record.clone();
    let expected = scope(&tools);
    let bound = usage.bind_finalizer(expected.clone()).unwrap();
    assert_eq!(bound.scope(), &expected);
    let locator = serde_json::to_value(bound.locator()).unwrap();
    assert_eq!(
        locator["record"],
        local_path.file_stem().unwrap().to_str().unwrap()
    );
    assert_eq!(
        transfer::read_record(&local_path).unwrap().finalizer,
        Some(expected)
    );
    bound.settle_current().unwrap();
    assert!(!local_path.exists());
    assert!(original_path.exists());
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        GenerationUseState::Unsettled
    );
}

#[test]
fn dropping_bound_finalizer_retains_its_unfinished_obligation() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let usage = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let path = usage.record.clone();
    drop(usage.bind_finalizer(scope(&tools)).unwrap());
    assert!(path.exists());
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        GenerationUseState::Unsettled
    );
}

#[test]
fn bound_locator_preserves_native_directory_bytes_without_path_adoption() {
    use std::os::unix::ffi::OsStringExt;
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let usage = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let path = usage.record.clone();
    let mut expected = scope(&tools);
    expected
        .directory
        .push(OsString::from_vec(vec![b'/', 0xff]));
    let bound = usage.bind_finalizer(expected.clone()).unwrap();
    assert_eq!(
        transfer::read_record(&path).unwrap().finalizer,
        Some(expected)
    );
    serde_json::to_vec(bound.locator()).unwrap();
    bound.settle_current().unwrap();
}

#[test]
fn excessive_finalizer_binding_refuses_before_changing_original_claim() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let usage = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let path = usage.record.clone();
    let original = fs::read(&path).unwrap();
    let mut oversized = scope(&tools);
    oversized.manager = "x".repeat(17 * 1024);
    assert!(usage.bind_finalizer(oversized).is_err());
    assert_eq!(fs::read(path).unwrap(), original);
}

#[test]
fn foreign_owner_scope_or_changed_binding_cannot_retire_original_claim() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let usage = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let path = usage.record.clone();
    let bound = usage.bind_finalizer(scope(&tools)).unwrap();
    let mut changed = transfer::read_record(&path).unwrap();
    changed
        .finalizer
        .as_mut()
        .unwrap()
        .directory
        .push("changed");
    fs::write(&path, encoded(&changed).unwrap()).unwrap();
    assert!(bound.settle_current().is_err());
    assert!(path.exists());
    let foreign = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let mut record = transfer::read_record(&foreign.record).unwrap();
    record.owner.pid = record.owner.pid.saturating_add(1);
    fs::write(&foreign.record, encoded(&record).unwrap()).unwrap();
    assert!(foreign.bind_finalizer(scope(&tools)).is_err());
    let other = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let mut wrong = scope(&tools);
    wrong.checkout.push("foreign-checkout");
    assert!(other.bind_finalizer(wrong).is_err());
}
