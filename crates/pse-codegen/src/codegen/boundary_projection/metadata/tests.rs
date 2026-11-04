// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_contract::metadata::{CorePolicy, Disposition};
use buffa::Message as _;
struct FixturePolicy;
const POLICY: FixturePolicy = FixturePolicy;
impl pse_contract::metadata::InterpretationPolicy for FixturePolicy {
    fn name(&self) -> &str { "independent/source-v1" }
    fn supports_interpretation(&self, subject: &str, version: u32) -> bool { subject == "independent.v1.Request" && version == 1 }
}
impl DescriptorPolicy for FixturePolicy {
    fn native_selection(&self) -> Option<NativeSelection<'_>> {
        Some(NativeSelection {
            owners: &["independent.v1.Request"],
            fields: &["independent.v1.Request.value", "independent.v1.Request.detail", "independent.v1.Request.Alternative1.payload"],
            methods: &["independent.v1.Procedure.Run"],
        })
    }
}
fn compared(old: &FileDescriptorSet, new: &FileDescriptorSet) -> pse_contract::metadata::Assessment {
    compare_descriptors(DescriptorInput { descriptors: old, selected_file: "options.proto", policy: &POLICY }, DescriptorInput { descriptors: new, selected_file: "options.proto", policy: &POLICY }, &POLICY, &[]).unwrap()
}

const SOURCE: &str = include_str!("../../../../tests/fixtures/metadata/options.proto");
const FORWARDED: &str = include_str!("../../../../tests/fixtures/metadata/forwarded.proto");

fn compile(source: &str, forwarded: &str) -> Result<FileDescriptorSet, String> {
    compile_with_core(source, forwarded, None)
}
fn compile_with_core(source: &str, forwarded: &str, core_override: Option<&str>) -> Result<FileDescriptorSet, String> {
    let scratch = tempfile::tempdir().unwrap();
    let core = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../pse-contract/proto/pse/contract/v1/metadata.proto");
    let destination = scratch.path().join(CORE_IMPORT);
    std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
    if let Some(source) = core_override { std::fs::write(destination, source).unwrap(); }
    else { std::fs::copy(core, destination).unwrap(); }
    std::fs::write(scratch.path().join("options.proto"), source).unwrap();
    std::fs::write(scratch.path().join("forwarded.proto"), forwarded).unwrap();
    let output = std::process::Command::new("buf").arg("build").arg(scratch.path())
        .args(["--as-file-descriptor-set", "-o", "-"]).output().unwrap();
    if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).into()); }
    FileDescriptorSet::decode_from_slice(&output.stdout).map_err(|error| error.to_string())
}

#[test]
fn imported_core_schema_cannot_redefine_known_v1_meaning() {
    let source = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../pse-contract/proto/pse/contract/v1/metadata.proto")).unwrap();
    for (before, after) in [
        ("optional uint32 interpretation_version = 3;", "optional uint32 interpretation_version = 9;"),
        ("DIRECTION_INPUT = 1;", "DIRECTION_INPUT = 9;"),
        ("optional Effects effects = 6;", "optional Effects effects = 6; optional string hidden_semantics = 7;"),
    ] {
        assert_eq!(source.matches(before).count(), 1);
        let altered_schema = source.replace(before, after);
        let acquired = compile_with_core(SOURCE, FORWARDED, Some(&altered_schema)).unwrap();
        assert!(check_descriptors(&acquired, "options.proto", &POLICY, &[]).is_err(), "imported schema drift must not normalize to compatible v1");
    }
}

fn changed_source(old: &str, new: &str) -> String {
    assert_eq!(SOURCE.matches(old).count(), 1);
    SOURCE.replace(old, new)
}

#[test]
fn independent_source_options_reveal_wire_compatible_semantic_changes() {
    let old = compile(SOURCE, FORWARDED).unwrap();
    assert_eq!(check_descriptors(&old, "options.proto", &POLICY, &[]).unwrap().disposition, Disposition::Compatible);
    let role = compile(&changed_source("CONSTRUCTION_ROLE_DATA", "CONSTRUCTION_ROLE_OBSERVATION_ONLY"), FORWARDED).unwrap();
    assert_eq!(compared(&old, &role).disposition, Disposition::Incompatible);
    let version = compile(&changed_source("interpretation_version: 1", "interpretation_version: 2"), FORWARDED).unwrap();
    assert_eq!(compared(&old, &version).disposition, Disposition::Unknown);
    let effects = compile(&changed_source("scope: EFFECT_SCOPE_NATIVE_EXECUTION read_only: false idempotent: false", "scope: EFFECT_SCOPE_OBSERVATION read_only: true idempotent: true"), FORWARDED).unwrap();
    assert_eq!(compared(&old, &effects).disposition, Disposition::Incompatible);
    // The independent compiler's message fields and service signatures are equal;
    // only typed options differ. No generated setter is the expected-value oracle.
    let old_file = old.file.iter().find(|file| file.name.as_deref() == Some("options.proto")).unwrap();
    let role_file = role.file.iter().find(|file| file.name.as_deref() == Some("options.proto")).unwrap();
    assert_eq!(old_file.message_type[0].field, role_file.message_type[0].field);
    assert_eq!(old_file.service, role_file.service);
}

#[test]
fn unknown_or_missing_source_meaning_never_becomes_current_authority() {
    let unknown = compile(&changed_source("CONSTRUCTION_ROLE_DATA", "CONSTRUCTION_ROLE_UNSPECIFIED"), FORWARDED).unwrap();
    let assessment = check_descriptors(&unknown, "options.proto", &POLICY, &[]).unwrap();
    assert_eq!(assessment.disposition, Disposition::Unknown);
    assert!(assessment.findings.iter().any(|finding| finding.subject == "independent.v1.Request" && finding.file == "options.proto"));
    let missing = compile(&changed_source("interpretation_version: 1", ""), FORWARDED).unwrap();
    assert_eq!(check_descriptors(&missing, "options.proto", &POLICY, &[]).unwrap().disposition, Disposition::Incompatible);
    let moved = compile(&changed_source("src/request.rs", "src/moved.rs"), FORWARDED).unwrap();
    let old = compile(SOURCE, FORWARDED).unwrap();
    assert_eq!(compared(&old, &moved).disposition, Disposition::Compatible);
}

#[test]
fn actual_compiler_imports_and_unknown_options_survive_lossless_descriptor_roundtrip() {
    let world = compile(SOURCE, FORWARDED).unwrap();
    let recovered = FileDescriptorSet::decode_from_slice(&world.encode_to_vec()).unwrap();
    assert_eq!(world, recovered);
    assert!(recovered.file.iter().any(|file| file.name.as_deref() == Some(CORE_IMPORT)));
    let file = recovered.file.iter().find(|file| file.name.as_deref() == Some("options.proto")).unwrap();
    let options = file.message_type[0].field[0].options.as_option().unwrap();
    assert!(options.unknown_fields().iter().any(|field| field.number == 51999 && field.data == buffa::UnknownFieldData::LengthDelimited(b"opaque-import-preserved".to_vec())));
    let collision = FORWARDED.replace("51999", "51065");
    assert!(compile(SOURCE, &collision).is_err(), "the actual compiler refuses conflicting extendee allocations");
}

#[test]
fn required_source_coverage_survives_stripped_options_and_removed_descriptors() {
    let original = compile(SOURCE, FORWARDED).unwrap();
    for target in 0..4 {
        let mut stripped = original.clone();
        let file = stripped.file.iter_mut().find(|file| file.name.as_deref() == Some("options.proto")).unwrap();
        let unknown = match target {
            0 => file.message_type[0].options.as_option_mut().unwrap().unknown_fields_mut(),
            1 => file.message_type[0].field[0].options.as_option_mut().unwrap().unknown_fields_mut(),
            2 => file.service[0].method[0].options.as_option_mut().unwrap().unknown_fields_mut(),
            _ => file.message_type[0].nested_type[0].field[0].options.as_option_mut().unwrap().unknown_fields_mut(),
        };
        unknown.retain(|field| field.number != 51065);
        let assessment = check_descriptors(&stripped, "options.proto", &POLICY, &[]).unwrap();
        assert_eq!(assessment.disposition, Disposition::Incompatible, "stripping selected options cannot remove the source obligation");
        assert!(assessment.findings.iter().any(|finding| finding.reason.contains("missing required") && finding.file == "options.proto"));
    }
    for target in 0..3 {
        let mut removed = original.clone();
        let file = removed.file.iter_mut().find(|file| file.name.as_deref() == Some("options.proto")).unwrap();
        match target {
            0 => { file.message_type.clear(); file.service.clear(); },
            1 => file.message_type[0].nested_type[0].field.clear(),
            _ => file.service[0].method.clear(),
        }
        assert!(check_descriptors(&removed, "options.proto", &POLICY, &[]).is_err(), "removing selected descriptors is not vacuous compatibility");
    }
    let inspection = CorePolicy { interpretations: &[("independent.v1.Request", 1)] };
    assert_eq!(check_descriptors(&original, "options.proto", &inspection, &[]).unwrap().disposition, Disposition::Unknown);
    let input = || DescriptorInput { descriptors: &original, selected_file: "options.proto", policy: &inspection };
    assert_eq!(compare_descriptors(input(), input(), &inspection, &[]).unwrap().disposition, Disposition::Unknown);
}

#[test]
fn selected_unknown_core_payload_fields_are_retained_then_refused_with_source() {
    let mut world = compile(SOURCE, FORWARDED).unwrap();
    let encoder = OptionEncoder::new(&world.file).unwrap();
    let file = world.file.iter_mut().find(|file| file.name.as_deref() == Some("options.proto")).unwrap();
    let options = file.message_type[0].options.as_option_mut().unwrap();
    let mut metadata = encoder.decode_extension(options, "pse.contract.v1.message").unwrap().unwrap();
    let mut additional = Vec::new();
    buffa::UnknownField { number: 999, data: buffa::UnknownFieldData::Varint(1) }.write_to(&mut additional);
    metadata.merge(&additional).unwrap();
    options.unknown_fields_mut().retain(|field| field.number != 51065);
    encoder.attach(options, "pse.contract.v1.message", metadata).unwrap();
    let failure = check_descriptors(&world, "options.proto", &POLICY, &[]).unwrap_err().to_string();
    assert!(failure.contains("options.proto:independent.v1.Request"), "{failure}");
    assert!(failure.contains("unknown selected metadata field 999"), "{failure}");
}

#[test]
fn typed_encoder_refuses_schema_drift_extendee_confusion_and_duplicate_options() {
    let imports = super::super::tests::imports();
    let encoder = OptionEncoder::new(&imports).unwrap();
    assert!(encoder.message("pse.contract.v1.FieldMetadata", &[("metadata_version", Value::Bool(true))]).is_err());
    let message = encoder.message("pse.contract.v1.FieldMetadata", &[("metadata_version", Value::U32(1))]).unwrap();
    let mut options = descriptor::MessageOptions::default();
    assert!(encoder.attach(&mut options, "pse.contract.v1.field", message.clone()).is_err());
    let mut options = descriptor::FieldOptions::default();
    encoder.attach(&mut options, "pse.contract.v1.field", message.clone()).unwrap();
    assert!(encoder.attach(&mut options, "pse.contract.v1.field", message).is_err());
    let mut changed = imports;
    let schema = changed.iter_mut().find(|file| file.name.as_deref() == Some(CORE_IMPORT)).unwrap();
    schema.enum_type.iter_mut().find(|enumeration| enumeration.name.as_deref() == Some("Direction")).unwrap().value[1].number = Some(9);
    assert!(OptionEncoder::new(&changed).is_err());
}

struct QuantityAnnotatedOwner;
impl pse_contract::boundary_projection::ProjectionType for QuantityAnnotatedOwner {
    fn projection(graph: &mut pse_contract::boundary_projection::ProjectionGraph) -> Result<pse_contract::boundary_projection::TypeRef, pse_contract::boundary_projection::ProjectionError> {
        use pse_contract::boundary_projection::{Direction, Field, Shape};
        graph.declare::<Self>("QuantityAnnotatedOwner", file!(), 1, Direction::Both, &[], &[], |graph| Ok(Shape::Record(vec![
            Field::of::<u64>(graph, "temperature", 1, Direction::Both, vec![])?.with_named_facets(std::collections::BTreeMap::from([
                (pse_schema::model::field_facets::ROLE.into(), "measure".into()),
                (pse_schema::model::field_facets::QUANTITY.into(), "quantity.temperature".into()),
            ])),
        ])))
    }
}

#[test]
fn physical_facet_reaches_inspection_and_requires_its_selected_owner_policy() {
    use super::super::registry_facets;
    let graph = pse_contract::boundary_projection::ProjectionGraph::of::<QuantityAnnotatedOwner>().unwrap();
    let set = super::super::descriptors_with_imports(&graph, "pse.physical.v1", "physical.proto", &super::super::tests::imports()).unwrap();
    let extensions = [registry_facets::EXTENSION];
    assert_eq!(check_descriptors(&set, "physical.proto", &CorePolicy { interpretations: &[("pse.physical.v1.QuantityAnnotatedOwner", 1)] }, &extensions).unwrap().disposition, Disposition::Unknown);
    let encoder = OptionEncoder::new(&set.file).unwrap();
    let validators: [(&str, DomainCheck); 1] = [(registry_facets::EXTENSION, registry_facets::check_option)];
    let selection = NativeSelection { owners: &["pse.physical.v1.QuantityAnnotatedOwner"], fields: &["pse.physical.v1.QuantityAnnotatedOwner.temperature"], methods: &[] };
    let policy = TypedPolicy::new(&validators, &[("pse.physical.v1.QuantityAnnotatedOwner", 1)], selection);
    assert_eq!(check_descriptors(&set, "physical.proto", &policy, &extensions).unwrap().disposition, Disposition::Compatible);
    let file = set.file.iter().find(|file| file.name.as_deref() == Some("physical.proto")).unwrap();
    let option = encoder.decode_extension(file.message_type[0].field[0].options.as_option().unwrap(), registry_facets::EXTENSION).unwrap().unwrap();
    let Value::Message(quantity) = option.field_by_number(6).unwrap() else { panic!("quantity reference is typed"); };
    assert_eq!(quantity.field_by_number(1), Some(&Value::String("quantity.temperature".into())));
    let mut before = set.clone();
    let file = before.file.iter_mut().find(|file| file.name.as_deref() == Some("physical.proto")).unwrap();
    file.message_type[0].field[0].options.as_option_mut().unwrap().unknown_fields_mut().retain(|field| field.number != 51066);
    let compared = compare_descriptors(DescriptorInput { descriptors: &before, selected_file: "physical.proto", policy: &policy }, DescriptorInput { descriptors: &set, selected_file: "physical.proto", policy: &policy }, &policy, &extensions).unwrap();
    assert_eq!(compared.disposition, Disposition::Incompatible, "a valid newly introduced physical facet changes existing field meaning");
}
