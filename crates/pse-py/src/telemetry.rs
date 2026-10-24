// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private opt-in process-host composition. Importing the extension has no SDK effects.
use std::{collections::BTreeMap, sync::{Mutex, OnceLock, atomic::{AtomicBool, Ordering}}};
use pyo3::{prelude::*, types::{PyDict, PyString}};
use pse_runtime::tooling::telemetry::{Carrier, Host, Settings};
struct Registration { host: Host, injector: Py<PyAny>, incarnation: String, active: bool }
static HOST: OnceLock<Mutex<Option<Registration>>> = OnceLock::new();
static INITIALIZING: AtomicBool = AtomicBool::new(false);
struct Initializing;
impl Drop for Initializing { fn drop(&mut self) { INITIALIZING.store(false, Ordering::Release); } }
fn host_thread<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T, &'static str> {
    std::thread::Builder::new().name("pse-telemetry-host".into()).spawn(work)
        .map_err(|_| "telemetry host thread unavailable")?.join().map_err(|_| "telemetry host thread failed")
}
fn state() -> &'static Mutex<Option<Registration>> { HOST.get_or_init(|| Mutex::new(None)) }
fn activation(status: &str, incarnation: Option<&str>) -> String {
    let sdk = pse_runtime::tooling::telemetry::sdk_metadata();
    serde_json::json!({"status":status,"process_incarnation":incarnation,
        "sdk_version":sdk.get("telemetry.sdk.version"),"sdk_language":sdk.get("telemetry.sdk.language"),
        "sdk_name":sdk.get("telemetry.sdk.name"),"transport":"http/protobuf",
        "scope":pse_runtime::tooling::telemetry::INSTRUMENTATION_SCOPE,"lifecycle_owner":"python-process-host"}).to_string()
}
/// Called only by an explicit Python process host with resolved settings and a fresh injector.
#[pyfunction]
#[pyo3(signature = (settings_json, injector: "typing.Callable[[], dict[str, str]]") -> "str")]
pub(crate) fn _telemetry_initialize(py: Python<'_>, settings_json: &str, injector: Py<PyAny>) -> PyResult<String> {
    if !injector.bind(py).is_callable() { return Err(pyo3::exceptions::PyTypeError::new_err("telemetry injector must be callable")); }
    let settings = Settings::from_json(settings_json).map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
    if INITIALIZING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
        return Ok(activation("initialization_pending", Some(&settings.process_incarnation)));
    }
    let _initializing = Initializing;
    let state_guard = state().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(existing) = state_guard.as_ref() {
        if existing.incarnation != settings.process_incarnation { return Err(pyo3::exceptions::PyRuntimeError::new_err("native telemetry already belongs to another process incarnation")); }
        return Ok(activation("already_initialized", Some(&existing.incarnation)));
    }
    drop(state_guard);
    if settings.resource.get("process.pid").map(String::as_str) != Some(std::process::id().to_string().as_str()) {
        return Err(pyo3::exceptions::PyValueError::new_err("native host process identity differs"));
    }
    // Reqwest blocking client construction and SDK lifecycle happen on a plain thread,
    // even if the embedded caller happens to be running an async host runtime.
    let incarnation = settings.process_incarnation.clone();
    let disabled = settings.disabled || !(settings.traces.enabled || settings.logs.enabled || settings.metrics.enabled);
    let host = py.detach(move || host_thread(move || Host::build(&settings, None)))
        .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("native telemetry initialization thread failed"))?
        .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("native telemetry initialization refused"))?;
    if host.install_global().is_err() {
        let _ = py.detach(move || host_thread(move || host.shutdown()));
        return Ok(activation("initialization_refused_preexisting_subscriber", Some(&incarnation)));
    }
    *state().lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Registration { host, injector, incarnation: incarnation.clone(), active: !disabled });
    Ok(activation(if disabled { "disabled" } else { "initialized" }, Some(&incarnation)))
}
/// The process host calls this only after native ownership and result publication settle.
#[pyfunction]
pub(crate) fn _telemetry_shutdown(py: Python<'_>) -> PyResult<String> {
    let registration = state().lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
    let Some(registration) = registration else { return Ok(activation("not_initialized", None)); };
    let incarnation = registration.incarnation;
    let result = py.detach(move || host_thread(move || registration.host.shutdown()));
    let status = match result {
        Ok(Ok(())) => "shutdown",
        _ => {
            tracing::warn!(target: "pse.telemetry.diagnostic", code = "telemetry.shutdown.failed", "native telemetry shutdown failed after owner settlement");
            "shutdown_failed"
        }
    };
    Ok(activation(status, Some(&incarnation)))
}
/// Resolve CURRENT Python context before detach, never cached during initialization.
/// Invalid/missing diagnostic context cannot reject native scientific work.
pub(crate) fn ingress(py: Python<'_>, owner: &'static str) -> tracing::Span {
    let injector = state().lock().unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref().filter(|registration| registration.active).map(|registration| registration.injector.clone_ref(py));
    let Some(injector) = injector else { return tracing::Span::current(); };
    let carrier = (|| -> PyResult<Carrier> {
        let value = injector.call0(py)?;
        let dictionary = value.bind(py).cast::<PyDict>()?;
        if dictionary.len() > 2 { return Err(pyo3::exceptions::PyValueError::new_err("carrier field bound")); }
        let mut fields = BTreeMap::new();
        for (key,value) in dictionary.iter() {
            let key = key.cast::<PyString>()?;
            let value = value.cast::<PyString>()?;
            let key = key.to_str()?;
            let value = value.to_str()?;
            if key.len() > 11 || value.len() > 512 { return Err(pyo3::exceptions::PyValueError::new_err("carrier size bound")); }
            fields.insert(key.to_owned(),value.to_owned());
        }
        Carrier::from_fields(fields).map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid W3C carrier"))
    })();
    match carrier {
        Ok(carrier) => carrier.operation_span(owner),
        Err(_) => {
            tracing::warn!(target: "pse.telemetry.diagnostic", code = "telemetry.carrier.invalid", owner, "native ingress has no valid Python parent");
            Carrier::default().operation_span(owner)
        }
    }
}
