pub mod client;
pub use client::RPCClient;
pub use client::RPCRequest;

pub mod stream;
pub use stream::StreamClient;
pub use stream::StreamUpdate;

pub mod error;

// Re-exported for the generated code
pub mod codec;
pub mod krpc;
pub use protobuf;

#[doc(hidden)]
#[macro_export]
macro_rules! batch_call_common {
    ($process_result:expr, $client:expr, ( $( $call:expr ),+ )) => {{
        let mut request = $crate::client::RPCRequest::default();
        $( request.add_call($call); )+
        match $client.submit_request(request) {
            Err(e) => {
                Err(e)
            }
            Ok(ref response) => {
                let mut _i = 0;
                Ok(( $({
                        let result = $call.get_result(response, _i); _i += 1;
                        $process_result(result)
                },)+ ))
            }
        }
    }};
}

/// Groups calls in a single packet. The return value is a tuple of `Result`s, one for each call.
///
/// # Example:
/// ```rust,ignore
///let client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000")?;
///let (vessel, time) = batch_call!(&client, (
///    &space_center::get_active_vessel(),
///    &space_center::get_ut(),
///))?;
/// ```
#[macro_export]
macro_rules! batch_call {
    ($client:expr, ( $( $call:expr ),+ $(,)? )) => {
        $crate::batch_call_common!(|result| { result }, $client, ( $( $call ),+ ))
    };
}

/// Does the same as [`batch_call!`] but unwraps all values automatically.
#[macro_export]
macro_rules! batch_call_unwrap {
    ($client:expr, ( $( $call:expr ),+ $(,)? )) => {{
        $crate::batch_call_common!(|result: ::std::result::Result<_, _>| { result.unwrap() }, $client, ( $( $call ),+ ))
    }};
}
