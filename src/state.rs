use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use cw_utils::Scheduled;

pub const LOCKBOX_SEQ: Item<Uint64> = Item::new("lockbox_seq");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Lockbox {
    pub id: Uint64,
    pub owner: Addr,
    pub claims: Vec<Claim>,
    pub expiration: Scheduled,
    pub total_amount: Uint128,
    pub reset: bool,
    pub native_denom: Option<String>,
    pub cw20_addr: Option<Addr>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Claim{
    pub addr: Addr,
    pub amount: Uint128,
    pub claimed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RawClaim{
    pub addr: String,
    pub amount: Uint128,
}

pub const CONFIG: Map<u64,Lockbox> = Map::new("lockboxes");
