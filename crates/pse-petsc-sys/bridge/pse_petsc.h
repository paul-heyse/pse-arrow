/* SPDX-License-Identifier: MIT OR Apache-2.0 */
/* Copyright (c) 2026 Paul Heyse */
#ifndef PSE_PETSC_H
#define PSE_PETSC_H
#include <stdint.h>

/* SNES/TS handles own callback registrations. Vec/Mat/scatter/options handles are
 * native opaque pointers. All calls require the native process owner's mutex.
 * Rust callback owners must catch every unwind and remain alive until destruction.
 * Callback result: 0 success, 1 native domain flag, otherwise terminal.
 * Domain recovery depends on the native method and evaluation phase.
 * Invalid initial points are refused by the safe profile owner before execution. */
typedef void *PsePetscHandle;
typedef int32_t (*PsePetscFunction)(void *, PsePetscHandle, PsePetscHandle);
typedef int32_t (*PsePetscJacobian)(void *, PsePetscHandle, PsePetscHandle, PsePetscHandle);
typedef int32_t (*PsePetscConvergence)(void *, int32_t, double, double, double, int32_t *);
typedef int32_t (*PsePetscIFunction)(void *, double, PsePetscHandle, PsePetscHandle, PsePetscHandle);
typedef int32_t (*PsePetscIJacobian)(void *, double, PsePetscHandle, PsePetscHandle, double, PsePetscHandle, PsePetscHandle);
typedef int32_t (*PsePetscDomain)(void *, double, PsePetscHandle, int32_t *);
typedef int32_t (*PsePetscPostStep)(void *, PsePetscHandle);
typedef int32_t (*PsePetscPreStage)(void *, PsePetscHandle, double);
typedef int32_t (*PsePetscBlockFunction)(void *, PsePetscHandle, PsePetscHandle, PsePetscHandle);
typedef int32_t (*PsePetscBlockJacobian)(void *, PsePetscHandle, PsePetscHandle, PsePetscHandle, PsePetscHandle);
/* Every overlap coordinate is a global index; interior coordinates must be a
 * subset of overlap. Jacobian triplets use local block coordinates. Maps and
 * pattern are copied at binding; callbacks/context/options remain borrowed.
 * The compact ghost vector follows the declared ghost global indices, includes
 * every overlap coordinate, and overlays current block unknowns on NASM's frozen
 * outer state. Only domain-safe unbounded blocks are
 * admitted by the safe owner. */
typedef struct {
  int32_t size;
  const int32_t *overlap;
  int32_t interior_size;
  const int32_t *interior;
  int32_t ghost_size;
  const int32_t *ghost;
  int32_t nonzeros;
  const int32_t *rows;
  const int32_t *columns;
  PsePetscBlockFunction function;
  PsePetscBlockJacobian jacobian;
  void *context;
  PsePetscHandle options;
} PsePetscBlock;

int32_t pse_petsc_abi(int32_t *, int32_t *, int32_t *, int32_t *, int32_t *);
int32_t pse_petsc_initialized(int32_t *, int32_t *);
int32_t pse_petsc_default_options_empty(int32_t *);
int32_t pse_petsc_initialize(void);
int32_t pse_petsc_finalize(void);
int32_t pse_petsc_error_message(int32_t, const char **);
int32_t pse_petsc_error_name(int32_t, const char **);
/* Project categories; original PETSc codes remain separate. Unknown codes are infrastructure. */
typedef enum { PSE_PETSC_SUCCESS = 0, PSE_PETSC_RESOURCE = 1, PSE_PETSC_CONTRACT = 2,
               PSE_PETSC_NUMERICAL = 3, PSE_PETSC_INFRASTRUCTURE = 4 } PsePetscErrorClass;
PsePetscErrorClass pse_petsc_error_class(int32_t);
extern const int32_t PSE_PETSC_ERR_MEM;
extern const int32_t PSE_PETSC_ERR_ARG_SIZ;
extern const int32_t PSE_PETSC_ERR_ORDER;
extern const int32_t PSE_PETSC_ERR_SYS;
extern const int32_t PSE_PETSC_ERR_LIB;
extern const int32_t PSE_PETSC_ERR_PLIB;
extern const int32_t PSE_PETSC_ERR_MEMC;
extern const int32_t PSE_PETSC_ERR_FP;
extern const int32_t PSE_PETSC_ERR_NOT_CONVERGED;
extern const int32_t PSE_PETSC_ERR_USER;
int32_t pse_petsc_options_create(PsePetscHandle *);
int32_t pse_petsc_options_set(PsePetscHandle, const char *, const char *);
int32_t pse_petsc_options_destroy(PsePetscHandle *);
int32_t pse_petsc_vec_create(int32_t, PsePetscHandle *);
int32_t pse_petsc_vec_destroy(PsePetscHandle *);
int32_t pse_petsc_vec_read(PsePetscHandle, int32_t, double *);
int32_t pse_petsc_vec_write(PsePetscHandle, int32_t, const double *);
int32_t pse_petsc_mat_create(int32_t, int32_t, const int32_t *, PsePetscHandle *);
int32_t pse_petsc_mat_destroy(PsePetscHandle *);
int32_t pse_petsc_mat_write(PsePetscHandle, int32_t, const int32_t *, const int32_t *, const double *);
int32_t pse_petsc_scatter_create(PsePetscHandle, PsePetscHandle, int32_t, const int32_t *, const int32_t *, PsePetscHandle *);
int32_t pse_petsc_scatter_destroy(PsePetscHandle *);
int32_t pse_petsc_snes_create(PsePetscHandle *);
int32_t pse_petsc_snes_destroy(PsePetscHandle *);
int32_t pse_petsc_snes_type(PsePetscHandle, const char *);
int32_t pse_petsc_snes_get_type(PsePetscHandle, const char **);
int32_t pse_petsc_snes_options(PsePetscHandle, PsePetscHandle);
int32_t pse_petsc_snes_from_options(PsePetscHandle);
int32_t pse_petsc_snes_function(PsePetscHandle, PsePetscHandle, PsePetscFunction, void *);
int32_t pse_petsc_snes_jacobian(PsePetscHandle, PsePetscHandle, PsePetscHandle, PsePetscJacobian, void *);
int32_t pse_petsc_snes_convergence(PsePetscHandle, PsePetscConvergence, void *);
int32_t pse_petsc_snes_default_convergence(PsePetscHandle, int32_t, double, double, double, int32_t *);
int32_t pse_petsc_snes_tolerances(PsePetscHandle, double, double, double, int32_t, int32_t);
int32_t pse_petsc_snes_error_if_not_converged(PsePetscHandle, int32_t);
int32_t pse_petsc_snes_tr_tolerances(PsePetscHandle, double, double, double);
int32_t pse_petsc_snes_tr_get_tolerances(PsePetscHandle, double *, double *, double *);
int32_t pse_petsc_snes_tr_update(PsePetscHandle, double, double, double, double, double);
int32_t pse_petsc_snes_linear(PsePetscHandle, const char *, const char *);
int32_t pse_petsc_snes_layout(PsePetscHandle, PsePetscHandle);
int32_t pse_petsc_snes_solve(PsePetscHandle, PsePetscHandle);
int32_t pse_petsc_snes_statistics(PsePetscHandle, int32_t *, int32_t *, int32_t *, int32_t *, double *);
int32_t pse_petsc_snes_reason_name(PsePetscHandle, const char **);
/* Successful registration consumes all sub-SNES wrapper handles and nulls the
 * caller's entries. Scatters remain caller-owned; NASM retains separate references.
 * Borrowed subsolver handles are valid only until their NASM owner is destroyed.
 * Registration is once per owner, before setup. */
int32_t pse_petsc_nasm_subdomains(PsePetscHandle, int32_t, PsePetscHandle *, const PsePetscHandle *, const PsePetscHandle *, const PsePetscHandle *);
int32_t pse_petsc_nasm_subsolver(PsePetscHandle, int32_t, PsePetscHandle *);
int32_t pse_petsc_nasm_restrict(PsePetscHandle, int32_t);
int32_t pse_petsc_nasm_damping(PsePetscHandle, double);
/* Production route: DMShell decomposition owns child DMs, and the library creates
 * sub-SNES. Setup completes before private child options are applied. Manual
 * registration above has a tagged 3.24.0 setup failure and is a negative control. */
int32_t pse_petsc_nasm_dm(PsePetscHandle, PsePetscHandle, int32_t, const PsePetscBlock *);
int32_t pse_petsc_ts_create(PsePetscHandle *);
int32_t pse_petsc_ts_destroy(PsePetscHandle *);
int32_t pse_petsc_ts_pseudo(PsePetscHandle);
int32_t pse_petsc_ts_get_type(PsePetscHandle, const char **);
int32_t pse_petsc_ts_snes(PsePetscHandle, PsePetscHandle *);
int32_t pse_petsc_ts_options(PsePetscHandle, PsePetscHandle);
int32_t pse_petsc_ts_from_options(PsePetscHandle);
int32_t pse_petsc_ts_ifunction(PsePetscHandle, PsePetscHandle, PsePetscIFunction, void *);
int32_t pse_petsc_ts_ijacobian(PsePetscHandle, PsePetscHandle, PsePetscHandle, PsePetscIJacobian, void *);
int32_t pse_petsc_ts_domain(PsePetscHandle, PsePetscDomain, void *);
int32_t pse_petsc_ts_post_step(PsePetscHandle, PsePetscPostStep, void *);
int32_t pse_petsc_ts_pre_stage(PsePetscHandle, PsePetscPreStage, void *);
int32_t pse_petsc_ts_solution(PsePetscHandle, PsePetscHandle);
int32_t pse_petsc_ts_time(PsePetscHandle, double, double, double, int32_t);
int32_t pse_petsc_ts_failures(PsePetscHandle, int32_t, int32_t);
int32_t pse_petsc_ts_adapt(PsePetscHandle, double, double, double);
int32_t pse_petsc_ts_pseudo_growth(PsePetscHandle, double, double);
int32_t pse_petsc_ts_reason(PsePetscHandle, int32_t);
int32_t pse_petsc_ts_solve(PsePetscHandle, PsePetscHandle);
int32_t pse_petsc_ts_statistics(PsePetscHandle, int32_t *, int32_t *, int32_t *, int32_t *, int32_t *, double *, double *);
int32_t pse_petsc_ts_reason_name(PsePetscHandle, const char **);
#endif
