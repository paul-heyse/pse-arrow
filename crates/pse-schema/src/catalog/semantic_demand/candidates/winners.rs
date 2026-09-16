// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Set difference preserves every greatest-rank selection before distinct-method counting.
use super::{
    Cell, CmpOp, E, P, RegistryBuilder, anti, eq, filter, join, literal, present, project, rule,
    scan,
};
use crate::model::{AggregateEmptyPolicy, AggregateNullPolicy, RuleAggregate, RuleAggregateFn};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    // An absent rank cannot participate in the ordered candidate relation. Keep
    // this membership condition inside the winner join, where it also establishes
    // the rank field used by every resulting winner assertion.
    let current = filter(
        filter(
            scan("inferred.potential_method_candidates", "current"),
            E::col("applicable"),
        ),
        present("current.rank"),
    );
    let other = filter(
        scan("inferred.potential_method_candidates", "other"),
        E::col("applicable"),
    );
    let shadowed = filter(
        join(
            current.clone(),
            other,
            vec![("current.requirement_id", "other.requirement_id")],
        ),
        E::cmp(CmpOp::Gt, E::col("other.rank"), E::col("current.rank")),
    );
    let shadowed = project(
        shadowed,
        vec![
            ("shadowed_requirement", E::col("current.requirement_id")),
            ("shadowed_selection", E::col("current.selection_id")),
            ("shadowed_method", E::col("current.method_id")),
        ],
    );
    let winner = anti(
        current,
        shadowed,
        vec![
            ("current.requirement_id", "shadowed_requirement"),
            ("current.selection_id", "shadowed_selection"),
            ("current.method_id", "shadowed_method"),
        ],
    );
    rule(
        builder,
        "P6.greatest_rank",
        3,
        "inferred.potential_method_winners",
        "provenance.potential_method_winner_assertions",
        project(
            winner,
            vec![
                ("requirement_id", E::col("current.requirement_id")),
                ("selection_id", E::col("current.selection_id")),
                ("method_id", E::col("current.method_id")),
                ("rank", E::col("current.rank")),
                ("derivation_id", E::col("current.derivation_id")),
            ],
        ),
        true,
    );
    let distinct = P::Distinct(Box::new(project(
        scan("inferred.potential_method_winners", "winners"),
        vec![
            ("count_requirement", E::col("requirement_id")),
            ("count_method", E::col("method_id")),
        ],
    )));
    let counts = P::Aggregate {
        input: Box::new(distinct),
        group: (vec!["count_requirement"])
            .into_iter()
            .map(Into::into)
            .collect(),
        aggregates: vec![RuleAggregate {
            function: RuleAggregateFn::Count,
            input: None,
            output_name: ("method_count").into(),
            order_by: vec![],
            null_policy: AggregateNullPolicy::Reject,
            empty_policy: AggregateEmptyPolicy::Zero,
        }],
    };
    let single = filter(
        counts.clone(),
        eq(E::col("method_count"), E::Lit(Cell::U64(1))),
    );
    let single = join(
        single,
        scan("inferred.potential_method_winners", "winners"),
        vec![("count_requirement", "winners.requirement_id")],
    );
    let single = join(
        single,
        scan("reference.method_specs", "methods"),
        vec![("winners.method_id", "methods.method_id")],
    );
    rule(
        builder,
        "P6.unique_method",
        4,
        "inferred.potential_method_resolutions",
        "provenance.potential_method_resolution_assertions",
        project(
            single,
            vec![
                ("requirement_id", E::col("winners.requirement_id")),
                ("method_id", E::col("methods.method_id")),
                ("realization", E::col("methods.realization")),
                ("template_id", E::col("methods.template_id")),
                ("status", literal("resolved")),
                ("derivation_id", E::col("winners.requirement_id")),
            ],
        ),
        false,
    );
    let ambiguous = filter(
        counts,
        E::cmp(CmpOp::Gt, E::col("method_count"), E::Lit(Cell::U64(1))),
    );
    unresolved(
        builder,
        "P6.ambiguous_method",
        ambiguous,
        "count_requirement",
        "ambiguous",
    );
    let absent = anti(
        scan("inferred.requirement_universe", "requirements"),
        scan("inferred.potential_method_winners", "winners"),
        vec![("requirements.requirement_id", "winners.requirement_id")],
    );
    unresolved(
        builder,
        "P6.unsupported_method",
        absent,
        "requirements.requirement_id",
        "unresolved",
    );
}
fn unresolved(
    builder: &mut RegistryBuilder,
    name: &'static str,
    input: P,
    key: &'static str,
    status: &'static str,
) {
    rule(
        builder,
        name,
        4,
        "inferred.potential_method_resolutions",
        "provenance.potential_method_resolution_assertions",
        project(
            input,
            vec![
                ("requirement_id", E::col(key)),
                ("method_id", E::Lit(Cell::Null)),
                ("realization", E::Lit(Cell::Null)),
                ("template_id", E::Lit(Cell::Null)),
                ("status", literal(status)),
                ("derivation_id", E::col(key)),
            ],
        ),
        true,
    );
}
