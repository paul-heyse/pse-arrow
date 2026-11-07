// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit release of one completed checkout-local host tool generation.
use super::*;
use crate::{invocation::InvocationStore, paths::CheckoutState};
use rustix::fs::{CWD, FlockOperation, OFlags, RenameFlags, flock, renameat_with};
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub(crate) struct Retirement {
    generation: String,
    allocated_bytes: u64,
    blockers: Vec<String>,
    release: Option<String>,
    outcome: &'static str,
}
impl Retirement {
    pub(crate) fn refused(&self) -> bool {
        self.outcome == "protected"
    }
}

fn open_directory(path: &Path) -> Result<File, HostError> {
    private_directory(path, false)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as i32)
        .open(path)?;
    same_directory(path, &file)?;
    Ok(file)
}

fn same_directory(path: &Path, file: &File) -> Result<(), HostError> {
    let opened = file.metadata()?;
    let current = fs::symlink_metadata(path)?;
    if !current.is_dir() || identity(&opened) != identity(&current) {
        return Err(invalid("prepared directory changed while being used"));
    }
    Ok(())
}

fn identity(meta: &fs::Metadata) -> (u64, u64) {
    (meta.dev(), meta.ino())
}

pub(super) fn borrow(path: &Path) -> Result<File, HostError> {
    let file = open_directory(path)?;
    flock(&file, FlockOperation::NonBlockingLockShared).map_err(std::io::Error::from)?;
    same_directory(path, &file)?;
    Ok(file)
}

fn lock_exclusive(file: &File) -> Result<(), HostError> {
    flock(file, FlockOperation::NonBlockingLockExclusive).map_err(|error| {
        invalid(format!(
            "host generation has a borrower or unavailable lock: {error}"
        ))
    })
}

fn instance(file: &File) -> Result<String, HostError> {
    let meta = file.metadata()?;
    let birth = meta
        .created()?
        .duration_since(UNIX_EPOCH)
        .map_err(|_| invalid("host generation directory birth time is unavailable"))?;
    Ok(format!(
        "{:x}-{:x}-{:x}",
        meta.dev(),
        meta.ino(),
        birth.as_nanos()
    ))
}

fn state_context(root: &Path, env: &Environment) -> Result<String, HostError> {
    let selected = CheckoutState::resolve(root, env)?.root;
    let namespace = selected
        .parent()
        .ok_or_else(|| invalid("invalid invocation namespace"))?;
    // Require the same actual state namespace at preview and release. An unmounted or
    // redirected state path cannot turn an unseen invocation into a clearance.
    let directory = open_directory(namespace)?;
    let mut identity = encoded(&instance(&directory)?)?;
    identity.extend_from_slice(namespace.as_os_str().as_bytes());
    identity.push(0);
    identity.extend_from_slice(root.as_os_str().as_bytes());
    Ok(digest(&identity))
}

fn members(path: &Path) -> Result<BTreeMap<OsString, fs::Metadata>, HostError> {
    let mut found = BTreeMap::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name();
        let metadata = fs::symlink_metadata(entry.path())?;
        if !(name == MANIFEST || bootstrap::TOOLS.iter().any(|tool| name == *tool))
            || !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o022 != 0
            || metadata.nlink() != 1
        {
            return Err(invalid(
                "prepared directory contains an unknown or shared member",
            ));
        }
        found.insert(name, metadata);
    }
    Ok(found)
}

fn generation_holds(
    root: &Path,
    generation: &str,
    env: &Environment,
) -> Result<Vec<String>, HostError> {
    let mut blockers = Vec::new();
    let tools = root.join("target/host-tools");
    let selected = bootstrap::selected(root)?
        .ok_or_else(|| invalid("current host selection is unavailable"))?;
    if selected == tools.join(generation) {
        blockers.push("current selection".into());
    }
    for entry in fs::read_dir(&tools)? {
        let entry = entry?;
        if entry.file_name() == "current" || !entry.file_type()?.is_symlink() {
            continue;
        }
        let target = fs::read_link(entry.path())?;
        if target == Path::new(generation) || target == tools.join(generation) {
            blockers.push(format!("retained selector {:?}", entry.file_name()));
        }
    }
    let legacy = tools.join(".uses");
    match fs::symlink_metadata(&legacy) {
        Ok(_) => {
            private_directory(&legacy, false)?;
            if fs::read_dir(legacy)?.next().is_some() {
                blockers.push("unresolved legacy use records".into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let state = CheckoutState::resolve(root, env)?;
    blockers.extend(
        InvocationStore::retained_host_controls(&state, generation)?
            .into_iter()
            .map(|id| format!("original invocation {id}")),
    );
    Ok(blockers)
}

fn running_image(members: &BTreeMap<OsString, fs::Metadata>) -> Result<bool, HostError> {
    let images = members
        .iter()
        .filter(|(name, _)| *name != MANIFEST)
        .map(|(_, metadata)| identity(metadata))
        .collect::<BTreeSet<_>>();
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let name = entry.file_name();
        if !name.as_bytes().iter().all(u8::is_ascii_digit) {
            continue;
        }
        let process = entry.path();
        match fs::metadata(process.join("exe")) {
            Ok(meta) if images.contains(&identity(&meta)) => return Ok(true),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                // A private prepared image is reachable only by its user. Retain an
                // inaccessible same-user process if its executable name could be ours.
                match fs::metadata(&process) {
                    Ok(meta) if meta.uid() != rustix::process::geteuid().as_raw() => continue,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(error.into()),
                    Ok(_) => {}
                }
                match fs::read(process.join("comm")) {
                    Ok(comm)
                        if bootstrap::TOOLS.iter().any(|tool| {
                            let comm = comm.strip_suffix(b"\n").unwrap_or(&comm);
                            tool.as_bytes().starts_with(comm)
                        }) =>
                    {
                        return Ok(true);
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(false)
}

fn absent(path: &Path) -> Result<bool, HostError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error.into()),
    }
}

fn report(
    generation: &str,
    members: &BTreeMap<OsString, fs::Metadata>,
    blockers: Vec<String>,
    release: Option<String>,
    outcome: &'static str,
) -> Retirement {
    Retirement {
        generation: generation.into(),
        allocated_bytes: members.values().map(|meta| meta.blocks() * 512).sum(),
        blockers,
        release,
        outcome,
    }
}

pub(crate) fn retire(
    root: &Path,
    env: &Environment,
    generation: &str,
    release: Option<&str>,
) -> Result<Retirement, HostError> {
    if !component(generation) || !generation.starts_with("generation.") || generation.len() > 64 {
        return Err(invalid("expected one exact generation component"));
    }
    let root = root.canonicalize()?;
    private_directory(&root.join("target"), false)?;
    let tools = root.join("target/host-tools");
    private_directory(&tools, false)?;
    let path = tools.join(generation);
    let publication = release
        .map(|_| owned_file(&tools.join(".update.lock"), false))
        .transpose()?;
    if let Some(lock) = publication.as_ref() {
        lock_exclusive(lock)?;
    }
    let context = state_context(&root, env)?;
    let mut blockers = generation_holds(&root, generation, env)?;
    if state_context(&root, env)? != context {
        return Err(invalid(
            "invocation state namespace changed during observation",
        ));
    }
    if let Some(token) = release {
        return remove(&tools, &path, generation, token, &context, blockers);
    }
    let opened = open_directory(&path)?;
    let members = members(&path)?;
    let token = full_token(&path, &opened, &members, &context)?;
    if lock_exclusive(&opened).is_err() || running_image(&members)? {
        blockers.push("running or borrowed host image".into());
    }
    let eligible = blockers.is_empty();
    Ok(report(
        generation,
        &members,
        blockers,
        eligible.then_some(token),
        "preview",
    ))
}

fn full_token(
    path: &Path,
    opened: &File,
    members: &BTreeMap<OsString, fs::Metadata>,
    context: &str,
) -> Result<String, HostError> {
    let manifest = GenerationManifest::load(path)?;
    if manifest.binaries.len() != bootstrap::TOOLS.len()
        || !bootstrap::TOOLS
            .iter()
            .all(|tool| manifest.binaries.contains_key(*tool))
        || members.len() != bootstrap::TOOLS.len() + 1
    {
        return Err(invalid(
            "retirement requires the complete fixed host tool set",
        ));
    }
    Ok(format!(
        "{}-{}-{context}",
        instance(opened)?,
        hash_file(&path.join(MANIFEST))?
    ))
}

fn remove(
    tools: &Path,
    path: &Path,
    generation: &str,
    token: &str,
    context: &str,
    blockers: Vec<String>,
) -> Result<Retirement, HostError> {
    if !blockers.is_empty() {
        return Ok(report(
            generation,
            &BTreeMap::new(),
            blockers,
            None,
            "protected",
        ));
    }
    let coordinates = token
        .strip_suffix(context)
        .and_then(|value| value.strip_suffix('-'))
        .ok_or_else(|| invalid("release does not bind the current invocation namespace"))?;
    let (original_instance, manifest_hash) = coordinates
        .rsplit_once('-')
        .filter(|(id, hash)| {
            coordinates.len() <= 120
                && hash.len() == 64
                && hash.bytes().all(|b| b.is_ascii_hexdigit())
                && id.split('-').count() == 3
                && id
                    .split('-')
                    .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_hexdigit()))
        })
        .ok_or_else(|| invalid("invalid release selection"))?;
    let tomb = tools.join(format!(
        ".retired-{generation}-{}",
        digest(token.as_bytes())
    ));
    let original_present = !absent(path)?;
    let tomb_present = !absent(&tomb)?;
    let selected = match (original_present, tomb_present) {
        (false, false) => {
            return Ok(report(
                generation,
                &BTreeMap::new(),
                blockers,
                None,
                "already_absent",
            ));
        }
        (true, true) => {
            return Err(invalid(
                "generation path was reused during interrupted retirement",
            ));
        }
        (true, false) => path,
        (false, true) => &tomb,
    };
    let opened = open_directory(selected)?;
    lock_exclusive(&opened)?;
    if instance(&opened)? != original_instance {
        return Err(invalid(
            "release names a different host generation instance",
        ));
    }
    let remaining = members(selected)?;
    if running_image(&remaining)? {
        return Err(invalid("host generation still has a running image"));
    }
    if original_present {
        if full_token(path, &opened, &remaining, context)? != token {
            return Err(invalid("host generation differs from its preview"));
        }
        same_directory(path, &opened)?;
        renameat_with(CWD, path, CWD, &tomb, RenameFlags::NOREPLACE)
            .map_err(std::io::Error::from)?;
        if let Err(error) = same_directory(&tomb, &opened) {
            renameat_with(CWD, &tomb, CWD, path, RenameFlags::NOREPLACE)
                .map_err(std::io::Error::from)?;
            return Err(error);
        }
        File::open(tools)?.sync_all()?;
    }
    if !remaining.is_empty() {
        let manifest_path = tomb.join(MANIFEST);
        if hash_file(&manifest_path)? != manifest_hash {
            return Err(invalid("retired host manifest differs from its preview"));
        }
        let manifest: GenerationManifest = serde_json::from_reader(File::open(&manifest_path)?)
            .map_err(|_| invalid("retired host manifest is invalid"))?;
        let binaries = bootstrap::TOOLS
            .iter()
            .filter(|tool| remaining.contains_key(OsStr::new(tool)));
        for tool in binaries.clone() {
            if manifest.binaries.get(*tool) != Some(&hash_file(&tomb.join(tool))?) {
                return Err(invalid("retired host member differs from its manifest"));
            }
        }
        for tool in binaries {
            fs::remove_file(tomb.join(tool))?;
        }
        fs::remove_file(manifest_path)?;
    }
    fs::remove_dir(tomb)?;
    File::open(tools)?.sync_all()?;
    Ok(report(generation, &remaining, blockers, None, "released"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::tests::installed_fixture;
    use std::os::unix::fs::PermissionsExt;

    const OLD: &str = "generation.old";
    const CURRENT: &str = "generation.current";

    fn publish_fixture(tools: &Path, name: &str, source: &SourceFingerprint) {
        let path = tools.join(name);
        private_directory(&path, true).unwrap();
        for tool in bootstrap::TOOLS {
            let binary = path.join(tool);
            fs::write(&binary, tool.as_bytes()).unwrap();
            fs::set_permissions(binary, fs::Permissions::from_mode(0o500)).unwrap();
        }
        GenerationManifest::capture(&path, source.clone(), source, bootstrap::TOOLS)
            .unwrap()
            .write(&path)
            .unwrap();
    }

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf, Environment) {
        let (fixture, tools) = installed_fixture();
        let root = tools.parent().unwrap().parent().unwrap().to_owned();
        fs::set_permissions(root.join("target"), fs::Permissions::from_mode(0o700)).unwrap();
        gix::init(&root).unwrap();
        let source = GenerationManifest::load(&tools.join("generation.fixture"))
            .unwrap()
            .source;
        publish_fixture(&tools, OLD, &source);
        publish_fixture(&tools, CURRENT, &source);
        std::os::unix::fs::symlink(CURRENT, tools.join("current")).unwrap();
        owned_file(&tools.join(".update.lock"), true).unwrap();
        let state = fixture.path().join("state");
        private_directory(&state, true).unwrap();
        private_directory(&state.join("pse-arrow"), true).unwrap();
        let env = Environment::from([("XDG_STATE_HOME".into(), state.into_os_string())]);
        (fixture, root, tools, env)
    }

    #[test]
    fn retire_host_checks_current_borrowers_and_new_rollback_selector() {
        let (_fixture, root, tools, env) = setup();
        let current = retire(&root, &env, CURRENT, None).unwrap();
        assert!(current.release.is_none());
        assert_eq!(current.blockers, ["current selection"]);
        let old = retire(&root, &env, OLD, None).unwrap().release.unwrap();
        let lease = borrow(&tools.join(OLD)).unwrap();
        let preview = retire(&root, &env, OLD, None).unwrap();
        assert_eq!(preview.blockers, ["running or borrowed host image"]);
        assert!(retire(&root, &env, OLD, Some(&old)).is_err());
        drop(lease);
        let rollback = tools.join("rollback");
        std::os::unix::fs::symlink(OLD, &rollback).unwrap();
        assert!(retire(&root, &env, OLD, Some(&old)).unwrap().refused());
        fs::remove_file(rollback).unwrap();
        let released = retire(&root, &env, OLD, Some(&old)).unwrap();
        assert_eq!(released.outcome, "released");
        assert!(released.allocated_bytes > 0);
        assert_eq!(
            retire(&root, &env, OLD, Some(&old)).unwrap().outcome,
            "already_absent"
        );
        assert_eq!(
            fs::read_link(tools.join("current")).unwrap(),
            Path::new(CURRENT)
        );
        GenerationManifest::load(&tools.join(CURRENT)).unwrap();
    }

    #[test]
    fn retire_host_refuses_replacement_and_resumes_exact_partial_release() {
        let (_fixture, root, tools, env) = setup();
        let old = retire(&root, &env, OLD, None).unwrap().release.unwrap();
        let path = tools.join(OLD);
        let replacement = tools.join("foreign");
        fs::rename(&path, &replacement).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(retire(&root, &env, OLD, Some(&old)).is_err());
        assert!(path.exists());
        fs::remove_dir(&path).unwrap();
        let tomb = tools.join(format!(".retired-{OLD}-{}", digest(old.as_bytes())));
        fs::rename(replacement, &tomb).unwrap();
        fs::remove_file(tomb.join("pse")).unwrap();
        assert_eq!(
            retire(&root, &env, OLD, Some(&old)).unwrap().outcome,
            "released"
        );
        assert!(!tomb.exists());
    }

    #[test]
    fn retire_host_rechecks_unregistered_checkout_invocation_holds() {
        let (_fixture, root, tools, env) = setup();
        let token = retire(&root, &env, OLD, None).unwrap().release.unwrap();
        let former = root.join("removed-worktree");
        let state = CheckoutState::from_paths(&former, &root.join(".git"), &env).unwrap();
        let ledger = InvocationStore::new(state);
        let id = crate::envelope::CliInvocationId::new();
        let manifest = GenerationManifest::load(&tools.join(OLD)).unwrap();
        ledger
            .begin(crate::invocation::InvocationSpec {
                invocation_id: id.clone(),
                command: "run".into(),
                session: None,
                parent: None,
                operation: (),
                control: PreparedGeneration {
                    generation: OLD.into(),
                    protocol: manifest.control_protocol(),
                    manifest_digest: manifest.digest().unwrap(),
                },
                environment_digest: "0".repeat(64),
                scientific: Vec::new(),
            })
            .unwrap();
        let preview = retire(&root, &env, OLD, None).unwrap();
        assert_eq!(preview.blockers, [format!("original invocation {id}")]);
        assert!(retire(&root, &env, OLD, Some(&token)).unwrap().refused());
        assert!(tools.join(OLD).exists());
    }
}
