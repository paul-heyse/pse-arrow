// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete selected function products with direct typed edges and declared argument order.
use super::*;
use crate::{Function, expression::{admission::{ExpressionAdmissions, ExpressionOccurrence, PhysicalAdmission}, inventory, occurrences::OccurrenceKey}, specialize::{Lineage, Symbol, Symbols, finite::{FiniteFacts, ReductionSelection}}};
use std::collections::{BTreeMap, BTreeSet};

/// One actually selected hard data envelope, independent of source argument spellings.
#[derive(Clone, Debug)]
pub struct FunctionEnvelope {
    /// Actual declaring scientific owner.
    pub owner: DeclarationId,
    /// Complete expected physical type.
    pub ty: Type,
    /// Direct original guard.
    pub predicate: PredicateIx,
    /// Actual data and argument positions consumed by the guard.
    pub reads: crate::envelope::Reads,
}
/// One complete scientific applicability consumer and its resolved observations.
#[derive(Clone, Debug)]
pub struct FunctionUse {
    /// Actual claims, permissions, unions and required dependencies.
    pub node: pse_model::applicability::Node,
    /// Ordered original predicates.
    pub predicates: Vec<PredicateIx>,
    /// Direct observation expressions.
    pub inputs: Vec<NodeIx>,
}
/// An external contract retains complete implementation, shape and derivative semantics.
#[derive(Clone, Debug)]
pub struct FunctionExternal {
    /// Actual selected implementation name, part of its public descriptor contract.
    pub implementation: String,
    /// Complete registered implementation content.
    pub revision: ContentHash,
    /// Actual bound data content.
    pub data: ContentHash,
    /// Exact selected output ordinal.
    pub output: usize,
    /// Actual derivative provenance.
    pub derivative_source: crate::external::DerivativeSource,
    /// Supplied derivative order.
    pub derivatives: u8,
    /// Declared continuity order.
    pub smoothness: u8,
    /// Actual ordered finite input gathers.
    pub shapes: Vec<crate::external::ArgumentShape>,
}
/// Executable function meaning; no normalized textual body is retained here.
#[derive(Clone, Debug)]
pub struct FunctionProduct {
    /// Complete consumed semantic identity, issued once by this producer.
    pub identity: ContentHash,
    /// Original source owner for diagnostics and declared physical authorizations.
    pub declaration: DeclarationId,
    /// One immutable direct syntax owner.
    pub syntax: ResolvedSyntax,
    /// Ordered actual formal slots, including unused declared arguments.
    pub formals: Vec<SymbolIx>,
    /// Ordered nominal call-boundary arguments, before any declared body authorization.
    pub signature: Vec<Type>,
    /// Complete nominal declared result.
    pub result: Type,
    /// Direct body, absent only for an external or synthesized finite-reduction implementation.
    pub body: Option<NodeIx>,
    /// Actual strict form guard.
    pub validity: Option<PredicateIx>,
    /// Original hard data guards in source order.
    pub envelopes: Vec<FunctionEnvelope>,
    /// Complete declared evidence consumers.
    pub applicability: Vec<FunctionUse>,
    /// Argument effects that precede the returned value.
    pub prerequisites: Vec<usize>,
    /// Source-owned physical obligations attached to the exact selected typed operations.
    pub admissions: BTreeMap<NodeIx, PhysicalAdmission>,
    /// Synthesized finite-reduction admission, distinct even when its carrier is empty.
    pub reduction_admission: Option<PhysicalAdmission>,
    /// Independently admitted reduction prototype and physical domain.
    pub reduction: Option<crate::FiniteReduction>,
    /// Exact declared carrier and actually selected members, when source specialization issued it.
    pub reduction_selection: Option<ReductionSelection>,
    /// Declared physical role authorization, never inferred from a numerical signature.
    pub physical_operation: Option<crate::PhysicalOperation>,
    /// Form-layer data and argument lineage.
    pub validity_reads: crate::envelope::Reads,
    /// Actual foreign implementation contract.
    pub external: Option<FunctionExternal>,
    /// Actual claimed piecewise continuity.
    pub continuity: Option<u8>,
}
/// Each selected function is prepared once; callers retain cheap immutable handles.
#[derive(Clone, Debug, Default)]
pub struct FunctionProducts {
    values: BTreeMap<String, Arc<FunctionProduct>>,
}
impl FunctionProducts {
    /// Produce a complete acyclic selection graph from actual specialized contracts.
    pub fn prepare(functions: &BTreeMap<String, Function>, finite: &FiniteFacts, allowance: usize) -> Result<Self> {
        let mut values=Self::default(); let mut active=BTreeSet::new();
        for name in functions.keys() {values.prepare_one(name, functions, finite, allowance, &mut active)?;}
        Ok(values)
    }
    /// Require an actual selected product by its explicit construction-boundary lookup name.
    pub fn get(&self, name: &str) -> Option<&Arc<FunctionProduct>> {self.values.get(name)}
    /// Enumerate actual selected products without deriving meaning from their names.
    pub fn iter(&self) -> impl Iterator<Item=(&str, &Arc<FunctionProduct>)> {self.values.iter().map(|(name, value)| (name.as_str(), value))}
    fn prepare_one(&mut self, name: &str, functions: &BTreeMap<String, Function>, finite: &FiniteFacts, allowance: usize, active: &mut BTreeSet<String>) -> Result<Arc<FunctionProduct>> {
        if let Some(value)=self.values.get(name) {return Ok(value.clone());}
        let function=functions.get(name).ok_or_else(|| invalid(SemanticId::NIL, "selected function product is absent"))?;
        if !active.insert(name.into()) {return Err(invalid(function.id, "resolved selected function cycle"));}
        let mut calls=BTreeSet::new();
        for root in inventory::function(function) {root.expression.walk(|expression| {
            if let ExprKind::NamedCall {name, ..} | ExprKind::Partial {function:name, ..}=&expression.kind {calls.insert(dsl::render_path(name));}
        });}
        let mut selected=BTreeMap::new();
        for call in calls {selected.insert(call.clone(), self.prepare_one(&call, functions, finite, allowance, active)?);}
        let product=Arc::new(FunctionProduct::prepare(function, selected, finite.reduction(name).cloned(), allowance)?);
        active.remove(name); self.values.insert(name.into(), product.clone()); Ok(product)
    }
}
struct FunctionNames {
    symbols: Symbols,
    formals: BTreeMap<String, SymbolIx>,
    functions: BTreeMap<String, FunctionIx>,
}
impl Names for FunctionNames {
    fn selection(&self, path: &dsl::Path) -> Option<Selection> {path.ident().and_then(|name| self.formals.get(name).copied().or_else(|| self.symbols.named(name))).map(Selection::Symbol)}
    fn function(&self, path: &dsl::Path) -> Option<FunctionIx> {self.functions.get(&dsl::render_path(path)).copied()}
    fn provider(&self, _: &str) -> Option<ProviderSelection> {None}
}
impl FunctionProduct {
    fn prepare(function: &Function, selected: BTreeMap<String, Arc<FunctionProduct>>, reduction_selection: Option<ReductionSelection>, allowance: usize) -> Result<Self> {
        let mut syntax=ResolvedSyntax::default(); let mut formals=Vec::new(); let mut names=BTreeMap::new();
        let body_arguments=function.physical_operation.as_ref().map_or_else(
            || function.arguments.clone(), |operation| operation.body_arguments(&function.arguments));
        for (ordinal, (name, ty)) in body_arguments.iter().enumerate() {
            let slot=SymbolIx::from(ordinal);
            let mut hash=FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1); hash.str("local-formal").u64(ordinal as u64);
            syntax.symbols.push(Symbol {id:hash.finish_id(), ty:ty.clone(), role:ModelingDeclarationKind::Variable,
                domain:ModelingVariableDomain::Continuous, definition:None, expression:None, initial:None,
                lineage:Lineage {declaration:function.id, instance:SemanticId::NIL.into(), path:name.clone(), demand:Vec::new(), default_owner:None, is_override:false, presets:Vec::new()}});
            names.insert(name.clone(), slot); formals.push(slot);
            let root=NodeIx::from(syntax.nodes.len()); syntax.nodes.push(Node::Reference(Reference::Symbol(slot))); syntax.outputs.push(root);
        }
        let mut calls=BTreeMap::new();
        for (name, product) in selected {
            let slot=FunctionIx::from(syntax.functions.len());
            syntax.functions.push(FunctionSelection {identity:product.identity, declaration:product.declaration,
                formals:product.formals.iter().map(|slot| product.syntax.symbols[*slot].lineage.path.clone()).collect(), product});
            calls.insert(name, slot);
        }
        let lookup=FunctionNames {symbols:syntax.symbols.clone(), formals:names, functions:calls};
        let key=|role: &str, position| OccurrenceKey {declaration:function.id, role:role.into(), position};
        let mut resolver=Resolver::new(syntax, &lookup, key("function.body",0), allowance);
        resolver.inventory(inventory::function(function).into_iter().map(|root| (key(root.role, root.repeated), root.part, root.expression)), InventoryStage::Specialized)?;
        let body=function.body.as_ref().map(|body| resolver.output(body)).transpose()?;
        let external=if let Some(external)=&function.external {
            let ExprKind::Number(number)=&external.output.kind else {return Err(invalid(function.id,"selected external output is not an exact ordinal"));};
            let output=number.integer().and_then(|value| usize::try_from(value).ok()).filter(|value| *value<=4095).ok_or_else(|| invalid(function.id,"selected external output ordinal outside capacity"))?;
            resolver.source(key("function.external.output",0)); resolver.output(&external.output)?;
            Some(FunctionExternal {implementation:external.implementation.clone(), revision:external.revision, data:external.data, output, derivative_source:external.derivative_source, derivatives:external.derivatives, smoothness:external.smoothness, shapes:external.shapes.clone()})
        } else {None};
        resolver.source(key("function.validity",0));
        let validity=function.validity.as_ref().map(|value| resolver.validity(value)).transpose()?;
        let mut envelopes=Vec::new();
        for (position, guard) in function.envelopes.iter().enumerate() {
            resolver.source(key("function.envelopes",position));
            envelopes.push(FunctionEnvelope {owner:guard.envelope.owner, ty:guard.envelope.ty.clone(), predicate:resolver.validity(&guard.predicate)?, reads:guard.reads.clone()});
        }
        let mut applicability=Vec::new();
        for (position, usage) in function.applicability_uses.iter().enumerate() {
            resolver.source(key("function.applicability_uses",position));
            let predicates=usage.predicates.iter().map(|value| resolver.validity(value)).collect::<Result<_>>()?;
            let inputs=usage.inputs.iter().map(|value| resolver.output(value)).collect::<Result<_>>()?;
            applicability.push(FunctionUse {node:usage.node.clone(), predicates, inputs});
        }
        for (position, value) in function.applicability.iter().enumerate() {resolver.source(key("function.applicability",position)); resolver.output(value)?;}
        let admissions=map_admissions(function.id, &function.physical_admissions, resolver.mapped())?;
        let reduction_admission=function.physical_admissions.get(&ExpressionOccurrence::FiniteReduction).cloned();
        let syntax=resolver.finish();
        let mut result=Self {identity:ContentHash::from_bytes([0;32]), declaration:function.id, syntax, formals, signature:function.arguments.iter().map(|(_,ty)| ty.clone()).collect(), result:function.result.clone(), body, validity, envelopes, applicability,
            prerequisites:function.prerequisites.clone(), admissions, reduction_admission, reduction:function.reduction.clone(), reduction_selection, physical_operation:function.physical_operation.clone(), validity_reads:function.validity_reads.clone(), external, continuity:function.continuity};
        result.identity=result.frame_identity()?; Ok(result)
    }
    fn frame_identity(&self) -> Result<ContentHash> {
        let mut hash=FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1);
        let layout=self.syntax.canonical_layout()?;
        hash.hash(&layout.identity).u64(self.signature.len() as u64);
        for ty in &self.signature {frame_type(ty, &mut hash);}
        hash.u64(self.formals.len() as u64);
        for slot in &self.formals {frame_type(&self.syntax.symbols[*slot].ty, &mut hash);} frame_type(&self.result, &mut hash);
        hash.u64(self.prerequisites.len() as u64); for ordinal in &self.prerequisites {if *ordinal>=self.formals.len() {return Err(invalid(self.declaration,"function prerequisite outside actual signature"));} hash.u64(*ordinal as u64);}
        self.validity_reads.frame(&mut hash); hash.u64(self.envelopes.len() as u64);
        for guard in &self.envelopes {hash.id(&guard.owner.as_id()); frame_type(&guard.ty,&mut hash); guard.reads.frame(&mut hash);}
        hash.u64(self.applicability.len() as u64); for usage in &self.applicability {usage.node.frame(&mut hash); hash.u64(usage.predicates.len() as u64).u64(usage.inputs.len() as u64);}
        hash.bool(self.continuity.is_some()); if let Some(order)=self.continuity {hash.u64(order.into());}
        hash.bool(self.physical_operation.is_some()); if let Some(operation)=&self.physical_operation {operation.frame(&mut hash);}
        hash.bool(self.reduction.is_some()); if let Some(reduction)=&self.reduction {hash.str(reduction.kind.as_str()).bool(reduction.domain.is_some()); if let Some(domain)=reduction.domain {hash.id(&domain.as_id());} reduction.prototype.frame(&mut hash);}
        hash.bool(self.reduction_selection.is_some()); if let Some(selection)=&self.reduction_selection {
            frame_type(&selection.carrier.element, &mut hash); hash.u64(selection.carrier.members.len() as u64); for value in selection.carrier.members.iter() {value.frame(&mut hash);}
            hash.u64(selection.selected.len() as u64); for value in selection.selected.iter() {value.frame(&mut hash);}
        }
        hash.bool(self.external.is_some()); if let Some(external)=&self.external {
            hash.str(&external.implementation).hash(&external.revision).hash(&external.data).u64(external.output as u64).str(external.derivative_source.as_str()).u64(external.derivatives.into()).u64(external.smoothness.into());
            hash.u64(external.shapes.len() as u64); for shape in &external.shapes {hash.id(&shape.argument).u64(shape.start as u64).u64(shape.axes.len() as u64); for axis in &shape.axes {hash.id(axis);} hash.u64(shape.coordinates.len() as u64); for tuple in &shape.coordinates {hash.u64(tuple.len() as u64); for value in tuple {hash.id(value);}}}
        }
        // Attach proofs to the exact operation ordinal issued by this same framing walk.
        let ordinals=layout.nodes.into_iter().enumerate().map(|(ordinal,slot)| (slot,ordinal)).collect::<BTreeMap<_,_>>();
        let mut admissions=self.admissions.iter().map(|(slot,admission)| {
            ordinals.get(slot).copied().map(|ordinal| (ordinal,admission)).ok_or_else(|| invalid(self.declaration,"function admission is outside its reached typed graph"))
        }).collect::<Result<Vec<_>>>()?;
        admissions.sort_by_key(|(ordinal,_)| *ordinal);
        hash.u64(admissions.len() as u64); for (ordinal,admission) in admissions {hash.u64(ordinal as u64); admission.frame(&mut hash);}
        hash.bool(self.reduction_admission.is_some()); if let Some(admission)=&self.reduction_admission {admission.frame(&mut hash);}
        Ok(hash.finish_hash())
    }
}
fn map_admissions(at: DeclarationId, admissions: &ExpressionAdmissions, mapped: &BTreeMap<ExpressionOccurrence,NodeIx>) -> Result<BTreeMap<NodeIx,PhysicalAdmission>> {
    admissions.iter().filter(|(occurrence,_)| **occurrence!=ExpressionOccurrence::FiniteReduction).map(|(occurrence,admission)| {
        let slot=mapped.get(occurrence).copied().ok_or_else(|| invalid(at,"physical admission has no actual resolved operation occurrence"))?;
        Ok((slot,admission.clone()))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn selected(body: &str, formal_names: (&str, &str)) -> (BTreeMap<String, Function>, FiniteFacts) {
        let (registry,_)=crate::kernel_types::physical();
        let facts=pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope=crate::PhysicalScope::default();
        let context=crate::TypeContext {admissions:None, formula_authority:None, quantities:&registry, preconditions:&facts, scope:&scope};
        let text=format!("package p {{fn pick({},{})->Scalar={body}; def Root {{var x:Scalar; var y:Scalar; eq e:pick(x,y)==1;}}}}",format!("{}:Scalar",formal_names.0),format!("{}:Scalar",formal_names.1));
        let package=crate::check(&crate::kernel_types::try_source(&text).unwrap(),&context).unwrap();
        let model=crate::specialize(&package,package.names["p.Root"],SemanticId::from_bytes([9;16]).into(),&crate::Bindings::default(),crate::Limits::default()).unwrap();
        (model.functions,model.finite_facts)
    }
    fn identity(body: &str, names: (&str,&str)) -> ContentHash {
        let (functions,finite)=selected(body,names);
        let products=FunctionProducts::prepare(&functions,&finite,1000).unwrap();
        products.iter().find(|(_,product)| product.formals.len()==2).unwrap().1.identity
    }
    #[test]
    fn selected_function_products_preserve_formal_ordinals_and_ignore_source_spelling() {
        assert_eq!(identity("a+b",("a","b")),identity("left+right",("left","right")));
        assert_ne!(identity("a",("a","b")),identity("b",("a","b")));
        assert_ne!(identity("a+b",("a","b")),identity("a-b",("a","b")));
    }
    #[test]
    fn selected_function_admissions_map_to_the_actual_lexical_inventory() {
        let (functions,finite)=selected("sum+sum where sum=a+b",("a","b"));
        let products=FunctionProducts::prepare(&functions,&finite,1000).unwrap();
        let product=products.iter().find(|(_,product)| product.formals.len()==2).unwrap().1;
        assert!(product.admissions.len()>=2);
        assert!(product.admissions.keys().all(|slot| product.syntax.node_sources[slot].coordinate.is_some_and(|coordinate| coordinate.stage==InventoryStage::Specialized)));
        let roots=product.syntax.select(vec![product.body.unwrap()],vec![]);
        assert!(Arc::ptr_eq(&roots.arena,&product.syntax.arena));
        assert!(std::ptr::eq(&roots.symbols[product.formals[0]].ty,&product.syntax.symbols[product.formals[0]].ty));
    }
}
