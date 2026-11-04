// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

const OWNER: Owner<'_> = Owner { rust_type: "native::Request", declaration_file: "src/request.rs" };
const META: MessageMetadata<'_> = MessageMetadata {
    metadata_version: Some(VERSION), owner: Some(OWNER), interpretation_version: Some(1),
    direction: Some(EnumValue::Known(BoundaryDirection::Both)),
    construction: Some(EnumValue::Known(ConstructionRole::Data)),
};
const FILES: &[File<'_>] = &[File { name: "request.proto", imports: &[] }];
const SYMBOLS: &[&str] = &["pse.test.v1.Request", "google.protobuf.MessageOptions", "google.protobuf.FieldOptions"];
const MESSAGE: Message<'_> = Message { name: "pse.test.v1.Request", file: "request.proto", metadata: Some(META) };
fn world<'a>(messages: &'a [Message<'a>]) -> World<'a> { World { files: FILES, symbols: SYMBOLS, messages, ..World::default() } }

#[test]
fn required_presence_and_unknown_interpretation_are_distinct() {
    assert_eq!(check(&world(&[MESSAGE]), &CorePolicy).disposition, Disposition::Compatible);
    let mut message = MESSAGE;
    message.metadata = None;
    let missing = check(&world(&[message]), &CorePolicy);
    assert_eq!(missing.disposition, Disposition::Incompatible);
    assert_eq!(missing.findings[0].file, "request.proto");
    message.metadata = Some(MessageMetadata { metadata_version: Some(99), direction: Some(EnumValue::Unknown(0)), ..META });
    let unknown = check(&world(&[message]), &CorePolicy);
    assert_eq!(unknown.disposition, Disposition::Unknown);
    assert_eq!(unknown.findings.len(), 2);
    assert_eq!(unknown.policy, "pse.contract.v1/core");
}

#[test]
fn wire_identical_role_and_version_changes_need_semantic_decisions() {
    let original = [MESSAGE];
    let old = world(&original);
    let changed = [Message { metadata: Some(MessageMetadata { construction: Some(EnumValue::Known(ConstructionRole::ObservationOnly)), ..META }), ..MESSAGE }];
    assert_eq!(compare(&old, &world(&changed), &CorePolicy).disposition, Disposition::Incompatible);
    let changed = [Message { metadata: Some(MessageMetadata { interpretation_version: Some(2), ..META }), ..MESSAGE }];
    assert_eq!(compare(&old, &world(&changed), &CorePolicy).disposition, Disposition::Unknown);
    struct DeclaredMigration;
    impl InterpretationPolicy for DeclaredMigration {
        fn name(&self) -> &str { "owner/declared-v1-v2" }
        fn interpretation_change(&self, _: &str, old: u32, new: u32) -> Disposition {
            if (old, new) == (1, 2) { Disposition::Compatible } else { Disposition::Unknown }
        }
    }
    assert_eq!(compare(&old, &world(&changed), &DeclaredMigration).disposition, Disposition::Compatible);
}

#[test]
fn provenance_moves_do_not_become_physics_changes() {
    let changed = [Message { metadata: Some(MessageMetadata { owner: Some(Owner { rust_type: "new::Request", declaration_file: "src/moved.rs" }), ..META }), ..MESSAGE }];
    assert_eq!(compare(&world(&[MESSAGE]), &world(&changed), &CorePolicy).disposition, Disposition::Compatible);
}

#[test]
fn effect_changes_are_meaningful_even_without_wire_changes() {
    let observation = [Method { name: "pse.test.v1.Service.Run", file: "request.proto", input: SYMBOLS[0], output: SYMBOLS[0], metadata: Some(MethodMetadata { metadata_version: Some(VERSION), effects: Some(EffectFacts::OBSERVATION.into()) }) }];
    let mut execution = observation;
    execution[0].metadata.as_mut().unwrap().effects = Some(EffectFacts::NATIVE_EXECUTION.into());
    let original = world(&[]);
    let old = World { methods: &observation, ..original };
    let new = World { methods: &execution, ..original };
    assert_eq!(check(&new, &CorePolicy).disposition, Disposition::Compatible);
    assert_eq!(compare(&old, &new, &CorePolicy).disposition, Disposition::Incompatible);
    execution[0].metadata.as_mut().unwrap().effects.as_mut().unwrap().read_only = Some(true);
    assert_eq!(check(&World { methods: &execution, ..original }, &CorePolicy).disposition, Disposition::Incompatible);
}

#[test]
fn imports_references_and_extensions_use_the_actual_world() {
    let extensions = [
        Extension { name: "pse.contract.v1.message", extendee: SYMBOLS[1], number: 51065, file: "request.proto" },
        Extension { name: "pse.contract.v1.field", extendee: SYMBOLS[2], number: 51065, file: "request.proto" },
    ];
    assert_eq!(check(&World { extensions: &extensions, ..world(&[]) }, &CorePolicy).disposition, Disposition::Compatible);
    let mut collision = extensions;
    collision[1].extendee = SYMBOLS[1];
    assert_eq!(check(&World { extensions: &collision, ..world(&[]) }, &CorePolicy).disposition, Disposition::Incompatible);
    let files = [File { name: "request.proto", imports: &["missing.proto"] }];
    let missing = check(&World { files: &files, ..world(&[]) }, &CorePolicy);
    assert!(missing.findings.iter().any(|finding| finding.reason == "missing transitive import"));
    let fields = [Field { name: "pse.test.v1.Request.value", file: "request.proto", metadata: Some(FieldMetadata { metadata_version: Some(VERSION), native_field: Some("value"), direction: Some(EnumValue::Known(BoundaryDirection::Input)) }), reference: Some("pse.absent.v1.Value") }];
    assert_eq!(check(&World { fields: &fields, ..world(&[]) }, &CorePolicy).disposition, Disposition::Incompatible);
}

#[test]
fn unavailable_domain_policy_never_falls_back_to_wire_compatibility() {
    let facets = [Facet { subject: SYMBOLS[0], file: "request.proto", policy: "pse.schema.v1.registry_field", value: &[1, 2] }];
    let world = World { facets: &facets, ..world(&[]) };
    assert_eq!(check(&world, &CorePolicy).disposition, Disposition::Unknown);
    assert_eq!(compare(&world, &world, &CorePolicy).disposition, Disposition::Unknown);
}
