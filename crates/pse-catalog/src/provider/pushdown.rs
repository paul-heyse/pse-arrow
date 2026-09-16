// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The exact predicate subset; evaluation remains DataFusion's responsibility.

use datafusion::logical_expr::{Expr, Operator};

use crate::RelationContract;

/// Whether a predicate uses only the exercised key/enum membership vocabulary.
pub fn supported(expr: &Expr, contract: &RelationContract) -> bool {
    match expr {
        Expr::BinaryExpr(binary) => match binary.op {
            Operator::And | Operator::Or => {
                supported(&binary.left, contract) && supported(&binary.right, contract)
            }
            Operator::Eq => {
                (column(&binary.left, contract) && literal(&binary.right))
                    || (literal(&binary.left) && column(&binary.right, contract))
            }
            _ => false,
        },
        Expr::InList(list) => column(&list.expr, contract) && list.list.iter().all(literal),
        Expr::IsNull(expr) | Expr::IsNotNull(expr) => column(expr, contract),
        _ => false,
    }
}
fn literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(..))
}
fn column(expr: &Expr, contract: &RelationContract) -> bool {
    let Expr::Column(column) = expr else {
        return false;
    };
    contract
        .canonical
        .schema
        .index_of(column.name())
        .is_ok_and(|index| {
            let field = contract.canonical.schema.field(index);
            let reference = field
                .metadata()
                .get(pse_schema::arrow::KEY_ROLE)
                .is_some_and(|role| role == "reference")
                && field
                    .metadata()
                    .get(pse_schema::arrow::KEY_LOGICAL_TYPE)
                    .is_some_and(|kind| {
                        matches!(kind.as_str(), "semantic_id" | "u64" | "i64" | "text")
                    });
            contract.canonical.primary_key.contains(&index)
                || contract.enum_columns.contains(&index)
                || reference
        })
}
