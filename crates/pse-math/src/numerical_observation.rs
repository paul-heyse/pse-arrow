// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physically bound execution-only observations over checked numerical templates.
use crate::{
    AllocationOwner, ExecutionAttribution, ExecutionAttributionSnapshot, MathError,
    assembly::AssemblyLimits,
    binding::{
        BindingTransport, CaseLimits, CaseValues, Contribution, EvalBindings, Row,
        ScalarInputInventory, SlotBinding, Target, Variable,
    },
    guarded::Evaluation,
    index::{GlobalCol, GlobalRow},
    transfer::{CheckedExecution, RestoredExecutionTemplate, TransferError},
};
use pse_ids::SemanticId;
use pse_kernels::{DerivativeOrder, ExecutionScope, Port};
use pse_quantity::QuantityRegistry;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
#[path = "numerical_observation_batch.rs"]
mod batch;
#[cfg(test)]
#[path = "numerical_observation_tests.rs"]
mod tests;
pub use batch::NumericalObservationBatchWorker;

/// Passive original physical transport and diagnostic context for one instance.
/// This grants neither symbolic authority nor numerical execution capability.
#[derive(Clone, Debug)]
pub struct NumericalObservationInstance {
    attribution: ExecutionAttribution,
    slots: Arc<Vec<SlotBinding>>,
    contributions: Vec<Contribution>,
    outputs: Vec<usize>,
    coordinates: Vec<usize>,
}
impl NumericalObservationInstance {
    /// Bind exact original slots, selected outputs/axes and original source maps.
    pub fn new(
        attribution: ExecutionAttributionSnapshot,
        slots: Vec<SlotBinding>,
        contributions: Vec<Contribution>,
        outputs: Vec<usize>,
        coordinates: Vec<usize>,
    ) -> Result<Self, MathError> {
        attribution
            .validate_shape()
            .map_err(|cause| MathError::Contract(cause.to_string()))?;
        if outputs.iter().collect::<BTreeSet<_>>().len() != outputs.len()
            || coordinates.iter().collect::<BTreeSet<_>>().len() != coordinates.len()
            || coordinates.iter().any(|&axis| axis >= slots.len())
            || contributions
                .iter()
                .any(|part| !outputs.contains(&part.output))
        {
            return Err(MathError::Contract(
                "invalid numerical observation selection".into(),
            ));
        }
        Ok(Self {
            attribution: attribution.into_attribution(),
            slots: Arc::new(slots),
            contributions,
            outputs,
            coordinates,
        })
    }
    /// Original instance-qualified diagnostic context, without a source-bearing body.
    pub fn attribution(&self) -> &ExecutionAttribution {
        &self.attribution
    }
    /// Complete original formal inventory, including control-only and unused slots.
    pub fn slots(&self) -> &[SlotBinding] {
        &self.slots
    }
    /// Original selected output ordinals in numerical result order.
    pub fn outputs(&self) -> &[usize] {
        &self.outputs
    }
    /// Original formal derivative axes in requested order.
    pub fn coordinates(&self) -> &[usize] {
        &self.coordinates
    }
    /// Exact sparse output contributions; aliases and repeated addends remain explicit.
    pub fn contributions(&self) -> &[Contribution] {
        &self.contributions
    }
    /// Conservative owned passive metadata extent, including original attribution.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.attribution.retained_bytes()
            + self.slots.capacity() * size_of::<SlotBinding>()
            + self.contributions.capacity() * size_of::<Contribution>()
            + (self.outputs.capacity() + self.coordinates.capacity()) * size_of::<usize>()
    }
}

/// Compiler-supplied passive observation binding. Contains no mathematical body,
/// source witness, symbolic support or executable program.
#[derive(Clone, Debug)]
pub struct NumericalObservationCase {
    inputs: Arc<ScalarInputInventory>,
    rows: Vec<Row>,
    coordinates: Vec<SemanticId>,
    instances: Vec<NumericalObservationInstance>,
    order: DerivativeOrder,
    limits: AssemblyLimits,
    owner: Option<Arc<dyn AllocationOwner>>,
}
impl NumericalObservationCase {
    /// Admit original scalar inventory and physical sparse binding independently
    /// of executable instructions. Rows and derivative coordinates retain caller order.
    #[expect(
        clippy::too_many_arguments,
        reason = "the passive case binds separate original scalar, output, derivative, instance and finite policy owners"
    )]
    pub fn new(
        variables: Vec<Variable>,
        parameters: Vec<Port>,
        rows: Vec<Row>,
        coordinates: Vec<SemanticId>,
        instances: Vec<NumericalObservationInstance>,
        order: DerivativeOrder,
        registry: &QuantityRegistry,
        limits: AssemblyLimits,
    ) -> Result<Self, MathError> {
        let bounds = CaseLimits::default();
        let inputs = Arc::new(ScalarInputInventory::new(
            variables,
            parameters,
            bounds.scalars,
        )?);
        if limits.contributions == 0
            || limits.native_index == 0
            || limits.worker_bytes == 0
            || instances.len() > bounds.instances
            || rows.len() > bounds.rows
            || rows.iter().map(|row| row.id).collect::<BTreeSet<_>>().len() != rows.len()
            || coordinates.iter().collect::<BTreeSet<_>>().len() != coordinates.len()
            || coordinates.iter().any(|&id| !inputs.contains(id))
            || instances
                .iter()
                .map(|instance| instance.attribution.instance())
                .collect::<BTreeSet<_>>()
                .len()
                != instances.len()
        {
            return Err(MathError::Contract(
                "invalid numerical observation inventory".into(),
            ));
        }
        for row in &rows {
            registry.quantity_type(row.quantity)?;
            if row.lower.is_nan()
                || row.upper.is_nan()
                || row.lower > row.upper
                || row.lower == f64::INFINITY
                || row.upper == f64::NEG_INFINITY
            {
                return Err(MathError::Contract(
                    "invalid numerical observation row".into(),
                ));
            }
        }
        let row_ids = rows.iter().map(|row| row.id).collect::<BTreeSet<_>>();
        let mut addends = 0usize;
        let mut slots = 0usize;
        for instance in &instances {
            inputs.validate_slots(&instance.slots)?;
            slots = slots
                .checked_add(instance.slots.len())
                .ok_or(MathError::Limit("observation slots"))?;
            addends = addends
                .checked_add(instance.contributions.len())
                .ok_or(MathError::Limit("observation contributions"))?;
            if slots > bounds.slots
                || addends > limits.contributions
                || instance.contributions.is_empty()
            {
                return Err(MathError::Limit("observation slots or contributions"));
            }
            for slot in instance.slots.iter() {
                if slot.readmit(slot.quantity(), registry)? != *slot {
                    return Err(MathError::Contract(
                        "changed observation input conversion".into(),
                    ));
                }
            }
            for part in &instance.contributions {
                if !part.scale.is_finite()
                    || part.scale == 0.
                    || !matches!(part.target, Target::Row(id) if row_ids.contains(&id))
                {
                    return Err(MathError::Contract(
                        "invalid numerical observation contribution".into(),
                    ));
                }
            }
            let expected = coordinates
                .iter()
                .flat_map(|id| {
                    instance
                        .slots
                        .iter()
                        .enumerate()
                        .filter_map(move |(formal, slot)| (slot.source() == *id).then_some(formal))
                })
                .collect::<Vec<_>>();
            if instance.coordinates != expected {
                return Err(MathError::Contract(
                    "observation formal axes differ from requested coordinates".into(),
                ));
            }
        }
        Ok(Self {
            inputs,
            rows,
            coordinates,
            instances,
            order,
            limits,
            owner: None,
        })
    }
    /// Original scalar input authority shared with ordinary case validation.
    pub fn inputs(&self) -> &ScalarInputInventory {
        &self.inputs
    }
    /// Original observation rows in requested order.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    /// Global derivative coordinates in requested order.
    pub fn coordinates(&self) -> &[SemanticId] {
        &self.coordinates
    }
    /// Original instances in producer order, independently of shared body identity.
    pub fn instances(&self) -> &[NumericalObservationInstance] {
        &self.instances
    }
    /// Explicit requested derivative ceiling; stronger execution is never inferred.
    pub fn order(&self) -> DerivativeOrder {
        self.order
    }
    /// Original finite assembly policy; this grants no receiving resource capacity.
    pub fn limits(&self) -> AssemblyLimits {
        self.limits
    }
    /// Retain the existing immutable metadata allocation owner on escaping bindings.
    /// If shared, the supplied owner must also cover the native vector copies.
    pub fn with_owner(mut self: Arc<Self>, owner: Arc<dyn AllocationOwner>) -> Arc<Self> {
        let value = Arc::make_mut(&mut self);
        value.owner = Some(crate::retain_allocation_owner(value.owner.take(), owner));
        self
    }
    /// Conservative owned passive metadata extent, with complete original inputs.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.inputs.retained_bytes()
            + self.rows.capacity() * size_of::<Row>()
            + self.coordinates.capacity() * size_of::<SemanticId>()
            + self
                .instances
                .iter()
                .map(NumericalObservationInstance::retained_bytes)
                .sum::<usize>()
    }

    /// Read-only correspondence validation; this copies no source maps, constructs
    /// no operation worker and grants no persisted scientific source authority.
    pub fn validate_templates(
        &self,
        templates: &[Arc<RestoredExecutionTemplate>],
    ) -> Result<(), MathError> {
        if templates.len() != self.instances.len() {
            return Err(MathError::Contract(
                "observation template population".into(),
            ));
        }
        for (instance, template) in self.instances.iter().zip(templates) {
            if template.order() != self.order
                || template.output_count() != instance.outputs.len()
                || template
                    .input_formals()
                    .iter()
                    .any(|&formal| formal >= instance.slots.len())
                || template.coordinate_offsets().len() != instance.coordinates.len()
                || template
                    .coordinate_offsets()
                    .iter()
                    .zip(&instance.coordinates)
                    .any(|(&offset, &formal)| template.input_formals().get(offset) != Some(&formal))
            {
                return Err(MathError::Contract(
                    "observation template request correspondence".into(),
                ));
            }
            template
                .check_closure_attribution_bindings(&instance.attribution)
                .map_err(|cause| MathError::Contract(cause.to_string()))?;
        }
        Ok(())
    }
    /// Source-backed constructor allowance, computed without allocating. Includes
    /// map/collect/sort scratch, incoming template-pointer overlap and retained
    /// compact transport/gathers; template/binding payloads already have owners.
    pub fn assembly_allocation_bytes(
        &self,
        templates: &[Arc<RestoredExecutionTemplate>],
    ) -> Result<usize, MathError> {
        fn add(left: usize, right: usize) -> Result<usize, MathError> {
            left.checked_add(right)
                .ok_or(MathError::Limit("observation assembly allocation"))
        }
        fn array<T>(count: usize) -> Result<usize, MathError> {
            std::alloc::Layout::array::<T>(count)
                .map(|layout| layout.size())
                .map_err(|_| MathError::Limit("observation assembly allocation"))
        }
        fn vector<T>(count: usize) -> Result<usize, MathError> {
            // Filter collectors can grow geometrically. 2*n plus the native
            // minimum of four slots covers requested Vec backing, not RSS.
            let capacity = count
                .checked_mul(2)
                .and_then(|n| n.checked_add(4))
                .ok_or(MathError::Limit("observation vector allocation"))?;
            array::<T>(capacity)
        }
        fn map<K, V>(count: usize) -> Result<usize, MathError> {
            // Pinned alloc BTreeMap/Set::from_iter collects then stable-sorts.
            // Keep both the geometric collect buffer and a full-length sorting
            // buffer beside the shared source-backed native node allowance.
            let nodes = crate::execution::btree_allocation_bytes::<K, V>(count)
                .map_err(|_| MathError::Limit("observation map allocation"))?;
            add(
                add(nodes, vector::<(K, V)>(count)?)?,
                add(array::<(K, V)>(count)?, size_of::<BTreeMap<K, V>>())?,
            )
        }
        let count = self.instances.len();
        let scalars = self
            .inputs
            .variables()
            .len()
            .checked_add(self.inputs.parameters().len())
            .ok_or(MathError::Limit("observation scalar extent"))?;
        let fixed = scalars
            .checked_sub(self.coordinates.len())
            .ok_or(MathError::Limit("observation coordinate extent"))?;
        let mut bytes = add(
            size_of::<NumericalObservationAssembly>() + 2 * size_of::<usize>(),
            array::<ObservationProgram>(count)?,
        )?;
        // Immediate owner attachment retains a case clone because the compiler
        // request still owns its original case Arc. Only these native vectors
        // are copied; input declarations, slots and source maps keep their owners.
        bytes = add(bytes, size_of::<Self>() + 2 * size_of::<usize>())?;
        bytes = add(
            bytes,
            add(
                array::<Row>(self.rows.len())?,
                array::<SemanticId>(self.coordinates.len())?,
            )?,
        )?;
        bytes = add(bytes, array::<NumericalObservationInstance>(count)?)?;
        bytes = add(
            bytes,
            array::<Arc<RestoredExecutionTemplate>>(templates.len())?,
        )?;
        for extent in [
            map::<SemanticId, GlobalCol>(self.coordinates.len())?,
            map::<SemanticId, GlobalRow>(self.rows.len())?,
            map::<SemanticId, usize>(fixed)?,
            map::<SemanticId, usize>(self.coordinates.len())?,
        ] {
            bytes = add(bytes, extent)?;
        }
        for (instance, template) in self.instances.iter().zip(templates) {
            let axes = instance
                .slots
                .iter()
                .filter(|slot| self.coordinates.contains(&slot.source()))
                .count();
            let inputs = template.input_formals().len();
            bytes = add(bytes, array::<Contribution>(instance.contributions.len())?)?;
            bytes = add(bytes, array::<Contribution>(instance.contributions.len())?)?;
            bytes = add(
                bytes,
                array::<usize>(
                    instance
                        .outputs
                        .len()
                        .checked_add(instance.coordinates.len())
                        .ok_or(MathError::Limit("observation case metadata"))?,
                )?,
            )?;
            bytes = add(bytes, vector::<crate::binding::TransportAxis>(axes)?)?;
            bytes = add(
                bytes,
                vector::<crate::binding::TransportOutput>(instance.contributions.len())?,
            )?;
            bytes = add(bytes, vector::<(usize, f64)>(instance.coordinates.len())?)?;
            bytes = add(
                bytes,
                EvalBindings::allocation_bound(inputs)
                    .ok_or(MathError::Limit("observation gather allocation"))?,
            )?;
            bytes = add(bytes, 2 * size_of::<usize>() + size_of::<Vec<usize>>())?;
            bytes = add(
                bytes,
                add(map::<&usize, ()>(inputs)?, map::<&usize, ()>(inputs)?)?,
            )?;
        }
        Ok(bytes)
    }

    /// Associate already checked immutable programs with this passive binding.
    /// The caller must establish protected scientific custody before this numerical check.
    pub fn assemble(
        self: &Arc<Self>,
        templates: Vec<Arc<RestoredExecutionTemplate>>,
    ) -> Result<Arc<NumericalObservationAssembly>, MathError> {
        self.validate_templates(&templates)?;
        let columns = self
            .coordinates
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, GlobalCol::new(i)))
            .collect();
        let rows = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, row)| (row.id, GlobalRow::new(i)))
            .collect();
        let fixed = self
            .inputs
            .variables()
            .iter()
            .map(|variable| variable.port.id)
            .chain(self.inputs.parameters().iter().map(|port| port.id))
            .filter(|id| !self.coordinates.contains(id))
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect();
        let gather_coordinates = self
            .coordinates
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();
        let mut prepared = Vec::with_capacity(templates.len());
        let mut worker_bytes = size_of::<NumericalObservationWorker>();
        for (instance, template) in self.instances.iter().zip(templates) {
            let mut mapped = instance.contributions.clone();
            for part in &mut mapped {
                part.output = instance
                    .outputs
                    .iter()
                    .position(|&output| output == part.output)
                    .ok_or_else(|| {
                        MathError::Contract("observation output correspondence".into())
                    })?;
            }
            let transport =
                BindingTransport::from_parts(Arc::clone(&instance.slots), &mapped, &columns, &rows);
            let gather = EvalBindings::new(
                &transport,
                template.input_formals(),
                &gather_coordinates,
                &fixed,
            )?;
            let axes = instance
                .coordinates
                .iter()
                .map(|&formal| {
                    transport
                        .axes
                        .iter()
                        .find(|axis| axis.formal.get() == formal)
                        .map(|axis| (axis.destination.get(), axis.scale))
                        .ok_or_else(|| MathError::Contract("observation derivative axis".into()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            worker_bytes = worker_bytes
                .checked_add(template.worker_bytes())
                .and_then(|bytes| bytes.checked_add(template.extents().evaluation_bytes))
                .and_then(|bytes| {
                    bytes.checked_add(
                        size_of::<CheckedExecution>()
                            + 2 * size_of::<Vec<f64>>()
                            + gather.len() * size_of::<f64>(),
                    )
                })
                .ok_or(MathError::Limit("observation worker backing"))?;
            prepared.push(ObservationProgram {
                template,
                transport,
                gather,
                axes,
            });
        }
        let components = self
            .coordinates
            .len()
            .checked_mul(self.coordinates.len())
            .and_then(|second| {
                1usize
                    .checked_add(if self.order >= DerivativeOrder::First {
                        self.coordinates.len()
                    } else {
                        0
                    })
                    .and_then(|first| {
                        first.checked_add(if self.order >= DerivativeOrder::Second {
                            second
                        } else {
                            0
                        })
                    })
            })
            .and_then(|components| components.checked_mul(self.rows.len()))
            .ok_or(MathError::Limit("observation result extent"))?;
        worker_bytes = worker_bytes
            .checked_add(
                components
                    .checked_mul(size_of::<f64>())
                    .ok_or(MathError::Limit("observation result backing"))?,
            )
            .ok_or(MathError::Limit("observation worker backing"))?;
        if worker_bytes > self.limits.worker_bytes || components > self.limits.native_index {
            return Err(MathError::Limit("observation worker backing"));
        }
        Ok(Arc::new(NumericalObservationAssembly {
            binding: Arc::clone(self),
            programs: prepared,
            worker_bytes,
            result_components: components,
            owner: None,
        }))
    }
}

#[derive(Clone, Debug)]
struct ObservationProgram {
    template: Arc<RestoredExecutionTemplate>,
    transport: BindingTransport,
    gather: EvalBindings,
    axes: Vec<(usize, f64)>,
}
/// Immutable execution-only observation assembly. No symbolic owner is retained.
#[derive(Clone, Debug)]
pub struct NumericalObservationAssembly {
    binding: Arc<NumericalObservationCase>,
    programs: Vec<ObservationProgram>,
    worker_bytes: usize,
    result_components: usize,
    owner: Option<Arc<dyn AllocationOwner>>,
}
impl NumericalObservationAssembly {
    /// Attach the original request/assembly metadata leases before publication.
    /// Workers and independently escaped bindings retain those same anchors.
    pub fn with_owner(mut self: Arc<Self>, owner: Arc<dyn AllocationOwner>) -> Arc<Self> {
        let value = Arc::make_mut(&mut self);
        value.binding = Arc::clone(&value.binding).with_owner(Arc::clone(&owner));
        value.owner = Some(crate::retain_allocation_owner(value.owner.take(), owner));
        self
    }
    /// Native retained assembly metadata, excluding already-owned numerical
    /// templates and shared original input/slot/source payloads.
    pub fn retained_bytes(&self) -> usize {
        let case = &self.binding;
        size_of::<Self>()
            + 2 * size_of::<usize>()
            + self.programs.capacity() * size_of::<ObservationProgram>()
            + size_of::<NumericalObservationCase>()
            + case.rows.capacity() * size_of::<Row>()
            + case.coordinates.capacity() * size_of::<SemanticId>()
            + case.instances.capacity() * size_of::<NumericalObservationInstance>()
            + case
                .instances
                .iter()
                .map(|instance| {
                    instance.contributions.capacity() * size_of::<Contribution>()
                        + (instance.outputs.capacity() + instance.coordinates.capacity())
                            * size_of::<usize>()
                })
                .sum::<usize>()
            + self
                .programs
                .iter()
                .map(|program| {
                    program.transport.retained_bytes()
                        + program.gather.retained_bytes()
                        + program.axes.capacity() * size_of::<(usize, f64)>()
                })
                .sum::<usize>()
    }
    /// Passive original binding and ordered coordinates.
    pub fn binding(&self) -> &Arc<NumericalObservationCase> {
        &self.binding
    }
    /// Complete worker/result backing bound to admit for each independent attempt.
    pub fn worker_bytes(&self) -> usize {
        self.worker_bytes
    }
    /// Create private numerical state under a fresh scope and one real backing owner.
    pub fn worker_scoped<E: std::fmt::Debug + std::fmt::Display>(
        self: &Arc<Self>,
        scope: ExecutionScope,
        admit: impl FnOnce(usize) -> Result<Arc<dyn AllocationOwner>, E>,
    ) -> Result<NumericalObservationWorker, TransferError<E>> {
        scope.check().map_err(crate::error::scope_error)?;
        let owner = admit(self.worker_bytes).map_err(TransferError::Admission)?;
        scope.check().map_err(crate::error::scope_error)?;
        let mut workers = Vec::with_capacity(self.programs.len());
        let mut inputs = Vec::with_capacity(self.programs.len());
        for program in &self.programs {
            workers.push(
                program
                    .template
                    .worker_scoped(scope.clone(), |_| Ok::<_, E>(Arc::clone(&owner)))?,
            );
            inputs.push(vec![0.; program.gather.len()]);
        }
        let rows = self.binding.rows.len();
        let columns = self.binding.coordinates.len();
        let result = Evaluation {
            applicability: Vec::new(),
            values: vec![0.; rows],
            jacobian: vec![
                0.;
                if self.binding.order >= DerivativeOrder::First {
                    rows * columns
                } else {
                    0
                }
            ],
            hessians: vec![
                0.;
                if self.binding.order >= DerivativeOrder::Second {
                    rows * columns * columns
                } else {
                    0
                }
            ],
        };
        Ok(NumericalObservationWorker {
            assembly: Arc::clone(self),
            workers,
            inputs,
            result,
            scope,
            _owner: owner,
        })
    }
}
/// One fresh operation's mutable numerical workers and compact gather scratch.
#[derive(Debug)]
pub struct NumericalObservationWorker {
    assembly: Arc<NumericalObservationAssembly>,
    workers: Vec<CheckedExecution>,
    inputs: Vec<Vec<f64>>,
    result: Evaluation,
    scope: ExecutionScope,
    _owner: Arc<dyn AllocationOwner>,
}
impl NumericalObservationWorker {
    /// Evaluate the explicit admitted order, accumulating sparse physical aliases.
    /// Affine offsets enter only values; derivative transport uses only scales.
    pub fn evaluate(&mut self, values: &CaseValues) -> Result<&Evaluation, MathError> {
        self.scope.check().map_err(crate::error::scope_error)?;
        let case = &self.assembly.binding;
        case.inputs.validate_values(values)?;
        let columns = case.coordinates.len();
        let result = &mut self.result;
        result.applicability.clear();
        result.values.fill(0.);
        result.jacobian.fill(0.);
        result.hessians.fill(0.);
        for (i, program) in self.assembly.programs.iter().enumerate() {
            let attribution = &case.instances[i].attribution;
            program
                .gather
                .fill(Some(values), &[], &[], &mut self.inputs[i])
                .map_err(|cause| attribution.evaluation_error(cause))?;
            let local = self.workers[i]
                .evaluate(&self.inputs[i], case.order)
                .map_err(|cause| attribution.evaluation_error(cause))?;
            scatter(
                program,
                &local,
                result,
                columns,
                case.order,
                attribution.instance(),
            )
            .map_err(|cause| attribution.evaluation_error(cause))?;
        }
        if result
            .values
            .iter()
            .chain(&result.jacobian)
            .chain(&result.hessians)
            .any(|value| !value.is_finite())
        {
            return Err(MathError::Contract(
                "nonfinite assembled observation result".into(),
            ));
        }
        self.scope.check().map_err(crate::error::scope_error)?;
        Ok(result)
    }
}

fn scatter(
    program: &ObservationProgram,
    local: &Evaluation,
    result: &mut Evaluation,
    columns: usize,
    order: DerivativeOrder,
    instance: SemanticId,
) -> Result<(), MathError> {
    for mut evidence in local.applicability.iter().cloned() {
        evidence.instance = Some(instance);
        if !result.applicability.contains(&evidence) {
            result.applicability.push(evidence);
        }
    }
    let width = program.axes.len();
    for contribution in &program.transport.outputs {
        let row = contribution
            .destination
            .ok_or_else(|| MathError::Contract("observation has objective contribution".into()))?
            .get();
        let output = contribution.output;
        result.values[row] += contribution.scale * local.values[output];
        if order >= DerivativeOrder::First {
            for (axis, &(column, scale)) in program.axes.iter().enumerate() {
                result.jacobian[row * columns + column] +=
                    contribution.scale * scale * local.jacobian[output * width + axis];
            }
        }
        if order >= DerivativeOrder::Second {
            for (a, &(column_a, scale_a)) in program.axes.iter().enumerate() {
                for (b, &(column_b, scale_b)) in program.axes.iter().enumerate() {
                    result.hessians[(row * columns + column_a) * columns + column_b] += contribution
                        .scale
                        * scale_a
                        * scale_b
                        * local.hessians[(output * width + a) * width + b];
                }
            }
        }
    }
    if result
        .values
        .iter()
        .chain(&result.jacobian)
        .chain(&result.hessians)
        .any(|value| !value.is_finite())
    {
        return Err(MathError::Contract(
            "nonfinite assembled observation result".into(),
        ));
    }
    Ok(())
}
