/* SPDX-License-Identifier: MIT OR Apache-2.0
 * Copyright (c) 2026 Paul Heyse
 *
 * Reports the linear solvers this Ipopt build can use and fails unless the
 * image contract holds: mumps, spral and pardisomkl present; no HSL routine,
 * no pardiso-project Pardiso and no WSMP (ADR-0108 items 8-9).
 */
#include <stdio.h>
#include <stdlib.h>

#include "IpLinearSolvers.h"
#include "IpStdCInterface.h"

int main(void)
{
   int major = 0, minor = 0, release = 0;
   GetIpoptVersion(&major, &minor, &release);
   printf("ipopt %d.%d.%d\n", major, minor, release);

   /* 0: also report solvers the run-time loader could dlopen.  The build has
    * no loader, so this equals the linked set; asking for the wider set is the
    * stronger check. */
   IpoptLinearSolver all = IpoptGetAvailableLinearSolvers(0);
   IpoptLinearSolver linked = IpoptGetAvailableLinearSolvers(1);
   printf("available 0x%03x linked 0x%03x\n", all, linked);
   printf("  mumps      %s\n", (all & IPOPTLINEARSOLVER_MUMPS) ? "yes" : "no");
   printf("  spral      %s\n", (all & IPOPTLINEARSOLVER_SPRAL) ? "yes" : "no");
   printf("  pardisomkl %s\n", (all & IPOPTLINEARSOLVER_PARDISOMKL) ? "yes" : "no");
   printf("  hsl        %s\n", (all & IPOPTLINEARSOLVER_ALLHSL) ? "yes" : "no");
   printf("  pardiso    %s\n", (all & IPOPTLINEARSOLVER_PARDISO) ? "yes" : "no");
   printf("  wsmp       %s\n", (all & IPOPTLINEARSOLVER_WSMP) ? "yes" : "no");

   const IpoptLinearSolver required =
      IPOPTLINEARSOLVER_MUMPS | IPOPTLINEARSOLVER_SPRAL | IPOPTLINEARSOLVER_PARDISOMKL;
   const IpoptLinearSolver forbidden =
      IPOPTLINEARSOLVER_ALLHSL | IPOPTLINEARSOLVER_PARDISO | IPOPTLINEARSOLVER_WSMP;
   if( (linked & required) != required )
   {
      fprintf(stderr, "missing a required linear solver\n");
      return EXIT_FAILURE;
   }
   if( all & forbidden )
   {
      fprintf(stderr, "an excluded linear solver is available\n");
      return EXIT_FAILURE;
   }
   if( all != linked )
   {
      fprintf(stderr, "a run-time loaded linear solver is available\n");
      return EXIT_FAILURE;
   }
   return EXIT_SUCCESS;
}
