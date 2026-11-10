// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authenticate the single header pin and derive its complete materialization extents.
#[cfg(not(feature = "compiled-evaluator"))]
fn main() {}

#[cfg(feature = "compiled-evaluator")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use sha2::Digest;
    use std::{
        collections::BTreeSet,
        env, fs,
        io::Write,
        path::{Component, Path, PathBuf},
    };
    let manifest =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("math manifest directory absent")?);
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or("workspace directory absent")?;
    let declaration = root.join("Cargo.toml");
    println!("cargo:rerun-if-changed={}", declaration.display());
    let workspace: toml::Value = toml::from_str(&fs::read_to_string(&declaration)?)?;
    let pin = &workspace["workspace"]["metadata"]["pse"]["xsimd"];
    let version = pin["version"].as_str().ok_or("xsimd version absent")?;
    let source = pin["source"].as_str().ok_or("xsimd source absent")?;
    let archive = root.join(pin["archive"].as_str().ok_or("xsimd archive absent")?);
    println!("cargo:rerun-if-changed={}", archive.display());
    let bytes = fs::read(archive)?;
    let digest: [u8; 32] = sha2::Sha256::digest(&bytes).into();
    let hexadecimal = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if pin["sha256"].as_str() != Some(hexadecimal.as_str()) {
        return Err("xsimd archive checksum differs from its single workspace pin".into());
    }
    let prefix = format!("xsimd-{version}");
    let mut entries = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_slice()));
    let mut files = 0usize;
    let mut file_bytes = 0usize;
    let mut name_bytes = 0usize;
    let mut max_name = 0usize;
    let mut directories = BTreeSet::new();
    let mut license = false;
    let mut headers = 0usize;
    for entry in entries.entries()? {
        let entry = entry?;
        if entry.header().entry_type().is_pax_global_extensions()
            || entry.header().entry_type().is_pax_local_extensions()
        {
            continue;
        }
        let path = entry.path()?;
        if path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err("unsafe pinned xsimd archive path".into());
        }
        let relative = path.strip_prefix(&prefix).map_err(|_| {
            format!(
                "xsimd archive entry outside declared root: {}",
                path.display()
            )
        })?;
        let selected = relative.starts_with("include") || relative == Path::new("LICENSE");
        if !selected || entry.header().entry_type().is_dir() {
            continue;
        }
        if !entry.header().entry_type().is_file() {
            return Err("pinned header entry is not a regular file".into());
        }
        let name = path.to_str().ok_or("non-UTF8 pinned header name")?;
        files = files.checked_add(1).ok_or("xsimd file count overflow")?;
        file_bytes = file_bytes
            .checked_add(usize::try_from(entry.size())?)
            .ok_or("xsimd file bytes overflow")?;
        name_bytes = name_bytes
            .checked_add(name.len() + 1)
            .ok_or("xsimd names overflow")?;
        max_name = max_name.max(name.len());
        if relative == Path::new("LICENSE") {
            license = true;
        } else {
            headers += 1;
        }
        for parent in path
            .ancestors()
            .skip(1)
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            directories.insert(parent.to_path_buf());
        }
    }
    if !license || headers == 0 {
        return Err("pinned header archive lacks headers or license".into());
    }
    for directory in &directories {
        name_bytes = name_bytes
            .checked_add(directory.as_os_str().len() + 1)
            .ok_or("xsimd directory names overflow")?;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("build output directory absent")?);
    fs::write(out.join("xsimd-source.tar.gz"), &bytes)?;
    let mut generated = fs::File::create(out.join("xsimd-header-data.rs"))?;
    writeln!(
        generated,
        "const VERSION: &str = {version:?};\nconst SOURCE: &str = {source:?};\nconst FINGERPRINT: [u8;32] = {digest:?};\nconst ROOT: &str = {prefix:?};"
    )?;
    writeln!(
        generated,
        "const FILE_COUNT: usize = {files};\nconst FILE_BYTES: usize = {file_bytes};\nconst DIRECTORY_COUNT: usize = {};\nconst NAME_BYTES: usize = {name_bytes};\nconst MAX_NAME: usize = {max_name};",
        directories.len()
    )?;
    writeln!(
        generated,
        "const ARCHIVE: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/xsimd-source.tar.gz\"));"
    )?;
    Ok(())
}
