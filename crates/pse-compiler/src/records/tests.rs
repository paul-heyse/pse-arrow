// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Terminal records preserve actual leaf diagnostics and nested values.
#![allow(clippy::expect_used, reason = "explicit diagnostic fixtures")]
use super::{Observation, collect_findings};
use crate::{CompilerError, PassStatus};
use pse_ids::SemanticId;
use pse_schema::model::Cell;

fn finding(index: u8) -> Vec<Cell> {
    use pse_relations::generated::{
        enums::FindingSeverity, runtime::diagnostics_findings as findings,
    };
    findings::Row {
        finding_id: SemanticId::from_bytes([index; 16]),
        subject_snapshot: None,
        run_id: None,
        check_id: SemanticId::from_bytes([4; 16]),
        severity: FindingSeverity::Error,
        subjects: vec![SemanticId::from_bytes([7; 16])],
        evidence: findings::RuntimeDiagnosticsFindingsFieldEvidence::from_row(
            findings::RuntimeDiagnosticsFindingsFieldEvidenceRow {
                relation_id: SemanticId::from_bytes([8; 16]),
                row_key: pse_ids::ContentHash::from_bytes([index; 32]),
            },
        ),
        message: format!("actual rule finding {index}"),
        next_steps: vec!["repair the actual input".into()],
    }
    .into_cells()
}
#[test]
fn mixed_aggregate_preserves_every_actual_finding_and_first_non_cancel_class() {
    let registry = pse_schema::catalog::assemble().expect("complete registry");
    let actual = vec![finding(1), finding(2)];
    let batch = pse_relations::cells::batch_from_cells(
        &registry,
        registry
            .relation("runtime.diagnostics_findings")
            .expect("finding schema"),
        &actual,
    )
    .expect("actual typed rule findings");
    let error = CompilerError::Rule(pse_rules::RuleError::Collection {
        errors: vec![
            pse_rules::RuleError::Catalog(pse_catalog::CatalogError::Cancelled),
            pse_rules::RuleError::InvariantViolations {
                count: 2,
                findings: vec![batch],
            },
            pse_rules::RuleError::Infrastructure {
                op: "after collection".to_owned(),
                detail: "store unavailable".to_owned(),
            },
        ],
    });
    let observation = Observation::success(None);
    let report =
        collect_findings(&registry, &observation, Some(&error)).expect("complete aggregate");
    assert_eq!(report.status, PassStatus::Failed);
    assert_eq!(report.failure_class, Some("validation.invariant"));
    assert_eq!(report.findings.len(), 4);
    assert_eq!(
        report.findings[1..3],
        actual.into_iter().map(Cell::Struct).collect::<Vec<_>>()
    );
    for row in [&report.findings[0], &report.findings[3]] {
        use pse_relations::{generated::provenance::pass_records as record, typed::CellCodec};
        let finding = record::ProvenancePassRecordsFieldFindingsItem::from_cell(row.clone())
            .expect("typed finding");
        assert!(finding.check_id.is_none());
        let record::ProvenancePassRecordsFieldFindingsItemEvidenceSelected::Execution(evidence) =
            finding.evidence.selected().expect("selected evidence")
        else {
            panic!("actual execution evidence")
        };
        assert!(
            evidence
                .diagnostic_code
                .as_ref()
                .is_some_and(|code| code.contains("runtime::"))
        );
        assert!(!evidence.attempt_error.is_empty());
    }
    let attempt = super::Attempt::new(registry.pass("P3").expect("pass"));
    let record = vec![
        Cell::Id(attempt.id),
        Cell::Id(attempt.pass),
        Cell::text(attempt.version),
        Cell::Null,
        Cell::Null,
        Cell::Null,
        Cell::List(vec![]),
        Cell::List(vec![]),
        Cell::List(vec![]),
        Cell::F64(1.0),
        Cell::I64(4),
        Cell::Enum(report.status.as_str()),
        Cell::List(report.findings),
        Cell::Enum(report.failure_class.expect("failed class")),
        Cell::List(vec![]),
    ];
    let spec = registry
        .relation("provenance.pass_records")
        .expect("record declaration");
    let batch =
        pse_relations::cells::batch_from_cells(&registry, spec, std::slice::from_ref(&record))
            .expect("complete typed record");
    assert_eq!(
        pse_relations::cells::cells_from_batch(&registry, spec, &batch)
            .expect("decode nested record"),
        vec![record]
    );
}
#[test]
fn all_actual_cancellation_leaves_remain_cancelled() {
    let registry = pse_schema::catalog::assemble().expect("complete registry");
    let error = CompilerError::Catalog(pse_catalog::CatalogError::Multiple {
        errors: vec![
            pse_catalog::CatalogError::Cancelled,
            pse_catalog::CatalogError::Canon(pse_ids::CanonError::Cancelled),
        ],
    });
    let report = collect_findings(&registry, &Observation::success(None), Some(&error))
        .expect("both cancellations retained");
    assert_eq!(report.status, PassStatus::Cancelled);
    assert_eq!(report.failure_class, Some("runtime.cancelled"));
    assert_eq!(report.findings.len(), 2);
}

#[test]
fn native_failures_preserve_cancellation_and_kernel_identity() {
    let registry = pse_schema::catalog::assemble().expect("complete registry");
    let method = SemanticId::from_bytes([61; 16]);
    let kernel = SemanticId::from_bytes([62; 16]);
    for (error, expected_status, expected_class) in [
        (
            CompilerError::Canon(pse_ids::CanonError::Cancelled),
            PassStatus::Cancelled,
            "runtime.cancelled",
        ),
        (
            CompilerError::KernelUnbound {
                method_id: method,
                kernel_id: Some(kernel),
                detail: "actual descriptor lacks executable binding".to_owned(),
            },
            PassStatus::Failed,
            "kernel.unbound_parameter",
        ),
    ] {
        let report = collect_findings(&registry, &Observation::success(None), Some(&error))
            .expect("exact native diagnostic");
        assert_eq!(report.status, expected_status);
        assert_eq!(report.failure_class, Some(expected_class));
        assert_eq!(report.findings.len(), 1);
        if expected_class == "kernel.unbound_parameter" {
            let text = report.findings[0].literal_spec();
            assert!(text.contains(&method.to_string()));
            assert!(text.contains(&kernel.to_string()));
        }
    }
}
