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

Fluxion and all of its dependencies only depend on 