use fluxion::{message::Message, system::System, actor::{path::ActorPath, Actor}, error::policy::ErrorPolicy, error_policy};



#[derive(Clone, Debug)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

#[tokio::main]
async fn main() {
    let policy: ErrorPolicy<usize> = error_policy!{
        run;
        loop 10;
        ignoreif 1;
        failif 2;
        ignore;
    };

    println!("{:?}", policy);
}