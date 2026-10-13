// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read-only settlement of an original source publication occurrence.
use super::{Runtime, WorkflowError, contract};
use pse_model::generated::runtime::canonical_revisions::Row as Revision;

/// Actual immutable acknowledgement of the requested source operation.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceAcknowledgement {
    /// Exact retained revision accepted by the existing source-import API.
    pub revision: String,
    /// Original acknowledged canonical predecessor, if any.
    pub parent: Option<String>,
    /// Actual canonical occurrence sequence, not a local compiler generation.
    pub sequence: u64,
}
/// Receipt observation for one original operation, without retrying any mutation.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceSettlement {
    /// Original problem identity carried by the source publication or refusal.
    pub problem: String,
    /// Original operation identity, never replaced with a fresh issue.
    pub operation: String,
    /// Actual acknowledgement when observed. Absence remains unproved: it does
    /// not establish that publication was never issued or cannot still complete.
    pub acknowledgement: Option<SourceAcknowledgement>,
}
impl SourceSettlement {
    fn observed(problem: &str, operation: &str, receipt: Option<Revision>) -> Result<Self, WorkflowError> {
        let acknowledgement=receipt.map(|receipt| {
            if receipt.problem!=problem || receipt.operation!=operation || receipt.key!=operation
                || receipt.interpretation!=pse_operations::generated::surreal::INTERPRETATION {
                return Err(contract("source settlement acknowledgement does not match the original operation"));
            }
            Ok(SourceAcknowledgement {revision:receipt.key,parent:receipt.parent,sequence:receipt.sequence})
        }).transpose()?;
        Ok(Self {problem:problem.into(),operation:operation.into(),acknowledgement})
    }
}
impl Runtime {
    /// Read the immutable acknowledgement of an original source edit. This does
    /// not submit source bytes, activate a stage or run scientific work.
    /// # Errors
    /// No sink is configured, the actual receipt read fails, or its identity differs.
    pub async fn source_settlement(&self,problem:&str,operation:&str)->Result<SourceSettlement,WorkflowError> {
        self.scope_activity(async {
            if problem.is_empty() || operation.is_empty() {return Err(contract("source settlement requires the original problem and operation"));}
            let receipt=self.operations()?.store().revision(operation).await?;
            SourceSettlement::observed(problem,operation,receipt)
        }).await
    }
}

pub(super) fn source_edit_error(problem:String,operation:String,source:pse_operations::canonical::CanonicalError)->WorkflowError {
    use pse_operations::canonical::CanonicalError;
    if source.proves_source_refusal() {
        return WorkflowError::Canonical(source);
    }
    if matches!(source,CanonicalError::OperationReused) {
        return WorkflowError::Canonical(CanonicalError::SourceEditRefused {problem,operation,source:Box::new(source)});
    }
    WorkflowError::Canonical(CanonicalError::SourceEditUnsettled {problem,operation,source:Box::new(source)})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn receipt(problem:&str,operation:&str)->Revision {
        Revision {key:operation.into(),problem:problem.into(),sequence:7,parent:Some("original-parent".into()),operation:operation.into(),request:vec![1,2,3].into(),interpretation:pse_operations::generated::surreal::INTERPRETATION.into()}
    }
    #[test]
    fn original_source_receipt_is_acknowledged_and_absence_stays_unproved() {
        let observed=SourceSettlement::observed("problem","operation",Some(receipt("problem","operation"))).unwrap();
        assert_eq!(observed.acknowledgement.unwrap(),SourceAcknowledgement {revision:"operation".into(),parent:Some("original-parent".into()),sequence:7});
        let unknown=SourceSettlement::observed("problem","operation",None).unwrap();assert!(unknown.acknowledgement.is_none());
        assert!(SourceSettlement::observed("foreign","operation",Some(receipt("problem","operation"))).is_err());
        assert!(SourceSettlement::observed("problem","other",Some(receipt("problem","operation"))).is_err());
    }
    #[test]
    fn source_error_requires_phase_owned_refusal_proof() {
        use pse_operations::canonical::CanonicalError;
        let unknown=source_edit_error("problem".into(),"operation".into(),CanonicalError::Timeout);
        assert!(matches!(unknown,WorkflowError::Canonical(CanonicalError::SourceEditUnsettled {problem,operation,..}) if problem=="problem"&&operation=="operation"));
        let proved=CanonicalError::SourceEditRefused {problem:"problem".into(),operation:"operation".into(),source:Box::new(CanonicalError::OperationReused)};
        assert!(matches!(source_edit_error("problem".into(),"operation".into(),proved),WorkflowError::Canonical(CanonicalError::SourceEditRefused {..})));
    }
    #[cfg(feature="canonical-tests")]
    #[tokio::test]
    async fn explicit_recording_reads_do_not_make_ephemeral_compute_durable() {
        let local=crate::workflow::tests::runtime();
        let recording=local.with_canonical(crate::workflow::tests::canonical_deployment()).unwrap();
        assert!(matches!(recording.durability(),super::super::Durability::Ephemeral));
        assert!(recording.operations().is_ok());
        let source=recording.modeling_package(pse_authoring::language::parse("package p {}",pse_ids::SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap(),crate::workflow::tests::physical()).await.unwrap();
        assert!(source.canonical_revision().is_none());
        let operation=pse_operations::mint_id::<pse_ids::SemanticId>().to_string();
        let before=recording.source_settlement("settlement-control",&operation).await.unwrap();assert!(before.acknowledgement.is_none());
        let receipt=recording.canonical_store().unwrap().edit("settlement-control",None,&operation,&[]).await.unwrap();
        let observed=recording.source_settlement("settlement-control",&operation).await.unwrap();
        assert_eq!(observed.acknowledgement.unwrap().revision,receipt.key);
        let again=recording.source_settlement("settlement-control",&operation).await.unwrap();
        assert_eq!(again.acknowledgement.unwrap().sequence,receipt.sequence);
        recording.close();assert!(matches!(local.source_settlement("settlement-control",&operation).await,Err(WorkflowError::CallerClosed)));
    }
}
