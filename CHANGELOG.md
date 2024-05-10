# Changelog

## 0.10.0

Version 0.10 is intended to be the final complete overhaul of Fluxion. The only scenario in which another rewrite should be expected is if this version has major usability issues. Otherwise, all future minor versions, as well as version 1.0.0, should just be bug fixes and minor feature updates.

This version replaces Fluxion's entire messaging/supervisor backend with raw function calls via [Slacktor](https://github.com/peperworx/slacktor). This provides a very similar API to the channel based backend, and comes with a variety of benefits. First and foremost is that Fluxion is now completely, 100%, executor agnostic. No boilerplate code or trait implementations are required anymore. Additionally, Fluxion is now orders of magnitude faster.

Version 1.10 also completely overhauls the idea of foreign messages. All data serialization and message passing is now handled by a "delegate", and actors no longer need to create a "foreign proxy" to handle foreign messages. Additionally, Fluxion is now much more flexible in requiring the Serialize and Deserialize traits.

Here are some bullet points of the core changes:


- Fluxion no longer uses channels.
  - These have been replaced with raw function calls.
  - While there are breaking API changes, this doesn't reduce the usability of the API.
- Fluxion no longer uses supervisor tasks.
  - They are not necessary if channels are not being used.
  - As a consequence of this, error policies are also gone. In the future, error policies may be moved into their own crate, as they were useful in their own right.
- Foreign messages are now entirely handled by "delegates"
  - This includes serialization and deserialization of methods.
- Actor initialization and deinitialization logic has been greatly simplified.
  - Only the `initialize` and `deinitialize` functions are provided.
  - Due to being unable to verify if all references to the actor have been destroyed, only `initialize` provides mutable access to the actor.
- This update should be the last major refactor of Fluxion. While there may be a few breaking changes until 1.0.0, unless there is a massive usability issue, then the Fluxion API will mostly stay where it is.
- Fluxion is several orders of magnitude faster.
  - This is due to the abstraction over raw function calls provided by [Slacktor](https://github.com/peperworx/slacktor).
- Fluxion is much easier to use. See the `simple` example for basic and heavily commented usage.
- Fluxion still supports `no_std` environments, and still requires `alloc` and `core`. 