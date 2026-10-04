// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dependencies float; an exact requirement or git revision carries a reason (ADR-0159).
//!
//! A caret or range requirement needs nothing: `Cargo.lock` records what resolved. An exact
//! `=` comparator or a git source is a pin, and a pin is allowed only for a member of a
//! family in `[workspace.metadata.pse.families]` or for a dependency listed with its reason
//! in `[workspace.metadata.pse.pins]`.
#![allow(
    clippy::expect_used,
    reason = "test reports malformed manifest contracts"
)]
mod common;
use toml::Value;

/// The kinds of declaration this check distinguishes.
#[derive(Debug, PartialEq, Eq)]
enum Declaration {
    /// An internal path dependency with no version, or a caret/range requirement.
    Floating,
    /// An exact `=` comparator, or a git source at a full commit.
    Pinned,
    /// A git source that names no full 40-hex commit: never reproducible, never allowed.
    MovingGit,
}

fn declaration(value: &Value) -> Declaration {
    if value.get("git").is_some() {
        let full_rev = value
            .get("rev")
            .and_then(Value::as_str)
            .is_some_and(|rev| rev.len() == 40 && rev.bytes().all(|c| c.is_ascii_hexdigit()));
        return if full_rev {
            Declaration::Pinned
        } else {
            Declaration::MovingGit
        };
    }
    let version = value
        .as_str()
        .or_else(|| value.get("version").and_then(Value::as_str));
    let pinned = version.is_some_and(|requirement| {
        requirement
            .split(',')
            .map(str::trim)
            .any(|comparator| comparator.starts_with('='))
    });
    if pinned {
        Declaration::Pinned
    } else {
        Declaration::Floating
    }
}

/// `arrow-*` style globs, as `family-check` reads them: only `*` is special.
fn glob_matches(pattern: &str, name: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == name,
        Some((prefix, rest)) => {
            name.len() >= prefix.len()
                && name.starts_with(prefix)
                && (0..=name.len() - prefix.len())
                    .any(|start| glob_matches(rest, &name[prefix.len() + start..]))
        }
    }
}

/// Is the package a declared member of any family?
fn family_member(families: Option<&Value>, package: &str) -> bool {
    families
        .and_then(Value::as_table)
        .into_iter()
        .flat_map(|table| table.values())
        .any(|family| {
            let listed = |key: &str| {
                family
                    .get(key)
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .any(|glob| glob_matches(glob, package))
            };
            listed("crates") && !listed("exclude")
        })
}

/// The dependencies whose declaration is not allowed under the policy, with why.
fn violations(manifest: &Value) -> Vec<String> {
    let dependencies = common::dig(manifest, &["workspace", "dependencies"])
        .and_then(Value::as_table)
        .expect("workspace dependency declarations");
    let families = common::dig(manifest, &["workspace", "metadata", "pse", "families"]);
    let reasons = common::dig(manifest, &["workspace", "metadata", "pse", "pins"])
        .and_then(Value::as_table);
    let reason = |name: &str| {
        reasons
            .and_then(|table| table.get(name))
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty())
    };
    let mut out = Vec::new();
    for (name, value) in dependencies {
        let package = value
            .get("package")
            .and_then(Value::as_str)
            .unwrap_or(name);
        match declaration(value) {
            Declaration::Floating => {}
            Declaration::MovingGit => {
                out.push(format!("{name}: a git source must name a full commit `rev`"));
            }
            Declaration::Pinned => {
                if !family_member(families, package) && !reason(name) {
                    out.push(format!(
                        "{name}: an exact requirement or git revision with no reason; use a \
                         caret, or add `{name} = \"<reason>\"` to [workspace.metadata.pse.pins]"
                    ));
                }
            }
        }
    }
    for name in reasons.into_iter().flat_map(|table| table.keys()) {
        let held = dependencies
            .get(name)
            .is_some_and(|value| declaration(value) == Declaration::Pinned);
        if !held {
            out.push(format!(
                "{name}: listed in [workspace.metadata.pse.pins] but not pinned; remove the entry"
            ));
        }
    }
    out
}

#[test]
fn exact_workspace_dependencies_carry_a_reason() {
    let found = violations(&common::root_manifest());
    assert!(
        found.is_empty(),
        "dependency pins without a reason (ADR-0159):\n  {}",
        found.join("\n  ")
    );
}

#[test]
fn carets_float_and_exact_pins_need_a_family_or_a_reason() {
    let manifest = |dependencies: &str| {
        toml::from_str::<Value>(&format!(
            "[workspace.dependencies]\n{dependencies}\n\
             [workspace.metadata.pse.families]\n\
             arrow = {{ version = '59.3.0', crates = ['arrow', 'arrow-*'] }}\n\
             [workspace.metadata.pse.pins]\n\
             held = 'a recorded reason'\n"
        ))
        .expect("fixture")
    };
    // A caret, a bare path and a range pass; the listed pin and the family member pass.
    for dependencies in [
        "serde = '1.0.229'\nheld = '=1.0.0'",
        "local = { path = 'crates/local' }\nheld = '=1.0.0'",
        "serde = '>=1.0, <2'\nheld = '=1.0.0'",
        "held = { git = 'https://example.test/repo', rev = '0123456789abcdef0123456789abcdef01234567' }",
        "arrow-array = '=59.3.0'\nheld = '=1.0.0'",
    ] {
        assert_eq!(
            violations(&manifest(dependencies)),
            Vec::<String>::new(),
            "{dependencies}"
        );
    }
    // An unlisted exact pin (alone or inside a range), an unlisted git revision and a
    // moving git branch fail; so does a reason left behind for a dependency that floats.
    for (dependencies, offender) in [
        ("serde = '=1.0.229'\nheld = '=1.0.0'", "serde"),
        ("serde = '=1.0.229, <2'\nheld = '=1.0.0'", "serde"),
        (
            "fork = { git = 'https://example.test/repo', rev = '0123456789abcdef0123456789abcdef01234567' }\nheld = '=1.0.0'",
            "fork",
        ),
        (
            "held = { git = 'https://example.test/repo', branch = 'main' }",
            "held",
        ),
        ("held = '1.0.0'", "held"),
    ] {
        let found = violations(&manifest(dependencies));
        assert!(
            found.iter().any(|line| line.starts_with(&format!("{offender}:"))),
            "{dependencies}: {found:?}"
        );
    }
}
