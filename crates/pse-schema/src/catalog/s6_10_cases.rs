// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Measurement datasets and observations consumed by authored fitting.
use super::declarations::{column,relation};
use crate::{RegistryBuilder,model::{FieldContract as T,Namespace as N,SnapshotClass as S}};

/// Declare measurement inputs; model cases live in the modeling IR.
pub fn declare(builder:&mut RegistryBuilder){
 declare_authored_datasets(builder);
 declare_authored_observations(builder);
}

fn declare_authored_datasets(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "datasets",
        S::Case,
        &["dataset_id"],
        vec![
            column("dataset_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("source", T::native(arrow_schema::DataType::Utf8)),
            column("content_hash", T::hash()),
        ],
        "blueprint §6.10 case: datasets.",
    );
}

fn declare_authored_observations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "observations",
        S::Case,
        &["observation_id"],
        vec![
            column("observation_id", T::id()),
            column("dataset_id", T::id()),
            column(
                "target",
                T::extended(crate::model::ExtensionUse::TargetPath),
            ),
            column("value", T::native(arrow_schema::DataType::Float64)).optional(),
            column("unit_id", T::id()),
            column("std_dev", T::native(arrow_schema::DataType::Float64)).optional(),
            column(
                "timestamp",
                T::native(crate::model::extension::timestamp_storage()),
            )
            .optional(),
            column("tag", T::native(arrow_schema::DataType::Utf8)).optional(),
            column(
                "source_span",
                T::extended(crate::model::ExtensionUse::SourceSpan),
            ),
        ],
        "blueprint §6.10 case: observations.",
    );
}
