// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual OS-thread custody shared by bounded native library teams.
use super::MathRuntimeError;
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
    ) -> Result<Self, MathRuntimeError> {
        let mut owner = Self {
            pool: None,
            workers: Vec::with_capacity(threads),
        };
        let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .stack_size(stack_bytes)
                .spawn_handler(|thread| {
                    owner.workers.push(spawn(thread)?);
                    Ok(())
                })
                .build()
        }))
        .map_err(|payload| worker_panic("native pool construction", payload))
        .and_then(|result| {
            result.map_err(|error| MathRuntimeError::Math(MathError::Library(error.to_string())))
        });
        match built {
            Ok(pool) => {
                owner.pool = Some(pool);
                Ok(owner)
            }
            Err(error) => MathRuntimeError::settle(Err(error), owner.finish()),
        }
    }
    pub(crate) fn spawn(&self, operation: impl FnOnce() + Send + 'static) {
        if let Some(pool) = &self.pool {
            pool.spawn(operation);
        }
    }
    /// Request Rayon termination and collect every actual OS-worker join outcome.
    pub(crate) fn finish(mut self) -> Result<(), MathRuntimeError> {
        self.retire()
    }
    fn retire(&mut self) -> Result<(), MathRuntimeError> {
        drop(self.pool.take());
        let mut result = Ok(());
        for worker in self.workers.drain(..) {
            let joined = worker
                .join()
                .map_err(|payload| worker_panic("native pool worker", payload));
            result = MathRuntimeError::settle(result, joined);
        }
        result
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
        // Abandonment still settles all physical workers; explicit finish reports failures.
        let _ = self.retire();
    }
}
pub(super) fn worker_panic(
    operation: &str,
    payload: Box<dyn std::any::Any + Send>,
) -> MathRuntimeError {
    let detail = pse_diagnostics::panic::settle_payload(payload, "native worker non-text panic");
    MathRuntimeError::Panic(format!("{operation}: {detail}"))
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
    fn finish_preserves_every_worker_panic_and_waits_for_sibling_tls_departure() {
        struct Departure {
            entered: std::sync::mpsc::Sender<()>,
            release: std::sync::mpsc::Receiver<()>,
        }
        impl Drop for Departure {
            fn drop(&mut self) {
                self.entered.send(()).unwrap();
                self.release.recv().unwrap();
            }
        }
        thread_local! {
            static DEPARTURE: std::cell::RefCell<Option<Departure>> = const { std::cell::RefCell::new(None) };
        }
        let (entered, departing) = std::sync::mpsc::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let (send, received) = std::sync::mpsc::channel();
        let owner = JoinedPool {
            pool: None,
            workers: vec![
                std::thread::spawn(|| panic!("first joined failure")),
                std::thread::spawn(move || {
                    DEPARTURE.with(|slot| {
                        *slot.borrow_mut() = Some(Departure {
                            entered,
                            release: gate,
                        })
                    });
                    panic!("second joined failure");
                }),
            ],
        };
        let finish = std::thread::spawn(move || send.send(owner.finish()).unwrap());
        departing
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let pending = matches!(
            received.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        );
        release.send(()).unwrap();
        let result = received
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        finish.join().unwrap();
        assert!(pending, "finish must wait for physical TLS destruction");
        assert!(
            matches!(result, Err(MathRuntimeError::Settlement { operation, cleanup })
            if matches!(*operation, MathRuntimeError::Panic(ref message) if message.contains("first joined failure"))
            && matches!(*cleanup, MathRuntimeError::Panic(ref message) if message.contains("second joined failure")))
        );
    }

    #[test]
    fn panic_payload_destructor_failure_cannot_skip_independent_worker_join() {
        struct Payload;
        impl Drop for Payload {
            fn drop(&mut self) {
                panic!("joined payload destructor failed");
            }
        }
        let joined = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let sibling = joined.clone();
        let owner = JoinedPool {
            pool: None,
            workers: vec![
                std::thread::spawn(|| std::panic::panic_any(Payload)),
                std::thread::spawn(move || {
                    sibling.store(true, std::sync::atomic::Ordering::Release)
                }),
            ],
        };
        let result = owner.finish();
        assert!(joined.load(std::sync::atomic::Ordering::Acquire));
        assert!(
            matches!(result, Err(MathRuntimeError::Panic(message)) if message.contains("joined payload destructor failed"))
        );
    }

    #[test]
    fn scientific_action_and_worker_cleanup_failures_both_survive_finish() {
        let owner = JoinedPool::new(1, 2 << 20, |thread| {
            Ok(std::thread::spawn(move || {
                thread.run();
                panic!("action cleanup worker failed");
            }))
        })
        .unwrap();
        let action = owner
            .install(|| Err::<(), _>(MathRuntimeError::Cancelled))
            .map_err(MathRuntimeError::from)
            .and_then(|result| result);
        let result = MathRuntimeError::settle(action, owner.finish());
        assert!(
            matches!(result, Err(MathRuntimeError::Settlement { operation, cleanup })
            if matches!(*operation, MathRuntimeError::Cancelled)
            && matches!(*cleanup, MathRuntimeError::Panic(ref message) if message.contains("action cleanup worker failed")))
        );
    }

    #[test]
    fn partial_team_failure_joins_every_started_worker_including_failed_sibling() {
        let (exiting, exited) = std::sync::mpsc::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let (finished, outcome) = std::sync::mpsc::channel();
        // Always release the owned destructor before joining on assertion failure.
        struct Release(Option<std::sync::mpsc::Sender<()>>);
        impl Drop for Release {
            fn drop(&mut self) {
                if let Some(release) = self.0.take() {
                    let _ = release.send(());
                }
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
        exited
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        assert!(
            matches!(
                outcome.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Empty)
            ),
            "failed sibling cannot bypass the remaining native worker join"
        );
        release.0.take().unwrap().send(()).unwrap();
        let result = outcome
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        assert!(
            matches!(result, Err(MathRuntimeError::Settlement {operation, cleanup})
                if matches!(*operation, MathRuntimeError::Math(MathError::Library(ref message)) if message.contains("partial team construction refused"))
                && matches!(*cleanup, MathRuntimeError::Panic(ref message) if message.contains("first native team worker")))
        );
        construction.join().unwrap();
    }
}
