// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Custody(Arc<AtomicUsize>);
fn check_custody(custody: &Custody, path: &str) -> Result<(), ValidationError> {
    if custody.0.load(Ordering::SeqCst) == 0 {
        Ok(())
    } else {
        Err(ValidationError::Rule {
            path: path.into(),
            rule: "test.custody",
            reason: "original owner was released".into(),
        })
    }
}
crate::projection_struct! {
    struct NativeObserved {
        projection: { name: "NativeObserved", version: 3, direction: Direction::Both,
            construction: ObservationOnly, reserved_tags: [], reserved_names: [] };
        recorded: String => { tag: 7, direction: Direction::Output,
            rules: [LocalRule::Length { min: 1, max: 8 }] };
        @local {
            custody: Custody => check: check_custody;
            retained: std::marker::PhantomPinned;
        }
    }
}
impl Drop for NativeObserved {
    fn drop(&mut self) {
        self.custody.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn observation_struct_borrows_nonclone_drop_owner_and_keeps_local_custody() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut native = NativeObserved {
        recorded: "actual".into(),
        custody: Custody(drops.clone()),
        retained: std::marker::PhantomPinned,
    };
    let graph = ProjectionGraph::of::<NativeObserved>().unwrap();
    assert_eq!(
        graph.definitions()["NativeObserved"].construction,
        ConstructionRole::ObservationOnly
    );
    assert_eq!(NativeObserved::FIELD_LAYOUT, &[(7, "recorded")]);
    assert_eq!(native.observation_fields().unwrap(), ("actual".to_owned(),));
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(
        Arc::strong_count(&drops),
        2,
        "snapshot never clones local custody"
    );
    let _retained = &native.retained;
    native.validate_projection().unwrap();
    drops.store(1, Ordering::SeqCst);
    assert!(
        matches!(native.validate_projection(), Err(ValidationError::Rule { path, rule: "test.custody", .. }) if path == "custody")
    );
    drops.store(0, Ordering::SeqCst);
    native.recorded.clear();
    assert!(
        native.validate_projection().is_err(),
        "one source declaration still owns local rules"
    );
    drop(native);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn explicit_data_role_retains_actual_reconstruction_independently_of_direction() {
    crate::projection_struct! {
        struct PassiveData {
            projection: { name: "PassiveData", version: 1, direction: Direction::Output,
                construction: Data, reserved_tags: [], reserved_names: [] };
            recorded: u64 => { tag: 1, direction: Direction::Output, rules: [] };
        }
    }
    let graph = ProjectionGraph::of::<PassiveData>().unwrap();
    assert_eq!(
        graph.definitions()["PassiveData"].construction,
        ConstructionRole::Data
    );
    let native = PassiveData::from_projection_fields((42,)).unwrap();
    assert_eq!(native.into_projection_fields(), (42,));
}
