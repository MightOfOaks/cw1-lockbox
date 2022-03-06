use cosmwasm_std::{Addr, Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use cw_utils::{Expiration, Scheduled};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{Claim, Lockbox};

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
        native_token: Option<String>,
        cw20_addr: Option<String>
    },
    Reset {},
    Deposit{id: Uint64},
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Deposit{id: Uint64},
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
    pub reset: bool,
    pub native_denom:Option<String>
}
impl Into<LockBoxResponse> for Lockbox{
    fn into(self) -> LockBoxResponse {
        LockBoxResponse{
            id: self.id,
            owner: self.owner,
            claims: self.claims,
            expiration: self.expiration,
            total_amount: self.total_amount,
            reset: self.reset,
            native_denom: self.native_denom
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockBoxListResponse {
    pub lockboxes: Vec<LockBoxResponse>,

}
