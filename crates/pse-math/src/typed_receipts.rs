// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete proof outcomes attached to a qualified selected numerical recipe.
//! No native atoms, symbols or proof engine state enter the durable description.
use crate::MathError;
use pse_ids::ContentHash;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Proof {
    request: ContentHash,
    branches: Vec<u8>,
}
/// Untrusted decoded proof data; only strict qualified replay consumes its authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofStream {
    version: u32,
    entries: Vec<Proof>,
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
        .map_err(|_| MathError::Contract("malformed mathematical receipt authority".into()))?
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
