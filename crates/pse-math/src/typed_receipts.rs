// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete proof outcomes attached to a qualified selected numerical recipe.
//! No native atoms, symbols or proof engine state enter the durable description.
use crate::MathError;
use pse_ids::ContentHash;
use serde::{Deserialize, Serialize};
use std::{alloc::Layout, cell::RefCell};
pse_contract::projection_struct! {
/// Passive complete mathematical receipt. Its fields convey no replay permission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    projection: { name: "MathematicalProofReceipt", rust_path: crate::typed::receipts::Proof,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    request: ContentHash => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    branches: Vec<u8> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Untrusted decoded proof data; only strict qualified replay consumes its authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofStream {
    projection: { name: "MathematicalProofStream", rust_path: crate::typed::receipts::ProofStream,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    version: u32 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: 1 }] };
    entries: Vec<Proof> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl ProofStream {
    /// Complete owned proof receipt extent, excluding native proof machinery.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.entries.capacity() * size_of::<Proof>()
            + self
                .entries
                .iter()
                .map(|v| v.branches.capacity())
                .sum::<usize>()
    }
}
enum State {
    Capture {
        depth: usize,
        entries: Vec<Proof>,
    },
    Replay {
        entries: Vec<Proof>,
        position: usize,
    },
}
thread_local! {static SESSION:RefCell<Option<State>>=const {RefCell::new(None)};}
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        SESSION.with(|s| *s.borrow_mut() = None);
    }
}
fn begin(state: State) -> Result<Reset, MathError> {
    SESSION.with(|s| {
        let mut s = s.borrow_mut();
        if s.is_some() {
            return Err(MathError::Contract(
                "nested mathematical proof receipt session".into(),
            ));
        }
        *s = Some(state);
        Ok(Reset)
    })
}
/// Capture successful bounded proofs during normal admission.
pub fn capture<T>(f: impl FnOnce() -> Result<T, MathError>) -> Result<(T, ProofStream), MathError> {
    let reset = begin(State::Capture {
        depth: 0,
        entries: Vec::new(),
    })?;
    let v = f()?;
    let entries = SESSION.with(|s| match s.borrow_mut().take() {
        Some(State::Capture { entries, .. }) => entries,
        _ => Vec::new(),
    });
    drop(reset);
    Ok((
        v,
        ProofStream {
            version: 1,
            entries,
        },
    ))
}
/// Consume all exact successful proof outcomes without entering the original prover.
pub fn replay<T>(
    stream: &ProofStream,
    authority: &pse_ids::scientific_replay::RecordAuthority,
    f: impl FnOnce() -> Result<T, MathError>,
) -> Result<T, MathError> {
    if !authority
        .admits(stream)
        .map_err(pse_quantity::QuantityError::RecordMembership)?
    {
        return Err(MathError::Contract(
            "mathematical proof receipts are absent from qualified recipe".into(),
        ));
    }
    if stream.version != 1 {
        return Err(MathError::Contract(
            "unsupported mathematical proof receipts".into(),
        ));
    }
    let reset = begin(State::Replay {
        entries: replay_entries(stream)?,
        position: 0,
    })?;
    let v = f()?;
    let complete=SESSION.with(|s|matches!(&*s.borrow(),Some(State::Replay {entries,position}) if *position==entries.len()));
    drop(reset);
    if !complete {
        return Err(MathError::Contract(
            "unconsumed mathematical proof receipts".into(),
        ));
    }
    Ok(v)
}

/// Charge the complete requested backing before copying any replay state. The
/// original stream and this copy overlap; the charge follows the enclosing
/// construction owner's existing lifetime, without a replay-local release.
/// Allocator-private overhead/rounding and membership scratch are separate.
fn replay_entries(stream: &ProofStream) -> Result<Vec<Proof>, MathError> {
    let overflow = || MathError::Limit("mathematical proof receipt allocation overflow");
    let mut bytes = Layout::array::<Proof>(stream.entries.len())
        .map_err(|_| overflow())?
        .size();
    for receipt in &stream.entries {
        bytes = bytes
            .checked_add(
                Layout::array::<u8>(receipt.branches.len())
                    .map_err(|_| overflow())?
                    .size(),
            )
            .ok_or_else(overflow)?;
    }
    crate::construction::additional(bytes)?;
    let allocation = || MathError::Limit("mathematical proof receipt allocation");
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(stream.entries.len())
        .map_err(|_| allocation())?;
    for receipt in &stream.entries {
        let mut branches = Vec::new();
        branches
            .try_reserve_exact(receipt.branches.len())
            .map_err(|_| allocation())?;
        branches.extend_from_slice(&receipt.branches);
        entries.push(Proof {
            request: receipt.request,
            branches,
        });
    }
    Ok(entries)
}
/// Dispatch one source/contract-bound proof. Only its outcome and branch capabilities
/// persist; the caller's selected recipe authenticates all authored expressions.
pub fn proof(
    request: ContentHash,
    calculate: impl FnOnce() -> Result<Vec<u8>, MathError>,
) -> Result<Vec<u8>, MathError> {
    if !SESSION.with(|s| s.borrow().is_some()) {
        return calculate();
    }
    let replay = SESSION.with(|s| {
        let mut s = s.borrow_mut();
        match s.as_mut() {
            Some(State::Replay { entries, position }) => Some(
                entries
                    .get_mut(*position)
                    .ok_or_else(|| MathError::Contract("missing mathematical proof receipt".into()))
                    .and_then(|v| {
                        if v.request != request {
                            return Err(MathError::Contract(
                                "mathematical proof request differs from qualified receipt".into(),
                            ));
                        }
                        *position += 1;
                        // This receipt is consumed exactly once. Transfer its
                        // already admitted backing rather than allocating a
                        // second branch copy while replay state is retained.
                        Ok(std::mem::take(&mut v.branches))
                    }),
            ),
            Some(State::Capture { depth, .. }) => {
                *depth += 1;
                None
            }
            None => None,
        }
    });
    if let Some(v) = replay {
        return v;
    }
    let v = pse_quantity::resolved::receipts::without_capture(calculate);
    SESSION.with(|s| {
        if let Some(State::Capture { depth, entries }) = s.borrow_mut().as_mut() {
            *depth -= 1;
            if *depth == 0
                && let Ok(branches) = &v
            {
                entries.push(Proof {
                    request,
                    branches: branches.clone(),
                });
            }
        }
    });
    v
}
/// Original provers must fail closed if accidentally entered during strict replay.
pub(super) fn require_admission() -> Result<(), MathError> {
    if SESSION.with(|s| matches!(&*s.borrow(), Some(State::Replay { .. }))) {
        Err(MathError::Contract(
            "mathematical proof is forbidden during reconstruction".into(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod projection_tests {
    use super::*;
    use pse_contract::boundary_projection::{
        ConstructionRole, ProjectionGraph, ProjectionRecord, ValidateProjection,
    };

    #[test]
    fn mathematical_receipt_projection_is_data_and_does_not_start_replay() {
        let request = ContentHash::from_bytes([27; 32]);
        let stream = ProofStream {
            version: 1,
            entries: vec![Proof {
                request,
                branches: vec![7, 2],
            }],
        };
        let original = serde_json::to_vec(&stream).unwrap();
        let reconstructed =
            ProofStream::from_projection_fields(stream.into_projection_fields()).unwrap();
        assert_eq!(serde_json::to_vec(&reconstructed).unwrap(), original);
        let graph = ProjectionGraph::of::<ProofStream>().unwrap();
        for name in ["MathematicalProofStream", "MathematicalProofReceipt"] {
            assert_eq!(
                graph.definitions()[name].construction,
                ConstructionRole::Data
            );
        }
        // Even complete decoded original receipt data does not install a replay session.
        assert_eq!(proof(request, || Ok(vec![9])).unwrap(), vec![9]);
        assert!(
            ProofStream {
                version: 2,
                entries: vec![]
            }
            .validate_projection()
            .is_err()
        );
    }
}

#[cfg(test)]
mod replay_tests {
    use super::*;
    use crate::construction::{ConstructionAdmission, scoped};
    use pse_ids::scientific_replay::{
        QualifiedScientificRecipe, RecordAuthority, admitted_recipe_payload_hash,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Debug)]
    struct Admission {
        allowance: usize,
        attempts: AtomicUsize,
        charged: AtomicUsize,
    }
    impl Admission {
        fn new(allowance: usize) -> Arc<Self> {
            Arc::new(Self {
                allowance,
                attempts: AtomicUsize::new(0),
                charged: AtomicUsize::new(0),
            })
        }
    }
    impl ConstructionAdmission for Admission {
        fn try_grow(&self, bytes: usize) -> Result<(), MathError> {
            self.attempts.fetch_add(1, Ordering::SeqCst);
            if bytes > self.allowance {
                return Err(MathError::ByteLimit {
                    resource: "test proof receipt backing",
                    required: bytes,
                    available: self.allowance,
                });
            }
            self.charged.fetch_add(bytes, Ordering::SeqCst);
            Ok(())
        }
    }

    fn stream() -> ProofStream {
        ProofStream {
            version: 1,
            entries: vec![
                Proof {
                    request: ContentHash::from_bytes([21; 32]),
                    branches: vec![7, 2],
                },
                Proof {
                    request: ContentHash::from_bytes([23; 32]),
                    branches: vec![1, 4, 9],
                },
            ],
        }
    }

    #[allow(
        unsafe_code,
        reason = "isolated receipt replay fixture controls exact original stream membership; no production authority"
    )]
    fn authority(stream: &ProofStream) -> RecordAuthority {
        let payload = serde_json::to_vec(stream).unwrap();
        // SAFETY: this private fixture owns the exact original receipt population;
        // the permit is used only to test replay, never as scientific admission.
        unsafe {
            QualifiedScientificRecipe::from_canonical_product(
                payload.clone(),
                admitted_recipe_payload_hash(&payload),
                ContentHash::from_bytes([29; 32]),
                ContentHash::from_bytes([31; 32]),
            )
        }
        .unwrap()
        .records()
        .clone()
    }

    fn recorded(request: ContentHash) -> Result<Vec<u8>, MathError> {
        proof(request, || {
            panic!("qualified replay must not enter the prover")
        })
    }

    #[test]
    fn mathematical_receipt_replay_admits_source_sized_backing_and_moves_consumed_branches() {
        let mut stream = stream();
        // Retained source spare capacity is not copied into the replay population.
        stream.entries.reserve(17);
        stream.entries[0].branches.reserve(101);
        let original = serde_json::to_vec(&stream).unwrap();
        let authority = authority(&stream);
        let expected = 2 * size_of::<Proof>() + 5;
        let admission = Admission::new(expected);
        let results = scoped(admission.clone(), || {
            replay(&stream, &authority, || {
                assert_eq!(admission.charged.load(Ordering::SeqCst), expected);
                let backing = SESSION.with(|session| match &*session.borrow() {
                    Some(State::Replay { entries, .. }) => entries[0].branches.as_ptr(),
                    _ => panic!("expected replay state"),
                });
                let first = recorded(stream.entries[0].request)?;
                assert_eq!(first.as_ptr(), backing);
                let second = recorded(stream.entries[1].request)?;
                Ok((first, second))
            })
        })
        .unwrap();
        assert_eq!(results, (vec![7, 2], vec![1, 4, 9]));
        assert_eq!(admission.attempts.load(Ordering::SeqCst), 1);
        // Consuming the receipts does not return the enclosing owner's charge.
        assert_eq!(admission.charged.load(Ordering::SeqCst), expected);
        assert_eq!(serde_json::to_vec(&stream).unwrap(), original);
    }

    #[test]
    fn mathematical_receipt_replay_refuses_backing_before_starting_reconstruction() {
        let stream = stream();
        let authority = authority(&stream);
        let expected = 2 * size_of::<Proof>() + 5;
        let admission = Admission::new(expected - 1);
        let result: Result<(), _> = scoped(admission.clone(), || {
            replay(&stream, &authority, || {
                panic!("refused replay must not start reconstruction")
            })
        });
        assert!(matches!(
            result,
            Err(MathError::ByteLimit {
                resource: "test proof receipt backing",
                required,
                available,
            }) if required == expected && available == expected - 1
        ));
        assert_eq!(admission.attempts.load(Ordering::SeqCst), 1);
        assert_eq!(admission.charged.load(Ordering::SeqCst), 0);
        assert!(SESSION.with(|session| session.borrow().is_none()));
        assert_eq!(
            proof(stream.entries[0].request, || Ok(vec![8])).unwrap(),
            vec![8]
        );
    }

    #[test]
    fn mathematical_receipt_replay_keeps_membership_version_order_and_complete_drain() {
        let stream = stream();
        let authority = authority(&stream);
        let admission = Admission::new(usize::MAX);
        let mut changed = stream.clone();
        changed.entries[0].branches[0] = 3;
        let membership = scoped(admission.clone(), || {
            replay(&changed, &authority, || Ok(()))
        });
        assert!(matches!(membership, Err(MathError::Contract(message))
            if message == "mathematical proof receipts are absent from qualified recipe"));
        assert_eq!(admission.attempts.load(Ordering::SeqCst), 0);

        let mut old = stream.clone();
        old.version = 2;
        let version = replay(&old, &self::authority(&old), || Ok(()));
        assert!(matches!(version, Err(MathError::Contract(message))
            if message == "unsupported mathematical proof receipts"));

        let reordered = replay(&stream, &authority, || recorded(stream.entries[1].request));
        assert!(matches!(reordered, Err(MathError::Contract(message))
            if message == "mathematical proof request differs from qualified receipt"));

        let extra = replay(&stream, &authority, || recorded(stream.entries[0].request));
        assert!(matches!(extra, Err(MathError::Contract(message))
            if message == "unconsumed mathematical proof receipts"));

        let missing = replay(&stream, &authority, || {
            recorded(stream.entries[0].request)?;
            recorded(stream.entries[1].request)?;
            recorded(ContentHash::from_bytes([33; 32]))
        });
        assert!(matches!(missing, Err(MathError::Contract(message))
            if message == "missing mathematical proof receipt"));

        let failed: Result<(), _> = replay(&stream, &authority, || {
            recorded(stream.entries[0].request)?;
            Err(MathError::Cancelled)
        });
        assert!(matches!(failed, Err(MathError::Cancelled)));
        assert!(SESSION.with(|session| session.borrow().is_none()));
        replay(&stream, &authority, || {
            assert_eq!(recorded(stream.entries[0].request)?, [7, 2]);
            assert_eq!(recorded(stream.entries[1].request)?, [1, 4, 9]);
            Ok(())
        })
        .unwrap();
    }
}
