// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Direct native values from the actual semantic projection graph.
use super::{ProjectionCodegenError, error, snake_case};
use pse_contract::boundary_projection::{
    ConstructionRole, ProjectionGraph, Scalar, Shape, TypeRef,
};
use std::fmt::Write as _;

/// Generate direct typed adapters. `support` names the consumer's native value support module.
/// No serialized document or protobuf descriptor participates in this projection.
pub fn native_value_adapters(
    graph: &ProjectionGraph,
    support: &str,
) -> Result<String, ProjectionCodegenError> {
    syn::parse_str::<syn::Path>(support).map_err(|e| error(e.to_string()))?;
    let mut source = format!(
        "use {support}::{{NativeValue, NativeError, required, exact_object}};\nuse surrealdb_types::{{Value,Object}};\n"
    );
    for owner in graph.definitions().values() {
        if owner.construction != ConstructionRole::Data {
            return Err(error(format!(
                "native value projection requires an ordinary data owner: {}",
                owner.name
            )));
        }
        let native = owner.public_rust_path.as_deref().unwrap_or(owner.rust_type);
        let native = native.replace("alloc::", "std::");
        let _ = writeln!(
            source,
            "impl NativeValue for {native} {{ fn into_native(self) -> Result<Value,NativeError> {{"
        );
        match &owner.shape {
            Shape::Record(fields) => {
                let accessor = if owner.checked_record {
                    "CheckedProjectionRecord"
                } else {
                    "ProjectionRecord"
                };
                source.push_str(&layout_check(
                    &native,
                    accessor,
                    None,
                    owner.version,
                    fields,
                    support,
                ));
                let bindings = (0..fields.len())
                    .map(|i| format!("f{i},"))
                    .collect::<String>();
                let extract = if owner.checked_record {
                    "pse_contract::boundary_projection::CheckedProjectionRecord::try_into_projection_fields(self).map_err(|_|NativeError)?"
                } else {
                    "pse_contract::boundary_projection::ProjectionRecord::into_projection_fields(self)"
                };
                let _ = writeln!(
                    source,
                    "let ({bindings}) = {extract}; let mut object=Object::new();"
                );
                for (i, field) in fields.iter().enumerate() {
                    let expression = encode(&field.ty, &format!("f{i}"), support);
                    let _ = writeln!(source, "object.insert({:?}, {expression});", field.name);
                }
                source.push_str("Ok(Value::Object(object))\n");
            }
            Shape::Alternatives(alternatives) => {
                source.push_str("let value=self;\n");
                for alternative in alternatives {
                    let tag = alternative.tag;
                    source.push_str(&layout_check(
                        &native,
                        "ProjectionAlternative",
                        Some(tag),
                        owner.version,
                        &alternative.fields,
                        support,
                    ));
                    let bindings = (0..alternative.fields.len())
                        .map(|i| format!("f{i},"))
                        .collect::<String>();
                    let _ = writeln!(
                        source,
                        "let value=match <Self as pse_contract::boundary_projection::ProjectionAlternative<{tag}>>::into_projection_alternative(value) {{ Ok(({bindings})) => {{ let mut object=Object::new(); object.insert(\"kind\",Value::String({:?}.into()));",
                        alternative.name
                    );
                    for (i, field) in alternative.fields.iter().enumerate() {
                        let expression = encode(&field.ty, &format!("f{i}"), support);
                        let _ = writeln!(source, "object.insert({:?},{expression});", field.name);
                    }
                    source.push_str("return Ok(Value::Object(object)); }, Err(other)=>other };\n");
                }
                source.push_str("let _=value; Err(NativeError)\n");
            }
            Shape::StringEnum(_) => source.push_str("Ok(Value::String(self.as_str().into()))\n"),
        }
        source.push_str("} fn from_native(value:Value)->Result<Self,NativeError> {\n");
        match &owner.shape {
            Shape::Record(fields) => {
                let accessor=if owner.checked_record {"CheckedProjectionRecord"} else {"ProjectionRecord"};
                source.push_str(&layout_check(&native,accessor,None,owner.version,fields,support));
                let names=fields.iter().map(|f|format!("{:?}",f.name)).collect::<Vec<_>>().join(",");
                let _=writeln!(source,"let mut object=exact_object(value,&[{names}])?;");
                let values=fields.iter().map(|f|format!("{},",decode(&f.ty,&field_value(f),support))).collect::<String>();
                let accessor=if owner.checked_record {"CheckedProjectionRecord"} else {"ProjectionRecord"};
                let _=writeln!(source,"let result=<Self as pse_contract::boundary_projection::{accessor}>::from_projection_fields(({values})).map_err(|_|NativeError)?;");
            }
            Shape::Alternatives(alternatives) => {
                source.push_str("let Value::Object(mut object)=value else {return Err(NativeError)}; let Value::String(kind)=required(&mut object,\"kind\")? else {return Err(NativeError)}; let result=match kind.as_str() {\n");
                for alternative in alternatives {
                    let names=alternative.fields.iter().map(|f|format!("{:?}",f.name)).collect::<Vec<_>>().join(",");
                    let values=alternative.fields.iter().map(|f|format!("{},",decode(&f.ty,&field_value(f),support))).collect::<String>();
                    let layout=layout_check(&native,"ProjectionAlternative",Some(alternative.tag),owner.version,&alternative.fields,support);
                    let check=if alternative.fields.is_empty() {format!("exact_object(Value::Object(object),&[{names}])?;")} else {format!("let mut object=exact_object(Value::Object(object),&[{names}])?;")};
                    let _=writeln!(source,"{:?} => {{ {layout} {check} <Self as pse_contract::boundary_projection::ProjectionAlternative<{}>>::from_projection_alternative(({values})) }},",alternative.name,alternative.tag);
                }
                source.push_str("_=>return Err(NativeError), };\n");
            }
            Shape::StringEnum(_) => source.push_str("let Value::String(text)=value else {return Err(NativeError)}; let result=text.parse::<Self>().map_err(|_|NativeError)?;\n"),
        }
        source.push_str("pse_contract::boundary_projection::ValidateProjection::validate_projection(&result).map_err(|_|NativeError)?; Ok(result) } }\n");
        let method = snake_case(&owner.name);
        let _ = writeln!(
            source,
            "/// Encode the source-owned value without a document intermediate.\npub fn encode_{method}(value:{native})->Result<Value,NativeError> {{ pse_contract::boundary_projection::ValidateProjection::validate_projection(&value).map_err(|_|NativeError)?; value.into_native() }}\n/// Decode the exact declared native product.\npub fn decode_{method}(value:Value)->Result<{native},NativeError> {{ {support}::admit(&value,1024*1024,128)?; NativeValue::from_native(value) }}"
        );
    }
    let parsed = syn::parse_file(&source).map_err(|e| error(e.to_string()))?;
    Ok(format!(
        "// @generated by pse-schema; do not edit\n{}",
        prettyplease::unparse(&parsed)
    ))
}

fn layout_check(
    native: &str,
    accessor: &str,
    tag: Option<u32>,
    version: u32,
    fields: &[pse_contract::boundary_projection::Field],
    support: &str,
) -> String {
    let accessor = tag.map_or_else(|| accessor.to_owned(), |tag| format!("{accessor}<{tag}>"));
    let fields = fields
        .iter()
        .map(|field| format!("({}, {:?})", field.tag, field.name))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{support}::check_layout(<{native} as pse_contract::boundary_projection::{accessor}>::VERSION, <{native} as pse_contract::boundary_projection::{accessor}>::FIELD_LAYOUT, {version}, &[{fields}])?;\n"
    )
}

fn field_value(field: &pse_contract::boundary_projection::Field) -> String {
    if matches!(field.ty, TypeRef::Optional(_)) {
        format!("object.remove({:?}).unwrap_or(Value::None)", field.name)
    } else {
        format!("required(&mut object,{:?})?", field.name)
    }
}

/// Native name for a source-owned product, independent of its Rust path.
pub fn native_type_name(root: &str) -> String {
    format!("pse_native_{}", snake_case(root))
}

/// Whether the selected source closure needs flexible native recursive subtrees.
pub fn projection_needs_flexible(graph: &ProjectionGraph, root: &str) -> bool {
    fn visit(
        graph: &ProjectionGraph,
        name: &str,
        path: &mut std::collections::BTreeSet<String>,
    ) -> bool {
        if !path.insert(name.into()) {
            return true;
        }
        let recursive = graph.definitions().get(name).is_some_and(|owner| {
            references(&owner.shape)
                .iter()
                .any(|name| visit(graph, name, path))
        });
        path.remove(name);
        recursive
    }
    visit(graph, root, &mut Default::default())
}
fn references(shape: &Shape) -> Vec<String> {
    fn collect(ty: &TypeRef, names: &mut Vec<String>) {
        match ty {
            TypeRef::Named(name) => names.push(name.clone()),
            TypeRef::Optional(inner) | TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
                collect(inner, names)
            }
            TypeRef::Map(key, value) => {
                collect(key, names);
                collect(value, names);
            }
            _ => {}
        }
    }
    let mut names = Vec::new();
    match shape {
        Shape::Record(fields) => {
            for field in fields {
                collect(&field.ty, &mut names);
            }
        }
        Shape::Alternatives(alternatives) => {
            for alternative in alternatives {
                for field in &alternative.fields {
                    collect(&field.ty, &mut names);
                }
            }
        }
        _ => {}
    }
    names
}
/// Emit native kinds in dependency order. Recursive edges remain native objects;
/// exact recursive shape is validated by generated owner construction at admission.
pub fn native_type_definitions(graph: &ProjectionGraph) -> Result<String, ProjectionCodegenError> {
    use std::collections::BTreeSet;
    fn visit(
        graph: &ProjectionGraph,
        name: &str,
        path: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
        cuts: &mut BTreeSet<(String, String)>,
        order: &mut Vec<String>,
    ) -> Result<(), ProjectionCodegenError> {
        if done.contains(name) {
            return Ok(());
        }
        path.insert(name.into());
        let owner = graph
            .definitions()
            .get(name)
            .ok_or_else(|| error(format!("unresolved native product {name}")))?;
        for child in references(&owner.shape) {
            if path.contains(&child) {
                cuts.insert((name.into(), child));
            } else {
                visit(graph, &child, path, done, cuts, order)?;
            }
        }
        path.remove(name);
        done.insert(name.into());
        order.push(name.into());
        Ok(())
    }
    let mut order = Vec::new();
    let mut done = BTreeSet::new();
    let mut cuts = BTreeSet::new();
    for name in graph.definitions().keys() {
        visit(
            graph,
            name,
            &mut BTreeSet::new(),
            &mut done,
            &mut cuts,
            &mut order,
        )?;
    }
    let mut output = String::new();
    for root in order {
        let owner = &graph.definitions()[&root];
        let object = |fields: &[pse_contract::boundary_projection::Field]| {
            fields
                .iter()
                .map(|f| format!("{}: {}", f.name, native_kind(&f.ty, &root, &cuts)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let kind = match &owner.shape {
            Shape::Record(fields) => format!("{{{}}}", object(fields)),
            Shape::Alternatives(alternatives) => alternatives
                .iter()
                .map(|alternative| {
                    let fields = object(&alternative.fields);
                    format!(
                        "{{kind: {:?}{}{fields}}}",
                        alternative.name,
                        if fields.is_empty() { "" } else { ", " }
                    )
                })
                .collect::<Vec<_>>()
                .join(" | "),
            Shape::StringEnum(values) => {
                let name = native_type_name(&owner.name);
                let members = values
                    .iter()
                    .map(|v| format!("{v:?}"))
                    .collect::<Vec<_>>()
                    .join(",");
                let _ = writeln!(
                    output,
                    "DEFINE TYPE {name} AS string ASSERT $value IN [{members}];"
                );
                continue;
            }
        };
        let assertion = native_assertion(owner);
        let _ = writeln!(
            output,
            "DEFINE TYPE {} AS {kind}{assertion};",
            native_type_name(&owner.name)
        );
    }
    Ok(output)
}
fn native_assertion(owner: &pse_contract::boundary_projection::Definition) -> String {
    if let Shape::Record(fields) = &owner.shape {
        let mut checks = Vec::new();
        for field in fields {
            let value = format!("$value.{}", field.name);
            match &field.ty {
                TypeRef::Scalar(Scalar::U8) => checks.push(format!("{value}<=255u64")),
                TypeRef::Scalar(Scalar::U16) => checks.push(format!("{value}<=65535u64")),
                TypeRef::Scalar(Scalar::U32) => checks.push(format!("{value}<=4294967295u64")),
                TypeRef::Scalar(Scalar::SemanticId | Scalar::I128Bits) => {
                    checks.push(format!("bytes::len({value})=16"))
                }
                TypeRef::Scalar(Scalar::ContentHash) => {
                    checks.push(format!("bytes::len({value})=32"))
                }
                TypeRef::Scalar(Scalar::F64Bits) => checks.push(format!("bytes::len({value})=8")),
                TypeRef::Scalar(Scalar::F32Bits) => checks.push(format!("bytes::len({value})=4")),
                _ => {}
            }
            for rule in &field.rules {
                match rule {
                    pse_contract::boundary_projection::LocalRule::UnsignedRange { min, max } => {
                        checks.push(format!("({value}>={min}u64 AND {value}<={max}u64)"))
                    }
                    pse_contract::boundary_projection::LocalRule::SignedRange { min, max } => {
                        checks.push(format!("({value}>={min} AND {value}<={max})"))
                    }
                    _ => {}
                }
            }
        }
        if checks.is_empty() {
            String::new()
        } else {
            format!(" ASSERT {}", checks.join(" AND "))
        }
    } else {
        String::new()
    }
}

fn native_kind(
    ty: &TypeRef,
    owner: &str,
    cuts: &std::collections::BTreeSet<(String, String)>,
) -> String {
    match ty {
        TypeRef::Scalar(Scalar::Bool) => "bool".into(),
        TypeRef::Scalar(Scalar::String) => "string".into(),
        TypeRef::Scalar(Scalar::U8 | Scalar::U16 | Scalar::U32 | Scalar::U64) => "u64".into(),
        TypeRef::Scalar(Scalar::I8 | Scalar::I16 | Scalar::I32 | Scalar::I64) => "int".into(),
        TypeRef::Scalar(_) => "bytes".into(),
        TypeRef::Named(name) if cuts.contains(&(owner.into(), name.clone())) => "object".into(),
        TypeRef::Named(name) => native_type_name(name),
        TypeRef::Optional(inner) => format!("option<{}>", native_kind(inner, owner, cuts)),
        TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
            format!("array<{}>", native_kind(inner, owner, cuts))
        }
        TypeRef::Map(key, value) => format!(
            "array<array<{} | {}>>",
            native_kind(key, owner, cuts),
            native_kind(value, owner, cuts)
        ),
    }
}

/// Expand only a projected root object where v4 FLEXIBLE eligibility cannot resolve aliases.
pub fn native_root_kind(
    graph: &ProjectionGraph,
    root: &str,
) -> Result<String, ProjectionCodegenError> {
    let owner = graph
        .definitions()
        .get(root)
        .ok_or_else(|| error(format!("missing native root {root}")))?;
    let cuts = std::collections::BTreeSet::new();
    match &owner.shape {
        Shape::Record(fields) => Ok(format!(
            "{{{}}}",
            fields
                .iter()
                .map(|field| format!("{}: {}", field.name, native_kind(&field.ty, root, &cuts)))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        _ => Err(error(format!(
            "flexible native root {root} must be a source-owned object"
        ))),
    }
}
/// Source-derived local root assertions also apply when the outer alias is expanded.
pub fn native_root_assertion(
    graph: &ProjectionGraph,
    root: &str,
) -> Result<Option<String>, ProjectionCodegenError> {
    let owner = graph
        .definitions()
        .get(root)
        .ok_or_else(|| error(format!("missing native root {root}")))?;
    let assertion = native_assertion(owner);
    Ok(assertion.strip_prefix(" ASSERT ").map(str::to_owned))
}

fn encode(ty: &TypeRef, value: &str, support: &str) -> String {
    match ty {
        TypeRef::Scalar(Scalar::F64Bits) => format!("{support}::Bits64::into_bits({value})"),
        TypeRef::Scalar(Scalar::F32Bits) => format!("{support}::Bits32::into_bits({value})"),
        TypeRef::Scalar(Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash) => format!(
            "Value::Bytes(pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes({value}).into())"
        ),
        TypeRef::Optional(inner) => format!(
            "match {value} {{ Some(value)=>{}, None=>Value::None }}",
            encode(inner, "value", support)
        ),
        TypeRef::Repeated(inner) | TypeRef::Set(inner) => format!(
            "Value::Array({value}.into_iter().map(|value| -> Result<Value,NativeError> {{ {} }}).collect::<Result<Vec<_>,_>>()?.into())",
            conversion_result(encode(inner, "value", support))
        ),
        TypeRef::Map(key, item) => format!(
            "Value::Array({value}.into_iter().map(|(key,value)| -> Result<Value,NativeError> {{ Ok(Value::Array(vec![{},{}].into())) }}).collect::<Result<Vec<_>,_>>()?.into())",
            encode(key, "key", support),
            encode(item, "value", support)
        ),
        _ => format!("NativeValue::into_native({value})?"),
    }
}
fn decode(ty: &TypeRef, value: &str, support: &str) -> String {
    match ty {
        TypeRef::Scalar(Scalar::F64Bits) => format!("{support}::Bits64::from_bits({value})?"),
        TypeRef::Scalar(Scalar::F32Bits) => format!("{support}::Bits32::from_bits({value})?"),
        TypeRef::Scalar(Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash) => format!(
            "pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes({support}::bytes({value})?,\"native\").map_err(|_|NativeError)?"
        ),
        TypeRef::Optional(inner) => format!(
            "match {value} {{ Value::None=>None, value=>Some({}) }}",
            decode(inner, "value", support)
        ),
        TypeRef::Repeated(inner) => format!(
            "{support}::array({value})?.into_iter().map(|value| -> Result<_,NativeError> {{ {} }}).collect::<Result<_,_>>()?",
            conversion_result(decode(inner, "value", support))
        ),
        TypeRef::Set(inner) => format!(
            "{support}::set({support}::array({value})?.into_iter().map(|value| -> Result<_,NativeError> {{ {} }}).collect::<Result<Vec<_>,_>>()?)?",
            conversion_result(decode(inner, "value", support))
        ),
        TypeRef::Map(key, item) => format!(
            "{support}::map({support}::array({value})?.into_iter().map(|value| -> Result<_,NativeError> {{ let [key,value]:[Value;2]={support}::array(value)?.try_into().map_err(|_|NativeError)?; Ok(({},{})) }}).collect::<Result<Vec<_>,_>>()?)?",
            decode(key, "key", support),
            decode(item, "value", support)
        ),
        _ => format!("NativeValue::from_native({value})?"),
    }
}

// Conversion expressions either return a value or unwrap their own Result with
// one outer `?`. A collection closure returns that Result directly.
fn conversion_result(expression: String) -> String {
    if let Some(fallible) = expression.strip_suffix('?') {
        fallible.to_owned()
    } else {
        format!("Ok({expression})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_study_projection_uses_actual_owner_construction_without_documents() {
        let mut graph = ProjectionGraph::default();
        graph.register::<pse_model::study::PointPolicy>().unwrap();
        graph.register::<pse_model::study::PointFacts>().unwrap();
        graph.register::<pse_model::study::PointAction>().unwrap();
        graph.register::<pse_model::study::PointOutcome>().unwrap();
        graph
            .register::<pse_model::study::ReadinessPremises>()
            .unwrap();
        let source = native_value_adapters(&graph, "crate::native").unwrap();
        assert!(source.contains("impl NativeValue for pse_model::study::PointPolicy"));
        assert!(source.contains("ProjectionRecord>::from_projection_fields"));
        assert!(source.contains("ProjectionAlternative<"));
        assert!(source.contains("crate::native::check_layout("));
        assert!(source.contains("ProjectionRecord>::FIELD_LAYOUT"));
        assert!(source.contains(">>::FIELD_LAYOUT"));
        assert!(!source.contains("serde_json"));
        assert!(!source.contains("buffa"));
    }
    #[test]
    fn native_study_kinds_are_parseable_and_do_not_alias_recursive_cycles() {
        let graph = pse_model::study::native_projection_graph().unwrap();
        let ddl = native_type_definitions(&graph).unwrap();
        surrealdb_expr::parser::parse(&ddl).unwrap();
        let diagnostic = ddl
            .lines()
            .find(|line| line.starts_with("DEFINE TYPE pse_native_model_boundary_diagnostic "))
            .unwrap();
        assert!(diagnostic.contains("causes: array<object>"));
        assert!(projection_needs_flexible(&graph, "StudyPointOutcome"));
        assert!(!projection_needs_flexible(&graph, "StudyPointFacts"));
    }
}
