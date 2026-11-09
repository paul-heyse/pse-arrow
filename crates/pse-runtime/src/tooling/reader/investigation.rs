// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit diagnostic effects and supplied-evidence reads through their existing owners.
use super::*;
use crate::tooling::{capture, frozen_inspection, review_analysis, review_pack, review_query};
use std::{num::NonZeroUsize, sync::Arc, time::Duration};

/// Explicit investigation paths; embedded evidence paths never authorize adjacent reads.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    /// Attach the existing bounded collector and join its owned workers before answering.
    Capture {
        /// Exact existing capture owner request; no scientific cancellation is acquired.
        request: capture::CaptureRequest,
    },
    /// Interpret exactly one supplied immutable evidence set.
    FrozenInspect {
        /// Explicit evidence-set v3 file; paths inside it are descriptive.
        evidence: PathBuf,
        /// Original recorded-time inspection selection.
        request: frozen_inspection::FrozenInspectionRequest,
    },
    /// Run the existing bounded evidence interpreter under an observer-owned allocation.
    Analyze {
        /// Explicit evidence-set v3 file.
        evidence: PathBuf,
        /// Original maintained question/threshold/window selection.
        request: review_analysis::AnalysisRequest,
        /// Shared observer working allocation, not a host workload placement cap.
        memory_bytes: NonZeroUsize,
    },
    /// Join/group explicitly supplied original developer report members without native actions.
    DeveloperAnalyze {
        /// Existing host-verified passive facts; scientific supplied membership stays separate.
        request: review_query::developer::Request,
    },
    /// Associate captures to original scientific invocations in one supplied immutable image.
    ScientificCaptureAnalyze {
        /// Exact evidence image and observer allocation; no native or scientific actions.
        request: review_query::scientific_capture::Request,
    },
    /// Return maintained SQL and declared columns without executing any external tool.
    QuerySpecification {
        /// Original maintained query selection and finite allowances.
        request: review_query::ReviewQueryRequest,
    },
    /// Export selected supplied facts and their analysis into a portable review pack.
    Export {
        /// Explicit evidence-set v3 file; embedded member paths are never followed.
        evidence: PathBuf,
        /// Original selected analysis request.
        request: review_analysis::AnalysisRequest,
        /// Observer-owned analysis allocation retained through publication.
        memory_bytes: NonZeroUsize,
        /// New output directory; existing destinations refuse.
        destination: PathBuf,
        /// Original explicit membership and publication allowances.
        selection: review_pack::PackSelection,
        /// Exact optional file prefixes authorized by the caller.
        files: Vec<review_pack::PackFileInput>,
    },
}
impl Request {
    pub(super) fn has_effects(&self) -> bool {
        matches!(self, Self::Capture { .. } | Self::Export { .. })
    }
}
fn serialize_analysis<S: serde::Serializer>(
    analysis: &review_analysis::Analysis,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    analysis.document().serialize(serializer)
}
fn serialize_developer_analysis<S: serde::Serializer>(
    analysis: &review_query::developer::Analysis,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    analysis.document().serialize(serializer)
}
fn serialize_scientific_capture_analysis<S: serde::Serializer>(
    analysis: &review_query::scientific_capture::Analysis,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    analysis.document().serialize(serializer)
}
/// Existing typed owner results. Analysis retains its admitted allocation through encoding.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ResultView {
    /// Explicit collector's terminal document, independent of scientific outcome.
    Capture {
        /// Original capture scope, prefixes, losses and collector errors.
        document: capture::CaptureDocument,
    },
    /// Recorded inspection without live freshness or result permission.
    FrozenInspect {
        /// Original recorded source interpretation and limitations.
        document: Box<frozen_inspection::FrozenInspectionDocument>,
    },
    /// Original source-linked questions and evidence strengths.
    Analyze {
        /// Protected admitted analysis, serialized directly from its owner's document.
        #[serde(serialize_with = "serialize_analysis")]
        #[schemars(with = "review_analysis::AnalysisDocument")]
        analysis: review_analysis::Analysis,
    },
    /// Maintained original developer diagnostic/capture/recorded-settlement pattern query.
    DeveloperAnalyze {
        /// Typed ephemeral answer retains its observer allocation through response encoding.
        #[serde(serialize_with = "serialize_developer_analysis")]
        #[schemars(with = "review_query::developer::Document")]
        analysis: review_query::developer::Analysis,
    },
    /// Maintained exact operation association with capture and invocation facts kept separate.
    ScientificCaptureAnalyze {
        /// Typed ephemeral answer retains its observer allocation through response encoding.
        #[serde(serialize_with = "serialize_scientific_capture_analysis")]
        #[schemars(with = "review_query::scientific_capture::Document")]
        analysis: review_query::scientific_capture::Analysis,
    },
    /// Specification only; callers cannot infer external-query execution.
    QuerySpecification {
        /// Maintained exact SQL and declared interpretation.
        specification: review_query::ReviewQuerySpecification,
        /// Always false for this pure specification route.
        executed: bool,
    },
    /// Original selected review-pack publication, including missing/failed members.
    Export {
        /// Explicit destination chosen by this request.
        destination: PathBuf,
        /// Original sealed publication scope and member outcomes.
        index: review_pack::PackIndex,
    },
}
pub(super) fn validate(request: &Request) -> Result<(), ReaderError> {
    match request {
        Request::Capture { request } => {
            absolute(&request.checkpoint)?;
            absolute(&request.output)?;
            if request.duration.is_zero() || request.duration > Duration::from_secs(20) {
                return Err(invalid(
                    "delegated capture requires a positive duration no greater than 20 seconds",
                ));
            }
        }
        Request::FrozenInspect { evidence, .. } | Request::Analyze { evidence, .. } => {
            absolute(evidence)?
        }
        Request::QuerySpecification { request } => {
            review_query::specification(request).map_err(ReaderError::Identity)?;
        }
        Request::DeveloperAnalyze { request } => review_query::developer::validate(request)?,
        Request::ScientificCaptureAnalyze { request } => {
            review_query::scientific_capture::validate(request)?
        }
        Request::Export {
            evidence,
            destination,
            selection,
            files,
            ..
        } => {
            absolute(evidence)?;
            absolute(destination)?;
            if selection
                .members
                .iter()
                .any(|member| member.result.is_some())
            {
                return Err(invalid(
                    "supplied-evidence export cannot grant protected scientific result access",
                ));
            }
            for file in files {
                absolute(&file.path)?;
            }
        }
    }
    Ok(())
}
fn evidence(path: &Path) -> Result<frozen_inspection::FrozenEvidenceSet, ReaderError> {
    let evidence: frozen_inspection::FrozenEvidenceSet =
        frozen_inspection::read_document::<_, 3>(path).map_err(ReaderError::Identity)?;
    evidence.validate().map_err(ReaderError::Identity)?;
    Ok(evidence)
}
fn analyze(
    evidence: &frozen_inspection::FrozenEvidenceSet,
    request: &review_analysis::AnalysisRequest,
    memory: NonZeroUsize,
    cancel: &pse_columnar::CancellationToken,
) -> Result<review_analysis::Analysis, ReaderError> {
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(memory.get()));
    review_analysis::analyze(evidence, request, &pool, cancel).map_err(ReaderError::Identity)
}
fn collect(
    request: capture::CaptureRequest,
    cancel: &pse_columnar::CancellationToken,
) -> Result<capture::CaptureDocument, ReaderError> {
    // The process root supplies stop. This owner joins only its collector before returning;
    // no public library call installs process-global signal policy.
    cancel.checkpoint()?;
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    executor.block_on(async {
        let session = capture::start_capture(request)?;
        while !session.is_finished() {
            tokio::select! {
                _ = cancel.cancelled() => return Ok(session.stop()?),
                _ = tokio::time::sleep(Duration::from_millis(25)) => {},
            }
        }
        Ok(session.wait()?)
    })
}
pub(super) fn handle(
    request: Request,
    cancel: &pse_columnar::CancellationToken,
) -> Result<Payload, ReaderError> {
    cancel.checkpoint()?;
    let result = match request {
        Request::Capture { request } => ResultView::Capture {
            document: collect(request, cancel)?,
        },
        Request::FrozenInspect {
            evidence: path,
            request,
        } => {
            let evidence = evidence(&path)?;
            ResultView::FrozenInspect {
                document: Box::new(
                    frozen_inspection::inspect_frozen(&evidence, request)
                        .map_err(ReaderError::Identity)?,
                ),
            }
        }
        Request::Analyze {
            evidence: path,
            request,
            memory_bytes,
        } => {
            let evidence = evidence(&path)?;
            ResultView::Analyze {
                analysis: analyze(&evidence, &request, memory_bytes, cancel)?,
            }
        }
        Request::DeveloperAnalyze { request } => {
            let executor = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            ResultView::DeveloperAnalyze {
                analysis: executor.block_on(review_query::developer::analyze(request, cancel))?,
            }
        }
        Request::ScientificCaptureAnalyze { request } => {
            let executor = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            ResultView::ScientificCaptureAnalyze {
                analysis: executor
                    .block_on(review_query::scientific_capture::analyze(request, cancel))?,
            }
        }
        Request::QuerySpecification { request } => ResultView::QuerySpecification {
            specification: review_query::specification(&request).map_err(ReaderError::Identity)?,
            executed: false,
        },
        Request::Export {
            evidence: path,
            request,
            memory_bytes,
            destination,
            selection,
            files,
        } => {
            let evidence = evidence(&path)?;
            let analysis = analyze(&evidence, &request, memory_bytes, cancel)?;
            let mut pack =
                review_pack::PackSession::create(&destination, selection, cancel.clone())?;
            pack.add_frozen(&evidence)?;
            pack.add_analysis(analysis.document())?;
            pack.copy_files(&files)?;
            ResultView::Export {
                destination,
                index: pack.seal()?,
            }
        }
    };
    Ok(Payload::Investigation { result })
}
