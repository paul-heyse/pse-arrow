// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dynamic roles project the shared typed compiler; only attempts own native state.
use crate::math::ExecutableCase;
use pse_backend_native::{
    ProblemError,
    dynamics::{self as native, Function, Oracle},
};
use pse_ids::{FramedHasher, SemanticId};
use pse_math::{assembly::CaseWorker, binding::CaseValues};
use std::{collections::BTreeMap, sync::Arc};
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
/// One range obligation observed at a point: the checked value and its endpoints.
struct Observed {
    value: f64,
    lower: Option<f64>,
    upper: Option<f64>,
}
impl GuardWorker {
    /// Observe `check` at `values`, whose guard rows are `rows`; an endpoint or value that is
    /// not finite, or a reversed range, is refused.
    fn observe(
        &self,
        check: &RangeCheck,
        rows: &[f64],
        values: &CaseValues,
    ) -> Result<Observed, ProblemError> {
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
        Ok(Observed {
            value,
            lower,
            upper,
        })
    }
    fn validate(&mut self, values: &CaseValues) -> Result<(), ProblemError> {
        let rows = self.worker.constraints(values)?;
        for check in &self.program.checks {
            let Observed {
                value,
                lower,
                upper,
            } = self.observe(check, &rows, values)?;
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
    /// The physical range each checked target is admitted in at `values`: the
    /// intersection of every obligation on it.
    fn ranges(
        &mut self,
        values: &CaseValues,
    ) -> Result<BTreeMap<SemanticId, (f64, f64)>, ProblemError> {
        let rows = self.worker.constraints(values)?;
        let mut ranges = BTreeMap::new();
        for check in &self.program.checks {
            let observed = self.observe(check, &rows, values)?;
            let (lower, upper): &mut (f64, f64) = ranges
                .entry(check.target)
                .or_insert((f64::NEG_INFINITY, f64::INFINITY));
            *lower = lower.max(observed.lower.unwrap_or(f64::NEG_INFINITY));
            *upper = upper.min(observed.upper.unwrap_or(f64::INFINITY));
        }
        Ok(ranges)
    }
}
fn provider_workers(
    providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    scope: &pse_kernels::ExecutionScope,
) -> Result<BTreeMap<pse_kernels::ProviderKey, Box<dyn pse_kernels::Provider>>, ProblemError> {
    crate::math::attempt_providers(providers, scope).map_err(ProblemError::Provider)
}
/// Immutable function execution inputs, independent of their source representation.
#[derive(Clone, Debug)]
pub(crate) struct DynamicProgram {
    pub contract: native::Contract,
    pub programs: Arc<[FunctionProgram]>,
    pub coordinates: DynamicCoordinates,
    pub modes: Vec<DynamicMode>,
    pub layout: Arc<DynamicLayout>,
    /// Frozen production state allowances in native coordinates, by mode.
    pub state_allowances: Arc<[Vec<f64>]>,
    /// Existing declared output decision budgets, when the physical policy supplies one.
    pub output_allowances: Arc<[Vec<Option<f64>>]>,
}
/// Retained immutable allowance arrays, including the two Arc allocation headers.
pub(crate) fn allowance_bytes(state: &[Vec<f64>], output: &[Vec<Option<f64>>]) -> Option<usize> {
    state
        .iter()
        .try_fold(size_of::<usize>() * 4, |bytes, values| {
            bytes
                .checked_add(size_of::<Vec<f64>>())?
                .checked_add(values.capacity().checked_mul(size_of::<f64>())?)
        })?
        .checked_add(output.iter().try_fold(0usize, |bytes, values| {
            bytes
                .checked_add(size_of::<Vec<Option<f64>>>())?
                .checked_add(values.capacity().checked_mul(size_of::<Option<f64>>())?)
        })?)
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
                    .map(|b| 3 + b.transfers.len())
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
        cells
            .saturating_mul(512)
            .saturating_add(size_of::<Self>())
            .saturating_add(self.layout.retained_bytes())
            .saturating_add(
                allowance_bytes(&self.state_allowances, &self.output_allowances)
                    .unwrap_or(usize::MAX),
            )
    }
    /// Source-issued fresh whole-program worker storage, before even cloning
    /// the owning DynamicProgram. Provider factories without a population
    /// contract retain a conservative caller-owned entry.
    pub(crate) fn worker_allocation_bound<'a>(
        contract: &native::Contract,
        programs: &[FunctionProgram],
        coordinates: &DynamicCoordinates,
        modes: impl Iterator<Item = &'a DynamicMode> + Clone,
        layout: &DynamicLayout,
        allowances: usize,
    ) -> Result<Option<usize>, ProblemError> {
        if modes.clone().any(|mode| !mode.providers.is_empty()) {
            return Ok(None);
        }
        let extent = || -> Option<usize> {
            let mut cells = contract
                .states
                .len()
                .checked_add(contract.parameters.len())?
                .checked_add(contract.outputs.len())?
                .checked_add(contract.quadratures.len())?
                .checked_add(coordinates.state.len())?
                .checked_add(coordinates.parameters.len())?;
            cells = contract
                .events
                .iter()
                .try_fold(cells, |n, e| n.checked_add(e.len()))?;
            cells = contract.balances.iter().try_fold(cells, |n, b| {
                n.checked_add(3)?.checked_add(b.transfers.len())
            })?;
            cells = modes.clone().try_fold(cells, |n, m| {
                n.checked_add(m.values.scalars.len())?.checked_add(
                    m.guard
                        .as_ref()
                        .map_or(0, |g| g.checks.len() + g.rows.len()),
                )
            })?;
            let metadata = cells
                .checked_mul(512)?
                .checked_add(size_of::<Self>())?
                .checked_add(programs.len().checked_mul(size_of::<FunctionProgram>())?)?
                .checked_add(allowances)?;
            let buffers = programs.iter().try_fold(layout.worker_bytes(), |n, p| {
                n.checked_add(p.case.assembly.numeric_worker_bytes())
            })?;
            let buffers = modes.clone().try_fold(buffers, |n, m| {
                n.checked_add(
                    m.guard
                        .as_ref()
                        .map_or(0, |g| g.case.assembly.numeric_worker_bytes()),
                )
            })?;
            // program() and worker() each clone mode/contract/coordinate metadata;
            // compiled program owners and immutable layout stay shared.
            buffers.checked_add(metadata.checked_mul(2)?)
        };
        extent()
            .map(Some)
            .ok_or_else(|| ProblemError::memory("dynamic worker construction extent"))
    }
    pub(crate) fn worker(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<DynamicWorker, ProblemError> {
        DynamicWorker::new(
            self.contract.clone(),
            &self.programs,
            self.coordinates.clone(),
            &self.modes,
            self.layout.clone(),
            scope,
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
/// Program-fixed coordinate chains, support and canonical sparse templates.
#[derive(Debug)]
pub(crate) struct DynamicLayout {
    chain: Vec<f64>,
    functions: BTreeMap<(usize, Function), Arc<FunctionLayout>>,
}
#[derive(Debug)]
struct FunctionLayout {
    jacobian: pse_math::sparse::AssemblyMatrix,
    refill: Vec<(usize, pse_math::index::Addend, f64)>,
    pairs: Vec<native::SupportEntry>,
}
impl DynamicLayout {
    pub(crate) fn new(
        programs: &[FunctionProgram],
        coordinates: &DynamicCoordinates,
        max_cells: usize,
    ) -> Result<Self, ProblemError> {
        let chain = coordinates
            .state
            .iter()
            .chain(&coordinates.parameters)
            .map(|c| c.scale)
            .collect::<Vec<_>>();
        if chain.len() > max_cells
            || max_cells > isize::MAX as usize
            || chain.iter().any(|v| !v.is_finite() || *v <= 0.)
        {
            return Err(ProblemError::Contract(
                "dynamic coordinate extent or scale".into(),
            ));
        }
        let mut functions = BTreeMap::new();
        for program in programs.iter() {
            let source = program.case.assembly.jacobian_pattern();
            if source.ncols() != chain.len()
                || program.rows.len() != program.scales.len()
                || program.rows.len() != program.offsets.len()
                || program.rows.iter().any(|r| *r >= source.nrows())
                || program.constants.keys().any(|r| *r >= program.rows.len())
                || program
                    .scales
                    .iter()
                    .chain(&program.offsets)
                    .chain(program.constants.values())
                    .any(|v| !v.is_finite())
            {
                return Err(ProblemError::Contract(
                    "dynamic function layout extent or values".into(),
                ));
            }
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
            if functions
                .insert(
                    (program.mode, program.function),
                    Arc::new(FunctionLayout {
                        jacobian,
                        refill,
                        pairs,
                    }),
                )
                .is_some()
            {
                return Err(ProblemError::internal("duplicate dynamic function"));
            }
        }
        Ok(Self { chain, functions })
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.chain.capacity() * size_of::<f64>()
            + self
                .functions
                .values()
                .map(|f| {
                    size_of::<FunctionLayout>()
                        + f.jacobian.retained_bytes()
                        + f.refill.capacity() * size_of::<(usize, pse_math::index::Addend, f64)>()
                        + f.pairs.capacity() * size_of::<native::SupportEntry>()
                        + 128
                })
                .sum::<usize>()
    }
    pub(crate) fn worker_bytes(&self) -> usize {
        self.functions
            .values()
            .map(|f| f.jacobian.retained_bytes())
            .sum()
    }
}
impl DynamicWorker {
    fn new(
        contract: native::Contract,
        programs: &[FunctionProgram],
        coordinates: DynamicCoordinates,
        modes: &[DynamicMode],
        layout: Arc<DynamicLayout>,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<Self, ProblemError> {
        scope.check().map_err(ProblemError::Provider)?;
        if modes.len() != contract.events.len() {
            return Err(ProblemError::Internal("dynamic mode extent".into()));
        }
        let mut functions = BTreeMap::new();
        for program in programs {
            let mode = modes
                .get(program.mode)
                .ok_or_else(|| ProblemError::internal("dynamic function mode absent"))?;
            let function_layout = layout
                .functions
                .get(&(program.mode, program.function))
                .ok_or_else(|| ProblemError::internal("dynamic function layout absent"))?
                .clone();
            functions.insert(
                (program.mode, program.function),
                FunctionWorker {
                    program: program.clone(),
                    worker: program
                        .case
                        .assembly
                        .worker_scoped(provider_workers(&mode.providers, &scope)?, scope.clone()),
                    jacobian: function_layout.jacobian.clone(),
                    layout: function_layout,
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
                            worker: g.case.assembly.worker_scoped(
                                provider_workers(&mode.providers, &scope)?,
                                scope.clone(),
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
            layout,
            scope,
        })
    }
}
#[derive(Debug)]
struct FunctionWorker {
    program: FunctionProgram,
    worker: CaseWorker,
    jacobian: pse_math::sparse::AssemblyMatrix,
    layout: Arc<FunctionLayout>,
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
    layout: Arc<DynamicLayout>,
    scope: pse_kernels::ExecutionScope,
}
impl DynamicWorker {
    #[cfg(test)]
    pub(crate) fn assert_independent_layout(&self, other: &Self) {
        assert!(Arc::ptr_eq(&self.layout, &other.layout));
        for (key, f) in &self.functions {
            let g = &other.functions[key];
            assert!(Arc::ptr_eq(&f.layout, &g.layout));
            if !f.jacobian.matrix().val().is_empty() {
                assert_ne!(
                    f.jacobian.matrix().val().as_ptr(),
                    g.jacobian.matrix().val().as_ptr()
                );
            }
        }
    }
    /// Bind one trial point into the mode's physical values and validate the mode's range
    /// obligations for every role but the initial values.
    fn bind(
        &mut self,
        mode: usize,
        role: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
    ) -> Result<(), ProblemError> {
        self.assign(mode, time, state, parameters)?;
        let context = &mut self.modes[mode];
        if role != Function::Initial
            && let Some(guard) = &mut context.guard
        {
            guard.validate(&context.values)?;
        }
        Ok(())
    }
    /// The box the mode's range obligations admit each coordinate in at one point: states
    /// then parameters, in native coordinates, unbounded where no obligation applies. An
    /// endpoint is rounded inward, so every coordinate of the box maps into its range.
    pub(crate) fn coordinate_box(
        &mut self,
        mode: usize,
        time: f64,
        state: &[f64],
        parameters: &[f64],
    ) -> Result<Vec<(f64, f64)>, ProblemError> {
        self.assign(mode, time, state, parameters)?;
        let context = &mut self.modes[mode];
        let ranges = match &mut context.guard {
            Some(guard) => guard.ranges(&context.values)?,
            None => BTreeMap::new(),
        };
        self.coordinates
            .state
            .iter()
            .chain(&self.coordinates.parameters)
            .map(|c| {
                let (lower, upper) = ranges
                    .get(&c.id)
                    .copied()
                    .unwrap_or((f64::NEG_INFINITY, f64::INFINITY));
                if !c.scale.is_finite() || c.scale <= 0. || !c.offset.is_finite() {
                    return Err(ProblemError::Internal("dynamic coordinate scale".into()));
                }
                let native = |v: f64| (v - c.offset) / c.scale;
                let physical = |x: f64| x * c.scale + c.offset;
                let mut low = native(lower);
                if low.is_finite() && physical(low) < lower {
                    low = low.next_up();
                }
                let mut high = native(upper);
                if high.is_finite() && physical(high) > upper {
                    high = high.next_down();
                }
                Ok((low, high))
            })
            .collect()
    }
    /// Set one trial point's time, state and parameter coordinates in the mode's physical
    /// values, after checking its dimensions and values.
    fn assign(
        &mut self,
        mode: usize,
        time: f64,
        state: &[f64],
        parameters: &[f64],
    ) -> Result<(), ProblemError> {
        self.scope.check().map_err(ProblemError::Provider)?;
        let d = &self.coordinates;
        if state.len() != d.state.len()
            || parameters.len() != d.parameters.len()
            || !time.is_finite()
            || state.iter().chain(parameters).any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "dynamic binding dimensions or values".into(),
            ));
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
        Ok(())
    }
}
impl Oracle for DynamicWorker {
    fn contract(&self) -> &native::Contract {
        &self.contract
    }
    fn support(&self, mode: usize, function: Function) -> Vec<native::SupportEntry> {
        self.functions
            .get(&(mode, function))
            .map_or_else(Vec::new, |w| w.layout.pairs.clone())
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
        self.scope.check().map_err(ProblemError::Provider)?;
        let role = function;
        let Some(function) = self.functions.get(&(mode, function)) else {
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
        self.bind(mode, role, time, state, parameters)?;
        let function = self
            .functions
            .get_mut(&(mode, role))
            .ok_or_else(|| ProblemError::Internal("missing compiled dynamic function".into()))?;
        function.cache = None;
        #[cfg(test)]
        {
            function.evaluations += 1;
        }
        let context = &self.modes[mode];
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
            for &(local, target, scale) in &function.layout.refill {
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
        self.scope.check().map_err(ProblemError::Provider)?;
        function.cache = Some((bits.collect(), result.clone()));
        Ok(result)
    }
    /// The compiled case's exact Lagrangian Hessian with the function rows' weights as
    /// multipliers (ADR-0110 item 4): a row's weight carries its value scale, constant
    /// rows carry no curvature, and each entry takes both coordinates' scales, as the
    /// first-order partials do.
    fn weighted_hessian(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
    ) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
        if self.contract.derivatives < pse_kernels::DerivativeOrder::Second {
            return Err(ProblemError::unsupported(
                "the dynamic functions were prepared without second derivatives",
            ));
        }
        self.bind(mode, function, time, state, parameters)?;
        let chain = &self.layout.chain;
        let worker = self
            .functions
            .get_mut(&(mode, function))
            .ok_or_else(|| ProblemError::Internal("missing compiled dynamic function".into()))?;
        let p = &worker.program;
        if weights.len() != p.rows.len() || weights.iter().any(|w| !w.is_finite()) {
            return Err(ProblemError::Contract(
                "dynamic Hessian weight extent or value".into(),
            ));
        }
        let mut multipliers = vec![0.0; worker.worker.assembly().structure().rows().len()];
        for (i, &row) in p.rows.iter().enumerate() {
            if !p.constants.contains_key(&i) {
                multipliers[row] += weights[i] * p.scales[i];
            }
        }
        let h = worker
            .worker
            .hessian(&self.modes[mode].values, 0.0, &multipliers)?;
        if h.ncols() != chain.len() || h.nrows() != chain.len() {
            return Err(ProblemError::Internal(
                "dynamic Hessian coordinate extent".into(),
            ));
        }
        let mut triplets = Vec::with_capacity(h.val().len());
        for col in 0..h.ncols() {
            for k in h.col_range(col) {
                let row = h.row_idx()[k];
                triplets.push(faer::sparse::Triplet::new(
                    row.max(col),
                    row.min(col),
                    h.val()[k] * chain[row] * chain[col],
                ));
            }
        }
        faer::sparse::SparseColMat::try_new_from_triplets(chain.len(), chain.len(), &triplets)
            .map_err(|e| ProblemError::memory(format!("dynamic Hessian assembly: {e:?}")))
    }
}

pub(crate) fn profile_identity(
    p: &SimulationProfile,
) -> Result<pse_ids::roles::ProfileHash, ProblemError> {
    let mut h = FramedHasher::new(pse_ids::Frame::DynamicProfileV9);
    pse_ids::document::frame(&mut h, p)
        .map_err(|error| ProblemError::Internal(error.to_string()))?;
    h.str("resolved_method");
    pse_ids::document::frame(&mut h, &p.resolved_method().ok())
        .map_err(|error| ProblemError::Internal(error.to_string()))?;
    h.hash(&p.numerics.key());
    Ok(pse_ids::roles::ProfileHash::from_id(h.finish_hash()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simulation_identity_includes_backend_and_trial_policy() {
        let base = SimulationProfile::default();
        let mut selected = base.clone();
        selected.method = native::Method::Idas;
        assert_ne!(
            profile_identity(&base).unwrap(),
            profile_identity(&selected).unwrap()
        );
        let idas = profile_identity(&selected).unwrap();
        selected.trial_failures = native::TrialPolicy::Recoverable;
        assert_ne!(idas, profile_identity(&selected).unwrap());
        selected = base.clone();
        selected.samples.reverse();
        if base.samples.len() > 1 {
            assert_ne!(
                profile_identity(&base).unwrap(),
                profile_identity(&selected).unwrap()
            );
        }
        selected = base.clone();
        selected.time_limit += std::time::Duration::from_nanos(1);
        assert_ne!(
            profile_identity(&base).unwrap(),
            profile_identity(&selected).unwrap()
        );
    }
}
