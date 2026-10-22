// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Executable-owned canonical configuration and complete outer attestation.

// A concrete synchronous code address in this actual worker composition module.
fn local_code_anchor() {}

/// Signals the owned native job if its startup future is dropped.
struct StartupInterrupt {
    listener: tokio::task::JoinHandle<()>,
    cancellation: pse_runtime::CancelSource,
}

impl StartupInterrupt {
    async fn finish(mut self) {
        self.listener.abort();
        let _ = (&mut self.listener).await;
    }
}

impl Drop for StartupInterrupt {
    fn drop(&mut self) {
        self.cancellation.cancel();
        self.listener.abort();
    }
}

pub(super) async fn open(
    database: Option<&str>,
    math: &std::sync::Arc<pse_runtime::math::MathService>,
) -> Result<pse_runtime::workflow::CanonicalDeployment, String> {
    let cancelled = pse_runtime::CancelSource::new();
    // The serving source does not exist yet. Own interruption before either
    // strict receipt work or local receiving observation can enter native work.
    let mut interrupt =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .map_err(|error| format!("deployment startup interrupt listener: {error}"))?;
    let stopped = cancelled.clone();
    let listener = tokio::spawn(async move {
        if interrupt.recv().await.is_some() {
            stopped.cancel();
        }
    });
    let interrupt = StartupInterrupt {
        listener,
        cancellation: cancelled.clone(),
    };
    let actual = std::env::current_exe().map_err(|error| error.to_string())?;
    let receipt = std::env::var_os("PSE_PRODUCER_RECEIPT").map(std::path::PathBuf::from);
    let configuration = std::sync::Arc::new(|| {
        #[allow(unsafe_code, reason = "controlled worker observes its immutable reconstruction configuration")]
        // SAFETY: supervised worker reconstruction excludes environment writers.
        let environment = unsafe {
            pse_buildinfo::identity::CurrentEnvironmentSnapshot::capture_scoped(&|| Ok(()))?
        };
        pse_runtime::math::portable::RuntimeConfigurationObservation::new(Vec::new(), environment)
    });
    let state = std::env::var_os("PSE_SURREAL_STATE")
        .ok_or("PSE_SURREAL_STATE must select a supervised canonical deployment")?;
    let host = pse_runtime::workflow::DeploymentHost {
        artifact: actual.clone(), receipt,
        target: pse_runtime::math::portable::ExpectedProducerTarget::WORKER,
        anchor: local_code_anchor as *const () as usize, configuration,
    };
    #[allow(unsafe_code, reason = "ADR-0164 actual worker asserts strict provenance and controlled local reconstruction")]
    // SAFETY: this is the actual worker artifact; its immutable Rust reconstruction
    // has no caller/plugin/provider callbacks or executable-map mutation. Unknown
    // contexts remain misses, and native operations retain managed generation scope.
    let deployment = unsafe {
        pse_runtime::workflow::CanonicalDeployment::open_observed(
            math, host, std::path::Path::new(&state), database, &cancelled,
            || pse_buildinfo::verify_loaded_module(&actual).map_err(|error| pse_runtime::workflow::WorkflowError::Input(error.to_string())),
        )
    }.await.map_err(|error| error.to_string())?;
    interrupt.finish().await;
    Ok(deployment)
}
