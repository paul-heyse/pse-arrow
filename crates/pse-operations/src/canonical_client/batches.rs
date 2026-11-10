// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Borrowed batch delivery retains the complete request and physical driver owner.
use super::{EngineResult, Query, StatementProjection, connection, errors, proto, results, rpc};
use std::{future::Future, pin::Pin};
use surrealdb_types::{Error, Value};

/// A batch cannot outlive its consume call; it is not a committed query result.
#[derive(Debug)]
pub struct TentativeBatch<'a> {
    /// Original statement position, before an operation projection.
    pub statement: usize,
    /// Values admitted and decoded for only this batch.
    pub values: &'a [Value],
    /// This statement ended; later statements, COMMIT or trailers can still fail.
    pub final_batch: bool,
}
/// A consumer's asynchronous work borrows only its current batch.
pub type BatchFuture<'a> = Pin<Box<dyn Future<Output = EngineResult<()>> + Send + 'a>>;
/// Consume each borrowed batch without accumulating its backing values.
pub trait BatchConsumer: Send + 'static {
    /// Process one tentative batch. Publication must wait for CompletedBatches.
    fn consume<'a>(&'a mut self, batch: TentativeBatch<'a>) -> BatchFuture<'a>;
}
pub(super) struct Discard;
impl BatchConsumer for Discard {
    fn consume<'a>(&'a mut self, _batch: TentativeBatch<'a>) -> BatchFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}
/// Terminal success, constructed only after QueryEnd, EOF, trailers and all errors.
#[derive(Debug)]
pub struct CompletedBatches<C> {
    consumer: C,
    rows: u64,
    statements: usize,
}
impl<C> CompletedBatches<C> {
    /// Number of selected values delivered, including zero for an empty selection.
    pub fn rows(&self) -> u64 {
        self.rows
    }
    /// Number of checked semantic statement slots.
    pub fn statements(&self) -> usize {
        self.statements
    }
    /// Recover staged consumer output after complete success.
    pub fn into_consumer(self) -> C {
        self.consumer
    }
}
pub(super) fn selected(
    projection: Option<&StatementProjection>,
    index: usize,
    count: usize,
) -> EngineResult<bool> {
    let (start, end) = match projection {
        None => (0, count),
        Some(StatementProjection::OperationReturn) => (
            count
                .checked_sub(2)
                .ok_or_else(|| connection("Incomplete operation framing"))?,
            count.saturating_sub(1),
        ),
        Some(StatementProjection::Window { prefix, suffix }) => (
            *prefix,
            count
                .checked_sub(*suffix)
                .ok_or_else(|| connection("Incomplete operation framing"))?,
        ),
    };
    if start >= end {
        return Err(connection("Incomplete operation framing"));
    }
    Ok(index >= start && index < end)
}
pub(super) async fn consume<C: BatchConsumer>(
    mut builder: Query<'static>,
    mut consumer: C,
) -> EngineResult<CompletedBatches<C>> {
    let _cleanup = std::mem::replace(
        &mut builder.cleanup,
        super::request::UndispatchedPayloadDrain::caller(None),
    );
    let Query {
        client,
        query,
        variables,
        context,
        observer,
        statement_window,
        #[cfg(test)]
        transaction,
        guarded_bindings: _,
        cleanup: _,
    } = builder;
    let context = context.ok_or_else(|| {
        Error::configuration("Query requires an original request context".into(), None)
    })?;
    let query = query
        .trim_end_matches(|c: char| c == ';' || c.is_whitespace())
        .to_owned();
    let limit = context.response_limits();
    let statement_limit = query
        .len()
        .min(1 << 20)
        .min(client.state.message_bytes / results::statement_slot_bytes())
        .min(limit.map_or(usize::MAX, |limit| {
            limit
                .nodes
                .min(limit.container_items)
                .min(limit.wire_bytes / results::statement_slot_bytes())
        }));
    client
        .execute(context, move |generation| async move {
            let variables: proto::Variables = variables?
                .into_iter()
                .map(|(key, value)| {
                    proto::Value::try_from(value)
                        .map(|value| (key, value))
                        .map_err(connection)
                })
                .collect::<EngineResult<Vec<_>>>()?
                .into_iter()
                .collect();
            let native_context = generation.context()?;
            #[cfg(test)]
            let native_context = match transaction {
                Some((transaction, session)) => {
                    if session != generation.session {
                        return Err(connection(
                            "Held test transaction lost its original connection",
                        ));
                    }
                    rpc::RequestContext {
                        transaction: Some(proto::Uuid::from_uuid(transaction)),
                        ..native_context
                    }
                }
                None => native_context,
            };
            let request = generation.checked(rpc::QueryRequest {
                context: Some(native_context),
                query,
                variables: Some(variables),
                accepted_encodings: Vec::new(),
                max_batch_records: 64,
            })?;
            let stream = generation
                .client
                .clone()
                .query(request)
                .await
                .map_err(errors::status_to_error)?
                .into_inner();
            let collected = results::collect_batches(
                stream,
                statement_limit,
                &mut consumer,
                statement_window.as_ref(),
            )
            .await;
            let (response, rows) = match collected {
                Ok(result) => result,
                Err(error) => {
                    generation.driver.shutdown().await;
                    return Err(error);
                }
            };
            if let Some(observer) = observer {
                response.observe(observer.as_ref());
            }
            let checked = match statement_window {
                Some(StatementProjection::Window { prefix, suffix }) => {
                    response.statement_window(prefix, suffix)
                }
                Some(StatementProjection::OperationReturn) => response.operation_return(),
                None => response.check(),
            }?;
            Ok(CompletedBatches {
                consumer,
                rows,
                statements: checked.num_statements(),
            })
        })
        .await
}
