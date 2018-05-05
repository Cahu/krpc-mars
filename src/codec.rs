use krpc;
use protobuf;

use std::io::Read;


#[derive(Debug, Fail)]
pub enum RPCFailure
{
    //#[fail(display = "Some unknown error occured")]
    //SomeFailure,

    #[fail(display = "Procedure failure: {:?}", _0)]
    ProcFailure(krpc::Error),

    #[fail(display = "Protobuf failure: {:?}", _0)]
    ProtobufFailure(protobuf::ProtobufError),
}

pub trait RPCExtractable: Sized {
    fn extract_value(input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError>;
}

impl RPCExtractable for bool {
    fn extract_value(input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError> {
        input.read_bool()
    }
}

impl RPCExtractable for () {
    fn extract_value(_input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError> {
        Ok(())
    }
}

impl RPCExtractable for f64 {
    fn extract_value(input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError> {
        input.read_double()
    }
}

impl RPCExtractable for f32 {
    fn extract_value(input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError> {
        input.read_float()
    }
}

impl<T, U> RPCExtractable for (T, U)
    where T: RPCExtractable,
          U: RPCExtractable
{
    fn extract_value(input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError> {
        let t = RPCExtractable::extract_value(input)?;
        let u = RPCExtractable::extract_value(input)?;
        Ok((t, u))
    }
}

impl<T, U, V> RPCExtractable for (T, U, V)
    where T: RPCExtractable,
          U: RPCExtractable,
          V: RPCExtractable
{
    fn extract_value(input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError> {
        let t = RPCExtractable::extract_value(input)?;
        let u = RPCExtractable::extract_value(input)?;
        let v = RPCExtractable::extract_value(input)?;
        Ok((t, u, v))
    }
}

impl<T, U, V, W> RPCExtractable for (T, U, V, W)
    where T: RPCExtractable,
          U: RPCExtractable,
          V: RPCExtractable,
          W: RPCExtractable
{
    fn extract_value(input: &mut protobuf::CodedInputStream) -> Result<Self, protobuf::ProtobufError> {
        let t = RPCExtractable::extract_value(input)?;
        let u = RPCExtractable::extract_value(input)?;
        let v = RPCExtractable::extract_value(input)?;
        let w = RPCExtractable::extract_value(input)?;
        Ok((t, u, v, w))
    }
}


pub trait RPCEncodable {
    fn encode_value(output: &mut protobuf::CodedOutputStream, val: &Self) -> Result<(), protobuf::ProtobufError>;
    fn encode_value_to_bytes(val: &Self) -> Result<Vec<u8>, protobuf::ProtobufError> {
        let mut bytes = Vec::new();
        {
            let mut output = protobuf::CodedOutputStream::new(&mut bytes);
            RPCEncodable::encode_value(&mut output, val)?;
            output.flush()?;
        }
        Ok(bytes)
    }
}

impl RPCEncodable for bool {
    fn encode_value(output: &mut protobuf::CodedOutputStream, val: &Self) -> Result<(), protobuf::ProtobufError> {
        output.write_bool_no_tag(*val)
    }
}

impl RPCEncodable for f64 {
    fn encode_value(output: &mut protobuf::CodedOutputStream, val: &Self) -> Result<(), protobuf::ProtobufError> {
        output.write_double_no_tag(*val)
    }
}

impl RPCEncodable for f32 {
    fn encode_value(output: &mut protobuf::CodedOutputStream, val: &Self) -> Result<(), protobuf::ProtobufError> {
        output.write_float_no_tag(*val)
    }
}

impl<T, U> RPCEncodable for (T, U)
    where T: RPCEncodable,
          U: RPCEncodable
{
    fn encode_value(output: &mut protobuf::CodedOutputStream, val: &Self) -> Result<(), protobuf::ProtobufError> {
        let &(ref t, ref u) = val;
        RPCEncodable::encode_value(output, t)?;
        RPCEncodable::encode_value(output, u)?;
        output.flush()?;
        Ok(())
    }
}

impl<T, U, V> RPCEncodable for (T, U, V)
    where T: RPCEncodable,
          U: RPCEncodable,
          V: RPCEncodable
{
    fn encode_value(output: &mut protobuf::CodedOutputStream, val: &Self) -> Result<(), protobuf::ProtobufError> {
        let &(ref t, ref u, ref v) = val;
        RPCEncodable::encode_value(output, t)?;
        RPCEncodable::encode_value(output, u)?;
        RPCEncodable::encode_value(output, v)?;
        output.flush()?;
        Ok(())
    }
}

impl<T, U, V, W> RPCEncodable for (T, U, V, W)
    where T: RPCEncodable,
          U: RPCEncodable,
          V: RPCEncodable,
          W: RPCEncodable
{
    fn encode_value(output: &mut protobuf::CodedOutputStream, val: &Self) -> Result<(), protobuf::ProtobufError> {
        let &(ref t, ref u, ref v, ref w) = val;
        RPCEncodable::encode_value(output, t)?;
        RPCEncodable::encode_value(output, u)?;
        RPCEncodable::encode_value(output, v)?;
        RPCEncodable::encode_value(output, w)?;
        output.flush()?;
        Ok(())
    }
}

pub fn read_message<M>(sock: &mut Read) -> Result<M, protobuf::ProtobufError>
    where M: protobuf::Message + protobuf::MessageStatic
{
    let mut input_stream = protobuf::CodedInputStream::new(sock);
    protobuf::parse_length_delimited_from::<M>(&mut input_stream)
}

pub fn extract<T>(proc_result: &krpc::ProcedureResult) -> Result<T, RPCFailure>
    where T: RPCExtractable
{
    if proc_result.has_error() {
        Err(RPCFailure::ProcFailure(proc_result.get_error().clone()))
    }
    else {
        let mut input = protobuf::CodedInputStream::from_bytes(&proc_result.value);
        RPCExtractable::extract_value(&mut input).map_err(RPCFailure::ProtobufFailure)
    }
}

pub fn encode_value_to_bytes<T: RPCEncodable>(value: &T) -> Result<Vec<u8>, protobuf::ProtobufError> {
    RPCEncodable::encode_value_to_bytes(value)
}
