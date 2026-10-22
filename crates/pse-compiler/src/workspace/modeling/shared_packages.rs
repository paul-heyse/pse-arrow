// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independently retained canonical physical inputs for entity-local packages.
use super::*;

#[derive(Clone, Debug)]
pub(super) struct SharedOwner<T> {
    owner: Arc<T>,
}
impl<T: PartialEq> PartialEq for SharedOwner<T> {
    fn eq(&self, other: &Self) -> bool { Arc::ptr_eq(&self.owner, &other.owner) || self.owner == other.owner }
}
impl<T: Eq> Eq for SharedOwner<T> {}
impl<T> std::hash::Hash for SharedOwner<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Complete immutable values own equality; allocation capacity is not meaning.
        std::hash::Hash::hash(&size_of::<T>(), state);
    }
}
#[salsa::interned(heap_size=quantity_heap)]
pub(super) struct Quantities<'db> { value: SharedOwner<QuantityRegistry> }
fn quantity_heap((value,): &(SharedOwner<QuantityRegistry>,)) -> usize { value.owner.allocation_extent() }
#[salsa::interned(heap_size=preconditions_heap)]
pub(super) struct Preconditions<'db> { value: SharedOwner<PhysicalPreconditions> }
fn preconditions_heap((value,): &(SharedOwner<PhysicalPreconditions>,)) -> usize { value.owner.allocation_extent() }
#[salsa::interned(heap_size=scope_heap)]
pub(super) struct Scope<'db> { value: SharedOwner<PhysicalScope> }
fn scope_heap((value,): &(SharedOwner<PhysicalScope>,)) -> usize { pse_modeling::SharedPackageInputs::scope_retained_bytes(&value.owner) }
#[salsa::interned(heap_size=types_heap)]
pub(super) struct PhysicalTypes<'db> { value: SharedOwner<BTreeMap<String, pse_modeling::Type>> }
fn types_heap((value,): &(SharedOwner<BTreeMap<String, pse_modeling::Type>>,)) -> usize { pse_modeling::SharedPackageInputs::physical_types_retained_bytes(&value.owner) }

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StaticFunctionPackage(Arc<CheckedPackage>);
impl Eq for StaticFunctionPackage {}
impl std::hash::Hash for StaticFunctionPackage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.0.declarations().next().map(|row| row.declaration_id), state);
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
fn package_heap((package, _, _, _, _): &(StaticFunctionPackage, Quantities<'_>, Preconditions<'_>, Scope<'_>, PhysicalTypes<'_>)) -> usize {
    package.0.retained_bytes_excluding_shared_inputs()
}
fn same_owners(a: &pse_modeling::SharedPackageInputs, b: &pse_modeling::SharedPackageInputs) -> bool {
    Arc::ptr_eq(&a.quantities, &b.quantities) && Arc::ptr_eq(&a.preconditions, &b.preconditions)
        && Arc::ptr_eq(&a.scope, &b.scope) && Arc::ptr_eq(&a.physical_types, &b.physical_types)
}

pub(super) fn intern_static_package<'db>(db: &'db dyn CompilerDb, mut package: Arc<CheckedPackage>) -> std::result::Result<StaticFunctionPackageKey<'db>, pse_modeling::ModelingError> {
    let inputs = package.shared_inputs();
    let quantities = Quantities::new(db, SharedOwner { owner: inputs.quantities.clone() });
    let preconditions = Preconditions::new(db, SharedOwner { owner: inputs.preconditions.clone() });
    let scope = Scope::new(db, SharedOwner { owner: inputs.scope.clone() });
    let types = PhysicalTypes::new(db, SharedOwner { owner: inputs.physical_types.clone() });
    let canonical = pse_modeling::SharedPackageInputs {
        quantities: quantities.value(db).owner.clone(), preconditions: preconditions.value(db).owner.clone(),
        scope: scope.value(db).owner.clone(), physical_types: types.value(db).owner.clone(),
    };
    if !same_owners(&inputs, &canonical) {
        Arc::make_mut(&mut package).canonicalize_shared_inputs(&canonical)?;
    }
    Ok(StaticFunctionPackageKey::new(db, StaticFunctionPackage(package), quantities, preconditions, scope, types))
}

pub(super) fn read_static_package(db: &dyn CompilerDb, key: &StaticFunctionPackageKey<'_>) -> std::result::Result<Arc<CheckedPackage>, pse_modeling::ModelingError> {
    // Read the owner ingredients in the consuming query too. A retained local
    // package cannot outlive its independently charged physical inputs unnoticed.
    let owners = pse_modeling::SharedPackageInputs {
        quantities: key.quantities(db).value(db).owner.clone(),
        preconditions: key.preconditions(db).value(db).owner.clone(),
        scope: key.scope(db).value(db).owner.clone(), physical_types: key.types(db).value(db).owner.clone(),
    };
    let package = key.package(db).0.clone();
    if !same_owners(&package.shared_inputs(), &owners) {
        return Err(pse_modeling::ModelingError::Contract { declaration: SemanticId::NIL, message: "static package lost its canonical shared input owner".into() });
    }
    Ok(package)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_packages_share_canonical_physical_allocations() {
        let mut workspace = CompilerWorkspace::new(
            crate::authored_transfer_tests::context(), WorkspaceLimits::default(),
        ).unwrap();
        let rows = pse_authoring::language::parse(
            "package p {def Root {var x:Scalar;}}", SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named, pse_authoring::ParseBudget::default(),
        ).unwrap();
        let revision = workspace.publish_modeling(rows, PhysicalScope::default()).unwrap();
        let original = revision.checked().clone();
        let mut separate = original.as_ref().clone();
        let inputs = original.shared_inputs();
        let separate_inputs = pse_modeling::SharedPackageInputs {
            quantities: Arc::new(inputs.quantities.as_ref().clone()),
            preconditions: Arc::new(inputs.preconditions.as_ref().clone()),
            scope: Arc::new(inputs.scope.as_ref().clone()),
            physical_types: Arc::new(inputs.physical_types.as_ref().clone()),
        };
        separate.canonicalize_shared_inputs(&separate_inputs).unwrap();
        assert!(!same_owners(&inputs, &separate.shared_inputs()));
        let first = intern_static_package(&workspace.db, original).unwrap();
        let second = intern_static_package(&workspace.db, Arc::new(separate)).unwrap();
        assert!(first == second);
        let canonical = read_static_package(&workspace.db, &second).unwrap();
        assert!(same_owners(&inputs, &canonical.shared_inputs()));
        assert!(canonical.retained_bytes() > canonical.retained_bytes_excluding_shared_inputs());
    }
}
