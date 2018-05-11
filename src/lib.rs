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


struct RPCClient_ {
    sock: TcpStream,
    _client_id: Vec<u8>,
}

// We must ensure that no two write happen concurrently. Thus we use an Arc<Mutex<>>
#[derive(Clone)]
pub struct RPCClient (Arc<Mutex<RPCClient_>>);


impl RPCClient {

    pub fn connect(addr: &str) -> Result<Self, RPCFailure> {
        let mut sock = TcpStream::connect(addr).map_err(RPCFailure::IoFailure)?;

        let mut conn_req = krpc::ConnectionRequest::new();
        conn_req.set_field_type(krpc::ConnectionRequest_Type::RPC);
        conn_req.set_client_name(String::from("Rigel"));

        conn_req.write_length_delimited_to_writer(&mut sock).map_err(RPCFailure::ProtobufFailure)?;

        let response = codec::read_message::<krpc::ConnectionResponse>(&mut sock).map_err(RPCFailure::ProtobufFailure)?;

        match response.status {
            krpc::ConnectionResponse_Status::OK => {
                Ok(RPCClient(Arc::new(Mutex::new(
                    RPCClient_ { sock, _client_id: response.client_identifier })
                )))
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

    pub fn submit_request(&self, request: &krpc::Request) -> Result<krpc::Response, RPCFailure> {
        if let Ok(ref mut client) = self.0.lock() {
            request.write_length_delimited_to_writer(&mut client.sock).map_err(RPCFailure::ProtobufFailure)?;
            codec::read_message::<krpc::Response>(&mut client.sock).map_err(RPCFailure::ProtobufFailure)
        }
        else {
            Err(RPCFailure::SomeFailure(String::from("Poinsoned mutex")))
        }
    }
}

