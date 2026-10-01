// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical admission products owned by expression checking, before numerical lowering.
use crate::{DeclarationId, Result, invalid};
use pse_authoring::dsl::{self, Expr};
use pse_quantity::{
    QuantityRegistry, ResolvedInference, ResolvedPhysicalContract,
    infer::{Exponent, InvariantChecker, OpRequest},
    scheme::{Scheme, Substitution},
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

/// Exact syntax occurrence within a single checked expression body.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpressionOccurrence {
    /// Root expression identity and deterministic preorder position in its authored AST.
    pub body: pse_ids::ContentHash,
    /// Distinguishes identical syntax in different lexical/binder positions.
    pub position: usize,
    /// Original byte range; specialized syntax can have an empty range.
    pub range: (u32, u32),
    /// Syntax distinguishes synthetic nodes whose byte ranges are empty.
    pub syntax: String,
}
impl ExpressionOccurrence {
    /// The one operation of a synthesized finite-reduction definition, even when empty.
    pub fn finite_reduction(owner: DeclarationId) -> Self {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::MathLocalOccurrenceV3);
        hash.id(&owner.as_id()).str("finite_reduction");
        Self {
            body: hash.finish_hash(),
            position: 0,
            range: (0, 0),
            syntax: "finite_reduction".into(),
        }
    }
    /// Identify an occurrence before any library normalization.
    pub fn of(expression: &Expr) -> Self {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::MathLocalOccurrenceV3);
        hash.str(&dsl::render_expr(expression));
        Self {
            body: hash.finish_hash(),
            position: 0,
            range: (expression.span.start, expression.span.end),
            syntax: dsl::render_expr(expression),
        }
    }
    /// Map temporary source addresses to stable body-relative positions. Addresses are
    /// only traversal aids and never enter a checked product, identity or cache key.
    pub fn in_body(body: &Expr) -> BTreeMap<usize, Self> {
        let root = Self::of(body).body;
        let mut positions = BTreeMap::new();
        let mut position = 0;
        body.walk(|expression| {
            let mut occurrence = Self::of(expression);
            occurrence.body = root;
            occurrence.position = position;
            position += 1;
            positions.insert(std::ptr::from_ref(expression) as usize, occurrence);
        });
        positions
    }
}

/// Checked operations retained by one declaration or specialized function body.
pub type ExpressionAdmissions = BTreeMap<ExpressionOccurrence, PhysicalAdmission>;
/// Scoped collection used only while constructing an immutable checked product.
#[derive(Debug, Default)]
pub struct AdmissionRecorder {
    products: RefCell<BTreeMap<DeclarationId, ExpressionAdmissions>>,
    depth: Cell<usize>,
    positions: RefCell<BTreeMap<usize, ExpressionOccurrence>>,
}
impl AdmissionRecorder {
    pub(crate) fn enter(&self, expression: &Expr) {
        if self.depth.get() == 0 {
            *self.positions.borrow_mut() = ExpressionOccurrence::in_body(expression);
        }
        self.depth.set(self.depth.get() + 1);
    }
    pub(crate) fn leave(&self) {
        self.depth.set(self.depth.get() - 1);
        if self.depth.get() == 0 {
            self.positions.borrow_mut().clear();
        }
    }
    pub(crate) fn record(
        &self,
        expression: &Expr,
        at: DeclarationId,
        admission: PhysicalAdmission,
    ) -> Result<()> {
        let occurrence = self
            .positions
            .borrow()
            .get(&(std::ptr::from_ref(expression) as usize))
            .cloned()
            .ok_or_else(|| {
                invalid(
                    at,
                    "checked operation is outside its expression occurrence tree",
                )
            })?;
        let mut products = self.products.borrow_mut();
        let entries = products.entry(at).or_default();
        if let Some(previous) = entries.get(&occurrence) {
            if previous != &admission {
                return Err(invalid(
                    at,
                    "one expression occurrence has conflicting physical contexts",
                ));
            }
        } else {
            entries.insert(occurrence, admission);
        }
        Ok(())
    }
    /// Finish constructing the immutable occurrence products.
    pub fn into_inner(self) -> BTreeMap<DeclarationId, ExpressionAdmissions> {
        self.products.into_inner()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PhysicalRequest {
    Add,
    Sub,
    Neg,
    Abs,
    Mul,
    Div,
    Pow(Option<pse_quantity::Ratio>),
    Sqrt,
    Transcendental(pse_quantity::Opcode),
    FiniteReduce(
        pse_quantity::ReductionKind,
        Option<pse_quantity::EntityKindId>,
    ),
    Reduce(pse_quantity::ReductionKind, pse_quantity::BoundIndexRef),
    Conditional,
}
impl PhysicalRequest {
    fn request(&self) -> OpRequest<'static> {
        match self {
            Self::Add => OpRequest::Add,
            Self::Sub => OpRequest::Sub,
            Self::Neg => OpRequest::Neg,
            Self::Abs => OpRequest::Abs,
            Self::Mul => OpRequest::Mul,
            Self::Div => OpRequest::Div,
            Self::Pow(exponent) => OpRequest::Pow {
                exponent: exponent.map_or(Exponent::Symbolic, Exponent::Rational),
            },
            Self::Sqrt => OpRequest::Sqrt,
            Self::Transcendental(opcode) => OpRequest::Transcendental(*opcode),
            Self::FiniteReduce(kind, domain) => OpRequest::FiniteReduce {
                kind: *kind,
                domain: *domain,
            },
            Self::Reduce(kind, bound) => OpRequest::Reduce {
                kind: *kind,
                bound: *bound,
            },
            Self::Conditional => OpRequest::Conditional,
        }
    }
}
/// An operation whose source operands may still contain quantified physical types.
#[derive(Clone, Debug)]
pub struct PhysicalAdmission {
    request: PhysicalRequest,
    operands: Vec<Scheme>,
    resolved: Option<ResolvedInference>,
}
impl PartialEq for PhysicalAdmission {
    fn eq(&self, other: &Self) -> bool {
        self.operands == other.operands
            && self.request == other.request
            && self.identity() == other.identity()
    }
}
impl PhysicalAdmission {
    /// Instantiate a checked scalar finite reduction's operand at its actual lexical
    /// binder. The checked reduction result and scales are retained; broadcasting only
    /// validates and installs the actual operand axis and binder before construction.
    #[expect(
        clippy::too_many_arguments,
        reason = "the checked reduction identity, actual binder and operand contract accompany the physical registry, invariant checker and declaration"
    )]
    pub fn instantiate_reduction_binder(
        &self,
        admission: &ResolvedInference,
        kind: pse_quantity::ReductionKind,
        bound: pse_quantity::BoundIndexRef,
        actual: &ResolvedPhysicalContract,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        at: DeclarationId,
    ) -> Result<ResolvedInference> {
        let PhysicalRequest::FiniteReduce(expected, domain) = &self.request else {
            return Err(invalid(
                at,
                "a bound reduction requires its checked finite-reduction obligation",
            ));
        };
        if *expected != kind || domain.is_some_and(|domain| domain != bound.kind) {
            return Err(invalid(
                at,
                "actual reduction binder differs from its checked kind or domain",
            ));
        }
        let [prototype] = admission.operands.as_slice() else {
            return Err(invalid(at, "checked reduction requires one prototype"));
        };
        let instantiated = pse_quantity::resolved::infer_in_context(
            &OpRequest::Broadcast { index: bound },
            std::slice::from_ref(prototype),
            None,
            registry,
            checker,
            None,
        )
        .map_err(|error| invalid(at, error.to_string()))?
        .result;
        if !instantiated.same_meaning(actual)
            || instantiated.representation() != actual.representation()
        {
            return Err(invalid(
                at,
                "actual reduction prototype differs from its retained admission",
            ));
        }
        let mut admission = admission.clone();
        admission.operands = vec![actual.clone()];
        Ok(admission)
    }
    /// Retain an owned operation request. Borrowing operations need their separate checked
    /// group, provider or continuous-domain product and cannot be smuggled into this one.
    pub fn checked(
        request: &OpRequest<'_>,
        operands: Vec<Scheme>,
        resolved: Option<ResolvedInference>,
        at: DeclarationId,
    ) -> Result<Self> {
        let request = match request {
            OpRequest::Add => PhysicalRequest::Add,
            OpRequest::Sub => PhysicalRequest::Sub,
            OpRequest::Neg => PhysicalRequest::Neg,
            OpRequest::Abs => PhysicalRequest::Abs,
            OpRequest::Mul => PhysicalRequest::Mul,
            OpRequest::Div => PhysicalRequest::Div,
            OpRequest::Pow { exponent } => PhysicalRequest::Pow(match exponent {
                Exponent::Rational(ratio) => Some(*ratio),
                Exponent::Symbolic => None,
            }),
            OpRequest::Sqrt => PhysicalRequest::Sqrt,
            OpRequest::Transcendental(opcode) => PhysicalRequest::Transcendental(*opcode),
            OpRequest::FiniteReduce { kind, domain } => {
                PhysicalRequest::FiniteReduce(*kind, *domain)
            }
            OpRequest::Reduce { kind, bound } => PhysicalRequest::Reduce(*kind, *bound),
            OpRequest::Conditional => PhysicalRequest::Conditional,
            _ => {
                return Err(invalid(
                    at,
                    "operation requires its separate checked payload product",
                ));
            }
        };
        Ok(Self {
            request,
            operands,
            resolved,
        })
    }
    /// The concrete result retained at checking, absent for an explicit generic obligation.
    pub fn resolved(&self) -> Option<&ResolvedInference> {
        self.resolved.as_ref()
    }
    /// Instantiate quantified schemes once, with the caller's admitted substitutions.
    pub fn specialize(
        &self,
        registry: &QuantityRegistry,
        substitutions: &Substitution,
        checker: &dyn InvariantChecker,
        formula: Option<&pse_quantity::PhysicalFormulaAuthority>,
        at: DeclarationId,
    ) -> Result<ResolvedInference> {
        if let Some(resolved) = &self.resolved {
            return Ok(resolved.clone());
        }
        let operands = self
            .operands
            .iter()
            .map(|scheme| {
                scheme
                    .resolve_contract_with_evidence(registry, substitutions, checker)
                    .map_err(|error| invalid(at, error.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        pse_quantity::resolved::infer_in_context(
            &self.request.request(),
            &operands,
            None,
            registry,
            checker,
            formula,
        )
        .map_err(|error| invalid(at, error.to_string()))
    }
    fn identity(&self) -> Option<pse_ids::ContentHash> {
        self.resolved.as_ref().map(|admission| {
            let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::MathResolvedAdmissionsV2);
            admission.frame(&mut hash);
            hash.finish_hash()
        })
    }
    /// Conservative retained storage for admission accounting.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.operands.capacity() * size_of::<Scheme>()
            + self
                .operands
                .iter()
                .map(crate::extent::scheme)
                .sum::<usize>()
            + self.resolved.as_ref().map_or(0, |inference| {
                inference.operands.capacity() * size_of::<ResolvedPhysicalContract>()
                    + inference
                        .operands
                        .iter()
                        .map(ResolvedPhysicalContract::heap_bytes)
                        .sum::<usize>()
                    + inference.result.heap_bytes()
                    + inference.conversions.capacity()
                        * size_of::<pse_quantity::infer::OperandConversion>()
                    + match &inference.selected {
                        pse_quantity::infer::OperationSelection::Registered {
                            operand_permutation,
                            ..
                        } => operand_permutation.capacity() * size_of::<u16>(),
                        _ => 0,
                    }
                    + inference.operand_scales.capacity() * size_of::<f64>()
            })
    }
}

/// Check a concrete operation at the expression boundary and hand its retained admission
/// to numerical construction. This is the same authority used for source schemes.
pub fn admit_operation(
    request: &OpRequest<'_>,
    operands: &[ResolvedPhysicalContract],
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
    formula: Option<&pse_quantity::PhysicalFormulaAuthority>,
    at: DeclarationId,
) -> Result<ResolvedInference> {
    pse_quantity::resolved::infer_in_context(request, operands, None, registry, checker, formula)
        .map_err(|error| invalid(at, error.to_string()))
}

/// Check the physical result of selected physical partial arguments before library differentiation.
pub fn admit_partial(
    value: &ResolvedPhysicalContract,
    arguments: &[ResolvedPhysicalContract],
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
    at: DeclarationId,
) -> Result<ResolvedInference> {
    pse_quantity::resolved::infer_partial(value, arguments, registry, checker)
        .map_err(|error| invalid(at, error.to_string()))
}
