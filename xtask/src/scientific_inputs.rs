// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit export and comparison for independently selected scientific consumers.

use anyhow::{Context, Result, ensure};
use std::{io::Read as _, path::Path};

#[path = "../../tests/fixtures/scientific-input-capture.rs"]
mod owner;
use owner::MAX_CAPTURE_BYTES;

fn capture(root: &Path, catalog: &Path) -> Result<owner::data::Capsule> {
    owner::capture(root, catalog).map_err(|error| anyhow::Error::from_boxed(error.into()))
}

pub(super) fn run(root: &Path, catalog: &Path, output: &Path) -> Result<()> {
    let capsule = capture(root, catalog)?;
    std::fs::write(output, serde_json::to_vec(&capsule)?)
        .with_context(|| format!("writing scientific inputs {}", output.display()))
}

/// Reuse the capture owner's complete file/membership interpretation. This operation
/// writes nothing, builds nothing and executes no scientific model or provider.
pub(super) fn check(root: &Path, catalog: &Path, original: &Path) -> Result<()> {
    let mut original_bytes = Vec::new();
    ensure!(
        std::fs::symlink_metadata(original)?.file_type().is_file(),
        "scientific capsule must be a regular file"
    );
    std::fs::File::open(original)?
        .take(MAX_CAPTURE_BYTES as u64 + 1)
        .read_to_end(&mut original_bytes)?;
    ensure!(
        original_bytes.len() <= MAX_CAPTURE_BYTES,
        "scientific capsule exceeds comparison byte bound"
    );
    let current = serde_json::to_vec(&capture(root, catalog)?)?;
    ensure!(
        current == original_bytes,
        "scientific capsule source bytes or dependency membership changed; explicitly prepare again"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use owner::{Catalog, discover_packages, resolve_families, verify_snapshot};
    use pse_admission::authoring_driver::document::capture_package;
    use pse_authoring::ParseBudget;
    use std::collections::{BTreeMap, BTreeSet};

    fn package(root: &Path, path: &str, identity: u8, dependencies: &[u8], source: &str) {
        let directory = root.join(path);
        std::fs::create_dir_all(directory.join("models")).unwrap();
        let dependencies = dependencies.iter().map(|id| format!(
            "{{package_id=\"{}\",version_req={{operator=\"exact\",major=1,minor=0,patch=0}}}}",
            format!("{id:02x}").repeat(16),
        )).collect::<Vec<_>>().join(",");
        let header = format!(
            r#"[package]
package_id = "{}"
name = "p{identity}"
version = "1.0.0"
kind = "model"
id_policy = "named"
dependencies = [{dependencies}]
doc = "scientific capture control"
"#,
            format!("{identity:02x}").repeat(16)
        );
        std::fs::write(directory.join("package.toml"), header).unwrap();
        std::fs::write(directory.join("models/source.pse"), source).unwrap();
    }

    fn catalog(root: &Path, selection: &str) {
        std::fs::write(
            root.join("inputs.toml"),
            format!(
                r#"version = 1
package_inventory = "inventory"
[[families]]
name = "control"
consumers = ["unit-control"]
{selection}
"#
            ),
        )
        .unwrap();
    }

    fn fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        package(
            root.path(),
            "inventory/a",
            1,
            &[2],
            "package alpha { use beta @\"1.0.0\"; }\n",
        );
        package(root.path(), "inventory/b", 2, &[], "package beta {}\n");
        package(
            root.path(),
            "inventory/unused",
            3,
            &[],
            "package unused {}\n",
        );
        catalog(
            root.path(),
            r#"roots = ["inventory/a"]
documents = ["inventory/a/models/source.pse"]
projection = "declarations""#,
        );
        root
    }

    #[test]
    fn scientific_inputs_check_refuses_changed_deleted_and_added_actual_dependencies() {
        for change in 0..6 {
            let root = fixture();
            let output = tempfile::tempdir().unwrap();
            let original = output.path().join("capsule.json");
            std::fs::write(root.path().join("synthetic.pse"), "package substitute {}\n").unwrap();
            let catalog_path = root.path().join("inputs.toml");
            let catalog = format!(
                "{}\nsubstitutions = [\"synthetic.pse\"]\n",
                std::fs::read_to_string(&catalog_path).unwrap()
            );
            std::fs::write(&catalog_path, &catalog).unwrap();
            run(root.path(), Path::new("inputs.toml"), &original).unwrap();
            let retained = std::fs::read(&original).unwrap();
            check(root.path(), Path::new("inputs.toml"), &original).unwrap();
            match change {
                0 => std::fs::write(
                    &catalog_path,
                    format!("{catalog}\n# changed original catalog bytes\n"),
                )
                .unwrap(),
                1 => std::fs::write(
                    root.path().join("inventory/b/models/source.pse"),
                    "package beta {}\n// changed authored bytes\n",
                )
                .unwrap(),
                2 => {
                    std::fs::remove_file(root.path().join("inventory/b/models/source.pse")).unwrap()
                }
                3 => package(
                    root.path(),
                    "inventory/new",
                    4,
                    &[],
                    "package additional {}\n",
                ),
                4 => std::fs::write(
                    root.path().join("inventory/b/models/additional.pse"),
                    "package extra {}\n",
                )
                .unwrap(),
                _ => std::fs::write(
                    root.path().join("synthetic.pse"),
                    "package substitute {}\n// changed substitution bytes\n",
                )
                .unwrap(),
            }
            assert!(
                check(root.path(), Path::new("inputs.toml"), &original).is_err(),
                "dependency change {change}"
            );
            assert_eq!(
                std::fs::read(&original).unwrap(),
                retained,
                "checking cannot rewrite original capsule"
            );
        }
    }

    #[test]
    fn scientific_inputs_derive_p0_imports_and_source_only_reload() {
        let root = fixture();
        std::fs::create_dir(root.path().join("inventory/b/data")).unwrap();
        let data = "inventory/b/data/bytes.parquet";
        std::fs::write(root.path().join(data), [0, 255, 13, 10]).unwrap();
        let first = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(first.schema, "scientific-inputs-v1");
        assert_eq!(
            first.families["control"].packages,
            ["inventory/a", "inventory/b"]
        );
        assert_eq!(first.families["control"].documents.len(), 2);
        assert_eq!(first.files[data], [0, 255, 13, 10]);
        assert_eq!(first.directories["inventory/a/data"], None);
        assert!(first.files.contains_key("inventory/unused/package.toml"));
        assert!(
            !first
                .files
                .contains_key("inventory/unused/models/source.pse")
        );
        // The same compiled capture code observes a data-only edit and a new
        // imported document, retaining exact CRLF source bytes and full transport.
        std::fs::write(root.path().join(data), [0, 254, 13, 10]).unwrap();
        std::fs::write(
            root.path().join("inventory/b/models/source.pse"),
            "package beta { use gamma @\"1.0.0\"; }\n",
        )
        .unwrap();
        assert!(capture(root.path(), Path::new("inputs.toml")).is_err());
        let added = "inventory/b/models/added.pse";
        let bytes = b"package gamma {}\r\n";
        std::fs::write(root.path().join(added), bytes).unwrap();
        let changed = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(changed.files[added], bytes);
        assert_eq!(changed.files[data], [0, 254, 13, 10]);
        assert!(
            changed.families["control"]
                .documents
                .iter()
                .any(|path| path == added)
        );
        assert!(
            changed.directories["inventory/b/models"]
                .as_ref()
                .unwrap()
                .contains(&"added.pse".into())
        );
        std::fs::remove_file(root.path().join(added)).unwrap();
        assert!(capture(root.path(), Path::new("inputs.toml")).is_err());
    }

    #[test]
    fn scientific_inputs_refuse_ambiguous_scopes_and_missing_package_dependencies() {
        let root = fixture();
        std::fs::write(
            root.path().join("inventory/b/models/duplicate.pse"),
            "package beta {}\n",
        )
        .unwrap();
        let error = capture(root.path(), Path::new("inputs.toml")).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("ambiguous modeling package scope beta")
        );
        std::fs::remove_file(root.path().join("inventory/b/models/duplicate.pse")).unwrap();
        package(root.path(), "inventory/a", 1, &[4], "package alpha {}\n");
        let error = capture(root.path(), Path::new("inputs.toml")).unwrap_err();
        assert!(error.to_string().contains("missing dependency"));
    }

    #[test]
    fn scientific_inputs_keep_parser_projection_and_synthetic_sources_distinct() {
        let root = fixture();
        std::fs::write(
            root.path().join("inventory/a/models/source.pse"),
            "package alpha { use deliberately_absent @\"1.0.0\"; }\n",
        )
        .unwrap();
        catalog(
            root.path(),
            r#"roots = ["inventory/a"]
documents = ["inventory/a/models/source.pse"]
projection = "parser-fields""#,
        );
        let parser_only = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(
            parser_only.families["control"].documents,
            ["inventory/a/models/source.pse"]
        );
        std::fs::write(
            root.path().join("inventory/a/models/source.pse"),
            "package alpha {}\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join("inventory/b/models/extra.pse"),
            "package extra {}\n",
        )
        .unwrap();
        let extra = b"package synthetic { use extra @\"1.0.0\"; }\r\n";
        std::fs::write(root.path().join("synthetic.pse"), extra).unwrap();
        catalog(
            root.path(),
            r#"roots = ["inventory/a"]
documents = ["inventory/a/models/source.pse"]
projection = "synthetic-declarations"
substitutions = ["synthetic.pse"]"#,
        );
        let synthetic = capture(root.path(), Path::new("inputs.toml")).unwrap();
        let family = &synthetic.families["control"];
        assert_eq!(family.substitutions, ["synthetic.pse"]);
        assert!(
            family
                .documents
                .iter()
                .any(|path| path == "inventory/b/models/extra.pse")
        );
        assert!(!family.documents.iter().any(|path| path == "synthetic.pse"));
        assert_eq!(synthetic.files["synthetic.pse"], extra);
        assert_eq!(family.projection.as_deref(), Some("synthetic-declarations"));
    }

    #[test]
    fn scientific_inputs_manifest_owns_run_roots_and_real_version_resolution() {
        let root = fixture();
        let manifest = r#"version = 1
[settings]
intent = "root"
diagnostics = "../diagnostics.json"
[[runs]]
package = "../inventory/a"
dependencies = ["../inventory/b"]
physical = "../inventory/b"
"#;
        let diagnostics = b"{\"limit\":1}\r\n";
        std::fs::write(root.path().join("diagnostics.json"), diagnostics).unwrap();
        std::fs::create_dir(root.path().join("config")).unwrap();
        std::fs::write(root.path().join("config/runs.toml"), manifest).unwrap();
        catalog(
            root.path(),
            r#"manifest = "config/runs.toml"
projection = "authentic-transport""#,
        );
        let captured = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(captured.files["config/runs.toml"], manifest.as_bytes());
        assert_eq!(captured.files["diagnostics.json"], diagnostics);
        assert_eq!(
            captured.families["control"].packages,
            ["inventory/a", "inventory/b"]
        );
        let header = root.path().join("inventory/b/package.toml");
        std::fs::write(
            &header,
            std::fs::read_to_string(&header)
                .unwrap()
                .replace("version = \"1.0.0\"", "version = \"1.0.1\""),
        )
        .unwrap();
        assert!(capture(root.path(), Path::new("inputs.toml")).is_err());
    }

    #[test]
    fn scientific_inputs_inherit_one_projection_and_refuse_missing_or_cyclic_bases() {
        let root = fixture();
        std::fs::write(
            root.path().join("inventory/b/models/extra.pse"),
            "package extra {}\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join("synthetic.pse"),
            "package synthetic { use beta @\"1.0.0\"; }\n",
        )
        .unwrap();
        let base = format!(
            "{}\nsubstitutions = [\"synthetic.pse\"]\n",
            std::fs::read_to_string(root.path().join("inputs.toml")).unwrap()
        );
        let inherited = format!(
            r#"{base}
[[families]]
name = "derived"
extends = "control"
roots = ["inventory/b"]
documents = ["inventory/b/models/extra.pse"]
consumers = ["derived-consumer"]
"#
        );
        std::fs::write(root.path().join("inputs.toml"), &inherited).unwrap();
        let capsule = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(
            capsule.families["derived"].projection,
            capsule.families["control"].projection
        );
        assert_eq!(capsule.families["control"].documents.len(), 2);
        assert_eq!(capsule.families["derived"].documents.len(), 3);
        assert_eq!(capsule.families["derived"].substitutions, ["synthetic.pse"]);
        let declared: Catalog = toml::from_str(&inherited).unwrap();
        let resolved = resolve_families(declared.families).unwrap();
        assert_eq!(
            resolved
                .iter()
                .find(|family| family.name == "derived")
                .unwrap()
                .consumers,
            ["derived-consumer"]
        );
        std::fs::write(
            root.path().join("inputs.toml"),
            inherited.replace("extends = \"control\"", "extends = \"missing\""),
        )
        .unwrap();
        assert!(
            capture(root.path(), Path::new("inputs.toml"))
                .unwrap_err()
                .to_string()
                .contains("missing scientific base family missing")
        );
        std::fs::write(
            root.path().join("inputs.toml"),
            inherited.replace(
                "name = \"control\"",
                "name = \"control\"\nextends = \"derived\"",
            ),
        )
        .unwrap();
        assert!(
            capture(root.path(), Path::new("inputs.toml"))
                .unwrap_err()
                .to_string()
                .contains("inheritance cycle")
        );
    }

    #[test]
    fn scientific_inputs_reject_package_auxiliary_and_inventory_drift_before_publication() {
        let root = fixture();
        let capsule = capture(root.path(), Path::new("inputs.toml")).unwrap();
        let registry = pse_schema::shared_registry().unwrap();
        let packages = capsule
            .package_headers
            .iter()
            .map(|header| {
                let path = header.strip_suffix("/package.toml").unwrap();
                (
                    path.to_owned(),
                    capture_package(&root.path().join(path), &registry, ParseBudget::default())
                        .unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut inventory = BTreeSet::new();
        let mut directories = BTreeMap::new();
        discover_packages(root.path(), "inventory", &mut inventory, &mut directories).unwrap();
        let verify = || {
            verify_snapshot(
                root.path(),
                "inventory",
                &inventory,
                &directories,
                &packages,
                &capsule.files,
            )
        };
        verify().unwrap();
        let source = root.path().join("inventory/b/models/source.pse");
        let original = std::fs::read(&source).unwrap();
        std::fs::write(&source, b"package beta {} // source drift\n").unwrap();
        assert!(
            verify()
                .unwrap_err()
                .to_string()
                .contains("changed during capture")
        );
        std::fs::write(&source, original).unwrap();
        let absent = root.path().join("inventory/b/data");
        std::fs::create_dir(&absent).unwrap();
        assert!(
            verify()
                .unwrap_err()
                .to_string()
                .contains("changed during capture")
        );
        std::fs::remove_dir(absent).unwrap();
        package(root.path(), "inventory/added", 4, &[], "package added {}\n");
        assert!(
            verify()
                .unwrap_err()
                .to_string()
                .contains("inventory changed during capture")
        );
        std::fs::remove_dir_all(root.path().join("inventory/added")).unwrap();
        verify().unwrap();
        let catalog = root.path().join("inputs.toml");
        std::fs::write(
            &catalog,
            format!(
                "{}\n# auxiliary drift\n",
                std::fs::read_to_string(&catalog).unwrap()
            ),
        )
        .unwrap();
        assert!(
            verify()
                .unwrap_err()
                .to_string()
                .contains("auxiliary input changed during capture")
        );
    }
}
