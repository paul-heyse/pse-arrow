// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conservative container storage, separate from nested owned payload and work budgets.

/// One conservative node extent for the pinned standard library's B-tree.
///
/// `alloc::collections::btree::node` uses eleven key/value slots and twelve child
/// links (B = 6). Sixteen slots, thirty-two words and two alignment paddings cover
/// both leaf/internal headers, edges and rounding, including over-aligned fields.
/// Charging a full node per entry deliberately covers sparse occupancy; this is
/// a source-derived admission bound, not a claim about actual allocator usage.
/// The toolchain owner must re-derive it if that node representation changes.
pub fn btree_node_bytes<K, V>() -> usize {
    let alignment = align_of::<K>()
        .max(align_of::<V>())
        .max(align_of::<usize>());
    size_of::<K>()
        .saturating_add(size_of::<V>())
        .saturating_mul(16)
        .saturating_add(32 * size_of::<usize>())
        .saturating_add(alignment.saturating_mul(2))
}

/// Complete conservative B-tree node population, excluding nested key/value payload.
///
/// At most one full node per entry plus one root/split node covers construction
/// overlap and retention. An empty tree can retain an allocated root after removal,
/// so its conservative extent is one node. A never-created temporary may be omitted
/// by its construction owner. Arithmetic overflow refuses before allocation.
pub fn btree_allocation_bound<K, V>(entries: usize) -> Option<usize> {
    entries
        .checked_add(1)?
        .checked_mul(btree_node_bytes::<K, V>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HeapUsage;
    use std::collections::{BTreeMap, BTreeSet};

    #[repr(align(4096))]
    struct Aligned;

    #[test]
    fn btree_bound_covers_sparse_roots_alignment_and_overflow() {
        assert!(btree_allocation_bound::<(), ()>(0).unwrap() >= 12 * size_of::<usize>());
        assert!(btree_node_bytes::<Aligned, ()>() >= 2 * align_of::<Aligned>());
        assert!(btree_allocation_bound::<u64, u64>(usize::MAX).is_none());
        assert!(btree_allocation_bound::<u64, u64>(usize::MAX / 2).is_none());
        let one = btree_allocation_bound::<u64, u64>(1).unwrap();
        let many = btree_allocation_bound::<u64, u64>(100).unwrap();
        assert!(many > one);
    }

    #[test]
    fn btree_retention_counts_nested_capacity_and_an_empty_retained_root() {
        let mut payload = Vec::<u8>::with_capacity(101);
        payload.extend([1, 2, 3]);
        let mut tree = BTreeMap::from([(String::from("key"), payload)]);
        assert_eq!(
            tree.heap_bytes(),
            btree_allocation_bound::<String, Vec<u8>>(1).unwrap() + 3 + 101
        );
        tree.clear();
        assert_eq!(tree.heap_bytes(), btree_node_bytes::<String, Vec<u8>>());
        let set = BTreeSet::from([String::from("payload")]);
        assert_eq!(
            set.heap_bytes(),
            btree_allocation_bound::<String, ()>(1).unwrap() + 7
        );
    }
}
