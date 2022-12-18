use krpc;
use protobuf;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Raised when the connection to the RPC server fails.
    #[error("Could not connect to the RPC server: {error} {status:?}")]
    RPCConnect {
        error: String,
        status: krpc::ConnectionResponse_Status,
    },

    /// Raised when the connection to the stream server fails.
    #[error("Could not connect to the stream server: {error} {status:?}")]
    StreamConnect {
        error: String,
        status: krpc::ConnectionResponse_Status,
    },

    /// A synchronization error. Mutexes are used to ensure that responses are not mixed together
    /// so this should only be raised when one mutex is poisoned.
    #[error("Sync errro: {0}")]
    Synchro(String),

    /// A failure while performing IO.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Raised when kRPC rejects a request
    #[error("The RPC request failed: service={} procedure={} description={}", .0.get_service(), .0.get_name(), .0.get_description())]
    Request(krpc::Error),

    /// Raised when kRPC rejects a procedure call
    #[error("The RPC failed: service={} procedure={} description={}", .0.get_service(), .0.get_name(), .0.get_description())]
    Procedure(krpc::Error),

    /// Serialization/deserialization error.
    #[error(transparent)]
    Protobuf(#[from] protobuf::ProtobufError),

    /// Error returned when an attempt is made to extract a value from a stream with no result for
    /// the given stream handle.
    #[error("No such stream")]
    NoSuchStream,
}

pub type Result<T> = std::result::Result<T, Error>;
