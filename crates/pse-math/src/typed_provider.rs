// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual provider physical contracts and representation operations prepared before
//! numerical construction. This owner has no modeling/source dependency.
use super::*;
use pse_ids::{ContentHash,FramedHasher};
use pse_quantity::unit::RepresentationConversionPlan;
use pse_quantity::resolved::receipts::{ContractRecord,RepresentationConversionRecord};
use serde::{Serialize,Deserialize};

/// Complete actual provider application. Every field is issued by prepare or strict
/// caller-qualified restoration; callers cannot fabricate detached conversion coefficients.
#[derive(Clone,Debug)]
pub struct PreparedProviderCall {
    registration:pse_kernels::AdmittedProvider,
    partial:Vec<usize>,
    inputs:Vec<ResolvedPhysicalContract>,
    input_conversions:Vec<RepresentationConversionPlan>,
    outputs:Vec<ResolvedPhysicalContract>,
    output_conversions:Vec<RepresentationConversionPlan>,
}
impl PreparedProviderCall {
    /// Prepare the exact existing registered-port mathematics once in its physical read capture.
    pub fn prepare(registration:&pse_kernels::AdmittedProvider,inputs:&[ResolvedPhysicalContract],partial:&[usize],registry:&QuantityRegistry,checker:&dyn InvariantChecker)->Result<Self,MathError> {
        let spec=registration.spec();
        if inputs.len()!=spec.inputs.len() || partial.iter().any(|ordinal|*ordinal>=inputs.len()) || partial.len()>spec.derivatives.min(spec.smoothness) as usize {return Err(MathError::Contract("prepared provider arity or partial capability".into()));}
        if inputs.windows(2).any(|pair|pair[0].indices()!=pair[1].indices()) {return Err(MathError::Contract("provider lexical indices differ".into()));}
        let input_conversions=inputs.iter().zip(&spec.inputs).map(|(input,port)| {
            pse_quantity::admission::require_same_contract(input.require_named()?,port.quantity,registry)?;
            let canonical=registry.quantity_type(port.quantity)?.canonical_unit;
            RepresentationConversionPlan::registered(registry,port.quantity,canonical,port.unit).map_err(MathError::from)
        }).collect::<Result<Vec<_>,_>>()?;
        let mut outputs=Vec::with_capacity(spec.outputs.len());let mut output_conversions=Vec::with_capacity(spec.outputs.len());
        for port in &spec.outputs {
            let canonical=registry.quantity_type(port.quantity)?.canonical_unit;
            output_conversions.push(RepresentationConversionPlan::registered(registry,port.quantity,port.unit,canonical)?);
            let mut scheme=pse_quantity::scheme::Scheme::Concrete(port.quantity);
            for ordinal in partial {scheme=pse_quantity::scheme::Scheme::Quotient(Box::new(pse_quantity::scheme::Scheme::Delta(Box::new(scheme))),Box::new(pse_quantity::scheme::Scheme::Delta(Box::new(pse_quantity::scheme::Scheme::from_contract(inputs[*ordinal].clone())))));}
            outputs.push(scheme.resolve_contract_with_evidence(registry,&std::collections::BTreeMap::new(),checker).map_err(|error|MathError::Contract(error.to_string()))?);
        }
        Ok(Self {registration:registration.clone(),partial:partial.to_vec(),inputs:inputs.to_vec(),input_conversions,outputs,output_conversions})
    }
    /// Exact full descriptor selected by this preparation.
    pub fn descriptor(&self)->&pse_kernels::AdmittedProvider {&self.registration}
    /// Complete ordered actual input contracts.
    pub fn inputs(&self)->&[ResolvedPhysicalContract] {&self.inputs}
    /// Exact ordered actual output/partial contracts.
    pub fn outputs(&self)->&[ResolvedPhysicalContract] {&self.outputs}
    /// Actual ordered differentiated input positions.
    pub fn partial(&self)->&[usize] {&self.partial}
    /// Full physical semantics, including all unit directions and exact coefficient bits.
    pub fn frame(&self,hash:&mut FramedHasher) {
        hash.str("prepared-provider-call").hash(&self.registration.spec().identity()).u64(self.partial.len() as u64);for ordinal in &self.partial {hash.u64(*ordinal as u64);}
        hash.u64(self.inputs.len() as u64);for (input,conversion) in self.inputs.iter().zip(&self.input_conversions) {input.frame(hash);conversion.frame(hash);}
        hash.u64(self.outputs.len() as u64);for (output,conversion) in self.outputs.iter().zip(&self.output_conversions) {output.frame(hash);conversion.frame(hash);}
    }
    /// Known retained facts; registration metadata remains a shared immutable owner.
    pub fn retained_bytes(&self)->usize {
        size_of::<Self>().saturating_add(self.partial.capacity().saturating_mul(size_of::<usize>()))
            .saturating_add(self.inputs.capacity().saturating_add(self.outputs.capacity()).saturating_mul(size_of::<ResolvedPhysicalContract>()))
            .saturating_add(self.inputs.iter().chain(&self.outputs).map(ResolvedPhysicalContract::heap_bytes).sum::<usize>())
            .saturating_add(self.input_conversions.capacity().saturating_add(self.output_conversions.capacity()).saturating_mul(size_of::<RepresentationConversionPlan>()))
            .saturating_add(self.input_conversions.iter().chain(&self.output_conversions).map(RepresentationConversionPlan::heap_bytes).sum::<usize>())
    }
}
/// Complete prepared provider facts, restored only by their caller-qualified recipe owner.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedProviderCallRecord {identity:ContentHash,registration:pse_kernels::portable::ProviderRecord,partial:Vec<usize>,inputs:Vec<ContractRecord>,input_conversions:Vec<RepresentationConversionRecord>,outputs:Vec<ContractRecord>,output_conversions:Vec<RepresentationConversionRecord>}
fn identity(value:&PreparedProviderCall)->ContentHash {let mut hash=FramedHasher::new(pse_ids::Frame::MathTypedDefinitionV10);hash.str("prepared-provider-record");value.frame(&mut hash);hash.finish_hash()}
impl PreparedProviderCallRecord {
    /// Capture opaque complete actual admissions, never a guessed output type.
    pub fn capture(value:&PreparedProviderCall)->Self {Self {identity:identity(value),registration:pse_kernels::portable::ProviderRecord::capture(&value.registration),partial:value.partial.clone(),inputs:value.inputs.iter().map(ContractRecord::capture).collect(),input_conversions:value.input_conversions.iter().map(RepresentationConversionRecord::capture).collect(),outputs:value.outputs.iter().map(ContractRecord::capture).collect(),output_conversions:value.output_conversions.iter().map(RepresentationConversionRecord::capture).collect()}}
    /// Restore only inside the enclosing caller-qualified immutable recipe session.
    /// No registry, inference, worker factory or conversion calculation is available here.
    pub fn restore(&self)->Result<PreparedProviderCall,MathError> {
        pse_quantity::resolved::receipts::require_record(self)?;
        let registration=self.registration.restore().map_err(|error|MathError::Contract(error.to_string()))?;let spec=registration.spec();
        if self.inputs.len()!=spec.inputs.len() || self.outputs.len()!=spec.outputs.len() || self.input_conversions.len()!=self.inputs.len() || self.output_conversions.len()!=self.outputs.len() || self.partial.len()>spec.derivatives.min(spec.smoothness) as usize || self.partial.iter().any(|ordinal|*ordinal>=self.inputs.len()) {return Err(MathError::Contract("strict prepared provider layout or capability differs".into()));}
        let value=PreparedProviderCall {registration,partial:self.partial.clone(),inputs:self.inputs.iter().map(ContractRecord::restore).collect::<Result<_,_>>()?,input_conversions:self.input_conversions.iter().map(RepresentationConversionRecord::restore).collect::<Result<_,_>>()?,outputs:self.outputs.iter().map(ContractRecord::restore).collect::<Result<_,_>>()?,output_conversions:self.output_conversions.iter().map(RepresentationConversionRecord::restore).collect::<Result<_,_>>()?};
        for (conversion,port) in value.input_conversions.iter().zip(&value.registration.spec().inputs) {if conversion.quantity()!=port.quantity || conversion.target_unit()!=port.unit {return Err(MathError::Contract("strict prepared provider input representation differs".into()));}}
        for (conversion,port) in value.output_conversions.iter().zip(&value.registration.spec().outputs) {if conversion.quantity()!=port.quantity || conversion.source_unit()!=port.unit {return Err(MathError::Contract("strict prepared provider output representation differs".into()));}}
        if identity(&value)!=self.identity {return Err(MathError::Contract("strict prepared provider complete physical identity differs".into()));}Ok(value)
    }
}
impl BodyBuilder<'_> {
    /// Consume a complete prepared provider call. All physical operations were issued
    /// before this numerical constructor; strict replay performs no physical resolution.
    pub fn provider_partial_admitted(&mut self,prepared:&PreparedProviderCall,inputs:&[TypedValue],source:SemanticId)->Result<Vec<TypedValue>,MathError> {
        self.tick()?;
        let spec=prepared.registration.spec();let partial=&prepared.partial;
        if inputs.len()!=prepared.inputs.len() || inputs.iter().zip(&prepared.inputs).any(|(actual,expected)|&actual.quantity!=expected) || inputs.windows(2).any(|pair|pair[0].indices!=pair[1].indices) {return Err(MathError::Contract("actual provider operands differ from their exact prepared physical contracts".into()));}
        let remaining=(spec.derivatives.min(spec.smoothness) as usize).checked_sub(partial.len()).ok_or_else(||MathError::Contract("prepared provider capability changed".into()))?;
        let available=match remaining {0=>DerivativeOrder::Value,1=>DerivativeOrder::First,_=>DerivativeOrder::Second};
        let cache_key=(spec.key(),inputs.iter().map(|value|value.atom.clone()).collect(),partial.clone());
        if !self.physical_only && let Some(outputs)=self.provider_cache.get(&cache_key) {return Ok(outputs.iter().cloned().map(|mut value| {value.effects.extend(inputs.iter().flat_map(|value|value.effects.iter().copied()));value.source=source;value.indices=inputs.first().map_or_else(IndexSet::new,|value|value.indices.clone());value}).collect());}
        let mut input_slots=Vec::with_capacity(inputs.len());
        for (value,conversion) in inputs.iter().zip(&prepared.input_conversions) {input_slots.push(self.materialize(&value.atom*Atom::num(conversion.scale())+Atom::num(conversion.offset()),source)?);}
        let mut slots=Vec::with_capacity(prepared.outputs.len());let mut outputs=Vec::with_capacity(prepared.outputs.len());
        for (quantity,conversion) in prepared.outputs.iter().zip(&prepared.output_conversions) {
            let slot=if self.physical_only {0} else {self.slot()?};slots.push(slot);
            let scale=partial.iter().fold(conversion.scale(),|scale,ordinal|scale*prepared.input_conversions[*ordinal].scale());
            if !scale.is_finite() {return Err(MathError::Contract("external partial unit scale".into()));}
            outputs.push(TypedValue {effects:inputs.iter().flat_map(|value|value.effects.iter().copied()).chain([slot]).collect(),atom:if self.physical_only {Atom::num(0)} else {library::formal(slot)?*Atom::num(scale)+Atom::num(if partial.is_empty() {conversion.offset()} else {0.})},quantity:quantity.clone(),indices:inputs.first().map_or_else(IndexSet::new,|value|value.indices.clone()),source});
        }
        self.provider_order=self.provider_order.min(available);
        if !self.physical_only {self.stages.push(Stage::Provider {spec:spec.clone(),partial:partial.clone(),inputs:input_slots,outputs:slots,source});self.provider_cache.insert(cache_key,outputs.clone());}
        Ok(outputs)
    }
}

#[cfg(test)]
#[path="typed_provider_tests.rs"]
mod tests;
