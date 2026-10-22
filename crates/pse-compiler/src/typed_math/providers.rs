// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual source-contract/provider joins are issued before semantic body-key publication.
#[path = "provider_records.rs"]
mod records;
use super::{MathError, ProviderCall};
use pse_ids::{ContentHash, FramedHasher};
use pse_modeling::{
    Type, TypeContext,
    resolved::{FunctionProduct, Node, ResolvedSyntax},
};
use pse_quantity::{AdmittedContractBoundary, IndexSet, ResolvedPhysicalContract, scheme::Scheme};
pub(crate) use records::SelectedProvidersRecord;
use std::collections::{BTreeMap, BTreeSet};

/// Selected source function and explicit formal partials map to one admitted registration.
#[derive(Clone, Debug)]
pub struct SelectedProviderCall {
    descriptor: ContentHash,
    output: usize,
    inputs: Vec<AdmittedContractBoundary>,
    result: AdmittedContractBoundary,
    prepared: std::sync::Arc<pse_math::typed::PreparedProviderCall>,
}
impl SelectedProviderCall {
    /// Complete descriptor identity; no implementation string lookup occurs in lowering.
    pub fn descriptor(&self) -> ContentHash {
        self.descriptor
    }
    /// Actual selected scalar output ordinal.
    pub fn output(&self) -> usize {
        self.output
    }
    /// Exact body-formal to descriptor input boundary at each actual scalar ordinal.
    pub fn inputs(&self) -> &[AdmittedContractBoundary] {
        &self.inputs
    }
    /// Complete prepared native units and actual ordered partial result contracts.
    pub fn prepared(&self) -> &pse_math::typed::PreparedProviderCall {
        &self.prepared
    }
    /// Actual descriptor partial-result to declared partial-result boundary.
    pub fn result(&self) -> &AdmittedContractBoundary {
        &self.result
    }
}
/// Complete demanded provider interpretation, with opaque actual quantity-owner witnesses.
#[derive(Clone, Debug)]
pub struct SelectedProviders {
    descriptors: BTreeMap<ContentHash, pse_kernels::AdmittedProvider>,
    functions: BTreeMap<(ContentHash, Vec<usize>), SelectedProviderCall>,
    direct_calls:
        BTreeMap<(ContentHash, usize), std::sync::Arc<pse_math::typed::PreparedProviderCall>>,
    premises: pse_modeling::resolved::PhysicalEvidence,
}
impl SelectedProviders {
    /// Require the exact immutable actual descriptor selected before body publication.
    pub fn descriptor(
        &self,
        identity: ContentHash,
    ) -> Result<&pse_kernels::AdmittedProvider, MathError> {
        self.descriptors
            .get(&identity)
            .ok_or_else(|| MathError::Contract("selected provider descriptor is absent".into()))
    }
    /// Require the exact function/ordered-partial join, including repeated formal ordinals.
    pub fn call(
        &self,
        function: ContentHash,
        wrt: &[usize],
    ) -> Result<&SelectedProviderCall, MathError> {
        self.functions
            .get(&(function, wrt.to_vec()))
            .ok_or_else(|| {
                MathError::Contract(
                    "selected source function/provider partial join is absent".into(),
                )
            })
    }
    /// Exact raw provider application in its canonical graph/node occurrence.
    pub fn direct(
        &self,
        syntax: ContentHash,
        ordinal: usize,
    ) -> Result<&pse_math::typed::PreparedProviderCall, MathError> {
        self.direct_calls
            .get(&(syntax, ordinal))
            .map(std::sync::Arc::as_ref)
            .ok_or_else(|| MathError::Contract("prepared direct provider call is absent".into()))
    }
    /// Exact selected registrations for portable capture; no implementation names are retained.
    pub fn descriptors(
        &self,
    ) -> impl Iterator<Item = (&ContentHash, &pse_kernels::AdmittedProvider)> {
        self.descriptors.iter()
    }
    /// Exact selected function/ordered-partial joins for portable readmission.
    pub fn calls(
        &self,
    ) -> impl Iterator<Item = (&(ContentHash, Vec<usize>), &SelectedProviderCall)> {
        self.functions.iter()
    }
    /// Full semantic provider premise; source implementation spelling is resolved already.
    pub fn frame(&self, hash: &mut FramedHasher) {
        hash.str("selected-provider-joins")
            .u64(self.descriptors.len() as u64);
        for identity in self.descriptors.keys() {
            hash.hash(identity);
        }
        hash.u64(self.functions.len() as u64);
        for ((function, wrt), call) in &self.functions {
            hash.hash(function).u64(wrt.len() as u64);
            for ordinal in wrt {
                hash.u64(*ordinal as u64);
            }
            hash.hash(&call.descriptor)
                .u64(call.output as u64)
                .u64(call.inputs.len() as u64);
            for input in &call.inputs {
                input.frame(hash);
            }
            call.result.frame(hash);
            call.prepared.frame(hash);
        }
        hash.u64(self.direct_calls.len() as u64);
        for ((syntax, ordinal), call) in &self.direct_calls {
            hash.hash(syntax).u64(*ordinal as u64);
            call.frame(hash);
        }
        self.premises.frame(hash);
    }
    /// Observe the actual joined provider quantity premises in the caller's read firewall.
    pub fn observe(
        &self,
        registry: &pse_quantity::QuantityRegistry,
        facts: &pse_quantity::PhysicalPreconditions,
    ) -> Result<bool, MathError> {
        Ok(self
            .premises
            .live()
            .map_err(|error| MathError::Contract(error.to_string()))?
            .observe(registry, facts))
    }
    /// Known owned extent; shared registration metadata is conservatively charged.
    pub fn retained_bytes(&self) -> usize {
        self.descriptors
            .values()
            .fold(size_of::<Self>(), |bytes, descriptor| {
                let spec = descriptor.spec();
                bytes
                    .saturating_add(size_of::<(ContentHash, pse_kernels::AdmittedProvider)>() + 96)
                    .saturating_add(
                        spec.inputs
                            .capacity()
                            .saturating_mul(size_of::<pse_kernels::Port>()),
                    )
                    .saturating_add(
                        spec.outputs
                            .capacity()
                            .saturating_mul(size_of::<pse_kernels::Port>()),
                    )
                    .saturating_add(spec.shapes.retained_bytes())
            })
            .saturating_add(
                self.functions
                    .iter()
                    .fold(0usize, |bytes, ((_, wrt), call)| {
                        bytes
                            .saturating_add(
                                size_of::<((ContentHash, Vec<usize>), SelectedProviderCall)>() + 96,
                            )
                            .saturating_add(wrt.capacity().saturating_mul(size_of::<usize>()))
                            .saturating_add(
                                call.inputs
                                    .capacity()
                                    .saturating_mul(size_of::<AdmittedContractBoundary>()),
                            )
                            .saturating_add(call.inputs.iter().fold(0usize, |bytes, boundary| {
                                bytes.saturating_add(boundary.heap_bytes())
                            }))
                            .saturating_add(call.result.heap_bytes())
                            .saturating_add(call.prepared.retained_bytes())
                    }),
            )
            .saturating_add(
                self.direct_calls
                    .values()
                    .map(|call| call.retained_bytes())
                    .sum::<usize>(),
            )
            .saturating_add(self.premises.retained_bytes())
    }
}
/// Join the actual selected direct graph to the existing immutable provider inventory.
/// This constructs no worker and grants no scientific role conversion.
pub fn prepare_selected_providers(
    syntax: &ResolvedSyntax,
    support: &pse_modeling::resolved::PhysicalSupport,
    lookup: &mut dyn FnMut(&str) -> Result<ProviderCall, MathError>,
    direct: &BTreeMap<ContentHash, pse_kernels::AdmittedProvider>,
    context: &TypeContext<'_>,
) -> Result<SelectedProviders, MathError> {
    let ((descriptors, functions, direct_calls), premises) =
        pse_quantity::premises::capture(context.quantities, context.preconditions, || {
            let mut producer = Producer {
                lookup,
                direct,
                available: BTreeMap::new(),
                context,
                descriptors: BTreeMap::new(),
                functions: BTreeMap::new(),
                direct_calls: BTreeMap::new(),
                seen: BTreeSet::new(),
                depth: 0,
            };
            producer.syntax(syntax, support)?;
            Ok::<_, MathError>((
                producer.descriptors,
                producer.functions,
                producer.direct_calls,
            ))
        })?;
    let premises = premises.ok_or_else(|| {
        MathError::Contract(
            "selected provider join lacks its complete actual physical premise capture".into(),
        )
    })?;
    Ok(SelectedProviders {
        descriptors,
        functions,
        direct_calls,
        premises: pse_modeling::resolved::PhysicalEvidence::capture(premises),
    })
}
struct Producer<'a, 'b> {
    lookup: &'a mut dyn FnMut(&str) -> Result<ProviderCall, MathError>,
    direct: &'a BTreeMap<ContentHash, pse_kernels::AdmittedProvider>,
    available: BTreeMap<String, ProviderCall>,
    context: &'a TypeContext<'b>,
    descriptors: BTreeMap<ContentHash, pse_kernels::AdmittedProvider>,
    functions: BTreeMap<(ContentHash, Vec<usize>), SelectedProviderCall>,
    direct_calls:
        BTreeMap<(ContentHash, usize), std::sync::Arc<pse_math::typed::PreparedProviderCall>>,
    seen: BTreeSet<ContentHash>,
    depth: usize,
}
fn contract(error: impl std::fmt::Display) -> MathError {
    MathError::Contract(error.to_string())
}
impl Producer<'_, '_> {
    fn syntax(
        &mut self,
        syntax: &ResolvedSyntax,
        support: &pse_modeling::resolved::PhysicalSupport,
    ) -> Result<(), MathError> {
        if self.depth >= 128 {
            return Err(MathError::Limit("selected provider join depth"));
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
                    if product.external.is_some() {
                        self.function(product, wrt)?;
                    }
                    if self.seen.insert(product.identity) {
                        self.syntax(&product.syntax, &product.physical)?;
                    }
                }
                Node::Provider { selection, args } => {
                    let actual = self
                        .direct
                        .get(&selection.identity)
                        .filter(|descriptor| descriptor.spec().identity() == selection.identity)
                        .ok_or_else(|| {
                            MathError::Contract(
                                "direct provider selection has no exact actual admitted descriptor"
                                    .into(),
                            )
                        })?;
                    if selection.output >= actual.spec().outputs.len() {
                        return Err(MathError::Contract(
                            "direct provider selected output is outside the actual descriptor"
                                .into(),
                        ));
                    }
                    let contracts = args
                        .iter()
                        .map(|arg| {
                            support
                                .node(*arg)
                                .map(|support| support.contract.clone())
                                .map_err(contract)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let call = pse_math::typed::PreparedProviderCall::prepare(
                        actual,
                        &contracts,
                        &[],
                        self.context.quantities,
                        self.context.preconditions,
                    )?;
                    let result = support
                        .node(slot)
                        .map_err(contract)?
                        .boundary
                        .as_ref()
                        .ok_or_else(|| {
                            contract("direct provider lacks its original result boundary")
                        })?;
                    if call.outputs().get(selection.output) != Some(result.source())
                        || result.target() != &support.node(slot).map_err(contract)?.contract
                    {
                        return Err(contract(
                            "direct provider actual output differs from its original declared scalar boundary",
                        ));
                    }
                    self.direct_calls
                        .insert((layout.identity, ordinal), std::sync::Arc::new(call));
                    self.descriptors.insert(selection.identity, actual.clone());
                }
                _ => {}
            }
        }
        self.depth -= 1;
        Ok(())
    }
    fn physical(&self, ty: &Type) -> Result<ResolvedPhysicalContract, MathError> {
        ty.quantity_scheme()
            .ok_or_else(|| {
                MathError::Contract(
                    "provider source contract requires an actual physical scalar".into(),
                )
            })?
            .resolve_contract_with_evidence(
                self.context.quantities,
                &BTreeMap::new(),
                self.context.preconditions,
            )
            .map_err(contract)
    }
    fn named(
        &self,
        quantity: pse_quantity::QuantityTypeId,
    ) -> Result<ResolvedPhysicalContract, MathError> {
        ResolvedPhysicalContract::named(quantity, IndexSet::new(), self.context.quantities)
            .map_err(contract)
    }
    fn function(&mut self, product: &FunctionProduct, wrt: &[usize]) -> Result<(), MathError> {
        let key = (product.identity, wrt.to_vec());
        if self.functions.contains_key(&key) {
            return Ok(());
        }
        let external = product.external.as_ref().ok_or_else(|| {
            MathError::Contract(
                "selected provider join requires an external source contract".into(),
            )
        })?;
        if !self.available.contains_key(&external.implementation) {
            let actual = (self.lookup)(&external.implementation)?;
            self.available
                .insert(external.implementation.clone(), actual);
        }
        let actual = self
            .available
            .get(&external.implementation)
            .ok_or_else(|| {
                MathError::Contract("selected actual provider registration disappeared".into())
            })?;
        let spec = actual.descriptor.spec();
        if spec.revision != external.revision
            || spec.data != external.data
            || spec.derivative_source != external.derivative_source
            || spec.derivatives as u8 != external.derivatives
            || spec.smoothness as u8 != external.smoothness
        {
            return Err(MathError::Contract("external implementation differs from its authored revision, data or derivative contract".into()));
        }
        if spec.inputs.len() != product.formals.len() {
            return Err(MathError::Contract(
                "external actual scalar input arity differs from registration".into(),
            ));
        }
        if external.shapes.len() != spec.shapes.inputs.len() {
            return Err(MathError::Contract(
                "external logical input shape count".into(),
            ));
        }
        for shape in &external.shapes {
            let logical = spec
                .shapes
                .inputs
                .iter()
                .find(|logical| logical.id == shape.argument)
                .ok_or_else(|| MathError::Contract("external logical argument identity".into()))?;
            let end = shape
                .start
                .checked_add(shape.coordinates.len())
                .ok_or(MathError::Limit("external logical extent"))?;
            let ports = spec
                .inputs
                .get(shape.start..end)
                .ok_or_else(|| MathError::Contract("external logical scalar range".into()))?;
            if logical.axes != shape.axes
                || logical.coordinates != shape.coordinates
                || logical.cells != ports.iter().map(|port| port.id).collect::<Vec<_>>()
            {
                return Err(MathError::Contract(
                    "external coordinate order differs from registration".into(),
                ));
            }
        }
        if wrt.len() > usize::from(external.derivatives)
            || wrt.len() > usize::from(external.smoothness)
            || wrt.iter().any(|ordinal| *ordinal >= spec.inputs.len())
        {
            return Err(MathError::Contract(
                "external partial exceeds the actual declared formal/capability contract".into(),
            ));
        }
        let output = spec.outputs.get(external.output).ok_or_else(|| {
            MathError::Contract("external output ordinal outside registration".into())
        })?;
        let mut inputs = Vec::with_capacity(spec.inputs.len());
        for (slot, port) in product.formals.iter().zip(&spec.inputs) {
            let formal = product.syntax.symbols.at(*slot).ok_or_else(|| {
                MathError::Contract("external body formal is outside its actual local owner".into())
            })?;
            inputs.push(
                AdmittedContractBoundary::prepare(
                    &self.physical(&formal.ty)?,
                    &self.named(port.quantity)?,
                    self.context.quantities,
                )
                .map_err(contract)?,
            );
        }
        let mut declared = product.result.quantity_scheme().cloned().ok_or_else(|| {
            MathError::Contract("external selected result is not an actual physical scalar".into())
        })?;
        let mut produced = Scheme::Concrete(output.quantity);
        for ordinal in wrt {
            let slot = product.formals[*ordinal];
            let actual = &product
                .syntax
                .symbols
                .at(slot)
                .ok_or_else(|| {
                    MathError::Contract(
                        "external partial formal is outside its actual local owner".into(),
                    )
                })?
                .ty;
            let formal = actual.quantity_scheme().cloned().ok_or_else(|| {
                MathError::Contract(
                    "external differentiated formal has no actual physical type".into(),
                )
            })?;
            declared = Scheme::Quotient(
                Box::new(Scheme::Delta(Box::new(declared))),
                Box::new(Scheme::Delta(Box::new(formal))),
            );
            produced = Scheme::Quotient(
                Box::new(Scheme::Delta(Box::new(produced))),
                Box::new(Scheme::Delta(Box::new(Scheme::Concrete(
                    spec.inputs[*ordinal].quantity,
                )))),
            );
        }
        let source = produced
            .resolve_contract_with_evidence(
                self.context.quantities,
                &BTreeMap::new(),
                self.context.preconditions,
            )
            .map_err(contract)?;
        let expected = declared
            .resolve_contract_with_evidence(
                self.context.quantities,
                &BTreeMap::new(),
                self.context.preconditions,
            )
            .map_err(contract)?;
        let result = AdmittedContractBoundary::prepare(&source, &expected, self.context.quantities)
            .map_err(contract)?;
        let descriptor = spec.identity();
        self.descriptors
            .insert(descriptor, actual.descriptor.clone());
        let native_inputs = inputs
            .iter()
            .map(|boundary| boundary.target().clone())
            .collect::<Vec<_>>();
        let prepared = std::sync::Arc::new(pse_math::typed::PreparedProviderCall::prepare(
            &actual.descriptor,
            &native_inputs,
            wrt,
            self.context.quantities,
            self.context.preconditions,
        )?);
        if prepared.outputs().get(external.output) != Some(result.source()) {
            return Err(MathError::Contract(
                "prepared provider partial result differs from selected source boundary".into(),
            ));
        }
        self.functions.insert(
            key,
            SelectedProviderCall {
                descriptor,
                output: external.output,
                inputs,
                result,
                prepared,
            },
        );
        Ok(())
    }
}
