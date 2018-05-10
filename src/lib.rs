#[macro_use]
pub extern crate failure;
use failure::Error;

pub extern crate protobuf;
use protobuf::Message;

pub mod krpc;
pub mod codec;
pub mod rpcfailure;

use std::net::TcpStream;


pub struct RPCClient {
    sock: TcpStream,
}

impl RPCClient {

    pub fn connect(addr: &str) -> Result<Self, Error> {
        let mut sock = TcpStream::connect(addr)?;

        let mut conn_req = krpc::ConnectionRequest::new();
        conn_req.set_field_type(krpc::ConnectionRequest_Type::RPC);
        conn_req.set_client_name(String::from("Rigel"));

        conn_req.write_length_delimited_to_writer(&mut sock)?;

        let response = codec::read_message::<krpc::ConnectionResponse>(&mut sock)?;
        println!("response: {:?}", response);

        Ok(RPCClient { sock })
    }

    pub fn make_proc_call(&mut self, proc_call: krpc::ProcedureCall) -> Result<krpc::Response, Error> {
        let mut calls = protobuf::RepeatedField::<krpc::ProcedureCall>::new();
        calls.push(proc_call);

        let mut request = krpc::Request::new();
        request.set_calls(calls);

        self.submit_request(&request)
    }

    pub fn submit_request(&mut self, request: &krpc::Request) -> Result<krpc::Response, Error> {
        request.write_length_delimited_to_writer(&mut self.sock)?;
        let response = codec::read_message::<krpc::Response>(&mut self.sock)?;
        
        Ok(response)
    }
}

