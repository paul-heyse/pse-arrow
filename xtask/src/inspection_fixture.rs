// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fresh exported publications for cold Rust/Python inspection.
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
        let manifest = environment
            .publish(path, &plan, PublicationKind::Relations)
            .await
            .context("publish native inspection source plan")?;
        drop((plan, environment));
        let reader = Environment::new(path)?;
        let publication = reader
            .open(manifest.clone())
            .await
            .context("reopen native inspection source publication")?;
        let tables = publication.session().inspection_tables();
        ensure!(
            !tables.is_empty(),
            "source publication omitted its complete typed inventory"
        );
        let index = Index { manifest, tables };
        let mut bytes = serde_json::to_vec_pretty(&index)?;
        bytes.push(b'\n');
        std::fs::write(path.join("publication-index.json"), bytes)?;
        Ok(())
    })
}
fn needs_publication(args: &[String]) -> bool {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--collect-only" | "--co"))
    {
        return false;
    }
    // The exact explicit unit route is independent. Other framework predicates
    // retain conservative setup; this is not a marker-expression parser.
    let mut mark = None;
    let mut arguments = args.iter();
    while let Some(argument) = arguments.next() {
        if argument == "-m" || argument == "--markexpr" {
            mark = arguments.next().map(String::as_str);
        } else if let Some(value) = argument.strip_prefix("--markexpr=") {
            mark = Some(value);
        }
    }
    mark != Some("unit")
}

pub(crate) fn python_tests(root: &Path, args: &[String]) -> Result<()> {
    let scratch = needs_publication(args)
        .then(tempfile::tempdir)
        .transpose()?;
    let path = scratch
        .as_ref()
        .map(|scratch| scratch.path().join("inspection"));
    let fixture = path.as_ref().map(|path| run(path));
    if let Some(Err(error)) = &fixture {
        eprintln!("inspection fixture failed; continuing independent Python tests: {error:#}");
    }
    let mut command = Command::new("uv");
    command
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
        .args(args);
    if let Some(path) = &path {
        command.env("PSE_INSPECTION_PUBLICATION", path);
    }
    let status = command.status().context("running selected Python tests")?;
    ensure!(
        fixture.as_ref().is_none_or(|fixture| fixture.is_ok()) && status.success(),
        "Python tests: {status}; fixture: {fixture:?}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn explicit_pure_collection_is_independent_of_publication_setup() {
        for args in [
            vec!["-m", "unit"],
            vec!["--markexpr=unit"],
            vec!["--collect-only"],
            vec!["--co", "-m", "component"],
        ] {
            assert!(!needs_publication(
                &args.into_iter().map(String::from).collect::<Vec<_>>()
            ));
        }
        for args in [
            vec![],
            vec!["-m", "unit or component"],
            vec!["-m", "component"],
            vec!["-m", "unit", "-m", "component"],
        ] {
            assert!(needs_publication(
                &args.into_iter().map(String::from).collect::<Vec<_>>()
            ));
        }
    }

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
