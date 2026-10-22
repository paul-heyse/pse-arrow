// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! No hand-written struct shadows a generated relation view.
//!
//! Blueprint §24.1: "no hand-written struct shadows a relation". Two definitions of the
//! same row shape is how a contract silently forks — one of them gets a field, the other
//! does not, and both compile.
//!
//! A missing or empty generated tree is a failure, never a vacuous pass.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use syn::visit::Visit;

#[derive(Default)]
struct PublicStructs(BTreeSet<String>);
impl<'ast> Visit<'ast> for PublicStructs {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if !matches!(item.vis, syn::Visibility::Inherited) {
            self.0.insert(item.ident.to_string());
        }
        syn::visit::visit_item_struct(self, item);
    }
}
fn public_structs(source: &str) -> BTreeSet<String> {
    let mut declarations = PublicStructs::default();
    declarations.visit_file(&syn::parse_file(source).expect("valid Rust source"));
    declarations.0
}

#[test]
fn generator_text_is_not_a_struct_declaration() {
    let source = r##"
        pub struct Actual;
        mod nested { pub(crate) struct Scoped; struct Private; }
        const GENERATED: &str = r#"
            pub struct Generated;
        "#;
        // pub struct Commented;
    "##;
    assert_eq!(
        public_structs(source),
        BTreeSet::from(["Actual".into(), "Scoped".into()])
    );
}

#[test]
fn no_hand_written_struct_shadows_a_generated_one() {
    let root = common::workspace_root();
    let mut generated: BTreeMap<String, String> = BTreeMap::new();
    for directory in ["pse-model", "pse-relations"] {
        let directory = root.join(format!("crates/{directory}/src/generated"));
        assert!(
            directory.is_dir(),
            "missing generated root: {}",
            directory.display()
        );
        let mut count = 0;
        for path in common::rust_sources(&directory) {
            for name in public_structs(&common::read(&path)) {
                generated.insert(name, common::rel(&path));
                count += 1;
            }
        }
        assert!(count > 0, "empty generated root: {}", directory.display());
    }

    let mut problems: Vec<String> = Vec::new();
    for path in common::crate_sources() {
        if common::rel(&path).contains("/src/generated/") {
            continue;
        }
        for name in public_structs(&common::read(&path)) {
            if let Some(origin) = generated.get(&name) {
                problems.push(format!(
                    "{}: `pub struct {name}` shadows the generated one in {origin}",
                    common::rel(&path)
                ));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "hand-written structs shadowing generated relation views:\n  {}\n\
         Use the generated view, or change the registry and regenerate.",
        problems.join("\n  ")
    );
}
