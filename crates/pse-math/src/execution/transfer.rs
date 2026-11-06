// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Passive numerical-template correspondence. This is not model/source authority.
//!
//! Only the supplier's checked f64 interpreter is restored. Root identity, original
//! attribution, unknown/CSC correspondence, providers and attempt authority belong
//! to the enclosing root/runtime owner. Requested backing is not allocator RSS.
use super::*;
use crate::{
    AllocationOwner,
    construction::ConstructionAdmission,
    jets::{JetLayoutSnapshot, JetLayoutView},
};
use std::{collections::TryReserveError, convert::Infallible};
pub use symbolica::evaluate::F64TransferError;
use symbolica::evaluate::{F64ProgramExtents, F64ProgramView, F64RestoreError};

/// Native control nesting supported by this transfer profile. A wire codec must
/// separately admit the generated message nesting; this is not a codec allowance.
pub const MAX_STAGE_DEPTH: usize = 512;

macro_rules! passive_record {
    ($(#[$attr:meta])* $name:ident, $projection:literal, { $($(#[$field_attr:meta])* $field:ident: $ty:ty => $tag:literal $({ $($rule:expr),* $(,)? })?),* $(,)? }) => {
        pse_contract::projection_struct! {
            $(#[$attr])*
            #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            pub struct $name {
                projection: { name: $projection, rust_path: crate::transfer::$name, version: 1,
                    direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
                $($(#[$field_attr])* pub $field: $ty => { tag: $tag, direction: pse_contract::boundary_projection::Direction::Both, rules: [$($($rule),*)?] };)*
            }
        }
    }
}
passive_record! {
    /// Fixed nondirectional SelectedPartials Value+First numerical snapshot.
    /// Every field is passive data; no serialized capacity or permission is accepted.
    ExecutionSnapshot, "MathExecutionSnapshot", {
        /// Fixed numerical execution profile; this is not a wire allowance.
        version: u32 => 1 { pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: 1 } },
        /// Compact input population, separate from authored formal IDs.
        inputs: usize => 2,
        /// Authored formal input IDs in their compact numerical order.
        input_formals: Vec<usize> => 3,
        /// Compact input offsets, in requested derivative-axis order.
        coordinate_offsets: Vec<usize> => 4,
        /// Logical scalar slots; sparse frame storage has a separate derived extent.
        slots: usize => 5,
        /// Logical output slots in selected output order, including aliases.
        outputs: Vec<usize> => 6,
        /// Original mathematical limits; actual native admission is separately required.
        limits: EvaluationLimits => 7 { pse_contract::boundary_projection::LocalRule::Required },
        /// Complete original Value coefficient and sparse slot layout.
        value_layout: JetLayoutSnapshot => 8 { pse_contract::boundary_projection::LocalRule::Required },
        /// Complete original First coefficient and sparse slot layout.
        first_layout: JetLayoutSnapshot => 9 { pse_contract::boundary_projection::LocalRule::Required },
        /// Contiguous half-open ranges into first_axes, one per selected output.
        first_rows: Vec<(usize, usize)> => 10,
        /// Sorted derivative-axis ordinals within each output's first_rows range.
        first_axes: Vec<usize> => 11,
        /// Original Value stages; an empty program is valid for initial-input outputs.
        value: Vec<StageSnapshot> => 12,
        /// Original First stages; an empty program retains its layout and seed contract.
        first: Vec<StageSnapshot> => 13
    }
}
passive_record! {
    /// Exact supplier program and native gather/scatter/check metadata; no warm values.
    BlockSnapshot, "MathBlockSnapshot", {
        /// Exact supplier fixed-profile bytes, including original constant bits.
        program: Vec<u8> => 1,
        /// Original qualified scratch-reset capability; false refuses transfer.
        resettable: bool => 2,
        /// Ordered global coefficient components used by this original block.
        components: Vec<usize> => 3,
        /// Logical scalar input slots, separate from gather's frame offsets.
        inputs: Vec<usize> => 4,
        /// Logical scalar output slots, separate from scatter's frame offsets.
        outputs: Vec<usize> => 5,
        /// Ordered (supplier argument offset, enclosing frame storage offset) writes.
        gather: Vec<(usize, usize)> => 6,
        /// Ordered (frame storage offset, optional supplier result offset) writes; None writes zero.
        scatter: Vec<(usize, Option<usize>)> => 7,
        /// Fresh argument population; structural absences start zero, gathered positions NaN.
        arguments: usize => 8,
        /// Fresh supplier result population, initialized to NaN.
        values: usize => 9,
        /// Original arithmetic failure occurrence.
        source: SemanticId => 10,
        /// Original logical source-check ranges in output order.
        checks: Vec<CheckSnapshot> => 11,
        /// Corresponding exclusive supplier-result ends, preserving materialized checks.
        checked_ends: Vec<usize> => 12
    }
}
passive_record! {
    /// Ordered original arithmetic source range.
    CheckSnapshot, "MathSourceCheckSnapshot", {
        /// Exclusive logical block-output end for this original source occurrence.
        end_output: usize => 1,
        /// Original authored occurrence for the covered arithmetic outputs.
        source: SemanticId => 2
    }
}
passive_record! {
    /// Value-only nested mathematical domain program and its original lineage.
    DomainSnapshot, "MathDomainSnapshot", {
        /// Complete original nested value program.
        stages: Vec<StageSnapshot> => 1,
        /// Fresh nested f64 frame population, initialized to NaN.
        frame: usize => 2,
        /// Parent logical slots copied into the nested frame, in storage order.
        reads: Vec<usize> => 3,
        /// Original local Value layout.
        layout: JetLayoutSnapshot => 4 { pse_contract::boundary_projection::LocalRule::Required },
        /// Local logical predicate slot.
        argument: usize => 5,
        /// Parent logical completion-token slot.
        token: usize => 6,
        /// Complete original mathematical validity attribution.
        lineage: pse_model::diagnostic::ValidityLineage => 7 { pse_contract::boundary_projection::LocalRule::Required }
    }
}
passive_record! {
    /// Complete existing applicability plan and its value-only numerical capture.
    ApplicabilitySnapshot, "MathApplicabilitySnapshot", {
        /// Complete original nested value program.
        stages: Vec<StageSnapshot> => 1,
        /// Fresh nested f64 frame population, initialized to NaN.
        frame: usize => 2,
        /// Parent logical slots copied into the nested frame, in storage order.
        reads: Vec<usize> => 3,
        /// Original local Value layout.
        layout: JetLayoutSnapshot => 4 { pse_contract::boundary_projection::LocalRule::Required },
        /// Local logical predicate slots in plan-predicate order.
        predicates: Vec<usize> => 5,
        /// Local logical input slots in plan-capture order.
        inputs: Vec<usize> => 6,
        /// Parent logical completion-token slot.
        token: usize => 7,
        /// Complete existing passive plan, including every alternative and permission.
        plan: pse_model::applicability::Node => 8 { pse_contract::boundary_projection::LocalRule::Required }
    }
}
pse_contract::projection_enum! {
/// Source-owned control stages around supplier arithmetic. Decoding is passive.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StageSnapshot {
    projection: { name: "MathStageSnapshot", rust_path: crate::transfer::StageSnapshot, version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact supplier arithmetic and native indexing metadata.
    Block(BlockSnapshot => { rules: [pse_contract::boundary_projection::LocalRule::Required] }) => 1;
    /// Original ordered mathematical requirement.
    Require {
        /// Logical scalar slot already computed by the enclosing program.
        argument: usize => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Existing finite real-domain condition.
        condition: Condition => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Minimum derivative order at which the requirement applies.
        order: DerivativeOrder => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original authored failure occurrence.
        source: SemanticId => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original mathematical validity attribution, when present.
        lineage: Option<pse_model::diagnostic::ValidityLineage> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
    /// Nested value-only mathematical domain.
    Domain(DomainSnapshot => { rules: [pse_contract::boundary_projection::LocalRule::Required] }) => 3;
    /// Nested value-only evidence plan and original attribution.
    Applicability(ApplicabilitySnapshot => { rules: [pse_contract::boundary_projection::LocalRule::Required] }) => 4;
    /// Original finite comparison and both independently retained alternatives.
    Branch {
        /// Require strict separation from the comparison boundary for this derivative profile.
        require_separation: bool => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Existing comparison semantics.
        comparison: Comparison => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Logical scalar left operand.
        left: usize => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Logical scalar right operand.
        right: usize => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original selected comparison arm.
        then: Vec<StageSnapshot> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original alternative comparison arm.
        otherwise: Vec<StageSnapshot> => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 5;
}
}

/// Structural/profile, supplier, real allocation, scope and original admission causes.
#[derive(Debug, thiserror::Error)]
pub enum TransferError<E: std::fmt::Debug + std::fmt::Display> {
    /// Well-defined execution state outside the fixed transferable capability.
    #[error("unsupported numerical snapshot profile: {0}")]
    Unsupported(&'static str),
    /// Malformed passive structure or inconsistent layout/signature metadata.
    #[error("numerical snapshot: {0}")]
    Invalid(&'static str),
    /// Unrepresentable checked population, layout or backing extent.
    #[error("numerical snapshot extent overflow: {0}")]
    Overflow(&'static str),
    /// Original supplier structural or dataflow refusal.
    #[error("supplier evaluator transfer: {0}")]
    Supplier(#[from] F64TransferError),
    /// Original allocation-free layout refusal or mathematical layout limit.
    #[error("numerical layout transfer: {0}")]
    Layout(#[source] crate::jets::LayoutSnapshotError),
    /// Original native mathematical limit or scope refusal, retaining its class.
    #[error("{0}")]
    Math(#[from] MathError),
    /// Actual fallible reservation failure.
    #[error("numerical snapshot reservation: {0}")]
    Allocation(#[source] TryReserveError),
    /// Unchanged original admission/cancellation error and source chain.
    #[error("numerical snapshot admission: {0}")]
    Admission(#[source] E),
}
impl<E: std::fmt::Debug + std::fmt::Display> From<crate::jets::LayoutSnapshotError>
    for TransferError<E>
{
    fn from(error: crate::jets::LayoutSnapshotError) -> Self {
        match error {
            crate::jets::LayoutSnapshotError::Unsupported(reason) => Self::Unsupported(reason),
            crate::jets::LayoutSnapshotError::Math(error) => Self::Math(error),
            error => Self::Layout(error),
        }
    }
}
impl<E: std::fmt::Debug + std::fmt::Display + std::error::Error + 'static> miette::Diagnostic
    for TransferError<E>
{
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            Self::Math(error) | Self::Layout(crate::jets::LayoutSnapshotError::Math(error)) => {
                miette::Diagnostic::code(error)
            }
            _ => Some(Box::new("math.transfer")),
        }
    }
    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        match self {
            Self::Math(error) | Self::Layout(crate::jets::LayoutSnapshotError::Math(error)) => {
                Some(error)
            }
            Self::Layout(error) => Some(error),
            _ => None,
        }
    }
}
type Result<T> = std::result::Result<T, TransferError<Infallible>>;
fn plus(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b)
        .ok_or(TransferError::Overflow("backing/population"))
}
fn bytes<T>(n: usize) -> Result<usize> {
    std::alloc::Layout::array::<T>(n)
        .map(|l| l.size())
        .map_err(|_| TransferError::Overflow("requested layout"))
}
fn arc_bytes<T>() -> Result<usize> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|(layout, _)| layout.pad_to_align().size())
        .map_err(|_| TransferError::Overflow("Arc backing"))
}
fn layout_arcs(layout: &JetLayoutView<'_>) -> Result<usize> {
    plus(
        arc_bytes::<Vec<(usize, usize)>>()?,
        if layout.offsets.is_some() {
            arc_bytes::<(Vec<usize>, Vec<usize>)>()?
        } else {
            0
        },
    )
}
fn reserve<T>(n: usize) -> Result<Vec<T>> {
    allocation_attempt()?;
    let mut value = Vec::new();
    value
        .try_reserve_exact(n)
        .map_err(TransferError::<Infallible>::Allocation)?;
    Ok(value)
}
fn allocation_attempt() -> Result<()> {
    #[cfg(test)]
    {
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        if FAIL_ALLOCATION.with(|fail| fail.get() == Some(ALLOCATIONS.with(std::cell::Cell::get))) {
            let mut impossible = Vec::<u8>::new();
            impossible
                .try_reserve_exact(usize::MAX)
                .map_err(TransferError::<Infallible>::Allocation)?;
        }
    }
    Ok(())
}
#[cfg(test)]
thread_local! {
    static ALLOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FAIL_ALLOCATION: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}
fn copied<T: Copy>(source: &[T]) -> Result<Vec<T>> {
    let mut value = reserve(source.len())?;
    value.extend_from_slice(source);
    Ok(value)
}
fn widen<E: std::fmt::Debug + std::fmt::Display>(
    error: TransferError<Infallible>,
) -> TransferError<E> {
    match error {
        TransferError::Invalid(e) => TransferError::Invalid(e),
        TransferError::Unsupported(e) => TransferError::Unsupported(e),
        TransferError::Overflow(e) => TransferError::Overflow(e),
        TransferError::Supplier(e) => TransferError::Supplier(e),
        TransferError::Layout(e) => TransferError::Layout(e),
        TransferError::Math(e) => TransferError::Math(e),
        TransferError::Allocation(e) => TransferError::Allocation(e),
        TransferError::Admission(e) => match e {},
    }
}

#[derive(Clone, Copy)]
enum Program<'a> {
    Native(&'a [CompiledStage]),
    Passive(&'a [StageSnapshot]),
}
impl<'a> Program<'a> {
    fn len(self) -> usize {
        match self {
            Self::Native(v) => v.len(),
            Self::Passive(v) => v.len(),
        }
    }
    fn at(self, index: usize) -> Result<StageView<'a>> {
        Ok(match self {
            Self::Native(stages) => match &stages[index] {
                CompiledStage::Block {
                    evaluator,
                    resettable,
                    components,
                    inputs,
                    outputs,
                    gather,
                    scatter,
                    arguments,
                    values,
                    source,
                    checks,
                    checked_ends,
                } => match evaluator {
                    BlockEvaluator::Interpreted(evaluator) => StageView::Block(BlockView {
                        program: Arithmetic::Native(evaluator),
                        resettable: *resettable,
                        components,
                        inputs,
                        outputs,
                        gather,
                        scatter,
                        arguments: arguments.len(),
                        values: values.len(),
                        source: *source,
                        checks: Checks::Native(checks),
                        checked_ends,
                    }),
                    #[cfg(feature = "compiled-evaluator")]
                    BlockEvaluator::Native(_) => {
                        return Err(TransferError::Unsupported("native/JIT evaluator"));
                    }
                },
                CompiledStage::Require {
                    argument,
                    condition,
                    order,
                    source,
                    lineage,
                } => StageView::Require {
                    argument: *argument,
                    condition: *condition,
                    order: *order,
                    source: *source,
                    lineage: lineage.as_deref(),
                },
                CompiledStage::Domain {
                    stages,
                    frame,
                    reads,
                    layout,
                    argument,
                    token,
                    lineage,
                } => StageView::Domain {
                    stages: Self::Native(stages),
                    frame: frame.len(),
                    reads,
                    layout: layout.transfer_view(),
                    argument: *argument,
                    token: *token,
                    lineage,
                },
                CompiledStage::Applicability {
                    stages,
                    frame,
                    reads,
                    layout,
                    predicates,
                    inputs,
                    token,
                    plan,
                } => StageView::Applicability {
                    stages: Self::Native(stages),
                    frame: frame.len(),
                    reads,
                    layout: layout.transfer_view(),
                    predicates,
                    inputs,
                    token: *token,
                    plan,
                },
                CompiledStage::Branch {
                    require_separation,
                    comparison,
                    left,
                    right,
                    then,
                    otherwise,
                } => StageView::Branch {
                    require_separation: *require_separation,
                    comparison: *comparison,
                    left: *left,
                    right: *right,
                    then: Self::Native(then),
                    otherwise: Self::Native(otherwise),
                },
                CompiledStage::Provider { .. } => {
                    return Err(TransferError::Unsupported("provider stage"));
                }
            },
            Self::Passive(stages) => match &stages[index] {
                StageSnapshot::Block(b) => StageView::Block(BlockView {
                    program: Arithmetic::Passive(&b.program),
                    resettable: b.resettable,
                    components: &b.components,
                    inputs: &b.inputs,
                    outputs: &b.outputs,
                    gather: &b.gather,
                    scatter: &b.scatter,
                    arguments: b.arguments,
                    values: b.values,
                    source: b.source,
                    checks: Checks::Passive(&b.checks),
                    checked_ends: &b.checked_ends,
                }),
                StageSnapshot::Require {
                    argument,
                    condition,
                    order,
                    source,
                    lineage,
                } => StageView::Require {
                    argument: *argument,
                    condition: *condition,
                    order: *order,
                    source: *source,
                    lineage: lineage.as_ref(),
                },
                StageSnapshot::Domain(d) => StageView::Domain {
                    stages: Self::Passive(&d.stages),
                    frame: d.frame,
                    reads: &d.reads,
                    layout: d.layout.view(),
                    argument: d.argument,
                    token: d.token,
                    lineage: &d.lineage,
                },
                StageSnapshot::Applicability(a) => StageView::Applicability {
                    stages: Self::Passive(&a.stages),
                    frame: a.frame,
                    reads: &a.reads,
                    layout: a.layout.view(),
                    predicates: &a.predicates,
                    inputs: &a.inputs,
                    token: a.token,
                    plan: &a.plan,
                },
                StageSnapshot::Branch {
                    require_separation,
                    comparison,
                    left,
                    right,
                    then,
                    otherwise,
                } => StageView::Branch {
                    require_separation: *require_separation,
                    comparison: *comparison,
                    left: *left,
                    right: *right,
                    then: Self::Passive(then),
                    otherwise: Self::Passive(otherwise),
                },
            },
        })
    }
}
#[derive(Clone, Copy)]
enum Arithmetic<'a> {
    Native(&'a ExpressionEvaluator<f64>),
    Passive(&'a [u8]),
}
impl Arithmetic<'_> {
    fn facts(self) -> Result<(F64ProgramExtents, usize)> {
        let (extents, count) = match self {
            Self::Native(e) => {
                let p = e.f64_transfer()?;
                (p.extents(), p.operation_count()?)
            }
            Self::Passive(b) => {
                let p = F64ProgramView::inspect(b)?;
                (p.extents(), p.operation_count()?)
            }
        };
        let count = plus(
            plus(count.additions, count.multiplications)?,
            plus(count.inversions, count.function_calls)?,
        )?;
        Ok((extents, count))
    }
}
#[derive(Clone, Copy)]
enum Checks<'a> {
    Native(&'a [SourceCheck]),
    Passive(&'a [CheckSnapshot]),
}
impl Checks<'_> {
    fn len(self) -> usize {
        match self {
            Self::Native(v) => v.len(),
            Self::Passive(v) => v.len(),
        }
    }
    fn at(self, i: usize) -> (usize, SemanticId) {
        match self {
            Self::Native(v) => (v[i].end_output, v[i].source),
            Self::Passive(v) => (v[i].end_output, v[i].source),
        }
    }
}
struct BlockView<'a> {
    program: Arithmetic<'a>,
    resettable: bool,
    components: &'a [usize],
    inputs: &'a [usize],
    outputs: &'a [usize],
    gather: &'a [(usize, usize)],
    scatter: &'a [(usize, Option<usize>)],
    arguments: usize,
    values: usize,
    source: SemanticId,
    checks: Checks<'a>,
    checked_ends: &'a [usize],
}
enum StageView<'a> {
    Block(BlockView<'a>),
    Require {
        argument: usize,
        condition: Condition,
        order: DerivativeOrder,
        source: SemanticId,
        lineage: Option<&'a pse_model::diagnostic::ValidityLineage>,
    },
    Domain {
        stages: Program<'a>,
        frame: usize,
        reads: &'a [usize],
        layout: JetLayoutView<'a>,
        argument: usize,
        token: usize,
        lineage: &'a pse_model::diagnostic::ValidityLineage,
    },
    Applicability {
        stages: Program<'a>,
        frame: usize,
        reads: &'a [usize],
        layout: JetLayoutView<'a>,
        predicates: &'a [usize],
        inputs: &'a [usize],
        token: usize,
        plan: &'a pse_model::applicability::Node,
    },
    Branch {
        require_separation: bool,
        comparison: Comparison,
        left: usize,
        right: usize,
        then: Program<'a>,
        otherwise: Program<'a>,
    },
}

/// Derived requested backing/populations. Input overlap remains live during restore.
/// These facts carry no permission and are never encoded in an execution snapshot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransferExtents {
    /// Requested passive capture backing, including metadata and supplier encodings.
    pub capture_bytes: usize,
    /// Actual owned passive backing overlapping restoration (capacities included).
    pub input_bytes: usize,
    /// Requested restored immutable template backing.
    pub template_bytes: usize,
    /// Prepaid complete lazy worker backing, derived from checked stages/layouts.
    pub worker_bytes: usize,
    /// Requested validation scratch peak, before immutable/worker materialization.
    pub validation_bytes: usize,
    /// Existing native point/result cache backing, independently retained by root consumers.
    pub evaluation_bytes: usize,
    /// Requested peak with the retained passive input, not allocator usable size/RSS.
    pub restore_peak_bytes: usize,
    /// Actual control and arithmetic stage population across both orders and arms.
    pub stages: usize,
    /// Checked supplier arithmetic population across both orders and arms.
    pub library_operations: usize,
    /// Checked supplier instruction population across both orders and arms.
    pub library_instructions: usize,
    /// Actual existing applicability-plan nodes, including every union alternative.
    pub applicability_nodes: usize,
}

#[derive(Default)]
struct Counts {
    facts: TransferExtents,
    order_bytes: [usize; 2],
    frame_entries: [usize; 2],
    order_counts: [OrderCounts; 2],
    index_scratch: usize,
    identity_scratch: usize,
    evidence_bytes: usize,
    numeric_bytes: usize,
}
#[derive(Default, Clone, Copy)]
struct OrderCounts {
    stages: usize,
    blocks: usize,
    inputs: usize,
    outputs: usize,
    controls: usize,
    operations: usize,
    instructions: usize,
    instruction_bytes: usize,
}
/// One enclosing execution frame's slot bound, fixed limits and product order.
#[derive(Clone, Copy)]
struct ProgramContext {
    slots: usize,
    limits: EvaluationLimits,
    order: usize,
}
fn check_index(index: usize, bound: usize) -> Result<()> {
    if index >= bound {
        Err(TransferError::Invalid("index outside checked extent"))
    } else {
        Ok(())
    }
}
fn indices(values: &[usize], bound: usize) -> Result<()> {
    for &i in values {
        check_index(i, bound)?;
    }
    Ok(())
}
fn layout_bytes(layout: &JetLayoutView<'_>) -> Result<usize> {
    let mut n = plus(
        bytes::<usize>(layout.coordinates.len())?,
        bytes::<Vec<usize>>(layout.shape.len())?,
    )?;
    for row in layout.shape {
        n = plus(n, bytes::<usize>(row.len())?)?;
    }
    n = plus(n, bytes::<(usize, usize)>(layout.pairs.len())?)?;
    if let Some(v) = layout.offsets {
        n = plus(n, bytes::<usize>(v.len())?)?;
    }
    if let Some(v) = layout.components {
        n = plus(n, bytes::<usize>(v.len())?)?;
    }
    Ok(n)
}

fn lineage_bytes(lineage: &pse_model::diagnostic::ValidityLineage) -> Result<usize> {
    plus(
        bytes::<SemanticId>(plus(lineage.sets.len(), lineage.members.len())?)?,
        bytes::<u32>(lineage.variables.len())?,
    )
}
fn plan_extents(
    plan: &pse_model::applicability::Node,
    predicates: usize,
    inputs: usize,
) -> Result<pse_model::applicability::CaptureExtents> {
    plan.checked_capture_extents(predicates, inputs, usize::MAX)
        .map_err(|error| match error {
            pse_model::applicability::CaptureExtentError::Structure => {
                TransferError::Invalid("applicability plan references")
            }
            pse_model::applicability::CaptureExtentError::Overflow => {
                TransferError::Math(MathError::Limit("applicability capture storage"))
            }
        })
}

impl Counts {
    fn program(
        &mut self,
        program: Program<'_>,
        layout: &JetLayoutView<'_>,
        context: ProgramContext,
        left: &mut usize,
        depth: usize,
    ) -> Result<()> {
        let ProgramContext {
            slots,
            limits,
            order,
        } = context;
        if depth > MAX_STAGE_DEPTH {
            return Err(TransferError::Unsupported("stage recursion"));
        }
        let frame = layout.validate(slots, slots, limits)?;
        self.index_scratch = self.index_scratch.max(layout.coordinates.len());
        let descriptors = bytes::<CompiledStage>(program.len())?;
        self.order_bytes[order] = plus(self.order_bytes[order], descriptors)?;
        self.facts.template_bytes = plus(self.facts.template_bytes, descriptors)?;
        self.facts.capture_bytes = plus(
            self.facts.capture_bytes,
            bytes::<StageSnapshot>(program.len())?,
        )?;
        for i in 0..program.len() {
            if *left == 0 {
                return Err(TransferError::Overflow("stage population"));
            }
            *left -= 1;
            self.facts.stages = plus(self.facts.stages, 1)?;
            self.order_counts[order].stages = plus(self.order_counts[order].stages, 1)?;
            match program.at(i)? {
                StageView::Block(b) => {
                    if !b.resettable {
                        return Err(TransferError::Unsupported("nonresettable evaluator"));
                    }
                    let (supplier, operations) = b.program.facts()?;
                    if supplier.parameters != b.arguments || supplier.results != b.values {
                        return Err(TransferError::Invalid(
                            "supplier/native argument or result extent",
                        ));
                    }
                    indices(b.inputs, slots)?;
                    indices(b.outputs, slots)?;
                    if b.components.is_empty()
                        || b.components.iter().any(|&c| c >= layout.shape.len())
                        || b.components.windows(2).any(|p| p[0] >= p[1])
                    {
                        return Err(TransferError::Invalid("block components"));
                    }
                    for &(argument, source) in b.gather {
                        check_index(argument, b.arguments)?;
                        check_index(source, frame)?;
                    }
                    for &(destination, value) in b.scatter {
                        check_index(destination, frame)?;
                        if let Some(value) = value {
                            check_index(value, b.values)?;
                        }
                    }
                    if b.checks.len() != b.checked_ends.len() {
                        return Err(TransferError::Invalid("source check extents"));
                    }
                    let (mut output_end, mut value_end) = (0, 0);
                    for j in 0..b.checks.len() {
                        let end = b.checks.at(j).0;
                        let checked = b.checked_ends[j];
                        if end <= output_end
                            || end > b.outputs.len()
                            || checked < value_end
                            || checked > b.values
                        {
                            return Err(TransferError::Invalid("source check ranges"));
                        }
                        output_end = end;
                        value_end = checked;
                    }
                    if output_end != b.outputs.len() || value_end != b.values {
                        return Err(TransferError::Invalid("incomplete source check ranges"));
                    }
                    let metadata = plus(
                        plus(
                            bytes::<usize>(plus(
                                plus(b.components.len(), b.inputs.len())?,
                                plus(b.outputs.len(), b.checked_ends.len())?,
                            )?)?,
                            bytes::<(usize, usize)>(b.gather.len())?,
                        )?,
                        plus(
                            bytes::<(usize, Option<usize>)>(b.scatter.len())?,
                            bytes::<SourceCheck>(b.checks.len())?,
                        )?,
                    )?;
                    let numeric = bytes::<f64>(plus(b.arguments, b.values)?)?;
                    self.numeric_bytes =
                        plus(self.numeric_bytes, plus(numeric, supplier.stack_bytes)?)?;
                    let retained = plus(plus(metadata, numeric)?, supplier.retained_bytes)?;
                    self.order_bytes[order] = plus(self.order_bytes[order], retained)?;
                    self.facts.template_bytes = plus(self.facts.template_bytes, retained)?;
                    self.facts.capture_bytes = plus(
                        self.facts.capture_bytes,
                        plus(metadata, supplier.encoded_bytes)?,
                    )?;
                    self.facts.validation_bytes =
                        self.facts.validation_bytes.max(supplier.validation_bytes);
                    let c = &mut self.order_counts[order];
                    c.blocks = plus(c.blocks, 1)?;
                    c.inputs = plus(c.inputs, b.inputs.len())?;
                    c.outputs = plus(c.outputs, b.outputs.len())?;
                    c.operations = plus(c.operations, operations)?;
                    c.instructions = plus(c.instructions, supplier.instructions)?;
                    c.instruction_bytes = plus(
                        c.instruction_bytes,
                        plus(supplier.instruction_bytes, supplier.result_bytes)?,
                    )?;
                    self.facts.library_operations =
                        plus(self.facts.library_operations, operations)?;
                    self.facts.library_instructions =
                        plus(self.facts.library_instructions, supplier.instructions)?;
                }
                StageView::Require {
                    argument, lineage, ..
                } => {
                    check_index(argument, slots)?;
                    self.order_counts[order].controls = plus(self.order_counts[order].controls, 1)?;
                    if let Some(lineage) = lineage {
                        let n = lineage_bytes(lineage)?;
                        self.facts.capture_bytes = plus(self.facts.capture_bytes, n)?;
                        self.facts.template_bytes = plus(
                            self.facts.template_bytes,
                            plus(n, arc_bytes::<pse_model::diagnostic::ValidityLineage>()?)?,
                        )?;
                    }
                }
                StageView::Domain {
                    stages,
                    frame: local,
                    reads,
                    layout: nested,
                    argument,
                    token,
                    lineage,
                } => {
                    self.nested(&nested, local, reads, token, context)?;
                    check_index(argument, local)?;
                    let n = lineage_bytes(lineage)?;
                    self.facts.capture_bytes = plus(self.facts.capture_bytes, n)?;
                    self.facts.template_bytes = plus(
                        self.facts.template_bytes,
                        plus(n, arc_bytes::<pse_model::diagnostic::ValidityLineage>()?)?,
                    )?;
                    self.program(
                        stages,
                        &nested,
                        ProgramContext {
                            slots: local,
                            ..context
                        },
                        left,
                        depth + 1,
                    )?;
                }
                StageView::Applicability {
                    stages,
                    frame: local,
                    reads,
                    layout: nested,
                    predicates,
                    inputs,
                    token,
                    plan,
                } => {
                    self.nested(&nested, local, reads, token, context)?;
                    indices(predicates, local)?;
                    indices(inputs, local)?;
                    let extent = plan_extents(plan, predicates.len(), inputs.len())?;
                    self.identity_scratch = self.identity_scratch.max(extent.validation_entries);
                    let plan_backing = extent.clone_bytes;
                    let observed = extent.observation_bytes;
                    self.facts.applicability_nodes =
                        plus(self.facts.applicability_nodes, extent.nodes)?;
                    let metadata = bytes::<usize>(plus(predicates.len(), inputs.len())?)?;
                    self.facts.capture_bytes =
                        plus(self.facts.capture_bytes, plus(metadata, plan_backing)?)?;
                    self.facts.template_bytes = plus(
                        self.facts.template_bytes,
                        plus(
                            plus(metadata, plan_backing)?,
                            arc_bytes::<pse_model::applicability::Node>()?,
                        )?,
                    )?;
                    self.order_bytes[order] = plus(self.order_bytes[order], metadata)?;
                    let witnesses = plus(
                        2 * size_of::<ApplicabilityWitness>(),
                        plus(
                            bytes::<bool>(plus(predicates.len(), predicates.len())?)?,
                            bytes::<f64>(plus(inputs.len(), inputs.len())?)?,
                        )?,
                    )?;
                    self.evidence_bytes =
                        plus(
                            self.evidence_bytes,
                            plus(
                                bytes::<u8>(observed.checked_mul(4).ok_or(
                                    TransferError::Overflow("applicability observations"),
                                )?)?,
                                witnesses,
                            )?,
                        )?;
                    self.program(
                        stages,
                        &nested,
                        ProgramContext {
                            slots: local,
                            ..context
                        },
                        left,
                        depth + 1,
                    )?;
                }
                StageView::Branch {
                    left: lhs,
                    right,
                    then,
                    otherwise,
                    ..
                } => {
                    check_index(lhs, slots)?;
                    check_index(right, slots)?;
                    self.order_counts[order].controls = plus(self.order_counts[order].controls, 1)?;
                    self.program(then, layout, context, left, depth + 1)?;
                    self.program(otherwise, layout, context, left, depth + 1)?;
                }
            }
        }
        Ok(())
    }
    fn nested(
        &mut self,
        layout: &JetLayoutView<'_>,
        frame: usize,
        reads: &[usize],
        token: usize,
        parent: ProgramContext,
    ) -> Result<()> {
        let ProgramContext {
            slots: parent_slots,
            limits,
            order,
        } = parent;
        if layout.order != DerivativeOrder::Value
            || !layout.coordinates.is_empty()
            || frame != reads.len()
            || layout.validate(frame, frame, limits)? != frame
        {
            return Err(TransferError::Invalid("nested value frame"));
        }
        indices(reads, parent_slots)?;
        check_index(token, parent_slots)?;
        let metadata = plus(bytes::<usize>(reads.len())?, layout_bytes(layout)?)?;
        self.facts.capture_bytes = plus(self.facts.capture_bytes, metadata)?;
        self.facts.template_bytes = plus(
            self.facts.template_bytes,
            plus(plus(metadata, layout_arcs(layout)?)?, bytes::<f64>(frame)?)?,
        )?;
        self.order_bytes[order] = plus(
            self.order_bytes[order],
            plus(metadata, bytes::<f64>(frame)?)?,
        )?;
        self.numeric_bytes = plus(self.numeric_bytes, bytes::<f64>(frame)?)?;
        self.order_counts[order].controls = plus(self.order_counts[order].controls, 1)?;
        Ok(())
    }
}

fn capture_layout(layout: &JetLayoutView<'_>) -> Result<JetLayoutSnapshot> {
    let mut shape = reserve(layout.shape.len())?;
    for row in layout.shape {
        shape.push(copied(row)?);
    }
    Ok(JetLayoutSnapshot {
        coordinates: copied(layout.coordinates)?,
        shape,
        pairs: copied(layout.pairs)?,
        order: layout.order,
        slot_offsets: layout.offsets.map(copied).transpose()?,
        slot_components: layout.components.map(copied).transpose()?,
    })
}
fn capture_lineage(
    l: &pse_model::diagnostic::ValidityLineage,
) -> Result<pse_model::diagnostic::ValidityLineage> {
    Ok(pse_model::diagnostic::ValidityLineage {
        layer: l.layer,
        source: l.source,
        form: l.form,
        sets: copied(&l.sets)?,
        variables: copied(&l.variables)?,
        members: copied(&l.members)?,
    })
}
fn capture_text(text: &str) -> Result<String> {
    allocation_attempt()?;
    let mut value = String::new();
    value
        .try_reserve_exact(text.len())
        .map_err(TransferError::<Infallible>::Allocation)?;
    value.push_str(text);
    Ok(value)
}
fn capture_plan(p: &pse_model::applicability::Node) -> Result<pse_model::applicability::Node> {
    use pse_model::applicability::{Claim, Node, Permission, Region};
    fn children(nodes: &[Node]) -> Result<Vec<Node>> {
        let mut result = reserve(nodes.len())?;
        for node in nodes {
            result.push(capture_plan(node)?);
        }
        Ok(result)
    }
    let c = &p.claim;
    let claim = Claim {
        id: c.id,
        coverage: c.coverage,
        owner: c.owner,
        owner_lineage: copied(&c.owner_lineage)?,
        evidence: c.evidence,
        form: c.form,
        call: c.call,
        records: copied(&c.records)?,
        dependencies: copied(&c.dependencies)?,
        layer: c.layer,
        basis: c.basis,
        reason: c.reason.as_deref().map(capture_text).transpose()?,
    };
    let region = match &p.region {
        Region::Predicate(i) => Region::Predicate(*i),
        Region::Unrestricted => Region::Unrestricted,
        Region::Unknown => Region::Unknown,
        Region::Union(v) => Region::Union(children(v)?),
    };
    let mut inputs = reserve(p.inputs.len())?;
    for (name, index, quantity) in &p.inputs {
        inputs.push((capture_text(name)?, *index, *quantity));
    }
    let mut permissions = reserve(p.permissions.len())?;
    for permission in &p.permissions {
        permissions.push(Permission {
            id: permission.id,
            scope: permission.scope,
            target_kind: permission.target_kind,
            targets: copied(&permission.targets)?,
            allow_unknown: permission.allow_unknown,
            allow_extrapolation: permission.allow_extrapolation,
        });
    }
    Ok(Node {
        claim,
        region,
        dependencies: children(&p.dependencies)?,
        inputs,
        permissions,
    })
}
fn capture_program(program: Program<'_>) -> Result<Vec<StageSnapshot>> {
    let mut result = reserve(program.len())?;
    for i in 0..program.len() {
        result.push(match program.at(i)? {
            StageView::Block(b) => {
                let Arithmetic::Native(evaluator) = b.program else {
                    return Err(TransferError::Invalid(
                        "capture requires original native program",
                    ));
                };
                let supplier = evaluator.f64_transfer()?;
                let mut program = reserve(supplier.encoded_len())?;
                program.resize(supplier.encoded_len(), 0);
                supplier.write_into(&mut program)?;
                let mut checks = reserve(b.checks.len())?;
                for j in 0..b.checks.len() {
                    let (end_output, source) = b.checks.at(j);
                    checks.push(CheckSnapshot { end_output, source });
                }
                StageSnapshot::Block(BlockSnapshot {
                    program,
                    resettable: b.resettable,
                    components: copied(b.components)?,
                    inputs: copied(b.inputs)?,
                    outputs: copied(b.outputs)?,
                    gather: copied(b.gather)?,
                    scatter: copied(b.scatter)?,
                    arguments: b.arguments,
                    values: b.values,
                    source: b.source,
                    checks,
                    checked_ends: copied(b.checked_ends)?,
                })
            }
            StageView::Require {
                argument,
                condition,
                order,
                source,
                lineage,
            } => StageSnapshot::Require {
                argument,
                condition,
                order,
                source,
                lineage: lineage.map(capture_lineage).transpose()?,
            },
            StageView::Domain {
                stages,
                frame,
                reads,
                layout,
                argument,
                token,
                lineage,
            } => StageSnapshot::Domain(DomainSnapshot {
                stages: capture_program(stages)?,
                frame,
                reads: copied(reads)?,
                layout: capture_layout(&layout)?,
                argument,
                token,
                lineage: capture_lineage(lineage)?,
            }),
            StageView::Applicability {
                stages,
                frame,
                reads,
                layout,
                predicates,
                inputs,
                token,
                plan,
            } => StageSnapshot::Applicability(ApplicabilitySnapshot {
                stages: capture_program(stages)?,
                frame,
                reads: copied(reads)?,
                layout: capture_layout(&layout)?,
                predicates: copied(predicates)?,
                inputs: copied(inputs)?,
                token,
                plan: capture_plan(plan)?,
            }),
            StageView::Branch {
                require_separation,
                comparison,
                left,
                right,
                then,
                otherwise,
            } => StageSnapshot::Branch {
                require_separation,
                comparison,
                left,
                right,
                then: capture_program(then)?,
                otherwise: capture_program(otherwise)?,
            },
        });
    }
    Ok(result)
}

/// Nonserializable capture owner. Passive data borrows keep its actual admission live.
#[derive(Debug)]
pub struct CapturedExecution {
    snapshot: ExecutionSnapshot,
    owner: Arc<dyn AllocationOwner>,
}
impl CapturedExecution {
    /// Bind already-admitted passive data to its actual allocation lifetime.
    /// This ownership-only operation validates nothing and grants no source, Root,
    /// compute or execution authority. Wire decode/conversion/path scratch must be
    /// admitted by the boundary before conversion, independently of later restore.
    pub fn from_admitted_snapshot(
        snapshot: ExecutionSnapshot,
        owner: Arc<dyn AllocationOwner>,
    ) -> Self {
        Self { snapshot, owner }
    }
    /// Borrow passive metadata while keeping its actual allocation owner alive.
    pub fn snapshot(&self) -> &ExecutionSnapshot {
        &self.snapshot
    }
    /// The publisher must retain the returned actual owner through encoding/drain.
    pub fn into_parts(self) -> (ExecutionSnapshot, Arc<dyn AllocationOwner>) {
        (self.snapshot, self.owner)
    }
}

/// Capture an original compiled template without warming any mutable evaluator.
/// Original source authority remains with `body`; the returned snapshot is data.
pub fn capture<E: std::fmt::Debug + std::fmt::Display>(
    body: &CompiledBody,
    scope: &pse_kernels::ExecutionScope,
    admit: impl FnOnce(&TransferExtents) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
) -> std::result::Result<CapturedExecution, TransferError<E>> {
    capture_frame(&body.numeric_frame_scoped(scope.clone()), scope, admit)
}

pub(crate) fn capture_frame<E: std::fmt::Debug + std::fmt::Display>(
    frame: &NumericFrame,
    scope: &pse_kernels::ExecutionScope,
    admit: impl FnOnce(&TransferExtents) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
) -> std::result::Result<CapturedExecution, TransferError<E>> {
    scope.check().map_err(crate::error::scope_error)?;
    let root = RootView::native(frame).map_err(widen::<E>)?;
    let counts = root.inspect().map_err(widen::<E>)?;
    scope.check().map_err(crate::error::scope_error)?;
    let owner = admit(&counts.facts).map_err(TransferError::Admission)?;
    scope.check().map_err(crate::error::scope_error)?;
    let snapshot = (|| -> Result<ExecutionSnapshot> {
        let mut first_rows = reserve(root.rows.len())?;
        for i in 0..root.rows.len() {
            first_rows.push(root.rows.at(i));
        }
        Ok(ExecutionSnapshot {
            version: 1,
            inputs: root.inputs,
            input_formals: copied(root.formals)?,
            coordinate_offsets: copied(root.coordinates)?,
            slots: root.slots,
            outputs: copied(root.outputs)?,
            limits: root.limits,
            value_layout: capture_layout(&root.layouts[0])?,
            first_layout: capture_layout(&root.layouts[1])?,
            first_rows,
            first_axes: copied(root.axes)?,
            value: capture_program(root.programs[0])?,
            first: capture_program(root.programs[1])?,
        })
    })()
    .map_err(widen::<E>)?;
    scope.check().map_err(crate::error::scope_error)?;
    Ok(CapturedExecution { snapshot, owner })
}

fn fresh(entries: usize, fill: f64) -> Result<Vec<f64>> {
    let mut result = reserve(entries)?;
    result.resize(entries, fill);
    Ok(result)
}
fn restore_program(
    stages: Vec<StageSnapshot>,
    limits: EvaluationLimits,
    index_scratch: &mut [usize],
    identity_scratch: &mut [SemanticId],
) -> Result<Vec<CompiledStage>> {
    let mut result = reserve(stages.len())?;
    for stage in stages {
        result.push(match stage {
            StageSnapshot::Block(b) => {
                let evaluator = F64ProgramView::inspect(&b.program)?
                    .restore_with(|_| Ok::<_, Infallible>(()))
                    .map_err(|error| match error {
                        F64RestoreError::Program(e) => TransferError::Supplier(e),
                        F64RestoreError::Allocation(e) => TransferError::Allocation(e),
                        F64RestoreError::Admission(e) => match e {},
                    })?;
                let mut arguments = fresh(b.arguments, 0.0)?;
                for &(argument, _) in &b.gather {
                    arguments[argument] = f64::NAN;
                }
                let mut checks = reserve(b.checks.len())?;
                for check in b.checks {
                    checks.push(SourceCheck {
                        end_output: check.end_output,
                        source: check.source,
                    });
                }
                CompiledStage::Block {
                    evaluator: BlockEvaluator::Interpreted(evaluator),
                    resettable: b.resettable,
                    components: b.components,
                    inputs: b.inputs,
                    outputs: b.outputs,
                    gather: b.gather,
                    scatter: b.scatter,
                    arguments,
                    values: fresh(b.values, f64::NAN)?,
                    source: b.source,
                    checks,
                    checked_ends: b.checked_ends,
                }
            }
            StageSnapshot::Require {
                argument,
                condition,
                order,
                source,
                lineage,
            } => CompiledStage::Require {
                argument,
                condition,
                order,
                source,
                lineage: lineage.map(Arc::new),
            },
            StageSnapshot::Domain(d) => CompiledStage::Domain {
                stages: restore_program(d.stages, limits, index_scratch, identity_scratch)?,
                frame: fresh(d.frame, f64::NAN)?,
                reads: d.reads,
                layout: d.layout.restore(d.frame, d.frame, limits, index_scratch)?,
                argument: d.argument,
                token: d.token,
                lineage: Arc::new(d.lineage),
            },
            StageSnapshot::Applicability(a) => {
                if !a.plan.valid_capture_with_scratch(
                    a.predicates.len(),
                    a.inputs.len(),
                    usize::MAX,
                    identity_scratch,
                ) {
                    return Err(TransferError::Invalid("applicability lineage validation"));
                }
                CompiledStage::Applicability {
                    stages: restore_program(a.stages, limits, index_scratch, identity_scratch)?,
                    frame: fresh(a.frame, f64::NAN)?,
                    reads: a.reads,
                    layout: a.layout.restore(a.frame, a.frame, limits, index_scratch)?,
                    predicates: a.predicates,
                    inputs: a.inputs,
                    token: a.token,
                    plan: Arc::new(a.plan),
                }
            }
            StageSnapshot::Branch {
                require_separation,
                comparison,
                left,
                right,
                then,
                otherwise,
            } => CompiledStage::Branch {
                require_separation,
                comparison,
                left,
                right,
                then: restore_program(then, limits, index_scratch, identity_scratch)?,
                otherwise: restore_program(otherwise, limits, index_scratch, identity_scratch)?,
            },
        });
    }
    Ok(result)
}

#[derive(Clone, Copy)]
enum Rows<'a> {
    Native(&'a [std::ops::Range<usize>]),
    Passive(&'a [(usize, usize)]),
}
impl Rows<'_> {
    fn len(self) -> usize {
        match self {
            Self::Native(v) => v.len(),
            Self::Passive(v) => v.len(),
        }
    }
    fn at(self, index: usize) -> (usize, usize) {
        match self {
            Self::Native(v) => (v[index].start, v[index].end),
            Self::Passive(v) => v[index],
        }
    }
}
fn validate_first_rows(
    rows: Rows<'_>,
    axes: &[usize],
    outputs: usize,
    coordinates: usize,
) -> Result<()> {
    if rows.len() != outputs {
        return Err(TransferError::Invalid("First row population"));
    }
    let mut end = 0;
    for row in 0..rows.len() {
        let (start, next) = rows.at(row);
        let selected = axes
            .get(start..next)
            .ok_or(TransferError::Invalid("First row range"))?;
        if start != end || selected.windows(2).any(|p| p[0] >= p[1]) {
            return Err(TransferError::Invalid("First row order/coverage"));
        }
        indices(selected, coordinates)?;
        end = next;
    }
    if end != axes.len() {
        return Err(TransferError::Invalid("First axis coverage"));
    }
    Ok(())
}
impl FirstLayout {
    fn checked_restore(
        rows: Vec<(usize, usize)>,
        axes: Vec<usize>,
        outputs: usize,
        coordinates: usize,
    ) -> Result<Self> {
        validate_first_rows(Rows::Passive(&rows), &axes, outputs, coordinates)?;
        let mut ranges = reserve(rows.len())?;
        for (start, end) in rows {
            ranges.push(start..end);
        }
        Ok(Self { rows: ranges, axes })
    }
}
struct RootView<'a> {
    inputs: usize,
    formals: &'a [usize],
    coordinates: &'a [usize],
    slots: usize,
    outputs: &'a [usize],
    limits: EvaluationLimits,
    layouts: [JetLayoutView<'a>; 2],
    rows: Rows<'a>,
    axes: &'a [usize],
    programs: [Program<'a>; 2],
}
impl<'a> RootView<'a> {
    fn native(frame: &'a NumericFrame) -> Result<Self> {
        let t = &frame.template;
        if t.directional
            || t.product != ProductKind::SelectedPartials
            || t.direct_product.is_some()
            || t.reverse_product.is_some()
            || !t.required_provider_keys.is_empty()
            || t.provider_visit_capacity != 0
            || t.programs[DerivativeOrder::Second].is_some()
            || t.layouts[DerivativeOrder::Second].is_some()
        {
            return Err(TransferError::Unsupported(
                "requires nondirectional interpreted Value+First selected partials",
            ));
        }
        let value = t.layouts[DerivativeOrder::Value]
            .as_ref()
            .ok_or(TransferError::Unsupported("missing Value layout"))?;
        let first = t.layouts[DerivativeOrder::First]
            .as_ref()
            .ok_or(TransferError::Unsupported("missing First layout"))?;
        Ok(Self {
            inputs: t.inputs,
            formals: &t.input_formals,
            coordinates: &t.coordinate_offsets,
            slots: t.slots,
            outputs: &t.outputs,
            limits: t.limits,
            layouts: [value.transfer_view(), first.transfer_view()],
            rows: Rows::Native(&t.first_layout.rows),
            axes: &t.first_layout.axes,
            programs: [
                Program::Native(
                    t.programs[DerivativeOrder::Value]
                        .as_deref()
                        .ok_or(TransferError::Unsupported("missing Value program"))?,
                ),
                Program::Native(
                    t.programs[DerivativeOrder::First]
                        .as_deref()
                        .ok_or(TransferError::Unsupported("missing First program"))?,
                ),
            ],
        })
    }
    fn passive(snapshot: &'a ExecutionSnapshot) -> Result<Self> {
        if snapshot.version != 1 {
            return Err(TransferError::Unsupported("snapshot version"));
        }
        Ok(Self {
            inputs: snapshot.inputs,
            formals: &snapshot.input_formals,
            coordinates: &snapshot.coordinate_offsets,
            slots: snapshot.slots,
            outputs: &snapshot.outputs,
            limits: snapshot.limits,
            layouts: [snapshot.value_layout.view(), snapshot.first_layout.view()],
            rows: Rows::Passive(&snapshot.first_rows),
            axes: &snapshot.first_axes,
            programs: [
                Program::Passive(&snapshot.value),
                Program::Passive(&snapshot.first),
            ],
        })
    }
    fn inspect(&self) -> Result<Counts> {
        self.limits.check()?;
        if self.inputs != self.formals.len()
            || self.inputs > self.slots
            || self.rows.len() != self.outputs.len()
        {
            return Err(TransferError::Invalid("template signature extent"));
        }
        indices(self.outputs, self.slots)?;
        indices(self.coordinates, self.inputs)?;
        validate_first_rows(
            self.rows,
            self.axes,
            self.outputs.len(),
            self.coordinates.len(),
        )?;
        let signature = plus(
            bytes::<usize>(plus(
                plus(self.formals.len(), self.coordinates.len())?,
                plus(self.outputs.len(), self.axes.len())?,
            )?)?,
            bytes::<(usize, usize)>(self.rows.len())?,
        )?;
        let mut counts = Counts {
            index_scratch: self.inputs.max(self.coordinates.len()),
            facts: TransferExtents {
                capture_bytes: plus(size_of::<ExecutionSnapshot>(), signature)?,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut shells = arc_bytes::<ExecutionTemplate>()?;
        for _ in 0..3 {
            shells = plus(shells, arc_bytes::<Vec<usize>>()?)?;
        }
        shells = plus(shells, arc_bytes::<Vec<ProviderKey>>()?)?;
        shells = plus(
            shells,
            arc_bytes::<EnumMap<DerivativeOrder, Option<JetLayout>>>()?,
        )?;
        shells = plus(
            shells,
            arc_bytes::<EnumMap<DerivativeOrder, Option<Vec<CompiledStage>>>>()?,
        )?;
        shells = plus(shells, arc_bytes::<FirstLayout>()?)?;
        shells = plus(
            shells,
            arc_bytes::<(
                Arc<dyn AllocationOwner>,
                Arc<dyn AllocationOwner>,
                Arc<dyn ConstructionAdmission>,
            )>()?,
        )?;
        counts.facts.template_bytes = plus(signature, shells)?;
        for (i, order) in [DerivativeOrder::Value, DerivativeOrder::First]
            .into_iter()
            .enumerate()
        {
            let layout = &self.layouts[i];
            if layout.order != order || layout.coordinates != self.coordinates {
                return Err(TransferError::Invalid("signature/layout coordinate order"));
            }
            counts.frame_entries[i] = layout.validate(self.slots, self.inputs, self.limits)?;
            let backing = layout_bytes(layout)?;
            counts.facts.capture_bytes = plus(counts.facts.capture_bytes, backing)?;
            counts.facts.template_bytes = plus(
                counts.facts.template_bytes,
                plus(backing, layout_arcs(layout)?)?,
            )?;
            let mut left = usize::MAX;
            counts.program(
                self.programs[i],
                layout,
                ProgramContext {
                    slots: self.slots,
                    limits: self.limits,
                    order: i,
                },
                &mut left,
                0,
            )?;
        }
        if counts.facts.library_operations > self.limits.operations {
            return Err(MathError::Limit("compiled operations").into());
        }
        let worker = plus(
            bytes::<f64>(*counts.frame_entries.iter().max().unwrap_or(&0))?,
            plus(counts.order_bytes[0], counts.order_bytes[1])?,
        )?;
        counts.facts.worker_bytes = plus(worker, counts.evidence_bytes)?;
        if plus(
            counts.numeric_bytes,
            plus(
                bytes::<f64>(*counts.frame_entries.iter().max().unwrap_or(&0))?,
                counts.evidence_bytes,
            )?,
        )? > self.limits.scratch_bytes
        {
            return Err(MathError::Limit("worker scratch bytes").into());
        }
        counts.facts.validation_bytes = plus(
            counts.facts.validation_bytes,
            plus(
                bytes::<usize>(counts.index_scratch)?,
                bytes::<SemanticId>(counts.identity_scratch)?,
            )?,
        )?;
        let results = plus(self.outputs.len(), self.axes.len())?;
        counts.facts.evaluation_bytes = plus(
            plus(
                bytes::<f64>(
                    plus(results, self.inputs)?
                        .checked_mul(2)
                        .ok_or(TransferError::Overflow("evaluation cache"))?,
                )?,
                counts.evidence_bytes,
            )?,
            size_of::<TrialEvaluation>() + size_of::<Vec<u64>>() + 64,
        )?;
        counts.facts.restore_peak_bytes = plus(
            counts.facts.template_bytes,
            counts.facts.validation_bytes.max(counts.facts.worker_bytes),
        )?;
        Ok(counts)
    }
}

fn owned_layout(layout: &JetLayoutSnapshot) -> Result<usize> {
    let mut n = plus(
        bytes::<usize>(layout.coordinates.capacity())?,
        bytes::<Vec<usize>>(layout.shape.capacity())?,
    )?;
    for v in &layout.shape {
        n = plus(n, bytes::<usize>(v.capacity())?)?;
    }
    n = plus(n, bytes::<(usize, usize)>(layout.pairs.capacity())?)?;
    for v in [&layout.slot_offsets, &layout.slot_components]
        .into_iter()
        .flatten()
    {
        n = plus(n, bytes::<usize>(v.capacity())?)?;
    }
    Ok(n)
}
fn owned_lineage(l: &pse_model::diagnostic::ValidityLineage) -> Result<usize> {
    plus(
        bytes::<SemanticId>(plus(l.sets.capacity(), l.members.capacity())?)?,
        bytes::<u32>(l.variables.capacity())?,
    )
}
fn owned_program(stages: &Vec<StageSnapshot>) -> Result<usize> {
    let mut n = bytes::<StageSnapshot>(stages.capacity())?;
    for stage in stages {
        n = plus(
            n,
            match stage {
                StageSnapshot::Block(b) => plus(
                    plus(
                        bytes::<u8>(b.program.capacity())?,
                        bytes::<usize>(plus(
                            plus(b.components.capacity(), b.inputs.capacity())?,
                            plus(b.outputs.capacity(), b.checked_ends.capacity())?,
                        )?)?,
                    )?,
                    plus(
                        bytes::<(usize, usize)>(b.gather.capacity())?,
                        plus(
                            bytes::<(usize, Option<usize>)>(b.scatter.capacity())?,
                            bytes::<CheckSnapshot>(b.checks.capacity())?,
                        )?,
                    )?,
                )?,
                StageSnapshot::Require { lineage, .. } => lineage
                    .as_ref()
                    .map(owned_lineage)
                    .transpose()?
                    .unwrap_or(0),
                StageSnapshot::Domain(d) => plus(
                    plus(owned_program(&d.stages)?, owned_layout(&d.layout)?)?,
                    plus(
                        bytes::<usize>(d.reads.capacity())?,
                        owned_lineage(&d.lineage)?,
                    )?,
                )?,
                StageSnapshot::Applicability(a) => plus(
                    plus(owned_program(&a.stages)?, owned_layout(&a.layout)?)?,
                    plus(
                        bytes::<usize>(plus(
                            plus(a.reads.capacity(), a.predicates.capacity())?,
                            a.inputs.capacity(),
                        )?)?,
                        plan_extents(&a.plan, a.predicates.len(), a.inputs.len())?.owned_bytes,
                    )?,
                )?,
                StageSnapshot::Branch {
                    then, otherwise, ..
                } => plus(owned_program(then)?, owned_program(otherwise)?)?,
            },
        )?;
    }
    Ok(n)
}
fn excess<T>(values: &Vec<T>) -> Result<usize> {
    bytes::<T>(values.capacity() - values.len())
}
fn layout_excess(layout: &JetLayoutSnapshot) -> Result<usize> {
    Ok(owned_layout(layout)? - layout_bytes(&layout.view())?)
}
fn lineage_excess(lineage: &pse_model::diagnostic::ValidityLineage) -> Result<usize> {
    Ok(owned_lineage(lineage)? - lineage_bytes(lineage)?)
}
fn retained_excess(stages: &[StageSnapshot]) -> Result<usize> {
    let mut n = 0;
    for stage in stages {
        n = plus(
            n,
            match stage {
                StageSnapshot::Block(b) => plus(
                    plus(
                        plus(excess(&b.components)?, excess(&b.inputs)?)?,
                        plus(excess(&b.outputs)?, excess(&b.checked_ends)?)?,
                    )?,
                    plus(excess(&b.gather)?, excess(&b.scatter)?)?,
                )?,
                StageSnapshot::Require { lineage, .. } => lineage
                    .as_ref()
                    .map(lineage_excess)
                    .transpose()?
                    .unwrap_or(0),
                StageSnapshot::Domain(d) => plus(
                    plus(retained_excess(&d.stages)?, layout_excess(&d.layout)?)?,
                    plus(excess(&d.reads)?, lineage_excess(&d.lineage)?)?,
                )?,
                StageSnapshot::Applicability(a) => {
                    let plan = plan_extents(&a.plan, a.predicates.len(), a.inputs.len())?;
                    plus(
                        plus(retained_excess(&a.stages)?, layout_excess(&a.layout)?)?,
                        plus(
                            plus(
                                excess(&a.reads)?,
                                plus(excess(&a.predicates)?, excess(&a.inputs)?)?,
                            )?,
                            plan.owned_bytes - plan.clone_bytes,
                        )?,
                    )?
                }
                StageSnapshot::Branch {
                    then, otherwise, ..
                } => plus(retained_excess(then)?, retained_excess(otherwise)?)?,
            },
        )?;
    }
    Ok(n)
}
impl ExecutionSnapshot {
    /// Inspect borrowed data without allocating; this grants no execution authority.
    pub fn inspect(&self) -> Result<TransferExtents> {
        Ok(self.checked_counts()?.facts)
    }
    fn checked_counts(&self) -> Result<Counts> {
        let mut counts = RootView::passive(self)?.inspect()?;
        // Owned decoded metadata moves into the immutable template. Preserve its
        // actual capacities; only new lazy clones request their source lengths.
        let signature = plus(
            plus(
                excess(&self.input_formals)?,
                excess(&self.coordinate_offsets)?,
            )?,
            plus(excess(&self.outputs)?, excess(&self.first_axes)?)?,
        )?;
        let layouts = plus(
            layout_excess(&self.value_layout)?,
            layout_excess(&self.first_layout)?,
        )?;
        let retained = plus(
            plus(signature, layouts)?,
            plus(retained_excess(&self.value)?, retained_excess(&self.first)?)?,
        )?;
        counts.facts.template_bytes = plus(counts.facts.template_bytes, retained)?;
        counts.facts.input_bytes = self.owned_bytes()?;
        counts.facts.restore_peak_bytes = plus(
            plus(counts.facts.input_bytes, counts.facts.restore_peak_bytes)?,
            retained,
        )?;
        Ok(counts)
    }
    fn owned_bytes(&self) -> Result<usize> {
        let signature = plus(
            bytes::<usize>(plus(
                plus(
                    self.input_formals.capacity(),
                    self.coordinate_offsets.capacity(),
                )?,
                plus(self.outputs.capacity(), self.first_axes.capacity())?,
            )?)?,
            bytes::<(usize, usize)>(self.first_rows.capacity())?,
        )?;
        plus(
            plus(size_of::<Self>(), signature)?,
            plus(
                plus(
                    owned_layout(&self.value_layout)?,
                    owned_layout(&self.first_layout)?,
                )?,
                plus(owned_program(&self.value)?, owned_program(&self.first)?)?,
            )?,
        )
    }
}

/// A checked execution frame, deliberately nonserializable and source-independent.
/// Native allocation/construction owners and the actual scope live through drain.
pub struct CheckedExecution {
    pub(crate) frame: NumericFrame,
    facts: TransferExtents,
}
impl std::fmt::Debug for CheckedExecution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckedExecution")
            .field("inputs", &self.frame.template.inputs)
            .field("outputs", &self.frame.template.outputs.len())
            .finish_non_exhaustive()
    }
}
impl CheckedExecution {
    /// Original checked requested populations, never a wire allocation grant.
    pub fn extents(&self) -> &TransferExtents {
        &self.facts
    }
    pub(crate) fn first_axis_slice(&self, row: usize) -> Option<&[usize]> {
        self.frame.transfer_first_axes(row)
    }
    pub(crate) fn retain_owner(&mut self, owner: Arc<dyn AllocationOwner>) {
        self.frame._owner = Some(crate::retain_allocation_owner(
            self.frame._owner.take(),
            owner,
        ));
    }
    pub(crate) fn check_closure_attribution(
        &self,
        members: &BTreeMap<SemanticId, SemanticId>,
        sources: &BTreeMap<SemanticId, SemanticId>,
    ) -> std::result::Result<(), AttributionTransferError> {
        check_closure_attribution(&self.frame, members, sources)
    }
    /// Original authored input IDs in compact numeric order.
    pub fn input_formals(&self) -> &[usize] {
        &self.frame.template.input_formals
    }
    /// Derivative-axis compact input offsets in original requested order.
    pub fn coordinate_offsets(&self) -> &[usize] {
        &self.frame.template.coordinate_offsets
    }
    /// Selected original output population in retained order.
    pub fn output_count(&self) -> usize {
        self.frame.template.outputs.len()
    }
    /// Complete compact logical slot population.
    pub fn slots(&self) -> usize {
        self.frame.template.slots
    }
    /// Ceiling guaranteed by this fixed Value+First profile.
    pub fn order(&self) -> DerivativeOrder {
        DerivativeOrder::First
    }
    /// Mathematical product guaranteed by this fixed profile.
    pub fn product(&self) -> ProductKind {
        ProductKind::SelectedPartials
    }
    /// Original sparse derivative axes retained for one selected output row.
    pub fn first_axes(&self, row: usize) -> impl Iterator<Item = usize> + '_ {
        self.frame
            .template
            .first_layout
            .entries(row)
            .map(|(axis, _)| axis)
    }
    /// Checked immutable coefficient layout; Second is absent in this profile.
    pub fn layout(&self, order: DerivativeOrder) -> Option<&JetLayout> {
        self.frame.template.layouts[order].as_ref()
    }
    /// Derived native point/result cache backing for the enclosing root consumer.
    pub fn evaluation_cache_bytes(&self) -> usize {
        self.frame.template.evaluation_cache_bytes
    }
    /// Derived complete lazy frame/clone backing already covered by the actual owner.
    pub fn worker_bytes(&self) -> usize {
        self.frame.template.worker_bytes
    }
}

pub(crate) fn check_closure_attribution(
    frame: &NumericFrame,
    members: &BTreeMap<SemanticId, SemanticId>,
    sources: &BTreeMap<SemanticId, SemanticId>,
) -> std::result::Result<(), AttributionTransferError> {
    fn lineage(
        lineage: &pse_model::diagnostic::ValidityLineage,
        members: &BTreeMap<SemanticId, SemanticId>,
        sources: &BTreeMap<SemanticId, SemanticId>,
    ) -> std::result::Result<(), AttributionTransferError> {
        if lineage.layer == pse_model::generated::enums::ModelingValidityLayer::Closure {
            for token in &lineage.members {
                if !members.contains_key(token) {
                    return Err(AttributionTransferError::MissingMember(*token));
                }
            }
            if !sources.contains_key(&lineage.source) {
                return Err(AttributionTransferError::MissingSource(lineage.source));
            }
        }
        Ok(())
    }
    fn visit(
        stages: &[CompiledStage],
        members: &BTreeMap<SemanticId, SemanticId>,
        sources: &BTreeMap<SemanticId, SemanticId>,
    ) -> std::result::Result<(), AttributionTransferError> {
        for stage in stages {
            match stage {
                CompiledStage::Require {
                    lineage: Some(value),
                    ..
                } => lineage(value, members, sources)?,
                CompiledStage::Domain {
                    lineage: value,
                    stages,
                    ..
                } => {
                    lineage(value, members, sources)?;
                    visit(stages, members, sources)?;
                }
                CompiledStage::Applicability { stages, .. } => visit(stages, members, sources)?,
                CompiledStage::Branch {
                    then, otherwise, ..
                } => {
                    visit(then, members, sources)?;
                    visit(otherwise, members, sources)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    for program in frame.template.programs.values().flatten() {
        visit(program, members, sources)?;
    }
    Ok(())
}

/// Restore only after actual native admission, retaining both returned allocation
/// ownership and the supplied original construction owner. Serialized limits are
/// mathematical profile data; they cannot admit an allocation or renew a scope.
pub fn restore<E: std::fmt::Debug + std::fmt::Display>(
    input: CapturedExecution,
    scope: pse_kernels::ExecutionScope,
    construction: Arc<dyn ConstructionAdmission>,
    admit: impl FnOnce(&TransferExtents) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
) -> std::result::Result<CheckedExecution, TransferError<E>> {
    // Declare the owner first so rollback drops passive buffers before their lease.
    let input_owner = input.owner;
    let snapshot = input.snapshot;
    scope.check().map_err(crate::error::scope_error)?;
    let counts = snapshot.checked_counts().map_err(widen::<E>)?;
    scope.check().map_err(crate::error::scope_error)?;
    let allocation = admit(&counts.facts).map_err(TransferError::Admission)?;
    scope.check().map_err(crate::error::scope_error)?;
    let frame = (|| -> Result<NumericFrame> {
        let mut index_scratch = reserve(counts.index_scratch)?;
        index_scratch.resize(counts.index_scratch, 0);
        let mut identity_scratch = reserve(counts.identity_scratch)?;
        identity_scratch.resize(counts.identity_scratch, SemanticId::NIL);
        let input_target = &mut index_scratch[..snapshot.inputs];
        input_target.copy_from_slice(&snapshot.input_formals);
        input_target.sort_unstable();
        if input_target.windows(2).any(|p| p[0] == p[1]) {
            return Err(TransferError::Invalid("duplicate input formals"));
        }
        let coordinate_target = &mut index_scratch[..snapshot.coordinate_offsets.len()];
        coordinate_target.copy_from_slice(&snapshot.coordinate_offsets);
        coordinate_target.sort_unstable();
        if coordinate_target.windows(2).any(|p| p[0] == p[1]) {
            return Err(TransferError::Invalid("duplicate coordinate offsets"));
        }
        let ExecutionSnapshot {
            inputs,
            input_formals,
            coordinate_offsets,
            slots,
            outputs,
            limits,
            value_layout,
            first_layout,
            first_rows,
            first_axes,
            value,
            first,
            ..
        } = snapshot;
        let mut layouts = EnumMap::default();
        layouts[DerivativeOrder::Value] =
            Some(value_layout.restore(slots, inputs, limits, &mut index_scratch)?);
        layouts[DerivativeOrder::First] =
            Some(first_layout.restore(slots, inputs, limits, &mut index_scratch)?);
        let mut programs = EnumMap::default();
        programs[DerivativeOrder::Value] = Some(restore_program(
            value,
            limits,
            &mut index_scratch,
            &mut identity_scratch,
        )?);
        programs[DerivativeOrder::First] = Some(restore_program(
            first,
            limits,
            &mut index_scratch,
            &mut identity_scratch,
        )?);
        let first_layout = FirstLayout::checked_restore(
            first_rows,
            first_axes,
            outputs.len(),
            coordinate_offsets.len(),
        )?;
        let mut order_worker_bytes = EnumMap::default();
        let mut census = EnumMap::default();
        for (i, order) in [DerivativeOrder::Value, DerivativeOrder::First]
            .into_iter()
            .enumerate()
        {
            order_worker_bytes[order] = counts.order_bytes[i];
            let c = counts.order_counts[i];
            census[order] = Some(CompiledOrderCensus {
                order,
                directional: false,
                product: ProductKind::SelectedPartials,
                direct_weighted: false,
                stages: c.stages,
                arithmetic_blocks: c.blocks,
                arithmetic_input_entries: c.inputs,
                arithmetic_output_entries: c.outputs,
                native_arithmetic_blocks: 0,
                provider_stages: 0,
                control_stages: c.controls,
                library_operations: c.operations,
                library_instructions: c.instructions,
                instruction_storage_bytes: c.instruction_bytes,
                frame_entries: counts.frame_entries[i],
                derivative_components: layouts[order].as_ref().map_or(0, JetLayout::width),
                second_pairs: 0,
                result_entries: plus(
                    outputs.len(),
                    if i == 1 { first_layout.axes.len() } else { 0 },
                )?,
                worker_bytes: counts.order_bytes[i],
            });
        }
        let template = Arc::new(ExecutionTemplate {
            directional: false,
            product: ProductKind::SelectedPartials,
            direct_product: None,
            reverse_product: None,
            scratch_bytes: plus(counts.numeric_bytes, counts.evidence_bytes)?,
            evaluation_cache_bytes: counts.facts.evaluation_bytes,
            worker_bytes: counts.facts.worker_bytes,
            order_worker_bytes,
            census,
            inputs,
            input_formals: Arc::new(input_formals),
            coordinate_offsets: Arc::new(coordinate_offsets),
            slots,
            outputs: Arc::new(outputs),
            layouts: Arc::new(layouts),
            first_layout: Arc::new(first_layout),
            programs: Arc::new(programs),
            limits,
            required_provider_keys: Arc::new(Vec::new()),
            provider_visit_capacity: 0,
        });
        drop(index_scratch);
        drop(identity_scratch);
        let owner: Arc<dyn AllocationOwner> = Arc::new((input_owner, allocation, construction));
        Ok(template.frame(Some(scope), Some(owner)))
    })()
    .map_err(widen::<E>)?;
    frame.check_scope()?;
    Ok(CheckedExecution {
        frame,
        facts: counts.facts,
    })
}

impl NumericFrame {
    pub(crate) fn transfer_coordinates(&self) -> &[usize] {
        &self.template.coordinate_offsets
    }
    pub(crate) fn transfer_outputs(&self) -> usize {
        self.template.outputs.len()
    }
    pub(crate) fn transfer_first_axes(&self, row: usize) -> Option<&[usize]> {
        self.template
            .first_layout
            .rows
            .get(row)
            .and_then(|range| self.template.first_layout.axes.get(range.clone()))
    }
}

#[cfg(test)]
mod tests;
