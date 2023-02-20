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

use crate::zinnia_request_response::RequestId;

pub use libp2p::core::upgrade::ProtocolName;

use libp2p::core::upgrade::{OutboundUpgrade, UpgradeInfo};
use libp2p::futures::{future::BoxFuture, prelude::*};
use libp2p::swarm::NegotiatedSubstream;
use smallvec::SmallVec;

use std::{fmt, io};

// FIXME: Can we use `[u8]` instead? How to avoid closing when sending the data between threads?
pub type RequestPayload = Vec<u8>;
pub type ResponsePayload = Vec<u8>;

/// Request substream upgrade protocol.
///
/// Sends a request and receives a response.
pub struct RequestProtocol<TProtocolInfo>
where
    TProtocolInfo: ProtocolName,
{
    pub(crate) protocols: SmallVec<[TProtocolInfo; 2]>,
    pub(crate) request_id: RequestId,
    pub(crate) request: RequestPayload,
}

impl<TProtocolInfo> fmt::Debug for RequestProtocol<TProtocolInfo>
where
    TProtocolInfo: ProtocolName + Send + Clone + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestProtocol")
            .field("request_id", &self.request_id)
            .finish()
    }
}

impl<TProtocolInfo> UpgradeInfo for RequestProtocol<TProtocolInfo>
where
    TProtocolInfo: ProtocolName + Send + Clone + 'static,
{
    type Info = TProtocolInfo;
    type InfoIter = smallvec::IntoIter<[Self::Info; 2]>;

    fn protocol_info(&self) -> Self::InfoIter {
        self.protocols.clone().into_iter()
    }
}

impl<TProtocolInfo> OutboundUpgrade<NegotiatedSubstream> for RequestProtocol<TProtocolInfo>
where
    TProtocolInfo: ProtocolName + Send + Clone + 'static,
{
    type Output = ResponsePayload;
    type Error = io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(
        mut self,
        mut io: NegotiatedSubstream,
        protocol: Self::Info,
    ) -> Self::Future {
        async move {
            // 1. Write the request payload
            io.write_all(&self.request).await?;
            io.flush().await?;

            // 2. Signal the end of request substream
            io.close().await?;

            // 3. Read back the response
            let mut response: ResponsePayload = Default::default();
            io.read_to_end(&mut response).await?;
            Ok(response)
        }
        .boxed()
    }
}
