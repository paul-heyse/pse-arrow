/* SPDX-License-Identifier: MIT OR Apache-2.0 */
/* Copyright (c) 2026 Paul Heyse */
#include "pse_petsc.h"
#include <petscts.h>
#include <petscdmshell.h>
#include <stdlib.h>
#include <string.h>

#if PETSC_VERSION_MAJOR != 3 || PETSC_VERSION_MINOR != 24 || PETSC_VERSION_SUBMINOR != 0
#error "The project ABI requires PETSc 3.24.0"
#endif
#if !defined(PETSC_HAVE_MPIUNI) || defined(PETSC_USE_COMPLEX) || !defined(PETSC_USE_REAL_DOUBLE) || defined(PETSC_USE_64BIT_INDICES)
#error "The project ABI requires serial MPIUNI, real double, PetscInt32"
#endif
_Static_assert(sizeof(PetscInt) == sizeof(int32_t), "PetscInt32 ABI");
_Static_assert(sizeof(PetscErrorCode) == sizeof(int32_t), "error code ABI");
_Static_assert(sizeof(PetscScalar) == sizeof(double), "real double ABI");
#define TRY(call) do { PetscErrorCode code_ = (call); if (code_) return code_; } while (0)
#define REQUIRE(value) do { if (!(value)) return PETSC_ERR_ARG_NULL; } while (0)

typedef struct PseSnes PseSnes;
typedef struct PseComposition PseComposition;
static PetscErrorCode composition_finish_failed_solve(PseSnes *, Vec, PetscErrorCode);
static void composition_start_solve(PseSnes *);
struct PseSnes {
  SNES raw;
  int owned;
  PsePetscFunction function;
  PsePetscJacobian jacobian;
  PsePetscConvergence convergence;
  void *function_ctx, *jacobian_ctx, *convergence_ctx;
  int32_t child_count;
  PseSnes **children;
  PseComposition *composition;
};
typedef struct {
  TS raw;
  PseSnes inner;
  PsePetscIFunction function;
  PsePetscIJacobian jacobian;
  PsePetscDomain domain;
  PsePetscPostStep post_step;
  PsePetscPreStage pre_stage;
  void *function_ctx, *jacobian_ctx, *domain_ctx, *post_step_ctx, *pre_stage_ctx;
} PseTs;

static PetscErrorCode function_callback(SNES snes, Vec x, Vec f, void *ctx) {
  PseSnes *owner = ctx;
  int32_t code = owner->function(owner->function_ctx, x, f);
  if (code == 1) return SNESSetFunctionDomainError(snes);
  return code ? PETSC_ERR_USER : PETSC_SUCCESS;
}
static PetscErrorCode jacobian_callback(SNES snes, Vec x, Mat a, Mat p, void *ctx) {
  PseSnes *owner = ctx;
  int32_t code = owner->jacobian(owner->jacobian_ctx, x, a, p);
  if (code == 1) return SNESSetJacobianDomainError(snes);
  return code ? PETSC_ERR_USER : PETSC_SUCCESS;
}
static PetscErrorCode convergence_callback(SNES snes, PetscInt iter, PetscReal xnorm, PetscReal ynorm, PetscReal fnorm, SNESConvergedReason *reason, void *ctx) {
  PseSnes *owner = ctx;
  int32_t value = (int32_t)*reason;
  (void)snes;
  if (owner->convergence(owner->convergence_ctx, iter, xnorm, ynorm, fnorm, &value)) return PETSC_ERR_USER;
  *reason = (SNESConvergedReason)value;
  return PETSC_SUCCESS;
}
static PetscErrorCode ifunction_callback(TS ts, PetscReal t, Vec x, Vec xdot, Vec f, void *ctx) {
  PseTs *owner = ctx;
  int32_t code = owner->function(owner->function_ctx, t, x, xdot, f);
  if (code == 1) {
    SNES snes;
    TRY(TSGetSNES(ts, &snes));
    return SNESSetFunctionDomainError(snes);
  }
  return code ? PETSC_ERR_USER : PETSC_SUCCESS;
}
static PetscErrorCode ijacobian_callback(TS ts, PetscReal t, Vec x, Vec xdot, PetscReal shift, Mat a, Mat p, void *ctx) {
  PseTs *owner = ctx;
  int32_t code = owner->jacobian(owner->jacobian_ctx, t, x, xdot, shift, a, p);
  if (code == 1) {
    SNES snes;
    TRY(TSGetSNES(ts, &snes));
    return SNESSetJacobianDomainError(snes);
  }
  return code ? PETSC_ERR_USER : PETSC_SUCCESS;
}
static PetscErrorCode domain_callback(TS ts, PetscReal t, Vec x, PetscBool *accept) {
  PseTs *owner;
  int32_t accepted = 1;
  TRY(TSGetApplicationContext(ts, &owner));
  if (owner->domain(owner->domain_ctx, t, x, &accepted)) return PETSC_ERR_USER;
  *accept = accepted != 0;
  return PETSC_SUCCESS;
}
static PetscErrorCode post_step_callback(TS ts) {
  PseTs *owner;
  TRY(TSGetApplicationContext(ts, &owner));
  return owner->post_step(owner->post_step_ctx, owner) ? PETSC_ERR_USER : PETSC_SUCCESS;
}
static PetscErrorCode pre_stage_callback(TS ts, PetscReal time) {
  PseTs *owner;
  TRY(TSGetApplicationContext(ts, &owner));
  return owner->pre_stage(owner->pre_stage_ctx, owner, time) ? PETSC_ERR_USER : PETSC_SUCCESS;
}
static void free_children(PseSnes *owner) {
  for (int32_t i = 0; i < owner->child_count; ++i) {
    free_children(owner->children[i]);
    free(owner->children[i]);
  }
  free(owner->children);
}
static PetscErrorCode snes_profile(PseSnes *owner, const char *type) {
  PetscBool matches;
  REQUIRE(owner); TRY(PetscObjectTypeCompare((PetscObject)owner->raw, type, &matches));
  return matches ? PETSC_SUCCESS : PETSC_ERR_ARG_WRONGSTATE;
}

int32_t pse_petsc_abi(int32_t *major, int32_t *minor, int32_t *patch, int32_t *integer_bytes, int32_t *scalar_bytes) {
  REQUIRE(major); REQUIRE(minor); REQUIRE(patch); REQUIRE(integer_bytes); REQUIRE(scalar_bytes);
  *major = PETSC_VERSION_MAJOR; *minor = PETSC_VERSION_MINOR; *patch = PETSC_VERSION_SUBMINOR;
  *integer_bytes = sizeof(PetscInt); *scalar_bytes = sizeof(PetscScalar);
  return PETSC_SUCCESS;
}
int32_t pse_petsc_initialized(int32_t *initialized, int32_t *finalized) {
  PetscBool init, fini;
  REQUIRE(initialized); REQUIRE(finalized);
  TRY(PetscInitialized(&init)); TRY(PetscFinalized(&fini));
  *initialized = init; *finalized = fini;
  return PETSC_SUCCESS;
}
int32_t pse_petsc_default_options_empty(int32_t *empty) {
  char *text = NULL;
  REQUIRE(empty); TRY(PetscOptionsGetAll(NULL, &text));
  *empty = !text || !*text; return PetscFree(text);
}
int32_t pse_petsc_initialize(void) {
  PetscBool initialized, finalized;
  TRY(PetscInitialized(&initialized)); TRY(PetscFinalized(&finalized));
  if (initialized || finalized) return PETSC_ERR_ORDER;
  TRY(PetscInitializeNoArguments());
  return PetscPushErrorHandler(PetscReturnErrorHandler, NULL);
}
int32_t pse_petsc_finalize(void) { return PetscFinalize(); }
int32_t pse_petsc_error_message(int32_t code, const char **message) { REQUIRE(message); return PetscErrorMessage(code, message, NULL); }
const int32_t PSE_PETSC_ERR_MEM = PETSC_ERR_MEM;
const int32_t PSE_PETSC_ERR_ARG_SIZ = PETSC_ERR_ARG_SIZ;
const int32_t PSE_PETSC_ERR_ORDER = PETSC_ERR_ORDER;
const int32_t PSE_PETSC_ERR_SYS = PETSC_ERR_SYS;
const int32_t PSE_PETSC_ERR_LIB = PETSC_ERR_LIB;
const int32_t PSE_PETSC_ERR_PLIB = PETSC_ERR_PLIB;
const int32_t PSE_PETSC_ERR_MEMC = PETSC_ERR_MEMC;
const int32_t PSE_PETSC_ERR_FP = PETSC_ERR_FP;
const int32_t PSE_PETSC_ERR_NOT_CONVERGED = PETSC_ERR_NOT_CONVERGED;
const int32_t PSE_PETSC_ERR_USER = PETSC_ERR_USER;
PsePetscErrorClass pse_petsc_error_class(int32_t code) {
  switch (code) {
    case PETSC_SUCCESS: return PSE_PETSC_SUCCESS;
    case PETSC_ERR_MEM: return PSE_PETSC_RESOURCE;
    case PETSC_ERR_ORDER: case PETSC_ERR_SUP: case PETSC_ERR_SUP_SYS:
    case PETSC_ERR_ARG_SIZ: case PETSC_ERR_ARG_IDN: case PETSC_ERR_ARG_WRONG:
    case PETSC_ERR_ARG_CORRUPT: case PETSC_ERR_ARG_OUTOFRANGE: case PETSC_ERR_ARG_BADPTR:
    case PETSC_ERR_ARG_NOTSAMETYPE: case PETSC_ERR_ARG_NOTSAMECOMM: case PETSC_ERR_ARG_WRONGSTATE:
    case PETSC_ERR_ARG_TYPENOTSET: case PETSC_ERR_ARG_INCOMP: case PETSC_ERR_ARG_NULL:
    case PETSC_ERR_ARG_UNKNOWN_TYPE: case PETSC_ERR_USER_INPUT: case PETSC_ERR_OPT_OVERWRITE:
    case PETSC_ERR_WRONG_MPI_SIZE: return PSE_PETSC_CONTRACT;
    case PETSC_ERR_FP: case PETSC_ERR_CONV_FAILED: case PETSC_ERR_NOT_CONVERGED:
    case PETSC_ERR_MAT_LU_ZRPVT: case PETSC_ERR_MAT_CH_ZRPVT: return PSE_PETSC_NUMERICAL;
    default: return PSE_PETSC_INFRASTRUCTURE;
  }
}
int32_t pse_petsc_error_name(int32_t code, const char **name) {
  REQUIRE(name);
#define ERROR_NAME(value) case value: *name = #value; return PETSC_SUCCESS
  switch (code) {
    ERROR_NAME(PETSC_SUCCESS);
    ERROR_NAME(PETSC_ERR_SUP_SYS); ERROR_NAME(PETSC_ERR_SIG); ERROR_NAME(PETSC_ERR_COR);
    ERROR_NAME(PETSC_ERR_CONV_FAILED); ERROR_NAME(PETSC_ERR_POINTER); ERROR_NAME(PETSC_ERR_MPI_LIB_INCOMP);
    ERROR_NAME(PETSC_ERR_FILE_OPEN); ERROR_NAME(PETSC_ERR_FILE_READ); ERROR_NAME(PETSC_ERR_FILE_WRITE); ERROR_NAME(PETSC_ERR_FILE_UNEXPECTED);
    ERROR_NAME(PETSC_ERR_MAT_LU_ZRPVT); ERROR_NAME(PETSC_ERR_MAT_CH_ZRPVT); ERROR_NAME(PETSC_ERR_INT_OVERFLOW);
    ERROR_NAME(PETSC_ERR_FLOP_COUNT); ERROR_NAME(PETSC_ERR_MISSING_FACTOR); ERROR_NAME(PETSC_ERR_OPT_OVERWRITE);
    ERROR_NAME(PETSC_ERR_WRONG_MPI_SIZE); ERROR_NAME(PETSC_ERR_USER_INPUT); ERROR_NAME(PETSC_ERR_GPU_RESOURCE);
    ERROR_NAME(PETSC_ERR_GPU); ERROR_NAME(PETSC_ERR_MPI); ERROR_NAME(PETSC_ERR_RETURN); ERROR_NAME(PETSC_ERR_PYTHON);
    ERROR_NAME(PETSC_ERR_MEM); ERROR_NAME(PETSC_ERR_SUP); ERROR_NAME(PETSC_ERR_ORDER);
    ERROR_NAME(PETSC_ERR_FP); ERROR_NAME(PETSC_ERR_LIB); ERROR_NAME(PETSC_ERR_PLIB);
    ERROR_NAME(PETSC_ERR_MEMC); ERROR_NAME(PETSC_ERR_USER); ERROR_NAME(PETSC_ERR_SYS);
    ERROR_NAME(PETSC_ERR_ARG_SIZ); ERROR_NAME(PETSC_ERR_ARG_IDN); ERROR_NAME(PETSC_ERR_ARG_WRONG);
    ERROR_NAME(PETSC_ERR_ARG_CORRUPT); ERROR_NAME(PETSC_ERR_ARG_OUTOFRANGE); ERROR_NAME(PETSC_ERR_ARG_BADPTR);
    ERROR_NAME(PETSC_ERR_ARG_NOTSAMETYPE); ERROR_NAME(PETSC_ERR_ARG_NOTSAMECOMM); ERROR_NAME(PETSC_ERR_ARG_WRONGSTATE);
    ERROR_NAME(PETSC_ERR_ARG_TYPENOTSET); ERROR_NAME(PETSC_ERR_ARG_INCOMP); ERROR_NAME(PETSC_ERR_ARG_NULL);
    ERROR_NAME(PETSC_ERR_ARG_UNKNOWN_TYPE); ERROR_NAME(PETSC_ERR_NOT_CONVERGED); ERROR_NAME(PETSC_ERR_MEM_LEAK);
    default: *name = "PETSC_UNKNOWN_ERROR"; return PETSC_SUCCESS;
  }
#undef ERROR_NAME
}
int32_t pse_petsc_options_create(PsePetscHandle *out) { REQUIRE(out); return PetscOptionsCreate((PetscOptions *)out); }
int32_t pse_petsc_options_set(PsePetscHandle options, const char *key, const char *value) { REQUIRE(options); REQUIRE(key); REQUIRE(value); return PetscOptionsSetValue(options, key, value); }
int32_t pse_petsc_options_destroy(PsePetscHandle *options) { REQUIRE(options); return PetscOptionsDestroy((PetscOptions *)options); }
int32_t pse_petsc_vec_create(int32_t n, PsePetscHandle *out) { REQUIRE(out); if (n <= 0) return PETSC_ERR_ARG_SIZ; return VecCreateSeq(PETSC_COMM_SELF, n, (Vec *)out); }
int32_t pse_petsc_vec_destroy(PsePetscHandle *vec) { REQUIRE(vec); return VecDestroy((Vec *)vec); }
int32_t pse_petsc_vec_read(PsePetscHandle vec, int32_t n, double *values) {
  PetscInt size;
  const PetscScalar *array;
  REQUIRE(vec); REQUIRE(values);
  TRY(VecGetLocalSize(vec, &size)); if (size != n) return PETSC_ERR_ARG_SIZ;
  TRY(VecGetArrayRead(vec, &array)); memcpy(values, array, (size_t)n * sizeof(double));
  return VecRestoreArrayRead(vec, &array);
}
int32_t pse_petsc_vec_write(PsePetscHandle vec, int32_t n, const double *values) {
  PetscInt size;
  PetscScalar *array;
  REQUIRE(vec); REQUIRE(values);
  TRY(VecGetLocalSize(vec, &size)); if (size != n) return PETSC_ERR_ARG_SIZ;
  TRY(VecGetArray(vec, &array)); memcpy(array, values, (size_t)n * sizeof(double));
  return VecRestoreArray(vec, &array);
}
int32_t pse_petsc_mat_create(int32_t n, int32_t m, const int32_t *row_capacity, PsePetscHandle *out) {
  REQUIRE(out); REQUIRE(row_capacity); if (n <= 0 || m <= 0) return PETSC_ERR_ARG_SIZ;
  return MatCreateSeqAIJ(PETSC_COMM_SELF, n, m, 0, row_capacity, (Mat *)out);
}
int32_t pse_petsc_mat_destroy(PsePetscHandle *mat) { REQUIRE(mat); return MatDestroy((Mat *)mat); }
int32_t pse_petsc_mat_write(PsePetscHandle mat, int32_t n, const int32_t *rows, const int32_t *columns, const double *values) {
  REQUIRE(mat); REQUIRE(rows); REQUIRE(columns); REQUIRE(values); if (n < 0) return PETSC_ERR_ARG_SIZ;
  TRY(MatZeroEntries(mat));
  for (int32_t i = 0; i < n; ++i) TRY(MatSetValues(mat, 1, rows + i, 1, columns + i, values + i, INSERT_VALUES));
  TRY(MatAssemblyBegin(mat, MAT_FINAL_ASSEMBLY));
  return MatAssemblyEnd(mat, MAT_FINAL_ASSEMBLY);
}
int32_t pse_petsc_scatter_create(PsePetscHandle global, PsePetscHandle local, int32_t n, const int32_t *global_indices, const int32_t *local_indices, PsePetscHandle *out) {
  IS from = NULL, to = NULL;
  PetscErrorCode code, cleanup;
  REQUIRE(global); REQUIRE(local); REQUIRE(global_indices); REQUIRE(local_indices); REQUIRE(out);
  if (n <= 0) return PETSC_ERR_ARG_SIZ;
  TRY(ISCreateGeneral(PETSC_COMM_SELF, n, global_indices, PETSC_COPY_VALUES, &from));
  code = ISCreateGeneral(PETSC_COMM_SELF, n, local_indices, PETSC_COPY_VALUES, &to);
  if (!code) code = VecScatterCreate(global, from, local, to, (VecScatter *)out);
  cleanup = ISDestroy(&from); if (!code) code = cleanup;
  cleanup = ISDestroy(&to); if (!code) code = cleanup;
  return code;
}
int32_t pse_petsc_scatter_destroy(PsePetscHandle *scatter) { REQUIRE(scatter); return VecScatterDestroy((VecScatter *)scatter); }
int32_t pse_petsc_snes_create(PsePetscHandle *out) {
  PseSnes *owner;
  PetscErrorCode code;
  REQUIRE(out); *out = NULL;
  owner = calloc(1, sizeof(*owner)); if (!owner) return PETSC_ERR_MEM;
  code = SNESCreate(PETSC_COMM_SELF, &owner->raw);
  if (code) { free(owner); return code; }
  owner->owned = 1; *out = owner;
  return PETSC_SUCCESS;
}
int32_t pse_petsc_snes_destroy(PsePetscHandle *handle) {
  PseSnes *owner;
  REQUIRE(handle); if (!*handle) return PETSC_SUCCESS;
  owner = *handle; if (!owner->owned) return PETSC_ERR_ARG_WRONGSTATE;
  TRY(SNESDestroy(&owner->raw)); free_children(owner); free(owner); *handle = NULL;
  return PETSC_SUCCESS;
}
int32_t pse_petsc_snes_type(PsePetscHandle handle, const char *type) { REQUIRE(handle); REQUIRE(type); if (((PseSnes *)handle)->children) return PETSC_ERR_ARG_WRONGSTATE; return SNESSetType(((PseSnes *)handle)->raw, type); }
int32_t pse_petsc_snes_get_type(PsePetscHandle handle, const char **type) { REQUIRE(handle); REQUIRE(type); return SNESGetType(((PseSnes *)handle)->raw, type); }
int32_t pse_petsc_snes_options(PsePetscHandle handle, PsePetscHandle options) {
  KSP ksp; PC pc;
  REQUIRE(handle); REQUIRE(options);
  TRY(PetscObjectSetOptions((PetscObject)((PseSnes *)handle)->raw, options));
  TRY(SNESGetKSP(((PseSnes *)handle)->raw, &ksp)); TRY(PetscObjectSetOptions((PetscObject)ksp, options));
  TRY(KSPGetPC(ksp, &pc)); return PetscObjectSetOptions((PetscObject)pc, options);
}
int32_t pse_petsc_snes_from_options(PsePetscHandle handle) { REQUIRE(handle); if (((PseSnes *)handle)->children) return PETSC_ERR_ARG_WRONGSTATE; return SNESSetFromOptions(((PseSnes *)handle)->raw); }
int32_t pse_petsc_snes_function(PsePetscHandle handle, PsePetscHandle f, PsePetscFunction callback, void *ctx) {
  PseSnes *owner = handle;
  REQUIRE(owner); REQUIRE(callback);
  owner->function = callback; owner->function_ctx = ctx;
  return SNESSetFunction(owner->raw, f, function_callback, owner);
}
int32_t pse_petsc_snes_jacobian(PsePetscHandle handle, PsePetscHandle a, PsePetscHandle p, PsePetscJacobian callback, void *ctx) {
  PseSnes *owner = handle;
  REQUIRE(owner); REQUIRE(callback); REQUIRE(a); REQUIRE(p);
  owner->jacobian = callback; owner->jacobian_ctx = ctx;
  return SNESSetJacobian(owner->raw, a, p, jacobian_callback, owner);
}
int32_t pse_petsc_snes_convergence(PsePetscHandle handle, PsePetscConvergence callback, void *ctx) {
  PseSnes *owner = handle;
  REQUIRE(owner); REQUIRE(callback);
  owner->convergence = callback; owner->convergence_ctx = ctx;
  return SNESSetConvergenceTest(owner->raw, convergence_callback, owner, NULL);
}
int32_t pse_petsc_snes_default_convergence(PsePetscHandle handle, int32_t iter, double xnorm, double ynorm, double fnorm, int32_t *reason) {
  SNESConvergedReason value = SNES_CONVERGED_ITERATING;
  REQUIRE(handle); REQUIRE(reason);
  TRY(SNESConvergedDefault(((PseSnes *)handle)->raw, iter, xnorm, ynorm, fnorm, &value, NULL));
  *reason = value; return PETSC_SUCCESS;
}
int32_t pse_petsc_snes_tolerances(PsePetscHandle handle, double atol, double rtol, double stol, int32_t iterations, int32_t evaluations) { REQUIRE(handle); return SNESSetTolerances(((PseSnes *)handle)->raw, atol, rtol, stol, iterations, evaluations); }
int32_t pse_petsc_snes_tr_tolerances(PsePetscHandle handle, double minimum, double maximum, double initial) { TRY(snes_profile(handle, SNESNEWTONTR)); return SNESNewtonTRSetTolerances(((PseSnes *)handle)->raw, minimum, maximum, initial); }
int32_t pse_petsc_snes_tr_get_tolerances(PsePetscHandle handle, double *minimum, double *maximum, double *initial) { REQUIRE(minimum); REQUIRE(maximum); REQUIRE(initial); TRY(snes_profile(handle, SNESNEWTONTR)); return SNESNewtonTRGetTolerances(((PseSnes *)handle)->raw, minimum, maximum, initial); }
int32_t pse_petsc_snes_tr_update(PsePetscHandle handle, double eta1, double eta2, double eta3, double shrink, double grow) { TRY(snes_profile(handle, SNESNEWTONTR)); return SNESNewtonTRSetUpdateParameters(((PseSnes *)handle)->raw, eta1, eta2, eta3, shrink, grow); }
int32_t pse_petsc_snes_linear(PsePetscHandle handle, const char *ksp_type, const char *pc_type) {
  KSP ksp; PC pc;
  REQUIRE(handle); REQUIRE(ksp_type); REQUIRE(pc_type);
  TRY(SNESGetKSP(((PseSnes *)handle)->raw, &ksp)); TRY(KSPSetType(ksp, ksp_type));
  TRY(KSPGetPC(ksp, &pc)); return PCSetType(pc, pc_type);
}
static PetscErrorCode local_begin(DM dm, Vec global, InsertMode mode, Vec local) { (void)dm; (void)mode; return VecCopy(global, local); }
static PetscErrorCode local_end(DM dm, Vec global, InsertMode mode, Vec local) { (void)dm; (void)global; (void)mode; (void)local; return PETSC_SUCCESS; }
int32_t pse_petsc_snes_layout(PsePetscHandle handle, PsePetscHandle layout) {
  DM dm = NULL;
  PetscErrorCode code, cleanup;
  REQUIRE(handle); REQUIRE(layout);
  TRY(DMCreate(PETSC_COMM_SELF, &dm));
  code = DMSetType(dm, DMSHELL);
  if (!code) code = DMShellSetGlobalVector(dm, layout);
  if (!code) code = DMShellSetLocalVector(dm, layout);
  if (!code) code = DMShellSetGlobalToLocal(dm, local_begin, local_end);
  if (!code) code = SNESSetDM(((PseSnes *)handle)->raw, dm);
  cleanup = DMDestroy(&dm); return code ? code : cleanup;
}
int32_t pse_petsc_snes_solve(PsePetscHandle handle, PsePetscHandle x) {
  PseSnes *owner = handle;
  REQUIRE(owner); REQUIRE(x);
  composition_start_solve(owner);
  return composition_finish_failed_solve(owner, x, SNESSolve(owner->raw, NULL, x));
}
int32_t pse_petsc_snes_statistics(PsePetscHandle handle, int32_t *reason, int32_t *iterations, int32_t *evaluations, int32_t *linear_iterations, double *norm) {
  SNES raw; SNESConvergedReason value;
  REQUIRE(handle); REQUIRE(reason); REQUIRE(iterations); REQUIRE(evaluations); REQUIRE(linear_iterations); REQUIRE(norm);
  raw = ((PseSnes *)handle)->raw;
  TRY(SNESGetConvergedReason(raw, &value)); *reason = value;
  TRY(SNESGetIterationNumber(raw, iterations)); TRY(SNESGetNumberFunctionEvals(raw, evaluations));
  TRY(SNESGetLinearSolveIterations(raw, linear_iterations)); return SNESGetFunctionNorm(raw, norm);
}
int32_t pse_petsc_snes_reason_name(PsePetscHandle handle, const char **name) { REQUIRE(handle); REQUIRE(name); return SNESGetConvergedReasonString(((PseSnes *)handle)->raw, name); }
int32_t pse_petsc_snes_error_if_not_converged(PsePetscHandle handle, int32_t enabled) { REQUIRE(handle); return SNESSetErrorIfNotConverged(((PseSnes *)handle)->raw, enabled ? PETSC_TRUE : PETSC_FALSE); }
int32_t pse_petsc_nasm_subdomains(PsePetscHandle handle, int32_t n, PsePetscHandle *children, const PsePetscHandle *interior, const PsePetscHandle *overlap, const PsePetscHandle *ghost) {
  PseSnes *owner = handle;
  SNES *raw;
  PseSnes **saved;
  PetscErrorCode code;
  REQUIRE(owner); REQUIRE(children); REQUIRE(interior); REQUIRE(overlap); REQUIRE(ghost);
  TRY(snes_profile(owner, SNESNASM));
  if (n <= 0) return PETSC_ERR_ARG_SIZ;
  if (owner->children) return PETSC_ERR_ARG_WRONGSTATE;
  for (int32_t i = 0; i < n; ++i) {
    REQUIRE(children[i]); REQUIRE(interior[i]); REQUIRE(overlap[i]); REQUIRE(ghost[i]);
    if (!((PseSnes *)children[i])->owned || children[i] == handle) return PETSC_ERR_ARG_WRONGSTATE;
    for (int32_t j = 0; j < i; ++j) if (children[j] == children[i]) return PETSC_ERR_ARG_WRONGSTATE;
  }
  raw = calloc((size_t)n, sizeof(*raw)); saved = calloc((size_t)n, sizeof(*saved));
  if (!raw || !saved) { free(raw); free(saved); return PETSC_ERR_MEM; }
  for (int32_t i = 0; i < n; ++i) { saved[i] = children[i]; raw[i] = saved[i]->raw; }
  code = SNESNASMSetSubdomains(owner->raw, n, raw, (VecScatter *)interior, (VecScatter *)overlap, (VecScatter *)ghost);
  free(raw);
  if (code) { free(saved); return code; }
  owner->children = saved; owner->child_count = n;
  for (int32_t i = 0; i < n; ++i) { saved[i]->owned = 0; children[i] = NULL; }
  return PETSC_SUCCESS;
}
int32_t pse_petsc_nasm_subsolver(PsePetscHandle handle, int32_t index, PsePetscHandle *out) {
  PseSnes *owner = handle;
  REQUIRE(owner); REQUIRE(out); if (index < 0 || index >= owner->child_count) return PETSC_ERR_ARG_OUTOFRANGE;
  *out = owner->children[index]; return PETSC_SUCCESS;
}
int32_t pse_petsc_nasm_restrict(PsePetscHandle handle, int32_t restricted) { TRY(snes_profile(handle, SNESNASM)); return SNESNASMSetType(((PseSnes *)handle)->raw, restricted ? PC_ASM_RESTRICT : PC_ASM_BASIC); }
int32_t pse_petsc_nasm_damping(PsePetscHandle handle, double damping) { TRY(snes_profile(handle, SNESNASM)); return SNESNASMSetDamping(((PseSnes *)handle)->raw, damping); }

typedef struct {
  DM dm;
  PetscInt size, interior_size, ghost_size;
  PetscInt *overlap, *interior, *interior_local, *ghost, *overlap_ghost;
  PsePetscBlockFunction function;
  PsePetscBlockJacobian jacobian;
  void *context;
  PetscOptions options;
  PetscInt ordinal;
  VecScatter overlap_scatter, ghost_scatter;
} PseBlock;
struct PseComposition {
  PetscInt count, global_size;
  Vec global;
  PseBlock **blocks;
  PetscInt active;
};
static void composition_start_solve(PseSnes *owner) {
  if (owner->composition) owner->composition->active = -1;
}
static PetscErrorCode composition_finish_failed_solve(PseSnes *owner, Vec input, PetscErrorCode result) {
  PseComposition *composition = owner->composition;
  Vec solution, *local, *ghost;
  VecScatter *overlap_scatter, *ghost_scatter;
  PetscInt count, vector_count;
  /* In tagged 3.24.0 NASM starts every forward scatter, then ends each child's
   * pair immediately before DMSubDomainRestrict/SNESSolve. A contained callback
   * error or child nonconvergence exits before later children end their pairs.
   * The public restriction hook records only completed pairs; never end those
   * again. This wrapper always supplies a NULL RHS, so no RHS scatter is active. */
  if (!composition || (result != PETSC_ERR_USER && result != PETSC_ERR_NOT_CONVERGED) || composition->active < 0) return result;
  TRY(SNESGetSolution(owner->raw, &solution));
  if (solution != input) return PETSC_ERR_ARG_WRONGSTATE;
  TRY(SNESNASMGetSubdomains(owner->raw, &count, NULL, NULL, &overlap_scatter, &ghost_scatter));
  TRY(SNESNASMGetSubdomainVecs(owner->raw, &vector_count, &local, NULL, NULL, &ghost));
  if (count != composition->count || vector_count != count) return PETSC_ERR_ARG_WRONGSTATE;
  for (PetscInt i = composition->active + 1; i < count; ++i) {
    TRY(VecScatterEnd(overlap_scatter[i], solution, local[i], INSERT_VALUES, SCATTER_FORWARD));
    TRY(VecScatterEnd(ghost_scatter[i], solution, ghost[i], INSERT_VALUES, SCATTER_FORWARD));
    composition->active = i;
  }
  return result;
}
static PetscErrorCode block_destroy(void *ctx) {
  PseBlock *block = ctx;
  TRY(PetscFree(block->overlap)); TRY(PetscFree(block->interior)); TRY(PetscFree(block->interior_local));
  TRY(PetscFree(block->ghost)); TRY(PetscFree(block->overlap_ghost));
  return PetscFree(block);
}
static PetscErrorCode composition_destroy(void *ctx) {
  PseComposition *owner = ctx;
  for (PetscInt i = 0; i < owner->count; ++i) if (owner->blocks[i]) {
    DM child = owner->blocks[i]->dm;
    TRY(DMDestroy(&child));
  }
  TRY(PetscFree(owner->blocks)); return PetscFree(owner);
}
static PetscErrorCode block_local_begin(DM dm, Vec global, InsertMode mode, Vec local) {
  PseBlock *block;
  const PetscScalar *x;
  PetscScalar *ghost;
  if (mode != INSERT_VALUES) return PETSC_ERR_SUP;
  TRY(DMShellGetContext(dm, &block)); TRY(VecGetArrayRead(global, &x));
  PetscErrorCode code = VecGetArray(local, &ghost);
  if (!code) {
    for (PetscInt i = 0; i < block->size; ++i) ghost[block->overlap_ghost[i]] = x[i];
    code = VecRestoreArray(local, &ghost);
  }
  PetscErrorCode cleanup = VecRestoreArrayRead(global, &x);
  return code ? code : cleanup;
}
static PetscErrorCode block_ghost(SNES snes, Vec x, DM *dm, Vec *ghost) {
  TRY(SNESGetDM(snes, dm)); TRY(DMGetLocalVector(*dm, ghost));
  PetscErrorCode code = DMGlobalToLocalBegin(*dm, x, INSERT_VALUES, *ghost);
  if (!code) code = DMGlobalToLocalEnd(*dm, x, INSERT_VALUES, *ghost);
  if (code) DMRestoreLocalVector(*dm, ghost);
  return code;
}
static PetscErrorCode block_function(SNES snes, Vec x, Vec f, void *ctx) {
  PseBlock *block = ctx;
  DM dm; Vec ghost;
  TRY(block_ghost(snes, x, &dm, &ghost));
  int32_t result = block->function(block->context, x, ghost, f);
  TRY(DMRestoreLocalVector(dm, &ghost));
  if (result == 1) return SNESSetFunctionDomainError(snes);
  return result ? PETSC_ERR_USER : PETSC_SUCCESS;
}
static PetscErrorCode block_jacobian(SNES snes, Vec x, Mat a, Mat p, void *ctx) {
  PseBlock *block = ctx;
  DM dm; Vec ghost;
  TRY(block_ghost(snes, x, &dm, &ghost));
  int32_t result = block->jacobian(block->context, x, ghost, a, p);
  TRY(DMRestoreLocalVector(dm, &ghost));
  if (result == 1) return SNESSetJacobianDomainError(snes);
  return result ? PETSC_ERR_USER : PETSC_SUCCESS;
}
static PetscErrorCode block_subdomain_hook(DM parent, DM child, void *ctx) {
  PseBlock *block;
  (void)parent; (void)ctx;
  /* The parent's standard DMSNES subdomain hook copies its outer residual onto
   * every child. This later public hook restores the declared block actions via
   * DMSNES copy-on-write, without modifying the parent's original oracle. */
  TRY(DMShellGetContext(child, &block));
  TRY(DMSNESSetFunction(child, block_function, block));
  return DMSNESSetJacobian(child, block_jacobian, block);
}
static PetscErrorCode block_restrict_hook(DM parent, VecScatter overlap, VecScatter ghost, DM child, void *ctx) {
  PseComposition *composition = ctx;
  PseBlock *block;
  (void)parent;
  TRY(DMShellGetContext(child, &block));
  if (block->ordinal < 0 || block->ordinal >= composition->count || composition->blocks[block->ordinal] != block) return PETSC_ERR_ARG_WRONGSTATE;
  if (overlap != block->overlap_scatter || ghost != block->ghost_scatter) return PETSC_ERR_ARG_WRONGSTATE;
  composition->active = block->ordinal;
  return PETSC_SUCCESS;
}
static PetscErrorCode decomposition(DM dm, PetscInt *count, char ***names, IS **inner, IS **outer, DM **children) {
  PseComposition *owner;
  TRY(DMShellGetContext(dm, &owner));
  if (count) *count = owner->count;
  if (names) *names = NULL;
  if (children) TRY(PetscCalloc1(owner->count, children));
  if (inner) TRY(PetscCalloc1(owner->count, inner));
  if (outer) TRY(PetscCalloc1(owner->count, outer));
  for (PetscInt i = 0; i < owner->count; ++i) {
    PseBlock *block = owner->blocks[i];
    if (children) { TRY(PetscObjectReference((PetscObject)block->dm)); (*children)[i] = block->dm; }
    if (inner) TRY(ISCreateGeneral(PETSC_COMM_SELF, block->interior_size, block->interior, PETSC_COPY_VALUES, &(*inner)[i]));
    if (outer) TRY(ISCreateGeneral(PETSC_COMM_SELF, block->size, block->overlap, PETSC_COPY_VALUES, &(*outer)[i]));
  }
  return PETSC_SUCCESS;
}
static PetscErrorCode decomposition_scatters(DM dm, PetscInt n, DM *children, VecScatter **interior, VecScatter **overlap, VecScatter **ghost) {
  PseComposition *owner;
  TRY(DMShellGetContext(dm, &owner));
  if (n != owner->count) return PETSC_ERR_ARG_SIZ;
  TRY(PetscCalloc1(n, interior)); TRY(PetscCalloc1(n, overlap)); TRY(PetscCalloc1(n, ghost));
  for (PetscInt i = 0; i < n; ++i) {
    PseBlock *block = owner->blocks[i];
    Vec x = NULL, local = NULL;
    IS from = NULL, to = NULL;
    PetscErrorCode code, cleanup;
    if (children[i] != block->dm) return PETSC_ERR_ARG_WRONGSTATE;
    code = DMCreateGlobalVector(block->dm, &x);
    if (!code) code = DMCreateLocalVector(block->dm, &local);
    if (!code) code = ISCreateGeneral(PETSC_COMM_SELF, block->interior_size, block->interior, PETSC_COPY_VALUES, &from);
    if (!code) code = ISCreateGeneral(PETSC_COMM_SELF, block->interior_size, block->interior_local, PETSC_COPY_VALUES, &to);
    if (!code) code = VecScatterCreate(owner->global, from, x, to, &(*interior)[i]);
    cleanup = ISDestroy(&from); if (!code) code = cleanup;
    cleanup = ISDestroy(&to); if (!code) code = cleanup;
    if (!code) code = ISCreateGeneral(PETSC_COMM_SELF, block->size, block->overlap, PETSC_COPY_VALUES, &from);
    if (!code) code = ISCreateStride(PETSC_COMM_SELF, block->size, 0, 1, &to);
    if (!code) code = VecScatterCreate(owner->global, from, x, to, &(*overlap)[i]);
    cleanup = ISDestroy(&from); if (!code) code = cleanup;
    cleanup = ISDestroy(&to); if (!code) code = cleanup;
    if (!code) code = ISCreateGeneral(PETSC_COMM_SELF, block->ghost_size, block->ghost, PETSC_COPY_VALUES, &from);
    if (!code) code = ISCreateStride(PETSC_COMM_SELF, block->ghost_size, 0, 1, &to);
    if (!code) code = VecScatterCreate(owner->global, from, local, to, &(*ghost)[i]);
    cleanup = ISDestroy(&from); if (!code) code = cleanup;
    cleanup = ISDestroy(&to); if (!code) code = cleanup;
    cleanup = VecDestroy(&x); if (!code) code = cleanup;
    cleanup = VecDestroy(&local); if (!code) code = cleanup;
    if (code) return code;
  }
  return PETSC_SUCCESS;
}
static PetscErrorCode create_block(PseComposition *owner, const PsePetscBlock *input, PseBlock **out) {
  PseBlock *block = NULL;
  Vec x = NULL, local = NULL;
  Mat matrix = NULL;
  PetscInt *capacity = NULL;
  PetscScalar *zeros = NULL;
  PetscErrorCode code, cleanup;
  REQUIRE(input->overlap); REQUIRE(input->interior); REQUIRE(input->ghost); REQUIRE(input->rows); REQUIRE(input->columns);
  REQUIRE(input->function); REQUIRE(input->jacobian); REQUIRE(input->options);
  if (input->size <= 0 || input->interior_size <= 0 || input->ghost_size <= 0 || input->nonzeros <= 0) return PETSC_ERR_ARG_SIZ;
  TRY(PetscNew(&block));
  code = DMShellCreate(PETSC_COMM_SELF, &block->dm);
  if (code) { PetscFree(block); return code; }
  *out = block;
  TRY(DMShellSetContext(block->dm, block)); TRY(DMShellSetDestroyContext(block->dm, block_destroy));
  block->size = input->size; block->interior_size = input->interior_size; block->ghost_size = input->ghost_size;
  block->function = input->function; block->jacobian = input->jacobian; block->context = input->context; block->options = input->options;
  TRY(PetscMalloc1(block->size, &block->overlap)); TRY(PetscMalloc1(block->interior_size, &block->interior)); TRY(PetscMalloc1(block->interior_size, &block->interior_local));
  TRY(PetscMalloc1(block->ghost_size, &block->ghost)); TRY(PetscMalloc1(block->size, &block->overlap_ghost));
  for (PetscInt i = 0; i < block->ghost_size; ++i) {
    PetscInt index = input->ghost[i];
    if (index < 0 || index >= owner->global_size) return PETSC_ERR_ARG_OUTOFRANGE;
    for (PetscInt j = 0; j < i; ++j) if (input->ghost[j] == index) return PETSC_ERR_ARG_WRONG;
    block->ghost[i] = index;
  }
  for (PetscInt i = 0; i < block->size; ++i) {
    PetscInt index = input->overlap[i], ghost_index = -1;
    if (index < 0 || index >= owner->global_size) return PETSC_ERR_ARG_OUTOFRANGE;
    for (PetscInt j = 0; j < i; ++j) if (input->overlap[j] == index) return PETSC_ERR_ARG_WRONG;
    for (PetscInt j = 0; j < block->ghost_size; ++j) if (block->ghost[j] == index) ghost_index = j;
    if (ghost_index < 0) return PETSC_ERR_ARG_WRONG;
    block->overlap[i] = index;
    block->overlap_ghost[i] = ghost_index;
  }
  for (PetscInt i = 0; i < block->interior_size; ++i) {
    PetscInt local_index = -1, index = input->interior[i];
    for (PetscInt j = 0; j < block->size; ++j) if (block->overlap[j] == index) local_index = j;
    if (local_index < 0) return PETSC_ERR_ARG_WRONG;
    for (PetscInt j = 0; j < i; ++j) if (input->interior[j] == index) return PETSC_ERR_ARG_WRONG;
    block->interior[i] = index; block->interior_local[i] = local_index;
  }
  TRY(PetscCalloc1(block->size, &capacity));
  for (PetscInt i = 0; i < input->nonzeros; ++i) {
    if (input->rows[i] < 0 || input->rows[i] >= block->size || input->columns[i] < 0 || input->columns[i] >= block->size) { PetscFree(capacity); return PETSC_ERR_ARG_OUTOFRANGE; }
    capacity[input->rows[i]]++;
  }
  code = VecCreateSeq(PETSC_COMM_SELF, block->size, &x);
  if (!code) code = VecCreateSeq(PETSC_COMM_SELF, block->ghost_size, &local);
  if (!code) code = MatCreateSeqAIJ(PETSC_COMM_SELF, block->size, block->size, 0, capacity, &matrix);
  cleanup = PetscFree(capacity); if (!code) code = cleanup;
  if (!code) code = PetscCalloc1(input->nonzeros, &zeros);
  if (!code) code = pse_petsc_mat_write(matrix, input->nonzeros, input->rows, input->columns, zeros);
  cleanup = PetscFree(zeros); if (!code) code = cleanup;
  if (!code) code = DMShellSetGlobalVector(block->dm, x);
  if (!code) code = DMShellSetLocalVector(block->dm, local);
  if (!code) code = DMShellSetMatrix(block->dm, matrix);
  if (!code) code = DMShellSetGlobalToLocal(block->dm, block_local_begin, local_end);
  if (!code) code = DMSNESSetFunction(block->dm, block_function, block);
  if (!code) code = DMSNESSetJacobian(block->dm, block_jacobian, block);
  cleanup = VecDestroy(&x); if (!code) code = cleanup;
  cleanup = VecDestroy(&local); if (!code) code = cleanup;
  cleanup = MatDestroy(&matrix); if (!code) code = cleanup;
  return code;
}
int32_t pse_petsc_nasm_dm(PsePetscHandle handle, PsePetscHandle global, int32_t n, const PsePetscBlock *inputs) {
  PseSnes *owner = handle;
  PseComposition *composition = NULL;
  DM dm = NULL;
  SNES *subsolvers = NULL;
  VecScatter *overlap_scatter = NULL, *ghost_scatter = NULL;
  PetscInt count;
  PetscErrorCode code, cleanup;
  REQUIRE(owner); REQUIRE(global); REQUIRE(inputs); TRY(snes_profile(owner, SNESNASM));
  if (n <= 0) return PETSC_ERR_ARG_SIZ;
  if (owner->children) return PETSC_ERR_ARG_WRONGSTATE;
  /* Original residual refresh provides the full outer anchor for compact child
   * domain reconstruction, and is the actual outer convergence evidence. */
  TRY(SNESSetNormSchedule(owner->raw, SNES_NORM_ALWAYS));
  TRY(PetscNew(&composition));
  composition->count = n; composition->global = global; composition->active = -1;
  code = VecGetLocalSize(global, &composition->global_size);
  if (!code) code = PetscCalloc1(n, &composition->blocks);
  if (!code) code = DMShellCreate(PETSC_COMM_SELF, &dm);
  if (code) { if (composition->blocks) PetscFree(composition->blocks); PetscFree(composition); return code; }
  code = DMShellSetContext(dm, composition);
  if (!code) code = DMShellSetDestroyContext(dm, composition_destroy);
  if (!code) code = DMShellSetGlobalVector(dm, global);
  if (!code) code = DMShellSetLocalVector(dm, global);
  if (!code) code = DMShellSetCreateDomainDecomposition(dm, decomposition);
  if (!code) code = DMShellSetCreateDomainDecompositionScatters(dm, decomposition_scatters);
  for (PetscInt i = 0; !code && i < n; ++i) {
    code = create_block(composition, inputs + i, &composition->blocks[i]);
    if (!code) composition->blocks[i]->ordinal = i;
  }
  if (!code) code = SNESSetDM(owner->raw, dm);
  if (!code) code = DMSubDomainHookAdd(dm, block_subdomain_hook, block_restrict_hook, composition);
  if (!code) code = SNESSetUp(owner->raw);
  if (!code) code = SNESNASMGetSubdomains(owner->raw, &count, &subsolvers, NULL, &overlap_scatter, &ghost_scatter);
  if (!code && count != n) code = PETSC_ERR_ARG_SIZ;
  if (!code) {
    owner->children = calloc((size_t)n, sizeof(*owner->children));
    if (!owner->children) code = PETSC_ERR_MEM;
  }
  for (PetscInt i = 0; !code && i < n; ++i) {
    PseSnes *child = calloc(1, sizeof(*child));
    if (!child) { code = PETSC_ERR_MEM; break; }
    owner->children[i] = child; owner->child_count++;
    child->raw = subsolvers[i];
    composition->blocks[i]->overlap_scatter = overlap_scatter[i];
    composition->blocks[i]->ghost_scatter = ghost_scatter[i];
    code = SNESSetOptionsPrefix(child->raw, NULL);
    if (!code) code = pse_petsc_snes_options(child, composition->blocks[i]->options);
    if (!code) code = SNESSetType(child->raw, SNESNEWTONTR);
    if (!code) code = SNESSetFromOptions(child->raw);
    if (!code) code = snes_profile(child, SNESNEWTONTR);
  }
  if (!code) owner->composition = composition;
  cleanup = DMDestroy(&dm); return code ? code : cleanup;
}
int32_t pse_petsc_ts_create(PsePetscHandle *out) {
  PseTs *owner;
  PetscErrorCode code;
  REQUIRE(out); *out = NULL;
  owner = calloc(1, sizeof(*owner)); if (!owner) return PETSC_ERR_MEM;
  code = TSCreate(PETSC_COMM_SELF, &owner->raw);
  if (!code) code = TSSetApplicationContext(owner->raw, owner);
  if (code) { if (owner->raw) TSDestroy(&owner->raw); free(owner); return code; }
  *out = owner; return PETSC_SUCCESS;
}
int32_t pse_petsc_ts_destroy(PsePetscHandle *handle) {
  PseTs *owner;
  REQUIRE(handle); if (!*handle) return PETSC_SUCCESS;
  owner = *handle; TRY(TSDestroy(&owner->raw)); free_children(&owner->inner); free(owner); *handle = NULL;
  return PETSC_SUCCESS;
}
int32_t pse_petsc_ts_pseudo(PsePetscHandle handle) { REQUIRE(handle); return TSSetType(((PseTs *)handle)->raw, TSPSEUDO); }
int32_t pse_petsc_ts_get_type(PsePetscHandle handle, const char **type) { REQUIRE(handle); REQUIRE(type); return TSGetType(((PseTs *)handle)->raw, type); }
int32_t pse_petsc_ts_snes(PsePetscHandle handle, PsePetscHandle *out) {
  PseTs *owner = handle;
  REQUIRE(owner); REQUIRE(out); TRY(TSGetSNES(owner->raw, &owner->inner.raw));
  *out = &owner->inner; return PETSC_SUCCESS;
}
int32_t pse_petsc_ts_options(PsePetscHandle handle, PsePetscHandle options) {
  PseTs *owner = handle; TSAdapt adapt; PsePetscHandle inner;
  REQUIRE(owner); REQUIRE(options);
  TRY(PetscObjectSetOptions((PetscObject)owner->raw, options));
  TRY(TSGetAdapt(owner->raw, &adapt)); TRY(PetscObjectSetOptions((PetscObject)adapt, options));
  TRY(pse_petsc_ts_snes(owner, &inner)); return pse_petsc_snes_options(inner, options);
}
int32_t pse_petsc_ts_from_options(PsePetscHandle handle) { REQUIRE(handle); return TSSetFromOptions(((PseTs *)handle)->raw); }
int32_t pse_petsc_ts_ifunction(PsePetscHandle handle, PsePetscHandle f, PsePetscIFunction callback, void *ctx) {
  PseTs *owner = handle; REQUIRE(owner); REQUIRE(callback);
  owner->function = callback; owner->function_ctx = ctx;
  return TSSetIFunction(owner->raw, f, ifunction_callback, owner);
}
int32_t pse_petsc_ts_ijacobian(PsePetscHandle handle, PsePetscHandle a, PsePetscHandle p, PsePetscIJacobian callback, void *ctx) {
  PseTs *owner = handle; REQUIRE(owner); REQUIRE(callback); REQUIRE(a); REQUIRE(p);
  owner->jacobian = callback; owner->jacobian_ctx = ctx;
  return TSSetIJacobian(owner->raw, a, p, ijacobian_callback, owner);
}
int32_t pse_petsc_ts_domain(PsePetscHandle handle, PsePetscDomain callback, void *ctx) {
  PseTs *owner = handle; REQUIRE(owner); REQUIRE(callback);
  owner->domain = callback; owner->domain_ctx = ctx;
  return TSSetFunctionDomainError(owner->raw, domain_callback);
}
int32_t pse_petsc_ts_post_step(PsePetscHandle handle, PsePetscPostStep callback, void *ctx) {
  PseTs *owner = handle; REQUIRE(owner); REQUIRE(callback);
  owner->post_step = callback; owner->post_step_ctx = ctx;
  return TSSetPostStep(owner->raw, post_step_callback);
}
int32_t pse_petsc_ts_pre_stage(PsePetscHandle handle, PsePetscPreStage callback, void *ctx) {
  PseTs *owner = handle; REQUIRE(owner); REQUIRE(callback);
  owner->pre_stage = callback; owner->pre_stage_ctx = ctx;
  return TSSetPreStage(owner->raw, pre_stage_callback);
}
int32_t pse_petsc_ts_solution(PsePetscHandle handle, PsePetscHandle x) { REQUIRE(handle); REQUIRE(x); return TSSetSolution(((PseTs *)handle)->raw, x); }
int32_t pse_petsc_ts_time(PsePetscHandle handle, double start, double dt, double end, int32_t steps) {
  TS raw;
  REQUIRE(handle); raw = ((PseTs *)handle)->raw;
  TRY(TSSetTime(raw, start)); TRY(TSSetTimeStep(raw, dt)); TRY(TSSetMaxTime(raw, end)); TRY(TSSetMaxSteps(raw, steps));
  return TSSetExactFinalTime(raw, TS_EXACTFINALTIME_STEPOVER);
}
int32_t pse_petsc_ts_failures(PsePetscHandle handle, int32_t nonlinear_failures, int32_t rejections) {
  TS raw;
  REQUIRE(handle); raw = ((PseTs *)handle)->raw;
  TRY(TSSetMaxSNESFailures(raw, nonlinear_failures)); TRY(TSSetMaxStepRejections(raw, rejections));
  return TSSetErrorIfStepFails(raw, PETSC_FALSE);
}
int32_t pse_petsc_ts_adapt(PsePetscHandle handle, double minimum, double maximum, double failed_scale) {
  TSAdapt adapt;
  REQUIRE(handle); TRY(TSGetAdapt(((PseTs *)handle)->raw, &adapt));
  TRY(TSAdaptSetStepLimits(adapt, minimum, maximum)); return TSAdaptSetScaleSolveFailed(adapt, failed_scale);
}
int32_t pse_petsc_ts_pseudo_growth(PsePetscHandle handle, double increment, double maximum) {
  REQUIRE(handle); TRY(TSPseudoSetTimeStepIncrement(((PseTs *)handle)->raw, increment));
  return TSPseudoSetMaxTimeStep(((PseTs *)handle)->raw, maximum);
}
int32_t pse_petsc_ts_reason(PsePetscHandle handle, int32_t reason) { REQUIRE(handle); return TSSetConvergedReason(((PseTs *)handle)->raw, (TSConvergedReason)reason); }
int32_t pse_petsc_ts_solve(PsePetscHandle handle, PsePetscHandle x) { REQUIRE(handle); REQUIRE(x); return TSSolve(((PseTs *)handle)->raw, x); }
int32_t pse_petsc_ts_statistics(PsePetscHandle handle, int32_t *reason, int32_t *steps, int32_t *nonlinear_iterations, int32_t *rejections, int32_t *failures, double *time, double *dt) {
  TS raw; TSConvergedReason value;
  REQUIRE(handle); REQUIRE(reason); REQUIRE(steps); REQUIRE(nonlinear_iterations); REQUIRE(rejections); REQUIRE(failures); REQUIRE(time); REQUIRE(dt);
  raw = ((PseTs *)handle)->raw;
  TRY(TSGetConvergedReason(raw, &value)); *reason = value;
  TRY(TSGetStepNumber(raw, steps)); TRY(TSGetSNESIterations(raw, nonlinear_iterations));
  TRY(TSGetStepRejections(raw, rejections)); TRY(TSGetSNESFailures(raw, failures));
  TRY(TSGetTime(raw, time)); return TSGetTimeStep(raw, dt);
}
int32_t pse_petsc_ts_reason_name(PsePetscHandle handle, const char **name) {
  TSConvergedReason reason;
  REQUIRE(handle); REQUIRE(name); TRY(TSGetConvergedReason(((PseTs *)handle)->raw, &reason));
#define TS_REASON(value) case value: *name = #value; return PETSC_SUCCESS
  switch (reason) {
    TS_REASON(TS_CONVERGED_ITERATING); TS_REASON(TS_CONVERGED_TIME); TS_REASON(TS_CONVERGED_ITS);
    TS_REASON(TS_CONVERGED_USER); TS_REASON(TS_CONVERGED_EVENT);
    TS_REASON(TS_CONVERGED_PSEUDO_FATOL); TS_REASON(TS_CONVERGED_PSEUDO_FRTOL);
    TS_REASON(TS_DIVERGED_NONLINEAR_SOLVE); TS_REASON(TS_DIVERGED_STEP_REJECTED);
    default: *name = "TS_UNKNOWN_REASON"; return PETSC_SUCCESS;
  }
#undef TS_REASON
}
