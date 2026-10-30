// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

const CHILD: &str = "generation::transfer::tests::handoff_child";
const MODE: &str = "PSE_HANDOFF_TEST_MODE";
const RECEIPT: &str = "PSE_HANDOFF_TEST_RECEIPT";

fn reference(tools: &Path, generation: &str) -> PreparedGeneration {
    let manifest = GenerationManifest::load(&tools.join(generation)).unwrap();
    PreparedGeneration {
        generation: generation.into(),
        protocol: manifest.control_protocol(),
        manifest_digest: manifest.digest().unwrap(),
    }
}
fn entry(tools: &Path, generation: &str) -> EntryGuard {
    EntryGuard {
        usage: Some(GenerationUse::acquire(tools, generation).unwrap()),
        reference: Some(reference(tools, generation)),
    }
}

#[test]
fn handoff_child() {
    let Some(mode) = std::env::var_os(MODE) else {
        return;
    };
    let executable = std::env::current_exe().unwrap();
    let tools = executable.parent().unwrap().parent().unwrap();
    if mode == "prepare" {
        let source = EntryGuard::acquire_current().unwrap();
        let target = GenerationUse::acquire(tools, "generation.target").unwrap();
        let handoff = begin(&source, &target, &reference(tools, "generation.target")).unwrap();
        let payload = String::from_utf8(encoded(&handoff).unwrap()).unwrap();
        let error = Command::new(tools.join("generation.target/pse-host"))
            .args(["--exact", CHILD, "--nocapture"])
            .env(MODE, "adopt")
            .env(HANDOFF, payload)
            .exec();
        panic!("same-process exec failed: {error}");
    }
    let adopted = EntryGuard::acquire_current().unwrap();
    assert_eq!(adopted.prepared().unwrap().generation, "generation.target");
    if mode == "adopt" {
        assert_eq!(
            GenerationUse::inspect(tools, "generation.source").unwrap(),
            GenerationUseState::Available
        );
    } else {
        assert_eq!(mode, "foreign");
        assert_eq!(
            GenerationUse::inspect(tools, "generation.source").unwrap(),
            GenerationUseState::Active
        );
    }
    adopted.complete(EntrySettlement::ResourcesSettled).unwrap();
    fs::write(
        std::env::var_os(RECEIPT).unwrap(),
        "authenticated same-image receipt\n",
    )
    .unwrap();
}

#[test]
fn actual_same_process_exec_authenticates_target_before_retiring_source() {
    let (_fixture, tools) = crate::generation::tests::transfer_fixture();
    let receipt = tools.join("receipt");
    let result = Command::new(tools.join("generation.source/pse-host"))
        .args(["--exact", CHILD, "--nocapture"])
        .env(MODE, "prepare")
        .env(RECEIPT, &receipt)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(receipt.is_file());
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.source").unwrap(),
        GenerationUseState::Available
    );
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.target").unwrap(),
        GenerationUseState::Available
    );
}

#[test]
fn inherited_locator_cannot_retire_another_process_owned_claims() {
    let (_fixture, tools) = crate::generation::tests::transfer_fixture();
    let source = entry(&tools, "generation.source");
    let target = GenerationUse::acquire(&tools, "generation.target").unwrap();
    let handoff = begin(&source, &target, &reference(&tools, "generation.target")).unwrap();
    let payload = String::from_utf8(encoded(&handoff).unwrap()).unwrap();
    let result = Command::new(tools.join("generation.target/pse-host"))
        .args(["--exact", CHILD, "--nocapture"])
        .env(MODE, "foreign")
        .env(RECEIPT, tools.join("receipt"))
        .env(HANDOFF, payload)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.source").unwrap(),
        GenerationUseState::Active
    );
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.target").unwrap(),
        GenerationUseState::Active
    );
    source.usage.as_ref().unwrap().set_handoff(None).unwrap();
    target.settle().unwrap();
    source.complete(EntrySettlement::NoEffects).unwrap();
}

#[test]
fn forged_target_identity_keeps_both_original_claims_protected() {
    let (_fixture, tools) = crate::generation::tests::transfer_fixture();
    let source = entry(&tools, "generation.source");
    let target = GenerationUse::acquire(&tools, "generation.target").unwrap();
    let mut target_identity = reference(&tools, "generation.target");
    target_identity.manifest_digest = "forged".into();
    let handoff = begin(&source, &target, &target_identity).unwrap();
    let payload = String::from_utf8(encoded(&handoff).unwrap()).unwrap();
    assert!(adopt(&tools, "generation.target", Some(OsStr::new(&payload))).is_err());
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.source").unwrap(),
        GenerationUseState::Active
    );
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.target").unwrap(),
        GenerationUseState::Active
    );
    source.usage.as_ref().unwrap().set_handoff(None).unwrap();
    target.settle().unwrap();
    source.complete(EntrySettlement::NoEffects).unwrap();
}

#[test]
fn exec_failure_retains_original_guard_and_settles_unused_target() {
    let (_fixture, tools) = crate::generation::tests::transfer_fixture();
    let source = entry(&tools, "generation.source");
    let target = GenerationUse::acquire(&tools, "generation.target").unwrap();
    begin(&source, &target, &reference(&tools, "generation.target")).unwrap();
    assert_eq!(
        Command::new(tools.join("missing-executable")).exec().kind(),
        std::io::ErrorKind::NotFound
    );
    cancel(&source, target).unwrap();
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.source").unwrap(),
        GenerationUseState::Active
    );
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.target").unwrap(),
        GenerationUseState::Available
    );
    assert!(
        read_record(&source.usage.as_ref().unwrap().record)
            .unwrap()
            .handoff
            .is_none()
    );
    source.complete(EntrySettlement::NoEffects).unwrap();
}

#[test]
fn loop_marker_is_scoped_to_actual_process_identity() {
    let mut inputs = Environment::from([("PSE_HOST_REEXEC".into(), "1".into())]);
    assert!(!was_reexecuted(&inputs).unwrap());
    let current = identity().unwrap();
    inputs.insert(
        "PSE_HOST_REEXEC".into(),
        String::from_utf8(encoded(&current).unwrap())
            .unwrap()
            .into(),
    );
    assert!(was_reexecuted(&inputs).unwrap());
    let mut foreign = current;
    foreign.pid = foreign.pid.saturating_add(1);
    inputs.insert(
        "PSE_HOST_REEXEC".into(),
        String::from_utf8(encoded(&foreign).unwrap())
            .unwrap()
            .into(),
    );
    assert!(!was_reexecuted(&inputs).unwrap());
}
