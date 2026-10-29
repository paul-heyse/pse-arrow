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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_team_failure_joins_every_started_worker_including_failed_sibling() {
        let (exiting, exited) = std::sync::mpsc::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let (finished, outcome) = std::sync::mpsc::channel();
        // Always release the owned destructor before joining on assertion failure.
        struct Release(Option<std::sync::mpsc::Sender<()>>);
        impl Drop for Release {
            fn drop(&mut self) {
                if let Some(release) = self.0.take() { let _ = release.send(()); }
            }
        }
        let mut release = Release(Some(release));
        let construction = std::thread::spawn(move || {
            let mut index = 0;
            let mut gate = Some(gate);
            let result = JoinedPool::new(3, 2 << 20, |thread| {
                index += 1;
                match index {
                    1 => Ok(std::thread::spawn(move || {
                        thread.run();
                        panic!("first native team worker failed during exit");
                    })),
                    2 => {
                        let gate = gate.take().unwrap();
                        let exiting = exiting.clone();
                        Ok(std::thread::spawn(move || {
                            thread.run();
                            exiting.send(()).unwrap();
                            gate.recv().unwrap();
                        }))
                    }
                    _ => Err(std::io::Error::other("partial team construction refused")),
                }
            });
            finished.send(result.map(|_| ())).unwrap();
        });
        exited.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(matches!(outcome.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)),
            "failed sibling cannot bypass the remaining native worker join");
        release.0.take().unwrap().send(()).unwrap();
        let result = outcome.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(matches!(result, Err(MathError::Library(message)) if message.contains("partial team construction refused")));
        construction.join().unwrap();
    }
}
