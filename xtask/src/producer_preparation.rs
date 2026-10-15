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
    #[cfg(unix)]
    {
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
    let fields = stat
        .rsplit_once(") ")
        .context("process identity framing")?
        .1
        .split_whitespace()
        .collect::<Vec<_>>();
    Ok((
        fields.get(1).context("process parent missing")?.parse()?,
        fields.get(19).context("process start missing")?.to_string(),
    ))
}

fn owner_is_ancestor(pid: u32, start: &str) -> Result<()> {
    ensure!(
        process(pid)?.1 == start,
        "preparation owner lifetime changed"
    );
    let mut current = std::process::id();
    for _ in 0..256 {
        if current == pid {
            return Ok(());
        }
        ensure!(current > 1, "capture is not a preparation owner descendant");
        current = process(current)?.0;
    }
    anyhow::bail!("preparation ancestry exceeds finite bound")
}

fn guard_path(root: &Path) -> PathBuf {
    root.join("target/.producer-preparation.lock")
}

fn require_held_guard(root: &Path) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(guard_path(root))?;
    match file.try_lock() {
        Ok(()) => anyhow::bail!("preparation target guard is not held by its owner"),
        Err(fs::TryLockError::WouldBlock) => Ok(()),
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
    let guard = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(guard_path(&root))?;
    guard
        .try_lock()
        .map_err(|error| anyhow::anyhow!("another strict preparation owns this target: {error}"))?;
    Ok(Some(guard))
}

pub(crate) fn begin(
    root: &Path,
    directory: &Path,
    selections: Vec<ProducerOptions>,
    owner_pid: u32,
    owner_start: &str,
) -> Result<()> {
    ensure!(
        !selections.is_empty(),
        "preparation requires selected roles"
    );
    let root = root.canonicalize()?;
    let directory = directory.canonicalize()?;
    owner_is_ancestor(owner_pid, owner_start)?;
    require_held_guard(&root)?;
    let guard: serde_json::Value = serde_json::from_slice(&fs::read(guard_path(&root))?)?;
    ensure!(
        guard["directory"] == directory.to_string_lossy().as_ref()
            && guard["owner_pid"] == owner_pid
            && guard["owner_start"] == owner_start,
        "preparation target guard belongs to another owner"
    );
    ensure!(
        !path_entry_exists(&directory.join("state.json"))?,
        "preparation is single-use"
    );
    let first = &selections[0];
    let mut families = BTreeSet::new();
    let mut sources = BTreeMap::new();
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&cargo_output(
        &root,
        &[
            "metadata".into(),
            "--format-version=1".into(),
            "--locked".into(),
            "--offline".into(),
            "--all-features".into(),
        ],
    )?)?;
    let packages = metadata
        .packages
        .into_iter()
        .map(|p| (p.id.to_string(), p))
        .collect::<BTreeMap<_, _>>();
    let configuration = configuration_state(&root)?;
    let manifest = bytes_hash(&root.join("Cargo.toml"))?;
    let lock = bytes_hash(&root.join("Cargo.lock"))?;
    for selection in &selections {
        ensure!(
            selection.workspace_root.canonicalize()? == root
                && selection.profile == first.profile
                && selection.target == first.target,
            "preparation selections differ in workspace/profile/target"
        );
        let graph: UnitGraph =
            serde_json::from_slice(&cargo_output(&root, &unit_graph_arguments(selection))?)?;
        ensure!(graph.version == 1, "unsupported preparation graph");
        validate_production_root(&graph, &selection.production_target)?;
        families.extend(executor_refresh_families(
            &executor_refresh_packages(&graph)?,
            &packages,
        )?);
        sources.extend(selected_source_state(&graph, &packages)?);
    }
    ensure!(
        families.len() <= 256,
        "preparation executor family bound exceeded"
    );
    let mut arguments = vec!["clean".into(), "--profile".into(), first.profile.clone()];
    if let Some(target) = &first.target {
        arguments.extend(["--target".into(), target.clone()]);
    }
    for family in &families {
        arguments.extend(["--package".into(), family.clone()]);
    }
    // Never issue unqualified cargo clean when the selected graph is pure.
    if !families.is_empty() {
        let output = streamed_child(
            Command::new("cargo").current_dir(&root).args(&arguments),
            Some(&directory),
            "refresh",
            false,
            &mut std::io::stderr(),
        )?;
        write_new(
            &directory.join("refresh.json"),
            &serde_json::json!({"arguments":arguments,
            "success":output.status.success(), "stream_errors":output.stream_errors,
            "stdout":String::from_utf8_lossy(&output.stdout),
            "stderr":String::from_utf8_lossy(&output.stderr)}),
        )?;
        ensure!(
            output.status.success() && output.stream_errors.is_empty(),
            "preparation family refresh failed"
        );
    }
    ensure!(
        configuration_state(&root)? == configuration
            && bytes_hash(&root.join("Cargo.toml"))? == manifest
            && bytes_hash(&root.join("Cargo.lock"))? == lock,
        "preparation configuration changed during family refresh"
    );
    for (id, expected) in &sources {
        let package = packages.get(id).context("preparation source package")?;
        ensure!(
            reviewed_source_identity(
                package
                    .manifest_path
                    .as_std_path()
                    .parent()
                    .context("package root")?
            )? == *expected,
            "preparation source changed during family refresh"
        );
    }
    write_new(
        &directory.join("state.json"),
        &State {
            version: 1,
            root,
            directory: directory.clone(),
            owner_pid,
            owner_start: owner_start.into(),
            profile: first.profile.clone(),
            target: first.target.clone(),
            families,
            configuration,
            sources,
            manifest,
            lock,
            compiled: BTreeMap::new(),
        },
    )
}

fn executor_bases(
    options: &ProducerOptions,
    root: &Path,
    graph: &UnitGraph,
    packages: &BTreeMap<String, cargo_metadata::Package>,
) -> Result<BTreeMap<String, String>> {
    let keys = unit_keys(graph, root)?;
    let sources = selected_source_state(graph, packages)?;
    let mut bases = BTreeMap::new();
    let mut native_options = options.clone();
    native_options.package.clear();
    native_options.production_target = ProducerTarget::Library;
    native_options.features.clear();
    native_options.no_default_features = false;
    let native_basis = (
        native_review_basis(&native_options, root)?,
        options.declarations.native_inputs_complete,
        options.declarations.native_environment.clone(),
    );
    // The source-bound assertion belongs to this role's eligibility review, not
    // to the shared executor's premises. Each role still validates its original
    // assertion in capture(); retain all actual native facts above conservatively.
    let caller_environment = native_caller_environment(options)?;
    let catalogs = options
        .declarations
        .review_catalogs
        .iter()
        .map(|path| Ok((path.clone(), bytes_hash(&absolute(root, path))?)))
        .collect::<Result<Vec<_>>>()?;
    for index in selected_indices(graph)? {
        let unit = &graph.units[index];
        if unit.mode != "build"
            || !unit
                .target
                .kind
                .iter()
                .any(|kind| kind == "custom-build" || kind == "proc-macro")
        {
            continue;
        }
        let mut normalized = options.clone();
        normalized.package = unit.pkg_id.clone();
        normalized.production_target = ProducerTarget::Library;
        normalized.features = unit.features.clone();
        normalized.no_default_features = false;
        // Common native/search state is observed once for this capture,
        // rather than rehashing every large namespace for every executor.
        normalized.native_inputs.clear();
        normalized.declarations.native_environment.clear();
        normalized.declarations.native_namespaces.clear();
        normalized.declarations.native_absent_files.clear();
        normalized
            .declarations
            .native_absent_environment_prefixes
            .clear();
        normalized.declarations.review_catalogs.clear();
        // Review the dependency-complete executable closure, rather than the
        // unrelated role's package/target as though it were executable input.
        let mut closure = graph.clone();
        closure.roots = vec![index];
        let owners = selected_indices(&closure)?
            .into_iter()
            .map(|i| graph.units[i].pkg_id.clone())
            .collect::<BTreeSet<_>>();
        normalized.declarations.build_scripts.retain(|owner, _| {
            owners.contains(owner) || owners.iter().any(|id| logical_package(id, root) == *owner)
        });
        normalized.declarations.proc_macros.retain(|owner, _| {
            owners.contains(owner) || owners.iter().any(|id| logical_package(id, root) == *owner)
        });
        let basis = hash_parts(&[
            b"finite-preparation-executor.v2",
            &serde_json::to_vec(&(
                &keys[&index],
                executable_closure_basis(graph, index, &sources, &keys)?,
                reviewed_input_state(&normalized, root)?,
                &native_basis,
                &caller_environment,
                &catalogs,
                &normalized.declarations.configuration_executors,
                &normalized.declarations.build_scripts,
                &normalized.declarations.proc_macros,
            ))?,
        ]);
        bases.insert(keys[&index].clone(), basis);
    }
    Ok(bases)
}

fn verify_compilation(prior: &Compilation, verified: &mut BTreeSet<PathBuf>) -> Result<()> {
    ensure!(
        bytes_hash(&prior.evidence)? == prior.evidence_hash,
        "preceding compilation evidence changed"
    );
    for output in &prior.outputs {
        output.verify()?;
    }
    if verified.insert(prior.premises.clone()) {
        ensure!(
            bytes_hash(&prior.premises)? == prior.premises_hash,
            "preparation consumed premises changed"
        );
        let association: deployment_identity::DeploymentAssociation =
            serde_json::from_slice(&fs::read(&prior.premises)?)?;
        association.verify_current(&association.artifact.path)?;
    }
    Ok(())
}

pub(super) struct Capture {
    directory: PathBuf,
    state: State,
    pending: BTreeMap<String, Compilation>,
    bases: BTreeMap<String, String>,
    _lock: File,
    done: bool,
}

impl Capture {
    pub(super) fn enter(
        options: &ProducerOptions,
        root: &Path,
        graph: &UnitGraph,
        packages: &BTreeMap<String, cargo_metadata::Package>,
    ) -> Result<Option<Self>> {
        let Some(directory) = &options.declarations.preparation else {
            return Ok(None);
        };
        ensure!(
            options.declarations.capture_actual_build
                && options.declarations.actual_build_evidence.is_some(),
            "preparation enrollment requires actual protected build evidence"
        );
        let directory = absolute(root, directory).canonicalize()?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(directory.join("capture.lock"))?;
        lock.try_lock()
            .map_err(|error| anyhow::anyhow!("preparation capture is already active: {error}"))?;
        ensure!(
            !path_entry_exists(&directory.join("FAILED"))?
                && !path_entry_exists(&directory.join("COMPLETE"))?
                && !path_entry_exists(&directory.join("in-progress.json"))?,
            "preparation is terminal or has an incomplete capture; start a new operation"
        );
        let state: State = serde_json::from_slice(&fs::read(directory.join("state.json"))?)?;
        ensure!(
            state.version == 1 && state.directory == directory && state.root == root,
            "preparation identity differs from selected workspace"
        );
        owner_is_ancestor(state.owner_pid, &state.owner_start)?;
        require_held_guard(root)?;
        let guard: serde_json::Value = serde_json::from_slice(&fs::read(guard_path(root))?)?;
        ensure!(
            guard["directory"] == directory.to_string_lossy().as_ref()
                && guard["owner_pid"] == state.owner_pid
                && guard["owner_start"] == state.owner_start,
            "preparation target guard changed"
        );
        write_new(
            &directory.join("in-progress.json"),
            &serde_json::json!({"pid":std::process::id(),
            "package":options.package,"evidence":options.declarations.actual_build_evidence}),
        )?;
        let mut capture = Self {
            directory,
            state,
            pending: BTreeMap::new(),
            bases: BTreeMap::new(),
            _lock: lock,
            done: false,
        };
        let state = &capture.state;
        ensure!(
            state.profile == options.profile && state.target == options.target,
            "actual child selects another preparation target/profile"
        );
        ensure!(
            configuration_state(root)? == state.configuration
                && bytes_hash(&root.join("Cargo.toml"))? == state.manifest
                && bytes_hash(&root.join("Cargo.lock"))? == state.lock,
            "preparation source/configuration baseline changed"
        );
        let families = executor_refresh_families(&executor_refresh_packages(graph)?, packages)?;
        ensure!(
            families.is_subset(&state.families),
            "actual child requires an unrefreshed executor family; start a new preparation"
        );
        for (id, source) in selected_source_state(graph, packages)? {
            ensure!(
                state.sources.get(&id) == Some(&source),
                "actual child source is outside preparation baseline"
            );
        }
        // Cargo can execute a cached build script while deciding what to rebuild.
        // Reject known substituted executors and changed consumed premises before
        // giving the actual child any effects, then check them again afterward.
        let bases = executor_bases(options, root, graph, packages)
            .context("preparation premises or original outputs changed")?;
        let mut verified = BTreeSet::new();
        for (key, basis) in &bases {
            if let Some(prior) = capture.state.compiled.get(key) {
                ensure!(
                    prior.basis == *basis,
                    "preparation premises or original outputs changed"
                );
                verify_compilation(prior, &mut verified)
                    .context("preparation premises or original outputs changed")?;
            }
        }
        capture.bases = bases;
        Ok(Some(capture))
    }

    pub(super) fn validate(
        &mut self,
        options: &ProducerOptions,
        root: &Path,
        graph: &UnitGraph,
        packages: &BTreeMap<String, cargo_metadata::Package>,
        messages: &[serde_json::Value],
        identity: &ProducerIdentity,
    ) -> Result<()> {
        let keys = unit_keys(graph, root)?;
        let bases = executor_bases(options, root, graph, packages)?;
        ensure!(
            bases == self.bases,
            "preparation premises or original outputs changed during child build"
        );
        let evidence = absolute(
            root,
            options
                .declarations
                .actual_build_evidence
                .as_ref()
                .context("actual preparation evidence")?,
        );
        let premises = evidence.with_extension("preparation-premises.json");
        write_new(
            &premises,
            &deployment_association(options, root, graph, messages, identity)?,
        )?;
        let premises_hash = bytes_hash(&premises)?;
        let mut verified = BTreeSet::new();
        let mut preceding = BTreeMap::new();
        for index in selected_indices(graph)? {
            let unit = &graph.units[index];
            // run-custom-build output association is captured separately by the
            // existing owner. It is not a compiler-artifact Fresh assertion.
            if unit.mode != "build"
                || !unit
                    .target
                    .kind
                    .iter()
                    .any(|kind| kind == "custom-build" || kind == "proc-macro")
            {
                continue;
            }
            let artifacts = messages
                .iter()
                .filter(|message| artifact_matches(unit, message))
                .collect::<Vec<_>>();
            ensure!(
                artifacts.len() == 1,
                "executor lacks an unambiguous exact compilation association"
            );
            let message = artifacts[0];
            let basis = bases[&keys[&index]].clone();
            let names = message["filenames"]
                .as_array()
                .context("executor output files missing")?;
            let mut outputs = names
                .iter()
                .map(|name| {
                    deployment_identity::FileObservation::capture(&absolute(
                        root,
                        Path::new(name.as_str().context("executor output name")?),
                    ))
                    .map_err(anyhow::Error::from)
                })
                .collect::<Result<Vec<_>>>()?;
            outputs.sort();
            outputs.dedup();
            ensure!(
                !outputs.is_empty(),
                "executor compilation has no original output"
            );
            match message["fresh"].as_bool() {
                Some(false) => {
                    self.pending.insert(
                        keys[&index].clone(),
                        Compilation {
                            basis,
                            outputs,
                            evidence: evidence.clone(),
                            evidence_hash: bytes_hash(&evidence)?,
                            premises: premises.clone(),
                            premises_hash: premises_hash.clone(),
                        },
                    );
                }
                Some(true) => {
                    let prior = self.state.compiled.get(&keys[&index]).context(
                        "Fresh executor lacks a preceding compilation in this preparation",
                    )?;
                    ensure!(
                        prior.basis == basis
                            && prior.outputs == outputs
                            && bytes_hash(&prior.evidence)? == prior.evidence_hash,
                        "Fresh executor preparation premises or original outputs changed"
                    );
                    verify_compilation(prior, &mut verified)
                        .context("preparation premises or original outputs changed")?;
                    preceding.insert(prior.evidence.clone(), serde_json::json!({
                        "compilation": prior, "actual_build_evidence":serde_json::from_slice::<serde_json::Value>(&fs::read(&prior.evidence)?)?,
                        "original_premises":serde_json::from_slice::<serde_json::Value>(&fs::read(&prior.premises)?)?}));
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
        let mut output = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&evidence)?;
        serde_json::to_writer(&mut output, &context)?;
        output.flush()?;
        output.sync_all()?;
        let hash = bytes_hash(&evidence)?;
        for compiled in self.pending.values_mut() {
            compiled.evidence_hash = hash.clone();
        }
        Ok(())
    }

    pub(super) fn commit(&mut self) -> Result<()> {
        owner_is_ancestor(self.state.owner_pid, &self.state.owner_start)?;
        require_held_guard(&self.state.root)?;
        ensure!(
            !path_entry_exists(&self.directory.join("FAILED"))?,
            "preparation became terminal during capture"
        );
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
        if !self.done {
            let _ = write_new(
                &self.directory.join("FAILED"),
                &serde_json::json!({"terminal":"incomplete capture"}),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        _scratch: tempfile::TempDir,
        root: PathBuf,
        directory: PathBuf,
        evidence: PathBuf,
        options: ProducerOptions,
        _guard: File,
    }

    impl Fixture {
        fn new() -> Self {
            let scratch = tempfile::tempdir().unwrap();
            let root = scratch.path().join("model");
            let evidence = scratch.path().join("evidence");
            fs::create_dir_all(root.join("src/bin")).unwrap();
            fs::create_dir(&evidence).unwrap();
            fs::write(root.join("Cargo.toml"), "[package]\nname='cohort-fixture'\nversion='0.1.0'\nedition='2024'\n[features]\nalternate=[]\nfail=[]\n").unwrap();
            fs::write(root.join("build.rs"), "fn main(){let p=std::path::PathBuf::from(std::env::var_os(\"OUT_DIR\").unwrap());std::fs::write(p.join(\"generated.rs\"),\"pub fn value()->u32{1}\").unwrap();println!(\"cargo:rerun-if-changed=build.rs\");}").unwrap();
            fs::write(root.join("src/lib.rs"), "#[cfg(feature=\"fail\")] compile_error!(\"selected child fails\");include!(concat!(env!(\"OUT_DIR\"),\"/generated.rs\"));").unwrap();
            fs::write(
                root.join("src/bin/worker.rs"),
                "fn main(){assert_eq!(cohort_fixture::value(),1);}",
            )
            .unwrap();
            let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
            fs::copy(
                repository.join("rust-toolchain.toml"),
                root.join("rust-toolchain.toml"),
            )
            .unwrap();
            cargo_output(&root, &["generate-lockfile".into(), "--offline".into()]).unwrap();
            let directory = root.join("target/cohort");
            fs::create_dir_all(&directory).unwrap();
            let guard = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(guard_path(&root))
                .unwrap();
            guard.try_lock().unwrap();
            let pid = std::process::id();
            let start = process(pid).unwrap().1;
            serde_json::to_writer(
                &guard,
                &serde_json::json!({"directory":directory,"owner_pid":pid,"owner_start":start}),
            )
            .unwrap();
            let options = ProducerOptions {
                workspace_root: root.clone(),
                package: "cohort-fixture".into(),
                profile: "dev".into(),
                target: None,
                production_target: ProducerTarget::Library,
                features: Vec::new(),
                no_default_features: false,
                dep_info: Vec::new(),
                declared_inputs: Vec::new(),
                native_inputs: Vec::new(),
                declared_environment: BTreeMap::new(),
                declarations: InputDeclarations {
                    capture_actual_build: true,
                    refresh_executors: true,
                    ..Default::default()
                },
            };
            begin(&root, &directory, vec![options.clone()], pid, &start).unwrap();
            Self {
                _scratch: scratch,
                root,
                directory,
                evidence,
                options,
                _guard: guard,
            }
        }

        fn selection(&self, name: &str) -> ProducerOptions {
            let mut options = self.options.clone();
            options.declarations.preparation = Some(self.directory.clone());
            options.declarations.actual_build_evidence =
                Some(self.evidence.join(format!("{name}.json")));
            options
        }

        fn native_reviewed_selection(&self, name: &str, target: ProducerTarget) -> ProducerOptions {
            let mut options = self.selection(name);
            options.production_target = target;
            let compiler = Command::new("rustc")
                .current_dir(&self.root)
                .args(["--print", "sysroot"])
                .output()
                .unwrap();
            assert!(compiler.status.success());
            options.native_inputs = vec![
                PathBuf::from(std::str::from_utf8(&compiler.stdout).unwrap().trim())
                    .join("bin/rustc"),
            ];
            // This tiny fixed owner only generates a constant Rust source file.
            // Its assertion is scoped to this fixture, never to PSE native code.
            options.declarations.native_inputs_complete = true;
            options.declarations.native_abi = Some("finite-rust-generator-compiler.v1".into());
            options.declarations.native_reviewed_source =
                Some(native_review_basis(&options, &self.root).unwrap());
            options
        }
    }

    fn assert_shared_compilation(fixture: &Fixture, name: &str) {
        let context: serde_json::Value = serde_json::from_slice(
            &fs::read(fixture.evidence.join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        assert!(
            !context["preparation"]["preceding_compilations"]
                .as_object()
                .unwrap()
                .is_empty()
        );
        assert!(
            context["preparation"]["first_compiled_contexts"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn producer_preparation_native_role_assertions_share_executor_and_retain_history() {
        let fixture = Fixture::new();
        let first = fixture.native_reviewed_selection("library", ProducerTarget::Library);
        let second =
            fixture.native_reviewed_selection("worker", ProducerTarget::Binary("worker".into()));
        assert_ne!(
            first.declarations.native_reviewed_source,
            second.declarations.native_reviewed_source
        );
        for (name, options) in [("library", &first), ("worker", &second)] {
            let identity = run(options).unwrap();
            assert!(
                !identity.reasons.iter().any(|reason| reason
                    .contains("native ABI/binary/input completeness")
                    || reason.contains("native/tool closure")),
                "{:?}",
                identity.reasons
            );
            assert_eq!(
                identity.reviewed_owner_sources["native-closure"],
                options
                    .declarations
                    .native_reviewed_source
                    .as_ref()
                    .unwrap()
                    .as_str()
            );
            identity
                .deployment
                .as_ref()
                .unwrap()
                .verify_current(&identity.deployment.as_ref().unwrap().artifact.path)
                .unwrap();
            let directory = fixture.evidence.join(format!("{name}.json.closure"));
            verify_historical_evidence(&directory).unwrap();
            let index: serde_json::Value =
                serde_json::from_slice(&fs::read(directory.join("index.json")).unwrap()).unwrap();
            assert_eq!(
                index["declarations"]["native_reviewed_source"],
                options
                    .declarations
                    .native_reviewed_source
                    .as_ref()
                    .unwrap()
                    .as_str()
            );
            assert_eq!(index["declarations"]["native_inputs_complete"], true);
        }
        assert_shared_compilation(&fixture, "worker");
        // Historical reconstruction uses sealed owned members, not the mutable
        // role evidence or the live preparation marker.
        fs::remove_file(fixture.directory.join("state.json")).unwrap();
        for name in ["library", "worker"] {
            fs::remove_file(fixture.evidence.join(format!("{name}.json"))).unwrap();
            verify_historical_evidence(&fixture.evidence.join(format!("{name}.json.closure")))
                .unwrap();
        }
    }

    #[test]
    fn producer_preparation_stale_native_assertion_reuses_but_remains_ineligible() {
        let fixture = Fixture::new();
        let first = fixture.native_reviewed_selection("library", ProducerTarget::Library);
        run(&first).unwrap();
        let mut stale =
            fixture.native_reviewed_selection("worker", ProducerTarget::Binary("worker".into()));
        stale.declarations.native_reviewed_source = first.declarations.native_reviewed_source;
        let identity = run(&stale).unwrap();
        assert!(!identity.persistent_reuse_eligible);
        assert!(
            identity
                .reasons
                .iter()
                .any(|reason| reason == "native/tool closure has no matching source-bound review")
        );
        assert_shared_compilation(&fixture, "worker");
    }

    #[test]
    fn producer_preparation_changed_native_abi_refuses_before_build() {
        let fixture = Fixture::new();
        let first = fixture.native_reviewed_selection("library", ProducerTarget::Library);
        run(&first).unwrap();
        let mut second =
            fixture.native_reviewed_selection("worker", ProducerTarget::Binary("worker".into()));
        second.declarations.native_abi = Some("changed-native-abi.v2".into());
        second.declarations.native_reviewed_source =
            Some(native_review_basis(&second, &fixture.root).unwrap());
        assert!(
            run(&second)
                .unwrap_err()
                .to_string()
                .contains("premises or original outputs changed")
        );
        assert!(fixture.directory.join("FAILED").is_file());
        assert!(!fixture.evidence.join("worker.json").exists());
        assert!(
            !fixture
                .evidence
                .join("worker.json.closure/build.stdout")
                .exists()
        );
        assert!(
            !fixture
                .evidence
                .join("worker.json.closure/build.status.json")
                .exists()
        );
    }

    #[test]
    fn producer_preparation_overlapping_roles_keep_current_inputs_and_history() {
        let fixture = Fixture::new();
        let first = run(&fixture.selection("library")).unwrap();
        assert!(!first.persistent_reuse_eligible);
        assert!(
            first
                .reasons
                .iter()
                .any(|reason| reason == "native ABI/binary/input completeness is unknown")
        );
        let mut second = fixture.selection("worker");
        second.production_target = ProducerTarget::Binary("worker".into());
        let second_identity = run(&second).unwrap();
        assert!(!second_identity.persistent_reuse_eligible);
        assert!(
            second_identity
                .reasons
                .iter()
                .any(|reason| reason == "native ABI/binary/input completeness is unknown")
        );
        let association = first.deployment.unwrap();
        association
            .verify_current(&association.artifact.path)
            .unwrap();
        let context: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture.evidence.join("worker.json")).unwrap())
                .unwrap();
        assert!(
            !context["preparation"]["preceding_compilations"]
                .as_object()
                .unwrap()
                .is_empty()
        );
        assert!(
            context["preparation"]["first_compiled_contexts"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        verify_historical_evidence(&fixture.evidence.join("library.json.closure")).unwrap();
        verify_historical_evidence(&fixture.evidence.join("worker.json.closure")).unwrap();
    }

    #[test]
    fn producer_preparation_distinct_feature_context_needs_its_own_compilation() {
        let fixture = Fixture::new();
        run(&fixture.selection("plain")).unwrap();
        let mut alternate = fixture.selection("alternate");
        alternate.features = vec!["alternate".into()];
        run(&alternate).unwrap();
        let state: State =
            serde_json::from_slice(&fs::read(fixture.directory.join("state.json")).unwrap())
                .unwrap();
        assert_eq!(state.compiled.len(), 2);
    }

    #[test]
    fn producer_preparation_prior_operation_fresh_and_failed_retry_refuse() {
        let fixture = Fixture::new();
        run(&fixture.selection("first")).unwrap();
        // Simulate a marker with no preceding admitted compilation. Cargo's old
        // Fresh output cannot manufacture this operation's missing proof.
        let path = fixture.directory.join("state.json");
        let mut state: State = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        state.compiled.clear();
        fs::write(&path, serde_json::to_vec(&state).unwrap()).unwrap();
        assert!(
            run(&fixture.selection("unproved"))
                .unwrap_err()
                .to_string()
                .contains("preceding compilation")
        );
        assert!(fixture.directory.join("FAILED").is_file());
        assert!(
            run(&fixture.selection("retry"))
                .unwrap_err()
                .to_string()
                .contains("terminal")
        );
    }

    #[test]
    fn producer_preparation_source_mutation_is_terminal_before_build() {
        let fixture = Fixture::new();
        run(&fixture.selection("first")).unwrap();
        fs::write(fixture.root.join("src/lib.rs"), "pub fn value()->u32{2}").unwrap();
        assert!(
            run(&fixture.selection("changed"))
                .unwrap_err()
                .to_string()
                .contains("source is outside")
        );
        assert!(fixture.directory.join("FAILED").is_file());
        assert!(!fixture.evidence.join("changed.json").exists());
    }

    #[test]
    fn producer_preparation_changed_native_input_cannot_reuse_fresh_executor() {
        let fixture = Fixture::new();
        let native = fixture.evidence.join("native-input");
        fs::write(&native, b"first native selection").unwrap();
        let mut first = fixture.selection("first");
        first.native_inputs.push(native.clone());
        run(&first).unwrap();
        fs::write(&native, b"different native selection").unwrap();
        let mut second = fixture.selection("changed-native");
        second.native_inputs.push(native);
        assert!(
            run(&second)
                .unwrap_err()
                .to_string()
                .contains("premises or original outputs changed")
        );
        assert!(fixture.directory.join("FAILED").is_file());
        assert!(!fixture.evidence.join("changed-native.json").exists());
        assert!(
            !fixture
                .evidence
                .join("changed-native.json.closure/build.status.json")
                .exists()
        );
        assert!(
            !fixture
                .evidence
                .join("changed-native.json.closure/build.stdout")
                .exists()
        );
    }

    #[test]
    fn producer_preparation_changed_absence_cannot_reuse_fresh_executor() {
        let fixture = Fixture::new();
        let absent = fixture.evidence.join("previously-absent");
        let mut first = fixture.selection("first");
        first.declarations.native_absent_files.push(absent.clone());
        run(&first).unwrap();
        fs::write(&absent, b"now present").unwrap();
        let mut second = fixture.selection("changed-absence");
        second.declarations.native_absent_files.push(absent);
        assert!(
            run(&second)
                .unwrap_err()
                .to_string()
                .contains("premises or original outputs changed")
        );
        assert!(fixture.directory.join("FAILED").is_file());
        assert!(!fixture.evidence.join("changed-absence.json").exists());
        assert!(
            !fixture
                .evidence
                .join("changed-absence.json.closure/build.status.json")
                .exists()
        );
        assert!(
            !fixture
                .evidence
                .join("changed-absence.json.closure/build.stdout")
                .exists()
        );
    }

    #[test]
    fn producer_preparation_changed_original_executor_output_refuses() {
        let fixture = Fixture::new();
        run(&fixture.selection("first")).unwrap();
        let state: State =
            serde_json::from_slice(&fs::read(fixture.directory.join("state.json")).unwrap())
                .unwrap();
        let compiled = state.compiled.values().next().unwrap();
        fs::write(&compiled.outputs[0].path, b"substituted executor bytes").unwrap();
        assert!(
            run(&fixture.selection("changed-output"))
                .unwrap_err()
                .to_string()
                .contains("premises or original outputs changed")
        );
        assert!(fixture.directory.join("FAILED").is_file());
        assert!(!fixture.evidence.join("changed-output.json").exists());
        assert!(
            !fixture
                .evidence
                .join("changed-output.json.closure/build.status.json")
                .exists()
        );
        assert!(
            !fixture
                .evidence
                .join("changed-output.json.closure/build.stdout")
                .exists()
        );
    }

    #[test]
    fn producer_preparation_unenrolled_family_refuses_before_child_build() {
        let fixture = Fixture::new();
        let path = fixture.directory.join("state.json");
        let mut state: State = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        state.families.clear();
        fs::write(path, serde_json::to_vec(&state).unwrap()).unwrap();
        assert!(
            run(&fixture.selection("unknown-family"))
                .unwrap_err()
                .to_string()
                .contains("unrefreshed executor family")
        );
        assert!(!fixture.evidence.join("unknown-family.json").exists());
        assert!(fixture.directory.join("FAILED").is_file());
    }

    #[test]
    fn producer_preparation_failed_child_cannot_enroll_or_resume() {
        let fixture = Fixture::new();
        run(&fixture.selection("first")).unwrap();
        let before = fs::read(fixture.directory.join("state.json")).unwrap();
        let mut failed = fixture.selection("failed-child");
        failed.features = vec!["fail".into()];
        assert!(
            run(&failed)
                .unwrap_err()
                .to_string()
                .contains("Cargo command failed")
        );
        assert_eq!(
            fs::read(fixture.directory.join("state.json")).unwrap(),
            before
        );
        assert!(fixture.directory.join("FAILED").is_file());
        assert!(
            run(&fixture.selection("retry"))
                .unwrap_err()
                .to_string()
                .contains("terminal")
        );
    }
}
