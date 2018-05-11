#[macro_use]
pub extern crate failure;

pub extern crate protobuf;
use protobuf::Message;

pub mod krpc;
pub mod codec;
pub mod rpcfailure;

use rpcfailure::RPCFailure;
use std::net::TcpStream;


pub struct RPCClient {
    sock: TcpStream,
    _client_id: Vec<u8>,
}

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
                Ok(RPCClient { sock, _client_id: response.client_identifier })
            }
            s => {
                Err(RPCFailure::SomeFailure(format!("{:?} - {}", s, response.message)))
            }
        }
    }

    pub fn make_proc_call(&mut self, proc_call: krpc::ProcedureCall) -> Result<krpc::Response, RPCFailure> {
        let mut calls = protobuf::RepeatedField::<krpc::ProcedureCall>::new();
        calls.push(proc_call);

        let mut request = krpc::Request::new();
        request.set_calls(calls);

        self.submit_request(&request)
    }

    pub fn submit_request(&mut self, request: &krpc::Request) -> Result<krpc::Response, RPCFailure> {
        request.write_length_delimited_to_writer(&mut self.sock).map_err(RPCFailure::ProtobufFailure)?;
        codec::read_message::<krpc::Response>(&mut self.sock).map_err(RPCFailure::ProtobufFailure)
    }
}

