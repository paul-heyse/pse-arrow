// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Local immutable instance construction facts; global root effects stay with Engine.
use super::*;
use std::sync::Arc;

/// Complete local membership/absence and transitive static evaluation premises.
/// Numerical member bodies are intentionally not part of this product.
#[derive(Clone, Debug, PartialEq)]
pub struct PreludePremises {
    pub(super) package: Arc<CheckedPackage>,
    pub(super) selection: crate::check::StaticPackageSelection,
    members: BTreeMap<String, PreludeMember>,
    conditional: BTreeMap<DeclarationId, Vec<PreludeMember>>,
}
#[derive(Clone, Debug, PartialEq)]
struct PreludeMember {
    id: DeclarationId,
    name: String,
    kind: Kind,
    is_override: bool,
}
impl PreludeMember {
    fn of(row: &Declaration) -> Self {
        Self {
            id: row.declaration_id,
            name: row.name.clone(),
            kind: row.value.kind,
            is_override: row.is_override,
        }
    }
}
impl PreludePremises {
    /// Project before local derivation so unrelated numerical bodies cannot be read.
    /// # Errors
    /// Missing definition or invalid static dependency projection.
    pub fn select(package: &CheckedPackage, bindings: &PreludeBindings) -> Result<Self> {
        let definition = bindings.definition;
        let members = package
            .members
            .get(&definition)
            .into_iter()
            .flat_map(|members| members.iter())
            .map(|(name, id)| (name.clone(), PreludeMember::of(&package.declarations[id])))
            .collect();
        let mut pending = package
            .members
            .get(&definition)
            .into_iter()
            .flat_map(|members| members.values())
            .copied()
            .collect::<Vec<_>>();
        let mut conditional = BTreeMap::new();
        while let Some(id) = pending.pop() {
            if matches!(
                package.declarations[&id].value.kind,
                Kind::When | Kind::Stage
            ) && !conditional.contains_key(&id)
            {
                let children = package
                    .children
                    .get(&id)
                    .into_iter()
                    .flatten()
                    .map(|child| PreludeMember::of(&package.declarations[child]))
                    .collect::<Vec<_>>();
                pending.extend(children.iter().map(|child| child.id));
                conditional.insert(id, children);
            }
        }
        fn dependencies(value: &Value, out: &mut Vec<DeclarationId>) {
            match value {
                Value::Function(id) => out.push(*id),
                Value::Definition { id, bindings } => {
                    out.push(*id);
                    for value in bindings.values() {
                        dependencies(value, out);
                    }
                }
                Value::Entity { id, kind } => {
                    out.push(*id);
                    out.push(*kind);
                }
                Value::Enum { enumeration, .. } => out.push(*enumeration),
                Value::Identifier { scheme, .. } => out.push(*scheme),
                Value::Row { table, fields, .. } => {
                    out.push(*table);
                    for value in fields.iter() {
                        dependencies(value, out);
                    }
                }
                Value::Set(values) | Value::Tuple(values) => {
                    for value in values {
                        dependencies(value, out);
                    }
                }
                Value::Boolean(_)
                | Value::Integer(_)
                | Value::Number { .. }
                | Value::Coordinate { .. }
                | Value::Text(_)
                | Value::QuantityType(_)
                | Value::ReferenceState(_)
                | Value::Missing => {}
            }
        }
        let mut additional = Vec::new();
        for value in [
            &bindings.arguments,
            &bindings.scope,
            &bindings.inherited,
            &bindings.facts,
        ]
        .into_iter()
        .flat_map(|environment| environment.values())
        {
            dependencies(value, &mut additional);
        }
        let selection = crate::check::StaticPackageSelection::static_scope(definition, additional);
        Ok(Self {
            package: Arc::new(selection.select(package)?),
            selection,
            members,
            conditional,
        })
    }
    /// Conservative owned extent under the workspace's finite retained-input policy.
    pub fn retained_bytes(&self) -> usize {
        self.package
            .retained_bytes()
            .saturating_add(self.selection.retained_bytes())
            .saturating_add(self.members.len().saturating_mul(256))
            .saturating_add(
                self.members
                    .values()
                    .map(|member| member.name.capacity())
                    .sum::<usize>(),
            )
            .saturating_add(
                self.conditional
                    .values()
                    .map(|members| {
                        members.capacity() * size_of::<PreludeMember>()
                            + members
                                .iter()
                                .map(|member| member.name.capacity())
                                .sum::<usize>()
                    })
                    .sum::<usize>(),
            )
    }
}
/// Explicit local binding facts. No Salsa handle enters project semantic identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreludeBindings {
    /// Actual declaration, separate from a process-local query key.
    pub definition: DeclarationId,
    /// Root test/production permission for static data reads.
    pub root: DeclarationId,
    /// Whether the admitted root is a test fixture.
    pub fixture: bool,
    /// Constructor arguments under authored names.
    pub arguments: Environment,
    /// Published scope values under authored names.
    pub scope: Environment,
    /// Only implicit instances inherit an enclosing environment.
    pub inherited: Environment,
    /// Typed analysis selection facts.
    pub facts: Environment,
    /// Finite structural evaluation allowance.
    pub members: usize,
}
impl PreludeBindings {
    /// Conservative retained-key extent, excluding unreported library heaps.
    pub fn retained_bytes(&self) -> usize {
        [&self.arguments, &self.scope, &self.inherited, &self.facts]
            .into_iter()
            .flat_map(|env| env.iter())
            .fold(size_of::<Self>(), |bytes, (name, value)| {
                bytes
                    .saturating_add(name.capacity())
                    .saturating_add(value.retained_bytes())
                    .saturating_add(128)
            })
    }
}
/// Immutable local result shared across root assemblies and observation choices.
#[derive(Clone, Debug, PartialEq)]
pub struct InstancePrelude {
    pub(crate) environment: Environment,
    pub(crate) members: BTreeMap<String, DeclarationId>,
    pub(crate) stages: BTreeSet<String>,
    pub(crate) selections: crate::scientific_selection::Selections,
}
impl InstancePrelude {
    /// Conservative known result extent.
    pub fn retained_bytes(&self) -> usize {
        self.environment
            .iter()
            .fold(size_of::<Self>(), |bytes, (name, value)| {
                bytes
                    .saturating_add(name.capacity())
                    .saturating_add(value.retained_bytes())
                    .saturating_add(128)
            })
            .saturating_add(
                self.members
                    .keys()
                    .map(|name| name.capacity() + 128)
                    .sum::<usize>(),
            )
            .saturating_add(
                self.stages
                    .iter()
                    .map(|stage| stage.capacity() + 128)
                    .sum::<usize>(),
            )
            .saturating_add(crate::extent::selections(&self.selections))
    }
}
/// The compiler provides query-owned derivation; standalone specialization derives
/// exactly the same pure result without introducing another production mechanism.
pub trait Fragments {
    /// Derive one source-owned expression with explicit local typed and namespace premises.
    fn field_fragment(
        &self,
        input: Arc<field_fragments::FieldFragmentInput>,
    ) -> Result<Arc<field_fragments::FieldFragment>>;
    /// Derive a local typed function body under its exact finite and physical premises.
    fn function_fragment(
        &self,
        input: Arc<function_fragments::FunctionFragmentInput>,
    ) -> Result<Arc<function_fragments::FunctionFragment>>;
    /// Derive from entity-local premises and exact bindings.
    fn instance_prelude(
        &self,
        premises: Arc<PreludePremises>,
        bindings: PreludeBindings,
    ) -> Result<Arc<InstancePrelude>>;
}
/// Pure local construction, shared by tracked and direct callers.
/// # Errors
/// Invalid constructor arguments, local guards, collisions or stage selection.
pub fn derive(premises: &PreludePremises, bindings: &PreludeBindings) -> Result<InstancePrelude> {
    let package = &premises.package;
    let context = package.context();
    let selection = crate::scientific_selection::Collector::default();
    let mut environment = bindings.inherited.clone();
    for name in premises.members.keys() {
        environment.remove(name);
    }
    environment.extend(bindings.facts.clone());
    for (name, value) in &bindings.scope {
        environment.insert(format!("scope.{name}"), value.clone());
    }
    let evaluator = Evaluator {
        lexical_bases: None,
        package,
        physical: &context,
        at: bindings.definition,
        env: &environment,
        limit: bindings.members,
        stack: Vec::new(),
        reader: if bindings.fixture {
            crate::provenance::Reader::Fixture
        } else {
            crate::provenance::Reader::Production(bindings.root)
        },
        selections: Some(&selection),
    };
    let (_, environment) = evaluator.definition_environment(
        bindings.definition,
        &bindings.arguments,
        environment.clone(),
    )?;
    let stages = premises
        .members
        .values()
        .filter(|member| member.kind == Kind::Stage)
        .map(|member| member.name.clone())
        .collect::<BTreeSet<_>>();
    if stages
        .iter()
        .filter(|name| {
            environment.get(&crate::analysis::Fact::Stage((*name).clone()).path())
                == Some(&Value::Boolean(true))
        })
        .count()
        > 1
    {
        return Err(invalid(
            bindings.definition,
            "multiple initialization stages selected in one definition",
        ));
    }
    let mut members = premises
        .members
        .iter()
        .filter(|(_, member)| !matches!(member.kind, Kind::When | Kind::Stage))
        .map(|(name, member)| (name.clone(), member.id))
        .collect::<BTreeMap<_, _>>();
    fn active(
        premises: &PreludePremises,
        member: &PreludeMember,
        environment: &Environment,
        context: &TypeContext<'_>,
        bindings: &PreludeBindings,
        selection: &crate::scientific_selection::Collector,
        output: &mut Vec<PreludeMember>,
    ) -> Result<()> {
        if !matches!(member.kind, Kind::When | Kind::Stage) {
            return Ok(());
        }
        let is_active = if member.kind == Kind::When {
            Evaluator {
                lexical_bases: None,
                package: &premises.package,
                physical: context,
                at: member.id,
                env: environment,
                limit: bindings.members,
                stack: Vec::new(),
                reader: if bindings.fixture {
                    crate::provenance::Reader::Fixture
                } else {
                    crate::provenance::Reader::Production(bindings.root)
                },
                selections: Some(selection),
            }
            .predicate(premises.package.predicate_at(
                member.id,
                "guard.predicate",
                0,
            )?)?
        } else {
            environment.get(&crate::analysis::Fact::Stage(member.name.clone()).path())
                == Some(&Value::Boolean(true))
        };
        if is_active {
            for child in premises.conditional.get(&member.id).into_iter().flatten() {
                if child.kind == Kind::When {
                    active(
                        premises,
                        child,
                        environment,
                        context,
                        bindings,
                        selection,
                        output,
                    )?;
                } else {
                    output.push(child.clone());
                }
            }
        }
        Ok(())
    }
    let mut conditional = Vec::new();
    for member in premises.members.values() {
        active(
            premises,
            member,
            &environment,
            &context,
            bindings,
            &selection,
            &mut conditional,
        )?;
    }
    for member in conditional {
        if members.insert(member.name, member.id).is_some() && !member.is_override {
            return Err(invalid(member.id, "active guarded member collision"));
        }
    }
    Ok(InstancePrelude {
        environment,
        members,
        stages,
        selections: selection.into_inner(),
    })
}
