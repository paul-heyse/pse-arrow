// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed construction and imported Arrow values obey the same selected-arm contract.
#![allow(clippy::unwrap_used, clippy::panic, reason = "contract assertions")]
use pse_relations::{
    generated::{
        enums::{ConfigCategory, ConfigValueKind},
        normalized::config_values as cv,
    },
    typed::CellCodec,
};

fn row(value: cv::NormalizedConfigValuesFieldValue) -> cv::Row {
    cv::Row {
        owner_id: pse_ids::SemanticId::NIL,
        category: ConfigCategory::Parameter,
        name: "selected".into(),
        value,
        source_relation_id: pse_ids::SemanticId::NIL,
        source_key: "source".into(),
        derivation_id: pse_ids::SemanticId::NIL,
    }
}

#[test]
fn generated_constructors_and_builders_preserve_wide_values_and_refuse_overlap() {
    use cv::{
        NormalizedConfigValuesFieldValue as Value,
        NormalizedConfigValuesFieldValueSelected as Selected,
    };
    let value =
        Value::from_unsigned(cv::NormalizedConfigValuesFieldValueUnsigned { value: u64::MAX });
    assert!(matches!(value.selected().unwrap(), Selected::Unsigned(arm) if arm.value == u64::MAX));
    let mut builder = cv::Builder::new().unwrap();
    builder.push(row(value.clone())).unwrap();
    let batch = builder.finish().unwrap().into_batch();
    assert_eq!(
        cv::View::try_from_batch(&batch)
            .unwrap()
            .row(0)
            .unwrap()
            .value,
        value
    );
    let mut overlapping = value.clone();
    overlapping.boolean = Some(cv::NormalizedConfigValuesFieldValueBoolean { value: true });
    assert!(overlapping.selected().is_err());
    assert!(cv::Builder::new().unwrap().push(row(overlapping)).is_err());
    let mut missing = value.clone();
    missing.unsigned = None;
    let mut mismatched = value;
    mismatched.kind = ConfigValueKind::Boolean;
    for invalid in [missing, mismatched] {
        assert!(invalid.selected().is_err());
        assert!(cv::Builder::new().unwrap().push(row(invalid)).is_err());
    }
}

#[test]
fn imported_nested_values_require_selection_and_honor_parent_masks() {
    use arrow_array::Array;
    use pse_schema::model::Cell;
    let registry = pse_schema::registry().unwrap();
    let schema = cv::schema().unwrap();
    let field = schema
        .field_with_name("value")
        .unwrap()
        .clone()
        .with_nullable(true);
    let valid = cv::NormalizedConfigValuesFieldValue::from_boolean(
        cv::NormalizedConfigValuesFieldValueBoolean { value: true },
    )
    .into_cell();
    let mut missing = valid.clone();
    let Cell::Struct(values) = &mut missing else {
        panic!("struct")
    };
    values[1] = Cell::Null;
    let mut unknown = valid.clone();
    let Cell::Struct(values) = &mut unknown else {
        panic!("struct")
    };
    values[0] = Cell::Enum("unknown");
    for (value, admitted) in [
        (valid, true),
        (missing, false),
        (unknown, false),
        (Cell::Null, true),
    ] {
        // The imported-column boundary validates both storage and declared meaning.
        let array = pse_relations::cells::array_from_cells(registry, &field, &[value]);
        assert_eq!(array.is_ok(), admitted);
        if let Ok(array) = array {
            assert_eq!(array.len(), 1);
        }
    }
}

#[test]
fn kernel_outcomes_separate_typed_success_from_shared_failure_payloads() {
    use outcomes::{
        RuntimeKernelEvaluationOutcomesFieldResult as Result,
        RuntimeKernelEvaluationOutcomesFieldResultFailure as Failure,
    };
    use pse_relations::generated::{
        enums::KernelFailure, extension_values::QuantityValue,
        runtime::kernel_evaluation_outcomes as outcomes,
    };
    let failure = Failure {
        reason_code: KernelFailure::DomainViolation,
        quantity_type_id: pse_ids::SemanticId::NIL,
        unit_id: pse_ids::SemanticId::NIL,
    };
    let success = Result::from_success(QuantityValue {
        value: 2.5,
        quantity_type_id: pse_ids::SemanticId::NIL,
        unit_id: pse_ids::SemanticId::NIL,
    });
    let row = |result| outcomes::Row {
        evaluation_id: pse_ids::SemanticId::NIL,
        row_ordinal: 0,
        output_ordinal: 0,
        result,
    };
    for result in [
        success.clone(),
        Result::from_domain_failure(failure.clone()),
        Result::from_missing_input(failure.clone()),
        Result::from_implementation_failure(failure.clone()),
    ] {
        let expected = row(result);
        let mut builder = outcomes::Builder::new().unwrap();
        builder.push(expected.clone()).unwrap();
        let batch = builder.finish().unwrap().into_batch();
        assert_eq!(
            outcomes::View::try_from_batch(&batch)
                .unwrap()
                .row(0)
                .unwrap(),
            expected
        );
    }
    let mut overlap = success.clone();
    overlap.failure = Some(failure);
    assert!(
        outcomes::Builder::new()
            .unwrap()
            .push(row(overlap))
            .is_err()
    );
    let mut absent = success;
    absent.success = None;
    assert!(outcomes::Builder::new().unwrap().push(row(absent)).is_err());
}

#[test]
fn nonlinear_incidence_cannot_carry_a_fabricated_linear_coefficient() {
    use inc::{
        CompiledIncidenceFieldDependence as Dependence,
        CompiledIncidenceFieldDependenceLinear as Linear,
    };
    use pse_relations::generated::compiled::incidence as inc;
    let row = |dependence| inc::Row {
        problem_id: pse_ids::SemanticId::NIL,
        equation_id: pse_ids::SemanticId::NIL,
        symbol_id: pse_ids::SemanticId::NIL,
        dependence,
    };
    let linear = Dependence::from_linear(Linear { coefficient: 0.0 });
    let nonlinear = Dependence::from_nonlinear();
    for value in [linear, nonlinear.clone()] {
        assert!(inc::Builder::new().unwrap().push(row(value)).is_ok());
    }
    let mut overlap = nonlinear;
    overlap.linear = Some(Linear { coefficient: 3.0 });
    assert!(inc::Builder::new().unwrap().push(row(overlap)).is_err());
}
