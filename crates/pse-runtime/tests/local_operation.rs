// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Downstream-style local science through public production constructors only.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixture and scientific assertions"
)]
use futures_util::FutureExt;
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_backend_native::{
    presolve::PolicyKind,
    solve::{Backend, SolveIntent},
};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{solve_runs, solve_variables},
};
use pse_runtime::{
    CancelSource, SharedRuntime,
    math::settings::SolveSettings,
    workflow::{
        ModelingPackage, OperationSource, PreparationSettings, PreparedOperation, Runtime,
        RuntimeOperationRequest,
    },
};
use std::panic::AssertUnwindSafe;
use std::{collections::BTreeMap, num::NonZeroUsize, path::Path, sync::Arc};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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
        self.package_sources(false).await
    }
    async fn package_sources(&self, invalid_physical: bool) -> Result<ModelingPackage> {
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
        let mut physical = files(&root.join("physical-primitives"))?;
        if invalid_physical {
            let document = physical.get_mut("materials/physical.yaml").unwrap();
            *document = String::from_utf8(document.clone())?
                .replace("\"scale_to_canonical\": 1.0", "\"scale_to_canonical\": 0.0")
                .into_bytes();
        }
        let manifest = std::fs::read_to_string(root.join("minimal_explicit/package.toml"))?.replace("id_policy = \"explicit\"", "id_policy = \"named\"").replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
        let modeling = BTreeMap::from([("package.toml".into(), manifest.into_bytes()), ("models/root.pse".into(), b"package local { test square fixture { dof 0; } { param p:Scalar=4; var x:Scalar; eq root:x*x==p; annotation start x(2); } }".to_vec())]);
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
        backend: Backend,
    ) -> Result<PreparedOperation> {
        let preparation = self.runtime.preparation_services();
        let case = package
            .declarations(&preparation)
            .await?
            .iter()
            .find(|row| row.name == "square")
            .unwrap()
            .declaration_id;
        let settings = SolveSettings {
            intent: SolveIntent::Root,
            backend: Some(backend),
            presolve: PolicyKind::Off,
            ..Default::default()
        };
        Ok(package
            .prepare_operation(
                &preparation,
                &OperationSource::of(package),
                &PreparationSettings::default(),
                RuntimeOperationRequest::DeclaredCase {
                    case,
                    settings: Box::new(settings),
                },
                None,
                &CancelSource::new(),
            )
            .await?)
    }
    async fn close(&self) {
        self.runtime.close();
        self.runtime.clear_program_cache();
        self.shared.math().shutdown_native_lanes().await.unwrap();
    }
}

#[tokio::test]
async fn public_local_operation_retains_science_after_producer_close() -> Result<()> {
    let local = Local::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = local.package().await?;
        let prepared = local.prepare(&package, Backend::Kinsol).await?;
        let handle = prepared
            .start(&local.runtime.execution_services(), &CancelSource::new())
            .await?;
        let observer = handle.observations();
        drop(observer);
        let (first, second) = tokio::join!(handle.wait(), handle.wait());
        let first = first?;
        let second = second?;
        assert!(Arc::ptr_eq(&first, &second));
        assert!(first.usable(), "{:?}", first.report());
        assert_eq!(first.run_id, handle.run_id());
        assert!(first.canonical_run_key().is_none());
        let rows = solve_variables::Row::rows(&first.table("runtime.solve_variables")?)?;
        assert_eq!(
            rows.len(),
            2,
            "the original output includes both parameter p and variable x"
        );
        let variables = rows.iter().filter(|row| !row.parameter).collect::<Vec<_>>();
        let parameters = rows.iter().filter(|row| row.parameter).collect::<Vec<_>>();
        assert_eq!(variables.len(), 1);
        assert_eq!(parameters.len(), 1);
        let x = variables[0].value.unwrap();
        let p = parameters[0].value.unwrap();
        assert!(x.is_finite());
        assert_eq!(p, 4.);
        let tolerance = variables[0]
            .tolerance
            .expect("original physical variable tolerance");
        assert!(
            (x * x - p).abs() <= tolerance * (2. * x.abs() + tolerance),
            "original square equation under its propagated physical tolerance"
        );
        let runs = solve_runs::Row::rows(&first.table("runtime.solve_runs")?)?;
        assert_eq!(runs.len(), 1);
        assert_eq!(
            runs[0].backend,
            Some(pse_model::generated::enums::NativeBackend::Kinsol)
        );
        assert!(!first.assessments().is_empty());
        drop(handle);
        drop(prepared);
        drop(package);
        local.close().await;
        let retained = solve_variables::Row::rows(&first.table("runtime.solve_variables")?)?;
        assert_eq!(retained, rows);
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
async fn public_local_operation_refuses_mismatched_bindings_and_prestart_stop() -> Result<()> {
    let local = Local::new()?;
    let other = Local::new()?;
    let outcome = AssertUnwindSafe(async {
        assert!(
            local.package_sources(true).await.is_err(),
            "zero physical unit scale must refuse admission"
        );
        let package = local.package().await?;
        let mismatch = other.prepare(&package, Backend::Kinsol).await.unwrap_err();
        assert!(
            mismatch
                .to_string()
                .contains("original registry and native validation bindings"),
            "{mismatch}"
        );
        assert!(local.prepare(&package, Backend::Highs).await.is_err());
        let prepared = local.prepare(&package, Backend::Kinsol).await?;
        let stop = CancelSource::new();
        stop.cancel();
        assert!(
            prepared
                .start(&local.runtime.execution_services(), &stop)
                .await
                .is_err()
        );
        local.runtime.close();
        assert!(
            prepared
                .start(&local.runtime.execution_services(), &CancelSource::new())
                .await
                .is_err()
        );
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .catch_unwind()
    .await;
    local.close().await;
    other.close().await;
    match outcome {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
