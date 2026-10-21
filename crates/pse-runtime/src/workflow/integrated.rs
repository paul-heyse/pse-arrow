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
    /// Immutable build/runtime observation retained by simulation admission.
    #[cfg_attr(
        not(any(feature = "solver-diffsol", feature = "solver-idas")),
        expect(
            dead_code,
            reason = "the snapshot is consumed by linked dynamic execution"
        )
    )]
    pub(crate) snapshot: native::execution::Snapshot,
    /// The compiled dynamic functions.
    pub(crate) program: super::dynamics::DynamicProgram,
    /// Per-snapshot allocation admission, supplied by the actual Runtime owner.
    #[cfg_attr(
        not(feature = "solver-diffsol"),
        expect(
            dead_code,
            reason = "the allocator is consumed by linked dynamic execution"
        )
    )]
    pub(crate) snapshots: super::dynamics::SnapshotAllocator,
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
        report: &native::dynamics::Report,
        sample: &native::dynamics::Sample,
        row: usize,
        binding: &Binding,
    ) -> Result<f64, ProblemError> {
        let column = report.sensitivity_column(binding.local)?;
        let offset = row
            .checked_mul(report.forward_axes.len())
            .and_then(|start| start.checked_add(column))
            .ok_or_else(|| ProblemError::internal("integrated output sensitivity offset"))?;
        sample
            .output_sensitivities
            .get(offset)
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
        let times = std::iter::once(start).chain(
            profile
                .schedule
                .iter()
                .flat_map(|s| s.times.iter().copied()),
        );
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
            .map(|v| {
                v.copied()
                    .ok_or_else(|| ProblemError::internal("shooting window value"))
            })
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
    let mut profile = requested_profile(profile, sensitivity);
    profile.time_limit = profile.time_limit.min(
        execution
            .time_limit
            .saturating_sub(execution.started.elapsed()),
    );
    profile
}
#[cfg(feature = "solver-diffsol")]
fn requested_profile(
    profile: &super::SimulationProfile,
    sensitivity: DynamicSensitivity,
) -> super::SimulationProfile {
    let mut profile = profile.clone();
    profile.sensitivity = sensitivity;
    if sensitivity != DynamicSensitivity::Forward {
        profile.forward_axes = None;
    }
    if sensitivity != DynamicSensitivity::Adjoint {
        profile.adjoint_parameters = native::dynamics::AdjointParameters::All;
    }
    profile
}

#[cfg(feature = "solver-diffsol")]
impl IntegratedExperiment {
    /// Construct neutral compiled scratch once on the experiment's owning lane. Each
    /// parameter trial still starts a fresh native IVP; no native history crosses trials.
    pub(crate) fn worker(
        &self,
        execution: &Execution,
    ) -> Result<super::dynamics::DynamicWorker, ProblemError> {
        self.worker_scoped(execution.scope()?)
    }
    /// Bind retained mathematical/provider ownership to the original operation,
    /// independently of the fresh native IVPs it will serve.
    pub(crate) fn worker_scoped(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<super::dynamics::DynamicWorker, ProblemError> {
        let worker = self.program.worker(scope, self.snapshots.clone())?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("dynamics.workflow_worker_constructions", "count", 1);
        }
        Ok(worker)
    }
    /// Integrate once at the consumer's values, with forward sensitivities or none.
    /// Integrations borrow the admitted outer worker; they never enqueue nested native jobs.
    pub(crate) fn integrate(
        &self,
        worker: &mut super::dynamics::DynamicWorker,
        value: &dyn Fn(usize) -> f64,
        execution: &Execution,
        sensitivity: DynamicSensitivity,
        selected: &[usize],
    ) -> Result<native::dynamics::Report, ProblemError> {
        if sensitivity == DynamicSensitivity::Adjoint {
            return Err(ProblemError::internal(
                "an adjoint product is a gradient, not an integration",
            ));
        }
        #[cfg(feature = "solver-idas")]
        if self.profile.resolved_method()? == native::dynamics::Method::Idas {
            let mut profile = requested_profile(&self.profile, sensitivity);
            if sensitivity == DynamicSensitivity::Forward {
                profile.forward_axes = Some(selected.to_vec());
            }
            let mut workspace = worker.take_idas_workspace();
            let result = native::dynamics::integrate_with_workspace_observed(
                worker,
                &profile,
                &self.bind(value),
                execution,
                &self.snapshot,
                &mut workspace,
            );
            let success = result
                .as_ref()
                .is_ok_and(|report| report.termination == native::dynamics::Termination::Completed);
            worker.restore_idas_workspace(workspace, success);
            return completed(result?);
        }
        let mut profile = attempt(&self.profile, execution, sensitivity);
        if sensitivity == DynamicSensitivity::Forward {
            profile.forward_axes = Some(selected.to_vec());
        }
        let report = native::dynamics::integrate_with_progress_observed(
            worker,
            &profile,
            &self.bind(value),
            execution.cancellation_control().clone(),
            std::sync::Arc::new(native::solve::Progress::new(256)),
            &self.snapshot,
        )?;
        completed(report)
    }
    /// Compose the frozen production state allowances through the current output's
    /// exact partials. This is a physical comparison allowance, not a global error
    /// enclosure inferred from the native integration controls.
    pub(crate) fn prediction_allowances(
        &self,
        worker: &mut super::dynamics::DynamicWorker,
        value: &dyn Fn(usize) -> f64,
        report: &native::dynamics::Report,
    ) -> Result<Vec<Vec<f64>>, ProblemError> {
        use native::dynamics::Oracle;
        let parameters = self.bind(value);
        report
            .samples
            .iter()
            .map(|sample| {
                let budgets = self
                    .program
                    .state_allowances
                    .get(sample.mode)
                    .ok_or_else(|| ProblemError::internal("fit prediction allowance mode"))?;
                if budgets.len() != sample.state.len()
                    || budgets.iter().any(|v| !v.is_finite() || *v < 0.)
                {
                    return Err(ProblemError::Contract(
                        "fit prediction physical allowances".into(),
                    ));
                }
                let evaluation = worker.evaluate(
                    sample.mode,
                    native::dynamics::Function::Output,
                    sample.time,
                    &sample.state,
                    &self.profile.parameters_at(&parameters, sample.time),
                    true,
                )?;
                let partials = evaluation
                    .jacobian
                    .ok_or_else(|| ProblemError::internal("fit prediction output partials"))?;
                let mut allowances = vec![0.; sample.outputs.len()];
                if partials.nrows() != allowances.len() || partials.ncols() < budgets.len() {
                    return Err(ProblemError::internal("fit prediction partial extent"));
                }
                for (column, budget) in budgets.iter().enumerate() {
                    for k in partials.col_range(column) {
                        allowances[partials.row_idx()[k]] += partials.val()[k].abs() * budget;
                    }
                }
                let declared = self
                    .program
                    .output_allowances
                    .get(sample.mode)
                    .ok_or_else(|| ProblemError::internal("fit output allowance mode"))?;
                if declared.len() != allowances.len() {
                    return Err(ProblemError::internal("fit output allowance extent"));
                }
                for (allowance, cap) in allowances.iter_mut().zip(declared) {
                    if let Some(cap) = cap {
                        if !cap.is_finite() || *cap < 0. {
                            return Err(ProblemError::Contract(
                                "fit declared output allowance".into(),
                            ));
                        }
                        *allowance = allowance.min(*cap);
                    }
                }
                if allowances.iter().any(|v| !v.is_finite()) {
                    return Err(ProblemError::numerical(
                        "nonfinite fit prediction allowance",
                    ));
                }
                Ok(allowances)
            })
            .collect()
    }
    /// One forward and one backward pass: the forward report and the gradient of the
    /// cotangent's functional with respect to each selected binding's consumer parameter, as
    /// `(parameter, value)` pairs. The checkpoints are charged against the attempt's
    /// foreign allowance.
    pub(crate) fn gradient(
        &self,
        worker: &mut super::dynamics::DynamicWorker,
        value: &dyn Fn(usize) -> f64,
        execution: &Execution,
        selected: &[usize],
        cotangent: native::dynamics::Cotangent<'_>,
    ) -> Result<(native::dynamics::Report, Vec<(usize, f64)>), ProblemError> {
        #[cfg(feature = "solver-idas")]
        worker.discard_idas_workspace();
        let bindings = selected
            .iter()
            .map(|index| {
                self.bindings
                    .get(*index)
                    .ok_or_else(|| ProblemError::internal("adjoint binding extent"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut axes = Vec::new();
        for binding in &bindings {
            if !axes.contains(&binding.local) {
                axes.push(binding.local);
            }
        }
        let mut profile = attempt(&self.profile, execution, DynamicSensitivity::Adjoint);
        profile.adjoint_parameters = native::dynamics::AdjointParameters::Selected(axes);
        let native::dynamics::Gradient {
            report,
            gradient,
            parameter_axes,
            ..
        } = native::dynamics::gradient_current_observed(
            worker,
            &profile,
            &self.bind(value),
            cotangent,
            execution,
            &self.snapshot,
        )?;
        let report = completed(report)?;
        let gradient = gradient
            .ok_or_else(|| ProblemError::internal("completed adjoint without a gradient"))?;
        let contributions = bindings
            .iter()
            .map(|b| {
                parameter_axes
                    .iter()
                    .position(|axis| *axis == b.local)
                    .and_then(|column| gradient.get(column))
                    .map(|g| (b.parameter, g * b.conversion.scale))
                    .ok_or_else(|| ProblemError::internal("adjoint gradient extent"))
            })
            .collect::<Result<_, _>>()?;
        Ok((report, contributions))
    }
}

/// Completed window values, physical parameter axes and their ordered gradient.
#[cfg(feature = "solver-diffsol")]
type WindowGradient = (native::dynamics::Report, Vec<usize>, Vec<f64>);

/// The forward report, the gradient contributions and the row-major Hessian over the
/// selected bindings of one second-order adjoint.
#[cfg(feature = "solver-idas")]
type SecondOrder = (native::dynamics::Report, Vec<(usize, f64)>, Vec<f64>);
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
        worker: &mut super::dynamics::DynamicWorker,
        value: &dyn Fn(usize) -> f64,
        execution: &Execution,
        selected: &[usize],
        cotangent: native::dynamics::Cotangent<'_>,
    ) -> Result<SecondOrder, ProblemError> {
        worker.discard_idas_workspace();
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
        let mut profile = attempt(&self.profile, execution, DynamicSensitivity::Adjoint);
        // This route consumes the complete first-order binding gradient independently
        // of the selected second-order directions (ADR-0199).
        profile.adjoint_parameters = native::dynamics::AdjointParameters::All;
        let native::dynamics::Gradient {
            report,
            gradient,
            hessian,
            ..
        } = native::dynamics::hessian_current_observed(
            worker,
            &profile,
            &self.bind(value),
            &directions,
            cotangent,
            execution,
            &self.snapshot,
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
        worker: &mut super::dynamics::DynamicWorker,
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
        execution.check()?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("dynamics.window_ivp_initializations", "count", 1);
        }
        #[cfg(feature = "solver-idas")]
        if window.profile.resolved_method()? == native::dynamics::Method::Idas {
            let mut workspace = worker.take_idas_workspace();
            let result = (|| {
                let mut oracle = worker.window(false)?;
                let profile = oracle.profile(&requested_profile(&window.profile, sensitivity));
                native::dynamics::integrate_with_workspace_observed(
                    &mut oracle,
                    &profile,
                    &Self::window_parameters(window, integration, anchors)?,
                    execution,
                    &self.snapshot,
                    &mut workspace,
                )
            })();
            let success = result
                .as_ref()
                .is_ok_and(|report| report.termination == native::dynamics::Termination::Completed);
            worker.restore_idas_workspace(workspace, success);
            return completed(result?);
        }
        let mut oracle = worker.window(false)?;
        let profile = oracle.profile(&attempt(&window.profile, execution, sensitivity));
        let report = native::dynamics::integrate_with_progress_observed(
            &mut oracle,
            &profile,
            &Self::window_parameters(window, integration, anchors)?,
            execution.cancellation_control().clone(),
            std::sync::Arc::new(native::solve::Progress::new(256)),
            &self.snapshot,
        )?;
        completed(report)
    }
    /// A combined shooting request reuses its already-needed forward state directions
    /// for exact endpoint integral derivatives, without a checkpointed adjoint pass.
    #[cfg(feature = "solver-idas")]
    pub(crate) fn window_forward_quadratures(
        &self,
        worker: &mut super::dynamics::DynamicWorker,
        window: &Window,
        integration: &[f64],
        anchors: &[f64],
        execution: &Execution,
    ) -> Result<native::dynamics::ForwardQuadratures, ProblemError> {
        execution.check()?;
        execution.memory.ok_or_else(|| {
            ProblemError::Contract(
                "forward quadratures need the worker's admitted foreign allowance".into(),
            )
        })?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("dynamics.window_ivp_initializations", "count", 1);
            context.count(
                "dynamics.window_forward_quadrature_initializations",
                "count",
                1,
            );
        }
        let mut workspace = worker.take_idas_workspace();
        let result = (|| {
            let mut oracle = worker.window(false)?;
            let profile = oracle.profile(&requested_profile(
                &window.profile,
                DynamicSensitivity::Forward,
            ));
            native::dynamics::forward_quadratures_with_workspace_observed(
                &mut oracle,
                &profile,
                &Self::window_parameters(window, integration, anchors)?,
                execution,
                &self.snapshot,
                &mut workspace,
            )
        })();
        let success = result.as_ref().is_ok_and(|product| {
            product.report.termination == native::dynamics::Termination::Completed
        });
        worker.restore_idas_workspace(workspace, success);
        let mut product = result?;
        product.report = completed(product.report)?;
        if product.derivatives.as_ref().is_none_or(|values| {
            values.len()
                != product
                    .report
                    .forward_axes
                    .len()
                    .saturating_mul(self.program.contract.quadratures.len())
        }) {
            return Err(ProblemError::internal(
                "completed forward quadrature product extent",
            ));
        }
        Ok(product)
    }
    /// One forward and one backward pass over a shooting window whose quadratures are
    /// observed as outputs after the model's own: the forward report and the gradient of
    /// the cotangent's functional over the selected window integration columns, with
    /// their physical coordinate identity. The checkpoints
    /// are charged against the attempt's foreign allowance.
    #[expect(
        clippy::too_many_arguments,
        reason = "window values, original execution and ordered parameter-gradient demand have separate owners"
    )]
    pub(crate) fn window_gradient(
        &self,
        worker: &mut super::dynamics::DynamicWorker,
        window: &Window,
        integration: &[f64],
        anchors: &[f64],
        execution: &Execution,
        selected: &[usize],
        cotangent: native::dynamics::Cotangent<'_>,
    ) -> Result<WindowGradient, ProblemError> {
        #[cfg(feature = "solver-idas")]
        worker.discard_idas_workspace();
        execution.check()?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("dynamics.window_adjoint_initializations", "count", 1);
        }
        let mut oracle = worker.window(true)?;
        let mut profile = oracle.profile(&attempt(
            &window.profile,
            execution,
            DynamicSensitivity::Adjoint,
        ));
        profile.adjoint_parameters =
            native::dynamics::AdjointParameters::Selected(selected.to_vec());
        let native::dynamics::Gradient {
            report,
            gradient,
            parameter_axes,
            ..
        } = native::dynamics::gradient_current_observed(
            &mut oracle,
            &profile,
            &Self::window_parameters(window, integration, anchors)?,
            cotangent,
            execution,
            &self.snapshot,
        )?;
        let report = completed(report)?;
        let gradient = gradient
            .ok_or_else(|| ProblemError::internal("completed adjoint without a gradient"))?;
        Ok((report, parameter_axes, gradient))
    }
}

/// A stopped integration keeps its stop; it is never an evaluation failure.
#[cfg(feature = "solver-diffsol")]
fn completed(report: native::dynamics::Report) -> Result<native::dynamics::Report, ProblemError> {
    use native::dynamics::Termination;
    match report.termination {
        Termination::Completed => Ok(report),
        Termination::Event => Err(ProblemError::unsupported(
            "the trajectory prefix ended before required fixed-domain observations",
        )),
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
