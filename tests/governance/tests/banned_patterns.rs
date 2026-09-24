// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! The regex layer of the blueprint §24.1 governance greps.
//!
//! `clippy.toml`'s `disallowed-methods` is the type-aware layer and catches the same calls
//! with full paths; this one catches them in macro arguments, in strings passed to a
//! builder, and in code that has not compiled yet. Two layers, because each misses what
//! the other sees.
//!
//! Generated sources are excluded: the fix for a violation there is the generator, and
//! `cargo xtask codegen --check` guards them.

mod common;

use std::path::{Path, PathBuf};

use regex::Regex;

/// A ban: the pattern, where it applies (empty = everywhere scanned), and why.
struct Ban {
    pattern: &'static str,
    scopes: &'static [&'static str],
    reason: &'static str,
    literals: bool,
}

const BANS: &[Ban] = &[
    Ban {
        pattern: r"Field::extension_type\(",
        literals: false,
        scopes: &[],
        reason: "panics on a missing or invalid extension; use `try_extension_type` (blueprint §4.4)",
    },
    Ban {
        pattern: r"SessionConfig::set_str",
        literals: false,
        scopes: &[],
        reason: "panics on an invalid key; use typed ConfigOptions (blueprint §23.2, `config.invalid`)",
    },
    Ban {
        pattern: r"SchemaLike::from_type",
        literals: false,
        scopes: &[],
        reason: "schemas come from the registry, never inferred (blueprint §5.3)",
    },
    Ban {
        pattern: r"from_samples\(",
        literals: false,
        scopes: &[],
        reason: "schemas come from the registry, never inferred from data (blueprint §5.3)",
    },
    Ban {
        pattern: r#""SERDE_ARROW:"#,
        literals: true,
        scopes: &[],
        reason: "a serde_arrow field-metadata key on a platform relation is a contract violation (blueprint §5.3)",
    },
    Ban {
        pattern: r"config_options\(",
        literals: false,
        scopes: &["crates/pse-kernels"],
        reason: "a kernel that reads engine configuration is not a pure function of its inputs (blueprint §9, §24.1)",
    },
    Ban {
        pattern: r"pretty_format",
        literals: false,
        scopes: &["crates/pse-ids", "crates/pse-catalog"],
        reason: "a rendered table is never an identity or manifest input (blueprint §5.3)",
    },
    Ban {
        pattern: r"try_with_compression",
        literals: false,
        scopes: &["crates/pse-columnar/src/canon"],
        reason: "canonical identity IPC is uncompressed; transport may use compression (blueprint §5.3, §3.3.1, ADR-0065)",
    },
    Ban {
        pattern: r#"format\s*!\s*\(\s*(?:r#*)?"[^"\n]*\{[^{}]*:#?\?"#,
        literals: true,
        scopes: &["crates/pse-ids"],
        reason: "a Debug rendering is not a stable hash input; hash canonical bytes (blueprint §5.3)",
    },
];

/// Blank non-code regions while retaining byte offsets and newlines. Literal-aware
/// bans retain actual string contents; call-shape bans cannot fire on quoted prose.
fn source_view(source: &str, keep_literals: bool) -> String {
    let bytes = source.as_bytes();
    let mut view = bytes.to_vec();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let start = cursor;
        let (end, mask) = if bytes[start..].starts_with(b"//") {
            (
                source[start..]
                    .find('\n')
                    .map_or(bytes.len(), |end| start + end),
                true,
            )
        } else if bytes[start..].starts_with(b"/*") {
            let mut depth = 1;
            cursor += 2;
            while cursor < bytes.len() && depth > 0 {
                if bytes[cursor..].starts_with(b"/*") {
                    depth += 1;
                    cursor += 2;
                } else if bytes[cursor..].starts_with(b"*/") {
                    depth -= 1;
                    cursor += 2;
                } else {
                    cursor += 1;
                }
            }
            (cursor, true)
        } else if let Some(end) = literal_end(source, cursor) {
            (end, !keep_literals)
        } else {
            cursor += source[cursor..].chars().next().unwrap().len_utf8();
            continue;
        };
        if mask {
            for byte in &mut view[start..end] {
                if *byte != b'\n' && *byte != b'\r' {
                    *byte = b' ';
                }
            }
        }
        cursor = end;
    }
    String::from_utf8(view).expect("masking complete source tokens preserves UTF-8")
}

fn literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes[start] == b'r' {
        let hashes = bytes[start + 1..]
            .iter()
            .take_while(|byte| **byte == b'#')
            .count();
        let opening = start + 1 + hashes;
        if bytes.get(opening) == Some(&b'"') {
            let closing = format!("\"{}", "#".repeat(hashes));
            return Some(
                source[opening + 1..]
                    .find(&closing)
                    .map_or(bytes.len(), |end| opening + 1 + end + closing.len()),
            );
        }
    }
    if !matches!(bytes[start], b'"' | b'\'') {
        return None;
    }
    let delimiter = bytes[start];
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == delimiter {
            return Some(cursor + 1);
        }
        if delimiter == b'\'' && matches!(bytes[cursor], b'\n' | b'\r' | b' ') {
            return None;
        }
        if bytes[cursor] == b'\\' {
            cursor += 1;
        }
        cursor += 1;
        // An unescaped character has exactly one scalar before its closing quote.
        // This prevents a lifetime from consuming code until another apostrophe.
        if delimiter == b'\''
            && bytes.get(start + 1) != Some(&b'\\')
            && cursor > start + 1 + source[start + 1..].chars().next()?.len_utf8()
        {
            return None;
        }
    }
    (delimiter == b'"').then_some(bytes.len())
}

fn matches(ban: &Ban, source: &str) -> Vec<usize> {
    let production = production_view(source);
    let view = source_view(&production, ban.literals);
    let code = source_view(&production, false);
    Regex::new(ban.pattern)
        .expect("static regex")
        .find_iter(&view)
        // The debug-format ban begins with a code token, not quoted example code.
        .filter(|found| {
            !found.as_str().starts_with("format")
                || code[found.start()..].starts_with("format")
        })
        .map(|found| source[..found.start()].bytes().filter(|byte| *byte == b'\n').count() + 1)
        .collect()
}

/// Invalid-input fixtures belong in explicitly test-only modules. Preserve offsets so
/// production diagnostics still point at the original source, including after a module.
fn production_view(source: &str) -> String {
    let code = source_view(source, false);
    let module = Regex::new(
        r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?mod\s+\w+\s*\{",
    )
    .expect("test-module regex");
    let mut view = source.as_bytes().to_vec();
    for found in module.find_iter(&code) {
        let mut depth = 1;
        let mut end = found.end();
        while end < code.len() && depth > 0 {
            match code.as_bytes()[end] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        for byte in &mut view[found.start()..end] {
            if *byte != b'\n' && *byte != b'\r' {
                *byte = b' ';
            }
        }
    }
    String::from_utf8(view).expect("masking complete source modules preserves UTF-8")
}

/// `crates/*/src` plus `xtask/src`: the code that is allowed to exist today.
fn scanned_files() -> Vec<PathBuf> {
    let mut files = common::crate_sources();
    files.extend(common::rust_sources(
        &common::workspace_root().join("xtask/src"),
    ));
    files
}

/// Does `path` fall inside one of a ban's scopes?
fn in_scope(path: &Path, scopes: &[&str]) -> bool {
    if scopes.is_empty() {
        return true;
    }
    let rel = common::rel(path);
    scopes
        .iter()
        .any(|scope| rel.starts_with(&format!("{scope}/")))
}

#[test]
fn no_banned_call_shapes() {
    let files = scanned_files();
    assert!(
        !files.is_empty(),
        "the scan found no sources; the paths are wrong"
    );

    let mut problems: Vec<String> = Vec::new();
    for ban in BANS {
        for path in &files {
            if !in_scope(path, ban.scopes) {
                continue;
            }
            for number in matches(ban, &common::read(path)) {
                problems.push(format!(
                    "{}:{}: `{}` — {}",
                    common::rel(path),
                    number,
                    ban.pattern,
                    ban.reason
                ));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "banned patterns:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn string_sensitive_bans_reject_real_metadata_and_debug_inputs() {
    let metadata = &BANS[4];
    let debug = &BANS[8];
    for source in [
        r#"metadata.insert("SERDE_ARROW:strategy", value);"#,
        r##"metadata.insert(r#"SERDE_ARROW:strategy"#, value);"##,
        "let url = \"https://example.test\"; let key = \"SERDE_ARROW:strategy\";",
    ] {
        assert_eq!(matches(metadata, source), [1], "{source}");
    }
    for source in [
        r#"format!("{:?}", value)"#,
        r##"format!(r#"{value:#?}"#)"##,
        "format! (\n    \"{value:?}\"\n)",
    ] {
        assert_eq!(matches(debug, source), [1], "{source}");
    }
}

#[test]
fn comments_and_quoted_examples_do_not_trigger_call_bans() {
    let source = r##"
        // SERDE_ARROW:strategy format!("{:?}", value)
        /* nested /* SERDE_ARROW:strategy */ format!("{:?}", value) */
        let prose = r#"format!("{:?}", value) Field::extension_type()"#;
        let quote = '"';
        let lifetime: &'a str = "SchemaLike::from_type";
        let url = "https://example.test";
        let diagnostic = "`SERDE_ARROW:*` keys are invalid";
    "##;
    for ban in BANS {
        assert!(matches(ban, source).is_empty(), "{}", ban.pattern);
    }
    assert_eq!(
        matches(
            &BANS[0],
            "// Field::extension_type()\nField::extension_type()"
        ),
        [2]
    );
}

#[test]
fn test_fixture_exclusion_does_not_hide_following_production_code() {
    let source = r#"
        #[cfg(test)]
        mod tests {
            fn bad_metadata() { insert("SERDE_ARROW:strategy"); }
        }
        fn production() { insert("SERDE_ARROW:strategy"); }
    "#;
    assert_eq!(matches(&BANS[4], source), [6]);
}
