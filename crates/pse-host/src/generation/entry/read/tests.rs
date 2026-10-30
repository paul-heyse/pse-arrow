// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn claims(tools: &Path) -> usize {
    fs::read_dir(tools.join(".uses/generation.fixture"))
        .unwrap()
        .count()
}

#[test]
fn passive_guard_holds_shared_lock_without_claim_and_promotes_before_effects() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let read = GenerationReadGuard::acquire(&tools, "generation.fixture").unwrap();
    assert_eq!(claims(&tools), 0);
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        GenerationUseState::Active
    );
    assert!(
        rustix::io::fcntl_getfd(&read._lock)
            .unwrap()
            .contains(rustix::io::FdFlags::CLOEXEC)
    );
    let usage = read.promote().unwrap();
    assert_eq!(claims(&tools), 1);
    drop(read);
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        GenerationUseState::Active
    );
    drop(usage);
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        GenerationUseState::Unsettled
    );
}

#[test]
fn passive_drop_does_not_retire_an_independent_original_claim() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let original = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    drop(original);
    let read = GenerationReadGuard::acquire(&tools, "generation.fixture").unwrap();
    drop(read);
    assert_eq!(claims(&tools), 1);
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        GenerationUseState::Unsettled
    );
}

#[test]
fn promotion_refuses_changed_integrity_and_keeps_passive_protection() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let read = GenerationReadGuard::acquire(&tools, "generation.fixture").unwrap();
    let mut changed = read.manifest().clone();
    changed.source.context.profile = "changed-context".into();
    fs::write(
        tools.join("generation.fixture/generation.json"),
        encoded(&changed).unwrap(),
    )
    .unwrap();
    let error = read.promote().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("manifest changed before durable promotion")
    );
    assert_eq!(claims(&tools), 0);
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        GenerationUseState::Active
    );
    drop(read);
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        GenerationUseState::Available
    );
}

const CHILD: &str = "generation::entry::read::tests::passive_process_child";
const TOOLS: &str = "PSE_PASSIVE_GENERATION_TEST_TOOLS";
const READY: &str = "PSE_PASSIVE_GENERATION_TEST_READY";
const PROMOTED: &str = "PSE_PASSIVE_GENERATION_TEST_PROMOTED";

#[test]
fn passive_process_child() {
    let Some(tools) = std::env::var_os(TOOLS) else {
        return;
    };
    let read = GenerationReadGuard::acquire(Path::new(&tools), "generation.fixture").unwrap();
    let usage = std::env::var_os(PROMOTED).map(|_| read.promote().unwrap());
    fs::write(std::env::var_os(READY).unwrap(), b"shared guard acquired").unwrap();
    let mut ignored = Vec::new();
    std::io::stdin().read_to_end(&mut ignored).unwrap();
    drop(read);
    drop(usage);
}

#[test]
fn killing_passive_reader_releases_only_its_kernel_guard() {
    killed_reader(false);
}

#[test]
fn killing_promoted_reader_preserves_durable_custody() {
    killed_reader(true);
}

fn killed_reader(promoted: bool) {
    let (fixture, tools) = crate::generation::tests::installed_fixture();
    let ready = fixture.path().join("reader-ready");
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", CHILD, "--nocapture"])
        .env(TOOLS, &tools)
        .env(READY, &ready)
        .env_remove(PROMOTED)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null());
    if promoted {
        command.env(PROMOTED, "1");
    }
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    let observed = if ready.exists() {
        Some(GenerationUse::inspect(&tools, "generation.fixture"))
    } else {
        None
    };
    let killed = child.kill();
    let waited = child.wait();
    killed.unwrap();
    waited.unwrap();
    assert_eq!(observed.unwrap().unwrap(), GenerationUseState::Active);
    assert_eq!(claims(&tools), usize::from(promoted));
    assert_eq!(
        GenerationUse::inspect(&tools, "generation.fixture").unwrap(),
        if promoted {
            GenerationUseState::Unsettled
        } else {
            GenerationUseState::Available
        }
    );
}

#[test]
fn passive_guard_excludes_generation_removal_without_holding_publication_lock() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let read = GenerationReadGuard::acquire(&tools, "generation.fixture").unwrap();
    let update = owned_file(&tools.join(".update.lock"), false).unwrap();
    rustix::fs::flock(
        &update,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    )
    .unwrap();
    let generation = owned_file(&tools.join(".uses/generation.fixture.lock"), false).unwrap();
    assert_eq!(
        rustix::fs::flock(
            &generation,
            rustix::fs::FlockOperation::NonBlockingLockExclusive
        ),
        Err(rustix::io::Errno::WOULDBLOCK)
    );
    drop(read);
    rustix::fs::flock(
        &generation,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    )
    .unwrap();
}
