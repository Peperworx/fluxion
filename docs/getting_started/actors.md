# Defining an Actor

All actors must implement the `Actor` trait, so we will include that at the top of our file:
```rust
use fluxion::Actor;
```

An actor can be any type, but we will go with a unit struct for now
```rust
struct TestActor;

impl Actor for TestActor {
    type Error = ();
}
```

Here we see that we are implementing the `Actor` trait for our struct. The `Actor` trait provides methods for initialization that can return errors, which are defined by the `Error` associated type. For now we will set it to `()`. These methods have default no-op implementations, so we do not need to worry about them unless we have a need.

## Adding an Actor to The System

Adding an actor to the system is easy:
```rust
let id = system.add(TestActor).await.unwrap();
```

This function adds the given actor to the system and returns the actor's new id. Unlike previous versions of Fluxion, actor ids are automatically assigned by the system. 

Before we can work with the actor further, we must first define a message and implemment a message handler, which we will do in the next chapter.