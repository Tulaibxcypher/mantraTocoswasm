use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{TokenInfo, BALANCES, PAUSED, TOKEN_INFO, WHITELIST};
use crate::msg::{TokenInfoResponse, BalanceResponse, BoolResponse};

const CONTRACT_NAME: &str = "crates.io:mantra-rwa-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runs ONCE at deploy time
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let token = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        issuer: info.sender.clone(),
        total_supply: 0,
        max_supply: msg.max_supply,
    };

    TOKEN_INFO.save(deps.storage, &token)?;
    PAUSED.save(deps.storage, &false)?;
    WHITELIST.save(deps.storage, &info.sender, &true)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("issuer", info.sender)
        .add_attribute("max_supply", msg.max_supply.to_string()))
}

/// Routes all execute messages
/// BUG FIX 1: Removed underscores from deps/env/info/msg — they ARE used in match arms
/// BUG FIX 2: Moved Ok(...) outside the match block (was incorrectly inside last arm)
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint { to, amount } => mint(deps, info, to, amount),

        ExecuteMsg::AddToWhitelist { addr } => add_to_whitelist(deps, info, addr),

        ExecuteMsg::RemoveFromWhitelist { addr } => remove_from_whitelist(deps, info, addr),

        ExecuteMsg::Transfer { to, amount } => transfer(deps, info, to, amount),

        ExecuteMsg::SetPaused { paused } => set_paused(deps, info, paused),
    }
    // BUG: previously had Ok(Response::new()...) here INSIDE the match — wrong placement
}

/// Add address to whitelist (issuer only)
fn add_to_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    let token = TOKEN_INFO.load(deps.storage)?;

    if info.sender != token.issuer {
        return Err(ContractError::Unauthorized);
    }

    let address = deps.api.addr_validate(&addr)?;
    WHITELIST.save(deps.storage, &address, &true)?;

    Ok(Response::new()
        .add_attribute("action", "add_to_whitelist")
        .add_attribute("addr", address))
}

/// Remove address from whitelist (issuer only)
fn remove_from_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    let token = TOKEN_INFO.load(deps.storage)?;

    if info.sender != token.issuer {
        return Err(ContractError::Unauthorized);
    }

    let address = deps.api.addr_validate(&addr)?;
    WHITELIST.save(deps.storage, &address, &false)?;

    Ok(Response::new()
        .add_attribute("action", "remove_from_whitelist")
        .add_attribute("addr", address))
}

/// Mint new tokens (issuer only, recipient must be whitelisted)
fn mint(
    deps: DepsMut,
    info: MessageInfo,
    to: String,
    amount: u128,
) -> Result<Response, ContractError> {
    let mut token = TOKEN_INFO.load(deps.storage)?;

    if info.sender != token.issuer {
        return Err(ContractError::Unauthorized);
    }

    let to_addr = deps.api.addr_validate(&to)?;

    let is_whitelisted = WHITELIST
        .may_load(deps.storage, &to_addr)?
        .unwrap_or(false);

    if !is_whitelisted {
        return Err(ContractError::NotWhitelisted);
    }

    let new_total = token.total_supply.saturating_add(amount);
    if new_total > token.max_supply {
        return Err(ContractError::MaxSupplyExceeded);
    }

    token.total_supply = new_total;
    TOKEN_INFO.save(deps.storage, &token)?;

    let current = BALANCES.may_load(deps.storage, &to_addr)?.unwrap_or(0);
    BALANCES.save(deps.storage, &to_addr, &(current.saturating_add(amount)))?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", to_addr)
        .add_attribute("amount", amount.to_string()))
}

/// Query handler
/// BUG FIX 3: Was `Match msg` (capital M) — Rust keywords are lowercase
/// BUG FIX 4: Was `_deps` / `_env` / `_msg` but then used as `deps` / `msg` — removed underscores
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo {} => {
            let token = TOKEN_INFO.load(deps.storage)?;
            let response = TokenInfoResponse {
                name: token.name,
                symbol: token.symbol,
                total_supply: token.total_supply,
                max_supply: token.max_supply,
            };
            to_json_binary(&response)
        }
        QueryMsg::IsPaused {} => {
            let paused = PAUSED.load(deps.storage)?;
            to_json_binary(&BoolResponse { value: paused })
        }
        QueryMsg::IsWhitelisted { addr } => {
            let address = deps.api.addr_validate(&addr)?;
            let status = WHITELIST
                .may_load(deps.storage, &address)?
                .unwrap_or(false);
            to_json_binary(&BoolResponse { value: status })
        }
        QueryMsg::Balance { addr } => {
            let address = deps.api.addr_validate(&addr)?;
            let balance = BALANCES
                .may_load(deps.storage, &address)?
                .unwrap_or(0);
            to_json_binary(&BalanceResponse { balance })
        }
    }
}

/// Transfer tokens between whitelisted addresses
fn transfer(
    deps: DepsMut,
    info: MessageInfo,
    to: String,
    amount: u128,
) -> Result<Response, ContractError> {
    let paused = PAUSED.load(deps.storage)?;
    if paused {
        return Err(ContractError::Paused);
    }

    let to_addr = deps.api.addr_validate(&to)?;
    let from_addr = info.sender;

    let from_ok = WHITELIST
        .may_load(deps.storage, &from_addr)?
        .unwrap_or(false);
    if !from_ok {
        return Err(ContractError::NotWhitelisted);
    }

    let to_ok = WHITELIST
        .may_load(deps.storage, &to_addr)?
        .unwrap_or(false);
    if !to_ok {
        return Err(ContractError::NotWhitelisted);
    }

    let from_bal = BALANCES.may_load(deps.storage, &from_addr)?.unwrap_or(0);
    if from_bal < amount {
        return Err(ContractError::InsufficientBalance);
    }
    BALANCES.save(deps.storage, &from_addr, &(from_bal - amount))?;

    let to_bal = BALANCES.may_load(deps.storage, &to_addr)?.unwrap_or(0);
    BALANCES.save(deps.storage, &to_addr, &(to_bal.saturating_add(amount)))?;

    Ok(Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", from_addr)
        .add_attribute("to", to_addr)
        .add_attribute("amount", amount.to_string()))
}

/// Pause or unpause transfers (issuer only)
fn set_paused(
    deps: DepsMut,
    info: MessageInfo,
    paused: bool,
) -> Result<Response, ContractError> {
    let token = TOKEN_INFO.load(deps.storage)?;

    if info.sender != token.issuer {
        return Err(ContractError::Unauthorized);
    }

    PAUSED.save(deps.storage, &paused)?;

    Ok(Response::new()
        .add_attribute("action", "set_paused")
        .add_attribute("paused", paused.to_string()))
}