# rust-libp2p-ping-poc

A PoC of a libp2p node written in Rust using production-grade protocols

- [src/peer.rs](src/peer.rs) implements a "network client" module that can be used by Zinnia
  (Filecoin Station Runtime).

- [src/main.rs](src/main.rs) show how we intend to call these "network client" APIs from Zinnia

The app is configured to dial the l1-node deployed from the following repo:
https://github.com/bajtos/saturn-interop-libp2p/

Example output:

```
Dialing 12D3KooWRH71QRJe5vrMp6zZXoH4K7z5MDSWwTXXPriG9dK8HQXk at /dns/saturn-link-poc.fly.dev/tcp/3030/p2p/12D3KooWRH71QRJe5vrMp6zZXoH4K7z5MDSWwTXXPriG9dK8HQXk
Connected in 305ms
Error: Cannot handle inbound request from peer 12D3KooWRH71QRJe5vrMp6zZXoH4K7z5MDSWwTXXPriG9dK8HQXk: The local peer supports none of the protocols requested by the remote
Round-trip time: 207ms
Round-trip time: 199ms
```

The error `Cannot handle inbound request` is expected. The app dials my dummy node running in the
cloud. The cloud node immediately sends a request back, for a protocol that my Rust PoC does not
support.

It would be awesome to find a way how to include the name of the request protocol in the error
message.
