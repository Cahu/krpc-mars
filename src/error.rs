use std::io;
use std::fmt;
use std::error;
use std::result;

use krpc;
use protobuf;

#[derive(Debug)]
pub enum Error
{
    /// Raised when the connection to the RPC server fails.
    RPCConnect { error: String, status: krpc::ConnectionResponse_Status },

    /// Raised when the connection to the stream server fails.
    StreamConnect { error: String, status: krpc::ConnectionResponse_Status },

    /// A synchronization error. Mutexes are used to ensure that responses are not mixed together
    /// so this should only be raised when one mutex is poisoned.
    Synchro(String),

    /// A failure while performing IO.
    Io(io::Error),

    /// Raised when kRPC rejects a request
    Request(krpc::Error),

    /// Raised when kRPC rejects a procedure call
    Procedure(krpc::Error),

    /// Serialization/deserialization error.
    Protobuf(protobuf::ProtobufError),

    /// Error returned when an attempt is made to extract a value from a stream with no result for
    /// the given stream handle.
    NoSuchStream,

    #[doc(hidden)]
    __Nonexhaustive,
}

pub type Result<T> = result::Result<T, Error>;


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::RPCConnect { ref error, ref status } => {
                write!(f, "Could not connect to the RPC server: {:?} {}", status, error.as_str())
            },
            Error::StreamConnect { ref error, ref status } => {
                write!(f, "Could not connect to the stream server: {:?} {}", status, error.as_str())
            },
            Error::Io(ref err) => {
                write!(f, "IO error: {}", err)
            },
            Error::Request(ref err) => {
                write!(f, "The RPC request failed: service={} procedure={} description={}", err.get_service(), err.get_name(), err.get_description())
            },
            Error::Procedure(ref err) => {
                write!(f, "The RPC failed: service={} procedure={} description={}", err.get_service(), err.get_name(), err.get_description())
            },
            Error::Protobuf(ref err) => {
                write!(f, "Protobuf error: {}", err)
            },
            Error::NoSuchStream => {
                write!(f, "No result for this stream")
            },
            _ => unreachable!(),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::RPCConnect    { ref error, .. } => error.as_str(),
            Error::StreamConnect { ref error, .. } => error.as_str(),
            Error::Io(ref err)               => err.description(),
            Error::Request(ref err)   => err.get_description(),
            Error::Procedure(ref err) => err.get_description(),
            Error::Protobuf(ref err)  => err.description(),
            Error::NoSuchStream              => "Stream not found",
            _ => unreachable!(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            Error::RPCConnect { .. }     => None,
            Error::StreamConnect { .. }  => None,
            Error::Io(ref err)           => Some(err),
            Error::Request(_)            => None,
            Error::Procedure(_)          => None,
            Error::Protobuf(ref err)     => Some(err),
            Error::NoSuchStream          => None,
            _ => unreachable!(),
        }
    }
}
