# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite on-demand OTLP sink and explicit selected-service capture lifecycle."""
from __future__ import annotations

import os
import socketserver
import threading
import time
import uuid
from pathlib import Path

from scripts import surreal_server as server, native_operation
from scripts.grpc_wire import fields

class Capture(socketserver.ThreadingTCPServer):
    """Finite on-demand OTLP sink; preserve span identities, never attributes or payloads."""
    allow_reuse_address = False
    daemon_threads = False
    def __init__(self) -> None:
        self.spans: list[dict[str,str]] = []
        self.total = 0
        self.omitted_spans = 0
        self.guard = threading.Lock()
        self.deadline = time.monotonic()+420
        self.errors: list[str] = []
        super().__init__(("127.0.0.1",0), Receiver)

    def observe(self, framed: bytes) -> None:
        with self.guard:
            self.total += len(framed)
            if self.total > 16 << 20:
                raise ValueError("OTLP capture exceeded its finite capture bound")
            position=0
            while position < len(framed):
                if len(framed)-position < 5 or framed[position]!=0:
                    raise ValueError("OTLP capture requires complete uncompressed gRPC messages")
                size=int.from_bytes(framed[position+1:position+5],"big")
                body=framed[position+5:position+5+size];position+=size+5
                if len(body)!=size:
                    raise ValueError("OTLP capture message was truncated")
                for resource in fields(body).get(1,[]):
                    if not isinstance(resource,bytes):continue
                    for scope in fields(resource).get(2,[]):
                        if not isinstance(scope,bytes):continue
                        for span in fields(scope).get(2,[]):
                            if not isinstance(span,bytes):continue
                            item=fields(span)
                            trace=item.get(1,[b""])[0]; identity=item.get(2,[b""])[0]; name=item.get(5,[b""])[0]
                            if isinstance(trace,bytes) and len(trace)==16 and isinstance(identity,bytes) and len(identity)==8 and isinstance(name,bytes):
                                parent=item.get(4,[b""])[0]
                                if len(self.spans)>=4096:
                                    self.omitted_spans += 1
                                    continue
                                self.spans.append({"trace_id":trace.hex(),"span_id":identity.hex(),"parent_span_id":parent.hex() if isinstance(parent,bytes) else "", "name":name.decode(errors="replace")[:256]})


class Receiver(socketserver.BaseRequestHandler):
    def handle(self) -> None:
        capture = self.server
        if not isinstance(capture,Capture):
            raise ValueError("Unexpected OTLP capture owner")
        def receive(size:int)->bytes:
            result=bytearray()
            while len(result)<size:
                self.request.settimeout(min(15,server.remaining(capture.deadline)))
                block=self.request.recv(size-len(result))
                if not block:raise EOFError
                result.extend(block)
            return bytes(result)
        def frame(kind:int,flags:int,stream:int,data:bytes)->None:
            self.request.sendall(len(data).to_bytes(3,"big")+bytes((kind,flags))+stream.to_bytes(4,"big")+data)
        pending:dict[int,bytearray]={}
        try:
            if receive(24)!=b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n":raise ValueError("OTLP capture did not receive HTTP/2")
            frame(4,0,0,b"")
            while True:
                header=receive(9); size=int.from_bytes(header[:3],"big");kind,flags=header[3:5];stream=int.from_bytes(header[5:9],"big")
                if size>1<<20:raise ValueError("OTLP frame exceeds fixture bound")
                data=receive(size)
                if kind==4 and not flags&1:frame(4,1,0,b"")
                elif kind==6 and not flags&1:frame(6,1,0,data)
                elif kind==7:return
                elif kind==3:pending.pop(stream,None)
                elif kind==0:
                    if stream not in pending and len(pending)>=32:
                        raise ValueError("OTLP capture exceeds concurrent stream bound")
                    payload=pending.setdefault(stream,bytearray());payload.extend(data)
                    if len(payload)>8<<20:raise ValueError("OTLP stream exceeds fixture bound")
                    if data:
                        frame(8,0,0,len(data).to_bytes(4,"big"));frame(8,0,stream,len(data).to_bytes(4,"big"))
                    if flags&1:
                        capture.observe(bytes(pending.pop(stream)))
                        frame(1,4,stream,b"\x88\x0f\x10\x10application/grpc")
                        frame(0,0,stream,b"\0\0\0\0\0")
                        frame(1,5,stream,b"\x00\x0bgrpc-status\x010")
        except (EOFError,OSError):
            return
        except (RuntimeError,ValueError,server.SupervisorError) as error:
            with capture.guard:
                if len(capture.errors)<16:
                    capture.errors.append(type(error).__name__)
            return



def capture_operation(state: Path, evidence: Path, command: list[str]) -> dict[str, object]:
    """Explicitly restart selected telemetry, run one literal owned command, restore and drain."""
    if not command or evidence.exists():
        raise server.SupervisorError("Trace capture requires a literal operation and fresh evidence destination")
    deadline=time.monotonic()+420
    before=server.selected_service(state,deadline)
    config=server.config_for(state,deadline=deadline)
    if config.get("diagnostic_telemetry") is not None:
        raise server.SupervisorError("Another diagnostic telemetry capture is already selected")
    evidence.mkdir(mode=0o700,parents=True)
    nonce=uuid.uuid4().hex
    result:dict[str,object]={"before":before.identity,"command":command,"capture_nonce":nonce,"workload":"not_run","restoration":"not_run"}
    with Capture() as capture:
        thread=threading.Thread(target=capture.serve_forever,daemon=True);thread.start()
        selection={"endpoint":f"http://127.0.0.1:{capture.server_address[1]}","capture_nonce":nonce}
        active=None
        try:
            result["activation"]=server.restart_diagnostic_configuration(state,{"diagnostic_telemetry":selection},"telemetry-capture","trace",deadline)
            active=server.selected_service(state,deadline)
            observation:dict[str,object]={}
            result["workload"]=native_operation.run_child_scope(command,dict(os.environ),deadline=min(deadline-120,time.monotonic()+180),observation=observation)
            result["process"]=observation
            active.recheck(deadline)
        finally:
            try:
                if active is not None:
                    active.recheck(deadline)
                    if server.config_for(state,deadline=deadline).get("diagnostic_telemetry")!=selection:
                        raise server.SupervisorError("Trace capture selection changed; restoration refused")
                    result["restoration"]=server.restart_diagnostic_configuration(state,{"diagnostic_telemetry":None},"telemetry-restore","trace",deadline)
            finally:
                capture.shutdown();thread.join(timeout=5);capture.server_close()
                result.update(exported_bytes=capture.total,spans=capture.spans,omitted_spans=capture.omitted_spans,capture_errors=capture.errors,
                    scope="selected server spans and any correlated module spans only; an application Rust exporter/SDK trace-context propagation remains unestablished; no claim of complete distributed causality")
                server.write_json(evidence/"capture.json",result)
    return result
