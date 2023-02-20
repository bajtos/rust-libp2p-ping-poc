// Copyright 2020 Parity Technologies (UK) Ltd.
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

//! The definition of a request/response protocol via inbound
//! and outbound substream upgrades. The inbound upgrade
//! receives a request and sends a response, whereas the
//! outbound upgrade send a request and receives a response.

use crate::zinnia_request_response::codec::RequestResponseCodec;
use crate::zinnia_request_response::RequestId;

use libp2p::core::upgrade::{InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use libp2p::futures::{future::BoxFuture, prelude::*};
use libp2p::swarm::NegotiatedSubstream;
use smallvec::SmallVec;

use std::{fmt, io};

/// Response substream upgrade protocol.
///
/// Receives a request and sends a response.
#[derive(Debug)]
pub struct ResponseProtocol<TCodec>
where
    TCodec: RequestResponseCodec,
{
    #[allow(dead_code)]
    pub(crate) codec: TCodec,
    #[allow(dead_code)]
    pub(crate) request_id: RequestId,
}

impl<TCodec> UpgradeInfo for ResponseProtocol<TCodec>
where
    TCodec: RequestResponseCodec,
{
    type Info = TCodec::Protocol;
    // We don't accept any inbound requests
    type InfoIter = [Self::Info; 0];

    fn protocol_info(&self) -> Self::InfoIter {
        Default::default()
    }
}

impl<TCodec> InboundUpgrade<NegotiatedSubstream> for ResponseProtocol<TCodec>
where
    TCodec: RequestResponseCodec + Send + 'static,
{
    type Output = bool;
    type Error = io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, _io: NegotiatedSubstream, _protocol: Self::Info) -> Self::Future {
        unreachable!("Zinnia does not accept inbound requests")
    }
}

/// Request substream upgrade protocol.
///
/// Sends a request and receives a response.
pub struct RequestProtocol<TCodec>
where
    TCodec: RequestResponseCodec,
{
    pub(crate) codec: TCodec,
    pub(crate) protocols: SmallVec<[TCodec::Protocol; 2]>,
    pub(crate) request_id: RequestId,
    pub(crate) request: TCodec::Request,
}

impl<TCodec> fmt::Debug for RequestProtocol<TCodec>
where
    TCodec: RequestResponseCodec,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestProtocol")
            .field("request_id", &self.request_id)
            .finish()
    }
}

impl<TCodec> UpgradeInfo for RequestProtocol<TCodec>
where
    TCodec: RequestResponseCodec,
{
    type Info = TCodec::Protocol;
    type InfoIter = smallvec::IntoIter<[Self::Info; 2]>;

    fn protocol_info(&self) -> Self::InfoIter {
        self.protocols.clone().into_iter()
    }
}

impl<TCodec> OutboundUpgrade<NegotiatedSubstream> for RequestProtocol<TCodec>
where
    TCodec: RequestResponseCodec + Send + 'static,
{
    type Output = TCodec::Response;
    type Error = io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(
        mut self,
        mut io: NegotiatedSubstream,
        protocol: Self::Info,
    ) -> Self::Future {
        async move {
            let write = self.codec.write_request(&protocol, &mut io, self.request);
            write.await?;
            io.close().await?;
            let read = self.codec.read_response(&protocol, &mut io);
            let response = read.await?;
            Ok(response)
        }
        .boxed()
    }
}
