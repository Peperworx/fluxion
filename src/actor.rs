trait Actor {
    /// This can be whatever your application needs
	type Context;

	/// This too can be whatever your application needs, but it is recommended
	/// for it to be the same among all Actors
	type Message;

	/// The main function, which is called every time the actor recieves a message
	async fn handle_message(&mut self, msg: Self::Message, ctx: Option<Self::Context>) -> (Self::Message, Self::Context);
}