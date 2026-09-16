// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Material candidate membership and products are selected in the native plan.
use super::{
    Configuration, Origins, invalid,
    native::{self, engine},
};
use crate::{CompilerError, passes::native_outputs::OutputRows};
use datafusion::{
    arrow::array::{Array, FixedSizeBinaryArray},
    common::{Column, NullHandling, ScalarValue, UnnestOptions},
    functions_aggregate::expr_fn::count,
    logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_ids::{SemanticId, named_id};
use pse_relations::{
    columnar::RelationRow,
    generated::{
        authored,
        enums::{DomainBindingSource, DomainKind},
        normalized, reference,
    },
};
use std::collections::BTreeMap;

pub(super) async fn bind(
    config: &mut Configuration<'_>,
    instance: &authored::instances::Row,
    source: DomainBindingSource,
    expected: DomainKind,
) -> Result<SemanticId, CompilerError> {
    let package = instance
        .property_package_id
        .ok_or_else(|| invalid("material domain has no selected property package"))?;
    let package = config
        .one::<authored::property_packages::Row>("property_package_id", package)
        .await?
        .row;
    let system = config
        .one::<authored::material_systems::Row>("material_system_id", package.material_system_id)
        .await?;
    let system_origins = config.origin(&system)?;
    let system = system.row;
    let kind = match source {
        DomainBindingSource::Species => DomainKind::Species,
        DomainBindingSource::Phase => DomainKind::Phase,
        DomainBindingSource::PhaseSpecies => DomainKind::PhaseSpecies,
        _ if source.as_str() == "element" => DomainKind::Element,
        _ => {
            return Err(invalid(
                "non-material source entered material domain binding",
            ));
        }
    };
    if kind != expected {
        return Err(invalid(
            "material source kind differs from template domain kind",
        ));
    }
    let domain = named_id(
        system.material_system_id,
        &format!("pse:material-domain:v1:{}", kind.as_str()),
    );
    if config.generated_domains.contains_key(&domain) {
        return Ok(domain);
    }
    let spec = authored::material_systems::Row::relation(config.registry)?;
    let batch = config
        .binding_batches
        .get(&spec.id)
        .ok_or_else(|| invalid("material system input absent"))?
        .clone();
    let session = config.session.with_checked_role_inputs(
        BTreeMap::from([("material_system".to_owned(), batch)]),
        config.cancel,
    )?;
    let selected = LogicalPlanBuilder::from(session.scan_role("material_system")?)
        .filter(col("material_system_id").eq(native::identity(
            config.registry,
            spec,
            "material_system_id",
            system.material_system_id,
        )?))
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let species = members(selected.clone(), "species_ids", "species_id")?;
    let phases = members(selected, "phase_ids", "phase_id")?;
    validate_members::<authored::species::Row>(config, &session, species.clone(), "species_id")
        .await?;
    validate_members::<authored::phases::Row>(config, &session, phases.clone(), "phase_id").await?;
    let null_id = lit(ScalarValue::FixedSizeBinary(16, None));
    let candidates = match kind {
        DomainKind::Species => LogicalPlanBuilder::from(species)
            .project([
                null_id.clone().alias("phase_id"),
                col("species_id"),
                null_id.alias("element_id"),
            ])
            .map_err(engine)?
            .build()
            .map_err(engine)?,
        DomainKind::Phase => LogicalPlanBuilder::from(phases)
            .project([
                col("phase_id"),
                null_id.clone().alias("species_id"),
                null_id.alias("element_id"),
            ])
            .map_err(engine)?
            .build()
            .map_err(engine)?,
        DomainKind::Element => element_candidates(config, &session, species).await?,
        _ => LogicalPlanBuilder::from(phases)
            .cross_join(species)
            .map_err(engine)?
            .project([
                col("phase_id"),
                col("species_id"),
                null_id.alias("element_id"),
            ])
            .map_err(engine)?
            .build()
            .map_err(engine)?,
    };
    let candidates = LogicalPlanBuilder::from(candidates)
        .sort([
            col("phase_id").sort(true, false),
            col("species_id").sort(true, false),
            col("element_id").sort(true, false),
        ])
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let completed = config
        .arguments
        .execute(candidates, &session, config.cancel)
        .await?;
    let generated = config
        .retain_generated(
            authored::domains::Row {
                domain_id: domain,
                owner_entity_id: system.material_system_id,
                kind,
                continuous: false,
                unit_id: None,
                parent_domain_id: None,
                doc: "Finite material-system candidate domain".to_owned(),
            },
            system_origins.clone(),
        )
        .await?;
    config.generated_domains.insert(domain, generated);
    let mut ordinal = 0_u32;
    let mut members = OutputRows::new(config.registry, config.reserver, config.cancel)?;
    members.ensure::<authored::domain_members::Row>()?;
    for batch in completed.batches() {
        let arrays = [0, 1, 2].map(|position| {
            batch
                .column(position)
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .ok_or_else(|| invalid("material coordinate is not a semantic identity"))
        });
        let [phase, species, element] = arrays;
        let (phase, species, element) = (phase?, species?, element?);
        for row in 0..batch.num_rows() {
            config.cancel.checkpoint()?;
            let coordinates = (
                identity(phase, row)?,
                identity(species, row)?,
                identity(element, row)?,
            );
            let origins = coordinate_origins(config, &system, &system_origins, coordinates).await?;
            members.push(
                emit_member(
                    config,
                    domain,
                    system.material_system_id,
                    ordinal,
                    coordinates,
                    &origins,
                )?,
                &origins,
            )?;
            ordinal = ordinal
                .checked_add(1)
                .ok_or_else(|| invalid("material domain ordinal exceeds u32"))?;
        }
    }
    for (key, batch) in members.finish()?.columns {
        config.remember_generated(key, &batch).await?;
        let spec = config
            .registry
            .relation_by_key(key)
            .ok_or_else(|| invalid("generated member declaration absent"))?;
        config
            .merge_generated(batch.payload(config.registry, spec)?)
            .await?;
    }
    Ok(domain)
}

fn members(input: LogicalPlan, field: &str, output: &str) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(input)
        .project([col(field)])
        .map_err(engine)?
        .unnest_column_with_options(
            Column::from_name(field),
            UnnestOptions::new().with_null_handling(NullHandling::Drop),
        )
        .map_err(engine)?
        .project([col(field).alias(output)])
        .map_err(engine)?
        .build()
        .map_err(engine)
}
async fn validate_members<T: RelationRow>(
    config: &mut Configuration<'_>,
    session: &pse_catalog::session::SnapshotSession,
    members: LogicalPlan,
    field: &str,
) -> Result<(), CompilerError> {
    let duplicates = LogicalPlanBuilder::from(members.clone())
        .aggregate([col(field)], [count(lit(1_i64)).alias("n")])
        .map_err(engine)?
        .filter(col("n").not_eq(lit(1_i64)))
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    crate::passes::native_rows::reject(
        duplicates,
        session,
        config.cancel,
        "material system repeats a member",
    )
    .await?;
    let source = config
        .binding_batches
        .get(&T::relation(config.registry)?.id)
        .ok_or_else(|| invalid("material member declaration absent"))?
        .clone();
    let selected = session.with_checked_role_inputs(
        BTreeMap::from([("material_declarations".to_owned(), source)]),
        config.cancel,
    )?;
    let declarations = LogicalPlanBuilder::from(selected.scan_role("material_declarations")?)
        .alias("d")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let members = LogicalPlanBuilder::from(members)
        .alias("m")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let missing = native::join(
        members,
        declarations,
        JoinType::LeftAnti,
        &[(&format!("m.{field}"), &format!("d.{field}"))],
    )?;
    crate::passes::native_rows::reject(
        missing,
        &selected,
        config.cancel,
        "material system references an absent member",
    )
    .await
}
async fn element_candidates(
    config: &mut Configuration<'_>,
    session: &pse_catalog::session::SnapshotSession,
    species: LogicalPlan,
) -> Result<LogicalPlan, CompilerError> {
    let composition = native::scan(
        session,
        authored::species_elements::Row::relation(config.registry)?,
        "c",
    )?;
    let species = LogicalPlanBuilder::from(species)
        .alias("s")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let missing = native::join(
        species.clone(),
        composition.clone(),
        JoinType::LeftAnti,
        &[("s.species_id", "c.species_id")],
    )?;
    crate::passes::native_rows::reject(
        missing,
        session,
        config.cancel,
        "element domain requires composition of every selected species",
    )
    .await?;
    let matched = native::join(
        species,
        composition,
        JoinType::Inner,
        &[("s.species_id", "c.species_id")],
    )?;
    let elements = native::scan(
        session,
        reference::elements::Row::relation(config.registry)?,
        "e",
    )?;
    let missing = native::join(
        matched.clone(),
        elements,
        JoinType::LeftAnti,
        &[("c.element_id", "e.element_id")],
    )?;
    crate::passes::native_rows::reject(
        missing,
        session,
        config.cancel,
        "species composition references an absent element",
    )
    .await?;
    LogicalPlanBuilder::from(matched)
        .project([col("c.element_id").alias("element_id")])
        .map_err(engine)?
        .distinct()
        .map_err(engine)?
        .project([
            lit(ScalarValue::FixedSizeBinary(16, None)).alias("phase_id"),
            lit(ScalarValue::FixedSizeBinary(16, None)).alias("species_id"),
            col("element_id"),
        ])
        .map_err(engine)?
        .build()
        .map_err(engine)
}
fn identity(array: &FixedSizeBinaryArray, row: usize) -> Result<Option<SemanticId>, CompilerError> {
    if array.is_null(row) {
        return Ok(None);
    }
    Ok(Some(SemanticId::from_bytes(
        array
            .value(row)
            .try_into()
            .map_err(|_| invalid("material identity has wrong width"))?,
    )))
}
fn emit_member(
    config: &mut Configuration<'_>,
    domain: SemanticId,
    system: SemanticId,
    ordinal: u32,
    (phase, species, element): (Option<SemanticId>, Option<SemanticId>, Option<SemanticId>),
    origins: &Origins,
) -> Result<authored::domain_members::Row, CompilerError> {
    let member = if let Some(element) = element {
        named_id(domain, &format!("member:element:{}", element.to_hex()))
    } else {
        named_id(
            domain,
            &format!(
                "member:{}:{}",
                phase.map_or_else(String::new, SemanticId::to_hex),
                species.map_or_else(String::new, SemanticId::to_hex)
            ),
        )
    };
    let label = if let Some(element) = element {
        element.to_hex()
    } else {
        match (phase, species) {
            (Some(phase), Some(species)) => format!("{phase}/{species}"),
            (Some(id), None) | (None, Some(id)) => id.to_hex(),
            _ => return Err(invalid("material member has no subject")),
        }
    };
    let row = authored::domain_members::Row {
        domain_id: domain,
        member_id: member,
        ordinal,
        label,
        coordinate: None,
        ref_entity_id: if phase.is_some() && species.is_some() {
            None
        } else {
            phase.or(species).or(element)
        },
    };
    config.columns.push(
        normalized::material_domain_members::Row {
            domain_id: domain,
            member_id: member,
            material_system_id: system,
            phase_id: phase,
            species_id: species,
            element_id: element,
            derivation_id: named_id(member, "pass:P3@1"),
        },
        origins,
    )?;
    Ok(row)
}

async fn coordinate_origins(
    config: &mut Configuration<'_>,
    system: &authored::material_systems::Row,
    base: &Origins,
    (phase, species, element): (Option<SemanticId>, Option<SemanticId>, Option<SemanticId>),
) -> Result<Origins, CompilerError> {
    let mut origins = base.clone();
    if let Some(id) = phase {
        let source = config.one::<authored::phases::Row>("phase_id", id).await?;
        origins.extend(config.origin(&source)?);
    }
    if let Some(id) = species {
        let source = config
            .one::<authored::species::Row>("species_id", id)
            .await?;
        origins.extend(config.origin(&source)?);
    }
    if let Some(id) = element {
        let source = config
            .one::<reference::elements::Row>("element_id", id)
            .await?;
        origins.extend(config.origin(&source)?);
        let spec = authored::species_elements::spec(config.registry)?;
        let species = system
            .species_ids
            .iter()
            .map(|id| native::identity(config.registry, spec, "species_id", *id))
            .collect::<Result<Vec<_>, _>>()?;
        let sources = config
            .select::<authored::species_elements::Row>(vec![
                col("element_id").eq(native::identity(config.registry, spec, "element_id", id)?),
                col("species_id").in_list(species, false),
            ])
            .await?;
        if sources.is_empty() {
            return Err(invalid(
                "material element has no selected composition source",
            ));
        }
        for source in &sources {
            origins.extend(config.origin(source)?);
        }
    }
    Ok(origins)
}
