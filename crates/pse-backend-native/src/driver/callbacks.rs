// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every C callback contains all Rust work inside the same unwind barrier.
use super::{Iteration, workspace::Workspace};
use crate::{NativeError, error::invalid};
use pse_ipopt_sys::UserDataPtr;
use std::panic::{AssertUnwindSafe, catch_unwind};

// SAFETY contract: data is the unique live Workspace passed to synchronous IpoptSolve.
// Ipopt invokes callbacks serially and never retains that pointer after Solve returns.
unsafe fn invoke(
    data: UserDataPtr,
    function: impl FnOnce(&mut Workspace) -> Result<(), NativeError>,
) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: caller supplies the exact unique workspace pointer described above.
    let workspace = unsafe { &mut *data.cast::<Workspace>() };
    match catch_unwind(AssertUnwindSafe(|| function(workspace))) {
        Ok(Ok(())) => true,
        Ok(Err(error)) => {
            workspace.failure = Some(error);
            false
        }
        Err(_) => {
            workspace.failure = Some(NativeError::CallbackPanic);
            false
        }
    }
}

// SAFETY contract: pointer refers to at least length live values supplied by Ipopt.
unsafe fn read<'a, T>(pointer: *const T, length: usize) -> Result<&'a [T], NativeError> {
    if length == 0 {
        return Ok(&[]);
    }
    if pointer.is_null() {
        return Err(invalid("null Ipopt callback input"));
    }
    // SAFETY: caller guarantees allocation, initialization, alignment and extent.
    Ok(unsafe { std::slice::from_raw_parts(pointer, length) })
}
// SAFETY contract: pointer refers to length uniquely borrowed outputs supplied by Ipopt.
unsafe fn write<'a, T>(pointer: *mut T, length: usize) -> Result<&'a mut [T], NativeError> {
    if length == 0 {
        return Ok(&mut []);
    }
    if pointer.is_null() {
        return Err(invalid("null Ipopt callback output"));
    }
    // SAFETY: caller guarantees allocation, alignment, extent and exclusive access.
    Ok(unsafe { std::slice::from_raw_parts_mut(pointer, length) })
}
fn extent(actual: i32, expected: usize) -> Result<(), NativeError> {
    if usize::try_from(actual).ok() == Some(expected) {
        Ok(())
    } else {
        Err(invalid(
            "Ipopt callback extent differs from the prepared problem",
        ))
    }
}

pub(super) unsafe extern "C" fn objective(
    n: i32,
    x: *mut f64,
    _: bool,
    value: *mut f64,
    data: UserDataPtr,
) -> bool {
    let callback = |w: &mut Workspace| {
        extent(n, w.positions.len())?;
        // SAFETY: n is verified against the C problem and x has that extent.
        let x = unsafe { read(x, w.positions.len()) }?;
        w.evaluate(x)?;
        // SAFETY: Ipopt supplies one objective destination.
        let output = unsafe { write(value, 1) }?;
        output[0] = w.residual(0);
        Ok(())
    };
    // SAFETY: registered callback receives the exact live workspace.
    unsafe { invoke(data, callback) }
}
pub(super) unsafe extern "C" fn gradient(
    n: i32,
    x: *mut f64,
    _: bool,
    values: *mut f64,
    data: UserDataPtr,
) -> bool {
    let callback = |w: &mut Workspace| {
        extent(n, w.positions.len())?;
        // SAFETY: Ipopt supplies n initialized decision values.
        let x = unsafe { read(x, w.positions.len()) }?;
        w.evaluate(x)?;
        // SAFETY: Ipopt supplies n uniquely borrowed gradient destinations.
        let values = unsafe { write(values, w.positions.len()) }?;
        values.fill(0.0);
        for &(output, column) in &w.objective_gradient {
            values[column] = w.derivative(output);
        }
        Ok(())
    };
    // SAFETY: registered callback receives the exact live workspace.
    unsafe { invoke(data, callback) }
}
pub(super) unsafe extern "C" fn constraints(
    n: i32,
    x: *mut f64,
    _: bool,
    m: i32,
    values: *mut f64,
    data: UserDataPtr,
) -> bool {
    let callback = |w: &mut Workspace| {
        extent(n, w.positions.len())?;
        extent(m, w.program.residual_count() - 1)?;
        // SAFETY: Ipopt supplies n initialized decision values.
        let x = unsafe { read(x, w.positions.len()) }?;
        w.evaluate(x)?;
        // SAFETY: Ipopt supplies m uniquely borrowed constraint destinations.
        let values = unsafe { write(values, w.program.residual_count() - 1) }?;
        for (row, value) in values.iter_mut().enumerate() {
            *value = w.residual(row + 1);
        }
        Ok(())
    };
    // SAFETY: registered callback receives the exact live workspace.
    unsafe { invoke(data, callback) }
}
pub(super) unsafe extern "C" fn jacobian(
    n: i32,
    x: *mut f64,
    _: bool,
    m: i32,
    nnz: i32,
    rows: *mut i32,
    columns: *mut i32,
    values: *mut f64,
    data: UserDataPtr,
) -> bool {
    let callback = |w: &mut Workspace| {
        extent(n, w.positions.len())?;
        extent(m, w.program.residual_count() - 1)?;
        extent(nnz, w.jacobian.len())?;
        w.cancel.checkpoint()?;
        if values.is_null() {
            // SAFETY: structure request supplies nnz row destinations.
            let rows = unsafe { write(rows, w.jacobian.len()) }?;
            // SAFETY: structure request supplies separate nnz column destinations.
            let columns = unsafe { write(columns, w.jacobian.len()) }?;
            for ((row, column), &(_, r, c)) in rows.iter_mut().zip(columns).zip(&w.jacobian) {
                *row = r;
                *column = c;
            }
        } else {
            // SAFETY: value request supplies n initialized decision values.
            let x = unsafe { read(x, w.positions.len()) }?;
            w.evaluate(x)?;
            // SAFETY: value request supplies nnz uniquely borrowed destinations.
            let values = unsafe { write(values, w.jacobian.len()) }?;
            for (value, &(output, _, _)) in values.iter_mut().zip(&w.jacobian) {
                *value = w.derivative(output);
            }
        }
        Ok(())
    };
    // SAFETY: registered callback receives the exact live workspace.
    unsafe { invoke(data, callback) }
}
pub(super) unsafe extern "C" fn hessian(
    _: i32,
    _: *mut f64,
    _: bool,
    _: f64,
    _: i32,
    _: *mut f64,
    _: bool,
    _: i32,
    _: *mut i32,
    _: *mut i32,
    _: *mut f64,
    data: UserDataPtr,
) -> bool {
    // SAFETY: this callback can only run if Ipopt violates the explicit selected
    // limited-memory profile. It refuses rather than inventing a Hessian.
    unsafe {
        invoke(data, |_| {
            Err(NativeError::Capability(
                "exact Hessian was not selected or implemented".into(),
            ))
        })
    }
}
pub(super) unsafe extern "C" fn intermediate(
    mode: i32,
    index: i32,
    objective: f64,
    primal: f64,
    dual: f64,
    barrier: f64,
    _: f64,
    _: f64,
    _: f64,
    step: f64,
    _: i32,
    data: UserDataPtr,
) -> bool {
    // SAFETY: this callback receives the exact workspace and only scalar C values.
    unsafe {
        invoke(data, |w| {
            w.cancel.checkpoint()?;
            if w.iterations.len() == w.iteration_capacity {
                return Err(invalid(
                    "Ipopt iteration history exceeded its reserved extent",
                ));
            }
            w.iterations.push(Iteration {
                index,
                restoration: mode == 1,
                objective,
                primal_infeasibility: primal,
                dual_infeasibility: dual,
                barrier,
                step,
            });
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_workspace_is_refused_without_dereference() {
        // SAFETY: invoke explicitly refuses a null workspace before any access.
        assert!(!unsafe { invoke(std::ptr::null_mut(), |_| Ok(())) });
    }
}
