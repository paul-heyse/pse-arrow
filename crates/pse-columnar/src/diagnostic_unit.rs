// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::expect_used,
    reason = "isolated adversarial contract units"
)]
use super::*;
use datafusion_common::DataFusionError;
use miette::Diagnostic;
use pse_diagnostics::diagnostic_leaves;
use std::{error::Error, sync::Arc};

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("domain failure {0}")]
#[diagnostic(help("preserved domain help"))]
struct Domain(u64);
impl TypedDiagnostic for Domain {
    fn diagnostic_code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode::CompileProperty)
    }
}

#[test]
fn shared_nested_causes_context_and_multiplicity_survive() {
    let native = Arc::new(DataFusionError::External(Box::new(Attachment::new(
        DiagnosticCode::CompileProperty,
        Domain(42),
    ))));
    let tree = Arc::new(DataFusionError::Context(
        "outer".into(),
        Box::new(DataFusionError::Collection(vec![
            DataFusionError::Shared(Arc::clone(&native)),
            DataFusionError::Shared(Arc::clone(&native)),
            DataFusionError::Collection(vec![]),
        ])),
    ));
    let error = EngineError::from_shared(Arc::clone(&tree), PlanOrigin::KernelUdf);
    assert!(Arc::ptr_eq(error.native(), &tree));
    assert_eq!(error.failures().len(), 2);
    for observation in error.observations() {
        assert_eq!(observation.code, DiagnosticCode::CompileProperty);
        assert_eq!(observation.contexts, ["outer"]);
        assert!(std::ptr::eq(observation.cause, native.as_ref()));
    }
    for failure in error.failures() {
        let DataFusionError::External(source) = failure.cause() else {
            unreachable!()
        };
        let attachment = source.downcast_ref::<Attachment>().unwrap();
        let cause: &(dyn Error + 'static) = attachment.cause();
        assert_eq!(cause.downcast_ref::<Domain>().unwrap().0, 42);
        assert_eq!(
            attachment
                .source()
                .unwrap()
                .downcast_ref::<Domain>()
                .unwrap()
                .0,
            42
        );
        assert_eq!(failure.help().unwrap().to_string(), "preserved domain help");
    }
}

#[test]
fn empty_collection_is_a_failed_result_without_invented_leaves() {
    let error = classify(DataFusionError::Collection(vec![]), PlanOrigin::Analytics);
    assert!(error.failures().is_empty());
    assert!(error.code().is_none());
    assert!(matches!(
        error.native().as_ref(),
        DataFusionError::Collection(_)
    ));
}

#[test]
fn standard_error_walk_reaches_the_domain_through_native_context_and_shared_wrappers() {
    let error = classify(
        DataFusionError::Context(
            "caller".into(),
            Box::new(DataFusionError::Shared(Arc::new(
                DataFusionError::External(Box::new(Attachment::new(
                    DiagnosticCode::CompileProperty,
                    Domain(42),
                ))),
            ))),
        ),
        PlanOrigin::Analytics,
    );
    let mut cause: &dyn Error = &error;
    loop {
        if let Some(domain) = cause.downcast_ref::<Domain>() {
            assert_eq!(domain.0, 42);
            break;
        }
        cause = cause
            .source()
            .expect("concrete domain retained in standard source chain");
    }
}

#[test]
fn origin_and_typed_codes_replace_message_heuristics() {
    let execution = DataFusionError::Execution("compile.property text is not authority".into());
    assert_eq!(
        observe(&execution, PlanOrigin::KernelUdf)[0].code,
        DiagnosticCode::SolveEvaluationError
    );
    let plan = DataFusionError::Plan("bad plan".into());
    assert_eq!(
        observe(&plan, PlanOrigin::Analytics)[0].code,
        DiagnosticCode::UserModel
    );
    assert_eq!(
        observe(&plan, PlanOrigin::RuleCompiler)[0].code,
        DiagnosticCode::InternalInvariant
    );
}

#[test]
fn unknown_external_remains_downcastable_and_native_diagnostic_survives() {
    let source =
        DataFusionError::External(Box::new(std::io::Error::other("io failure"))).with_diagnostic(
            datafusion_common::diagnostic::Diagnostic::new_error("SQL attribution", None),
        );
    let error = classify(source, PlanOrigin::Analytics);
    let observed = error.observations();
    assert_eq!(observed[0].diagnostics[0].message, "SQL attribution");
    let DataFusionError::External(source) = observed[0].cause else {
        unreachable!()
    };
    assert!(source.downcast_ref::<std::io::Error>().is_some());
}

#[test]
fn every_declared_code_roundtrips_and_has_a_declared_class() {
    for code in DiagnosticCode::ALL {
        assert_eq!(DiagnosticCode::parse(code.as_str()), Some(*code));
        assert_eq!(DiagnosticCode::parse(&code.to_string()), Some(*code));
        assert!(FailureClass::ALL.contains(&code.class()));
    }
}

#[test]
fn external_native_trees_keep_inner_context_and_collection_multiplicity() {
    let source = DataFusionError::External(Box::new(DataFusionError::Context(
        "external context".into(),
        Box::new(DataFusionError::Collection(vec![
            DataFusionError::ResourcesExhausted("pool".into()),
            DataFusionError::Plan("invalid plan".into()),
        ])),
    )));
    let error = classify(source, PlanOrigin::Analytics);
    let observations = error.observations();
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].contexts, ["external context"]);
    assert_eq!(observations[0].code, DiagnosticCode::RuntimeResourceLimit);
    assert_eq!(observations[1].code, DiagnosticCode::UserModel);
    assert!(matches!(
        error.failures()[0].cause(),
        DataFusionError::ResourcesExhausted(_)
    ));
}

#[test]
fn arrow_external_wrappers_preserve_mixed_leaves_and_original_sources() {
    let source = DataFusionError::ArrowError(
        Box::new(arrow::error::ArrowError::ExternalError(Box::new(
            DataFusionError::Context(
                "arrow boundary".into(),
                Box::new(DataFusionError::Collection(vec![
                    DataFusionError::ResourcesExhausted("pool".into()),
                    DataFusionError::Execution("kernel".into()),
                ])),
            ),
        ))),
        None,
    );
    let error = classify(source, PlanOrigin::KernelUdf);
    assert_eq!(error.diagnostic_code(), None);
    let leaves = diagnostic_leaves(&error);
    assert_eq!(
        leaves
            .iter()
            .map(|leaf| leaf.diagnostic_code())
            .collect::<Vec<_>>(),
        [
            Some(DiagnosticCode::RuntimeResourceLimit),
            Some(DiagnosticCode::SolveEvaluationError),
        ]
    );
    assert!(matches!(
        error.failures()[0].cause(),
        DataFusionError::ResourcesExhausted(_)
    ));
    assert_eq!(error.observations()[1].contexts, ["arrow boundary"]);
}

#[test]
fn homogeneous_aggregate_has_one_identity_but_retains_all_occurrences() {
    let error = classify(
        DataFusionError::Collection(vec![
            DataFusionError::ResourcesExhausted("first".into()),
            DataFusionError::ResourcesExhausted("second".into()),
        ]),
        PlanOrigin::Analytics,
    );
    assert_eq!(
        error.diagnostic_code(),
        Some(DiagnosticCode::RuntimeResourceLimit)
    );
    assert_eq!(diagnostic_leaves(&error).len(), 2);
    assert!(
        diagnostic_leaves(&classify(
            DataFusionError::Collection(vec![]),
            PlanOrigin::Analytics
        ))
        .is_empty()
    );
}

#[test]
fn typed_platform_aggregates_survive_repeated_external_boundaries() {
    let mixed = classify(
        DataFusionError::Collection(vec![
            DataFusionError::ResourcesExhausted("pool".into()),
            DataFusionError::Execution("kernel".into()),
        ]),
        PlanOrigin::KernelUdf,
    );
    let wrapped = classify(external(mixed), PlanOrigin::Analytics);
    assert_eq!(wrapped.diagnostic_code(), None);
    assert_eq!(
        wrapped
            .failures()
            .iter()
            .map(Failure::diagnostic_code)
            .collect::<Vec<_>>(),
        [
            DiagnosticCode::RuntimeResourceLimit,
            DiagnosticCode::SolveEvaluationError
        ]
    );
    assert!(matches!(
        wrapped.failures()[0]
            .leaf_cause()
            .downcast_ref::<DataFusionError>(),
        Some(DataFusionError::ResourcesExhausted(_))
    ));
    let empty = classify(
        external(classify(
            DataFusionError::Collection(vec![]),
            PlanOrigin::Analytics,
        )),
        PlanOrigin::KernelUdf,
    );
    assert!(empty.failures().is_empty());
}
