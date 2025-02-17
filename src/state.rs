use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint256};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub retry_delay: u64,
    pub job_id: String,
    pub owner: Addr,
}

pub const RELEASES: Map<&[u8], (String, Uint256)> = Map::new("releases");
pub const WITHDRAW_TIMESTAMP: Map<(String, String), Timestamp> = Map::new("withdraw_timestamp");
pub const STATE: Item<State> = Item::new("state");
