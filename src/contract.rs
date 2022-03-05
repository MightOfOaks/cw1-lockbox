use std::ops::Add;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128, Uint64, OverflowError, Order};
use cw2::set_contract_version;
use cw_utils::Scheduled;
use cw_storage_plus::Bound;
use serde::de::StdError;
use crate::ContractError::LockBoxExpired;

use crate::error::ContractError;
use crate::msg::{LockBoxResponse, ExecuteMsg, InstantiateMsg, QueryMsg, LockBoxListResponse};
use crate::state::{Claim, Config, CONFIG, LOCK_BOX_SEQ, Lockbox, LOCKBOXES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw1-new-lockbox";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = Config {};
    CONFIG.save(deps.storage, &state)?;
    LOCK_BOX_SEQ.save(deps.storage,&Uint64::zero())?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("admin", msg.admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateLockbox {
            owner,
            claims,
            expiration,
        } => execute_create_lockbox(deps, _env, info, owner, claims, expiration),
        ExecuteMsg::Reset {} => unimplemented!(),
    }
}
pub fn execute_create_lockbox(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    claims: Vec<Claim>,
    expiration: Scheduled,
) -> Result<Response, ContractError> {

    let owner = deps.api.addr_validate(&owner)?;
    if expiration.is_triggered(&env.block){
        return Err(ContractError::LockBoxExpired {});
    }

    /*
    let mut total_amount = Uint128::zero();
    for c in claims {
        total_amount += c.amount;
    }*/
    let total_amount: Uint128 = claims.clone().into_iter().map(|c| c.amount).sum();

    let id = LOCK_BOX_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id|{
        Ok(id.add(Uint64::new(1)))
    })?;
    let lockbox = Lockbox{
        id,
        owner,
        claims,
        expiration,
        total_amount,
        reset: false,
    };
    LOCKBOXES.save(deps.storage,id.u64(),&lockbox);

    Ok(Response::new().add_attribute("method","execute_create_lockbox"))

}

/*
pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}
*/


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLockBox { id } => to_binary(&query_lockbox(deps, id)?),
    }
}

fn query_lockbox(deps: Deps, id: Uint64) -> StdResult<LockBoxResponse> {
    let lockbox = LOCKBOXES.load(deps.storage, id.u64())?;
    Ok(LockBoxResponse {
        id: lockbox.id,
        owner: lockbox.owner,
        claims: lockbox.claims,
        expiration: lockbox.expiration,
        total_amount: lockbox.total_amount,
        reset: lockbox.reset })
}
// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn range_lockbox(deps: Deps,
                 start_after: Option<u64>,
                 limit: Option<u32>,
                ) -> StdResult<LockBoxListResponse> {

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let lockboxes: StdResult<Vec<_>> = LOCKBOXES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();
    let res = LockBoxListResponse{
        lockboxes: lockboxes?.into_iter().map(|l|l.1).collect(),
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};
/*
    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }
*/
    #[test]
    fn create_lockbox() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {admin: "ADMIN".to_string()};
        let info = mock_info("creator", &[]);
        let mut env = mock_env();
        env.block.height = 1;
        let _res = instantiate(deps.as_mut(), env, info.clone(), msg).unwrap();

        // beneficiary can release it

        let claims = vec![
            Claim{addr: Addr::unchecked("Claim1").to_string(), amount: Uint128::new(5)},
            Claim{addr: Addr::unchecked("Claim2").to_string(), amount: Uint128::new(10)}];

        let msg = ExecuteMsg::CreateLockbox {
            owner: "OWNER".to_string(),
            claims: claims.clone(),
            expiration: Scheduled::AtHeight(64)
        };

        ///mock_env().block.height = 12_345
        let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::LockBoxExpired {},err);

        let msg = ExecuteMsg::CreateLockbox {
            owner: "OWNER".to_string(),
            claims: claims.clone(),
            expiration: Scheduled::AtHeight(1_000_000)
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let res = query_lockbox(deps.as_ref(), Uint64::new(1)).unwrap();
        println!("{:?}", res);
        assert_eq!(res.id, Uint64::new(1));

        //assert_eq!(ContractError::LockBoxExpired {},err)
    }
/*
    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }*/
}
