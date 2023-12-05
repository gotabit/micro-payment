use cosmwasm_std::{Addr, CanonicalAddr};
use cosmwasm_tools::config_item;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PAYMENT_CHANNELS: Map<String, PaymentChannel> = Map::new("payment_channel");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Denom {
    Native(String),
    Cw20(Addr),
}

#[config_item]
pub struct Config {
    pub denom: Denom,
    pub auto_release_time: u64,
    pub owner: CanonicalAddr,
    pub max_recipient: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PaymentChannel {
    pub recipients: Vec<Recipient>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Recipient {
    pub recipient_pubkey_hash: String,
    pub max_amount: u128,
    pub nonce_withdrawl: Option<u64>,
    pub face_value: Option<u128>,
    pub auto_release: Option<u64>,
}

impl Recipient {
    pub fn remain(&self) -> u128 {
        self.max_amount - self.nonce_withdrawl.unwrap_or(0) as u128 * self.face_value.unwrap_or(0)
    }
}

impl PaymentChannel {
    pub fn add(&mut self, recipient_pubkey_hash: String, amount: u128, face_value: Option<u128>) {
        // add code here
        if let Some(recipient) = self
            .recipients
            .iter_mut()
            .find(|x| x.recipient_pubkey_hash == recipient_pubkey_hash)
            .as_mut()
        {
            recipient.max_amount = recipient.max_amount + amount;
            return;
        }

        self.recipients.push(Recipient {
            recipient_pubkey_hash,
            max_amount: amount,
            nonce_withdrawl: None,
            auto_release: None,
            face_value,
        });
    }
}
