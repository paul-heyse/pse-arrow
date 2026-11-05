// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete proof outcomes attached to a qualified selected numerical recipe.
//! No native atoms, symbols or proof engine state enter the durable description.
use crate::MathError;
use pse_ids::ContentHash;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
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
/// Consume the original proof backing without a receipt copy or original prover call.
/// Membership scratch and downstream reconstruction retain their own admission.
pub fn replay<T>(
    stream: ProofStream,
    authority: &pse_ids::scientific_replay::RecordAuthority,
    f: impl FnOnce() -> Result<T, MathError>,
) -> Result<T, MathError> {
    if !authority
        .admits(&stream)
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
        entries: stream.entries,
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
                        // original backing rather than allocating a
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
    struct DenyGrowth(AtomicUsize);
    impl ConstructionAdmission for DenyGrowth {
        fn try_grow(&self, _: usize) -> Result<(), MathError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(MathError::Limit("unexpected proof receipt backing growth"))
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
        let digest = admitted_recipe_payload_hash(&payload);
        // SAFETY: this private fixture owns the exact original receipt population;
        // the permit is used only to test replay, never as scientific admission.
        unsafe {
            QualifiedScientificRecipe::from_canonical_product(
                payload,
                digest,
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

    fn qualified_stream() -> (ProofStream, RecordAuthority, [ContentHash; 2]) {
        let stream = stream();
        let authority = authority(&stream);
        let requests = [stream.entries[0].request, stream.entries[1].request];
        (stream, authority, requests)
    }

    #[test]
    fn mathematical_receipt_replay_moves_original_backing_under_deny_growth() {
        let mut stream = stream();
        // Retain even spare capacity by ownership, without reserving a new copy.
        stream.entries.reserve(17);
        stream.entries[0].branches.reserve(101);
        stream.entries[1].branches.reserve(103);
        let authority = authority(&stream);
        let requests = [stream.entries[0].request, stream.entries[1].request];
        let entries_backing = (stream.entries.as_ptr(), stream.entries.capacity());
        let branch_backing = stream
            .entries
            .iter()
            .map(|receipt| (receipt.branches.as_ptr(), receipt.branches.capacity()))
            .collect::<Vec<_>>();
        let admission = Arc::new(DenyGrowth(AtomicUsize::new(0)));
        let results = scoped(admission.clone(), || {
            replay(stream, &authority, || {
                SESSION.with(|session| match &*session.borrow() {
                    Some(State::Replay { entries, .. }) => {
                        assert_eq!((entries.as_ptr(), entries.capacity()), entries_backing);
                        for (receipt, expected) in entries.iter().zip(&branch_backing) {
                            assert_eq!(
                                (receipt.branches.as_ptr(), receipt.branches.capacity()),
                                *expected
                            );
                        }
                    }
                    _ => panic!("expected replay state"),
                });
                let first = recorded(requests[0])?;
                let second = recorded(requests[1])?;
                Ok((first, second))
            })
        })
        .unwrap();
        assert_eq!(results, (vec![7, 2], vec![1, 4, 9]));
        for (result, expected) in [&results.0, &results.1].iter().zip(&branch_backing) {
            assert_eq!((result.as_ptr(), result.capacity()), *expected);
        }
        // This scope exercises receipt entry/consumption only; membership scratch
        // and real downstream constructors retain separate admission obligations.
        assert_eq!(admission.0.load(Ordering::SeqCst), 0);
        assert!(SESSION.with(|session| session.borrow().is_none()));
    }

    #[test]
    fn mathematical_receipt_replay_keeps_membership_version_order_and_complete_drain() {
        let (mut changed, authority, _) = qualified_stream();
        changed.entries[0].branches[0] = 3;
        let membership = replay(changed, &authority, || Ok(()));
        assert!(matches!(membership, Err(MathError::Contract(message))
            if message == "mathematical proof receipts are absent from qualified recipe"));
        let mut old = stream();
        old.version = 2;
        let old_authority = self::authority(&old);
        let version = replay(old, &old_authority, || Ok(()));
        assert!(matches!(version, Err(MathError::Contract(message))
            if message == "unsupported mathematical proof receipts"));

        let (stream, authority, requests) = qualified_stream();
        let reordered = replay(stream, &authority, || recorded(requests[1]));
        assert!(matches!(reordered, Err(MathError::Contract(message))
            if message == "mathematical proof request differs from qualified receipt"));

        let (stream, authority, requests) = qualified_stream();
        let extra = replay(stream, &authority, || recorded(requests[0]));
        assert!(matches!(extra, Err(MathError::Contract(message))
            if message == "unconsumed mathematical proof receipts"));

        let (stream, authority, requests) = qualified_stream();
        let missing = replay(stream, &authority, || {
            recorded(requests[0])?;
            recorded(requests[1])?;
            recorded(ContentHash::from_bytes([33; 32]))
        });
        assert!(matches!(missing, Err(MathError::Contract(message))
            if message == "missing mathematical proof receipt"));

        let (stream, authority, requests) = qualified_stream();
        let failed: Result<(), _> = replay(stream, &authority, || {
            recorded(requests[0])?;
            Err(MathError::Cancelled)
        });
        assert!(matches!(failed, Err(MathError::Cancelled)));
        assert!(SESSION.with(|session| session.borrow().is_none()));
        let (stream, authority, requests) = qualified_stream();
        replay(stream, &authority, || {
            assert_eq!(recorded(requests[0])?, [7, 2]);
            assert_eq!(recorded(requests[1])?, [1, 4, 9]);
            Ok(())
        })
        .unwrap();
    }
}
