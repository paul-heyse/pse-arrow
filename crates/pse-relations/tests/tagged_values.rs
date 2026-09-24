// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed construction and imported Arrow values obey the same selected-arm contract.
#![allow(clippy::unwrap_used, clippy::panic, reason = "contract assertions")]
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
        assert!(rejects(row(invalid)));
    }
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
        assert!(rejects(row(source)));
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
            pse_relations::testing::array_from_literals(
                registry,
                field,
                &[pse_relations::testing::literal_value(field, &value).unwrap()]
            )
            .is_err()
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
            pse_relations::testing::array_from_literals(
                registry,
                field,
                &[pse_relations::testing::literal_value(field, &value).unwrap()]
            )
            .is_err()
        );
    }
}

// Invalid semantic values can be encoded; they cannot leave the finished builder.
fn rejects<T: pse_relations::columnar::RelationRow>(row: T) -> bool {
    let mut builder = T::builder(pse_schema::registry().unwrap(), 1).unwrap();
    T::push(&mut builder, row)
        .and_then(|()| T::finish(builder))
        .is_err()
}
