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
        source_key: pse_ids::ContentHash::from_bytes([1; 32]),
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
fn source_syntax_admission_refuses_partial_variants_and_wrong_operand_arity() {
    use pse_relations::generated::{
        enums::PredicateComparison,
        normalized::{equation_nodes as e, predicate_nodes as p},
    };
    use pse_schema::model::Cell;
    let registry = pse_schema::registry().unwrap();
    let operand = p::NormalizedPredicateNodesFieldValueCompareOperandsItem::from_expression(
        p::NormalizedPredicateNodesFieldValueCompareOperandsItemExpression { node_id: 9 },
    );
    let compare = p::NormalizedPredicateNodesFieldValueCompare {
        comparison: PredicateComparison::Eq,
        operands: vec![operand.clone(), operand],
    };
    let schema = p::schema().unwrap();
    let arrow_schema::DataType::Struct(children) =
        schema.field_with_name("value").unwrap().data_type()
    else {
        panic!("predicate struct")
    };
    let field = children
        .iter()
        .find(|field| field.name() == "compare")
        .unwrap();
    for arity in [0, 1, 2, 3] {
        let mut value = compare.clone().into_cell();
        let Cell::Struct(fields) = &mut value else {
            panic!("comparison struct")
        };
        let Cell::List(operands) = &mut fields[1] else {
            panic!("operand pair")
        };
        let first = operands[0].clone();
        *operands = vec![first; arity];
        assert_eq!(
            pse_relations::cells::array_from_cells(registry, field, &[value]).is_ok(),
            arity == 2
        );
    }
    let mut equation = e::NormalizedEquationNodesFieldValue::from_conditional(
        e::NormalizedEquationNodesFieldValueConditional {
            guard: 1,
            then: 2,
            otherwise: 3,
        },
    );
    equation.conditional = None;
    let mut builder = e::Builder::new().unwrap();
    assert!(
        builder
            .push(e::Row {
                source_id: pse_ids::SemanticId::NIL,
                equation_id: 4,
                value: equation
            })
            .is_err()
    );
    let mut predicate = p::NormalizedPredicateNodesFieldValue::from_null();
    predicate.compare = Some(compare);
    assert!(
        p::Builder::new()
            .unwrap()
            .push(p::Row {
                source_id: pse_ids::SemanticId::NIL,
                predicate_id: 4,
                value: predicate
            })
            .is_err()
    );
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

#[test]
fn template_domain_sources_require_only_the_selected_payload() {
    use bindings::{
        AuthoredTemplateDomainBindingsFieldSource as Source,
        AuthoredTemplateDomainBindingsFieldSourceDomain as Domain,
        AuthoredTemplateDomainBindingsFieldSourceParameter as Parameter,
    };
    use pse_relations::generated::authored::template_domain_bindings as bindings;
    let id = pse_ids::SemanticId::from_bytes([10; 16]);
    let row = |source| bindings::Row {
        template_id: id,
        name: "axis".into(),
        source,
    };
    let domain = Source::from_domain(Domain { domain_id: id });
    for source in [
        domain.clone(),
        Source::from_parameter(Parameter {
            name: "members".into(),
        }),
        Source::from_species(),
        Source::from_phase(),
        Source::from_phase_species(),
        Source::from_element(),
    ] {
        let mut builder = bindings::Builder::new().unwrap();
        builder.push(row(source.clone())).unwrap();
        let batch = builder.finish().unwrap().into_batch();
        assert_eq!(
            bindings::View::try_from_batch(&batch)
                .unwrap()
                .row(0)
                .unwrap()
                .source,
            source
        );
    }
    let mut missing = domain.clone();
    missing.domain = None;
    let mut overlap = domain.clone();
    overlap.parameter = Some(Parameter {
        name: "extra".into(),
    });
    let mut unit_with_payload = Source::from_phase();
    unit_with_payload.domain = domain.domain;
    for invalid in [missing, overlap, unit_with_payload] {
        assert!(
            bindings::Builder::new()
                .unwrap()
                .push(row(invalid))
                .is_err()
        );
    }
}

#[test]
fn expression_sources_have_one_complete_typed_owner() {
    use pse_relations::generated::{
        enums::{ExpressionFamily, ExpressionSyntax},
        extension_values::SourceSpan,
        normalized::expression_sources as sources,
    };
    use sources::{
        NormalizedExpressionSourcesFieldOwner as Owner,
        NormalizedExpressionSourcesFieldOwnerInstance as Instance,
        NormalizedExpressionSourcesFieldOwnerTemplate as Template,
    };
    let id = pse_ids::SemanticId::from_bytes([9; 16]);
    let row = |owner| sources::Row {
        source_id: id,
        family: ExpressionFamily::Template,
        source_relation_id: id,
        source_key: pse_ids::ContentHash::from_bytes([1; 32]),
        field_path: "expression".into(),
        owner,
        syntax: ExpressionSyntax::Expression,
        root_id: 0,
        derivation_id: id,
        source_span: SourceSpan {
            document_id: id,
            start: 0,
            end: 1,
        },
    };
    let template = Owner::from_template(Template { template_id: id });
    let instance = Owner::from_instance(Instance { instance_id: id });
    for owner in [template.clone(), instance.clone()] {
        let mut builder = sources::Builder::new().unwrap();
        builder.push(row(owner.clone())).unwrap();
        let batch = builder.finish().unwrap().into_batch();
        assert_eq!(
            sources::View::try_from_batch(&batch)
                .unwrap()
                .row(0)
                .unwrap()
                .owner,
            owner
        );
    }
    let mut overlap = template.clone();
    overlap.instance = instance.instance;
    let mut missing = template;
    missing.template = None;
    for owner in [overlap, missing] {
        assert!(sources::Builder::new().unwrap().push(row(owner)).is_err());
    }
}

#[test]
fn method_producers_keep_the_actual_symbol_and_one_complete_correspondence() {
    use method::{
        CompiledMethodRealizationsFieldRealization as Producer,
        CompiledMethodRealizationsFieldRealizationKernelOutput as Kernel,
        CompiledMethodRealizationsFieldRealizationTemplateSymbol as Template,
    };
    use pse_relations::generated::compiled::method_realizations as method;
    let id = pse_ids::SemanticId::from_bytes([17; 16]);
    let row = |realization| method::Row {
        requirement_id: id,
        method_id: id,
        output_symbol_id: id,
        realization,
        derivation_id: id,
    };
    let kernel = Producer::from_kernel_output(Kernel {
        kernel_binding_id: id,
        output_ordinal: 2,
    });
    let template = Producer::from_template_symbol(Template {
        template_instance_id: id,
    });
    for producer in [kernel.clone(), template.clone()] {
        let mut builder = method::Builder::new().unwrap();
        builder.push(row(producer.clone())).unwrap();
        let batch = builder.finish().unwrap().into_batch();
        let actual = method::View::try_from_batch(&batch)
            .unwrap()
            .row(0)
            .unwrap();
        assert_eq!(actual.output_symbol_id, id);
        assert_eq!(actual.realization, producer);
    }
    let mut overlapping = kernel;
    overlapping.template_symbol = template.template_symbol;
    assert!(
        method::Builder::new()
            .unwrap()
            .push(row(overlapping))
            .is_err()
    );
    for ordinal in [-1, i64::from(u16::MAX) + 1] {
        let invalid = Producer::from_kernel_output(Kernel {
            kernel_binding_id: id,
            output_ordinal: ordinal,
        });
        assert!(method::Builder::new().unwrap().push(row(invalid)).is_err());
    }
}

#[test]
fn equation_targets_require_one_complete_selected_comparison() {
    use equations::{
        InferredMathIndexedEquationsFieldConstraint as Constraint,
        InferredMathIndexedEquationsFieldConstraintRange as Range,
        InferredMathIndexedEquationsFieldConstraintSingle as Single,
    };
    use pse_relations::generated::inferred::math_indexed_equations as equations;
    use pse_schema::model::Cell;

    let registry = pse_schema::registry().unwrap();
    let schema = equations::schema().unwrap();
    let field = schema.field_with_name("constraint").unwrap();
    let equality = Constraint::from_eq(Single { node_id: 0 });
    let range = Constraint::from_range(Range {
        lower_node_id: 1,
        upper_node_id: 2,
    });
    for valid in [
        equality.clone(),
        Constraint::from_ge(Single { node_id: 3 }),
        Constraint::from_le(Single { node_id: 4 }),
        Constraint::from_definition(Single { node_id: 5 }),
        range.clone(),
    ] {
        let cells = [valid.clone().into_cell()];
        let array = pse_relations::cells::array_from_cells(registry, field, &cells).unwrap();
        assert_eq!(array.len(), 1);
    }
    let mut overlapping = equality.clone();
    overlapping.range = range.range;
    let mut absent = equality.clone();
    absent.single = None;
    let negative = Constraint::from_eq(Single { node_id: -1 });
    for invalid in [overlapping, absent, negative] {
        assert!(
            pse_relations::cells::array_from_cells(registry, field, &[invalid.into_cell()])
                .is_err()
        );
    }
    let partial_range = Cell::Struct(vec![
        Cell::Enum("range"),
        Cell::Null,
        Cell::Struct(vec![Cell::I64(1), Cell::Null]),
    ]);
    assert!(pse_relations::cells::array_from_cells(registry, field, &[partial_range]).is_err());
}

#[test]
fn dependency_coordinates_validate_the_selected_source_before_planning() {
    use dependencies::{
        ReferenceMethodDependenciesFieldIndexMapItem as Coordinate,
        ReferenceMethodDependenciesFieldIndexMapItemSource as Source,
        ReferenceMethodDependenciesFieldIndexMapItemSourceFixedMember as Member,
        ReferenceMethodDependenciesFieldIndexMapItemSourceSourceAxis as Axis,
    };
    use pse_relations::generated::{
        enums::{DomainKind, MethodDependencyTarget, MethodScopeMap},
        reference::method_dependencies as dependencies,
    };
    let id = pse_ids::SemanticId::from_bytes([23; 16]);
    let row = |source| dependencies::Row {
        method_id: id,
        ordinal: 0,
        target_kind: MethodDependencyTarget::Property,
        target_id: id,
        scope_map: MethodScopeMap::SameState,
        index_map: vec![Coordinate {
            domain_kind: DomainKind::Species,
            source,
        }],
    };
    let axis = Source::from_source_axis(Axis { position: 2 });
    let member = Source::from_fixed_member(Member { member_id: id });
    for source in [axis.clone(), member.clone(), Source::from_bound_domain()] {
        let mut builder = dependencies::Builder::new().unwrap();
        let expected = row(source);
        builder.push(expected.clone()).unwrap();
        let batch = builder.finish().unwrap().into_batch();
        assert_eq!(
            dependencies::View::try_from_batch(&batch)
                .unwrap()
                .row(0)
                .unwrap(),
            expected
        );
    }
    let mut overlap = axis.clone();
    overlap.fixed_member = member.fixed_member;
    let mut absent = axis;
    absent.source_axis = None;
    let mut stray = Source::from_bound_domain();
    stray.source_axis = Some(Axis { position: 0 });
    for source in [
        overlap,
        absent,
        stray,
        Source::from_source_axis(Axis { position: -1 }),
    ] {
        assert!(
            dependencies::Builder::new()
                .unwrap()
                .push(row(source))
                .is_err()
        );
    }
}

#[test]
fn method_declarations_require_one_complete_producer() {
    use methods::{
        ReferenceMethodSpecsFieldRealization as Producer,
        ReferenceMethodSpecsFieldRealizationEquationTemplate as Template,
        ReferenceMethodSpecsFieldRealizationKernel as Kernel,
    };
    use pse_relations::generated::reference::method_specs as methods;
    let registry = pse_schema::registry().unwrap();
    let schema = methods::schema().unwrap();
    let field = schema.field_with_name("realization").unwrap();
    let id = pse_ids::SemanticId::from_bytes([31; 16]);
    let template = Producer::from_equation_template(Template { template_id: id });
    let kernel = Producer::from_kernel(Kernel { kernel_id: id });
    for value in [template.clone(), kernel.clone()] {
        let mut builder = methods::Builder::new().unwrap();
        builder
            .push(methods::Row {
                method_id: id,
                family: pse_relations::generated::enums::MethodFamily::Custom,
                name: "producer".into(),
                version: "1".into(),
                provides: vec![],
                requires: vec![],
                parameter_kinds: vec![],
                realization: value.clone(),
                validity: vec![],
                doc: String::new(),
            })
            .unwrap();
        let batch = builder.finish().unwrap().into_batch();
        assert_eq!(
            methods::View::try_from_batch(&batch)
                .unwrap()
                .row(0)
                .unwrap()
                .realization,
            value
        );
    }
    let mut overlap = template.clone();
    overlap.kernel = kernel.kernel;
    let mut absent = template;
    absent.equation_template = None;
    for value in [overlap, absent] {
        assert!(value.selected().is_err());
        assert!(
            pse_relations::cells::array_from_cells(registry, field, &[value.into_cell()]).is_err()
        );
    }
}

#[test]
fn method_provisions_require_one_symbol_or_bounded_output_position() {
    use provisions::{
        ReferenceMethodProvisionsFieldOutput as Output,
        ReferenceMethodProvisionsFieldOutputKernelOutput as Kernel,
        ReferenceMethodProvisionsFieldOutputTemplateSymbol as Template,
    };
    use pse_relations::generated::reference::method_provisions as provisions;
    let registry = pse_schema::registry().unwrap();
    let schema = provisions::schema().unwrap();
    let field = schema.field_with_name("output").unwrap();
    let template = Output::from_template_symbol(Template {
        symbol_decl_id: pse_ids::SemanticId::from_bytes([32; 16]),
    });
    let kernel = Output::from_kernel_output(Kernel {
        ordinal: i64::from(u16::MAX),
    });
    for value in [template.clone(), kernel.clone()] {
        let id = pse_ids::SemanticId::from_bytes([33; 16]);
        let mut builder = provisions::Builder::new().unwrap();
        builder
            .push(provisions::Row {
                method_id: id,
                property_kind_id: id,
                output: value.clone(),
                quantity_type_id: id,
                natural_unit_id: id,
                indexed_by: vec![],
            })
            .unwrap();
        let batch = builder.finish().unwrap().into_batch();
        assert_eq!(
            provisions::View::try_from_batch(&batch)
                .unwrap()
                .row(0)
                .unwrap()
                .output,
            value
        );
    }
    let mut overlap = template.clone();
    overlap.kernel_output = kernel.kernel_output;
    let mut absent = template;
    absent.template_symbol = None;
    for value in [
        overlap,
        absent,
        Output::from_kernel_output(Kernel { ordinal: -1 }),
        Output::from_kernel_output(Kernel {
            ordinal: i64::from(u16::MAX) + 1,
        }),
    ] {
        assert!(
            pse_relations::cells::array_from_cells(registry, field, &[value.into_cell()]).is_err()
        );
    }
}
