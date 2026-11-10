// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Frozen analytical and exact/assurance observations under ordinary result custody.
use super::declarations::{column, documented, relation, run_id};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, ResultSelectionRole as R, SnapshotClass as S},
};
use arrow_schema::DataType as D;
pub(super) fn declare(builder: &mut RegistryBuilder) {
    let text = || T::native(D::Utf8);
    let native_view = T::structure(vec![
        column("path", text()),
        column("symbolica", text()),
        column("latex", text()),
        column("typst", text()),
    ])
    .named("MathematicalNativeView");
    relation(
        builder,
        N::Runtime,
        "mathematical_products",
        S::Derived,
        &["run_id", "ordinal"],
        vec![
            run_id().with_result_selection(R::Run).with_result_sort(0),
            column("ordinal", T::nonnegative(i64::MAX)).with_result_sort(1),
            column("source_id", T::id()).with_result_selection(R::Owner),
            column("source_revision", T::hash()),
            column("product_kind", text()),
            column("operation", text()),
            column("claim", text()),
            documented(
                "supported",
                T::native(D::Boolean),
                "The requested mathematical product was established under its original premises. This is independent of physical feasibility, empirical applicability or permission to reuse the result.",
            ),
            column("support_reason", text()).optional(),
            documented(
                "source_context",
                text(),
                "Exact producer-encoded original physical/source/guard observation. Native view references point into this same row's native_views; this is never an executable symbolic codec.",
            ),
            documented(
                "result_context",
                text(),
                "Frozen operation-specific mathematical observation, including actual order, precision, conditions and stopping meaning. Readers never rerun the producer.",
            ),
            column("native_views", T::list(native_view)),
        ],
        "Original selected mathematical products, persisted by the producing run and terminal attempt under sealed ordinary result manifests and physical closure. Formula strings are native presentation, not identity, execution input or proof. Partial reader pages do not change the original operation's scope or support.",
    );
}
