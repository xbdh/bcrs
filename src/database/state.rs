use std::collections::HashMap;
use std::fs::{OpenOptions, File};
use std::io::{BufRead, BufReader, Write};
use std::ops::Deref;
use crate::database::tx::{Account,Tx};
use anyhow::Result;
use log::info;
use crate::database::block::{BHash, Block, BlockFS};
use crate::database::init_genesis;
use crate::database::tx::TxType;
use parking_lot::RwLock;

#[derive(Debug)]
pub struct Status{
    balances: HashMap<Account,u64>,
    tx_mem_pool: Vec<Tx>,
    db_file:File,
    last_block_hash: BHash,
}

impl Status {
   pub fn new() -> Result<Self> {
       let genesis = init_genesis();
       let mut bs=HashMap::new();
       for (k,v) in genesis.balances {
           bs.insert(k, v);
       }
       let tx_mem_pool = Vec::new();
       // 可读可写的方式打开文件

       let db_file = OpenOptions::new()
           .read(true)
           .write(true)
           .create(true)
           .append(true)
           .open("./db/block.db")?;

       let mut status = Status {
           balances: bs,
           tx_mem_pool,
           db_file: db_file,
           last_block_hash: BHash([0;32]),
       };

       let mut blocks=Vec::new();
       let bufreader = BufReader::new(&status.db_file);
       for line in bufreader.lines() {
           let line = line.unwrap();
           let block_fs :BlockFS=serde_json::from_str(&line).unwrap();
           let block = block_fs.block;
           //status.apply_block(block); // mutable borrow happens here
              blocks.push(block);
       }

       for block in blocks {
           status.add_block(block);
       }
        Ok(status)

    }


    pub fn add_tx(&mut self, tx: Tx) -> Result<()> {
        self.apply_tx(tx.clone())?;
        self.tx_mem_pool.push(tx);

        Ok(())
    }
    pub fn apply_tx(&mut self, tx: Tx) -> Result<()> {
        info!("apply tx: {:?}", tx);
        match tx.tx_type {
            TxType::Normal => {
                //两次mut borrow所以用if let
               if let Some(from_balance) = self.balances.get_mut(&tx.from){
                   if *from_balance < tx.value {
                       return Err(anyhow::anyhow!("insufficient balance"));
                   }
                    *from_balance -= tx.value;
                }
               // if  let Some(to_balance) = self.balances.get_mut(&tx.to) {
               //     *to_balance += tx.value;
               // }
                self.balances.entry(tx.to).and_modify(|e| *e += tx.value).or_insert(tx.value);

            }
            TxType::Reward => {
                if let Some(to_balance) = self.balances.get_mut(&tx.to){
                    *to_balance += tx.value;
                }
            }
        }

        Ok(())
    }
    pub fn persist(&mut self) -> Result<BHash> {
        info!("persisting block" );
        let time_now = std::time::SystemTime::now();
        // 将时间转换为时间戳
        let since_the_epoch = time_now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards").as_secs();

        let block = Block::new(self.last_block_hash,since_the_epoch , self.tx_mem_pool.clone());
        let block_hash = block.hash()?;
        let block_fs= BlockFS::new(block_hash,block)?;
        let block_str = serde_json::to_string(&block_fs)?;
        let line = format!("{}\n", block_str);
        //info!("persisting block: {:?}", line);
        self.db_file.write_all(line.as_bytes()).map_err(|e| anyhow::anyhow!("failed to write to db file: {}", e))?;

        self.last_block_hash = block_hash;
        self.tx_mem_pool.clear();

        Ok(block_hash)
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        info!("apply block ,txs len: {:?}", block.txs.len());
        for tx in block.txs {
            self.add_tx(tx)?;
        }
        Ok(())
    }
    pub fn get_balance(&self) -> HashMap<Account,u64> {
        self.balances.clone()
    }

    pub fn get_last_block_hash(&self) -> BHash {
        self.last_block_hash
    }
}
