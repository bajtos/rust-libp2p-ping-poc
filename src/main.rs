use std::time::Instant;

use libp2p::core::{Multiaddr, PeerId};
use libp2p::multiaddr::Protocol;
use rand::{distributions, thread_rng, Rng};
use tokio::spawn;

mod peer;
mod ping;
mod zinnia_request_response;

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
        .dial(peer_id, remote_addr.clone())
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

    // ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️
    // BUT THAT 👆🏻 IS NOT WHAT WE NEED FOR ZINNIA!
    // ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️ ⚠️
    //
    // We want to drive the behaviour from this main file, as shown below.
    //
    // Check out the following Deno example for the rationale behind this API design:
    // https://github.com/denoland/deno/blob/848e2c0d57febf744ed585702f314dc64bc8b4ae/core/examples/http_bench_json_ops/main.rs

    let request: Vec<u8> = {
        let payload: ping::PingPayload = thread_rng().sample(distributions::Standard);
        payload.into()
    };

    // 1. Send a request to the given peer
    let started = Instant::now();
    let response = network_client
        .request_protocol(peer_id, remote_addr, ping::PROTOCOL_NAME, request.clone())
        .await
        .expect("request ping protocol should succeed");
    let duration = started.elapsed();

    // 2. Process the response and report results
    if response != request {
        println!(
            "Ping {} payload mismatch. Sent {:?}, received {:?}",
            peer_id, request, response,
        );
    } else {
        println!("Round-trip time: {}ms", duration.as_millis(),)
    }
}

// // Inspired by futures_util::io::read_exact
// // In Zinnia, this helper would be implemented differently
// async fn read_exact(
//     client: &mut peer::Client,
//     handle: &mut peer::StreamHandle,
//     outbuf: &mut [u8],
// ) -> Result<(), Box<dyn std::error::Error + Send>> {
//     let mut buf = outbuf;

//     while !buf.is_empty() {
//         let n = client.read(handle, &mut buf).await?;
//         if n == 0 {
//             return Err(Box::new(std::io::Error::from(
//                 std::io::ErrorKind::UnexpectedEof,
//             )));
//         }
//         let (_, rest) = std::mem::take(&mut buf).split_at_mut(n);
//         buf = rest;
//     }

//     Ok(())
// }
