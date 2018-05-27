pub extern crate protobuf;
use protobuf::Message;

pub mod krpc;
pub mod codec;
pub mod error;
pub use error::Error;
pub use error::Result;

use std::sync::Arc;
use std::sync::Mutex;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::collections::HashMap;
use std::marker::PhantomData;


struct RPCClient_ {
    sock: Mutex<TcpStream>, //We must ensure that no two write happen concurrently
    client_id: Vec<u8>,
}

#[derive(Clone)]
pub struct RPCClient (Arc<RPCClient_>);


#[derive(Debug)]
struct StreamClient_ {
    sock: Mutex<TcpStream>,
}

#[derive(Clone)]
pub struct StreamClient (Arc<StreamClient_>);


type StreamID = u64;

#[derive(Clone)]
pub struct RPCRequest {
    calls: protobuf::RepeatedField<krpc::ProcedureCall>,
}

#[derive(Clone)]
pub struct CallHandle<T> {
    proc_call: krpc::ProcedureCall,
    _phantom: PhantomData<T>,
}

#[derive(Copy, Clone, Debug)]
pub struct StreamHandle<T> {
    stream_id: StreamID,
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct StreamUpdate {
    updates: HashMap<StreamID, krpc::ProcedureResult>,
}


#[doc(hidden)]
#[macro_export]
macro_rules! batch_call_common {
    ($process_result:expr, $client:expr, ( $( $call:expr ),+ )) => {{
        let mut request = $crate::RPCRequest::new();
        $( request.add_call($call); )+
        match $client.submit_request(request) {
            Err(e) => {
                Err(e)
            }
            Ok(ref mut response) if response.has_error() => {
                Err($crate::Error::Request(response.take_error()))
            }
            Ok(ref response) => {
                let mut _i = 0;
                Ok(( $({
                        let result = $call.get_result(&response.results[_i]); _i += 1;
                        $process_result(result)
                },)+ ))
            }
        }
    }};
}

#[macro_export]
macro_rules! batch_call {
    ($client:expr, ( $( $call:expr ),+ )) => {
        batch_call_common!(|result| { result }, $client, ( $( $call ),+ ))
    };
    ($client:expr, ( $( $call:expr ),+ , )) => /* retry without last ';' */ {
        batch_call!($client, ( $( $call ),+ ))
    };
}


#[macro_export]
macro_rules! batch_call_unwrap {
    ($client:expr, ( $( $call:expr ),+ )) => {{
        batch_call_common!(|result: $crate::error::Result<_>| { result.unwrap() }, $client, ( $( $call ),+ ))
    }};
    ($client:expr, ( $( $call:expr ),+ , )) => /* retry without last ';' */ {
        batch_call_unwrap!($client, ( $( $call ),+ ))
    };
}


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


impl RPCClient {

    pub fn connect<A: ToSocketAddrs>(client_name: &str, addr: A) -> Result<Self> {
        let mut sock = TcpStream::connect(addr).map_err(Error::Io)?;

        let mut conn_req = krpc::ConnectionRequest::new();
        conn_req.set_field_type(krpc::ConnectionRequest_Type::RPC);
        conn_req.set_client_name(client_name.to_string());

        conn_req.write_length_delimited_to_writer(&mut sock).map_err(Error::Protobuf)?;

        let mut response = codec::read_message::<krpc::ConnectionResponse>(&mut sock).map_err(Error::Protobuf)?;

        match response.status {
            krpc::ConnectionResponse_Status::OK => {
                Ok(RPCClient(Arc::new(
                    RPCClient_ {
                        sock: Mutex::new(sock),
                        client_id: response.client_identifier
                    })
                ))
            }
            s => {
                Err(Error::RPCConnect {
                    error: response.take_message(),
                    status: s,
                })
            }
        }
    }

    pub fn mk_call<T: codec::RPCExtractable>(&self, call: &CallHandle<T>) -> Result<T> {
        let (result,) = batch_call!(self, ( call ))?;
        result
    }

    pub fn submit_request(&self, request: RPCRequest) -> Result<krpc::Response> {
        let raw_request = request.build();
        if let Ok(mut sock_guard) = self.0.sock.lock() {
            raw_request.write_length_delimited_to_writer(&mut *sock_guard).map_err(Error::Protobuf)?;
            codec::read_message::<krpc::Response>(&mut *sock_guard).map_err(Error::Protobuf)
        }
        else {
            Err(Error::Synchro(String::from("Poisoned mutex")))
        }
    }
}


impl StreamClient {
    pub fn connect<A: ToSocketAddrs>(client: &RPCClient, addr: A) -> Result<Self> {
        let mut sock = TcpStream::connect(addr).map_err(Error::Io)?;

        let mut conn_req = krpc::ConnectionRequest::new();
        conn_req.set_field_type(krpc::ConnectionRequest_Type::STREAM);
        conn_req.set_client_identifier(client.0.client_id.clone());

        conn_req.write_length_delimited_to_writer(&mut sock).map_err(Error::Protobuf)?;

        let mut response = codec::read_message::<krpc::ConnectionResponse>(&mut sock).map_err(Error::Protobuf)?;

        match response.status {
            krpc::ConnectionResponse_Status::OK => {
                Ok(StreamClient(Arc::new(StreamClient_ { sock: Mutex::new(sock) })))
            }
            s => {
                Err(Error::StreamConnect {
                    error: response.take_message(),
                    status: s,
                })
            }
        }
    }

    pub fn recv_update(&self) -> Result<StreamUpdate> {
        let updates;
        if let Ok(mut sock_guard) = self.0.sock.lock() {
            updates = codec::read_message::<krpc::StreamUpdate>(&mut *sock_guard).map_err(Error::Protobuf)?;
        }
        else {
            return Err(Error::Synchro(String::from("Poinsoned mutex")));
        }

        let mut map = HashMap::new();
        for mut result in updates.results.into_iter() {
            map.insert(result.id, result.take_result());
        }

        Ok(StreamUpdate { updates: map })
    }
}


impl RPCRequest {
    pub fn new() -> Self {
        RPCRequest { calls: protobuf::RepeatedField::<krpc::ProcedureCall>::new() }
    }

    pub fn add_call<T: codec::RPCExtractable>(&mut self, handle: &CallHandle<T>) {
        self.calls.push(handle.get_call().clone())
    }

    fn build(self) -> krpc::Request {
        let mut req = krpc::Request::new();
        req.set_calls(self.calls);
        req
    }
}


impl<T> CallHandle<T>
    where T: codec::RPCExtractable
{
    pub fn new(proc_call: krpc::ProcedureCall) -> Self {
        CallHandle { proc_call, _phantom: PhantomData }
    }

    pub fn get_result(&self, result: &krpc::ProcedureResult) -> Result<T> {
        codec::extract_result(result)
    }

    fn get_call(&self) -> &krpc::ProcedureCall {
        &self.proc_call
    }
}


impl<T> StreamHandle<T> {
    pub fn new(stream_id: StreamID) -> Self {
        StreamHandle { stream_id, _phantom: PhantomData }
    }

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


impl StreamUpdate {
    pub fn get_result<T>(&self, handle: &StreamHandle<T>) -> Result<T>
        where T: codec::RPCExtractable
    {
        let result = self.updates.get(&handle.stream_id).ok_or(Error::NoSuchStream)?;
        codec::extract_result(&result)
    }
}
