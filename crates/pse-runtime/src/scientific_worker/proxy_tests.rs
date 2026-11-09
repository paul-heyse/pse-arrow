// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual-process failure controls through the production generated transport.
use super::*;
use datafusion::execution::memory_pool::{FairSpillPool, MemoryPool};
use pse_kernels::{DerivativeOrder, ProviderBuffer};
use std::io::Write as _;
use std::os::unix::net::UnixDatagram;

const DISPATCHED: &[u8] = b"native-dispatched";

fn observe_phase(socket: &UnixDatagram, expected: &[u8]) -> std::io::Result<()> {
    let mut received = [0_u8; 128];
    assert!(expected.len() < received.len());
    let length = socket.recv(&mut received)?;
    if &received[..length] == expected {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unexpected native worker phase",
        ))
    }
}

#[test]
fn cancellation_dispatch_gate_refuses_wrong_and_missing_phase() {
    let (owner, observer) = UnixDatagram::pair().unwrap();
    observer.set_nonblocking(true).unwrap();
    assert_eq!(
        observe_phase(&observer, DISPATCHED).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    owner.send(b"prepared").unwrap();
    assert_eq!(
        observe_phase(&observer, DISPATCHED).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
    owner.send(DISPATCHED).unwrap();
    observe_phase(&observer, DISPATCHED).unwrap();
}

fn original_provider_error(error: &ProviderError) -> &ProviderError {
    fn original_worker(error: &WorkerError) -> Option<&ProviderError> {
        match error {
            WorkerError::Provider(error) => Some(original_provider_error(error)),
            WorkerError::Retirement { causes } => causes.first().and_then(original_worker),
            _ => None,
        }
    }
    match error {
        ProviderError::Nested { cause, .. } => cause
            .as_error()
            .downcast_ref::<WorkerError>()
            .and_then(original_worker)
            .unwrap_or(error),
        _ => error,
    }
}
fn original_worker_error(error: &WorkerError) -> &WorkerError {
    match error {
        WorkerError::Retirement { causes } => original_worker_error(causes.first().unwrap()),
        _ => error,
    }
}
fn original_remote_error(error: &ProviderError) -> &WorkerError {
    let ProviderError::Nested { cause, .. } = error else {
        panic!("typed remote error required");
    };
    original_worker_error(cause.as_error().downcast_ref::<WorkerError>().unwrap())
}

#[derive(Debug)]
struct ControlledFactory {
    spec: ProviderSpec,
    configuration: ContentHash,
    mode: String,
    marker: std::path::PathBuf,
    generation: SemanticId,
}
impl ProviderFactory for ControlledFactory {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn configuration_key(&self) -> ContentHash {
        self.configuration
    }
    fn create(&self) -> Result<ProviderBinding, ProviderError> {
        Ok(ProviderBinding::new(
            self.spec.clone(),
            Box::new(ControlledNative {
                mode: self.mode.clone(),
                marker: self.marker.clone(),
                generation: self.generation,
                refused_preparation: false,
            }),
        ))
    }
}
#[derive(Debug)]
struct ControlledNative {
    mode: String,
    marker: std::path::PathBuf,
    generation: SemanticId,
    refused_preparation: bool,
}
impl Provider for ControlledNative {
    fn finish(self: Box<Self>) -> Result<(), ProviderError> {
        let mut expired = false;
        if matches!(
            self.mode.as_str(),
            "evaluate-departure-retire-faults" | "prepare-refusal-departure-retire-faults"
        ) {
            let generation = self.generation.to_string();
            writeln!(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(self.marker.with_extension("finished"))
                    .unwrap(),
                "{generation}"
            )
            .unwrap();
            expired = std::fs::read_to_string(self.marker.with_extension("expiring-generation"))
                .is_ok_and(|selected| selected == generation);
        }
        if matches!(
            self.mode.as_str(),
            "prepare-departure-retire-faults"
                | "shutdown-departure-retire-faults"
                | "close-departure-retire-faults"
        ) || expired
            || self.refused_preparation
        {
            writeln!(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(self.marker.with_extension("failed-finishes"))
                    .unwrap(),
                "{}",
                self.generation,
            )
            .unwrap();
            return Err(ProviderError::Contract(format!(
                "controlled native retirement failure {}",
                self.generation
            )));
        }
        if matches!(self.mode.as_str(), "close-pending" | "prepare-departure") {
            if self.mode == "close-pending" {
                assert!(
                    self.marker.with_extension("returned").exists(),
                    "native finish must follow the in-flight fill's actual return"
                );
            }
            writeln!(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(self.marker.with_extension("finished"))
                    .unwrap(),
                "finished"
            )
            .unwrap();
        }
        if self.mode == "finish-native-faults" {
            writeln!(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(self.marker.with_extension("finished"))
                    .unwrap(),
                "finished"
            )
            .unwrap();
            return Err(ProviderError::Contract(
                "controlled native finish failure".into(),
            ));
        }
        Ok(())
    }
    fn prepare_demand(&mut self, request: &PreparedProviderRequest) -> Result<(), ProviderError> {
        if self.mode == "prepare-refusal-departure-retire-faults"
            && request.order == DerivativeOrder::Second
        {
            self.refused_preparation = true;
            return Err(ProviderError::Contract(
                "controlled original provider admission refusal".into(),
            ));
        }
        Ok(())
    }
    fn supports_batch(&self, request: &PreparedProviderRequest) -> bool {
        self.mode == "batch-cooperative"
            && request.order == DerivativeOrder::Value
            && matches!(request.demand, ProviderDemand::CompleteJets)
    }
    fn fill_batch(
        &mut self,
        points: &[&ProviderBatchPoint],
        _request: &PreparedProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderBatchResults, ProviderError> {
        assert_eq!(points.len(), 2, "one controlled group owns both points");
        std::fs::write(&self.marker, b"one native group active").unwrap();
        loop {
            if let Err(error) = context.check() {
                std::fs::write(
                    self.marker.with_extension("settled"),
                    b"native group returned",
                )
                .unwrap();
                return Err(error);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn supports(&self, demand: &ProviderDemand) -> bool {
        matches!(self.mode.as_str(), "actions" | "actions-cooperative")
            || matches!(demand, ProviderDemand::CompleteJets)
    }
    fn fill(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        let mut marker = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.marker)
            .unwrap();
        writeln!(marker, "dispatched").unwrap();
        marker.sync_all().unwrap();
        if self.mode == "crash" {
            std::process::exit(73);
        }
        if self.mode == "cancel-dispatch" {
            UnixDatagram::unbound()
                .unwrap()
                .send_to(DISPATCHED, self.marker.with_extension("dispatch"))
                .unwrap();
            loop {
                std::thread::park();
            }
        }
        if self.mode == "delay" {
            std::thread::sleep(Duration::from_secs(30));
        }
        if self.mode == "close-pending" && inputs[0] == 13. {
            let gate = UnixDatagram::bind(self.marker.with_extension("close-release")).unwrap();
            gate.set_read_timeout(Some(Duration::from_secs(15)))
                .unwrap();
            gate.send_to(b"active", self.marker.with_extension("close-events"))
                .unwrap();
            observe_phase(&gate, b"release").unwrap();
            context.check()?;
            std::fs::write(
                self.marker.with_extension("returned"),
                b"native fill returned",
            )
            .unwrap();
        }
        if self.mode == "actions-cooperative" && inputs[0] == 13. {
            std::fs::write(self.marker.with_extension("active"), b"native active").unwrap();
            loop {
                context.check()?;
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if matches!(self.mode.as_str(), "actions" | "actions-cooperative") {
            return polynomial_products(inputs, request, runtime, output);
        }
        output.values.fill(1.0);
        output.jacobian.fill(2.0);
        output.hessians.fill(3.0);
        Ok(())
    }
}

fn polynomial_products(
    inputs: &[f64],
    request: &PreparedProviderRequest,
    runtime: ProviderRuntime<'_>,
    output: ProviderOutput<'_>,
) -> Result<(), ProviderError> {
    let [x, y] = inputs else {
        return Err(ProviderError::Contract(
            "controlled polynomial inputs".into(),
        ));
    };
    let values = [x * x + y, x * y, y * y];
    let first = |row, column| match (row, column) {
        (0, 0) => 2.0 * x,
        (0, 1) => 1.0,
        (1, 0) => *y,
        (1, 1) => *x,
        (2, 0) => 0.0,
        (2, 1) => 2.0 * y,
        _ => panic!("controlled polynomial derivative indices must be admitted"),
    };
    let second = |row, a, b| match (row, a, b) {
        (0, 0, 0) | (2, 1, 1) => 2.0,
        (1, 0, 1) | (1, 1, 0) => 1.0,
        _ => 0.0,
    };
    let m = request.outputs.len();
    for (position, &row) in request.outputs.iter().enumerate() {
        output.values[position] = values[row];
    }
    match &request.demand {
        ProviderDemand::CompleteJets => {
            for (position, &row) in request.outputs.iter().enumerate() {
                if request.order >= DerivativeOrder::First {
                    for column in 0..2 {
                        output.jacobian[position * 2 + column] = first(row, column);
                    }
                }
                if request.order == DerivativeOrder::Second {
                    for a in 0..2 {
                        for b in 0..2 {
                            output.hessians[position * 4 + a * 2 + b] = second(row, a, b);
                        }
                    }
                }
            }
        }
        ProviderDemand::SelectedColumns { columns } => {
            for (position, &row) in request.outputs.iter().enumerate() {
                for (column, &source) in columns.iter().enumerate() {
                    output.jacobian[position * columns.len() + column] = first(row, source);
                }
                if request.order == DerivativeOrder::Second {
                    for (a, &source_a) in columns.iter().enumerate() {
                        for (b, &source_b) in columns.iter().enumerate() {
                            output.hessians[position * columns.len() * columns.len()
                                + a * columns.len()
                                + b] = second(row, source_a, source_b);
                        }
                    }
                }
            }
        }
        ProviderDemand::JacobianAction { directions } => {
            for (position, &row) in request.outputs.iter().enumerate() {
                for direction in 0..*directions {
                    output.jacobian[position * directions + direction] = first(row, 0)
                        * runtime.directions[direction * 2]
                        + first(row, 1) * runtime.directions[direction * 2 + 1];
                }
            }
        }
        ProviderDemand::TransposeAction { cotangents } => {
            for cotangent in 0..*cotangents {
                for column in 0..2 {
                    output.jacobian[cotangent * 2 + column] = request
                        .outputs
                        .iter()
                        .enumerate()
                        .map(|(position, &row)| {
                            runtime.cotangents[cotangent * m + position] * first(row, column)
                        })
                        .sum();
                }
            }
        }
        ProviderDemand::WeightedSecondAction { directions } => {
            for column in 0..2 {
                output.jacobian[column] = request
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(position, &row)| runtime.weights[position] * first(row, column))
                    .sum();
                for direction in 0..*directions {
                    output.hessians[column * directions + direction] = request
                        .outputs
                        .iter()
                        .enumerate()
                        .map(|(position, &row)| {
                            runtime.weights[position]
                                * (second(row, column, 0) * runtime.directions[direction * 2]
                                    + second(row, column, 1)
                                        * runtime.directions[direction * 2 + 1])
                        })
                        .sum();
                }
            }
        }
    }
    Ok(())
}

#[test]
fn controlled_child() {
    let Ok(mode) = std::env::var("PSE_SCIENCE_TEST_MODE") else {
        return;
    };
    let marker = std::path::PathBuf::from(std::env::var_os("PSE_SCIENCE_TEST_MARKER").unwrap());
    let endpoint = std::path::PathBuf::from(std::env::var_os("PSE_SCIENCE_TEST_ENDPOINT").unwrap());
    let session =
        SemanticId::parse_hex(&std::env::var("PSE_SCIENCE_TEST_SESSION").unwrap()).unwrap();
    let parent = std::env::var("PSE_SCIENCE_TEST_PARENT")
        .unwrap()
        .parse()
        .unwrap();
    super::super::transport::serve_with_preparation(
        endpoint,
        session,
        parent,
        move |request, cancellation| {
            if request.handle.session != session {
                return Err(ProviderError::Contract("foreign controlled session".into()).into());
            }
            let registry = request.registry.build()
                .map_err(|error| ProviderError::Contract(error.to_string()))?;
            let regime = match request.regime.as_str() {
                "liquid" => WaterRegime::Liquid,
                "vapor" => WaterRegime::Vapor,
                "supercritical" => WaterRegime::Supercritical,
                _ => return Err(ProviderError::Contract("controlled water regime".into()).into()),
            };
            let spec = if mode.starts_with("actions") {
                action_spec(&registry)
            } else {
                pse_backend_native::coolprop::descriptor(&registry, regime)?.spec().clone()
            };
            let admitted = pse_kernels::AdmittedProvider::new(spec.clone(), &registry)?;
            if pse_kernels::portable::ProviderRecord::capture(&admitted) != request.spec {
                return Err(ProviderError::Contract("controlled native descriptor differs".into()).into());
            }
            let checked = PreparedProviderRequest::new(Arc::new(spec.clone()), request.request)?;
            if mode.starts_with("water-batch") {
                let regime = match request.regime.as_str() {
                    "liquid" => WaterRegime::Liquid,
                    "vapor" => WaterRegime::Vapor,
                    _ => return Err(ProviderError::Contract("controlled water regime".into()).into()),
                };
                let scope = handler::scientific_scope(cancellation, request.remaining_ns, Instant::now())?;
                scope.check()?;
                let factory = WaterFactory::prepare(&registry, regime)?;
                return super::super::PreparedSession::prepare(request.handle, &factory,
                    request.configuration, checked, scope, request.result_limit as usize);
            }
            let factory = ControlledFactory {
                spec: spec.clone(),
                configuration: request.configuration,
                mode: mode.clone(),
                marker: marker.clone(),
                generation: request.handle.generation,
            };
            let gated = mode.starts_with("prepare-departure") && checked.order == DerivativeOrder::Second;
            let generation = request.handle.generation;
            let prepared = super::super::PreparedSession::prepare(
                request.handle,
                &factory,
                request.configuration,
                checked,
                handler::scientific_scope(cancellation, request.remaining_ns, Instant::now())?,
                request.result_limit as usize,
            )?;
            if gated {
                std::fs::write(marker.with_extension("prepare-built"), generation.to_string()).unwrap();
                let end = Instant::now() + Duration::from_secs(10);
                while !marker.with_extension("prepare-release").exists() {
                    assert!(Instant::now() < end, "caller did not release actual preparation gate");
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            Ok(prepared)
        },
    )
    .unwrap();
}

#[test]
fn scientific_scope_preserves_unbounded_and_long_original_lifetimes_without_renewal() {
    let control = pse_diagnostics::Cancellation::default();
    let prepared_at = Instant::now() - Duration::from_secs(601);
    let unbounded = ExecutionScope::new(control.clone(), None);
    let remaining = scientific_remaining_ns(&unbounded, prepared_at).unwrap();
    assert_eq!(remaining, None);
    let child = handler::scientific_scope(control.clone(), remaining, prepared_at).unwrap();
    assert_eq!(child.deadline(), None);
    child.check().unwrap();

    let original_deadline = prepared_at + Duration::from_secs(1800);
    let original = ExecutionScope::new(control.clone(), Some(original_deadline));
    let remaining = scientific_remaining_ns(&original, prepared_at).unwrap();
    assert_eq!(remaining, Some(1_800_000_000_000));
    let child = handler::scientific_scope(control.clone(), remaining, prepared_at).unwrap();
    assert_eq!(child.deadline(), Some(original_deadline));
    child.check().unwrap();
    // A second preparation uses the remaining original clock, not a fresh 1800s.
    let reused_at = prepared_at + Duration::from_secs(601);
    let remaining = scientific_remaining_ns(&original, reused_at).unwrap();
    assert_eq!(remaining, Some(1_199_000_000_000));
    assert_eq!(
        handler::scientific_scope(control.clone(), remaining, reused_at)
            .unwrap()
            .deadline(),
        Some(original_deadline)
    );

    let expired =
        handler::scientific_scope(control.clone(), Some(600_000_000_000), prepared_at).unwrap();
    assert!(matches!(expired.check(), Err(ProviderError::Deadline)));
    assert!(!control.flag().load(std::sync::atomic::Ordering::Acquire));
}

fn service() -> (
    Arc<crate::math::MathService>,
    Arc<dyn MemoryPool>,
    Arc<pse_engine::cache_service::NativeCacheService>,
) {
    let pool: Arc<dyn MemoryPool> = Arc::new(pse_engine::resources::ReleaseNotifyingPool::new(
        Arc::new(FairSpillPool::new(1 << 30)),
    ));
    let native = pse_engine::cache_service::NativeCacheService::new(
        pse_engine::cache_service::CacheBudget::disabled(1024),
        &pool,
    )
    .unwrap();
    let math = crate::math::MathService::new(
        pool.clone(),
        Arc::new(pse_engine::resources::ReleaseNotifyingSemaphore::new(16)),
        16,
        crate::math::MathPolicy {
            workspace_bytes: 16 << 20,
            ..Default::default()
        },
        &native,
    );
    (math, pool, native)
}

#[test]
fn opaque_local_error_extent_stays_unknown_and_finite_retention_refuses_its_sentinel() {
    let original = WorkerError::Process {
        action: "controlled error",
        source: std::io::Error::other("x".repeat(1 << 20)),
    };
    assert_eq!(original.retained_bytes(), None);
    let aggregate = WorkerError::Retirement {
        causes: vec![
            ProviderError::Trial("original trial".into()).into(),
            original,
        ],
    };
    assert_eq!(aggregate.retained_bytes(), None);
    let provider = aggregate.into_provider();
    let ProviderError::Nested { cause, retained } = &provider else {
        panic!("original typed cause must stay owned");
    };
    assert_eq!(
        *retained,
        usize::MAX,
        "unknown extent is a conservative refusal sentinel"
    );
    let WorkerError::Retirement { causes } =
        cause.as_error().downcast_ref::<WorkerError>().unwrap()
    else {
        panic!("original aggregate must stay typed");
    };
    let WorkerError::Process { source, .. } = &causes[1] else {
        panic!("original I/O error must stay typed");
    };
    assert_eq!(source.to_string().len(), 1 << 20);
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    assert!(
        service
            .reserve(
                "controlled-unknown-error-retention",
                provider.retained_bytes()
            )
            .is_err()
    );
    assert_eq!(pool.reserved(), baseline);
}

fn provider(
    mode: &str,
    marker: &std::path::Path,
    scope: ExecutionScope,
    service: &Arc<crate::math::MathService>,
    pool: &Arc<dyn MemoryPool>,
) -> (ProviderBinding, PreparedProviderRequest, u32, usize) {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let image = PreparedExecutable::observe(std::env::current_exe().unwrap(), 64 << 20)
        .unwrap()
        .test_entry(
            "scientific_worker::proxy::tests::controlled_child",
            mode,
            marker.to_path_buf(),
        );
    let baseline = pool.reserved();
    let factory =
        LocalWaterFactory::prepare(image, &registry, WaterRegime::Vapor, service.clone()).unwrap();
    let prepared_reservation = pool.reserved();
    assert!(
        prepared_reservation > baseline,
        "physical declaration storage is admitted before launch"
    );
    let raw = ProviderRequest::all(factory.spec(), DerivativeOrder::Second);
    let prepared =
        PreparedProviderRequest::new(Arc::new(factory.spec().clone()), raw.clone()).unwrap();
    let mut proxy = Proxy {
        factory: factory.clone(),
        scope,
        attempt: pse_operations::mint_id(),
        remote: None,
        prepared: vec![],
        invocation: 0,
        failed: false,
    };
    proxy.prepare_demand(&prepared).unwrap();
    let pid = proxy.remote.as_ref().unwrap().child.pid();
    let mut binding = ProviderBinding::new(factory.spec().clone(), Box::new(proxy));
    let request = binding.prepare(raw).unwrap();
    (binding, request, pid, prepared_reservation)
}

fn finish_proxy(
    mode: &str,
    marker: &std::path::Path,
    service: &Arc<crate::math::MathService>,
) -> Proxy {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let executable = PreparedExecutable::observe(std::env::current_exe().unwrap(), 64 << 20)
        .unwrap()
        .test_entry(
            "scientific_worker::proxy::tests::controlled_child",
            mode,
            marker.to_path_buf(),
        );
    let factory =
        LocalWaterFactory::prepare(executable, &registry, WaterRegime::Vapor, service.clone())
            .unwrap();
    let mut proxy = Proxy {
        factory,
        scope: ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
        attempt: pse_operations::mint_id(),
        remote: None,
        prepared: vec![],
        invocation: 0,
        failed: false,
    };
    for order in [DerivativeOrder::Value, DerivativeOrder::First] {
        let request = PreparedProviderRequest::new(
            Arc::new(proxy.factory.spec().clone()),
            ProviderRequest::all(proxy.factory.spec(), order),
        )
        .unwrap();
        proxy.prepare_demand(&request).unwrap();
    }
    proxy
}

#[test]
fn shutdown_peer_exits_with_client_sender_live_before_parent_io_retirement() {
    use pse_diagnostics::process_facts::{ProcessRead, ProcessReader};
    let directory = tempfile::tempdir().unwrap();
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let mut proxy = finish_proxy("ordinary", &directory.path().join("native"), &service);
    let mut remote = proxy.remote.take().unwrap();
    drop(proxy);
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let (entered, release) = remote.io.as_mut().unwrap().delay_cleanup();
    let scope = ExecutionScope::new(
        pse_diagnostics::Cancellation::default(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    assert!(matches!(
        remote.exchange(p::Request::Shutdown, &scope, None).unwrap(),
        p::Response::Shutdown
    ));
    remote.closed = true;
    // No parent I/O stop or sender drop has occurred. Observe leader exit without
    // reaping it; consuming retirement below still owns the reap and every join.
    let reader = ProcessReader::default();
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        match reader.stat(pid) {
            ProcessRead::Present(stat) if stat.state == 'Z' => break,
            ProcessRead::Present(_) => {}
            other => panic!("owned shutdown child observation changed: {other:?}"),
        }
        assert!(Instant::now() < end, "peer waited for parent sender drop");
        std::thread::sleep(Duration::from_millis(2));
    }
    let charged = pool.reserved();
    assert!(charged > baseline);
    let finish = std::thread::spawn(move || remote.finish(Vec::new()));
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(!finish.is_finished());
    assert!(endpoint.exists());
    assert_eq!(pool.reserved(), charged);
    release.send(()).unwrap();
    finish.join().unwrap().unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn provider_binding_finish_reaches_actual_remote_handles_process_and_io() {
    let directory = tempfile::tempdir().unwrap();
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let (mut binding, request, pid, _) = provider(
        "ordinary",
        &directory.path().join("native"),
        ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
        &service,
        &pool,
    );
    let flag = std::sync::atomic::AtomicBool::new(false);
    let mut buffer = ProviderBuffer::new(&request);
    buffer
        .fill(
            &mut binding,
            &[500., 20.],
            &request,
            ProviderRuntime::default(),
            &EvaluationContext {
                cancelled: &flag,
                max_result_bytes: 4096,
            },
        )
        .unwrap();
    binding.finish().unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(pool.reserved(), baseline);
}

fn orphan_preparation(proxy: &Proxy) -> p::Prepare {
    let raw = ProviderRequest::all(proxy.factory.spec(), DerivativeOrder::Second);
    let checked =
        PreparedProviderRequest::new(Arc::new(proxy.factory.spec().clone()), raw.clone()).unwrap();
    p::Prepare {
        version: p::PREPARE_VERSION,
        handle: p::SessionHandle {
            attempt: proxy.attempt,
            session: proxy.remote.as_ref().unwrap().session,
            generation: pse_operations::mint_id(),
        },
        spec: proxy.factory.descriptor.clone(),
        configuration: proxy.factory.water.configuration_key(),
        registry: (*proxy.factory.registry).clone(),
        physical: proxy.factory.physical,
        regime: "vapor".into(),
        request: raw,
        remaining_ns: None,
        result_limit: checked.extents().bytes().unwrap() as u64,
    }
}

#[test]
fn current_worker_refuses_retired_service_and_preparation_on_original_connection() {
    let directory = tempfile::tempdir().unwrap();
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let proxy = finish_proxy("ordinary", &directory.path().join("native"), &service);
    let remote = proxy.remote.as_ref().unwrap();
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let io = remote.io.as_ref().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    // Shutdown has the same wire tag in both revisions. The retired method must
    // refuse that operation at routing, before it can close the current child.
    let error = runtime
        .block_on(io.exchange_at_path(
            "/pse.scientific_worker.v1.ScientificWorker/Exchange",
            p::Request::Shutdown,
        ))
        .unwrap_err();
    assert!(matches!(
        error,
        WorkerError::Exchange(error)
            if error.code == connectrpc::ErrorCode::Unimplemented
                && error.message.as_deref() == Some(
                    "method not found: pse.scientific_worker.v1.ScientificWorker/Exchange"
                )
    ));

    let original = orphan_preparation(&proxy);
    let handle = original.handle;
    let exchange = |request| remote.exchange(request, &proxy.scope, None).unwrap();
    let mut foreign = original.clone();
    foreign.handle.session = pse_operations::mint_id();
    foreign.handle.generation = SemanticId::from_bytes([u8::MAX; 16]);
    assert!(matches!(
        exchange(p::Request::Prepare(foreign)),
        p::Response::Failure(p::Failure::Contract(detail))
            if detail == "scientific worker foreign preparation context"
    ));
    assert!(matches!(
        exchange(p::Request::Prepare(original.clone())),
        p::Response::Prepared(value) if value.handle == handle
    ));
    assert!(matches!(
        exchange(p::Request::Close(handle)),
        p::Response::Closed(value) if value == handle
    ));
    assert!(matches!(
        exchange(p::Request::Prepare(original)),
        p::Response::Failure(p::Failure::Contract(detail))
            if detail == "scientific worker stale preparation generation"
    ));

    let fresh = orphan_preparation(&proxy);
    let handle = fresh.handle;
    assert!(matches!(
        exchange(p::Request::Prepare(fresh)),
        p::Response::Prepared(value) if value.handle == handle
    ));
    assert!(matches!(
        exchange(p::Request::Close(handle)),
        p::Response::Closed(value) if value == handle
    ));
    drop(runtime);
    Box::new(proxy).finish().unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);
}

async fn preparation_witness(path: std::path::PathBuf, expected: &str) {
    let end = Instant::now() + Duration::from_secs(10);
    while std::fs::read_to_string(&path).ok().as_deref() != Some(expected) {
        assert!(
            Instant::now() < end,
            "missing actual child witness: {path:?}"
        );
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

fn assert_terminal_retirements(
    failure: &p::Failure,
    orphan: &str,
    remaining: &[String],
    key: ContentHash,
) {
    use pse_model::diagnostic::{BoundaryDiagnostic, Observation};
    fn text(value: &BoundaryDiagnostic, name: &str, expected: &str) -> bool {
        matches!(value.observations.get(name), Some(Observation::Text(actual)) if actual == expected)
            || value.causes.iter().any(|cause| text(cause, name, expected))
    }
    fn contract(value: &BoundaryDiagnostic, expected: &str) -> bool {
        (value.code == pse_diagnostics::DiagnosticCode::KernelUnboundParameter
            && text(value, "detail", expected))
            || value.causes.iter().any(|cause| contract(cause, expected))
    }
    let p::Failure::Retirement { causes } = failure else {
        panic!("complete native aggregate required: {failure:?}")
    };
    let [
        p::Failure::ProviderRetirement { failures },
        p::Failure::Retirement { causes: stops },
    ] = causes.as_slice()
    else {
        panic!("original orphan and all remaining native errors required: {causes:?}")
    };
    let [original] = failures.as_slice() else {
        panic!("one original keyed orphan failure required")
    };
    assert_eq!(original.key, key);
    assert!(
        matches!(original.cause.as_ref(), p::Failure::Contract(detail)
        if detail == &format!("controlled native retirement failure {orphan}"))
    );
    let mut generations = Vec::new();
    for stop in stops {
        let p::Failure::Diagnostic(diagnostic) = stop else {
            panic!("Stop facts must remain explicitly passive observations")
        };
        assert!(
            matches!(diagnostic.observations.get("action"), Some(Observation::Text(action)) if action == "Stop")
        );
        let Some(Observation::Text(generation)) = diagnostic.observations.get("generation") else {
            panic!("original Stop generation required")
        };
        assert!(text(diagnostic, "provider_key", &key.to_string()));
        assert!(contract(
            diagnostic,
            &format!("provider contract: controlled native retirement failure {generation}")
        ));
        generations.push(generation.clone());
    }
    generations.sort();
    assert_eq!(generations, remaining);
}

#[test]
fn actual_abandoned_prepare_method_retires_only_orphan_and_keeps_survivors() {
    struct ReleaseGate(std::path::PathBuf);
    impl Drop for ReleaseGate {
        fn drop(&mut self) {
            let _ = std::fs::write(&self.0, b"caller released preparation gate");
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("native");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let mut proxy = finish_proxy("prepare-departure", &marker, &service);
    let remote = proxy.remote.as_ref().unwrap();
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let admitted = pool.reserved();
    assert!(admitted > baseline);
    let preparation = orphan_preparation(&proxy);
    let orphan = preparation.handle;
    let generation = orphan.generation.to_string();
    let io = remote.io.as_ref().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        // Declared before the future so unwind drops its strong client reference
        // before releasing the child gate or beginning Proxy containment.
        let release = ReleaseGate(marker.with_extension("prepare-release"));
        let mut exchange = Box::pin(io.departing_exchange(p::Request::Prepare(preparation)).unwrap());
        tokio::select! {
            result = &mut exchange => panic!("gated generated Prepare unexpectedly completed: {result:?}"),
            _ = preparation_witness(marker.with_extension("prepare-built"), &generation) => {},
        }
        // This drops the actual generated Exchange future, not a synchronous
        // caller's reply receiver. Hyper 1.11.1/h2 0.4.19 cancel this stream;
        // the native Sender-closed witness below establishes server departure.
        drop(exchange);
        drop(release);
        preparation_witness(marker.with_extension("prepare-claimed"), &generation).await;
        preparation_witness(marker.with_extension("prepare-departed"), &generation).await;
        preparation_witness(marker.with_extension("prepare-retired"), &generation).await;
    });
    drop(runtime);
    assert_eq!(
        std::fs::read_to_string(marker.with_extension("finished")).unwrap(),
        "finished\n"
    );
    // Normal production dispatch still uses the same authenticated connection.
    // Controls must refuse the orphan, while numerical admission and both
    // independently prepared native owners remain usable.
    for request in [
        p::Request::Close(orphan),
        p::Request::Evaluate(p::Evaluate {
            handle: orphan,
            invocation: 1,
            inputs: vec![500., 20.],
            directions: vec![],
            cotangents: vec![],
            weights: vec![],
        }),
    ] {
        assert!(matches!(
            io.exchange(request, &proxy.scope, None).unwrap(),
            p::Response::Failure(p::Failure::Contract(detail)) if detail == "scientific worker unknown or foreign handle"
        ));
    }
    for (_, handle, _) in &proxy.prepared {
        let p::Response::Values(values) = io
            .exchange(
                p::Request::Evaluate(p::Evaluate {
                    handle: *handle,
                    invocation: 1,
                    inputs: vec![500., 20.],
                    directions: vec![],
                    cotangents: vec![],
                    weights: vec![],
                }),
                &proxy.scope,
                None,
            )
            .unwrap()
        else {
            panic!("method departure revoked an unrelated native handle")
        };
        assert_eq!(&values.handle, handle);
        assert_eq!(values.invocation, 1);
        assert!(!values.values.is_empty());
        assert!(values.values.iter().all(|value| *value == 1.));
    }
    proxy
        .remote
        .as_mut()
        .unwrap()
        .child
        .check_running()
        .unwrap();
    assert_eq!(pool.reserved(), admitted);
    assert!(endpoint.exists());
    let (entered, release) = proxy
        .remote
        .as_mut()
        .unwrap()
        .io
        .as_mut()
        .unwrap()
        .delay_cleanup();
    let finish = std::thread::spawn(move || Box::new(proxy).finish());
    entered.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(!finish.is_finished());
    assert_eq!(pool.reserved(), admitted);
    assert!(endpoint.exists());
    release.send(()).unwrap();
    finish.join().unwrap().unwrap();
    assert_eq!(
        std::fs::read_to_string(marker.with_extension("finished")).unwrap(),
        "finished\nfinished\nfinished\n"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn actual_abandoned_prepare_retirement_failure_finishes_remaining_owners_and_fails_closed() {
    receive_failed_retirement(false);
}

#[test]
fn immediate_parent_failure_retains_already_joined_child_finalization() {
    receive_failed_retirement(true);
}

#[test]
fn actual_abandoned_shutdown_retains_original_native_retirement_failures() {
    use pse_diagnostics::process_facts::{ProcessRead, ProcessReader};

    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("native");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let mut proxy = finish_proxy("shutdown-departure-retire-faults", &marker, &service);
    let mut expected: Vec<_> = proxy
        .prepared
        .iter()
        .map(|(_, handle, _)| handle.generation.to_string())
        .collect();
    expected.sort();
    let remote = proxy.remote.as_mut().unwrap();
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let admitted = pool.reserved();
    assert!(admitted > baseline);
    let io = remote.io.as_mut().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut exchange = Box::pin(io.departing_exchange(p::Request::Shutdown).unwrap());
        tokio::select! {
            result = &mut exchange => panic!("gated generated Shutdown unexpectedly completed: {result:?}"),
            _ = preparation_witness(marker.with_extension("shutdown-claimed"), "retired") => {},
        }
        // The child verifies that the generated method's receiver closed before
        // offering the failed native retirement, on this same H2 connection.
        drop(exchange);
        preparation_witness(marker.with_extension("native-errors"), &expected.join("\n")).await;
    });
    drop(runtime);
    let mut finished: Vec<_> = std::fs::read_to_string(marker.with_extension("failed-finishes"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    finished.sort();
    assert_eq!(
        finished, expected,
        "both native owners must be attempted once"
    );
    io.request_stop();
    let reader = ProcessReader::default();
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        match reader.stat(pid) {
            ProcessRead::Present(stat) if stat.state == 'Z' => break,
            ProcessRead::Present(_) => {}
            other => panic!("original child observation changed: {other:?}"),
        }
        assert!(
            Instant::now() < end,
            "failed child did not exit through graceful drain"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(endpoint.exists());
    assert_eq!(pool.reserved(), admitted);
    let result = proxy.remote.take().unwrap().finish(Vec::new());
    assert!(
        matches!(&result, Err(WorkerError::Exit(status)) if status.code() == Some(101)),
        "parent finish must retain the original unsuccessful child exit: {result:?}"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert!(!endpoint.exists());
    // The proxy still owns its pre-launch physical declaration reservation.
    assert_eq!(pool.reserved(), baseline + proxy.factory.registry_bytes);
    drop(proxy);
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn actual_abandoned_close_preserves_original_native_retirement_failure() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("native");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let proxy = finish_proxy("close-departure-retire-faults", &marker, &service);
    let closing = proxy.prepared[0].1;
    let survivor = proxy.prepared[1].1;
    let remote = proxy.remote.as_ref().unwrap();
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let io = remote.io.as_ref().unwrap();
    let generation = closing.generation.to_string();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut exchange = Box::pin(io.departing_exchange(p::Request::Close(closing)).unwrap());
        tokio::select! {
            result = &mut exchange => panic!("gated Close unexpectedly replied: {result:?}"),
            _ = preparation_witness(marker.with_extension("close-claimed"), &generation) => {},
        }
        drop(exchange);
        preparation_witness(marker.with_extension("close-departed"), &generation).await;
    });
    drop(runtime);
    let observed = io.exchange(
        p::Request::Evaluate(p::Evaluate {
            handle: survivor,
            invocation: 1,
            inputs: vec![500., 20.],
            directions: vec![],
            cotangents: vec![],
            weights: vec![],
        }),
        &proxy.scope,
        None,
    );
    let finished = Box::new(proxy).finish();
    let mut actual: Vec<_> = std::fs::read_to_string(marker.with_extension("failed-finishes"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    actual.sort();
    let mut expected = [generation.clone(), survivor.generation.to_string()];
    expected.sort();
    assert_eq!(actual, expected);
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);
    assert!(finished.is_err());
    let p::Response::Failure(p::Failure::Retirement { causes }) = observed.unwrap() else {
        panic!("original failed Close and independent retirement must reach the next observer");
    };
    assert_eq!(causes.len(), 2);
    use pse_model::diagnostic::Observation;
    for (cause, (action, generation)) in causes.iter().zip([
        ("Close", generation),
        ("Stop", survivor.generation.to_string()),
    ]) {
        let p::Failure::Diagnostic(detail) = cause else {
            panic!("typed handle retirement facts required");
        };
        assert!(
            matches!(detail.observations.get("action"), Some(Observation::Text(actual)) if actual == action)
        );
        assert!(matches!(
            detail.observations.get("generation"),
            Some(Observation::Text(actual)) if actual == &generation
        ));
        assert_eq!(detail.causes.len(), 1);
        assert_eq!(detail.causes[0].causes.len(), 1);
        let keyed = &detail.causes[0].causes[0];
        assert_eq!(keyed.causes.len(), 1);
        assert_eq!(
            keyed.causes[0].code,
            pse_diagnostics::DiagnosticCode::KernelUnboundParameter
        );
        let expected_detail =
            format!("provider contract: controlled native retirement failure {generation}");
        assert!(matches!(
            keyed.causes[0].observations.get("detail"),
            Some(Observation::Text(actual)) if actual == &expected_detail
        ));
    }
}

#[test]
fn actual_abandoned_expired_evaluation_preserves_original_deadline_and_finish_failure() {
    receive_abandoned_effect_retirement(true);
}

#[test]
fn actual_abandoned_preparation_refusal_preserves_original_admission_and_finish_failure() {
    receive_abandoned_effect_retirement(false);
}

fn receive_abandoned_effect_retirement(expired_evaluation: bool) {
    use pse_diagnostics::DiagnosticCode;
    use pse_model::diagnostic::{BoundaryDiagnostic, Observation};

    fn descendants(value: &BoundaryDiagnostic) -> impl Iterator<Item = &BoundaryDiagnostic> {
        let mut pending = vec![value];
        std::iter::from_fn(move || {
            let next = pending.pop()?;
            pending.extend(next.causes.iter());
            Some(next)
        })
    }

    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("native");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let (phase, action) = if expired_evaluation {
        ("evaluate", "Evaluate")
    } else {
        ("prepare-refusal", "Prepare")
    };
    let proxy = finish_proxy(
        &format!("{phase}-departure-retire-faults"),
        &marker,
        &service,
    );
    let remote = proxy.remote.as_ref().unwrap();
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let io = remote.io.as_ref().unwrap();
    let mut request = orphan_preparation(&proxy);
    let handle = request.handle;
    let generation = handle.generation.to_string();
    let lost_request = if expired_evaluation {
        let allowance = Duration::from_secs(2);
        request.remaining_ns = Some(u64::try_from(allowance.as_nanos()).unwrap());
        assert!(matches!(
            remote.exchange(p::Request::Prepare(request), &proxy.scope, None).unwrap(),
            p::Response::Prepared(value) if value.handle == handle
        ));
        std::fs::write(marker.with_extension("expiring-generation"), &generation).unwrap();
        // The original child clock started before the authenticated Prepare reply.
        std::thread::sleep(allowance);
        p::Request::Evaluate(p::Evaluate {
            handle,
            invocation: 1,
            inputs: vec![500., 20.],
            directions: vec![],
            cotangents: vec![],
            weights: vec![],
        })
    } else {
        p::Request::Prepare(request)
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut exchange = Box::pin(io.departing_exchange(lost_request).unwrap());
        tokio::select! {
            result = &mut exchange => panic!("gated {action} unexpectedly replied: {result:?}"),
            _ = preparation_witness(marker.with_extension(format!("{phase}-claimed")), &generation) => {},
        }
        drop(exchange);
        preparation_witness(marker.with_extension(format!("{phase}-departed")), &generation).await;
    });
    drop(runtime);
    let survivor = proxy.prepared[1].1;
    let observed = io
        .exchange(p::Request::Close(survivor), &proxy.scope, None)
        .unwrap();
    let mut expected: Vec<_> = proxy
        .prepared
        .iter()
        .map(|(_, handle, _)| handle.generation.to_string())
        .collect();
    expected.push(generation.clone());
    expected.sort();
    assert!(Box::new(proxy).finish().is_err());
    let mut finished: Vec<_> = std::fs::read_to_string(marker.with_extension("finished"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    finished.sort();
    assert_eq!(
        finished, expected,
        "each original native binding finishes once"
    );
    assert!(
        !marker.exists(),
        "refused or joined work must not enter native fill"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);

    let p::Response::Failure(p::Failure::Diagnostic(detail)) = observed else {
        panic!("lost {action} must expose the original joined retirement causes: {observed:?}");
    };
    let text = |value: &BoundaryDiagnostic, field: &str, expected: &str| matches!(value.observations.get(field), Some(Observation::Text(actual)) if actual == expected);
    assert!(text(&detail, "action", action));
    assert!(text(&detail, "generation", &generation));
    if expired_evaluation {
        assert!(descendants(&detail).any(|cause| cause.code == DiagnosticCode::RuntimeTimeout));
    } else {
        assert!(descendants(&detail).any(|cause| {
            cause.code == DiagnosticCode::KernelUnboundParameter
                && text(
                    cause,
                    "detail",
                    "provider contract: controlled original provider admission refusal",
                )
        }));
    }
    let finish = format!("provider contract: controlled native retirement failure {generation}");
    assert!(descendants(&detail).any(|cause| {
        cause.code == DiagnosticCode::KernelUnboundParameter && text(cause, "detail", &finish)
    }));
}

fn receive_failed_retirement(immediate_parent_finish: bool) {
    use pse_diagnostics::process_facts::{ProcessRead, ProcessReader};
    struct ReleaseGate(std::path::PathBuf);
    impl Drop for ReleaseGate {
        fn drop(&mut self) {
            let _ = std::fs::write(&self.0, b"caller released failing preparation gate");
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("native");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let mut proxy = finish_proxy("prepare-departure-retire-faults", &marker, &service);
    let preparation = orphan_preparation(&proxy);
    let orphan = preparation.handle;
    let generation = orphan.generation.to_string();
    let survivor = proxy.prepared[0].1;
    let request = PreparedProviderRequest::new(
        Arc::new(proxy.factory.spec().clone()),
        proxy.prepared[0].0.clone(),
    )
    .unwrap();
    let mut expected: Vec<_> = proxy
        .prepared
        .iter()
        .map(|(_, handle, _)| handle.generation.to_string())
        .collect();
    expected.push(generation.clone());
    expected.sort();
    let mut remaining = expected
        .iter()
        .filter(|value| **value != generation)
        .cloned()
        .collect::<Vec<_>>();
    remaining.sort();
    let key = proxy.factory.spec().key().0;
    let remote = proxy.remote.as_mut().unwrap();
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let admitted = pool.reserved();
    assert!(admitted > baseline);
    let io = remote.io.as_mut().unwrap();
    let (entered, release) = io.delay_cleanup();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let gate = ReleaseGate(marker.with_extension("prepare-release"));
        let mut exchange = Box::pin(io.departing_exchange(p::Request::Prepare(preparation)).unwrap());
        tokio::select! {
            result = &mut exchange => panic!("gated generated Prepare unexpectedly completed: {result:?}"),
            _ = preparation_witness(marker.with_extension("prepare-built"), &generation) => {},
        }
        drop(exchange);
        drop(gate);
        preparation_witness(marker.with_extension("prepare-claimed"), &generation).await;
        preparation_witness(marker.with_extension("prepare-departed"), &generation).await;
        preparation_witness(marker.with_extension("prepare-retirement-error"), &generation).await;
        // The child checks the actual native typed error tree before publishing
        // this witness. These are not parent-received wire causes.
        preparation_witness(marker.with_extension("native-errors"), &expected.join("\n")).await;
    });
    drop(runtime);
    assert!(
        !marker.with_extension("prepare-retired").exists(),
        "failed cleanup cannot fabricate successful retirement"
    );
    let mut actual: Vec<_> = std::fs::read_to_string(marker.with_extension("failed-finishes"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    actual.sort();
    assert_eq!(
        actual, expected,
        "all three exact native finishes must be attempted once"
    );
    let delivery = io.exchange(
        p::Request::Evaluate(p::Evaluate {
            handle: survivor,
            invocation: 1,
            inputs: vec![500., 20.],
            directions: vec![],
            cotangents: vec![],
            weights: vec![],
        }),
        &proxy.scope,
        None,
    );
    assert!(
        matches!(&delivery, Ok(p::Response::Failure(_))),
        "joined native retirement facts must reach the parent, not generic unavailability: {delivery:?}"
    );
    let p::Response::Failure(failure) = delivery.unwrap() else {
        panic!("terminal observations required")
    };
    assert_terminal_retirements(&failure, &generation, &remaining, key);
    assert_eq!(
        std::fs::read_to_string(marker.with_extension("native-joined")).unwrap(),
        "joined"
    );
    let original = handler::refusal(failure);
    // Observation does not revive science or a normal Shutdown grant. A second
    // terminal reader receives the same immutable original outcome.
    let p::Response::Failure(repeated) = io
        .exchange(p::Request::Shutdown, &proxy.scope, None)
        .unwrap()
    else {
        panic!("joined failure must not acknowledge successful Shutdown")
    };
    assert_terminal_retirements(&repeated, &generation, &remaining, key);
    if !immediate_parent_finish {
        io.request_stop();
        entered.recv_timeout(Duration::from_secs(10)).unwrap();
        // The test child returns its original joined error (normal test failure),
        // rather than taking terminate_unsettled_native's abrupt exit125 path.
        let reader = ProcessReader::default();
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            match reader.stat(pid) {
                ProcessRead::Present(stat) if stat.state == 'Z' => break,
                ProcessRead::Present(_) => {}
                other => panic!("owned failing child observation changed: {other:?}"),
            }
            assert!(
                Instant::now() < end,
                "failed child did not exit through existing containment"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    assert_eq!(pool.reserved(), admitted);
    assert!(endpoint.exists());
    let finish = std::thread::spawn(move || {
        let result = proxy.fail(original);
        assert!(proxy.failed && proxy.remote.is_none() && proxy.prepared.is_empty());
        assert!(
            matches!(proxy.prepare(&request), Err(WorkerError::Infrastructure(detail))
            if detail == "scientific worker binding is invalidated")
        );
        drop(proxy);
        result
    });
    if immediate_parent_finish {
        entered.recv_timeout(Duration::from_secs(10)).unwrap();
    }
    // Reap can finish while the I/O thread is still at its actual join gate;
    // admission and endpoint must remain owned until that independent join.
    let end = Instant::now() + Duration::from_secs(10);
    while std::path::Path::new(&format!("/proc/{pid}")).exists() {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!finish.is_finished());
    assert_eq!(pool.reserved(), admitted);
    assert!(endpoint.exists());
    release.send(()).unwrap();
    let result = finish.join().unwrap();
    let WorkerError::Retirement { causes } = &result else {
        panic!("parent must preserve independent dispatch and child-exit causes")
    };
    let native = if matches!(
        causes.first(),
        Some(WorkerError::Provider(ProviderError::Retirement { .. }))
    ) {
        &result
    } else {
        causes.first().unwrap()
    };
    assert_terminal_retirements(
        &handler::failure_observation(native),
        &generation,
        &remaining,
        key,
    );
    assert!(
        causes
            .iter()
            .any(|cause| matches!(cause, WorkerError::Exit(status) if status.code() == Some(101))),
        "normal parent failure lost the already joined child's original finalization: {result:?}"
    );
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn actual_close_pending_drains_fill_and_preserves_an_independent_prepared_handle() {
    struct ReleaseGate(std::path::PathBuf);
    impl Drop for ReleaseGate {
        fn drop(&mut self) {
            // A failing assertion must release the real native thread before
            // std::thread::scope joins the two request callers during unwind.
            let _ = UnixDatagram::unbound().and_then(|gate| gate.send_to(b"release", &self.0));
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("native");
    let phases = UnixDatagram::bind(marker.with_extension("close-events")).unwrap();
    phases
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let mut proxy = finish_proxy("close-pending", &marker, &service);
    let closing = proxy.prepared[0].1;
    let survivor = proxy.prepared[1].1;
    let remote = proxy.remote.as_ref().unwrap();
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let admitted = pool.reserved();
    assert!(admitted > baseline);
    let io = remote.io.as_ref().unwrap();
    std::thread::scope(|threads| {
        let release = ReleaseGate(marker.with_extension("close-release"));
        let closing_fill = closing.clone();
        let scope = &proxy.scope;
        let evaluation = threads.spawn(move || {
            io.exchange(
                p::Request::Evaluate(p::Evaluate {
                    handle: closing_fill,
                    invocation: 1,
                    inputs: vec![13., 20.],
                    directions: vec![],
                    cotangents: vec![],
                    weights: vec![],
                }),
                scope,
                None,
            )
        });
        observe_phase(&phases, b"active").unwrap();
        let close_handle = closing;
        let close =
            threads.spawn(move || io.exchange(p::Request::Close(close_handle), scope, None));
        observe_phase(&phases, format!("queued:{}", closing.generation).as_bytes()).unwrap();
        assert!(!evaluation.is_finished());
        assert!(
            !close.is_finished(),
            "Close cannot acknowledge before native return"
        );
        assert!(!marker.with_extension("finished").exists());
        assert_eq!(pool.reserved(), admitted);
        drop(release);
        let p::Response::Values(values) = evaluation.join().unwrap().unwrap() else {
            panic!("Close drains the original fill rather than cancelling it")
        };
        assert_eq!(values.handle, closing);
        assert_eq!(values.invocation, 1);
        assert!(!values.values.is_empty());
        assert!(values.values.iter().all(|value| *value == 1.));
        assert!(matches!(
            close.join().unwrap().unwrap(),
            p::Response::Closed(actual) if actual == closing
        ));
        assert!(marker.with_extension("returned").exists());
        assert_eq!(
            std::fs::read_to_string(marker.with_extension("finished")).unwrap(),
            "finished\n"
        );
        let p::Response::Values(values) = io
            .exchange(
                p::Request::Evaluate(p::Evaluate {
                    handle: survivor,
                    invocation: 1,
                    inputs: vec![500., 20.],
                    directions: vec![],
                    cotangents: vec![],
                    weights: vec![],
                }),
                scope,
                None,
            )
            .unwrap()
        else {
            panic!("another handle's queued Close must preserve this preparation")
        };
        assert_eq!(values.handle, survivor);
        assert!(!values.jacobian.is_empty());
        assert!(values.jacobian.iter().all(|value| *value == 2.));
        assert_eq!(pool.reserved(), admitted);
    });
    // The exact acknowledged handle is already retired. Explicit normal finish
    // still owns the survivor, process wait and every parent I/O join.
    proxy.prepared.retain(|(_, handle, _)| *handle != closing);
    Box::new(proxy).finish().unwrap();
    assert_eq!(
        std::fs::read_to_string(marker.with_extension("finished")).unwrap(),
        "finished\nfinished\n"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn finish_attempts_every_handle_and_shutdown_retaining_all_typed_acknowledgement_failures() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("native");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let proxy = finish_proxy("finish-ack-faults", &marker, &service);
    let pid = proxy.remote.as_ref().unwrap().child.pid();
    let error = Box::new(proxy).finish().unwrap_err();
    let ProviderError::Nested { cause, .. } = error else {
        panic!("remote finish must preserve typed cause");
    };
    let error = cause.as_error().downcast_ref::<WorkerError>().unwrap();
    let WorkerError::Retirement { causes } = error else {
        panic!("independent lifecycle failures must all survive: {error}");
    };
    assert_eq!(causes.len(), 3);
    assert!(
        matches!(&causes[0], WorkerError::Lifecycle { action: "Close", handle: Some(_), cause } if matches!(cause.as_ref(), WorkerError::Provider(ProviderError::Contract(_))))
    );
    assert!(matches!(
        &causes[1],
        WorkerError::Lifecycle {
            action: "Close",
            handle: Some(_),
            ..
        }
    ));
    assert!(matches!(
        &causes[2],
        WorkerError::Lifecycle {
            action: "Shutdown",
            handle: None,
            ..
        }
    ));
    assert_eq!(
        std::fs::read_to_string(marker.with_extension("lifecycle")).unwrap(),
        "Close\nClose\nShutdown\n"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn unknown_reap_keeps_original_finish_endpoint_and_admission_until_actual_settlement() {
    let directory = tempfile::tempdir().unwrap();
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let mut proxy = finish_proxy("ordinary", &directory.path().join("native"), &service);
    let remote = proxy.remote.as_mut().unwrap();
    let endpoint = remote._endpoint.path().to_path_buf();
    let pid = remote.child.pid();
    let (unknown, release) = remote.child.delay_reap();
    let charged = pool.reserved();
    let finish = std::thread::spawn(move || Box::new(proxy).finish());
    unknown.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(!finish.is_finished());
    assert!(endpoint.exists());
    assert_eq!(pool.reserved(), charged);
    release.send(()).unwrap();
    assert!(
        finish.join().unwrap().is_err(),
        "failed wait observation remains reportable after actual retirement"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn actual_child_finishes_every_native_owner_even_when_each_close_fails() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("native");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let proxy = finish_proxy("finish-native-faults", &marker, &service);
    let key = proxy.factory.spec().key();
    let pid = proxy.remote.as_ref().unwrap().child.pid();
    let error = Box::new(proxy).finish().unwrap_err();
    let ProviderError::Nested { cause, .. } = error else {
        panic!("typed remote failure required");
    };
    let WorkerError::Retirement { causes } =
        cause.as_error().downcast_ref::<WorkerError>().unwrap()
    else {
        panic!("both native close failures must survive");
    };
    assert_eq!(causes.len(), 2);
    for error in causes {
        let WorkerError::Lifecycle {
            action: "Close",
            handle: Some(_),
            cause,
        } = error
        else {
            panic!("independent handle close required");
        };
        let WorkerError::Provider(ProviderError::Retirement { failures }) = cause.as_ref() else {
            panic!("original keyed native retirement required");
        };
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].key, key);
        assert!(
            matches!(failures[0].cause.as_ref(), ProviderError::Contract(detail) if detail == "controlled native finish failure")
        );
    }
    assert_eq!(
        std::fs::read_to_string(marker.with_extension("finished")).unwrap(),
        "finished\nfinished\n"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn successful_native_close_still_holds_admission_until_late_io_cleanup_finishes() {
    let directory = tempfile::tempdir().unwrap();
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let mut proxy = finish_proxy("ordinary", &directory.path().join("native"), &service);
    let remote = proxy.remote.as_mut().unwrap();
    let pid = remote.child.pid();
    let endpoint = remote._endpoint.path().to_path_buf();
    let (entered, release) = remote.io.as_mut().unwrap().delay_cleanup();
    let charged = pool.reserved();
    let finish = std::thread::spawn(move || Box::new(proxy).finish());
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    // The I/O owner can reach its gate before the parent has reaped the child.
    let end = Instant::now() + Duration::from_secs(5);
    while std::path::Path::new(&format!("/proc/{pid}")).exists() {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!finish.is_finished());
    assert!(endpoint.exists());
    assert_eq!(pool.reserved(), charged);
    release.send(()).unwrap();
    finish.join().unwrap().unwrap();
    assert!(!endpoint.exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn actual_water_group_transport_preserves_raw_point_admission_and_capability() {
    let (service, pool, native) = service();
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("batch-native");
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let image = PreparedExecutable::observe(std::env::current_exe().unwrap(), 64 << 20)
        .unwrap()
        .test_entry(
            "scientific_worker::proxy::tests::controlled_child",
            "water-batch",
            marker,
        );
    let factory =
        LocalWaterFactory::prepare(image, &registry, WaterRegime::Liquid, service.clone()).unwrap();
    let raw = ProviderRequest {
        outputs: vec![2, 0, 1],
        order: DerivativeOrder::Value,
        demand: ProviderDemand::CompleteJets,
    };
    let request = PreparedProviderRequest::new(Arc::new(factory.spec().clone()), raw).unwrap();
    let scope = ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
    let mut proxy = Proxy {
        factory,
        scope,
        attempt: pse_operations::mint_id(),
        remote: None,
        prepared: vec![],
        invocation: 0,
        failed: false,
    };
    proxy.prepare_demand(&request).unwrap();
    assert!(proxy.supports_batch(&request));
    let pid = proxy.remote.as_ref().unwrap().child.pid();
    let derivative = PreparedProviderRequest::new(
        Arc::new(proxy.factory.spec().clone()),
        ProviderRequest::all(proxy.factory.spec(), DerivativeOrder::First),
    )
    .unwrap();
    proxy.prepare_demand(&derivative).unwrap();
    assert!(!proxy.supports_batch(&derivative));
    let points = [
        ProviderBatchPoint::values(
            SemanticId::from_bytes([1; 16]),
            vec![300.0, 996.556 / 0.018_015_268],
        ),
        ProviderBatchPoint::values(
            SemanticId::from_bytes([2; 16]),
            vec![f64::from_bits(0x7ff8_0000_0000_1234), 55000.0],
        ),
        ProviderBatchPoint::values(SemanticId::from_bytes([3; 16]), vec![250.0, 55000.0]),
        ProviderBatchPoint::values(
            SemanticId::from_bytes([4; 16]),
            vec![500.0, 0.435 / 0.018_015_268],
        ),
        ProviderBatchPoint::values(SemanticId::from_bytes([5; 16]), vec![310.0, 56000.0]),
    ];
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    let context = EvaluationContext {
        cancelled: &cancelled,
        max_result_bytes: 65536,
    };
    // Enter below parent-side scalar shape admission so raw NaN actually crosses the binary wire.
    let refs: Vec<_> = points.iter().collect();
    let results = proxy.fill_batch(&refs, &request, &context).unwrap();
    let first = results[0].as_ref().unwrap();
    assert!(
        (first.values[1] - 99_241.835_2).abs() < 0.02,
        "independent IAPWS pressure control"
    );
    assert!(results[1..4].iter().all(Result::is_err));
    assert!(results[4].is_ok());
    let handle = proxy.handle_for(&request).unwrap();
    let response = proxy
        .remote
        .as_ref()
        .unwrap()
        .exchange(
            p::Request::EvaluateBatch(p::EvaluateBatch {
                handle,
                invocation: proxy.invocation,
                points: vec![p::BatchPoint {
                    id: points[0].id,
                    inputs: points[0].inputs.iter().map(|x| x.to_bits()).collect(),
                    directions: vec![],
                    cotangents: vec![],
                    weights: vec![],
                }],
            }),
            &proxy.scope,
            None,
        )
        .unwrap();
    assert!(matches!(
        response,
        p::Response::Failure(p::Failure::Contract(_))
    ));
    let duplicate = p::BatchPoint {
        id: points[0].id,
        inputs: points[0].inputs.iter().map(|x| x.to_bits()).collect(),
        directions: vec![],
        cotangents: vec![],
        weights: vec![],
    };
    let response = proxy
        .remote
        .as_ref()
        .unwrap()
        .exchange(
            p::Request::EvaluateBatch(p::EvaluateBatch {
                handle,
                invocation: proxy.invocation + 1,
                points: vec![duplicate.clone(), duplicate],
            }),
            &proxy.scope,
            None,
        )
        .unwrap();
    assert!(matches!(
        response,
        p::Response::Failure(p::Failure::Contract(_))
    ));
    // A refused replay did not revoke this original prepared session or its next invocation.
    assert!(proxy.fill_batch(&refs, &request, &context).unwrap()[0].is_ok());
    drop(proxy);
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(pool.reserved(), 0);
    let execution = native.execution_report();
    for name in ["math_native_active", "math_native_population"] {
        assert_eq!(
            execution
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, count)| *count),
            Some(Some(0)),
            "native admission counter {name} must be present and settled",
        );
    }
}

#[test]
fn stop_active_group_drains_native_work_and_preserves_control_admission() {
    let (service, pool, _native) = service();
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("group-active");
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let image = PreparedExecutable::observe(std::env::current_exe().unwrap(), 64 << 20)
        .unwrap()
        .test_entry(
            "scientific_worker::proxy::tests::controlled_child",
            "batch-cooperative",
            marker.clone(),
        );
    let factory =
        LocalWaterFactory::prepare(image, &registry, WaterRegime::Vapor, service.clone()).unwrap();
    let request = PreparedProviderRequest::new(
        Arc::new(factory.spec().clone()),
        ProviderRequest::all(factory.spec(), DerivativeOrder::Value),
    )
    .unwrap();
    let scope = ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
    let mut proxy = Proxy {
        factory,
        scope,
        attempt: pse_operations::mint_id(),
        remote: None,
        prepared: vec![],
        invocation: 0,
        failed: false,
    };
    proxy.prepare_demand(&request).unwrap();
    assert!(proxy.supports_batch(&request));
    let handle = proxy.handle_for(&request).unwrap();
    let remote = proxy.remote.as_ref().unwrap();
    let pid = remote.child.pid();
    let io = remote.io.as_ref().unwrap();
    std::thread::scope(|threads| {
        let evaluating = threads.spawn(|| {
            io.exchange(
                p::Request::EvaluateBatch(p::EvaluateBatch {
                    handle,
                    invocation: 1,
                    points: [1, 2]
                        .map(|id| p::BatchPoint {
                            id: SemanticId::from_bytes([id; 16]),
                            inputs: [500.0f64, 20.0].map(f64::to_bits).to_vec(),
                            directions: vec![],
                            cotangents: vec![],
                            weights: vec![],
                        })
                        .to_vec(),
                }),
                &proxy.scope,
                None,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(pool.reserved() > 0);
        assert!(
            matches!(io.exchange(p::Request::Stop(handle), &proxy.scope, None).unwrap(),
            p::Response::Closed(actual) if actual == handle)
        );
        assert!(
            marker.with_extension("settled").exists(),
            "stop acknowledgement follows actual group return"
        );
        assert!(matches!(
            evaluating.join().unwrap().unwrap(),
            p::Response::Failure(p::Failure::Cancelled)
        ));
        assert!(
            pool.reserved() > 0,
            "process and I/O custody retain aggregate admission after one group retires"
        );
    });
    drop(proxy);
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn malformed_group_completion_never_publishes_partial_products() {
    let (service, pool, _native) = service();
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let directory = tempfile::tempdir().unwrap();
    for mode in [
        "water-batch-wrong-shape",
        "water-batch-wrong-point",
        "water-batch-missing-point",
        "water-batch-nonfinite",
    ] {
        let image = PreparedExecutable::observe(std::env::current_exe().unwrap(), 64 << 20)
            .unwrap()
            .test_entry(
                "scientific_worker::proxy::tests::controlled_child",
                mode,
                directory.path().join(mode),
            );
        let factory =
            LocalWaterFactory::prepare(image, &registry, WaterRegime::Liquid, service.clone())
                .unwrap();
        let prepared_reservation = pool.reserved();
        let request = PreparedProviderRequest::new(
            Arc::new(factory.spec().clone()),
            ProviderRequest::all(factory.spec(), DerivativeOrder::Value),
        )
        .unwrap();
        let scope = ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
        let mut proxy = Proxy {
            factory,
            scope,
            attempt: pse_operations::mint_id(),
            remote: None,
            prepared: vec![],
            invocation: 0,
            failed: false,
        };
        proxy.prepare_demand(&request).unwrap();
        let pid = proxy.remote.as_ref().unwrap().child.pid();
        let points = [
            ProviderBatchPoint::values(
                SemanticId::from_bytes([1; 16]),
                vec![300.0, 996.556 / 0.018_015_268],
            ),
            ProviderBatchPoint::values(SemanticId::from_bytes([2; 16]), vec![310.0, 56000.0]),
        ];
        let refs: Vec<_> = points.iter().collect();
        let cancelled = std::sync::atomic::AtomicBool::new(false);
        let result = proxy.fill_batch(
            &refs,
            &request,
            &EvaluationContext {
                cancelled: &cancelled,
                max_result_bytes: 65536,
            },
        );
        assert!(
            result.is_err(),
            "malformed whole-group framing never grants its first point's partial success"
        );
        assert!(proxy.failed);
        assert!(proxy.remote.is_none());
        assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
        assert_eq!(pool.reserved(), prepared_reservation);
        drop(proxy);
        assert_eq!(pool.reserved(), 0);
    }
}

#[test]
fn crash_after_dispatch_is_unknown_reaped_and_never_replayed() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("calls");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let control = pse_diagnostics::Cancellation::default();
    let flag = control.flag();
    let scope = ExecutionScope::new(control, Some(Instant::now() + Duration::from_secs(60)));
    let (mut binding, request, pid, prepared_reservation) =
        provider("crash", &marker, scope, &service, &pool);
    assert!(pool.reserved() > baseline);
    let context = EvaluationContext {
        cancelled: &flag,
        max_result_bytes: 4096,
    };
    let mut buffer = ProviderBuffer::new(&request);
    let error = buffer
        .fill(
            &mut binding,
            &[500.0, 20.0],
            &request,
            ProviderRuntime::default(),
            &context,
        )
        .unwrap_err();
    assert!(
        marker.exists(),
        "crash control must reach native dispatch: {error}"
    );
    use pse_model::diagnostic::{DiagnosticProjection as _, Observation};
    assert!(matches!(
        original_remote_error(&error)
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
            .observations
            .get("scientific_worker_completion_unknown"),
        Some(Observation::Boolean(true))
    ));
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(pool.reserved(), prepared_reservation);
    assert!(
        buffer
            .fill(
                &mut binding,
                &[500.0, 20.0],
                &request,
                ProviderRuntime::default(),
                &context
            )
            .is_err()
    );
    assert_eq!(std::fs::read_to_string(marker).unwrap().lines().count(), 1);
    drop(binding);
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn failed_remote_retains_aggregate_admission_after_reap_until_io_executor_joins() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("calls");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let registry = pse_quantity::generated::standard_registry().unwrap();
    let image = PreparedExecutable::observe(std::env::current_exe().unwrap(), 64 << 20)
        .unwrap()
        .test_entry(
            "scientific_worker::proxy::tests::controlled_child",
            "ordinary",
            marker,
        );
    let factory =
        LocalWaterFactory::prepare(image, &registry, WaterRegime::Vapor, service.clone()).unwrap();
    let prepared_reservation = pool.reserved();
    let request = PreparedProviderRequest::new(
        Arc::new(factory.spec().clone()),
        ProviderRequest::all(factory.spec(), DerivativeOrder::Second),
    )
    .unwrap();
    let mut proxy = Proxy {
        factory,
        scope: ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
        attempt: pse_operations::mint_id(),
        remote: None,
        prepared: vec![],
        invocation: 0,
        failed: false,
    };
    proxy.prepare_demand(&request).unwrap();
    let mut remote = proxy.remote.take().unwrap();
    let pid = remote.child.pid();
    let charged = pool.reserved();
    assert!(charged > baseline);
    let (cleanup_started, release_cleanup) = remote.io.as_mut().unwrap().delay_cleanup();
    let reaped = remote.child.witness_reap();
    let cleanup = std::thread::spawn(move || {
        remote.fail(WorkerError::UnknownCompletion("controlled failure".into()))
    });
    cleanup_started
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    reaped.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(
        pool.reserved(),
        charged,
        "parent executor and buffers still require their aggregate reservation"
    );
    release_cleanup.send(()).unwrap();
    use pse_model::diagnostic::{DiagnosticProjection as _, Observation};
    let error = cleanup.join().unwrap();
    assert!(matches!(
        original_worker_error(&error)
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
            .observations
            .get("scientific_worker_completion_unknown"),
        Some(Observation::Boolean(true))
    ));
    assert_eq!(
        pool.reserved(),
        prepared_reservation,
        "actual I/O join releases the aggregate while the prepared factory remains owned"
    );
    drop(proxy);
    assert_eq!(
        pool.reserved(),
        baseline,
        "final prepared factory disposal releases its metadata"
    );
}

#[test]
fn malformed_shape_nonfinite_and_stale_context_responses_never_publish_completion() {
    for mode in ["wrong-shape", "wrong-context", "nonfinite", "oversized"] {
        let directory = tempfile::tempdir().unwrap();
        let marker = directory.path().join("calls");
        let (service, pool, _native) = service();
        let baseline = pool.reserved();
        let control = pse_diagnostics::Cancellation::default();
        let flag = control.flag();
        let scope = ExecutionScope::new(control, Some(Instant::now() + Duration::from_secs(60)));
        let (mut binding, request, pid, prepared_reservation) =
            provider(mode, &marker, scope, &service, &pool);
        let context = EvaluationContext {
            cancelled: &flag,
            max_result_bytes: 4096,
        };
        let mut buffer = ProviderBuffer::new(&request);
        let error = buffer
            .fill(
                &mut binding,
                &[500.0, 20.0],
                &request,
                ProviderRuntime::default(),
                &context,
            )
            .unwrap_err();
        use pse_model::diagnostic::{DiagnosticProjection as _, Observation};
        assert!(
            matches!(
                original_remote_error(&error)
                    .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
                    .observations
                    .get("scientific_worker_completion_unknown"),
                Some(Observation::Boolean(true))
            ),
            "{mode}: {error}"
        );
        assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
        assert_eq!(pool.reserved(), prepared_reservation);
        assert!(
            buffer
                .fill(
                    &mut binding,
                    &[500.0, 20.0],
                    &request,
                    ProviderRuntime::default(),
                    &context
                )
                .is_err()
        );
        assert_eq!(std::fs::read_to_string(marker).unwrap().lines().count(), 1);
        drop(binding);
        assert_eq!(pool.reserved(), baseline);
    }
}

fn action_spec(registry: &pse_quantity::QuantityRegistry) -> ProviderSpec {
    let quantity = pse_quantity::standard::ids::quantity("neutral");
    let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
    let id = SemanticId::from_bytes([0xd1; 16]);
    let ports = |offset, count| {
        (offset..offset + count)
            .map(|ordinal: usize| pse_kernels::Port {
                id: pse_ids::named_id(id, &ordinal.to_string()),
                quantity,
                unit,
            })
            .collect()
    };
    ProviderSpec {
        id,
        revision: ContentHash::from_bytes([0x31; 32]),
        data: ContentHash::from_bytes([0x32; 32]),
        shapes: Default::default(),
        derivative_source: pse_kernels::DerivativeSource::Analytic,
        inputs: ports(0, 2),
        outputs: ports(2, 3),
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    }
}

#[test]
fn action_capable_native_owner_preserves_every_product_through_actual_transport() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("calls");
    let (service, _pool, _native) = service();
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let image = PreparedExecutable::observe(std::env::current_exe().unwrap(), 64 << 20)
        .unwrap()
        .test_entry(
            "scientific_worker::proxy::tests::controlled_child",
            "actions-cooperative",
            marker.clone(),
        );
    let factory =
        LocalWaterFactory::prepare(image, &registry, WaterRegime::Vapor, service.clone()).unwrap();
    let scope = ExecutionScope::new(
        pse_diagnostics::Cancellation::default(),
        Some(Instant::now() + Duration::from_secs(60)),
    );
    let mut proxy = Proxy {
        factory,
        scope,
        attempt: pse_operations::mint_id(),
        remote: None,
        prepared: vec![],
        invocation: 0,
        failed: false,
    };
    proxy.launch().unwrap();
    let session = proxy.remote.as_ref().unwrap().session;
    let specification = action_spec(&registry);
    let descriptor = pse_kernels::AdmittedProvider::new(specification.clone(), &registry).unwrap();
    // Independent exact controls at x=3,y=5 for (x*x+y, x*y, y*y).
    let full = [
        (
            ProviderDemand::CompleteJets,
            DerivativeOrder::Second,
            vec![],
            vec![],
            vec![],
            vec![6., 1., 5., 3., 0., 10.],
            vec![2., 0., 0., 0., 0., 1., 1., 0., 0., 0., 0., 2.],
        ),
        (
            ProviderDemand::SelectedColumns {
                columns: vec![1, 0],
            },
            DerivativeOrder::Second,
            vec![],
            vec![],
            vec![],
            vec![1., 6., 3., 5., 10., 0.],
            vec![0., 0., 0., 2., 0., 1., 1., 0., 2., 0., 0., 0.],
        ),
        (
            ProviderDemand::JacobianAction { directions: 2 },
            DerivativeOrder::First,
            vec![1., 0., 0., 2.],
            vec![],
            vec![],
            vec![6., 2., 5., 6., 0., 20.],
            vec![],
        ),
        (
            ProviderDemand::TransposeAction { cotangents: 2 },
            DerivativeOrder::First,
            vec![],
            vec![1., 0., 2., 0., 1., 1.],
            vec![],
            vec![6., 21., 5., 13.],
            vec![],
        ),
        (
            ProviderDemand::WeightedSecondAction { directions: 2 },
            DerivativeOrder::Second,
            vec![1., 0., 0., 2.],
            vec![],
            vec![1., 2., 3.],
            vec![16., 37.],
            vec![2., 4., 2., 12.],
        ),
    ];
    let selected = [
        (
            ProviderDemand::CompleteJets,
            DerivativeOrder::Second,
            vec![],
            vec![],
            vec![],
            vec![0., 10., 6., 1.],
            vec![0., 0., 0., 2., 2., 0., 0., 0.],
        ),
        (
            ProviderDemand::SelectedColumns {
                columns: vec![1, 0],
            },
            DerivativeOrder::Second,
            vec![],
            vec![],
            vec![],
            vec![10., 0., 1., 6.],
            vec![2., 0., 0., 0., 0., 0., 0., 2.],
        ),
        (
            ProviderDemand::JacobianAction { directions: 2 },
            DerivativeOrder::First,
            vec![1., 0., 0., 2.],
            vec![],
            vec![],
            vec![0., 20., 6., 2.],
            vec![],
        ),
        (
            ProviderDemand::TransposeAction { cotangents: 2 },
            DerivativeOrder::First,
            vec![],
            vec![1., 2., 0., 1.],
            vec![],
            vec![12., 12., 6., 1.],
            vec![],
        ),
        (
            ProviderDemand::WeightedSecondAction { directions: 2 },
            DerivativeOrder::Second,
            vec![1., 0., 0., 2.],
            vec![],
            vec![1., 2.],
            vec![12., 12.],
            vec![4., 0., 0., 4.],
        ),
    ];
    let cases = full
        .into_iter()
        .map(|case| (vec![0, 1, 2], vec![14., 15., 25.], case))
        .chain(
            selected
                .into_iter()
                .map(|case| (vec![2, 0], vec![25., 14.], case)),
        );
    for (
        index,
        (outputs, expected_values, (demand, order, directions, cotangents, weights, first, second)),
    ) in cases.enumerate()
    {
        let raw = ProviderRequest {
            outputs,
            order,
            demand,
        };
        let checked =
            PreparedProviderRequest::new(Arc::new(specification.clone()), raw.clone()).unwrap();
        let handle = p::SessionHandle {
            attempt: proxy.attempt,
            session,
            generation: pse_operations::mint_id(),
        };
        let request = p::Prepare {
            version: p::PREPARE_VERSION,
            handle,
            spec: pse_kernels::portable::ProviderRecord::capture(&descriptor),
            configuration: specification.identity(),
            registry: registry.to_builder(),
            physical: handler::physical_identity(&registry).unwrap(),
            regime: "vapor".into(),
            request: raw.clone(),
            remaining_ns: scientific_remaining_ns(&proxy.scope, Instant::now()).unwrap(),
            result_limit: checked.extents().bytes().unwrap() as u64,
        };
        let response = proxy
            .remote
            .as_ref()
            .unwrap()
            .exchange(p::Request::Prepare(request), &proxy.scope, None)
            .unwrap();
        assert!(
            matches!(response, p::Response::Prepared(actual) if actual.handle == handle && !actual.batch_supported)
        );
        proxy.prepared.push((raw, handle, false));
        let invocation = index as u64 + 1;
        let response = proxy
            .remote
            .as_ref()
            .unwrap()
            .exchange(
                p::Request::Evaluate(p::Evaluate {
                    handle,
                    invocation,
                    inputs: vec![3., 5.],
                    directions,
                    cotangents,
                    weights,
                }),
                &proxy.scope,
                None,
            )
            .unwrap();
        let p::Response::Values(values) = response else {
            panic!("controlled action owner must complete")
        };
        assert_eq!(values.handle, handle);
        assert_eq!(values.invocation, invocation);
        assert_eq!(values.values, expected_values);
        assert_eq!(values.jacobian, first);
        assert_eq!(values.hessians, second);
        assert_eq!(values.values.len(), checked.extents().values);
        assert_eq!(values.jacobian.len(), checked.extents().jacobian);
        assert_eq!(values.hessians.len(), checked.extents().hessians);
    }
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap().lines().count(),
        10,
        "one coherent native invocation per product and output selection"
    );
    let active = proxy.prepared[0].1;
    let first = proxy.prepared[2].1;
    let survivor = proxy.prepared[1].1;
    let stopped = proxy.prepared[3].1;
    let remote = proxy.remote.as_ref().unwrap();
    assert!(matches!(
        remote.exchange(p::Request::Close(first), &proxy.scope, None).unwrap(),
        p::Response::Closed(actual) if actual == first
    ));
    assert!(matches!(
        remote.exchange(p::Request::Stop(stopped), &proxy.scope, None).unwrap(),
        p::Response::Closed(actual) if actual == stopped
    ));
    let mut foreign = survivor;
    foreign.attempt = SemanticId::NIL;
    assert!(matches!(
        remote
            .exchange(p::Request::Stop(foreign), &proxy.scope, None)
            .unwrap(),
        p::Response::Failure(_)
    ));
    let io = remote.io.as_ref().unwrap();
    let scope = &proxy.scope;
    std::thread::scope(|threads| {
        let evaluate_handle = active;
        let evaluating = threads.spawn(move || {
            io.exchange(
                p::Request::Evaluate(p::Evaluate {
                    handle: evaluate_handle,
                    invocation: 20_000,
                    inputs: vec![13., 5.],
                    directions: vec![],
                    cotangents: vec![],
                    weights: vec![],
                }),
                scope,
                None,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !marker.with_extension("active").exists() {
            assert!(
                Instant::now() < deadline,
                "controlled native evaluation never entered"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(matches!(
            io.exchange(p::Request::Stop(active.clone()), scope, None).unwrap(),
            p::Response::Closed(actual) if actual == active
        ));
        assert!(matches!(
            evaluating.join().unwrap().unwrap(),
            p::Response::Failure(p::Failure::Cancelled)
        ));
    });
    // Independent scalar values for the still-live selected-columns handle. Neither
    // another handle's close/stop nor a foreign context may cancel this preparation.
    let response = remote
        .exchange(
            p::Request::Evaluate(p::Evaluate {
                handle: survivor,
                invocation: 10_000,
                inputs: vec![3., 5.],
                directions: vec![],
                cotangents: vec![],
                weights: vec![],
            }),
            &proxy.scope,
            None,
        )
        .unwrap();
    assert!(matches!(response, p::Response::Values(values)
        if values.handle == survivor && values.values == [14., 15., 25.]));
    for (invocation, successful) in [(10_000, false), (10_001, true)] {
        let response = remote
            .exchange(
                p::Request::Evaluate(p::Evaluate {
                    handle: survivor,
                    invocation,
                    inputs: vec![3., 5.],
                    directions: vec![],
                    cotangents: vec![],
                    weights: vec![],
                }),
                &proxy.scope,
                None,
            )
            .unwrap();
        assert_eq!(
            matches!(response, p::Response::Values(_)),
            successful,
            "a replay refused before mutation must not erase the admitted native preparation"
        );
    }
    assert!(matches!(
        remote
            .exchange(p::Request::Close(first), &proxy.scope, None)
            .unwrap(),
        p::Response::Failure(_)
    ));
    assert!(matches!(
        remote
            .exchange(p::Request::Shutdown, &proxy.scope, None)
            .unwrap(),
        p::Response::Shutdown
    ));
    proxy.remote.as_mut().unwrap().closed = true;
}

#[test]
fn changed_actual_stock_datum_is_refused_by_the_selected_reference_binding() {
    let (service, _pool, _native) = service();
    let registry = pse_quantity::generated::standard_registry().unwrap();
    // Mutate an actual declaration through its owner's serialization, then
    // re-admit the complete builder; no digest assertion stands in for data.
    let mut declarations = serde_json::to_value(registry.to_builder()).unwrap();
    let references = declarations["reference_states"].as_array_mut().unwrap();
    let mut changed = 0;
    for reference in references {
        if reference["name"] == "stock" {
            let original = reference["temperature"]["value"].as_f64().unwrap();
            reference["temperature"]["value"] = serde_json::json!(original + 1.0);
            changed += 1;
        }
    }
    assert_eq!(changed, 1);
    let changed: pse_quantity::QuantityRegistryBuilder =
        serde_json::from_value(declarations).unwrap();
    let changed = changed.build().unwrap();
    let original = WaterFactory::prepare(&registry, WaterRegime::Vapor).unwrap();
    let different = WaterFactory::prepare(&changed, WaterRegime::Vapor).unwrap();
    assert_ne!(original.configuration_key(), different.configuration_key());
    let image = PreparedExecutable::observe(std::env::current_exe().unwrap(), 64 << 20).unwrap();
    let local = LocalWaterFactory::prepare(image, &changed, WaterRegime::Vapor, service).unwrap();
    assert_eq!(
        local.water.configuration_key(),
        different.configuration_key()
    );
    let handle = p::SessionHandle {
        attempt: pse_operations::mint_id(),
        session: pse_operations::mint_id(),
        generation: pse_operations::mint_id(),
    };
    let request = p::Prepare {
        version: p::PREPARE_VERSION,
        handle,
        spec: local.descriptor.clone(),
        configuration: original.configuration_key(),
        registry: changed.to_builder(),
        physical: handler::physical_identity(&changed).unwrap(),
        regime: "vapor".into(),
        request: ProviderRequest::all(local.spec(), DerivativeOrder::Second),
        remaining_ns: None,
        result_limit: 4096,
    };
    assert!(matches!(
        handler::prepare(
            request.clone(),
            handle.session,
            pse_diagnostics::Cancellation::default()
        ),
        Err(WorkerError::Provider(ProviderError::Contract(_)))
    ));
    let mut admitted = request;
    admitted.configuration = different.configuration_key();
    handler::prepare(
        admitted,
        handle.session,
        pse_diagnostics::Cancellation::default(),
    )
    .unwrap();
}

#[test]
fn active_native_cancellation_reaps_before_releasing_admission() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("calls");
    let dispatch = UnixDatagram::bind(marker.with_extension("dispatch")).unwrap();
    dispatch
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let control = pse_diagnostics::Cancellation::default();
    let flag = control.flag();
    let scope = ExecutionScope::new(
        control.clone(),
        Some(Instant::now() + Duration::from_secs(60)),
    );
    let (mut binding, request, pid, prepared_reservation) =
        provider("cancel-dispatch", &marker, scope, &service, &pool);
    assert!(pool.reserved() > baseline);
    let cancellation = std::thread::spawn(move || {
        observe_phase(&dispatch, DISPATCHED).unwrap();
        control.cancel();
    });
    let context = EvaluationContext {
        cancelled: &flag,
        max_result_bytes: 4096,
    };
    let mut buffer = ProviderBuffer::new(&request);
    let error = buffer
        .fill(
            &mut binding,
            &[500.0, 20.0],
            &request,
            ProviderRuntime::default(),
            &context,
        )
        .unwrap_err();
    assert!(matches!(
        original_provider_error(&error),
        ProviderError::Cancelled
    ));
    cancellation.join().unwrap();
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(pool.reserved(), prepared_reservation);
    drop(binding);
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn independent_attempts_share_admission_without_sharing_process_or_stop_context() {
    let directory = tempfile::tempdir().unwrap();
    let blocked_marker = directory.path().join("blocked");
    let ready_marker = directory.path().join("ready");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let stop_a = pse_diagnostics::Cancellation::default();
    let flag_a = stop_a.flag();
    let stop_b = pse_diagnostics::Cancellation::default();
    let flag_b = stop_b.flag();
    let (mut a, request_a, pid_a, _) = provider(
        "delay",
        &blocked_marker,
        ExecutionScope::new(
            stop_a.clone(),
            Some(Instant::now() + Duration::from_secs(60)),
        ),
        &service,
        &pool,
    );
    let (mut b, request_b, pid_b, _) = provider(
        "ready",
        &ready_marker,
        ExecutionScope::new(stop_b, Some(Instant::now() + Duration::from_secs(60))),
        &service,
        &pool,
    );
    assert_ne!(pid_a, pid_b);
    let blocked = std::thread::spawn(move || {
        let mut buffer = ProviderBuffer::new(&request_a);
        let error = buffer
            .fill(
                &mut a,
                &[500., 20.],
                &request_a,
                ProviderRuntime::default(),
                &EvaluationContext {
                    cancelled: &flag_a,
                    max_result_bytes: 4096,
                },
            )
            .unwrap_err();
        matches!(original_provider_error(&error), ProviderError::Cancelled)
    });
    let until = Instant::now() + Duration::from_secs(10);
    while !blocked_marker.exists() {
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(2));
    }
    let mut buffer_b = ProviderBuffer::new(&request_b);
    let context_b = EvaluationContext {
        cancelled: &flag_b,
        max_result_bytes: 4096,
    };
    assert!(
        buffer_b
            .fill(
                &mut b,
                &[500., 20.],
                &request_b,
                ProviderRuntime::default(),
                &context_b
            )
            .is_ok()
    );
    assert!(pool.reserved() > baseline);
    stop_a.cancel();
    assert!(
        blocked.join().unwrap(),
        "the blocked attempt must observe its original cancellation"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid_a}")).exists());
    assert!(!flag_b.load(std::sync::atomic::Ordering::Acquire));
    assert!(
        buffer_b
            .fill(
                &mut b,
                &[500., 20.],
                &request_b,
                ProviderRuntime::default(),
                &context_b
            )
            .is_ok()
    );
    drop(b);
    assert!(!std::path::Path::new(&format!("/proc/{pid_b}")).exists());
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn original_deadline_expires_during_native_work_without_late_completion() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("calls");
    let (service, pool, _native) = service();
    let baseline = pool.reserved();
    let control = pse_diagnostics::Cancellation::default();
    let flag = control.flag();
    let scope = ExecutionScope::new(control, Some(Instant::now() + Duration::from_secs(10)));
    let (mut binding, request, pid, prepared_reservation) =
        provider("delay", &marker, scope, &service, &pool);
    let context = EvaluationContext {
        cancelled: &flag,
        max_result_bytes: 4096,
    };
    let mut buffer = ProviderBuffer::new(&request);
    let error = buffer
        .fill(
            &mut binding,
            &[500.0, 20.0],
            &request,
            ProviderRuntime::default(),
            &context,
        )
        .unwrap_err();
    assert!(
        matches!(original_provider_error(&error), ProviderError::Deadline),
        "original scientific deadline must survive settlement: {error:?}"
    );
    assert!(!flag.load(std::sync::atomic::Ordering::Acquire));
    assert!(
        marker.exists(),
        "deadline control must reach actual native dispatch"
    );
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    assert_eq!(pool.reserved(), prepared_reservation);
    drop(binding);
    assert_eq!(pool.reserved(), baseline);
}

#[test]
fn parent_process() {
    let Some(marker) = std::env::var_os("PSE_SCIENCE_PARENT_MARKER") else {
        return;
    };
    let marker = std::path::PathBuf::from(marker);
    let (service, pool, _native) = service();
    let control = pse_diagnostics::Cancellation::default();
    let flag = control.flag();
    let scope = ExecutionScope::new(control, Some(Instant::now() + Duration::from_secs(60)));
    let (mut binding, request, pid, _) = provider("delay", &marker, scope, &service, &pool);
    std::fs::write(marker.with_extension("pid"), pid.to_string()).unwrap();
    let context = EvaluationContext {
        cancelled: &flag,
        max_result_bytes: 4096,
    };
    let mut buffer = ProviderBuffer::new(&request);
    let _ = buffer.fill(
        &mut binding,
        &[500.0, 20.0],
        &request,
        ProviderRuntime::default(),
        &context,
    );
}

#[test]
fn parent_loss_terminates_actual_active_native_process() {
    struct Parent(std::process::Child);
    impl Drop for Parent {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("parent-loss");
    let mut parent = Parent(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "scientific_worker::proxy::tests::parent_process",
                "--nocapture",
            ])
            .env("PSE_SCIENCE_PARENT_MARKER", &marker)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let end = Instant::now() + Duration::from_secs(30);
    while !marker.exists() {
        assert!(
            Instant::now() < end,
            "controlled parent did not dispatch native work"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let pid: u32 = std::fs::read_to_string(marker.with_extension("pid"))
        .unwrap()
        .parse()
        .unwrap();
    use pse_diagnostics::process_facts::{ProcessRead, ProcessReader};
    let reader = ProcessReader::default();
    let ProcessRead::Present(original) = reader.process(pid) else {
        panic!("actual active child identity unavailable")
    };
    parent.0.kill().unwrap();
    parent.0.wait().unwrap();
    let stopped = Instant::now() + Duration::from_secs(3);
    loop {
        let same_process = match reader.stat(pid) {
            ProcessRead::Absent => false,
            ProcessRead::Present(stat) => stat.start_ticks == original.identity.start_ticks,
            other => panic!("original child lifetime observation unavailable: {other:?}"),
        };
        let alive = if same_process {
            match std::fs::read_dir(format!("/proc/{pid}/task")) {
                Ok(mut tasks) => tasks.any(|task| {
                    let tid = task.unwrap().file_name().to_str().unwrap().parse().unwrap();
                    match reader.stat(tid) {
                        ProcessRead::Present(stat) => !matches!(stat.state, 'Z' | 'X'),
                        ProcessRead::Absent => false,
                        other => {
                            panic!("child thread lifetime observation unavailable: {other:?}")
                        }
                    }
                }),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => panic!("child thread inventory unavailable: {error}"),
            }
        } else {
            false
        };
        if !alive {
            break;
        }
        assert!(
            Instant::now() < stopped,
            "native child survived original parent loss"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
