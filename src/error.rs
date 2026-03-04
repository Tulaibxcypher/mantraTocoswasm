use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Not Whitelisted")]
    NotWhitelisted,
    #[error("Transfers are paused")]
    Paused,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Max supply exceeded")]
    MaxSupplyExceeded,
}
