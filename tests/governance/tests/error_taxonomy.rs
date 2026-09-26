// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Every `pub enum *Error` under `crates/*/src` implements Error and
//! `miette::Diagnostic` (derive or compile-asserted typed projection), while Clippy owns erased-error type bans.
//!
//! Blueprint §23.2: every `pse-*` crate returns concrete error enums carrying the §23.2
//! class as a `#[diagnostic(code(...))]`; `miette::Result` and the graphical reporter
//! exist only in the CLI and the driver (`xtask`, which is outside this scan). An
//! `anyhow::Error` in a library erases exactly the structure a finding is made of.
//!
//! Trivially green in phase 0 — which is the point: the first error enum written is
//! checked, not the hundredth.

mod common;

fn path_is(path: &syn::Path, expected: &[&str]) -> bool {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .eq(expected.iter().copied())
}

fn derives(attrs: &[syn::Attribute], expected: &[&str]) -> bool {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .any(|a| {
            a.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|paths| paths.iter().any(|p| path_is(p, expected)))
        })
}

fn diagnostic_impl(items: &[syn::Item], name: &syn::Ident) -> bool {
    items.iter().any(|item| match item {
        syn::Item::Impl(i) => i.trait_.as_ref().is_some_and(|(path, _)| {
            i.modifiers.polarity.is_none() && path_is(path, &["miette", "Diagnostic"])
                && matches!(i.self_ty.as_ref(), syn::Type::Path(t) if t.path.is_ident(name) ||
                    (t.qself.is_none() && t.path.segments.len() == 1 && t.path.segments[0].ident == *name))
        }),
        syn::Item::Macro(m) if path_is(&m.mac.path, &["pse_diagnostics", "impl_diagnostic"])
            || path_is(&m.mac.path, &["crate", "impl_diagnostic"]) => {
            use syn::parse::Parser;
            (|input: syn::parse::ParseStream<'_>| {
                let target: syn::Ident = input.parse()?;
                let _: syn::Token![,] = input.parse()?;
                let _: proc_macro2::TokenStream = input.parse()?;
                Ok(target)
            }).parse2(m.mac.tokens.clone()).is_ok_and(|target| target == *name)
        }
        _ => false,
    })
}

fn taxonomy_problems(items: &[syn::Item], scope: &str, out: &mut Vec<String>) {
    for item in items {
        match item {
            syn::Item::Enum(e)
                if !matches!(e.vis, syn::Visibility::Inherited)
                    && e.ident.to_string().ends_with("Error") =>
            {
                for required in [["thiserror", "Error"], ["miette", "Diagnostic"]] {
                    if !(derives(&e.attrs, &required)
                        || required[0] == "miette" && diagnostic_impl(items, &e.ident))
                    {
                        out.push(format!(
                            "{scope}::{} does not implement {}::{} (blueprint §23.2)",
                            e.ident, required[0], required[1]
                        ));
                    }
                }
            }
            syn::Item::Mod(m) => {
                if let Some((_, children)) = &m.content {
                    taxonomy_problems(children, &format!("{scope}::{}", m.ident), out);
                }
            }
            _ => {}
        }
    }
}

#[test]
fn error_enums_implement_error_and_diagnostic() {
    let mut problems = Vec::new();
    for path in common::crate_sources() {
        let syntax = syn::parse_file(&common::read(&path)).unwrap();
        taxonomy_problems(&syntax.items, &common::rel(&path), &mut problems);
    }
    assert!(
        problems.is_empty(),
        "error taxonomy:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn generic_implementations_are_scoped_and_comments_are_not_implementations() {
    let source = r"
        #[derive(thiserror::Error)] pub enum GenericError<E> { Value(E) }
        impl<E: std::fmt::Debug> miette::Diagnostic for GenericError<E> {}
        mod missing {
            #[derive(thiserror::Error)] pub enum GenericError { Value }
            // impl miette::Diagnostic for GenericError {}
        }
        #[derive(thiserror::Error)] pub enum NegativeError { Value }
        impl !miette::Diagnostic for NegativeError {}
    ";
    let mut problems = Vec::new();
    taxonomy_problems(
        &syn::parse_file(source).unwrap().items,
        "fixture",
        &mut problems,
    );
    assert_eq!(problems.len(), 2);
    assert!(problems[0].contains("missing::GenericError"));
    assert!(problems[1].contains("NegativeError"));
}
