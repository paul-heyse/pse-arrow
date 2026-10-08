// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Executable-owned canonical configuration and complete outer attestation.

// A concrete synchronous code address in this actual worker composition module.
fn local_code_anchor() {}

pub(super) async fn open(
    database: Option<&str>,
) -> Result<pse_runtime::workflow::CanonicalDeployment, String> {
    let state = std::env::var_os("PSE_SURREAL_STATE")
        .ok_or("PSE_SURREAL_STATE must select a supervised canonical deployment")?;
    let mut options =
        pse_operations::canonical::CanonicalOptions::from_state(std::path::Path::new(&state))
            .map_err(|e| e.to_string())?;
    if let Some(database) = database {
        options.database = database.to_owned();
    }
    let store = pse_operations::canonical::CanonicalStore::connect(&options)
        .await
        .map_err(|e| e.to_string())?;
    store.open().await.map_err(|e| e.to_string())?;
    let actual = std::env::current_exe().map_err(|error| error.to_string())?;
    pse_buildinfo::verify_loaded_module(&actual).map_err(|error| error.to_string())?;
    let outer = pse_buildinfo::observe_deployment(&actual).map_err(|error| error.to_string())?;
    let attestation = pse_runtime::workflow::OuterAttestation {
        source: outer.source,
        build: outer.build,
    };
    let receipt = std::env::var_os("PSE_PRODUCER_RECEIPT");
    let strict = receipt.as_ref().map(|path| {
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        let artifact = pse_buildinfo::verify_receipt_artifact(&bytes, &actual).map_err(|error| error.to_string())?;
        // SAFETY: the operator selects the reviewed actual capture; current_exe
        // and current consumed inputs independently establish its association.
        #[allow(unsafe_code, reason = "ADR-0164 controlled worker deployment qualification; no unsafe memory operation")]
        unsafe { pse_runtime::math::portable::QualifiedProducer::from_deployment_receipt(&bytes, &artifact.sha256, pse_runtime::math::portable::ExpectedProducerTarget::WORKER) }
            .map_err(|error| error.to_string())
    }).transpose()?.flatten();
    let producer = if receipt.is_some() {
        strict.map(Into::into)
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
        unsafe {
            pse_runtime::math::portable::ReplayAdmission::observe_local(
                pse_runtime::math::portable::ExpectedProducerTarget::WORKER,
                local_code_anchor as *const () as usize,
                std::sync::Arc::new(|| Ok(Vec::new())),
            )
        }
        .ok()
    };
    Ok(pse_runtime::workflow::CanonicalDeployment::new(
        store,
        attestation,
        producer,
    ))
}
