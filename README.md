# Fluxion

Distributed actor framework written in rust.

## About

Fluxion is an actor framework designed with distributed systems in mind, namely sending messages not just between actors, but also between systems.
Fluxion implements three different methods of communication:
- Messages are unique to a specific actor and are used when communicating with that actor. They can send and recieve responses, and are sent to a single instance of an actor.
- Federated Messages are generic: every actor implements the same data type as a Federated Message. They can send and recieve responses, and are sent to a single instance of an actor.
- Notifications are generic to every actor. They can not recieve response, and are sent to every instance of every actor.

Fluxion allows an external task to subscribe to an mpsc channel over which messages bound to Foreign actors are sent. This external task can then relay them as it sees fit. See the example `foreign` for more details.