#![cfg_attr(feature="nightly", feature(async_fn_in_trait))]

use fluxion::{message::{Message, SerdeDispatcher}, actor::{Actor, Handle}, error::FluxionError};
use serde::Serialize;
use serde::Deserialize;


struct BincodeDispatcher;

impl SerdeDispatcher for BincodeDispatcher {
    fn dispatch_serialized<M: Message + Serialize + for<'a> Deserialize<'a>, E>(actor: &dyn Actor<Error = E>, message: Vec<u8>) -> Result<Vec<u8>, FluxionError<E>> {
        
        // Use bincode to deserialize the message
        let message: M = bincode::deserialize(&message).or(Err(FluxionError::DeserializeError))?;


        
        todo!()
    }
}


#[derive(Serialize, Deserialize)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();

    type Error = ();

    type Dispatcher = BincodeDispatcher;
}

#[tokio::main]
async fn main() {
    
}