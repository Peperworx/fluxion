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

The above code initializes a new Fluxion instance with the id "system id" and an empty delagate, which means that all attempts to retrieve foreign actors will return `None`. Later on we will create a simple delegate which will be able to send messages between multiple systems. Before we get to that point, however, we first need to add an actor to the system. We will do this in the next chapter.