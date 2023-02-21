use std::time::Instant;

use libp2p::core::{Multiaddr, PeerId};
use libp2p::multiaddr::Protocol;

pub mod peer;
use peer::PeerNode;

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

    // 1. Setup the peer and spawn the network task for it to run in the background.
    let mut peer =
        PeerNode::spawn(Default::default()).expect("should be able to create a new peer");

    // 3. Dial a remote peer using a peer_id & remote_addr
    // Zinnia will not register with DHT in the initial version.
    println!("Dialing {peer_id} at {remote_addr}");
    peer.dial(peer_id, remote_addr.clone())
        .await
        .expect("Dial should succeed");
    println!("Connected!");

    // 4. Request the `ping` protocol.
    // Real-world modules will invoke different protocols, e.g BitSwap.
    println!("Sending the first ping request");
    let result = peer.ping(peer_id).await.expect("Ping should succeeed");

    // 5. Report results
    println!("Round-trip time: {}ms", result.as_millis());

    // ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️
    // BUT THAT 👆🏻 IS NOT WHAT WE NEED FOR ZINNIA!
    // ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️
    //
    // We want to drive the behaviour from this main file, as shown below.
    //
    // Check out the following Deno example for the rationale behind this API design:
    // https://github.com/denoland/deno/blob/848e2c0d57febf744ed585702f314dc64bc8b4ae/core/examples/http_bench_json_ops/main.rs

    let request = ping::new_request_payload();

    // 1. Send a request to the given peer
    let started = Instant::now();
    println!("Sending the second request at {:?}", started);
    let response = peer
        .request_protocol(
            peer_id,
            remote_addr.clone(),
            ping::PROTOCOL_NAME,
            request.clone(),
        )
        .await
        .expect("request ping protocol should succeed");
    let duration = started.elapsed();
    println!("Elapsed: {}ms", duration.as_millis());

    // 2. Process the response and report results
    if response != request {
        println!(
            "Ping {} payload mismatch. Sent {:?}, received {:?}",
            peer_id, request, response,
        );
    } else {
        println!("Round-trip time: {}ms", duration.as_millis(),)
    }

    // TRY AGAIN

    let request = ping::new_request_payload();

    // 1. Send a request to the given peer
    let started = Instant::now();
    println!("Sending the second request at {:?}", started);
    let response = peer
        .request_protocol(peer_id, remote_addr, ping::PROTOCOL_NAME, request.clone())
        .await
        .expect("request ping protocol should succeed");
    let duration = started.elapsed();
    println!("Elapsed: {}ms", duration.as_millis());

    // 2. Process the response and report results
    if response != request {
        println!(
            "Ping {} payload mismatch. Sent {:?}, received {:?}",
            peer_id, request, response,
        );
    } else {
        println!("Round-trip time: {}ms", duration.as_millis(),)
    }

    // SHUTDOWN
    peer.shutdown()
        .await
        .expect("should be able to cleanly stop the peer")
}

mod ping {
    use rand::{distributions, thread_rng, Rng};

    use crate::peer::RequestPayload;

    pub const PROTOCOL_NAME: &[u8] = b"/ipfs/ping/1.0.0";
    pub const PING_SIZE: usize = 32;
    pub type PingRequestPayload = [u8; PING_SIZE];

    pub fn new_request_payload() -> RequestPayload {
        let payload: PingRequestPayload = thread_rng().sample(distributions::Standard);
        payload.into()
    }
}
