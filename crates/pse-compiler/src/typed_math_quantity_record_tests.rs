// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiler-qualified replay controls for opaque quantity-owned records.
use pse_ids::SemanticId;
use pse_quantity::{
    BaseDimension, DimensionVector as Dimension, QuantityAdditionKind, QuantityError as Error,
    QuantityKind, QuantityRegistry, QuantityRegistryBuilder, QuantityType, QuantityTypeKey,
    ScaleKind,
    unit::{
        CanonicalConversionPlan, CanonicalConversionRecord, RepresentationConversionPlan,
        RepresentationConversionRecord, Unit,
    },
};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn unit(n: u8, symbol: &str, scale: f64, offset: f64) -> Unit {
    Unit {
        id: id(n).into(),
        symbol: symbol.into(),
        dimension: Dimension::base(BaseDimension::Temperature),
        scale_to_canonical: scale,
        offset_to_canonical: offset,
        is_affine: offset != 0.0,
        reference_state: None,
        definition: None,
    }
}
fn registry() -> QuantityRegistry {
    let mut b = QuantityRegistryBuilder::new();
    b.unit(unit(1, "K", 1.0, 0.0))
        .unit(unit(2, "degC", 1.0, 273.15))
        .unit(unit(3, "twiceK", 2.0, 0.0))
        .unit(unit(4, "hugeOrigin", 1.0, f64::MAX))
        .unit(unit(5, "twoRoundings", 1.0 + f64::EPSILON, -1.0))
        .unit(unit(6, "halfK", 0.5, 0.0))
        .kind(QuantityKind {
            id: id(20).into(),
            dimension: Dimension::base(BaseDimension::Temperature),
            extensive: false,
            addition_kind: QuantityAdditionKind::OriginSensitive,
            category: None,
            definition: None,
        });
    for (n, scale_kind) in [(30, ScaleKind::Point), (31, ScaleKind::Difference)] {
        b.quantity_type(QuantityType {
            id: id(n).into(),
            name: None,
            key: QuantityTypeKey {
                kind: id(20).into(),
                basis: None,
                reference_state: None,
                scale_kind,
                shape: vec![],
                subject_kind: None,
            },
            canonical_unit: id(1).into(),
            nominal_magnitude: None,
        });
    }
    b.build().unwrap()
}

// The test payload consists only of witnesses actually admitted just above;
// this models the private parent-to-child recipe channel, never user DTO authority.
#[allow(
    unsafe_code,
    reason = "controlled test fixture transfers actual admitted quantity records through the private scientific recipe boundary"
)]
fn qualified_record_fixture(
    value: &impl serde::Serialize,
) -> pse_ids::scientific_replay::QualifiedScientificRecipe {
    use pse_ids::scientific_replay::{QualifiedScientificRecipe, admitted_recipe_payload_hash};
    let payload = serde_json::to_vec(value).unwrap();
    let digest = admitted_recipe_payload_hash(&payload);
    // SAFETY: Only this module's actual successful admissions enter this
    // isolated immutable test channel, with their exact complete payload.
    unsafe {
        QualifiedScientificRecipe::from_admitted_process_message(
            payload,
            digest,
            pse_ids::ContentHash::from_bytes([91; 32]),
            pse_ids::ContentHash::from_bytes([92; 32]),
        )
    }
    .unwrap()
}

fn empty_receipt_stream() -> pse_quantity::resolved::receipts::ReceiptStream {
    pse_quantity::resolved::receipts::capture(|| Ok::<_, Error>(()))
        .unwrap()
        .1
}

fn record_boundaries() -> (
    pse_quantity::resolved::AdmittedContractBoundary,
    pse_quantity::resolved::AdmittedOutputBoundary,
) {
    use pse_quantity::resolved::{
        AdmittedContractBoundary, AdmittedOutputBoundary, ResolvedPhysicalContract,
    };
    let registry = registry();
    let source =
        ResolvedPhysicalContract::named(id(31).into(), pse_quantity::IndexSet::new(), &registry)
            .unwrap();
    let target =
        ResolvedPhysicalContract::named(id(30).into(), pse_quantity::IndexSet::new(), &registry)
            .unwrap();
    (
        AdmittedContractBoundary::prepare(&source, &source, &registry).unwrap(),
        AdmittedOutputBoundary::declared_contract(id(42), source, target).unwrap(),
    )
}

#[test]
fn direct_representation_record_requires_exact_qualified_session_and_preserves_frame() {
    use pse_quantity::resolved::receipts;
    let registry = registry();
    let plans = [
        RepresentationConversionPlan::registered(
            &registry,
            id(30).into(),
            id(2).into(),
            id(5).into(),
        )
        .unwrap(),
        RepresentationConversionPlan::registered(
            &registry,
            id(30).into(),
            id(5).into(),
            id(2).into(),
        )
        .unwrap(),
        RepresentationConversionPlan::registered(
            &registry,
            id(31).into(),
            id(2).into(),
            id(6).into(),
        )
        .unwrap(),
        RepresentationConversionPlan::registered(
            &registry,
            id(30).into(),
            id(1).into(),
            id(1).into(),
        )
        .unwrap(),
    ];
    let records = plans
        .iter()
        .map(RepresentationConversionRecord::capture)
        .collect::<Vec<_>>();
    for record in &records {
        assert!(record.restore().is_err());
    }
    let mut unknown = serde_json::to_value(&records[0]).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unexpected".into(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<RepresentationConversionRecord>(unknown).is_err());
    let stream = empty_receipt_stream();
    let recipe = qualified_record_fixture(&(&stream, &records));
    let decoded: Vec<RepresentationConversionRecord> =
        serde_json::from_slice(&serde_json::to_vec(&records).unwrap()).unwrap();
    assert_eq!(decoded, records);
    receipts::replay(&stream, recipe.records(), || -> Result<(), Error> {
        for (record, original) in decoded.iter().zip(&plans) {
            let restored = record.restore()?;
            assert_eq!(&restored, original);
            let mut expected =
                pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
            let mut actual =
                pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
            original.frame(&mut expected);
            restored.frame(&mut actual);
            assert_eq!(actual.finish_hash(), expected.finish_hash());
            for value in [-0.0, 0.0, 1.0 - f64::EPSILON, 298.15] {
                assert_eq!(
                    restored.apply(value)?.to_bits(),
                    original.apply(value)?.to_bits()
                );
            }
        }
        assert_eq!(
            decoded[3].restore()?.apply(-0.0)?.to_bits(),
            0.0_f64.to_bits()
        );
        Ok(())
    })
    .unwrap();
    for record in &decoded {
        assert!(record.restore().is_err());
    }
}

#[test]
fn direct_representation_record_refuses_altered_full_context_and_coefficients() {
    use pse_quantity::resolved::receipts;
    let plan = RepresentationConversionPlan::registered(
        &registry(),
        id(30).into(),
        id(2).into(),
        id(5).into(),
    )
    .unwrap();
    let record = RepresentationConversionRecord::capture(&plan);
    let stream = empty_receipt_stream();
    let recipe = qualified_record_fixture(&(&stream, &record));
    receipts::replay(&stream, recipe.records(), || -> Result<(), Error> {
        assert!(record.restore().is_ok());
        for key in ["quantity", "source", "target"] {
            let mut value = serde_json::to_value(&record).unwrap();
            value[key] = serde_json::to_value(id(99)).unwrap();
            assert!(
                serde_json::from_value::<RepresentationConversionRecord>(value)
                    .unwrap()
                    .restore()
                    .is_err()
            );
        }
        for key in ["kind", "basis", "reference_state", "subject_kind"] {
            let mut value = serde_json::to_value(&record).unwrap();
            value["context"][key] = serde_json::to_value(id(99)).unwrap();
            assert!(
                serde_json::from_value::<RepresentationConversionRecord>(value)
                    .unwrap()
                    .restore()
                    .is_err()
            );
        }
        let mut value = serde_json::to_value(&record).unwrap();
        value["context"]["shape"] = serde_json::json!([id(99)]);
        assert!(
            serde_json::from_value::<RepresentationConversionRecord>(value)
                .unwrap()
                .restore()
                .is_err()
        );
        let mut value = serde_json::to_value(&record).unwrap();
        value["context"]["scale_kind"] = serde_json::json!("difference");
        assert!(
            serde_json::from_value::<RepresentationConversionRecord>(value)
                .unwrap()
                .restore()
                .is_err()
        );
        for (key, bits) in [
            ("scale_bits", 2.0_f64.to_bits()),
            ("offset_bits", (-0.0_f64).to_bits()),
        ] {
            let mut value = serde_json::to_value(&record).unwrap();
            value[key] = serde_json::json!(bits);
            assert!(
                serde_json::from_value::<RepresentationConversionRecord>(value)
                    .unwrap()
                    .restore()
                    .is_err()
            );
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn qualified_quantity_record_roundtrip_preserves_boundary_frames_and_conversion_bits() {
    use pse_quantity::resolved::receipts::{self, ContractBoundaryRecord, OutputBoundaryRecord};
    let registry = registry();
    let (ordinary, output) = record_boundaries();
    let ordinary_record = ContractBoundaryRecord::capture(&ordinary);
    let output_record = OutputBoundaryRecord::capture(&output);
    let plans = [
        CanonicalConversionPlan::registered(&registry, id(30).into(), id(2).into()).unwrap(),
        CanonicalConversionPlan::registered(&registry, id(30).into(), id(5).into()).unwrap(),
        CanonicalConversionPlan::registered(&registry, id(31).into(), id(2).into()).unwrap(),
        CanonicalConversionPlan::registered(&registry, id(30).into(), id(1).into()).unwrap(),
        CanonicalConversionPlan::canonical(&registry, id(30).into()).unwrap(),
    ];
    let conversions = plans
        .iter()
        .map(CanonicalConversionRecord::capture)
        .collect::<Vec<_>>();
    let stream = empty_receipt_stream();
    let recipe =
        qualified_record_fixture(&(&stream, &ordinary_record, &output_record, &conversions));
    let ordinary_decoded: ContractBoundaryRecord =
        serde_json::from_slice(&serde_json::to_vec(&ordinary_record).unwrap()).unwrap();
    let output_decoded: OutputBoundaryRecord =
        serde_json::from_slice(&serde_json::to_vec(&output_record).unwrap()).unwrap();
    let conversions_decoded: Vec<CanonicalConversionRecord> =
        serde_json::from_slice(&serde_json::to_vec(&conversions).unwrap()).unwrap();
    receipts::replay(&stream, recipe.records(), || -> Result<(), Error> {
        let restored = ordinary_decoded.restore()?;
        assert_eq!(&restored, &ordinary);
        assert_eq!(restored.source(), ordinary.source());
        assert_eq!(restored.target(), ordinary.target());
        let mut expected = pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
        let mut actual = pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
        ordinary.frame(&mut expected);
        restored.frame(&mut actual);
        assert_eq!(actual.finish_hash(), expected.finish_hash());
        assert_eq!(
            restored.apply_contract(ordinary.source())?,
            ordinary.apply_contract(ordinary.source())?
        );
        assert!(restored.apply_contract(output.target()).is_err());
        let restored = output_decoded.restore()?;
        assert_eq!(&restored, &output);
        assert_eq!(restored.operation(), id(42));
        assert_eq!(restored.source(), output.source());
        assert_eq!(restored.target(), output.target());
        let mut expected = pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
        let mut actual = pse_ids::FramedHasher::new(pse_ids::Frame::MathConcreteReceiptRequestV1);
        output.frame(&mut expected);
        restored.frame(&mut actual);
        assert_eq!(actual.finish_hash(), expected.finish_hash());
        assert_eq!(
            restored.apply_contract(output.source())?,
            output.apply_contract(output.source())?
        );
        assert!(restored.apply_contract(output.target()).is_err());
        for (record, original) in conversions_decoded.iter().zip(&plans) {
            let restored = record.restore()?;
            assert_eq!(&restored, original);
            assert_eq!(restored.context(), original.context());
            assert_eq!(restored.source_unit(), original.source_unit());
            assert_eq!(restored.canonical_unit(), original.canonical_unit());
            assert_eq!(restored.scale().to_bits(), original.scale().to_bits());
            assert_eq!(restored.offset().to_bits(), original.offset().to_bits());
            for value in [-0.0, 0.0, 1.0 - f64::EPSILON, 298.15] {
                assert_eq!(restored.apply(value)?.bits(), original.apply(value)?.bits());
            }
            assert!(restored.apply(f64::INFINITY).is_err());
        }
        assert_eq!(
            conversions_decoded[4].restore()?.apply(-0.0)?.bits(),
            (-0.0_f64).to_bits()
        );
        assert_ne!(
            conversions_decoded[3].restore()?.apply(-0.0)?.bits(),
            (-0.0_f64).to_bits()
        );
        Ok(())
    })
    .unwrap();
    assert!(ordinary_decoded.restore().is_err());
    assert!(output_decoded.restore().is_err());
    assert!(conversions_decoded[0].restore().is_err());
}

#[test]
fn qualified_quantity_records_refuse_altered_records_under_original_authority() {
    use pse_quantity::resolved::receipts::{self, ContractBoundaryRecord, OutputBoundaryRecord};
    let (ordinary, output) = record_boundaries();
    let ordinary = ContractBoundaryRecord::capture(&ordinary);
    let output = OutputBoundaryRecord::capture(&output);
    let conversion = CanonicalConversionRecord::capture(
        &CanonicalConversionPlan::registered(&registry(), id(30).into(), id(2).into()).unwrap(),
    );
    let stream = empty_receipt_stream();
    let recipe = qualified_record_fixture(&(&stream, &ordinary, &output, &conversion));
    receipts::replay(&stream, recipe.records(), || -> Result<(), Error> {
        assert!(ordinary.restore().is_ok());
        assert!(output.restore().is_ok());
        assert!(conversion.restore().is_ok());
        let mut value = serde_json::to_value(&ordinary).unwrap();
        value["scale_bits"] = serde_json::json!((1.0 + f64::EPSILON).to_bits());
        assert!(
            serde_json::from_value::<ContractBoundaryRecord>(value)
                .unwrap()
                .restore()
                .is_err()
        );
        let mut value = serde_json::to_value(&ordinary).unwrap();
        value["source"]["scale_bits"] = serde_json::json!(2.0_f64.to_bits());
        assert!(
            serde_json::from_value::<ContractBoundaryRecord>(value)
                .unwrap()
                .restore()
                .is_err()
        );
        let mut value = serde_json::to_value(&output).unwrap();
        value["operation"] = serde_json::to_value(id(99)).unwrap();
        assert!(
            serde_json::from_value::<OutputBoundaryRecord>(value)
                .unwrap()
                .restore()
                .is_err()
        );
        let mut value = serde_json::to_value(&output).unwrap();
        value["source"] = value["target"].clone();
        assert!(
            serde_json::from_value::<OutputBoundaryRecord>(value)
                .unwrap()
                .restore()
                .is_err()
        );
        for key in ["quantity", "source", "target"] {
            let mut value = serde_json::to_value(&conversion).unwrap();
            value[key] = serde_json::to_value(id(99)).unwrap();
            assert!(
                serde_json::from_value::<CanonicalConversionRecord>(value)
                    .unwrap()
                    .restore()
                    .is_err()
            );
        }
        let mut value = serde_json::to_value(&conversion).unwrap();
        value["context"]["shape"] = serde_json::json!([id(99)]);
        assert!(
            serde_json::from_value::<CanonicalConversionRecord>(value)
                .unwrap()
                .restore()
                .is_err()
        );
        for (key, bits) in [
            ("scale_bits", 2.0_f64.to_bits()),
            ("offset_bits", (-0.0_f64).to_bits()),
        ] {
            let mut value = serde_json::to_value(&conversion).unwrap();
            value[key] = serde_json::json!(bits);
            assert!(
                serde_json::from_value::<CanonicalConversionRecord>(value)
                    .unwrap()
                    .restore()
                    .is_err()
            );
        }
        let mut value = serde_json::to_value(&conversion).unwrap();
        value["already_canonical"] = serde_json::Value::Bool(true);
        assert!(
            serde_json::from_value::<CanonicalConversionRecord>(value)
                .unwrap()
                .restore()
                .is_err()
        );
        Ok(())
    })
    .unwrap();
}
