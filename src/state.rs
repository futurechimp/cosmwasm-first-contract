use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, HumanAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

// configuration instance key. config object will be saved under this key.
pub static CONFIG_KEY: &[u8] = b"config";

// contract state structure, this will be saved.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub creator: HumanAddr,
    pub owner: HumanAddr,
    pub collateral: Vec<Coin>,
    pub counter_offer: Vec<Coin>,
    pub expires: u64,
}

// returns a bucket to read/write to store.
pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

// returns a readonly bucket only read store.
// Safer to use read_only when no need to write like querying.
pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

// this is optional.
// ConfigResponse as returned as response structure to query_config.
// good to have aliasing.
pub type ConfigResponse = State;
