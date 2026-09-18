// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable function declarations, shared as native Arrow fields. These four
//! shapes depend only on the static registry and codec kind, never on input rows.
use super::{Codec, DataType, FieldContract, FieldRef, Kind};
use datafusion::{
    common::{DataFusionError, Result},
    logical_expr::ScalarUDFImpl,
};
use std::sync::{Arc, LazyLock};

static OUTPUTS: LazyLock<std::result::Result<[FieldRef; 4], Arc<pse_schema::SchemaError>>> =
    LazyLock::new(|| {
        Ok([
            declare(Kind::Literal, false).map_err(Arc::new)?,
            declare(Kind::IdList, false).map_err(Arc::new)?,
            declare(Kind::NamedId, false).map_err(Arc::new)?,
            declare(Kind::NamedId, true).map_err(Arc::new)?,
        ])
    });

pub(super) fn output(kind: Kind, nullable: bool) -> Result<FieldRef> {
    let fields = OUTPUTS
        .as_ref()
        .map_err(|error| DataFusionError::External(Box::new(Arc::clone(error))))?;
    let index = match kind {
        Kind::Literal => 0,
        Kind::IdList => 1,
        Kind::NamedId => 2 + usize::from(nullable),
    };
    Ok(Arc::clone(&fields[index]))
}

fn declare(kind: Kind, nullable: bool) -> std::result::Result<FieldRef, pse_schema::SchemaError> {
    let logical = match kind {
        Kind::NamedId => FieldContract::id(),
        Kind::IdList => FieldContract::list(FieldContract::id()),
        Kind::Literal => FieldContract::native(DataType::Utf8),
    };
    let column = FieldContract::payload("value", logical, "Exact native diagnostic codec output.")
        .with_nullable(nullable);
    Ok(Arc::new(
        pse_schema::arrow::field_for(pse_schema::registry()?, &column)?
            .with_name(Codec::new(kind).name()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{arrow::datatypes::Field, logical_expr::ReturnFieldArgs};

    #[test]
    fn native_codec_fields_share_exact_declarations_but_still_check_arguments() {
        for (kind, nullable) in [
            (Kind::Literal, false),
            (Kind::IdList, false),
            (Kind::NamedId, false),
            (Kind::NamedId, true),
        ] {
            let first = output(kind, nullable).unwrap();
            assert_eq!(first, declare(kind, nullable).unwrap());
            assert!(Arc::ptr_eq(&first, &output(kind, nullable).unwrap()));
            assert_eq!(first.is_nullable(), nullable);
        }
        let named = Codec::new(Kind::NamedId);
        for nullable in [false, true] {
            let args = [
                Arc::new(Field::new(
                    "namespace",
                    DataType::FixedSizeBinary(16),
                    false,
                )),
                Arc::new(Field::new("payload", DataType::Utf8, nullable)),
            ];
            let field = named
                .return_field_from_args(ReturnFieldArgs {
                    arg_fields: &args,
                    scalar_arguments: &[None, None],
                })
                .unwrap();
            assert!(Arc::ptr_eq(
                &field,
                &output(Kind::NamedId, nullable).unwrap()
            ));
        }
        let invalid = [Arc::new(Field::new("foreign", DataType::Int64, false))];
        for kind in [Kind::NamedId, Kind::IdList] {
            assert!(
                Codec::new(kind)
                    .return_field_from_args(ReturnFieldArgs {
                        arg_fields: &invalid,
                        scalar_arguments: &[None],
                    })
                    .is_err()
            );
        }
    }
}
