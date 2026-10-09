// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared admitted scientific sources for study adapters and scalar preparation.
use super::WorkflowRuntime;
use pse_runtime::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_runtime::workflow::{ModelingPackage, PackageSources, PhysicalContext, Runtime};
use std::{collections::BTreeMap, path::Path};

pub(super) const SOURCE: &str = "package k4 { def Root { param a:Scalar=2; var x:Scalar; var y:Scalar; eq first:x==a; eq second:y==3; annotation start x(1); annotation start y(1); annotation report x(\"x\"); annotation check x(abs(x-a)<1e-7); annotation check y(abs(y-3)<1e-7); } }";

pub(super) fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}
fn documents(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut result = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                result.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                    std::fs::read(&path).unwrap(),
                );
            }
        }
    }
    result
}
pub(super) fn sources(source: &str) -> PackageSources {
    let root = repository().join("tests/fixtures/packages");
    let manifest = std::fs::read_to_string(root.join("minimal_explicit/package.toml")).unwrap()
        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
        .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
    PackageSources {
        physical: documents(&root.join("physical-primitives")),
        modeling: vec![BTreeMap::from([
            ("package.toml".into(), manifest.into_bytes()),
            ("models/root.pse".into(), source.as_bytes().to_vec()),
        ])],
    }
}
pub(super) fn admitted_documents(
    owner: &WorkflowRuntime,
    sources: &[BTreeMap<String, Vec<u8>>],
) -> OwnedDocumentSet {
    let pool = owner.runtime.pool();
    let validation = owner.sessions.validation_context(&owner.registry).unwrap();
    let bundles = sources
        .iter()
        .map(|texts| {
            load_package_documents_owned(
                texts,
                &owner.registry,
                Default::default(),
                &pool,
                &owner.cancel,
                &validation,
            )
            .unwrap()
        })
        .collect();
    OwnedDocumentSet::try_from_bundles(bundles, &pool, &owner.cancel).unwrap()
}
pub(super) async fn physical(
    runtime: &Runtime,
    owner: &WorkflowRuntime,
    sources: &PackageSources,
) -> PhysicalContext {
    runtime
        .physical_from_documents(
            &admitted_documents(owner, std::slice::from_ref(&sources.physical)),
            &owner.cancel,
        )
        .await
        .unwrap()
}
#[allow(
    dead_code,
    reason = "study adapter admits physical context explicitly for typed assignments"
)]
pub(super) async fn package(
    runtime: &Runtime,
    owner: &WorkflowRuntime,
    sources: &PackageSources,
) -> ModelingPackage {
    let physical = physical(runtime, owner, sources).await;
    runtime
        .modeling_from_documents(
            &admitted_documents(owner, &sources.modeling),
            physical,
            &owner.cancellation,
        )
        .await
        .unwrap()
}
pub(super) fn counts(
    before: pse_runtime::math::PreparationCounts,
    after: pse_runtime::math::PreparationCounts,
) -> serde_json::Value {
    serde_json::json!({"views":after.views-before.views,"observations":after.observations-before.observations,"rebuilt":after.rebuilt-before.rebuilt,"shared":after.shared-before.shared})
}
