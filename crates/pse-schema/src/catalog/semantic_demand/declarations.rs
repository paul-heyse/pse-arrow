// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Identity framing is separate from relational validity and applicability.
use super::{N, RegistryBuilder, S, T, assertion, column, index, provenance, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    framing(builder);
    requests(builder);
    reads(builder);
    assertion(
        builder,
        "inferred.demand_scope_requests",
        "demand_scope_request_assertions",
    );
    assertion(
        builder,
        "inferred.demand_active_reads",
        "demand_active_read_assertions",
    );
    assertion(
        builder,
        "inferred.demand_obligations",
        "demand_obligation_assertions",
    );
    selection(builder);
    for (head, name) in [
        ("inferred.state_scopes", "state_scope_assertions"),
        (
            "inferred.demand_seed_bindings",
            "demand_seed_binding_assertions",
        ),
        (
            "inferred.requirement_universe",
            "requirement_universe_assertions",
        ),
        (
            "inferred.selection_inventory",
            "selection_inventory_assertions",
        ),
        (
            "inferred.potential_method_candidates",
            "potential_method_candidate_assertions",
        ),
        (
            "inferred.potential_method_winners",
            "potential_method_winner_assertions",
        ),
        (
            "inferred.potential_method_resolutions",
            "potential_method_resolution_assertions",
        ),
        (
            "inferred.property_requirements",
            "property_requirement_assertions",
        ),
        (
            "inferred.requirement_support",
            "requirement_support_assertions",
        ),
        ("inferred.method_candidates", "method_candidate_assertions"),
        (
            "inferred.method_resolutions",
            "method_resolution_assertions",
        ),
    ] {
        assertion(builder, head, name);
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one declarative catalog family keeps its native rules and field declarations together"
)]
fn framing(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "state_dependency_keys",
        S::Derived,
        &["requirement_id", "method_id", "dependency_ordinal", "index"],
        vec![
            column("requirement_id", T::id()),
            column("method_id", T::id()),
            column("dependency_ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("symbol_decl_id", T::id()),
            column("product_id", T::id()),
            column("index", index()),
            provenance(),
        ],
        "Replayed exact state-symbol dependency coordinate maps; rules require the declaration to belong to the actual selected state template.",
    );
    relation(
        builder,
        N::Inferred,
        "state_scope_keys",
        S::Derived,
        &["state_instance_id"],
        vec![
            column("state_instance_id", T::id()),
            column("state_scope_id", T::id()),
            provenance(),
        ],
        "Replayed singleton identity framing for prospective actual instances; no state eligibility claim.",
    );
    relation(
        builder,
        N::Inferred,
        "state_scopes",
        S::Derived,
        &["state_scope_id"],
        vec![
            column("state_scope_id", T::id()),
            column("state_instance_id", T::id()),
            column("property_package_id", T::id()),
            provenance(),
        ],
        "One actual selected state-definition instance per singleton scope, independent of the selecting authored scope.",
    );
    relation(
        builder,
        N::Inferred,
        "requirement_keys",
        S::Derived,
        &["requirement_id"],
        vec![
            column("requirement_id", T::id()),
            column("state_scope_id", T::id()),
            column("state_instance_id", T::id()),
            column("property_kind_id", T::id()),
            column("index", index()),
            column("product_id", T::id()),
            column("shape", T::list(T::enumeration("DomainKind"))),
            provenance(),
        ],
        "Finite framed prospective ordered requirement tuples; validity is established by member and state joins.",
    );
    relation(
        builder,
        N::Inferred,
        "requirement_key_axes",
        S::Derived,
        &["requirement_id", "position"],
        vec![
            column("requirement_id", T::id()),
            column("position", T::nonnegative(i64::from(u16::MAX))),
            column("domain_id", T::id()),
            column("member_id", T::id()),
            column("kind", T::enumeration("DomainKind")),
            column("subject_id", T::id()).optional(),
            provenance(),
        ],
        "Complete actual ordered domain/member map behind each prospective requirement index.",
    );
    relation(
        builder,
        N::Inferred,
        "requirement_scope_keys",
        S::Derived,
        &["requirement_id", "scope_kind", "scope_ids"],
        vec![
            column("requirement_id", T::id()),
            column("scope_kind", T::enumeration("ScopeKind")),
            column("scope_ids", index()),
            provenance(),
        ],
        "Structural source-subject tuples for exact package/phase/species/phase-species/reaction selection scope comparison.",
    );
    relation(
        builder,
        N::Inferred,
        "requirement_universe",
        S::Derived,
        &["requirement_id"],
        vec![
            column("requirement_id", T::id()),
            column("state_scope_id", T::id()),
            column("state_instance_id", T::id()),
            column("property_package_id", T::id()),
            column("property_kind_id", T::id()),
            column("index", index()),
            provenance(),
        ],
        "Potential demands with actual eligible state and every index member admitted; no demand asserted yet.",
    );
    relation(
        builder,
        N::Inferred,
        "dependency_key_maps",
        S::Derived,
        &[
            "requirement_id",
            "method_id",
            "dependency_ordinal",
            "target_requirement_id",
        ],
        vec![
            column("requirement_id", T::id()),
            column("method_id", T::id()),
            column("dependency_ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("target_requirement_id", T::id()),
            provenance(),
        ],
        "Replayed finite tagged dependency index mapping; the selected method and actual target universe decide traversal.",
    );
    relation(
        builder,
        N::Inferred,
        "method_parameter_keys",
        S::Derived,
        &["requirement_id", "method_id", "name", "index"],
        vec![
            column("requirement_id", T::id()),
            column("method_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("index", index()),
            provenance(),
        ],
        "Actual finite parameter tuple framing for the declared complete method parameter signature.",
    );
}
fn selection(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "selection_inventory",
        S::Derived,
        &["selection_id"],
        vec![
            column("selection_id", T::id()),
            column("is_default", T::native(arrow_schema::DataType::Boolean)),
            column("property_package_id", T::id()),
            column("scope_kind", T::enumeration("ScopeKind")),
            column("scope_ids", index()),
            column("property_kind_id", T::id()).optional(),
            column("family", T::enumeration("MethodFamily")),
            column("method_id", T::id()),
            column("source_kind", T::native(arrow_schema::DataType::Utf8)),
            provenance(),
        ],
        "Complete authored selections and explicitly selected state-definition methods with distinguishable actual support.",
    );
    relation(
        builder,
        N::Inferred,
        "state_method_selection_keys",
        S::Derived,
        &["property_package_id"],
        vec![
            column("property_package_id", T::id()),
            column("selection_id", T::id()),
            provenance(),
        ],
        "Identity framing for the direct selected state-method source; never presented as an authored method-selection row.",
    );
    let Some(mut candidate) = builder
        .declared_relations()
        .iter()
        .find(|r| r.key.qualified_name() == "inferred.method_candidates")
        .cloned()
    else {
        return;
    };
    candidate.key.name = "potential_method_candidates";
    candidate.doc = "Complete lower-stratum candidate classification over the finite requirement universe, before positive demand closure.";
    builder.declare_relation(candidate);
    relation(
        builder,
        N::Inferred,
        "potential_method_winners",
        S::Derived,
        &["requirement_id", "selection_id", "method_id"],
        vec![
            column("requirement_id", T::id()),
            column("selection_id", T::id()),
            column("method_id", T::id()),
            column("rank", T::nonnegative(i64::from(u16::MAX))),
            provenance(),
        ],
        "All greatest-rank actual applicable selections; equal-ranked distinct methods remain ambiguous.",
    );
    let Some(mut resolution) = builder
        .declared_relations()
        .iter()
        .find(|r| r.key.qualified_name() == "inferred.method_resolutions")
        .cloned()
    else {
        return;
    };
    resolution.key.name = "potential_method_resolutions";
    resolution.doc = "Complete static resolution before recursive demand; missing and ambiguous winners remain explicit.";
    builder.declare_relation(resolution);
}
fn requests(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "demand_request_keys",
        S::Derived,
        &["seed_id", "requester_instance_id"],
        vec![
            column("seed_id", T::id()),
            column("requester_instance_id", T::id()),
            column("requester_id", T::id()),
            provenance(),
        ],
        "Replayed source-seed and actual prospective occurrence identity; rules establish active source ownership.",
    );
    relation(
        builder,
        N::Inferred,
        "demand_seed_bindings",
        S::Derived,
        &["read_id", "requirement_id"],
        vec![
            column("read_id", T::id()),
            column("seed_id", T::id()),
            column("requester_id", T::id()),
            column("requester_instance_id", T::id()),
            column("requirement_id", T::id()),
            column("state_scope_id", T::id()),
            column("property_kind_id", T::id()),
            column("index", index()),
            provenance(),
        ],
        "Exact source demand occurrences with actual scope/member and guarded tuple correspondence.",
    );
}
fn reads(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "demand_scope_requests",
        S::Derived,
        &["read_id", "property_kind_id", "scope_decl_id"],
        vec![
            column("read_id", T::id()),
            column("seed_id", T::id()),
            column("requester_instance_id", T::id()),
            column("scope_decl_id", T::id()),
            column("owner_instance_id", T::id()),
            column("property_kind_id", T::id()),
            provenance(),
        ],
        "Actual read's explicit scope declaration and occurrence owner before resolving relative or global membership.",
    );
    relation(
        builder,
        N::Inferred,
        "demand_obligations",
        S::Derived,
        &["read_id", "property_kind_id", "scope_id"],
        vec![
            column("read_id", T::id()),
            column("seed_id", T::id()),
            column("requester_instance_id", T::id()),
            column("scope_id", T::id()),
            column("property_kind_id", T::id()),
            provenance(),
        ],
        "Every active property-annotated read retains its actual selected scope even if no eligible target tuple exists.",
    );
    let columns = vec![
        column("read_id", T::id()),
        column("seed_id", T::id()),
        column("requester_instance_id", T::id()),
        column("owner_instance_id", T::id()),
        column("symbol_decl_id", T::id()).optional(),
        column("index", index()),
        column("guard_source_id", T::id()).optional(),
        column("guard_node_id", T::nonnegative(i64::MAX)).optional(),
        column("guard_index", index()),
        column("outer_guard_source_id", T::id()).optional(),
        column("outer_guard_node_id", T::nonnegative(i64::MAX)).optional(),
        provenance(),
    ];
    relation(
        builder,
        N::Inferred,
        "demand_read_keys",
        S::Derived,
        &["read_id"],
        columns.clone(),
        "Replayed exact read coordinates and projected predicate tuple under complete finite source binder assignments; no demand or guard truth is asserted.",
    );
    relation(
        builder,
        N::Inferred,
        "demand_active_reads",
        S::Derived,
        &["read_id"],
        columns,
        "Actual unguarded or exactly true/unknown read occurrences; false branches do not demand their symbols.",
    );
    relation(
        builder,
        N::Inferred,
        "demand_index_maps",
        S::Derived,
        &["read_id", "requirement_id"],
        vec![
            column("read_id", T::id()),
            column("requirement_id", T::id()),
            provenance(),
        ],
        "Structural ordered source-domain/member correspondence to prospective requirement tuples, before actual property and scope admission.",
    );
}
