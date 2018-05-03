use krpc;
use codec;

use protobuf;

use failure::Error;

const SERVICE_ID : u32 = 2;


pub fn get_ut(client: &mut super::RPCClient) -> Result<f64, Error> {
    let mut proc_call = krpc::ProcedureCall::new();
    proc_call.set_service_id(SERVICE_ID);
    proc_call.set_procedure_id(35);

    let mut calls = protobuf::RepeatedField::<krpc::ProcedureCall>::new();
    calls.push(proc_call);

    let mut request = krpc::Request::new();
    request.set_calls(calls);

    let response = client.submit_request(&request)?;
    let value = codec::extract(&response.results[0])?;

    Ok(value)
}
