// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Request-local IBEX validated covering; numerical proposals supply no proof authority.
use pse_ids::ContentHash;
use pse_kernels::DerivativeOrder;
use pse_math::{
    MathError,
    factorable::{
        Constant, Constraint, Fidelity, Node, ObligationKind, ObligationScope, ProjectedObligation,
        Rational, RootIsolationProgram,
    },
    guarded::Condition,
    implicit::{
        ProofInterval, SelectionChart, SelectionEvidence, SelectionProofRefusal,
        SelectionProofRequest, SelectionScope, SelectionVerifier,
    },
};
use std::{
    ffi::c_void,
    sync::{
        Arc, Mutex, TryLockError,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

const MAX_CELLS: u64 = 1_048_576;
const MAX_NODES: usize = 65_536;
const MAX_EDGES: usize = 1_048_576;
const MAX_GUARDS: usize = 8192;
const MAX_INPUTS: usize = 128;
const MAX_ALTERNATIVES: usize = 64;
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
struct NativeResult {
    status: u32,
    cells: u64,
    solution: u64,
    boundary: u64,
    unknown: u64,
    pending: u64,
}
#[expect(
    unsafe_code,
    reason = "bounded request buffers are borrowed synchronously by the catch-all C++ adapter"
)]
unsafe extern "C" {
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
impl SelectionVerifier for Ibex {
    fn identity(&self) -> ContentHash {
        pse_math::implicit::solver_identity(&format!(
            "ibex.competitive-selection.largest-first.v2:{}",
            include_str!(concat!(env!("OUT_DIR"), "/root-isolation-manifest.json"))
        ))
    }
    fn workspace_bytes(&self, programs: &[Arc<RootIsolationProgram>]) -> Result<usize, MathError> {
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
                let covering = (MAX_CELLS as usize + 1)
                    .checked_mul(width.checked_mul(32)?.checked_add(4096)?)?;
                native = native.max(graph.checked_add(covering)?);
            }
            transports
                .checked_add(native)?
                .checked_add(16 * 1024 * 1024)
        };
        checked().ok_or_else(|| MathError::Contract("IBEX workspace extent overflow".into()))
    }
    #[expect(
        unsafe_code,
        reason = "all native pointers borrow finite checked vectors; native ABI catches every C++ exception"
    )]
    fn certify(&self, request: &SelectionProofRequest<'_>) -> Result<SelectionEvidence, MathError> {
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
                max_cells: MAX_CELLS,
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
    }
    #[derive(Clone, Copy)]
    enum Score {
        Constant(f64),
        Unknown,
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
            Residual::Square | Residual::Singular => residual
                .binary(Binary::Mul, x.clone(), x.clone(), None, source())
                .unwrap(),
        };
        let equation = match shape {
            Residual::Singular => function,
            Residual::Square | Residual::Log => residual
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
