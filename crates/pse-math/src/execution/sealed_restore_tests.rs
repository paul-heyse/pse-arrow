// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A narrow copy-only control, not a production recipe decoder. The test parent
//! acts as the custody owner of exact bytes from an identified typed producer.
use super::*;
use crate::typed::{Binary, BodyBuilder, BodyLimits};
use pse_ids::{ContentHash, preimage};
use pse_quantity::{
    IndexSet, QuantityTypeId,
    literal::LiteralContext,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use serde::{Deserialize, Serialize};
use symbolica::atom::{
    AtomView, NamespacedSymbol,
    packing::{FloatEnvironment, PackingFrame, inspect_packing},
};

const INPUTS: usize = 2;
const GUARDED: usize = INPUTS;
const OUTPUT: usize = GUARDED + 1;
const SLOTS: usize = OUTPUT + 1;
const SNAPSHOT_ENV: &str = "PSE_SYMBOLIC_TEST_SEALED_BODY";
const DIGEST_ENV: &str = "PSE_SYMBOLIC_TEST_SEALED_DIGEST";
const SUBSTITUTION_ENV: &str = "PSE_SYMBOLIC_TEST_SEALED_SUBSTITUTION";
const MAX_FIXTURE_BYTES: usize = 4096;
const FORMAL_NAMES: [&str; SLOTS] = [
    "pse_math::slot_0",
    "pse_math::slot_1",
    "pse_math::slot_2",
    "pse_math::slot_3",
];
// A restrictive scan policy, not a claim about the backend's actual Float
// range: anything that passes this filter still refuses if it contains a Float.
const FLOAT_SCAN: FloatEnvironment = FloatEnvironment {
    min_precision: 1,
    max_precision: 1,
    min_exponent: 0,
    max_exponent: 0,
};

fn occurrence(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}

fn occurrence_tokens() -> BTreeSet<SemanticId> {
    (1..=6).map(occurrence).collect()
}

fn produce() -> PreparedBody {
    let context = crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let mut builder = BodyBuilder::new(
        context,
        &registry,
        &StandardInvariantChecker,
        INPUTS,
        BodyLimits::default(),
    )
    .unwrap();
    let neutral = ids::quantity("neutral");
    let mut x = builder
        .input(0, neutral, IndexSet::new(), occurrence(1))
        .unwrap();
    builder
        .input(1, neutral, IndexSet::new(), occurrence(4))
        .unwrap();
    builder
        .require(
            &mut x,
            Condition::Positive,
            DerivativeOrder::Value,
            occurrence(2),
        )
        .unwrap();
    let square = builder
        .binary(Binary::Mul, x.clone(), x, None, occurrence(5))
        .unwrap();
    let unit = registry
        .unit(registry.quantity_type(neutral).unwrap().canonical_unit)
        .unwrap();
    let multiplier = builder
        .literal(
            2.0,
            unit,
            LiteralContext::Explicit { quantity_type: neutral },
            occurrence(6),
        )
        .unwrap();
    let out = builder
        .binary(Binary::Mul, square, multiplier, None, occurrence(3))
        .unwrap();
    builder
        .prepare(&[out])
        .unwrap()
        .with_occurrence_tokens(occurrence_tokens())
        .unwrap()
}

// The snapshot represents exactly this fixture's two Blocks, one Require and
// one result. It is deliberately not a second general schema for guarded bodies.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    ids: [u32; SLOTS],
    atoms: [Vec<u8>; 4],
    quantity: [QuantityTypeId; INPUTS + 1],
    remaining: usize,
    occurrences: usize,
}

fn snapshot(body: &PreparedBody) -> Fixture {
    assert_eq!(
        (body.inputs, body.slots, body.outputs.as_slice()),
        (INPUTS, SLOTS, &[OUTPUT][..]),
    );
    assert!(body.formal_slots.is_empty() && body.providers.is_empty());
    assert!(body.checked_members().is_empty() && body.checked_sources().is_empty());
    let [
        Stage::Block {
            expressions: input,
            outputs: input_slots,
            source: input_source,
        },
        Stage::Require {
            argument: GUARDED,
            condition: Condition::Positive,
            order: DerivativeOrder::Value,
            source: guard_source,
            lineage: None,
        },
        Stage::Block {
            expressions: result,
            outputs: result_slots,
            source: result_source,
        },
    ] = body.stages.as_slice()
    else {
        panic!("fixture did not produce the selected guarded shape");
    };
    assert_eq!(
        (*input_source, *guard_source, *result_source),
        (occurrence(2), occurrence(2), occurrence(3)),
    );
    assert_eq!(input_slots.as_slice(), &[GUARDED]);
    assert_eq!(result_slots.as_slice(), &[OUTPUT]);
    assert_eq!(body.output_effects, Some(vec![BTreeSet::from([GUARDED])]));
    let [(Some(obligation), Condition::Positive)] = body.obligations.as_slice() else {
        panic!("fixture did not retain its original guard");
    };
    let encode = |atom: &Atom| atom.as_view().get_data().to_vec();
    let ids = crate::context()
        .unwrap()
        .pool
        .with_existing_formals(SLOTS, |formals| {
            Ok(std::array::from_fn(|index| {
                formals[index].get_symbol().unwrap().get_id()
            }))
        })
        .unwrap();
    Fixture {
        ids,
        atoms: [
            encode(&input[0]),
            encode(&result[0]),
            encode(obligation),
            encode(body.expression(0).unwrap()),
        ],
        quantity: [
            body.input_quantities[0].unwrap(),
            body.input_quantities[1].unwrap(),
            body.output_quantities[0],
        ],
        remaining: body.remaining_occurrences,
        occurrences: body.occurrences,
    }
}

impl Fixture {
    fn decode_original(
        payload: &str,
        expected: ContentHash,
        allowance: usize,
    ) -> Result<Self, MathError> {
        let available = allowance.min(MAX_FIXTURE_BYTES);
        if payload.len() > available {
            return Err(MathError::ByteLimit {
                resource: "sealed fixture payload",
                required: payload.len(),
                available,
            });
        }
        // The parent supplies the digest out of band, from the identified typed
        // producer's exact output. The digest does not prove normalization or
        // semantic authority. Inspect and restore only the decoded original.
        if preimage::hash(payload.as_bytes()) != expected {
            return Err(MathError::Contract(
                "sealed fixture original bytes differ".into(),
            ));
        }
        serde_json::from_str(payload).map_err(|error| MathError::Contract(error.to_string()))
    }

    fn inspect(&self, allowance: usize) -> Result<[&[u8]; 4], MathError> {
        crate::context()?
            .pool
            .with_existing_formals(SLOTS, |formals| {
                let actual: [u32; SLOTS] =
                    std::array::from_fn(|i| formals[i].get_symbol().unwrap().get_id());
                if actual != self.ids {
                    return Err(MathError::Contract(
                        "sealed fixture destination symbols differ".into(),
                    ));
                }
                for (name, id) in FORMAL_NAMES.into_iter().zip(actual) {
                    let name = NamespacedSymbol::try_parse_lit(name).unwrap();
                    if Symbol::get_symbol(name).map(|symbol| symbol.get_id()) != Some(id) {
                        return Err(MathError::Contract(
                            "sealed fixture destination formal role differs".into(),
                        ));
                    }
                }
                // The typed producer's shape is fixed by this control. Its caller
                // owns the surrounding metadata and the fixed inspection stack.
                let required = self.atoms.iter().try_fold(0_usize, |total, atom| {
                    total
                        .checked_add(atom.len())
                        .ok_or(MathError::Limit("sealed fixture atoms"))
                })?;
                if required > allowance {
                    return Err(MathError::ByteLimit {
                        resource: "sealed fixture atoms",
                        required,
                        available: allowance,
                    });
                }
                let mut frames = [PackingFrame::default(); 8];
                let mut inspect = |index: usize| {
                    let bytes = self.atoms[index].as_slice();
                    // An executable block may use any earlier slot in this
                    // shape; flattened data can use only original inputs.
                    // This checks admissible references, not original dependence.
                    let slots = if index == 1 { OUTPUT } else { INPUTS };
                    let summary = inspect_packing(bytes, &mut frames, FLOAT_SCAN, |symbol| {
                        if symbol.function || !actual[..slots].contains(&symbol.id) {
                            return Err("sealed fixture symbol outside its formal layout");
                        }
                        Ok(())
                    })
                    .map_err(|error| MathError::Contract(error.to_string()))?;
                    if summary.float_components != 0
                        || summary.large_rational_bytes != 0
                        || summary.rational_components != summary.numbers
                    {
                        return Err(MathError::Contract(
                            "sealed fixture only admits small real rationals".into(),
                        ));
                    }
                    let shape = (index, AtomView::from(bytes));
                    if summary.variables != 1
                        || !matches!(shape, (0 | 2, AtomView::Var(_)) | (1 | 3, AtomView::Mul(_)))
                    {
                        return Err(MathError::Contract(
                            "sealed fixture atom shape differs".into(),
                        ));
                    }
                    Ok(bytes)
                };
                Ok([inspect(0)?, inspect(1)?, inspect(2)?, inspect(3)?])
            })
    }

    fn restore(&self, allowance: usize) -> Result<PreparedBody, MathError> {
        let [input, result, obligation, expression] = self.inspect(allowance)?;
        let restore = |packed: &[u8]| {
            // The test parent forwards exact normalized bytes from this executable's
            // typed producer. Inspection already matched destination IDs and roles.
            // The concrete view copies directly into backing that was admitted
            // together with all other atom backing before any copy begins.
            let mut buffer = Vec::new();
            buffer
                .try_reserve_exact(packed.len())
                .map_err(|error| MathError::Library(error.to_string()))?;
            match AtomView::from(packed) {
                AtomView::Var(view) => Ok(Atom::Var(view.clone_into_raw(buffer))),
                AtomView::Mul(view) => Ok(Atom::Mul(view.clone_into_raw(buffer))),
                _ => Err(MathError::Contract(
                    "sealed fixture expects a formal or product root".into(),
                )),
            }
        };
        let symbols = crate::context()?
            .pool
            .with_existing_formals(SLOTS, |formals| Ok(symbol_map(formals)))?;
        Ok(PreparedBody {
            data: Arc::new(PreparedBodyData {
                inputs: INPUTS,
                slots: SLOTS,
                formal_slots: vec![],
                symbols,
                outputs: vec![OUTPUT],
                output_effects: Some(vec![BTreeSet::from([GUARDED])]),
                stages: vec![
                    Stage::Block {
                        expressions: vec![restore(input)?],
                        outputs: vec![GUARDED],
                        source: occurrence(2),
                    },
                    Stage::Require {
                        argument: GUARDED,
                        condition: Condition::Positive,
                        order: DerivativeOrder::Value,
                        source: occurrence(2),
                        lineage: None,
                    },
                    Stage::Block {
                        expressions: vec![restore(result)?],
                        outputs: vec![OUTPUT],
                        source: occurrence(3),
                    },
                ],
                remaining_occurrences: self.remaining,
                obligations: vec![(Some(restore(obligation)?), Condition::Positive)],
                input_quantities: self.quantity[..INPUTS].iter().copied().map(Some).collect(),
                output_quantities: vec![self.quantity[INPUTS]],
                expressions: vec![Some(restore(expression)?)],
                providers: vec![],
                checked_members: BTreeSet::new(),
                checked_sources: BTreeSet::new(),
                occurrence_tokens: occurrence_tokens(),
                occurrences: self.occurrences,
            }),
            owner: None,
        })
    }
}

#[test]
fn sealed_guarded_body_child() {
    let Ok(command) = std::env::var(SNAPSHOT_ENV) else {
        return;
    };
    if command == "produce" {
        let body = produce();
        let fixture = snapshot(&body);
        assert_eq!(fixture.restore(MAX_FIXTURE_BYTES).unwrap(), body);
        let polynomial = |slot, multiplier: i32| {
            let square = library::formal(slot).unwrap().pow(Atom::num(2));
            (Atom::num(multiplier) * square).as_view().get_data().to_vec()
        };
        let mut dependency = fixture.clone();
        dependency.atoms[1] = polynomial(1, 2);
        dependency.atoms[3] = polynomial(1, 2);
        let mut coefficient = fixture.clone();
        coefficient.atoms[1] = polynomial(GUARDED, 3);
        coefficient.atoms[3] = polynomial(0, 3);
        for (label, candidate) in [
            ("SEALED_FIXTURE", fixture),
            ("DEPENDENCY_SUBSTITUTION", dependency),
            ("COEFFICIENT_SUBSTITUTION", coefficient),
        ] {
            println!("{label}:{}", serde_json::to_string(&candidate).unwrap());
        }
        return;
    }
    let expected = ContentHash::parse_hex(&std::env::var(DIGEST_ENV).unwrap()).unwrap();
    assert!(
        crate::context().is_err(),
        "receiver must start with no initialized pool",
    );
    let original = Fixture::decode_original(&command, expected, command.len());
    if std::env::var(SUBSTITUTION_ENV).unwrap() == "1" {
        assert!(matches!(original, Err(MathError::Contract(reason))
            if reason == "sealed fixture original bytes differ"));
        assert!(crate::context().is_err());
        // Independently show the counterexample reaches past layout inspection.
        // This test-only witness neither admits nor copies its changed atoms.
        crate::initialize().unwrap();
        let unqualified: Fixture = serde_json::from_str(&command).unwrap();
        unqualified.inspect(MAX_FIXTURE_BYTES).unwrap();
        return;
    }
    assert!(matches!(
        Fixture::decode_original(&command, expected, command.len() - 1),
        Err(MathError::ByteLimit { resource: "sealed fixture payload", .. }),
    ));
    let fixture = original.unwrap();
    crate::initialize().unwrap();
    let registered = library::formal_pool_len().unwrap();
    let required: usize = fixture.atoms.iter().map(Vec::len).sum();
    assert!(matches!(
        fixture.restore(required - 1),
        Err(MathError::ByteLimit { .. }),
    ));
    let mut foreign = fixture.clone();
    foreign.ids.swap(0, 1);
    assert!(foreign.restore(required).is_err());
    let mut crossed = fixture.clone();
    crossed.atoms.swap(1, 3);
    assert!(crossed.restore(required).is_err());
    let mut malformed = fixture.clone();
    malformed.atoms[1].pop();
    assert!(malformed.restore(required).is_err());
    let mut unnormalized = fixture.clone();
    unnormalized.atoms[1][0] |= 0x80;
    assert!(unnormalized.restore(required).is_err());
    let mut large = fixture.clone();
    large.atoms[1] = symbolica::parse!("340282366920938463463374607431768211457")
        .as_view()
        .get_data()
        .to_vec();
    assert!(matches!(
        large.restore(MAX_FIXTURE_BYTES),
        Err(MathError::Contract(reason)) if reason == "sealed fixture only admits small real rationals",
    ));
    let restored = fixture.restore(required).unwrap();
    assert_eq!(library::formal_pool_len().unwrap(), registered);
    let scope = pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None);
    // The actual symbolic support consumer differentiates restored expressions
    // before any evaluator exists. Its work is separate from the copy allowance.
    let support = restored
        .prepare_support(&[0], &[0, 1], DerivativeOrder::Second, scope.cancellation())
        .unwrap();
    assert!(support.derivative_operations() > 0);
    assert_eq!(support.first_for_output(0), Some(&BTreeSet::from([0])));
    assert_eq!(support.second_for_output(0), Some(&BTreeSet::from([(0, 0)])));
    assert_eq!(support.support().controls, BTreeSet::from([0]));
    let mut worker = support
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &scope,
        )
        .unwrap()
        .worker();
    let mut providers = ProviderBindings::new();
    let evaluated = worker
        .evaluate(
            &[3.0, 11.0],
            DerivativeOrder::Second,
            &mut providers,
            scope.cancellation(),
        )
        .unwrap();
    assert_eq!(evaluated.values, vec![18.0]);
    assert_eq!(evaluated.jacobian, vec![12.0, 0.0]);
    assert_eq!(evaluated.hessians, vec![4.0, 0.0, 0.0, 0.0]);
    assert!(matches!(
        worker.evaluate(&[-3.0, 11.0], DerivativeOrder::Second, &mut providers, scope.cancellation()),
        Err(MathError::Domain { source_id, .. }) if source_id == occurrence(2),
    ));
}

#[test]
fn sealed_guarded_body_restores_after_typed_producer_retires() {
    let run = |input: &str, digest: Option<ContentHash>, substitution: bool| {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "execution::sealed_restore_tests::sealed_guarded_body_child",
                "--nocapture",
            ])
            .env(SNAPSHOT_ENV, input)
            .env(DIGEST_ENV, digest.map(ContentHash::to_hex).unwrap_or_default())
            .env(SUBSTITUTION_ENV, if substitution { "1" } else { "0" })
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let producer = run("produce", None, false);
    let frame = |label| {
        producer.lines().find_map(|line| line.strip_prefix(label)).unwrap()
    };
    let payload = frame("SEALED_FIXTURE:");
    let digest = preimage::hash(payload.as_bytes());
    run(payload, Some(digest), false);
    let original: Fixture = serde_json::from_str(payload).unwrap();
    for label in ["DEPENDENCY_SUBSTITUTION:", "COEFFICIENT_SUBSTITUTION:"] {
        let changed = frame(label);
        let fixture: Fixture = serde_json::from_str(changed).unwrap();
        assert_ne!(fixture.atoms[1], original.atoms[1]);
        assert_eq!(
            fixture.atoms.each_ref().map(Vec::len),
            original.atoms.each_ref().map(Vec::len),
        );
        run(changed, Some(digest), true);
    }
}
