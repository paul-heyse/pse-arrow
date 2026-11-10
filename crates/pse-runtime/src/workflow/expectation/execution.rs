// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original durable finite children and protected sealed-boundary reconciliation.
use super::*;
use crate::workflow::{
    IssuedStudyIdentity, ModelingPackage, Runtime, StudyDefinition, WorkflowError, contract,
};
use pse_model::generated::identities::StudyId;
use pse_operations::canonical_expectations::{Boundary, Expectation, Finalization, Proposal};
use serde::{Serialize, de::DeserializeOwned};

pub(super) const DOCUMENT_BYTES: usize = 16 << 20;
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedReport<T> {
    version: pse_model::document::Version<1>,
    report: T,
}
pub(super) fn encode_report(report: &ExpectationReport) -> Result<pse_model::Bytes, WorkflowError> {
    encode(&RecordedReport {
        version: Default::default(),
        report,
    })
}
pub(super) fn decode_report(bytes: &[u8]) -> Result<ExpectationReport, WorkflowError> {
    Ok(decode::<RecordedReport<ExpectationReport>>(bytes)?.report)
}
pub(super) fn report_document(report: &ExpectationReport) -> Result<String, WorkflowError> {
    String::from_utf8(encode_report(report)?.into_vec())
        .map_err(|error| contract(error.to_string()))
}
fn encode(value: &impl Serialize) -> Result<pse_model::Bytes, WorkflowError> {
    let bytes = serde_json::to_vec(value).map_err(|error| contract(error.to_string()))?;
    if bytes.is_empty() || bytes.len() > DOCUMENT_BYTES {
        return Err(contract("expectation original document extent"));
    }
    Ok(bytes.into())
}
pub(super) fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WorkflowError> {
    if bytes.is_empty() || bytes.len() > DOCUMENT_BYTES {
        return Err(contract("expectation retained document extent"));
    }
    serde_json::from_slice(bytes)
        .map_err(|error| contract(format!("expectation original document: {error}")))
}
fn child_id(value: &str) -> Result<StudyId, WorkflowError> {
    pse_ids::SemanticId::parse_hex(value)
        .map(StudyId::from_id)
        .map_err(|error| contract(error.to_string()))
}

impl Runtime {
    /// Execute or reconcile an original explicit-measure parent using immutable finite children.
    /// Resume reconstructs only a protected sealed checkpoint and reuses an acknowledged proposal.
    pub(in crate::workflow) async fn execute_expectation_science(
        &self,
        package: &ModelingPackage,
        parent: StudyId,
        request: ExpectationRequest,
        cancel: &crate::CancelSource,
    ) -> Result<ExpectationReport, WorkflowError> {
        let services = self.execution_services();
        let store = self.canonical_store()?;
        let preparation = services.preparation();
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                cancel
                    .token()
                    .checkpoint()
                    .map_err(pse_engine::EngineError::from)?;
                let admitted = package
                    .admit_expectation(preparation, request.clone(), cancel)
                    .await?;
                let supplier = pse_math::context()
                    .map_err(crate::workflow::math)?
                    .environment
                    .identity()
                    .to_string();
                let original = store
                    .create_expectation(Expectation {
                        key: parent.to_string(),
                        request: encode(&request)?,
                        supplier: supplier.clone(),
                        revision: request.source.revision.to_string(),
                        physical: request.source.physical_context.to_string(),
                    })
                    .await?;
                if original.request.as_slice() != encode(&request)?.as_slice()
                    || original.supplier != supplier
                {
                    return Err(contract(
                        "expectation original parent source or realization differs",
                    ));
                }
                if let Some(finalized) = store.expectation_finalization(&parent.to_string()).await?
                {
                    return decode_report(finalized.report.as_slice());
                }
                let latest = store.expectation_boundary(&parent.to_string()).await?;
                let mut last_boundary = latest.as_ref().map(|boundary| boundary.ordinal);
                let mut estimator = match latest {
                    Some(boundary) => {
                        let report = decode_report(boundary.report.as_slice())?;
                        let Some(checkpoint) = boundary.checkpoint else {
                            store
                                .finalize_expectation(Finalization {
                                    key: parent.to_string(),
                                    latest_boundary: last_boundary,
                                    report: encode_report(&report)?,
                                })
                                .await?;
                            return Ok(report);
                        };
                        let decoded = ExpectationCheckpoint::decode_original(
                            &admitted,
                            boundary.ordinal as usize + 1,
                            checkpoint.as_slice(),
                        )?;
                        let restored =
                            ExpectationParent::restore(admitted, parent, &supplier, decoded)?;
                        if !restored.has_next() {
                            store
                                .finalize_expectation(Finalization {
                                    key: parent.to_string(),
                                    latest_boundary: last_boundary,
                                    report: encode_report(&report)?,
                                })
                                .await?;
                            return Ok(report);
                        }
                        restored
                    }
                    None => ExpectationParent::new(admitted, parent, supplier.clone())?,
                };
                while estimator.has_next() {
                    if cancel.token().is_cancelled()
                        || cancel
                            .deadline()
                            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
                    {
                        let report = estimator.interrupt();
                        store
                            .finalize_expectation(Finalization {
                                key: parent.to_string(),
                                latest_boundary: last_boundary,
                                report: encode_report(&report)?,
                            })
                            .await?;
                        return Ok(report);
                    }
                    let ordinal = estimator.checkpoint()?.next_iteration() as u64;
                    let retained = store
                        .expectation_proposal(&parent.to_string(), ordinal)
                        .await?;
                    let (mut iteration, definition) = if let Some(proposal) = retained {
                        let child = child_id(&proposal.child)?;
                        let mut iteration = estimator.prepare_iteration(preparation, child)?;
                        // The recorded sampler outcome and transformed physical assignments are
                        // authority. This preparation only checks correspondence with the prior seal.
                        let samples: Vec<ExpectationSample> = decode(proposal.samples.as_slice())?;
                        iteration.samples = samples.into();
                        estimator.retain_transformed_iteration(iteration.clone())?;
                        (
                            iteration,
                            decode::<StudyDefinition>(proposal.definition.as_slice())?,
                        )
                    } else {
                        let child: StudyId = pse_operations::mint_id();
                        let (iteration, local) =
                            estimator.prepare_child(&services, child, cancel).await?;
                        let definition = self.record_study(&local, cancel).await?;
                        let proposal = Proposal {
                            key: format!("{parent}/{ordinal}"),
                            parent: parent.to_string(),
                            ordinal,
                            child: child.to_string(),
                            samples: encode(&iteration.samples)?,
                            definition: encode(&definition)?,
                        };
                        // A competing caller may win original proposal custody. No member has
                        // started here; refuse this caller rather than activate a different child.
                        store.retain_expectation_proposal(proposal).await?;
                        (iteration, definition)
                    };
                    let submission =
                        pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &definition)
                            .map_err(|error| contract(error.to_string()))?;
                    let issued = IssuedStudyIdentity {
                        study: iteration.child,
                        operation: format!("expectation-{parent}-{ordinal}"),
                        submission,
                    };
                    let report = self
                        .execute_study_with_issuance(
                            &definition,
                            crate::workflow::MAXIMUM_STUDY_POINTS,
                            cancel,
                            Some(issued),
                        )
                        .await?;
                    // Cancellation already settled the original child. Independent retained
                    // read/publication work must preserve its incomplete evidence.
                    let drain = crate::CancelSource::new();
                    let observed = iteration
                        .read_observations(estimator.admitted(), &report, &drain)
                        .await?;
                    let result = estimator.commit_iteration(iteration.child, &observed)?;
                    let checkpoint = estimator
                        .checkpoint()
                        .ok()
                        .map(|checkpoint| encode(&checkpoint))
                        .transpose()?;
                    store
                        .seal_expectation_boundary(Boundary {
                            key: format!("{parent}/{ordinal}"),
                            parent: parent.to_string(),
                            ordinal,
                            child: iteration.child.to_string(),
                            checkpoint,
                            report: encode_report(&result)?,
                            observations: encode(&observed)?,
                        })
                        .await?;
                    last_boundary = Some(ordinal);
                    iteration.samples = std::sync::Arc::from([]);
                }
                let report = estimator.report();
                store
                    .finalize_expectation(Finalization {
                        key: parent.to_string(),
                        latest_boundary: last_boundary,
                        report: encode_report(&report)?,
                    })
                    .await?;
                Ok(report)
            })
            .await
    }
    /// Execute or reconcile and publish through the ordinary run/attempt result owner.
    pub async fn execute_expectation(
        &self,
        package: &ModelingPackage,
        parent: StudyId,
        request: ExpectationRequest,
        cancel: &crate::CancelSource,
    ) -> Result<RetainedExpectationReport, WorkflowError> {
        request.source.check(package)?;
        package.check_service_bindings(&self.preparation_services())?;
        if self
            .canonical_store()?
            .expectation_publication(&parent.to_string())
            .await?
            .is_some()
        {
            let original = self
                .canonical_store()?
                .expectation(&parent.to_string())
                .await?
                .ok_or_else(|| contract("expectation original request absent"))?;
            if original.request.as_slice() != encode(&request)?.as_slice() {
                return Err(contract("expectation original request differs"));
            }
            return self.expectation_report(parent).await;
        }
        let prepared = PreparedExpectationOperation {
            source: package.clone(),
            request,
            parent,
            runtime: self.clone(),
        };
        let services = self.execution_services();
        let handle = prepared.start(&services, cancel)?;
        let result = handle.wait().await?;
        if let Some(error) = result.error() {
            return Err(WorkflowError::Shared(error.clone()));
        }
        let crate::workflow::RunDurability::Durable(record) = result.durability() else {
            return Err(WorkflowError::RecordingUnavailable);
        };
        record
            .attempt
            .as_ref()
            .map_err(|error| WorkflowError::Shared(error.clone()))?;
        let manifest = record
            .manifest
            .as_ref()
            .ok_or_else(|| contract("expectation original result manifest absent"))?;
        let run = record
            .run
            .as_ref()
            .ok_or_else(|| contract("expectation original run absent"))?;
        let attempt = record
            .attempt_key
            .as_ref()
            .ok_or_else(|| contract("expectation original attempt absent"))?;
        self.canonical_store()?
            .publish_expectation(pse_operations::canonical_expectations::Publication {
                key: parent.to_string(),
                run: run.key.to_string(),
                attempt: attempt.clone(),
                manifest: manifest.key.clone(),
            })
            .await?;
        self.expectation_report(parent).await
    }
}
