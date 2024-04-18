use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CanonicalAddr, Decimal, Uint128};
use cw_controllers::Claims;
use cw_storage_plus::Item;
use cw_utils::Duration;

pub const CLAIMS: Claims = Claims::new("claims");

#[cw_serde]
pub struct LockPrd {
    pub long: Duration,
    pub short: Duration,
}

#[cw_serde]
pub struct LockTax {
    pub long: Decimal,
    pub short: Decimal,
}


#[cw_serde]
pub struct StakingInfo {
    /// Owner created the contract and takes a cut
    pub owner: Addr,
    /// staking token denom
    pub stake_denom: String,
    /// after this perio, you can get back your qtum token, 
    /// 
    pub staking_token_address: CanonicalAddr,
    pub period : LockPrd,
    /// This is how much the owner takes as a cut when someone unstake
    pub tax: LockTax,
    /// This is how much the staker pay for unstake the qtum before period
    pub penalty: Decimal
}



/// Supply is dynamic and tracks the current supply of staked and ERC20 tokens.
#[cw_serde]
#[derive(Default)]
pub struct Supply {
    /// issued is how many derivative tokens this contract has issued
    pub issued: Uint128,
    /// bonded is how many qtum tokens locked on this contract
    pub locked: Uint128,
    /// fees is how many qtum tokens collected tax and penalty
    pub fees: Uint128,
}


pub const STAKING_INFO: Item<StakingInfo> = Item::new("staking_info");

pub const TOTAL_SUPPLY: Item<Supply> = Item::new("total_supply");
