// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Authenticate maintained supplier inputs once, while building the loaded artifact.

use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, error::Error, fs, path::Path};

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn selected_sundials(root: &Path, kind: &str, receipt: Option<&Path>) -> Result<String, Box<dyn Error>> {
    match kind {
        "bundled" => authenticated(root, "sundials-sys"),
        "external" => {
            let receipt = receipt.ok_or("selected external SUNDIALS requires PSE_SUNDIALS_PROVIDER_RECEIPT")?;
            println!("cargo:rerun-if-changed={}", receipt.display());
            let bytes = fs::read(receipt)?;
            let document: serde_json::Value = serde_json::from_slice(&bytes)?;
            for field in ["identity", "files"] {
                if document[field].as_object().is_none_or(|values| values.is_empty()) {
                    return Err("selected external SUNDIALS receipt requires identity and files".into());
                }
            }
            Ok(format!("external-sundials:{}", String::from_utf8(bytes)?))
        }
        _ => Err("unknown selected SUNDIALS source kind".into()),
    }
}

/// The receipt is derived by reconstructing the published archive plus complete
/// maintained delta. Refuse stale receipts, changed/added/deleted source and modes.
/// No Cargo cache or serving-time source scan participates in native identity.
pub(crate) fn authenticated(root: &Path, name: &str) -> Result<String, Box<dyn Error>> {
    let tree = root.join("vendor").join(name);
    let receipt = root.join(format!("vendor/deltas/{name}.source.json"));
    let delta = root.join(format!("vendor/deltas/{name}.patch"));
    for path in [&tree, &receipt, &delta] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let bytes = fs::read(&receipt)?;
    let document: serde_json::Value = serde_json::from_slice(&bytes)?;
    if document["archive_sha256"].as_str().is_none_or(|hash| hash.len() != 64)
        || document["delta_sha256"].as_str() != Some(sha256(&fs::read(delta)?).as_str())
    {
        return Err(format!("{name}: stale supplier source receipt; refresh scripts/supplier_source.py").into());
    }
    let files = document["files"].as_object().ok_or("missing supplier source paths")?;
    if files.is_empty() {
        return Err("empty supplier source receipt".into());
    }
    let mut actual = BTreeSet::new();
    paths(&tree, &tree, &mut actual)?;
    let expected: BTreeSet<_> = files.keys().cloned().collect();
    if actual != expected {
        return Err(format!("{name}: supplier source path set changed; refresh complete delta and receipt").into());
    }
    for (relative, facts) in files {
        let path = tree.join(relative);
        if facts["sha256"].as_str() != Some(sha256(&fs::read(&path)?).as_str()) {
            return Err(format!("{name}: changed source {relative}; refresh complete delta and receipt").into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let executable = fs::metadata(path)?.permissions().mode() & 0o111 != 0;
            if executable != (facts["mode"].as_str() == Some("100755")) {
                return Err(format!("{name}: source mode changed for {relative}").into());
            }
        }
    }
    // The checked receipt includes the authenticated baseline, complete delta and
    // exact source path set. Its digest is an input, not a new semantic hash frame.
    Ok(format!("{name}:checked-archive-delta-sha256:{}", sha256(&bytes)))
}

fn paths(root: &Path, directory: &Path, output: &mut BTreeSet<String>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(root)?;
        if ["target", ".git"].iter().any(|name| relative.components().any(|part| part.as_os_str() == *name))
            || relative == Path::new("PATCH.md")
        {
            continue;
        }
        if entry.file_type()?.is_dir() {
            paths(root, &path, output)?;
        } else if entry.file_type()?.is_file() {
            output.insert(relative.to_str().ok_or("non-UTF8 supplier source path")?.replace('\\', "/"));
        } else {
            return Err("supplier source symlinks are not authenticated".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_external_receipt_never_substitutes_bundled_source() {
        let root = std::env::temp_dir().join(format!("pse-external-source-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let receipt = root.join("selected.json");
        assert!(selected_sundials(&root, "external", None).is_err());
        fs::write(&receipt, b"{}").unwrap();
        assert!(selected_sundials(&root, "external", Some(&receipt)).is_err());
        fs::write(&receipt, br#"{"identity":{"provider":"selected-v1"},"files":{"lib/libsundials_idas.a":"digest"}}"#).unwrap();
        let first = selected_sundials(&root, "external", Some(&receipt)).unwrap();
        assert_eq!(selected_sundials(&root, "external", Some(&receipt)).unwrap(), first);
        // No bundled source directory exists: only the selected provider is consumed.
        assert!(selected_sundials(&root, "bundled", Some(&receipt)).is_err());
        fs::write(&receipt, br#"{"identity":{"provider":"selected-v2"},"files":{"lib/libsundials_idas.a":"digest"}}"#).unwrap();
        assert_ne!(selected_sundials(&root, "external", Some(&receipt)).unwrap(), first);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn supplier_source_authentication_refuses_stale_delta_and_source() {
        let root = std::env::temp_dir().join(format!("pse-source-{}", std::process::id()));
        fs::create_dir_all(root.join("vendor/supplier/src")).unwrap();
        fs::create_dir_all(root.join("vendor/deltas")).unwrap();
        let source = root.join("vendor/supplier/src/lib.rs");
        let delta = root.join("vendor/deltas/supplier.patch");
        fs::write(&source, b"original source").unwrap();
        fs::write(&delta, b"complete delta").unwrap();
        let receipt = serde_json::json!({"archive_sha256": "a".repeat(64),
            "delta_sha256": sha256(b"complete delta"),
            "files": {"src/lib.rs": {"sha256": sha256(b"original source"), "mode": "100644"}}});
        fs::write(root.join("vendor/deltas/supplier.source.json"), receipt.to_string()).unwrap();
        let identity = authenticated(&root, "supplier").unwrap();
        assert_eq!(authenticated(&root, "supplier").unwrap(), identity);
        fs::write(&source, b"changed same-version source").unwrap();
        assert!(authenticated(&root, "supplier").is_err());
        fs::write(&source, b"original source").unwrap();
        fs::write(&delta, b"changed delta").unwrap();
        assert!(authenticated(&root, "supplier").is_err());
        fs::write(&delta, b"complete delta").unwrap();
        fs::write(root.join("vendor/supplier/src/new.rs"), b"new source").unwrap();
        assert!(authenticated(&root, "supplier").is_err());
        fs::remove_file(root.join("vendor/supplier/src/new.rs")).unwrap();
        fs::remove_file(source).unwrap();
        assert!(authenticated(&root, "supplier").is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
