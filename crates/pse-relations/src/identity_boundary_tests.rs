// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The actual native owner, generated alias and Arrow codec share one physical domain.
#![allow(clippy::unwrap_used, reason = "explicit Arrow boundary contract assertions")]
use crate::columnar::ArrowValue;

#[test]
fn source_owned_physical_alias_roundtrips_exact_arrow_bytes_and_nulls() {
    let owner = pse_quantity::UnitId::from_bytes([0xa7; 16]);
    let projected: pse_model::generated::identities::UnitId = owner;
    let mut output = arrow_array::builder::make_builder(&arrow_schema::DataType::FixedSizeBinary(16), 2);
    projected.append(output.as_mut()).unwrap();
    pse_model::generated::identities::UnitId::append_null(output.as_mut()).unwrap();
    let array = output.finish();
    assert_eq!(pse_quantity::UnitId::read(array.as_ref(), 0).unwrap(), owner);
    assert_eq!(pse_ids::SemanticId::read(array.as_ref(), 0).unwrap().as_bytes(), &[0xa7; 16]);
    assert!(pse_quantity::UnitId::read(array.as_ref(), 1).is_err());
    assert!(pse_quantity::UnitId::read(array.as_ref(), 2).is_err());
}
