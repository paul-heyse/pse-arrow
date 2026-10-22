# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One owned OTLP/gRPC/module inquiry; no collector or exporter remains installed/running."""

from __future__ import annotations

import json
import socket
import subprocess
import tempfile
import threading
import time
import urllib.request
from pathlib import Path

from scripts import surreal_server as server
from scripts.surreal_telemetry import Capture


def run(selection:Path,evidence:Path)->None:
    if evidence.exists():
        raise ValueError("OTLP inquiry evidence already exists")
    evidence.mkdir(parents=True,mode=0o700)
    config=server.config_for(selection)
    module=server.study_module(config)
    binary=Path(str(server.object_mapping(config["server"])["binary"]))
    credentials={"username":"otlp-fixture", "password":__import__("secrets").token_urlsafe(32)}
    environment=server.server_environment(config,credentials)
    for key in list(environment):
        if key.startswith("OTEL_"):
            environment.pop(key)
    with Capture() as capture,tempfile.TemporaryDirectory(prefix="pse-otlp-") as directory:
        thread=threading.Thread(target=capture.serve_forever,daemon=True)
        thread.start()
        environment.update(SURREAL_TELEMETRY_PROVIDER="otlp", SURREAL_TELEMETRY_DISABLE_METRICS="true", OTEL_EXPORTER_OTLP_ENDPOINT=f"http://127.0.0.1:{capture.server_address[1]}",OTEL_BSP_SCHEDULE_DELAY="100")
        with socket.socket() as reservation:
            reservation.bind(("127.0.0.1",0))
            port=reservation.getsockname()[1]
        with (evidence/"server.log").open("wb") as log:
            child=subprocess.Popen([str(binary),"start","--no-banner","--no-defaults","--log=warn","--bind",f"127.0.0.1:{port}",f"mem://{directory}/database?aol=sync&sync=every&snapshot=31s&versioned=false&memory_limit=268435456&max_transaction_age=90s&max_async_queue_depth=32&max_async_queue_bytes=16777216"],env=environment,stdout=log,stderr=log)
            try:
                deadline=time.monotonic()+20
                while True:
                    if child.poll() is not None:
                        raise ValueError("OTLP fixture server exited before readiness")
                    try:
                        with urllib.request.urlopen(f"http://127.0.0.1:{port}/health",timeout=0.2) as reply:

                            if reply.status==200:
                                break
                    except OSError:
                        time.sleep(min(0.05,server.remaining(deadline)))
                bucket="pse_study_"+str(module["sha256"])
                sql=f"DEFINE NAMESPACE trace; USE NS trace; DEFINE DATABASE trace; USE DB trace; DEFINE BUCKET {bucket} READONLY BACKEND {json.dumps('file:'+str(Path(str(module['artifact'])).parent))} PERMISSIONS NONE; DEFINE MODULE mod::pse FROM f\"{bucket}:/study.surli\" UNSIGNED PERMISSIONS FULL; RETURN mod::pse::study::readiness({{generation:0dec,cancelled:false,rows:[],items:[]}});"
                reply=subprocess.run([str(binary),"sql","--endpoint",f"grpc://127.0.0.1:{port}","--json","--hide-welcome"],input=sql,text=True,capture_output=True,env=environment,cwd=directory,timeout=20,check=False)
                values=[line.removeprefix("> ").strip() for line in reply.stdout.splitlines()]
                payload=next((line for line in values if line.startswith("[")),"")
                results=json.loads(payload)
                if reply.returncode or not isinstance(results[-1],dict) or results[-1].get("accepted") is not True:
                    raise ValueError("Owned Rust CLI/gRPC/module operation failed")
                until=time.monotonic()+5
                while time.monotonic()<until and not capture.spans:
                    time.sleep(0.05)
            finally:
                child.terminate()
                try:
                    child.wait(timeout=20)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait()
                    raise
                capture.shutdown()
                thread.join(timeout=5)
                capture.server_close()
            if child.returncode!=0:
                raise ValueError("OTLP fixture process failed graceful drain")
            server.write_json(evidence/"capture.json",{"exported_bytes":capture.total,"spans":capture.spans,"operation":"selected Rust CLI -> gRPC -> DB -> readonly Surrealism readiness(empty)","module_accepted":True,"scope":"gRPC request, database executor and Surrealism spans share observed server parentage; originating application Rust exporter/SDK trace-context propagation remains unestablished","decision":"adopt on-demand selected server/module capture with application gap explicit; no permanent telemetry stack","reopen":"an actual correlated application/gRPC/module capture with attributable parentage"})


if __name__=="__main__":
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selection",type=Path,required=True)
    parser.add_argument("--evidence",type=Path,required=True)
    arguments=parser.parse_args()
    run(arguments.selection,arguments.evidence)
