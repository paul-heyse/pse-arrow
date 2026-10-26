// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

#[test]
fn private_native_frames_refuse_oversize_and_keep_original_stop() {
    let (mut reader, mut writer) = UnixStream::pair().unwrap();
    writer
        .write_all(&((RESPONSE_LIMIT + 1) as u64).to_le_bytes())
        .unwrap();
    let scope = ExecutionScope::new(Cancellation::default(), None);
    let error = receive::<Response>(&mut reader, RESPONSE_LIMIT, &scope)
        .err()
        .expect("private frame must refuse");
    assert!(matches!(error, MathRuntimeError::Limit(_)));
    assert_eq!(
        error
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
            .class,
        pse_model::diagnostic::BoundaryClass::ResourceLimit
    );
    let cancel = Cancellation::new(true);
    let scope = ExecutionScope::new(cancel, None);
    let error = receive::<Response>(&mut reader, RESPONSE_LIMIT, &scope)
        .err()
        .expect("private frame must refuse");
    assert_eq!(
        error
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
            .class,
        pse_model::diagnostic::BoundaryClass::Cancelled
    );
}

#[test]
fn private_native_outgoing_frame_budget_differs_from_serialization_invariant() {
    let (_reader, mut writer) = UnixStream::pair().unwrap();
    let scope = ExecutionScope::new(Cancellation::default(), None);
    let error = send(&mut writer, &vec![1_u8, 2, 3], 1, &scope).unwrap_err();
    assert_eq!(
        error
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
            .class,
        pse_model::diagnostic::BoundaryClass::ResourceLimit
    );
    struct Broken;
    impl Serialize for Broken {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("admitted serializer failed"))
        }
    }
    let error = send(&mut writer, &Broken, RESPONSE_LIMIT, &scope).unwrap_err();
    assert!(matches!(error, MathRuntimeError::Internal(_)));
    assert_eq!(
        error
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
            .class,
        pse_model::diagnostic::BoundaryClass::Internal
    );
}

#[test]
fn private_native_corrupt_admitted_frame_is_internal() {
    let (mut reader, mut writer) = UnixStream::pair().unwrap();
    writer.write_all(&1_u64.to_le_bytes()).unwrap();
    writer.write_all(b"?").unwrap();
    let scope = ExecutionScope::new(Cancellation::default(), None);
    let error = receive::<Response>(&mut reader, RESPONSE_LIMIT, &scope)
        .err()
        .expect("private frame must refuse");
    assert!(matches!(error, MathRuntimeError::Internal(_)));
    let diagnostic = error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native);
    assert_eq!(
        diagnostic.class,
        pse_model::diagnostic::BoundaryClass::Internal
    );
    assert_eq!(
        diagnostic.code,
        pse_diagnostics::DiagnosticCode::InternalInvariant
    );
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
    if let Ok(other) = builder.build() {
        assert_ne!(other, registry)
    }
}

fn source() -> PortableRootSource {
    use pse_compiler::typed_math::{Request, prepare_selected_providers};
    use pse_math::typed::BodyLimits;
    use pse_modeling::{
        Bindings, Limits, PhysicalScope, TypeContext,
        resolved::{RegionOutput, RegionRequest},
    };
    pse_math::initialize().unwrap();
    let registry = Arc::new(pse_quantity::generated::standard_registry().unwrap());
    let preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    let context = CompilerContext {
        quantities: registry.clone(),
        preconditions,
        providers: Default::default(),
    };
    // Preserve the original unknown-first, parameter-second physical Length residual.
    let rows = pse_authoring::language::parse(
        "package native_child_fixture {def Root {var x:Length;var p:Length;annotation valid x(0{m},10{m});let residual:Length=x-p;}}",
        SemanticId::from_bytes([188;16]), pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    ).unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let scope = PhysicalScope::default();
    let types = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: &registry,
        preconditions: context.preconditions.as_ref(),
        scope: &scope,
    };
    let checked = pse_modeling::check(&rows, &types).unwrap();
    let model = pse_modeling::specialize(
        &checked,
        root,
        pse_modeling::specialize::root_instance(root),
        &Bindings {
            demand: vec!["residual".into()],
            ..Default::default()
        },
        Limits::default(),
    )
    .unwrap();
    let member = |name: &str| {
        let suffix = format!(".{name}");
        let selected = model
            .symbols
            .values()
            .filter(|symbol| symbol.lineage.path.ends_with(&suffix))
            .collect::<Vec<_>>();
        assert_eq!(
            selected.len(),
            1,
            "actual native-child fixture member {name}"
        );
        selected[0].id
    };
    let x = member("x");
    let p = member("p");
    let outputs = [RegionOutput::Value(
        model.symbol_root(member("residual")).unwrap(),
    )];
    let contracts = model.function_contracts(&checked);
    let region = model
        .prepare_region(
            RegionRequest {
                outputs: &outputs,
                validity: &[],
            },
            &contracts.context(),
        )
        .unwrap();
    let actual_inputs = region
        .inputs
        .iter()
        .map(|local| {
            region
                .bindings
                .iter()
                .find(|binding| binding.local == *local)
                .unwrap()
                .actual
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual_inputs,
        [x, p],
        "portable root coordinates retain unknown then parameter"
    );
    let providers = prepare_selected_providers(
        &region.syntax,
        &region.support,
        &mut |name| {
            context.providers.get(name).cloned().ok_or_else(|| {
                pse_math::MathError::Contract(format!(
                    "unregistered native-child fixture provider {name}"
                ))
            })
        },
        &Default::default(),
        &contracts.context(),
    )
    .unwrap();
    let body = Request {
        definition: root.as_id(),
        syntax: &region.syntax,
        inputs: &region.inputs,
        support: &region.support,
        providers: &providers,
        physical: pse_compiler::workspace::physical_identity(&registry, &context.preconditions),
        structure: region.syntax.identity().unwrap(),
        limits: BodyLimits::default(),
    }
    .admit(&registry, context.preconditions.as_ref(), &Arc::default())
    .unwrap();
    let bindings = SourceBindings::new(
        pse_modeling::specialize::root_instance(root).as_id(),
        body.math(),
        region.checked_members.clone(),
        region.checked_sources.clone(),
        region.occurrences.clone(),
    )
    .unwrap();
    PortableRootSource::new(
        Arc::new(body),
        bindings,
        context,
        vec![Unknown {
            id: x,
            lower: 0.0,
            upper: 10.0,
        }],
        vec![member("residual")],
        vec![2.0],
        Options {
            start: vec![5.0],
            variable_nominals: vec![1.0],
            variable_tolerance: vec![1e-8],
            residual_tolerance: vec![1e-8],
            iterations: 50,
            time_limit: Duration::from_secs(20),
            derivative_tolerance: 1e-10,
        },
        EvaluationLimits {
            scratch_bytes: 1 << 20,
            ..Default::default()
        },
        4096,
    )
    .unwrap()
}

#[test]
fn native_child_source_bindings_wire_keeps_actual_receiver_and_refuses_missing_maps() {
    let source = source();
    assert!(!source.bindings.checked_members().is_empty());
    assert!(!source.bindings.checked_sources().is_empty());
    assert!(!source.bindings.occurrences().is_empty());
    let bytes = serde_json::to_vec(&source.wire().unwrap()).unwrap();
    let wire: RootWire = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        wire.source.bind(source.body.math()).unwrap(),
        source.bindings
    );
    let scope = ExecutionScope::new(Cancellation::default(), None);
    let native_interval_error = source
        .problem(&scope)
        .unwrap()
        .evaluate_unknown(
            &source.parameters,
            &[11.0],
            DerivativeOrder::First,
            scope.cancellation(),
        )
        .unwrap_err();
    assert!(matches!(
        native_interval_error,
        pse_math::MathError::Domain { source_id, requirement }
            if source_id == source.rows[0]
                && requirement == "implicit unknown outside its declared interval"
    ));
    // A wider native search interval lets this trial reach the unchanged authored
    // range guard, whose error must retain the actual receiving source maps.
    let unknowns = [Unknown {
        id: source.unknowns[0].id,
        lower: -1.0,
        upper: 12.0,
    }];
    let error = compile_problem(
        (&source.body, &source.bindings),
        &unknowns,
        &source.rows,
        source.parameters.len(),
        source.evaluation,
        source.max_entries,
        &scope,
    )
    .unwrap()
    .evaluate_unknown(
        &source.parameters,
        &[11.0],
        DerivativeOrder::First,
        scope.cancellation(),
    )
    .unwrap_err();
    let pse_math::MathError::Instance {
        instance,
        checked_members,
        checked_sources,
        occurrences,
        cause,
    } = error
    else {
        panic!("native child original range failure lacks its actual receiving context");
    };
    let pse_math::MathError::Validity(lineage) = cause.as_ref() else {
        panic!("native child authored range lost its validity lineage: {cause:?}");
    };
    assert_eq!(
        lineage.layer,
        pse_model::generated::enums::ModelingValidityLayer::Closure
    );
    assert!(checked_sources.contains_key(&lineage.source));
    assert_eq!(lineage.members.len(), 1);
    assert_eq!(checked_members[&lineage.members[0]], source.unknowns[0].id);
    assert_eq!(instance, source.bindings.instance());
    assert_eq!(&checked_members, source.bindings.checked_members());
    assert_eq!(&checked_sources, source.bindings.checked_sources());
    assert_eq!(&occurrences, source.bindings.occurrences());
    for missing in 0..4 {
        let mut wire = SourceWire::capture(&source.bindings);
        match missing {
            0 => wire.checked_members.clear(),
            1 => wire.checked_sources.clear(),
            2 => wire.occurrences.clear(),
            _ => wire.instance = SemanticId::NIL,
        }
        assert!(
            wire.bind(source.body.math()).is_err(),
            "missing actual source slot {missing}"
        );
    }
}

#[test]
fn native_child_value_source_keeps_unknown_jacobian_without_parameter_jet_preparation() {
    let source = source();
    let scope = ExecutionScope::new(Cancellation::default(), None);
    let problem = source.problem(&scope).unwrap();
    assert_eq!(problem.native_order(), DerivativeOrder::First);
    let values = problem
        .evaluate(
            &source.parameters,
            &[2.0],
            DerivativeOrder::Value,
            scope.cancellation(),
        )
        .unwrap();
    assert_eq!(values.values, [0.0]);
    assert!(values.jacobian.is_empty() && values.hessians.is_empty());
    let unknown = problem
        .evaluate_unknown(
            &source.parameters,
            &[2.0],
            DerivativeOrder::First,
            scope.cancellation(),
        )
        .unwrap();
    assert_eq!(unknown.jacobian, [1.0]);
    let error = problem
        .evaluate(
            &source.parameters,
            &[2.0],
            DerivativeOrder::First,
            scope.cancellation(),
        )
        .unwrap_err();
    assert!(matches!(error, pse_math::MathError::Contract(message)
        if message == "complete implicit product was not prepared before numerical execution"));
    // A genuine full consumer may prepare its complete product before callbacks.
    problem
        .prepare_complete(DerivativeOrder::First, &scope)
        .unwrap();
    let complete = problem
        .evaluate(
            &source.parameters,
            &[2.0],
            DerivativeOrder::First,
            scope.cancellation(),
        )
        .unwrap();
    assert_eq!(complete.jacobian, [1.0, -1.0]);
}

#[test]
fn native_child_source_requires_actual_physical_registry_before_launch() {
    let mut source = source();
    source.context.quantities = Arc::new(
        pse_quantity::QuantityRegistryBuilder::new()
            .build()
            .unwrap(),
    );
    assert!(
        PortableRootSource::new(
            source.body,
            source.bindings,
            source.context,
            source.unknowns,
            source.rows,
            source.parameters,
            source.options,
            source.evaluation,
            source.max_entries
        )
        .is_err()
    );
}

#[test]
fn native_child_descriptor_only_provider_is_not_a_reconstructible_factory() {
    let mut source = source();
    let quantity = source.body.quantities()[0];
    let unit = source
        .context
        .quantities
        .quantity_type(quantity)
        .unwrap()
        .canonical_unit;
    let descriptor = pse_kernels::AdmittedProvider::new(
        pse_kernels::ProviderSpec {
            shapes: Default::default(),
            derivative_source: pse_kernels::DerivativeSource::Analytic,
            id: SemanticId::from_bytes([192; 16]),
            revision: ContentHash::from_bytes([193; 32]),
            data: ContentHash::from_bytes([194; 32]),
            inputs: vec![],
            outputs: vec![pse_kernels::Port {
                id: SemanticId::from_bytes([195; 16]),
                quantity,
                unit,
            }],
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        },
        &source.context.quantities,
    )
    .unwrap();
    source.context.providers.insert(
        "opaque".into(),
        pse_compiler::typed_math::ProviderCall {
            descriptor,
            output: 0,
        },
    );
    assert!(matches!(
        PortableRootSource::new(
            source.body,
            source.bindings,
            source.context,
            source.unknowns,
            source.rows,
            source.parameters,
            source.options,
            source.evaluation,
            source.max_entries
        ),
        Err(MathRuntimeError::Solve(
            pse_backend_native::ProblemError::Contract(_)
        ))
    ));
}

// The actual private binary is an explicit executable prerequisite supplied by the
// integrated selector. Never discover a stale PATH binary or build one within a test.
#[tokio::test]
async fn actual_portable_native_root_child_reconstructs_and_parent_screens() {
    // Ambient exporter selection alone does not initialize a test process host.
    // The external acceptance driver explicitly selects this composition and supplies
    // an independently chosen W3C parent. Ordinary scientific runs keep their baseline.
    let telemetry = if std::env::var("PSE_NATIVE_TELEMETRY_ACCEPTANCE").as_deref() == Ok("1") {
        Some(
            std::thread::Builder::new()
                .name("native-child-acceptance-telemetry".into())
                .spawn(|| {
                    crate::tooling::telemetry::Host::from_environment(
                        "pse-native-parent-acceptance",
                    )
                })
                .unwrap()
                .join()
                .unwrap()
                .unwrap()
                .expect("telemetry acceptance explicitly requires enabled resolved host settings"),
        )
    } else {
        None
    };
    if let Some(host) = telemetry {
        let carrier = crate::tooling::telemetry::Carrier::from_json(
            &std::env::var("PSE_NATIVE_TELEMETRY_ACCEPTANCE_PARENT")
                .expect("telemetry acceptance supplies an independent bounded W3C parent"),
        )
        .unwrap();
        let span = carrier.operation_span("native.child.actual_acceptance");
        tracing::Instrument::instrument(actual_portable_native_root_fixture(), span).await;
        // Every original solve result, reservation and actual process-reap assertion
        // above has settled before optional parent-host SDK teardown. Export failure
        // is a separate diagnostic fact and cannot rewrite that scientific outcome.
        let shutdown = std::thread::Builder::new()
            .name("native-child-acceptance-telemetry-shutdown".into())
            .spawn(move || host.shutdown())
            .unwrap()
            .join()
            .unwrap();
        if shutdown.is_err() {
            tracing::warn!(target: "pse.telemetry.diagnostic", code = "telemetry.acceptance.shutdown_failed", "optional acceptance export shutdown failed after native settlement");
        }
    } else {
        actual_portable_native_root_fixture().await;
    }
}

async fn actual_portable_native_root_fixture() {
    let executable = std::env::var_os("PSE_NATIVE_LANE_EXECUTABLE")
        .expect("integrator supplies exact built pse-native-lane executable");
    let (service, _) = super::super::tests::service_with_policy(
        512 << 20,
        super::super::MathPolicy {
            worker_bytes: 256 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let baseline = service.pool.reserved();
    let child = NativeChild::qualify(executable, 128 << 20).unwrap();
    for target in [2.0, 4.0, 2.0] {
        // These are acceptance references issued by the actual reaped result, not
        // guessed process associations or replacement scientific identifiers.
        let case = tracing::info_span!(
            "pse.native.child.acceptance.case",
            acceptance.target = target,
            native.child.pid = tracing::field::Empty,
            native.attempt = tracing::field::Empty,
        );
        tracing::Instrument::instrument(
            async {
                let mut input = source();
                input.parameters[0] = target;
                let handle = service
                    .solve_portable_root(input, child.clone(), Cancellation::default(), None)
                    .unwrap();
                let (result, lease) = handle.finish().await.unwrap();
                assert_ne!(result.child_pid, std::process::id());
                assert!(result.refusal.is_none(), "{:?}", result.refusal);
                assert!((result.proposal.as_ref().unwrap()[0] - target).abs() < 1e-8);
                assert!(
                    !Path::new(&format!("/proc/{}", result.child_pid)).exists(),
                    "result cannot precede actual child reap"
                );
                case.record("native.child.pid", result.child_pid);
                case.record("native.attempt", tracing::field::display(result.attempt));
                drop((result, lease));
                assert_eq!(service.pool.reserved(), baseline);
            },
            case.clone(),
        )
        .await;
    }
}

#[test]
#[expect(
    clippy::exit,
    reason = "the child-process death control deliberately exits with code 17 to test supervision"
)]
#[allow(
    unsafe_code,
    reason = "the owned child fixture invokes the private inherited-socket composition root"
)]
fn private_native_fake_process() {
    let Ok(mode) = std::env::var("PSE_PRIVATE_NATIVE_TEST_MODE") else {
        return;
    };
    let marker = std::env::var_os("PSE_PRIVATE_NATIVE_TEST_MARKER").unwrap();
    std::fs::write(marker, std::process::id().to_string()).unwrap();
    if mode == "death" {
        std::process::exit(17);
    }
    let mut channel = UnixStream::from(io::stdin().as_fd().try_clone_to_owned().unwrap());
    let scope = ExecutionScope::new(Cancellation::default(), None);
    let attempt =
        SemanticId::parse_hex(&std::env::var("PSE_PRIVATE_NATIVE_TEST_ATTEMPT").unwrap()).unwrap();
    let parent_record = std::fs::read_to_string("/proc/self/status").unwrap();
    let parent = parent_record
        .lines()
        .find_map(|line| line.strip_prefix("PPid:"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    if mode == "actual" {
        // SAFETY: this fixture owns the exact inherited socket, parent and admitted request;
        // the production receiver performs its ordinary reconstruction and handshake checks.
        unsafe { serve_private(attempt, parent) }.unwrap();
        return;
    }
    let diagnostic = crate::observation::process_diagnostics()
        .hub()
        .start("native.child.fixture");
    let hello = Hello {
        diagnostic_operation: diagnostic.context().id(),
        protocol: PROTOCOL.into(),
        attempt: if mode == "foreign" {
            SemanticId::NIL
        } else {
            attempt
        },
        pid: std::process::id(),
        parent,
        executable: pse_buildinfo::FileObservation::capture(&std::env::current_exe().unwrap())
            .unwrap(),
        build: pse_buildinfo::BuildInfo::captured(),
        native_build: if mode == "native" {
            ContentHash::from_bytes([0; 32])
        } else {
            native_profile().unwrap()
        },
    };
    send(&mut channel, &hello, RESPONSE_LIMIT, &scope).unwrap();
    if mode == "foreign" {
        return;
    }
    let request: Request = receive(&mut channel, REQUEST_LIMIT, &scope).unwrap();
    if mode == "oversize" {
        channel
            .write_all(&((RESPONSE_LIMIT + 1) as u64).to_le_bytes())
            .unwrap();
        return;
    }
    if mode == "truncated" {
        channel.write_all(&128_u64.to_le_bytes()).unwrap();
        channel.write_all(b"{").unwrap();
        return;
    }
    if mode == "late" {
        std::thread::sleep(Duration::from_secs(30));
    }
    let point = if mode == "invalid" {
        vec![9.0]
    } else {
        vec![2.0]
    };
    let _ = send(
        &mut channel,
        &Response {
            attempt,
            execution: request.execution,
            request: if mode == "response" {
                ContentHash::from_bytes([0; 32])
            } else {
                request.request
            },
            result: Ok(point),
        },
        RESPONSE_LIMIT,
        &scope,
    );
}

fn fake_child(mode: &str, marker: &Path) -> NativeChild {
    let mut child = NativeChild::qualify(std::env::current_exe().unwrap(), 128 << 20).unwrap();
    child.test_mode = Some((mode.into(), marker.to_path_buf()));
    child
}

#[tokio::test]
async fn actual_private_child_publishes_a_distinct_parent_linked_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
    let parent = hub.start("native.parent.fixture");
    let (service, _) = super::super::tests::service_with_policy(
        512 << 20,
        super::super::MathPolicy {
            worker_bytes: 256 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let child = fake_child("actual", &directory.path().join("child.pid"));
    let (result, lease) = pse_diagnostics::observation::scope(Some(parent.context()), async {
        service
            .solve_portable_root(source(), child, Cancellation::default(), None)
            .unwrap()
            .finish()
            .await
            .unwrap()
    })
    .await;
    assert!(result.refusal.is_none(), "{:?}", result.refusal);
    assert!((result.proposal.as_ref().unwrap()[0] - 2.0).abs() < 1e-8);
    assert!(!Path::new(&format!("/proc/{}", result.child_pid)).exists());
    let remote = hub
        .list(0, 128)
        .operations
        .iter()
        .filter_map(|operation| hub.reader(operation.id))
        .flat_map(|reader| reader.snapshot(Duration::from_secs(60)).children)
        .find(|child| child.id.process_instance != hub.process_instance())
        .unwrap()
        .id;
    let root = crate::observation::process_diagnostics()
        .checkpoint_path()
        .parent()
        .unwrap()
        .to_owned();
    let path = root.join(format!(
        "{:016x}-{:016x}.json",
        remote.process_instance[0], remote.process_instance[1]
    ));
    let image = crate::observation::read_checkpoint(&path).unwrap();
    assert!(
        image
            .parent
            .is_some_and(|id| id.process_instance == hub.process_instance())
    );
    let operation = image
        .operations
        .iter()
        .find(|operation| operation.id == remote)
        .unwrap();
    assert_eq!(
        operation.outcome,
        Some(pse_diagnostics::observation::Outcome::Completed)
    );
    assert!(
        operation
            .metrics
            .iter()
            .any(|metric| metric.name == "native.response.sent"
                && metric.latest == pse_diagnostics::observation::Value::Boolean(true))
    );
    // This exact child is reaped; remove only its owned diagnostic checkpoint.
    std::fs::remove_file(path).unwrap();
    drop((result, lease));
}

#[tokio::test]
async fn private_native_process_death_foreign_attempt_and_original_residual_refuse_after_reap() {
    let (service, _) = super::super::tests::service_with_policy(
        512 << 20,
        super::super::MathPolicy {
            worker_bytes: 256 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let baseline = service.pool.reserved();
    for mode in [
        "death",
        "foreign",
        "native",
        "response",
        "oversize",
        "truncated",
        "invalid",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("owned-child");
        let handle = service
            .solve_portable_root(
                source(),
                fake_child(mode, &marker),
                Cancellation::default(),
                None,
            )
            .unwrap();
        let result = handle.finish().await;
        assert!(
            result.is_err(),
            "{mode}: a dead, foreign or physically invalid proposal cannot publish"
        );
        let pid: u32 = std::fs::read_to_string(&marker).unwrap().parse().unwrap();
        assert!(
            !Path::new(&format!("/proc/{pid}")).exists(),
            "{mode}: failure returned before actual reap"
        );
        assert_eq!(
            service.pool.reserved(),
            baseline,
            "{mode}: owned parent charges released after reap"
        );
    }
}

#[tokio::test]
async fn private_native_queue_wait_uses_original_clock_without_launch_or_cancellation() {
    let (service, _) = super::super::tests::service_with_policy(
        512 << 20,
        super::super::MathPolicy {
            worker_bytes: 256 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("must-not-launch");
    let child = fake_child("late", &marker);
    let occupied = service.cpu.clone().acquire_many_owned(2).await.unwrap();
    let control = Cancellation::default();
    let handle = service
        .solve_portable_root(
            source(),
            child,
            control.clone(),
            Some(Instant::now() + Duration::from_millis(20)),
        )
        .unwrap();
    assert!(handle.finish().await.is_err());
    assert!(!marker.exists());
    assert!(!control.flag().load(std::sync::atomic::Ordering::Acquire));
    drop(occupied);
    assert_eq!(service.pool.reserved(), 0);
}

#[tokio::test]
async fn private_native_late_output_and_cancellation_hold_parent_charges_until_reap() {
    let (service, _) = super::super::tests::service_with_policy(
        512 << 20,
        super::super::MathPolicy {
            worker_bytes: 256 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let baseline = service.pool.reserved();
    for cancel_owned in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("owned-child");
        let control = Cancellation::default();
        let deadline = if cancel_owned {
            None
        } else {
            Some(Instant::now() + Duration::from_secs(10))
        };
        let handle = service
            .solve_portable_root(
                source(),
                fake_child("late", &marker),
                control.clone(),
                deadline,
            )
            .unwrap();
        for _ in 0..10000 {
            if marker.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        let pid: u32 = std::fs::read_to_string(&marker).unwrap().parse().unwrap();
        assert!(Path::new(&format!("/proc/{pid}")).exists());
        assert!(
            service.pool.reserved() > baseline,
            "live child keeps parent admission charges"
        );
        if cancel_owned {
            control.cancel();
        }
        assert!(
            handle.finish().await.is_err(),
            "late output cannot acquire proposal permission"
        );
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(
            control.flag().load(std::sync::atomic::Ordering::Acquire),
            cancel_owned,
            "deadline never masquerades as cancellation"
        );
    }
}

#[tokio::test]
async fn native_child_strict_complete_memory_selection_refuses_before_launch() {
    let (service, _) = super::super::tests::service_with_policy(
        512 << 20,
        super::super::MathPolicy {
            worker_bytes: 256 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let baseline = service.pool.reserved();
    let mut child = NativeChild::qualify(std::env::current_exe().unwrap(), 128 << 20).unwrap();
    child.require_complete_memory_bound = true;
    assert!(matches!(
        service.solve_portable_root(source(), child, Cancellation::default(), None),
        Err(MathRuntimeError::Solve(
            pse_backend_native::ProblemError::Unsupported(_)
        ))
    ));
    assert_eq!(service.pool.reserved(), baseline);
}

#[test]
#[cfg(target_os = "linux")]
fn private_native_readiness_wakes_on_data_hup_cancel_and_original_deadline() {
    use rustix::event::PollFlags;
    for cancel_before in [false, true] {
        let (reader, _writer) = UnixStream::pair().unwrap();
        let stop = Cancellation::new(cancel_before);
        let scope =
            ExecutionScope::new(stop.clone(), Some(Instant::now() + Duration::from_secs(5)));
        let (started, entered) = std::sync::mpsc::sync_channel(1);
        let join = std::thread::spawn(move || {
            started.send(()).unwrap();
            wait_ready(&scope, &reader, PollFlags::IN)
        });
        entered.recv().unwrap();
        stop.cancel();
        assert!(join.join().unwrap().is_err());
    }
    for close in [false, true] {
        let (reader, mut writer) = UnixStream::pair().unwrap();
        let scope = ExecutionScope::new(
            Cancellation::default(),
            Some(Instant::now() + Duration::from_secs(5)),
        );
        let join = std::thread::spawn(move || wait_ready(&scope, &reader, PollFlags::IN));
        if close {
            drop(writer)
        } else {
            writer.write_all(&[42]).unwrap()
        }
        assert!(join.join().unwrap().is_ok());
    }
    let (reader, _writer) = UnixStream::pair().unwrap();
    let stop = Cancellation::default();
    let scope = ExecutionScope::new(stop.clone(), Some(Instant::now()));
    assert!(wait_ready(&scope, &reader, PollFlags::IN).is_err());
    assert!(!stop.is_cancelled());
}
