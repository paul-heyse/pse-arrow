// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded inference reuse over complete requests and actual immutable checker owners.
use super::{Exponent, Inferred, InvariantChecker, OpRequest, Operand, infer_with_evidence};
use crate::{
    BoundIndexRef, DomainKind, IndexSet, InvariantId, Opcode, QuantityError, QuantityRegistry,
    QuantityTypeId, Ratio, ReductionKind, UnitId, WeightNormalization,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Request {
    Literal(UnitId, crate::literal::LiteralContext),
    Add,
    Sub,
    Neg,
    Abs,
    Mul,
    Div,
    Pow(Option<Ratio>),
    Sqrt,
    Transcendental(Opcode),
    Affine(Vec<i8>, bool),
    Smooth(Opcode, u64),
    Gather(QuantityTypeId, Vec<BoundIndexRef>),
    WeightedMean(WeightNormalization, Option<InvariantId>),
    Reduce(ReductionKind, BoundIndexRef),
    Broadcast(BoundIndexRef),
    Conditional,
    UnitConvert(UnitId, UnitId, u64, u64),
    KernelCall(Vec<QuantityTypeId>, QuantityTypeId),
    ImplicitRef(QuantityTypeId),
    Derivative(UnitId, DomainKind, u8),
    Integral(UnitId, BoundIndexRef),
    PiecewiseLinear(QuantityTypeId, QuantityTypeId),
}
impl From<&OpRequest<'_>> for Request {
    fn from(request: &OpRequest<'_>) -> Self {
        match request {
            OpRequest::Literal { unit, context } => Self::Literal(*unit, *context),
            OpRequest::Add => Self::Add,
            OpRequest::Sub => Self::Sub,
            OpRequest::Neg => Self::Neg,
            OpRequest::Abs => Self::Abs,
            OpRequest::Mul => Self::Mul,
            OpRequest::Div => Self::Div,
            OpRequest::Pow { exponent } => Self::Pow(match exponent {
                Exponent::Rational(r) => Some(*r),
                Exponent::Symbolic => None,
            }),
            OpRequest::Sqrt => Self::Sqrt,
            OpRequest::Transcendental(op) => Self::Transcendental(*op),
            OpRequest::Affine {
                term_signs,
                has_constant,
            } => Self::Affine(term_signs.to_vec(), *has_constant),
            OpRequest::Smooth { opcode, eps } => Self::Smooth(*opcode, eps.to_bits()),
            OpRequest::Gather {
                group_type,
                coordinates,
            } => Self::Gather(*group_type, coordinates.to_vec()),
            OpRequest::WeightedMean {
                normalization,
                certified_invariant,
            } => Self::WeightedMean(*normalization, *certified_invariant),
            OpRequest::Reduce { kind, bound } => Self::Reduce(*kind, *bound),
            OpRequest::Broadcast { index } => Self::Broadcast(*index),
            OpRequest::Conditional => Self::Conditional,
            OpRequest::UnitConvert { spec } => Self::UnitConvert(
                spec.from,
                spec.to,
                spec.scale.to_bits(),
                spec.offset.to_bits(),
            ),
            OpRequest::KernelCall {
                declared_inputs,
                declared_output,
            } => Self::KernelCall(declared_inputs.to_vec(), *declared_output),
            OpRequest::ImplicitRef { unknown } => Self::ImplicitRef(*unknown),
            OpRequest::Derivative {
                domain_unit,
                domain_kind,
                order,
            } => Self::Derivative(*domain_unit, *domain_kind, *order),
            OpRequest::Integral { domain_unit, bound } => Self::Integral(*domain_unit, *bound),
            OpRequest::PiecewiseLinear { input, output } => Self::PiecewiseLinear(*input, *output),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    request: Request,
    operands: Vec<(QuantityTypeId, IndexSet)>,
    checker: usize,
}
/// An invocation's finite semantic index. Borrows retain actual registry/checker
/// owners; no pointer can outlive its referent or substitute a declaration digest.
/// Caller admission covers the request/result structures retained by this owner.
pub struct InferenceCache<'a> {
    registry: &'a QuantityRegistry,
    checkers: Vec<&'a dyn InvariantChecker>,
    entries: BTreeMap<Key, Inferred>,
    capacity: usize,
    hits: usize,
    misses: usize,
    resets: usize,
}
impl std::fmt::Debug for InferenceCache<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InferenceCache")
            .field("entries", &self.entries.len())
            .field("capacity", &self.capacity)
            .finish_non_exhaustive()
    }
}
impl<'a> InferenceCache<'a> {
    /// Bind one immutable registry and an explicit maximum number of successful entries.
    pub fn new(registry: &'a QuantityRegistry, capacity: usize) -> Self {
        Self {
            registry,
            capacity,
            checkers: Vec::new(),
            entries: BTreeMap::new(),
            hits: 0,
            misses: 0,
            resets: 0,
        }
    }
    /// Cumulative cache hits, uncached inference attempts and full-capacity resets.
    pub const fn counts(&self) -> (usize, usize, usize) {
        (self.hits, self.misses, self.resets)
    }
    /// Reuse successful inference only for an immutable actual checker and complete request.
    /// # Errors
    /// Every ordinary physical inference refusal remains in force; failures are not cached.
    pub fn infer(
        &mut self,
        request: &OpRequest<'_>,
        operands: &[Operand<'_>],
        checker: &'a dyn InvariantChecker,
    ) -> Result<Inferred, QuantityError> {
        if self.capacity == 0 || !checker.immutable() {
            self.misses += 1;
            return infer_with_evidence(request, operands, self.registry, checker);
        }
        let checker_index = self
            .checkers
            .iter()
            .position(|old| std::ptr::eq(*old, checker))
            .unwrap_or_else(|| {
                self.checkers.push(checker);
                self.checkers.len() - 1
            });
        let mut key = Key {
            request: request.into(),
            checker: checker_index,
            operands: operands
                .iter()
                .map(|arg| (arg.quantity_type, arg.indices.clone()))
                .collect(),
        };
        if let Some(value) = self.entries.get(&key) {
            self.hits += 1;
            return Ok(value.clone());
        }
        self.misses += 1;
        if self.entries.len() == self.capacity || self.checkers.len() > self.capacity {
            self.resets += 1;
            self.entries.clear();
            self.checkers.clear();
            self.checkers.push(checker);
            key.checker = 0;
        }
        let value = infer_with_evidence(request, operands, self.registry, checker)?;
        self.entries.insert(key, value.clone());
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::cell::Cell;
    fn fixture() -> (QuantityRegistry, QuantityTypeId) {
        let id = pse_ids::SemanticId::from_bytes([1; 16]);
        let unit = UnitId::from_id(id);
        let kind = QuantityKindId::from_id(id);
        let quantity = QuantityTypeId::from_id(id);
        let mut builder = QuantityRegistryBuilder::new();
        builder.unit(Unit {
            id: unit,
            symbol: "1".into(),
            dimension: DimensionVector::DIMENSIONLESS,
            scale_to_canonical: 1.0,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state: None,
        });
        builder.kind(QuantityKind {
            id: kind,
            dimension: DimensionVector::DIMENSIONLESS,
            extensive: false,
            addition_kind: QuantityAdditionKind::Additive,
        });
        builder.quantity_type(QuantityType {
            id: quantity,
            key: QuantityTypeKey {
                kind,
                basis: None,
                reference_state: None,
                scale_kind: ScaleKind::Point,
                shape: vec![],
                subject_kind: None,
            },
            canonical_unit: unit,
            nominal_magnitude: None,
        });
        builder.neutral_dimensionless(quantity);
        (builder.build().unwrap(), quantity)
    }
    struct Checker {
        calls: Cell<usize>,
        immutable: bool,
    }
    impl InvariantChecker for Checker {
        fn immutable(&self) -> bool {
            self.immutable
        }
        fn check(
            &self,
            _: InvariantId,
            _: &OpRequest<'_>,
            _: Option<&QuantityOperation>,
            _: &[Operand<'_>],
            _: &QuantityRegistry,
        ) -> Result<(), QuantityError> {
            self.calls.set(self.calls.get() + 1);
            Ok(())
        }
    }
    #[test]
    fn reuse_binds_actual_checker_and_complete_request_and_does_not_cache_refusals() {
        let (registry, quantity) = fixture();
        let indices = IndexSet::new();
        let args = [Operand {
            quantity_type: quantity,
            indices: &indices,
        }; 2];
        let request = OpRequest::WeightedMean {
            normalization: WeightNormalization::CertifiedUnitSum,
            certified_invariant: Some(InvariantId::from_id(pse_ids::SemanticId::NIL)),
        };
        let checker = Checker {
            calls: Cell::new(0),
            immutable: true,
        };
        let other = Checker {
            calls: Cell::new(0),
            immutable: true,
        };
        let mutable = Checker {
            calls: Cell::new(0),
            immutable: false,
        };
        let mut cache = InferenceCache::new(&registry, 1);
        for _ in 0..2 {
            cache.infer(&request, &args, &checker).unwrap();
        }
        assert_eq!(checker.calls.get(), 1);
        assert_eq!(cache.counts(), (1, 1, 0));
        cache.infer(&request, &args, &other).unwrap();
        assert_eq!(other.calls.get(), 1);
        assert_eq!(cache.counts(), (1, 2, 1));
        for _ in 0..2 {
            cache.infer(&request, &args, &mutable).unwrap();
        }
        assert_eq!(mutable.calls.get(), 2);
        assert!(cache.infer(&request, &args[..1], &checker).is_err());
        assert_eq!(
            registry.unit_by_symbol("1").unwrap().id,
            UnitId::from_id(quantity.as_id())
        );
        assert_eq!(
            registry
                .resolve_key(&registry.quantity_type(quantity).unwrap().key)
                .unwrap(),
            quantity
        );
    }
    #[test]
    fn neutral_supports_bounded_reset_characterization() {
        let (registry, quantity) = fixture();
        let indices = IndexSet::new();
        let operands = [Operand {
            quantity_type: quantity,
            indices: &indices,
        }];
        let checker = infer::NoInvariantFacts;
        let mut cache = InferenceCache::new(&registry, 1);
        assert_eq!(
            cache
                .infer(&OpRequest::Neg, &operands, &checker)
                .unwrap()
                .result,
            quantity
        );
        cache.infer(&OpRequest::Abs, &operands, &checker).unwrap();
        cache.infer(&OpRequest::Neg, &operands, &checker).unwrap();
        assert_eq!(cache.counts(), (0, 3, 2));
    }
}
