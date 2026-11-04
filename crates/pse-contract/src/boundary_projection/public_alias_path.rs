// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit-alias hook for recursive generic arguments. The source declaration
//! supplies the actual public alias before its recursive graph definition exists.

/// Supply the actual owner's explicit public alias during recursive projection.
/// This hook is emitted only when the original declaration supplies `rust_path`.
#[macro_export]
macro_rules! projection_public_alias_path {
    ($public:path) => {
        fn projection_rust_path(
            _: &mut $crate::boundary_projection::ProjectionGraph,
        ) -> ::core::result::Result<String, $crate::boundary_projection::ProjectionError> {
            #[allow(
                unused_qualifications,
                reason = "the explicit public alias is checked against its actual native owner"
            )]
            let _identity: fn(Self) -> $public = |value| value;
            Ok($crate::boundary_projection::adapter_source_path::<Self>(
                stringify!($public),
            ))
        }
    };
}

#[cfg(test)]
mod tests {
    use super::super::*;

    /// Recursion here deliberately reaches a generic pair before this owner's
    /// graph definition exists. The source alias, rather than type_name fallback,
    /// is the information sufficient to name that exact pair's public Rust path.
    pub(crate) enum RecursiveAliasFixture {
        Leaf,
        Function(Vec<(String, RecursiveAliasFixture)>),
    }
    impl ProjectionType for RecursiveAliasFixture {
        fn projection(
            graph: &mut ProjectionGraph,
        ) -> ::core::result::Result<TypeRef, ProjectionError> {
            let root = graph.declare::<Self>(
                "RecursiveAliasFixture",
                file!(),
                1,
                Direction::Output,
                &[],
                &[],
                |graph| {
                    Ok(Shape::Alternatives(vec![
                        Alternative {
                            name: "Leaf".into(),
                            tag: 1,
                            fields: vec![],
                        },
                        Alternative {
                            name: "Function".into(),
                            tag: 2,
                            fields: vec![Field::of::<Vec<(String, Self)>>(
                                graph,
                                "arguments",
                                1,
                                Direction::Output,
                                vec![],
                            )?],
                        },
                    ]))
                },
            )?;
            graph.public_path::<Self>(
                &root,
                "crate::boundary_projection::public_alias_path::tests::RecursiveAliasFixture",
            )?;
            Ok(root)
        }
        projection_public_alias_path!(
            crate::boundary_projection::public_alias_path::tests::RecursiveAliasFixture
        );
    }

    #[test]
    fn explicit_public_alias_resolves_recursive_pair_before_outer_owner_finishes() {
        let graph = ProjectionGraph::of::<RecursiveAliasFixture>().unwrap();
        let pair = &graph.definitions()["Tuple2OfStringAndRecursiveAliasFixture"];
        assert_eq!(
            pair.public_rust_path.as_deref(),
            Some(
                "(String,pse_contract::boundary_projection::public_alias_path::tests::RecursiveAliasFixture)"
            )
        );
        assert_eq!(
            pair.rust_type,
            core::any::type_name::<(String, RecursiveAliasFixture)>()
        );
        // Keep the original constructors exercised independently of graph traversal.
        let value =
            RecursiveAliasFixture::Function(vec![("x".into(), RecursiveAliasFixture::Leaf)]);
        let RecursiveAliasFixture::Function(arguments) = value else {
            panic!("actual function alternative")
        };
        assert_eq!(arguments[0].0, "x");
    }
}
