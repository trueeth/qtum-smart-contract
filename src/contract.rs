
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, from_json, to_json_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult, Uint128 
};

use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg ;
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info,
};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

use crate::error::ContractError;
use crate::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, InvestmentResponse, LockType, QueryMsg, UserStakingInfoResponse};
use crate::state::{  LockPrd, LockTax, StakingInfo, Supply, UserStakingInfo, STAKING_INFO, TOTAL_SUPPLY, USER_STAKING};


const FALLBACK_RATIO: Decimal = Decimal::one();

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:xqtum";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // store token info using cw20-base format
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };

    TOKEN_INFO.save(deps.storage, &data)?;

    let staking_info = StakingInfo {
        owner: info.sender,
        stake_denom: msg.stake_denom,
        staking_token_address: deps.api.addr_validate(&msg.staking_token_address)?,
        period: LockPrd {
            long: msg.long_period,
            short: msg.short_period
        },
        tax: LockTax {
            long: Decimal::percent(msg.long_tax),
            short: Decimal::percent(msg.short_tax)
        },
        penalty: Decimal::percent(msg.penalty)
    };

    STAKING_INFO.save(deps.storage, &staking_info)?;

    // set supply to 0
    let supply = Supply::default();
    TOTAL_SUPPLY.save(deps.storage, &supply)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),       

        ExecuteMsg::UnLock { idx, amount } => unlock(deps, env, info,  idx, amount ),
        // these all come from cw20-base to implement the cw20 standard
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(execute_transfer(deps, env, info, recipient, amount)?)
        }
        ExecuteMsg::Burn { amount } => Ok(execute_burn(deps, env, info, amount)?),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(deps, env, info, contract, amount, msg)?),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => Ok(execute_transfer_from(
            deps, env, info, owner, recipient, amount,
        )?),
        ExecuteMsg::BurnFrom { owner, amount } => {
            Ok(execute_burn_from(deps, env, info, owner, amount)?)
        }
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => Ok(execute_send_from(
            deps, env, info, owner, contract, amount, msg,
        )?),
    }
}


pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg
) -> Result<Response, ContractError> {
    let stake_info = STAKING_INFO.load(deps.storage)?;

    match from_json(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Lock { idx, lock_type}) => {
            // only staking token contract can execute this message
            if stake_info.staking_token_address != deps.api.addr_validate(info.sender.as_str())? {
                return Err(ContractError::InvalidStakingToken {});
            }

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            lock(deps, env, cw20_sender, idx,   cw20_msg.amount, lock_type)
        }

        Err(_) => Err(ContractError::InvalidLockType {  }),
    }
}


pub fn lock(
    deps: DepsMut, 
    env: Env, 
    sender: Addr, 
    idx: String,  
    lock_amount: Uint128, 
    lock_type: LockType
) -> Result<Response, ContractError> {

    let stake_info = STAKING_INFO.load(deps.storage)?;

    // locked is the total number of tokens user locked to this address
    // let locked = get_locked(deps.as_ref(), &env.contract.address)?;

     // calculate to_mint and update total supply
    let mut supply = TOTAL_SUPPLY.load(deps.storage)?;


    let tax = match lock_type {
        LockType::Long {} => lock_amount * stake_info.tax.long,
        LockType::Short {} => lock_amount * stake_info.tax.short
     };

    let to_mint = lock_amount - tax;

    supply.locked  += lock_amount;
    supply.issued += to_mint;
    supply.fees += tax;

    TOTAL_SUPPLY.save(deps.storage, &supply)?;

      // call into cw20-base to mint the token, call as self as no one else is allowed
      let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    let user_staking = UserStakingInfo {
        idx: idx.clone(),
        owner: sender.clone(),
        amount: lock_amount,
        date: env.block.time.seconds(),
        period: match lock_type {
            LockType::Long {} => stake_info.period.long,
            LockType::Short{} => stake_info.period.short
        }
    };

    USER_STAKING.update(deps.storage, &idx, |old| match old {
        Some(_) => Err(ContractError::InvalidIndex{}),
        None => Ok(user_staking)
    })?;

    execute_mint(deps, env, sub_info, sender.to_string(), to_mint)?;


    // bond them to the validator
    let res = Response::new()
    .add_attribute("action", "lock")
    .add_attribute("from", sender)
    .add_attribute("locked", lock_amount)
    .add_attribute("minted", to_mint);
    Ok(res)

}


pub fn unlock(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    idx: String,
    amount: Uint128
) -> Result<Response, ContractError> {
    let stake_info = STAKING_INFO.load(deps.storage)?;
    let user_staking_info = USER_STAKING.load(deps.storage, &idx)?;
   
    if info.sender != user_staking_info.owner {
       return Err(ContractError::Unauthorized {})
    } else if amount > user_staking_info.amount {
        return Err(ContractError::InsufficientLockBalance{});
    }
    // calculate tax and remainer to unlock
    let tax : Uint128 = if  env.block.time.seconds() < user_staking_info.date + user_staking_info.period {
         user_staking_info.amount * stake_info.penalty
    } else {
         Uint128::zero()
    };

    // burn from the original caller
    execute_burn(deps.branch(), env.clone(), info.clone(), amount)?;
  

    // calculate how many native tokens this is worth and update supply
    let unlock = amount.checked_sub(tax).map_err(StdError::overflow)?;
    let mut supply = TOTAL_SUPPLY.load(deps.storage)?;
    // TODO: this is just a safety assertion - do we keep it, or remove caching?
    // in the end supply is just there to cache the (expected) results of get_bonded() so we don't
    // have expensive queries everywhere
    if amount == user_staking_info.amount {
        USER_STAKING.remove(deps.storage, &idx)?;
    } else {
        USER_STAKING.update(deps.storage, &idx, |old| match old {
            Some(_) => Ok(UserStakingInfo{
                amount: user_staking_info.amount.checked_sub(amount).map_err(StdError::overflow)?,
                ..user_staking_info
            }),
            None => Err(ContractError::Unauthorized {  })
        })?;
    }

    supply.locked = supply.locked.checked_sub(unlock).map_err(StdError::overflow)?;
    supply.issued = supply
        .issued
        .checked_sub(amount)
        .map_err(StdError::overflow)?;
    supply.fees += tax;
    TOTAL_SUPPLY.save(deps.storage, &supply)?;



    // unbond them
    let res = Response::new()
        .add_attribute("action", "unlock")
        .add_attribute("to", info.sender)
        .add_attribute("unlocked", unlock)
        .add_attribute("burnt", amount);
    Ok(res)
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
       
        QueryMsg::Investment {} => to_json_binary(&query_investment(deps)?),
        // inherited from cw20-base
        QueryMsg::TokenInfo {} => to_json_binary(&query_token_info(deps)?),
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, address)?),
        QueryMsg::Allowance { owner, spender } => {
            to_json_binary(&query_allowance(deps, owner, spender)?)
        },
        QueryMsg::StakingInfo { owner } => to_json_binary(&query_user_staking(deps, owner)?)
    }
}

pub fn query_user_staking(deps: Deps, owner: String) -> StdResult<UserStakingInfoResponse> {
    let user_stakings =  USER_STAKING
    .idx
    .owner
    .prefix(Addr::unchecked(owner))
    .range(deps.storage, None, None, Order::Ascending)
    .map(|result| result.unwrap().1)
    .collect::<Vec<UserStakingInfo>>();

    Ok(UserStakingInfoResponse {infos: user_stakings} )
}

pub fn query_investment(deps: Deps) -> StdResult<InvestmentResponse> {
    let stake_info = STAKING_INFO.load(deps.storage)?;
    let supply = TOTAL_SUPPLY.load(deps.storage)?;

    let res = InvestmentResponse {
        owner: stake_info.owner.to_string(),
        penalty: stake_info.penalty,
        token_supply: supply.issued,
        staked_tokens: coin(supply.locked.u128(), &stake_info.stake_denom),
        nominal_value: if supply.issued.is_zero() {
            FALLBACK_RATIO
        } else {
            Decimal::from_ratio(supply.locked, supply.issued)
        },
        period: LockPrd {
            long: stake_info.period.long,
            short:stake_info.period.short
        },
        tax: LockTax {
            long: stake_info.tax.long,
            short: stake_info.tax.short
        }
    };
    Ok(res)
}


#[cfg(test)]
mod tests {

    use super::*;

    use cosmwasm_std::testing::{
         mock_dependencies_with_balance, mock_env, mock_info 
    };

 
 
    fn default_instantiate() -> InstantiateMsg {
        InstantiateMsg {
            name: "xQtum".to_string(),
            symbol: "xQtum".to_string(),
            decimals: 6,
            long_period:  30 * 14400,
            short_period: 15 * 14400,
            long_tax: 2,
            short_tax: 3,
            penalty: 2,
            stake_denom: "qtum".to_string(),
            staking_token_address: "qtum".to_string()
        }
    }


    fn get_balance<U: Into<String>>(deps: Deps, addr: U) -> Uint128 {
        query_balance(deps, addr.into()).unwrap().balance
    }


    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies_with_balance(&[]);
        let creator = String::from("creator");

        let msg = InstantiateMsg {
            name: "xQtum".to_string(),
            symbol: "xQtum".to_string(),
            decimals: 6,
            long_period:  30 * 14400,
            short_period: 15 * 14400,
            long_tax: 2,
            short_tax: 3,
            penalty: 2,
            stake_denom: "qtum".to_string(),
            staking_token_address: Addr::unchecked("qtum").to_string()
        };
        let info = mock_info(&creator, &[]);

        let res =  instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());


        let token = query_token_info(deps.as_ref()).unwrap();

        assert_eq!(&token.name, &msg.name);
        assert_eq!(&token.symbol, &msg.symbol);
        assert_eq!(token.decimals, msg.decimals);
        assert_eq!(token.total_supply, Uint128::zero());

        // no balance
        assert_eq!(get_balance(deps.as_ref(), &creator), Uint128::zero());

        let staking_info = query_investment(deps.as_ref()).unwrap();
        assert_eq!(&staking_info.owner, &creator);
        assert_eq!(staking_info.staked_tokens, coin(0, "qtum"));
        assert_eq!(staking_info.tax, LockTax {long: Decimal::percent(msg.long_tax), short: Decimal::percent(msg.short_tax)});
        assert_eq!(staking_info.token_supply, Uint128::zero());

    
    }

    #[test]
    fn lock() {

        let mut deps = mock_dependencies_with_balance(&[]);


        let instantiate_msg = default_instantiate();
        let info = mock_info("addr0000", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "addr0000".to_string(),
            amount: Uint128::from(100u128),
            msg: to_json_binary(&Cw20HookMsg::Lock {lock_type:LockType::Long {}, idx: "1".to_string()}).unwrap(),
        });

        let info = mock_info("qtum", &[]);
        let env = mock_env();
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();       

        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "addr0000".to_string(),
            amount: Uint128::from(100u128),
            msg: to_json_binary(&Cw20HookMsg::Lock {lock_type:LockType::Short  {}, idx:"2".to_string()}).unwrap(),
        });
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "addr1111".to_string(),
            amount: Uint128::from(100u128),
            msg: to_json_binary(&Cw20HookMsg::Lock {lock_type:LockType::Short  {}, idx:"3".to_string()}).unwrap(),
        });
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();


        let supply_info = TOTAL_SUPPLY.load(deps.as_ref().storage).unwrap();

        assert_eq!(supply_info.issued, Uint128::new(292) );

        let user_staking = query_user_staking(deps.as_ref(), "addr0000".to_string()).unwrap();
       
        assert_eq!(user_staking.infos.len(), 2);

        let info = mock_info("addr0000", &[]);

        let msg = ExecuteMsg::UnLock { 
            idx:  "1".to_string(), 
            amount: Uint128::from(100u128),
        };

        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        let user_staking = query_user_staking(deps.as_ref(), "addr0000".to_string()).unwrap();
        let supply_info = TOTAL_SUPPLY.load(deps.as_ref().storage).unwrap();
        // println!("user_staking {}", user_staking.infos[0].amount);
        assert_eq!(supply_info.issued, Uint128::new(192) );
        assert_eq!(user_staking.infos.len(), 1);

    }

}