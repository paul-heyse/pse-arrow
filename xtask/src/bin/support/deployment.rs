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
    use pse_runtime::math::portable::{PortableError, ReplayAdmission};

    let deadline = std::time::Instant::now()
        .checked_add(ReplayAdmission::STARTUP_OBSERVATION_LIMIT)
        .ok_or("deployment startup deadline extent")?;
    let cancelled = pse_runtime::CancelSource::new();
    // The serving source does not exist yet. Own interruption before either
    // strict receipt work or local receiving observation can enter native work.
    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
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
    pse_buildinfo::verify_loaded_module(&actual).map_err(|error| error.to_string())?;
    let receipt = std::env::var_os("PSE_PRODUCER_RECEIPT").map(std::path::PathBuf::from);
    #[allow(
        unsafe_code,
        reason = "actual worker composition root supplies reviewed strict deployment provenance"
    )]
    // SAFETY: this is the actual worker artifact; the operator owns the selected
    // capture contract. The native operation verifies current consumed inputs.
    let startup = unsafe {
        math.observe_deployment_runtime(
            actual.clone(),
            receipt.clone(),
            pse_runtime::math::portable::ExpectedProducerTarget::WORKER,
            deadline,
            &cancelled,
        )
    }
    .await
    .map_err(|error| error.to_string())?;
    let producer = if receipt.is_some() {
        startup.producer.clone().map(Into::into)
    } else {
        // SAFETY: the controlled worker's own code observes its receiving artifact.
        // Reconstruction is immutable Rust math without caller/plugin/provider
        // callbacks or direct executable-map mutation. Native operations retain
        // their separate managed-generation lifetime; unknown context is a miss.
        #[allow(
            unsafe_code,
            reason = "ADR-0164 controlled worker deployment-local reconstruction admission"
        )]
        // SAFETY: this actual worker module supplies its own observed code address;
        // reconstruction follows the controlled immutable contract described above.
        let observed = unsafe {
            math.observe_local_runtime(
                pse_runtime::math::portable::ExpectedProducerTarget::WORKER,
                local_code_anchor as *const () as usize,
                std::sync::Arc::new(|| {
                    // SAFETY: this supervised worker's controlled reconstruction
                    // lifetime excludes all current-environment writers.
                    let environment =
                        pse_buildinfo::identity::CurrentEnvironmentSnapshot::capture_scoped(
                            &|| Ok(()),
                        )?;
                    pse_runtime::math::portable::RuntimeConfigurationObservation::new(
                        Vec::new(),
                        environment,
                    )
                }),
                deadline,
                &cancelled,
            )
        }
        .await;
        match observed {
            Ok(admission) => Some(admission),
            Err(PortableError::Qualification(_)) => None,
            Err(error) => return Err(error.to_string()),
        }
    };
    pse_buildinfo::verify_loaded_module(&actual).map_err(|error| error.to_string())?;
    // Startup refusal has no canonical connection/open effect. Keep the compact
    // observation lease through this subsequent connection and publication boundary.
    cancelled.checkpoint().map_err(|error| error.to_string())?;
    let state = std::env::var_os("PSE_SURREAL_STATE")
        .ok_or("PSE_SURREAL_STATE must select a supervised canonical deployment")?;
    let options = pse_operations::canonical::CanonicalOptions::from_state_for_database(
        std::path::Path::new(&state),
        database,
    )
    .map_err(|e| e.to_string())?;
    let store = pse_operations::canonical::CanonicalStore::connect(&options)
        .await
        .map_err(|e| e.to_string())?;
    cancelled.checkpoint().map_err(|error| error.to_string())?;
    store.open().await.map_err(|e| e.to_string())?;
    cancelled.checkpoint().map_err(|error| error.to_string())?;
    interrupt.finish().await;
    Ok(pse_runtime::workflow::CanonicalDeployment::new(
        store,
        pse_runtime::workflow::OuterAttestation {
            source: startup.outer.source,
            build: startup.outer.build,
        },
        producer,
    ))
}
