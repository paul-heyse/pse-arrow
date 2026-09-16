// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Positive demand/support closure consumes a fully settled finite winner relation.
use super::{Cell, E, P, RegistryBuilder, eq, filter, join, literal, null, project, rule, scan};
pub(super) mod seeds;

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    seeds::declare(builder);
    rule(
        builder,
        "P6.seed_requirement",
        5,
        "inferred.property_requirements",
        "provenance.property_requirement_assertions",
        project(
            scan("inferred.demand_seed_bindings", "seeds"),
            vec![
                ("requirement_id", E::col("requirement_id")),
                ("state_scope_id", E::col("state_scope_id")),
                ("property_kind_id", E::col("property_kind_id")),
                ("index", E::col("index")),
                ("derivation_id", E::col("requirement_id")),
            ],
        ),
        false,
    );
    rule(
        builder,
        "P6.seed_support",
        5,
        "inferred.requirement_support",
        "provenance.requirement_support_assertions",
        project(
            scan("inferred.demand_seed_bindings", "seeds"),
            vec![
                ("requirement_id", E::col("requirement_id")),
                ("requester_id", E::col("requester_id")),
                ("source_kind", literal("seed")),
                ("derivation_id", E::col("requester_id")),
            ],
        ),
        false,
    );
    let selected = join(
        scan("inferred.property_requirements", "required"),
        filter(
            scan("inferred.potential_method_resolutions", "resolution"),
            eq(E::col("status"), literal("resolved")),
        ),
        vec![("required.requirement_id", "resolution.requirement_id")],
    );
    let target = join(
        selected,
        scan("inferred.dependency_key_maps", "maps"),
        vec![
            ("required.requirement_id", "maps.requirement_id"),
            ("resolution.method_id", "maps.method_id"),
        ],
    );
    let target = join(
        target,
        scan("inferred.requirement_universe", "target"),
        vec![("maps.target_requirement_id", "target.requirement_id")],
    );
    rule(
        builder,
        "P6.dependency_requirement",
        5,
        "inferred.property_requirements",
        "provenance.property_requirement_assertions",
        project(
            target.clone(),
            vec![
                ("requirement_id", E::col("target.requirement_id")),
                ("state_scope_id", E::col("target.state_scope_id")),
                ("property_kind_id", E::col("target.property_kind_id")),
                ("index", E::col("target.index")),
                ("derivation_id", E::col("target.requirement_id")),
            ],
        ),
        false,
    );
    rule(
        builder,
        "P6.dependency_support",
        5,
        "inferred.requirement_support",
        "provenance.requirement_support_assertions",
        project(
            target,
            vec![
                ("requirement_id", E::col("target.requirement_id")),
                ("requester_id", E::col("required.requirement_id")),
                ("source_kind", literal("requirement")),
                ("derivation_id", E::col("required.requirement_id")),
            ],
        ),
        false,
    );
    let candidates = join(
        scan("inferred.property_requirements", "required"),
        scan("inferred.potential_method_candidates", "candidates"),
        vec![("required.requirement_id", "candidates.requirement_id")],
    );
    rule(
        builder,
        "P6.demanded_candidates",
        6,
        "inferred.method_candidates",
        "provenance.method_candidate_assertions",
        project(
            candidates,
            vec![
                ("requirement_id", E::col("candidates.requirement_id")),
                ("selection_id", E::col("candidates.selection_id")),
                ("method_id", E::col("candidates.method_id")),
                ("applicable", E::col("candidates.applicable")),
                ("rank", E::col("candidates.rank")),
                ("reason", E::col("candidates.reason")),
                ("derivation_id", E::col("candidates.derivation_id")),
            ],
        ),
        false,
    );
    let resolution = join(
        scan("inferred.property_requirements", "required"),
        scan("inferred.potential_method_resolutions", "resolved"),
        vec![("required.requirement_id", "resolved.requirement_id")],
    );
    rule(
        builder,
        "P6.demanded_resolutions",
        6,
        "inferred.method_resolutions",
        "provenance.method_resolution_assertions",
        project(
            resolution,
            vec![
                ("requirement_id", E::col("resolved.requirement_id")),
                ("method_id", E::col("resolved.method_id")),
                ("realization", E::col("resolved.realization")),
                ("template_id", E::col("resolved.template_id")),
                ("status", E::col("resolved.status")),
                ("derivation_id", E::col("resolved.derivation_id")),
            ],
        ),
        false,
    );
}
