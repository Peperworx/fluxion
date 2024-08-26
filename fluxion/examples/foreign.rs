use std::sync::Arc;

use fluxion::{actor, message, Delegate, Handler, Identifier, Message, MessageID, MessageSender};
use maitake_sync::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Receiver, Sender};


#[actor]
struct ActorA;

impl Handler<MessageA> for ActorA {
    async fn handle_message<D: fluxion::Delegate>(&self, _message: MessageA, context: &fluxion::ActorContext<D>) -> <MessageA as fluxion::Message>::Result {
        println!("Actor {}:{} received {}", context.system().get_id(), context.get_id(), MessageA::ID);
    }
}
impl Handler<MessageB> for ActorA {
    async fn handle_message<D: fluxion::Delegate>(&self, _message: MessageB, context: &fluxion::ActorContext<D>) -> <MessageB as fluxion::Message>::Result {
        println!("Actor {}:{} received {}", context.system().get_id(), context.get_id(), MessageB::ID);
    }
}

#[actor]
struct ActorB;

impl Handler<MessageA> for ActorB {
    async fn handle_message<D: fluxion::Delegate>(&self, _message: MessageA, context: &fluxion::ActorContext<D>) -> <MessageA as fluxion::Message>::Result {
        println!("Actor {}:{} received {}", context.system().get_id(), context.get_id(), MessageA::ID);
    }
}

impl Handler<MessageB> for ActorB {
    async fn handle_message<D: fluxion::Delegate>(&self, _message: MessageB, context: &fluxion::ActorContext<D>) -> <MessageB as fluxion::Message>::Result {
        println!("Actor {}:{} received {}", context.system().get_id(), context.get_id(), MessageB::ID);
    }
}


#[message]
#[derive(Serialize, Deserialize)]
struct MessageA;

#[message]
#[derive(Serialize, Deserialize)]
struct MessageB;


struct DelegateMessageHandler {

}

struct SerdeDelegate {
    // The system's id
    system_id: &'static str,
    // Channel for sending serialized data to the other half of the delegate
    // This is connected to the other delegate's `receiver`
    sender: Sender<Vec<u8>>,
    // Channel for receiving serialized data to the other half of the delegate
    // This is connected to the other delegate's `sender`
    receiver: Receiver<Vec<u8>>,
    // Hashm
}

impl SerdeDelegate {
    pub fn new(system_id: &'static str, sender: Sender<Vec<u8>>, receiver: Receiver<Vec<u8>>) -> Self {
        Self {
            system_id,
            sender,
            receiver: receiver,
        }
    }


    /// Registers an actor as being able to receive a specific message type.
    pub async fn register_actor_message<A: Handler<M>, M: fluxion::IndeterminateMessage>(&self, id: u64)
        where M::Result: serde::Serialize + for<'de> serde::Deserialize<'de>{
        println!("{} is registering actor with id {} to handle message {}", self.system_id, id, M::ID);
    }
}

impl Delegate for SerdeDelegate {
    async fn get_actor<'a, A: Handler<M>, M: fluxion::IndeterminateMessage>(&self, id: Identifier<'a>) -> Option<Arc<dyn MessageSender<M>>>
        where M::Result: serde::Serialize + for<'de> serde::Deserialize<'de> {

        // We shouldn't be able to return local ids
        let Identifier::Foreign(id, system) = id else {
            return None;
        };
        
        println!("{} is requesting a foreign actor on system {} with id {} that can handle message {}", self.system_id, system, id, M::ID);
        todo!()
    }
}

#[tokio::main]
async fn main() {

    let a_to_b = mpsc::channel(64);
    let b_to_a = mpsc::channel(64);


    // Initialize the two systems
    let system_a = fluxion::Fluxion::new("system_a", SerdeDelegate::new("system_a", a_to_b.0, b_to_a.1));
    let system_b = fluxion::Fluxion::new("system_b", SerdeDelegate::new("system_b", b_to_a.0, a_to_b.1));

    // Create both actors on system a
    let actor_a = system_a.add(ActorA).await.unwrap();
    system_a.get_delegate().register_actor_message::<ActorA, MessageA>(actor_a).await;
    system_a.get_delegate().register_actor_message::<ActorA, MessageB>(actor_a).await;
    let actor_b = system_a.add(ActorB).await.unwrap();
    system_a.get_delegate().register_actor_message::<ActorB, MessageA>(actor_b).await;
    system_a.get_delegate().register_actor_message::<ActorB, MessageB>(actor_b).await;
   
    // Get both actors on system b
    let foreign_a = system_b.get::<ActorA, MessageB>(Identifier::Foreign(actor_a, "system_a")).await.unwrap();
    let foreign_b = system_b.get::<ActorB, MessageA>(Identifier::Foreign(actor_b, "system_a")).await.unwrap();

    foreign_a.send(MessageB).await;
    foreign_b.send(MessageA).await;
}