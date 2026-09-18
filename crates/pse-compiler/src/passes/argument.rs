// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Enforced algorithm argument views. A projected argument never exposes its
//! original relation owner; undeclared values cannot be recovered through this API.
use super::{CompilerError, invalid};
use pse_catalog::session::RelationFacts;
use pse_schema::{
    Registry,
    model::{AlgorithmSpec, algorithm::InputConsumption},
};
use std::sync::Arc;

/// One owned argument with an enforced whole or projected field contract.
#[derive(Clone, Debug)]
pub struct BoundInput {
    relation: Option<Arc<RelationFacts>>,
    fields: datafusion::arrow::array::RecordBatch,
    relation_id: pse_ids::SemanticId,
}

impl BoundInput {
    /// Bind a conservative whole-input argument.
    /// # Errors
    /// The declaration is absent or differs from these fields.
    pub fn from_facts(
        relation: Arc<RelationFacts>,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        let spec = registry
            .relation_by_id(relation.checked().relation_id())
            .ok_or_else(|| invalid("input relation is undeclared"))?;
        relation.checked().check_declaration(registry, spec)?;
        Ok(Self {
            fields: relation.checked().batch().clone(),
            relation_id: spec.id,
            relation: Some(relation),
        })
    }
    pub(crate) fn declared(
        relation: Arc<RelationFacts>,
        algorithm: &AlgorithmSpec,
        port: &str,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        let argument = algorithm
            .inputs
            .iter()
            .find(|argument| argument.port == port)
            .ok_or_else(|| invalid("algorithm argument is undeclared"))?;
        let registered = registry
            .algorithms()
            .iter()
            .find(|spec| spec.id == algorithm.id)
            .ok_or_else(|| invalid("algorithm declaration is absent from the registry"))?;
        if registered.inputs != algorithm.inputs {
            return Err(invalid(
                "algorithm input contract differs from the registry",
            ));
        }
        let fields = pse_relations::generated::algorithm_arguments::project(
            algorithm.id,
            port,
            relation.checked(),
            registry,
        )?;
        let relation_id = relation.checked().relation_id();
        let relation = matches!(argument.consumption, InputConsumption::Whole).then_some(relation);
        Ok(Self {
            relation,
            fields,
            relation_id,
        })
    }
    /// Complete fields only for arguments declared as whole-input consumers.
    /// # Errors
    /// The contract limits this argument to a generated projection.
    pub fn relation(&self) -> Result<&Arc<RelationFacts>, CompilerError> {
        self.relation.as_ref().ok_or_else(|| {
            invalid("whole relation access exceeds the declared argument projection")
        })
    }
    /// Only the fields the declaration permits this algorithm to read.
    pub fn fields(&self) -> &datafusion::arrow::array::RecordBatch {
        &self.fields
    }
    /// Original registered relation identity, independent of the restricted schema.
    pub fn relation_id(&self) -> pse_ids::SemanticId {
        self.relation_id
    }
    pub(crate) fn check(
        &self,
        argument: &pse_schema::model::ArgumentSpec,
        registry: &Registry,
    ) -> Result<(), CompilerError> {
        let spec = registry
            .relation(&argument.relation)
            .ok_or_else(|| invalid("input relation missing"))?;
        if self.relation_id != spec.id {
            return Err(invalid("input declaration changed"));
        }
        match &argument.consumption {
            InputConsumption::Whole => self
                .relation()?
                .checked()
                .check_declaration(registry, spec)
                .map_err(Into::into),
            InputConsumption::Columns(columns) => {
                let actual = self
                    .fields
                    .schema()
                    .fields()
                    .iter()
                    .map(|field| field.name().clone())
                    .collect::<std::collections::BTreeSet<_>>();
                if &actual != columns || self.relation.is_some() {
                    return Err(invalid(
                        "argument access differs from its projection contract",
                    ));
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_relations::generated::reference::elements;
    use pse_schema::model::{AlgorithmDecl, ArgumentSpec, Determinism};
    #[test]
    fn registered_extension_projection_removes_full_access_and_preserves_rows() {
        let mut builder = pse_schema::RegistryBuilder::new();
        pse_schema::catalog::declare(&mut builder);
        builder.declare_algorithm(
            AlgorithmDecl::new("StructureOnly", "1", Determinism::Deterministic).inputs(vec![
                ArgumentSpec {
                    port: "values".into(),
                    relation: "reference.elements".into(),
                    required: true,
                    consumption: InputConsumption::Columns(
                        ["element_id".into(), "symbol".into()].into(),
                    ),
                },
            ]),
        );
        let registry = builder.build().unwrap();
        let mut rows = elements::Builder::with_registry(&registry, 1).unwrap();
        rows.push(elements::Row {
            element_id: pse_ids::SemanticId::NIL,
            symbol: "C".into(),
            name: "Carbon".into(),
            atomic_mass: 12.011,
        })
        .unwrap();
        let facts = Arc::new(RelationFacts::from_checked(rows.finish().unwrap()));
        let spec = registry.algorithm("StructureOnly@1").unwrap();
        let restricted = BoundInput::declared(facts, spec, "values", &registry).unwrap();
        assert!(restricted.relation().is_err());
        assert_eq!(restricted.fields().num_rows(), 1);
        assert!(restricted.fields().column_by_name("atomic_mass").is_none());
        assert!(restricted.fields().column_by_name("symbol").is_some());
        restricted.check(&spec.inputs[0], &registry).unwrap();
    }
}
