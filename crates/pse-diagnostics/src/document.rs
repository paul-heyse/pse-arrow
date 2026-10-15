// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Closed projections of standard transport types, independent of scientific models.
use std::borrow::Cow;

/// Schema projection of serde's closed standard-duration representation.
/// Integer widths and fields come from schemars's Duration projection; serde's
/// standard visitor owns decoding, overflow and unknown-field refusal.
#[derive(Debug)]
pub struct ClosedDuration;
impl schemars::JsonSchema for ClosedDuration {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("ClosedDuration")
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut schema = <std::time::Duration as schemars::JsonSchema>::json_schema(generator);
        schema.insert("additionalProperties".into(), false.into());
        schema
    }
}
