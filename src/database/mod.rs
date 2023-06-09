pub mod bstate;
pub mod tx;
pub mod block;
pub  mod purems;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::database::tx::Account;
use config::Config;

pub use block::Block;
pub use tx::Tx;
pub use bstate::BStatus;
pub use tx::TxType;
pub use block::BHash;
pub use purems::Pure;


#[derive(Serialize, Deserialize,Debug, Clone)]
pub struct Genesis {
    genesis_time: String,
    chain_id: String,
    pub balances: HashMap<Account, u64>,
}

pub fn init_genesis() -> Genesis {
    let settings = Config::builder()
        // Add in `./genesis.json`
        .add_source(config::File::with_name("./db/genesis"))
        .build()
        .unwrap();
    
    settings.try_deserialize::<Genesis>().unwrap()

}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_genesis() {
        let genesis = init_genesis();
        println!("\n{:?}", genesis.balances);

    }
}