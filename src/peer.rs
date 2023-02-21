// Inspired by File Sharing example from libp2p
// https://github.com/libp2p/rust-libp2p/blob/caed1fe2c717ba1688a4eb0549284cddba8c9ea6/examples/file-sharing.rs

// Copyright 2021 Protocol Labs.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use libp2p::core::muxing::StreamMuxerBox;
use std::collections::{hash_map, HashMap};
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use libp2p::core::{transport, upgrade, Multiaddr, PeerId};
use libp2p::futures::StreamExt;
use libp2p::identity;
use libp2p::multiaddr::Protocol;
use libp2p::noise;
use libp2p::swarm::{ConnectionHandlerUpgrErr, NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::yamux;
use libp2p::Transport;

mod behaviour;
mod handler;

use behaviour::{
    ProtocolInfo, RequestId, RequestResponse, RequestResponseEvent, RequestResponseMessage,
};
pub use behaviour::{RequestPayload, ResponsePayload};

/// A Zinnia peer node wrapping rust-libp2p and providing higher-level APIs
/// for consumption by Deno ops.
pub struct PeerNode {
    command_sender: mpsc::Sender<Command>,
    #[allow(unused)]
    event_loop_task: JoinHandle<()>,
}

impl PeerNode {
    /// Spawns the [`PeerNode`] in a tokio task.
    ///
    /// This will create the underlying network client and spawn a tokio task handling
    /// networking event loop. The returned [`PeerNode`] can be used to control the task.
    pub async fn spawn() -> Result<PeerNode, Box<dyn Error>> {
        // Create a new random public/private key pair
        // Zinnia will always generate a new key pair on (re)start
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = id_keys.public().to_peer_id();

        let tcp_transport = create_transport(&id_keys)?;

        // In the initial version, Zinnia nodes ARE NOT dialable.
        // Each module must connect to a remote server (dial the orchestrator)
        //
        // let tcp_listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
        // tcp_transport.listen_on(tcp_listen_addr.clone())?;

        // Build the Swarm, connecting the lower layer transport logic with the
        // higher layer network behaviour logic.
        let swarm = Swarm::with_tokio_executor(
            tcp_transport,
            ComposedBehaviour {
                zinnia: RequestResponse::new(Default::default()),
            },
            peer_id,
        );

        let (command_sender, command_receiver) = mpsc::channel::<Command>(1);

        let event_loop = EventLoop::new(swarm, command_receiver);
        let event_loop_task = tokio::spawn(event_loop.run());

        Ok(Self {
            command_sender,
            event_loop_task,
        })
    }

    /// Dial the given peer at the given address.
    pub async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Ping the remote peer. You must dial the peer first before calling this method.
    pub async fn ping(&mut self, peer_id: PeerId) -> Result<Duration, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        let request_payload = crate::ping::new_request_payload();
        let started = Instant::now();

        self.command_sender
            .send(Command::Request {
                peer_id,
                protocol: crate::ping::PROTOCOL_NAME.into(),
                payload: request_payload.clone(),
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        let response_payload = receiver.await.expect("Sender not be dropped.")?;

        let duration = started.elapsed();
        if response_payload != request_payload {
            println!(
                "Ping {} payload mismatch. Sent {:?}, received {:?}",
                peer_id, request_payload, response_payload
            );
            let err: Box<dyn std::error::Error + std::marker::Send> = Box::new(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Ping payload mismatch"),
            );
            Err(err)
        } else {
            println!("Ping {} completed in {}ms", peer_id, duration.as_millis());
            Ok(duration)
        }
    }

    // NEW API FOR ZINNIA

    pub async fn request_protocol(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
        protocol: &[u8],
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.dial(peer_id, peer_addr).await?;
        self.command_sender
            .send(Command::Request {
                peer_id,
                protocol: protocol.into(),
                payload,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    // pub async fn dial_protocol(
    //     &self,
    //     peer_id: PeerId,
    //     peer_addr: Multiaddr,
    //     proto_name: &[u8],
    // ) -> Result<StreamHandle, Box<dyn Error + Send>> {
    //     todo!(
    //         "TODO: dial {peer_id} at {peer_addr} and start protocol {}",
    //         String::from_utf8_lossy(proto_name)
    //     );
    // }

    // pub async fn write_all(
    //     &self,
    //     handle: &StreamHandle,
    //     buf: &[u8],
    // ) -> Result<(), Box<dyn Error + Send>> {
    //     todo!("TODO: write {} bytes to {:?}", buf.len(), handle)
    // }

    // pub async fn close_writer(
    //     &self,
    //     handle: &mut StreamHandle,
    // ) -> Result<(), Box<dyn Error + Send>> {
    //     todo!("TODO: close writer {:?}", handle)
    // }

    // pub async fn read(
    //     &self,
    //     handle: &mut StreamHandle,
    //     buf: &[u8],
    // ) -> Result<usize, Box<dyn Error + Send>> {
    //     todo!("TODO: read up to {} bytes from {:?}", buf.len(), handle)
    // }
}

pub fn create_transport(
    id_keys: &identity::Keypair,
) -> Result<transport::Boxed<(PeerId, StreamMuxerBox)>, noise::NoiseError> {
    // Setup the transport + multiplex + auth
    // Zinnia will hard-code this configuration initially.
    // We need to pick reasonable defaults that will allow Zinnia nodes to interoperate with
    // as many other libp2p nodes as possible.
    let tcp_transport = libp2p::dns::TokioDnsConfig::system(libp2p::tcp::tokio::Transport::new(
        libp2p::tcp::Config::new(),
    ))?
    .upgrade(upgrade::Version::V1)
    .authenticate(noise::NoiseAuthenticated::xx(&id_keys)?)
    .multiplex(upgrade::SelectUpgrade::new(
        yamux::YamuxConfig::default(),
        libp2p::mplex::MplexConfig::default(),
    ))
    .timeout(std::time::Duration::from_secs(5))
    .boxed();
    return Ok(tcp_transport);
}

// /// A handle representing a substream opened by our network behaviour
// #[derive(Debug)]
// pub struct StreamHandle;

pub struct EventLoop {
    swarm: Swarm<ComposedBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_requests: HashMap<RequestId, PendingRequest>,
}

pub struct PendingRequest {
    sender: oneshot::Sender<Result<ResponsePayload, Box<dyn Error + Send>>>,
}

impl EventLoop {
    fn new(swarm: Swarm<ComposedBehaviour>, command_receiver: mpsc::Receiver<Command>) -> Self {
        Self {
            swarm,
            command_receiver,
            pending_dial: Default::default(),
            pending_requests: Default::default(),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.next() => self.handle_event(event.expect("Swarm stream to be infinite.")).await,
                command = self.command_receiver.recv() => match command {
                    Some(c) => self.handle_command(c).await,
                    // Command channel closed, thus shutting down the network event loop.
                    None=>  return,
                },
            }
        }
    }

    async fn handle_event(
        &mut self,
        event: SwarmEvent<ComposedEvent, ConnectionHandlerUpgrErr<std::io::Error>>,
    ) {
        match event {
            SwarmEvent::Behaviour(ComposedEvent::Zinnia(result)) => {
                match result {
                    RequestResponseEvent::OutboundFailure {
                        request_id,
                        error,
                        peer,
                    } => {
                        println!("Cannot request {}: {}", peer, error);
                        let pending_request = self
                            .pending_requests
                            .remove(&request_id)
                            .expect("Ping request should be still be pending.");
                        pending_request
                            .sender
                            .send(Err(Box::new(error)))
                            .expect("Request should have an active sender to receive the result.");
                    }

                    RequestResponseEvent::Message {
                        peer: _,
                        message:
                            RequestResponseMessage::Response {
                                request_id,
                                response,
                            },
                    } => {
                        let pending_request = self
                            .pending_requests
                            .remove(&request_id)
                            .expect("Request should be still be pending.");

                        pending_request
                            .sender
                            .send(Ok(response))
                            .expect("Request should have an active sender to receive the result.");
                    }

                    // incoming requests - we don't support that!
                    //
                    RequestResponseEvent::InboundFailure { peer, error } => {
                        println!(
                            "Error: Cannot handle inbound request from peer {}: {}",
                            peer, error
                        );
                    }
                }
            }

            SwarmEvent::NewListenAddr { .. } => {}
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing(peer_id) => eprintln!("Dialing {peer_id}"),
            e => panic!("{e:?}"),
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                if self.swarm.is_connected(&peer_id) {
                    let _ = sender.send(Ok(()));
                    return;
                }

                if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    self.swarm
                        .behaviour_mut()
                        .zinnia
                        .add_address(&peer_id, peer_addr.clone());

                    match self
                        .swarm
                        .dial(peer_addr.with(Protocol::P2p(peer_id.into())))
                    {
                        Ok(()) => {
                            e.insert(sender);
                        }
                        Err(e) => {
                            let _ = sender.send(Err(Box::new(e)));
                        }
                    }
                } else {
                    todo!("Already dialing peer.");
                }
            }

            Command::Request {
                peer_id,
                protocol,
                payload,
                sender,
            } => {
                let request_id =
                    self.swarm
                        .behaviour_mut()
                        .zinnia
                        .send_request(&peer_id, &[protocol], payload);
                self.pending_requests
                    .insert(request_id, PendingRequest { sender });
            }
        }
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ComposedEvent")]
struct ComposedBehaviour {
    pub zinnia: RequestResponse,
    // We can add more behaviours later.
    // request_response: request_response::Behaviour<FileExchangeCodec>,
}

#[derive(Debug)]
enum ComposedEvent {
    Zinnia(RequestResponseEvent),
    // We can add more events later.
}

impl From<RequestResponseEvent> for ComposedEvent {
    fn from(event: RequestResponseEvent) -> Self {
        ComposedEvent::Zinnia(event)
    }
}

#[derive(Debug)]
enum Command {
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Request {
        peer_id: PeerId,
        protocol: ProtocolInfo,
        payload: RequestPayload,
        sender: oneshot::Sender<Result<ResponsePayload, Box<dyn Error + Send>>>,
    },
}
