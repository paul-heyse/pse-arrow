// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical source contracts for the unit-inference driver (blueprint §7.4, §8.3).
use crate::relations::vec_sink::KernelBinding;
use crate::{NodeId, index::DomainFacts};
use pse_ids::SemanticId;
use pse_quantity::infer::{InvariantChecker, NoInvariantFacts};
use pse_quantity::{DomainId, QuantityKindId, QuantityTypeId, UnitId};
use std::collections::BTreeMap;

/// Group declaration joined to its actual member rows and valid tuple relation.
#[derive(Clone, Debug)]
pub struct GroupFacts {
    /// Complete indexed type of the group.
    pub quantity_type: QuantityTypeId,
    /// Ordered product factor domains.
    pub domains: Vec<DomainId>,
    /// The complete valid tuple set declared by the predecessor pass.
    pub valid_tuples: Vec<Vec<SemanticId>>,
    /// Actual member tuple to scalar symbol rows; validated against `valid_tuples`.
    pub members: BTreeMap<Vec<SemanticId>, SemanticId>,
}
/// One declared kernel input or output contract.
#[derive(Clone, Debug)]
pub struct KernelPort {
    /// Declared port name.
    pub name: String,
    /// Complete physical contract.
    pub quantity_type: QuantityTypeId,
    /// Actual representation unit accepted/emitted by the kernel port.
    pub unit: UnitId,
    /// Independently declared port shape (or parameter `indexed_by`), checked against its type.
    pub shape: Vec<pse_quantity::DomainKind>,
}
/// Kernel specification facts required by P10, joined from the pinned registry.
#[derive(Clone, Debug)]
pub struct KernelContract {
    /// Inputs in declared ordinal order.
    pub inputs: Vec<KernelPort>,
    /// Outputs in declared ordinal order.
    pub outputs: Vec<KernelPort>,
    /// Required parameter names with complete quantity and representation-unit contracts.
    pub parameters: Vec<KernelPort>,
}
/// Read-only actual predecessor facts. Missing facts remain errors; no default success,
/// hash comparison or membership in an invariant-ID set supplies semantic evidence.
pub trait SymbolTypeSource {
    /// Complete type declared for a scalar symbol.
    fn symbol_type(&self, symbol: SemanticId) -> Option<QuantityTypeId>;
    /// A symbol's representation unit when it differs from its canonical quantity unit.
    fn symbol_unit(&self, _symbol: SemanticId) -> Option<UnitId> {
        None
    }
    /// Registered Boolean kind identity used by compile-time guards.
    fn boolean_kind(&self) -> Option<QuantityKindId> {
        None
    }
    /// Actual group and member declarations.
    fn group(&self, _group: SemanticId) -> Option<&GroupFacts> {
        None
    }
    /// Actual domain declaration and ordered members.
    fn domain(&self, _domain: DomainId) -> Option<&DomainFacts> {
        None
    }
    /// Kernel specification addressed by the binding's kernel identity.
    fn kernel_contract(&self, _kernel: SemanticId) -> Option<&KernelContract> {
        None
    }
    /// Complete implicit unknown contract by actual system and unknown ordinal.
    fn implicit_unknown(&self, _system: SemanticId, _ordinal: u16) -> Option<QuantityTypeId> {
        None
    }
    /// Resolve the existing bound kernel for one explicit quantity-conversion occurrence.
    /// The driver checks its actual kernel, input node, full contracts and parameter facts.
    fn conversion_binding(
        &self,
        _node: NodeId,
        _operand: u16,
        _conversion: pse_quantity::ConversionId,
    ) -> Option<SemanticId> {
        None
    }
    /// Actual prerequisite checker; conservative when the source supplies no facts.
    fn invariant_checker(&self, _node: NodeId) -> &dyn InvariantChecker {
        &NoInvariantFacts
    }
}
/// Explicit source identities for the complete binding dependency graph.
pub type KernelBindings = BTreeMap<SemanticId, KernelBinding>;
