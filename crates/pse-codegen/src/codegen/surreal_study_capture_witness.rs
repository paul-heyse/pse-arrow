// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact selector/fact dependencies for atomic study capture publication.

/// Every writer, including removal and future matches, changes the witness in
/// its own transaction. Lease, heartbeat and worker-expiry fields are absent.
pub(super) fn append(ddl: &mut String) {
    ddl.push_str(
        "DEFINE FUNCTION fn::pse_workbench_v1::publication($rpc_expiry:int) -> bool {\n\
         LET $witness=SELECT * FROM ONLY type::record('canonical_guards','study-publication') FOR UPDATE;\n\
         IF $witness=NONE { THROW 'PSE_REFUSAL:study_capture_publication_witness_unavailable'; };\n\
         fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;\n};\n\
         DEFINE FUNCTION fn::pse_workbench_v1::changed() -> bool {\n\
         LET $witness=SELECT * FROM ONLY type::record('canonical_guards','study-publication') FOR UPDATE;\n\
         IF $witness=NONE { THROW 'PSE_REFUSAL:study_capture_publication_witness_unavailable'; };\n\
         UPDATE ONLY type::record('canonical_guards','study-publication') SET generation+=1u64 RETURN NONE;\n\
         RETURN true;\n} PERMISSIONS NONE;\n",
    );
    for (table, fields) in [
        (
            "canonical_studies",
            "key,run,problem,revision,physical_revision,physical_identity,context,point_count,generation,active,cancelled,terminal",
        ),
        (
            "canonical_study_points",
            "key,study,ordinal,occurrence,run,attempt,revision,settled,facts",
        ),
        (
            "canonical_runs",
            "key,problem,revision,terminal_attempt,terminal_class,research_selection,research_dataset",
        ),
        (
            "canonical_attempts",
            "key,run,phase,closed_manifest,generation",
        ),
        ("canonical_result_manifests", "key,attempt,generation"),
        ("canonical_result_retirements", "key"),
        ("canonical_project_branches", "key,project,problem"),
        ("canonical_research_references", "key,run,revision"),
        ("canonical_research_publications", "key,selection,revision"),
        ("canonical_history_retirements", "key"),
    ] {
        let projection = |side: &str| {
            fields
                .split(',')
                .map(|field| format!("${side}.{field}"))
                .collect::<Vec<_>>()
                .join(",")
        };
        ddl.push_str(&format!(
            "DEFINE EVENT study_publication_witness ON TABLE {table} WHEN [{}]!=[{}] THEN {{ fn::pse_workbench_v1::changed(); }};\n",
            projection("before"), projection("after"),
        ));
    }
}
