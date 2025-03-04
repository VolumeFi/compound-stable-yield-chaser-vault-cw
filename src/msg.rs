use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, CustomMsg, DenomUnit, Uint128, Uint256};

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
    CreateBobby {
        compass_job_id: String,
        blueprint: String,
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

    TokenFactoryMsg {
        create_denom: Option<CreateDenomMsg>,
        mint_tokens: Option<MintMsg>,
    },

    SkywayMsg {
        set_erc20_to_denom: Option<SetErc20ToDenom>,
        send_tx: Option<SendTx>,
    },
}

#[cw_serde]
pub struct CreateDenomMsg {
    pub subdenom: String,
    pub metadata: Metadata,
}

#[cw_serde]
pub struct Metadata {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
}

#[cw_serde]
pub struct MintMsg {
    pub denom: String,
    pub amount: Uint128,
    pub mint_to_address: String,
}

#[cw_serde]
pub struct SendTx {
    pub remote_chain_destination_address: String,
    pub amount: String,
    pub chain_reference_id: String,
}

#[cw_serde]
pub struct SetErc20ToDenom {
    pub erc20_address: String,
    pub token_denom: String,
    pub chain_reference_id: String,
}

impl CustomMsg for PalomaMsg {}
