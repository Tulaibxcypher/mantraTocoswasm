use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub issuer: Addr,
    pub total_supply: u128,
    pub max_supply: u128,
}

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const PAUSED: Item<bool> = Item::new("paused");
pub const BALANCES: Map<&Addr, u128> = Map::new("balances");
pub const WHITELIST: Map<&Addr, bool> = Map::new("whitelist");
