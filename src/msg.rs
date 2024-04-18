use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Binary, Coin, Decimal, Uint128};
use cw20::{Cw20ReceiveMsg, Expiration};
use cw20::{AllowanceResponse, BalanceResponse, TokenInfoResponse};
pub use cw_controllers::ClaimsResponse;
use crate::state::{LockPrd, LockTax};


#[cw_serde] 
pub struct InitMsg {
    /// name of the derivative token
    pub name: String,
    /// symbol / ticker of the derivative token
    pub symbol: String,
    /// decimal places of the derivative token (for UI)
    pub decimals: u8,
    /// staking token
    pub stake_denom: String
}

#[cw_serde]
pub struct InstantiateMsg {
    /// name of the derivative token
    pub name: String,
    /// symbol / ticker of the derivative token
    pub symbol: String,
    /// decimal places of the derivative token (for UI)
    pub decimals: u8,
    pub stake_denom: String,
    pub staking_token_address: String,

    pub long_period: u64,
    pub short_period: u64,
    pub long_tax: u64,
    pub short_tax: u64,
    pub penalty: u64
}

#[cw_serde]
pub enum ExecuteMsg {

    Receive(Cw20ReceiveMsg),
    /// Unbond will "burn" the given amount of derivative tokens and send the unbonded
    /// staking tokens to the message sender (after exit tax is deducted)
    UnLock { amount: Uint128 },

    /// Implements CW20. Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Implements CW20. Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Implements CW20.  Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Implements CW20 "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

    /// Investment shows metadata on the staking info of the contract
    #[returns(InvestmentResponse)]
    Investment {},

    /// Implements CW20. Returns the current balance of the given address, 0 if unset.
    #[returns(BalanceResponse)]
    Balance { address: String },
    /// Implements CW20. Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(TokenInfoResponse)]
    TokenInfo {},
    /// Implements CW20 "allowance" extension.
    /// Returns how much spender can use from owner account, 0 if unset.
    #[returns(AllowanceResponse)]
    Allowance { owner: String, spender: String },
}

#[cw_serde]
pub struct InvestmentResponse {
    pub token_supply: Uint128,
    pub staked_tokens: Coin,
    // ratio of staked_tokens / token_supply (or how many native tokens that one derivative token is nominally worth)
    pub nominal_value: Decimal,

    /// owner created the contract and takes a cut
    pub owner: String,
    /// this is how much the owner takes as a cut when someone unbonds
    pub penalty: Decimal,
    /// This is how much the owner takes as a cut when someone unstake
    pub tax: LockTax,
    /// after this perio, you can get back your qtum token, 
    pub period : LockPrd,

}


#[cw_serde]
pub enum LockType {
    Long {},
    Short {}
}


#[cw_serde]
pub enum Cw20HookMsg {
    Lock {
        lock_type: LockType
    },
}

