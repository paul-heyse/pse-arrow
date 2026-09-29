// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dynamic roles project the shared typed compiler; only attempts own native state.
use crate::math::ExecutableCase;
use pse_backend_native::{
    ProblemError,
    dynamics::{self as native, Function, Oracle},
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_math::{assembly::CaseWorker, binding::CaseValues};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};
/// Dynamic controls use the concrete native profile; values are validated at preparation.
pub type SimulationProfile = native::Profile;
#[derive(Clone, Debug)]
pub(crate) struct FunctionProgram {
    pub(crate) mode: usize,
    pub(crate) function: Function,
    pub(crate) case: Arc<ExecutableCase>,
    pub(crate) rows: Vec<usize>,
    pub(crate) scales: Vec<f64>,
    pub(crate) offsets: Vec<f64>,
    pub(crate) constants: BTreeMap<usize, f64>,
}
/// Canonical endpoint/value obtained through the same compiled observation path.
#[derive(Clone, Debug)]
pub(crate) enum RangeValue {
    Input(SemanticId),
    Output(SemanticId),
    Constant(f64),
}
#[derive(Clone, Debug)]
pub(crate) struct RangeCheck {
    pub source: SemanticId,
    pub target: SemanticId,
    pub value: RangeValue,
    pub lower: Option<RangeValue>,
    pub upper: Option<RangeValue>,
}
#[derive(Clone, Debug)]
pub(crate) struct GuardProgram {
    pub case: Arc<ExecutableCase>,
    pub rows: BTreeMap<SemanticId, usize>,
    pub checks: Vec<RangeCheck>,
}
#[derive(Debug)]
struct GuardWorker {
    program: GuardProgram,
    worker: CaseWorker,
}
impl GuardWorker {
    fn validate(&mut self, values: &CaseValues) -> Result<(), ProblemError> {
        let rows = self.worker.constraints(values)?;
        let read = |v: &RangeValue| -> Result<f64, ProblemError> {
            match v {
                RangeValue::Input(id) => values
                    .scalars
                    .get(id)
                    .copied()
                    .ok_or_else(|| ProblemError::Internal("dynamic range input absent".into())),
                RangeValue::Output(id) => self
                    .program
                    .rows
                    .get(id)
                    .and_then(|i| rows.get(*i))
                    .copied()
                    .ok_or_else(|| ProblemError::Internal("dynamic range output absent".into())),
                RangeValue::Constant(v) => Ok(*v),
            }
        };
        for check in &self.program.checks {
            let value = read(&check.value)?;
            let lower = check.lower.as_ref().map(&read).transpose()?;
            let upper = check.upper.as_ref().map(&read).transpose()?;
            if !value.is_finite()
                || lower.into_iter().chain(upper).any(|v| !v.is_finite())
                || lower.zip(upper).is_some_and(|(a, b)| a > b)
            {
                return Err(ProblemError::Contract(
                    "invalid dynamic physical range".into(),
                ));
            }
            if lower.is_some_and(|v| value < v) || upper.is_some_and(|v| value > v) {
                return Err(pse_math::MathError::OutsideRange {
                    source_id: check.source,
                    target: check.target,
                    value,
                    lower,
                    upper,
                }
                .into());
            }
        }
        Ok(())
    }
}
fn provider_workers(
    providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    cancel: &Arc<AtomicBool>,
) -> Result<BTreeMap<pse_kernels::ProviderKey, Box<dyn pse_kernels::Provider>>, ProblemError> {
    providers
        .values()
        .map(|r| {
            r.worker_scoped(cancel.clone())
                .map(|w| (r.spec().key(), w))
                .map_err(ProblemError::Provider)
        })
        .collect()
}
/// Immutable function execution inputs, independent of their source representation.
#[derive(Clone, Debug)]
pub(crate) struct DynamicProgram {
    pub contract: native::Contract,
    pub programs: Arc<[FunctionProgram]>,
    pub coordinates: DynamicCoordinates,
    pub modes: Vec<DynamicMode>,
    pub max_cells: usize,
}
impl DynamicProgram {
    /// Conservative owned metadata allowance; compiled functions keep their own leases.
    pub(crate) fn metadata_bytes(&self) -> usize {
        let c = &self.contract;
        let cells = c
            .states
            .len()
            .saturating_add(c.parameters.len())
            .saturating_add(c.outputs.len())
            .saturating_add(c.quadratures.len())
            .saturating_add(c.events.iter().map(Vec::len).sum::<usize>())
            .saturating_add(
                c.balances
                    .iter()
                    .map(|b| 1 + b.impulses.len())
                    .sum::<usize>(),
            )
            .saturating_add(self.coordinates.state.len())
            .saturating_add(self.coordinates.parameters.len())
            .saturating_add(
                self.modes
                    .iter()
                    .map(|m| {
                        m.values.scalars.len()
                            + m.providers.len()
                            + m.guard
                                .as_ref()
                                .map_or(0, |g| g.checks.len() + g.rows.len())
                    })
                    .sum::<usize>(),
            );
        cells.saturating_mul(512).saturating_add(size_of::<Self>())
    }
    pub(crate) fn worker(&self, cancel: Arc<AtomicBool>) -> Result<DynamicWorker, ProblemError> {
        DynamicWorker::new(
            self.contract.clone(),
            &self.programs,
            self.coordinates.clone(),
            &self.modes,
            self.max_cells,
            cancel,
        )
    }
}
/// One immutable mode's physical values, nested providers and range obligations.
#[derive(Clone, Debug)]
pub(crate) struct DynamicMode {
    pub values: CaseValues,
    pub providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    pub guard: Option<GuardProgram>,
}
/// One native-to-physical coordinate conversion; derived from checked ports.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CoordinateBinding {
    pub id: SemanticId,
    pub scale: f64,
    pub offset: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DynamicCoordinates {
    pub time: SemanticId,
    pub time_scale: f64,
    pub time_origin: f64,
    pub state: Vec<CoordinateBinding>,
    pub parameters: Vec<CoordinateBinding>,
}
impl DynamicWorker {
    pub(crate) fn new(
        contract: native::Contract,
        programs: &[FunctionProgram],
        coordinates: DynamicCoordinates,
        modes: &[DynamicMode],
        max_cells: usize,
        cancel: Arc<AtomicBool>,
    ) -> Result<Self, ProblemError> {
        if modes.len() != contract.events.len() {
            return Err(ProblemError::Internal("dynamic mode extent".into()));
        }
        let chain = coordinates
            .state
            .iter()
            .chain(&coordinates.parameters)
            .map(|c| c.scale)
            .collect::<Vec<_>>();
        let mut functions = BTreeMap::new();
        for program in programs.iter() {
            let mode = modes
                .get(program.mode)
                .ok_or_else(|| ProblemError::Internal("dynamic function mode absent".into()))?;
            let providers = provider_workers(&mode.providers, &cancel)?;
            let source = program.case.assembly.jacobian_pattern();
            // The dynamic oracle's support: `(row, coordinate)` entries over the
            // function's rows and the state followed by the parameters.
            let mut pairs = Vec::new();
            let mut refill = Vec::new();
            for (c, scale) in chain.iter().enumerate() {
                for k in source.col_range(c) {
                    for (row, &original) in program.rows.iter().enumerate() {
                        if !program.constants.contains_key(&row) && source.row_idx()[k] == original
                        {
                            refill.push((
                                k,
                                pse_math::index::Addend::new(pairs.len()),
                                program.scales[row] * scale,
                            ));
                            pairs.push(native::SupportEntry::new(
                                pse_math::index::OriginalRow::new(row),
                                pse_math::index::OriginalCol::new(c),
                            ));
                        }
                    }
                }
            }
            let jacobian = pse_math::sparse::AssemblyMatrix::new(
                program.rows.len(),
                chain.len(),
                &pairs,
                max_cells,
            )?;
            functions.insert(
                (program.mode, program.function),
                FunctionWorker {
                    program: program.clone(),
                    worker: program.case.assembly.worker(providers, cancel.clone()),
                    jacobian,
                    refill,
                    pairs,
                    cache: None,
                    #[cfg(test)]
                    evaluations: 0,
                },
            );
        }
        let modes = modes
            .iter()
            .map(|mode| {
                let guard = mode
                    .guard
                    .as_ref()
                    .map(|g| {
                        Ok::<_, ProblemError>(GuardWorker {
                            program: g.clone(),
                            worker: g.case.assembly.worker(
                                provider_workers(&mode.providers, &cancel)?,
                                cancel.clone(),
                            ),
                        })
                    })
                    .transpose()?;
                Ok::<_, ProblemError>(ModeWorker {
                    values: mode.values.clone(),
                    guard,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(DynamicWorker {
            modes,
            contract,
            coordinates,
            functions,
            cancel,
        })
    }
}
#[derive(Debug)]
struct FunctionWorker {
    program: FunctionProgram,
    worker: CaseWorker,
    jacobian: pse_math::sparse::AssemblyMatrix,
    refill: Vec<(usize, pse_math::index::Addend, f64)>,
    pairs: Vec<native::SupportEntry>,
    // Mode/function and provider/build identity are fixed by this worker. Every
    // varying time/state/parameter bit participates, including signed zero.
    cache: Option<(Vec<u64>, native::Evaluation)>,
    #[cfg(test)]
    evaluations: usize,
}
#[derive(Debug)]
struct ModeWorker {
    values: CaseValues,
    guard: Option<GuardWorker>,
}
#[derive(Debug)]
pub(crate) struct DynamicWorker {
    modes: Vec<ModeWorker>,
    contract: native::Contract,
    coordinates: DynamicCoordinates,
    functions: BTreeMap<(usize, Function), FunctionWorker>,
    cancel: Arc<AtomicBool>,
}
impl Oracle for DynamicWorker {
    fn contract(&self) -> &native::Contract {
        &self.contract
    }
    fn support(&self, mode: usize, function: Function) -> Vec<native::SupportEntry> {
        self.functions
            .get(&(mode, function))
            .map_or_else(Vec::new, |w| w.pairs.clone())
    }
    fn evaluate(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        derivatives: bool,
    ) -> Result<native::Evaluation, ProblemError> {
        if self.cancel.load(std::sync::atomic::Ordering::Acquire) {
            return Err(pse_math::MathError::Cancelled.into());
        }
        let d = &self.coordinates;
        let n = d.state.len();
        if state.len() != n
            || parameters.len() != d.parameters.len()
            || !time.is_finite()
            || state.iter().chain(parameters).any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "dynamic binding dimensions or values".into(),
            ));
        }
        let role = function;
        let Some(function) = self.functions.get_mut(&(mode, function)) else {
            if function == Function::Roots && mode < self.contract.events.len() {
                return Ok(native::Evaluation {
                    values: vec![],
                    jacobian: None,
                });
            }
            return Err(ProblemError::Internal(
                "missing compiled dynamic function".into(),
            ));
        };
        let bits = std::iter::once(time)
            .chain(state.iter().copied())
            .chain(parameters.iter().copied())
            .map(f64::to_bits);
        if let Some((key, evaluation)) = &function.cache
            && (!derivatives || evaluation.jacobian.is_some())
            && key.iter().copied().eq(bits.clone())
        {
            let mut result = evaluation.clone();
            if !derivatives {
                result.jacobian = None;
            }
            return Ok(result);
        }
        function.cache = None;
        #[cfg(test)]
        {
            function.evaluations += 1;
        }
        let context = self
            .modes
            .get_mut(mode)
            .ok_or_else(|| ProblemError::Internal("dynamic mode absent".into()))?;
        context
            .values
            .scalars
            .insert(d.time, (time - d.time_origin) / d.time_scale);
        for (c, x) in d
            .state
            .iter()
            .zip(state)
            .chain(d.parameters.iter().zip(parameters))
        {
            context.values.scalars.insert(c.id, x * c.scale + c.offset);
        }
        if role != Function::Initial
            && let Some(guard) = &mut context.guard
        {
            guard.validate(&context.values)?;
        }
        let p = &function.program;
        let rows = function.worker.constraints(&context.values)?;
        let values = p
            .rows
            .iter()
            .enumerate()
            .map(|(i, &r)| {
                p.constants
                    .get(&i)
                    .copied()
                    .unwrap_or_else(|| rows[r] * p.scales[i] + p.offsets[i])
            })
            .collect::<Vec<_>>();
        let jacobian = if derivatives {
            let source = function.worker.jacobian(&context.values)?;
            function.jacobian.clear();
            for &(local, target, scale) in &function.refill {
                function.jacobian.add(target, source.val()[local] * scale)?;
            }
            Some(function.jacobian.matrix().clone())
        } else {
            None
        };
        if values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite dynamic values"));
        }
        let result = native::Evaluation { values, jacobian };
        function.cache = Some((bits.collect(), result.clone()));
        Ok(result)
    }
}

pub(crate) fn profile_identity(p: &SimulationProfile) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DynamicProfileV4);
    h.str(&native::profile_json(p).to_string())
        .hash(&p.numerics.key());
    h.finish_hash()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simulation_identity_includes_backend_and_trial_policy() {
        let base = SimulationProfile::default();
        let mut selected = base.clone();
        selected.method = native::Method::Idas;
        assert_ne!(profile_identity(&base), profile_identity(&selected));
        let idas = profile_identity(&selected);
        selected.trial_failures = native::TrialPolicy::Recoverable;
        assert_ne!(idas, profile_identity(&selected));
        selected = base.clone();
        selected.samples.reverse();
        if base.samples.len() > 1 {
            assert_ne!(profile_identity(&base), profile_identity(&selected));
        }
        selected = base.clone();
        selected.time_limit += std::time::Duration::from_nanos(1);
        assert_ne!(profile_identity(&base), profile_identity(&selected));
    }
}
