// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parse-only tooling, separate from the server's capability-gated parser entry.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Serialize;

const SCHEMA: &str = "surrealql-validation-v1";

#[derive(Debug, Serialize)]
struct ParserInfo {
    library: &'static str,
    version: String,
    syntax_profile: &'static str,
    scope: &'static str,
}

#[derive(Debug, Serialize)]
struct ValidationError {
    kind: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct FileValidation {
    input: PathBuf,
    valid: bool,
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
    let mut packages = lock.package.into_iter().filter(|package| package.name == "surrealdb-syn");
    let package = packages.next().context("compiled lockfile lacks surrealdb-syn")?;
    ensure!(packages.next().is_none(), "compiled lockfile has multiple surrealdb-syn versions");
    Ok(ParserInfo {
        library: "surrealdb-syn",
        version: package.version,
        syntax_profile: "all-features",
        scope: "parse-only",
    })
}

fn parse_error(source: &str) -> Option<ValidationError> {
    surrealdb_syn::parse(source).err().map(|error| ValidationError {
        kind: "syntax",
        message: error.to_string(),
    })
}

fn validate_file(input: &Path) -> FileValidation {
    let error = match std::fs::read_to_string(input) {
        Ok(source) => parse_error(&source),
        Err(error) => Some(ValidationError { kind: "input", message: error.to_string() }),
    };
    FileValidation { input: input.to_path_buf(), valid: error.is_none(), error }
}

pub(crate) fn run(inputs: &[PathBuf]) -> Result<i32> {
    let files: Vec<_> = inputs.iter().map(|input| validate_file(input)).collect();
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
        assert!(parse_error("DEFINE MODULE mod::probe FROM silo::example::probe::1.0.0 UNSIGNED;").is_none());
        assert!(parse_error("RETURN 1;").is_none());
        let info = parser_info().unwrap();
        assert_eq!(info.library, "surrealdb-syn");
        assert_eq!(info.syntax_profile, "all-features");
        assert_eq!(info.scope, "parse-only");
        assert!(!info.version.is_empty());
    }

    #[test]
    fn malformed_module_and_junk_keep_library_diagnostics() {
        for source in ["DEFINE MODULE mod::probe FROM ;", "RETURN (; junk"] {
            let error = parse_error(source).expect("malformed source must fail");
            assert_eq!(error.kind, "syntax");
            assert!(error.message.starts_with("Parse error:"));
        }
    }

    #[test]
    fn each_file_retains_its_own_outcome_and_input_error() {
        let directory = tempfile::tempdir().unwrap();
        let valid = directory.path().join("valid.surql");
        let invalid = directory.path().join("invalid.surql");
        std::fs::write(&valid, "RETURN 1;").unwrap();
        std::fs::write(&invalid, "RETURN (;").unwrap();
        assert!(validate_file(&valid).valid);
        let rejected = validate_file(&invalid);
        assert_eq!(rejected.input, invalid);
        assert!(!rejected.valid);
        assert_eq!(rejected.error.unwrap().kind, "syntax");
        let missing = validate_file(&directory.path().join("missing.surql"));
        assert!(!missing.valid);
        assert_eq!(missing.error.unwrap().kind, "input");
    }
}
