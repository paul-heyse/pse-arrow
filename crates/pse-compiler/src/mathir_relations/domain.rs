// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete domain alternatives at the normalized relation/algorithm boundary.

use crate::CompilerError;
use pse_mathir::DomainRef;
use pse_quantity::DomainId;
use pse_relations::generated::normalized;

pub(crate) trait DomainValue: Sized {
    fn from_domain(value: &DomainRef) -> Self;
    fn domain_ref(&self) -> Result<DomainRef, CompilerError>;
}

macro_rules! domain_value {
    ($module:ident, $record:ident, $selected:ident, $actual:ident, $template:ident) => {
        impl DomainValue for normalized::$module::$record {
            fn from_domain(value: &DomainRef) -> Self {
                use normalized::$module::{$actual, $template};
                match value {
                    DomainRef::Actual(domain) => Self::from_actual($actual {
                        domain_id: domain.as_id(),
                    }),
                    DomainRef::Template {
                        template_id,
                        domain_name,
                    } => Self::from_template($template {
                        template_id: *template_id,
                        name: domain_name.clone(),
                    }),
                }
            }

            fn domain_ref(&self) -> Result<DomainRef, CompilerError> {
                use normalized::$module::$selected;
                match self.selected()? {
                    $selected::Actual(value) => {
                        Ok(DomainRef::Actual(DomainId::from_id(value.domain_id)))
                    }
                    $selected::Template(value) => {
                        if value.name.is_empty() {
                            return Err(
                                super::malformed("template domain name must not be empty").into()
                            );
                        }
                        Ok(DomainRef::Template {
                            template_id: value.template_id,
                            domain_name: value.name.clone(),
                        })
                    }
                }
            }
        }
    };
}

domain_value!(
    expression_index_bindings,
    NormalizedExpressionIndexBindingsFieldDomain,
    NormalizedExpressionIndexBindingsFieldDomainSelected,
    NormalizedExpressionIndexBindingsFieldDomainActual,
    NormalizedExpressionIndexBindingsFieldDomainTemplate
);
domain_value!(
    predicate_nodes,
    NormalizedPredicateNodesFieldValueInDomain,
    NormalizedPredicateNodesFieldValueInDomainSelected,
    NormalizedPredicateNodesFieldValueInDomainActual,
    NormalizedPredicateNodesFieldValueInDomainTemplate
);
