//! Preserve schema-defined oneof vocabulary in owned and borrowed Rust output.
use buffa_codegen::{CodeGenConfig, generate, generated::descriptor::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, OneofDescriptorProto, field_descriptor_proto::{Label, Type}}};

fn make_field(name: &str, number: i32, label: Label, ty: Type) -> FieldDescriptorProto {
    FieldDescriptorProto { name: Some(name.into()), number: Some(number), label: Some(label), r#type: Some(ty), ..Default::default() }
}

#[test]
fn schema_oneof_names_keep_owned_and_view_vocabulary_with_a_local_style_exception() {
    use quote::ToTokens as _;

    let mut file = FileDescriptorProto { name: Some("schema_names.proto".into()), syntax: Some("proto3".into()), ..Default::default() };
    file.package = Some("names.v1".into());
    for (message, first) in [("DomainValue", "entity_kind"), ("PlainValue", "text")] {
        let mut field = make_field(first, 17, Label::LABEL_OPTIONAL, Type::TYPE_STRING);
        field.oneof_index = Some(0);
        let mut other = make_field("other", 88, Label::LABEL_OPTIONAL, Type::TYPE_STRING);
        other.oneof_index = Some(0);
        file.message_type.push(DescriptorProto {
            name: Some(message.into()),
            field: vec![field, other],
            oneof_decl: vec![OneofDescriptorProto {
                name: Some("kind".into()),
                ..Default::default()
            }],
            ..Default::default()
        });
    }
    let files = generate(
        &[file],
        &["schema_names.proto".into()],
        &CodeGenConfig::default(),
    )
    .unwrap();
    fn inspect(items: &[syn::Item], observed: &mut Vec<(Vec<String>, bool)>) {
        for item in items {
            match item {
                syn::Item::Enum(item) if item.ident == "Kind" => {
                    let variants = item
                        .variants
                        .iter()
                        .map(|variant| variant.ident.to_string())
                        .collect();
                    let exception = item.attrs.iter().any(|attribute| {
                        attribute.path().is_ident("allow")
                            && attribute
                                .meta
                                .to_token_stream()
                                .to_string()
                                .contains("enum_variant_names")
                    });
                    observed.push((variants, exception));
                }
                syn::Item::Mod(item) => {
                    if let Some((_, items)) = &item.content {
                        inspect(items, observed);
                    }
                }
                _ => {}
            }
        }
    }
    let mut observed = Vec::new();
    for file in files {
        let parsed = syn::parse_file(&file.content).unwrap();
        inspect(&parsed.items, &mut observed);
    }
    assert_eq!(observed.len(), 4, "owned and view enums for both schemas");
    assert_eq!(
        observed
            .iter()
            .filter(|(names, exception)| {
                names == &["EntityKind", "Other"] && *exception
            })
            .count(),
        2
    );
    assert_eq!(
        observed
            .iter()
            .filter(|(names, exception)| names == &["Text", "Other"] && !*exception)
            .count(),
        2
    );
}
