// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#ifndef PSE_IPOPT_SEQUENCE_H
#define PSE_IPOPT_SEQUENCE_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
#define PSE_IPOPT_NOEXCEPT noexcept
extern "C" {
#else
#define PSE_IPOPT_NOEXCEPT
#endif
/* Foreign bridge status, separate from Ipopt ApplicationReturnStatus. */
enum pse_ipopt_sequence_status { PSE_IPOPT_SEQUENCE_OK = 0, PSE_IPOPT_SEQUENCE_INVALID = 1,
    PSE_IPOPT_SEQUENCE_EXCEPTION = 2, PSE_IPOPT_SEQUENCE_MEMORY = 3,
    PSE_IPOPT_SEQUENCE_TERMINAL = 4, PSE_IPOPT_SEQUENCE_CANCEL = 5,
    PSE_IPOPT_SEQUENCE_DEADLINE = 6, PSE_IPOPT_SEQUENCE_ABANDON = 7,
    PSE_IPOPT_SEQUENCE_NO_RESULT = 8 };
/* Evaluations: 0 success, 1 recoverable trial, 2 terminal. Poll: 0 continue,
 * 1 cancelled, 2 absolute deadline, 3 abandoned, 4 terminal typed scope cause. */
typedef int32_t (*pse_ipopt_sequence_objective)(int32_t, const double*, double*, void*);
typedef int32_t (*pse_ipopt_sequence_gradient)(int32_t, const double*, double*, void*);
typedef int32_t (*pse_ipopt_sequence_constraints)(int32_t, int32_t, const double*, double*, void*);
typedef int32_t (*pse_ipopt_sequence_jacobian)(int32_t, int32_t, const double*, double*, void*);
typedef int32_t (*pse_ipopt_sequence_hessian)(int32_t, int32_t, int32_t, const double*, double,
    const double*, double*, void*);
typedef int32_t (*pse_ipopt_sequence_poll)(void*);
/* Current iterate/gradient arrays are nullable when the native query cannot
 * provide them, otherwise n readable values valid only during this callback. */
typedef int32_t (*pse_ipopt_sequence_iteration)(int32_t mode, int32_t iteration, double objective,
    double primal, double dual, double barrier, double step, double regularization,
    double alpha_dual, double alpha_primal, int32_t line_search_trials, int32_t n,
    const double* current_primal, const double* lagrangian_gradient, void*);
typedef struct {
    pse_ipopt_sequence_objective objective;
    pse_ipopt_sequence_gradient gradient;
    pse_ipopt_sequence_constraints constraints;
    pse_ipopt_sequence_jacobian jacobian;
    pse_ipopt_sequence_hessian hessian;
    pse_ipopt_sequence_poll poll;
    pse_ipopt_sequence_iteration iteration;
    void* user;
} pse_ipopt_sequence_callbacks;
typedef struct pse_ipopt_sequence_handle pse_ipopt_sequence_handle;
typedef struct { int32_t application_status; int32_t solver_status;
    int32_t iterations; int32_t reoptimized; int32_t iterations_available; int32_t barrier_available; int32_t candidate_available;
    double objective; double barrier; } pse_ipopt_sequence_result;
/* Structure, bounds and callbacks are copied and immutable. Rust context must
 * outlive destroy. COO is unique, zero-based; Hessian uses the lower triangle.
 * Caller owns exclusive access and admits semantic/profile/structure compatibility
 * before retaining/reusing a handle. No foreign object crosses this boundary. */
int32_t pse_ipopt_sequence_create(int32_t n, int32_t m, int32_t jac_nnz, int32_t hess_nnz,
    const double* lower, const double* upper, const double* row_lower, const double* row_upper,
    const int32_t* jac_rows, const int32_t* jac_columns, const int32_t* hess_rows, const int32_t* hess_columns,
    const pse_ipopt_sequence_callbacks*, pse_ipopt_sequence_handle**, char*, size_t) PSE_IPOPT_NOEXCEPT;
/* Structural native options must be set before the first solve. The six numeric
 * restart options may change between attempts after reset_restart. Sequence-owned
 * warm_start_same_structure, warm_start_init_point, max_wall_time and option-file
 * controls cannot be set here. Remaining wall time belongs to each task scope. */
int32_t pse_ipopt_sequence_set_string(pse_ipopt_sequence_handle*, const char*, const char*, char*, size_t) PSE_IPOPT_NOEXCEPT;
int32_t pse_ipopt_sequence_set_double(pse_ipopt_sequence_handle*, const char*, double, char*, size_t) PSE_IPOPT_NOEXCEPT;
int32_t pse_ipopt_sequence_set_integer(pse_ipopt_sequence_handle*, const char*, int32_t, char*, size_t) PSE_IPOPT_NOEXCEPT;
/* Reset only the six per-attempt restart options to their registered defaults. */
int32_t pse_ipopt_sequence_reset_restart(pse_ipopt_sequence_handle*, char*, size_t) PSE_IPOPT_NOEXCEPT;
/* reoptimize=0 requires a fresh handle; 1 requires a prior successful OptimizeTNLP
 * on this exact TNLP/application. Initial primal is explicit and finite; Ipopt
 * owns its adjustment into the native bounds. Optional
 * duals must be supplied together; no hidden replacement iterate is synthesized. */
int32_t pse_ipopt_sequence_solve(pse_ipopt_sequence_handle*, int32_t reoptimize, double remaining_seconds,
    int32_t n, const double* primal, const double* lower_dual, const double* upper_dual,
    int32_t m, const double* row_dual, char*, size_t) PSE_IPOPT_NOEXCEPT;
int32_t pse_ipopt_sequence_get_result(pse_ipopt_sequence_handle*, pse_ipopt_sequence_result*, char*, size_t) PSE_IPOPT_NOEXCEPT;
int32_t pse_ipopt_sequence_get_vectors(pse_ipopt_sequence_handle*, int32_t n, double* primal,
    double* lower_dual, double* upper_dual, int32_t m, double* constraints, double* row_dual,
    char*, size_t) PSE_IPOPT_NOEXCEPT;
int32_t pse_ipopt_sequence_destroy(pse_ipopt_sequence_handle**, char*, size_t) PSE_IPOPT_NOEXCEPT;
#ifdef __cplusplus
}
#endif
#undef PSE_IPOPT_NOEXCEPT
#endif
