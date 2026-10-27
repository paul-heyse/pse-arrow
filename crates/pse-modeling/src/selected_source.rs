// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source requirements before inference. Immutable selectors supply these rows to
//! the existing scientific checker; this module performs no scientific admission.
use crate::Declaration;
use crate::check::provenance_paths;
use pse_authoring::dsl;

/// Complete physical name bindings of authored top-level packages. These remain
/// an explicit physical-registry prerequisite and do not require scientific inference.
pub fn physical_bindings<'a>(
    rows: impl IntoIterator<Item = &'a Declaration>,
    scope: &crate::PhysicalScope,
    quantities: &pse_quantity::QuantityRegistry,
) -> std::collections::BTreeMap<String, pse_ids::SemanticId> {
    let mut bindings = std::collections::BTreeMap::new();
    for package in rows
        .into_iter()
        .filter(|row| row.parent_id.is_none() && scope.sees(row.document_id))
    {
        for (name, value) in quantities.physical_names() {
            let id = match value {
                pse_quantity::PhysicalName::QuantityType(id) => id.as_id(),
                pse_quantity::PhysicalName::ReferenceState(id) => id.as_id(),
            };
            bindings.insert(format!("{}.{name}", package.name), id);
        }
    }
    bindings
}

/// Exact atomic paths named by a registry-owned cell and its nested reference keys.
pub fn cell_paths(cell: &pse_authoring::language::Cell) -> Vec<Vec<String>> {
    use pse_authoring::language::CellSelected;
    match cell.value.selected() {
        Ok(CellSelected::Reference(v)) => vec![v.path.clone()],
        Ok(CellSelected::References(v)) => v
            .paths
            .iter()
            .flat_map(|p| {
                std::iter::once(p.path.clone()).chain(
                    p.keys
                        .iter()
                        .flatten()
                        .filter_map(|key| pse_authoring::language::key_cell_value(key).ok())
                        .flat_map(|cell| cell_paths(&cell)),
                )
            })
            .collect(),
        Ok(CellSelected::Identifier(v)) => vec![v.scheme.clone()],
        Ok(CellSelected::Row(v)) => std::iter::once(v.target.clone())
            .chain(
                v.keys
                    .iter()
                    .filter_map(|key| pse_authoring::language::key_cell_value(key).ok())
                    .flat_map(|cell| cell_paths(&cell)),
            )
            .collect(),
        _ => Vec::new(),
    }
}

/// Nominal type paths, retaining atomic authored names. The source selector
/// excludes declared type parameters before resolving these in a namespace.
pub fn type_paths(row: &Declaration) -> Vec<dsl::Path> {
    requirements(row)
        .types
        .into_iter()
        .flatten()
        .filter_map(|node| node.path.as_ref())
        .map(|parts| dsl::Path {
            segments: parts
                .iter()
                .map(|name| dsl::PathSegment {
                    name: name.clone(),
                    indices: Vec::new(),
                })
                .collect(),
        })
        .collect()
}

/// Structural path segments from registry-owned cells, completeness and provenance.
/// Expressions are collected separately by the source occurrence parser.
pub fn structural_paths(row: &Declaration) -> Vec<dsl::Path> {
    fn path(parts: &[String]) -> dsl::Path {
        dsl::Path {
            segments: parts
                .iter()
                .map(|name| dsl::PathSegment {
                    name: name.clone(),
                    indices: Vec::new(),
                })
                .collect(),
        }
    }
    let requirements = requirements(row);
    requirements
        .sets
        .into_iter()
        .chain(requirements.paths)
        .map(|parts| path(parts))
        .chain(
            requirements
                .cells
                .into_iter()
                .flat_map(cell_paths)
                .map(|parts| path(&parts)),
        )
        .collect()
}

/// A lexical lookup may wait for a protected source read. An absent name and an
/// unread name are distinct, so a selector cannot skip a nearer shadowing scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lookup {
    /// A required immutable membership probe has not completed.
    Pending,
    /// This inspected namespace contains no such declaration.
    Absent,
    /// Source contracts, including all possible guarded representatives.
    Found(Vec<crate::DeclarationId>),
}

/// The scientific checker and immutable source selector provide their own member
/// inventories to one lexical rule. Member validation remains with the checker.
pub trait LexicalLookup {
    /// The already selected lexical ancestor.
    fn declaration(&self, id: crate::DeclarationId) -> Option<&Declaration>;
    /// Whether an argument/parameter binds this atomic name.
    fn bound(&self, owner: crate::DeclarationId, name: &str) -> bool;
    /// An explicit import alias takes priority over an ordinary member.
    fn imported(&mut self, owner: crate::DeclarationId, name: &str) -> Lookup;
    /// Direct, inherited and guarded member contracts of one selected scope.
    fn member(&mut self, owner: crate::DeclarationId, name: &str) -> Lookup;
}

/// Declaration-local lexical parameters shared by checked and pre-admission selection.
/// A checked function signature replaces authored function arguments after normalization.
/// Other binder families keep their registry-owned argument/parameter names.
pub fn lexical_bindings<'a>(
    row: &'a Declaration,
    checked_function: Option<&'a crate::Function>,
) -> impl Iterator<Item = &'a str> {
    checked_function
        .into_iter()
        .flat_map(|function| function.arguments.iter().map(|(name, _)| name.as_str()))
        .chain(
            row.value
                .function
                .iter()
                .flat_map(|v| &v.arguments)
                .map(|p| p.name.as_str())
                .chain(
                    row.value
                        .scope
                        .iter()
                        .flat_map(|v| &v.parameters)
                        .map(|p| p.name.as_str()),
                )
                .chain(
                    row.value
                        .coordinate_map
                        .iter()
                        .flat_map(|v| &v.arguments)
                        .map(|p| p.name.as_str()),
                )
                .chain(
                    row.value
                        .response
                        .iter()
                        .flat_map(|v| &v.arguments)
                        .map(|p| p.name.as_str()),
                )
                .chain(
                    row.value
                        .reconstruction
                        .iter()
                        .flat_map(|v| &v.arguments)
                        .map(|p| p.name.as_str()),
                )
                .chain(
                    row.value
                        .reference_translation
                        .iter()
                        .flat_map(|v| &v.arguments)
                        .map(|p| p.name.as_str()),
                )
                .filter(move |_| checked_function.is_none()),
        )
}

/// Resolve parser-decoded segments with exact lexical shadowing. Pending probes
/// stop the walk before an outer declaration can be mistaken for a nearer name.
pub fn resolve_segments(
    source: &mut impl LexicalLookup,
    mut owner: crate::DeclarationId,
    segments: &[dsl::PathSegment],
) -> Lookup {
    let Some((first, tail)) = segments.split_first() else {
        return Lookup::Absent;
    };
    let mut ancestors = std::collections::BTreeSet::new();
    let selected = loop {
        if !ancestors.insert(owner) {
            return Lookup::Absent;
        }
        if source.bound(owner, &first.name) {
            return Lookup::Absent;
        }
        match source.imported(owner, &first.name) {
            Lookup::Pending => return Lookup::Pending,
            Lookup::Found(ids) => break ids,
            Lookup::Absent => {}
        }
        match source.member(owner, &first.name) {
            Lookup::Pending => return Lookup::Pending,
            Lookup::Found(ids) => break ids,
            Lookup::Absent => {}
        }
        let Some(row) = source.declaration(owner) else {
            return Lookup::Pending;
        };
        if let Some(parent) = row.parent_id {
            owner = parent;
        } else if row.name == first.name {
            break vec![owner];
        } else {
            return Lookup::Absent;
        }
    };
    let mut selected = selected;
    for segment in tail {
        let mut next = std::collections::BTreeSet::new();
        for owner in selected {
            match source.member(owner, &segment.name) {
                Lookup::Pending => return Lookup::Pending,
                Lookup::Found(ids) => next.extend(ids),
                Lookup::Absent => {}
            }
        }
        if next.is_empty() {
            return Lookup::Absent;
        }
        selected = next.into_iter().collect();
    }
    Lookup::Found(selected)
}

/// Resolve a native name without reparsing source expression text. Exact atomic
/// names take priority; a dotted member path is tried only after proved absence.
pub fn resolve_name(
    source: &mut impl LexicalLookup,
    owner: crate::DeclarationId,
    name: &str,
) -> Lookup {
    let atomic = [dsl::PathSegment {
        name: name.into(),
        indices: Vec::new(),
    }];
    match resolve_segments(source, owner, &atomic) {
        Lookup::Absent if name.contains('.') => {
            let segments = name
                .split('.')
                .map(|name| dsl::PathSegment {
                    name: name.into(),
                    indices: Vec::new(),
                })
                .collect::<Vec<_>>();
            resolve_segments(source, owner, &segments)
        }
        result => result,
    }
}

/// Parse exact source fields through the existing occurrence collector before
/// inference or scientific admission. Binder metadata and source locations remain
/// available to the immutable selector.
/// # Errors
/// Invalid source grammar or missing original field locations.
pub fn occurrences(
    row: &Declaration,
    fields: Option<&std::collections::BTreeMap<String, pse_authoring::SourceSpan>>,
) -> crate::Result<
    std::collections::BTreeMap<
        crate::expression::occurrences::OccurrenceKey,
        crate::expression::occurrences::CheckedExpression,
    >,
> {
    struct Fields<'a>(Option<&'a std::collections::BTreeMap<String, pse_authoring::SourceSpan>>);
    impl crate::document::Documents for Fields<'_> {
        fn field_spans(
            &self,
            _: crate::DeclarationId,
        ) -> Option<&std::collections::BTreeMap<String, pse_authoring::SourceSpan>> {
            self.0
        }
        fn resolve(&self, _: pse_ids::SemanticId, _: &str) -> Option<pse_ids::SemanticId> {
            None
        }
        fn admit(
            &self,
            plan: &std::sync::Arc<crate::document::DocumentPlan>,
            document: pse_ids::SemanticId,
        ) -> crate::Result<std::sync::Arc<crate::document::DocumentTable>> {
            crate::document::Documents::admit(&crate::document::NoDocuments, plan, document)
        }
    }
    crate::expression::occurrences::collect(std::slice::from_ref(row), &Fields(fields))
}

/// Exact record identities and their scientific source supplier, after the
/// existing typed dataset/key/default/provenance admission has completed.
/// No key framing or dataset interpretation is duplicated at the storage boundary.
pub fn record_sources(
    package: &crate::CheckedPackage,
) -> impl Iterator<Item = (crate::DeclarationId, crate::DeclarationId)> + '_ {
    package
        .entities
        .iter()
        .map(|(id, record)| (*id, record.origin))
}

/// Dataset admission and identifier/key obligations require all suppliers of a
/// selected nominal contract, including refinements and shared identifier schemes.
pub fn requires_suppliers(row: &Declaration) -> bool {
    row.value.table.is_some()
        || matches!(
            row.value.kind,
            pse_model::generated::enums::ModelingDeclarationKind::EntityKind
                | pse_model::generated::enums::ModelingDeclarationKind::IdentifierScheme
        )
}

/// Concrete scopes require their full member/guard contract. Package ancestors
/// contribute imports and probed names without forcing every unrelated member.
pub fn requires_members(row: &Declaration) -> bool {
    owns_namespace(row)
        && row.value.kind != pse_model::generated::enums::ModelingDeclarationKind::Package
}

/// A typed package marker contributes an implicit shared numerical dependency.
/// Both immutable acquisition and checked projection retain these markers.
pub fn is_engineering_rule_marker(row: &Declaration) -> bool {
    row.value.annotation.as_ref().is_some_and(|annotation| {
        annotation.kind == pse_model::generated::enums::ModelingAnnotationKind::EngineeringRule
    })
}

/// Namespace-bearing declarations in the authored grammar. Ordinary expression,
/// binding and function leaves bind arguments but cannot declare nested members.
pub fn owns_namespace(row: &Declaration) -> bool {
    row.value.scope.is_some() || row.value.coordinate_map.is_some() || row.value.guard.is_some()
}

/// Parsed lexical requirements of one immutable row. Resolved requirements can be
/// discarded while unread requirements wait for the next protected frontier.
#[derive(Debug, Default)]
pub struct DeclarationReferences {
    /// Nominal paths, excluding lexical type parameters at resolution time.
    pub types: Vec<dsl::Path>,
    /// Scientific field/provenance paths whose longest declared prefix is selected.
    pub prefixes: Vec<dsl::Path>,
    /// Atomic names, with dotted spelling fallback after an exact-name absence.
    pub names: Vec<String>,
}

impl pse_model::HeapUsage for DeclarationReferences {
    fn heap_bytes(&self) -> usize {
        fn paths(values: &Vec<dsl::Path>) -> usize {
            values.capacity() * size_of::<dsl::Path>()
                + values
                    .iter()
                    .map(|path| {
                        path.segments.capacity() * size_of::<dsl::PathSegment>()
                            + path
                                .segments
                                .iter()
                                .map(|segment| segment.name.capacity())
                                .sum::<usize>()
                    })
                    .sum::<usize>()
        }
        let Self { types, prefixes, names } = self;
        paths(types)
            .saturating_add(paths(prefixes))
            .saturating_add(names.capacity().saturating_mul(size_of::<String>()))
            .saturating_add(names.iter().fold(0usize, |bytes, name| bytes.saturating_add(name.capacity())))
    }
}

/// Parse each immutable source field once, preserving the existing occurrence
/// collector's binder exclusions and quoted atomic path interpretation.
/// # Errors
/// Invalid source grammar or missing original field locations.
pub fn declaration_references(
    row: &Declaration,
    fields: Option<&std::collections::BTreeMap<String, pse_authoring::SourceSpan>>,
) -> crate::Result<DeclarationReferences> {
    let parsed = crate::expression::occurrences::ParsedFields::parse(row, fields)?;
    declaration_references_parsed(row, &parsed)
}

/// Extract lexical requirements from the same immutable grammar fields consumed
/// by checked admission, without retaining resolved bindings or physical assurance.
pub fn declaration_references_parsed(
    row: &Declaration,
    parsed: &crate::expression::occurrences::ParsedFields,
) -> crate::Result<DeclarationReferences> {
    let occurrences = parsed.expressions();
    let mut result = DeclarationReferences {
        types: type_paths(row),
        prefixes: structural_paths(row),
        names: Vec::new(),
    };
    result
        .names
        .extend(requirements(row).names.into_iter().map(str::to_owned));
    for occurrence in occurrences.values() {
        for reference in syntax_references(&occurrence.syntax) {
            match reference {
                SourceReference::Name(name) => result.names.push(name),
                SourceReference::Segments(path) => {
                    if !path.segments.first().is_some_and(|segment| {
                        occurrence
                            .index_obligations
                            .iter()
                            .any(|(name, _)| name == &segment.name)
                    }) {
                        result.prefixes.push(path);
                    }
                }
            }
        }
    }
    // The occurrence walker already visits index expressions independently. Only
    // decoded names are consumed by lexical resolution; do not retain their ASTs.
    for path in result.types.iter_mut().chain(&mut result.prefixes) {
        for segment in &mut path.segments {
            segment.indices = Vec::new();
        }
    }
    Ok(result)
}

/// Borrowed source fields whose names/types/cells require lexical selection.
/// Every field retains its registry-generated type; this is an algorithm view.
#[derive(Debug)]
pub struct SourceRequirements<'a> {
    /// Native structural source spellings; expressions come only from parsed occurrences.
    pub names: Vec<&'a str>,
    /// Existing authored type arenas.
    pub types: Vec<&'a Vec<pse_authoring::language::TypeNode>>,
    /// Existing authored cells.
    pub cells: Vec<&'a pse_authoring::language::Cell>,
    /// Completeness scopes.
    pub sets: Vec<&'a Vec<String>>,
    /// Provenance, roles and lineage.
    pub paths: Vec<&'a Vec<String>>,
}

/// Collect source-owned structural requirements before checking or hydrating a package.
/// The checked-package selector consumes this same view.
pub fn requirements(row: &Declaration) -> SourceRequirements<'_> {
    // Only native structural spellings belong here. Grammar-bearing fields are
    // declared by the registry and collected under exact OccurrenceKey values.
    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut cells = Vec::new();
    let mut sets = Vec::new();
    let mut paths = Vec::new();
    if let Some(v) = &row.value.scope {
        names.extend(v.bases.iter().map(String::as_str));
        paths.extend(v.oracle.iter());
        if let Some(v) = v.fixture.as_ref().and_then(|f| f.expected_failure.as_ref())
            && let Some(v) = &v.applicability
        {
            names.extend([v.claim.as_str(), v.form.as_str()]);
        }
        types.extend(v.parameters.iter().map(|p| &p.r#type));
    }
    if let Some(v) = &row.value.binding {
        types.extend(v.r#type.as_ref());
    }
    if let Some(v) = &row.value.function {
        types.push(&v.return_type);
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.applicability {
        names.push(v.owner.as_str());
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.coordinate_map {
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.reconstruction {
        names.extend([v.map.as_str(), v.reference.as_str()]);
        types.push(&v.return_type);
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.response {
        types.push(&v.return_type);
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.reference_translation {
        names.extend([v.source_anchor.as_str(), v.target_anchor.as_str()]);
        types.push(&v.return_type);
        types.extend(v.arguments.iter().map(|a| &a.r#type));
        paths.extend(provenance_paths(&v.provenance));
    }
    if let Some(v) = &row.value.table {
        types.extend(v.value_type.as_ref());
        cells.extend(v.default_value.as_ref());
        types.extend(v.keys.iter().map(|k| &k.r#type));
        types.extend(v.columns.iter().map(|c| &c.r#type));
        sets.extend(v.complete_over.iter().filter_map(|e| e.set.as_ref()));
        types.extend(v.envelopes.iter().map(|e| &e.r#type));
    }
    if let Some(v) = &row.value.envelope {
        types.push(&v.r#type);
    }
    if let Some(v) = &row.value.dataset {
        names.push(v.target.as_str());
        paths.extend(provenance_paths(&v.provenance));
        sets.extend(v.complete_over.iter().filter_map(|e| e.set.as_ref()));
        cells.extend(v.bindings.iter().map(|b| &b.value));
        for r in &v.rows {
            cells.extend(r.keys.iter().chain(&r.values));
        }
    }
    if let Some(v) = &row.value.entity {
        names.push(v.kind_name.as_str());
        cells.extend(v.attributes.iter().map(|a| &a.value));
        paths.extend(v.provenance.iter().flat_map(provenance_paths));
        paths.extend(
            v.attributes
                .iter()
                .flat_map(|a| a.provenance.iter().flat_map(provenance_paths)),
        );
    }
    if let Some(v) = &row.value.attribute {
        types.extend(v.r#type.as_ref());
        cells.extend(v.value.as_ref());
    }
    if let Some(v) = &row.value.constant {
        types.push(&v.r#type);
        cells.push(&v.value);
        paths.extend(provenance_paths(&v.provenance));
    }
    if let Some(v) = &row.value.accumulator {
        types.push(&v.r#type);
    }
    if let Some(v) = &row.value.inventory_balance {
        types.push(&v.r#type);
    }
    if let Some(v) = &row.value.continuous {
        types.push(&v.r#type);
    }
    if let Some(v) = &row.value.discretization {
        names.extend([v.target.as_str(), v.scheme.as_str()]);
    }
    if let Some(v) = &row.value.realization {
        names.push(v.target.as_str());
    }
    if let Some(v) = &row.value.temporal {
        names.extend([v.target.as_str(), v.axis.as_str()]);
    }
    SourceRequirements {
        names,
        types,
        cells,
        sets,
        paths,
    }
}

/// An atomic source reference for the existing lexical resolver.
#[derive(Clone, Debug, PartialEq)]
pub enum SourceReference {
    /// Structured atomic segments preserve quoted dots and index expression paths.
    Segments(dsl::Path),
    /// A source spelling from the shared static application grammar.
    Name(String),
}

/// Traverse one parser-owned syntax tree without requiring inferred types or a package.
/// The checked selector uses this same traversal.
pub fn syntax_references(value: &crate::expression::occurrences::Syntax) -> Vec<SourceReference> {
    use pse_authoring::{
        dsl::{Equation, EquationKind, Expr},
        language::StaticValue,
    };
    fn path(path: &dsl::Path, out: &mut Vec<SourceReference>) {
        out.push(SourceReference::Segments(path.clone()));
    }
    fn expression(e: &Expr, out: &mut Vec<SourceReference>) {
        for value in crate::expression::source_paths(e) {
            path(value, out);
        }
    }
    fn predicate(value: &dsl::Predicate, out: &mut Vec<SourceReference>) {
        use dsl::PredicateKind as K;
        match &value.kind {
            K::Compare { lhs, rhs, .. } => {
                expression(lhs, out);
                expression(rhs, out);
            }
            K::In { expr, domain } => {
                expression(expr, out);
                path(domain, out);
            }
            K::Atom(e) => expression(e, out),
            K::And(a, b) | K::Or(a, b) => {
                predicate(a, out);
                predicate(b, out);
            }
            K::Not(p) => predicate(p, out),
            _ => {}
        }
    }
    fn bound(reference: &SourceReference, names: &std::collections::BTreeSet<String>) -> bool {
        match reference {
            SourceReference::Segments(path) => path
                .segments
                .first()
                .is_some_and(|first| names.contains(&first.name)),
            SourceReference::Name(name) => {
                names.contains(name)
                    || name
                        .split_once('.')
                        .is_some_and(|(first, _)| names.contains(first))
            }
        }
    }
    fn syntax(
        value: &StaticValue,
        names: &std::collections::BTreeSet<String>,
        out: &mut Vec<SourceReference>,
    ) {
        match value {
            StaticValue::Expression(e) => {
                let mut local = Vec::new();
                expression(e, &mut local);
                out.extend(
                    local
                        .into_iter()
                        .filter(|reference| !bound(reference, names)),
                );
            }
            StaticValue::Set(values) | StaticValue::Tuple(values) => {
                for value in values {
                    syntax(value, names, out);
                }
            }
            StaticValue::Comprehension {
                body,
                bindings,
                filter,
            } => {
                let mut local = names.clone();
                for (name, domain) in bindings {
                    syntax(domain, &local, out);
                    local.insert(name.clone());
                }
                if let Some(filter) = filter {
                    let mut references = Vec::new();
                    predicate(filter, &mut references);
                    out.extend(
                        references
                            .into_iter()
                            .filter(|reference| !bound(reference, &local)),
                    );
                }
                syntax(body, &local, out);
            }
            StaticValue::Apply { name, arguments } => {
                let reference = SourceReference::Name(name.clone());
                if !bound(&reference, names) {
                    out.push(reference);
                }
                for (_, value) in arguments {
                    syntax(value, names, out);
                }
            }
            StaticValue::Text(_) => {}
        }
    }
    fn equation(value: &Equation, out: &mut Vec<SourceReference>) {
        match &value.kind {
            EquationKind::Relation { lhs, rhs, .. } => {
                expression(lhs, out);
                expression(rhs, out);
            }
            EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                predicate(guard, out);
                equation(then, out);
                equation(otherwise, out);
            }
        }
    }
    use crate::expression::occurrences::Syntax;
    let mut out = Vec::new();
    match value {
        Syntax::Expression(value) => expression(value, &mut out),
        Syntax::Static(value) => syntax(value, &std::collections::BTreeSet::new(), &mut out),
        Syntax::Equation(value) => equation(value, &mut out),
        Syntax::Predicate(value) => predicate(value, &mut out),
        Syntax::Logic(value) => value.expressions(&mut |value| expression(value, &mut out)),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_authoring::language;
    use pse_ids::SemanticId;

    #[test]
    fn unread_nearer_scope_cannot_resolve_to_an_outer_shadow() {
        struct Source {
            rows: Vec<Declaration>,
            inner: crate::DeclarationId,
            outer: crate::DeclarationId,
            nearer: Lookup,
        }
        impl LexicalLookup for Source {
            fn declaration(&self, id: crate::DeclarationId) -> Option<&Declaration> {
                self.rows.iter().find(|row| row.declaration_id == id)
            }
            fn bound(&self, _: crate::DeclarationId, _: &str) -> bool {
                false
            }
            fn imported(&mut self, _: crate::DeclarationId, _: &str) -> Lookup {
                Lookup::Absent
            }
            fn member(&mut self, owner: crate::DeclarationId, _: &str) -> Lookup {
                if owner == self.inner {
                    self.nearer.clone()
                } else {
                    Lookup::Found(vec![self.outer])
                }
            }
        }
        let rows = language::parse(
            "package p {param x:Scalar=1; def D {param x:Scalar=2;} }",
            SemanticId::NIL,
            language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let inner = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .declaration_id;
        let outer = rows
            .iter()
            .find(|row| row.name == "x" && row.parent_id != Some(inner))
            .unwrap()
            .declaration_id;
        let shadow = rows
            .iter()
            .find(|row| row.name == "x" && row.parent_id == Some(inner))
            .unwrap()
            .declaration_id;
        let path = [dsl::PathSegment {
            name: "x".into(),
            indices: Vec::new(),
        }];
        let mut source = Source {
            rows,
            inner,
            outer,
            nearer: Lookup::Pending,
        };
        assert_eq!(resolve_segments(&mut source, inner, &path), Lookup::Pending);
        source.nearer = Lookup::Found(vec![shadow]);
        assert_eq!(
            resolve_segments(&mut source, inner, &path),
            Lookup::Found(vec![shadow])
        );
        source.nearer = Lookup::Absent;
        assert_eq!(
            resolve_segments(&mut source, inner, &path),
            Lookup::Found(vec![outer])
        );
    }

    #[test]
    fn source_requirements_precede_type_admission_and_preserve_original_fields() {
        let source = "package p {def D {var x[j in Item]:UnknownScientificType; eq e[j in Item]:'law.table'(x[j])==offset;} }";
        let (rows, spans) = language::parse_with_spans(
            source,
            SemanticId::NIL,
            language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let variable = rows.iter().find(|row| row.name == "x").unwrap();
        let requirements = requirements(variable);
        assert!(requirements.names.is_empty());
        let references =
            declaration_references(variable, spans.get(&variable.declaration_id)).unwrap();
        assert_eq!(references.prefixes[0].ident(), Some("Item"));
        assert_eq!(requirements.types.len(), 1);
        let equation = rows.iter().find(|row| row.name == "e").unwrap();
        let occurrences = occurrences(equation, spans.get(&equation.declaration_id)).unwrap();
        let value = occurrences
            .iter()
            .find(|(key, _)| key.role == "equation.expression")
            .unwrap()
            .1;
        assert!(value.declared_type.is_none());
        assert!(value.dependencies.is_empty());
        assert_eq!(value.index_obligations, [("j".into(), "Item".into())]);
        assert_eq!(
            &source[value.source.start as usize..value.source.end as usize],
            value.text
        );
        let paths = syntax_references(&value.syntax)
            .into_iter()
            .filter_map(|reference| match reference {
                SourceReference::Segments(path) => Some(
                    path.segments
                        .iter()
                        .map(|segment| segment.name.clone())
                        .collect::<Vec<_>>(),
                ),
                SourceReference::Name(_) => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            paths.contains(&vec!["law.table".into()]),
            "quoted dot stays atomic"
        );
        assert!(paths.contains(&vec!["x".into()]));
        assert!(paths.contains(&vec!["j".into()]));
        assert!(paths.contains(&vec!["offset".into()]));
        let references =
            declaration_references(equation, spans.get(&equation.declaration_id)).unwrap();
        let paths = references
            .prefixes
            .iter()
            .map(|path| {
                path.segments
                    .iter()
                    .map(|segment| segment.name.as_str())
                    .collect::<Vec<_>>()
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert!(paths.contains(&vec!["law.table"]));
        assert!(paths.contains(&vec!["x"]));
        assert!(paths.contains(&vec!["offset"]));
        assert!(
            !paths.contains(&vec!["j"]),
            "index binder is excluded while its Item set remains required"
        );
        assert!(
            paths.contains(&vec!["Item"]),
            "the authored index set remains a required declared prefix"
        );
        assert!(
            references
                .prefixes
                .iter()
                .flat_map(|path| &path.segments)
                .all(|segment| segment.indices.capacity() == 0)
        );
    }

    #[test]
    fn member_inventory_policy_follows_authored_containers_not_expression_leaves() {
        let rows = language::parse(
            "package p {enum Item {a,b}; fn law(x:Scalar)->Scalar=x; def D {param g:Scalar=1; var x:Scalar; when g>0 {eq e:law(x)==1;} } }",
            SemanticId::NIL, language::IdentityPolicy::Named, Default::default(),
        ).unwrap();
        for row in &rows {
            let expected = matches!(
                row.value.kind,
                pse_model::generated::enums::ModelingDeclarationKind::Package
                    | pse_model::generated::enums::ModelingDeclarationKind::Definition
                    | pse_model::generated::enums::ModelingDeclarationKind::When
            );
            assert_eq!(owns_namespace(row), expected, "{}", row.name);
            assert_eq!(
                requires_members(row),
                expected && row.name != "p",
                "{}",
                row.name
            );
        }
        assert!(rows.iter().all(|row| row.parent_id.is_none_or(|parent| {
            owns_namespace(
                rows.iter()
                    .find(|row| row.declaration_id == parent)
                    .unwrap(),
            )
        })));
    }

    #[test]
    fn conditional_dependency_closure_keeps_both_arms_and_guard() {
        let rows = language::parse(
            "package p {def D {eq e:if active > 0 then left == left_reference else right == right_reference;} }",
            SemanticId::NIL, language::IdentityPolicy::Named, Default::default(),
        ).unwrap();
        let equation = rows.iter().find(|row| row.name == "e").unwrap();
        let occurrences = occurrences(equation, None).unwrap();
        let value = occurrences
            .iter()
            .find(|(key, _)| key.role == "equation.expression")
            .unwrap()
            .1;
        let names = syntax_references(&value.syntax)
            .into_iter()
            .filter_map(|reference| match reference {
                SourceReference::Segments(path) => path.ident().map(str::to_owned),
                SourceReference::Name(name) => Some(name),
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            names,
            [
                "active",
                "left",
                "left_reference",
                "right",
                "right_reference"
            ]
            .into_iter()
            .map(str::to_owned)
            .collect()
        );
    }
}

#[cfg(test)]
mod occurrence_contract_tests {
    use super::*;

    #[test]
    fn index_domain_precedes_its_own_binder() {
        let rows = crate::kernel_types::source("package p {def Root {var x[i in i]:Scalar;}}");
        let row = rows.iter().find(|row| row.name == "x").unwrap();
        let parsed = crate::expression::occurrences::ParsedFields::parse(row, None).unwrap();
        let reference = declaration_references_parsed(row, &parsed).unwrap();
        assert!(requirements(row).names.is_empty());
        assert_eq!(reference.prefixes.len(), 1);
        assert_eq!(reference.prefixes[0].ident(), Some("i"));
        let domain = parsed.expressions().values().next().unwrap();
        assert!(
            domain.index_obligations.is_empty(),
            "domain is evaluated before i is bound"
        );
    }

    #[test]
    fn static_comprehension_and_intrinsic_callees_share_the_checked_reference_walk() {
        let rows = crate::kernel_types::source(
            "package p {entity kind item {} entity item a {} entity item j {} set items:Set<item>={a}; set selected:Set<item>={j for j in items where j!=a}; param count:Integer=size(items);}",
        );
        let selected = rows.iter().find(|row| row.name == "selected").unwrap();
        let references = declaration_references(selected, None).unwrap();
        let names = references
            .prefixes
            .iter()
            .filter_map(dsl::Path::ident)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names, std::collections::BTreeSet::from(["items", "a"]));
        let count = rows.iter().find(|row| row.name == "count").unwrap();
        let references = declaration_references(count, None).unwrap();
        assert_eq!(
            references
                .prefixes
                .iter()
                .filter_map(dsl::Path::ident)
                .collect::<Vec<_>>(),
            ["items"]
        );
    }

    #[test]
    fn native_dependencies_and_equal_expression_fields_remain_independent() {
        let rows = crate::kernel_types::source(
            "package p {def Root extends shared {var x:Scalar; annotation bounds x(shared,shared);}}",
        );
        let root = rows.iter().find(|row| row.name == "Root").unwrap();
        let bounds = rows
            .iter()
            .find(|row| row.value.annotation.is_some())
            .unwrap();
        assert_eq!(
            declaration_references(root, None).unwrap().names,
            ["shared"]
        );
        let parsed = crate::expression::occurrences::ParsedFields::parse(bounds, None).unwrap();
        let expressions = parsed.expressions();
        assert!(
            expressions
                .keys()
                .any(|key| key.role == "annotation.arguments" && key.position == 0)
        );
        assert!(
            expressions
                .keys()
                .any(|key| key.role == "annotation.arguments" && key.position == 1)
        );
        assert!(requirements(bounds).names.is_empty());
        let references = declaration_references_parsed(bounds, &parsed).unwrap();
        assert_eq!(
            references
                .prefixes
                .iter()
                .filter(|path| path.ident() == Some("shared"))
                .count(),
            2
        );
    }
}
