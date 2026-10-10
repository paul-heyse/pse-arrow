// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical admission products owned by expression checking, before numerical lowering.
use crate::{DeclarationId, Result, invalid};
use pse_authoring::dsl::Expr;
use pse_quantity::{
    QuantityRegistry, ResolvedInference, ResolvedPhysicalContract,
    infer::{Exponent, InvariantChecker, OpRequest},
    scheme::{Scheme, Substitution},
};
use std::{cell::RefCell, collections::BTreeMap};

/// Compact coordinates under one retained field or function owner.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ExpressionOccurrence {
    /// Expression-root slot and node preorder position in the complete owner inventory.
    Node {
        /// Expression-root slot in the complete owning inventory.
        body: usize,
        /// Node preorder position within that root.
        position: usize,
    },
    /// The distinct synthesized operation of a finite-reduction function, even empty.
    FiniteReduction,
}
impl ExpressionOccurrence {
    /// Temporary address map for one standalone expression, whose root slot is zero.
    pub fn in_body(body: &Expr) -> BTreeMap<usize, Self> {
        super::inventory::positions([body])
    }
}
/// Checked operations retained by one field or specialized function inventory.
pub type ExpressionAdmissions = BTreeMap<ExpressionOccurrence, PhysicalAdmission>;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Owner {
    Field(super::occurrences::OccurrenceKey),
    Function(DeclarationId),
}
/// Scoped collection used only while constructing an immutable checked product.
#[derive(Debug)]
pub struct AdmissionRecorder {
    products: RefCell<BTreeMap<Owner, ExpressionAdmissions>>,
    positions: RefCell<BTreeMap<usize, Vec<(Owner, ExpressionOccurrence)>>>,
}
impl AdmissionRecorder {
    /// Prepare the complete immutable function inventory before checking any root.
    pub fn for_function(function: &crate::Function) -> Self {
        let recorder = Self {
            products: RefCell::default(),
            positions: RefCell::default(),
        };
        recorder.register(
            Owner::Function(function.id),
            super::inventory::function(function)
                .iter()
                .map(|root| root.expression),
        );
        recorder
    }
    pub(crate) fn for_package(package: &crate::CheckedPackage) -> Result<Self> {
        let recorder = Self {
            products: RefCell::default(),
            positions: RefCell::default(),
        };
        let fields = package
            .expressions
            .iter()
            .map(|(key, value)| {
                let roots = super::inventory::field(&value.syntax);
                recorder.register(Owner::Field(key.clone()), roots.iter().copied());
                (key.clone(), super::inventory::StructuralIndex::new(roots))
            })
            .collect::<BTreeMap<_, _>>();
        for function in package.functions.values() {
            let roots = super::inventory::function(function);
            recorder.register(
                Owner::Function(function.id),
                roots.iter().map(|root| root.expression),
            );
            let index = super::inventory::StructuralIndex::new(
                roots.iter().map(|root| root.expression).collect(),
            );
            for (slot, root) in roots.iter().enumerate() {
                let key = super::occurrences::OccurrenceKey {
                    declaration: function.id,
                    role: root.role.into(),
                    position: root.repeated,
                };
                let Some(field) = fields.get(&key) else {
                    continue;
                };
                // Exact source role/ordinal chooses the contextual owner. Full original
                // syntax equality verifies this mapping independently of prehashes.
                if !index.matches(slot, field, root.part) {
                    return Err(invalid(
                        function.id,
                        "function/source field inventory conflict",
                    ));
                }
                let owner = Owner::Field(key);
                let mut position = 0;
                root.expression.walk(|node| {
                    recorder
                        .positions
                        .borrow_mut()
                        .entry(std::ptr::from_ref(node) as usize)
                        .or_default()
                        .push((
                            owner.clone(),
                            ExpressionOccurrence::Node {
                                body: root.part,
                                position,
                            },
                        ));
                    position += 1;
                });
            }
        }
        Ok(recorder)
    }
    fn register<'a>(&self, owner: Owner, roots: impl IntoIterator<Item = &'a Expr>) {
        for (address, coordinate) in super::inventory::positions(roots) {
            self.positions
                .borrow_mut()
                .entry(address)
                .or_default()
                .push((owner.clone(), coordinate));
        }
    }
    pub(crate) fn record(
        &self,
        expression: &Expr,
        at: DeclarationId,
        admission: PhysicalAdmission,
    ) -> Result<()> {
        let coordinates = self
            .positions
            .borrow()
            .get(&(std::ptr::from_ref(expression) as usize))
            .cloned()
            .ok_or_else(|| {
                invalid(
                    at,
                    "checked operation is outside its retained owner inventory",
                )
            })?;
        let mut products = self.products.borrow_mut();
        for (owner, coordinate) in coordinates {
            let entries = products.entry(owner).or_default();
            if let Some(previous) = entries.get(&coordinate) {
                if previous != &admission {
                    return Err(invalid(
                        at,
                        "one expression occurrence has conflicting physical contexts",
                    ));
                }
            } else {
                entries.insert(coordinate, admission.clone());
            }
        }
        Ok(())
    }
    pub(crate) fn into_products(self) -> BTreeMap<Owner, ExpressionAdmissions> {
        self.products.into_inner()
    }
    /// Finish a standalone function's immutable products.
    pub fn into_inner(self) -> BTreeMap<DeclarationId, ExpressionAdmissions> {
        self.into_products()
            .into_iter()
            .filter_map(|(owner, products)| match owner {
                Owner::Function(id) => Some((id, products)),
                Owner::Field(_) => None,
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
/// Untrusted persisted obligation and its complete optional concrete admission.
/// This wire does not deserialize a `PhysicalAdmission` or admitted physical witness.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PhysicalAdmissionRecord {
    request: PhysicalRequest,
    operands: Vec<pse_quantity::resolved::receipts::SchemeRecord>,
    resolved: Option<pse_quantity::resolved::receipts::InferenceRecord>,
}
impl PhysicalAdmissionRecord {
    /// Complete owned extent of schemes and concrete operation result.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.operands.capacity() * size_of::<pse_quantity::resolved::receipts::SchemeRecord>()
            + self
                .operands
                .iter()
                .map(pse_quantity::resolved::receipts::SchemeRecord::retained_bytes)
                .sum::<usize>()
            + self.resolved.as_ref().map_or(
                0,
                pse_quantity::resolved::receipts::InferenceRecord::retained_bytes,
            )
    }
    /// Capture the owning checked operation without specializing or re-inferring it.
    pub fn capture(value: &PhysicalAdmission) -> Self {
        Self {
            request: value.request.clone(),
            operands: value
                .operands
                .iter()
                .map(pse_quantity::resolved::receipts::SchemeRecord::capture)
                .collect(),
            resolved: value
                .resolved
                .as_ref()
                .map(pse_quantity::resolved::receipts::InferenceRecord::capture),
        }
    }
    /// Restore only inside the enclosing qualified strict numerical reconstruction.
    pub fn restore(&self, at: DeclarationId) -> Result<PhysicalAdmission> {
        pse_quantity::resolved::receipts::require_record(self)
            .map_err(|e| invalid(at, e.to_string()))?;
        Ok(PhysicalAdmission {
            request: self.request.clone(),
            operands: self
                .operands
                .iter()
                .map(|v| v.restore().map_err(|e| invalid(at, e.to_string())))
                .collect::<Result<_>>()?,
            resolved: self
                .resolved
                .as_ref()
                .map(|v| v.restore().map_err(|e| invalid(at, e.to_string())))
                .transpose()?,
        })
    }
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
