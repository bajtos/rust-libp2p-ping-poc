use libp2p::core::{Multiaddr, PeerId};
use libp2p::multiaddr::Protocol;
use tokio::spawn;

mod peer;
mod ping;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let remote_addr: Multiaddr =
        "/dns/saturn-link-poc.fly.dev/tcp/3030/p2p/12D3KooWRH71QRJe5vrMp6zZXoH4K7z5MDSWwTXXPriG9dK8HQXk"
        .parse()
        .expect("should be able to parse our hard-coded multiaddr");

    let peer_id = match remote_addr.iter().last() {
        Some(Protocol::P2p(hash)) => PeerId::from_multihash(hash).expect("Valid hash."),
        _ => {
            panic!("The peer multiaddr should contain peer ID.");
        }
    };

    // DEMO USAGE OF THE `peer` MODULE

    // 1. Setup the peer
    let (mut network_client, network_event_loop) = peer::new()
        .await
        .expect("should be able to create a new peer");

    // 2. Spawn the network task for it to run in the background.
    spawn(network_event_loop.run());

    // 3. Dial a remote peer using a peer_id & remote_addr
    // Zinnia will not register with DHT in the initial version.
    network_client
        .dial(peer_id, remote_addr)
        .await
        .expect("Dial should succeed");

    // 4. Request the `ping` protocol.
    // Real-world modules will invoke different protocols, e.g BitSwap.
    let result = network_client
        .ping(peer_id)
        .await
        .expect("Ping should succeeed");

    // 5. Report results
    println!("Round-trip time: {}ms", result.as_millis());
}
