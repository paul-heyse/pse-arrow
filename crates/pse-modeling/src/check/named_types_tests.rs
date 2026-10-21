// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;

// Previous exhaustive behavior is a test-only semantic reference.
fn exhaustive_named_types(
    package: &CheckedPackage,
    owner: DeclarationId,
) -> BTreeMap<String, Type> {
    let mut names = package
            .names
            .iter()
            // Untyped declarations cannot contribute to this environment. Avoid
            // resolving their names again for every declaration in the closure.
            .filter(|(_, id)| package.types.contains_key(*id))
            .filter(|(name, id)| package.resolve(owner, name) == Some(**id))
            .filter_map(|(name, id)| package.types.get(id).map(|ty| (name.clone(), ty.clone())))
            .collect::<BTreeMap<_, _>>();
    let mut chain = Vec::new();
    let mut node = Some(owner);
    while let Some(id) = node {
        chain.push(id);
        node = package.declarations[&id].parent_id;
    }
    // ADR-0123 Outcome 6: a package whose manifest depends on the declaring package
    // sees the physical names unqualified and as `<package>.<Name>`.
    if let Some(row) = chain.last().and_then(|id| package.declarations.get(id))
        && package.scope.sees(row.document_id)
    {
        for (name, value) in package.quantities.physical_names() {
            let ty = match value {
                pse_quantity::PhysicalName::QuantityType(id) => {
                    Type::Quantity(pse_quantity::scheme::Scheme::Concrete(id))
                }
                pse_quantity::PhysicalName::ReferenceState(_) => Type::ReferenceState,
            };
            if let Some(prefix) = &package.scope.package {
                names.insert(format!("{prefix}.{name}"), ty.clone());
            }
            names.insert(name.into(), ty);
        }
    }
    for owner in chain.into_iter().rev() {
        // A nominal coordinate slot is addressed relative to every visible lexical
        // owner exactly as ordinary declaration lookup addresses it.
        if let Some(prefix) = package
            .names
            .iter()
            .find_map(|(name, id)| (*id == owner).then(|| format!("{name}.")))
        {
            for (name, id) in &package.names {
                if let Some(relative) = name.strip_prefix(&prefix)
                    && package.resolve(owner, relative) == Some(*id)
                    && let Some(ty) = package.types.get(id)
                {
                    names.insert(relative.to_owned(), ty.clone());
                }
            }
        }
        for id in package.children.get(&owner).into_iter().flatten() {
            let row = &package.declarations[id];
            if let Some(import) = &row.value.import {
                let alias = import.alias.as_deref().unwrap_or(&row.name);
                let prefix = format!("{}.", row.name);
                for (name, id) in &package.names {
                    if let Some(tail) = name.strip_prefix(&prefix)
                        && let Some(ty) = package.types.get(id)
                    {
                        names.insert(format!("{alias}.{tail}"), ty.clone());
                    }
                }
            }
        }
        for (name, id) in package.members.get(&owner).into_iter().flatten() {
            if let Some(ty) = package.types.get(id) {
                names.insert(name.clone(), ty.clone());
            }
        }
        for id in package.children.get(&owner).into_iter().flatten() {
            if let Some(ty) = package.types.get(id) {
                names.insert(package.declarations[id].name.clone(), ty.clone());
            }
        }
    }
    names
}

fn checked(source: &str) -> CheckedPackage {
    let (quantities, _) = crate::kernel_types::physical();
    let preconditions =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    let scope = crate::PhysicalScope {
        package: Some("physical".into()),
        documents: None,
    };
    check(
        &crate::kernel_types::source(source),
        &TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &quantities,
            preconditions: &preconditions,
            scope: &scope,
        },
    )
    .unwrap()
}

#[test]
fn typed_child_under_expression_leaf_is_refused_by_structural_admission() {
    let mut rows = crate::kernel_types::source(
        "package p {fn law(x:Scalar)->Scalar=x; def D {var x:Scalar;} }",
    );
    let leaf = rows
        .iter()
        .find(|row| row.name == "law")
        .unwrap()
        .declaration_id;
    let child = rows.iter_mut().find(|row| row.name == "x").unwrap();
    child.parent_id = Some(leaf);
    let child_id = child.declaration_id;
    let (quantities, _) = crate::kernel_types::physical();
    let preconditions =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    let scope = crate::PhysicalScope {
        package: Some("physical".into()),
        documents: None,
    };
    let error = check(
        &rows,
        &TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &quantities,
            preconditions: &preconditions,
            scope: &scope,
        },
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("parent declaration cannot own nested members"),
        "{error}"
    );
    assert!(error.to_string().contains(&child_id.to_string()), "{error}");
}

const VISIBLE: &str = r#"
package library {
 entity kind item {}
 interface Base { let inherited:Scalar=2; }
 coordinate map ratio(x:Length) { slot x=x/1{m}; }
 fn twice(x:Scalar)->Scalar=x*2;
}
package lib { fn unrelated(x:Scalar)->Scalar=x; }
package ab { entity kind hidden {} fn value(x:Integer)->Integer=x; }
package a {
 use library @"1.0.0" as lib;
 def Root:lib.Base { param Temperature:Scalar=1; var local:Integer; }
 def Deep {
  use ab @"1.0.0" as nested;
  fn identity(x:Integer)->Integer=nested.value(x);
 }
}
"#;

#[test]
fn lazy_lookup_visits_do_not_enumerate_unrelated_package_names() {
    use crate::TypeLookup;
    for extra in [0, 32] {
        let mut source = String::from(VISIBLE);
        for index in 0..extra {
            source.push_str(&format!(
                "package unrelated_{index} {{fn identity(x:Integer)->Integer=x;}}"
            ));
        }
        let package = checked(&source);
        let owner = package.entry("a.Root").unwrap();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("one indexed lexical lookup");
        {
            let _entered = operation.context().enter();
            assert_eq!(
                package.named_types(owner).get("local"),
                Some(&Type::Integer)
            );
        }
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let visits = snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == "modeling.lexical_lookup_visits")
            .unwrap();
        assert_eq!(
            visits.cumulative,
            pse_diagnostics::observation::Value::Count(1)
        );
        assert_eq!(package.named_types(owner).get("unrelated_0.identity"), None);
        assert_eq!(package.named_types(owner).get("ab.hidden"), None);
    }
}

#[test]
fn lazy_named_types_preserve_lowered_function_resolution_boundary() {
    let package = checked(
        "package a { fn twice(x:Scalar)->Scalar=x*2; def Root { var x:Scalar; eq value:twice(x)==2; } }",
    );
    let owner = package.entry("a.Root").unwrap();
    let model = crate::specialize(
        &package,
        owner,
        crate::InstanceId::from_bytes([7; 16]),
        &crate::Bindings::default(),
        crate::Limits::default(),
    )
    .unwrap();
    let contracts = model.function_contracts(&package);
    assert!(!model.functions.is_empty());
    // Actual scalar specialization grants synthetic call lookup, while the
    // immutable source inventories and lexical types retain their original owner.
    assert!(std::ptr::eq(contracts.package(), &package));
    for (name, function) in &model.functions {
        let id =
            DeclarationId::from(SemanticId::parse_hex(name.strip_prefix("f_").unwrap()).unwrap());
        assert_eq!(contracts.resolve(owner, name.as_str()), Some(id));
        assert_eq!(
            contracts.resolve_segments(owner, &dsl::Path::single(name.as_str()).segments),
            Some(id)
        );
        assert!(std::ptr::eq(contracts.function(id).unwrap(), function));
        assert!(!package.names.contains_key(name.as_str()));
    }
    use crate::TypeLookup;
    let names = contracts.named_types(owner);
    for (name, ty) in exhaustive_named_types(&package, owner) {
        assert_eq!(names.get(&name), Some(&ty));
    }
    for name in model.functions.keys() {
        assert_eq!(names.get(name), None);
    }
}

#[test]
fn finite_function_admission_reads_the_current_owner_after_earlier_use() {
    let package = checked(
        "package a { fn twice(x:Scalar)->Scalar=x*2; def Root { var x:Scalar; eq value:twice(x)==2; } }",
    );
    let owner = package.entry("a.Root").unwrap();
    let complete = crate::specialize(
        &package,
        owner,
        crate::InstanceId::from_bytes([7; 16]),
        &crate::Bindings::default(),
        crate::Limits::default(),
    )
    .unwrap();
    let (name, function) = complete.functions.iter().next().unwrap();
    let (name, function) = (name.clone(), function.clone());
    let id = DeclarationId::from(SemanticId::parse_hex(name.strip_prefix("f_").unwrap()).unwrap());
    let mut current = crate::SpecializedModel::default();
    assert!(
        current
            .function_contracts(&package)
            .resolve(owner, &name)
            .is_none()
    );
    current.functions.insert(name.clone(), function.clone());
    let contracts = current.function_contracts(&package);
    assert_eq!(contracts.resolve(owner, name.as_str()), Some(id));
    assert_eq!(
        contracts.resolve_segments(owner, &dsl::Path::single(name.as_str()).segments),
        Some(id)
    );
    let mut call = function.body.clone().unwrap();
    call.kind = dsl::ExprKind::NamedCall {
        name: dsl::Path::single(name),
        args: vec![dsl::Expr {
            kind: dsl::ExprKind::Number(dsl::Number {
                value: 2.,
                exact_integer: Some(2),
                unit: None,
            }),
            span: Default::default(),
        }],
    };
    assert_eq!(
        crate::expression::infer(
            &call,
            &BTreeMap::new(),
            &contracts,
            &package.context(),
            owner,
            Some(&function.result)
        )
        .unwrap(),
        function.result
    );
}

#[test]
fn reconstruction_result_view_is_local_and_does_not_change_function_owner() {
    let package = checked(
        "package a {fn law(x:Scalar)->Scalar=x; def Root {var x:Scalar; eq value:law(x)==2;}}",
    );
    let owner = package.entry("a.Root").unwrap();
    let mut model = crate::specialize(
        &package,
        owner,
        crate::InstanceId::from_bytes([7; 16]),
        &crate::Bindings::default(),
        crate::Limits::default(),
    )
    .unwrap();
    let map = DeclarationId::from(SemanticId::from_bytes([2; 16]));
    let reconstruction = DeclarationId::from(SemanticId::from_bytes([3; 16]));
    let (name, function) = model.functions.iter_mut().next().unwrap();
    let name = name.clone();
    let scheme = function.result.quantity_scheme().unwrap().clone();
    function.result = Type::RefinedQuantity {
        quantity: scheme.clone(),
        refinement: crate::PhysicalRefinement::ReducedLaw {
            map,
            reconstruction,
        },
    };
    let nominal = function.result.clone();
    let id = DeclarationId::from(SemanticId::parse_hex(name.strip_prefix("f_").unwrap()).unwrap());
    let operation = crate::PhysicalOperation::Reconstruction {
        map,
        reconstruction,
        reference: owner,
    };
    let contracts = model.function_contracts(&package);
    let local = contracts.for_reconstruction(Some(&operation));
    let mut call = dsl::parse_expr("law(2)").unwrap();
    let dsl::ExprKind::NamedCall { name, .. } = &mut call.kind else {
        panic!("call")
    };
    *name = dsl::Path::single(format!("f_{}", id.as_id().to_hex()));
    assert_eq!(
        crate::expression::infer(
            &call,
            &BTreeMap::new(),
            &local,
            &package.context(),
            owner,
            None
        )
        .unwrap(),
        Type::Quantity(scheme)
    );
    assert_eq!(
        crate::expression::infer(
            &call,
            &BTreeMap::new(),
            &contracts,
            &package.context(),
            owner,
            None
        )
        .unwrap(),
        nominal
    );
    assert_eq!(contracts.function(id).unwrap().result, nominal);
}

#[test]
fn lazy_named_types_match_complete_enumeration_and_explicit_precedence() {
    use crate::TypeLookup;
    let package = checked(VISIBLE);
    for owner in package.declarations.keys().copied() {
        let lazy = package.named_types(owner);
        for (name, expected) in exhaustive_named_types(&package, owner) {
            assert_eq!(lazy.get(&name), Some(&expected), "{owner}: {name}");
        }
        assert_eq!(lazy.get("definitely_absent"), None);
    }
    let root = package.entry("a.Root").unwrap();
    let names = package.named_types(root);
    assert_eq!(names.get("local"), Some(&Type::Integer));
    assert_eq!(
        names.get("Temperature"),
        Some(&package.types[&package.entry("a.Root.Temperature").unwrap()])
    );
    assert_eq!(
        names.get("physical.Temperature"),
        package.physical_types.get("Temperature")
    );
    assert!(matches!(names.get("lib.item"), Some(Type::Entity(_))));
    assert!(names.get("lib.ratio.x").is_some());
    assert!(names.get("inherited").is_some());
    assert!(names.get("physical.Length").is_some());
    for absent in ["library.item", "ab.hidden", "nested.value", "lib.unrelated"] {
        assert_eq!(names.get(absent), None, "{absent}");
    }
    let deep = package.entry("a.Deep.identity").unwrap();
    assert!(matches!(
        package.named_types(deep).get("nested.value"),
        Some(Type::Function { .. })
    ));
}

#[test]
fn lazy_same_owner_reads_current_types_extent_and_reader_refinements() {
    use crate::TypeLookup;
    let mut package = checked("package p {entity kind item {} def Root {var value:Integer;}}");
    let owner = package.entry("p.Root").unwrap();
    let value = package.entry("p.Root.value").unwrap();
    let kind = package.entry("p.item").unwrap();
    assert_eq!(
        package.named_types(owner).get("value"),
        Some(&Type::Integer)
    );
    package.types.remove(&value);
    assert_eq!(package.named_types(owner).get("value"), None);
    package.types.insert(value, Type::Boolean);
    assert_eq!(
        package.named_types(owner).get("value"),
        Some(&Type::Boolean)
    );
    assert_eq!(
        package.named_types(owner).get("item"),
        Some(&Type::Entity(kind))
    );
    assert_eq!(
        package.named_types(owner).with_extents(false).get("item"),
        Some(&Type::Set(Box::new(Type::Entity(kind))))
    );
    let base = package.named_types(owner);
    let refined = BTreeMap::from([("value".into(), Type::Text)]);
    let frame = crate::TypeScope::owned(base).with_upper(&refined);
    assert_eq!(frame.get("value"), Some(&Type::Text));
    assert_eq!(
        package.named_types(owner).get("value"),
        Some(&Type::Boolean)
    );
}

#[test]
fn extent_views_borrow_one_current_inventory_projection() {
    use crate::TypeLookup;
    let mut package = checked("package p {entity kind item {} def Root {var item:Integer;}}");
    let owner = package.entry("p.Root").unwrap();
    let kind = package.entry("p.item").unwrap();
    let retained = package.retained_bytes();
    {
        let first = package.named_types(owner).with_extents(false);
        let second = package.named_types(owner).with_extents(true);
        assert_eq!(first.get("item"), Some(&Type::Integer));
        assert_eq!(
            second.get("item"),
            Some(&Type::Set(Box::new(Type::Entity(kind))))
        );
        assert!(std::ptr::eq(
            first.get("p.item").unwrap(),
            second.get("item").unwrap()
        ));
        assert_eq!(
            first.get("physical.Length"),
            package.physical_types.get("Length")
        );
        assert_eq!(second.get("physical.Length"), None);
    }
    assert_eq!(package.retained_bytes(), retained);
    let projection: *const Type = package.kinds.extent_type(&kind).unwrap();
    let mut replacement = package.kinds[&kind].clone();
    replacement.keys.push("changed_schema".into());
    package.kinds.insert(kind, replacement);
    let current_projection: *const Type = package.kinds.extent_type(&kind).unwrap();
    assert_eq!(current_projection, projection);
    assert_eq!(package.kinds[&kind].keys, ["changed_schema"]);
    assert_eq!(
        package.named_types(owner).with_extents(true).get("item"),
        Some(&Type::Set(Box::new(Type::Entity(kind))))
    );
}

#[test]
fn extent_inventory_mutations_update_current_lookup_and_retained_storage() {
    use crate::TypeLookup;
    let mut package = checked("package p {entity kind item {} def Root {}}");
    let owner = package.entry("p.Root").unwrap();
    let kind = package.entry("p.item").unwrap();
    let retained = package.retained_bytes();
    let projection_bytes = package.kinds.extent_bytes();
    let record = package.kinds.remove(&kind).unwrap();
    assert_eq!(package.kinds.extent_type(&kind), None);
    assert_eq!(
        package.named_types(owner).with_extents(true).get("item"),
        None
    );
    assert_eq!(
        package.named_types(owner).with_extents(false).get("item"),
        Some(&Type::Entity(kind))
    );
    assert_eq!(
        projection_bytes,
        size_of::<(DeclarationId, Type)>() + 64 + size_of::<Type>()
    );
    assert_eq!(package.kinds.extent_bytes(), 0);
    assert_eq!(
        retained - package.retained_bytes(),
        projection_bytes + size_of::<(DeclarationId, crate::entity::Kind)>() + 64
    );
    package.kinds.insert(kind, record);
    assert_eq!(package.retained_bytes(), retained);
    assert_eq!(
        package.named_types(owner).with_extents(true).get("item"),
        Some(&Type::Set(Box::new(Type::Entity(kind))))
    );
}

#[test]
fn selected_extent_inventory_retains_only_its_kind_closure() {
    use crate::TypeLookup;
    let package = checked("package p {entity kind item {} entity kind other {}}");
    let kind = package.entry("p.item").unwrap();
    let other = package.entry("p.other").unwrap();
    let selected = package.select(kind).unwrap();
    assert_eq!(selected.kinds.keys().copied().collect::<Vec<_>>(), [kind]);
    assert_eq!(
        selected.kinds.extents.keys().copied().collect::<Vec<_>>(),
        [kind]
    );
    assert!(!selected.kinds.contains_key(&other));
    assert_eq!(
        selected.kinds.extent_bytes() * 2,
        package.kinds.extent_bytes()
    );
    assert!(selected.retained_bytes() < package.retained_bytes());
    let view = selected.named_types(kind).with_extents(true);
    assert_eq!(
        view.get("p.item"),
        Some(&Type::Set(Box::new(Type::Entity(kind))))
    );
    assert_eq!(view.get("p.other"), None);
}

#[test]
fn lazy_lookup_keeps_nested_shadows_and_quoted_atomic_names_distinct() {
    use crate::TypeLookup;
    let package = checked(
        r#"package p {var outer:Integer; def D {var outer:Boolean; var "a.b":Integer; def Child {var leaf:Text;}}}"#,
    );
    let owner = package.entry("p.D").unwrap();
    assert_eq!(
        package.named_types(owner).get("outer"),
        Some(&Type::Boolean)
    );
    assert_eq!(
        package.named_types(owner).get("p.outer"),
        Some(&Type::Integer)
    );
    assert_eq!(
        package.named_types(owner).get("Child.leaf"),
        Some(&Type::Text)
    );
    let path = dsl::Path::single("a.b");
    assert_eq!(
        package.resolve_segments(owner, &path.segments),
        package.entry("p.D.a.b")
    );
    let split = dsl::Path {
        segments: vec![
            dsl::PathSegment {
                name: "a".into(),
                indices: vec![],
            },
            dsl::PathSegment {
                name: "b".into(),
                indices: vec![],
            },
        ],
    };
    assert_eq!(package.resolve_segments(owner, &split.segments), None);
}

#[test]
fn indexed_import_lookup_preserves_nested_alias_shadowing_and_refuses_conflicts() {
    use crate::TypeLookup;
    let prelude = "package left {fn identity(x:Integer)->Integer=x;} package right {fn identity(x:Boolean)->Boolean=x;} ";
    let package = checked(&format!(
        "{prelude} package p {{use left @\"1.0.0\" as shared;def D {{use right @\"1.0.0\" as shared;fn use_it(x:Boolean)->Boolean=shared.identity(x);}}}}"
    ));
    let owner = package.entry("p.D").unwrap();
    assert_eq!(
        package.named_types(owner).get("shared.identity"),
        package.types.get(&package.entry("right.identity").unwrap())
    );
    assert!(
        matches!(package.named_types(owner).get("shared.identity"), Some(Type::Function { result, .. }) if **result == Type::Boolean)
    );
    let (quantities, _) = crate::kernel_types::physical();
    let preconditions =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    let scope = crate::PhysicalScope::default();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: &quantities,
        preconditions: &preconditions,
        scope: &scope,
    };
    for (imports, expected) in [
        (
            "use left @\"1.0.0\" as shared;use right @\"1.0.0\" as shared;",
            "import alias conflicts with a local name",
        ),
        (
            "use left @\"1.0.0\" as shared;def shared {}",
            "import alias conflicts with a local name",
        ),
    ] {
        let rows = crate::kernel_types::source(&format!("{prelude} package p {{{imports}}}"));
        let error = check(&rows, &context).unwrap_err().to_string();
        assert!(error.contains(expected), "{expected}: {error}");
    }
    let repeated = format!(
        "{prelude} package p {{use left @\"1.0.0\" as shared;use left @\"1.0.0\" as shared;}}"
    );
    assert!(
        crate::kernel_types::try_source(&repeated)
            .unwrap_err()
            .to_string()
            .contains("unique declaration identity")
    );
}
