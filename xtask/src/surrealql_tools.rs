// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parse-only tooling, separate from the server's capability-gated parser entry.

use std::{
    fs::{self, File},
    io::Read as _,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use surrealdb_parser_next::DiagKind;

const SCHEMA: &str = "surrealql-validation-v1";
const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_DIAGNOSTICS: usize = 16;

#[derive(Debug, Serialize)]
struct ParserInfo {
    library: &'static str,
    version: String,
    syntax_profile: &'static str,
    scope: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
struct ValidationError {
    kind: String,
    message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    diagnostics: Vec<ParseDiagnostic>,
    #[serde(default, skip_serializing_if = "is_zero")]
    omitted_diagnostics: usize,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

#[derive(Debug, Deserialize, Serialize)]
struct ParseDiagnostic {
    kind: String,
    message: String,
    primary: Label,
    secondary: Option<Label>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Label {
    label: Option<String>,
    primary: bool,
    byte_offset: usize,
    byte_length: usize,
    line: Option<usize>,
    column: Option<usize>,
    column_encoding: String,
    unavailable: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct FileValidation {
    input: PathBuf,
    sha256: Option<String>,
    pub(crate) valid: bool,
    error: Option<ValidationError>,
}

#[derive(Debug, Serialize)]
struct Validation {
    schema: &'static str,
    parser: ParserInfo,
    valid: bool,
    files: Vec<FileValidation>,
}

fn parser_info() -> Result<ParserInfo> {
    // Retain the resolved metadata compiled into this selected binary, rather than
    // reading whichever checkout happens to be present at execution time.
    let lock: crate::Lockfile = toml::from_str(include_str!("../../Cargo.lock"))
        .context("reading compiled parser dependency metadata")?;
    let mut packages = lock
        .package
        .into_iter()
        .filter(|package| package.name == "surrealdb-parser-next");
    let package = packages
        .next()
        .context("compiled lockfile lacks surrealdb-parser-next")?;
    ensure!(
        packages.next().is_none(),
        "compiled lockfile has multiple surrealdb-parser-next versions"
    );
    Ok(ParserInfo {
        library: "surrealdb-parser-next",
        version: package.version,
        syntax_profile: "default-limits",
        scope: "parse-only",
    })
}

fn parse_error(source: &str) -> Option<ValidationError> {
    let (_, diagnostics) = surrealdb_parser_next::parse(source);
    if diagnostics.is_empty() {
        return None;
    }
    let starts: Vec<_> = std::iter::once(0)
        .chain(source.match_indices('\n').map(|(offset, _)| offset + 1))
        .collect();
    let span = |start: u32, end: u32, label: &str, primary| {
        let start = start as usize;
        let end = end as usize;
        let line = source
            .get(start..end)
            .map(|_| starts.partition_point(|offset| *offset <= start) - 1);
        Label {
            label: (!label.is_empty()).then(|| label.to_owned()),
            primary,
            byte_offset: start,
            byte_length: end.saturating_sub(start),
            line,
            column: line.map(|line| start - starts[line]),
            column_encoding: "utf8_bytes".into(),
            unavailable: line
                .is_none()
                .then(|| "parser range is outside the UTF-8 source".into()),
        }
    };
    Some(ValidationError {
        kind: "syntax".into(),
        message: "SurrealQL parsing failed".into(),
        omitted_diagnostics: diagnostics.len().saturating_sub(MAX_DIAGNOSTICS),
        diagnostics: diagnostics
            .into_iter()
            .take(MAX_DIAGNOSTICS)
            .map(|diagnostic| {
                // Keep native templates and labels without retaining any source excerpts.
                let (kind, message) = match diagnostic.kind {
                    DiagKind::Unexpected => (
                        "unexpected",
                        format!("Unexpected token, expected `{}`", diagnostic.msg),
                    ),
                    DiagKind::Message => ("message", diagnostic.msg.to_owned()),
                };
                ParseDiagnostic {
                    kind: kind.into(),
                    message,
                    primary: span(diagnostic.start, diagnostic.end, diagnostic.label, true),
                    secondary: diagnostic
                        .extra
                        .map(|extra| span(extra.start, extra.end, extra.label, false)),
                }
            })
            .collect(),
    })
}

/// Validate the exact selected bytes. A staged file's storage path stays private.
pub(crate) fn validate_file(input: &Path, stored: &Path, expected: Option<&str>) -> FileValidation {
    let mut sha256 = None;
    let result = (|| -> Result<Option<ValidationError>> {
        ensure!(
            fs::metadata(stored)?.is_file(),
            "SurrealQL input is not a regular file"
        );
        let mut file = File::open(stored)?;
        let mut bytes = Vec::new();
        file.by_ref()
            .take(MAX_INPUT_BYTES + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 <= MAX_INPUT_BYTES,
            "SurrealQL input exceeds its byte limit"
        );
        sha256 = Some(
            Sha256::digest(&bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        );
        ensure!(
            expected.is_none() || expected == sha256.as_deref(),
            "SurrealQL input digest does not match the staged postimage"
        );
        Ok(parse_error(std::str::from_utf8(&bytes)?))
    })();
    let error = result.unwrap_or_else(|error| {
        Some(ValidationError {
            kind: "input".into(),
            message: error.to_string(),
            diagnostics: Vec::new(),
            omitted_diagnostics: 0,
        })
    });
    FileValidation {
        input: input.to_path_buf(),
        sha256,
        valid: error.is_none(),
        error,
    }
}

pub(crate) fn run(inputs: &[PathBuf]) -> Result<i32> {
    let files: Vec<_> = inputs
        .iter()
        .map(|input| validate_file(input, input, None))
        .collect();
    let report = Validation {
        schema: SCHEMA,
        parser: parser_info()?,
        valid: files.iter().all(|file| file.valid),
        files,
    };
    println!("{}", serde_json::to_string(&report)?);
    Ok(i32::from(!report.valid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_surrealism_module_without_loading_its_archive() {
        let source = r#"DEFINE MODULE mod::probe FROM f"fixture:/probe.surli" UNSIGNED; RETURN mod::probe::ping();"#;
        assert!(parse_error(source).is_none(), "{:?}", parse_error(source));
        assert!(parse_error("RETURN 1;").is_none());
        let info = parser_info().unwrap();
        assert_eq!(info.library, "surrealdb-parser-next");
        assert_eq!(info.version, "4.0.0");
        assert_eq!(info.syntax_profile, "default-limits");
        assert_eq!(info.scope, "parse-only");
        assert!(!info.version.is_empty());
    }

    #[test]
    fn malformed_module_and_junk_keep_library_diagnostics() {
        for source in ["DEFINE MODULE mod::probe FROM ;", "RETURN (; junk"] {
            let error = parse_error(source).expect("malformed source must fail");
            assert_eq!(error.kind, "syntax");
            assert!(!error.message.is_empty());
        }
    }

    #[test]
    fn each_file_retains_its_own_outcome_and_input_error() {
        let directory = tempfile::tempdir().unwrap();
        let valid = directory.path().join("valid.surql");
        let invalid = directory.path().join("invalid.surql");
        fs::write(&valid, "RETURN 1;").unwrap();
        fs::write(&invalid, "RETURN (;").unwrap();
        assert!(validate_file(&valid, &valid, None).valid);
        let rejected = validate_file(&invalid, &invalid, None);
        assert_eq!(rejected.input, invalid);
        assert!(!rejected.valid);
        assert_eq!(rejected.error.unwrap().kind, "syntax");
        let missing = validate_file(&invalid, &directory.path().join("missing.surql"), None);
        assert!(!missing.valid);
        assert_eq!(missing.error.unwrap().kind, "input");
        assert!(missing.sha256.is_none());
    }

    #[test]
    fn parser_labels_preserve_utf8_and_the_original_delimiter() {
        let error = parse_error("RETURN 'μ';\nRETURN ('水' ;").unwrap();
        let [diagnostic] = error.diagnostics.as_slice() else {
            panic!("{error:?}");
        };
        assert_eq!(diagnostic.kind, "message");
        let primary = &diagnostic.primary;
        assert_eq!((primary.byte_offset, primary.byte_length), (27, 1));
        assert_eq!((primary.line, primary.column), (Some(1), Some(14)));
        let secondary = diagnostic.secondary.as_ref().unwrap();
        assert_eq!((secondary.byte_offset, secondary.byte_length), (20, 1));
        assert_eq!((secondary.line, secondary.column), (Some(1), Some(7)));
        assert_eq!(
            secondary.label.as_deref(),
            Some("expected this delimiter to close")
        );
    }

    #[test]
    fn exact_staged_digest_controls_which_bytes_may_be_parsed() {
        let directory = tempfile::tempdir().unwrap();
        let storage = directory.path().join("private-custody");
        fs::write(&storage, "RETURN 1;").unwrap();
        let logical = Path::new("generated/canonical.surql");
        let invalid = validate_file(logical, &storage, Some("wrong digest"));
        assert_eq!(invalid.input, logical);
        assert_eq!(invalid.error.unwrap().kind, "input");
        let digest = invalid.sha256.unwrap();
        let exact = validate_file(logical, &storage, Some(&digest));
        assert!(exact.valid);
        assert_eq!(exact.sha256.as_deref(), Some(digest.as_str()));
    }
}
