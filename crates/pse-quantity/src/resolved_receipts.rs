// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Concrete operation receipts. Decoding this stream does not establish admission;
//! its caller must first qualify the immutable product and physical interpretation.
use super::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

crate::quantity_passive_record! {
    callback:physical_contract_facts_boundary;owner:resolved::receipts::ContractWire;
    name:"PhysicalContractFacts";version:1;direction:Output;
    /// Passive exact contract data; decoding never establishes physical admission.
    #[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
    pub struct ContractWire {
        named:Option<QuantityTypeId> =>1,Native;
        kinds:Vec<KindFactor> =>2,Native;
        qualified:Vec<QualifiedFactorWire> =>3,Native;
        axes:Vec<EntityKindId> =>4,Native;
        indices:Vec<crate::BoundIndexRef> =>5,Native;
        factors:Vec<UnitFactor> =>6,Native;
        dimension:DimensionVector=>7,Native;
        scale_bits:u64=>8,Binary64Bits;
        pure_number:bool=>9,Native;
        formula:Option<Box<FormulaWire>> =>10,Native;
    }
}
crate::quantity_passive_record! {
    callback:physical_qualified_factor_boundary;owner:resolved::receipts::QualifiedFactorWire;
    name:"PhysicalQualifiedFactor";version:1;direction:Output;
    /// Passive complete quantity key with its original exact exponent.
    #[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
    pub struct QualifiedFactorWire {
        key:QuantityTypeKey=>1,Native;
        exponent:Ratio=>2,Native;
    }
}
crate::quantity_passive_record! {
    callback:physical_formula_facts_boundary;owner:resolved::receipts::FormulaWire;
    name:"PhysicalFormulaFacts";version:1;direction:Output;
    /// Passive original formula attribution; restoring authority remains receipt-qualified.
    #[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
    pub struct FormulaWire {
        authority:pse_ids::SemanticId=>1,Native;
        operation:String=>2,Native;
        operands:Vec<ContractWire> =>3,Native;
        powers:Vec<Ratio> =>4,Native;
    }
}
impl From<&ResolvedPhysicalContract> for ContractWire {
    fn from(value: &ResolvedPhysicalContract) -> Self {
        Self {
            named: value.named,
            kinds: value.kinds.clone(),
            qualified: value
                .qualified
                .iter()
                .map(|v| QualifiedFactorWire {
                    key: v.key.clone(),
                    exponent: v.exponent,
                })
                .collect(),
            axes: value.axes.clone(),
            indices: value.indices.iter().copied().collect(),
            factors: value.representation.factors.clone(),
            dimension: value.representation.dimension,
            scale_bits: value.representation.scale_bits,
            pure_number: value.pure_number,
            formula: value.formula.as_ref().map(|f| {
                Box::new(FormulaWire {
                    authority: f.authority.identity(),
                    operation: f.operation.into(),
                    operands: f.operands.iter().map(Self::from).collect(),
                    powers: f.powers.clone(),
                })
            }),
        }
    }
}
impl ContractWire {
    fn heap_bytes(&self) -> usize {
        self.kinds.capacity() * size_of::<KindFactor>()
            + self.qualified.capacity() * size_of::<QualifiedFactorWire>()
            + self
                .qualified
                .iter()
                .map(|v| v.key.shape.capacity() * size_of::<EntityKindId>())
                .sum::<usize>()
            + self.axes.capacity() * size_of::<EntityKindId>()
            + self.indices.capacity() * size_of::<crate::BoundIndexRef>()
            + self.factors.capacity() * size_of::<UnitFactor>()
            + self.formula.as_ref().map_or(0, |v| {
                size_of::<FormulaWire>()
                    + v.operation.capacity()
                    + v.operands.capacity() * size_of::<ContractWire>()
                    + v.operands.iter().map(Self::heap_bytes).sum::<usize>()
                    + v.powers.capacity() * size_of::<Ratio>()
            })
    }
    fn contract(&self) -> Result<ResolvedPhysicalContract, QuantityError> {
        if !f64::from_bits(self.scale_bits).is_finite() || f64::from_bits(self.scale_bits) <= 0.0 {
            return Err(refusal(
                "physical.receipt",
                "invalid receipt representation scale",
            ));
        }
        let mut indices = IndexSet::new();
        for bound in &self.indices {
            indices
                .insert(*bound)
                .map_err(|_| refusal("physical.receipt", "conflicting receipt binder"))?;
        }
        if indices.len() != self.indices.len() {
            return Err(refusal("physical.receipt", "duplicate receipt binder"));
        }
        let formula = self
            .formula
            .as_ref()
            .map(|f| {
                let operation = match f.operation.as_str() {
                    "add" => "add",
                    "sub" => "sub",
                    "product" => "product",
                    _ => {
                        return Err(refusal(
                            "physical.receipt",
                            "unknown formula receipt operation",
                        ));
                    }
                };
                Ok(Box::new(FormulaEvidence {
                    authority: PhysicalFormulaAuthority::response(f.authority),
                    operation,
                    operands: f
                        .operands
                        .iter()
                        .map(Self::contract)
                        .collect::<Result<_, _>>()?,
                    powers: f.powers.clone(),
                }))
            })
            .transpose()?;
        Ok(ResolvedPhysicalContract {
            named: self.named,
            kinds: self.kinds.clone(),
            qualified: self
                .qualified
                .iter()
                .map(|v| QualifiedFactor {
                    key: v.key.clone(),
                    exponent: v.exponent,
                })
                .collect(),
            axes: self.axes.clone(),
            indices,
            representation: UnitRepresentation {
                factors: self.factors.clone(),
                dimension: self.dimension,
                scale_bits: self.scale_bits,
            },
            pure_number: self.pure_number,
            formula,
        })
    }
}
pse_contract::projection_struct! {
/// Passive complete operation outcome; original receipt replay admits its use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceWire {
    projection: { name: "PhysicalInferenceFacts", rust_path: crate::resolved::receipts::InferenceWire,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    operands: Vec<ContractWire> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    result: ContractWire => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    selected: OperationSelection => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    conversions: Vec<OperandConversion> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    operand_scales: Vec<u64> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    result_scale: u64 => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [], representation: pse_contract::boundary_projection::Scalar::F64Bits };
}
}
crate::quantity_passive_record! {
    callback:physical_contract_record_boundary;owner:resolved::receipts::ContractRecord;
    name:"PhysicalContractRecord";version:1;direction:Output;
    /// Opaque persisted complete contract. Decoding alone grants no admitted authority.
    #[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
    pub struct ContractRecord(ContractWire)=>value:1,Native;
}
impl ContractRecord {
    /// Complete owned heap extent of the serialized contract fields.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.0.heap_bytes()
    }
    /// Capture an already admitted contract without re-inferring it.
    pub fn capture(value: &ResolvedPhysicalContract) -> Self {
        Self(ContractWire::from(value))
    }
    /// Restore inside strict reconstruction of a caller-qualified immutable product.
    pub fn restore(&self) -> Result<ResolvedPhysicalContract, QuantityError> {
        require_record(self)?;
        self.0.contract()
    }
}
pse_contract::projection_struct! {
/// Opaque persisted ordinary boundary; decoding grants no admitted authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractBoundaryRecord {
    projection: { name: "PhysicalContractBoundaryRecord", rust_path: crate::resolved::receipts::ContractBoundaryRecord,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    source: ContractRecord => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    target: ContractRecord => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    scale_bits: u64 => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [], representation: pse_contract::boundary_projection::Scalar::F64Bits };
}
}
impl ContractBoundaryRecord {
    /// Capture the complete contracts and exact scale of an admitted boundary.
    pub fn capture(value: &AdmittedContractBoundary) -> Self {
        Self {
            source: ContractRecord::capture(&value.source),
            target: ContractRecord::capture(&value.target),
            scale_bits: value.scale_bits,
        }
    }
    /// Restore only this complete record inside a qualified immutable recipe session.
    pub fn restore(&self) -> Result<AdmittedContractBoundary, QuantityError> {
        require_record(self)?;
        AdmittedContractBoundary::from_qualified_record(
            self.source.restore()?,
            self.target.restore()?,
            self.scale_bits,
        )
    }
    /// Inline record plus the complete owned heap extents of its contracts.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.source.0.heap_bytes())
            .saturating_add(self.target.0.heap_bytes())
    }
}

pse_contract::projection_struct! {
/// Opaque persisted scientific output authorization, including its exact operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputBoundaryRecord {
    projection: { name: "PhysicalOutputBoundaryRecord", rust_path: crate::resolved::receipts::OutputBoundaryRecord,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    operation: pse_ids::SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    source: ContractRecord => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    target: ContractRecord => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    scale_bits: u64 => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [], representation: pse_contract::boundary_projection::Scalar::F64Bits };
}
}
impl OutputBoundaryRecord {
    /// Capture one already admitted output authorization without re-admission.
    pub fn capture(value: &AdmittedOutputBoundary) -> Self {
        Self {
            operation: value.operation,
            source: ContractRecord::capture(&value.source),
            target: ContractRecord::capture(&value.target),
            scale_bits: value.scale_bits,
        }
    }
    /// Restore only this complete record inside a qualified immutable recipe session.
    pub fn restore(&self) -> Result<AdmittedOutputBoundary, QuantityError> {
        require_record(self)?;
        AdmittedOutputBoundary::from_qualified_record(
            self.operation,
            self.source.restore()?,
            self.target.restore()?,
            self.scale_bits,
        )
    }
    /// Inline record plus the complete owned heap extents of both contracts.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.source.0.heap_bytes())
            .saturating_add(self.target.0.heap_bytes())
    }
}

// Reconstruction remains in the representation owner, where the actual witness
// fields are private. This is the common record namespace consumed by lowering.
pub use crate::unit::{CanonicalConversionRecord, RepresentationConversionRecord};

crate::quantity_passive_record! {
    callback:physical_inference_record_boundary;owner:resolved::receipts::InferenceRecord;
    name:"PhysicalInferenceRecord";version:1;direction:Both;
    /// Opaque persisted complete operation result, including selection and conversion scales.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct InferenceRecord(InferenceWire)=>value:1,Native;
}
impl InferenceRecord {
    /// Complete owned extent of the concrete operation fields.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .0
                .operands
                .iter()
                .map(ContractWire::heap_bytes)
                .sum::<usize>()
            + self.0.operands.capacity() * size_of::<ContractWire>()
            + self.0.result.heap_bytes()
            + self.0.conversions.capacity() * size_of::<OperandConversion>()
            + self.0.operand_scales.capacity() * size_of::<u64>()
            + match &self.0.selected {
                OperationSelection::Registered {
                    operand_permutation,
                    ..
                } => operand_permutation.capacity() * size_of::<u16>(),
                _ => 0,
            }
    }
    /// Capture a concrete operation already admitted by its owner.
    pub fn capture(value: &ResolvedInference) -> Self {
        Self(InferenceWire::from(value))
    }
    /// Restore inside strict reconstruction, never through inference.
    pub fn restore(&self) -> Result<ResolvedInference, QuantityError> {
        require_record(self)?;
        self.0.inference()
    }
}
crate::quantity_passive_record! {
    callback:physical_scheme_record_boundary;owner:resolved::receipts::SchemeRecord;
    name:"PhysicalSchemeRecord";version:1;direction:Output;
    /// Untrusted syntax of a physical scheme; retained resolved leaves are opaque contracts.
    #[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
    pub struct SchemeRecord(SchemeWire)=>value:1,Native;
}
crate::quantity_passive_enum! {
    callback:physical_scheme_shape_boundary;owner:resolved::receipts::SchemeWire;
    name:"PhysicalSchemeShape";version:1;direction:Output;
    /// Untrusted actual source scheme document, with opaque resolved leaves.
    #[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
    pub enum SchemeWire {
        /// A registered complete physical type identity retained as passive syntax.
        Concrete [quantity:QuantityTypeId=>1;]=>1;
        /// A captured complete physical contract whose restoration requires its qualified receipt.
        Resolved [contract:ContractRecord=>1;]=>2;
        /// A universally quantified physical type name.
        Variable [name:String=>1;]=>3;
        /// Difference type of the enclosed point or difference quantity scheme.
        Delta [value:Box<SchemeWire> =>1;]=>4;
        /// Product of two schemes, interpreted by registered physical operation rules.
        Product [left:Box<SchemeWire> =>1;right:Box<SchemeWire> =>2;]=>5;
        /// Quotient of two schemes, interpreted by registered physical operation rules.
        Quotient [left:Box<SchemeWire> =>1;right:Box<SchemeWire> =>2;]=>6;
        /// A scheme raised to its exact rational power.
        Power [value:Box<SchemeWire> =>1;power:Ratio=>2;]=>7;
    }
}
impl SchemeWire {
    fn heap_bytes(&self) -> usize {
        match self {
            Self::Resolved(v) => v.retained_bytes(),
            Self::Variable(v) => v.capacity(),
            Self::Delta(v) | Self::Power(v, _) => size_of::<Self>() + v.heap_bytes(),
            Self::Product(a, b) | Self::Quotient(a, b) => {
                2 * size_of::<Self>() + a.heap_bytes() + b.heap_bytes()
            }
            Self::Concrete(_) => 0,
        }
    }
    fn capture(value: &crate::scheme::Scheme) -> Self {
        use crate::scheme::Scheme as S;
        match value {
            S::Concrete(v) => Self::Concrete(*v),
            S::Resolved(v) => Self::Resolved(ContractRecord::capture(v)),
            S::Variable(v) => Self::Variable(v.clone()),
            S::Delta(v) => Self::Delta(Box::new(Self::capture(v))),
            S::Product(a, b) => {
                Self::Product(Box::new(Self::capture(a)), Box::new(Self::capture(b)))
            }
            S::Quotient(a, b) => {
                Self::Quotient(Box::new(Self::capture(a)), Box::new(Self::capture(b)))
            }
            S::Power(v, p) => Self::Power(Box::new(Self::capture(v)), *p),
        }
    }
    fn restore(&self) -> Result<crate::scheme::Scheme, QuantityError> {
        use crate::scheme::Scheme as S;
        Ok(match self {
            Self::Concrete(v) => S::Concrete(*v),
            Self::Resolved(v) => S::Resolved(Box::new(v.restore()?)),
            Self::Variable(v) => S::Variable(v.clone()),
            Self::Delta(v) => S::Delta(Box::new(v.restore()?)),
            Self::Product(a, b) => S::Product(Box::new(a.restore()?), Box::new(b.restore()?)),
            Self::Quotient(a, b) => S::Quotient(Box::new(a.restore()?), Box::new(b.restore()?)),
            Self::Power(v, p) => S::Power(Box::new(v.restore()?), *p),
        })
    }
}
impl SchemeRecord {
    /// Complete owned heap extent of the retained scheme.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.0.heap_bytes()
    }
    /// Capture the complete checked signature, including concrete anonymous contracts.
    pub fn capture(value: &crate::scheme::Scheme) -> Self {
        Self(SchemeWire::capture(value))
    }
    /// Restore in a qualified strict reconstruction session, never resolving the scheme.
    pub fn restore(&self) -> Result<crate::scheme::Scheme, QuantityError> {
        require_record(self)?;
        self.0.restore()
    }
}
/// Refuse restoration unless this complete owning record belongs to the qualified recipe.
pub fn require_record<T: Serialize>(record: &T) -> Result<(), QuantityError> {
    let admitted = SESSION
        .with(|s| match &*s.borrow() {
            Some(State::Replay { authority, .. }) => authority.admits(record),
            _ => Ok(false),
        })
        .map_err(QuantityError::RecordMembership)?;
    if admitted {
        Ok(())
    } else {
        Err(refusal(
            "physical.receipt",
            "admitted record is absent from qualified reconstruction recipe",
        ))
    }
}
impl From<&ResolvedInference> for InferenceWire {
    fn from(v: &ResolvedInference) -> Self {
        Self {
            operands: v.operands.iter().map(ContractWire::from).collect(),
            result: ContractWire::from(&v.result),
            selected: v.selected.clone(),
            conversions: v.conversions.clone(),
            operand_scales: v.operand_scales.iter().map(|v| v.to_bits()).collect(),
            result_scale: v.result_scale.to_bits(),
        }
    }
}
impl InferenceWire {
    fn inference(&self) -> Result<ResolvedInference, QuantityError> {
        let operand_scales = self
            .operand_scales
            .iter()
            .map(|v| f64::from_bits(*v))
            .collect::<Vec<_>>();
        let result_scale = f64::from_bits(self.result_scale);
        if operand_scales.iter().any(|v| !v.is_finite()) || !result_scale.is_finite() {
            return Err(refusal(
                "physical.receipt",
                "nonfinite receipt operation scale",
            ));
        }
        Ok(ResolvedInference {
            operands: self
                .operands
                .iter()
                .map(ContractWire::contract)
                .collect::<Result<_, _>>()?,
            result: self.result.contract()?,
            selected: self.selected.clone(),
            conversions: self.conversions.clone(),
            operand_scales,
            result_scale,
        })
    }
}
pse_contract::projection_struct! {
/// Passive exact inference request/outcome association; not a current scientific grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    projection: { name: "PhysicalInferenceReceipt", rust_path: crate::resolved::receipts::Receipt,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    request: pse_ids::ContentHash => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    result: InferenceWire => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Passive original operation and complete output contracts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryWire {
    projection: { name: "PhysicalOutputBoundaryFacts", rust_path: crate::resolved::receipts::BoundaryWire,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    operation: pse_ids::SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    source: ContractWire => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    target: ContractWire => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    scale_bits: u64 => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [], representation: pse_contract::boundary_projection::Scalar::F64Bits };
}
}
impl BoundaryWire {
    fn capture(v: &AdmittedOutputBoundary) -> Self {
        Self {
            operation: v.operation,
            source: ContractWire::from(&v.source),
            target: ContractWire::from(&v.target),
            scale_bits: v.scale_bits,
        }
    }
    fn restore(&self) -> Result<AdmittedOutputBoundary, QuantityError> {
        let scale = f64::from_bits(self.scale_bits);
        if !scale.is_finite() || scale <= 0.0 {
            return Err(refusal("physical.receipt", "invalid role boundary scale"));
        }
        Ok(AdmittedOutputBoundary {
            operation: self.operation,
            source: self.source.contract()?,
            target: self.target.contract()?,
            scale_bits: self.scale_bits,
        })
    }
}
pse_contract::projection_struct! {
/// Passive exact boundary request/outcome association.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryReceipt {
    projection: { name: "PhysicalOutputBoundaryReceipt", rust_path: crate::resolved::receipts::BoundaryReceipt,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    request: pse_ids::ContentHash => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    result: BoundaryWire => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Untrusted decoded operation data, with no public admitted-witness deserializer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptStream {
    projection: { name: "PhysicalReceiptStream", rust_path: crate::resolved::receipts::ReceiptStream,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    version: u32 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: 1 }] };
    entries: Vec<Receipt> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    boundaries: Vec<BoundaryReceipt> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl ReceiptStream {
    /// Owned extent of the concrete receipts and their complete physical contracts.
    pub fn heap_bytes(&self) -> usize {
        self.entries.capacity() * size_of::<Receipt>()
            + self
                .entries
                .iter()
                .map(|r| {
                    r.result.operands.capacity() * size_of::<ContractWire>()
                        + r.result
                            .operands
                            .iter()
                            .map(ContractWire::heap_bytes)
                            .sum::<usize>()
                        + r.result.result.heap_bytes()
                        + r.result.conversions.capacity() * size_of::<OperandConversion>()
                        + r.result.operand_scales.capacity() * size_of::<u64>()
                        + match &r.result.selected {
                            OperationSelection::Registered {
                                operand_permutation,
                                ..
                            } => operand_permutation.capacity() * size_of::<u16>(),
                            _ => 0,
                        }
                })
                .sum::<usize>()
            + self.boundaries.capacity() * size_of::<BoundaryReceipt>()
            + self
                .boundaries
                .iter()
                .map(|v| v.result.source.heap_bytes() + v.result.target.heap_bytes())
                .sum::<usize>()
    }
    /// Number of concrete operation and role receipts in this product.
    pub fn len(&self) -> usize {
        self.entries.len() + self.boundaries.len()
    }
    /// Whether the construction consumed no physical operation.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.boundaries.is_empty()
    }
}
enum State {
    Capture {
        depth: usize,
        entries: Vec<Receipt>,
        boundaries: Vec<BoundaryReceipt>,
    },
    Replay {
        entries: Vec<Receipt>,
        position: usize,
        boundaries: Vec<BoundaryReceipt>,
        boundary_position: usize,
        authority: pse_ids::scientific_replay::RecordAuthority,
    },
}
thread_local! { static SESSION: RefCell<Option<State>> = const {RefCell::new(None)}; }
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        SESSION.with(|s| *s.borrow_mut() = None);
    }
}
fn begin(state: State) -> Result<Reset, QuantityError> {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if s.is_some() {
            return Err(refusal(
                "physical.receipt",
                "nested admission receipt session",
            ));
        }
        *s = Some(state);
        Ok(Reset)
    })
}
/// Capture concrete physical operations during normal admission. Failed construction
/// never yields a reusable stream; nested inference remains one complete receipt.
pub fn capture<T, E: From<QuantityError>>(
    f: impl FnOnce() -> Result<T, E>,
) -> Result<(T, ReceiptStream), E> {
    let reset = begin(State::Capture {
        depth: 0,
        entries: Vec::new(),
        boundaries: Vec::new(),
    })?;
    let value = f()?;
    let (entries, boundaries) = SESSION.with(|s| match s.borrow_mut().take() {
        Some(State::Capture {
            entries,
            boundaries,
            ..
        }) => (entries, boundaries),
        _ => (Vec::new(), Vec::new()),
    });
    drop(reset);
    Ok((
        value,
        ReceiptStream {
            version: 1,
            entries,
            boundaries,
        },
    ))
}
/// Consume receipts from a caller-qualified immutable product. Move their original
/// backing into the replay session rather than making an overlapping deep copy.
/// Every request must match its next receipt; inference is never invoked.
pub fn replay<T, E: From<QuantityError>>(
    stream: ReceiptStream,
    authority: &pse_ids::scientific_replay::RecordAuthority,
    f: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    if !authority
        .admits(&stream)
        .map_err(QuantityError::RecordMembership)?
    {
        return Err(refusal(
            "physical.receipt",
            "physical receipts are absent from qualified recipe",
        )
        .into());
    }
    if stream.version != 1 {
        return Err(refusal("physical.receipt", "unsupported receipt interpretation").into());
    }
    let reset = begin(State::Replay {
        entries: stream.entries,
        position: 0,
        boundaries: stream.boundaries,
        boundary_position: 0,
        authority: authority.clone(),
    })?;
    let value = f()?;
    let complete=SESSION.with(|s|matches!(&*s.borrow(),Some(State::Replay {entries,position,boundaries,boundary_position,..}) if *position==entries.len()&&*boundary_position==boundaries.len()));
    drop(reset);
    if !complete {
        return Err(refusal("physical.receipt", "unconsumed physical receipts").into());
    }
    Ok(value)
}
fn request_key(
    request: &OpRequest<'_>,
    values: &[ResolvedPhysicalContract],
    expected: Option<QuantityTypeId>,
    authority: Option<&PhysicalFormulaAuthority>,
) -> pse_ids::ContentHash {
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
    h.str("concrete-request-v1");
    match request {
        OpRequest::Literal { unit, context } => {
            h.str("literal")
                .id(&unit.id.as_id())
                .str(&unit.symbol)
                .part(&unit.dimension.canonical_bytes())
                .u64(unit.scale_to_canonical.to_bits())
                .u64(unit.offset_to_canonical.to_bits())
                .bool(unit.is_affine);
            h.bool(unit.reference_state.is_some());
            if let Some(id) = unit.reference_state {
                h.id(&id.as_id());
            }
            h.bool(unit.definition.is_some());
            if let Some(factors) = &unit.definition {
                h.u64(factors.len() as u64);
                for f in factors {
                    h.id(&f.unit.as_id())
                        .part(&f.exponent.num().to_le_bytes())
                        .part(&f.exponent.den().to_le_bytes());
                }
            }
            match context {
                crate::literal::LiteralContext::Free => {
                    h.str("free");
                }
                crate::literal::LiteralContext::Additive { sibling } => {
                    h.str("additive").id(&sibling.as_id());
                }
                crate::literal::LiteralContext::EquationSide { side } => {
                    h.str("equation").id(&side.as_id());
                }
                crate::literal::LiteralContext::Explicit { quantity_type } => {
                    h.str("explicit").id(&quantity_type.as_id());
                }
            }
        }
        OpRequest::Add => {
            h.str("add");
        }
        OpRequest::Sub => {
            h.str("sub");
        }
        OpRequest::Neg => {
            h.str("neg");
        }
        OpRequest::Abs => {
            h.str("abs");
        }
        OpRequest::Mul => {
            h.str("mul");
        }
        OpRequest::Div => {
            h.str("div");
        }
        OpRequest::Sqrt => {
            h.str("sqrt");
        }
        OpRequest::Pow { exponent } => {
            h.str("pow");
            match exponent {
                Exponent::Rational(v) => {
                    h.str("rational")
                        .part(&v.num().to_le_bytes())
                        .part(&v.den().to_le_bytes());
                }
                Exponent::Symbolic => {
                    h.str("symbolic");
                }
            }
        }
        OpRequest::Transcendental(opcode) => {
            h.str("transcendental").str(opcode.as_str());
        }
        OpRequest::Affine {
            term_signs,
            has_constant,
        } => {
            h.str("affine")
                .bool(*has_constant)
                .u64(term_signs.len() as u64);
            for sign in *term_signs {
                h.part(&sign.to_le_bytes());
            }
        }
        OpRequest::Smooth { opcode, eps } => {
            h.str("smooth").str(opcode.as_str()).u64(eps.to_bits());
        }
        OpRequest::Gather {
            group_type,
            coordinates,
        } => {
            h.str("gather")
                .id(&group_type.as_id())
                .u64(coordinates.len() as u64);
            for v in *coordinates {
                frame_bound(&mut h, *v);
            }
        }
        OpRequest::WeightedMean {
            normalization,
            certified_invariant,
        } => {
            h.str("weighted_mean")
                .str(normalization.as_str())
                .bool(certified_invariant.is_some());
            if let Some(id) = certified_invariant {
                h.id(&id.as_id());
            }
        }
        OpRequest::Reduce { kind, bound } => {
            h.str("reduce").str(kind.as_str());
            frame_bound(&mut h, *bound);
        }
        OpRequest::FiniteReduce { kind, domain } => {
            h.str("finite_reduce")
                .str(kind.as_str())
                .bool(domain.is_some());
            if let Some(id) = domain {
                h.id(&id.as_id());
            }
        }
        OpRequest::Broadcast { index } => {
            h.str("broadcast");
            frame_bound(&mut h, *index);
        }
        OpRequest::Conditional => {
            h.str("conditional");
        }
        OpRequest::UnitConvert { spec } => {
            h.str("unit_convert")
                .id(&spec.from.as_id())
                .id(&spec.to.as_id())
                .u64(spec.scale.to_bits())
                .u64(spec.offset.to_bits());
        }
        OpRequest::KernelCall {
            declared_inputs,
            declared_output,
        } => {
            h.str("kernel").u64(declared_inputs.len() as u64);
            for id in *declared_inputs {
                h.id(&id.as_id());
            }
            h.id(&declared_output.as_id());
        }
        OpRequest::ImplicitRef { unknown } => {
            h.str("implicit").id(&unknown.as_id());
        }
        OpRequest::Derivative {
            domain_unit,
            domain_kind,
            order,
        } => {
            h.str("derivative")
                .id(&domain_unit.as_id())
                .id(&domain_kind.as_id())
                .u64(u64::from(*order));
        }
        OpRequest::Integral { domain_unit, bound } => {
            h.str("integral").id(&domain_unit.as_id());
            frame_bound(&mut h, *bound);
        }
        OpRequest::PiecewiseLinear { input, output } => {
            h.str("piecewise").id(&input.as_id()).id(&output.as_id());
        }
    }
    h.u64(values.len() as u64);
    for value in values {
        value.frame(&mut h);
    }
    h.bool(expected.is_some());
    if let Some(id) = expected {
        h.id(&id.as_id());
    }
    h.bool(authority.is_some());
    if let Some(v) = authority {
        h.id(&v.identity());
    }
    h.finish_hash()
}
fn frame_bound(h: &mut pse_ids::FramedHasher, v: crate::BoundIndexRef) {
    h.id(&v.bound_index.as_id())
        .id(&v.domain.as_id())
        .id(&v.kind.as_id());
}
pub(super) fn dispatch(
    request: &OpRequest<'_>,
    values: &[ResolvedPhysicalContract],
    expected: Option<QuantityTypeId>,
    authority: Option<&PhysicalFormulaAuthority>,
    calculate: impl FnOnce() -> Result<ResolvedInference, QuantityError>,
) -> Result<ResolvedInference, QuantityError> {
    if SESSION.with(|s| matches!(&*s.borrow(), Some(State::Replay { .. }))) {
        crate::premises::unsupported();
    }
    if !SESSION.with(|s| s.borrow().is_some()) {
        return calculate();
    }
    let key = request_key(request, values, expected, authority);
    let replay = SESSION.with(|s| {
        let mut state = s.borrow_mut();
        match state.as_mut() {
            Some(State::Replay {
                entries, position, ..
            }) => {
                let receipt = entries.get(*position).ok_or_else(|| {
                    refusal("physical.receipt", "missing concrete physical receipt")
                });
                Some(receipt.and_then(|v| {
                    if v.request != key {
                        return Err(refusal(
                            "physical.receipt",
                            "physical request differs from qualified receipt",
                        ));
                    }
                    *position += 1;
                    v.result.inference()
                }))
            }
            Some(State::Capture { depth, .. }) => {
                *depth += 1;
                None
            }
            None => None,
        }
    });
    if let Some(value) = replay {
        return value;
    }
    let result = calculate();
    SESSION.with(|s| {
        if let Some(State::Capture { depth, entries, .. }) = s.borrow_mut().as_mut() {
            *depth -= 1;
            if *depth == 0
                && let Ok(value) = &result
            {
                entries.push(Receipt {
                    request: key,
                    result: InferenceWire::from(value),
                });
            }
        }
    });
    result
}

struct ResumeCapture(bool);
impl Drop for ResumeCapture {
    fn drop(&mut self) {
        if self.0 {
            SESSION.with(|s| {
                if let Some(State::Capture { depth, .. }) = s.borrow_mut().as_mut() {
                    *depth -= 1;
                }
            });
        }
    }
}
/// Proof-only shadow construction has no executable physical operation stream.
/// Suppress nested operation recording while its owning proof receipt is captured.
pub fn without_capture<T>(f: impl FnOnce() -> T) -> T {
    let paused = SESSION.with(|s| {
        if let Some(State::Capture { depth, .. }) = s.borrow_mut().as_mut() {
            *depth += 1;
            true
        } else {
            false
        }
    });
    let reset = ResumeCapture(paused);
    let value = f();
    drop(reset);
    value
}
/// Capture/consume a declaration-owned physical role boundary. Its complete admitted
/// source, destination and scale are retained; reconstruction never re-admits the role.
pub fn boundary(
    request: pse_ids::ContentHash,
    calculate: impl FnOnce() -> Result<AdmittedOutputBoundary, QuantityError>,
) -> Result<AdmittedOutputBoundary, QuantityError> {
    if SESSION.with(|s| matches!(&*s.borrow(), Some(State::Replay { .. }))) {
        crate::premises::unsupported();
    }
    if !SESSION.with(|s| s.borrow().is_some()) {
        return calculate();
    }
    let replay = SESSION.with(|s| {
        let mut s = s.borrow_mut();
        match s.as_mut() {
            Some(State::Replay {
                boundaries,
                boundary_position,
                ..
            }) => Some(
                boundaries
                    .get(*boundary_position)
                    .ok_or_else(|| refusal("physical.receipt", "missing physical role receipt"))
                    .and_then(|v| {
                        if v.request != request {
                            return Err(refusal(
                                "physical.receipt",
                                "physical role request differs from qualified receipt",
                            ));
                        }
                        *boundary_position += 1;
                        v.result.restore()
                    }),
            ),
            Some(State::Capture { depth, .. }) => {
                *depth += 1;
                None
            }
            None => None,
        }
    });
    if let Some(v) = replay {
        return v;
    }
    let value = calculate();
    SESSION.with(|s| {
        if let Some(State::Capture {
            depth, boundaries, ..
        }) = s.borrow_mut().as_mut()
        {
            *depth -= 1;
            if *depth == 0
                && let Ok(v) = &value
            {
                boundaries.push(BoundaryReceipt {
                    request,
                    result: BoundaryWire::capture(v),
                });
            }
        }
    });
    value
}

/// Refuse accidental inference while a strict reconstruction session is active.
pub(super) fn require_admission_mode() -> Result<(), QuantityError> {
    if SESSION.with(|s| matches!(&*s.borrow(), Some(State::Replay { .. }))) {
        return Err(refusal(
            "physical.receipt",
            "physical inference is forbidden during reconstruction",
        ));
    }
    Ok(())
}

// Additive methods only. Neither original restore() changes its qualification
// requirement; nested Resolved still calls ContractRecord::restore() unchanged.
impl SchemeRecord {
    /// Adopt only passive scheme syntax; resolved leaves still require their original receipt.
    /// # Errors
    /// Every unqualified resolved record refuses through ContractRecord::restore.
    pub fn restore_passive(&self) -> Result<crate::scheme::Scheme, QuantityError> {
        self.0.restore()
    }
}

#[cfg(test)]
#[path = "resolved_receipts_projection_tests.rs"]
mod projection_tests;

#[cfg(test)]
mod replay_tests {
    use super::*;
    use pse_ids::scientific_replay::{
        QualifiedScientificRecipe, RecordAuthority, admitted_recipe_payload_hash,
    };

    fn journey(
        registry: &QuantityRegistry,
    ) -> Result<(Vec<ResolvedInference>, Vec<AdmittedOutputBoundary>), QuantityError> {
        let quantity = registry.neutral_dimensionless().unwrap();
        let contract = ResolvedPhysicalContract::named(quantity, IndexSet::new(), registry)?;
        let operations = [OpRequest::Add, OpRequest::Mul]
            .into_iter()
            .map(|operation| {
                infer_operation(
                    &operation,
                    &[contract.clone(), contract.clone()],
                    None,
                    registry,
                    &crate::infer::NoInvariantFacts,
                )
            })
            .collect::<Result<_, _>>()?;
        let boundaries = [47, 48]
            .into_iter()
            .map(|identity| {
                AdmittedOutputBoundary::declared(
                    pse_ids::SemanticId::from_bytes([identity; 16]),
                    contract.clone(),
                    quantity,
                    registry,
                )
            })
            .collect::<Result<_, _>>()?;
        Ok((operations, boundaries))
    }

    #[allow(
        unsafe_code,
        reason = "isolated owner-captured physical receipt fixture tests replay without issuing production authority"
    )]
    fn authority(stream: &ReceiptStream) -> RecordAuthority {
        let payload = serde_json::to_vec(stream).unwrap();
        let digest = admitted_recipe_payload_hash(&payload);
        // SAFETY: this private fixture controls the exact complete receipt stream,
        // including deliberate corruptions used only to test fail-closed replay.
        unsafe {
            QualifiedScientificRecipe::from_canonical_product(
                payload,
                digest,
                pse_ids::ContentHash::from_bytes([29; 32]),
                pse_ids::ContentHash::from_bytes([31; 32]),
            )
        }
        .unwrap()
        .records()
        .clone()
    }

    #[test]
    fn physical_receipt_replay_moves_original_backing_and_preserves_complete_outcomes() {
        let registry = crate::generated::standard_registry().unwrap();
        let (original, mut stream) = capture(|| journey(&registry)).unwrap();
        assert_eq!(stream.entries.len(), 2);
        assert_eq!(stream.boundaries.len(), 2);
        stream.entries.reserve(17);
        stream.boundaries.reserve(19);
        stream.entries[0].result.operands.reserve(23);
        let authority = authority(&stream);
        let backing = (
            stream.entries.as_ptr(),
            stream.boundaries.as_ptr(),
            stream.entries[0].result.operands.as_ptr(),
            stream.entries.capacity(),
            stream.boundaries.capacity(),
            stream.entries[0].result.operands.capacity(),
        );
        let restored = replay(stream, &authority, || {
            SESSION.with(|session| match &*session.borrow() {
                Some(State::Replay {
                    entries,
                    boundaries,
                    ..
                }) => {
                    assert_eq!(
                        (
                            entries.as_ptr(),
                            boundaries.as_ptr(),
                            entries[0].result.operands.as_ptr(),
                            entries.capacity(),
                            boundaries.capacity(),
                            entries[0].result.operands.capacity(),
                        ),
                        backing
                    );
                }
                _ => panic!("expected physical replay session"),
            });
            journey(&registry)
        })
        .unwrap();
        for (original, restored) in original.0.iter().zip(&restored.0) {
            let mut expected =
                pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
            let mut actual =
                pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
            original.frame(&mut expected);
            restored.frame(&mut actual);
            assert_eq!(actual.finish_hash(), expected.finish_hash());
        }
        assert_eq!(restored.1, original.1);
        assert!(SESSION.with(|session| session.borrow().is_none()));
        assert!(require_record(&InferenceRecord::capture(&restored.0[0])).is_err());
    }

    #[test]
    fn physical_receipt_replay_preserves_membership_version_and_cleanup_refusals() {
        let registry = crate::generated::standard_registry().unwrap();
        let (_, stream) = capture(|| journey(&registry)).unwrap();
        let original_authority = authority(&stream);
        // Independent corruptions deliberately retain the original fixture data.
        let mut changed = stream.clone();
        changed.entries[0].request = pse_ids::ContentHash::from_bytes([99; 32]);
        let membership: Result<(), QuantityError> = replay(changed, &original_authority, || {
            panic!("foreign receipt stream must refuse before restoration")
        });
        assert!(
            membership
                .unwrap_err()
                .to_string()
                .contains("physical receipts are absent")
        );

        let mut old = stream.clone();
        old.version = 2;
        let old_authority = authority(&old);
        let version: Result<(), QuantityError> = replay(old, &old_authority, || {
            panic!("unsupported receipt version must refuse before restoration")
        });
        assert!(
            version
                .unwrap_err()
                .to_string()
                .contains("unsupported receipt interpretation")
        );

        let failed: Result<(), QuantityError> = replay(stream, &original_authority, || {
            Err(refusal(
                "physical.receipt.test",
                "injected reconstruction refusal",
            ))
        });
        assert!(
            failed
                .unwrap_err()
                .to_string()
                .contains("injected reconstruction refusal")
        );
        assert!(SESSION.with(|session| session.borrow().is_none()));
        journey(&registry).unwrap();
    }
}
