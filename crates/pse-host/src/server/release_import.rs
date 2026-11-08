// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit snapshot admission of an established source release, without rebuilding it.
use super::*;
fn decode_release(bytes: &[u8], source: &SourceIdentity) -> Result<Receipt, HostError> {
    let receipt: Receipt = serde_json::from_slice(bytes)
        .map_err(|error| invalid(format!("invalid source release: {error}")))?;
    if receipt.schema != 1
        || !source.supported()
        || &receipt.identity.source != source
        || !matches!(&receipt.identity.origin, ConstructionOrigin::Built { context } if context.complete())
        || receipt
            .files
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != BTreeSet::from(["surreal", "surreal-ext", "Cargo.lock"])
        || receipt
            .files
            .values()
            .any(|digest| !hexadecimal(digest, 64))
    {
        return Err(invalid(
            "source release lacks the exact prepared pair and original build context",
        ));
    }
    Ok(receipt)
}
fn open_regular(path: &Path) -> Result<File, HostError> {
    // Bind the file description without following any symlink component. Hash the
    // bytes read from that same description during the copy, never a reopened path.
    let descriptor = rustix::fs::openat2(
        rustix::fs::CWD,
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NONBLOCK,
        rustix::fs::Mode::empty(),
        rustix::fs::ResolveFlags::NO_SYMLINKS,
    )
    .map_err(std::io::Error::from)?;
    let file = File::from(descriptor);
    if !file.metadata()?.is_file() {
        return Err(invalid("source release artifacts must be regular files"));
    }
    Ok(file)
}
struct Candidate {
    root: PathBuf,
    identity: Identity,
    receipt: Vec<u8>,
    binary: File,
    extension_builder: File,
    lock: File,
    binary_sha256: String,
    extension_builder_sha256: String,
    lock_sha256: String,
}
fn admit(root: &Path, path: &Path) -> Result<Candidate, HostError> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut receipt = Vec::new();
    open_regular(&path)?.read_to_end(&mut receipt)?;
    let source = source_identity(root)?;
    let release = decode_release(&receipt, &source)?;
    let directory = path
        .parent()
        .ok_or_else(|| invalid("source release has no installation directory"))?;
    let open_executable = |path: &Path| {
        let file = open_regular(path)?;
        if file.metadata()?.permissions().mode() & 0o111 == 0 {
            return Err(invalid("source release artifact is not executable"));
        }
        Ok(file)
    };
    let binary = open_executable(&directory.join("surreal"))?;
    let extension_builder = open_executable(&directory.join("surreal-ext"))?;
    let lock = open_regular(&directory.join("Cargo.lock"))?;
    Ok(Candidate {
        root: root.to_path_buf(),
        identity: Identity {
            source,
            origin: ConstructionOrigin::ImportedRelease {
                release_sha256: hash(&receipt),
            },
        },
        receipt,
        binary,
        extension_builder,
        lock,
        binary_sha256: release.files["surreal"].clone(),
        extension_builder_sha256: release.files["surreal-ext"].clone(),
        lock_sha256: release.files["Cargo.lock"].clone(),
    })
}
fn verify_bytes(
    input: &mut impl Read,
    mut output: impl Write,
    expected: &str,
) -> Result<(), HostError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let size = input.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        output.write_all(&buffer[..size])?;
        hasher.update(&buffer[..size]);
    }
    let actual: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if actual != expected {
        return Err(invalid(
            "source release bytes differ from the original receipt",
        ));
    }
    Ok(())
}
fn copy_verified(input: &mut impl Read, path: &Path, expected: &str) -> Result<(), HostError> {
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)?;
    verify_bytes(input, &mut output, expected)?;
    output.sync_all()?;
    Ok(())
}
const LOCK_PINS: [(&str, &str); 8] = [
    (SERVER_PACKAGE, VERSION),
    ("surrealdb", VERSION),
    ("surrealdb-core", VERSION),
    ("surrealdb-types", VERSION),
    ("surrealdb-protocol", PROTOCOL),
    ("surrealmx", ENGINE),
    ("surrealism-runtime", SURREALISM),
    (EXTENSION_PACKAGE, EXTENSION_VERSION),
];
pub(super) fn lock_pins(path: &Path) -> Result<(), HostError> {
    let lock: toml::Value = toml::from_str(&fs::read_to_string(path)?)
        .map_err(|error| invalid(format!("invalid imported lockfile: {error}")))?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| invalid("imported lockfile has no packages"))?;
    for (name, version) in LOCK_PINS {
        let found = packages
            .iter()
            .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some(name))
            .collect::<Vec<_>>();
        if found.len() != 1
            || found[0].get("version").and_then(toml::Value::as_str) != Some(version)
        {
            return Err(invalid(format!(
                "imported lockfile differs from pinned {name} {version}"
            )));
        }
    }
    Ok(())
}
#[cfg(test)]
pub(super) fn fixture_lock() -> String {
    let mut lock = String::from("version = 4\n");
    for (name, version) in LOCK_PINS {
        lock.push_str(&format!(
            "[[package]]\nname='{name}'\nversion='{version}'\n"
        ));
    }
    lock
}
fn import_candidate(
    environment: &Environment,
    mut candidate: Candidate,
) -> Result<PreparedServer, HostError> {
    let profile = profile_root(environment)?;
    private_directory(&profile)?;
    let recipe = hash(&encode(&candidate.identity)?);
    let locks = profile.join("locks");
    private_directory(&locks)?;
    let lock_path = locks.join(&recipe);
    if lock_path.exists() {
        regular(&lock_path)?;
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(lock_path)?;
    lock.lock()?;
    if let Ok(current) = select(environment)
        && current.recipe == recipe
    {
        // Validate the newly supplied descriptors even when the immutable destination
        // already matches. Reuse should not make another full staging copy.
        verify_bytes(
            &mut candidate.binary,
            std::io::sink(),
            &candidate.binary_sha256,
        )?;
        verify_bytes(
            &mut candidate.extension_builder,
            std::io::sink(),
            &candidate.extension_builder_sha256,
        )?;
        verify_bytes(&mut candidate.lock, std::io::sink(), &candidate.lock_sha256)?;
        if source_identity(&candidate.root)? != candidate.identity.source {
            return Err(invalid(
                "selected source/package premises changed during release import",
            ));
        }
        return Ok(current);
    }
    let staging = profile.join("staging");
    private_directory(&staging)?;
    let scratch = tempfile::Builder::new()
        .prefix("import-")
        .tempdir_in(staging)?;
    let install = scratch.path().join("install");
    fs::create_dir(&install)?;
    copy_verified(
        &mut candidate.binary,
        &install.join("surreal"),
        &candidate.binary_sha256,
    )?;
    copy_verified(
        &mut candidate.extension_builder,
        &install.join("surreal-ext"),
        &candidate.extension_builder_sha256,
    )?;
    copy_verified(
        &mut candidate.lock,
        &install.join("Cargo.lock"),
        &candidate.lock_sha256,
    )?;
    lock_pins(&install.join("Cargo.lock"))?;
    fs::write(install.join("release.json"), &candidate.receipt)?;
    File::open(install.join("release.json"))?.sync_all()?;
    if source_identity(&candidate.root)? != candidate.identity.source {
        return Err(invalid(
            "selected source/package premises changed during release import; no selection published",
        ));
    }
    publish_install(&profile, &install, &candidate.identity)
}
pub(super) fn import_inner(
    root: &Path,
    environment: &Environment,
    path: &Path,
) -> Result<PreparedServer, HostError> {
    import_candidate(environment, admit(root, path)?)
}
pub(super) fn origin_complete(
    generation: &Path,
    identity: &Identity,
    files: &BTreeMap<String, String>,
) -> Result<bool, HostError> {
    match &identity.origin {
        ConstructionOrigin::Built { context } => Ok(context.complete()),
        ConstructionOrigin::ImportedRelease { release_sha256 } => {
            if !hexadecimal(release_sha256, 64) {
                return Ok(false);
            }
            let bytes = fs::read(generation.join("release.json"))?;
            if hash(&bytes) != *release_sha256 {
                return Ok(false);
            }
            let release = decode_release(&bytes, &identity.source)?;
            lock_pins(&generation.join("Cargo.lock"))?;
            Ok(release
                .files
                .iter()
                .all(|(name, digest)| files.get(name) == Some(digest)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    struct Fixture {
        directory: tempfile::TempDir,
        root: PathBuf,
        release: PathBuf,
        environment: Environment,
    }
    fn writable(path: &Path) {
        let metadata = fs::symlink_metadata(path).unwrap();
        if metadata.is_dir() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
            for entry in fs::read_dir(path).unwrap() {
                writable(&entry.unwrap().path());
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            writable(self.directory.path());
        }
    }
    fn fixture() -> Fixture {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("checkout");
        let patches = root.join("scripts/surrealdb-source");
        fs::create_dir_all(&patches).unwrap();
        for name in PATCHES {
            fs::write(patches.join(name), format!("original patch {name}")).unwrap();
        }
        fs::write(patches.join(DISKANN_PATCH), "original DiskANN patch").unwrap();
        let source = source_identity(&root).unwrap();
        let original = directory.path().join("original");
        fs::create_dir(&original).unwrap();
        let binary = original.join("surreal");
        fs::write(&binary, b"#!/bin/sh\nprintf 'original server\\n'\n").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        let extension_builder = original.join("surreal-ext");
        fs::write(
            &extension_builder,
            b"#!/bin/sh\nprintf 'original builder\\n'\n",
        )
        .unwrap();
        fs::set_permissions(&extension_builder, fs::Permissions::from_mode(0o755)).unwrap();
        let lock = original.join("Cargo.lock");
        fs::write(&lock, fixture_lock()).unwrap();
        let tool = Tool {
            path: "/original/bin/tool".into(),
            sha256: "b".repeat(64),
        };
        let context = BuildContext {
            root: "/original-checkout".into(),
            target: None,
            configuration: BTreeMap::from([("/original-checkout/.cargo/config.toml".into(), None)]),
            toolchain_declaration: "a".repeat(64),
            tools: [
                "cargo",
                "rustc",
                "rustdoc",
                "git",
                "curl",
                "tar",
                "CC",
                "CXX",
                "CLANG_PATH",
            ]
            .into_iter()
            .map(|name| (name.into(), tool.clone()))
            .collect(),
            toolchain: ["cargo", "rustc", "rustdoc"]
                .into_iter()
                .map(|name| (name.into(), format!("original {name}")))
                .collect(),
            native: BTreeMap::from([("/original/lib/native.so".into(), "c".repeat(64))]),
            environment: BTreeMap::from([
                ("HOME".into(), "d".repeat(64)),
                ("PATH".into(), "e".repeat(64)),
            ]),
        };
        let release = original.join(".complete.json");
        let receipt = Receipt {
            schema: 1,
            identity: Identity {
                source,
                origin: ConstructionOrigin::Built {
                    context: Box::new(context),
                },
            },
            files: file_map(&original).unwrap(),
        };
        fs::write(&release, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        let mut environment = Environment::new();
        environment.insert(
            "PSE_SURREAL_TOOLS".into(),
            directory.path().join("native-tools").into_os_string(),
        );
        Fixture {
            directory,
            root,
            release,
            environment,
        }
    }
    fn rewrite(fixture: &Fixture, change: impl FnOnce(&mut Value)) {
        let mut receipt: Value =
            serde_json::from_slice(&fs::read(&fixture.release).unwrap()).unwrap();
        change(&mut receipt);
        fs::write(&fixture.release, encode(&receipt).unwrap()).unwrap();
    }
    #[test]
    fn server_import_preserves_verbatim_original_context_and_selects_after_original_removed() {
        let fixture = fixture();
        let original = fs::read(&fixture.release).unwrap();
        let prepared = import_inner(&fixture.root, &fixture.environment, &fixture.release).unwrap();
        assert_eq!(
            fs::read(prepared.generation.join("release.json")).unwrap(),
            original
        );
        let receipt: Receipt = read_record(&prepared.generation.join(".complete.json")).unwrap();
        assert!(matches!(
            receipt.identity.origin,
            ConstructionOrigin::ImportedRelease { .. }
        ));
        let published = fs::read_to_string(prepared.generation.join(".complete.json")).unwrap();
        assert!(!published.contains("current-compiler"));
        fs::remove_dir_all(fixture.release.parent().unwrap()).unwrap();
        fs::remove_dir_all(&fixture.root).unwrap();
        assert_eq!(
            select(&fixture.environment).unwrap().generation(),
            prepared.generation()
        );
        assert_eq!(
            fs::read(prepared.executable()).unwrap(),
            b"#!/bin/sh\nprintf 'original server\\n'\n"
        );
        assert_eq!(
            fs::read(prepared.extension_builder()).unwrap(),
            b"#!/bin/sh\nprintf 'original builder\\n'\n"
        );
    }
    #[test]
    fn server_import_reuses_matching_generation_without_staging_and_checks_input_bytes() {
        let fixture = fixture();
        let prepared = import_inner(&fixture.root, &fixture.environment, &fixture.release).unwrap();
        let stage = profile_root(&fixture.environment).unwrap().join("staging");
        fs::remove_dir(&stage).unwrap();
        fs::write(&stage, b"unexpected staging entry").unwrap();
        let repeated = import_inner(&fixture.root, &fixture.environment, &fixture.release).unwrap();
        assert_eq!(repeated.generation(), prepared.generation());
        let binary = fixture.release.parent().unwrap().join("surreal");
        fs::write(binary, b"changed source bytes").unwrap();
        let failure = import_inner(&fixture.root, &fixture.environment, &fixture.release)
            .unwrap_err()
            .to_string();
        assert!(failure.contains("source release bytes differ"), "{failure}");
        assert_eq!(
            select(&fixture.environment).unwrap().generation(),
            prepared.generation()
        );
    }
    #[test]
    fn server_import_wrong_patch_or_package_identity_refuses() {
        for key in [
            "patch",
            "patch_order",
            "features",
            "config",
            "revision",
            "engine",
            "surrealism",
            "context",
        ] {
            let fixture = fixture();
            rewrite(&fixture, |value| match key {
                "patch" => {
                    value["identity"]["source"]["patches"][PATCHES[0]] =
                        Value::String("f".repeat(64))
                }
                "patch_order" => value["identity"]["source"]["patch_order"] = serde_json::json!([]),
                "features" => {
                    value["identity"]["source"]["features"] = Value::String("other".into())
                }
                "config" => {
                    value["identity"]["source"]["build_config"] =
                        Value::String("profile.release.panic=abort".into())
                }
                "revision" => {
                    value["identity"]["source"]["revision"] = Value::String("f".repeat(40))
                }
                "engine" => value["identity"]["source"]["engine"] = Value::String("other".into()),
                "surrealism" => {
                    value["identity"]["source"]["surrealism"] = Value::String("other".into())
                }
                _ => value["identity"]["origin"]["context"]["native"] = serde_json::json!({}),
            });
            assert!(
                import_inner(&fixture.root, &fixture.environment, &fixture.release).is_err(),
                "{key}"
            );
            assert!(
                !profile_root(&fixture.environment)
                    .unwrap()
                    .join("current.json")
                    .exists()
            );
        }
    }
    #[test]
    fn server_import_binary_or_lock_hash_failure_preserves_prior_selection() {
        for key in ["surreal", "surreal-ext", "Cargo.lock"] {
            let fixture = fixture();
            let prepared =
                import_inner(&fixture.root, &fixture.environment, &fixture.release).unwrap();
            let before = fs::read(
                profile_root(&fixture.environment)
                    .unwrap()
                    .join("current.json"),
            )
            .unwrap();
            rewrite(&fixture, |value| {
                value["files"][key] = Value::String("0".repeat(64))
            });
            assert!(import_inner(&fixture.root, &fixture.environment, &fixture.release).is_err());
            assert_eq!(
                fs::read(
                    profile_root(&fixture.environment)
                        .unwrap()
                        .join("current.json")
                )
                .unwrap(),
                before
            );
            assert_eq!(
                select(&fixture.environment).unwrap().generation(),
                prepared.generation()
            );
        }
    }
    #[test]
    fn server_import_matching_lock_hash_with_wrong_engine_resolution_refuses() {
        let fixture = fixture();
        let lock = fixture.release.parent().unwrap().join("Cargo.lock");
        let contents = fs::read_to_string(&lock).unwrap().replace(
            &format!("name='surrealmx'\nversion='{ENGINE}'"),
            "name='surrealmx'\nversion='1.0.0'",
        );
        fs::write(&lock, contents).unwrap();
        rewrite(&fixture, |value| {
            value["files"]["Cargo.lock"] = Value::String(digest(&lock).unwrap())
        });
        assert!(import_inner(&fixture.root, &fixture.environment, &fixture.release).is_err());
        assert!(
            !profile_root(&fixture.environment)
                .unwrap()
                .join("current.json")
                .exists()
        );
    }
    #[test]
    fn server_import_hashes_the_copied_open_description_after_source_mutation() {
        let fixture = fixture();
        let prepared = import_inner(&fixture.root, &fixture.environment, &fixture.release).unwrap();
        let candidate = admit(&fixture.root, &fixture.release).unwrap();
        let binary = fixture.release.parent().unwrap().join("surreal");
        fs::write(binary, b"mutated after admission before snapshot").unwrap();
        assert!(import_candidate(&fixture.environment, candidate).is_err());
        assert_eq!(
            select(&fixture.environment).unwrap().generation(),
            prepared.generation()
        );
    }
    #[test]
    fn server_import_open_description_cannot_be_redirected_by_source_path_replacement() {
        let fixture = fixture();
        let candidate = admit(&fixture.root, &fixture.release).unwrap();
        let original_sha = candidate.binary_sha256.clone();
        let binary = fixture.release.parent().unwrap().join("surreal");
        fs::rename(&binary, binary.with_extension("previous")).unwrap();
        fs::write(&binary, b"replacement pathname").unwrap();
        let prepared = import_candidate(&fixture.environment, candidate).unwrap();
        assert_eq!(digest(&prepared.executable()).unwrap(), original_sha);
    }
    #[test]
    fn server_import_changed_selected_patches_during_copy_refuses_publication() {
        let fixture = fixture();
        let candidate = admit(&fixture.root, &fixture.release).unwrap();
        fs::write(
            fixture
                .root
                .join("scripts/surrealdb-source")
                .join(PATCHES[0]),
            b"changed source premise",
        )
        .unwrap();
        assert!(import_candidate(&fixture.environment, candidate).is_err());
        assert!(
            !profile_root(&fixture.environment)
                .unwrap()
                .join("current.json")
                .exists()
        );
    }
    #[test]
    fn server_import_symlinked_receipt_binary_lock_or_ancestor_refuses() {
        for part in ["receipt", "binary", "extension_builder", "lock", "ancestor"] {
            let fixture = fixture();
            let path = match part {
                "receipt" => fixture.release.clone(),
                "binary" => fixture.release.parent().unwrap().join("surreal"),
                "extension_builder" => fixture.release.parent().unwrap().join("surreal-ext"),
                "lock" => fixture.release.parent().unwrap().join("Cargo.lock"),
                _ => fixture.release.parent().unwrap().to_path_buf(),
            };
            let moved = path.with_extension("moved");
            fs::rename(&path, &moved).unwrap();
            std::os::unix::fs::symlink(&moved, &path).unwrap();
            assert!(
                import_inner(&fixture.root, &fixture.environment, &fixture.release).is_err(),
                "{part}"
            );
        }
    }
}
