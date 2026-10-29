// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete query-frame assembly and the consumed typed extraction contract.
use super::{
    EngineResult,
    errors::{proto_error, status_to_error},
};
use std::{collections::BTreeMap, time::Duration};
use surrealdb_protocol::proto::rpc::v1 as rpc;
use surrealdb_types::{Array, Error, SurrealValue, Value};

/// Actual per-statement server statistics; an absent duration is not zero.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// The original reported statement duration, when supplied.
    pub execution_time: Option<Duration>,
}

/// A bounded caller's observer; it never owns request or connection authority.
pub trait QueryStatisticsObserver: Send + Sync {
    /// One complete received statement before its values are consumed.
    fn observe(&self, statement: usize, stats: Stats, succeeded: bool);
    /// No complete response was delivered.
    fn request_failed(&self) {}
}

/// Ordered statement results, published only after QueryEnd and trailers settle.
#[derive(Debug, Default)]
pub struct IndexedResults {
    values: BTreeMap<usize, (Stats, EngineResult<Value>)>,
}
impl IndexedResults {
    /// Number of remaining statement slots, including failed slots.
    pub fn num_statements(&self) -> usize {
        self.values.len()
    }
    /// Preserve all original errors before removing owner-added transaction framing.
    pub(crate) fn statement_window(self, prefix: usize, suffix: usize) -> EngineResult<Self> {
        let checked = self.check()?;
        if checked.values.keys().copied().ne(0..checked.values.len()) {
            return Err(Error::configuration("Canonical envelope response has missing statement slots".into(), None));
        }
        let end = checked.values.len().checked_sub(suffix).filter(|end| *end > prefix)
            .ok_or_else(|| Error::configuration("Canonical envelope response is incomplete".into(), None))?;
        let values = checked.values.into_iter().filter_map(|(index, value)| {
            (index >= prefix && index < end).then(|| (index - prefix, value))
        }).collect();
        Ok(Self { values })
    }
    /// Remove errors in original compacted statement order.
    pub fn take_errors(&mut self) -> BTreeMap<usize, Error> {
        let keys = self
            .values
            .iter()
            .filter_map(|(index, (_, value))| value.is_err().then_some(*index))
            .collect::<Vec<_>>();
        keys.into_iter()
            .filter_map(|index| match self.values.remove(&index) {
                Some((_, Err(error))) => Some((index, error)),
                _ => None,
            })
            .collect()
    }
    /// Consume one slot using the supported Option/Vec/Value result rules.
    pub fn take<R: SurrealValue>(&mut self, index: impl QueryResult<R>) -> EngineResult<R> {
        index.extract(self)
    }
    /// Refuse a failed statement, preferring substantive failure to cancellation.
    pub fn check(mut self) -> EngineResult<Self> {
        let errors = self.take_errors();
        let substantive = errors.iter().find_map(|(index, error)| {
            (!matches!(
                error.query_details(),
                Some(
                    surrealdb_types::QueryError::NotExecuted
                        | surrealdb_types::QueryError::Cancelled
                )
            ))
            .then_some(*index)
        });
        if let Some((_, error)) = errors
            .into_iter()
            .find(|(index, _)| substantive.is_none_or(|selected| *index == selected))
        {
            return Err(error);
        }
        Ok(self)
    }
    pub(super) fn observe(&self, observer: &dyn QueryStatisticsObserver) {
        for (index, (stats, result)) in &self.values {
            observer.observe(*index, *stats, result.is_ok());
        }
    }
    fn remove(&mut self, index: usize) -> EngineResult<Value> {
        self.values
            .remove(&index)
            .map_or(Ok(Value::None), |(_, result)| result)
    }
}

mod sealed {
    pub trait Index<R> {}
}
/// The consumed query-result index contract. Only supported result families exist.
pub trait QueryResult<R: SurrealValue>: sealed::Index<R> {
    /// Remove the indexed slot and perform its original typed conversion.
    fn extract(self, response: &mut IndexedResults) -> EngineResult<R>;
}
impl sealed::Index<Value> for usize {}
impl QueryResult<Value> for usize {
    fn extract(self, response: &mut IndexedResults) -> EngineResult<Value> {
        response.remove(self)
    }
}
impl<T: SurrealValue> sealed::Index<Option<T>> for usize {}
impl<T: SurrealValue> QueryResult<Option<T>> for usize {
    fn extract(self, response: &mut IndexedResults) -> EngineResult<Option<T>> {
        let value = match response.remove(self)? {
            Value::Array(array) => {
                let mut values = array.into_vec();
                match values.len() {
                    0 => return Ok(None),
                    1 => values.remove(0),
                    _ => return Err(Error::internal(
                        "Tried to take only a single result from a query that contains multiple"
                            .into(),
                    )),
                }
            }
            value => value,
        };
        if value == Value::None {
            return Ok(None);
        }
        T::from_value(value)
            .map(Some)
            .map_err(|error| Error::internal(error.to_string()))
    }
}
impl<T: SurrealValue> sealed::Index<Vec<T>> for usize {}
impl<T: SurrealValue> QueryResult<Vec<T>> for usize {
    fn extract(self, response: &mut IndexedResults) -> EngineResult<Vec<T>> {
        let values = match response.values.remove(&self) {
            None => return Ok(Vec::new()),
            Some((_, Err(error))) => return Err(error),
            Some((_, Ok(Value::Array(array)))) => array.into_vec(),
            Some((_, Ok(value))) => vec![value],
        };
        values
            .into_iter()
            .map(|value| T::from_value(value).map_err(|error| Error::internal(error.to_string())))
            .collect()
    }
}

#[derive(Default)]
struct Statement {
    complete: bool,
    batches: u64,
    values: Vec<Value>,
    single: bool,
    stats: Option<rpc::QueryStats>,
    kind: i32,
    error: Option<Error>,
}
impl Statement {
    fn push(&mut self, batch: rpc::QueryBatchFrame) -> EngineResult<()> {
        if self.complete || batch.batch_index != self.batches {
            return Err(protocol("Query batch sequence is incomplete or repeated"));
        }
        self.batches = self
            .batches
            .checked_add(1)
            .ok_or_else(|| protocol("Query batch count overflow"))?;
        let kind = rpc::QueryResponseKind::try_from(batch.kind)
            .map_err(|_| protocol("Query carried an unknown batch kind"))?;
        if !matches!(
            kind,
            rpc::QueryResponseKind::Single
                | rpc::QueryResponseKind::Batched
                | rpc::QueryResponseKind::BatchedFinal
        ) {
            return Err(protocol("Query carried an unspecified batch kind"));
        }
        if self.batches > 1 && kind == rpc::QueryResponseKind::Single {
            return Err(protocol(
                "Single query response cannot follow an earlier batch",
            ));
        }
        if self.batches > 1 && self.kind != batch.statement_kind {
            return Err(protocol("Query changed its statement kind"));
        }
        self.kind = batch.statement_kind;
        self.complete = batch.error.is_some()
            || matches!(
                kind,
                rpc::QueryResponseKind::Single | rpc::QueryResponseKind::BatchedFinal
            );
        self.single = kind == rpc::QueryResponseKind::Single;
        if batch.stats.is_some() {
            self.stats = batch.stats;
        }
        if let Some(error) = batch.error {
            self.error = Some(proto_error(error));
            return Ok(());
        }
        match batch.payload {
            Some(rpc::query_batch_frame::Payload::Values(values)) => {
                if self.single && values.values.len() > 1 {
                    return Err(protocol("Single query response carried multiple values"));
                }
                self.values.reserve(values.values.len());
                for value in values.values {
                    self.values.push(Value::try_from(value).map_err(|error| {
                        Error::serialization(
                            error.to_string(),
                            surrealdb_types::SerializationError::Deserialization,
                        )
                    })?);
                }
            }
            Some(rpc::query_batch_frame::Payload::Arrow(_)) => {
                return Err(protocol("Server sent unrequested columnar data"));
            }
            None => {}
        }
        Ok(())
    }
    fn finish(self) -> EngineResult<(Stats, EngineResult<Value>)> {
        if self.kind == rpc::QueryStatementKind::Live as i32 {
            return Err(protocol(
                "Unsupported LIVE response requires closing its native generation",
            ));
        }
        let stats = Stats {
            execution_time: self
                .stats
                .and_then(|stats| stats.execution_duration)
                .map(Duration::try_from)
                .transpose()
                .map_err(|error| protocol(&error.to_string()))?,
        };
        let result = match self.error {
            Some(error) => Err(error),
            None if self.single => Ok(self.values.into_iter().next().unwrap_or(Value::None)),
            None => Ok(Value::Array(Array::from(self.values))),
        };
        Ok((stats, result))
    }
}
fn protocol(message: &str) -> Error {
    Error::connection(message.to_owned(), None)
}

pub(super) fn statement_slot_bytes() -> usize {
    size_of::<Option<Statement>>()
}

pub(super) async fn collect(
    mut stream: tonic::Streaming<rpc::QueryResponse>,
    statement_limit: usize,
) -> EngineResult<IndexedResults> {
    let mut statements: Vec<Option<Statement>> = Vec::new();
    let mut began = false;
    let mut end = None;
    while let Some(response) = stream.message().await.map_err(status_to_error)? {
        match response.frame {
            Some(rpc::query_response::Frame::Begin(begin)) => {
                let count = usize::try_from(begin.statement_count)
                    .map_err(|_| protocol("Query statement count overflow"))?;
                if began {
                    return Err(protocol("Query repeated its Begin frame"));
                }
                if count > statement_limit {
                    return Err(protocol(
                        "Query statement count exceeds original statement allocation allowance",
                    ));
                }
                began = true;
                statements.resize_with(count, || None);
            }
            Some(rpc::query_response::Frame::Batch(batch)) => {
                let index = usize::try_from(batch.query_index)
                    .map_err(|_| protocol("Query statement index overflow"))?;
                if !began {
                    return Err(protocol("Query batch preceded Begin"));
                }
                statements
                    .get_mut(index)
                    .ok_or_else(|| protocol("Query statement index exceeds Begin count"))?
                    .get_or_insert_with(Statement::default)
                    .push(batch)?;
            }
            Some(rpc::query_response::Frame::End(value)) => {
                if !began {
                    return Err(protocol("Query End preceded Begin"));
                }
                end = Some(value);
                break;
            }
            Some(rpc::query_response::Frame::Error(error)) => {
                let error = proto_error(error);
                if stream.message().await.map_err(status_to_error)?.is_some() {
                    return Err(protocol("Query carried frames after terminal failure"));
                }
                return Err(error);
            }
            None => return Err(protocol("Query carried an unknown frame")),
        }
    }
    let end = end.ok_or_else(|| protocol("Query ended before completion"))?;
    if statements
        .iter()
        .flatten()
        .any(|statement| !statement.complete)
    {
        return Err(protocol("Query ended before every statement completed"));
    }
    if stream.message().await.map_err(status_to_error)?.is_some() {
        return Err(protocol("Query carried frames after End"));
    }
    let actual = statements.iter().flatten().count();
    if actual
        != usize::try_from(end.result_count).map_err(|_| protocol("Query result count overflow"))?
    {
        return Err(protocol(
            "Query End result count differs from received results",
        ));
    }
    let mut values = BTreeMap::new();
    for (index, statement) in statements.into_iter().flatten().enumerate() {
        if statement.kind == rpc::QueryStatementKind::Kill as i32 {
            continue;
        }
        values.insert(index, statement.finish()?);
    }
    Ok(IndexedResults { values })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn result(value: Value) -> IndexedResults {
        IndexedResults {
            values: BTreeMap::from([(0, (Stats::default(), Ok(value)))]),
        }
    }
    fn envelope(body: EngineResult<Value>, commit: EngineResult<Value>) -> IndexedResults {
        IndexedResults { values: BTreeMap::from([
            (0, (Stats::default(), Ok(Value::None))),
            (1, (Stats::default(), Ok(Value::Bool(true)))),
            (2, (Stats::default(), body)),
            (3, (Stats::default(), commit)),
        ]) }
    }
    #[test]
    fn schema_envelope_keeps_original_slots_only_after_complete_success() {
        let mut result = envelope(Ok(Value::String("original".into())), Ok(Value::None)).statement_window(2,1).unwrap();
        assert_eq!(result.num_statements(),1);
        assert_eq!(result.take::<Value>(0).unwrap(),Value::String("original".into()));
    }
    #[test]
    fn schema_envelope_preserves_guard_and_commit_refusals_before_projection() {
        let error = Error::configuration("original commit failed".into(),None);
        assert!(envelope(Ok(Value::String("provisional".into())),Err(error)).statement_window(2,1).is_err());
        let mut reply = envelope(Ok(Value::None),Ok(Value::None));
        reply.values.insert(1,(Stats::default(),Err(Error::configuration("schema fence refused".into(),None))));
        assert!(reply.statement_window(2,1).is_err());
    }
    #[test]
    fn schema_envelope_refuses_missing_slots_without_publishing_provisional_values() {
        let mut reply = envelope(Ok(Value::String("provisional".into())),Ok(Value::None));
        reply.values.remove(&1);
        assert!(reply.statement_window(2,1).is_err());
    }
    #[test]
    fn typed_cardinality_and_removal_preserve_consumed_contract() {
        for value in [
            Value::Array(Array::from(Vec::<Value>::new())),
            Value::Array(Array::from(vec![Value::None])),
            Value::None,
        ] {
            let mut rows = result(value);
            assert_eq!(rows.take::<Option<String>>(0).unwrap(), None);
            assert_eq!(rows.num_statements(), 0);
        }
        let mut rows = result(Value::Array(Array::from(vec![
            Value::String("one".into()),
            Value::String("two".into()),
        ])));
        assert!(rows.take::<Option<String>>(0).is_err());
        assert_eq!(rows.num_statements(), 0);
        assert_eq!(rows.take::<Value>(9).unwrap(), Value::None);
        assert!(rows.take::<Vec<String>>(9).unwrap().is_empty());
        let mut rows = result(Value::String("one".into()));
        assert_eq!(rows.take::<Vec<String>>(0).unwrap(), ["one"]);
        let mut rows = result(Value::Bool(true));
        assert!(rows.take::<Option<String>>(0).unwrap_err().is_internal());
    }
}
