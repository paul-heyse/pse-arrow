// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! `unsafe` appears only in the crates `[workspace.metadata.pse].unsafe-allowlist` names,
//! and each of those carries a reasoned `allow(unsafe_code)` at its required scope.
//!
//! `unsafe_code = "deny"` is a workspace lint, so the compiler already stops an accidental
//! `unsafe` block. What the compiler cannot stop is a crate quietly adding a scoped
//! allow to get past it. This test makes that a governance failure instead, and is the
//! trigger condition for re-adopting Miri (docs/adr/register.md): today the only `unsafe`
//! in the workspace is FFI.

mod common;

use std::collections::BTreeSet;

use regex::Regex;
use toml::Value;

fn allowlist() -> BTreeSet<String> {
    let manifest = common::root_manifest();
    common::dig(
        &manifest,
        &["workspace", "metadata", "pse", "unsafe-allowlist"],
    )
    .and_then(Value::as_array)
    .expect("[workspace.metadata.pse] unsafe-allowlist")
    .iter()
    .map(|v| {
        v.as_str()
            .expect("allowlist entries are strings")
            .to_owned()
    })
    .collect()
}

#[test]
fn unsafe_appears_only_in_allowlisted_crates() {
    let allowed = allowlist();
    let re = Regex::new(r"\bunsafe\b").expect("static regex");
    let mut problems: Vec<String> = Vec::new();

    for (name, dir) in common::crate_dirs() {
        if allowed.contains(&name) {
            continue;
        }
        for path in common::rust_sources(&dir.join("src")) {
            for (number, line) in common::read(&path).lines().enumerate() {
                if re.is_match(&common::code_only(line)) {
                    problems.push(format!("{}:{}", common::rel(&path), number + 1));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "`unsafe` outside the unsafe-allowlist:\n  {}\n\
         Adding a crate to the allowlist is an ADR, and re-enables Miri in the scheduled \
         workflow (register row).",
        problems.join("\n  ")
    );
}

#[derive(Default)]
struct Allowances {
    count: usize,
    missing_reason: usize,
}
impl<'ast> syn::visit::Visit<'ast> for Allowances {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if attr.path().is_ident("allow") {
            let entries = attr
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )
                .expect("valid lint attribute");
            if entries
                .iter()
                .any(|m| matches!(m, syn::Meta::Path(p) if p.is_ident("unsafe_code")))
            {
                self.count += 1;
                let reason = entries.iter().any(|m| {
                    matches!(m, syn::Meta::NameValue(v)
                    if v.path.is_ident("reason") && matches!(&v.value, syn::Expr::Lit(l)
                        if matches!(&l.lit, syn::Lit::Str(s) if !s.value().trim().is_empty())))
                });
                self.missing_reason += usize::from(!reason);
            }
        }
        syn::visit::visit_attribute(self, attr);
    }
}

#[test]
fn allowlisted_crates_declare_the_allow_with_a_reason() {
    use syn::visit::Visit;
    let allowed = allowlist();
    let mut found = BTreeSet::new();
    for (name, dir) in common::crate_dirs() {
        let mut allowances = Allowances::default();
        for path in common::rust_sources(&dir.join("src")) {
            allowances.visit_file(&syn::parse_file(&common::read(&path)).unwrap());
        }
        assert_eq!(
            allowances.missing_reason, 0,
            "{name}: unsafe allowances require nonempty reasons"
        );
        if allowances.count > 0 {
            assert!(
                allowed.contains(&name),
                "{name}: unsafe allowance outside allowlist"
            );
            found.insert(name);
        }
    }
    assert_eq!(
        found, allowed,
        "every allowlisted crate must have a reasoned unsafe allowance"
    );
}

#[test]
fn narrow_module_allowances_require_actual_nonempty_reasons() {
    use syn::visit::Visit;
    let source = r#"
        mod ffi { #![allow(unsafe_code, reason = "FFI boundary")] }
        mod missing { #![allow(unsafe_code)] }
        mod empty { #![allow(unsafe_code, reason = " ")] }
        // #![allow(unsafe_code, reason = "not an attribute")]
    "#;
    let mut allowances = Allowances::default();
    allowances.visit_file(&syn::parse_file(source).unwrap());
    assert_eq!(allowances.count, 3);
    assert_eq!(allowances.missing_reason, 2);
}
