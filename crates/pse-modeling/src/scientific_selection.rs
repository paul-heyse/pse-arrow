// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite, context-dependent authored selection closure. Coefficients remain in their records.
use crate::{DeclarationId, Result, invalid, specialize::Value};
use std::collections::{BTreeMap, BTreeSet};

/// The roots and edges that justified one finite structural selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionClosure {
    /// Original explicitly requested record references, in canonical identity order.
    pub roots: Vec<Value>,
    /// The consuming scope passed unchanged to every authored dependency evaluation.
    pub context: Value,
    /// Automatically closed record references, in canonical identity order.
    pub records: Vec<Value>,
    /// Authored dependency edges, including scoped atomic-fit membership.
    pub edges: BTreeMap<Value, BTreeSet<Value>>,
}

/// Close authored dependencies without interpreting scientific family names.
/// The same callback is evaluated once per reached identity, under the unchanged context.
/// # Errors
/// Non-record members, an exceeded finite budget or a dependency evaluator refusal.
pub fn close(
    roots: Vec<Value>,
    context: Value,
    at: DeclarationId,
    limit: usize,
    mut dependencies: impl FnMut(&Value, &Value) -> Result<Vec<Value>>,
) -> Result<SelectionClosure> {
    let roots = roots.into_iter().collect::<BTreeSet<_>>();
    let mut pending = roots.clone();
    let mut records = BTreeSet::new();
    let mut edges = BTreeMap::new();
    while let Some(record) = pending.pop_first() {
        if !matches!(record, Value::Entity { .. }) {
            return Err(invalid(
                at,
                "selection closure contains only record references",
            ));
        }
        if !records.insert(record.clone()) {
            continue;
        }
        if records.len() > limit {
            return Err(invalid(
                at,
                "selection closure exceeds the finite structural budget",
            ));
        }
        let next = dependencies(&record, &context)?
            .into_iter()
            .collect::<BTreeSet<_>>();
        if next.iter().any(|v| !matches!(v, Value::Entity { .. })) {
            return Err(invalid(
                at,
                "selection dependencies contain only record references",
            ));
        }
        for target in &next {
            if !records.contains(target) {
                pending.insert(target.clone());
            }
        }
        if records.len().saturating_add(pending.len()) > limit {
            return Err(invalid(
                at,
                "selection closure exceeds the finite structural budget",
            ));
        }
        edges.insert(record, next);
    }
    Ok(SelectionClosure {
        roots: roots.into_iter().collect(),
        context,
        records: records.into_iter().collect(),
        edges,
    })
}

/// Stable source occurrence of one context-dependent selection operation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectionOccurrence {
    /// Declaration whose static evaluation requested the closure.
    pub declaration: DeclarationId,
    /// Authored dependency callable.
    pub dependencies: DeclarationId,
    /// Canonical spelling distinguishes operations within the same declaration.
    pub expression: String,
    /// Actual requested roots and context distinguish structural applications.
    pub roots: Vec<Value>,
    /// Actual consuming context.
    pub context: Value,
}
/// Immutable selection products retained after static construction.
pub type Selections = BTreeMap<SelectionOccurrence, SelectionClosure>;
/// Construction-local collector, never stored inside an admitted package.
#[derive(Clone, Default)]
pub(crate) struct Collector {
    products: std::rc::Rc<std::cell::RefCell<Selections>>,
    frames: std::rc::Rc<
        std::cell::RefCell<Vec<(ScopeKind, std::rc::Weak<std::cell::RefCell<Consumed>>)>>,
    >,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    Function,
    Instance,
}
#[derive(Default)]
struct Consumed {
    selections: Selections,
    direct_records: BTreeSet<DeclarationId>,
}
/// One active consumer captures its own products. Instance scopes exclude nested owners.
pub(crate) struct SelectionScope {
    collector: Collector,
    products: std::rc::Rc<std::cell::RefCell<Consumed>>,
}
impl Collector {
    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, Selections> {
        self.products.borrow()
    }
    pub(crate) fn into_inner(self) -> Selections {
        match std::rc::Rc::try_unwrap(self.products) {
            Ok(products) => products.into_inner(),
            Err(products) => products.borrow().clone(),
        }
    }
    pub(crate) fn record(&self, occurrence: SelectionOccurrence, closure: SelectionClosure) {
        self.products
            .borrow_mut()
            .insert(occurrence.clone(), closure.clone());
        let frames = self.frames.borrow();
        let owner = frames
            .iter()
            .rposition(|(kind, _)| *kind == ScopeKind::Instance);
        for (index, (kind, frame)) in frames.iter().enumerate() {
            if (*kind == ScopeKind::Function || owner == Some(index))
                && let Some(frame) = frame.upgrade()
            {
                frame
                    .borrow_mut()
                    .selections
                    .insert(occurrence.clone(), closure.clone());
            }
        }
    }
    /// Track numerical attribute consumption; applicability interprets admitted record contracts.
    pub(crate) fn record_numeric(&self, id: DeclarationId) {
        let frames = self.frames.borrow();
        let origin = frames
            .iter()
            .rposition(|(kind, _)| *kind == ScopeKind::Function);
        for (index, (kind, frame)) in frames.iter().enumerate() {
            if *kind == ScopeKind::Function
                && let Some(frame) = frame.upgrade()
            {
                let mut consumed = frame.borrow_mut();
                if origin == Some(index) {
                    consumed.direct_records.insert(id);
                }
            }
        }
    }
    fn open(&self, kind: ScopeKind) -> SelectionScope {
        let products = std::rc::Rc::new(std::cell::RefCell::new(Consumed::default()));
        self.frames
            .borrow_mut()
            .push((kind, std::rc::Rc::downgrade(&products)));
        SelectionScope {
            collector: self.clone(),
            products,
        }
    }
    pub(crate) fn scope(&self) -> SelectionScope {
        self.open(ScopeKind::Function)
    }
    pub(crate) fn instance_scope(&self) -> SelectionScope {
        self.open(ScopeKind::Instance)
    }
    /// Products consumed by the current function, without unrelated prior calls.
    pub(crate) fn current_frame(&self) -> Selections {
        self.frames
            .borrow()
            .iter()
            .rev()
            .find_map(|(kind, frame)| {
                (*kind == ScopeKind::Function)
                    .then(|| frame.upgrade())
                    .flatten()
            })
            .map(|products| products.borrow().selections.clone())
            .unwrap_or_default()
    }
    /// Direct reads belong to the innermost function; nested helpers carry their own claims.
    pub(crate) fn current_direct_records(&self) -> BTreeSet<DeclarationId> {
        self.frames
            .borrow()
            .iter()
            .rev()
            .find_map(|(kind, frame)| {
                (*kind == ScopeKind::Function)
                    .then(|| frame.upgrade())
                    .flatten()
            })
            .map(|products| products.borrow().direct_records.clone())
            .unwrap_or_default()
    }
}
impl SelectionScope {
    pub(crate) fn finish(self) -> Selections {
        std::mem::take(&mut self.products.borrow_mut().selections)
    }
}
impl Drop for SelectionScope {
    fn drop(&mut self) {
        let products = std::rc::Rc::downgrade(&self.products);
        self.collector
            .frames
            .borrow_mut()
            .retain(|(_, frame)| !frame.ptr_eq(&products));
    }
}

impl SelectionClosure {
    /// Frame scientific identities and the edges that justified their selection.
    pub fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.u64(self.roots.len() as u64);
        for record in &self.roots {
            record.frame(hash);
        }
        self.context.frame(hash);
        hash.u64(self.records.len() as u64);
        for record in &self.records {
            record.frame(hash);
        }
        hash.u64(self.edges.len() as u64);
        for (record, dependencies) in &self.edges {
            record.frame(hash);
            hash.u64(dependencies.len() as u64);
            for dependency in dependencies {
                dependency.frame(hash);
            }
        }
    }
}
/// Frame immutable admitted products in source-occurrence order.
pub(crate) fn frame(selections: &Selections, hash: &mut pse_ids::FramedHasher) {
    hash.str("scientific-selections-v1")
        .u64(selections.len() as u64);
    for (occurrence, closure) in selections {
        hash.id(&occurrence.declaration.as_id())
            .id(&occurrence.dependencies.as_id())
            .str(&occurrence.expression);
        closure.frame(hash);
    }
}

/// Retain package-admitted closure products when their actual context is consumed.
pub(crate) fn retain_context(
    package: &crate::CheckedPackage,
    value: &Value,
    collector: Option<&Collector>,
) {
    let Some(collector) = collector else {
        return;
    };
    // A table key may own its context. Follow only declared direct entity attributes.
    let mut owned_contexts = Vec::new();
    if let Value::Entity { id, kind } = value
        && let Some(record) = package.record(*id)
        && let Some(schema) = package.kinds.get(kind)
    {
        for (name, attribute) in &schema.attributes {
            let Some(mut ty) = package.types.get(attribute) else {
                continue;
            };
            if let crate::Type::Optional(inner) = ty {
                ty = inner;
            }
            let crate::Type::Entity(declared) = ty else {
                continue;
            };
            if let Some(candidate @ Value::Entity { kind: actual, .. }) = record.values.get(name)
                && package.refines(*actual, *declared)
            {
                owned_contexts.push(candidate);
            }
        }
    }
    for (occurrence, closure) in &package.selection_closures {
        if &closure.context == value || owned_contexts.contains(&&closure.context) {
            collector.record(occurrence.clone(), closure.clone());
        }
    }
}

/// Structural constructor conformance, preserving optionality and nominal entity ancestry.
pub(crate) fn accepts_type(
    actual: &crate::Type,
    expected: &crate::Type,
    p: &crate::CheckedPackage,
) -> bool {
    use crate::Type;
    match (actual, expected) {
        (a, b) if a == b => true,
        (Type::Optional(a), Type::Optional(b)) => accepts_type(a, b, p),
        (a, Type::Optional(b)) => accepts_type(a, b, p),
        (Type::Entity(a), Type::Entity(b)) => p.refines(*a, *b),
        (Type::Set(a), Type::Set(b)) => accepts_type(a, b, p),
        (Type::Tuple(a), Type::Tuple(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| accepts_type(a, b, p))
        }
        _ => false,
    }
}
