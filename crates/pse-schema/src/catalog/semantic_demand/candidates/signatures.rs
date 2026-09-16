// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source declarations constrain every physical axis of an advertised method output.
use super::{E, P, anti, filter, join, project, same, scan};

pub(super) fn expected(input: P) -> P {
    let actual = join(
        input.clone(),
        scan("inferred.demand_seed_bindings", "bound_reads"),
        vec![("requirements.requirement_id", "bound_reads.requirement_id")],
    );
    let actual = join(
        actual,
        scan("inferred.demand_active_reads", "read_declarations"),
        vec![("bound_reads.read_id", "read_declarations.read_id")],
    );
    let actual = join(
        actual,
        scan("normalized.template_symbols", "read_symbols"),
        vec![(
            "read_declarations.symbol_decl_id",
            "read_symbols.symbol_decl_id",
        )],
    );
    let actual = join(
        actual,
        scan("reference.quantity_types", "expected_types"),
        vec![(
            "read_symbols.quantity_type_id",
            "expected_types.quantity_type_id",
        )],
    );
    let equal = E::And(vec![
        same(
            "quantities.quantity_kind_id",
            "expected_types.quantity_kind_id",
        ),
        same("quantities.basis_id", "expected_types.basis_id"),
        same(
            "quantities.reference_state_id",
            "expected_types.reference_state_id",
        ),
        same("quantities.scale_kind", "expected_types.scale_kind"),
        same("quantities.subject_kind", "expected_types.subject_kind"),
        same("quantities.shape", "expected_types.shape"),
    ]);
    let wrong = filter(actual, E::Not(Box::new(equal)));
    let wrong = project(
        wrong,
        vec![
            (
                "mismatch_requirement",
                E::col("requirements.requirement_id"),
            ),
            ("mismatch_selection", E::col("selections.selection_id")),
            ("mismatch_method", E::col("selections.method_id")),
        ],
    );
    anti(
        input,
        wrong,
        vec![
            ("requirements.requirement_id", "mismatch_requirement"),
            ("selections.selection_id", "mismatch_selection"),
            ("selections.method_id", "mismatch_method"),
        ],
    )
}
