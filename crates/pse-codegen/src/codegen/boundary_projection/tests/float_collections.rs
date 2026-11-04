// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_model::boundary_projection::Float64Domain;

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
            "this.value.all(element_1, (!(has(element_1.value)) || (element_1.value.value.all(element_3, (element_3 < {}u || (element_3 >= {}u && element_3 < {}u))))))",
            f64::INFINITY.to_bits(),
            (-0.0_f64).to_bits(),
            f64::NEG_INFINITY.to_bits()
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
                    .is_some_and(|part| part.ident == "native_from")
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
