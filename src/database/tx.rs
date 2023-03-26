use serde::{Deserialize, Serialize};
use crate::database::BHash;
use anyhow::Result;
use blake3;
pub type Account = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tx{
    pub from:Account,
    pub to:Account,
    pub value:u64,
    pub timestamp:u64,
    pub tx_type:TxType,
}

impl Tx {
    pub fn new(from: Account, to: Account, value: u64, tx_type: TxType) -> Self {
        Self {
            from,
            to,
            value,
            timestamp: chrono::Utc::now().timestamp() as u64,
            tx_type,
        }
    }
    pub fn tx_hash(&self) -> BHash{
        let tx_str = serde_json::to_string(&self).unwrap();
        let hash = blake3::hash(tx_str.as_bytes());
        BHash(hash.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TxType{
   Normal,
   Reward,
}