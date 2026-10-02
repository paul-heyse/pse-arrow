// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry-lifetime memoization with explicit construction and wait dependencies.
use crate::SchemaError;
use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread::ThreadId,
};
type Value = Arc<dyn Any + Send + Sync>;
type Key = (TypeId, u64);
enum Slot {
    Building { owner: ThreadId, name: String },
    Complete(Result<Value, SchemaError>),
}
#[derive(Default)]
struct State {
    slots: HashMap<Key, Slot>,
    active: HashMap<ThreadId, Vec<(Key, String)>>,
    waits: HashMap<ThreadId, (ThreadId, String)>,
}
#[derive(Default)]
/// Shared construction slots with typed namespaces and owner-local dynamic keys.
pub struct ImplementationCache {
    owner: Arc<()>,
    state: Mutex<State>,
    changed: Condvar,
}
impl std::fmt::Debug for ImplementationCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImplementationCache")
            .finish_non_exhaustive()
    }
}
impl ImplementationCache {
    pub(crate) fn owner(&self) -> Arc<()> {
        Arc::clone(&self.owner)
    }
    /// Construct the one typed implementation in the default slot.
    /// # Errors
    /// Construction, lock poisoning or a construction dependency cycle.
    pub fn get<T: Any + Send + Sync>(
        &self,
        build: impl FnOnce() -> Result<T, SchemaError>,
    ) -> Result<Arc<T>, SchemaError> {
        self.get_at(0, build)
    }
    /// Construct a typed implementation at an owner-local dynamic slot.
    /// The owner must assign equal keys exactly when all construction inputs are equal.
    /// # Errors
    /// Construction, lock poisoning or a construction dependency cycle.
    pub fn get_at<T: Any + Send + Sync>(
        &self,
        slot: u64,
        build: impl FnOnce() -> Result<T, SchemaError>,
    ) -> Result<Arc<T>, SchemaError> {
        let key = (TypeId::of::<T>(), slot);
        let name = format!("{}[{slot}]", type_name::<T>());
        let thread = std::thread::current().id();
        let mut state = self
            .state
            .lock()
            .map_err(|_| invalid("implementation owner lock poisoned"))?;
        loop {
            match state.slots.get(&key) {
                Some(Slot::Complete(value)) => {
                    return value
                        .clone()?
                        .downcast()
                        .map_err(|_| invalid("implementation key/type mismatch"));
                }
                Some(Slot::Building {
                    owner,
                    name: target,
                }) => {
                    let owner = *owner;
                    let target = target.clone();
                    let mut cursor = owner;
                    let mut keys = vec![target.to_owned()];
                    if owner == thread {
                        if let Some(stack) = state.active.get(&thread) {
                            keys = stack
                                .iter()
                                .skip_while(|(id, _)| *id != key)
                                .map(|(_, name)| name.clone())
                                .collect();
                            keys.push(name.to_owned());
                        }
                        return Err(SchemaError::ImplementationCycle { keys });
                    }
                    while let Some((next, name)) = state.waits.get(&cursor) {
                        keys.push(name.clone());
                        cursor = *next;
                        if cursor == thread {
                            break;
                        }
                    }
                    if cursor == thread {
                        if let Some(stack) = state.active.get(&thread)
                            && let Some((_, name)) = stack.last()
                        {
                            keys.push(name.clone());
                        }
                        return Err(SchemaError::ImplementationCycle { keys });
                    }
                    state.waits.insert(thread, (owner, target));
                    state = self
                        .changed
                        .wait(state)
                        .map_err(|_| invalid("implementation wait lock poisoned"))?;
                    state.waits.remove(&thread);
                }
                None => {
                    state.slots.insert(
                        key,
                        Slot::Building {
                            owner: thread,
                            name: name.clone(),
                        },
                    );
                    state
                        .active
                        .entry(thread)
                        .or_default()
                        .push((key, name.clone()));
                    break;
                }
            }
        }
        drop(state);
        let mut construction = Construction {
            cache: self,
            key,
            thread,
            finished: false,
        };
        let value = build().map(|value| -> Value { Arc::new(value) });
        construction.finish(value.clone())?;
        value?
            .downcast()
            .map_err(|_| invalid("implementation key/type mismatch"))
    }
}
struct Construction<'a> {
    cache: &'a ImplementationCache,
    key: Key,
    thread: ThreadId,
    finished: bool,
}
impl Construction<'_> {
    fn finish(&mut self, value: Result<Value, SchemaError>) -> Result<(), SchemaError> {
        let mut state = self
            .cache
            .state
            .lock()
            .map_err(|_| invalid("implementation completion lock poisoned"))?;
        state.slots.insert(self.key, Slot::Complete(value));
        if let Some(stack) = state.active.get_mut(&self.thread) {
            stack.pop();
            if stack.is_empty() {
                state.active.remove(&self.thread);
            }
        }
        state.waits.remove(&self.thread);
        self.finished = true;
        self.cache.changed.notify_all();
        Ok(())
    }
}
impl Drop for Construction<'_> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.finish(Err(invalid("implementation construction interrupted")));
        }
    }
}
fn invalid(reason: &str) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: "derived implementation".into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct A;
    struct B;
    #[test]
    fn nested_distinct_slots_and_same_chain_cycle() {
        let cache = ImplementationCache::default();
        cache
            .get(|| {
                cache.get(|| Ok(B))?;
                Ok(A)
            })
            .unwrap();
        let cache = ImplementationCache::default();
        let error = cache
            .get::<A>(|| {
                cache.get::<B>(|| {
                    cache.get::<A>(|| Ok(A))?;
                    Ok(B)
                })?;
                Ok(A)
            })
            .err()
            .unwrap();
        assert!(matches!(error, SchemaError::ImplementationCycle { keys } if keys.len() == 3));
    }
    #[test]
    fn concurrent_waiters_share_completion() {
        let cache = Arc::new(ImplementationCache::default());
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        std::thread::scope(|scope| {
            let tasks = (0..8)
                .map(|_| {
                    let cache = cache.clone();
                    let count = count.clone();
                    scope.spawn(move || {
                        cache
                            .get(|| {
                                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                Ok(A)
                            })
                            .unwrap()
                    })
                })
                .collect::<Vec<_>>();
            let owners = tasks
                .into_iter()
                .map(|task| task.join().unwrap())
                .collect::<Vec<_>>();
            assert!(owners.iter().all(|owner| Arc::ptr_eq(owner, &owners[0])));
        });
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
    #[test]
    fn cross_worker_wait_cycle_is_refused() {
        let cache = Arc::new(ImplementationCache::default());
        let barrier = Arc::new(std::sync::Barrier::new(2));
        std::thread::scope(|scope| {
            let a = scope.spawn(|| {
                cache.get::<A>(|| {
                    barrier.wait();
                    cache.get::<B>(|| Ok(B))?;
                    Ok(A)
                })
            });
            let b = scope.spawn(|| {
                cache.get::<B>(|| {
                    barrier.wait();
                    cache.get::<A>(|| Ok(A))?;
                    Ok(B)
                })
            });
            assert!(matches!(
                a.join().unwrap(),
                Err(SchemaError::ImplementationCycle { .. })
            ));
            assert!(matches!(
                b.join().unwrap(),
                Err(SchemaError::ImplementationCycle { .. })
            ));
        });
    }
    #[test]
    fn interrupted_constructor_releases_waiters() {
        let cache = ImplementationCache::default();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cache.get::<A>(|| panic!("interrupted"))
        }));
        assert!(cache.get::<A>(|| Ok(A)).is_err());
        cache.get::<B>(|| Ok(B)).unwrap();
    }
}
