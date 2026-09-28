// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Equality form of an NLP for POUNCE's ℓ1 exact penalty (ADR-0109).
//!
//! `pounce-l1penalty` relaxes equality rows only; inequality rows pass through it unrelaxed.
//! This wrapper presents each inequality row `g_L ≤ c(x) ≤ g_U` as the equality
//! `c(x) − s = 0` with a bounded slack `s ∈ [g_L, g_U]`, the form an interior-point method
//! uses internally, so the library's ℓ1 penalty measures the violation of every row. It adds
//! no objective terms and no reformulation of the rows themselves; the solution is truncated
//! back to the original variables before the inner problem receives it.
use pounce_nlp::tnlp::{
    BoundsInfo, IndexStyle, IpoptCq, IpoptData, IterStats, NlpInfo, Solution, SparsityRequest,
    StartingPoint, TNLP,
};
use std::{cell::RefCell, rc::Rc};

pub(super) struct Equalities {
    inner: Rc<RefCell<dyn TNLP>>,
    info: NlpInfo,
    /// Inequality rows, in order; slack `k` belongs to row `slacked[k]`.
    slacked: Vec<usize>,
    /// Inner row bounds.
    rows: Vec<(f64, f64)>,
}
impl Equalities {
    /// Wrap `inner`; `None` when the inner problem refuses its dimensions or bounds.
    pub(super) fn new(inner: Rc<RefCell<dyn TNLP>>) -> Option<Self> {
        let info = inner.borrow_mut().get_nlp_info()?;
        if info.index_style != IndexStyle::C || info.n < 0 || info.m < 0 {
            return None;
        }
        let (n, m) = (info.n as usize, info.m as usize);
        let (mut xl, mut xu, mut gl, mut gu) =
            (vec![0.0; n], vec![0.0; n], vec![0.0; m], vec![0.0; m]);
        if !inner.borrow_mut().get_bounds_info(BoundsInfo {
            x_l: &mut xl,
            x_u: &mut xu,
            g_l: &mut gl,
            g_u: &mut gu,
        }) {
            return None;
        }
        let rows: Vec<_> = gl.into_iter().zip(gu).collect();
        let slacked = rows
            .iter()
            .enumerate()
            .filter(|(_, (l, u))| l != u)
            .map(|(i, _)| i)
            .collect();
        Some(Self {
            inner,
            info,
            slacked,
            rows,
        })
    }
    fn n(&self) -> usize {
        self.info.n as usize
    }
    fn m(&self) -> usize {
        self.info.m as usize
    }
}
impl TNLP for Equalities {
    fn get_nlp_info(&mut self) -> Option<NlpInfo> {
        let k = i32::try_from(self.slacked.len()).ok()?;
        Some(NlpInfo {
            n: self.info.n.checked_add(k)?,
            m: self.info.m,
            nnz_jac_g: self.info.nnz_jac_g.checked_add(k)?,
            nnz_h_lag: self.info.nnz_h_lag,
            index_style: IndexStyle::C,
        })
    }
    fn get_bounds_info(&mut self, b: BoundsInfo<'_>) -> bool {
        let n = self.n();
        let (x_l, s_l) = b.x_l.split_at_mut(n);
        let (x_u, s_u) = b.x_u.split_at_mut(n);
        if !self.inner.borrow_mut().get_bounds_info(BoundsInfo {
            x_l,
            x_u,
            g_l: b.g_l,
            g_u: b.g_u,
        }) {
            return false;
        }
        for (k, &i) in self.slacked.iter().enumerate() {
            (s_l[k], s_u[k]) = self.rows[i];
            b.g_l[i] = 0.0;
            b.g_u[i] = 0.0;
        }
        true
    }
    fn get_starting_point(&mut self, sp: StartingPoint<'_>) -> bool {
        let n = self.n();
        let (x, slacks) = sp.x.split_at_mut(n);
        let (z_l, z_ls) = sp.z_l.split_at_mut(n);
        let (z_u, z_us) = sp.z_u.split_at_mut(n);
        if !self.inner.borrow_mut().get_starting_point(StartingPoint {
            init_x: sp.init_x,
            x,
            init_z: sp.init_z,
            z_l,
            z_u,
            init_lambda: sp.init_lambda,
            lambda: sp.lambda,
        }) {
            return false;
        }
        // Each slack starts at its row's value, moved into the row's interval.
        let mut g = vec![0.0; self.m()];
        if !self.inner.borrow_mut().eval_g(x, true, &mut g) {
            return false;
        }
        for (k, &i) in self.slacked.iter().enumerate() {
            let (l, u) = self.rows[i];
            slacks[k] = g[i].max(l).min(u);
        }
        z_ls.fill(0.0);
        z_us.fill(0.0);
        true
    }
    fn eval_f(&mut self, x: &[f64], new_x: bool) -> Option<f64> {
        self.inner.borrow_mut().eval_f(&x[..self.n()], new_x)
    }
    fn eval_grad_f(&mut self, x: &[f64], new_x: bool, grad_f: &mut [f64]) -> bool {
        let n = self.n();
        grad_f[n..].fill(0.0);
        self.inner
            .borrow_mut()
            .eval_grad_f(&x[..n], new_x, &mut grad_f[..n])
    }
    fn eval_g(&mut self, x: &[f64], new_x: bool, g: &mut [f64]) -> bool {
        let n = self.n();
        if !self.inner.borrow_mut().eval_g(&x[..n], new_x, g) {
            return false;
        }
        for (k, &i) in self.slacked.iter().enumerate() {
            g[i] -= x[n + k];
        }
        true
    }
    fn eval_jac_g(&mut self, x: Option<&[f64]>, new_x: bool, mode: SparsityRequest<'_>) -> bool {
        let (n, nnz) = (self.n(), self.info.nnz_jac_g as usize);
        let x = x.map(|x| &x[..n]);
        match mode {
            SparsityRequest::Structure { irow, jcol } => {
                let (inner_rows, rows) = irow.split_at_mut(nnz);
                let (inner_cols, cols) = jcol.split_at_mut(nnz);
                if !self.inner.borrow_mut().eval_jac_g(
                    x,
                    new_x,
                    SparsityRequest::Structure {
                        irow: inner_rows,
                        jcol: inner_cols,
                    },
                ) {
                    return false;
                }
                for (k, &i) in self.slacked.iter().enumerate() {
                    let (Ok(row), Ok(col)) = (i32::try_from(i), i32::try_from(n + k)) else {
                        return false;
                    };
                    rows[k] = row;
                    cols[k] = col;
                }
                true
            }
            SparsityRequest::Values { values } => {
                let (inner, slacks) = values.split_at_mut(nnz);
                slacks.fill(-1.0);
                self.inner.borrow_mut().eval_jac_g(
                    x,
                    new_x,
                    SparsityRequest::Values { values: inner },
                )
            }
        }
    }
    fn eval_h(
        &mut self,
        x: Option<&[f64]>,
        new_x: bool,
        obj_factor: f64,
        lambda: Option<&[f64]>,
        new_lambda: bool,
        mode: SparsityRequest<'_>,
    ) -> bool {
        // Slacks enter linearly: the Lagrangian Hessian is the inner one.
        let n = self.n();
        self.inner.borrow_mut().eval_h(
            x.map(|x| &x[..n]),
            new_x,
            obj_factor,
            lambda,
            new_lambda,
            mode,
        )
    }
    fn finalize_solution(&mut self, sol: Solution<'_>, ip_data: &IpoptData, ip_cq: &IpoptCq) {
        let n = self.n();
        let mut g = sol.g.to_vec();
        for (k, &i) in self.slacked.iter().enumerate() {
            if let Some(s) = sol.x.get(n + k) {
                g[i] += s;
            }
        }
        self.inner.borrow_mut().finalize_solution(
            Solution {
                status: sol.status,
                x: &sol.x[..n.min(sol.x.len())],
                z_l: &sol.z_l[..n.min(sol.z_l.len())],
                z_u: &sol.z_u[..n.min(sol.z_u.len())],
                g: &g,
                lambda: sol.lambda,
                obj_value: sol.obj_value,
            },
            ip_data,
            ip_cq,
        );
    }
    fn intermediate_callback(
        &mut self,
        stats: IterStats,
        ip_data: &IpoptData,
        ip_cq: &IpoptCq,
    ) -> bool {
        self.inner
            .borrow_mut()
            .intermediate_callback(stats, ip_data, ip_cq)
    }
}
