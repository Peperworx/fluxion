//! # Foreign messages
//! This is a rather convoluted example of using foreign messages. It really only exists to show that it can be done.
//! To run this example, make sure to enable the serde and foreign features.

use std::{collections::HashMap, marker::PhantomData, sync::Arc};


use fluxion::{actor, message, Delegate, Handler, Identifier, LocalRef, Message, MessageID, MessageSender};
use maitake_sync::RwLock;
use serde::{Deserialize, Serialize};
use slacktor::{ActorHandle, Slacktor};
use tokio::sync::{mpsc, oneshot};


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


struct DelegateSender<M: Message + MessageID> {
    actor_id: u64,
    other_delegate: ActorHandle<DelegateActor>,
    _phantom: PhantomData<M>,
}

#[async_trait::async_trait]
impl<M: Message + MessageID + Serialize> fluxion::MessageSender<M> for DelegateSender<M>
where M::Result: for<'de> Deserialize<'de> {
   
    async fn send(&self,message:M) -> Result<M::Result, Box<dyn std::error::Error>> {
        // Send the message
        let res = self.other_delegate.send(DelegateMessage(self.actor_id, M::ID.to_string(), bincode::serialize(&message).unwrap())).await;

        // Deserialize the response
        Ok(bincode::deserialize(&res).unwrap())
    }
}

struct DelegateActor(Arc<SerdeDelegate>);

impl slacktor::Actor for DelegateActor {}

impl slacktor::actor::Handler<DelegateMessage> for DelegateActor {
    async fn handle_message(&self, message: DelegateMessage) -> <DelegateMessage as Message>::Result  {
        
        // Get the channel
        let handlers = self.0.actor_handlers.read().await;
        let channels = handlers.get(&(message.0, message.1)).unwrap();

        // Construct the response oneshot
        let (response_sender, response) = oneshot::channel();

        // Send the message
        channels.send((message.2, response_sender)).await.unwrap();

        // Wait for the response and return
        response.await.unwrap()
    }
}

struct DelegateMessage(u64, String, Vec<u8>);

impl slacktor::Message for DelegateMessage {
    type Result = Vec<u8>;
}

struct SerdeDelegate {
    // The system's id
    system_id: &'static str,
    // Backing slacktor instance
    slacktor: Arc<RwLock<Slacktor>>,
    // The other delegate's id,
    other_id: usize,
    // Hashmap of message handling channels for actors
    actor_handlers: RwLock<HashMap<(u64, String), mpsc::Sender<(Vec<u8>, oneshot::Sender<Vec<u8>>)>>>
}


impl SerdeDelegate {
    pub fn new(system_id: &'static str, slacktor: Arc<RwLock<Slacktor>>, other_id: usize) -> Self {
        Self {
            system_id,
            slacktor,
            other_id,
            actor_handlers: Default::default()
        }
    }


    /// Registers an actor as being able to receive a specific message type.
    pub async fn register_actor_message<A: Handler<M>, M: fluxion::IndeterminateMessage, S: Delegate + AsRef<Self>>(&self, actor: LocalRef<A, S>)
        where M::Result: serde::Serialize + for<'de> serde::Deserialize<'de>{
        
        let id = actor.get_id();

        println!("{} is registering actor with id {} to handle message {}", self.system_id, id, M::ID);

        // Create channels
        let (send_message, mut receive_message) = mpsc::channel::<(Vec<u8>,_)>(64);

        // Spawn task
        tokio::spawn(async move {
            loop {
                let Some(next_message): Option<(Vec<u8>, oneshot::Sender<Vec<u8>>)> = receive_message.recv().await else {
                    println!("Message handler {}/{} stopped recieving messages.", actor.get_id() ,M::ID);
                    break;
                };

                // Decode the message
                let decoded: M = bincode::deserialize(&next_message.0).unwrap();

                // Handle the message
                let res = actor.send(decoded).await.expect("this delegate doesn't error");

                // Send the response
                next_message.1.send(bincode::serialize(&res).unwrap()).unwrap();
            }
        });

        // Add the handler
        self.actor_handlers.write().await.insert((id, M::ID.to_string()), send_message);

    }
}

impl Delegate for SerdeDelegate {
    async fn get_actor<'a, A: Handler<M>, M: fluxion::IndeterminateMessage>(&self, id: Identifier<'a>) -> Option<Arc<dyn MessageSender<M>>>
        where M::Result: serde::Serialize + for<'de> serde::Deserialize<'de> {

        // We shouldn't be able to return local ids.
        // Ignore named IDs for now.
        let Identifier::Foreign(id, system) = id else {
            return None;
        };
        
        println!("{} is requesting a foreign actor on system {} with id {} that can handle message {}", self.system_id, system, id, M::ID);

        // Get the other actor on the slacktor system
        let slacktor = self.slacktor.read().await;
        let other = slacktor.get::<DelegateActor>(self.other_id)?;
        
        // Wrap with a message sender and return
        Some(Arc::new(DelegateSender {
            actor_id: id,
            other_delegate: other.clone(),
            _phantom: PhantomData,
        }))
    }
}

#[tokio::main]
async fn main() {

    // This basic example just uses slacktor as the communication medium between delegates.
    // In practice, any system that can provide request/response semantics can be used to create a delegate.
    let delegate_backplane = Arc::new(RwLock::new(Slacktor::new()));
    let backplane = delegate_backplane.clone();
    let mut backplane = backplane.write().await;

    let delegate_a = Arc::new(SerdeDelegate::new("system_a", delegate_backplane.clone(), 1));
    let delegate_b = Arc::new(SerdeDelegate::new("system_a", delegate_backplane, 0));

    backplane.spawn(DelegateActor(delegate_a.clone()));
    backplane.spawn(DelegateActor(delegate_b.clone()));

    // Drop slacktor, or else the delegates will hang forever.
    drop(backplane);


    // Initialize the two systems
    let system_a = fluxion::Fluxion::new("system_a", delegate_a);
    let system_b = fluxion::Fluxion::new("system_b", delegate_b);

    // Create both actors on system a
    let actor_a = system_a.add(ActorA).await.unwrap();
    system_a.get_delegate().register_actor_message::<ActorA, MessageA, _>(system_a.get_local(actor_a).await.unwrap()).await;
    system_a.get_delegate().register_actor_message::<ActorA, MessageB, _>(system_a.get_local(actor_a).await.unwrap()).await;
    let actor_b = system_a.add(ActorB).await.unwrap();
    system_a.get_delegate().register_actor_message::<ActorB, MessageA, _>(system_a.get_local(actor_b).await.unwrap()).await;
    system_a.get_delegate().register_actor_message::<ActorB, MessageB, _>(system_a.get_local(actor_b).await.unwrap()).await;

    
   
    // Get both actors on system b
    let foreign_a = system_b.get::<ActorA, MessageB>(Identifier::Foreign(actor_a, "system_a")).await.unwrap();
    let foreign_b = system_b.get::<ActorB, MessageA>(Identifier::Foreign(actor_b, "system_a")).await.unwrap();

    foreign_a.send(MessageB).await.expect("this delegate doesn't error");
    foreign_b.send(MessageA).await.expect("this delegate doesn't error");
}