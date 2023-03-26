use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use crate::database::tx::Tx;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use blake3;

#[derive( Debug, Clone,Copy,Default,Eq,Hash)]
pub struct BHash(pub [u8;32]);

impl PartialEq for BHash {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
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

#[derive(Serialize, Deserialize, Debug, Clone,Default)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Tx>,
}

#[derive(Serialize, Deserialize, Debug, Clone,Default)]
pub struct BlockHeader {
    pub prev_hash: BHash,
    pub number: u64,
    pub nonce: u64,
    pub timestamp: u64,
}

impl Block{
    pub fn new(prev_hash: BHash, timestamp: u64, number:u64, nonce :u64,txs: Vec<Tx>) -> Self {
        Self {
            header: BlockHeader {
                prev_hash,
                number,
                nonce,
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
// get all blocks after hash ,not include hash
pub fn get_blocks_after_hash(hash: BHash,dir_path:String) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let db_path = dir_path + "/block.db";
    let db_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(db_path)?;

    let bufreader = BufReader::new(db_file);
    for line in bufreader.lines() {
        let line = line.unwrap();
        let block_fs :BlockFS=serde_json::from_str(&line).unwrap();
        let block = block_fs.block;
        // 当hash为默认值时，返回所有的block

        if hash==Default::default() {
            blocks.push(block.clone());
            continue;
        }
        if block.header.prev_hash == hash {
            blocks.push(block.clone());
        }
    }
    Ok(blocks)
}

pub fn is_block_hash_valid(hash: &BHash) ->bool {
    let mut hash_array = hash.0;
    let hash_str = hex::encode(hash_array);

    if hash_str.starts_with("0000") {
        return true;
    }
    false
}