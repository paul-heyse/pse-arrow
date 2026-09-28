// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Trial-local numerical configuration. Hint programs execute on the admitted worker.
use super::*;

/// Interpret already evaluated physical hints, without scheduling or model evaluation.
pub trait HintResolver: std::fmt::Debug + Send + Sync {
    fn identity(&self) -> ContentHash;
    fn retained_bytes(&self) -> usize;
    fn time_limit(&self) -> Duration;
    fn resolve(
        &self,
        values: &[f64],
        original_terms: Option<&[f64]>,
    ) -> Result<(Vec<Unknown>, Options), MathError>;
}

/// Fixed caller configuration or typed model hints evaluated for each enclosing trial.
#[derive(Clone, Debug)]
pub enum Configuration {
    Fixed(Vec<Unknown>, Options),
    Hints(Arc<dyn HintResolver>),
}
impl Configuration {
    pub fn time_limit(&self) -> Duration {
        match self {
            Self::Fixed(_, options) => options.time_limit,
            Self::Hints(r) => r.time_limit(),
        }
    }
    pub fn retained_bytes(&self) -> usize {
        match self {
            Self::Hints(r) => r.retained_bytes(),
            Self::Fixed(u, o) => {
                size_of_val(u.as_slice())
                    + [
                        &o.start,
                        &o.variable_nominals,
                        &o.variable_tolerance,
                        &o.residual_tolerance,
                    ]
                    .iter()
                    .map(|v| size_of_val(v.as_slice()))
                    .sum::<usize>()
            }
        }
    }
    pub fn identity(&self) -> ContentHash {
        match self {
            Self::Hints(r) => r.identity(),
            Self::Fixed(unknowns, options) => {
                let mut h =
                    pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitFixedConfigurationV1);
                for u in unknowns {
                    h.id(&u.id).u64(u.lower.to_bits()).u64(u.upper.to_bits());
                }
                for values in [
                    &options.start,
                    &options.variable_nominals,
                    &options.variable_tolerance,
                    &options.residual_tolerance,
                ] {
                    h.u64(values.len() as u64);
                    for value in values {
                        h.u64(value.to_bits());
                    }
                }
                h.u64(options.iterations as u64)
                    .u64(options.time_limit.as_secs())
                    .u64(options.time_limit.subsec_nanos() as u64)
                    .u64(options.derivative_tolerance.to_bits());
                h.finish_hash()
            }
        }
    }
}
#[derive(Debug)]
pub(super) struct ConfigurationWorker {
    source: Configuration,
    hints: Option<Worker>,
    terms: Option<Worker>,
}
impl ConfigurationWorker {
    pub(super) fn new(
        source: Configuration,
        hints: Option<&Arc<CompiledBody>>,
        terms: Option<&Arc<CompiledBody>>,
    ) -> Self {
        Self {
            source,
            hints: hints.map(|b| b.worker()),
            terms: terms.map(|b| b.worker()),
        }
    }
    pub(super) fn resolve(
        &mut self,
        problem: &mut Arc<Problem>,
        inputs: &[f64],
        cancel: &Arc<AtomicBool>,
    ) -> Result<Options, MathError> {
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        if inputs.len() != problem.inputs {
            return Err(MathError::Contract("implicit hint input extent".into()));
        }
        let (unknowns, options) = match &self.source {
            Configuration::Fixed(unknowns, options) => (unknowns.clone(), options.clone()),
            Configuration::Hints(resolver) => {
                let values = if let Some(worker) = &mut self.hints {
                    // Admission proves these unknown coordinates are absent from hint dependencies.
                    let mut point = vec![0.; problem.unknowns.len()];
                    point.extend_from_slice(inputs);
                    worker
                        .evaluate(
                            &point,
                            DerivativeOrder::Value,
                            &mut *problem.providers.lock().map_err(|_| {
                                MathError::Library("implicit hint provider lock poisoned".into())
                            })?,
                            cancel,
                        )?
                        .values
                } else {
                    Vec::new()
                };
                let initial = resolver.resolve(&values, None)?;
                if let Some(worker) = &mut self.terms {
                    let mut nominal = initial.1.variable_nominals.clone();
                    nominal.extend_from_slice(inputs);
                    let terms = worker
                        .evaluate(
                            &nominal,
                            DerivativeOrder::Value,
                            &mut *problem.providers.lock().map_err(|_| {
                                MathError::Library("implicit nominal provider lock poisoned".into())
                            })?,
                            cancel,
                        )?
                        .values;
                    resolver.resolve(&values, Some(&terms))?
                } else {
                    initial
                }
            }
        };
        if unknowns.len() != problem.unknowns.len()
            || unknowns.iter().zip(&problem.unknowns).any(|(a, b)| {
                a.id != b.id
                    || a.lower.is_nan()
                    || a.upper.is_nan()
                    || a.lower > a.upper
                    || a.lower == f64::INFINITY
                    || a.upper == f64::NEG_INFINITY
            })
        {
            return Err(MathError::Contract(
                "implicit trial configuration identities or intervals".into(),
            ));
        }
        let problem = Arc::get_mut(problem).ok_or_else(|| {
            MathError::Contract("inner solver retained a completed trial problem".into())
        })?;
        problem.unknowns = unknowns;
        problem.validate_options(&options)?;
        Ok(options)
    }
}
