// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Concrete independently chosen examples of the declared conditional shapes.
use super::{Rows, id, set};
use pse_schema::model::{Cell, InvariantSpec};

pub(super) fn populate(invariant: &InvariantSpec, valid: &mut Rows, invalid: &mut Rows) -> bool {
    let relation = invariant.relation.as_str();
    let (values, broken): (Vec<(&str, Cell)>, (&str, Cell)) = match invariant.name.as_str() {
        "owner_alternative" => (
            vec![
                ("owner_kind", Cell::Enum("template")),
                ("owner_template_id", id(1)),
                ("owner_instance_id", Cell::Null),
            ],
            ("owner_instance_id", id(1)),
        ),
        "source_alternative" => (
            vec![
                ("source", Cell::Enum("domain")),
                ("domain_id", id(1)),
                ("parameter_name", Cell::Null),
            ],
            ("parameter_name", Cell::text("extra")),
        ),
        "winner_alternative" => (
            vec![
                ("status", Cell::Enum("resolved")),
                ("method_id", id(1)),
                ("realization", Cell::Enum("equation_template")),
                ("template_id", id(1)),
            ],
            ("method_id", Cell::Null),
        ),
        "realization_alternative" => (
            vec![
                ("realization", Cell::Enum("equation_template")),
                ("template_id", id(1)),
                ("kernel_id", Cell::Null),
            ],
            ("kernel_id", id(1)),
        ),
        "output_alternative" if relation == "reference.method_provisions" => (
            vec![
                ("output_kind", Cell::Enum("template_symbol")),
                ("symbol_decl_id", id(1)),
                ("kernel_output_ordinal", Cell::Null),
            ],
            ("symbol_decl_id", Cell::Null),
        ),
        "output_alternative" => (
            vec![
                ("output_kind", Cell::Enum("template_symbol")),
                ("template_instance_id", id(1)),
                ("output_symbol_id", id(1)),
                ("kernel_binding_id", Cell::Null),
                ("kernel_output_ordinal", Cell::Null),
            ],
            ("output_symbol_id", Cell::Null),
        ),
        "subject_alternative" => (
            vec![
                ("subject_kind", Cell::Enum("total")),
                ("subject_id", Cell::Null),
                ("subject_axis", Cell::Null),
                ("phase_id", Cell::Null),
                ("phase_axis", Cell::Null),
            ],
            ("subject_id", id(1)),
        ),
        "decision_alternative" => (
            vec![
                ("decision", Cell::Enum("included")),
                ("reason", Cell::Enum("matched")),
                ("sign", Cell::I64(1)),
            ],
            ("sign", Cell::I64(0)),
        ),
        _ => return false,
    };
    for (column, value) in values {
        set(valid, relation, column, value);
    }
    *invalid = valid.clone();
    set(invalid, relation, broken.0, broken.1);
    true
}
