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
use protocol::{
    Association, AssociationManifest, SOURCE_RANGE_PRODUCER, SOURCE_RANGE_PROFILE, SourceByteRange,
};
use std::collections::{BTreeMap, BTreeSet};
use syn::ext::IdentExt as _;
use syn::visit::Visit as _;
use tree_sitter::Node;

type MessageFields = BTreeMap<u32, String>;

struct NativeField {
    name: String,
    range: SourceByteRange,
}
struct NativeDeclaration {
    range: SourceByteRange,
    fields: BTreeMap<u32, Vec<NativeField>>,
}
/// Syntax supplies only written declaration intervals. The existing descriptor
/// supplies the file, qualified message identity and numbered field name to join.
struct AuthoredSyntax(BTreeMap<String, Vec<NativeDeclaration>>);
fn child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    node.named_children(&mut node.walk())
        .find(|child| child.kind() == kind)
}
fn node_range(node: Node<'_>) -> SourceByteRange {
    SourceByteRange {
        start: node.start_byte(),
        end: node.end_byte(),
        producer: SOURCE_RANGE_PRODUCER.into(),
        profile: SOURCE_RANGE_PROFILE.into(),
    }
}
fn authored_field_number(node: Node<'_>, source: &[u8]) -> Result<u32> {
    let number = node.utf8_text(source)?;
    let (digits, radix) = if let Some(digits) = number
        .strip_prefix("0x")
        .or_else(|| number.strip_prefix("0X"))
    {
        (digits, 16)
    } else if number.len() > 1 && number.starts_with('0') {
        (&number[1..], 8)
    } else {
        (number, 10)
    };
    Ok(u32::from_str_radix(digits, radix)?)
}
impl AuthoredSyntax {
    fn parse(source: &[u8]) -> Result<Self> {
        std::str::from_utf8(source).context("authored proto is not UTF-8")?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_proto::LANGUAGE.into())?;
        let tree = parser
            .parse(source, None)
            .context("authored syntax parse stopped")?;
        let root = tree.root_node();
        if root.has_error() {
            bail!("authored proto requires native syntax recovery; no intervals qualified");
        }
        let packages = root
            .named_children(&mut root.walk())
            .filter(|node| node.kind() == "package")
            .collect::<Vec<_>>();
        let package = match packages.as_slice() {
            [] => String::new(),
            [package] => {
                let name = child(*package, "full_ident")
                    .context("package has no native full identifier")?;
                name.named_children(&mut name.walk())
                    .filter(|node| node.kind() == "identifier")
                    .map(|node| node.utf8_text(source))
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .join(".")
            }
            _ => bail!("authored package identity is ambiguous"),
        };
        let prefix = if package.is_empty() {
            package
        } else {
            format!("{package}.")
        };
        let mut syntax = Self(BTreeMap::new());
        syntax.messages(root, &prefix, source)?;
        Ok(syntax)
    }
    fn messages(&mut self, parent: Node<'_>, prefix: &str, source: &[u8]) -> Result<()> {
        for message in parent
            .named_children(&mut parent.walk())
            .filter(|node| node.kind() == "message")
        {
            let name = child(message, "message_name")
                .context("native message has no name")?
                .utf8_text(source)?;
            let name = format!("{prefix}{name}");
            let body = child(message, "message_body").context("native message has no body")?;
            let mut fields: BTreeMap<u32, Vec<NativeField>> = BTreeMap::new();
            for field in body
                .named_children(&mut body.walk())
                .filter(|node| matches!(node.kind(), "field" | "map_field"))
            {
                let number = authored_field_number(
                    child(field, "field_number").context("native field has no tag")?,
                    source,
                )?;
                let field_name = child(field, "identifier")
                    .context("native field has no name")?
                    .utf8_text(source)?;
                fields.entry(number).or_default().push(NativeField {
                    name: field_name.into(),
                    range: node_range(field),
                });
            }
            self.0
                .entry(name.clone())
                .or_default()
                .push(NativeDeclaration {
                    range: node_range(message),
                    fields,
                });
            self.messages(body, &format!("{name}."), source)?;
        }
        Ok(())
    }
    fn locate(&self, message: &str, field: Option<(u32, &str)>) -> Result<SourceByteRange> {
        let messages = self
            .0
            .get(message)
            .context("descriptor-qualified message has no supported authored syntax")?;
        let [message] = messages.as_slice() else {
            bail!("descriptor-qualified message has ambiguous authored syntax");
        };
        let Some((number, name)) = field else {
            return Ok(message.range.clone());
        };
        let fields = message
            .fields
            .get(&number)
            .context("descriptor field has no supported authored field syntax")?;
        let [field] = fields.as_slice() else {
            bail!("descriptor field tag has ambiguous authored syntax")
        };
        if field.name != name {
            bail!("authored field name does not match descriptor field name and tag");
        }
        Ok(field.range.clone())
    }
}

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
                        &format!("{module}{}::", item.ident.unraw()),
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
                let syn::Type::Path(self_type) = item.self_ty.as_ref() else {
                    continue;
                };
                if self_type.qself.is_some() || self_type.path.leading_colon.is_some() {
                    continue;
                }
                // The emitted trait identifies the declaration; type/lifetime arguments do
                // not form part of its source-outline path. Normalize identifier identity,
                // while preserving original spelling in the authenticated output bytes.
                let rust_type = format!(
                    "{module}{}",
                    self_type
                        .path
                        .segments
                        .iter()
                        .map(|segment| segment.ident.unraw().to_string())
                        .collect::<Vec<_>>()
                        .join("::")
                );
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

struct AuthoredDeclaration {
    file: String,
    fields: MessageFields,
}
fn messages(set: &FileDescriptorSet) -> BTreeMap<String, AuthoredDeclaration> {
    fn walk(
        output: &mut BTreeMap<String, AuthoredDeclaration>,
        file: &str,
        prefix: &str,
        items: &[buffa_descriptor::generated::descriptor::DescriptorProto],
    ) {
        for item in items {
            let name = format!("{prefix}{}", item.name.as_deref().unwrap_or_default());
            output.insert(
                name.clone(),
                AuthoredDeclaration {
                    file: file.into(),
                    fields: item
                        .field
                        .iter()
                        .filter(|field| {
                            field.oneof_index.is_none() || field.proto3_optional.unwrap_or(false)
                        })
                        .filter_map(|field| {
                            let number = u32::try_from(field.number?).ok()?;
                            Some((number, field.name.as_deref()?.into()))
                        })
                        .collect(),
                },
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
    // Parse each authenticated authored file once, independent of the number of
    // generated Rust types/views and accessor associations it supplies.
    let authored_syntax = inputs
        .files
        .iter()
        .filter(|(path, _)| {
            path.extension()
                .is_some_and(|extension| extension == "proto")
        })
        .map(|(path, bytes)| {
            (
                path.clone(),
                AuthoredSyntax::parse(bytes).map_err(|error| format!("{error:#}")),
            )
        })
        .collect::<BTreeMap<_, _>>();
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
            let Some(declaration) = declarations.get(&name) else {
                continue;
            };
            let file = &declaration.file;
            let fields = &declaration.fields;
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
                source_range: None,
                source_range_unavailable: None,
                output: path.to_string_lossy().into_owned(),
                output_sha256: hash(bytes),
                rust_type: rust_type.clone(),
                rust_field: None,
                state: String::new(),
            };
            let qualify = |mut association: Association, field: Option<(u32, &str)>| {
                let range = authored_syntax
                    .get(Path::new(file))
                    .context("descriptor file has no authenticated native syntax input")
                    .and_then(|syntax| {
                        syntax
                            .as_ref()
                            .map_err(|reason| anyhow::anyhow!("{reason}"))
                    })
                    .and_then(|syntax| syntax.locate(&name, field));
                match range {
                    Ok(range) => association.source_range = Some(range),
                    Err(error) => {
                        association.source_range_unavailable = Some(format!(
                            "{SOURCE_RANGE_PRODUCER} {SOURCE_RANGE_PROFILE}: {error}"
                        ))
                    }
                }
                association
            };
            associations.push(qualify(base.clone(), None));
            if let Some(accessors) = accessors.get(&rust_type) {
                for (number, member) in accessors {
                    if let Some(field_name) = fields.get(number) {
                        let mut field = base.clone();
                        field.declaration = format!("{name}#{number}");
                        field.rust_field = Some(member.clone());
                        associations.push(qualify(field, Some((*number, field_name))));
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
        "Cargo.toml",
        "xtask/Cargo.toml",
        "Cargo.lock",
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
    fn actual_descriptor(source: &str) -> (tempfile::TempDir, Vec<u8>, FileDescriptorSet) {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("buf.yaml"), "version: v2\n").unwrap();
        std::fs::write(directory.path().join("source.proto"), source).unwrap();
        let descriptor = directory.path().join("descriptor.bin");
        run(
            Command::new("buf")
                .args(["build", "--as-file-descriptor-set", "-o"])
                .arg(&descriptor)
                .current_dir(directory.path()),
            "compile actual authored-byte correspondence control",
        )
        .unwrap();
        let bytes = std::fs::read(&descriptor).unwrap();
        let set = FileDescriptorSet::decode_from_slice(&bytes).unwrap();
        (directory, bytes, set)
    }
    #[test]
    fn authored_ranges_follow_actual_buf_tabs_unicode_combining_and_crlf() {
        for newline in ["\n", "\r\n"] {
            let source = [
                "syntax = \"proto3\";",
                "package test;",
                "message ActualProto {",
                "\t/* π */\tstring field = 7;",
                " /* 界 e\u{301} 👩\u{200d}👩 \u{2e3a} \u{2e3b} */ string wide = 8;",
                "\t/* 字 🦀 */\tmap<string, string> labels = 9;",
                " optional string present = 10;",
                " message Same { string inside = 7; }",
                "}",
                "message Other { message Same { string different = 7; } }",
            ]
            .join(newline);
            let (_directory, _bytes, mut set) = actual_descriptor(&source);
            // Authored intervals must not depend on the compiler's SourceInfo
            // coordinate convention, presence or display-width Unicode version.
            for file in &mut set.file {
                file.source_code_info = Default::default();
            }
            let declarations = messages(&set);
            let syntax = AuthoredSyntax::parse(source.as_bytes()).unwrap();
            for (message, tag, expected) in [
                ("test.ActualProto", 7, "string field = 7;"),
                ("test.ActualProto", 8, "string wide = 8;"),
                ("test.ActualProto", 9, "map<string, string> labels = 9;"),
                ("test.ActualProto", 10, "optional string present = 10;"),
                ("test.ActualProto.Same", 7, "string inside = 7;"),
                ("test.Other.Same", 7, "string different = 7;"),
            ] {
                let name = &declarations[message].fields[&tag];
                let range = syntax.locate(message, Some((tag, name))).unwrap();
                assert_eq!(&source[range.start..range.end], expected);
                assert_eq!(range.start, source.find(expected).unwrap());
                assert_eq!(range.producer, SOURCE_RANGE_PRODUCER);
                assert_eq!(range.profile, SOURCE_RANGE_PROFILE);
            }
            let message = syntax.locate("test.ActualProto.Same", None).unwrap();
            assert_eq!(
                &source[message.start..message.end],
                "message Same { string inside = 7; }"
            );
            assert!(
                syntax
                    .locate("test.Other.Same", Some((7, "inside")))
                    .is_err()
            );
        }
    }
    #[test]
    fn native_syntax_recovery_ambiguity_and_unsupported_groups_remain_unknown() {
        assert!(AuthoredSyntax::parse(b"message Broken { string field = ; }").is_err());
        let duplicates = AuthoredSyntax::parse(b"message Same {} message Same {}").unwrap();
        assert!(duplicates.locate("Same", None).is_err());
        let tags = AuthoredSyntax::parse(b"message Same { string a = 7; string b = 7; }").unwrap();
        assert!(tags.locate("Same", Some((7, "a"))).is_err());
        let source = "syntax = \"proto2\"; package test; message Legacy { optional group Grouped = 1 { optional string inner = 2; } optional string ordinary = 3; }";
        let (_directory, _bytes, set) = actual_descriptor(source);
        let declarations = messages(&set);
        let syntax = AuthoredSyntax::parse(source.as_bytes()).unwrap();
        assert_eq!(declarations["test.Legacy"].fields[&1], "grouped");
        assert!(syntax.locate("test.Legacy", Some((1, "grouped"))).is_err());
        assert!(syntax.locate("test.Legacy.Grouped", None).is_err());
        let ordinary = syntax
            .locate(
                "test.Legacy",
                Some((3, &declarations["test.Legacy"].fields[&3])),
            )
            .unwrap();
        assert_eq!(
            &source[ordinary.start..ordinary.end],
            "optional string ordinary = 3;"
        );
    }
    #[test]
    fn actual_descriptor_and_explicit_raw_rust_traits_emit_authenticated_native_ranges() {
        let source = "syntax = \"proto3\"; package test; message ActualProto { /* ⸺ ⸻ 👩\u{200d}👩 */ string field = 7; }";
        let (directory, descriptor, set) = actual_descriptor(source);
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let inputs = Inputs {
            root: directory.path().into(),
            files: BTreeMap::from([(PathBuf::from("source.proto"), source.as_bytes().to_vec())]),
        };
        let mut tree = GeneratedTree::empty(Vec::new());
        tree.files.insert(PathBuf::from("generated.rs"), br#"
            mod r#type {
                impl buffa::MessageName for r#match { const FULL_NAME: &'static str = "test.ActualProto"; }
                impl reflect::ReflectMessage for r#match { fn get(&self, field: &F) -> V { match field.number() { 7 => &self.r#type, _ => false } } }
            }
        "#.to_vec());
        // Source location is checkout-relative; the bytes are authenticated inputs.
        generate(
            root,
            &root.join("authored"),
            &inputs,
            &descriptor,
            &set,
            &mut tree,
        )
        .unwrap();
        let manifest_path = PathBuf::from(pse_codegen::codegen::provenance::ROOT)
            .join("plan61-fixtures/associations.json");
        let manifest: AssociationManifest =
            serde_json::from_slice(&tree.files[&manifest_path]).unwrap();
        let field = manifest
            .associations
            .iter()
            .find(|item| item.declaration == "test.ActualProto#7")
            .unwrap();
        assert_eq!(field.rust_type, "type::match");
        assert_eq!(field.rust_field.as_deref(), Some("r#type"));
        assert!(field.source_range_unavailable.is_none());
        let range = field.source_range.as_ref().unwrap();
        assert_eq!(&source[range.start..range.end], "string field = 7;");
        assert_eq!(
            manifest.descriptor_sha256,
            sha2::Sha256::digest(&descriptor)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
    }
    #[test]
    fn raw_module_and_type_identifiers_match_normalized_outline_identity() {
        let syntax = syn::parse_file(r#"
            mod r#type {
                impl buffa::MessageName for r#match { const FULL_NAME: &'static str = "test.Inner"; }
                impl reflect::ReflectMessage for r#match { fn get(&self, field: &F) -> V { match field.number() { 1 => &self.r#type, _ => false } } }
            }
            impl<'a> buffa::MessageName for r#async<'a> { const FULL_NAME: &'static str = "test.View"; }
        "#).unwrap();
        let mut names = BTreeMap::new();
        let mut fields = BTreeMap::new();
        implementations(&syntax.items, "", &mut names, &mut fields);
        assert_eq!(names["type::match"], "test.Inner");
        assert_eq!(names["async"], "test.View");
        assert_eq!(fields["type::match"][&1], "r#type");
        assert_eq!(names.len(), 2);
    }
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
