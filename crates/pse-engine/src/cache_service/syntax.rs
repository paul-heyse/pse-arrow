// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parsed native syntax only. Every consumer binds against its current session.
use super::{NativeCacheService, envelope::Envelope};
use datafusion::{
    common::{DataFusionError, Result, TableReference},
    execution::{
        cache::{Cache, CacheKey, CacheValue},
        memory_pool::MemoryConsumer,
        session_state::SessionState,
    },
    sql::parser::Statement,
};
use pse_columnar::CancellationToken;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Key {
    parser: &'static str,
    sql: String,
    options: Vec<(String, Option<String>)>,
}
impl CacheKey for Key {
    fn size(&self) -> usize {
        self.sql
            .capacity()
            .saturating_add(
                self.options
                    .iter()
                    .map(|(k, v)| k.capacity() + v.as_ref().map_or(0, String::capacity) + 128)
                    .sum::<usize>(),
            )
            .saturating_add(256)
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}
#[derive(Clone)]
pub(super) struct Value {
    statement: Arc<Statement>,
    extent: usize,
}
impl CacheValue for Value {
    fn size(&self) -> usize {
        self.extent
    }
}
pub(super) type SyntaxCache = Envelope<Key, Value>;

/// Parse under the actual dialect/options; syntax reuse never establishes binding.
/// # Errors
/// Cancellation, native parser failure, or refusal of the current parse/clone budget.
pub(crate) fn parse(
    state: &SessionState,
    sql: &str,
    cancel: &CancellationToken,
) -> Result<Statement> {
    let checkpoint = || {
        cancel
            .checkpoint()
            .map_err(|error| pse_columnar::external(crate::EngineError::from(error)))
    };
    checkpoint()?;
    // Account syntax/control overhead independently of native allocator/RSS claims.
    let extent = sql
        .len()
        .checked_mul(128)
        .and_then(|n| n.checked_add(4096))
        .ok_or_else(|| DataFusionError::ResourcesExhausted("SQL syntax extent overflow".into()))?;
    let owner =
        MemoryConsumer::new("session:sql-syntax").register(&state.runtime_env().memory_pool);
    owner.try_grow(extent)?;
    let service = state.config().get_extension::<NativeCacheService>();
    let mut key = Key {
        parser: "datafusion-55.1/sqlparser-0.62",
        sql: sql.to_owned(),
        options: state
            .config_options()
            .entries()
            .into_iter()
            .filter(|entry| entry.key.starts_with("datafusion.sql_parser."))
            .map(|entry| (entry.key, entry.value))
            .collect(),
    };
    key.options.sort();
    if let Some(value) = service
        .as_ref()
        .and_then(|service| service.syntax.get(&key))
    {
        checkpoint()?;
        return Ok(value.statement.as_ref().clone());
    }
    super::metrics::record(state, |metrics| &metrics.sql_parses);
    let statement = state.sql_to_statement(sql, &state.config_options().sql_parser.dialect)?;
    checkpoint()?;
    if let Some(service) = service {
        service.syntax.put(
            &key,
            Value {
                statement: Arc::new(statement.clone()),
                extent,
            },
        );
    }
    Ok(statement)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::execution::{
        context::SessionContext, memory_pool::GreedyMemoryPool, session_state::SessionStateBuilder,
    };
    #[test]
    fn scoped_reuse_unit_syntax_is_separate_from_options_and_failed_population() {
        let state = SessionContext::new().state();
        let pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool> =
            Arc::new(GreedyMemoryPool::new(1 << 20));
        let mut budget = super::super::CacheBudget::disabled(1);
        budget.syntax_bytes = 64 << 10;
        let cache = NativeCacheService::new(budget, &pool).unwrap();
        let state = SessionStateBuilder::new_from_existing(state.clone())
            .with_config(state.config().clone().with_extension(cache.clone()))
            .build();
        let cancel = CancellationToken::new();
        assert!(parse(&state, "SELECT 1", &cancel).is_ok());
        assert!(parse(&state, "SELECT 1", &cancel).is_ok());
        assert_eq!(cache.syntax.len(), 1);
        assert_eq!(
            cache
                .metrics
                .sql_parses
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert!(parse(&state, "SELECT (", &cancel).is_err());
        assert_eq!(cache.syntax.len(), 1);
        let mut config = state.config().clone();
        config.options_mut().sql_parser.enable_ident_normalization = false;
        let changed = SessionStateBuilder::new_from_existing(state)
            .with_config(config)
            .build();
        assert!(parse(&changed, "SELECT 1", &cancel).is_ok());
        assert_eq!(cache.syntax.len(), 2);
        let tiny = Arc::new(
            datafusion::execution::runtime_env::RuntimeEnvBuilder::new()
                .with_memory_pool(Arc::new(GreedyMemoryPool::new(1)))
                .build()
                .unwrap(),
        );
        let restricted = SessionStateBuilder::new_from_existing(changed.clone())
            .with_runtime_env(tiny)
            .build();
        assert!(
            parse(&restricted, "SELECT 1", &cancel).is_err(),
            "a retained parse does not bypass the current pool"
        );
        let disabled =
            NativeCacheService::new(super::super::CacheBudget::disabled(1), &pool).unwrap();
        let off = SessionStateBuilder::new_from_existing(changed.clone())
            .with_config(changed.config().clone().with_extension(disabled.clone()))
            .build();
        parse(&off, "SELECT 1", &cancel).unwrap();
        parse(&off, "SELECT 1", &cancel).unwrap();
        assert_eq!(disabled.syntax.len(), 0);
        assert_eq!(
            disabled
                .metrics
                .sql_parses
                .load(std::sync::atomic::Ordering::Relaxed),
            2
        );
        cancel.cancel();
        assert!(parse(&changed, "SELECT 1", &cancel).is_err());
    }
}
