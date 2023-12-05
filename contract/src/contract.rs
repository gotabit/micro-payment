use crate::error::ContractError;
use crate::handler::*;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};

const CONTRACT_NAME: &str = "crates.io:micro_payment";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
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
