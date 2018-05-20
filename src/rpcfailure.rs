use std::io;

use krpc;
use protobuf;

#[derive(Debug, Fail)]
pub enum RPCFailure
{
    #[fail(display = "Error: {}", _0)]
    SomeFailure(String),

    #[fail(display = "IO error: {:?}", _0)]
    IoFailure(io::Error),

    #[fail(display = "Procedure failure: {:?}", _0)]
    RequestFailure(krpc::Error),

    #[fail(display = "Procedure failure: {:?}", _0)]
    ProcFailure(krpc::Error),

    #[fail(display = "No such stream")]
    NoSuchStream,

    #[fail(display = "Protobuf failure: {:?}", _0)]
    ProtobufFailure(protobuf::ProtobufError),
}
