// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed native inquiry conversion; missing presence never becomes a default conclusion.
use super::*;
pub(super) fn target_to_pb(t: &ImportTarget) -> pb::NativeImportTarget {
    pb::NativeImportTarget {
        file: path(&t.file),
        byte_start: number(t.byte_start),
        byte_end: number(t.byte_end),
        source_sha256: t.source_sha256.clone(),
        ..Default::default()
    }
}
pub(super) fn target_from_pb(t: pb::NativeImportTarget) -> Result<ImportTarget> {
    if t.file.is_empty() || t.source_sha256.is_empty() || t.byte_start > t.byte_end {
        return Err(malformed("native operand guard"));
    }
    Ok(ImportTarget {
        file: from_path(t.file),
        byte_start: size(t.byte_start, "operand.start")?,
        byte_end: size(t.byte_end, "operand.end")?,
        source_sha256: t.source_sha256,
    })
}
pub(super) fn operation_to_pb(op: CompatibilityOperation) -> pb::NativeCompatibilityOperation {
    match op {
        CompatibilityOperation::Equal => pb::NativeCompatibilityOperation::TypeEqual,
        CompatibilityOperation::CouldUnify => pb::NativeCompatibilityOperation::TypeCouldUnify,
        CompatibilityOperation::CouldCoerce => pb::NativeCompatibilityOperation::TypeCouldCoerce,
    }
}
pub(super) fn operation_from_pb(
    op: buffa::EnumValue<pb::NativeCompatibilityOperation>,
) -> Result<CompatibilityOperation> {
    Ok(
        match op
            .as_known()
            .ok_or_else(|| malformed("compatibility.operation"))?
        {
            pb::NativeCompatibilityOperation::TypeEqual => CompatibilityOperation::Equal,
            pb::NativeCompatibilityOperation::TypeCouldUnify => CompatibilityOperation::CouldUnify,
            pb::NativeCompatibilityOperation::TypeCouldCoerce => {
                CompatibilityOperation::CouldCoerce
            }
        },
    )
}
pub(super) fn origin_to_pb(k: OriginKind) -> pb::NativeOriginKind {
    match k {
        OriginKind::Format => pb::NativeOriginKind::FormatOrigin,
        OriginKind::Derive => pb::NativeOriginKind::DeriveOrigin,
    }
}
pub(super) fn origin_from_pb(k: buffa::EnumValue<pb::NativeOriginKind>) -> Result<OriginKind> {
    Ok(
        match k.as_known().ok_or_else(|| malformed("origin.kind"))? {
            pb::NativeOriginKind::FormatOrigin => OriginKind::Format,
            pb::NativeOriginKind::DeriveOrigin => OriginKind::Derive,
        },
    )
}
fn target_kind(k: OriginTargetKind) -> pb::native_origin_observation::TargetKind {
    use pb::native_origin_observation::TargetKind as K;
    match k {
        OriginTargetKind::Declaration => K::Declaration,
        OriginTargetKind::Local => K::Local,
        OriginTargetKind::AssemblyOperand => K::AssemblyOperand,
        OriginTargetKind::OriginatingAdt => K::OriginatingAdt,
        OriginTargetKind::DeriveHelper => K::DeriveHelper,
        OriginTargetKind::GeneratedImpl => K::GeneratedImpl,
        OriginTargetKind::Unresolved => K::Unresolved,
    }
}
pub(super) fn to_pb(v: NativeInvestigationFacts) -> Result<pb::NativeInvestigationFacts> {
    Ok(pb::NativeInvestigationFacts {
        documentation_links: v
            .documentation_links
            .into_iter()
            .map(|link| pb::NativeDocumentationLinkObservation {
                written: link.written,
                owner: link.owner.map(native_location_to_pb).into(),
                target: link.target.map(native_location_to_pb).into(),
                namespace: link.namespace,
                limitations: link.limitations,
                ..Default::default()
            })
            .collect(),
        body: v
            .body
            .map(|b| pb::NativeBodyObservation {
                location: b.location.map(native_location_to_pb).into(),
                kind: b.kind,
                package_id: b.package_id,
                role: b.role,
                owner: b.owner.map(native_location_to_pb).into(),
                limitations: b.limitations,
                ..Default::default()
            })
            .into(),
        compatibility: v
            .compatibility
            .map(|c| pb::NativeCompatibilityObservation {
                operation: operation_to_pb(c.operation).into(),
                left: native_location_to_pb(c.left).into(),
                right: native_location_to_pb(c.right).into(),
                left_type: c.left_type,
                right_type: c.right_type,
                result: c.result,
                compatible_owner: c.compatible_owner,
                limitations: c.limitations,
                ..Default::default()
            })
            .into(),
        visible: v.visible,
        origins: v
            .origins
            .into_iter()
            .map(|o| pb::NativeOriginObservation {
                ordinal: number(o.ordinal),
                target_kind: target_kind(o.target_kind).into(),
                occurrence: o.occurrence.map(native_location_to_pb).into(),
                target: o.target.map(native_location_to_pb).into(),
                name: o.name,
                originating_adt: o.originating_adt.map(native_location_to_pb).into(),
                limitations: o.limitations,
                ..Default::default()
            })
            .collect(),
        limitations: v.limitations,
        ..Default::default()
    })
}
pub(super) fn from_pb(mut v: pb::NativeInvestigationFacts) -> Result<NativeInvestigationFacts> {
    Ok(NativeInvestigationFacts {
        documentation_links: v
            .documentation_links
            .into_iter()
            .map(|mut link| {
                Ok(DocumentationLinkObservation {
                    written: link.written,
                    owner: link.owner.take().map(native_location_from_pb).transpose()?,
                    target: link
                        .target
                        .take()
                        .map(native_location_from_pb)
                        .transpose()?,
                    namespace: link.namespace,
                    limitations: link.limitations,
                })
            })
            .collect::<Result<_>>()?,
        body: v
            .body
            .take()
            .map(|mut b| -> Result<_> {
                Ok(NativeBodyObservation {
                    location: b.location.take().map(native_location_from_pb).transpose()?,
                    kind: b.kind,
                    package_id: b.package_id,
                    role: b.role,
                    owner: b.owner.take().map(native_location_from_pb).transpose()?,
                    limitations: b.limitations,
                })
            })
            .transpose()?,
        compatibility: v
            .compatibility
            .take()
            .map(|mut c| -> Result<_> {
                Ok(CompatibilityObservation {
                    operation: operation_from_pb(c.operation)?,
                    left: native_location_from_pb(required(c.left.take(), "comparison.left")?)?,
                    right: native_location_from_pb(required(c.right.take(), "comparison.right")?)?,
                    left_type: c.left_type,
                    right_type: c.right_type,
                    result: c.result,
                    compatible_owner: c.compatible_owner,
                    limitations: c.limitations,
                })
            })
            .transpose()?,
        visible: v.visible,
        origins: v
            .origins
            .into_iter()
            .map(|mut o| {
                use pb::native_origin_observation::TargetKind as K;
                Ok(OriginObservation {
                    ordinal: size(o.ordinal, "origin.ordinal")?,
                    target_kind: match o
                        .target_kind
                        .as_known()
                        .ok_or_else(|| malformed("origin.target_kind"))?
                    {
                        K::Declaration => OriginTargetKind::Declaration,
                        K::Local => OriginTargetKind::Local,
                        K::AssemblyOperand => OriginTargetKind::AssemblyOperand,
                        K::OriginatingAdt => OriginTargetKind::OriginatingAdt,
                        K::DeriveHelper => OriginTargetKind::DeriveHelper,
                        K::GeneratedImpl => OriginTargetKind::GeneratedImpl,
                        K::Unresolved => OriginTargetKind::Unresolved,
                    },
                    occurrence: o
                        .occurrence
                        .take()
                        .map(native_location_from_pb)
                        .transpose()?,
                    target: o.target.take().map(native_location_from_pb).transpose()?,
                    name: o.name,
                    originating_adt: o
                        .originating_adt
                        .take()
                        .map(native_location_from_pb)
                        .transpose()?,
                    limitations: o.limitations,
                })
            })
            .collect::<Result<_>>()?,
        limitations: v.limitations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_native_inquiry_wire_preserves_unknown_false_and_plural_source_origins() {
        for visible in [None, Some(false), Some(true)] {
            let facts = NativeInvestigationFacts {
                visible,
                origins: vec![
                    OriginObservation {
                        ordinal: 0,
                        target_kind: OriginTargetKind::GeneratedImpl,
                        occurrence: None,
                        target: None,
                        name: Some("Clone".into()),
                        originating_adt: None,
                        limitations: vec!["no exact generated identity".into()],
                    },
                    OriginObservation {
                        ordinal: 1,
                        target_kind: OriginTargetKind::DeriveHelper,
                        occurrence: None,
                        target: None,
                        name: Some("helper".into()),
                        originating_adt: None,
                        limitations: vec!["plural nominations retained".into()],
                    },
                ],
                ..Default::default()
            };
            let expected = serde_json::to_value(&facts).unwrap();
            let actual = from_pb(to_pb(facts).unwrap()).unwrap();
            assert_eq!(serde_json::to_value(actual).unwrap(), expected);
        }
        let mut bad = pb::NativeInvestigationFacts::default();
        bad.origins.push(pb::NativeOriginObservation {
            target_kind: buffa::EnumValue::from(999),
            ..Default::default()
        });
        assert!(
            from_pb(bad).is_err(),
            "unknown target kind cannot become declaration"
        );
        let mut bad = pb::NativeInvestigationFacts::default();
        bad.compatibility = Some(pb::NativeCompatibilityObservation::default()).into();
        assert!(
            from_pb(bad).is_err(),
            "comparisons require both original operands"
        );
    }
}
