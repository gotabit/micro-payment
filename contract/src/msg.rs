use crate::state::Denom;
use crate::state::{Config, Recipient};
use cosmwasm_schema::{cw_serde, QueryResponses};
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
        chan_key: String,
        channels: Vec<Channel>,
        operator: Option<String>,
    },
    ClosePaymentChan {
        chan_key: String,
        commitment: Vec<u8>,
        channels: Vec<(String, Vec<u8>)>,
    },
    AddSigner {
        chan_key: String,
        recipient_key: String,
        signers: Vec<String>,
    },
    Cashing {
        recipient_key: String,
        cheques: Vec<PaymentCheque>,
    },
    /// Change the admin
    UpdateConfig {
        owner: Option<String>,
        auto_release_time: Option<u64>,
        max_recipient: Option<u32>,
    },
    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    pub key: String,
    pub max_amount: u128,
    pub face_value: Option<u128>,
    pub approve_signers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PaymentCheque {
    pub sender_key: String,
    pub sender_commitment: Vec<u8>,
    pub recipient_key: String,
    pub recipient_commitment: Vec<u8>,
    pub value: Option<u128>,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    #[returns(Option<Vec<Recipient>>)]
    PaymentChan {
        sender_pubkey_hash: String,
        recipient_pubkey_hash: Option<String>,
        page: Option<u32>,
        size: Option<u32>,
    },
    #[returns(Config)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
