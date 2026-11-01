// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Inspect explicit generator-emitted traits; never reconstruct Rust names from proto spelling.
#[path = "../../../../scripts/protobuf-associations.rs"]
#[allow(
    unreachable_pub,
    reason = "Shared protocol types are public in the host library and private inside this generator binary"
)]
mod protocol;
use super::*;
use protocol::{Association, AssociationManifest};
use quote::ToTokens as _;
use std::collections::{BTreeMap, BTreeSet};
use syn::ext::IdentExt as _;
use syn::visit::Visit as _;

#[derive(Default)]
struct SelfFields(BTreeSet<String>);
impl<'ast> syn::visit::Visit<'ast> for SelfFields {
    fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
        if let syn::Expr::Path(path) = field.base.as_ref()
            && path.path.is_ident("self")
            && let syn::Member::Named(name) = &field.member
        {
            self.0.insert(name.to_string());
        }
        syn::visit::visit_expr_field(self, field);
    }
}
#[derive(Default)]
struct Accessors(BTreeMap<u32, String>, BTreeSet<u32>);
fn field_number(expression: &syn::Expr) -> bool {
    let syn::Expr::MethodCall(call) = expression else {
        return false;
    };
    matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1 && path.path.segments[0].ident.unraw() == "field" && matches!(path.path.segments[0].arguments, syn::PathArguments::None))
        && call.method.unraw() == "number"
        && call.turbofish.is_none()
        && call.args.is_empty()
}
fn literal_number(pattern: &syn::Pat) -> Option<u32> {
    let syn::Pat::Lit(pattern) = pattern else {
        return None;
    };
    let syn::Lit::Int(number) = &pattern.lit else {
        return None;
    };
    if !matches!(number.suffix(), "" | "u32") {
        return None;
    }
    number.base10_parse().ok()
}
impl<'ast> syn::visit::Visit<'ast> for Accessors {
    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        if field_number(&expression.expr) {
            for arm in &expression.arms {
                let Some(number) = literal_number(&arm.pat) else {
                    continue;
                };
                let mut members = SelfFields::default();
                members.visit_expr(&arm.body);
                // Oneof accessors can refer to several Rust members. They remain unassociated.
                if members.0.len() == 1
                    && let Some(member) = members.0.into_iter().next()
                {
                    if self.1.contains(&number) {
                        continue;
                    }
                    if self
                        .0
                        .get(&number)
                        .is_some_and(|previous| previous != &member)
                    {
                        self.0.remove(&number);
                        self.1.insert(number);
                    } else {
                        self.0.insert(number, member);
                    }
                }
            }
        }
        syn::visit::visit_expr_match(self, expression);
    }
}

fn implementations(
    items: &[syn::Item],
    module: &str,
    names: &mut BTreeMap<String, String>,
    accessors: &mut BTreeMap<String, BTreeMap<u32, String>>,
) {
    for item in items {
        match item {
            syn::Item::Mod(item) => {
                if let Some((_, items)) = &item.content {
                    implementations(
                        items,
                        &format!("{module}{}::", item.ident),
                        names,
                        accessors,
                    );
                }
            }
            syn::Item::Impl(item) => {
                let Some((path, _)) = &item.trait_ else {
                    continue;
                };
                let Some(segment) = path.segments.last() else {
                    continue;
                };
                let rust_type = format!("{module}{}", item.self_ty.to_token_stream());
                if segment.ident.unraw() == "MessageName" {
                    for child in &item.items {
                        if let syn::ImplItem::Const(value) = child
                            && value.ident.unraw() == "FULL_NAME"
                            && let syn::Expr::Lit(value) = &value.expr
                            && let syn::Lit::Str(value) = &value.lit
                        {
                            names.insert(rust_type.clone(), value.value());
                        }
                    }
                } else if segment.ident.unraw() == "ReflectMessage" {
                    for child in &item.items {
                        if let syn::ImplItem::Fn(function) = child
                            && function.sig.ident.unraw() == "get"
                        {
                            let mut visitor = Accessors::default();
                            visitor.visit_block(&function.block);
                            accessors.insert(rust_type.clone(), visitor.0);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn messages(set: &FileDescriptorSet) -> BTreeMap<String, (String, BTreeSet<u32>)> {
    fn walk(
        output: &mut BTreeMap<String, (String, BTreeSet<u32>)>,
        file: &str,
        prefix: &str,
        items: &[buffa_descriptor::generated::descriptor::DescriptorProto],
    ) {
        for item in items {
            let name = format!("{prefix}{}", item.name.as_deref().unwrap_or_default());
            output.insert(
                name.clone(),
                (
                    file.into(),
                    item.field
                        .iter()
                        .filter(|field| {
                            field.oneof_index.is_none() || field.proto3_optional.unwrap_or(false)
                        })
                        .filter_map(|field| {
                            field.number.and_then(|number| u32::try_from(number).ok())
                        })
                        .collect(),
                ),
            );
            walk(output, file, &format!("{name}."), &item.nested_type);
        }
    }
    let mut result = BTreeMap::new();
    for file in &set.file {
        let prefix = file
            .package
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(|value| format!("{value}."))
            .unwrap_or_default();
        walk(
            &mut result,
            file.name.as_deref().unwrap_or_default(),
            &prefix,
            &file.message_type,
        );
    }
    result
}

pub(super) fn generate(
    root: &Path,
    source: &Path,
    inputs: &Inputs,
    descriptor: &[u8],
    set: &FileDescriptorSet,
    tree: &mut GeneratedTree,
) -> Result<()> {
    let hash = |bytes: &[u8]| {
        sha2::Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let declarations = messages(set);
    let mut associations = Vec::new();
    for (path, bytes) in &tree.files {
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let syntax = syn::parse_file(std::str::from_utf8(bytes)?)?;
        let mut names = BTreeMap::new();
        let mut accessors = BTreeMap::new();
        implementations(&syntax.items, "", &mut names, &mut accessors);
        for (rust_type, name) in names {
            let Some((file, fields)) = declarations.get(&name) else {
                continue;
            };
            let Some(authored) = inputs.files.get(Path::new(file)) else {
                continue;
            };
            let base = Association {
                declaration: name.clone(),
                source: source
                    .join(file)
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .into_owned(),
                source_sha256: hash(authored),
                output: path.to_string_lossy().into_owned(),
                output_sha256: hash(bytes),
                rust_type: rust_type.clone(),
                rust_field: None,
                state: String::new(),
            };
            associations.push(base.clone());
            if let Some(accessors) = accessors.get(&rust_type) {
                for (number, member) in accessors {
                    if fields.contains(number) {
                        let mut field = base.clone();
                        field.declaration = format!("{name}#{number}");
                        field.rust_field = Some(member.clone());
                        associations.push(field);
                    }
                }
            }
        }
    }
    let mut configurations = inputs
        .files
        .iter()
        .map(|(path, bytes)| {
            Ok((
                source
                    .join(path)
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .into_owned(),
                hash(bytes),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    for path in [
        "xtask/src/codegen/code_intelligence.rs",
        "xtask/src/codegen/code_intelligence/associations.rs",
        "scripts/protobuf-associations.rs",
    ] {
        configurations.insert(path.into(), hash(&std::fs::read(root.join(path))?));
    }
    // Configuration/producer bytes and the complete descriptor qualify the actual emitted mapping.
    let manifest = AssociationManifest {
        descriptor_sha256: hash(descriptor),
        inputs: configurations,
        associations,
    };
    let producer = if source.ends_with("proto") {
        "code-intelligence"
    } else {
        "plan61-fixtures"
    };
    let path = PathBuf::from(pse_codegen::codegen::provenance::ROOT)
        .join(producer)
        .join("associations.json");
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    tree.files.insert(path, bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_traits_and_numbered_accessors_never_guess_names() {
        let syntax=syn::parse_file(r#"
            impl buffa::MessageName for DifferentRustName { const FULL_NAME: &'static str = "test.ActualProto"; }
            impl reflect::ReflectMessage for DifferentRustName { fn get(&self,field:&F)->V { match field.number() { 7u32=>V::String(&self.r#type), 8u32=>match &self.choice { A(v)=>v, B(v)=>v }, _=>false } } }
        "#).unwrap();
        let mut names = BTreeMap::new();
        let mut fields = BTreeMap::new();
        implementations(&syntax.items, "", &mut names, &mut fields);
        assert_eq!(names["DifferentRustName"], "test.ActualProto");
        assert_eq!(fields["DifferentRustName"][&7], "r#type");
        assert_eq!(fields["DifferentRustName"][&8], "choice");
        assert!(!fields["DifferentRustName"].contains_key(&1));
    }
    #[test]
    fn typed_accessor_numbers_accept_radices_and_refuse_unproven_routes() {
        let syntax = syn::parse_file(
            r#"
            impl reflect::ReflectMessage for Actual {
                fn get(&self, field: &F) -> V {
                    match r#field . r#number () {
                        0x7_u32 => &self.r#type,
                        0b1000 => &self.eight,
                        9usize => &self.wrong_suffix,
                        NUMBER => &self.nonliteral,
                        10 if enabled() => &self.guarded,
                        11 => (&self.one, &self.two),
                        12 => &self.first,
                        12u32 => &self.second,
                        _ => false,
                    }
                    match other.number() { 13 => &self.unrelated, _ => false }
                    match field.number(argument) { 14 => &self.argument, _ => false }
                }
            }
        "#,
        )
        .unwrap();
        let mut names = BTreeMap::new();
        let mut fields = BTreeMap::new();
        implementations(&syntax.items, "", &mut names, &mut fields);
        assert_eq!(
            fields["Actual"],
            BTreeMap::from([(7, "r#type".into()), (8, "eight".into())])
        );
    }
}
