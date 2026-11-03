#![warn(rust_2018_idioms)]

use protoc_gen_protovalidate_buffa::{emit::render, scan::MessageValidators};

#[test]
fn nested_message_emits_owned_and_view_validators() {
    let files = render(&[MessageValidators {
        proto_name: "test.v1.Outer.Nested".to_string(),
        package: "test.v1".to_string(),
        source_file: "test/v1/nested.proto".to_string(),
        message_cel: Vec::new(),
        message_oneofs: Vec::new(),
        field_rules: Vec::new(),
        oneof_rules: Vec::new(),
        compile_error: None,
    }])
    .expect("render must succeed");
    let source = files
        .into_iter()
        .filter_map(|file| file.content)
        .collect::<String>();

    assert!(source.contains("Validate for outer::Nested"));
    assert!(source.contains("Validate for __buffa::view::outer::NestedView<'_>"));
    assert_eq!(source.matches("let mut violations").count(), 2);
    assert!(!source.contains("to_owned_message"));
}

#[test]
fn validator_packaging_uses_declared_package_with_unrelated_logical_filename() {
    let files = render(&[MessageValidators {
        proto_name: "pse.scientific_worker.v1.Request".into(),
        package: "pse.scientific_worker.v1".into(),
        source_file: "scientific_worker.proto".into(),
        message_cel: Vec::new(), message_oneofs: Vec::new(), field_rules: Vec::new(),
        oneof_rules: Vec::new(), compile_error: None,
    }]).expect("descriptor package determines packaging");
    let module = files.iter().find(|file| file.name.as_deref() == Some("mod.rs")).unwrap().content.as_ref().unwrap();
    assert!(module.contains("pub mod pse"));
    assert!(module.contains("pub mod scientific_worker"));
    assert!(module.contains("pub mod v1"));
    assert!(module.contains("use crate::proto::pse::scientific_worker::v1::*"));
    let leaf = files.iter().find(|file| file.name.as_deref() == Some("pse.scientific_worker.v1.mod.rs")).unwrap().content.as_ref().unwrap();
    assert!(leaf.contains("include!(\"scientific_worker.rs\")"));
}
