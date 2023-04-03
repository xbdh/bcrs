use serde::{Deserialize, Serialize};
use crate::database::BHash;
use anyhow::Result;
use blake3;

pub type Account = String;
// impl Account {
//     pub fn new(s:String) -> Self {
//         Self {
//           s
//         }
//     }
// }

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
    pub fn tx_encode(&self) -> Vec<u8>{
        let tx_str = serde_json::to_string(&self).unwrap();
        tx_str.as_bytes().to_vec()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TxType{
   Normal,
   Reward,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SignedTx{
    pub tx:Tx,
    pub signature:Vec<u8>,
}

impl SignedTx {
    pub fn new(tx: Tx, signature: Vec<u8>) -> Self {
        Self {
            tx,
            signature,
        }
    }
    pub fn sg_tx_hash(&self) -> BHash{
        
        let hash = self.tx.tx_hash();
        hash
    }
    
    pub fn is_authentic(&self) -> bool{
        let tx_hash= self.tx.tx_hash();
        true
    }
}
