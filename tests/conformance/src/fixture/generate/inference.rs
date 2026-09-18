// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual selected tuple and guard evidence, independent of rule evaluation.
use super::{Rows, id, put, set};
use pse_schema::{
    Registry,
    model::{Cell, InvariantSpec},
};

#[expect(
    clippy::too_many_lines,
    reason = "complete target fixture construction and its independent assertions are kept in execution order"
)]
pub(super) fn populate(
    registry: &Registry,
    invariant: &InvariantSpec,
    valid: &mut Rows,
    invalid: &mut Rows,
) -> bool {
    let relation = invariant.relation.as_str();
    match invariant.name.as_str() {
        "active_read_coordinate" => {
            valid.insert(relation.to_owned(), vec![]);
            set(invalid, relation, "guard_source_id", Cell::Null);
            set(invalid, relation, "guard_node_id", Cell::Null);
            set(invalid, relation, "outer_guard_source_id", Cell::Null);
            set(invalid, relation, "outer_guard_node_id", Cell::Null);
        }
        "provides_matches_complete_contract" | "requires_matches_complete_contract" => {
            let field = if invariant.name.starts_with("provides") {
                "provides"
            } else {
                "requires"
            };
            set(valid, relation, field, Cell::List(vec![]));
            *invalid = valid.clone();
            set(invalid, relation, field, Cell::List(vec![id(1)]));
        }
        "child_guards_decided" | "all_state_targets_bound" => {
            valid.insert(relation.to_owned(), vec![]);
        }
        "containment_acyclic" => {
            set(valid, relation, "descendant_id", id(2));
            *invalid = valid.clone();
            set(invalid, relation, "descendant_id", id(1));
        }
        "actual_demand_target" | "actual_scope_binding" => {
            let target = if invariant.name == "actual_demand_target" {
                "inferred.demand_seed_bindings"
            } else {
                "inferred.demand_obligations"
            };
            put(registry, valid, target, 1);
            *invalid = valid.clone();
            invalid.insert(target.to_owned(), vec![]);
        }
        "demanded_method_resolved" => {
            set(
                valid,
                relation,
                "outcome",
                Cell::Struct(vec![Cell::Enum("resolved"), Cell::Struct(vec![id(1)])]),
            );
            *invalid = valid.clone();
            set(
                invalid,
                relation,
                "outcome",
                Cell::Struct(vec![Cell::Enum("unresolved"), Cell::Null]),
            );
        }
        "selection_compatible" => {
            set(valid, relation, "compatible", Cell::Bool(true));
            *invalid = valid.clone();
            set(invalid, relation, "compatible", Cell::Bool(false));
        }
        "material_bounds_satisfied" | "feature_resolved" | "feature_rule_satisfied" => {
            set(valid, relation, "truth", Cell::Enum("true"));
            *invalid = valid.clone();
            set(invalid, relation, "truth", Cell::Enum("false"));
        }
        "relative_target_present" => {
            set(valid, relation, "op", Cell::Enum("self"));
            set(valid, relation, "target_entity_id", id(1));
            *invalid = valid.clone();
            set(invalid, relation, "target_entity_id", Cell::Null);
        }
        "kind_has_actual_universe" => {
            set(valid, relation, "op", Cell::Enum("kind_is"));
            set(valid, relation, "target_kind", Cell::Enum("instance"));
            *invalid = valid.clone();
            set(invalid, relation, "target_kind", Cell::Null);
        }
        "actual_guard_tuple"
        | "guard_not_conflict"
        | "actual_outer_guard"
        | "outer_guard_not_conflict" => {
            let outer = invariant.name.contains("outer");
            let source = if outer {
                "outer_guard_source_id"
            } else {
                "guard_source_id"
            };
            let node = if outer {
                "outer_guard_node_id"
            } else {
                "guard_node_id"
            };
            set(valid, relation, source, id(1));
            // Local predicate node IDs remain unsigned until the coherent math cut.
            set(valid, relation, node, Cell::I64(1));
            let outcomes = "inferred.predicate_outcomes";
            put(registry, valid, outcomes, 1);
            set(valid, outcomes, "outcome", Cell::Enum("true"));
            if outer {
                set(valid, outcomes, "index", Cell::List(vec![]));
            }
            *invalid = valid.clone();
            if invariant.name.starts_with("actual") {
                invalid.insert(outcomes.to_owned(), vec![]);
            } else {
                set(invalid, outcomes, "outcome", Cell::Enum("conflict"));
            }
        }
        _ => return false,
    }
    true
}
