// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Caller-qualified direct graph records. Selected functions are a content-indexed DAG,
//! not recursively expanded trees. No source AST or resolver participates in restoration.
use super::functions::{ArgumentLayout, FunctionEnvelope, FunctionExternal, SelectionMeaning};
use super::support::record::BodyInputRecord;
use super::*;
use crate::expression::admission::PhysicalAdmissionRecord;
use crate::portable::{NodeWire, ReadsWire, TypeRecord, ValueRecord};
use pse_quantity::resolved::receipts::{ContractRecord, OutputBoundaryRecord};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScalarRecord {
    id: SemanticId,
    ty: TypeRecord,
    role: ModelingDeclarationKind,
    domain: ModelingVariableDomain,
    definition: Option<NodeIx>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CarrierRecord {
    kind: CarrierKind,
    element: TypeRecord,
    members: Vec<ValueRecord>,
}
impl CarrierRecord {
    fn capture(value: &Carrier) -> Self {
        Self {
            kind: value.kind,
            element: TypeRecord::capture(&value.element),
            members: value.members.iter().map(ValueRecord::capture).collect(),
        }
    }
    fn restore(&self) -> Result<Carrier> {
        Ok::<_, crate::ModelingError>(Carrier {
            kind: self.kind,
            element: self.element.restore(SemanticId::NIL.into())?,
            members: self
                .members
                .iter()
                .map(|value| value.restore(SemanticId::NIL.into()))
                .collect::<Result<Vec<_>>>()?
                .into(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupRecord {
    axes: Vec<CarrierIx>,
    members: Vec<(Vec<ValueRecord>, Option<SymbolIx>)>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FoldRecord {
    root: NodeIx,
    carrier: CarrierIx,
    selected: Vec<ValueRecord>,
    prototype: TypeRecord,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceRecord {
    node: NodeWire,
    predicates: Vec<PredicateIx>,
    inputs: Vec<NodeIx>,
}
impl EvidenceRecord {
    fn capture(value: &EvidenceUse) -> Self {
        Self {
            node: NodeWire::capture(&value.node),
            predicates: value.predicates.clone(),
            inputs: value.inputs.clone(),
        }
    }
    fn restore(&self) -> EvidenceUse {
        EvidenceUse {
            node: self.node.restore(),
            predicates: self.predicates.clone(),
            inputs: self.inputs.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionSlot {
    identity: ContentHash,
    declaration: DeclarationId,
    formal_count: usize,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntaxRecord {
    identity: ContentHash,
    symbols: Vec<ScalarRecord>,
    nodes: Vec<Node>,
    predicates: Vec<ResolvedPredicate>,
    binders: Vec<Binding>,
    functions: Vec<FunctionSlot>,
    evidence: Vec<EvidenceRecord>,
    carriers: Vec<CarrierRecord>,
    groups: Vec<GroupRecord>,
    member_ranges: Vec<(SymbolIx, MemberRange)>,
    finite_folds: Vec<FoldRecord>,
    outputs: Vec<NodeIx>,
    validity: Vec<PredicateIx>,
}
impl SyntaxRecord {
    fn capture(value: &ResolvedSyntax) -> Result<Self> {
        Ok::<_, crate::ModelingError>(Self {
            identity: value.identity()?,
            symbols: value
                .symbols
                .slots()
                .iter()
                .map(|symbol| ScalarRecord {
                    id: symbol.id,
                    ty: TypeRecord::capture(&symbol.ty),
                    role: symbol.role,
                    domain: symbol.domain,
                    definition: symbol.definition,
                })
                .collect(),
            nodes: value.nodes.iter().cloned().collect(),
            predicates: value.predicates.iter().cloned().collect(),
            binders: value.binders.iter().cloned().collect(),
            functions: value
                .functions
                .iter()
                .map(|selected| FunctionSlot {
                    identity: selected.identity,
                    declaration: selected.declaration,
                    formal_count: selected.formals.len(),
                })
                .collect(),
            evidence: value.evidence.iter().map(EvidenceRecord::capture).collect(),
            carriers: value.carriers.iter().map(CarrierRecord::capture).collect(),
            groups: value
                .groups
                .iter()
                .map(|group| GroupRecord {
                    axes: group.axes.clone(),
                    members: group
                        .members
                        .iter()
                        .map(|(tuple, member)| {
                            (tuple.iter().map(ValueRecord::capture).collect(), *member)
                        })
                        .collect(),
                })
                .collect(),
            member_ranges: value
                .member_ranges
                .iter()
                .map(|(symbol, range)| (*symbol, range.clone()))
                .collect(),
            finite_folds: value
                .finite_folds
                .iter()
                .map(|(root, fold)| FoldRecord {
                    root: *root,
                    carrier: fold.carrier,
                    selected: fold.selected.iter().map(ValueRecord::capture).collect(),
                    prototype: TypeRecord::capture(&fold.prototype),
                })
                .collect(),
            outputs: value.outputs.clone(),
            validity: value.validity.clone(),
        })
    }
    fn entries(&self) -> usize {
        self.symbols
            .len()
            .saturating_add(self.nodes.len())
            .saturating_add(self.predicates.len())
            .saturating_add(self.binders.len())
            .saturating_add(self.functions.len())
            .saturating_add(self.evidence.len())
            .saturating_add(self.carriers.len())
            .saturating_add(self.groups.len())
            .saturating_add(self.member_ranges.len())
            .saturating_add(self.finite_folds.len())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgumentRecord {
    ty: TypeRecord,
    start: usize,
    len: usize,
    tuples: Vec<Vec<ValueRecord>>,
    domains: Vec<(usize, Vec<ValueRecord>, CarrierRecord)>,
    static_value: Option<ValueRecord>,
}
impl ArgumentRecord {
    fn capture(value: &ArgumentLayout) -> Self {
        Self {
            ty: TypeRecord::capture(&value.ty),
            start: value.start,
            len: value.len,
            tuples: value
                .tuples
                .iter()
                .map(|tuple| tuple.iter().map(ValueRecord::capture).collect())
                .collect(),
            domains: value
                .domains
                .iter()
                .map(|(position, prefix, carrier)| {
                    (
                        *position,
                        prefix.iter().map(ValueRecord::capture).collect(),
                        CarrierRecord::capture(carrier),
                    )
                })
                .collect(),
            static_value: value.static_value.as_ref().map(ValueRecord::capture),
        }
    }
    fn restore(&self, at: DeclarationId) -> Result<ArgumentLayout> {
        Ok::<_, crate::ModelingError>(ArgumentLayout {
            ty: self.ty.restore(at)?,
            start: self.start,
            len: self.len,
            tuples: self
                .tuples
                .iter()
                .map(|tuple| tuple.iter().map(|value| value.restore(at)).collect())
                .collect::<Result<_>>()?,
            domains: self
                .domains
                .iter()
                .map(|(position, prefix, carrier)| {
                    Ok::<_, crate::ModelingError>((
                        *position,
                        prefix
                            .iter()
                            .map(|value| value.restore(at))
                            .collect::<Result<_>>()?,
                        carrier.restore()?,
                    ))
                })
                .collect::<Result<_>>()?,
            static_value: self
                .static_value
                .as_ref()
                .map(|value| value.restore(at))
                .transpose()?,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectedRecord {
    kind: DeclarationId,
    origin: DeclarationId,
    values: Vec<(String, ValueRecord)>,
    uncertainties: Vec<(
        String,
        pse_model::generated::enums::ModelingUncertaintyKind,
        u64,
    )>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionRecord {
    dependencies: DeclarationId,
    roots: Vec<ValueRecord>,
    context: ValueRecord,
    records: Vec<ValueRecord>,
    edges: Vec<(ValueRecord, Vec<ValueRecord>)>,
    selected: Vec<SelectedRecord>,
}
impl SelectionRecord {
    fn capture(value: &SelectionMeaning) -> Self {
        Self {
            dependencies: value.dependencies,
            roots: value
                .closure
                .roots
                .iter()
                .map(ValueRecord::capture)
                .collect(),
            context: ValueRecord::capture(&value.closure.context),
            records: value
                .closure
                .records
                .iter()
                .map(ValueRecord::capture)
                .collect(),
            edges: value
                .closure
                .edges
                .iter()
                .map(|(source, targets)| {
                    (
                        ValueRecord::capture(source),
                        targets.iter().map(ValueRecord::capture).collect(),
                    )
                })
                .collect(),
            selected: value
                .records
                .iter()
                .map(|record| SelectedRecord {
                    kind: record.kind,
                    origin: record.origin,
                    values: record
                        .values
                        .iter()
                        .map(|(key, value)| (key.clone(), ValueRecord::capture(value)))
                        .collect(),
                    uncertainties: record
                        .uncertainties
                        .iter()
                        .map(|(key, value)| (key.clone(), value.kind, value.magnitude.to_bits()))
                        .collect(),
                })
                .collect(),
        }
    }
    fn restore(&self, at: DeclarationId) -> Result<SelectionMeaning> {
        let selected = self
            .selected
            .iter()
            .map(|record| {
                let values = record
                    .values
                    .iter()
                    .map(|(key, value)| {
                        Ok::<_, crate::ModelingError>((key.clone(), value.restore(at)?))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?;
                let uncertainties = record
                    .uncertainties
                    .iter()
                    .map(|(key, kind, bits)| {
                        let magnitude = f64::from_bits(*bits);
                        if !magnitude.is_finite() || magnitude < 0.0 {
                            return Err(invalid(at, "invalid strict scientific uncertainty"));
                        }
                        Ok::<_, crate::ModelingError>((
                            key.clone(),
                            crate::entity::Uncertainty {
                                kind: *kind,
                                magnitude,
                            },
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?;
                if values.len() != record.values.len()
                    || uncertainties.len() != record.uncertainties.len()
                {
                    return Err(invalid(at, "duplicate strict scientific record field"));
                }
                Ok::<_, crate::ModelingError>(crate::entity::Record {
                    kind: record.kind,
                    origin: record.origin,
                    values,
                    uncertainties,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let edges = self
            .edges
            .iter()
            .map(|(source, targets)| {
                Ok::<_, crate::ModelingError>((
                    source.restore(at)?,
                    targets
                        .iter()
                        .map(|value| value.restore(at))
                        .collect::<Result<BTreeSet<_>>>()?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        if edges.len() != self.edges.len() {
            return Err(invalid(
                at,
                "duplicate strict scientific selection edge owner",
            ));
        }
        Ok::<_, crate::ModelingError>(SelectionMeaning {
            dependencies: self.dependencies,
            closure: crate::scientific_selection::SelectionClosure {
                roots: self
                    .roots
                    .iter()
                    .map(|value| value.restore(at))
                    .collect::<Result<_>>()?,
                context: self.context.restore(at)?,
                records: self
                    .records
                    .iter()
                    .map(|value| value.restore(at))
                    .collect::<Result<_>>()?,
                edges,
            },
            records: selected,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeRecord {
    owner: DeclarationId,
    ty: TypeRecord,
    predicate: PredicateIx,
    reads: ReadsWire,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReductionRecord {
    kind: pse_quantity::ReductionKind,
    domain: Option<pse_quantity::EntityKindId>,
    prototype: ContractRecord,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReductionSelectionRecord {
    carrier: CarrierRecord,
    selected: Vec<ValueRecord>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShapeRecord {
    argument: SemanticId,
    axes: Vec<SemanticId>,
    coordinates: Vec<Vec<SemanticId>>,
    start: usize,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalRecord {
    implementation: String,
    revision: ContentHash,
    data: ContentHash,
    output: usize,
    derivative_source: crate::external::DerivativeSource,
    derivatives: u8,
    smoothness: u8,
    shapes: Vec<ShapeRecord>,
}
impl ExternalRecord {
    fn capture(value: &FunctionExternal) -> Self {
        Self {
            implementation: value.implementation.clone(),
            revision: value.revision,
            data: value.data,
            output: value.output,
            derivative_source: value.derivative_source,
            derivatives: value.derivatives,
            smoothness: value.smoothness,
            shapes: value
                .shapes
                .iter()
                .map(|shape| ShapeRecord {
                    argument: shape.argument,
                    axes: shape.axes.clone(),
                    coordinates: shape.coordinates.clone(),
                    start: shape.start,
                })
                .collect(),
        }
    }
    fn restore(&self) -> Result<FunctionExternal> {
        if self.implementation.is_empty() || self.derivatives > 2 || self.smoothness > 2 {
            return Err(invalid(
                SemanticId::NIL,
                "invalid strict selected external contract",
            ));
        }
        Ok::<_, crate::ModelingError>(FunctionExternal {
            implementation: self.implementation.clone(),
            revision: self.revision,
            data: self.data,
            output: self.output,
            derivative_source: self.derivative_source,
            derivatives: self.derivatives,
            smoothness: self.smoothness,
            shapes: self
                .shapes
                .iter()
                .map(|shape| crate::external::ArgumentShape {
                    argument: shape.argument,
                    axes: shape.axes.clone(),
                    coordinates: shape.coordinates.clone(),
                    start: shape.start,
                })
                .collect(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionRecord {
    identity: ContentHash,
    declaration: DeclarationId,
    syntax: SyntaxRecord,
    physical: SupportRecord,
    formals: Vec<SymbolIx>,
    signature: Vec<TypeRecord>,
    arguments: Vec<ArgumentRecord>,
    selections: Vec<SelectionRecord>,
    literals: Vec<(NodeIx, TypeRecord)>,
    result: TypeRecord,
    body: Option<NodeIx>,
    validity: Option<PredicateIx>,
    envelopes: Vec<EnvelopeRecord>,
    applicability: Vec<EvidenceRecord>,
    prerequisites: Vec<usize>,
    admissions: Vec<(NodeIx, PhysicalAdmissionRecord)>,
    reduction_admission: Option<PhysicalAdmissionRecord>,
    reduction: Option<ReductionRecord>,
    reduction_selection: Option<ReductionSelectionRecord>,
    physical_operation: Option<operation::OperationRecord>,
    output_boundary: Option<OutputBoundaryRecord>,
    body_inputs: Vec<Option<BodyInputRecord>>,
    validity_reads: ReadsWire,
    external: Option<ExternalRecord>,
    continuity: Option<u8>,
}
impl FunctionRecord {
    fn capture(value: &FunctionProduct) -> Result<Self> {
        Ok::<_, crate::ModelingError>(Self {
            identity: value.identity,
            declaration: value.declaration,
            syntax: SyntaxRecord::capture(&value.syntax)?,
            physical: SupportRecord::capture(&value.physical)?,
            formals: value.formals.clone(),
            signature: value.signature.iter().map(TypeRecord::capture).collect(),
            arguments: value
                .arguments
                .iter()
                .map(ArgumentRecord::capture)
                .collect(),
            selections: value
                .selections
                .iter()
                .map(SelectionRecord::capture)
                .collect(),
            literals: value
                .literals
                .iter()
                .map(|(slot, ty)| (*slot, TypeRecord::capture(ty)))
                .collect(),
            result: TypeRecord::capture(&value.result),
            body: value.body,
            validity: value.validity,
            envelopes: value
                .envelopes
                .iter()
                .map(|guard| EnvelopeRecord {
                    owner: guard.owner,
                    ty: TypeRecord::capture(&guard.ty),
                    predicate: guard.predicate,
                    reads: ReadsWire::from(&guard.reads),
                })
                .collect(),
            applicability: value
                .applicability
                .iter()
                .map(EvidenceRecord::capture)
                .collect(),
            prerequisites: value.prerequisites.clone(),
            admissions: value
                .admissions
                .iter()
                .map(|(slot, admission)| (*slot, PhysicalAdmissionRecord::capture(admission)))
                .collect(),
            reduction_admission: value
                .reduction_admission
                .as_ref()
                .map(PhysicalAdmissionRecord::capture),
            reduction: value.reduction.as_ref().map(|reduction| ReductionRecord {
                kind: reduction.kind,
                domain: reduction.domain,
                prototype: ContractRecord::capture(&reduction.prototype),
            }),
            reduction_selection: value.reduction_selection.as_ref().map(|selection| {
                ReductionSelectionRecord {
                    carrier: CarrierRecord::capture(&selection.carrier),
                    selected: selection
                        .selected
                        .iter()
                        .map(ValueRecord::capture)
                        .collect(),
                }
            }),
            physical_operation: value
                .physical_operation
                .as_ref()
                .map(operation::OperationRecord::capture),
            output_boundary: value
                .output_boundary
                .as_ref()
                .map(OutputBoundaryRecord::capture),
            body_inputs: value
                .body_inputs
                .iter()
                .map(|view| view.as_ref().map(BodyInputRecord::capture))
                .collect(),
            validity_reads: ReadsWire::from(&value.validity_reads),
            external: value.external.as_ref().map(ExternalRecord::capture),
            continuity: value.continuity,
        })
    }
}
/// Complete selected direct graph and a bounded, content-indexed function pool.
/// Deserialization grants no admission and never reconstructs live physical read captures.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedRecord {
    syntax: SyntaxRecord,
    functions: Vec<FunctionRecord>,
}
impl ResolvedRecord {
    /// Capture only this immutable selected graph and each complete function product once.
    pub fn capture(syntax: &ResolvedSyntax, limit: usize) -> Result<Self> {
        fn collect(
            syntax: &ResolvedSyntax,
            functions: &mut BTreeMap<ContentHash, Arc<FunctionProduct>>,
            active: &mut BTreeSet<ContentHash>,
            depth: usize,
            limit: usize,
        ) -> Result<()> {
            if depth >= 128 {
                return Err(invalid(SemanticId::NIL, "strict function capture depth"));
            }
            for selected in &syntax.functions {
                if functions.contains_key(&selected.identity) {
                    continue;
                }
                if !active.insert(selected.identity) {
                    return Err(invalid(
                        selected.declaration,
                        "strict function capture cycle",
                    ));
                }
                if functions.len().saturating_add(active.len()) > limit {
                    return Err(invalid(
                        selected.declaration,
                        "strict selected function capture budget",
                    ));
                }
                collect(
                    &selected.product.syntax,
                    functions,
                    active,
                    depth + 1,
                    limit,
                )?;
                active.remove(&selected.identity);
                functions.insert(selected.identity, selected.product.clone());
            }
            Ok::<_, crate::ModelingError>(())
        }
        let mut functions = BTreeMap::new();
        collect(syntax, &mut functions, &mut BTreeSet::new(), 0, limit)?;
        let record = Self {
            syntax: SyntaxRecord::capture(syntax)?,
            functions: functions
                .values()
                .map(|value| FunctionRecord::capture(value))
                .collect::<Result<_>>()?,
        };
        if record
            .functions
            .iter()
            .fold(record.syntax.entries(), |count, function| {
                count.saturating_add(function.syntax.entries())
            })
            > limit
        {
            return Err(invalid(
                SemanticId::NIL,
                "strict complete direct graph capture budget",
            ));
        }
        Ok::<_, crate::ModelingError>(record)
    }
    /// Join every exact record inside the enclosing immutable recipe's qualified session.
    /// No AST, text parser, type inference, provider inventory or mutable context is consulted.
    pub fn restore(&self, limit: usize) -> Result<ResolvedSyntax> {
        pse_quantity::resolved::receipts::require_record(self)
            .map_err(|error| invalid(SemanticId::NIL, error.to_string()))?;
        let pool = self
            .functions
            .iter()
            .map(|function| (function.identity, function))
            .collect::<BTreeMap<_, _>>();
        if pool.len() != self.functions.len() {
            return Err(invalid(
                SemanticId::NIL,
                "duplicate strict selected function identity",
            ));
        }
        let mut rebuild = Rebuild {
            pool,
            functions: BTreeMap::new(),
            active: BTreeSet::new(),
            remaining: limit,
            depth: 0,
        };
        let syntax = rebuild.syntax(&self.syntax)?;
        if rebuild.functions.len() != self.functions.len() {
            return Err(invalid(
                SemanticId::NIL,
                "strict graph contains unrelated function records",
            ));
        }
        Ok::<_, crate::ModelingError>(syntax)
    }
}
struct Rebuild<'a> {
    pool: BTreeMap<ContentHash, &'a FunctionRecord>,
    functions: BTreeMap<ContentHash, Arc<FunctionProduct>>,
    active: BTreeSet<ContentHash>,
    remaining: usize,
    depth: usize,
}
impl Rebuild<'_> {
    fn syntax(&mut self, record: &SyntaxRecord) -> Result<ResolvedSyntax> {
        self.remaining = self
            .remaining
            .checked_sub(record.entries())
            .ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "strict complete direct graph restoration budget",
                )
            })?;
        let mut symbols = crate::specialize::Symbols::default();
        for scalar in &record.symbols {
            if scalar.id == SemanticId::NIL || symbols.contains_key(&scalar.id) {
                return Err(invalid(
                    scalar.id,
                    "invalid or repeated strict scalar identity",
                ));
            }
            symbols.push(crate::specialize::Symbol {
                id: scalar.id,
                ty: scalar.ty.restore(SemanticId::NIL.into())?,
                role: scalar.role,
                domain: scalar.domain,
                definition: scalar.definition,
                expression: None,
                initial: None,
                lineage: crate::specialize::Lineage {
                    declaration: scalar.id.into(),
                    instance: SemanticId::NIL.into(),
                    path: String::new(),
                    demand: vec![],
                    default_owner: None,
                    is_override: false,
                    presets: vec![],
                },
            });
        }
        let functions = record
            .functions
            .iter()
            .map(|slot| {
                Ok::<_, crate::ModelingError>(FunctionSelection {
                    identity: slot.identity,
                    declaration: slot.declaration,
                    formals: vec![String::new(); slot.formal_count],
                    product: self.function(slot.identity)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let member_ranges = record
            .member_ranges
            .iter()
            .cloned()
            .collect::<BTreeMap<_, _>>();
        if member_ranges.len() != record.member_ranges.len() {
            return Err(invalid(SemanticId::NIL, "duplicate strict scalar range"));
        }
        let finite_folds = record
            .finite_folds
            .iter()
            .map(|fold| {
                Ok::<_, crate::ModelingError>((
                    fold.root,
                    FiniteFold {
                        carrier: fold.carrier,
                        selected: fold
                            .selected
                            .iter()
                            .map(|value| value.restore(SemanticId::NIL.into()))
                            .collect::<Result<Vec<_>>>()?
                            .into(),
                        prototype: fold.prototype.restore(SemanticId::NIL.into())?,
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        if finite_folds.len() != record.finite_folds.len() {
            return Err(invalid(SemanticId::NIL, "duplicate strict finite fold"));
        }
        let arena = Arena {
            nodes: record.nodes.clone().into(),
            predicates: record.predicates.clone().into(),
            binders: record.binders.clone().into(),
            functions: functions.into(),
            evidence: record
                .evidence
                .iter()
                .map(EvidenceRecord::restore)
                .collect::<Vec<_>>()
                .into(),
            carriers: record
                .carriers
                .iter()
                .map(CarrierRecord::restore)
                .collect::<Result<Vec<_>>>()?
                .into(),
            groups: record
                .groups
                .iter()
                .map(|group| {
                    Ok::<_, crate::ModelingError>(IndexedGroup {
                        axes: group.axes.clone(),
                        members: group
                            .members
                            .iter()
                            .map(|(tuple, slot)| {
                                Ok::<_, crate::ModelingError>((
                                    tuple
                                        .iter()
                                        .map(|value| value.restore(SemanticId::NIL.into()))
                                        .collect::<Result<_>>()?,
                                    *slot,
                                ))
                            })
                            .collect::<Result<_>>()?,
                    })
                })
                .collect::<Result<Vec<_>>>()?
                .into(),
            member_ranges,
            finite_folds,
            node_sources: BTreeMap::new(),
            predicate_sources: BTreeMap::new(),
        };
        let syntax = ResolvedSyntax {
            symbols,
            arena: Arc::new(arena),
            outputs: record.outputs.clone(),
            validity: record.validity.clone(),
        };
        if syntax.identity()? != record.identity {
            return Err(invalid(
                SemanticId::NIL,
                "restored exact typed syntax identity differs",
            ));
        }
        Ok::<_, crate::ModelingError>(syntax)
    }
    fn function(&mut self, identity: ContentHash) -> Result<Arc<FunctionProduct>> {
        if let Some(value) = self.functions.get(&identity) {
            return Ok::<_, crate::ModelingError>(value.clone());
        }
        if self.depth >= 128 || !self.active.insert(identity) {
            return Err(invalid(
                SemanticId::NIL,
                "strict selected function cycle or depth",
            ));
        }
        self.depth += 1;
        let result = self.function_inner(identity);
        self.depth -= 1;
        self.active.remove(&identity);
        let function = result?;
        self.functions.insert(identity, function.clone());
        Ok::<_, crate::ModelingError>(function)
    }
    fn function_inner(&mut self, identity: ContentHash) -> Result<Arc<FunctionProduct>> {
        let record = *self.pool.get(&identity).ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                "strict selected function is absent from its pool",
            )
        })?;
        pse_quantity::resolved::receipts::require_record(record)
            .map_err(|error| invalid(record.declaration, error.to_string()))?;
        let at = record.declaration;
        let syntax = self.syntax(&record.syntax)?;
        let physical = Arc::new(record.physical.restore(&syntax)?);
        let literals = record
            .literals
            .iter()
            .map(|(slot, ty)| Ok::<_, crate::ModelingError>((*slot, ty.restore(at)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let admissions = record
            .admissions
            .iter()
            .map(|(slot, admission)| Ok::<_, crate::ModelingError>((*slot, admission.restore(at)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        if literals.len() != record.literals.len() || admissions.len() != record.admissions.len() {
            return Err(invalid(at, "duplicate strict function physical coordinate"));
        }
        let product = FunctionProduct {
            identity,
            has_applicability_effects: false,
            declaration: at,
            syntax,
            physical,
            formals: record.formals.clone(),
            signature: record
                .signature
                .iter()
                .map(|ty| ty.restore(at))
                .collect::<Result<_>>()?,
            arguments: record
                .arguments
                .iter()
                .map(|argument| argument.restore(at))
                .collect::<Result<_>>()?,
            selections: record
                .selections
                .iter()
                .map(|selection| selection.restore(at))
                .collect::<Result<_>>()?,
            literals,
            result: record.result.restore(at)?,
            body: record.body,
            validity: record.validity,
            envelopes: record
                .envelopes
                .iter()
                .map(|guard| {
                    Ok::<_, crate::ModelingError>(FunctionEnvelope {
                        owner: guard.owner,
                        ty: guard.ty.restore(at)?,
                        predicate: guard.predicate,
                        reads: guard.reads.restore(),
                    })
                })
                .collect::<Result<_>>()?,
            applicability: record
                .applicability
                .iter()
                .map(EvidenceRecord::restore)
                .collect(),
            prerequisites: record.prerequisites.clone(),
            admissions,
            reduction_admission: record
                .reduction_admission
                .as_ref()
                .map(|value| value.restore(at))
                .transpose()?,
            reduction: record
                .reduction
                .as_ref()
                .map(|value| {
                    Ok::<_, crate::ModelingError>(crate::FiniteReduction {
                        kind: value.kind,
                        domain: value.domain,
                        prototype: value
                            .prototype
                            .restore()
                            .map_err(|error| invalid(at, error.to_string()))?,
                    })
                })
                .transpose()?,
            reduction_selection: record
                .reduction_selection
                .as_ref()
                .map(|value| {
                    Ok::<_, crate::ModelingError>(crate::specialize::finite::ReductionSelection {
                        carrier: value.carrier.restore()?,
                        selected: value
                            .selected
                            .iter()
                            .map(|value| value.restore(at))
                            .collect::<Result<Vec<_>>>()?
                            .into(),
                    })
                })
                .transpose()?,
            physical_operation: record
                .physical_operation
                .as_ref()
                .map(|value| value.restore(at))
                .transpose()?,
            output_boundary: record
                .output_boundary
                .as_ref()
                .map(|value| {
                    value
                        .restore()
                        .map_err(|error| invalid(at, error.to_string()))
                })
                .transpose()?,
            body_inputs: record
                .body_inputs
                .iter()
                .map(|value| value.as_ref().map(BodyInputRecord::restore).transpose())
                .collect::<Result<_>>()?,
            validity_reads: record.validity_reads.restore(),
            external: record
                .external
                .as_ref()
                .map(ExternalRecord::restore)
                .transpose()?,
            continuity: record.continuity,
        };
        let product = product.seal()?;
        if product.identity != identity {
            return Err(invalid(
                at,
                "restored function differs from its complete admitted semantic identity",
            ));
        }
        Ok::<_, crate::ModelingError>(product)
    }
}
