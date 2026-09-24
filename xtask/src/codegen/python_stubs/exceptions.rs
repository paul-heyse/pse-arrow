// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use anyhow::{Result, bail, ensure};
use pyo3_introspection::model::{Attribute, Class, Constant, Expr, Operator};
use std::{fs, path::Path};
use syn::{
    Field, GenericArgument, Ident, Item, PathArguments, Token, Type,
    parse::{Parse, ParseStream},
};

struct Declaration {
    name: Ident,
    base: Ident,
    fields: Vec<Field>,
}
impl Parse for Declaration {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let base = input.parse()?;
        input.parse::<Token![,]>()?;
        let content;
        syn::braced!(content in input);
        let fields = content
            .parse_terminated(Field::parse_named, Token![,])?
            .into_iter()
            .collect();
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        Ok(Self { name, base, fields })
    }
}
pub(super) fn read(path: &Path) -> Result<Vec<Class>> {
    declarations(&fs::read_to_string(path)?)
}
fn declarations(source: &str) -> Result<Vec<Class>> {
    syn::parse_file(source)?
        .items
        .into_iter()
        .filter_map(|item| {
            let Item::Macro(item) = item else {
                return None;
            };
            item.mac
                .path
                .is_ident("inspection_exception")
                .then_some(item.mac.tokens)
        })
        .map(|tokens| class(syn::parse2(tokens)?))
        .collect()
}
fn name(id: &str) -> Expr {
    Expr::Name { id: id.to_owned() }
}
fn class(declaration: Declaration) -> Result<Class> {
    // This is the actual native PyO3 base named in the runtime macro invocation.
    ensure!(
        declaration.base == "PyException",
        "unsupported native exception base"
    );
    let attributes = declaration
        .fields
        .into_iter()
        .map(|field| {
            Ok(Attribute {
                name: field
                    .ident
                    .ok_or_else(|| anyhow::anyhow!("unnamed diagnostic field"))?
                    .to_string(),
                annotation: Some(annotation(&field.ty)?),
                value: None,
                docstring: None,
            })
        })
        .collect::<Result<_>>()?;
    Ok(Class {
        name: declaration.name.to_string(),
        bases: vec![name("Exception")],
        methods: Vec::new(),
        attributes,
        decorators: Vec::new(),
        inner_classes: Vec::new(),
        docstring: None,
    })
}
fn annotation(ty: &Type) -> Result<Expr> {
    match ty {
        Type::Tuple(tuple) => Ok(Expr::Subscript {
            value: Box::new(name("tuple")),
            slice: Box::new(Expr::Tuple {
                elts: tuple.elems.iter().map(annotation).collect::<Result<_>>()?,
            }),
        }),
        Type::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            let segment = &path.path.segments[0];
            match (&*segment.ident.to_string(), &segment.arguments) {
                ("String", PathArguments::None) => Ok(name("str")),
                ("DiagnosticReport", PathArguments::None) => Ok(Expr::Attribute {
                    value: Box::new(name("_native")),
                    attr: "DiagnosticReport".into(),
                }),
                ("Option" | "Vec", PathArguments::AngleBracketed(arguments))
                    if arguments.args.len() == 1 =>
                {
                    let Some(GenericArgument::Type(inner)) = arguments.args.first() else {
                        bail!("exception field needs a concrete type");
                    };
                    let inner = annotation(inner)?;
                    Ok(if segment.ident == "Option" {
                        Expr::BinOp {
                            left: Box::new(inner),
                            op: Operator::BitOr,
                            right: Box::new(Expr::Constant {
                                value: Constant::None,
                            }),
                        }
                    } else {
                        Expr::Subscript {
                            value: Box::new(name("list")),
                            slice: Box::new(inner),
                        }
                    })
                }
                _ => bail!("unsupported native diagnostic field type"),
            }
        }
        _ => bail!("unsupported native diagnostic field shape"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_stub_is_derived_from_the_typed_runtime_macro() {
        let source = "inspection_exception!(ReadFailure, PyException, { code: Option<String>, related: Vec<(Option<String>, String)>, });";
        let classes = declarations(source).unwrap();
        assert_eq!(classes[0].name, "ReadFailure");
        assert_eq!(classes[0].attributes.len(), 2);
        assert_eq!(classes[0].attributes[1].name, "related");
        assert!(declarations(&source.replace("Option<String>", "Option<usize>")).is_err());
        let named = declarations(
            "inspection_exception!(InspectionError, PyException, { report: DiagnosticReport, });",
        )
        .unwrap();
        assert_eq!(
            named[0].attributes[0].annotation,
            Some(Expr::Attribute {
                value: Box::new(name("_native")),
                attr: "DiagnosticReport".into()
            })
        );
    }
}
