// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#ifndef PSE_UNO_H
#define PSE_UNO_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
#define PSE_UNO_NOEXCEPT noexcept
extern "C" {
#else
#define PSE_UNO_NOEXCEPT
#endif
/* Bridge errors are independent of Uno optimization and iterate statuses. */
enum pse_uno_status {
    PSE_UNO_OK = 0, PSE_UNO_INVALID = 1, PSE_UNO_EXCEPTION = 2,
    PSE_UNO_MEMORY = 3, PSE_UNO_CALLBACK_TERMINAL = 4, PSE_UNO_CANCELLED = 5,
    PSE_UNO_DEADLINE = 6, PSE_UNO_ABANDONED = 7, PSE_UNO_NO_RESULT = 8
};
enum pse_uno_profile { PSE_UNO_SQP_LBFGS = 0, PSE_UNO_SLP = 1 };
/* Rust owns original typed causes and catches panics before returning. */
enum pse_uno_evaluation { PSE_UNO_EVAL_OK = 0, PSE_UNO_EVAL_TRIAL = 1, PSE_UNO_EVAL_TERMINAL = 2 };
enum pse_uno_scope { PSE_UNO_SCOPE_CONTINUE = 0, PSE_UNO_SCOPE_CANCEL = 1,
    PSE_UNO_SCOPE_DEADLINE = 2, PSE_UNO_SCOPE_ABANDON = 3, PSE_UNO_SCOPE_TERMINAL = 4 };
typedef struct pse_uno_handle pse_uno_handle;
typedef int32_t (*pse_uno_objective)(int32_t, const double*, double*, void*);
typedef int32_t (*pse_uno_constraints)(int32_t, int32_t, const double*, double*, void*);
typedef int32_t (*pse_uno_gradient)(int32_t, const double*, double*, void*);
typedef int32_t (*pse_uno_jacobian)(int32_t, int32_t, const double*, double*, void*);
typedef int32_t (*pse_uno_poll)(void*);
typedef struct {
    pse_uno_objective objective;
    pse_uno_constraints constraints;
    pse_uno_gradient gradient;
    pse_uno_jacobian jacobian;
    pse_uno_poll poll;
    void* user;
} pse_uno_callbacks;
typedef struct {
    int32_t profile;
    int32_t max_iterations;
    int32_t lbfgs_memory; /* >0 for SQP; must be 0 (absent) for SLP */
    double remaining_seconds;
    double primal_tolerance;
    double dual_tolerance;
    double trust_radius;
    uint64_t materialization_limit_bytes; /* positive finite foreign allowance for SQP */
} pse_uno_config;
typedef struct {
    int32_t optimization_status;
    int32_t iterate_status;
    int32_t iterations;
    int32_t objective_evaluations;
    int32_t constraint_evaluations;
    int32_t gradient_evaluations;
    int32_t jacobian_evaluations;
    int32_t subproblems;
    double objective;
    double primal_feasibility;
    double stationarity;
    double complementarity;
    double cpu_seconds;
    uint64_t materialization_count;
    uint64_t materialization_columns;
    uint64_t materialization_numeric_bytes; /* largest produced triangular numeric matrix */
    double highs_primal_tolerance; /* effective readbacks; NaN if not initialized */
    double highs_dual_tolerance;
    double highs_optimality_tolerance;
    double highs_qp_regularization;
    uint64_t feasibility_calls; /* current-QP native LP feasibility producers */
    uint64_t feasibility_iterations;
    int32_t feasibility_status; /* actual last HighsModelStatus, separate from QP */
    int32_t feasibility_status_available;
    int32_t feasibility_iterations_available; /* zero is known only when this is 1 */
    int32_t qp_hot_start; /* effective bool readback, -1 if not initialized */
    double feasibility_primal_tolerance; /* actual readback, NaN if unavailable */
} pse_uno_result;
/* Inputs are copied. Callbacks/user must remain alive until destroy returns. All
 * operations on a handle require exclusive access. Bounds may be infinite; COO
 * is zero-based and unique. Only minimization, L=f+lambda*c, is admitted here.
 * No borrowed native pointer escapes. Error text is always NUL terminated when
 * error_capacity>0. Every exported function shields all C++ exceptions. */
/* Library-owned supported floor; no handle or native initialization required. */
double pse_uno_minimum_tolerance(void) PSE_UNO_NOEXCEPT;
int32_t pse_uno_create(int32_t n, int32_t m, int32_t nnz,
    const double* lower, const double* upper, const double* row_lower, const double* row_upper,
    const int32_t* rows, const int32_t* columns, const pse_uno_config*, const pse_uno_callbacks*,
    pse_uno_handle** out, char* error, size_t error_capacity) PSE_UNO_NOEXCEPT;
/* The finite time limit is an additional native safeguard. poll remains the
 * absolute task scope authority, including before/after every evaluation and
 * while acquiring Uno's process-global logger permit. Handles are single-use. */
int32_t pse_uno_solve(pse_uno_handle*, const double* initial, int32_t n,
    char* error, size_t error_capacity) PSE_UNO_NOEXCEPT;
int32_t pse_uno_get_result(pse_uno_handle*, pse_uno_result*, char*, size_t) PSE_UNO_NOEXCEPT;
int32_t pse_uno_get_vectors(pse_uno_handle*, double* primal, double* lower_dual, double* upper_dual,
    int32_t n, double* constraints, double* row_dual, int32_t m, char*, size_t) PSE_UNO_NOEXCEPT;
/* Option keys are read-only; caller capacity includes the terminating NUL. */
int32_t pse_uno_get_string_option(pse_uno_handle*, const char* key, char* value, size_t capacity,
    char* error, size_t error_capacity) PSE_UNO_NOEXCEPT;
int32_t pse_uno_get_double_option(pse_uno_handle*, const char* key, double* value,
    char* error, size_t error_capacity) PSE_UNO_NOEXCEPT;
int32_t pse_uno_get_bool_option(pse_uno_handle*, const char* key, int32_t* value,
    char* error, size_t error_capacity) PSE_UNO_NOEXCEPT;
int32_t pse_uno_get_method(pse_uno_handle*, char* value, size_t capacity,
    char* error, size_t error_capacity) PSE_UNO_NOEXCEPT;
int32_t pse_uno_get_integer_option(pse_uno_handle*, const char* key, int32_t* value,
    char* error, size_t error_capacity) PSE_UNO_NOEXCEPT;
/* Frees both foreign objects even when one destructor throws, then clears *handle. */
int32_t pse_uno_destroy(pse_uno_handle**, char*, size_t) PSE_UNO_NOEXCEPT;
#ifdef __cplusplus
}
#endif
#undef PSE_UNO_NOEXCEPT
#endif
