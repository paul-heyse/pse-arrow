// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The thermodynamic domain schema as the reference packages declare it (Plan 23 D0): the
//! chemical core in `pse.physical`'s `chemistry` module, and the property, interaction and
//! constant schema of `pse.domain`. The refusal corpus in `tests/fixtures/domain-refusals`
//! applies the schema to small packages that each violate one constraint.
use crate::kernel_types::physical;
use crate::specialize::Value;
use crate::*;
use pse_ids::SemanticId;
use std::path::{Path, PathBuf};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The schema's modeling sources: the chemistry and compatibility modules of `pse.physical`
/// and every module of `pse.domain`, in path order.
fn schema() -> Vec<Declaration> {
    let models = repository().join("packages/reference");
    let mut paths = vec![
        models.join("physical/models/chemistry.pse"),
        models.join("physical/models/compatibility.pse"),
    ];
    let mut domain = std::fs::read_dir(models.join("domain/models"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "pse"))
        .collect::<Vec<_>>();
    domain.sort();
    paths.extend(domain);
    let mut rows = Vec::new();
    {
        for path in paths {
            let text = std::fs::read_to_string(&path).unwrap();
            rows.extend(
                pse_authoring::language::parse(
                    &text,
                    SemanticId::NIL,
                    pse_authoring::language::IdentityPolicy::Explicit,
                    pse_authoring::ParseBudget::default(),
                )
                .unwrap_or_else(|e| panic!("{}: {e}", path.display())),
            );
        }
    }
    rows
}

/// Admission against the standard physical document and its operation preconditions.
fn admitted(rows: &[Declaration]) -> Result<CheckedPackage> {
    let (registry, _) = physical();
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        preconditions: &pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap(),
        quantities: &registry,
        scope: &PhysicalScope::default(),
    };
    check(rows, &context)
}

/// The schema admits on its own, with no data rows: the chemistry kinds carry their
/// attribute schemas, the canonical phases their types, and the gas constant its typed kind
/// and provenance.
#[test]
fn domain_schema_admits_without_data_rows() {
    let p = admitted(&schema()).unwrap();
    let species = p.names["chemistry.species"];
    let kind = &p.kinds[&species];
    assert_eq!(
        kind.attributes
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>(),
        ["cas", "inchikey", "charge", "molar_mass"]
    );
    assert_eq!(
        kind.derived
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>(),
        ["molar_mass"]
    );
    let element = &p.kinds[&p.names["chemistry.element"]];
    assert_eq!(
        element
            .unique
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>(),
        ["symbol", "atomic_number"]
    );
    assert_eq!(
        p.kinds[&p.names["chemistry.apparent"]].requirements.len(),
        1
    );
    assert!(p.refines(p.names["chemistry.apparent"], species));
    // The canonical phases carry their types.
    let liquid = p.record(p.names["chemistry.liquid"]).unwrap();
    assert!(matches!(liquid.values["type"], Value::Enum { .. }));
    // The gas constant is typed by its own kind and cites its source.
    let r = p.names["constants.gas_constant"];
    let Some(Type::Quantity(scheme)) = p.types.get(&r) else {
        panic!("gas constant type")
    };
    assert_eq!(
        p.quantities.physical_name("GasConstant"),
        Some(pse_quantity::PhysicalName::QuantityType(
            scheme
                .resolve_with_evidence(&p.quantities, &Default::default(), p.preconditions.as_ref())
                .unwrap()
        ))
    );
    assert!((p.constants[&r].value.scalar(r).unwrap() - 8.31446261815324).abs() < 1e-14);
    assert_eq!(
        p.provenance(r).unwrap().source,
        p.names["constants.codata_2018"]
    );
    // The parameter set is keyed and abstract; its families refine it.
    let (key_kind, keys) = p.keys(p.names["properties.caloric_set"]).unwrap();
    assert_eq!(key_kind, p.names["properties.parameter_set"]);
    assert_eq!(
        keys.iter().map(|k| k.0.as_str()).collect::<Vec<_>>(),
        ["subject", "property", "phase_type", "source", "variant"]
    );
}

/// One case of the refusal corpus: its body, the substrings its refusal names, the root whose
/// specialization refuses it (when admission does not) and the edit that makes it admissible.
struct Case {
    name: String,
    body: String,
    refusals: Vec<String>,
    root: Option<String>,
    control: (String, String),
}

fn corpus_directory() -> PathBuf {
    repository().join("tests/fixtures/domain-refusals")
}

/// Every case of the corpus, in name order; its header comments declare the rest.
fn corpus() -> Vec<Case> {
    let mut paths = std::fs::read_dir(corpus_directory())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "pse"))
        .filter(|path| path.file_stem().is_some_and(|stem| stem != "scaffold"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path).unwrap();
            let header = |key: &str| {
                text.lines()
                    .filter_map(|line| line.strip_prefix(&format!("// {key}: ")))
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            };
            let control = header("control");
            let [control] = control.as_slice() else {
                panic!("{}: one control edit", path.display())
            };
            let (from, to) = control.split_once(" =>").unwrap();
            Case {
                name: path.file_stem().unwrap().to_string_lossy().into_owned(),
                body: text
                    .lines()
                    .filter(|line| !line.starts_with("//"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                refusals: header("refusal"),
                root: header("root").into_iter().next(),
                control: (from.to_owned(), to.trim_start().to_owned()),
            }
        })
        .collect()
}

/// Admit the schema, the corpus scaffold and a case, then specialize its root if it names one.
fn outcome(body: &str, root: Option<&str>) -> std::result::Result<(), String> {
    let parse = |text: &str| {
        pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .map_err(|e| e.to_string())
    };
    let mut rows = schema();
    rows.extend(parse(
        &std::fs::read_to_string(corpus_directory().join("scaffold.pse")).unwrap(),
    )?);
    rows.extend(parse(body)?);
    let p = admitted(&rows).map_err(|e| e.to_string())?;
    if let Some(root) = root {
        specialize(
            &p,
            p.names[root],
            InstanceId::from_bytes([7; 16]),
            &Bindings::default(),
            Limits::default(),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Each case of the refusal corpus violates one constraint of the domain schema and is
/// refused with a message naming it; its declared control edit removes the violation and
/// admits, so the refusal is the constraint's and nothing else's.
#[test]
fn domain_schema_refusals_name_the_violated_constraint() {
    outcome("", None).unwrap();
    let cases = corpus();
    assert_eq!(
        cases.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        [
            "apparent-electroneutrality",
            "cation-charge",
            "coefficient-unit-mismatch",
            "conflicting-symmetric-pair",
            "duplicate-cas",
            "missing-pair-selection",
            "oracle-row-in-production",
            "reaction-element-closure",
            "selection-subject-mismatch",
            "unknown-formula-element",
        ]
    );
    for case in cases {
        let error = outcome(&case.body, case.root.as_deref())
            .err()
            .unwrap_or_else(|| panic!("{} admitted", case.name));
        assert!(!case.refusals.is_empty(), "{}", case.name);
        for part in &case.refusals {
            assert!(
                error.contains(part.as_str()),
                "{}: expected {part}: {error}",
                case.name
            );
        }
        let (from, to) = &case.control;
        assert_eq!(
            case.body.matches(from.as_str()).count(),
            1,
            "{}: control {from}",
            case.name
        );
        if let Err(error) = outcome(
            &case.body.replacen(from.as_str(), to, 1),
            case.root.as_deref(),
        ) {
            panic!("{}: the control is refused: {error}", case.name);
        }
    }
}

const OPERATIONS: &str = r#"package operations {
 use chemistry @"1.0.0";
 use corpus @"1.0.0";
 use interactions @"1.0.0";
 use properties @"1.0.0";
 use provenance @"1.0.0";
 entity kind constant_psat extends properties.vapor_pressure_set { attribute p0: Pressure; psat = constant_pressure; }
 fn constant_pressure(T: Temperature, s: constant_psat) -> Pressure = s.p0;
 entity kind constant_density extends properties.liquid_density_set { attribute rho: MolarDensity; density = constant_rho; }
 fn constant_rho(T: Temperature, s: constant_density) -> MolarDensity = s.rho;
 dataset vapor: constant_psat bind(source = corpus.bank, property = properties.vapor_pressure) provenance(corpus.bank, provenance.Role.published) { [corpus.benzene, liquidPhase] = [250{K}, 400{K}, 10000{Pa}]; }
 dataset density: constant_density bind(source = corpus.bank, property = properties.liquid_molar_density) provenance(corpus.bank, provenance.Role.published) { [corpus.benzene, liquidPhase] = [250{K}, 400{K}, 11000{mol/m^3}]; }
 set components: Set<chemistry.species> = {corpus.benzene};
 set pure: Set<properties.property> = {properties.heat_capacity, properties.vapor_pressure, properties.liquid_molar_density};
 dataset choice: properties.selection complete_over(k in corpus.packages, j in components, p in pure, t in corpus.liquid) provenance(corpus.bank, provenance.Role.published) {
  [corpus.pkg, corpus.benzene, properties.heat_capacity, liquidPhase] = [properties.parameter_set[corpus.benzene, properties.heat_capacity, liquidPhase, corpus.bank]];
  [corpus.pkg, corpus.benzene, properties.vapor_pressure, liquidPhase] = [properties.parameter_set[corpus.benzene, properties.vapor_pressure, liquidPhase, corpus.bank]];
  [corpus.pkg, corpus.benzene, properties.liquid_molar_density, liquidPhase] = [properties.parameter_set[corpus.benzene, properties.liquid_molar_density, liquidPhase, corpus.bank]];
 }
 entity properties.property kij { quantity = Scalar, shape = properties.IndexShape.pair, applies = {liquidPhase, vaporPhase} }
 set kijs: Set<properties.property> = {kij};
 dataset kij_values: interactions.pair provenance(corpus.bank, provenance.Role.published) { [corpus.bank, kij, corpus.benzene, corpus.toluene] = [0.01]; }
 dataset kij_source: interactions.pair_selection complete_over(k in corpus.packages, p in kijs) provenance(corpus.bank, provenance.Role.published) { [corpus.pkg, kij] = [corpus.bank]; }
 def Root {
  var c: MolarCp; var h: DeltaH; var s: DeltaS; var p: Pressure; var d: MolarDensity; var k: Scalar;
  eq heat: c == properties.cp(corpus.pkg, chemistry.liquid, corpus.benzene, 300{K});
  eq enthalpy: h == properties.enthalpy_increment(corpus.pkg, chemistry.liquid, corpus.benzene, 298.15{K}, 300{K});
  eq entropy: s == properties.entropy_increment(corpus.pkg, chemistry.liquid, corpus.benzene, 298.15{K}, 300{K});
  eq saturation: p == properties.psat(corpus.pkg, chemistry.liquid, corpus.benzene, 300{K});
  eq molar_density: d == properties.liquid_density(corpus.pkg, chemistry.liquid, corpus.benzene, 300{K});
  eq pair: k == interactions.pair_parameter(corpus.pkg, kij, corpus.toluene, corpus.benzene);
 }
}"#;

/// The generic property operations are the domain's contract for reading a property: each
/// resolves the package's selection, narrows the selected set to its family by its keys and
/// evaluates the form's bound function, whatever form the package selected. A missing
/// selection is refused naming the package, species, property and phase type.
#[test]
fn property_operations_dispatch_through_selection() {
    outcome(OPERATIONS, Some("operations.Root")).unwrap();
    let error = outcome(
        &OPERATIONS.replace(
            "properties.cp(corpus.pkg, chemistry.liquid,",
            "properties.cp(corpus.pkg, chemistry.vapor,",
        ),
        Some("operations.Root"),
    )
    .unwrap_err();
    assert!(
        error.contains("lookup selection[pkg, benzene, heat_capacity, vaporPhase] is outside the completeness of selection"),
        "{error}"
    );
}

/// The schema's pure fixtures admit over the schema alone and each specializes as a test.
#[test]
fn domain_fixtures_admit_and_specialize() {
    let mut rows = schema();
    let path = repository().join("packages/reference/domain-fixtures/models/schema-fixtures.pse");
    rows.extend(
        pse_authoring::language::parse(
            &std::fs::read_to_string(path).unwrap(),
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Explicit,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap(),
    );
    let p = admitted(&rows).unwrap();
    for test in [
        "derived_molar_mass",
        "two_group_unifac_admission",
        "property_operations_read_the_selected_form",
    ] {
        let root = p.names[&format!("schema_fixtures.{test}")];
        specialize(
            &p,
            root,
            InstanceId::from_bytes([7; 16]),
            &Bindings::default(),
            Limits::default(),
        )
        .unwrap_or_else(|e| panic!("{test}: {e}"));
    }
    // Synthetic data taints what derives from it: the formula rows make the molar masses of
    // the formulated species test-only.
    assert!(p.is_test_only(p.names["schema_fixtures.light_heavy"]));
    assert!(!p.is_test_only(p.names["schema_fixtures.unformulated"]));
}
