/* -----------------------------------------------------------------
 * Programmer(s): Daniel Reynolds @ SMU
 * Based on codes <solver>_klu.c, written by Carol Woodward @ LLNL
 * -----------------------------------------------------------------
 * SUNDIALS Copyright Start
 * Copyright (c) 2002-2024, Lawrence Livermore National Security
 * and Southern Methodist University.
 * All rights reserved.
 *
 * See the top-level LICENSE and NOTICE files for details.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 * SUNDIALS Copyright End
 * -----------------------------------------------------------------
 * This is the implementation file for the KLU implementation of
 * the SUNLINSOL package.
 * -----------------------------------------------------------------*/

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <sundials/sundials_errors.h>
#include <sundials/sundials_math.h>
#include <sunlinsol/sunlinsol_klu.h>

#include "sundials_macros.h"

#define ZERO      SUN_RCONST(0.0)
#define ONE       SUN_RCONST(1.0)
#define TWO       SUN_RCONST(2.0)
#define TWOTHIRDS SUN_RCONST(0.666666666666666666666666666666667)

/*
 * -----------------------------------------------------------------
 * KLU solver structure accessibility macros:
 * -----------------------------------------------------------------
 */

#define KLU_CONTENT(S)    ((SUNLinearSolverContent_KLU)(S->content))
#define LASTFLAG(S)       (KLU_CONTENT(S)->last_flag)
#define FIRSTFACTORIZE(S) (KLU_CONTENT(S)->first_factorize)
#define SYMBOLIC(S)       (KLU_CONTENT(S)->symbolic)
#define NUMERIC(S)        (KLU_CONTENT(S)->numeric)
#define COMMON(S)         (KLU_CONTENT(S)->common)
#define SOLVE(S)          (KLU_CONTENT(S)->klu_solver)
#define WORK(S)           (KLU_CONTENT(S)->work)
#define FIXED(S)          (KLU_CONTENT(S)->fixed_matrix != NULL)
/* Constant-time object guards. Exact CSC support is owned and checked by the
 * integration before replacement, then restored by its Jacobian callback. */
static int fixed_matrix_matches(SUNLinearSolver S, SUNMatrix A) {
  return A && A == KLU_CONTENT(S)->fixed_matrix &&
    SUNMatGetID(A) == SUNMATRIX_SPARSE &&
    SUNSparseMatrix_SparseType(A) == CSC_MAT &&
    SUNSparseMatrix_Rows(A) == KLU_CONTENT(S)->fixed_dimension &&
    SUNSparseMatrix_Columns(A) == KLU_CONTENT(S)->fixed_dimension &&
    SUNSparseMatrix_NP(A) == KLU_CONTENT(S)->fixed_dimension &&
    SUNSparseMatrix_NNZ(A) == KLU_CONTENT(S)->fixed_nnz;
}
static void invalidate_numeric(SUNLinearSolver S) {
  KLU_CONTENT(S)->numeric_valid = 0;
  FIRSTFACTORIZE(S) = 1;
  if (NUMERIC(S)) { sun_klu_free_numeric(&NUMERIC(S), &COMMON(S)); }
}
static int setup_failure(SUNLinearSolver S, int flag, int preserve_symbolic) {
  if (FIXED(S)) {
    invalidate_numeric(S);
    if (!preserve_symbolic && SYMBOLIC(S)) {
      sun_klu_free_symbolic(&SYMBOLIC(S), &COMMON(S));
    }
  }
  LASTFLAG(S) = flag;
  return flag;
}
static void count_work(uint64_t* value, int* overflow) {
  if (*value == UINT64_MAX) { *overflow = 1; }
  else { ++*value; }
}
#define COUNT_WORK(S, field) count_work(&WORK(S).field, &WORK(S).overflow)
static void observe_work(SUNLinearSolver S, int phase, int begin) {
  if (KLU_CONTENT(S)->observer) {
    KLU_CONTENT(S)->observer(phase, begin, KLU_CONTENT(S)->observer_data);
  }
}

/*
 * -----------------------------------------------------------------
 * exported functions
 * -----------------------------------------------------------------
 */

/* ----------------------------------------------------------------------------
 * Function to create a new KLU linear solver
 */

SUNLinearSolver SUNLinSol_KLU(N_Vector y, SUNMatrix A, SUNContext sunctx)
{
  SUNLinearSolver S;
  SUNLinearSolverContent_KLU content;
  int flag;

  /* Check compatibility with supplied SUNMatrix and N_Vector */
  if (SUNMatGetID(A) != SUNMATRIX_SPARSE) { return (NULL); }

  if (SUNSparseMatrix_Rows(A) != SUNSparseMatrix_Columns(A)) { return (NULL); }

  if ((N_VGetVectorID(y) != SUNDIALS_NVEC_SERIAL) &&
      (N_VGetVectorID(y) != SUNDIALS_NVEC_OPENMP) &&
      (N_VGetVectorID(y) != SUNDIALS_NVEC_PTHREADS))
  {
    return (NULL);
  }

  if (SUNSparseMatrix_Rows(A) != N_VGetLength(y)) { return (NULL); }

  /* Create an empty linear solver */
  S = NULL;
  S = SUNLinSolNewEmpty(sunctx);
  if (S == NULL) { return (NULL); }

  /* Attach operations */
  S->ops->gettype    = SUNLinSolGetType_KLU;
  S->ops->getid      = SUNLinSolGetID_KLU;
  S->ops->initialize = SUNLinSolInitialize_KLU;
  S->ops->setup      = SUNLinSolSetup_KLU;
  S->ops->solve      = SUNLinSolSolve_KLU;
  S->ops->lastflag   = SUNLinSolLastFlag_KLU;
  S->ops->space      = SUNLinSolSpace_KLU;
  S->ops->free       = SUNLinSolFree_KLU;

  /* Create content */
  content = NULL;
  content = (SUNLinearSolverContent_KLU)malloc(sizeof *content);
  if (content == NULL)
  {
    SUNLinSolFree(S);
    return (NULL);
  }

  /* Attach content */
  S->content = content;

  /* Fill content */
  content->last_flag       = 0;
  content->first_factorize = 1;
  content->work = (SUNLinearSolver_KLUWork){0};
  content->observer = NULL;
  content->observer_data = NULL;
  content->symbolic        = NULL;
  content->numeric         = NULL;
  content->fixed_matrix    = NULL;
  content->fixed_dimension = 0;
  content->fixed_nnz       = 0;
  content->numeric_valid   = 0;

#if defined(SUNDIALS_INT64_T)
  if (SUNSparseMatrix_SparseType(A) == CSC_MAT)
  {
    content->klu_solver = (KLUSolveFn)&klu_l_solve;
  }
  else { content->klu_solver = (KLUSolveFn)&klu_l_tsolve; }
#elif defined(SUNDIALS_INT32_T)
  if (SUNSparseMatrix_SparseType(A) == CSC_MAT)
  {
    content->klu_solver = &klu_solve;
  }
  else { content->klu_solver = &klu_tsolve; }
#else /* incompatible sunindextype for KLU */
#error Incompatible sunindextype for KLU
#endif

  flag = sun_klu_defaults(&(content->common));
  if (flag == 0)
  {
    SUNLinSolFree(S);
    return (NULL);
  }
  (content->common).ordering = SUNKLU_ORDERING_DEFAULT;

  return (S);
}

SUNLinearSolver SUNLinSol_KLUFixedPattern(N_Vector y, SUNMatrix A, SUNContext sunctx)
{
  SUNLinearSolver S;
  if (!A || !y || SUNMatGetID(A) != SUNMATRIX_SPARSE ||
      SUNSparseMatrix_SparseType(A) != CSC_MAT) { return NULL; }
  S = SUNLinSol_KLU(y, A, sunctx);
  if (!S) { return NULL; }
  KLU_CONTENT(S)->fixed_matrix = A;
  KLU_CONTENT(S)->fixed_dimension = SUNSparseMatrix_Rows(A);
  KLU_CONTENT(S)->fixed_nnz = SUNSparseMatrix_NNZ(A);
  return S;
}

SUNErrCode SUNLinSol_KLUBeginApplication(SUNLinearSolver S)
{
  if (!S || !S->content) { return SUN_ERR_ARG_CORRUPT; }
  if (SUNLinSolGetID(S) != SUNLINEARSOLVER_KLU || !FIXED(S)) {
    return SUN_ERR_ARG_INCOMPATIBLE;
  }
  invalidate_numeric(S);
  if (!fixed_matrix_matches(S, KLU_CONTENT(S)->fixed_matrix)) {
    return setup_failure(S, SUN_ERR_ARG_INCOMPATIBLE, 0);
  }
  LASTFLAG(S) = SUN_SUCCESS;
  return SUN_SUCCESS;
}

SUNErrCode SUNLinSol_KLUGetMemUsage(SUNLinearSolver S, size_t* bytes)
{
  if (!S || !S->content || !bytes) { return SUN_ERR_ARG_CORRUPT; }
  if (SUNLinSolGetID(S) != SUNLINEARSOLVER_KLU) { return SUN_ERR_ARG_INCOMPATIBLE; }
  *bytes = COMMON(S).memusage;
  return SUN_SUCCESS;
}

/* ----------------------------------------------------------------------------
 * Function to reinitialize a KLU linear solver
 */

SUNErrCode SUNLinSol_KLUReInit(SUNLinearSolver S, SUNMatrix A, sunindextype nnz,
                               int reinit_type)
{
  /* Check for non-NULL SUNLinearSolver */
  if ((S == NULL) || (A == NULL)) { return SUN_ERR_ARG_CORRUPT; }
  if (FIXED(S)) { return SUN_ERR_ARG_INCOMPATIBLE; }

  /* Check for valid SUNMatrix */
  if (SUNMatGetID(A) != SUNMATRIX_SPARSE) { return SUN_ERR_ARG_INCOMPATIBLE; }

  /* Check for valid reinit_type */
  if ((reinit_type != SUNKLU_REINIT_FULL) &&
      (reinit_type != SUNKLU_REINIT_PARTIAL))
  {
    return SUN_ERR_ARG_INCOMPATIBLE;
  }

  /* Full re-initialization: reallocate matrix for updated storage */
  if (reinit_type == SUNKLU_REINIT_FULL)
  {
    if (SUNSparseMatrix_Reallocate(A, nnz) != 0) { return SUN_ERR_MEM_FAIL; }
  }

  /* Free the prior factorization and reset for first factorization */
  if (SYMBOLIC(S) != NULL) { sun_klu_free_symbolic(&SYMBOLIC(S), &COMMON(S)); }
  if (NUMERIC(S) != NULL) { sun_klu_free_numeric(&NUMERIC(S), &COMMON(S)); }
  FIRSTFACTORIZE(S) = 1;

  LASTFLAG(S) = SUN_SUCCESS;
  return SUN_SUCCESS;
}

/* ----------------------------------------------------------------------------
 * Function to set the ordering type for a KLU linear solver
 */

SUNErrCode SUNLinSol_KLUSetOrdering(SUNLinearSolver S, int ordering_choice)
{
  /* Check for legal ordering_choice */
  if ((ordering_choice < 0) || (ordering_choice > 2))
  {
    return SUN_ERR_ARG_INCOMPATIBLE;
  }

  /* Check for non-NULL SUNLinearSolver */
  if (S == NULL) { return SUN_ERR_ARG_CORRUPT; }
  if (FIXED(S)) { return SUN_ERR_ARG_INCOMPATIBLE; }

  /* Set ordering_choice */
  COMMON(S).ordering = ordering_choice;

  return SUN_SUCCESS;
}

/*
 * -----------------------------------------------------------------
 * accessor functions
 * -----------------------------------------------------------------
 */

sun_klu_symbolic* SUNLinSol_KLUGetSymbolic(SUNLinearSolver S)
{
  if (!S || !S->content || FIXED(S)) { return NULL; }
  return (SYMBOLIC(S));
}

sun_klu_numeric* SUNLinSol_KLUGetNumeric(SUNLinearSolver S)
{
  if (!S || !S->content || FIXED(S)) { return NULL; }
  return (NUMERIC(S));
}

sun_klu_common* SUNLinSol_KLUGetCommon(SUNLinearSolver S)
{
  if (!S || !S->content || FIXED(S)) { return NULL; }
  return (&(COMMON(S)));
}
SUNErrCode SUNLinSol_KLUGetWork(SUNLinearSolver S, SUNLinearSolver_KLUWork* work)
{
  if (!S || !S->content || !work) { return SUN_ERR_ARG_CORRUPT; }
  if (SUNLinSolGetID(S) != SUNLINEARSOLVER_KLU) { return SUN_ERR_ARG_INCOMPATIBLE; }
  *work = WORK(S);
  return SUN_SUCCESS;
}

SUNErrCode SUNLinSol_KLUSetObserver(SUNLinearSolver S,
  SUNLinearSolver_KLUObserver observer, void* userdata)
{
  if (!S || !S->content) { return SUN_ERR_ARG_CORRUPT; }
  if (SUNLinSolGetID(S) != SUNLINEARSOLVER_KLU) { return SUN_ERR_ARG_INCOMPATIBLE; }
  KLU_CONTENT(S)->observer = observer;
  KLU_CONTENT(S)->observer_data = observer ? userdata : NULL;
  return SUN_SUCCESS;
}


/*
 * -----------------------------------------------------------------
 * implementation of linear solver operations
 * -----------------------------------------------------------------
 */

SUNLinearSolver_Type SUNLinSolGetType_KLU(SUNDIALS_MAYBE_UNUSED SUNLinearSolver S)
{
  return (SUNLINEARSOLVER_DIRECT);
}

SUNLinearSolver_ID SUNLinSolGetID_KLU(SUNDIALS_MAYBE_UNUSED SUNLinearSolver S)
{
  return (SUNLINEARSOLVER_KLU);
}

SUNErrCode SUNLinSolInitialize_KLU(SUNLinearSolver S)
{
  if (!S || !S->content) { return SUN_ERR_ARG_CORRUPT; }
  if (FIXED(S)) {
    return SUNLinSol_KLUBeginApplication(S);
  }
  /* Force factorization */
  FIRSTFACTORIZE(S) = 1;

  LASTFLAG(S) = SUN_SUCCESS;
  return (LASTFLAG(S));
}

int SUNLinSolSetup_KLU(SUNLinearSolver S, SUNMatrix A)
{
  int retval;
  sunrealtype uround_twothirds;

  if (!S || !S->content) { return SUN_ERR_ARG_CORRUPT; }
  COUNT_WORK(S, setup_issued);
  if (!A) { return setup_failure(S, SUN_ERR_ARG_CORRUPT, 1); }
  if (FIXED(S)) {
    if (!fixed_matrix_matches(S, A)) {
      return setup_failure(S, SUN_ERR_ARG_INCOMPATIBLE, 0);
    }
    KLU_CONTENT(S)->numeric_valid = 0;
  }
  uround_twothirds = SUNRpowerR(SUN_UNIT_ROUNDOFF, TWOTHIRDS);

  /* Ensure that A is a sparse matrix */
  if (SUNMatGetID(A) != SUNMATRIX_SPARSE)
  {
    return setup_failure(S, SUN_ERR_ARG_INCOMPATIBLE, 1);
  }

  /* On first decomposition, get the symbolic factorization */
  if (FIRSTFACTORIZE(S))
  {
    /* Generic mode preserves upstream analysis semantics. Fixed mode retains
     * analysis only while its caller-owned pattern and native owner stay valid. */
    if (!FIXED(S) || !SYMBOLIC(S)) {
    /* Perform symbolic analysis of sparsity structure */
    if (SYMBOLIC(S)) { sun_klu_free_symbolic(&SYMBOLIC(S), &COMMON(S)); }
    COUNT_WORK(S, symbolic_issued);
    observe_work(S, 1, 1);
    SYMBOLIC(S) = sun_klu_analyze(SUNSparseMatrix_NP(A),
                                  SUNSparseMatrix_IndexPointers(A),
                                  SUNSparseMatrix_IndexValues(A), &COMMON(S));
    observe_work(S, 1, 0);
    if (SYMBOLIC(S) == NULL)
    {
      return setup_failure(S, SUN_ERR_EXT_FAIL, 0);
    }

    COUNT_WORK(S, symbolic_completed);
    }

    /* ------------------------------------------------------------
       Compute the LU factorization of the matrix
       ------------------------------------------------------------*/
    if (NUMERIC(S)) { sun_klu_free_numeric(&NUMERIC(S), &COMMON(S)); }
    COUNT_WORK(S, numeric_factor_issued);
    observe_work(S, 2, 1);
    NUMERIC(S) = sun_klu_factor(SUNSparseMatrix_IndexPointers(A),
                                SUNSparseMatrix_IndexValues(A),
                                SUNSparseMatrix_Data(A), SYMBOLIC(S), &COMMON(S));
    observe_work(S, 2, 0);
    if (NUMERIC(S) == NULL)
    {
      return setup_failure(S, SUN_ERR_EXT_FAIL, 1);
    }

    COUNT_WORK(S, numeric_factor_completed);
    FIRSTFACTORIZE(S) = 0;
  }
  else
  { /* not the first decomposition, so just refactor */

    COUNT_WORK(S, refactor_issued);
    observe_work(S, 3, 1);
    retval = sun_klu_refactor(SUNSparseMatrix_IndexPointers(A),
                              SUNSparseMatrix_IndexValues(A),
                              SUNSparseMatrix_Data(A), SYMBOLIC(S), NUMERIC(S),
                              &COMMON(S));
    observe_work(S, 3, 0);
    if (retval == 0)
    {
      return setup_failure(S, SUNLS_PACKAGE_FAIL_REC, 1);
    }

    COUNT_WORK(S, refactor_completed);

    /*-----------------------------------------------------------
      Check if a cheap estimate of the reciprocal of the condition
      number is getting too small.  If so, delete
      the prior numeric factorization and recompute it.
      -----------------------------------------------------------*/

    COUNT_WORK(S, rcond_issued);
    observe_work(S, 4, 1);
    retval = sun_klu_rcond(SYMBOLIC(S), NUMERIC(S), &COMMON(S));
    observe_work(S, 4, 0);
    if (retval == 0)
    {
      return setup_failure(S, SUNLS_PACKAGE_FAIL_REC, 1);
    }

    COUNT_WORK(S, rcond_completed);
    if (COMMON(S).rcond < uround_twothirds)
    {
      /* Condition number may be getting large.
	 Compute more accurate estimate */
      COUNT_WORK(S, condest_issued);
      observe_work(S, 5, 1);
      retval = sun_klu_condest(SUNSparseMatrix_IndexPointers(A),
                               SUNSparseMatrix_Data(A), SYMBOLIC(S), NUMERIC(S),
                               &COMMON(S));
      observe_work(S, 5, 0);
      if (retval == 0)
      {
        return setup_failure(S, SUNLS_PACKAGE_FAIL_REC, 1);
      }

      COUNT_WORK(S, condest_completed);
      if (COMMON(S).condest > (ONE / uround_twothirds))
      {
        /* More accurate estimate also says condition number is
           large, so recompute the numeric factorization */
        sun_klu_free_numeric(&NUMERIC(S), &COMMON(S));
        COUNT_WORK(S, numeric_factor_issued);
        observe_work(S, 2, 1);
        NUMERIC(S) = sun_klu_factor(SUNSparseMatrix_IndexPointers(A),
                                    SUNSparseMatrix_IndexValues(A),
                                    SUNSparseMatrix_Data(A), SYMBOLIC(S),
                                    &COMMON(S));
        observe_work(S, 2, 0);
        if (NUMERIC(S) == NULL)
        {
          return setup_failure(S, SUN_ERR_EXT_FAIL, 1);
        }
        COUNT_WORK(S, numeric_factor_completed);
      }
    }
  }

  if (FIXED(S)) { KLU_CONTENT(S)->numeric_valid = 1; }
  COUNT_WORK(S, setup_completed);
  LASTFLAG(S) = SUN_SUCCESS;
  return (LASTFLAG(S));
}

int SUNLinSolSolve_KLU(SUNLinearSolver S, SUNMatrix A, N_Vector x, N_Vector b,
                       SUNDIALS_MAYBE_UNUSED sunrealtype tol)
{
  int flag;
  sunrealtype* xdata;

  if (S && S->content) { COUNT_WORK(S, solve_issued); }
  /* check for valid inputs */
  if (!S || !S->content) { return SUN_ERR_ARG_CORRUPT; }
  if ((A == NULL) || (x == NULL) || (b == NULL))
  {
    if (FIXED(S)) { return setup_failure(S, SUN_ERR_ARG_CORRUPT, 1); }
    return SUN_ERR_ARG_CORRUPT;
  }
  if (FIXED(S) && (!fixed_matrix_matches(S, A) ||
      !KLU_CONTENT(S)->numeric_valid || !SYMBOLIC(S) || !NUMERIC(S) ||
      N_VGetLength(x) != KLU_CONTENT(S)->fixed_dimension ||
      N_VGetLength(b) != KLU_CONTENT(S)->fixed_dimension)) {
    return setup_failure(S, SUN_ERR_ARG_INCOMPATIBLE, fixed_matrix_matches(S, A));
  }

  /* copy b into x */
  N_VScale(ONE, b, x);

  /* access x data array */
  xdata = N_VGetArrayPointer(x);
  if (xdata == NULL)
  {
    return setup_failure(S, SUN_ERR_MEM_FAIL, 1);
  }

  /* Call KLU to solve the linear system */
  observe_work(S, 6, 1);
  flag = SOLVE(S)(SYMBOLIC(S), NUMERIC(S), SUNSparseMatrix_NP(A), 1, xdata,
                  &COMMON(S));
  observe_work(S, 6, 0);
  if (flag == 0)
  {
    return setup_failure(S, SUNLS_PACKAGE_FAIL_REC, 1);
  }

  COUNT_WORK(S, solve_completed);
  LASTFLAG(S) = SUN_SUCCESS;
  return (LASTFLAG(S));
}

sunindextype SUNLinSolLastFlag_KLU(SUNLinearSolver S) { return (LASTFLAG(S)); }

SUNErrCode SUNLinSolSpace_KLU(SUNDIALS_MAYBE_UNUSED SUNLinearSolver S,
                              long int* lenrwLS, long int* leniwLS)
{
  /* since the klu structures are opaque objects, we
     omit those from these results */
  *leniwLS = 2;
  *lenrwLS = 0;
  return SUN_SUCCESS;
}

SUNErrCode SUNLinSolFree_KLU(SUNLinearSolver S)
{
  /* return with success if already freed */
  if (S == NULL) { return SUN_SUCCESS; }

  /* delete items from the contents structure (if it exists) */
  if (S->content)
  {
    if (NUMERIC(S)) { sun_klu_free_numeric(&NUMERIC(S), &COMMON(S)); }
    if (SYMBOLIC(S)) { sun_klu_free_symbolic(&SYMBOLIC(S), &COMMON(S)); }
    free(S->content);
    S->content = NULL;
  }

  /* delete generic structures */
  if (S->ops)
  {
    free(S->ops);
    S->ops = NULL;
  }
  free(S);
  S = NULL;
  return SUN_SUCCESS;
}
