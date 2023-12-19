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
    let config = CONFIG.load(deps.storage)?;
    match config.denom {
        Denom::Native(_) => return Err(ContractError::UnsupportDenom()),
        Denom::Cw20(addr) => assert_eq!(addr, info.sender),
    };

    let raw: ExecuteMsg = from_json(&msg.msg)?;
    match raw {
        ExecuteMsg::AddPaymentChan {
            chan_key: sender_pubkey_hash,
            channels,
            operator,
        } => build_payment_chan(
            deps,
            env,
            msg,
            sender_pubkey_hash,
            channels,
            operator.unwrap_or(info.sender.to_string()),
        ),
        _ => unreachable!(),
    }
}

pub fn build_payment_chan(
    deps: DepsMut,
    _env: Env,
    msg: Cw20ReceiveMsg,
    sender_pubkey_hash: String,
    channels: Vec<Channel>, // recipient_pubkey_hash, face_value, total
    operator: String,
) -> Result<Response, ContractError> {
    let mut total_amt = 0;
    for chan in channels.iter() {
        total_amt += chan.max_amount;
    }
    if msg.amount.lt(&Uint128::from(total_amt)) {
        return Err(ContractError::InsufficientFund);
    }
    // create a new one if not exist for the given key
    let mut payment_chan = PAYMENT_CHANNELS
        .may_load(deps.storage, sender_pubkey_hash.clone())?
        .unwrap_or(PaymentChannel {
            operator: operator.clone(),
            recipients: HashMap::new(),
        });

    assert_eq!(payment_chan.operator, operator);

    for chan in channels {
        let recipient = payment_chan.recipients.get_mut(chan.key.as_str());
        if let Some(r) = recipient {
            r.max_amount += chan.max_amount;
        } else {
            payment_chan.recipients.insert(
                chan.key.clone(),
                Recipient::new(
                    chan.approve_signers,
                    chan.max_amount,
                    chan.face_value.unwrap(),
                ),
            );
        }
    }

    PAYMENT_CHANNELS.save(deps.storage, sender_pubkey_hash, &payment_chan)?;

    Ok(Response::new().add_attribute("method", "add_payment"))
}

pub fn add_signer(
    deps: DepsMut,
    info: MessageInfo,
    sender_pubkey_hash: String,
    recipient_pubkey_hash: String,
    mut signers: Vec<String>,
) -> Result<Response, ContractError> {
    let mut payment_chan = PAYMENT_CHANNELS.load(deps.storage, sender_pubkey_hash.clone())?;

    assert_eq!(payment_chan.operator, info.sender.to_string());

    let recipient = payment_chan
        .recipients
        .get_mut(&recipient_pubkey_hash)
        .unwrap();

    recipient.approve_signers.append(&mut signers);

    PAYMENT_CHANNELS.save(deps.storage, sender_pubkey_hash, &payment_chan)?;

    Ok(Response::new().add_attribute("method", "add_signer"))
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

    assert_eq!(payment_chan.operator, info.sender.to_string());

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
    cheques: Vec<PaymentCheque>,
) -> Result<Response, ContractError> {
    payment_check_interval_verify(deps.as_ref(), &cheques)?;

    let cfg = CONFIG.load(deps.storage)?;
    let mut total_cash = 0;
    for cheque in cheques {
        let mut payment_chan = PAYMENT_CHANNELS.load(deps.storage, cheque.sender_key.clone())?;

        let recipient = payment_chan
            .recipients
            .get_mut(recipient_pubkey_hash.as_str())
            .unwrap();

        assert!(recipient.nonce_withdrawl.unwrap_or(0) < cheque.nonce);

        total_cash += (cheque.nonce - recipient.nonce_withdrawl.unwrap_or(0)) as u128
            * recipient.face_value.unwrap();
        recipient.nonce_withdrawl = Some(cheque.nonce);
        PAYMENT_CHANNELS.save(deps.storage, cheque.sender_key, &payment_chan)?;
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
    _sender_pubkey_hash: &str,
    _commitment_type: CommitmentType,
    _commitment: Vec<u8>,
) -> Result<(), ContractError> {
    // TODO:
    Ok(())
}
fn payment_check_interval_verify(
    _deps: Deps,
    checks: &Vec<PaymentCheque>,
) -> Result<(), ContractError> {
    // TODO:
    for _start in checks {
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
