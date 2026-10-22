// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Construction from already selected typed edges. This module never resolves an AST.
use super::*;
use crate::expression::{admission::ExpressionOccurrence, occurrences::OccurrenceKey};

/// Inspection syntax and its executable root travel together during specialization.
#[derive(Debug)]
pub(crate) struct Rewritten {
    pub(crate) inspection: Expr,
    pub(crate) root: NodeIx,
}

/// A predicate retains its inspection syntax independently of its executable edges.
#[derive(Debug)]
pub(crate) struct RewrittenPredicate {
    pub(crate) inspection: Predicate,
    pub(crate) root: PredicateIx,
}

/// An actual selected relation, including its original orientation and equation sense.
#[derive(Debug)]
pub(crate) struct RewrittenEquation {
    pub(crate) inspection: dsl::Equation,
    pub(crate) lhs: NodeIx,
    pub(crate) sense: dsl::EquationSense,
    pub(crate) rhs: NodeIx,
}

/// Temporary exact source correspondence, created before the owning rewrite changes syntax.
/// Borrowing the source roots prevents addresses from being reused during this inventory.
#[derive(Debug)]
pub(crate) struct OriginalInventory<'a> {
    sources: BTreeMap<usize, Source>,
    _owner: std::marker::PhantomData<&'a Expr>,
}
impl<'a> OriginalInventory<'a> {
    pub(crate) fn new(roots: impl IntoIterator<Item=(OccurrenceKey, usize, &'a Expr)>, allowance: usize) -> Result<Self> {
        let mut sources=BTreeMap::new();
        for (field, part, root) in roots {
            let mut position=0;
            let mut conflict=false;
            let mut exhausted=false;
            root.walk(|expression| {
                if sources.len()>=allowance {exhausted=true; return;}
                let source=Source {field:field.clone(), span:expression.span,
                    coordinate:Some(InventoryCoordinate {stage:InventoryStage::Original,
                        occurrence:ExpressionOccurrence::Node {body:part, position}})};
                conflict |= sources.insert(std::ptr::from_ref(expression) as usize, source).is_some();
                position+=1;
            });
            if exhausted {return Err(crate::ModelingError::Budget("original occurrence inventory allowance exceeded".into()));}
            if conflict {return Err(invalid(field.declaration,"original source roots overlap"));}
        }
        Ok(Self {sources, _owner:std::marker::PhantomData})
    }
    pub(crate) fn source(&self, expression: &Expr) -> Result<Source> {
        self.sources.get(&(std::ptr::from_ref(expression) as usize)).cloned()
            .ok_or_else(|| invalid(SemanticId::NIL,"rewritten expression lacks its actual original source occurrence"))
    }
}

/// Mutable only while one region is constructed. The scalar table is attached once at finish,
/// so allocating a scalar cannot detach and copy a second shared symbol table on every insert.
#[derive(Debug)]
pub(crate) struct Construction {
    arena: Arena,
    // Construction-local lookup only. The owning Arcs remain in arena.functions until finish.
    // These addresses never enter a semantic frame or a published product.
    functions: BTreeMap<usize, FunctionIx>,
    allowance: usize,
    used: usize,
}
impl Construction {
    pub(crate) fn new(allowance: usize) -> Self {Self {arena:Arena::default(), functions:BTreeMap::new(), allowance, used:0}}
    fn reserve(&mut self, amount: usize) -> Result<()> {
        let used=self.used.checked_add(amount).filter(|used| *used<=self.allowance)
            .ok_or_else(|| crate::ModelingError::Budget("resolved construction allowance exceeded".into()))?;
        self.used=used; Ok(())
    }
    fn node_edge(&self, slot: NodeIx) -> Result<()> {
        self.arena.nodes.get(slot).map(|_| ()).ok_or_else(|| invalid(SemanticId::NIL,"construction operation edge is outside its owner"))
    }
    fn predicate_edge(&self, slot: PredicateIx) -> Result<()> {
        self.arena.predicates.get(slot).map(|_| ()).ok_or_else(|| invalid(SemanticId::NIL,"construction predicate edge is outside its owner"))
    }
    fn binder_edge(&self, slot: BinderIx) -> Result<()> {
        self.arena.binders.get(slot).map(|_| ()).ok_or_else(|| invalid(SemanticId::NIL,"construction binding edge is outside its owner"))
    }
    fn carrier_edge(&self, slot: CarrierIx) -> Result<()> {
        self.arena.carriers.get(slot).map(|_| ()).ok_or_else(|| invalid(SemanticId::NIL,"construction carrier edge is outside its owner"))
    }
    pub(crate) fn node(&mut self, node: Node, source: Source) -> Result<NodeIx> {
        let edges=match &node {
            Node::Reference(Reference::Indexed {indices,..}) => indices.len(),
            Node::Call {args,..} | Node::Apply {args,..} | Node::Provider {args,..} => args.len(),
            Node::Partial {args,wrt,..} => args.len().saturating_add(wrt.len()),
            Node::Let {bindings,..} => bindings.len().saturating_add(1),
            Node::Binary {..} => 2,
            Node::Conditional {..} => 3,
            Node::Reduce {filter,..} => 2+usize::from(filter.is_some()),
            Node::Fold {filter,..} => 5+usize::from(filter.is_some()),
            Node::Derivative {..} => 2,
            Node::Neg(_) | Node::Reference(Reference::Lexical(_)) => 1,
            Node::Number(_) | Node::Reference(Reference::Symbol(_)) => 0,
        };
        match &node {
            Node::Number(value) if !value.value.is_finite() => return Err(invalid(source.field.declaration,"nonfinite constructed literal")),
            Node::Number(_) | Node::Reference(Reference::Symbol(_)) => {}
            Node::Reference(Reference::Lexical(binding)) => self.binder_edge(*binding)?,
            Node::Reference(Reference::Indexed {group,indices}) => {
                if self.arena.groups.get(*group).is_none() {return Err(invalid(source.field.declaration,"construction indexed family is outside its owner"));}
                for index in indices {self.node_edge(*index)?;}
            }
            Node::Neg(value) => self.node_edge(*value)?,
            Node::Binary {lhs,rhs,..} => {self.node_edge(*lhs)?; self.node_edge(*rhs)?;}
            Node::Call {args,..} | Node::Provider {args,..} => {for arg in args {self.node_edge(*arg)?;}}
            Node::Apply {function,args} | Node::Partial {function,args,..} => {
                let selected=self.arena.functions.get(*function).ok_or_else(|| invalid(source.field.declaration,"construction function is outside its owner"))?;
                if selected.product.signature.len()!=args.len() {return Err(invalid(source.field.declaration,"constructed function argument arity differs"));}
                if let Node::Partial {wrt,..}=&node
                    && wrt.iter().any(|ordinal| *ordinal>=args.len()) {return Err(invalid(source.field.declaration,"constructed partial is outside the ordered signature"));}
                for arg in args {self.node_edge(*arg)?;}
            }
            Node::Reduce {binder,filter,body,..} => {
                self.binder_edge(*binder)?; if let Some(filter)=filter {self.predicate_edge(*filter)?;} self.node_edge(*body)?;
            }
            Node::Fold {binder,filter,value,accumulator,item,step} => {
                for binder in [binder,accumulator,item] {self.binder_edge(*binder)?;}
                if let Some(filter)=filter {self.predicate_edge(*filter)?;}
                self.node_edge(*value)?; self.node_edge(*step)?;
            }
            Node::Derivative {body,domain} => {self.node_edge(*body)?; self.carrier_edge(*domain)?;}
            Node::Conditional {guard,then,otherwise} => {self.predicate_edge(*guard)?; self.node_edge(*then)?; self.node_edge(*otherwise)?;}
            Node::Let {bindings,body} => {for binding in bindings {self.binder_edge(*binding)?;} self.node_edge(*body)?;}
        }
        self.reserve(edges.saturating_add(1))?;
        let slot=NodeIx::from(self.arena.nodes.len());
        self.arena.nodes.push(node); self.arena.node_sources.insert(slot,source); Ok(slot)
    }
    pub(crate) fn pair(&mut self, node: Node, inspection: ExprKind, source: Source) -> Result<Rewritten> {
        let span=source.span;
        Ok(Rewritten {root:self.node(node,source)?, inspection:Expr {kind:inspection,span}})
    }
    pub(crate) fn predicate(&mut self, predicate: ResolvedPredicate, source: Source) -> Result<PredicateIx> {
        match &predicate {
            ResolvedPredicate::Compare {lhs,rhs,..} => {self.node_edge(*lhs)?; self.node_edge(*rhs)?;}
            ResolvedPredicate::In {value,domain} => {self.node_edge(*value)?; self.carrier_edge(*domain)?;}
            ResolvedPredicate::And(lhs,rhs) | ResolvedPredicate::Or(lhs,rhs) => {self.predicate_edge(*lhs)?; self.predicate_edge(*rhs)?;}
            ResolvedPredicate::Not(value) => self.predicate_edge(*value)?,
            ResolvedPredicate::Atom(value) => self.node_edge(*value)?,
            ResolvedPredicate::Bool(_) | ResolvedPredicate::Null => {}
        }
        self.reserve(match &predicate {
            ResolvedPredicate::Compare {..} | ResolvedPredicate::In {..} | ResolvedPredicate::And(..) | ResolvedPredicate::Or(..) => 3,
            ResolvedPredicate::Not(_) | ResolvedPredicate::Atom(_) => 2,
            ResolvedPredicate::Bool(_) | ResolvedPredicate::Null => 1,
        })?;
        let slot=PredicateIx::from(self.arena.predicates.len());
        self.arena.predicates.push(predicate); self.arena.predicate_sources.insert(slot,source); Ok(slot)
    }
    pub(crate) fn binding(&mut self, binding: Binding) -> Result<BinderIx> {
        match binding {Binding::Value(value)=>self.node_edge(value)?, Binding::Coordinate(carrier)=>self.carrier_edge(carrier)?, Binding::Accumulator | Binding::Item=>{}}
        self.reserve(1+usize::from(matches!(binding,Binding::Value(_) | Binding::Coordinate(_))))?;
        let slot=BinderIx::from(self.arena.binders.len()); self.arena.binders.push(binding); Ok(slot)
    }
    pub(crate) fn function(&mut self, product: Arc<FunctionProduct>) -> Result<FunctionIx> {
        let owner=Arc::as_ptr(&product) as usize;
        if let Some(slot)=self.functions.get(&owner) {return Ok(*slot);}
        self.reserve(product.formals.len().saturating_add(1))?;
        let slot=FunctionIx::from(self.arena.functions.len());
        self.arena.functions.push(FunctionSelection {identity:product.identity,declaration:product.declaration,
            formals:product.formals.iter().map(|slot| product.syntax.symbols[*slot].lineage.path.clone()).collect(),product});
        self.functions.insert(owner,slot); Ok(slot)
    }
    pub(crate) fn carrier(&mut self, carrier: Carrier) -> Result<CarrierIx> {
        self.reserve(carrier.members.len().saturating_add(1))?;
        let slot=CarrierIx::from(self.arena.carriers.len()); self.arena.carriers.push(carrier); Ok(slot)
    }
    pub(crate) fn group(&mut self, group: IndexedGroup) -> Result<GroupIx> {
        for axis in &group.axes {self.carrier_edge(*axis)?;}
        let mut seen=std::collections::BTreeSet::new();
        for (tuple,_) in &group.members {
            if tuple.len()!=group.axes.len() || !seen.insert(tuple) {return Err(invalid(SemanticId::NIL,"constructed indexed family has a repeated tuple or wrong rank"));}
        }
        self.reserve(group.members.iter().fold(group.axes.len().saturating_add(1),|used,(tuple,_)| used.saturating_add(tuple.len()).saturating_add(1)))?;
        let slot=GroupIx::from(self.arena.groups.len()); self.arena.groups.push(group); Ok(slot)
    }
    pub(crate) fn finish(self, symbols: crate::specialize::Symbols, outputs: Vec<NodeIx>, validity: Vec<PredicateIx>) -> Result<ResolvedSyntax> {
        for root in &outputs {self.node_edge(*root)?;}
        for root in &validity {self.predicate_edge(*root)?;}
        for node in &self.arena.nodes {
            if let Node::Reference(Reference::Symbol(slot))=node
                && symbols.at(*slot).is_none() {return Err(invalid(SemanticId::NIL,"constructed scalar edge is outside its authoritative table"));}
        }
        for group in &self.arena.groups {for (_,member) in &group.members {
            if member.is_some_and(|slot| symbols.at(slot).is_none()) {return Err(invalid(SemanticId::NIL,"constructed indexed member is outside its authoritative table"));}
        }}
        Ok(ResolvedSyntax {symbols,arena:Arc::new(self.arena),outputs,validity})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn field() -> OccurrenceKey {OccurrenceKey {declaration:SemanticId::from_bytes([3;16]).into(),role:"binding.expression".into(),position:0}}
    #[test]
    fn original_coordinates_are_issued_before_rewriting_and_never_guessed_from_clones() {
        let expression=dsl::parse_expr("a+a where a=2").unwrap();
        let inventory=OriginalInventory::new([(field(),0,&expression)],5).unwrap();
        let mut sources=Vec::new(); expression.walk(|node| sources.push(inventory.source(node).unwrap()));
        assert_eq!(sources.len(),5);
        for (position,source) in sources.iter().enumerate() {
            assert_eq!(source.coordinate,Some(InventoryCoordinate {stage:InventoryStage::Original,occurrence:ExpressionOccurrence::Node {body:0,position}}));
        }
        assert!(inventory.source(&expression.clone()).is_err());
        assert!(OriginalInventory::new([(field(),0,&expression),(field(),1,&expression)],10).is_err());
        assert!(OriginalInventory::new([(field(),0,&expression)],4).is_err());
    }
    #[test]
    fn direct_construction_retains_supplied_edges_and_enforces_owner_and_allowance() {
        let expression=dsl::parse_expr("2").unwrap();
        let inventory=OriginalInventory::new([(field(),0,&expression)],1).unwrap();
        let source=inventory.source(&expression).unwrap();
        let ExprKind::Number(number)=&expression.kind else {panic!()};
        let mut construction=Construction::new(3);
        assert!(construction.node(Node::Neg(NodeIx::from(0)),source.clone()).is_err());
        let literal=construction.pair(Node::Number(number.clone()),expression.kind.clone(),source.clone()).unwrap();
        let root=construction.node(Node::Neg(literal.root),source.clone()).unwrap();
        assert!(construction.node(Node::Neg(root),source).is_err());
        let syntax=construction.finish(crate::specialize::Symbols::default(),vec![root],vec![]).unwrap();
        assert_eq!(syntax.nodes[root],Node::Neg(literal.root));
        assert_eq!(literal.inspection,expression);
        assert_eq!(syntax.node_sources[&literal.root].coordinate.unwrap().stage,InventoryStage::Original);
    }
}
