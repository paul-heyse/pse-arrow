// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Store-free edit, prepare and resolve through public production owners.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixture and independent scientific assertions"
)]

use futures_util::FutureExt;
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_authoring::language::Declaration;
use pse_backend_native::{
    presolve::PolicyKind,
    solve::{Backend, SolveIntent},
};
use pse_model::generated::{enums::NativeBackend, identities::DeclarationId};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{solve_runs, solve_variables},
};
use pse_runtime::{
    CancelSource, SharedRuntime,
    math::settings::SolveSettings,
    workflow::{
        ModelingInspectionRequest, ModelingPackage, OperationSource, PreparationSettings,
        PreparedOperation, RunResult, Runtime, RuntimeOperationRequest, SourceDelta, WorkflowError,
    },
};
use std::{
    collections::BTreeMap, num::NonZeroUsize, panic::AssertUnwindSafe, path::Path, sync::Arc,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn solve_settings() -> SolveSettings {
    SolveSettings {
        intent: SolveIntent::Root,
        backend: Some(Backend::Kinsol),
        presolve: PolicyKind::Off,
        ..Default::default()
    }
}

struct Local {
    shared: Arc<SharedRuntime>,
    runtime: Runtime,
    _spill: tempfile::TempDir,
}

impl Local {
    fn new() -> Result<Self> {
        let spill = tempfile::tempdir()?;
        let n = |v| NonZeroUsize::new(v).unwrap();
        let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
            memory_limit_bytes: n(4 << 30),
            spill_dir: spill.path().to_owned(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: n(16),
            threads: pse_engine::ThreadBudget {
                pool_threads: n(16),
                target_partitions: n(16),
            },
            execution: Default::default(),
            cache: pse_runtime::CacheBudget::disabled(1024),
            math: pse_runtime::math::MathPolicy {
                worker_bytes: 1 << 30,
                workspace_bytes: 1 << 30,
                foreign_bytes: 16 << 20,
                ..Default::default()
            },
        })?;
        let factory =
            Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
        let runtime = Runtime::from_shared_local(
            shared.clone(),
            factory.bind_model(pse_schema::shared_registry()?)?,
        )?;
        Ok(Self {
            shared,
            runtime,
            _spill: spill,
        })
    }

    async fn package(&self) -> Result<ModelingPackage> {
        fn files(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
            fn walk(root: &Path, at: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
                for entry in std::fs::read_dir(at)? {
                    let path = entry?.path();
                    if path.is_dir() {
                        walk(root, &path, out)?;
                    } else {
                        out.insert(
                            path.strip_prefix(root)?
                                .to_str()
                                .unwrap()
                                .replace('\\', "/"),
                            std::fs::read(path)?,
                        );
                    }
                }
                Ok(())
            }
            let mut result = BTreeMap::new();
            walk(root, root, &mut result)?;
            Ok(result)
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
        let physical = files(&root.join("physical-primitives"))?;
        let manifest = std::fs::read_to_string(root.join("minimal_explicit/package.toml"))?
            .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
            .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
        let modeling = BTreeMap::from([
            ("package.toml".into(), manifest.into_bytes()),
            ("models/root.pse".into(), b"package local { test square fixture { dof 0; } { param p:Scalar=4; var x:Scalar; eq root:x*x==p; annotation start x(2); } }".to_vec()),
        ]);
        let pool = self.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let validation = self.runtime.validation_context();
        let load = |texts: &BTreeMap<String, Vec<u8>>| -> Result<OwnedDocumentSet> {
            Ok(OwnedDocumentSet::try_from_bundles(
                vec![load_package_documents_owned(
                    texts,
                    self.runtime.registry(),
                    Default::default(),
                    &pool,
                    &token,
                    &validation,
                )?],
                &pool,
                &token,
            )?)
        };
        let physical = self
            .runtime
            .physical_from_documents(&load(&physical)?, &token)
            .await?;
        Ok(self
            .runtime
            .modeling_from_documents(&load(&modeling)?, physical, &CancelSource::new())
            .await?)
    }

    async fn prepare(
        &self,
        package: &ModelingPackage,
        case: DeclarationId,
        source: &OperationSource,
    ) -> std::result::Result<PreparedOperation, WorkflowError> {
        package
            .prepare_operation(
                &self.runtime.preparation_services(),
                source,
                &PreparationSettings::default(),
                RuntimeOperationRequest::DeclaredCase {
                    case,
                    settings: Box::new(solve_settings()),
                },
                None,
                &CancelSource::new(),
            )
            .await
    }

    async fn solve(&self, prepared: &PreparedOperation) -> Result<Arc<RunResult>> {
        let handle = prepared
            .start(&self.runtime.execution_services(), &CancelSource::new())
            .await?;
        let result = handle.wait().await?;
        assert_eq!(result.run_id, handle.run_id());
        assert!(result.canonical_run_key().is_none());
        Ok(result)
    }

    async fn close(&self) {
        self.runtime.close();
        self.runtime.clear_program_cache();
        self.shared.math().shutdown_native_lanes().await.unwrap();
    }
}

fn declaration<'a>(rows: &'a [Declaration], name: &str) -> &'a Declaration {
    rows.iter()
        .find(|row| row.name == name)
        .expect("original authored declaration")
}

fn assert_science(
    result: &RunResult,
    case: DeclarationId,
    source: &OperationSource,
    parameter: f64,
    expected_root: f64,
    equation_rhs: f64,
) -> Result<Vec<solve_variables::Row>> {
    assert!(result.usable(), "{:?}", result.report());
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs")?)?;
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].case_id, Some(case));
    assert_eq!(runs[0].revision, Some(source.revision.as_id()));
    assert_eq!(runs[0].backend, Some(NativeBackend::Kinsol));
    assert!(!result.assessments().is_empty());
    let rows = solve_variables::Row::rows(&result.table("runtime.solve_variables")?)?;
    assert_eq!(
        rows.len(),
        2,
        "retain both the authored parameter and variable"
    );
    let parameters = rows.iter().filter(|row| row.parameter).collect::<Vec<_>>();
    let variables = rows.iter().filter(|row| !row.parameter).collect::<Vec<_>>();
    assert_eq!(parameters.len(), 1);
    assert_eq!(variables.len(), 1);
    assert_eq!(parameters[0].value, Some(parameter));
    let x = variables[0].value.expect("original solved coordinate");
    let tolerance = variables[0]
        .tolerance
        .expect("original physical variable tolerance");
    assert!(x.is_finite());
    assert!(tolerance.is_finite() && tolerance > 0.);
    assert!(
        (x - expected_root).abs() <= tolerance,
        "independent positive root {expected_root}, got {x}, tolerance {tolerance}"
    );
    assert!(
        (x * x - equation_rhs).abs() <= tolerance * (2. * x.abs() + tolerance),
        "original equation under propagated physical tolerance"
    );
    Ok(rows)
}

#[tokio::test]
async fn public_workspace_edits_reprepare_and_retain_original_science() -> Result<()> {
    let local = Local::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = local.package().await?;
        let preparation = local.runtime.preparation_services();
        let original = package.declarations(&preparation).await?;
        let case = declaration(&original, "square").declaration_id;
        let parameter = declaration(&original, "p");
        let equation = declaration(&original, "root");
        assert_ne!(parameter.document_id, pse_ids::SemanticId::NIL);
        let source = OperationSource::of(&package);
        let inspection_request = ModelingInspectionRequest {
            case,
            preparation: PreparationSettings::default(),
            settings: solve_settings(),
            offset: 0,
            limit: 16,
        };
        let inspection = package
            .inspect_case_request(&preparation, &inspection_request, &CancelSource::new())
            .await?;
        assert_eq!(inspection.source, source);
        assert_eq!(inspection.population.members, 2);
        let symbols = inspection
            .members
            .iter()
            .map(|member| {
                assert!(
                    original
                        .iter()
                        .any(|row| row.declaration_id == member.lineage.declaration)
                );
                (member.lineage.declaration, member.id)
            })
            .collect::<BTreeMap<_, _>>();
        let prepared = local.prepare(&package, case, &source).await?;
        let first = local.solve(&prepared).await?;
        let first_rows = assert_science(&first, case, &source, 4., 2., 4.)?;
        assert_eq!(
            first_rows
                .iter()
                .find(|row| row.parameter)
                .unwrap()
                .symbol_id,
            symbols[&parameter.declaration_id]
        );
        assert_eq!(
            first_rows
                .iter()
                .find(|row| !row.parameter)
                .unwrap()
                .symbol_id,
            symbols[&declaration(&original, "x").declaration_id]
        );

        let mut replacement = parameter.clone();
        replacement.value.binding.as_mut().unwrap().expression = Some("9".into());
        let edited = package
            .with_delta(
                &local.runtime.source_services(),
                SourceDelta {
                    parent: package.revision_id().into(),
                    replacements: vec![replacement.clone()],
                    ..Default::default()
                },
                &CancelSource::new(),
            )
            .await?;
        let edited_source = OperationSource::of(&edited);
        assert_ne!(edited.revision_id(), package.revision_id());
        assert_ne!(edited_source.revision, source.revision);
        assert_eq!(edited_source.physical_context, source.physical_context);
        assert_eq!(&*package.declarations(&preparation).await?, &*original);
        let edited_rows = edited.declarations(&preparation).await?;
        assert_eq!(declaration(&edited_rows, "p"), &replacement);
        assert_eq!(declaration(&edited_rows, "root"), equation);
        assert_eq!(declaration(&edited_rows, "square").declaration_id, case);
        let edited_inspection = edited
            .inspect_case_request(&preparation, &inspection_request, &CancelSource::new())
            .await?;
        assert_eq!(edited_inspection.source, edited_source);
        assert_eq!(
            edited_inspection
                .members
                .iter()
                .map(|member| (member.lineage.declaration, member.id))
                .collect::<BTreeMap<_, _>>(),
            symbols
        );

        let stale = local.prepare(&edited, case, &source).await.unwrap_err();
        assert_eq!(
            stale.boundary_diagnostic().rule,
            pse_diagnostics::DiagnosticRule::StudyBindingRevision
        );
        let edited_prepared = local.prepare(&edited, case, &edited_source).await?;
        let second = local.solve(&edited_prepared).await?;
        let second_rows = assert_science(&second, case, &edited_source, 9., 3., 9.)?;
        assert_ne!(first.run_id, second.run_id);
        assert_eq!(
            first_rows
                .iter()
                .map(|row| row.symbol_id)
                .collect::<Vec<_>>(),
            second_rows
                .iter()
                .map(|row| row.symbol_id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            first_rows
                .iter()
                .map(|row| row.tolerance)
                .collect::<Vec<_>>(),
            second_rows
                .iter()
                .map(|row| row.tolerance)
                .collect::<Vec<_>>()
        );

        let mut changed_equation = equation.clone();
        changed_equation.value.equation.as_mut().unwrap().expression = "x*x==p+7".into();
        let changed = edited
            .with_delta(
                &local.runtime.source_services(),
                SourceDelta {
                    parent: edited.revision_id().into(),
                    replacements: vec![changed_equation.clone()],
                    ..Default::default()
                },
                &CancelSource::new(),
            )
            .await?;
        let changed_source = OperationSource::of(&changed);
        assert_ne!(changed_source.revision, edited_source.revision);
        assert_eq!(changed_source.physical_context, source.physical_context);
        assert_eq!(
            declaration(&changed.declarations(&preparation).await?, "root"),
            &changed_equation
        );
        let changed_prepared = local.prepare(&changed, case, &changed_source).await?;
        let third = local.solve(&changed_prepared).await?;
        assert_science(&third, case, &changed_source, 9., 4., 16.)?;

        // Returned preparations keep their own exact inputs after later revisions and eviction.
        drop(changed_prepared);
        drop(changed);
        drop(edited);
        drop(package);
        local.runtime.clear_program_cache();
        let original_again = local.solve(&prepared).await?;
        assert_science(&original_again, case, &source, 4., 2., 4.)?;
        let edited_again = local.solve(&edited_prepared).await?;
        assert_science(&edited_again, case, &edited_source, 9., 3., 9.)?;
        drop(prepared);
        drop(edited_prepared);
        local.close().await;
        assert_eq!(
            solve_variables::Row::rows(&first.table("runtime.solve_variables")?)?,
            first_rows
        );
        assert_eq!(
            solve_variables::Row::rows(&second.table("runtime.solve_variables")?)?,
            second_rows
        );
        assert_science(&third, case, &changed_source, 9., 4., 16.)?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .catch_unwind()
    .await;
    local.close().await;
    match outcome {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

#[tokio::test]
async fn public_workspace_deletion_and_absent_lookup_do_not_poison_prior_preparation() -> Result<()>
{
    let local = Local::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = local.package().await?;
        let preparation = local.runtime.preparation_services();
        let original = package.declarations(&preparation).await?;
        let case = declaration(&original, "square").declaration_id;
        let parameter = declaration(&original, "p").clone();
        let source = OperationSource::of(&package);
        let prepared = local.prepare(&package, case, &source).await?;

        let deleted = package
            .with_delta(
                &local.runtime.source_services(),
                SourceDelta {
                    parent: package.revision_id().into(),
                    removals: vec![parameter.declaration_id],
                    ..Default::default()
                },
                &CancelSource::new(),
            )
            .await?;
        let deleted_source = OperationSource::of(&deleted);
        assert!(
            !deleted
                .declarations(&preparation)
                .await?
                .iter()
                .any(|row| row.declaration_id == parameter.declaration_id)
        );
        // The equation still names p: the previously successful name lookup must be invalidated.
        let missing = local
            .prepare(&deleted, case, &deleted_source)
            .await
            .unwrap_err();
        let diagnostic = missing.boundary_diagnostic();
        assert_eq!(
            diagnostic.rule,
            pse_diagnostics::DiagnosticRule::ModelingContract,
            "{missing:?}"
        );
        assert!(
            diagnostic
                .sources
                .contains(&declaration(&original, "root").declaration_id.as_id()),
            "retain the original equation identity: {diagnostic:?}"
        );
        assert_eq!(&*package.declarations(&preparation).await?, &*original);

        let stale_parent = deleted
            .with_delta(
                &local.runtime.source_services(),
                SourceDelta {
                    parent: package.revision_id().into(),
                    additions: vec![parameter.clone()],
                    ..Default::default()
                },
                &CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(
            stale_parent.to_string().contains("different local parent"),
            "{stale_parent}"
        );

        // The absent-name result must also be invalidated when the same authored p returns.
        let repaired = deleted
            .with_delta(
                &local.runtime.source_services(),
                SourceDelta {
                    parent: deleted.revision_id().into(),
                    additions: vec![parameter.clone()],
                    ..Default::default()
                },
                &CancelSource::new(),
            )
            .await?;
        assert_eq!(
            declaration(&repaired.declarations(&preparation).await?, "p"),
            &parameter
        );
        let repaired_source = OperationSource::of(&repaired);
        let repaired_prepared = local.prepare(&repaired, case, &repaired_source).await?;
        let repaired_result = local.solve(&repaired_prepared).await?;
        assert_science(&repaired_result, case, &repaired_source, 4., 2., 4.)?;

        let removed = package
            .with_delta(
                &local.runtime.source_services(),
                SourceDelta {
                    parent: package.revision_id().into(),
                    removals: original
                        .iter()
                        .filter(|row| row.declaration_id == case || row.parent_id == Some(case))
                        .map(|row| row.declaration_id)
                        .collect(),
                    ..Default::default()
                },
                &CancelSource::new(),
            )
            .await?;
        assert!(
            !removed
                .declarations(&preparation)
                .await?
                .iter()
                .any(|row| row.declaration_id == case)
        );
        assert!(
            local
                .prepare(&removed, case, &OperationSource::of(&removed))
                .await
                .is_err(),
            "the original authored case is now absent"
        );
        let retained = local.solve(&prepared).await?;
        assert_science(&retained, case, &source, 4., 2., 4.)?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .catch_unwind()
    .await;
    local.close().await;
    match outcome {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
