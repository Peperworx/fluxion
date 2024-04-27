---
title: Fluxion
---

![fluxion](assets/fluxion_wide.png)

Fluxion is an actor library that is designed to allow communication between multiple systems.

# Why Fluxion?

Fluxion is designed for a very specific usecase: creating applications that require extremely flexible plugin solutions and communication between different running instances. If you do not need any of the specific features provided by Fluxion, you are probably best off using a different actor system. Fluxion uses [Slacktor](https://github.com/peperworx/slacktor) as a backend to provide "simulated messages" and a core actor system to work off of. Slacktor was purpose built for use in Fluxion, and is provided separately from Fluxion for higher-performance usecases, as well as those that do not need Fluxion's features.

# Core Features

Fluxion embraces a few core features, and tries to keep an API that is as as simple and extensible as possible.
Descriptions about Fluxion's few core features as well as any rationale behind their implementation follows.

## No-Std Support

Fluxion and its dependencies only depend on `core` and `alloc`. This means that Fluxion can be used in `no-std` environments, as long as an allocator is available.

## Executor Agnosticism

Fluxion does not depend on a specific async executor, and the only async dependencies of Fluxion are [maitake_sync] and [async_trait], which are both executor agnostic.
Fluxion is able to remain executor agnostic for three primary reasons:
1. Message handlers can not mutate the actor.
2. Fluxion doesn't support non-blocking messages.
3. Fluxion is built on [Slacktor](https://github.com/peperworx/slacktor), which simulates message passing on top of raw function calls.
   
These reasons mean that Fluxion can be structured such that it never needs to spawn any tasks. This means that Fluxion does not need to access any specific executor library and is completely executor agnostic with no boilerplate required. You can use Tokio, async_std, Smol, or even write your own executor and Fluxion will not care.


## Foreign Messages

Fluxion's core feature is being able to send messages between systems, and is conditional on the `foreign` feature. Fluxion accomplishes this by allowing the user to define a "delegate" (the noun) which responds to requests for specific foreign actors. The delegate returns an implementor of a trait that enables sending messages of a specific type. If the `serde` feature is enabled, then messages will be required to implement serialization functionality to be foreign messages.