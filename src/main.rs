use fluxion::{message::Message, system::System};



#[derive(Clone, Debug)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

#[tokio::main]
async fn main() {
    let system = System::<TestMessage, ()>::new();

    // Get the foreign channel
    let f = system.get_foreign().await.unwrap();

    tokio::spawn(async move {
        let mut f = f;
        loop {
            if let Some(res) = f.recv().await {
                println!("recieved foreign {res:?}");
            }
        }
    });

    system.force_foreign_send(TestMessage, None).await.unwrap();
}