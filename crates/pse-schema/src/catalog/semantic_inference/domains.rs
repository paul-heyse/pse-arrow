// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual member admission precedes tuple difference; absence reads settled strata.
use super::{
    E, N, P, RegistryBuilder, RuleDecl, RuleHead, S, T, assertion, column, join, project,
    provenance, relation, scan,
};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "domain_eligible_members",
        S::Derived,
        &["domain_id", "member_id"],
        vec![
            column("domain_id", T::id()),
            column("member_id", T::id()),
            provenance(),
        ],
        "Actual finite members admitted by explicit material subject correspondence.",
    );
    relation(
        builder,
        N::Inferred,
        "invalid_index_tuples",
        S::Derived,
        &["product_id", "tuple"],
        vec![
            column("product_id", T::id()),
            column("tuple", T::extended(crate::model::ExtensionUse::IndexTuple)),
            provenance(),
        ],
        "Finite candidates with at least one member excluded by actual material membership.",
    );
    for (head, name) in [
        (
            "inferred.domain_eligible_members",
            "domain_member_assertions",
        ),
        ("inferred.invalid_index_tuples", "invalid_index_assertions"),
        ("inferred.valid_index_tuples", "valid_index_assertions"),
    ] {
        assertion(builder, head, name);
    }
    let material = scan("normalized.material_domain_members", "material");
    let ordinary = P::AntiJoin {
        left: Box::new(scan("normalized.domain_members", "members")),
        right: Box::new(material.clone()),
        keys: (vec![
            ("members.domain_id", "material.domain_id"),
            ("members.member_id", "material.member_id"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    member(
        builder,
        "P5.domain_ordinary",
        ordinary,
        "members.domain_id",
        "members.member_id",
        "members.member_id",
    );
    let phase = filtered(
        material.clone(),
        E::And(vec![
            nonnull("phase_id"),
            null("species_id"),
            null("element_id"),
        ]),
    );
    let phase = join(
        phase,
        scan("normalized.phases", "phases"),
        vec![("material.phase_id", "phases.phase_id")],
    );
    member(
        builder,
        "P5.domain_phase",
        phase,
        "material.domain_id",
        "material.member_id",
        "material.derivation_id",
    );
    let species = filtered(
        material.clone(),
        E::And(vec![
            null("phase_id"),
            nonnull("species_id"),
            null("element_id"),
        ]),
    );
    let species = join(
        species,
        scan("inferred.phase_species", "allowed"),
        vec![
            ("material.material_system_id", "allowed.material_system_id"),
            ("material.species_id", "allowed.species_id"),
        ],
    );
    member(
        builder,
        "P5.domain_species",
        species,
        "material.domain_id",
        "material.member_id",
        "material.derivation_id",
    );
    let pair = filtered(
        material.clone(),
        E::And(vec![
            nonnull("phase_id"),
            nonnull("species_id"),
            null("element_id"),
        ]),
    );
    let pair = join(
        pair,
        scan("inferred.phase_species", "allowed"),
        vec![
            ("material.material_system_id", "allowed.material_system_id"),
            ("material.phase_id", "allowed.phase_id"),
            ("material.species_id", "allowed.species_id"),
        ],
    );
    member(
        builder,
        "P5.domain_phase_species",
        pair,
        "material.domain_id",
        "material.member_id",
        "material.derivation_id",
    );
    let element = filtered(
        material,
        E::And(vec![
            null("phase_id"),
            null("species_id"),
            nonnull("element_id"),
        ]),
    );
    let element = join(
        element,
        scan("normalized.species_elements", "composition"),
        vec![("material.element_id", "composition.element_id")],
    );
    let element = join(
        element,
        scan("inferred.phase_species", "allowed"),
        vec![
            ("material.material_system_id", "allowed.material_system_id"),
            ("composition.species_id", "allowed.species_id"),
        ],
    );
    member(
        builder,
        "P5.domain_element",
        element,
        "material.domain_id",
        "material.member_id",
        "material.derivation_id",
    );
    pair_constraints(builder);
    let invalid = P::AntiJoin {
        left: Box::new(scan("normalized.candidate_index_members", "members")),
        right: Box::new(scan("inferred.domain_eligible_members", "eligible")),
        keys: (vec![
            ("members.domain_id", "eligible.domain_id"),
            ("members.member_id", "eligible.member_id"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    builder.declare_rule(
        RuleDecl::new(
            "P5.index_invalid_member",
            "1",
            6,
            RuleHead::Relation("inferred.invalid_index_tuples".to_owned()),
            project(
                invalid,
                vec![
                    ("product_id", E::col("members.product_id")),
                    ("tuple", E::col("members.tuple")),
                    ("derivation_id", E::col("members.derivation_id")),
                ],
            ),
        )
        .assertions("provenance.invalid_index_assertions")
        .stratified_negation(),
    );
    let valid = P::AntiJoin {
        left: Box::new(scan("normalized.candidate_index_tuples", "candidates")),
        right: Box::new(scan("inferred.invalid_index_tuples", "invalid")),
        keys: (vec![
            ("candidates.product_id", "invalid.product_id"),
            ("candidates.tuple", "invalid.tuple"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    builder.declare_rule(
        RuleDecl::new(
            "P5.index_valid",
            "1",
            7,
            RuleHead::Relation("inferred.valid_index_tuples".to_owned()),
            project(
                valid,
                vec![
                    ("product_id", E::col("candidates.product_id")),
                    ("tuple", E::col("candidates.tuple")),
                    ("derivation_id", E::col("candidates.derivation_id")),
                ],
            ),
        )
        .assertions("provenance.valid_index_assertions")
        .stratified_negation(),
    );
}
fn member(
    builder: &mut RegistryBuilder,
    name: &'static str,
    input: P,
    domain: &'static str,
    member: &'static str,
    derivation: &'static str,
) {
    builder.declare_rule(
        RuleDecl::new(
            name,
            "1",
            5,
            RuleHead::Relation("inferred.domain_eligible_members".to_owned()),
            P::Union(vec![project(
                input,
                vec![
                    ("domain_id", E::col(domain)),
                    ("member_id", E::col(member)),
                    ("derivation_id", E::col(derivation)),
                ],
            )]),
        )
        .assertions("provenance.domain_member_assertions")
        .stratified_negation(),
    );
}
fn null(name: &'static str) -> E {
    E::IsNull(Box::new(E::col(name)))
}
fn nonnull(name: &'static str) -> E {
    E::IsNotNull(Box::new(E::col(name)))
}
fn filtered(input: P, predicate: E) -> P {
    P::Filter {
        input: Box::new(input),
        predicate,
    }
}

fn pair_constraints(builder: &mut RegistryBuilder) {
    let phase = join(
        scan("normalized.candidate_index_members", "phase_members"),
        filtered(
            scan("normalized.material_domain_members", "phases"),
            E::And(vec![
                nonnull("phase_id"),
                null("species_id"),
                null("element_id"),
            ]),
        ),
        vec![
            ("phase_members.domain_id", "phases.domain_id"),
            ("phase_members.member_id", "phases.member_id"),
        ],
    );
    let phase = project(
        phase,
        vec![
            ("phase_product", E::col("phase_members.product_id")),
            ("phase_tuple", E::col("phase_members.tuple")),
            ("phase_material", E::col("phases.material_system_id")),
            ("phase_id", E::col("phases.phase_id")),
            ("pair_derivation", E::col("phase_members.derivation_id")),
        ],
    );
    let species = join(
        scan("normalized.candidate_index_members", "species_members"),
        filtered(
            scan("normalized.material_domain_members", "species"),
            E::And(vec![
                null("phase_id"),
                nonnull("species_id"),
                null("element_id"),
            ]),
        ),
        vec![
            ("species_members.domain_id", "species.domain_id"),
            ("species_members.member_id", "species.member_id"),
        ],
    );
    let species = project(
        species,
        vec![
            ("species_product", E::col("species_members.product_id")),
            ("species_tuple", E::col("species_members.tuple")),
            ("species_material", E::col("species.material_system_id")),
            ("species_id", E::col("species.species_id")),
        ],
    );
    let pairs = join(
        phase,
        species,
        vec![
            ("phase_product", "species_product"),
            ("phase_tuple", "species_tuple"),
            ("phase_material", "species_material"),
        ],
    );
    let rejected = P::AntiJoin {
        left: Box::new(pairs),
        right: Box::new(scan("inferred.phase_species", "allowed")),
        keys: (vec![
            ("phase_material", "allowed.material_system_id"),
            ("phase_id", "allowed.phase_id"),
            ("species_id", "allowed.species_id"),
        ])
        .into_iter()
        .map(|(left, right)| (left.into(), right.into()))
        .collect(),
    };
    builder.declare_rule(
        RuleDecl::new(
            "P5.index_invalid_phase_species_pair",
            "1",
            6,
            RuleHead::Relation("inferred.invalid_index_tuples".to_owned()),
            project(
                rejected,
                vec![
                    ("product_id", E::col("phase_product")),
                    ("tuple", E::col("phase_tuple")),
                    ("derivation_id", E::col("pair_derivation")),
                ],
            ),
        )
        .assertions("provenance.invalid_index_assertions")
        .stratified_negation(),
    );
}
