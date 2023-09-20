# Fluxion Types

Core types and traits used by every part of Fluxion, including the Actor trait.

## Associated Types as Generics

Due to its nature, Fluxion requires many different feature flags to enable and disable many different generics. This can be a PITA when these generics are used almost everywhere, particularly in `impl`s. This crate contains several traits in the module `atg` that have their associated types enabled and disabled by cfg macros. In the `param` module, there equivalent structs which "convert" the associated types to generics, and are re-exported at the top level of Fluxion for use by the user. This allows generic values to be passed around without needing to constantly redefine different types. 