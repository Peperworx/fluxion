---
title: Getting Started
---


## Adding Fluxion To Your Project

Fluxion is available on [crates.io](https://crates.io/crates/fluxion).

You can add Fluxion to your crate by running the following command:
```sh
cargo add fluxion
```

## Fluxion's Architecture

Fluxion has a simple architecture consisting of four main parts:
- Systems
- Actors
- Messages
- Delegates

Systems are individual Fluxion instances.
Actors consist of a piece of data and associated message handlers.
Messages are data that is sent between actors and the responses to that data.
Delegates handle communications between systems.

## Creating a System

Creating a Fluxion system is super easy:

```rust
use fluxion::Fluxion;

let system = Fluxion::new("system id", ());
```

The above code initializes a new Fluxion instance with the id "system id" and an empty delagate, which means that all attempts to retrieve foreign actors will return `None`. Later on we will create a simple delegate which will be able to send messages between multiple systems. Before we get to that point, however, we first need to add an actor to the system.

## Defining an Actor

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