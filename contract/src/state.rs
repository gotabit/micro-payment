use std::collections::HashMap;

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
    pub recipients: HashMap<String, Recipient>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Recipient {
    pub max_amount: u128,
    pub nonce_withdrawl: Option<u64>,
    pub face_value: Option<u128>,
    pub auto_release: Option<u64>,
}

impl Recipient {
    #[inline]
    pub fn new(max_amount: u128, face_value: u128) -> Self {
        Self {
            max_amount,
            nonce_withdrawl: None,
            face_value: Some(face_value),
            auto_release: None,
        }
    }

    pub fn remain(&self) -> u128 {
        self.max_amount - self.nonce_withdrawl.unwrap_or(0) as u128 * self.face_value.unwrap_or(0)
    }
}
