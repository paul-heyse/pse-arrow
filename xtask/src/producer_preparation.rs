// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A finite, nonresumable compilation sharing lifetime. This carries actual
//! compilation evidence, never executable I/O completeness or current admission.

use super::*;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct State {
    version: u32,
    root: PathBuf,
    directory: PathBuf,
    owner_pid: u32,
    owner_start: String,
    profile: String,
    target: Option<String>,
    families: BTreeSet<String>,
    configuration: BTreeMap<String, String>,
    sources: BTreeMap<String, String>,
    manifest: String,
    lock: String,
    compiled: BTreeMap<String, Compilation>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Compilation {
    basis: String,
    outputs: Vec<deployment_identity::FileObservation>,
    evidence: PathBuf,
    evidence_hash: String,
    premises: PathBuf,
    premises_hash: String,
}

fn new_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}

fn write_new(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut file = new_file(path)?;
    serde_json::to_writer(&mut file, value)?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

fn bytes_hash(path: &Path) -> Result<String> {
    Ok(deployment_identity::value_digest(&fs::read(path)?))
}

fn process(pid: u32) -> Result<(u32, String)> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let fields = stat.rsplit_once(") ").context("process identity framing")?.1
        .split_whitespace().collect::<Vec<_>>();
    Ok((fields.get(1).context("process parent missing")?.parse()?,
        fields.get(19).context("process start missing")?.to_string()))
}

fn owner_is_ancestor(pid: u32, start: &str) -> Result<()> {
    ensure!(process(pid)?.1 == start, "preparation owner lifetime changed");
    let mut current = std::process::id();
    for _ in 0..256 {
        if current == pid { return Ok(()); }
        ensure!(current > 1, "capture is not a preparation owner descendant");
        current = process(current)?.0;
    }
    anyhow::bail!("preparation ancestry exceeds finite bound")
}

fn guard_path(root: &Path) -> PathBuf { root.join("target/.producer-preparation.lock") }

fn require_held_guard(root: &Path) -> Result<()> {
    let file = OpenOptions::new().read(true).write(true).open(guard_path(root))?;
    match file.try_lock() {
        Ok(()) => anyhow::bail!("preparation target guard is not held by its owner"),
        Err(std::fs::TryLockError::WouldBlock) => Ok(()),
        Err(error) => Err(anyhow::anyhow!("cannot observe preparation guard: {error}")),
    }
}

/// Only strict preparation/capture operations cooperate with this target-local
/// exclusion. It is not a host lock or protection from arbitrary Cargo tools.
pub(super) fn standalone_guard(options: &ProducerOptions) -> Result<Option<File>> {
    if options.declarations.preparation.is_some() || !options.declarations.capture_actual_build {
        return Ok(None);
    }
    let root = options.workspace_root.canonicalize()?;
    fs::create_dir_all(root.join("target"))?;
    let guard = OpenOptions::new().read(true).write(true).create(true).truncate(false)
        .open(guard_path(&root))?;
    guard.try_lock().map_err(|error| anyhow::anyhow!("another strict preparation owns this target: {error}"))?;
    Ok(Some(guard))
}

pub(crate) fn begin(root: &Path, directory: &Path, selections: Vec<ProducerOptions>, owner_pid: u32, owner_start: &str) -> Result<()> {
    ensure!(!selections.is_empty(), "preparation requires selected roles");
    let root = root.canonicalize()?;
    let directory = directory.canonicalize()?;
    owner_is_ancestor(owner_pid, owner_start)?;
    require_held_guard(&root)?;
    let guard: serde_json::Value = serde_json::from_slice(&fs::read(guard_path(&root))?)?;
    ensure!(guard["directory"] == directory.to_string_lossy().as_ref()
        && guard["owner_pid"] == owner_pid && guard["owner_start"] == owner_start,
        "preparation target guard belongs to another owner");
    ensure!(!path_entry_exists(&directory.join("state.json"))?, "preparation is single-use");
    let first = &selections[0];
    let mut families = BTreeSet::new();
    let mut sources = BTreeMap::new();
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&cargo_output(&root,
        &["metadata".into(), "--format-version=1".into(), "--locked".into(), "--offline".into(), "--all-features".into()])?)?;
    let packages = metadata.packages.into_iter().map(|p| (p.id.to_string(), p)).collect::<BTreeMap<_, _>>();
    let configuration = configuration_state(&root)?;
    let manifest = bytes_hash(&root.join("Cargo.toml"))?;
    let lock = bytes_hash(&root.join("Cargo.lock"))?;
    for selection in &selections {
        ensure!(selection.workspace_root.canonicalize()? == root
            && selection.profile == first.profile && selection.target == first.target,
            "preparation selections differ in workspace/profile/target");
        let graph: UnitGraph = serde_json::from_slice(&cargo_output(&root, &unit_graph_arguments(selection))?)?;
        ensure!(graph.version == 1, "unsupported preparation graph");
        validate_production_root(&graph, &selection.production_target)?;
        families.extend(executor_refresh_families(&executor_refresh_packages(&graph)?, &packages)?);
        sources.extend(selected_source_state(&graph, &packages)?);
    }
    ensure!(families.len() <= 256, "preparation executor family bound exceeded");
    let mut arguments = vec!["clean".into(), "--profile".into(), first.profile.clone()];
    if let Some(target) = &first.target { arguments.extend(["--target".into(), target.clone()]); }
    for family in &families { arguments.extend(["--package".into(), family.clone()]); }
    // Never issue unqualified cargo clean when the selected graph is pure.
    if !families.is_empty() {
        let output = streamed_child(Command::new("cargo").current_dir(&root).args(&arguments),
            Some(&directory), "refresh", false, &mut std::io::stderr())?;
        write_new(&directory.join("refresh.json"), &serde_json::json!({"arguments":arguments,
            "success":output.status.success(), "stream_errors":output.stream_errors,
            "stdout":String::from_utf8_lossy(&output.stdout),
            "stderr":String::from_utf8_lossy(&output.stderr)}))?;
        ensure!(output.status.success() && output.stream_errors.is_empty(), "preparation family refresh failed");
    }
    ensure!(configuration_state(&root)? == configuration
        && bytes_hash(&root.join("Cargo.toml"))? == manifest
        && bytes_hash(&root.join("Cargo.lock"))? == lock,
        "preparation configuration changed during family refresh");
    for (id, expected) in &sources {
        let package = packages.get(id).context("preparation source package")?;
        ensure!(reviewed_source_identity(package.manifest_path.as_std_path().parent().context("package root")?)? == *expected,
            "preparation source changed during family refresh");
    }
    write_new(&directory.join("state.json"), &State { version:1, root, directory:directory.clone(),
        owner_pid, owner_start:owner_start.into(), profile:first.profile.clone(), target:first.target.clone(),
        families, configuration, sources, manifest, lock, compiled:BTreeMap::new() })
}

pub(super) struct Capture {
    directory: PathBuf,
    state: State,
    pending: BTreeMap<String, Compilation>,
    _lock: File,
    done: bool,
}

impl Capture {
    pub(super) fn enter(options: &ProducerOptions, root: &Path, graph: &UnitGraph,
        packages: &BTreeMap<String, cargo_metadata::Package>) -> Result<Option<Self>> {
        let Some(directory) = &options.declarations.preparation else { return Ok(None); };
        ensure!(options.declarations.capture_actual_build && options.declarations.actual_build_evidence.is_some(),
            "preparation enrollment requires actual protected build evidence");
        let directory = absolute(root, directory).canonicalize()?;
        let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(directory.join("capture.lock"))?;
        lock.try_lock().map_err(|error| anyhow::anyhow!("preparation capture is already active: {error}"))?;
        ensure!(!path_entry_exists(&directory.join("FAILED"))?
            && !path_entry_exists(&directory.join("COMPLETE"))?
            && !path_entry_exists(&directory.join("in-progress.json"))?,
            "preparation is terminal or has an incomplete capture; start a new operation");
        let state: State = serde_json::from_slice(&fs::read(directory.join("state.json"))?)?;
        ensure!(state.version == 1 && state.directory == directory && state.root == root,
            "preparation identity differs from selected workspace");
        owner_is_ancestor(state.owner_pid, &state.owner_start)?;
        require_held_guard(root)?;
        let guard: serde_json::Value = serde_json::from_slice(&fs::read(guard_path(root))?)?;
        ensure!(guard["directory"] == directory.to_string_lossy().as_ref()
            && guard["owner_pid"] == state.owner_pid && guard["owner_start"] == state.owner_start,
            "preparation target guard changed");
        ensure!(state.profile == options.profile && state.target == options.target,
            "actual child selects another preparation target/profile");
        ensure!(configuration_state(root)? == state.configuration
            && bytes_hash(&root.join("Cargo.toml"))? == state.manifest
            && bytes_hash(&root.join("Cargo.lock"))? == state.lock,
            "preparation source/configuration baseline changed");
        let families = executor_refresh_families(&executor_refresh_packages(graph)?, packages)?;
        ensure!(families.is_subset(&state.families), "actual child requires an unrefreshed executor family; start a new preparation");
        for (id, source) in selected_source_state(graph, packages)? {
            ensure!(state.sources.get(&id) == Some(&source), "actual child source is outside preparation baseline");
        }
        write_new(&directory.join("in-progress.json"), &serde_json::json!({"pid":std::process::id(),
            "package":options.package,"evidence":options.declarations.actual_build_evidence}))?;
        Ok(Some(Self { directory, state, pending:BTreeMap::new(), _lock:lock, done:false }))
    }

    pub(super) fn validate(&mut self, options: &ProducerOptions, root: &Path, graph: &UnitGraph,
        packages: &BTreeMap<String, cargo_metadata::Package>, messages: &[serde_json::Value], identity: &ProducerIdentity) -> Result<()> {
        let keys = unit_keys(graph, root)?;
        let sources = selected_source_state(graph, packages)?;
        let evidence = absolute(root, options.declarations.actual_build_evidence.as_ref().context("actual preparation evidence")?);
        let premises = evidence.with_extension("preparation-premises.json");
        write_new(&premises, &deployment_association(options,root,graph,messages,identity)?)?;
        let premises_hash = bytes_hash(&premises)?;
        let mut verified = BTreeSet::new();
        let mut preceding = BTreeMap::new();
        for index in selected_indices(graph)? {
            let unit = &graph.units[index];
            // run-custom-build output association is captured separately by the
            // existing owner. It is not a compiler-artifact Fresh assertion.
            if unit.mode != "build" || !unit.target.kind.iter().any(|kind| kind == "custom-build" || kind == "proc-macro") { continue; }
            let artifacts = messages.iter().filter(|message| artifact_matches(unit, message)).collect::<Vec<_>>();
            ensure!(artifacts.len() == 1, "executor lacks an unambiguous exact compilation association");
            let message = artifacts[0];
            let mut normalized = options.clone();
            normalized.package = unit.pkg_id.clone();
            normalized.production_target = ProducerTarget::Library;
            normalized.features = unit.features.clone();
            normalized.no_default_features = false;
            // Review the dependency-complete executable closure, rather than the
            // unrelated role's package/target as though it were executable input.
            let mut closure = graph.clone();
            closure.roots = vec![index];
            let owners = selected_indices(&closure)?.into_iter().map(|i| graph.units[i].pkg_id.clone()).collect::<BTreeSet<_>>();
            normalized.declarations.build_scripts.retain(|owner, _| owners.contains(owner) || owners.iter().any(|id| logical_package(id, root) == *owner));
            normalized.declarations.proc_macros.retain(|owner, _| owners.contains(owner) || owners.iter().any(|id| logical_package(id, root) == *owner));
            let basis = hash_parts(&[b"finite-preparation-executor.v1", &serde_json::to_vec(&(
                &keys[&index], executable_closure_basis(graph,index,&sources,&keys)?,
                reviewed_input_state(&normalized,root)?, native_caller_environment(&normalized)?,
                native_namespace_state(&normalized,root)?, native_environment_prefix_state(&normalized)?,
                &normalized.declarations.build_scripts, &normalized.declarations.proc_macros))?]);
            let names = message["filenames"].as_array().context("executor output files missing")?;
            let mut outputs = names.iter().map(|name| deployment_identity::FileObservation::capture(
                &absolute(root, Path::new(name.as_str().context("executor output name")?))).map_err(anyhow::Error::from))
                .collect::<Result<Vec<_>>>()?;
            outputs.sort(); outputs.dedup();
            ensure!(!outputs.is_empty(), "executor compilation has no original output");
            match message["fresh"].as_bool() {
                Some(false) => { self.pending.insert(keys[&index].clone(), Compilation { basis, outputs,
                    evidence:evidence.clone(), evidence_hash:bytes_hash(&evidence)?,
                    premises:premises.clone(), premises_hash:premises_hash.clone() }); }
                Some(true) => {
                    let prior = self.state.compiled.get(&keys[&index]).context("Fresh executor lacks a preceding compilation in this preparation")?;
                    ensure!(prior.basis == basis && prior.outputs == outputs
                        && bytes_hash(&prior.evidence)? == prior.evidence_hash,
                        "Fresh executor preparation premises or original outputs changed");
                    for output in &prior.outputs { output.verify()?; }
                    preceding.insert(prior.evidence.clone(), serde_json::json!({
                        "compilation": prior, "actual_build_evidence":serde_json::from_slice::<serde_json::Value>(&fs::read(&prior.evidence)?)?,
                        "original_premises":serde_json::from_slice::<serde_json::Value>(&fs::read(&prior.premises)?)?}));
                    if verified.insert(prior.premises.clone()) {
                        ensure!(bytes_hash(&prior.premises)? == prior.premises_hash, "preparation consumed premises changed");
                        let association: deployment_identity::DeploymentAssociation = serde_json::from_slice(&fs::read(&prior.premises)?)?;
                        association.verify_current(&association.artifact.path)?;
                    }
                }
                None => anyhow::bail!("executor compilation has no actual Fresh state"),
            }
        }
        // The historical sidecar protects this complete raw context. Its review
        // must not depend on a later-live operation marker or earlier role path.
        let mut context: serde_json::Value = serde_json::from_slice(&fs::read(&evidence)?)?;
        context["preparation"] = serde_json::json!({"version":1,"directory":self.directory,
            "owner_pid":self.state.owner_pid,"owner_start":self.state.owner_start,
            "families":self.state.families,"first_compiled_contexts":self.pending.keys().collect::<Vec<_>>(),
            "preceding_compilations":preceding,
            "refresh":if self.directory.join("refresh.json").is_file() { Some(serde_json::from_slice::<serde_json::Value>(&fs::read(self.directory.join("refresh.json"))?)?) } else {None},
            "assurance":"fresh executor compilation; script output association is not execution proof"});
        let mut output = OpenOptions::new().write(true).truncate(true).open(&evidence)?;
        serde_json::to_writer(&mut output, &context)?;
        output.flush()?; output.sync_all()?;
        let hash = bytes_hash(&evidence)?;
        for compiled in self.pending.values_mut() { compiled.evidence_hash = hash.clone(); }
        Ok(())
    }

    pub(super) fn commit(&mut self) -> Result<()> {
        self.state.compiled.append(&mut self.pending);
        let next = self.directory.join("state.next.json");
        write_new(&next, &self.state)?;
        fs::rename(next, self.directory.join("state.json"))?;
        fs::remove_file(self.directory.join("in-progress.json"))?;
        self.done = true;
        Ok(())
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if !self.done { let _ = write_new(&self.directory.join("FAILED"), &serde_json::json!({"terminal":"incomplete capture"})); }
    }
}
