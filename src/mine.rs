use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::database::{BHash, Block, Tx};
use crate::database::block::is_block_hash_valid;
use anyhow::Result;
use log::info;
use tokio::select;
use crate::database::tx::Account;
use crate::node::{Node, PeerNode};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PendingBlock {
    pub prev_hash: BHash,
    pub  number: u64,
    pub timestamp: u64,
    pub account: Account,
    pub txs: Vec<Tx>,

}

impl PendingBlock{
    pub fn new(prev_hash: BHash, number:u64, account:String,txs: Vec<Tx>) -> Self {
        Self {
            prev_hash,
            number,
            timestamp: chrono::Utc::now().timestamp() as u64,
            account: account,
            txs,
        }
    }
}



fn generate_nonce() -> u64 {
    rand::random()
}


impl Node {
    pub async fn mine(&self)->Result<()>{
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(10));

        loop {
            let node = self.clone();
            let node2 = self.clone();
            let new_sync_block_channel = self.new_sync_block_channel.clone();
            select! {

                _ = ticker.tick() => {
                    info!("********定时器时间到*********");
                    tokio::spawn(async move {
                        let len = node.get_pending_txs().await.len();
                        let is_mining = node.get_is_mining().await;
                        if len > 0 && !is_mining {
                            info!("********start mining*********");
                            node.set_is_mining(true).await;
                            node.mine_pending_txs().await.unwrap();
                            node.set_is_mining(false).await;
                            info!("********end mining*********");
                        }

                      });
                    }
                    // 如果syncblockchannel 获取到了新的区块，并且正在挖矿，就停止挖矿，删除pending txs 中已经挖矿的交易
                    res = new_sync_block_channel.recv()=> {
                      match res{
                        Ok(block) => {
                            info!("从channel 中获取新同步的块 {:?}",block);
                            let is_mining = node2.get_is_mining().await;
                            if is_mining {
                                node2.remove_mined_pending_txs(&block).await;
                                node2.cancel_flag.cancel();
                                info! ("******** mining canceled*********");
                                //node2.cancel_flag.reset();
                                node2.set_is_mining(false).await;
                            }
                            // 如果不在挖矿，并且本地pending tx中有同步的block的交易，就删除
                            else{
                                node2.remove_mined_pending_txs(&block).await;
                            }
                        }
                       Err(_) => {
                            info!("new_sync_block_channel closed");
                        }
                      }

                    }
           }
        }

    }


    // if success, means the block is valid and add to blockchain
    pub async fn mine_pending_txs(&self) -> Result<()> {
        let block_to_mine=PendingBlock::new(
            self.get_last_block_hash().await,
            self.get_next_block_number().await,
            self.info.account.clone(),
            get_txsv_from_txsmp(&self.get_pending_txs().await),
        );
        let node = self.clone();
        let mined_block= tokio::task::spawn_blocking(
             move|| {
                    let block=  node.do_mine(block_to_mine);
                    block
            }
        ).await.unwrap();
      // mined_block.
        //let mined_block= do_mine(block_to_mine);
        match mined_block {
            Ok(block) => {
                self.remove_mined_pending_txs(&block).await?;
                self.add_block(block).await?;
            }
            Err(e) => {
                info!("mine canceled: {}", e);
                self.cancel_flag.reset();
            }
        }
        Ok(())
    }


    pub fn do_mine(&self,pending_block: PendingBlock) -> Result<Block> {
        if pending_block.txs.len() == 0 {

            return Err(anyhow::anyhow!("no txs in pending block"));
        }
        let mut block:Block=Default::default();
        let start = std::time::Instant::now();
        let mut attempts = 0;

        loop {
            if self.cancel_flag.is_canceled() {
                info!("cancel mining");

                return Err(anyhow::anyhow!("cancel mining"));
            }

            let nonce = generate_nonce();
            let b = Block::new(pending_block.prev_hash, pending_block.timestamp, pending_block.number, nonce, pending_block.account.clone(),pending_block.txs.clone());
            let hash = b.hash()?;
            if is_block_hash_valid(&hash) {
                block=b;
                break;
            }
            attempts += 1;
            if attempts % 100000 == 0|| attempts == 1 {
                info!("Mine attempts: {} times, ", attempts);
            }
        }
        info!("Miner is : {}", self.info.account);
        info!("Mine block success, cost: {:?}s", start.elapsed().as_secs_f64());
        info!("Mine block hash: {:?}", hex::encode(block.hash().unwrap().0)) ;
        info!("Mine block nonce: {:?}", block.header.nonce);
        info!("Mine block number: {:?}", block.header.number);
        info!("Mine block prev_hash: {:?}",hex::encode(block.header.prev_hash.0));
        info!("total attempts: {}", attempts);

        Ok(block)
    }

    pub async fn remove_mined_pending_txs(&self, mined_block: &Block) -> Result<()> {
        let pending_txs = self.get_pending_txs().await;
        if mined_block.txs.len() > 0 && pending_txs.len() >0{
            info!("remove_mined_pending_txs");
        }
        // delete mined txs if it exist in pending txs， also add it to archieved txs
        for tx in &mined_block.txs {
            let tx_hash = tx.tx_hash();
            if pending_txs.contains_key(&tx_hash) {
                self.remove_tx_from_pending_txs(tx_hash).await;
                self.add_tx_to_archive_txs(tx.clone()).await;
            }
        }


        Ok(())
    }

    pub async fn add_pending_tx(&self, tx: Tx,from_peer:&PeerNode) -> Result<()> {
        let tx_hash = tx.tx_hash();
        let pending_txs = self.get_pending_txs().await;
        let archive_txs = self.get_archive_txs().await;

        if !pending_txs.contains_key(&tx_hash) && !archive_txs.contains_key(&tx_hash) {
            info!("add_pending_tx from peer: {:?}",from_peer);
            self.add_tx_to_pending_txs(tx.clone()).await;
        }
        Ok(())
    }

}

// 在区块链挖矿过程中，这三个hashmap类型的变量代表着不同阶段的交易记录。
//
// mined_txs（已挖掘的交易记录）：这个hashmap保存着已经被矿工打包进区块的交易记录。
// 当矿工成功挖出一个新的区块时，这些交易记录会从pending_txs中被移除并添加到mined_txs中。
//
// pending_txs（待确认的交易记录）：这个hashmap保存着尚未被打包进区块的交易记录。
// 当用户发起一个交易请求后，这个交易会被添加到pending_txs中，等待被矿工打包进区块。
//
// archive_txs（存档交易记录）：这个hashmap保存着已经被打包进区块并且被确认的交易记录。
// 通常情况下，archive_txs只是mined_txs的历史记录，
// 但是在某些情况下，矿工可能会选择清除已经确认的交易记录，此时这些记录会被移动到archive_txs中，
// 以便于后续的分析和审计。
//
// 这三个hashmap之间的转换过程通常是由矿工或节点的软件来处理的。
// 当矿工挖出一个新的区块时，他们会将pending_txs中的交易记录移动到mined_txs中，并广播这个新区块的信息给整个网络。
// 其他节点收到这个新区块后，也会同步更新他们自己的交易记录。
// 当确认的交易记录不再需要保留时，矿工或节点的软件可能会将这些记录从mined_txs中移动到archive_txs中。

// hashmap<BHash,Tx> zhuanhuan wei vec<Tx>
pub fn get_txsv_from_txsmp(pending_txs: &HashMap<BHash, Tx>) -> Vec<Tx> {
    let mut txs: Vec<Tx> = Vec::new();
    for (_, tx) in pending_txs {
        txs.push(tx.clone());
    }
    txs
}
