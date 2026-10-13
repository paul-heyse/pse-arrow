// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::sync::atomic::{AtomicBool};

#[test]
fn private_native_frames_refuse_oversize_and_keep_original_stop() {
    let (mut reader, mut writer) = UnixStream::pair().unwrap();
    writer.write_all(&((RESPONSE_LIMIT + 1) as u64).to_le_bytes()).unwrap();
    let scope = ExecutionScope::new(Arc::default(), None);
    assert!(matches!(receive::<Response>(&mut reader, RESPONSE_LIMIT, &scope), Err(MathRuntimeError::Solve(_))));
    let cancel = Arc::new(AtomicBool::new(true));
    let scope = ExecutionScope::new(cancel, None);
    assert!(receive::<Response>(&mut reader, RESPONSE_LIMIT, &scope).is_err());
}

#[test]
fn portable_registry_declarations_roundtrip_and_changed_context_is_distinct() {
    let registry = pse_quantity::generated::standard_registry().unwrap();
    let bytes = serde_json::to_vec(&registry.to_builder()).unwrap();
    let builder: pse_quantity::QuantityRegistryBuilder = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(builder.build().unwrap(), registry);
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    wire["units"][0]["scale_to_canonical"] = serde_json::json!(2.0);
    let changed = serde_json::to_vec(&wire).unwrap();
    assert_ne!(digest(&bytes), digest(&changed));
    let builder: pse_quantity::QuantityRegistryBuilder = serde_json::from_slice(&changed).unwrap();
    match builder.build() { Ok(other) => assert_ne!(other, registry), Err(_) => {} }
}

fn source() -> PortableRootSource {
    use pse_compiler::typed_math::{BodyLimits, Formal, Request};
    pse_math::initialize().unwrap();
    let registry = Arc::new(pse_quantity::generated::standard_registry().unwrap());
    let preconditions = Arc::new(pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap());
    let context = CompilerContext { quantities: registry.clone(), preconditions, providers: Default::default() };
    let length = match registry.physical_name("Length").unwrap() {
        pse_quantity::PhysicalName::QuantityType(id) => id, other => panic!("{other:?}"),
    };
    let expressions = [pse_authoring::dsl::parse_expr("x - 2{m}").unwrap()];
    let formals = [Formal { path: "x".into(), quantity: length }];
    let empty = std::collections::BTreeMap::new();
    let body = Request { definition: SemanticId::from_bytes([188;16]), expressions: &expressions,
        formals: &formals, domains: &empty, groups: &Default::default(), providers: &Default::default(),
        literals: &Default::default(), physical: pse_compiler::workspace::physical_identity(&registry, &context.preconditions),
        structure: ContentHash::from_bytes([189;32]), limits: BodyLimits::default() }
        .admit_modeling_outputs(&registry, &pse_quantity::standard::StandardInvariantChecker,
            &Arc::default(), &Default::default(), &[length], &Default::default(), &Default::default()).unwrap();
    PortableRootSource::new(Arc::new(body), context,
        vec![Unknown { id: SemanticId::from_bytes([190;16]), lower: 0.0, upper: 10.0 }],
        vec![SemanticId::from_bytes([191;16])], vec![],
        Options { start: vec![5.0], variable_nominals: vec![1.0], variable_tolerance: vec![1e-8],
            residual_tolerance: vec![1e-8], iterations: 50, time_limit: Duration::from_secs(20), derivative_tolerance: 1e-10 },
        EvaluationLimits { scratch_bytes: 1<<20, ..Default::default() }, 4096).unwrap()
}

#[test]
fn native_child_source_requires_actual_physical_registry_before_launch() {
    let mut source = source();
    source.context.quantities = Arc::new(pse_quantity::QuantityRegistryBuilder::new().build().unwrap());
    assert!(PortableRootSource::new(source.body, source.context, source.unknowns, source.rows,
        source.parameters, source.options, source.evaluation, source.max_entries).is_err());
}

// The actual private binary is an explicit executable prerequisite supplied by the
// integrated selector. Never discover a stale PATH binary or build one within a test.
#[tokio::test]
async fn actual_portable_native_root_child_reconstructs_and_parent_screens() {
    let executable = std::env::var_os("PSE_NATIVE_LANE_EXECUTABLE").expect("integrator supplies exact built pse-native-lane executable");
    let (service, _) = super::super::tests::service_with_policy(512<<20,
        super::super::MathPolicy { worker_bytes: 256<<20, foreign_bytes: 1<<20, ..Default::default() });
    let baseline = service.pool.reserved();
    let child = NativeChild::qualify(executable, 128<<20).unwrap();
    let handle = service.solve_portable_root(source(), child, FlightCancellation::default(), None).unwrap();
    let (result, lease) = handle.finish().await.unwrap();
    assert_ne!(result.child_pid, std::process::id());
    assert!(result.refusal.is_none(), "{:?}", result.refusal);
    assert!((result.proposal.as_ref().unwrap()[0]-2.0).abs() < 1e-8);
    assert!(!Path::new(&format!("/proc/{}", result.child_pid)).exists(), "result cannot precede actual child reap");
    drop((result, lease));
    assert_eq!(service.pool.reserved(), baseline);
}

#[test]
fn private_native_fake_process() {
    let Ok(mode) = std::env::var("PSE_PRIVATE_NATIVE_TEST_MODE") else { return; };
    let marker = std::env::var_os("PSE_PRIVATE_NATIVE_TEST_MARKER").unwrap();
    std::fs::write(marker, std::process::id().to_string()).unwrap();
    if mode == "death" { std::process::exit(17); }
    let mut channel = UnixStream::from(std::io::stdin().as_fd().try_clone_to_owned().unwrap());
    let scope = ExecutionScope::new(Arc::default(), None);
    let attempt = SemanticId::parse_hex(&std::env::var("PSE_PRIVATE_NATIVE_TEST_ATTEMPT").unwrap()).unwrap();
    let parent_record = std::fs::read_to_string("/proc/self/status").unwrap();
    let parent = parent_record.lines().find_map(|line| line.strip_prefix("PPid:")).unwrap().trim().parse().unwrap();
    let hello = Hello { protocol: PROTOCOL.into(), attempt: if mode == "foreign" { SemanticId::NIL } else { attempt },
        pid: std::process::id(), parent,
        executable: pse_buildinfo::FileObservation::capture(&std::env::current_exe().unwrap()).unwrap(),
        build: pse_buildinfo::BuildInfo::captured(), native_build: pse_backend_native::execution::adapter(pse_backend_native::solve::Backend::Kinsol).build(),
    };
    send(&mut channel, &hello, RESPONSE_LIMIT, &scope).unwrap();
    if mode == "foreign" { return; }
    let request: Request = receive(&mut channel, REQUEST_LIMIT, &scope).unwrap();
    if mode == "late" { std::thread::sleep(Duration::from_secs(8)); }
    let point = if mode == "invalid" { vec![9.0] } else { vec![2.0] };
    let _ = send(&mut channel, &Response { attempt, execution: request.execution, request: request.request, result: Ok(point) }, RESPONSE_LIMIT, &scope);
}

fn fake_child(mode: &str, marker: &Path) -> NativeChild {
    let mut child = NativeChild::qualify(std::env::current_exe().unwrap(), 128<<20).unwrap();
    child.test_mode = Some((mode.into(), marker.to_path_buf()));
    child
}

#[tokio::test]
async fn private_native_process_death_foreign_attempt_and_original_residual_refuse_after_reap() {
    let (service, _) = super::super::tests::service_with_policy(512<<20,
        super::super::MathPolicy { worker_bytes: 256<<20, foreign_bytes: 1<<20, ..Default::default() });
    let baseline = service.pool.reserved();
    for mode in ["death", "foreign", "invalid"] {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("owned-child");
        let handle = service.solve_portable_root(source(), fake_child(mode, &marker), FlightCancellation::default(), None).unwrap();
        let result = handle.finish().await;
        assert!(result.is_err(), "{mode}: a dead, foreign or physically invalid proposal cannot publish");
        let pid: u32 = std::fs::read_to_string(&marker).unwrap().parse().unwrap();
        assert!(!Path::new(&format!("/proc/{pid}")).exists(), "{mode}: failure returned before actual reap");
        assert_eq!(service.pool.reserved(), baseline, "{mode}: owned parent charges released after reap");
    }
}

#[tokio::test]
async fn private_native_late_output_and_cancellation_hold_parent_charges_until_reap() {
    let (service, _) = super::super::tests::service_with_policy(512<<20,
        super::super::MathPolicy { worker_bytes: 256<<20, foreign_bytes: 1<<20, ..Default::default() });
    let baseline = service.pool.reserved();
    for cancel_owned in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("owned-child");
        let control = FlightCancellation::default();
        let deadline = if cancel_owned { None } else { Some(Instant::now()+Duration::from_secs(3)) };
        let handle = service.solve_portable_root(source(), fake_child("late", &marker), control.clone(), deadline).unwrap();
        for _ in 0..1000 {
            if marker.exists() { break; }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        let pid: u32 = std::fs::read_to_string(&marker).unwrap().parse().unwrap();
        assert!(Path::new(&format!("/proc/{pid}")).exists());
        assert!(service.pool.reserved() > baseline, "live child keeps parent admission charges");
        if cancel_owned { control.cancel(); }
        assert!(handle.finish().await.is_err(), "late output cannot acquire proposal permission");
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(control.flag().load(std::sync::atomic::Ordering::Acquire), cancel_owned,
            "deadline never masquerades as cancellation");
    }
}
