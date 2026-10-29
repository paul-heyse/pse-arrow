// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit immutable native providers; selection never resolves source or constructs tools.
use super::*;
use std::collections::BTreeSet;

const SOLVER: &[&str] = &[
    "include/coin-or/IpIpoptApplication.hpp",
    "include/coin-or/IpStdCInterface.h",
    "include/coin-or/IpLinearSolvers.h",
    "include/coin-or/IpTypes.h",
    "include/coin-or/IpoptConfig.h",
    "include/coin-or/IpReturnCodes.h",
    "include/coin-or/IpReturnCodes_inc.h",
    "lib/libipopt.so",
    "lib/libcoinmumps.so.3",
    "lib/libmetis.so",
    "include/spral_ssids.h",
    "lib/libspral.so",
    "include/mkl.h",
    "lib/libmkl_intel_lp64.so.3",
    "lib/libmkl_gnu_thread.so.3",
    "lib/libmkl_core.so.3",
    "lib/pkgconfig/mkl-dynamic-lp64-gomp.pc",
    "include/scip/scip.h",
    "include/scip/scipdefplugins.h",
    "include/scip/config.h",
    "lib/libscip.so",
    "share/pse-solvers/build-info.txt",
];
const ISOLATION: &[&str] = &[
    "include/ibex.h",
    "lib/libibex.a",
    "lib/ibex/3rd/libprim.a",
    "lib/ibex/3rd/libsoplex.a",
    "share/pkgconfig/ibex.pc",
];
const UNO: &[&str] = &["lib/libuno.a", "include/uno/Uno_C_API.h"];
const PETSC: &[&str] = &[
    "lib/libpetsc.so",
    "include/petscsnes.h",
    "include/petscts.h",
];
const ORDER: &[&str] = &["compiler", "solver", "klu", "isolation", "uno", "petsc"];
fn required(kind: &str) -> &'static [&'static str] {
    match kind {
        "compiler" => &["compiler.json"],
        "solver" => SOLVER,
        "isolation" => ISOLATION,
        "uno" => UNO,
        "petsc" => PETSC,
        _ => &[],
    }
}
pub(super) fn capabilities(value: &str) -> Result<BTreeSet<&str>, HostError> {
    if value.is_empty() {
        return Ok(BTreeSet::new());
    }
    let requested: BTreeSet<_> = value.split(',').collect();
    if requested.is_empty() || requested.iter().any(|name| !ORDER.contains(name)) {
        return Err(invalid(
            "unknown native capability; expected compiler,solver,klu,isolation,uno,petsc",
        ));
    }
    let mut result = requested;
    if result
        .iter()
        .any(|name| matches!(*name, "klu" | "isolation" | "uno" | "petsc"))
    {
        result.insert("compiler");
    }
    if result.contains("uno") || result.contains("petsc") {
        result.insert("solver");
    }
    Ok(result)
}
/// Check the native capability vocabulary without reading sources or cache state.
pub fn validate_capabilities(value: &str) -> Result<(), HostError> {
    capabilities(value).map(|_| ())
}
fn directory(env: &Environment, kind: &str) -> Result<PathBuf, HostError> {
    Ok(profile_root(env)?
        .parent()
        .ok_or_else(|| invalid("invalid native root"))?
        .join(kind))
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProviderIdentity {
    kind: String,
    pin: Option<serde_json::Value>,
    inputs: BTreeMap<String, Option<String>>,
    tools: BTreeMap<String, Tool>,
    selections: BTreeMap<String, String>,
    /// Content identities, pins and build options captured only at explicit preparation.
    construction: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderReceipt {
    version: u32,
    identity: ProviderIdentity,
    files: BTreeMap<String, String>,
}
fn validate_provider(
    prefix: &Path,
    wanted: Option<&ProviderIdentity>,
) -> Result<ProviderReceipt, HostError> {
    for ancestor in prefix.ancestors().take(5) {
        if fs::symlink_metadata(ancestor)?.file_type().is_symlink() {
            return Err(invalid("provider path traverses a symlink"));
        }
    }
    let receipt: ProviderReceipt = read_json(&prefix.join(".complete.json"))?;
    if receipt.version != 1
        || required(&receipt.identity.kind).is_empty()
        || wanted.is_some_and(|identity| identity != &receipt.identity)
        || required(&receipt.identity.kind)
            .iter()
            .any(|name| !receipt.files.contains_key(*name))
        || receipt.files != file_map(prefix)?
    {
        return Err(invalid(
            "native provider identity or bytes differ; run explicit preparation",
        ));
    }
    Ok(receipt)
}
fn selected(env: &Environment, kind: &str) -> Result<(PathBuf, ProviderReceipt), HostError> {
    let base = directory(env, kind)?;
    let pointer: Pointer = read_json(&base.join("current.json")).map_err(|_| invalid(format!("no completed {kind} provider; run scripts/pse-env -- scripts/pse-host native prepare --capabilities {kind}")))?;
    if pointer.version != 1
        || !component(&pointer.recipe, 64)
        || !component(&pointer.generation, 32)
    {
        return Err(invalid("invalid native provider pointer"));
    }
    let prefix = base
        .join("generations")
        .join(&pointer.recipe)
        .join(pointer.generation);
    immutable(&prefix)?;
    let receipt = validate_provider(&prefix, None)?;
    if receipt.identity.kind != kind || hash_bytes(&json(&receipt.identity)?) != pointer.recipe {
        return Err(invalid("native provider recipe differs from pointer"));
    }
    Ok((prefix, receipt))
}
fn apply_inputs(env: &Environment, identity: &ProviderIdentity) -> Result<Environment, HostError> {
    for (key, expected) in &identity.inputs {
        let actual = text(env, key)?;
        if actual != expected.as_deref()
            && !identity
                .selections
                .get(key)
                .is_some_and(|value| actual == Some(value.as_str()))
        {
            return Err(invalid(format!("{key} changed; run explicit preparation")));
        }
    }
    for tool in identity.tools.values() {
        if digest(&tool.path)? != tool.sha256 {
            return Err(invalid(
                "native provider tool changed; run explicit preparation",
            ));
        }
    }
    let mut result = env.clone();
    for (key, value) in &identity.selections {
        result.insert(key.into(), value.into());
    }
    Ok(result)
}
fn enforce(env: &mut Environment, key: &str, value: &Path) -> Result<(), HostError> {
    if env
        .get(OsStr::new(key))
        .is_some_and(|actual| Path::new(actual) != value)
    {
        return Err(invalid(format!(
            "{key} conflicts with selected native provider"
        )));
    }
    env.insert(key.into(), value.as_os_str().into());
    Ok(())
}
fn libraries(env: &mut Environment, prefix: &Path) -> Result<(), HostError> {
    let first = prefix.join("lib");
    let mut paths = vec![first.clone()];
    if let Some(existing) = env.get(OsStr::new("LD_LIBRARY_PATH")) {
        paths.extend(std::env::split_paths(existing).filter(|path| *path != first));
    }
    let value =
        std::env::join_paths(paths).map_err(|_| invalid("invalid provider library path"))?;
    env.insert("LD_LIBRARY_PATH".into(), value);
    Ok(())
}
fn apply_provider(root: &Path, env: &Environment, kind: &str) -> Result<Environment, HostError> {
    let (prefix, receipt) = selected(env, kind)?;
    let mut inputs = env.clone();
    for key in provider_variables(kind) {
        if inputs
            .get(OsStr::new(key))
            .is_some_and(|value| Path::new(value) == prefix)
            && let Some(expected) = receipt.identity.inputs.get(*key)
        {
            if let Some(expected) = expected {
                inputs.insert((*key).into(), expected.into());
            } else {
                inputs.remove(OsStr::new(key));
            }
        }
    }
    if receipt.identity.construction.contains_key("external") {
        // Own-prefix paths identify construction inputs, not a live dependency of
        // the completed snapshot. Normalize an omitted path for validation only;
        // the launch environment below receives the authenticated generation.
        // Other recorded inputs, including tools and source-built dependencies,
        // remain subject to the ordinary compatibility checks.
        for key in provider_variables(kind) {
            if !inputs.contains_key(OsStr::new(key))
                && let Some(Some(expected)) = receipt.identity.inputs.get(*key)
            {
                inputs.insert((*key).into(), expected.into());
            }
        }
    }
    let mut result = apply_inputs(&inputs, &receipt.identity)?;
    // Explicit preparation can admit an external provider by snapshotting it. Its original
    // selection is accepted as an input, but execution carries the immutable snapshot.
    if receipt.identity.construction.contains_key("external") {
        for key in provider_variables(kind) {
            if receipt.identity.inputs.get(*key).and_then(Option::as_deref) == text(&result, key)? {
                result.remove(OsStr::new(key));
            }
        }
    }
    match kind {
        "compiler" => {
            cache_bridge(root, &mut result);
        }
        "solver" => {
            enforce(&mut result, "IPOPT_DIR", &prefix)?;
            enforce(&mut result, "SCIPOPTDIR", &prefix)?;
            libraries(&mut result, &prefix)?;
            let off: BTreeSet<_> = text(&result, "PSE_NATIVE_OFF")?
                .unwrap_or("")
                .split(',')
                .map(str::to_owned)
                .collect();
            for (key, value) in &receipt.identity.construction {
                if let Some(name) = key.strip_prefix("runtime:") {
                    if matches!(name, "MKL_CBWR" | "MKL_DYNAMIC" | "OMP_CANCELLATION") {
                        result.insert(name.into(), value.into());
                    } else if !off.contains(name) {
                        result.entry(name.into()).or_insert_with(|| value.into());
                    }
                }
            }
        }
        "isolation" => enforce(&mut result, "PSE_ROOT_ISOLATION_DIR", &prefix)?,
        "uno" => {
            enforce(&mut result, "UNO_DIR", &prefix)?;
            libraries(&mut result, &prefix)?;
        }
        "petsc" => {
            enforce(&mut result, "PETSC_DIR", &prefix)?;
            libraries(&mut result, &prefix)?;
        }
        _ => return Err(invalid("unknown provider")),
    }
    Ok(result)
}
/// Select the completed providers needed by a native workload without source discovery or builds.
pub fn select_capabilities(
    root: &Path,
    env: &Environment,
    requested: &str,
) -> Result<Environment, HostError> {
    let requested = capabilities(requested)?;
    if !requested.is_empty() {
        preflight(env)?;
    }
    let mut result = env.clone();
    for kind in ORDER.iter().filter(|kind| requested.contains(**kind)) {
        // The established combined KLU generation also supplies compiler setup.
        if *kind == "compiler" && requested.contains("klu") {
            continue;
        }
        result = if *kind == "klu" {
            select(root, &result)?
        } else {
            apply_provider(root, &result, kind)?
        };
    }
    runtime(&mut result)?;
    Ok(result)
}
fn runtime(env: &mut Environment) -> Result<(), HostError> {
    // Solver defaults are captured from the single Dockerfile declaration by preparation.
    let mut off: BTreeSet<String> = text(env, "PSE_NATIVE_OFF")?
        .unwrap_or("")
        .split(',')
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect();
    for name in [
        "OMP_NUM_THREADS",
        "OPENBLAS_NUM_THREADS",
        "MKL_NUM_THREADS",
        "OMP_PROC_BIND",
        "OMP_PLACES",
        "HWLOC_COMPONENTS",
    ] {
        if text(env, name)? == Some("off") {
            off.insert(name.into());
        } else if env.contains_key(OsStr::new(name)) {
            off.remove(name);
        }
    }
    for name in ["OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"] {
        if !off.contains(name) {
            env.entry(name.into()).or_insert_with(|| "1".into());
        }
    }
    for name in &off {
        env.remove(OsStr::new(name));
    }
    if !off.is_empty() {
        env.insert(
            "PSE_NATIVE_OFF".into(),
            off.into_iter().collect::<Vec<_>>().join(",").into(),
        );
    } else {
        env.remove(OsStr::new("PSE_NATIVE_OFF"));
    }
    env.remove(OsStr::new("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"));
    Ok(())
}
fn commit_provider<F>(
    env: &Environment,
    identity: ProviderIdentity,
    build: F,
) -> Result<PathBuf, HostError>
where
    F: FnOnce(&Path, &Path, &Path) -> Result<(), HostError>,
{
    let profile = directory(env, &identity.kind)?;
    private_directory(&profile)?;
    let recipe = hash_bytes(&json(&identity)?);
    let generations = profile.join("generations").join(&recipe);
    private_directory(&generations)?;
    let locks = profile.join("locks");
    private_directory(&locks)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(locks.join(&recipe))?;
    lock.lock()?;
    for entry in fs::read_dir(&generations)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| component(name, 32))
            && immutable(&entry.path()).is_ok()
            && validate_provider(&entry.path(), Some(&identity)).is_ok()
        {
            let generation = entry.file_name();
            publish_pointer(
                &profile,
                &recipe,
                generation
                    .to_str()
                    .ok_or_else(|| invalid("invalid generation"))?,
            )?;
            return Ok(entry.path());
        }
    }
    let generation = uuid::Uuid::now_v7().simple().to_string();
    let destination = generations.join(&generation);
    let staging = profile.join("staging");
    private_directory(&staging)?;
    let scratch = tempfile::Builder::new()
        .prefix("prepare-")
        .tempdir_in(staging)?;
    let install = scratch.path().join("install");
    fs::create_dir(&install)?;
    build(&install, &destination, scratch.path())?;
    materialize_links(&install, &install)?;
    let receipt = ProviderReceipt {
        version: 1,
        identity,
        files: file_map(&install)?,
    };
    fs::write(install.join(".complete.json"), json(&receipt)?)?;
    validate_provider(&install, Some(&receipt.identity))?;
    seal(&install)?;
    fs::set_permissions(&install, fs::Permissions::from_mode(0o700))?;
    fs::rename(&install, &destination)?;
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o500))?;
    immutable(&destination)?;
    publish_pointer(&profile, &recipe, &generation)?;
    Ok(destination)
}
fn materialize_links(root: &Path, directory: &Path) -> Result<(), HostError> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            let target = path.canonicalize()?;
            if !target.starts_with(root) || !target.is_file() {
                return Err(invalid(
                    "provider symlink escapes installation or names a directory",
                ));
            }
            let bytes = fs::read(&target)?;
            let permissions = target.metadata()?.permissions();
            fs::remove_file(&path)?;
            fs::write(&path, bytes)?;
            fs::set_permissions(&path, permissions)?;
        } else if metadata.is_dir() {
            materialize_links(root, &path)?;
        } else if !metadata.is_file() {
            return Err(invalid("provider contains a special file"));
        }
    }
    Ok(())
}
fn identity(root: &Path, env: &Environment, kind: &str) -> Result<ProviderIdentity, HostError> {
    let (mut tools, selections) = compiler(root, env)?;
    let mut effective = env.clone();
    for (key, value) in &selections {
        effective.insert(key.into(), value.into());
    }
    cache_bridge(root, &mut effective);
    if let Some(path) = effective.get(OsStr::new("CMAKE_TOOLCHAIN_FILE")) {
        let path = PathBuf::from(path);
        tools.insert(
            "effective-cmake-toolchain".into(),
            Tool {
                sha256: digest(&path)?,
                path,
                version: String::new(),
            },
        );
    }
    let mut selections = selections;
    if let Some(path) = effective.get(OsStr::new("CMAKE_TOOLCHAIN_FILE")) {
        selections.insert(
            "CMAKE_TOOLCHAIN_FILE".into(),
            path.to_string_lossy().into_owned(),
        );
    }
    Ok(ProviderIdentity {
        kind: kind.into(),
        pin: None,
        inputs: inputs(env)?,
        tools,
        selections,
        construction: BTreeMap::new(),
    })
}
/// Explicitly prepare all requested immutable providers, retaining previous generations on failure.
pub fn prepare_capabilities(
    root: &Path,
    env: &Environment,
    requested: &str,
) -> Result<BTreeMap<String, PathBuf>, HostError> {
    let requested = capabilities(requested)?;
    if !requested.is_empty() {
        preflight(env)?;
    }
    let mut effective = env.clone();
    let mut result = BTreeMap::new();
    for kind in ORDER.iter().filter(|kind| requested.contains(**kind)) {
        if *kind == "compiler" && requested.contains("klu") {
            continue;
        }
        let prefix = match *kind {
            "klu" => prepare(root, &effective)?,
            "compiler" => {
                if let Ok((prefix, receipt)) = selected(&effective, "compiler")
                    && apply_inputs(&effective, &receipt.identity).is_ok()
                {
                    effective = apply_provider(root, &effective, "compiler")?;
                    result.insert("compiler".into(), prefix);
                    continue;
                }
                let identity = identity(root, &effective, "compiler")?;
                let bytes = json(&identity.selections)?;
                commit_provider(&effective, identity, |install, _, _| {
                    fs::write(install.join("compiler.json"), bytes)?;
                    Ok(())
                })?
            }
            "solver" => prepare_solver(root, &effective)?,
            _ => prepare_source(root, &effective, kind)?,
        };
        effective = if *kind == "klu" {
            select(root, &effective)?
        } else {
            apply_provider(root, &effective, kind)?
        };
        result.insert((*kind).into(), prefix);
    }
    Ok(result)
}
fn immutable_image(image: &str) -> bool {
    let digest = image
        .strip_prefix("sha256:")
        .or_else(|| image.rsplit_once("@sha256:").map(|(_, hash)| hash));
    !image.chars().any(char::is_whitespace) && digest.is_some_and(|hash| component(hash, 64))
}
fn docker_runtime(root: &Path) -> Result<BTreeMap<String, String>, HostError> {
    let source = fs::read_to_string(root.join("docker/solvers/Dockerfile"))?;
    let mut joined = String::new();
    let mut selected = false;
    let mut runtime = BTreeMap::new();
    for raw in source.lines() {
        let raw = raw.trim();
        if raw.starts_with('#') {
            continue;
        }
        joined.push_str(raw.trim_end_matches('\\'));
        joined.push(' ');
        if raw.ends_with('\\') {
            continue;
        }
        let words: Vec<_> = joined.split_whitespace().collect();
        if words
            .first()
            .is_some_and(|word| word.eq_ignore_ascii_case("FROM"))
        {
            selected =
                words.len() >= 4 && words[2].eq_ignore_ascii_case("AS") && words[3] == "solvers";
        } else if selected
            && words
                .first()
                .is_some_and(|word| word.eq_ignore_ascii_case("ENV"))
        {
            for word in &words[1..] {
                let (key, value) = word
                    .split_once('=')
                    .ok_or_else(|| invalid("unsupported solver ENV declaration"))?;
                if key != "PATH" && !value.contains("/opt/pse-solvers") {
                    runtime.insert(key.into(), value.into());
                }
            }
        }
        joined.clear();
    }
    if runtime.is_empty() {
        return Err(invalid("solver Dockerfile has no runtime environment"));
    }
    Ok(runtime)
}
fn prepare_solver(root: &Path, env: &Environment) -> Result<PathBuf, HostError> {
    if env.contains_key(OsStr::new("IPOPT_DIR")) {
        return prepare_external(root, env, "solver");
    }
    let pins: BTreeMap<String, String> = read_json(&root.join(".github/setup/solver-images.json"))?;
    let image = text(env, "PSE_SOLVER_IMAGE")?
        .filter(|image| !image.is_empty())
        .or_else(|| pins.get("dev").map(String::as_str))
        .ok_or_else(|| invalid("missing solver image pin"))?
        .to_owned();
    if !immutable_image(&image) {
        return Err(invalid(
            "PSE_SOLVER_IMAGE must be an immutable sha256 image ID or digest reference",
        ));
    }
    let mut identity = ProviderIdentity {
        pin: None,
        kind: "solver".into(),
        inputs: BTreeMap::from([(
            "PSE_SOLVER_IMAGE".into(),
            text(env, "PSE_SOLVER_IMAGE")?.map(str::to_owned),
        )]),
        tools: BTreeMap::new(),
        selections: BTreeMap::new(),
        construction: BTreeMap::from([("image".into(), image.clone())]),
    };
    for (name, value) in docker_runtime(root)? {
        identity
            .construction
            .insert(format!("runtime:{name}"), value);
    }
    let docker = resolve_program(OsStr::new("docker"), env)?;
    let tar = resolve_program(OsStr::new("tar"), env)?;
    commit_provider(env, identity, |install, _, _| {
        let mut producer = Command::new(&docker)
            .args([
                "run",
                "--rm",
                "--entrypoint",
                "tar",
                &image,
                "-C",
                "/opt/pse-solvers",
                "-cf",
                "-",
                ".",
            ])
            .env_clear()
            .envs(env)
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        let stdout = producer
            .stdout
            .take()
            .ok_or_else(|| invalid("solver extraction pipe unavailable"))?;
        let extraction = Command::new(&tar)
            .args([
                OsStr::new("-xf"),
                OsStr::new("-"),
                OsStr::new("-C"),
                install.as_os_str(),
            ])
            .env_clear()
            .envs(env)
            .stdin(stdout)
            .status();
        if !matches!(&extraction,Ok(status) if status.success()) {
            let _ = producer.kill();
        }
        let produced = producer.wait()?;
        if !extraction?.success() || !produced.success() {
            return Err(invalid("solver image extraction failed"));
        }
        Ok(())
    })
}
#[derive(Deserialize, Serialize)]
struct Pin {
    archive: String,
    sha256: String,
    options: Vec<String>,
    patch: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, toml::Value>,
}
fn pin(root: &Path, kind: &str) -> Result<Pin, HostError> {
    let manifest: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)
        .map_err(|_| invalid("invalid workspace manifest"))?;
    manifest
        .get("workspace")
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get("pse"))
        .and_then(|value| {
            value.get(if kind == "isolation" {
                "root-isolation"
            } else {
                kind
            })
        })
        .ok_or_else(|| invalid("missing provider source pin"))?
        .clone()
        .try_into()
        .map_err(|_| invalid("invalid provider source pin"))
}
fn jobs(env: &Environment) -> Result<String, HostError> {
    let value = text(env, "CARGO_BUILD_JOBS")?
        .ok_or_else(|| invalid("native preparation needs selected CARGO_BUILD_JOBS"))?;
    if value.parse::<usize>().ok().is_none_or(|value| value == 0) {
        return Err(invalid("native compiler width must be a positive integer"));
    }
    Ok(value.into())
}
fn run_at(
    program: &Path,
    args: &[OsString],
    env: &Environment,
    cwd: &Path,
) -> Result<(), HostError> {
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(env)
        .status()?;
    if !status.success() {
        return Err(invalid(format!(
            "native construction {} failed with {status}",
            program.display()
        )));
    }
    Ok(())
}
fn source_archive(env: &Environment, pin: &Pin) -> Result<PathBuf, HostError> {
    if !component(&pin.sha256, 64) || !pin.archive.starts_with("https://") {
        return Err(invalid("native source needs HTTPS and a SHA256 pin"));
    }
    let sources = directory(env, "sources")?;
    private_directory(&sources)?;
    let destination = sources.join(format!("{}.tar.gz", pin.sha256));
    if destination.is_file() && digest(&destination)? == pin.sha256 {
        return Ok(destination);
    }
    let download = tempfile::NamedTempFile::new_in(&sources)?;
    let curl = resolve_program(OsStr::new("curl"), env)?;
    run(
        &curl,
        &[
            "--fail".into(),
            "--location".into(),
            "--proto".into(),
            "-all,https".into(),
            "--proto-redir".into(),
            "-all,https".into(),
            "--max-time".into(),
            "120".into(),
            "--output".into(),
            download.path().as_os_str().into(),
            pin.archive.clone().into(),
        ],
        env,
    )?;
    if digest(download.path())? != pin.sha256 {
        return Err(invalid("native source archive checksum mismatch"));
    }
    download
        .persist(&destination)
        .map_err(|error| HostError::Io(error.error))?;
    Ok(destination)
}
fn high_provider(root: &Path, env: &Environment) -> Result<PathBuf, HostError> {
    let cargo = resolve_program(
        env.get(OsStr::new("PSE_CORRECTNESS_REAL_CARGO"))
            .or_else(|| env.get(OsStr::new("CARGO")))
            .map(OsString::as_os_str)
            .unwrap_or(OsStr::new("cargo")),
        env,
    )?;
    // Cargo's build-script message identifies the actual matching HiGHS unit and OUT_DIR.
    let output = Command::new(cargo)
        .args([
            "build",
            "--locked",
            "-p",
            "pse-uno-sys",
            "--features",
            "pse-uno-sys/highs-provider",
            "--message-format=json",
        ])
        .current_dir(root)
        .env_clear()
        .envs(env)
        .output()?;
    std::io::stderr().write_all(&output.stderr)?;
    let mut candidates = Vec::new();
    for line in output.stdout.split(|byte| *byte == b'\n') {
        if let Ok(message) = serde_json::from_slice::<serde_json::Value>(line) {
            if message["reason"] == "compiler-message"
                && let Some(rendered) = message["message"]["rendered"].as_str()
            {
                std::io::stderr().write_all(rendered.as_bytes())?;
            }
            if message["reason"] == "build-script-executed"
                && message["package_id"].as_str().is_some_and(|package| {
                    package
                        .rsplit('#')
                        .next()
                        .is_some_and(|name| name.starts_with("highs-sys@"))
                })
                && let Some(path) = message["out_dir"].as_str()
            {
                candidates.push(PathBuf::from(path));
            }
        }
    }
    if !output.status.success() || candidates.len() != 1 {
        return Err(invalid(
            "Cargo must supply one successful matching HiGHS build-script output",
        ));
    }
    let candidate = candidates.remove(0);
    if !candidate.join("lib/libhighs.a").is_file()
        || !candidate.join("include/highs/Highs.h").is_file()
    {
        return Err(invalid("Cargo HiGHS installation is incomplete"));
    }
    Ok(candidate)
}
fn prepare_source(root: &Path, env: &Environment, kind: &str) -> Result<PathBuf, HostError> {
    if provider_variables(kind)
        .first()
        .is_some_and(|key| env.contains_key(OsStr::new(key)))
    {
        return prepare_external(root, env, kind);
    }
    let pin = pin(root, kind)?;
    let width = jobs(env)?;
    let mut identity = identity(root, env, kind)?;
    identity.pin = Some(serde_json::to_value(&pin).map_err(|_| invalid("invalid provider pin"))?);
    let mut effective = apply_inputs(env, &identity)?;
    cache_bridge(root, &mut effective);
    let patch = pin.patch.as_ref().map(|path| root.join(path));
    if let Some(patch) = &patch {
        identity
            .construction
            .insert("patch-sha256".into(), digest(patch)?);
    }
    let mut blas = Vec::new();
    if matches!(kind, "uno" | "petsc") {
        let solver = env
            .get(OsStr::new("IPOPT_DIR"))
            .ok_or_else(|| invalid("foreign provider needs selected solver prefix"))?;
        identity.inputs.insert(
            "IPOPT_DIR".into(),
            Some(solver.to_string_lossy().into_owned()),
        );
        for name in [
            "libmkl_intel_lp64.so.3",
            "libmkl_gnu_thread.so.3",
            "libmkl_core.so.3",
        ] {
            let path = Path::new(solver).join("lib").join(name);
            identity.tools.insert(
                format!("blas:{name}"),
                Tool {
                    sha256: digest(&path)?,
                    path: path.clone(),
                    version: "real-double-lp64".into(),
                },
            );
            blas.push(path);
        }
    }
    let high = if kind == "uno" {
        Some(high_provider(root, &effective)?)
    } else {
        None
    };
    if let Some(high) = &high {
        identity.construction.insert(
            "highs-archive".into(),
            digest(&high.join("lib/libhighs.a"))?,
        );
        identity.construction.insert(
            "highs-headers".into(),
            hash_bytes(&json(&file_map(&high.join("include"))?)?),
        );
    }
    let archive = source_archive(&effective, &pin)?;
    commit_provider(env, identity, |install, destination, work| {
        let unpacked = work.join("source");
        fs::create_dir(&unpacked)?;
        let tar = resolve_program(OsStr::new("tar"), &effective)?;
        // The archive is authenticated against the accepted source pin before extraction.
        run(
            &tar,
            &[
                "-xzf".into(),
                archive.as_os_str().into(),
                "-C".into(),
                unpacked.as_os_str().into(),
                "--no-same-owner".into(),
            ],
            &effective,
        )?;
        let entries = fs::read_dir(&unpacked)?.collect::<Result<Vec<_>, _>>()?;
        if entries.len() != 1 || !entries[0].file_type()?.is_dir() {
            return Err(invalid("expected one foreign source root"));
        }
        let source = entries[0].path();
        if let Some(patch) = &patch {
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
        }
        let build = work.join("compiled");
        if kind == "petsc" {
            // This is PETSc's upstream third-party configure, never a first-party Python controller.
            let python = resolve_program(OsStr::new("python3"), &effective)?;
            let mut settings: Vec<OsString> = vec![source.join("configure").into_os_string()];
            settings.extend(pin.options.iter().map(OsString::from));
            settings.extend([
                format!("--prefix={}", destination.display()).into(),
                format!("--with-cc={}", identity_tool(&effective, "CC")?).into(),
                format!("--with-cxx={}", identity_tool(&effective, "CXX")?).into(),
                "COPTFLAGS=-O3 -ffp-contract=off -fno-fast-math".into(),
                "CXXOPTFLAGS=-O3 -ffp-contract=off -fno-fast-math".into(),
                format!(
                    "--with-blaslapack-lib=[{},-lgomp,-lpthread,-lm,-ldl]",
                    blas.iter()
                        .map(|path| path.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(",")
                )
                .into(),
            ]);
            run_at(&python, &settings, &effective, &source)?;
            let make = resolve_program(OsStr::new("make"), &effective)?;
            run_at(
                &make,
                &[format!("-j{width}").into(), "all".into()],
                &effective,
                &source,
            )?;
            let destdir = work.join("dest");
            run_at(
                &make,
                &[
                    "install".into(),
                    format!("DESTDIR={}", destdir.display()).into(),
                ],
                &effective,
                &source,
            )?;
            let staged = destdir.join(
                destination
                    .strip_prefix("/")
                    .map_err(|_| invalid("invalid absolute provider destination"))?,
            );
            for entry in fs::read_dir(staged)? {
                let entry = entry?;
                fs::rename(entry.path(), install.join(entry.file_name()))?;
            }
            verify_petsc(install, &effective, work)?;
        } else {
            let cmake = resolve_program(
                effective
                    .get(OsStr::new("CMAKE"))
                    .map(OsString::as_os_str)
                    .unwrap_or(OsStr::new("cmake")),
                &effective,
            )?;
            let mut options: Vec<OsString> = vec![
                "-S".into(),
                source.into_os_string(),
                "-B".into(),
                build.as_os_str().into(),
                format!("-DCMAKE_INSTALL_PREFIX={}", install.display()).into(),
            ];
            if kind == "isolation" {
                options.extend(["-G".into(), "Unix Makefiles".into()]);
            }
            options.extend(
                pin.options
                    .iter()
                    .map(|option| format!("-D{option}").into()),
            );
            if let Some(high) = &high {
                options.extend([
                    format!("-DHIGHS={}", high.join("lib/libhighs.a").display()).into(),
                    format!(
                        "-DHIGHS_INCLUDE_DIR={}",
                        high.join("include/highs").display()
                    )
                    .into(),
                    format!(
                        "-DBLAS_LIBRARIES={}",
                        blas.iter()
                            .map(|path| path.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(";")
                    )
                    .into(),
                    format!(
                        "-DLAPACK_LIBRARIES={}",
                        blas.iter()
                            .map(|path| path.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(";")
                    )
                    .into(),
                ]);
            }
            run(&cmake, &options, &effective)?;
            let mut options = vec![
                "--build".into(),
                build.as_os_str().into(),
                "--parallel".into(),
                width.clone().into(),
            ];
            if kind == "uno" {
                options.extend(["--target".into(), "uno_static".into()]);
            }
            run(&cmake, &options, &effective)?;
            run(
                &cmake,
                &["--install".into(), build.as_os_str().into()],
                &effective,
            )?;
            if kind == "isolation" {
                let configuration =
                    fs::read_to_string(install.join("share/ibex/cmake/ibex-config.cmake"))?;
                for (key, name) in [
                    ("IBEX_INTERVAL_LIB_VERSION", "interval-version"),
                    ("IBEX_LP_LIB_VERSION", "lp-version"),
                ] {
                    let value = pin
                        .extra
                        .get(name)
                        .and_then(toml::Value::as_str)
                        .ok_or_else(|| invalid("missing isolation dependency version"))?;
                    if !configuration.contains(&format!("set ({key} {value})")) {
                        return Err(invalid(format!(
                            "validated-root dependency mismatch: {key}"
                        )));
                    }
                }
            }
        }
        relocate_metadata(install, install, destination)?;
        Ok(())
    })
}
fn identity_tool(env: &Environment, key: &str) -> Result<String, HostError> {
    text(env, key)?
        .map(str::to_owned)
        .ok_or_else(|| invalid(format!("missing {key}")))
}
fn relocate_metadata(root: &Path, directory: &Path, destination: &Path) -> Result<(), HostError> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.is_dir() {
            relocate_metadata(root, &path, destination)?;
        } else if metadata.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "cmake" || extension == "pc")
        {
            let content = fs::read_to_string(&path)?.replace(
                root.to_string_lossy().as_ref(),
                destination.to_string_lossy().as_ref(),
            );
            fs::write(path, content)?;
        }
    }
    Ok(())
}
fn verify_petsc(install: &Path, env: &Environment, work: &Path) -> Result<(), HostError> {
    let source = work.join("petsc-string-control.c");
    fs::write(
        &source,
        r#"#include <petscsys.h>
#include <string.h>
#define CHECK(c) do { if (!(c)) return __LINE__; } while (0)
int main(void) {
 char exact[5]="a",truncated[5]="a",invalid[5]="abcd",zero[1]="";
 struct {char data[1];char guard;} one={{0},'G'};
 struct {char data[4];char guard;} boundary={{'a',0,0,0},'G'};
 if(PetscInitializeNoArguments()) return 1;
 if(PetscPushErrorHandler(PetscReturnErrorHandler,NULL)) return 2;
 CHECK(PetscStrlcat(exact,"bcd",sizeof(exact))==PETSC_SUCCESS && strcmp(exact,"abcd")==0);
 CHECK(PetscStrlcat(truncated,"bcdefgh",sizeof(truncated))==PETSC_SUCCESS && strcmp(truncated,"abcd")==0);
 CHECK(PetscStrlcat(boundary.data,"bcdefgh",sizeof(boundary.data))==PETSC_SUCCESS && strcmp(boundary.data,"abc")==0 && boundary.guard=='G');
 CHECK(PetscStrlcat(one.data,"x",sizeof(one.data))==PETSC_SUCCESS && one.data[0]==0 && one.guard=='G');
 CHECK(PetscStrlcat(zero,"x",0)==PETSC_ERR_ARG_SIZ && zero[0]==0);
 CHECK(PetscStrlcat(invalid,"x",4)==PETSC_ERR_ARG_SIZ && strcmp(invalid,"abcd")==0);
 CHECK(PetscStrlcat(invalid,"x",1)==PETSC_ERR_ARG_SIZ && strcmp(invalid,"abcd")==0);
 CHECK(PetscStrlcat(one.data,NULL,0)==PETSC_SUCCESS && one.data[0]==0);
 if(PetscPopErrorHandler()) return 3; return PetscFinalize();
}
"#,
    )?;
    let binary = work.join("petsc-string-control");
    let cc = resolve_program(
        env.get(OsStr::new("CC"))
            .map(OsString::as_os_str)
            .unwrap_or(OsStr::new("cc")),
        env,
    )?;
    run(
        &cc,
        &[
            "-std=c11".into(),
            "-O3".into(),
            "-Wall".into(),
            "-Werror=stringop-overflow".into(),
            "-I".into(),
            install.join("include").into_os_string(),
            source.into_os_string(),
            "-L".into(),
            install.join("lib").into_os_string(),
            format!("-Wl,-rpath,{}", install.join("lib").display()).into(),
            "-lpetsc".into(),
            "-o".into(),
            binary.as_os_str().into(),
        ],
        env,
    )?;
    let mut runtime = env.clone();
    libraries(&mut runtime, install)?;
    run(&binary, &[], &runtime)
}
fn provider_variables(kind: &str) -> &'static [&'static str] {
    match kind {
        "solver" => &["IPOPT_DIR", "SCIPOPTDIR"],
        "isolation" => &["PSE_ROOT_ISOLATION_DIR"],
        "uno" => &["UNO_DIR"],
        "petsc" => &["PETSC_DIR"],
        _ => &[],
    }
}
fn copy_external(root: &Path, source: &Path, destination: &Path) -> Result<(), HostError> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name() == ".complete.json" {
            continue;
        }
        let path = entry.path();
        let target = path.canonicalize()?;
        if !target.starts_with(root) {
            return Err(invalid("external provider symlink escapes its prefix"));
        }
        let metadata = fs::symlink_metadata(&path)?;
        let output = destination.join(entry.file_name());
        if metadata.is_dir() {
            fs::create_dir(&output)?;
            copy_external(root, &path, &output)?;
        } else if target.is_file() {
            fs::copy(&target, &output)?;
            // Immutable external inputs remain untouched. Only our private staged
            // copy needs owner-write permission for metadata relocation; publication
            // seals the complete destination after validation.
            let mode = output.metadata()?.permissions().mode();
            fs::set_permissions(&output, fs::Permissions::from_mode(mode | 0o200))?;
        } else {
            return Err(invalid(
                "external provider contains directory symlinks or special files",
            ));
        }
    }
    Ok(())
}
fn prepare_external(root: &Path, env: &Environment, kind: &str) -> Result<PathBuf, HostError> {
    let key = provider_variables(kind)
        .first()
        .ok_or_else(|| invalid("provider cannot be external"))?;
    let prefix = PathBuf::from(
        env.get(OsStr::new(key))
            .ok_or_else(|| invalid("missing external provider prefix"))?,
    );
    if !prefix.is_absolute() {
        return Err(invalid("external provider prefix must be absolute"));
    }
    let prefix = prefix.canonicalize()?;
    if required(kind)
        .iter()
        .any(|name| !prefix.join(name).is_file())
    {
        return Err(invalid(
            "external provider required interface is incomplete",
        ));
    }
    let mut identity = if kind == "solver" {
        ProviderIdentity {
            pin: None,
            kind: kind.into(),
            inputs: BTreeMap::new(),
            tools: BTreeMap::new(),
            selections: BTreeMap::new(),
            construction: BTreeMap::new(),
        }
    } else {
        identity(root, env, kind)?
    };
    for key in provider_variables(kind) {
        identity
            .inputs
            .insert((*key).into(), text(env, key)?.map(str::to_owned));
    }
    if matches!(kind, "uno" | "petsc") {
        let existing: serde_json::Value = read_json(&prefix.join(".complete.json"))?;
        let expected =
            serde_json::to_value(pin(root, kind)?).map_err(|_| invalid("invalid provider pin"))?;
        if existing["identity"]["pin"] != expected {
            return Err(invalid(
                "external foreign provider does not carry the selected pinned source qualification",
            ));
        }
        identity.pin = Some(expected);
    }
    // Capture all bytes into the private stage before hashing: the published recipe then
    // authenticates the exact snapshot, even if the external installation changes meanwhile.
    let staging = directory(env, "external-staging")?;
    private_directory(&staging)?;
    let scratch = tempfile::tempdir_in(staging)?;
    copy_external(&prefix, &prefix, scratch.path())?;
    identity.construction.insert(
        "external".into(),
        hash_bytes(&json(&file_map(scratch.path())?)?),
    );
    if kind == "solver" {
        for (name, value) in docker_runtime(root)? {
            identity
                .construction
                .insert(format!("runtime:{name}"), value);
        }
    }
    commit_provider(env, identity, |install, destination, _| {
        copy_external(scratch.path(), scratch.path(), install)?;
        relocate_metadata(&prefix, install, destination)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(kind: &str) -> (tempfile::TempDir, Environment, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let mut env = Environment::new();
        env.insert(
            "PSE_NATIVE_CACHE".into(),
            directory.path().join("cache").into_os_string(),
        );
        let identity = ProviderIdentity {
            pin: None,
            kind: kind.into(),
            inputs: BTreeMap::from([("CFLAGS".into(), None)]),
            tools: BTreeMap::new(),
            selections: BTreeMap::new(),
            construction: BTreeMap::new(),
        };
        let prefix = commit_provider(&env, identity, |install, _, _| {
            for name in required(kind) {
                let path = install.join(name);
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(path, b"fixture")?;
            }
            Ok(())
        })
        .unwrap();
        (directory, env, prefix)
    }
    #[test]
    fn providers_selection_is_source_free_and_reentrant() {
        let (directory, env, prefix) = fixture("isolation");
        // This root has no manifest, source, Dockerfile, or executable tools.
        let selected = apply_provider(directory.path(), &env, "isolation").unwrap();
        assert_eq!(
            Path::new(&selected[OsStr::new("PSE_ROOT_ISOLATION_DIR")]),
            prefix
        );
        assert_eq!(
            apply_provider(directory.path(), &selected, "isolation").unwrap(),
            selected
        );
        let mut changed = env.clone();
        changed.insert("CFLAGS".into(), "-changed".into());
        assert!(apply_provider(directory.path(), &changed, "isolation").is_err());
    }
    #[test]
    fn providers_failed_build_preserves_previous_selection() {
        let (_directory, env, prefix) = fixture("solver");
        let identity = ProviderIdentity {
            pin: None,
            kind: "solver".into(),
            inputs: BTreeMap::new(),
            tools: BTreeMap::new(),
            selections: BTreeMap::new(),
            construction: BTreeMap::from([("new-recipe".into(), "changed".into())]),
        };
        assert!(
            commit_provider(&env, identity, |_, _, _| Err(invalid(
                "injected build failure"
            )))
            .is_err()
        );
        assert_eq!(selected(&env, "solver").unwrap().0, prefix);
    }
    #[test]
    fn providers_corrupt_or_writable_payload_refuses_selection() {
        let (_directory, env, prefix) = fixture("uno");
        let path = prefix.join("lib/libuno.a");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(selected(&env, "uno").is_err());
        fs::write(&path, b"corrupt").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(selected(&env, "uno").is_err());
    }
    #[test]
    fn providers_internal_library_aliases_are_materialized_and_escape_refuses() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("stage");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("versioned.so"), b"library").unwrap();
        std::os::unix::fs::symlink("versioned.so", root.join("library.so")).unwrap();
        materialize_links(&root, &root).unwrap();
        assert!(
            !fs::symlink_metadata(root.join("library.so"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(root.join("library.so")).unwrap(), b"library");
        fs::write(directory.path().join("outside"), b"outside").unwrap();
        std::os::unix::fs::symlink("../outside", root.join("escape")).unwrap();
        assert!(materialize_links(&root, &root).is_err());
    }
    #[test]
    fn providers_solver_defaults_preserve_caller_thread_and_off_selections() {
        let (directory, env, _) = fixture("solver");
        let mut receipt = selected(&env, "solver").unwrap().1;
        receipt.identity.construction.extend([
            ("runtime:MKL_CBWR".into(), "COMPATIBLE".into()),
            ("runtime:OMP_NUM_THREADS".into(), "1".into()),
            ("runtime:OMP_PLACES".into(), "sockets".into()),
        ]);
        commit_provider(&env, receipt.identity, |install, _, _| {
            for name in SOLVER {
                let path = install.join(name);
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(path, b"solver")?;
            }
            Ok(())
        })
        .unwrap();
        let mut input = env.clone();
        input.insert("OMP_NUM_THREADS".into(), "8".into());
        input.insert("OMP_PLACES".into(), "off".into());
        input.insert("MKL_CBWR".into(), "wrong".into());
        let selected = select_capabilities(directory.path(), &input, "solver").unwrap();
        assert_eq!(selected[OsStr::new("OMP_NUM_THREADS")], "8");
        assert_eq!(selected[OsStr::new("MKL_CBWR")], "COMPATIBLE");
        assert!(!selected.contains_key(OsStr::new("OMP_PLACES")));
        assert_eq!(
            select_capabilities(directory.path(), &selected, "solver").unwrap(),
            selected
        );
    }
    #[test]
    fn providers_image_identity_rejects_tags_and_malformed_digests() {
        assert!(immutable_image(&format!("sha256:{}", "a".repeat(64))));
        assert!(immutable_image(&format!(
            "registry/image@sha256:{}",
            "b".repeat(64)
        )));
        for image in [
            "registry/image:latest",
            "sha256:abc",
            "sha256: ",
            "@sha256:no",
        ] {
            assert!(!immutable_image(image));
        }
    }
    #[test]
    fn providers_missing_selection_has_no_effects() {
        let directory = tempfile::tempdir().unwrap();
        let mut env = Environment::new();
        let cache = directory.path().join("missing");
        env.insert("PSE_NATIVE_CACHE".into(), cache.clone().into_os_string());
        for kind in ["compiler", "solver", "isolation", "uno", "petsc"] {
            assert!(selected(&env, kind).is_err());
        }
        assert!(!cache.exists());
    }
    #[test]
    fn providers_capability_dependencies_and_invalid_vocabulary() {
        assert_eq!(
            capabilities("petsc,uno,klu").unwrap(),
            BTreeSet::from(["compiler", "solver", "klu", "uno", "petsc"])
        );
        for value in [",", "compiler,", "ipopt", "compiler,IPOPT"] {
            assert!(validate_capabilities(value).is_err());
        }
    }
    #[test]
    fn providers_empty_profile_selects_runtime_without_provider_or_source_access() {
        let directory = tempfile::tempdir().unwrap();
        let cache = directory.path().join("missing-cache");
        let mut env = Environment::new();
        env.insert("PSE_NATIVE_CACHE".into(), cache.clone().into_os_string());
        env.insert("OMP_NUM_THREADS".into(), "off".into());
        env.insert(
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER".into(),
            "retired".into(),
        );
        let selected = select_capabilities(directory.path(), &env, "").unwrap();
        assert!(!selected.contains_key(OsStr::new("OMP_NUM_THREADS")));
        assert_eq!(selected[OsStr::new("OPENBLAS_NUM_THREADS")], "1");
        assert!(!selected.contains_key(OsStr::new("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER")));
        assert_eq!(
            select_capabilities(directory.path(), &selected, "").unwrap(),
            selected
        );
        assert!(
            prepare_capabilities(directory.path(), &env, "")
                .unwrap()
                .is_empty()
        );
        assert!(!cache.exists());
        let mut enabled = selected;
        enabled.insert("OMP_NUM_THREADS".into(), "8".into());
        let enabled = select_capabilities(directory.path(), &enabled, "").unwrap();
        assert_eq!(enabled[OsStr::new("OMP_NUM_THREADS")], "8");
        assert!(!enabled.contains_key(OsStr::new("PSE_NATIVE_OFF")));
    }
    fn executable(path: &Path, source: &str) {
        fs::write(path, source).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    fn tools(directory: &Path) -> Environment {
        let bin = directory.join("bin");
        fs::create_dir(&bin).unwrap();
        let resource = directory.join("resource");
        fs::create_dir_all(resource.join("include")).unwrap();
        fs::write(resource.join("include/stddef.h"), b"resource").unwrap();
        let lib = directory.join("libclang");
        fs::create_dir(&lib).unwrap();
        fs::write(lib.join("libclang.so"), b"libclang").unwrap();
        let script = "#!/bin/sh\ncase \"$1\" in\n --version) echo fixture-tool-1;;\n -dumpmachine) echo x86_64-linux-gnu;;\n -print-resource-dir) echo \"$TEST_RESOURCE\";;\n *) exit 2;;\nesac\n";
        for name in ["cc", "c++", "clang", "cmake"] {
            executable(&bin.join(name), script);
        }
        let mut env = Environment::new();
        env.insert(
            "PATH".into(),
            format!("{}:/usr/bin:/bin", bin.display()).into(),
        );
        env.insert("LIBCLANG_PATH".into(), lib.into_os_string());
        env.insert("TEST_RESOURCE".into(), resource.into_os_string());
        env.insert(
            "PSE_NATIVE_CACHE".into(),
            directory.join("cache").into_os_string(),
        );
        env.insert("CARGO_BUILD_JOBS".into(), "32".into());
        env
    }
    #[test]
    fn providers_compiler_preparation_probes_tools_and_preserves_reentry() {
        let directory = tempfile::tempdir().unwrap();
        let env = tools(directory.path());
        let prepared = prepare_capabilities(directory.path(), &env, "compiler").unwrap();
        let selected = select_capabilities(directory.path(), &env, "compiler").unwrap();
        assert_eq!(
            selected[OsStr::new("CC")],
            directory.path().join("bin/cc").as_os_str()
        );
        assert!(
            selected[OsStr::new("BINDGEN_EXTRA_CLANG_ARGS")]
                .to_str()
                .unwrap()
                .contains("-resource-dir=")
        );
        assert_eq!(
            prepare_capabilities(directory.path(), &selected, "compiler").unwrap(),
            prepared
        );
        // The first recipe remains valid; later preparation never mutates its payload.
        assert!(validate_provider(&prepared["compiler"], None).is_ok());
        executable(&directory.path().join("bin/cc"), "#!/bin/sh\nexit 1\n");
        assert!(select_capabilities(directory.path(), &env, "compiler").is_err());
    }
    #[test]
    fn providers_solver_extraction_runs_native_tools_and_survives_failed_update() {
        let directory = tempfile::tempdir().unwrap();
        let mut env = tools(directory.path());
        let source = directory.path().join("payload");
        fs::create_dir(&source).unwrap();
        for name in SOLVER {
            let path = source.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"solver-library").unwrap();
        }
        std::os::unix::fs::symlink("libipopt.so", source.join("lib/libipopt.so.3")).unwrap();
        executable(
            &directory.path().join("bin/docker"),
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$TEST_DOCKER_LOG\"\nif [ \"${TEST_DOCKER_FAIL:-}\" = 1 ]; then exit 41; fi\nexec /usr/bin/tar -C \"$TEST_SOLVER_SOURCE\" -cf - .\n",
        );
        env.insert("TEST_SOLVER_SOURCE".into(), source.clone().into_os_string());
        let log = directory.path().join("docker.log");
        env.insert("TEST_DOCKER_LOG".into(), log.clone().into_os_string());
        env.insert(
            "PSE_SOLVER_IMAGE".into(),
            format!("sha256:{}", "a".repeat(64)).into(),
        );
        fs::create_dir_all(directory.path().join(".github/setup")).unwrap();
        fs::write(
            directory.path().join(".github/setup/solver-images.json"),
            b"{\"dev\":\"unused\"}",
        )
        .unwrap();
        fs::create_dir_all(directory.path().join("docker/solvers")).unwrap();
        fs::write(
            directory.path().join("docker/solvers/Dockerfile"),
            "FROM base AS solvers\nENV MKL_CBWR=COMPATIBLE OMP_NUM_THREADS=1\n",
        )
        .unwrap();
        let prepared = prepare_capabilities(directory.path(), &env, "solver").unwrap();
        let original = prepared["solver"].clone();
        assert!(
            fs::read_to_string(log)
                .unwrap()
                .contains("run --rm --entrypoint tar sha256:")
        );
        assert!(
            !fs::symlink_metadata(original.join("lib/libipopt.so.3"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        env.insert("TEST_DOCKER_FAIL".into(), "1".into());
        env.insert(
            "PSE_SOLVER_IMAGE".into(),
            format!("sha256:{}", "b".repeat(64)).into(),
        );
        assert!(prepare_capabilities(directory.path(), &env, "solver").is_err());
        assert_eq!(selected(&env, "solver").unwrap().0, original);
        // Runtime selection has no dependence on source, Docker, or tool availability.
        env.insert(
            "PSE_SOLVER_IMAGE".into(),
            format!("sha256:{}", "a".repeat(64)).into(),
        );
        fs::remove_dir_all(directory.path().join("docker")).unwrap();
        fs::remove_file(directory.path().join("bin/docker")).unwrap();
        assert!(select_capabilities(directory.path(), &env, "solver").is_ok());
    }
    #[test]
    fn providers_external_solver_is_snapshotted_and_remains_selected_after_external_change() {
        let directory = tempfile::tempdir().unwrap();
        let mut env = Environment::new();
        env.insert(
            "PSE_NATIVE_CACHE".into(),
            directory.path().join("cache").into_os_string(),
        );
        let source = directory.path().join("external");
        fs::create_dir(&source).unwrap();
        for name in SOLVER {
            let path = source.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"external-library").unwrap();
        }
        env.insert("IPOPT_DIR".into(), source.clone().into_os_string());
        fs::create_dir_all(directory.path().join("docker/solvers")).unwrap();
        fs::write(
            directory.path().join("docker/solvers/Dockerfile"),
            "FROM base AS solvers\nENV MKL_CBWR=COMPATIBLE\n",
        )
        .unwrap();
        let prepared = prepare_capabilities(directory.path(), &env, "solver").unwrap();
        let prefix = &prepared["solver"];
        fs::write(source.join("lib/libipopt.so"), b"external-changed").unwrap();
        fs::remove_dir_all(directory.path().join("docker")).unwrap();
        fs::remove_dir_all(&source).unwrap();
        env.remove(OsStr::new("IPOPT_DIR"));
        env.remove(OsStr::new("SCIPOPTDIR"));
        let selected = select_capabilities(directory.path(), &env, "solver").unwrap();
        assert_eq!(selected[OsStr::new("IPOPT_DIR")], prefix.as_os_str());
        assert_eq!(
            fs::read(prefix.join("lib/libipopt.so")).unwrap(),
            b"external-library"
        );
        assert_eq!(
            select_capabilities(directory.path(), &selected, "solver").unwrap(),
            selected
        );
        let mut original = env.clone();
        original.insert("IPOPT_DIR".into(), source.into_os_string());
        assert_eq!(select_capabilities(directory.path(), &original, "solver").unwrap(), selected);
        for key in ["IPOPT_DIR", "SCIPOPTDIR"] {
            let mut conflict = env.clone();
            conflict.insert(key.into(), directory.path().join("different").into_os_string());
            assert!(select_capabilities(directory.path(), &conflict, "solver").is_err());
        }
    }
    #[test]
    fn providers_external_isolation_metadata_consumes_the_snapshot_after_source_removal() {
        let directory = tempfile::tempdir().unwrap();
        let mut env = tools(directory.path());
        let source = directory.path().join("external-isolation");
        fs::create_dir(&source).unwrap();
        for name in ISOLATION {
            let path = source.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"external-interface").unwrap();
        }
        fs::write(source.join("share/pkgconfig/ibex.pc"),format!("prefix={}\nName: ibex\nDescription: isolated snapshot\nVersion: 1.0\nCflags: -I${{prefix}}/include\nLibs: -L${{prefix}}/lib -libex\n",source.display())).unwrap();
        env.insert(
            "PSE_ROOT_ISOLATION_DIR".into(),
            source.clone().into_os_string(),
        );
        let metadata_path = source.join("share/pkgconfig/ibex.pc");
        let original_metadata = fs::read(&metadata_path).unwrap();
        fs::set_permissions(&metadata_path, fs::Permissions::from_mode(0o400)).unwrap();
        let prefix = prepare_external(directory.path(), &env, "isolation").unwrap();
        assert_eq!(fs::read(&metadata_path).unwrap(), original_metadata);
        assert_eq!(
            metadata_path.metadata().unwrap().permissions().mode() & 0o777,
            0o400
        );
        assert_eq!(
            prefix
                .join("share/pkgconfig/ibex.pc")
                .metadata()
                .unwrap()
                .permissions()
                .mode()
                & 0o222,
            0
        );
        fs::remove_dir_all(&source).unwrap();
        env.remove(OsStr::new("PSE_ROOT_ISOLATION_DIR"));
        let selected = apply_provider(directory.path(), &env, "isolation").unwrap();
        assert_eq!(
            selected[OsStr::new("PSE_ROOT_ISOLATION_DIR")],
            prefix.as_os_str()
        );
        let result = Command::new("/usr/bin/pkg-config")
            .args(["--cflags", "ibex"])
            .env_clear()
            .env("PKG_CONFIG_PATH", prefix.join("share/pkgconfig"))
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        let flags = String::from_utf8(result.stdout).unwrap();
        assert!(
            flags.contains(prefix.join("include").to_str().unwrap()),
            "{flags}"
        );
        assert!(!flags.contains(source.to_str().unwrap()), "{flags}");
        assert_eq!(apply_provider(directory.path(), &selected, "isolation").unwrap(), selected);
        let mut original = env.clone();
        original.insert("PSE_ROOT_ISOLATION_DIR".into(), source.into_os_string());
        assert_eq!(apply_provider(directory.path(), &original, "isolation").unwrap(), selected);
        let mut conflict = env.clone();
        conflict.insert("PSE_ROOT_ISOLATION_DIR".into(), directory.path().join("different").into_os_string());
        assert!(apply_provider(directory.path(), &conflict, "isolation").is_err());
        let mut changed = env.clone();
        changed.insert("CFLAGS".into(), "-changed".into());
        assert!(apply_provider(directory.path(), &changed, "isolation").is_err());
    }
    #[test]
    fn providers_isolation_build_uses_pinned_archive_options_width_and_final_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let mut env = tools(directory.path());
        let source = directory.path().join("archive-source");
        fs::create_dir_all(source.join("ibex")).unwrap();
        fs::write(source.join("ibex/CMakeLists.txt"), b"fixture").unwrap();
        let archive = directory.path().join("source.tar.gz");
        assert!(
            Command::new("/usr/bin/tar")
                .args([
                    OsStr::new("-czf"),
                    archive.as_os_str(),
                    OsStr::new("-C"),
                    source.as_os_str(),
                    OsStr::new("ibex")
                ])
                .status()
                .unwrap()
                .success()
        );
        env.insert("TEST_ARCHIVE".into(), archive.clone().into_os_string());
        let log = directory.path().join("cmake.log");
        env.insert("TEST_CMAKE_LOG".into(), log.clone().into_os_string());
        env.insert(
            "TEST_BUILD_PREFIX".into(),
            directory.path().join("prefix").into_os_string(),
        );
        executable(
            &directory.path().join("bin/curl"),
            "#!/bin/sh\nwhile [ \"$1\" != --output ]; do shift; done\ncp \"$TEST_ARCHIVE\" \"$2\"\n",
        );
        executable(
            &directory.path().join("bin/cmake"),
            r#"#!/bin/sh
if [ "$1" = --version ]; then echo cmake-fixture; exit; fi
printf '%s\n' "$*" >> "$TEST_CMAKE_LOG"
if [ "$1" = -S ]; then
 for arg in "$@"; do case "$arg" in -DCMAKE_INSTALL_PREFIX=*) printf '%s' "${arg#*=}" > "$TEST_BUILD_PREFIX";; esac; done
elif [ "$1" = --install ]; then
 prefix=$(cat "$TEST_BUILD_PREFIX")
 mkdir -p "$prefix/include" "$prefix/lib/ibex/3rd" "$prefix/share/pkgconfig" "$prefix/share/ibex/cmake"
 for path in include/ibex.h lib/libibex.a lib/ibex/3rd/libprim.a lib/ibex/3rd/libsoplex.a; do echo fixture > "$prefix/$path"; done
 printf 'prefix=%s\n' "$prefix" > "$prefix/share/pkgconfig/ibex.pc"
 printf 'set (IBEX_INTERVAL_LIB_VERSION 3.0.2.2)\nset (IBEX_LP_LIB_VERSION 4.0.2)\n' > "$prefix/share/ibex/cmake/ibex-config.cmake"
fi
"#,
        );
        fs::write(directory.path().join("Cargo.toml"),format!("[workspace.metadata.pse.root-isolation]\narchive='https://source.invalid/ibex.tar.gz'\nsha256='{}'\ninterval-version='3.0.2.2'\nlp-version='4.0.2'\noptions=['INTERVAL_LIB=filib','LP_LIB=soplex']\n",digest(&archive).unwrap())).unwrap();
        let result = prepare_capabilities(directory.path(), &env, "isolation").unwrap();
        let prefix = &result["isolation"];
        let commands = fs::read_to_string(log).unwrap();
        assert!(commands.contains("-DINTERVAL_LIB=filib"));
        assert!(commands.contains("-DLP_LIB=soplex"));
        assert!(commands.contains("--parallel 32"));
        assert_eq!(
            fs::read_to_string(prefix.join("share/pkgconfig/ibex.pc")).unwrap(),
            format!("prefix={}\n", prefix.display())
        );
        fs::remove_file(directory.path().join("Cargo.toml")).unwrap();
        assert!(select_capabilities(directory.path(), &env, "isolation").is_ok());
    }
    fn foreign_fixture(kind: &str) -> (tempfile::TempDir, Environment, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let mut env = tools(directory.path());
        let solver = directory.path().join("solver");
        fs::create_dir(&solver).unwrap();
        for name in SOLVER {
            let path = solver.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"solver").unwrap();
        }
        env.insert("IPOPT_DIR".into(), solver.into_os_string());
        fs::create_dir_all(directory.path().join("docker/solvers")).unwrap();
        fs::write(
            directory.path().join("docker/solvers/Dockerfile"),
            "FROM base AS solvers\nENV MKL_CBWR=COMPATIBLE\n",
        )
        .unwrap();
        fs::write(directory.path().join("patch"), b"fixture patch").unwrap();
        let source = directory.path().join("archive-source");
        fs::create_dir_all(source.join("foreign")).unwrap();
        fs::write(source.join("foreign/configure"), b"upstream configure").unwrap();
        let archive = directory.path().join("source.tar.gz");
        assert!(
            Command::new("/usr/bin/tar")
                .args([
                    OsStr::new("-czf"),
                    archive.as_os_str(),
                    OsStr::new("-C"),
                    source.as_os_str(),
                    OsStr::new("foreign")
                ])
                .status()
                .unwrap()
                .success()
        );
        env.insert("TEST_ARCHIVE".into(), archive.clone().into_os_string());
        let log = directory.path().join("native.log");
        env.insert("TEST_NATIVE_LOG".into(), log.clone().into_os_string());
        env.insert(
            "TEST_BUILD_PREFIX".into(),
            directory.path().join("prefix").into_os_string(),
        );
        executable(
            &directory.path().join("bin/curl"),
            "#!/bin/sh\nwhile [ \"$1\" != --output ]; do shift; done\ncp \"$TEST_ARCHIVE\" \"$2\"\n",
        );
        executable(
            &directory.path().join("bin/git"),
            "#!/bin/sh\nprintf 'git %s\\n' \"$*\" >> \"$TEST_NATIVE_LOG\"\n",
        );
        let options = if kind == "uno" {
            "['BUILD_STATIC_LIBS=ON','USE_OPENMP=OFF']"
        } else {
            "['--with-scalar-type=real','--with-precision=double','--with-64-bit-indices=0','--with-64-bit-blas-indices=0','--with-mpi=0']"
        };
        fs::write(directory.path().join("Cargo.toml"),format!("[workspace.metadata.pse.{kind}]\narchive='https://source.invalid/foreign.tar.gz'\nsha256='{}'\nrevision='fixture-revision'\nversion='3.24.0'\npatch='patch'\noptions={options}\n",digest(&archive).unwrap())).unwrap();
        (directory, env, log)
    }
    #[test]
    fn providers_external_foreign_snapshots_need_no_original_prefix_but_keep_qualification() {
        for kind in ["uno", "petsc"] {
            let (directory, mut env, _) = foreign_fixture(kind);
            let source = directory.path().join("external-qualified");
            fs::create_dir(&source).unwrap();
            for name in required(kind) {
                let path = source.join(name);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, b"qualified interface").unwrap();
            }
            let qualification = serde_json::json!({"identity":{"pin":pin(directory.path(), kind).unwrap()}});
            fs::write(source.join(".complete.json"), json(&qualification).unwrap()).unwrap();
            let key = provider_variables(kind)[0];
            env.insert(key.into(), source.clone().into_os_string());
            let prefix = prepare_external(directory.path(), &env, kind).unwrap();
            // A mismatched pinned source cannot replace this completed snapshot.
            fs::write(source.join(".complete.json"), br#"{"identity":{"pin":null}}"#).unwrap();
            assert!(prepare_external(directory.path(), &env, kind).is_err());
            assert_eq!(selected(&env, kind).unwrap().0, prefix);
            fs::remove_dir_all(&source).unwrap();
            env.remove(OsStr::new(key));
            let selected = apply_provider(directory.path(), &env, kind).unwrap();
            assert_eq!(selected[OsStr::new(key)], prefix.as_os_str());
            assert_eq!(apply_provider(directory.path(), &selected, kind).unwrap(), selected);
            let mut original = env.clone();
            original.insert(key.into(), source.into_os_string());
            assert_eq!(apply_provider(directory.path(), &original, kind).unwrap(), selected);
            let mut conflict = env.clone();
            conflict.insert(key.into(), directory.path().join("different").into_os_string());
            assert!(apply_provider(directory.path(), &conflict, kind).is_err());
            let mut changed = env.clone();
            changed.insert("CFLAGS".into(), "-changed".into());
            assert!(apply_provider(directory.path(), &changed, kind).is_err());
        }
    }
    #[test]
    fn providers_uno_uses_actual_cargo_highs_unit_and_one_lp64_provider() {
        let (directory, mut env, log) = foreign_fixture("uno");
        let high = directory.path().join("cargo-highs");
        fs::create_dir_all(high.join("lib")).unwrap();
        fs::create_dir_all(high.join("include/highs")).unwrap();
        fs::write(high.join("lib/libhighs.a"), b"highs archive").unwrap();
        fs::write(high.join("include/highs/Highs.h"), b"highs header").unwrap();
        let message = serde_json::json!({"reason":"build-script-executed","package_id":"registry+fixture#highs-sys@2.4.0","out_dir":high});
        env.insert("TEST_CARGO_MESSAGE".into(), message.to_string().into());
        executable(
            &directory.path().join("bin/cargo"),
            "#!/bin/sh\nprintf 'cargo %s\\n' \"$*\" >> \"$TEST_NATIVE_LOG\"\nprintf '%s\\n' \"$TEST_CARGO_MESSAGE\"\n",
        );
        env.insert(
            "PSE_CORRECTNESS_REAL_CARGO".into(),
            directory.path().join("bin/cargo").into_os_string(),
        );
        executable(
            &directory.path().join("bin/cmake"),
            r#"#!/bin/sh
if [ "$1" = --version ]; then echo cmake-fixture; exit; fi
printf 'cmake %s\n' "$*" >> "$TEST_NATIVE_LOG"
if [ "$1" = -S ]; then
 for arg in "$@"; do case "$arg" in -DCMAKE_INSTALL_PREFIX=*) printf '%s' "${arg#*=}" > "$TEST_BUILD_PREFIX";; esac; done
elif [ "$1" = --install ]; then
 prefix=$(cat "$TEST_BUILD_PREFIX");mkdir -p "$prefix/include/uno" "$prefix/lib"
 echo archive > "$prefix/lib/libuno.a";echo header > "$prefix/include/uno/Uno_C_API.h"
fi
"#,
        );
        let prepared = prepare_capabilities(directory.path(), &env, "uno").unwrap();
        let prefix = &prepared["uno"];
        let receipt: serde_json::Value = read_json(&prefix.join(".complete.json")).unwrap();
        assert_eq!(receipt["identity"]["pin"]["revision"], "fixture-revision");
        let commands = fs::read_to_string(log).unwrap();
        assert!(commands.contains("cargo build --locked -p pse-uno-sys --features pse-uno-sys/highs-provider --message-format=json"));
        assert!(commands.contains(&format!(
            "-DHIGHS={}",
            high.join("lib/libhighs.a").display()
        )));
        assert!(commands.contains("--target uno_static"));
        assert!(commands.contains("--parallel 32"));
        assert!(commands.contains("-DUSE_OPENMP=OFF"));
        for name in [
            "libmkl_intel_lp64.so.3",
            "libmkl_gnu_thread.so.3",
            "libmkl_core.so.3",
        ] {
            assert!(commands.contains(name));
        }
        let selected = select_capabilities(directory.path(), &env, "uno").unwrap();
        let mut missing_dependency = selected.clone();
        missing_dependency.remove(OsStr::new("IPOPT_DIR"));
        assert!(apply_provider(directory.path(), &missing_dependency, "uno").is_err());
        let mut changed_dependency = selected;
        changed_dependency.insert("IPOPT_DIR".into(), directory.path().join("different-solver").into_os_string());
        assert!(apply_provider(directory.path(), &changed_dependency, "uno").is_err());
    }
    #[test]
    fn providers_petsc_preserves_upstream_configuration_destdir_and_guard_control() {
        let (directory, env, log) = foreign_fixture("petsc");
        executable(
            &directory.path().join("bin/python3"),
            r#"#!/bin/sh
printf 'configure %s\n' "$*" >> "$TEST_NATIVE_LOG"
for arg in "$@"; do case "$arg" in --prefix=*) printf '%s' "${arg#*=}" > "$TEST_BUILD_PREFIX";; esac; done
"#,
        );
        executable(
            &directory.path().join("bin/make"),
            r#"#!/bin/sh
printf 'make %s\n' "$*" >> "$TEST_NATIVE_LOG"
if [ "$1" = install ]; then
 dest=${2#DESTDIR=};prefix=$(cat "$TEST_BUILD_PREFIX");install="$dest$prefix"
 mkdir -p "$install/include" "$install/lib";echo library > "$install/lib/libpetsc.so"
 echo header > "$install/include/petscsnes.h";echo header > "$install/include/petscts.h"
fi
"#,
        );
        executable(
            &directory.path().join("bin/cc"),
            r#"#!/bin/sh
case "$1" in --version) echo cc-fixture;; -dumpmachine) echo x86_64-linux-gnu;; *)
 printf 'guard-compile %s\n' "$*" >> "$TEST_NATIVE_LOG"
 while [ "$1" != -o ]; do shift; done
 printf '#!/bin/sh\necho guard-control-invoked >> "$TEST_NATIVE_LOG"\n' > "$2";chmod 700 "$2";;
esac
"#,
        );
        let prepared = prepare_capabilities(directory.path(), &env, "petsc").unwrap();
        let prefix = &prepared["petsc"];
        let receipt: serde_json::Value = read_json(&prefix.join(".complete.json")).unwrap();
        assert_eq!(receipt["identity"]["pin"]["version"], "3.24.0");
        let commands = fs::read_to_string(log).unwrap();
        for option in [
            "--with-scalar-type=real",
            "--with-precision=double",
            "--with-64-bit-indices=0",
            "--with-64-bit-blas-indices=0",
            "--with-mpi=0",
            "COPTFLAGS=-O3 -ffp-contract=off -fno-fast-math",
            "make -j32 all",
            "make install DESTDIR=",
            "guard-control-invoked",
        ] {
            assert!(commands.contains(option), "missing {option}: {commands}");
        }
        assert!(commands.contains(&format!("--prefix={}", prefix.display())));
        let selected = select_capabilities(directory.path(), &env, "petsc").unwrap();
        let mut missing_dependency = selected.clone();
        missing_dependency.remove(OsStr::new("IPOPT_DIR"));
        assert!(apply_provider(directory.path(), &missing_dependency, "petsc").is_err());
        let mut changed_dependency = selected;
        changed_dependency.insert("IPOPT_DIR".into(), directory.path().join("different-solver").into_os_string());
        assert!(apply_provider(directory.path(), &changed_dependency, "petsc").is_err());
    }
}
