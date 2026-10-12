"""Disposable Plan 36 boundary, effects and filesystem controls; no product effects."""
import errno
import fcntl
import hashlib
import importlib.metadata
import os
from pathlib import Path
import struct
import sys
import tempfile
import time
from types import ModuleType

import msgspec
# Load actual codec/contracts with a namespace-only package shell. The ID/hash codecs
# still load the installed native extension; no NativeRuntime, Arrow batch or solver runs.
package = ModuleType("pse")
package.__path__ = [str(Path(__file__).resolve().parents[3] / "python" / "pse")]
sys.modules["pse"] = package
from pse import codec
from pse.contracts import authored, documents, enums
from pse.contracts.values import ContentHash, SemanticId


def converter():
    value = codec.converter()
    value.register_unstructure_hook(SemanticId, SemanticId.to_hex)
    value.register_unstructure_hook(ContentHash, ContentHash.to_prefixed)
    return value


def conversion():
    doc = documents.AuthoredModelingDeclarationsRow(
        declaration_id="1" * 32, document_id="2" * 32, is_override=False,
        name="p", ordinal=0, source_start=0, source_end=1,
        value=documents.AuthoredModelingDeclarationsFieldValue(kind=enums.ModelingDeclarationKind.PACKAGE, scope=documents.AuthoredModelingDeclarationsFieldValueScope(bases=(), facets=(), parameters=(), type_parameters=())),
    )
    row = codec.document_rows([doc], authored.AuthoredModelingDeclarationsRow)[0]
    private = converter()
    caller = codec.converter()
    caller.register_unstructure_hook(SemanticId, lambda _: "corrupted")
    assert private.unstructure(row)["declaration_id"] == "1" * 32
    assert caller.unstructure(row)["declaration_id"] == "corrupted"
    def current(rows):
        conv = converter()
        plain = {"declarations": tuple(conv.unstructure(value) for value in rows)}
        return codec.encode_json(codec.decode_json(codec.encode_json(plain), documents.DeclarationEdit))
    def target(rows):
        plain = {"declarations": tuple(private.unstructure(value) for value in rows)}
        return codec.encode_json(msgspec.convert(plain, type=documents.DeclarationEdit, strict=True))
    for size in [1,128]:
        rows = [row] * size
        assert current(rows) == target(rows)
        observations = []
        for operation in [current,target]:
            start = time.perf_counter()
            for _ in range(8):
                operation(rows)
            observations.append(round((time.perf_counter()-start)*1000,3))
        print(f"conversion rows={size} iterations=8 current_ms={observations[0]} private_direct_ms={observations[1]} equivalent=passed")
    fit = documents.AuthoredFitCasesRow(
        fit_id="3" * 32,
        parameters=(documents.AuthoredFitCasesFieldParametersItem(symbol_id="4" * 32, fixed=False, value=-0.0, scale=2.0, lower=-10.0, upper=10.0),),
        experiments=(documents.AuthoredFitCasesFieldExperimentsItem(experiment_id="5" * 32, case_id="6" * 32, route=enums.ModelingAnalysisRoute.STEADY, bindings=()),),
        observations=(documents.AuthoredFitCasesFieldObservationsItem(observation_id="7" * 32, experiment_id="5" * 32, output_path="unit.x", value_attribute="value", importance=1.0, included=True),),
    )
    fit_row = codec.document_rows([fit], authored.AuthoredFitCasesRow)[0]
    for size in [1,128]:
        fresh = converter()
        plain = {"fits": tuple(fresh.unstructure(fit_row) for _ in range(size))}
        old = codec.encode_json(codec.decode_json(codec.encode_json(plain), documents.FitDeclarations))
        plain_private = {"fits": tuple(private.unstructure(fit_row) for _ in range(size))}
        new = codec.encode_json(msgspec.convert(plain_private, type=documents.FitDeclarations, strict=True))
        assert old == new
    print("fit nested-parameter/experiment/observation/identity/enum/signed-zero rows=1,128 equivalent=passed")
    for extra in [{"declarations":[],"unknown":True}, {"declarations":[{"declaration_id":"bad"}]}]:
        for operation in [lambda x: codec.decode_json(codec.encode_json(x), documents.DeclarationEdit),lambda x: msgspec.convert(x,type=documents.DeclarationEdit,strict=True)]:
            try:
                operation(extra)
            except msgspec.ValidationError:
                pass
            else:
                raise AssertionError("bad shape accepted")
    print("caller-hook-isolation/unknown-field/bad-shape=passed; version-header equivalence not tested by this fixture")


def filesystem():
    with tempfile.TemporaryDirectory(prefix="plan36-validity-", dir=Path(__file__).parent) as directory:
        file = Path(directory) / "artifact"
        file.write_bytes(b"original")
        file.chmod(0o444)
        before = file.stat()
        # Same owner can undo read-only permissions: managed chmod is a protocol, not kernel sealing.
        file.chmod(0o644)
        file.write_bytes(b"replaced")
        os.utime(file, ns=(before.st_atime_ns,before.st_mtime_ns))
        after = file.stat()
        assert (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns)==(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns)
        assert hashlib.sha256(b"original").digest()!=hashlib.sha256(file.read_bytes()).digest()
        assert before.st_ctime_ns != after.st_ctime_ns
        print("readonly-owner-mutation=passed; dev/inode/size/mtime stat-hint counterexample=passed; ctime changed (no full-stat forgery claimed)")
        file.chmod(0o444)
        args = struct.pack("<IIIIQIIQ11Q",1,1,4096,0,0,0,0,0,*([0]*11))
        assert len(args)==128
        fd = os.open(file,os.O_RDONLY)
        try:
            try:
                fcntl.ioctl(fd,0x40806685,args)
                print("FS_IOC_ENABLE_VERITY=enabled on owned disposable artifact only")
            except OSError as error:
                print(f"FS_IOC_ENABLE_VERITY=unavailable errno={error.errno} {errno.errorcode.get(error.errno)}")
        finally:
            os.close(fd)


def effects():
    # Target-shaped immutable operation and allocation owners, with an explicit optional effect.
    # No server, retry timer or product API is modeled as already implemented.
    clock = 10
    operation = ("op1", 20)
    owners = {"payload", "receipt", "protection", "settlement"}
    state = "prepared-unpublished"
    assert state != "prepared-persisted"
    state = "publishing"
    dispatched = operation
    clock = 19
    state = "uncertain"
    assert operation == dispatched and owners == {"payload", "receipt", "protection", "settlement"}
    clock = 21
    assert clock > operation[1]  # Retry cannot refresh original expiry or create a new root.
    state = "cancelled-awaiting-drain"
    assert owners
    state = "drained"
    owners.clear()
    assert state == "drained" and not owners
    print("optional-effect separate-outcomes/original-clock/uncertain-owner-retention/cancel-drain=passed (state-machine fixture only)")


if __name__ == "__main__":
    print(f"cattrs={importlib.metadata.version('cattrs')} msgspec={importlib.metadata.version('msgspec')}")
    conversion()
    filesystem()
    effects()
