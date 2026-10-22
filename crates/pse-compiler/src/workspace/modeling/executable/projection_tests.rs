// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Projected typed regions carry only the members and guards they read (H8).
use super::*;
use crate::workspace::{CompilerWorkspace, WorkspaceLimits};
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_modeling::{Limits, specialize::Bindings};

#[test]
fn dependency_index_reaches_unexpressed_rates_and_preserves_guard_closure() {
    let id = |n| SemanticId::from_bytes([n; 16]);
    let mut index = dependencies::Dependencies::new((1..=7).map(id));
    index.numerical(id(2), [id(1)]);
    index.numerical(id(3), [id(2)]);
    index.numerical(id(4), [id(3)]);
    index.guard(id(3), [id(5)]);
    index.numerical(id(5), [id(6)]);
    let workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let reached = index.consumers([id(1)], &workspace.db);
    assert_eq!(reached, BTreeSet::from([id(1), id(2), id(3), id(4)]));
    assert!(!reached.contains(&id(7)));
    let members = BTreeSet::from([id(2), id(3), id(5)]);
    let order = index.order(&members).unwrap();
    assert!(
        order
            .iter()
            .position(|id| *id == SemanticId::from_bytes([2; 16]))
            .unwrap()
            < order
                .iter()
                .position(|id| *id == SemanticId::from_bytes([3; 16]))
                .unwrap()
    );
    index.numerical(id(2), [id(3)]);
    assert!(index.order(&members).is_err());
}

#[test]
fn dependency_index_traversal_honors_cancellation() {
    let id = SemanticId::from_bytes([1; 16]);
    let index = dependencies::Dependencies::new([id]);
    let workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace.db.cancel.store(true, Ordering::Release);
    assert!(salsa::Cancelled::catch(|| index.consumers([id], &workspace.db)).is_err());
}

#[test]
fn member_reads_follow_guards_on_free_inputs_transitively() {
    let p = projected_source(
        "package p { def Root { var x:Scalar; var z:Scalar; var unused:Scalar; let lower:Scalar=z+1; let floor:Scalar=z; annotation valid x(lower,10); annotation valid lower(floor,10); eq e:x==1; } }",
    );
    let member = |name: &str| {
        p.model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
            .unwrap_or_else(|| panic!("missing actual member {name}"))
            .id
    };
    let index = p
        .outputs
        .iter()
        .position(|output| matches!(output, ModelingOutput::Equation { .. }))
        .unwrap();
    let region = &p.regions[index];
    assert_eq!(region.reads.numeric, BTreeSet::from([member("x")]));
    assert_eq!(
        region.reads.guards,
        BTreeSet::from([member("lower"), member("floor"), member("z")])
    );
    let selected = region
        .bindings
        .iter()
        .map(|binding| binding.actual)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selected,
        BTreeSet::from([member("x"), member("lower"), member("floor"), member("z")])
    );
    assert!(!selected.contains(&member("unused")));
    assert_eq!(
        region
            .checked_members
            .values()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([member("x"), member("lower")])
    );
    assert_eq!(region.checked_sources.len(), 2);
    assert_eq!(
        region.support.checked_ranges(&region.syntax).unwrap().len(),
        2
    );
}

/// A mesh of `cells` replicas, each with a local expression member and one equation, and
/// one member that reads every replica.
fn projected(cells: usize) -> Arc<Projection> {
    let text = format!(
        "package p {{ difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0,1); def Cell {{ var x:Scalar; let y:Scalar=2*x; eq e:y==1; }} def Root {{ domain t:Scalar from 0 to 1; discretize grid on t using backward(elements={},order=1); child cell[i in t]:Cell=Cell(); let total:Scalar=integral(i in t | cell[i].y); eq closure:total==1; }} }}",
        cells - 1
    );
    projected_source(&text)
}

fn projected_source(text: &str) -> Arc<Projection> {
    let rows = parse(
        text,
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
    let input = crate::authored_transfer_tests::context();
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
#[test]
fn projected_outputs_bind_only_the_members_they_read() {
    let cells = 9;
    let p = projected(cells);
    let rows = p
        .outputs
        .iter()
        .zip(&p.regions)
        .filter(|(output, _)| matches!(output, ModelingOutput::Equation { .. }))
        .map(|(_, region)| {
            region
                .bindings
                .iter()
                .filter(|binding| binding.role == pse_modeling::derived::LocalRole::Member)
                .count()
        })
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
            .regions
            .iter()
            .map(pse_modeling::resolved::SelectedRegion::retained_bytes)
            .sum::<usize>()
    };
    let (small, large) = (size(9), size(36));
    // Four times the replicas: linear growth is a factor of four, the former wrapping of
    // every output in every member a factor of sixteen.
    assert!(large <= 5 * small, "{small} -> {large}");
}

#[test]
fn supplier_topology_owner_attachment_shares_memo_and_keeps_original_view_separate() {
    let rows = crate::authored_transfer_tests::rows(
        "package p { def Root { param p:Scalar=2; implicit a { var selected_value:Scalar; eq selected:selected_value==p; annotation start selected_value(1); annotation bounds selected_value(0,4); } realize ra on a using nested; var y:Scalar; eq e:y==a.selected_value; } }",
    );
    let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let original = prepared.original_equations().unwrap();
    assert!(!Arc::ptr_eq(
        &prepared.admitted.supplier_topology,
        &original.admitted.supplier_topology
    ));
    let before = prepared.retained_bytes();
    let attached = prepared.clone().with_owner(Arc::new("topology-owner"));
    assert!(Arc::ptr_eq(
        &prepared.admitted.supplier_topology,
        &attached.admitted.supplier_topology
    ));
    assert!(Arc::ptr_eq(
        &original.admitted.supplier_topology,
        &attached
            .original_equations()
            .unwrap()
            .admitted
            .supplier_topology
    ));
    assert_eq!(
        prepared.admitted.supplier_topology_bytes, attached.admitted.supplier_topology_bytes,
        "owner attachment retains one unchanged memo reservation"
    );
    assert_eq!(
        original.admitted.supplier_topology_bytes,
        attached
            .original_equations()
            .unwrap()
            .admitted
            .supplier_topology_bytes
    );
    assert!(
        attached.retained_bytes() <= before,
        "owner attachment does not add a second memo charge"
    );
    attached.implicit_order_for(Some(&BTreeSet::new())).unwrap();
    assert!(prepared.admitted.supplier_topology.get().is_some());
    assert!(
        original.admitted.supplier_topology.get().is_none(),
        "view selection does not fill another view's memo"
    );
    assert_eq!(before, prepared.retained_bytes());
}

#[test]
fn resolved_fixture_specifications_are_observed_in_the_first_projection() {
    let rows = crate::authored_transfer_tests::rows(
        "package p { test Root fixture { dof 0; value unused=3; } { param unused:Scalar=2; var x:Scalar; eq e:x==1; annotation start x(0); } }",
    );
    let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    let instance = pse_modeling::specialize::root_instance(root);
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let fixture = &prepared.model.fixtures[&instance];
    let target = fixture.specifications["unused"].target;
    assert!(prepared.admitted.inputs.contains(&target));
    assert_eq!(
        prepared
            .case_values(&ModelingCaseBindings::from(fixture))
            .unwrap()
            .scalars[&target],
        3.0
    );
}

#[test]
fn prepared_pure_checks_do_not_read_a_later_workspace_revision() {
    let text = "package p { test Root fixture {dof 1; procedure check; value x=3;} {var x:Scalar; expect x==3 tolerance 1e-12;} }";
    let rows = crate::authored_transfer_tests::rows(text);
    let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
    let instance = pse_modeling::specialize::root_instance(root);
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    let cancel = pse_diagnostics::Cancellation::default();
    let model = workspace
        .prepare_modeling_cancellable(
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            cancel.flag(),
        )
        .unwrap();
    workspace
        .publish_modeling(
            crate::authored_transfer_tests::rows(&text.replace("value x=3", "value x=4")),
            PhysicalScope::default(),
        )
        .unwrap();
    let checked = workspace
        .check_prepared_modeling_point(
            &model,
            root,
            instance,
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            cancel,
        )
        .unwrap();
    assert!(checked.expectations.iter().all(|row| row.passed));
}

#[test]
fn implicit_projection_refuses_scratch_before_building_indexes() {
    let rows =
        crate::authored_transfer_tests::rows("package p {def Root {var x:Scalar; eq e:x==1;}}");
    let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    workspace.db.projection_scratch_limit = 0;
    assert!(matches!(
        workspace.prepare_modeling_cancellable(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            Arc::new(AtomicBool::new(false)),
        ),
        Err(CompileError::Limit("implicit projection scratch allowance"))
    ));
}
