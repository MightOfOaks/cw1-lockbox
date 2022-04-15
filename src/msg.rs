use cosmwasm_std::{Addr, Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use cw_utils::{Scheduled};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{Claim, Lockbox, RawClaim};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateLockbox {
        owner: String,
        raw_claims: Vec<RawClaim>,
        expiration: Scheduled,
        native_token: Option<String>,
        cw20_addr: Option<String>
    },
    Reset {id: Uint64},
    Deposit{id: Uint64},
    Receive(Cw20ReceiveMsg),
    Claim{id: Uint64},
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
    ListLockBoxes {start_after: Option<u64>, limit: Option<u32>}
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
    pub native_denom:Option<String>,
    pub cw20_addr: Option<Addr>
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
            native_denom: self.native_denom,
            cw20_addr: self.cw20_addr
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockBoxListResponse {
    pub lockboxes: Vec<LockBoxResponse>,

}
