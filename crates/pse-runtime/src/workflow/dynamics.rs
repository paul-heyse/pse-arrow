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

mod neutral;
use crate::math::ExecutableCase;
pub(crate) use neutral::{DynamicScratch, NeutralCase};
use pse_backend_native::{
    ProblemError,
    dynamics::{self as native, Function, Oracle},
};
use pse_ids::SemanticId;
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
) -> Result<pse_kernels::ProviderBindings, ProblemError> {
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
                program
                    .case
                    .assembly
                    .jacobian_entries()
                    .len()
                    .checked_mul(size_of::<f64>())?,
            )?
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
        self.worker_with_scratch(scope, snapshots, None)
    }
    pub(crate) fn worker_with_scratch(
        &self,
        scope: pse_kernels::ExecutionScope,
        snapshots: SnapshotAllocator,
        scratch: Option<&DynamicScratch>,
    ) -> Result<DynamicWorker, ProblemError> {
        DynamicWorker::new(self, scope, snapshots, scratch)
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
    jacobian: Option<pse_math::sparse::AssemblyMatrix>,
    diagonal_scales: Option<Vec<f64>>,
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
            let full = program.case.assembly.order() >= pse_kernels::DerivativeOrder::First;
            let jacobian = full
                .then(|| {
                    pse_math::sparse::AssemblyMatrix::new(
                        program.rows.len(),
                        chain.len(),
                        &pairs,
                        max_cells,
                    )
                })
                .transpose()?;
            let refill = if let Some(jacobian) = &jacobian {
                refill
                    .into_iter()
                    .map(|(source, target, scale)| {
                        Ok((source, jacobian.contribution_index(target)?, scale))
                    })
                    .collect::<Result<Vec<_>, pse_math::MathError>>()?
            } else {
                Vec::new()
            };
            let diagonal_scales = if program.function == Function::Rhs
                && !full
                && !program.case.assembly.jacobian_entries().is_empty()
            {
                let entries = program.case.assembly.jacobian_entries();
                if entries.len() != coordinates.state.len() || entries.len() != program.rows.len() {
                    return Err(ProblemError::Contract(
                        "dynamic selected diagonal extent".into(),
                    ));
                }
                let mut scales = Vec::with_capacity(entries.len());
                for (i, &(row, column)) in entries.iter().enumerate() {
                    if row != program.case.assembly.structure().rows()[program.rows[i]].id
                        || column != coordinates.state[i].id
                    {
                        return Err(ProblemError::Contract(
                            "dynamic selected diagonal identity/order".into(),
                        ));
                    }
                    let scale = if program.constants.contains_key(&i) {
                        0.
                    } else {
                        program.scales[i] * coordinates.state[i].scale
                    };
                    if !scale.is_finite() {
                        return Err(ProblemError::numerical("dynamic selected diagonal scale"));
                    }
                    scales.push(scale);
                }
                Some(scales)
            } else {
                None
            };
            if functions
                .insert(
                    (program.mode, program.function),
                    Arc::new(FunctionLayout {
                        jacobian,
                        diagonal_scales,
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
                        + f.jacobian
                            .as_ref()
                            .map_or(0, pse_math::sparse::AssemblyMatrix::retained_bytes)
                        + f.diagonal_scales
                            .as_ref()
                            .map_or(0, |v| v.capacity() * size_of::<f64>())
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
    #[cfg(all(test, any(feature = "solver-idas", feature = "solver-diffsol")))]
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
        source: &DynamicProgram,
        scope: pse_kernels::ExecutionScope,
        snapshots: SnapshotAllocator,
        scratch: Option<&DynamicScratch>,
    ) -> Result<Self, ProblemError> {
        let contract = source.contract.clone();
        let coordinates = source.coordinates.clone();
        let layout = source.layout.clone();
        let programs = &source.programs;
        let modes = &source.modes;
        scope.check().map_err(ProblemError::Provider)?;
        if modes.len() != contract.events.len() {
            return Err(ProblemError::Internal("dynamic mode extent".into()));
        }
        let frames = modes
            .iter()
            .map(|mode| DynamicFrame::new(&mode.values, &coordinates))
            .collect::<Result<Vec<_>, _>>()?;
        let mut functions = BTreeMap::new();
        for program in programs.iter() {
            let mode = modes
                .get(program.mode)
                .ok_or_else(|| ProblemError::internal("dynamic function mode absent"))?;
            let function_layout = layout
                .functions
                .get(&(program.mode, program.function))
                .ok_or_else(|| ProblemError::internal("dynamic function layout absent"))?
                .clone();
            let worker = if let Some(slot) =
                scratch.and_then(|scratch| scratch.function(program.mode, program.function))
            {
                slot.worker(&program.case.assembly, &mode.providers, scope.clone())?
            } else {
                let mut worker = program
                    .case
                    .assembly
                    .worker_scoped(provider_workers(&mode.providers, &scope)?, scope.clone());
                worker.prepare_providers()?;
                worker
            };
            functions.insert(
                (program.mode, program.function),
                FunctionWorker {
                    program: program.clone(),
                    worker,
                    binding: FrameBinding::new(&program.case.assembly, &frames[program.mode])?,
                    jacobian: None,
                    layout: function_layout,
                    values: Vec::with_capacity(program.rows.len()),
                    diagonal: vec![f64::NAN; program.case.assembly.jacobian_entries().len()],
                    key: Vec::with_capacity(
                        1 + coordinates.state.len() + coordinates.parameters.len(),
                    ),
                    cache: None,
                    pure: program
                        .case
                        .assembly
                        .programs()
                        .iter()
                        .all(|p| p.census().all(|c| c.provider_stages == 0)),
                    actions: ActionScratch::new(program, &coordinates)?,
                    #[cfg(test)]
                    evaluations: 0,
                },
            );
        }
        let modes = modes
            .iter()
            .zip(frames)
            .enumerate()
            .map(|(mode_index, (mode, frame))| {
                let guard = mode
                    .guard
                    .as_ref()
                    .map(|g| {
                        let worker = if let Some(slot) =
                            scratch.and_then(|scratch| scratch.guard(mode_index))
                        {
                            slot.worker(&g.case.assembly, &mode.providers, scope.clone())?
                        } else {
                            let mut worker = g.case.assembly.worker_scoped(
                                provider_workers(&mode.providers, &scope)?,
                                scope.clone(),
                            );
                            worker.prepare_providers()?;
                            worker
                        };
                        Ok::<_, ProblemError>(GuardWorker {
                            program: g.clone(),
                            binding: FrameBinding::new(&g.case.assembly, &frame)?,
                            worker,
                        })
                    })
                    .transpose()?;
                Ok::<_, ProblemError>(ModeWorker { frame, guard })
            })
            .collect::<Result<_, _>>()?;
        Ok(DynamicWorker {
            #[cfg(feature = "solver-idas")]
            idas_workspace: None,
            modes,
            contract,
            coordinates,
            functions,
            layout,
            scope,
            snapshots,
            #[cfg(feature = "solver-diffsol")]
            window_layouts: [None, None],
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
    diagonal: Vec<f64>,
    key: Vec<u64>,
    cache: Option<FunctionCoverage>,
    pure: bool,
    actions: ActionScratch,
    #[cfg(test)]
    evaluations: usize,
}
/// Completed products at the exact current function key. A derivative product
/// never grants value coverage that was not actually obtained.
#[derive(Clone, Copy, Debug, Default)]
struct FunctionCoverage {
    values: bool,
    full: bool,
    selected: bool,
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
        let transpose = program.case.assembly.has_transpose_actions();
        let columns = program.case.assembly.columns().len();
        let rows = program.case.assembly.structure().rows().len();
        let physical = coordinates
            .state
            .len()
            .checked_add(coordinates.parameters.len())?;
        Some([
            if jacobian || hessian { columns } else { 0 },
            (if jacobian { rows } else { 0 }).max(if hessian || transpose { columns } else { 0 }),
            if hessian || transpose { rows } else { 0 },
            (if jacobian { program.rows.len() } else { 0 }).max(if hessian || transpose {
                physical
            } else {
                0
            }),
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
    #[cfg(feature = "solver-idas")]
    idas_workspace: Option<native::IdasWorkspace>,
    modes: Vec<ModeWorker>,
    contract: native::Contract,
    coordinates: DynamicCoordinates,
    functions: BTreeMap<(usize, Function), FunctionWorker>,
    layout: Arc<DynamicLayout>,
    scope: pse_kernels::ExecutionScope,
    snapshots: SnapshotAllocator,
    #[cfg(feature = "solver-diffsol")]
    window_layouts: [Option<Arc<native::AnchoredLayout>>; 2],
}
impl DynamicWorker {
    #[cfg(feature = "solver-idas")]
    pub(crate) fn take_idas_workspace(&mut self) -> native::IdasWorkspace {
        self.idas_workspace.take().unwrap_or_default()
    }
    #[cfg(feature = "solver-idas")]
    pub(crate) fn restore_idas_workspace(
        &mut self,
        workspace: native::IdasWorkspace,
        success: bool,
    ) {
        self.idas_workspace = success.then_some(workspace);
    }
    #[cfg(feature = "solver-idas")]
    pub(crate) fn discard_idas_workspace(&mut self) {
        self.idas_workspace = None;
    }
    /// Discard trial-derived numerical validity while retaining prepared mathematical owners.
    /// Native integration history is invocation-owned and is never retained here.
    #[cfg(feature = "solver-diffsol")]
    pub(crate) fn invalidate_trial(&mut self) {
        for function in self.functions.values_mut() {
            function.worker.invalidate_trial();
            function.cache = None;
            function.jacobian = None;
            function.values.fill(f64::NAN);
            function.diagonal.fill(f64::NAN);
            function.key.clear();
            function.actions.seed.fill(f64::NAN);
            function.actions.local.fill(f64::NAN);
            function.actions.multipliers.fill(f64::NAN);
            function.actions.physical.fill(f64::NAN);
        }
        for mode in &mut self.modes {
            if let Some(guard) = &mut mode.guard {
                guard.worker.invalidate_trial();
            }
        }
    }
    /// Borrow a window over lane-owned numeric scratch and source-prepared plain
    /// or observed wrapper layouts. Native integration history remains fresh.
    #[cfg(feature = "solver-diffsol")]
    pub(crate) fn window(
        &mut self,
        observe_quadratures: bool,
    ) -> Result<native::Anchored<&mut Self>, ProblemError> {
        let observed = observe_quadratures && !self.contract.quadratures.is_empty();
        let index = usize::from(observed);
        let reused = self.window_layouts[index].is_some();
        if !reused {
            let entries = self
                .layout
                .functions
                .values()
                .try_fold(0usize, |n, function| n.checked_add(function.pairs.len()))
                .ok_or_else(|| ProblemError::memory("window support extent"))?;
            let prepared = {
                let view = native::Anchored::new(&mut *self, observe_quadratures)?;
                let allowance = view.layout_allocation_bound(entries)?;
                view.prepare_layout(allowance)?
            };
            self.window_layouts[index] = Some(prepared);
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("dynamics.window_layout_constructions", "count", 1);
            }
        }
        let prepared = self.window_layouts[index]
            .clone()
            .ok_or_else(|| ProblemError::internal("prepared window layout absent"))?;
        let mut view = native::Anchored::new(self, observe_quadratures)?;
        view.use_prepared_layout(prepared)?;
        if reused && let Some(context) = pse_diagnostics::observation::current() {
            context.count("dynamics.window_layout_reuses", "count", 1);
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
    fn rhs_state_diagonal(
        &mut self,
        mode: usize,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        output: &mut [f64],
    ) -> Result<native::RhsDiagonalSource, ProblemError> {
        let selected = self
            .functions
            .get(&(mode, Function::Rhs))
            .is_some_and(|function| function.layout.diagonal_scales.is_some());
        if !selected {
            return native::full_rhs_state_diagonal(self, mode, time, state, parameters, output);
        }
        let result = (|| {
            self.scope.check().map_err(ProblemError::Provider)?;
            if state.len() != self.coordinates.state.len()
                || output.len() != state.len()
                || parameters.len() != self.coordinates.parameters.len()
                || !time.is_finite()
                || state.iter().chain(parameters).any(|v| !v.is_finite())
            {
                return Err(ProblemError::Contract(
                    "dynamic selected diagonal bindings".into(),
                ));
            }
            // Current guards/provider binding run even when this pure product is cached.
            self.bind(mode, Function::Rhs, time, state, parameters)?;
            let function = self
                .functions
                .get_mut(&(mode, Function::Rhs))
                .ok_or_else(|| ProblemError::internal("dynamic RHS absent"))?;
            let bits = std::iter::once(time)
                .chain(state.iter().copied())
                .chain(parameters.iter().copied())
                .map(f64::to_bits);
            let same =
                function.pure && function.cache.is_some() && function.key.iter().copied().eq(bits);
            if same && function.cache.is_some_and(|coverage| coverage.selected) {
                output.copy_from_slice(&function.diagonal);
                return Ok(native::RhsDiagonalSource::SelectedEntries);
            }
            if !same {
                function.cache = None;
                function.key.clear();
                function.values.clear();
            }
            function.binding.refill(&self.modes[mode].frame)?;
            function
                .worker
                .jacobian_entries(function.binding.trial()?, &mut function.diagonal)?;
            let scales = function
                .layout
                .diagonal_scales
                .as_ref()
                .ok_or_else(|| ProblemError::internal("dynamic diagonal scales absent"))?;
            for (value, scale) in function.diagonal.iter_mut().zip(scales) {
                *value *= scale;
            }
            if function.diagonal.iter().any(|value| !value.is_finite()) {
                return Err(ProblemError::numerical(
                    "nonfinite dynamic selected diagonal",
                ));
            }
            self.scope.check().map_err(ProblemError::Provider)?;
            function.key.clear();
            function.key.extend(
                std::iter::once(time)
                    .chain(state.iter().copied())
                    .chain(parameters.iter().copied())
                    .map(f64::to_bits),
            );
            let mut coverage = function.cache.unwrap_or_default();
            coverage.selected = true;
            function.cache = Some(coverage);
            output.copy_from_slice(&function.diagonal);
            Ok(native::RhsDiagonalSource::SelectedEntries)
        })();
        if result.is_err() {
            for function in self.functions.values_mut() {
                if !function.pure || function.cache.is_none() {
                    function.cache = None;
                    function.key.clear();
                    function.values.clear();
                }
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
    fn evaluate(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        derivatives: bool,
    ) -> Result<native::Evaluation, ProblemError> {
        // An unavailable product cannot begin a provider trial or invalidate an
        // independently completed current product. A numerical key alone does
        // not authorize merging coverage across provider-bearing evaluations.
        if derivatives
            && self
                .functions
                .get(&(mode, function))
                .is_some_and(|function| function.layout.jacobian.is_none())
        {
            self.scope.check().map_err(ProblemError::Provider)?;
            return Err(ProblemError::unsupported(
                "full dynamic Jacobian was not prepared",
            ));
        }
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
            self.bind(mode, role, time, state, parameters)?;
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
            if let Some(coverage) = function.cache
                && coverage.values
                && (!derivatives || coverage.full)
                && function.key.iter().copied().eq(bits.clone())
                && function.pure
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
            let function = self.functions.get_mut(&(mode, role)).ok_or_else(|| {
                ProblemError::Internal("missing compiled dynamic function".into())
            })?;
            // Same-key pure Value coverage remains authoritative during a
            // stronger request. Only a complete successful upgrade may replace it.
            let keep_current =
                function.pure && function.cache.is_some() && function.key.iter().copied().eq(bits);
            if !keep_current {
                function.cache = None;
                function.key.clear();
                function.values.clear();
            }
            #[cfg(test)]
            {
                function.evaluations += 1;
            }
            let context = &self.modes[mode];
            let p = &function.program;
            function.binding.refill(&context.frame)?;
            let rows = function.worker.constraints(function.binding.trial()?)?;
            let mut candidate_values = Vec::with_capacity(p.rows.len());
            for (i, &r) in p.rows.iter().enumerate() {
                candidate_values.push(
                    p.constants
                        .get(&i)
                        .copied()
                        .unwrap_or_else(|| rows[r] * p.scales[i] + p.offsets[i]),
                );
            }
            if candidate_values.iter().any(|v| !v.is_finite()) {
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
                    let template = function.layout.jacobian.as_ref().ok_or_else(|| {
                        ProblemError::unsupported("full dynamic Jacobian was not prepared")
                    })?;
                    let bytes = template
                        .retained_bytes()
                        .checked_add(size_of::<usize>() * 2)
                        .ok_or_else(|| {
                            ProblemError::memory("dynamic derivative snapshot extent")
                        })?;
                    let owner = self.snapshots.reserve(bytes)?;
                    function.jacobian = Some(
                        pse_math::SharedAllocation::from(Arc::new(template.matrix().clone()))
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
                values: candidate_values,
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
            function.values.clear();
            function.values.extend_from_slice(&result.values);
            let previous = function.cache.unwrap_or_default();
            function.cache = Some(FunctionCoverage {
                values: true,
                full: derivatives || previous.full,
                selected: previous.selected,
            });
            Ok(result)
        })();
        if result.is_err() {
            for function in self.functions.values_mut() {
                // Failed stronger work does not erase an independently completed
                // pure Value tuple. Guards and scope are still requalified before
                // every future lookup; effectful owners have no reusable tuple.
                if !function.pure || function.cache.is_none() {
                    function.cache = None;
                    function.key.clear();
                    function.values.fill(f64::NAN);
                    function.values.clear();
                }
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
        reason = "physical point and cotangent bindings are independent scientific inputs"
    )]
    fn jacobian_transpose_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.bind(mode, function, time, state, parameters)?;
        let chain = &self.layout.chain;
        let worker = self
            .functions
            .get_mut(&(mode, function))
            .ok_or_else(|| ProblemError::internal("missing compiled dynamic transpose function"))?;
        if !worker.worker.assembly().has_transpose_actions() {
            return Err(ProblemError::unsupported(
                "the selected dynamic profile did not prepare transpose actions",
            ));
        }
        let p = &worker.program;
        if weights.len() != p.rows.len()
            || output.len() != chain.len()
            || weights.iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "dynamic transpose dimensions or weights".into(),
            ));
        }
        worker.actions.multipliers.fill(0.);
        for (i, &row) in p.rows.iter().enumerate() {
            if !p.constants.contains_key(&i) {
                worker.actions.multipliers[row] += weights[i] * p.scales[i];
            }
        }
        worker.binding.refill(&self.modes[mode].frame)?;
        let columns = worker.worker.assembly().columns().len();
        worker.worker.jacobian_transpose_product(
            worker.binding.trial()?,
            &worker.actions.multipliers,
            &mut worker.actions.local[..columns],
        )?;
        worker.actions.physical[..chain.len()].fill(0.);
        for (local, &axis) in worker.layout.columns.iter().enumerate() {
            worker.actions.physical[axis] += worker.actions.local[local] * chain[axis];
        }
        let candidate = &worker.actions.physical[..output.len()];
        if candidate.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite physical transpose action",
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
    fn rhs_state_diagonal(
        &mut self,
        mode: usize,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        output: &mut [f64],
    ) -> Result<native::RhsDiagonalSource, ProblemError> {
        (**self).rhs_state_diagonal(mode, time, state, parameters, output)
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
        reason = "physical point and cotangent bindings are independent scientific inputs"
    )]
    fn jacobian_transpose_action(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        weights: &[f64],
        output: &mut [f64],
    ) -> Result<(), ProblemError> {
        (**self).jacobian_transpose_action(mode, function, time, state, parameters, weights, output)
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

pub(crate) use native::profile_identity;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "solver-diffsol")]
    #[tokio::test]
    async fn dynamic_failed_full_upgrade_preserves_only_completed_same_key_values() {
        let runtime = super::super::tests::runtime();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; param offset: Time = 1{s}; var x[i in t]: Time; eq ode[i in t]: d(x[i])/di == p; eq initial: x[0{s}] == offset; annotation start x(0{s}); annotation nominal x(10{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::from_bytes([175; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, super::super::tests::physical())
            .await
            .unwrap();
        let prepared = package
            .prepare_simulation(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                pse_modeling::Bindings::default(),
                pse_modeling::Limits::default(),
                pse_compiler::workspace::ModelingCaseBindings::default(),
                super::super::tests::compiler_profile(),
                native::Profile {
                    end: 2.0,
                    samples: vec![0.0, 1.0, 2.0],
                    parameter_scales: vec![1.0, 1.0],
                    sensitivity: native::DynamicSensitivity::Forward,
                    ..native::Profile::default()
                },
                pse_kernels::DerivativeOrder::First,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None);
        let mut worker = prepared.worker(runtime.native(), scope).unwrap();
        let first = worker
            .evaluate(
                0,
                Function::Output,
                0.0,
                &[0.1],
                prepared.parameters(),
                false,
            )
            .unwrap();
        assert!(first.jacobian.is_none());
        assert_eq!(worker.evaluation_count(0, Function::Output), 1);
        let allocator = worker.snapshots.clone();
        worker.snapshots =
            SnapshotAllocator::new(|_| Err(ProblemError::memory("forced candidate refusal")));
        let error = worker
            .evaluate(
                0,
                Function::Output,
                0.0,
                &[0.1],
                prepared.parameters(),
                true,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Memory,
                ..
            }
        ));
        let coverage = worker.functions[&(0, Function::Output)].cache.unwrap();
        assert!(coverage.values && !coverage.full);
        let weaker = worker
            .evaluate(
                0,
                Function::Output,
                0.0,
                &[0.1],
                prepared.parameters(),
                false,
            )
            .unwrap();
        assert_eq!(weaker.values, first.values);
        assert!(weaker.jacobian.is_none());
        assert_eq!(
            worker.evaluation_count(0, Function::Output),
            2,
            "Value is reused, never fabricated as a completed full derivative"
        );
        worker.snapshots = allocator;
        let complete = worker
            .evaluate(
                0,
                Function::Output,
                0.0,
                &[0.1],
                prepared.parameters(),
                true,
            )
            .unwrap();
        assert!(complete.jacobian.is_some());
        assert_eq!(complete.values, first.values);
        assert_eq!(worker.evaluation_count(0, Function::Output), 3);
        let invalid = worker.evaluate(
            0,
            Function::Output,
            f64::NAN,
            &[0.1],
            prepared.parameters(),
            false,
        );
        assert!(invalid.is_err(), "binding guards run before cache lookup");
        let current = worker
            .evaluate(
                0,
                Function::Output,
                0.0,
                &[0.2],
                prepared.parameters(),
                false,
            )
            .unwrap();
        assert_ne!(current.values, first.values);
        assert!(
            !worker.functions[&(0, Function::Output)].cache.unwrap().full,
            "changed points cannot inherit stronger numerical coverage"
        );
    }
    #[cfg(feature = "solver-idas")]
    const SELECTED_JACOBI_WIDTH: usize = 16;
    #[cfg(feature = "solver-idas")]
    fn selected_jacobi_source(with_time_observable: bool) -> String {
        let mut source = String::from(
            "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); ",
        );
        for i in 0..SELECTED_JACOBI_WIDTH {
            source.push_str(&format!("var x{i}[i in t]: Scalar; "));
        }
        for i in 0..SELECTED_JACOBI_WIDTH {
            let rhs = if i == 0 {
                "-x0[i]".to_owned()
            } else {
                format!("(x0[i] - {}*x{i}[i])", i + 1)
            };
            source.push_str(&format!("eq rate{i}[i in t]: d(x{i}[i])/di == {rhs}/1{{s}}; eq initial{i}: x{i}[0{{s}}] == {}; annotation start x{i}(0); annotation nominal x{i}({}); ",
                usize::from(i == 0), 10usize.pow((i % 3) as u32)));
        }
        if with_time_observable {
            // A passive quantity-valued view permits the existing physical goal
            // owner to bind the otherwise dimensionless relaxation state.
            source.push_str("let relaxation_time[i in t]:Time=x15[i]*1{s}; annotation report relaxation_time(\"dimensionless relaxation state expressed as a time magnitude\"); ");
        }
        source.push_str("} }");
        source
    }
    #[cfg(feature = "solver-idas")]
    fn selected_jacobi_profile() -> native::Profile {
        use pse_model::scalars::PositiveCount;
        native::Profile {
            method: native::Method::Idas,
            end: 0.2,
            samples: vec![0., 0.1, 0.2],
            sensitivity: native::DynamicSensitivity::None,
            rtol: 0.001,
            atol: vec![0.001; SELECTED_JACOBI_WIDTH],
            idas: native::IdasSettings {
                linear: native::IdasLinear::Spgmr {
                    dimension: pse_model::scalar!(PositiveCount(8)),
                    preconditioner: pse_backend_native::solve::Preconditioner::Jacobi,
                },
                ..Default::default()
            },
            ..native::Profile::default()
        }
    }
    #[cfg(feature = "solver-idas")]
    #[tokio::test]
    async fn selected_jacobi_compiler_runtime_native_pipeline_keeps_exact_demand_and_coverage() {
        let runtime = super::super::tests::runtime();
        const WIDTH: usize = SELECTED_JACOBI_WIDTH;
        let source = selected_jacobi_source(false);
        let rows = pse_authoring::language::parse(
            &source,
            SemanticId::from_bytes([175; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, super::super::tests::physical())
            .await
            .unwrap();
        let prepared = package
            .prepare_simulation(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                pse_modeling::Bindings::default(),
                pse_modeling::Limits::default(),
                pse_compiler::workspace::ModelingCaseBindings::default(),
                super::super::tests::compiler_profile(),
                selected_jacobi_profile(),
                pse_kernels::DerivativeOrder::First,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None);
        let mut worker = prepared.worker(runtime.native(), scope).unwrap();
        let rhs = &worker.functions[&(0, Function::Rhs)];
        assert_eq!(
            rhs.program.case.assembly.order(),
            pse_kernels::DerivativeOrder::Value
        );
        assert_eq!(rhs.program.case.assembly.jacobian_entries().len(), WIDTH);
        assert!(rhs.program.case.assembly.has_directional_actions());
        assert!(
            rhs.layout.jacobian.is_none() && rhs.jacobian.is_none(),
            "selected preparation has static support but no full numeric template/snapshot"
        );
        assert!(
            rhs.layout.pairs.len() > WIDTH,
            "off-diagonal structural support remains available"
        );
        let point = vec![0.1; WIDTH];
        let mut diagonal = vec![42.; WIDTH];
        assert!(matches!(
            worker
                .rhs_state_diagonal(0, 0., &point, prepared.parameters(), &mut diagonal)
                .unwrap(),
            native::RhsDiagonalSource::SelectedEntries
        ));
        let mut sorted = diagonal.clone();
        sorted.sort_by(f64::total_cmp);
        assert_eq!(
            sorted,
            (1..=WIDTH).rev().map(|i| -(i as f64)).collect::<Vec<_>>()
        );
        let coverage = worker.functions[&(0, Function::Rhs)].cache.unwrap();
        assert!(coverage.selected && !coverage.values && !coverage.full);
        let value = worker
            .evaluate(0, Function::Rhs, 0., &point, prepared.parameters(), false)
            .unwrap();
        assert!(
            worker
                .evaluate(0, Function::Rhs, 0., &point, prepared.parameters(), true)
                .is_err()
        );
        let coverage = worker.functions[&(0, Function::Rhs)].cache.unwrap();
        assert!(
            !coverage.selected && coverage.values && !coverage.full,
            "failed unprepared full upgrade preserves Value without merging a prior provider context"
        );
        assert_eq!(
            worker
                .evaluate(0, Function::Rhs, 0., &point, prepared.parameters(), false)
                .unwrap()
                .values,
            value.values
        );
        let mut again = vec![43.; WIDTH];
        worker
            .rhs_state_diagonal(0, 0., &point, prepared.parameters(), &mut again)
            .unwrap();
        assert_eq!(again, diagonal);
        assert!(
            worker
                .evaluate(0, Function::Rhs, 0., &point, prepared.parameters(), true)
                .is_err()
        );
        let coverage = worker.functions[&(0, Function::Rhs)].cache.unwrap();
        assert!(
            coverage.selected && !coverage.values && !coverage.full,
            "failed unprepared full upgrade also preserves the fresh selected product"
        );
        assert!(
            worker
                .rhs_state_diagonal(0, f64::NAN, &point, prepared.parameters(), &mut again)
                .is_err()
        );
        assert_eq!(again, diagonal, "guards refuse before cached publication");
        let hub = pse_diagnostics::observation::ObservationHub::new(
            pse_diagnostics::observation::Policy {
                budget_bytes: 64 << 20,
                max_metrics: 256,
                max_children: 128,
                ..Default::default()
            },
        );
        let operation = hub.start("selected Jacobi pipeline");
        let context = operation.context();
        let report = {
            let _entered = context.enter();
            native::integrate(
                &mut worker,
                prepared.profile(),
                prepared.parameters(),
                native::Cancellation::default(),
            )
            .unwrap()
        };
        assert_eq!(
            report.termination,
            native::Termination::Completed,
            "{:?}",
            report.error
        );
        let mut reference_profile = prepared.profile().clone();
        reference_profile.idas.linear = native::IdasLinear::Klu;
        let reference = package
            .prepare_simulation(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                pse_modeling::Bindings::default(),
                pse_modeling::Limits::default(),
                pse_compiler::workspace::ModelingCaseBindings::default(),
                super::super::tests::compiler_profile(),
                reference_profile,
                pse_kernels::DerivativeOrder::First,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let mut reference_worker = reference
            .worker(
                runtime.native(),
                pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            )
            .unwrap();
        assert_eq!(worker.coordinates.state, reference_worker.coordinates.state);
        let reference_report = native::integrate(
            &mut reference_worker,
            reference.profile(),
            reference.parameters(),
            native::Cancellation::default(),
        )
        .unwrap();
        assert_eq!(
            reference_report.termination,
            native::Termination::Completed,
            "{:?}",
            reference_report.error
        );
        assert_eq!(report.samples.len(), reference_report.samples.len());
        let mut original_invalid = 0;
        for (sample, reference_sample) in report.samples.iter().zip(&reference_report.samples) {
            assert_eq!(sample.time, reference_sample.time);
            for (i, coordinate) in worker.coordinates.state.iter().enumerate() {
                let rate = -diagonal[i];
                let expected = if rate == 1. {
                    (-sample.time).exp()
                } else {
                    ((-sample.time).exp() - (-rate * sample.time).exp()) / (rate - 1.)
                };
                let physical = sample.state[i] * coordinate.scale + coordinate.offset;
                let reference_physical =
                    reference_sample.state[i] * coordinate.scale + coordinate.offset;
                let target = super::super::tests::engineering_target(
                    prepared.numerics(),
                    pse_model::generated::enums::NumericalTarget::Variable,
                    coordinate.id,
                );
                let budget = target.engineering.as_ref().unwrap().budget;
                let selected_error = (physical - expected).abs();
                let reference_error = (reference_physical - expected).abs();
                assert_eq!(
                    selected_error <= budget,
                    reference_error <= budget,
                    "rate={rate} t={} selected_error={selected_error} full_klu_error={reference_error} budget={budget}",
                    sample.time
                );
                // This comparison uses the original physical agreement allowance;
                // it does not turn the native local controls into a global bound.
                assert!(
                    (physical - reference_physical).abs() <= budget,
                    "selected and full linear routes disagree beyond the original physical allowance"
                );
                if selected_error > budget {
                    original_invalid += 1;
                    eprintln!(
                        "selected_jacobi_original_invalid rate={rate} time={} selected_error={selected_error} full_klu_error={reference_error} budget={budget}",
                        sample.time
                    );
                }
            }
        }
        assert!(
            original_invalid > 0,
            "the raw native local-tolerance fixture must retain its independently observed physical refusal"
        );
        let mut pending = vec![context.reader()];
        let mut visited = std::collections::HashSet::new();
        let mut snapshots = Vec::new();
        while let Some(reader) = pending.pop() {
            if !visited.insert(reader.id()) {
                continue;
            }
            let snapshot = reader.snapshot(std::time::Duration::from_secs(60));
            for child in &snapshot.children {
                if let Some(reader) = hub.reader(child.id) {
                    pending.push(reader);
                }
            }
            snapshots.push(snapshot);
        }
        // Metrics belong to actual producer instances and linked operations.
        // Neither a root-only lookup nor the first same-named producer is a sum.
        let count = |name: &str| {
            snapshots
                .iter()
                .flat_map(|snapshot| &snapshot.metrics)
                .filter(|metric| metric.name == name)
                .map(|metric| match metric.cumulative {
                    pse_diagnostics::observation::Value::Count(n) => n,
                    ref other => panic!("{name} has no observed cumulative count: {other:?}"),
                })
                .sum::<u64>()
        };
        let native_setups = report
            .statistics
            .iter()
            .filter(|statistics| statistics["backend"] == "idas")
            .map(|statistics| {
                statistics["preconditioner_evaluations"]
                    .as_u64()
                    .expect("each completed IDAS population must expose its native setup count")
            })
            .sum::<u64>();
        let selected_setups = count("idas.jacobi.selected_setups");
        for snapshot in &snapshots {
            if snapshot.id != context.reader().id()
                && !snapshot.metrics.iter().any(|metric| {
                    metric.name.starts_with("idas.jacobi.")
                        || metric.name == "math.selected_entry_calls"
                })
            {
                continue;
            }
            eprintln!(
                "selected_jacobi_observation operation={} kind={} coverage={:?} metrics={} children={} selected={} native_setups={native_setups}",
                snapshot.id,
                snapshot.kind,
                snapshot.coverage,
                snapshot.metrics.len(),
                snapshot.children.len(),
                snapshot
                    .metrics
                    .iter()
                    .filter(|metric| metric.name == "idas.jacobi.selected_setups")
                    .map(|metric| format!("{:?}:{:?}", metric.producer, metric.cumulative))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        assert!(
            selected_setups > 0,
            "actual selected setup emission must be retained; native_setups={native_setups}"
        );
        assert_eq!(
            selected_setups, native_setups,
            "source-owned selected setup observations must cover the actual native setup population"
        );
        // Equality with the independent complete native population is required
        // before an absent full-setup fact can mean zero, even if unrelated child
        // detail has been curtailed. No observed setup can hide in that absence.
        assert_eq!(count("idas.jacobi.full_setups"), 0);
        assert!(count("math.selected_entry_calls") > 0);
        assert!(worker.functions.values().all(|f| f.jacobian.is_none()));
    }
    #[cfg(all(feature = "solver-idas", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn selected_jacobi_physical_goal_refuses_missing_rate_uncertainty_without_promoting_native_completion()
     {
        use pse_model::generated::enums::{
            AccuracyGoalStatus, AccuracyGoalSubject, AccuracyGoalUse, AccuracyObservation,
            AccuracyUnavailableReason, NumericalAccuracyClass, NumericalSource, NumericalTarget,
        };
        let runtime = super::super::tests::runtime();
        let rows = pse_authoring::language::parse(
            &selected_jacobi_source(true),
            SemanticId::from_bytes([175; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, super::super::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let base = package
            .prepare_simulation(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                super::super::tests::compiler_profile(),
                selected_jacobi_profile(),
                pse_kernels::DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        let model = &base.model().compiled().model;
        let observable = model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(".relaxation_time"))
            .unwrap();
        let pse_modeling::Type::Quantity(quantity) = &observable.ty else {
            panic!("time-valued observable");
        };
        let quantity = quantity
            .resolve(&package.quantities, &BTreeMap::new())
            .unwrap();
        let unit = package
            .quantities
            .quantity_type(quantity)
            .unwrap()
            .canonical_unit;
        let expected_time = ((-0.1_f64).exp() - (-1.6_f64).exp()) / 15.;
        let mut profile = base.profile().clone();
        profile.numerics.goals.push(pse_model::engineering_accuracy::AccuracyGoal {
            goal_id: pse_ids::named_id(observable.id, "physical-relaxation-time-resolution").into(),
            model_id: None, case_id: None, instance_id: None, fit_id: None,
            target_id: observable.id, target_kind: NumericalTarget::Observable,
            quantity_id: quantity.as_id(), unit_id: unit.as_id(),
            subject: AccuracyGoalSubject::SelectedOutput, observation: AccuracyObservation::Sample,
            time: Some(0.1), resolution: Some(0.001),
            criterion_lower: Some(expected_time - 0.001), criterion_upper: Some(expected_time + 0.001),
            required_class: NumericalAccuracyClass::Estimated,
            use_policy: AccuracyGoalUse::RequireSatisfied, refine: true,
            source: NumericalSource::Analysis, priority: 0,
            provenance: "Explicit physical time-magnitude resolution; selected First is not arithmetic uncertainty".into(),
        });
        let requested = package
            .prepare_simulation(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                super::super::tests::compiler_profile(),
                profile,
                pse_kernels::DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(requested.profile().rtol, base.profile().rtol);
        assert_eq!(requested.profile().atol, base.profile().atol);
        let program = requested.program();
        assert!(
            program
                .programs
                .iter()
                .all(|role| role.case.assembly.order() == pse_kernels::DerivativeOrder::Value)
        );
        assert!(
            program.modes.iter().any(|mode| !mode.providers.is_empty()),
            "the refusal must exercise actual generated-rate uncertainty, not an absent evidence backend"
        );
        let state = model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(".x15"))
            .unwrap()
            .id;
        let index = program
            .coordinates
            .state
            .iter()
            .position(|coordinate| coordinate.id == state)
            .unwrap();
        let coordinate = &program.coordinates.state[index];
        let budget = super::super::tests::engineering_target(
            requested.numerics(),
            NumericalTarget::Variable,
            state,
        )
        .engineering
        .as_ref()
        .unwrap()
        .budget;
        assert_eq!(budget, 0.001);
        let result = requested
            .run(&runtime.execution_services(), &cancel)
            .await
            .unwrap();
        assert_eq!(
            result.report().termination,
            native::Termination::Completed,
            "{:?}",
            result.diagnostic()
        );
        assert!(
            !result.accepted(),
            "a completed native trajectory has no satisfied physical accuracy evidence"
        );
        let accuracy = &result.completion().accuracy;
        assert_eq!(accuracy.len(), 1);
        assert_eq!(
            accuracy[0].classification.status,
            AccuracyGoalStatus::Unresolved
        );
        assert_eq!(
            accuracy[0].classification.unavailable,
            Some(AccuracyUnavailableReason::EvaluatorUncertainty)
        );
        assert!(
            !result
                .report()
                .progress
                .iter()
                .any(|event| event.phase == "accuracy.dynamic.comparator"),
            "missing arithmetic evidence must not issue a guessed refinement"
        );
        let sample = result
            .report()
            .samples
            .iter()
            .find(|sample| sample.time == 0.1)
            .unwrap();
        let physical = sample.state[index] * coordinate.scale + coordinate.offset;
        let expected = ((-sample.time).exp() - (-16. * sample.time).exp()) / 15.;
        assert!(
            (physical - expected).abs() > budget,
            "the independent original physical check rejects this local-tolerance-only trajectory"
        );
    }
    #[cfg(feature = "solver-idas")]
    #[tokio::test]
    async fn selected_jacobi_coupled_implicit_owner_qualifies_but_opaque_owner_does_not() {
        use pse_model::scalars::PositiveCount;
        let runtime = super::super::tests::runtime();
        let source = "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; var y[i in t]: Time; eq first[i in t]: p*d(x[i])/di+d(y[i])/di == -p*x[i]/1{s}; eq second[i in t]: d(x[i])/di+2*d(y[i])/di == 0; eq ix: x[0{s}] == 1{s}; eq iy: y[0{s}] == 3{s}; annotation nominal x(10{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::from_bytes([175; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, super::super::tests::physical())
            .await
            .unwrap();
        let prepared = package
            .prepare_simulation(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                pse_modeling::Bindings::default(),
                pse_modeling::Limits::default(),
                pse_compiler::workspace::ModelingCaseBindings::default(),
                super::super::tests::compiler_profile(),
                native::Profile {
                    method: native::Method::Idas,
                    end: 0.2,
                    samples: vec![0., 0.1, 0.2],
                    parameter_scales: vec![1.],
                    sensitivity: native::DynamicSensitivity::None,
                    rtol: 0.001,
                    atol: vec![0.001; 2],
                    idas: native::IdasSettings {
                        linear: native::IdasLinear::Spgmr {
                            dimension: pse_model::scalar!(PositiveCount(4)),
                            preconditioner: pse_backend_native::solve::Preconditioner::Jacobi,
                        },
                        ..Default::default()
                    },
                    ..native::Profile::default()
                },
                pse_kernels::DerivativeOrder::First,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let program = prepared.program();
        assert!(program.modes.iter().any(|mode| !mode.providers.is_empty()));
        #[derive(Debug)]
        struct Opaque {
            original: pse_kernels::Registration,
            created: Arc<std::sync::atomic::AtomicUsize>,
        }
        impl pse_kernels::ProviderFactory for Opaque {
            fn spec(&self) -> &pse_kernels::ProviderSpec {
                self.original.spec()
            }
            fn create(&self) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
                self.created
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.original.worker()
            }
            fn create_scoped(
                &self,
                scope: pse_kernels::ExecutionScope,
            ) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
                self.created
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.original.worker_scoped(scope)
            }
        }
        let eligible = crate::math::modeling::selected_entry_provider_eligible;
        for registration in program
            .modes
            .iter()
            .flat_map(|mode| mode.providers.values())
        {
            assert!(
                eligible(registration),
                "actual childless implicit owner qualifies its selected implementation"
            );
            let created = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let opaque = pse_kernels::Registration::bind(
                registration.descriptor(),
                Arc::new(Opaque {
                    original: registration.clone(),
                    created: created.clone(),
                }),
            )
            .unwrap();
            assert_eq!(opaque.spec(), registration.spec());
            assert!(
                !eligible(&opaque),
                "matching admitted descriptor does not qualify an opaque actual factory"
            );
            assert_eq!(
                created.load(std::sync::atomic::Ordering::Relaxed),
                0,
                "classification happens before any factory effect"
            );
        }
        let rhs = program
            .programs
            .iter()
            .find(|p| p.function == Function::Rhs)
            .unwrap();
        assert_eq!(
            rhs.case.assembly.order(),
            pse_kernels::DerivativeOrder::Value
        );
        assert_eq!(rhs.case.assembly.jacobian_entries().len(), 2);
        let x = prepared
            .model()
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        let xi = program
            .coordinates
            .state
            .iter()
            .position(|coordinate| coordinate.id == x)
            .unwrap();
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None);
        let mut worker = prepared.worker(runtime.native(), scope).unwrap();
        let mut diagonal = [42.; 2];
        assert!(matches!(
            worker
                .rhs_state_diagonal(0, 0., &[0.1, 0.1], prepared.parameters(), &mut diagonal)
                .unwrap(),
            native::RhsDiagonalSource::SelectedEntries
        ));
        assert!((diagonal[xi] + 4. / 3.).abs() < 1e-12);
        assert_eq!(diagonal[1 - xi], 0.);
        let report = native::integrate(
            &mut worker,
            prepared.profile(),
            prepared.parameters(),
            native::Cancellation::default(),
        )
        .unwrap();
        assert_eq!(
            report.termination,
            native::Termination::Completed,
            "{:?}",
            report.error
        );
        for sample in &report.samples {
            let exact_x = (-4. * sample.time / 3.).exp();
            for (i, coordinate) in program.coordinates.state.iter().enumerate() {
                let expected = if i == xi {
                    exact_x
                } else {
                    3. + 0.5 * (1. - exact_x)
                };
                let physical = sample.state[i] * coordinate.scale + coordinate.offset;
                let budget = super::super::tests::engineering_target(
                    prepared.numerics(),
                    pse_model::generated::enums::NumericalTarget::Variable,
                    coordinate.id,
                )
                .engineering
                .as_ref()
                .unwrap()
                .budget;
                assert!(
                    (physical - expected).abs() <= budget,
                    "coupled implicit state {i}: {physical} != {expected}, budget={budget}"
                );
            }
        }
    }
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
