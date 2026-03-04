pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: crate::msg::InstantiateMsg,
) -> Result<Response, ContractError> {
    contract::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: crate::msg::ExecuteMsg,
) -> Result<Response, ContractError> {
    contract::execute(deps, env, info, msg)
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: crate::msg::QueryMsg,
) -> StdResult<Binary> {
    contract::query(deps, env, msg)
}
