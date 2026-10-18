// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dynamic roles project the shared typed compiler; only attempts own native state.
/// Observe an actual synchronous workflow analysis under its transferred invocation.
/// Its inclusive interval is not another work charge or scientific permission.
pub(crate) fn analysis<T, E>(
    name: &'static str,
    work: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let stage = crate::invocation::stage(name);
    let result = work();
    stage.finish(if result.is_ok() {
        crate::invocation::InvocationOutcome::Completed
    } else {
        crate::invocation::InvocationOutcome::Failed
    });
    result
}

use crate::math::ExecutableCase;
use pse_backend_native::{
    ProblemError,
    dynamics::{self as native, Function, Oracle},
};
use pse_ids::{FramedHasher, SemanticId};
use pse_math::{
    assembly::{BoundValues, CaseWorker},
    binding::CaseValues,
};
use std::{collections::BTreeMap, sync::Arc};
/// Dynamic controls use the concrete native profile; values are validated at preparation.
pub type SimulationProfile = native::Profile;
type AllocateSnapshot =
    dyn Fn(usize) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> + Send + Sync;
/// Reserve each owned derivative snapshot before its matrix is cloned. The
/// resulting owner follows all aliases, independently of the worker lifetime.
#[derive(Clone)]
pub(crate) struct SnapshotAllocator(Arc<AllocateSnapshot>);
impl std::fmt::Debug for SnapshotAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SnapshotAllocator")
    }
}
impl SnapshotAllocator {
    pub(crate) fn new(
        allocate: impl Fn(usize) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self(Arc::new(allocate))
    }
    pub(crate) fn reserve(
        &self,
        bytes: usize,
    ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
        (self.0)(bytes)
    }
}
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
    binding: FrameBinding,
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
        program: &GuardProgram,
        check: &RangeCheck,
        rows: &[f64],
        values: &DynamicFrame,
    ) -> Result<Observed, ProblemError> {
        let read = |v: &RangeValue| -> Result<f64, ProblemError> {
            match v {
                RangeValue::Input(id) => values
                    .get(*id)
                    .ok_or_else(|| ProblemError::Internal("dynamic range input absent".into())),
                RangeValue::Output(id) => program
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
    fn validate(&mut self, values: &DynamicFrame) -> Result<(), ProblemError> {
        self.binding.refill(values)?;
        let rows = self.worker.constraints(self.binding.trial()?)?;
        for check in &self.program.checks {
            let Observed {
                value,
                lower,
                upper,
            } = Self::observe(&self.program, check, rows, values)?;
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
        values: &DynamicFrame,
    ) -> Result<BTreeMap<SemanticId, (f64, f64)>, ProblemError> {
        self.binding.refill(values)?;
        let rows = self.worker.constraints(self.binding.trial()?)?;
        let mut ranges = BTreeMap::new();
        for check in &self.program.checks {
            let observed = Self::observe(&self.program, check, rows, values)?;
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
    pub(crate) fn binding_buffer_bound(
        assembly: &pse_math::assembly::CaseAssembly,
    ) -> Option<usize> {
        FrameBinding::allocation_bound(assembly)
    }
    pub(crate) fn function_buffer_bound(
        program: &FunctionProgram,
        coordinates: &DynamicCoordinates,
    ) -> Option<usize> {
        FrameBinding::allocation_bound(&program.case.assembly)?
            .checked_add(ActionScratch::allocation_bound(program, coordinates)?)?
            .checked_add(program.rows.len().checked_mul(size_of::<f64>())?)?
            .checked_add(
                1usize
                    .checked_add(coordinates.state.len())?
                    .checked_add(coordinates.parameters.len())?
                    .checked_mul(size_of::<u64>())?,
            )
    }
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
                n.checked_add(p.case.assembly.numeric_worker_bytes())?
                    .checked_add(Self::function_buffer_bound(p, coordinates)?)
            })?;
            let buffers = modes.clone().try_fold(buffers, |n, m| {
                n.checked_add(m.guard.as_ref().map_or(Some(0), |g| {
                    g.case
                        .assembly
                        .numeric_worker_bytes()
                        .checked_add(FrameBinding::allocation_bound(&g.case.assembly)?)
                })?)
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
        snapshots: SnapshotAllocator,
    ) -> Result<DynamicWorker, ProblemError> {
        DynamicWorker::new(
            self.contract.clone(),
            &self.programs,
            self.coordinates.clone(),
            &self.modes,
            self.layout.clone(),
            scope,
            snapshots,
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
    native: Arc<native::PreparedLayout>,
    chain: Vec<f64>,
    functions: BTreeMap<(usize, Function), Arc<FunctionLayout>>,
}
#[derive(Debug)]
struct FunctionLayout {
    jacobian: pse_math::sparse::AssemblyMatrix,
    refill: Vec<(usize, usize, f64)>,
    pairs: Vec<native::SupportEntry>,
    /// Assembly column to native state/parameter axis, compiled by semantic identity.
    columns: Vec<usize>,
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
            let axes = coordinates
                .state
                .iter()
                .chain(&coordinates.parameters)
                .map(|c| c.id)
                .collect::<Vec<_>>();
            let columns = program
                .case
                .assembly
                .columns()
                .iter()
                .map(|id| {
                    axes.iter().position(|axis| axis == id).ok_or_else(|| {
                        ProblemError::Contract("dynamic derivative coordinate identity".into())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
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
            for (source_col, &c) in columns.iter().enumerate() {
                let scale = chain[c];
                for k in source.col_range(source_col) {
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
            let refill = refill
                .into_iter()
                .map(|(source, target, scale)| {
                    Ok((source, jacobian.contribution_index(target)?, scale))
                })
                .collect::<Result<Vec<_>, pse_math::MathError>>()?;
            if functions
                .insert(
                    (program.mode, program.function),
                    Arc::new(FunctionLayout {
                        jacobian,
                        refill,
                        pairs,
                        columns,
                    }),
                )
                .is_some()
            {
                return Err(ProblemError::internal("duplicate dynamic function"));
            }
        }
        let native = Arc::new(native::PreparedLayout::new(
            coordinates.state.len(),
            functions
                .iter()
                .filter(|((_, function), _)| *function == Function::Rhs)
                .flat_map(|(_, layout)| layout.pairs.iter().copied()),
            max_cells
                .checked_mul(64)
                .ok_or_else(|| ProblemError::memory("dynamic native layout allowance extent"))?,
        )?);
        Ok(Self {
            chain,
            functions,
            native,
        })
    }
    pub(crate) fn native(&self) -> &Arc<native::PreparedLayout> {
        &self.native
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.native.retained_bytes()
            + self.chain.capacity() * size_of::<f64>()
            + self
                .functions
                .values()
                .map(|f| {
                    size_of::<FunctionLayout>()
                        + f.jacobian.retained_bytes()
                        + f.refill.capacity() * size_of::<(usize, usize, f64)>()
                        + f.pairs.capacity() * size_of::<native::SupportEntry>()
                        + f.columns.capacity() * size_of::<usize>()
                        + 128
                })
                .sum::<usize>()
    }
    pub(crate) fn worker_bytes(&self) -> usize {
        self.functions
            .values()
            .map(|_| size_of::<Option<pse_math::SharedAllocation<faer::sparse::SparseColMat<usize, f64>>>>())
            .sum()
    }
}
impl DynamicWorker {
    #[cfg(test)]
    pub(crate) fn evaluation_count(&self, mode: usize, function: Function) -> usize {
        self.functions[&(mode, function)].evaluations
    }
    #[cfg(test)]
    pub(crate) fn buffer_identity(
        &self,
        mode: usize,
        function: Function,
    ) -> (usize, usize, usize, usize) {
        let worker = &self.functions[&(mode, function)];
        (
            worker.values.as_ptr() as usize,
            worker.values.capacity(),
            worker.key.as_ptr() as usize,
            worker.key.capacity(),
        )
    }
    #[cfg(all(test, feature = "solver-idas"))]
    pub(crate) fn action_buffer_identity(&self, mode: usize, function: Function) -> [usize; 4] {
        let scratch = &self.functions[&(mode, function)].actions;
        [
            scratch.seed.as_ptr() as usize,
            scratch.local.as_ptr() as usize,
            scratch.multipliers.as_ptr() as usize,
            scratch.physical.as_ptr() as usize,
        ]
    }
    #[cfg(test)]
    pub(crate) fn action_buffer_sizes(&self, mode: usize, function: Function) -> [usize; 4] {
        let scratch = &self.functions[&(mode, function)].actions;
        [
            scratch.seed.len(),
            scratch.local.len(),
            scratch.multipliers.len(),
            scratch.physical.len(),
        ]
    }
    fn new(
        contract: native::Contract,
        programs: &[FunctionProgram],
        coordinates: DynamicCoordinates,
        modes: &[DynamicMode],
        layout: Arc<DynamicLayout>,
        scope: pse_kernels::ExecutionScope,
        snapshots: SnapshotAllocator,
    ) -> Result<Self, ProblemError> {
        scope.check().map_err(ProblemError::Provider)?;
        if modes.len() != contract.events.len() {
            return Err(ProblemError::Internal("dynamic mode extent".into()));
        }
        let frames = modes
            .iter()
            .map(|mode| DynamicFrame::new(&mode.values, &coordinates))
            .collect::<Result<Vec<_>, _>>()?;
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
                    binding: FrameBinding::new(&program.case.assembly, &frames[program.mode])?,
                    jacobian: None,
                    layout: function_layout,
                    values: Vec::with_capacity(program.rows.len()),
                    key: Vec::with_capacity(
                        1 + coordinates.state.len() + coordinates.parameters.len(),
                    ),
                    cache: None,
                    actions: ActionScratch::new(program, &coordinates)?,
                    #[cfg(test)]
                    evaluations: 0,
                },
            );
        }
        let modes = modes
            .iter()
            .zip(frames)
            .map(|(mode, frame)| {
                let guard = mode
                    .guard
                    .as_ref()
                    .map(|g| {
                        Ok::<_, ProblemError>(GuardWorker {
                            program: g.clone(),
                            binding: FrameBinding::new(&g.case.assembly, &frame)?,
                            worker: g.case.assembly.worker_scoped(
                                provider_workers(&mode.providers, &scope)?,
                                scope.clone(),
                            ),
                        })
                    })
                    .transpose()?;
                Ok::<_, ProblemError>(ModeWorker { frame, guard })
            })
            .collect::<Result<_, _>>()?;
        Ok(DynamicWorker {
            modes,
            contract,
            coordinates,
            functions,
            layout,
            scope,
            snapshots,
            #[cfg(feature = "solver-diffsol")]
            observed_layout: None,
        })
    }
}
#[derive(Debug)]
struct FunctionWorker {
    program: FunctionProgram,
    worker: CaseWorker,
    binding: FrameBinding,
    jacobian: Option<pse_math::SharedAllocation<faer::sparse::SparseColMat<usize, f64>>>,
    layout: Arc<FunctionLayout>,
    // Mode/function and provider/build identity are fixed by this worker. Every
    // varying time/state/parameter bit participates, including signed zero.
    values: Vec<f64>,
    key: Vec<u64>,
    cache: Option<bool>,
    actions: ActionScratch,
    #[cfg(test)]
    evaluations: usize,
}
/// One admitted workspace for directional products. The physical candidate is
/// separate from the caller's output so failed products never publish a prefix.
#[derive(Debug)]
struct ActionScratch {
    seed: Vec<f64>,
    local: Vec<f64>,
    multipliers: Vec<f64>,
    physical: Vec<f64>,
}
impl ActionScratch {
    fn extents(program: &FunctionProgram, coordinates: &DynamicCoordinates) -> Option<[usize; 4]> {
        let jacobian = program.case.assembly.has_directional_actions();
        let hessian = program.case.assembly.has_hessian_actions();
        let columns = program.case.assembly.columns().len();
        let rows = program.case.assembly.structure().rows().len();
        let physical = coordinates
            .state
            .len()
            .checked_add(coordinates.parameters.len())?;
        Some([
            if jacobian || hessian { columns } else { 0 },
            (if jacobian { rows } else { 0 }).max(if hessian { columns } else { 0 }),
            if hessian { rows } else { 0 },
            (if jacobian { program.rows.len() } else { 0 }).max(if hessian { physical } else { 0 }),
        ])
    }
    fn allocation_bound(
        program: &FunctionProgram,
        coordinates: &DynamicCoordinates,
    ) -> Option<usize> {
        Self::extents(program, coordinates)?
            .into_iter()
            .try_fold(0usize, |n, size| n.checked_add(size))?
            .checked_mul(size_of::<f64>())?
            .checked_add(size_of::<Self>())
    }
    fn new(
        program: &FunctionProgram,
        coordinates: &DynamicCoordinates,
    ) -> Result<Self, ProblemError> {
        let [seed, local, multipliers, physical] = Self::extents(program, coordinates)
            .ok_or_else(|| ProblemError::memory("dynamic action workspace extent"))?;
        Ok(Self {
            seed: vec![0.; seed],
            local: vec![0.; local],
            multipliers: vec![0.; multipliers],
            physical: vec![0.; physical],
        })
    }
}
/// Attempt-owned dense physical values; the semantic index is compiled once.
#[derive(Debug)]
struct DynamicFrame {
    index: BTreeMap<SemanticId, usize>,
    values: Vec<f64>,
    dynamic: Vec<usize>,
    scratch: Vec<f64>,
}
impl DynamicFrame {
    fn new(values: &CaseValues, coordinates: &DynamicCoordinates) -> Result<Self, ProblemError> {
        if !coordinates.time_scale.is_finite()
            || coordinates.time_scale <= 0.0
            || !coordinates.time_origin.is_finite()
            || coordinates
                .state
                .iter()
                .chain(&coordinates.parameters)
                .any(|c| !c.scale.is_finite() || c.scale <= 0.0 || !c.offset.is_finite())
        {
            return Err(ProblemError::Contract(
                "dynamic coordinate conversion".into(),
            ));
        }
        let mut scalars = values.scalars.clone();
        let ids = std::iter::once(coordinates.time)
            .chain(
                coordinates
                    .state
                    .iter()
                    .chain(&coordinates.parameters)
                    .map(|c| c.id),
            )
            .collect::<Vec<_>>();
        for (i, id) in ids.iter().enumerate() {
            if ids[..i].contains(id) {
                return Err(ProblemError::Contract(
                    "duplicate dynamic coordinate".into(),
                ));
            }
            scalars.entry(*id).or_insert(0.0);
        }
        let index = scalars
            .keys()
            .enumerate()
            .map(|(i, id)| (*id, i))
            .collect::<BTreeMap<_, _>>();
        let dynamic = ids.iter().map(|id| index[id]).collect();
        Ok(Self {
            index,
            values: scalars.values().copied().collect(),
            dynamic,
            scratch: vec![0.0; ids.len()],
        })
    }
    fn get(&self, id: SemanticId) -> Option<f64> {
        self.index.get(&id).map(|i| self.values[*i])
    }
}
/// Immutable gathers plus attempt-owned compact fixed and coordinate frames.
#[derive(Debug)]
struct FrameBinding {
    values: BoundValues,
    fixed: Vec<usize>,
    columns: Vec<usize>,
    trial: Vec<f64>,
}
impl FrameBinding {
    fn allocation_bound(assembly: &pse_math::assembly::CaseAssembly) -> Option<usize> {
        assembly
            .input_ids()
            .len()
            .checked_add(assembly.columns().len())?
            .checked_mul(size_of::<usize>() + size_of::<f64>())?
            .checked_add(size_of::<Self>())
    }
    fn new(
        assembly: &Arc<pse_math::assembly::CaseAssembly>,
        frame: &DynamicFrame,
    ) -> Result<Self, ProblemError> {
        let reads = |ids: &[SemanticId]| {
            ids.iter()
                .map(|id| {
                    frame.index.get(id).copied().ok_or_else(|| {
                        ProblemError::Contract("missing dynamic compact input".into())
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        };
        Ok(Self {
            values: assembly.bind_inputs(|id| frame.get(id))?,
            fixed: reads(assembly.input_ids())?,
            columns: reads(assembly.columns())?,
            trial: vec![0.0; assembly.columns().len()],
        })
    }
    fn refill(&mut self, frame: &DynamicFrame) -> Result<(), ProblemError> {
        self.values.refill_mapped(&frame.values, &self.fixed)?;
        for (value, &read) in self.trial.iter_mut().zip(&self.columns) {
            *value = frame.values[read];
        }
        Ok(())
    }
    fn trial(&self) -> Result<pse_math::assembly::EvalInputs<'_>, ProblemError> {
        Ok(self.values.trial(&self.trial)?)
    }
}
#[derive(Debug)]
struct ModeWorker {
    frame: DynamicFrame,
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
    snapshots: SnapshotAllocator,
    #[cfg(feature = "solver-diffsol")]
    observed_layout: Option<Arc<native::AnchoredLayout>>,
}
impl DynamicWorker {
    /// Borrow an anchored view without retaining native integration history. Plain
    /// views forward the base Arc; the observed quadrature transform is prepared
    /// only on first actual adjoint demand and remains owned by this lane.
    #[cfg(feature = "solver-diffsol")]
    pub(crate) fn window(
        &mut self,
        observe_quadratures: bool,
    ) -> Result<native::Anchored<&mut Self>, ProblemError> {
        let observed = observe_quadratures && !self.contract.quadratures.is_empty();
        let reused = observed && self.observed_layout.is_some();
        if observed && self.observed_layout.is_none() {
            let entries = self
                .layout
                .functions
                .iter()
                .filter(|((_, function), _)| {
                    matches!(function, Function::Rhs | Function::QuadratureFlux)
                })
                .try_fold(0usize, |n, (_, function)| {
                    n.checked_add(function.pairs.len())
                })
                .ok_or_else(|| ProblemError::memory("window support extent"))?;
            let states = self
                .contract
                .states
                .len()
                .checked_add(self.contract.quadratures.len())
                .ok_or_else(|| ProblemError::memory("window state extent"))?;
            let allowance = native::PreparedLayout::construction_bound(states, entries)
                // Source support vectors, their extension/reallocation overlap and
                // all-mode headers coexist with the destination construction.
                .and_then(|n| n.checked_add(entries.checked_mul(4 * size_of::<native::SupportEntry>())?))
                .and_then(|n| n.checked_add(self.contract.events.len().checked_mul(size_of::<Vec<native::SupportEntry>>())?))
                .and_then(|n| n.checked_add(256))
                .ok_or_else(|| ProblemError::memory("window layout construction extent"))?;
            let prepared = {
                let view = native::Anchored::new(&mut *self, true)?;
                view.prepare_layout(allowance)?
            };
            self.observed_layout = Some(prepared);
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("dynamics.window_layout_constructions", "count", 1);
            }
        }
        let prepared = if observed {
            self.observed_layout.clone()
        } else {
            None
        };
        let mut view = native::Anchored::new(self, observe_quadratures)?;
        if let Some(prepared) = prepared {
            view.use_prepared_layout(prepared)?;
            if reused && let Some(context) = pse_diagnostics::observation::current() {
                context.count("dynamics.window_layout_reuses", "count", 1);
            }
        }
        Ok(view)
    }
    #[cfg(test)]
    pub(crate) fn assert_independent_layout(&self, other: &Self) {
        assert!(Arc::ptr_eq(&self.layout, &other.layout));
        for (key, f) in &self.functions {
            let g = &other.functions[key];
            assert!(Arc::ptr_eq(&f.layout, &g.layout));
            if !f.binding.trial.is_empty() {
                assert_ne!(f.binding.trial.as_ptr(), g.binding.trial.as_ptr());
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
            guard.validate(&context.frame)?;
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
            Some(guard) => guard.ranges(&context.frame)?,
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
        // Stage every affine conversion before publishing any value to a consumer.
        context.frame.scratch[0] = (time - d.time_origin) / d.time_scale;
        for (i, (c, x)) in d
            .state
            .iter()
            .zip(state)
            .chain(d.parameters.iter().zip(parameters))
            .enumerate()
        {
            context.frame.scratch[i + 1] = x * c.scale + c.offset;
        }
        if context.frame.scratch.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "nonfinite dynamic affine binding".into(),
            ));
        }
        for (&slot, &value) in context.frame.dynamic.iter().zip(&context.frame.scratch) {
            context.frame.values[slot] = value;
        }
        Ok(())
    }
}
impl Oracle for DynamicWorker {
    fn snapshot_owner(
        &self,
        bytes: usize,
    ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
        self.snapshots.reserve(bytes)
    }
    fn contract(&self) -> &native::Contract {
        &self.contract
    }
    fn prepared_layout(
        &self,
        allowance: usize,
    ) -> Result<Arc<native::PreparedLayout>, ProblemError> {
        if self.layout.native.retained_bytes() > allowance {
            return Err(ProblemError::memory(
                "dynamic native layout exceeds allowance",
            ));
        }
        Ok(self.layout.native.clone())
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
        let result = (|| {
            self.scope.check().map_err(ProblemError::Provider)?;
            if state.len() != self.coordinates.state.len()
                || parameters.len() != self.coordinates.parameters.len()
                || !time.is_finite()
                || state.iter().chain(parameters).any(|v| !v.is_finite())
            {
                return Err(ProblemError::Contract(
                    "dynamic binding dimensions or values".into(),
                ));
            }
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
            if let Some(has_derivatives) = function.cache
                && (!derivatives || has_derivatives)
                && function.key.iter().copied().eq(bits.clone())
            {
                return Ok(native::Evaluation {
                    values: function.values.clone(),
                    jacobian: if derivatives {
                        function.jacobian.clone()
                    } else {
                        None
                    },
                });
            }
            self.bind(mode, role, time, state, parameters)?;
            let function = self.functions.get_mut(&(mode, role)).ok_or_else(|| {
                ProblemError::Internal("missing compiled dynamic function".into())
            })?;
            function.cache = None;
            #[cfg(test)]
            {
                function.evaluations += 1;
            }
            let context = &self.modes[mode];
            let p = &function.program;
            function.binding.refill(&context.frame)?;
            let rows = function.worker.constraints(function.binding.trial()?)?;
            function.values.clear();
            for (i, &r) in p.rows.iter().enumerate() {
                function.values.push(
                    p.constants
                        .get(&i)
                        .copied()
                        .unwrap_or_else(|| rows[r] * p.scales[i] + p.offsets[i]),
                );
            }
            if function.values.iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical("nonfinite dynamic values"));
            }
            let jacobian = if derivatives {
                let source = function.worker.jacobian(function.binding.trial()?)?;
                // Escaped snapshots stay immutable. The common drained callback path
                // reuses its charged allocation, including CSC storage, without a clone.
                if function
                    .jacobian
                    .as_mut()
                    .and_then(|j| j.get_mut())
                    .is_none()
                {
                    let bytes = function
                        .layout
                        .jacobian
                        .retained_bytes()
                        .checked_add(size_of::<usize>() * 2)
                        .ok_or_else(|| {
                            ProblemError::memory("dynamic derivative snapshot extent")
                        })?;
                    let owner = self.snapshots.reserve(bytes)?;
                    function.jacobian = Some(
                        pse_math::SharedAllocation::from(Arc::new(
                            function.layout.jacobian.matrix().clone(),
                        ))
                        .with_owner(owner),
                    );
                }
                let snapshot = function
                    .jacobian
                    .as_mut()
                    .ok_or_else(|| ProblemError::internal("dynamic derivative snapshot absent"))?;
                let matrix = snapshot.get_mut().ok_or_else(|| {
                    ProblemError::internal("dynamic derivative snapshot still borrowed")
                })?;
                matrix.val_mut().fill(0.0);
                for &(local, target, scale) in &function.layout.refill {
                    let value = matrix.val()[target] + source.val()[local] * scale;
                    if !value.is_finite() {
                        return Err(ProblemError::numerical("nonfinite dynamic partials"));
                    }
                    matrix.val_mut()[target] = value;
                }
                Some(snapshot.clone())
            } else {
                None
            };
            // The native Oracle contract returns an owned result. Keep the attempt
            // scratch and copy only at that escaping boundary.
            let result = native::Evaluation {
                values: function.values.clone(),
                jacobian,
            };
            self.scope.check().map_err(ProblemError::Provider)?;
            function.key.clear();
            function.key.extend(
                std::iter::once(time)
                    .chain(state.iter().copied())
                    .chain(parameters.iter().copied())
                    .map(f64::to_bits),
            );
            function.cache = Some(derivatives);
            Ok(result)
        })();
        if result.is_err() {
            for function in self.functions.values_mut() {
                function.cache = None;
                function.key.clear();
                function.values.fill(f64::NAN);
                function.values.clear();
                function.worker.invalidate_trial();
            }
            for mode in &mut self.modes {
                if let Some(guard) = &mut mode.guard {
                    guard.worker.invalidate_trial();
                }
            }
        }
        result
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit function, physical point and runtime product bindings have distinct scientific meanings"
    )]
    fn jacobian_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.bind(mode, function, time, state, parameters)?;
        let chain = &self.layout.chain;
        let worker = self
            .functions
            .get_mut(&(mode, function))
            .ok_or_else(|| ProblemError::internal("missing compiled dynamic action function"))?;
        if !worker.worker.assembly().has_directional_actions() {
            return Err(ProblemError::unsupported(
                "the selected dynamic profile did not prepare Jacobian actions",
            ));
        }
        let p = &worker.program;
        if direction.len() != chain.len()
            || output.len() != p.rows.len()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "dynamic Jacobian action dimensions or direction".into(),
            ));
        }
        for (seed, &axis) in worker.actions.seed.iter_mut().zip(&worker.layout.columns) {
            *seed = direction[axis] * chain[axis];
        }
        worker.binding.refill(&self.modes[mode].frame)?;
        let rows = worker.worker.assembly().structure().rows().len();
        worker.worker.jacobian_product(
            worker.binding.trial()?,
            &worker.actions.seed,
            &mut worker.actions.local[..rows],
        )?;
        for (i, &row) in p.rows.iter().enumerate() {
            worker.actions.physical[i] = if p.constants.contains_key(&i) {
                0.
            } else {
                worker.actions.local[row] * p.scales[i]
            };
        }
        let candidate = &worker.actions.physical[..output.len()];
        if candidate.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite physical Jacobian action",
            ));
        }
        self.scope.check().map_err(ProblemError::Provider)?;
        output.copy_from_slice(candidate);
        Ok(())
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit physical point, output weights and direction are separate scientific product bindings"
    )]
    fn weighted_hessian_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        if self.contract.derivatives < pse_kernels::DerivativeOrder::Second {
            return Err(ProblemError::unsupported(
                "the dynamic functions were prepared without second derivatives",
            ));
        }
        self.bind(mode, function, time, state, parameters)?;
        let chain = &self.layout.chain;
        let worker = self.functions.get_mut(&(mode, function)).ok_or_else(|| {
            ProblemError::internal("missing compiled dynamic Hessian action function")
        })?;
        if !worker.worker.assembly().has_hessian_actions() {
            return Err(ProblemError::unsupported(
                "the selected dynamic profile did not prepare Hessian actions",
            ));
        }
        let p = &worker.program;
        if weights.len() != p.rows.len()
            || direction.len() != chain.len()
            || output.len() != chain.len()
            || weights.iter().chain(direction).any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "dynamic Hessian action dimensions or bindings".into(),
            ));
        }
        worker.actions.multipliers.fill(0.);
        for (i, &row) in p.rows.iter().enumerate() {
            if !p.constants.contains_key(&i) {
                worker.actions.multipliers[row] += weights[i] * p.scales[i];
            }
        }
        for (seed, &axis) in worker.actions.seed.iter_mut().zip(&worker.layout.columns) {
            *seed = direction[axis] * chain[axis];
        }
        worker.binding.refill(&self.modes[mode].frame)?;
        let columns = worker.actions.seed.len();
        worker.worker.hessian_product(
            worker.binding.trial()?,
            0.,
            &worker.actions.multipliers,
            &worker.actions.seed,
            &mut worker.actions.local[..columns],
        )?;
        worker.actions.physical[..chain.len()].fill(0.);
        for (local, &axis) in worker.layout.columns.iter().enumerate() {
            worker.actions.physical[axis] += worker.actions.local[local] * chain[axis];
        }
        let candidate = &worker.actions.physical[..output.len()];
        if candidate.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite physical Hessian action"));
        }
        self.scope.check().map_err(ProblemError::Provider)?;
        output.copy_from_slice(candidate);
        Ok(())
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
        worker.binding.refill(&self.modes[mode].frame)?;
        let h = worker
            .worker
            .hessian(worker.binding.trial()?, 0.0, &multipliers)?;
        if h.ncols() != chain.len() || h.nrows() != chain.len() {
            return Err(ProblemError::Internal(
                "dynamic Hessian coordinate extent".into(),
            ));
        }
        let mut triplets = Vec::with_capacity(h.val().len());
        for col in 0..h.ncols() {
            for k in h.col_range(col) {
                let row = h.row_idx()[k];
                let native_row = worker.layout.columns[row];
                let native_col = worker.layout.columns[col];
                let value = h.val()[k] * chain[native_row] * chain[native_col];
                if !value.is_finite() {
                    return Err(ProblemError::numerical(
                        "nonfinite dynamic Hessian conversion",
                    ));
                }
                triplets.push(faer::sparse::Triplet::new(
                    native_row.max(native_col),
                    native_row.min(native_col),
                    value,
                ));
            }
        }
        faer::sparse::SparseColMat::try_new_from_triplets(chain.len(), chain.len(), &triplets)
            .map_err(|e| ProblemError::memory(format!("dynamic Hessian assembly: {e:?}")))
    }
}

// An anchored window borrows its lane's mathematical/provider worker. Its native IVP
// remains fresh, while compiled scratch and provider owners never leave that lane.
impl Oracle for &mut DynamicWorker {
    fn prepared_layout(
        &self,
        allowance: usize,
    ) -> Result<Arc<native::PreparedLayout>, ProblemError> {
        (**self).prepared_layout(allowance)
    }
    fn contract(&self) -> &native::Contract {
        (**self).contract()
    }
    fn support(&self, mode: usize, function: Function) -> Vec<native::SupportEntry> {
        (**self).support(mode, function)
    }
    fn snapshot_owner(
        &self,
        bytes: usize,
    ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
        (**self).snapshot_owner(bytes)
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
        (**self).evaluate(mode, function, time, state, parameters, derivatives)
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit function, physical point and runtime product bindings have distinct scientific meanings"
    )]
    fn jacobian_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        (**self).jacobian_action(mode, function, time, state, parameters, direction, output)
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit function, physical point and runtime product bindings have distinct scientific meanings"
    )]
    fn weighted_hessian_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        (**self).weighted_hessian_action(
            mode, function, time, state, parameters, weights, direction, output,
        )
    }
    fn weighted_hessian(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
    ) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
        (**self).weighted_hessian(mode, function, time, state, parameters, weights)
    }
}

pub(crate) fn profile_identity(
    p: &SimulationProfile,
) -> Result<pse_ids::roles::ProfileHash, ProblemError> {
    let mut h = FramedHasher::new(pse_ids::Frame::DynamicProfileV11);
    pse_ids::document::frame(&mut h, p)
        .map_err(|error| ProblemError::Internal(error.to_string()))?;
    h.str("forward_axes");
    pse_ids::document::frame(&mut h, &p.forward_axes)
        .map_err(|error| ProblemError::Internal(error.to_string()))?;
    h.str("adjoint_parameters");
    pse_ids::document::frame(&mut h, &p.adjoint_parameters)
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
    fn simulation_identity_distinguishes_selected_physical_axes() {
        let full = SimulationProfile::default();
        let mut keys = std::collections::BTreeSet::new();
        for axes in [
            None,
            Some(vec![]),
            Some(vec![0]),
            Some(vec![1]),
            Some(vec![1, 0]),
        ] {
            let mut profile = full.clone();
            profile.forward_axes = axes;
            assert!(keys.insert(profile_identity(&profile).unwrap()));
        }
    }
    #[test]
    fn simulation_identity_distinguishes_ordered_adjoint_parameter_demand() {
        let mut keys = std::collections::BTreeSet::new();
        for selection in [
            native::AdjointParameters::All,
            native::AdjointParameters::Selected(vec![]),
            native::AdjointParameters::Selected(vec![0]),
            native::AdjointParameters::Selected(vec![1, 0]),
            native::AdjointParameters::Selected(vec![0, 1]),
        ] {
            let profile = SimulationProfile {
                adjoint_parameters: selection,
                ..SimulationProfile::default()
            };
            assert!(keys.insert(profile_identity(&profile).unwrap()));
        }
    }
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
