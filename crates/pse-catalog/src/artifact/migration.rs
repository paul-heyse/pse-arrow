// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked artifact transformations retain exact sources and use ordinary publication.

use super::{ArtifactPlan, PublicationSelection, RelationOutput, invalid};
use crate::delta::publication::Publication;
use datafusion::{
    common::{ResolvedTableReference, TableReference},
    logical_expr::{LogicalPlan, LogicalPlanBuilder},
};
use pse_columnar::{AllocationLease, CancellationToken, MemoryConsumer};
use pse_engine::{
    EngineError,
    session::{EngineSession, PreparedComputation},
};
use pse_ids::{Frame, FramedHasher};
use pse_model::HeapUsage;
use pse_relations::generated::{
    runtime::artifact_migration_lineage as lineage, structures::MemberDescriptor,
};
use pse_schema::{
    compatibility::{MigrationAdmission, VerifiedRecordedContract},
    fingerprint::SemanticContract,
    model::{MigrationSpec, MigrationStep},
};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit selected member and registered or independently declared transformation.
#[derive(Clone, Debug)]
pub struct MemberMigration {
    /// Exact source role in the opened publication.
    pub source: ResolvedTableReference,
    /// Fully qualified output role in the new artifact.
    pub output: ResolvedTableReference,
    /// Ordered portable operations and exact endpoint versions.
    pub declaration: MigrationSpec,
}

/// One complete checked migration over an immutable selected publication generation.
#[derive(Clone, Debug)]
pub struct ArtifactMigration {
    artifact: ArtifactPlan,
    retained: Vec<MemberDescriptor>,
    lineage: Vec<lineage::Row>,
}
impl ArtifactMigration {
    /// Compile all requested member migrations without reading or writing rows.
    /// Retained support is explicit and must be an exact source selection admitted by
    /// the target consumer. Changed references require an explicit key-map policy.
    /// # Errors
    /// Wrong source/target identities, undeclared maps, inconsistent referenced key
    /// mappings, incomplete target support or duplicate output roles.
    pub fn new(
        source: &Publication,
        migrations: Vec<MemberMigration>,
        retained: Vec<MemberDescriptor>,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        if migrations.is_empty() {
            return Err(invalid(
                "artifact migration requires explicit transformations",
            ));
        }
        let mut session = source.session().clone();
        let registry = session.registry().clone();
        let mut outputs = BTreeMap::new();
        let mut rows = Vec::new();
        let mut sources = BTreeSet::new();
        let mut target_ids = BTreeSet::new();
        let mut endpoints = BTreeMap::new();
        for migration in &migrations {
            cancel.checkpoint()?;
            if !sources.insert(migration.source.clone()) {
                return Err(invalid("migration source is selected twice"));
            }
            let member = source.member(&migration.source)?;
            if member.relation_version != i64::from(migration.declaration.from_version) {
                return Err(invalid("source selector differs from migration endpoint"));
            }
            let target = registry
                .relations()
                .iter()
                .find(|target| {
                    target.qualified_name() == migration.declaration.relation
                        && target.key.version == migration.declaration.to_version
                })
                .ok_or_else(|| invalid("migration target declaration is absent"))?;
            if !target_ids.insert(target.id) {
                return Err(invalid("migration target declaration is ambiguous"));
            }
            let target_contract = VerifiedRecordedContract::from_contract(
                SemanticContract::new(&registry, &[target.id].into()).map_err(schema_error)?,
            )
            .map_err(pse_columnar::external)
            .map_err(pse_engine::session::engine)?;
            let admission = MigrationAdmission::new(
                source.recorded_member(&migration.source)?.clone(),
                target_contract,
            );
            check_changed_references(
                &migration.declaration,
                admission.source(),
                member.relation_id,
                admission.target(),
                target.id,
            )?;
            let plan = session.plan_schema_transform(
                &migration.declaration,
                selected_plan(&session, &migration.source)?,
                &admission,
                target,
                cancel,
            )?;
            if outputs
                .insert(
                    migration.output.clone(),
                    RelationOutput {
                        relation_id: target.id,
                        plan,
                    },
                )
                .is_some()
            {
                return Err(invalid("migration output role is duplicated"));
            }
            let mut digest = FramedHasher::new(Frame::ArtifactMigrationV1);
            digest
                .hash(&admission.source_identity())
                .hash(&admission.target_identity())
                .str(&migration.declaration.qualified_name())
                .str(&migration.declaration.plan_spec());
            rows.push(lineage::Row {
                source_publication_id: source.record().publication_id,
                source_member: member.clone(),
                target_relation_id: target.id,
                target_relation_version: i64::from(target.key.version),
                declaration: migration.declaration.qualified_name(),
                transformation_digest: digest.finish_hash(),
                output_catalog: migration.output.catalog.to_string(),
                output_schema: migration.output.schema.to_string(),
                output_table: migration.output.table.to_string(),
            });
            endpoints.insert(member.relation_id, (target.id, migration));
        }
        for member in &retained {
            let reference = ResolvedTableReference {
                catalog: member.catalog_name.clone().into(),
                schema: member.schema_name.clone().into(),
                table: member.table_name.clone().into(),
            };
            if source.member(&reference)? != *member
                || sources.contains(&reference)
                || outputs.contains_key(&reference)
                || !target_ids.insert(member.relation_id)
            {
                return Err(invalid(
                    "retained migration member is not one unambiguous unchanged source",
                ));
            }
            let expected = SemanticContract::new(&registry, &[member.relation_id].into())
                .map_err(schema_error)?;
            source
                .recorded_member(&reference)?
                .project(&expected)
                .map_err(pse_columnar::external)
                .map_err(pse_engine::session::engine)?;
            check_retained_mapping_closure(
                source.recorded_member(&reference)?,
                member.relation_id,
                &endpoints,
            )?;
        }
        for migration in &migrations {
            check_mapping_closure(
                source.recorded_member(&migration.source)?,
                source.member(&migration.source)?.relation_id,
                &migration.declaration,
                &endpoints,
            )?;
        }
        let roots = outputs
            .values()
            .map(|output| output.relation_id)
            .chain(retained.iter().map(|member| member.relation_id))
            .collect();
        let closure =
            pse_schema::product::support_closure(&registry, &roots).map_err(schema_error)?;
        if !closure.is_subset(&target_ids) {
            return Err(invalid(
                "migration omits target reference or invariant support members",
            ));
        }
        for (name, output) in &outputs {
            for required in
                pse_schema::product::support_closure(&registry, &[output.relation_id].into())
                    .map_err(schema_error)?
            {
                if !outputs.iter().any(|(candidate, output)| {
                    candidate.catalog == name.catalog && output.relation_id == required
                }) && !retained.iter().any(|member| {
                    member.catalog_name == name.catalog.as_ref() && member.relation_id == required
                }) {
                    return Err(invalid(
                        "migration reference closure must reside in the output's exact publication catalog",
                    ));
                }
            }
        }
        let allocation = MemoryConsumer::new("catalog:migration-lineage").register(session.pool());
        allocation
            .try_grow(
                rows.iter()
                    .fold(0usize, |bytes, row| bytes.saturating_add(row.owned_bytes()))
                    .saturating_mul(16)
                    .saturating_add(4096),
            )
            .map_err(pse_engine::session::engine)?;
        let mut builder = lineage::Builder::with_registry(
            &registry,
            rows.len(),
            session.validation_context()?.as_ref(),
        )?;
        for row in &rows {
            builder.push(row.clone())?;
        }
        session = session.with_checked_workspace(
            BTreeMap::from([(lineage::RELATION_KEY, builder.finish()?)]),
            cancel,
        )?;
        session.retain_owner(AllocationLease::new(allocation));
        let lineage_source = ResolvedTableReference {
            catalog: "workspace".into(),
            schema: "runtime".into(),
            table: lineage::NAME.into(),
        };
        let lineage_output = ResolvedTableReference {
            catalog: "artifact".into(),
            schema: "runtime".into(),
            table: lineage::NAME.into(),
        };
        if outputs
            .insert(
                lineage_output,
                RelationOutput {
                    relation_id: lineage::RELATION_ID,
                    plan: session.relation_plan(&lineage_source)?.plan().clone(),
                },
            )
            .is_some()
        {
            return Err(invalid("migration lineage output collides"));
        }
        let mut artifact = ArtifactPlan::new(session, outputs, cancel)?;
        artifact.publication = PublicationSelection::Migration(std::sync::Arc::new(rows.clone()));
        Ok(Self {
            artifact,
            retained,
            lineage: rows,
        })
    }
    /// Exact lineage declarations that will be a member of the new publication.
    pub fn lineage(&self) -> &[lineage::Row] {
        &self.lineage
    }
    /// Prepare transient checked output. All target values, keys and references are
    /// validated over the same retained generation before any output is exposed.
    /// This performs no storage or catalog mutation.
    /// # Errors
    /// Any target obligation, native execution, cancellation or resource refusal.
    pub async fn prepare_read(
        &self,
        output: &ResolvedTableReference,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        let mut inputs = self
            .artifact
            .outputs
            .values()
            .map(|output| (output.relation_id, output.plan.clone()))
            .collect::<BTreeMap<_, _>>();
        for member in &self.retained {
            let reference = ResolvedTableReference {
                catalog: member.catalog_name.clone().into(),
                schema: member.schema_name.clone().into(),
                table: member.table_name.clone().into(),
            };
            inputs.insert(
                member.relation_id,
                selected_plan(self.artifact.session(), &reference)?,
            );
        }
        let native =
            pse_engine::validation::NativeValidation(self.artifact.session().bound_state()?);
        for check in pse_relations::validate::obligations::ObligationTemplates::new(
            self.artifact.session().registry(),
        )
        .bind(&inputs, &native, None)
        .map_err(pse_engine::session::engine)?
        {
            let completed = self
                .artifact
                .session()
                .prepare_rule_plan(check, cancel)?
                .execute(cancel)
                .await?;
            if completed
                .batches()
                .iter()
                .any(|batch| batch.num_rows() != 0)
            {
                return Err(invalid(
                    "transient migration output violates target obligations",
                ));
            }
        }
        self.artifact.prepare(output, cancel)
    }
    /// Transfer the checked plans and explicit unchanged support to the ordinary
    /// atomic publication path. Neither method starts member writes.
    pub fn into_publication(self) -> (ArtifactPlan, Vec<MemberDescriptor>) {
        (self.artifact, self.retained)
    }
}

fn check_retained_mapping_closure(
    source: &VerifiedRecordedContract,
    source_id: pse_ids::SemanticId,
    endpoints: &BTreeMap<pse_ids::SemanticId, (pse_ids::SemanticId, &MemberMigration)>,
) -> Result<(), EngineError> {
    for (_, target, columns) in recorded_references(source, source_id)? {
        if let Some((_, migration)) = endpoints.get(&target) {
            let mapping = mapping_for(&migration.declaration, &columns).ok_or_else(|| {
                invalid("retained dependent requires an explicit owner identity-key policy")
            })?;
            if !matches!(mapping.policy, pse_schema::model::MappingPolicy::Identity) {
                return Err(invalid(
                    "a dependent of recoded keys must migrate with the same mapping",
                ));
            }
        }
    }
    Ok(())
}

fn selected_plan(
    session: &EngineSession,
    reference: &ResolvedTableReference,
) -> Result<LogicalPlan, EngineError> {
    let table = TableReference::full(
        reference.catalog.clone(),
        reference.schema.clone(),
        reference.table.clone(),
    );
    let binding = session
        .bindings()
        .iter()
        .find_map(|(_, binding)| (binding.reference == table).then_some(binding))
        .ok_or_else(|| invalid("migration source provider is absent"))?;
    LogicalPlanBuilder::scan(
        table,
        datafusion::datasource::provider_as_source(binding.provider.clone()),
        None,
    )
    .and_then(LogicalPlanBuilder::build)
    .map_err(pse_engine::session::engine)
}
fn mapping_for<'a>(
    spec: &'a MigrationSpec,
    columns: &[String],
) -> Option<&'a pse_schema::model::MigrationValues> {
    let renamed = |columns: Vec<String>| {
        spec.steps.iter().fold(columns, |mut columns, step| {
            if let MigrationStep::RenameColumn { from, to } = step {
                for column in &mut columns {
                    if column == from {
                        *column = (*to).to_owned();
                    }
                }
            }
            columns
        })
    };
    let selected_columns = renamed(columns.to_vec());
    spec.steps.iter().find_map(|step| match step {
        MigrationStep::MapReferenceKey {
            columns: selected,
            mapping,
        } if renamed(selected.iter().map(|column| (*column).to_owned()).collect())
            == selected_columns =>
        {
            Some(mapping)
        }
        _ => None,
    })
}
fn check_changed_references(
    spec: &MigrationSpec,
    source: &VerifiedRecordedContract,
    source_id: pse_ids::SemanticId,
    target: &VerifiedRecordedContract,
    target_id: pse_ids::SemanticId,
) -> Result<(), EngineError> {
    let source = &source.contract().relations[&source_id];
    let target = &target.contract().relations[&target_id];
    let references = |description: &serde_json::Value| -> Vec<Vec<String>> {
        let mut columns = description["foreign_keys"]
            .as_object()
            .into_iter()
            .flat_map(|references| references.values())
            .filter_map(|reference| serde_json::from_value(reference["columns"].clone()).ok())
            .collect::<Vec<_>>();
        if let Some(fields) = description["fields"].as_array() {
            for field in fields {
                if let Ok(field) =
                    serde_json::from_value::<datafusion::arrow::datatypes::Field>(field.clone())
                    && field.metadata().contains_key(pse_schema::arrow::KEY_FK)
                {
                    columns.push(vec![field.name().clone()]);
                }
            }
        }
        columns
    };
    let fields =
        |description: &serde_json::Value| -> BTreeMap<String, datafusion::arrow::datatypes::Field> {
            description["fields"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|value| {
                    serde_json::from_value::<datafusion::arrow::datatypes::Field>(value.clone())
                        .ok()
                })
                .map(|field| (field.name().clone(), field))
                .collect()
        };
    let source_fields = fields(source);
    let target_fields = fields(target);
    for (name, field) in &target_fields {
        let keys = [
            pse_schema::arrow::KEY_FK,
            pse_schema::model::reference::KEY_REFERENCE,
        ];
        if keys.iter().any(|key| {
            source_fields
                .get(name)
                .and_then(|source| source.metadata().get(*key))
                != field.metadata().get(*key)
        }) && mapping_for(spec, std::slice::from_ref(name)).is_none()
        {
            return Err(invalid(
                "changed field reference requires an explicit key mapping or identity policy",
            ));
        }
    }
    if source["foreign_keys"] != target["foreign_keys"] {
        for columns in references(source).into_iter().chain(references(target)) {
            if mapping_for(spec, &columns).is_none() {
                return Err(invalid(
                    "changed reference requires an explicit composite key mapping or identity policy",
                ));
            }
        }
    }
    Ok(())
}
fn check_mapping_closure(
    source: &VerifiedRecordedContract,
    source_id: pse_ids::SemanticId,
    spec: &MigrationSpec,
    endpoints: &BTreeMap<pse_ids::SemanticId, (pse_ids::SemanticId, &MemberMigration)>,
) -> Result<(), EngineError> {
    let references = recorded_references(source, source_id)?;
    for (columns, target_id, target_columns) in references {
        let Some((_, endpoint)) = endpoints.get(&target_id) else {
            continue;
        };
        let dependent = mapping_for(spec, &columns)
            .ok_or_else(|| invalid("migrated reference target requires the dependent key map"))?;
        let owner = mapping_for(&endpoint.declaration, &target_columns)
            .ok_or_else(|| invalid("migrated reference target requires its owner key map"))?;
        let values = |mapping: &pse_schema::model::MigrationValues| match &mapping.policy {
            pse_schema::model::MappingPolicy::Identity => None,
            pse_schema::model::MappingPolicy::Finite(rows) => Some(
                rows.iter()
                    .map(|row| {
                        (
                            row.source
                                .iter()
                                .map(|value| value.as_json().to_owned())
                                .collect::<Vec<_>>(),
                            row.target
                                .iter()
                                .map(|value| value.as_json().to_owned())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<BTreeMap<_, _>>(),
            ),
        };
        if values(dependent) != values(owner) {
            return Err(invalid("reference and owner key maps differ"));
        }
    }
    Ok(())
}

type RecordedReference = (Vec<String>, pse_ids::SemanticId, Vec<String>);

fn recorded_references(
    source: &VerifiedRecordedContract,
    source_id: pse_ids::SemanticId,
) -> Result<Vec<RecordedReference>, EngineError> {
    let description = &source.contract().relations[&source_id];
    let mut result = Vec::new();
    let decode =
        |value: &serde_json::Value| serde_json::from_value(value.clone()).map_err(json_error);
    if let Some(references) = description["foreign_keys"].as_object() {
        for reference in references.values() {
            result.push((
                decode(&reference["columns"])?,
                serde_json::from_value(reference["target"].clone()).map_err(json_error)?,
                decode(&reference["target_columns"])?,
            ));
        }
    }
    let target_id = |name: &str| {
        source
            .contract()
            .relations
            .iter()
            .find_map(|(id, description)| {
                description["relation"]
                    .as_str()
                    .and_then(|key| key.rsplit_once('@'))
                    .filter(|(relation, _)| *relation == name)
                    .map(|_| *id)
            })
            .ok_or_else(|| invalid("recorded reference support is absent"))
    };
    let mut fields = description["fields"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|value| {
            serde_json::from_value::<datafusion::arrow::datatypes::Field>(value.clone())
                .map_err(json_error)
        })
        .collect::<Result<Vec<_>, _>>()?;
    while let Some(field) = fields.pop() {
        if let Some(fk) = field.metadata().get(pse_schema::arrow::KEY_FK) {
            let (relation, column) = fk
                .rsplit_once('.')
                .ok_or_else(|| invalid("recorded scalar reference is malformed"))?;
            result.push((
                vec![field.name().clone()],
                target_id(relation)?,
                vec![column.into()],
            ));
        }
        if let Some(text) = field
            .metadata()
            .get(pse_schema::model::reference::KEY_REFERENCE)
        {
            let reference: pse_schema::model::ReferenceContract =
                serde_json::from_str(text).map_err(json_error)?;
            if reference
                .columns
                .iter()
                .any(|column| !column.source.is_empty())
            {
                return Err(invalid(
                    "nested composite reference migration requires an explicit supported field-path transformation",
                ));
            }
            result.push((
                vec![field.name().clone()],
                target_id(&reference.relation)?,
                reference
                    .columns
                    .iter()
                    .map(|column| column.target.clone())
                    .collect(),
            ));
        }
        fields.extend(
            pse_columnar::native_field::children(field.data_type())
                .into_iter()
                .cloned(),
        );
    }
    Ok(result)
}

fn schema_error(error: pse_schema::SchemaError) -> EngineError {
    EngineError::Semantic(std::sync::Arc::new(error))
}

fn json_error(error: serde_json::Error) -> EngineError {
    invalid(&format!("malformed recorded migration reference: {error}"))
}

#[cfg(test)]
mod mapping_unit {
    use super::*;
    use datafusion::{
        arrow::datatypes::{DataType, Field},
        common::ScalarValue,
    };
    use pse_schema::{
        RegistryBuilder,
        model::{
            Authority, FieldContract, MappingPolicy, MigrationNulls, MigrationValuePair,
            MigrationValues, Namespace, RelationDecl, SnapshotClass,
        },
    };
    use std::sync::Arc;
    fn recorded(
        version: u32,
    ) -> (
        VerifiedRecordedContract,
        pse_ids::SemanticId,
        pse_ids::SemanticId,
    ) {
        let col = |name| FieldContract::key(name, FieldContract::native(DataType::Int64), "Key");
        let decl = |name| {
            RelationDecl::new(
                Namespace::Authored,
                name,
                version,
                Authority::Authored,
                SnapshotClass::Model,
                "closure",
            )
        };
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(
            decl("owners")
                .pk(&["a", "b"])
                .columns(vec![col("a"), col("b")]),
        );
        builder.declare_relation(
            decl("dependents")
                .pk(&["id"])
                .columns(vec![col("id"), col("ca"), col("cb")])
                .foreign_key("owner", &["ca", "cb"], "authored.owners", &["a", "b"]),
        );
        let registry = builder.build().unwrap();
        let owner = registry.relation("authored.owners").unwrap().id;
        let dependent = registry.relation("authored.dependents").unwrap().id;
        (
            VerifiedRecordedContract::from_contract(
                SemanticContract::new(&registry, &[dependent].into()).unwrap(),
            )
            .unwrap(),
            owner,
            dependent,
        )
    }
    fn migration(
        name: &'static str,
        columns: Vec<&'static str>,
        policy: MappingPolicy,
    ) -> MemberMigration {
        MemberMigration {
            source: ResolvedTableReference {
                catalog: "artifact".into(),
                schema: "authored".into(),
                table: name.into(),
            },
            output: ResolvedTableReference {
                catalog: "artifact".into(),
                schema: "authored".into(),
                table: name.into(),
            },
            declaration: MigrationSpec {
                relation: if name == "owners" {
                    "authored.owners"
                } else {
                    "authored.dependents"
                },
                from_version: 1,
                to_version: 2,
                steps: vec![MigrationStep::MapReferenceKey {
                    columns,
                    mapping: MigrationValues {
                        policy,
                        nulls: MigrationNulls::Reject,
                    },
                }],
                doc: "explicit tuple mapping",
            },
        }
    }
    fn finite(second: i64) -> MappingPolicy {
        let value = |name, value| {
            pse_schema::NativeLiteral::from_scalar(
                Arc::new(Field::new(name, DataType::Int64, false)),
                &ScalarValue::Int64(Some(value)),
            )
            .unwrap()
        };
        MappingPolicy::Finite(vec![MigrationValuePair {
            source: vec![value("a", 1), value("b", 2)],
            target: vec![value("a", 11), value("b", second)],
        }])
    }
    #[test]
    fn retained_dependents_refuse_recoded_owners_even_if_target_keys_could_exist() {
        let (source, owner, dependent) = recorded(1);
        let mapped = migration("owners", vec!["a", "b"], finite(22));
        let endpoints = BTreeMap::from([(owner, (pse_ids::SemanticId::NIL, &mapped))]);
        assert!(check_retained_mapping_closure(&source, dependent, &endpoints).is_err());
        let identity = migration("owners", vec!["a", "b"], MappingPolicy::Identity);
        let endpoints = BTreeMap::from([(owner, (pse_ids::SemanticId::NIL, &identity))]);
        assert!(check_retained_mapping_closure(&source, dependent, &endpoints).is_ok());
    }
    #[test]
    fn composite_reference_closure_requires_same_mapping_and_changed_refs_cannot_infer_identity() {
        let (source, owner, dependent) = recorded(1);
        let (target, _, target_dependent) = recorded(2);
        let owner_migration = migration("owners", vec!["a", "b"], finite(22));
        let endpoints = BTreeMap::from([(owner, (pse_ids::SemanticId::NIL, &owner_migration))]);
        let dependent_migration = migration("dependents", vec!["ca", "cb"], finite(22));
        assert!(
            check_mapping_closure(
                &source,
                dependent,
                &dependent_migration.declaration,
                &endpoints
            )
            .is_ok()
        );
        let mismatched = migration("dependents", vec!["ca", "cb"], finite(33));
        assert!(
            check_mapping_closure(&source, dependent, &mismatched.declaration, &endpoints).is_err()
        );
        let mut renamed = migration("dependents", vec!["renamed_ca", "cb"], finite(22));
        renamed.declaration.steps.insert(
            0,
            MigrationStep::RenameColumn {
                from: "ca",
                to: "renamed_ca",
            },
        );
        assert!(
            check_mapping_closure(&source, dependent, &renamed.declaration, &endpoints).is_ok()
        );
        let mut missing = dependent_migration.declaration.clone();
        missing.steps.clear();
        assert!(
            check_changed_references(&missing, &source, dependent, &target, target_dependent)
                .is_err()
        );
        assert!(
            check_changed_references(
                &dependent_migration.declaration,
                &source,
                dependent,
                &target,
                target_dependent
            )
            .is_ok()
        );
    }
    #[test]
    fn ordinary_artifact_cannot_claim_migration_publication_kind() {
        use pse_relations::generated::{enums::PublicationKind, runtime::publication_manifests};
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "ordinary",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "ordinary",
            )
            .pk(&[])
            .columns(vec![FieldContract::payload(
                "value",
                FieldContract::native(DataType::Int64),
                "value",
            )]),
        );
        let registry = Arc::new(builder.build().unwrap());
        let spec = registry.relation("authored.ordinary").unwrap();
        let relation_id = spec.id;
        let batch = datafusion::arrow::array::RecordBatch::new_empty(Arc::new(
            pse_schema::arrow::relation_schema(&registry, spec).unwrap(),
        ));
        let factory = pse_testkit::NativeFixture::new((32 << 20).try_into().unwrap())
            .unwrap()
            .into_factory();
        let cancel = CancellationToken::new();
        let session = factory
            .candidate(BTreeMap::from([(spec.key, batch)]), registry, &cancel)
            .unwrap();
        let source = session
            .table_reference(&session.registry().relation_by_id(relation_id).unwrap().key)
            .unwrap()
            .resolve("", "");
        let output = RelationOutput {
            relation_id,
            plan: session.relation_plan(&source).unwrap().plan().clone(),
        };
        let artifact =
            ArtifactPlan::new(session, BTreeMap::from([(source, output)]), &cancel).unwrap();
        let hash = pse_ids::ContentHash::from_bytes([1; 32]);
        let descriptor = pse_model::artifact::ArtifactDescriptor::create(pse_relations::generated::runtime::artifact_descriptors::Row {
            artifact_id: hash, descriptor_version: 3, profile: PublicationKind::Migration,
            profile_contract: hash, requested_relations: vec![relation_id], profile_required_relations: Some(vec![]),
            release_id: hash, release_members: vec![], semantic_identity: hash,
            implementation: pse_relations::generated::runtime::artifact_descriptors::RuntimeArtifactDescriptorsFieldImplementation { source: hash, build: hash, registry: hash, algorithms: hash },
            target_contract: hash, value_assumptions: vec![], reconstruction: pse_relations::generated::enums::ArtifactReconstruction::None,
        }).unwrap();
        assert!(
            artifact
                .clone()
                .with_product(descriptor, &cancel)
                .unwrap_err()
                .to_string()
                .contains("checked transformation and lineage admission")
        );
        let id = pse_ids::SemanticId::from_bytes([1; 16]);
        let header = publication_manifests::Row {
            publication_id: id.into(),
            workspace_id: id.into(),
            parent_publication_id: None,
            attempt_id: id.into(),
            kind: PublicationKind::Migration,
            inputs: vec![],
            members: vec![],
            windows: vec![],
            exported_at: None,
            export_lease_id: None,
            export_expires_at: None,
            maintenance_epoch: None,
            store_fingerprint: None,
        };
        let error = artifact
            .prepare_publication(header, BTreeMap::new(), vec![], &cancel)
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("checked transformation and lineage capability")
        );
    }
}
