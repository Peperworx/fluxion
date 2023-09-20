# Fluxion
[![crates.io](https://img.shields.io/crates/l/fluxion?style=for-the-badge)](https://crates.io/crates/fluxion)
[![crates.io](https://img.shields.io/crates/v/fluxion?style=for-the-badge)](https://crates.io/crates/fluxion)
[![docs.rs](https://img.shields.io/docsrs/fluxion?style=for-the-badge)](https://docs.rs/fluxion)

Distributed actor framework written in rust.

## About

Fluxion is an actor framework designed with distributed systems in mind, namely sending messages not just between actors, but also between systems.


## Compromises and Limitations

When designing Fluxion, an issue was encountered: to establish communication between a message sender and an actor, a shared type must be known. There are two options for this: the type of the Actor may be shared, or the type of the Message. When the type of the Actor is shared, many different `Handle`s may be implemented for the actor, and all of them may be used. On the other hand, if the type of the Message is shared then the actor may only receive a single Message type. The reasoning behind this is rather complicated, but put simply: a channel must send and receive a concrete type.

The winner seems obvious at first: share the type of the Actor, not the Message. However, this removes an important abstraction and now requires that an Actor's type be known. For example: No longer can an actor be treated as a sender/receiver of `DatabaseMessages`, now it must be known as `ThisSpecificDatabaseActor`. To abstract that away, generics are needed, and generics can get very messy, very quickly.

An additional issue is determining which type foreign messages should deserialize to. This can also get very messy with multiple message types, so Fluxion constrains each actor to handling a single message type. This limitation, however, has a work around. The "main" actor can receive messages in the form of an enum, and then a second actor can "relay" messages to the main actor. This can be implemented generically, so Fluxion will contain default a `Relay<Message, MessageEnum>` implementation.

## License
Fluxion is Dual-Licensed under Apache 2.0 and MIT.