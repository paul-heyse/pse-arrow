// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use std::time::Duration;

#[test]
fn positive_duration_lowering_uses_checked_component_names_and_keeps_canonical_nanos() {
    projection_struct! {
        struct TimeBudget {
            projection:{name:"TimeBudget",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            required: Duration => {tag:1,direction:Direction::Both,rules:[LocalRule::PositiveDuration]};
            optional: Option<Duration> => {tag:2,direction:Direction::Both,rules:[LocalRule::PositiveDuration]};
            general: Duration => {tag:3,direction:Direction::Both,rules:[]};
        }
    }
    let graph = ProjectionGraph::of::<TimeBudget>().unwrap();
    let set =
        descriptors_with_imports(&graph, "pse.duration.v1", "duration.proto", &imports()).unwrap();
    let file = set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("duration.proto"))
        .unwrap();
    let budget = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("TimeBudget"))
        .unwrap();
    for tag in [1, 2] {
        let field = budget
            .field
            .iter()
            .find(|field| field.number == Some(tag))
            .unwrap();
        assert_eq!(
            field.type_name.as_deref(),
            Some(".pse.duration.v1.NonnegativeDuration")
        );
        let rules = field
            .options
            .as_option()
            .unwrap()
            .extension(&validate::FIELD)
            .unwrap();
        assert_eq!(rules.cel.len(), 1);
        assert_eq!(rules.cel[0].id.as_deref(), Some("duration.positive"));
        assert_eq!(
            rules.cel[0].expression.as_deref(),
            Some("this.secs > 0u || this.nanos > 0u")
        );
        assert!(
            !rules.required.unwrap_or(false),
            "absence remains governed by the original field/presence contract"
        );
    }
    assert!(
        !budget.field[2]
            .options
            .as_option()
            .is_some_and(|options| options.extension(&validate::FIELD).is_some())
    );
    let duration = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("NonnegativeDuration"))
        .unwrap();
    let seconds = duration
        .field
        .iter()
        .find(|field| field.name.as_deref() == Some("secs"))
        .unwrap();
    assert_eq!(seconds.r#type, Some(Type::TYPE_UINT64));
    assert!(
        !seconds
            .options
            .as_option()
            .is_some_and(|options| options.extension(&validate::FIELD).is_some()),
        "no contextual clock/date ceiling belongs to the intrinsic Duration domain"
    );
    let nanos = duration
        .field
        .iter()
        .find(|field| field.name.as_deref() == Some("nanos"))
        .unwrap();
    let rules = nanos
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    assert!(
        matches!(rules.r#type, Some(validate::__buffa::oneof::field_rules::Type::Uint32(bounds))
        if bounds.less_than == Some(u_int32rules::LessThan::Lte(999_999_999)))
    );
    let source = native_adapters(&graph, &["TimeBudget"], "crate::wire", "pse_codegen").unwrap();
    syn::parse_file(&source).unwrap();
}
