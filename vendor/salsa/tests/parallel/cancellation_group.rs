#![cfg(not(feature = "shuttle"))]
//! Real tracked computations in nested Rayon database descendants.
use salsa::{Cancelled, Database};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

struct Gates {
    entered: mpsc::Sender<u32>,
    released: Barrier,
    block: AtomicBool,
}

#[salsa::db]
trait GroupDatabase: Database {
    fn gates(&self) -> &Gates;
}

#[salsa::db]
#[derive(Clone)]
struct GroupDb {
    storage: salsa::Storage<Self>,
    gates: Arc<Gates>,
}
#[salsa::db]
impl salsa::Database for GroupDb {}
#[salsa::db]
impl GroupDatabase for GroupDb {
    fn gates(&self) -> &Gates {
        &self.gates
    }
}
impl GroupDb {
    fn request(&self) -> (Self, salsa::CancellationGroup) {
        let (storage, group) = self.storage.fork_cancellation_group();
        (
            Self {
                storage,
                gates: self.gates.clone(),
            },
            group,
        )
    }
}

#[salsa::input]
struct Key {
    value: u32,
}

#[salsa::tracked(returns(copy))]
fn query(db: &dyn GroupDatabase, key: Key) -> u32 {
    let key = *key.value(db);
    if key < 2 && db.gates().block.load(Ordering::Acquire) {
        db.gates().entered.send(key).unwrap();
        db.gates().released.wait();
    }
    db.unwind_if_revision_cancelled();
    key + 40
}

#[test]
fn cancellation_reaches_nested_rayon_queries_and_preserves_independent_requests() {
    let (entered, receiver) = mpsc::channel();
    let gates = Arc::new(Gates {
        entered,
        released: Barrier::new(3),
        block: AtomicBool::new(true),
    });
    let root = GroupDb {
        storage: Default::default(),
        gates: gates.clone(),
    };
    let zero = Key::new(&root, 0);
    let one = Key::new(&root, 1);
    let two = Key::new(&root, 2);
    let (request, cancellation) = root.request();
    let child = request.clone();
    let grandchild = child.clone();
    let (sibling, sibling_cancellation) = root.request();
    let worker = std::thread::spawn(move || {
        rayon::join(
            move || Cancelled::catch(|| query(&child, zero)),
            move || Cancelled::catch(|| query(&grandchild, one)),
        )
    });
    let mut actual = std::collections::BTreeSet::new();
    for _ in 0..2 {
        actual.insert(
            receiver
                .recv_timeout(std::time::Duration::from_secs(30))
                .expect("both actual tracked query bodies must enter before cancellation"),
        );
    }
    assert_eq!(actual, std::collections::BTreeSet::from([0, 1]));
    assert_eq!(
        query(&sibling, two),
        42,
        "another request can execute while the selected children are active"
    );
    cancellation.cancel();
    gates.released.wait();
    let (first, second) = worker.join().unwrap();
    assert!(matches!(first, Err(Cancelled::Local)));
    assert!(matches!(second, Err(Cancelled::Local)));
    assert!(!sibling_cancellation.is_cancelled());
    gates.block.store(false, Ordering::Release);
    assert_eq!(
        query(&sibling, zero),
        40,
        "an independent request can recompute the same canceled memo"
    );
    let (next, next_cancellation) = root.request();
    assert!(!next_cancellation.is_cancelled());
    assert_eq!(query(&next, one), 41);
    assert!(
        cancellation.is_cancelled(),
        "settled/dropped descendants never clear the group's monotonic request"
    );
}
