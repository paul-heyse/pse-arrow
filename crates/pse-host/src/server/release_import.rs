// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit snapshot admission of an established source release, without rebuilding it.
use super::*;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRelease {
    schema: String,
    version: String,
    upstream_revision: String,
    surrealmx_version: String,
    surrealism_sdk_version: String,
    effective_key: String,
    binary: PathBuf,
    binary_sha256: String,
    lockfile_sha256: String,
    inputs: Value,
    raw_provenance: Value,
}
fn source_inputs(source: &SourceIdentity) -> Value {
    serde_json::json!({
        "version":source.version, "upstream_revision":source.revision, "upstream_origin":source.origin,
        "surrealmx_version":source.engine, "surrealism_sdk_version":source.surrealism,
        "features":source.features, "profile":source.profile, "build_config":[source.build_config],
        "patches":source.patches, "build_metadata":source.metadata,
        "diskann_archive_sha256":source.diskann_archive, "diskann_patch_sha256":source.diskann_patch,
        "platform":format!("Linux-{}",std::env::consts::ARCH),
    })
}
fn nonempty(value: &Value) -> bool {
    match value {
        Value::Object(value) => !value.is_empty(), Value::Array(value) => !value.is_empty(),
        Value::String(value) => !value.is_empty(), _ => false,
    }
}
/// Match the established receipt's canonical sorted ASCII JSON hash without Python.
fn canonical(value: &Value) -> Result<String, HostError> {
    Ok(match value {
        Value::Object(values) => {
            let mut pairs=values.iter().collect::<Vec<_>>();pairs.sort_by_key(|(key,_)|*key);
            let pairs=pairs.into_iter().map(|(key,value)|Ok(format!("{}:{}",canonical(&Value::String(key.clone()))?,canonical(value)?))).collect::<Result<Vec<_>,HostError>>()?;
            format!("{{{}}}",pairs.join(","))
        }
        Value::Array(values) => format!("[{}]",values.iter().map(canonical).collect::<Result<Vec<_>,_>>()?.join(",")),
        Value::String(value) => {
            let encoded=serde_json::to_string(value).map_err(|_|invalid("cannot encode release context"))?;
            let mut result=String::new();
            for character in encoded.chars() {
                if character.is_ascii() {result.push(character);}
                else {for unit in character.encode_utf16(&mut [0;2]).iter() {result.push_str(&format!("\\u{unit:04x}"));}}
            }
            result
        }
        Value::Number(value) if value.is_i64() || value.is_u64() => value.to_string(),
        Value::Number(_) => return Err(invalid("floating-point build context identities require a separately established canonical encoding")),
        Value::Bool(value) => value.to_string(), Value::Null => "null".into(),
    })
}
fn decode_release(bytes:&[u8], source:&SourceIdentity) -> Result<SourceRelease,HostError> {
    let release:SourceRelease=serde_json::from_slice(bytes).map_err(|error|invalid(format!("invalid source release: {error}")))?;
    let inputs=release.inputs.as_object().ok_or_else(||invalid("source release inputs are incomplete"))?;
    let context=inputs.get("build_context").and_then(Value::as_object).ok_or_else(||invalid("source release lacks its original build context"))?;
    let mut package=inputs.clone();package.remove("build_context");
    let namespace=package.remove("installation_namespace").and_then(|value|value.as_str().map(PathBuf::from));
    if release.schema!="surreal-source-release-v2" || release.version!=source.version
        || release.upstream_revision!=source.revision || release.surrealmx_version!=source.engine
        || release.surrealism_sdk_version!=source.surrealism || !source.supported()
        || Value::Object(package)!=source_inputs(source)
        || namespace.is_none_or(|path|!path.is_absolute())
        || context.get("schema").and_then(Value::as_str)!=Some("server-build-context-v1")
        || ["configuration","toolchain_declaration","tools","toolchain","native","owners"].iter().any(|key|!context.get(*key).is_some_and(nonempty))
        || !release.raw_provenance.as_object().is_some_and(|value|!value.is_empty())
        || !hexadecimal(&release.binary_sha256,64) || !hexadecimal(&release.lockfile_sha256,64)
        || !hexadecimal(&release.effective_key,64) || hash(canonical(&release.inputs)?.as_bytes())!=release.effective_key {
        return Err(invalid("source release does not retain the complete selected source/package identity and original provenance"));
    }
    Ok(release)
}
fn open_regular(path:&Path)->Result<File,HostError> {
    // Bind the file description without following any symlink component. Hash the
    // bytes read from that same description during the copy, never a reopened path.
    let descriptor=rustix::fs::openat2(rustix::fs::CWD,path,
        rustix::fs::OFlags::RDONLY|rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),rustix::fs::ResolveFlags::NO_SYMLINKS)
        .map_err(std::io::Error::from)?;
    let file=File::from(descriptor);
    if !file.metadata()?.is_file() {return Err(invalid("source release artifacts must be regular files"));}
    Ok(file)
}
struct Candidate {
    root: PathBuf,
    identity: Identity,
    receipt: Vec<u8>,
    binary: File,
    lock: File,
    binary_sha256: String,
    lock_sha256: String,
}
fn admit(root:&Path, path:&Path)->Result<Candidate,HostError> {
    let path=if path.is_absolute() {path.to_path_buf()} else {std::env::current_dir()?.join(path)};
    let mut receipt=Vec::new();open_regular(&path)?.read_to_end(&mut receipt)?;
    let source=source_identity(root)?;
    let release=decode_release(&receipt,&source)?;
    let directory=path.parent().ok_or_else(||invalid("source release has no installation directory"))?;
    let binary=directory.join("source/target/release/surreal");
    if release.binary!=binary {return Err(invalid("source release binary path differs from its installation"));}
    let binary=open_regular(&binary)?;
    if binary.metadata()?.permissions().mode()&0o111==0 {return Err(invalid("source release binary is not executable"));}
    let lock=open_regular(&directory.join("source/Cargo.lock"))?;
    Ok(Candidate {
        root: root.to_path_buf(),
        identity:Identity {source,origin:ConstructionOrigin::ImportedRelease {release_sha256:hash(&receipt)}},
        receipt,binary,lock,binary_sha256:release.binary_sha256,lock_sha256:release.lockfile_sha256,
    })
}
fn copy_verified(input:&mut impl Read,path:&Path,expected:&str)->Result<(),HostError> {
    let mut output=OpenOptions::new().create_new(true).write(true).mode(0o600).open(path)?;
    let mut hasher=Sha256::new();let mut buffer=[0_u8;65536];
    loop {let size=input.read(&mut buffer)?;if size==0 {break;}output.write_all(&buffer[..size])?;hasher.update(&buffer[..size]);}
    let actual:String=hasher.finalize().iter().map(|byte|format!("{byte:02x}")).collect();
    if actual!=expected {return Err(invalid("copied source release bytes differ from the original receipt"));}
    output.sync_all()?;Ok(())
}
fn lock_pins(path:&Path)->Result<(),HostError> {
    let lock:toml::Value=toml::from_str(&fs::read_to_string(path)?).map_err(|error|invalid(format!("invalid imported lockfile: {error}")))?;
    let packages=lock.get("package").and_then(toml::Value::as_array).ok_or_else(||invalid("imported lockfile has no packages"))?;
    for (name,version) in [("surrealdb",VERSION),("surrealdb-core",VERSION),("surrealmx",ENGINE),("surrealism-runtime",SURREALISM)] {
        let found=packages.iter().filter(|package|package.get("name").and_then(toml::Value::as_str)==Some(name)).collect::<Vec<_>>();
        if found.len()!=1 || found[0].get("version").and_then(toml::Value::as_str)!=Some(version) {
            return Err(invalid(format!("imported lockfile differs from pinned {name} {version}")));
        }
    } Ok(())
}
fn import_candidate(environment:&Environment,mut candidate:Candidate)->Result<PreparedServer,HostError> {
    let profile=profile_root(environment)?;private_directory(&profile)?;
    let recipe=hash(&encode(&candidate.identity)?);
    let locks=profile.join("locks");private_directory(&locks)?;let lock_path=locks.join(&recipe);
    if lock_path.exists() {regular(&lock_path)?;}
    let lock=OpenOptions::new().create(true).truncate(false).read(true).write(true).mode(0o600).open(lock_path)?;lock.lock()?;
    let staging=profile.join("staging");private_directory(&staging)?;
    let scratch=tempfile::Builder::new().prefix("import-").tempdir_in(staging)?;
    let install=scratch.path().join("install");fs::create_dir(&install)?;
    copy_verified(&mut candidate.binary,&install.join("surreal"),&candidate.binary_sha256)?;
    copy_verified(&mut candidate.lock,&install.join("Cargo.lock"),&candidate.lock_sha256)?;
    lock_pins(&install.join("Cargo.lock"))?;
    fs::write(install.join("release.json"),&candidate.receipt)?;File::open(install.join("release.json"))?.sync_all()?;
    if source_identity(&candidate.root)?!=candidate.identity.source {
        return Err(invalid("selected source/package premises changed during release import; no selection published"));
    }
    // Explicit repeat admission can reuse an immutable generation, but only after
    // verifying the presently supplied bytes against the presently supplied receipt.
    if let Ok(current)=select(environment) {if current.recipe==recipe {return Ok(current);}}
    publish_install(&profile,&install,&candidate.identity)
}
pub(super) fn import_inner(root:&Path,environment:&Environment,path:&Path)->Result<PreparedServer,HostError> {
    import_candidate(environment,admit(root,path)?)
}
pub(super) fn origin_complete(generation:&Path,identity:&Identity,files:&BTreeMap<String,String>)->Result<bool,HostError> {
    match &identity.origin {
        ConstructionOrigin::Built {context} => Ok(context.complete()),
        ConstructionOrigin::ImportedRelease {release_sha256} => {
            if !hexadecimal(release_sha256,64) {return Ok(false);}
            let bytes=fs::read(generation.join("release.json"))?;
            if hash(&bytes)!=*release_sha256 {return Ok(false);}
            let release=decode_release(&bytes,&identity.source)?;
            lock_pins(&generation.join("Cargo.lock"))?;
            Ok(files.get("surreal")==Some(&release.binary_sha256)
                && files.get("Cargo.lock")==Some(&release.lockfile_sha256))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture { directory:tempfile::TempDir,root:PathBuf,release:PathBuf,environment:Environment }
    fn writable(path:&Path) {
        let metadata=fs::symlink_metadata(path).unwrap();
        if metadata.is_dir() {fs::set_permissions(path,fs::Permissions::from_mode(0o700)).unwrap();for entry in fs::read_dir(path).unwrap() {writable(&entry.unwrap().path());}}
    }
    impl Drop for Fixture {fn drop(&mut self) {writable(self.directory.path());}}
    fn fixture()->Fixture {
        let directory=tempfile::tempdir().unwrap();let root=directory.path().join("checkout");let patches=root.join("scripts/surrealdb-source");fs::create_dir_all(&patches).unwrap();
        for name in PATCHES {fs::write(patches.join(name),format!("original patch {name}")).unwrap();}
        fs::write(patches.join("diskann-0.56.0.patch"),b"DiskANN patch").unwrap();
        let source=source_identity(&root).unwrap();
        let original=directory.path().join("original");let binary=original.join("source/target/release/surreal");fs::create_dir_all(binary.parent().unwrap()).unwrap();
        fs::write(&binary,b"#!/bin/sh\nprintf 'original server\\n'\n").unwrap();fs::set_permissions(&binary,fs::Permissions::from_mode(0o755)).unwrap();
        let lock=original.join("source/Cargo.lock");
        fs::write(&lock,format!("version = 4\n[[package]]\nname='surrealdb'\nversion='{VERSION}'\n[[package]]\nname='surrealdb-core'\nversion='{VERSION}'\n[[package]]\nname='surrealmx'\nversion='{ENGINE}'\n[[package]]\nname='surrealism-runtime'\nversion='{SURREALISM}'\n")).unwrap();
        let mut inputs=source_inputs(&source);
        inputs["installation_namespace"]=Value::String(original.to_string_lossy().into_owned());
        inputs["build_context"]=serde_json::json!({"schema":"server-build-context-v1","cwd":"/original-checkout","configuration":[{"original-config":"retained"}],"toolchain_declaration":["original-toolchain"],"tools":{"original-compiler":"retained"},"toolchain":{"rustc":"original rustc"},"native":["original ABI"],"owners":{"original-controller":"recorded"},"environment":{"ORIGINAL":"retained"}});
        let receipt=serde_json::json!({"schema":"surreal-source-release-v2","version":VERSION,"upstream_revision":REVISION,"surrealmx_version":ENGINE,"surrealism_sdk_version":SURREALISM,
            "effective_key":hash(canonical(&inputs).unwrap().as_bytes()),"binary":binary,"binary_sha256":digest(&binary).unwrap(),"lockfile_sha256":digest(&lock).unwrap(),"inputs":inputs,
            "raw_provenance":{"cargo_configuration":[{"original":"raw input"}],"toolchain_sha256":"original toolchain","unrecognized_original_field":[1,true,"é"]}});
        let release=original.join("release.json");fs::write(&release,format!("{}\n",serde_json::to_string_pretty(&receipt).unwrap())).unwrap();
        let mut environment=Environment::new();environment.insert("PSE_SURREAL_TOOLS".into(),directory.path().join("native-tools").into_os_string());
        Fixture {directory,root,release,environment}
    }
    fn rewrite(fixture:&Fixture,change:impl FnOnce(&mut Value)) {
        let mut receipt:Value=serde_json::from_slice(&fs::read(&fixture.release).unwrap()).unwrap();change(&mut receipt);
        receipt["effective_key"]=Value::String(hash(canonical(&receipt["inputs"]).unwrap().as_bytes()));fs::write(&fixture.release,encode(&receipt).unwrap()).unwrap();
    }
    #[test]
    fn server_import_preserves_verbatim_original_context_and_selects_after_original_removed() {
        let fixture=fixture();let original=fs::read(&fixture.release).unwrap();
        let prepared=import_inner(&fixture.root,&fixture.environment,&fixture.release).unwrap();
        assert_eq!(fs::read(prepared.generation.join("release.json")).unwrap(),original);
        let receipt:Receipt=read_record(&prepared.generation.join(".complete.json")).unwrap();
        assert!(matches!(receipt.identity.origin,ConstructionOrigin::ImportedRelease{..}));
        let published=fs::read_to_string(prepared.generation.join(".complete.json")).unwrap();assert!(!published.contains("current-compiler"));
        fs::remove_dir_all(fixture.release.parent().unwrap()).unwrap();fs::remove_dir_all(&fixture.root).unwrap();
        assert_eq!(select(&fixture.environment).unwrap().generation(),prepared.generation());
        assert_eq!(fs::read(prepared.executable()).unwrap(),b"#!/bin/sh\nprintf 'original server\\n'\n");
    }
    #[test]
    fn server_import_wrong_patch_or_package_identity_refuses_even_with_recomputed_context_key() {
        for key in ["patch","diskann","features","config","revision","engine","surrealism","context"] {
            let fixture=fixture();rewrite(&fixture,|value|match key {
                "patch"=>value["inputs"]["patches"][PATCHES[0]]=Value::String("f".repeat(64)),
                "diskann"=>value["inputs"]["diskann_patch_sha256"]=Value::String("f".repeat(64)),
                "features"=>value["inputs"]["features"]=Value::String("other".into()),
                "config"=>value["inputs"]["build_config"]=serde_json::json!(["profile.release.panic=abort"]),
                "revision"=>value["inputs"]["upstream_revision"]=Value::String("f".repeat(40)),
                "engine"=>value["inputs"]["surrealmx_version"]=Value::String("other".into()),
                "surrealism"=>value["inputs"]["surrealism_sdk_version"]=Value::String("other".into()),
                _=>value["inputs"]["build_context"]["native"]=serde_json::json!([]),
            });
            assert!(import_inner(&fixture.root,&fixture.environment,&fixture.release).is_err(),"{key}");
            assert!(!profile_root(&fixture.environment).unwrap().join("current.json").exists());
        }
    }
    #[test]
    fn server_import_binary_or_lock_hash_failure_preserves_prior_selection() {
        for key in ["binary_sha256","lockfile_sha256"] {
            let fixture=fixture();let prepared=import_inner(&fixture.root,&fixture.environment,&fixture.release).unwrap();let before=fs::read(profile_root(&fixture.environment).unwrap().join("current.json")).unwrap();
            rewrite(&fixture,|value|value[key]=Value::String("0".repeat(64)));
            assert!(import_inner(&fixture.root,&fixture.environment,&fixture.release).is_err());
            assert_eq!(fs::read(profile_root(&fixture.environment).unwrap().join("current.json")).unwrap(),before);
            assert_eq!(select(&fixture.environment).unwrap().generation(),prepared.generation());
        }
    }
    #[test]
    fn server_import_matching_lock_hash_with_wrong_engine_resolution_refuses() {
        let fixture=fixture();let lock=fixture.release.parent().unwrap().join("source/Cargo.lock");
        let contents=fs::read_to_string(&lock).unwrap().replace("version='1.0.0'","version='0.9.0'");fs::write(&lock,contents).unwrap();
        rewrite(&fixture,|value|value["lockfile_sha256"]=Value::String(digest(&lock).unwrap()));
        assert!(import_inner(&fixture.root,&fixture.environment,&fixture.release).is_err());
        assert!(!profile_root(&fixture.environment).unwrap().join("current.json").exists());
    }
    #[test]
    fn server_import_hashes_the_copied_open_description_after_source_mutation() {
        let fixture=fixture();let prepared=import_inner(&fixture.root,&fixture.environment,&fixture.release).unwrap();
        let candidate=admit(&fixture.root,&fixture.release).unwrap();let binary=fixture.release.parent().unwrap().join("source/target/release/surreal");
        fs::write(binary,b"mutated after admission before snapshot").unwrap();
        assert!(import_candidate(&fixture.environment,candidate).is_err());
        assert_eq!(select(&fixture.environment).unwrap().generation(),prepared.generation());
    }
    #[test]
    fn server_import_open_description_cannot_be_redirected_by_source_path_replacement() {
        let fixture=fixture();let candidate=admit(&fixture.root,&fixture.release).unwrap();let original_sha=candidate.binary_sha256.clone();
        let binary=fixture.release.parent().unwrap().join("source/target/release/surreal");fs::rename(&binary,binary.with_extension("previous")).unwrap();fs::write(&binary,b"replacement pathname").unwrap();
        let prepared=import_candidate(&fixture.environment,candidate).unwrap();assert_eq!(digest(&prepared.executable()).unwrap(),original_sha);
    }
    #[test]
    fn server_import_changed_selected_patches_during_copy_refuses_publication() {
        let fixture=fixture();let candidate=admit(&fixture.root,&fixture.release).unwrap();
        fs::write(fixture.root.join("scripts/surrealdb-source").join(PATCHES[0]),b"changed source premise").unwrap();
        assert!(import_candidate(&fixture.environment,candidate).is_err());
        assert!(!profile_root(&fixture.environment).unwrap().join("current.json").exists());
    }
    #[test]
    fn server_import_symlinked_receipt_binary_lock_or_ancestor_refuses() {
        for part in ["receipt","binary","lock","ancestor"] {
            let fixture=fixture();let path=match part {"receipt"=>fixture.release.clone(),"binary"=>fixture.release.parent().unwrap().join("source/target/release/surreal"),"lock"=>fixture.release.parent().unwrap().join("source/Cargo.lock"),_=>fixture.release.parent().unwrap().join("source")};
            let moved=path.with_extension("moved");fs::rename(&path,&moved).unwrap();std::os::unix::fs::symlink(&moved,&path).unwrap();
            assert!(import_inner(&fixture.root,&fixture.environment,&fixture.release).is_err(),"{part}");
        }
    }
    #[test]
    fn server_import_context_hash_uses_original_sorted_ascii_json_spelling() {
        let value=serde_json::json!({"é":"🦀","z":[true,32,null],"a":"line\n"});
        assert_eq!(canonical(&value).unwrap(),"{\"a\":\"line\\n\",\"z\":[true,32,null],\"\\u00e9\":\"\\ud83e\\udd80\"}");
    }
}
