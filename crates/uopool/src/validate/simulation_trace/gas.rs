use crate::validate::{SimulationTraceCheck, SimulationTraceHelper};
use ethers::providers::Middleware;
use silius_primitives::{
    consts::entities::LEVEL_TO_ENTITY, simulation::SimulationCheckError, UserOperation,
};

pub struct Gas;

#[async_trait::async_trait]
impl<M: Middleware> SimulationTraceCheck<M> for Gas {
    async fn check_user_operation(
        &self,
        _uo: &UserOperation,
        helper: &mut SimulationTraceHelper<M>,
    ) -> Result<(), SimulationCheckError> {
        for (i, _) in LEVEL_TO_ENTITY.iter().enumerate() {
            if let Some(l) = helper.js_trace.number_levels.get(i) {
                if l.oog.unwrap_or(false) {
                    return Err(SimulationCheckError::OutOfGas {});
                }
            }
        }
        Ok(())
    }
}
