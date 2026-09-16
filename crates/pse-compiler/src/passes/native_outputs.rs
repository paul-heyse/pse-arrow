// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated algorithm columns couple to exact bound sources in one native plan.
pub(crate) mod support;
use super::native_rows::{column, engine, join};
use crate::{CompilerError, InputBundle};
use datafusion::{
    common::ScalarValue,
    logical_expr::{Expr, JoinType, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{SnapshotSession, output::checked_literal, scalar};
use pse_ids::{CancellationToken, MemoryReserver, Reservation, ReservationLease, SemanticId};
use pse_relations::columnar::{Collection, FieldCheckedBatch, RelationRow};
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
use support::SupportedBatch;

/// An exact key calculated by a retained source projection, never a membership proof.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SourceKey {
    pub(crate) relation: RelationKey,
    pub(crate) port: String,
    pub(crate) key: pse_ids::ContentHash,
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
    retained: BTreeMap<RelationKey, SupportedBatch>,
    cancel: &'a CancellationToken,
    support: BTreeMap<RelationKey, support::Builder>,
    positive_sources: BTreeMap<RelationKey, BTreeSet<SourceRole>>,
    read_scopes: BTreeMap<RelationKey, BTreeSet<SourceRole>>,
    registry: &'a Registry,
    reserver: &'a dyn MemoryReserver,
    work: Box<dyn Reservation>,
}

pub(crate) struct GeneratedOutputs {
    pub(crate) columns: BTreeMap<RelationKey, SupportedBatch>,
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
        cancel.checkpoint()?;
        Ok(Self {
            columns: Collection::new(registry, reserver, cancel),
            retained: BTreeMap::new(),
            cancel,
            support: BTreeMap::new(),
            positive_sources: BTreeMap::new(),
            read_scopes: BTreeMap::new(),
            registry,
            reserver,
            work: reserver.open("algorithm-output-bookkeeping"),
        })
    }
    pub(crate) fn ensure<T: RelationRow>(&mut self) -> Result<(), CompilerError> {
        self.columns.ensure::<T>()?;
        let key = T::relation(self.registry)?.key;
        if !self.support.contains_key(&key) {
            self.work
                .try_grow(8192)
                .map_err(pse_ids::CanonError::from)?;
            self.support
                .insert(key, support::Builder::new(self.registry)?);
        }
        Ok(())
    }
    pub(crate) fn push<T: RelationRow>(
        &mut self,
        row: T,
        sources: &BTreeSet<SourceKey>,
    ) -> Result<(), CompilerError> {
        self.ensure::<T>()?;
        let spec = T::relation(self.registry)?;
        self.record_sources(spec, sources)?;
        self.columns.push(row)?;
        self.support
            .get_mut(&spec.key)
            .ok_or_else(|| invalid("algorithm support builder absent"))?
            .push(sources, self.registry)?;
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
        let extent = sources.iter().try_fold(1024usize, |sum, source| {
            source
                .port
                .len()
                .checked_add(256)
                .and_then(|bytes| bytes.checked_mul(8))
                .and_then(|bytes| sum.checked_add(bytes))
                .ok_or_else(|| invalid("algorithm support capacity overflow"))
        })?;
        self.work
            .try_grow(extent)
            .map_err(pse_ids::CanonError::from)?;
        for source in sources {
            self.positive_sources
                .entry(spec.key)
                .or_default()
                .insert(source.role());
        }
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
        self.ensure::<T>()?;
        self.flush()?;
        let key = T::relation(self.registry)?.key;
        let batch = self
            .retained
            .get(&key)
            .ok_or_else(|| invalid("typed algorithm output absent"))?;
        self.work
            .try_grow(
                pse_ids::validation_extent(batch.data.batch())?
                    .checked_mul(2)
                    .ok_or_else(|| invalid("typed output view extent overflow"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        Ok(T::rows(
            &batch.payload(self.registry, T::relation(self.registry)?)?,
        )?)
    }

    /// Attach source occurrences while traversing this actual completed algorithm
    /// batch. The callback reads that same batch/ordinal; it does not rerun a producer.
    pub(crate) fn append_checked(
        &mut self,
        input: &FieldCheckedBatch,
        mut sources: impl FnMut(&FieldCheckedBatch, usize) -> Result<BTreeSet<SourceKey>, CompilerError>,
    ) -> Result<(), CompilerError> {
        self.flush()?;
        let spec = self
            .registry
            .relation_by_id(input.relation_id())
            .ok_or_else(|| invalid("algorithm output declaration absent"))?;
        input.check_declaration(self.registry, spec)?;
        self.work
            .try_grow(8192)
            .map_err(pse_ids::CanonError::from)?;
        let mut support = support::Builder::new(self.registry)?;
        for row in 0..input.batch().num_rows() {
            self.cancel.checkpoint()?;
            let sources = sources(input, row)?;
            self.record_sources(spec, &sources)?;
            support.push(&sources, self.registry)?;
        }
        self.retain(
            spec.key,
            SupportedBatch::new(
                input,
                support.finish(),
                self.registry,
                self.reserver,
                self.cancel,
            )?,
        )
    }

    /// Preserve an existing co-located batch, deriving its roles from its own lists.
    pub(crate) fn append_supported(
        &mut self,
        key: RelationKey,
        input: SupportedBatch,
    ) -> Result<(), CompilerError> {
        self.flush()?;
        let spec = self
            .registry
            .relation_by_key(key)
            .ok_or_else(|| invalid("supported output declaration absent"))?;
        self.work
            .try_grow(
                pse_ids::validation_extent(&input.data)?
                    .checked_mul(2)
                    .ok_or_else(|| invalid("supported output view extent overflow"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        input.payload(self.registry, spec)?;
        for row in 0..input.data.num_rows() {
            self.cancel.checkpoint()?;
            let sources = support::row_sources(&input.data, row, self.registry)?;
            self.record_sources(spec, &sources)?;
        }
        self.retain(key, input)
    }

    /// Export payload views while retaining their source lists for later execution.
    pub(crate) fn checked(
        &mut self,
    ) -> Result<BTreeMap<RelationKey, FieldCheckedBatch>, CompilerError> {
        self.flush()?;
        self.retained
            .iter()
            .map(|(key, batch)| {
                let spec = self
                    .registry
                    .relation_by_key(*key)
                    .ok_or_else(|| invalid("supported output declaration absent"))?;
                Ok((*key, batch.payload(self.registry, spec)?))
            })
            .collect()
    }

    fn retain(&mut self, key: RelationKey, input: SupportedBatch) -> Result<(), CompilerError> {
        let input = if let Some(prior) = self.retained.get(&key) {
            if input.data.num_rows() == 0 {
                return Ok(());
            }
            if prior.data.num_rows() == 0 {
                input
            } else {
                let extent = pse_ids::validation_extent(&prior.data)?
                    .checked_add(pse_ids::validation_extent(&input.data)?)
                    .and_then(|n| n.checked_mul(3))
                    .ok_or_else(|| invalid("supported output concatenation extent overflow"))?;
                let mut scratch = self.reserver.open("algorithm:supported-concatenation");
                scratch
                    .try_grow(extent)
                    .map_err(pse_ids::CanonError::from)?;
                self.cancel.checkpoint()?;
                let batch = datafusion::arrow::compute::concat_batches(
                    &prior.data.schema(),
                    [prior.data.batch(), input.data.batch()],
                )
                .map_err(pse_relations::RelationError::from)?;
                SupportedBatch {
                    data: pse_ids::owned_buffer::OwnedRecordBatch::export(
                        batch,
                        self.reserver,
                        self.cancel,
                    )?,
                }
            }
        } else {
            input
        };
        self.retained.insert(key, input);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), CompilerError> {
        let columns = std::mem::replace(
            &mut self.columns,
            Collection::new(self.registry, self.reserver, self.cancel),
        );
        for (key, input) in columns.finish()? {
            let support = self
                .support
                .remove(&key)
                .ok_or_else(|| invalid("algorithm support builder absent at completion"))?;
            self.retain(
                key,
                SupportedBatch::new(
                    &input,
                    support.finish(),
                    self.registry,
                    self.reserver,
                    self.cancel,
                )?,
            )?;
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
        origins: impl IntoIterator<Item = (RelationKey, pse_ids::ContentHash)>,
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
            key, batch, sources, &positives, &scopes, pass, session, cancel,
        )
        .await?;
        output.insert(key, input);
    }
    Ok(output)
}

async fn materialize_relation(
    key: RelationKey,
    batch: SupportedBatch,
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
    let source_identity = pse_schema::model::FieldContract::source_support_member()
        .children()
        .into_iter()
        .find(|field| field.name() == "source_relation_id")
        .ok_or_else(|| invalid("support source field absent"))?;
    let mut session = session
        .with_columnar_argument("algorithm_output", batch.data, cancel)
        .await?;
    let output = LogicalPlanBuilder::from(session.scan_computation_role("algorithm_output")?)
        .alias("o")
        .map_err(engine)?
        .unnest_column_with_options(
            datafusion::common::Column::new(Some("o"), support::COLUMN),
            datafusion::common::UnnestOptions::new().with_preserve_nulls(false),
        )
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let mut joined = super::native_construction::append(
        output,
        [
            datafusion::functions::core::expr_fn::get_field(
                col(support::COLUMN),
                "source_relation_id",
            )
            .alias("algorithm_occurrence_relation"),
            datafusion::functions::core::expr_fn::get_field(col(support::COLUMN), "source_port")
                .alias("algorithm_occurrence_port"),
            datafusion::functions::core::expr_fn::get_field(col(support::COLUMN), "source_key")
                .alias("algorithm_occurrence_key"),
        ],
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
            &source_identity,
            ScalarValue::FixedSizeBinary(16, Some(source.id.as_bytes().to_vec())),
        )
        .map_err(engine)?;
        let when = col("algorithm_occurrence_relation")
            .eq(source_id.clone())
            .and(col("algorithm_occurrence_port").eq(lit(port.clone())));
        let alias = format!("algorithm_source_{ordinal}");
        let matched = when.and(column(&alias, "algorithm_source_key").is_not_null());
        known =
            Some(known.map_or_else(|| matched.clone(), |previous| previous.or(matched.clone())));
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
                source.id,
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
                    "algorithm_occurrence_relation",
                    &format!("{alias}.algorithm_source_relation"),
                ),
                (
                    "algorithm_occurrence_port",
                    &format!("{alias}.algorithm_source_port"),
                ),
                (
                    "algorithm_occurrence_key",
                    &format!("{alias}.algorithm_source_key"),
                ),
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
            key_columns: Some(key_columns),
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
            key_columns: None,
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
        "algorithm occurrence names an unbound source relation or key",
    )
    .await?;
    let mut fields = spec
        .columns
        .iter()
        .map(|field| column("o", field.name()).alias(format!("result_{}", field.name())))
        .collect::<Vec<_>>();
    fields.push(col("algorithm_occurrence_relation").alias("algorithm_source_relation_id"));
    fields.push(col("algorithm_occurrence_port").alias("algorithm_source_port"));
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
    use datafusion::arrow::array::Array;
    use pse_ids::FixedBudget;
    use pse_relations::generated::{authored, normalized::template_expr_int_constants as ints};

    #[tokio::test]
    async fn colocated_support_materializes_exact_membership_and_refuses_an_unbound_key() {
        let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
        let budget = FixedBudget::new(128 << 20);
        let cancel = CancellationToken::new();
        let mut inputs = BTreeMap::new();
        native_test::put(
            &mut inputs,
            &registry,
            vec![ints::Row {
                node_id: 2,
                value: 7,
            }],
        );
        let source = inputs.remove(&ints::RELATION_KEY).unwrap();
        let (session, _) =
            native_test::session(&registry, BTreeMap::new(), &budget, &cancel).unwrap();
        let session = session
            .with_checked_role_inputs(BTreeMap::from([("source".into(), source.clone())]), &cancel)
            .unwrap();
        let mut arguments =
            super::super::native_rows::AlgorithmInputs::new(budget.as_ref(), "support-test");
        let selected = super::super::native_rows::keyed_rows::<ints::Row>(
            &mut arguments,
            session.scan_role("source").unwrap(),
            &session,
            &registry,
            &cancel,
        )
        .await
        .unwrap();
        let mut sources = Sources::new();
        sources.insert(
            ints::RELATION_KEY,
            (
                "source".into(),
                LocatedRuleInput {
                    relation: ints::RELATION_KEY,
                    location: RuleInputLocation::Facts(Arc::new(
                        pse_catalog::session::RelationFacts::from_checked(source),
                    )),
                },
            ),
        );
        let pass = registry
            .passes()
            .iter()
            .find(|pass| pass.name == "P3")
            .unwrap();
        for valid in [true, false] {
            let mut output = OutputRows::new(&registry, budget.as_ref(), &cancel).unwrap();
            output
                .push(
                    ints::Row {
                        node_id: 9,
                        value: 11,
                    },
                    &BTreeSet::from([SourceKey {
                        relation: ints::RELATION_KEY,
                        port: "source".into(),
                        key: if valid {
                            selected[0].key
                        } else {
                            pse_ids::ContentHash::from_bytes([0; 32])
                        },
                    }]),
                )
                .unwrap();
            let result =
                materialize(output.finish().unwrap(), &sources, pass, &session, &cancel).await;
            if valid {
                let result = result.unwrap();
                let rows = ints::View::from_checked(result[&ints::RELATION_KEY].checked())
                    .unwrap()
                    .rows()
                    .unwrap();
                assert_eq!(
                    rows,
                    [ints::Row {
                        node_id: 9,
                        value: 11
                    }]
                );
                assert!(result[&ints::RELATION_KEY].derivations().batch().num_rows() > 0);
            } else {
                assert!(
                    result.is_err(),
                    "an unbound support token must not disappear during UNNEST/join"
                );
            }
        }
    }

    #[test]
    fn appended_batches_keep_source_lists_in_each_row_across_flushes() {
        let registry = pse_schema::catalog::assemble().unwrap();
        let budget = FixedBudget::new(1 << 20);
        let cancel = CancellationToken::new();
        let mut output = OutputRows::new(&registry, budget.as_ref(), &cancel).unwrap();
        output.ensure::<ints::Row>().unwrap();
        for (key, nodes) in [
            (pse_ids::ContentHash::from_bytes([1; 32]), vec![9]),
            (pse_ids::ContentHash::from_bytes([2; 32]), vec![4, 12]),
        ] {
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
                .append_checked(&inputs.remove(&ints::RELATION_KEY).unwrap(), |_, _| {
                    Ok(BTreeSet::from([SourceKey {
                        relation: authored::template_equations::RELATION_KEY,
                        port: "equations".to_owned(),
                        key,
                    }]))
                })
                .unwrap();
        }
        let generated = output.finish().unwrap();
        let supported = &generated.columns[&ints::RELATION_KEY];
        let payload = supported
            .payload(&registry, ints::spec(&registry).unwrap())
            .unwrap();
        let values = ints::View::from_checked(&payload).unwrap().rows().unwrap();
        let supports = supported
            .data
            .column_by_name(support::COLUMN)
            .unwrap()
            .as_any()
            .downcast_ref::<datafusion::arrow::array::ListArray>()
            .unwrap();
        let paired = values
            .iter()
            .enumerate()
            .map(|(row, value)| {
                let list = supports.value(row);
                let source = list
                    .as_any()
                    .downcast_ref::<datafusion::arrow::array::StructArray>()
                    .unwrap();
                assert_eq!(source.len(), 1);
                let port = source
                    .column_by_name("source_port")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<datafusion::arrow::array::StringArray>()
                    .unwrap();
                assert_eq!(port.value(0), "equations");
                let relation = source
                    .column_by_name("source_relation_id")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<datafusion::arrow::array::FixedSizeBinaryArray>()
                    .unwrap();
                assert_eq!(
                    relation.value(0),
                    authored::template_equations::RELATION_ID.as_bytes()
                );
                let key = source
                    .column_by_name("source_key")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<datafusion::arrow::array::FixedSizeBinaryArray>()
                    .unwrap();
                (
                    value.node_id,
                    pse_ids::ContentHash::try_from_slice(key.value(0)).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            paired,
            [
                (9, pse_ids::ContentHash::from_bytes([1; 32])),
                (4, pse_ids::ContentHash::from_bytes([2; 32])),
                (12, pse_ids::ContentHash::from_bytes([2; 32]))
            ]
        );
    }
}
