// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parse port grammar once and couple generated rows to their exact native sources.

#[cfg(test)]
mod tests;

use super::invalid;
use crate::{
    CompilerError,
    passes::{
        native_outputs::{OutputRows, SourceKey, Sources, materialize},
        native_rows::AlgorithmInputs,
    },
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{
    authored::template_ports,
    normalized::{port_binding_lengths, port_binding_steps},
};
use pse_rules::strata::native_input::NativeInput;
use pse_schema::model::RelationKey;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(super) async fn emit(
    sources: &Sources,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let mut arguments = AlgorithmInputs::new(session.reserver(), "P3:port-path-inputs");
    let ports =
        super::native::rows::<template_ports::Row>(&mut arguments, sources, session, cancel)
            .await?;
    let registry = session.registry();
    let pass = registry
        .algorithm("P3@1")
        .ok_or_else(|| invalid("P3 declaration absent"))?;
    let mut output = OutputRows::new(registry, session.reserver(), cancel)?;
    parse(ports, &mut output, cancel)?;
    materialize(output.finish()?, sources, pass, session, cancel).await
}

fn parse(
    ports: Vec<(template_ports::Row, SourceKey)>,
    output: &mut OutputRows<'_>,
    cancel: &CancellationToken,
) -> Result<(), CompilerError> {
    output.ensure::<port_binding_lengths::Row>()?;
    output.ensure::<port_binding_steps::Row>()?;
    for (port, source) in ports {
        cancel.checkpoint()?;
        let support = BTreeSet::from([source]);
        let mut length = 0_i64;
        if port.bound_to != "self" {
            for child in port.bound_to.split('.') {
                cancel.checkpoint()?;
                if !identifier(child) {
                    return Err(invalid(&format!(
                        "port {} binding requires self or an ordered dotted child path; invalid segment {child:?}",
                        port.name
                    )));
                }
                output.push(
                    port_binding_steps::Row {
                        template_id: port.template_id,
                        name: port.name.clone(),
                        position: length,
                        child_name: child.to_owned(),
                        derivation_id: SemanticId::NIL,
                    },
                    &support,
                )?;
                length = length
                    .checked_add(1)
                    .ok_or_else(|| invalid("port path length overflow"))?;
            }
        }
        output.push(
            port_binding_lengths::Row {
                template_id: port.template_id,
                name: port.name,
                length,
                derivation_id: SemanticId::NIL,
            },
            &support,
        )?;
    }
    Ok(())
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.chars().enumerate().all(|(index, character)| {
            character == '_' || character.is_alphabetic() || index > 0 && character.is_ascii_digit()
        })
}
