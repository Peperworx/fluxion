#![feature(async_fn_in_trait)]

use std::{collections::HashMap, pin::Pin};

struct TestActor {

}

impl Actor for TestActor {
    async fn handle_message(&mut self, msg: Message, _ctx: Option<Context>) -> (Message, Context) {
        match msg {
            Message::Ping => (Message::Pong, Context::default()),
            Message::Pong => (Message::Ping, Context::default()),
        }
    }
}


trait Actor {
	/// The main function, which is called every time the actor recieves a message
	async fn handle_message(&mut self, msg: Message, ctx: Option<Context>) -> (Message, Context);

}

trait ActorWrapper {
    fn handle_message_wrapper(&mut self, msg: Message, ctx: Option<Context>) -> Pin<Box<dyn std::future::Future<Output = (Message, Context)> + '_>>;
}

impl<T: Actor> ActorWrapper for T {
    fn handle_message_wrapper(&mut self, msg: Message, ctx: Option<Context>) -> Pin<Box<dyn std::future::Future<Output = (Message, Context)> + '_>> {
        Box::pin(self.handle_message(msg, ctx))
    }
}

#[derive(Debug, Copy, Clone)]
enum Message {
    Ping,
    Pong
}

#[derive(Debug, Copy, Clone, Default)]
struct Context;

#[derive(Default)]
struct System {
    actors: HashMap<&'static str, Box<dyn ActorWrapper>>,

}

impl System {

    fn add_actor(&mut self, key: &'static str, actor: impl Actor + 'static) {
        self.actors.insert(key, Box::new(actor));
    }
    
    async fn request(&mut self, actor: &'static str, msg: Message, ctx: Option<Context>) -> (Message, Context) {
        // Get the actor
        let a = self.actors.get_mut(actor).unwrap();

        a.handle_message_wrapper(msg, ctx).await
    }
}

#[tokio::main]
async fn main() {
    let mut sys = System::default();

    let new_actor = TestActor {};

    sys.add_actor("a", new_actor);

    let (msg, ctx) = sys.request("a", Message::Pong, None).await;
    println!("{msg:?}");
    println!("{ctx:?}");
}
