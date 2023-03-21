use serde::{Deserialize, Serialize};

pub type Account = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tx{
    pub from:Account,
    pub to:Account,
    pub value:u64,
    pub tx_type:TxType,
}

impl Tx {
    pub fn new(from: Account, to: Account, value: u64, tx_type: TxType) -> Self {
        Self {
            from,
            to,
            value,
            tx_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TxType{
   Normal,
   Reward,
}