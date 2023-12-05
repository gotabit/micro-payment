use cosmwasm_std::{
    from_json, to_binary, BankMsg, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, Uint128, WasmMsg,
};

use crate::{
    error::ContractError,
    msg::{self, *},
    state::{Config, Denom, PaymentChannel, CONFIG, PAYMENT_CHANNELS},
};
use cw20::Cw20ReceiveMsg;

use cosmwasm_tools::access_ctrl as constraints;

pub fn add_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_pubkey_hash: Vec<u8>,
    recipients: Vec<(String, u128)>,
) -> Result<Response, ContractError> {
    // TODO:
    let cfg = CONFIG.load(deps.storage)?;

    Ok(Response::new())
}

pub fn close_payment(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    sender_pubkey_hash: String,
    sender_commitment: Vec<u8>,
    recipients: Vec<(String, Vec<u8>)>, // recipient_pubkey_hash, recipient_commitment
) -> Result<Response, ContractError> {
    verify_commitment(
        &sender_pubkey_hash,
        CommitmentType::CloseChannel,
        sender_commitment,
    )?;

    let cfg = CONFIG.load(deps.storage)?;

    let mut payment_chan = PAYMENT_CHANNELS.load(deps.storage, sender_pubkey_hash.clone())?;

    for (addr, commitment) in recipients {
        let mut channel = payment_chan
            .recipients
            .iter_mut()
            .find(|x| x.recipient_pubkey_hash == addr);
        let mut refund_amt = 0;
        if let Some(chan) = channel {
            if verify_commitment(&addr, CommitmentType::CloseChannel, commitment).is_ok() {
                // settlement
            } else {
                // auto release
                if let Some(auto_release) = chan.auto_release {
                    if auto_release <= env.block.time.seconds() {
                        refund_amt += chan.remain()
                    }
                    // TODO: remove channel
                } else {
                    chan.auto_release = Some(env.block.time.seconds() + cfg.auto_release_time);
                }
            }
        }
    }

    PAYMENT_CHANNELS.save(deps.storage, sender_pubkey_hash, &payment_chan)?;

    Ok(Response::new())
}

use cw20::Cw20ExecuteMsg;

fn build_transfer_msg(
    cfg: &Config,
    resp: &mut Response,
    to: String,
    amt: u128,
) -> Result<(), ContractError> {
    match cfg.denom.clone() {
        Denom::Native(denom) => {
            resp.clone().add_message(BankMsg::Send {
                to_address: to,
                amount: vec![Coin {
                    denom,
                    amount: Uint128::new(amt),
                }],
            });
        }
        Denom::Cw20(addr) => {
            let refund_msg = Cw20ExecuteMsg::Transfer {
                recipient: to,
                amount: Uint128::new(amt),
            };

            let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                msg: to_binary(&refund_msg)?,
                funds: vec![],
            });
            resp.clone().add_message(msg);
        }
    }

    Ok(())
}

pub fn payment_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient_pubkey_hash: String,
    cheques: Vec<(PaymentCheque, PaymentCheque)>,
) -> Result<Response, ContractError> {
    // TODO:
    let cfg = CONFIG.load(deps.storage)?;
    payment_check_interval_verify(deps.as_ref(), &cheques)?;

    let mut total_value = 0;
    for (start, end) in cheques {
        let mut payment_channel = PAYMENT_CHANNELS.load(deps.storage, end.sender_pubkey_hash)?;
        let channel = payment_channel
            .recipients
            .iter_mut()
            .find(|x| x.recipient_pubkey_hash == recipient_pubkey_hash);
        if let Some(chan) = channel {
            chan.nonce_withdrawl = Some(end.nonce);
            total_value += (end.nonce - start.nonce) as u128 * chan.face_value.unwrap();

            PAYMENT_CHANNELS.save(deps.storage, start.sender_pubkey_hash, &payment_channel)?;
        }
    }
    // TODO: make transfer
    //
    Ok(Response::new())
}

pub enum CommitmentType {
    Cheque,
    CloseChannel,
}

fn verify_commitment(
    sender_pubkey_hash: &String,
    commitment_type: CommitmentType,
    commitment: Vec<u8>,
) -> Result<(), ContractError> {
    // TODO:
    Ok(())
}
fn payment_check_interval_verify(
    deps: Deps,
    checks: &Vec<(PaymentCheque, PaymentCheque)>,
) -> Result<(), ContractError> {
    // TODO:
    for (start, end) in checks {
        // verify commitment and noce
    }
    Ok(())
}

// admin only
#[constraints(owner_only(deps.as_ref(), &info, None))]
pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    auto_release_time: Option<u64>,
    owner: Option<String>,
    max_recipient: Option<u32>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if let Some(auto_release_time) = auto_release_time {
        config.auto_release_time = auto_release_time;
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    if let Some(max_recipient) = max_recipient {
        config.max_recipient = max_recipient;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

pub fn receive_cw20(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    match cfg.denom {
        Denom::Cw20(addr) => {
            assert_eq!(addr.to_string(), info.sender.to_string());

            let execute_msg: ExecuteMsg = from_json(msg.msg)?;
            match execute_msg {
                ExecuteMsg::AddPaymentChan {
                    sender_pubkey_hash,
                    recipients,
                    face_value,
                } => {
                    let mut total_fund: u128 = 0;

                    let _ = recipients.iter().map(|(_, amount)| total_fund += amount);
                    if msg.amount.u128() < total_fund {
                        return Err(ContractError::InsufficientFund);
                    }

                    let mut payment_chan =
                        PAYMENT_CHANNELS.load(deps.storage, sender_pubkey_hash.clone())?;

                    for (recipient, amt) in recipients {
                        payment_chan.add(recipient, amt, face_value);
                        if payment_chan.recipients.len() > cfg.max_recipient as usize {
                            return Err(ContractError::ExceedRecipientNum);
                        }
                    }

                    PAYMENT_CHANNELS.save(
                        deps.storage,
                        sender_pubkey_hash.clone(),
                        &payment_chan,
                    )?;
                }
                _ => return Err(ContractError::UnsupportMsg),
            }
        }
        Denom::Native(_) => return Err(ContractError::UnsupportDenom()),
    }

    Ok(Response::new().add_attribute("method", "receive_cw20"))
}

fn owner_only(
    deps: Deps,
    info: &MessageInfo,
    owner: Option<CanonicalAddr>,
) -> Result<bool, ContractError> {
    let owner = owner.unwrap_or(CONFIG.load(deps.storage)?.owner);

    let sender = deps.api.addr_canonicalize(info.sender.as_str())?;
    if sender != owner {
        return Err(ContractError::NotOwner {
            sender: info.sender.to_string(),
            owner: deps.api.addr_humanize(&owner)?.to_string(),
        });
    }

    Ok(true)
}
