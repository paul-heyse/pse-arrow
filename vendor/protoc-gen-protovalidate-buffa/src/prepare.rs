//! One request-local admitted scan and schema shared by preflight and emission.

use buffa_codegen::generated::compiler::{CodeGeneratorRequest, code_generator_response::File};
use crate::{emit, scan};

/// A selected, admitted world and its original output configuration.
/// Private fields prevent mutation between admission and emission.
#[derive(Debug)]
pub struct PreparedValidators {
    messages: Vec<scan::MessageValidators>,
    schemas: emit::cel::SchemaIndex,
    options: emit::Options,
}

impl PreparedValidators {
    /// Emit owned/view validators and the existing package/module layout from
    /// exactly the scan and schema that passed admission.
    pub fn render(&self) -> anyhow::Result<Vec<File>> {
        emit::render_with_schema(&self.messages, &self.options, &self.schemas)
    }
}

/// Original input failures or structured schema-bound CEL refusal diagnostics.
#[derive(Debug)]
pub enum PreparationError {
    /// Descriptor/target validation or the original scanner failed.
    Input(anyhow::Error),
    /// Required CEL rules were invalid or unsupported; no emission is admitted.
    Cel(Vec<emit::cel::PreflightDiagnostic>),
}

impl PreparationError {
    /// Preserve source/rule/category details independently of the CLI renderer.
    pub fn diagnostics(&self) -> &[emit::cel::PreflightDiagnostic] {
        match self {
            Self::Input(_) => &[],
            Self::Cel(diagnostics) => diagnostics,
        }
    }
}

impl std::fmt::Display for PreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(error) => std::fmt::Display::fmt(error, formatter),
            Self::Cel(diagnostics) => write!(formatter, "CEL preflight refused {} rule(s)", diagnostics.len()),
        }
    }
}

impl std::error::Error for PreparationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(error) => Some(error.as_ref()),
            Self::Cel(_) => None,
        }
    }
}

/// Check targets, scan their rules with complete import visibility, and admit
/// against one schema. Each call owns its world; there is no shared cache.
pub fn prepare(request: &CodeGeneratorRequest) -> Result<PreparedValidators, PreparationError> {
    check_targets(request).map_err(PreparationError::Input)?;
    let messages = scan::gather(request).map_err(PreparationError::Input)?;
    let schemas = emit::cel::build_schema_index(&messages);
    let diagnostics = emit::cel::preflight_with_schema(&messages, &schemas);
    if !diagnostics.is_empty() {
        return Err(PreparationError::Cel(diagnostics));
    }
    Ok(PreparedValidators {
        messages,
        schemas,
        options: emit::Options::from_parameter(request.parameter.as_deref().unwrap_or("")),
    })
}

fn check_targets(request: &CodeGeneratorRequest) -> anyhow::Result<()> {
    anyhow::ensure!(!request.file_to_generate.is_empty(), "explicit authored generation targets are required");
    let mut names = std::collections::BTreeSet::new();
    for file in &request.proto_file {
        let name = file.name.as_deref().ok_or_else(|| anyhow::anyhow!("descriptor has no logical filename"))?;
        anyhow::ensure!(names.insert(name), "duplicate descriptor filename: {name}");
    }
    let mut selected = std::collections::BTreeSet::new();
    for name in &request.file_to_generate {
        anyhow::ensure!(selected.insert(name), "duplicate authored target: {name}");
        anyhow::ensure!(names.contains(name.as_str()), "authored target missing from descriptor: {name}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::ExtensionSet;
    use buffa_codegen::generated::descriptor::{
        DescriptorProto, FieldDescriptorProto, FileDescriptorProto, MessageOptions,
        field_descriptor_proto::{Label, Type},
    };
    use protovalidate_buffa_protos::buf::validate::{MessageRules, Rule, __buffa::ext::MESSAGE};

    fn rule(expression: &str) -> Rule {
        Rule { id: Some("positive".into()), message: Some("positive value".into()), expression: Some(expression.into()), ..Default::default() }
    }
    fn request(expression: &str) -> CodeGeneratorRequest {
        let mut options = MessageOptions::default();
        options.set_extension(&MESSAGE, MessageRules { cel: vec![rule(expression)], ..Default::default() });
        CodeGeneratorRequest {
            file_to_generate: vec!["authored.proto".into()],
            parameter: Some("unknown=future, proto_module = crate::selected ".into()),
            proto_file: vec![FileDescriptorProto {
                name: Some("authored.proto".into()), package: Some("independent.v1".into()), syntax: Some("proto3".into()),
                message_type: vec![DescriptorProto {
                    name: Some("Request".into()), options: options.into(),
                    field: vec![FieldDescriptorProto { name: Some("value".into()), number: Some(1), r#type: Some(Type::TYPE_INT32), label: Some(Label::LABEL_OPTIONAL), ..Default::default() }],
                    ..Default::default()
                }], ..Default::default()
            }], ..Default::default()
        }
    }

    #[test]
    fn prepared_generation_keeps_original_world_options_and_complete_output() {
        let mut request = request("this.value > 0");
        let prepared = prepare(&request).unwrap();
        let actual = prepared.render().unwrap();
        // Retain the previous complete scan/emission result, including packaging,
        // instead of checking only a generated symbol or a successful response.
        let expected = emit::render_with_options(&scan::gather(&request).unwrap(), &emit::Options { proto_module: "crate::selected".into() }).unwrap();
        assert_eq!(actual, expected);
        let names = actual.iter().map(|file| file.name.as_deref().unwrap()).collect::<Vec<_>>();
        assert_eq!(names, ["authored.rs", "independent.v1.mod.rs", "mod.rs"]);
        let source = actual.iter().filter_map(|file| file.content.as_deref()).collect::<String>();
        assert!(source.contains("Validate for Request"));
        assert!(source.contains("Validate for __buffa::view::RequestView<'_>"));
        assert!(source.contains("use crate::selected::independent::v1::*"));
        request.parameter = Some("proto_module=crate::changed".into());
        request.proto_file[0].package = Some("changed.v2".into());
        assert_eq!(prepared.render().unwrap(), actual);
        let changed = prepare(&request).unwrap().render().unwrap();
        assert!(changed.iter().any(|file| file.name.as_deref() == Some("changed.v2.mod.rs")));
        assert!(changed.iter().filter_map(|file| file.content.as_deref()).collect::<String>().contains("use crate::changed::changed::v2::*"));
    }

    #[test]
    fn preparation_retains_structured_cel_refusal_and_selected_targets() {
        use emit::cel_compile::FallbackKind;
        let cases: &[(&str, &[(&str, FallbackKind, &str)])] = &[
            // The original scanner detects a missing top-level field, and the
            // schema-bound lowerer independently attributes it to this rule.
            ("this.missing > 0", &[
                ("", FallbackKind::Invalid, "expression references a non-existent field missing"),
                ("positive", FallbackKind::Invalid, "unknown field: missing"),
            ]),
            ("this.value +", &[("positive", FallbackKind::Invalid, "parse: ")]),
            ("[this.value, 1.5].size() > 0", &[("positive", FallbackKind::Unsupported, "heterogeneous numeric list needs a lossless dynamic representation")]),
        ];
        for (expression, expected) in cases {
            let error = prepare(&request(expression)).unwrap_err();
            let diagnostics = error.diagnostics();
            assert_eq!(diagnostics.len(), expected.len(), "{expression}: {diagnostics:?}");
            for (diagnostic, (rule, kind, reason)) in diagnostics.iter().zip(*expected) {
                assert_eq!(diagnostic.source_file, "authored.proto");
                assert_eq!(diagnostic.message, "independent.v1.Request");
                assert_eq!(diagnostic.rule, *rule);
                assert_eq!(diagnostic.kind, *kind);
                assert!(diagnostic.field.is_none());
                if *expression == "this.value +" {
                    // Parser internals supply the suffix; retain its complete
                    // original detail rather than manufacturing a replacement.
                    assert!(diagnostic.reason.starts_with(*reason));
                    assert!(diagnostic.reason.len() > reason.len());
                } else {
                    assert_eq!(diagnostic.reason, *reason);
                }
            }
            assert_eq!(error.to_string(), format!("CEL preflight refused {} rule(s)", expected.len()));
        }
        let mut selected = request("this.value > 0");
        let mut imported = request("this.missing > 0").proto_file.remove(0);
        imported.name = Some("unselected.proto".into());
        imported.package = Some("unselected.v1".into());
        selected.proto_file.push(imported);
        assert!(prepare(&selected).unwrap().render().unwrap().iter().all(|file| file.name.as_deref() != Some("unselected.rs")));
        selected.file_to_generate.push("unselected.proto".into());
        let error = prepare(&selected).unwrap_err();
        assert_eq!(error.diagnostics()[0].source_file, "unselected.proto");
    }

    #[test]
    fn preparation_refuses_invalid_target_population_before_rule_scanning() {
        let original = request("this.value > 0");
        for (change, expected) in [(0, "explicit authored"), (1, "missing.proto"), (2, "duplicate authored target"), (3, "duplicate descriptor filename"), (4, "no logical filename")] {
            let mut changed = original.clone();
            match change {
                0 => changed.file_to_generate.clear(),
                1 => changed.file_to_generate = vec!["missing.proto".into()],
                2 => changed.file_to_generate.push("authored.proto".into()),
                3 => changed.proto_file.push(changed.proto_file[0].clone()),
                _ => changed.proto_file[0].name = None,
            }
            let error = prepare(&changed).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
            assert!(error.diagnostics().is_empty());
            assert!(std::error::Error::source(&error).is_some());
        }
    }

    #[test]
    fn preparation_uses_predefined_rules_from_unselected_imports() {
        use buffa_codegen::generated::descriptor::FieldOptions;
        use protovalidate_buffa_protos::buf::validate::{
            FieldRules, Int32Rules, PredefinedRules,
            __buffa::{ext::{FIELD, PREDEFINED}, oneof::field_rules},
        };
        let mut request = request("this.value > 0");
        let mut predefined = FieldOptions::default();
        predefined.set_extension(&PREDEFINED, PredefinedRules { cel: vec![rule("this > 0")], ..Default::default() });
        request.proto_file.insert(0, FileDescriptorProto {
            name: Some("imported.proto".into()), package: Some("rules.v1".into()), syntax: Some("proto2".into()),
            extension: vec![FieldDescriptorProto { name: Some("positive".into()), number: Some(1001), extendee: Some(".buf.validate.Int32Rules".into()), r#type: Some(Type::TYPE_BOOL), label: Some(Label::LABEL_OPTIONAL), options: predefined.into(), ..Default::default() }],
            ..Default::default()
        });
        request.proto_file[1].dependency.push("imported.proto".into());
        let mut int32 = Int32Rules::default();
        int32.__buffa_unknown_fields.push(buffa::UnknownField { number: 1001, data: buffa::UnknownFieldData::Varint(1) });
        let mut options = FieldOptions::default();
        options.set_extension(&FIELD, FieldRules { r#type: Some(field_rules::Type::Int32(int32.into())), ..Default::default() });
        request.proto_file[1].message_type[0].field[0].options = options.into();
        let prepared = prepare(&request).unwrap();
        let source = prepared.render().unwrap().into_iter().filter_map(|file| file.content).collect::<String>();
        assert!(source.contains("positive"));
        assert!(!source.contains("imported.rs"));
        // The same imported declaration now carries an invalid required rule.
        let mut invalid = FieldOptions::default();
        invalid.set_extension(&PREDEFINED, PredefinedRules { cel: vec![rule("this + true")], ..Default::default() });
        request.proto_file[0].extension[0].options = invalid.into();
        let error = prepare(&request).unwrap_err();
        assert_eq!(error.diagnostics()[0].field.as_deref(), Some("value"));
        assert_eq!(error.diagnostics()[0].rule, "positive");
        assert_eq!(error.diagnostics()[0].source_file, "authored.proto");
    }
}
