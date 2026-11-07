// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One explicitly acquired, offline-built CoolProp image and its complete input closure.
use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    name: String,
    archive: String,
    sha256: String,
}

fn unpack(archive: &Path, destination: &Path, env: &Environment) -> Result<PathBuf, HostError> {
    fs::create_dir(destination)?;
    let tar = resolve_program(OsStr::new("tar"), env)?;
    run(
        &tar,
        &[
            "-xzf".into(),
            archive.as_os_str().into(),
            "-C".into(),
            destination.as_os_str().into(),
            "--no-same-owner".into(),
        ],
        env,
    )?;
    let entries = fs::read_dir(destination)?.collect::<Result<Vec<_>, _>>()?;
    if entries.len() != 1 || !entries[0].file_type()?.is_dir() {
        return Err(invalid(
            "CoolProp input archive must have one directory root",
        ));
    }
    Ok(entries[0].path())
}

fn normalize_distribution(install: &Path) -> Result<(), HostError> {
    let distribution = install.join("shared_library/Linux/64bit");
    fs::create_dir(install.join("lib"))?;
    // Copy while upstream's relative SONAME links still resolve, then retire only
    // this owned staging distribution. The published prefix has one image path.
    fs::copy(
        distribution.join("libCoolProp.so"),
        install.join("lib/libCoolProp.so"),
    )?;
    fs::remove_dir_all(distribution)?;
    fs::rename(
        install.join("shared_library/include"),
        install.join("include"),
    )?;
    Ok(())
}

pub(super) fn prepare(root: &Path, env: &Environment) -> Result<PathBuf, HostError> {
    if text(env, "COOLPROP_DIR")?.is_some() {
        return Err(invalid(
            "CoolProp preparation requires its declared source recipe, not an external image",
        ));
    }
    let pin = pin(root, "coolprop")?;
    let dependencies: Vec<Source> = pin
        .extra
        .get("dependencies")
        .ok_or_else(|| invalid("CoolProp dependency closure absent"))?
        .clone()
        .try_into()
        .map_err(|_| invalid("invalid CoolProp dependency closure"))?;
    let mut names = BTreeSet::new();
    for source in &dependencies {
        if source.name.is_empty()
            || !source
                .name
                .bytes()
                .all(|v| v.is_ascii_alphanumeric() || v == b'_' || v == b'-')
            || !names.insert(source.name.clone())
        {
            return Err(invalid("invalid or duplicate CoolProp dependency name"));
        }
    }
    let mut identity = identity(root, env, "coolprop")?;
    identity.pin = Some(serde_json::to_value(&pin).map_err(|_| invalid("invalid CoolProp pin"))?);
    let patch = root.join(
        pin.patch
            .as_ref()
            .ok_or_else(|| invalid("CoolProp ABI patch absent"))?,
    );
    identity
        .construction
        .insert("patch-sha256".into(), digest(&patch)?);
    let python = resolve_program(OsStr::new("python3"), env)?;
    identity.tools.insert(
        "header-generator-python".into(),
        Tool {
            sha256: digest(&python)?,
            version: String::from_utf8(capture(&python, &[OsStr::new("--version")], env)?)
                .map_err(|_| invalid("invalid Python version"))?,
            path: python.clone(),
        },
    );
    identity.inputs.insert(
        "SOURCE_DATE_EPOCH".into(),
        text(env, "SOURCE_DATE_EPOCH")?.map(str::to_owned),
    );
    let mut effective = apply_inputs(env, &identity)?;
    cache_bridge(root, &mut effective);
    let width = jobs(&effective)?;
    // All network acquisition precedes configuration. Every input has a declared SHA256.
    let archive = source_archive(&effective, &pin)?;
    let mut archives = Vec::new();
    for source in &dependencies {
        let source_pin = Pin {
            archive: source.archive.clone(),
            sha256: source.sha256.clone(),
            options: Vec::new(),
            patch: None,
            extra: BTreeMap::new(),
        };
        archives.push(source_archive(&effective, &source_pin)?);
    }
    let inputs = json(&identity)?;
    commit_provider(&effective, identity, |install, destination, work| {
        let source = unpack(&archive, &work.join("source"), &effective)?;
        let git = resolve_program(OsStr::new("git"), &effective)?;
        run_at(
            &git,
            &["apply".into(), "--check".into(), patch.as_os_str().into()],
            &effective,
            &source,
        )?;
        run_at(
            &git,
            &["apply".into(), patch.as_os_str().into()],
            &effective,
            &source,
        )?;
        let revision = pin
            .extra
            .get("revision")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| invalid("CoolProp upstream revision absent"))?;
        fs::write(source.join("dev/gitrevision.txt"), revision)?;
        let build = work.join("compiled");
        let mut options: Vec<OsString> = vec![
            "-S".into(),
            source.as_os_str().into(),
            "-B".into(),
            build.as_os_str().into(),
            format!("-DCOOLPROP_INSTALL_PREFIX={}", install.display()).into(),
            format!("-DPython_EXECUTABLE={}", python.display()).into(),
            "-DCPM_LOCAL_PACKAGES_ONLY=ON".into(),
            "-DFETCHCONTENT_FULLY_DISCONNECTED=ON".into(),
        ];
        for (dependency, archive) in dependencies.iter().zip(&archives) {
            let path = unpack(
                archive,
                &work.join(format!("dependency-{}", dependency.name)),
                &effective,
            )?;
            options.push(format!("-DCPM_{}_SOURCE={}", dependency.name, path.display()).into());
        }
        options.extend(
            pin.options
                .iter()
                .map(|option| format!("-D{option}").into()),
        );
        let cmake = resolve_program(OsStr::new("cmake"), &effective)?;
        run(&cmake, &options, &effective)?;
        run(
            &cmake,
            &[
                "--build".into(),
                build.as_os_str().into(),
                "--parallel".into(),
                width.into(),
            ],
            &effective,
        )?;
        run(
            &cmake,
            &["--install".into(), build.as_os_str().into()],
            &effective,
        )?;
        // Upstream installs its distribution layout rather than a Unix provider prefix.
        // Move the original image/header tree; do not introduce a second image or loader.
        normalize_distribution(install)?;
        let notes = install.join("share/pse-coolprop");
        fs::create_dir_all(&notes)?;
        fs::write(notes.join("inputs.json"), &inputs)?;
        relocate_metadata(install, install, destination)?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_distribution_soname_chain_becomes_one_unix_provider_image() {
        let staging = tempfile::tempdir().unwrap();
        let original = staging.path().join("shared_library/Linux/64bit");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(staging.path().join("shared_library/include/CoolProp")).unwrap();
        fs::write(original.join("libCoolProp.so.8.0.0"), b"original image").unwrap();
        std::os::unix::fs::symlink("libCoolProp.so.8.0.0", original.join("libCoolProp.so.8"))
            .unwrap();
        std::os::unix::fs::symlink("libCoolProp.so.8", original.join("libCoolProp.so")).unwrap();
        normalize_distribution(staging.path()).unwrap();
        assert_eq!(
            fs::read(staging.path().join("lib/libCoolProp.so")).unwrap(),
            b"original image"
        );
        assert!(!original.exists());
        assert!(staging.path().join("include/CoolProp").is_dir());
    }
}
