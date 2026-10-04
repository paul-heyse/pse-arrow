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
fn named_types_prefix_enumeration_preserves_lexical_imports_and_physical_names() {
    let package = checked(VISIBLE);
    for owner in package.declarations.keys().copied() {
        assert_eq!(
            package.named_types(owner),
            exhaustive_named_types(&package, owner),
            "{}",
            package.qualified_name(owner).unwrap()
        );
    }
    let root = package.entry("a.Root").unwrap();
    let names = package.named_types(root);
    assert!(names.contains_key("lib.item"));
    assert!(names.contains_key("lib.ratio.x"));
    assert!(names.contains_key("inherited"));
    assert!(names.contains_key("physical.Length"));
    assert_eq!(
        names["Temperature"],
        package.types[&package.entry("a.Root.Temperature").unwrap()]
    );
    assert!(!names.contains_key("lib.unrelated"));
    assert!(!names.contains_key("library.item"));
    assert!(!names.contains_key("ab.hidden"));
    assert!(!names.contains_key("nested.value"));
    let deep = package.entry("a.Deep.identity").unwrap();
    assert!(package.named_types(deep).contains_key("nested.value"));
}

#[test]
fn named_types_prefix_enumeration_reads_current_types_without_a_retained_cache() {
    let mut package = checked(VISIBLE);
    let owner = package.entry("a.Root").unwrap();
    let local = package.entry("a.Root.local").unwrap();
    let original = package.types.remove(&local).unwrap();
    assert!(!package.named_types(owner).contains_key("local"));
    assert_eq!(
        package.named_types(owner),
        exhaustive_named_types(&package, owner)
    );
    package.types.insert(local, Type::Boolean);
    assert_eq!(package.named_types(owner)["local"], Type::Boolean);
    assert_eq!(
        package.named_types(owner),
        exhaustive_named_types(&package, owner)
    );
    package.types.insert(local, original);
}

#[test]
fn named_types_prefix_range_excludes_unrelated_packages_and_prefix_collisions() {
    let mut source = String::from(VISIBLE);
    // These admitted, typed names would all enter the previous global scan.
    // They remain available to their owners and invisible to package a.
    for index in 0..32 {
        source.push_str(&format!(
            "package unrelated_{index} {{ fn identity(x:Integer)->Integer=x; }}"
        ));
    }
    let package = checked(&source);
    let candidates = package
        .names_below("a")
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|name| name.starts_with("a.")));
    assert!(candidates.iter().all(|name| !name.starts_with("ab.")));
    assert!(candidates.len() < package.names.len());
    let owner = package.entry("a.Root").unwrap();
    assert_eq!(
        package.named_types(owner),
        exhaustive_named_types(&package, owner)
    );
    assert!(package.resolve(owner, "unrelated_0.identity").is_none());
}

#[test]
fn named_types_prefix_enumeration_preserves_lowered_function_resolution_boundary() {
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
    assert!(!contracts.lowered_functions.is_empty());
    // Actual scalar specialization grants synthetic call lookup, without adding
    // global qualified names or a parallel type environment for those functions.
    assert_eq!(contracts.names, package.names);
    for id in &contracts.lowered_functions {
        let name = format!("f_{}", id.as_id().to_hex());
        assert_eq!(contracts.resolve(owner, &name), Some(*id));
        assert!(!contracts.names.contains_key(&name));
    }
    assert_eq!(
        contracts.named_types(owner),
        exhaustive_named_types(&contracts, owner)
    );
}
