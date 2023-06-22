# Fluxion

Distributed actor framework written in rust.

## About

Fluxion is an actor framework designed with distributed systems in mind, namely sending messages not just between actors, but also between systems.
Fluxion implements three different methods of communication:
- Messages are unique to a specific actor and are used when communicating with that actor. They can send and recieve responses, and are sent to a single instance of an actor.
- Federated Messages are generic: every actor implements the same data type as a Federated Message. They can send and recieve responses, and are sent to a single instance of an actor.
- Notifications are generic to every actor. They can not recieve response, and are sent to every instance of every actor.

Fluxion allows an external task to subscribe to an mpsc channel over which messages bound to Foreign actors are sent. This external task can then relay them as it sees fit. See the example `foreign` for more details.

## Usage

In Fluxion, there are multiple ways to create a system. This depends on if you want to use Federated Messages and/or Notifications. In the future this will be behind a feature flag, but for now a placeholder type that is automatically implemented is used. Creating a system in Fluxion is simple:
```rust
// Create a system that uses both federated messages and notifications
let system = fluxion::system::new::<FederatedMessage, Notification>("id");

// Notifications only
let system = fluxion::system::new_notifications::<Notification>("id");

// Federated messages only
let system = fluxion::system::new_federated::<FederatedMessage>("id");

// Neither
let system = fluxion::system::new_none("id");
```
### Defining Notifications, Messages, and Federated Messages
Defining Notifications is even easier. You don't have too. The `Notification` trait is automatically implemented for any type which is `Clone + Send + Sync + 'static`. Derive `Clone` and you are good to go. Both Messages and Federated Messages are defined using the `Message` trait, and can be any type as long as they implement `Any + Clone + Send + Sync + 'static`. They require a response, but if you don't need to send any, `()` can be used.
```rust
/// A test message
#[derive(Clone)]
struct TestMessage;

impl fluxion::Message for TestMessage {
    type Response = ();
}
```
### Defining Actors
Actors are slightly more complicated to define. They require initialization, deinitialization, and cleanup methods. The function `initialize` is called whenever the actor is started. `deinitialize` is called whenever the actor stops during normal operation, either due to a shutdown or an error. `cleanup` is always called, even when `initialize` or `deinitialize` fail. The `Actor` trait requies the actor be ` Send + Sync + 'static`, and also requires the `async_trait` crate until async functions in traits are stabilized.
```rust
/// A test actor
struct TestActor;

#[async_trait::async_trait]
impl fluxion::Actor for TestActor {
    /// Called upon actor initialization, when the supervisor begins to run.
    async fn initialize<F: Message, N: Notification>(
        &mut self,
        context: &mut ActorContext<F, N>,
    ) -> Result<(), ActorError> {
        Ok(())
    }

    /// Called upon actor deinitialization, when the supervisor stops.
    /// Note that this will not be called if the initialize function fails.
    /// For handling cases of initialization failure, use [`Actor::cleanup`]
    async fn deinitialize<F: Message, N: Notification>(
        &mut self,
        context: &mut ActorContext<F, N>,
    ) -> Result<(), ActorError> {
        Ok(())
    }

    /// Called when the actor supervisor is killed, either as the result of a graceful shutdown
    /// or if initialization fails.
    async fn cleanup(&mut self) -> Result<(), ActorError> {
        Ok(())
    }
}
```

### Implementing Handlers for Actors
If you created your system without using Notifications and/or Federated Messages, you will not need to implement those handlers on your actor. Each communication method has its own trait which can be implemented on an actor:
```rust
#[async_trait::async_trait]
impl HandleNotification<()> for TestActor {
    async fn notified<F: Message>(
        &mut self,
        _context: &mut ActorContext<F, ()>,
        _notification: (),
    ) -> Result<(), ActorError> {
        println!("notification");
        Ok(())
    }
}

#[async_trait::async_trait]
impl HandleMessage<TestMessage> for TestActor {
    async fn message<F: Message, N: Notification>(
        &mut self,
        context: &mut ActorContext<F, N>,
        _message: TestMessage,
    ) -> Result<(), ActorError> {
        println!("message recieved by actor {:?}", context.get_path());
        Ok(())
    }
}

#[async_trait::async_trait]
impl HandleFederated<FederatedMessageType> for TestActor {
    async fn federated_message<N: Notification>(
        &mut self,
        context: &mut ActorContext<TestFederated, N>,
        _message: FederatedMessageType,
    ) -> Result<(), ActorError> {
        println!("federated message recieved by actor {:?}", context.get_path());
        Ok(())
    }
}
```
While multiple of each handler *can* be implemented for a single actor, which handler is used depends on the actor's initialization. For example: if multiple message handlers are implemented, then only the message used when the actor was originally added to the System will work. Multiple Notifications or Federated Messages will just use the system's type by default.

### Adding actors to the system
As simple as
```rust
let handle = system.add_actor(TestActor, "actor_id", SupervisorErrorPolicy::default())
    .await
    .unwrap();
```
This will return an actor handle that allows messages and federated messages to be sent to the actor like this:
```rust
// Send a message
handle.send(TestMessage).await.unwrap();

// Send a message and wait for a response
let resp = handle.request(TestMessage).await.unwrap();

// Same for federated messages

// Send a federated message
handle.send_federated(FederatedMessage).await.unwrap();

// Send a federated message and wait for a response
let resp = handle.request_federated(FederatedMessage).await.unwrap();
```
Note that if the program terminates after sending a message without waiting for a response, then the recipient may not recieve the message.

### Retrieving an actor from the system again
Actors can be retrieved from the system at a later time using their id and message type:
```rust
// Get an actor from the system
let handle = system.get_actor::<TestMessage>("id").await.unwrap();
```
The `context` passed to an actor also implements `get_actor` to enable inter actor communication.

### Foreign Actors
Foreign actors are actors running on other systems. The logic for handling the transmission of foreign messages is entirely up to the user. Take a look at the `foreign` example for more information. Foreign actors can be retrieved by using `get_actor` with a path to another actor. Each colon-separated value is a system, with the final one being the actor's id. This means that `system1:system2:actor` would mean that "the actor `actor` can be accessed by having the system `system1` ask `system2` to relay the message to `actor`".
```rust
// Get the channel over which foreign messages are sent
let foreign_channel = system..get_foreign().await.unwrap();

// Get a handle to a foreign actor
let handle = system.get_actor::<TestMessage>("system1:system2:actor").await.unwrap();
```

## License
Fluxion is Dual-Licensed under Apache 2.0 and MIT.