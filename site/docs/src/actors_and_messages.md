# Defining Actors

Any type that is `Send`, `Sync`, and `'static` can be an actor. Lets go ahead and define a unit struct, and use the `actor` macro to implement the `Actor` trait automatically.

```rust
use fluxion::actor;

#[actor]
struct MyActor;
```

## Adding the Actor to the System

Adding actors to the system is rather simple:

```rust
let id = system.add(MyActor).await.unwrap();
```

This runs the actor's initialization method, adds the actor to the system, and returns the actor's ID.

The actor's ID can be used to retrieve a reference to the actor from the system. There are two ways to retrieve an actor from the system: `get` and `get_local`. We will take a look at `get_local` first:

```rust
let actor_ref = system.get_local::<MyActor>(id).await.unwrap();
```

The `get_local` method requires only the actor's type and ID, and returns a concrete type that depends on the actor's type. This allows a single actor reference to send any message that the actor implements, however, this message must reside on the local system.

`get`, on the other hand, enables foreign messages, but also requires that the message type is specified. The returned type, however, is abstracted over handlers of the message type.

To use `get`, we must first define a message type.

# Defining Messages

Messages have similar requirements to actors, and we can use a macro to define them as well:
```rust
#[message(())]
struct MyMessage;
```

The above code defines `MyMessage` as a message with the response type `()`.
Messages also each have an ID, which we can optionally set:
```rust
#[message((), "my_message")]
struct MyMessage;
```

We will just use the following code, however, as it sets the message's response to the default `()` and the message's ID to its full module path (in this case, `project_name::MyMessage`):
```rust
#[message]
struct Mymessage;
```