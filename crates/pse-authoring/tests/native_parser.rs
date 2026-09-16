// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native parser failures remain execution failures with their concrete source causes.
#![allow(clippy::unwrap_used, reason = "integration assertions")]

use datafusion::{execution::context::SessionContext, physical_plan::collect};
use pse_authoring::{AuthoringError, ParseBudget};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::generated::authored::{documents, packages};

fn rows(mutation: &str) -> Vec<documents::Row> {
    let package = SemanticId::parse_hex("01991d6a13a070008000000000000001").unwrap();
    let source = include_str!("../../../tests/fixtures/packages/minimal_explicit/package.toml");
    let mut row = documents::Row {
        document_id: pse_ids::named_id(package, "package.toml"),
        package_id: package,
        path: "package.toml".into(),
        source_text: source.into(),
    };
    match mutation {
        "syntax" => row.source_text = "[package\n".into(),
        "package" => row.package_id = SemanticId::NIL,
        "identity" => row.document_id = SemanticId::NIL,
        _ => {}
    }
    if mutation == "duplicate" {
        vec![row.clone(), row]
    } else {
        vec![row]
    }
}

#[tokio::test]
async fn native_parser_defers_invalid_sources_and_retains_typed_failures() {
    for mutation in ["syntax", "package", "identity", "duplicate", "budget"] {
        let context = SessionContext::new();
        let mut builder = documents::Builder::new().unwrap();
        for row in rows(mutation) {
            builder.push(row).unwrap();
        }
        let source = context
            .read_batch(builder.finish().unwrap().into_batch())
            .unwrap()
            .into_unoptimized_plan();
        let resources = FixedBudget::new(128 << 20);
        let budget = if mutation == "budget" {
            ParseBudget {
                max_bytes: 1,
                ..ParseBudget::default()
            }
        } else {
            ParseBudget::default()
        };
        let plan = pse_authoring::native::relation_plan(
            source,
            packages::RELATION_ID,
            &pse_schema::shared_registry().unwrap(),
            budget,
            resources.clone(),
            CancellationToken::default(),
        )
        .unwrap();
        assert!(plan.display_indent().to_string().contains("pse_parse_"));
        let state = context.state();
        let physical = state.create_physical_plan(&plan).await.unwrap();
        assert_eq!(resources.reserved(), 0, "planning must not run the parser");
        let error = collect(physical, state.task_ctx()).await.unwrap_err();
        let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(&error);
        let mut typed = false;
        while let Some(error) = cause {
            typed |= error.downcast_ref::<AuthoringError>().is_some();
            cause = error.source();
        }
        assert!(typed, "parser lost its structured cause: {error}");
        assert_eq!(
            resources.reserved(),
            0,
            "failed parsing releases its workspace"
        );
    }
}

#[tokio::test]
async fn native_parser_observes_cancellation_and_shared_resource_limits() {
    for cancelled in [false, true] {
        let context = SessionContext::new();
        let mut builder = documents::Builder::new().unwrap();
        builder.push(rows("").remove(0)).unwrap();
        let input = context
            .read_batch(builder.finish().unwrap().into_batch())
            .unwrap()
            .into_unoptimized_plan();
        let cancel = CancellationToken::default();
        let resources = FixedBudget::new(if cancelled { 128 << 20 } else { 1 });
        let plan = pse_authoring::native::relation_plan(
            input,
            packages::RELATION_ID,
            &pse_schema::shared_registry().unwrap(),
            ParseBudget::default(),
            resources.clone(),
            cancel.clone(),
        )
        .unwrap();
        let state = context.state();
        let physical = state.create_physical_plan(&plan).await.unwrap();
        if cancelled {
            cancel.cancel();
        }
        assert!(collect(physical, state.task_ctx()).await.is_err());
        assert_eq!(resources.reserved(), 0);
    }
}
