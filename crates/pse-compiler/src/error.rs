// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! What the compiler can be wrong about (blueprint §23.2).
//!
//! The authoring and rule errors are re-exported through `#[from]` with
//! `#[diagnostic(transparent)]`: a caller sees the original code — `authoring.parse.syntax`,
//! `rule.float_key` — rather than a compiler-shaped wrapper that has lost which layer knew
//! what was wrong. Failures carry the relation rows and source spans involved; they are
//! never flattened into "run failed".

use pse_authoring::AuthoringError;
use pse_rules::RuleError;
use pse_schema::SchemaError;

/// A pipeline that will not run, or a stage that failed.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CompilerError {
    /// Cancellation retained every finding the pass had already produced.
    #[error("pass cancelled")]
    #[diagnostic(code(runtime::cancelled))]
    Cancelled {
        /// Actual diagnostic rows from the interrupted execution.
        findings: Vec<pse_relations::RecordBatch>,
    },
    /// Output admission failed after a pass had produced structured findings.
    #[error("{source}")]
    #[diagnostic(forward(source))]
    PassFailure {
        /// Actual admission or publication failure.
        #[source]
        source: Box<Self>,
        /// Complete findings already produced by that pass.
        findings: Vec<pse_relations::RecordBatch>,
    },
    /// The attempt failed and its complete terminal record was durably admitted.
    #[error("pass attempt {pass_run_id} failed: {source}")]
    #[diagnostic(forward(source))]
    AttemptFailed {
        /// Identity assigned before execution.
        pass_run_id: pse_ids::SemanticId,
        /// Actual admitted failure sidecar, available for complete typed inspection.
        record: Box<pse_catalog::store::sidecar::SidecarArtifact>,
        /// Original failure, preserving its class and structured findings.
        #[source]
        source: Box<Self>,
    },
    /// The original failed attempt and the failed terminal recording operation.
    #[error("pass attempt {pass_run_id} failed and its terminal record could not be persisted")]
    #[diagnostic(code(runtime::infrastructure))]
    TerminalRecording {
        /// Identity assigned before execution.
        pass_run_id: pse_ids::SemanticId,
        /// Original execution failure followed by the recording failure, both retained.
        #[related]
        errors: Vec<Self>,
    },
    /// Successful work could not obtain an admitted terminal record.
    #[error("pass attempt {pass_run_id} completed but terminal recording failed: {source}")]
    #[diagnostic(code(runtime::infrastructure))]
    SuccessRecording {
        /// Identity assigned before execution.
        pass_run_id: pse_ids::SemanticId,
        /// Complete output, when work produced a snapshot; no memo hint was written.
        output: Option<pse_ids::SnapshotId>,
        /// Recording failure, without an invented execution failure.
        #[source]
        source: Box<Self>,
    },
    /// A completed, recorded pass could not publish its optional reuse index.
    #[error("successful pass attempt {pass_run_id} could not publish its reuse hint: {source}")]
    #[diagnostic(code(runtime::infrastructure))]
    AuxiliaryHint {
        /// The truthful successful terminal attempt.
        pass_run_id: pse_ids::SemanticId,
        /// Its actual complete admitted output.
        output: pse_ids::SnapshotId,
        /// Auxiliary store/index failure.
        #[source]
        source: Box<Self>,
    },
    /// A successful recorded P2 result could not become the selected commit revision.
    #[error("successful P2 attempt {pass_run_id} could not publish its commit ref: {source}")]
    #[diagnostic(code(runtime::infrastructure))]
    CommitPublication {
        /// The successful immutable P2 attempt.
        pass_run_id: pse_ids::SemanticId,
        /// The complete immutable case output.
        output: pse_ids::SnapshotId,
        /// The truthful successful P2 terminal record, already admitted.
        record: Box<pse_catalog::store::sidecar::SidecarArtifact>,
        /// The final publication failure; the pass outcome remains successful.
        #[source]
        source: Box<Self>,
    },
    /// The declared ports do not describe a runnable pipeline (blueprint §14.1).
    #[error("stage graph: {reason}")]
    #[diagnostic(
        code(compile::stage_graph),
        help("every read names its source stage and port; one port has one producer")
    )]
    StageGraph {
        /// What is wrong with the graph.
        reason: String,
    },

    /// A pass postcondition failed.
    #[error("internal invariant: {what}")]
    #[diagnostic(
        code(internal::invariant),
        help(
            "a failed postcondition is a platform bug; the pass declared it and did not establish it"
        )
    )]
    Internal {
        /// What did not hold.
        what: String,
    },

    /// A pass failed its declared contract, retaining the actual diagnostic rows.
    #[error("{pass} failed its postconditions")]
    #[diagnostic(code(internal::invariant))]
    Postcondition {
        /// The pass whose declared output contract failed.
        pass: String,
        /// All findings produced by the failed execution.
        findings: Vec<pse_relations::RecordBatch>,
    },

    /// A budgeted resource ran out.
    #[error("{consumer} exceeded its resource limit")]
    #[diagnostic(
        code(runtime::resource_limit),
        help(
            "preflight reserves before allocating; exceeding a bound is never a partial publication (blueprint §5.3 step 8)"
        )
    )]
    ResourceLimit {
        /// Which consumer exhausted its budget.
        consumer: String,
        /// The configuration keys that bound it.
        config_keys: Vec<String>,
    },

    /// Storage or input/output failed underneath a pass.
    #[error("infrastructure failure during {op}: {detail}")]
    #[diagnostic(
        code(runtime::infrastructure),
        help("an interrupted operation leaves the old ref valid and publishes nothing mixed")
    )]
    Infrastructure {
        /// What was being done.
        op: String,
        /// What the layer below reported.
        detail: String,
    },

    /// An authoring failure, with its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Authoring(#[from] AuthoringError),

    /// A rule failure, with its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Rule(#[from] RuleError),

    /// A catalog admission or publication failure, retaining its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Catalog(#[from] pse_catalog::CatalogError),

    /// A typed relation admission failure.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Relation(#[from] pse_relations::RelationError),

    /// Cancellation or a bounded canonical operation failed.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Canon(#[from] pse_ids::CanonError),

    /// Physical quantity admission failed.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Quantity(#[from] pse_quantity::QuantityError),

    /// Mathematical graph admission failed.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MathIr(#[from] pse_mathir::MathIrError),

    /// A registry failure, with its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Schema(#[from] SchemaError),
}
