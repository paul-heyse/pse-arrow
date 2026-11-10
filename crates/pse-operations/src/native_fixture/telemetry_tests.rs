// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A real Collector is the independent native export receiver; no substitute receiver is used.
use crate::native_service::{
    NativeCanonicalService, NativePersistenceObservation, NativeTelemetry,
};
use opentelemetry::trace::TracerProvider as _;
use pse_host::{HostManager, LaunchIntent, ResourceControls, ServiceSpec};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use tracing::instrument::WithSubscriber as _;
use tracing_subscriber::prelude::*;

#[derive(Clone, Debug)]
struct RequestSeen(std::sync::Arc<std::sync::atomic::AtomicBool>);
impl opentelemetry_sdk::trace::SpanExporter for RequestSeen {
    async fn export(
        &self,
        batch: Vec<opentelemetry_sdk::trace::SpanData>,
    ) -> opentelemetry_sdk::error::OTelSdkResult {
        if batch
            .iter()
            .any(|span| span.name == crate::transport::client::REQUEST_SPAN_NAME)
        {
            self.0.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }
}

#[tokio::test]
async fn native_server_metrics_reach_owned_collector_with_original_incarnation() {
    let work_end = Instant::now() + Duration::from_secs(180);
    let final_end = work_end + Duration::from_secs(60);
    let collector = std::env::var_os("PSE_NATIVE_FIXTURE_COLLECTOR")
        .expect("admitted checksum-verified Collector");
    // Observe the actual client's existing request span through a scoped SDK
    // exporter. This does not install a global logger or a second trace receiver.
    let request_seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let traces = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_simple_exporter(RequestSeen(request_seen.clone()))
        .build();
    let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(
        tracing_opentelemetry::layer().with_tracer(traces.tracer("native-collector-fixture")),
    ));
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("metrics.jsonl");
    let config = directory.path().join("collector.json");
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    std::fs::write(&config,serde_json::to_vec(&serde_json::json!({
        "receivers":{"otlp":{"protocols":{"grpc":{"endpoint":address.to_string()}}}},
        "exporters":{"file":{"path":output,"flush_interval":"100ms"}},
        "service":{"telemetry":{"metrics":{"level":"none","readers":[]}},"pipelines":{"metrics":{"receivers":["otlp"],"exporters":["file"]}}}
    })).unwrap()).unwrap();
    let manager = HostManager::connect_user(None).await.unwrap();
    let enclosing = manager.current_unit().await.unwrap();
    let intent = LaunchIntent::fresh().unwrap();
    let effect = pse_host::fixtures::target_settlement::TargetEffect::begin().unwrap();
    let selected = ServiceSpec {
        intent: intent.clone(),
        executable: collector.into(),
        arguments: vec!["--config".into(), config.to_string_lossy().into_owned()],
        environment: vec!["PATH=/usr/bin:/bin".into(), "LANG=C.UTF-8".into()],
    };
    let start = manager
        .start_service_bound(
            &selected,
            &manager.borrow(enclosing.unit),
            &ResourceControls::default(),
            work_end,
        )
        .await;
    let mut unit = match start {
        Ok(unit) => unit,
        Err(error) => {
            if let Some(mut original) = manager
                .reconcile_if_present(&intent, final_end)
                .await
                .unwrap()
            {
                original.stop_and_drain(final_end).await.unwrap();
            }
            if let Some(effect) = effect {
                effect.settle().unwrap();
            }
            panic!("Collector launch failed: {error}")
        }
    };
    let result: Result<NativePersistenceObservation, crate::native_service::NativeServiceError> =
        async {
            while tokio::net::TcpStream::connect(address).await.is_err() {
                let observed = unit.observe().await?;
                if matches!(observed.active_state.as_str(), "failed" | "inactive") {
                    return Err(pse_host::HostError::Ownership(format!(
                        "original Collector {} exited before listener readiness",
                        intent.unit()
                    ))
                    .into());
                }
                if Instant::now() >= work_end {
                    return Err(crate::native_service::NativeServiceError::Refused(
                        "Collector listener did not become ready",
                    ));
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            let mut spec = super::owned_test_service_spec();
            spec.telemetry = Some(NativeTelemetry::collector(
                format!("http://{address}"),
                BTreeMap::new(),
                100,
            )?);
            let mut service = NativeCanonicalService::new(spec)?;
            let received = service
                .run_until(work_end, final_end, |service| {
                    Box::pin(async move {
                        service.launch(work_end).await?;
                        service
                            .fixture_query("RETURN 17;", surrealdb_types::Object::new())
                            .await?;
                        let direct = service.inspect_persistence(work_end).await?;
                        if direct.source
                            != crate::native_service::NativeMetricSource::OriginalServer
                            || !direct.gauges.contains_key("surrealmx.live_versions")
                        {
                            return Err(crate::native_service::NativeServiceError::Refused(
                                "original authenticated MX metric was not observed",
                            ));
                        }
                        loop {
                            if let Ok(bytes) = std::fs::read(&output) {
                                for line in bytes
                                    .split(|&byte| byte == b'\n')
                                    .filter(|line| !line.is_empty())
                                {
                                    if let Ok(observed) = service.persistence_observation(line, 0) {
                                        return Ok(observed);
                                    }
                                }
                            }
                            if Instant::now() >= work_end {
                                return Err(crate::native_service::NativeServiceError::Refused(
                                    "original native MX metric was not delivered",
                                ));
                            }
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    })
                })
                .with_subscriber(dispatch)
                .await;
            if received.is_ok() {
                if crate::native_service::inspect_owned_persistence(service.state(), work_end)
                    .await
                    .is_ok()
                {
                    return Err(crate::native_service::NativeServiceError::Refused(
                        "passive inspection accepted a stopped original producer",
                    ));
                }
                service.dispose()?;
            }
            received
        }
        .await;
    let settled = unit.stop_and_drain(final_end).await;
    if settled.is_ok() {
        if let Some(effect) = effect {
            effect.settle().unwrap();
        }
    }
    settled.unwrap();
    traces.force_flush().unwrap();
    traces.shutdown().unwrap();
    let observed = result.unwrap();
    assert!(
        request_seen.load(std::sync::atomic::Ordering::SeqCst),
        "the actual canonical request span must reach its selected SDK exporter"
    );
    assert!(observed.gauges.contains_key("surrealmx.live_versions"));
    assert!(!observed.recorded_versions_available);
    assert!(!observed.manual_snapshot_available);
}
