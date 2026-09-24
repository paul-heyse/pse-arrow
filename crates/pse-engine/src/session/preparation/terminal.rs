// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Terminal answers do not imply materialization of their underlying relation.
use super::{EngineError, PreparedComputation};

use pse_columnar::{CancellationToken, owned_buffer::OwnedRecordBatch};

/// Consumption selected before query preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalDemand {
    /// Incremental rows.
    Stream,
    /// Fully retained owned rows.
    Retain,
    /// Whether any row exists.
    Exists,
    /// Exact bag cardinality.
    Count,
    /// Arrival-order rows plus an observed truncation witness.
    Sample(usize),
}
/// Explicit retained-sample ownership policy; no automatic size heuristic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleRetention {
    /// Keep views and their actual parent reservations.
    Share,
    /// Attempt a reserved Arrow copy, retaining the view on quota refusal.
    Compact,
}
impl SampleRetention {
    fn retain(
        self,
        view: OwnedRecordBatch,
        pool: &std::sync::Arc<dyn pse_columnar::MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<OwnedRecordBatch, EngineError> {
        if self == Self::Share {
            return Ok(view);
        }
        match view.compact(pool, cancel) {
            Ok(compacted) => Ok(compacted),
            Err(
                pse_columnar::CanonError::Reservation(_)
                | pse_columnar::CanonError::NativeResource(
                    datafusion::common::DataFusionError::ResourcesExhausted(_),
                ),
            ) => {
                cancel.checkpoint()?;
                Ok(view)
            }
            Err(error) => Err(error.into()),
        }
    }
}
pub(super) fn specialize(
    plan: datafusion::logical_expr::LogicalPlan,
    demand: TerminalDemand,
    pure: bool,
) -> datafusion::common::Result<datafusion::logical_expr::LogicalPlan> {
    use datafusion::{
        functions_aggregate::expr_fn::count,
        logical_expr::{LogicalPlanBuilder, lit},
    };
    let builder = LogicalPlanBuilder::from(plan);
    match demand {
        TerminalDemand::Count => builder
            .aggregate(
                Vec::<datafusion::logical_expr::Expr>::new(),
                [count(lit(1_i64)).alias("__pse_terminal_count")],
            )?
            .build(),
        TerminalDemand::Exists if pure => builder.limit(0, Some(1))?.build(),
        TerminalDemand::Sample(limit) if pure => builder.limit(0, limit.checked_add(1))?.build(),
        _ => builder.build(),
    }
}

/// A bounded arrival-order sample, with truthful completeness and no estimated count.
#[derive(Debug)]
pub struct DiagnosticSample {
    /// Checked ownership of retained result slices.
    pub batches: Vec<OwnedRecordBatch>,
    /// At least one additional row was observed.
    pub truncated: bool,
}

impl PreparedComputation {
    fn requires_terminal_drain(&self) -> bool {
        self.0.effects.iter().any(|effect| {
            use pse_schema::model::provider::OperationEffect as E;
            matches!(effect, E::Write | E::Namespace | E::Publish)
        })
    }

    /// Determine non-emptiness without retaining the result. A negative answer
    /// requires exhaustion; required effectful work is always drained.
    /// # Errors
    /// Native admission, execution, ownership, cancellation or settlement failure.
    pub async fn exists(self, cancel: &CancellationToken) -> Result<bool, EngineError> {
        if matches!(self.0.terminal, TerminalDemand::Count) {
            return Err(EngineError::Admission {
                path: "operation.exists".into(),
                reason: "count output cannot establish source existence".into(),
            });
        }
        let drain = self.requires_terminal_drain();
        let mut stream = self.execute_stream(cancel).await?;
        let mut found = false;
        while let Some(batch) = stream.next_batch(cancel).await? {
            found |= batch.num_rows() != 0;
            if found && !drain {
                return Ok(true);
            }
        }
        Ok(found)
    }

    /// Count every row exactly without retaining result batches or sorting them.
    /// Native aggregate terminals can use the same execution boundary when the
    /// consumer requires distinct rather than bag cardinality.
    /// # Errors
    /// Execution errors or cardinality overflow.
    pub async fn count_rows(self, cancel: &CancellationToken) -> Result<u64, EngineError> {
        if self.0.terminal != TerminalDemand::Count {
            return Err(EngineError::Admission {
                path: "operation.count".into(),
                reason: "select TerminalDemand::Count before preparation".into(),
            });
        }
        let completed = self.execute(cancel).await?;
        let mut total = 0u64;
        for batch in completed.batches() {
            let values = batch
                .column(0)
                .as_any()
                .downcast_ref::<datafusion::arrow::array::Int64Array>()
                .ok_or_else(|| EngineError::Admission {
                    path: "operation.count".into(),
                    reason: "native count result is not Int64".into(),
                })?;
            for value in values {
                let value = value
                    .and_then(|value| u64::try_from(value).ok())
                    .ok_or_else(|| EngineError::Admission {
                        path: "operation.count".into(),
                        reason: "native count is null or negative".into(),
                    })?;
                total = total
                    .checked_add(value)
                    .ok_or_else(|| EngineError::Admission {
                        path: "operation.count".into(),
                        reason: "count overflow".into(),
                    })?;
            }
        }
        Ok(total)
    }

    /// Retain at most `limit` rows in native arrival order. Observe one more row
    /// before reporting truncation; required effects still settle completely.
    /// # Errors
    /// Native execution, cancellation or ownership failure.
    pub async fn sample(
        self,
        limit: usize,
        cancel: &CancellationToken,
    ) -> Result<DiagnosticSample, EngineError> {
        self.sample_with_retention(limit, SampleRetention::Share, cancel)
            .await
    }

    /// Retain a bounded sample with explicit optional compaction.
    /// # Errors
    /// Native execution, incompatible terminal demand, cancellation or Arrow failure.
    pub async fn sample_with_retention(
        self,
        limit: usize,
        retention: SampleRetention,
        cancel: &CancellationToken,
    ) -> Result<DiagnosticSample, EngineError> {
        if !matches!(
            self.0.terminal,
            TerminalDemand::Stream | TerminalDemand::Retain
        ) && self.0.terminal != TerminalDemand::Sample(limit)
        {
            return Err(EngineError::Admission {
                path: "operation.sample".into(),
                reason: "sample request differs from prepared demand".into(),
            });
        }
        let pool = self.0.session.pool.clone();
        let drain = self.requires_terminal_drain();
        let mut stream = self.execute_stream(cancel).await?;
        let mut remaining = limit;
        let mut result = DiagnosticSample {
            batches: Vec::new(),
            truncated: false,
        };
        while let Some(batch) = stream.next_batch(cancel).await? {
            let keep = remaining.min(batch.num_rows());
            result.truncated |= batch.num_rows() > keep;
            if keep != 0 {
                let view = batch.slice(0, keep)?;
                let retained = retention.retain(view, &pool, cancel)?;
                result.batches.push(retained);
                remaining -= keep;
            }
            if result.truncated && !drain {
                break;
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod improvement_unit {
    use super::*;
    use datafusion::arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    };
    use std::sync::Arc;

    #[test]
    fn compact_refusal_keeps_parent_charge_and_cancellation_is_not_a_refusal() {
        let cancel = CancellationToken::new();
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let refusing: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        let raw = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::Int64,
                false,
            )])),
            vec![Arc::new(Int64Array::from_iter_values(0..1024))],
        )
        .unwrap();
        let owner = OwnedRecordBatch::copy(&raw, &pool, &cancel).unwrap();
        let bytes = pool.reserved();
        let view = owner.slice(0, 1).unwrap();
        let retained = SampleRetention::Compact
            .retain(view, &refusing, &cancel)
            .unwrap();
        drop(owner);
        assert_eq!(pool.reserved(), bytes);
        assert_eq!(retained.num_rows(), 1);
        cancel.cancel();
        assert!(
            SampleRetention::Compact
                .retain(retained.clone(), &refusing, &cancel)
                .is_err()
        );
        drop(retained);
        assert_eq!(pool.reserved(), 0);
    }
}
