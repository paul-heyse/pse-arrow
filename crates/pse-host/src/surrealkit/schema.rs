// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Strict, coverage-aware feedback from the explicitly prepared schema compiler.
//!
//! This projection supplements the native installed-definition and extension witnesses.
//! A matching v2 digest does not establish those witnesses or application admission.

use super::{VerifiedRelease, invalid};
use crate::HostError;

/// Original schema feedback or physical child-settlement refusal.
#[derive(Debug, thiserror::Error)]
pub enum FeedbackError {
    /// Existing host selection or semantic admission refusal.
    #[error(transparent)]
    Host(#[from] HostError),
    /// Original file-system refusal.
    #[error("schema feedback I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original process ownership and settlement refusal.
    #[error(transparent)]
    Supervision(#[from] pse_diagnostics::SupervisionError),
    /// Work observation failed after the original child was successfully drained.
    #[error("schema feedback work failed after child settlement: {0}")]
    Work(#[source] pse_diagnostics::SupervisionError),
    /// Supplier errors remain inspectable after strict refusal and physical settlement.
    #[error("schema feedback refused {} supplier diagnostics", diagnostics.len())]
    Diagnostics {
        /// Bounded, credential-redacted diagnostics with source hashes.
        diagnostics: Vec<Diagnostic>,
        /// Additional supplier diagnostics existed beyond the bounded detail.
        omitted: usize,
    },
    /// Captured original child outcome, after successful physical settlement.
    #[error("schema feedback child {status}; captured diagnostics incomplete: {truncated}")]
    Command {
        /// Original status or signal.
        status: String,
        /// Bounded, credential-redacted stderr.
        stderr: String,
        /// Bounded, credential-redacted stdout.
        stdout: String,
        /// Either stream exceeded its admitted capture.
        truncated: bool,
    },
    /// Work and physical settlement can fail independently; both causes survive.
    #[error("schema feedback child settlement failed: {source}")]
    Settlement {
        /// Original failed work observation, when present.
        work: Option<Box<pse_diagnostics::SupervisionError>>,
        /// Original settlement refusal.
        #[source]
        source: Box<pse_diagnostics::SupervisionError>,
    },
}

use pse_diagnostics::SupervisedChild;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _};
use std::{fs, io::Read as _, path::Path, process::Command, time::Instant};

pse_diagnostics::impl_diagnostic! {
    FeedbackError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::HostOwnership) },
    forward(this) { match this {
        Self::Host(source) => Some(source),
        Self::Supervision(source) => Some(source),
        Self::Work(source) => Some(source),
        Self::Settlement { source, .. } => Some(source.as_ref()),
        Self::Io(_) | Self::Diagnostics { .. } | Self::Command { .. } => None,
    } },
    help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        use pse_diagnostics::{DiagnosticFacts, DiagnosticObservation};
        let mut facts = DiagnosticFacts::default();
        match this {
            Self::Diagnostics { diagnostics, omitted } => {
                for (index, diagnostic) in diagnostics.iter().enumerate() {
                    for (name, value) in [("code", diagnostic.code.clone()),
                        ("severity", diagnostic.severity.clone()), ("reason", diagnostic.reason.clone()),
                        ("context", Value::Object(diagnostic.context.clone()).to_string())] {
                        facts.observe(format!("supplier_{index}_{name}"), DiagnosticObservation::Text(value));
                    }
                }
                facts.observe("supplier_diagnostics_omitted", DiagnosticObservation::Text(omitted.to_string()));
            }
            Self::Command { status, stderr, stdout, truncated } => {
                facts.observe("child_status", DiagnosticObservation::Text(status.clone()));
                facts.observe("supplier_stderr", DiagnosticObservation::Text(stderr.clone()));
                facts.observe("supplier_stdout", DiagnosticObservation::Text(stdout.clone()));
                facts.observe("supplier_capture_incomplete", DiagnosticObservation::Boolean(*truncated));
                facts.observe("child_settled", DiagnosticObservation::Boolean(true));
            }
            _ => {},
        }
        facts
    }
}

const MAX_DOCUMENT: u64 = 32 << 20;
// Full generated schemas emit the complete body of each unsupported named type.
// Keep bounded diagnostics sized for that normal population, rather than a small case.
const MAX_TOOL_DIAGNOSTICS: usize = 4 << 20;
const COLLECTIONS: &[&str] = &[
    "tables",
    "functions",
    "params",
    "accesses",
    "analyzers",
    "sequences",
    "buckets",
    "apis",
    "modules",
    "configs",
    "users",
];

/// Original supplier diagnostic, including affected definition and source context.
#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct Diagnostic {
    /// Supplier severity; unknown severities refuse document admission.
    pub severity: String,
    /// Stable supplier diagnostic identity.
    pub code: String,
    /// Supplier's human-readable explanation.
    pub reason: String,
    /// Additional definition/source location facts preserved without interpretation.
    #[serde(flatten)]
    pub context: serde_json::Map<String, Value>,
}

/// Version-qualified supplier facts. Construction requires strict admission.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
pub struct Document {
    /// Supplier digest, independently checked against the represented content.
    pub semantic_digest: String,
    /// Engine, adapter and original compile/introspection selection.
    pub provenance: Value,
    /// Diagnostics are never hidden by digest equality.
    pub diagnostics: Vec<Diagnostic>,
    /// Host-observed prepared executable association; absent for passive file admission.
    pub preparation: Option<Preparation>,
    #[serde(skip)]
    semantic: Value,
}

/// Facts supplied by the release verifier, outside the supplier's semantic digest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, schemars::JsonSchema)]
pub struct Preparation {
    /// Pinned inputs, features and source patch identity.
    pub input_sha256: String,
    /// Verified standalone executable bytes.
    pub binary_sha256: String,
}

/// Observed represented-subset comparison, separate from complete native agreement.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
pub struct Comparison {
    /// Whether the represented semantics agree.
    pub represented_match: bool,
    /// Every v2 comparison has additional application/native witness obligations.
    pub native_witness_required: bool,
    /// Exact prepared-tool equality, or unknown for independently supplied files.
    pub prepared_tool_match: Option<bool>,
    /// JSON-pointer locations of semantic differences, bounded by the caller.
    pub differences: Vec<String>,
    /// Additional differences existed beyond the returned detail.
    pub differences_omitted: bool,
    /// Compiler engine/source and coverage observations.
    pub compiled: Document,
    /// Installed engine/source and coverage observations.
    pub installed: Document,
}

/// Explicit remote selection. Credentials never enter the serialized feedback.
pub struct LiveSelection<'a> {
    /// Caller-authorized remote gRPC endpoint.
    pub endpoint: &'a str,
    /// Selected namespace.
    pub namespace: &'a str,
    /// Selected database.
    pub database: &'a str,
    /// Prepared namespace inspection user.
    pub username: &'a str,
    /// Prepared namespace inspection credential.
    pub password: &'a str,
}

impl std::fmt::Debug for LiveSelection<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LiveSelection")
            .field("endpoint", &self.endpoint)
            .field("namespace", &self.namespace)
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}

/// Stage exact schema bytes in a newly owned directory with explicit typegen settings.
/// # Errors
/// The directory already exists or its source/configuration cannot be written.
pub fn stage(directory: &Path, schema: &[u8]) -> Result<(), FeedbackError> {
    fs::DirBuilder::new().mode(0o700).create(directory)?;
    fs::create_dir(directory.join("schema"))?;
    fs::write(directory.join("schema/generated.surql"), schema)?;
    fs::write(
        directory.join("surrealkit.toml"),
        "[typegen]\nschema_version = 2\nstrict = true\nengine = \"4.0\"\nexperimental = [\"surrealism\"]\n",
    )?;
    Ok(())
}

fn command(release: &VerifiedRelease, directory: &Path) -> Result<Command, FeedbackError> {
    if !directory.is_absolute() {
        return Err(invalid("schema feedback requires an absolute staging directory").into());
    }
    let mut command = Command::new(release.binary());
    command
        .env_clear()
        .env("HOME", directory)
        .current_dir(directory)
        .stdin(std::process::Stdio::null())
        .arg("--folder")
        .arg(directory);
    Ok(command)
}

fn execute(
    mut command: Command,
    output: &Path,
    deadline: Instant,
) -> Result<Document, FeedbackError> {
    if output.try_exists()? {
        return Err(invalid("schema feedback requires a fresh output path").into());
    }
    let work_end = deadline
        .checked_sub(std::time::Duration::from_secs(2))
        .ok_or_else(|| invalid("schema feedback work deadline unavailable"))?;
    if work_end <= Instant::now() || !output.is_absolute() {
        return Err(invalid(
            "schema feedback requires available work time and an absolute output path",
        )
        .into());
    }
    let mut environment = std::env::vars_os().collect::<crate::environment::Environment>();
    for (name, value) in command.get_envs() {
        if let Some(value) = value {
            environment.insert(name.into(), value.into());
        }
    }
    let policy = crate::envelope::RedactionPolicy::from_inputs(&environment);
    let mut child = SupervisedChild::spawn_captured(
        &mut command,
        MAX_TOOL_DIAGNOSTICS,
        MAX_TOOL_DIAGNOSTICS,
        deadline,
    )?;
    let work = child.wait_captured(None, work_end);
    // The original supervised child owns settlement, including failed work.
    let status = match child.cancel() {
        Ok(status) => status,
        Err(source) => {
            return Err(FeedbackError::Settlement {
                work: work.err().map(Box::new),
                source: Box::new(source),
            });
        }
    };
    work.map_err(FeedbackError::Work)?;
    let capture = child
        .captured_output()
        .ok_or_else(|| invalid("schema feedback capture absent"))?;
    if !status.success() || capture.stdout.overflow || capture.stderr.overflow {
        return Err(FeedbackError::Command {
            status: status.to_string(),
            stderr: policy.redact(&String::from_utf8_lossy(&capture.stderr.prefix)),
            stdout: policy.redact(&String::from_utf8_lossy(&capture.stdout.prefix)),
            truncated: capture.stdout.overflow || capture.stderr.overflow,
        });
    }
    match read_document(output) {
        Ok(mut document) => {
            sanitize(&mut document.diagnostics, &policy);
            Ok(document)
        }
        Err(FeedbackError::Diagnostics {
            mut diagnostics,
            omitted,
        }) => {
            sanitize(&mut diagnostics, &policy);
            Err(FeedbackError::Diagnostics {
                diagnostics,
                omitted,
            })
        }
        Err(error) => Err(error),
    }
}

/// Read a bounded owned supplier document without preparing tools or contacting a database.
/// # Errors
/// The file or its strict document contract cannot be admitted.
pub fn read_document(output: &Path) -> Result<Document, FeedbackError> {
    // Supplier output is ordinary caller-owned data, not a retained custody receipt.
    // Refuse links, foreign owners and nonregular replacements without imposing the
    // receipt publisher's mode on the supplier's ordinary atomic file writer.
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(output)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.uid() != rustix::process::geteuid().as_raw() {
        return Err(invalid("schema feedback requires an owned regular document").into());
    }
    if metadata.len() > MAX_DOCUMENT {
        return Err(invalid("schema feedback document exceeds admitted extent").into());
    }
    let document: Value = serde_json::from_reader(file.take(MAX_DOCUMENT + 1))
        .map_err(|_| invalid("invalid schema feedback JSON"))?;
    Document::admit(document)
}

/// Compile exact staged definitions with the prepared embedded engine.
/// # Errors
/// Compilation, physical settlement or strict version/diagnostic admission fails.
pub fn compile(
    release: &VerifiedRelease,
    directory: &Path,
    output: &Path,
    deadline: Instant,
) -> Result<Document, FeedbackError> {
    let mut command = command(release, directory)?;
    command.arg("compile").arg("--out").arg(output);
    let mut document = execute(command, output, deadline)?;
    document.preparation = Some(Preparation {
        input_sha256: release.input_sha256().into(),
        binary_sha256: release.binary_sha256().into(),
    });
    if document
        .provenance
        .pointer("/engine/mode")
        .and_then(Value::as_str)
        != Some("embedded")
        || document
            .provenance
            .pointer("/source/kind")
            .and_then(Value::as_str)
            != Some("compile")
    {
        return Err(
            invalid("schema compile provenance differs from selected embedded operation").into(),
        );
    }
    Ok(document)
}

/// Inspect an explicitly selected live schema without applying definitions or seeds.
/// # Errors
/// Inspection, settlement, strict admission or the original selection check fails.
pub fn inspect(
    release: &VerifiedRelease,
    directory: &Path,
    output: &Path,
    selection: LiveSelection<'_>,
    deadline: Instant,
) -> Result<Document, FeedbackError> {
    let mut command = command(release, directory)?;
    command
        .args([
            "--host",
            selection.endpoint,
            "--ns",
            selection.namespace,
            "--db",
            selection.database,
            "--auth-level",
            "namespace",
            "--connect-timeout-secs",
            "10",
        ])
        .args(["typegen", "--strict", "--out"])
        .arg(output)
        .env("SURREALDB_USER", selection.username)
        .env("SURREALDB_PASSWORD", selection.password);
    let mut document = execute(command, output, deadline)?;
    document.preparation = Some(Preparation {
        input_sha256: release.input_sha256().into(),
        binary_sha256: release.binary_sha256().into(),
    });
    for (pointer, expected) in [
        ("/engine/mode", "remote"),
        ("/source/kind", "introspect"),
        ("/source/namespace", selection.namespace),
        ("/source/database", selection.database),
    ] {
        if document.provenance.pointer(pointer).and_then(Value::as_str) != Some(expected) {
            return Err(
                invalid("schema inspection provenance differs from original selection").into(),
            );
        }
    }
    Ok(document)
}

impl Document {
    /// Strictly admit a v2 document, including errors the supplier may acknowledge.
    /// # Errors
    /// Required structure, selected engine, diagnostics or semantic digest is invalid.
    pub fn admit(mut document: Value) -> Result<Self, FeedbackError> {
        if document.get("version").and_then(Value::as_u64) != Some(2)
            || COLLECTIONS
                .iter()
                .any(|key| !document.get(*key).is_some_and(Value::is_array))
        {
            return Err(invalid(
                "schema feedback requires complete schema document v2 collections",
            )
            .into());
        }
        let provenance = document
            .get("provenance")
            .cloned()
            .ok_or_else(|| invalid("schema feedback provenance absent"))?;
        let version = provenance
            .pointer("/engine/version")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("schema feedback engine version absent"))?;
        if !version
            .strip_prefix("4.0.")
            .is_some_and(|patch| !patch.is_empty() && patch.bytes().all(|b| b.is_ascii_digit()))
            || provenance.get("adapter").and_then(Value::as_str) != Some("structure-4.0")
            || provenance
                .pointer("/generator/name")
                .and_then(Value::as_str)
                != Some("surrealkit")
        {
            return Err(
                invalid("schema feedback requires selected v4.0 STRUCTURE interpretation").into(),
            );
        }
        let mut diagnostics: Vec<Diagnostic> = serde_json::from_value(
            document
                .get("diagnostics")
                .cloned()
                .ok_or_else(|| invalid("schema feedback diagnostics absent"))?,
        )
        .map_err(|_| invalid("invalid schema feedback diagnostics"))?;
        sanitize(
            &mut diagnostics,
            &crate::envelope::RedactionPolicy::from_environment(),
        );
        if diagnostics
            .iter()
            .any(|d| !matches!(d.severity.as_str(), "warning" | "info"))
        {
            let omitted = diagnostics.len().saturating_sub(256);
            diagnostics.sort_by_key(|diagnostic| {
                matches!(diagnostic.severity.as_str(), "warning" | "info")
            });
            diagnostics.truncate(256);
            return Err(FeedbackError::Diagnostics {
                diagnostics,
                omitted,
            });
        }
        let semantic_digest = document
            .get("semanticDigest")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("schema feedback digest absent"))?
            .to_owned();
        let object = document
            .as_object_mut()
            .ok_or_else(|| invalid("schema feedback is not an object"))?;
        for key in ["semanticDigest", "provenance", "diagnostics"] {
            object.remove(key);
        }
        if let Some(tables) = object.get_mut("tables").and_then(Value::as_array_mut) {
            for table in tables {
                if let Some(table) = table.as_object_mut() {
                    table.remove("define");
                    if let Some(fields) = table.get_mut("fields").and_then(Value::as_array_mut) {
                        for field in fields {
                            if let Some(field) = field.as_object_mut() {
                                field.remove("define");
                                for (key, omitted) in
                                    [("value", "readsInput"), ("assert", "recognized")]
                                {
                                    if let Some(clause) =
                                        field.get_mut(key).and_then(Value::as_object_mut)
                                    {
                                        clause.remove(omitted);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let semantic = canonical(document)?;
        let bytes = serde_json::to_vec(&semantic)
            .map_err(|_| invalid("schema feedback canonicalization failed"))?;
        let hash = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let expected = format!("sha256:{hash}");
        if expected != semantic_digest {
            return Err(invalid("schema feedback semantic digest disagrees with content").into());
        }
        Ok(Self {
            semantic_digest,
            provenance,
            diagnostics,
            preparation: None,
            semantic,
        })
    }

    /// Compare the represented subset; coverage and original provenance remain visible.
    pub fn compare(self, installed: Self, max_differences: usize) -> Comparison {
        let mut differences = Vec::new();
        let mut omitted = false;
        let mut roles_match = true;
        for (side, document, mode, kind) in [
            ("compiled", &self, "embedded", "compile"),
            ("installed", &installed, "remote", "introspect"),
        ] {
            for (pointer, expected) in [("/engine/mode", mode), ("/source/kind", kind)] {
                if document.provenance.pointer(pointer).and_then(Value::as_str) != Some(expected) {
                    roles_match = false;
                    if differences.len() < max_differences {
                        differences.push(format!("/{side}/provenance{pointer}"));
                    } else {
                        omitted = true;
                    }
                }
            }
        }
        let engine_match = self.provenance.pointer("/engine/version")
            == installed.provenance.pointer("/engine/version");
        if !engine_match {
            if differences.len() < max_differences {
                differences.push("/provenance/engine/version".into());
            } else {
                omitted = true;
            }
        }
        differences_at(
            "",
            &self.semantic,
            &installed.semantic,
            max_differences,
            &mut differences,
            &mut omitted,
        );
        let prepared_tool_match = self
            .preparation
            .as_ref()
            .zip(installed.preparation.as_ref())
            .map(|(left, right)| left == right);
        if prepared_tool_match == Some(false) {
            if differences.len() < max_differences {
                differences.push("/preparation".into());
            } else {
                omitted = true;
            }
        }
        Comparison {
            represented_match: roles_match
                && engine_match
                && prepared_tool_match != Some(false)
                && self.semantic == installed.semantic,
            native_witness_required: true,
            prepared_tool_match,
            differences,
            differences_omitted: omitted,
            compiled: self,
            installed,
        }
    }
}

fn sanitize(diagnostics: &mut [Diagnostic], policy: &crate::envelope::RedactionPolicy) {
    fn redact(value: &mut Value, policy: &crate::envelope::RedactionPolicy) {
        match value {
            Value::String(text) => *text = policy.redact(text),
            Value::Array(values) => {
                for value in values {
                    redact(value, policy);
                }
            }
            Value::Object(values) => {
                for value in values.values_mut() {
                    redact(value, policy);
                }
            }
            _ => {}
        }
    }
    for diagnostic in diagnostics {
        if let Some(source) = diagnostic.context.remove("source") {
            let bytes = source.as_str().map_or_else(
                || source.to_string().into_bytes(),
                |text| text.as_bytes().to_vec(),
            );
            let hash = Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            diagnostic
                .context
                .insert("source_sha256".into(), hash.into());
        }
        diagnostic.code = policy.redact(&diagnostic.code);
        diagnostic.reason = policy.redact(&diagnostic.reason);
        for value in diagnostic.context.values_mut() {
            redact(value, policy);
        }
    }
}

fn canonical(value: Value) -> Result<Value, FeedbackError> {
    Ok(match value {
        Value::Object(object) => {
            let sorted: std::collections::BTreeMap<_, _> = object.into_iter().collect();
            Value::Object(
                sorted
                    .into_iter()
                    .map(|(key, value)| Ok((key, canonical(value)?)))
                    .collect::<Result<_, FeedbackError>>()?,
            )
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(canonical)
                .collect::<Result<_, _>>()?,
        ),
        Value::Number(ref number) if !number.is_i64() && !number.is_u64() => {
            return Err(invalid("schema feedback contains non-integer number").into());
        }
        value => value,
    })
}

fn differences_at(
    path: &str,
    left: &Value,
    right: &Value,
    limit: usize,
    result: &mut Vec<String>,
    omitted: &mut bool,
) {
    if left == right {
        return;
    }
    if let (Some(left), Some(right)) = (left.as_object(), right.as_object()) {
        let keys: std::collections::BTreeSet<_> = left.keys().chain(right.keys()).collect();
        for key in keys {
            let child = format!("{path}/{}", key.replace('~', "~0").replace('/', "~1"));
            match (left.get(key), right.get(key)) {
                (Some(left), Some(right)) => {
                    differences_at(&child, left, right, limit, result, omitted)
                }
                _ if result.len() < limit => result.push(child),
                _ => *omitted = true,
            }
        }
    } else if let (Some(left), Some(right)) = (left.as_array(), right.as_array()) {
        for index in 0..left.len().max(right.len()) {
            let child = format!("{path}/{index}");
            match (left.get(index), right.get(index)) {
                (Some(left), Some(right)) => {
                    differences_at(&child, left, right, limit, result, omitted)
                }
                _ if result.len() < limit => result.push(child),
                _ => *omitted = true,
            }
        }
    } else if result.len() < limit {
        result.push(path.to_owned());
    } else {
        *omitted = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn document() -> Value {
        let mut result = json!({"version": 2, "provenance": {
            "generator": {"name": "surrealkit", "version": "1.0.0-rc.1"},
            "adapter": "structure-4.0", "engine": {"version": "4.0.0", "mode": "embedded"},
            "source": {"kind": "compile"}
        }, "diagnostics": []});
        for name in COLLECTIONS {
            result[*name] = json!([]);
        }
        seal(result)
    }

    fn seal(mut result: Value) -> Value {
        let mut projected = result.clone();
        let object = projected.as_object_mut().unwrap();
        for key in ["provenance", "diagnostics", "semanticDigest"] {
            object.remove(key);
        }
        let bytes = serde_json::to_vec(&canonical(projected).unwrap()).unwrap();
        result["semanticDigest"] = format!(
            "sha256:{}",
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        )
        .into();
        result
    }

    fn remote(mut document: Value) -> Value {
        document["provenance"]["engine"]["mode"] = json!("remote");
        document["provenance"]["source"] =
            json!({"kind":"introspect", "namespace":"selected", "database":"selected"});
        document
    }

    #[test]
    fn native_schema_feedback_refuses_errors_wrong_engine_and_forged_digest() {
        let valid = document();
        assert!(Document::admit(valid.clone()).is_ok());
        for diagnostic in [
            json!({"severity":"error", "code":"kind-unrecognized", "reason":"unknown kind", "acknowledged":true}),
            json!({"severity":"unexpected", "code":"new", "reason":"new severity"}),
        ] {
            let mut invalid = valid.clone();
            invalid["diagnostics"] = json!([diagnostic]);
            assert!(Document::admit(invalid).is_err());
        }
        let mut invalid = valid.clone();
        invalid["provenance"]["engine"]["version"] = json!("3.3.0");
        assert!(Document::admit(invalid).is_err());
        let mut invalid = valid;
        invalid["tables"] = json!([{"name":"changed"}]);
        assert!(Document::admit(invalid).is_err());
    }

    #[test]
    fn native_schema_feedback_preserves_unsupported_coverage_and_bounded_differences() {
        let original = document();
        let mut live = remote(original.clone());
        live["diagnostics"] = json!([{"severity":"warning", "code":"definition-unsupported",
            "reason":"named type body is not represented", "source":"DEFINE TYPE pse_state AS string"}]);
        let comparison = Document::admit(original.clone())
            .unwrap()
            .compare(Document::admit(live).unwrap(), 8);
        assert!(comparison.represented_match && comparison.native_witness_required);
        assert!(
            serde_json::to_value(&comparison).unwrap()["compiled"]
                .get("semantic")
                .is_none()
        );
        assert_eq!(comparison.prepared_tool_match, None);
        assert_eq!(
            comparison.installed.diagnostics[0].code,
            "definition-unsupported"
        );
        assert!(
            comparison.installed.diagnostics[0]
                .context
                .get("source")
                .is_none()
        );
        assert!(
            comparison.installed.diagnostics[0]
                .context
                .get("source_sha256")
                .is_some()
        );
        let mut changed = remote(original.clone());
        changed["functions"] = json!([{"name":"changed", "args":[]}]);
        changed["tables"] = json!([{"name":"changed"}]);
        let comparison = Document::admit(original)
            .unwrap()
            .compare(Document::admit(seal(changed)).unwrap(), 1);
        assert!(!comparison.represented_match && comparison.differences_omitted);
        assert_eq!(comparison.differences.len(), 1);
        let mut newer = remote(document());
        newer["provenance"]["engine"]["version"] = json!("4.0.1");
        let mismatch = Document::admit(document())
            .unwrap()
            .compare(Document::admit(newer).unwrap(), 4);
        assert!(!mismatch.represented_match);
        assert_eq!(mismatch.differences, ["/provenance/engine/version"]);
        let mut paths = Vec::new();
        let mut omitted = false;
        differences_at(
            "",
            &json!({}),
            &json!({"missing":null}),
            4,
            &mut paths,
            &mut omitted,
        );
        assert_eq!(paths, ["/missing"]);
        let wrong_role = Document::admit(document())
            .unwrap()
            .compare(Document::admit(document()).unwrap(), 4);
        assert!(!wrong_role.represented_match);
        assert_eq!(
            wrong_role.differences,
            [
                "/installed/provenance/engine/mode",
                "/installed/provenance/source/kind"
            ]
        );
    }

    #[test]
    fn native_schema_feedback_staging_is_explicit_and_never_overwrites() {
        let temporary = tempfile::tempdir().unwrap();
        let directory = temporary.path().join("schema-stage");
        stage(&directory, b"DEFINE TABLE exact SCHEMAFULL;").unwrap();
        assert_eq!(
            fs::read(directory.join("schema/generated.surql")).unwrap(),
            b"DEFINE TABLE exact SCHEMAFULL;"
        );
        assert!(
            fs::read_to_string(directory.join("surrealkit.toml"))
                .unwrap()
                .contains("schema_version = 2")
        );
        assert!(stage(&directory, b"changed").is_err());
        assert_eq!(
            fs::read(directory.join("schema/generated.surql")).unwrap(),
            b"DEFINE TABLE exact SCHEMAFULL;"
        );
    }

    #[test]
    fn native_schema_feedback_refuses_stale_absent_and_failed_child_output() {
        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("document.json");
        fs::write(&output, serde_json::to_vec(&document()).unwrap()).unwrap();
        let deadline = || Instant::now() + std::time::Duration::from_secs(10);
        assert!(execute(Command::new("true"), &output, deadline()).is_err());
        fs::remove_file(&output).unwrap();
        assert!(execute(Command::new("true"), &output, deadline()).is_err());
        assert!(execute(Command::new("false"), &output, deadline()).is_err());
        assert!(!output.exists());
        let mut refused = Command::new("sh");
        refused
            .args(["-c", "printf '%s' \"$SURREALDB_PASSWORD\" >&2; exit 1"])
            .env("SURREALDB_PASSWORD", "private-feedback-credential");
        let error = execute(refused, &output, deadline()).unwrap_err();
        let FeedbackError::Command {
            stderr, truncated, ..
        } = error
        else {
            panic!("wrong refusal");
        };
        assert!(!truncated && !stderr.contains("private-feedback-credential"));
        assert!(stderr.contains("redacted:SURREALDB_PASSWORD"));
        let mut sleeping = Command::new("sleep");
        sleeping.arg("60");
        let error = execute(
            sleeping,
            &output,
            Instant::now() + std::time::Duration::from_secs(3),
        )
        .unwrap_err();
        assert!(matches!(error, FeedbackError::Work(_)), "{error:?}");
    }

    #[test]
    fn native_schema_feedback_refusal_retains_supplier_location_and_source_hash() {
        let mut invalid = document();
        invalid["diagnostics"] = json!([{"severity":"error", "code":"kind-unrecognized",
            "reason":"wrong declared kind", "table":"exact", "field":["generation"], "source":"bad kind"}]);
        let error = Document::admit(invalid).unwrap_err();
        let facts = pse_diagnostics::TypedDiagnostic::diagnostic_facts(&error);
        assert!(matches!(facts.observations.get("supplier_0_code"),
            Some(pse_diagnostics::DiagnosticObservation::Text(code)) if code == "kind-unrecognized"));
        let FeedbackError::Diagnostics {
            diagnostics,
            omitted,
        } = error
        else {
            panic!("wrong refusal");
        };
        assert_eq!(omitted, 0);
        assert_eq!(diagnostics[0].context["field"], json!(["generation"]));
        assert!(diagnostics[0].context.contains_key("source_sha256"));
        assert!(!diagnostics[0].context.contains_key("source"));
    }
}
