// Copyright 2019, The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::{base_node::chain_metadata_service::error::ChainMetadataSyncError, chain_storage::ChainMetadata};
use futures::{stream::Fuse, StreamExt};
use std::{
    fmt::{Display, Error, Formatter},
    time::Instant,
};
use tari_broadcast_channel::Subscriber;
use tari_comms::peer_manager::NodeId;
use tari_service_framework::reply_channel::SenderService;
use tower_service::Service;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerChainMetadata {
    pub node_id: NodeId,
    pub chain_metadata: ChainMetadata,
}

impl PeerChainMetadata {
    pub fn new(node_id: NodeId, chain_metadata: ChainMetadata) -> Self {
        Self {
            node_id,
            chain_metadata,
        }
    }
}

impl Display for PeerChainMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f, "Node ID: {}", self.node_id)?;
        writeln!(f, "Chain metadata: {}", self.chain_metadata)
    }
}

#[derive(Debug)]
pub enum ChainMetadataEvent {
    PeerChainMetadataReceived(Vec<PeerChainMetadata>),
}

#[derive(Debug)]
pub enum ChainMetadataServiceRequest {
    FullRoundOfMetadataReceived,
}

pub enum ChainMetadataServiceResponse {
    FullRoundOfMetadataReceived(bool),
}

#[derive(Clone)]
pub struct ChainMetadataHandle {
    handle: SenderService<ChainMetadataServiceRequest, Result<ChainMetadataServiceResponse, ChainMetadataSyncError>>,
    event_stream: Subscriber<ChainMetadataEvent>,
}

impl ChainMetadataHandle {
    pub fn new(
        handle: SenderService<
            ChainMetadataServiceRequest,
            Result<ChainMetadataServiceResponse, ChainMetadataSyncError>,
        >,
        event_stream: Subscriber<ChainMetadataEvent>,
    ) -> Self
    {
        Self { handle, event_stream }
    }

    pub fn get_event_stream(&self) -> Subscriber<ChainMetadataEvent> {
        self.event_stream.clone()
    }

    pub fn get_event_stream_fused(&self) -> Fuse<Subscriber<ChainMetadataEvent>> {
        self.get_event_stream().fuse()
    }

    pub async fn full_round_of_metadata_received(&mut self) -> Result<bool, ChainMetadataSyncError> {
        match self
            .handle
            .call(ChainMetadataServiceRequest::FullRoundOfMetadataReceived)
            .await??
        {
            ChainMetadataServiceResponse::FullRoundOfMetadataReceived(received) => Ok(received),
        }
    }
}
