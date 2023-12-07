use crate::error::ContractError;
use crate::handler::*;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};

const CONTRACT_NAME: &str = "crates.io:micro_payment";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = deps.api.addr_canonicalize(info.sender.as_str())?;
    CONFIG.save(deps.storage, &{
        Config {
            denom: msg.denom,
            auto_release_time: msg.auto_release_time,
            max_recipient: msg.max_recipient,
            owner,
        }
    })?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => handle_cw20_msg(deps, env, info, msg),
        ExecuteMsg::ClosePaymentChan {
            sender_pubkey_hash,
            sender_commitment,
            recipients,
        } => close_payment(
            deps,
            env,
            info,
            sender_pubkey_hash,
            sender_commitment,
            recipients,
        ),
        ExecuteMsg::Cashing {
            recipient_pubkey_hash,
            cheques,
        } => cashing(deps, env, info, recipient_pubkey_hash, cheques),
        ExecuteMsg::UpdateConfig {
            owner,
            auto_release_time,
            max_recipient,
        } => update_config(deps, env, info, auto_release_time, owner, max_recipient),
        _ => panic!("unsupport"),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // QueryMsg::Config {} => to_binary(&query::config(deps)?),
        // QueryMsg::Auction { id } => to_binary(&query::auction(deps, id)?),
        // QueryMsg::AuctionList {
        //     status,
        //     page,
        //     limit,
        // } => to_binary(&query::auction_list(deps, env, status, page, limit)?),
        _ => Ok(Binary(vec![])),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _: MigrateMsg) -> Result<Response, ContractError> {
    let ver = get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }

    if ver.version.gt(&CONTRACT_VERSION.to_string()) {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }

    // set the new version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}