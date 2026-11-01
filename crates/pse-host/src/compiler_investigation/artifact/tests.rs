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
            _ => panic!("native stdout fixture selected an unsupported artifact kind: {kind:?}"),
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
        fact_focus: FactFocus::All,
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
fn actual_pinned_facts_preserve_capture_suspension_cleanup_and_constant_boundaries() {
    use crate::code_intelligence_contracts::pb::pse::code_intelligence::v1 as pb;
    use pb::compiler_capture_fact::Mode;
    use pb::compiler_cleanup_fact::{Kind as Terminator, TerminationCause};

    let (_scratch, root, environment, state, host) = fixture();
    fs::write(root.join("src/lib.rs"), r#"
pub fn cases() {
    fn disjoint() {
        let pair = (String::from("borrowed"), String::from("moved"));
        let action = || { std::hint::black_box(&pair.0); drop(pair.1); };
        action();
    }
    async fn scoped() {
        { let transient = std::rc::Rc::new(1u8); std::hint::black_box(&transient); }
        std::future::pending::<()>().await;
    }
    async fn retained() {
        let retained = std::rc::Rc::new(2u8);
        std::future::pending::<()>().await;
        std::hint::black_box(&retained);
        std::future::pending::<()>().await;
        std::hint::black_box(&retained);
    }
    fn conditional(flag: bool) {
        let value: String;
        if flag { value = String::from("conditionally initialized"); }
        std::hint::black_box(flag);
    }
    fn rust_call() { std::hint::black_box(()); }
    extern "C-unwind" fn unwind_call() { rust_call(); }
    extern "C" fn abi_boundary() { panic!("ABI forbids unwind"); }
    fn cleanup() {
        let first = String::from("first");
        let second = String::from("second");
        unwind_call();
        drop(first);
        drop(second);
    }
    const NAMED: usize = 3;
    let inline = const { 4usize };
    std::hint::black_box((NAMED, inline, disjoint, scoped, retained, conditional, cleanup, abi_boundary));
}
"#).unwrap();
    // Explicit preparation belongs to this fixture's isolated environment. Directly waiting
    // its compiler is component evidence; retain_completed does not grant managed custody.
    crate::prepared_xtask::prepare_for(
        &checkout(),
        &environment,
        crate::prepared_xtask::NativeToolTarget::CompilerAnalysis,
    )
    .unwrap();
    let prepared = prepare(
        &root,
        &checkout(),
        &state,
        &CliInvocationId::new(),
        request(Kind::Facts, Some("cases"), &[]),
        &environment,
        &host,
    )
    .unwrap();
    prepared.verify().unwrap();
    let output = actual(&prepared);
    let index = retain_completed(&prepared, &state, &output);
    let file = index
        .files
        .iter()
        .find(|file| Path::new(&file.path).file_name() == Some(OsStr::new("facts.binpb")))
        .unwrap();
    let facts = super::super::facts::decode(&fs::read(Path::new(&file.path)).unwrap()).unwrap();
    assert!(facts.bodies.iter().any(|body| body.path == "cases"));
    assert!(
        facts
            .bodies
            .iter()
            .all(|body| body.path == "cases" || body.path.starts_with("cases::"))
    );

    let captures = facts
        .bodies
        .iter()
        .filter(|body| body.path.starts_with("cases::disjoint::"))
        .flat_map(|body| &body.captures)
        .collect::<Vec<_>>();
    assert!(
        captures
            .iter()
            .any(|capture| capture.place_display.contains("pair.0")
                && capture.mode.as_known() == Some(Mode::SHARED)
                && capture.source_type_display == "std::string::String"
                && capture.stored_type_display.as_deref() == Some("&std::string::String")),
        "{captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|capture| capture.place_display.contains("pair.1")
                && capture.mode.as_known() == Some(Mode::MOVED)
                && capture.source_type_display == "std::string::String"
                && capture.stored_type_display.as_deref() == Some("std::string::String")),
        "{captures:?}"
    );

    let coroutine = |name: &str| {
        facts
            .bodies
            .iter()
            .find(|body| body.coroutine && body.path.starts_with(name))
            .unwrap()
    };
    let scoped = coroutine("cases::scoped::");
    assert!(scoped.unavailable.is_empty(), "{scoped:?}");
    assert!(
        !scoped
            .saved_fields
            .iter()
            .any(|field| field.type_display.contains("Rc<")),
        "{scoped:?}"
    );
    let retained = coroutine("cases::retained::");
    assert!(retained.unavailable.is_empty(), "{retained:?}");
    let rc = retained
        .saved_fields
        .iter()
        .find(|field| field.type_display.contains("Rc<"))
        .unwrap();
    assert!(
        retained
            .variants
            .iter()
            .filter(|variant| variant.fields.contains(&rc.index))
            .count()
            >= 2,
        "the same Rc storage crosses both authored awaits: {retained:?}"
    );

    let conditional = facts
        .bodies
        .iter()
        .find(|body| body.path == "cases::conditional")
        .unwrap();
    assert!(
        conditional
            .cleanup
            .iter()
            .any(|fact| fact.kind.as_known() == Some(Terminator::DROP)
                && fact.phase.starts_with("Analysis")
                && fact
                    .initialization
                    .as_option()
                    .is_some_and(|state| state.may_initialized && state.may_uninitialized)),
        "{conditional:?}"
    );
    assert!(
        conditional
            .cleanup
            .iter()
            .any(|fact| fact.phase.starts_with("Runtime")),
        "{conditional:?}"
    );
    let cleanup = facts
        .bodies
        .iter()
        .find(|body| body.path == "cases::cleanup")
        .unwrap();
    assert!(
        cleanup
            .cleanup
            .iter()
            .any(|fact| fact.is_cleanup && fact.kind.as_known() == Some(Terminator::DROP)),
        "{cleanup:?}"
    );
    assert!(
        cleanup.cleanup.iter().any(|fact| fact
            .termination_cause
            .as_ref()
            .and_then(|cause| cause.as_known())
            == Some(TerminationCause::IN_CLEANUP)),
        "{cleanup:?}"
    );
    assert!(
        cleanup.cleanup.iter().any(|fact| fact
            .callee
            .as_option()
            .is_some_and(|callee| callee.abi == "C-unwind"
                && callee.declaration_path.as_deref() == Some("cases::unwind_call"))),
        "{cleanup:?}"
    );
    let abi = facts
        .bodies
        .iter()
        .find(|body| body.path == "cases::abi_boundary")
        .unwrap();
    assert_eq!(abi.abi, "C");
    assert!(
        abi.cleanup.iter().any(|fact| fact
            .termination_cause
            .as_ref()
            .and_then(|cause| cause.as_known())
            == Some(TerminationCause::ABI)),
        "{abi:?}"
    );
    let constants = facts
        .bodies
        .iter()
        .filter(|body| {
            body.unavailable
                .iter()
                .any(|reason| reason.contains("CTFE"))
        })
        .collect::<Vec<_>>();
    assert!(
        constants.len() >= 2,
        "named and inline constants are explicitly unavailable: {constants:?}"
    );
    assert!(constants.iter().any(|body| body.path == "cases::NAMED"));
    assert!(constants.iter().all(|body| body.captures.is_empty()
        && body.saved_fields.is_empty()
        && body.cleanup.is_empty()));
}

#[test]
fn runtime_alias_retarget_refuses_even_when_original_image_digest_is_unchanged() {
    let scratch = tempfile::tempdir().unwrap();
    let original = scratch.path().join("liboriginal.so.1");
    let replacement = scratch.path().join("libreplacement.so.1");
    let alias = scratch.path().join("libcompiler.so");
    fs::write(&original, b"original native image").unwrap();
    fs::write(&replacement, b"different native image").unwrap();
    std::os::unix::fs::symlink(&original, &alias).unwrap();
    let digest = file_digest(&original).unwrap();
    let paths = [RuntimePath {
        requested: alias.as_os_str().to_owned(),
        resolved: alias.canonicalize().unwrap().into_os_string(),
    }];
    verify_runtime_paths(&paths).unwrap();
    fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&replacement, &alias).unwrap();
    assert_eq!(file_digest(&original).unwrap().sha256, digest.sha256);
    assert!(verify_runtime_paths(&paths).is_err());
}
#[test]
fn native_flags_are_filtered_without_shell_interpolation_and_unavailable_filters_refuse() {
    assert!(
        flags(
            Kind::Facts,
            Some("captures"),
            &[],
            Path::new("/original/native")
        )
        .unwrap()
        .is_empty()
    );
    assert!(flags(Kind::Facts, None, &[], Path::new("/original/native")).is_err());
    assert!(
        flags(
            Kind::Facts,
            Some("captures"),
            &["ElaborateDrops".into()],
            Path::new("/original/native")
        )
        .is_err()
    );
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
            fact_focus: FactFocus::All,
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
            fact_focus: FactFocus::All,
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
