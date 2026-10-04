// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dependencies float; a pin carries a reason (ADR-0159).
//!
//! A caret or a plain `>=` floor needs nothing: `Cargo.lock` records what resolved. A
//! requirement that bounds a version from above (`=`, `<`, `<=`, `~`, a wildcard) or a git
//! source is a pin. A pin is allowed only for a member of a family in
//! `[workspace.metadata.pse.families]` or for a dependency listed with its reason in
//! `[workspace.metadata.pse.pins]`. A family member must itself be exact at the family's
//! declared version, so a `cargo add` caret cannot slip a member out of its family.
//! Python pin reasons are comments in `pyproject.toml` and are instruction-only.
#![allow(
    clippy::expect_used,
    reason = "test reports malformed manifest contracts"
)]
mod common;
use toml::Value;

/// The kinds of declaration this check distinguishes.
#[derive(Debug, PartialEq, Eq)]
enum Declaration {
    /// An internal path dependency with no version, a caret, or a floor with no upper bound.
    Floating,
    /// An upper-bounding comparator (`=`, `<`, `<=`, `~`, a wildcard), or a git source at a
    /// full commit.
    Pinned,
    /// A git source that names no full 40-hex commit: never reproducible, never allowed.
    MovingGit,
}

fn requirement(value: &Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.get("version").and_then(Value::as_str))
}

fn bounds_from_above(comparator: &str) -> bool {
    comparator.starts_with('=')
        || comparator.starts_with('<')
        || comparator.starts_with('~')
        || comparator.contains('*')
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
    let pinned = requirement(value).is_some_and(|text| {
        text.split(',')
            .map(str::trim)
            .any(bounds_from_above)
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

/// The family the package belongs to, if any: `(name, declared version, match kind)`.
fn family_of<'v>(families: Option<&'v Value>, package: &str) -> Option<(&'v str, &'v str, &'v str)> {
    families
        .and_then(Value::as_table)
        .into_iter()
        .flat_map(|table| table.iter())
        .find_map(|(name, family)| {
            let listed = |key: &str| {
                family
                    .get(key)
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .any(|glob| glob_matches(glob, package))
            };
            (listed("crates") && !listed("exclude")).then(|| {
                let version = family.get("version").and_then(Value::as_str).unwrap_or("");
                let kind = family.get("match").and_then(Value::as_str).unwrap_or("exact");
                (name.as_str(), version, kind)
            })
        })
}

/// Is the requirement a single exact comparator at the family's declared version?
fn exact_at(requirement: Option<&str>, version: &str, kind: &str) -> bool {
    let Some(exact) = requirement.and_then(|text| text.trim().strip_prefix('=')) else {
        return false;
    };
    let exact = exact.trim();
    if exact.contains(',') || semver::Version::parse(exact).is_err() {
        return false;
    }
    match kind {
        "minor" => exact.starts_with(&format!("{version}.")),
        _ => exact == version,
    }
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
        let family = family_of(families, package);
        if let Some((family_name, version, kind)) = family {
            if !exact_at(requirement(value), version, kind) {
                out.push(format!(
                    "{name}: a member of family `{family_name}` must be exact at its declared \
                     version {version} ({kind} match); move the family as a unit"
                ));
            }
            continue;
        }
        match declaration(value) {
            Declaration::Floating => {}
            Declaration::MovingGit => {
                out.push(format!("{name}: a git source must name a full commit `rev`"));
            }
            Declaration::Pinned => {
                if !reason(name) {
                    out.push(format!(
                        "{name}: a pin (exact, capped or git) with no reason; use a caret, or \
                         add `{name} = \"<reason>\"` to [workspace.metadata.pse.pins]"
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
fn workspace_pins_carry_a_reason_and_families_stay_exact() {
    let found = violations(&common::root_manifest());
    assert!(
        found.is_empty(),
        "dependency pins without a reason, or inexact family members (ADR-0159):\n  {}",
        found.join("\n  ")
    );
}

#[test]
fn carets_float_and_pins_need_a_family_or_a_reason() {
    let manifest = |dependencies: &str| {
        toml::from_str::<Value>(&format!(
            "[workspace.dependencies]\n{dependencies}\n\
             [workspace.metadata.pse.families]\n\
             arrow = {{ version = '59.3.0', crates = ['arrow', 'arrow-*'] }}\n\
             pyo3 = {{ version = '0.29', match = 'minor', crates = ['pyo3'] }}\n\
             [workspace.metadata.pse.pins]\n\
             held = 'a recorded reason'\n"
        ))
        .expect("fixture")
    };
    // A caret, a bare path and an open floor pass; a listed pin (exact, capped, tilde or git)
    // and family members exact at their declared version pass.
    for dependencies in [
        "serde = '1.0.229'\nheld = '=1.0.0'",
        "local = { path = 'crates/local' }\nheld = '=1.0.0'",
        "serde = '>=1.0'\nheld = '=1.0.0'",
        "held = '>=1.0, <1.5'",
        "held = '~1.2.3'",
        "held = { git = 'https://example.test/repo', rev = '0123456789abcdef0123456789abcdef01234567' }",
        "arrow-array = '=59.3.0'\npyo3 = '=0.29.2'\nheld = '=1.0.0'",
    ] {
        assert_eq!(
            violations(&manifest(dependencies)),
            Vec::<String>::new(),
            "{dependencies}"
        );
    }
    // Unlisted pins fail: exact, inside a range, an upper cap, a tilde, a wildcard minor and
    // a git revision; so do a moving git branch, a caret or off-version family member, and a
    // reason left behind for a dependency that floats.
    for (dependencies, offender) in [
        ("serde = '=1.0.229'\nheld = '=1.0.0'", "serde"),
        ("serde = '=1.0.229, <2'\nheld = '=1.0.0'", "serde"),
        ("serde = '>=1.0, <2'\nheld = '=1.0.0'", "serde"),
        ("serde = '<=1.0.300'\nheld = '=1.0.0'", "serde"),
        ("serde = '~1.0.229'\nheld = '=1.0.0'", "serde"),
        ("serde = '1.0.*'\nheld = '=1.0.0'", "serde"),
        (
            "fork = { git = 'https://example.test/repo', rev = '0123456789abcdef0123456789abcdef01234567' }\nheld = '=1.0.0'",
            "fork",
        ),
        (
            "held = { git = 'https://example.test/repo', branch = 'main' }",
            "held",
        ),
        ("arrow-json = '59.3.0'\nheld = '=1.0.0'", "arrow-json"),
        ("arrow = '=59.2.0'\nheld = '=1.0.0'", "arrow"),
        ("pyo3 = '=0.30.0'\nheld = '=1.0.0'", "pyo3"),
        ("held = '1.0.0'", "held"),
    ] {
        let found = violations(&manifest(dependencies));
        assert!(
            found.iter().any(|line| line.starts_with(&format!("{offender}:"))),
            "{dependencies}: {found:?}"
        );
    }
}
