// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Engineering history selects ordinary admitted sources and freezes original provenance.
use super::{ModelingPackage,PhysicalContext,Runtime,WorkflowError,contract};
pub use pse_operations::canonical_contexts::{ContextCut,ContextSelection,ProjectContext};
use std::collections::BTreeMap;
impl Runtime {
    /// Load the exact original source through the existing source owner. This never selects latest.
    pub async fn project_context_package(&self,selection:&ContextSelection,physical:PhysicalContext,cancel:&crate::CancelSource)->Result<ModelingPackage,WorkflowError>{
        let deadline = cancel.deadline().unwrap_or_else(|| std::time::Instant::now()+pse_operations::canonical::REQUEST_TIMEOUT);
        self.scope_activity(pse_operations::canonical::operation_scope(deadline, async {
            cancel.token().checkpoint().map_err(pse_relations::RelationError::from).map_err(super::relation)?;
            let store=self.canonical_store()?;
            store.validate_project_context(selection).await?;
            if physical.identity().to_string()!=selection.context.physical_context{return Err(contract("historical context physical closure differs"));}
            let revision=store.revision(&selection.context.revision).await?.ok_or_else(||contract("historical context source unavailable"))?;
            let mut package=self.modeling_revision(revision,physical,BTreeMap::new()).await?;
            if package.revision.identity().to_string()!=selection.context.source_identity{return Err(contract("historical context authored source identity differs"));}
            package.project_context=Some(selection.clone());
            Ok(package)
        })).await
    }
    /// Pin a selected admitted package into project history, without copying scientific numbers.
    #[allow(clippy::too_many_arguments,reason="explicit immutable effective interval, correction ancestry and original operation")]
    pub async fn project_record_context(&self,operation:&str,project:&pse_operations::canonical_projects::Project,scope:&str,package:&ModelingPackage,effective_from_us:i64,effective_to_us:Option<i64>,corrects:Option<&str>,deleted:bool)->Result<ProjectContext,WorkflowError>{
        let deadline = std::time::Instant::now()+pse_operations::canonical::REQUEST_TIMEOUT;
        self.scope_activity(pse_operations::canonical::operation_scope(deadline, async {
            if package.runtime.canonical_store()?.issuing_store()? != self.canonical_store()?.issuing_store()? {return Err(contract("context source issuing authority differs"));}
            let revision=package.canonical_revision().ok_or_else(||contract("context requires a durable admitted source revision"))?;
            self.canonical_store()?.append_project_context(operation,project,scope,&revision.key,&package.revision.identity().to_string(),&package.physical.identity().to_string(),effective_from_us,effective_to_us,corrects,deleted).await.map_err(Into::into)
        })).await
    }
    pub(in crate::workflow) async fn restore_project_context(&self,package:&mut ModelingPackage,selection:Option<&ContextSelection>)->Result<(),WorkflowError>{
        if let Some(selection)=selection {
            self.canonical_store()?.validate_project_context(selection).await?;
            if package.canonical_revision().map(|r|r.key.as_str())!=Some(selection.context.revision.as_str())||package.physical.identity().to_string()!=selection.context.physical_context||package.revision.identity().to_string()!=selection.context.source_identity{return Err(contract("recorded context and actual source closure differ"));}
            package.project_context=Some(selection.clone());
        }
        Ok(())
    }
}
impl ModelingPackage {
    /// Exact context selected before editing/preparation. Unrelated authored edits clear it.
    pub fn project_context(&self)->Option<&ContextSelection>{self.project_context.as_ref()}
}
