// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public exact-parent source reload and independent fresh admission, without a store.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "native source fixture assertions"
)]
#[path = "reference_support/mod.rs"]
mod reference_support;
use futures_util::FutureExt;
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_runtime::{
    CancelSource,
    workflow::{ModelingPackage, SourceDelta},
};
use reference_support::{NativeContext, Result, root};
use std::{collections::BTreeMap, panic::AssertUnwindSafe};

const SOURCE: &str = "package reload {param a:Scalar=1;}";
fn sources() -> Result<BTreeMap<String, Vec<u8>>> {
    let manifest = std::fs::read_to_string(root().join("tests/fixtures/packages/minimal_explicit/package.toml"))?
        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
        .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
    Ok(BTreeMap::from([
        ("package.toml".into(), manifest.into_bytes()),
        ("models/model.pse".into(), SOURCE.as_bytes().to_vec()),
    ]))
}
async fn fresh(
    context: &NativeContext,
    original: &ModelingPackage,
    texts: &BTreeMap<String, Vec<u8>>,
) -> Result<ModelingPackage> {
    let pool = context.shared.pool();
    let token = pse_columnar::CancellationToken::new();
    let documents = OwnedDocumentSet::try_from_bundles(
        vec![load_package_documents_owned(
            texts,
            context.runtime.registry(),
            Default::default(),
            &pool,
            &token,
            &context.runtime.validation_context(),
        )?],
        &pool,
        &token,
    )?;
    Ok(Box::pin(context.runtime.modeling_from_documents(
        &documents,
        original.physical_context().clone(),
        &CancelSource::new(),
    ))
    .await?)
}
async fn inventory(
    context: &NativeContext,
    package: &ModelingPackage,
) -> Result<
    BTreeMap<pse_model::generated::identities::DeclarationId, pse_authoring::language::Declaration>,
> {
    let rows = package
        .declarations(&context.runtime.preparation_services())
        .await?;
    let keyed = rows
        .iter()
        .cloned()
        .map(|row| (row.declaration_id, row))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(keyed.len(), rows.len(), "source identities remain unique");
    Ok(keyed)
}
async fn revision(
    context: &NativeContext,
    package: &ModelingPackage,
) -> Result<pse_ids::roles::SourceRevisionHash> {
    Ok(package
        .knowledge(
            &context.runtime.preparation_services(),
            None,
            4096,
            4 << 20,
            &pse_columnar::CancellationToken::new(),
        )
        .await?
        .source_revision())
}

#[tokio::test(flavor = "multi_thread")]
async fn public_parsed_reload_preserves_exact_parent_and_complete_immutable_history() -> Result<()>
{
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let package = context.package(SOURCE).await?;
        let services = context.runtime.source_services();
        let cancel = CancelSource::new();
        let original = sources()?;
        let original_rows = inventory(&context, &package).await?;
        let original_source = revision(&context, &package).await?;
        let same = Box::pin(package.reload_from_parent(
            &services,
            std::slice::from_ref(&original),
            &cancel,
        ))
        .await?;
        assert_eq!(same.revision_id(), package.revision_id());
        assert_eq!(inventory(&context, &same).await?, original_rows);
        let mut expanded = original.clone();
        expanded.insert(
            "models/a.pse".into(),
            b"package earlier {param b:Scalar=2;}".to_vec(),
        );
        let changed =
            Box::pin(package.reload_from_parent(&services, &[expanded.clone()], &cancel)).await?;
        let independent = fresh(&context, &package, &expanded).await?;
        assert_ne!(changed.revision_id(), package.revision_id());
        let changed_rows = inventory(&context, &changed).await?;
        let changed_source = revision(&context, &changed).await?;
        assert_eq!(changed_rows, inventory(&context, &independent).await?);
        assert_eq!(changed_source, revision(&context, &independent).await?);
        assert_ne!(changed_source, original_source);
        assert!(
            changed_rows
                .values()
                .any(|row| row.name == "earlier" && row.parent_id.is_none())
        );
        let restored = Box::pin(changed.reload_from_parent(
            &services,
            std::slice::from_ref(&original),
            &cancel,
        ))
        .await?;
        assert_eq!(inventory(&context, &restored).await?, original_rows);
        assert_eq!(revision(&context, &restored).await?, original_source);
        assert_eq!(inventory(&context, &package).await?, original_rows);
        assert_eq!(revision(&context, &package).await?, original_source);
        assert_eq!(inventory(&context, &changed).await?, changed_rows);
        assert_eq!(revision(&context, &changed).await?, changed_source);
        let mut malformed = original.clone();
        malformed.insert(
            "models/model.pse".into(),
            b"package reload {def Broken {eq e:1 +;}}".to_vec(),
        );
        let error = Box::pin(package.reload_from_parent(&services, &[malformed], &cancel))
            .await
            .unwrap_err();
        let diagnostic = error.boundary_diagnostic();
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::InvalidModel,
            "{error}"
        );
        assert!(
            package
                .with_delta(
                    &services,
                    SourceDelta {
                        parent: "different-canonical-parent".into(),
                        ..Default::default()
                    },
                    &cancel
                )
                .await
                .is_err()
        );
        assert_eq!(inventory(&context, &package).await?, original_rows);
        assert_eq!(revision(&context, &package).await?, original_source);
        assert_eq!(
            Box::pin(package.reload_from_parent(&services, &[original], &cancel))
                .await?
                .revision_id(),
            package.revision_id()
        );
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })
    .catch_unwind()
    .await;
    let cleanup = context.close().await;
    match body {
        Ok(result) => {
            result?;
            cleanup
        }
        Err(panic) => {
            let _ = cleanup;
            std::panic::resume_unwind(panic)
        }
    }
}
