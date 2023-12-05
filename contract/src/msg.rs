use crate::state::Denom;
use crate::state::Recipient;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: Denom,
    pub admin: Option<String>,
    pub auto_release_time: u64,
    pub max_recipient: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddPaymentChan {
        sender_pubkey_hash: String,
        recipients: Vec<(String, u128)>,
        face_value: Option<u128>,
    },
    ClosePaymentChan {
        sender_pubkey_hash: String,
        sender_commitment: Vec<u8>,
        recipient_pubkey_hash: String,
        recipient_commitment: Option<Vec<u8>>,
    },
    PaymentClaim {
        sender_pubkey_hash: String,
        checks: Vec<(PaymentCheque, PaymentCheque)>,
    },
    /// Change the admin
    UpdateConfig {
        admin: Option<String>,
        auto_release_time: u64,
        max_recipient: Option<u32>,
    },
    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PaymentCheque {
    pub sender_pubkey_hash: String,
    pub sender_commitment: Vec<u8>,
    pub recipient_pubkey_hash: String,
    pub recipient_commitment: Vec<u8>,
    pub value: u128,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}
