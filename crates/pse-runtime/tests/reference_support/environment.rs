// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Receive the performing reference process's environment and stage observations.

use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead as _, BufReader},
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn pins(root: &Path) -> Result<BTreeMap<String, String>> {
    let project: toml::Value =
        toml::from_str(&std::fs::read_to_string(root.join("pyproject.toml"))?)?;
    let group = project
        .get("dependency-groups")
        .and_then(|groups| groups.get("parity"))
        .and_then(toml::Value::as_array)
        .ok_or("selected parity dependency group absent")?;
    ["idaes-pse", "pyomo"]
        .into_iter()
        .map(|name| {
            let prefix = format!("{name}==");
            let pin = group
                .iter()
                .filter_map(toml::Value::as_str)
                .find_map(|entry| entry.strip_prefix(&prefix))
                .ok_or_else(|| format!("selected parity pin for {name} absent"))?
                .split(';')
                .next()
                .ok_or_else(|| format!("selected parity pin for {name} is empty"))?
                .trim();
            Ok((name.into(), pin.into()))
        })
        .collect()
}

#[allow(
    dead_code,
    reason = "ordinary reference returns an attributable interpreter refusal"
)]
pub fn interpreter(role: &str) -> PathBuf {
    let python = PathBuf::from(
        std::env::var_os(role)
            .unwrap_or_else(|| panic!("select the locked interpreter with {role}")),
    );
    assert!(
        python.is_absolute() && python.is_file(),
        "{role} must identify an absolute existing interpreter"
    );
    python
}

pub struct PreparedSolver {
    path: PathBuf,
    digest: String,
}

pub fn prepared(role: &str, name: &str) -> Result<PreparedSolver> {
    let directory = std::env::var_os(role)
        .map(PathBuf::from)
        .or_else(|| {
            // IDAES's ASL PETSc extension is distinct from the native PETSc provider.
            (role == "IDAES_DATA")
                .then(|| std::env::var_os("HOME"))
                .flatten()
                .map(|home| PathBuf::from(home).join(".idaes"))
        })
        .ok_or_else(|| format!("prepared role {role} absent"))?;
    let path = directory.join("bin").join(name).canonicalize()?;
    let digest = image_digest(&path)?;
    Ok(PreparedSolver { path, digest })
}

pub fn image_digest(path: &Path) -> io::Result<String> {
    let mut digest = Sha256::new();
    let mut file = BufReader::new(File::open(path)?);
    loop {
        let chunk = file.fill_buf()?;
        if chunk.is_empty() {
            break;
        }
        digest.update(chunk);
        let length = chunk.len();
        file.consume(length);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub fn verify(
    evidence: &BTreeMap<String, String>,
    python: &Path,
    interpreter_digest: &str,
    pins: &BTreeMap<String, String>,
) -> Result<()> {
    for (name, pin) in pins {
        if evidence.get(name) != Some(pin) {
            return Err(format!("reference package {name} differs from {pin}").into());
        }
    }
    if !evidence
        .get("python")
        .is_some_and(|value| !value.is_empty())
        || evidence.get("python.executable").map(Path::new) != Some(python)
        || evidence.get("python.sha256").map(String::as_str) != Some(interpreter_digest)
    {
        return Err("reference interpreter image differs from the selected image".into());
    }
    Ok(())
}

pub fn verify_stage(
    evidence: &BTreeMap<String, String>,
    stage: &str,
    selected: &PreparedSolver,
    solves: RangeInclusive<usize>,
) -> Result<()> {
    let observed = |field| {
        evidence
            .get(&format!("{stage}.{field}"))
            .map(String::as_str)
    };
    if observed("executable").map(Path::new) != Some(selected.path.as_path())
        || observed("sha256") != Some(selected.digest.as_str())
        || !observed("release").is_some_and(|value| !value.is_empty())
        || !observed("solves")
            .and_then(|value| value.parse::<usize>().ok())
            .is_some_and(|count| solves.contains(&count))
    {
        return Err(
            format!("performed reference solver evidence differs for stage {stage}").into(),
        );
    }
    Ok(())
}
