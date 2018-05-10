use krpc;
use protobuf;

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
