// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native DML owns an isolated mutable target and admits its replacement before exposure.

mod native;
pub(crate) use native::bound_source;
pub(crate) use native::isolate;

use super::{EngineSession, capture::CapturedProvider};
use crate::{BoxFut, EngineError};
use datafusion::{
    catalog::{Session, TableProvider},
    common::{Result as NativeResult, TableReference},
    datasource::MemTable,
};
use pse_columnar::CancellationToken;
use std::sync::Arc;

/// Actual factory for an exclusively owned candidate. It must isolate mutable backing
/// from the source and all other attempts, preserve the exact schema/default semantics,
/// and perform no externally visible writes. Native hooks report their actual support.
/// This is a contract on the bound implementation, not a capability flag or provider roster.
pub trait PrivateTableFactory: std::fmt::Debug + Send + Sync {
    /// Construct the private target during native execution, using the shared runtime.
    /// # Errors
    /// Unsupported isolation, schema/default incompatibility, resource or native failure.
    fn begin<'a>(
        &'a self,
        source: Arc<dyn TableProvider>,
        state: &'a dyn Session,
    ) -> BoxFut<'a, NativeResult<Arc<dyn TableProvider>>>;
}

/// DataFusion's native in-memory DML hooks over fresh partition locks and retained buffers.
/// At the pinned release these support append INSERT, DELETE and UPDATE; other hooks
/// retain the engine's truthful unsupported result.
#[derive(Debug, Default)]
pub struct MemoryTableFactory;
impl PrivateTableFactory for MemoryTableFactory {
    fn begin<'a>(
        &'a self,
        source: Arc<dyn TableProvider>,
        state: &'a dyn Session,
    ) -> BoxFut<'a, NativeResult<Arc<dyn TableProvider>>> {
        Box::pin(async move {
            let defaults = source
                .schema()
                .fields()
                .iter()
                .filter_map(|field| {
                    source
                        .get_column_default(field.name())
                        .map(|default| (field.name().to_owned(), default.clone()))
                })
                .collect();
            let table: Arc<dyn TableProvider> = Arc::new(
                MemTable::load(source, None, state)
                    .await?
                    .with_column_defaults(defaults),
            );
            Ok(table)
        })
    }
}

impl EngineSession {
    /// Enable private DML on an actually captured source generation using a bound factory.
    /// Every successful mutation returns a new immutable generation; existing readers keep
    /// their original source. Failed or dropped attempts expose no replacement.
    /// # Errors
    /// Foreign capture, conflicting name, invalid fields or cancellation.
    pub fn with_mutable_capture(
        &self,
        reference: &TableReference,
        captured: &CapturedProvider,
        factory: Arc<dyn PrivateTableFactory>,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        let mut session = self.with_captured_provider(reference.clone(), captured, cancel)?;
        session
            .bindings
            .mutation_factory(reference, factory)
            .map_err(super::engine)?;
        Ok(session)
    }
}
