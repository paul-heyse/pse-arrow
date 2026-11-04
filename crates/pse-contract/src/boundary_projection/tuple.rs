// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Named source fields retain existing tuple document and native alternative forms.

/// Declare a named native record while preserving its existing tuple JSON encoding.
/// Do not derive Serde or JsonSchema on the record: these implementations delegate
/// to the original field tuple, in its explicitly declared order.
#[macro_export]
macro_rules! projection_tuple_struct {
    ($(#[$meta:meta])* $vis:vis struct $name:ident {
        projection: { $($projection:tt)* };
        $($(#[$field_meta:meta])* $field_vis:vis $field:ident: $ty:ty => {
            tag: $tag:expr, direction: $direction:expr, rules: [$($rule:expr),* $(,)?] };)*
    }) => {
        $crate::projection_struct! {
            $(#[$meta])* $vis struct $name {
                projection: { $($projection)* };
                $($(#[$field_meta])* $field_vis $field: $ty => { tag: $tag, direction: $direction, rules: [$($rule),*] };)*
            }
        }
        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> {
                ::serde::Serialize::serialize(&($( &self.$field,)*), serializer)
            }
        }
        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
                let ($($field,)*) = <($($ty,)*) as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
                Ok(Self { $($field,)* })
            }
        }
        impl ::schemars::JsonSchema for $name {
            fn schema_name() -> ::std::borrow::Cow<'static, str> { stringify!($name).into() }
            fn json_schema(generator: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                <($($ty,)*) as ::schemars::JsonSchema>::json_schema(generator)
            }
        }
    };
}

/// Declare actual unit, tuple and named enum alternatives in one source field table.
/// Tuple constructors retain their payload order; every wire field has an explicit tag.
#[macro_export]
macro_rules! projection_tuple_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        projection: { name: $wire:expr, $(rust_path: $public:path,)? version: $version:expr, direction: $direction:expr,
            reserved_tags: [$($reserved:expr),* $(,)?], reserved_names: [$($reserved_name:expr),* $(,)?] };
        $($(#[$variant_meta:meta])* $variant:ident
            $([ $($(#[$tuple_meta:meta])* $tuple_field:ident: $tuple_ty:ty => {
                tag: $tuple_tag:expr, direction: $tuple_direction:expr, rules: [$($tuple_rule:expr),* $(,)?] $(, representation: $tuple_representation:expr)? };)+ ])?
            $({ $($(#[$field_meta:meta])* $field:ident: $ty:ty => {
                tag: $tag:expr, direction: $field_direction:expr, rules: [$($rule:expr),* $(,)?] $(, representation: $representation:expr)? };)* })?
            => $variant_tag:expr;
        )*
    }) => {
        $(#[$meta])* $vis enum $name {
            $($(#[$variant_meta])* $variant
                $(($($(#[$tuple_meta])* $tuple_ty,)+))?
                $({ $($(#[$field_meta])* $field: $ty,)* })?,
            )*
        }
        $(#[allow(unused_qualifications, reason = "the actual enum's explicit public alias is retained for downstream generated adapters")]
        const _: fn($name) -> $public = |value| value;)?
        impl $crate::boundary_projection::ProjectionType for $name {
            fn projection(graph: &mut $crate::boundary_projection::ProjectionGraph) -> ::core::result::Result<$crate::boundary_projection::TypeRef, $crate::boundary_projection::ProjectionError> {
                let root = graph.declare::<Self>($wire, file!(), $version, $direction, &[$($reserved),*], &[$($reserved_name),*], |graph| {
                    let _ = &graph;
                    Ok($crate::boundary_projection::Shape::Alternatives(vec![$(
                        $crate::boundary_projection::Alternative {
                            name: stringify!($variant).into(), tag: $variant_tag,
                            fields: vec![
                                $($($crate::boundary_projection::Field::of::<$tuple_ty>(graph,
                                    stringify!($tuple_field), $tuple_tag, $tuple_direction, vec![$($tuple_rule),*])? $(.representation($tuple_representation)?)?,)+)?
                                $($($crate::boundary_projection::Field::of::<$ty>(graph,
                                    stringify!($field), $tag, $field_direction, vec![$($rule),*])? $(.representation($representation)?)?,)*)?
                            ],
                        },
                    )*]))
                })?;
                $(graph.public_path::<Self>(&root, stringify!($public))?;)?
                Ok(root)
            }
            $( $crate::projection_public_alias_path!($public); )?
        }
        impl $crate::boundary_projection::ValidateProjection for $name {
            fn validate_at(&self, path: &str) -> ::core::result::Result<(), $crate::boundary_projection::ValidationError> {
                let _ = path;
                $(
                    $(#[allow(irrefutable_let_patterns, reason = "an actual enum can intentionally declare one tuple alternative")]
                    if let Self::$variant($($tuple_field,)+) = self {
                        let alternative = $crate::boundary_projection::field_path(path, stringify!($variant));
                        $($crate::boundary_projection::validate_field($tuple_field,
                            &$crate::boundary_projection::field_path(&alternative, stringify!($tuple_field)), &[$($tuple_rule),*])?;)+
                    })?
                    $(#[allow(irrefutable_let_patterns, reason = "an actual enum can intentionally declare one named alternative")]
                    if let Self::$variant { $($field,)* } = self {
                        let alternative = $crate::boundary_projection::field_path(path, stringify!($variant));
                        $($crate::boundary_projection::validate_field($field,
                            &$crate::boundary_projection::field_path(&alternative, stringify!($field)), &[$($rule),*])?;)*
                    })?
                )*
                Ok(())
            }
        }
        $($crate::projection_tuple_enum!(@access $name, $version, $variant, $variant_tag;
            $([$($tuple_field: $tuple_ty => $tuple_tag,)+])?
            $({$($field: $ty => $tag,)*})?
        );)*
    };
    (@access $name:ident, $version:expr, $variant:ident, $tag:expr;) => {
        $crate::projection_enum!(@access $name, $version, $variant, $tag;);
    };
    (@access $name:ident, $version:expr, $variant:ident, $tag:expr; {$($field:ident: $ty:ty => $field_tag:expr,)*}) => {
        $crate::projection_enum!(@access $name, $version, $variant, $tag; {$($field: $ty => $field_tag,)*} @local {});
    };
    (@access $name:ident, $version:expr, $variant:ident, $tag:expr; [$($field:ident: $ty:ty => $field_tag:expr,)+]) => {
        impl $crate::boundary_projection::ProjectionAlternative<$tag> for $name {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($field_tag, stringify!($field))),+];
            type Fields = ($($ty,)+);
            #[allow(unreachable_patterns, reason = "an actual enum can intentionally declare one tuple alternative")]
            fn into_projection_alternative(self) -> ::core::result::Result<Self::Fields, Self> {
                match self { Self::$variant($($field,)+) => Ok(($($field,)+)), other => Err(other) }
            }
            fn from_projection_alternative(($($field,)+): Self::Fields) -> Self { Self::$variant($($field,)+) }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::super::*;
    projection_tuple_enum! {
        #[derive(Debug, serde::Serialize)]
        enum ActualMixedBits {
            projection:{name:"ActualMixedBits",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            Tuple [bits: u32 => {tag:1,direction:Direction::Both,rules:[],representation:Scalar::F32Bits};] => 1;
            Named {bits: u64 => {tag:1,direction:Direction::Both,rules:[],representation:Scalar::F64Bits};} => 2;
        }
    }
    #[test]
    fn mixed_tuple_enum_bit_representations_keep_native_integer_payloads_exact() {
        let graph = ProjectionGraph::of::<ActualMixedBits>().unwrap();
        let Shape::Alternatives(variants) = &graph.definitions()["ActualMixedBits"].shape else { panic!("actual source alternatives") };
        assert_eq!(variants[0].fields[0].ty, TypeRef::Scalar(Scalar::F32Bits));
        assert_eq!(variants[1].fields[0].ty, TypeRef::Scalar(Scalar::F64Bits));
        let narrow = <ActualMixedBits as ProjectionAlternative<1>>::from_projection_alternative((u32::MAX,));
        assert_eq!(serde_json::to_vec(&narrow).unwrap(), br#"{"Tuple":4294967295}"#);
        assert_eq!(<ActualMixedBits as ProjectionAlternative<1>>::into_projection_alternative(narrow).unwrap(), (u32::MAX,));
        let wide = <ActualMixedBits as ProjectionAlternative<2>>::from_projection_alternative((u64::MAX,));
        assert_eq!(serde_json::to_vec(&wide).unwrap(), br#"{"Named":{"bits":18446744073709551615}}"#);
    }
    projection_tuple_enum! {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        enum ActualMixed {
            projection:{name:"ActualMixed",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
            Unit => 9;
            Pair [name: String => {tag:7,direction:Direction::Both,rules:[]};
                count: u32 => {tag:2,direction:Direction::Both,rules:[LocalRule::UnsignedRange {min:1,max:10}]};] => 3;
            Named { label: String => {tag:5,direction:Direction::Both,rules:[]};
                child: Box<ActualMixed> => {tag:1,direction:Direction::Both,rules:[]}; } => 12;
        }
    }
    #[test]
    fn mixed_source_alternatives_retain_actual_native_constructors_and_tagged_fields() {
        let pair = ActualMixed::Pair("a".into(), 2);
        assert_eq!(serde_json::to_string(&pair).unwrap(), r#"{"Pair":["a",2]}"#);
        let fields =
            <ActualMixed as ProjectionAlternative<3>>::into_projection_alternative(pair).unwrap();
        assert_eq!(fields, ("a".to_owned(), 2));
        let pair = <ActualMixed as ProjectionAlternative<3>>::from_projection_alternative(fields);
        let named = ActualMixed::Named {
            label: "b".into(),
            child: Box::new(pair),
        };
        named.validate_projection().unwrap();
        assert_eq!(
            serde_json::to_string(&named).unwrap(),
            r#"{"Named":{"label":"b","child":{"Pair":["a",2]}}}"#
        );
        let graph = ProjectionGraph::of::<ActualMixed>().unwrap();
        let Shape::Alternatives(variants) = &graph.definitions()["ActualMixed"].shape else {
            panic!("actual mixed enum")
        };
        assert_eq!(
            variants
                .iter()
                .map(|variant| variant.tag)
                .collect::<Vec<_>>(),
            vec![9, 3, 12]
        );
        assert_eq!(
            <ActualMixed as ProjectionAlternative<12>>::FIELD_LAYOUT,
            &[(5, "label"), (1, "child")]
        );
        assert!(
            ActualMixed::Pair("a".into(), 0)
                .validate_projection()
                .is_err()
        );
    }
}
