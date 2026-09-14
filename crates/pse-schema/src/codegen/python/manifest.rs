// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One strict msgspec object graph from the manifest declaration.

use std::fmt::Write as _;

use crate::model::{ManifestField, ManifestSpec, ManifestType};

use super::pascal;

pub(super) fn render(spec: &ManifestSpec) -> String {
    let mut out = String::from(
        "\"\"\"The declared snapshot envelope; logical validity is checked on admission.\"\"\"\n\nimport re\nfrom datetime import datetime\nfrom typing import Annotated, Literal\n\nimport msgspec\n",
    );
    out.push_str("\n\ndef _validate_timestamp(value: str) -> None:\n    if re.fullmatch(r\"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-5][0-9](?:\\.[0-9]{1,9})?Z\", value) is None:\n        message = \"timestamp must use the exact UTC RFC3339 wire shape\"\n        raise ValueError(message)\n    datetime.fromisoformat(value)\n");
    structure("Manifest", spec.fields(), spec, &mut out);
    out
}

fn structure(name: &str, fields: &[ManifestField], spec: &ManifestSpec, out: &mut String) {
    let mut body = String::new();
    let attributes = super::identifiers::fields(fields.iter().map(|field| field.name));
    for (field, attribute) in fields.iter().zip(&attributes) {
        let annotation = if name == "Manifest" && field.name == "manifest_version" {
            format!("Literal[{:?}]", spec.version)
        } else if name == "Manifest" && field.name == "membership_profile" {
            format!("Literal[{:?}]", spec.membership_profile)
        } else {
            annotation(
                &field.ty,
                &format!("{name}{}", pascal(field.name)),
                spec,
                out,
            )
        };
        let optional = matches!(field.ty, ManifestType::Optional(_));
        let default = if field.name != attribute.as_str() {
            let default = if optional { ", default=None" } else { "" };
            format!(" = msgspec.field(name={:?}{default})", field.name)
        } else if optional {
            " = None".to_owned()
        } else {
            String::new()
        };
        let _ = writeln!(body, "    {attribute}: {annotation}{default}");
    }
    let mut checks = String::new();
    for (field, attribute) in fields.iter().zip(&attributes) {
        if matches!(field.ty, ManifestType::U64) {
            let _ = writeln!(
                checks,
                "        if type(self.{attribute}) is not int or not 0 <= self.{attribute} <= 18446744073709551615:\n            message = {:?}\n            raise ValueError(message)",
                format!("{} must fit UInt64", field.name)
            );
        }
        if matches!(field.ty, ManifestType::Timestamp) {
            let _ = writeln!(checks, "        _validate_timestamp(self.{attribute})");
        }
    }
    if !checks.is_empty() {
        let _ = write!(body, "\n    def __post_init__(self) -> None:\n{checks}");
    }
    let _ = write!(
        out,
        "\n\nclass {name}(msgspec.Struct, frozen=True, forbid_unknown_fields=True, kw_only=True):\n    \"\"\"Declared manifest object.\"\"\"\n\n{body}"
    );
}

fn annotation(ty: &ManifestType, stem: &str, spec: &ManifestSpec, out: &mut String) -> String {
    match ty {
        ManifestType::Text | ManifestType::Timestamp => "str".to_owned(),
        ManifestType::U32 => "Annotated[int, msgspec.Meta(ge=0, le=4294967295)]".to_owned(),
        ManifestType::U64 => "Annotated[int, msgspec.Meta(ge=0)]".to_owned(),
        ManifestType::Bool => "bool".to_owned(),
        ManifestType::Id => "Annotated[str, msgspec.Meta(pattern=r\"^[0-9a-f]{32}$\")]".to_owned(),
        ManifestType::Hash => {
            "Annotated[str, msgspec.Meta(pattern=r\"^blake3:[0-9a-f]{64}$\")]".to_owned()
        }
        ManifestType::List(child) => format!(
            "tuple[{}, ...]",
            annotation(child, &format!("{stem}Item"), spec, out)
        ),
        ManifestType::Optional(child) => format!("{} | None", annotation(child, stem, spec, out)),
        ManifestType::Struct(fields) => {
            structure(stem, fields, spec, out);
            stem.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{ManifestField, ManifestSpec, ManifestType};

    #[test]
    fn keyword_fields_preserve_wire_names_defaults_and_validation_targets() {
        let spec = ManifestSpec::new(
            ManifestSpec::VERSION,
            ManifestSpec::MEMBERSHIP_PROFILE,
            vec![
                ManifestField::new("class", ManifestType::U64, "Keyword count."),
                ManifestField::new("class_", ManifestType::Text, "Distinct legal field."),
                ManifestField::new(
                    "from",
                    ManifestType::Optional(Box::new(ManifestType::Text)),
                    "Optional keyword.",
                ),
                ManifestField::new("with", ManifestType::Timestamp, "Keyword timestamp."),
            ],
        );
        let output = super::render(&spec);
        assert!(output.contains(
            "class__: Annotated[int, msgspec.Meta(ge=0)] = msgspec.field(name=\"class\")"
        ));
        assert!(output.contains("    class_: str\n"));
        assert!(output.contains("from_: str | None = msgspec.field(name=\"from\", default=None)"));
        assert!(output.contains("type(self.class__) is not int"));
        assert!(output.contains("_validate_timestamp(self.with_)"));
        assert!(!output.contains("    class:"));
        assert!(!output.contains("self.class)"));
    }
}
