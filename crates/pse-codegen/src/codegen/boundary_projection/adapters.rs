// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-graph-derived native adoption for selected Buffa roots.

use std::collections::BTreeSet;
use std::fmt::Write;

use pse_contract::boundary_projection::{
    ConstructionRole, Definition, Field, LocalRule, ProjectionGraph, Scalar, Shape, TypeRef,
};

use super::ProjectionCodegenError;
use super::{error, protobuf_map_key, snake_case};
mod borrowed;
pub use borrowed::{NativeViewTarget, native_view_adapters};

/// Reachability of generated root wrappers, independent of unknown-field adoption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterVisibility {
    /// Public source owner exported by its consumer crate.
    Public,
    /// Root wrapper is an internal crossing of the consumer crate.
    Crate,
    /// Root wrapper is internal to its immediate native consumer module.
    Parent,
}
impl AdapterVisibility {
    fn rust(self) -> &'static str {
        match self {
            Self::Public => "pub",
            Self::Crate => "pub(crate)",
            Self::Parent => "pub(super)",
        }
    }
}

#[cfg(test)]
mod visibility_tests {
    use super::*;
    pse_contract::projection_struct! {
        #[derive(Clone, Debug)]
        pub(super) struct LocalOwner {
            projection: { name: "LocalOwner", rust_path: crate::codegen::boundary_projection::adapters::visibility_tests::LocalOwner, version: 1, direction: pse_contract::BoundaryDirection::Both, reserved_tags: [], reserved_names: [] };
            pub value: bool => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [] };
        }
    }
    #[test]
    fn scoped_wrapper_visibility_preserves_strict_unknown_adoption() {
        let graph = ProjectionGraph::of::<LocalOwner>().unwrap();
        let strict = native_adapters_scoped(
            &graph,
            &["LocalOwner"],
            "crate::pb",
            "pse_codegen",
            AdapterVisibility::Crate,
        )
        .unwrap();
        assert!(strict.contains("pub(crate) fn encode_local_owner"));
        assert!(strict.contains("pub(crate) fn decode_local_owner"));
        assert!(strict.contains("unknown_fields(&wire.__buffa_unknown_fields, path)?"));
        let public = native_adapters(&graph, &["LocalOwner"], "crate::pb", "pse_codegen").unwrap();
        assert!(public.contains("pub fn decode_local_owner"));
        assert!(public.contains("unknown_fields(&wire.__buffa_unknown_fields, path)?"));
        let retained =
            native_adapters_retaining_wire(&graph, &["LocalOwner"], "crate::pb", "pse_codegen")
                .unwrap();
        assert!(retained.contains("pub(super) fn decode_local_owner"));
        assert!(!retained.contains("unknown_fields(&wire.__buffa_unknown_fields, path)?"));
    }
}

/// Emit adapters for exactly the selected native roots and their actual type closure.
/// `wire_root` names the generated Buffa package module; `owner_crate` is the consuming
/// Rust crate. Public alias wiring comes from the checked source declaration macro.
pub fn native_adapters(
    graph: &ProjectionGraph,
    roots: &[&str],
    wire_root: &str,
    owner_crate: &str,
) -> Result<String, ProjectionCodegenError> {
    native_adapters_scoped(
        graph,
        roots,
        wire_root,
        owner_crate,
        AdapterVisibility::Public,
    )
}

/// Emit strict native adoption with an explicit root-wrapper visibility. Private
/// native owners do not need a weaker unknown-field policy to keep their crossing local.
pub fn native_adapters_scoped(
    graph: &ProjectionGraph,
    roots: &[&str],
    wire_root: &str,
    owner_crate: &str,
    visibility: AdapterVisibility,
) -> Result<String, ProjectionCodegenError> {
    generate(graph, roots, wire_root, owner_crate, false, visibility)
}

/// Emit known-field adoption only for a caller that retains the complete original wire
/// message alongside that interpretation. Unknown alternatives still refuse; callers
/// must forward the retained message, never re-encode the partial native interpretation.
pub fn native_adapters_retaining_wire(
    graph: &ProjectionGraph,
    roots: &[&str],
    wire_root: &str,
    owner_crate: &str,
) -> Result<String, ProjectionCodegenError> {
    generate(
        graph,
        roots,
        wire_root,
        owner_crate,
        true,
        AdapterVisibility::Parent,
    )
}

fn generate(
    graph: &ProjectionGraph,
    roots: &[&str],
    wire_root: &str,
    owner_crate: &str,
    retained_wire: bool,
    visibility: AdapterVisibility,
) -> Result<String, ProjectionCodegenError> {
    syn::parse_str::<syn::Path>(wire_root)
        .map_err(|e| error(format!("invalid adapter package path: {e}")))?;
    let mut selected = BTreeSet::new();
    for name in roots {
        select(graph, name, &mut selected)?;
    }
    for name in &selected {
        let owner = &graph.definitions()[name];
        let mut closure = BTreeSet::new();
        match &owner.shape {
            Shape::Record(fields) => {
                for field in fields {
                    select_type(graph, &field.ty, &mut closure)?;
                }
            }
            Shape::Alternatives(alternatives) => {
                for alternative in alternatives {
                    for field in &alternative.fields {
                        select_type(graph, &field.ty, &mut closure)?;
                    }
                }
            }
            Shape::StringEnum(_) => {}
        }
        if owner.construction == ConstructionRole::Data
            && let Some(child) = closure.iter().find(|child| {
                graph.definitions()[*child].construction == ConstructionRole::ObservationOnly
            })
        {
            return Err(error(format!(
                "{name} contains observation-only native owner {child}; expose passive snapshot fields instead of reconstructing or moving live custody"
            )));
        }
        if owner.construction == ConstructionRole::ObservationOnly
            && !matches!(owner.shape, Shape::Record(_))
        {
            return Err(error(format!(
                "{name}: borrowed observation requires a record accessor"
            )));
        }
    }
    let naming_config = buffa_codegen::CodeGenConfig::default();
    let naming = buffa_codegen::context::CodeGenContext::new(&[], &naming_config, &[]);
    let generator = Generator {
        wire_root,
        owner_crate,
        retained_wire,
        naming: &naming,
        graph,
    };
    let mut out = String::new();
    for name in &selected {
        generator.owner(&graph.definitions()[name], &mut out)?;
    }
    for name in roots {
        let definition = &graph.definitions()[*name];
        let native = generator.native(definition)?;
        let wire = generator.wire(&[(*name).into()]);
        let method = snake_case(name);
        let visibility = visibility.rust();
        if definition.construction == ConstructionRole::ObservationOnly {
            let _ = writeln!(
                out,
                "/// Observe `{name}` without consuming native custody or constructing native authority.\n{visibility} fn encode_{method}(native: &{native}) -> Result<{wire}, BoundaryProjectionError> {{ pse_contract::boundary_projection::ValidateProjection::validate_projection(native).map_err(BoundaryProjectionError::OutgoingNative)?; let wire: {wire} = native_into(native, {name:?}).map_err(BoundaryProjectionError::OutgoingAdoption)?; protovalidate_buffa::Validate::validate(&wire).map_err(BoundaryProjectionError::OutgoingWire)?; Ok(wire) }}\n"
            );
            continue;
        }
        let _ = writeln!(
            out,
            "/// Encode the actual native `{name}` and validate outgoing local rules.\n{visibility} fn encode_{method}(native: {native}) -> Result<{wire}, BoundaryProjectionError> {{ encode_root(native, {name:?}) }}\n"
        );
        let _ = writeln!(
            out,
            "/// Adopt known `{name}` meaning under this package's declared unknown-field policy.\n{visibility} fn decode_{method}(wire: {wire}) -> Result<{native}, BoundaryProjectionError> {{ decode_root(wire, {name:?}) }}\n"
        );
    }
    // Import only mechanics actually used by this graph. A glob would also import the
    // parent's generated package and make its explicit public Rust paths redundant.
    let helpers = [
        "AdapterError",
        "AdapterFailure",
        "BoundaryProjectionError",
        "NativeInto",
        "NativeFrom",
        "layout",
        "child_path",
        "refused",
        "required",
        "unknown_fields",
        "native_into",
        "native_from",
        "encode_root",
        "decode_root",
        "items",
        "from_items",
        "unique_pairs",
        "wire_map",
    ];
    let imports = helpers
        .into_iter()
        .filter(|helper| {
            out.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .any(|token| token == *helper)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let out = format!(
        "// @generated by pse-codegen from actual typed owners; do not edit.\nuse super::{{{imports}}};\n{out}"
    );
    let parsed = syn::parse_file(&out)
        .map_err(|e| error(format!("native adapter rendering failed: {e}")))?;
    Ok(prettyplease::unparse(&parsed))
}

fn select(
    graph: &ProjectionGraph,
    name: &str,
    selected: &mut BTreeSet<String>,
) -> Result<(), ProjectionCodegenError> {
    if !selected.insert(name.into()) {
        return Ok(());
    }
    let definition = graph
        .definitions()
        .get(name)
        .ok_or_else(|| error(format!("native adapter root {name} is unresolved")))?;
    match &definition.shape {
        Shape::Record(fields) => {
            for field in fields {
                select_type(graph, &field.ty, selected)?;
            }
        }
        Shape::Alternatives(alternatives) => {
            for alternative in alternatives {
                for field in &alternative.fields {
                    select_type(graph, &field.ty, selected)?;
                }
            }
        }
        Shape::StringEnum(_) => {}
    }
    Ok(())
}

fn select_type(
    graph: &ProjectionGraph,
    ty: &TypeRef,
    selected: &mut BTreeSet<String>,
) -> Result<(), ProjectionCodegenError> {
    match ty {
        TypeRef::Named(name) => select(graph, name, selected)?,
        TypeRef::Optional(inner) | TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
            select_type(graph, inner, selected)?
        }
        TypeRef::Map(key, value) => {
            select_type(graph, key, selected)?;
            select_type(graph, value, selected)?;
        }
        TypeRef::Scalar(_) => {}
    }
    Ok(())
}

struct Generator<'a> {
    wire_root: &'a str,
    owner_crate: &'a str,
    retained_wire: bool,
    naming: &'a buffa_codegen::context::CodeGenContext<'a>,
    graph: &'a ProjectionGraph,
}

impl Generator<'_> {
    fn unknown_guard(&self, value: &str, path: &str) -> String {
        if self.retained_wire {
            String::new()
        } else {
            format!("unknown_fields(&{value}.__buffa_unknown_fields, {path})?;")
        }
    }
    fn native(&self, owner: &Definition) -> Result<String, ProjectionCodegenError> {
        let path = owner.public_rust_path.as_deref().unwrap_or(owner.rust_type);
        let mut native = syn::parse_str::<syn::Type>(path).map_err(|e| {
            error(format!(
                "{} needs an explicit nameable native Rust type: {e}",
                owner.name
            ))
        })?;
        struct LocalCrate<'a>(&'a str);
        impl syn::visit_mut::VisitMut for LocalCrate<'_> {
            fn visit_path_mut(&mut self, path: &mut syn::Path) {
                syn::visit_mut::visit_path_mut(self, path);
                if let Some(first) = path.segments.first_mut()
                    && first.ident == self.0
                {
                    first.ident = syn::Ident::new("crate", first.ident.span());
                    path.leading_colon = None;
                }
                // The native type universe supplies fully qualified standard
                // containers. Generated modules use Rust's identical prelude
                // names, avoiding redundant qualifications in generic owners.
                if path.segments.len() == 3 {
                    let parts = path
                        .segments
                        .iter()
                        .map(|part| part.ident.to_string())
                        .collect::<Vec<_>>();
                    if matches!(parts.as_slice(), [root, module, name]
                        if root == "std" && matches!((module.as_str(), name.as_str()),
                            ("vec", "Vec") | ("option", "Option") | ("result", "Result") | ("string", "String") | ("boxed", "Box")))
                    {
                        if let Some(last) = path.segments.pop() {
                            path.segments.clear();
                            path.segments.push(last);
                        }
                        path.leading_colon = None;
                    }
                }
            }
        }
        syn::visit_mut::VisitMut::visit_type_mut(&mut LocalCrate(self.owner_crate), &mut native);
        Ok(quote::quote!(#native).to_string())
    }

    fn wire(&self, path: &[String]) -> String {
        let mut result = self.wire_root.to_owned();
        for (index, segment) in path.iter().enumerate() {
            result.push_str("::");
            result.push_str(&if index + 1 == path.len() {
                segment.clone()
            } else {
                buffa_codegen::idents::escape_mod_ident(
                    &self.naming.nested_module_name("", segment),
                )
            });
        }
        result
    }

    fn nested(&self, parent: &[String], name: String) -> Vec<String> {
        let mut nested = parent.to_vec();
        nested.push(name);
        nested
    }

    fn layout(
        &self,
        native: &str,
        fields: &[Field],
        version: u32,
        alternative: Option<u32>,
        checked: bool,
    ) -> String {
        let accessor = alternative.map_or_else(
            || {
                if checked {
                    "CheckedProjectionRecord".into()
                } else {
                    "ProjectionRecord".into()
                }
            },
            |tag| format!("ProjectionAlternative<{tag}>"),
        );
        let layout = fields
            .iter()
            .map(|field| format!("({}, {:?})", field.tag, field.name))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "layout(<{native} as pse_contract::boundary_projection::{accessor}>::VERSION, <{native} as pse_contract::boundary_projection::{accessor}>::FIELD_LAYOUT, {version}, &[{layout}], path)?;"
        )
    }

    fn fields_encode(
        &self,
        fields: &[Field],
        message: &[String],
    ) -> Result<String, ProjectionCodegenError> {
        let mut out = String::new();
        for (index, field) in fields.iter().enumerate() {
            let name = buffa_codegen::idents::make_field_ident(&field.name);
            let child = format!("&child_path(path, {:?})", field.name);
            let encoded = self.encode(
                &field.ty,
                &format!("field_{index}"),
                &child,
                message,
                field.tag,
                false,
            )?;
            let encoded = if requires_scalar_presence(field) {
                format!("Some({encoded})")
            } else {
                encoded
            };
            let _ = write!(out, "{name}: {encoded},");
        }
        Ok(out)
    }

    fn fields_decode(
        &self,
        fields: &[Field],
        message: &[String],
    ) -> Result<String, ProjectionCodegenError> {
        let mut out = String::from("(");
        for field in fields {
            let name = buffa_codegen::idents::make_field_ident(&field.name);
            let child = format!("&child_path(path, {:?})", field.name);
            let input = if requires_scalar_presence(field) {
                format!("required(wire.{name}, {child})?")
            } else {
                format!("wire.{name}")
            };
            let decoded = self.decode(&field.ty, &input, &child, message, field.tag, false)?;
            let _ = write!(out, "{decoded},");
        }
        out.push(')');
        Ok(out)
    }

    fn owner(&self, owner: &Definition, out: &mut String) -> Result<(), ProjectionCodegenError> {
        let native = self.native(owner)?;
        let message = vec![owner.name.clone()];
        let wire = self.wire(&message);
        match &owner.shape {
            Shape::Record(fields) => {
                let bindings = fields
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!("field_{index},"))
                    .collect::<String>();
                let observation = owner.construction == ConstructionRole::ObservationOnly;
                let layout = if observation {
                    let layout = fields
                        .iter()
                        .map(|field| format!("({}, {:?})", field.tag, field.name))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        "layout(<{native} as pse_contract::boundary_projection::ProjectionObservationRecord>::VERSION, <{native} as pse_contract::boundary_projection::ProjectionObservationRecord>::FIELD_LAYOUT, {}, &[{layout}], path)?;",
                        owner.version
                    )
                } else {
                    self.layout(&native, fields, owner.version, None, owner.checked_record)
                };
                let accessor = if observation {
                    "ProjectionObservationRecord"
                } else if owner.checked_record {
                    "CheckedProjectionRecord"
                } else {
                    "ProjectionRecord"
                };
                let extract = if observation {
                    format!(
                        "pse_contract::boundary_projection::{accessor}::observation_fields(self).map_err(|error| error.at(path))?"
                    )
                } else if owner.checked_record {
                    format!(
                        "pse_contract::boundary_projection::{accessor}::try_into_projection_fields(self).map_err(|error| error.at(path))?"
                    )
                } else {
                    format!(
                        "pse_contract::boundary_projection::{accessor}::into_projection_fields(self)"
                    )
                };
                let encode = self.fields_encode(fields, &message)?;
                if observation {
                    let _ = writeln!(
                        out,
                        "impl NativeInto<{wire}> for &{native} {{ fn into_wire(self, path: &str) -> Result<{wire}, AdapterError> {{ {layout} let ({bindings}) = {extract}; Ok({wire} {{ {encode} ..Default::default() }}) }} }}"
                    );
                    return Ok(());
                }
                let decode = self.fields_decode(fields, &message)?;
                let unknown = self.unknown_guard("wire", "path");
                let _ = writeln!(
                    out,
                    "impl NativeInto<{wire}> for {native} {{ fn into_wire(self, path: &str) -> Result<{wire}, AdapterError> {{ {layout} let ({bindings}) = {extract}; Ok({wire} {{ {encode} ..Default::default() }}) }} }}"
                );
                let _ = writeln!(
                    out,
                    "impl NativeFrom<{wire}> for {native} {{ fn from_wire(wire: {wire}, path: &str) -> Result<Self, AdapterError> {{ {unknown} {layout} let fields = {decode}; pse_contract::boundary_projection::{accessor}::from_projection_fields(fields).map_err(|error| error.at(path)) }} }}"
                );
            }
            Shape::Alternatives(alternatives) => {
                let kind = format!(
                    "{}::__buffa::oneof::{}::Kind",
                    self.wire_root,
                    snake_case(&owner.name)
                );
                let _ = writeln!(
                    out,
                    "impl NativeInto<{wire}> for {native} {{ fn into_wire(self, path: &str) -> Result<{wire}, AdapterError> {{ let value = self;"
                );
                for alternative in alternatives {
                    let layout = self.layout(
                        &native,
                        &alternative.fields,
                        owner.version,
                        Some(alternative.tag),
                        false,
                    );
                    let bindings = alternative
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(index, _)| format!("field_{index},"))
                        .collect::<String>();
                    let payload_path =
                        self.nested(&message, format!("Alternative{}", alternative.tag));
                    let payload = self.wire(&payload_path);
                    let encode = self.fields_encode(&alternative.fields, &payload_path)?;
                    let variant =
                        buffa_codegen::idents::to_upper_camel_case(&snake_case(&alternative.name));
                    let tag = alternative.tag;
                    let alternative_name = &alternative.name;
                    let path_setup = if !alternative
                        .fields
                        .iter()
                        .any(|field| encoding_uses_path(&field.ty))
                    {
                        String::new()
                    } else {
                        format!(
                            "let alternative_path = child_path(path, {alternative_name:?}); let path = alternative_path.as_str();"
                        )
                    };
                    let _ = writeln!(
                        out,
                        "{layout} let value = match <{native} as pse_contract::boundary_projection::ProjectionAlternative<{tag}>>::into_projection_alternative(value) {{ Ok(({bindings})) => {{ {path_setup} return Ok({wire} {{ kind: Some({kind}::{variant}(Box::new({payload} {{ {encode} ..Default::default() }}))), ..Default::default() }}) }}, Err(value) => value }};"
                    );
                }
                let _ = writeln!(
                    out,
                    "let _ = value; Err(refused(path, AdapterFailure::UnknownAlternative, \"actual alternative is outside the generated source closure\")) }} }}"
                );
                let unknown = self.unknown_guard("wire", "path");
                let _ = writeln!(
                    out,
                    "impl NativeFrom<{wire}> for {native} {{ fn from_wire(wire: {wire}, path: &str) -> Result<Self, AdapterError> {{ {unknown} match wire.kind.ok_or_else(|| refused(path, AdapterFailure::UnknownAlternative, \"required alternative is absent\"))? {{"
                );
                for alternative in alternatives {
                    let layout = self.layout(
                        &native,
                        &alternative.fields,
                        owner.version,
                        Some(alternative.tag),
                        false,
                    );
                    let payload_path =
                        self.nested(&message, format!("Alternative{}", alternative.tag));
                    let decode = self.fields_decode(&alternative.fields, &payload_path)?;
                    let variant =
                        buffa_codegen::idents::to_upper_camel_case(&snake_case(&alternative.name));
                    let tag = alternative.tag;
                    let alternative_name = &alternative.name;
                    let unknown = self.unknown_guard("wire", "path");
                    let _ = writeln!(
                        out,
                        "{kind}::{variant}(wire) => {{ let alternative_path = child_path(path, {alternative_name:?}); let path = alternative_path.as_str(); let wire = *wire; {unknown} {layout} let fields = {decode}; Ok(<{native} as pse_contract::boundary_projection::ProjectionAlternative<{tag}>>::from_projection_alternative(fields)) }},"
                    );
                }
                let _ = writeln!(out, "}} }} }}");
            }
            Shape::StringEnum(_) => {
                let unknown = self.unknown_guard("wire", "path");
                let _ = writeln!(
                    out,
                    "impl NativeInto<{wire}> for {native} {{ fn into_wire(self, _: &str) -> Result<{wire}, AdapterError> {{ Ok({wire} {{ value: self.as_str().into(), ..Default::default() }}) }} }}"
                );
                let _ = writeln!(
                    out,
                    "impl NativeFrom<{wire}> for {native} {{ fn from_wire(wire: {wire}, path: &str) -> Result<Self, AdapterError> {{ {unknown} wire.value.parse::<Self>().map_err(|_| refused(path, AdapterFailure::UnknownAlternative, \"unknown canonical source vocabulary member\")) }} }}"
                );
            }
        }
        Ok(())
    }

    fn encode(
        &self,
        ty: &TypeRef,
        value: &str,
        path: &str,
        parent: &[String],
        tag: u32,
        raw: bool,
    ) -> Result<String, ProjectionCodegenError> {
        Ok(match ty {
            TypeRef::Scalar(Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash) => format!(
                "pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes({value})"
            ),
            TypeRef::Scalar(scalar) => format!(
                "native_into::<_, {}>({value}, {path})?",
                scalar_wire(*scalar)
            ),
            TypeRef::Named(name) => {
                let value = if let Some(owner) = self.graph.definitions().get(name)
                    && owner.construction == ConstructionRole::ObservationOnly
                {
                    let native = self.native(owner)?;
                    format!("::core::borrow::Borrow::<{native}>::borrow(&{value})")
                } else {
                    value.to_owned()
                };
                let converted = format!(
                    "native_into::<_, {}>({value}, {path})?",
                    self.wire(&[name.clone()])
                );
                if raw {
                    converted
                } else {
                    format!("buffa::MessageField::some({converted})")
                }
            }
            TypeRef::Optional(inner) => match inner.as_ref() {
                TypeRef::Scalar(_) => {
                    let converted = self.encode(inner, "value", path, parent, tag, true)?;
                    format!(
                        "{value}.map(|value| -> Result<_, AdapterError> {{ Ok({converted}) }}).transpose()?"
                    )
                }
                TypeRef::Named(_) => {
                    let converted = self.encode(inner, "value", path, parent, tag, true)?;
                    format!(
                        "{value}.map(|value| -> Result<_, AdapterError> {{ Ok({converted}) }}).transpose()?.into()"
                    )
                }
                _ => {
                    let wrapped = self.encode_wrapper(inner, "value", path, parent, tag)?;
                    format!(
                        "{value}.map(|value| -> Result<_, AdapterError> {{ Ok({wrapped}) }}).transpose()?.into()"
                    )
                }
            },
            TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
                if matches!(
                    inner.as_ref(),
                    TypeRef::Scalar(Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash)
                ) {
                    // Owner-checked byte extraction is infallible. Avoid manufacturing
                    // unused per-element error paths for that native conversion.
                    return Ok(format!(
                        "items({value}).map(pse_contract::boundary_projection::ProjectionBytes::into_projection_bytes).collect::<Vec<_>>()"
                    ));
                }
                let converted = if matches!(inner.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_))
                {
                    self.encode(
                        inner,
                        "value",
                        "&format!(\"{path}[{index}]\", path = current_path)",
                        parent,
                        tag,
                        true,
                    )?
                } else {
                    self.encode_wrapper(
                        inner,
                        "value",
                        "&format!(\"{path}[{index}]\", path = current_path)",
                        parent,
                        tag,
                    )?
                };
                if !encoding_uses_path(inner) {
                    return Ok(format!(
                        "items({value}).map(|value| -> Result<_, AdapterError> {{ Ok({converted}) }}).collect::<Result<Vec<_>, _>>()?"
                    ));
                }
                format!(
                    "{{ let current_path = {path}; items({value}).enumerate().map(|(index, value)| -> Result<_, AdapterError> {{ Ok({converted}) }}).collect::<Result<Vec<_>, _>>()? }}"
                )
            }
            TypeRef::Map(key, item) => {
                let entry_path = self.nested(parent, format!("Field{tag}Entry"));
                let encoded_key =
                    self.encode(key, "key", path, &entry_path, 1, protobuf_map_key(key))?;
                if protobuf_map_key(key) {
                    let encoded_item =
                        if matches!(item.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_)) {
                            self.encode(item, "value", path, &entry_path, 2, true)?
                        } else {
                            self.encode_wrapper(item, "value", path, parent, tag)?
                        };
                    format!(
                        "wire_map({value}.into_iter().map(|(key, value)| -> Result<_, AdapterError> {{ Ok(({encoded_key}, {encoded_item})) }}).collect::<Result<Vec<_>, _>>()?.into_iter())"
                    )
                } else {
                    let entry = self.wire(&entry_path);
                    let encoded_item = self.encode(item, "value", path, &entry_path, 2, false)?;
                    format!(
                        "{value}.into_iter().map(|(key, value)| -> Result<_, AdapterError> {{ Ok({entry} {{ key: {encoded_key}, value: {encoded_item}, ..Default::default() }}) }}).collect::<Result<Vec<_>, _>>()?"
                    )
                }
            }
        })
    }

    fn encode_wrapper(
        &self,
        inner: &TypeRef,
        value: &str,
        path: &str,
        parent: &[String],
        tag: u32,
    ) -> Result<String, ProjectionCodegenError> {
        let nested = self.nested(parent, format!("Field{tag}Value"));
        let message = self.wire(&nested);
        let value = self.encode(inner, value, path, &nested, 1, false)?;
        Ok(format!(
            "{message} {{ value: {value}, ..Default::default() }}"
        ))
    }

    fn decode(
        &self,
        ty: &TypeRef,
        value: &str,
        path: &str,
        parent: &[String],
        tag: u32,
        raw: bool,
    ) -> Result<String, ProjectionCodegenError> {
        Ok(match ty {
            TypeRef::Scalar(Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash) => format!(
                "pse_contract::boundary_projection::ProjectionBytes::from_projection_bytes({value}, {path})?"
            ),
            TypeRef::Scalar(scalar) => format!(
                "native_from::<_, {}>({value}, {path})?",
                scalar_wire(*scalar)
            ),
            TypeRef::Named(name) => {
                let actual = if raw {
                    value.to_owned()
                } else {
                    format!("required({value}.into_option(), {path})?")
                };
                format!(
                    "native_from::<_, {}>({actual}, {path})?",
                    self.wire(&[name.clone()])
                )
            }
            TypeRef::Optional(inner) => {
                let input = if matches!(inner.as_ref(), TypeRef::Scalar(_)) {
                    value.to_owned()
                } else {
                    format!("{value}.into_option()")
                };
                let converted = if matches!(inner.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_))
                {
                    self.decode(inner, "value", path, parent, tag, true)?
                } else {
                    self.decode_wrapper(inner, "value", path, parent, tag)?
                };
                format!(
                    "{input}.map(|value| -> Result<_, AdapterError> {{ Ok({converted}) }}).transpose()?"
                )
            }
            TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
                let indexed = "&format!(\"{path}[{index}]\", path = current_path)";
                let converted = if matches!(inner.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_))
                {
                    self.decode(inner, "value", indexed, parent, tag, true)?
                } else {
                    self.decode_wrapper(inner, "value", indexed, parent, tag)?
                };
                format!(
                    "{{ let current_path = {path}; let values = {value}.into_iter().enumerate().map(|(index, value)| -> Result<_, AdapterError> {{ Ok({converted}) }}).collect::<Result<Vec<_>, _>>()?; from_items(values, current_path)? }}"
                )
            }
            TypeRef::Map(key, item) => {
                let entry_path = self.nested(parent, format!("Field{tag}Entry"));
                if protobuf_map_key(key) {
                    let key = self.decode(key, "key", path, &entry_path, 1, true)?;
                    let item = if matches!(item.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_)) {
                        self.decode(item, "value", path, &entry_path, 2, true)?
                    } else {
                        self.decode_wrapper(item, "value", path, parent, tag)?
                    };
                    format!(
                        "unique_pairs({value}.into_iter().map(|(key, value)| -> Result<_, AdapterError> {{ Ok(({key}, {item})) }}).collect::<Result<Vec<_>, _>>()?, {path})?"
                    )
                } else {
                    let key = self.decode(key, "entry.key", path, &entry_path, 1, false)?;
                    let item = self.decode(item, "entry.value", path, &entry_path, 2, false)?;
                    let unknown = self.unknown_guard("entry", path);
                    format!(
                        "unique_pairs({value}.into_iter().map(|entry| -> Result<_, AdapterError> {{ {unknown} Ok(({key}, {item})) }}).collect::<Result<Vec<_>, _>>()?, {path})?"
                    )
                }
            }
        })
    }

    fn decode_wrapper(
        &self,
        inner: &TypeRef,
        value: &str,
        path: &str,
        parent: &[String],
        tag: u32,
    ) -> Result<String, ProjectionCodegenError> {
        let nested = self.nested(parent, format!("Field{tag}Value"));
        let decoded = self.decode(inner, "wrapped.value", path, &nested, 1, false)?;
        let unknown = self.unknown_guard("wrapped", path);
        Ok(format!("{{ let wrapped = {value}; {unknown} {decoded} }}"))
    }
}

fn requires_scalar_presence(field: &Field) -> bool {
    field.rules.contains(&LocalRule::Required) && matches!(field.ty, TypeRef::Scalar(_))
}

fn encoding_uses_path(ty: &TypeRef) -> bool {
    match ty {
        TypeRef::Scalar(Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash) => false,
        TypeRef::Scalar(_) | TypeRef::Named(_) => true,
        TypeRef::Optional(inner) | TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
            encoding_uses_path(inner)
        }
        TypeRef::Map(key, value) => encoding_uses_path(key) || encoding_uses_path(value),
    }
}

fn scalar_wire(scalar: Scalar) -> &'static str {
    match scalar {
        Scalar::Bool => "bool",
        Scalar::String => "String",
        Scalar::U8 | Scalar::U16 | Scalar::U32 | Scalar::F32Bits => "u32",
        Scalar::U64 | Scalar::F64Bits => "u64",
        Scalar::I8 | Scalar::I16 | Scalar::I32 => "i32",
        Scalar::I64 => "i64",
        Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash | Scalar::I128Bits => "Vec<u8>",
    }
}
