# First Steps


Lets start by creating a Cargo project (in the current directory) and adding Fluxion:
```sh
cargo init project_name
cd project_name
cargo add fluxion
```

Fluxion needs to be called from an async context. As Fluxion is executor agnostic, it doesn't matter which library is used.
Here we will be using [Tokio](https://tokio.rs/), although any executor will work:
```sh
cargo add tokio --features full
```

Next, we need to make the main function async, and import a few helpers from Fluxion, and create the Fluxion system:

```rust
use fluxion::Fluxion;

#[tokio::main]
async fn main() {
    // Create the Fluxion system:
    let system = Fluxion::new("system_id", ());
}
```

The above code initializes a Fluxion system with the id "system_id" and the delegate `()`.

What is a delegate?

A delegate is an external type that provides methods to retrieve `MessageSender`s, which allow actors to communicate with actors on external systems. The unit type (`()`) is a simple delegate that always returns that no foreign actor was found. Later in this book, we will explore creating our own simple delegate.

Now that we have a system, we need to add an actor to it, and to do that, we must define an actor.



