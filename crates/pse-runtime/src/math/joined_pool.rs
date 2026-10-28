// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual OS-thread custody shared by bounded native library teams.
use pse_math::MathError;

/// Own both the Rayon termination request and the actual OS worker lifetimes.
/// This guard also drains partially constructed pools before constructor errors escape.
pub(crate) struct JoinedPool {
    pool: Option<rayon::ThreadPool>,
    workers: Vec<std::thread::JoinHandle<()>>,
}
impl JoinedPool {
    pub(crate) fn new(
        threads: usize,
        stack_bytes: usize,
        mut spawn: impl FnMut(rayon::ThreadBuilder) -> std::io::Result<std::thread::JoinHandle<()>>,
    ) -> Result<Self, MathError> {
        let mut owner = Self {
            pool: None,
            workers: Vec::with_capacity(threads),
        };
        owner.pool = Some(
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .stack_size(stack_bytes)
                .spawn_handler(|thread| {
                    owner.workers.push(spawn(thread)?);
                    Ok(())
                })
                .build()
                .map_err(|error| MathError::Library(error.to_string()))?,
        );
        Ok(owner)
    }
    pub(crate) fn spawn(&self, operation: impl FnOnce() + Send + 'static) {
        if let Some(pool) = &self.pool {
            pool.spawn(operation);
        }
    }
}
impl JoinedPool {
    /// Execute nested library Rayon work only on this admitted, joined team.
    pub(crate) fn install<T: Send>(&self, work: impl FnOnce() -> T + Send) -> Result<T, MathError> {
        self.pool
            .as_ref()
            .map(|pool| pool.install(work))
            .ok_or_else(|| MathError::Contract("joined pool already retired".into()))
    }
}
impl Drop for JoinedPool {
    fn drop(&mut self) {
        // Rayon drop requests termination; only OS joins establish actual exit.
        drop(self.pool.take());
        for worker in self.workers.drain(..) {
            // A failed worker must not prevent joining the remaining workers.
            let _ = worker.join();
        }
    }
}
pub(crate) fn spawn_pool_worker(
    thread: rayon::ThreadBuilder,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    let mut builder = std::thread::Builder::new();
    if let Some(name) = thread.name() {
        builder = builder.name(name.to_owned());
    }
    if let Some(bytes) = thread.stack_size() {
        builder = builder.stack_size(bytes);
    }
    builder.spawn(|| thread.run())
}
