# Defining and Adding an Actor

Any type that is `Send`, `Sync`, and `'static` can implement `Actor`. Lets just create a unit type, and implement `Actor` on it:

```rust
use fluxion::Actor;

struct MyActor;

impl Actor for MyActor {
    type Error = ();
}
```

The only required field on an `Actor` is the `Error` type. This is the type returned as an error by the optional initialization and deinitialization methods. If you do not define your own initialization/deinitialization methods, you can just leave this as the unit type, as the default implementations will never return an error.

Adding an actor to the system is very easy:

```rust
let id = system.add(MyActor).await.unwrap();
```

This runs the actor's initialization method, adds the actor to the system, and returns the actor's ID.

The actor's ID can be used to retrieve a reference to the actor from the system. There are two ways to retrieve an actor from the system: `get` and `get_local`. We will take a look at `get_local` first:

```rust
let actor_ref = system.get_local::<TestActor>(id).await.unwrap();
```

The `get_local` method requires only the actor's type and ID, and returns a concrete type that depends on the actor's type. This allows a single actor reference to send any message that the actor implements, however, this message must reside on the local system.

`get`, on the other hand, enables foreign messages, but also requires that the message type is specified. The returned type, however, is abstract over handlers of the message type.

To use `get`, we must first define a message type.