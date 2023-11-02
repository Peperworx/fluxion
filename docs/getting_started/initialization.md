---
title: Initializing Fluxion
---

Now that we have our `FluxionConfig` struct defined in the `config` module, we can initialize a Fluxion instance in main.rs:

```rust
mod config;

use config::FluxionConfig;

use fluxion::Fluxion;

#[tokio::main]
fn main() {
    // Create a Fluxion instance using our configuration, and give it the name `host`
    let system = 
}

```