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

use async_trait::async_trait;
use libp2p::futures::{io, AsyncReadExt, AsyncWriteExt};
use libp2p::futures::{AsyncRead, AsyncWrite};
use libp2p::request_response::{
    ProtocolName, ProtocolSupport, RequestId, RequestResponse, RequestResponseCodec,
    RequestResponseConfig, RequestResponseEvent, RequestResponseMessage,
};
use rand::{distributions, thread_rng, Rng};

// Custom ping protocol implementation

pub const PROTOCOL_NAME: &[u8] = libp2p::ping::PROTOCOL_NAME;
pub const PING_SIZE: usize = 32;
pub type PingPayload = [u8; PING_SIZE];

pub type PingBehaviour = RequestResponse<PingCodec>;

pub fn new(cfg: RequestResponseConfig) -> PingBehaviour {
    RequestResponse::<PingCodec>::new(
        PingCodec(),
        std::iter::once((PingProtocol, ProtocolSupport::Full)),
        cfg,
    )
}

pub type PingEvent = RequestResponseEvent<PingRequest, PingResponse>;
pub type PingMessage = RequestResponseMessage<PingRequest, PingResponse>;
pub type PingRequestId = RequestId;

#[derive(Debug, Clone)]
pub struct PingProtocol;

#[derive(Clone)]
pub struct PingCodec();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingRequest(PingPayload);

impl PingRequest {
    pub fn new() -> Self {
        let payload: PingPayload = thread_rng().sample(distributions::Standard);
        Self(payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingResponse(PingPayload);

impl PartialEq<PingRequest> for PingResponse {
    fn eq(&self, other: &PingRequest) -> bool {
        self.0 == other.0
    }
}

impl ProtocolName for PingProtocol {
    fn protocol_name(&self) -> &[u8] {
        PROTOCOL_NAME
    }
}

#[async_trait]
impl RequestResponseCodec for PingCodec {
    type Protocol = PingProtocol;
    type Request = PingRequest;
    type Response = PingResponse;

    async fn read_request<T>(&mut self, _: &PingProtocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut payload: PingPayload = Default::default();
        io.read_exact(&mut payload).await?;
        Ok(PingRequest(payload))
    }

    async fn read_response<T>(&mut self, _: &PingProtocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut payload: PingPayload = Default::default();
        io.read_exact(&mut payload).await?;
        Ok(PingResponse(payload))
    }

    async fn write_request<T>(
        &mut self,
        _: &PingProtocol,
        io: &mut T,
        PingRequest(data): PingRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &PingProtocol,
        io: &mut T,
        PingResponse(data): PingResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&data).await?;
        io.flush().await?;
        Ok(())
    }
}
