use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("unsupport denom")]
    UnsupportDenom(),
    #[error("unsupport msg")]
    UnsupportMsg,
    #[error("Insufficient funds")]
    InsufficientFund,
    #[error("Exceed max recipient num")]
    ExceedRecipientNum,
    #[error("NotOwner: Sender is {sender}, but owner is {owner}.")]
    NotOwner { sender: String, owner: String },
    #[error("ErrChecks: Verify checks failed")]
    ChecksVerifyFailed,
}
