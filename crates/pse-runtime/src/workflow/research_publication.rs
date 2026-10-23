// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Deliberate qualified-fit publication through ordinary authored admission and source CAS.
use super::{Runtime,ModelingPackage,WorkflowError,SourceDelta,ResearchSource,contract};
use super::research::{encoded,decoded};
use pse_operations::{canonical_projects::{Project,Branch,ProjectPublication,ResearchPublicationBinding},canonical_contexts::ContextSelection,canonical_research::ResearchPublication};
use pse_ids::SemanticId;
use pse_model::generated::identities::{DeclarationId,InstanceId};
use pse_relations::{columnar::RelationRow,generated::runtime::fitted_parameter_cells as cells};
use std::collections::{BTreeMap,BTreeSet};
/// Explicit original fitted-cell to authored scalar-parameter mapping.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchParameterTarget {
    /// Original global fitted parameter identity.
    pub parameter:SemanticId,
    /// Original target authored parameter declaration.
    pub declaration:DeclarationId,
    /// Exact root used for physical admission.
    pub root:DeclarationId,
    /// Explicit target instance.
    pub instance:InstanceId,
    /// Original target parameter path.
    pub path:String,
    /// Full target quantity convention, including basis/reference state.
    pub quantity:SemanticId,
    /// Target canonical unit; representation conversions require a separately admitted source edit.
    pub unit:SemanticId,
    /// Explicit closed target lower bound as IEEE bits; None explicitly means unbounded.
    pub lower_bits:Option<u64>,
    /// Explicit closed target upper bound as IEEE bits; None explicitly means unbounded.
    pub upper_bits:Option<u64>,
}
/// Original fit and exact target branch/source decision; retain it unchanged for settlement.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchPublicationRequest {
    /// Caller-issued immutable source operation.
    pub operation:String,
    /// Original research selection consumed before the fit executed.
    pub selection:String,
    /// Exact original run, never newest.
    pub run:String,
    /// Exact original attempt.
    pub attempt:String,
    /// Exact target project generation.
    pub project:Project,
    /// Exact target branch generation and canonical problem.
    pub branch:Branch,
    /// Exact original target source/physical context.
    pub target:ContextSelection,
    /// Explicit bounded mapping; all unrelated authored declarations remain inherited.
    pub targets:Vec<ResearchParameterTarget>,
}
/// Retained publication intent binds complete original evidence and target semantics.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchPublicationIntent {
    /// Complete original scientific decision.
    pub request:ResearchPublicationRequest,
    /// Sole source owner's exact activation intent and authenticated original result receipt.
    pub publication:ProjectPublication,
}
/// Nonexecuting locally admitted source preview. Values belong to the authored delta only.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchPublicationPreview {
    /// Retain unchanged through restart or uncertain completion.
    pub intent:ResearchPublicationIntent,
    /// Ordinary source delta, inspected and admitted through the original source owner.
    pub delta:SourceDelta,
}
impl ResearchPublicationIntent {
    fn commitment(&self)->Result<String,WorkflowError>{Ok(pse_operations::canonical_execution::result_payload_digest(&encoded(&self.request)?))}
    /// Authenticate complete original decision before any mutable-head lookup or effect.
    pub fn validate(&self)->Result<(),WorkflowError>{
        self.publication.validate()?;
        if self.publication.operation()!=self.request.operation||self.publication.expected()!=Some(self.request.target.context.revision.as_str())||self.publication.context_commitment()!=Some(self.commitment()?.as_str())||self.publication.research().is_none_or(|r|r.selection!=self.request.selection||r.run!=self.request.run||r.attempt!=self.request.attempt||r.mapping!=encoded(&self.request.targets).unwrap_or_default()) {return Err(contract("research publication immutable original inputs differ"));}
        Ok(())
    }
}
impl Runtime {
    async fn research_publication_source(&self,request:&ResearchPublicationRequest,cancel:&crate::CancelSource)->Result<(ModelingPackage,SourceDelta,ResearchPublicationBinding,super::CanonicalResultReader),WorkflowError>{
        if request.targets.is_empty()||request.targets.len()>512||request.project.key!=request.branch.project||request.project.key!=request.target.context.project{return Err(contract("research target project or population differs"));}
        let (original,fit)=self.research_selection_open(&request.selection,cancel).await?;
        let target=self.project_context_reopen(&request.target,cancel).await?;
        if target.recorded_revision()?.problem!=request.branch.problem||target.physical.identity()!=original.physical.identity(){return Err(contract("research target source owner or physical meaning differs"));}
        let mut reader=self.results(&request.run,&request.attempt,"runtime.fitted_parameter_cells",0,u64::MAX,cancel.token()).await?;
        let read=reader.selection();
        if read.run().revision!=original.recorded_revision()?.key{return Err(contract("original fit source revision differs"));}
        // This is the actual original consumed-input receipt, not a generic terminal status.
        let (envelope,original_receipt,_original_sources):(u8,Vec<u8>,Vec<(String,String,u64,String)>)=decoded(read.run().source_selection.as_slice())?;
        if envelope!=1{return Err(contract("original canonical source selection codec unavailable"));}
        let receipt:serde_json::Value=decoded(&original_receipt)?;
        let elements=receipt.as_array().ok_or_else(||contract("original fit source receipt unavailable"))?;
        if elements.first().and_then(|v|v.as_u64())!=Some(3){return Err(contract("original qualified research fit source receipt unavailable"));}
        let sources:Vec<super::OperationSource>=serde_json::from_value(elements.get(4).cloned().ok_or_else(||contract("original fit source receipt incomplete"))?).map_err(|e|contract(e.to_string()))?;
        if sources.len()!=1||sources[0]!=super::OperationSource::of(&original){return Err(contract("fit did not consume the exact frozen research selection"));}
        let set=read.sets().iter().find(|s|s.name==cells::RELATION_ID.to_string()).ok_or_else(||contract("qualified fit export explicitly unavailable"))?;
        if set.row_count==0||set.row_count>512{return Err(contract("qualified fit export population invalid"));}
        let binding=ResearchPublicationBinding{selection:request.selection.clone(),run:request.run.clone(),attempt:request.attempt.clone(),result_set:set.key.clone(),manifest_digest:read.manifest().digest.clone(),source_selection:read.run().source_selection.as_slice().to_vec(),mapping:encoded(&request.targets)?};
        let mut exported=BTreeMap::new();
        while let Some(batch)=reader.next_relation_batch().await? {
            for cell in cells::Row::rows(&batch).map_err(super::relation)? {
                if cell.fit_id!=fit||cell.source_revision!=original.revision.identity().as_id()||format!("run:{}",cell.run_id)!=request.run||!cell.value.is_finite()||exported.insert(cell.parameter_id,cell).is_some(){return Err(contract("original qualified fitted cell coordinates or value differ"));}
            }
        }
        let declarations=target.declarations().await?;
        let mut delta=SourceDelta{parent:target.revision_id().into(),..Default::default()};
        let mut targets=BTreeSet::new();let mut parameters=BTreeSet::new();
        for mapping in &request.targets {
            let cell=exported.get(&mapping.parameter).ok_or_else(||contract("requested qualified fitted cell unavailable"))?;
            if !targets.insert(mapping.declaration)||!parameters.insert(mapping.parameter)||cell.quantity_type_id!=mapping.quantity||cell.unit_id!=mapping.unit{return Err(contract("research mapping duplicate or physical meaning differs"));}
            let lower=mapping.lower_bits.map(f64::from_bits);let upper=mapping.upper_bits.map(f64::from_bits);
            if lower.is_some_and(|v|!v.is_finite()||cell.value<v)||upper.is_some_and(|v|!v.is_finite()||cell.value>v)||lower.zip(upper).is_some_and(|(l,u)|l>u){return Err(contract("fitted cell outside explicit target bounds"));}
            // Select the authored member before specialization prunes unrequested
            // mathematics. A parameter need not already support a case equation.
            let bindings=pse_modeling::Bindings{demand:vec![mapping.path.clone()],..Default::default()};
            let model=target.prepare(mapping.root,mapping.instance,bindings,Default::default(),cancel).await?;
            let compiled=model.compiled();
            let id=compiled.model.paths.get(&mapping.path).ok_or_else(||contract("research target parameter path absent"))?;
            let symbol=compiled.model.symbols.get(id).ok_or_else(||contract("research target parameter symbol absent"))?;
            let port=compiled.admitted.case().parameters().iter().find(|p|p.id==*id).ok_or_else(||contract("research target must be an authored numerical parameter"))?;
            if symbol.lineage.declaration!=mapping.declaration||port.quantity.as_id()!=mapping.quantity||port.unit.as_id()!=mapping.unit{return Err(contract("research target declaration or full physical port differs"));}
            let mut declaration=declarations.iter().find(|d|d.declaration_id==mapping.declaration).ok_or_else(||contract("research target declaration unavailable"))?.clone();
            if declaration.value.kind!=pse_model::generated::enums::ModelingDeclarationKind::Parameter{return Err(contract("research target is not an authored parameter"));}
            let value=declaration.value.binding.as_mut().ok_or_else(||contract("research parameter binding absent"))?;
            if !value.indices.is_empty(){return Err(contract("research scalar mapping cannot replace an indexed parameter"));}
            let unit=target.physical.quantities.unit_product(port.unit).map_err(|e|contract(e.to_string()))?;
            value.expression=Some(format!("{:?}{{{unit}}}",cell.value));
            delta.replacements.push(declaration);
        }
        // Ordinary admission resolves quantities, parameter domains and retained source constraints.
        let admitted=target.with_delta(delta.clone(),cancel).await?;
        Ok((admitted,delta,binding,reader))
    }
    /// Preview the actual authored delta after reading original protected qualified cells.
    pub async fn research_publication_preview(&self,request:&ResearchPublicationRequest,cancel:&crate::CancelSource)->Result<ResearchPublicationPreview,WorkflowError>{
        let (_,delta,binding,_reader)=self.research_publication_source(request,cancel).await?;
        let commitment=pse_operations::canonical_execution::result_payload_digest(&encoded(request)?);
        let publication=self.canonical_store()?.project_publication_intent(&request.project,&request.branch,&request.target.context.revision,None,&request.operation)?.with_context_commitment(&commitment)?.with_research_publication(binding)?;
        Ok(ResearchPublicationPreview{intent:ResearchPublicationIntent{request:request.clone(),publication},delta})
    }
    /// Settle the original operation first; otherwise re-read exact protected cells and admit source CAS.
    pub async fn research_publish(&self,intent:&ResearchPublicationIntent,cancel:&crate::CancelSource)->Result<ResearchPublication,WorkflowError>{
        intent.validate()?;
        let store=self.canonical_store()?;
        if store.settled_project_publication(&intent.publication).await?.is_some(){
            return store.research_publication(&intent.request.operation).await?.ok_or_else(||contract("accepted research publication receipt unavailable"));
        }
        let (admitted,_,binding,reader)=self.research_publication_source(&intent.request,cancel).await?;
        let actual=encoded(&binding)?;let original=encoded(intent.publication.research().ok_or_else(||contract("research publication binding absent"))?)?;
        if actual!=original{return Err(contract("research original protected evidence changed"));}
        // Keep the original result read protection alive until activation installs
        // the permanent research reference in the same source-owner transaction.
        self.project_publish_admitted(&intent.publication,&admitted,cancel).await?;
        let receipt=store.research_publication(&intent.request.operation).await?.ok_or_else(||contract("research publication receipt unavailable"))?;
        drop(reader);
        Ok(receipt)
    }
    /// Reopen the resulting admitted parameter source with original research ancestry for downstream studies.
    pub async fn research_publication_open(&self,key:&str,cancel:&crate::CancelSource)->Result<ModelingPackage,WorkflowError>{
        let store=self.canonical_store()?;
        let receipt=store.research_publication(key).await?.ok_or_else(||contract("research publication unavailable"))?;
        let (original,_)=self.research_selection_open(&receipt.selection,cancel).await?;
        let revision=store.revision(&receipt.revision).await?.ok_or_else(||contract("research published source unavailable"))?;
        let mut package=self.modeling_revision(revision,original.physical.clone(),BTreeMap::new()).await?;
        let source=ResearchSource{issuing_store:store.issuing_store()?.into(),selection:receipt.selection,dataset:original.research_source.as_ref().ok_or_else(||contract("original research source absent"))?.dataset.clone(),revision:receipt.revision,physical_context:original.physical.identity().to_string(),publication:Some(key.into())};
        self.restore_research_source(&mut package,Some(&source)).await?;
        Ok(package)
    }
}
