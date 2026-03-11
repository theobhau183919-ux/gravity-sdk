// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{block::Block, vote_data::VoteData};
use gaptos::{
    aptos_crypto::HashValue,
    aptos_crypto_derive::{BCSCryptoHash, CryptoHasher},
    aptos_types::{epoch_state::EpochState, proof::accumulator::InMemoryTransactionAccumulator},
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// This structure contains all the information needed by safety rules to
/// evaluate a proposal / block for correctness / safety and to produce a Vote.
#[derive(Clone, Debug, CryptoHasher, Deserialize, BCSCryptoHash, Serialize)]
pub struct VoteProposal {
    /// The block / proposal to evaluate
    #[serde(bound(deserialize = "Block: Deserialize<'de>"))]
    block: Block,
    /// An optional field containing the next epoch info.
    next_epoch_state: Option<EpochState>,
    /// Executed state id to embed into vote data.
    executed_state_id: HashValue,
    /// Executed state version to embed into vote data.
    executed_state_version: u64,
    /// Represents whether the executed state id is dummy or not.
    decoupled_execution: bool,
}

impl VoteProposal {
    pub fn new(
        block: Block,
        next_epoch_state: Option<EpochState>,
        executed_state_id: HashValue,
        executed_state_version: u64,
        decoupled_execution: bool,
    ) -> Self {
        Self {
            block,
            next_epoch_state,
            executed_state_id,
            executed_state_version,
            decoupled_execution,
        }
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn next_epoch_state(&self) -> Option<&EpochState> {
        self.next_epoch_state.as_ref()
    }

    /// This function returns the vote data with a dummy executed_state_id and version
    fn vote_data_ordering_only(&self) -> VoteData {
        VoteData::new(
            self.block().gen_block_info(
                self.executed_state_id,
                self.executed_state_version,
                self.next_epoch_state().cloned(),
            ),
            self.block().quorum_cert().certified_block().clone(),
        )
    }

    /// This function returns the vote data with a extension proof.
    /// Attention: this function itself does not verify the proof.
    fn vote_data_with_extension_proof(
        &self,
        new_tree: &InMemoryTransactionAccumulator,
    ) -> VoteData {
        VoteData::new(
            self.block().gen_block_info(
                new_tree.root_hash(),
                new_tree.version(),
                self.next_epoch_state().cloned(),
            ),
            self.block().quorum_cert().certified_block().clone(),
        )
    }

    /// Generate vote data depends on the config.
    pub fn gen_vote_data(&self) -> anyhow::Result<VoteData> {
        if self.decoupled_execution {
            Ok(self.vote_data_ordering_only())
        } else {
            anyhow::bail!("missing accumulator extension proof for coupled execution")
        }
    }
}

impl Display for VoteProposal {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "VoteProposal[block: {}]", self.block,)
    }
}
