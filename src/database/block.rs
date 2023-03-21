use crate::database::tx::Tx;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use blake3;

#[derive( Debug, Clone,Copy)]
pub struct BHash(pub [u8;32]);

impl Serialize for BHash{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = hex::encode(&self.0);
        serializer.serialize_str(&s)
    }
}
impl<'de> Deserialize<'de> for BHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(|err| serde::de::Error::custom(err))?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom(format!("Invalid hash length: {}", bytes.len())));
        }
        let mut array = [0; 32];
        array.copy_from_slice(&bytes);
        Ok(BHash(array))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Tx>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub prev_hash: BHash,
    pub timestamp: u64,
}

impl Block{
    pub fn new(prev_hash: BHash, timestamp: u64, txs: Vec<Tx>) -> Self {
        Self {
            header: BlockHeader {
                prev_hash,
                timestamp,
            },
            txs,
        }
    }
    // 对block进行hash
    pub(crate) fn hash(&self) ->Result<BHash>{
        let block_str = serde_json::to_string(&self)?;
        let hash = blake3::hash(block_str.as_bytes());

        Ok(BHash(hash.into()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub  struct BlockFS{
   pub hash: BHash,
   pub block: Block,
}

impl BlockFS {
    pub fn new(key:BHash ,block: Block) -> Result<Self> {

        Ok(Self {
            hash: key,
            block: block,
        })
    }
}