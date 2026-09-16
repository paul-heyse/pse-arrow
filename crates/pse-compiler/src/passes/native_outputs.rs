// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated algorithm columns couple to exact bound sources in one native plan.
use super::native_rows::{column, engine, join};
use crate::{CompilerError, InputBundle};
use datafusion::{
    common::ScalarValue,
    logical_expr::{Expr, JoinType, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{SnapshotSession, output::checked_literal, scalar};
use pse_ids::{CancellationToken, MemoryReserver, Reservation, ReservationLease, SemanticId};
use pse_relations::{
    columnar::{Collection, FieldCheckedBatch, RelationRow},
    generated::provenance,
};
use pse_rules::strata::{
    LocatedRuleInput, RuleInputLocation,
    native_input::{NativeInput, NativeWitness},
};
use pse_schema::{
    Registry,
    model::{PassSpec, RelationKey},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// An exact key calculated by a retained source projection, never a membership proof.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SourceKey {
    pub(crate) relation: RelationKey,
    pub(crate) port: String,
    pub(crate) key: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SourceRole {
    pub(crate) relation: RelationKey,
    pub(crate) port: String,
}

impl SourceKey {
    pub(crate) fn role(&self) -> SourceRole {
        SourceRole {
            relation: self.relation,
            port: self.port.clone(),
        }
    }
}

/// One algorithm's registered columns and exact source associations, including
/// private intermediate relations. Final pass publication checks its exact ports
/// in the catalog executor; a private algorithm result is not a pass port.
pub(crate) struct OutputRows<'a> {
    columns: Collection<'a>,
    retained: BTreeMap<RelationKey, FieldCheckedBatch>,
    cancel: &'a CancellationToken,
    occurrences: Collection<'a>,
    counts: BTreeMap<RelationKey, u64>,
    positive_sources: BTreeMap<RelationKey, BTreeSet<SourceRole>>,
    read_scopes: BTreeMap<RelationKey, BTreeSet<SourceRole>>,
    registry: &'a Registry,
    reserver: &'a dyn MemoryReserver,
    work: Box<dyn Reservation>,
}

pub(crate) struct GeneratedOutputs {
    pub(crate) columns: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(crate) occurrences: FieldCheckedBatch,
    pub(crate) positive_sources: BTreeMap<RelationKey, BTreeSet<SourceRole>>,
    pub(crate) read_scopes: BTreeMap<RelationKey, BTreeSet<SourceRole>>,
    pub(crate) _work: Arc<ReservationLease>,
}

impl<'a> OutputRows<'a> {
    pub(crate) fn new(
        registry: &'a Registry,
        reserver: &'a dyn MemoryReserver,
        cancel: &'a CancellationToken,
    ) -> Result<Self, CompilerError> {
        let mut occurrences = Collection::new(registry, reserver, cancel);
        occurrences.ensure::<provenance::algorithm_source_occurrences::Row>()?;
        Ok(Self {
            columns: Collection::new(registry, reserver, cancel),
            retained: BTreeMap::new(),
            cancel,
            occurrences,
            counts: BTreeMap::new(),
            positive_sources: BTreeMap::new(),
            read_scopes: BTreeMap::new(),
            registry,
            reserver,
            work: reserver.open("algorithm-output-bookkeeping"),
        })
    }
    pub(crate) fn ensure<T: RelationRow>(&mut self) -> Result<(), CompilerError> {
        self.columns.ensure::<T>()?;
        Ok(())
    }
    pub(crate) fn push<T: RelationRow>(
        &mut self,
        row: T,
        sources: &BTreeSet<SourceKey>,
    ) -> Result<(), CompilerError> {
        let spec = T::relation(self.registry)?;
        self.record_sources(spec, sources)?;
        self.columns.push(row)?;
        Ok(())
    }
    fn record_sources(
        &mut self,
        spec: &pse_schema::model::RelationSpec,
        sources: &BTreeSet<SourceKey>,
    ) -> Result<(), CompilerError> {
        if sources.is_empty() {
            return Err(invalid("algorithm output has no actual source occurrence"));
        }
        let extent = sources
            .len()
            .checked_mul(128)
            .and_then(|size| size.checked_add(512))
            .ok_or_else(|| invalid("algorithm output bookkeeping extent overflow"))?;
        self.work
            .try_grow(extent)
            .map_err(pse_ids::CanonError::from)?;
        let ordinal = self.counts.entry(spec.key).or_default();
        let next = ordinal
            .checked_add(1)
            .ok_or_else(|| invalid("algorithm output ordinal overflow"))?;
        for source in sources {
            self.positive_sources
                .entry(spec.key)
                .or_default()
                .insert(source.role());
            let relation = self
                .registry
                .relation(&source.relation.qualified_name())
                .filter(|spec| spec.key == source.relation)
                .ok_or_else(|| invalid("algorithm source declaration absent"))?;
            let mut temporary = self.reserver.open("algorithm-occurrence-dto");
            temporary
                .try_grow(
                    source
                        .key
                        .len()
                        .checked_add(size_of::<provenance::algorithm_source_occurrences::Row>())
                        .ok_or_else(|| invalid("algorithm occurrence extent overflow"))?,
                )
                .map_err(pse_ids::CanonError::from)?;
            self.occurrences
                .push(provenance::algorithm_source_occurrences::Row {
                    output_relation_id: spec.id,
                    constructed_row_ordinal: *ordinal,
                    source_port: source.port.clone(),
                    source_relation_id: relation.id,
                    source_key: source.key.clone(),
                })?;
        }
        *ordinal = next;
        Ok(())
    }
    pub(crate) fn read_scope(
        &mut self,
        output: RelationKey,
        source: SourceRole,
    ) -> Result<(), CompilerError> {
        self.work.try_grow(256).map_err(pse_ids::CanonError::from)?;
        self.read_scopes.entry(output).or_default().insert(source);
        Ok(())
    }
    /// A typed view of current algorithm columns. The output owner retains both
    /// the native fields and DTO reservation while dependent local algorithms run.
    pub(crate) fn rows<T: RelationRow>(&mut self) -> Result<Vec<T>, CompilerError> {
        self.columns.ensure::<T>()?;
        self.flush()?;
        let key = T::relation(self.registry)?.key;
        let batch = self
            .retained
            .get(&key)
            .ok_or_else(|| invalid("typed algorithm output absent"))?;
        self.work
            .try_grow(
                pse_ids::validation_extent(batch.batch())?
                    .checked_mul(2)
                    .ok_or_else(|| invalid("typed output view extent overflow"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        Ok(T::rows(batch)?)
    }

    /// Attach source occurrences while traversing this actual completed algorithm
    /// batch. The callback reads that same batch/ordinal; it does not rerun a producer.
    pub(crate) fn append_checked(
        &mut self,
        input: FieldCheckedBatch,
        mut sources: impl FnMut(&FieldCheckedBatch, usize) -> Result<BTreeSet<SourceKey>, CompilerError>,
    ) -> Result<(), CompilerError> {
        self.flush()?;
        let spec = self
            .registry
            .relation_by_id(input.relation_id())
            .ok_or_else(|| invalid("algorithm output declaration absent"))?;
        input.check_declaration(self.registry, spec)?;
        for row in 0..input.batch().num_rows() {
            self.cancel.checkpoint()?;
            self.record_sources(spec, &sources(&input, row)?)?;
        }
        self.retain(input)
    }

    fn retain(&mut self, input: FieldCheckedBatch) -> Result<(), CompilerError> {
        let spec = self
            .registry
            .relation_by_id(input.relation_id())
            .ok_or_else(|| invalid("algorithm output declaration absent"))?;
        let input = if let Some(prior) = self.retained.get(&spec.key) {
            if input.batch().num_rows() == 0 {
                return Ok(());
            }
            if prior.batch().num_rows() == 0 {
                input
            } else {
                FieldCheckedBatch::concat_reserved(
                    self.registry,
                    spec,
                    &[prior.clone(), input],
                    self.reserver,
                    self.cancel,
                )?
            }
        } else {
            input
        };
        self.retained.insert(spec.key, input);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), CompilerError> {
        let columns = std::mem::replace(
            &mut self.columns,
            Collection::new(self.registry, self.reserver, self.cancel),
        );
        for input in columns.finish()?.into_values() {
            self.retain(input)?;
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<GeneratedOutputs, CompilerError> {
        self.flush()?;
        Ok(GeneratedOutputs {
            _work: ReservationLease::new(self.work),
            positive_sources: self.positive_sources,
            read_scopes: self.read_scopes,
            columns: self.retained,
            occurrences: self
                .occurrences
                .finish()?
                .remove(&provenance::algorithm_source_occurrences::RELATION_KEY)
                .ok_or_else(|| invalid("algorithm occurrence batch absent"))?,
        })
    }
}

/// Each binding names the actual immutable source, including completed native inputs.
#[derive(Clone, Debug, Default)]
pub(crate) struct Sources(BTreeMap<SourceRole, (String, LocatedRuleInput)>);

impl Sources {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Resolve association keys to one explicit bound role. Actual membership is
    /// established later by the native output join to that retained source.
    pub(crate) fn locate(
        &self,
        origins: impl IntoIterator<Item = (RelationKey, String)>,
    ) -> Result<BTreeSet<SourceKey>, CompilerError> {
        origins
            .into_iter()
            .map(|(relation, key)| {
                let (port, _) = self
                    .get(&relation)
                    .ok_or_else(|| invalid("source occurrence needs one exact bound role"))?;
                Ok(SourceKey {
                    relation,
                    port: port.clone(),
                    key,
                })
            })
            .collect()
    }

    /// Retain every actual pass input role, even when two roles share a declaration.
    pub(crate) fn from_inputs(
        inputs: &InputBundle,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        let mut sources = Self::new();
        for (port, bound) in &inputs.ports {
            let Some(bound) = bound else {
                continue;
            };
            let spec = registry
                .relation_by_id(bound.relation_id())
                .ok_or_else(|| invalid("algorithm input declaration is absent"))?;
            let location = RuleInputLocation::Facts(Arc::new(
                pse_catalog::session::RelationFacts::from_checked(
                    bound.relation().checked().clone(),
                ),
            ));
            sources.insert(
                spec.key,
                (
                    (*port).to_owned(),
                    LocatedRuleInput {
                        relation: spec.key,
                        location,
                    },
                ),
            );
        }
        Ok(sources)
    }

    /// Replace an unambiguous relation role with its actual native output.
    /// Its native support mapping retains the input owners used by that computation.
    pub(crate) fn replace_native(
        &mut self,
        relation: RelationKey,
        input: Arc<NativeInput>,
    ) -> Result<(), CompilerError> {
        self.replace_location(relation, RuleInputLocation::Native(input))
    }

    pub(crate) fn replace_location(
        &mut self,
        relation: RelationKey,
        location: RuleInputLocation,
    ) -> Result<(), CompilerError> {
        if !matches!(
            location,
            RuleInputLocation::Native(_) | RuleInputLocation::Completed(_)
        ) {
            return Err(invalid(
                "a source successor must retain its actual native producer",
            ));
        }
        let roles = self
            .0
            .keys()
            .filter(|role| role.relation == relation)
            .collect::<Vec<_>>();
        let port = match roles.as_slice() {
            [] => relation.qualified_name(),
            [role] => role.port.clone(),
            _ => {
                return Err(invalid(
                    "native replacement requires one explicit source role",
                ));
            }
        };
        self.insert(relation, (port, LocatedRuleInput { relation, location }));
        Ok(())
    }

    pub(crate) fn locations(
        &self,
    ) -> Result<BTreeMap<RelationKey, RuleInputLocation>, CompilerError> {
        let mut result = BTreeMap::new();
        for (role, (_, input)) in &self.0 {
            if result
                .insert(role.relation, input.location.clone())
                .is_some()
            {
                return Err(invalid(
                    "relation-addressed source use requires one exact role",
                ));
            }
        }
        Ok(result)
    }

    pub(crate) fn insert(&mut self, relation: RelationKey, binding: (String, LocatedRuleInput)) {
        self.0.insert(
            SourceRole {
                relation,
                port: binding.0.clone(),
            },
            binding,
        );
    }

    /// Relation-only lookup is valid only for an unambiguous single role.
    pub(crate) fn get(&self, relation: &RelationKey) -> Option<&(String, LocatedRuleInput)> {
        let mut bindings = self.0.iter().filter(|(role, _)| role.relation == *relation);
        let (_, binding) = bindings.next()?;
        bindings.next().is_none().then_some(binding)
    }

    pub(crate) fn contains_key(&self, relation: &RelationKey) -> bool {
        self.get(relation).is_some()
    }

    fn role(&self, role: &SourceRole) -> Option<&(String, LocatedRuleInput)> {
        self.0.get(role)
    }
}

pub(crate) async fn materialize(
    generated: GeneratedOutputs,
    sources: &Sources,
    pass: &PassSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let mut output = BTreeMap::new();
    for (key, batch) in generated.columns {
        let positives = generated
            .positive_sources
            .get(&key)
            .cloned()
            .unwrap_or_default();
        let scopes = generated.read_scopes.get(&key).cloned().unwrap_or_default();
        let input = materialize_relation(
            key,
            batch,
            &generated.occurrences,
            sources,
            &positives,
            &scopes,
            pass,
            session,
            cancel,
        )
        .await?;
        output.insert(key, input);
    }
    Ok(output)
}

async fn materialize_relation(
    key: RelationKey,
    batch: FieldCheckedBatch,
    occurrences: &FieldCheckedBatch,
    sources: &Sources,
    positives: &BTreeSet<SourceRole>,
    scopes: &BTreeSet<SourceRole>,
    pass: &PassSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<Arc<NativeInput>, CompilerError> {
    let registry = session.registry();
    let spec = registry
        .relation(&key.qualified_name())
        .filter(|spec| spec.key == key)
        .ok_or_else(|| invalid("algorithm output declaration absent"))?;
    let occurrence_spec = provenance::algorithm_source_occurrences::spec(registry)?;
    let mut session = session.with_indexed_checked_role("algorithm_output", &batch, cancel)?;
    session = session.with_checked_role_inputs(
        BTreeMap::from([("algorithm_occurrences".to_owned(), occurrences.clone())]),
        cancel,
    )?;
    let output = LogicalPlanBuilder::from(session.scan_computation_role("algorithm_output")?)
        .alias("o")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let occurrences = LogicalPlanBuilder::from(session.scan_role("algorithm_occurrences")?)
        .filter(
            col("output_relation_id").eq(checked_literal(
                registry,
                occurrence_spec
                    .column("output_relation_id")
                    .ok_or_else(|| invalid("occurrence output field absent"))?,
                ScalarValue::FixedSizeBinary(16, Some(spec.id.as_bytes().to_vec())),
            )
            .map_err(engine)?),
        )
        .map_err(engine)?
        .alias("a")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let unsupported = join(
        output.clone(),
        occurrences.clone(),
        JoinType::LeftAnti,
        &[("o.constructed_row_ordinal", "a.constructed_row_ordinal")],
    )?;
    crate::passes::native_rows::reject(
        unsupported,
        &session,
        cancel,
        "algorithm output omitted actual source correspondence",
    )
    .await?;
    let detached = join(
        occurrences.clone(),
        output.clone(),
        JoinType::LeftAnti,
        &[("a.constructed_row_ordinal", "o.constructed_row_ordinal")],
    )?;
    crate::passes::native_rows::reject(
        detached,
        &session,
        cancel,
        "algorithm occurrence refers outside its retained output batch",
    )
    .await?;
    let mut joined = join(
        output,
        occurrences,
        JoinType::Inner,
        &[("o.constructed_row_ordinal", "a.constructed_row_ordinal")],
    )?;
    let mut witnesses = Vec::new();
    let mut support_fields = Vec::new();
    let mut known: Option<Expr> = None;
    for (ordinal, source_key) in positives.iter().enumerate() {
        let (port, input) = sources
            .role(source_key)
            .ok_or_else(|| invalid("algorithm positive source role is not bound"))?;
        let source = registry
            .relation(&source_key.relation.qualified_name())
            .filter(|spec| spec.key == source_key.relation)
            .ok_or_else(|| invalid("algorithm source declaration absent"))?;
        let source_id = checked_literal(
            registry,
            occurrence_spec
                .column("source_relation_id")
                .ok_or_else(|| invalid("occurrence source field absent"))?,
            ScalarValue::FixedSizeBinary(16, Some(source.id.as_bytes().to_vec())),
        )
        .map_err(engine)?;
        let when = column("a", "source_relation_id")
            .eq(source_id.clone())
            .and(column("a", "source_port").eq(lit(port.clone())));
        known = Some(known.map_or_else(|| when.clone(), |previous| previous.or(when.clone())));
        let alias = format!("algorithm_source_{ordinal}");
        let checked = match &input.location {
            RuleInputLocation::Facts(facts) => facts.checked().clone(),
            RuleInputLocation::Completed(input) => input.checked().clone(),
            RuleInputLocation::Native(input) => input.checked().clone(),
            RuleInputLocation::Workspace => {
                return Err(invalid("algorithm source has no retained immutable fields"));
            }
        };
        session =
            session.with_checked_role_inputs(BTreeMap::from([(alias.clone(), checked)]), cancel)?;
        let plan = LogicalPlanBuilder::from(session.scan_role(&alias)?)
            .alias(&alias)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let mut fields = source
            .primary_key
            .iter()
            .map(|name| column(&alias, name).alias(*name))
            .collect::<Vec<_>>();
        fields.push(
            scalar::key(
                source
                    .primary_key
                    .iter()
                    .map(|name| (*name, column(&alias, name)))
                    .collect(),
            )
            .alias("algorithm_source_key"),
        );
        fields.push(source_id.clone().alias("algorithm_source_relation"));
        fields.push(lit(port.clone()).alias("algorithm_source_port"));
        let plan = LogicalPlanBuilder::from(plan)
            .project(fields)
            .map_err(engine)?
            .alias(&alias)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        joined = join(
            joined,
            plan,
            JoinType::Left,
            &[
                (
                    "a.source_relation_id",
                    &format!("{alias}.algorithm_source_relation"),
                ),
                ("a.source_port", &format!("{alias}.algorithm_source_port")),
                ("a.source_key", &format!("{alias}.algorithm_source_key")),
            ],
        )?;
        let key_columns = source
            .primary_key
            .iter()
            .map(|name| {
                let output = format!("source_{ordinal}_{name}");
                support_fields.push(column(&alias, name).alias(&output));
                output
            })
            .collect();
        witnesses.push(NativeWitness {
            port: port.clone(),
            input: input.clone(),
            key_columns,
            when: Some(
                col("algorithm_source_relation_id")
                    .eq(source_id)
                    .and(col("algorithm_source_port").eq(lit(port.clone()))),
            ),
        });
    }
    for scope in scopes {
        let (port, input) = sources
            .role(scope)
            .ok_or_else(|| invalid("algorithm read scope role is not bound"))?;
        witnesses.push(NativeWitness {
            port: port.clone(),
            input: input.clone(),
            key_columns: vec![],
            when: None,
        });
    }
    let unknown = LogicalPlanBuilder::from(joined.clone())
        .filter(known.unwrap_or_else(|| lit(false)).eq(lit(false)))
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    crate::passes::native_rows::reject(
        unknown,
        &session,
        cancel,
        "algorithm occurrence names an unbound source relation",
    )
    .await?;
    let mut fields = spec
        .columns
        .iter()
        .map(|field| column("o", field.name()).alias(format!("result_{}", field.name())))
        .collect::<Vec<_>>();
    fields.push(column("a", "source_relation_id").alias("algorithm_source_relation_id"));
    fields.push(column("a", "source_port").alias("algorithm_source_port"));
    fields.extend(support_fields);
    let joined = LogicalPlanBuilder::from(joined)
        .project(fields)
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let columns = spec
        .columns
        .iter()
        .map(|field| (field.name().to_owned(), format!("result_{}", field.name())))
        .collect();
    Ok(NativeInput::build(joined, key, pass.id, columns, witnesses, &session, cancel).await?)
}

fn invalid(detail: &str) -> CompilerError {
    pse_templates::TemplateError::Binding {
        instance: SemanticId::NIL,
        detail: detail.to_owned(),
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passes::native_test;
    use pse_ids::FixedBudget;
    use pse_relations::generated::{authored, normalized::template_expr_int_constants as ints};

    #[test]
    fn appended_batches_keep_exact_source_ordinals_across_flushes() {
        let registry = pse_schema::catalog::assemble().unwrap();
        let budget = FixedBudget::new(1 << 20);
        let cancel = CancellationToken::new();
        let mut output = OutputRows::new(&registry, budget.as_ref(), &cancel).unwrap();
        output.ensure::<ints::Row>().unwrap();
        for (key, nodes) in [("first", vec![9]), ("second", vec![4, 12])] {
            let mut inputs = BTreeMap::new();
            native_test::put(
                &mut inputs,
                &registry,
                nodes
                    .into_iter()
                    .map(|node_id| ints::Row { node_id, value: 7 })
                    .collect(),
            );
            output
                .append_checked(inputs.remove(&ints::RELATION_KEY).unwrap(), |_, _| {
                    Ok(BTreeSet::from([SourceKey {
                        relation: authored::template_equations::RELATION_KEY,
                        port: "equations".to_owned(),
                        key: key.to_owned(),
                    }]))
                })
                .unwrap();
        }
        let generated = output.finish().unwrap();
        let values = ints::View::from_checked(&generated.columns[&ints::RELATION_KEY])
            .unwrap()
            .rows()
            .unwrap();
        let occurrences =
            provenance::algorithm_source_occurrences::View::from_checked(&generated.occurrences)
                .unwrap()
                .rows()
                .unwrap();
        let paired = occurrences
            .iter()
            .map(|source| {
                assert_eq!(source.output_relation_id, ints::RELATION_ID);
                assert_eq!(
                    source.source_relation_id,
                    authored::template_equations::RELATION_ID
                );
                assert_eq!(source.source_port, "equations");
                (
                    values[usize::try_from(source.constructed_row_ordinal).unwrap()].node_id,
                    source.source_key.as_str(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(paired, [(9, "first"), (4, "second"), (12, "second")]);
    }
}
