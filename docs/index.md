---
title: Fluxion
---

![fluxion](assets/fluxion_wide.png)

Fluxion is an actor framework, written in Rust, that allows communication between actors on different systems.

# Why Fluxion?

Fluxion is designed for a very specific usecase: creating apps that require extremely flexible plugin solutions and communication between different running instances of the app. If you do not need any of the specific features provided by Fluxion, you are probably best off using a different actor system. I personally recommend [Actix](https://github.com/actix/actix), which is battle tested and performant, neither of which describes Fluxion.

# Core Features

Fluxion embraces a few core features, and tries to keep an API that is as extensible as possible. Some of Fluxion's core features include

## No-Std Support

Fluxion and its dependencies only depend on `core` and `alloc`. This means that Fluxion can be used in `no-std` environments, as long as an allocator is available.

## Executor Agnosticism

Fluxion does not depend on a specific async executor. Instead, a user-defined type implements the `Executor` trait, which Fluxion then uses to spawn tasks.

## Foreign Messages

Fluxion's core feature is being able to send messages between systems. Fluxion serializes messages and sends them over a channel, which a separate task can dispatch, allowing custom routing logic for different applications.

## Serialization Formats

Fluxion does not rely on a specific serialization format for  foreign messages, instead allowing users to define their own to use.
