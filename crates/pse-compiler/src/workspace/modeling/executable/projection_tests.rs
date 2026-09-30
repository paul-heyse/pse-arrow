// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Projected outputs carry only the expression members they read (H8).
use super::*;
use crate::workspace::{CompilerWorkspace, WorkspaceLimits};
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_modeling::{Limits, specialize::Bindings};

/// A mesh of `cells` replicas, each with a local expression member and one equation, and
/// one member that reads every replica.
fn projected(cells: usize) -> Arc<Projection> {
    let text = format!(
        "package p {{ difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0,1); def Cell {{ var x:Scalar; let y:Scalar=2*x; eq e:y==1; }} def Root {{ domain t:Scalar from 0 to 1; discretize grid on t using backward(elements={},order=1); child cell[i in t]:Cell=Cell(); let total:Scalar=integral(i in t | cell[i].y); eq closure:total==1; }} }}",
        cells - 1
    );
    let rows = parse(
        &text,
        SemanticId::from_bytes([83; 16]),
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let input = crate::workspace::tests::inputs();
    let mut workspace = CompilerWorkspace::new(input, WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    let (catalog, request) = workspace
        .modeling_request(
            root,
            InstanceId::from_id(SemanticId::NIL),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    projection(&workspace.db, workspace.inventory, catalog, request).unwrap()
}
fn members(expression: &Expr) -> usize {
    match &expression.kind {
        ExprKind::Let { bindings, .. } => bindings.len(),
        _ => 0,
    }
}

#[test]
fn projected_outputs_bind_only_the_members_they_read() {
    let cells = 9;
    let p = projected(cells);
    let rows = p
        .outputs
        .iter()
        .zip(&p.expressions)
        .filter(|(output, _)| matches!(output, ModelingOutput::Equation { .. }))
        .map(|(_, e)| members(e))
        .collect::<Vec<_>>();
    // One local member per replica equation; the closure reads the total and the member
    // of every replica the backward quadrature weights (all but the first).
    assert_eq!(rows.len(), cells + 1);
    assert_eq!(rows.iter().filter(|&&m| m == 1).count(), cells);
    assert_eq!(rows.iter().filter(|&&m| m == cells).count(), 1);
}

#[test]
fn projection_size_grows_linearly_with_replicas() {
    let size = |cells| {
        projected(cells)
            .expressions
            .iter()
            .map(pse_modeling::expression::retained_bytes)
            .sum::<usize>()
    };
    let (small, large) = (size(9), size(36));
    // Four times the replicas: linear growth is a factor of four, the former wrapping of
    // every output in every member a factor of sixteen.
    assert!(large <= 5 * small, "{small} -> {large}");
}
