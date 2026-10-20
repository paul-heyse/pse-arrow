// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Derived diagnostic views; raw source artifacts retain their independent authority.
use super::declarations::{column, relation, relation_version};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
pub(super) fn declare(b: &mut RegistryBuilder) {
    let text = || T::native(D::Utf8);
    let count = || T::native(D::UInt64);
    relation(
        b,
        N::Runtime,
        "diagnostic_evidence_fields",
        S::Derived,
        &[
            "operation",
            "producer",
            "generation",
            "event_sequence",
            "field_ordinal",
        ],
        vec![
            column("operation", text()),
            column("producer", count()),
            column("generation", count()),
            column("event_sequence", count()),
            column("family_sequence", count()).optional(),
            column("observed_seconds", count()),
            column("observed_nanoseconds", T::native(D::UInt32)),
            column("family", text()),
            column("phase", text()),
            column("coordinates", text()).optional(),
            column("norm", text()).optional(),
            column("field_ordinal", count()),
            column("field_name", text()).optional(),
            column("unit", text()).optional(),
            column("value_kind", text()).optional(),
            column("count_value", count()).optional(),
            column("integer_value", T::native(D::Int64)).optional(),
            column("number_value", T::native(D::Float64)).optional(),
            column("boolean_value", T::native(D::Boolean)).optional(),
            column("text_value", text()).optional(),
            column("availability", text()).optional(),
            column("partial", T::native(D::Boolean)),
        ],
        "Immutable diagnostic field projection. Operation/producer/generation/event identifies one source event; field ordinal is one-based, zero preserves an empty event. Typed values, units, availability and exact monotonic time retain source meaning. Partial never implies complete history; raw source pages remain authority.",
    );
    relation(
        b,
        N::Runtime,
        "diagnostic_query_results",
        S::Derived,
        &["member"],
        vec![
            column("member", text()),
            column("question", text()),
            column("clock_scope", text()),
            column("status", text()),
            column("query_version", T::native(D::UInt32)),
            column("tool_version", text()).optional(),
            column("rows", count()).optional(),
            column("document_json", text()),
        ],
        "Selected declared-query result and exact query/tool/window/schema context. Failed, unavailable, empty successful and partial outputs are distinct; source clock and full UInt64 authority remain with original native evidence.",
    );
    relation(
        b,
        N::Runtime,
        "diagnostic_engine_observations",
        S::Derived,
        &["member", "observation"],
        vec![
            column("member", text()),
            column("observation", count()),
            column("operation", text()).optional(),
            column("policy", text()),
            column("status", text()).optional(),
            column("dropped", count()),
            column("document_json", text()),
        ],
        "Copied retained engine observations. Actual policy, metric unit/kind and stream completion/abandonment stay in the typed document; absent plans/metrics are unavailable rather than implicitly enabled or zero.",
    );
    for (name, key, columns, doc) in [
        (
            "diagnostic_invocations",
            "run",
            vec![
                column("run", text()),
                column("attempt", text()).optional(),
                column("operation", text()).optional(),
                column("family", text()),
                column("outcome", text()),
                column("clock_scope", text()),
                column("qualification_seconds", T::native(D::Float64)).optional(),
                column("terminal_seconds", T::native(D::Float64)).optional(),
                column("response_ready_seconds", T::native(D::Float64)).optional(),
                column("partial", T::native(D::Boolean)),
                column("document_json", text()),
            ],
            "Immutable joined invocation evidence. Endpoints are independently observed supervisor-clock seconds; missing is not zero. The original typed document retains attempts, requested products, coverage, starts and settings; intervals overlap and must not be summed.",
        ),
        (
            "diagnostic_captures",
            "subject",
            vec![
                column("subject", text()),
                column("operation", text()).optional(),
                column("launch_process_json", text()).optional(),
                column("capture_kind", text()),
                column("status", text()),
                column("document_json", text()),
            ],
            "Immutable optional capture link and its original typed document. Capture failure, output prefixes, loss, symbols and process/thread scope remain independent from scientific result permission.",
        ),
    ] {
        relation_version(
            b,
            N::Runtime,
            name,
            if name == "diagnostic_captures" { 2 } else { 1 },
            S::Derived,
            &[key],
            columns,
            doc,
        );
    }
}
