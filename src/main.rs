#![feature(async_fn_in_trait)]

use fluxion::{actor::Actor, handle::Handle};




struct TestActor {
    ctx_count: usize,
}

impl Actor for TestActor {
    type Context = usize;
    type Message = usize;

    async fn handle_message(&mut self, handle: Handle<Self::Message, Self::Context>) -> (Self::Message, Self::Context) {

        let ret_ctx = handle.ctx.unwrap_or({
            self.ctx_count += 1;
            self.ctx_count
        });

        (self.ctx_count, ret_ctx)
    }
}


fn main() {}