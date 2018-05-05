use krpc;
use codec;

use protobuf;

use failure::Error;

const SERVICE_ID : u32 = 2;


pub trait SpaceCenter {
    fn get_ut(&mut self)                  -> Result<f64, Error>;
    fn get_navball(&mut self)             -> Result<bool, Error>;
    fn set_navball(&mut self, val: bool)  -> Result<(), Error>;
}


impl SpaceCenter for super::RPCClient {

    fn get_ut(&mut self) -> Result<f64, Error> {
        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service_id(SERVICE_ID);
        proc_call.set_procedure_id(35);

        let response = self.make_proc_call(proc_call)?;
        let value = codec::extract(&response.results[0])?;

        Ok(value)
    }

    fn get_navball(&mut self) -> Result<bool, Error> {
        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service_id(SERVICE_ID);
        proc_call.set_procedure_id(33);

        let response = self.make_proc_call(proc_call)?;
        let value = codec::extract(&response.results[0])?;

        Ok(value)
    }

    fn set_navball(&mut self, val: bool) -> Result<(), Error> {
        let mut arg1 = krpc::Argument::new();
        arg1.set_position(0);
        let val = codec::encode_value_to_bytes(&val)?;
        println!("val = {:?}", val);
        arg1.set_value(val);

        let mut arguments = protobuf::RepeatedField::<krpc::Argument>::new();
        arguments.push(arg1);

        let mut proc_call = krpc::ProcedureCall::new();
        proc_call.set_service_id(SERVICE_ID);
        proc_call.set_procedure_id(34);
        proc_call.set_arguments(arguments);

        let response = self.make_proc_call(proc_call)?;
        codec::extract(&response.results[0])?;

        Ok(())
    }
}
