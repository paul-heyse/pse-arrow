// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Strict complete readmission of producer-issued support. No inference runs here.
use super::*;
use crate::portable::TypeRecord;
use pse_quantity::resolved::receipts::{
    CanonicalConversionRecord, ContractBoundaryRecord, ContractRecord, InferenceRecord,
    OutputBoundaryRecord,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::resolved) struct BodyInputRecord {
    relationship: crate::derived::Relationship,
    boundary: OutputBoundaryRecord,
}
impl BodyInputRecord {
    pub(in crate::resolved) fn capture(value: &BodyInputSupport) -> Self {
        Self {
            relationship: value.relationship,
            boundary: OutputBoundaryRecord::capture(&value.boundary),
        }
    }
    pub(in crate::resolved) fn restore(&self) -> Result<BodyInputSupport> {
        let boundary = self.boundary.restore().map_err(refusal)?;
        if boundary.operation() != self.relationship.identity() {
            return Err(refusal(
                "restored body authorization relationship differs from its exact operation",
            ));
        }
        match self.relationship {
            crate::derived::Relationship::FunctionBodyInputView { .. }
            | crate::derived::Relationship::ReconstructionResultView { .. } => {}
            _ => {
                return Err(refusal(
                    "restored body view lacks its declared authorization role",
                ));
            }
        }
        Ok::<_, crate::ModelingError>(BodyInputSupport {
            relationship: self.relationship,
            boundary,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CallRecord {
    function: usize,
    nominal_result: ContractRecord,
    wrt: Vec<usize>,
    inputs: Vec<ContractBoundaryRecord>,
    body_result: Option<BodyInputRecord>,
    partial: Option<InferenceRecord>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RangeRecord {
    lower: ContractBoundaryRecord,
    upper: ContractBoundaryRecord,
    lower_difference: InferenceRecord,
    upper_difference: InferenceRecord,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeRecord {
    slot: usize,
    ty: TypeRecord,
    contract: ContractRecord,
    operation: Option<InferenceRecord>,
    literal: Option<CanonicalConversionRecord>,
    boundary: Option<ContractBoundaryRecord>,
    call: Option<CallRecord>,
    range: Option<RangeRecord>,
    exponent: Option<pse_quantity::Ratio>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BooleanRecord {
    contract: ContractRecord,
    conversion: CanonicalConversionRecord,
    literal: InferenceRecord,
    conditional: InferenceRecord,
}
/// Complete opaque support record. It is meaningful only inside a caller-qualified
/// immutable recipe session and joined to the exact reconstructed typed graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportRecord {
    identity: ContentHash,
    syntax: ContentHash,
    nodes: Vec<NodeRecord>,
    outputs: Vec<ContractBoundaryRecord>,
    inputs: Vec<(usize, ContractRecord)>,
    premises: PhysicalEvidenceRecord,
    boolean: Option<BooleanRecord>,
    reduction: Option<InferenceRecord>,
    reduction_output: Option<ContractBoundaryRecord>,
}
fn refusal(error: impl std::fmt::Display) -> crate::ModelingError {
    invalid(SemanticId::NIL, error.to_string())
}
fn identity(value: &PhysicalSupport) -> ContentHash {
    let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
    hash.str("complete-support-record");
    value.frame(&mut hash);
    hash.finish_hash()
}
impl SupportRecord {
    /// Capture every actual consumed admission and the complete owner-computed premise identity.
    pub fn capture(value: &PhysicalSupport) -> Result<Self> {
        let nodes = value
            .order
            .iter()
            .map(|slot| {
                let node = value
                    .nodes
                    .get(slot)
                    .ok_or_else(|| refusal("support order lost an actual node"))?;
                Ok::<_, crate::ModelingError>(NodeRecord {
                    slot: slot.position(),
                    ty: TypeRecord::capture(&node.ty),
                    contract: ContractRecord::capture(&node.contract),
                    operation: node.operation.as_ref().map(InferenceRecord::capture),
                    literal: node
                        .literal
                        .as_ref()
                        .map(|literal| CanonicalConversionRecord::capture(&literal.conversion)),
                    boundary: node.boundary.as_ref().map(ContractBoundaryRecord::capture),
                    call: node.call.as_ref().map(|call| CallRecord {
                        function: call.function.position(),
                        nominal_result: ContractRecord::capture(&call.nominal_result),
                        wrt: call.wrt.clone(),
                        inputs: call
                            .inputs
                            .iter()
                            .map(ContractBoundaryRecord::capture)
                            .collect(),
                        body_result: call.body_result.as_ref().map(BodyInputRecord::capture),
                        partial: call.partial.as_ref().map(InferenceRecord::capture),
                    }),
                    range: node.range.as_ref().map(|range| RangeRecord {
                        lower: ContractBoundaryRecord::capture(&range.lower),
                        upper: ContractBoundaryRecord::capture(&range.upper),
                        lower_difference: InferenceRecord::capture(&range.lower_difference),
                        upper_difference: InferenceRecord::capture(&range.upper_difference),
                    }),
                    exponent: node.exponent,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if nodes.len() != value.nodes.len() {
            return Err(refusal("support has unordered retained admissions"));
        }
        Ok::<_, crate::ModelingError>(Self {
            identity: identity(value),
            syntax: value.syntax,
            nodes,
            outputs: value
                .outputs
                .iter()
                .map(ContractBoundaryRecord::capture)
                .collect(),
            inputs: value
                .inputs
                .iter()
                .map(|(slot, value)| (slot.position(), ContractRecord::capture(value)))
                .collect(),
            premises: value.premises.record(),
            boolean: value.boolean.as_ref().map(|boolean| BooleanRecord {
                contract: ContractRecord::capture(&boolean.contract),
                conversion: CanonicalConversionRecord::capture(&boolean.conversion),
                literal: InferenceRecord::capture(&boolean.literal),
                conditional: InferenceRecord::capture(&boolean.conditional),
            }),
            reduction: value.reduction.as_ref().map(InferenceRecord::capture),
            reduction_output: value
                .reduction_output
                .as_ref()
                .map(ContractBoundaryRecord::capture),
        })
    }
    /// Restore exact admitted records without a registry, context, resolver, expected type,
    /// or fallback. Qualified replay products explicitly cannot become live Salsa evidence.
    pub fn restore(&self, syntax: &ResolvedSyntax) -> Result<PhysicalSupport> {
        pse_quantity::resolved::receipts::require_record(self).map_err(refusal)?;
        if self.syntax != syntax.identity()? || self.outputs.len() != syntax.outputs.len() {
            return Err(refusal(
                "support record belongs to another exact typed graph or output layout",
            ));
        }
        let mut nodes = BTreeMap::new();
        let mut order = Vec::with_capacity(self.nodes.len());
        for record in &self.nodes {
            let slot = NodeIx::from(record.slot);
            if syntax.nodes.get(slot).is_none() || nodes.contains_key(&slot) {
                return Err(refusal(
                    "support record has an absent or repeated typed coordinate",
                ));
            }
            let call = record
                .call
                .as_ref()
                .map(|call| {
                    let function = FunctionIx::from(call.function);
                    let selected = syntax.functions.get(function).ok_or_else(|| {
                        refusal("support selected function is outside its exact graph")
                    })?;
                    if call.inputs.len() != selected.product.formals.len()
                        || call.wrt.iter().any(|ordinal| *ordinal >= call.inputs.len())
                    {
                        return Err(refusal("support selected function formal layout differs"));
                    }
                    Ok::<_, crate::ModelingError>(CallSupport {
                        function,
                        nominal_result: call.nominal_result.restore().map_err(refusal)?,
                        wrt: call.wrt.clone(),
                        inputs: call
                            .inputs
                            .iter()
                            .map(|value| value.restore().map_err(refusal))
                            .collect::<Result<_>>()?,
                        body_result: call
                            .body_result
                            .as_ref()
                            .map(BodyInputRecord::restore)
                            .transpose()?,
                        partial: call
                            .partial
                            .as_ref()
                            .map(|value| value.restore().map_err(refusal))
                            .transpose()?,
                    })
                })
                .transpose()?;
            let range = record
                .range
                .as_ref()
                .map(|range| {
                    Ok::<_, crate::ModelingError>(RangeSupport {
                        lower: range.lower.restore().map_err(refusal)?,
                        upper: range.upper.restore().map_err(refusal)?,
                        lower_difference: range.lower_difference.restore().map_err(refusal)?,
                        upper_difference: range.upper_difference.restore().map_err(refusal)?,
                    })
                })
                .transpose()?;
            let value = NodeSupport {
                ty: record.ty.restore(SemanticId::NIL.into())?,
                contract: record.contract.restore().map_err(refusal)?,
                operation: record
                    .operation
                    .as_ref()
                    .map(|value| value.restore().map_err(refusal))
                    .transpose()?,
                literal: record
                    .literal
                    .as_ref()
                    .map(|value| {
                        Ok::<_, crate::ModelingError>(LiteralSupport {
                            conversion: value.restore().map_err(refusal)?,
                        })
                    })
                    .transpose()?,
                boundary: record
                    .boundary
                    .as_ref()
                    .map(|value| value.restore().map_err(refusal))
                    .transpose()?,
                call,
                range,
                exponent: record.exponent,
            };
            order.push(slot);
            nodes.insert(slot, value);
        }
        for output in &syntax.outputs {
            if !nodes.contains_key(output) {
                return Err(refusal("support record omitted an actual typed output"));
            }
        }
        let boolean = self
            .boolean
            .as_ref()
            .map(|value| {
                Ok::<_, crate::ModelingError>(BooleanSupport {
                    contract: value.contract.restore().map_err(refusal)?,
                    conversion: value.conversion.restore().map_err(refusal)?,
                    literal: value.literal.restore().map_err(refusal)?,
                    conditional: value.conditional.restore().map_err(refusal)?,
                })
            })
            .transpose()?;
        let mut seen_inputs = std::collections::BTreeSet::new();
        let inputs = self
            .inputs
            .iter()
            .map(|(slot, record)| {
                let slot = SymbolIx::from(*slot);
                if !seen_inputs.insert(slot)
                    || syntax.symbols.at(slot).is_none_or(|symbol| {
                        symbol.definition.is_some() || symbol.local_role() == LocalRole::Member
                    })
                {
                    return Err(refusal(
                        "strict support input layout is absent, repeated or defined",
                    ));
                }
                Ok::<_, crate::ModelingError>((slot, record.restore().map_err(refusal)?))
            })
            .collect::<Result<Vec<_>>>()?;
        let value = PhysicalSupport {
            syntax: self.syntax,
            nodes,
            order,
            inputs,
            outputs: self
                .outputs
                .iter()
                .map(|value| value.restore().map_err(refusal))
                .collect::<Result<_>>()?,
            premises: self.premises.restore()?,
            boolean,
            reduction: self
                .reduction
                .as_ref()
                .map(|value| value.restore().map_err(refusal))
                .transpose()?,
            reduction_output: self
                .reduction_output
                .as_ref()
                .map(|value| value.restore().map_err(refusal))
                .transpose()?,
        };
        if identity(&value) != self.identity {
            return Err(refusal(
                "restored support differs from its complete admitted semantic identity",
            ));
        }
        Ok::<_, crate::ModelingError>(value)
    }
    /// Known wire extent, including every nested owned admission and complete nominal type.
    pub fn retained_bytes(&self) -> usize {
        self.nodes
            .iter()
            .fold(
                size_of::<Self>()
                    .saturating_add(
                        self.inputs
                            .capacity()
                            .saturating_mul(size_of::<(usize, ContractRecord)>()),
                    )
                    .saturating_add(
                        self.inputs
                            .iter()
                            .map(|(_, value)| value.retained_bytes())
                            .sum::<usize>(),
                    )
                    .saturating_add(
                        self.nodes
                            .capacity()
                            .saturating_mul(size_of::<NodeRecord>()),
                    )
                    .saturating_add(
                        self.outputs
                            .capacity()
                            .saturating_mul(size_of::<ContractBoundaryRecord>()),
                    )
                    .saturating_add(
                        self.outputs
                            .iter()
                            .map(ContractBoundaryRecord::retained_bytes)
                            .sum::<usize>(),
                    ),
                |bytes, node| {
                    bytes
                        .saturating_add(node.ty.retained_bytes())
                        .saturating_add(node.contract.retained_bytes())
                        .saturating_add(
                            node.operation
                                .as_ref()
                                .map_or(0, InferenceRecord::retained_bytes),
                        )
                        .saturating_add(
                            node.literal
                                .as_ref()
                                .map_or(0, CanonicalConversionRecord::retained_bytes),
                        )
                        .saturating_add(
                            node.boundary
                                .as_ref()
                                .map_or(0, ContractBoundaryRecord::retained_bytes),
                        )
                        .saturating_add(node.call.as_ref().map_or(0, |call| {
                            call.nominal_result
                                .retained_bytes()
                                .saturating_add(
                                    call.wrt.capacity().saturating_mul(size_of::<usize>()),
                                )
                                .saturating_add(
                                    call.inputs
                                        .capacity()
                                        .saturating_mul(size_of::<ContractBoundaryRecord>()),
                                )
                                .saturating_add(
                                    call.inputs
                                        .iter()
                                        .map(ContractBoundaryRecord::retained_bytes)
                                        .sum::<usize>(),
                                )
                                .saturating_add(
                                    call.body_result
                                        .as_ref()
                                        .map_or(0, |view| view.boundary.retained_bytes()),
                                )
                                .saturating_add(
                                    call.partial
                                        .as_ref()
                                        .map_or(0, InferenceRecord::retained_bytes),
                                )
                        }))
                        .saturating_add(node.range.as_ref().map_or(0, |range| {
                            range
                                .lower
                                .retained_bytes()
                                .saturating_add(range.upper.retained_bytes())
                                .saturating_add(range.lower_difference.retained_bytes())
                                .saturating_add(range.upper_difference.retained_bytes())
                        }))
                },
            )
            .saturating_add(self.boolean.as_ref().map_or(0, |boolean| {
                boolean
                    .contract
                    .retained_bytes()
                    .saturating_add(boolean.conversion.retained_bytes())
                    .saturating_add(boolean.literal.retained_bytes())
                    .saturating_add(boolean.conditional.retained_bytes())
            }))
            .saturating_add(
                self.reduction
                    .as_ref()
                    .map_or(0, InferenceRecord::retained_bytes),
            )
            .saturating_add(
                self.reduction_output
                    .as_ref()
                    .map_or(0, ContractBoundaryRecord::retained_bytes),
            )
    }
}
