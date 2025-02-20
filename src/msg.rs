use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, CustomMsg, Uint256};

#[allow(unused_imports)]
use crate::state::State;

#[cw_serde]
pub struct InstantiateMsg {
    pub retry_delay: u64,
    pub job_id: String,
}

#[cw_serde]
pub struct SwapInfo {
    pub route: Vec<String>,
    pub swap_params: Vec<Vec<Uint256>>,
    pub amount: Uint256,
    pub expected: Uint256,
    pub pools: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    ChangeAsset {
        new_c_asset: String,
        swap_info: SwapInfo,
    },
    SetBobby {
        bobby: String,
    },
    ReleaseBobby {
        recipient: String,
        amount: Uint256,
        nonce: Uint256,
    },
    SetPaloma {},
    UpdateCompass {
        new_compass: String,
    },
    UpdateRefundWallet {
        new_refund_wallet: String,
    },
    UpdateEntranceFee {
        new_entrance_fee: Uint256,
    },
    UpdateServiceFeeCollector {
        new_service_fee_collector: String,
    },
    UpdateServiceFee {
        new_service_fee: Uint256,
    },
    UpdateJobId {
        new_job_id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(State)]
    GetState {},
}

#[cw_serde]
pub struct ExecuteJob {
    pub job_id: String,
    pub payload: Binary,
}

#[cw_serde]
pub enum PalomaMsg {
    /// Message struct for cross-chain calls.
    SchedulerMsg { execute_job: ExecuteJob },
}

impl CustomMsg for PalomaMsg {}
