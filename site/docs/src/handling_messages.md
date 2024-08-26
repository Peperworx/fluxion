# Handling Messages# Defining and Sending Messages

Implementing a message handler on an actor is relatively simple:

```rust
use fluxion::Handler;

impl Handler<MyMessage> for MyActor {
    async fn handle_message<D: Delegate>(&self, message: TestMessage, context: &ActorContext<D>) {
        println!("{:?} received by {}", message, context.get_id());
    }
}

```

Message handlers have access to the message, and to a context that provides information about the current actor, as well as access to the system to create more actors and send further messages.

Sending a message is pretty simple. Using the local handle we previously retrieved, we can send any message type that is handled by the actor:

```rust
use fluxion::MessageSender;

actor_ref.send(MyMessage).await;
```

We do not need to unwrap this call, because message sending will never error, and just returns the type dictated by the message's result. In this case, we used `()`.


Now we can also retrieve a `MessageSender`, which can only send a specific message type:

```rust
let actor_ref = system.get::<MyActor, MyMessage>(id).await.unwrap();
actor_ref.send(MyMessage).await;
```

We will look closer at `MessageSender`s when we get to foreign messages in the future. Our final code for this section, with thorough comments, can be found in the `simple` example on github.