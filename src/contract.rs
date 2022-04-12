use std::num::FpCategory::Nan;
use std::ops::Add;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128, Uint64, OverflowError, Order, Coin, from_slice, CosmosMsg, BankMsg, WasmMsg};
use cosmwasm_std::CosmosMsg::Bank;
use cw2::set_contract_version;
use cw20::{Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_utils::{NativeBalance, Scheduled};
use cw_storage_plus::Bound;
use serde::de::StdError;
use crate::ContractError::LockBoxExpired;

use crate::error::ContractError;
use crate::msg::{LockBoxResponse, ExecuteMsg, InstantiateMsg, QueryMsg, LockBoxListResponse, ReceiveMsg};
use crate::state::{Claim, Config, CONFIG, LOCK_BOX_SEQ, Lockbox, LOCKBOXES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw1-lockbox";
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
            native_token,
            cw20_addr
        } => execute_create_lockbox(deps, _env, info, owner, claims, expiration, native_token, cw20_addr),
        ExecuteMsg::Reset {id} => execute_reset_lockbox(deps, _env, id),
        ExecuteMsg::Deposit { id } => execute_deposit_native(deps, _env, info, id),
        //This accepts a properly-encoded ReceiveMsg from a CW20 contract
        ExecuteMsg::Receive(Cw20ReceiveMsg) => execute_receive(deps, _env, info,Cw20ReceiveMsg),
        ExecuteMsg::Claim { id } => execute_claim(deps,_env,info,id),
    }
}
pub fn execute_reset_lockbox(
    deps: DepsMut,
    env: Env,
    id: Uint64,
) -> Result<Response, ContractError> {
    //Set lockbox reset to true
    let mut lockbox = LOCKBOXES.load(deps.storage,id.u64())?;
    if lockbox.expiration.is_triggered(&env.block){
        return Err(ContractError::LockBoxExpired {});
    }
    let owner = deps.api.addr_validate(&lockbox.owner.to_string())?;
    lockbox.reset = true;
    LOCKBOXES.save(deps.storage,id.u64(), &lockbox)?;

    //Giving back tokens to owner
    //Get the amount to pay back
    let mut paybackAmount: Uint128 = Uint128::zero();
    let claims_iter = lockbox.claims.iter();
    for claim in claims_iter {
        paybackAmount.add(Uint128::new(u128::from(claim.amount)));
    }
    paybackAmount = paybackAmount - lockbox.total_amount;
    //Pay back the amount to the owner
    let msg: CosmosMsg = match(lockbox.cw20_addr, lockbox.native_denom){
        (Some(_), Some(_)) => Err(ContractError::Unauthorized {}),
        (None, None) => Err(ContractError::Unauthorized {}),
        (Some(cw20_addr), None) => {
            let message = Cw20ExecuteMsg::Transfer { recipient: lockbox.owner.to_string(), amount: paybackAmount };
            Cw20Contract(cw20_addr).call(message).map_err(ContractError::Std)
        },
        (None, Some(native)) => {
            let message = BankMsg::Send { to_address: lockbox.owner.to_string(), amount: vec![Coin{ denom: native, amount: paybackAmount }] };
            Ok(CosmosMsg::Bank(message))
        }
    }?;
    let res = Response::new().add_message(msg);
    Ok(res)
}


pub fn execute_create_lockbox(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    claims: Vec<Claim>,
    expiration: Scheduled,
    native_token: Option<String>,
    cw20_addr: Option<Addr>
) -> Result<Response, ContractError> {

    let owner = deps.api.addr_validate(&owner)?;
    if expiration.is_triggered(&env.block){
        return Err(ContractError::LockBoxExpired {});
    }

    match(native_token.clone(), cw20_addr.clone()){
        (Some(_), Some(_)) => Err(ContractError::DenomNotSupported {}),
        (None, None) => Err(ContractError::DenomNotSupported {}),
        (_,_) => Ok(())
    };
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
        native_denom: native_token,
        cw20_addr,
    };
    LOCKBOXES.save(deps.storage,id.u64(),&lockbox);
    Ok(Response::new().add_attribute("method","execute_create_lockbox"))
}

pub fn execute_deposit_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Uint64,
) -> Result<Response, ContractError> {
    let mut lockbox = LOCKBOXES.load(deps.storage,id.u64())?;
    if lockbox.expiration.is_triggered(&env.block){
        return Err(ContractError::LockBoxExpired {});
    }

    let denom = lockbox.clone().native_denom.ok_or(ContractError::NativeTokensRequired {})?;

    let coin: &Coin = info.funds
        .iter()
        .find(|c| c.denom == denom)
        .ok_or(ContractError::DenomNotSupported {})?;

    lockbox.total_amount -= coin.amount;
    LOCKBOXES.save(deps.storage,id.u64(), &lockbox)?;

    Ok(Response::default()
            .add_attribute("action","execute_deposit")
            .add_attribute("amount", coin.amount.to_string()))
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {

    let msg: ReceiveMsg = from_slice(&wrapper.msg)?;
    let amount = wrapper.amount;
    match msg{
        ReceiveMsg::Deposit {id} => execute_deposit(deps, env, info, id, amount),
    }
}

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Uint64,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut lockbox = LOCKBOXES.load(deps.storage, id.u64())?;
    let cw20_addr = lockbox.clone().cw20_addr.ok_or((ContractError::DenomNotSupported {}))?;
    if info.sender != cw20_addr{
        return Err(ContractError::Unauthorized {});
    }
    lockbox.total_amount.checked_sub(amount).unwrap();
    LOCKBOXES.save(deps.storage,id.u64(), &lockbox)?;

    Ok(Response::default()
        .add_attribute("action","execute_deposit")
        .add_attribute("amount", amount))
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Uint64,
) -> Result<Response, ContractError> {
    let lockbox= LOCKBOXES.load(deps.storage, id.u64())?;
    if lockbox.reset {
        return Err(ContractError::LockBoxReset {})
    }
    if !lockbox.expiration.is_triggered(&env.block){
        return Err(ContractError::LockBoxNotExpired {})
    }
    let claim = lockbox.claims
        .into_iter()
        .find(|c| c.addr == info.sender.to_string())
        .ok_or(ContractError::Unauthorized {})?;

    let msg: CosmosMsg = match(lockbox.cw20_addr, lockbox.native_denom){
        (Some(_), Some(_)) => Err(ContractError::Unauthorized {}),
        (None, None) => Err(ContractError::Unauthorized {}),
        (Some(cw20_addr), None) => {
            /*
            let balance = Cw20Contract(cw20_addr).balance(&deps.querier, env.contract.address)?;
            if balance < claim.amount{
                return Err(ContractError::InsufficientFunds {})
            }
            */
            let message = Cw20ExecuteMsg::Transfer { recipient: claim.addr, amount: claim.amount };
            Cw20Contract(cw20_addr).call(message).map_err(ContractError::Std)
        },
        (None, Some(native)) => {
            let balance = deps.querier.query_balance(env.contract.address, native.clone())?;
            if balance.amount < claim.amount{
                return Err(ContractError::InsufficientFunds {})
            }
            let message = BankMsg::Send { to_address: claim.addr, amount: vec![Coin{ denom: native, amount: claim.amount }] };
            Ok(CosmosMsg::Bank(message))
        }
    }?;

    let res = Response::new().add_message(msg);
   Ok(res)
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
        QueryMsg::ListLockBoxes { start_after, limit } => to_binary(&range_lockbox(deps, start_after, limit)?),
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
        reset: lockbox.reset,
        native_denom: lockbox.native_denom,
        cw20_addr: lockbox.cw20_addr,
    })
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
        lockboxes: lockboxes?.into_iter().map(|l|l.1.into()).collect(),
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
            expiration: Scheduled::AtHeight(64),
            native_token: None,
            cw20_addr: None
        };

        ///mock_env().block.height = 12_345
        let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
        assert_eq!(ContractError::LockBoxExpired {},err);

        let msg = ExecuteMsg::CreateLockbox {
            owner: "OWNER".to_string(),
            claims: claims.clone(),
            expiration: Scheduled::AtHeight(1_000_000),
            native_token:None,
            cw20_addr:None,

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
