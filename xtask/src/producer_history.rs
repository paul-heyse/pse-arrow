// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Private historical evidence, deliberately separate from producer identity and
//! current deployment admission. Only selected consumed files are copied; search
//! namespaces retain complete topology/digest inventories, not all search bytes.

use super::*;
use std::io::{Read, Write};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Member {
    original: deployment_identity::FileObservation,
    original_file_identity: Option<(u64, u64)>,
    protected: PathBuf,
    extent: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Namespace {
    identity: String,
    entries: BTreeMap<String, serde_json::Value>,
    aliases: Vec<BTreeSet<PathBuf>>,
    raw_coverage: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Index {
    version: u32,
    purpose: String,
    workspace_root: PathBuf,
    context: serde_json::Value,
    graph: UnitGraph,
    receipt: ProducerIdentity,
    declarations: InputDeclarations,
    pest_grammars: BTreeMap<String, Vec<PestGrammarInput>>,
    associations: BTreeMap<usize, Vec<PathBuf>>,
    artifact_aliases: BTreeMap<PathBuf, Vec<PathBuf>>,
    members: BTreeMap<PathBuf, Member>,
    namespaces: BTreeMap<PathBuf, Namespace>,
    absences: BTreeMap<PathBuf, bool>,
    streams: BTreeMap<String, String>,
}

fn bytes_digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

fn file_digest(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut input = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65_536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 { break; }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn retain(directory: &Path, path: &Path, members: &mut BTreeMap<PathBuf, Member>) -> Result<()> {
    if members.contains_key(path) { return Ok(()); }
    let original = deployment_identity::FileObservation::capture(path)?;
    let mut input = fs::File::open(path)?;
    let metadata = input.metadata()?;
    #[cfg(unix)]
    let original_file_identity = {
        use std::os::unix::fs::MetadataExt;
        Some((metadata.dev(), metadata.ino()))
    };
    #[cfg(not(unix))]
    let original_file_identity = None;
    let protected = PathBuf::from("members").join(&original.sha256);
    let destination = directory.join(&protected);
    if !destination.try_exists()? {
        let temporary = tempfile::NamedTempFile::new_in(directory.join("members"))?;
        let mut output = temporary.as_file();
        std::io::copy(&mut input, &mut output)?;
        output.sync_all()?;
        ensure!(file_digest(temporary.path())? == original.sha256,
            "consumed evidence changed while protecting {}", path.display());
        temporary.persist_noclobber(&destination).map_err(|error| error.error)?;
    }
    ensure!(deployment_identity::FileObservation::capture(path)? == original,
        "consumed evidence changed before protection completed: {}", path.display());
    members.insert(path.to_path_buf(), Member { original, original_file_identity, protected, extent: metadata.len() });
    Ok(())
}

pub(super) fn prepare(
    directory: &Path,
    options: &ProducerOptions,
    root: &Path,
    graph: &UnitGraph,
    packages: &BTreeMap<String, cargo_metadata::Package>,
    messages: &[serde_json::Value],
    receipt: &ProducerIdentity,
) -> Result<()> {
    fs::create_dir(directory.join("members"))?;
    let mut members = BTreeMap::new();
    let mut associations = BTreeMap::new();
    let mut artifact_aliases = BTreeMap::new();
    for index in selected_indices(graph)? {
        let unit = &graph.units[index];
        if let Some(paths) = artifact_dep_info(unit, graph, messages, root)? {
            for path in &paths {
                retain(directory, path, &mut members)?;
                for input in parse_dep_info(&fs::read_to_string(path)?)? {
                    retain(directory, &absolute(root, &input), &mut members)?;
                }
            }
            associations.insert(index, paths);
        }
        // Source-bound executor/helper review includes complete selected package
        // source. Retain that selected source, never the entire Cargo cache.
        let package = packages.get(&unit.pkg_id).context("historical unit package")?;
        let package_root = package.manifest_path.as_std_path().parent().context("package root")?;
        for source in package_files(package_root)? {
            retain(directory, &source, &mut members)?;
        }
        for message in messages.iter().filter(|message| artifact_matches(unit, message)) {
            for file in message["filenames"].as_array().into_iter().flatten().filter_map(serde_json::Value::as_str) {
                let artifact = absolute(root, Path::new(file));
                if artifact.is_file() && (unit.target.kind == ["bin"] || unit.target.crate_types.iter().any(|kind| kind == "cdylib")) {
                    let aliases = executable_artifact_aliases(&artifact)?;
                    retain(directory, &artifact, &mut members)?;
                    for alias in &aliases { retain(directory, alias, &mut members)?; }
                    artifact_aliases.insert(artifact, aliases);
                }
            }
        }
    }
    for input in receipt.consumed_inputs.values().flatten() {
        if let Some(observation) = &input.observation {
            retain(directory, &observation.path, &mut members)?;
            ensure!(members[&observation.path].original == *observation,
                "protected consumed observation differs from receipt");
        }
    }
    for path in [root.join("Cargo.toml"), root.join("Cargo.lock")]
        .into_iter().chain(cargo_configuration_files(root))
        .chain(options.declarations.review_catalogs.iter().map(|path| absolute(root, path)))
    {
        retain(directory, &path, &mut members)?;
    }
    if let Some(association) = &receipt.deployment {
        retain(directory, &association.built_artifact.path, &mut members)?;
        ensure!(members[&association.built_artifact.path].original == association.built_artifact,
            "protected artifact differs from actual role association");
    }
    if let Some(provider) = &receipt.native_provider {
        for observation in &provider.files { retain(directory, &observation.path, &mut members)?; }
    }
    let mut namespace_paths: BTreeSet<_> = options.declarations.native_namespaces.iter().map(|path| absolute(root, path)).collect();
    let mut absences = BTreeMap::new();
    for declaration in options.declarations.build_scripts.values().chain(options.declarations.proc_macros.values()) {
        namespace_paths.extend(declaration.namespaces.iter().map(|path| absolute(root, path)));
        for path in &declaration.absent_files {
            let path = absolute(root, path);
            absences.insert(path.clone(), path.try_exists()?);
        }
        for grammar in &declaration.pest_grammars {
            for path in [&grammar.preferred, &grammar.fallback] {
                let present = path.try_exists()?;
                absences.insert(path.clone(), present);
                if present { retain(directory, path, &mut members)?; }
            }
        }
        for path in &declaration.files { retain(directory, &absolute(root, path), &mut members)?; }
    }
    for path in options.declarations.native_absent_files.iter().chain(options.declarations.configuration_executors.iter().flat_map(|executor| executor.absent_files.iter())) {
        let path = absolute(root, path);
        absences.insert(path.clone(), path.try_exists()?);
    }
    let namespaces = namespace_paths.into_iter().map(|path| {
        let observation = namespace_observation(&path, 262_144)?;
        Ok((path, Namespace {
            identity: observation.identity, entries: observation.entries,
            aliases: observation.aliases.into_values().collect(),
            raw_coverage: "inventory-only; selected consumed raw files separately retained".into(),
        }))
    }).collect::<Result<_>>()?;
    let mut streams = BTreeMap::new();
    for name in ["build.stdout", "build.stderr", "build.status.json", "refresh.stdout", "refresh.stderr", "refresh.status.json"] {
        let path = directory.join(name);
        if path.try_exists()? { streams.insert(name.into(), file_digest(&path)?); }
    }
    let index = Index {
        version: 1, purpose: "historical reconstruction only; never current deployment admission".into(),
        workspace_root: root.to_path_buf(),
        context: serde_json::json!({"package": options.package, "profile": options.profile,
            "target": options.target, "production_target": options.production_target,
            "features": options.features, "no_default_features": options.no_default_features,
            "caller_environment": native_caller_environment(options)?}),
        graph: graph.clone(), receipt: receipt.clone(), declarations: options.declarations.clone(),
        pest_grammars: options.declarations.proc_macros.iter().map(|(key, value)| (key.clone(), value.pest_grammars.clone())).collect(),
        associations, artifact_aliases, members, namespaces, absences, streams,
    };
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(directory.join("index.json"))?;
    serde_json::to_writer(&mut file, &index)?;
    file.sync_all()?;
    Ok(())
}

pub(super) fn seal(directory: &Path) -> Result<()> {
    // Seal only after every original premise still agrees and Cargo actually
    // succeeded. A partial directory never becomes a historical product.
    let index = validate(directory)?;
    for member in index.members.values() { member.original.verify()?; }
    for (path, namespace) in &index.namespaces {
        ensure!(namespace_identity(path)? == namespace.identity, "namespace changed before historical seal");
    }
    for (path, present) in &index.absences {
        ensure!(path.try_exists()? == *present, "absence premise changed before historical seal");
    }
    let digest = file_digest(&directory.join("index.json"))?;
    let mut seal = fs::OpenOptions::new().write(true).create_new(true).open(directory.join("COMPLETE"))?;
    seal.write_all(digest.as_bytes())?;
    seal.sync_all()?;
    Ok(())
}

fn protected_bytes(directory: &Path, member: &Member) -> Result<Vec<u8>> {
    ensure!(member.protected == PathBuf::from("members").join(&member.original.sha256), "invalid protected member path");
    let bytes = fs::read(directory.join(&member.protected))?;
    ensure!(bytes.len() as u64 == member.extent && bytes_digest(&bytes) == member.original.sha256, "historical raw member missing or changed");
    Ok(bytes)
}

fn validate(directory: &Path) -> Result<Index> {
    let index: Index = serde_json::from_slice(&fs::read(directory.join("index.json"))?)?;
    ensure!(index.version == 1, "unsupported historical evidence version");
    for (path, member) in &index.members {
        ensure!(path == &member.original.path, "historical member original association differs");
        // Stream hashing avoids allocating native libraries merely to verify them.
        ensure!(member.protected == PathBuf::from("members").join(&member.original.sha256)
            && fs::metadata(directory.join(&member.protected))?.len() == member.extent
            && file_digest(&directory.join(&member.protected))? == member.original.sha256,
            "historical member missing or changed: {}", path.display());
    }
    for (name, expected) in &index.streams {
        ensure!(["build.stdout", "build.stderr", "build.status.json", "refresh.stdout", "refresh.stderr", "refresh.status.json"].contains(&name.as_str()), "unknown historical stream");
        ensure!(file_digest(&directory.join(name))? == *expected, "historical child stream changed");
    }
    let status: serde_json::Value = serde_json::from_slice(&fs::read(directory.join("build.status.json"))?)?;
    ensure!(status["success"] == true && status["stream_errors"] == serde_json::json!([]), "historical child did not complete successfully");
    let messages = parse_cargo_messages(&fs::read(directory.join("build.stdout"))?)?;
    let root = &index.workspace_root;
    let keys = unit_keys(&index.graph, root)?;
    let selected = selected_indices(&index.graph)?;
    for (number, paths) in &index.associations {
        ensure!(selected.contains(number), "historical association names an unselected unit");
        let unit = &index.graph.units[*number];
        for path in paths {
            let member = index.members.get(path).context("historical dep-info missing")?;
            let text = String::from_utf8(protected_bytes(directory, member)?)?;
            let outputs = dep_info_outputs(&text)?.into_iter().map(|path| absolute(root, &path)).collect::<BTreeSet<_>>();
            let inputs = parse_dep_info(&text)?.into_iter().map(|path| absolute(root, &path)).collect::<BTreeSet<_>>();
            ensure!(inputs.contains(&unit.target.src_path), "historical dep-info source differs from selected unit");
            ensure!(inputs.iter().all(|input| index.members.contains_key(input)), "historical generated/consumed input missing");
            let matches = messages.iter().filter(|message| artifact_matches(unit, message)).any(|message| {
                message["filenames"].as_array().into_iter().flatten().filter_map(serde_json::Value::as_str).any(|file| {
                    let artifact = absolute(root, Path::new(file));
                    let mut artifacts = vec![artifact.clone()];
                    for alias in index.artifact_aliases.get(&artifact).into_iter().flatten() {
                        let same = index.members.get(&artifact).zip(index.members.get(alias)).is_some_and(|(a,b)| a.original_file_identity.is_some() && a.original_file_identity == b.original_file_identity);
                        if same { artifacts.push(alias.clone()); }
                    }
                    artifacts.iter().any(|artifact| {
                        let mut candidates = vec![artifact.with_extension("d")];
                        if let Some(name) = artifact.file_name().and_then(|name| name.to_str()).and_then(|name| name.strip_prefix("lib")) {
                            candidates.push(artifact.with_file_name(name).with_extension("d"));
                        }
                        candidates.contains(path) && ((outputs.contains(path) && outputs.contains(artifact))
                            || cargo_cached_dep_info_linkage(unit, root, path, artifact, &outputs))
                    })
                })
            });
            ensure!(matches, "historical dep-info lacks actual Cargo unit/artifact linkage: {}", keys[number]);
        }
    }
    for namespace in index.namespaces.values() {
        let mut entries = BTreeMap::new();
        for (label, entry) in &namespace.entries {
            let digest = if entry["absent"] == true { hash_parts(&[b"namespace-absence"]) } else {
                let mode: Vec<u8> = serde_json::from_value(entry["mode_frame"].clone())?;
                let target_mode: Vec<u8> = serde_json::from_value(entry["target_mode_frame"].clone())?;
                let content: Vec<u8> = serde_json::from_value(entry["content_frame"].clone())?;
                let link: Option<PathBuf> = serde_json::from_value(entry["link"].clone())?;
                hash_parts(&[b"native-namespace-entry.v1", entry["kind"].as_str().context("namespace kind")?.as_bytes(), &mode, &target_mode, &serde_json::to_vec(&link)?, &content])
            };
            entries.insert(label.clone(), digest);
        }
        let mut aliases: Vec<_> = namespace.aliases.iter().filter(|paths| paths.len() > 1).collect();
        aliases.sort();
        let alias_identity = hash_parts(&[b"native-file-alias-equivalence.v1", &serde_json::to_vec(&aliases)?]);
        ensure!(hash_parts(&[b"native-namespace.v1", &serde_json::to_vec(&entries)?, alias_identity.as_bytes()]) == namespace.identity, "historical namespace framing differs");
    }
    let actual_associated = index.associations.keys().copied().collect::<BTreeSet<_>>();
    for number in &selected {
        let unit = &index.graph.units[*number];
        if !unit.target.kind.iter().any(|kind| kind == "proc-macro") { continue; }
        let package_id = logical_package(&unit.pkg_id, root);
        let review_key = format!("macro-caller-inputs:{package_id}:{}", keys[number]);
        if let Some(expected) = index.receipt.reviewed_owner_sources.get(&review_key) {
            let mut declaration = index.declarations.proc_macros.get(&unit.pkg_id)
                .or_else(|| index.declarations.proc_macros.get(&package_id)).cloned();
            if let Some(value) = declaration.as_mut() {
                value.pest_grammars = index.pest_grammars.get(&unit.pkg_id)
                    .or_else(|| index.pest_grammars.get(&package_id)).cloned().unwrap_or_default();
            }
            let basis = macro_caller_input_basis_with_reader(&index.graph, *number, &keys,
                &index.associations, &actual_associated, root, declaration.as_ref(), |path| {
                    if let Some(member) = index.members.get(path) { Ok(Some(protected_bytes(directory, member)?)) }
                    else if index.absences.get(path) == Some(&false) { Ok(None) }
                    else { anyhow::bail!("historical caller premise missing: {}", path.display()) }
                })?;
            ensure!(basis.as_ref() == Some(expected), "historical macro caller basis differs");
        }
    }
    let scientific = serde_json::to_value(&index.receipt)?;
    deployment_identity::verify_scientific_producer_identity(&scientific)?;
    if let Some(association) = &index.receipt.deployment {
        ensure!(index.members.get(&association.built_artifact.path).is_some_and(|member| member.original == association.built_artifact), "historical role artifact association missing");
        for file in &association.files {
            ensure!(index.members.get(&file.path).is_some_and(|member| member.original == *file), "historical consumed original observation missing");
        }
    }
    Ok(index)
}

pub(crate) fn verify_historical_evidence(directory: &Path) -> Result<serde_json::Value> {
    let expected = fs::read_to_string(directory.join("COMPLETE")).context("historical evidence is incomplete")?;
    ensure!(file_digest(&directory.join("index.json"))? == expected, "historical index seal differs");
    let index = validate(directory)?;
    Ok(serde_json::json!({"version": index.version, "historical_only": true,
        "producer_identity": index.receipt.identity, "selected_root": index.receipt.selected_root,
        "unit_associations": index.associations.len(), "protected_members": index.members.len(),
        "namespace_raw_coverage": "inventory-only except separately retained consumed members",
        "reviewed_owner_sources": index.receipt.reviewed_owner_sources}))
}
