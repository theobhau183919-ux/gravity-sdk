pub mod block_executor {
    use anyhow::Result;

    use aptos_executor_types::{
        BlockExecutorTrait, ExecutorError, ExecutorResult, StateComputeResult,
    };
    use gaptos::{
        aptos_crypto::HashValue,
        aptos_executor::block_executor::BlockExecutor as InnerBlockExecutor,
        aptos_executor_types::BlockExecutorTrait as InnerBlockExecutorTrait,
        aptos_storage_interface::DbReaderWriter,
        aptos_types::{
            block_executor::{
                config::BlockExecutorConfigFromOnchain, partitioner::ExecutableBlock,
            },
            ledger_info::LedgerInfoWithSignatures,
        },
        aptos_vm::AptosVM,
    };

    pub struct BlockExecutor {
        pub db: DbReaderWriter,
        inner: InnerBlockExecutor<AptosVM>,
    }

    impl BlockExecutor {
        pub fn new(db: DbReaderWriter) -> Self {
            let inner = InnerBlockExecutor::new(db.clone());
            Self { db, inner }
        }
    }

    impl BlockExecutorTrait for BlockExecutor {
        fn committed_block_id(&self) -> HashValue {
            InnerBlockExecutorTrait::committed_block_id(&self.inner)
        }

        fn reset(&self) -> Result<()> {
            InnerBlockExecutorTrait::reset(&self.inner).map_err(|e| anyhow::anyhow!(e.to_string()))
        }

        fn execute_and_state_checkpoint(
            &self,
            block: ExecutableBlock,
            parent_block_id: HashValue,
            onchain_config: BlockExecutorConfigFromOnchain,
        ) -> ExecutorResult<()> {
            InnerBlockExecutorTrait::execute_and_update_state(
                &self.inner,
                block,
                parent_block_id,
                onchain_config,
            )
            .map_err(|e| ExecutorError::internal_err(e.to_string()))
        }

        fn ledger_update(
            &self,
            block_id: HashValue,
            parent_block_id: HashValue,
        ) -> ExecutorResult<StateComputeResult> {
            let res =
                InnerBlockExecutorTrait::ledger_update(&self.inner, block_id, parent_block_id)
                    .map_err(|e| ExecutorError::internal_err(e.to_string()))?;
            Ok(StateComputeResult::with_root_hash(res.root_hash()))
        }

        fn commit_blocks(
            &self,
            block_ids: Vec<HashValue>,
            ledger_info_with_sigs: LedgerInfoWithSignatures,
        ) -> ExecutorResult<()> {
            for block_id in block_ids {
                self.pre_commit_block(block_id)?;
            }
            self.commit_ledger(ledger_info_with_sigs, vec![])
        }

        fn finish(&self) {
            InnerBlockExecutorTrait::finish(&self.inner)
        }

        fn pre_commit_block(&self, block_id: HashValue) -> ExecutorResult<()> {
            InnerBlockExecutorTrait::pre_commit_block(&self.inner, block_id)
                .map_err(|e| ExecutorError::internal_err(e.to_string()))
        }

        fn commit_ledger(
            &self,
            block_ids: Vec<HashValue>,
            ledger_info_with_sigs: LedgerInfoWithSignatures,
            randomness_data: Vec<(u64, Vec<u8>)>,
        ) -> ExecutorResult<()> {
            if !randomness_data.is_empty() {
                return Err(ExecutorError::internal_err(
                    "randomness_data is not supported by aptos-executor backend",
                ));
            }

            if let Some(last_block_id) = block_ids.last() {
                let commit_block_id = ledger_info_with_sigs.ledger_info().commit_info().id();
                if commit_block_id != *last_block_id {
                    return Err(ExecutorError::internal_err(format!(
                        "ledger_info commit id {} does not match last block id {}",
                        commit_block_id, last_block_id
                    )));
                }
            }

            for block_id in block_ids {
                InnerBlockExecutorTrait::pre_commit_block(&self.inner, block_id)
                    .map_err(|e| ExecutorError::internal_err(e.to_string()))?;
            }

            InnerBlockExecutorTrait::commit_ledger(&self.inner, ledger_info_with_sigs)
                .map_err(|e| ExecutorError::internal_err(e.to_string()))
        }
    }
}
