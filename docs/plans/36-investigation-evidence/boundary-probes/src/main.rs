use arrow::{array::{Array, Int64Array, RecordBatch, StringArray, StructArray, UInt32Array}, buffer::Buffer, datatypes::{DataType, Field, Schema}, ipc::{self, reader::StreamDecoder, writer::StreamWriter}};
use std::{sync::Arc, panic::{catch_unwind, AssertUnwindSafe}};
fn encode(batch: &RecordBatch) -> Vec<u8> { let mut out=vec![]; {let mut w=StreamWriter::try_new(&mut out,batch.schema().as_ref()).unwrap(); w.write(batch).unwrap(); w.finish().unwrap();} out }
fn decode(mut buf: Buffer, align:bool)->Result<Vec<RecordBatch>,arrow::error::ArrowError> { let mut decoder=StreamDecoder::new().with_require_alignment(align); let mut batches=vec![]; while !buf.is_empty() {if let Some(batch)=decoder.decode(&mut buf)? {batches.push(batch)}} decoder.finish()?; Ok(batches) }
fn outcome(name:&str, data:Vec<u8>) {let result=catch_unwind(AssertUnwindSafe(||decode(Buffer::from(data),false))); let observed=match result {Ok(Ok(v))=>format!("accepted {} batches",v.len()),Ok(Err(e))=>format!("error: {e}"),Err(_)=>"PANIC".into()};let expected=match name {"valid"|"no_explicit_EOS"|"overlapping_primitive_values"=>"accepted 1 batches","short_struct_validity"|"oversized_struct_node"=>"PANIC",_=>"error:"};assert!(observed.starts_with(expected),"unexpected {name}: {observed}");println!("{}",serde_json::json!({"case":name,"outcome":observed}));}
fn main() {
std::panic::set_hook(Box::new(|_|{}));
let field=Arc::new(Field::new("x",DataType::Int64,true));
let child=Arc::new(Int64Array::from(vec![Some(7),None,Some(11)]));
let structure=StructArray::try_new(vec![field].into(),vec![child],Some(arrow::buffer::NullBuffer::from(vec![true,false,true]))).unwrap();
let batch=RecordBatch::try_new(Arc::new(Schema::new(vec![Field::new("s",structure.data_type().clone(),true)])),vec![Arc::new(structure)]).unwrap();
let bytes=encode(&batch);
let first=i32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
let pos=8+first;
let size=i32::from_le_bytes(bytes[pos+4..pos+8].try_into().unwrap()) as usize;
let (nodes,buffers)={let message=ipc::root_as_message(&bytes[pos+8..pos+8+size]).unwrap(); let b=message.header_as_record_batch().unwrap(); (b.nodes().unwrap().bytes().as_ptr() as usize - bytes.as_ptr() as usize,b.buffers().unwrap().bytes().as_ptr() as usize - bytes.as_ptr() as usize)};
outcome("valid",bytes.clone());
outcome("no_explicit_EOS",bytes[..bytes.len()-8].to_vec());
let mut extra=bytes.clone(); extra.push(1); outcome("trailing_after_EOS",extra);
let mut short=bytes.clone();short[buffers+8..buffers+16].copy_from_slice(&0i64.to_le_bytes());outcome("short_struct_validity",short);
let mut huge=bytes.clone();huge[nodes..nodes+8].copy_from_slice(&i64::MAX.to_le_bytes());outcome("oversized_struct_node",huge);
let mut overlap=bytes.clone();overlap[buffers+32..buffers+40].copy_from_slice(&0i64.to_le_bytes());outcome("overlapping_primitive_values",overlap);
let mut negative=bytes.clone();negative[pos+4..pos+8].copy_from_slice(&(-8i32).to_le_bytes());outcome("negative_metadata_length",negative);
let mut shifted=vec![0];shifted.extend_from_slice(&bytes); let owner=bytes::Bytes::from(shifted);let buffer=Buffer::from(owner.slice(1..));
let allowed=decode(buffer.clone(),false).unwrap();let required=decode(buffer,true).is_err();drop(owner);assert_eq!(allowed[0],batch);println!("{}",serde_json::json!({"case":"unaligned_owned_payload","default_accepts_and_survives_owner_drop":true,"require_alignment_refuses":required}));
let strings=RecordBatch::try_new(Arc::new(Schema::new(vec![Field::new("text",DataType::Utf8,false)])),vec![Arc::new(StringArray::from(vec!["a","bc"]))]).unwrap();
let mut malformed=encode(&strings);let mlen=i32::from_le_bytes(malformed[4..8].try_into().unwrap()) as usize;let mp=8+mlen;let n=i32::from_le_bytes(malformed[mp+4..mp+8].try_into().unwrap()) as usize;
let end={let msg=ipc::root_as_message(&malformed[mp+8..mp+8+n]).unwrap();let b=msg.header_as_record_batch().unwrap();mp+8+n+b.buffers().unwrap().get(1).offset() as usize+8};malformed[end..end+4].copy_from_slice(&(-1i32).to_le_bytes());outcome("malformed_string_offset",malformed);
let primitive=Int64Array::from(vec![Some(1),None,Some(3)]);let indices=UInt32Array::from(vec![2,0,2,1]);let native=arrow::compute::take(&primitive,&indices,Some(arrow::compute::TakeOptions{check_bounds:true})).unwrap();let data=primitive.to_data();let mut copying=arrow::array::MutableArrayData::new(vec![&data],false,0);for i in indices.values(){copying.try_extend(0,*i as usize,*i as usize+1).unwrap();}let copied=arrow::array::make_array(copying.freeze());assert_eq!(native.to_data(),copied.to_data());println!("{}",serde_json::json!({"case":"primitive_take_kernel_equivalence","duplicates_order_nulls":true}));
for bits in [1u64,0x0010000000000001,0x3fd5555555555555,0x7fefffffffffffff,0x8000000000000000] {let x=f64::from_bits(bits);let s=serde_json::to_string(&x).unwrap();let y: f64=serde_json::from_str(&s).unwrap();assert_eq!(x.to_bits(),y.to_bits());}println!("{}",serde_json::json!({"case":"serde_float_roundtrip","exact_values":5,"feature":cfg!(feature="float-roundtrip")}));
let mut random=0x123456789abcdefu64;let mut mismatches=0;let mut first=None;for _ in 0..10000 {random^=random<<13;random^=random>>7;random^=random<<17;let x=f64::from_bits(random);if !x.is_finite(){continue;}let text=serde_json::to_string(&x).unwrap();let y:f64=serde_json::from_str(&text).unwrap();if x.to_bits()!=y.to_bits(){mismatches+=1;if first.is_none(){first=Some(serde_json::json!({"text":text,"input_bits":x.to_bits(),"parsed_bits":y.to_bits()}));}}}println!("{}",serde_json::json!({"case":"serde_shortest_decimal_samples","samples":10000,"mismatches":mismatches,"first":first,"feature":cfg!(feature="float-roundtrip")}));
}
