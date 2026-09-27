/* SPDX-License-Identifier: MIT OR Apache-2.0
 * Copyright (c) 2026 Paul Heyse
 *
 * oneMKL conditional numerical reproducibility and dispatch check.
 *
 *   mkl_cbwr env      report the branch MKL_CBWR selected and require
 *                     COMPATIBLE (the image's pinned branch)
 *   mkl_cbwr set      select COMPATIBLE with mkl_cbwr_set before any other
 *                     MKL call and require it to be in force
 *   mkl_cbwr report   only report the branch in force (for controls)
 *
 * Every mode then runs DGEMM and a symmetric-indefinite DSYTRF/DSYTRS solve
 * (the factorization family Pardiso and SSIDS use) so the CPU-dispatch
 * kernels are actually loaded, and checks the residual.
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <mkl.h>

static const char* branch_name(int branch)
{
   switch( branch )
   {
      case MKL_CBWR_OFF: return "OFF";
      case MKL_CBWR_BRANCH_OFF: return "BRANCH_OFF";
      case MKL_CBWR_AUTO: return "AUTO";
      case MKL_CBWR_COMPATIBLE: return "COMPATIBLE";
      case MKL_CBWR_SSE2: return "SSE2";
      case MKL_CBWR_SSE4_2: return "SSE4_2";
      case MKL_CBWR_AVX2: return "AVX2";
      case MKL_CBWR_AVX512: return "AVX512";
      case MKL_CBWR_AVX512_E1: return "AVX512_E1";
      case MKL_CBWR_AVX10: return "AVX10";
      default: return "UNKNOWN";
   }
}

int main(int argc, char** argv)
{
   const char* mode = argc > 1 ? argv[1] : "env";

   if( strcmp(mode, "set") == 0 )
   {
      /* mkl_cbwr_set must precede every other oneMKL call. */
      int rc = mkl_cbwr_set(MKL_CBWR_COMPATIBLE);
      if( rc != MKL_CBWR_SUCCESS )
      {
         fprintf(stderr, "mkl_cbwr_set(COMPATIBLE) returned %d\n", rc);
         return EXIT_FAILURE;
      }
   }

   char version[256];
   mkl_get_version_string(version, (int)sizeof(version));
   int branch = mkl_cbwr_get(MKL_CBWR_BRANCH);
   int autobranch = mkl_cbwr_get_auto_branch();
   printf("%s\n", version);
   printf("cbwr branch %s (%d); auto branch on this CPU %s (%d); max threads %d; dynamic %d\n",
      branch_name(branch), branch, branch_name(autobranch), autobranch, mkl_get_max_threads(), mkl_get_dynamic());

   if( strcmp(mode, "report") != 0 && branch != MKL_CBWR_COMPATIBLE )
   {
      fprintf(stderr, "CBWR branch in force is %s, expected COMPATIBLE\n", branch_name(branch));
      return EXIT_FAILURE;
   }

   /* C = A * B with a known answer. */
   double a[4] = { 1.0, 2.0, 3.0, 4.0 };
   double b[4] = { 5.0, 6.0, 7.0, 8.0 };
   double c[4] = { 0.0, 0.0, 0.0, 0.0 };
   cblas_dgemm(CblasRowMajor, CblasNoTrans, CblasNoTrans, 2, 2, 2, 1.0, a, 2, b, 2, 0.0, c, 2);
   if( c[0] != 19.0 || c[1] != 22.0 || c[2] != 43.0 || c[3] != 50.0 )
   {
      fprintf(stderr, "dgemm returned a wrong product\n");
      return EXIT_FAILURE;
   }

   /* Symmetric indefinite system K x = r with K = [[2, 1, 0], [1, -3, 1], [0, 1, 1]]. */
   double k[9] = { 2.0, 1.0, 0.0, 1.0, -3.0, 1.0, 0.0, 1.0, 1.0 };
   double k0[9];
   memcpy(k0, k, sizeof(k));
   double x[3] = { 1.0, 2.0, 3.0 };
   double r[3];
   for( int i = 0; i < 3; ++i )
      r[i] = k0[3 * i] * 1.0 + k0[3 * i + 1] * -2.0 + k0[3 * i + 2] * 0.5;
   memcpy(x, r, sizeof(r));
   lapack_int ipiv[3];
   lapack_int info = LAPACKE_dsytrf(LAPACK_ROW_MAJOR, 'L', 3, k, 3, ipiv);
   if( info == 0 )
      info = LAPACKE_dsytrs(LAPACK_ROW_MAJOR, 'L', 3, 1, k, 3, ipiv, x, 1);
   if( info != 0 || fabs(x[0] - 1.0) > 1e-12 || fabs(x[1] + 2.0) > 1e-12 || fabs(x[2] - 0.5) > 1e-12 )
   {
      fprintf(stderr, "dsytrf/dsytrs failed: info %d x = (%g, %g, %g)\n", (int)info, x[0], x[1], x[2]);
      return EXIT_FAILURE;
   }
   printf("dgemm and dsytrf/dsytrs ok\n");
   return EXIT_SUCCESS;
}
