use super::Storage;
use crate::DatabaseImpl;

#[test]
fn ordinary_clones_keep_independent_local_cancellation() {
    let root = Storage::<DatabaseImpl>::default();
    let child = root.clone();
    child.zalsa_local.cancellation_token().cancel();
    assert!(child.zalsa_local.should_trigger_local_cancellation());
    assert!(!root.zalsa_local.should_trigger_local_cancellation());
    assert!(
        !child
            .clone()
            .zalsa_local
            .should_trigger_local_cancellation(),
        "ordinary clone behavior remains unchanged"
    );
}

#[test]
fn nested_group_clones_preserve_local_suppression_and_monotonic_group_request() {
    let root = Storage::<DatabaseImpl>::default();
    let (member, group) = root.fork_cancellation_group();
    let child = member.clone();
    let grandchild = child.clone();
    let (other, other_group) = root.fork_cancellation_group();
    let old = member.zalsa_local.set_cancellation_disabled(true);
    assert!(!old);
    group.cancel();
    assert!(group.is_cancelled());
    assert!(
        !member.zalsa_local.should_trigger_local_cancellation(),
        "this handle retains local suppression"
    );
    assert!(
        child.zalsa_local.should_trigger_local_cancellation(),
        "parent suppression must not suppress its child"
    );
    assert!(grandchild.zalsa_local.should_trigger_local_cancellation());
    assert!(!root.zalsa_local.should_trigger_local_cancellation());
    assert!(!other.zalsa_local.should_trigger_local_cancellation());
    assert!(!other_group.is_cancelled());
    child.zalsa_local.uncancel();
    assert!(
        child.zalsa_local.should_trigger_local_cancellation(),
        "local reset cannot clear the group request"
    );
    child.zalsa_local.set_cancellation_disabled(true);
    assert!(!child.zalsa_local.should_trigger_local_cancellation());
    assert!(
        grandchild.zalsa_local.should_trigger_local_cancellation(),
        "child suppression cannot mask another handle"
    );
    child.zalsa_local.set_cancellation_disabled(false);
    member.zalsa_local.set_cancellation_disabled(old);
    assert!(member.zalsa_local.should_trigger_local_cancellation());
    drop(child);
    assert!(
        grandchild.zalsa_local.should_trigger_local_cancellation(),
        "dropping a member cannot reset group cancellation"
    );
    assert!(
        grandchild
            .clone()
            .zalsa_local
            .should_trigger_local_cancellation(),
        "later descendants inherit a requested group"
    );
    let (next, next_group) = root.fork_cancellation_group();
    assert!(!next_group.is_cancelled());
    assert!(!next.zalsa_local.should_trigger_local_cancellation());
    assert!(
        crate::sync::Arc::ptr_eq(&root.handle.zalsa_impl, &member.handle.zalsa_impl),
        "the branch shares original input/query storage, not a second database world"
    );
}
