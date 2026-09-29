// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One integrated experiment, shared by the fit oracle and the shooting oracle (Plan 22 I8):
//! it binds a consumer's parameter values into the integration vector, integrates once, and
//! maps the output sensitivities, or an adjoint product, back to those parameters. The
//! consumers stay separate oracles; this is the integration they have in common.
use pse_backend_native::{self as native, ProblemError};
#[cfg(feature = "solver-diffsol")]
use pse_backend_native::{dynamics::DynamicSensitivity, solve::Execution};
use pse_kernels::Port;

/// A consumer parameter's integration column and the conversion into its unit.
#[derive(Clone, Debug)]
pub(crate) struct Binding {
    /// The integration column in effect at the start: a scheduled input's first interval,
    /// whose later intervals keep their scheduled values (I6).
    pub(crate) local: usize,
    /// The consumer's parameter.
    pub(crate) parameter: usize,
    /// The consumer's value to the integration value.
    pub(crate) conversion: pse_quantity::UnitConvertSpec,
}

/// A prepared dynamic simulation with the consumer's parameter bindings.
#[derive(Clone, Debug)]
pub(crate) struct IntegratedExperiment {
    /// The compiled dynamic functions.
    pub(crate) program: super::dynamics::DynamicProgram,
    /// The prepared integration profile; its sensitivity is what the consumer requested.
    pub(crate) profile: super::SimulationProfile,
    /// The integration vector at the model's values.
    pub(crate) parameters: Vec<f64>,
    /// The physical contract of each output.
    pub(crate) output_ports: Vec<Port>,
    /// The consumer's parameters bound into the integration vector.
    pub(crate) bindings: Vec<Binding>,
}

#[cfg_attr(
    not(feature = "solver-diffsol"),
    expect(dead_code, reason = "integration needs the linked dynamics adapter")
)]
impl IntegratedExperiment {
    /// The integration vector with every binding's consumer value converted into its column.
    pub(crate) fn bind(&self, value: &dyn Fn(usize) -> f64) -> Vec<f64> {
        let mut parameters = self.parameters.clone();
        for binding in &self.bindings {
            parameters[binding.local] =
                value(binding.parameter) * binding.conversion.scale + binding.conversion.offset;
        }
        parameters
    }
    /// The derivative of output `row` at `sample` with respect to the binding's consumer
    /// parameter, from the forward output sensitivities.
    pub(crate) fn response(
        &self,
        sample: &native::dynamics::Sample,
        row: usize,
        binding: &Binding,
    ) -> Result<f64, ProblemError> {
        sample
            .output_sensitivities
            .get(row * self.parameters.len() + binding.local)
            .map(|v| v * binding.conversion.scale)
            .ok_or_else(|| ProblemError::internal("integrated output sensitivity extent"))
    }
}

#[cfg(feature = "solver-diffsol")]
impl IntegratedExperiment {
    /// The shooting window `[start, end]` sampled at `samples` (ADR-0110 Outcome 5): its
    /// profile, whose schedule keeps the changes strictly inside, and the source of each
    /// column of its anchored integration vector — the experiment's column in effect at
    /// the same time, or one of the window's anchors, which follow the unscheduled
    /// contract parameters.
    pub(crate) fn window(
        &self,
        start: f64,
        end: f64,
        samples: Vec<f64>,
    ) -> Result<Window, ProblemError> {
        let np = self.program.contract.parameters.len();
        let anchors = self
            .program
            .contract
            .differential
            .iter()
            .filter(|d| **d)
            .count();
        let profile = self.profile.window(start, end, samples);
        let mut columns = vec![None; profile.integration_width(np + anchors)];
        let times = std::iter::once(start)
            .chain(profile.schedule.iter().flat_map(|s| s.times.iter().copied()));
        for t in times {
            let local = profile.columns_at(np + anchors, t);
            let global = self.profile.columns_at(np, t);
            for (q, column) in local.iter().enumerate() {
                let source = if q < np {
                    WindowColumn::Integration(global[q])
                } else {
                    WindowColumn::Anchor(q - np)
                };
                match columns.get_mut(*column) {
                    Some(slot @ None) => *slot = Some(source),
                    Some(Some(existing)) if *existing == source => {}
                    _ => return Err(ProblemError::internal("shooting window column map")),
                }
            }
        }
        let columns = columns
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| ProblemError::internal("shooting window column without a source"))?;
        Ok(Window { profile, columns })
    }
    /// The window's integration vector from the experiment's and the anchors' values.
    pub(crate) fn window_parameters(
        window: &Window,
        integration: &[f64],
        anchors: &[f64],
    ) -> Result<Vec<f64>, ProblemError> {
        window
            .columns
            .iter()
            .map(|c| match c {
                WindowColumn::Integration(g) => integration.get(*g),
                WindowColumn::Anchor(a) => anchors.get(*a),
            })
            .map(|v| v.copied().ok_or_else(|| ProblemError::internal("shooting window value")))
            .collect()
    }
}

/// Where a shooting window's integration column takes its value (ADR-0110 Outcome 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(feature = "solver-diffsol")]
pub(crate) enum WindowColumn {
    /// A column of the experiment's integration vector.
    Integration(usize),
    /// The start of the window's k-th differential state.
    Anchor(usize),
}
/// One shooting window of an integrated experiment: its unanchored profile and the source
/// of each column of its anchored integration vector.
#[derive(Clone, Debug)]
#[cfg(feature = "solver-diffsol")]
pub(crate) struct Window {
    /// The window's horizon, samples and schedule.
    pub(crate) profile: super::SimulationProfile,
    /// The source of every anchored integration column.
    pub(crate) columns: Vec<WindowColumn>,
}
/// `profile` with the requested derivatives and at most the attempt's remaining time.
#[cfg(feature = "solver-diffsol")]
fn attempt(
    profile: &super::SimulationProfile,
    execution: &Execution,
    sensitivity: DynamicSensitivity,
) -> super::SimulationProfile {
    let mut profile = profile.clone();
    profile.sensitivity = sensitivity;
    profile.time_limit = profile.time_limit.min(
        execution
            .time_limit
            .saturating_sub(execution.started.elapsed()),
    );
    profile
}

#[cfg(feature = "solver-diffsol")]
impl IntegratedExperiment {
    /// Integrate once at the consumer's values, with forward sensitivities or none.
    /// Integrations borrow the admitted outer worker; they never enqueue nested native jobs.
    pub(crate) fn integrate(
        &self,
        value: &dyn Fn(usize) -> f64,
        execution: &Execution,
        sensitivity: DynamicSensitivity,
    ) -> Result<native::dynamics::Report, ProblemError> {
        if sensitivity == DynamicSensitivity::Adjoint {
            return Err(ProblemError::internal(
                "an adjoint product is a gradient, not an integration",
            ));
        }
        let profile = attempt(&self.profile, execution, sensitivity);
        let mut worker = self.program.worker(execution.cancel.clone())?;
        let report = native::dynamics::integrate(
            &mut worker,
            &profile,
            &self.bind(value),
            execution.cancel.clone(),
        )?;
        completed(report)
    }
    /// One forward and one backward pass: the forward report and the gradient of the
    /// cotangent's functional with respect to each binding's consumer parameter, as
    /// `(parameter, value)` pairs. The checkpoints are charged against the attempt's
    /// foreign allowance.
    pub(crate) fn gradient(
        &self,
        value: &dyn Fn(usize) -> f64,
        execution: &Execution,
        cotangent: native::dynamics::Cotangent<'_>,
    ) -> Result<(native::dynamics::Report, Vec<(usize, f64)>), ProblemError> {
        let memory = execution.memory.ok_or_else(|| {
            ProblemError::Contract("adjoint gradients need the worker's admitted foreign allowance".into())
        })?;
        let profile = attempt(&self.profile, execution, DynamicSensitivity::Adjoint);
        let mut worker = self.program.worker(execution.cancel.clone())?;
        let native::dynamics::Gradient {
            report, gradient, ..
        } = native::dynamics::gradient(
            &mut worker,
            &profile,
            &self.bind(value),
            cotangent,
            execution.cancel.clone(),
            memory,
        )?;
        let report = completed(report)?;
        let gradient =
            gradient.ok_or_else(|| ProblemError::internal("completed adjoint without a gradient"))?;
        let contributions = self
            .bindings
            .iter()
            .map(|b| {
                gradient
                    .get(b.local)
                    .map(|g| (b.parameter, g * b.conversion.scale))
                    .ok_or_else(|| ProblemError::internal("adjoint gradient extent"))
            })
            .collect::<Result<_, _>>()?;
        Ok((report, contributions))
    }
}

#[cfg(feature = "solver-idas")]
impl IntegratedExperiment {
    /// One forward pass with state sensitivities and one second-order backward pass: the
    /// forward report, the gradient contributions as in [`Self::gradient`], and the Hessian
    /// of the cotangent's functional with respect to the `selected` bindings' consumer
    /// parameters, row-major over `selected` (ADR-0110 item 4). The checkpoints with their
    /// sensitivities and the backward problems are charged against the attempt's foreign
    /// allowance.
    pub(crate) fn hessian(
        &self,
        value: &dyn Fn(usize) -> f64,
        execution: &Execution,
        selected: &[usize],
        cotangent: native::dynamics::Cotangent<'_>,
    ) -> Result<(native::dynamics::Report, Vec<(usize, f64)>, Vec<f64>), ProblemError> {
        let memory = execution.memory.ok_or_else(|| {
            ProblemError::Contract(
                "second-order adjoints need the worker's admitted foreign allowance".into(),
            )
        })?;
        let bindings = selected
            .iter()
            .map(|b| {
                self.bindings
                    .get(*b)
                    .ok_or_else(|| ProblemError::internal("second-order binding extent"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut directions = bindings.iter().map(|b| b.local).collect::<Vec<_>>();
        directions.sort_unstable();
        directions.dedup();
        let profile = attempt(&self.profile, execution, DynamicSensitivity::Adjoint);
        let mut worker = self.program.worker(execution.cancel.clone())?;
        let native::dynamics::Gradient {
            report,
            gradient,
            hessian,
            ..
        } = native::dynamics::hessian(
            &mut worker,
            &profile,
            &self.bind(value),
            &directions,
            cotangent,
            execution.cancel.clone(),
            memory,
        )?;
        let report = completed(report)?;
        let (gradient, hessian) = gradient.zip(hessian).ok_or_else(|| {
            ProblemError::internal("completed second-order adjoint without its derivatives")
        })?;
        let position = |local: usize| {
            directions
                .binary_search(&local)
                .map_err(|_| ProblemError::internal("second-order direction"))
        };
        let contributions = self
            .bindings
            .iter()
            .map(|b| {
                gradient
                    .get(b.local)
                    .map(|g| (b.parameter, g * b.conversion.scale))
                    .ok_or_else(|| ProblemError::internal("adjoint gradient extent"))
            })
            .collect::<Result<_, _>>()?;
        let mut curvature = Vec::with_capacity(bindings.len() * bindings.len());
        for a in &bindings {
            for b in &bindings {
                curvature.push(
                    hessian[(position(a.local)?, position(b.local)?)]
                        * a.conversion.scale
                        * b.conversion.scale,
                );
            }
        }
        Ok((report, contributions, curvature))
    }
}

#[cfg(feature = "solver-diffsol")]
impl IntegratedExperiment {
    /// Integrate one shooting window from its anchors, with forward sensitivities over its
    /// integration columns or none. The window's oracle anchors every differential start.
    pub(crate) fn integrate_window(
        &self,
        window: &Window,
        integration: &[f64],
        anchors: &[f64],
        execution: &Execution,
        sensitivity: DynamicSensitivity,
    ) -> Result<native::dynamics::Report, ProblemError> {
        if sensitivity == DynamicSensitivity::Adjoint {
            return Err(ProblemError::internal(
                "an adjoint product is a gradient, not an integration",
            ));
        }
        let mut oracle = native::dynamics::Anchored::new(
            self.program.worker(execution.cancel.clone())?,
            false,
        )?;
        let profile = oracle.profile(&attempt(&window.profile, execution, sensitivity));
        let report = native::dynamics::integrate(
            &mut oracle,
            &profile,
            &Self::window_parameters(window, integration, anchors)?,
            execution.cancel.clone(),
        )?;
        completed(report)
    }
    /// One forward and one backward pass over a shooting window whose quadratures are
    /// observed as outputs after the model's own: the forward report and the gradient of
    /// the cotangent's functional over the window's integration columns. The checkpoints
    /// are charged against the attempt's foreign allowance.
    pub(crate) fn window_gradient(
        &self,
        window: &Window,
        integration: &[f64],
        anchors: &[f64],
        execution: &Execution,
        cotangent: native::dynamics::Cotangent<'_>,
    ) -> Result<(native::dynamics::Report, Vec<f64>), ProblemError> {
        let memory = execution.memory.ok_or_else(|| {
            ProblemError::Contract(
                "adjoint gradients need the worker's admitted foreign allowance".into(),
            )
        })?;
        let mut oracle = native::dynamics::Anchored::new(
            self.program.worker(execution.cancel.clone())?,
            true,
        )?;
        let profile =
            oracle.profile(&attempt(&window.profile, execution, DynamicSensitivity::Adjoint));
        let native::dynamics::Gradient {
            report, gradient, ..
        } = native::dynamics::gradient(
            &mut oracle,
            &profile,
            &Self::window_parameters(window, integration, anchors)?,
            cotangent,
            execution.cancel.clone(),
            memory,
        )?;
        let report = completed(report)?;
        let gradient =
            gradient.ok_or_else(|| ProblemError::internal("completed adjoint without a gradient"))?;
        Ok((report, gradient))
    }
}

/// A stopped integration keeps its stop; it is never an evaluation failure.
#[cfg(feature = "solver-diffsol")]
fn completed(
    report: native::dynamics::Report,
) -> Result<native::dynamics::Report, ProblemError> {
    use native::dynamics::Termination;
    match report.termination {
        Termination::Completed => Ok(report),
        Termination::Cancelled => Err(ProblemError::Cancelled),
        Termination::TimeLimit => Err(ProblemError::Limit {
            kind: native::LimitKind::Time,
            detail: "integrated experiment deadline".into(),
        }),
        termination => Err(report.error.unwrap_or_else(|| {
            ProblemError::internal(format!(
                "incomplete integration without a typed cause: {}",
                termination.as_str()
            ))
        })),
    }
}
