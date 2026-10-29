// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native sessions report their own lifecycle facts, never native-job counters.
use pse_diagnostics::{lifecycle::LifecycleBoundary, observation::ObservationContext};
/// A session reports its own request/join boundaries, independently of job populations.
pub(crate) struct NativeSessionLifecycle(Option<ObservationContext>);
impl NativeSessionLifecycle {
    pub(crate) fn for_context(context: Option<ObservationContext>) -> Self {
        Self(context)
    }
    pub(crate) fn record(&self, boundary: LifecycleBoundary, value: bool) {
        if let Some(context) = &self.0 {
            context.boolean(boundary.native_session_key(), value);
        }
        let code = match boundary {
            LifecycleBoundary::StopRequested => "native.cancel.requested",
            _ => boundary.native_session_key(),
        };
        tracing::info!(target: "pse.telemetry.diagnostic", code, value, "native session lifecycle");
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    #[tokio::test]
    async fn native_sessions_keep_job_populations_distinct_without_subscriber() {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("independent populations");
        let before = crate::tooling::operational_metrics::snapshot();
        pse_diagnostics::observation::scope(Some(operation.context()), async {
            NativeSessionLifecycle::for_context(pse_diagnostics::observation::current())
                .record(LifecycleBoundary::Joined, true);
        })
        .await;
        let after = crate::tooling::operational_metrics::snapshot();
        assert_eq!(after.native_joined, before.native_joined);
        assert_eq!(after.native_attempts, before.native_attempts);
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert!(snapshot.metrics.iter().any(|metric| metric.name
            == LifecycleBoundary::Joined.native_session_key()
            && metric.latest == Value::Boolean(true)));
        assert!(
            !snapshot
                .metrics
                .iter()
                .any(|metric| metric.name == LifecycleBoundary::Joined.native_job_key())
        );
    }
}
