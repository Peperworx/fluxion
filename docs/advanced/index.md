---
title: Advanced Usage
---

Fluxion has a few advanced features that may be useful in some cases. If you find any of these features lacking, pull requests and issues are especially welcome for these features.

## Tracing

Fluxion provides support for [tracing](https://github.com/tokio-rs/tracing).
This can be enabled using the `tracing` feature flag, with no changes needed to your code other than to register a tracing subscriber. 

## Error Policies

Sometimes you may want to ignore certain errors that may crop up, or maybe retry an operation before failing.
Fluxion implements a very simple error policy system that allows custom error handling logic to be implemented.
After enabling the `error-policy` feature flag, you should make sure that all of your error types implement at least `Debug` and `Display`. Because Fluxion is `no_std`, we do not require `std::error::Error` to be implemented, and our error types do not implement it either. To fix this solution until the `Error` trait is added to `core`, check out [dispair](https://github.com/peperworx/dispair).

Once the feature flag is enabled and your error types implement the required traits, the default error policy is to ignore errors, logging them (if tracing is enabled) and continuing on with execution. This is the same behaviour if `error-policy` is not enabled. To define a custom error policy, you can define an associated `const` in your `Actor`'s trait implementation:

```rust
const ERROR_POLICY: ErrorPolicy<ActorError<Self::Error>> = error_policy! {
    ignore;
};
```

A simple DSL is provided by the `error_policy` macro. See the API documentation for details.