// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Keep one real child per heterogeneous algorithm result. A finite consumer reads
//! several tuple members without copying the producing subtree for every argument.
use super::layout::Layout;
use datafusion::logical_expr::LogicalPlan;
use pse_catalog::session::RelationPlan;
use std::sync::Arc;

/// A native relation value, retaining its shared finite producer when applicable.
#[derive(Clone, Debug)]
pub struct Value {
    pub(super) relation: RelationPlan,
    pub(super) tuple: Option<(Arc<Tuple>, String)>,
}
impl Value {
    /// An arbitrary native relation, including an exactly selected durable source.
    pub const fn from_relation(relation: RelationPlan) -> Self {
        Self {
            relation,
            tuple: None,
        }
    }
    /// The ordinary native relation plan is independently composable and publishable.
    pub const fn relation(&self) -> &RelationPlan {
        &self.relation
    }
}
#[derive(Clone, Debug)]
pub(super) struct Tuple {
    pub plan: LogicalPlan,
    pub layout: Arc<Layout>,
    pub identity: Arc<()>,
}
#[derive(Clone, Debug)]
pub(super) enum Child {
    Relation(Box<RelationPlan>),
    Tuple(Arc<Tuple>),
}
impl Child {
    pub(super) fn plan(&self) -> &LogicalPlan {
        match self {
            Self::Relation(value) => value.plan(),
            Self::Tuple(tuple) => &tuple.plan,
        }
    }
    pub(super) fn rewritten(&self, plan: LogicalPlan) -> Self {
        match self {
            Self::Relation(value) => Self::Relation(Box::new(value.rewritten(plan))),
            Self::Tuple(tuple) => Self::Tuple(Arc::new(Tuple {
                plan,
                ..tuple.as_ref().clone()
            })),
        }
    }
}
#[derive(Clone, Debug)]
pub(super) struct Argument {
    pub role: Option<String>,
    pub child: usize,
    pub member: Option<String>,
}
pub(super) fn bind(
    value: Value,
    role: Option<String>,
    children: &mut Vec<Child>,
    arguments: &mut Vec<Argument>,
) {
    let (child, member) = if let Some((tuple, member)) = value.tuple {
        let existing = children.iter().position(|child| matches!(child, Child::Tuple(other) if Arc::ptr_eq(&other.identity, &tuple.identity)));
        let child = existing.unwrap_or_else(|| {
            let index = children.len();
            children.push(Child::Tuple(tuple));
            index
        });
        (child, Some(member))
    } else {
        let index = children.len();
        children.push(Child::Relation(Box::new(value.relation)));
        (index, None)
    };
    arguments.push(Argument {
        role,
        child,
        member,
    });
}
