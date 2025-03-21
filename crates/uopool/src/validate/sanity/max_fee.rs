use crate::{
    mempool::Mempool,
    uopool::{VecCh, VecUo},
    validate::{SanityCheck, SanityHelper},
    Reputation,
};
use ethers::{
    providers::Middleware,
    types::{BlockNumber, U256},
};
use silius_primitives::{reputation::ReputationEntry, sanity::SanityCheckError, UserOperation};

pub struct MaxFee {
    pub min_priority_fee_per_gas: U256,
}

#[async_trait::async_trait]
impl<M: Middleware, P, R, E> SanityCheck<M, P, R, E> for MaxFee
where
    P: Mempool<UserOperations = VecUo, CodeHashes = VecCh, Error = E> + Send + Sync,
    R: Reputation<ReputationEntries = Vec<ReputationEntry>, Error = E> + Send + Sync,
{
    /// The [check_user_operation] method implementation that checks the max fee
    ///
    /// # Arguments
    /// `uo` - The user operation to check
    /// `helper` - The helper struct that contains the middleware
    ///
    /// # Returns
    /// None if the check passes, otherwise a [SanityCheckError]
    async fn check_user_operation(
        &self,
        uo: &UserOperation,
        helper: &SanityHelper<M, P, R, E>,
    ) -> Result<(), SanityCheckError> {
        if uo.max_priority_fee_per_gas > uo.max_fee_per_gas {
            return Err(SanityCheckError::HighMaxPriorityFeePerGas {
                max_priority_fee_per_gas: uo.max_priority_fee_per_gas,
                max_fee_per_gas: uo.max_fee_per_gas,
            });
        }

        let block = helper
            .entry_point
            .eth_client()
            .get_block(BlockNumber::Latest)
            .await
            .map_err(|err| SanityCheckError::UnknownError {
                message: err.to_string(),
            })?
            .ok_or(SanityCheckError::UnknownError {
                message: "No block found".to_string(),
            })?;
        let base_fee_per_gas = block
            .base_fee_per_gas
            .ok_or(SanityCheckError::UnknownError {
                message: "No base fee".to_string(),
            })?;

        if base_fee_per_gas > uo.max_fee_per_gas {
            return Err(SanityCheckError::LowMaxFeePerGas {
                max_fee_per_gas: uo.max_fee_per_gas,
                base_fee_per_gas,
            });
        }

        if uo.max_priority_fee_per_gas < self.min_priority_fee_per_gas {
            return Err(SanityCheckError::LowMaxPriorityFeePerGas {
                max_priority_fee_per_gas: uo.max_priority_fee_per_gas,
                min_priority_fee_per_gas: self.min_priority_fee_per_gas,
            });
        }

        Ok(())
    }
}
