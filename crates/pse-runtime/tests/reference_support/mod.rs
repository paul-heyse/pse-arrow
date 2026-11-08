// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Local public scientific construction shared by independent reference journeys.
mod context;
mod manifest;
pub(super) use context::{NativeContext, Result, root};

use futures_util::{FutureExt, future::BoxFuture};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_runtime::{CancelSource, workflow::ModelingPackage};
use std::{collections::BTreeMap, path::Path};

impl NativeContext {
    pub(super) fn package<'a>(
        &'a self,
        source: &'a str,
    ) -> BoxFuture<'a, Result<(ModelingPackage, BTreeMap<String, Vec<u8>>)>> {
        async move {
            let fixtures = root().join("tests/fixtures/packages");
            let mut physical_inputs = BTreeMap::new();
            read_files(
                &fixtures.join("physical-primitives"),
                &fixtures.join("physical-primitives"),
                &mut physical_inputs,
            )?;
            let original_manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))?;
            let manifest = manifest::named_model_manifest(
                &original_manifest,
                physical_inputs
                    .get("package.toml")
                    .ok_or("physical package manifest absent")?,
            )?;
            let modeling = BTreeMap::from([
                ("package.toml".into(), manifest),
                ("models/model.pse".into(), source.as_bytes().to_vec()),
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
            let physical = Box::pin(
                self.runtime
                    .physical_from_documents(&load(&physical_inputs)?, &token),
            )
            .await?;
            let cancel = CancelSource::new();
            let package = Box::pin(
                self.runtime
                    .modeling_from_documents(&load(&modeling)?, physical, &cancel),
            )
            .await?;
            let mut originals = BTreeMap::from([(
                "tests/fixtures/packages/minimal_explicit/package.toml".into(),
                original_manifest.into_bytes(),
            )]);
            originals.extend(physical_inputs.into_iter().map(|(relative, content)| {
                (format!("tests/fixtures/packages/physical-primitives/{relative}"), content)
            }));
            Ok((package, originals))
        }
        .boxed()
    }
}

fn read_files(root: &Path, at: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
    for entry in std::fs::read_dir(at)? {
        let path = entry?.path();
        if path.is_dir() {
            read_files(root, &path, out)?;
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
