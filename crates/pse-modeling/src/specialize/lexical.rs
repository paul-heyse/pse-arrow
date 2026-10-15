// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Lexical bases live only for one immutable package specialization.
use crate::{CheckedPackage, DeclarationId, Result, Type, TypeContext};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Default)]
pub(crate) struct LexicalBases {
    evaluation: RefCell<BTreeMap<DeclarationId, Arc<BTreeMap<String, Type>>>>,
    source: RefCell<BTreeMap<DeclarationId, Arc<BTreeMap<String, Type>>>>,
}
impl LexicalBases {
    fn owner(package: &CheckedPackage, at: DeclarationId) -> DeclarationId {
        let row = &package.declarations[&at];
        if crate::selected_source::owns_namespace(row) || package.functions.contains_key(&at) {
            at
        } else {
            // Expression leaves introduce no namespace; their sibling occurrences
            // share the enclosing lexical base. Function formals remain distinct.
            row.parent_id.unwrap_or(at)
        }
    }
    fn retain(
        inventory: &RefCell<BTreeMap<DeclarationId, Arc<BTreeMap<String, Type>>>>,
        at: DeclarationId,
        build: impl FnOnce() -> Result<BTreeMap<String, Type>>,
    ) -> Result<Arc<BTreeMap<String, Type>>> {
        if let Some(base) = inventory.borrow().get(&at) {
            return Ok(Arc::clone(base));
        }
        let base = Arc::new(build()?);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("modeling.lexical_base_builds", "bases", 1);
            context.count(
                "modeling.lexical_base_bindings",
                "bindings",
                base.len() as u64,
            );
        }
        inventory.borrow_mut().insert(at, Arc::clone(&base));
        Ok(base)
    }
    pub(super) fn evaluation(
        &self,
        package: &CheckedPackage,
        at: DeclarationId,
    ) -> Arc<BTreeMap<String, Type>> {
        // This inventory is exclusively used by immutable specialization readers.
        let owner = Self::owner(package, at);
        if let Some(base) = self.evaluation.borrow().get(&owner) {
            return Arc::clone(base);
        }
        let mut base = package.named_types(at);
        if let Some(function) = package.functions.get(&at) {
            base.extend(function.arguments.iter().cloned());
        }
        let base = Arc::new(base);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("modeling.lexical_base_builds", "bases", 1);
            context.count(
                "modeling.lexical_base_bindings",
                "bindings",
                base.len() as u64,
            );
        }
        self.evaluation
            .borrow_mut()
            .insert(owner, Arc::clone(&base));
        base
    }
    pub(super) fn source(
        &self,
        package: &CheckedPackage,
        context: &TypeContext<'_>,
        at: DeclarationId,
    ) -> Result<Arc<BTreeMap<String, Type>>> {
        Self::retain(&self.source, Self::owner(package, at), || {
            let mut types = package.named_types(at);
            let mut owners = Vec::new();
            let mut owner = Some(at);
            while let Some(id) = owner {
                owners.push(id);
                owner = package.declarations[&id].parent_id;
            }
            let mut variables = BTreeSet::new();
            for id in owners.into_iter().rev() {
                if let Some(scope) = &package.declarations[&id].value.scope {
                    variables.extend(scope.type_parameters.iter().cloned());
                    for argument in &scope.parameters {
                        types.insert(
                            argument.name.clone(),
                            context.resolve(&argument.r#type, &variables, &types, at)?,
                        );
                    }
                }
            }
            if let Some(function) = package.functions.get(&at) {
                types.extend(function.arguments.iter().cloned());
            }
            Ok(types)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialize::value::{Environment, Evaluator, Value};
    use crate::{Bindings, PhysicalScope};

    #[test]
    fn immutable_evaluators_share_base_but_recompute_current_value_types() {
        let (registry, names) = crate::kernel_types::physical();
        let preconditions = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        let context = TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &registry,
            preconditions: &preconditions,
            scope: &PhysicalScope::default(),
        };
        let package = crate::check(
            &crate::kernel_types::source(
                "package p {def Root {param x:Scalar=1;param y:Scalar=2;}}",
            ),
            &context,
        )
        .unwrap();
        let at = package.entry("p.Root").unwrap();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("lexical evaluation");
        let _entered = operation.context().enter();
        let bases = LexicalBases::default();
        let expression = pse_authoring::dsl::parse_expr("x*2").unwrap();
        for (member, name, value) in [("x", "Length", 2.), ("y", "Time", 3.)] {
            let at = package.entry(&format!("p.Root.{member}")).unwrap();
            let env = Environment::from([(
                "x".into(),
                Value::Number {
                    bits: f64::to_bits(value),
                    quantity: names[name],
                },
            )]);
            let actual = Evaluator {
                lexical_bases: Some(&bases),
                selections: None,
                package: &package,
                physical: &context,
                at,
                env: &env,
                limit: 32,
                stack: Vec::new(),
                reader: crate::provenance::Reader::Production(at),
            }
            .expr(&expression, None, 0)
            .unwrap();
            assert_eq!(
                actual,
                Value::Number {
                    bits: f64::to_bits(value * 2.),
                    quantity: names[name]
                }
            );
        }
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let count = |name: &str| {
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .unwrap()
                .cumulative
                .clone()
        };
        assert_eq!(
            count("modeling.lexical_base_builds"),
            pse_diagnostics::observation::Value::Count(1)
        );
        assert_eq!(
            count("modeling.type_overlays"),
            pse_diagnostics::observation::Value::Count(2)
        );
        // A later specialization has its own inventory, and no cache is attached to
        // the checked package or survives in the returned model.
        crate::specialize(
            &package,
            at,
            crate::InstanceId::from_bytes([1; 16]),
            &Bindings::default(),
            crate::Limits::default(),
        )
        .unwrap();
    }
}
