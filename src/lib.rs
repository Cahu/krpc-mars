#[macro_use]
pub extern crate failure;

pub extern crate protobuf;
use protobuf::Message;

pub mod krpc;
pub mod codec;
pub mod rpcfailure;

use rpcfailure::RPCFailure;

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
pub struct StreamClient_ {
    sock: Mutex<TcpStream>,
}

#[derive(Clone)]
pub struct StreamClient (Arc<StreamClient_>);


type StreamID = u64;

pub struct StreamHandle<T> {
    stream_id: StreamID,
    parent_client: RPCClient,
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct StreamUpdate {
    updates: HashMap<StreamID, krpc::ProcedureResult>,
}


impl RPCClient {

    pub fn connect<A: ToSocketAddrs>(client_name: &str, addr: A) -> Result<Self, RPCFailure> {
        let mut sock = TcpStream::connect(addr).map_err(RPCFailure::IoFailure)?;

        let mut conn_req = krpc::ConnectionRequest::new();
        conn_req.set_field_type(krpc::ConnectionRequest_Type::RPC);
        conn_req.set_client_name(client_name.to_string());

        conn_req.write_length_delimited_to_writer(&mut sock).map_err(RPCFailure::ProtobufFailure)?;

        let response = codec::read_message::<krpc::ConnectionResponse>(&mut sock).map_err(RPCFailure::ProtobufFailure)?;

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
                Err(RPCFailure::SomeFailure(format!("{:?} - {}", s, response.message)))
            }
        }
    }

    pub fn make_proc_call(&self, proc_call: krpc::ProcedureCall) -> Result<krpc::Response, RPCFailure> {
        let mut calls = protobuf::RepeatedField::<krpc::ProcedureCall>::new();
        calls.push(proc_call);

        let mut request = krpc::Request::new();
        request.set_calls(calls);

        self.submit_request(&request)
    }

    pub fn make_stream_req(&self, stream_proc_call: krpc::ProcedureCall) -> Result<krpc::Response, RPCFailure> {
        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service(String::from("KRPC"));
        proc_call.set_procedure(String::from("AddStream"));

        let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();

        let mut arg = krpc::Argument::new();
        arg.set_position(0);
        arg.set_value(stream_proc_call.write_to_bytes().map_err(RPCFailure::ProtobufFailure)?);
        arguments.push(arg);

        proc_call.set_arguments(arguments);

        self.make_proc_call(proc_call)
    }

    pub fn submit_request(&self, request: &krpc::Request) -> Result<krpc::Response, RPCFailure> {
        if let Ok(mut sock_guard) = self.0.sock.lock() {
            request.write_length_delimited_to_writer(&mut *sock_guard).map_err(RPCFailure::ProtobufFailure)?;
            codec::read_message::<krpc::Response>(&mut *sock_guard).map_err(RPCFailure::ProtobufFailure)
        }
        else {
            Err(RPCFailure::SomeFailure(String::from("Poinsoned mutex")))
        }
    }
}


impl StreamClient {
    pub fn connect<A: ToSocketAddrs>(client: &RPCClient, addr: A) -> Result<Self, RPCFailure> {
        let mut sock = TcpStream::connect(addr).map_err(RPCFailure::IoFailure)?;

        let mut conn_req = krpc::ConnectionRequest::new();
        conn_req.set_field_type(krpc::ConnectionRequest_Type::STREAM);
        conn_req.set_client_identifier(client.0.client_id.clone());

        conn_req.write_length_delimited_to_writer(&mut sock).map_err(RPCFailure::ProtobufFailure)?;

        let response = codec::read_message::<krpc::ConnectionResponse>(&mut sock).map_err(RPCFailure::ProtobufFailure)?;

        match response.status {
            krpc::ConnectionResponse_Status::OK => {
                Ok(StreamClient(Arc::new(StreamClient_ { sock: Mutex::new(sock) })))
            }
            s => {
                Err(RPCFailure::SomeFailure(format!("{:?} - {}", s, response.message)))
            }
        }
    }

    pub fn recv_update(&mut self) -> Result<StreamUpdate, RPCFailure> {
        let updates;
        if let Ok(mut sock_guard) = self.0.sock.lock() {
            updates = codec::read_message::<krpc::StreamUpdate>(&mut *sock_guard).map_err(RPCFailure::ProtobufFailure)?;
        }
        else {
            return Err(RPCFailure::SomeFailure(String::from("Poinsoned mutex")));
        }

        let mut map = HashMap::new();
        for mut result in updates.results.into_iter() {
            map.insert(result.id, result.take_result());
        }

        Ok(StreamUpdate { updates: map })
    }
}


impl<T> StreamHandle<T> {
    pub fn new(stream_id: StreamID, parent_client: RPCClient) -> Self {
        StreamHandle { stream_id, parent_client, _phantom: PhantomData }
    }
}


impl StreamUpdate {
    pub fn get_result<T>(&self, handle: &StreamHandle<T>) -> Result<T, RPCFailure>
        where T: codec::RPCExtractable
    {
        let result = self.updates.get(&handle.stream_id).ok_or(RPCFailure::NoSuchStream)?;
        codec::extract_result(&handle.parent_client, &result)
    }
}
