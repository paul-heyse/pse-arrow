// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native DataFusion programs for the finite semantic strata.
use crate::{
    RegistryBuilder,
    model::{DependencyMode, RuleDecl, RuleInput},
};

#[expect(
    clippy::too_many_lines,
    reason = "one declarative catalog family keeps its native rules and field declarations together"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    builder.declare_rule(
        RuleDecl::new(
            "P4.count_empty_phases",
            "1",
            0,
            "inferred.material_member_counts",
            include_str!("native_rules/p4-count-empty-phases.sql"),
            vec![RuleInput {
                relation: "normalized.material_systems".into(),
                port: "materials",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.material_count_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.count_empty_species",
            "1",
            0,
            "inferred.material_member_counts",
            include_str!("native_rules/p4-count-empty-species.sql"),
            vec![RuleInput {
                relation: "normalized.material_systems".into(),
                port: "materials",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.material_count_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.count_phases",
            "1",
            0,
            "inferred.material_member_counts",
            include_str!("native_rules/p4-count-phases.sql"),
            vec![RuleInput {
                relation: "normalized.material_systems".into(),
                port: "materials",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.material_count_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.count_species",
            "1",
            0,
            "inferred.material_member_counts",
            include_str!("native_rules/p4-count-species.sql"),
            vec![RuleInput {
                relation: "normalized.material_systems".into(),
                port: "materials",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.material_count_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_excludes",
            "1",
            2,
            "inferred.feature_checks",
            include_str!("native_rules/p4-feature-excludes.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instance_features".into(),
                    port: "left_feature",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.instance_features".into(),
                    port: "right_feature",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_feature_rules".into(),
                    port: "declarations",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.feature_check_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_implies",
            "1",
            0,
            "inferred.feature_candidates",
            include_str!("native_rules/p4-feature-implies.sql"),
            vec![
                RuleInput {
                    relation: "inferred.feature_candidates".into(),
                    port: "features",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_feature_rules".into(),
                    port: "implications",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.feature_candidate_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_inherit",
            "1",
            0,
            "inferred.feature_candidates",
            include_str!("native_rules/p4-feature-inherit.sql"),
            vec![
                RuleInput {
                    relation: "inferred.feature_candidates".into(),
                    port: "features",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.feature_inheritance".into(),
                    port: "inheritance",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.feature_candidate_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_missing",
            "1",
            2,
            "inferred.feature_requirements",
            include_str!("native_rules/p4-feature-missing.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instance_features".into(),
                    port: "features",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_features".into(),
                    port: "declarations",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.feature_requirement_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_requires",
            "1",
            2,
            "inferred.feature_checks",
            include_str!("native_rules/p4-feature-requires.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instance_features".into(),
                    port: "left_feature",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.instance_features".into(),
                    port: "right_feature",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_feature_rules".into(),
                    port: "declarations",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.feature_check_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_resolve",
            "1",
            1,
            "inferred.instance_features",
            include_str!("native_rules/p4-feature-resolve.sql"),
            vec![RuleInput {
                relation: "inferred.feature_candidates".into(),
                port: "features",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.instance_feature_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.feature_seed",
            "1",
            0,
            "inferred.feature_candidates",
            include_str!("native_rules/p4-feature-seed.sql"),
            vec![RuleInput {
                relation: "normalized.config_values".into(),
                port: "config",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.feature_candidate_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.material_template_check",
            "1",
            1,
            "inferred.material_template_checks",
            include_str!("native_rules/p4-material-template-check.sql"),
            vec![
                RuleInput {
                    relation: "inferred.material_member_counts".into(),
                    port: "phases",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.material_member_counts".into(),
                    port: "species",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_packages".into(),
                    port: "packages",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_material_constraints".into(),
                    port: "constraints",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.material_check_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.method_absent",
            "1",
            0,
            "inferred.method_compatibility",
            include_str!("native_rules/p4-method-absent.sql"),
            vec![
                RuleInput {
                    relation: "normalized.method_selections".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.method_compatibility_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.method_compatible",
            "1",
            0,
            "inferred.method_compatibility",
            include_str!("native_rules/p4-method-compatible.sql"),
            vec![
                RuleInput {
                    relation: "normalized.method_selections".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.method_compatibility_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.method_family_mismatch",
            "1",
            0,
            "inferred.method_compatibility",
            include_str!("native_rules/p4-method-family-mismatch.sql"),
            vec![
                RuleInput {
                    relation: "normalized.method_selections".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.method_compatibility_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.method_missing_provision",
            "1",
            0,
            "inferred.method_compatibility",
            include_str!("native_rules/p4-method-missing-provision.sql"),
            vec![
                RuleInput {
                    relation: "normalized.method_selections".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.method_selections".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.method_compatibility_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.phase_species_henry",
            "1",
            0,
            "inferred.phase_species",
            include_str!("native_rules/p4-phase-species-henry.sql"),
            vec![
                RuleInput {
                    relation: "normalized.henry_declarations".into(),
                    port: "henry",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_systems".into(),
                    port: "phase_members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_systems".into(),
                    port: "species_members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.phase_species".into(),
                    port: "restrictions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.phase_species".into(),
                    port: "restrictions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.phases".into(),
                    port: "phases",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.species".into(),
                    port: "species",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.phase_species_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P4.phase_species_ordinary",
            "1",
            0,
            "inferred.phase_species",
            include_str!("native_rules/p4-phase-species-ordinary.sql"),
            vec![
                RuleInput {
                    relation: "normalized.henry_declarations".into(),
                    port: "henry",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.material_systems".into(),
                    port: "phase_members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_systems".into(),
                    port: "species_members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.phase_species".into(),
                    port: "restrictions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.phase_species".into(),
                    port: "restrictions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.phases".into(),
                    port: "phases",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.species".into(),
                    port: "species",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.phase_species_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.boundary_external",
            "1",
            19,
            "inferred.boundary_crossings",
            include_str!("native_rules/p5-boundary-external.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_port_decisions".into(),
                    port: "from",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_port_decisions".into(),
                    port: "to",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.topology_edges".into(),
                    port: "edges",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.boundary_crossing_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.boundary_inbound",
            "1",
            19,
            "inferred.boundary_crossings",
            include_str!("native_rules/p5-boundary-inbound.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_port_decisions".into(),
                    port: "from",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_port_decisions".into(),
                    port: "to",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.topology_edges".into(),
                    port: "edges",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.boundary_crossing_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.boundary_internal",
            "1",
            19,
            "inferred.boundary_crossings",
            include_str!("native_rules/p5-boundary-internal.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_port_decisions".into(),
                    port: "from",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_port_decisions".into(),
                    port: "to",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.topology_edges".into(),
                    port: "edges",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.boundary_crossing_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.boundary_outbound",
            "1",
            19,
            "inferred.boundary_crossings",
            include_str!("native_rules/p5-boundary-outbound.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_port_decisions".into(),
                    port: "from",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_port_decisions".into(),
                    port: "to",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.topology_edges".into(),
                    port: "edges",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.boundary_crossing_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.connection_direction",
            "1",
            17,
            "inferred.connection_violations",
            include_str!("native_rules/p5-connection-direction.sql"),
            vec![
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "from",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "to",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.connection_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.connection_from_missing",
            "1",
            17,
            "inferred.connection_violations",
            include_str!("native_rules/p5-connection-from-missing.sql"),
            vec![
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.connection_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.connection_kind",
            "1",
            17,
            "inferred.connection_violations",
            include_str!("native_rules/p5-connection-kind.sql"),
            vec![
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "from",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "to",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.connection_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.connection_member_domains",
            "1",
            17,
            "inferred.connection_violations",
            include_str!("native_rules/p5-connection-member-domains.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_member_domains".into(),
                    port: "left_domains",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_member_domains".into(),
                    port: "right_domains",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_members".into(),
                    port: "other",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.connection_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.connection_member_from_missing",
            "1",
            17,
            "inferred.connection_violations",
            include_str!("native_rules/p5-connection-member-from-missing.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_members".into(),
                    port: "other",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.connection_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.connection_member_physical",
            "1",
            17,
            "inferred.connection_violations",
            include_str!("native_rules/p5-connection-member-physical.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_member_domains".into(),
                    port: "left_domains",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_member_domains".into(),
                    port: "right_domains",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_members".into(),
                    port: "other",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "left_quantity",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "right_quantity",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.connection_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.connection_member_to_missing",
            "1",
            17,
            "inferred.connection_violations",
            include_str!("native_rules/p5-connection-member-to-missing.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_members".into(),
                    port: "other",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.connection_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.connection_to_missing",
            "1",
            17,
            "inferred.connection_violations",
            include_str!("native_rules/p5-connection-to-missing.sql"),
            vec![
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.connection_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.containment_depth",
            "1",
            10,
            "inferred.instance_tree",
            include_str!("native_rules/p5-containment-depth.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instance_reachability".into(),
                    port: "paths",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.instance_reachability".into(),
                    port: "prefix",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.instance_reachability".into(),
                    port: "suffix",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.instance_tree_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.containment_direct",
            "1",
            9,
            "inferred.instance_reachability",
            include_str!("native_rules/p5-containment-direct.sql"),
            vec![RuleInput {
                relation: "inferred.instances".into(),
                port: "instances",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.instance_reachability_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.containment_transitive",
            "1",
            9,
            "inferred.instance_reachability",
            include_str!("native_rules/p5-containment-transitive.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instance_reachability".into(),
                    port: "paths",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "children",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.instance_reachability_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.domain_element",
            "1",
            5,
            "inferred.domain_eligible_members",
            include_str!("native_rules/p5-domain-element.sql"),
            vec![
                RuleInput {
                    relation: "inferred.phase_species".into(),
                    port: "allowed",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_domain_members".into(),
                    port: "material",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.species_elements".into(),
                    port: "composition",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.domain_member_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.domain_ordinary",
            "1",
            5,
            "inferred.domain_eligible_members",
            include_str!("native_rules/p5-domain-ordinary.sql"),
            vec![
                RuleInput {
                    relation: "normalized.domain_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_domain_members".into(),
                    port: "material",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.domain_member_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.domain_phase",
            "1",
            5,
            "inferred.domain_eligible_members",
            include_str!("native_rules/p5-domain-phase.sql"),
            vec![
                RuleInput {
                    relation: "normalized.material_domain_members".into(),
                    port: "material",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.phases".into(),
                    port: "phases",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.domain_member_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.domain_phase_species",
            "1",
            5,
            "inferred.domain_eligible_members",
            include_str!("native_rules/p5-domain-phase-species.sql"),
            vec![
                RuleInput {
                    relation: "inferred.phase_species".into(),
                    port: "allowed",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_domain_members".into(),
                    port: "material",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.domain_member_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.domain_species",
            "1",
            5,
            "inferred.domain_eligible_members",
            include_str!("native_rules/p5-domain-species.sql"),
            vec![
                RuleInput {
                    relation: "inferred.phase_species".into(),
                    port: "allowed",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_domain_members".into(),
                    port: "material",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.domain_member_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.global_scope_candidates",
            "1",
            6,
            "inferred.scope_candidates",
            include_str!("native_rules/p5-global-scope-candidates.sql"),
            vec![
                RuleInput {
                    relation: "normalized.scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_scopes".into(),
                    port: "declarations",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.scope_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.global_scopes",
            "1",
            13,
            "inferred.resolved_scopes",
            include_str!("native_rules/p5-global-scopes.sql"),
            vec![RuleInput {
                relation: "inferred.scope_candidates".into(),
                port: "scopes",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.scope_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.guard_malformed",
            "1",
            9,
            "inferred.instance_guard_violations",
            include_str!("native_rules/p5-guard-malformed.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "parents",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_binding_products".into(),
                    port: "binding_products",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "prospective",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.instance_guard_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.guard_missing",
            "1",
            9,
            "inferred.instance_guard_violations",
            include_str!("native_rules/p5-guard-missing.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "parents",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.predicate_outcomes".into(),
                    port: "guards",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_binding_products".into(),
                    port: "binding_products",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "prospective",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.instance_guard_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.guard_unresolved",
            "1",
            9,
            "inferred.instance_guard_violations",
            include_str!("native_rules/p5-guard-unresolved.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "parents",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.predicate_outcomes".into(),
                    port: "guards",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_binding_products".into(),
                    port: "binding_products",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "prospective",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.instance_guard_violation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.index_invalid_member",
            "1",
            6,
            "inferred.invalid_index_tuples",
            include_str!("native_rules/p5-index-invalid-member.sql"),
            vec![
                RuleInput {
                    relation: "inferred.domain_eligible_members".into(),
                    port: "eligible",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.candidate_index_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.invalid_index_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.index_invalid_phase_species_pair",
            "1",
            6,
            "inferred.invalid_index_tuples",
            include_str!("native_rules/p5-index-invalid-phase-species-pair.sql"),
            vec![
                RuleInput {
                    relation: "inferred.phase_species".into(),
                    port: "allowed",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.candidate_index_members".into(),
                    port: "phase_members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.candidate_index_members".into(),
                    port: "species_members",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_domain_members".into(),
                    port: "phases",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.material_domain_members".into(),
                    port: "species",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.invalid_index_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.index_valid",
            "1",
            7,
            "inferred.valid_index_tuples",
            include_str!("native_rules/p5-index-valid.sql"),
            vec![
                RuleInput {
                    relation: "inferred.invalid_index_tuples".into(),
                    port: "invalid",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.candidate_index_tuples".into(),
                    port: "candidates",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.valid_index_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.instance_guarded",
            "1",
            8,
            "inferred.instances",
            include_str!("native_rules/p5-instance-guarded.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "parents",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.predicate_outcomes".into(),
                    port: "guards",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_binding_products".into(),
                    port: "binding_products",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "prospective",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.instance_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.instance_roots",
            "1",
            8,
            "inferred.instances",
            include_str!("native_rules/p5-instance-roots.sql"),
            vec![
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_binding_products".into(),
                    port: "binding_products",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "prospective",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.instance_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.instance_unguarded",
            "1",
            8,
            "inferred.instances",
            include_str!("native_rules/p5-instance-unguarded.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "parents",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_binding_products".into(),
                    port: "binding_products",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "prospective",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.instance_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.port_member_domains",
            "1",
            12,
            "inferred.port_member_domains",
            include_str!("native_rules/p5-port-member-domains.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_member_domain_candidates".into(),
                    port: "candidates",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.port_member_domain_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.port_members",
            "1",
            12,
            "inferred.port_members",
            include_str!("native_rules/p5-port-members.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_member_candidates".into(),
                    port: "candidates",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.port_member_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.port_state_domains",
            "1",
            12,
            "inferred.port_state_domains",
            include_str!("native_rules/p5-port-state-domains.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_state_domain_candidates".into(),
                    port: "candidates",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.port_state_domain_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.port_state_targets",
            "1",
            12,
            "inferred.port_state_targets",
            include_str!("native_rules/p5-port-state-targets.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "states",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_state_candidates".into(),
                    port: "targets",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_state_domain_candidates".into(),
                    port: "domains",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.port_state_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.ports",
            "1",
            11,
            "inferred.ports",
            include_str!("native_rules/p5-ports.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_candidates".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_state_candidates".into(),
                    port: "targets",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_state_domain_candidates".into(),
                    port: "domains",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.unbound_port_targets".into(),
                    port: "missing",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.port_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.relative_scope_candidates",
            "1",
            6,
            "inferred.scope_candidates",
            include_str!("native_rules/p5-relative-scope-candidates.sql"),
            vec![
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_scopes".into(),
                    port: "declarations",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.scope_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.relative_scopes",
            "1",
            13,
            "inferred.resolved_scopes",
            include_str!("native_rules/p5-relative-scopes.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_candidates".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.scope_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_bindings",
            "1",
            14,
            "inferred.scope_bindings",
            include_str!("native_rules/p5-scope-bindings.sql"),
            vec![RuleInput {
                relation: "inferred.resolved_scopes".into(),
                port: "scopes",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.scope_binding_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_descendant_ports",
            "1",
            13,
            "inferred.scope_reachability",
            include_str!("native_rules/p5-scope-descendant-ports.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instance_reachability".into(),
                    port: "paths",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.scope_reachability_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_direct_ports",
            "1",
            13,
            "inferred.scope_reachability",
            include_str!("native_rules/p5-scope-direct-ports.sql"),
            vec![RuleInput {
                relation: "inferred.ports".into(),
                port: "ports",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.scope_reachability_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_instance_descendants",
            "1",
            13,
            "inferred.scope_reachability",
            include_str!("native_rules/p5-scope-instance-descendants.sql"),
            vec![RuleInput {
                relation: "inferred.instance_reachability".into(),
                port: "paths",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.scope_reachability_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_instance_universe",
            "1",
            13,
            "inferred.scope_entities",
            include_str!("native_rules/p5-scope-instance-universe.sql"),
            vec![RuleInput {
                relation: "inferred.instances".into(),
                port: "instances",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.scope_entity_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_members",
            "1",
            15,
            "inferred.scope_members",
            include_str!("native_rules/p5-scope-members.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_decisions".into(),
                    port: "decisions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.selector_roots".into(),
                    port: "roots",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.scope_member_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_port_explicit",
            "1",
            16,
            "inferred.scope_port_states",
            include_str!("native_rules/p5-scope-port-explicit.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_state_targets".into(),
                    port: "targets",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.scope_port_state_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_port_no",
            "1",
            17,
            "inferred.scope_port_decisions",
            include_str!("native_rules/p5-scope-port-no.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_state_targets".into(),
                    port: "targets",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_port_states".into(),
                    port: "included",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.scope_port_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_port_owner",
            "1",
            16,
            "inferred.scope_port_states",
            include_str!("native_rules/p5-scope-port-owner.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_state_targets".into(),
                    port: "targets",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.scope_port_state_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_port_state",
            "1",
            16,
            "inferred.scope_port_states",
            include_str!("native_rules/p5-scope-port-state.sql"),
            vec![
                RuleInput {
                    relation: "inferred.port_state_targets".into(),
                    port: "targets",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.scope_port_state_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_port_universe",
            "1",
            13,
            "inferred.scope_entities",
            include_str!("native_rules/p5-scope-port-universe.sql"),
            vec![RuleInput {
                relation: "inferred.ports".into(),
                port: "ports",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.scope_entity_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.scope_port_yes",
            "1",
            17,
            "inferred.scope_port_decisions",
            include_str!("native_rules/p5-scope-port-yes.sql"),
            vec![RuleInput {
                relation: "inferred.scope_port_states".into(),
                port: "included",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.scope_port_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_absolute_context",
            "1",
            8,
            "inferred.selector_contexts",
            include_str!("native_rules/p5-selector-absolute-context.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_candidates".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.selector_nodes".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_context_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_constant",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-constant.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_entities".into(),
                    port: "entities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_descendant_no",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-descendant-no.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_entities".into(),
                    port: "entities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_reachability".into(),
                    port: "reach",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_descendant_yes",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-descendant-yes.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_entities".into(),
                    port: "entities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_reachability".into(),
                    port: "reach",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_difference",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-difference.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_decisions".into(),
                    port: "left",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_decisions".into(),
                    port: "right",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_exclude",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-exclude.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_entities".into(),
                    port: "entities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_identity",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-identity.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_decisions".into(),
                    port: "child",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_include",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-include.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_entities".into(),
                    port: "entities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_intersection",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-intersection.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_decisions".into(),
                    port: "left",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_decisions".into(),
                    port: "right",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_kind",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-kind.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_entities".into(),
                    port: "entities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_parameter",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-parameter.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_entities".into(),
                    port: "entities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_parameter_context",
            "1",
            8,
            "inferred.selector_contexts",
            include_str!("native_rules/p5-selector-parameter-context.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_candidates".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_parameter_targets".into(),
                    port: "targets",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.selector_nodes".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_context_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_parameter_missing",
            "1",
            8,
            "inferred.selector_contexts",
            include_str!("native_rules/p5-selector-parameter-missing.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_candidates".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_parameter_targets".into(),
                    port: "targets",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.selector_nodes".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_context_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_parameter_target",
            "1",
            7,
            "inferred.selector_parameter_targets",
            include_str!("native_rules/p5-selector-parameter-target.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_candidates".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.config_values".into(),
                    port: "values",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "owner",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "target",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.selector_nodes".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_params".into(),
                    port: "parameters",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.schema_logical_types".into(),
                    port: "logical",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_parameter_target_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_self",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-self.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_entities".into(),
                    port: "entities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_self_context",
            "1",
            8,
            "inferred.selector_contexts",
            include_str!("native_rules/p5-selector-self-context.sql"),
            vec![
                RuleInput {
                    relation: "inferred.scope_candidates".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.selector_nodes".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_context_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.selector_union",
            "1",
            14,
            "inferred.selector_decisions",
            include_str!("native_rules/p5-selector-union.sql"),
            vec![
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_contexts".into(),
                    port: "nodes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_decisions".into(),
                    port: "left",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selector_decisions".into(),
                    port: "right",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selector_decision_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.topology",
            "1",
            18,
            "inferred.topology_edges",
            include_str!("native_rules/p5-topology.sql"),
            vec![
                RuleInput {
                    relation: "inferred.connection_violations".into(),
                    port: "invalid",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "from",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.ports".into(),
                    port: "to",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.connections".into(),
                    port: "connections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.topology_edge_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P5.unbound_port_target",
            "1",
            10,
            "inferred.unbound_port_targets",
            include_str!("native_rules/p5-unbound-port-target.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "instances",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.port_candidates".into(),
                    port: "ports",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_state_candidates".into(),
                    port: "targets",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.port_state_domain_candidates".into(),
                    port: "domains",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "valid",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.unbound_port_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.active_read",
            "1",
            50,
            "inferred.demand_active_reads",
            include_str!("native_rules/p6-active-read.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_read_keys".into(),
                    port: "reads",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.predicate_outcomes".into(),
                    port: "guards",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.predicate_outcomes".into(),
                    port: "outer_guard",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_active_read_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.ambiguous_method",
            "1",
            54,
            "inferred.potential_method_resolutions",
            include_str!("native_rules/p6-ambiguous-method.sql"),
            vec![RuleInput {
                relation: "inferred.potential_method_winners".into(),
                port: "winners",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.potential_method_resolution_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.authored_selection_inventory",
            "1",
            50,
            "inferred.selection_inventory",
            include_str!("native_rules/p6-authored-selection-inventory.sql"),
            vec![RuleInput {
                relation: "normalized.method_selections".into(),
                port: "selections",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.selection_inventory_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.candidate_applicable",
            "1",
            52,
            "inferred.potential_method_candidates",
            include_str!("native_rules/p6-candidate-applicable.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_active_reads".into(),
                    port: "read_declarations",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.demand_seed_bindings".into(),
                    port: "bound_reads",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.dependency_key_maps".into(),
                    port: "dependency_maps",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.method_parameter_keys".into(),
                    port: "parameter_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "dependency_targets",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.state_dependency_keys".into(),
                    port: "state_maps",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "state_tuples",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "state_instances",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.parameter_values".into(),
                    port: "values",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "read_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "state_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_dependencies".into(),
                    port: "dependencies",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_parameters".into(),
                    port: "parameters",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_precedence".into(),
                    port: "precedence",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "expected_types",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "parameter_types",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "declared_units",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "parameter_units",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.potential_method_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.candidate_family_mismatch",
            "1",
            52,
            "inferred.potential_method_candidates",
            include_str!("native_rules/p6-candidate-family-mismatch.sql"),
            vec![
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.potential_method_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.candidate_incompatible_signature",
            "1",
            52,
            "inferred.potential_method_candidates",
            include_str!("native_rules/p6-candidate-incompatible-signature.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_active_reads".into(),
                    port: "read_declarations",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.demand_seed_bindings".into(),
                    port: "bound_reads",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "read_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "expected_types",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.potential_method_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.candidate_missing_dependency",
            "1",
            52,
            "inferred.potential_method_candidates",
            include_str!("native_rules/p6-candidate-missing-dependency.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_active_reads".into(),
                    port: "read_declarations",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.demand_seed_bindings".into(),
                    port: "bound_reads",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.dependency_key_maps".into(),
                    port: "dependency_maps",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "dependency_targets",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.state_dependency_keys".into(),
                    port: "state_maps",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "state_tuples",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "state_instances",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "read_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "state_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_dependencies".into(),
                    port: "dependencies",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "expected_types",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.potential_method_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.candidate_missing_parameter",
            "1",
            52,
            "inferred.potential_method_candidates",
            include_str!("native_rules/p6-candidate-missing-parameter.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_active_reads".into(),
                    port: "read_declarations",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.demand_seed_bindings".into(),
                    port: "bound_reads",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.dependency_key_maps".into(),
                    port: "dependency_maps",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.method_parameter_keys".into(),
                    port: "parameter_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "dependency_targets",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.state_dependency_keys".into(),
                    port: "state_maps",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "state_tuples",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "state_instances",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.parameter_values".into(),
                    port: "values",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "read_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "state_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_dependencies".into(),
                    port: "dependencies",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_parameters".into(),
                    port: "parameters",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "expected_types",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "parameter_types",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "declared_units",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "parameter_units",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.potential_method_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.candidate_missing_provision",
            "1",
            52,
            "inferred.potential_method_candidates",
            include_str!("native_rules/p6-candidate-missing-provision.sql"),
            vec![
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.potential_method_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.candidate_missing_rank",
            "1",
            52,
            "inferred.potential_method_candidates",
            include_str!("native_rules/p6-candidate-missing-rank.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_active_reads".into(),
                    port: "read_declarations",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.demand_seed_bindings".into(),
                    port: "bound_reads",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.dependency_key_maps".into(),
                    port: "dependency_maps",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.method_parameter_keys".into(),
                    port: "parameter_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "dependency_targets",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.state_dependency_keys".into(),
                    port: "state_maps",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "state_tuples",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "state_instances",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.parameter_values".into(),
                    port: "values",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "read_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "normalized.template_symbols".into(),
                    port: "state_symbols",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_dependencies".into(),
                    port: "dependencies",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_parameters".into(),
                    port: "parameters",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_precedence".into(),
                    port: "precedence",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.property_kinds".into(),
                    port: "properties",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "expected_types",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "parameter_types",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.quantity_types".into(),
                    port: "quantities",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "canonical",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "declared_units",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "natural",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.units".into(),
                    port: "parameter_units",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.potential_method_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.candidate_scope_mismatch",
            "1",
            52,
            "inferred.potential_method_candidates",
            include_str!("native_rules/p6-candidate-scope-mismatch.sql"),
            vec![
                RuleInput {
                    relation: "inferred.requirement_scope_keys".into(),
                    port: "scope_keys",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.selection_inventory".into(),
                    port: "selections",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.potential_method_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.demanded_candidates",
            "1",
            56,
            "inferred.method_candidates",
            include_str!("native_rules/p6-demanded-candidates.sql"),
            vec![
                RuleInput {
                    relation: "inferred.potential_method_candidates".into(),
                    port: "candidates",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.property_requirements".into(),
                    port: "required",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.method_candidate_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.demanded_resolutions",
            "1",
            56,
            "inferred.method_resolutions",
            include_str!("native_rules/p6-demanded-resolutions.sql"),
            vec![
                RuleInput {
                    relation: "inferred.potential_method_resolutions".into(),
                    port: "resolved",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.property_requirements".into(),
                    port: "required",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.method_resolution_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.dependency_requirement",
            "1",
            55,
            "inferred.property_requirements",
            include_str!("native_rules/p6-dependency-requirement.sql"),
            vec![
                RuleInput {
                    relation: "inferred.dependency_key_maps".into(),
                    port: "maps",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.potential_method_resolutions".into(),
                    port: "resolution",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.property_requirements".into(),
                    port: "required",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "target",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.property_requirement_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.dependency_support",
            "1",
            55,
            "inferred.requirement_support",
            include_str!("native_rules/p6-dependency-support.sql"),
            vec![
                RuleInput {
                    relation: "inferred.dependency_key_maps".into(),
                    port: "maps",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.potential_method_resolutions".into(),
                    port: "resolution",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.property_requirements".into(),
                    port: "required",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "target",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.requirement_support_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.direct_read_scope",
            "1",
            51,
            "inferred.demand_scope_requests",
            include_str!("native_rules/p6-direct-read-scope.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_active_reads".into(),
                    port: "reads",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_demand_seeds".into(),
                    port: "seeds",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_scope_request_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.global_scope_obligation",
            "1",
            51,
            "inferred.demand_obligations",
            include_str!("native_rules/p6-global-scope-obligation.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_scope_requests".into(),
                    port: "requests",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.resolved_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_obligation_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.greatest_rank",
            "1",
            53,
            "inferred.potential_method_winners",
            include_str!("native_rules/p6-greatest-rank.sql"),
            vec![
                RuleInput {
                    relation: "inferred.potential_method_candidates".into(),
                    port: "current",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.potential_method_candidates".into(),
                    port: "current",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.potential_method_candidates".into(),
                    port: "other",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.potential_method_winner_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.path_read_scope",
            "1",
            51,
            "inferred.demand_scope_requests",
            include_str!("native_rules/p6-path-read-scope.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_active_reads".into(),
                    port: "reads",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_path_demands".into(),
                    port: "demands",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.template_symbol_properties".into(),
                    port: "properties",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_scope_request_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.path_request_key",
            "1",
            48,
            "inferred.demand_request_keys",
            include_str!("native_rules/p6-path-request-key.sql"),
            vec![
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_path_demands".into(),
                    port: "seed",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_request_key_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.relative_scope_obligation",
            "1",
            51,
            "inferred.demand_obligations",
            include_str!("native_rules/p6-relative-scope-obligation.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_scope_requests".into(),
                    port: "requests",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_bindings".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_obligation_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.requirement_universe",
            "1",
            51,
            "inferred.requirement_universe",
            include_str!("native_rules/p6-requirement-universe.sql"),
            vec![
                RuleInput {
                    relation: "inferred.domain_eligible_members".into(),
                    port: "eligible",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_key_axes".into(),
                    port: "axes",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_keys".into(),
                    port: "keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.state_scopes".into(),
                    port: "scopes",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.valid_index_tuples".into(),
                    port: "tuples",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.requirement_universe_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.scoped_read_requirement",
            "1",
            51,
            "inferred.demand_seed_bindings",
            include_str!("native_rules/p6-scoped-read-requirement.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_index_maps".into(),
                    port: "maps",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.demand_obligations".into(),
                    port: "obligations",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.demand_request_keys".into(),
                    port: "requesters",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "required",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_members".into(),
                    port: "members",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_seed_binding_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.seed_request_key",
            "1",
            48,
            "inferred.demand_request_keys",
            include_str!("native_rules/p6-seed-request-key.sql"),
            vec![
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "instances",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_demand_seeds".into(),
                    port: "seed",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_request_key_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.seed_requirement",
            "1",
            55,
            "inferred.property_requirements",
            include_str!("native_rules/p6-seed-requirement.sql"),
            vec![RuleInput {
                relation: "inferred.demand_seed_bindings".into(),
                port: "seeds",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.property_requirement_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.seed_support",
            "1",
            55,
            "inferred.requirement_support",
            include_str!("native_rules/p6-seed-support.sql"),
            vec![RuleInput {
                relation: "inferred.demand_seed_bindings".into(),
                port: "seeds",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.requirement_support_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.state_method_selection_key",
            "1",
            48,
            "inferred.state_method_selection_keys",
            include_str!("native_rules/p6-state-method-selection-key.sql"),
            vec![RuleInput {
                relation: "normalized.property_packages".into(),
                port: "source",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.state_method_selection_key_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.state_provision_read_requirement",
            "1",
            51,
            "inferred.demand_seed_bindings",
            include_str!("native_rules/p6-state-provision-read-requirement.sql"),
            vec![
                RuleInput {
                    relation: "inferred.demand_active_reads".into(),
                    port: "reads",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.demand_index_maps".into(),
                    port: "maps",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.demand_request_keys".into(),
                    port: "requesters",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "required",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_packages".into(),
                    port: "packages",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_path_demands".into(),
                    port: "demands",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_provisions".into(),
                    port: "provisions",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.demand_seed_binding_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.state_scope",
            "1",
            50,
            "inferred.state_scopes",
            include_str!("native_rules/p6-state-scope.sql"),
            vec![
                RuleInput {
                    relation: "inferred.instances".into(),
                    port: "actual",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.state_scope_keys".into(),
                    port: "keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.instance_bindings".into(),
                    port: "bound",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_packages".into(),
                    port: "packages",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.state_scope_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.state_scope_key",
            "1",
            48,
            "inferred.state_scope_keys",
            include_str!("native_rules/p6-state-scope-key.sql"),
            vec![RuleInput {
                relation: "normalized.instance_bindings".into(),
                port: "source",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.state_scope_key_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.state_selection_inventory",
            "1",
            50,
            "inferred.selection_inventory",
            include_str!("native_rules/p6-state-selection-inventory.sql"),
            vec![
                RuleInput {
                    relation: "inferred.state_method_selection_keys".into(),
                    port: "keys",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "normalized.property_packages".into(),
                    port: "packages",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.selection_inventory_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.unique_method",
            "1",
            54,
            "inferred.potential_method_resolutions",
            include_str!("native_rules/p6-unique-method.sql"),
            vec![
                RuleInput {
                    relation: "inferred.potential_method_winners".into(),
                    port: "winners",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.method_specs".into(),
                    port: "methods",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.potential_method_resolution_assertions"),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P6.unsupported_method",
            "1",
            54,
            "inferred.potential_method_resolutions",
            include_str!("native_rules/p6-unsupported-method.sql"),
            vec![
                RuleInput {
                    relation: "inferred.potential_method_winners".into(),
                    port: "winners",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.requirement_universe".into(),
                    port: "requirements",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.potential_method_resolution_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_applications",
            "1",
            100,
            "compiled.law_applications",
            include_str!("native_rules/p8-law-applications.sql"),
            vec![
                RuleInput {
                    relation: "inferred.law_contexts".into(),
                    port: "context",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "reference.law_bindings".into(),
                    port: "binding",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.law_application_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_candidates",
            "1",
            101,
            "inferred.law_candidates",
            include_str!("native_rules/p8-law-candidates.sql"),
            vec![
                RuleInput {
                    relation: "compiled.contributions".into(),
                    port: "term",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "compiled.law_applications".into(),
                    port: "law",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_members".into(),
                    port: "scope",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.law_candidate_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_empty_applications",
            "1",
            105,
            "inferred.law_empty_applications",
            include_str!("native_rules/p8-law-empty-applications.sql"),
            vec![
                RuleInput {
                    relation: "compiled.law_applications".into(),
                    port: "law",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_ordered_terms".into(),
                    port: "ordered",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.law_empty_application_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_family_exclusion",
            "1",
            103,
            "inferred.law_participation_decisions",
            include_str!("native_rules/p8-law-family-exclusion.sql"),
            vec![
                RuleInput {
                    relation: "compiled.contributions".into(),
                    port: "term",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "compiled.law_applications".into(),
                    port: "law",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_candidates".into(),
                    port: "candidate",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.law_participation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_inclusion",
            "1",
            103,
            "inferred.law_participation_decisions",
            include_str!("native_rules/p8-law-inclusion.sql"),
            vec![
                RuleInput {
                    relation: "compiled.contributions".into(),
                    port: "term",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "compiled.law_applications".into(),
                    port: "law",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_candidates".into(),
                    port: "candidate",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_internal_transfers".into(),
                    port: "internal",
                    mode: DependencyMode::Negate,
                },
                RuleInput {
                    relation: "inferred.law_subject_matches".into(),
                    port: "subject",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.law_participation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_internal_exclusion",
            "1",
            103,
            "inferred.law_participation_decisions",
            include_str!("native_rules/p8-law-internal-exclusion.sql"),
            vec![
                RuleInput {
                    relation: "compiled.contributions".into(),
                    port: "term",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "compiled.law_applications".into(),
                    port: "law",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_candidates".into(),
                    port: "candidate",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_internal_transfers".into(),
                    port: "internal",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_subject_matches".into(),
                    port: "subject",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.law_participation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_internal_transfers",
            "1",
            102,
            "inferred.law_internal_transfers",
            include_str!("native_rules/p8-law-internal-transfers.sql"),
            vec![
                RuleInput {
                    relation: "compiled.contributions".into(),
                    port: "paired",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "compiled.contributions".into(),
                    port: "term",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "compiled.law_applications".into(),
                    port: "law",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.boundary_crossings".into(),
                    port: "cut",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_candidates".into(),
                    port: "candidate",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.scope_members".into(),
                    port: "paired_scope",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.law_internal_transfer_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_ordered_terms",
            "1",
            104,
            "inferred.law_ordered_terms",
            include_str!("native_rules/p8-law-ordered-terms.sql"),
            vec![RuleInput {
                relation: "inferred.law_participation_decisions".into(),
                port: "terms",
                mode: DependencyMode::Read,
            }],
        )
        .assertions("provenance.law_ordered_term_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_subject_exclusion",
            "1",
            103,
            "inferred.law_participation_decisions",
            include_str!("native_rules/p8-law-subject-exclusion.sql"),
            vec![
                RuleInput {
                    relation: "compiled.contributions".into(),
                    port: "term",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "compiled.law_applications".into(),
                    port: "law",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_candidates".into(),
                    port: "candidate",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_subject_matches".into(),
                    port: "subject",
                    mode: DependencyMode::Negate,
                },
            ],
        )
        .assertions("provenance.law_participation_assertions")
        .stratified_negation(),
    );
    builder.declare_rule(
        RuleDecl::new(
            "P8.law_subject_matches",
            "1",
            102,
            "inferred.law_subject_matches",
            include_str!("native_rules/p8-law-subject-matches.sql"),
            vec![
                RuleInput {
                    relation: "compiled.contributions".into(),
                    port: "term",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "compiled.law_applications".into(),
                    port: "law",
                    mode: DependencyMode::Read,
                },
                RuleInput {
                    relation: "inferred.law_candidates".into(),
                    port: "candidate",
                    mode: DependencyMode::Read,
                },
            ],
        )
        .assertions("provenance.law_subject_match_assertions")
        .stratified_negation(),
    );
}
