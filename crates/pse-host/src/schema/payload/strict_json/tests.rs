// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use buffa_descriptor::{
    ReflectMessage,
    generated::descriptor::{
        DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
        MessageOptions,
        field_descriptor_proto::{Label, Type},
    },
};

fn fixture() -> DescriptorPool {
    let field =
        |name: &str, number, kind: Type, repeated, type_name: Option<&str>| FieldDescriptorProto {
            name: Some(name.into()),
            number: Some(number),
            r#type: Some(kind),
            label: Some(if repeated {
                Label::Repeated
            } else {
                Label::Optional
            }),
            type_name: type_name.map(str::to_owned),
            ..Default::default()
        };
    DescriptorPool::new(FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("strict-json.proto".into()),
            package: Some("strict".into()),
            syntax: Some("proto3".into()),
            message_type: vec![
                DescriptorProto {
                    name: Some("Child".into()),
                    field: vec![FieldDescriptorProto {
                        json_name: Some("guardToken".into()),
                        ..field("source_guard", 1, Type::String, false, None)
                    }],
                    ..Default::default()
                },
                DescriptorProto {
                    name: Some("Root".into()),
                    field: vec![
                        field("child", 1, Type::Message, false, Some(".strict.Child")),
                        field("children", 2, Type::Message, true, Some(".strict.Child")),
                        field(
                            "by_name",
                            3,
                            Type::Message,
                            true,
                            Some(".strict.Root.ByNameEntry"),
                        ),
                        field("labels", 4, Type::String, true, None),
                    ],
                    nested_type: vec![DescriptorProto {
                        name: Some("ByNameEntry".into()),
                        options: MessageOptions {
                            map_entry: Some(true),
                            ..Default::default()
                        }
                        .into(),
                        field: vec![
                            field("key", 1, Type::String, false, None),
                            field("value", 2, Type::Message, false, Some(".strict.Child")),
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    })
    .unwrap()
}

fn compiled(pool: &DescriptorPool, bytes: &[u8]) -> Result<(), SchemaError> {
    let message = pool.message(pool.message_index("strict.Root").unwrap());
    check(bytes, Some((pool, message)))
}

#[test]
fn schema_payload_strict_json_checks_message_list_and_map_names_and_aliases() {
    let pool = fixture();
    for bytes in [
        br#"{"child":{"source_guard":"a"},"children":[{"guardToken":"b"}],"byName":{"arbitrary-key":{"guardToken":"c"}}}"#.as_slice(),
        br#"{"child":null,"children":null,"by_name":null}"#,
        br#"{"children":[null,{}],"by_name":{"source_guard":{},"guardToken":{}}}"#,
    ] {
        compiled(&pool, bytes).unwrap();
    }
    for (bytes, expected) in [
        (
            br#"{"unknown":1}"#.as_slice(),
            "unknown JSON field strict.Root.unknown",
        ),
        (
            br#"{"child":{"unknown":1}}"#,
            "unknown JSON field strict.Child.unknown",
        ),
        (
            br#"{"children":[{}, {"unknown":1}]}"#,
            "unknown JSON field strict.Child.unknown",
        ),
        (
            br#"{"by_name":{"arbitrary-key":{"unknown":1}}}"#,
            "unknown JSON field strict.Child.unknown",
        ),
        (
            br#"{"child":{"source_guard":"a","guardToken":"b"}}"#,
            "duplicate JSON field aliases: strict.Child.guardToken",
        ),
        (
            br#"{"children":[{"source_guard":null,"guardToken":null}]}"#,
            "duplicate JSON field aliases: strict.Child.guardToken",
        ),
        (
            br#"{"by_name":{"x":{"source_guard":"a","guardToken":"b"}}}"#,
            "duplicate JSON field aliases: strict.Child.guardToken",
        ),
        (
            br#"{"by_name":{},"byName":{}}"#,
            "duplicate JSON field aliases: strict.Root.byName",
        ),
        // The descriptor's explicit JSON spelling is authoritative.
        (
            br#"{"child":{"sourceGuard":"a"}}"#,
            "unknown JSON field strict.Child.sourceGuard",
        ),
    ] {
        let SchemaError::Decode(reason) = compiled(&pool, bytes).unwrap_err() else {
            panic!(
                "wrong error category for {}",
                String::from_utf8_lossy(bytes)
            );
        };
        assert_eq!(reason, expected);
    }
}

#[test]
fn schema_payload_strict_json_literal_duplicates_precede_descriptor_refusals() {
    let pool = fixture();
    for bytes in [
        br#"{"child":{"guardToken":"a","guardToken":"b"}}"#.as_slice(),
        br#"{"by_name":{"x":{},"x":{}}}"#,
        br#"{"labels":[{"x":1,"x":2}]}"#,
        br#"{"unknown":{"x":1,"x":2}}"#,
        br#"{"child":{"source_guard":"a","guardToken":"b"},"unknown":{"x":1,"x":2}}"#,
        br#"{"child":{"guardToken":"a","guard\u0054oken":"b"}}"#,
    ] {
        let error = compiled(&pool, bytes).unwrap_err();
        assert!(matches!(&error, SchemaError::Json(_)), "{error:?}");
        assert!(
            error.to_string().contains("duplicate JSON member:"),
            "{error:?}"
        );
    }
    for bytes in [
        br#"{"unknown":1} {}"#.as_slice(),
        br#"{"unknown":1,"child":"#,
    ] {
        assert!(matches!(compiled(&pool, bytes), Err(SchemaError::Json(_))));
    }
}

#[test]
fn schema_payload_strict_json_leaves_value_shapes_to_the_typed_decoder() {
    let pool = fixture();
    // As before, preflight owns keys only when the value has its expected
    // collection shape; generated decoding remains the value-grammar authority.
    for bytes in [
        b"null".as_slice(),
        br#"{"child":[],"children":{"arbitrary":1},"by_name":[{"arbitrary":1}]}"#,
        br#"{"labels":[{"arbitrary":1}]}"#,
        br#"[{"arbitrary":1}]"#,
    ] {
        compiled(&pool, bytes).unwrap();
    }
    let mut bytes = vec![b'['; 150];
    bytes.extend(vec![b']'; 150]);
    let SchemaError::Json(error) = check(&bytes, None).unwrap_err() else {
        panic!()
    };
    assert!(
        error.to_string().contains("recursion limit exceeded"),
        "{error}"
    );
}

#[test]
fn schema_payload_strict_json_preserves_wkt_owned_arbitrary_json() {
    let cases: [(&dyn ReflectMessage, &[u8]); 3] = [
        (
            &buffa_types::Struct::default(),
            br#"{"arbitrary":{"source_guard":1,"guardToken":2},"other":[null,{}]}"#,
        ),
        (
            &buffa_types::Value::default(),
            br#"{"arbitrary":[true,"text",null,42]}"#,
        ),
        (
            &buffa_types::ListValue::default(),
            br#"[{"arbitrary":[{"another":1}]}]"#,
        ),
    ];
    for (message, bytes) in cases {
        check(bytes, Some((message.pool(), message.message_descriptor()))).unwrap();
        assert!(matches!(
            check(
                br#"{"arbitrary":{"x":1,"x":2}}"#,
                Some((message.pool(), message.message_descriptor()))
            ),
            Err(SchemaError::Json(_))
        ));
    }
    let bytes = br#"{"arbitrary":{"source_guard":1,"guardToken":2},"other":[null,{}]}"#;
    let value: buffa_types::Struct = serde_json::from_slice(bytes).unwrap();
    assert_eq!(value.fields.len(), 2);
    let _: buffa_types::Value =
        serde_json::from_slice(br#"{"arbitrary":[true,"text",null,42]}"#).unwrap();
    let _: buffa_types::ListValue =
        serde_json::from_slice(br#"[{"arbitrary":[{"another":1}]}]"#).unwrap();
}

#[test]
fn schema_payload_strict_json_keeps_imported_any_policy_explicit() {
    let any = buffa_types::Any::default();
    let world = Some((any.pool().as_ref(), any.message_descriptor()));
    check(
        br#"{"typeUrl":"type.googleapis.com/strict.Child","value":"AA=="}"#,
        world,
    )
    .unwrap();
    // The compiled preflight retains the existing descriptor-key policy for
    // imported Any; it does not invent an @type resolver or a new external policy.
    assert!(matches!(
        check(br#"{"@type":"type.googleapis.com/strict.Child"}"#, world),
        Err(SchemaError::Decode(_))
    ));
    assert!(matches!(
        check(br#"{"type_url":"a","typeUrl":"b"}"#, world),
        Err(SchemaError::Decode(_))
    ));
    // Acquired dynamic worlds still receive a duplicate-only token pass before
    // DynamicMessage owns descriptor-qualified Any conversion.
    check(
        br#"{"@type":"type.googleapis.com/strict.Child","guardToken":"a"}"#,
        None,
    )
    .unwrap();
    assert!(matches!(
        check(br#"{"@type":"a","@type":"b"}"#, None),
        Err(SchemaError::Json(_))
    ));
}

#[test]
fn schema_payload_strict_json_keeps_imported_scalar_wkt_readers() {
    let cases: [(&dyn ReflectMessage, &[u8]); 3] = [
        (
            &buffa_types::Timestamp::default(),
            br#""2026-11-06T12:34:56Z""#,
        ),
        (&buffa_types::Duration::default(), br#""1.250s""#),
        (
            &buffa_types::FieldMask::default(),
            br#""someField,nested.value""#,
        ),
    ];
    for (message, bytes) in cases {
        let world = Some((message.pool().as_ref(), message.message_descriptor()));
        check(bytes, world).unwrap();
        check(b"null", world).unwrap();
        assert!(matches!(
            check(br#"{"unknown":1}"#, world),
            Err(SchemaError::Decode(_))
        ));
    }
    let _: buffa_types::Timestamp = serde_json::from_slice(br#""2026-11-06T12:34:56Z""#).unwrap();
    let _: buffa_types::Duration = serde_json::from_slice(br#""1.250s""#).unwrap();
    let mask: buffa_types::FieldMask =
        serde_json::from_slice(br#""someField,nested.value""#).unwrap();
    assert_eq!(mask.paths, ["some_field", "nested.value"]);
}
