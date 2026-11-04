// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;

#[test]
fn diagnostic_observations_and_performing_locators_project_original_data_without_authority() {
    use crate::diagnostic::{Observation, OperandContract, PhysicalObservation};
    use pse_diagnostics::observation::{OperationId, ProducerId, ProducerRole};
    let locator = ProducerId {
        operation: OperationId {
            process_instance: [7, 11],
            sequence: 13,
        },
        sequence: 17,
    };
    let restored = ProducerId::from_projection_fields(locator.into_projection_fields()).unwrap();
    assert_eq!(restored, locator);
    let graph = ProjectionGraph::of::<ProducerId>().unwrap();
    assert_eq!(
        graph.definitions()["DiagnosticProducerId"]
            .public_rust_path
            .as_deref(),
        Some("pse_diagnostics::observation::ProducerId")
    );
    assert_eq!(
        graph.definitions()["DiagnosticOperationId"].direction,
        Direction::Output
    );
    let role = ProducerRole::NativeFactor;
    assert_eq!(
        <ProducerRole as ProjectionAlternative<8>>::from_projection_alternative(()),
        role
    );
    let contract = OperandContract {
        quantity: [3; 16],
        indices: vec![[[4; 16], [5; 16], [6; 16]]],
    };
    let value = Observation::Contracts(vec![contract]);
    let bytes = serde_json::to_vec(&value).unwrap();
    let fields =
        <Observation as ProjectionAlternative<8>>::into_projection_alternative(value).unwrap();
    let restored = <Observation as ProjectionAlternative<8>>::from_projection_alternative(fields);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    assert!(Observation::Real(f64::NAN).validate_projection().is_err());
    assert!(serde_json::to_vec(&Observation::Real(f64::NAN)).is_err());
    let value = Observation::Physical(PhysicalObservation {
        magnitude: -0.0,
        quantity: pse_ids::SemanticId::from_bytes([1; 16]),
        unit: pse_ids::SemanticId::from_bytes([2; 16]),
        context: pse_ids::ContentHash::from_bytes([3; 32]),
    });
    value.validate_projection().unwrap();
    let fields =
        <Observation as ProjectionAlternative<4>>::into_projection_alternative(value).unwrap();
    assert_eq!(fields.0.magnitude.to_bits(), (-0.0_f64).to_bits());
    let graph = ProjectionGraph::of::<Observation>().unwrap();
    assert!(
        graph
            .definitions()
            .contains_key("DiagnosticOperandContract")
    );
    assert!(graph.definitions().contains_key("Array3Array16U8"));
}

#[test]
fn named_tuple_records_and_multi_payload_alternatives_keep_original_native_json() {
    crate::projection_tuple_struct! {
        #[derive(Debug, PartialEq)]
        struct Move {
            projection: { name: "SourceMove", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            path: String => { tag: 7, direction: Direction::Both, rules: [] };
            input: usize => { tag: 2, direction: Direction::Both, rules: [] };
        }
    }
    let value = Move {
        path: "u".into(),
        input: 3,
    };
    assert_eq!(
        serde_json::to_vec(&value).unwrap(),
        serde_json::to_vec(&("u", 3usize)).unwrap()
    );
    assert_eq!(serde_json::from_str::<Move>(r#"["u",3]"#).unwrap(), value);
    assert!(serde_json::from_str::<Move>(r#"["u",3,4]"#).is_err());
    assert!(serde_json::from_str::<Move>(r#"{"path":"u","input":3}"#).is_err());
    assert_eq!(Move::FIELD_LAYOUT, &[(7, "path"), (2, "input")]);
    let mut record = serde_json::to_value(schemars::schema_for!(Move)).unwrap();
    let mut tuple = serde_json::to_value(schemars::schema_for!((String, usize))).unwrap();
    record.as_object_mut().unwrap().remove("title");
    tuple.as_object_mut().unwrap().remove("title");
    assert_eq!(record, tuple);

    crate::projection_tuple_enum! {
        #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        enum Expression {
            projection: { name: "TupleExpression", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            Variable [name: String => { tag: 1, direction: Direction::Both, rules: [] };] => 1;
            Product [left: Box<Expression> => { tag: 8, direction: Direction::Both, rules: [] };
                right: Box<Expression> => { tag: 2, direction: Direction::Both, rules: [] };] => 4;
            Unit => 9;
        }
    }
    let value = Expression::Product(
        Box::new(Expression::Variable("a".into())),
        Box::new(Expression::Variable("b".into())),
    );
    let json = r#"{"Product":[{"Variable":"a"},{"Variable":"b"}]}"#;
    assert_eq!(serde_json::to_string(&value).unwrap(), json);
    assert_eq!(serde_json::from_str::<Expression>(json).unwrap(), value);
    let fields =
        <Expression as ProjectionAlternative<4>>::into_projection_alternative(value.clone())
            .unwrap();
    assert_eq!(
        <Expression as ProjectionAlternative<4>>::from_projection_alternative(fields),
        value
    );
    assert_eq!(
        <Expression as ProjectionAlternative<4>>::FIELD_LAYOUT,
        &[(8, "left"), (2, "right")]
    );
    let graph = ProjectionGraph::of::<Expression>().unwrap();
    let Shape::Alternatives(alternatives) = &graph.definitions()["TupleExpression"].shape else {
        panic!("tuple expression")
    };
    assert_eq!(
        alternatives[1]
            .fields
            .iter()
            .map(|field| field.tag)
            .collect::<Vec<_>>(),
        vec![8, 2]
    );
}

#[test]
fn tuple_payload_attributes_rules_and_unprojected_native_states_are_source_owned() {
    fn finite<S: serde::Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
        if value.is_finite() {
            serializer.serialize_f64(*value)
        } else {
            Err(serde::ser::Error::custom(
                "finite source observation required",
            ))
        }
    }
    crate::projection_enum! {
        #[derive(Debug, Default, serde::Serialize)]
        #[serde(tag = "kind", content = "value", rename_all = "snake_case")]
        enum NativeSettings {
            projection: { name: "NativeSettings", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            @unprojected { #[default] #[serde(skip)] Default; }
            Real(#[serde(serialize_with = "finite")] f64 => { rules: [LocalRule::Float64Domain(Float64Domain::Finite)] }) => 3;
        }
    }
    assert!(NativeSettings::Default.validate_projection().is_err());
    assert!(serde_json::to_string(&NativeSettings::Default).is_err());
    assert!(
        <NativeSettings as ProjectionAlternative<3>>::into_projection_alternative(
            NativeSettings::Default
        )
        .is_err()
    );
    assert!(
        NativeSettings::Real(f64::INFINITY)
            .validate_projection()
            .is_err()
    );
    assert!(serde_json::to_string(&NativeSettings::Real(f64::INFINITY)).is_err());
    assert_eq!(
        serde_json::to_string(&NativeSettings::Real(-0.0)).unwrap(),
        r#"{"kind":"real","value":-0.0}"#
    );
    let graph = ProjectionGraph::of::<NativeSettings>().unwrap();
    let Shape::Alternatives(alternatives) = &graph.definitions()["NativeSettings"].shape else {
        panic!("source alternatives")
    };
    assert_eq!(alternatives.len(), 1);
    assert_eq!(alternatives[0].tag, 3);
    assert_eq!(
        alternatives[0].fields[0].rules,
        vec![LocalRule::Float64Domain(Float64Domain::Finite)]
    );
}

#[test]
fn remote_enum_binding_reuses_actual_variants_and_preserves_both_existing_json_forms() {
    crate::projection_enum! {
        @serde_remote {
            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(remote = "Signal", tag = "kind", content = "value", rename_all = "snake_case", deny_unknown_fields)]
            enum SignalDocument;
        }
        #[derive(Clone, Debug, PartialEq, serde::Serialize)]
        enum Signal {
            projection: { name: "OriginalSignal", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            Constant(String) => 1;
            Periods(Vec<String>) => 2;
        }
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Request {
        #[serde(with = "SignalDocument")]
        signal: Signal,
    }
    let native = Signal::Constant("u".into());
    assert_eq!(
        serde_json::to_string(&native).unwrap(),
        r#"{"Constant":"u"}"#
    );
    let bytes = r#"{"signal":{"kind":"constant","value":"u"}}"#;
    assert_eq!(
        serde_json::to_string(&Request {
            signal: native.clone()
        })
        .unwrap(),
        bytes
    );
    assert_eq!(
        serde_json::from_str::<Request>(bytes).unwrap().signal,
        native
    );
    let native = Signal::Periods(vec!["u0".into(), "u1".into()]);
    let bytes = serde_json::to_string(&Request {
        signal: native.clone(),
    })
    .unwrap();
    assert_eq!(
        serde_json::from_str::<Request>(&bytes).unwrap().signal,
        native
    );
    let graph = ProjectionGraph::of::<Signal>().unwrap();
    let Shape::Alternatives(alternatives) = &graph.definitions()["OriginalSignal"].shape else {
        panic!("native alternatives")
    };
    assert_eq!(
        alternatives
            .iter()
            .map(|alternative| (alternative.name.as_str(), alternative.tag))
            .collect::<Vec<_>>(),
        vec![("Constant", 1), ("Periods", 2)]
    );
}

crate::projection_struct! {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct CapturedProjectionFixture<T = u64> {
        projection: { name: "CapturedProjectionFixtureFor", rust_path: crate::boundary_projection::tests::CapturedProjectionFixture,
            version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        position: u64 => { tag: 1, direction: Direction::Both, rules: [] };
        value: T => { tag: 2, direction: Direction::Both, rules: [] };
    }
}

#[test]
fn generic_source_records_preserve_native_argument_identity_and_access() {
    let native = CapturedProjectionFixture {
        position: 4,
        value: 27usize,
    };
    assert_eq!(
        serde_json::to_string(&native).unwrap(),
        r#"{"position":4,"value":27}"#
    );
    let restored = CapturedProjectionFixture::<usize>::from_projection_fields(
        native.clone().into_projection_fields(),
    )
    .unwrap();
    assert_eq!(restored, native);
    let mut graph = ProjectionGraph::of::<CapturedProjectionFixture<usize>>().unwrap();
    CapturedProjectionFixture::<u64>::projection(&mut graph).unwrap();
    CapturedProjectionFixture::<Box<u64>>::projection(&mut graph).unwrap();
    CapturedProjectionFixture::<Vec<usize>>::projection(&mut graph).unwrap();
    for (argument, path) in [
        ("Usize", "usize"),
        ("U64", "u64"),
        ("BoxedU64", "std::boxed::Box<u64>"),
        ("ListUsize", "std::vec::Vec<usize>"),
    ] {
        let definition = &graph.definitions()[&format!("CapturedProjectionFixtureFor{argument}")];
        assert_eq!(
            definition.public_rust_path.as_deref(),
            Some(
                format!("pse_model::boundary_projection::tests::CapturedProjectionFixture<{path}>")
                    .as_str()
            )
        );
    }
}

#[test]
fn actual_study_point_facts_keep_nominal_occurrence_and_source_enum_meaning() {
    use crate::study::{EffectState, OccurrenceKey, PointFacts, ScientificFacts};
    let facts = PointFacts {
        admitted_seed_need: None,
        key: OccurrenceKey(7),
        revision: 19,
        lifecycle: crate::generated::enums::StudyPointState::ALL[0],
        native_started: false,
        scientific: ScientificFacts::default(),
        attempt_count: 3,
        retry_failure: None,
        effect: EffectState::ALL[0],
        seed: None,
    };
    assert_eq!(serde_json::to_string(&facts.key).unwrap(), "7");
    let restored =
        PointFacts::from_projection_fields(facts.clone().into_projection_fields()).unwrap();
    assert_eq!(restored, facts);
    let graph = ProjectionGraph::of::<PointFacts>().unwrap();
    let Shape::Record(fields) = &graph.definitions()["StudyPointFacts"].shape else {
        panic!("actual fact record")
    };
    assert_eq!(fields.len(), 10);
    assert_eq!(fields[1].ty, TypeRef::Named("StudyOccurrenceKey".into()));
    let Shape::StringEnum(members) = &graph.definitions()["StudyPointState"].shape else {
        panic!("source vocabulary")
    };
    assert_eq!(
        members,
        &crate::generated::enums::StudyPointState::ALL
            .iter()
            .map(|state| state.as_str().to_owned())
            .collect::<Vec<_>>()
    );
}

#[test]
fn foreign_remote_binding_uses_one_field_table_and_checked_shared_access() {
    use super::{Adapted, CheckedProjectionRecord, ProjectionAdapter, SharedAdapter};
    type NativeRangeAlias = std::ops::Range<u16>;
    crate::projection_remote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(remote = "std::ops::Range<u16>", deny_unknown_fields)]
        struct RangeBinding for std::ops::Range<u16> {
            projection: { name: "BoundedNativeRange", rust_path: RangeBinding, target_path: NativeRangeAlias, version: 2,
                direction: Direction::Input, reserved_tags: [4], reserved_names: ["retired"] };
            start: u16 => { tag: 1, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: 16 }] };
            end: u16 => { tag: 3, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: 32 }] };
        }
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Document {
        #[serde(with = "RangeBinding")]
        range: std::ops::Range<u16>,
    }
    crate::projection_struct! {
        struct NativeOwner {
            projection: { name: "NativeRangeOwner", version: 1, direction: Direction::Both,
                reserved_tags: [], reserved_names: [] };
            range: std::ops::Range<u16> => { tag: 1, direction: Direction::Both, rules: [], adapter: RangeBinding };
        }
    }
    type Bound = Adapted<std::ops::Range<u16>, RangeBinding>;
    type Shared = Adapted<std::sync::Arc<std::ops::Range<u16>>, SharedAdapter<RangeBinding>>;
    let native = 2..17;
    let json = serde_json::to_string(&Document {
        range: native.clone(),
    })
    .unwrap();
    let restored: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.range, native);
    let fields = Bound::new(native.clone())
        .try_into_projection_fields()
        .unwrap();
    assert_eq!(fields, (2, 17));
    assert_eq!(
        Bound::from_projection_fields(fields).unwrap().into_inner(),
        native
    );
    assert!(Bound::new(0..17).try_into_projection_fields().is_err());
    assert!(Bound::from_projection_fields((2, 33)).is_err());
    let owner = NativeOwner { range: 0..17 };
    assert!(owner.validate_projection().is_err());
    let graph = ProjectionGraph::of::<NativeOwner>().unwrap();
    let binding = &graph.definitions()["BoundedNativeRange"];
    assert!(binding.checked_record);
    assert!(
        binding
            .public_rust_path
            .as_ref()
            .unwrap()
            .contains("Adapted<NativeRangeAlias,RangeBinding>")
    );
    assert_eq!(binding.direction, Direction::Input);
    assert_eq!(binding.reserved_tags, vec![4]);
    assert_eq!(binding.reserved_names, vec!["retired"]);
    assert_eq!(RangeBinding::FIELD_LAYOUT, &[(1, "start"), (3, "end")]);
    let shared = std::sync::Arc::new(native.clone());
    let extracted = Shared::new(shared.clone())
        .try_into_projection_fields()
        .unwrap();
    assert_eq!(std::sync::Arc::strong_count(&shared), 1);
    let restored = Shared::from_projection_fields(extracted)
        .unwrap()
        .into_inner();
    assert_eq!(*restored, native);
    let graph = ProjectionGraph::of::<Shared>().unwrap();
    assert!(graph.definitions()["SharedBoundedNativeRange"].checked_record);
    for (scalar, maximum) in [
        (Scalar::U8, 256),
        (Scalar::U16, 65_536),
        (Scalar::U32, u64::from(u32::MAX) + 1),
    ] {
        let mut graph = ProjectionGraph::default();
        assert!(
            graph
                .declare::<RangeBinding>(
                    "InvalidNativeWidth",
                    file!(),
                    1,
                    Direction::Both,
                    &[],
                    &[],
                    |_| {
                        Ok(Shape::Record(vec![Field {
                            name: "value".into(),
                            tag: 1,
                            ty: TypeRef::Scalar(scalar),
                            direction: Direction::Both,
                            rules: vec![LocalRule::UnsignedRange {
                                min: 0,
                                max: maximum,
                            }],
                            named_facets: Default::default(),
                        }]))
                    }
                )
                .is_err()
        );
    }
}

#[test]
fn standard_version_duration_and_hash_roles_preserve_exact_native_leaves() {
    use crate::document::Version;
    let graph = ProjectionGraph::of::<Version<8>>().unwrap();
    let Shape::Record(fields) = &graph.definitions()["DocumentVersion8"].shape else {
        panic!("version record")
    };
    assert_eq!(
        fields[0].rules,
        [LocalRule::UnsignedRange { min: 8, max: 8 }]
    );
    assert_eq!(Version::<8>.into_projection_fields(), (8,));
    assert_eq!(
        Version::<8>::from_projection_fields((8,)).unwrap(),
        Version::<8>
    );
    for unsupported in [0, 7, 9, u32::MAX] {
        assert!(Version::<8>::from_projection_fields((unsupported,)).is_err());
    }
    let edge = std::time::Duration::new(u64::MAX, 999_999_999);
    assert_eq!(edge.into_projection_fields(), (u64::MAX, 999_999_999));
    assert_eq!(
        std::time::Duration::from_projection_fields(edge.into_projection_fields()).unwrap(),
        edge
    );
    for nanos in [1_000_000_000, u32::MAX] {
        assert!(
            std::time::Duration::from_projection_fields((u64::MAX, nanos)).is_err(),
            "malformed nanos refuse before normalization/overflow"
        );
    }
    let graph = ProjectionGraph::of::<std::time::Duration>().unwrap();
    let Shape::Record(fields) = &graph.definitions()["NonnegativeDuration"].shape else {
        panic!("duration")
    };
    assert_eq!(
        fields[1].rules,
        [LocalRule::UnsignedRange {
            min: 0,
            max: 999_999_999
        }]
    );
    let hash = pse_ids::ContentHash::from_bytes([0x42; 32]);
    let revision = pse_ids::roles::SourceRevisionHash::from_projection_fields((hash,)).unwrap();
    let binding = pse_ids::roles::BindingHash::from_projection_fields((hash,)).unwrap();
    assert_eq!(revision.into_projection_fields(), (hash,));
    assert_eq!(binding.into_projection_fields(), (hash,));
    let mut graph = ProjectionGraph::of::<pse_ids::roles::SourceRevisionHash>().unwrap();
    graph.register::<pse_ids::roles::BindingHash>().unwrap();
    assert_eq!(
        graph.definitions().len(),
        2,
        "equal bytes never erase different nominal roles"
    );
}

#[test]
fn record_local_fields_use_explicit_projection_checks_and_original_default() {
    fn no_detached_proof(
        value: &Option<std::sync::Arc<()>>,
        path: &str,
    ) -> Result<(), ValidationError> {
        if value.is_none() {
            Ok(())
        } else {
            Err(ValidationError::Rule {
                path: path.into(),
                rule: "local_proof",
                reason: "local evidence requires its original owner reconciliation".into(),
            })
        }
    }
    projection_struct! {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct OwnedInput {
            projection: { name: "OwnedInput", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            value: u16 => { tag: 1, direction: Direction::Both, rules: [] };
            @local {
                #[serde(skip)]
                proof: Option<std::sync::Arc<()>> = None => check: no_detached_proof;
            }
        }
    }
    let graph = ProjectionGraph::of::<OwnedInput>().unwrap();
    let Shape::Record(fields) = &graph.definitions()["OwnedInput"].shape else {
        panic!("actual record")
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].ty, TypeRef::Scalar(Scalar::U16));
    let decoded: OwnedInput = serde_json::from_str(r#"{"value":65535}"#).unwrap();
    let reconstructed =
        OwnedInput::from_projection_fields(decoded.into_projection_fields()).unwrap();
    assert_eq!(reconstructed.value, u16::MAX);
    assert!(reconstructed.proof.is_none());
    reconstructed.validate_projection().unwrap();
    let with_proof = OwnedInput {
        value: 1,
        proof: Some(std::sync::Arc::new(())),
    };
    assert!(
        matches!(with_proof.validate_projection(), Err(ValidationError::Rule { path, rule: "local_proof", .. }) if path == "proof")
    );
}

#[test]
fn optional_unsigned_rules_constrain_present_values_without_requiring_a_deadline() {
    projection_struct! {
        struct OptionalDeadline {
            projection: { name: "OptionalDeadline", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
            remaining_ns: Option<u64> => { tag: 10, direction: Direction::Input,
                rules: [LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
        }
    }
    let graph = ProjectionGraph::of::<OptionalDeadline>().unwrap();
    let Shape::Record(fields) = &graph.definitions()["OptionalDeadline"].shape else {
        panic!("actual optional scalar owner")
    };
    assert_eq!(
        fields[0].ty,
        TypeRef::Optional(Box::new(TypeRef::Scalar(Scalar::U64)))
    );
    for remaining_ns in [None, Some(1), Some(u64::MAX)] {
        OptionalDeadline { remaining_ns }
            .validate_projection()
            .unwrap();
    }
    assert!(
        OptionalDeadline {
            remaining_ns: Some(0)
        }
        .validate_projection()
        .is_err()
    );
    let range = LocalRule::UnsignedRange { min: 1, max: 10 };
    assert!(validate_field(&None::<u32>, "required", &[range, LocalRule::Required]).is_err());
    assert!(validate_field(&Some(11_u32), "required", &[LocalRule::Required, range]).is_err());
    validate_field(&Some(10_u32), "required", &[LocalRule::Required, range]).unwrap();
    validate_field(&Some(Box::new(10_u64)), "boxed", &[range]).unwrap();
    assert!(validate_field(&Some(Box::new(11_u64)), "boxed", &[range]).is_err());
}

projection_enum! {
    #[derive(Debug)]
    enum Recursive {
        projection: { name: "Recursive", version: 1, direction: Direction::Both, reserved_tags: [9], reserved_names: ["Removed"] };
        Missing => 1;
        Number(f64) => 2;
        Rows(Vec<Recursive>) => 4;
        Record {
            id: pse_ids::SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
            next: Option<Box<Recursive>> => { tag: 2, direction: Direction::Both, rules: [] };
        } => 7;
    }
}

projection_struct! {
    #[derive(Debug)]
    struct Request {
        projection: { name: "Request", version: 2, direction: Direction::Input, reserved_tags: [3], reserved_names: ["old"] };
        limit: u32 => { tag: 7, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: 128 }] };
        nested: Option<Vec<Option<Recursive>>> => { tag: 2, direction: Direction::Input, rules: [] };
    }
}

#[test]
fn actual_declarations_resolve_recursive_fields_and_explicit_identity() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    assert_eq!(graph.definitions().len(), 2);
    let request = &graph.definitions()["Request"];
    assert_eq!(request.version, 2);
    assert_eq!(request.reserved_tags, [3]);
    let Shape::Record(fields) = &request.shape else {
        panic!("record")
    };
    assert_eq!(fields[0].tag, 7);
    assert_eq!(
        fields[1].ty,
        TypeRef::Optional(Box::new(TypeRef::Repeated(Box::new(TypeRef::Optional(
            Box::new(TypeRef::Named("Recursive".into()))
        )))))
    );
    let Shape::Alternatives(alternatives) = &graph.definitions()["Recursive"].shape else {
        panic!("enum")
    };
    assert_eq!(
        alternatives[1].fields[0].ty,
        TypeRef::Scalar(Scalar::F64Bits)
    );
    assert_eq!(
        alternatives[3].fields[0].ty,
        TypeRef::Scalar(Scalar::SemanticId)
    );
    // These constructors are the actual Rust owner, not a parallel metadata catalogue.
    let value = Request {
        limit: 128,
        nested: Some(vec![Some(Recursive::Rows(vec![
            Recursive::Number(-0.0),
            Recursive::Missing,
            Recursive::Record {
                id: pse_ids::SemanticId::from_bytes([0; 16]),
                next: None,
            },
        ]))]),
    };
    assert_eq!(value.limit, 128);
    assert_eq!(value.nested.as_ref().unwrap().len(), 1);
}

#[test]
fn duplicate_reserved_and_invalid_tags_refuse() {
    for (tags, reserved) in [
        (vec![1, 1], vec![]),
        (vec![1, 2], vec![2]),
        (vec![0, 2], vec![]),
        (vec![19_000, 2], vec![]),
    ] {
        let mut graph = ProjectionGraph::default();
        let result = graph.declare::<Request>(
            "Broken",
            file!(),
            1,
            Direction::Both,
            &reserved,
            &[],
            |_| {
                Ok(Shape::Record(
                    tags.iter()
                        .enumerate()
                        .map(|(i, tag)| Field {
                            name: format!("f{i}"),
                            tag: *tag,
                            ty: TypeRef::Scalar(Scalar::U32),
                            direction: Direction::Both,
                            rules: vec![],
                            named_facets: Default::default(),
                        })
                        .collect(),
                ))
            },
        );
        assert!(result.is_err(), "{tags:?}/{reserved:?}");
    }
}

#[test]
fn public_name_collisions_and_wrong_type_rules_refuse() {
    let mut graph = ProjectionGraph::of::<Request>().unwrap();
    let collision =
        graph.declare::<Recursive>("Request", file!(), 1, Direction::Both, &[], &[], |_| {
            Ok(Shape::Record(vec![]))
        });
    assert!(collision.is_err());
    let bad_rule =
        graph.declare::<Recursive>("BadRule", file!(), 1, Direction::Both, &[], &[], |_| {
            Ok(Shape::Record(vec![Field {
                name: "number".into(),
                tag: 1,
                ty: TypeRef::Scalar(Scalar::F64Bits),
                direction: Direction::Both,
                rules: vec![LocalRule::UnsignedRange { min: 0, max: 1 }],
                named_facets: Default::default(),
            }]))
        });
    assert!(bad_rule.is_err());
}

#[test]
fn multiple_roots_share_one_complete_recursive_world() {
    let mut graph = ProjectionGraph::of::<Request>().unwrap();
    graph.register::<Recursive>().unwrap();
    graph.register::<Request>().unwrap();
    assert_eq!(graph.roots().len(), 2);
    assert_eq!(graph.definitions().len(), 2);
}

#[test]
fn one_owner_rule_validates_native_values_with_exact_field_paths() {
    let invalid = Request {
        limit: 0,
        nested: None,
    };
    assert_eq!(
        invalid.validate_projection(),
        Err(ValidationError::Rule {
            path: "limit".into(),
            rule: "unsigned.range",
            reason: "expected inclusive range 1..=128".into(),
        })
    );
    Request {
        limit: 128,
        nested: Some(vec![Some(Recursive::Missing)]),
    }
    .validate_projection()
    .unwrap();
    assert!(
        validate_field(
            &u64::MAX,
            "integer",
            &[LocalRule::UnsignedRange {
                min: u64::MAX,
                max: u64::MAX
            }]
        )
        .is_ok()
    );
    assert!(
        validate_field(
            &u64::MAX,
            "integer",
            &[LocalRule::UnsignedRange {
                min: 0,
                max: u64::MAX - 1
            }]
        )
        .is_err()
    );
    validate_field(
        &"é𐐷".to_owned(),
        "text",
        &[LocalRule::Length { min: 2, max: 2 }],
    )
    .unwrap();
    assert!(validate_field(&None::<u32>, "optional", &[LocalRule::Required]).is_err());
}

#[test]
fn typed_access_preserves_private_fields_and_explicit_alternative_tags() {
    let request = Request {
        limit: 17,
        nested: Some(vec![Some(Recursive::Number(-0.0))]),
    };
    assert_eq!(
        <Request as ProjectionRecord>::FIELD_LAYOUT,
        &[(7, "limit"), (2, "nested")]
    );
    let (limit, nested) = request.into_projection_fields();
    let request = Request::from_projection_fields((limit, nested)).unwrap();
    assert_eq!(request.limit, 17);
    let alternative = request.nested.unwrap().remove(0).unwrap();
    let (number,) =
        <Recursive as ProjectionAlternative<2>>::into_projection_alternative(alternative).unwrap();
    assert_eq!(number.to_bits(), 0x8000_0000_0000_0000);
    let missing = <Recursive as ProjectionAlternative<1>>::from_projection_alternative(());
    assert!(<Recursive as ProjectionAlternative<2>>::into_projection_alternative(missing).is_err());
}

#[test]
fn quantity_callbacks_follow_actual_fields_and_checked_canonical_components() {
    let graph = ProjectionGraph::of::<pse_quantity::QuantityType>().unwrap();
    assert_eq!(
        <pse_quantity::QuantityType as ProjectionRecord>::FIELD_LAYOUT,
        &[
            (1, "id"),
            (2, "name"),
            (3, "key"),
            (4, "canonical_unit"),
            (5, "nominal_magnitude")
        ]
    );
    assert!(graph.definitions().contains_key("QuantityTypeKey"));
    let ratio = <pse_quantity::Ratio as ProjectionRecord>::from_projection_fields((1, 2)).unwrap();
    assert_eq!(ratio, pse_quantity::Ratio::new(1, 2).unwrap());
    for components in [(2, 4), (1, 0), (1, -2), (0, 2)] {
        assert!(
            <pse_quantity::Ratio as ProjectionRecord>::from_projection_fields(components).is_err(),
            "unreduced/negative/zero denominator must not normalize at the boundary"
        );
    }
    let graph = ProjectionGraph::of::<pse_quantity::DimensionVector>().unwrap();
    let Shape::Record(fields) = &graph.definitions()["Array8PhysicalRatio"].shape else {
        panic!("fixed array wrapper")
    };
    assert_eq!(fields[0].rules, [LocalRule::Length { min: 8, max: 8 }]);
    assert!(
        <[pse_quantity::Ratio; 8] as ProjectionRecord>::from_projection_fields((vec![ratio; 7],))
            .is_err()
    );
    assert_eq!(
        <[pse_quantity::Ratio; 8] as ProjectionRecord>::from_projection_fields((vec![ratio; 8],))
            .unwrap(),
        [ratio; 8]
    );
    let graph = ProjectionGraph::of::<pse_quantity::Unit>().unwrap();
    assert!(graph.definitions().contains_key("PhysicalDimensionVector"));
    assert!(graph.definitions().contains_key("PhysicalRatio"));
    assert!(graph.definitions().contains_key("ReferenceStateId"));
}

#[test]
fn scalar_containers_skip_recursive_walk_but_keep_field_extent_and_owner_predicates() {
    use std::cell::Cell;
    struct Leaf<'a>(&'a Cell<usize>);
    impl ValidateProjection for Leaf<'_> {
        const NEEDS_RECURSIVE_VALIDATION: bool = false;
        fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }
    struct Owner<'a>(&'a Cell<usize>);
    impl ValidateProjection for Owner<'_> {
        fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }
    let count = Cell::new(0);
    let leaves = vec![Leaf(&count), Leaf(&count)];
    validate_field(&leaves, "bytes", &[LocalRule::Length { min: 2, max: 2 }]).unwrap();
    assert_eq!(
        count.get(),
        0,
        "leaf promises avoid per-element traversal/path allocation"
    );
    assert!(validate_field(&leaves, "bytes", &[LocalRule::Length { min: 0, max: 1 }]).is_err());
    vec![Owner(&count), Owner(&count)]
        .validate_projection()
        .unwrap();
    assert_eq!(
        count.get(),
        2,
        "custom native owner remains conservative by default"
    );
    assert!(
        validate_field(
            &"é".to_owned(),
            "name",
            &[LocalRule::Utf8Bytes { min: 0, max: 1 }]
        )
        .is_err()
    );
    validate_field(
        &"é".to_owned(),
        "name",
        &[LocalRule::Length { min: 1, max: 1 }],
    )
    .unwrap();
}

#[test]
fn explicitly_local_payload_custody_is_excluded_from_the_wire_layout() {
    projection_enum! {
        #[derive(Debug)]
        enum Retained {
            projection: { name: "Retained", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
            Page {
                bytes: Vec<u8> => { tag: 3, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 32 }] };
                @local { custody: Option<std::sync::Arc<()>> = None; }
            } => 1;
        }
    }
    let owner = std::sync::Arc::new(());
    let native = Retained::Page {
        bytes: vec![1, 2],
        custody: Some(owner.clone()),
    };
    let Retained::Page { custody, .. } = &native;
    assert!(custody.is_some());
    assert_eq!(std::sync::Arc::strong_count(&owner), 2);
    let fields =
        <Retained as ProjectionAlternative<1>>::into_projection_alternative(native).unwrap();
    assert_eq!(fields, (vec![1, 2],));
    assert_eq!(std::sync::Arc::strong_count(&owner), 1);
    let restored = <Retained as ProjectionAlternative<1>>::from_projection_alternative(fields);
    let Retained::Page { custody, .. } = restored;
    assert!(
        custody.is_none(),
        "native allocation authority cannot be reconstructed from wire data"
    );
    let graph = ProjectionGraph::of::<Retained>().unwrap();
    let Shape::Alternatives(alternatives) = &graph.definitions()["Retained"].shape else {
        panic!("owner enum")
    };
    assert_eq!(alternatives[0].fields.len(), 1);
    assert_eq!(alternatives[0].fields[0].tag, 3);
}

#[test]
fn method_effects_are_explicit_original_facts_and_cannot_be_annotated_twice() {
    use crate::operation_effects::EffectFacts;
    assert!(
        Service::new("Selected")
            .effects(EffectFacts::OBSERVATION)
            .is_err()
    );
    let mut graph = ProjectionGraph::default();
    let service = Service::new("Selected")
        .unary::<Request, Recursive>(&mut graph, "Inspect")
        .unwrap();
    assert_eq!(service.methods[0].effects, None);
    let service = service.effects(EffectFacts::OBSERVATION).unwrap();
    assert_eq!(service.methods[0].effects, Some(EffectFacts::OBSERVATION));
    assert!(
        service
            .clone()
            .effects(EffectFacts::CANONICAL_COMMAND)
            .is_err()
    );
    graph.service(service).unwrap();
    assert!(
        graph.services()["Selected"].methods[0]
            .effects
            .unwrap()
            .read_only
    );
}

#[test]
fn overlapping_rule_families_and_unrepresentable_collection_presence_refuse() {
    for (ty, rules) in [
        (
            TypeRef::Scalar(Scalar::U64),
            vec![
                LocalRule::UnsignedRange { min: 1, max: 5 },
                LocalRule::UnsignedRange { min: 2, max: 7 },
            ],
        ),
        (
            TypeRef::Repeated(Box::new(TypeRef::Scalar(Scalar::U64))),
            vec![LocalRule::Required],
        ),
    ] {
        let mut graph = ProjectionGraph::default();
        assert!(
            graph
                .declare::<Request>(
                    "UnsupportedRules",
                    file!(),
                    1,
                    Direction::Both,
                    &[],
                    &[],
                    |_| Ok(Shape::Record(vec![Field {
                        name: "value".into(),
                        tag: 1,
                        ty,
                        direction: Direction::Both,
                        rules,
                        named_facets: Default::default()
                    },]))
                )
                .is_err()
        );
    }
}

#[test]
fn signed_domains_preserve_extremes_and_optional_presence_without_float_conversion() {
    let rule = [LocalRule::SignedRange {
        min: i64::MIN,
        max: -1,
    }];
    assert!(validate_field(&i64::MIN, "native.signed", &rule).is_ok());
    assert!(validate_field(&-1_i64, "native.signed", &rule).is_ok());
    assert!(validate_field(&0_i64, "native.signed", &rule).is_err());
    assert!(validate_field(&None::<i64>, "native.optional", &rule).is_ok());
    assert!(validate_field(&Some(i64::MIN), "native.optional", &rule).is_ok());
    assert!(validate_field(&Some(0_i64), "native.optional", &rule).is_err());
    let mut graph = ProjectionGraph::default();
    assert!(
        graph
            .declare::<i16>(
                "InvalidWidth",
                file!(),
                1,
                Direction::Both,
                &[],
                &[],
                |graph| {
                    Ok(Shape::Record(vec![Field::of::<i16>(
                        graph,
                        "value",
                        1,
                        Direction::Both,
                        vec![LocalRule::SignedRange {
                            min: i64::from(i16::MIN) - 1,
                            max: 0,
                        }],
                    )?]))
                }
            )
            .is_err()
    );
}
