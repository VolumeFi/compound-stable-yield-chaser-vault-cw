use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint256};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub retry_delay: u64,
    pub job_id: String,
    pub owner: Addr,
    pub release_nonce: u128,
}

pub const NONCE: Map<&[u8], (String, Uint256)> = Map::new("nonce");
pub const WITHDRAW_TIMESTAMP: Map<(String, String), Timestamp> = Map::new("withdraw_timestamp");
pub const STATE: Item<State> = Item::new("state");
