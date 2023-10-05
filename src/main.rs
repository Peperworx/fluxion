use fluxion::types::message::{MessageHandler, Handler};


trait ActorRef<M: Message> {
    async fn send(&self, message: M) -> M::Response;
}

trait StoredRef {
    async fn send_any(&self, message: Box<dyn Any>);
}



#[tokio::main]
async fn main() {

    let mut refs = Vec::<StoredRef>::new();


    // Get a stored ref
    let sref = refs[0];

    

}