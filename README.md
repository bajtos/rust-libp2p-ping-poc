# rust-libp2p-ping-poc

A PoC of a libp2p node written in Rust using production-grade protocols

- [src/peer.rs](src/peer.rs) implements a "network client" module that can be used by Zinnia
  (Filecoin Station Runtime).

- [src/main.rs](src/main.rs) show how we intend to call these "network client" APIs from Zinnia

- [src/ping.rs](src/ping.rs) is a temporary helper to allow us to dial & invoke the built-in `ping`
  protocol
