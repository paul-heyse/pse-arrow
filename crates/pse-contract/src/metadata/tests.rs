// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
const POLICY: CorePolicy<'_> = CorePolicy { interpretations: &[("pse.test.v1.Request", 1)] };

const OWNER: Owner<'_> = Owner { rust_type: "native::Request", declaration_file: "src/request.rs" };
const META: MessageMetadata<'_> = MessageMetadata {
    metadata_version: Some(VERSION), owner: Some(OWNER), interpretation_version: Some(1),
    direction: Some(EnumValue::Known(BoundaryDirection::Both)),
    construction: Some(EnumValue::Known(ConstructionRole::Data)),
    effects: None,
};
const FILES: &[File<'_>] = &[File { name: "request.proto", imports: &[] }];
const SYMBOLS: &[&str] = &["pse.test.v1.Request", "google.protobuf.MessageOptions", "google.protobuf.FieldOptions"];
const MESSAGE: Message<'_> = Message { name: "pse.test.v1.Request", file: "request.proto", metadata: Some(META) };
fn world<'a>(messages: &'a [Message<'a>]) -> World<'a> { World { files: FILES, symbols: SYMBOLS, messages, ..World::default() } }

#[test]
fn required_presence_and_unknown_interpretation_are_distinct() {
    assert_eq!(check(&world(&[MESSAGE]), &POLICY).disposition, Disposition::Compatible);
    assert_eq!(check(&world(&[MESSAGE]), &CorePolicy { interpretations: &[] }).disposition, Disposition::Unknown);
    let mut message = MESSAGE;
    message.metadata = None;
    let missing = check(&world(&[message]), &POLICY);
    assert_eq!(missing.disposition, Disposition::Incompatible);
    assert_eq!(missing.findings[0].file, "request.proto");
    message.metadata = Some(MessageMetadata { metadata_version: Some(99), direction: Some(EnumValue::Unknown(0)), ..META });
    let unknown = check(&world(&[message]), &POLICY);
    assert_eq!(unknown.disposition, Disposition::Unknown);
    assert_eq!(unknown.findings.len(), 2);
    assert_eq!(unknown.policy, "pse.contract.v1/core");
}

#[test]
fn wire_identical_role_and_version_changes_need_semantic_decisions() {
    let original = [MESSAGE];
    let old = world(&original);
    let changed = [Message { metadata: Some(MessageMetadata { construction: Some(EnumValue::Known(ConstructionRole::ObservationOnly)), ..META }), ..MESSAGE }];
    assert_eq!(compare(&old, &world(&changed), &POLICY).disposition, Disposition::Incompatible);
    let changed = [Message { metadata: Some(MessageMetadata { interpretation_version: Some(2), ..META }), ..MESSAGE }];
    assert_eq!(compare(&old, &world(&changed), &POLICY).disposition, Disposition::Unknown);
    struct DeclaredMigration;
    impl InterpretationPolicy for DeclaredMigration {
        fn name(&self) -> &str { "owner/declared-v1-v2" }
        fn supports_interpretation(&self, subject: &str, version: u32) -> bool { subject == "pse.test.v1.Request" && matches!(version, 1 | 2) }
        fn interpretation_change(&self, _: &str, old: u32, new: u32) -> Disposition {
            if (old, new) == (1, 2) { Disposition::Compatible } else { Disposition::Unknown }
        }
    }
    assert_eq!(compare(&old, &world(&changed), &DeclaredMigration).disposition, Disposition::Compatible);
}

#[test]
fn provenance_moves_do_not_become_physics_changes() {
    let changed = [Message { metadata: Some(MessageMetadata { owner: Some(Owner { rust_type: "new::Request", declaration_file: "src/moved.rs" }), ..META }), ..MESSAGE }];
    assert_eq!(compare(&world(&[MESSAGE]), &world(&changed), &POLICY).disposition, Disposition::Compatible);
}

#[test]
fn effect_changes_are_meaningful_even_without_wire_changes() {
    let observation = [Method { name: "pse.test.v1.Service.Run", file: "request.proto", input: SYMBOLS[0], output: SYMBOLS[0], metadata: Some(MethodMetadata { metadata_version: Some(VERSION), effects: Some(EffectFacts::OBSERVATION.into()) }) }];
    let mut execution = observation;
    execution[0].metadata.as_mut().unwrap().effects = Some(EffectFacts::NATIVE_EXECUTION.into());
    let original = world(&[]);
    let old = World { methods: &observation, ..original };
    let new = World { methods: &execution, ..original };
    assert_eq!(check(&new, &POLICY).disposition, Disposition::Compatible);
    assert_eq!(compare(&old, &new, &POLICY).disposition, Disposition::Incompatible);
    execution[0].metadata.as_mut().unwrap().effects.as_mut().unwrap().read_only = Some(true);
    assert_eq!(check(&World { methods: &execution, ..original }, &POLICY).disposition, Disposition::Incompatible);
}

#[test]
fn imports_references_and_extensions_use_the_actual_world() {
    let extensions = [
        Extension { name: "pse.contract.v1.message", extendee: SYMBOLS[1], number: 51065, file: "request.proto" },
        Extension { name: "pse.contract.v1.field", extendee: SYMBOLS[2], number: 51065, file: "request.proto" },
    ];
    assert_eq!(check(&World { extensions: &extensions, ..world(&[]) }, &POLICY).disposition, Disposition::Compatible);
    let mut collision = extensions;
    collision[1].extendee = SYMBOLS[1];
    assert_eq!(check(&World { extensions: &collision, ..world(&[]) }, &POLICY).disposition, Disposition::Incompatible);
    let files = [File { name: "request.proto", imports: &["missing.proto"] }];
    let missing = check(&World { files: &files, ..world(&[]) }, &POLICY);
    assert!(missing.findings.iter().any(|finding| finding.reason == "missing transitive import"));
    let fields = [Field { name: "pse.test.v1.Request.value", file: "request.proto", metadata: Some(FieldMetadata { metadata_version: Some(VERSION), native_field: Some("value"), direction: Some(EnumValue::Known(BoundaryDirection::Input)) }), reference: Some("pse.absent.v1.Value") }];
    assert_eq!(check(&World { fields: &fields, ..world(&[]) }, &POLICY).disposition, Disposition::Incompatible);
}

#[test]
fn unavailable_domain_policy_never_falls_back_to_wire_compatibility() {
    let facets = [Facet { subject: SYMBOLS[0], file: "request.proto", policy: "pse.schema.v1.registry_field", value: &[1, 2], interpretation_available: true }];
    let world = World { facets: &facets, ..world(&[]) };
    assert_eq!(check(&world, &POLICY).disposition, Disposition::Unknown);
    assert_eq!(compare(&world, &world, &POLICY).disposition, Disposition::Unknown);
}

#[test]
fn original_action_effects_are_checked_independently_of_methods() {
    let mut graph = crate::boundary_projection::ProjectionGraph::default();
    graph.declare::<()>("Action", file!(), 2, BoundaryDirection::Input, &[], &[], |_| Ok(crate::boundary_projection::Shape::Record(vec![]))).unwrap();
    graph.record_effects("Action", EffectFacts::OBSERVATION).unwrap();
    graph.record_effects("Action", EffectFacts::OBSERVATION).unwrap();
    assert_eq!(graph.definitions()["Action"].effects, Some(EffectFacts::OBSERVATION));
    assert!(graph.record_effects("Action", EffectFacts::SOURCE_ADMISSION).is_err());
    assert!(graph.record_effects("Absent", EffectFacts::OBSERVATION).is_err());
    let old = [Message { metadata: Some(MessageMetadata { effects: Some(EffectFacts::OBSERVATION.into()), ..META }), ..MESSAGE }];
    let new = [Message { metadata: Some(MessageMetadata { effects: Some(EffectFacts::SOURCE_ADMISSION.into()), ..META }), ..MESSAGE }];
    assert_eq!(compare(&world(&old), &world(&new), &POLICY).disposition, Disposition::Incompatible);
}

#[test]
fn newly_introduced_meaning_and_unavailable_original_policy_need_owner_decisions() {
    struct DomainPolicy;
    impl InterpretationPolicy for DomainPolicy {
        fn name(&self) -> &str { "domain/known-current" }
        fn supports_interpretation(&self, subject: &str, version: u32) -> bool { POLICY.supports_interpretation(subject, version) }
        fn check_facet(&self, _: &Facet<'_>) -> Disposition { Disposition::Compatible }
        fn compare_facet(&self, _: &Facet<'_>, _: &Facet<'_>) -> Disposition { Disposition::Compatible }
    }
    let messages = [MESSAGE];
    let old = world(&messages);
    let facets = [Facet { subject: MESSAGE.name, file: MESSAGE.file, policy: "domain.quantity", value: &[1], interpretation_available: true }];
    let new = World { facets: &facets, ..old };
    assert_eq!(check(&new, &DomainPolicy).disposition, Disposition::Compatible);
    assert_eq!(compare(&old, &new, &DomainPolicy).disposition, Disposition::Unknown, "introducing an understood facet needs a declared relation");
    let unavailable = [Facet { interpretation_available: false, ..facets[0] }];
    let original = World { facets: &unavailable, ..old };
    assert_eq!(compare(&original, &new, &DomainPolicy).disposition, Disposition::Unknown, "identical bytes and current policy cannot replace unavailable original interpretation");
}
