// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Rust-owned passive inspection and contextual explanations over copied observations.

use crate::observation::{
    Diagnostics, ObservationError, ProcessCheckpoint, ProcessLocator, read_checkpoint,
};
use pse_diagnostics::{
    inspection::{NextObservation, Statement, StatementKind},
    observation::{OperationId, OperationSnapshot, OperationSummary, Value},
};
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

fn default_window() -> Duration {
    Duration::from_secs(60)
}
fn default_related() -> bool {
    true
}
fn default_limit() -> usize {
    16
}
/// Request for copied diagnostics only; it cannot request scientific work or result admission.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionRequest {
    /// Standalone diagnostic request encoding.
    pub version: Version<2>,
    /// Exact operation selected by its diagnostic process instance and sequence.
    pub operation: OperationId,
    /// Requested recent interval, clamped to observed operation lifetime.
    #[serde(default = "default_window")]
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub window: Duration,
    /// Copy bounded directly related operations; deeper navigation selects their exact IDs.
    #[serde(default = "default_related")]
    pub include_related: bool,
    /// Maximum directly related snapshots; effective ceiling is 32.
    #[serde(default = "default_limit")]
    pub related_limit: usize,
    /// Explicit selected analytical thresholds; absent means only common factual interpretations.
    #[serde(default)]
    pub numerical_history_policy: Option<pse_diagnostics::inspection::NumericalHistoryPolicy>,
}
impl InspectionRequest {
    /// Ordinary recent request with direct related navigation and no expensive capture.
    pub fn new(operation: OperationId) -> Self {
        Self {
            version: Version,
            operation,
            window: default_window(),
            include_related: true,
            related_limit: 16,
            numerical_history_policy: None,
        }
    }
}
/// Checkpoint transport availability, distinct from producer freshness and scientific outcomes.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionTransport {
    /// Actual file read, absent for an in-memory read.
    pub checkpoint: Option<PathBuf>,
    /// Actual process attachment locator, if supplied by the source.
    pub process: Option<ProcessLocator>,
    /// Optional publisher timestamp; it is not native fact freshness.
    pub published_unix_micros: Option<u128>,
    /// Optional writer or transport failure; it never becomes numerical failure.
    pub failure: Option<String>,
    /// Required operation or relationship was absent/curtailed/stale in transport.
    pub partial: bool,
    /// Cross-process clocks are not an atomic observation; alignment may be unavailable.
    pub clock_uncertainty_nanoseconds: Option<u64>,
}
/// One typed interpretation shared by Rust, generated Python and CLI consumers.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectionDocument {
    /// Standalone diagnostic response encoding with explicit producer/record references.
    pub version: Version<3>,
    /// Original bounded read request.
    pub request: InspectionRequest,
    /// Exact selected operation, absent when unavailable/evicted rather than invented complete.
    pub operation: Option<OperationSnapshot>,
    /// Bounded directly related observations, with their independent clocks and outcomes.
    pub related: Vec<OperationSnapshot>,
    /// Established explanations, hypotheses and their source-linked next observations.
    pub statements: Vec<Statement>,
    /// Statements or requested children omitted by response bounds.
    pub omitted: usize,
    /// Independent transport/collector explanation.
    pub transport: InspectionTransport,
}
/// Discovery request; only matching page images are decoded, never scientific store members.
#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticListRequest {
    /// Exclusive checkpoint filename cursor; not a durable catalogue generation.
    pub after: Option<String>,
    /// Requested page size, default 16 and effective ceiling 32.
    #[serde(default = "default_limit")]
    pub limit: usize,
}
/// One discovered process prefix; errors retain the path without implying an empty process.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticProcessSummary {
    /// Explicit producer-summary admission, expiry and pressure losses when readable.
    pub summaries: Option<pse_diagnostics::observation::SummaryCoverage>,
    /// Actual checkpoint path.
    pub checkpoint: PathBuf,
    /// Process diagnostic identity, if the image is readable.
    pub process: Option<String>,
    /// Actual operational locator, if readable.
    pub locator: Option<ProcessLocator>,
    /// Bounded available operation catalogue from this image.
    pub operations: Vec<OperationSummary>,
    /// Optional decode/access error, independent of simulation success.
    pub failure: Option<String>,
    /// Image omitted observations or registry collection was curtailed.
    pub partial: bool,
}
/// Bounded checkpoint discovery result.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticProcessPage {
    /// Standalone diagnostic page encoding.
    pub version: Version<1>,
    /// The selected readable/unreadable process prefixes.
    pub processes: Vec<DiagnosticProcessSummary>,
    /// Exclusive filename cursor when another page may exist.
    pub next: Option<String>,
}
/// Default diagnostic directory shared with process publication; no registry is initialized.
pub fn default_directory() -> PathBuf {
    std::env::var_os("PSE_DIAGNOSTICS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("XDG_STATE_HOME")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
                })
                .unwrap_or_else(std::env::temp_dir)
                .join("pse/diagnostics")
        })
}
/// Discover one bounded page without constructing Runtime, opening a store or executing work.
pub fn list_processes(
    directory: &Path,
    request: DiagnosticListRequest,
) -> Result<DiagnosticProcessPage, ObservationError> {
    let limit = if request.limit == 0 {
        16
    } else {
        request.limit.min(32)
    };
    let mut paths = std::collections::BTreeSet::new();
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DiagnosticProcessPage {
                version: Version,
                processes: Vec::new(),
                next: None,
            });
        }
        Err(error) => return Err(error.into()),
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if request
            .after
            .as_ref()
            .is_some_and(|after| name <= after.as_str())
        {
            continue;
        }
        paths.insert(path);
        if paths.len() > limit + 1 {
            paths.pop_last();
        }
    }
    let more = paths.len() > limit;
    if more {
        paths.pop_last();
    }
    let next = more
        .then(|| {
            paths
                .last()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
        })
        .flatten();
    let processes = paths
        .into_iter()
        .map(|checkpoint| match read_checkpoint(&checkpoint) {
            Ok(image) => DiagnosticProcessSummary {
                summaries: Some(image.summaries),
                checkpoint,
                process: Some(image.process),
                locator: Some(image.locator),
                operations: image
                    .operations
                    .into_iter()
                    .map(|op| OperationSummary {
                        summary: op.summary,
                        id: op.id,
                        kind: op.kind,
                        started_at: op.started_at,
                        finished_at: op.finished_at,
                        outcome: op.outcome,
                    })
                    .collect(),
                failure: None,
                partial: image.omitted > 0
                    || image.registry_coverage.curtailed
                    || !image.registry_coverage.available,
            },
            Err(error) => DiagnosticProcessSummary {
                summaries: None,
                checkpoint,
                process: None,
                locator: None,
                operations: Vec::new(),
                failure: Some(error.to_string()),
                partial: true,
            },
        })
        .collect();
    Ok(DiagnosticProcessPage {
        version: Version,
        processes,
        next,
    })
}
impl Diagnostics {
    /// Inspect without borrowing a scientific caller, cancellation owner or native actor.
    pub fn inspect(&self, request: InspectionRequest) -> InspectionDocument {
        let operation = self
            .reader(request.operation)
            .map(|reader| reader.snapshot(request.window));
        let mut related = Vec::new();
        let mut omitted = 0;
        if request.include_related
            && let Some(operation) = &operation
        {
            for child in &operation.children {
                if related.len() >= request.related_limit.min(32) {
                    omitted += 1;
                    continue;
                }
                match self.reader(child.id) {
                    Some(reader) => related.push(reader.snapshot(request.window)),
                    None => omitted += 1,
                }
            }
        }
        let transport = InspectionTransport {
            checkpoint: None,
            process: Some(self.locator()),
            published_unix_micros: None,
            failure: self.writer_failure().or_else(|| {
                (!self.hub().policy().enabled).then(|| "diagnostic collection disabled".into())
            }),
            partial: operation.is_none() || omitted > 0,
            clock_uncertainty_nanoseconds: self
                .clock_mapping()
                .map(|mapping| mapping.uncertainty_nanoseconds),
        };
        document(request, operation, related, omitted, transport)
    }
}
/// Read one actual checkpoint and bounded known remote links, never querying a busy owner.
pub fn inspect_checkpoint(
    path: &Path,
    request: InspectionRequest,
) -> Result<InspectionDocument, ObservationError> {
    let image = read_checkpoint(path)?;
    Ok(inspect_image(path, request, image))
}
/// Inspect this already decoded image without rereading its possibly replaced path.
pub fn inspect_image(
    path: &Path,
    request: InspectionRequest,
    mut image: ProcessCheckpoint,
) -> InspectionDocument {
    let now = image
        .clock
        .as_ref()
        .and_then(|mapping| mapping.observation_now());
    let operation = image
        .operations
        .iter()
        .position(|op| op.id == request.operation)
        .map(|index| {
            let mut op = image.operations.remove(index);
            if now.is_none()
                && op
                    .summary
                    .as_ref()
                    .is_some_and(|summary| summary.expires_at.is_some())
            {
                // Without current clock projection, terminal-summary eligibility
                // cannot be established from the old publication timestamp.
                op.coverage.stale = true;
            }
            let read_at = now.unwrap_or(op.read_at);
            pse_diagnostics::inspection::select_window(op, read_at, request.window)
        });
    let mut related = Vec::new();
    let mut omitted = 0;
    let mut related_partial = false;
    if request.include_related
        && let Some(operation) = &operation
    {
        for child in &operation.children {
            if related.len() >= request.related_limit.min(32) {
                omitted += 1;
                continue;
            }
            let local = image
                .operations
                .iter()
                .find(|op| op.id == child.id)
                .cloned();
            let found = local.map(|op| (op, now)).or_else(|| {
                if child.id.process_instance == request.operation.process_instance {
                    return None;
                }
                let filename = format!(
                    "{:016x}-{:016x}.json",
                    child.id.process_instance[0], child.id.process_instance[1]
                );
                let remote = read_checkpoint(&path.parent()?.join(filename)).ok()?;
                let remote_now = remote
                    .clock
                    .as_ref()
                    .and_then(|mapping| mapping.observation_now());
                related_partial |= remote_now.is_none()
                    || remote.omitted > 0
                    || remote.registry_coverage.curtailed
                    || !remote.registry_coverage.available;
                let op = remote.operations.into_iter().find(|op| op.id == child.id)?;
                Some((op, remote_now))
            });
            if let Some((mut op, read_at)) = found {
                if read_at.is_none()
                    && op
                        .summary
                        .as_ref()
                        .is_some_and(|summary| summary.expires_at.is_some())
                {
                    op.coverage.stale = true;
                }
                let read_at = read_at.unwrap_or(op.read_at);
                related.push(pse_diagnostics::inspection::select_window(
                    op,
                    read_at,
                    request.window,
                ));
            } else {
                omitted += 1;
            }
        }
    }
    let transport = InspectionTransport {
        checkpoint: Some(path.to_owned()),
        process: Some(image.locator),
        published_unix_micros: Some(image.published_unix_micros),
        failure: None,
        partial: operation.is_none()
            || omitted > 0
            || image.omitted > 0
            || now.is_none()
            || related_partial
            || image.registry_coverage.curtailed
            || !image.registry_coverage.available,
        clock_uncertainty_nanoseconds: image.clock.map(|mapping| mapping.uncertainty_nanoseconds),
    };
    document(request, operation, related, omitted, transport)
}
/// Interpret one copied operation, without retaining its scientific handle or executing work.
pub fn inspect_snapshot(
    snapshot: OperationSnapshot,
    request: InspectionRequest,
) -> InspectionDocument {
    let operation = (snapshot.id == request.operation).then_some(snapshot);
    let transport = InspectionTransport {
        checkpoint: None,
        process: None,
        published_unix_micros: None,
        failure: None,
        partial: operation.is_none(),
        clock_uncertainty_nanoseconds: None,
    };
    let omitted = if request.include_related {
        operation.as_ref().map_or(0, |op| op.children.len())
    } else {
        0
    };
    let mut transport = transport;
    transport.partial |= omitted > 0;
    document(request, operation, Vec::new(), omitted, transport)
}
pub(crate) fn document(
    request: InspectionRequest,
    operation: Option<OperationSnapshot>,
    related: Vec<OperationSnapshot>,
    mut omitted: usize,
    transport: InspectionTransport,
) -> InspectionDocument {
    let mut statements = Vec::new();
    for snapshot in operation.iter().chain(&related) {
        let mut common = pse_diagnostics::inspection::explain(snapshot);
        // Context explanations precede generic point listings so bounds cannot hide meaning.
        statements.extend(context_explanations(snapshot));
        statements.extend(native_detail_explanations(snapshot));
        if let Some(policy) = &request.numerical_history_policy {
            statements.extend(pse_diagnostics::inspection::explain_numerical_history(
                snapshot, policy,
            ));
        }
        statements.append(&mut common);
        for duration in snapshot
            .metrics
            .iter()
            .filter(|metric| metric.unit == "nanoseconds")
        {
            let name = duration
                .name
                .strip_prefix("phase.")
                .unwrap_or(&duration.name);
            let stem = name
                .strip_suffix(".wall")
                .or_else(|| name.strip_suffix(".nanoseconds"))
                .or_else(|| name.strip_suffix(".elapsed_nanoseconds"));
            if let Some(stem) = stem
                && let Some(calls) = snapshot
                    .metrics
                    .iter()
                    .find(|metric| {
                        metric.producer == duration.producer
                            && metric.name == format!("{stem}.completed_calls")
                    })
                    .or_else(|| {
                        snapshot.metrics.iter().find(|metric| {
                            metric.producer == duration.producer
                                && metric.name == format!("{stem}.calls")
                        })
                    })
                && let Some(statement) =
                    pse_diagnostics::inspection::seconds_per_call(snapshot, duration, calls)
            {
                statements.push(statement);
            }
        }
    }
    if statements.len() > 128 {
        omitted += statements.len() - 128;
        statements.truncate(128);
    }
    InspectionDocument {
        version: Version,
        request,
        operation,
        related,
        statements,
        omitted,
        transport,
    }
}
fn native_detail_explanations(snapshot: &OperationSnapshot) -> Vec<Statement> {
    let mut seen = std::collections::BTreeSet::new();
    snapshot.producers.iter().flat_map(|producer| producer.current.iter().chain(&producer.recent)).filter_map(|record| {
        let (code,message)=match record.family.as_str() {
            "dynamic_local_error.v1" => ("native.idas.local_error_scope", "IDAS local-error vectors describe a fresh accepted internal state step on the actual native generation/return, with explicit availability for repeated, negative and zero-step returns. Contributor records join by generation, return ordinal and accepted step; they are state-only normalized coordinates and native weighted RMS contributions, not output accuracy or sensitivity/quadrature error estimates."),
            "native_factor.v2" => ("native.factor_scope", "Returned factor facts retain the actual factor-call parent, scaling/ordering status and current snapshot validity. Pivot magnitudes are not condition estimates; reported inertia concerns the perturbed native matrix. MC64 retry attempts and scaling fallbacks cover only their stated native populations, not all inner numerical retries."),
            "native_refinement.v2" => ("native.refinement_scope", "Selected monolithic refinement reports actual issued/completed RHS substitutions, correction batches and residual products. Parent call ordinals identify their actual enclosing backsolve; outer Schur/PdFullSpace paths and unselected intervals do not acquire these counts. Refinement bounds are not executed iteration counts."),
            "native_linear_call.v1" => ("native.linear_call_scope", "Begin/end boundaries carry actual attempt-local call ordinals and parentage, including nested operations. RHS population, issued/completed work and outer failure stay separate; these inclusive boundaries do not establish exclusive CPU cost."),
            _ => return None,
        };
        if !seen.insert(code) {return None;}
        Some(Statement {kind:StatementKind::Established,code:code.into(),message:message.into(),evidence:vec![pse_diagnostics::inspection::EvidenceRef {
            operation:snapshot.id,producer:Some(record.producer),record_sequence:Some(record.sequence),metric:None,observed_at:Some(record.observed_at),generation:Some(record.generation),value:None,partial:snapshot.interval_partial || snapshot.coverage.curtailed,
        }],next:Vec::new()})
    }).collect()
}
fn context_explanations(snapshot: &OperationSnapshot) -> Vec<Statement> {
    const RECIPES: &[(&str, &str, &str)] = &[
        (
            "compiler.",
            "context.preparation",
            "Preparation observations distinguish requested demand, actual construction, memo validation and rebind. Repeated requests alone do not establish rebuilding or reuse.",
        ),
        (
            "preparation.",
            "context.preparation",
            "Preparation counts describe their selected body/row/coordinate scope; construction and retained reuse remain distinct.",
        ),
        (
            "affine.",
            "context.affine",
            "Affine work follows original-row contributions. Candidate symbols, derivative attempts, nonzero coefficients and allowance units are different quantities; whole-program setup is separate.",
        ),
        (
            "callback.",
            "context.evaluation",
            "Callback duration is inclusive. Calls, derivative demand and nested provider/implicit work distinguish repetition from cost per evaluation.",
        ),
        (
            "provider.",
            "context.property",
            "Provider work retains derivative demand and original validity boundaries; a returned property checkpoint does not grant original scientific permission.",
        ),
        (
            "implicit.",
            "context.implicit",
            "Inner stages retain branch, accuracy, derivative and original-assessment meanings. Failed trials are actual attempted work, not accepted physical states.",
        ),
        (
            "ipopt.",
            "context.nlp",
            "Ipopt main/restoration and native/scaled checkpoints remain distinct from original candidate assessment. Enabled completed timers do not describe currently running timer tasks.",
        ),
        (
            "kinsol.",
            "context.root",
            "KINSOL root/fixed-point checkpoints retain method and failed-map attempts. Attempted iterations and transformed residuals are not accepted original states.",
        ),
        (
            "petsc.",
            "context.petsc",
            "PETSc nonlinear and linear convergence reasons remain separate; logging-off or ineligible getter coverage is unavailable, not zero work.",
        ),
        (
            "pounce.",
            "context.nlp",
            "POUNCE NLP/restoration, finite-difference and returned factor facts describe their actual owner and reset scope; they are not the separate convex route.",
        ),
        (
            "uno.",
            "context.nlp",
            "UNO accepted-iterate and termination-check callbacks are structured owner records with separate ordinals and meanings; neither supplies an inferred native iteration number. Opaque internal intervals require an explicit profile.",
        ),
        (
            "highs.",
            "context.coefficient",
            "HiGHS method-specific work, incumbent, bound and gap have different meanings. Basis submission does not prove retained factor reuse.",
        ),
        (
            "clarabel.",
            "context.conic",
            "Conic native residuals and normalized dimensions retain their formulation; public summaries do not expose unavailable private timer detail.",
        ),
        (
            "scip.",
            "context.discrete",
            "SCIP incumbent/bound/node facts and eligible plugin statistics retain stage, activation and restart scope; an incumbent is not global optimality.",
        ),
        (
            "feral.",
            "context.factor",
            "Symbolic analysis, numeric factorization and triangular solves are distinct. Returned fill/pivot facts are not concurrent reads of a mutable factor.",
        ),
        (
            "faer.",
            "context.factor",
            "Actual symbolic/numeric/solve boundaries and allocated extents are distinct from private fill and factor reuse permission.",
        ),
        (
            "klu.",
            "context.factor",
            "KLU attempted refactor, accepted refactor, conditioning rejection and full-factor fallback remain separate outcomes.",
        ),
        (
            "path.",
            "context.continuation",
            "Continuation trials, rejected corrections, subdivision and committed predecessors remain separate. A trial point does not become an accepted path point.",
        ),
        (
            "initialization.",
            "context.initialization",
            "Initialization stage attempts and retries retain their original assessment; completed inner work does not qualify the final process state.",
        ),
        (
            "recycle.",
            "context.recycle",
            "Recycle/tear trials and raw-map failures are attempted work. Iteration counts alone do not establish original closure.",
        ),
        (
            "sensitivity.",
            "context.analysis",
            "Sensitivity RHS/factor work and withholding are quantity-specific; one unavailable response does not invalidate another.",
        ),
        (
            "reduced.",
            "context.analysis",
            "Reduced/Hessian quantities retain their own active-set, scaling and validity premises; a factor alone does not establish all response permissions.",
        ),
        (
            "uncertainty.",
            "context.uncertainty",
            "Uncertainty propagation retains source covariance and response availability; missing quantities remain unavailable rather than zero uncertainty.",
        ),
        (
            "diffsol.",
            "context.dynamics",
            "Native mesh time, returned physical time, accepted/rejected steps and segment resets differ. Output samples are not native step counts.",
        ),
        (
            "idas.",
            "context.dynamics",
            "IDAS IC, forward/sensitivity/quadrature and backward memory retain initialization, reset and replay scope. Proposed steps are not accepted progress.",
        ),
        (
            "dynamics.",
            "context.dynamics",
            "Physical/returned/native times and segment/reset identities must be compared within their own scope, including forward and adjoint work.",
        ),
        (
            "study.",
            "context.study",
            "Study frontier/readiness, queued occurrences and actual worker execution are distinct. Follow dependencies and worker links; caller-local zero does not establish remote idleness.",
        ),
        (
            "fit.",
            "context.fitting",
            "Outer trials, experiment lanes, derivative upgrades, cache hits and profile chains retain separate identities; overlapping experiment times cannot be summed as wall latency.",
        ),
        (
            "surrogate.",
            "context.surrogate",
            "Surrogate fidelity/proposal work is separate from original correction and scientific permission; inspecting a busy actor does not request a model snapshot.",
        ),
        (
            "shooting.",
            "context.shooting",
            "Shooting trial/cache work, window integration and forward/adjoint requests are distinct nested owners.",
        ),
        (
            "horizon.",
            "context.horizon",
            "Foreground control, prediction and next-sample background work have distinct roles; fallback/hold decisions do not establish a new accepted optimum.",
        ),
        (
            "engine.",
            "context.engine",
            "Engine planning/operator/partition observations update independently; overlapping operator times are not request wall time or a coherent all-node instant.",
        ),
        (
            "canonical.",
            "context.transport",
            "Canonical query samples preserve statement status and supplied server duration, separately from client transport, admission and publication. They are not local active intervals or a complete query ledger.",
        ),
        (
            "native.",
            "context.drain",
            "Native work, stop request, returned result and actual join/drain are separate. Waiting on a native join overlaps its execution.",
        ),
    ];
    let mut result = Vec::new();
    for &(prefix, code, message) in RECIPES {
        if let Some(metric) = snapshot
            .metrics
            .iter()
            .find(|metric| metric.name.starts_with(prefix))
        {
            result.push(Statement::metric(
                snapshot,
                metric,
                StatementKind::Established,
                code,
                message,
                vec![NextObservation::InspectRelated],
            ));
        }
    }
    for metric in snapshot.metrics.iter().filter(|metric| {
        metric.name.ends_with("withheld_cause")
            || metric.name.contains("withheld_reason")
            || metric.name.ends_with("failure_code")
    }) {
        result.push(Statement::metric(snapshot,metric,StatementKind::Established,"quantity.cause","The owner supplied this quantity-specific cause. It is not a blanket failure of other quantities.",vec![]));
    }
    if snapshot.metrics.iter().any(|metric| {
        metric.name.contains("resident")
            || metric.name.contains("reserved")
            || metric.name.contains("cpu_ticks")
    }) {
        result.push(Statement::operation(snapshot,StatementKind::Limitation,"resource.scope","Pool reservations, current RSS, lifetime high-water and process CPU ticks have different scopes. None establishes a private operation peak, allocation churn or a known wait cause.",vec![NextObservation::OptionalAllocationProbe,NextObservation::OptionalWaitProbe]));
    }
    if snapshot
        .metrics
        .iter()
        .any(|metric| metric.name == "native.joined" && metric.latest == Value::Boolean(true))
    {
        result.push(Statement::operation(snapshot,StatementKind::Established,"native.joined","The native owner published actual join/drain, separately from numerical usability and publication.",vec![]));
    }
    if !snapshot.phases.is_empty() && snapshot.outcome.is_none() {
        result.push(Statement::operation(snapshot,StatementKind::Hypothesis,"activity.interior.unresolved","The last published phase may contain active computation or a wait not described by these checkpoints. CPU stacks and a later owner checkpoint can distinguish these possibilities; phase age alone cannot.",vec![NextObservation::CpuProfile,NextObservation::AwaitOwnerCheckpoint]));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::{Policy, Relationship};
    #[test]
    fn inspection_uno_keeps_accepted_trial_and_missing_kind_without_inferred_iterations() {
        use pse_diagnostics::observation::{ObservationHub, ProducerRole, RecordField, Value};
        let hub = ObservationHub::new(Policy::default());
        let owner = hub.start("uno.recorded");
        owner.context().count("uno.callbacks", "calls", 3);
        let producer = owner
            .context()
            .producer(ProducerRole::NativeIteration, "uno.actual.callback");
        for kind in [
            Some("native_accepted_iterate"),
            Some("native_termination_check"),
            None,
        ] {
            let mut fields = vec![RecordField {
                name: "coordinates".into(),
                unit: "availability".into(),
                value: Value::Unavailable("truncated coordinate capture".into()),
                availability: None,
            }];
            if let Some(kind) = kind {
                fields.push(RecordField {
                    name: "checkpoint_kind".into(),
                    unit: "interpretation".into(),
                    value: Value::Text(kind.into()),
                    availability: None,
                });
            }
            let record = producer.record("native_nlp.v1", "iterate", None, None, fields);
            assert!(producer.publish(&record));
        }
        let snapshot = owner.reader().snapshot(Duration::from_secs(60));
        let before = serde_json::to_vec(&snapshot).unwrap();
        let document = inspect_snapshot(snapshot.clone(), InspectionRequest::new(snapshot.id));
        let statement = document
            .statements
            .iter()
            .find(|statement| statement.code == "context.nlp")
            .unwrap();
        assert!(statement.message.contains("separate ordinals"));
        assert!(statement.message.contains("inferred native iteration"));
        assert_eq!(
            serde_json::to_vec(document.operation.as_ref().unwrap()).unwrap(),
            before
        );
        let records = &document.operation.as_ref().unwrap().producers[0].recent;
        assert!(records.iter().any(|record| record.values.iter().any(
            |field| matches!(&field.value,Value::Text(kind) if kind=="native_accepted_iterate")
        )));
        assert!(records.iter().any(|record| record.values.iter().any(
            |field| matches!(&field.value,Value::Text(kind) if kind=="native_termination_check")
        )));
        assert!(
            records
                .iter()
                .all(|record| record.values.iter().all(|field| field.name != "iteration"))
        );
    }
    #[test]
    fn inspection_busy_owner_is_passive_and_context_specific() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let operation = owner.hub().start("fit.experiment");
        let context = operation.context();
        let phase = context.phase("callback.derivative");
        context.duration("callback.wall", Duration::from_secs(6));
        context.count("callback.calls", "calls", 3);
        context.count("fit.cache_hits", "calls", 2);
        context.gauge("idas.native_time", "seconds", 2.0);
        context.gauge("dynamics.returned_time", "seconds", 1.5);
        context.unavailable("sensitivity.withheld_cause", "original assessment refused");
        context.count("process.resident_bytes", "bytes", 1024);
        let before = operation.reader().snapshot(Duration::from_secs(60));
        for _ in 0..8 {
            let document = owner.inspect(InspectionRequest::new(context.id()));
            let find = |code: &str| {
                document
                    .statements
                    .iter()
                    .find(|statement| statement.code == code)
                    .unwrap()
            };
            assert!(find("work.seconds_per_call").message.contains("2.000000"));
            assert_eq!(
                find("context.fitting").evidence[0].metric.as_deref(),
                Some("fit.cache_hits")
            );
            assert!(find("context.dynamics").message.contains("IDAS"));
            assert!(
                find("quantity.cause")
                    .message
                    .contains("not a blanket failure")
            );
            assert!(find("resource.scope").message.contains("known wait cause"));
            assert_eq!(
                find("activity.interior.unresolved").kind,
                StatementKind::Hypothesis
            );
            let observed = document.operation.as_ref().unwrap();
            assert!(
                observed
                    .interval_end
                    .saturating_sub(observed.interval_start)
                    < Duration::from_secs(60)
            );
        }
        let after = operation.reader().snapshot(Duration::from_secs(60));
        for metric in before.metrics {
            let current = after
                .metrics
                .iter()
                .find(|current| current.name == metric.name)
                .unwrap();
            assert_eq!(
                metric.cumulative, current.cumulative,
                "inspection executed producer work"
            );
            assert_eq!(metric.observed_at, current.observed_at);
        }
        drop(phase);
        owner.release().unwrap();
    }
    #[test]
    fn inspection_remote_links_and_missing_children_stay_explicit() {
        let directory = tempfile::tempdir().unwrap();
        let caller = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let worker = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let parent = caller.hub().start("study");
        let child = worker.hub().start("worker");
        parent
            .context()
            .link_id(child.context().id(), Relationship::DependencyWait);
        child.context().count("study.executed", "cases", 1);
        caller.publish().unwrap();
        worker.publish().unwrap();
        let request = InspectionRequest::new(parent.context().id());
        let document = inspect_checkpoint(&caller.checkpoint_path(), request.clone()).unwrap();
        assert_eq!(document.related.len(), 1);
        assert_eq!(document.related[0].id, child.context().id());
        let mut remote = read_checkpoint(&worker.checkpoint_path()).unwrap();
        remote.clock = None;
        std::fs::write(
            worker.checkpoint_path(),
            serde_json::to_vec(&remote).unwrap(),
        )
        .unwrap();
        let unmapped = inspect_checkpoint(&caller.checkpoint_path(), request.clone()).unwrap();
        assert_eq!(unmapped.related.len(), 1);
        assert!(
            unmapped.transport.partial,
            "remote clock absence is not root clock coverage"
        );
        worker.release().unwrap();
        let missing = inspect_checkpoint(&caller.checkpoint_path(), request).unwrap();
        assert!(missing.related.is_empty());
        assert_eq!(missing.omitted, 1);
        assert!(missing.transport.partial);
        caller.release().unwrap();
    }
    #[test]
    fn unmapped_terminal_summary_eligibility_is_stale_for_selected_and_related_copies() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
        let parent = owner.hub().start("parent responsibility");
        parent.context().declare_summary("parent facts");
        let child = parent
            .context()
            .child("child responsibility", Relationship::Nested);
        child.context().declare_summary("child facts");
        let child_reader = child.reader();
        let parent_id = parent.context().id();
        child.finish(pse_diagnostics::observation::Outcome::Completed);
        parent.finish(pse_diagnostics::observation::Outcome::Completed);
        owner.publish().unwrap();
        let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
        image.clock = None;
        let document = inspect_image(
            &owner.checkpoint_path(),
            InspectionRequest::new(parent_id),
            image,
        );
        assert!(document.operation.unwrap().coverage.stale);
        assert_eq!(document.related.len(), 1);
        assert_eq!(document.related[0].id, child_reader.id());
        assert!(document.related[0].coverage.stale);
        assert!(document.transport.partial);
    }
    #[test]
    fn inspection_evicted_root_is_absent_while_exact_child_and_reader_remain_available() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(
            Policy {
                max_operations: 2,
                ..Policy::default()
            },
            directory.path().to_owned(),
        );
        let root = owner.hub().start("root");
        let reader = root.reader();
        let child = root.context().child("active.child", Relationship::Nested);
        child.context().count("actual.requests", "calls", 3);
        root.finish(pse_diagnostics::observation::Outcome::Completed);
        let replacement = owner.hub().start("replacement");
        owner.publish().unwrap();
        let request = InspectionRequest::new(reader.id());
        for document in [
            owner.inspect(request.clone()),
            inspect_checkpoint(&owner.checkpoint_path(), request.clone()).unwrap(),
        ] {
            let copied: InspectionDocument =
                serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
            assert_eq!(copied.request.operation, reader.id());
            assert!(copied.operation.is_none());
            assert!(copied.related.is_empty());
            assert!(copied.transport.partial);
        }
        let retained = inspect_snapshot(reader.snapshot(Duration::from_secs(60)), request);
        assert_eq!(
            retained.operation.unwrap().children[0].id,
            child.context().id()
        );
        assert!(
            retained.transport.partial,
            "copied root alone cannot fabricate child coverage"
        );
        let active = inspect_checkpoint(
            &owner.checkpoint_path(),
            InspectionRequest::new(child.context().id()),
        )
        .unwrap();
        assert_eq!(active.operation.as_ref().unwrap().id, child.context().id());
        assert!(active.operation.as_ref().unwrap().outcome.is_none());
        assert!(
            owner
                .inspect(InspectionRequest::new(replacement.context().id()))
                .related
                .is_empty()
        );
        owner.release().unwrap();
    }
    #[test]
    fn inspection_serialization_preserves_producer_generations_resets_and_loss() {
        let directory = tempfile::tempdir().unwrap();
        let owner = Diagnostics::new(
            Policy {
                detail_capacity: 1,
                max_metrics: 2,
                ..Policy::default()
            },
            directory.path().to_owned(),
        );
        let operation = owner.hub().start("native.checkpoints");
        let context = operation.context();
        context.count_value("native.calls", "calls", 100, 7);
        context.count_value("native.calls", "calls", 105, 7);
        context.count_value("native.calls", "calls", 2, 8);
        context.count_value("native.calls", "calls", 5, 8);
        context.integer("native.exact", "count", i64::MAX);
        context.count("refused.extra", "calls", 1);
        drop(context.phase("sample"));
        operation.finish(pse_diagnostics::observation::Outcome::Completed);
        context.count("native.calls", "calls", 999);
        owner.publish().unwrap();
        let before = owner
            .reader(context.id())
            .unwrap()
            .snapshot(Duration::from_secs(60));
        for _ in 0..8 {
            let documents = [
                owner.inspect(InspectionRequest::new(context.id())),
                inspect_checkpoint(
                    &owner.checkpoint_path(),
                    InspectionRequest::new(context.id()),
                )
                .unwrap(),
            ];
            for document in documents {
                let copied: InspectionDocument =
                    serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
                assert!(
                    copied
                        .statements
                        .iter()
                        .any(|statement| statement.code == "coverage.partial")
                );
                let reset = copied
                    .statements
                    .iter()
                    .find(|statement| statement.code == "metric.reset")
                    .unwrap();
                assert_eq!(reset.kind, StatementKind::Limitation);
                assert_eq!(reset.evidence[0].operation, context.id());
                assert!(reset.evidence[0].partial);
                let snapshot = copied.operation.unwrap();
                assert_eq!(snapshot.id, context.id());
                assert_eq!(snapshot.outcome, before.outcome);
                assert_eq!(snapshot.coverage.dropped_updates, 1);
                assert!(snapshot.coverage.refused_items > 0);
                assert!(snapshot.coverage.dropped_detail > 0);
                assert!(
                    snapshot.coverage.curtailed
                        && snapshot.interval_partial
                        && snapshot.detail_partial
                );
                let calls = snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == "native.calls")
                    .unwrap();
                assert_eq!(calls.latest, Value::Count(5));
                assert_eq!(calls.cumulative, Value::Count(8));
                assert_eq!(calls.generation, 8);
                assert_eq!(calls.resets, 1);
                assert!(calls.cumulative_partial && calls.recent_partial && calls.history_partial);
                assert!(calls.history.iter().any(|point| point.reset));
                assert_eq!(calls.history.first().unwrap().first_generation, 7);
                assert_eq!(calls.history.last().unwrap().generation, 8);
                assert_eq!(
                    snapshot
                        .metrics
                        .iter()
                        .find(|metric| metric.name == "native.exact")
                        .unwrap()
                        .latest,
                    Value::Integer(i64::MAX)
                );
            }
        }
        let after = owner
            .reader(context.id())
            .unwrap()
            .snapshot(Duration::from_secs(60));
        assert_eq!(
            serde_json::to_vec(&before.metrics).unwrap(),
            serde_json::to_vec(&after.metrics).unwrap()
        );
        assert_eq!(before.observed_at, after.observed_at);
        owner.release().unwrap();
    }
    #[test]
    fn inspection_discovery_keeps_corruption_and_has_bounded_pages() {
        let directory = tempfile::tempdir().unwrap();
        for name in ["a.json", "b.json", "c.json"] {
            std::fs::write(directory.path().join(name), b"corrupt").unwrap();
        }
        let first = list_processes(
            directory.path(),
            DiagnosticListRequest {
                after: None,
                limit: 2,
            },
        )
        .unwrap();
        assert_eq!(first.processes.len(), 2);
        assert_eq!(first.next.as_deref(), Some("b.json"));
        assert!(
            first
                .processes
                .iter()
                .all(|process| process.partial && process.failure.is_some())
        );
        let last = list_processes(
            directory.path(),
            DiagnosticListRequest {
                after: first.next,
                limit: 2,
            },
        )
        .unwrap();
        assert_eq!(last.processes.len(), 1);
        assert!(last.next.is_none());
    }
}
