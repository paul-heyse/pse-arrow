// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical subjects, coordinate selection and participation are complete values.
use super::{RegistryBuilder, T, enumeration};
use crate::model::TaggedAlternative;

pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(builder, "ContributionSubjectKind", ["total", "energy", "momentum", "species", "element", "phase_species"]);
    enumeration(builder, "PhysicalCoordinateKind", ["fixed", "axis"]);
    enumeration(builder, "ContributionSign", ["positive", "negative"]);
    enumeration(
        builder,
        "ParticipationExclusionReason",
        ["family_mismatch", "subject_mismatch", "internal_transfer"],
    );
}
fn arm(name: &str, fields: Vec<T>) -> T {
    T::structure(fields).with_name(name).optional()
}
fn alternative(kind: &str, arms: Vec<T>) -> T {
    let tagged = TaggedAlternative::new(
        "kind",
        arms.iter()
            .map(|field| (field.name().to_owned(), field.name().to_owned())),
    );
    let mut fields = vec![T::enumeration(kind).with_name("kind")];
    fields.extend(arms);
    T::structure(fields).with_alternative(&tagged)
}

/// An actual entity or one declared axis; neither may be half-present.
pub(in super::super) fn coordinate() -> T {
    alternative(
        "PhysicalCoordinateKind",
        vec![
            arm("fixed", vec![T::id().with_name("entity_id")]),
            arm(
                "axis",
                vec![T::nonnegative(i64::from(u16::MAX)).with_name("position")],
            ),
        ],
    )
}
/// Coordinates awaiting the explicit selected law binding's subject kind.
pub(in super::super) fn coordinates() -> T {
    T::structure(vec![
        coordinate().with_name("member").optional(),
        coordinate().with_name("phase").optional(),
    ])
}
/// A contribution or law subject. Laws may narrow species/element totals by phase.
pub(in super::super) fn subject(law: bool) -> T {
    let mut arms = ["total", "energy", "momentum"]
        .into_iter()
        .map(|kind| arm(kind, vec![coordinate().with_name("phase").optional()]))
        .collect::<Vec<_>>();
    for kind in ["species", "element"] {
        let mut fields = vec![coordinate().with_name("member")];
        if law {
            fields.push(coordinate().with_name("phase").optional());
        }
        arms.push(arm(kind, fields));
    }
    arms.push(arm(
        "phase_species",
        vec![
            coordinate().with_name("member"),
            coordinate().with_name("phase"),
        ],
    ));
    alternative("ContributionSubjectKind", arms)
}
