// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_contract::boundary_projection::Float64Domain;
use pse_contract::boundary_projection::Float64Interval;

#[test]
fn finite_interval_descriptors_preserve_strict_endpoints_and_nominal_value_paths() {
    const ALPHA: Float64Domain =
        Float64Domain::Interval(Float64Interval::new(1.0, false, 2.0, true));
    const PIVOT: Float64Domain =
        Float64Domain::Interval(Float64Interval::new(0.0, true, 1.0, true));
    const NEGATIVE: Float64Domain =
        Float64Domain::Interval(Float64Interval::new(-2.0, true, -1.0, false));
    const OPEN_UNIT: Float64Domain =
        Float64Domain::Interval(Float64Interval::new(0.0, false, 1.0, false));
    projection_struct! {
        struct IntervalSettings {
            projection:{name:"IntervalSettings",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            alpha: f64 => {tag:1,direction:Direction::Both,rules:[LocalRule::Float64Domain(ALPHA)]};
            pivot: Option<f64> => {tag:2,direction:Direction::Both,rules:[LocalRule::Float64Domain(PIVOT)]};
            negative: f64 => {tag:3,direction:Direction::Both,rules:[LocalRule::Float64Domain(NEGATIVE)]};
            confidence: pse_model::scalars::Fraction => {tag:4,direction:Direction::Both,rules:[LocalRule::Float64Domain(OPEN_UNIT)]};
            offsets: Vec<Option<pse_model::scalars::FiniteBound>> => {tag:5,direction:Direction::Both,rules:[LocalRule::Float64Domain(NEGATIVE)]};
        }
    }
    let graph = ProjectionGraph::of::<IntervalSettings>().unwrap();
    let set =
        descriptors_with_imports(&graph, "pse.interval.v1", "interval.proto", &imports()).unwrap();
    let file = set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("interval.proto"))
        .unwrap();
    let message = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("IntervalSettings"))
        .unwrap();
    let rules = |tag| {
        message
            .field
            .iter()
            .find(|field| field.number == Some(tag))
            .unwrap()
            .options
            .as_option()
            .unwrap()
            .extension(&validate::FIELD)
            .unwrap()
    };
    let samples = [
        f64::NAN,
        f64::from_bits(0xfff8_0000_0000_0042),
        f64::INFINITY,
        f64::NEG_INFINITY,
        -2.0,
        f64::from_bits((-2.0_f64).to_bits() - 1),
        -1.0,
        f64::from_bits((-1.0_f64).to_bits() + 1),
        -0.0,
        0.0,
        1.0,
        f64::from_bits(1.0_f64.to_bits() + 1),
        2.0,
        f64::from_bits(2.0_f64.to_bits() + 1),
    ];
    // Evaluate the actual emitted standard fixed64 bounds, including the reversed
    // unsigned ordering of negative IEEE values, against the original native predicate.
    for (tag, domain) in [(1, ALPHA), (3, NEGATIVE)] {
        let envelope = rules(tag);
        let Some(validate::__buffa::oneof::field_rules::Type::Fixed64(bounds)) = envelope.r#type
        else {
            panic!("standard fixed64 interval")
        };
        for value in samples {
            let bits = value.to_bits();
            let lower = match &bounds.greater_than {
                Some(fixed64rules::GreaterThan::Gt(minimum)) => bits > *minimum,
                Some(fixed64rules::GreaterThan::Gte(minimum)) => bits >= *minimum,
                None => true,
            };
            let upper = match &bounds.less_than {
                Some(fixed64rules::LessThan::Lt(maximum)) => bits < *maximum,
                Some(fixed64rules::LessThan::Lte(maximum)) => bits <= *maximum,
                None => true,
            };
            assert_eq!(
                domain.contains(value),
                lower && upper,
                "tag {tag}, {value:?}"
            );
        }
    }
    let pivot = rules(2);
    assert!(
        pivot.r#type.is_none(),
        "both signed zeros require disjoint unsigned intervals"
    );
    assert_eq!(
        pivot.cel[0].expression.as_deref(),
        Some(
            format!(
                "this <= {}u || this == {}u",
                1.0_f64.to_bits(),
                (-0.0_f64).to_bits()
            )
            .as_str()
        )
    );
    assert!(message.field[1].proto3_optional.unwrap_or(false));
    let confidence = rules(4);
    assert_eq!(
        confidence.cel[0].expression.as_deref(),
        Some(
            format!(
                "this.value >= 1u && this.value <= {}u",
                1.0_f64.to_bits() - 1
            )
            .as_str()
        )
    );
    let offsets = rules(5);
    let expression = offsets.cel[0].expression.as_deref().unwrap();
    assert!(expression.contains("has(element_0.value)"), "{expression}");
    assert!(
        expression.contains("element_0.value.value >="),
        "{expression}"
    );
    // The actual Fraction child still publishes its original (0,1] constructor contract.
    let fraction = file
        .message_type
        .iter()
        .find(|owner| owner.name.as_deref() == Some("Fraction"))
        .unwrap();
    let child = fraction.field[0]
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    assert!(
        matches!(child.r#type, Some(validate::__buffa::oneof::field_rules::Type::Fixed64(bounds))
        if bounds.less_than == Some(fixed64rules::LessThan::Lte(1.0_f64.to_bits())))
    );
}

#[test]
fn binary64_collection_domains_follow_declared_wire_wrappers_and_semantic_keys() {
    projection_struct! {
        struct NumericCollections {
            projection: {name:"NumericCollections",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            matrix: Option<Vec<Option<Vec<f64>>>> => {tag:1,direction:Direction::Both,
                rules:[LocalRule::Float64Domain(Float64Domain::Finite)]};
            coordinates: BTreeMap<pse_ids::SemanticId, Option<Vec<[f64; 2]>>> => {tag:2,direction:Direction::Both,
                rules:[LocalRule::Float64Domain(Float64Domain::Finite)]};
            named: BTreeMap<String, Option<Vec<f64>>> => {tag:3,direction:Direction::Both,
                rules:[LocalRule::Float64Domain(Float64Domain::Finite)]};
            windows: [Option<Vec<f64>>; 2] => {tag:4,direction:Direction::Both,
                rules:[LocalRule::Float64Domain(Float64Domain::NonnegativeFinite)]};
            sparse_start: Option<BTreeMap<pse_ids::SemanticId, f64>> => {tag:5,direction:Direction::Both,
                rules:[LocalRule::Float64Domain(Float64Domain::Finite)]};
        }
    }
    let graph = ProjectionGraph::of::<NumericCollections>().unwrap();
    let descriptor_set =
        descriptors_with_imports(&graph, "pse.numeric.v1", "numeric.proto", &imports()).unwrap();
    let file = descriptor_set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("numeric.proto"))
        .unwrap();
    let message = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("NumericCollections"))
        .unwrap();
    let expression = |tag| {
        let field = message
            .field
            .iter()
            .find(|field| field.number == Some(tag))
            .unwrap();
        let rules = field
            .options
            .as_option()
            .unwrap()
            .extension(&validate::FIELD)
            .unwrap();
        assert_eq!(rules.cel.len(), 1);
        assert_eq!(
            rules.cel[0].id.as_deref(),
            Some("float64.collection_domain")
        );
        rules.cel[0].expression.clone().unwrap()
    };
    // Optional repeated values retain their message wrapper; nested optional values
    // are guarded by actual field presence rather than a numeric default.
    assert_eq!(
        expression(1),
        format!(
            "this.value.all(element_1, (!(has(element_1.value)) || (element_1.value.value.all(element_3, (element_3 <= {}u || (element_3 >= {}u && element_3 <= {}u))))))",
            f64::MAX.to_bits(),
            (-0.0_f64).to_bits(),
            (-f64::MAX).to_bits()
        )
    );
    let entries = message
        .nested_type
        .iter()
        .find(|owner| owner.name.as_deref() == Some("Field2Entry"))
        .unwrap();
    assert!(
        !entries
            .options
            .as_option()
            .is_some_and(|options| options.map_entry.unwrap_or(false))
    );
    assert_eq!(entries.field[0].r#type, Some(Type::TYPE_BYTES));
    let key_rules = entries.field[0]
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    assert!(matches!(key_rules.r#type,
        Some(validate::__buffa::oneof::field_rules::Type::Bytes(bytes)) if bytes.len == Some(16)));
    let semantic = expression(2);
    assert!(semantic.starts_with("this.all(entry_0, ("), "{semantic}");
    assert!(semantic.contains("has(entry_0.value.value)"), "{semantic}");
    assert!(semantic.contains(".values.all("), "{semantic}");
    let string_map = expression(3);
    assert!(string_map.starts_with("this.all(key_0, ("), "{string_map}");
    assert!(
        string_map.contains("has(this[key_0].value)"),
        "{string_map}"
    );
    let arrays = expression(4);
    assert!(arrays.starts_with("this.values.all("), "{arrays}");
    assert!(
        arrays.contains(&format!("== {}u", (-0.0_f64).to_bits())),
        "{arrays}"
    );
    let sparse = expression(5);
    assert!(sparse.starts_with("this.value.all(entry_1, ("), "{sparse}");
    assert!(sparse.contains("entry_1.value <="), "{sparse}");
    let sparse_wrapper = message
        .nested_type
        .iter()
        .find(|owner| owner.name.as_deref() == Some("Field5Value"))
        .unwrap();
    assert_eq!(sparse_wrapper.field[0].name.as_deref(), Some("value"));
    assert_eq!(sparse_wrapper.field[0].label, Some(Label::LABEL_REPEATED));
    let sparse_entry = &sparse_wrapper.nested_type[0];
    assert_eq!(sparse_entry.field[0].r#type, Some(Type::TYPE_BYTES));
    assert_eq!(sparse_entry.field[1].r#type, Some(Type::TYPE_FIXED64));
    // All layout expressions are emitted with their actual messages, not a
    // synthetic flattened schema; adapters retain the native semantic-ID key.
    let source = native_adapters(
        &graph,
        &["NumericCollections"],
        "crate::wire",
        "pse_codegen",
    )
    .unwrap();
    let file = syn::parse_file(&source).unwrap();
    // Native types are inferred from the owner's associated field tuple. Verify
    // semantic-entry byte adoption instead of requiring a redundant Rust spelling.
    #[derive(Default)]
    struct EntryKey(bool);
    impl<'ast> syn::visit::Visit<'ast> for EntryKey {
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            if let syn::Expr::Path(function) = call.func.as_ref()
                && function
                    .path
                    .segments
                    .last()
                    .is_some_and(|part| part.ident == "from_projection_bytes")
                && function
                    .path
                    .segments
                    .iter()
                    .any(|part| part.ident == "ProjectionBytes")
                && let Some(syn::Expr::Field(field)) = call.args.first()
                && matches!(&field.member, syn::Member::Named(name) if name == "key")
                && matches!(field.base.as_ref(), syn::Expr::Path(base) if base.path.is_ident("entry"))
            {
                self.0 = true;
            }
            syn::visit::visit_expr_call(self, call);
        }
    }
    let mut key = EntryKey::default();
    syn::visit::Visit::visit_file(&mut key, &file);
    assert!(key.0);
}
