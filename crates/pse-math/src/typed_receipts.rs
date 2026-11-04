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
        entries: stream.entries.clone(),
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
                    .get(*position)
                    .ok_or_else(|| MathError::Contract("missing mathematical proof receipt".into()))
                    .and_then(|v| {
                        if v.request != request {
                            return Err(MathError::Contract(
                                "mathematical proof request differs from qualified receipt".into(),
                            ));
                        }
                        *position += 1;
                        Ok(v.branches.clone())
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
