// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native identity projections over the complete prospective demand inventory.
use super::{RegistryBuilder, assertion};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (head, name) in [
        ("inferred.state_scope_keys", "state_scope_key_assertions"),
        (
            "inferred.state_method_selection_keys",
            "state_method_selection_key_assertions",
        ),
        (
            "inferred.demand_request_keys",
            "demand_request_key_assertions",
        ),
    ] {
        assertion(builder, head, name);
    }
}
