use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_utils::{Expiration, Scheduled};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::Claim;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateLockbox {
        owner: String,
        claims: Vec<Claim>,
        expiration: Scheduled,
    },
    Reset {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetLockBox {id: Uint64},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockBoxResponse {
    pub id: Uint64,
    pub owner: Addr,
    pub claims: Vec<Claim>,
    pub expiration: Scheduled,
    pub total_amount: Uint128,
    pub reset: bool
}
