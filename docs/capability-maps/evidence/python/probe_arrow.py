"""Probes 1, 2, 8 — the Arrow/Python boundary. Measures what §4.3, §4.4, §21.1 and §21.5 assert."""
import pyarrow as pa, numpy as np, io

class SemanticId(pa.ExtensionType):
    def __init__(self):
        super().__init__(pa.binary(16), "pse.semantic_id")
    def __arrow_ext_serialize__(self): return b"{}"
    @classmethod
    def __arrow_ext_deserialize__(cls, storage_type, serialized): return cls()

def make_table(with_ext):
    idt = SemanticId() if with_ext else pa.binary(16)
    f_id = pa.field("id", idt, nullable=False, metadata={b"pse.semantic.role": b"key"})
    f_v  = pa.field("v", pa.float64(), nullable=True,
                    metadata={b"pse.semantic.logical_type": b"f64",
                              b"pse.semantic.quantity_type": b"qt-demo"})
    schema = pa.schema([f_id, f_v], metadata={b"pse.contract.id": b"rel-1",
                                              b"pse.contract.version": b"1"})
    ids = pa.array([b"\x00"*15 + bytes([i]) for i in range(3)], pa.binary(16))
    if with_ext: ids = pa.ExtensionArray.from_storage(idt, ids)
    vals = pa.array([1.0, None, -0.0], pa.float64())
    return pa.table([ids, vals], schema=schema)

print("== PROBE 1: extension type through IPC, registered vs not")
pa.register_extension_type(SemanticId())
t = make_table(True)
buf = io.BytesIO()
with pa.ipc.new_stream(buf, t.schema) as w: w.write_table(t)
raw = buf.getvalue()
rt = pa.ipc.open_stream(io.BytesIO(raw)).read_all()
print("   registered   -> field type:", type(rt.schema.field("id").type).__name__,
      "| extension_name:", getattr(rt.schema.field("id").type, "extension_name", None))
pa.unregister_extension_type("pse.semantic_id")
rt2 = pa.ipc.open_stream(io.BytesIO(raw)).read_all()
ft = rt2.schema.field("id").type
print("   UNregistered -> field type:", type(ft).__name__,
      "| extension_name:", getattr(ft, "extension_name", None),
      "| storage:", getattr(ft, "storage_type", ft))
print("   unregistered field metadata retains ARROW:extension:name:",
      b"ARROW:extension:name" in (rt2.schema.field("id").metadata or {}))
pa.register_extension_type(SemanticId())

print("== PROBE 2: schema and field metadata across IPC and PyCapsule")
print("   IPC  schema md :", rt.schema.metadata.get(b"pse.contract.id"))
print("   IPC  field  md :", rt.schema.field("v").metadata.get(b"pse.semantic.quantity_type"))
reader = pa.RecordBatchReader.from_stream(t)          # exercises __arrow_c_stream__
cap = pa.table(reader)
print("   caps schema md :", cap.schema.metadata.get(b"pse.contract.id"))
print("   caps field  md :", cap.schema.field("v").metadata.get(b"pse.semantic.quantity_type"))
print("   caps id type   :", type(cap.schema.field("id").type).__name__)

print("== PROBE 8: numpy boundary — zero-copy and nulls")
col = rt.column("v").combine_chunks()
try:
    arr = col.to_numpy(zero_copy_only=True); print("   zero-copy with nulls: OK", arr)
except Exception as e:
    print("   zero-copy with nulls: REFUSED ->", type(e).__name__, str(e)[:90])
arr2 = col.to_numpy(zero_copy_only=False)
print("   zero_copy_only=False ->", arr2, "| dtype", arr2.dtype, "| null became:", arr2[1])
nonull = pa.array([1.0, 2.0, -0.0], pa.float64())
a3 = nonull.to_numpy(zero_copy_only=True)
print("   no-null zero-copy    ->", a3, "| shares buffer:", a3.base is not None)
print("   -0.0 sign preserved  :", np.signbit(arr2[2]))
