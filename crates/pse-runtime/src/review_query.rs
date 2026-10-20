// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Context of a declared optional query. No tool is installed or run by decoding it.
use pse_model::document::Version;
use serde::{Deserialize, Serialize};

/// A half-open interval in one explicitly named source clock, in exact nanoseconds.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QueryInterval {
    /// Inclusive lower bound.
    pub start: i64,
    /// Exclusive upper bound.
    pub end: i64,
}
/// Existing registry views over exactly one selected immutable supplied member.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewTableKind {
    /// Whole coherent event fields, with native UInt64 columns and empty events.
    Detail,
    /// Joined invocation facts.
    Invocation,
    /// Optional capture state, independent of numerical permission.
    Capture,
    /// Copied retained engine observations.
    Engine,
    /// Contextual declared-query results, including failures.
    Query,
}
/// Finite typed-table projection, not an arbitrary SQL or file-discovery request.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewTableRequest {
    /// Request codec.
    pub version: Version<1>,
    /// Existing registry view.
    pub kind: ReviewTableKind,
    /// Exact zero-based member in the corresponding supplied collection.
    pub index: usize,
    /// Participating projection allocation; escaped Arrow buffers retain their lease.
    pub memory_bytes: usize,
}
/// A finalized selected typed-table export; the IPC schema retains its registry metadata.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewTableDocument {
    /// Result codec.
    pub version: Version<1>,
    /// Exact accepted table/member/allocation request.
    pub request: ReviewTableRequest,
    /// Supplied source identity, descriptive only on future offline read.
    pub input_member: String,
    /// Explicit finalized local IPC stream, never an IPC file.
    pub output: String,
    /// Returned rows, not a claim about unselected source populations.
    pub rows: u64,
    /// Actual registry/schema metadata copied from the finalized table.
    pub schema_metadata: std::collections::BTreeMap<String, String>,
}
/// One of the maintained queries; callers cannot provide arbitrary commands.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewQuery {
    /// Completed inclusive intervals and their clipped union per capture-local track.
    Timeline,
    /// Source numerical events; typed native inputs remain full-UInt64 authority.
    Numerical,
    /// Reported trace errors/loss, not proof of capture completeness.
    Coverage,
}
/// Explicit optional query request, separate from a scientific operation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewQueryRequest {
    /// Request codec.
    pub version: Version<1>,
    /// Exact selected capture/member identity; not an original-path lookup hint.
    pub input_member: String,
    /// Declared query.
    pub question: ReviewQuery,
    /// Source clock name, scoped to this capture.
    pub clock_scope: String,
    /// Requested interval, or whole supplied capture.
    pub interval: Option<QueryInterval>,
    /// Finite positive child lifetime in seconds.
    pub timeout_seconds: f64,
    /// Finite combined stdout/stderr extent; metadata has a separate bounded extent.
    pub output_bytes: u64,
}
/// Status never conflates absent/failed output with empty success.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewQueryStatus {
    /// One well-formed result set, possibly empty.
    Completed,
    /// Optional tool or input was not available.
    Unavailable,
    /// Tool exited unsuccessfully.
    Failed,
    /// Caller clock elapsed; exact child was joined.
    TimedOut,
    /// Explicit caller interruption; exact child was joined.
    Cancelled,
    /// Retained output allowance was exhausted.
    OutputLimit,
    /// Successful process did not produce the declared single CSV result schema.
    Malformed,
}
/// A declared output column; strings preserve canonical native integer text.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewQueryColumn {
    /// Stable output name.
    pub name: String,
    /// Integer, number or text; Perfetto integers are signed, never full UInt64.
    pub kind: String,
    /// Explicit unit, absent for labels/identities.
    pub unit: Option<String>,
}
/// Retained query product, including readable failure prefixes and interpretation limits.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewQueryDocument {
    /// Result codec.
    pub version: Version<1>,
    /// Exact accepted request.
    pub request: ReviewQueryRequest,
    /// Maintained query revision, independent of tool release.
    pub query_version: u32,
    /// Exact executed SQL, including interval parameters.
    pub sql: String,
    /// Actual `--version` output, unavailable rather than declared pin substitution.
    pub tool_version: Option<String>,
    /// Actual execution status.
    pub status: ReviewQueryStatus,
    /// Exact process exit status, absent if no process was started.
    pub exit_code: Option<i32>,
    /// Schema expected from this query, not inferred from returned data.
    pub columns: Vec<ReviewQueryColumn>,
    /// Interval actually covered by source metadata, if supplied/known.
    pub covered_interval: Option<QueryInterval>,
    /// Relative private output path and readable bytes, even on failure.
    pub output: String,
    /// Actual retained bytes, not necessarily complete CSV rows on failure.
    pub output_prefix_bytes: u64,
    /// Relative retained stderr path.
    pub error_output: String,
    /// Actual retained error bytes.
    pub error_prefix_bytes: u64,
    /// Decoded rows only after successful schema validation; no count on failure.
    pub rows: Option<u64>,
    /// Source gaps, clock limits, unsupported arithmetic and output omissions.
    pub limitations: Vec<String>,
}

/// One declared query and its schema. Data values never define a schema.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewQuerySpecification {
    /// Exact SQL including checked integer parameters.
    pub sql: String,
    /// Declared result schema.
    pub columns: Vec<ReviewQueryColumn>,
    /// Query revision.
    pub query_version: u32,
    /// Limits applying even to successful output.
    pub limitations: Vec<String>,
}
/// Prepare only a maintained query; this performs no I/O or external execution.
pub fn specification(
    request: &ReviewQueryRequest,
) -> Result<ReviewQuerySpecification, crate::observation::ObservationError> {
    use crate::observation::ObservationError;
    if request.input_member.is_empty()
        || request.clock_scope.is_empty()
        || !request.timeout_seconds.is_finite()
        || request.timeout_seconds <= 0.
        || request.output_bytes < 2
        || request.output_bytes > (1 << 30)
        || request.interval.as_ref().is_some_and(|i| i.start >= i.end)
    {
        return Err(ObservationError::Format(
            "invalid finite query request".into(),
        ));
    }
    type QueryColumn<'a> = (&'a str, &'a str, Option<&'a str>);
    let (sql, names): (&str, &[QueryColumn<'_>]) = match request.question {
        ReviewQuery::Timeline => (
            include_str!("../../../scripts/diagnostic_sql/timeline.sql"),
            &[
                ("track_id", "integer", None),
                ("owner", "text", None),
                ("intervals", "integer", Some("count")),
                ("original_inclusive_ns", "integer", Some("nanoseconds")),
                ("clipped_inclusive_ns", "integer", Some("nanoseconds")),
                ("union_ns", "integer", Some("nanoseconds")),
                ("first_original_ts", "integer", Some("nanoseconds")),
                ("last_original_end", "integer", Some("nanoseconds")),
                ("instants", "integer", Some("count")),
                ("open_intervals", "integer", Some("count")),
            ],
        ),
        ReviewQuery::Coverage => (
            include_str!("../../../scripts/diagnostic_sql/coverage.sql"),
            &[
                ("name", "text", None),
                ("severity", "text", None),
                ("source", "text", None),
                ("value", "integer", Some("native")),
            ],
        ),
        ReviewQuery::Numerical => (
            include_str!("../../../scripts/diagnostic_sql/numerical.sql"),
            &[
                ("track_id", "integer", None),
                ("ts", "integer", Some("nanoseconds")),
                ("phase", "text", None),
                ("operation_id", "text", None),
                ("producer_sequence", "text", None),
                ("family", "text", None),
                ("source_sequence", "text", None),
                ("family_sequence", "text", None),
                ("generation", "text", None),
                ("history_partial", "integer", None),
                ("iteration", "text", None),
                ("primal_native", "number", Some("native")),
                ("pounce_primal", "number", Some("native")),
                ("step_norm", "number", Some("native")),
                ("pounce_step", "number", Some("native")),
                ("step_available", "integer", None),
                ("step_norm_kind", "text", None),
                ("step_coordinates", "text", None),
            ],
        ),
    };
    let (start, end) = request
        .interval
        .as_ref()
        .map_or((i64::MIN, i64::MAX), |i| (i.start, i.end));
    let mut limitations = vec![
        "Track/upid/utid are scoped to this capture. No cross-clock alignment or critical path is inferred.".into(),
        "Inclusive, clipped union, sampled CPU and exclusive ownership time are different populations.".into(),
        "Perfetto SQL is signed-int64/double/text; original native UInt64/Arrow or canonical text remains authority. Unsupported counter arithmetic and wrapped values are not interpreted.".into(),
        "Unsigned text companions absent in earlier traces remain unavailable; wrapped SQL counts are never a fallback.".into(),
        "Covered source interval is unavailable without supplied capture metadata; empty rows do not establish capture completeness. Open intervals have no complete duration.".into(),
    ];
    if matches!(request.question, ReviewQuery::Coverage) && request.interval.is_some() {
        limitations.push("Importer loss/error statistics are capture-wide and cannot be filtered to the requested interval.".into());
    }
    Ok(ReviewQuerySpecification {
        sql: sql
            .replace("__START__", &start.to_string())
            .replace("__END__", &end.to_string()),
        columns: names
            .iter()
            .map(|(name, kind, unit)| ReviewQueryColumn {
                name: (*name).into(),
                kind: (*kind).into(),
                unit: unit.map(str::to_owned),
            })
            .collect(),
        query_version: 2,
        limitations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_parameters_are_exact_bounded_and_never_arbitrary_sql() {
        let mut request = ReviewQueryRequest {
            version: Default::default(),
            input_member: "capture-1".into(),
            question: ReviewQuery::Timeline,
            clock_scope: "capture-1:monotonic".into(),
            interval: Some(QueryInterval { start: 4, end: 8 }),
            timeout_seconds: 1.,
            output_bytes: 4096,
        };
        let spec = specification(&request).unwrap();
        assert!(
            spec.sql
                .contains("ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING")
        );
        assert!(!spec.sql.contains("__START__"));
        assert!(spec.sql.contains("s.ts<8"));
        request.interval = Some(QueryInterval { start: 8, end: 4 });
        assert!(specification(&request).is_err());
    }
}

#[cfg(test)]
mod adapter_contract_tests {
    use super::*;
    fn request(question: ReviewQuery) -> ReviewQueryRequest {
        ReviewQueryRequest {
            version: Default::default(),
            input_member: "frozen#/captures/7".into(),
            question,
            clock_scope: "capture-7:monotonic".into(),
            interval: Some(QueryInterval {
                start: i64::MIN,
                end: i64::MAX,
            }),
            timeout_seconds: 0.25,
            output_bytes: 4096,
        }
    }
    #[test]
    fn numerical_query_uses_unsigned_text_authority_and_declared_column_meaning() {
        let request = request(ReviewQuery::Numerical);
        let before = serde_json::to_vec(&request).unwrap();
        let spec = specification(&request).unwrap();
        for name in [
            "producer_sequence",
            "source_sequence",
            "family_sequence",
            "generation",
            "iteration",
        ] {
            let column = spec
                .columns
                .iter()
                .find(|column| column.name == name)
                .unwrap();
            assert_eq!(column.kind, "text");
            assert!(spec.sql.contains(&format!("debug.{name}.u64_text")));
            assert!(
                !spec
                    .sql
                    .contains(&format!("EXTRACT_ARG(s.arg_set_id, 'debug.{name}')")),
                "wrapped native integer must not become fallback authority"
            );
        }
        assert!(spec.sql.contains(&i64::MIN.to_string()));
        assert!(spec.sql.contains(&i64::MAX.to_string()));
        assert!(!spec.sql.contains("__START__"));
        assert!(!spec.sql.contains("__END__"));
        assert!(
            spec.limitations
                .iter()
                .any(|text| text.contains("earlier traces remain unavailable"))
        );
        assert_eq!(
            serde_json::to_vec(&request).unwrap(),
            before,
            "specification is pure over the supplied request"
        );
    }
    #[test]
    fn optional_query_refuses_invalid_finite_context_and_keeps_capture_wide_coverage() {
        for question in [
            ReviewQuery::Timeline,
            ReviewQuery::Numerical,
            ReviewQuery::Coverage,
        ] {
            let request = request(question);
            assert!(specification(&request).is_ok());
            for timeout in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut invalid = request.clone();
                invalid.timeout_seconds = timeout;
                assert!(specification(&invalid).is_err());
            }
            for bytes in [0, 1, (1u64 << 30) + 1] {
                let mut invalid = request.clone();
                invalid.output_bytes = bytes;
                assert!(specification(&invalid).is_err());
            }
            for interval in [
                QueryInterval { start: 5, end: 5 },
                QueryInterval { start: 6, end: 5 },
            ] {
                let mut invalid = request.clone();
                invalid.interval = Some(interval);
                assert!(specification(&invalid).is_err());
            }
            let mut missing = request.clone();
            missing.input_member.clear();
            assert!(specification(&missing).is_err());
            missing = request.clone();
            missing.clock_scope.clear();
            assert!(specification(&missing).is_err());
        }
        let request = request(ReviewQuery::Coverage);
        let spec = specification(&request).unwrap();
        assert!(
            spec.limitations
                .iter()
                .any(|text| text.contains("capture-wide"))
        );
        assert_eq!(
            spec.columns
                .iter()
                .map(|column| column.name.as_str())
                .collect::<Vec<_>>(),
            ["name", "severity", "source", "value"]
        );
    }
}

/// Exact original collection pointer selected by this registry projection.
pub fn table_source(request: &ReviewTableRequest) -> String {
    let collection = match request.kind {
        ReviewTableKind::Detail => "detail_pages",
        ReviewTableKind::Invocation => "invocations",
        ReviewTableKind::Capture => "captures",
        ReviewTableKind::Engine => "engine_observations",
        ReviewTableKind::Query => "query_results",
    };
    format!("/{collection}/{}", request.index)
}
/// Declared registry relation, independent of scientific-result lineage.
pub fn table_relation(request: &ReviewTableRequest) -> &'static str {
    match request.kind {
        ReviewTableKind::Detail => "runtime.diagnostic_evidence_fields",
        ReviewTableKind::Invocation => "runtime.diagnostic_invocations",
        ReviewTableKind::Capture => "runtime.diagnostic_captures",
        ReviewTableKind::Engine => "runtime.diagnostic_engine_observations",
        ReviewTableKind::Query => "runtime.diagnostic_query_results",
    }
}
/// Project supplied immutable facts through the existing schema owner and retained allocation.
pub fn table_arrow(
    set: &crate::frozen_inspection::FrozenEvidenceSet,
    request: &ReviewTableRequest,
    cancel: &pse_columnar::CancellationToken,
) -> Result<datafusion::arrow::record_batch::RecordBatch, crate::capture::CaptureError> {
    use crate::capture::{CaptureError, evidence};
    if request.memory_bytes == 0 || request.memory_bytes > 64 << 20 {
        return Err(CaptureError::Invalid(
            "review view needs a finite allocation at most 64 MiB".into(),
        ));
    }
    let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(request.memory_bytes));
    let missing = || CaptureError::Invalid("selected supplied table member is unavailable".into());
    match request.kind {
        ReviewTableKind::Detail => evidence::detail_arrow(
            set.detail_pages.get(request.index).ok_or_else(missing)?,
            &pool,
            cancel,
        ),
        ReviewTableKind::Invocation => evidence::invocation_arrow(
            set.invocations.get(request.index).ok_or_else(missing)?,
            &pool,
            cancel,
        ),
        ReviewTableKind::Capture => evidence::capture_arrow(
            set.captures.get(request.index).ok_or_else(missing)?,
            &pool,
            cancel,
        ),
        ReviewTableKind::Engine => evidence::engine_arrow(
            &format!("frozen#{}", table_source(request)),
            set.engine_observations
                .get(request.index)
                .ok_or_else(missing)?,
            &pool,
            cancel,
        ),
        ReviewTableKind::Query => evidence::query_arrow(
            &format!("frozen#{}", table_source(request)),
            set.query_results.get(request.index).ok_or_else(missing)?,
            &pool,
            cancel,
        ),
    }
}
