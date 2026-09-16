// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordinary columnar package admission before the pure leaf fixture projection.

use std::{collections::BTreeMap, num::NonZeroUsize, path::Path, sync::Arc};

use anyhow::{Context, Result, ensure};
use pse_authoring::{
    ParseBudget,
    document::{OwnedDocumentSet, load_package},
};
use pse_catalog::session::{
    ExecutionSettings, SnapshotSession, ThreadBudget, native_engine_profile,
};
use pse_compiler::quantity_relations::PhysicalInventory;
use pse_ids::CancellationToken;
use pse_relations::{columnar::FieldCheckedBatch, generated::authored};
use pse_runtime::{ResourceBudget, SharedRuntime};
use pse_schema::{
    Registry,
    codegen::GeneratedTree,
    model::{Namespace, RelationKey, SnapshotClass},
};
use serde::Deserialize;

#[cfg(test)]
mod tests;

type Physical = PhysicalInventory;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Projection {
    packages: Vec<String>,
}

pub(super) fn append(root: &Path, registry: &Registry, tree: &mut GeneratedTree) -> Result<()> {
    let physical = load(root, registry)?;
    pse_schema::codegen::rust::physical::append_quantity_fixture(
        tree,
        physical.quantities(),
        physical.preconditions(),
    )?;
    pse_schema::codegen::rust::physical::append_element_fixture(tree, physical.elements())?;
    Ok(())
}

fn load(root: &Path, registry: &Registry) -> Result<Physical> {
    let configuration = root.join("packages/reference/fixture-projection.toml");
    let projection: Projection = toml::from_str(&std::fs::read_to_string(&configuration)?)
        .context("reading explicit physical fixture package inventory")?;
    ensure!(
        !projection.packages.is_empty(),
        "physical fixture package selection is empty"
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let source = Sources::load(root, &projection.packages, registry).await?;
        let physical = PhysicalInventory::load(
            &source.session,
            source.session.registry(),
            &CancellationToken::new(),
        )
        .await?;
        ensure!(
            physical.neutral().is_some(),
            "physical fixture must declare an explicit neutral quantity"
        );
        ensure!(
            physical.elements().elements().len() != 0,
            "physical fixture must declare actual elements"
        );
        Ok(physical)
    })
}

/// Build-tool ownership only: original source bytes, checked columns and their one engine.
/// The semantic inventory is constructed solely by the production native adapter.
struct Sources {
    session: SnapshotSession,
    _documents: OwnedDocumentSet,
    _checked: BTreeMap<RelationKey, FieldCheckedBatch>,
    _runtime: Arc<SharedRuntime>,
    _spill: tempfile::TempDir,
}
impl Sources {
    async fn load(root: &Path, packages: &[String], declaration: &Registry) -> Result<Self> {
        let registry = Arc::new(pse_schema::catalog::assemble()?);
        let spill = tempfile::tempdir()?;
        let one = NonZeroUsize::new(1).context("positive thread count")?;
        let runtime = SharedRuntime::build(ResourceBudget {
            memory_limit_bytes: NonZeroUsize::new(32usize << 30).context("positive tool budget")?,
            spill_dir: spill.path().to_path_buf(),
            max_temp_dir_bytes: 32 << 30,
            top_consumers: NonZeroUsize::new(16).context("positive consumer count")?,
            threads: ThreadBudget {
                pool_threads: one,
                target_partitions: one,
            },
            execution: ExecutionSettings::default(),
            hashing_may_use_pool: false,
        })?;
        let cancel = CancellationToken::new();
        let reserver = runtime.reserver();
        let mut bundles = Vec::new();
        for path in packages {
            ensure!(
                super::normal_relative(Path::new(path)),
                "invalid physical package path"
            );
            bundles.push(load_package(
                &root.join(path),
                declaration,
                ParseBudget::default(),
            )?);
        }
        let documents = pse_authoring::document::load_bundles_owned(
            &bundles,
            declaration,
            reserver.as_ref(),
            &cancel,
        )?;
        let mut checked = BTreeMap::new();
        for spec in registry.relations().iter().filter(|spec| {
            spec.snapshot_class == SnapshotClass::Model
                && matches!(
                    spec.key.namespace,
                    Namespace::Authored | Namespace::Reference
                )
        }) {
            let parts = documents
                .bundles()
                .iter()
                .filter_map(|bundle| bundle.batches.get(&spec.id))
                .cloned()
                .collect::<Vec<_>>();
            let batch = FieldCheckedBatch::concat_reserved(
                &registry,
                spec,
                &parts,
                reserver.as_ref(),
                &cancel,
            )?;
            checked.insert(spec.key, batch);
        }
        // Registry self-description comes from the registry generator, not authored defaults.
        for (key, batch) in pse_relations::registry_relations::materialize(&registry)? {
            let spec = registry
                .relation(&key.qualified_name())
                .context("registry source declaration")?;
            checked.insert(
                key,
                FieldCheckedBatch::admit(&registry, spec, batch)?
                    .retained(reserver.as_ref(), &cancel)?,
            );
        }
        let session = runtime
            .session_factory(native_engine_profile())?
            .candidate_checked(checked.clone(), registry, &cancel)?;
        let packages = checked
            .get(&authored::packages::spec(session.registry())?.key)
            .context("package headers absent")?;
        pse_authoring::p0::resolve(packages, &session, &cancel).await?;
        let rows = checked
            .iter()
            .map(|(key, batch)| (*key, batch.batch().clone()))
            .collect();
        let report = pse_rules::invariants::run_invariants(
            &rows,
            &session,
            session.registry(),
            pse_rules::invariants::InvariantScope::Model,
            None,
            &cancel,
        )
        .await?;
        if report.error_count() != 0 {
            let mut messages = Vec::new();
            for batch in report.findings() {
                for row in
                    pse_relations::generated::runtime::diagnostics_findings::View::from_checked(
                        batch,
                    )?
                    .rows()?
                {
                    messages.push(format!(
                        "{}: {} ({:?})",
                        row.check_id, row.message, row.evidence
                    ));
                }
            }
            anyhow::bail!(
                "physical source registry obligations failed: {}",
                messages.join("; ")
            );
        }
        Ok(Self {
            session,
            _documents: documents,
            _checked: checked,
            _runtime: runtime,
            _spill: spill,
        })
    }
}
