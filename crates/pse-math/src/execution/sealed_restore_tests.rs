// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A narrow copy-only control, not a production recipe decoder. The test parent
//! acts as the custody owner of exact bytes from an identified typed producer.
use super::*;
use crate::typed::{Binary, BodyBuilder, BodyLimits};
use pse_quantity::{
    IndexSet, QuantityTypeId,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use serde::{Deserialize, Serialize};
use symbolica::atom::{
    AtomView, NamespacedSymbol,
    packing::{FloatEnvironment, PackingFrame, inspect_packing},
};

const INPUTS: usize = 1;
const SLOTS: usize = 3;
const SNAPSHOT_ENV: &str = "PSE_SYMBOLIC_TEST_SEALED_BODY";
const MAX_FIXTURE_BYTES: usize = 4096;
const FORMAL_NAMES: [&str; SLOTS] = ["pse_math::slot_0", "pse_math::slot_1", "pse_math::slot_2"];
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
    let mut x = builder
        .input(0, ids::quantity("neutral"), IndexSet::new(), occurrence(1))
        .unwrap();
    builder
        .require(
            &mut x,
            Condition::Positive,
            DerivativeOrder::Value,
            occurrence(2),
        )
        .unwrap();
    let out = builder
        .binary(Binary::Mul, x.clone(), x, None, occurrence(3))
        .unwrap();
    builder
        .prepare(&[out])
        .unwrap()
        .with_occurrence_tokens(BTreeSet::from([
            occurrence(1),
            occurrence(2),
            occurrence(3),
        ]))
        .unwrap()
}

// The snapshot represents exactly this fixture's two Blocks, one Require and
// one result. It is deliberately not a second general schema for guarded bodies.
#[derive(Clone, Serialize, Deserialize)]
struct Fixture {
    ids: [u32; SLOTS],
    atoms: [Vec<u8>; 4],
    quantity: [QuantityTypeId; 2],
    remaining: usize,
    occurrences: usize,
}

fn snapshot(body: &PreparedBody) -> Fixture {
    assert_eq!(
        (body.inputs, body.slots, body.outputs.as_slice()),
        (INPUTS, SLOTS, &[2][..]),
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
            argument: 1,
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
    assert_eq!(input_slots.as_slice(), &[1]);
    assert_eq!(result_slots.as_slice(), &[2]);
    assert_eq!(body.output_effects, Some(vec![BTreeSet::from([1])]));
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
        quantity: [body.input_quantities[0].unwrap(), body.output_quantities[0]],
        remaining: body.remaining_occurrences,
        occurrences: body.occurrences,
    }
}

impl Fixture {
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
                let required = self.atoms.iter().map(Vec::len).sum();
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
                    let expected_slot = usize::from(index == 1);
                    let summary = inspect_packing(bytes, &mut frames, FLOAT_SCAN, |symbol| {
                        if symbol.function || symbol.id != actual[expected_slot] {
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
                        || !matches!(shape, (0 | 2, AtomView::Var(_)) | (1 | 3, AtomView::Pow(_)))
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
                AtomView::Pow(view) => Ok(Atom::Pow(view.clone_into_raw(buffer))),
                _ => Err(MathError::Contract(
                    "sealed fixture expects a formal or power root".into(),
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
                outputs: vec![2],
                output_effects: Some(vec![BTreeSet::from([1])]),
                stages: vec![
                    Stage::Block {
                        expressions: vec![restore(input)?],
                        outputs: vec![1],
                        source: occurrence(2),
                    },
                    Stage::Require {
                        argument: 1,
                        condition: Condition::Positive,
                        order: DerivativeOrder::Value,
                        source: occurrence(2),
                        lineage: None,
                    },
                    Stage::Block {
                        expressions: vec![restore(result)?],
                        outputs: vec![2],
                        source: occurrence(3),
                    },
                ],
                remaining_occurrences: self.remaining,
                obligations: vec![(Some(restore(obligation)?), Condition::Positive)],
                input_quantities: vec![Some(self.quantity[0])],
                output_quantities: vec![self.quantity[1]],
                expressions: vec![Some(restore(expression)?)],
                providers: vec![],
                checked_members: BTreeSet::new(),
                checked_sources: BTreeSet::new(),
                occurrence_tokens: BTreeSet::from([occurrence(1), occurrence(2), occurrence(3)]),
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
        println!(
            "SEALED_FIXTURE:{}",
            serde_json::to_string(&fixture).unwrap()
        );
        return;
    }
    assert!(command.len() < MAX_FIXTURE_BYTES);
    let fixture: Fixture = serde_json::from_str(&command).unwrap();
    assert!(
        crate::context().is_err(),
        "receiver must start with no initialized pool",
    );
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
    let mut worker = restored
        .compile(
            &[0],
            &[0],
            DerivativeOrder::Second,
            Optimization::default(),
            EvaluationLimits::default(),
            &scope,
        )
        .unwrap()
        .worker();
    let mut providers = ProviderBindings::new();
    let evaluated = worker
        .evaluate(
            &[3.0],
            DerivativeOrder::Second,
            &mut providers,
            scope.cancellation(),
        )
        .unwrap();
    assert_eq!(evaluated.values, vec![9.0]);
    assert_eq!(evaluated.jacobian, vec![6.0]);
    assert_eq!(evaluated.hessians, vec![2.0]);
    assert!(matches!(
        worker.evaluate(&[-3.0], DerivativeOrder::Second, &mut providers, scope.cancellation()),
        Err(MathError::Domain { source_id, .. }) if source_id == occurrence(2),
    ));
}

#[test]
fn sealed_guarded_body_restores_after_typed_producer_retires() {
    let run = |input: &str| {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "execution::sealed_restore_tests::sealed_guarded_body_child",
                "--nocapture",
            ])
            .env(SNAPSHOT_ENV, input)
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
    let producer = run("produce");
    let payload = producer
        .lines()
        .find_map(|line| line.strip_prefix("SEALED_FIXTURE:"))
        .unwrap();
    assert!(payload.len() < MAX_FIXTURE_BYTES);
    run(payload);
}
