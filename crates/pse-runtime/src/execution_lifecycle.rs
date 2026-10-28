// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Session and Python populations share facts, never native-job counters.
use pse_diagnostics::{lifecycle::LifecycleBoundary, observation::ObservationContext};
/// A session reports its own request/join boundaries, independently of job populations.
pub(crate) struct NativeSessionLifecycle(Option<ObservationContext>);
impl NativeSessionLifecycle {
    pub(crate) fn for_context(context: Option<ObservationContext>) -> Self { Self(context) }
    pub(crate) fn record(&self, boundary: LifecycleBoundary, value: bool) {
        if let Some(context) = &self.0 { context.boolean(boundary.native_session_key(), value); }
        let code = match boundary {
            LifecycleBoundary::StopRequested => "native.cancel.requested",
            _ => boundary.native_session_key(),
        };
        tracing::info!(target: "pse.telemetry.diagnostic", code, value, "native session lifecycle");
    }
}
/// Python pull completion emits observation and tracing without counting a native job join.
#[derive(Debug)]
pub struct PythonPullLifecycle {
    observation: Option<ObservationContext>,
    span: tracing::Span,
}
impl PythonPullLifecycle {
    /// Capture the already established Python ingress; never installs a subscriber/recorder.
    pub fn new(span: tracing::Span) -> Self {
        Self { observation: pse_diagnostics::observation::current(), span }
    }
    /// Report cooperative stop while the performing pull remains owned.
    pub fn stop_requested(&self) {
        if let Some(context) = &self.observation { context.boolean("python.native_pull.stop_requested", true); }
        crate::tooling::telemetry::lifecycle(&self.span, "native.cancel.requested", "python.native_pull");
    }
    /// The pull has actually joined, including native/TLS destruction.
    pub fn drain_settled(&self) {
        if let Some(context) = &self.observation { context.boolean("python.native_pull.joined", true); }
        crate::tooling::telemetry::lifecycle(&self.span, "native.drain.settled", "python.native_pull");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    #[tokio::test]
    async fn sessions_and_python_pulls_keep_distinct_hub_populations_without_subscriber() {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("independent populations");
        let before = crate::tooling::operational_metrics::snapshot();
        pse_diagnostics::observation::scope(Some(operation.context()), async {
            NativeSessionLifecycle::for_context(pse_diagnostics::observation::current()).record(LifecycleBoundary::Joined, true);
            PythonPullLifecycle::new(tracing::Span::none()).drain_settled();
        }).await;
        let after = crate::tooling::operational_metrics::snapshot();
        assert_eq!(after.native_joined, before.native_joined);
        assert_eq!(after.native_attempts, before.native_attempts);
        let snapshot = operation.reader().snapshot(std::time::Duration::from_secs(60));
        assert!(snapshot.metrics.iter().any(|metric| metric.name == LifecycleBoundary::Joined.native_session_key() && metric.latest == Value::Boolean(true)));
        assert!(snapshot.metrics.iter().any(|metric| metric.name == "python.native_pull.joined" && metric.latest == Value::Boolean(true)));
        assert!(!snapshot.metrics.iter().any(|metric| metric.name == LifecycleBoundary::Joined.native_job_key()));
    }
}
