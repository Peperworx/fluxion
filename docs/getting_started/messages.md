# Creating a Message

Creating a message type is super simple. Any type can be a message, and just has to implement the `Message` trait:

```rust
use fluxion::Message;

struct TestMessage;

impl Message for TestMessage {
    type Result = ();
}
```
The response can be any type that is Send, Sync, and `static`, here we just choose the unit type.

# Defining a Message Handler

For an actor to handle a message, it must define a message handler. This is pretty simple:

```rust
impl Handler<TestMessage> for TestActor {
    async fn handle_message<D: Delegate>(&self, _message: TestMessage, _context: &ActorContext<D>) {
        println!("Test message received!");
        
    }
}
```

# Sending a Message to an Actor

To send a message to an actor, 