// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_contract::boundary_projection::{
    AdapterError, ProjectionError, ProjectionObservationRecord, ProjectionType,
};

struct NativeObservation {
    recorded: u64,
}

impl Drop for NativeObservation {
    fn drop(&mut self) {}
}

impl ProjectionType for NativeObservation {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let root = graph.declare::<Self>("NativeObservation", file!(), 1,
            Direction::Output, &[], &[], |graph| Ok(Shape::Record(vec![
                Field::of::<u64>(graph, "recorded", 1, Direction::Output, vec![])?,
            ])))?;
        graph.observation_record::<Self>(&root)?;
        Ok(root)
    }
}

impl ProjectionObservationRecord for NativeObservation {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "recorded")];
    type Fields = (u64,);
    fn observation_fields(&self) -> Result<Self::Fields, AdapterError> {
        Ok((self.recorded,))
    }
}

impl pse_contract::boundary_projection::ValidateProjection for NativeObservation {
    fn validate_at(&self, _: &str) -> Result<(), pse_contract::boundary_projection::ValidationError> {
        Ok(())
    }
}

#[test]
fn observation_owners_are_borrowed_and_never_have_inverse_native_adapters() {
    let native = NativeObservation { recorded: 42 };
    assert_eq!(native.observation_fields().unwrap(), (42,));
    assert_eq!(native.recorded, 42, "observation does not consume Drop custody");
    let graph = ProjectionGraph::of::<NativeObservation>().unwrap();
    let descriptors = descriptors_with_imports(&graph, "pse.observation.v1", "observation.proto", &imports()).unwrap();
    let file = descriptors.file.iter().find(|file| file.name.as_deref() == Some("observation.proto")).unwrap();
    assert!(file.source_code_info.as_option().is_none(), "owner provenance does not invent a Rust source span");
    let encoder = metadata::OptionEncoder::new(&descriptors.file).unwrap();
    let options = encoder.decode_extension(file.message_type[0].options.as_option().unwrap(), "pse.contract.v1.message").unwrap().unwrap();
    assert_eq!(options.field_by_number(4), Some(&buffa_descriptor::Value::EnumNumber(2)), "output direction is independently published");
    assert_eq!(options.field_by_number(5), Some(&buffa_descriptor::Value::EnumNumber(2)), "the native observation role is published explicitly");
    for source in [
        native_adapters(&graph, &["NativeObservation"], "crate::wire", "pse_codegen").unwrap(),
        native_adapters_retaining_wire(&graph, &["NativeObservation"], "crate::wire", "pse_codegen").unwrap(),
    ] {
        let parsed = syn::parse_file(&source).unwrap();
        assert!(source.contains("ProjectionObservationRecord"));
        assert!(!source.contains("NativeFrom"));
        assert!(!source.contains("from_projection_fields"));
        assert!(!source.contains("decode_native_observation"));
        let encode = parsed.items.iter().find_map(|item| match item {
            syn::Item::Fn(function) if function.sig.ident == "encode_native_observation" => Some(function),
            _ => None,
        }).unwrap();
        assert!(matches!(encode.sig.inputs.first(), Some(syn::FnArg::Typed(argument))
            if matches!(argument.ty.as_ref(), syn::Type::Reference(_))));
    }
}

#[test]
fn reconstructible_parent_cannot_hide_observation_only_native_child() {
    projection_struct! {
        struct NativeParent {
            projection: { name: "NativeParent", version: 1, direction: Direction::Both,
                reserved_tags: [], reserved_names: [] };
            child: NativeObservation => { tag: 1, direction: Direction::Output, rules: [] };
        }
    }
    let graph = ProjectionGraph::of::<NativeParent>().unwrap();
    let error = native_adapters(&graph, &["NativeParent"], "crate::wire", "pse_codegen").unwrap_err();
    assert!(error.to_string().contains("observation-only native owner NativeObservation"));
}
