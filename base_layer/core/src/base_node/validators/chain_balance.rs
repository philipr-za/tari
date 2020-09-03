//  Copyright 2020, The Tari Project
//
//  Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//  following conditions are met:
//
//  1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//  disclaimer.
//
//  2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//  following disclaimer in the documentation and/or other materials provided with the distribution.
//
//  3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//  products derived from this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//  INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//  SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//  WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//  USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use super::header_iter::HeaderIter;
use crate::{
    chain_storage::{BlockchainBackend, BlockchainDatabase},
    consensus::ConsensusManager,
    transactions::{
        tari_amount::MicroTari,
        types::{BlindingFactor, Commitment, CryptoFactories, PrivateKey},
    },
    validation::{StatelessValidation, ValidationError},
};
use log::*;
use tari_crypto::commitment::HomomorphicCommitmentFactory;

const LOG_TARGET: &str = "c::bn::state_machine_service::states::horizon_state_sync::chain_balance";

/// Validate that the chain balances at a given height.
pub struct ChainBalanceValidator<B> {
    rules: ConsensusManager,
    db: BlockchainDatabase<B>,
    factories: CryptoFactories,
}

impl<B: BlockchainBackend> ChainBalanceValidator<B> {
    pub fn new(db: BlockchainDatabase<B>, rules: ConsensusManager, factories: CryptoFactories) -> Self {
        Self { db, rules, factories }
    }
}

impl<B: BlockchainBackend> StatelessValidation<u64> for ChainBalanceValidator<B> {
    fn validate(&self, horizon_height: &u64) -> Result<(), ValidationError> {
        let total_offset = self.fetch_total_offset_commitment(*horizon_height)?;
        let emission_h = self.get_emission_commitment_at(*horizon_height);
        let kernel_excess = self.db.fetch_kernel_commitment_sum()?;
        let genesis_input_commit = self.get_aggregate_genesis_commitment();
        let output = self.db.fetch_utxo_commitment_sum()?;

        // Validate: ∑UTXO_i ?= Emission + ∑GENESIS_COMMIT_i + ∑KERNEL_EXCESS_i + ∑OFFSET_i
        let agg_excess = &kernel_excess + &genesis_input_commit;
        let input = &(&emission_h + &agg_excess) + &total_offset;

        if output != input {
            return Err(ValidationError::ChainBalanceValidationFailed(*horizon_height));
        }

        Ok(())
    }
}

impl<B: BlockchainBackend> ChainBalanceValidator<B> {
    fn fetch_total_offset_commitment(&self, height: u64) -> Result<Commitment, ValidationError> {
        let header_iter = HeaderIter::new(&self.db, height, 50);
        let mut total_offset = BlindingFactor::default();
        let mut count = 0u64;
        for header in header_iter {
            let header = header?;
            count += 1;
            total_offset = total_offset + header.total_kernel_offset;
        }
        trace!(target: LOG_TARGET, "Fetched {} headers", count);
        let offset_commitment = self.factories.commitment.commit(&total_offset, &0u64.into());
        Ok(offset_commitment)
    }

    fn get_emission_commitment_at(&self, height: u64) -> Commitment {
        let total_supply = self.rules.emission_schedule().supply_at_block(height) -
            self.rules.consensus_constants().get_genesis_coinbase_value_offset();
        trace!(
            target: LOG_TARGET,
            "Expected emission at height {} is {}",
            height,
            total_supply
        );
        self.commit_value(total_supply)
    }

    fn get_aggregate_genesis_commitment(&self) -> Commitment {
        // Get the sum of unspent genesis block UTXOs (excl coinbase)
        self.rules
            .get_genesis_block()
            .body
            .outputs()
            .iter()
            .filter(|u| !u.is_coinbase())
            .map(|u| &u.commitment)
            .sum()
    }

    #[inline]
    fn commit_value(&self, v: MicroTari) -> Commitment {
        self.factories.commitment.commit_value(&PrivateKey::default(), v.into())
    }
}
