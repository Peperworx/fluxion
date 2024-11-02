// Imports from Fluxion that are needed for this example
use fluxion::{actor, message, ActorContext, Delegate, Fluxion, Handler, MessageSender};



/// # [`TestActor`]`
/// A unit struct that serves as our test actor.
/// Fluxion doesn't actually care what type you use as an actor.
/// You can use a unit struct like this, a regular struct, tuple struct, or an enum.
/// You can even use generics, and as long as they satisfy [`Send`] + [`Sync`] + `'static` then they will work.
/// 
/// All that actors require is that the `Actor` trait be implemented for them
/// Unless you define custom initialization or deinitialization logic for your actor,
/// the following macro should be used. Otherwise, the [`fluxion::Actor`] trait must
/// be manually implemented.
#[actor]
struct TestActor;


/// # [`TestMessage`]
/// A unit struct that serves as out test message.
/// Just like with actors, messages can be any type.
/// They just need to satisfy [`Send`] + [`Sync`] + `'static`
/// If the `serde` feature is enabled, messages that wish to be able to be sent to foreign actors,
/// as well as use the [`Fluxion::get`] method, must implemment `Serialize` and `Deserialize`.
/// Actors that do not implement these traits can still be accessed with [`Fluxion::get_local`].
/// 
/// Optionally, the message's response type may be provided. The actor's ID may also be provided.
/// Here we use the full syntax, but it can be reduced to simply `#[message]`, and the effect will be the same. 
/// The default response type is `()` and the default ID for a message is it's full module path.
#[message((), "simple::TestMessage")]
struct TestMessage;




// Message handlers are also pretty simple
impl Handler<TestMessage> for TestActor {

    /// The only real complex bit is this function signature.
    /// Even with foreign messages disabled, Fluxion still requies the [`Delegate`]
    /// generic so that enabling and disabling the `foreign` feature doesn't completely mess up the API.
    /// That is, the only implementation detail that should differentiate foreign and local messages
    /// is if they implement serialization (or with no `serde` feature flag, there should be no difference).
    async fn handle_message<D: Delegate>(&self, _message: TestMessage, context: &ActorContext<D>) {
        // Both the contents of the message and the `context` are available here.
        // The context allows the handler to access this actor's ID, as well as a reference to the system.
        println!("Test message received by {}", context.get_id());
    }
}



// Fluxion requires async to run.
// We just use tokio here, as it is the most popular option.
// Fluxion is completely executor agnostic, so you can use async_std, smol, or even write your own executor.
// Additionally, Fluxion is no_std compatible. All that is required is that you provide an allocator,
// and that you call into Fluxion from an async function.
#[tokio::main]
async fn main() {

    // Creating a Fluxion system is easy.
    // Here we create it with an empty delegate (the unit type)
    // which just means that foreign messages are disabled for this system.
    // Any requests for foreign messages will just return [`None`].
    let system = Fluxion::new("system", ());
    
    // Adding an actor to the system assigns it with an ID.
    // You can also provide a name, in this case "test".
    let id = system.add("test", TestActor).await.unwrap();

    // You can use this ID to retrieve a reference to the actor.
    // There are two ways to do this.

    // The first is using the [`Fluxion::get`] method.
    // This only works if your actor is compatible with foreign messages.
    // The only time that a message will not be compatible with foreign messages is if
    // the `serde` feature is enabled, and your message doesn't implement `Serialize` and `Deserialize`.
    // The actor ref returned by this method only supports sending a single message type.
    // Additionally, the type of the actor must be known when you retrieve it, however
    // the returned type doesn not depend on the actor's type.
    // This means that you can get the reference in code that knows about the actor's type,
    // and then use it in code that only knows about the message's type.
    // For example: an external crate that handles database transactions only needs to accept the type
    // `Arc<dyn MessageSender<DatabaseMessage>>` and doesn't need to know about the actor's type.
    let actor_ref = system.get::<TestActor, TestMessage>(id).await.unwrap();

    // You can send messages to actor refs using the send method.
    // This method returns the actor's response to the message.
    // In this case, because we have defined the response as the unit type,
    // we do not need to unwrap anything.
    actor_ref.send(TestMessage).await;

    // We can also retrieve an actor reference using the [`Fluxion::get_local`] method.
    // This only requires the actor's type, and only works on actors runnign on the local system.
    // The returned actor reference can be used for any message type handled by the actor,
    // and can be used in the same way as the actor ref returned by the `get` method.
    let actor = system.get_local::<TestActor>(id).await.unwrap();

    // Use in the same way as above
    actor.send(TestMessage).await;
}