If this crate successfully builds, then Fluxion and its dependencies fully support no_std by default.

To successfully build, this requires core to be built/installed for the target `x86_64-unknown-none`.

This can be accomplished via the following command:
```sh
rustup toolchain add x86_64-unknown-none
```