use fluxion::{message::Message, system::System, actor::{path::ActorPath, Actor}};



#[derive(Clone, Debug)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

#[tokio::main]
async fn main() {
    let system = System::<TestMessage, ()>::new("test");

    /* 
    // Get the foreign channel
    let f = system.get_foreign().await.unwrap();

    tokio::spawn(async move {
        let mut f = f;
        loop {
            if let Some(res) = f.recv().await {
                println!("recieved foreign {res:?}");
            }
        }
    });*/

    println!("{}", system.is_foreign(ActorPath::new("test2:test").unwrap()));
}