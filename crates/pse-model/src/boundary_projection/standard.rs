// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact standard-library and low-level nominal leaves of native requests.

use super::*;

const DURATION_SECONDS: (u32, &str) = (1, "secs");
const DURATION_NANOSECONDS: (u32, &str) = (2, "nanos");
const NANOS_PER_SECOND: u32 = 1_000_000_000;

impl ProjectionType for std::time::Duration {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let root = graph.declare::<Self>(
            "NonnegativeDuration",
            file!(),
            1,
            Direction::Both,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![
                    Field::of::<u64>(
                        graph,
                        DURATION_SECONDS.1,
                        DURATION_SECONDS.0,
                        Direction::Both,
                        vec![],
                    )?,
                    Field::of::<u32>(
                        graph,
                        DURATION_NANOSECONDS.1,
                        DURATION_NANOSECONDS.0,
                        Direction::Both,
                        vec![LocalRule::UnsignedRange {
                            min: 0,
                            max: u64::from(NANOS_PER_SECOND - 1),
                        }],
                    )?,
                ]))
            },
        )?;
        graph.duration_owner(&root)?;
        Ok(root)
    }
}
impl ProjectionGraph {
    /// Called only by the actual Duration projection; no record-name/shape inference.
    pub(super) fn duration_owner(&mut self, root: &TypeRef) -> Result<(), ProjectionError> {
        let TypeRef::Named(name) = root else {
            return Err(ProjectionError::invalid(
                "duration",
                "duration owner is not named",
            ));
        };
        let owner = self
            .definitions
            .get_mut(name)
            .ok_or_else(|| ProjectionError::invalid(name, "duration owner is unresolved"))?;
        if owner.rust_type != std::any::type_name::<std::time::Duration>() {
            return Err(ProjectionError::invalid(
                name,
                "duration hook must identify the actual standard Duration owner",
            ));
        }
        if !matches!(&owner.shape, Shape::Record(fields) if fields.len() == 2
            && fields[0].name == DURATION_SECONDS.1 && fields[0].tag == DURATION_SECONDS.0
            && fields[0].ty == TypeRef::Scalar(Scalar::U64)
            && fields[1].name == DURATION_NANOSECONDS.1 && fields[1].tag == DURATION_NANOSECONDS.0
            && fields[1].ty == TypeRef::Scalar(Scalar::U32)
            && fields[1].rules == [LocalRule::UnsignedRange { min: 0, max: u64::from(NANOS_PER_SECOND - 1) }])
        {
            return Err(ProjectionError::invalid(
                name,
                "Duration components differ from their typed source declaration",
            ));
        }
        owner.native_container = Some(NativeContainer::Duration {
            seconds_field: DURATION_SECONDS.1,
            nanoseconds_field: DURATION_NANOSECONDS.1,
        });
        Ok(())
    }
}
impl ProjectionRecord for std::time::Duration {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[DURATION_SECONDS, DURATION_NANOSECONDS];
    type Fields = (u64, u32);
    fn into_projection_fields(self) -> Self::Fields {
        (self.as_secs(), self.subsec_nanos())
    }
    fn from_projection_fields((secs, nanos): Self::Fields) -> Result<Self, AdapterError> {
        // Duration::new normalizes an out-of-domain nanos component, and may then
        // overflow secs. Boundary data must already have its exact canonical shape.
        if nanos >= NANOS_PER_SECOND {
            return Err(AdapterError::refused(
                "nanos",
                AdapterFailure::InvalidValue,
                "duration nanoseconds must be below one second",
            ));
        }
        Ok(Self::new(secs, nanos))
    }
}
impl ValidateProjection for std::time::Duration {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
        Ok(())
    }
    fn projection_duration(&self) -> Option<Self> {
        Some(*self)
    }
}

macro_rules! hash_role_projection {
    ($($ty:path => $name:literal),* $(,)?) => { $(
        impl ProjectionType for $ty {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                graph.declare::<Self>($name, "crates/pse-ids/src/roles.rs", 1, Direction::Both, &[], &[], |graph|
                    Ok(Shape::Record(vec![Field::of::<pse_ids::ContentHash>(graph, "id", 1,
                        Direction::Both, vec![LocalRule::Required])?])))
            }
        }
        impl ProjectionRecord for $ty {
            const VERSION: u32 = 1;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "id")];
            type Fields = (pse_ids::ContentHash,);
            fn into_projection_fields(self) -> Self::Fields { (self.as_id(),) }
            fn from_projection_fields((id,): Self::Fields) -> Result<Self, AdapterError> { Ok(Self::from_id(id)) }
        }
        impl ValidateProjection for $ty {
            const NEEDS_RECURSIVE_VALIDATION: bool = false;
            fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
        }
    )* };
}
hash_role_projection!(
    pse_ids::roles::SourceRevisionHash => "SourceRevisionHash",
    pse_ids::roles::BindingHash => "BindingHash",
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection_struct;
    use std::time::Duration;

    #[test]
    fn positive_duration_rule_preserves_general_zero_and_full_native_duration_domain() {
        projection_struct! {
            struct PositiveWindow {
                projection:{name:"PositiveWindow",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
                value: Option<Duration> => {tag:1,direction:Direction::Both,rules:[LocalRule::PositiveDuration]};
            }
        }
        let graph = ProjectionGraph::of::<PositiveWindow>().unwrap();
        assert!(matches!(
            graph.definitions()["NonnegativeDuration"].native_container,
            Some(NativeContainer::Duration {
                seconds_field: "secs",
                nanoseconds_field: "nanos"
            })
        ));
        Duration::ZERO.validate_projection().unwrap();
        validate_field(&Duration::ZERO, "general", &[]).unwrap();
        PositiveWindow { value: None }
            .validate_projection()
            .unwrap();
        assert!(
            PositiveWindow {
                value: Some(Duration::ZERO)
            }
            .validate_projection()
            .is_err()
        );
        for value in [
            Duration::from_nanos(1),
            Duration::from_secs(1),
            Duration::MAX,
        ] {
            PositiveWindow { value: Some(value) }
                .validate_projection()
                .unwrap();
            assert_eq!(
                Duration::from_projection_fields(value.into_projection_fields()).unwrap(),
                value
            );
        }
        assert!(
            Duration::from_projection_fields((0, NANOS_PER_SECOND)).is_err(),
            "the original canonical subsecond constraint remains independent of positivity"
        );
    }

    #[test]
    fn positive_duration_rule_refuses_unrelated_record_and_impostor_metadata() {
        projection_struct! {
            struct SimilarComponents {
                projection:{name:"SimilarDurationComponents",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
                secs:u64 => {tag:1,direction:Direction::Both,rules:[]};
                nanos:u32 => {tag:2,direction:Direction::Both,rules:[]};
            }
        }
        projection_struct! {
            struct UnsupportedDuration {
                projection:{name:"UnsupportedDuration",version:1,direction:Direction::Both,reserved_tags:[],reserved_names:[]};
                value:SimilarComponents => {tag:1,direction:Direction::Both,rules:[LocalRule::PositiveDuration]};
            }
        }
        let mut graph = ProjectionGraph::of::<SimilarComponents>().unwrap();
        let root = SimilarComponents::projection(&mut graph).unwrap();
        assert!(graph.duration_owner(&root).is_err());
        assert!(ProjectionGraph::of::<UnsupportedDuration>().is_err());
        let mut graph = ProjectionGraph::default();
        let root = graph
            .declare::<Duration>(
                "NonnegativeDuration",
                file!(),
                1,
                Direction::Both,
                &[],
                &[],
                |_| Ok(Shape::Record(vec![])),
            )
            .unwrap();
        assert!(graph.duration_owner(&root).is_err());
    }
}
