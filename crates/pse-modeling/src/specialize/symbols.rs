// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One scalar owner with typed arena coordinates and explicit identity/name lookup indexes.
use super::{Symbol, symbol_name};
use crate::{Type, TypeLookup, resolved::{SymbolIx, TiVec}};
use pse_ids::SemanticId;
use std::{collections::BTreeMap, ops::{Index, IndexMut}, sync::Arc};

#[derive(Clone, Debug, Default, PartialEq)]
struct Table {
    values: TiVec<SymbolIx, Symbol>,
    identities: BTreeMap<SemanticId, SymbolIx>,
    names: BTreeMap<String, SymbolIx>,
}

/// Shared authoritative scalar declarations. Mutation during preparation detaches a shared owner.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Symbols(Arc<Table>);
impl Symbols {
    /// Number of actual scalar declarations.
    pub fn len(&self) -> usize {self.0.values.len()}
    /// Whether no scalar has been realized.
    pub fn is_empty(&self) -> bool {self.0.values.is_empty()}
    /// Actual scalar at its stable arena coordinate.
    pub fn at(&self, slot: SymbolIx) -> Option<&Symbol> {self.0.values.get(slot)}
    /// Actual scalar by its instantiated identity.
    pub fn get(&self, id: &SemanticId) -> Option<&Symbol> {self.slot(*id).and_then(|slot| self.at(slot))}
    pub(crate) fn get_mut(&mut self, id: &SemanticId) -> Option<&mut Symbol> {
        let slot=self.slot(*id)?; Arc::make_mut(&mut self.0).values.get_mut(slot)
    }
    /// Resolve an exact instantiated identity without parsing any spelling.
    pub fn slot(&self, id: SemanticId) -> Option<SymbolIx> {self.0.identities.get(&id).copied()}
    /// Resolve a prepared source-boundary name through its explicit owner index.
    pub fn named(&self, name: &str) -> Option<SymbolIx> {self.0.names.get(name).copied()}
    /// Resolve an exact scalar path through the prepared index, without decoding private names.
    pub fn reference(&self, expression: &pse_authoring::dsl::Expr) -> Option<SemanticId> {
        let pse_authoring::dsl::ExprKind::Path(path) = &expression.kind else {return None;};
        self.path(path)
    }
    /// Resolve a scalar path; indexed source families have a separate finite owner.
    pub fn path(&self, path: &pse_authoring::dsl::Path) -> Option<SemanticId> {
        let name = path.ident()?;
        self.named(name).and_then(|slot| self.at(slot)).map(|symbol| symbol.id)
    }
    /// Whether this actual scalar was realized.
    pub fn contains_key(&self, id: &SemanticId) -> bool {self.0.identities.contains_key(id)}
    /// Declarations in semantic-identity order, preserving inspection order independently of slots.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item=(&SemanticId, &Symbol)> + ExactSizeIterator {
        self.0.identities.iter().map(|(id, slot)| (id, &self.0.values[*slot]))
    }
    /// Instantiated identities in deterministic order.
    pub fn keys(&self) -> impl DoubleEndedIterator<Item=&SemanticId> + ExactSizeIterator {self.0.identities.keys()}
    /// Actual declarations in semantic-identity order.
    pub fn values(&self) -> impl DoubleEndedIterator<Item=&Symbol> + ExactSizeIterator {self.iter().map(|(_, symbol)| symbol)}
    /// Borrow the authoritative typed table.
    pub fn slots(&self) -> &crate::resolved::TiSlice<SymbolIx, Symbol> {&self.0.values}
    /// Install or replace an actual declaration while preserving an existing scalar's slot.
    pub fn insert(&mut self, id: SemanticId, mut symbol: Symbol) -> Option<Symbol> {
        symbol.id = id;
        let table = Arc::make_mut(&mut self.0);
        if let Some(slot) = table.identities.get(&id) {return Some(std::mem::replace(&mut table.values[*slot], symbol));}
        let slot = SymbolIx::from(table.values.len());
        table.values.push(symbol);
        table.identities.insert(id, slot);
        table.names.insert(symbol_name(id), slot);
        None
    }
    /// Install a newly prepared scalar at the next typed slot.
    pub fn push(&mut self, symbol: Symbol) {self.insert(symbol.id, symbol);}
    /// Set the executable definition on this owner's actual scalar.
    pub fn define(&mut self, slot: SymbolIx, root: crate::resolved::NodeIx) -> crate::Result<()> {
        let symbol = Arc::make_mut(&mut self.0).values.get_mut(slot).ok_or_else(|| crate::invalid(SemanticId::NIL, "resolved definition has no scalar owner"))?;
        symbol.definition = Some(root);
        Ok(())
    }
    /// Remap an actual identity without changing its consumed contract or typed slot.
    pub fn rebind(&mut self, slot: SymbolIx, id: SemanticId) -> crate::Result<()> {
        let table = Arc::make_mut(&mut self.0);
        if table.identities.get(&id).is_some_and(|other| *other != slot) {return Err(crate::invalid(id, "scalar remapping collides with another actual member"));}
        let symbol = table.values.get_mut(slot).ok_or_else(|| crate::invalid(id, "scalar remapping has no actual slot"))?;
        table.identities.remove(&symbol.id);
        table.names.remove(&symbol_name(symbol.id));
        symbol.id = id;
        table.identities.insert(id, slot);
        table.names.insert(symbol_name(id), slot);
        Ok(())
    }
    /// Known retained bytes of the single table and its lookup indexes.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Table>().saturating_add(self.0.values.capacity().saturating_mul(size_of::<Symbol>()))
            .saturating_add(self.values().fold(0usize, |bytes, symbol| bytes.saturating_add(crate::resolved::extent::ty(&symbol.ty))
                .saturating_add(symbol.expression.as_ref().map_or(0, crate::extent::expression))
                .saturating_add(symbol.initial.as_ref().map_or(0, super::Value::retained_bytes))
                .saturating_add(crate::extent::lineage(&symbol.lineage))))
            .saturating_add(self.0.identities.len().saturating_mul(size_of::<(SemanticId, SymbolIx)>() + 96))
            .saturating_add(self.0.names.iter().fold(0usize, |bytes, (name, _)| bytes.saturating_add(size_of::<(String, SymbolIx)>() + 96).saturating_add(name.capacity())))
    }
}
impl TypeLookup for Symbols {
    fn get(&self, name: &str) -> Option<&Type> {self.named(name).and_then(|slot| self.at(slot)).map(|symbol| &symbol.ty)}
}
impl Index<&SemanticId> for Symbols {
    type Output = Symbol;
    fn index(&self, id: &SemanticId) -> &Symbol {&self.0.values[self.0.identities[id]]}
}
impl Index<SymbolIx> for Symbols {
    type Output = Symbol;
    fn index(&self, slot: SymbolIx) -> &Symbol {&self.0.values[slot]}
}
impl IndexMut<SymbolIx> for Symbols {
    fn index_mut(&mut self, slot: SymbolIx) -> &mut Symbol {&mut Arc::make_mut(&mut self.0).values[slot]}
}
/// Borrowed deterministic inspection iterator over the authoritative table.
#[derive(Debug)]
pub struct Iter<'a> {table: &'a Table, ids: std::collections::btree_map::Iter<'a, SemanticId, SymbolIx>}
impl<'a> Iterator for Iter<'a> {
    type Item = (&'a SemanticId, &'a Symbol);
    fn next(&mut self) -> Option<Self::Item> {self.ids.next().map(|(id, slot)| (id, &self.table.values[*slot]))}
    fn size_hint(&self) -> (usize, Option<usize>) {self.ids.size_hint()}
}
impl ExactSizeIterator for Iter<'_> {}
impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {self.ids.next_back().map(|(id, slot)| (id, &self.table.values[*slot]))}
}
impl<'a> IntoIterator for &'a Symbols {
    type Item = (&'a SemanticId, &'a Symbol);
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {Iter {table:&self.0, ids:self.0.identities.iter()}}
}

impl Symbol {
    /// Role admitted from the actual authored scalar kind, independent of case-fixed values.
    pub fn local_role(&self) -> crate::derived::LocalRole {
        match self.role {
            pse_model::generated::enums::ModelingDeclarationKind::Variable => crate::derived::LocalRole::Coordinate,
            pse_model::generated::enums::ModelingDeclarationKind::Parameter => crate::derived::LocalRole::Parameter,
            _ => crate::derived::LocalRole::Member,
        }
    }
}
