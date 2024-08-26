/// # Benchmark
/// Extremely simple benchmark for Fluxion's message passing speed.
/// May not be entirely representitive of real usecases.


// Imports from Fluxion that are needed for this example
use fluxion::{message, Actor, ActorContext, Delegate, Fluxion, Handler, Message, MessageID, MessageSender};
use std::time::Instant;


/// # [`TestActor`]
/// For this example, the actor stores a 64-bit integer, which is xored with the data in the message as a result.
/// This 64-bit integer is randomly generated.
struct TestActor(pub u64);

impl Actor for TestActor {
    type Error = ();
}

/// # [`TestMessage`]
/// The 64-bit integer sent to the actor.
#[message(u64)]
struct TestMessage(pub u64);



impl Handler<TestMessage> for TestActor {
    #[inline(always)]
    async fn handle_message<D: Delegate>(&self, message: TestMessage, _context: &ActorContext<D>) -> u64 {
       // XOR the message's value with the value stored in self.
       message.0 ^ self.0
    }
}




#[tokio::main]
async fn main() {
    // Create the system
    let system = Fluxion::new("system", ());
    
    // Add the actor, returning the ID
    let id = system.add(TestActor(rand::random())).await.unwrap();

    // Get a local reference to the actor
    let actor = system.get_local::<TestActor>(id).await.unwrap();

    // Test with 1 billion messages.
    // If this takes too long, lower values also
    // give pretty accurate results.
    let num_messages = 1_000_000_000;

    // Reserve num_messages in the vector.
    // If this uses too much ram, then decrease num_messages.
    let mut out = Vec::with_capacity(num_messages);

    // Begin timing
    let start = Instant::now();

    // Send num_messages messages.
    for i in 0..num_messages {
        // Send the message
        let v = actor.send(TestMessage(i as u64)).await;

        // Add the message to the output vector
        out.push(v);
    }

    // Calculate epalsed duration
    let elapsed = start.elapsed();

    // Calculate and print numbers per second.
    println!(
        "{:.2} messages/sec",
        num_messages as f64 / elapsed.as_secs_f64()
    );
}