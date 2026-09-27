/* SPDX-License-Identifier: MIT OR Apache-2.0
 * Copyright (c) 2026 Paul Heyse
 *
 * SCIP acceptance test: version, linked external codes (the image's Ipopt,
 * SoPlex, PaPILO, GMP/MPFR), the nested-Ipopt parameters, and one small
 * nonconvex MINLP built through the expression API with a known global optimum:
 *
 *   min  t
 *   s.t. t + x*y >= 0             (bilinear, nonconvex)
 *        x^2 + y^2 <= 6
 *        x in [0, 2], y in {0, 1, 2}, t in [-10, 10]
 *
 * y = 2 gives x = sqrt(2) and t* = -2*sqrt(2); y = 1 gives only -2.
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "scip/scip.h"
#include "scip/scipdefplugins.h"

#define CHECK(call)                                                                      \
   do                                                                                    \
   {                                                                                     \
      SCIP_RETCODE rc_ = (call);                                                         \
      if( rc_ != SCIP_OKAY )                                                             \
      {                                                                                  \
         fprintf(stderr, "%s:%d: %s returned %d\n", __FILE__, __LINE__, #call, (int)rc_); \
         return EXIT_FAILURE;                                                            \
      }                                                                                  \
   } while( 0 )

static int has_code(SCIP* scip, const char* prefix)
{
   char** names = SCIPgetExternalCodeNames(scip);
   for( int i = 0; i < SCIPgetNExternalCodes(scip); ++i )
      if( strncmp(names[i], prefix, strlen(prefix)) == 0 )
         return 1;
   return 0;
}

int main(void)
{
   SCIP* scip = NULL;
   CHECK( SCIPcreate(&scip) );
   CHECK( SCIPincludeDefaultPlugins(scip) );
   /* Never install SCIP's SIGINT handler in a host process. */
   CHECK( SCIPsetBoolParam(scip, "misc/catchctrlc", FALSE) );
   CHECK( SCIPsetIntParam(scip, "display/verblevel", 0) );

   /* SCIP has no run-time API-version query: SCIP_APIVERSION is a header
    * constant, so the library is matched to its headers by major/minor/tech. */
   printf("scip %d.%d.%d, headers %d.%d.%d, header api %d, build %s\n", SCIPmajorVersion(), SCIPminorVersion(),
      SCIPtechVersion(), SCIP_VERSION_MAJOR, SCIP_VERSION_MINOR, SCIP_VERSION_PATCH, SCIP_APIVERSION, SCIP_BUILD_TYPE);
   if( SCIPmajorVersion() != SCIP_VERSION_MAJOR || SCIPminorVersion() != SCIP_VERSION_MINOR
      || SCIPtechVersion() != SCIP_VERSION_PATCH || SCIP_VERSION_MAJOR != 10 || SCIP_VERSION_MINOR != 0
      || SCIP_VERSION_PATCH != 2 || SCIP_APIVERSION != 156 || sizeof(SCIP_Real) != sizeof(double) )
   {
      fprintf(stderr, "SCIP library and headers are not the pinned 10.0.2 (API 156)\n");
      return EXIT_FAILURE;
   }

   char** names = SCIPgetExternalCodeNames(scip);
   char** descs = SCIPgetExternalCodeDescriptions(scip);
   for( int i = 0; i < SCIPgetNExternalCodes(scip); ++i )
      printf("  external %-24s %s\n", names[i], descs[i] != NULL ? descs[i] : "");
   const char* required[] = { "Ipopt 3.14.20", "SoPlex", "PaPILO", "GMP", "MPFR" };
   for( size_t i = 0; i < sizeof(required) / sizeof(required[0]); ++i )
      if( !has_code(scip, required[i]) )
      {
         fprintf(stderr, "SCIP is not linked with %s\n", required[i]);
         return EXIT_FAILURE;
      }
   if( SCIPfindNlpi(scip, "ipopt") == NULL )
   {
      fprintf(stderr, "SCIP has no Ipopt NLP interface\n");
      return EXIT_FAILURE;
   }
   /* The nested Ipopt offers exactly the image's linear solvers.  hsllib does
    * not exist (no HSL, no loader); Ipopt 3.14 registers pardisolib
    * unconditionally, but no linear_solver value can reach it. */
   CHECK( SCIPsetStringParam(scip, "nlpi/ipopt/linear_solver", "mumps") );
   const char* lsdesc = SCIPparamGetDesc(SCIPgetParam(scip, "nlpi/ipopt/linear_solver"));
   printf("  nlpi/ipopt/linear_solver: %s\n", lsdesc);
   printf("  nlpi/ipopt/hsllib: %s; nlpi/ipopt/pardisolib: %s\n",
      SCIPgetParam(scip, "nlpi/ipopt/hsllib") != NULL ? "present" : "absent",
      SCIPgetParam(scip, "nlpi/ipopt/pardisolib") != NULL ? "present" : "absent");
   if( SCIPgetParam(scip, "nlpi/ipopt/hsllib") != NULL || strstr(lsdesc, " mumps") == NULL
      || strstr(lsdesc, " spral") == NULL || strstr(lsdesc, " pardisomkl") == NULL || strstr(lsdesc, " ma") != NULL
      || strstr(lsdesc, " pardiso ") != NULL || strstr(lsdesc, " wsmp") != NULL )
   {
      fprintf(stderr, "nested Ipopt linear solvers are not exactly mumps, spral and pardisomkl\n");
      return EXIT_FAILURE;
   }

   CHECK( SCIPcreateProbBasic(scip, "bilinear_minlp") );
   SCIP_VAR* x;
   SCIP_VAR* y;
   SCIP_VAR* t;
   CHECK( SCIPcreateVarBasic(scip, &x, "x", 0.0, 2.0, 0.0, SCIP_VARTYPE_CONTINUOUS) );
   CHECK( SCIPcreateVarBasic(scip, &y, "y", 0.0, 2.0, 0.0, SCIP_VARTYPE_INTEGER) );
   CHECK( SCIPcreateVarBasic(scip, &t, "t", -10.0, 10.0, 1.0, SCIP_VARTYPE_CONTINUOUS) );
   CHECK( SCIPaddVar(scip, x) );
   CHECK( SCIPaddVar(scip, y) );
   CHECK( SCIPaddVar(scip, t) );

   SCIP_EXPR* ex;
   SCIP_EXPR* ey;
   SCIP_EXPR* et;
   CHECK( SCIPcreateExprVar(scip, &ex, x, NULL, NULL) );
   CHECK( SCIPcreateExprVar(scip, &ey, y, NULL, NULL) );
   CHECK( SCIPcreateExprVar(scip, &et, t, NULL, NULL) );

   SCIP_EXPR* factors[2] = { ex, ey };
   SCIP_EXPR* xy;
   CHECK( SCIPcreateExprProduct(scip, &xy, 2, factors, 1.0, NULL, NULL) );
   SCIP_EXPR* terms[2] = { et, xy };
   SCIP_Real ones[2] = { 1.0, 1.0 };
   SCIP_EXPR* bilinear;
   CHECK( SCIPcreateExprSum(scip, &bilinear, 2, terms, ones, 0.0, NULL, NULL) );
   SCIP_CONS* cbilinear;
   CHECK( SCIPcreateConsBasicNonlinear(scip, &cbilinear, "bilinear", bilinear, 0.0, SCIPinfinity(scip)) );
   CHECK( SCIPaddCons(scip, cbilinear) );

   SCIP_EXPR* x2;
   SCIP_EXPR* y2;
   CHECK( SCIPcreateExprPow(scip, &x2, ex, 2.0, NULL, NULL) );
   CHECK( SCIPcreateExprPow(scip, &y2, ey, 2.0, NULL, NULL) );
   SCIP_EXPR* squares[2] = { x2, y2 };
   SCIP_EXPR* circle;
   CHECK( SCIPcreateExprSum(scip, &circle, 2, squares, ones, 0.0, NULL, NULL) );
   SCIP_CONS* ccircle;
   CHECK( SCIPcreateConsBasicNonlinear(scip, &ccircle, "circle", circle, -SCIPinfinity(scip), 6.0) );
   CHECK( SCIPaddCons(scip, ccircle) );

   CHECK( SCIPsolve(scip) );

   SCIP_STATUS status = SCIPgetStatus(scip);
   SCIP_Real primal = SCIPgetPrimalbound(scip);
   SCIP_Real dual = SCIPgetDualbound(scip);
   SCIP_Real gap = SCIPgetGap(scip);
   SCIP_SOL* best = SCIPgetBestSol(scip);
   printf("status %d primal %.10f dual %.10f gap %.3g nodes %lld\n", (int)status, primal, dual, gap,
      (long long)SCIPgetNTotalNodes(scip));
   int ok = status == SCIP_STATUS_OPTIMAL && best != NULL && fabs(primal + 2.0 * sqrt(2.0)) <= 1e-6
      && fabs(dual - primal) <= 1e-6 && fabs(SCIPgetSolVal(scip, best, y) - 2.0) <= 1e-9;
   if( best != NULL )
      printf("x %.8f y %.8f t %.8f\n", SCIPgetSolVal(scip, best, x), SCIPgetSolVal(scip, best, y),
         SCIPgetSolVal(scip, best, t));

   CHECK( SCIPreleaseCons(scip, &ccircle) );
   CHECK( SCIPreleaseCons(scip, &cbilinear) );
   CHECK( SCIPreleaseExpr(scip, &circle) );
   CHECK( SCIPreleaseExpr(scip, &y2) );
   CHECK( SCIPreleaseExpr(scip, &x2) );
   CHECK( SCIPreleaseExpr(scip, &bilinear) );
   CHECK( SCIPreleaseExpr(scip, &xy) );
   CHECK( SCIPreleaseExpr(scip, &et) );
   CHECK( SCIPreleaseExpr(scip, &ey) );
   CHECK( SCIPreleaseExpr(scip, &ex) );
   CHECK( SCIPreleaseVar(scip, &t) );
   CHECK( SCIPreleaseVar(scip, &y) );
   CHECK( SCIPreleaseVar(scip, &x) );
   CHECK( SCIPfree(&scip) );

   if( !ok )
   {
      fprintf(stderr, "SCIP did not prove the known global optimum -2*sqrt(2)\n");
      return EXIT_FAILURE;
   }
   return EXIT_SUCCESS;
}
