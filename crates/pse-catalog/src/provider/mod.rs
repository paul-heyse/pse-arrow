// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native provider hierarchy and immutable semantic bindings.

pub(crate) mod binding;
pub mod catalog;
pub mod list;
pub mod schema;

/// A boxed, `Send` future with an explicit lifetime: the hand-desugared form of an
/// `async fn` in a trait implementation.
///
/// `async-trait` is not used in library code here. The macro boxes every call, hides the
/// `Send` bound behind an attribute, and makes the resulting signature something a reader
/// has to reconstruct before they can tell whether it matches the trait it implements.
/// DataFusion's own async trait methods return exactly this shape, so writing it out
/// costs one type alias and buys a signature that says what it is.
pub type BoxFut<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(test)]
mod tests {
    use super::*;

    /// A trait with a hand-desugared async method, standing in for `SchemaProvider`.
    trait Resolves {
        /// Resolves a name, without touching a disk.
        fn resolve<'a>(&'a self, name: &'a str) -> BoxFut<'a, Option<&'a str>>;
    }

    struct Pinned(Vec<String>);

    impl Resolves for Pinned {
        fn resolve<'a>(&'a self, name: &'a str) -> BoxFut<'a, Option<&'a str>> {
            Box::pin(async move {
                self.0
                    .iter()
                    .find(|held| held.as_str() == name)
                    .map(String::as_str)
            })
        }
    }

    #[test]
    fn a_snapshot_lookup_completes_on_its_first_poll() {
        let pinned = Pinned(vec!["units".to_owned()]);
        assert_eq!(poll_once(pinned.resolve("units")), Some("units"));
        assert_eq!(poll_once(pinned.resolve("other")), None);
    }

    /// Polls a future exactly once, asserting that it was already finished.
    ///
    /// This *is* the §5.4 claim under test: `table()` is `async` by signature and
    /// "resolves from the manifest loaded at session creation and performs no I/O". A
    /// future that satisfies that never yields, so one poll against a no-op waker is a
    /// complete executor for it — and a future that did reach a disk would fail here
    /// rather than quietly lengthen every plan.
    fn poll_once<T>(future: BoxFut<'_, T>) -> T {
        use std::task::{Context, Poll, Waker};

        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let mut future = future;
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("a snapshot lookup must not yield"),
        }
    }
}
