// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public bounded study browsing and immutable cohort analysis over protected original results.
use super::{Runtime,WorkflowError,contract};
use pse_columnar::{CancellationToken,MemoryConsumer,MemoryReservation};
use std::sync::Arc;
pub use pse_operations::canonical_study_workbench::{StudyWorkbenchFilter,StudyWorkbenchHeader,StudyCohortMember};

/// Authority-bound continuation for live browsing, never a frozen analysis population.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchCursor {
    /// Canonical issuing database identity.
    pub issuing_store:String,
    /// Exact original predicates.
    pub filter:StudyWorkbenchFilter,
    /// Last displayed study key.
    pub after:String,
}
/// Bounded live header selection; continuation pages can observe newer metadata.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchRequest {
    /// Exact scientific and engineering predicates.
    pub filter:StudyWorkbenchFilter,
    /// Maximum headers, from one through 64.
    pub maximum:usize,
    /// Issuer- and predicate-bound continuation.
    pub cursor:Option<StudyWorkbenchCursor>,
}
/// Live browsing page, explicitly independent from frozen completeness.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchPage {
    /// Compact actual metadata; payloads are unopened.
    pub headers:Vec<StudyWorkbenchHeader>,
    /// More headers under this predicate, if present.
    pub next:Option<StudyWorkbenchCursor>,
}
/// One complete bounded population selected before any analytical payload work.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortRequest {
    /// Exact native predicates.
    pub filter:StudyWorkbenchFilter,
    /// Capture refuses if more studies match; maximum 64.
    pub maximum_studies:usize,
    /// Capture refuses if complete selected population exceeds this; maximum 128.
    pub maximum_members:usize,
}
/// Copied facts about a successful native snapshot. This document grants no payload permission.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortDocument {
    /// Exact issuing canonical database.
    pub issuing_store:String,
    /// Original bounded selection.
    pub request:StudyCohortRequest,
    /// Exact original headers.
    pub headers:Vec<StudyWorkbenchHeader>,
    /// Every selected occurrence, including missing, failed, cancelled and unattempted members.
    pub members:Vec<StudyCohortMember>,
    /// True only after the single native complete selection succeeded.
    pub complete:bool,
}
/// Nonserializable retained cohort capability; analysis never refetches floating membership.
#[derive(Clone,Debug)]
pub struct FrozenStudyCohort {
    runtime:Runtime,
    document:StudyCohortDocument,
    _owner:Arc<MemoryReservation>,
}
/// One original scientific output, identically selected across the frozen population.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyKpiRequest {
    /// Existing registered result relation.
    pub relation:String,
    /// Exact authored scientific output owner.
    pub owner:pse_ids::SemanticId,
    /// Existing declared numeric output field.
    pub field:String,
    /// Existing exact output partition.
    pub partition:String,
    /// Inclusive original relation row coordinate.
    pub start:u64,
    /// Exclusive original relation row coordinate; no implicit all-trajectories request.
    pub end:u64,
}
/// DataFusion aggregates plus the complete captured population. No optimum claim is made.
#[derive(Debug)]
pub struct StudyKpiAnalysis {
    /// Complete population and original outcomes, including unavailable members.
    pub cohort:StudyCohortDocument,
    /// Members whose original operation permits usable-result analysis.
    pub included:Vec<usize>,
    /// Explicit remaining members; absence is never a zero numerical observation.
    pub unavailable:Vec<usize>,
    /// Selected native Arrow aggregation: count, minimum, maximum and mean.
    pub batches:Vec<datafusion::arrow::record_batch::RecordBatch>,
}
impl Runtime {
    /// Browse bounded compact headers; no occurrence or result payload is read.
    pub async fn study_workbench_page(&self,request:StudyWorkbenchRequest)->Result<StudyWorkbenchPage,WorkflowError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            let issuer=self.canonical_store()?.issuing_store()?.to_owned();
            if request.cursor.as_ref().is_some_and(|c|c.issuing_store!=issuer||c.filter!=request.filter) {return Err(contract("workbench cursor issuer or predicates differ"));}
            let _response=super::connected_results::response_owner(self,"study-workbench:headers",pse_operations::canonical::MetadataResponse::Page)?;
            let page=pse_operations::canonical::response_ownership_scope(_response,self.canonical_store()?.select_study_workbench(&request.filter,request.cursor.as_ref().map(|c|c.after.as_str()),request.maximum,None)).await?;
            let next=if page.has_more {page.headers.last().map(|h|StudyWorkbenchCursor{issuing_store:issuer,filter:request.filter,after:h.study.clone()})}else{None};
            Ok(StudyWorkbenchPage{headers:page.headers,next})
        }).await
    }
    /// Freeze every member and actual metadata revision in one native SSI snapshot.
    pub async fn freeze_study_cohort(&self,request:StudyCohortRequest)->Result<FrozenStudyCohort,WorkflowError> {
        let activity=self.admit_activity()?;
        activity.scope(async {
            let owner=Arc::new(MemoryConsumer::new("study-workbench:frozen-cohort").register(&self.shared.pool()));
            owner.try_grow(2*1024*1024).map_err(pse_engine::EngineError::from)?;
            let page=pse_operations::canonical::response_ownership_scope(owner.clone(),self.canonical_store()?.select_study_workbench(&request.filter,None,request.maximum_studies,Some(request.maximum_members))).await?;
            Ok(FrozenStudyCohort{runtime:self.clone(),document:StudyCohortDocument{issuing_store:self.canonical_store()?.issuing_store()?.into(),request,headers:page.headers,members:page.members,complete:true},_owner:owner})
        }).await
    }
}
impl FrozenStudyCohort {
    /// Exact captured facts; borrowing them does not acquire result lifetime.
    pub fn document(&self)->&StudyCohortDocument {&self.document}
    /// Lazy detail from one exact captured occurrence. Original protection/refusal remains active.
    pub async fn result(&self,member:usize,relation:&str,start:u64,end:u64,cancel:CancellationToken)->Result<super::CanonicalResultReader,WorkflowError> {
        let point=self.document.members.get(member).ok_or_else(||contract("cohort member absent"))?;
        let attempt=point.attempt.as_deref().ok_or_else(||contract("cohort member has no attempted result"))?;
        self.runtime.results(&point.run,attempt,relation,start,end,cancel).await
    }
    /// Aggregate one selected output via Arrow/DataFusion across compatible exact sources.
    /// Differing source/physical meanings require an admitted transform outside this operation.
    pub async fn aggregate(&self,request:&StudyKpiRequest,cancel:CancellationToken)->Result<StudyKpiAnalysis,WorkflowError> {
        use datafusion::{prelude::{SessionContext,col},functions_aggregate::expr_fn::{count,min,max,avg}};
        compatible(&self.document.headers)?;
        if request.start>request.end {return Err(contract("KPI row range reversed"));}
        let spec=self.runtime.registry().relation(&request.relation).ok_or_else(||contract("unknown KPI relation"))?;
        if !spec.columns.iter().any(|c|c.name()=="quantity_id") || !spec.columns.iter().any(|c|c.name()=="unit_id") {
            return Err(contract("KPI relation does not retain complete quantity and unit meaning; use an original physically typed output"));
        }
        let mut dataframe=None::<datafusion::dataframe::DataFrame>;
        let mut included=Vec::new(); let mut unavailable=Vec::new();
        for (index,member) in self.document.members.iter().enumerate() {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            if !member.facts.scientific.usable || member.attempt.is_none() {unavailable.push(index);continue;}
            let attempt=member.attempt.as_deref().ok_or_else(||contract("selected KPI attempt absent"))?;
            let reader=self.runtime.output_results(&member.run,attempt,&request.relation,request.owner,&request.field,&request.partition,request.start,request.end,None,None,None,cancel.clone()).await?;
            let snapshot=reader.selected_snapshot(cancel.clone()).await?;
            if snapshot.row_count()==0 {unavailable.push(index);continue;}
            let context=SessionContext::new_with_state(snapshot.session_state());
            let selected=context.read_table(snapshot.table_provider()).map_err(pse_engine::EngineError::from)?.select(vec![col("quantity_id"),col("unit_id"),col(&request.field)]).map_err(pse_engine::EngineError::from)?;
            dataframe=Some(match dataframe {None=>selected,Some(previous)=>previous.union(selected).map_err(pse_engine::EngineError::from)?});
            included.push(index);
        }
        let batches=if let Some(frame)=dataframe {
            let query=frame.aggregate(vec![col("quantity_id"),col("unit_id")],vec![count(col(&request.field)).alias("count"),min(col(&request.field)).alias("minimum"),max(col(&request.field)).alias("maximum"),avg(col(&request.field)).alias("mean")]).map_err(pse_engine::EngineError::from)?;
            cancel.until_cancelled(query.collect()).await.map_err(pse_engine::EngineError::from)?.map_err(pse_engine::EngineError::from)?
        }else{Vec::new()};
        if batches.iter().map(|b|b.num_rows()).sum::<usize>()>1 {return Err(contract("KPI outputs have incompatible actual quantity, basis, reference state or units"));}
        Ok(StudyKpiAnalysis{cohort:self.document.clone(),included,unavailable,batches})
    }
}
fn compatible(headers:&[StudyWorkbenchHeader])->Result<(),WorkflowError> {
    if let Some(first)=headers.first() {
        if first.physical_revision.is_none() || first.physical_identity.is_none() || headers.iter().any(|h|h.physical_revision!=first.physical_revision||h.physical_identity!=first.physical_identity) {
            return Err(contract("KPI source or physical meaning differs or is unavailable; select compatible exact sources or an admitted transform"));
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workbench_refuses_incompatible_kpi_meaning_before_payload_access() {
        let mut h:StudyWorkbenchHeader=serde_json::from_value(serde_json::json!({"study":"s","run":"r","problem":"p","source":"revision","physical_revision":"physical","physical_identity":"identity","context":null,"point_count":3,"generation":0,"active":true,"cancelled":false,"terminal":false,"result_attempt":null,"outcome":null})).unwrap();
        assert!(compatible(&[h.clone(),h.clone()]).is_ok());
        h.physical_revision=Some("other-unit-system".into());
        let mut original=h.clone(); original.physical_revision=Some("physical".into());
        assert!(compatible(&[original,h]).is_err());
    }
}
