// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Deterministic bulk facts over one retained rustdoc artifact, without acquisition.
use super::{ApiError, Artifact};
use rustdoc_types::ItemEnum;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// One exported table; item identities are local to the manifest's original artifact.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Table {
    /// Relative file within the bundle.
    pub file: String,
    /// Number of whole JSONL records.
    pub records: usize,
    /// Exact serialized table bytes.
    pub sha256: String,
}
/// Completed facts bundle; written last, never a claim of build applicability.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Manifest {
    /// Export contract version, independent of upstream rustdoc format.
    pub version: u32,
    /// Caller-selected bundle location.
    pub directory: PathBuf,
    /// Exact source-qualified Cargo package identity.
    pub package_id: String,
    /// Original immutable API artifact location.
    pub artifact: PathBuf,
    /// Original JSON digest.
    pub artifact_sha256: String,
    /// Full acquisition context digest.
    pub context_key: String,
    /// Producer/decoder format.
    pub rustdoc_format: u32,
    /// Documentation features, not an ordinary compiler-unit assertion.
    pub features: Vec<String>,
    /// Documentation target.
    pub target: String,
    /// Original extraction conditions, including documentation-only flags.
    pub extraction_flags: Vec<String>,
    /// Written tables with exact counts and digests.
    pub tables: BTreeMap<String, Table>,
    /// Meaningful bounds of these extracted facts.
    pub limitations: Vec<String>,
}

// Canonicalize every object, including nested rustdoc maps, before JSONL emission.
fn canonical(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| (key, canonical(value)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        Value::Array(array) => Value::Array(array.into_iter().map(canonical).collect()),
        value => value,
    }
}

fn types(value: &Value, pointer: &str, owner: u32, output: &mut Vec<Value>) {
    match value {
        Value::Object(object) => {
            if object.len() == 1
                && object.keys().next().is_some_and(|key| {
                    matches!(
                        key.as_str(),
                        "resolved_path"
                            | "dyn_trait"
                            | "generic"
                            | "primitive"
                            | "function_pointer"
                            | "tuple"
                            | "slice"
                            | "array"
                            | "pat"
                            | "impl_trait"
                            | "raw_pointer"
                            | "borrowed_ref"
                            | "qualified_path"
                    )
                })
            {
                output.push(json!({"item_id":owner,"selector":pointer,"type":value}));
            }
            for (key, child) in object {
                let escaped = key.replace('~', "~0").replace('/', "~1");
                types(child, &format!("{pointer}/{escaped}"), owner, output);
            }
        }
        Value::Array(array) => {
            for (index, child) in array.iter().enumerate() {
                types(child, &format!("{pointer}/{index}"), owner, output);
            }
        }
        _ => {}
    }
}

fn table(directory: &Path, name: &str, rows: Vec<Value>) -> Result<Table, ApiError> {
    let count = rows.len();
    let mut records = rows
        .into_iter()
        .map(|row| serde_json::to_vec(&canonical(row)))
        .collect::<Result<Vec<_>, _>>()?;
    records.sort();
    let mut bytes = Vec::new();
    for record in records {
        bytes.extend(record);
        bytes.push(b'\n');
    }
    let file = format!("{name}.jsonl");
    fs::write(directory.join(&file), &bytes)?;
    Ok(Table {
        file,
        records: count,
        sha256: crate::probe::digest(&bytes),
    })
}

/// Export all retained declarations and public routes in one pass. A new directory
/// is required so an export never silently overwrites caller data or acquired evidence.
pub fn write(artifact: &Artifact, directory: &Path) -> Result<Manifest, ApiError> {
    // Revalidate immutable bytes before creating any output.
    super::artifact_bytes(artifact)?;
    fs::create_dir(directory)?;
    let mut tables = BTreeMap::new();
    for (name, rows) in facts(&artifact.document)? {
        tables.insert(name.clone(), table(directory, &name, rows)?);
    }
    let manifest = Manifest {
        version:1, directory:directory.to_owned(), package_id:artifact.context.package_id.clone(),
        artifact:artifact.path.clone(), artifact_sha256:artifact.sha256.clone(),
        context_key:artifact.context.key()?, rustdoc_format:artifact.document.format_version,
        features:artifact.context.features.iter().cloned().collect(), target:artifact.context.target.clone(),
        extraction_flags:artifact.context.extraction_flags.clone(), tables,
        limitations:vec![
            "declarations include the complete retained index, including non-public and anonymous items; public-paths separately records reachable exposure".into(),
            "trait/impl records preserve original rustdoc defaults, bounds, flags and artifact-local member IDs; they do not prove trait applicability".into(),
            "external-items are path summaries, not acquired defining declarations; no external crate was acquired by export".into(),
            "documentation context includes cfg(doc); ordinary build availability, workspace usage and runtime effects are not established".into(),
            "type-occurrences selectors address structural type variants in original item inner data; absent spans and stripped items remain unknown".into(),
        ],
    };
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}

pub(super) fn facts(
    document: &rustdoc_types::Crate,
) -> Result<BTreeMap<String, Vec<Value>>, ApiError> {
    let mut declarations = Vec::new();
    let mut traits = Vec::new();
    let mut implementations = Vec::new();
    let mut occurrences = Vec::new();
    for item in document.index.values() {
        let original = serde_json::to_value(item)?;
        let row = json!({"item_id":item.id.0,"declaration":original});
        match &item.inner {
            ItemEnum::Trait(trait_) => {
                let mut required = Vec::new();
                let mut provided = Vec::new();
                let mut missing = Vec::new();
                for id in &trait_.items {
                    match document.index.get(id).map(|item| &item.inner) {
                        Some(ItemEnum::Function(function)) if function.has_body => {
                            provided.push(id.0)
                        }
                        Some(ItemEnum::Function(_)) => required.push(id.0),
                        None => missing.push(id.0),
                        _ => {}
                    }
                }
                required.sort();
                provided.sort();
                missing.sort();
                traits.push(json!({"item_id":item.id.0,"declaration":original,
                    "required_methods":required,"provided_methods":provided,"missing_members":missing}));
            }
            ItemEnum::Impl(_) => implementations.push(row.clone()),
            _ => {}
        }
        types(&original["inner"], "/inner", item.id.0, &mut occurrences);
        declarations.push(row);
    }
    let public_paths = super::query::addresses(document)
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    let external = document
        .paths
        .iter()
        .filter(|(id, _)| !document.index.contains_key(id))
        .map(|(id, summary)| json!({"item_id":id.0,"summary":summary}))
        .collect();
    Ok([
        ("public-paths", public_paths),
        ("declarations", declarations),
        ("traits", traits),
        ("implementations", implementations),
        ("type-occurrences", occurrences),
        ("external-items", external),
    ]
    .into_iter()
    .map(|(name, rows)| (name.into(), rows))
    .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn retained_fixture(directory: &Path) -> Artifact {
        use rustdoc_types::{Crate, Id, Item, Module, Target, Visibility};
        let document = Crate {
            root: Id(0),
            crate_version: Some("1.2.3".into()),
            includes_private: false,
            index: [(
                Id(0),
                Item {
                    id: Id(0),
                    crate_id: 0,
                    name: Some("fixture".into()),
                    span: None,
                    visibility: Visibility::Public,
                    docs: Some("A retained fixture.".into()),
                    links: Default::default(),
                    attrs: Vec::new(),
                    deprecation: None,
                    stability: None,
                    const_stability: None,
                    inner: ItemEnum::Module(Module {
                        is_crate: true,
                        items: Vec::new(),
                        is_stripped: false,
                    }),
                },
            )]
            .into(),
            paths: Default::default(),
            external_crates: Default::default(),
            target: Target {
                triple: "x86_64-unknown-linux-gnu".into(),
                target_features: Vec::new(),
            },
            format_version: rustdoc_types::FORMAT_VERSION,
        };
        let context: super::super::Context = serde_json::from_value(json!({
            "version":1, "source_root":directory, "package_id":"registry+https://example.invalid#fixture@1.2.3",
            "manifest":directory.join("Cargo.toml"), "package_source_sha256":"fixture-source",
            "dependency_sources":{}, "metadata_sha256":"fixture-metadata", "features":["advanced"],
            "metadata_features":["advanced"], "unit_graph_sha256":"fixture-units",
            "source":{"version":1,"package_roots":[],"additional_files":[],"files":{},
                "context":{"rustc":"fixture-rustc","cargo":"fixture-cargo","profile":"test","features":[],"environment":{}}},
            "target":"x86_64-unknown-linux-gnu", "extraction_flags":["--output-format","json"],
            "rustdoc_format":rustdoc_types::FORMAT_VERSION,"rustdoc_identity":"fixture-rustdoc",
            "environment_sha256":"fixture-environment"
        })).unwrap();
        let bytes = serde_json::to_vec(&document).unwrap();
        fs::write(directory.join("document.json"), &bytes).unwrap();
        fs::write(
            directory.join("context.json"),
            serde_json::to_vec(&context).unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("artifact.json"),
            serde_json::to_vec(&json!({
                "context_key":context.key().unwrap(),"sha256":crate::probe::digest(&bytes)
            }))
            .unwrap(),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt as _;
        for name in ["document.json", "context.json", "artifact.json"] {
            fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o600)).unwrap();
        }
        super::super::retained(&directory.join("document.json")).unwrap()
    }
    #[test]
    fn retained_export_preserves_provenance_and_verifiable_table_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let artifact = retained_fixture(temp.path());
        let output = temp.path().join("bundle");
        let manifest = write(&artifact, &output).unwrap();
        assert_eq!(manifest.package_id, artifact.context.package_id);
        assert_eq!(manifest.context_key, artifact.context.key().unwrap());
        assert_eq!(manifest.artifact_sha256, artifact.sha256);
        assert_eq!(manifest.features, vec!["advanced"]);
        assert_eq!(manifest.target, artifact.context.target);
        assert_eq!(manifest.tables["declarations"].records, 1);
        assert_eq!(manifest.tables.len(), 6);
        let disk: Value =
            serde_json::from_slice(&fs::read(output.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(disk, serde_json::to_value(&manifest).unwrap());
        for table in manifest.tables.values() {
            let bytes = fs::read(output.join(&table.file)).unwrap();
            assert_eq!(table.sha256, crate::probe::digest(&bytes));
            assert_eq!(
                table.records,
                bytes
                    .split(|byte| *byte == b'\n')
                    .filter(|line| !line.is_empty())
                    .count()
            );
        }
    }
    #[test]
    fn export_refuses_existing_destination_without_overwriting_caller_data() {
        let temp = tempfile::tempdir().unwrap();
        let artifact = retained_fixture(temp.path());
        let output = temp.path().join("existing");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("sentinel"), b"preserve").unwrap();
        assert!(write(&artifact, &output).is_err());
        assert_eq!(fs::read(output.join("sentinel")).unwrap(), b"preserve");
        assert!(!output.join("manifest.json").exists());
        assert_eq!(fs::read_dir(output).unwrap().count(), 1);
    }
    #[test]
    fn changed_original_bytes_refuse_before_any_bundle_publication() {
        let temp = tempfile::tempdir().unwrap();
        let artifact = retained_fixture(temp.path());
        fs::write(&artifact.path, b"changed original").unwrap();
        let output = temp.path().join("refused");
        assert!(matches!(
            write(&artifact, &output),
            Err(ApiError::Invalid(_))
        ));
        assert!(!output.exists());
        assert!(!output.join("manifest.json").exists());
    }
    #[test]
    fn types_keep_owner_and_exact_nested_selectors() {
        let input = json!({"function":{"sig":{"inputs":[["x",{"borrowed_ref":{
            "type":{"resolved_path":{"path":"Vec","id":7,"args":null}},
            "is_mutable":false,"lifetime":null}}]]}}});
        let mut rows = Vec::new();
        types(&input, "/inner", 11, &mut rows);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["item_id"], 11);
        assert_eq!(
            rows[1]["selector"],
            "/inner/function/sig/inputs/0/1/borrowed_ref/type"
        );
    }
    #[test]
    fn table_bytes_are_order_independent_and_count_whole_records() {
        let temp = tempfile::tempdir().unwrap();
        let a = table(
            temp.path(),
            "a",
            vec![json!({"z":2,"a":{"y":2,"b":1}}), json!({"x":1})],
        )
        .unwrap();
        let b = table(
            temp.path(),
            "b",
            vec![json!({"x":1}), json!({"a":{"b":1,"y":2},"z":2})],
        )
        .unwrap();
        assert_eq!(a.sha256, b.sha256);
        assert_eq!(a.records, 2);
    }
}
