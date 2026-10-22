// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete selected provider joins, readmitted solely from qualified immutable records.
use super::*;
use pse_modeling::resolved::PhysicalEvidenceRecord;
use pse_quantity::resolved::receipts::ContractBoundaryRecord;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CallRecord {
    function: ContentHash,
    wrt: Vec<usize>,
    descriptor: ContentHash,
    output: usize,
    inputs: Vec<ContractBoundaryRecord>,
    result: ContractBoundaryRecord,
    prepared: pse_math::typed::PreparedProviderCallRecord,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectRecord {
    syntax: ContentHash,
    ordinal: usize,
    prepared: pse_math::typed::PreparedProviderCallRecord,
}
/// Opaque full selected descriptor/join product. Source names never act as restoration keys.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SelectedProvidersRecord {
    identity: ContentHash,
    descriptors: Vec<(ContentHash, pse_kernels::portable::ProviderRecord)>,
    calls: Vec<CallRecord>,
    direct_calls: Vec<DirectRecord>,
    premises: PhysicalEvidenceRecord,
}
fn identity(providers: &SelectedProviders) -> ContentHash {
    let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedBodyV1);
    hash.str("complete-selected-provider-record");
    providers.frame(&mut hash);
    hash.finish_hash()
}
impl SelectedProvidersRecord {
    pub(crate) fn capture(value: &SelectedProviders) -> Self {
        Self {
            identity: identity(value),
            descriptors: value
                .descriptors
                .iter()
                .map(|(identity, provider)| {
                    (
                        *identity,
                        pse_kernels::portable::ProviderRecord::capture(provider),
                    )
                })
                .collect(),
            calls: value
                .functions
                .iter()
                .map(|((function, wrt), call)| CallRecord {
                    function: *function,
                    wrt: wrt.clone(),
                    descriptor: call.descriptor,
                    output: call.output,
                    inputs: call
                        .inputs
                        .iter()
                        .map(ContractBoundaryRecord::capture)
                        .collect(),
                    result: ContractBoundaryRecord::capture(&call.result),
                    prepared: pse_math::typed::PreparedProviderCallRecord::capture(&call.prepared),
                })
                .collect(),
            direct_calls: value
                .direct_calls
                .iter()
                .map(|((syntax, ordinal), call)| DirectRecord {
                    syntax: *syntax,
                    ordinal: *ordinal,
                    prepared: pse_math::typed::PreparedProviderCallRecord::capture(call),
                })
                .collect(),
            premises: value.premises.record(),
        }
    }
    pub(crate) fn restore(
        &self,
        syntax: &ResolvedSyntax,
        support: &pse_modeling::resolved::PhysicalSupport,
    ) -> Result<SelectedProviders, MathError> {
        pse_quantity::resolved::receipts::require_record(self).map_err(contract)?;
        let mut descriptors = BTreeMap::new();
        for (key, record) in &self.descriptors {
            let descriptor = record.restore().map_err(contract)?;
            if descriptor.spec().identity() != *key
                || descriptors.insert(*key, descriptor).is_some()
            {
                return Err(contract("strict provider descriptor identity collision"));
            }
        }
        let mut functions = BTreeMap::new();
        for call in &self.calls {
            let descriptor = descriptors
                .get(&call.descriptor)
                .ok_or_else(|| contract("strict provider join has no exact selected descriptor"))?;
            if call.output >= descriptor.spec().outputs.len()
                || call.inputs.len() != descriptor.spec().inputs.len()
                || call.wrt.iter().any(|ordinal| *ordinal >= call.inputs.len())
            {
                return Err(contract(
                    "strict provider join output/formal layout differs",
                ));
            }
            let value = SelectedProviderCall {
                descriptor: call.descriptor,
                output: call.output,
                inputs: call
                    .inputs
                    .iter()
                    .map(|record| record.restore().map_err(contract))
                    .collect::<Result<_, _>>()?,
                result: call.result.restore().map_err(contract)?,
                prepared: std::sync::Arc::new(call.prepared.restore()?),
            };
            if value.prepared.descriptor().spec().identity() != call.descriptor
                || value.prepared.partial() != call.wrt
                || value
                    .prepared
                    .inputs()
                    .iter()
                    .ne(value.inputs.iter().map(AdmittedContractBoundary::target))
                || value.prepared.outputs().get(value.output) != Some(value.result.source())
            {
                return Err(contract(
                    "strict native provider facts differ from exact selected call boundaries",
                ));
            }
            if functions
                .insert((call.function, call.wrt.clone()), value)
                .is_some()
            {
                return Err(contract("duplicate strict selected provider join"));
            }
        }
        let mut direct_calls = BTreeMap::new();
        for call in &self.direct_calls {
            if direct_calls
                .insert(
                    (call.syntax, call.ordinal),
                    std::sync::Arc::new(call.prepared.restore()?),
                )
                .is_some()
            {
                return Err(contract("duplicate strict direct provider application"));
            }
        }
        let value = SelectedProviders {
            descriptors,
            functions,
            direct_calls,
            premises: self.premises.restore().map_err(contract)?,
        };
        // This walk checks graph joins only. It never calls the mutable provider inventory,
        // admission, quantity resolution, inference, or worker factories.
        let mut used = Used {
            value: &value,
            functions: BTreeSet::new(),
            descriptors: BTreeSet::new(),
            direct_calls: BTreeSet::new(),
            seen: BTreeSet::new(),
            depth: 0,
        };
        used.syntax(syntax, support)?;
        if used.functions.len() != value.functions.len()
            || used.descriptors.len() != value.descriptors.len()
            || used.direct_calls.len() != value.direct_calls.len()
            || identity(&value) != self.identity
        {
            return Err(contract(
                "strict selected provider product differs from its complete admitted graph joins",
            ));
        }
        Ok(value)
    }
}
struct Used<'a> {
    value: &'a SelectedProviders,
    functions: BTreeSet<(ContentHash, Vec<usize>)>,
    descriptors: BTreeSet<ContentHash>,
    direct_calls: BTreeSet<(ContentHash, usize)>,
    seen: BTreeSet<ContentHash>,
    depth: usize,
}
impl Used<'_> {
    fn syntax(
        &mut self,
        syntax: &ResolvedSyntax,
        support: &pse_modeling::resolved::PhysicalSupport,
    ) -> Result<(), MathError> {
        if self.depth >= 128 {
            return Err(MathError::Limit("strict selected provider depth"));
        }
        self.depth += 1;
        let layout = syntax.canonical_layout().map_err(contract)?;
        for (ordinal, slot) in layout.nodes.into_iter().enumerate() {
            match &syntax.nodes[slot] {
                Node::Apply { function, .. } | Node::Partial { function, .. } => {
                    let product = &syntax.functions[*function].product;
                    let wrt = if let Node::Partial { wrt, .. } = &syntax.nodes[slot] {
                        wrt.as_slice()
                    } else {
                        &[]
                    };
                    if let Some(external) = &product.external {
                        let join = self.value.call(product.identity, wrt)?;
                        let spec = self.value.descriptor(join.descriptor)?.spec();
                        if join.output != external.output
                            || spec.revision != external.revision
                            || spec.data != external.data
                            || spec.derivative_source != external.derivative_source
                            || spec.derivatives as u8 != external.derivatives
                            || spec.smoothness as u8 != external.smoothness
                            || join.inputs.len() != product.formals.len()
                            || wrt.len() > usize::from(external.derivatives)
                            || wrt.len() > usize::from(external.smoothness)
                        {
                            return Err(contract("strict external/provider contract join differs"));
                        }
                        if external.shapes.len() != spec.shapes.inputs.len() {
                            return Err(contract("strict external logical shape count differs"));
                        }
                        for shape in &external.shapes {
                            let logical = spec
                                .shapes
                                .inputs
                                .iter()
                                .find(|logical| logical.id == shape.argument)
                                .ok_or_else(|| {
                                    contract("strict external logical identity absent")
                                })?;
                            let end = shape
                                .start
                                .checked_add(shape.coordinates.len())
                                .ok_or(MathError::Limit("strict external scalar range"))?;
                            let ports = spec
                                .inputs
                                .get(shape.start..end)
                                .ok_or_else(|| contract("strict external scalar range absent"))?;
                            if logical.axes != shape.axes
                                || logical.coordinates != shape.coordinates
                                || logical.cells
                                    != ports.iter().map(|port| port.id).collect::<Vec<_>>()
                            {
                                return Err(contract(
                                    "strict external exact coordinate order differs",
                                ));
                            }
                        }
                        self.functions.insert((product.identity, wrt.to_vec()));
                        self.descriptors.insert(join.descriptor);
                    }
                    if self.seen.insert(product.identity) {
                        self.syntax(&product.syntax, &product.physical)?;
                    }
                }
                Node::Provider { selection, args } => {
                    let descriptor = self.value.descriptor(selection.identity)?;
                    let prepared = self.value.direct(layout.identity, ordinal)?;
                    if prepared.descriptor() != descriptor
                        || !prepared.partial().is_empty()
                        || selection.output >= prepared.outputs().len()
                        || prepared.inputs().len() != args.len()
                    {
                        return Err(contract("strict direct provider output/layout differs"));
                    }
                    for (arg, expected) in args.iter().zip(prepared.inputs()) {
                        if &support.node(*arg).map_err(contract)?.contract != expected {
                            return Err(contract(
                                "strict direct provider actual input contract differs",
                            ));
                        }
                    }
                    let result = support
                        .node(slot)
                        .map_err(contract)?
                        .boundary
                        .as_ref()
                        .ok_or_else(|| {
                            contract("strict direct provider lacks original result boundary")
                        })?;
                    if prepared.outputs().get(selection.output) != Some(result.source())
                        || result.target() != &support.node(slot).map_err(contract)?.contract
                    {
                        return Err(contract(
                            "strict direct provider original result contract differs",
                        ));
                    }
                    self.descriptors.insert(selection.identity);
                    self.direct_calls.insert((layout.identity, ordinal));
                }
                _ => {}
            }
        }
        self.depth -= 1;
        Ok(())
    }
}
