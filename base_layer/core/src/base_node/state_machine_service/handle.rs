// Copyright 2020. The Tari Project
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

use crate::base_node::state_machine_service::states::{StateEvent, StatusInfo};
use core::sync::atomic::{AtomicBool, Ordering};
use futures::{stream::Fuse, StreamExt};
use std::sync::Arc;
use tari_broadcast_channel::Subscriber;
use tari_shutdown::ShutdownSignal;

#[derive(Clone)]
pub struct StateMachineHandle {
    state_change_event_subscriber: Subscriber<StateEvent>,
    status_event_subscriber: Subscriber<StatusInfo>,
    shutdown_signal: ShutdownSignal,
    initial_blockchain_sync_success: Arc<AtomicBool>,
}

impl StateMachineHandle {
    pub fn new(
        state_change_event_subscriber: Subscriber<StateEvent>,
        status_event_subscriber: Subscriber<StatusInfo>,
        shutdown_signal: ShutdownSignal,
    ) -> Self
    {
        Self {
            state_change_event_subscriber,
            status_event_subscriber,
            shutdown_signal,
            initial_blockchain_sync_success: Arc::new(AtomicBool::new(false)),
        }
    }

    /// This clones the receiver end of the channel and gives out a copy to the caller
    /// This allows multiple subscribers to this channel by only keeping one channel and cloning the receiver for every
    /// caller.
    pub fn get_state_change_event_stream(&self) -> Subscriber<StateEvent> {
        self.state_change_event_subscriber.clone()
    }

    pub fn get_state_change_event_stream_fused(&self) -> Fuse<Subscriber<StateEvent>> {
        self.get_state_change_event_stream().fuse()
    }

    /// This clones the receiver end of the channel and gives out a copy to the caller
    /// This allows multiple subscribers to this channel by only keeping one channel and cloning the receiver for every
    /// caller.
    pub fn get_status_event_stream(&self) -> Subscriber<StatusInfo> {
        self.status_event_subscriber.clone()
    }

    pub fn get_status_event_stream_fused(&self) -> Fuse<Subscriber<StatusInfo>> {
        self.get_status_event_stream().fuse()
    }

    pub fn shutdown_signal(&self) -> ShutdownSignal {
        self.shutdown_signal.clone()
    }

    /// This lets any interested party update the the initial blockchain sync status
    pub fn set_initial_blockchain_sync_success_status(&self, status: bool) {
        self.initial_blockchain_sync_success.store(status, Ordering::Relaxed);
    }

    /// This lets any interested party know if the initial blockchain sync has been successful or not
    pub fn get_initial_blockchain_sync_success_status(&self) -> bool {
        self.initial_blockchain_sync_success.load(Ordering::SeqCst)
    }
}
