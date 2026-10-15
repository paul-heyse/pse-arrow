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
    original_link: Option<PathBuf>,
    protected: PathBuf,
    extent: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Namespace {
    identity: String,
    entries: BTreeMap<String, serde_json::Value>,
    aliases: Vec<((u64, u64), BTreeSet<PathBuf>)>,
    raw_coverage: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PackageSource {
    root: PathBuf,
    paths: Vec<PathBuf>,
    identity: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Presence {
    entry: bool,
    target: bool,
}

fn presence(path: &Path) -> Result<Presence> {
    Ok(Presence {
        entry: path_entry_exists(path)?,
        target: path.try_exists()?,
    })
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Index {
    version: u32,
    purpose: String,
    workspace_root: PathBuf,
    context: serde_json::Value,
    legacy_evidence: Option<PathBuf>,
    graph: UnitGraph,
    receipt: ProducerIdentity,
    declarations: InputDeclarations,
    pest_grammars: BTreeMap<String, Vec<PestGrammarInput>>,
    package_sources: BTreeMap<String, PackageSource>,
    associations: BTreeMap<usize, Vec<PathBuf>>,
    artifact_aliases: BTreeMap<PathBuf, Vec<PathBuf>>,
    members: BTreeMap<PathBuf, Member>,
    namespaces: BTreeMap<PathBuf, Namespace>,
    absences: BTreeMap<PathBuf, Presence>,
    streams: BTreeMap<String, String>,
}

fn bytes_digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn file_digest(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut input = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65_536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn retain(directory: &Path, path: &Path, members: &mut BTreeMap<PathBuf, Member>) -> Result<()> {
    if members.contains_key(path) {
        return Ok(());
    }
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
    let original_link = if fs::symlink_metadata(path)?.is_symlink() {
        Some(fs::read_link(path)?)
    } else {
        None
    };
    let protected = PathBuf::from("members").join(&original.sha256);
    let destination = directory.join(&protected);
    if !destination.try_exists()? {
        let temporary = tempfile::NamedTempFile::new_in(directory.join("members"))?;
        let mut output = temporary.as_file();
        std::io::copy(&mut input, &mut output)?;
        output.sync_all()?;
        ensure!(
            file_digest(temporary.path())? == original.sha256,
            "consumed evidence changed while protecting {}",
            path.display()
        );
        temporary
            .persist_noclobber(&destination)
            .map_err(|error| error.error)?;
    }
    ensure!(
        deployment_identity::FileObservation::capture(path)? == original,
        "consumed evidence changed before protection completed: {}",
        path.display()
    );
    members.insert(
        path.to_path_buf(),
        Member {
            original,
            original_file_identity,
            original_link,
            protected,
            extent: metadata.len(),
        },
    );
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
    let legacy_evidence = options
        .declarations
        .actual_build_evidence
        .as_ref()
        .map(|path| absolute(root, path));
    if let Some(path) = &legacy_evidence {
        retain(directory, path, &mut members)?;
    }
    let mut associations = BTreeMap::new();
    let mut artifact_aliases = BTreeMap::new();
    let mut package_sources = BTreeMap::new();
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
        let package = packages
            .get(&unit.pkg_id)
            .context("historical unit package")?;
        let package_root = package
            .manifest_path
            .as_std_path()
            .parent()
            .context("package root")?;
        if !package_sources.contains_key(&unit.pkg_id) {
            let paths = package_files(package_root)?;
            for source in &paths {
                retain(directory, source, &mut members)?;
            }
            package_sources.insert(
                unit.pkg_id.clone(),
                PackageSource {
                    root: package_root.to_path_buf(),
                    paths,
                    identity: reviewed_source_identity(package_root)?,
                },
            );
        }
        for message in messages
            .iter()
            .filter(|message| artifact_matches(unit, message))
        {
            for file in message["filenames"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
            {
                let artifact = absolute(root, Path::new(file));
                if artifact.is_file()
                    && (unit.target.kind == ["bin"]
                        || unit.target.crate_types.iter().any(|kind| kind == "cdylib"))
                {
                    let aliases = executable_artifact_aliases(&artifact)?;
                    retain(directory, &artifact, &mut members)?;
                    for alias in &aliases {
                        retain(directory, alias, &mut members)?;
                    }
                    artifact_aliases.insert(artifact, aliases);
                }
            }
        }
    }
    for input in receipt.consumed_inputs.values().flatten() {
        if let Some(observation) = &input.observation {
            retain(directory, &observation.path, &mut members)?;
            ensure!(
                members[&observation.path].original == *observation,
                "protected consumed observation differs from receipt"
            );
        }
    }
    for path in [root.join("Cargo.toml"), root.join("Cargo.lock")]
        .into_iter()
        .chain(cargo_configuration_files(root))
        .chain(
            options
                .declarations
                .review_catalogs
                .iter()
                .map(|path| absolute(root, path)),
        )
    {
        retain(directory, &path, &mut members)?;
    }
    if let Some(association) = &receipt.deployment {
        retain(directory, &association.built_artifact.path, &mut members)?;
        ensure!(
            members[&association.built_artifact.path].original == association.built_artifact,
            "protected artifact differs from actual role association"
        );
    }
    if let Some(provider) = &receipt.native_provider {
        for observation in &provider.files {
            retain(directory, &observation.path, &mut members)?;
        }
    }
    let mut namespace_paths: BTreeSet<_> = options
        .declarations
        .native_namespaces
        .iter()
        .map(|path| absolute(root, path))
        .collect();
    let mut absences = BTreeMap::new();
    for declaration in options
        .declarations
        .build_scripts
        .values()
        .chain(options.declarations.proc_macros.values())
    {
        namespace_paths.extend(
            declaration
                .namespaces
                .iter()
                .map(|path| absolute(root, path)),
        );
        for path in &declaration.absent_files {
            let path = absolute(root, path);
            absences.insert(path.clone(), presence(&path)?);
        }
        for grammar in &declaration.pest_grammars {
            for path in [&grammar.preferred, &grammar.fallback] {
                let present = path.try_exists()?;
                absences.insert(path.clone(), presence(path)?);
                if present {
                    retain(directory, path, &mut members)?;
                }
            }
        }
        for path in &declaration.files {
            retain(directory, &absolute(root, path), &mut members)?;
        }
    }
    for path in options.declarations.native_absent_files.iter().chain(
        options
            .declarations
            .configuration_executors
            .iter()
            .flat_map(|executor| executor.absent_files.iter()),
    ) {
        let path = absolute(root, path);
        absences.insert(path.clone(), presence(&path)?);
    }
    let namespaces = namespace_paths
        .into_iter()
        .map(|path| {
            let observation = namespace_observation(&path, 262_144)?;
            Ok((
                path,
                Namespace {
                    identity: observation.identity,
                    entries: observation.entries,
                    aliases: observation.aliases.into_iter().collect(),
                    raw_coverage: "inventory-only; selected consumed raw files separately retained"
                        .into(),
                },
            ))
        })
        .collect::<Result<_>>()?;
    let mut streams = BTreeMap::new();
    for name in [
        "build.stdout",
        "build.stderr",
        "build.status.json",
        "refresh.stdout",
        "refresh.stderr",
        "refresh.status.json",
        "unit-graph.stdout",
        "unit-graph.arguments.json",
        "metadata.stdout",
    ] {
        let path = directory.join(name);
        if path.try_exists()? {
            streams.insert(name.into(), file_digest(&path)?);
        }
    }
    let index = Index {
        version: 1,
        purpose: "historical reconstruction only; never current deployment admission".into(),
        workspace_root: root.to_path_buf(),
        context: serde_json::json!({"package": options.package, "profile": options.profile,
            "target": options.target, "production_target": options.production_target,
            "features": options.features, "no_default_features": options.no_default_features,
            "caller_environment": native_caller_environment(options)?}),
        legacy_evidence,
        graph: graph.clone(),
        receipt: receipt.clone(),
        declarations: options.declarations.clone(),
        pest_grammars: options
            .declarations
            .proc_macros
            .iter()
            .map(|(key, value)| (key.clone(), value.pest_grammars.clone()))
            .collect(),
        package_sources,
        associations,
        artifact_aliases,
        members,
        namespaces,
        absences,
        streams,
    };
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("index.json"))?;
    serde_json::to_writer(&mut file, &index)?;
    file.sync_all()?;
    Ok(())
}

pub(super) fn seal(directory: &Path) -> Result<()> {
    // Seal only after every original premise still agrees and Cargo actually
    // succeeded. A partial directory never becomes a historical product.
    let index = validate(directory)?;
    for member in index.members.values() {
        member.original.verify()?;
        let actual_link = if fs::symlink_metadata(&member.original.path)?.is_symlink() {
            Some(fs::read_link(&member.original.path)?)
        } else {
            None
        };
        ensure!(
            actual_link == member.original_link,
            "consumed link changed before historical seal"
        );
    }
    for source in index.package_sources.values() {
        ensure!(
            package_files(&source.root)? == source.paths,
            "package source membership changed before historical seal"
        );
    }
    for (path, namespace) in &index.namespaces {
        ensure!(
            namespace_identity(path)? == namespace.identity,
            "namespace changed before historical seal"
        );
    }
    for (path, present) in &index.absences {
        ensure!(
            presence(path)? == *present,
            "absence premise changed before historical seal"
        );
    }
    let mut live_native = BTreeMap::new();
    let mut live_aliases = BTreeMap::<(u64, u64), BTreeSet<PathBuf>>::new();
    for root in &index.declarations.native_namespaces {
        let path = absolute(&index.workspace_root, root);
        let observed = namespace_observation(&path, 262_144)?;
        live_native.insert(path.to_string_lossy().into_owned(), observed.identity);
        for (identity, paths) in observed.aliases {
            live_aliases.entry(identity).or_default().extend(paths);
        }
    }
    live_native.insert(
        "file-identity-aliases".into(),
        namespace_alias_identity(&live_aliases)?,
    );
    ensure!(
        live_native == historical_native_namespaces(&index)?,
        "cross-namespace alias premise changed before historical seal"
    );
    let digest = file_digest(&directory.join("index.json"))?;
    let mut seal = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("COMPLETE"))?;
    seal.write_all(digest.as_bytes())?;
    seal.sync_all()?;
    Ok(())
}

fn protected_bytes(directory: &Path, member: &Member) -> Result<Vec<u8>> {
    ensure!(
        member.protected == PathBuf::from("members").join(&member.original.sha256),
        "invalid protected member path"
    );
    let bytes = fs::read(directory.join(&member.protected))?;
    ensure!(
        bytes.len() as u64 == member.extent && bytes_digest(&bytes) == member.original.sha256,
        "historical raw member missing or changed"
    );
    Ok(bytes)
}

fn historical_native_namespaces(index: &Index) -> Result<BTreeMap<String, String>> {
    let mut state = BTreeMap::new();
    let mut aliases = BTreeMap::<(u64, u64), BTreeSet<PathBuf>>::new();
    for root in &index.declarations.native_namespaces {
        let path = absolute(&index.workspace_root, root);
        let namespace = index
            .namespaces
            .get(&path)
            .context("historical native namespace missing")?;
        state.insert(
            path.to_string_lossy().into_owned(),
            namespace.identity.clone(),
        );
        for (identity, paths) in &namespace.aliases {
            aliases
                .entry(*identity)
                .or_default()
                .extend(paths.iter().cloned());
        }
    }
    state.insert(
        "file-identity-aliases".into(),
        namespace_alias_identity(&aliases)?,
    );
    Ok(state)
}

fn validate(directory: &Path) -> Result<Index> {
    let index: Index = serde_json::from_slice(&fs::read(directory.join("index.json"))?)?;
    ensure!(
        index.version == 1,
        "unsupported historical evidence version"
    );
    for (path, member) in &index.members {
        ensure!(
            path == &member.original.path,
            "historical member original association differs"
        );
        // Stream hashing avoids allocating native libraries merely to verify them.
        ensure!(
            member.protected == PathBuf::from("members").join(&member.original.sha256)
                && fs::metadata(directory.join(&member.protected))?.len() == member.extent
                && file_digest(&directory.join(&member.protected))? == member.original.sha256,
            "historical member missing or changed: {}",
            path.display()
        );
    }
    for (name, expected) in &index.streams {
        ensure!(
            [
                "build.stdout",
                "build.stderr",
                "build.status.json",
                "refresh.stdout",
                "refresh.stderr",
                "refresh.status.json",
                "unit-graph.stdout",
                "unit-graph.arguments.json",
                "metadata.stdout"
            ]
            .contains(&name.as_str()),
            "unknown historical stream"
        );
        ensure!(
            file_digest(&directory.join(name))? == *expected,
            "historical child stream changed"
        );
    }
    if let Some(path) = &index.legacy_evidence {
        for name in [
            "build.stdout",
            "build.stderr",
            "build.status.json",
            "unit-graph.stdout",
            "unit-graph.arguments.json",
            "metadata.stdout",
        ] {
            ensure!(
                index.streams.contains_key(name),
                "historical exact raw context member missing: {name}"
            );
        }
        let original = index
            .members
            .get(path)
            .context("historical exact Cargo context evidence missing")?;
        let evidence: serde_json::Value =
            serde_json::from_slice(&protected_bytes(directory, original)?)?;
        ensure!(
            evidence["unit_graph"] == serde_json::to_value(&index.graph)?,
            "historical selected unit graph differs from exact build context"
        );
        ensure!(
            evidence["arguments"]
                .as_array()
                .is_some_and(|arguments| !arguments.is_empty())
                && evidence["success"] == true,
            "historical exact build invocation missing"
        );
        let raw_graph: UnitGraph =
            serde_json::from_slice(&fs::read(directory.join("unit-graph.stdout"))?)?;
        ensure!(
            serde_json::to_value(raw_graph)? == evidence["unit_graph"],
            "historical raw unit graph differs"
        );
        let _: Vec<String> =
            serde_json::from_slice(&fs::read(directory.join("unit-graph.arguments.json"))?)?;
        let _: cargo_metadata::Metadata =
            serde_json::from_slice(&fs::read(directory.join("metadata.stdout"))?)?;
    }
    let status: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("build.status.json"))?)?;
    ensure!(
        status["success"] == true && status["stream_errors"] == serde_json::json!([]),
        "historical child did not complete successfully"
    );
    let messages = parse_cargo_messages(&fs::read(directory.join("build.stdout"))?)?;
    let root = &index.workspace_root;
    let keys = unit_keys(&index.graph, root)?;
    let selected = selected_indices(&index.graph)?;
    ensure!(
        index.graph.version == 1 && index.graph.roots.len() == 1,
        "historical graph lacks one supported production root"
    );
    ensure!(
        index.receipt.selected_root.as_ref() == keys.get(&index.graph.roots[0]),
        "historical selected root differs"
    );
    for (number, paths) in &index.associations {
        ensure!(
            selected.contains(number),
            "historical association names an unselected unit"
        );
        let unit = &index.graph.units[*number];
        for path in paths {
            let member = index
                .members
                .get(path)
                .context("historical dep-info missing")?;
            let text = String::from_utf8(protected_bytes(directory, member)?)?;
            let outputs = dep_info_outputs(&text)?
                .into_iter()
                .map(|path| absolute(root, &path))
                .collect::<BTreeSet<_>>();
            let inputs = parse_dep_info(&text)?
                .into_iter()
                .map(|path| absolute(root, &path))
                .collect::<BTreeSet<_>>();
            ensure!(
                inputs.contains(&unit.target.src_path),
                "historical dep-info source differs from selected unit"
            );
            ensure!(
                inputs.iter().all(|input| index.members.contains_key(input)),
                "historical generated/consumed input missing"
            );
            let matches = messages
                .iter()
                .filter(|message| artifact_matches(unit, message))
                .any(|message| {
                    message["filenames"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(serde_json::Value::as_str)
                        .any(|file| {
                            let artifact = absolute(root, Path::new(file));
                            let mut artifacts = vec![artifact.clone()];
                            for alias in index.artifact_aliases.get(&artifact).into_iter().flatten()
                            {
                                let same = index
                                    .members
                                    .get(&artifact)
                                    .zip(index.members.get(alias))
                                    .is_some_and(|(a, b)| {
                                        a.original_file_identity.is_some()
                                            && a.original_file_identity == b.original_file_identity
                                    });
                                if same {
                                    artifacts.push(alias.clone());
                                }
                            }
                            artifacts.iter().any(|artifact| {
                                let mut candidates = vec![artifact.with_extension("d")];
                                if let Some(name) = artifact
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .and_then(|name| name.strip_prefix("lib"))
                                {
                                    candidates
                                        .push(artifact.with_file_name(name).with_extension("d"));
                                }
                                candidates.contains(path)
                                    && ((outputs.contains(path) && outputs.contains(artifact))
                                        || cargo_cached_dep_info_linkage(
                                            unit, root, path, artifact, &outputs,
                                        ))
                            })
                        })
                });
            ensure!(
                matches,
                "historical dep-info lacks actual Cargo unit/artifact linkage: {}",
                keys[number]
            );
        }
    }
    for namespace in index.namespaces.values() {
        let mut entries = BTreeMap::new();
        for (label, entry) in &namespace.entries {
            let digest = if entry["absent"] == true {
                hash_parts(&[b"namespace-absence"])
            } else {
                let mode: Vec<u8> = serde_json::from_value(entry["mode_frame"].clone())?;
                let target_mode: Vec<u8> =
                    serde_json::from_value(entry["target_mode_frame"].clone())?;
                let content: Vec<u8> = serde_json::from_value(entry["content_frame"].clone())?;
                let link: Option<PathBuf> = serde_json::from_value(entry["link"].clone())?;
                hash_parts(&[
                    b"native-namespace-entry.v1",
                    entry["kind"].as_str().context("namespace kind")?.as_bytes(),
                    &mode,
                    &target_mode,
                    &serde_json::to_vec(&link)?,
                    &content,
                ])
            };
            entries.insert(label.clone(), digest);
        }
        let mut aliases: Vec<_> = namespace
            .aliases
            .iter()
            .map(|(_, paths)| paths)
            .filter(|paths| paths.len() > 1)
            .collect();
        aliases.sort();
        let alias_identity = hash_parts(&[
            b"native-file-alias-equivalence.v1",
            &serde_json::to_vec(&aliases)?,
        ]);
        ensure!(
            hash_parts(&[
                b"native-namespace.v1",
                &serde_json::to_vec(&entries)?,
                alias_identity.as_bytes()
            ]) == namespace.identity,
            "historical namespace framing differs"
        );
    }
    let native_state = historical_native_namespaces(&index)?;
    for input in index
        .receipt
        .consumed_inputs
        .get("native-namespaces")
        .into_iter()
        .flatten()
    {
        ensure!(
            native_state.get(&input.path) == Some(&input.identity),
            "historical cross-namespace identity differs"
        );
    }
    let mut sources = BTreeMap::new();
    for (package, source) in &index.package_sources {
        let mut hash = FramedHasher::new(Frame::ProducerV1);
        hash.str("reviewed-executable-source");
        let mut previous = None;
        for path in &source.paths {
            ensure!(
                previous.as_ref().is_none_or(|previous| previous < path),
                "historical package member order differs"
            );
            previous = Some(path.clone());
            let member = index
                .members
                .get(path)
                .context("historical package source missing")?;
            hash.str(&path.strip_prefix(&source.root)?.to_string_lossy())
                .part(&protected_bytes(directory, member)?);
        }
        let observed = hash.finish_hash().to_hex();
        ensure!(
            observed == source.identity,
            "historical executable source basis differs"
        );
        sources.insert(package.clone(), observed);
    }
    for number in &selected {
        let unit = &index.graph.units[*number];
        if unit.mode != "run-custom-build"
            && !unit.target.kind.iter().any(|kind| kind == "proc-macro")
        {
            continue;
        }
        let package_id = logical_package(&unit.pkg_id, root);
        if let Some(expected) = index.receipt.reviewed_owner_sources.get(&package_id) {
            ensure!(
                sources.get(&unit.pkg_id) == Some(expected),
                "historical owner source basis differs"
            );
        }
        let name = format!("executable-closure:{package_id}:{}", keys[number]);
        if let Some(expected) = index.receipt.reviewed_owner_sources.get(&name) {
            ensure!(
                executable_closure_basis(&index.graph, *number, &sources, &keys)? == *expected,
                "historical executable helper closure differs"
            );
        }
    }
    let actual_associated = index.associations.keys().copied().collect::<BTreeSet<_>>();
    for number in &selected {
        let unit = &index.graph.units[*number];
        if !unit.target.kind.iter().any(|kind| kind == "proc-macro") {
            continue;
        }
        let package_id = logical_package(&unit.pkg_id, root);
        let review_key = format!("macro-caller-inputs:{package_id}:{}", keys[number]);
        if let Some(expected) = index.receipt.reviewed_owner_sources.get(&review_key) {
            let mut declaration = index
                .declarations
                .proc_macros
                .get(&unit.pkg_id)
                .or_else(|| index.declarations.proc_macros.get(&package_id))
                .cloned();
            if let Some(value) = declaration.as_mut() {
                value.pest_grammars = index
                    .pest_grammars
                    .get(&unit.pkg_id)
                    .or_else(|| index.pest_grammars.get(&package_id))
                    .cloned()
                    .unwrap_or_default();
            }
            let basis = macro_caller_input_basis_with_reader(
                &index.graph,
                *number,
                &keys,
                DepInfoAssociations {
                    paths_by_unit: &index.associations,
                    actual_units: &actual_associated,
                },
                root,
                declaration.as_ref(),
                |path| {
                    if let Some(member) = index.members.get(path) {
                        Ok(Some(protected_bytes(directory, member)?))
                    } else if index
                        .absences
                        .get(path)
                        .is_some_and(|presence| !presence.target)
                    {
                        Ok(None)
                    } else {
                        anyhow::bail!("historical caller premise missing: {}", path.display())
                    }
                },
            )?;
            ensure!(
                basis.as_ref() == Some(expected),
                "historical macro caller basis differs"
            );
        }
    }
    let scientific = serde_json::to_value(&index.receipt)?;
    deployment_identity::verify_scientific_producer_identity(&scientific)?;
    if let Some(association) = &index.receipt.deployment {
        ensure!(
            association.producer_identity == index.receipt.identity
                && Some(&association.selected_root) == index.receipt.selected_root.as_ref(),
            "historical deployment producer/root differs"
        );
        ensure!(
            index
                .members
                .get(&association.built_artifact.path)
                .is_some_and(|member| member.original == association.built_artifact),
            "historical role artifact association missing"
        );
        for file in &association.files {
            ensure!(
                index
                    .members
                    .get(&file.path)
                    .is_some_and(|member| member.original == *file),
                "historical consumed original observation missing"
            );
        }
    }
    Ok(index)
}

pub(crate) fn verify_historical_evidence(directory: &Path) -> Result<serde_json::Value> {
    let expected = fs::read_to_string(directory.join("COMPLETE"))
        .context("historical evidence is incomplete")?;
    ensure!(
        file_digest(&directory.join("index.json"))? == expected,
        "historical index seal differs"
    );
    let index = validate(directory)?;
    // Production captures retain this exact context as an owned raw member.
    // Consumers no longer reopen the original mutable build-evidence JSON.
    let actual_build_evidence = index
        .legacy_evidence
        .as_ref()
        .map(|path| {
            let member = index
                .members
                .get(path)
                .context("historical actual build context missing")?;
            Ok::<serde_json::Value, anyhow::Error>(serde_json::from_slice(&protected_bytes(
                directory, member,
            )?)?)
        })
        .transpose()?;
    Ok(
        serde_json::json!({"version": index.version, "historical_only": true,
        "actual_build_evidence": actual_build_evidence, "context": index.context,
        "unit_graph": index.graph,
        "producer_identity": index.receipt.identity, "selected_root": index.receipt.selected_root,
        "unit_associations": index.associations.len(), "protected_members": index.members.len(),
        "namespace_raw_coverage": "inventory-only except separately retained consumed members",
        "reviewed_owner_sources": index.receipt.reviewed_owner_sources}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(directory: &Path) -> (PathBuf, PathBuf) {
        let original = directory.join("original");
        let evidence = directory.join("capture.closure");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(evidence.join("members")).unwrap();
        let source = original.join("lib.rs");
        let generated = directory.join("outside-generated.rs");
        let artifact = original.join("libselected-123.rlib");
        let dep_info = original.join("selected-123.d");
        fs::write(&source, b"include!(\"../outside-generated.rs\");").unwrap();
        fs::write(&generated, b"pub fn value() -> u32 { 7 }").unwrap();
        fs::write(&artifact, b"independently supplied artifact fixture bytes").unwrap();
        fs::write(
            &dep_info,
            format!(
                "{} {}: {} {}\n# env-dep:SELECTED=actual\n",
                artifact.display(),
                dep_info.display(),
                source.display(),
                generated.display()
            ),
        )
        .unwrap();
        let unit = Unit {
            pkg_id: "selected#0.1.0".into(),
            target: CargoTarget {
                kind: vec!["lib".into()],
                crate_types: vec!["lib".into()],
                name: "selected".into(),
                src_path: source.clone(),
                edition: "2024".into(),
            },
            profile: serde_json::json!({"name":"dev","opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true}),
            platform: None,
            mode: "build".into(),
            features: Vec::new(),
            dependencies: Vec::new(),
            is_std: false,
        };
        let mut profile = unit.profile.clone();
        profile["test"] = serde_json::json!(false);
        let message = serde_json::json!({"reason":"compiler-artifact","package_id":unit.pkg_id,
            "target": {"kind":["lib"],"crate_types":["lib"],"name":"selected","src_path":source,"edition":"2024"},
            "profile":profile,"features":[],"filenames":[artifact]});
        fs::write(
            evidence.join("build.stdout"),
            format!(
                "{}\n{{\"reason\":\"build-finished\",\"success\":true}}\n",
                message
            ),
        )
        .unwrap();
        fs::write(
            evidence.join("build.stderr"),
            b"selected child diagnostic\n",
        )
        .unwrap();
        fs::write(
            evidence.join("build.status.json"),
            br#"{"success":true,"code":0,"stream_errors":[]}"#,
        )
        .unwrap();
        let graph = UnitGraph {
            version: 1,
            units: vec![unit],
            roots: vec![0],
        };
        let selected_root = unit_keys(&graph, &original).unwrap()[&0].clone();
        let artifact_observation =
            deployment_identity::FileObservation::capture(&artifact).unwrap();
        let source_observation = deployment_identity::FileObservation::capture(&source).unwrap();
        let mut receipt = ProducerIdentity {
            receipt_version: deployment_identity::DEPLOYMENT_RECEIPT_VERSION,
            deployment: Some(deployment_identity::DeploymentAssociation {
                producer_identity: String::new(),
                selected_root: selected_root.clone(),
                workspace_root: original.clone(),
                built_artifact: artifact_observation.clone(),
                artifact: artifact_observation,
                files: vec![source_observation],
                absent_files: vec![original.join("absent")],
                namespaces: BTreeMap::new(),
                native_environment: BTreeMap::new(),
                native_absent_environment_prefixes: Vec::new(),
            }),
            native_provider: None,
            frame: Frame::ProducerV1.as_str().into(),
            package: "selected".into(),
            selected_root: Some(selected_root),
            identity: String::new(),
            persistent_reuse_eligible: false,
            reasons: vec!["fixture only; no executor/native qualification".into()],
            units: Vec::new(),
            selected_lock_records: Vec::new(),
            consumed_inputs: BTreeMap::new(),
            declared_environment: BTreeMap::new(),
            native_abi: None,
            reviewed_owner_sources: BTreeMap::new(),
        };
        receipt.identity = deployment_identity::scientific_producer_identity(
            &receipt.package,
            &receipt.units,
            &receipt.selected_lock_records,
            &receipt.consumed_inputs,
            &receipt.declared_environment,
            &receipt.native_abi,
        )
        .unwrap()
        .to_hex();
        receipt.deployment.as_mut().unwrap().producer_identity = receipt.identity.clone();
        let mut members = BTreeMap::new();
        for path in [&source, &generated, &artifact, &dep_info] {
            retain(&evidence, path, &mut members).unwrap();
        }
        let namespace = namespace_observation(&original, 64).unwrap();
        let index = Index {
            version: 1,
            purpose: "historical fixture only".into(),
            workspace_root: original.clone(),
            context: serde_json::json!({}),
            legacy_evidence: None,
            graph,
            receipt,
            declarations: InputDeclarations::default(),
            pest_grammars: BTreeMap::new(),
            package_sources: BTreeMap::new(),
            associations: BTreeMap::from([(0, vec![dep_info])]),
            artifact_aliases: BTreeMap::new(),
            members,
            namespaces: BTreeMap::from([(
                original.clone(),
                Namespace {
                    identity: namespace.identity,
                    entries: namespace.entries,
                    aliases: namespace.aliases.into_iter().collect(),
                    raw_coverage: "inventory-only".into(),
                },
            )]),
            absences: BTreeMap::from([(
                original.join("absent"),
                Presence {
                    entry: false,
                    target: false,
                },
            )]),
            streams: ["build.stdout", "build.stderr", "build.status.json"]
                .into_iter()
                .map(|name| (name.into(), file_digest(&evidence.join(name)).unwrap()))
                .collect(),
        };
        fs::write(
            evidence.join("index.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        (original, evidence)
    }

    #[cfg(unix)]
    #[test]
    fn producer_historical_actual_worker_survives_second_role_executor_refresh() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("workspace");
        let package = root.join("model");
        let captures = directory.path().join("captures");
        fs::create_dir_all(package.join("src/bin")).unwrap();
        fs::create_dir_all(root.join("emitter/src")).unwrap();
        fs::create_dir(&captures).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers=['model','emitter']\nresolver='3'\n",
        )
        .unwrap();
        fs::write(root.join("emitter/Cargo.toml"), "[package]\nname='p39-history-emitter'\nversion='0.1.0'\nedition='2024'\n[lib]\nproc-macro=true\n").unwrap();
        fs::write(root.join("emitter/src/lib.rs"), "extern crate proc_macro;\n#[proc_macro] pub fn identity(input: proc_macro::TokenStream) -> proc_macro::TokenStream { input }\n").unwrap();
        fs::write(package.join("Cargo.toml"), "[package]\nname='p39-history-fixture'\nversion='0.1.0'\nedition='2024'\n[lib]\ncrate-type=['cdylib']\n[features]\npython-context=[]\n[dependencies]\np39-history-emitter={path='../emitter',version='=0.1.0'}\n").unwrap();
        fs::write(package.join("build.rs"), r#"fn main() {
            let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
            std::fs::write(out.join("generated.rs"), "pub fn generated_value() -> u32 { 7 }\n").unwrap();
            println!("cargo:rerun-if-changed=build.rs");
        }"#).unwrap();
        fs::write(package.join("src/lib.rs"), "p39_history_emitter::identity! { include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\")); }\n").unwrap();
        fs::write(package.join("src/bin/worker.rs"), "p39_history_emitter::identity! { include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\")); }\nfn main() { assert_eq!(generated_value(), 7); }\n").unwrap();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        fs::copy(
            repository.join("rust-toolchain.toml"),
            root.join("rust-toolchain.toml"),
        )
        .unwrap();
        cargo_output(&root, &["generate-lockfile".into(), "--offline".into()]).unwrap();
        let metadata: cargo_metadata::Metadata = serde_json::from_slice(
            &cargo_output(
                &root,
                &[
                    "metadata".into(),
                    "--format-version=1".into(),
                    "--offline".into(),
                ],
            )
            .unwrap(),
        )
        .unwrap();
        let emitter = metadata
            .packages
            .iter()
            .find(|package| package.name.as_str() == "p39-history-emitter")
            .unwrap()
            .id
            .to_string();
        let worker_evidence = captures.join("worker.json");
        let options = ProducerOptions {
            workspace_root: root.clone(),
            package: "p39-history-fixture".into(),
            profile: "dev".into(),
            target: None,
            production_target: ProducerTarget::Binary("worker".into()),
            features: Vec::new(),
            no_default_features: false,
            dep_info: Vec::new(),
            declared_inputs: Vec::new(),
            native_inputs: Vec::new(),
            declared_environment: BTreeMap::new(),
            declarations: InputDeclarations {
                capture_actual_build: true,
                actual_build_evidence: Some(worker_evidence.clone()),
                proc_macros: BTreeMap::from([(
                    emitter,
                    BuildScriptInputs {
                        reviewed_callers: Some(Vec::new()),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
        };
        let worker = run(&options).unwrap();
        // This is a real historical capture, deliberately not an arbitrary-I/O
        // qualification of the fixture build script, host compiler or native ABI.
        assert!(!worker.persistent_reuse_eligible);
        assert!(
            worker
                .reviewed_owner_sources
                .keys()
                .any(|name| name.starts_with("macro-caller-inputs:"))
        );
        let evidence = captures.join("worker.json.closure");
        let worker_index: Index =
            serde_json::from_slice(&fs::read(evidence.join("index.json")).unwrap()).unwrap();
        let dep_info: Vec<_> = worker_index
            .associations
            .values()
            .flatten()
            .cloned()
            .collect();
        let generated: Vec<_> = worker_index
            .members
            .keys()
            .filter(|path| path.file_name().is_some_and(|name| name == "generated.rs"))
            .cloned()
            .collect();
        assert!(!dep_info.is_empty());
        assert!(!generated.is_empty());
        assert!(generated.iter().all(|path| !path.starts_with(&package)));
        let mut second = options.clone();
        second.production_target = ProducerTarget::Cdylib(package.join("Cargo.toml"));
        second.features = vec!["python-context".into()];
        second.declarations.refresh_executors = true;
        second.declarations.actual_build_evidence = Some(captures.join("python-like.json"));
        run(&second).unwrap();
        assert!(
            dep_info.iter().any(|path| !path.exists()),
            "actual executor refresh must remove first role dep-info"
        );
        assert!(
            generated.iter().any(|path| !path.exists()),
            "second role context must replace first role generated inputs"
        );
        assert!(
            worker
                .deployment
                .as_ref()
                .unwrap()
                .built_artifact
                .verify()
                .is_err()
        );
        assert_eq!(
            verify_historical_evidence(&evidence).unwrap()["historical_only"],
            true
        );
        let retained_context =
            verify_historical_evidence(&evidence).unwrap()["actual_build_evidence"].clone();
        assert_eq!(
            retained_context,
            serde_json::from_slice::<serde_json::Value>(&fs::read(&worker_evidence).unwrap())
                .unwrap()
        );
        fs::remove_file(&worker_evidence).unwrap();
        fs::remove_dir_all(&root).unwrap();
        let historical = verify_historical_evidence(&evidence).unwrap();
        assert_eq!(historical["actual_build_evidence"], retained_context);
        assert_eq!(historical["context"]["package"], "p39-history-fixture");
        let context_member = &worker_index.members[&worker_evidence].protected;
        fs::write(
            evidence.join(context_member),
            b"substituted installer/compiler context",
        )
        .unwrap();
        assert!(verify_historical_evidence(&evidence).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn producer_historical_evidence_reconstructs_cross_root_aliases_and_dangling_presence() {
        let directory = tempfile::tempdir().unwrap();
        let (_, evidence) = fixture(directory.path());
        let mut index: Index =
            serde_json::from_slice(&fs::read(evidence.join("index.json")).unwrap()).unwrap();
        let left = directory.path().join("include-left");
        let right = directory.path().join("include-right");
        fs::create_dir(&left).unwrap();
        fs::create_dir(&right).unwrap();
        fs::write(left.join("shared.h"), b"identical bytes").unwrap();
        fs::hard_link(left.join("shared.h"), right.join("shared.h")).unwrap();
        index.declarations.native_namespaces = vec![left.clone(), right.clone()];
        let observe = |root: &Path| {
            let observed = namespace_observation(root, 64).unwrap();
            Namespace {
                identity: observed.identity,
                entries: observed.entries,
                aliases: observed.aliases.into_iter().collect(),
                raw_coverage: "inventory-only".into(),
            }
        };
        index.namespaces.insert(left.clone(), observe(&left));
        index.namespaces.insert(right.clone(), observe(&right));
        let linked = historical_native_namespaces(&index).unwrap();
        let options = ProducerOptions {
            workspace_root: index.workspace_root.clone(),
            package: "fixture".into(),
            profile: "dev".into(),
            target: None,
            production_target: ProducerTarget::Library,
            features: Vec::new(),
            no_default_features: false,
            dep_info: Vec::new(),
            declared_inputs: Vec::new(),
            native_inputs: Vec::new(),
            declared_environment: BTreeMap::new(),
            declarations: index.declarations.clone(),
        };
        assert_eq!(
            linked,
            native_namespace_state(&options, &index.workspace_root).unwrap()
        );
        fs::remove_file(right.join("shared.h")).unwrap();
        fs::write(right.join("shared.h"), b"identical bytes").unwrap();
        index.namespaces.insert(left.clone(), observe(&left));
        index.namespaces.insert(right.clone(), observe(&right));
        let copied = historical_native_namespaces(&index).unwrap();
        assert_eq!(
            linked[&left.to_string_lossy().into_owned()],
            copied[&left.to_string_lossy().into_owned()]
        );
        assert_eq!(
            linked[&right.to_string_lossy().into_owned()],
            copied[&right.to_string_lossy().into_owned()]
        );
        assert_ne!(
            linked["file-identity-aliases"],
            copied["file-identity-aliases"]
        );
        let dangling = directory.path().join("dangling");
        std::os::unix::fs::symlink("absent-target", &dangling).unwrap();
        assert_eq!(
            presence(&dangling).unwrap(),
            Presence {
                entry: true,
                target: false
            }
        );
    }

    #[test]
    fn producer_historical_evidence_survives_refresh_of_original_paths() {
        let directory = tempfile::tempdir().unwrap();
        let (original, evidence) = fixture(directory.path());
        seal(&evidence).unwrap();
        fs::remove_dir_all(&original).unwrap();
        fs::remove_file(directory.path().join("outside-generated.rs")).unwrap();
        let observed = verify_historical_evidence(&evidence).unwrap();
        assert_eq!(observed["historical_only"], true);
        assert_eq!(observed["unit_associations"], 1);
        assert_eq!(observed["protected_members"], 4);
        assert!(!original.exists());
    }

    #[test]
    fn producer_historical_evidence_refuses_missing_changed_and_unsealed_members() {
        for mutation in ["unsealed", "raw", "missing", "index"] {
            let directory = tempfile::tempdir().unwrap();
            let (_, evidence) = fixture(directory.path());
            if mutation != "unsealed" {
                seal(&evidence).unwrap();
            }
            let index: Index =
                serde_json::from_slice(&fs::read(evidence.join("index.json")).unwrap()).unwrap();
            let member = &index.members.values().next().unwrap().protected;
            match mutation {
                "raw" => fs::write(evidence.join(member), b"tampered").unwrap(),
                "missing" => fs::remove_file(evidence.join(member)).unwrap(),
                "index" => fs::write(evidence.join("index.json"), b"{}").unwrap(),
                _ => (),
            }
            assert!(verify_historical_evidence(&evidence).is_err(), "{mutation}");
        }
    }

    #[test]
    fn producer_historical_evidence_refuses_substituted_association_and_changed_absence() {
        let directory = tempfile::tempdir().unwrap();
        let (_original, evidence) = fixture(directory.path());
        let mut index: Index =
            serde_json::from_slice(&fs::read(evidence.join("index.json")).unwrap()).unwrap();
        index.graph.units[0]
            .features
            .push("different-compilation".into());
        fs::write(
            evidence.join("index.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        assert!(
            seal(&evidence)
                .unwrap_err()
                .to_string()
                .contains("historical selected root differs")
        );
        // A self-consistent substituted root still cannot inherit another actual
        // compiler context's dep-info association from the retained Cargo stream.
        let replacement = unit_keys(&index.graph, &index.workspace_root).unwrap()[&0].clone();
        index.receipt.selected_root = Some(replacement.clone());
        index.receipt.deployment.as_mut().unwrap().selected_root = replacement;
        fs::write(
            evidence.join("index.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        assert!(
            seal(&evidence)
                .unwrap_err()
                .to_string()
                .contains("Cargo unit/artifact linkage")
        );
        assert!(!evidence.join("COMPLETE").exists());
        let directory = tempfile::tempdir().unwrap();
        let (original, evidence) = fixture(directory.path());
        fs::write(original.join("absent"), b"new shadow").unwrap();
        assert!(seal(&evidence).is_err());
        assert!(!evidence.join("COMPLETE").exists());
        let _ = original;
    }
}
