// Inspired by ping handler from libp2p
// https://github.com/libp2p/rust-libp2p/tree/239a62c7640681290c4b2737c6067cf49aba828b/protocols/ping

// Copyright 2019 Parity Technologies (UK) Ltd.
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

use crate::zinnia_request_response::{
    ProtocolName, RequestId, RequestPayload, RequestResponse, RequestResponseConfig,
    RequestResponseEvent, RequestResponseMessage,
};
use rand::{distributions, thread_rng, Rng};

// Custom ping protocol implementation

pub const PROTOCOL_NAME: &[u8] = libp2p::ping::PROTOCOL_NAME;
pub const PING_SIZE: usize = 32;
pub type PingPayload = [u8; PING_SIZE];

pub type PingBehaviour = RequestResponse<PingProtocol>;

pub fn new(cfg: RequestResponseConfig) -> PingBehaviour {
    RequestResponse::<PingProtocol>::new(cfg)
}

pub type PingEvent = RequestResponseEvent;
pub type PingMessage = RequestResponseMessage;
pub type PingRequestId = RequestId;

#[derive(Debug, Clone)]
pub struct PingProtocol;

impl ProtocolName for PingProtocol {
    fn protocol_name(&self) -> &[u8] {
        PROTOCOL_NAME
    }
}

pub fn new_ping_payload() -> RequestPayload {
    let payload: PingPayload = thread_rng().sample(distributions::Standard);
    payload.into()
}
