use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized Token")]
    Unauthorized {},


    #[error("Invalid Lock Type")]
    InvalidLockType {},


    #[error("Invalid Staking token")]
    InvalidStakingToken {},

    #[error("Stored locked {stored}, but query locked {queried}")]
    LockedMismatch { stored: Uint128, queried: Uint128 },

    #[error("No {denom} tokens sent")]
    EmptyBalance { denom: String },


    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Invalid expiration")]
    InvalidExpiration {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("No allowance for this account")]
    NoAllowance {},

    #[error("Minting cannot exceed the cap")]
    CannotExceedCap {},

    #[error("Duplicate initial balance addresses")]
    DuplicateInitialBalanceAddresses {},
}

impl From<cw20_base::ContractError> for ContractError {
    fn from(err: cw20_base::ContractError) -> Self {
        match err {
            cw20_base::ContractError::Std(error) => ContractError::Std(error),
            cw20_base::ContractError::Unauthorized {} => ContractError::Unauthorized {},
            cw20_base::ContractError::CannotSetOwnAccount {} => {
                ContractError::CannotSetOwnAccount {}
            }
            cw20_base::ContractError::InvalidExpiration {} => ContractError::InvalidExpiration {},
            cw20_base::ContractError::InvalidZeroAmount {} => ContractError::InvalidZeroAmount {},
            cw20_base::ContractError::Expired {} => ContractError::Expired {},
            cw20_base::ContractError::NoAllowance {} => ContractError::NoAllowance {},
            cw20_base::ContractError::CannotExceedCap {} => ContractError::CannotExceedCap {},
            // This should never happen, as this contract doesn't use logo
            cw20_base::ContractError::LogoTooBig {}
            | cw20_base::ContractError::InvalidPngHeader {}
            | cw20_base::ContractError::InvalidXmlPreamble {} => {
                ContractError::Std(StdError::generic_err(err.to_string()))
            }
            cw20_base::ContractError::DuplicateInitialBalanceAddresses {} => {
                ContractError::DuplicateInitialBalanceAddresses {}
            }
        }
    }
}
