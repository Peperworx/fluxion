#![no_std]
use fluxion::{types::{actor::Actor, Handle, errors::ActorError, message::{MessageHandler, MessageWrapper}}, supervisor::Supervisor};

extern crate alloc;
use alloc::boxed::Box;

struct TestActor;

impl Actor for TestActor {
    type Error = ();
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl Handle<()> for TestActor {
    async fn message(
        &self,
        _message: &(),
    ) -> Result<(), ActorError<Self::Error>> {
        //println!("message");
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let a = TestActor;

    let (supervisor, sender) = Supervisor::new(a);

    tokio::spawn(async move {
        
        loop {
            supervisor.tick().await;
        }
    });

    let (mh, rx) = MessageWrapper::new(());
}