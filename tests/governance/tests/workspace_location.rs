// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "disposable filesystem fixtures report setup failures directly"
)]

//! Root discovery must follow the runtime checkout when build artifacts are reused.

mod common;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "pse-workspace-location-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("unique temporary directory");
        Self(path)
    }

    fn checkout(&self, name: &str) -> PathBuf {
        let root = self.0.join(name);
        std::fs::create_dir_all(root.join("xtask/src")).unwrap();
        std::fs::create_dir_all(root.join("tests/governance/tests")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "[workspace.metadata.pse]\n").unwrap();
        std::fs::write(root.join("xtask/Cargo.toml"), "[package]\nname = 'xtask'\n").unwrap();
        std::fs::write(
            root.join("tests/governance/Cargo.toml"),
            "[package]\nname = 'pse-tests-governance'\n",
        )
        .unwrap();
        root
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove disposable fixture");
    }
}

#[test]
fn relocated_checkout_is_discovered_after_original_is_removed() {
    let fixture = Fixture::new();
    let original = fixture.checkout("original");
    let moved = fixture.0.join("moved checkout");
    std::fs::rename(&original, &moved).unwrap();
    let canonical = moved.canonicalize().unwrap();
    for relative in ["", "xtask/src", "tests/governance/tests"] {
        assert_eq!(
            common::workspace::find_workspace_root(&moved.join(relative)).unwrap(),
            canonical
        );
    }
    assert!(common::workspace::find_workspace_root(&original).is_err());
}

#[test]
fn invocation_selects_its_own_checkout() {
    let fixture = Fixture::new();
    let first = fixture.checkout("first");
    let second = fixture.checkout("second");
    assert_eq!(
        common::workspace::find_workspace_root(&first.join("xtask")).unwrap(),
        first.canonicalize().unwrap()
    );
    assert_eq!(
        common::workspace::find_workspace_root(&second.join("xtask")).unwrap(),
        second.canonicalize().unwrap()
    );
}

#[test]
fn unrelated_or_incomplete_workspaces_fail_closed() {
    let fixture = Fixture::new();
    std::fs::write(fixture.0.join("Cargo.toml"), "[workspace]\n").unwrap();
    assert!(common::workspace::find_workspace_root(&fixture.0).is_err());
    let root = fixture.checkout("incomplete");
    std::fs::write(root.join("xtask/Cargo.toml"), "[package]\nname = 'wrong'\n").unwrap();
    assert!(
        common::workspace::find_workspace_root(&root)
            .unwrap_err()
            .contains("must declare package xtask")
    );
    std::fs::remove_file(root.join("tests/governance/Cargo.toml")).unwrap();
    assert!(common::workspace::find_workspace_root(&root).is_err());
}
