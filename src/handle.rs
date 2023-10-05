use crate::types::{Handle, errors::{ActorError, SendError}, message::{MessageHandler, MessageSender, Handler, Message}, actor::Actor};
use alloc::boxed::Box;

/// 
pub struct LocalHandle<A: Actor> {
    sender: whisk::Channel<Box<dyn Handler<A>>>,
}

impl<A: Actor> LocalHandle<A> {

    /// Sends a message to the actor and waits for a response
    /// 
    /// # Errors
    /// Returns an error if no response is received
    pub async fn request<M: Message>(&self, message: M) -> Result<M::Response, SendError>
    where
        A: Handle<M> {

        // Create the message handle
        let (mh, rx) = MessageHandler::new(message);

        // Send the handler
        self.sender.send(Box::new(mh)).await;

        // Wait for a response
        rx.await.or(Err(SendError::NoResponse))
    }
}

#[cfg_attr(async_trait, async_trait::async_trait)]
impl<A: Actor + Handle<M>, M: Message> MessageSender<M> for LocalHandle<A> {
    async fn request(&self, message: M) -> Result<<M>::Response, SendError> {
        self.request(message).await
    }
}
