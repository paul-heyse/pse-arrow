// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Borrowed eager-view mechanics with the same native adoption/error boundary.
use super::{AdapterError, AdapterFailure, BoundaryProjectionError, NativeFrom, refused};
use pse_contract::boundary_projection::ValidateProjection;

/// Refuse unknown binary meaning retained by the borrowed source view.
pub fn unknown_fields_view(fields: &buffa::UnknownFieldsView<'_>, path: &str) -> Result<(), AdapterError> {
    if fields.is_empty() { Ok(()) }
    else { Err(refused(path, AdapterFailure::UnknownField, "binary fields are retained by Buffa but this native owner cannot adopt their unknown meaning")) }
}

/// Validate the actual view, construct native data once, then validate native rules.
/// No owned protobuf message or dynamic representation is created.
pub fn decode_view_root<'view, N, W>(wire: &'view W, path: &str) -> Result<N, BoundaryProjectionError>
where N: NativeFrom<&'view W> + ValidateProjection, W: protovalidate_buffa::Validate + 'view {
    wire.validate()?;
    let native = N::from_wire(wire, path)?;
    native.validate_projection().map_err(AdapterError::from)?;
    Ok(native)
}

impl NativeFrom<&str> for String {
    fn from_wire(wire: &str, _: &str) -> Result<Self, AdapterError> { Ok(wire.to_owned()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message;
    #[test]
    fn borrowed_unknown_records_have_the_same_native_refusal() {
        // An unknown varint field100 in the actual pinned descriptor message.
        let bytes=[0xa0,0x06,0x01];
        let mut unknown=buffa::UnknownFieldsView::new();
        unknown.push_raw(&bytes);
        let owned=buffa_descriptor::generated::descriptor::FileDescriptorProto::decode_from_slice(&bytes).unwrap();
        for error in [unknown_fields_view(&unknown,"input.file").unwrap_err(),super::super::unknown_fields(&owned.__buffa_unknown_fields,"input.file").unwrap_err()] {
            assert!(matches!(error,AdapterError::Refused{path,kind:AdapterFailure::UnknownField,..} if path=="input.file"));
        }
    }
    #[test]
    fn protobuf_map_views_match_owned_last_write_wins_but_custom_entries_refuse_duplicates() {
        let view=buffa::MapView::new(vec![("first",1_u64),("first",9),("second",3)]);
        let owned=view.iter().map(|(key,value)|((*key).to_owned(),*value)).collect::<std::collections::BTreeMap<_,_>>();
        let borrowed=super::super::unique_pairs(view.iter_unique().map(|(key,value)|((*key).to_owned(),*value)).collect(),"map").unwrap();
        assert_eq!(borrowed,owned);
        assert_eq!(borrowed["first"],9);
        assert_eq!(view.len(),3,"original view still retains overwritten entries for raw extent accounting");
        assert!(matches!(super::super::unique_pairs(view.iter().map(|(key,value)|((*key).to_owned(),*value)).collect(),"custom_entries"),Err(AdapterError::Refused{kind:AdapterFailure::DuplicateKey,..})));
    }
}
