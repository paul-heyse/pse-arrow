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
    /// Structural spellings which require the entire path.
    pub exact: Vec<dsl::Path>,
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
        paths(&self.types)
            + paths(&self.exact)
            + paths(&self.prefixes)
            + self.names.capacity() * size_of::<String>()
            + self.names.iter().map(String::capacity).sum::<usize>()
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
    let occurrences = occurrences(row, fields)?;
    let mut result = DeclarationReferences {
        types: type_paths(row),
        exact: Vec::new(),
        prefixes: structural_paths(row),
        names: Vec::new(),
    };
    for text in requirements(row).names {
        if occurrences
            .values()
            .any(|occurrence| occurrence.text == text)
        {
            continue;
        }
        if let Ok(expression) = dsl::parse_expr(text) {
            for reference in syntax_references(&crate::expression::occurrences::Syntax::Expression(
                expression,
            )) {
                match reference {
                    SourceReference::Name(name) => result.names.push(name),
                    SourceReference::Segments(path) => result.exact.push(path),
                }
            }
        } else {
            result.names.push(text.into());
        }
    }
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
    for path in result
        .types
        .iter_mut()
        .chain(&mut result.exact)
        .chain(&mut result.prefixes)
    {
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
    /// Direct source spellings and expressions; parsed occurrences supply atomic references.
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
    let mut texts: Vec<&str> = Vec::new();
    let mut types: Vec<&Vec<pse_authoring::language::TypeNode>> = Vec::new();
    // Cells name declarations only as reference paths (ADR-0123 Outcome 1).
    let mut cells: Vec<&pse_authoring::language::Cell> = Vec::new();
    // Completeness names declared sets and enumerations by path (Outcome 3).
    let mut sets: Vec<&Vec<String>> = Vec::new();
    // Provenance and oracles name sources, roles and lineage by path (Outcome 5).
    let mut paths: Vec<&Vec<String>> = Vec::new();
    if let Some(v) = &row.value.scope {
        texts.extend(v.bases.iter().map(String::as_str));
        if let Some(selection) = &v.selection {
            texts.push(&selection.criterion);
            texts.push(&selection.tolerance);
        }
        texts.extend(v.branch.as_deref());
        if let Some(operation) = &v.operational {
            texts.extend(operation.anchors.iter().map(|a| a.expression.as_str()));
            texts.extend(operation.neighborhood.as_deref());
        }
        texts.extend(v.eligibility.as_deref());
        // A test depends on its oracle source (ADR-0123 Outcome 5).
        paths.extend(v.oracle.iter());
        if let Some(fixture) = &v.fixture {
            if let Some(expected) = &fixture.expected_failure
                && let Some(v) = &expected.applicability
            {
                texts.extend([v.claim.as_str(), v.form.as_str()]);
                texts.extend(v.sets.iter().map(String::as_str));
            }
            if let Some(integration) = &fixture.integration {
                texts.extend(integration.samples.iter().map(String::as_str));
                texts.push(&integration.initial_step);
                for q in &integration.quadratures {
                    texts.push(&q.target);
                    texts.push(&q.absolute_tolerance);
                }
                for s in &integration.schedules {
                    texts.push(&s.target);
                    texts.extend(
                        s.times
                            .iter()
                            .chain(&s.values)
                            .chain(s.lower.iter().chain(&s.upper))
                            .map(String::as_str),
                    );
                }
            }
            if let Some(shooting) = &fixture.shooting {
                texts.extend(shooting.nodes.iter().map(String::as_str));
            }
            for e in fixture.modes.iter().flat_map(|m| &m.events) {
                texts.push(&e.guard);
                texts.push(&e.tolerance);
                for r in &e.reset {
                    texts.push(&r.target);
                    texts.push(&r.expression);
                }
            }
            for s in &fixture.specifications {
                texts.push(&s.target);
                texts.extend(s.expression.as_deref());
            }
        }
        for p in &v.parameters {
            types.push(&p.r#type);
            texts.extend(p.default_value.as_deref());
        }
    }
    if let Some(v) = &row.value.binding {
        types.extend(v.r#type.as_ref());
        texts.extend(v.expression.as_deref());
        texts.extend(v.defined_by.as_deref());
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.function {
        if let Some(external) = &v.external {
            texts.push(&external.output);
        }
        texts.extend(v.body.as_deref());
        texts.extend(v.validity.as_deref());
        texts.extend(v.applicability.iter().map(String::as_str));
        types.push(&v.return_type);
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.applicability {
        texts.extend([v.owner.as_str(), v.evidence.as_str()]);
        texts.extend(v.predicate.as_deref());
        texts.extend(v.lower.as_deref());
        texts.extend(v.upper.as_deref());
        texts.extend(
            v.alternatives
                .iter()
                .chain(&v.dependencies)
                .map(String::as_str),
        );
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.permission {
        texts.extend(v.targets.iter().map(String::as_str));
    }
    if let Some(v) = &row.value.coordinate_map {
        types.extend(v.arguments.iter().map(|a| &a.r#type));
        texts.extend(v.validity.as_deref());
    }
    if let Some(v) = &row.value.coordinate_slot {
        texts.push(&v.expression);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.reconstruction {
        texts.extend([
            v.map.as_str(),
            v.reference.as_str(),
            v.normalization.as_str(),
        ]);
        types.push(&v.return_type);
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.response {
        texts.push(&v.body);
        types.push(&v.return_type);
        types.extend(v.arguments.iter().map(|a| &a.r#type));
    }
    if let Some(v) = &row.value.reference_translation {
        texts.extend([
            v.source_anchor.as_str(),
            v.target_anchor.as_str(),
            v.temperature.as_str(),
            v.pressure.as_str(),
        ]);
        types.push(&v.return_type);
        types.extend(v.arguments.iter().map(|a| &a.r#type));
        paths.extend(provenance_paths(&v.provenance));
    }
    if let Some(v) = &row.value.boundary {
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.exchange {
        texts.extend([v.from.as_str(), v.to.as_str()]);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.table {
        types.extend(v.value_type.as_ref());
        cells.extend(v.default_value.as_ref());
        types.extend(v.keys.iter().map(|k| &k.r#type));
        types.extend(v.columns.iter().map(|c| &c.r#type));
        texts.extend(v.columns.iter().filter_map(|c| c.derived.as_deref()));
        texts.extend(v.requirements.iter().map(String::as_str));
        sets.extend(v.complete_over.iter().filter_map(|e| e.set.as_ref()));
        types.extend(v.envelopes.iter().map(|e| &e.r#type));
    }
    if let Some(v) = &row.value.envelope {
        types.push(&v.r#type);
    }
    if let Some(v) = &row.value.dataset {
        texts.push(&v.target);
        paths.extend(provenance_paths(&v.provenance));
        sets.extend(v.complete_over.iter().filter_map(|e| e.set.as_ref()));
        cells.extend(v.bindings.iter().map(|b| &b.value));
        for r in &v.rows {
            cells.extend(r.keys.iter().chain(&r.values));
        }
    }
    if let Some(v) = &row.value.entity {
        texts.push(&v.kind_name);
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
        texts.extend(v.derived.as_deref());
    }
    if let Some(v) = &row.value.constant {
        types.push(&v.r#type);
        cells.push(&v.value);
        paths.extend(provenance_paths(&v.provenance));
    }
    if let Some(v) = &row.value.equation {
        texts.push(&v.expression);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
        texts.extend(v.condition.as_ref().map(|c| c.variable.as_str()));
    }
    if let Some(v) = &row.value.ordered_set {
        texts.extend([v.member.as_str(), v.weight.as_str()]);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.cardinality {
        texts.extend([v.count.as_str(), v.member.as_str()]);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.piecewise {
        texts.extend([
            v.output.as_str(),
            v.input.as_str(),
            v.abscissa.as_str(),
            v.ordinate.as_str(),
        ]);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.logic {
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.complementarity {
        texts.extend([v.first.as_str(), v.second.as_str()]);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.realization {
        texts.extend(v.argument.as_deref());
        texts.extend(v.function.as_deref());
    }
    if let Some(v) = &row.value.guard {
        texts.push(&v.predicate);
    }
    if let Some(v) = &row.value.requirement {
        texts.push(&v.predicate);
    }
    if let Some(v) = &row.value.accumulator {
        types.push(&v.r#type);
        texts.push(&v.tolerance);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.contribution {
        texts.push(&v.target);
        texts.push(&v.expression);
        texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
    }
    if let Some(v) = &row.value.connection {
        texts.push(&v.from);
        texts.push(&v.to);
    }
    if let Some(v) = &row.value.annotation {
        texts.push(&v.target);
        texts.extend(v.arguments.iter().map(String::as_str));
        if let Some(o) = &v.objective {
            texts.extend(
                [
                    &o.weight,
                    &o.normalization,
                    &o.absolute_tolerance,
                    &o.relative_tolerance,
                ]
                .into_iter()
                .flatten()
                .map(String::as_str),
            );
        }
    }
    if let Some(v) = &row.value.continuous {
        types.push(&v.r#type);
        texts.extend([v.lower.as_str(), v.upper.as_str()]);
    }
    if let Some(v) = &row.value.discretization {
        texts.extend([
            v.target.as_str(),
            v.scheme.as_str(),
            v.elements.as_str(),
            v.order.as_str(),
        ]);
    }
    if let Some(v) = &row.value.realization {
        texts.push(&v.target);
    }
    if let Some(v) = &row.value.temporal {
        texts.extend([v.target.as_str(), v.axis.as_str()]);
    }
    if let Some(v) = &row.value.relaxation {
        texts.extend([v.target.as_str(), v.nominal.as_str()]);
    }
    if let Some(v) = &row.value.continuation {
        texts.extend([v.target.as_str(), v.start.as_str(), v.end.as_str()]);
    }
    if let Some(v) = &row.value.expectation {
        texts.extend([v.actual.as_str(), v.expected.as_str(), v.tolerance.as_str()]);
        texts.extend(v.relative_tolerance.as_deref());
    }
    SourceRequirements {
        names: texts,
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
        dsl::{Equation, EquationKind, Expr, ExprKind},
        language::StaticValue,
    };
    fn path(path: &dsl::Path, out: &mut Vec<SourceReference>) {
        out.push(SourceReference::Segments(path.clone()));
    }
    fn expression(e: &Expr, out: &mut Vec<SourceReference>) {
        for value in e.free_paths() {
            path(value, out);
        }
        e.walk(|node| call(node, out));
    }
    fn call(node: &Expr, out: &mut Vec<SourceReference>) {
        let name = match &node.kind {
            ExprKind::NamedCall { name, .. } => Some(name),
            ExprKind::Partial { function, .. } => Some(function),
            _ => None,
        };
        if let Some(name) = name {
            path(name, out);
        }
    }
    fn predicate(value: &dsl::Predicate, out: &mut Vec<SourceReference>) {
        for value in value.paths() {
            path(value, out);
        }
        value.walk_expressions(|node| call(node, out));
    }
    fn syntax(value: &StaticValue, out: &mut Vec<SourceReference>) {
        match value {
            StaticValue::Expression(e) => expression(e, out),
            StaticValue::Set(values) | StaticValue::Tuple(values) => {
                for v in values {
                    syntax(v, out);
                }
            }
            StaticValue::Comprehension {
                body,
                bindings,
                filter,
            } => {
                for (_, v) in bindings {
                    syntax(v, out);
                }
                syntax(body, out);
                if let Some(filter) = filter {
                    predicate(filter, out);
                }
            }
            StaticValue::Apply { name, arguments } => {
                out.push(SourceReference::Name(name.clone()));
                for (_, v) in arguments {
                    syntax(v, out);
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
        Syntax::Static(value) => syntax(value, &mut out),
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
        assert_eq!(requirements.names, ["Item"]);
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
