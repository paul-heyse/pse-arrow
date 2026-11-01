// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Differences between explicitly selected schema worlds; native Rust identity stays separate.
use super::{Bundle, SchemaError};
use buffa_descriptor::generated::descriptor::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet,
};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::{collections::BTreeMap, path::Path};

/// Observed descriptor change, not a proof of semantic compatibility.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Difference {
    /// Bundle-local declaration/member identity.
    pub declaration: String,
    /// Changed descriptor property.
    pub property: String,
    /// Original property, absent for additions.
    pub before: Option<Value>,
    /// Selected property, absent for removals.
    pub after: Option<Value>,
    /// Dimensions requiring assessment; native Buf results supply rule classifications.
    pub dimensions: Vec<String>,
}
/// Original native rule result, retaining errors independently of intentional breaks.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NativeClassification {
    /// WIRE, WIRE_JSON or FILE.
    pub category: String,
    /// compatible, breaking, unavailable or failed.
    pub outcome: String,
    /// Native process status, including expected breaking status 100.
    pub exit_code: Option<i32>,
    /// Exact selected tool image identity.
    pub executable_sha256: Option<String>,
    /// Actual tool version response.
    pub version: Option<String>,
    /// Original diagnostic bytes (JSON serialization preserves every byte).
    pub stdout: Vec<u8>,
    /// Original diagnostic bytes, separate from findings.
    pub stderr: Vec<u8>,
}
#[path = "../../../../scripts/protobuf-associations.rs"]
mod association_protocol;
pub use association_protocol::{Association, AssociationManifest};
/// Typed passive comparison; no decoded RPC is invoked.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Comparison {
    /// Before bundle identity.
    pub baseline_digest: String,
    /// After bundle identity.
    pub current_digest: String,
    /// Structural and constraint observations.
    pub differences: Vec<Difference>,
    /// Independently executed native rule classifications.
    pub native: Vec<NativeClassification>,
    /// Qualified generated-origin nominations.
    pub associations: Vec<Association>,
    /// Unexamined meaning remains explicit.
    pub limitations: Vec<String>,
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn image_argument(path: &Path) -> std::ffi::OsString {
    let mut argument = path.as_os_str().to_owned();
    argument.push("#format=binpb");
    argument
}
fn options(value: &impl buffa::Message) -> Value {
    // ProtoJSON alone omits unknown custom extensions. Original encoded options preserve them.
    Value::String(
        value
            .encode_to_vec()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}
fn record(
    records: &mut BTreeMap<String, Value>,
    key: String,
    value: &impl Serialize,
    encoded_options: Option<Value>,
) -> Result<(), SchemaError> {
    let mut value = serde_json::to_value(value)?;
    if let Some(object) = value.as_object_mut() {
        object.remove("options");
        if let Some(options) = encoded_options {
            object.insert("encoded_options".into(), options);
        }
    }
    records.insert(key, value);
    Ok(())
}
fn enumeration(
    records: &mut BTreeMap<String, Value>,
    prefix: &str,
    item: &EnumDescriptorProto,
) -> Result<(), SchemaError> {
    let name = qualified(prefix, item.name.as_deref().unwrap_or_default());
    let mut header = serde_json::to_value(item)?;
    if let Some(object) = header.as_object_mut() {
        object.remove("value");
    }
    record(
        records,
        name.clone(),
        &header,
        item.options.as_option().map(options),
    )?;
    // Aliases can share a number, so authored value names remain part of enum-member identity.
    for value in &item.value {
        record(
            records,
            format!(
                "{name}:{}:{}",
                value.number.unwrap_or_default(),
                value.name.as_deref().unwrap_or_default()
            ),
            value,
            value.options.as_option().map(options),
        )?;
    }
    Ok(())
}
fn message(
    records: &mut BTreeMap<String, Value>,
    prefix: &str,
    item: &DescriptorProto,
) -> Result<(), SchemaError> {
    let name = qualified(prefix, item.name.as_deref().unwrap_or_default());
    let mut header = serde_json::to_value(item)?;
    if let Some(object) = header.as_object_mut() {
        for key in ["field", "extension", "nestedType", "enumType", "oneofDecl"] {
            object.remove(key);
        }
    }
    record(
        records,
        name.clone(),
        &header,
        item.options.as_option().map(options),
    )?;
    for field in &item.field {
        record(
            records,
            format!("{name}#{}", field.number.unwrap_or_default()),
            field,
            field.options.as_option().map(options),
        )?;
    }
    for field in &item.extension {
        record(
            records,
            format!("{name}:extension:{}", field.number.unwrap_or_default()),
            field,
            field.options.as_option().map(options),
        )?;
    }
    for (index, oneof) in item.oneof_decl.iter().enumerate() {
        record(
            records,
            format!("{name}:oneof:{index}"),
            oneof,
            oneof.options.as_option().map(options),
        )?;
    }
    for child in &item.nested_type {
        message(records, &name, child)?;
    }
    for enumeration in &item.enum_type {
        self::enumeration(records, &name, enumeration)?;
    }
    Ok(())
}
fn qualified(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.into()
    } else {
        format!("{prefix}.{name}")
    }
}
fn records(set: &FileDescriptorSet) -> Result<BTreeMap<String, Value>, SchemaError> {
    let mut records = BTreeMap::new();
    for file in &set.file {
        let prefix = file.package.as_deref().unwrap_or_default();
        let mut header = serde_json::to_value(file)?;
        if let Some(object) = header.as_object_mut() {
            for key in [
                "messageType",
                "enumType",
                "service",
                "extension",
                "sourceCodeInfo",
            ] {
                object.remove(key);
            }
        }
        record(
            &mut records,
            format!("file:{}", file.name.as_deref().unwrap_or_default()),
            &header,
            file.options.as_option().map(options),
        )?;
        for item in &file.message_type {
            message(&mut records, prefix, item)?;
        }
        for item in &file.enum_type {
            enumeration(&mut records, prefix, item)?;
        }
        for item in &file.extension {
            record(
                &mut records,
                format!(
                    "extension:{}:{}",
                    item.extendee.as_deref().unwrap_or_default(),
                    item.number.unwrap_or_default()
                ),
                item,
                item.options.as_option().map(options),
            )?;
        }
        for service in &file.service {
            let name = qualified(prefix, service.name.as_deref().unwrap_or_default());
            record(
                &mut records,
                name.clone(),
                &serde_json::json!({"name":service.name}),
                service.options.as_option().map(options),
            )?;
            for method in &service.method {
                record(
                    &mut records,
                    qualified(&name, method.name.as_deref().unwrap_or_default()),
                    method,
                    method.options.as_option().map(options),
                )?;
            }
        }
    }
    Ok(records)
}
/// Compare two complete immutable descriptor worlds without acquiring source or native sessions.
/// # Errors
/// Descriptor properties cannot be represented in the typed observation.
pub fn compare(current: &Bundle, baseline: &Bundle) -> Result<Comparison, SchemaError> {
    let old = records(&baseline.set)?;
    let new = records(&current.set)?;
    let keys: std::collections::BTreeSet<_> = old.keys().chain(new.keys()).collect();
    let mut differences = Vec::new();
    for key in keys {
        let properties: std::collections::BTreeSet<_> = old
            .get(key)
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|value| value.keys())
            .chain(
                new.get(key)
                    .and_then(Value::as_object)
                    .into_iter()
                    .flat_map(|value| value.keys()),
            )
            .collect();
        for property in properties {
            let before = old.get(key).and_then(|value| value.get(property));
            let after = new.get(key).and_then(|value| value.get(property));
            if before == after {
                continue;
            }
            let dimensions = match property.as_str() {
                "name" | "jsonName" => vec!["json", "generated_rust_source"],
                "encoded_options" => vec!["constraint_or_custom_option_semantics"],
                "dependency" | "publicDependency" | "weakDependency" | "optionDependency" => {
                    vec!["descriptor_context"]
                }
                _ => vec!["binary", "json", "generated_rust_source", "semantic"],
            };
            differences.push(Difference {
                declaration: key.clone(),
                property: property.clone(),
                before: before.cloned(),
                after: after.cloned(),
                dimensions: dimensions.into_iter().map(str::to_owned).collect(),
            });
        }
    }
    Ok(Comparison { baseline_digest:baseline.digest.clone(), current_digest:current.digest.clone(), differences, native:Vec::new(), associations:Vec::new(), limitations:vec!["Dimensions nominate assessment; Buf classifications are separate native rule evidence, never an intentional-break veto.".into(), "Custom-option bytes reveal changes but do not establish CEL or other semantic equivalence; source comments and descriptor ordering are not semantic differences.".into(), "Generated associations nominate source only. Rust consumers, cfg worlds and API identities require independently qualified original source/HIR/rustdoc investigations.".into()] })
}

/// Execute independent native compatibility categories against frozen copies of both inputs.
/// # Errors
/// Captured inputs or native child I/O cannot be read or settled.
pub async fn classify(
    current: &Bundle,
    baseline: &Bundle,
) -> Result<Vec<NativeClassification>, SchemaError> {
    let directory = std::sync::Arc::new(tempfile::tempdir()?);
    let old = directory.path().join("baseline.bin");
    let new = directory.path().join("current.bin");
    std::fs::write(&old, &baseline.original)?;
    std::fs::write(&new, &current.original)?;
    let selected = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .map(|path| path.join("buf"))
        .find(|path| path.is_file());
    let Some(executable) = selected else {
        return Ok(["WIRE", "WIRE_JSON", "FILE"]
            .into_iter()
            .map(|category| NativeClassification {
                category: category.into(),
                outcome: "unavailable".into(),
                exit_code: None,
                executable_sha256: None,
                version: None,
                stdout: Vec::new(),
                stderr: b"buf is unavailable; install explicitly before native classification"
                    .to_vec(),
            })
            .collect());
    };
    let executable_sha256 = hash(&std::fs::read(&executable)?);
    let mut version_command = std::process::Command::new(&executable);
    version_command.arg("--version");
    let version_output = capture(version_command, std::sync::Arc::clone(&directory)).await;
    let version = version_output
        .as_ref()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned());
    let run = |category: &'static str| {
        let executable = &executable;
        let executable_sha256 = &executable_sha256;
        let version = &version;
        let old = &old;
        let new = &new;
        let custody = std::sync::Arc::clone(&directory);
        async move {
            let config = format!("version: v2\nbreaking:\n  use: [{category}]\n");
            let mut command = std::process::Command::new(executable);
            command
                .arg("breaking")
                .arg(image_argument(new))
                .arg("--against")
                .arg(image_argument(old))
                .arg("--config")
                .arg(config)
                .args(["--error-format", "json"]);
            match capture(command, custody).await {
                Ok(output) => {
                    let exit_code = output.status.code();
                    NativeClassification {
                        category: category.into(),
                        outcome: match exit_code {
                            Some(0) => "compatible",
                            Some(100) => "breaking",
                            _ => "failed",
                        }
                        .into(),
                        exit_code,
                        executable_sha256: Some(executable_sha256.clone()),
                        version: version.clone(),
                        stdout: output.stdout,
                        stderr: output.stderr,
                    }
                }
                Err(error) => NativeClassification {
                    category: category.into(),
                    outcome: "failed".into(),
                    exit_code: None,
                    executable_sha256: Some(executable_sha256.clone()),
                    version: version.clone(),
                    stdout: Vec::new(),
                    stderr: error.to_string().into_bytes(),
                },
            }
        }
    };
    // Every independently started reader settles, including after sibling failure.
    let (wire, json, source) = tokio::join!(run("WIRE"), run("WIRE_JSON"), run("FILE"));
    Ok(vec![wire, json, source])
}

struct CancelCapture(pse_diagnostics::Cancellation);
impl Drop for CancelCapture {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
async fn capture(
    mut command: std::process::Command,
    custody: std::sync::Arc<tempfile::TempDir>,
) -> Result<std::process::Output, SchemaError> {
    let control = pse_diagnostics::Cancellation::default();
    let _cancel_on_drop = CancelCapture(control.clone());
    tokio::task::spawn_blocking(move || {
        // Both input snapshots and the actual supervisor live until this worker settles,
        // even if the caller drops its future. Cancellation wakes its pidfd wait.
        let _custody = custody;
        use pse_diagnostics::{DrainFailurePolicy, SupervisedChild};
        use std::io::{Read as _, Seek as _};
        use std::process::Stdio;
        let mut stdout = tempfile::tempfile()?;
        let mut stderr = tempfile::tempfile()?;
        command
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout.try_clone()?))
            .stderr(Stdio::from(stderr.try_clone()?));
        let mut child = SupervisedChild::spawn(&mut command, DrainFailurePolicy::Park, None)
            .map_err(|error| SchemaError::Tool(error.to_string()))?;
        let waited = child.wait_ready(
            Some(&control),
            Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
        );
        let settled = child.cancel();
        let mut out = Vec::new();
        let mut err = Vec::new();
        stdout.rewind()?;
        stdout.read_to_end(&mut out)?;
        stderr.rewind()?;
        stderr.read_to_end(&mut err)?;
        if let Err(error) = waited {
            return Err(SchemaError::Tool(format!(
                "Buf: {error}; settlement={settled:?}; {}",
                String::from_utf8_lossy(&err)
            )));
        }
        let status =
            settled.map_err(|error| SchemaError::Tool(format!("Buf settlement: {error}")))?;
        Ok(std::process::Output {
            status,
            stdout: out,
            stderr: err,
        })
    })
    .await
    .map_err(|error| SchemaError::Tool(format!("Buf reader task: {error}")))?
}

/// Resolve only exact producer associations; stale/missing originals retain their limits.
/// # Errors
/// A present manifest is malformed or its configuration cannot be interpreted.
pub fn associations(root: &Path, bundle: &Bundle) -> Result<Vec<Association>, SchemaError> {
    let mut result = Vec::new();
    for producer in ["code-intelligence", "plan61-fixtures"] {
        let path = root
            .join("docs/generated/source-relationships")
            .join(producer)
            .join("associations.json");
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        let manifest: AssociationManifest = serde_json::from_slice(&bytes)?;
        if manifest.descriptor_sha256 != bundle.digest {
            continue;
        }
        for path in manifest.inputs.keys().map(String::as_str).chain(
            manifest
                .associations
                .iter()
                .flat_map(|association| [association.source.as_str(), association.output.as_str()]),
        ) {
            if !crate::ownership::protocol::normal_relative(Path::new(path)) {
                return Err(SchemaError::Invalid(
                    "generator association path must be checkout-relative without traversal".into(),
                ));
            }
        }
        let config_current = manifest.inputs.iter().all(|(path, digest)| {
            std::fs::read(root.join(path)).is_ok_and(|bytes| hash(&bytes) == *digest)
        });
        for mut association in manifest.associations {
            association.state = match std::fs::read(root.join(&association.source)) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => "missing_source",
                Err(error) => return Err(error.into()),
                Ok(bytes) if hash(&bytes) != association.source_sha256 => "stale_source",
                Ok(_) if !config_current => "stale_generation_inputs",
                Ok(_) => match std::fs::read(root.join(&association.output)) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => "missing_output",
                    Err(error) => return Err(error.into()),
                    Ok(bytes) if hash(&bytes) != association.output_sha256 => "stale_output",
                    Ok(_) => "verified",
                },
            }
            .into();
            result.push(association);
        }
    }
    Ok(result)
}

/// Follow an exact generated Rust hit through an explicit declaration association.
/// # Errors
/// Present producer artifacts are malformed; no name-only fallback is attempted.
pub fn origin_associations(
    root: &Path,
    source: &crate::source_read::SourceText,
    selected: std::ops::Range<usize>,
) -> Result<Vec<Association>, SchemaError> {
    if selected.start > selected.end || source.text().get(selected.clone()).is_none() {
        return Err(SchemaError::Invalid(
            "origin selection is not an original UTF-8 source range".into(),
        ));
    }
    if !matches!(
        source.identity().origin(),
        crate::source_read::SourceOrigin::Working
    ) {
        return Ok(Vec::new());
    }
    let path = source.identity().path();
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut candidates = Vec::new();
    for (producer, descriptor) in [
        (
            "code-intelligence",
            "crates/pse-host/src/generated/code-intelligence/descriptor.bin",
        ),
        (
            "plan61-fixtures",
            "crates/pse-host/src/generated/plan61/descriptor.bin",
        ),
    ] {
        let manifest = root
            .join("docs/generated/source-relationships")
            .join(producer)
            .join("associations.json");
        if !manifest.is_file() {
            continue;
        }
        let manifest: AssociationManifest = serde_json::from_slice(&std::fs::read(&manifest)?)?;
        if !manifest
            .associations
            .iter()
            .any(|association| Path::new(&association.output) == relative)
        {
            continue;
        }
        let bundle = Bundle::load(&root.join(descriptor))?;
        candidates.extend(
            associations(root, &bundle)?
                .into_iter()
                .filter(|association| {
                    Path::new(&association.output) == relative
                        && association.output_sha256 == source.identity().source_sha256()
                }),
        );
    }
    if candidates.is_empty() {
        return Ok(candidates);
    }
    let outline = crate::source_read::rust(source.text(), crate::source_read::RustEdition::E2024);
    let enclosing = outline
        .entries
        .iter()
        .filter(|entry| {
            matches!(entry.kind.as_str(), "STRUCT" | "IMPL")
                && entry.range.byte_start <= selected.start
                && entry.range.byte_end >= selected.end
        })
        .min_by_key(|entry| entry.range.byte_end - entry.range.byte_start);
    let Some(enclosing) = enclosing else {
        return Ok(Vec::new());
    };
    let written_type = enclosing.path.join("::");
    let field = selected_rust_field(source.text(), selected);
    candidates.retain(|association| {
        association.rust_type == written_type
            && match &field {
                Some(field) => association.rust_field.as_deref().is_some_and(|name| {
                    name.trim_start_matches("r#") == field.trim_start_matches("r#")
                }),
                None => association.rust_field.is_none(),
            }
    });
    Ok(candidates)
}

fn selected_rust_field(source: &str, selected: std::ops::Range<usize>) -> Option<String> {
    use ra_ap_syntax::{AstNode as _, ast, ast::HasName as _};
    let parsed = ra_ap_syntax::SourceFile::parse(source, ra_ap_syntax::Edition::Edition2024);
    let contains = |node: &ra_ap_syntax::SyntaxNode| {
        let range = node.text_range();
        usize::from(range.start()) <= selected.start && usize::from(range.end()) >= selected.end
    };
    let self_member = |node: ra_ap_syntax::SyntaxNode| -> Option<String> {
        let field = ast::FieldExpr::cast(node)?;
        let receiver = field.expr()?;
        let ast::Expr::PathExpr(receiver) = receiver else {
            return None;
        };
        let path = receiver.path()?;
        if path.as_single_name_ref()?.text() != "self" {
            return None;
        }
        field.name_ref().map(|name| name.text().to_string())
    };
    let mut fields = parsed.syntax_node().descendants().filter_map(|node| {
        if !contains(&node) {
            return None;
        }
        if let Some(field) = ast::RecordField::cast(node.clone()) {
            return field.name().map(|name| name.text().to_string());
        }
        self_member(node)
    });
    if let Some(first) = fields.next() {
        return fields.all(|field| field == first).then_some(first);
    }
    let function = parsed
        .syntax_node()
        .descendants()
        .filter_map(ast::Fn::cast)
        .filter(|function| contains(function.syntax()))
        .last()?;
    // An accessor maps only through its actual self-member evidence, never its spelling.
    let mut members = function
        .syntax()
        .descendants()
        .filter(|node| node.ancestors().find_map(ast::Fn::cast).as_ref() == Some(&function))
        .filter_map(self_member)
        .collect::<std::collections::BTreeSet<_>>();
    (members.len() == 1).then(|| members.pop_first()).flatten()
}

/// Resolve a verified producer association's exact authored descriptor span in original bytes.
/// # Errors
/// A present descriptor or association manifest is malformed.
pub fn authored_range(
    root: &Path,
    association: &Association,
    source: &str,
) -> Result<Option<std::ops::Range<usize>>, SchemaError> {
    if association.state != "verified" || hash(source.as_bytes()) != association.source_sha256 {
        return Ok(None);
    }
    for descriptor in [
        "crates/pse-host/src/generated/code-intelligence/descriptor.bin",
        "crates/pse-host/src/generated/plan61/descriptor.bin",
    ] {
        let path = root.join(descriptor);
        if !path.is_file() {
            continue;
        }
        let bundle = Bundle::load(&path)?;
        if !associations(root, &bundle)?.iter().any(|candidate| {
            candidate.state == "verified"
                && candidate.declaration == association.declaration
                && candidate.source == association.source
                && candidate.output == association.output
                && candidate.output_sha256 == association.output_sha256
                && candidate.rust_field == association.rust_field
        }) {
            continue;
        }
        let declaration = bundle.declaration(&association.declaration)?;
        let Some(info) = &declaration.source_info else {
            return Ok(None);
        };
        let (line_start, column_start, line_end, column_end) = match info.span.as_slice() {
            [line, start, end] => (*line, *start, *line, *end),
            [start_line, start, end_line, end] => (*start_line, *start, *end_line, *end),
            _ => return Ok(None),
        };
        let lines = crate::source_read::line_starts(source);
        let offset = |line: i32, column: i32| -> Option<usize> {
            let line = usize::try_from(line).ok()?;
            let column = usize::try_from(column).ok()?;
            let start = *lines.get(line)?;
            let end = lines.get(line + 1).map_or(source.len(), |end| end - 1);
            let offset = start.checked_add(column)?;
            (offset <= end && source.is_char_boundary(offset)).then_some(offset)
        };
        let Some(start) = offset(line_start, column_start) else {
            return Ok(None);
        };
        let Some(end) = offset(line_end, column_end) else {
            return Ok(None);
        };
        return Ok((start <= end && source.get(start..end).is_some()).then_some(start..end));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    #[test]
    fn schema_comparison_image_arguments_preserve_native_path_bytes() {
        use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
        let path = std::path::PathBuf::from(std::ffi::OsString::from_vec(
            b"/tmp/schema-\xff.bin".to_vec(),
        ));
        assert_eq!(
            image_argument(&path).as_bytes(),
            b"/tmp/schema-\xff.bin#format=binpb"
        );
    }
    fn bundle(set: &FileDescriptorSet) -> Bundle {
        Bundle::from_bytes("schema.bin".into(), set.encode_to_vec()).unwrap()
    }
    fn original() -> FileDescriptorSet {
        FileDescriptorSet::decode_from_slice(include_bytes!("../generated/plan61/descriptor.bin"))
            .unwrap()
    }
    fn payload(set: &mut FileDescriptorSet) -> &mut DescriptorProto {
        set.file
            .iter_mut()
            .flat_map(|file| &mut file.message_type)
            .find(|message| message.name.as_deref() == Some("Payload"))
            .unwrap()
    }
    #[test]
    fn schema_comparison_field_rename_preserves_number_identity() {
        let old = original();
        let mut new = old.clone();
        let field = payload(&mut new)
            .field
            .iter_mut()
            .find(|field| field.number == Some(6))
            .unwrap();
        field.name = Some("renamed_label".into());
        field.json_name = Some("renamedLabel".into());
        let report = compare(&bundle(&new), &bundle(&old)).unwrap();
        assert_eq!(report.differences.len(), 2);
        assert!(report.differences.iter().all(|difference| {
            difference.declaration == "plan61.qualification.v1.Payload#6"
                && !difference
                    .dimensions
                    .iter()
                    .any(|dimension| dimension == "binary")
        }));
    }
    #[test]
    fn schema_comparison_custom_constraint_bytes_are_not_lost_to_json() {
        let old = original();
        let mut new = old.clone();
        payload(&mut new).options = Default::default();
        let report = compare(&bundle(&new), &bundle(&old)).unwrap();
        assert_eq!(report.differences.len(), 1);
        assert_eq!(report.differences[0].property, "encoded_options");
        assert_eq!(
            report.differences[0].dimensions,
            ["constraint_or_custom_option_semantics"]
        );
    }
    #[tokio::test]
    async fn schema_comparison_buf_qualifies_wire_safe_json_break() {
        let old = original();
        let mut new = old.clone();
        let field = payload(&mut new)
            .field
            .iter_mut()
            .find(|field| field.number == Some(6))
            .unwrap();
        field.name = Some("renamed_label".into());
        field.json_name = Some("renamedLabel".into());
        let reports = classify(&bundle(&new), &bundle(&old)).await.unwrap();
        assert_eq!(
            reports
                .iter()
                .map(|report| report.outcome.as_str())
                .collect::<Vec<_>>(),
            ["compatible", "breaking", "breaking"]
        );
        assert_eq!(reports[1].exit_code, Some(100));
        assert!(!reports[1].stdout.is_empty());
    }
    #[test]
    fn schema_associations_verify_inputs_source_output_and_descriptor() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let selected = bundle(&original());
        std::fs::write(root.join("source.proto"), b"authored").unwrap();
        std::fs::write(root.join("generated.rs"), b"emitted").unwrap();
        std::fs::write(root.join("config"), b"configuration").unwrap();
        let record = Association {
            declaration: "plan61.qualification.v1.Payload".into(),
            source: "source.proto".into(),
            source_sha256: hash(b"authored"),
            output: "generated.rs".into(),
            output_sha256: hash(b"emitted"),
            rust_type: "ExplicitRustType".into(),
            rust_field: None,
            state: String::new(),
        };
        let manifest = AssociationManifest {
            descriptor_sha256: selected.digest.clone(),
            inputs: BTreeMap::from([("config".into(), hash(b"configuration"))]),
            associations: vec![record],
        };
        let path = root.join("docs/generated/source-relationships/code-intelligence");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(
            path.join("associations.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(associations(root, &selected).unwrap()[0].state, "verified");
        std::fs::write(root.join("generated.rs"), b"changed").unwrap();
        assert_eq!(
            associations(root, &selected).unwrap()[0].state,
            "stale_output"
        );
        std::fs::remove_file(root.join("source.proto")).unwrap();
        assert_eq!(
            associations(root, &selected).unwrap()[0].state,
            "missing_source"
        );
        assert!(
            associations(
                root,
                &Bundle::from_bytes(
                    "different".into(),
                    FileDescriptorSet::default().encode_to_vec()
                )
                .unwrap()
            )
            .unwrap()
            .is_empty()
        );
    }
    #[test]
    fn schema_origin_uses_actual_enclosing_type_and_exact_generated_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let selected = bundle(&original());
        let output = "crates/pse-host/src/generated/code-intelligence/example.rs";
        let text = "pub struct ActualRust { pub field: i32 }\npub struct Namesake { pub field: i32 }\nimpl ActualRust { fn accessor(&self) -> i32 { self.field } }\n";
        let authored =
            include_str!("../../tests/fixtures/plan61/plan61/qualification/v1/contract.proto");
        let generated = root.join(output);
        std::fs::create_dir_all(generated.parent().unwrap()).unwrap();
        std::fs::write(&generated, text).unwrap();
        std::fs::write(
            generated.parent().unwrap().join("descriptor.bin"),
            &selected.original,
        )
        .unwrap();
        std::fs::write(root.join("original.proto"), authored).unwrap();
        let manifest = AssociationManifest {
            descriptor_sha256: selected.digest,
            inputs: BTreeMap::new(),
            associations: vec![
                Association {
                    declaration: "plan61.qualification.v1.Payload".into(),
                    source: "original.proto".into(),
                    source_sha256: hash(authored.as_bytes()),
                    output: output.into(),
                    output_sha256: hash(text.as_bytes()),
                    rust_type: "ActualRust".into(),
                    rust_field: None,
                    state: String::new(),
                },
                Association {
                    declaration: "plan61.qualification.v1.Payload#6".into(),
                    source: "original.proto".into(),
                    source_sha256: hash(authored.as_bytes()),
                    output: output.into(),
                    output_sha256: hash(text.as_bytes()),
                    rust_type: "ActualRust".into(),
                    rust_field: Some("field".into()),
                    state: String::new(),
                },
            ],
        };
        let path = root.join("docs/generated/source-relationships/code-intelligence");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(
            path.join("associations.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let source = crate::source_read::SourceText::new(generated.clone(), text.into()).unwrap();
        assert_eq!(origin_associations(root, &source, 4..10).unwrap().len(), 1);
        let field = text.find("field").unwrap();
        let origins = origin_associations(root, &source, field..field + 5).unwrap();
        assert_eq!(origins.len(), 1);
        assert_eq!(origins[0].declaration, "plan61.qualification.v1.Payload#6");
        let span = authored_range(root, &origins[0], authored)
            .unwrap()
            .unwrap();
        assert_eq!(&authored[span], "string label = 6;");
        let accessor = text.find("accessor").unwrap();
        let origins = origin_associations(root, &source, accessor..accessor + 8).unwrap();
        assert_eq!(origins[0].declaration, "plan61.qualification.v1.Payload#6");
        let wrong = text.find("Namesake").unwrap();
        assert!(
            origin_associations(root, &source, wrong..wrong + 1)
                .unwrap()
                .is_empty()
        );
        for origin in [
            crate::source_read::SourceOrigin::Index {
                git_dir: crate::repository::GitName::Utf8(".git".into()),
                index_path: crate::repository::GitName::Utf8(".git/index".into()),
                blob: "original-blob".into(),
            },
            crate::source_read::SourceOrigin::Blob {
                git_dir: crate::repository::GitName::Utf8(".git".into()),
                revision: "original-revision".into(),
                blob: "original-blob".into(),
            },
        ] {
            let identity = crate::source_read::SourceIdentity::from_parts(
                std::path::PathBuf::from(output),
                hash(text.as_bytes()),
                origin,
            )
            .unwrap();
            let historical =
                crate::source_read::SourceText::from_identity(identity, text.into()).unwrap();
            assert!(
                origin_associations(root, &historical, 4..10)
                    .unwrap()
                    .is_empty()
            );
        }
        let changed =
            crate::source_read::SourceText::new(generated, format!("{text}// changed")).unwrap();
        assert!(
            origin_associations(root, &changed, 4..10)
                .unwrap()
                .is_empty()
        );
    }
}
