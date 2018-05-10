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
    fn encode(&self, output: &mut protobuf::CodedOutputStream) -> Result<(), protobuf::ProtobufError>;
    fn encode_to_bytes(&self) -> Result<Vec<u8>, protobuf::ProtobufError> {
        let mut bytes = Vec::new();
        {
            let mut output = protobuf::CodedOutputStream::new(&mut bytes);
            self.encode(&mut output)?;
            output.flush()?;
        }
        Ok(bytes)
    }
}


impl RPCEncodable for bool {
    fn encode(&self, output: &mut protobuf::CodedOutputStream) -> Result<(), protobuf::ProtobufError> {
        output.write_bool_no_tag(*self)
    }
}

impl RPCEncodable for f64 {
    fn encode(&self, output: &mut protobuf::CodedOutputStream) -> Result<(), protobuf::ProtobufError> {
        output.write_double_no_tag(*self)
    }
}

impl RPCEncodable for f32 {
    fn encode(&self, output: &mut protobuf::CodedOutputStream) -> Result<(), protobuf::ProtobufError> {
        output.write_float_no_tag(*self)
    }
}

impl<T, U> RPCEncodable for (T, U)
    where T: RPCEncodable,
          U: RPCEncodable
{
    fn encode(&self, output: &mut protobuf::CodedOutputStream) -> Result<(), protobuf::ProtobufError> {
        let &(ref t, ref u) = self;
        t.encode(output)?;
        u.encode(output)?;
        output.flush()?;
        Ok(())
    }
}

impl<T, U, V> RPCEncodable for (T, U, V)
    where T: RPCEncodable,
          U: RPCEncodable,
          V: RPCEncodable
{
    fn encode(&self, output: &mut protobuf::CodedOutputStream) -> Result<(), protobuf::ProtobufError> {
        let &(ref t, ref u, ref v) = self;
        t.encode(output)?;
        u.encode(output)?;
        v.encode(output)?;
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
    fn encode(&self, output: &mut protobuf::CodedOutputStream) -> Result<(), protobuf::ProtobufError> {
        let &(ref t, ref u, ref v, ref w) = self;
        t.encode(output)?;
        u.encode(output)?;
        v.encode(output)?;
        w.encode(output)?;
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
        let mut input = protobuf::CodedInputStream::from_bytes(proc_result.get_value());
        RPCExtractable::extract_value(&mut input).map_err(RPCFailure::ProtobufFailure)
    }
}
