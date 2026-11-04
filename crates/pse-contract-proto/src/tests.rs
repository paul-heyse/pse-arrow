// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use buffa::Message as _;
use pse_contract::metadata::{CorePolicy, Disposition};
struct FixturePolicy;
const POLICY: FixturePolicy = FixturePolicy;
impl pse_contract::metadata::InterpretationPolicy for FixturePolicy {
    fn name(&self) -> &str {
        "independent/source-v1"
    }
    fn supports_interpretation(&self, subject: &str, version: u32) -> bool {
        subject == "independent.v1.Request" && version == 1
    }
}
impl DescriptorPolicy for FixturePolicy {
    fn native_selection(&self) -> Option<NativeSelection<'_>> {
        Some(NativeSelection {
            owners: &["independent.v1.Request"],
            fields: &[
                "independent.v1.Request.value",
                "independent.v1.Request.detail",
                "independent.v1.Request.Alternative1.payload",
            ],
            methods: &["independent.v1.Procedure.Run"],
            facets: &[],
            facet_fields: &[],
        })
    }
}
fn compared(
    old: &FileDescriptorSet,
    new: &FileDescriptorSet,
) -> pse_contract::metadata::Assessment {
    compare_descriptors(
        DescriptorInput {
            descriptors: old,
            selected_file: "options.proto",
            policy: &POLICY,
        },
        DescriptorInput {
            descriptors: new,
            selected_file: "options.proto",
            policy: &POLICY,
        },
        &POLICY,
        &[],
    )
    .unwrap()
}

const SOURCE: &str = include_str!("../tests/fixtures/metadata/options.proto");
const FORWARDED: &str = include_str!("../tests/fixtures/metadata/forwarded.proto");
const NATIVE_FIELDS: &str = include_str!("../tests/fixtures/metadata/native-fields.proto");

#[test]
fn independent_within_world_field_copies_cannot_agree_only_across_worlds() {
    let baseline = compile(NATIVE_FIELDS, FORWARDED).unwrap();
    assert_eq!(
        check_descriptors(&baseline, "options.proto", &NativeFactsPolicy, &[])
            .unwrap()
            .disposition,
        Disposition::Compatible
    );
    for (before, after) in [
        ("NATIVE_SCALAR_U64", "NATIVE_SCALAR_I64"),
        (
            "name: \"value\" tag: 1 direction: DIRECTION_BOTH",
            "name: \"different\" tag: 1 direction: DIRECTION_BOTH",
        ),
        (
            "name: \"value\" tag: 1 direction: DIRECTION_BOTH",
            "name: \"value\" tag: 1 direction: DIRECTION_INPUT",
        ),
        (
            "NATIVE_FLOAT64_PRESET_FINITE",
            "NATIVE_FLOAT64_PRESET_POSITIVE_FINITE",
        ),
    ] {
        // Only the first source declaration changes; occurrence metadata stays
        // untouched. This compiler fixture is independent of option setters.
        let changed = NATIVE_FIELDS.replacen(before, after, 1);
        assert_ne!(changed, NATIVE_FIELDS);
        let world = compile(&changed, FORWARDED).unwrap();
        assert_ne!(
            compare_descriptors(
                DescriptorInput {
                    descriptors: &world,
                    selected_file: "options.proto",
                    policy: &NativeFactsPolicy
                },
                DescriptorInput {
                    descriptors: &world,
                    selected_file: "options.proto",
                    policy: &NativeFactsPolicy
                },
                &NativeFactsPolicy,
                &[]
            )
            .unwrap()
            .disposition,
            Disposition::Compatible,
            "contradiction {before} must be refused or unavailable"
        );
    }
    let baseline = compile(SOURCE, FORWARDED).unwrap();
    assert_eq!(
        compared(&baseline, &baseline).disposition,
        Disposition::Compatible
    );
    for (before, after) in [
        ("native_field: \"detail\"", "native_field: \"wrong\""),
        (
            "native_field: \"payload\" direction: DIRECTION_INPUT",
            "native_field: \"payload\" direction: DIRECTION_BOTH",
        ),
        (
            "scalar: NATIVE_SCALAR_STRING",
            "scalar: NATIVE_SCALAR_BYTES",
        ),
    ] {
        let changed = SOURCE.replacen(before, after, 1);
        assert_ne!(changed, SOURCE);
        let world = compile(&changed, FORWARDED).unwrap();
        assert_eq!(
            compared(&world, &world).disposition,
            Disposition::Incompatible
        );
    }
}

#[test]
fn independent_string_vocabulary_wrapper_uses_its_original_owner_declaration() {
    struct Policy;
    impl pse_contract::metadata::InterpretationPolicy for Policy {
        fn name(&self) -> &str {
            "independent/string-vocabulary"
        }
        fn supports_interpretation(&self, subject: &str, version: u32) -> bool {
            subject == "independent.v1.Request" && version == 1
        }
    }
    impl DescriptorPolicy for Policy {
        fn native_selection(&self) -> Option<NativeSelection<'_>> {
            Some(NativeSelection {
                owners: &["independent.v1.Request"],
                fields: &["independent.v1.Request.value"],
                methods: &[],
                facets: &[],
                facet_fields: &[],
            })
        }
    }
    let source = r#"syntax="proto2"; package independent.v1; import "pse/contract/v1/metadata.proto";
message Request {
option (pse.contract.v1.message) = { metadata_version:2 owner:{rust_type:"native::Vocabulary" declaration_file:"vocabulary.rs"} interpretation_version:1 direction:DIRECTION_BOTH construction_role:CONSTRUCTION_ROLE_DATA native_shape:{string_enum:{values:"first" values:"second"}} };
optional string value=1 [(pse.contract.v1.field)={metadata_version:2 native_field:"value" direction:DIRECTION_BOTH native_contract:{native_type:{named_owner:"independent.v1.Request"}}}];
}"#;
    let old = compile(source, FORWARDED).unwrap();
    assert_eq!(
        check_descriptors(&old, "options.proto", &Policy, &[])
            .unwrap()
            .disposition,
        Disposition::Compatible
    );
    for changed in [
        source.replace("native_field:\"value\"", "native_field:\"other\""),
        source.replace(
            "native_type:{named_owner:\"independent.v1.Request\"}",
            "native_type:{scalar:NATIVE_SCALAR_STRING}",
        ),
    ] {
        let world = compile(&changed, FORWARDED).unwrap();
        assert_eq!(
            compare_descriptors(
                DescriptorInput {
                    descriptors: &world,
                    selected_file: "options.proto",
                    policy: &Policy
                },
                DescriptorInput {
                    descriptors: &world,
                    selected_file: "options.proto",
                    policy: &Policy
                },
                &Policy,
                &[]
            )
            .unwrap()
            .disposition,
            Disposition::Incompatible
        );
    }
    let changed = compile(
        &source.replace("values:\"second\"", "values:\"third\""),
        FORWARDED,
    )
    .unwrap();
    assert_eq!(
        compare_descriptors(
            DescriptorInput {
                descriptors: &old,
                selected_file: "options.proto",
                policy: &Policy
            },
            DescriptorInput {
                descriptors: &changed,
                selected_file: "options.proto",
                policy: &Policy
            },
            &Policy,
            &[]
        )
        .unwrap()
        .disposition,
        Disposition::Incompatible
    );
}

#[test]
fn supported_native_owner_cannot_omit_selected_record_or_alternative_members() {
    let old = compile(NATIVE_FIELDS, FORWARDED).unwrap();
    assert_eq!(
        check_descriptors(&old, "options.proto", &NativeFactsPolicy, &[])
            .unwrap()
            .disposition,
        Disposition::Compatible
    );
    let missing_all = NATIVE_FIELDS
        .lines()
        .filter(|line| !line.trim_start().starts_with("fields:"))
        .collect::<Vec<_>>()
        .join("\n");
    let renamed = NATIVE_FIELDS.replacen("name: \"value\" tag: 1", "name: \"different\" tag: 1", 1);
    for source in [missing_all, renamed] {
        let world = compile(&source, FORWARDED).unwrap();
        let assessment = compare_descriptors(
            DescriptorInput {
                descriptors: &world,
                selected_file: "options.proto",
                policy: &NativeFactsPolicy,
            },
            DescriptorInput {
                descriptors: &world,
                selected_file: "options.proto",
                policy: &NativeFactsPolicy,
            },
            &NativeFactsPolicy,
            &[],
        )
        .unwrap();
        assert_eq!(assessment.disposition, Disposition::Incompatible);
        assert!(
            assessment
                .findings
                .iter()
                .any(|finding| finding.reason.contains("absent from its known owner"))
        );
    }
    let arm = "      alternatives: { name: \"detail\" tag: 2 fields: { name: \"payload\" tag: 1 direction: DIRECTION_INPUT contract: { native_type: { scalar: NATIVE_SCALAR_STRING } } } }";
    assert_eq!(SOURCE.matches(arm).count(), 1);
    for source in [
        SOURCE.replace(arm, ""),
        SOURCE.replace(arm, "      alternatives: { name: \"detail\" tag: 2 }"),
        SOURCE.replace(arm, &arm.replace("name: \"detail\"", "name: \"renamed\"")),
    ] {
        let world = compile(&source, FORWARDED).unwrap();
        assert_eq!(
            compared(&world, &world).disposition,
            Disposition::Incompatible
        );
    }
}

struct NativeFactsPolicy;
impl pse_contract::metadata::InterpretationPolicy for NativeFactsPolicy {
    fn name(&self) -> &str {
        "independent/native-field-v2"
    }
    fn supports_interpretation(&self, subject: &str, version: u32) -> bool {
        subject == "independent.v1.Request" && version == 1
    }
}
impl DescriptorPolicy for NativeFactsPolicy {
    fn native_selection(&self) -> Option<NativeSelection<'_>> {
        Some(NativeSelection {
            owners: &["independent.v1.Request"],
            fields: &[
                "independent.v1.Request.value",
                "independent.v1.Request.cells",
                "independent.v1.Request.identity",
                "independent.v1.Request.sample",
            ],
            methods: &[],
            facets: &[],
            facet_fields: &[],
        })
    }
}

#[test]
fn independent_native_contract_changes_are_incompatible_with_identical_wire_declarations() {
    let old = compile(NATIVE_FIELDS, FORWARDED).unwrap();
    for (before, after) in [
        (
            "native_type: { scalar: NATIVE_SCALAR_U64 }",
            "native_type: { scalar: NATIVE_SCALAR_I64 }",
        ),
        ("set_value", "repeated_value"),
        ("NATIVE_SCALAR_SEMANTIC_ID", "NATIVE_SCALAR_BYTES"),
        (
            "NATIVE_FLOAT64_PRESET_FINITE",
            "NATIVE_FLOAT64_PRESET_POSITIVE_FINITE",
        ),
    ] {
        assert_eq!(
            NATIVE_FIELDS.matches(before).count(),
            2,
            "literal source owner shape and occurrence facts both retain the native declaration"
        );
        let new = compile(&NATIVE_FIELDS.replace(before, after), FORWARDED).unwrap();
        let selected = |world: &FileDescriptorSet| {
            world
                .file
                .iter()
                .find(|file| file.name.as_deref() == Some("options.proto"))
                .unwrap()
                .message_type[0]
                .field
                .iter()
                .map(|field| {
                    (
                        field.name.clone(),
                        field.number,
                        field.r#type,
                        field.type_name.clone(),
                        field.label,
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            selected(&old),
            selected(&new),
            "native meaning changes independently of every selected wire declaration"
        );
        assert_eq!(
            compare_descriptors(
                DescriptorInput {
                    descriptors: &old,
                    selected_file: "options.proto",
                    policy: &NativeFactsPolicy
                },
                DescriptorInput {
                    descriptors: &new,
                    selected_file: "options.proto",
                    policy: &NativeFactsPolicy
                },
                &NativeFactsPolicy,
                &[]
            )
            .unwrap()
            .disposition,
            Disposition::Incompatible
        );
    }
}

#[test]
fn legacy_missing_and_unknown_native_contracts_never_become_current_compatibility() {
    // Exact historical authored sources from the pre-v2 source checkpoint;
    // retained only as independent inputs, with no legacy producer/reader path.
    let legacy = compile_with_core(
        include_str!("../tests/fixtures/metadata/options-v1.proto"),
        FORWARDED,
        Some(include_str!("../tests/fixtures/metadata/core-v1.proto")),
    )
    .unwrap();
    assert_eq!(
        check_descriptors(&legacy, "options.proto", &POLICY, &[])
            .unwrap()
            .disposition,
        Disposition::Unknown
    );
    assert_eq!(compared(&legacy, &legacy).disposition, Disposition::Unknown);
    let current = compile(SOURCE, FORWARDED).unwrap();
    assert_eq!(
        compared(&legacy, &current).disposition,
        Disposition::Unknown
    );
    for (before, after, expected) in [
        (
            "native_contract: { native_type: { scalar: NATIVE_SCALAR_U64 } }",
            "",
            Disposition::Incompatible,
        ),
        (
            "native_type: { scalar: NATIVE_SCALAR_U64 }",
            "native_type: { scalar: NATIVE_SCALAR_UNSPECIFIED }",
            Disposition::Unknown,
        ),
        (
            "NATIVE_FLOAT64_PRESET_FINITE",
            "NATIVE_FLOAT64_PRESET_UNSPECIFIED",
            Disposition::Unknown,
        ),
        (
            "metadata_version: 2",
            "metadata_version: 99",
            Disposition::Unknown,
        ),
    ] {
        let world = compile(&NATIVE_FIELDS.replace(before, after), FORWARDED).unwrap();
        assert_eq!(
            check_descriptors(&world, "options.proto", &NativeFactsPolicy, &[])
                .unwrap()
                .disposition,
            expected
        );
    }
}

#[test]
fn native_field_codec_preserves_existing_owners_and_exact_interval_bits() {
    use pse_contract::boundary_projection::{Float64Domain, Float64Interval, Scalar, TypeRef};
    let world = compile(NATIVE_FIELDS, FORWARDED).unwrap();
    let encoder = OptionEncoder::new(&world.file).unwrap();
    let roundtrip = |ty: &TypeRef, rules: &[LocalRule]| {
        let metadata = encoder
            .message(
                "pse.contract.v1.FieldMetadata",
                &[
                    (
                        "metadata_version",
                        Value::U32(pse_contract::metadata::VERSION),
                    ),
                    (
                        "native_contract",
                        Value::Message(
                            encoder
                                .native_contract(ty, rules, "independent.v1")
                                .unwrap(),
                        ),
                    ),
                ],
            )
            .unwrap();
        assert_eq!(
            field_contract::decode(&metadata).unwrap(),
            Some((ty.clone(), rules.to_vec()))
        );
    };
    for scalar in [
        Scalar::Bool,
        Scalar::String,
        Scalar::U32,
        Scalar::U8,
        Scalar::U16,
        Scalar::U64,
        Scalar::I16,
        Scalar::I32,
        Scalar::I64,
        Scalar::F32Bits,
        Scalar::F64Bits,
        Scalar::Bytes,
        Scalar::SemanticId,
        Scalar::ContentHash,
        Scalar::I128Bits,
        Scalar::I8,
    ] {
        roundtrip(&TypeRef::Scalar(scalar), &[]);
    }
    let named = TypeRef::Named("independent.v1.Request".into());
    let values = TypeRef::Repeated(Box::new(TypeRef::Set(Box::new(named))));
    let mapping = TypeRef::Map(
        Box::new(TypeRef::Scalar(Scalar::SemanticId)),
        Box::new(values),
    );
    roundtrip(&TypeRef::Optional(Box::new(mapping)), &[]);
    for domain in [
        Float64Domain::Finite,
        Float64Domain::NonnegativeFinite,
        Float64Domain::PositiveFinite,
        Float64Domain::PositiveUnit,
        Float64Domain::Interval(Float64Interval::new(-0.0, false, 1.0, true)),
    ] {
        roundtrip(
            &TypeRef::Scalar(Scalar::F64Bits),
            &[LocalRule::Float64Domain(domain)],
        );
    }
    roundtrip(
        &TypeRef::Scalar(Scalar::I64),
        &[LocalRule::SignedRange {
            min: i64::MIN,
            max: -1,
        }],
    );
    roundtrip(
        &TypeRef::Scalar(Scalar::U64),
        &[LocalRule::UnsignedRange {
            min: 0,
            max: u64::MAX,
        }],
    );
    roundtrip(
        &TypeRef::Repeated(Box::new(TypeRef::Scalar(Scalar::Bool))),
        &[LocalRule::Length { min: 0, max: 17 }],
    );
    roundtrip(
        &TypeRef::Scalar(Scalar::String),
        &[LocalRule::Utf8Bytes { min: 1, max: 8 }],
    );
    roundtrip(
        &TypeRef::Optional(Box::new(TypeRef::Scalar(Scalar::Bool))),
        &[LocalRule::Required],
    );
    roundtrip(
        &TypeRef::Named("independent.v1.StdDuration".into()),
        &[LocalRule::PositiveDuration],
    );
}

#[test]
fn signed_i128_metadata_allocation_is_distinct_and_unknown_readers_do_not_default() {
    let source = NATIVE_FIELDS.replace("NATIVE_SCALAR_SEMANTIC_ID", "NATIVE_SCALAR_I128_BITS");
    let old = compile(NATIVE_FIELDS, FORWARDED).unwrap();
    let current = compile(&source, FORWARDED).unwrap();
    let compare = |old: &FileDescriptorSet, new: &FileDescriptorSet| {
        compare_descriptors(
            DescriptorInput {
                descriptors: old,
                selected_file: "options.proto",
                policy: &NativeFactsPolicy,
            },
            DescriptorInput {
                descriptors: new,
                selected_file: "options.proto",
                policy: &NativeFactsPolicy,
            },
            &NativeFactsPolicy,
            &[],
        )
        .unwrap()
    };
    assert_eq!(
        compare(&current, &current).disposition,
        Disposition::Compatible
    );
    assert_eq!(
        compare(&old, &current).disposition,
        Disposition::Incompatible,
        "same bytes wire type does not equate native identity and signed integer bits"
    );
    let unknown = compile(
        &source.replace("NATIVE_SCALAR_I128_BITS", "NATIVE_SCALAR_UNSPECIFIED"),
        FORWARDED,
    )
    .unwrap();
    assert_eq!(
        check_descriptors(&unknown, "options.proto", &NativeFactsPolicy, &[])
            .unwrap()
            .disposition,
        Disposition::Unknown
    );
    let core = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pse-contract/proto/pse/contract/v1/metadata.proto"),
    )
    .unwrap();
    for schema in [
        core.replace(
            "NATIVE_SCALAR_I128_BITS = 15;",
            "NATIVE_SCALAR_I128_BITS = 17;",
        ),
        core.replace("NATIVE_SCALAR_I128_BITS = 15;", ""),
    ] {
        // A reader's known schema vocabulary and the acquired world disagree;
        // neither an old world nor a changed allocation becomes known by default.
        let world = compile_with_core(NATIVE_FIELDS, FORWARDED, Some(&schema)).unwrap();
        assert_eq!(
            check_descriptors(&world, "options.proto", &NativeFactsPolicy, &[])
                .unwrap()
                .disposition,
            Disposition::Unknown
        );
    }
}

fn compile(source: &str, forwarded: &str) -> Result<FileDescriptorSet, String> {
    compile_with_core(source, forwarded, None)
}
fn compile_with_core(
    source: &str,
    forwarded: &str,
    core_override: Option<&str>,
) -> Result<FileDescriptorSet, String> {
    let scratch = tempfile::tempdir().unwrap();
    let core = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pse-contract/proto/pse/contract/v1/metadata.proto");
    let destination = scratch.path().join(CORE_IMPORT);
    std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
    if let Some(source) = core_override {
        std::fs::write(destination, source).unwrap();
    } else {
        std::fs::copy(core, destination).unwrap();
    }
    std::fs::write(scratch.path().join("options.proto"), source).unwrap();
    std::fs::write(scratch.path().join("forwarded.proto"), forwarded).unwrap();
    let output = std::process::Command::new("buf")
        .arg("build")
        .arg(scratch.path())
        .args(["--as-file-descriptor-set", "-o", "-"])
        .output()
        .unwrap();
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }
    FileDescriptorSet::decode_from_slice(&output.stdout).map_err(|error| error.to_string())
}

#[test]
fn imported_core_schema_cannot_redefine_known_v2_meaning() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pse-contract/proto/pse/contract/v1/metadata.proto"),
    )
    .unwrap();
    for (before, after) in [
        (
            "optional uint32 interpretation_version = 3;",
            "optional uint32 interpretation_version = 9;",
        ),
        (
            "optional uint32 interpretation_version = 3;",
            "optional uint32 interpretation_version = 3 [default = 1];",
        ),
        ("DIRECTION_INPUT = 1;", "DIRECTION_INPUT = 9;"),
        (
            "optional Effects effects = 6;",
            "optional Effects effects = 6; optional string hidden_semantics = 8;",
        ),
    ] {
        assert_eq!(source.matches(before).count(), 1);
        let altered_schema = source.replace(before, after);
        let acquired = compile_with_core(SOURCE, FORWARDED, Some(&altered_schema)).unwrap();
        assert_eq!(
            check_descriptors(&acquired, "options.proto", &POLICY, &[])
                .unwrap()
                .disposition,
            Disposition::Unknown,
            "imported schema drift must not normalize to compatible v2"
        );
    }
}

fn changed_source(old: &str, new: &str) -> String {
    assert_eq!(SOURCE.matches(old).count(), 1);
    SOURCE.replace(old, new)
}

#[test]
fn independent_source_options_reveal_wire_compatible_semantic_changes() {
    let old = compile(SOURCE, FORWARDED).unwrap();
    assert_eq!(
        check_descriptors(&old, "options.proto", &POLICY, &[])
            .unwrap()
            .disposition,
        Disposition::Compatible
    );
    let role = compile(
        &changed_source(
            "CONSTRUCTION_ROLE_DATA",
            "CONSTRUCTION_ROLE_OBSERVATION_ONLY",
        ),
        FORWARDED,
    )
    .unwrap();
    assert_eq!(compared(&old, &role).disposition, Disposition::Incompatible);
    let version = compile(
        &changed_source("interpretation_version: 1", "interpretation_version: 2"),
        FORWARDED,
    )
    .unwrap();
    assert_eq!(compared(&old, &version).disposition, Disposition::Unknown);
    let effects = compile(
        &changed_source(
            "scope: EFFECT_SCOPE_NATIVE_EXECUTION read_only: false idempotent: false",
            "scope: EFFECT_SCOPE_OBSERVATION read_only: true idempotent: true",
        ),
        FORWARDED,
    )
    .unwrap();
    assert_eq!(
        compared(&old, &effects).disposition,
        Disposition::Incompatible
    );
    // The independent compiler's message fields and service signatures are equal;
    // only typed options differ. No generated setter is the expected-value oracle.
    let old_file = old
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("options.proto"))
        .unwrap();
    let role_file = role
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("options.proto"))
        .unwrap();
    assert_eq!(
        old_file.message_type[0].field,
        role_file.message_type[0].field
    );
    assert_eq!(old_file.service, role_file.service);
}

#[test]
fn unknown_or_missing_source_meaning_never_becomes_current_authority() {
    let unknown = compile(
        &changed_source("CONSTRUCTION_ROLE_DATA", "CONSTRUCTION_ROLE_UNSPECIFIED"),
        FORWARDED,
    )
    .unwrap();
    let assessment = check_descriptors(&unknown, "options.proto", &POLICY, &[]).unwrap();
    assert_eq!(assessment.disposition, Disposition::Unknown);
    assert!(
        assessment
            .findings
            .iter()
            .any(|finding| finding.subject == "independent.v1.Request"
                && finding.file == "options.proto")
    );
    let missing = compile(&changed_source("interpretation_version: 1", ""), FORWARDED).unwrap();
    assert_eq!(
        check_descriptors(&missing, "options.proto", &POLICY, &[])
            .unwrap()
            .disposition,
        Disposition::Incompatible
    );
    let moved = compile(&changed_source("src/request.rs", "src/moved.rs"), FORWARDED).unwrap();
    let old = compile(SOURCE, FORWARDED).unwrap();
    assert_eq!(compared(&old, &moved).disposition, Disposition::Compatible);
}

#[test]
fn actual_compiler_imports_and_unknown_options_survive_lossless_descriptor_roundtrip() {
    let world = compile(SOURCE, FORWARDED).unwrap();
    let recovered = FileDescriptorSet::decode_from_slice(&world.encode_to_vec()).unwrap();
    assert_eq!(world, recovered);
    assert!(
        recovered
            .file
            .iter()
            .any(|file| file.name.as_deref() == Some(CORE_IMPORT))
    );
    let file = recovered
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("options.proto"))
        .unwrap();
    let options = file.message_type[0].field[0].options.as_option().unwrap();
    assert!(
        options
            .unknown_fields()
            .iter()
            .any(|field| field.number == 51999
                && field.data
                    == buffa::UnknownFieldData::LengthDelimited(
                        b"opaque-import-preserved".to_vec()
                    ))
    );
    let collision = FORWARDED.replace("51999", "51065");
    assert!(
        compile(SOURCE, &collision).is_err(),
        "the actual compiler refuses conflicting extendee allocations"
    );
}

#[test]
fn required_source_coverage_survives_stripped_options_and_removed_descriptors() {
    let original = compile(SOURCE, FORWARDED).unwrap();
    for target in 0..4 {
        let mut stripped = original.clone();
        let file = stripped
            .file
            .iter_mut()
            .find(|file| file.name.as_deref() == Some("options.proto"))
            .unwrap();
        let unknown = match target {
            0 => file.message_type[0]
                .options
                .as_option_mut()
                .unwrap()
                .unknown_fields_mut(),
            1 => file.message_type[0].field[0]
                .options
                .as_option_mut()
                .unwrap()
                .unknown_fields_mut(),
            2 => file.service[0].method[0]
                .options
                .as_option_mut()
                .unwrap()
                .unknown_fields_mut(),
            _ => file.message_type[0].nested_type[0].field[0]
                .options
                .as_option_mut()
                .unwrap()
                .unknown_fields_mut(),
        };
        unknown.retain(|field| field.number != 51065);
        let assessment = check_descriptors(&stripped, "options.proto", &POLICY, &[]).unwrap();
        assert_eq!(
            assessment.disposition,
            Disposition::Incompatible,
            "stripping selected options cannot remove the source obligation"
        );
        assert!(
            assessment
                .findings
                .iter()
                .any(|finding| finding.reason.contains("missing required")
                    && finding.file == "options.proto")
        );
    }
    for target in 0..3 {
        let mut removed = original.clone();
        let file = removed
            .file
            .iter_mut()
            .find(|file| file.name.as_deref() == Some("options.proto"))
            .unwrap();
        match target {
            0 => {
                file.message_type.clear();
                file.service.clear();
            }
            1 => file.message_type[0].nested_type[0].field.clear(),
            _ => file.service[0].method.clear(),
        }
        assert!(
            check_descriptors(&removed, "options.proto", &POLICY, &[]).is_err(),
            "removing selected descriptors is not vacuous compatibility"
        );
    }
    let inspection = CorePolicy {
        interpretations: &[("independent.v1.Request", 1)],
    };
    assert_eq!(
        check_descriptors(&original, "options.proto", &inspection, &[])
            .unwrap()
            .disposition,
        Disposition::Unknown
    );
    let input = || DescriptorInput {
        descriptors: &original,
        selected_file: "options.proto",
        policy: &inspection,
    };
    assert_eq!(
        compare_descriptors(input(), input(), &inspection, &[])
            .unwrap()
            .disposition,
        Disposition::Unknown
    );
}

#[test]
fn selected_unknown_core_payload_fields_are_retained_then_refused_with_source() {
    let mut world = compile(SOURCE, FORWARDED).unwrap();
    let encoder = OptionEncoder::new(&world.file).unwrap();
    let file = world
        .file
        .iter_mut()
        .find(|file| file.name.as_deref() == Some("options.proto"))
        .unwrap();
    let options = file.message_type[0].options.as_option_mut().unwrap();
    let mut metadata = encoder
        .decode_extension(options, "pse.contract.v1.message")
        .unwrap()
        .unwrap();
    let mut additional = Vec::new();
    buffa::UnknownField {
        number: 999,
        data: buffa::UnknownFieldData::Varint(1),
    }
    .write_to(&mut additional);
    metadata.merge(&additional).unwrap();
    options
        .unknown_fields_mut()
        .retain(|field| field.number != 51065);
    encoder
        .attach(options, "pse.contract.v1.message", metadata)
        .unwrap();
    let result = check_descriptors(&world, "options.proto", &POLICY, &[]).unwrap();
    assert_eq!(result.disposition, Disposition::Unknown);
    assert_eq!(result.findings[0].subject, "independent.v1.Request");
    let failure = &result.findings[0].reason;
    assert!(
        failure.contains("options.proto:independent.v1.Request"),
        "{failure}"
    );
    assert!(
        failure.contains("unknown selected metadata field 999"),
        "{failure}"
    );
}

#[test]
fn unknown_enum_occurrence_cannot_hide_beside_an_earlier_known_value() {
    let mut world = compile(NATIVE_FIELDS, FORWARDED).unwrap();
    let encoder = OptionEncoder::new(&world.file).unwrap();
    let options = world
        .file
        .iter_mut()
        .find(|file| file.name.as_deref() == Some("options.proto"))
        .unwrap()
        .message_type[0]
        .field[0]
        .options
        .as_option_mut()
        .unwrap();
    let metadata = encoder
        .decode_extension(options, "pse.contract.v1.field")
        .unwrap()
        .unwrap();
    let Some(Value::Message(contract)) = metadata.field_by_number(4) else {
        panic!("contract");
    };
    let Some(Value::Message(native_type)) = contract.field_by_number(1) else {
        panic!("type");
    };
    let mut native_type = native_type.clone();
    let mut raw = Vec::new();
    buffa::UnknownField {
        number: 1,
        data: buffa::UnknownFieldData::Varint(99),
    }
    .write_to(&mut raw);
    native_type.merge(&raw).unwrap();
    assert_eq!(
        native_type.field_by_number(1),
        Some(&Value::EnumNumber(6)),
        "the known scalar remains beside the retained future enum occurrence"
    );
    let contract = encoder
        .message(
            "pse.contract.v1.NativeFieldContract",
            &[("native_type", Value::Message(native_type))],
        )
        .unwrap();
    let metadata = encoder
        .message(
            "pse.contract.v1.FieldMetadata",
            &[
                ("metadata_version", Value::U32(2)),
                ("native_field", Value::String("value".into())),
                ("direction", Value::EnumNumber(3)),
                ("native_contract", Value::Message(contract)),
            ],
        )
        .unwrap();
    options
        .unknown_fields_mut()
        .retain(|field| field.number != 51065);
    encoder
        .attach(options, "pse.contract.v1.field", metadata)
        .unwrap();
    assert_eq!(
        check_descriptors(&world, "options.proto", &NativeFactsPolicy, &[])
            .unwrap()
            .disposition,
        Disposition::Unknown
    );
}

#[test]
fn native_shape_codec_preserves_declared_string_members_and_refuses_empty_alternatives() {
    use pse_contract::boundary_projection::Shape;
    let imports = compile(SOURCE, FORWARDED).unwrap();
    let encoder = OptionEncoder::new(&imports.file).unwrap();
    let shape = Shape::StringEnum(vec!["".into(), "present".into()]);
    let metadata = encoder
        .message(
            "pse.contract.v1.MessageMetadata",
            &[
                ("metadata_version", Value::U32(2)),
                (
                    "native_shape",
                    Value::Message(encoder.native_shape(&shape, "independent.v1").unwrap()),
                ),
            ],
        )
        .unwrap();
    assert_eq!(
        owner_shape::decode(&metadata).unwrap(),
        Some(shape),
        "the existing native Shape owner permits an explicit empty-string member"
    );
    let empty = encoder
        .message("pse.contract.v1.NativeAlternatives", &[])
        .unwrap();
    let shape = encoder
        .message(
            "pse.contract.v1.NativeOwnerShape",
            &[("alternatives", Value::Message(empty))],
        )
        .unwrap();
    let metadata = encoder
        .message(
            "pse.contract.v1.MessageMetadata",
            &[
                ("metadata_version", Value::U32(2)),
                ("native_shape", Value::Message(shape)),
            ],
        )
        .unwrap();
    assert!(owner_shape::decode(&metadata).is_err());
}

#[test]
fn typed_encoder_refuses_schema_drift_extendee_confusion_and_duplicate_options() {
    let imports = compile(SOURCE, FORWARDED).unwrap().file;
    let encoder = OptionEncoder::new(&imports).unwrap();
    assert!(
        encoder
            .message(
                "pse.contract.v1.FieldMetadata",
                &[("metadata_version", Value::Bool(true))]
            )
            .is_err()
    );
    let message = encoder
        .message(
            "pse.contract.v1.FieldMetadata",
            &[("metadata_version", Value::U32(1))],
        )
        .unwrap();
    let mut options = descriptor::MessageOptions::default();
    assert!(
        encoder
            .attach(&mut options, "pse.contract.v1.field", message.clone())
            .is_err()
    );
    let mut options = descriptor::FieldOptions::default();
    encoder
        .attach(&mut options, "pse.contract.v1.field", message.clone())
        .unwrap();
    assert!(
        encoder
            .attach(&mut options, "pse.contract.v1.field", message)
            .is_err()
    );
    let mut changed = imports;
    let schema = changed
        .iter_mut()
        .find(|file| file.name.as_deref() == Some(CORE_IMPORT))
        .unwrap();
    schema
        .enum_type
        .iter_mut()
        .find(|enumeration| enumeration.name.as_deref() == Some("Direction"))
        .unwrap()
        .value[1]
        .number = Some(9);
    assert!(OptionEncoder::new(&changed).is_err());
}

#[test]
fn native_metadata_errors_keep_original_domain_type_and_diagnostic_category() {
    #[derive(Debug, thiserror::Error, miette::Diagnostic)]
    #[error("original native owner refusal")]
    #[diagnostic(code("workflow.internal"))]
    struct OwnerRefusal;
    let error = MetadataError::domain("independent.quantity", OwnerRefusal)
        .at("source.proto", "Request.quantity");
    assert_eq!(
        miette::Diagnostic::code(&error).unwrap().to_string(),
        "workflow.internal"
    );
    assert!(error.to_string().contains("source.proto:Request.quantity"));
    let mut current: Option<&dyn std::error::Error> = Some(&error);
    let mut original = false;
    while let Some(cause) = current {
        original |= cause.downcast_ref::<OwnerRefusal>().is_some();
        current = cause.source();
    }
    assert!(
        original,
        "consumer context must not erase the original domain error"
    );
    let encoder = OptionEncoder::new(&compile(SOURCE, FORWARDED).unwrap().file).unwrap();
    let reflection = encoder
        .message(
            "pse.contract.v1.FieldMetadata",
            &[("metadata_version", Value::Bool(true))],
        )
        .unwrap_err();
    let mut current: Option<&dyn std::error::Error> = Some(&reflection);
    let mut original = false;
    while let Some(cause) = current {
        original |= cause
            .downcast_ref::<buffa_descriptor::ReflectError>()
            .is_some();
        current = cause.source();
    }
    assert!(
        original,
        "checked reflection keeps the original library failure"
    );
}
