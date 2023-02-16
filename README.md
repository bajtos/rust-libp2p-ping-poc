# rust-libp2p-ping-poc

A PoC of a libp2p node written in Rust using production-grade protocols

- [src/peer.rs](src/peer.rs) implements a "network client" module that can be used by Zinnia
  (Filecoin Station Runtime).

- [src/main.rs](src/main.rs) show how we intend to call these "network client" APIs from Zinnia

- [src/ping.rs](src/ping.rs) is a temporary helper to allow us to dial & invoke the built-in `ping`
  protocol

The app is configured to dial the l1-node deployed from the following repo:
https://github.com/bajtos/saturn-interop-libp2p/

Example output:

```
inbound ping error: request id 2, peer: 12D3KooWRH71QRJe5vrMp6zZXoH4K7z5MDSWwTXXPriG9dK8HQXk, error: The local peer supports none of the protocols requested by the remote
Ping 12D3KooWRH71QRJe5vrMp6zZXoH4K7z5MDSWwTXXPriG9dK8HQXk completed in 166ms
Round-trip time: 166ms
```

The `inbound ping error` is expected. The app dials my dummy node running in
the cloud. The cloud  node immediately sends a request back, for a protocol
that my Rust PoC does not support.

It would be awesome to find a way how to include the name of the request
protocol in the error message.
