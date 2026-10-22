// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed syntax traversal and masking from declared source field contracts.
//! A physical string is never sufficient to select a parser or permit masking.
use crate::{
    Registry, SchemaError,
    model::{
        ExpressionGrammar, ExtensionUse, FieldContract, expression::AnnotationSyntax, field_facets,
    },
};
use arrow_schema::DataType;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn grammar(grammar: ExpressionGrammar) -> TokenStream {
    let variant = format_ident!("{grammar:?}");
    quote!(SourceGrammar::#variant)
}
fn metadata<'a>(field: &'a FieldContract, key: &str) -> Option<&'a str> {
    field.field().metadata().get(key).map(String::as_str)
}
fn cases(field: &FieldContract) -> Result<Option<Vec<AnnotationSyntax>>, SchemaError> {
    let cases: Option<Vec<AnnotationSyntax>> = metadata(field, field_facets::EXPRESSION_CASES)
        .map(|text| {
            serde_json::from_str(text)
                .map_err(|error| super::error(format!("invalid source syntax selection: {error}")))
        })
        .transpose()?;
    for case in cases.iter().flatten() {
        let grammar = case
            .grammar
            .as_deref()
            .map(|name| {
                ExpressionGrammar::from_name(name)
                    .filter(|grammar| {
                        !matches!(
                            grammar,
                            ExpressionGrammar::AnnotationArguments
                                | ExpressionGrammar::AnnotationTarget
                        )
                    })
                    .ok_or_else(|| {
                        super::error(format!("unsupported selected source grammar {name}"))
                    })
            })
            .transpose()?;
        if (case.count > 0) != grammar.is_some() {
            return Err(super::error(format!(
                "annotation {} has inconsistent argument grammar/cardinality",
                case.kind
            )));
        }
    }
    Ok(cases)
}
fn contains(field: &FieldContract) -> bool {
    matches!(field.extension(), Some(ExtensionUse::ExprDsl(_)))
        || field.children().iter().any(contains)
}

struct Traversal {
    mutable: bool,
}
impl Traversal {
    fn field(
        &self,
        field: &FieldContract,
        value: TokenStream,
        path: &str,
        indices: &[proc_macro2::Ident],
        present: bool,
    ) -> Result<TokenStream, SchemaError> {
        if !contains(field) {
            return Ok(TokenStream::new());
        }
        if field.nullable() && !present {
            let body = self.field(field, quote!(source_value), path, indices, true)?;
            return Ok(if self.mutable {
                quote!({ let source_optional = #value; if let Some(source_value) = source_optional.as_mut() { #body } })
            } else {
                quote!({ let source_optional = #value; if let Some(source_value) = source_optional.as_ref() { #body } })
            });
        }
        if let Some(ExtensionUse::ExprDsl(declared)) = field.extension() {
            let role = metadata(field, field_facets::EXPRESSION_ROLE).unwrap_or(path);
            let ordinal = metadata(field, field_facets::EXPRESSION_ORDINAL).unwrap_or("list");
            let role_value = if role.contains('{') {
                let count = role
                    .split('{')
                    .skip(1)
                    .map(|part| part.split('}').next().unwrap_or(""))
                    .map(|part| {
                        part.parse::<usize>().map(|n| n + 1).map_err(|error| {
                            super::error(format!("invalid source role template {role}: {error}"))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .max()
                    .unwrap_or(0);
                let arguments = indices.get(..count).ok_or_else(|| {
                    super::error(format!(
                        "source role {role} names an unavailable enclosing ordinal"
                    ))
                })?;
                quote!(let source_role_owned = format!(#role, #(#arguments),*); let source_role = source_role_owned.as_str();)
            } else {
                quote!(let source_role = #role;)
            };
            let position = match ordinal {
                "zero" => quote!(0usize),
                "list" => indices
                    .last()
                    .map_or_else(|| quote!(0usize), |index| quote!(#index)),
                "flatten" => quote!({
                    let next = source_ordinals.entry(source_role.to_owned()).or_default();
                    let position = *next;
                    *next += 1;
                    position
                }),
                _ => {
                    return Err(super::error(format!(
                        "unknown source ordinal policy {ordinal}"
                    )));
                }
            };
            let callback = if self.mutable {
                quote! { let source = #value; if select(source_grammar, source_role, source_position, source) { source.clear(); masked += 1; } }
            } else {
                quote! { visitor.expression(source_grammar, source_role, source_position, #value)?; }
            };
            let body = quote! { #role_value let source_position = #position; #callback };
            return Ok(if declared == ExpressionGrammar::AnnotationArguments {
                quote!({ if let Some(source_grammar) = source_annotation_grammar { let _ = source_grammar; #body } })
            } else {
                let grammar = grammar(if declared == ExpressionGrammar::AnnotationTarget {
                    ExpressionGrammar::Expression
                } else {
                    declared
                });
                quote!({ let source_grammar = #grammar; let _ = source_grammar; #body })
            });
        }
        match field.data_type() {
            DataType::Struct(_) => {
                let children = field.children();
                let selection = cases(field)?;
                let select = selection.as_ref().map(|cases| {
                    let arms = cases.iter().map(|case| {
                        let kind = super::types::ident(&super::types::pascal(&case.kind));
                        let grammar = case.grammar.as_deref().and_then(ExpressionGrammar::from_name)
                            .map_or_else(|| quote!(None), |g| { let g = grammar(g); quote!(Some(#g)) });
                        let target = case.target;
                        quote!(super::super::enums::ModelingAnnotationKind::#kind => (#grammar, #target))
                    });
                    // The semantic relation lives two module levels below generated.
                    quote!(let (source_annotation_grammar, source_annotation_target) = match source_value.kind { #(#arms,)* };)
                });
                let scope = children
                    .iter()
                    .any(|child| metadata(child, field_facets::EXPRESSION_BINDER).is_some());
                let mut bodies = Vec::new();
                // Binder domains must be visited before introducing each name and before
                // any consuming field, regardless of native storage field order.
                for child in
                    children
                        .iter()
                        .filter(|child| metadata(child, field_facets::EXPRESSION_BINDER).is_some())
                        .chain(children.iter().filter(|child| {
                            metadata(child, field_facets::EXPRESSION_BINDER).is_none()
                        }))
                {
                    let name = super::types::ident(child.name());
                    let child_path = if path.is_empty() {
                        child.name().to_owned()
                    } else {
                        format!("{path}.{}", child.name())
                    };
                    let next = if self.mutable {
                        quote!(&mut source_value.#name)
                    } else {
                        quote!(&source_value.#name)
                    };
                    let body = self.field(child, next, &child_path, indices, false)?;
                    bodies.push(if selection.is_some() && child.name() == "target" {
                        quote!(if source_annotation_target { #body })
                    } else {
                        body
                    });
                }
                let enter = (scope && !self.mutable).then(|| quote!(visitor.enter_scope();));
                let leave = (scope && !self.mutable).then(|| quote!(visitor.leave_scope();));
                let mut tokens = value.clone().into_iter();
                let already_bound = matches!(tokens.next(),Some(proc_macro2::TokenTree::Ident(name)) if name=="source_value")
                    && tokens.next().is_none();
                let bind = (!already_bound).then(|| quote!(let source_value=#value;));
                Ok(quote!({ #bind #select #enter #(#bodies)* #leave }))
            }
            DataType::List(_) | DataType::FixedSizeList(_, _) => {
                let children = field.children();
                let child = children.first().ok_or_else(|| {
                    super::error(format!("missing source list element at {path}"))
                })?;
                let index = format_ident!("source_index_{}", indices.len());
                let mut nested = indices.to_vec();
                nested.push(index.clone());
                let body = self.field(child, quote!(source_value), path, &nested, false)?;
                let bind = if metadata(field, field_facets::EXPRESSION_BINDER).is_some()
                    && !self.mutable
                {
                    quote!(visitor.bind_index(&source_value.name, &source_value.domain);)
                } else {
                    TokenStream::new()
                };
                Ok(if self.mutable {
                    quote!({ let source_values = #value; for (#index, source_value) in source_values.iter_mut().enumerate() { #body #bind } })
                } else {
                    quote!({ let source_values = #value; for (#index, source_value) in source_values.iter().enumerate() { #body #bind } })
                })
            }
            _ => Err(super::error(format!(
                "source syntax declared on unsupported field {path}"
            ))),
        }
    }
}

fn annotation(field: &FieldContract, registry: &Registry) -> Result<TokenStream, SchemaError> {
    let Some(cases) = cases(field)? else {
        return Ok(TokenStream::new());
    };
    let kind = registry
        .enum_spec("ModelingAnnotationKind")
        .ok_or_else(|| super::error("missing annotation kinds".into()))?;
    let declared = kind
        .members
        .iter()
        .map(|member| member.name)
        .collect::<std::collections::BTreeSet<_>>();
    let selected = cases
        .iter()
        .map(|case| case.kind.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if declared != selected || selected.len() != cases.len() {
        return Err(super::error(
            "annotation source syntax must select every declared kind exactly once".into(),
        ));
    }
    let members = cases
        .iter()
        .filter_map(|case| case.member.as_deref())
        .collect::<std::collections::BTreeSet<_>>();
    let shapes = cases
        .iter()
        .map(|case| case.shape.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let variants = shapes.iter().map(|shape| {
        let name = super::types::ident(shape);
        if *shape == "Expressions" {
            quote!(#name(usize))
        } else {
            quote!(#name)
        }
    });
    let shape_arms = cases.iter().map(|case| {
        let kind = super::types::ident(&super::types::pascal(&case.kind));
        let shape = super::types::ident(&case.shape);
        let count = case.count;
        let shape = if case.shape == "Expressions" {
            quote!(SourceAnnotationShape::#shape(#count))
        } else {
            quote!(SourceAnnotationShape::#shape)
        };
        quote!(super::super::enums::ModelingAnnotationKind::#kind => #shape)
    });
    let valid_arms = cases.iter().map(|case| {
        let kind = super::types::ident(&super::types::pascal(&case.kind));
        let count = case.count;
        let checks = members.iter().map(|member| {
            let name = super::types::ident(member);
            let required = case.member.as_deref() == Some(*member);
            if required {
                quote!(self.#name.is_some())
            } else {
                quote!(self.#name.is_none())
            }
        });
        let cardinality = if count == 0 {
            quote!(self.arguments.is_empty())
        } else {
            quote!(self.arguments.len()==#count)
        };
        quote!(super::super::enums::ModelingAnnotationKind::#kind => #cardinality #(&& #checks)*)
    });
    let count_arms = cases.iter().map(|case| {
        let kind = super::types::ident(&super::types::pascal(&case.kind));
        let count = case.count;
        quote!(super::super::enums::ModelingAnnotationKind::#kind => #count)
    });
    Ok(quote! {
        /// Typed annotation shape selected by the authoritative kind declaration.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum SourceAnnotationShape { #(#variants,)* }
        impl AuthoredModelingDeclarationsFieldValueAnnotation {
            /// Exact typed argument shape; validation remains attributed by the caller.
            pub fn source_shape(&self) -> SourceAnnotationShape { match self.kind { #(#shape_arms,)* } }
            /// Whether positional cardinality and all typed member presences agree.
            pub fn matches_source_shape(&self) -> bool { match self.kind { #(#valid_arms,)* } }
            /// Exact declared positional cardinality.
            pub fn source_argument_count(&self) -> usize { match self.kind { #(#count_arms,)* } }
        }
    })
}

pub(super) fn render(registry: &Registry) -> Result<TokenStream, SchemaError> {
    let Some(relation) = registry.relation("authored.modeling_declarations") else {
        return Ok(TokenStream::new());
    };
    let value = relation
        .column("value")
        .ok_or_else(|| super::error("modeling declaration has no value".into()))?;
    let annotations = value
        .children()
        .iter()
        .find(|field| field.name() == "annotation")
        .map(|field| annotation(field, registry))
        .transpose()?;
    let immutable = Traversal { mutable: false }.field(value, quote!(self), "", &[], true)?;
    let mutable = Traversal { mutable: true }.field(value, quote!(self), "", &[], true)?;
    let grammar_variants = ExpressionGrammar::ALL
        .into_iter()
        .filter(|grammar| {
            !matches!(
                grammar,
                ExpressionGrammar::AnnotationArguments | ExpressionGrammar::AnnotationTarget
            )
        })
        .map(|grammar| format_ident!("{grammar:?}"))
        .collect::<Vec<_>>();
    Ok(quote! {
        /// Existing source parser selected by a declared field, independent of storage strings.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum SourceGrammar { #(#grammar_variants,)* }
        /// Typed source traversal, with lexical binder lifetime made explicit.
        pub trait SourceExpressionVisitor {
            /// Attributed parser or admission failure.
            type Error;
            /// Admit one exact source occurrence.
            fn expression(&mut self, grammar: SourceGrammar, role: &str, position: usize, source: &str) -> Result<(), Self::Error>;
            /// Save the enclosing index environment.
            fn enter_scope(&mut self);
            /// Restore the enclosing environment after a payload or nested sibling.
            fn leave_scope(&mut self);
            /// Introduce an index only after its domain occurrence has been visited.
            fn bind_index(&mut self, name: &str, domain: &str);
        }
        #annotations
        impl AuthoredModelingDeclarationsFieldValue {
            /// Visit only explicitly declared grammar fields, preserving list ordinals and scopes.
            pub fn visit_source_expressions<V: SourceExpressionVisitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
                let mut source_ordinals = std::collections::BTreeMap::<String, usize>::new();
                #immutable
                Ok(())
            }
            /// Visit each declared source string once and clear only exactly selected occurrences.
            /// Plain labels and scientific cells are never passed to the selector.
            pub fn mask_source_expressions(&mut self, mut select: impl FnMut(SourceGrammar, &str, usize, &str) -> bool) -> usize {
                let mut masked = 0usize;
                let mut source_ordinals = std::collections::BTreeMap::<String, usize>::new();
                #mutable
                masked
            }
        }
    })
}
