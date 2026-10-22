# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Retained small diagnostic controls; none qualifies ordinary scientific timing."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import subprocess
import sys
from pathlib import Path

from scripts import active_diagnostics

SOURCE = r'''#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <stdio.h>
#include <sys/syscall.h>
volatile void *retained;
void thread_identity(const char *role) {
 char path[128], line[4096]; long tid=syscall(SYS_gettid);
 snprintf(path,sizeof(path),"/proc/self/task/%ld/stat",tid);
 FILE *f=fopen(path,"r"); if(!f) return;
 if(fgets(line,sizeof(line),f)) {
  char *field=strrchr(line,')'); field=field ? strtok(field+2," ") : NULL;
  for(int n=3;n<22 && field;n++) field=strtok(NULL," ");
  if(field) printf("PSE_CONTROL_THREAD pid=%d tid=%ld start=%s role=%s\n",getpid(),tid,field,role);
 }
 fclose(f); fflush(stdout);
}
pthread_mutex_t mutex=PTHREAD_MUTEX_INITIALIZER;
__attribute__((noinline)) void allocate_temporary(void) {
 for(int i=0;i<1000;i++){ char *p=malloc(4096); memset(p,i,4096); asm volatile("" : : "r"(p) : "memory"); free(p); }
}
__attribute__((noinline)) void allocate_retained(void) { retained=malloc(4*1024*1024); }
__attribute__((noinline)) void crash_native(void) { *(volatile int *)0=1; }
__attribute__((noinline)) void known_cpu(void) { for(volatile long n=0;n<10000000;n++){} }
void *contender(void *unused) { thread_identity("mutex-contender"); pthread_mutex_lock(&mutex); allocate_temporary(); known_cpu(); pthread_mutex_unlock(&mutex); return NULL; }
int main(int argc,char **argv){
 thread_identity("main-cpu-and-sleep");
 if(argc>1 && !strcmp(argv[1],"crash")) crash_native();
 pthread_mutex_lock(&mutex); pthread_t thread; pthread_create(&thread,NULL,contender,NULL);
 allocate_retained(); allocate_temporary(); known_cpu(); usleep(100000);
 pthread_mutex_unlock(&mutex); pthread_join(thread,NULL); return 0;
}
'''


def action(root: Path, name: str, kind: str, command: list[str] | None = None, options: list[str] | None = None) -> dict:
    output = io.StringIO()
    with contextlib.redirect_stdout(output):
        code = active_diagnostics.main([kind, "--output", str(root / name), *(options or []), *(["--", *command] if command else [])])
    value = json.loads(output.getvalue())
    return {"action": name, "reader": str(root / name / "action.json"), "adapter_exit": code,
            "status": value.get("status"), "target_outcome": value.get("target_outcome"), "error": value.get("error")}


def native(root: Path) -> list[dict]:
    source = root / "native-control.c"
    source.write_text(SOURCE)
    executable = root / "native-control"
    library = root / "ffi-control.so"
    subprocess.run(["cc", "-O0", "-g", "-fno-omit-frame-pointer", "-pthread", str(source), "-o", str(executable)], check=True)
    subprocess.run(["cc", "-O0", "-g", "-fno-omit-frame-pointer", "-fPIC", "-shared", "-pthread", str(source), "-o", str(library)], check=True)
    ffi = [sys.executable, "-c", "import ctypes,sys; ctypes.CDLL(sys.argv[1]).crash_native()", str(library)]
    return [action(root, "gdb-native", "gdb-launch", [str(executable), "crash"]),
            action(root, "gdb-python-ffi", "gdb-launch", ffi, ["--faulthandler"]),
            action(root, "native-allocations", "heaptrack", [str(executable)]),
            action(root, "scheduler", "sched", [str(executable)]),
            action(root, "counters", "stat", [str(executable)], ["--events", "cycles:u,instructions:u,branches:u,branch-misses:u"]),
            action(root, "feasibility", "feasibility")]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(mode=0o700, parents=False, exist_ok=False)
    result = native(args.output)
    (args.output / "control.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
