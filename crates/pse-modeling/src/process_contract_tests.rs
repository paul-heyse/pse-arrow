// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original process state correspondence and explicit composition responsibilities.
use crate::{
    Bindings, InstanceId, Limits, ModelingError, PhysicalScope, SpecializedModel, TypeContext,
    check, kernel_types, specialize,
};

fn checked(text: &str) -> Result<crate::CheckedPackage, ModelingError> {
    let (registry, _) = kernel_types::physical();
    let preconditions =
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap();
    check(
        &kernel_types::try_source(text)?,
        &TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &registry,
            preconditions: &preconditions,
            scope: &PhysicalScope::default(),
        },
    )
}
fn run(text: &str) -> Result<SpecializedModel, ModelingError> {
    let package = checked(text)?;
    specialize(
        &package,
        package.entry("p.Root").unwrap(),
        InstanceId::from_bytes([7; 16]),
        &Bindings::default(),
        Limits::default(),
    )
}

#[test]
fn process_contract_unannotated_children_retain_nominal_connection_contracts() {
    let text = "package p {def Unit {var x:Scalar;port inlet:Scalar=x;annotation connectivity inlet(1,0);port outlet:Scalar=x;annotation connectivity outlet(0,1);} def Root {child a=Unit();child b=Unit();connect a.outlet->b.inlet;}}";
    let model = run(text).unwrap();
    assert_eq!(model.connections.len(), 1);
    let error = checked(&text.replace("b.inlet", "missing.inlet")).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unknown contextual declaration reference")
    );
    let error = checked(&text.replace("b.inlet", "b.missing")).unwrap_err();
    assert!(error.to_string().contains("member"));
}

#[test]
fn process_contract_ftpz_and_fphz_keep_explicit_reference_species_and_derived_observations() {
    for (thermal_type, thermal_slot, thermal_tolerance) in [
        ("Temperature", "temperature", "1e-6{K}"),
        ("MolarEnthalpy", "enthalpy", "1e-6{J/mol}"),
    ] {
        let text = format!(
            r#"package p {{
         entity kind species {{}}entity species a {{}}entity species b {{}}
         set all:Set<species>={{a,b}};set independent:Set<species>={{a}};
         def Side(supplied:Boolean) {{
          var F:Flow;var P:Pressure;var thermal:{thermal_type};var z[j in all]:Scalar;
          let diagnostic:Scalar=sum(j in all | z[j]);
          state s supplied(supplied) {{
           coordinate flow=F;coordinate pressure=P;coordinate {thermal_slot}=thermal;
           coordinate fraction[j in independent]=z[j];
           reconstruct reference:z[b]==1-z[a] tolerance 1e-8;
           transport flow=F tolerance 1e-8{{mol/s}};
           transport pressure=P tolerance 1e-5{{Pa}};
           transport {thermal_slot}=thermal tolerance {thermal_tolerance};
           transport fraction[j in all]=z[j] tolerance 1e-8;
           transport diagnostic=diagnostic tolerance 1e-8;
          }}
          material port port=s;annotation connectivity port(1,1);
         }}
         def Root {{child source:Side=Side(supplied=true);child receiver:Side=Side(supplied=false);
          connect stream:source.port->receiver.port;}}
        }}"#
        );
        let model = run(&text).unwrap();
        let connection = model.connections.values().next().unwrap();
        assert_eq!(connection.bindings.len(), 4);
        assert_eq!(connection.rows.len(), 4);
        assert_eq!(model.equations.len(), 5);
        assert_eq!(
            model
                .equations
                .iter()
                .filter(|row| row.lineage.path.contains("receiver.s"))
                .count(),
            1
        );
        assert!(
            !model
                .equations
                .iter()
                .any(|row| row.lineage.path.contains("source.s"))
        );
        for specification in model.state_specifications.values() {
            assert_eq!(specification.coordinates.len(), 4);
            assert_eq!(
                specification
                    .coordinates
                    .keys()
                    .filter(|key| key.name == "fraction")
                    .count(),
                1
            );
            assert_eq!(
                specification
                    .transports
                    .keys()
                    .filter(|key| key.name == "fraction")
                    .count(),
                2
            );
            assert!(
                specification
                    .transports
                    .keys()
                    .any(|key| key.name == "diagnostic")
            );
            assert!(
                !specification
                    .coordinates
                    .keys()
                    .any(|key| key.name == "diagnostic")
            );
            let pse_authoring::dsl::EquationKind::Relation { lhs, .. } =
                &specification.reconstructions[0].0.equation.kind
            else {
                panic!("reference reconstruction")
            };
            let dependent = specialize::symbol_reference(lhs).unwrap();
            assert!(!model.ports.values().any(|port| port.symbol == dependent));
        }
    }
}

#[test]
fn process_contract_observation_assesses_evaluated_original_delta_terms() {
    for (outgoing, expected) in [(1.95, true), (1.5, false)] {
        let text = format!(
            "package p {{def Root {{param incoming:DeltaTemperature=2{{K}};param outgoing:DeltaTemperature={outgoing}{{K}};accumulate difference:DeltaTemperature observation tolerance 0.1{{K}};contribute difference role positive=incoming;contribute difference role negative=outgoing;}}}}"
        );
        let package = checked(&text).unwrap();
        let model = run(&text).unwrap();
        let closure = model.closures.values().next().unwrap();
        assert_eq!(
            closure.mode,
            pse_model::generated::enums::ModelingAccumulatorMode::Observation
        );
        assert!(model.equations.is_empty());
        let (registry, _) = kernel_types::physical();
        let preconditions = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        let scope = PhysicalScope::default();
        let context = TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &registry,
            preconditions: &preconditions,
            scope: &scope,
        };
        let env = model
            .symbols
            .iter()
            .filter_map(|(id, symbol)| {
                symbol
                    .initial
                    .clone()
                    .map(|value| (specialize::symbol_name(*id), value))
            })
            .collect();
        let at = package.entry("p.Root").unwrap();
        let mut evaluator = specialize::value::Evaluator {
            selections: None,
            package: &package,
            physical: &context,
            at,
            env: &env,
            limit: 100,
            stack: Vec::new(),
            reader: crate::provenance::Reader::Production(at),
        };
        let magnitudes = closure
            .terms
            .iter()
            .map(|term| {
                let value = evaluator
                    .expr(&term.expression, Some(&closure.ty), 0)
                    .unwrap();
                (term.id, value.scalar(at).unwrap())
            })
            .collect();
        let assessment = model
            .assess_closures(&magnitudes, &std::collections::BTreeSet::from([closure.id]))
            .unwrap();
        assert_eq!(assessment[0].satisfied, Some(expected));
        assert!((assessment[0].net - (2.0 - outgoing)).abs() < 1e-12);
        let mut incomplete = magnitudes;
        incomplete.remove(&closure.terms[0].id);
        assert!(
            model
                .assess_closures(&incomplete, &std::collections::BTreeSet::from([closure.id]))
                .is_err()
        );
    }
}

#[test]
fn process_contract_fixed_parameter_coordinates_keep_identity_while_original_energy_agreement_is_assessed()
 {
    for (received, expected) in [(1.95, true), (1.5, false)] {
        let text = format!(
            r#"package p {{
         def Side(supplied:Boolean,energy_input:EnergyTransferRate) {{
          param F:Flow=1{{mol/s}};param P:Pressure=100000{{Pa}};param T:Temperature=300{{K}};
          param H:EnergyTransferRate=energy_input*1;
          state s supplied(supplied) {{
           coordinate flow=F;coordinate pressure=P;coordinate temperature=T;
           transport energy=H tolerance 0.1{{W}};
          }}
          material port port=s;annotation connectivity port(1,1);
         }}
         def Root {{child source:Side=Side(supplied=true,energy_input=2{{W}});
          child receiver:Side=Side(supplied=false,energy_input={received}{{W}});
          connect stream:source.port->receiver.port;}}
        }}"#
        );
        let package = checked(&text).unwrap();
        let model = run(&text).unwrap();
        let connection = model.connections.values().next().unwrap();
        assert_eq!(connection.bindings.len(), 3);
        assert_eq!(model.equations.len(), 3);
        for (source, target) in &connection.bindings {
            let source = &model.symbols[&model.ports[source].symbol];
            let target = &model.symbols[&model.ports[target].symbol];
            assert_ne!(source.id, target.id);
            assert!(source.initial.is_some());
            assert_eq!(source.initial, target.initial);
            assert_eq!(source.lineage.declaration, target.lineage.declaration);
            assert_eq!(
                source.role,
                pse_model::generated::enums::ModelingDeclarationKind::Parameter
            );
            assert_eq!(
                target.role,
                pse_model::generated::enums::ModelingDeclarationKind::Parameter
            );
            assert!(!source.lineage.path.ends_with(".H"));
            assert!(!target.lineage.path.ends_with(".H"));
        }
        assert_eq!(model.closures.len(), 1);
        let closure = model.closures.values().next().unwrap();
        assert!(closure.observation_only);
        let (registry, _) = kernel_types::physical();
        let preconditions = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        let scope = PhysicalScope::default();
        let context = TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &registry,
            preconditions: &preconditions,
            scope: &scope,
        };
        let env = model
            .symbols
            .iter()
            .filter_map(|(id, symbol)| {
                symbol
                    .initial
                    .clone()
                    .map(|value| (specialize::symbol_name(*id), value))
            })
            .collect();
        let at = package.entry("p.Root").unwrap();
        let mut evaluator = specialize::value::Evaluator {
            selections: None,
            package: &package,
            physical: &context,
            at,
            env: &env,
            limit: 100,
            stack: Vec::new(),
            reader: crate::provenance::Reader::Production(at),
        };
        let assessment_type = crate::Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            closure
                .ty
                .quantity_scheme()
                .unwrap()
                .resolve_with_evidence(&registry, &Default::default(), &preconditions)
                .unwrap(),
        ));
        let magnitudes = closure
            .terms
            .iter()
            .map(|term| {
                let value = evaluator
                    .expr(&term.expression, Some(&assessment_type), 0)
                    .unwrap();
                let actual = specialize::value::value_type(&value)
                    .unwrap()
                    .quantity_scheme()
                    .unwrap()
                    .resolve_contract_with_evidence(&registry, &Default::default(), &preconditions)
                    .unwrap();
                let expected = closure
                    .ty
                    .quantity_scheme()
                    .unwrap()
                    .resolve_contract_with_evidence(&registry, &Default::default(), &preconditions)
                    .unwrap();
                assert!(actual.same_meaning(&expected));
                (term.id, value.scalar(at).unwrap())
            })
            .collect();
        let assessment = model
            .assess_closures(&magnitudes, &std::collections::BTreeSet::from([closure.id]))
            .unwrap();
        assert_eq!(assessment[0].satisfied, Some(expected));
        assert!((assessment[0].net - (2.0 - received)).abs() < 1e-12);
    }
}

#[test]
fn process_contract_static_inline_set_tuple_and_comprehension_indices_retain_full_source_grammar() {
    for domain in ["{a,b}", "{member for member in {b,a}}", "{[a,b],[b,a]}"] {
        let text = format!(
            r#"package p {{entity kind species {{}}entity species a {{}}entity species b {{}}
         def Root {{var x[j in {domain}]:Scalar;var y[k in {domain}]:Scalar;
          state outgoing supplied(true) {{coordinate amount[j in {domain}]=x[j];transport amount[j in {domain}]=x[j] tolerance 1e-8;}}
          state incoming supplied(false) {{coordinate amount[k in {domain}]=y[k];transport amount[k in {domain}]=y[k] tolerance 1e-8;}}
          material port output=outgoing;material port input=incoming;
          annotation connectivity output(0,1);annotation connectivity input(1,0);
          connect stream:output->input;
         }}
        }}"#
        );
        let package = checked(&text).unwrap();
        let state = package.entry("p.Root.outgoing").unwrap();
        let index = package
            .expression_occurrence(state, "state_specification.coordinates.0.indices.domain", 0)
            .unwrap();
        assert!(index.dependencies.contains(&package.entry("p.a").unwrap()));
        assert!(index.dependencies.contains(&package.entry("p.b").unwrap()));
        assert!(!index.unresolved_references.contains("member"));
        assert!(
            package
                .static_at(state, "state_specification.coordinates.0.indices.domain", 0)
                .is_ok()
        );
        let model = run(&text).unwrap();
        assert_eq!(model.connections.values().next().unwrap().bindings.len(), 2);
        assert_eq!(model.equations.len(), 2);
    }
}

#[test]
fn process_contract_finite_time_set_requires_realized_continuous_inventory_coordinates() {
    let text = "package p {set times:Set<Time>={0{s},1{s}};def Root {var x[i in times]:Time;conserve stock[i in times]:Time on times inventory x[i] flux 1 tolerance 1e-6{s};}}";
    let package = checked(text).unwrap();
    let stock = package
        .declarations
        .values()
        .find(|row| row.name == "stock")
        .unwrap()
        .declaration_id;
    let error = specialize(
        &package,
        package.entry("p.Root").unwrap(),
        InstanceId::from_bytes([7; 16]),
        &Bindings::default(),
        Limits::default(),
    )
    .unwrap_err();
    let ModelingError::Contract {
        declaration,
        message,
    } = error
    else {
        panic!("unexpected finite-set refusal: {error}");
    };
    assert_eq!(declaration, stock.into());
    assert!(
        message.contains("realized continuous coordinate"),
        "{message}"
    );
}

#[test]
fn process_contract_inventory_event_transfer_requires_a_local_numeric_guard_member() {
    let source = "package p {def Root {domain t:Time from 0{s} to 1{s};var x[i in t]:Time;let hit[i in t]:Time=x[i]-0.5{s};param flag:Boolean=true;conserve stock[i in t]:Time on t inventory x[i] flux 1 tolerance 1e-6{s} transfers(EVENT);}}";
    for event in ["hit[0{s}]=1{s}", "hit[i]=1{s}"] {
        checked(&source.replace("EVENT", event)).unwrap();
    }
    for (event, diagnostic) in [
        (
            "root.hit[0{s}]=1{s}",
            "unknown contextual declaration reference",
        ),
        ("hit[0{s}]+1{s}=1{s}", "requires a member path"),
        ("flag=1{s}", "physical numeric member"),
        (
            "hit[0{s}]=1{s},hit[0 {s}]=1{s}",
            "duplicate inventory event transfer",
        ),
    ] {
        let error = checked(&source.replace("EVENT", event))
            .unwrap_err()
            .to_string();
        assert!(error.contains(diagnostic), "{event}: {error}");
    }
}

#[test]
fn process_contract_inherited_guarded_balance_members_share_only_their_source_physical_contract() {
    let source = r#"package p {
     interface Balances {
      param mode:Integer;
      when mode==0 {accumulate energy:Power conservation tolerance 0.001{W};}
      when mode==1 {accumulate energy:Power observation tolerance 0.001{W};}
      when mode==2 {accumulate energy:Power accounting tolerance 0.001{W};}
      contribute energy role positive=1{W};
     }
     interface CellBalances extends Balances {let demand:Power=energy;}
     def Root:CellBalances {param mode:Integer=MODE;}
    }"#;
    for (mode, expected) in [
        (
            "0",
            pse_model::generated::enums::ModelingAccumulatorMode::Conservation,
        ),
        (
            "1",
            pse_model::generated::enums::ModelingAccumulatorMode::Observation,
        ),
        (
            "2",
            pse_model::generated::enums::ModelingAccumulatorMode::Accounting,
        ),
    ] {
        let model = run(&source.replace("MODE", mode)).unwrap();
        assert_eq!(model.closures.len(), 1);
        assert_eq!(model.closures.values().next().unwrap().mode, expected);
    }
    let unlike = source
        .replace(
            "energy:Power accounting tolerance 0.001{W}",
            "energy:Flow accounting tolerance 0.001{mol/s}",
        )
        .replace("MODE", "0");
    let error = checked(&unlike).unwrap_err().to_string();
    assert!(error.contains("unknown member energy"), "{error}");
}

#[test]
fn process_contract_indexed_coordinate_map_uses_its_lexical_callable_formals() {
    let source = r#"package p {
     entity kind item {}entity item a {}
     param scale:Time=1{s};
     coordinate map reduced(n:Amount[item],members:Set<item>,scale:Amount) valid(scale>0{mol}) {
      slot fraction[i in members]=n[i]/scale;
     }
     def Root {}
    }"#;
    let package = checked(source).unwrap();
    let slot = package
        .declarations
        .values()
        .find(|row| row.name == "fraction")
        .unwrap()
        .declaration_id;
    let occurrence = package
        .expression_occurrence(slot, "coordinate_slot.indices.domain", 0)
        .unwrap();
    assert!(occurrence.unresolved_references.contains("members"));
    assert!(
        package.functions[&slot]
            .arguments
            .iter()
            .any(|(name, ty)| name == "members" && matches!(ty, crate::Type::Set(_)))
    );
    let map = package.declarations[&slot].parent_id.unwrap();
    let validity = package
        .expression_occurrence(map, "coordinate_map.validity", 0)
        .unwrap();
    assert!(validity.unresolved_references.contains("scale"));
    let global_scale = package
        .declarations
        .values()
        .find(|row| row.name == "scale")
        .unwrap()
        .declaration_id;
    assert!(!validity.dependencies.contains(&global_scale));
    let unlike = source.replace("members:Set<item>", "members:Scalar");
    assert!(
        checked(&unlike)
            .unwrap_err()
            .to_string()
            .contains("finite-set argument")
    );
}

const CORRESPONDENCE: &str = r#"package p {
 entity kind species {} entity species a {} entity species b {}
 set forward:Set<species>={a,b}; set reverse:Set<species>={b,a};
 def Source(members:Set<species>) {
  var amount[j in members]:Scalar;
  state s supplied(true) {
   coordinate component[j in members]=amount[j];
   transport component[j in members]=amount[j] tolerance 1e-8;
  }
  material port output=s;
  annotation connectivity output(0,2);
 }
 def Receiver(members:Set<species>) {
  var amount[k in members]:Scalar; var total:Scalar;
  state s supplied(false) {
   coordinate component[k in members]=amount[k];
   reconstruct total: total==sum(k in members | amount[k]) tolerance 1e-8;
   transport component[k in members]=amount[k] tolerance 1e-8;
  }
  material port input=s;
  annotation connectivity input(2,0);
 }
 def Root {
  child source:Source=Source(members=forward);
  child receiver:Receiver=Receiver(members=reverse);
  connect stream: source.output->receiver.input;
 }
}"#;

#[test]
fn process_contract_species_reordering_binds_semantic_members_and_one_receiver_reconstruction() {
    let model = run(CORRESPONDENCE).unwrap();
    assert_eq!(model.material_ports.len(), 2);
    let connection = model.connections.values().next().unwrap();
    let source = &model.material_ports[&connection.from];
    let target = &model.material_ports[&connection.to];
    assert_eq!(source.coordinates.len(), 2);
    for (species, port) in &source.coordinates {
        assert!(
            connection
                .bindings
                .contains(&(*port, target.coordinates[species]))
        );
    }
    assert_eq!(connection.bindings.len(), 2);
    assert_eq!(connection.rows.len(), 2);
    assert_eq!(model.equations.len(), 3);
    assert_eq!(
        model
            .equations
            .iter()
            .filter(|row| row.lineage.path.contains("receiver.s"))
            .count(),
        1
    );
    assert_eq!(
        model
            .closures
            .values()
            .filter(|closure| closure.observation_only)
            .count(),
        3
    );
}

#[test]
fn process_contract_indexed_material_specs_ports_and_connections_keep_one_occurrence_per_index() {
    let model = run(r#"package p {
     entity kind species {} entity species a {} entity species b {}
     set forward:Set<species>={a,b}; set reverse:Set<species>={b,a};
     def Root {
      var x[j in forward]:Scalar; var y[k in reverse]:Scalar;
      state outgoing[j in forward] supplied(true) {coordinate amount=x[j];transport amount=x[j] tolerance 1e-8;}
      state incoming[k in reverse] supplied(false) {coordinate amount=y[k];transport amount=y[k] tolerance 1e-8;}
      material port output[j in forward]=outgoing[j];
      material port input[k in reverse]=incoming[k];
      annotation connectivity output(0,1);annotation connectivity input(1,0);
      connect streams: [j in forward] output[j]->input[j];
     }
    }"#).unwrap();
    assert_eq!(model.state_specifications.len(), 4);
    assert_eq!(model.material_ports.len(), 4);
    assert_eq!(model.connections.len(), 2);
    assert_eq!(model.equations.len(), 2);
    assert!(
        model
            .connections
            .values()
            .all(|connection| connection.bindings.len() == 1 && connection.rows.len() == 1)
    );
    let endpoints = model
        .connections
        .values()
        .map(|connection| (connection.from, connection.to))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(endpoints.len(), 2);
}

#[test]
fn process_contract_multiple_connections_do_not_duplicate_receiver_reconstruction() {
    let text = CORRESPONDENCE.replace("connect stream: source.output->receiver.input;", "connect first: source.output->receiver.input; connect second: source.output->receiver.input;");
    let model = run(&text).unwrap();
    assert_eq!(model.connections.len(), 2);
    assert_eq!(model.equations.len(), 5);
    assert_eq!(
        model
            .equations
            .iter()
            .filter(|row| row.lineage.path.contains("receiver.s"))
            .count(),
        1
    );
}

#[test]
fn process_contract_state_extension_reuses_independent_slots_and_reconstruction_identity() {
    let model = run(r#"package p {def Root {
     var amount:Scalar;var total:Scalar;
     state base supplied(false) {coordinate amount=amount;reconstruct total:total==amount tolerance 1e-8;transport amount=amount tolerance 1e-8;}
     state enriched extends base supplied(false) {transport detail=total tolerance 1e-8;}
     material port output=enriched;material port input=enriched;
     annotation connectivity output(0,1);annotation connectivity input(1,0);
     connect stream:output->input;
    }}"#).unwrap();
    assert_eq!(model.state_specifications.len(), 2);
    let base = model
        .state_specifications
        .values()
        .find(|specification| specification.lineage.path.ends_with(".base"))
        .unwrap();
    let enriched = model
        .state_specifications
        .values()
        .find(|specification| specification.lineage.path.ends_with(".enriched"))
        .unwrap();
    assert_eq!(base.coordinates, enriched.coordinates);
    assert_eq!(
        base.reconstructions[0].0.id,
        enriched.reconstructions[0].0.id
    );
    assert_eq!(enriched.transports.len(), 2);
    assert_eq!(
        model
            .equations
            .iter()
            .filter(|row| row.id == base.reconstructions[0].0.id)
            .count(),
        1
    );
}

#[test]
fn process_contract_recursive_state_extension_refuses_before_specialization() {
    let error = checked(r#"package p {def Root {
     var amount:Scalar;
     state first extends second supplied(false) {coordinate amount=amount;transport amount=amount tolerance 1e-8;}
     state second extends first supplied(false) {transport detail=amount tolerance 1e-8;}
    }}"#).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("recursive state specification extension")
    );
}

#[test]
fn process_contract_extension_cannot_duplicate_an_original_independent_member() {
    let error = run(r#"package p {def Root {
     var amount:Scalar;
     state base supplied(false) {coordinate amount=amount;transport amount=amount tolerance 1e-8;}
     state enriched extends base supplied(false) {coordinate duplicate=amount;transport detail=amount tolerance 1e-8;}
    }}"#).unwrap_err().to_string();
    assert!(
        error.contains("one independent coordinate cannot occupy multiple state slots"),
        "{error}"
    );
}

#[test]
fn process_contract_source_types_reject_scalar_ports_and_unlike_reconstruction_tolerances() {
    for (text, expected) in [
        (
            "package p {def Root {var amount:Scalar;state s supplied(true) {coordinate amount=amount;} material port inlet=s;eq wrong:inlet==amount;}}",
            "unknown member inlet",
        ),
        (
            "package p {def Root {var amount:Amount;state s supplied(true) {coordinate amount=amount;reconstruct amount:amount==amount tolerance 1e-8{s};}}}",
            "conversion.dimension",
        ),
        (
            "package p {def Root {var amount:Scalar;state s supplied(1) {coordinate amount=amount;}}}",
            "physical",
        ),
    ] {
        let error = run(text).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn process_contract_unlike_species_and_transported_coverage_require_explicit_translation() {
    let fewer = CORRESPONDENCE.replace(
        "set reverse:Set<species>={b,a};",
        "set reverse:Set<species>={a};",
    );
    assert!(
        run(&fewer)
            .unwrap_err()
            .to_string()
            .contains("species correspondence")
    );
    let unlike = CORRESPONDENCE.replace(
        "transport component[k in members]",
        "transport other[k in members]",
    );
    assert!(
        run(&unlike)
            .unwrap_err()
            .to_string()
            .contains("transport/species coverage")
    );
}

#[test]
fn process_contract_unlike_transport_physical_conventions_require_explicit_translation() {
    let text = r#"package p {def Root {
     var a:Scalar; var b:Scalar; var molar:Flow; var volumetric:VolumeFlow;
     state source supplied(true) {coordinate amount=a;transport rate=molar tolerance 1e-8{mol/s};}
     state target supplied(false) {coordinate amount=b;transport rate=volumetric tolerance 1e-8{m^3/s};}
     material port output=source;material port input=target;
     connect stream:output->input;
    }}"#;
    let error = run(text).unwrap_err().to_string();
    assert!(
        error.contains("transport basis/reference conventions"),
        "{error}"
    );
}

#[test]
fn process_contract_explicit_translator_unit_owns_separate_specs_and_conservation_equations() {
    let text = CORRESPONDENCE.replace("coordinate component[k in members]", "coordinate translated[k in members]")
        .replace(" def Root {", r#" def Translator(members:Set<species>) {
         var incoming[j in members]:Scalar;var outgoing[k in members]:Scalar;
         state left supplied(false) {coordinate component[j in members]=incoming[j];transport component[j in members]=incoming[j] tolerance 1e-8;}
         state right supplied(false) {coordinate translated[k in members]=outgoing[k];transport component[k in members]=outgoing[k] tolerance 1e-8;}
         material port input=left;material port output=right;
         annotation connectivity input(1,0);annotation connectivity output(0,1);
         eq conservation[j in members]:outgoing[j]==incoming[j];
        }
        def Root {"#)
        .replace("connect stream: source.output->receiver.input;", "child translator:Translator=Translator(members=forward); connect first:source.output->translator.input;connect second:translator.output->receiver.input;");
    let model = run(&text).unwrap();
    assert_eq!(model.material_ports.len(), 4);
    assert_eq!(model.connections.len(), 2);
    assert_eq!(
        model
            .equations
            .iter()
            .filter(|row| row.lineage.path.contains("translator.conservation"))
            .count(),
        2
    );
    let translator_specs = model
        .state_specifications
        .values()
        .filter(|specification| specification.lineage.path.contains("translator."))
        .collect::<Vec<_>>();
    assert_eq!(translator_specs.len(), 2);
    assert_ne!(
        translator_specs[0].coordinates.keys().next().unwrap().name,
        translator_specs[1].coordinates.keys().next().unwrap().name
    );
}
