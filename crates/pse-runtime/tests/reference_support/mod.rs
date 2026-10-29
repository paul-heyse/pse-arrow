// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Local public scientific construction shared by independent reference journeys.
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_runtime::{CancelSource, SharedRuntime, workflow::{ModelingPackage, Runtime}};
use futures_util::{FutureExt, future::BoxFuture};
use std::{collections::BTreeMap, num::NonZeroUsize, path::{Path, PathBuf}, sync::Arc};

pub(super) type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub(super) fn root() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../..") }
pub(super) struct NativeContext {
    pub(super) shared: Arc<SharedRuntime>,
    pub(super) runtime: Runtime,
    _spill: tempfile::TempDir,
}

impl NativeContext {
    pub(super) fn new() -> Result<Self> {
        // Existing downstream local-operation construction, without a canonical service.
        let spill = tempfile::tempdir()?;
        let n = |v| NonZeroUsize::new(v).unwrap();
        let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
            memory_limit_bytes: n(4 << 30), spill_dir: spill.path().to_owned(),
            max_temp_dir_bytes: 1 << 30, top_consumers: n(16),
            threads: pse_engine::ThreadBudget { pool_threads: n(16), target_partitions: n(16) },
            execution: Default::default(), cache: pse_runtime::CacheBudget::disabled(1024),
            math: pse_runtime::math::MathPolicy {
                worker_bytes: 1 << 30, workspace_bytes: 1 << 30, foreign_bytes: 16 << 20,
                jobs: std::thread::available_parallelism()?.get(), ..Default::default()
            },
        })?;
        let factory = Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
        let runtime = Runtime::from_shared_local(shared.clone(), factory.bind_model(pse_schema::shared_registry()?)?)?;
        Ok(Self { shared, runtime, _spill: spill })
    }

    pub(super) fn package<'a>(&'a self, source: &'a str) -> BoxFuture<'a,Result<ModelingPackage>> {
        async move {
        let fixtures = root().join("tests/fixtures/packages");
        let mut physical = BTreeMap::new();
        read_files(&fixtures.join("physical-primitives"), &fixtures.join("physical-primitives"), &mut physical)?;
        let manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))?
            .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
            .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
        let modeling = BTreeMap::from([
            ("package.toml".into(), manifest.into_bytes()),
            ("models/model.pse".into(), source.as_bytes().to_vec()),
        ]);
        let pool = self.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let validation = self.runtime.validation_context();
        let load = |texts: &BTreeMap<String, Vec<u8>>| -> Result<OwnedDocumentSet> {
            Ok(OwnedDocumentSet::try_from_bundles(vec![load_package_documents_owned(
                texts, self.runtime.registry(), Default::default(), &pool, &token, &validation,
            )?], &pool, &token)?)
        };
        let physical = Box::pin(self.runtime.physical_from_documents(&load(&physical)?, &token)).await?;
        let cancel = CancelSource::new();
        Box::pin(self.runtime.modeling_from_documents(&load(&modeling)?, physical, &cancel)).await.map_err(Into::into)

        }.boxed()
    }
    pub(super) async fn close(&self) -> Result<()> {
        self.runtime.close(); self.runtime.clear_program_cache();
        Ok(self.shared.math().shutdown_native_lanes().await?)
    }
}

fn read_files(root: &Path, at: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
    for entry in std::fs::read_dir(at)? {
        let path = entry?.path();
        if path.is_dir() { read_files(root, &path, out)?; }
        else { out.insert(path.strip_prefix(root)?.to_str().unwrap().replace('\\', "/"), std::fs::read(path)?); }
    }
    Ok(())
}
