// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use std::collections::BTreeSet;
#[test]
fn signed_registry_domains_lower_to_exact_sint64_rules_and_optional_presence() {
    projection_struct! {
        struct SignedDomain {
            projection: {name:"SignedDomain",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            value: Option<i64> => {tag:1,direction:Direction::Both,rules:[LocalRule::SignedRange {min:i64::MIN,max:-1}]};
        }
    }
    let graph = ProjectionGraph::of::<SignedDomain>().unwrap();
    let set =
        descriptors_with_imports(&graph, "pse.signed.v1", "signed.proto", &imports()).unwrap();
    let file = set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("signed.proto"))
        .unwrap();
    let field = &file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("SignedDomain"))
        .unwrap()
        .field[0];
    assert!(field.proto3_optional.unwrap_or(false));
    let rules = field
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Sint64(rules)) = rules.r#type else {
        panic!("actual signed64 field")
    };
    assert!(matches!(
        rules.greater_than,
        Some(s_int64rules::GreaterThan::Gte(i64::MIN))
    ));
    assert!(matches!(
        rules.less_than,
        Some(s_int64rules::LessThan::Lte(-1))
    ));
}
use pse_model::boundary_projection::{Direction, LocalRule, ProjectionType, Service};
use pse_model::{projection_enum, projection_struct};

projection_struct! {
    pub struct GenericAdapterFixture<T> {
        projection: { name: "GenericAdapterFixtureFor", rust_path: crate::codegen::boundary_projection::tests::GenericAdapterFixture,
            version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        value: T => { tag: 3, direction: Direction::Both, rules: [] };
    }
}

#[test]
fn generic_adapter_sources_use_concrete_checked_public_argument_paths() {
    let mut graph = ProjectionGraph::of::<GenericAdapterFixture<usize>>().unwrap();
    GenericAdapterFixture::<pse_ids::SemanticId>::projection(&mut graph).unwrap();
    let source = native_adapters(
        &graph,
        &[
            "GenericAdapterFixtureForUsize",
            "GenericAdapterFixtureForSemanticId",
        ],
        "crate::wire",
        "pse_codegen",
    )
    .unwrap();
    let parsed = syn::parse_file(&source).unwrap();
    let mut arguments = BTreeSet::new();
    for item in &parsed.items {
        let syn::Item::Impl(implementation) = item else {
            continue;
        };
        let syn::Type::Path(owner) = implementation.self_ty.as_ref() else {
            continue;
        };
        let Some(segment) = owner.path.segments.last() else {
            continue;
        };
        if segment.ident != "GenericAdapterFixture" {
            continue;
        }
        let syn::PathArguments::AngleBracketed(generic) = &segment.arguments else {
            panic!("concrete generic native owner")
        };
        assert_eq!(generic.args.len(), 1);
        let syn::GenericArgument::Type(syn::Type::Path(argument)) = &generic.args[0] else {
            panic!("actual named native argument")
        };
        arguments.insert(
            argument
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(
        arguments,
        BTreeSet::from([
            vec!["usize".to_owned()],
            vec!["pse_ids".to_owned(), "SemanticId".to_owned()]
        ])
    );
    let set =
        descriptors_with_imports(&graph, "pse.generic.v1", "generic.proto", &imports()).unwrap();
    let file = set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("generic.proto"))
        .unwrap();
    assert_eq!(file.message_type.len(), 2);
    assert!(
        file.message_type
            .iter()
            .all(|message| message.field[0].number == Some(3))
    );
}

#[test]
fn binary64_setting_domains_lower_to_exact_fixed64_bounds_and_finite_cel() {
    projection_struct! {
        struct ScalarDomains {
            projection: { name: "ScalarDomains", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            tolerance: pse_model::scalars::Tolerance => { tag: 1, direction: Direction::Both, rules: [] };
            fraction: pse_model::scalars::Fraction => { tag: 2, direction: Direction::Both, rules: [] };
            bound: pse_model::scalars::FiniteBound => { tag: 3, direction: Direction::Both, rules: [] };
        }
    }
    let graph = ProjectionGraph::of::<ScalarDomains>().unwrap();
    let descriptors =
        descriptors_with_imports(&graph, "pse.domains.v1", "domains.proto", &imports()).unwrap();
    let file = descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("domains.proto"))
        .unwrap();
    for (name, maximum) in [
        ("Tolerance", f64::MAX.to_bits()),
        ("Fraction", 1.0_f64.to_bits()),
    ] {
        let field = &file
            .message_type
            .iter()
            .find(|message| message.name.as_deref() == Some(name))
            .unwrap()
            .field[0];
        assert_eq!(field.r#type, Some(Type::TYPE_FIXED64));
        let rules = field
            .options
            .as_option()
            .unwrap()
            .extension(&validate::FIELD)
            .unwrap();
        let Some(validate::__buffa::oneof::field_rules::Type::Fixed64(rules)) = rules.r#type else {
            panic!("exact float bit interval")
        };
        assert!(matches!(
            rules.greater_than,
            Some(fixed64rules::GreaterThan::Gt(0))
        ));
        assert_eq!(rules.less_than, Some(fixed64rules::LessThan::Lte(maximum)));
    }
    let field = &file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("FiniteBound"))
        .unwrap()
        .field[0];
    let rules = field
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    assert_eq!(rules.cel.len(), 1);
    assert_eq!(rules.cel[0].id.as_deref(), Some("float64.finite"));
    assert_eq!(
        rules.cel[0].expression.as_deref(),
        Some(
            "this < 9218868437227405312u || (this >= 9223372036854775808u && this < 18442240474082181120u)"
        )
    );
}

#[test]
fn checked_foreign_adapter_generation_uses_source_binding_and_fallible_extraction() {
    pse_model::projection_remote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(remote = "std::ops::Range<u16>", deny_unknown_fields)]
        struct NativeRangeBinding for std::ops::Range<u16> {
            projection: { name: "CheckedForeignRange", rust_path: NativeRangeBinding, version: 1,
                direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            start: u16 => { tag: 1, direction: Direction::Both, rules: [] };
            end: u16 => { tag: 7, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: 128 }] };
        }
    }
    projection_struct! {
        struct ForeignOwner {
            projection: { name: "ForeignOwner", version: 1, direction: Direction::Both,
                reserved_tags: [], reserved_names: [] };
            range: std::ops::Range<u16> => { tag: 3, direction: Direction::Both, rules: [], adapter: NativeRangeBinding };
        }
    }
    let graph = ProjectionGraph::of::<ForeignOwner>().unwrap();
    assert!(graph.definitions()["CheckedForeignRange"].checked_record);
    assert!(!graph.definitions()["ForeignOwner"].checked_record);
    let source = native_adapters(&graph, &["ForeignOwner"], "crate::wire", "pse_codegen").unwrap();
    fn call_is(expression: &syn::Expr, accessor: &str, method: &str, argument: &str) -> bool {
        let syn::Expr::Call(call) = expression else {
            return false;
        };
        let syn::Expr::Path(function) = call.func.as_ref() else {
            return false;
        };
        let segments = function
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        segments.ends_with(&[accessor.to_owned(), method.to_owned()])
            && call.args.len() == 1
            && matches!(&call.args[0], syn::Expr::Path(value) if value.path.is_ident(argument))
    }
    #[derive(Default)]
    struct AccessCalls {
        checked_extraction: bool,
        checked_adoption: bool,
        ordinary_extraction: bool,
        carrier: bool,
    }
    impl<'ast> syn::visit::Visit<'ast> for AccessCalls {
        fn visit_expr_try(&mut self, expression: &'ast syn::ExprTry) {
            if let syn::Expr::MethodCall(map) = expression.expr.as_ref()
                && map.method == "map_err"
                && map.args.len() == 1
                && call_is(
                    &map.receiver,
                    "CheckedProjectionRecord",
                    "try_into_projection_fields",
                    "self",
                )
                && let syn::Expr::Closure(closure) = &map.args[0]
                && let syn::Expr::MethodCall(rebase) = closure.body.as_ref()
                && rebase.method == "at"
                && rebase.args.len() == 1
                && matches!(&rebase.args[0], syn::Expr::Path(path) if path.path.is_ident("path"))
            {
                self.checked_extraction = true;
            }
            syn::visit::visit_expr_try(self, expression);
        }
        fn visit_expr(&mut self, expression: &'ast syn::Expr) {
            self.checked_adoption |= call_is(
                expression,
                "CheckedProjectionRecord",
                "from_projection_fields",
                "fields",
            );
            self.ordinary_extraction |= call_is(
                expression,
                "ProjectionRecord",
                "into_projection_fields",
                "self",
            );
            syn::visit::visit_expr(self, expression);
        }
        fn visit_type(&mut self, ty: &'ast syn::Type) {
            if let syn::Type::Path(carrier) = ty
                && carrier
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    == ["pse_model", "boundary_projection", "Adapted"]
                && let syn::PathArguments::AngleBracketed(arguments) =
                    &carrier.path.segments.last().unwrap().arguments
                && arguments.args.len() == 2
                && let syn::GenericArgument::Type(syn::Type::Path(range)) = &arguments.args[0]
                && range
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    == ["std", "ops", "Range"]
                && let syn::PathArguments::AngleBracketed(element) =
                    &range.path.segments.last().unwrap().arguments
                && element.args.len() == 1
                && matches!(&element.args[0], syn::GenericArgument::Type(syn::Type::Path(ty)) if ty.path.is_ident("u16"))
                && matches!(&arguments.args[1], syn::GenericArgument::Type(syn::Type::Path(ty)) if ty.path.is_ident("NativeRangeBinding"))
            {
                self.carrier = true;
            }
            syn::visit::visit_type(self, ty);
        }
    }
    let parsed = syn::parse_file(&source).unwrap();
    let mut access = AccessCalls::default();
    syn::visit::Visit::visit_file(&mut access, &parsed);
    assert!(
        access.checked_extraction,
        "checked extraction must rebase its refusal and propagate it with ?"
    );
    assert!(
        access.checked_adoption,
        "incoming fields must use the source binding's checked constructor"
    );
    assert!(
        access.ordinary_extraction,
        "ordinary native records must retain their original access contract"
    );
    assert!(
        access.carrier,
        "generated access must target the actual typed foreign carrier"
    );
    let descriptors =
        descriptors_with_imports(&graph, "pse.checked.v1", "checked.proto", &imports()).unwrap();
    let file = descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("checked.proto"))
        .unwrap();
    let foreign = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("CheckedForeignRange"))
        .unwrap();
    assert_eq!(
        foreign
            .field
            .iter()
            .map(|field| field.number.unwrap())
            .collect::<Vec<_>>(),
        vec![1, 7]
    );
}

#[test]
fn standard_leaf_descriptors_keep_intrinsic_width_version_and_duration_bounds() {
    projection_struct! {
        struct StandardLeaves {
            projection: { name: "StandardLeaves", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            version: pse_model::document::Version<8> => { tag: 1, direction: Direction::Both, rules: [LocalRule::Required] };
            duration: std::time::Duration => { tag: 2, direction: Direction::Both, rules: [LocalRule::Required] };
            pool: u16 => { tag: 3, direction: Direction::Both, rules: [] };
        }
    }
    let graph = ProjectionGraph::of::<StandardLeaves>().unwrap();
    let descriptors =
        descriptors_with_imports(&graph, "pse.standard.v1", "standard.proto", &imports()).unwrap();
    let file = descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("standard.proto"))
        .unwrap();
    let field = |name: &str, index: usize| {
        &file
            .message_type
            .iter()
            .find(|message| message.name.as_deref() == Some(name))
            .unwrap()
            .field[index]
    };
    let rules = field("StandardLeaves", 2)
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Uint32(rules)) = rules.r#type else {
        panic!("u16 native width")
    };
    assert!(matches!(
        rules.less_than,
        Some(u_int32rules::LessThan::Lte(65535))
    ));
    let rules = field("DocumentVersion8", 0)
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Uint32(rules)) = rules.r#type else {
        panic!("version")
    };
    assert!(matches!(
        rules.greater_than,
        Some(u_int32rules::GreaterThan::Gte(8))
    ));
    assert!(matches!(
        rules.less_than,
        Some(u_int32rules::LessThan::Lte(8))
    ));
    let rules = field("NonnegativeDuration", 1)
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Uint32(rules)) = rules.r#type else {
        panic!("canonical nanos")
    };
    assert!(matches!(
        rules.less_than,
        Some(u_int32rules::LessThan::Lte(999_999_999))
    ));
    let source =
        native_adapters(&graph, &["StandardLeaves"], "crate::wire", "pse_codegen").unwrap();
    assert!(source.contains("Version<8>"));
    assert!(source.contains("time::Duration"));
    syn::parse_file(&source).unwrap();
}

projection_enum! {
    #[derive(Debug)]
    enum Value {
        projection: { name: "Value", version: 1, direction: Direction::Both, reserved_tags: [8], reserved_names: ["retired"] };
        Missing => 1;
        Number(f64) => 3;
        Rows(Vec<Value>) => 7;
    }
}
projection_struct! {
    #[derive(Debug)]
    struct Request {
        projection: { name: "Request", version: 2, direction: Direction::Input, reserved_tags: [5], reserved_names: ["retired"] };
        limit: u32 => { tag: 2, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: 128 }] };
        id: pse_ids::SemanticId => { tag: 4, direction: Direction::Both, rules: [] };
        value: Option<Vec<Option<Value>>> => { tag: 7, direction: Direction::Input, rules: [] };
        output_only: bool => { tag: 9, direction: Direction::Output, rules: [] };
        files: BTreeMap<String, Vec<u8>> => { tag: 11, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 8 }] };
    }
}

// Import schema only: no host operation or semantic owner is projected from this corpus.
fn imports() -> Vec<FileDescriptorProto> {
    FileDescriptorSet::decode_from_slice(include_bytes!(
        "../../../../pse-host/src/generated/code-intelligence/descriptor.bin"
    ))
    .unwrap()
    .file
    .into_iter()
    .filter(|file| {
        file.name
            .as_deref()
            .is_some_and(|name| name.starts_with("google/") || name.starts_with("buf/"))
    })
    .collect()
}

#[test]
fn optional_unsigned_range_retains_presence_and_only_constrains_supplied_values() {
    projection_struct! {
        struct OptionalDeadline {
            projection: { name: "OptionalDeadline", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
            remaining_ns: Option<u64> => { tag: 10, direction: Direction::Input,
                rules: [LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
        }
    }
    let graph = ProjectionGraph::of::<OptionalDeadline>().unwrap();
    let descriptors =
        descriptors_with_imports(&graph, "pse.deadline.v1", "deadline.proto", &imports()).unwrap();
    let file = descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("deadline.proto"))
        .unwrap();
    let field = &file.message_type[0].field[0];
    assert_eq!(field.number, Some(10));
    assert_eq!(field.proto3_optional, Some(true));
    assert!(
        field.oneof_index.is_some(),
        "None must remain distinct from supplied zero"
    );
    let rules = field
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    assert_ne!(rules.required, Some(true));
    let Some(validate::__buffa::oneof::field_rules::Type::Uint64(rules)) = rules.r#type else {
        panic!("unsigned scalar rule")
    };
    assert!(matches!(
        rules.greater_than,
        Some(u_int64rules::GreaterThan::Gte(1))
    ));
    assert!(
        rules.less_than.is_none(),
        "the exact uint64 domain needs no tautological upper test"
    );
}

#[test]
fn explicit_tags_bits_identity_rules_and_direction_reach_descriptors() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    let set = descriptors_for_direction(
        &graph,
        "pse.boundary.v1",
        "pse/boundary/v1/owners.proto",
        &imports(),
        Direction::Input,
    )
    .unwrap();
    let file = set
        .file
        .iter()
        .find(|file| file.package.as_deref() == Some("pse.boundary.v1"))
        .unwrap();
    let request = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Request"))
        .unwrap();
    assert_eq!(request.reserved_range[0].start, Some(5));
    assert_eq!(request.reserved_range[0].end, Some(6));
    assert!(
        !request
            .field
            .iter()
            .any(|field| field.name.as_deref() == Some("output_only"))
    );
    let limit = request
        .field
        .iter()
        .find(|field| field.number == Some(2))
        .unwrap();
    let rules = limit
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Uint32(bounds)) = rules.r#type else {
        panic!("uint32 rules")
    };
    assert!(matches!(
        bounds.greater_than,
        Some(u_int32rules::GreaterThan::Gte(1))
    ));
    assert!(matches!(
        bounds.less_than,
        Some(u_int32rules::LessThan::Lte(128))
    ));
    let id = request
        .field
        .iter()
        .find(|field| field.number == Some(4))
        .unwrap();
    let rules = id
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    assert!(
        matches!(rules.r#type, Some(validate::__buffa::oneof::field_rules::Type::Bytes(bytes)) if bytes.len == Some(16))
    );
    let value = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Value"))
        .unwrap();
    assert_eq!(
        value
            .field
            .iter()
            .map(|field| field.number)
            .collect::<Vec<_>>(),
        [Some(1), Some(3), Some(7)]
    );
    assert_eq!(
        value.nested_type[1].field[0].r#type,
        Some(Type::TYPE_FIXED64)
    );
    assert!(
        value.oneof_decl[0]
            .options
            .as_option()
            .unwrap()
            .extension(&validate::ONEOF)
            .unwrap()
            .required
            .unwrap()
    );
    // Independently assert a known bit representation. No floating conversion occurs in proto.
    assert_eq!((-0.0_f64).to_bits(), 0x8000_0000_0000_0000);
}

#[test]
fn nested_presence_maps_and_recursive_names_are_fully_resolved() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    let set =
        descriptors_with_imports(&graph, "pse.boundary.v1", "owners.proto", &imports()).unwrap();
    let file = set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("owners.proto"))
        .unwrap();
    let request = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Request"))
        .unwrap();
    let optional_sequence = request
        .field
        .iter()
        .find(|field| field.number == Some(7))
        .unwrap();
    assert_eq!(optional_sequence.label, Some(Label::LABEL_OPTIONAL));
    assert_eq!(
        optional_sequence.type_name.as_deref(),
        Some(".pse.boundary.v1.Request.Field7Value")
    );
    let wrapper = request
        .nested_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Field7Value"))
        .unwrap();
    assert_eq!(wrapper.field[0].label, Some(Label::LABEL_REPEATED));
    assert_eq!(
        wrapper.nested_type[0].field[0].type_name.as_deref(),
        Some(".pse.boundary.v1.Value")
    );
    let entry = request
        .nested_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Field11Entry"))
        .unwrap();
    assert!(entry.options.as_option().unwrap().map_entry.unwrap());
    assert_eq!(entry.field[1].r#type, Some(Type::TYPE_BYTES));
    assert!(!entry.field[1].proto3_optional.unwrap_or_default());
}

#[test]
fn complete_services_generate_one_shared_root_and_reproducible_rust() {
    let mut graph = ProjectionGraph::default();
    let service = Service::new("EngineeringService")
        .unary::<Request, Request>(&mut graph, "Inspect")
        .unwrap();
    graph.service(service).unwrap();
    let config = default_config("crate::generated::__buffa_fds");
    let first = generate_with_config(
        &graph,
        "pse.boundary.v1",
        "owners.proto",
        &imports(),
        &config,
    )
    .unwrap();
    let second = generate_with_config(
        &graph,
        "pse.boundary.v1",
        "owners.proto",
        &imports(),
        &config,
    )
    .unwrap();
    assert_eq!(first.sources, second.sources);
    assert_eq!(first.descriptor_bytes, second.descriptor_bytes);
    assert!(first.warnings.is_empty(), "{:?}", first.warnings);
    let file = first
        .descriptor_set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("owners.proto"))
        .unwrap();
    assert_eq!(
        file.service[0].method[0].input_type.as_deref(),
        Some(".pse.boundary.v1.Request")
    );
    assert!(first.sources["mod.rs"].contains("shared-root.rs"));
    assert!(first.sources["shared-root.rs"].contains("pub mod __buffa_fds"));
    for source in first.sources.values() {
        syn::parse_file(source).expect("generated Rust parses");
    }
    let decoded = FileDescriptorSet::decode_from_slice(&first.descriptor_bytes).unwrap();
    assert!(
        decoded
            .file
            .iter()
            .any(|file| file.name.as_deref() == Some("buf/validate/validate.proto"))
    );
}

#[test]
fn missing_imports_unknown_types_and_unversioned_packages_refuse() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    assert!(descriptors(&graph, "pse.boundary.v1", "owners.proto").is_err());
    assert!(descriptors_with_imports(&graph, "pse.boundary", "owners.proto", &imports()).is_err());
    struct Unsupported;
    impl pse_model::boundary_projection::ValidateProjection for Unsupported {
        fn validate_at(
            &self,
            _: &str,
        ) -> Result<(), pse_model::boundary_projection::ValidationError> {
            Ok(())
        }
    }
    impl ProjectionType for Unsupported {
        fn projection(
            _: &mut ProjectionGraph,
        ) -> Result<TypeRef, pse_model::boundary_projection::ProjectionError> {
            Ok(TypeRef::Named("Absent".into()))
        }
    }
    projection_struct! {
        struct Bad {
            projection: { name: "Bad", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            value: Unsupported => { tag: 1, direction: Direction::Both, rules: [] };
        }
    }
    assert!(
        descriptors(
            &ProjectionGraph::of::<Bad>().unwrap(),
            "pse.boundary.v1",
            "bad.proto"
        )
        .is_err()
    );
}

#[test]
fn native_adapters_follow_source_fields_explicit_tags_and_recursive_presence() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    let source =
        native_adapters(&graph, &["Request", "Value"], "crate::wire", "pse_codegen").unwrap();
    syn::parse_file(&source).unwrap();
    let compact = source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .replace(",>", ">");
    assert!(compact.contains("ProjectionAlternative<7>"));
    assert!(compact.contains("ProjectionRecord>::FIELD_LAYOUT"));
    assert!(source.contains("decode_request"));
    assert!(source.contains("encode_value"));
    assert!(source.contains("unknown_fields"));
    assert!(source.contains("unique_pairs"));
    assert!(source.contains("Field7Value"));
    assert!(native_adapters(&graph, &["Absent"], "crate::wire", "pse_codegen").is_err());
}

#[test]
fn retaining_original_wire_is_an_explicit_policy_and_keeps_alternative_refusal() {
    let graph = ProjectionGraph::of::<Value>().unwrap();
    let strict = native_adapters(&graph, &["Value"], "crate::wire", "pse_codegen").unwrap();
    let retained =
        native_adapters_retaining_wire(&graph, &["Value"], "crate::wire", "pse_codegen").unwrap();
    assert!(strict.contains("unknown_fields("));
    assert!(!retained.contains("unknown_fields("));
    assert!(retained.contains("UnknownAlternative"));
    syn::parse_file(&retained).unwrap();
}

#[test]
fn domain_tautologies_are_lowered_exactly_without_unsigned_zero_comparisons() {
    use pse_model::boundary_projection::ValidateProjection;
    projection_struct! {
        struct DomainBounds {
            projection: { name: "DomainBounds", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            count: u64 => { tag: 1, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 0, max: 128 }] };
            entries: Vec<u64> => { tag: 2, direction: Direction::Both, rules: [LocalRule::Length { min: 0, max: 32 }] };
            invocation: u64 => { tag: 3, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
        }
    }
    let graph = ProjectionGraph::of::<DomainBounds>().unwrap();
    let descriptors =
        descriptors_with_imports(&graph, "pse.boundary.v1", "domain.proto", &imports()).unwrap();
    let message = &descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("domain.proto"))
        .unwrap()
        .message_type[0];
    let count = message.field[0]
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Uint64(count)) = count.r#type else {
        panic!("uint64 rule")
    };
    assert!(count.greater_than.is_none());
    assert!(matches!(
        count.less_than,
        Some(u_int64rules::LessThan::Lte(128))
    ));
    let entries = message.field[1]
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Repeated(entries)) = entries.r#type
    else {
        panic!("collection rule")
    };
    assert_eq!(entries.min_items, None);
    assert_eq!(entries.max_items, Some(32));
    let invocation = message.field[2]
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Uint64(invocation)) = invocation.r#type
    else {
        panic!("uint64 rule")
    };
    assert!(matches!(
        invocation.greater_than,
        Some(u_int64rules::GreaterThan::Gte(1))
    ));
    assert!(invocation.less_than.is_none());
    DomainBounds {
        count: 0,
        entries: vec![],
        invocation: u64::MAX,
    }
    .validate_projection()
    .unwrap();
    assert!(
        DomainBounds {
            count: 129,
            entries: vec![],
            invocation: 1
        }
        .validate_projection()
        .is_err()
    );
    assert!(
        DomainBounds {
            count: 0,
            entries: vec![1; 33],
            invocation: 1
        }
        .validate_projection()
        .is_err()
    );
}

#[test]
fn quantity_owner_callbacks_emit_checked_array_adapters_and_utf8_byte_rules() {
    projection_struct! {
        struct PhysicalSelection {
            projection: { name: "PhysicalSelection", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            unit: pse_quantity::Unit => { tag: 1, direction: Direction::Both, rules: [LocalRule::Required] };
            name: String => { tag: 2, direction: Direction::Both, rules: [LocalRule::Utf8Bytes { min: 0, max: 1024 }] };
        }
    }
    let graph = ProjectionGraph::of::<PhysicalSelection>().unwrap();
    let descriptors =
        descriptors_with_imports(&graph, "pse.physical.v1", "physical.proto", &imports()).unwrap();
    let file = descriptors
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("physical.proto"))
        .unwrap();
    let selection = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("PhysicalSelection"))
        .unwrap();
    let rules = selection.field[1]
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::String(rules)) = rules.r#type else {
        panic!("text bytes rule")
    };
    assert_eq!(rules.max_bytes, Some(1024));
    assert_eq!(
        rules.max_len, None,
        "byte limits must not become code-point limits"
    );
    let ratio = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("PhysicalRatio"))
        .unwrap();
    for field in &ratio.field {
        let rules = field
            .options
            .as_option()
            .unwrap()
            .extension(&validate::FIELD)
            .unwrap();
        let Some(validate::__buffa::oneof::field_rules::Type::Sint32(rules)) = rules.r#type else {
            panic!("actual signed16 width")
        };
        assert!(matches!(
            rules.greater_than,
            Some(s_int32rules::GreaterThan::Gte(-32768))
        ));
        assert!(matches!(
            rules.less_than,
            Some(s_int32rules::LessThan::Lte(32767))
        ));
    }
    let adapters =
        native_adapters(&graph, &["PhysicalSelection"], "crate::wire", "pse_codegen").unwrap();
    assert!(adapters.contains(".map_err(|error| error.at(path))"));
    assert!(adapters.contains("pse_quantity::dimension::Ratio; 8"));
    assert!(!adapters.contains("use super::*"));
    syn::parse_file(&adapters).unwrap();
}
