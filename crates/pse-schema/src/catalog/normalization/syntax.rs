// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete source syntax values; field admission owns alternative and operand arity.

use super::super::math_value::{alternatives, domain, record};
use crate::model::{CollectionContract, FieldContract as T};

fn ordinal() -> T {
    T::nonnegative(i64::MAX)
}

pub(super) fn predicate() -> T {
    let operand = alternatives(
        "PredicateOperandKind",
        vec![
            ("expression", record(vec![("node_id", ordinal())])),
            (
                "enum_literal",
                record(vec![
                    ("enum_id", T::id()),
                    ("member", T::native(arrow_schema::DataType::Utf8)),
                ]),
            ),
        ],
        &[],
    );
    alternatives(
        "PredicateKind",
        vec![
            (
                "boolean",
                record(vec![("value", T::native(arrow_schema::DataType::Boolean))]),
            ),
            ("atom", record(vec![("expression", ordinal())])),
            (
                "compare",
                record(vec![
                    ("comparison", T::enumeration("PredicateComparison")),
                    (
                        "operands",
                        T::list(operand.with_name("item")).with_collection(CollectionContract {
                            minimum: 2,
                            maximum: Some(2),
                            ..CollectionContract::SEQUENCE
                        }),
                    ),
                ]),
            ),
            (
                "in",
                record(vec![("expression", ordinal()), ("domain", domain())]),
            ),
            (
                "and",
                record(vec![("left", ordinal()), ("right", ordinal())]),
            ),
            (
                "or",
                record(vec![("left", ordinal()), ("right", ordinal())]),
            ),
            ("not", record(vec![("predicate", ordinal())])),
        ],
        &["null"],
    )
}

pub(super) fn equation() -> T {
    alternatives(
        "EquationSyntax",
        vec![
            (
                "relation",
                record(vec![
                    ("sense", T::enumeration("Sense")),
                    ("left", ordinal()),
                    ("right", ordinal()),
                ]),
            ),
            (
                "conditional",
                record(vec![
                    ("guard", ordinal()),
                    ("then", ordinal()),
                    ("otherwise", ordinal()),
                ]),
            ),
        ],
        &[],
    )
}
