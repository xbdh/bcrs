use std::collections::HashMap;
use std::fs::{OpenOptions, File};
use std::io::{BufRead, BufReader, Write};
use crate::database::tx::{Account,Tx};
use anyhow::Result;
use log::info;
use crate::database::block::{BHash, Block, BlockFS};
use crate::database::init_genesis;
use crate::database::tx::TxType;

#[derive(Debug)]
pub struct BStatus {
    balances: HashMap<Account,u64>,
    tx_mem_pool: Vec<Tx>,
    db_file:File,
    pub(crate) last_block_hash: BHash,
    last_block :Block,
    // is_genesis: bool, // block.db是否一个块都没有
}

impl Clone for BStatus{
    fn clone(&self) -> Self {
        BStatus {
            balances: self.balances.clone(),
            tx_mem_pool: self.tx_mem_pool.clone(),
            db_file: self.db_file.try_clone().unwrap(),
            last_block_hash: self.last_block_hash.clone(),
            last_block: self.last_block.clone(),
            // is_genesis: self.is_genesis,
        }
    }
}

impl BStatus {
   pub fn new(dir_path:String) -> Result<Self> {
       let genesis = init_genesis();
       let mut bs=HashMap::new();
       for (k,v) in genesis.balances {
           bs.insert(k, v);
       }
       let tx_mem_pool = Vec::new();
       // 可读可写的方式打开文件

       let db_path = dir_path + "/block.db";
       let db_file = OpenOptions::new()
           .read(true)
           .write(true)
           .create(true)
           .append(true)
           .open(db_path)?;

       let mut status = BStatus {
           balances: bs,
           tx_mem_pool,
           db_file: db_file,
           last_block_hash: Default::default(),
           last_block: Default::default(),
           // is_genesis: true,
       };

       let mut blocks=Vec::new();
       let bufreader = BufReader::new(&status.db_file);
       for line in bufreader.lines() {
           let line = line.unwrap();
           let block_fs :BlockFS=serde_json::from_str(&line).unwrap();
           let block = block_fs.block;
           //status.apply_block(block); // mutable borrow happens here
           blocks.push(block.clone());

           status.last_block = block;
           status.last_block_hash = block_fs.hash;
       }
       info!("block.len:{}",blocks.len());
       for block in blocks {
           apply_txs(block.txs, &mut status)?;
       }
       let hstr=serde_json::to_string(&status.last_block_hash).unwrap().trim_matches('"').to_string();
       info!("init status ok ,last block hash: {:?},block height:{}", hstr,status.get_height());
        Ok(status)

    }

    pub fn add_block(&mut self, block: Block) -> Result<BHash> {
        info !("add block: {:?}", block);
        apply_block( block.clone(),self)?;
        let block_hash = block.hash()?;

        let block_fs = BlockFS {
            hash: block_hash.clone(),
            block,
        };
        let block_str = serde_json::to_string(&block_fs)?;
        let line= block_str + "\n";
        self.db_file.write_all(line.as_bytes())?;


        //self.balances = pending_state.balances;
        self.last_block_hash = block_hash.clone();
        self.last_block = block_fs.block;

        Ok(block_hash)
    }

    // fn next_block_number(&self) -> u64 {
    //     if self.is_genesis {
    //         return 0;
    //     }
    //     self.last_block.header.number + 1
    // }

    pub fn get_balance(&self) -> HashMap<Account,u64> {
        self.balances.clone()
    }

    pub fn get_last_block_hash(&self) -> BHash {
        self.last_block_hash
    }

    pub fn get_last_block(&self) -> Block {
        self.last_block.clone()
    }

    pub fn get_height(&self) -> u64 {
        self.last_block.header.number
    }
}

// 对新加入的块进行验证，验证通过后，将交易加入到内存池中
// 这个块可能是新块，也可能是从别的节点同步过来的块，是第一块
fn apply_block(block: Block,bstatus:&mut BStatus) -> Result<()> {
    // let next_expected_block_number = bstatus.next_block_number();
    //
    // if block.header.number != next_expected_block_number {
    //     return Err(anyhow::anyhow!("invalid block number"));
    // }
    // if bstatus.get_height()>0&&  block.header.prev_hash != bstatus.get_last_block_hash() {
    //     return Err(anyhow::anyhow!("invalid prev block hash"));
    // }

    apply_txs(block.txs,bstatus)?;
    Ok(())
}

fn apply_txs(txs: Vec<Tx>, bstatus: &mut BStatus) -> Result<()> {
    for tx in txs {
        apply_tx(tx,bstatus)?;
    }
    Ok(())
}

fn apply_tx(tx: Tx, bstatus: &mut BStatus) -> Result<()> {
    match tx.tx_type {
        TxType::Normal => {
            if let Some(from_balance) = bstatus.balances.get_mut(&tx.from) {
                if *from_balance < tx.value {
                    return Err(anyhow::anyhow!("insufficient balance"));
                }
                *from_balance -= tx.value;
            }
            bstatus.balances.entry(tx.to).and_modify(|e| *e += tx.value).or_insert(tx.value);
        }
        TxType::Reward => {
            if let Some(to_balance) = bstatus.balances.get_mut(&tx.to) {
                *to_balance += tx.value;
            }
        }
    }
    Ok(())
}