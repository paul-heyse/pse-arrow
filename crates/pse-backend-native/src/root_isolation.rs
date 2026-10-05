// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Request-local IBEX validated covering; numerical proposals supply no proof authority.
use pse_ids::ContentHash;
use pse_kernels::DerivativeOrder;
use pse_math::{
    MathError,
    factorable::{
        Constant, Constraint, Fidelity, Node, ObligationKind, ObligationScope, ProjectedObligation,
        PointArithmeticProgram, Rational, RootIsolationProgram,
    },
    guarded::Condition,
    implicit::{
        ChartChainCoverage, ChartChainEvidence, ChartChainProof, ChartChainRequest, ChartChainWork,
        ProofInterval, RootActionEvidence, RootNeighborhoodEvidence, RootPointEvidence,
        SelectionChart, SelectionEvidence, SelectionProofRefusal, SelectionProofRequest,
        SelectionScope, SelectionVerifier,
    },
};
use std::{
    ffi::c_void,
    sync::{
        Arc, Mutex, TryLockError,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const MAX_CELLS: u64 = 1_048_576;
const MAX_NODES: usize = 65_536;
const MAX_EDGES: usize = 1_048_576;
const MAX_GUARDS: usize = 8192;
const MAX_INPUTS: usize = 128;
const MAX_ALTERNATIVES: usize = 64;
const MAX_ARITHMETIC_OUTPUTS: usize = 512;
const MAX_SECOND_INPUTS: usize = 32;
const MAX_SECOND_OUTPUTS: usize = 128;
const MAX_HESSIAN_ENTRIES: usize = 32_768;
// IBEX 2.9.1 has process-global random/contractor state. All calls through this adapter
// share this lock; it owns no retained session or cached evidence.
static IBEX: Mutex<()> = Mutex::new(());

/// The single supported native validated mathematics implementation.
#[derive(Debug, Default)]
pub struct Ibex;
#[repr(C)]
#[derive(Default)]
struct NativeNode {
    op: u32,
    a: u32,
    b: u32,
    begin: u32,
    count: u32,
    power: i32,
    lower: f64,
    upper: f64,
}
#[repr(C)]
struct NativeGuard {
    node: u32,
    kind: u32,
    chart_only: u32,
    domain: u32,
    lower: f64,
    upper: f64,
}
#[repr(C)]
struct NativeRequest {
    nodes: *const NativeNode,
    node_count: u32,
    edges: *const u32,
    edge_count: u32,
    residuals: *const u32,
    unknown_count: u32,
    parameter_count: u32,
    score: u32,
    tolerance: u32,
    guards: *const NativeGuard,
    guard_count: u32,
    lower: *const f64,
    upper: *const f64,
    parameters: *const f64,
    candidate: *const f64,
    max_cells: u64,
    seconds: f64,
    cancelled: extern "C" fn(*const c_void) -> i32,
    cancel_context: *const c_void,
}
#[repr(C)]
#[derive(Default)]
struct NativeChartChainResult {
    status: u32,
    proof_cells: u64,
    charts: u64,
    connections: u64,
}
#[repr(C)]
#[derive(Default)]
struct NativeResult {
    status: u32,
    cells: u64,
    solution: u64,
    boundary: u64,
    unknown: u64,
    pending: u64,
}
/// Actual library calls attempted for an original-point arithmetic operation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PointArithmeticWork {
    /// Guard scalar evaluations (including calls that fail).
    pub guard_evaluations: u64,
    /// Vector value evaluations (at most one).
    pub value_evaluations: u64,
    /// Rectangular First/Jacobian evaluations (at most one).
    pub jacobian_evaluations: u64,
    /// Scalar-output Hessian evaluations (zero for First; at most one per output for Second).
    pub hessian_evaluations: u64,
}
#[repr(C)]
#[derive(Default)]
struct NativePointArithmeticResult {
    status: u32,
    work: PointArithmeticWork,
}
/// Actual outward arithmetic at an original physical point, without a root claim.
#[derive(Clone, Debug, PartialEq)]
pub enum PointArithmeticEvidence {
    /// Every Value and First guard was admitted, with finite outward outputs.
    Enclosed {
        /// Graph output order; row positions are declared by the source projection.
        values: Vec<ProofInterval>,
        /// Dense row-major output-by-original-input First enclosures.
        jacobian: Vec<ProofInterval>,
        /// Requested Second arithmetic, dense output-by-input-by-input in row-major
        /// order. First supplies `None`; no Third arithmetic is implemented.
        hessian: Option<Vec<ProofInterval>>,
        /// Actual attempted native calls.
        work: PointArithmeticWork,
    },
    /// No arithmetic enclosure is supplied; the actual refusal remains visible.
    Incomplete {
        /// Unsupported representation, domain boundary or bounded resources.
        reason: SelectionProofRefusal,
        /// Actual attempted native calls.
        work: PointArithmeticWork,
    },
    /// Cancellation interrupts admission, mutex waiting or native arithmetic.
    Interrupted {
        /// Actual attempted native calls.
        work: PointArithmeticWork,
    },
}
#[expect(
    unsafe_code,
    reason = "bounded request buffers are borrowed synchronously by the catch-all C++ adapter"
)]
unsafe extern "C" {
    fn pse_ibex_point_arithmetic(
        request: *const NativeRequest,
        output_count: u32,
        order: u32,
        result: *mut NativePointArithmeticResult,
        value_lower: *mut f64,
        value_upper: *mut f64,
        jacobian_lower: *mut f64,
        jacobian_upper: *mut f64,
        hessian_lower: *mut f64,
        hessian_upper: *mut f64,
    ) -> i32;
    fn pse_ibex_certify(
        requests: *const NativeRequest,
        count: u32,
        winner: u32,
        result: *mut NativeResult,
        parameter_lower: *mut f64,
        parameter_upper: *mut f64,
        existence_lower: *mut f64,
        existence_upper: *mut f64,
        uniqueness_lower: *mut f64,
        uniqueness_upper: *mut f64,
    ) -> i32;
    fn pse_ibex_promote(
        request: *const NativeRequest,
        result: *mut NativeResult,
        parameter_lower: *const f64,
        parameter_upper: *const f64,
        uniqueness_lower: *const f64,
        uniqueness_upper: *const f64,
    ) -> i32;
    fn pse_ibex_connect(
        request: *const NativeRequest,
        result: *mut NativeResult,
        intersection_lower: *const f64,
        intersection_upper: *const f64,
    ) -> i32;
    fn pse_ibex_enclose_action(
        request: *const NativeRequest,
        result: *mut NativeResult,
        existence_lower: *const f64,
        existence_upper: *const f64,
        uniqueness_lower: *const f64,
        uniqueness_upper: *const f64,
        direction: *const f64,
        action_lower: *mut f64,
        action_upper: *mut f64,
    ) -> i32;
    fn pse_ibex_enclose_neighborhood(
        request: *const NativeRequest,
        result: *mut NativeResult,
        existence_lower: *const f64,
        existence_upper: *const f64,
        uniqueness_lower: *const f64,
        uniqueness_upper: *const f64,
        parameter_lower: *const f64,
        parameter_upper: *const f64,
        direction_lower: *const f64,
        direction_upper: *const f64,
        point_lower: *mut f64,
        point_upper: *mut f64,
        action_lower: *mut f64,
        action_upper: *mut f64,
    ) -> i32;
    fn pse_ibex_refine_point(
        request: *const NativeRequest,
        result: *mut NativeResult,
        existence_lower: *const f64,
        existence_upper: *const f64,
        uniqueness_lower: *const f64,
        uniqueness_upper: *const f64,
        unknown_scales: *const f64,
        row_scales: *const f64,
        root_lower: *mut f64,
        root_upper: *mut f64,
        inverse_norm_upper: *mut f64,
    ) -> i32;
    fn pse_ibex_connect_chain(
        request: *const NativeRequest,
        result: *mut NativeChartChainResult,
        origin: *const f64,
        previous_pl: *const f64,
        previous_pu: *const f64,
        previous_el: *const f64,
        previous_eu: *const f64,
        previous_ul: *const f64,
        previous_uu: *const f64,
        next_pl: *const f64,
        next_pu: *const f64,
        next_el: *const f64,
        next_eu: *const f64,
        next_ul: *const f64,
        next_uu: *const f64,
    ) -> i32;
}
#[expect(
    unsafe_code,
    reason = "callback context points to the caller-owned AtomicBool for the synchronous native call"
)]
extern "C" fn cancelled(context: *const c_void) -> i32 {
    // No allocation, locking or panic can cross this callback boundary.
    if context.is_null() {
        return 1;
    }
    // SAFETY: certify passes an aligned AtomicBool borrowed from its live Arc;
    // the synchronous adapter neither retains this pointer nor calls after return.
    i32::from(unsafe { &*context.cast::<AtomicBool>() }.load(Ordering::Relaxed))
}
fn index(value: usize) -> Result<u32, SelectionProofRefusal> {
    u32::try_from(value).map_err(|_| SelectionProofRefusal::Resource)
}
fn rational_bounds(value: &Rational) -> Result<(f64, f64), SelectionProofRefusal> {
    let center = value.to_f64();
    if !center.is_finite() {
        return Err(SelectionProofRefusal::Unsupported);
    }
    // Symbolica/Numerica owns both exact binary64 conversion and rational ordering.
    let binary = Rational::try_from(center).map_err(|_| SelectionProofRefusal::Unsupported)?;
    let bounds = match binary.cmp(value) {
        std::cmp::Ordering::Equal => (center, center),
        std::cmp::Ordering::Less => (center, center.next_up()),
        std::cmp::Ordering::Greater => (center.next_down(), center),
    };
    if !bounds.0.is_finite() || !bounds.1.is_finite() {
        return Err(SelectionProofRefusal::Unsupported);
    }
    let lower = Rational::try_from(bounds.0).map_err(|_| SelectionProofRefusal::Unsupported)?;
    let upper = Rational::try_from(bounds.1).map_err(|_| SelectionProofRefusal::Unsupported)?;
    // Refuse even an unexpected library conversion distance; never silently round inward.
    if lower > *value || upper < *value {
        return Err(SelectionProofRefusal::Unsupported);
    }
    Ok(bounds)
}
fn constant_bounds(value: &Constant) -> Result<(f64, f64), SelectionProofRefusal> {
    match value {
        Constant::Rational(value) => rational_bounds(value),
        Constant::Float(value) if value.is_finite() => Ok((*value, *value)),
        Constant::Float(_) => Err(SelectionProofRefusal::Unsupported),
    }
}
fn arithmetic_guard_ceiling(program: &RootIsolationProgram, order: DerivativeOrder) -> Option<usize> {
    let guards = program.obligations.iter()
        .chain(program.derivative_obligations.iter().filter(|(minimum,_)| *minimum <= order).map(|(_,g)| g))
        .try_fold(program.eligibility.len(), |n,g| {
            n.checked_add(g.constraints.len())?.checked_add(usize::from(g.argument.is_some()))
        })?;
    program.nodes.iter().try_fold(guards, |n,node| n.checked_add(match node {
        Node::Log(_) => 1,
        Node::Pow { exponent, .. } if exponent.value() == 0.5 => 2,
        Node::Pow { exponent, .. } if exponent.value() < 0.0 => 1,
        _ => 0,
    }))
}
#[expect(unsafe_code,reason = "synchronous point-arithmetic callback borrows its live Execution without retaining it")]
extern "C" fn execution_cancelled(context: *const c_void) -> i32 {
    if context.is_null() { return 1; }
    // SAFETY: the wrapper lends its immutable live Execution for the synchronous
    // call. stopped reads only atomics and clocks, and neither allocates nor panics.
    i32::from(unsafe { &*context.cast::<crate::solve::Execution>() }.stopped().is_some())
}
fn constraint(
    value: &Constraint,
    chart_only: bool,
    domain: bool,
) -> Result<NativeGuard, SelectionProofRefusal> {
    if value.lower.is_nan() || value.upper.is_nan() || value.lower > value.upper {
        return Err(SelectionProofRefusal::Unsupported);
    }
    Ok(NativeGuard {
        node: index(value.expression)?,
        kind: u32::from(value.strict),
        chart_only: u32::from(chart_only),
        domain: u32::from(domain),
        lower: value.lower,
        upper: value.upper,
    })
}
fn obligation(
    value: &ProjectedObligation,
    chart_only: bool,
    guards: &mut Vec<NativeGuard>,
) -> Result<(), SelectionProofRefusal> {
    if !value.represented
        || value.fidelity != Fidelity::Exact
        || value.scope != ObligationScope::Unconditional
    {
        return Err(SelectionProofRefusal::Unsupported);
    }
    for value in &value.constraints {
        guards.push(constraint(value, chart_only, true)?);
    }
    if let ObligationKind::Require(condition) = value.kind {
        let argument = value.argument.ok_or(SelectionProofRefusal::Unsupported)?;
        guards.push(NativeGuard {
            node: index(argument)?,
            kind: match condition {
                Condition::Nonzero => 2,
                Condition::Positive => 3,
                Condition::Nonnegative => 4,
            },
            chart_only: u32::from(chart_only),
            domain: 1,
            lower: 0.0,
            upper: f64::INFINITY,
        });
    }
    Ok(())
}
struct Transport {
    nodes: Vec<NativeNode>,
    edges: Vec<u32>,
    residuals: Vec<u32>,
    guards: Vec<NativeGuard>,
}
fn encode(
    program: &RootIsolationProgram,
    order: DerivativeOrder,
) -> Result<Transport, SelectionProofRefusal> {
    if program.inputs == 0
        || program.inputs > MAX_INPUTS
        || program.nodes.is_empty()
        || program.nodes.len() > MAX_NODES
    {
        return Err(SelectionProofRefusal::Resource);
    }
    let mut transport = Transport {
        nodes: Vec::with_capacity(program.nodes.len()),
        edges: Vec::new(),
        residuals: program
            .residuals
            .iter()
            .copied()
            .map(index)
            .collect::<Result<_, _>>()?,
        guards: Vec::new(),
    };
    let mut depths: Vec<usize> = Vec::with_capacity(program.nodes.len());
    for (ordinal, node) in program.nodes.iter().enumerate() {
        let child = |value: usize| {
            if value >= ordinal {
                Err(SelectionProofRefusal::Unsupported)
            } else {
                index(value)
            }
        };
        let mut result = NativeNode::default();
        match node {
            Node::Var(value) if *value < program.inputs => {
                result.a = index(*value)?;
            }
            Node::Const(value) => {
                result.op = 1;
                (result.lower, result.upper) = constant_bounds(value)?;
            }
            Node::Sum(values) | Node::Product(values) => {
                result.op = if matches!(node, Node::Sum(_)) { 2 } else { 3 };
                result.begin = index(transport.edges.len())?;
                result.count = index(values.len())?;
                for value in values {
                    transport.edges.push(child(*value)?);
                }
            }
            Node::Pow { base, exponent } => {
                result.a = child(*base)?;
                let (lower, upper) = constant_bounds(exponent)?;
                if lower == upper && lower == 0.5 {
                    result.op = 5;
                } else if lower == upper
                    && lower.fract() == 0.0
                    && lower >= f64::from(i32::MIN)
                    && lower <= f64::from(i32::MAX)
                {
                    result.op = 4;
                    result.power = lower as i32;
                } else {
                    return Err(SelectionProofRefusal::Unsupported);
                }
            }
            Node::Exp(value) => {
                result.op = 6;
                result.a = child(*value)?;
            }
            Node::Log(value) => {
                result.op = 7;
                result.a = child(*value)?;
            }
            Node::Sin(value) => {
                result.op = 8;
                result.a = child(*value)?;
            }
            Node::Cos(value) => {
                result.op = 9;
                result.a = child(*value)?;
            }
            Node::Var(_) | Node::Aux(_) | Node::Abs(_) => {
                return Err(SelectionProofRefusal::Unsupported);
            }
        }
        // Intrinsic domains supplement authored guards without replacing their meaning.
        let intrinsic = if result.op == 7 {
            Some(3)
        } else if result.op == 5 {
            Some(4)
        } else if result.op == 4 && result.power < 0 {
            Some(2)
        } else {
            None
        };
        if let Some(kind) = intrinsic {
            transport.guards.push(NativeGuard {
                node: result.a,
                kind,
                chart_only: 0,
                domain: 1,
                lower: 0.0,
                upper: f64::INFINITY,
            });
            if result.op == 5 {
                transport.guards.push(NativeGuard {
                    node: result.a,
                    kind: 3,
                    chart_only: 1,
                    domain: 1,
                    lower: 0.0,
                    upper: f64::INFINITY,
                });
            }
        }
        let depth = match node {
            Node::Var(_) | Node::Const(_) => 1,
            Node::Sum(children) | Node::Product(children) => {
                let deepest = children.iter().map(|id| depths[*id]).max().unwrap_or(0);
                let levels = usize::BITS - children.len().saturating_sub(1).leading_zeros();
                deepest + levels as usize
            }
            Node::Pow { base, .. } => depths[*base] + 1,
            Node::Exp(id) | Node::Log(id) | Node::Sin(id) | Node::Cos(id) => depths[*id] + 1,
            Node::Aux(_) | Node::Abs(_) => return Err(SelectionProofRefusal::Unsupported),
        };
        if depth > 256 {
            return Err(SelectionProofRefusal::Resource);
        }
        depths.push(depth);
        transport.nodes.push(result);
    }
    for value in &program.eligibility {
        transport.guards.push(constraint(value, false, false)?);
    }
    for value in &program.obligations {
        obligation(value, false, &mut transport.guards)?;
    }
    for (minimum, value) in &program.derivative_obligations {
        if *minimum <= order {
            obligation(value, true, &mut transport.guards)?;
        }
    }
    if transport.edges.len() > MAX_EDGES || transport.guards.len() > MAX_GUARDS {
        return Err(SelectionProofRefusal::Resource);
    }
    if program
        .criterion
        .iter()
        .any(|id| *id >= transport.nodes.len())
        || transport
            .residuals
            .iter()
            .any(|id| *id as usize >= transport.nodes.len())
        || transport
            .guards
            .iter()
            .any(|g| g.node as usize >= transport.nodes.len())
    {
        return Err(SelectionProofRefusal::Unsupported);
    }
    // Authored obligations retain their distinct attribution in the source program.
    // Native guard functions need only one copy of an identical predicate and scope.
    let mut guards = std::collections::BTreeSet::new();
    transport.guards.retain(|guard| {
        guards.insert((
            guard.node,
            guard.kind,
            guard.chart_only,
            guard.domain,
            guard.lower.to_bits(),
            guard.upper.to_bits(),
        ))
    });
    Ok(transport)
}
impl Ibex {
    /// Checked, conservative transient admission extent for point transport, complete
    /// interval DAG/guard copies, rectangular AD and output buffers. Source projection
    /// bytes remain owned by the caller. The caller's byte ceiling is enforced before
    /// encoding or entering IBEX; there is no covering allocation in this operation.
    /// # Errors
    /// Resource extents outside the bounded transport or checked-size overflow.
    pub fn point_arithmetic_workspace_bytes(
        &self,
        program: &PointArithmeticProgram,
    ) -> Result<usize, MathError> {
        self.point_arithmetic_workspace_bytes_for_order(program,DerivativeOrder::First)
    }
    /// Checked Value, First or Second arithmetic admission. Second retains every Second
    /// guard and bounds dense output Hessians to 32 inputs, 128 outputs and 32768
    /// entries. The complete source identity is independent of the requested order.
    /// # Errors
    /// Unsupported order or bounded resource/checked-size overflow.
    pub fn point_arithmetic_workspace_bytes_for_order(
        &self,
        program: &PointArithmeticProgram,
        order: DerivativeOrder,
    ) -> Result<usize, MathError> {
        let graph = &program.graph;
        let edges = graph.nodes.iter().try_fold(0usize, |n, node| {
            n.checked_add(match node { Node::Sum(v) | Node::Product(v) => v.len(), _ => 0 })
        });
        let guards = arithmetic_guard_ceiling(graph,order);
        let checked = || {
            let edges = edges?;
            let guards = guards?;
            let outputs = graph.residuals.len(); let width = graph.inputs;
            if width == 0 || width > MAX_INPUTS || outputs == 0 || outputs > MAX_ARITHMETIC_OUTPUTS
                || graph.nodes.is_empty() || graph.nodes.len() > MAX_NODES
                || edges > MAX_EDGES || guards > MAX_GUARDS { return None; }
            let hessian_entries = if order == DerivativeOrder::Second {
                let entries = outputs.checked_mul(width)?.checked_mul(width)?;
                if width > MAX_SECOND_INPUTS || outputs > MAX_SECOND_OUTPUTS || entries > MAX_HESSIAN_ENTRIES {
                    return None;
                }
                entries
            } else { 0 };
            let derivative_width = if order == DerivativeOrder::Second {
                width.checked_mul(width)?.checked_add(width)?.checked_add(1)?
            } else if order == DerivativeOrder::First { width.checked_add(1)? } else { 1 };
            let native_nodes = graph.nodes.len().checked_add(edges)?.checked_add(guards.checked_mul(4)?)?;
            let native = native_nodes.checked_mul(derivative_width)?
                .checked_mul(outputs.checked_add(guards)?.checked_add(4)?)?.checked_mul(1024)?;
            let transport = width.checked_mul(2 * size_of::<f64>())?
                .checked_add(graph.nodes.len().checked_mul(size_of::<NativeNode>() + size_of::<usize>())?)?
                .checked_add(edges.checked_mul(2 * size_of::<u32>())?)?
                .checked_add(guards.checked_mul(4 * size_of::<NativeGuard>() + 128)?)?
                .checked_add(outputs.checked_mul(size_of::<u32>())?)?
                .checked_add(outputs.checked_mul(width.checked_add(1)?)?.checked_add(hessian_entries)?.checked_mul(64)?)?;
            native.checked_add(transport)?.checked_add(1024 * 1024)
        };
        checked().ok_or(MathError::Limit("IBEX point arithmetic workspace extent"))
    }

    /// Enclose all projected original row residuals and First entries at a finite
    /// binary64 physical point. Binary64 inputs are exact dyadic values; arithmetic
    /// and rational conversion uncertainty comes from IBEX/FILIB's outward DAG.
    /// No tolerance, binary spacing, contraction or root search supplies uncertainty.
    /// # Errors
    /// Incorrect point dimensions/nonfinite coordinates or native transport failure.
    pub fn enclose_point_arithmetic(
        &self,
        program: &PointArithmeticProgram,
        point: &[f64],
        time_limit: Duration,
        cancel: &Arc<AtomicBool>,
        max_workspace_bytes: usize,
    ) -> Result<PointArithmeticEvidence, MathError> {
        self.point_arithmetic_core(program,point,point,DerivativeOrder::First,time_limit,cancel,max_workspace_bytes,None)
    }

    /// Reserve the bounded native call ceiling before encoding, then reconcile the
    /// actual calls on every returned enclosure/refusal/interruption. Unreceipted
    /// failure leaves the evaluation counter unknown; no estimate becomes an observation.
    /// The original Execution owns cancellation, abandonment and absolute deadline.
    /// # Errors
    /// Actual execution stop, work admission/reconciliation or arithmetic failure.
    pub fn enclose_point_arithmetic_with_execution(
        &self,
        program: &PointArithmeticProgram,
        point: &[f64],
        max_workspace_bytes: usize,
        execution: &crate::solve::Execution,
    ) -> Result<PointArithmeticEvidence, crate::ProblemError> {
        self.enclose_point_arithmetic_order_with_execution(program,point,DerivativeOrder::First,max_workspace_bytes,execution)
    }
    /// Requested First or Second original-point arithmetic with source-owned work
    /// reservation and actual observations. Second uses IBEX component differentiation
    /// and interval Jacobians; no local AD or Hessian approximation is involved.
    /// # Errors
    /// Actual execution stop, admission/reconciliation or arithmetic failure.
    pub fn enclose_point_arithmetic_order_with_execution(
        &self,
        program: &PointArithmeticProgram,
        point: &[f64],
        order: DerivativeOrder,
        max_workspace_bytes: usize,
        execution: &crate::solve::Execution,
    ) -> Result<PointArithmeticEvidence, crate::ProblemError> {
        execution.check()?;
        if order == DerivativeOrder::Value {
            return Ok(PointArithmeticEvidence::Incomplete {
                reason:SelectionProofRefusal::Unsupported,work:PointArithmeticWork::default(),
            });
        }
        self.arithmetic_with_execution(program,point,point,order,max_workspace_bytes,execution)
    }
    /// Enclose source values on an actual certified coordinate box. First additionally
    /// encloses the library-owned Jacobian uniformly across that same box. Every requested
    /// guard must hold on the whole region; no point gradient is promoted to a region bound.
    /// # Errors
    /// Invalid box, actual execution stop, work admission or transport failure.
    pub fn enclose_arithmetic_box_with_execution(
        &self, program: &PointArithmeticProgram, region: &[ProofInterval], order: DerivativeOrder,
        max_workspace_bytes: usize, execution: &crate::solve::Execution,
    ) -> Result<PointArithmeticEvidence, crate::ProblemError> {
        execution.check()?;
        if order == DerivativeOrder::Second {
            return Ok(PointArithmeticEvidence::Incomplete { reason: SelectionProofRefusal::Unsupported,
                work: PointArithmeticWork::default() });
        }
        if region.len() != program.graph.inputs || region.iter().any(|interval| !interval.valid()) {
            return Err(MathError::Contract("IBEX arithmetic box coordinates".into()).into());
        }
        if self.point_arithmetic_workspace_bytes_for_order(program,order).map_or(true, |bytes| bytes > max_workspace_bytes) {
            return Ok(PointArithmeticEvidence::Incomplete { reason: SelectionProofRefusal::Resource,
                work: PointArithmeticWork::default() });
        }
        let lower = region.iter().map(|interval| interval.lower).collect::<Vec<_>>();
        let upper = region.iter().map(|interval| interval.upper).collect::<Vec<_>>();
        self.arithmetic_with_execution(program,&lower,&upper,order,max_workspace_bytes,execution)
    }
    /// Value-only convenience for selected authored outputs over certified coordinates.
    /// # Errors
    /// Invalid box, actual execution stop, work admission or transport failure.
    pub fn enclose_arithmetic_box_values_with_execution(
        &self, program: &PointArithmeticProgram, region: &[ProofInterval], max_workspace_bytes: usize,
        execution: &crate::solve::Execution,
    ) -> Result<PointArithmeticEvidence, crate::ProblemError> {
        self.enclose_arithmetic_box_with_execution(program,region,DerivativeOrder::Value,max_workspace_bytes,execution)
    }
    fn arithmetic_with_execution(
        &self, program: &PointArithmeticProgram, lower: &[f64], upper: &[f64], order: DerivativeOrder,
        max_workspace_bytes: usize, execution: &crate::solve::Execution,
    ) -> Result<PointArithmeticEvidence, crate::ProblemError> {
        use crate::solve::WorkEvidence;
        execution.check()?;
        // A preflight resource refusal allocates no transport and performs no arithmetic.
        if self.point_arithmetic_workspace_bytes_for_order(program,order).map_or(true, |bytes| bytes > max_workspace_bytes) {
            return Ok(PointArithmeticEvidence::Incomplete {
                reason:SelectionProofRefusal::Resource,work:PointArithmeticWork::default(),
            });
        }
        let ceiling = arithmetic_guard_ceiling(&program.graph,order)
            .and_then(|guards| guards.checked_add(if order == DerivativeOrder::Value {1} else {2}))
            .and_then(|n| n.checked_add(if order == DerivativeOrder::Second {program.graph.residuals.len()} else {0}))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(MathError::Limit("IBEX point arithmetic work extent"))?;
        let unit = |evaluations| WorkEvidence {
            evaluations,iterations:Some(0),factorizations:Some(0),proof_steps:Some(0),
        };
        if let Some(owner) = &execution.work_admission {
            crate::quality::contained(|| owner.admit(unit(Some(ceiling))))?;
        }
        let result = crate::quality::contained(|| {
            let remaining = execution.scope()?.remaining(execution.time_limit)?;
            self.point_arithmetic_core(program,lower,upper,order,remaining,&execution.cancel,max_workspace_bytes,Some(execution))
                .map_err(crate::ProblemError::from)
        });
        let evaluations = result.as_ref().ok().map(|evidence| {
            let work = match evidence {
                PointArithmeticEvidence::Enclosed {work,..} | PointArithmeticEvidence::Incomplete {work,..}
                    | PointArithmeticEvidence::Interrupted {work} => work,
            };
            work.guard_evaluations + work.value_evaluations + work.jacobian_evaluations + work.hessian_evaluations
        });
        let reconciled = execution.work_admission.as_ref().map_or(Ok(()), |owner|
            crate::quality::contained(|| owner.observe(unit(evaluations))));
        let evidence = result?;
        reconciled?;
        execution.check()?;
        Ok(evidence)
    }
    fn point_arithmetic_core(
        &self,
        program: &PointArithmeticProgram,
        lower: &[f64],
        upper: &[f64],
        order: DerivativeOrder,
        time_limit: Duration,
        cancel: &Arc<AtomicBool>,
        max_workspace_bytes: usize,
        execution: Option<&crate::solve::Execution>,
    ) -> Result<PointArithmeticEvidence, MathError> {
        let started = Instant::now();
        let empty_work = PointArithmeticWork::default();
        let incomplete = |reason,work| PointArithmeticEvidence::Incomplete {reason,work};
        if cancel.load(Ordering::Acquire) { return Ok(PointArithmeticEvidence::Interrupted {work:empty_work}); }
        if lower.len() != program.graph.inputs || upper.len() != lower.len()
            || lower.iter().zip(upper).any(|(l,u)| !l.is_finite() || !u.is_finite() || l > u) {
            return Err(MathError::Contract("IBEX original point arithmetic coordinates".into()));
        }
        if time_limit.is_zero() || self.point_arithmetic_workspace_bytes_for_order(program,order)
            .map_or(true, |bytes| bytes > max_workspace_bytes) {
            return Ok(incomplete(SelectionProofRefusal::Resource,empty_work));
        }
        let transport = match encode(&program.graph,order) {
            Ok(t) => t, Err(reason) => return Ok(incomplete(reason,empty_work)),
        };
        let library_guard = loop {
            if cancel.load(Ordering::Acquire) { return Ok(PointArithmeticEvidence::Interrupted {work:empty_work}); }
            if started.elapsed() >= time_limit { return Ok(incomplete(SelectionProofRefusal::Resource,empty_work)); }
            if execution.is_some_and(|e| e.stopped().is_some()) {
                return Ok(incomplete(SelectionProofRefusal::Resource,empty_work));
            }
            match IBEX.try_lock() {
                Ok(g) => break g,
                Err(TryLockError::Poisoned(_)) => return Ok(incomplete(SelectionProofRefusal::Resource,empty_work)),
                Err(TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(1)),
            }
        };
        let outputs = transport.residuals.len();
        let entries = if order == DerivativeOrder::Value {0} else {outputs * lower.len()}; // Bounded before encoding.
        let mut vl = vec![0.0;outputs]; let mut vu = vec![0.0;outputs];
        let mut jl = vec![0.0;entries]; let mut ju = vec![0.0;entries];
        let second = order == DerivativeOrder::Second;
        let hessian_entries = if second {entries * lower.len()} else {0}; // Second preflight checked this product.
        let mut hl = vec![0.0;hessian_entries]; let mut hu = vec![0.0;hessian_entries];
        let native = NativeRequest {
            nodes: transport.nodes.as_ptr(),node_count: transport.nodes.len() as u32,
            edges: transport.edges.as_ptr(),edge_count: transport.edges.len() as u32,
            residuals: transport.residuals.as_ptr(),unknown_count: lower.len() as u32,parameter_count: 0,
            score: program.graph.criterion[0] as u32,tolerance: program.graph.criterion[1] as u32,
            guards: transport.guards.as_ptr(),guard_count: transport.guards.len() as u32,
            lower: lower.as_ptr(),upper: upper.as_ptr(),parameters: lower.as_ptr(),candidate: lower.as_ptr(),
            max_cells: 0,seconds: time_limit.saturating_sub(started.elapsed()).as_secs_f64(),
            cancelled: if execution.is_some() { execution_cancelled } else { cancelled },
            cancel_context: execution.map_or(std::ptr::from_ref(cancel.as_ref()).cast(), |e| std::ptr::from_ref(e).cast()),
        };
        let mut result = NativePointArithmeticResult::default();
        #[expect(unsafe_code,reason = "bounded borrowed point/DAG/output buffers enter serialized synchronous caught IBEX arithmetic")]
        // SAFETY: bounded dimensions and all borrowed buffers remain live through the
        // serialized synchronous adapter; C++ catches every exception and retains nothing.
        let code = unsafe { pse_ibex_point_arithmetic(&native,outputs as u32,match order {
            DerivativeOrder::Value => 0, DerivativeOrder::First => 1, DerivativeOrder::Second => 2 },&mut result,
            vl.as_mut_ptr(),vu.as_mut_ptr(),jl.as_mut_ptr(),ju.as_mut_ptr(),hl.as_mut_ptr(),hu.as_mut_ptr()) };
        drop(library_guard);
        let work = result.work;
        if cancel.load(Ordering::Acquire) { return Ok(PointArithmeticEvidence::Interrupted {work}); }
        if started.elapsed() >= time_limit { return Ok(incomplete(SelectionProofRefusal::Resource,work)); }
        if code != 0 { return Err(MathError::Contract("IBEX point arithmetic transport".into())); }
        if result.status != 0 {
            return Ok(incomplete(match result.status {
                4 => SelectionProofRefusal::Resource,5 => SelectionProofRefusal::Boundary,
                _ => SelectionProofRefusal::Unsupported,
            },work));
        }
        let intervals = |lower:Vec<f64>,upper:Vec<f64>| lower.into_iter().zip(upper)
            .map(|(lower,upper)| ProofInterval {lower,upper}).collect::<Vec<_>>();
        let values = intervals(vl,vu); let jacobian = intervals(jl,ju);
        let hessian = second.then(|| intervals(hl,hu));
        if work.guard_evaluations > transport.guards.len() as u64
            || work.value_evaluations != 1 || work.jacobian_evaluations != u64::from(order != DerivativeOrder::Value)
            || work.hessian_evaluations != if second {outputs as u64} else {0}
            || values.iter().chain(&jacobian).chain(hessian.iter().flatten()).any(|i| !i.valid()) {
            return Err(MathError::Contract("IBEX point arithmetic result extent".into()));
        }
        Ok(PointArithmeticEvidence::Enclosed {values,jacobian,hessian,work})
    }

    /// Actual native ceiling for attempted interval proof cells in one chain. The
    /// shared outer time/cancellation limit may refuse earlier; this is no guarantee
    /// of certification. Only root-sheet coverage is currently implemented.
    pub const fn chart_chain_cell_limit(&self) -> u64 {
        MAX_CELLS
    }
}
impl SelectionVerifier for Ibex {
    fn identity(&self) -> ContentHash {
        pse_math::implicit::solver_identity(&format!(
            "ibex.competitive-selection.hc4-unknown-shaving.guarded-smear-sum-relative.v10:{}",
            include_str!(concat!(env!("OUT_DIR"), "/root-isolation-manifest.json"))
        ))
    }
    fn workspace_bytes(&self, programs: &[Arc<RootIsolationProgram>]) -> Result<usize, MathError> {
        self.bounded_workspace_bytes(programs, MAX_CELLS)
    }
    fn bounded_workspace_bytes(
        &self,
        programs: &[Arc<RootIsolationProgram>],
        max_cells: u64,
    ) -> Result<usize, MathError> {
        let max_cells = max_cells.min(MAX_CELLS);
        // All Rust transports coexist; native graphs and coverings are sequential.
        // Source DAGs already belong to the factory and are not charged twice.
        let checked = || {
            let mut transports = 0usize;
            let mut native = 0usize;
            for program in programs {
                let width = program.inputs;
                let guards = program
                    .obligations
                    .iter()
                    .chain(
                        program
                            .derivative_obligations
                            .iter()
                            .map(|(_, value)| value),
                    )
                    .try_fold(program.eligibility.len(), |total, value| {
                        total
                            .checked_add(value.constraints.len())?
                            .checked_add(usize::from(value.argument.is_some()))
                    })?
                    .checked_add(program.nodes.len().checked_mul(2)?)?;
                let edges = program.nodes.iter().try_fold(0usize, |total, node| {
                    total.checked_add(match node {
                        Node::Sum(values) | Node::Product(values) => values.len(),
                        _ => 0,
                    })
                })?;
                // Vec growth, encoding depths, output buffers and original bounds.
                let transport = program
                    .nodes
                    .len()
                    .checked_mul(size_of::<NativeNode>() + size_of::<usize>())?
                    .checked_add(edges.checked_mul(2 * size_of::<u32>())?)?
                    .checked_add(guards.checked_mul(4 * size_of::<NativeGuard>() + 128)?)?
                    .checked_add(program.residuals.len().checked_mul(size_of::<u32>())?)?
                    .checked_add(width.checked_mul(128)?)?
                    .checked_add(size_of::<NativeRequest>() + size_of::<Transport>() + 4096)?;
                transports = transports.checked_add(transport)?;
                let native_nodes = program
                    .nodes
                    .len()
                    .checked_add(edges)?
                    .checked_add(guards.checked_mul(4)?)?;
                let graph = width
                    .checked_mul(width)?
                    .checked_mul(64)?
                    .checked_add(guards.checked_mul(256)?)?
                    .checked_add(4096)?
                    .checked_mul(native_nodes)?;
                let covering = (max_cells as usize + 1)
                    .checked_mul(width.checked_mul(32)?.checked_add(4096)?)?;
                native = native.max(graph.checked_add(covering)?);
            }
            transports
                .checked_add(native)?
                .checked_add(16 * 1024 * 1024)
        };
        checked().ok_or_else(|| MathError::Contract("IBEX workspace extent overflow".into()))
    }

    fn promote(
        &self,
        request: &SelectionProofRequest<'_>,
        chart: &SelectionChart,
    ) -> Result<SelectionEvidence, MathError> {
        chart.validate_scope(request, self.identity())?;
        if chart.order >= request.order {
            return Ok(SelectionEvidence::Unique(chart.clone()));
        }
        let started = Instant::now();
        let checkpoint = || {
            if request.cancel.load(Ordering::Acquire) {
                Err(MathError::Cancelled)
            } else {
                Ok(started.elapsed() < request.time_limit)
            }
        };
        let winner = &request.alternatives[request.winner];
        let transport = match encode(winner.program, request.order.max(DerivativeOrder::First)) {
            Ok(transport) => transport,
            Err(reason) => return Ok(SelectionEvidence::Incomplete(reason)),
        };
        let library_guard = loop {
            if !checkpoint()? {
                return Ok(SelectionEvidence::Incomplete(
                    SelectionProofRefusal::Resource,
                ));
            }
            match IBEX.try_lock() {
                Ok(guard) => break guard,
                Err(TryLockError::Poisoned(_)) => {
                    return Ok(SelectionEvidence::Incomplete(
                        SelectionProofRefusal::Resource,
                    ));
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        };
        let lower = winner.unknowns.iter().map(|u| u.lower).collect::<Vec<_>>();
        let upper = winner.unknowns.iter().map(|u| u.upper).collect::<Vec<_>>();
        let pl = chart.parameters.iter().map(|i| i.lower).collect::<Vec<_>>();
        let pu = chart.parameters.iter().map(|i| i.upper).collect::<Vec<_>>();
        let ul = chart.uniqueness.iter().map(|i| i.lower).collect::<Vec<_>>();
        let uu = chart.uniqueness.iter().map(|i| i.upper).collect::<Vec<_>>();
        let native = NativeRequest {
            nodes: transport.nodes.as_ptr(),
            node_count: transport.nodes.len() as u32,
            edges: transport.edges.as_ptr(),
            edge_count: transport.edges.len() as u32,
            residuals: transport.residuals.as_ptr(),
            unknown_count: winner.unknowns.len() as u32,
            parameter_count: request.parameters.len() as u32,
            score: winner.program.criterion[0] as u32,
            tolerance: winner.program.criterion[1] as u32,
            guards: transport.guards.as_ptr(),
            guard_count: transport.guards.len() as u32,
            lower: lower.as_ptr(),
            upper: upper.as_ptr(),
            parameters: request.parameters.as_ptr(),
            candidate: request.candidate.as_ptr(),
            max_cells: MAX_CELLS,
            seconds: request
                .time_limit
                .saturating_sub(started.elapsed())
                .as_secs_f64(),
            cancelled,
            cancel_context: std::ptr::from_ref(request.cancel.as_ref()).cast(),
        };
        let mut result = NativeResult::default();
        // SAFETY: all dimensions were validated against the original chart/request;
        // owned buffers and cancellation Arc live across the synchronous guarded call.
        #[expect(
            unsafe_code,
            reason = "synchronous IBEX chart promotion with borrowed validated buffers"
        )]
        // SAFETY: validated dimensions, owned buffers and cancellation Arc remain live
        // through the serialized synchronous IBEX call.
        let code = unsafe {
            pse_ibex_promote(
                &native,
                &mut result,
                pl.as_ptr(),
                pu.as_ptr(),
                ul.as_ptr(),
                uu.as_ptr(),
            )
        };
        drop(library_guard);
        if !checkpoint()? {
            return Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Resource,
            ));
        }
        if code != 0 {
            return Err(MathError::Contract("IBEX chart promotion transport".into()));
        }
        match result.status {
            0 if result.solution == 1 => {
                let mut promoted = chart.clone();
                promoted.order = request.order;
                promoted.validate(request, self.identity())?;
                Ok(SelectionEvidence::Unique(promoted))
            }
            4 => Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Resource,
            )),
            5 => Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Boundary,
            )),
            6 => Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Unsupported,
            )),
            _ => Ok(SelectionEvidence::Incomplete(SelectionProofRefusal::Chart)),
        }
    }
    fn connect(
        &self,
        request: &SelectionProofRequest<'_>,
        previous: &SelectionChart,
        next: &SelectionChart,
    ) -> Result<bool, MathError> {
        previous.validate_scope(request, self.identity())?;
        next.validate_scope(request, self.identity())?;
        let started = Instant::now();
        let checkpoint = || {
            if request.cancel.load(Ordering::Acquire) {
                Err(MathError::Cancelled)
            } else {
                Ok(started.elapsed() < request.time_limit)
            }
        };
        let winner = &request.alternatives[request.winner];
        let transport = match encode(winner.program, request.order.max(DerivativeOrder::First)) {
            Ok(t) => t,
            Err(_) => return Ok(false),
        };
        let lower = winner.unknowns.iter().map(|u| u.lower).collect::<Vec<_>>();
        let upper = winner.unknowns.iter().map(|u| u.upper).collect::<Vec<_>>();
        let intersection_lower = previous
            .uniqueness
            .iter()
            .zip(&next.uniqueness)
            .map(|(a, b)| a.lower.max(b.lower))
            .collect::<Vec<_>>();
        let intersection_upper = previous
            .uniqueness
            .iter()
            .zip(&next.uniqueness)
            .map(|(a, b)| a.upper.min(b.upper))
            .collect::<Vec<_>>();
        if intersection_lower
            .iter()
            .zip(&intersection_upper)
            .any(|(l, u)| l >= u)
        {
            return Ok(false);
        }
        let library_guard = loop {
            if !checkpoint()? {
                return Ok(false);
            }
            match IBEX.try_lock() {
                Ok(g) => break g,
                Err(TryLockError::Poisoned(_)) => return Ok(false),
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        };
        let native = NativeRequest {
            nodes: transport.nodes.as_ptr(),
            node_count: transport.nodes.len() as u32,
            edges: transport.edges.as_ptr(),
            edge_count: transport.edges.len() as u32,
            residuals: transport.residuals.as_ptr(),
            unknown_count: winner.unknowns.len() as u32,
            parameter_count: request.parameters.len() as u32,
            score: winner.program.criterion[0] as u32,
            tolerance: winner.program.criterion[1] as u32,
            guards: transport.guards.as_ptr(),
            guard_count: transport.guards.len() as u32,
            lower: lower.as_ptr(),
            upper: upper.as_ptr(),
            parameters: request.parameters.as_ptr(),
            candidate: request.candidate.as_ptr(),
            max_cells: MAX_CELLS,
            seconds: request
                .time_limit
                .saturating_sub(started.elapsed())
                .as_secs_f64(),
            cancelled,
            cancel_context: std::ptr::from_ref(request.cancel.as_ref()).cast(),
        };
        let mut result = NativeResult::default();
        // SAFETY: chart/request checks establish all coordinate extents; borrowed
        // buffers remain live throughout the synchronous serialized native operation.
        #[expect(
            unsafe_code,
            reason = "synchronous IBEX common-root connection with validated borrowed buffers"
        )]
        // SAFETY: validated dimensions, owned buffers and cancellation Arc remain live
        // through the serialized synchronous IBEX call.
        let code = unsafe {
            pse_ibex_connect(
                &native,
                &mut result,
                intersection_lower.as_ptr(),
                intersection_upper.as_ptr(),
            )
        };
        drop(library_guard);
        if !checkpoint()? {
            return Ok(false);
        }
        if code != 0 {
            return Err(MathError::Contract(
                "IBEX common-root connection transport".into(),
            ));
        }
        Ok(result.status == 0 && result.solution == 1)
    }
    fn refine_point(
        &self,
        request: &SelectionProofRequest<'_>,
        chart: &SelectionChart,
        unknown_scales: &[f64],
        row_scales: &[f64],
        max_cells: u64,
    ) -> Result<RootPointEvidence, MathError> {
        chart.validate(request, self.identity())?;
        let winner = &request.alternatives[request.winner];
        let n = winner.unknowns.len();
        if request.order < DerivativeOrder::First {
            return Err(MathError::Contract(
                "IBEX fixed-point inverse requires original First support".into(),
            ));
        }
        if unknown_scales.len() != n
            || row_scales.len() != n
            || unknown_scales
                .iter()
                .chain(row_scales)
                .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err(MathError::Contract(
                "IBEX fixed-point physical normalization".into(),
            ));
        }
        let started = Instant::now();
        if request.cancel.load(Ordering::Acquire) {
            return Ok(RootPointEvidence::Interrupted { proof_cells: 0 });
        }
        if request.time_limit.is_zero() || max_cells == 0 {
            return Ok(RootPointEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: 0,
            });
        }
        let transport = match encode(winner.program, DerivativeOrder::First) {
            Ok(t) => t,
            Err(reason) => {
                return Ok(RootPointEvidence::Incomplete {
                    reason,
                    proof_cells: 0,
                });
            }
        };
        let lower = winner.unknowns.iter().map(|u| u.lower).collect::<Vec<_>>();
        let upper = winner.unknowns.iter().map(|u| u.upper).collect::<Vec<_>>();
        let el = chart.existence.iter().map(|i| i.lower).collect::<Vec<_>>();
        let eu = chart.existence.iter().map(|i| i.upper).collect::<Vec<_>>();
        let ul = chart.uniqueness.iter().map(|i| i.lower).collect::<Vec<_>>();
        let uu = chart.uniqueness.iter().map(|i| i.upper).collect::<Vec<_>>();
        let mut root_lower = vec![0.0; n];
        let mut root_upper = vec![0.0; n];
        let mut inverse_norm_upper = 0.0;
        let library_guard = loop {
            if request.cancel.load(Ordering::Acquire) {
                return Ok(RootPointEvidence::Interrupted { proof_cells: 0 });
            }
            if started.elapsed() >= request.time_limit {
                return Ok(RootPointEvidence::Incomplete {
                    reason: SelectionProofRefusal::Resource,
                    proof_cells: 0,
                });
            }
            match IBEX.try_lock() {
                Ok(g) => break g,
                Err(TryLockError::Poisoned(_)) => {
                    return Ok(RootPointEvidence::Incomplete {
                        reason: SelectionProofRefusal::Resource,
                        proof_cells: 0,
                    });
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        };
        let native = NativeRequest {
            nodes: transport.nodes.as_ptr(),
            node_count: transport.nodes.len() as u32,
            edges: transport.edges.as_ptr(),
            edge_count: transport.edges.len() as u32,
            residuals: transport.residuals.as_ptr(),
            unknown_count: n as u32,
            parameter_count: request.parameters.len() as u32,
            score: winner.program.criterion[0] as u32,
            tolerance: winner.program.criterion[1] as u32,
            guards: transport.guards.as_ptr(),
            guard_count: transport.guards.len() as u32,
            lower: lower.as_ptr(),
            upper: upper.as_ptr(),
            parameters: request.parameters.as_ptr(),
            candidate: request.candidate.as_ptr(),
            max_cells: max_cells.min(MAX_CELLS),
            seconds: request
                .time_limit
                .saturating_sub(started.elapsed())
                .as_secs_f64(),
            cancelled,
            cancel_context: std::ptr::from_ref(request.cancel.as_ref()).cast(),
        };
        let mut result = NativeResult::default();
        // SAFETY: exact chart scope, dimensions and finite positive physical scales
        // are checked above; borrowed buffers live across this serialized caught call.
        #[expect(
            unsafe_code,
            reason = "synchronous IBEX fixed-parameter enclosure and uniform interval inverse with validated borrowed buffers"
        )]
        // SAFETY: validated dimensions, owned buffers and cancellation Arc remain live
        // through the serialized synchronous IBEX call.
        let code = unsafe {
            pse_ibex_refine_point(
                &native,
                &mut result,
                el.as_ptr(),
                eu.as_ptr(),
                ul.as_ptr(),
                uu.as_ptr(),
                unknown_scales.as_ptr(),
                row_scales.as_ptr(),
                root_lower.as_mut_ptr(),
                root_upper.as_mut_ptr(),
                &mut inverse_norm_upper,
            )
        };
        drop(library_guard);
        if request.cancel.load(Ordering::Acquire) {
            return Ok(RootPointEvidence::Interrupted {
                proof_cells: result.cells,
            });
        }
        if started.elapsed() >= request.time_limit {
            return Ok(RootPointEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: result.cells,
            });
        }
        if code != 0 {
            return Err(MathError::Contract(
                "IBEX fixed-point enclosure transport".into(),
            ));
        }
        let intervals = root_lower
            .into_iter()
            .zip(root_upper)
            .map(|(lower, upper)| ProofInterval { lower, upper })
            .collect::<Vec<_>>();
        if result.status == 0
            && result.solution == 1
            && result.cells > 0
            && result.cells <= max_cells.min(MAX_CELLS)
            && inverse_norm_upper.is_finite()
            && inverse_norm_upper > 0.0
            && intervals
                .iter()
                .zip(&chart.existence)
                .all(|(local, original)| {
                    local.valid() && local.lower >= original.lower && local.upper <= original.upper
                })
        {
            return Ok(RootPointEvidence::Enclosed {
                intervals,
                inverse_norm_upper,
                proof_cells: result.cells,
            });
        }
        let reason = match result.status {
            4 => SelectionProofRefusal::Resource,
            5 => SelectionProofRefusal::Boundary,
            6 => SelectionProofRefusal::Unsupported,
            _ => SelectionProofRefusal::Chart,
        };
        Ok(RootPointEvidence::Incomplete {
            reason,
            proof_cells: result.cells,
        })
    }
    fn enclose_action(
        &self,
        request: &SelectionProofRequest<'_>,
        chart: &SelectionChart,
        direction: &[f64],
    ) -> Result<RootActionEvidence, MathError> {
        self.enclose_action_bounded(request, chart, direction, MAX_CELLS)
    }
    fn enclose_action_bounded(
        &self,
        request: &SelectionProofRequest<'_>,
        chart: &SelectionChart,
        direction: &[f64],
        max_cells: u64,
    ) -> Result<RootActionEvidence, MathError> {
        chart.validate(request, self.identity())?;
        if request.order < DerivativeOrder::First
            || direction.len() != request.parameters.len()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "IBEX selected action direction/order".into(),
            ));
        }
        let started = Instant::now();
        let checkpoint = || {
            if request.cancel.load(Ordering::Acquire) {
                Err(MathError::Cancelled)
            } else {
                Ok(started.elapsed() < request.time_limit)
            }
        };
        if max_cells == 0 || !checkpoint()? {
            return Ok(RootActionEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: 0,
            });
        }
        let winner = &request.alternatives[request.winner];
        let transport = match encode(winner.program, DerivativeOrder::First) {
            Ok(t) => t,
            Err(reason) => {
                return Ok(RootActionEvidence::Incomplete {
                    reason,
                    proof_cells: 0,
                });
            }
        };
        let lower = winner.unknowns.iter().map(|u| u.lower).collect::<Vec<_>>();
        let upper = winner.unknowns.iter().map(|u| u.upper).collect::<Vec<_>>();
        let el = chart.existence.iter().map(|i| i.lower).collect::<Vec<_>>();
        let eu = chart.existence.iter().map(|i| i.upper).collect::<Vec<_>>();
        let ul = chart.uniqueness.iter().map(|i| i.lower).collect::<Vec<_>>();
        let uu = chart.uniqueness.iter().map(|i| i.upper).collect::<Vec<_>>();
        let mut action_lower = vec![0.0; el.len()];
        let mut action_upper = vec![0.0; el.len()];
        let library_guard = loop {
            if !checkpoint()? {
                return Ok(RootActionEvidence::Incomplete {
                    reason: SelectionProofRefusal::Resource,
                    proof_cells: 0,
                });
            }
            match IBEX.try_lock() {
                Ok(g) => break g,
                Err(TryLockError::Poisoned(_)) => {
                    return Ok(RootActionEvidence::Incomplete {
                        reason: SelectionProofRefusal::Resource,
                        proof_cells: 0,
                    });
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        };
        let native = NativeRequest {
            nodes: transport.nodes.as_ptr(),
            node_count: transport.nodes.len() as u32,
            edges: transport.edges.as_ptr(),
            edge_count: transport.edges.len() as u32,
            residuals: transport.residuals.as_ptr(),
            unknown_count: winner.unknowns.len() as u32,
            parameter_count: request.parameters.len() as u32,
            score: winner.program.criterion[0] as u32,
            tolerance: winner.program.criterion[1] as u32,
            guards: transport.guards.as_ptr(),
            guard_count: transport.guards.len() as u32,
            lower: lower.as_ptr(),
            upper: upper.as_ptr(),
            parameters: request.parameters.as_ptr(),
            candidate: request.candidate.as_ptr(),
            max_cells: max_cells.min(MAX_CELLS),
            seconds: request
                .time_limit
                .saturating_sub(started.elapsed())
                .as_secs_f64(),
            cancelled,
            cancel_context: std::ptr::from_ref(request.cancel.as_ref()).cast(),
        };
        let mut result = NativeResult::default();
        // SAFETY: scope validation establishes finite original coordinate extents;
        // all input/output buffers live across the serialized synchronous caught ABI.
        #[expect(
            unsafe_code,
            reason = "synchronous IBEX interval IFT action with validated borrowed buffers"
        )]
        // SAFETY: validated dimensions, owned buffers and cancellation Arc remain live
        // through the serialized synchronous IBEX call.
        let code = unsafe {
            pse_ibex_enclose_action(
                &native,
                &mut result,
                el.as_ptr(),
                eu.as_ptr(),
                ul.as_ptr(),
                uu.as_ptr(),
                direction.as_ptr(),
                action_lower.as_mut_ptr(),
                action_upper.as_mut_ptr(),
            )
        };
        drop(library_guard);
        if request.cancel.load(Ordering::Acquire) {
            return Ok(RootActionEvidence::Interrupted {
                proof_cells: result.cells,
            });
        }
        if !checkpoint()? {
            return Ok(RootActionEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: result.cells,
            });
        }
        if code != 0 {
            return Err(MathError::Contract("IBEX selected action transport".into()));
        }
        let intervals = action_lower
            .into_iter()
            .zip(action_upper)
            .map(|(lower, upper)| ProofInterval { lower, upper })
            .collect::<Vec<_>>();
        if result.status == 0
            && result.solution == 1
            && result.cells > 0
            && result.cells <= max_cells.min(MAX_CELLS)
            && intervals.iter().all(|i| i.valid())
        {
            return Ok(RootActionEvidence::Enclosed {
                intervals,
                proof_cells: result.cells,
            });
        }
        let reason = match result.status {
            4 => SelectionProofRefusal::Resource,
            5 => SelectionProofRefusal::Boundary,
            6 => SelectionProofRefusal::Unsupported,
            _ => SelectionProofRefusal::Chart,
        };
        Ok(RootActionEvidence::Incomplete {
            reason,
            proof_cells: result.cells,
        })
    }
    fn enclose_neighborhood(
        &self,
        request: &SelectionProofRequest<'_>,
        chart: &SelectionChart,
        parameters: &[ProofInterval],
        directions: Option<&[ProofInterval]>,
        max_cells: u64,
    ) -> Result<RootNeighborhoodEvidence, MathError> {
        chart.validate(request, self.identity())?;
        if request.order < DerivativeOrder::First
            || parameters.len() != request.parameters.len()
            || directions
                .is_some_and(|v| v.len() != parameters.len() || v.iter().any(|i| !i.valid()))
            || parameters
                .iter()
                .zip(&chart.parameters)
                .zip(request.parameters)
                .any(|((interval, certified), point)| {
                    !interval.contains(*point)
                        || interval.lower < certified.lower
                        || interval.upper > certified.upper
                })
        {
            return Ok(RootNeighborhoodEvidence::Incomplete {
                reason: SelectionProofRefusal::Coverage,
                proof_cells: 0,
            });
        }
        let started = Instant::now();
        let checkpoint = || {
            if request.cancel.load(Ordering::Acquire) {
                Err(MathError::Cancelled)
            } else {
                Ok(started.elapsed() < request.time_limit)
            }
        };
        if max_cells == 0 || !checkpoint()? {
            return Ok(RootNeighborhoodEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: 0,
            });
        }
        let winner = &request.alternatives[request.winner];
        let transport = match encode(winner.program, DerivativeOrder::First) {
            Ok(t) => t,
            Err(reason) => {
                return Ok(RootNeighborhoodEvidence::Incomplete {
                    reason,
                    proof_cells: 0,
                });
            }
        };
        let lower = winner.unknowns.iter().map(|u| u.lower).collect::<Vec<_>>();
        let upper = winner.unknowns.iter().map(|u| u.upper).collect::<Vec<_>>();
        let el = chart.existence.iter().map(|i| i.lower).collect::<Vec<_>>();
        let eu = chart.existence.iter().map(|i| i.upper).collect::<Vec<_>>();
        let ul = chart.uniqueness.iter().map(|i| i.lower).collect::<Vec<_>>();
        let uu = chart.uniqueness.iter().map(|i| i.upper).collect::<Vec<_>>();
        let pl: Vec<_> = parameters.iter().map(|i| i.lower).collect();
        let pu: Vec<_> = parameters.iter().map(|i| i.upper).collect();
        let dl: Vec<_> = directions.into_iter().flatten().map(|i| i.lower).collect();
        let du: Vec<_> = directions.into_iter().flatten().map(|i| i.upper).collect();
        let mut point_lower = vec![0.0; el.len()];
        let mut point_upper = vec![0.0; el.len()];
        let mut action_lower = vec![0.0; el.len()];
        let mut action_upper = vec![0.0; el.len()];
        let library_guard = loop {
            if !checkpoint()? {
                return Ok(RootNeighborhoodEvidence::Incomplete {
                    reason: SelectionProofRefusal::Resource,
                    proof_cells: 0,
                });
            }
            match IBEX.try_lock() {
                Ok(g) => break g,
                Err(TryLockError::Poisoned(_)) => {
                    return Ok(RootNeighborhoodEvidence::Incomplete {
                        reason: SelectionProofRefusal::Resource,
                        proof_cells: 0,
                    });
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        };
        let native = NativeRequest {
            nodes: transport.nodes.as_ptr(),
            node_count: transport.nodes.len() as u32,
            edges: transport.edges.as_ptr(),
            edge_count: transport.edges.len() as u32,
            residuals: transport.residuals.as_ptr(),
            unknown_count: winner.unknowns.len() as u32,
            parameter_count: request.parameters.len() as u32,
            score: winner.program.criterion[0] as u32,
            tolerance: winner.program.criterion[1] as u32,
            guards: transport.guards.as_ptr(),
            guard_count: transport.guards.len() as u32,
            lower: lower.as_ptr(),
            upper: upper.as_ptr(),
            parameters: request.parameters.as_ptr(),
            candidate: request.candidate.as_ptr(),
            max_cells: max_cells.min(MAX_CELLS),
            seconds: request
                .time_limit
                .saturating_sub(started.elapsed())
                .as_secs_f64(),
            cancelled,
            cancel_context: std::ptr::from_ref(request.cancel.as_ref()).cast(),
        };
        let mut result = NativeResult::default();
        // SAFETY: scope validation establishes finite original coordinate extents;
        // all input/output buffers live across the serialized synchronous caught ABI.
        #[expect(
            unsafe_code,
            reason = "synchronous IBEX interval IFT action with validated borrowed buffers"
        )]
        // SAFETY: validated dimensions, owned buffers and cancellation Arc remain live
        // through the serialized synchronous IBEX call.
        let code = unsafe {
            pse_ibex_enclose_neighborhood(
                &native,
                &mut result,
                el.as_ptr(),
                eu.as_ptr(),
                ul.as_ptr(),
                uu.as_ptr(),
                pl.as_ptr(),
                pu.as_ptr(),
                if directions.is_some() {
                    dl.as_ptr()
                } else {
                    std::ptr::null()
                },
                if directions.is_some() {
                    du.as_ptr()
                } else {
                    std::ptr::null()
                },
                point_lower.as_mut_ptr(),
                point_upper.as_mut_ptr(),
                action_lower.as_mut_ptr(),
                action_upper.as_mut_ptr(),
            )
        };
        drop(library_guard);
        if request.cancel.load(Ordering::Acquire) {
            return Ok(RootNeighborhoodEvidence::Interrupted {
                proof_cells: result.cells,
            });
        }
        if !checkpoint()? {
            return Ok(RootNeighborhoodEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: result.cells,
            });
        }
        if code != 0 {
            return Err(MathError::Contract("IBEX selected action transport".into()));
        }
        let points = point_lower
            .into_iter()
            .zip(point_upper)
            .map(|(lower, upper)| ProofInterval { lower, upper })
            .collect::<Vec<_>>();
        let actions = action_lower
            .into_iter()
            .zip(action_upper)
            .map(|(lower, upper)| ProofInterval { lower, upper })
            .collect::<Vec<_>>();
        if result.status == 0
            && result.solution == 1
            && result.cells > 0
            && result.cells <= max_cells.min(MAX_CELLS)
            && points.iter().all(|i| i.valid())
            && (directions.is_none() || actions.iter().all(|i| i.valid()))
        {
            return Ok(RootNeighborhoodEvidence::Enclosed {
                points,
                actions: directions.map(|_| actions),
                proof_cells: result.cells,
            });
        }
        let reason = match result.status {
            4 => SelectionProofRefusal::Resource,
            5 => SelectionProofRefusal::Boundary,
            6 => SelectionProofRefusal::Unsupported,
            _ => SelectionProofRefusal::Chart,
        };
        Ok(RootNeighborhoodEvidence::Incomplete {
            reason,
            proof_cells: result.cells,
        })
    }
    fn connect_chain(
        &self,
        chain: &ChartChainRequest<'_>,
    ) -> Result<ChartChainEvidence, MathError> {
        self.connect_chain_bounded(chain, MAX_CELLS)
    }
    fn connect_chain_bounded(
        &self,
        chain: &ChartChainRequest<'_>,
        max_cells: u64,
    ) -> Result<ChartChainEvidence, MathError> {
        let max_cells = max_cells.min(MAX_CELLS);
        chain.validate(self.identity())?;
        // Endpoint exclusion evidence does not certify intermediate competitors.
        if chain.coverage == ChartChainCoverage::SelectedFunction {
            return Ok(ChartChainEvidence::Incomplete(
                SelectionProofRefusal::Coverage,
            ));
        }
        let request = chain.endpoint;
        let started = Instant::now();
        let checkpoint = || {
            if request.cancel.load(Ordering::Acquire) {
                Err(MathError::Cancelled)
            } else {
                Ok(started.elapsed() < request.time_limit)
            }
        };
        if !checkpoint()? {
            return Ok(ChartChainEvidence::Incomplete(
                SelectionProofRefusal::Resource,
            ));
        }
        let winner = &request.alternatives[request.winner];
        let transport = match encode(winner.program, request.order.max(DerivativeOrder::First)) {
            Ok(value) => value,
            Err(reason) => return Ok(ChartChainEvidence::Incomplete(reason)),
        };
        let lower = winner.unknowns.iter().map(|u| u.lower).collect::<Vec<_>>();
        let upper = winner.unknowns.iter().map(|u| u.upper).collect::<Vec<_>>();
        let intervals = |values: &[ProofInterval]| {
            (
                values.iter().map(|i| i.lower).collect::<Vec<_>>(),
                values.iter().map(|i| i.upper).collect::<Vec<_>>(),
            )
        };
        let (previous_pl, previous_pu) = intervals(&chain.previous.parameters);
        let (previous_el, previous_eu) = intervals(&chain.previous.existence);
        let (previous_ul, previous_uu) = intervals(&chain.previous.uniqueness);
        let (next_pl, next_pu) = intervals(&chain.next.parameters);
        let (next_el, next_eu) = intervals(&chain.next.existence);
        let (next_ul, next_uu) = intervals(&chain.next.uniqueness);
        let library_guard = loop {
            if !checkpoint()? {
                return Ok(ChartChainEvidence::Incomplete(
                    SelectionProofRefusal::Resource,
                ));
            }
            match IBEX.try_lock() {
                Ok(guard) => break guard,
                Err(TryLockError::Poisoned(_)) => {
                    return Ok(ChartChainEvidence::Incomplete(
                        SelectionProofRefusal::Resource,
                    ));
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        };
        let native = NativeRequest {
            nodes: transport.nodes.as_ptr(),
            node_count: transport.nodes.len() as u32,
            edges: transport.edges.as_ptr(),
            edge_count: transport.edges.len() as u32,
            residuals: transport.residuals.as_ptr(),
            unknown_count: winner.unknowns.len() as u32,
            parameter_count: request.parameters.len() as u32,
            score: winner.program.criterion[0] as u32,
            tolerance: winner.program.criterion[1] as u32,
            guards: transport.guards.as_ptr(),
            guard_count: transport.guards.len() as u32,
            lower: lower.as_ptr(),
            upper: upper.as_ptr(),
            parameters: request.parameters.as_ptr(),
            candidate: request.candidate.as_ptr(),
            max_cells,
            seconds: request
                .time_limit
                .saturating_sub(started.elapsed())
                .as_secs_f64(),
            cancelled,
            cancel_context: std::ptr::from_ref(request.cancel.as_ref()).cast(),
        };
        let mut result = NativeChartChainResult::default();
        // SAFETY: exact source/domain and all coordinate extents were checked against
        // both endpoint certificates. Owned buffers and cancellation Arc remain live
        // across the serialized synchronous C++ call, whose boundary catches exceptions.
        #[expect(
            unsafe_code,
            reason = "synchronous IBEX finite chart-chain proof with checked borrowed buffers"
        )]
        // SAFETY: validated dimensions, owned buffers and cancellation Arc remain live
        // through the serialized synchronous IBEX call.
        let code = unsafe {
            pse_ibex_connect_chain(
                &native,
                &mut result,
                chain.origin.as_ptr(),
                previous_pl.as_ptr(),
                previous_pu.as_ptr(),
                previous_el.as_ptr(),
                previous_eu.as_ptr(),
                previous_ul.as_ptr(),
                previous_uu.as_ptr(),
                next_pl.as_ptr(),
                next_pu.as_ptr(),
                next_el.as_ptr(),
                next_eu.as_ptr(),
                next_ul.as_ptr(),
                next_uu.as_ptr(),
            )
        };
        drop(library_guard);
        let work = ChartChainWork {
            charts: result.charts,
            connections: result.connections,
            proof_cells: result.proof_cells,
        };
        if request.cancel.load(Ordering::Acquire) {
            return Ok(ChartChainEvidence::Interrupted(work));
        }
        if !checkpoint()? {
            return Ok(ChartChainEvidence::Refused {
                reason: SelectionProofRefusal::Resource,
                work,
            });
        }
        if code != 0 {
            return Err(MathError::Contract("IBEX chart-chain transport".into()));
        }
        match result.status {
            0 if result.charts > 0
                && result.connections == result.charts.saturating_add(1)
                && result.proof_cells >= result.charts.saturating_add(result.connections)
                && result.proof_cells <= max_cells =>
            {
                Ok(ChartChainEvidence::Connected(ChartChainProof {
                    coverage: ChartChainCoverage::RootSheet,
                    charts: result.charts,
                    connections: result.connections,
                    proof_cells: result.proof_cells,
                }))
            }
            4 => Ok(ChartChainEvidence::Refused {
                reason: SelectionProofRefusal::Resource,
                work,
            }),
            5 => Ok(ChartChainEvidence::Refused {
                reason: SelectionProofRefusal::Boundary,
                work,
            }),
            6 => Ok(ChartChainEvidence::Refused {
                reason: SelectionProofRefusal::Unsupported,
                work,
            }),
            _ => Ok(ChartChainEvidence::Refused {
                reason: SelectionProofRefusal::Chart,
                work,
            }),
        }
    }
    fn certify(&self, request: &SelectionProofRequest<'_>) -> Result<SelectionEvidence, MathError> {
        self.certify_bounded(request, MAX_CELLS)
    }
    #[expect(
        unsafe_code,
        reason = "all native pointers borrow finite checked vectors; native ABI catches every C++ exception"
    )]
    fn certify_bounded(
        &self,
        request: &SelectionProofRequest<'_>,
        max_cells: u64,
    ) -> Result<SelectionEvidence, MathError> {
        let max_cells = max_cells.min(MAX_CELLS);
        if max_cells < 2 {
            return Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Resource,
            ));
        }
        let started = Instant::now();
        let checkpoint = || {
            if request.cancel.load(Ordering::Relaxed) {
                Err(MathError::Cancelled)
            } else {
                Ok(started.elapsed() < request.time_limit)
            }
        };
        if !checkpoint()? {
            return Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Resource,
            ));
        }
        let Some(winner) = request.alternatives.get(request.winner) else {
            return Err(MathError::Contract("IBEX winning alternative".into()));
        };
        if request.alternatives.len() > MAX_ALTERNATIVES {
            return Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Resource,
            ));
        }
        let unknown_count = winner.unknowns.len();
        if unknown_count == 0
            || request.candidate.len() != unknown_count
            || request.alternatives.iter().any(|alternative| {
                alternative.program.inputs != unknown_count + request.parameters.len()
                    || alternative.program.residuals.len() != unknown_count
                    || alternative.unknowns.len() != unknown_count
                    || alternative
                        .unknowns
                        .iter()
                        .zip(winner.unknowns)
                        .any(|(a, b)| a.id != b.id)
            })
        {
            return Err(MathError::Contract("IBEX coordinate layout".into()));
        }
        let mut transports = Vec::with_capacity(request.alternatives.len());
        let mut bounds = Vec::with_capacity(request.alternatives.len());
        for alternative in request.alternatives {
            if !checkpoint()? {
                return Ok(SelectionEvidence::Incomplete(
                    SelectionProofRefusal::Resource,
                ));
            }
            let transport = match encode(
                alternative.program,
                request.order.max(DerivativeOrder::First),
            ) {
                Ok(value) => value,
                Err(reason) => return Ok(SelectionEvidence::Incomplete(reason)),
            };
            transports.push(transport);
            bounds.push((
                alternative
                    .unknowns
                    .iter()
                    .map(|value| value.lower)
                    .collect::<Vec<_>>(),
                alternative
                    .unknowns
                    .iter()
                    .map(|value| value.upper)
                    .collect::<Vec<_>>(),
            ));
        }
        let _library = loop {
            if !checkpoint()? {
                return Ok(SelectionEvidence::Incomplete(
                    SelectionProofRefusal::Resource,
                ));
            }
            match IBEX.try_lock() {
                Ok(guard) => break guard,
                Err(TryLockError::Poisoned(_)) => {
                    return Ok(SelectionEvidence::Incomplete(
                        SelectionProofRefusal::Resource,
                    ));
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        };
        let seconds = request
            .time_limit
            .saturating_sub(started.elapsed())
            .as_secs_f64();
        let inputs: Vec<_> = request
            .alternatives
            .iter()
            .zip(&transports)
            .zip(&bounds)
            .map(|((alternative, transport), (lower, upper))| NativeRequest {
                nodes: transport.nodes.as_ptr(),
                node_count: transport.nodes.len() as u32,
                edges: transport.edges.as_ptr(),
                edge_count: transport.edges.len() as u32,
                residuals: transport.residuals.as_ptr(),
                unknown_count: unknown_count as u32,
                parameter_count: request.parameters.len() as u32,
                score: alternative.program.criterion[0] as u32,
                tolerance: alternative.program.criterion[1] as u32,
                guards: transport.guards.as_ptr(),
                guard_count: transport.guards.len() as u32,
                lower: lower.as_ptr(),
                upper: upper.as_ptr(),
                parameters: request.parameters.as_ptr(),
                candidate: request.candidate.as_ptr(),
                max_cells,
                seconds,
                cancelled,
                cancel_context: std::ptr::from_ref(request.cancel.as_ref()).cast(),
            })
            .collect();
        let mut parameter_lower = vec![0.0; request.parameters.len()];
        let mut parameter_upper = parameter_lower.clone();
        let mut existence_lower = vec![0.0; unknown_count];
        let mut existence_upper = existence_lower.clone();
        let mut uniqueness_lower = existence_lower.clone();
        let mut uniqueness_upper = existence_lower.clone();
        let mut result = NativeResult::default();
        // SAFETY: validated dimensions match every frozen request buffer and output
        // vector. Their heap allocations and the cancellation Arc stay live and
        // stationary for this synchronous call; C++ catches all exceptions and
        // does not retain any pointer. The IBEX lock serializes its global state.
        let code = unsafe {
            pse_ibex_certify(
                inputs.as_ptr(),
                inputs.len() as u32,
                request.winner as u32,
                &mut result,
                parameter_lower.as_mut_ptr(),
                parameter_upper.as_mut_ptr(),
                existence_lower.as_mut_ptr(),
                existence_upper.as_mut_ptr(),
                uniqueness_lower.as_mut_ptr(),
                uniqueness_upper.as_mut_ptr(),
            )
        };
        if !checkpoint()? {
            return Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Resource,
            ));
        }
        if code != 0 {
            return Err(MathError::Contract("IBEX native transport".into()));
        }
        let intervals = |lower: Vec<f64>, upper: Vec<f64>| {
            lower
                .into_iter()
                .zip(upper)
                .map(|(lower, upper)| ProofInterval { lower, upper })
                .collect::<Vec<_>>()
        };
        match result.status {
            0 if result.solution == 1
                && result.boundary == 0
                && result.unknown == 0
                && result.pending == 0 =>
            {
                let chart = SelectionChart {
                    selection: request.selection,
                    alternatives: request
                        .alternatives
                        .iter()
                        .map(|alternative| SelectionScope {
                            id: alternative.id,
                            program: Arc::clone(alternative.program),
                            residual_identity: alternative.residual_identity,
                            unknowns: alternative.unknowns.to_vec(),
                        })
                        .collect(),
                    winner: request.winner,
                    verifier_identity: self.identity(),
                    parameters: intervals(parameter_lower, parameter_upper),
                    existence: intervals(existence_lower, existence_upper),
                    uniqueness: intervals(uniqueness_lower, uniqueness_upper),
                    order: request.order,
                };
                chart.validate(request, self.identity())?;
                Ok(SelectionEvidence::Unique(chart))
            }
            2 => Ok(SelectionEvidence::Incomplete(SelectionProofRefusal::Chart)),
            4 => Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Resource,
            )),
            5 => Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Boundary,
            )),
            6 => Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Unsupported,
            )),
            _ => Ok(SelectionEvidence::Incomplete(
                SelectionProofRefusal::Coverage,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rational_transport_encloses_exact_constants() {
        for value in [
            Rational::from((1, 3)),
            Rational::from((-7, 11)),
            Rational::from(2),
        ] {
            let (lower, upper) = rational_bounds(&value).unwrap();
            assert!(Rational::try_from(lower).unwrap() <= value);
            assert!(Rational::try_from(upper).unwrap() >= value);
        }
        assert!(constant_bounds(&Constant::Float(f64::NAN)).is_err());
    }
    fn arithmetic_program(nodes: Vec<Node>, outputs: Vec<usize>) -> PointArithmeticProgram {
        PointArithmeticProgram {
            key: ContentHash::from_bytes([19;32]),
            rows: vec![pse_math::factorable::PointArithmeticRow {
                value: 0,lower_residual: Some(0),upper_residual: Some(0),
            }],
            objective: None,
            graph: RootIsolationProgram {
                inputs: 1,criterion: [nodes.len()-1,nodes.len()-1],nodes,
                residuals: outputs,eligibility: vec![],obligations: vec![],derivative_obligations: vec![],
            },
        }
    }
    fn point_arithmetic(program: &PointArithmeticProgram, x:f64) -> PointArithmeticEvidence {
        let bytes = Ibex.point_arithmetic_workspace_bytes(program).unwrap();
        Ibex.enclose_point_arithmetic(program,&[x],Duration::from_secs(10),
            &Arc::new(AtomicBool::new(false)),bytes).unwrap()
    }
    #[test]
    fn native_point_arithmetic_encloses_catastrophic_cancellation_residual() {
        let program = arithmetic_program(vec![
            Node::Var(0),Node::Exp(0),Node::Const(Constant::Float(1e16)),
            Node::Sum(vec![1,2]),Node::Const(Constant::Float(-(1e16+2.0))),
            Node::Sum(vec![3,4]),Node::Const(Constant::Float(0.0)),
        ],vec![5]);
        assert_eq!((0.5_f64.exp()+1e16)-(1e16+2.0),0.0);
        let PointArithmeticEvidence::Enclosed {values,jacobian,work,..} = point_arithmetic(&program,0.5)
            else { panic!("original point arithmetic absent"); };
        let exact = 0.5_f64.exp()-2.0;
        assert!(values[0].lower <= exact && values[0].upper >= exact);
        assert!(values[0].lower < 0.0);
        assert!(jacobian[0].lower <= 0.5_f64.exp() && jacobian[0].upper >= 0.5_f64.exp());
        assert_eq!(work.value_evaluations,1);assert_eq!(work.jacobian_evaluations,1);
    }
    #[test]
    fn native_point_arithmetic_box_covers_nonlinear_values_and_uniform_first() {
        let program=arithmetic_program(vec![Node::Var(0),Node::Pow {base:0,
            exponent:Constant::Rational(Rational::from(2))},Node::Exp(0),Node::Const(Constant::Float(0.0))],vec![1,2]);
        let flag=Arc::new(AtomicBool::new(false));
        let execution=crate::solve::Execution::within(flag.clone(),&crate::solve::Controls::default(),
            pse_kernels::ExecutionScope::new(flag,None)).unwrap();
        let region=[ProofInterval {lower:1.0,upper:2.0}];
        for order in [DerivativeOrder::Value,DerivativeOrder::First] {
            let bytes=Ibex.point_arithmetic_workspace_bytes_for_order(&program,order).unwrap();
            let PointArithmeticEvidence::Enclosed {values,jacobian,work,hessian}=Ibex
                .enclose_arithmetic_box_with_execution(&program,&region,order,bytes,&execution).unwrap()
                else {panic!("actual source box arithmetic unavailable");};
            assert!(values[0].lower<=1.0&&values[0].upper>=4.0);
            assert!(values[1].lower<=1.0_f64.exp()&&values[1].upper>=2.0_f64.exp());
            assert_eq!(work.value_evaluations,1);assert_eq!(work.jacobian_evaluations,u64::from(order==DerivativeOrder::First));
            assert!(hessian.is_none());
            if order==DerivativeOrder::First {assert!(jacobian[0].lower<=2.0&&jacobian[0].upper>=4.0);}
            else {assert!(jacobian.is_empty());}
        }
        assert!(matches!(Ibex.enclose_arithmetic_box_values_with_execution(&program,&region,0,&execution).unwrap(),
            PointArithmeticEvidence::Incomplete {reason:SelectionProofRefusal::Resource,work:PointArithmeticWork {value_evaluations:0,..}}));
    }
    #[test]
    fn native_point_arithmetic_box_checks_whole_domain_and_value_needs_no_first_regularity() {
        let program=arithmetic_program(vec![Node::Var(0),Node::Pow {base:0,
            exponent:Constant::Rational(Rational::from((1,2)))},Node::Const(Constant::Float(0.0))],vec![1]);
        let flag=Arc::new(AtomicBool::new(false));
        let execution=crate::solve::Execution::within(flag.clone(),&crate::solve::Controls::default(),
            pse_kernels::ExecutionScope::new(flag,None)).unwrap();
        let region=[ProofInterval {lower:0.0,upper:1.0}];
        let bytes=Ibex.point_arithmetic_workspace_bytes_for_order(&program,DerivativeOrder::First).unwrap();
        assert!(matches!(Ibex.enclose_arithmetic_box_values_with_execution(&program,&region,bytes,&execution).unwrap(),
            PointArithmeticEvidence::Enclosed {work:PointArithmeticWork {jacobian_evaluations:0,..},..}));
        assert!(matches!(Ibex.enclose_arithmetic_box_with_execution(&program,&region,DerivativeOrder::First,bytes,&execution).unwrap(),
            PointArithmeticEvidence::Incomplete {reason:SelectionProofRefusal::Boundary,..}));
        assert!(matches!(Ibex.enclose_arithmetic_box_values_with_execution(&program,
            &[ProofInterval {lower:-1.0,upper:1.0}],bytes,&execution).unwrap(),
            PointArithmeticEvidence::Incomplete {reason:SelectionProofRefusal::Boundary,..}));
    }
    #[test]
    fn native_point_arithmetic_preserves_rational_uncertainty_and_rectangular_first() {
        let mut program = arithmetic_program(vec![
            Node::Var(0),Node::Const(Constant::Rational(Rational::from((1,3)))),
            Node::Sum(vec![0,1]),Node::Exp(0),Node::Const(Constant::Float(0.0)),
        ],vec![2,3]);
        program.objective=Some(1);
        let PointArithmeticEvidence::Enclosed {values,jacobian,..} = point_arithmetic(&program,0.0)
            else { panic!("rectangular arithmetic absent"); };
        assert_eq!(values.len(),2);assert_eq!(jacobian.len(),2);
        assert!(Rational::try_from(values[0].lower).unwrap() <= Rational::from((1,3)));
        assert!(Rational::try_from(values[0].upper).unwrap() >= Rational::from((1,3)));
        assert!(values[0].lower < values[0].upper);
        assert!(jacobian.iter().all(|i| i.lower <= 1.0 && i.upper >= 1.0));
    }
    #[test]
    fn native_point_arithmetic_refuses_unsupported_and_first_boundary() {
        let unsupported = arithmetic_program(vec![Node::Aux(0),Node::Const(Constant::Float(0.0))],vec![0]);
        assert!(matches!(point_arithmetic(&unsupported,0.0),PointArithmeticEvidence::Incomplete {
            reason:SelectionProofRefusal::Unsupported,work:PointArithmeticWork {guard_evaluations:0,value_evaluations:0,jacobian_evaluations:0,hessian_evaluations:0},
        }));
        let square_root = arithmetic_program(vec![Node::Var(0),Node::Pow {
            base:0,exponent:Constant::Rational(Rational::from((1,2))),
        },Node::Const(Constant::Float(0.0))],vec![1]);
        assert!(matches!(point_arithmetic(&square_root,0.0),PointArithmeticEvidence::Incomplete {
            reason:SelectionProofRefusal::Boundary,work:PointArithmeticWork {value_evaluations:0,jacobian_evaluations:0,..},
        }));
    }
    #[test]
    fn native_point_arithmetic_observes_cancellation_and_byte_admission_before_work() {
        let program = arithmetic_program(vec![Node::Var(0),Node::Const(Constant::Float(0.0))],vec![0]);
        let cancelled = Arc::new(AtomicBool::new(true));
        assert_eq!(Ibex.enclose_point_arithmetic(&program,&[0.0],Duration::from_secs(10),&cancelled,usize::MAX).unwrap(),
            PointArithmeticEvidence::Interrupted {work:PointArithmeticWork::default()});
        cancelled.store(false,Ordering::Release);
        for (duration,bytes) in [(Duration::ZERO,usize::MAX),(Duration::from_secs(10),0)] {
            assert_eq!(Ibex.enclose_point_arithmetic(&program,&[0.0],duration,&cancelled,bytes).unwrap(),
                PointArithmeticEvidence::Incomplete {reason:SelectionProofRefusal::Resource,work:PointArithmeticWork::default()});
        }
    }
    #[test]
    fn native_point_arithmetic_cancels_while_waiting_for_process_mutex() {
        let held=IBEX.lock().unwrap();
        let program=arithmetic_program(vec![Node::Var(0),Node::Const(Constant::Float(0.0))],vec![0]);
        let bytes=Ibex.point_arithmetic_workspace_bytes(&program).unwrap();
        let cancel=Arc::new(AtomicBool::new(false));let worker_cancel=cancel.clone();
        let (started_tx,started_rx)=std::sync::mpsc::channel();
        let (result_tx,result_rx)=std::sync::mpsc::channel();
        let worker=std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            let result=Ibex.enclose_point_arithmetic(&program,&[0.0],Duration::from_secs(10),&worker_cancel,bytes);
            result_tx.send(result).unwrap();
        });
        started_rx.recv().unwrap();std::thread::sleep(Duration::from_millis(20));
        cancel.store(true,Ordering::Release);
        let result=result_rx.recv_timeout(Duration::from_secs(1));
        drop(held);worker.join().unwrap();
        assert_eq!(result.unwrap().unwrap(),PointArithmeticEvidence::Interrupted {work:PointArithmeticWork::default()});
    }
    #[test]
    fn native_point_arithmetic_execution_reserves_ceiling_and_observes_actual_calls() {
        use crate::solve::{Controls,Execution,WorkAdmission,WorkEvidence};
        #[derive(Debug,Default)]
        struct Ledger { admitted: Mutex<Vec<WorkEvidence>>,observed: Mutex<Vec<WorkEvidence>> }
        impl WorkAdmission for Ledger {
            fn admit(&self,work:WorkEvidence) -> Result<(),crate::ProblemError> {
                self.admitted.lock().unwrap().push(work);Ok(())
            }
            fn observe(&self,work:WorkEvidence) -> Result<(),crate::ProblemError> {
                self.observed.lock().unwrap().push(work);Ok(())
            }
        }
        let owner=Arc::new(Ledger::default());
        let mut execution=Execution::new(Arc::new(AtomicBool::new(false)),&Controls::default());
        execution.work_admission=Some(owner.clone());
        let unsupported=arithmetic_program(vec![Node::Aux(0),Node::Const(Constant::Float(0.0))],vec![0]);
        let bytes=Ibex.point_arithmetic_workspace_bytes(&unsupported).unwrap();
        assert!(matches!(Ibex.enclose_point_arithmetic_with_execution(&unsupported,&[0.0],bytes,&execution).unwrap(),
            PointArithmeticEvidence::Incomplete {reason:SelectionProofRefusal::Unsupported,..}));
        let valid=arithmetic_program(vec![Node::Var(0),Node::Const(Constant::Float(0.0))],vec![0]);
        let bytes=Ibex.point_arithmetic_workspace_bytes(&valid).unwrap();
        assert!(matches!(Ibex.enclose_point_arithmetic_with_execution(&valid,&[0.5],bytes,&execution).unwrap(),
            PointArithmeticEvidence::Enclosed {..}));
        assert!(Ibex.enclose_point_arithmetic_with_execution(&valid,&[],bytes,&execution).is_err());
        let bytes=Ibex.point_arithmetic_workspace_bytes_for_order(&valid,DerivativeOrder::Second).unwrap();
        assert!(matches!(Ibex.enclose_point_arithmetic_order_with_execution(&valid,&[0.5],DerivativeOrder::Second,bytes,&execution).unwrap(),
            PointArithmeticEvidence::Enclosed {hessian:Some(_),..}));
        let admitted=owner.admitted.lock().unwrap();
        let observed=owner.observed.lock().unwrap();
        assert_eq!(admitted.iter().map(|w| w.evaluations).collect::<Vec<_>>(),vec![Some(2),Some(2),Some(2),Some(3)]);
        assert_eq!(observed.iter().map(|w| w.evaluations).collect::<Vec<_>>(),vec![Some(0),Some(2),None,Some(3)]);
        assert!(admitted.iter().chain(observed.iter()).all(|w|
            w.iterations==Some(0)&&w.factorizations==Some(0)&&w.proof_steps==Some(0)));
    }
    fn point_arithmetic_second(program:&PointArithmeticProgram,point:&[f64]) -> PointArithmeticEvidence {
        let execution=crate::solve::Execution::new(Arc::new(AtomicBool::new(false)),&crate::solve::Controls::default());
        let bytes=Ibex.point_arithmetic_workspace_bytes_for_order(program,DerivativeOrder::Second).unwrap();
        Ibex.enclose_point_arithmetic_order_with_execution(program,point,DerivativeOrder::Second,bytes,&execution).unwrap()
    }
    #[test]
    fn native_point_arithmetic_second_encloses_polynomial_and_objective_hessians() {
        let mut program=arithmetic_program(vec![
            Node::Var(0),Node::Var(1),Node::Pow {base:0,exponent:Constant::Rational(Rational::from(2))},
            Node::Product(vec![0,1]),Node::Pow {base:1,exponent:Constant::Rational(Rational::from(2))},
            Node::Const(Constant::Float(3.0)),Node::Product(vec![5,4]),Node::Sum(vec![2,3,6]),
            Node::Exp(0),Node::Const(Constant::Float(0.0)),
        ],vec![7,8]);
        program.graph.inputs=2;program.objective=Some(1);
        let PointArithmeticEvidence::Enclosed {hessian:Some(hessian),work,..}=point_arithmetic_second(&program,&[0.5,1.0])
            else { panic!("Second arithmetic absent"); };
        assert_eq!(hessian.len(),8);
        for (interval,exact) in hessian.iter().zip([2.0,1.0,1.0,6.0,0.5_f64.exp(),0.0,0.0,0.0]) {
            assert!(interval.contains(exact),"{interval:?} does not enclose {exact}");
        }
        assert_eq!(work.hessian_evaluations,2);
        assert_eq!(work.value_evaluations,1);assert_eq!(work.jacobian_evaluations,1);
    }
    #[test]
    fn native_point_arithmetic_second_enforces_second_and_intrinsic_domain_guards() {
        for node in [Node::Log(0),Node::Pow {base:0,exponent:Constant::Rational(Rational::from((1,2)))}] {
            let program=arithmetic_program(vec![Node::Var(0),node,Node::Const(Constant::Float(0.0))],vec![1]);
            assert!(matches!(point_arithmetic_second(&program,&[0.0]),PointArithmeticEvidence::Incomplete {
                reason:SelectionProofRefusal::Boundary,work:PointArithmeticWork {value_evaluations:0,jacobian_evaluations:0,hessian_evaluations:0,..},
            }));
        }
        let mut program=arithmetic_program(vec![Node::Var(0),Node::Const(Constant::Float(0.0))],vec![0]);
        program.graph.derivative_obligations.push((DerivativeOrder::Second,ProjectedObligation {
            instance:source(),source:source(),kind:ObligationKind::Require(Condition::Nonzero),
            scope:ObligationScope::Unconditional,argument:Some(0),constraints:vec![],represented:true,fidelity:Fidelity::Exact,
        }));
        assert!(matches!(point_arithmetic(&program,0.0),PointArithmeticEvidence::Enclosed {hessian:None,..}));
        assert!(matches!(point_arithmetic_second(&program,&[0.0]),PointArithmeticEvidence::Incomplete {
            reason:SelectionProofRefusal::Boundary,work:PointArithmeticWork {guard_evaluations:1,value_evaluations:0,jacobian_evaluations:0,hessian_evaluations:0},
        }));
        program.graph.derivative_obligations[0].1.scope=ObligationScope::Conditional;
        assert!(matches!(point_arithmetic_second(&program,&[1.0]),PointArithmeticEvidence::Incomplete {
            reason:SelectionProofRefusal::Unsupported,work:PointArithmeticWork {guard_evaluations:0,value_evaluations:0,jacobian_evaluations:0,hessian_evaluations:0},
        }));
    }
    #[test]
    fn native_point_arithmetic_second_refuses_dense_extent_before_work() {
        let execution=crate::solve::Execution::new(Arc::new(AtomicBool::new(false)),&crate::solve::Controls::default());
        for (width,outputs) in [(33,1),(1,129),(17,128)] {
            let mut program=arithmetic_program(vec![Node::Var(0),Node::Const(Constant::Float(0.0))],vec![0;outputs]);
            program.graph.inputs=width;
            assert!(Ibex.point_arithmetic_workspace_bytes_for_order(&program,DerivativeOrder::Second).is_err());
            assert!(matches!(Ibex.enclose_point_arithmetic_order_with_execution(&program,&vec![0.0;width],
                DerivativeOrder::Second,usize::MAX,&execution).unwrap(),PointArithmeticEvidence::Incomplete {
                reason:SelectionProofRefusal::Resource,work:PointArithmeticWork {guard_evaluations:0,value_evaluations:0,jacobian_evaluations:0,hessian_evaluations:0},
            }));
        }
    }
    use pse_ids::SemanticId;
    use pse_math::{
        Function,
        factorable::root_isolation_program,
        guarded::{Comparison, PreparedBody},
        implicit::{SelectionAlternative, Unknown},
        typed::{Binary, BodyBuilder, BodyLimits, TypedValue},
    };
    use pse_quantity::{
        IndexSet, QuantityRegistry,
        literal::LiteralContext,
        standard::{StandardInvariantChecker, standard_registry},
    };
    use std::{sync::Arc, time::Duration};

    fn source() -> SemanticId {
        SemanticId::from_bytes([153; 16])
    }
    fn builder(registry: &QuantityRegistry, inputs: usize) -> BodyBuilder<'_> {
        BodyBuilder::new(
            pse_math::initialize().unwrap(),
            registry,
            &StandardInvariantChecker,
            inputs,
            BodyLimits::default(),
        )
        .unwrap()
    }
    fn literal(b: &mut BodyBuilder<'_>, registry: &QuantityRegistry, value: f64) -> TypedValue {
        let quantity = registry.neutral_dimensionless().unwrap();
        let unit = registry
            .unit(registry.quantity_type(quantity).unwrap().canonical_unit)
            .unwrap();
        b.literal(
            value,
            unit,
            LiteralContext::Explicit {
                quantity_type: quantity,
            },
            source(),
        )
        .unwrap()
    }
    fn input(b: &mut BodyBuilder<'_>, registry: &QuantityRegistry, slot: usize) -> TypedValue {
        b.input(
            slot,
            registry.neutral_dimensionless().unwrap(),
            IndexSet::new(),
            source(),
        )
        .unwrap()
    }
    #[derive(Clone, Copy)]
    enum Residual {
        Square,
        Log,
        Singular,
        Linear,
        ConstantTwoRoots,
    }
    #[derive(Clone, Copy)]
    enum Score {
        Constant(f64),
        Unknown,
        ParameterWeightedUnknown,
    }
    fn projected(
        coupled: bool,
        logarithm: bool,
        strict_lower: Option<f64>,
    ) -> RootIsolationProgram {
        projected_selection(
            coupled,
            if logarithm {
                Residual::Log
            } else {
                Residual::Square
            },
            strict_lower,
            Score::Constant(0.0),
            0.0,
        )
    }
    fn projected_selection(
        coupled: bool,
        shape: Residual,
        strict_lower: Option<f64>,
        score: Score,
        tolerance: f64,
    ) -> RootIsolationProgram {
        let registry = standard_registry().unwrap();
        let unknowns = if coupled { 2 } else { 1 };
        let inputs = unknowns + 1;
        let mut residual = builder(&registry, inputs);
        let x = input(&mut residual, &registry, 0);
        let p = input(&mut residual, &registry, unknowns);
        let function = match shape {
            Residual::Log => residual.unary(Function::Log, x.clone(), source()).unwrap(),
            Residual::Linear => x.clone(),
            Residual::Square | Residual::Singular | Residual::ConstantTwoRoots => residual
                .binary(Binary::Mul, x.clone(), x.clone(), None, source())
                .unwrap(),
        };
        let equation = match shape {
            Residual::Singular => function,
            Residual::ConstantTwoRoots => {
                let one = literal(&mut residual, &registry, 1.0);
                residual
                    .binary(Binary::Sub, function, one, None, source())
                    .unwrap()
            }
            Residual::Square | Residual::Log | Residual::Linear => residual
                .binary(Binary::Sub, function, p, None, source())
                .unwrap(),
        };
        let mut outputs = vec![equation];
        if coupled {
            let y = input(&mut residual, &registry, 1);
            outputs.push(residual.binary(Binary::Sub, y, x, None, source()).unwrap());
        }
        let residual: PreparedBody = residual.prepare(&outputs).unwrap();
        let mut eligibility = builder(&registry, inputs);
        let one = literal(&mut eligibility, &registry, 1.0);
        let predicate = if let Some(lower) = strict_lower {
            let zero = literal(&mut eligibility, &registry, 0.0);
            let lower = literal(&mut eligibility, &registry, lower);
            let x = input(&mut eligibility, &registry, 0);
            let guard = eligibility
                .compare(Comparison::Lt, &lower, &x, source())
                .unwrap();
            eligibility
                .conditional(guard, |_| Ok(one), |_| Ok(zero), source())
                .unwrap()
        } else {
            one
        };
        let eligibility = eligibility.prepare(&[predicate]).unwrap();
        let mut criterion = builder(&registry, inputs);
        let score = match score {
            Score::Constant(value) => literal(&mut criterion, &registry, value),
            Score::Unknown => input(&mut criterion, &registry, 0),
            Score::ParameterWeightedUnknown => {
                let x = input(&mut criterion, &registry, 0);
                let p = input(&mut criterion, &registry, unknowns);
                criterion.binary(Binary::Mul, p, x, None, source()).unwrap()
            }
        };
        let tolerance = literal(&mut criterion, &registry, tolerance);
        let criterion = criterion.prepare(&[score, tolerance]).unwrap();
        root_isolation_program(
            source(),
            &residual,
            &eligibility,
            &criterion,
            &Arc::new(AtomicBool::new(false)),
            10_000,
        )
        .unwrap()
        .unwrap()
    }
    #[test]
    fn repeated_source_guards_share_native_functions_without_merging_scopes() {
        let mut program = projected(false, true, None);
        let argument = program
            .nodes
            .iter()
            .position(|node| matches!(node, Node::Var(0)))
            .unwrap();
        program.derivative_obligations.push((
            DerivativeOrder::First,
            ProjectedObligation {
                instance: source(),
                source: source(),
                kind: ObligationKind::Require(Condition::Positive),
                scope: ObligationScope::Unconditional,
                argument: Some(argument),
                constraints: vec![],
                represented: true,
                fidelity: Fidelity::Exact,
            },
        ));
        let original = encode(&program, DerivativeOrder::Second).unwrap();
        let obligations = program.obligations.clone();
        let derivatives = program.derivative_obligations.clone();
        for _ in 0..8 {
            program.obligations.extend(obligations.clone());
            program.derivative_obligations.extend(derivatives.clone());
        }
        let repeated = encode(&program, DerivativeOrder::Second).unwrap();
        assert_eq!(repeated.guards.len(), original.guards.len());
        assert!(repeated.guards.iter().any(|guard| guard.chart_only == 0));
        assert!(repeated.guards.iter().any(|guard| guard.chart_only == 1));
        let fixture = Fixture::new(program, &[(0.01, 4.0)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[0.0], &[1.0], DerivativeOrder::Second);
        assert!(matches!(
            Ibex.certify(&request).unwrap(),
            SelectionEvidence::Unique(_)
        ));
    }
    struct Fixture {
        program: Arc<RootIsolationProgram>,
        unknowns: Vec<Unknown>,
        cancel: Arc<AtomicBool>,
    }
    impl Fixture {
        fn new(program: RootIsolationProgram, bounds: &[(f64, f64)]) -> Self {
            Self {
                program: Arc::new(program),
                unknowns: bounds
                    .iter()
                    .enumerate()
                    .map(|(ordinal, &(lower, upper))| Unknown {
                        id: pse_ids::named_id(source(), &format!("unknown-{ordinal}")),
                        lower,
                        upper,
                    })
                    .collect(),
                cancel: Arc::new(AtomicBool::new(false)),
            }
        }
        fn alternative(&self, ordinal: usize) -> SelectionAlternative<'_> {
            SelectionAlternative {
                id: pse_ids::named_id(source(), &format!("alternative-{ordinal}")),
                program: &self.program,
                residual_identity: ContentHash::from_bytes([153; 32]),
                unknowns: &self.unknowns,
            }
        }
        fn request<'a>(
            &'a self,
            alternatives: &'a [SelectionAlternative<'a>],
            parameters: &'a [f64],
            candidate: &'a [f64],
            order: DerivativeOrder,
        ) -> SelectionProofRequest<'a> {
            SelectionProofRequest {
                selection: source(),
                alternatives,
                winner: 0,
                parameters,
                candidate,
                order,
                time_limit: Duration::from_secs(5),
                cancel: &self.cancel,
            }
        }
    }
    fn unique(evidence: SelectionEvidence) -> SelectionChart {
        match evidence {
            SelectionEvidence::Unique(chart) => chart,
            other => panic!("expected complete chart, got {other:?}"),
        }
    }

    #[test]
    fn actual_ibex_single_unknown_no_parameter_affine_root_has_bounded_chart() {
        let fixture = Fixture::new(
            RootIsolationProgram {
                inputs: 1,
                nodes: vec![
                    Node::Var(0),
                    Node::Const(Constant::Rational(Rational::from(-3))),
                    Node::Sum(vec![0, 1]),
                    Node::Const(Constant::Rational(Rational::from(0))),
                ],
                residuals: vec![2],
                eligibility: vec![],
                criterion: [3, 3],
                obligations: vec![],
                derivative_obligations: vec![],
            },
            &[(1.0, 4.0)],
        );
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[], &[3.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify_bounded(&request, 64).unwrap());
        assert!(matches!(
            Ibex.refine_point(&request, &chart, &[1.0], &[1.0], 64)
                .unwrap(),
            RootPointEvidence::Enclosed { .. }
        ));
    }

    #[test]
    fn actual_ibex_initial_chart_and_workspace_consume_declared_cell_ceiling() {
        let program = Arc::new(projected(false, false, None));
        let bounded = Ibex
            .bounded_workspace_bytes(std::slice::from_ref(&program), 256)
            .unwrap();
        let complete = Ibex
            .workspace_bytes(std::slice::from_ref(&program))
            .unwrap();
        assert!(bounded < 64 * 1024 * 1024);
        assert!(complete > 4 * 1024 * 1024 * 1024);
        let fixture = Fixture::new(program.as_ref().clone(), &[(0.01, 3.0)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        assert!(!matches!(
            Ibex.certify_bounded(&request, 0).unwrap(),
            SelectionEvidence::Unique(_)
        ));
        let chart = unique(Ibex.certify_bounded(&request, 256).unwrap());
        chart.validate(&request, Ibex.identity()).unwrap();
    }

    #[test]
    fn actual_ibex_inverse_encloses_selected_ift_action_without_numerical_certificate() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.01, 3.0)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&request).unwrap());
        let (intervals, cells) = match Ibex.enclose_action(&request, &chart, &[2.0]).unwrap() {
            RootActionEvidence::Enclosed {
                intervals,
                proof_cells,
            } => (intervals, proof_cells),
            other => panic!("{other:?}"),
        };
        assert_eq!(cells, 2);
        assert!(intervals[0].contains(0.5));
        assert!(intervals[0].upper - intervals[0].lower < 1e-12);
        let mut expired = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        expired.time_limit = Duration::ZERO;
        assert_eq!(
            Ibex.enclose_action(&expired, &chart, &[2.0]).unwrap(),
            RootActionEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: 0
            }
        );
        assert!(Ibex.enclose_action(&request, &chart, &[f64::NAN]).is_err());
        let candidate = [5.0_f64.sqrt()];
        let changed = fixture.request(&alternatives, &[5.0], &candidate, DerivativeOrder::First);
        assert!(Ibex.enclose_action(&changed, &chart, &[2.0]).is_err());
    }
    #[test]
    fn actual_ibex_fixed_parameter_root_contracts_without_changing_uniform_chart() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.01, 3.0)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&request).unwrap());
        let retained = chart.clone();
        let (intervals, bound, cells) = match Ibex
            .refine_point(&request, &chart, &[2.0], &[8.0], 2)
            .unwrap()
        {
            RootPointEvidence::Enclosed {
                intervals,
                inverse_norm_upper,
                proof_cells,
            } => (intervals, inverse_norm_upper, proof_cells),
            other => panic!("{other:?}"),
        };
        assert_eq!(
            cells, 2,
            "one bounded fixed-parameter existence call and one interval inverse cell"
        );
        assert!(intervals[0].contains(2.0));
        assert!(intervals[0].upper - intervals[0].lower < 1e-12);
        assert!(
            intervals[0].upper - intervals[0].lower
                < (chart.existence[0].upper - chart.existence[0].lower) / 64.0
        );
        assert!(
            (1.0..1.0 + 1e-12).contains(&bound),
            "normalized inverse of Fz=4 with Sz=2, Sr=8: {bound}"
        );
        assert_eq!(chart.existence, retained.existence);
        assert_eq!(chart.uniqueness, retained.uniqueness);
        assert_eq!(chart.parameters, retained.parameters);
        assert_eq!(chart.order, retained.order);
        // The derivative bound covers the segment to the actual approximate root,
        // even when that point is outside the tightly contracted root enclosure.
        let candidate = [(chart.uniqueness[0].lower + chart.existence[0].lower) * 0.5];
        let displaced = fixture.request(&alternatives, &[4.0], &candidate, DerivativeOrder::First);
        let RootPointEvidence::Enclosed {
            inverse_norm_upper, ..
        } = Ibex
            .refine_point(&displaced, &chart, &[2.0], &[8.0], 2)
            .unwrap()
        else {
            panic!("the admitted uniqueness hull must be uniformly regular")
        };
        assert!(candidate[0] < intervals[0].lower);
        assert!(
            inverse_norm_upper > 1.0,
            "inverse at the root alone cannot bound the entire approximate-root segment"
        );
    }
    #[test]
    fn actual_ibex_incoming_neighborhood_encloses_point_and_uncertain_direction() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.01, 3.0)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&request).unwrap());
        let radius = (4.0 - chart.parameters[0].lower).min(chart.parameters[0].upper - 4.0) * 0.25;
        assert!(radius > 0.0);
        let parameters = [ProofInterval {
            lower: 4.0 - radius,
            upper: 4.0 + radius,
        }];
        let directions = [ProofInterval {
            lower: 1.9,
            upper: 2.1,
        }];
        let RootNeighborhoodEvidence::Enclosed {
            points,
            actions: Some(actions),
            proof_cells,
        } = Ibex
            .enclose_neighborhood(&request, &chart, &parameters, Some(&directions), 4)
            .unwrap()
        else {
            panic!("actual uniform incoming point/action enclosure required")
        };
        assert!(proof_cells > 0 && proof_cells <= 4);
        for p in [parameters[0].lower, 4.0, parameters[0].upper] {
            assert!(points[0].contains(p.sqrt()));
            for direction in [1.9, 2.0, 2.1] {
                assert!(actions[0].contains(direction / (2.0 * p.sqrt())));
            }
        }
        let outside = [ProofInterval {
            lower: chart.parameters[0].lower.next_down(),
            upper: 4.0,
        }];
        assert!(matches!(
            Ibex.enclose_neighborhood(&request, &chart, &outside, None, 4)
                .unwrap(),
            RootNeighborhoodEvidence::Incomplete {
                reason: SelectionProofRefusal::Coverage,
                proof_cells: 0
            }
        ));
        assert!(matches!(
            Ibex.enclose_neighborhood(&request, &chart, &parameters, None, 0)
                .unwrap(),
            RootNeighborhoodEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: 0
            }
        ));
    }
    #[test]
    fn actual_ibex_fixed_point_and_action_share_explicit_remaining_proof_limits() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.01, 3.0)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&request).unwrap());
        for cap in [0, 1] {
            assert_eq!(
                Ibex.refine_point(&request, &chart, &[1.0], &[1.0], cap)
                    .unwrap(),
                RootPointEvidence::Incomplete {
                    reason: SelectionProofRefusal::Resource,
                    proof_cells: cap
                }
            );
            assert_eq!(
                Ibex.enclose_action_bounded(&request, &chart, &[2.0], cap)
                    .unwrap(),
                RootActionEvidence::Incomplete {
                    reason: SelectionProofRefusal::Resource,
                    proof_cells: cap
                }
            );
        }
        let mut expired = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        expired.time_limit = Duration::ZERO;
        assert_eq!(
            Ibex.refine_point(&expired, &chart, &[1.0], &[1.0], 2)
                .unwrap(),
            RootPointEvidence::Incomplete {
                reason: SelectionProofRefusal::Resource,
                proof_cells: 0
            }
        );
        assert!(Ibex.refine_point(&request, &chart, &[], &[1.0], 2).is_err());
        assert!(
            Ibex.refine_point(&request, &chart, &[0.0], &[1.0], 2)
                .is_err()
        );
        assert!(
            Ibex.refine_point(&request, &chart, &[1.0], &[f64::NAN], 2)
                .is_err()
        );
        fixture.cancel.store(true, Ordering::Release);
        assert_eq!(
            Ibex.refine_point(&request, &chart, &[1.0], &[1.0], 2)
                .unwrap(),
            RootPointEvidence::Interrupted { proof_cells: 0 }
        );
        fixture.cancel.store(false, Ordering::Release);
        let candidate = [5.0_f64.sqrt()];
        let changed = fixture.request(&alternatives, &[5.0], &candidate, DerivativeOrder::First);
        assert!(
            Ibex.refine_point(&changed, &chart, &[1.0], &[1.0], 2)
                .is_err()
        );
    }
    #[test]
    fn actual_ibex_coupled_uniform_inverse_uses_physical_row_and_coordinate_scales() {
        let fixture = Fixture::new(projected(true, false, None), &[(0.01, 3.0), (0.01, 3.0)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[2.0, 2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&request).unwrap());
        let RootPointEvidence::Enclosed {
            intervals,
            inverse_norm_upper,
            proof_cells,
        } = Ibex
            .refine_point(&request, &chart, &[2.0, 4.0], &[8.0, 3.0], 2)
            .unwrap()
        else {
            panic!("coupled original interval inverse must be enclosed")
        };
        assert_eq!(proof_cells, 2);
        assert!(
            intervals
                .iter()
                .all(|i| i.contains(2.0) && i.upper - i.lower < 1e-12)
        );
        // Fz^-1=[[1/4,0],[1/4,1]], so Sz^-1 Fz^-1 Sr has row sums [1,1.25].
        assert!(
            (1.25..1.25 + 1e-12).contains(&inverse_norm_upper),
            "{inverse_norm_upper}"
        );
        let RootActionEvidence::Enclosed {
            intervals,
            proof_cells,
        } = Ibex
            .enclose_action_bounded(&request, &chart, &[2.0], 2)
            .unwrap()
        else {
            panic!("coupled fixed-root IFT action must be enclosed")
        };
        assert_eq!(proof_cells, 2);
        assert!(
            intervals
                .iter()
                .all(|i| i.contains(0.5) && i.upper - i.lower < 1e-12)
        );
    }
    #[test]
    fn native_finite_chart_chain_moves_beyond_endpoint_chart_and_reports_actual_cells() {
        let fixture = Fixture::new(
            projected_selection(false, Residual::Linear, None, Score::Constant(0.0), 0.0),
            &[(-2.0, 2.0)],
        );
        let alternatives = [fixture.alternative(0)];
        let origin = fixture.request(&alternatives, &[0.0], &[0.0], DerivativeOrder::First);
        let endpoint = fixture.request(&alternatives, &[1.0], &[1.0], DerivativeOrder::First);
        let previous = unique(Ibex.certify(&origin).unwrap());
        let next = unique(Ibex.certify(&endpoint).unwrap());
        assert!(previous.parameters[0].upper < next.parameters[0].lower);
        let chain = ChartChainRequest {
            endpoint: &endpoint,
            previous: &previous,
            next: &next,
            origin: origin.parameters,
            coverage: ChartChainCoverage::RootSheet,
        };
        let proof = match Ibex.connect_chain(&chain).unwrap() {
            ChartChainEvidence::Connected(p) => p,
            other => panic!("{other:?}"),
        };
        assert_eq!(proof.coverage, ChartChainCoverage::RootSheet);
        assert!(proof.charts > 0);
        assert_eq!(proof.connections, proof.charts + 1);
        assert!(proof.proof_cells >= proof.charts + proof.connections);
        assert!(proof.proof_cells <= Ibex.chart_chain_cell_limit());
        assert_eq!(
            Ibex.connect_chain(&ChartChainRequest {
                coverage: ChartChainCoverage::SelectedFunction,
                ..chain
            })
            .unwrap(),
            ChartChainEvidence::Incomplete(SelectionProofRefusal::Coverage)
        );
    }

    #[test]
    fn native_useful_chart_serves_neighboring_inputs_with_original_scope() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.01, 3.0)]);
        let alternatives = [fixture.alternative(0)];
        let origin = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&origin).unwrap());
        let retained = chart.clone();
        for parameter in [4.0_f64 - 4e-6, 4.0_f64 + 4e-6] {
            let parameters = [parameter];
            let candidate = [parameter.sqrt()];
            let request = fixture.request(
                &alternatives,
                &parameters,
                &candidate,
                DerivativeOrder::First,
            );
            chart.validate(&request, Ibex.identity()).unwrap();
            // Reuse the same selected chart; only the demanded fixed-point
            // enclosure is produced. No second competitive covering is invoked.
            let RootPointEvidence::Enclosed {
                intervals,
                proof_cells,
                ..
            } = Ibex
                .refine_point(&request, &chart, &[1.0], &[1.0], 2)
                .unwrap()
            else {
                panic!("neighbor inside the certified chart must admit fixed-point refinement")
            };
            assert_eq!(proof_cells, 2);
            assert!(intervals[0].contains(candidate[0]));
            assert!(intervals[0].upper - intervals[0].lower < 1e-12);
        }
        assert_eq!(chart.parameters, retained.parameters);
        assert_eq!(chart.existence, retained.existence);
        assert_eq!(chart.uniqueness, retained.uniqueness);
        let changed_unknowns = [Unknown {
            upper: 4.0,
            ..fixture.unknowns[0].clone()
        }];
        let changed_alternatives = [SelectionAlternative {
            unknowns: &changed_unknowns,
            ..fixture.alternative(0)
        }];
        let changed = fixture.request(
            &changed_alternatives,
            &[4.0],
            &[2.0],
            DerivativeOrder::First,
        );
        assert!(chart.validate(&changed, Ibex.identity()).is_err());
    }

    #[test]
    fn native_useful_chart_refuses_reuse_across_a_competitive_score_boundary() {
        let fixture = Fixture::new(
            projected_selection(
                false,
                Residual::ConstantTwoRoots,
                None,
                Score::ParameterWeightedUnknown,
                0.0,
            ),
            &[(-2.0, 2.0)],
        );
        let alternatives = [fixture.alternative(0)];
        let origin = fixture.request(&alternatives, &[1e-6], &[-1.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&origin).unwrap());
        // Scores p*x switch the selected root at p=0. A wider proposal must
        // narrow or refuse; the returned certificate cannot cross that switch.
        assert!(chart.parameters[0].lower > 0.0);
        assert!(chart.uniqueness[0].upper < 0.0);
        let opposite = fixture.request(&alternatives, &[-1e-6], &[1.0], DerivativeOrder::First);
        assert!(chart.validate(&opposite, Ibex.identity()).is_err());
        let other = unique(Ibex.certify(&opposite).unwrap());
        assert!(other.parameters[0].upper < 0.0);
        assert!(other.uniqueness[0].lower > 0.0);
    }

    #[test]
    fn native_useful_chart_anisotropic_seed_preserves_small_log_domain_and_large_response() {
        let registry = standard_registry().unwrap();
        let mut residual = builder(&registry, 4);
        let small = input(&mut residual, &registry, 0);
        let large = input(&mut residual, &registry, 1);
        let zero = input(&mut residual, &registry, 2);
        let parameter = input(&mut residual, &registry, 3);
        let small_constant = literal(&mut residual, &registry, 1e-8);
        let small_log = residual
            .unary(Function::Log, small.clone(), source())
            .unwrap();
        let constant_log = residual
            .unary(Function::Log, small_constant.clone(), source())
            .unwrap();
        let first = residual
            .binary(Binary::Sub, small_log, constant_log, None, source())
            .unwrap();
        let scale = literal(&mut residual, &registry, 1e5);
        let scaled_large = residual
            .binary(Binary::Div, large, scale, None, source())
            .unwrap();
        let product = residual
            .binary(Binary::Mul, scaled_large, small, None, source())
            .unwrap();
        let target = residual
            .binary(Binary::Mul, small_constant, parameter, None, source())
            .unwrap();
        let product_log = residual.unary(Function::Log, product, source()).unwrap();
        let target_log = residual.unary(Function::Log, target, source()).unwrap();
        let second = residual
            .binary(Binary::Sub, product_log, target_log, None, source())
            .unwrap();
        let residual = residual.prepare(&[first, second, zero]).unwrap();
        let mut eligibility = builder(&registry, 4);
        let one = literal(&mut eligibility, &registry, 1.0);
        let eligibility = eligibility.prepare(&[one]).unwrap();
        let mut criterion = builder(&registry, 4);
        let score = literal(&mut criterion, &registry, 0.0);
        let tolerance = literal(&mut criterion, &registry, 0.0);
        let criterion = criterion.prepare(&[score, tolerance]).unwrap();
        let program = root_isolation_program(
            source(),
            &residual,
            &eligibility,
            &criterion,
            &Arc::new(AtomicBool::new(false)),
            10_000,
        )
        .unwrap()
        .unwrap();
        let fixture = Fixture::new(program, &[(1e-10, 1e-6), (5e4, 2e5), (-1.0, 1.0)]);
        let alternatives = [fixture.alternative(0)];
        let origin = fixture.request(
            &alternatives,
            &[1.0],
            &[1e-8, 1e5, 0.0],
            DerivativeOrder::First,
        );
        let chart = unique(Ibex.certify(&origin).unwrap());
        // The useful input box requires order-one movement of the large root,
        // while the small component must stay on its original positive log domain.
        for parameter in [1.0_f64 - 1e-6, 1.0_f64 + 1e-6] {
            let parameters = [parameter];
            let candidate = [1e-8, 1e5 * parameter, 0.0];
            let neighboring = fixture.request(
                &alternatives,
                &parameters,
                &candidate,
                DerivativeOrder::First,
            );
            chart.validate(&neighboring, Ibex.identity()).unwrap();
            assert!(chart.uniqueness[0].lower > 0.0);
            assert!(chart.uniqueness[1].contains(candidate[1]));
        }
        let invalid = fixture.request(
            &alternatives,
            &[0.0],
            &[1e-8, 1e5, 0.0],
            DerivativeOrder::First,
        );
        assert!(chart.validate(&invalid, Ibex.identity()).is_err());
        assert!(matches!(
            Ibex.certify(&invalid).unwrap(),
            SelectionEvidence::Incomplete(SelectionProofRefusal::Boundary)
        ));
    }
    #[test]
    fn native_finite_chart_chain_connects_a_nonlinear_root_beyond_the_previous_seed() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.5, 2.5)]);
        let alternatives = [fixture.alternative(0)];
        let origin = fixture.request(&alternatives, &[1.0], &[1.0], DerivativeOrder::First);
        let endpoint = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let previous = unique(Ibex.certify(&origin).unwrap());
        let next = unique(Ibex.certify(&endpoint).unwrap());
        assert!(previous.existence[0].upper < next.existence[0].lower);
        assert!(previous.parameters[0].upper < next.parameters[0].lower);
        let proof = match Ibex
            .connect_chain(&ChartChainRequest {
                endpoint: &endpoint,
                previous: &previous,
                next: &next,
                origin: origin.parameters,
                coverage: ChartChainCoverage::RootSheet,
            })
            .unwrap()
        {
            ChartChainEvidence::Connected(proof) => proof,
            other => panic!("positive square-root sheet must connect: {other:?}"),
        };
        assert_eq!(proof.coverage, ChartChainCoverage::RootSheet);
        assert!(proof.charts > 0);
        assert_eq!(proof.connections, proof.charts + 1);
        assert!(proof.proof_cells >= proof.charts + proof.connections);
        assert!(proof.proof_cells <= Ibex.chart_chain_cell_limit());
    }
    #[test]
    fn native_finite_chart_chain_refuses_disconnected_same_regime_selected_roots() {
        let fixture = Fixture::new(
            projected_selection(
                false,
                Residual::ConstantTwoRoots,
                None,
                Score::ParameterWeightedUnknown,
                0.0,
            ),
            &[(-2.0, 2.0)],
        );
        let alternatives = [fixture.alternative(0)];
        let origin = fixture.request(&alternatives, &[-1.0], &[1.0], DerivativeOrder::First);
        let mut endpoint = fixture.request(&alternatives, &[1.0], &[-1.0], DerivativeOrder::First);
        let previous = unique(Ibex.certify(&origin).unwrap());
        let next = unique(Ibex.certify(&endpoint).unwrap());
        endpoint.time_limit = Duration::from_millis(100);
        let chain = ChartChainRequest {
            endpoint: &endpoint,
            previous: &previous,
            next: &next,
            origin: origin.parameters,
            coverage: ChartChainCoverage::RootSheet,
        };
        let evidence = Ibex.connect_chain(&chain).unwrap();
        assert!(matches!(
            evidence,
            ChartChainEvidence::Incomplete(_) | ChartChainEvidence::Refused { .. }
        ));
        assert!(evidence.work().proof_cells > 0);
        assert!(evidence.work().proof_cells <= Ibex.chart_chain_cell_limit());
    }

    #[test]
    fn native_finite_chart_chain_connects_pressure_scaled_constant_mixed_root() {
        // The pressure and temperature factors cancel mathematically, but remain
        // inside independent logarithmic terms in the actual interval program.
        // Five unknowns include an exact zero and two coupled normalized outputs.
        let registry = standard_registry().unwrap();
        let mut residual = builder(&registry, 9);
        let coordinates: Vec<_> = (0..9).map(|i| input(&mut residual, &registry, i)).collect();
        let liquid_one = residual
            .binary(
                Binary::Sub,
                coordinates[1].clone(),
                coordinates[5].clone(),
                None,
                source(),
            )
            .unwrap();
        let liquid_two = residual
            .binary(
                Binary::Sub,
                coordinates[2].clone(),
                coordinates[8].clone(),
                None,
                source(),
            )
            .unwrap();
        let first_pressure = residual
            .binary(
                Binary::Mul,
                coordinates[7].clone(),
                coordinates[3].clone(),
                None,
                source(),
            )
            .unwrap();
        let second_pressure = residual
            .binary(
                Binary::Mul,
                coordinates[7].clone(),
                coordinates[4].clone(),
                None,
                source(),
            )
            .unwrap();
        let first_scaled = residual
            .binary(
                Binary::Div,
                first_pressure,
                coordinates[6].clone(),
                None,
                source(),
            )
            .unwrap();
        let second_scaled = residual
            .binary(
                Binary::Div,
                second_pressure,
                coordinates[6].clone(),
                None,
                source(),
            )
            .unwrap();
        let first_log = residual
            .unary(Function::Log, first_scaled, source())
            .unwrap();
        let second_log = residual
            .unary(Function::Log, second_scaled, source())
            .unwrap();
        let output_ratio = residual
            .binary(Binary::Sub, first_log, second_log, None, source())
            .unwrap();
        let input_ratio = residual
            .binary(
                Binary::Div,
                coordinates[1].clone(),
                coordinates[2].clone(),
                None,
                source(),
            )
            .unwrap();
        let factor = literal(&mut residual, &registry, 0.4);
        let input_ratio = residual
            .binary(Binary::Mul, factor, input_ratio, None, source())
            .unwrap();
        let input_log = residual
            .unary(Function::Log, input_ratio, source())
            .unwrap();
        let equilibrium = residual
            .binary(Binary::Sub, output_ratio, input_log, None, source())
            .unwrap();
        let sum = residual
            .binary(
                Binary::Add,
                coordinates[3].clone(),
                coordinates[4].clone(),
                None,
                source(),
            )
            .unwrap();
        let one = literal(&mut residual, &registry, 1.0);
        let normalization = residual
            .binary(Binary::Sub, sum, one, None, source())
            .unwrap();
        let residual = residual
            .prepare(&[
                coordinates[0].clone(),
                liquid_one,
                liquid_two,
                equilibrium,
                normalization,
            ])
            .unwrap();
        let mut eligibility = builder(&registry, 9);
        let one = literal(&mut eligibility, &registry, 1.0);
        let eligibility = eligibility.prepare(&[one]).unwrap();
        let mut criterion = builder(&registry, 9);
        let score = literal(&mut criterion, &registry, 0.0);
        let tolerance = literal(&mut criterion, &registry, 0.0);
        let criterion = criterion.prepare(&[score, tolerance]).unwrap();
        let program = root_isolation_program(
            source(),
            &residual,
            &eligibility,
            &criterion,
            &Arc::new(AtomicBool::new(false)),
            10_000,
        )
        .unwrap()
        .unwrap();
        let bounds = [
            (-20.0, 20.0),
            (1e-10, 1.0),
            (1e-10, 1.0),
            (1e-10, 1.0),
            (1e-10, 1.0),
        ];
        let fixture = Fixture::new(program, &bounds);
        let alternatives = [fixture.alternative(0)];
        let candidate = [0.0, 0.5, 0.5, 0.4 / 1.4, 1.0 / 1.4];
        let origin_parameters = [0.5, 358.0, 101325.0, 0.5];
        let endpoint_parameters = [0.5, 358.0, 101330.0, 0.5];
        let origin = fixture.request(
            &alternatives,
            &origin_parameters,
            &candidate,
            DerivativeOrder::First,
        );
        let endpoint = fixture.request(
            &alternatives,
            &endpoint_parameters,
            &candidate,
            DerivativeOrder::First,
        );
        let previous = unique(Ibex.certify(&origin).unwrap());
        let next = unique(Ibex.certify(&endpoint).unwrap());
        assert!(previous.parameters[2].upper < next.parameters[2].lower);
        for (i, (a, b)) in previous.existence.iter().zip(&next.existence).enumerate() {
            assert!(
                a.lower <= b.upper && b.lower <= a.upper,
                "the root stays fixed: {a:?} {b:?}"
            );
            assert!(
                a.upper - a.lower < previous.uniqueness[i].upper - previous.uniqueness[i].lower
            );
            assert!(b.upper - b.lower < next.uniqueness[i].upper - next.uniqueness[i].lower);
        }
        assert_eq!(previous.existence[0].lower, 0.0);
        assert_eq!(previous.existence[0].upper, 0.0);
        let retained_previous = previous.clone();
        let retained_next = next.clone();
        let proof = match Ibex
            .connect_chain(&ChartChainRequest {
                endpoint: &endpoint,
                previous: &previous,
                next: &next,
                origin: &origin_parameters,
                coverage: ChartChainCoverage::RootSheet,
            })
            .unwrap()
        {
            ChartChainEvidence::Connected(proof) => proof,
            other => panic!("pressure-scaled constant mixed root must connect: {other:?}"),
        };
        assert_eq!(proof.coverage, ChartChainCoverage::RootSheet);
        assert!(proof.charts > 0);
        assert_eq!(proof.connections, proof.charts + 1);
        assert!(proof.proof_cells >= proof.charts + proof.connections);
        assert!(proof.proof_cells <= Ibex.chart_chain_cell_limit());
        for (chart, retained) in [(&previous, &retained_previous), (&next, &retained_next)] {
            assert_eq!(chart.parameters, retained.parameters);
            assert_eq!(chart.existence, retained.existence);
            assert_eq!(chart.uniqueness, retained.uniqueness);
            assert_eq!(chart.order, retained.order);
        }
        for (unknown, bound) in fixture.unknowns.iter().zip(bounds) {
            assert_eq!((unknown.lower, unknown.upper), bound);
        }
    }
    #[test]
    fn native_finite_chart_chain_preserves_scope_and_outer_controls() {
        let fixture = Fixture::new(
            projected_selection(false, Residual::Linear, None, Score::Constant(0.0), 0.0),
            &[(-2.0, 2.0)],
        );
        let alternatives = [fixture.alternative(0)];
        let origin = fixture.request(&alternatives, &[0.0], &[0.0], DerivativeOrder::First);
        let mut endpoint = fixture.request(&alternatives, &[1.0], &[1.0], DerivativeOrder::First);
        let previous = unique(Ibex.certify(&origin).unwrap());
        let next = unique(Ibex.certify(&endpoint).unwrap());
        endpoint.time_limit = Duration::ZERO;
        let chain = ChartChainRequest {
            endpoint: &endpoint,
            previous: &previous,
            next: &next,
            origin: origin.parameters,
            coverage: ChartChainCoverage::RootSheet,
        };
        assert_eq!(
            Ibex.connect_chain(&chain).unwrap(),
            ChartChainEvidence::Incomplete(SelectionProofRefusal::Resource)
        );
        fixture.cancel.store(true, Ordering::Release);
        assert!(matches!(
            Ibex.connect_chain(&chain),
            Err(MathError::Cancelled)
        ));
        fixture.cancel.store(false, Ordering::Release);
        let mut changed = fixture.unknowns.clone();
        changed[0].upper = 2.5;
        let changed_alternatives = [SelectionAlternative {
            unknowns: &changed,
            ..fixture.alternative(0)
        }];
        let changed_request = fixture.request(
            &changed_alternatives,
            &[1.0],
            &[1.0],
            DerivativeOrder::First,
        );
        assert!(
            Ibex.connect_chain(&ChartChainRequest {
                endpoint: &changed_request,
                ..chain
            })
            .is_err()
        );
    }
    #[test]
    fn projected_multiple_roots_refuse_first_derivatives() {
        let fixture = Fixture::new(projected(false, false, None), &[(-2.5, 2.5)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let evidence = Ibex.certify(&request).unwrap();
        assert!(
            matches!(
                evidence,
                SelectionEvidence::Incomplete(_) | SelectionEvidence::Multiple
            ),
            "{evidence:?}"
        );
    }

    #[test]
    fn projected_coupled_second_order_chart_excludes_the_whole_complement() {
        let fixture = Fixture::new(projected(true, false, None), &[(0.5, 2.5), (0.5, 2.5)]);
        assert_eq!(fixture.program.inputs, 3);
        assert_eq!(fixture.program.residuals.len(), 2);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[2.0, 2.0], DerivativeOrder::Second);
        let chart = unique(Ibex.certify(&request).unwrap());
        chart.validate(&request, Ibex.identity()).unwrap();
        assert_eq!(chart.order, DerivativeOrder::Second);
        assert!(chart.parameters[0].interior_contains(4.0));
        for interval in chart.existence {
            assert!(interval.contains(2.0));
        }
    }

    #[test]
    fn projected_mixed_scale_coupled_chart_preserves_parameter_neighborhood() {
        let registry = standard_registry().unwrap();
        let mut residual = builder(&registry, 3);
        let density = input(&mut residual, &registry, 0);
        let fraction = input(&mut residual, &registry, 1);
        let parameter = input(&mut residual, &registry, 2);
        let scale = literal(&mut residual, &registry, 10_000.0);
        let scaled = residual
            .binary(Binary::Div, density, scale, None, source())
            .unwrap();
        let square = residual
            .binary(Binary::Mul, scaled.clone(), scaled.clone(), None, source())
            .unwrap();
        let first = residual
            .binary(Binary::Sub, square, parameter, None, source())
            .unwrap();
        let half = literal(&mut residual, &registry, 0.5);
        let coupled = residual
            .binary(Binary::Mul, half, scaled, None, source())
            .unwrap();
        let second = residual
            .binary(Binary::Sub, fraction, coupled, None, source())
            .unwrap();
        let residual = residual.prepare(&[first, second]).unwrap();
        let mut eligibility = builder(&registry, 3);
        let one = literal(&mut eligibility, &registry, 1.0);
        let eligibility = eligibility.prepare(&[one]).unwrap();
        let mut criterion = builder(&registry, 3);
        let zero = literal(&mut criterion, &registry, 0.0);
        let criterion = criterion.prepare(&[zero.clone(), zero]).unwrap();
        let program = root_isolation_program(
            source(),
            &residual,
            &eligibility,
            &criterion,
            &Arc::new(AtomicBool::new(false)),
            10_000,
        )
        .unwrap()
        .unwrap();
        let fixture = Fixture::new(program, &[(5_000.0, 11_000.0), (0.0, 1.0)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(
            &alternatives,
            &[1.0],
            &[10_000.0, 0.5],
            DerivativeOrder::Second,
        );
        let chart = unique(Ibex.certify(&request).unwrap());
        chart.validate(&request, Ibex.identity()).unwrap();
        assert!(chart.parameters[0].interior_contains(1.0));
        assert!(chart.existence[0].contains(10_000.0));
        assert!(chart.existence[1].contains(0.5));
        let mut expired = request;
        expired.time_limit = Duration::ZERO;
        assert!(matches!(
            Ibex.certify(&expired).unwrap(),
            SelectionEvidence::Incomplete(SelectionProofRefusal::Resource)
        ));
    }

    #[test]
    fn projected_log_chart_handles_an_initial_crossing_domain_without_false_absence() {
        let fixture = Fixture::new(projected(false, true, None), &[(-1.0, 3.0)]);
        assert!(!fixture.program.obligations.is_empty());
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[0.0], &[1.0], DerivativeOrder::Second);
        let chart = unique(Ibex.certify(&request).unwrap());
        chart.validate(&request, Ibex.identity()).unwrap();
        assert!(chart.uniqueness[0].lower > 0.0);
        assert!(chart.existence[0].contains(1.0));
    }

    #[test]
    fn projected_strict_eligibility_boundary_refuses_a_neighborhood_chart() {
        let fixture = Fixture::new(projected(false, false, Some(2.0)), &[(0.5, 2.5)]);
        let alternatives = [fixture.alternative(0)];
        assert!(
            fixture
                .program
                .eligibility
                .iter()
                .any(|constraint| constraint.strict)
        );
        let evidence = Ibex
            .certify(&fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First))
            .unwrap();
        // Parameters on either side of 4 move the positive root across x>2.
        assert!(
            matches!(evidence, SelectionEvidence::Incomplete(_)),
            "{evidence:?}"
        );
    }

    #[test]
    fn projected_physical_face_refuses_first_derivatives() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.5, 2.0)]);
        let alternatives = [fixture.alternative(0)];
        let evidence = Ibex
            .certify(&fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First))
            .unwrap();
        assert!(
            matches!(evidence, SelectionEvidence::Incomplete(_)),
            "{evidence:?}"
        );
    }

    #[test]
    fn chart_belongs_to_its_actual_parameter_neighborhood_and_residual_identity() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.5, 4.0)]);
        let alternatives = [fixture.alternative(0)];
        let first_request = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let first = unique(Ibex.certify(&first_request).unwrap());
        let changed_request =
            fixture.request(&alternatives, &[9.0], &[3.0], DerivativeOrder::First);
        assert!(first.validate(&changed_request, Ibex.identity()).is_err());
        let changed = unique(Ibex.certify(&changed_request).unwrap());
        assert!(changed.existence[0].contains(3.0));
        assert!(!changed.parameters[0].contains(4.0));
        let mut changed_alternatives = [fixture.alternative(0)];
        changed_alternatives[0].residual_identity = ContentHash::from_bytes([154; 32]);
        let changed_identity = fixture.request(
            &changed_alternatives,
            &[4.0],
            &[2.0],
            DerivativeOrder::First,
        );
        assert!(first.validate(&changed_identity, Ibex.identity()).is_err());
    }
    #[test]
    fn retained_chart_promotes_guard_order_without_repeating_competitive_covering() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.5, 4.0)]);
        let alternatives = [fixture.alternative(0)];
        let first = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&first).unwrap());
        let second = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::Second);
        assert!(chart.validate_scope(&second, Ibex.identity()).is_ok());
        assert!(chart.validate(&second, Ibex.identity()).is_err());
        let promoted = unique(Ibex.promote(&second, &chart).unwrap());
        assert_eq!(promoted.order, DerivativeOrder::Second);
        assert_eq!(promoted.parameters, chart.parameters);
        assert_eq!(promoted.existence, chart.existence);
        assert_eq!(promoted.uniqueness, chart.uniqueness);
        let mut program = projected(false, false, None);
        program.nodes.push(Node::Const(Constant::Float(-1.0)));
        let argument = program.nodes.len() - 1;
        program.derivative_obligations.push((
            DerivativeOrder::Second,
            ProjectedObligation {
                instance: source(),
                source: source(),
                kind: ObligationKind::Require(Condition::Positive),
                scope: ObligationScope::Unconditional,
                argument: Some(argument),
                constraints: vec![],
                represented: true,
                fidelity: Fidelity::Exact,
            },
        ));
        let guarded = Fixture::new(program, &[(0.5, 4.0)]);
        let alternatives = [guarded.alternative(0)];
        let first = guarded.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&first).unwrap());
        let second = guarded.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::Second);
        assert!(matches!(
            Ibex.promote(&second, &chart).unwrap(),
            SelectionEvidence::Incomplete(SelectionProofRefusal::Boundary)
        ));
        assert_eq!(chart.order, DerivativeOrder::First);
    }
    #[test]
    fn retained_uniform_charts_connect_through_an_ibex_certified_common_root() {
        let fixture = Fixture::new(projected(false, false, None), &[(0.5, 4.0)]);
        let alternatives = [fixture.alternative(0)];
        let first = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let a = unique(Ibex.certify(&first).unwrap());
        let p = 4.0f64 + 1e-9;
        let moved_parameters = [p];
        let moved_root = [p.sqrt()];
        let moved = fixture.request(
            &alternatives,
            &moved_parameters,
            &moved_root,
            DerivativeOrder::First,
        );
        let b = unique(Ibex.certify(&moved).unwrap());
        let common = 4.0f64 + 0.5e-9;
        let common_parameters = [common];
        let common_root = [common.sqrt()];
        let request = fixture.request(
            &alternatives,
            &common_parameters,
            &common_root,
            DerivativeOrder::First,
        );
        assert!(Ibex.connect(&request, &a, &b).unwrap());
    }

    #[test]
    fn projected_worse_multiroot_and_singular_rivals_allow_the_regular_winner() {
        let winner = Fixture::new(projected(false, false, None), &[(0.5, 2.5)]);
        for shape in [Residual::Square, Residual::Singular] {
            let rival = Fixture::new(
                projected_selection(false, shape, None, Score::Constant(2.0), 0.0),
                &[(-2.5, 2.5)],
            );
            // The rival has roots {-2, 2} or the singular root {0}. None can
            // compete with the regular winner's score of zero.
            let alternatives = [winner.alternative(0), rival.alternative(1)];
            let request = winner.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::Second);
            let chart = unique(Ibex.certify(&request).unwrap());
            chart.validate(&request, Ibex.identity()).unwrap();
            assert_eq!(chart.alternatives.len(), 2);
            assert_eq!(chart.winner, 0);
            assert_eq!(chart.order, DerivativeOrder::Second);
            assert!(chart.existence[0].contains(2.0));
        }
    }

    #[test]
    fn projected_tied_better_and_tolerance_competitors_refuse_selection() {
        let winner = Fixture::new(projected(false, false, None), &[(0.5, 2.5)]);
        for (score, tolerance) in [(0.0, 0.0), (-1.0, 0.0), (2.0, 2.0)] {
            let rival = Fixture::new(
                projected_selection(
                    false,
                    Residual::Square,
                    None,
                    Score::Constant(score),
                    tolerance,
                ),
                &[(-2.5, 2.5)],
            );
            let alternatives = [winner.alternative(0), rival.alternative(1)];
            let request = winner.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
            let evidence = Ibex.certify(&request).unwrap();
            // A retained covering cell refuses proof; it does not establish Multiple.
            assert!(
                matches!(
                    evidence,
                    SelectionEvidence::Incomplete(SelectionProofRefusal::Coverage)
                ),
                "score={score}, tolerance={tolerance}: {evidence:?}"
            );
        }
    }

    #[test]
    fn projected_singular_winner_cannot_supply_derivative_permission() {
        let fixture = Fixture::new(
            projected_selection(false, Residual::Singular, None, Score::Constant(0.0), 0.0),
            &[(-1.0, 1.0)],
        );
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[0.0], DerivativeOrder::First);
        assert!(matches!(
            Ibex.certify(&request).unwrap(),
            SelectionEvidence::Incomplete(SelectionProofRefusal::Chart)
        ));
    }

    #[test]
    fn projected_negative_rival_tolerance_is_unsupported() {
        let winner = Fixture::new(projected(false, false, None), &[(0.5, 2.5)]);
        let rival = Fixture::new(
            projected_selection(false, Residual::Singular, None, Score::Constant(2.0), -1.0),
            &[(-2.5, 2.5)],
        );
        let alternatives = [winner.alternative(0), rival.alternative(1)];
        let request = winner.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        assert!(matches!(
            Ibex.certify(&request).unwrap(),
            SelectionEvidence::Incomplete(SelectionProofRefusal::Unsupported)
        ));
    }

    #[test]
    fn projected_winning_regime_can_contain_a_noncompetitive_second_root() {
        let fixture = Fixture::new(
            projected_selection(false, Residual::Square, None, Score::Unknown, 0.0),
            &[(-2.5, 2.5)],
        );
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[4.0], &[-2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&request).unwrap());
        chart.validate(&request, Ibex.identity()).unwrap();
        assert!(chart.existence[0].contains(-2.0));
        let wrong_winner = fixture.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        assert!(matches!(
            Ibex.certify(&wrong_winner).unwrap(),
            SelectionEvidence::Incomplete(SelectionProofRefusal::Coverage)
        ));
        assert!(chart.validate(&wrong_winner, Ibex.identity()).is_err());
    }

    #[test]
    fn projected_exact_root_with_no_independent_parameters_has_a_scoped_chart() {
        let registry = standard_registry().unwrap();
        let mut residual = builder(&registry, 1);
        let x = input(&mut residual, &registry, 0);
        let square = residual
            .binary(Binary::Mul, x.clone(), x, None, source())
            .unwrap();
        let four = literal(&mut residual, &registry, 4.0);
        let equation = residual
            .binary(Binary::Sub, square, four, None, source())
            .unwrap();
        let residual = residual.prepare(&[equation]).unwrap();
        let mut eligibility = builder(&registry, 1);
        let one = literal(&mut eligibility, &registry, 1.0);
        let eligibility = eligibility.prepare(&[one]).unwrap();
        let mut criterion = builder(&registry, 1);
        let zero = literal(&mut criterion, &registry, 0.0);
        let criterion = criterion.prepare(&[zero.clone(), zero]).unwrap();
        let program = root_isolation_program(
            source(),
            &residual,
            &eligibility,
            &criterion,
            &Arc::new(AtomicBool::new(false)),
            10_000,
        )
        .unwrap()
        .unwrap();
        let fixture = Fixture::new(program, &[(0.5, 2.5)]);
        let alternatives = [fixture.alternative(0)];
        let request = fixture.request(&alternatives, &[], &[2.0], DerivativeOrder::Second);
        let chart = unique(Ibex.certify(&request).unwrap());
        assert!(chart.parameters.is_empty());
        assert!(chart.existence[0].contains(2.0));
        chart.validate(&request, Ibex.identity()).unwrap();
    }

    #[test]
    fn chart_scope_includes_every_alternatives_source_criterion_and_physical_bounds() {
        let winner = Fixture::new(projected(false, false, None), &[(0.5, 2.5)]);
        let rival = Fixture::new(
            projected_selection(false, Residual::Square, None, Score::Constant(2.0), 0.0),
            &[(-2.5, 2.5)],
        );
        let alternatives = [winner.alternative(0), rival.alternative(1)];
        let request = winner.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        let chart = unique(Ibex.certify(&request).unwrap());
        chart.validate(&request, Ibex.identity()).unwrap();

        let mut changed_selection =
            winner.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        changed_selection.selection = pse_ids::named_id(source(), "another-selection");
        assert!(chart.validate(&changed_selection, Ibex.identity()).is_err());
        let mut changed_winner =
            winner.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        changed_winner.winner = 1;
        assert!(chart.validate(&changed_winner, Ibex.identity()).is_err());
        let higher_order = winner.request(&alternatives, &[4.0], &[2.0], DerivativeOrder::Second);
        assert!(chart.validate(&higher_order, Ibex.identity()).is_err());
        assert!(
            chart
                .validate(&request, ContentHash::from_bytes([0; 32]))
                .is_err()
        );

        let reversed = [rival.alternative(1), winner.alternative(0)];
        let reversed_request = winner.request(&reversed, &[4.0], &[2.0], DerivativeOrder::First);
        assert!(chart.validate(&reversed_request, Ibex.identity()).is_err());
        let omitted = [winner.alternative(0)];
        let omitted_request = winner.request(&omitted, &[4.0], &[2.0], DerivativeOrder::First);
        assert!(chart.validate(&omitted_request, Ibex.identity()).is_err());

        let mut changed_id = [winner.alternative(0), rival.alternative(1)];
        changed_id[1].id = pse_ids::named_id(source(), "another-rival");
        let changed_id_request =
            winner.request(&changed_id, &[4.0], &[2.0], DerivativeOrder::First);
        assert!(
            chart
                .validate(&changed_id_request, Ibex.identity())
                .is_err()
        );
        let mut changed_residual = [winner.alternative(0), rival.alternative(1)];
        changed_residual[1].residual_identity = ContentHash::from_bytes([154; 32]);
        let changed_residual_request =
            winner.request(&changed_residual, &[4.0], &[2.0], DerivativeOrder::First);
        assert!(
            chart
                .validate(&changed_residual_request, Ibex.identity())
                .is_err()
        );

        // These are new exact typed projections, including the same residual
        // with a changed score or tolerance and an entirely changed source DAG.
        for program in [
            projected_selection(false, Residual::Square, None, Score::Constant(3.0), 0.0),
            projected_selection(false, Residual::Square, None, Score::Constant(2.0), 0.25),
            projected_selection(false, Residual::Log, None, Score::Constant(2.0), 0.0),
        ] {
            let replacement = Fixture::new(program, &[(-2.5, 2.5)]);
            let changed_source = [winner.alternative(0), replacement.alternative(1)];
            let changed_source_request =
                winner.request(&changed_source, &[4.0], &[2.0], DerivativeOrder::First);
            assert!(
                chart
                    .validate(&changed_source_request, Ibex.identity())
                    .is_err()
            );
        }
        let mut changed_bounds = rival.unknowns.clone();
        changed_bounds[0].upper = 2.6;
        let mut bounds_alternatives = [winner.alternative(0), rival.alternative(1)];
        bounds_alternatives[1].unknowns = &changed_bounds;
        let changed_bounds_request =
            winner.request(&bounds_alternatives, &[4.0], &[2.0], DerivativeOrder::First);
        assert!(
            chart
                .validate(&changed_bounds_request, Ibex.identity())
                .is_err()
        );
        let mut changed_coordinates = rival.unknowns.clone();
        changed_coordinates[0].id = pse_ids::named_id(source(), "another-unknown");
        let mut coordinate_alternatives = [winner.alternative(0), rival.alternative(1)];
        coordinate_alternatives[1].unknowns = &changed_coordinates;
        let changed_coordinates_request = winner.request(
            &coordinate_alternatives,
            &[4.0],
            &[2.0],
            DerivativeOrder::First,
        );
        assert!(
            chart
                .validate(&changed_coordinates_request, Ibex.identity())
                .is_err()
        );
    }

    #[test]
    fn projected_empty_rival_does_not_defeat_the_winner_and_cancellation_is_preserved() {
        let rival = Fixture::new(projected(false, false, None), &[(0.5, 2.5)]);
        let candidate = [(-4.0_f64).exp()];
        // log(x)=-4 has a regular winner; x²=-4 has no real rival root.
        let fixture = Fixture::new(projected(false, true, None), &[(0.001, 4.0)]);
        let alternatives = [fixture.alternative(0), rival.alternative(1)];
        let request = fixture.request(&alternatives, &[-4.0], &candidate, DerivativeOrder::First);
        let chart = unique(Ibex.certify(&request).unwrap());
        chart.validate(&request, Ibex.identity()).unwrap();
        fixture.cancel.store(true, Ordering::Relaxed);
        assert!(matches!(Ibex.certify(&request), Err(MathError::Cancelled)));
    }
}
