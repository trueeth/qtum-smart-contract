use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr,  Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item,  MultiIndex};


#[cw_serde]
pub struct LockPrd {
    pub long: u64,
    pub short: u64,
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
    pub staking_token_address: Addr,
    pub period : LockPrd,
    /// This is how much the owner takes as a cut when someone unstake
    pub tax: LockTax,
    /// This is how much the staker pay for unstake the qtum before period
    pub penalty: Decimal
}

#[cw_serde]

pub struct UserStakingInfo {
    pub idx: String,
    pub owner: Addr,
    pub date : u64,
    pub amount : Uint128,
    pub period: u64
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


pub struct  UserStakingIndexes<'a> {
    pub owner: MultiIndex<'a, Addr, UserStakingInfo, String>
}


impl<'a> IndexList<UserStakingInfo> for UserStakingIndexes<'a> {
    
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<UserStakingInfo>> + '_> {
        let v: Vec<&dyn Index<UserStakingInfo>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}


pub fn staking_owner_idx(_pk: &[u8], d: &UserStakingInfo) -> Addr {
    d.owner.clone()
}



pub const USER_STAKING : IndexedMap<&str, UserStakingInfo, UserStakingIndexes>  = 
    IndexedMap::new("user_staking_info", UserStakingIndexes {
        owner: MultiIndex::new(staking_owner_idx, "user_staking_info", "tokens__owner")
    });


