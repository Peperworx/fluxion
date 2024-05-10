<div align="center" id="centered">

<img style="width: 80%" src="docs/assets/fluxion_wide.png" alt = "Fluxion" class="oranda-hide">

[![crates.io](https://img.shields.io/crates/l/fluxion?style=for-the-badge)](https://crates.io/crates/fluxion)
[![crates.io](https://img.shields.io/crates/v/fluxion?style=for-the-badge)](https://crates.io/crates/fluxion)
[![docs.rs](https://img.shields.io/docsrs/fluxion?style=for-the-badge)](https://docs.rs/fluxion)

Distributed actor framework written in Rust.
</div>


# About

Fluxion is an actor framework designed with distributed systems in mind, namely sending messages not just between actors, but also between systems.

## Why Fluxion?

Fluxion is designed for a very specific usecase: creating applications that require extremely flexible plugin solutions and communication between different running instances. If you do not need any of the specific features provided by Fluxion, you are probably best off using a different actor system. Fluxion uses [Slacktor](https://github.com/peperworx/slacktor) as a backend to provide "simulated messages" and a core actor system to work off of. Slacktor was purpose built for use in Fluxion, and is provided separately from Fluxion for higher-performance usecases, as well as those that do not need Fluxion's features.

## Core Features

Fluxion embraces a few core features, and tries to keep an API that is as as simple and extensible as possible.
Descriptions about Fluxion's few core features as well as any rationale behind their implementation follows.

### No-Std Support

Fluxion and its dependencies only depend on `core` and `alloc`. This means that Fluxion can be used in `no-std` environments, as long as an allocator is available.

### Executor Agnosticism

Fluxion is structured such that it never needs to spawn any tasks. This means that Fluxion does not need to access any specific executor library and is completely executor agnostic with no boilerplate required. You can use Tokio, async_std, Smol, or even write your own executor and Fluxion will not care.

### Foreign Messages

Fluxion's core feature is being able to send messages between systems, and is conditional on the `foreign` feature. Fluxion accomplishes this by allowing the user to define a "delegate" that responds to requests for specific foreign actors. The delegate returns an implementor of a trait that enables sending messages of a specific type. If the `serde` feature is enabled, then messages will be required to implement serialization functionality to be foreign messages.

## Getting Started

The best way to get started right now is to take a look at the `simple` example. It contains the most bare bones possible example for using Fluxion as an actor system with no special features. For more advanced usage, take a look at [the book](https://fluxion.peperworx.com/book/).

## License

Fluxion is Dual-Licensed under Apache 2.0 and MIT.