//! Client to the KRPC Stream server.
use crate::codec;
use crate::krpc;

use crate::client::CallHandle;
use crate::error::Error;
use crate::error::Result;

use std::net::TcpStream;
use std::net::ToSocketAddrs;

use std::marker::PhantomData;

use std::collections::HashMap;

use protobuf::Message;

pub(crate) type StreamID = u64;

/// A client to the Stream server.
#[derive(Debug)]
pub struct StreamClient {
    sock: TcpStream,
}

/// A handle to a stream. The type parameter is the type of the value produced by the stream.
#[derive(Copy, Clone, Debug)]
pub struct StreamHandle<T> {
    pub(crate) stream_id: StreamID,
    _phantom: PhantomData<T>,
}

impl<T> StreamHandle<T> {
    #[doc(hidden)]
    /// Creates a new StreamHande. The function is public so that the generated code from
    /// krpc-mars-terraformer can use it but it is hidden from user docs.
    pub fn new(stream_id: StreamID) -> Self {
        StreamHandle {
            stream_id,
            _phantom: PhantomData,
        }
    }

    /// Creates an RPC request that will remove this stream.
    pub fn remove(self) -> CallHandle<()> {
        use codec::RPCEncodable;

        let mut arg = krpc::Argument::new();
        arg.set_position(0);
        arg.set_value(self.stream_id.encode_to_bytes().unwrap());

        let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();
        arguments.push(arg);

        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service(String::from("KRPC"));
        proc_call.set_procedure(String::from("RemoveStream"));
        proc_call.set_arguments(arguments);

        CallHandle::<()>::new(proc_call)
    }
}

/// Creates a stream request from a CallHandle. For less verbosity, you can use the
/// [`CallHandle::to_stream`] instead.
///
/// Note that types don't prevent you from chaining multiple `mk_stream`. This will build a stream
/// request of a stream request. Turns out this is accepted by the RPC server and the author of
/// this library confesses he had some fun with this.
pub fn mk_stream<T: codec::RPCExtractable>(call: &CallHandle<T>) -> CallHandle<StreamHandle<T>> {
    let mut arg = krpc::Argument::new();
    arg.set_position(0);
    arg.set_value(call.get_call().write_to_bytes().unwrap());

    let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();
    arguments.push(arg);

    let mut proc_call = krpc::ProcedureCall::new();
    proc_call.set_service(String::from("KRPC"));
    proc_call.set_procedure(String::from("AddStream"));
    proc_call.set_arguments(arguments);

    CallHandle::<StreamHandle<T>>::new(proc_call)
}

impl StreamClient {
    /// Connect to the stream server associated with the given client.
    pub fn connect<A: ToSocketAddrs>(client: &super::RPCClient, addr: A) -> Result<Self> {
        let mut sock = TcpStream::connect(addr)?;

        let mut conn_req = krpc::ConnectionRequest::new();
        conn_req.set_field_type(krpc::ConnectionRequest_Type::STREAM);
        conn_req.set_client_identifier(client.client_id.clone());

        conn_req.write_length_delimited_to_writer(&mut sock)?;

        let mut response = codec::read_message::<krpc::ConnectionResponse>(&mut sock)?;

        match response.status {
            krpc::ConnectionResponse_Status::OK => Ok(Self { sock }),
            s => Err(Error::StreamConnect {
                error: response.take_message(),
                status: s,
            }),
        }
    }

    pub fn recv_update(&mut self) -> Result<StreamUpdate> {
        let updates = codec::read_message::<krpc::StreamUpdate>(&mut self.sock)?;

        let mut map = HashMap::new();
        for mut result in updates.results.into_iter() {
            map.insert(result.id, result.take_result());
        }

        Ok(StreamUpdate { updates: map })
    }
}

/// A collection of updates received from the stream server.
#[derive(Debug, Clone, Default)]
pub struct StreamUpdate {
    updates: HashMap<StreamID, krpc::ProcedureResult>,
}

impl StreamUpdate {
    pub fn get_result<T>(&self, handle: &StreamHandle<T>) -> Result<T>
    where
        T: codec::RPCExtractable,
    {
        let result = self
            .updates
            .get(&handle.stream_id)
            .ok_or(Error::NoSuchStream)?;
        codec::extract_result(&result)
    }

    /// Merge two update objects. The Stream server doesn't update values that don't change, so
    /// this can be used to retain previous values of streams.
    pub fn merge_with(&mut self, other: StreamUpdate) {
        self.updates.extend(other.updates)
    }
}
