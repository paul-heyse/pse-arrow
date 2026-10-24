// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independently retained canonical physical inputs for entity-local packages.
use super::*;

#[derive(Clone, Debug)]
pub(super) struct SharedOwner<T> {
    owner: Arc<T>,
}
impl<T: PartialEq> PartialEq for SharedOwner<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner) || self.owner == other.owner
    }
}
impl<T: Eq> Eq for SharedOwner<T> {}
impl<T> std::hash::Hash for SharedOwner<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Complete immutable values own equality; allocation capacity is not meaning.
        std::hash::Hash::hash(&size_of::<T>(), state);
    }
}
#[salsa::interned(heap_size=quantity_heap)]
pub(super) struct Quantities<'db> {
    value: SharedOwner<QuantityRegistry>,
}
fn quantity_heap((value,): &(SharedOwner<QuantityRegistry>,)) -> usize {
    value.owner.allocation_extent()
}
#[salsa::interned(heap_size=preconditions_heap)]
pub(super) struct Preconditions<'db> {
    value: SharedOwner<PhysicalPreconditions>,
}
fn preconditions_heap((value,): &(SharedOwner<PhysicalPreconditions>,)) -> usize {
    value.owner.allocation_extent()
}
#[salsa::interned(heap_size=scope_heap)]
pub(super) struct Scope<'db> {
    value: SharedOwner<PhysicalScope>,
}
fn scope_heap((value,): &(SharedOwner<PhysicalScope>,)) -> usize {
    pse_modeling::SharedPackageInputs::scope_retained_bytes(&value.owner)
}
#[salsa::interned(heap_size=types_heap)]
pub(super) struct PhysicalTypes<'db> {
    value: SharedOwner<BTreeMap<String, pse_modeling::Type>>,
}
fn types_heap((value,): &(SharedOwner<BTreeMap<String, pse_modeling::Type>>,)) -> usize {
    pse_modeling::SharedPackageInputs::physical_types_retained_bytes(&value.owner)
}

// Immutable payload interning happens only inside tracked projection producers,
// never on each field/function callback. Parent/selector handles belong to the
// producer key; equal complete payloads still share across independent parents.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct StaticFunctionPackage(Arc<CheckedPackage>);
impl Eq for StaticFunctionPackage {}
impl std::hash::Hash for StaticFunctionPackage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Bucket hint only; complete checked equality remains the payload authority.
        std::hash::Hash::hash(
            &self.0.declarations().next().map(|row| row.declaration_id),
            state,
        );
    }
}
#[salsa::interned(heap_size=package_heap)]
pub(super) struct StaticFunctionPackageKey<'db> {
    package: StaticFunctionPackage,
    quantities: Quantities<'db>,
    preconditions: Preconditions<'db>,
    scope: Scope<'db>,
    types: PhysicalTypes<'db>,
}
fn package_heap(
    (package, _, _, _, _): &(
        StaticFunctionPackage,
        Quantities<'_>,
        Preconditions<'_>,
        Scope<'_>,
        PhysicalTypes<'_>,
    ),
) -> usize {
    package.0.retained_bytes_excluding_shared_inputs()
}
#[salsa::interned(heap_size=selection_heap)]
pub(super) struct Selection<'db> {
    value: pse_modeling::check::StaticPackageSelection,
}
fn selection_heap((value,): &(pse_modeling::check::StaticPackageSelection,)) -> usize {
    value.retained_bytes()
}
// Errors contain owned static diagnostics; the wrapper lets Salsa verify the
// lifetime-bearing Result structurally without erasing its typed cause.
#[derive(Clone, Debug, PartialEq, salsa::SalsaValue)]
struct SelectionFailure {
    cause: CompileError,
}
impl From<CompileError> for SelectionFailure {
    fn from(cause: CompileError) -> Self {
        Self { cause }
    }
}
impl From<pse_modeling::ModelingError> for SelectionFailure {
    fn from(cause: pse_modeling::ModelingError) -> Self {
        Self {
            cause: cause.into(),
        }
    }
}
type PackageResult<'db> = std::result::Result<StaticFunctionPackageKey<'db>, SelectionFailure>;
#[salsa::tracked(returns(clone), lru = 64)]
fn root_package_query<'db>(
    db: &'db dyn CompilerDb,
    catalog: Catalog,
    root: DeclarationId,
) -> PackageResult<'db> {
    checkpoint(db);
    let selector = pse_modeling::check::StaticPackageSelection::root(root);
    let package = Arc::new(selector.select(catalog.checked(db))?);
    Ok(own_static_package(db, package)?)
}
#[salsa::tracked(returns(clone), lru = 256)]
fn selected_package<'db>(
    db: &'db dyn CompilerDb,
    parent: StaticFunctionPackageKey<'db>,
    selection: Selection<'db>,
) -> PackageResult<'db> {
    checkpoint(db);
    let source = read_static_package(db, &parent)?;
    let package = Arc::new(selection.value(db).select(&source)?);
    Ok(own_static_package(db, package)?)
}
pub(super) fn root_package<'db>(
    db: &'db dyn CompilerDb,
    catalog: Catalog,
    root: DeclarationId,
) -> Result<StaticFunctionPackageKey<'db>> {
    root_package_query(db, catalog, root).map_err(|error| error.cause)
}
pub(super) fn select_package<'db>(
    db: &'db dyn CompilerDb,
    parent: StaticFunctionPackageKey<'db>,
    selection: pse_modeling::check::StaticPackageSelection,
) -> Result<StaticFunctionPackageKey<'db>> {
    selected_package(db, parent, Selection::new(db, selection)).map_err(|error| error.cause)
}
pub(super) fn configure(db: &mut dyn CompilerDb, capacity: usize) {
    root_package_query::set_lru_capacity(db, capacity);
    selected_package::set_lru_capacity(db, capacity);
}
fn same_owners(
    a: &pse_modeling::SharedPackageInputs,
    b: &pse_modeling::SharedPackageInputs,
) -> bool {
    Arc::ptr_eq(&a.quantities, &b.quantities)
        && Arc::ptr_eq(&a.preconditions, &b.preconditions)
        && Arc::ptr_eq(&a.scope, &b.scope)
        && Arc::ptr_eq(&a.physical_types, &b.physical_types)
}

fn own_static_package<'db>(
    db: &'db dyn CompilerDb,
    mut package: Arc<CheckedPackage>,
) -> std::result::Result<StaticFunctionPackageKey<'db>, pse_modeling::ModelingError> {
    let inputs = package.shared_inputs();
    let quantities = Quantities::new(
        db,
        SharedOwner {
            owner: inputs.quantities.clone(),
        },
    );
    let preconditions = Preconditions::new(
        db,
        SharedOwner {
            owner: inputs.preconditions.clone(),
        },
    );
    let scope = Scope::new(
        db,
        SharedOwner {
            owner: inputs.scope.clone(),
        },
    );
    let types = PhysicalTypes::new(
        db,
        SharedOwner {
            owner: inputs.physical_types.clone(),
        },
    );
    let canonical = pse_modeling::SharedPackageInputs {
        quantities: quantities.value(db).owner.clone(),
        preconditions: preconditions.value(db).owner.clone(),
        scope: scope.value(db).owner.clone(),
        physical_types: types.value(db).owner.clone(),
    };
    if !same_owners(&inputs, &canonical) {
        Arc::make_mut(&mut package).canonicalize_shared_inputs(&canonical)?;
    }
    Ok(StaticFunctionPackageKey::new(
        db,
        StaticFunctionPackage(package),
        quantities,
        preconditions,
        scope,
        types,
    ))
}

pub(super) fn read_static_package(
    db: &dyn CompilerDb,
    key: &StaticFunctionPackageKey<'_>,
) -> std::result::Result<Arc<CheckedPackage>, pse_modeling::ModelingError> {
    // Read the owner ingredients in the consuming query too. A retained local
    // package cannot outlive its independently charged physical inputs unnoticed.
    let owners = pse_modeling::SharedPackageInputs {
        quantities: key.quantities(db).value(db).owner.clone(),
        preconditions: key.preconditions(db).value(db).owner.clone(),
        scope: key.scope(db).value(db).owner.clone(),
        physical_types: key.types(db).value(db).owner.clone(),
    };
    let package = key.package(db).0.clone();
    if !same_owners(&package.shared_inputs(), &owners) {
        return Err(pse_modeling::ModelingError::Contract {
            reason: pse_modeling::ModelingContractReason::Unclassified,
            declaration: SemanticId::NIL,
            message: "static package lost its canonical shared input owner".into(),
        });
    }
    Ok(package)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[salsa::tracked(returns(clone))]
    fn fixture_owner<'db>(db: &'db dyn CompilerDb, catalog: Catalog) -> PackageResult<'db> {
        Ok(own_static_package(db, catalog.checked(db).clone())?)
    }
    #[test]
    fn tracked_static_selection_reuses_exact_parent_reads_and_preserves_attribution() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let executions = Arc::new(AtomicUsize::new(0));
        let observed = executions.clone();
        let mut workspace = CompilerWorkspace::with_events(
            crate::authored_transfer_tests::context(),
            WorkspaceLimits::default(),
            Some(Box::new(move |event: salsa::Event| {
                let event = format!("{:?}", event.kind);
                if event.contains("WillExecute") && event.contains("selected_package(") {
                    observed.fetch_add(1, Ordering::Relaxed);
                }
            })),
        )
        .unwrap();
        let rows = pse_authoring::language::parse(
            "package p {def Left {var x:Scalar;} def Right {var y:Scalar;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let declaration = |name: &str| {
            rows.iter()
                .find(|row| row.name == name)
                .unwrap()
                .declaration_id
        };
        let left = declaration("Left");
        let y = declaration("y");
        let revision = workspace
            .publish_modeling(rows, PhysicalScope::default())
            .unwrap();
        let catalog = Catalog::new(&workspace.db, revision.checked().clone());
        let parent = fixture_owner(&workspace.db, catalog).unwrap();
        let selection = pse_modeling::check::StaticPackageSelection::static_scope(left, Vec::new());
        let first = select_package(&workspace.db, parent, selection.clone()).unwrap();
        let again = select_package(&workspace.db, parent, selection).unwrap();
        assert!(first == again);
        assert_eq!(
            executions.load(Ordering::Relaxed),
            1,
            "same exact projection is not rebuilt or compared as a package key"
        );
        let original = read_static_package(&workspace.db, &first).unwrap();
        assert!(!original.declarations().any(|row| row.declaration_id == y));
        assert!(
            original
                .declarations()
                .any(|row| row.declaration_id == left && row.source_start > 0)
        );
        let selected = select_package(
            &workspace.db,
            parent,
            pse_modeling::check::StaticPackageSelection::static_scope(left, vec![y]),
        )
        .unwrap();
        assert!(
            read_static_package(&workspace.db, &selected)
                .unwrap()
                .declarations()
                .any(|row| row.declaration_id == y)
        );
        let same_reads = select_package(
            &workspace.db,
            parent,
            pse_modeling::check::StaticPackageSelection::static_scope(left, vec![y, y]),
        )
        .unwrap();
        assert!(selected == same_reads);
        assert_eq!(executions.load(Ordering::Relaxed), 2);
        let field = select_package(
            &workspace.db,
            parent,
            pse_modeling::check::StaticPackageSelection::field(left, Vec::new()),
        )
        .unwrap();
        assert!(
            read_static_package(&workspace.db, &field)
                .unwrap()
                .declarations()
                .all(|row| row.source_start == 0 && row.source_end == 0)
        );
        assert_eq!(executions.load(Ordering::Relaxed), 3);
        let missing = select_package(
            &workspace.db,
            parent,
            pse_modeling::check::StaticPackageSelection::static_scope(
                SemanticId::from_bytes([255; 16]).into(),
                Vec::new(),
            ),
        );
        assert!(matches!(
            missing,
            Err(CompileError::Modeling(
                pse_modeling::ModelingError::Contract { .. }
            ))
        ));
    }

    #[test]
    fn equal_packages_share_canonical_physical_allocations() {
        let mut workspace = CompilerWorkspace::new(
            crate::authored_transfer_tests::context(),
            WorkspaceLimits::default(),
        )
        .unwrap();
        let rows = pse_authoring::language::parse(
            "package p {def Root {var x:Scalar;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let revision = workspace
            .publish_modeling(rows, PhysicalScope::default())
            .unwrap();
        let original = revision.checked().clone();
        let mut separate = original.as_ref().clone();
        let inputs = original.shared_inputs();
        let separate_inputs = pse_modeling::SharedPackageInputs {
            quantities: Arc::new(inputs.quantities.as_ref().clone()),
            preconditions: Arc::new(inputs.preconditions.as_ref().clone()),
            scope: Arc::new(inputs.scope.as_ref().clone()),
            physical_types: Arc::new(inputs.physical_types.as_ref().clone()),
        };
        separate
            .canonicalize_shared_inputs(&separate_inputs)
            .unwrap();
        assert!(!same_owners(&inputs, &separate.shared_inputs()));
        let source = Catalog::new(&workspace.db, original);
        let first = fixture_owner(&workspace.db, source).unwrap();
        assert!(first == fixture_owner(&workspace.db, source).unwrap());
        let independent = Catalog::new(&workspace.db, Arc::new(separate));
        let second = fixture_owner(&workspace.db, independent).unwrap();
        assert!(
            first == second,
            "equal complete content canonicalizes across independent parents"
        );
        assert!(first.quantities(&workspace.db) == second.quantities(&workspace.db));
        assert!(first.preconditions(&workspace.db) == second.preconditions(&workspace.db));
        assert!(first.scope(&workspace.db) == second.scope(&workspace.db));
        assert!(first.types(&workspace.db) == second.types(&workspace.db));
        let canonical = read_static_package(&workspace.db, &second).unwrap();
        assert!(same_owners(&inputs, &canonical.shared_inputs()));
        assert!(canonical.retained_bytes() > canonical.retained_bytes_excluding_shared_inputs());
    }
}
