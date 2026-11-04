// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
const POLICY: CorePolicy<'_> = CorePolicy {
    interpretations: &[("pse.test.v1.Request", 1)],
};

const OWNER: Owner<'_> = Owner {
    rust_type: "native::Request",
    declaration_file: "src/request.rs",
};
const META: MessageMetadata<'_> = MessageMetadata {
    metadata_version: Some(VERSION),
    owner: Some(OWNER),
    interpretation_version: Some(1),
    direction: Some(EnumValue::Known(BoundaryDirection::Both)),
    construction: Some(EnumValue::Known(ConstructionRole::Data)),
    effects: None,
    native_shape: Some(&crate::boundary_projection::Shape::Record(Vec::new())),
};
const FILES: &[File<'_>] = &[File {
    name: "request.proto",
    imports: &[],
}];
const SYMBOLS: &[&str] = &[
    "pse.test.v1.Request",
    "google.protobuf.MessageOptions",
    "google.protobuf.FieldOptions",
];
const MESSAGE: Message<'_> = Message {
    name: "pse.test.v1.Request",
    file: "request.proto",
    metadata: Some(META),
};
fn native_field(ty: TypeRef, rules: Vec<LocalRule>) -> crate::boundary_projection::Field {
    crate::boundary_projection::Field {
        name: "value".into(),
        tag: 1,
        ty,
        direction: BoundaryDirection::Both,
        rules,
        named_facets: Default::default(),
    }
}
const NATIVE_U64: TypeRef = TypeRef::Scalar(crate::boundary_projection::Scalar::U64);
const FIELD_META: FieldMetadata<'_> = FieldMetadata {
    metadata_version: Some(VERSION),
    native_field: Some("value"),
    direction: Some(EnumValue::Known(BoundaryDirection::Both)),
    native_type: Some(&NATIVE_U64),
    local_rules: Some(&[]),
};
fn world<'a>(messages: &'a [Message<'a>]) -> World<'a> {
    World {
        files: FILES,
        symbols: SYMBOLS,
        messages,
        ..World::default()
    }
}

#[test]
fn native_scalar_container_and_local_rule_changes_do_not_hide_behind_wire_shapes() {
    use crate::boundary_projection::{Float64Domain, Scalar};
    for (old_type, new_type) in [
        (TypeRef::Scalar(Scalar::U64), TypeRef::Scalar(Scalar::I64)),
        (
            TypeRef::Scalar(Scalar::SemanticId),
            TypeRef::Scalar(Scalar::Bytes),
        ),
        (
            TypeRef::Set(Box::new(NATIVE_U64)),
            TypeRef::Repeated(Box::new(NATIVE_U64)),
        ),
    ] {
        let declared = native_field(old_type.clone(), vec![]);
        let old_fields = [Field {
            name: "pse.test.v1.Request.value",
            file: "request.proto",
            metadata: Some(FieldMetadata {
                native_type: Some(&old_type),
                ..FIELD_META
            }),
            declaration: FieldDeclaration::Present(&declared),
            reference: None,
        }];
        assert_eq!(
            check(
                &World {
                    fields: &old_fields,
                    ..world(&[MESSAGE])
                },
                &POLICY
            )
            .disposition,
            Disposition::Compatible
        );
        let new_fields = [Field {
            metadata: Some(FieldMetadata {
                native_type: Some(&new_type),
                ..FIELD_META
            }),
            ..old_fields[0]
        }];
        assert_eq!(
            compare(
                &World {
                    fields: &old_fields,
                    ..world(&[MESSAGE])
                },
                &World {
                    fields: &new_fields,
                    ..world(&[MESSAGE])
                },
                &POLICY
            )
            .disposition,
            Disposition::Incompatible
        );
    }
    let float = TypeRef::Scalar(Scalar::F64Bits);
    let finite = [LocalRule::Float64Domain(Float64Domain::Finite)];
    let positive = [LocalRule::Float64Domain(Float64Domain::PositiveFinite)];
    let declared = native_field(float.clone(), finite.to_vec());
    let old_fields = [Field {
        name: "pse.test.v1.Request.value",
        file: "request.proto",
        metadata: Some(FieldMetadata {
            native_type: Some(&float),
            local_rules: Some(&finite),
            ..FIELD_META
        }),
        declaration: FieldDeclaration::Present(&declared),
        reference: None,
    }];
    assert_eq!(
        check(
            &World {
                fields: &old_fields,
                ..world(&[MESSAGE])
            },
            &POLICY
        )
        .disposition,
        Disposition::Compatible
    );
    let new_fields = [Field {
        metadata: Some(FieldMetadata {
            native_type: Some(&float),
            local_rules: Some(&positive),
            ..FIELD_META
        }),
        ..old_fields[0]
    }];
    assert_eq!(
        compare(
            &World {
                fields: &old_fields,
                ..world(&[MESSAGE])
            },
            &World {
                fields: &new_fields,
                ..world(&[MESSAGE])
            },
            &POLICY
        )
        .disposition,
        Disposition::Incompatible
    );
}

#[test]
fn native_owner_string_vocabulary_and_alternative_tags_are_meaning() {
    use crate::boundary_projection::{Alternative, Shape};
    for (old, new) in [
        (
            Shape::StringEnum(vec!["before".into()]),
            Shape::StringEnum(vec!["after".into()]),
        ),
        (
            Shape::Alternatives(vec![Alternative {
                name: "Unit".into(),
                tag: 1,
                fields: vec![],
            }]),
            Shape::Alternatives(vec![Alternative {
                name: "Unit".into(),
                tag: 2,
                fields: vec![],
            }]),
        ),
    ] {
        let before = [Message {
            metadata: Some(MessageMetadata {
                native_shape: Some(&old),
                ..META
            }),
            ..MESSAGE
        }];
        let after = [Message {
            metadata: Some(MessageMetadata {
                native_shape: Some(&new),
                ..META
            }),
            ..MESSAGE
        }];
        assert_eq!(
            compare(&world(&before), &world(&after), &POLICY).disposition,
            Disposition::Incompatible
        );
    }
}

#[test]
fn old_contracts_stay_unknown_and_missing_current_field_meaning_refuses() {
    let declared = native_field(NATIVE_U64, vec![]);
    let field = Field {
        name: "pse.test.v1.Request.value",
        file: "request.proto",
        metadata: Some(FieldMetadata {
            native_type: None,
            local_rules: None,
            ..FIELD_META
        }),
        declaration: FieldDeclaration::Present(&declared),
        reference: None,
    };
    assert_eq!(
        check(
            &World {
                fields: &[field],
                ..world(&[MESSAGE])
            },
            &POLICY
        )
        .disposition,
        Disposition::Incompatible
    );
    let legacy = Field {
        metadata: Some(FieldMetadata {
            metadata_version: Some(1),
            native_type: None,
            local_rules: None,
            ..FIELD_META
        }),
        ..field
    };
    assert_eq!(
        check(
            &World {
                fields: &[legacy],
                ..world(&[MESSAGE])
            },
            &POLICY
        )
        .disposition,
        Disposition::Unknown
    );
}

#[test]
fn within_world_native_field_copies_must_agree_even_when_both_worlds_are_identical() {
    let declaration = native_field(NATIVE_U64, vec![]);
    let original = Field {
        name: "pse.test.v1.Request.value",
        file: "request.proto",
        metadata: Some(FIELD_META),
        declaration: FieldDeclaration::Present(&declaration),
        reference: None,
    };
    let honest = World {
        fields: &[original],
        ..world(&[MESSAGE])
    };
    assert_eq!(check(&honest, &POLICY).disposition, Disposition::Compatible);
    let signed = TypeRef::Scalar(crate::boundary_projection::Scalar::I64);
    let required = [LocalRule::Required];
    for metadata in [
        FieldMetadata {
            native_field: Some("other"),
            ..FIELD_META
        },
        FieldMetadata {
            direction: Some(EnumValue::Known(BoundaryDirection::Input)),
            ..FIELD_META
        },
        FieldMetadata {
            native_type: Some(&signed),
            ..FIELD_META
        },
        FieldMetadata {
            local_rules: Some(&required),
            ..FIELD_META
        },
    ] {
        let field = Field {
            metadata: Some(metadata),
            ..original
        };
        let inconsistent = World {
            fields: &[field],
            ..world(&[MESSAGE])
        };
        assert_eq!(
            compare(&inconsistent, &inconsistent, &POLICY).disposition,
            Disposition::Incompatible
        );
    }
    let field = Field {
        declaration: FieldDeclaration::Unavailable,
        ..original
    };
    assert_eq!(
        check(
            &World {
                fields: &[field],
                ..world(&[MESSAGE])
            },
            &POLICY
        )
        .disposition,
        Disposition::Unknown
    );
    let absent = Field {
        declaration: FieldDeclaration::KnownAbsent,
        ..field
    };
    assert_eq!(
        check(
            &World {
                fields: &[absent],
                ..world(&[MESSAGE])
            },
            &POLICY
        )
        .disposition,
        Disposition::Incompatible
    );
}

#[test]
fn required_presence_and_unknown_interpretation_are_distinct() {
    assert_eq!(
        check(&world(&[MESSAGE]), &POLICY).disposition,
        Disposition::Compatible
    );
    assert_eq!(
        check(
            &world(&[MESSAGE]),
            &CorePolicy {
                interpretations: &[]
            }
        )
        .disposition,
        Disposition::Unknown
    );
    let mut message = MESSAGE;
    message.metadata = None;
    let missing = check(&world(&[message]), &POLICY);
    assert_eq!(missing.disposition, Disposition::Incompatible);
    assert_eq!(missing.findings[0].file, "request.proto");
    message.metadata = Some(MessageMetadata {
        metadata_version: Some(99),
        direction: Some(EnumValue::Unknown(0)),
        ..META
    });
    let unknown = check(&world(&[message]), &POLICY);
    assert_eq!(unknown.disposition, Disposition::Unknown);
    assert_eq!(unknown.findings.len(), 2);
    assert_eq!(unknown.policy, "pse.contract.v2/native-crossing");
}

#[test]
fn wire_identical_role_and_version_changes_need_semantic_decisions() {
    let original = [MESSAGE];
    let old = world(&original);
    let changed = [Message {
        metadata: Some(MessageMetadata {
            construction: Some(EnumValue::Known(ConstructionRole::ObservationOnly)),
            ..META
        }),
        ..MESSAGE
    }];
    assert_eq!(
        compare(&old, &world(&changed), &POLICY).disposition,
        Disposition::Incompatible
    );
    let changed = [Message {
        metadata: Some(MessageMetadata {
            interpretation_version: Some(2),
            ..META
        }),
        ..MESSAGE
    }];
    assert_eq!(
        compare(&old, &world(&changed), &POLICY).disposition,
        Disposition::Unknown
    );
    struct DeclaredMigration;
    impl InterpretationPolicy for DeclaredMigration {
        fn name(&self) -> &str {
            "owner/declared-v1-v2"
        }
        fn supports_interpretation(&self, subject: &str, version: u32) -> bool {
            subject == "pse.test.v1.Request" && matches!(version, 1 | 2)
        }
        fn interpretation_change(&self, _: &str, old: u32, new: u32) -> Disposition {
            if (old, new) == (1, 2) {
                Disposition::Compatible
            } else {
                Disposition::Unknown
            }
        }
    }
    assert_eq!(
        compare(&old, &world(&changed), &DeclaredMigration).disposition,
        Disposition::Compatible
    );
}

#[test]
fn provenance_moves_do_not_become_physics_changes() {
    let changed = [Message {
        metadata: Some(MessageMetadata {
            owner: Some(Owner {
                rust_type: "new::Request",
                declaration_file: "src/moved.rs",
            }),
            ..META
        }),
        ..MESSAGE
    }];
    assert_eq!(
        compare(&world(&[MESSAGE]), &world(&changed), &POLICY).disposition,
        Disposition::Compatible
    );
}

#[test]
fn effect_changes_are_meaningful_even_without_wire_changes() {
    let observation = [Method {
        name: "pse.test.v1.Service.Run",
        file: "request.proto",
        input: SYMBOLS[0],
        output: SYMBOLS[0],
        metadata: Some(MethodMetadata {
            metadata_version: Some(VERSION),
            effects: Some(EffectFacts::OBSERVATION.into()),
        }),
    }];
    let mut execution = observation;
    execution[0].metadata.as_mut().unwrap().effects = Some(EffectFacts::NATIVE_EXECUTION.into());
    let original = world(&[]);
    let old = World {
        methods: &observation,
        ..original
    };
    let new = World {
        methods: &execution,
        ..original
    };
    assert_eq!(check(&new, &POLICY).disposition, Disposition::Compatible);
    assert_eq!(
        compare(&old, &new, &POLICY).disposition,
        Disposition::Incompatible
    );
    execution[0]
        .metadata
        .as_mut()
        .unwrap()
        .effects
        .as_mut()
        .unwrap()
        .read_only = Some(true);
    assert_eq!(
        check(
            &World {
                methods: &execution,
                ..original
            },
            &POLICY
        )
        .disposition,
        Disposition::Incompatible
    );
}

#[test]
fn imports_references_and_extensions_use_the_actual_world() {
    let extensions = [
        Extension {
            name: "pse.contract.v1.message",
            extendee: SYMBOLS[1],
            number: 51065,
            file: "request.proto",
        },
        Extension {
            name: "pse.contract.v1.field",
            extendee: SYMBOLS[2],
            number: 51065,
            file: "request.proto",
        },
    ];
    assert_eq!(
        check(
            &World {
                extensions: &extensions,
                ..world(&[])
            },
            &POLICY
        )
        .disposition,
        Disposition::Compatible
    );
    let mut collision = extensions;
    collision[1].extendee = SYMBOLS[1];
    assert_eq!(
        check(
            &World {
                extensions: &collision,
                ..world(&[])
            },
            &POLICY
        )
        .disposition,
        Disposition::Incompatible
    );
    let files = [File {
        name: "request.proto",
        imports: &["missing.proto"],
    }];
    let missing = check(
        &World {
            files: &files,
            ..world(&[])
        },
        &POLICY,
    );
    assert!(
        missing
            .findings
            .iter()
            .any(|finding| finding.reason == "missing transitive import")
    );
    let mut declared = native_field(NATIVE_U64, vec![]);
    declared.direction = BoundaryDirection::Input;
    let fields = [Field {
        name: "pse.test.v1.Request.value",
        file: "request.proto",
        metadata: Some(FieldMetadata {
            metadata_version: Some(VERSION),
            native_field: Some("value"),
            direction: Some(EnumValue::Known(BoundaryDirection::Input)),
            native_type: Some(&TypeRef::Scalar(crate::boundary_projection::Scalar::U64)),
            local_rules: Some(&[]),
        }),
        declaration: FieldDeclaration::Present(&declared),
        reference: Some("pse.absent.v1.Value"),
    }];
    assert_eq!(
        check(
            &World {
                fields: &fields,
                ..world(&[])
            },
            &POLICY
        )
        .disposition,
        Disposition::Incompatible
    );
}

#[test]
fn unavailable_domain_policy_never_falls_back_to_wire_compatibility() {
    let facets = [Facet {
        subject: SYMBOLS[0],
        file: "request.proto",
        policy: "pse.schema.v1.registry_field",
        value: &[1, 2],
        interpretation_available: true,
        interpretation_reason: None,
    }];
    let world = World {
        facets: &facets,
        ..world(&[])
    };
    assert_eq!(check(&world, &POLICY).disposition, Disposition::Unknown);
    assert_eq!(
        compare(&world, &world, &POLICY).disposition,
        Disposition::Unknown
    );
}

#[test]
fn original_action_effects_are_checked_independently_of_methods() {
    let mut graph = crate::boundary_projection::ProjectionGraph::default();
    graph
        .declare::<()>(
            "Action",
            file!(),
            2,
            BoundaryDirection::Input,
            &[],
            &[],
            |_| Ok(crate::boundary_projection::Shape::Record(vec![])),
        )
        .unwrap();
    graph
        .record_effects("Action", EffectFacts::OBSERVATION)
        .unwrap();
    graph
        .record_effects("Action", EffectFacts::OBSERVATION)
        .unwrap();
    assert_eq!(
        graph.definitions()["Action"].effects,
        Some(EffectFacts::OBSERVATION)
    );
    assert!(
        graph
            .record_effects("Action", EffectFacts::SOURCE_ADMISSION)
            .is_err()
    );
    assert!(
        graph
            .record_effects("Absent", EffectFacts::OBSERVATION)
            .is_err()
    );
    let old = [Message {
        metadata: Some(MessageMetadata {
            effects: Some(EffectFacts::OBSERVATION.into()),
            ..META
        }),
        ..MESSAGE
    }];
    let new = [Message {
        metadata: Some(MessageMetadata {
            effects: Some(EffectFacts::SOURCE_ADMISSION.into()),
            ..META
        }),
        ..MESSAGE
    }];
    assert_eq!(
        compare(&world(&old), &world(&new), &POLICY).disposition,
        Disposition::Incompatible
    );
}

#[test]
fn newly_introduced_meaning_and_unavailable_original_policy_need_owner_decisions() {
    struct DomainPolicy;
    impl InterpretationPolicy for DomainPolicy {
        fn name(&self) -> &str {
            "domain/known-current"
        }
        fn supports_interpretation(&self, subject: &str, version: u32) -> bool {
            POLICY.supports_interpretation(subject, version)
        }
        fn check_facet(&self, _: &Facet<'_>) -> Disposition {
            Disposition::Compatible
        }
        fn compare_facet(&self, _: &Facet<'_>, _: &Facet<'_>) -> Disposition {
            Disposition::Compatible
        }
    }
    let messages = [MESSAGE];
    let old = world(&messages);
    let facets = [Facet {
        subject: MESSAGE.name,
        file: MESSAGE.file,
        policy: "domain.quantity",
        value: &[1],
        interpretation_available: true,
        interpretation_reason: None,
    }];
    let new = World {
        facets: &facets,
        ..old
    };
    assert_eq!(
        check(&new, &DomainPolicy).disposition,
        Disposition::Compatible
    );
    assert_eq!(
        compare(&old, &new, &DomainPolicy).disposition,
        Disposition::Unknown,
        "introducing an understood facet needs a declared relation"
    );
    let unavailable = [Facet {
        interpretation_available: false,
        ..facets[0]
    }];
    let original = World {
        facets: &unavailable,
        ..old
    };
    assert_eq!(
        compare(&original, &new, &DomainPolicy).disposition,
        Disposition::Unknown,
        "identical bytes and current policy cannot replace unavailable original interpretation"
    );
}
