#[cfg(test)]
mod allocation_controls {
    use super::allocation::__buffa::{view::{RootView, ChildView}, lazy_view::RootLazyView};
    use buffa::{DecodeContext, DecodeError, MessageView, LazyMessageView};
    use core::{cell::Cell, mem::size_of};

    fn eager(bytes: &[u8], budget: usize) -> Result<usize, DecodeError> {
        let fields = Cell::new(1000);
        let budget = Cell::new(budget);
        let before = budget.get();
        let ctx = DecodeContext::new(100, &fields).with_allocation_budget(&budget);
        let _view = RootView::decode_view_ctx(bytes, ctx)?;
        Ok(before - budget.get())
    }
    fn lazy(bytes: &[u8], budget: usize) -> Result<usize, DecodeError> {
        let fields = Cell::new(1000);
        let budget = Cell::new(budget);
        let before = budget.get();
        let ctx = DecodeContext::new(100, &fields).with_allocation_budget(&budget);
        let _view = RootLazyView::decode_lazy_with_ctx(bytes, ctx)?;
        Ok(before - budget.get())
    }
    fn exact(bytes: &[u8], eager_cost: usize, lazy_cost: usize) {
        for (decode, cost) in [(eager as fn(&[u8], usize) -> _, eager_cost), (lazy as fn(&[u8], usize) -> _, lazy_cost)] {
            assert_eq!(decode(bytes, cost), Ok(cost));
            if cost > 0 { assert_eq!(decode(bytes, cost - 1), Err(DecodeError::AllocationLimitExceeded)); }
        }
    }
    #[test]
    fn ordinary_imported_stock_decoders_refuse_and_lazy_traversal_checks_child_capability() {
        use super::allocation::__buffa::{view::ImportedRootView, lazy_view::ImportedRootLazyView};
        use super::ordinary::__buffa::{view::LegacyView, lazy_view::LegacyLazyView};
        assert!(!LegacyView::allocation_accounted());
        assert!(!LegacyLazyView::allocation_accounted());
        for bytes in [&[10,0][..], &[18,0], &[26,0], &[34,0]] {
            let fields = Cell::new(1000);
            let budget = Cell::new(100000);
            let ctx = DecodeContext::new(100, &fields).with_allocation_budget(&budget);
            assert!(matches!(ImportedRootView::decode_view_ctx(bytes, ctx), Err(DecodeError::UnaccountedView)));
            assert!(ImportedRootView::decode_view(bytes).is_ok());
            if bytes[0] >= 26 {
                assert!(matches!(ImportedRootLazyView::decode_lazy_with_ctx(bytes, ctx), Err(DecodeError::UnaccountedView)));
            }
        }
        let fields = Cell::new(1000);
        let budget = Cell::new(100000);
        let ctx = DecodeContext::new(100, &fields).with_allocation_budget(&budget);
        // Recording this raw child is accounted; it has not decoded the child.
        let parent = ImportedRootLazyView::decode_lazy_with_ctx(&[10,0], ctx).unwrap();
        assert!(ImportedRootLazyView::allocation_accounted());
        assert_eq!(parent.singular.fragments(), [&[][..]]);
        // An explicit global traversal checks the deferred decoder BEFORE
        // invoking it. Default getters remain outside this shared phase.
        assert_eq!(ctx.require_allocation_accounted(LegacyLazyView::allocation_accounted()), Err(DecodeError::UnaccountedView));
        assert!(matches!(LegacyView::decode_view_ctx(&[], ctx), Err(DecodeError::UnaccountedView)));
    }
    #[test]
    fn accounted_external_decoders_are_accepted_in_all_eager_carveouts() {
        use super::allocation::__buffa::{view::AccountedRootView, lazy_view::AccountedRootLazyView};
        use super::accounted::__buffa::{view::PreparedView, lazy_view::PreparedLazyView};
        assert!(PreparedView::allocation_accounted());
        assert!(PreparedLazyView::allocation_accounted());
        for bytes in [&[10,0][..], &[18,0], &[26,0], &[34,0]] {
            let fields = Cell::new(1000);
            let budget = Cell::new(100000);
            let ctx = DecodeContext::new(100, &fields).with_allocation_budget(&budget);
            AccountedRootView::decode_view_ctx(bytes, ctx).unwrap();
            AccountedRootLazyView::decode_lazy_with_ctx(bytes, ctx).unwrap();
            assert!(budget.get() < 100000);
        }
    }
    #[test]
    fn external_decoders_refuse_in_all_selected_routes_and_remain_stock_without_budget() {
        use super::allocation::__buffa::{view::ExternalRootView, lazy_view::ExternalRootLazyView};
        for bytes in [&[10,0][..], &[18,0], &[26,0], &[34,0]] {
            let fields = Cell::new(1000);
            let budget = Cell::new(100000);
            let ctx = DecodeContext::new(100, &fields).with_allocation_budget(&budget);
            assert!(matches!(ExternalRootView::decode_view_ctx(bytes, ctx), Err(DecodeError::UnaccountedView)));
            assert!(matches!(ExternalRootLazyView::decode_lazy_with_ctx(bytes, ctx), Err(DecodeError::UnaccountedView)));
            assert!(ExternalRootView::decode_view(bytes).is_ok());
            assert!(ExternalRootLazyView::decode_lazy(bytes).is_ok());
        }
    }
    #[test]
    fn repeated_empty_messages_fragments_and_shared_siblings() {
        let child = size_of::<ChildView>();
        let span = size_of::<&[u8]>();
        exact(&[18,0,18,0,18,0], child * 7, span * 7);
        exact(&[10,0,10,0,10,0], child, span * 6);
        // Singular child and sibling repeated vector must draw from one cell.
        assert_eq!(eager(&[10,0,18,0], child), Err(DecodeError::AllocationLimitExceeded));
    }
    #[test]
    fn packed_unpacked_fixedwidth_and_duplicate_map_entries() {
        exact(&[26,3,1,2,3], 3 * size_of::<u64>(), 3 * size_of::<u64>());
        exact(&[24,1,24,2,24,3], 7 * size_of::<u64>(), 7 * size_of::<u64>());
        let mut doubles = vec![74,16]; doubles.extend_from_slice(&[0;16]);
        exact(&doubles, 16, 16);
        // All three duplicate entries are retained in MapView's wire store.
        let entry = size_of::<(&str, ChildView)>();
        exact(&[34,0,34,0,34,0], entry * 7, entry * 7);
    }
    #[test]
    fn oneof_replacement_charges_new_box_merge_reuses_it_and_groups_charge() {
        let child = size_of::<ChildView>();
        exact(&[42,0,42,0], child, child);
        exact(&[42,0,50,0,42,0], child * 3, child * 3);
        // Group value has its own concrete backing type, exercised for exact
        // fit/refusal independently of any wire-byte multiplier.
        let bytes = [59,8,1,60];
        let cost = eager(&bytes, usize::MAX).unwrap();
        assert!(cost > 0);
        exact(&bytes, cost, cost);
    }
    #[test]
    fn closed_enum_unknowns_and_coalesced_unknown_groups_execute() {
        let packed = [66,3,0,99,1];
        let cost = eager(&packed, usize::MAX).unwrap();
        assert!(cost > 3 * size_of::<i32>());
        exact(&packed, cost, cost);
        // Unknown group with a nested unknown field, followed by an adjacent
        // unknown varint: coalescing retains one span while allowances count all.
        let bytes = [83,8,1,84,88,1];
        let cost = eager(&bytes, usize::MAX).unwrap();
        exact(&bytes, cost, cost);
        assert!(RootView::decode_view(&bytes).is_ok());
    }
}
