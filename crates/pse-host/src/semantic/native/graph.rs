// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Demand-selected graph extraction inside one immutable native worker snapshot.
use super::*;
use crate::{
    investigation::{Direction, Relationship, graph},
    semantic::{GraphAnswer, GraphNode, GraphQuery, GraphRole, GraphWitness},
};
use ra_ap_hir::{HasSource, ModuleDef};
use std::collections::{HashSet, VecDeque};

struct Catalog<'a> {
    snapshot: QuerySnapshot<'a>,
    nodes: Vec<GraphNode>,
    keys: BTreeMap<(PathBuf, usize, usize), usize>,
    witnesses: Vec<GraphWitness>,
    edges: Vec<graph::Edge>,
    complete: bool,
    limitations: BTreeSet<String>,
}
impl<'a> Catalog<'a> {
    fn node(
        &mut self,
        location: NativeLocation,
        name: Option<String>,
        declaration: bool,
    ) -> Option<usize> {
        let digest = self.snapshot.digests.get(&location.path)?.clone();
        let key = (location.path.clone(), location.start, location.end);
        if let Some(&id) = self.keys.get(&key) {
            if self.nodes[id].name.is_none() {
                self.nodes[id].name = name;
            }
            return Some(id);
        }
        let id = self.nodes.len();
        self.nodes.push(GraphNode {
            location,
            name,
            source_sha256: digest,
            declaration,
        });
        self.keys.insert(key, id);
        Some(id)
    }
    fn unknown(&mut self, message: &str) {
        self.complete = false;
        self.limitations.insert(message.to_owned());
    }
    fn witness(
        &mut self,
        location: Option<NativeLocation>,
        role: GraphRole,
        declared_type: Option<String>,
    ) -> usize {
        let source_sha256 = location
            .as_ref()
            .and_then(|loc| self.snapshot.digests.get(&loc.path))
            .cloned();
        let mapped = location.is_some() && source_sha256.is_some();
        let limitations = if mapped {
            Vec::new()
        } else {
            vec!["native occurrence has no precise original snapshot mapping".into()]
        };
        if !mapped {
            self.unknown("one or more relationship occurrences have no precise original mapping");
        }
        let id = self.witnesses.len();
        self.witnesses.push(GraphWitness {
            location: mapped.then_some(location).flatten(),
            source_sha256,
            role,
            declared_type,
            instantiated_type: None,
            limitations,
        });
        id
    }
    fn edge(
        &mut self,
        source: usize,
        target: usize,
        relation: Relationship,
        location: Option<NativeLocation>,
        role: GraphRole,
        declared_type: Option<String>,
    ) {
        let witness = self.witness(location, role, declared_type);
        self.edges.push(graph::Edge {
            source,
            target,
            relation,
            witness,
        });
    }
    fn source<D: HasSource>(
        &self,
        sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
        def: D,
    ) -> Option<NativeLocation> {
        let source = sema.source(def)?;
        let original = sema.original_range_opt(source.value.syntax())?;
        location(
            self.snapshot.paths,
            original.file_id.file_id(self.snapshot.db),
            original.range,
        )
    }
    fn definition_location(
        &self,
        sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
        definition: ModuleDef,
    ) -> Option<NativeLocation> {
        match definition {
            ModuleDef::Module(module) => {
                if let Some(source) = module.declaration_source(self.snapshot.db) {
                    let original = sema.original_range_opt(source.value.syntax())?;
                    location(
                        self.snapshot.paths,
                        original.file_id.file_id(self.snapshot.db),
                        original.range,
                    )
                } else {
                    let file = module.as_source_file_id(self.snapshot.db)?;
                    let parsed = sema.parse(file);
                    let original = sema.original_range_opt(parsed.syntax())?;
                    location(
                        self.snapshot.paths,
                        original.file_id.file_id(self.snapshot.db),
                        original.range,
                    )
                }
            }
            ModuleDef::Function(value) => self.source(sema, value),
            ModuleDef::Adt(value) => self.source(sema, value),
            ModuleDef::EnumVariant(value) => self.source(sema, value),
            ModuleDef::Const(value) => self.source(sema, value),
            ModuleDef::Static(value) => self.source(sema, value),
            ModuleDef::Trait(value) => self.source(sema, value),
            ModuleDef::TypeAlias(value) => self.source(sema, value),
            ModuleDef::Macro(value) => self.source(sema, value),
            ModuleDef::BuiltinType(_) => None,
        }
    }
    fn definition(
        &mut self,
        sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
        definition: ModuleDef,
    ) -> Option<usize> {
        let Some(location) = self.definition_location(sema, definition) else {
            if !matches!(definition, ModuleDef::BuiltinType(_)) {
                self.unknown("native declaration has no precise original source mapping");
            }
            return None;
        };
        let name = definition.name(self.snapshot.db).map(|name| {
            name.display(self.snapshot.db, native_ra_syntax::Edition::CURRENT)
                .to_string()
        });
        self.node(location, name, true)
    }
    fn declaration_at(
        &self,
        sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
        location: &NativeLocation,
    ) -> Option<(NativeLocation, Option<String>)> {
        let file = *self.snapshot.files.get(&location.path)?;
        let parsed = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
            self.snapshot.db,
            file,
        ));
        let offset = native_ra_syntax::TextSize::from(u32::try_from(location.start).ok()?);
        let node = parsed
            .syntax()
            .token_at_offset(offset)
            .right_biased()?
            .parent()?;
        for node in node.ancestors() {
            let at_start = node.text_range().start() == offset;
            let at_name = node
                .descendants()
                .find_map(ast::Name::cast)
                .is_some_and(|name| name.syntax().text_range().contains(offset));
            if !at_start && !at_name {
                continue;
            }
            if let Some(value) = ast::RecordField::cast(node.clone()) {
                let field = sema.to_def(&value)?;
                return Some((
                    self.source(sema, field)?,
                    Some(
                        field
                            .name(self.snapshot.db)
                            .display(self.snapshot.db, native_ra_syntax::Edition::CURRENT)
                            .to_string(),
                    ),
                ));
            }
            if let Some(value) = ast::TupleField::cast(node.clone()) {
                return Some((self.source(sema, sema.to_def(&value)?)?, None));
            }
            if let Some(value) = ast::Impl::cast(node.clone()) {
                return Some((self.source(sema, sema.to_def(&value)?)?, None));
            }
            let definition = if let Some(value) = ast::Fn::cast(node.clone()) {
                sema.to_def(&value).map(ModuleDef::Function)
            } else if let Some(value) = ast::Struct::cast(node.clone()) {
                sema.to_def(&value)
                    .map(|value| ModuleDef::Adt(value.into()))
            } else if let Some(value) = ast::Enum::cast(node.clone()) {
                sema.to_def(&value)
                    .map(|value| ModuleDef::Adt(value.into()))
            } else if let Some(value) = ast::Union::cast(node.clone()) {
                sema.to_def(&value)
                    .map(|value| ModuleDef::Adt(value.into()))
            } else if let Some(value) = ast::Variant::cast(node.clone()) {
                sema.to_def(&value).map(ModuleDef::EnumVariant)
            } else if let Some(value) = ast::Trait::cast(node.clone()) {
                sema.to_def(&value).map(ModuleDef::Trait)
            } else if let Some(value) = ast::TypeAlias::cast(node.clone()) {
                sema.to_def(&value).map(ModuleDef::TypeAlias)
            } else if let Some(value) = ast::Const::cast(node.clone()) {
                sema.to_def(&value).map(ModuleDef::Const)
            } else if let Some(value) = ast::Static::cast(node.clone()) {
                sema.to_def(&value).map(ModuleDef::Static)
            } else if let Some(value) = ast::Module::cast(node.clone()) {
                sema.to_def(&value).map(ModuleDef::Module)
            } else {
                None
            };
            if let Some(definition) = definition {
                let location = self.definition_location(sema, definition)?;
                let name = definition.name(self.snapshot.db).map(|name| {
                    name.display(self.snapshot.db, native_ra_syntax::Edition::CURRENT)
                        .to_string()
                });
                return Some((location, name));
            }
        }
        None
    }
    fn anchor(&self, location: &NativeLocation) -> Result<(usize, usize)> {
        let file = *self
            .snapshot
            .files
            .get(&location.path)
            .ok_or(Error::Stale)?;
        let sema = Semantics::new(self.snapshot.db);
        let parsed = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
            self.snapshot.db,
            file,
        ));
        let range = native_ra_syntax::TextRange::new(
            native_ra_syntax::TextSize::from(
                u32::try_from(location.start).map_err(|_| Error::Stale)?,
            ),
            native_ra_syntax::TextSize::from(
                u32::try_from(location.end).map_err(|_| Error::Stale)?,
            ),
        );
        let offset = parsed
            .syntax()
            .descendants()
            .find(|node| node.text_range() == range)
            .and_then(|node| node.descendants().find_map(ast::Name::cast))
            .map_or(location.start, |name| {
                usize::from(name.syntax().text_range().start())
            });
        let text = self.snapshot.db.file_text(file);
        let prefix = text
            .text(self.snapshot.db)
            .get(..offset)
            .ok_or(Error::Stale)?;
        Ok((
            prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
            prefix
                .rsplit('\n')
                .next()
                .map_or(1, |line| line.chars().count() + 1),
        ))
    }
    fn dependencies(
        &mut self,
        sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
        source: usize,
        ty: &ra_ap_hir::Type<'a>,
        relation: Relationship,
        witness: Option<NativeLocation>,
        role: GraphRole,
        display: ra_ap_hir::DisplayTarget,
    ) {
        let mut nominal = Vec::new();
        let mut seen = HashSet::new();
        ty.walk(self.snapshot.db, |ty| {
            if let Some(adt) = ty.as_adt() {
                if seen.insert(adt) {
                    nominal.push(adt);
                }
            }
        });
        let declared = ty.display(self.snapshot.db, display).to_string();
        for adt in nominal {
            if let Some(target) = self.definition(sema, ModuleDef::Adt(adt)) {
                self.edge(
                    source,
                    target,
                    relation,
                    witness.clone(),
                    role.clone(),
                    Some(declared.clone()),
                );
            }
        }
        if ty.contains_unknown() {
            self.unknown("one or more declaration types contain unresolved native components");
        }
    }
    fn fields(
        &mut self,
        sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
        source: usize,
        container: Option<usize>,
        fields: Vec<ra_ap_hir::Field>,
        selected: &[Relationship],
        display: ra_ap_hir::DisplayTarget,
    ) {
        for (index, field) in fields.into_iter().enumerate() {
            let witness = self.source(sema, field);
            if selected.contains(&Relationship::Contains) {
                if let Some(field_location) = witness.clone() {
                    let name = field
                        .name(self.snapshot.db)
                        .display(self.snapshot.db, display.edition)
                        .to_string();
                    if let (Some(container), Some(target)) = (
                        container,
                        self.node(field_location.clone(), Some(name), true),
                    ) {
                        self.edge(
                            container,
                            target,
                            Relationship::Contains,
                            Some(field_location),
                            GraphRole::Containment,
                            None,
                        );
                    }
                }
            }
            if selected.contains(&Relationship::FieldType) {
                let name = Some(
                    field
                        .name(self.snapshot.db)
                        .display(self.snapshot.db, display.edition)
                        .to_string(),
                );
                self.dependencies(
                    sema,
                    source,
                    &field.ty(self.snapshot.db),
                    Relationship::FieldType,
                    witness,
                    GraphRole::Field { index, name },
                    display,
                );
            }
        }
    }
    fn structural(
        &mut self,
        domain: &BTreeSet<PathBuf>,
        selected: &[Relationship],
        max_edges: usize,
        token: &CancellationToken,
    ) -> Result<()> {
        let sema = Semantics::new(self.snapshot.db);
        let mut modules = VecDeque::new();
        let mut seen = HashSet::new();
        for path in domain {
            let file = self.snapshot.files.get(path).ok_or_else(|| {
                Error::Unavailable(
                    "traversal source is outside the prepared native snapshot".into(),
                )
            })?;
            let parsed = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
                self.snapshot.db,
                *file,
            ));
            if parsed
                .syntax()
                .descendants()
                .filter_map(ast::Item::cast)
                .any(|item| {
                    item.syntax()
                        .ancestors()
                        .skip(1)
                        .any(|node| ast::BlockExpr::can_cast(node.kind()))
                })
            {
                self.unknown("block-local item declarations are not exhaustively enumerated by the selected module/impl declaration population");
            }
            let mut interpretations = sema.file_to_module_defs(*file);
            let Some(module) = interpretations.next() else {
                self.unknown("traversal source is inactive in the selected native snapshot");
                continue;
            };
            if interpretations.next().is_some() {
                return Err(Error::Unavailable(
                    "relationship traversal source belongs to plural native module contexts".into(),
                ));
            }
            modules.push_back(module);
        }
        while let Some(module) = modules.pop_front() {
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            if !seen.insert(module) {
                continue;
            }
            let parent = self.definition(&sema, ModuleDef::Module(module));
            let mut declarations = module
                .declarations(self.snapshot.db)
                .into_iter()
                .map(|declaration| (declaration, parent))
                .collect::<VecDeque<_>>();
            for impl_ in module.impl_defs(self.snapshot.db) {
                let impl_source = self.source(&sema, impl_);
                let impl_node = impl_source.and_then(|location| self.node(location, None, true));
                if selected.contains(&Relationship::Contains) {
                    if let (Some(parent), Some(target)) = (parent, impl_node) {
                        self.edge(
                            parent,
                            target,
                            Relationship::Contains,
                            Some(self.nodes[target].location.clone()),
                            GraphRole::Containment,
                            None,
                        );
                    }
                }
                declarations.extend(impl_.items(self.snapshot.db).into_iter().map(|item| {
                    (
                        match item {
                            ra_ap_hir::AssocItem::Function(value) => ModuleDef::Function(value),
                            ra_ap_hir::AssocItem::Const(value) => ModuleDef::Const(value),
                            ra_ap_hir::AssocItem::TypeAlias(value) => ModuleDef::TypeAlias(value),
                        },
                        impl_node,
                    )
                }));
            }
            while let Some((declaration, container)) = declarations.pop_front() {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                let Some(source) = self.definition(&sema, declaration) else {
                    continue;
                };
                if !domain.contains(&self.nodes[source].location.path) {
                    continue;
                }
                if selected.contains(&Relationship::Contains) {
                    if let Some(parent) = container {
                        self.edge(
                            parent,
                            source,
                            Relationship::Contains,
                            Some(self.nodes[source].location.clone()),
                            GraphRole::Containment,
                            None,
                        );
                    }
                }
                let display = module
                    .krate(self.snapshot.db)
                    .to_display_target(self.snapshot.db);
                match declaration {
                    ModuleDef::Module(child) => modules.push_back(child),
                    ModuleDef::Function(function) => {
                        if selected.contains(&Relationship::ParameterType) {
                            for parameter in function.params_without_self(self.snapshot.db) {
                                self.dependencies(
                                    &sema,
                                    source,
                                    parameter.ty(),
                                    Relationship::ParameterType,
                                    self.source(&sema, parameter.clone()),
                                    GraphRole::Parameter {
                                        index: parameter.index(),
                                    },
                                    display,
                                );
                            }
                        }
                        if selected.contains(&Relationship::ReturnType) {
                            let witness = sema
                                .source(function)
                                .and_then(|source| source.value.ret_type())
                                .and_then(|ret| sema.original_range_opt(ret.syntax()))
                                .and_then(|range| {
                                    location(
                                        self.snapshot.paths,
                                        range.file_id.file_id(self.snapshot.db),
                                        range.range,
                                    )
                                });
                            self.dependencies(
                                &sema,
                                source,
                                &function.ret_type(self.snapshot.db),
                                Relationship::ReturnType,
                                witness.clone(),
                                GraphRole::Return {
                                    async_output: false,
                                },
                                display,
                            );
                            if let Some(output) = function.async_ret_type(self.snapshot.db) {
                                self.dependencies(
                                    &sema,
                                    source,
                                    &output,
                                    Relationship::ReturnType,
                                    witness,
                                    GraphRole::Return { async_output: true },
                                    display,
                                );
                            }
                        }
                    }
                    ModuleDef::Adt(ra_ap_hir::Adt::Struct(value)) => self.fields(
                        &sema,
                        source,
                        Some(source),
                        value.fields(self.snapshot.db),
                        selected,
                        display,
                    ),
                    ModuleDef::Adt(ra_ap_hir::Adt::Union(value)) => self.fields(
                        &sema,
                        source,
                        Some(source),
                        value.fields(self.snapshot.db),
                        selected,
                        display,
                    ),
                    ModuleDef::Adt(ra_ap_hir::Adt::Enum(value)) => {
                        for variant in value.variants(self.snapshot.db) {
                            let container = self.definition(&sema, ModuleDef::EnumVariant(variant));
                            if let Some(target) = container {
                                if selected.contains(&Relationship::Contains) {
                                    self.edge(
                                        source,
                                        target,
                                        Relationship::Contains,
                                        self.source(&sema, variant),
                                        GraphRole::Containment,
                                        None,
                                    );
                                }
                            }
                            self.fields(
                                &sema,
                                source,
                                container,
                                variant.fields(self.snapshot.db),
                                selected,
                                display,
                            );
                        }
                    }
                    ModuleDef::Trait(value) => {
                        for item in value.items(self.snapshot.db) {
                            let declaration = match item {
                                ra_ap_hir::AssocItem::Function(value) => ModuleDef::Function(value),
                                ra_ap_hir::AssocItem::Const(value) => ModuleDef::Const(value),
                                ra_ap_hir::AssocItem::TypeAlias(value) => {
                                    ModuleDef::TypeAlias(value)
                                }
                            };
                            declarations.push_back((declaration, Some(source)));
                        }
                    }
                    _ => {}
                }
                if self.edges.len() > max_edges {
                    self.unknown("relationship extraction reached the edge examination allowance before the inverse population was complete");
                    return Ok(());
                }
            }
        }
        if selected.contains(&Relationship::Uses) {
            self.limitations.insert("uses reports explicit resolved paths and receiver-method syntax; implicit language operations are not enumerated".into());
            for path in domain {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                let file = self.snapshot.files[path];
                let parsed = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
                    self.snapshot.db,
                    file,
                ));
                if parsed
                    .syntax()
                    .descendants()
                    .any(|node| ast::FieldExpr::can_cast(node.kind()))
                {
                    self.unknown("explicit field-access uses are not yet exhaustively projected; field read/write classification belongs to the remaining native relation scope");
                }
                for path in parsed.syntax().descendants().filter_map(ast::Path::cast) {
                    if token.is_cancelled() {
                        return Err(Error::Cancelled);
                    }
                    let target = match sema.resolve_path(&path) {
                        Some(ra_ap_hir::PathResolution::Def(target)) => target,
                        Some(_) => continue,
                        None => {
                            self.unknown("one or more written paths do not resolve in the selected native context");
                            continue;
                        }
                    };
                    let Some(destination) = self.definition(&sema, target) else {
                        continue;
                    };
                    let function = path
                        .syntax()
                        .ancestors()
                        .find_map(ast::Fn::cast)
                        .and_then(|function| sema.to_def(&function));
                    let owner = function.map(ModuleDef::Function).or_else(|| {
                        sema.scope(path.syntax())
                            .map(|scope| ModuleDef::Module(scope.module()))
                    });
                    let Some(source) = owner.and_then(|owner| self.definition(&sema, owner)) else {
                        self.unknown("resolved use has no mapped owning declaration");
                        continue;
                    };
                    let witness = sema.original_range_opt(path.syntax()).and_then(|range| {
                        location(
                            self.snapshot.paths,
                            range.file_id.file_id(self.snapshot.db),
                            range.range,
                        )
                    });
                    self.edge(
                        source,
                        destination,
                        Relationship::Uses,
                        witness,
                        GraphRole::Reference {
                            read: None,
                            write: None,
                            import: Some(
                                path.syntax()
                                    .ancestors()
                                    .any(|node| ast::Use::can_cast(node.kind())),
                            ),
                            test: None,
                        },
                        None,
                    );
                    if self.edges.len() > max_edges {
                        self.unknown(
                            "resolved-use extraction reached the edge examination allowance",
                        );
                        return Ok(());
                    }
                }
                for method in parsed
                    .syntax()
                    .descendants()
                    .filter_map(ast::MethodCallExpr::cast)
                {
                    if token.is_cancelled() {
                        return Err(Error::Cancelled);
                    }
                    let Some(function) = sema.resolve_method_call(&method) else {
                        self.unknown("one or more written method calls do not resolve in the selected native context");
                        continue;
                    };
                    let Some(destination) = self.definition(&sema, ModuleDef::Function(function))
                    else {
                        continue;
                    };
                    let owner = method
                        .syntax()
                        .ancestors()
                        .find_map(ast::Fn::cast)
                        .and_then(|function| sema.to_def(&function))
                        .map(ModuleDef::Function)
                        .or_else(|| {
                            sema.scope(method.syntax())
                                .map(|scope| ModuleDef::Module(scope.module()))
                        });
                    let Some(source) = owner.and_then(|owner| self.definition(&sema, owner)) else {
                        self.unknown("resolved method use has no mapped owning declaration");
                        continue;
                    };
                    let witness = method
                        .name_ref()
                        .and_then(|name| sema.original_range_opt(name.syntax()))
                        .and_then(|range| {
                            location(
                                self.snapshot.paths,
                                range.file_id.file_id(self.snapshot.db),
                                range.range,
                            )
                        });
                    self.edge(
                        source,
                        destination,
                        Relationship::Uses,
                        witness,
                        GraphRole::Reference {
                            read: None,
                            write: None,
                            import: Some(false),
                            test: None,
                        },
                        None,
                    );
                    if self.edges.len() > max_edges {
                        self.unknown(
                            "resolved-use extraction reached the edge examination allowance",
                        );
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
}

pub(super) fn query(
    snapshot: QuerySnapshot<'_>,
    session: AnalysisContext,
    query: GraphQuery,
    token: &CancellationToken,
) -> Result<GraphAnswer> {
    if query.seeds.is_empty()
        || query.relations.is_empty()
        || query.depth == 0
        || query.max_nodes == 0
        || query.max_edges == 0
    {
        return Err(Error::InvalidRequest("bounded relationships require nonempty seeds/relations and positive depth/node/edge bounds".into()));
    }
    if query.relations.iter().any(|relation| {
        matches!(
            (relation, query.direction),
            (Relationship::Definition, Direction::Incoming)
                | (Relationship::References, Direction::Outgoing)
                | (Relationship::Implementations, Direction::Incoming)
        )
    }) {
        return Err(Error::unsupported(
            "selected navigation relation has no supported canonical traversal in this direction; resolved uses and structural relations support explicit incoming and outgoing traversal",
        ));
    }
    let domain = query
        .traversal_paths
        .iter()
        .map(|path| path.canonicalize().map_err(Error::from))
        .collect::<Result<BTreeSet<_>>>()?
        .into_iter()
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "rs")
                || snapshot.files.contains_key(path)
        })
        .collect::<BTreeSet<_>>();
    let output = query
        .output_paths
        .iter()
        .map(|path| path.canonicalize().map_err(Error::from))
        .collect::<Result<BTreeSet<_>>>()?;
    let mut catalog = Catalog {
        snapshot,
        nodes: Vec::new(),
        keys: BTreeMap::new(),
        witnesses: Vec::new(),
        edges: Vec::new(),
        complete: true,
        limitations: BTreeSet::new(),
    };
    let mut seeds = Vec::new();
    for seed in &query.seeds {
        if !domain.contains(&seed.file.canonicalize()?) {
            return Err(Error::InvalidRequest(
                "native relationship seed is outside the traversal domain".into(),
            ));
        }
        let answer = query_world(
            QuerySnapshot {
                db: catalog.snapshot.db,
                files: catalog.snapshot.files,
                paths: catalog.snapshot.paths,
                digests: catalog.snapshot.digests,
                packages: catalog.snapshot.packages,
                runnable_targets: catalog.snapshot.runnable_targets,
            },
            session.clone(),
            seed.clone(),
            token,
        )?;
        let targets = answer
            .observed
            .relationships
            .iter()
            .filter(|fact| fact.relation == super::super::FactRelation::Definition)
            .filter_map(|fact| match &fact.location {
                super::super::FactLocation::Bytes { path, start, end } => Some(NativeLocation {
                    path: path.clone(),
                    start: *start,
                    end: *end,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        if targets.len() > 1 {
            return Err(Error::Unavailable(
                "relationship seed resolves to plural native declarations".into(),
            ));
        }
        let sema = Semantics::new(catalog.snapshot.db);
        if query.relations.contains(&Relationship::Definition) {
            let path = seed.file.canonicalize()?;
            let file = catalog.snapshot.files[&path];
            let text = catalog.snapshot.db.file_text(file);
            let offset = crate::source_read::byte_offset(
                text.text(catalog.snapshot.db),
                seed.line,
                seed.column,
            )?;
            let point = NativeLocation {
                path: path.clone(),
                start: offset,
                end: offset,
            };
            let node = if let Some((declaration, name)) = catalog.declaration_at(&sema, &point) {
                catalog.node(declaration, name, true)
            } else {
                let parsed = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
                    catalog.snapshot.db,
                    file,
                ));
                let token = parsed
                    .syntax()
                    .token_at_offset(native_ra_syntax::TextSize::from(
                        u32::try_from(offset).map_err(|_| Error::Stale)?,
                    ))
                    .right_biased()
                    .ok_or_else(|| {
                        Error::Unavailable("relationship seed has no original source token".into())
                    })?;
                let range = token.text_range();
                catalog.node(
                    NativeLocation {
                        path,
                        start: usize::from(range.start()),
                        end: usize::from(range.end()),
                    },
                    None,
                    false,
                )
            }
            .ok_or_else(|| {
                Error::Unavailable("relationship seed is outside original snapshot custody".into())
            })?;
            if !domain.contains(&catalog.nodes[node].location.path) {
                return Err(Error::InvalidRequest(
                    "relationship seed is outside the traversal domain".into(),
                ));
            }
            seeds.push(node);
            continue;
        }
        let original = if let Some(target) = targets.first() {
            target.clone()
        } else {
            let file = catalog.snapshot.files[&seed.file.canonicalize()?];
            let text = catalog.snapshot.db.file_text(file);
            let offset = crate::source_read::byte_offset(
                text.text(catalog.snapshot.db),
                seed.line,
                seed.column,
            )?;
            NativeLocation {
                path: seed.file.canonicalize()?,
                start: offset,
                end: offset,
            }
        };
        let (declaration, name) = catalog.declaration_at(&sema, &original).ok_or_else(|| {
            Error::Unavailable("relationship seed has no unique mapped native declaration".into())
        })?;
        let node = catalog.node(declaration, name, true).ok_or_else(|| {
            Error::Unavailable(
                "relationship seed declaration is outside original snapshot custody".into(),
            )
        })?;
        if !domain.contains(&catalog.nodes[node].location.path) {
            return Err(Error::InvalidRequest(
                "resolved native relationship seed is outside the traversal domain".into(),
            ));
        }
        seeds.push(node);
    }
    // A broad working selector can include loose files or Cargo's scratch Rust files.
    // The question is over this prepared active world, not every filesystem Rust file.
    // Seed admission above still uses query_world's inactive/plural-context refusal.
    let sema = Semantics::new(catalog.snapshot.db);
    let mut active_domain = BTreeSet::new();
    for path in domain {
        let file = catalog.snapshot.files.get(&path).ok_or_else(|| {
            Error::Unavailable(format!(
                "traversal source is outside the prepared native snapshot: {path:?}"
            ))
        })?;
        let mut interpretations = sema.file_to_module_defs(*file);
        if interpretations.next().is_none() {
            catalog.limitations.insert(format!("selected source has no active module in this prepared world and is outside the native traversal domain: {path:?}"));
            continue;
        }
        if interpretations.next().is_some() {
            return Err(Error::Unavailable(format!(
                "relationship traversal source belongs to plural native module contexts: {path:?}"
            )));
        }
        active_domain.insert(path);
    }
    let domain = active_domain;
    let structural = query.relations.iter().any(|relation| {
        matches!(
            relation,
            Relationship::Contains
                | Relationship::ParameterType
                | Relationship::ReturnType
                | Relationship::FieldType
                | Relationship::Uses
        )
    });
    if structural {
        catalog.structural(&domain, &query.relations, query.max_edges, token)?;
    }
    let intent = graph::Traversal {
        seeds,
        relations: query.relations.clone(),
        direction: query.direction,
        depth: query.depth,
        max_nodes: query.max_nodes,
        max_edges: query.max_edges,
        seed_match: query.seed_match,
        exists: query.exists,
    };
    // Selection is stable in this snapshot; future dynamically discovered navigation nodes
    // are selected using their original path by the precomputed side table below.
    let selected = std::cell::RefCell::new(BTreeSet::new());
    for (id, node) in catalog.nodes.iter().enumerate() {
        if output.contains(&node.location.path) {
            selected.borrow_mut().insert(id);
        }
    }
    let mut traversal = graph::walk(
        &intent,
        |node| {
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let mut edges = catalog
                .edges
                .iter()
                .filter(|edge| {
                    domain.contains(&catalog.nodes[edge.source].location.path)
                        && domain.contains(&catalog.nodes[edge.target].location.path)
                        && match query.direction {
                            Direction::Outgoing => edge.source == node,
                            Direction::Incoming => edge.target == node,
                        }
                })
                .cloned()
                .collect::<Vec<_>>();
            for relation in &query.relations {
                // A reference's canonical destination is a declaration. Asking for
                // incoming references at an already emitted use site cannot rebind
                // that occurrence to its declaration and invent use-to-use edges.
                if *relation == Relationship::References && !catalog.nodes[node].declaration {
                    continue;
                }
                let kind = match (relation, query.direction) {
                    (Relationship::Definition, Direction::Outgoing) => Some(Kind::Explain),
                    (Relationship::Definition | Relationship::References, Direction::Incoming)
                    | (Relationship::References, Direction::Outgoing) => Some(Kind::Refs),
                    (Relationship::Calls, _) => Some(Kind::Calls),
                    (Relationship::Implementations, Direction::Outgoing) => {
                        Some(Kind::Implementations)
                    }
                    (Relationship::Implementations, Direction::Incoming) => {
                        catalog.unknown("inverse native implementation navigation is unsupported");
                        None
                    }
                    _ => None,
                };
                let Some(kind) = kind else {
                    continue;
                };
                let native = &catalog.nodes[node];
                let (line, column) = catalog.anchor(&native.location)?;
                let answer = query_world(
                    QuerySnapshot {
                        db: catalog.snapshot.db,
                        files: catalog.snapshot.files,
                        paths: catalog.snapshot.paths,
                        digests: catalog.snapshot.digests,
                        packages: catalog.snapshot.packages,
                        runnable_targets: catalog.snapshot.runnable_targets,
                    },
                    session.clone(),
                    Query {
                        kind,
                        file: native.location.path.clone(),
                        line,
                        column,
                        if_source: Some(native.source_sha256.clone()),
                        if_generation: Some(1),
                        if_candidate: None,
                        limit: query.max_edges.saturating_add(1),
                        end_line: None,
                    },
                    token,
                )?;
                if answer
                    .omitted_relationships
                    .values()
                    .any(|omitted| *omitted > 0)
                {
                    catalog.unknown("native relationship navigation was truncated");
                }
                if *relation == Relationship::Definition
                    && !answer
                        .observed
                        .relationships
                        .iter()
                        .any(|fact| fact.relation == super::super::FactRelation::Definition)
                {
                    catalog.unknown("native definition navigation did not establish a resolved declaration target");
                }
                for fact in answer.observed.relationships {
                    let expected = match relation {
                        Relationship::Definition => super::super::FactRelation::Definition,
                        Relationship::References => super::super::FactRelation::References,
                        Relationship::Calls => super::super::FactRelation::Calls,
                        Relationship::Implementations => {
                            super::super::FactRelation::Implementations
                        }
                        _ => continue,
                    };
                    if fact.relation != expected {
                        continue;
                    }
                    if *relation == Relationship::Calls
                        && !matches!(
                            (fact.direction, query.direction),
                            (super::super::FactDirection::Incoming, Direction::Incoming)
                                | (super::super::FactDirection::Outgoing, Direction::Outgoing)
                        )
                    {
                        continue;
                    }
                    let super::super::FactLocation::Bytes { path, start, end } = fact.location
                    else {
                        catalog
                            .unknown("native relationship has no byte-qualified original location");
                        continue;
                    };
                    if !domain.contains(&path) {
                        continue;
                    }
                    let location = NativeLocation { path, start, end };
                    let other = if *relation == Relationship::Definition
                        || *relation == Relationship::Implementations
                    {
                        let sema = Semantics::new(catalog.snapshot.db);
                        match catalog.declaration_at(&sema, &location) {
                            Some((declaration, name)) => {
                                catalog.node(declaration, name.or(fact.name), true)
                            }
                            None => {
                                catalog.unknown("native navigation target has no uniquely mapped declaration identity");
                                None
                            }
                        }
                    } else {
                        catalog.node(location.clone(), fact.name, false)
                    };
                    let Some(other) = other else {
                        catalog.unknown(
                            "native relationship target is outside snapshot source custody",
                        );
                        continue;
                    };
                    if output.contains(&catalog.nodes[other].location.path) {
                        selected.borrow_mut().insert(other);
                    }
                    let incoming = matches!(query.direction, Direction::Incoming)
                        || *relation == Relationship::References;
                    let (source, target) = if incoming {
                        (other, node)
                    } else {
                        (node, other)
                    };
                    let witness = if *relation == Relationship::Definition {
                        catalog.witness(
                            Some(catalog.nodes[node].location.clone()),
                            GraphRole::Navigation,
                            None,
                        )
                    } else {
                        catalog.witness(
                            Some(location),
                            GraphRole::Reference {
                                read: fact.read,
                                write: fact.write,
                                import: fact.import,
                                test: fact.test,
                            },
                            None,
                        )
                    };
                    edges.push(graph::Edge {
                        source,
                        target,
                        relation: *relation,
                        witness,
                    });
                }
            }
            Ok(graph::Adjacency {
                edges,
                complete: catalog.complete,
            })
        },
        |node| selected.borrow().contains(&node),
        || token.is_cancelled(),
    )?;
    // Extraction may inspect a larger inverse population. Export only admitted nodes,
    // witnessed edge endpoints and frontier; request-local indexes are remapped once.
    let keep = traversal
        .reached
        .iter()
        .map(|r| r.node)
        .chain(traversal.frontier.iter().copied())
        .chain(traversal.edges.iter().flat_map(|e| [e.source, e.target]))
        .collect::<BTreeSet<_>>();
    let remap = keep
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect::<BTreeMap<_, _>>();
    let nodes = catalog
        .nodes
        .into_iter()
        .enumerate()
        .filter_map(|(id, node)| keep.contains(&id).then_some(node))
        .collect();
    let witness_keep = traversal
        .edges
        .iter()
        .map(|edge| edge.witness)
        .collect::<BTreeSet<_>>();
    let witness_map = witness_keep
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect::<BTreeMap<_, _>>();
    let witnesses = catalog
        .witnesses
        .into_iter()
        .enumerate()
        .filter_map(|(id, witness)| witness_keep.contains(&id).then_some(witness))
        .collect();
    for reached in &mut traversal.reached {
        reached.node = remap[&reached.node];
    }
    for selected in &mut traversal.selected {
        *selected = remap[selected];
    }
    for frontier in &mut traversal.frontier {
        *frontier = remap[frontier];
    }
    for edge in &mut traversal.edges {
        edge.source = remap[&edge.source];
        edge.target = remap[&edge.target];
        edge.witness = witness_map[&edge.witness];
    }
    Ok(GraphAnswer {
        session,
        generation: 1,
        nodes,
        witnesses,
        traversal,
        limitations: catalog.limitations.into_iter().collect(),
    })
}
