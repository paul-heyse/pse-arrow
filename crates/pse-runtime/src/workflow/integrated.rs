// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One integrated experiment, shared by the fit oracle and the shooting oracle (Plan 22 I8):
//! it binds a consumer's parameter values into the integration vector, integrates once, and
//! maps the output sensitivities, or an adjoint product, back to those parameters. The
//! consumers stay separate oracles; this is the integration they have in common.
use pse_backend_native::{
    self as native, ProblemError, dynamics::DynamicSensitivity, solve::Execution,
};
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
    /// The profile with the requested derivatives and at most the attempt's remaining time.
    fn attempt(&self, execution: &Execution, sensitivity: DynamicSensitivity) -> super::SimulationProfile {
        let mut profile = self.profile.clone();
        profile.sensitivity = sensitivity;
        profile.time_limit = profile.time_limit.min(
            execution
                .time_limit
                .saturating_sub(execution.started.elapsed()),
        );
        profile
    }
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
        let profile = self.attempt(execution, sensitivity);
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
        let profile = self.attempt(execution, DynamicSensitivity::Adjoint);
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
