// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Research versions select original admitted typed objects, never duplicate measurements.
use super::{Runtime,ModelingPackage,WorkflowError,contract};
use pse_operations::{canonical_projects::Project,canonical_contexts::ContextSelection,canonical_research::{ResearchDataset,ResearchSelection}};
use pse_model::generated::identities::{DeclarationId,FitId};
use pse_ids::SemanticId;
use std::collections::BTreeSet;
/// Authenticated original research ancestry carried by actual operation inputs.
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchSource {
    /// Canonical installation incarnation.
    pub issuing_store:String,
    /// Frozen original observation selection.
    pub selection:String,
    /// Immutable dataset version.
    pub dataset:String,
    /// Exact currently consumed canonical source.
    pub revision:String,
    /// Original physical meaning.
    pub physical_context:String,
    /// Explicit parameter publication ancestry, when consuming its new source.
    pub publication:Option<String>,
}
/// Exact attribute selection into the original typed measurement owner.
#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchDataObject {
    /// Original admitted reader root (normally the experiment case).
    pub root:DeclarationId,
    /// Original typed inline or package-data record identity.
    pub record:DeclarationId,
    /// Original authored value attribute, including any authored derivation.
    pub attribute:String,
    /// Explicit authored transformed standard uncertainty, or original annotation.
    pub standard_deviation_attribute:Option<String>,
    /// Expected full physical quantity convention, including basis/reference state.
    pub quantity:SemanticId,
    /// Original acquisition-clock meaning; never inferred from recording time.
    pub acquisition_clock:String,
    /// Parent version member for curated/derived correspondence.
    pub parent_member:Option<u64>,
    /// Authored declarations owning the selected calibration or quantity conversion.
    pub calibration:Vec<DeclarationId>,
}
/// Create one immutable research version from an exact temporal context.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchDatasetRequest {
    /// Original caller operation/version identity.
    pub key:String,
    /// Exact project generation.
    pub project:Project,
    /// Exact effective-time and recorded-cut selection.
    pub context:ContextSelection,
    /// Original parent version, never a floating latest selector.
    pub parent:Option<String>,
    /// raw, curated or derived.
    pub role:String,
    /// Bounded exact selections; data remains in original sources.
    pub members:Vec<ResearchDataObject>,
}
/// One observation's explicit selection explanation.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchObservation {
    /// Original observation identity in the authored fit.
    pub observation:DeclarationId,
    /// Exact dataset member position.
    pub member:u64,
    /// Explicit inclusion, exclusion or calibration explanation.
    pub reason:String,
}
/// Freeze an already authored fit's complete selection; no fitting shortcut is introduced.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchSelectionRequest {
    /// Immutable original selection operation.
    pub key:String,
    /// Exact project generation.
    pub project:Project,
    /// Exact recorded context.
    pub context:ContextSelection,
    /// Exact immutable dataset version.
    pub dataset:String,
    /// Existing authored fitting declaration.
    pub fit:FitId,
    /// Complete observation population, including exclusions.
    pub observations:Vec<ResearchObservation>,
}
/// Exact discovery and selected detail commands; metadata does not retain payloads.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum ResearchReadRequest {
    /// Indexed immutable-key seek page.
    Datasets { #[doc="Project owner."] project:String, #[doc="Exclusive immutable key."] after:String, #[doc="At most128metadata records."] limit:usize },
    /// One exact version.
    Dataset { #[doc="Original version key."] key:String },
    /// One exact frozen selection.
    Selection { #[doc="Original selection key."] key:String },
}
/// Metadata inventory without payload authority.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum ResearchReadResult {
    /// Bounded dataset headers.
    Datasets { #[doc="Original immutable versions."] rows:Vec<ResearchDataset> },
    /// Original version, if available.
    Dataset { #[doc="Exact original metadata."] row:Option<ResearchDataset> },
    /// Original selection, if available.
    Selection { #[doc="Exact original metadata."] row:Option<ResearchSelection> },
}
pub(super) fn encoded<T:serde::Serialize>(v:&T)->Result<Vec<u8>,WorkflowError>{serde_json::to_vec(v).map_err(|e|contract(e.to_string()))}
pub(super) fn decoded<T:serde::de::DeserializeOwned>(v:&[u8])->Result<T,WorkflowError>{serde_json::from_slice(v).map_err(|e|contract(e.to_string()))}
fn checkpoint(cancel:&crate::CancelSource)->Result<(),WorkflowError>{cancel.token().checkpoint().map_err(pse_engine::EngineError::from)?;Ok(())}
impl Runtime {
    /// Indexed compact discovery; no measurement/result payload is decoded.
    pub async fn research_read(&self,request:&ResearchReadRequest)->Result<ResearchReadResult,WorkflowError>{
        let store=self.canonical_store()?;
        Ok(match request {
            ResearchReadRequest::Datasets{project,after,limit}=>ResearchReadResult::Datasets{rows:store.research_datasets(project,after,*limit).await?},
            ResearchReadRequest::Dataset{key}=>ResearchReadResult::Dataset{row:store.research_dataset(key).await?},
            ResearchReadRequest::Selection{key}=>ResearchReadResult::Selection{row:store.research_selection(key).await?},
        })
    }
    /// Explicitly reopen selected data through its original protected source/physical owners.
    pub async fn research_dataset_open(&self,key:&str,context:&ContextSelection,cancel:&crate::CancelSource)->Result<ModelingPackage,WorkflowError>{
        let row=self.canonical_store()?.research_dataset(key).await?.ok_or_else(||contract("research dataset unavailable"))?;
        if row.context!=context.context.key || row.project!=context.context.project || row.revision!=context.context.revision || row.physical_context!=context.context.physical_context{return Err(contract("research dataset context differs"));}
        self.project_context_reopen(context,cancel).await
    }
    /// Validate typed measurements and append a new immutable version, retaining parent raw data.
    pub async fn research_dataset_create(&self,request:&ResearchDatasetRequest,cancel:&crate::CancelSource)->Result<ResearchDataset,WorkflowError>{
        checkpoint(cancel)?;
        if request.members.is_empty()||request.members.len()>512 || request.project.key!=request.context.context.project || !["raw","curated","derived"].contains(&request.role.as_str()){return Err(contract("research dataset population or context invalid"));}
        let package=self.project_context_reopen(&request.context,cancel).await?;
        let parent=if let Some(key)=&request.parent {
            let row=self.canonical_store()?.research_dataset(key).await?.ok_or_else(||contract("research parent unavailable"))?;
            if row.project!=request.project.key||row.physical_context!=request.context.context.physical_context{return Err(contract("research parent physical context differs"));}
            Some(decoded::<Vec<ResearchDataObject>>(&row.members)?)
        }else{None};
        if request.role!="raw"&&parent.is_none(){return Err(contract("curated/derived research requires original parent"));}
        let mut identities=BTreeSet::new();
        for member in &request.members {
            checkpoint(cancel)?;
            if member.attribute.is_empty()||member.acquisition_clock.is_empty()||member.acquisition_clock.len()>4096||!identities.insert((member.record,member.attribute.clone())){return Err(contract("research member identity or acquisition clock invalid"));}
            if let Some(parent)=&parent {
                let original=member.parent_member.and_then(|i|usize::try_from(i).ok()).and_then(|i|parent.get(i)).ok_or_else(||contract("research parent correspondence missing"))?;
                if original.quantity!=member.quantity{return Err(contract("research correspondence physical meaning differs"));}
                if request.role=="derived"&&(member.calibration.is_empty()||member.attribute==original.attribute&&member.record==original.record){return Err(contract("derived data requires explicit authored calibration and derived attribute"));}
            } else if member.parent_member.is_some()||!member.calibration.is_empty(){return Err(contract("raw data cannot claim calibration ancestry"));}
            let mut roots=vec![member.root,member.record];roots.extend(&member.calibration);
            let revision=package.selected_revision_roots(&roots,cancel).await?;
            let measured=revision.checked().measurement(member.root,member.record,&member.attribute,member.standard_deviation_attribute.as_deref()).map_err(|e|contract(e.to_string()))?;
            if measured.quantity.as_id()!=member.quantity{return Err(contract("research member quantity/basis differs"));}
            if measured.value.is_some_and(|v|!v.is_finite())||measured.standard_deviation.is_some_and(|v|!v.is_finite()||v<=0.){return Err(contract("research measurement or standard uncertainty invalid"));}
        }
        let row=ResearchDataset{key:request.key.clone(),project:request.project.key.clone(),revision:request.context.context.revision.clone(),physical_context:request.context.context.physical_context.clone(),context:request.context.context.key.clone(),parent:request.parent.clone(),role:request.role.clone(),members:encoded(&request.members)?,request:Vec::new()};
        Ok(self.canonical_store()?.create_research_dataset(&request.project,row).await?)
    }
    /// Freeze every original fit observation and verify its exact selected typed attribute.
    pub async fn research_selection_create(&self,request:&ResearchSelectionRequest,cancel:&crate::CancelSource)->Result<ResearchSelection,WorkflowError>{
        let package=self.research_dataset_open(&request.dataset,&request.context,cancel).await?;
        if request.project.key!=request.context.context.project{return Err(contract("research selection project differs"));}
        let dataset=self.canonical_store()?.research_dataset(&request.dataset).await?.ok_or_else(||contract("research dataset unavailable"))?;
        let members:Vec<ResearchDataObject>=decoded(&dataset.members)?;
        let fit=package.fit_declaration(request.fit).await?;
        if fit.observations.len()!=request.observations.len(){return Err(contract("research selection must preserve complete observation population"));}
        let mut selected=BTreeSet::new();
        for item in &request.observations {
            checkpoint(cancel)?;
            let observation=fit.observations.iter().find(|o|o.observation_id==item.observation).ok_or_else(||contract("research observation absent from fit"))?;
            let member=usize::try_from(item.member).ok().and_then(|i|members.get(i)).ok_or_else(||contract("research observation member absent"))?;
            let experiment=fit.experiments.iter().find(|e|e.experiment_id==observation.experiment_id).ok_or_else(||contract("research fit experiment absent"))?;
            if item.reason.is_empty()||item.reason.len()>4096||!selected.insert(item.observation)||member.root!=experiment.case_id||member.record!=observation.observation_id||member.attribute!=observation.value_attribute||member.standard_deviation_attribute!=observation.standard_deviation_attribute{return Err(contract("research observation original selection differs"));}
            let revision=package.selected_revision_roots(&[member.root,member.record],cancel).await?;
            let measured=revision.checked().measurement(member.root,member.record,&member.attribute,member.standard_deviation_attribute.as_deref()).map_err(|e|contract(e.to_string()))?;
            if observation.included&&(measured.value.is_none_or(|v|!v.is_finite())||measured.standard_deviation.is_none_or(|s|!s.is_finite()||s<=0.)||!observation.importance.is_finite()||observation.importance<=0.){return Err(contract("included research observation requires authored positive standard uncertainty and importance"));}
        }
        let row=ResearchSelection{key:request.key.clone(),project:request.project.key.clone(),dataset:request.dataset.clone(),revision:dataset.revision,physical_context:dataset.physical_context,context:dataset.context,fit:request.fit.as_id().to_string(),selection:encoded(request)?,request:Vec::new()};
        Ok(self.canonical_store()?.create_research_selection(&request.project,row).await?)
    }
    /// Reopen a frozen fit selection through its exact original context; prepare_fit remains the mathematical owner.
    pub async fn research_selection_open(&self,key:&str,cancel:&crate::CancelSource)->Result<(ModelingPackage,FitId),WorkflowError>{
        let row=self.canonical_store()?.research_selection(key).await?.ok_or_else(||contract("research selection unavailable"))?;
        let request:ResearchSelectionRequest=decoded(&row.selection)?;
        if request.key!=row.key||request.dataset!=row.dataset||request.context.context.key!=row.context||request.fit.as_id().to_string()!=row.fit{return Err(contract("research selection immutable coordinates differ"));}
        let mut package=self.research_dataset_open(&row.dataset,&request.context,cancel).await?;
        package.research_source=Some(ResearchSource{issuing_store:self.canonical_store()?.issuing_store()?.into(),selection:row.key,dataset:row.dataset,revision:row.revision,physical_context:row.physical_context,publication:None});
        Ok((package,request.fit))
    }
    /// Authenticate research attribution before recording actual scientific execution.
    pub(in crate::workflow) async fn restore_research_source(&self,package:&mut ModelingPackage,source:Option<&ResearchSource>)->Result<(),WorkflowError>{
        if let Some(source)=source {
            let store=self.canonical_store()?;
            let selection=store.research_selection(&source.selection).await?.ok_or_else(||contract("original research selection unavailable"))?;
            let revision=package.recorded_revision()?;
            if source.issuing_store!=store.issuing_store()? || source.dataset!=selection.dataset || source.physical_context!=selection.physical_context || source.physical_context!=package.physical.identity().to_string() || source.revision!=revision.key{return Err(contract("research operation source differs"));}
            if let Some(key)=&source.publication {
                let publication=store.research_publication(key).await?.ok_or_else(||contract("research publication unavailable"))?;
                if publication.selection!=source.selection||publication.revision!=source.revision{return Err(contract("research publication source differs"));}
            }else if selection.revision!=source.revision || package.project_context.as_ref().is_none_or(|context|context.context.key!=selection.context){return Err(contract("research original observation context differs"));}
            package.research_source=Some(source.clone());
        }
        Ok(())
    }

}
