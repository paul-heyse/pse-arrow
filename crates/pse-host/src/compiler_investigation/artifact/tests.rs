// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::process::Output;
fn checkout() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_owned()
}
fn command(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}
fn fixture() -> (
    tempfile::TempDir,
    PathBuf,
    Environment,
    CheckoutState,
    PreparedGeneration,
) {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path().join("selected");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname=\"native-investigation-case\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"),r#"pub struct Pair { left:String, right:String }
pub fn partial(flag:bool) { let pair=Pair {left:String::from("left"),right:String::from("right")}; let left=pair.left; if flag {drop(left);} let _remaining=pair.right.len(); }
pub async fn across() { let value=String::from("saved"); std::future::pending::<()>().await; drop(value); }
#[inline(never)] pub fn identity<T>(value:T)->T {value}
pub fn instances() { let _=identity(3u32); let _=identity(4u64); }
pub fn autoref(value:&String)->usize {value.len()}
"#).unwrap();
    fs::copy(
        checkout().join("rust-toolchain.toml"),
        root.join("rust-toolchain.toml"),
    )
    .unwrap();
    command(&root, &["init", "--quiet", "--initial-branch=main"]);
    let mut environment: Environment = std::env::vars_os().collect();
    environment.insert(
        "XDG_STATE_HOME".into(),
        scratch.path().join("state").into_os_string(),
    );
    environment.remove(OsStr::new("CARGO_BUILD_TARGET"));
    let lock = Command::new(correctness::real_cargo_in(&environment).unwrap())
        .current_dir(&root)
        .env_clear()
        .envs(&environment)
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(lock.status.success(), "{lock:?}");
    let state = CheckoutState::resolve(&root, &environment).unwrap();
    let host = crate::generation::prepared(&checkout()).unwrap().unwrap();
    (scratch, root, environment, state, host)
}
fn retain_completed(prepared: &Prepared, state: &CheckoutState, output: &Output) -> Index {
    assert!(output.status.success(), "{output:?}");
    let store = InvocationStore::new(state.clone());
    // This is fixture metadata for the actual directly waited command, not a fabricated native
    // managed pass/settlement capability. The public journey qualifies that enclosing owner.
    let directory = store
        .begin(crate::invocation::InvocationSpec {
            invocation_id: prepared.context.invocation.clone(),
            command: "compiler generate".into(),
            session: None,
            parent: None,
            operation: serde_json::json!({"command":prepared.command}),
            control: prepared.context.prepared_host.clone(),
            environment_digest: prepared.context.environment_sha256.clone(),
            scientific: vec![],
        })
        .unwrap();
    publish(&directory.path().join("stdout.raw"), &output.stdout).unwrap();
    publish(&directory.path().join("stderr.raw"), &output.stderr).unwrap();
    finish(prepared, &store).unwrap_or_else(|error| {
        panic!(
            "{error}; kind={:?}; body={:?}; args={:?}; original={output:?}",
            prepared.context.kind, prepared.context.body, prepared.context.admitted.arguments
        )
    });
    serde_json::from_slice(&fs::read(prepared.directory.join("index.json")).unwrap()).unwrap()
}
#[test]
fn actual_filtered_partial_move_drop_and_saved_async_value_keep_original_native_files() {
    let (_scratch, root, environment, state, host) = fixture();
    let partial = prepare(
        &root,
        &checkout(),
        &state,
        &CliInvocationId::new(),
        request(Kind::Mir, Some("partial"), &["ElaborateDrops"]),
        &environment,
        &host,
    )
    .unwrap();
    partial.verify().unwrap();
    let output = actual(&partial);
    let index = retain_completed(&partial, &state, &output);
    let mir = index
        .files
        .iter()
        .filter(|file| Path::new(&file.path).extension() == Some(OsStr::new("mir")))
        .collect::<Vec<_>>();
    assert_eq!(
        mir.len(),
        2,
        "native before/after only for selected body/pass: {mir:?}"
    );
    assert!(
        mir.iter()
            .all(|file| file.path.to_str().unwrap().contains("partial"))
    );
    let after = mir
        .iter()
        .find(|file| file.path.to_str().unwrap().contains("after.mir"))
        .unwrap();
    let text = fs::read_to_string(Path::new(&after.path)).unwrap();
    assert!(text.contains("drop("));
    assert!(
        text.contains(".1:"),
        "remaining right field appears in original selected MIR"
    );
    let verified = crate::artifacts::read_descriptor(
        crate::test_inventory::open_private(Path::new(&after.path)).unwrap(),
        0,
        std::num::NonZeroU64::new(32),
        Some(&after.sha256),
    )
    .unwrap();
    assert_eq!(verified.byte_range.end, 32);
    assert_eq!(
        verified.verified_sha256.as_deref(),
        Some(after.sha256.as_str())
    );
    fs::write(Path::new(&after.path), "changed debug bytes").unwrap();
    assert!(
        crate::artifacts::read_descriptor(
            crate::test_inventory::open_private(Path::new(&after.path)).unwrap(),
            0,
            std::num::NonZeroU64::new(32),
            Some(&after.sha256)
        )
        .is_err()
    );
    let across = prepare(
        &root,
        &checkout(),
        &state,
        &CliInvocationId::new(),
        request(Kind::Mir, Some("across"), &[]),
        &environment,
        &host,
    )
    .unwrap();
    let output = actual(&across);
    let index = retain_completed(&across, &state, &output);
    let transformed = index
        .files
        .iter()
        .filter(|file| file.path.to_str().unwrap().contains("StateTransform"))
        .collect::<Vec<_>>();
    assert!(
        !transformed.is_empty(),
        "selected pinned compiler coroutine stage"
    );
    assert!(transformed.iter().any(|file| {
        fs::read_to_string(Path::new(&file.path))
            .unwrap()
            .contains("std::string::String")
    }));
}
#[test]
fn actual_thir_layout_and_mono_output_remain_opaque_with_selected_tool_and_unit_context() {
    let (_scratch, root, environment, state, host) = fixture();
    for kind in [Kind::Thir, Kind::TypeSizes, Kind::MonoItems] {
        let prepared = prepare(
            &root,
            &checkout(),
            &state,
            &CliInvocationId::new(),
            request(kind, None, &[]),
            &environment,
            &host,
        )
        .unwrap();
        prepared.verify().unwrap();
        let output = actual(&prepared);
        assert!(output.status.success(), "{kind:?}: {output:?}");
        assert!(prepared.context.rustc_identity.contains("commit-hash:"));
        assert!(prepared.context.cargo_identity.contains("host:"));
        assert!(
            prepared
                .context
                .admitted
                .arguments
                .iter()
                .any(|arg| arg == "rustc")
        );
        let text = String::from_utf8(output.stdout.clone()).unwrap();
        match kind {
            Kind::Thir => assert!(text.contains("Scope") && text.contains("autoref")),
            Kind::TypeSizes => assert!(
                text.contains("Pair")
                    && text.contains("48 bytes")
                    && text.contains("alignment: 8 bytes")
            ),
            Kind::MonoItems => {
                assert!(text.contains("identity::<u32>") && text.contains("identity::<u64>"))
            }
            _ => unreachable!(),
        };
        let index = retain_completed(&prepared, &state, &output);
        assert!(
            index
                .files
                .iter()
                .any(|file| Path::new(&file.path).file_name() == Some(OsStr::new("source.rs")))
        );
        assert!(
            index
                .files
                .iter()
                .any(|file| Path::new(&file.path).file_name() == Some(OsStr::new("stdout.raw")))
        );
    }
}
fn request(kind: Kind, body: Option<&str>, passes: &[&str]) -> Request {
    Request {
        kind,
        package: "native-investigation-case".into(),
        binary: None,
        target: None,
        feature_arguments: vec![],
        body: body.map(str::to_owned),
        passes: passes.iter().map(|value| (*value).into()).collect(),
        question: "independent selected native evidence".into(),
    }
}
fn actual(prepared: &Prepared) -> Output {
    // This tiny independent fixture uses the actual prepared correctness executable, while
    // production generation enters the existing managed foreground owner around the same args.
    Command::new(&prepared.command[0])
        .args(&prepared.command[1..])
        .current_dir(Path::new(&prepared.context.cwd))
        .env_clear()
        .envs(&prepared.environment)
        .output()
        .unwrap()
}
#[test]
fn native_flags_are_filtered_without_shell_interpolation_and_unavailable_filters_refuse() {
    let native = flags(
        Kind::Mir,
        Some("partial"),
        &["ElaborateDrops".into(), "StateTransform".into()],
        Path::new("/original/native"),
    )
    .unwrap();
    assert_eq!(
        native[0],
        "-Zdump-mir=partial & ElaborateDrops | partial & StateTransform"
    );
    for body in [
        None,
        Some(""),
        Some("  "),
        Some("all"),
        Some(" all "),
        Some("one | all"),
    ] {
        assert!(flags(Kind::Mir, body, &[], Path::new("/original/native")).is_err());
    }
    assert!(
        flags(
            Kind::Thir,
            Some("partial"),
            &[],
            Path::new("/original/native")
        )
        .is_err()
    );
    assert!(
        flags(
            Kind::MonoItems,
            None,
            &["all".into()],
            Path::new("/original/native")
        )
        .is_err()
    );
    assert!(flags(Kind::Evaluate, None, &[], Path::new("/original/native")).is_err());
}
#[test]
fn selected_native_context_respects_explicit_checkout_and_stale_source_refuses_before_generation() {
    let (_scratch, root, environment, state, host) = fixture();
    let before = std::env::current_dir().unwrap();
    // The selected fixture has no prepared host. Only the actual executable owner is reused;
    // no source fixture is bootstrapped and no global cwd/environment is changed.
    let prepared = prepare(
        &root,
        &checkout(),
        &state,
        &CliInvocationId::new(),
        request(Kind::Mir, Some("partial"), &["ElaborateDrops"]),
        &environment,
        &host,
    )
    .unwrap();
    prepared.verify().unwrap();
    assert_eq!(Path::new(&prepared.context.cwd), root);
    assert_eq!(prepared.context.source.path(), root.join("src/lib.rs"));
    assert!(
        prepared
            .context
            .admitted
            .units
            .iter()
            .any(|unit| unit.root && unit.package_name == "native-investigation-case")
    );
    assert!(!root.join("target/debug/deps").exists());
    fs::write(root.join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
    assert!(prepared.verify().is_err());
    assert_eq!(std::env::current_dir().unwrap(), before);
}
#[tokio::test]
async fn authored_probe_concrete_layout_constant_and_unresolved_generic_remain_distinct() {
    let scratch = tempfile::tempdir().unwrap();
    let root = checkout();
    let mut environment: Environment = std::env::vars_os().collect();
    environment.insert(
        "XDG_STATE_HOME".into(),
        scratch.path().join("state").into_os_string(),
    );
    environment.remove(OsStr::new("CARGO_BUILD_TARGET"));
    environment.remove(OsStr::new("RUSTUP_TOOLCHAIN"));
    let state = CheckoutState::resolve(&root, &environment).unwrap();
    InvocationStore::new(state.clone()).register().unwrap();
    let snapshot = crate::dependency::capture(&root, &environment).unwrap();
    let host = crate::generation::prepared(&root).unwrap().unwrap();
    let concrete =
        crate::probe::new(&root, &state.root, "concrete-layout", &[], &snapshot).unwrap();
    fs::write(concrete.directory.join("src/main.rs"),r#"#[repr(C)] struct Concrete {tag:u8,value:u64}
const CONSTANT:usize=3*4+1;
fn main() {println!("constant={} size={} align={} value_offset={}",CONSTANT,std::mem::size_of::<Concrete>(),std::mem::align_of::<Concrete>(),std::mem::offset_of!(Concrete,value));}
"#).unwrap();
    let (prepared, run) = prepare_probe(
        &root,
        &root,
        &state,
        &CliInvocationId::new(),
        ProbeRequest {
            name: "concrete-layout".into(),
            kind: Kind::Evaluate,
            question: "concrete constant and host layout".into(),
            body: None,
            passes: vec![],
        },
        &environment,
        &host,
    )
    .unwrap();
    assert!(
        crate::probe::rm(&root, &state.root, "concrete-layout", &environment)
            .await
            .is_err()
    );
    prepared.verify().unwrap();
    let output = actual(&prepared);
    assert_eq!(
        String::from_utf8(output.stdout.clone()).unwrap().trim(),
        "constant=13 size=16 align=8 value_offset=8"
    );
    let index = retain_completed(&prepared, &state, &output);
    assert!(
        index
            .files
            .iter()
            .any(|file| Path::new(&file.path).file_name() == Some(OsStr::new("source.rs")))
    );
    assert!(prepared.context.probe.is_some());
    drop(run.lease);
    assert!(
        crate::probe::rm(&root, &state.root, "concrete-layout", &environment)
            .await
            .is_err(),
        "direct component capture has no authenticated managed reference and cannot authorize product deletion"
    );
    assert!(prepared.directory.join("source.rs").is_file());
    let unresolved =
        crate::probe::new(&root, &state.root, "unresolved-layout", &[], &snapshot).unwrap();
    fs::write(
        unresolved.directory.join("src/main.rs"),
        "fn main() { println!(\"{}\",std::mem::size_of::<_>()); }\n",
    )
    .unwrap();
    let (prepared, run) = prepare_probe(
        &root,
        &root,
        &state,
        &CliInvocationId::new(),
        ProbeRequest {
            name: "unresolved-layout".into(),
            kind: Kind::Evaluate,
            question: "unresolved concrete substitution".into(),
            body: None,
            passes: vec![],
        },
        &environment,
        &host,
    )
    .unwrap();
    let output = actual(&prepared);
    assert_eq!(output.status.code(), Some(101));
    assert!(String::from_utf8(output.stderr).unwrap().contains("E0282"));
    assert!(!prepared.directory.join("index.json").exists());
    assert!(prepared.directory.join("source.rs").is_file());
    drop(run.lease);
    assert!(
        crate::probe::rm(&root, &state.root, "unresolved-layout", &environment)
            .await
            .is_err(),
        "missing original managed custody is unknown even after the component waited its direct fixture child"
    );
}
