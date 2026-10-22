// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned expression derivation over local typed premises, imported at root assembly.
use super::*;
use crate::resolved::{NodeIx, SymbolIx, fragment::{CapturePremises, Fragment, ImportedFragment}};
use std::sync::Arc;

/// Complete local source and typed premises, without a root arena.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldFragmentInput {
    package: Arc<CheckedPackage>,
    recipe: Arc<FieldFragmentRecipe>,
}
/// Exact derivation fields excluding the independently shared checked package.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldFragmentRecipe {
    instance: InstanceId,
    at: DeclarationId,
    root: DeclarationId,
    fixture: bool,
    expression: Expr,
    sources: construction::ExpressionSources,
    env: Environment,
    chain: Vec<DeclarationId>,
    limits: Limits,
    states: BTreeMap<InstanceId, State>,
    foreign: CapturePremises,
    context: SpecializedModel,
    numerical_sources: BTreeMap<(InstanceId, String), NumericalSource>,
    numerical_source_stack: Vec<(InstanceId, String)>,
    set_alias_sources: BTreeMap<String, NumericalSource>,
    source_locals: BTreeSet<String>,
    temporal_owners: BTreeMap<InstanceId, InstanceId>,
    temporal_axis: Option<SemanticId>,
    stack: Vec<DeclarationId>,
    preset_stack: Vec<DeclarationId>,
    facts: Environment,
    permissions: Vec<pse_model::applicability::Permission>,
    continuity_done: BTreeSet<SemanticId>,
}
impl Eq for FieldFragmentRecipe {}
impl std::hash::Hash for FieldFragmentRecipe {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.instance.hash(state); self.at.hash(state); self.env.hash(state); }
}
impl FieldFragmentInput {
    /// Share package ownership independently of local construction fields.
    pub fn parts(&self) -> (Arc<CheckedPackage>, Arc<FieldFragmentRecipe>) { (self.package.clone(), self.recipe.clone()) }
    /// Reassemble a tracked package owner and local immutable recipe.
    pub fn from_parts(package: Arc<CheckedPackage>, recipe: Arc<FieldFragmentRecipe>) -> Self { Self { package, recipe } }
}
impl FieldFragmentRecipe {
    /// Conservative known owned extent, excluding separately counted package storage.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() + crate::extent::expression(&self.expression) + self.sources.retained_bytes()
            + self.foreign.retained_bytes() + self.context.retained_bytes()
            + environment(&self.env) + environment(&self.facts)
            + self.states.values().map(state_extent).sum::<usize>()
            + self.numerical_sources.values().chain(self.set_alias_sources.values()).map(|source| source.role.capacity() + crate::extent::expression(&source.expression) + environment(&source.env) + 256).sum::<usize>()
            + self.source_locals.iter().map(|value| value.capacity()+128).sum::<usize>()
            + self.numerical_source_stack.iter().map(|(_,name)|name.capacity()+128).sum::<usize>()
            + self.temporal_owners.len()*128 + (self.chain.capacity()+self.stack.capacity()+self.preset_stack.capacity())*size_of::<DeclarationId>()
            + self.continuity_done.len()*128
            + self.permissions.iter().map(|permission| permission.targets.capacity()*size_of::<SemanticId>()+256).sum::<usize>()
    }
}
fn environment(value: &Environment) -> usize { value.iter().map(|(name,value)|name.capacity()+value.retained_bytes()+128).sum() }
fn state_extent(value: &State) -> usize {
    size_of::<State>() + environment(&value.env)+environment(&value.scope)+value.path.capacity()
        + value.members.keys().map(|name|name.capacity()+128).sum::<usize>() + value.member_kinds.len()*128
        + value.children.keys().map(|(name,coordinates)|name.capacity()+coordinates.capacity()*size_of::<SemanticId>()+128).sum::<usize>()
        + value.symbols.keys().map(|(_,coordinates)|coordinates.capacity()*size_of::<SemanticId>()+128).sum::<usize>()
        + crate::extent::selections(&value.selections)+value.presets.capacity()*size_of::<DeclarationId>()+value.stack.capacity()*size_of::<DeclarationId>()
}
/// Immutable typed local expression and its original lazy member effects.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldFragment {
    typed: Fragment,
    inspection: Expr,
    model: SpecializedModel,
    states: BTreeMap<InstanceId, StateEffects>,
    bindings: Vec<SemanticId>,
    selections: crate::scientific_selection::Selections,
    finite: finite::FiniteFacts,
    original_guards: Vec<OriginalGuard>,
    construction_items: usize,
    continuity_done: BTreeSet<SemanticId>,
}
#[derive(Clone,Debug,PartialEq)]
struct StateEffects {
    symbols:BTreeMap<(DeclarationId,Vec<SemanticId>),SemanticId>,
    selections:crate::scientific_selection::Selections,
}
impl FieldFragment {
    /// Conservative known owned extent, excluding separately counted package storage.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()+self.typed.retained_bytes()+crate::extent::expression(&self.inspection)+self.model.retained_bytes()
            +self.continuity_done.len()*128+self.bindings.capacity()*size_of::<SemanticId>()
            +self.states.values().map(|state|state.symbols.keys().map(|(_,coordinates)|coordinates.capacity()*size_of::<SemanticId>()+128).sum::<usize>()+crate::extent::selections(&state.selections)).sum::<usize>()+crate::extent::selections(&self.selections)+self.finite.retained_bytes()
            +self.original_guards.iter().map(|guard|guard.source.field.role.capacity()+guard.declarations.len()*128+256).sum::<usize>()
    }
}
/// Derive through the sole specialization interpreter with explicit immutable local premises.
/// # Errors
/// Original source, member lookup, physical, finite or resource refusals.
pub fn derive(input: &FieldFragmentInput, fragments: Option<&dyn prelude::Fragments>, cancel: &dyn Fn()->bool, discretizer: &dyn crate::continuous::Discretizer) -> Result<FieldFragment> {
    let recipe = &input.recipe;
    let context = input.package.context();
    let mut engine = Engine::new(&input.package,&context,recipe.root,recipe.limits,cancel,discretizer,fragments);
    engine.field_fragment_active = true;
    engine.reader = if recipe.fixture { crate::provenance::Reader::Fixture } else { crate::provenance::Reader::Production(recipe.root) };
    engine.model = recipe.context.clone();
    engine.model.symbols = recipe.foreign.symbols.clone();
    let actual = (0..engine.model.symbols.len()).map(SymbolIx::from).collect::<Vec<_>>();
    engine.construction.import_fragment(&recipe.foreign.fragment,&engine.model.symbols,&actual)?;
    engine.states = recipe.states.clone();
    engine.numerical_sources = recipe.numerical_sources.clone();
    engine.numerical_source_stack = recipe.numerical_source_stack.clone();
    engine.set_alias_sources = recipe.set_alias_sources.clone();
    engine.source_locals = recipe.source_locals.clone();
    engine.temporal_owners = recipe.temporal_owners.clone();
    engine.temporal_axis = recipe.temporal_axis;
    engine.stack = recipe.stack.clone();
    engine.preset_stack = recipe.preset_stack.clone();
    engine.facts = recipe.facts.clone();
    engine.continuity_done=recipe.continuity_done.clone();
    engine.function_permissions = Some(Arc::new(recipe.permissions.clone()));
    // Source-only names are stable across unrelated root construction order.
    let mut naming = FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1);
    naming.str("field-inspection").id(&recipe.at.as_id()).id(&recipe.instance.as_id())
        .str(&dsl::render_expr(&recipe.expression));
    for (name,value) in &recipe.env {naming.str(name);value.frame(&mut naming);}
    for symbol in recipe.foreign.symbols.values() {naming.id(&symbol.id);}
    recipe.sources.inspection_frame(&mut naming);
    engine.inspection_context = Some(naming.finish_hash());
    let scope = recipe.sources.install(&recipe.expression,recipe.limits.items)?;
    let rewritten = engine.in_original_scope(scope, |engine| engine.rewrite(recipe.instance,&recipe.expression,&recipe.env,&recipe.chain))?;
    let mut roots = vec![rewritten.root];
    roots.extend(engine.model.symbols.values().filter_map(|symbol|symbol.definition));
    for relation in engine.model.equation_roots.values() { roots.extend([relation.lhs,relation.rhs]); }
    let mut guards = Vec::new();
    for annotation in &engine.model.annotations { roots.extend(annotation.roots.values()); guards.extend(annotation.roots.predicate()); }
    let typed = engine.construction.into_fragment_delta(roots,guards,&engine.model.symbols,&recipe.foreign)?;
    let bindings=engine.model.symbols.slots().iter().map(|symbol|symbol.id).collect();
    let mut new_symbols=Symbols::default();
    for symbol in engine.model.symbols.slots() {if !recipe.foreign.symbols.contains_key(&symbol.id) {new_symbols.push(symbol.clone());}}
    engine.model.symbols=new_symbols;
    engine.model.integrated.clear();engine.model.meshes.clear();engine.model.closures.clear();
    engine.model.derivatives.retain(|id,_|!recipe.context.derivatives.contains_key(id));
    engine.model.integrals.retain(|id,_|!recipe.context.integrals.contains_key(id));
    let states=engine.states.into_iter().filter_map(|(id,state)|{
        let prior=&recipe.states[&id];
        let symbols=state.symbols.into_iter().filter(|(key,value)|prior.symbols.get(key)!=Some(value)).collect::<BTreeMap<_,_>>();
        if symbols.is_empty()&&state.selections.is_empty() {None} else {Some((id,StateEffects{symbols,selections:state.selections}))}
    }).collect();
    Ok(FieldFragment { typed, inspection: rewritten.inspection, model: engine.model, states,bindings,
        selections: engine.selection_collector.into_inner(), finite: engine.finite_collector.finish(),
        original_guards: engine.original_guards, construction_items: engine.count, continuity_done: engine.continuity_done })
}

impl Engine<'_, '_> {
    pub(super) fn rewrite_local_field(&mut self, instance: InstanceId, expression: &Expr, env: &Environment, chain: &[DeclarationId]) -> Result<crate::resolved::construction::Rewritten> {
        let at = chain.last().copied().unwrap_or(self.states[&instance].definition);
        let source = self.original_expression_source(expression,at)?;
        let mut additional = chain.to_vec();
        let mut owner=Some(instance);
        while let Some(id)=owner {additional.push(self.states[&id].definition);owner=self.states[&id].parent;}
        // Resolve only structural namespace routes. Do not evaluate indices, realize
        // members, simplify guards or issue source-selection receipts during capture.
        fn path_dependencies(states:&BTreeMap<InstanceId,State>, instance:InstanceId, segments:&[PathSegment], ancestor:bool, out:&mut Vec<DeclarationId>) {
            let Some((first,rest))=segments.split_first() else{return};
            let state=&states[&instance];
            if first.name=="parent" {if let Some(parent)=state.parent {path_dependencies(states,parent,rest,true,out);}return;}
            if let Some(member)=state.members.get(&first.name) {
                out.push(*member);
                if !rest.is_empty() {for ((name,_),child) in &state.children {if name==&first.name {path_dependencies(states,*child,rest,false,out);}}}
            } else if ancestor {if let Some(parent)=state.parent {path_dependencies(states,parent,segments,true,out);}}
        }
        fn expression_dependencies(states:&BTreeMap<InstanceId,State>,instance:InstanceId,expression:&Expr,out:&mut Vec<DeclarationId>) {
            expression.walk(|value| {
                let path=match &value.kind {
                    ExprKind::Path(path)|ExprKind::NamedCall{name:path,..}|ExprKind::Partial{function:path,..}|ExprKind::Derivative{wrt:path,..}=>Some(path),
                    ExprKind::Reduce{binder,..}|ExprKind::Fold{binder,..}=>Some(&binder.domain), _=>None,
                };
                if let Some(path)=path {path_dependencies(states,instance,&path.segments,true,out);}
            });
        }
        expression_dependencies(&self.states,instance,expression,&mut additional);
        fn dependencies(value: &Value, out: &mut Vec<DeclarationId>) {
            match value {
                Value::Function(id) => out.push(*id), Value::Definition {id,bindings} => {out.push(*id); for value in bindings.values(){dependencies(value,out);}},
                Value::Entity {id,kind} => {out.push(*id);out.push(*kind);}, Value::Enum {enumeration,..} =>out.push(*enumeration), Value::Identifier{scheme,..}=>out.push(*scheme),
                Value::Row{table,fields,..}=>{out.push(*table);for value in fields.iter(){dependencies(value,out);}},
                Value::Set(values)|Value::Tuple(values)=>{for value in values{dependencies(value,out);}}, _=>{}
            }
        }
        for value in env.values().chain(self.states[&instance].env.values()) { dependencies(value,&mut additional); }
        for source in self.numerical_sources.values().chain(self.set_alias_sources.values()) {
            if additional.contains(&self.states[&source.instance].definition) {
                additional.push(source.declaration);
                expression_dependencies(&self.states,source.instance,&source.expression,&mut additional);
            }
        }
        let mut dependencies=additional.into_iter().collect::<BTreeSet<_>>();
        let package=loop {
            self.checkpoint()?;
            let package=self.p.select_field(source.field.declaration,&dependencies.iter().copied().collect::<Vec<_>>())?;
            let mut reached=Vec::new();
            for (field,checked) in &package.expressions {
                for (instance,state) in &self.states {
                    if state.definition==field.declaration || state.members.values().any(|id|*id==field.declaration) {
                        for expression in crate::expression::inventory::field(&checked.syntax) {
                            expression_dependencies(&self.states,*instance,expression,&mut reached);
                        }
                    }
                }
            }
            let previous=dependencies.len();
            dependencies.extend(reached);
            if dependencies.len()==previous {break Arc::new(package);}
        };
        let mut states = self.states.iter().filter(|(id,state)| **id==instance || package.declarations.contains_key(&state.definition)
            || state.members.values().any(|id|package.declarations.contains_key(id))).map(|(id,state)|(*id,state.clone())).collect::<BTreeMap<_,_>>();
        // Preserve lexical ancestry and complete namespaces, including failed lookups.
        let mut owner = Some(instance);
        while let Some(id)=owner { let state=&self.states[&id]; states.entry(id).or_insert_with(||state.clone()); owner=state.parent; }
        let mut selected = self.model.symbols.slots().iter_enumerated().filter(|(_,symbol)|states.contains_key(&symbol.lineage.instance)
            && package.declarations.contains_key(&symbol.lineage.declaration)).map(|(slot,_)|slot).collect::<BTreeSet<_>>();
        // Derived-coordinate facts carry actual scalar edges too. Keep their
        // complete existing owner closure without realizing a new coordinate.
        loop {
            let before=selected.len();
            for derivative in self.model.derivatives.values() {
                let slots=[self.model.symbols.slot(derivative.state),self.model.symbols.slot(derivative.rate)];
                if slots.iter().flatten().any(|slot|selected.contains(slot)) {selected.extend(slots.into_iter().flatten());}
            }
            for integral in self.model.integrals.values() {
                let slots=[self.model.symbols.slot(integral.result),self.model.symbols.slot(integral.integrand)];
                if slots.iter().flatten().any(|slot|selected.contains(slot)) {selected.extend(slots.into_iter().flatten());}
            }
            if selected.len()==before {break;}
        }
        let captured = self.construction.capture_scalar_premises(&self.model.symbols,&selected.into_iter().collect::<Vec<_>>())?;
        debug_assert_eq!(captured.actual.len(),captured.premises.symbols.len());
        let foreign_bindings=captured.bindings;
        let foreign=captured.premises;
        for state in states.values_mut() { state.symbols.retain(|_,id|foreign.symbols.contains_key(id)); }
        for state in states.values_mut() { state.selections.clear(); }
        let numerical_sources = self.numerical_sources.iter().filter(|((instance,_),source)|states.contains_key(instance)&&package.declarations.contains_key(&source.declaration)).map(|(key,value)|(key.clone(),value.clone())).collect();
        let set_alias_sources = self.set_alias_sources.iter().filter(|(_,source)|states.contains_key(&source.instance)&&package.declarations.contains_key(&source.declaration)).map(|(key,value)|(key.clone(),value.clone())).collect();
        let mut context = SpecializedModel { construction_limit:self.limits.items, ..SpecializedModel::default() };
        context.integrated = self.model.integrated.clone();
        context.derivatives = self.model.derivatives.iter().filter(|(id,_)|foreign.symbols.contains_key(id)).map(|(id,value)|(*id,value.clone())).collect();
        context.integrals = self.model.integrals.iter().filter(|(_,value)|foreign.symbols.contains_key(&value.result)).map(|(id,value)|(*id,value.clone())).collect();
        context.meshes = self.model.meshes.clone();
        context.closures = self.model.closures.iter().filter(|(_,value)|states.contains_key(&value.lineage.instance)&&package.declarations.contains_key(&value.lineage.declaration)).map(|(id,value)|{let mut value=value.clone();value.terms.clear();(*id,value)}).collect();
        let recipe = Arc::new(FieldFragmentRecipe { instance,at,root:self.root,fixture:matches!(self.reader,crate::provenance::Reader::Fixture),
            expression:expression.clone(),sources:self.expression_sources(expression,at)?,env:env.clone(),chain:chain.to_vec(),limits:self.limits,
            states,foreign,context,numerical_sources,numerical_source_stack:self.numerical_source_stack.clone(),
            set_alias_sources,source_locals:self.source_locals.clone(),temporal_owners:self.temporal_owners.clone(),temporal_axis:self.temporal_axis,
            stack:self.stack.clone(),preset_stack:self.preset_stack.clone(),facts:self.facts.clone(),permissions:self.evidence_permissions(instance)?,continuity_done:self.continuity_done.clone() });
        let input = Arc::new(FieldFragmentInput {package,recipe});
        let fragment = if let Some(fragments)=self.fragments { fragments.field_fragment(input)? } else {Arc::new(derive(&input,None,self.cancel,self.discretizer)?)};
        self.reserve(fragment.construction_items)?;
        let mut new = BTreeSet::new();
        for symbol in fragment.model.symbols.slots() {
            if !self.model.symbols.contains_key(&symbol.id) {
                new.insert(symbol.id);
                let mut symbol=symbol.clone(); symbol.definition=None;
                self.model.symbols.push(symbol);
            }
        }
        let bindings=fragment.bindings.iter().map(|id|self.model.symbols.slot(*id).ok_or_else(||invalid(*id,"field fragment scalar binding missing"))).collect::<Result<Vec<_>>>()?;
        let imported=self.construction.import_fragment_with_foreign(&fragment.typed,&self.model.symbols,&bindings,Some(&foreign_bindings))?;
        for symbol in fragment.model.symbols.slots() { if new.contains(&symbol.id) { let mut symbol=symbol.clone();symbol.definition=symbol.definition.map(|root|imported.node(root)).transpose()?;self.model.symbols.insert(symbol.id,symbol); } }
        for (id,state) in &fragment.states { if let Some(target)=self.states.get_mut(id) {target.symbols.extend(state.symbols.clone());target.selections.extend(state.selections.clone());} }
        for row in &fragment.model.equations {
            if self.model.equation_roots.contains_key(&row.id) {
                return Err(invalid(row.id,"field fragment repeats an existing generated row owner"));
            }
            let relation=fragment.model.equation_roots.get(&row.id).ok_or_else(||invalid(row.id,"field fragment row lacks its actual typed operands"))?;
            self.model.equation_roots.insert(row.id,crate::resolved::Relation{lhs:imported.node(relation.lhs)?,sense:relation.sense,rhs:imported.node(relation.rhs)?});
            self.model.equations.push(row.clone());
        }
        for annotation in &fragment.model.annotations { let mut annotation=annotation.clone();relocate_annotation(&mut annotation.roots,&imported)?;self.model.annotations.push(annotation); }
        for (relationship,_) in fragment.model.derivations.iter(){self.model.derivations.publish(*relationship)?;}
        for (name,function) in &fragment.model.functions {if self.model.functions.get(name).is_some_and(|prior|prior!=function){return Err(invalid(at,format!("field fragment inspection conflicts with original meaning: {name}; prior={}; incoming={}",self.model.functions[name].id,function.id)));}self.model.functions.insert(name.clone(),function.clone());}
        self.continuity_done.extend(fragment.continuity_done.iter().copied());
        self.model.derivatives.extend(fragment.model.derivatives.clone());self.model.integrals.extend(fragment.model.integrals.clone());
        for (occurrence,closure) in &fragment.selections {self.selection_collector.record(occurrence.clone(),closure.clone());}
        self.finite_collector.merge(&fragment.finite)?;self.original_guards.extend(fragment.original_guards.clone());
        Ok(crate::resolved::construction::Rewritten {inspection:fragment.inspection.clone(),root:imported.outputs[0]})
    }
}
fn relocate_annotation(roots:&mut crate::annotation::AnnotationRoots, imported:&ImportedFragment)->Result<()> {
    use crate::annotation::AnnotationRoots as A;
    fn node(value:&mut NodeIx, imported:&ImportedFragment)->Result<()>{*value=imported.node(*value)?;Ok(())}
    match roots {
        A::None=>{}, A::Scalar{value}|A::Engineering{value}=>node(value,imported)?,
        A::Bounds{lower,upper}|A::Valid{lower,upper}=>{node(lower,imported)?;node(upper,imported)?;},
        A::Check{predicate}=>{*predicate=imported.predicate(*predicate)?;},
        A::Accuracy{time,resolution,criterion_lower,criterion_upper}=>{for value in [time,resolution,criterion_lower,criterion_upper].into_iter().flatten(){node(value,imported)?;}}
    }
    Ok(())
}
