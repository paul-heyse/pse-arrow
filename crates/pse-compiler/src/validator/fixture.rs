// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The explicit fixture importer admits exactly the authored complete fixture.
use crate::{
    CompilerError, InputBundle,
    passes::{dag::invalid, p10::fixture},
};
use pse_catalog::Catalog;
use pse_ids::CancellationToken;
use pse_relations::RecordBatch;
use pse_schema::model::PassSpec;
use std::collections::BTreeMap;

pub(super) fn validate(
    catalog: &Catalog,
    spec: &PassSpec,
    inputs: &InputBundle,
    candidates: &BTreeMap<String, RecordBatch>,
    cancel: &CancellationToken,
) -> Result<(), CompilerError> {
    let registry = catalog.registry();
    let expected_inputs = fixture::model_inputs(registry)?;
    let actual_inputs = inputs.rows(registry)?;
    for port in &spec.inputs {
        let key = registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("fixture input undeclared"))?
            .key;
        let expected = expected_inputs
            .get(&key)
            .ok_or_else(|| invalid("fixed fixture input absent"))?;
        let actual = actual_inputs
            .get(&key)
            .ok_or_else(|| invalid("actual fixture input absent"))?;
        super::compare::rows(
            catalog,
            &port.relation,
            std::slice::from_ref(expected),
            std::slice::from_ref(actual),
            cancel,
        )?;
    }
    let (expected, _) = fixture::arithmetic_inputs(registry)?;
    for port in &spec.outputs {
        let key = registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("fixture output undeclared"))?
            .key;
        let expected = expected
            .get(&key)
            .ok_or_else(|| invalid("fixed fixture output absent"))?;
        let actual = candidates
            .get(port.port)
            .ok_or_else(|| invalid("actual fixture output absent"))?;
        super::compare::rows(
            catalog,
            &port.relation,
            std::slice::from_ref(expected),
            std::slice::from_ref(actual),
            cancel,
        )?;
    }
    Ok(())
}
