use fluxion::{message::Message, system::System, actor::path::ActorPath};



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

    system.force_foreign_send_message(TestMessage, None, ActorPath::new("test").unwrap()).await.unwrap();

    system.force_foreign_send_federated(TestMessage, None, ActorPath::new("test").unwrap()).await.unwrap();

    system.force_foreign_send_notification(()).await.unwrap();
}