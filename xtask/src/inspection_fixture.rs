// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fresh target Delta publications for cold Rust/Python inspection.
use anyhow::{Context, Result, ensure};
use pse_relations::generated::enums::PublicationKind;
use std::{path::Path, process::Command};
pub(crate) mod environment;
mod index;
use environment::Environment;
use index::Index;

pub(crate) fn run(path: &Path) -> Result<()> {
    std::fs::create_dir(path)
        .with_context(|| format!("creating fresh publication fixture {}", path.display()))?;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    executor.block_on(async {
        let environment = Environment::new(path)?;
        let plan = environment
            .source_plan(&[])
            .await
            .context("compose native inspection source plan")?;
        let root = environment
            .publish(path, &plan, PublicationKind::Relations)
            .await
            .context("publish native inspection source plan")?;
        drop((plan, environment));
        let reader = Environment::new(path)?;
        let publication = reader
            .open(root.clone())
            .await
            .context("reopen native inspection source publication")?;
        let tables = publication.session().inspection_tables();
        ensure!(
            !tables.is_empty(),
            "source publication omitted its complete typed inventory"
        );
        let index = Index { root, tables };
        let mut bytes = serde_json::to_vec_pretty(&index)?;
        bytes.push(b'\n');
        std::fs::write(path.join("publication-index.json"), bytes)?;
        Ok(())
    })
}
pub(crate) fn python_tests(root: &Path, args: &[String]) -> Result<()> {
    let scratch = tempfile::tempdir()?;
    let path = scratch.path().join("inspection");
    let fixture = run(&path);
    if let Err(error) = &fixture {
        eprintln!("inspection fixture failed; continuing independent Python tests: {error:#}");
    }
    let status = Command::new("uv")
        .current_dir(root)
        .args([
            "run",
            "--no-sync",
            "pytest",
            "python/pse/tests",
            "--maxfail=0",
            "--continue-on-collection-errors",
            "-m",
            "unit or component",
            "-n",
            "auto",
        ])
        .args(args)
        .env("PSE_INSPECTION_PUBLICATION", &path)
        .status()
        .context("running Python tests against fresh native Delta publication")?;
    ensure!(
        fixture.is_ok() && status.success(),
        "Python tests: {status}; fixture: {fixture:?}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn empty_source_outputs_have_executable_declared_native_fields() {
        let directory = tempfile::tempdir().unwrap();
        let environment = Environment::new(directory.path()).unwrap();
        let plan = environment.source_plan(&[]).await.unwrap();
        for (name, output) in plan.outputs() {
            let prepared = plan.prepare(name, &environment.cancel).unwrap();
            let completed = prepared
                .execute(&environment.cancel)
                .await
                .unwrap_or_else(|error| panic!("source output {name}: {error}"));
            let spec = environment
                .registry
                .relation_by_id(output.relation_id)
                .unwrap();
            let layout = pse_catalog::delta::layout::DurableLayout::new(Arc::new(
                pse_schema::arrow::relation_schema(&environment.registry, spec).unwrap(),
            ))
            .unwrap();
            assert_eq!(
                output.plan.schema().as_arrow(),
                layout.execution_schema().as_ref(),
                "source output {name} retains its complete declaration"
            );
            let encoded = layout.encode(output.plan.clone()).unwrap();
            let encoded = plan
                .session()
                .prepare_rule_plan(encoded, &environment.cancel)
                .unwrap();
            encoded
                .execute(&environment.cancel)
                .await
                .unwrap_or_else(|error| panic!("source output {name} storage projection: {error}"));

            completed
                .into_checked_relation(&environment.registry, spec, &environment.cancel)
                .unwrap_or_else(|error| panic!("source output {name} declaration: {error}"));
        }
    }
}
