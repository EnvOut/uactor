# uActor
## Overview
The fastest and most modular actor system that doesn’t force you to pay for what you don’t need

## Examples
Examples can be found [here](src/uactor/examples).

### Features
1. Simplified creation of a tokio actor topic oriented actor
2. Minimum boilerplate code
3. Support different tokio channels including `watch`, `broadcast`, `oneshot`, `mpsc`.
4. Each actor is able to listen up to 30 channels.
5. Added support of actors with single real channel and routing messages to the defined handler [example](src/uactor/examples/single_channel_actor.rs)
6. [ ] Include `tick` (scheduler) inside ActorSelect

### Other projects:
1. Actix
2. Ractor
3. Tokactor
4. tiny-tokio-actor

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in uActor by you, shall be licensed as MIT, without any additional
terms or conditions.