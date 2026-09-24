// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Every `extern "C" fn` in the FFI crates catches unwinds in its own body.
//!
//! An unwind across the Ipopt (or CoolProp) C frames is undefined behaviour, and the
//! release profile is `panic = "unwind"` because PyO3 requires it — so `abort` is not
//! doing this for us. The rule is per callback, not per crate, which is why this counts
//! function bodies rather than grepping for one `catch_unwind` somewhere in the file
//! (blueprint §18.3).
//!
//! Trivially green in phase 0: the first callback written is the one this catches.

mod common;

/// Returns each `extern "C" fn` body in `text`, as `(name, body)`.
fn extern_c_bodies(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut search_from = 0;
    while let Some(found) = text[search_from..].find("extern \"C\" fn") {
        let start = search_from + found;
        search_from = start + 1;
        let after = start + "extern \"C\" fn".len();
        let name: String = text[after..]
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let Some(open_offset) = text[after..].find('{') else {
            continue;
        };
        let open = text[..after + open_offset].chars().count();
        let mut depth = 0_i32;
        let mut end = None;
        for (index, ch) in bytes.iter().enumerate().skip(open) {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(end) = end {
            out.push((name, bytes[open..=end].iter().collect()));
        }
    }
    out
}

#[test]
fn every_extern_c_callback_catches_unwind() {
    let root = common::workspace_root();
    let mut problems: Vec<String> = Vec::new();

    for crate_name in ["pse-backend-native"] {
        let dir = root.join("crates").join(crate_name).join("src");
        for path in common::rust_sources(&dir) {
            if common::is_generated(&path) {
                continue;
            }
            let code = common::code_only_file(&path);
            for (name, body) in extern_c_bodies(&code) {
                if !body.contains("catch_unwind") {
                    problems.push(format!(
                        "{}: `extern \"C\" fn {name}` has no catch_unwind in its body",
                        common::rel(&path)
                    ));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "FFI callbacks that can unwind into C:\n  {}\n\
         Wrap the body in std::panic::catch_unwind and convert the payload into the \
         callback's error return (blueprint §18.3).",
        problems.join("\n  ")
    );
}
