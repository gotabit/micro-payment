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
        //添加支付通道
        chan_key: String,         // 通道key， sender pubkey hash
        channels: Vec<Channel>,   // 通道信息
        operator: Option<String>, // 操作者地址
    },
    ClosePaymentChan {
        // 关闭支付通道
        chan_key: String,                 // 通道key， sender pubkey hash
        channels: Vec<(String, Vec<u8>)>, // tuple(接受者key， 接受者授权证明)
    },
    AddSigner {
        // 添加授权这杯
        chan_key: String,      // 通道key， sender pubkey hash
        recipient_key: String, // 接受者pubkey hash
        signers: Vec<String>,  // 子设备pubkey
    },
    Cashing {
        // 提取支票
        recipient_key: String,       // 接受者pubkey hash
        cheques: Vec<PaymentCheque>, // 最后一张支票信息
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
    // 支付通道
    pub key: String,                  // sender pubkey hash
    pub max_amount: u128,             // 最大支付数量
    pub face_value: Option<u128>,     // 面值
    pub approve_signers: Vec<String>, // 授权的子设备
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PaymentCheque {
    pub cheque: Cheque,                // 通道id
    pub sender_commitment: Vec<u8>,    // sender授权证明
    pub recipient_commitment: Vec<u8>, // 接受者证明文件
    pub value: Option<u128>,           // 支票数额, 可选字段，若为等额支票则为空即可
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cheque {
    pub chan_id: u32,          // 通道id
    pub sender_key: String,    // sender pubkey
    pub recipient_key: String, // 接受者pubkey hash
    pub seq: u64,              // 序号(又称nonce)
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
