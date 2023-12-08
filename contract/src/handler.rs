use cosmwasm_std::{
    from_json, to_json_binary, BankMsg, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, SubMsg, Uint128, WasmMsg,
};

use crate::{
    error::ContractError,
    msg::*,
    state::{Config, Denom, PaymentChannel, Recipient, CONFIG, PAYMENT_CHANNELS},
};
use cw20::Cw20ReceiveMsg;
use std::collections::HashMap;

use cosmwasm_tools::access_ctrl as constraints;

pub fn handle_cw20_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let raw: ExecuteMsg = from_json(&msg.msg)?;
    match raw {
        ExecuteMsg::AddPaymentChan {
            sender_pubkey_hash,
            recipients,
        } => return build_payment_chan(deps, env, info, msg, sender_pubkey_hash, recipients),
        _ => unreachable!(),
    }
}

pub fn build_payment_chan(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: Cw20ReceiveMsg,
    sender_pubkey_hash: String,
    recipients: Vec<(String, u128, u128)>, // recipient_pubkey_hash, face_value, total
) -> Result<Response, ContractError> {
    let mut total_amt = 0;
    for (_, _, amt) in recipients.iter() {
        total_amt += *amt
    }
    if msg.amount.lt(&Uint128::from(total_amt)) {
        return Err(ContractError::InsufficientFund);
    }

    let mut payment_chan = PAYMENT_CHANNELS
        .may_load(deps.storage, sender_pubkey_hash.clone())?
        .unwrap_or(PaymentChannel {
            recipients: HashMap::new(),
        });

    for (recipient_pubkey_hash, face_value, total) in recipients {
        let recipient = payment_chan
            .recipients
            .get_mut(recipient_pubkey_hash.as_str());
        if let Some(r) = recipient {
            r.max_amount += total;
        } else {
            payment_chan.recipients.insert(
                recipient_pubkey_hash.clone(),
                Recipient::new(total, face_value),
            );
        }
    }

    PAYMENT_CHANNELS.save(deps.storage, sender_pubkey_hash, &payment_chan)?;

    Ok(Response::new().add_attribute("method", "add_payment"))
}

pub fn close_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
    let mut refund_amt = 0;
    for (addr, commitment) in recipients {
        let recipient = payment_chan.recipients.get_mut(addr.as_str());
        if recipient.is_none() {
            continue;
        }

        if let Some(r) = recipient {
            if verify_commitment(&addr, CommitmentType::CloseChannel, commitment).is_ok() {
                // settlement
                refund_amt += r.remain();
                payment_chan.recipients.remove(addr.as_str());
            } else {
                // auto release
                if let Some(auto_release) = r.auto_release {
                    if auto_release <= env.block.time.seconds() {
                        refund_amt += r.remain()
                    }

                    payment_chan.recipients.remove(addr.as_str());
                } else {
                    r.auto_release = Some(env.block.time.seconds() + cfg.auto_release_time);
                }
            }
        }
    }

    // make refund
    let sub_msg = build_transfer_msg(&cfg, info.sender.to_string(), refund_amt)?;

    PAYMENT_CHANNELS.save(deps.storage, sender_pubkey_hash, &payment_chan)?;

    Ok(Response::new().add_submessages(sub_msg))
}

use cw20::Cw20ExecuteMsg;

fn build_transfer_msg(cfg: &Config, to: String, amt: u128) -> Result<Vec<SubMsg>, ContractError> {
    let mut res = vec![];
    match cfg.denom.clone() {
        Denom::Native(denom) => {
            res.push(SubMsg::new(BankMsg::Send {
                to_address: to,
                amount: vec![Coin {
                    denom,
                    amount: Uint128::new(amt),
                }],
            }));
        }
        Denom::Cw20(addr) => {
            let refund_msg = Cw20ExecuteMsg::Transfer {
                recipient: to,
                amount: Uint128::new(amt),
            };

            let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                msg: to_json_binary(&refund_msg)?,
                funds: vec![],
            });
            res.push(SubMsg::new(msg));
        }
    }

    Ok(res)
}

pub fn cashing(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient_pubkey_hash: String,
    cheques: Vec<(PaymentCheque, PaymentCheque)>,
) -> Result<Response, ContractError> {
    payment_check_interval_verify(deps.as_ref(), &cheques)?;

    let cfg = CONFIG.load(deps.storage)?;
    let mut total_cash = 0;
    for (start, end) in cheques {
        assert!(end.nonce >= start.nonce);
        assert_eq!(end.sender_pubkey_hash, start.sender_pubkey_hash);

        let mut payment_chan =
            PAYMENT_CHANNELS.load(deps.storage, start.sender_pubkey_hash.clone())?;
        let recipient = payment_chan
            .recipients
            .get_mut(recipient_pubkey_hash.as_str())
            .unwrap();

        assert_eq!(recipient.nonce_withdrawl.unwrap_or(0) + 1, start.nonce);

        total_cash += (end.nonce - start.nonce + 1) as u128 * recipient.face_value.unwrap();
        recipient.nonce_withdrawl = Some(end.nonce);
        PAYMENT_CHANNELS.save(deps.storage, start.sender_pubkey_hash, &payment_chan)?;
    }

    let sub_msgs = build_transfer_msg(&cfg, info.sender.to_string(), total_cash)?;

    Ok(Response::new()
        .add_attribute("method", "cashing")
        .add_submessages(sub_msgs))
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

    Ok(Response::new().add_attribute("method", "update_config"))
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

const DEFAULT_SIZE: u32 = 10;

pub fn payment_chan(
    deps: Deps,
    _env: Env,
    sender_pubkey_hash: String,
    recipient_pubkey_hash: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
) -> StdResult<Option<Vec<Recipient>>> {
    let chan = PAYMENT_CHANNELS.load(deps.storage, sender_pubkey_hash)?;
    let mut res = vec![];
    if let Some(recipient_pubkey_hash) = recipient_pubkey_hash {
        let recipient_chan = chan.recipients.get(&recipient_pubkey_hash);
        if let Some(rc) = recipient_chan {
            res.push(rc.clone());
        }
    } else {
        let size = size.unwrap_or(DEFAULT_SIZE);
        let start = page.unwrap_or(0) * size;

        for (i, v) in chan.recipients.iter().enumerate() {
            if i >= start as usize {
                res.push(v.1.clone());
            }

            if i > (start + size) as usize {
                break;
            }
        }
    }

    Ok(Some(res))
}

pub fn config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}
