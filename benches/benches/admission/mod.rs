// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Resolve the selected Criterion workload before opening a fixture or scientific runtime.
use pse_benches::CriterionMode;
use serde_json::{Value, json};
use std::{error::Error, io, path::Path};

pub(crate) const PREPARED: &str = "PSE_BENCHMARK_PREPARED";
pub(crate) const CANONICAL_BUILD: &str = "PSE_BENCHMARK_CANONICAL_BUILD";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuthoredInput {
    Physical,
    SeedTransport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Inputs {
    pub(crate) canonical: bool,
    pub(crate) solver_feature: Option<&'static str>,
    pub(crate) authored: Option<AuthoredInput>,
    pub(crate) unavailable: Option<&'static str>,
}

impl Inputs {
    pub(crate) const fn local(self) -> bool {
        !self.canonical && self.solver_feature.is_none()
    }

    fn view(self) -> Value {
        json!({
            "canonical": self.canonical,
            "solver_feature": self.solver_feature,
            "authored_input": self.authored.map(|input| match input {
                AuthoredInput::Physical => "physical-package",
                AuthoredInput::SeedTransport => "seed-transport",
            }),
        })
    }
}

pub(crate) fn preparation_inputs(workload: &Value) -> Inputs {
    let local_scalar = workload["model"] == "scalar-k4"
        && (workload["demand_orders"] == true || workload["worker_permit_lifetime"] == true);
    Inputs {
        canonical: !local_scalar,
        solver_feature: (workload["realization"] == "nested").then_some("solver-kinsol"),
        authored: None,
        unavailable: None,
    }
}

pub(crate) fn process_inputs(workload: &Value) -> Inputs {
    let Some(operation) = workload["operation"].as_str() else {
        return Inputs {
            canonical: true,
            solver_feature: Some("native-process"),
            authored: None,
            unavailable: Some("registered process workload requires a string operation"),
        };
    };
    let authored = match operation {
        "source-edit-copy"
        | "result_analysis"
        | "k4-study"
        | "k4-restart"
        | "k4-accuracy"
        | "compact-implicit-derivatives" => None,
        "heater"
        | "flash"
        | "cancellation"
        | "results"
        | "vessel"
        | "dynamic-rebind"
        | "fit"
        | "evented-fit"
        | "compact-reference"
        | "compact-property-value" => Some(AuthoredInput::SeedTransport),
        _ => Some(AuthoredInput::Physical),
    };
    Inputs {
        canonical: !(operation.starts_with("compact-") || operation.starts_with("review-")),
        // Cargo's native_process target requires this feature even for compact work.
        solver_feature: Some("native-process"),
        authored,
        unavailable: (workload["adapter"] == "managed-durable")
            .then_some("requires a database-matched managed primary allocation and receiver"),
    }
}

pub(crate) fn selection_record(
    target: &str,
    mode: CriterionMode,
    selected: &[&Value],
    registered: &[Value],
    inputs: impl Fn(&Value) -> Inputs,
) -> Value {
    let prepared_only = registered.iter().filter_map(|workload| {
        let inputs = inputs(workload);
        (!inputs.local() && inputs.unavailable.is_none())
            .then(|| json!({"id": workload["id"], "inputs": inputs.view()}))
    });
    let unavailable = registered.iter().filter_map(|workload| {
        inputs(workload)
            .unavailable
            .map(|reason| json!({"id": workload["id"], "reason": reason}))
    });
    json!({
        "experiment": "benchmark_selection",
        "target": target,
        "mode": mode.label(),
        "selected": selected.iter().map(|workload| &workload["id"]).collect::<Vec<_>>(),
        "prepared_only": prepared_only.collect::<Vec<_>>(),
        "unavailable": unavailable.collect::<Vec<_>>(),
    })
}

/// Return true when this process has handed off the workload to the fixture child.
pub(crate) fn admit(
    inputs: Inputs,
    mode: CriterionMode,
    output: Option<&Path>,
    scientific_inputs: Option<&Path>,
) -> Result<bool, Box<dyn Error>> {
    if inputs.local() || mode == CriterionMode::List {
        return Ok(false);
    }
    if let Some(reason) = inputs.unavailable {
        return Err(io::Error::other(reason).into());
    }
    if std::env::var_os(PREPARED).as_deref() != Some(std::ffi::OsStr::new("1")) {
        return Err(
            io::Error::other("this case requires the explicit prepared benchmark route").into(),
        );
    }
    if !matches!(mode, CriterionMode::Smoke | CriterionMode::Measure) {
        return Err(io::Error::other("prepared benchmark mode must be smoke or measure").into());
    }
    if let Some(feature) = inputs.solver_feature {
        let linked = match feature {
            "solver-kinsol" => cfg!(any(feature = "solver-kinsol", feature = "native-process")),
            "native-process" => cfg!(feature = "native-process"),
            _ => false,
        };
        if !linked {
            return Err(io::Error::other(format!(
                "prepared solver case requires Cargo feature {feature}"
            ))
            .into());
        }
    }
    if !inputs.canonical {
        return Ok(false);
    }
    if std::env::var_os(pse_operations::native_fixture::SELECTION_ENV).is_some() {
        pse_operations::native_fixture::FixtureSelection::selected()?;
        return Ok(false);
    }
    let output = output
        .ok_or_else(|| io::Error::other("canonical benchmark requires an output directory"))?;
    let helper_build = std::env::var_os(CANONICAL_BUILD).ok_or_else(|| {
        io::Error::other("canonical benchmark requires a prepared service helper capture")
    })?;
    if inputs.authored.is_some() && scientific_inputs.is_none() {
        return Err(
            io::Error::other("canonical benchmark requires a retained scientific capsule").into(),
        );
    }
    launch_fixture(Path::new(&helper_build), output, scientific_inputs)
}

#[cfg(feature = "prepared-benchmark")]
fn launch_fixture(
    helper_build: &Path,
    output: &Path,
    scientific_inputs: Option<&Path>,
) -> Result<bool, Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("benchmark checkout root unavailable"))?;
    let status = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(pse_operations::native_fixture::run_benchmark_fixture(
            root,
            helper_build,
            output.join("fixture-outcome.json"),
            scientific_inputs,
        ))?;
    if status.success() {
        Ok(true)
    } else {
        Err(io::Error::other(format!("original benchmark child exited with {status}")).into())
    }
}

#[cfg(not(feature = "prepared-benchmark"))]
fn launch_fixture(
    _helper_build: &Path,
    _output: &Path,
    _scientific_inputs: Option<&Path>,
) -> Result<bool, Box<dyn Error>> {
    Err(
        io::Error::other("canonical fixture owner requires Cargo feature prepared-benchmark")
            .into(),
    )
}
