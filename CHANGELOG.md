# Changelog

## 0.10.0

This version is the final complete overhaul of Fluxion. It completely replaces Fluxion's entire messaing backend with raw function calls. While this still provides almost exactly the same API as when using channels, it significantly increases performance (by several orders of magnitude) and reduces the memory footprint of programs using Fluxion. Because Fluxion no longer spawns supervisor tasks, it is 100% executor agnostic, and doesn't require any boilerplate.

This version also offloads a ton of the work previously done for foreign messages onto the foreign message handler. This enables significantly greater flexibility, and also means that actors no longer need to create a "foreign proxy" to send foreign messages. Additionally, Fluxion is now much more flexible in requiring the Serialize and Deserialize traits.

Here is a TLDR of the core changes that have been made:
- Foreign messages are now handles by "Delegates" instead of being sent over a channel.
- Fluxion no longer uses channels.
  - These have been replaced with raw function calls.
  - While there are breaking API changes, this doesn't reduce the usability of the API.
- Fluxion no longer uses supervisor tasks.
  - They are not necessary if channels are not being used.
  - As a consequence of this, error policies are also gone. These were actually pretty useful, and as such may be released as an external crate.
- Fluxion no longer handles serialization of messages.
  - Foreign messages are now entirely handled by "delegates" which serve to relay foreign messages.
- Actor initialization and deinitialization logic has been greatly simplified.
- All of these very breaking changes should be the last major refactor of Fluxion. While there may be a few breaking changes until 1.0.0, unless there is a massive usability issue, then the Fluxion API will mostly stay where it is.
- Fluxion is several orders of magnitude faster.
- Fluxion is several orders of magnitude easier to use. 