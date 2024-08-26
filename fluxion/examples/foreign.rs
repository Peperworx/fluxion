use std::{borrow::BorrowMut, collections::HashMap, sync::Arc};

use fluxion::{actor, message, Delegate, Handler, Identifier, LocalRef, Message, MessageID, MessageSender};
use maitake_sync::{Mutex, RwLock};
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
    receiver: Mutex<Receiver<Vec<u8>>>,
    // Hashmap of message handling channels for actors
    actor_handlers: Mutex<HashMap<(u64, String), (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>)>>
}

impl SerdeDelegate {
    pub fn new(system_id: &'static str, sender: Sender<Vec<u8>>, receiver: Receiver<Vec<u8>>) -> Self {
        Self {
            system_id,
            sender,
            receiver: Mutex::new(receiver),
            actor_handlers: Default::default()
        }
    }


    /// Registers an actor as being able to receive a specific message type.
    pub async fn register_actor_message<A: Handler<M>, M: fluxion::IndeterminateMessage>(&self, actor: LocalRef<A, Self>)
        where M::Result: serde::Serialize + for<'de> serde::Deserialize<'de>{
        
        let id = actor.get_id();

        println!("{} is registering actor with id {} to handle message {}", self.system_id, id, M::ID);

        // Create channels
        let (send_message, mut receive_message) = mpsc::channel::<Vec<u8>>(64);
        let (send_response, receive_response) = mpsc::channel::<Vec<u8>>(64);

        // Spawn task
        tokio::spawn(async move {
            loop {
                let Some(next_message) = receive_message.recv().await else {
                    println!("Message handler {}/{} stopped recieving messages.", actor.get_id() ,M::ID);
                    break;
                };

                // Decode the message
                let decoded: M = bincode::deserialize(&next_message).unwrap();

                // Handle the message
                let res = actor.send(decoded).await;

                // Send the response
                send_response.send(bincode::serialize(&res).unwrap()).await.unwrap();
                
            }
        });

        // Add the handler
        self.actor_handlers.lock().await.insert((id, M::ID.to_string()), (send_message, receive_response));

    }

    /// Runs the delegate
    pub async fn run(&self) {

        // In an actual implementation, there will need to be more synchronization between message/response pairs, as each
        // delegate may actually be handling more than one message in parallel. This will most likely be done by
        // using a separate connection per message, over a protocol like QUIC that can handle the multiplexing for us.

        let mut receiver = self.receiver.lock().await;
        loop {
            // Wait for data
            let Some(next_data) = receiver.recv().await else {
                println!("Delegate disconnected");
                return;
            };

            // Deserialize the data
            let (actor_id, message_id, data): (u64, String, Vec<u8>) = bincode::deserialize(&next_data).unwrap();

            // Get the actor's message handler
            let mut handlers = self.actor_handlers.lock().await;
            let channels = handlers.get_mut(&(actor_id, message_id.to_string())).unwrap();

            // Send the message data
            channels.0.send(data).await.unwrap();

            // Wait for the response
            let res = channels.1.recv().await.unwrap();

            // Send the response
            self.sender.send(res).await.unwrap();
        }
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

        // Create a channel 

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
    system_a.get_delegate().register_actor_message::<ActorA, MessageA>(system_a.get_local(actor_a).await.unwrap()).await;
    system_a.get_delegate().register_actor_message::<ActorA, MessageB>(system_a.get_local(actor_a).await.unwrap()).await;
    let actor_b = system_a.add(ActorB).await.unwrap();
    system_a.get_delegate().register_actor_message::<ActorB, MessageA>(system_a.get_local(actor_b).await.unwrap()).await;
    system_a.get_delegate().register_actor_message::<ActorB, MessageB>(system_a.get_local(actor_b).await.unwrap()).await;
   
    // Get both actors on system b
    let foreign_a = system_b.get::<ActorA, MessageB>(Identifier::Foreign(actor_a, "system_a")).await.unwrap();
    let foreign_b = system_b.get::<ActorB, MessageA>(Identifier::Foreign(actor_b, "system_a")).await.unwrap();

    foreign_a.send(MessageB).await;
    foreign_b.send(MessageA).await;
}