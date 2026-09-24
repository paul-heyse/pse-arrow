// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One-consumption Arrow reader with separately retained native failure ownership.
use datafusion::arrow::{
    array::{RecordBatch, RecordBatchReader},
    datatypes::SchemaRef,
    error::ArrowError,
};
use pse_columnar::CancellationToken;
use pse_engine::EngineError;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

type NextBatch = Box<dyn FnMut() -> Result<Option<RecordBatch>, EngineError> + Send>;

/// Native transport state shared by the handle and its exported Arrow reader.
pub struct BatchStream {
    state: Arc<Mutex<State>>,
    schema: SchemaRef,
    consumed: AtomicBool,
    cancel: CancellationToken,
}
impl std::fmt::Debug for BatchStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchStream")
            .field("schema", &self.schema)
            .finish_non_exhaustive()
    }
}
impl BatchStream {
    /// The producer owns all execution resources; batches retain native buffer owners.
    pub fn new(schema: SchemaRef, cancel: CancellationToken, next: NextBatch) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                next: Some(next),
                failure: None,
            })),
            schema,
            consumed: AtomicBool::new(false),
            cancel,
        }
    }
    /// Exact source schema including nested metadata.
    pub fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
    /// Export once. Requested-schema conversion belongs before this boundary.
    /// # Errors
    /// Closed, cancelled, already exported or poisoned state.
    pub fn reader(&self) -> Result<Box<dyn RecordBatchReader + Send>, EngineError> {
        let state = self
            .state
            .lock()
            .map_err(|_| invalid("stream lock poisoned"))?;
        if let Some(error) = &state.failure {
            return Err(EngineError::Semantic(error.clone()));
        }
        if state.next.is_none() || self.consumed.swap(true, Ordering::AcqRel) {
            return Err(invalid("stream is closed or already consumed"));
        }
        Ok(Box::new(Reader {
            state: self.state.clone(),
            schema: self.schema(),
            terminal: false,
        }))
    }
    /// Stop unread work. Already yielded arrays retain their independent ownership.
    /// # Errors
    /// Poisoned stream state.
    pub fn close(&self) -> Result<(), EngineError> {
        self.cancel.cancel();
        self.state
            .lock()
            .map_err(|_| invalid("stream lock poisoned"))?
            .next = None;
        Ok(())
    }
    /// Stop work with a terminal structured cancellation error.
    /// # Errors
    /// Poisoned stream state.
    pub fn cancel(&self) -> Result<(), EngineError> {
        self.cancel.cancel();
        let mut state = self
            .state
            .lock()
            .map_err(|_| invalid("stream lock poisoned"))?;
        state
            .failure
            .get_or_insert_with(|| Arc::new(EngineError::Cancelled));
        state.next = None;
        Ok(())
    }
    /// Original terminal error, independent of the Arrow ABI's textual last-error slot.
    /// # Errors
    /// Poisoned stream state.
    pub fn failure(&self) -> Result<Option<Arc<EngineError>>, EngineError> {
        Ok(self
            .state
            .lock()
            .map_err(|_| invalid("stream lock poisoned"))?
            .failure
            .clone())
    }
}
struct State {
    next: Option<NextBatch>,
    failure: Option<Arc<EngineError>>,
}
impl State {
    fn next_batch(&mut self, schema: &SchemaRef) -> Result<Option<RecordBatch>, Arc<EngineError>> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let result = self
            .next
            .as_mut()
            .map_or(Ok(None), |next| next())
            .and_then(|batch| {
                if batch
                    .as_ref()
                    .is_some_and(|batch| batch.schema() != *schema)
                {
                    return Err(invalid("batch schema differs from declared stream schema"));
                }
                Ok(batch)
            });
        match result {
            Ok(Some(batch)) => Ok(Some(batch)),
            Ok(None) => {
                self.next = None;
                Ok(None)
            }
            Err(error) => {
                self.next = None;
                let error = Arc::new(error);
                self.failure = Some(error.clone());
                Err(error)
            }
        }
    }
}
#[derive(Debug)]
struct StreamFailure(Arc<EngineError>);
impl std::fmt::Display for StreamFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use miette::Diagnostic;
        if let Some(code) = self.0.code() {
            write!(f, "[{code}] ")?;
        }
        self.0.fmt(f)
    }
}
impl std::error::Error for StreamFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}
struct Reader {
    state: Arc<Mutex<State>>,
    schema: SchemaRef,
    terminal: bool,
}
impl Iterator for Reader {
    type Item = Result<RecordBatch, ArrowError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.terminal {
            return None;
        }
        let result = self
            .state
            .lock()
            .map_err(|_| Arc::new(invalid("stream lock poisoned")))
            .and_then(|mut state| state.next_batch(&self.schema));
        match result {
            Ok(Some(batch)) => Some(Ok(batch)),
            Ok(None) => {
                self.terminal = true;
                None
            }
            Err(error) => {
                self.terminal = true;
                Some(Err(ArrowError::ExternalError(Box::new(StreamFailure(
                    error,
                )))))
            }
        }
    }
}
impl RecordBatchReader for Reader {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
}
impl Drop for Reader {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.next = None;
        }
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "Arrow stream".into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod delta_boundary_unit {
    use super::*;
    use datafusion::{
        arrow::{
            array::{Array, Int64Array},
            datatypes::{DataType, Field, Schema},
            ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream},
        },
        execution::memory_pool::{GreedyMemoryPool, MemoryPool},
    };

    #[test]
    fn ffi_consumer_keeps_original_allocations_after_close() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));
        let batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![3, 5]))])
                .unwrap();
        let pointer = batch.column(0).to_data().buffers()[0].as_ptr();
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1 << 20));
        let cancel = CancellationToken::new();
        let mut batch = Some(
            pse_columnar::owned_buffer::AllocationScope::default()
                .retain_native(batch, &pool)
                .unwrap()
                .into_batch(),
        );
        let handle = BatchStream::new(schema.clone(), cancel, Box::new(move || Ok(batch.take())));
        let mut reader =
            ArrowArrayStreamReader::try_new(FFI_ArrowArrayStream::new(handle.reader().unwrap()))
                .unwrap();
        assert!(handle.reader().is_err());
        let exported = reader.next().unwrap().unwrap();
        assert_eq!(exported.schema(), schema);
        assert_eq!(exported.column(0).to_data().buffers()[0].as_ptr(), pointer);
        handle.close().unwrap();
        assert!(reader.next().is_none());
        drop(reader);
        drop(handle);
        assert!(pool.reserved() > 0);
        assert_eq!(
            exported
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .value(1),
            5
        );
        drop(exported);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn native_terminal_errors_survive_text_only_ffi_and_cancel_differs_from_close() {
        let schema = Arc::new(Schema::empty());
        let handle = BatchStream::new(
            schema.clone(),
            CancellationToken::new(),
            Box::new(|| {
                Err(EngineError::UserModel {
                    message: "native refusal".into(),
                })
            }),
        );
        let mut reader =
            ArrowArrayStreamReader::try_new(FFI_ArrowArrayStream::new(handle.reader().unwrap()))
                .unwrap();
        assert!(reader.next().unwrap().is_err());
        assert!(reader.next().is_none());
        assert!(matches!(
            handle.failure().unwrap().unwrap().as_ref(),
            EngineError::UserModel { .. }
        ));
        let cancelled = BatchStream::new(schema, CancellationToken::new(), Box::new(|| Ok(None)));
        let mut reader = cancelled.reader().unwrap();
        cancelled.cancel().unwrap();
        assert!(reader.next().unwrap().is_err());
        assert!(matches!(
            cancelled.failure().unwrap().unwrap().as_ref(),
            EngineError::Cancelled
        ));
    }
}
