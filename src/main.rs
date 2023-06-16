use fluxion::{message::Message, system::System, actor::{path::ActorPath, Actor}, error_policy, error::policy::ErrorPolicyCommand};



#[derive(Clone, Debug)]
struct TestMessage;

impl Message for TestMessage {
    type Response = ();
}

#[tokio::main]
async fn main() {
    let policy: Vec<ErrorPolicyCommand<u32>> = error_policy!{
        pass;
        loop 10;
        ignoreif 1;
        failif 2;
        ignore;
    };

    println!("{:?}", policy);
}