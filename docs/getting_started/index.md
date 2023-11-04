---
title: Getting Started
---

Fluxion provides a very flexible and extensible API. This comes at the cost of much more verbose initialization logic, all of which is covered in this chapter.
All of the code samples used in this chapter are contained in the `getting_started` example on GitHub.

## Adding Fluxion To Your Project

Fluxion is available on [crates.io](https://crates.io/crates/fluxion).

You can add Fluxion to your crate by running the following command:
```sh
cargo add fluxion
```

Or by adding the following to your `Cargo.toml`:
``` toml
fluxion = "0.8"
```

## Basic Configuration

Fluxion needs to pass around several types via generics to many different structs at runtime. Instead of having many different generics everywhere, Fluxion uses different associated types on a single trait, which is then passed around as a single generic (`C`, for Configuration).

Lets create a new unit struct that implements this trait:

``` rust
use fluxion::FluxionParams;

/// Our Fluxion instance's configuration parameters
struct FluxionConfig;

/// The actual configuration parameters
impl FluxionParams for FluxionConfig {
    /// The async executor that Fluxion should use
    type Executor = TokioExecutor;

    /// The serialization mechanism that Fluxion should use for foreign messages.
    type Serializer = BincodeSerializer;
}
```

Here we define the executor and serialization mechanism that Fluxion should use. We will define these two types in the next pages.