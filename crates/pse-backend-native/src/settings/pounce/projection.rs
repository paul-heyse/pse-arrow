// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked named FERAL choices use the existing settings binding's exact conversions.

use super::{FeralOrdering, FeralScaling, OrderingMethod, ScalingStrategy, ordering, scaling};
use pse_model::boundary_projection::{
    AdapterError, AdapterFailure, Direction, Field, ProjectionAdapter, ProjectionError,
    ProjectionGraph, ValidationError, adapter_source_path,
};

macro_rules! named_choice {
    ($marker:ident, $native:ty, $canonical:ty, $wire:literal, $native_path:path, $adapter_path:path, $binding:ident) => {
        /// Checked source binding of one library-owned named configuration choice.
        #[derive(Debug)]
        pub struct $marker;
        #[allow(
            unused_qualifications,
            reason = "the exact public library alias is checked for downstream generated code"
        )]
        const _: fn($native) -> $native_path = |value| value;
        #[allow(
            unused_qualifications,
            reason = "the exact public adapter alias is checked for downstream generated code"
        )]
        const _: fn($marker) -> $adapter_path = |value| value;
        impl ProjectionAdapter<$native> for $marker {
            const NAME: &'static str = $wire;
            const SOURCE: &'static str = file!();
            const VERSION: u32 = 1;
            const DIRECTION: Direction = Direction::Both;
            const RESERVED_TAGS: &'static [u32] = &[];
            const RESERVED_NAMES: &'static [&'static str] = &[];
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "value")];
            type Fields = ($canonical,);
            fn native_path() -> String {
                adapter_source_path::<Self>(stringify!($native_path))
            }
            fn adapter_path() -> String {
                adapter_source_path::<Self>(stringify!($adapter_path))
            }
            fn public_path() -> Option<String> {
                Some(format!(
                    "pse_model::boundary_projection::Adapted<{},{}>",
                    <Self as ProjectionAdapter<$native>>::native_path(),
                    <Self as ProjectionAdapter<$native>>::adapter_path()
                ))
            }
            fn fields(graph: &mut ProjectionGraph) -> Result<Vec<Field>, ProjectionError> {
                Ok(vec![Field::of::<$canonical>(
                    graph,
                    "value",
                    1,
                    Direction::Both,
                    vec![],
                )?])
            }
            fn validate(value: &$native, path: &str) -> Result<(), ValidationError> {
                $binding::named(value)
                    .map(|_| ())
                    .map_err(|reason| ValidationError::Rule {
                        path: path.into(),
                        rule: "foreign.named_setting",
                        reason: reason.into(),
                    })
            }
            fn try_fields(value: &$native) -> Result<Self::Fields, AdapterError> {
                $binding::named(value)
                    .map(|named| (named,))
                    .map_err(|reason| {
                        AdapterError::refused("", AdapterFailure::InvalidValue, reason)
                    })
            }
            fn from_fields((value,): Self::Fields) -> Result<$native, AdapterError> {
                Ok($binding::from_named(value))
            }
        }
    };
}
named_choice!(
    OrderingProjection,
    OrderingMethod,
    FeralOrdering,
    "FeralOrderingSetting",
    crate::settings::pounce::NativeOrdering,
    crate::settings::pounce::OrderingProjection,
    ordering
);
named_choice!(
    ScalingProjection,
    ScalingStrategy,
    FeralScaling,
    "FeralScalingSetting",
    crate::settings::pounce::NativeScaling,
    crate::settings::pounce::ScalingProjection,
    scaling
);

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::boundary_projection::{Adapted, CheckedProjectionRecord};

    #[test]
    fn actual_feral_named_choices_roundtrip_and_refuse_problem_data_variants() {
        type Ordering = Adapted<OrderingMethod, OrderingProjection>;
        type Scaling = Adapted<ScalingStrategy, ScalingProjection>;
        for value in FeralOrdering::ALL {
            let native = Ordering::from_projection_fields((value,)).unwrap();
            assert_eq!(native.try_into_projection_fields().unwrap(), (value,));
        }
        for value in FeralScaling::ALL {
            let native = Scaling::from_projection_fields((value,)).unwrap();
            assert_eq!(native.try_into_projection_fields().unwrap(), (value,));
        }
        assert!(
            Ordering::new(OrderingMethod::External(vec![1, 0]))
                .try_into_projection_fields()
                .is_err()
        );
        assert!(
            Scaling::new(ScalingStrategy::External(vec![1.0, 1.0]))
                .try_into_projection_fields()
                .is_err()
        );
        assert!(
            Scaling::new(ScalingStrategy::ExternalShared(std::sync::Arc::from([
                1.0, 1.0
            ])))
            .try_into_projection_fields()
            .is_err()
        );
    }
}
